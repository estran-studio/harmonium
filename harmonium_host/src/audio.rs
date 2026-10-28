use std::sync::{Arc, Mutex, atomic::AtomicU64};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use harmonium_audio::backend::{
    AudioRenderer,
    recorder::{OverflowPolicy, RecorderBackend, RecorderWorker},
    synth_backend::SynthBackend,
};
use harmonium_core::{host_resources::HostResources, log};
use web_time::Instant;

use crate::{
    clock::{ClockReader, TransportClock},
    composer::MusicComposer,
    playback::{
        MEASURE_RING_CAP, PlaybackCommand, PlaybackEngine, RETURN_RING_CAP, TaggedMeasure, pack,
    },
};

/// Control side of an engine: everything the main thread drives, the
/// [`PlaybackEngine`] (audio side) being handed to a stream or to the caller.
pub struct EngineControls {
    /// Main-thread composer (direct calls, no queue).
    pub composer: MusicComposer,
    /// Commands to the playback engine.
    pub commands: rtrb::Producer<PlaybackCommand>,
    /// Reports published by the playback engine.
    pub reports: rtrb::Consumer<harmonium_core::EngineReport>,
    /// Packed transport position `(bar, step)`, written by the playback
    /// engine and read lock-free.
    pub transport_position: Arc<AtomicU64>,
    /// Lock-free reader of the published [`TransportClock`].
    pub clock_reader: ClockReader,
    /// Completed recordings.
    pub finished_recordings: crate::FinishedRecordings,
    /// Encoder of the recordings when no thread runs it (wasm): the host
    /// advances it from [`LiveHost::pump`](crate::LiveHost::pump). `None` when
    /// the recorder has its own thread.
    pub recorder: Option<RecorderWorker>,
}

/// Build the whole engine graph at `sample_rate`: the control side and the
/// playback engine that renders it. No audio device is touched.
///
/// `overflow` is the recorder's policy: real time drops rather than blocks
/// the audio thread; offline blocks, so recordings are complete.
pub(crate) fn build_engine(
    sf2_bytes: Option<&[u8]>,
    feet_sf2_bytes: Option<&[u8]>,
    odin_presets: &[harmonium_audio::odin_source::OdinPresetSource],
    sample_rate: f64,
    overflow: OverflowPolicy,
) -> (EngineControls, PlaybackEngine) {
    // Command/report ring buffers (lock-free for audio thread)
    let (commands, playback_cmd_rx) = rtrb::RingBuffer::<PlaybackCommand>::new(256);
    let (report_tx, reports) = rtrb::RingBuffer::<harmonium_core::EngineReport>::new(256);

    // Handoff rings: composer → playback staged measures, playback →
    // composer returned boxes (deallocated on the main thread).
    let (measure_tx, measure_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(MEASURE_RING_CAP);
    let (return_tx, return_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(RETURN_RING_CAP);
    // Shared transport position, packed (bar, step) — initial (1, 0)
    let transport_position = Arc::new(AtomicU64::new(pack(1, 0)));

    // Predictive clock: single writer (audio callback) → lock-free reader.
    let initial_clock = TransportClock::initial(sample_rate as u32);
    let (clock_tx, clock_output) = triple_buffer::TripleBuffer::new(&initial_clock).split();
    let clock_reader = ClockReader::from_output(clock_output);

    // Create renderer: every logical channel on SoundFont bank 0
    let default_routing = [0; harmonium_core::timeline::TrackId::ALL.len()];
    let synth = SynthBackend::new(sample_rate, sf2_bytes, feet_sf2_bytes, &default_routing);
    // Ce que l'hôte a réellement chargé : le compositeur en tire les presets
    // Odin morphés et prévient quand un profil désigne autre chose.
    let resources = HostResources {
        percussion_banks: synth.percussion_banks().to_vec(),
        odin: harmonium_audio::odin_bank::load_bank(odin_presets),
    };
    let inner_backend: Box<dyn AudioRenderer> = Box::new(synth);

    let finished_recordings = Arc::new(Mutex::new(Vec::new()));
    let (recorder_backend, recorder) =
        build_recorder(inner_backend, finished_recordings.clone(), sample_rate as u32, overflow);
    let recorder_backend = Box::new(recorder_backend);

    // NOTE: No pre-generation here. The caller sets params first, then
    // generates: pre-generating with defaults would produce bars that don't
    // match the user's settings.
    let mut composer =
        MusicComposer::new(sample_rate, measure_tx, return_rx, transport_position.clone());
    composer.set_host_resources(resources);

    let playback = PlaybackEngine::new(
        sample_rate,
        recorder_backend,
        measure_rx,
        return_tx,
        playback_cmd_rx,
        report_tx,
        transport_position.clone(),
        clock_tx,
    );

    (
        EngineControls {
            composer,
            commands,
            reports,
            transport_position,
            clock_reader,
            finished_recordings,
            recorder,
        },
        playback,
    )
}

/// The recorder of an engine. Native: its encoder has a thread, and `overflow`
/// decides what a full queue does.
#[cfg(not(target_arch = "wasm32"))]
fn build_recorder(
    inner: Box<dyn AudioRenderer>,
    finished: crate::FinishedRecordings,
    sample_rate: u32,
    overflow: OverflowPolicy,
) -> (RecorderBackend, Option<RecorderWorker>) {
    (RecorderBackend::with_policy(inner, finished, sample_rate, overflow), None)
}

/// The recorder of an engine. Wasm has no thread: the encoder goes to the host,
/// which advances it from [`LiveHost::pump`](crate::LiveHost::pump). The
/// callback never blocks, so a full queue always drops and counts, whatever
/// `overflow` says.
#[cfg(target_arch = "wasm32")]
fn build_recorder(
    inner: Box<dyn AudioRenderer>,
    finished: crate::FinishedRecordings,
    sample_rate: u32,
    _overflow: OverflowPolicy,
) -> (RecorderBackend, Option<RecorderWorker>) {
    let (backend, worker) = RecorderBackend::pumped(inner, finished, sample_rate);
    (backend, Some(worker))
}

/// Create a decoupled timeline stream for real-time playback: the engine
/// graph of [`build_engine`] with its playback engine moved into a `cpal`
/// output stream (already playing).
///
/// `buffer_frames` fixes the size of the device buffer in frames; `None` keeps
/// the platform default.
pub fn create_timeline_stream(
    sf2_bytes: Option<&[u8]>,
    feet_sf2_bytes: Option<&[u8]>,
    odin_presets: &[harmonium_audio::odin_source::OdinPresetSource],
    buffer_frames: Option<u32>,
) -> Result<(cpal::Stream, EngineControls), String> {
    let host = cpal::default_host();
    let device =
        host.default_output_device().ok_or_else(|| "No output device found".to_string())?;

    let config = device.default_output_config().map_err(|e| e.to_string())?;
    let sample_rate = config.sample_rate().0 as f64;
    let channels = config.channels() as usize;
    let mut stream_config: cpal::StreamConfig = config.into();
    if let Some(frames) = buffer_frames {
        stream_config.buffer_size = cpal::BufferSize::Fixed(frames);
    }

    log::info(&format!("Decoupled Engine - Sample rate: {}, Channels: {}", sample_rate, channels));

    // Playback engine: moved into the CPAL closure.
    let (controls, mut playback) = build_engine(
        sf2_bytes,
        feet_sf2_bytes,
        odin_presets,
        sample_rate,
        OverflowPolicy::DropAndCount,
    );

    let err_fn = |err| log::error(&format!("an error occurred on stream: {}", err));

    let stream = device
        .build_output_stream(
            &stream_config,
            move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                // Audible instant of this buffer's first frame: callback
                // entry time + (playback − callback) device latency. Falls
                // back to no offset when the timestamps don't line up
                // (documented degraded path, never silent).
                let now = Instant::now();
                let ts = info.timestamp();
                let buffer_start_instant =
                    ts.playback.duration_since(&ts.callback).map_or(now, |offset| now + offset);
                playback.process_buffer(data, channels, buffer_start_instant);
            },
            err_fn,
            None,
        )
        .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;

    Ok((stream, controls))
}

/// Create a timeline engine for offline (non-realtime) rendering.
///
/// No audio device is opened. The caller drives `playback.process_buffer()`
/// in a tight loop to render as fast as possible. Native only: the recorder
/// blocks on its encoder thread, and wasm has none (use
/// [`LiveHost::detached`](crate::LiveHost::detached), whose `pump` drives the
/// encoder).
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::type_complexity)]
pub fn create_offline_engine(
    sf2_bytes: Option<&[u8]>,
    feet_sf2_bytes: Option<&[u8]>,
    odin_presets: &[harmonium_audio::odin_source::OdinPresetSource],
    sample_rate: f64,
) -> Result<
    (
        MusicComposer,
        PlaybackEngine,
        rtrb::Producer<PlaybackCommand>,
        rtrb::Consumer<harmonium_core::EngineReport>,
        ClockReader,
        crate::FinishedRecordings,
    ),
    String,
> {
    // Offline there is no deadline: block rather than drop, so recordings are
    // complete and published when the stop command returns.
    let (controls, playback) =
        build_engine(sf2_bytes, feet_sf2_bytes, odin_presets, sample_rate, OverflowPolicy::Block);
    Ok((
        controls.composer,
        playback,
        controls.commands,
        controls.reports,
        controls.clock_reader,
        controls.finished_recordings,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoupled_engine_produces_audio() {
        let sample_rate = 44100.0;
        let channels = 2;

        let (mut composer, mut playback, mut _cmd_tx, mut _report_rx, _clock_reader, _recordings) =
            create_offline_engine(None, None, &[], sample_rate)
                .expect("create_offline_engine failed");

        // Pre-generate bars
        composer.set_writehead_lookahead(16);
        composer.generate_bars(8);

        // Process audio buffers
        let mut buffer = vec![0.0f32; 1024 * channels];
        let mut total_energy = 0.0f64;

        for _ in 0..500 {
            composer.generate_ahead();
            for s in buffer.iter_mut() {
                *s = 0.0;
            }
            playback.process_buffer(&mut buffer, channels, Instant::now());
            total_energy += buffer.iter().map(|s| (*s as f64) * (*s as f64)).sum::<f64>();
        }

        assert!(
            total_energy > 0.0,
            "Decoupled engine produced zero audio output after 500 buffers"
        );
    }

    #[test]
    fn test_invalidate_after_preview_preserves_bars() {
        let sample_rate = 44100.0;

        let (mut composer, _playback, mut _cmd_tx, mut _report_rx, _clock_reader, _recordings) =
            create_offline_engine(None, None, &[], sample_rate)
                .expect("create_offline_engine failed");

        // Generate 8 bars with default params
        composer.set_writehead_lookahead(16);
        composer.generate_bars(8);

        // Snapshot the first 4 bars (preview window) from the timeline
        let preview_bars: Vec<_> = (1..=4)
            .map(|i| composer.timeline_measure(i).expect("bar should exist").clone())
            .collect();

        // Change BPM significantly (default is 120, change to 180)
        composer.set_bpm(180.0);

        // Invalidate after 4 bars of preview (playhead is at bar 1 = default AtomicUsize)
        // This should keep bars 1-4 and regenerate 5+
        composer.invalidate_after_preview(4);

        // Verify preview bars are exactly the same
        for (i, original) in preview_bars.iter().enumerate() {
            let bar_idx = i + 1;
            let current =
                composer.timeline_measure(bar_idx).expect("preview bar should still exist");
            assert_eq!(original.index, current.index, "Preview bar {} index changed", bar_idx);
            assert_eq!(
                original.tempo, current.tempo,
                "Preview bar {} tempo should be unchanged (still old BPM)",
                bar_idx
            );
        }

        // Generate new bars beyond the preview (lookahead is still 16 from earlier)
        composer.generate_ahead();

        // Verify post-preview bars use new BPM (180)
        for bar_idx in 5..=8 {
            let measure =
                composer.timeline_measure(bar_idx).expect("post-preview bar should exist");
            assert!(
                (measure.tempo - 180.0).abs() < 1.0,
                "Post-preview bar {} should have new BPM ~180, got {}",
                bar_idx,
                measure.tempo
            );
        }
    }

    #[test]
    fn test_bpm_override_persists_through_emotion_change() {
        let sample_rate = 44100.0;

        let (mut composer, _playback, mut _cmd_tx, mut _report_rx, _clock_reader, _recordings) =
            create_offline_engine(None, None, &[], sample_rate)
                .expect("create_offline_engine failed");

        // Set explicit BPM override
        composer.set_bpm(140.0);
        assert!(
            (composer.musical_params().bpm - 140.0).abs() < 0.01,
            "BPM should be 140 after set_bpm"
        );

        // Switch to emotion mode and set emotions (which would normally change BPM)
        composer.use_emotion_mode();
        composer.set_emotions(0.9, 0.5, 0.7, 0.6);

        // BPM should still be 140 because of the override
        assert!(
            (composer.musical_params().bpm - 140.0).abs() < 0.01,
            "BPM should still be 140 after emotion change, got {}",
            composer.musical_params().bpm
        );
    }

    #[test]
    fn test_reset_bpm_reverts_to_emotion_mapped() {
        let sample_rate = 44100.0;

        let (mut composer, _playback, mut _cmd_tx, mut _report_rx, _clock_reader, _recordings) =
            create_offline_engine(None, None, &[], sample_rate)
                .expect("create_offline_engine failed");

        // Switch to emotion mode and set emotions to establish a mapped BPM
        composer.use_emotion_mode();
        composer.set_emotions(0.9, 0.5, 0.7, 0.6);
        let emotion_bpm = composer.musical_params().bpm;

        // Override with explicit BPM
        composer.set_bpm(140.0);
        assert!(
            (composer.musical_params().bpm - 140.0).abs() < 0.01,
            "BPM should be 140 after override"
        );

        // Reset BPM override — should revert to emotion-mapped value
        composer.reset_bpm();
        assert!(
            (composer.musical_params().bpm - emotion_bpm).abs() < 0.01,
            "BPM should revert to emotion-mapped {} after reset, got {}",
            emotion_bpm,
            composer.musical_params().bpm
        );
    }

    #[test]
    fn test_set_bpm_without_emotion_mode() {
        let sample_rate = 44100.0;

        let (mut composer, _playback, mut _cmd_tx, mut _report_rx, _clock_reader, _recordings) =
            create_offline_engine(None, None, &[], sample_rate)
                .expect("create_offline_engine failed");
        let nominal_bpm = composer.profile().meter.default_bpm;

        // In direct mode, set_bpm should work normally
        composer.set_bpm(155.0);
        assert!((composer.musical_params().bpm - 155.0).abs() < 0.01, "BPM should be 155");

        // Reset BPM restores the active profile's nominal tempo.
        composer.reset_bpm();
        assert!(
            (composer.musical_params().bpm - nominal_bpm).abs() < 0.01,
            "BPM should revert to the profile tempo after reset, got {}",
            composer.musical_params().bpm
        );
    }
}

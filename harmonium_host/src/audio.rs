use std::sync::{Arc, Mutex, atomic::AtomicUsize};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "odin2")]
use harmonium_audio::backend::odin2_backend::Odin2Backend;
use harmonium_audio::backend::{
    AudioRenderer, recorder::RecorderBackend, synth_backend::SynthBackend,
};
use harmonium_core::{log, params::SessionConfig};

use crate::{
    composer::MusicComposer,
    playback::{PlaybackCommand, PlaybackEngine},
};

/// Available audio backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioBackendType {
    /// FundSP + Oxisynth backend (default)
    #[default]
    FundSP,
    /// Odin2 synthesizer backend
    #[cfg(feature = "odin2")]
    Odin2,
}

/// Create a decoupled timeline stream for real-time playback.
///
/// Returns:
/// - `cpal::Stream` — audio output stream
/// - `SessionConfig` — session key/scale/bpm
/// - `Mutex<MusicComposer>` — main-thread composer (direct calls, no queue)
/// - `rtrb::Producer<PlaybackCommand>` — send commands to playback engine
/// - `rtrb::Consumer<EngineReport>` — receive reports from playback engine
/// - `FontQueue` — SoundFont loading queue
/// - `FinishedRecordings` — completed recordings
#[allow(clippy::type_complexity)]
pub fn create_timeline_stream(
    sf2_bytes: Option<&[u8]>,
    backend_type: AudioBackendType,
) -> Result<
    (
        cpal::Stream,
        SessionConfig,
        Mutex<MusicComposer>,
        rtrb::Producer<PlaybackCommand>,
        rtrb::Consumer<harmonium_core::EngineReport>,
        crate::FontQueue,
        crate::FinishedRecordings,
    ),
    String,
> {
    let host = cpal::default_host();
    let device =
        host.default_output_device().ok_or_else(|| "No output device found".to_string())?;

    let config = device.default_output_config().map_err(|e| e.to_string())?;
    let sample_rate = config.sample_rate().0 as f64;
    let channels = config.channels() as usize;

    log::info(&format!("Decoupled Engine - Sample rate: {}, Channels: {}", sample_rate, channels));

    // Command/report ring buffers (lock-free for audio thread)
    let (playback_cmd_tx, playback_cmd_rx) = rtrb::RingBuffer::<PlaybackCommand>::new(256);
    let (report_tx, report_rx) = rtrb::RingBuffer::<harmonium_core::EngineReport>::new(256);

    // Shared pages: composer writes by index, playback reads by index
    let shared_pages: crate::SharedPages = Arc::new(Mutex::new(Vec::with_capacity(64)));

    // Shared playhead position
    let playhead_bar = Arc::new(AtomicUsize::new(1));

    // Font queue (shared between NativeHandle and PlaybackEngine)
    let font_queue = Arc::new(std::sync::Mutex::new(Vec::new()));

    // Create renderer
    // Route to Oxisynth only when a SoundFont is loaded; otherwise use FundSP
    let default_routing = if sf2_bytes.is_some() { vec![0, 1, 2, 3] } else { vec![-1, -1, -1, -1] };
    let inner_backend: Box<dyn AudioRenderer> = match backend_type {
        AudioBackendType::FundSP => {
            Box::new(SynthBackend::new(sample_rate, sf2_bytes, &default_routing))
        }
        #[cfg(feature = "odin2")]
        AudioBackendType::Odin2 => Box::new(Odin2Backend::new(sample_rate)),
    };

    let finished_recordings = Arc::new(Mutex::new(Vec::new()));
    let recorder_backend = Box::new(RecorderBackend::new(
        inner_backend,
        finished_recordings.clone(),
        sample_rate as u32,
    ));

    // Create composer (main thread)
    let composer = MusicComposer::new(
        sample_rate,
        shared_pages.clone(),
        playhead_bar.clone(),
        font_queue.clone(),
    );
    let session_config = composer.config.clone();

    // NOTE: No pre-generation here. The caller (NativeHandle::start → initialize_with_config)
    // sets params first, then generates. Pre-generating with defaults would produce
    // bars that don't match the user's saved settings.

    // Create playback engine (will be moved into CPAL closure)
    let mut playback = PlaybackEngine::new(
        sample_rate,
        recorder_backend,
        shared_pages,
        playback_cmd_rx,
        report_tx,
        playhead_bar,
    );

    let err_fn = |err| log::error(&format!("an error occurred on stream: {}", err));

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                playback.process_buffer(data, channels);
            },
            err_fn,
            None,
        )
        .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;

    let composer_mutex = Mutex::new(composer);

    Ok((
        stream,
        session_config,
        composer_mutex,
        playback_cmd_tx,
        report_rx,
        font_queue,
        finished_recordings,
    ))
}

/// Create a timeline engine for offline (non-realtime) rendering.
///
/// No audio device is opened. The caller drives `playback.process_buffer()`
/// in a tight loop to render as fast as possible.
#[allow(clippy::type_complexity)]
pub fn create_offline_engine(
    sf2_bytes: Option<&[u8]>,
    backend_type: AudioBackendType,
    sample_rate: f64,
) -> Result<
    (
        MusicComposer,
        PlaybackEngine,
        rtrb::Producer<PlaybackCommand>,
        rtrb::Consumer<harmonium_core::EngineReport>,
        crate::FinishedRecordings,
    ),
    String,
> {
    let (playback_cmd_tx, playback_cmd_rx) = rtrb::RingBuffer::<PlaybackCommand>::new(256);
    let (report_tx, report_rx) = rtrb::RingBuffer::<harmonium_core::EngineReport>::new(256);

    let shared_pages: crate::SharedPages = Arc::new(Mutex::new(Vec::with_capacity(64)));
    let playhead_bar = Arc::new(AtomicUsize::new(1));
    let font_queue = Arc::new(std::sync::Mutex::new(Vec::new()));

    let default_routing = if sf2_bytes.is_some() { vec![0, 1, 2, 3] } else { vec![-1, -1, -1, -1] };
    let inner_backend: Box<dyn AudioRenderer> = match backend_type {
        AudioBackendType::FundSP => {
            Box::new(SynthBackend::new(sample_rate, sf2_bytes, &default_routing))
        }
        #[cfg(feature = "odin2")]
        AudioBackendType::Odin2 => Box::new(Odin2Backend::new(sample_rate)),
    };

    let finished_recordings = Arc::new(Mutex::new(Vec::new()));
    let recorder_backend = Box::new(RecorderBackend::new(
        inner_backend,
        finished_recordings.clone(),
        sample_rate as u32,
    ));

    let composer =
        MusicComposer::new(sample_rate, shared_pages.clone(), playhead_bar.clone(), font_queue);

    let playback = PlaybackEngine::new(
        sample_rate,
        recorder_backend,
        shared_pages,
        playback_cmd_rx,
        report_tx,
        playhead_bar,
    );

    Ok((composer, playback, playback_cmd_tx, report_rx, finished_recordings))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoupled_engine_produces_audio() {
        let sample_rate = 44100.0;
        let channels = 2;

        let (mut composer, mut playback, mut _cmd_tx, mut _report_rx, _recordings) =
            create_offline_engine(None, AudioBackendType::FundSP, sample_rate)
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
            playback.process_buffer(&mut buffer, channels);
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

        let (mut composer, _playback, mut _cmd_tx, mut _report_rx, _recordings) =
            create_offline_engine(None, AudioBackendType::FundSP, sample_rate)
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

        let (mut composer, _playback, mut _cmd_tx, mut _report_rx, _recordings) =
            create_offline_engine(None, AudioBackendType::FundSP, sample_rate)
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

        let (mut composer, _playback, mut _cmd_tx, mut _report_rx, _recordings) =
            create_offline_engine(None, AudioBackendType::FundSP, sample_rate)
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

        let (mut composer, _playback, mut _cmd_tx, mut _report_rx, _recordings) =
            create_offline_engine(None, AudioBackendType::FundSP, sample_rate)
                .expect("create_offline_engine failed");

        // In direct mode, set_bpm should work normally
        composer.set_bpm(155.0);
        assert!((composer.musical_params().bpm - 155.0).abs() < 0.01, "BPM should be 155");

        // Reset BPM reverts to the initial emotion_mapped_bpm (120.0 default)
        composer.reset_bpm();
        assert!(
            (composer.musical_params().bpm - 120.0).abs() < 0.01,
            "BPM should revert to default 120 after reset, got {}",
            composer.musical_params().bpm
        );
    }
}

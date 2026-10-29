use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use harmonium_audio::backend::recorder::RecorderWorker;
use harmonium_core::{
    EngineCommand, EngineReport, MeasureSnapshot, MusicalParams,
    events::RecordFormat,
    params::SessionConfig,
    trad::{BarRole, TunePlan},
};
use web_time::Instant;

use crate::{
    FinishedRecordings,
    audio::{self, EngineControls},
    clock::{BeatInfo, ClockReader},
    composer::MusicComposer,
    playback::{PlaybackCommand, PlaybackEngine, TransportPosition},
};

/// The output stream held by the host.
///
/// Desktop: `cpal::Stream` is `Send + Sync` here (see the SAFETY note), so a
/// host behind a `Mutex` can be shared between the REPL's threads. Wasm: the
/// stream wraps JS objects of the page's one thread and is not `Send`, so
/// neither is the host there: it lives on the thread that created it.
struct OutputStream(cpal::Stream);

// SAFETY: cpal::Stream on desktop platforms (CoreAudio, WASAPI, ALSA) uses
// thread-safe OS handles. We only interact via play()/pause()/drop().
#[cfg(not(target_arch = "wasm32"))]
#[allow(unsafe_code)]
unsafe impl Send for OutputStream {}
#[cfg(not(target_arch = "wasm32"))]
#[allow(unsafe_code)]
unsafe impl Sync for OutputStream {}

/// The one live host of the Harmonium engine: REPL, OSC and studio drive the
/// same object.
///
/// It owns the `MusicComposer` (main thread), the command producer toward the
/// audio-thread `PlaybackEngine`, the report consumer, the transport position
/// and the predictive clock reader. Two entry points do everything:
///
/// - [`apply`](Self::apply) executes an [`EngineCommand`] — the only way to
///   change the engine;
/// - [`pump`](Self::pump) composes ahead of the playhead and drains the
///   published reports — the caller drives generation (a thread in the REPL
///   and the native studio, a Worker timer in the web studio).
///
/// Both take `&mut self`: callers that share the host between threads put it
/// behind a `Mutex`.
pub struct LiveHost {
    /// Output stream; `None` for a [detached](Self::detached) host.
    stream: Option<OutputStream>,
    composer: MusicComposer,
    playback_cmd_tx: rtrb::Producer<PlaybackCommand>,
    report_rx: rtrb::Consumer<EngineReport>,
    /// Shared transport position, packed (bar, step) — written by the audio
    /// thread, read lock-free through the host's own Arc clone
    transport_position: Arc<AtomicU64>,
    /// Lock-free reader of the published predictive clock. Single-consumer:
    /// `beat_info` reads it on the owning thread; `take_clock_reader` hands it
    /// to another thread. `None` while it is taken — `beat_info` then reports
    /// `None` rather than a false beat.
    clock_reader: Option<ClockReader>,
    finished_recordings: FinishedRecordings,
    /// Recorder encoder without its own thread (wasm), advanced by `pump`;
    /// `None` natively, where a thread encodes.
    recorder: Option<RecorderWorker>,
    /// Last report received from the audio thread.
    cached_state: Option<EngineReport>,
}

impl LiveHost {
    /// Start the engine on the default output device and begin playback.
    ///
    /// `sf2_bytes` is the main bank, `feet_sf2_bytes` the feet bank (without
    /// it the feet fall back to the GM kit on channel 9), `odin_presets` the
    /// `.odin` presets provided by the game (name + bytes); without the
    /// `odin` feature they are ignored (SoundFont rendering). `buffer_frames`
    /// fixes the device buffer size in frames (`None`: platform default).
    pub fn start(
        sf2_bytes: Option<&[u8]>,
        feet_sf2_bytes: Option<&[u8]>,
        odin_presets: &[harmonium_audio::odin_source::OdinPresetSource],
        buffer_frames: Option<u32>,
    ) -> Result<Self, String> {
        let (stream, controls) =
            audio::create_timeline_stream(sf2_bytes, feet_sf2_bytes, odin_presets, buffer_frames)?;
        Self::from_controls(Some(OutputStream(stream)), controls)
    }

    /// Start the engine in a paused state.
    pub fn start_paused(
        sf2_bytes: Option<&[u8]>,
        feet_sf2_bytes: Option<&[u8]>,
        odin_presets: &[harmonium_audio::odin_source::OdinPresetSource],
        buffer_frames: Option<u32>,
    ) -> Result<Self, String> {
        let host = Self::start(sf2_bytes, feet_sf2_bytes, odin_presets, buffer_frames)?;
        host.pause()?;
        Ok(host)
    }

    /// A host without output device: returns the [`PlaybackEngine`] for the
    /// caller to run (`process_buffer`), at `sample_rate`. The recorder blocks
    /// instead of dropping, as in offline rendering (wasm: no thread to block
    /// on, so it drops and counts when the caller lets [`Self::pump`] lag
    /// behind). For tests and rendering.
    pub fn detached(
        sf2_bytes: Option<&[u8]>,
        feet_sf2_bytes: Option<&[u8]>,
        odin_presets: &[harmonium_audio::odin_source::OdinPresetSource],
        sample_rate: f64,
    ) -> Result<(Self, PlaybackEngine), String> {
        let (controls, playback) = audio::build_engine(
            sf2_bytes,
            feet_sf2_bytes,
            odin_presets,
            sample_rate,
            harmonium_audio::backend::recorder::OverflowPolicy::Block,
        );
        Ok((Self::from_controls(None, controls)?, playback))
    }

    /// Wrap the control side and publish the composer's initial state (its
    /// parameters and the style's program changes) to the audio thread.
    fn from_controls(
        stream: Option<OutputStream>,
        controls: EngineControls,
    ) -> Result<Self, String> {
        let mut host = Self {
            stream,
            composer: controls.composer,
            playback_cmd_tx: controls.commands,
            report_rx: controls.reports,
            transport_position: controls.transport_position,
            clock_reader: Some(controls.clock_reader),
            finished_recordings: controls.finished_recordings,
            recorder: controls.recorder,
            cached_state: None,
        };
        host.publish()?;
        Ok(host)
    }

    // === Transport ===

    /// Resume the output stream (no-op on a detached host).
    pub fn resume(&self) -> Result<(), String> {
        use cpal::traits::StreamTrait;
        self.stream.as_ref().map_or(Ok(()), |s| s.0.play().map_err(|e| e.to_string()))
    }

    /// Pause the output stream (no-op on a detached host).
    pub fn pause(&self) -> Result<(), String> {
        use cpal::traits::StreamTrait;
        self.stream.as_ref().map_or(Ok(()), |s| s.0.pause().map_err(|e| e.to_string()))
    }

    /// Read the current transport position at grid resolution.
    ///
    /// No lock is taken: the host holds its own clone of the shared atomic.
    pub fn transport_position(&self) -> TransportPosition {
        TransportPosition::from_packed(self.transport_position.load(Ordering::Relaxed))
    }

    // === Beat clock (predictive, lock-free) ===

    /// Read the latest [`BeatInfo`] for the given wall-clock instant.
    ///
    /// Returns `None` while the clock reader is taken (see
    /// [`Self::take_clock_reader`]): the host never fabricates a beat, so a
    /// caller that holds the reader on another thread gets a truthful "no
    /// data" instead of a false one.
    pub fn beat_info(&mut self, now: Instant) -> Option<BeatInfo> {
        let clock = self.clock_reader.as_mut()?.read();
        Some(BeatInfo::from_clock(&clock, now))
    }

    /// Hand the lock-free clock reader to another thread (a metronome view,
    /// the game).
    ///
    /// Returns `Some` the first time, `None` thereafter — the caller returns
    /// it via [`Self::restore_clock_reader`] so the host can read beats again.
    pub fn take_clock_reader(&mut self) -> Option<ClockReader> {
        self.clock_reader.take()
    }

    /// Return a previously-taken clock reader so [`Self::beat_info`] can read
    /// the beat again (the reader is single-consumer, not cloneable).
    pub fn restore_clock_reader(&mut self, reader: ClockReader) {
        self.clock_reader = Some(reader);
    }

    // === Generation, driven by the caller ===

    /// Compose ahead of the playhead and drain the audio thread's reports.
    ///
    /// Call periodically (every ~50 ms): it stages measures for playback, frees
    /// the played ones on this thread and refreshes [`Self::poll_state`].
    /// Returns the snapshots of the measures composed (or restaged) since the
    /// last call, for a caller that draws them. Without an encoder thread
    /// (wasm), it also advances the recorder: recordings finish through here.
    pub fn pump(&mut self) -> Vec<MeasureSnapshot> {
        if let Some(recorder) = &mut self.recorder {
            recorder.pump();
        }
        self.composer.generate_ahead();
        #[cfg(feature = "odin")]
        {
            // Une file pleine ou le second emplacement occupé ne perd pas le
            // choix courant : le pump suivant reprend, sans rafale de presets.
            let _ = self.composer.send_odin(&mut self.playback_cmd_tx);
        }
        self.drain_reports();
        self.composer.take_snapshots()
    }

    fn drain_reports(&mut self) {
        while let Ok(report) = self.report_rx.pop() {
            self.cached_state = Some(report);
        }
    }

    // === Published state ===

    /// The latest report published by the audio thread, `None` before the
    /// first one (and right after a soft seek, until a fresh one arrives).
    pub fn poll_state(&mut self) -> Option<&EngineReport> {
        self.drain_reports();
        self.cached_state.as_ref()
    }

    /// Recordings completed since the last call (format + file bytes).
    pub fn take_recordings(&self) -> Vec<(RecordFormat, Vec<u8>)> {
        self.finished_recordings
            .lock()
            .map(|mut queue| std::mem::take(&mut *queue))
            .unwrap_or_default()
    }

    /// Tempo, pas et pulsations par mesure du profil actif.
    pub fn session_config(&self) -> &SessionConfig {
        &self.composer.config
    }

    /// Commanded musical parameters (what the next composed bar will use).
    pub fn musical_params(&self) -> &MusicalParams {
        self.composer.musical_params()
    }

    /// Ressources que le profil actif désigne et que l'hôte n'a pas chargées
    /// (déjà journalisées au chargement du profil).
    pub fn missing_resources(&self) -> &[harmonium_core::host_resources::MissingResource] {
        self.composer.missing_resources()
    }

    /// The four axes playing `[arousal, valence, density, tension]`: the
    /// commanded emotions in emotion mode, the direct parameters otherwise.
    /// Entering emotion mode starts from them, so changing one axis leaves
    /// the other three where they were.
    pub fn emotions(&self) -> [f32; 4] {
        self.composer.emotions()
    }

    /// Active style profile (the one the next measures are composed with).
    pub fn profile(&self) -> &harmonium_core::StyleProfile {
        self.composer.profile()
    }

    /// Seed of the session (to save with a recording).
    pub fn session_seed(&self) -> u64 {
        self.composer.session_seed()
    }

    /// Control mode: `true` when the emotions drive the generation
    /// (`SetEmotionParams`, `UseEmotionMode`), `false` in direct mode
    /// (`UseDirectMode`).
    pub fn is_emotion_mode(&self) -> bool {
        self.composer.is_emotion_mode()
    }

    /// Plan and role of bar `bar` (1-based) in the active profile's form.
    #[must_use]
    pub fn profile_bar_info(&self, bar: usize) -> Option<(TunePlan, BarRole)> {
        self.composer.profile_bar_info(bar)
    }

    /// Bar under the playhead, as seen by the composer.
    pub fn playhead_bar(&self) -> usize {
        self.composer.playhead_bar()
    }

    // === The executor ===

    /// Execute a command: the only way to change the engine.
    ///
    /// Changes to the generation take effect on the next composed bar
    /// (future bars are invalidated and evicted from playback); playback
    /// commands reach the audio thread through the lock-free ring. Composing
    /// the replacement bars is left to [`Self::pump`].
    pub fn apply(&mut self, command: EngineCommand) -> Result<(), String> {
        match command {
            // === Emotions and tempo of the measures ===
            EngineCommand::SetEmotionParams { arousal, valence, density, tension } => {
                self.composer.use_emotion_mode();
                self.composer.set_emotions(arousal, valence, density, tension);
                self.invalidate_future()
            }
            EngineCommand::UseEmotionMode => {
                self.composer.use_emotion_mode();
                Ok(())
            }
            EngineCommand::UseDirectMode => {
                self.composer.use_direct_mode();
                Ok(())
            }
            EngineCommand::SetBpm(bpm) => {
                self.composer.set_bpm(bpm);
                self.invalidate_future()
            }
            EngineCommand::ResetBpm => {
                self.composer.reset_bpm();
                self.invalidate_future()
            }
            EngineCommand::SetTimeSignature { numerator, denominator } => {
                self.composer.set_time_signature(numerator, denominator);
                self.invalidate_future()
            }
            EngineCommand::SetSwing(ratio) => {
                self.composer.set_swing_ratio(ratio);
                self.invalidate_future()
            }
            EngineCommand::EnableRhythm(enabled) => {
                self.composer.enable_rhythm(enabled);
                self.invalidate_future()
            }
            EngineCommand::EnableHarmony(enabled) => {
                self.composer.enable_harmony(enabled);
                self.invalidate_future()
            }
            EngineCommand::EnableMelody(enabled) => {
                self.composer.enable_melody(enabled);
                self.invalidate_future()
            }

            // === Style ===
            EngineCommand::SetStyleProfile(profile) => {
                self.composer.set_profile(*profile)?;
                self.publish()
            }
            EngineCommand::SetFeetDensity(density) => {
                self.composer.set_feet_density(density);
                self.invalidate_future()
            }
            EngineCommand::SetIntensity(intensity) => {
                self.composer.set_intensity(intensity);
                self.invalidate_future()
            }
            EngineCommand::Reset => {
                // Defaults of the active profile, no loop, no imposed audible
                // tempo, back to bar 1 with the same seed.
                self.composer.reset();
                self.send(PlaybackCommand::ClearLoop)?;
                self.send(PlaybackCommand::ClearTempo)?;
                self.restart_session()
            }

            // === Session ===
            EngineCommand::SetSeed(seed) => {
                self.composer.set_seed(seed);
                self.restart_session()
            }
            EngineCommand::NewMelody => {
                self.composer.new_melody();
                self.restart_session()
            }
            EngineCommand::SetWriteheadLookahead(bars) => {
                self.composer.set_writehead_lookahead(bars);
                Ok(())
            }

            // === Timeline ===
            EngineCommand::Seek(bar) => {
                // Deterministic: the generator replays to the target bar as if
                // generation had proceeded linearly from bar 1 with the seed.
                // Staged boxes from the target onward are evicted first
                // (`DropFrom`), then the playhead jumps (`Seek`): no stale
                // measure can play after the seek.
                let bar = bar.max(1);
                self.composer.deterministic_seek(bar);
                self.publish()?;
                self.send(PlaybackCommand::Seek(bar))
            }
            EngineCommand::SeekPlayhead(bar) => {
                // Staged measures stay valid — playback reads by index, and a
                // box is only loaded when its index equals the playhead bar.
                self.send(PlaybackCommand::SeekPlayhead(bar.max(1)))?;
                // Reports queued before the jump would leak through while
                // paused: forget them until the audio thread publishes anew.
                self.cached_state = None;
                while self.report_rx.pop().is_ok() {}
                Ok(())
            }
            EngineCommand::SetLoop { start_bar, end_bar } => {
                // Validated here, never in the audio callback: a loop longer
                // than the playback window could not stay fully staged.
                if !crate::playback::check_loop_len(start_bar, end_bar) {
                    return Err(format!(
                        "Loop too long: at most {} bars fit the playback window",
                        crate::playback::MEASURE_WINDOW_CAP
                    ));
                }
                self.send(PlaybackCommand::SetLoop { start_bar, end_bar })
            }
            EngineCommand::ClearLoop => self.send(PlaybackCommand::ClearLoop),

            // === Audible tempo (applied at pulse boundaries) ===
            EngineCommand::SetAudibleTempo { bpm, ramp_pulses } => {
                self.send(PlaybackCommand::SetTempo { bpm, ramp_pulses })
            }
            EngineCommand::ClearAudibleTempo => self.send(PlaybackCommand::ClearTempo),

            // === Mixer and output ===
            EngineCommand::SetMasterVolume(volume) => {
                self.send(PlaybackCommand::SetMasterVolume(volume))
            }
            EngineCommand::SetChannelGain { channel, gain } => {
                self.send(PlaybackCommand::SetChannelGain { channel, gain })
            }
            EngineCommand::SetChannelMute { channel, muted } => {
                self.send(PlaybackCommand::SetChannelMute { channel, muted })
            }
            EngineCommand::SetChannelRoute { channel, bank_id } => {
                self.send(PlaybackCommand::SetChannelRoute { channel, bank_id })
            }
            EngineCommand::SetOutputMute(muted) => self.send(PlaybackCommand::SetOutputMute(muted)),

            // === Recording ===
            EngineCommand::StartRecording(format) => {
                self.composer.start_recording(format, &mut self.playback_cmd_tx)
            }
            EngineCommand::StopRecording(format) => self.composer.stop_recording(
                format,
                &mut self.playback_cmd_tx,
                &self.finished_recordings,
            ),
        }
    }

    fn send(&mut self, command: PlaybackCommand) -> Result<(), String> {
        self.playback_cmd_tx.push(command).map_err(|_| "file de commandes audio pleine".to_string())
    }

    /// Invalidate the future measures (the playhead's own bar is kept) so
    /// they are composed again with the new parameters.
    fn invalidate_future(&mut self) -> Result<(), String> {
        self.composer.invalidate_future();
        self.publish()
    }

    /// Back to bar 1 (new seed, or reset): evict, then move the playhead.
    fn restart_session(&mut self) -> Result<(), String> {
        self.publish()?;
        self.send(PlaybackCommand::Seek(1))
    }

    /// After a change to the composer: forward the pending eviction (if the
    /// timeline changed) and the style's program changes, then hand the
    /// audio thread the commanded parameters (recorders, reporting) and the
    /// Odin presets morphed by the current emotions (prepared here, copied by
    /// the callback). Epoch tags make the order against the
    /// replacement bars irrelevant for correctness; evicting first keeps the
    /// window from filling with stale boxes.
    fn publish(&mut self) -> Result<(), String> {
        if let Some((bar, _)) = self.composer.take_pending_drop() {
            self.send(PlaybackCommand::DropFrom { bar })?;
        }
        for (channel, program) in self.composer.take_pending_program_changes() {
            self.send(PlaybackCommand::ProgramChange { channel, program })?;
        }
        self.send(PlaybackCommand::UpdateMusicalParams(
            self.composer.musical_params().playback_params(),
        ))?;
        self.composer.send_odin(&mut self.playback_cmd_tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The take/restore contract: while the clock reader is taken, `beat_info`
    /// reports `None` — never a fabricated beat — and after restore it reports
    /// `Some` coherent with the restored reader.
    #[test]
    fn take_and_restore_clock_reader_never_fabricates_a_beat() -> Result<(), String> {
        let (mut host, _playback) = LiveHost::detached(None, None, &[], 44100.0)?;
        let now = Instant::now();

        // Reader present: a real beat.
        assert!(host.beat_info(now).is_some(), "beat_info before take");

        // Take it: the caller owns the reader, the host must report None.
        let mut reader = host.take_clock_reader().ok_or("reader present before take")?;
        assert!(host.take_clock_reader().is_none(), "reader is single-instance");
        assert!(host.beat_info(now).is_none(), "no fabricated beat while taken");

        // The caller reads the clock through the moved-out reader.
        let clock = reader.read();

        // Restore it: beat_info is Some again and coherent with the reader.
        host.restore_clock_reader(reader);
        let beat = host.beat_info(now).ok_or("beat after restore")?;
        assert_eq!(beat.bar, clock.position.bar);
        assert_eq!(beat.pulse, clock.pulse_in_bar + 1);
        assert_eq!(beat.next_pulse_frame, clock.next_pulse_frame);
        Ok(())
    }
}

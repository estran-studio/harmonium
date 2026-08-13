use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

use harmonium_core::{EngineReport, MeasureSnapshot, params::SessionConfig};

use crate::{
    FinishedRecordings, FontQueue,
    audio::AudioBackendType,
    composer::MusicComposer,
    playback::{PlaybackCommand, TransportPosition},
};

/// Owned handle to the live-MIDI monitor channel. Wraps the SPSC producer so
/// callers (the MIDI input subsystem) get a clean `note_on`/`note_off` API
/// without depending on `rtrb`. Single-instance: moved into the MIDI callback
/// for the life of the connection, then handed back via
/// [`NativeHandle::restore_live_midi_sender`].
pub struct LiveMidiSender {
    tx: rtrb::Producer<crate::playback::LiveMidiEvent>,
}

impl LiveMidiSender {
    /// Voice a note the player just pressed. Drops silently if the audio
    /// thread is momentarily behind (ring full) — a missed monitor note is
    /// preferable to blocking the MIDI callback.
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        let _ = self.tx.push(crate::playback::LiveMidiEvent { on: true, note, velocity });
    }

    /// Release a held note.
    pub fn note_off(&mut self, note: u8) {
        let _ = self.tx.push(crate::playback::LiveMidiEvent { on: false, note, velocity: 0 });
    }
}

/// Wrapper that makes `cpal::Stream` `Send + Sync`.
struct SendStream(cpal::Stream);

// SAFETY: cpal::Stream on desktop platforms (CoreAudio, WASAPI, ALSA) uses
// thread-safe OS handles. We only interact via play()/pause()/drop().
#[allow(unsafe_code)]
unsafe impl Send for SendStream {}
#[allow(unsafe_code)]
unsafe impl Sync for SendStream {}

/// Native (non-WASM) handle for driving the Harmonium engine.
///
/// Owns a `MusicComposer` (direct calls via Mutex) and a `PlaybackCommand`
/// producer for sending commands to the audio-thread PlaybackEngine.
pub struct NativeHandle {
    stream: SendStream,
    composer: Mutex<MusicComposer>,
    playback_cmd_tx: rtrb::Producer<PlaybackCommand>,
    /// Shared transport position, packed (bar, step) — written by the audio
    /// thread, read lock-free through the handle's own Arc clone
    transport_position: Arc<AtomicU64>,
    /// SPSC producer for the live-MIDI monitor. `take`n once by the MIDI
    /// input subsystem (the controller note-on/off callback), which then
    /// owns it for the life of the connection. `None` after it's taken.
    live_midi_tx: Option<LiveMidiSender>,
    report_rx: rtrb::Consumer<EngineReport>,
    session_config: SessionConfig,
    font_queue: FontQueue,
    #[allow(dead_code)]
    finished_recordings: FinishedRecordings,
    /// Accumulated measures from the composer.
    measures_buffer: Vec<MeasureSnapshot>,
    /// Cached state (last received report).
    cached_state: Option<EngineReport>,
}

impl NativeHandle {
    /// Start the engine and immediately begin playback.
    pub fn start(sf2_bytes: Option<&[u8]>, backend: AudioBackendType) -> Result<Self, String> {
        let (
            stream,
            session_config,
            composer,
            playback_cmd_tx,
            live_midi_tx,
            report_rx,
            font_queue,
            transport_position,
            finished_recordings,
        ) = crate::audio::create_timeline_stream(sf2_bytes, backend)?;

        Ok(Self {
            stream: SendStream(stream),
            composer,
            playback_cmd_tx,
            transport_position,
            live_midi_tx: Some(LiveMidiSender { tx: live_midi_tx }),
            report_rx,
            session_config,
            font_queue,
            finished_recordings,
            measures_buffer: Vec::new(),
            cached_state: None,
        })
    }

    /// Start the engine in a paused state.
    pub fn start_paused(
        sf2_bytes: Option<&[u8]>,
        backend: AudioBackendType,
    ) -> Result<Self, String> {
        let handle = Self::start(sf2_bytes, backend)?;
        handle.pause()?;
        Ok(handle)
    }

    /// Take ownership of the live-MIDI monitor producer. Returns `Some` the
    /// first time (handed to the MIDI input callback), `None` thereafter —
    /// the caller is responsible for parking it and giving it back via
    /// [`Self::restore_live_midi_sender`] when MIDI input stops, so a later
    /// start can reclaim it.
    pub fn take_live_midi_sender(&mut self) -> Option<LiveMidiSender> {
        self.live_midi_tx.take()
    }

    /// Return a previously-taken live-MIDI producer so the next MIDI start
    /// can reuse it (the SPSC producer is single-instance, not cloneable).
    pub fn restore_live_midi_sender(&mut self, tx: LiveMidiSender) {
        self.live_midi_tx = Some(tx);
    }

    // === Playback Controls ===

    pub fn resume(&self) -> Result<(), String> {
        use cpal::traits::StreamTrait;
        self.stream.0.play().map_err(|e| e.to_string())
    }

    pub fn pause(&self) -> Result<(), String> {
        use cpal::traits::StreamTrait;
        self.stream.0.pause().map_err(|e| e.to_string())
    }

    // === Session Info ===

    pub fn session_config(&self) -> &SessionConfig {
        &self.session_config
    }

    // === Generation (direct calls to MusicComposer) ===

    /// Generate bars synchronously. No audio stream needed.
    pub fn generate_bars(&self, count: usize) {
        if let Ok(mut composer) = self.composer.lock() {
            composer.generate_bars(count);
        }
    }

    /// Generate ahead of the playhead (incremental). Call this periodically
    /// during playback to keep shared pages populated.
    pub fn generate_ahead(&self) {
        if let Ok(mut composer) = self.composer.lock() {
            composer.generate_ahead();
        }
    }

    /// Reset composer and clear timeline + shared pages.
    pub fn reset_composer(&mut self) {
        if let Ok(mut composer) = self.composer.lock() {
            composer.reset();
        }
    }

    /// Invalidate future measures and regenerate with current params.
    pub fn invalidate_and_regenerate(&mut self, bars: usize) {
        if let Ok(mut composer) = self.composer.lock() {
            composer.invalidate_future();
            composer.generate_bars(bars);
        }
    }

    /// Full reset + regenerate. Shared pages are updated in-place.
    pub fn reset_and_regenerate(&mut self, bars: usize) {
        if let Ok(mut composer) = self.composer.lock() {
            composer.reset();
            composer.generate_bars(bars);
        }
        self.measures_buffer.clear();
    }

    /// Clear timeline and regenerate, keeping current musical params.
    /// Use this for "New" / regenerate where the user wants fresh bars
    /// but with their current settings (emotions, rhythm mode, etc.).
    pub fn regenerate_with_current_params(&mut self, bars: usize) {
        if let Ok(mut composer) = self.composer.lock() {
            composer.reset_timeline();
            composer.generate_bars(bars);
        }
        self.measures_buffer.clear();
        // Clear stale reports so poll_state() doesn't return pre-regeneration data
        self.cached_state = None;
        while self.report_rx.pop().is_ok() {}
    }

    /// Re-key the procedural session to a concert pitch class (0–11) and
    /// regenerate `bars` from bar 1. Keeps the cached session key in sync so
    /// `session_config().key` reflects the new key. True no-op if already in
    /// `key_pc` (the timeline is left untouched).
    pub fn set_session_key(&mut self, key_pc: u8, bars: usize) {
        let new_key = if let Ok(mut composer) = self.composer.lock() {
            if !composer.set_session_key(key_pc) {
                return;
            }
            composer.reset_timeline();
            // Re-seed the RNG/generator for the new key so a later seek back to
            // bar 1 reproduces these bars instead of morphing the melody.
            composer.deterministic_seek(1);
            composer.generate_bars(bars);
            composer.session_key_string()
        } else {
            return;
        };
        self.session_config.key = new_key;
        self.measures_buffer.clear();
        self.cached_state = None;
        while self.report_rx.pop().is_ok() {}
    }

    /// Read the current playhead bar (from the shared atomic).
    pub fn playhead_bar(&self) -> usize {
        if let Ok(composer) = self.composer.lock() { composer.playhead_bar() } else { 1 }
    }

    // === Transport position (lock-free) ===

    /// Read the current transport position at grid resolution.
    ///
    /// No lock is taken: the handle holds its own clone of the shared
    /// atomic, so this is safe to call from a real-time thread (FR-002).
    pub fn transport_position(&self) -> TransportPosition {
        TransportPosition::from_packed(self.transport_position.load(Ordering::Relaxed))
    }

    /// The raw shared position handle, for real-time callers (MIDI
    /// callback): clone ONCE at input startup, then `load(Ordering::Relaxed)`
    /// per event and decode with [`crate::playback::unpack`].
    pub fn transport_position_handle(&self) -> Arc<AtomicU64> {
        self.transport_position.clone()
    }

    /// Apply param changes while preserving the preview window.
    ///
    /// Preview bars stay intact in both timeline and shared pages.
    /// Bars beyond are invalidated for regeneration with new params.
    pub fn apply_params_preserving_preview(&mut self, preview_bars: usize) {
        if let Ok(mut composer) = self.composer.lock() {
            composer.invalidate_after_preview(preview_bars);
        }
    }

    // === Timeline / Measure API ===

    /// Drain newly-generated measures from the composer, append to buffer.
    pub fn poll_measures(&mut self) -> Vec<MeasureSnapshot> {
        self.poll_reports();

        let new = if let Ok(mut composer) = self.composer.lock() {
            composer.take_snapshots()
        } else {
            Vec::new()
        };

        for m in &new {
            if let Some(existing) = self.measures_buffer.iter_mut().find(|e| e.index == m.index) {
                *existing = m.clone();
            } else {
                self.measures_buffer.push(m.clone());
            }
        }
        new
    }

    /// Get measures from the accumulated buffer for a given range.
    pub fn get_buffered_measures(&self, from_bar: usize, count: usize) -> Vec<MeasureSnapshot> {
        self.measures_buffer
            .iter()
            .filter(|m| m.index >= from_bar && m.index < from_bar + count)
            .cloned()
            .collect()
    }

    pub fn buffered_measure_count(&self) -> usize {
        self.measures_buffer.len()
    }

    pub fn clear_measures(&mut self) {
        self.measures_buffer.clear();
    }

    // === Report polling ===

    fn poll_reports(&mut self) {
        while let Ok(report) = self.report_rx.pop() {
            self.cached_state = Some(report);
        }
    }

    pub fn poll_state(&mut self) -> Option<&EngineReport> {
        self.poll_reports();
        self.cached_state.as_ref()
    }

    // === Composer setters (generation params — direct calls) ===

    pub fn use_emotion_mode(&self) {
        if let Ok(mut c) = self.composer.lock() {
            c.use_emotion_mode();
        }
    }

    pub fn use_direct_mode(&self) {
        if let Ok(mut c) = self.composer.lock() {
            c.use_direct_mode();
        }
    }

    pub fn set_bpm(&self, bpm: f32) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_bpm(bpm);
        }
    }

    pub fn reset_bpm(&self) {
        if let Ok(mut c) = self.composer.lock() {
            c.reset_bpm();
        }
    }

    pub fn bpm_override(&self) -> Option<f32> {
        self.composer.lock().ok().and_then(|c| c.bpm_override())
    }

    pub fn emotion_mapped_bpm(&self) -> f32 {
        self.composer.lock().ok().map(|c| c.emotion_mapped_bpm()).unwrap_or(120.0)
    }

    pub fn set_emotions(&self, arousal: f32, valence: f32, density: f32, tension: f32) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_emotions(arousal, valence, density, tension);
        }
    }

    pub fn set_time_signature(&self, numerator: usize, denominator: usize) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_time_signature(numerator, denominator);
        }
    }

    pub fn set_density(&self, density: f32) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_rhythm_density(density);
        }
    }

    pub fn set_rhythm_density(&self, density: f32) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_rhythm_density(density);
        }
    }

    pub fn set_rhythm_tension(&self, tension: f32) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_rhythm_tension(tension);
        }
    }

    pub fn enable_melody(&self, enabled: bool) {
        if let Ok(mut c) = self.composer.lock() {
            c.enable_melody(enabled);
        }
    }

    pub fn enable_harmony(&self, enabled: bool) {
        if let Ok(mut c) = self.composer.lock() {
            c.enable_harmony(enabled);
        }
    }

    pub fn enable_rhythm(&self, enabled: bool) {
        if let Ok(mut c) = self.composer.lock() {
            c.enable_rhythm(enabled);
        }
    }

    pub fn enable_voicing(&self, enabled: bool) {
        if let Ok(mut c) = self.composer.lock() {
            c.enable_voicing(enabled);
        }
    }

    pub fn set_melody_smoothness(&self, smoothness: f32) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_melody_smoothness(smoothness);
        }
    }

    pub fn set_rhythm_steps(&self, steps: usize) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_rhythm_steps(steps);
        }
    }

    pub fn set_rhythm_pulses(&self, pulses: usize) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_rhythm_pulses(pulses);
        }
    }

    pub fn set_rhythm_rotation(&self, rotation: usize) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_rhythm_rotation(rotation);
        }
    }

    pub fn set_harmony_tension(&self, tension: f32) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_harmony_tension(tension);
        }
    }

    pub fn set_harmony_valence(&self, valence: f32) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_harmony_valence(valence);
        }
    }

    pub fn set_tuning(&self, tuning: harmonium_core::tuning::TuningParams) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_tuning(tuning);
            c.sync_generator();
        }
    }

    pub fn set_rhythm_mode(&self, mode: harmonium_core::sequencer::RhythmMode) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_rhythm_mode(mode);
        }
    }

    pub fn set_harmony_mode(&self, mode: harmonium_core::harmony::HarmonyMode) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_harmony_mode(mode);
        }
    }

    pub fn set_harmony_measures_per_chord(&self, measures: usize) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_harmony_measures_per_chord(measures);
        }
    }

    pub fn set_key_root(&self, root: u8) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_key_root(root);
        }
    }

    pub fn set_harmony_strategy(&self, strategy: harmonium_core::params::HarmonyStrategy) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_harmony_strategy(strategy);
        }
    }

    pub fn set_chord_chart(&self, chart: &[String]) {
        if let Ok(mut c) = self.composer.lock() {
            let chart_arr: Vec<arrayvec::ArrayString<16>> = chart
                .iter()
                .filter_map(|s| arrayvec::ArrayString::try_from(s.as_str()).ok())
                .collect();
            c.set_chord_chart(chart_arr);
        }
    }

    pub fn set_instrument_lead(&self, config: harmonium_core::params::InstrumentConfig) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_instrument_lead(config);
        }
    }

    pub fn set_instrument_bass(&self, config: harmonium_core::params::InstrumentConfig) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_instrument_bass(config);
        }
    }

    pub fn set_writehead_lookahead(&self, bars: usize) {
        if let Ok(mut c) = self.composer.lock() {
            c.set_writehead_lookahead(bars);
        }
    }

    /// Sync the generator with current musical params.
    /// Call after batch param changes and before the first generate_bars().
    pub fn sync_generator(&self) {
        if let Ok(mut c) = self.composer.lock() {
            c.sync_generator();
        }
    }

    // === Playback commands (sent to audio thread) ===

    pub fn set_channel_gain(&mut self, channel: u8, gain: f32) {
        let _ = self.playback_cmd_tx.push(PlaybackCommand::SetChannelGain { channel, gain });
    }

    pub fn set_channel_mute(&mut self, channel: u8, muted: bool) {
        let _ = self.playback_cmd_tx.push(PlaybackCommand::SetChannelMute { channel, muted });
    }

    pub fn set_channel_route(&mut self, channel: u8, bank_id: i32) {
        let _ = self.playback_cmd_tx.push(PlaybackCommand::SetChannelRoute { channel, bank_id });
    }

    pub fn set_output_mute(&mut self, muted: bool) {
        let _ = self.playback_cmd_tx.push(PlaybackCommand::SetOutputMute(muted));
    }

    /// Deterministic seek: reset RNG + generator, replay to target bar.
    ///
    /// Ensures the composer is in the exact state for `target_bar` as if
    /// generation had proceeded linearly from bar 1 with the session seed.
    pub fn seek(&mut self, bar: usize) {
        let target_bar = bar.max(1);
        if let Ok(mut composer) = self.composer.lock() {
            composer.deterministic_seek(target_bar);
        }
        let _ = self.playback_cmd_tx.push(PlaybackCommand::Seek(target_bar));
    }

    /// Seek playhead without resetting writehead.
    /// Shared pages already have the measures — playback reads by index.
    pub fn seek_playhead(&mut self, bar: usize) {
        let target_bar = bar.max(1);
        let _ = self.playback_cmd_tx.push(PlaybackCommand::SeekPlayhead(target_bar));
        // Clear cached state so poll_state() returns None until the audio thread
        // sends a fresh report from the new position. Without this, stale reports
        // from before the seek leak through when the stream is paused.
        self.cached_state = None;
        // Drain any in-flight reports that were queued before the seek
        while self.report_rx.pop().is_ok() {}
    }

    pub fn set_loop(&mut self, start_bar: usize, end_bar: usize) {
        let _ = self.playback_cmd_tx.push(PlaybackCommand::SetLoop { start_bar, end_bar });
    }

    pub fn clear_loop(&mut self) {
        let _ = self.playback_cmd_tx.push(PlaybackCommand::ClearLoop);
    }

    /// Generate a new melody with a fresh random seed.
    /// Resets the session to bar 1 with entirely new content.
    pub fn new_melody(&mut self) {
        if let Ok(mut composer) = self.composer.lock() {
            composer.new_melody();
        }
        let _ = self.playback_cmd_tx.push(PlaybackCommand::Seek(1));
    }

    /// Set an explicit seed and regenerate from bar 1.
    /// Used for reproducible sessions (e.g. restoring a saved track).
    pub fn set_seed(&mut self, seed: u64) {
        if let Ok(mut composer) = self.composer.lock() {
            composer.set_seed(seed);
        }
        let _ = self.playback_cmd_tx.push(PlaybackCommand::Seek(1));
    }

    /// Get the current session seed (for saving with track records).
    pub fn session_seed(&self) -> Option<u64> {
        self.composer.lock().ok().map(|c| c.session_seed())
    }

    pub fn start_recording(&mut self, format: harmonium_core::events::RecordFormat) {
        let _ = self.playback_cmd_tx.push(PlaybackCommand::StartRecording(format));
    }

    pub fn stop_recording(&mut self, format: harmonium_core::events::RecordFormat) {
        let _ = self.playback_cmd_tx.push(PlaybackCommand::StopRecording(format));
    }

    /// Send a GM Program Change to a logical channel.
    pub fn set_channel_program(&mut self, channel: u8, program: u8) {
        let _ = self.playback_cmd_tx.push(PlaybackCommand::ProgramChange { channel, program });
    }

    /// Add a SoundFont to a specific bank.
    pub fn add_soundfont(&self, bank_id: u32, sf2_bytes: Vec<u8>) {
        if let Ok(mut queue) = self.font_queue.lock() {
            queue.push((bank_id, sf2_bytes));
        }
    }

    /// Load queued fonts into playback engine.
    pub fn flush_fonts(&mut self) {
        if let Ok(mut queue) = self.font_queue.try_lock() {
            while let Some((id, bytes)) = queue.pop() {
                let _ = self.playback_cmd_tx.push(PlaybackCommand::LoadFont { id, bytes });
            }
        }
    }
}

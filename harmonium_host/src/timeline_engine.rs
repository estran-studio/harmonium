//! Timeline-based engine - separates generation (Writehead) from playback (Playhead)
//!
//! This engine replaces the monolithic tick() approach with:
//! - A Writehead that generates measures ahead of playback (main thread)
//! - A Playhead that reads measures and emits AudioEvents (audio thread)
//! - A ring buffer connecting them (lock-free SPSC)
//!
//! The legacy engine is preserved as `engine.rs` for A/B comparison.

use std::sync::{Arc, Mutex};

use arrayvec::ArrayString;
use harmonium_audio::backend::AudioRenderer;
use harmonium_core::{
    events::AudioEvent,
    harmony::{Chord, ChordType, HarmonicDriver, HarmonyNavigator},
    log,
    params::{CurrentState, EngineParams, MusicalParams, SessionConfig, TimeSignature},
    sequencer::Sequencer,
    timeline::{GenerationContext, Measure, Playhead, TimelineGenerator, Writehead},
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_music_theory::{note::PitchSymbol, scale::ScaleType};

use crate::mapper::EmotionMapper;

/// Timeline-based engine that separates music generation from audio playback.
///
/// The Writehead runs on the main thread, generating measures ahead of time.
/// The Playhead runs on the audio thread, reading measures and emitting events.
pub struct TimelineEngine {
    pub config: SessionConfig,

    // Communication queues (same as legacy engine)
    command_rx: rtrb::Consumer<harmonium_core::EngineCommand>,
    report_tx: rtrb::Producer<harmonium_core::EngineReport>,
    pub font_queue: crate::FontQueue,

    // === WRITEHEAD (main thread generation) ===
    generator: TimelineGenerator,
    writehead: Writehead,

    // === PLAYHEAD (audio thread playback) ===
    playhead: Playhead,

    // === MEASURE RING BUFFER (Writehead → Playhead) ===
    measure_tx: rtrb::Producer<Measure>,
    measure_rx: rtrb::Consumer<Measure>,

    // === AUDIO RENDERER ===
    renderer: Box<dyn AudioRenderer>,
    sample_rate: f64,

    // === TIMING ===
    sample_counter: usize,
    samples_per_step: usize,

    // === STATE ===
    musical_params: MusicalParams,
    rng: ChaCha8Rng,
    params_dirty: bool,

    // Seed + init params (for deterministic seek replay)
    session_seed: u64,
    init_key: PitchSymbol,
    init_scale: ScaleType,
    init_key_pc: u8,

    // === STYLE TUNING ===
    tuning: TuningParams,

    // === BPM OVERRIDE ===
    bpm_override: Option<f32>,
    emotion_mapped_bpm: f32,

    // === EMOTION MODE ===
    emotion_mapper: EmotionMapper,
    emotion_mode: bool,
    cached_emotions: EngineParams,

    // === REPORT CACHE ===
    last_chord_name: ArrayString<64>,
    last_chord_root_offset: i32,
    last_chord_is_minor: bool,

    // Recording state
    is_recording_wav: bool,
    is_recording_midi: bool,
    is_recording_musicxml: bool,

    // Mute tracking
    last_muted_channels: Vec<bool>,

    // Loop region (None = no loop)
    loop_region: Option<(usize, usize)>, // (start_bar, end_bar) inclusive

    // When true, skip real-time safety checks (for offline rendering)
    offline: bool,

    // When true, zero the output buffer after processing (silent pre-generation)
    output_muted: bool,

    // Pending measure snapshots to include in the next report
    pending_measure_snapshots: Vec<harmonium_core::report::MeasureSnapshot>,

    // Pending note events to include in the next report (for VST MIDI output)
    pending_notes: Vec<harmonium_core::report::NoteEvent>,

    // Current sample offset within process_buffer (for accurate MIDI timing)
    current_sample_offset: u32,

    // True once the playhead has loaded its first measure.
    // Before that, output is zeroed to suppress DSP-graph init transients.
    first_measure_loaded: bool,
}

impl TimelineEngine {
    pub fn new(
        sample_rate: f64,
        command_rx: rtrb::Consumer<harmonium_core::EngineCommand>,
        report_tx: rtrb::Producer<harmonium_core::EngineReport>,
        renderer: Box<dyn AudioRenderer>,
    ) -> Self {
        use rand::Rng;
        let session_seed: u64 = rand::thread_rng().r#gen();
        Self::new_with_seed(sample_rate, command_rx, report_tx, renderer, session_seed)
    }

    /// Set initial channel routing (e.g. for SoundFont: route channels to OxiSynth banks).
    /// Must be called before the audio stream starts processing.
    pub fn set_channel_routing(&mut self, routing: &[i32]) {
        for (i, &bank) in routing.iter().enumerate() {
            if i < self.musical_params.channel_routing.len() {
                self.musical_params.channel_routing[i] = bank;
            }
        }
    }

    /// Create timeline engine with explicit seed for deterministic/reproducible output.
    pub fn new_with_seed(
        sample_rate: f64,
        command_rx: rtrb::Consumer<harmonium_core::EngineCommand>,
        report_tx: rtrb::Producer<harmonium_core::EngineReport>,
        mut renderer: Box<dyn AudioRenderer>,
        session_seed: u64,
    ) -> Self {
        use rand::Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(session_seed);

        let font_queue = Arc::new(Mutex::new(Vec::new()));
        let bpm = 120.0;
        let steps = 16;
        let initial_pulses = 4;

        let keys = [
            PitchSymbol::C,
            PitchSymbol::D,
            PitchSymbol::E,
            PitchSymbol::F,
            PitchSymbol::G,
            PitchSymbol::A,
            PitchSymbol::B,
        ];
        let scales = [ScaleType::PentatonicMinor, ScaleType::PentatonicMajor];
        let random_key = keys[rng.gen_range(0..keys.len())];
        let random_scale = scales[rng.gen_range(0..scales.len())];

        let config = SessionConfig {
            bpm,
            key: format!("{}", random_key),
            scale: format!("{:?}", random_scale),
            pulses: initial_pulses,
            steps,
        };

        log::info(&format!(
            "Timeline Engine - Session: {} {} | BPM: {:.1}",
            config.key, config.scale, bpm
        ));

        // Initialize sequencers (match legacy engine initialization)
        let sequencer_primary = Sequencer::new(steps, initial_pulses, bpm);
        let default_density = 0.4;
        let secondary_pulses = std::cmp::min((default_density * 8.0) as usize + 1, 12);
        let sequencer_secondary = Sequencer::new_with_rotation(12, secondary_pulses, bpm, 0);

        // Initialize harmony
        let harmony = HarmonyNavigator::new(random_key, random_scale, 4);
        let key_pc = match random_key {
            PitchSymbol::C => 0,
            PitchSymbol::D => 2,
            PitchSymbol::E => 4,
            PitchSymbol::F => 5,
            PitchSymbol::G => 7,
            PitchSymbol::A => 9,
            PitchSymbol::B => 11,
            _ => 0,
        };
        let tuning = TuningParams::default();
        let harmonic_driver = Some(HarmonicDriver::new(key_pc, &tuning.harmony_driver));

        let mut musical_params = MusicalParams::default();
        // The session key drives the relative→absolute conversion of every
        // chord root (issue #26) — it must match the navigator/driver key
        // from the very first bar, not default to C.
        musical_params.key_root = key_pc;

        let initial_state = CurrentState {
            bpm,
            density: musical_params.rhythm_density,
            tension: musical_params.rhythm_tension,
            smoothness: musical_params.melody_smoothness,
            ..CurrentState::default()
        };

        // Create the generator
        let generator = TimelineGenerator::new(
            sequencer_primary,
            sequencer_secondary,
            harmony,
            harmonic_driver,
            musical_params.clone(),
            initial_state,
            tuning.clone(),
        );

        // Create writehead and playhead
        let writehead = Writehead::new(sample_rate, 4);
        let playhead = Playhead::new(sample_rate, 4);

        // Ring buffer for measures (64 slots — enough for pre-generation + playback runway)
        let (measure_tx, measure_rx) = rtrb::RingBuffer::<Measure>::new(64);

        let samples_per_step = (sample_rate * 60.0 / (bpm as f64) / 4.0) as usize;
        renderer.handle_event(AudioEvent::TimingUpdate { samples_per_step });

        Self {
            config,
            command_rx,
            report_tx,
            font_queue,
            generator,
            writehead,
            playhead,
            measure_tx,
            measure_rx,
            renderer,
            sample_rate,
            sample_counter: 0,
            samples_per_step,
            musical_params,
            rng,
            params_dirty: false,
            session_seed,
            init_key: random_key,
            init_scale: random_scale,
            init_key_pc: key_pc,
            tuning,
            bpm_override: None,
            emotion_mapped_bpm: bpm,
            emotion_mapper: EmotionMapper::new(),
            emotion_mode: false,
            cached_emotions: EngineParams::default(),
            last_chord_name: ArrayString::from(&Chord::new(key_pc, ChordType::Major).name())
                .unwrap_or_default(),
            last_chord_root_offset: 0,
            last_chord_is_minor: false,
            is_recording_wav: false,
            is_recording_midi: false,
            is_recording_musicxml: false,
            last_muted_channels: vec![false; 16],
            loop_region: None,
            offline: false,
            output_muted: false,
            pending_measure_snapshots: Vec::new(),
            pending_notes: Vec::with_capacity(16),
            current_sample_offset: 0,
            first_measure_loaded: false,
        }
    }

    /// Enable offline mode (skip real-time safety checks for non-realtime rendering)
    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }

    /// Main thread: generate measures ahead of playback and push to ring buffer
    pub fn generate_ahead(&mut self) {
        let playhead_bar = self.playhead.current_bar();

        while self.writehead.needs_generation(playhead_bar) {
            let bar_idx = self.writehead.current_bar;

            // If this bar already exists in the timeline (e.g. after SeekPlayhead),
            // re-push the existing measure instead of generating a new one.
            if let Some(existing) = self.writehead.timeline.get_measure(bar_idx) {
                if self.measure_tx.push(existing.clone()).is_err() {
                    break; // Ring buffer full
                }
                // Snapshot re-pushed bars so poll_measures() sees them
                self.pending_measure_snapshots
                    .push(harmonium_core::report::MeasureSnapshot::from_measure(&existing));
                self.writehead.current_bar = bar_idx + 1;
                continue;
            }

            let measure = self.generator.generate_measure(bar_idx, &mut self.rng);

            // Update report cache from generated measure
            self.last_chord_name =
                ArrayString::from(&measure.chord_context.chord_name).unwrap_or_default();
            self.last_chord_root_offset = measure.chord_context.root_offset;
            self.last_chord_is_minor = measure.chord_context.is_minor;

            // Try ring buffer first — if full, stop (don't snapshot yet)
            match self.measure_tx.push(measure.clone()) {
                Ok(()) => {}
                Err(_) => {
                    // Ring buffer full - playhead hasn't consumed yet, skip
                    break;
                }
            }

            // Only snapshot AFTER successful ring buffer push (prevents duplicates)
            self.pending_measure_snapshots
                .push(harmonium_core::report::MeasureSnapshot::from_measure(&measure));

            // Store in master timeline
            self.writehead.commit_measure(measure);
        }
    }

    /// Build a GenerationContext from stored init params.
    fn generation_context(&self) -> GenerationContext {
        GenerationContext {
            session_seed: self.session_seed,
            key: self.init_key,
            scale: self.init_scale,
            key_pc: self.init_key_pc,
        }
    }

    /// Re-derive init key/scale from a seed and update stored init params.
    fn apply_seed(&mut self, seed: u64) {
        use rand::Rng;
        self.session_seed = seed;
        let mut init_rng = ChaCha8Rng::seed_from_u64(seed);
        let keys = [
            PitchSymbol::C,
            PitchSymbol::D,
            PitchSymbol::E,
            PitchSymbol::F,
            PitchSymbol::G,
            PitchSymbol::A,
            PitchSymbol::B,
        ];
        let scales = [ScaleType::PentatonicMinor, ScaleType::PentatonicMajor];
        self.init_key = keys[init_rng.gen_range(0..keys.len())];
        self.init_scale = scales[init_rng.gen_range(0..scales.len())];
        self.init_key_pc = match self.init_key {
            PitchSymbol::C => 0,
            PitchSymbol::D => 2,
            PitchSymbol::E => 4,
            PitchSymbol::F => 5,
            PitchSymbol::G => 7,
            PitchSymbol::A => 9,
            PitchSymbol::B => 11,
            _ => 0,
        };
        // Keep the relative→absolute chord-root conversion on the new key
        // (issue #26).
        self.musical_params.key_root = self.init_key_pc;
        self.params_dirty = true;
    }

    /// Full reset: stop notes, reset playhead, drain buffer, reset writehead/loop,
    /// then deterministic seek to bar 1.
    fn full_reset_to_bar1(&mut self) {
        self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 0 });
        self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 1 });
        self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 2 });
        self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 3 });
        self.playhead.seek_to_bar(1);
        while self.measure_rx.pop().is_ok() {}
        self.writehead.reset();
        self.loop_region = None;
        self.deterministic_seek(1);
    }

    /// Deterministic seek: reset RNG + generator, replay to target bar.
    ///
    /// Ensures the generator and RNG are in the exact state for `target_bar`
    /// as if generation had proceeded linearly from bar 1.
    fn deterministic_seek(&mut self, target_bar: usize) {
        use rand::Rng;

        // 1. Re-seed RNG
        let mut rng = ChaCha8Rng::seed_from_u64(self.session_seed);

        // 2. Consume the same init draws as new_with_seed()
        let keys_len = 7usize;
        let scales_len = 2usize;
        let _ = rng.gen_range(0..keys_len);
        let _ = rng.gen_range(0..scales_len);

        // 3. Reset generator
        let ctx = self.generation_context();
        self.generator.reset_to_initial(&ctx);

        // 4. Apply current musical params (update_controls does this every buffer,
        //    so linear generation always has these applied before generate_measure)
        self.generator.update_params(self.musical_params.clone());

        // 5. Silent advance
        if target_bar > 1 {
            self.generator.silent_advance(target_bar, &mut rng);
        }

        // 6. Store the RNG and set writehead position
        self.rng = rng;
        self.writehead.current_bar = target_bar;
    }

    /// Audio thread: process a buffer of audio samples
    pub fn process_buffer(&mut self, output: &mut [f32], channels: usize) {
        if !self.offline {
            crate::realtime::rt_check::enter_audio_context();
        }

        let total_samples = output.len() / channels;
        let mut processed = 0;

        // Process commands and generate ahead (main thread work)
        if !self.offline {
            crate::realtime::rt_check::exit_audio_context();
        }
        self.update_controls();
        self.generate_ahead();
        if !self.offline {
            crate::realtime::rt_check::enter_audio_context();
        }

        while processed < total_samples {
            let remaining = total_samples - processed;
            let samples_until_tick = if self.samples_per_step > self.sample_counter {
                self.samples_per_step - self.sample_counter
            } else {
                1
            };

            let chunk_size = std::cmp::min(remaining, samples_until_tick);
            let start_idx = processed * channels;
            let end_idx = (processed + chunk_size) * channels;
            let chunk = &mut output[start_idx..end_idx];

            self.renderer.process_buffer(chunk, channels);
            self.sample_counter += chunk_size;
            processed += chunk_size;

            if self.sample_counter >= self.samples_per_step {
                self.sample_counter = 0;
                self.current_sample_offset = processed as u32;
                if !self.offline {
                    crate::realtime::rt_check::exit_audio_context();
                }
                self.tick();
                if !self.offline {
                    crate::realtime::rt_check::enter_audio_context();
                }
            }
        }

        // Zero output when muted or before the first measure is loaded
        // (suppresses DSP-graph initialization transients)
        if self.output_muted || !self.first_measure_loaded {
            for sample in output.iter_mut() {
                *sample = 0.0;
            }
        }

        if !self.offline {
            crate::realtime::rt_check::exit_audio_context();
        }
    }

    /// Audio thread tick: read from playhead and emit events
    fn tick(&mut self) {
        // Check loop region: if playhead crossed end_bar, seek back to start_bar
        if let Some((start_bar, end_bar)) = self.loop_region {
            if self.playhead.current_bar() > end_bar {
                // Stop active notes before looping
                self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 0 });
                self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 1 });
                self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 2 });
                self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 3 });
                self.playhead.seek_to_bar(start_bar);
                // Drain ring buffer
                while self.measure_rx.pop().is_ok() {}
                // Re-push committed timeline measures into ring buffer (deterministic loop)
                for bar in start_bar..=end_bar {
                    if let Some(m) = self.writehead.timeline.get_measure(bar) {
                        if self.measure_tx.push(m.clone()).is_err() {
                            break;
                        }
                    }
                }
                // Set writehead past the loop region so generate_ahead doesn't
                // re-generate loop bars (they're already in the ring buffer)
                self.writehead.current_bar = end_bar + 1;
            }
        }

        // If playhead needs a new measure, try to read from ring buffer
        if self.playhead.needs_measure() {
            if let Ok(measure) = self.measure_rx.pop() {
                // Update timing from measure tempo
                let steps_per_beat = 4.0f64;
                let new_sps =
                    (self.sample_rate * 60.0 / (measure.tempo as f64) / steps_per_beat) as usize;
                if new_sps != self.samples_per_step {
                    self.samples_per_step = new_sps;
                    self.renderer
                        .handle_event(AudioEvent::TimingUpdate { samples_per_step: new_sps });
                }

                self.playhead.load_measure(measure);
                self.first_measure_loaded = true;
            } else {
                // No measure available - silence (underflow)
                return;
            }
        }

        // Tick the playhead to get events for this step
        let events = self.playhead.tick();

        // Forward events to renderer, filtering out NoteOn for muted channels
        // Also capture note events for VST MIDI output
        for event in events {
            if let AudioEvent::NoteOn { channel, .. } = &event {
                if self
                    .musical_params
                    .muted_channels
                    .get(*channel as usize)
                    .copied()
                    .unwrap_or(false)
                {
                    continue;
                }
            }
            // Capture NoteOn/NoteOff for report (VST MIDI output)
            match &event {
                AudioEvent::NoteOn { note, velocity, channel } => {
                    self.pending_notes.push(harmonium_core::report::NoteEvent {
                        note_midi: *note,
                        velocity: *velocity,
                        channel: *channel,
                        is_note_on: true,
                    });
                }
                AudioEvent::NoteOff { note, channel } => {
                    self.pending_notes.push(harmonium_core::report::NoteEvent {
                        note_midi: *note,
                        velocity: 0,
                        channel: *channel,
                        is_note_on: false,
                    });
                }
                _ => {}
            }
            self.renderer.handle_event(event.clone());
        }

        // Send report every 2 steps (8th note resolution) for smooth visualization,
        // or immediately when note events are pending
        let current_step = self.playhead.position.step_in_bar(4);
        if current_step.is_multiple_of(2) || !self.pending_notes.is_empty() {
            self.send_report();
        }
    }

    /// Process pending commands
    fn process_commands(&mut self) {
        use harmonium_core::EngineCommand;

        while let Ok(cmd) = self.command_rx.pop() {
            match cmd {
                EngineCommand::SetBpm(bpm) => {
                    let clamped = bpm.clamp(70.0, 180.0);
                    self.bpm_override = Some(clamped);
                    self.musical_params.bpm = clamped;
                    let steps_per_beat = 4.0f64;
                    let new_sps = (self.sample_rate * 60.0
                        / (self.musical_params.bpm as f64)
                        / steps_per_beat) as usize;
                    if new_sps != self.samples_per_step {
                        self.samples_per_step = new_sps;
                        self.renderer
                            .handle_event(AudioEvent::TimingUpdate { samples_per_step: new_sps });
                    }
                    self.params_dirty = true;
                }
                EngineCommand::ResetBpm => {
                    self.bpm_override = None;
                    self.musical_params.bpm = self.emotion_mapped_bpm;
                    let steps_per_beat = 4.0f64;
                    let new_sps = (self.sample_rate * 60.0
                        / (self.musical_params.bpm as f64)
                        / steps_per_beat) as usize;
                    if new_sps != self.samples_per_step {
                        self.samples_per_step = new_sps;
                        self.renderer
                            .handle_event(AudioEvent::TimingUpdate { samples_per_step: new_sps });
                    }
                    self.params_dirty = true;
                }
                EngineCommand::SetMasterVolume(volume) => {
                    self.musical_params.master_volume = volume.clamp(0.0, 1.0);
                }
                EngineCommand::SetTimeSignature { numerator, denominator } => {
                    self.musical_params.time_signature = TimeSignature { numerator, denominator };
                    self.params_dirty = true;
                }
                EngineCommand::EnableRhythm(e) => {
                    self.musical_params.enable_rhythm = e;
                    self.params_dirty = true;
                }
                EngineCommand::EnableHarmony(e) => {
                    self.musical_params.enable_harmony = e;
                    self.params_dirty = true;
                }
                EngineCommand::EnableMelody(e) => {
                    self.musical_params.enable_melody = e;
                    self.params_dirty = true;
                }
                EngineCommand::EnableVoicing(e) => {
                    self.musical_params.enable_voicing = e;
                    self.params_dirty = true;
                }
                EngineCommand::SetRhythmMode(m) => {
                    self.musical_params.rhythm_mode = m;
                    self.params_dirty = true;
                }
                EngineCommand::SetRhythmSteps(s) => {
                    self.musical_params.rhythm_steps = s;
                    self.params_dirty = true;
                }
                EngineCommand::SetRhythmPulses(p) => {
                    self.musical_params.rhythm_pulses = p;
                    self.params_dirty = true;
                }
                EngineCommand::SetRhythmRotation(r) => {
                    self.musical_params.rhythm_rotation = r;
                    self.params_dirty = true;
                }
                EngineCommand::SetRhythmDensity(d) => {
                    self.musical_params.rhythm_density = d.clamp(0.0, 1.0);
                    self.params_dirty = true;
                }
                EngineCommand::SetRhythmTension(t) => {
                    self.musical_params.rhythm_tension = t.clamp(0.0, 1.0);
                    self.params_dirty = true;
                }
                EngineCommand::SetRhythmSecondary { steps, pulses, rotation } => {
                    self.musical_params.rhythm_secondary_steps = steps;
                    self.musical_params.rhythm_secondary_pulses = pulses;
                    self.musical_params.rhythm_secondary_rotation = rotation;
                    self.params_dirty = true;
                }
                EngineCommand::SetFixedKick(f) => {
                    self.musical_params.fixed_kick = f;
                    self.params_dirty = true;
                }
                EngineCommand::SetHarmonyMode(m) => {
                    self.musical_params.harmony_mode = m;
                    self.params_dirty = true;
                }
                EngineCommand::SetHarmonyStrategy(s) => {
                    self.musical_params.harmony_strategy = s;
                    self.params_dirty = true;
                }
                EngineCommand::SetHarmonyTension(t) => {
                    self.musical_params.harmony_tension = t.clamp(0.0, 1.0);
                    self.params_dirty = true;
                }
                EngineCommand::SetHarmonyValence(v) => {
                    self.musical_params.harmony_valence = v.clamp(-1.0, 1.0);
                    self.params_dirty = true;
                }
                EngineCommand::SetHarmonyMeasuresPerChord(m) => {
                    self.musical_params.harmony_measures_per_chord = m;
                    self.params_dirty = true;
                }
                EngineCommand::SetKeyRoot(r) => {
                    // A key change is a session re-key: navigator scale,
                    // harmonic driver and key_root must move together or
                    // chord labels/audio desync (issue #26).
                    let key_pc = r % 12;
                    let symbol = harmonium_core::params::key_root_to_pitch_symbol(key_pc);
                    self.init_key = symbol;
                    self.init_key_pc = key_pc;
                    self.config.key = format!("{symbol}");
                    self.musical_params.key_root = key_pc;
                    self.generator.harmony = HarmonyNavigator::new(symbol, self.init_scale, 4);
                    if let Some(driver) = self.generator.harmonic_driver.as_mut() {
                        driver.set_key(key_pc);
                    }
                    self.params_dirty = true;
                }
                EngineCommand::SetMelodySmoothness(s) => {
                    self.musical_params.melody_smoothness = s.clamp(0.0, 1.0);
                    self.params_dirty = true;
                }
                EngineCommand::SetMelodyOctave(o) => {
                    self.musical_params.melody_octave = o.clamp(3, 6);
                    self.params_dirty = true;
                }
                EngineCommand::SetVoicingDensity(d) => {
                    self.musical_params.voicing_density = d.clamp(0.0, 1.0);
                    self.params_dirty = true;
                }
                EngineCommand::SetVoicingTension(t) => {
                    self.musical_params.voicing_tension = t.clamp(0.0, 1.0);
                    self.params_dirty = true;
                }
                EngineCommand::SetChannelGain { channel, gain } => match channel {
                    0 => self.musical_params.gain_bass = gain.clamp(0.0, 1.0),
                    1 => self.musical_params.gain_lead = gain.clamp(0.0, 1.0),
                    2 => self.musical_params.gain_snare = gain.clamp(0.0, 1.0),
                    3 => self.musical_params.gain_hat = gain.clamp(0.0, 1.0),
                    _ => {}
                },
                EngineCommand::SetChannelMute { channel, muted } => {
                    if (channel as usize) < self.musical_params.muted_channels.len() {
                        self.musical_params.muted_channels[channel as usize] = muted;
                    }
                }
                EngineCommand::SetChannelRoute { channel, bank_id } => {
                    if (channel as usize) < self.musical_params.channel_routing.len() {
                        self.musical_params.channel_routing[channel as usize] = bank_id;
                    }
                }
                EngineCommand::SetVelocityBase { channel, velocity } => match channel {
                    0 => self.musical_params.vel_base_bass = velocity,
                    2 => self.musical_params.vel_base_snare = velocity,
                    _ => {}
                },
                EngineCommand::StartRecording(format) => match format {
                    harmonium_core::events::RecordFormat::Wav => {
                        self.musical_params.record_wav = true
                    }
                    harmonium_core::events::RecordFormat::Midi => {
                        self.musical_params.record_midi = true
                    }
                    harmonium_core::events::RecordFormat::MusicXml => {
                        self.musical_params.record_musicxml = true
                    }
                },
                EngineCommand::StopRecording(format) => match format {
                    harmonium_core::events::RecordFormat::Wav => {
                        self.musical_params.record_wav = false
                    }
                    harmonium_core::events::RecordFormat::Midi => {
                        self.musical_params.record_midi = false
                    }
                    harmonium_core::events::RecordFormat::MusicXml => {
                        self.musical_params.record_musicxml = false
                    }
                },
                EngineCommand::UseEmotionMode => {
                    self.emotion_mode = true;
                    log::info("Timeline Engine: switched to Emotion mode");
                }
                EngineCommand::UseDirectMode => {
                    self.emotion_mode = false;
                    log::info("Timeline Engine: switched to Direct mode");
                }
                EngineCommand::SetEmotionParams { arousal, valence, density, tension } => {
                    if self.emotion_mode {
                        // Update cached emotions with the 4 axis values
                        self.cached_emotions.arousal = arousal;
                        self.cached_emotions.valence = valence;
                        self.cached_emotions.density = density;
                        self.cached_emotions.tension = tension;

                        // Map emotions → musical params via EmotionMapper
                        let mapped = self.emotion_mapper.map(&self.cached_emotions);

                        // Preserve runtime state that shouldn't be overwritten by the mapper
                        let mut new_params = mapped;
                        new_params.enable_rhythm = self.musical_params.enable_rhythm;
                        new_params.enable_harmony = self.musical_params.enable_harmony;
                        new_params.enable_melody = self.musical_params.enable_melody;
                        new_params.enable_voicing = self.musical_params.enable_voicing;
                        new_params.record_wav = self.musical_params.record_wav;
                        new_params.record_midi = self.musical_params.record_midi;
                        new_params.record_musicxml = self.musical_params.record_musicxml;
                        new_params.muted_channels = self.musical_params.muted_channels.clone();
                        new_params.channel_routing = self.musical_params.channel_routing.clone();
                        // Preserve per-channel gains set via mixer UI
                        new_params.gain_lead = self.musical_params.gain_lead;
                        new_params.gain_bass = self.musical_params.gain_bass;
                        new_params.gain_snare = self.musical_params.gain_snare;
                        new_params.gain_hat = self.musical_params.gain_hat;

                        // Store emotion-mapped BPM, then apply override if set
                        self.emotion_mapped_bpm = new_params.bpm;
                        if let Some(override_bpm) = self.bpm_override {
                            new_params.bpm = override_bpm;
                        }

                        // Apply to generator (updates sequencers, harmony, etc.)
                        self.generator.update_params(new_params.clone());
                        self.musical_params = new_params;

                        // Sync audio timing with mapped BPM
                        let steps_per_beat = 4.0f64;
                        let new_sps = (self.sample_rate * 60.0
                            / (self.musical_params.bpm as f64)
                            / steps_per_beat) as usize;
                        if new_sps != self.samples_per_step {
                            self.samples_per_step = new_sps;
                            self.renderer.handle_event(AudioEvent::TimingUpdate {
                                samples_per_step: new_sps,
                            });
                        }

                        self.params_dirty = true;

                        log::info(&format!(
                            "Emotion mapped: arousal={:.2} valence={:.2} density={:.2} tension={:.2} → bpm={:.0} strategy={:?}",
                            arousal,
                            valence,
                            density,
                            tension,
                            self.musical_params.bpm,
                            self.musical_params.harmony_strategy
                        ));
                    }
                }
                EngineCommand::SetAllRhythmParams {
                    mode,
                    steps,
                    pulses,
                    rotation,
                    density,
                    tension,
                    secondary_steps,
                    secondary_pulses,
                    secondary_rotation,
                } => {
                    self.musical_params.rhythm_mode = mode;
                    self.musical_params.rhythm_steps = steps;
                    self.musical_params.rhythm_pulses = pulses;
                    self.musical_params.rhythm_rotation = rotation;
                    self.musical_params.rhythm_density = density.clamp(0.0, 1.0);
                    self.musical_params.rhythm_tension = tension.clamp(0.0, 1.0);
                    self.musical_params.rhythm_secondary_steps = secondary_steps;
                    self.musical_params.rhythm_secondary_pulses = secondary_pulses;
                    self.musical_params.rhythm_secondary_rotation = secondary_rotation;
                    self.params_dirty = true;
                }
                EngineCommand::Seek(bar) => {
                    let target_bar = bar.max(1);
                    log::info(&format!("Deterministic seek to bar {target_bar}"));
                    // Stop active notes
                    self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 0 });
                    self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 1 });
                    self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 2 });
                    self.renderer.handle_event(AudioEvent::AllNotesOff { channel: 3 });
                    // Seek playhead
                    self.playhead.seek_to_bar(target_bar);
                    // Drain the measure ring buffer
                    while self.measure_rx.pop().is_ok() {}
                    // Deterministic: reset RNG + generator, replay to target
                    self.deterministic_seek(target_bar);
                }
                EngineCommand::NewMelody => {
                    use rand::Rng;
                    let new_seed: u64 = rand::thread_rng().r#gen();
                    log::info(&format!("New melody with seed {new_seed}"));
                    self.apply_seed(new_seed);
                    self.full_reset_to_bar1();
                }
                EngineCommand::SetSeed(seed) => {
                    log::info(&format!("Set seed to {seed}"));
                    self.apply_seed(seed);
                    self.full_reset_to_bar1();
                }
                EngineCommand::SetLoop { start_bar, end_bar } => {
                    let start = start_bar.max(1);
                    let end = end_bar.max(start);
                    log::info(&format!("Loop set: bars {start}-{end}"));
                    self.loop_region = Some((start, end));
                }
                EngineCommand::ClearLoop => {
                    log::info("Loop cleared");
                    self.loop_region = None;
                }
                EngineCommand::ExportTimeline(format) => {
                    log::info(&format!("Timeline export requested: {:?}", format));
                    match format {
                        harmonium_core::events::RecordFormat::MusicXml => {
                            let xml =
                                harmonium_core::timeline::timeline_to_musicxml_with_instruments(
                                    &self.writehead.timeline,
                                    "Harmonium Export",
                                    &self.musical_params.instrument_lead,
                                    &self.musical_params.instrument_bass,
                                );
                            if let Ok(()) = std::fs::write("timeline_export.musicxml", &xml) {
                                log::info(&format!(
                                    "Timeline exported to timeline_export.musicxml ({} bytes)",
                                    xml.len()
                                ));
                            }
                        }
                        _ => {
                            log::warn(&format!(
                                "Timeline export only supports MusicXML, got {:?}",
                                format
                            ));
                        }
                    }
                }
                EngineCommand::SetOutputMute(muted) => {
                    self.output_muted = muted;
                    if muted {
                        // Stop any active notes so the un-mute is clean
                        for ch in 0..4u8 {
                            self.renderer.handle_event(AudioEvent::AllNotesOff { channel: ch });
                        }
                    }
                }
                EngineCommand::SeekPlayhead(bar) => {
                    let target_bar = bar.max(1);
                    log::info(&format!("SeekPlayhead to bar {target_bar}"));
                    // Stop active notes
                    for ch in 0..4u8 {
                        self.renderer.handle_event(AudioEvent::AllNotesOff { channel: ch });
                    }
                    // Reset playhead to target bar
                    self.playhead.seek_to_bar(target_bar);
                    // Drain ring buffer, re-fill from writehead's committed timeline
                    while self.measure_rx.pop().is_ok() {}
                    let end_bar = (target_bar + 8).min(self.writehead.current_bar);
                    for b in target_bar..end_bar {
                        if let Some(measure) = self.writehead.timeline.get_measure(b) {
                            if self.measure_tx.push(measure.clone()).is_err() {
                                break;
                            }
                        }
                    }
                    // Reset writehead to continue from where the ring buffer fill
                    // stopped, so generate_ahead() re-pushes existing bars 9-16+
                    // before generating new ones.
                    self.writehead.current_bar = end_bar;
                }
                EngineCommand::SetWriteheadLookahead(n) => {
                    self.writehead.lookahead = n.max(4);
                }
                EngineCommand::GetState => {}
                EngineCommand::Reset => {
                    self.musical_params = MusicalParams::default();
                    while self.measure_rx.pop().is_ok() {}
                    self.playhead.reset();
                    self.writehead.reset();
                    self.loop_region = None;
                }
                EngineCommand::SetStyleProfile(ref profile) => {
                    self.tuning = profile.to_tuning_params();
                    self.generator.update_tuning(self.tuning.clone());
                    self.params_dirty = true;
                }
                EngineCommand::ClearStyleProfile => {
                    self.tuning = TuningParams::default();
                    self.generator.update_tuning(self.tuning.clone());
                    self.params_dirty = true;
                }
            }
        }
    }

    /// Update controls and sync generator with latest params
    fn update_controls(&mut self) {
        self.process_commands();

        // If musical params changed, flush stale measures from the ring buffer
        // so generate_ahead() regenerates them with the new params.
        if self.params_dirty {
            self.params_dirty = false;
            // If the playhead hasn't loaded a measure yet, regenerate from its
            // current bar; otherwise regenerate from the next bar.
            let regen_from = if self.playhead.needs_measure() {
                self.playhead.current_bar()
            } else {
                self.playhead.current_bar() + 1
            };
            // Stop active notes to avoid lingering sounds from old params
            for ch in 0..4u8 {
                self.renderer.handle_event(AudioEvent::AllNotesOff { channel: ch });
            }
            // Drain ring buffer
            while self.measure_rx.pop().is_ok() {}
            // Invalidate future measures in the timeline so they get regenerated
            self.writehead.timeline.invalidate_from(regen_from);
            // Reset writehead to regenerate from regen_from
            self.writehead.current_bar = regen_from;
        }

        // Sync generator with updated params
        self.generator.update_params(self.musical_params.clone());

        // Load fonts
        if let Ok(mut queue) = self.font_queue.try_lock() {
            while let Some((id, bytes)) = queue.pop() {
                self.renderer.handle_event(AudioEvent::LoadFont { id, bytes });
            }
        }

        // Sync routing
        for (i, &mode) in self.musical_params.channel_routing.iter().enumerate() {
            if i < 16 {
                self.renderer
                    .handle_event(AudioEvent::SetChannelRoute { channel: i as u8, bank: mode });
            }
        }

        // Mute control
        for (i, &is_muted) in self.musical_params.muted_channels.iter().enumerate() {
            if i < 16 && i < self.last_muted_channels.len() {
                if is_muted && !self.last_muted_channels[i] {
                    self.renderer.handle_event(AudioEvent::AllNotesOff { channel: i as u8 });
                }
                self.last_muted_channels[i] = is_muted;
            }
        }

        // Mixer gains
        self.renderer.handle_event(AudioEvent::SetMixerGains {
            lead: self.musical_params.gain_lead,
            bass: self.musical_params.gain_bass,
            snare: self.musical_params.gain_snare,
            hat: self.musical_params.gain_hat,
        });

        // Recording control
        self.sync_recording();

        // Timing update
        let steps_per_beat = 4.0f64;
        let new_sps =
            (self.sample_rate * 60.0 / (self.musical_params.bpm as f64) / steps_per_beat) as usize;
        if new_sps != self.samples_per_step {
            self.samples_per_step = new_sps;
            self.renderer.handle_event(AudioEvent::TimingUpdate { samples_per_step: new_sps });
        }
    }

    fn sync_recording(&mut self) {
        let mp = &self.musical_params;

        if mp.record_wav != self.is_recording_wav {
            self.is_recording_wav = mp.record_wav;
            let fmt = harmonium_core::events::RecordFormat::Wav;
            if self.is_recording_wav {
                self.renderer.handle_event(AudioEvent::StartRecording { format: fmt });
            } else {
                self.renderer.handle_event(AudioEvent::StopRecording { format: fmt });
            }
        }

        if mp.record_midi != self.is_recording_midi {
            self.is_recording_midi = mp.record_midi;
            let fmt = harmonium_core::events::RecordFormat::Midi;
            if self.is_recording_midi {
                self.renderer.handle_event(AudioEvent::StartRecording { format: fmt });
            } else {
                self.renderer.handle_event(AudioEvent::StopRecording { format: fmt });
            }
        }

        if mp.record_musicxml != self.is_recording_musicxml {
            self.is_recording_musicxml = mp.record_musicxml;
            let fmt = harmonium_core::events::RecordFormat::MusicXml;
            if self.is_recording_musicxml {
                self.renderer
                    .handle_event(AudioEvent::UpdateMusicalParams { params: Box::new(mp.clone()) });
                self.renderer.handle_event(AudioEvent::StartRecording { format: fmt });
            } else {
                self.renderer.handle_event(AudioEvent::StopRecording { format: fmt });
            }
        }
    }

    fn send_report(&mut self) {
        use harmonium_core::EngineReport;

        let mut report = EngineReport::new();

        report.current_bar = self.playhead.current_bar();
        report.current_beat = self.playhead.position.beat;
        // Map playhead position to sequencer step space.
        // step_in_bar(4) gives 0..15 for 4/4, but primary_steps may differ
        // (e.g. 48 for PerfectBalance). Scale proportionally.
        let raw_step = self.playhead.position.step_in_bar(4);
        let bar_steps_16 = self.musical_params.time_signature.steps_per_bar(4);
        let target_steps = self.musical_params.rhythm_steps;
        report.current_step = if bar_steps_16 > 0 && bar_steps_16 != target_steps {
            (raw_step * target_steps) / bar_steps_16
        } else {
            raw_step
        };
        report.time_signature = self.musical_params.time_signature;

        report.current_chord = self.last_chord_name.clone();
        report.chord_root_offset = self.last_chord_root_offset;
        report.chord_is_minor = self.last_chord_is_minor;
        report.harmony_mode = self.musical_params.harmony_mode;

        report.rhythm_mode = self.musical_params.rhythm_mode;
        report.primary_steps = self.musical_params.rhythm_steps;
        report.primary_pulses = self.musical_params.rhythm_pulses;
        report.primary_rotation = self.generator.sequencer_primary.rotation;
        report.secondary_steps = self.musical_params.rhythm_secondary_steps;
        report.secondary_pulses = self.musical_params.rhythm_secondary_pulses;
        report.secondary_rotation = self.generator.sequencer_secondary.rotation;

        // Export sequencer patterns as boolean arrays (any trigger = active)
        let prim_pat = &self.generator.sequencer_primary.pattern;
        for (i, trigger) in prim_pat.iter().enumerate() {
            if i < 192 {
                report.primary_pattern[i] = trigger.kick || trigger.snare || trigger.hat;
            }
        }
        let sec_pat = &self.generator.sequencer_secondary.pattern;
        for (i, trigger) in sec_pat.iter().enumerate() {
            if i < 192 {
                report.secondary_pattern[i] = trigger.kick || trigger.snare || trigger.hat;
            }
        }

        report.musical_params = self.musical_params.clone();
        report.session_key = ArrayString::from(&self.config.key).unwrap_or_default();
        report.session_scale = ArrayString::from(&self.config.scale).unwrap_or_default();

        // Drain pending note events into this report
        if !self.pending_notes.is_empty() {
            report.notes = std::mem::take(&mut self.pending_notes);
            self.pending_notes = Vec::with_capacity(16);
        }
        report.sample_offset = self.current_sample_offset;

        // Drain pending measure snapshots into this report
        if !self.pending_measure_snapshots.is_empty() {
            report.new_measures = std::mem::take(&mut self.pending_measure_snapshots);
        }

        let _ = self.report_tx.push(report);
    }
}

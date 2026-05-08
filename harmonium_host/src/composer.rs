//! MusicComposer - Main-thread music generation, decoupled from audio playback.
//!
//! Extracts the generation side of TimelineEngine: writehead, generator, musical params,
//! emotion mapping, and measure snapshots. Callable directly (no audio stream needed).

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use arrayvec::ArrayString;
use harmonium_core::{
    harmony::{HarmonicDriver, HarmonyNavigator},
    log,
    params::{CurrentState, EngineParams, MusicalParams, SessionConfig, TimeSignature},
    sequencer::Sequencer,
    timeline::{GenerationContext, Measure, TimelineGenerator, Writehead},
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_music_theory::{note::PitchSymbol, scale::ScaleType};

use crate::mapper::EmotionMapper;

/// Main-thread music composer. Generates measures synchronously.
///
/// This struct owns the generation pipeline: TimelineGenerator, Writehead,
/// MusicalParams, EmotionMapper, and the measure ring buffer producer.
/// It reads `playhead_bar` via an `Arc<AtomicUsize>` shared with PlaybackEngine.
pub struct MusicComposer {
    pub config: SessionConfig,

    // Generation
    generator: TimelineGenerator,
    writehead: Writehead,

    // Shared pages (Composer writes by index, PlaybackEngine reads by index)
    shared_pages: crate::SharedPages,

    // State
    musical_params: MusicalParams,
    rng: ChaCha8Rng,
    sample_rate: f64,

    // Seed + init params (for deterministic seek replay)
    session_seed: u64,
    init_key: PitchSymbol,
    init_scale: ScaleType,
    init_key_pc: u8,

    // BPM override (user-set BPM persists through emotion changes)
    bpm_override: Option<f32>,
    emotion_mapped_bpm: f32,

    // Emotion mode
    emotion_mapper: EmotionMapper,
    emotion_mode: bool,
    cached_emotions: EngineParams,

    // Report cache (chord info from last generated measure)
    last_chord_name: ArrayString<64>,
    last_chord_root_offset: i32,
    last_chord_is_minor: bool,

    // Pending measure snapshots for the frontend
    pending_measure_snapshots: Vec<harmonium_core::report::MeasureSnapshot>,

    // Shared playhead position (written by PlaybackEngine, read here)
    playhead_bar: Arc<AtomicUsize>,

    // Font queue (shared with PlaybackEngine via NativeHandle)
    pub font_queue: crate::FontQueue,
}

impl MusicComposer {
    /// Create a new MusicComposer.
    #[allow(clippy::type_complexity)]
    pub fn new(
        sample_rate: f64,
        shared_pages: crate::SharedPages,
        playhead_bar: Arc<AtomicUsize>,
        font_queue: crate::FontQueue,
    ) -> Self {
        use rand::Rng;
        let session_seed: u64 = rand::thread_rng().r#gen();
        Self::new_with_seed(sample_rate, shared_pages, playhead_bar, font_queue, session_seed)
    }

    /// Create with explicit seed for deterministic output.
    pub fn new_with_seed(
        sample_rate: f64,
        shared_pages: crate::SharedPages,
        playhead_bar: Arc<AtomicUsize>,
        font_queue: crate::FontQueue,
        session_seed: u64,
    ) -> Self {
        use rand::Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(session_seed);

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
            "MusicComposer - Session: {} {} | BPM: {:.1}",
            config.key, config.scale, bpm
        ));

        let sequencer_primary = Sequencer::new(steps, initial_pulses, bpm);
        let default_density = 0.4;
        let secondary_pulses = std::cmp::min((default_density * 8.0) as usize + 1, 12);
        let sequencer_secondary = Sequencer::new_with_rotation(12, secondary_pulses, bpm, 0);

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

        let musical_params = MusicalParams::default();

        let initial_state = CurrentState {
            bpm,
            density: musical_params.rhythm_density,
            tension: musical_params.rhythm_tension,
            smoothness: musical_params.melody_smoothness,
            ..CurrentState::default()
        };

        let generator = TimelineGenerator::new(
            sequencer_primary,
            sequencer_secondary,
            harmony,
            harmonic_driver,
            musical_params.clone(),
            initial_state,
            tuning,
        );

        let writehead = Writehead::new(sample_rate, 4);

        Self {
            config,
            generator,
            writehead,
            shared_pages,
            musical_params,
            rng,
            sample_rate,
            session_seed,
            init_key: random_key,
            init_scale: random_scale,
            init_key_pc: key_pc,
            bpm_override: None,
            emotion_mapped_bpm: bpm,
            emotion_mapper: EmotionMapper::new(),
            emotion_mode: false,
            cached_emotions: EngineParams::default(),
            last_chord_name: ArrayString::from("I").unwrap_or_default(),
            last_chord_root_offset: 0,
            last_chord_is_minor: false,
            pending_measure_snapshots: Vec::new(),
            playhead_bar,
            font_queue,
        }
    }

    /// Generate a specific number of bars synchronously.
    /// No audio stream needed — called directly on the main thread.
    pub fn generate_bars(&mut self, count: usize) {
        let old_lookahead = self.writehead.lookahead;
        self.writehead.lookahead = count;
        self.generate_ahead();
        self.writehead.lookahead = old_lookahead;
    }

    /// Generate measures ahead of the playhead, publishing to shared pages.
    pub fn generate_ahead(&mut self) {
        let playhead_bar = self.playhead_bar.load(Ordering::Relaxed);

        while self.writehead.needs_generation(playhead_bar) {
            let bar_idx = self.writehead.current_bar;

            // If this bar already exists in the timeline (e.g. after seek),
            // ensure it's in shared pages and advance.
            if let Some(existing) = self.writehead.timeline.get_measure(bar_idx) {
                self.publish_measure(existing.clone());
                let mut snapshot = harmonium_core::report::MeasureSnapshot::from_measure(&existing);
                snapshot.composition_bpm = self.emotion_mapped_bpm;
                self.pending_measure_snapshots.push(snapshot);
                self.writehead.current_bar = bar_idx + 1;
                continue;
            }

            let measure = self.generator.generate_measure(bar_idx, &mut self.rng);

            // Update report cache
            self.last_chord_name =
                ArrayString::from(&measure.chord_context.chord_name).unwrap_or_default();
            self.last_chord_root_offset = measure.chord_context.root_offset;
            self.last_chord_is_minor = measure.chord_context.is_minor;

            let mut snapshot = harmonium_core::report::MeasureSnapshot::from_measure(&measure);
            snapshot.composition_bpm = self.emotion_mapped_bpm;
            self.pending_measure_snapshots.push(snapshot);

            self.publish_measure(measure.clone());
            self.writehead.commit_measure(measure);
        }
    }

    /// Write a measure into shared pages by index (insert or replace).
    fn publish_measure(&self, measure: Measure) {
        if let Ok(mut pages) = self.shared_pages.lock() {
            let idx = measure.index;
            if let Some(existing) = pages.iter_mut().find(|m| m.index == idx) {
                *existing = measure;
            } else {
                pages.push(measure);
            }
        }
    }

    /// Drain pending measure snapshots (called by NativeHandle).
    pub fn take_snapshots(&mut self) -> Vec<harmonium_core::report::MeasureSnapshot> {
        std::mem::take(&mut self.pending_measure_snapshots)
    }

    /// Invalidate future measures and regenerate from the next bar.
    /// Called when musical params change.
    pub fn invalidate_future(&mut self) {
        let playhead_bar = self.playhead_bar.load(Ordering::Relaxed);
        let regen_from = if playhead_bar == 0 { 1 } else { playhead_bar + 1 };

        self.writehead.timeline.invalidate_from(regen_from);
        self.writehead.current_bar = regen_from;

        // Clear shared pages beyond invalidation point
        if let Ok(mut pages) = self.shared_pages.lock() {
            pages.retain(|m| m.index < regen_from);
        }

        // Sync generator with updated params and snap current_state
        // so new measures use the correct tempo/density immediately
        self.generator.update_params(self.musical_params.clone());
        self.generator.snap_current_state();
    }

    /// Invalidate measures beyond a preview window, preserving N bars ahead of playhead.
    ///
    /// Preview bars stay in both timeline and shared pages.
    /// The playback engine reads directly by index — no refill needed.
    pub fn invalidate_after_preview(&mut self, preview_bars: usize) {
        let playhead_bar = self.playhead_bar.load(Ordering::Relaxed);
        let keep_until =
            if playhead_bar == 0 { 1 + preview_bars } else { playhead_bar + preview_bars };

        // Invalidate timeline from keep_until onward (preview bars stay intact)
        self.writehead.timeline.invalidate_from(keep_until);
        self.writehead.current_bar = keep_until;

        // Clear shared pages beyond preview window
        if let Ok(mut pages) = self.shared_pages.lock() {
            pages.retain(|m| m.index < keep_until);
        }

        // Update generator with new params
        self.generator.update_params(self.musical_params.clone());
    }

    /// Reset to defaults, clear timeline and shared pages.
    pub fn reset(&mut self) {
        self.musical_params = MusicalParams::default();
        self.writehead.reset();
        self.generator.update_params(self.musical_params.clone());
        if let Ok(mut pages) = self.shared_pages.lock() {
            pages.clear();
        }
    }

    /// Clear timeline, writehead, and shared pages WITHOUT resetting musical_params.
    /// Use this when you want to regenerate from scratch with current settings.
    pub fn reset_timeline(&mut self) {
        self.writehead.reset();
        self.generator.reset_chart_index();
        self.generator.update_params(self.musical_params.clone());
        if let Ok(mut pages) = self.shared_pages.lock() {
            pages.clear();
        }
    }

    /// Seek the writehead to a specific bar (legacy, non-deterministic).
    pub fn seek_writehead(&mut self, bar: usize) {
        self.writehead.current_bar = bar;
    }

    /// Deterministic seek: reset RNG + generator, replay to target bar.
    ///
    /// After this call, the generator and RNG are in the exact state they would
    /// be at `target_bar` if generation had proceeded linearly from bar 1.
    /// Shared pages from target onward are cleared so they get regenerated.
    pub fn deterministic_seek(&mut self, target_bar: usize) {
        use rand::Rng;
        let target_bar = target_bar.max(1);

        // 1. Re-seed RNG from session seed
        let mut rng = ChaCha8Rng::seed_from_u64(self.session_seed);

        // 2. Consume the same init draws as new_with_seed() to advance RNG past init
        let keys_len = 7usize;
        let scales_len = 2usize;
        let _ = rng.gen_range(0..keys_len);
        let _ = rng.gen_range(0..scales_len);

        // 3. Reset generator to initial state
        let ctx = GenerationContext {
            session_seed: self.session_seed,
            key: self.init_key,
            scale: self.init_scale,
            key_pc: self.init_key_pc,
        };
        self.generator.reset_to_initial(&ctx);

        // 4. Silent advance to target bar
        if target_bar > 1 {
            self.generator.silent_advance(target_bar, &mut rng);
        }

        // 5. Update state
        self.rng = rng;
        self.writehead.current_bar = target_bar;

        // 6. Clear shared pages from target onward
        if let Ok(mut pages) = self.shared_pages.lock() {
            pages.retain(|m| m.index < target_bar);
        }

        log::info(&format!("Deterministic seek to bar {target_bar}"));
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
    }

    /// Generate a new melody with a fresh random seed.
    pub fn new_melody(&mut self) {
        use rand::Rng;
        let new_seed: u64 = rand::thread_rng().r#gen();
        log::info(&format!("New melody with seed {new_seed}"));
        self.apply_seed(new_seed);
        self.writehead.reset();
        if let Ok(mut pages) = self.shared_pages.lock() {
            pages.clear();
        }
        self.deterministic_seek(1);
    }

    /// Set an explicit seed and regenerate from bar 1.
    pub fn set_seed(&mut self, seed: u64) {
        log::info(&format!("Set seed to {seed}"));
        self.apply_seed(seed);
        self.writehead.reset();
        if let Ok(mut pages) = self.shared_pages.lock() {
            pages.clear();
        }
        self.deterministic_seek(1);
    }

    /// Get the current session seed.
    pub fn session_seed(&self) -> u64 {
        self.session_seed
    }

    // === Parameter setters (called directly, no command queue) ===

    pub fn set_bpm(&mut self, bpm: f32) {
        let clamped = bpm.clamp(70.0, 180.0);
        self.bpm_override = Some(clamped);
        self.musical_params.bpm = clamped;
    }

    pub fn reset_bpm(&mut self) {
        self.bpm_override = None;
        self.musical_params.bpm = self.emotion_mapped_bpm;
    }

    pub fn set_time_signature(&mut self, numerator: usize, denominator: usize) {
        self.musical_params.time_signature = TimeSignature { numerator, denominator };
    }

    pub fn enable_rhythm(&mut self, e: bool) {
        self.musical_params.enable_rhythm = e;
    }
    pub fn enable_harmony(&mut self, e: bool) {
        self.musical_params.enable_harmony = e;
    }
    pub fn enable_melody(&mut self, e: bool) {
        self.musical_params.enable_melody = e;
    }
    pub fn enable_voicing(&mut self, e: bool) {
        self.musical_params.enable_voicing = e;
    }

    pub fn set_rhythm_mode(&mut self, m: harmonium_core::sequencer::RhythmMode) {
        self.musical_params.rhythm_mode = m;
    }
    pub fn set_rhythm_steps(&mut self, s: usize) {
        self.musical_params.rhythm_steps = s;
    }
    pub fn set_rhythm_pulses(&mut self, p: usize) {
        self.musical_params.rhythm_pulses = p;
    }
    pub fn set_rhythm_rotation(&mut self, r: usize) {
        self.musical_params.rhythm_rotation = r;
    }
    pub fn set_rhythm_density(&mut self, d: f32) {
        self.musical_params.rhythm_density = d.clamp(0.0, 1.0);
    }
    pub fn set_rhythm_tension(&mut self, t: f32) {
        self.musical_params.rhythm_tension = t.clamp(0.0, 1.0);
    }
    /// Override the rhythmic-cell variety knob (CORELIB-13).
    /// Default is 0.5 from `VarietyParams::default()`. Set 0.0 to disable
    /// clave/3+3+2 cells and gap=2 splits — useful for legacy/A-B comparison.
    pub fn set_rhythmic_cell_variety(&mut self, v: f32) {
        self.musical_params.variety.rhythmic_cell_variety = v.clamp(0.0, 1.0);
    }
    pub fn set_rhythm_secondary(&mut self, steps: usize, pulses: usize, rotation: usize) {
        self.musical_params.rhythm_secondary_steps = steps;
        self.musical_params.rhythm_secondary_pulses = pulses;
        self.musical_params.rhythm_secondary_rotation = rotation;
    }
    pub fn set_fixed_kick(&mut self, f: bool) {
        self.musical_params.fixed_kick = f;
    }

    pub fn set_harmony_mode(&mut self, m: harmonium_core::harmony::HarmonyMode) {
        self.musical_params.harmony_mode = m;
    }
    pub fn set_harmony_strategy(&mut self, s: harmonium_core::params::HarmonyStrategy) {
        self.musical_params.harmony_strategy = s;
    }
    pub fn set_chord_chart(&mut self, chart: Vec<arrayvec::ArrayString<16>>) {
        self.musical_params.chord_chart = chart;
    }
    pub fn set_harmony_tension(&mut self, t: f32) {
        self.musical_params.harmony_tension = t.clamp(0.0, 1.0);
    }
    pub fn set_harmony_valence(&mut self, v: f32) {
        self.musical_params.harmony_valence = v.clamp(-1.0, 1.0);
    }
    pub fn set_harmony_measures_per_chord(&mut self, m: usize) {
        self.musical_params.harmony_measures_per_chord = m;
    }
    pub fn set_key_root(&mut self, r: u8) {
        self.musical_params.key_root = r % 12;
    }

    pub fn set_melody_smoothness(&mut self, s: f32) {
        self.musical_params.melody_smoothness = s.clamp(0.0, 1.0);
    }
    pub fn set_melody_octave(&mut self, o: i32) {
        self.musical_params.melody_octave = o.clamp(3, 6);
    }
    pub fn set_voicing_density(&mut self, d: f32) {
        self.musical_params.voicing_density = d.clamp(0.0, 1.0);
    }
    pub fn set_voicing_tension(&mut self, t: f32) {
        self.musical_params.voicing_tension = t.clamp(0.0, 1.0);
    }

    pub fn set_all_rhythm_params(
        &mut self,
        mode: harmonium_core::sequencer::RhythmMode,
        steps: usize,
        pulses: usize,
        rotation: usize,
        density: f32,
        tension: f32,
        secondary_steps: usize,
        secondary_pulses: usize,
        secondary_rotation: usize,
    ) {
        self.musical_params.rhythm_mode = mode;
        self.musical_params.rhythm_steps = steps;
        self.musical_params.rhythm_pulses = pulses;
        self.musical_params.rhythm_rotation = rotation;
        self.musical_params.rhythm_density = density.clamp(0.0, 1.0);
        self.musical_params.rhythm_tension = tension.clamp(0.0, 1.0);
        self.musical_params.rhythm_secondary_steps = secondary_steps;
        self.musical_params.rhythm_secondary_pulses = secondary_pulses;
        self.musical_params.rhythm_secondary_rotation = secondary_rotation;
    }

    // === Instrument config ===

    pub fn set_instrument_lead(&mut self, config: harmonium_core::params::InstrumentConfig) {
        self.musical_params.instrument_lead = config;
    }

    pub fn set_instrument_bass(&mut self, config: harmonium_core::params::InstrumentConfig) {
        self.musical_params.instrument_bass = config;
    }

    // === Emotion mode ===

    pub fn use_emotion_mode(&mut self) {
        self.emotion_mode = true;
        log::info("MusicComposer: switched to Emotion mode");
    }

    pub fn use_direct_mode(&mut self) {
        self.emotion_mode = false;
        log::info("MusicComposer: switched to Direct mode");
    }

    pub fn is_emotion_mode(&self) -> bool {
        self.emotion_mode
    }

    pub fn set_emotions(&mut self, arousal: f32, valence: f32, density: f32, tension: f32) {
        if !self.emotion_mode {
            return;
        }

        self.cached_emotions.arousal = arousal;
        self.cached_emotions.valence = valence;
        self.cached_emotions.density = density;
        self.cached_emotions.tension = tension;

        let mapped = self.emotion_mapper.map(&self.cached_emotions);

        // Preserve runtime state that shouldn't be overwritten by the mapper
        let mut new_params = mapped;
        new_params.rhythm_mode = self.musical_params.rhythm_mode;
        new_params.enable_rhythm = self.musical_params.enable_rhythm;
        new_params.enable_harmony = self.musical_params.enable_harmony;
        new_params.enable_melody = self.musical_params.enable_melody;
        new_params.enable_voicing = self.musical_params.enable_voicing;
        new_params.record_wav = self.musical_params.record_wav;
        new_params.record_midi = self.musical_params.record_midi;
        new_params.record_musicxml = self.musical_params.record_musicxml;
        new_params.muted_channels = self.musical_params.muted_channels.clone();
        new_params.channel_routing = self.musical_params.channel_routing.clone();
        new_params.gain_lead = self.musical_params.gain_lead;
        new_params.gain_bass = self.musical_params.gain_bass;
        new_params.gain_snare = self.musical_params.gain_snare;
        new_params.gain_hat = self.musical_params.gain_hat;
        new_params.instrument_lead = self.musical_params.instrument_lead;
        new_params.instrument_bass = self.musical_params.instrument_bass;

        // Store emotion-mapped BPM, then apply override if set
        self.emotion_mapped_bpm = new_params.bpm;
        if let Some(override_bpm) = self.bpm_override {
            new_params.bpm = override_bpm;
        }

        // Only store params — do NOT sync the generator here.
        // The generator gets synced explicitly by invalidate_after_preview()
        // or invalidate_future() so that already-generated bars are preserved.
        self.musical_params = new_params;

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

    // === Writehead controls ===

    pub fn set_writehead_lookahead(&mut self, n: usize) {
        self.writehead.lookahead = n.max(4);
    }

    /// Apply a TuningParams to the generator (for style profile loading).
    pub fn set_tuning(&mut self, tuning: harmonium_core::tuning::TuningParams) {
        self.generator.update_tuning(tuning);
    }

    /// Reset the RNG to a deterministic seed (for reproducible rendering).
    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }

    /// Sync generator with current musical params (call after batch param changes).
    pub fn sync_generator(&mut self) {
        self.generator.update_params(self.musical_params.clone());
    }

    /// Export timeline to MusicXML.
    pub fn export_timeline(&self, format: harmonium_core::events::RecordFormat) {
        match format {
            harmonium_core::events::RecordFormat::MusicXml => {
                let xml = harmonium_core::timeline::timeline_to_musicxml_with_instruments(
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
                log::warn(&format!("Timeline export only supports MusicXML, got {:?}", format));
            }
        }
    }

    // === Getters for report building ===

    /// Current writehead position (next bar to generate).
    pub fn writehead_position(&self) -> usize {
        self.writehead.current_bar
    }

    /// Current playhead bar (from the shared atomic, written by PlaybackEngine).
    pub fn playhead_bar(&self) -> usize {
        self.playhead_bar.load(Ordering::Relaxed)
    }

    /// Get a reference to a measure from the timeline by bar index.
    pub fn timeline_measure(&self, bar: usize) -> Option<&harmonium_core::timeline::Measure> {
        self.writehead.timeline.get_measure(bar)
    }

    pub fn last_chord_name(&self) -> &ArrayString<64> {
        &self.last_chord_name
    }
    pub fn last_chord_root_offset(&self) -> i32 {
        self.last_chord_root_offset
    }
    pub fn last_chord_is_minor(&self) -> bool {
        self.last_chord_is_minor
    }
    pub fn bpm_override(&self) -> Option<f32> {
        self.bpm_override
    }
    pub fn emotion_mapped_bpm(&self) -> f32 {
        self.emotion_mapped_bpm
    }
    pub fn musical_params(&self) -> &MusicalParams {
        &self.musical_params
    }
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}

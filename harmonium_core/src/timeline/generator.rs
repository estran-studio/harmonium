//! TimelineGenerator - Extracts tick() logic into measure-level generation
//!
//! This module faithfully replicates all state transitions from `engine.rs::tick()`:
//! 1. Tick both sequencers per step
//! 2. Bar boundary: swap patterns, prepare_next_bar, advance harmony
//! 3. Per-step: evaluate triggers, apply drum variations, generate lead/bass
//! 4. Maintain CurrentState morphing between steps
//! 5. Call next_note_structured() in identical order
//!
//! The algorithms are IDENTICAL to the legacy engine - just writing to
//! `Measure` structs instead of emitting ephemeral `AudioEvent`s.

use super::{
    Articulation, ChordContext, GenerationContext, Measure, NoteId, StateSnapshot, TimelineNote,
    TrackId,
};
use crate::{
    harmony::{
        HarmonyMode, RngCore,
        basic::{ChordQuality, ChordStep, Progression},
        chord::ChordType,
        driver::HarmonicDriver,
        melody::HarmonyNavigator,
    },
    params::{CurrentState, MusicalParams},
    sequencer::{RhythmMode, Sequencer, StepTrigger},
    tuning::TuningParams,
};

/// Generates measures offline (on the main thread) using the same algorithms
/// as the legacy engine's tick() function.
pub struct TimelineGenerator {
    // === Sequencers (cloned from engine state) ===
    pub sequencer_primary: Sequencer,
    pub sequencer_secondary: Sequencer,

    // === Harmony ===
    pub harmony: HarmonyNavigator,
    pub harmonic_driver: Option<HarmonicDriver>,
    pub harmony_mode: HarmonyMode,

    // === Progression state (Basic mode) ===
    current_progression: Vec<ChordStep>,
    progression_index: usize,
    last_valence_choice: f32,
    last_tension_choice: f32,

    // === Morphed state ===
    pub current_state: CurrentState,

    // === Musical parameters ===
    pub musical_params: MusicalParams,

    // === Chord tracking ===
    chord_root_offset: i32,
    chord_is_minor: bool,
    chord_name: String,
    current_chord_type: ChordType,

    // === Chart mode ===
    chord_chart: Vec<crate::harmony::Chord>,
    chart_index: usize,

    // === Note ID counter ===
    next_note_id: NoteId,

    // === Bar counter ===
    current_bar: usize,

    // === Phrase arc state (variable length) ===
    phrase_len: usize,
    phrase_bar_counter: usize,

    // === Style tuning ===
    pub tuning: TuningParams,
}

impl TimelineGenerator {
    /// Create a new generator with the given initial state
    #[must_use]
    pub fn new(
        sequencer_primary: Sequencer,
        sequencer_secondary: Sequencer,
        harmony: HarmonyNavigator,
        harmonic_driver: Option<HarmonicDriver>,
        musical_params: MusicalParams,
        current_state: CurrentState,
        tuning: TuningParams,
    ) -> Self {
        let current_progression = Progression::get_palette(
            current_state.valence,
            current_state.tension,
            &tuning.emotional_quadrant,
        );

        let chord_chart = Self::parse_chord_chart(&musical_params.chord_chart);

        Self {
            sequencer_primary,
            sequencer_secondary,
            harmony,
            harmonic_driver,
            harmony_mode: musical_params.harmony_mode,
            current_progression,
            progression_index: 0,
            last_valence_choice: current_state.valence,
            last_tension_choice: current_state.tension,
            current_state,
            musical_params,
            chord_root_offset: 0,
            chord_is_minor: false,
            chord_name: "I".to_string(),
            current_chord_type: ChordType::Major,
            chord_chart,
            chart_index: 0,
            next_note_id: 1,
            current_bar: 0,
            phrase_len: 4,
            phrase_bar_counter: 0,
            tuning,
        }
    }

    /// Update tuning parameters (called when a style profile is loaded/cleared).
    pub fn update_tuning(&mut self, tuning: TuningParams) {
        self.sequencer_primary.pb_params = tuning.perfect_balance.clone();
        self.sequencer_primary.cg_params = tuning.classic_groove.clone();
        self.sequencer_secondary.pb_params = tuning.perfect_balance.clone();
        self.sequencer_secondary.cg_params = tuning.classic_groove.clone();
        self.harmony.set_melody_params(&tuning.melody);
        self.tuning = tuning;
    }

    /// Reset the generator to its initial state for deterministic seek/replay.
    ///
    /// Creates a fresh generator with the same constructor logic as `new_with_seed()`,
    /// preserving only `next_note_id` (monotonically increasing) and `musical_params`.
    /// Then applies `update_params()` to match the state that `update_controls()` would
    /// produce before the first `generate_measure()` call.
    pub fn reset_to_initial(&mut self, ctx: &GenerationContext) {
        let bpm = 120.0f32; // Hardcoded to match new_with_seed()
        let steps = 16usize;
        let initial_pulses = 4usize;

        // Rebuild sequencers (exact same code as new_with_seed)
        let sequencer_primary = Sequencer::new(steps, initial_pulses, bpm);
        let default_density = 0.4f32;
        let secondary_pulses = std::cmp::min((default_density * 8.0) as usize + 1, 12);
        let sequencer_secondary = Sequencer::new_with_rotation(12, secondary_pulses, bpm, 0);

        let harmony = HarmonyNavigator::new(ctx.key, ctx.scale, 4);
        let harmonic_driver = Some(HarmonicDriver::new(ctx.key_pc, &self.tuning.harmony_driver));
        let musical_params = self.musical_params.clone();

        let initial_state = CurrentState {
            bpm,
            density: musical_params.rhythm_density,
            tension: musical_params.rhythm_tension,
            smoothness: musical_params.melody_smoothness,
            ..CurrentState::default()
        };

        let tuning = self.tuning.clone();
        let current_progression = Progression::get_palette(
            initial_state.valence,
            initial_state.tension,
            &tuning.emotional_quadrant,
        );
        let chord_chart = self.chord_chart.clone();

        // Preserve next_note_id
        let saved_note_id = self.next_note_id;

        // Build fresh generator state (matching TimelineGenerator::new)
        self.sequencer_primary = sequencer_primary;
        self.sequencer_secondary = sequencer_secondary;
        self.harmony = harmony;
        self.harmonic_driver = harmonic_driver;
        self.harmony_mode = musical_params.harmony_mode;
        self.current_progression = current_progression;
        self.progression_index = 0;
        self.last_valence_choice = initial_state.valence;
        self.last_tension_choice = initial_state.tension;
        self.current_state = initial_state;
        self.musical_params = musical_params;
        self.chord_root_offset = 0;
        self.chord_is_minor = false;
        self.chord_name = "I".to_string();
        self.current_chord_type = ChordType::Major;
        self.chord_chart = chord_chart;
        self.chart_index = 0;
        self.next_note_id = saved_note_id;
        self.current_bar = 0;
        self.phrase_len = 4;
        self.phrase_bar_counter = 0;
        self.tuning = tuning;
    }

    /// Fast-forward the generator to `target_bar` by replaying bars 1..target_bar-1.
    ///
    /// Calls `generate_measure()` for each bar, discarding the output. After this,
    /// the generator and RNG are in the exact state they would be at bar `target_bar`
    /// if generation had proceeded linearly from bar 1.
    ///
    /// `next_note_id` will advance during replay (creating "phantom" IDs), which is
    /// acceptable since note IDs only need to be monotonically increasing, not contiguous.
    pub fn silent_advance(&mut self, target_bar: usize, rng: &mut dyn RngCore) {
        for bar in 1..target_bar {
            let _ = self.generate_measure(bar, rng);
        }
    }

    /// Calculate velocity with beat-level accents and phrase-level dynamics (CORELIB-6)
    ///
    /// - `base_vel`: base velocity from arousal/params
    /// - `step`: step index within the bar
    /// - `steps_per_bar`: total steps in the bar
    /// - `bar_index`: current bar number (for phrase dynamics)
    /// - `phrase_len`: bars per phrase (typically 4 or 8)
    fn shape_velocity(
        &self,
        base_vel: u8,
        step: usize,
        steps_per_bar: usize,
        bar_index: usize,
        phrase_len: usize,
        rng: &mut dyn RngCore,
    ) -> u8 {
        let tpb = self.sequencer_primary.ticks_per_beat;

        // === Beat-level accents ===
        let beat_offset: i16 = if tpb > 0 && step % tpb == 0 {
            let beat_in_bar = step / tpb;
            if beat_in_bar % 2 == 0 {
                18 // Strong beats (1, 3): +18
            } else {
                -8 // Weak beats (2, 4): -8
            }
        } else {
            -20 // Offbeats: -20
        };

        // === Phrase-level dynamics ===
        let bar_in_phrase = bar_index % phrase_len;
        let phrase_progress = bar_in_phrase as f32 / phrase_len as f32;
        let phrase_offset = if phrase_progress < 0.5 {
            (phrase_progress * 2.0 * 20.0) as i16
        } else {
            ((1.0 - phrase_progress) * 2.0 * 20.0) as i16
        };

        // === Step-within-bar dynamics ===
        let bar_progress = step as f32 / steps_per_bar.max(1) as f32;
        let bar_offset = (bar_progress * 10.0) as i16;

        // === Velocity jitter (variety.velocity_jitter) ===
        let jitter_range = self.musical_params.variety.velocity_jitter;
        let jitter = ((rng.next_f32() - 0.5) * 2.0 * jitter_range) as i16;

        let final_vel = base_vel as i16 + beat_offset + phrase_offset + bar_offset + jitter;
        final_vel.clamp(20, 127) as u8
    }

    /// Generate a single measure, faithfully replicating tick() behavior.
    ///
    /// This ticks both sequencers through all steps in the measure, calling
    /// the same melody/harmony/rhythm functions in identical order as tick().
    /// Phrase arc with variable length and jitter (CORELIB-4 + anti-repetition).
    /// Uses `variety.phrase_arc_jitter` for amplitude randomness.
    fn phrase_arc_modifier(&mut self, rng: &mut dyn RngCore) -> f32 {
        let jitter_range = self.musical_params.variety.phrase_arc_jitter;

        // Re-roll phrase length at phrase boundary (3, 4, or 5 bars)
        if self.phrase_bar_counter >= self.phrase_len {
            self.phrase_bar_counter = 0;
            let r = rng.next_f32();
            self.phrase_len = if r < 0.25 {
                3
            } else if r < 0.75 {
                4
            } else {
                5
            };
        }

        let phase = self.phrase_bar_counter as f32 / self.phrase_len as f32;
        let arc = (phase * std::f32::consts::PI).sin();

        // Amplitude jitter: ±jitter_range around 1.0
        let jitter = 1.0 + (rng.next_f32() - 0.5) * 2.0 * jitter_range;

        self.phrase_bar_counter += 1;
        (arc - 0.5) * 0.40 * jitter
    }

    /// Section-level dynamic contrast with jitter (CORELIB-7 + anti-repetition).
    /// Uses `variety.section_arc_jitter` for per-section randomness.
    fn section_arc_modifier(&self, bar_index: usize, rng: &mut dyn RngCore) -> f32 {
        let jitter_range = self.musical_params.variety.section_arc_jitter;
        const SECTION_LEN: usize = 8;
        let base = match (bar_index / SECTION_LEN) % 4 {
            0 => 0.0,
            1 => -0.25,
            2 => 0.25,
            3 => -0.10,
            _ => 0.0,
        };
        let jitter = (rng.next_f32() - 0.5) * 2.0 * jitter_range;
        base + jitter
    }

    pub fn generate_measure(&mut self, bar_index: usize, rng: &mut dyn RngCore) -> Measure {
        self.current_bar = bar_index;

        // Snap current_state to match musical_params first —
        // params affect the writehead directly, next bar uses new values.
        self.snap_current_state();

        // === SECTION ARC (CORELIB-7) — with jitter ===
        let section = self.section_arc_modifier(bar_index, rng);
        self.current_state.arousal = (self.current_state.arousal + section).clamp(0.0, 1.0);
        self.current_state.density = (self.current_state.density + section * 0.5).clamp(0.05, 1.0);

        // === PHRASE ARC (CORELIB-4) — with variable length + jitter ===
        let arc = self.phrase_arc_modifier(rng);
        self.current_state.density = (self.current_state.density + arc).clamp(0.05, 1.0);
        self.current_state.tension = (self.current_state.tension + arc * 0.5).clamp(0.0, 1.0);
        self.current_state.arousal = (self.current_state.arousal + arc * 0.3).clamp(0.0, 1.0);

        let time_sig = self.musical_params.time_signature;
        let steps = time_sig.steps_per_bar(self.sequencer_primary.ticks_per_beat);

        let mut measure = Measure::new(bar_index, time_sig, self.current_state.bpm, steps);

        // === BARLINE LOGIC (replicates tick() bar_crossed branch) ===
        self.handle_barline(bar_index, rng);

        // Snapshot state at generation time
        measure.state_snapshot = StateSnapshot::from(&self.current_state);
        measure.chord_context = ChordContext {
            root_offset: self.chord_root_offset,
            is_minor: self.chord_is_minor,
            chord_name: self.chord_name.clone(),
        };

        // === PER-STEP LOGIC (replicates tick() event generation) ===
        let rhythm_enabled = self.musical_params.enable_rhythm;
        let melody_enabled = self.musical_params.enable_melody;

        // Context flags (replicates the "virtual drummer" context from tick())
        let arr = self.tuning.arrangement.clone();
        let is_high_tension = self.current_state.tension > arr.energy_high_tension;
        let is_high_density = self.current_state.density > arr.energy_high_density;
        let is_high_energy = self.current_state.arousal > arr.energy_high_arousal;
        let is_low_energy = self.current_state.arousal < 0.4;
        let fill_zone_start = steps.saturating_sub(arr.fill_zone_size);

        // === WALKING BASS (CORELIB-11) ===
        // When density > 0.4 and smoothness > 0.4, generate a walking bass line
        // that plays quarter notes on every beat: root → arpeggio → passing → approach
        let tpb = self.sequencer_primary.ticks_per_beat;
        let walking_bass = rhythm_enabled
            && !self.musical_params.fixed_kick
            && !self.musical_params.muted_channels.first().copied().unwrap_or(false)
            && self.current_state.density > 0.4
            && self.current_state.smoothness > 0.4
            && tpb >= 2;

        if walking_bass {
            let root = 36i32 + self.chord_root_offset;
            let is_minor = self.chord_is_minor;
            let third_interval = if is_minor { 3 } else { 4 };
            let num_beats = time_sig.numerator;
            let base_vel =
                self.musical_params.vel_base_bass + (self.current_state.arousal * 25.0) as u8;

            let variety = self.musical_params.variety.walking_bass_variety;

            for beat in 0..num_beats {
                let beat_step = beat * tpb;
                if beat_step >= steps {
                    break;
                }

                // Rest insertion on non-downbeats (scaled by variety)
                if beat > 0 && rng.next_f32() < 0.10 * variety {
                    continue;
                }

                let r = rng.next_f32();
                let pitch = match beat % 4 {
                    0 => {
                        // Beat 1: root, occasionally 5th below or octave up
                        if r < (1.0 - 0.25 * variety) {
                            root
                        } else if r < (1.0 - 0.10 * variety) {
                            root - 5
                        } else {
                            root + 12
                        }
                    }
                    1 => {
                        // Beat 2: expanded palette scaled by variety
                        if r < 0.30 {
                            root + third_interval
                        } else if r < 0.60 {
                            root + 7
                        } else if r < (0.60 + 0.20 * variety) {
                            root + 9
                        } else {
                            root + 5
                        }
                    }
                    2 => {
                        // Beat 3: passing tones — wider palette
                        if r < 0.25 {
                            root + 5
                        } else if r < 0.50 {
                            root + 2
                        } else if r < 0.70 {
                            root + 7
                        } else if r < (0.70 + 0.15 * variety) {
                            root + 10
                        }
                        // b7 (bluesy)
                        else {
                            root + third_interval
                        }
                    }
                    3 => {
                        // Beat 4: approach notes
                        if r < 0.30 {
                            root + 11
                        } else if r < 0.55 {
                            root - 1
                        } else if r < (0.55 + 0.20 * variety) {
                            root - 2
                        } else if r < (0.75 + 0.15 * variety) {
                            root + 10
                        } else {
                            root + 5
                        }
                    }
                    _ => root,
                };

                let midi_note =
                    self.musical_params.instrument_bass.apply(pitch.clamp(28, 60) as u8);
                let vel = self.shape_velocity(base_vel, beat_step, steps, bar_index, 4, rng);
                // Variable duration: occasionally short (8th note feel)
                let duration = if rng.next_f32() < 0.15 * variety {
                    (tpb / 2).max(1).min(steps - beat_step)
                } else {
                    tpb.min(steps - beat_step)
                };

                measure.add_note(
                    TrackId::Bass,
                    TimelineNote {
                        id: self.next_id(),
                        pitch: midi_note,
                        start_step: beat_step,
                        duration_steps: duration,
                        velocity: vel,
                        articulation: Articulation::Normal,
                    },
                );
            }
        }

        // Watermark: when a multi-gap rhythmic cell is emitted (e.g. clave 3+3+2
        // spanning two quarter gaps), suppress lead triggers until this step.
        let mut skip_lead_until: usize = 0;

        for step in 0..steps {
            // Tick primary sequencer
            let trigger_primary = if step < self.sequencer_primary.pattern.len() {
                self.sequencer_primary.pattern[step]
            } else {
                StepTrigger::default()
            };

            // Tick secondary sequencer (Euclidean mode only for polyrhythm)
            let trigger_secondary = if self.sequencer_primary.mode == RhythmMode::Euclidean
                && step < self.sequencer_secondary.pattern.len()
            {
                self.sequencer_secondary.pattern[step % self.sequencer_secondary.pattern.len()]
            } else {
                StepTrigger::default()
            };

            let is_in_fill_zone = step >= fill_zone_start;

            // === CORELIB-5: Mid-bar chord change at very high tension ===
            // At tension > 0.75, change chord halfway through the bar
            if step == steps / 2
                && self.current_state.tension > 0.75
                && self.musical_params.enable_harmony
                && self.harmony_mode == HarmonyMode::Driver
            {
                if let Some(ref mut driver) = self.harmonic_driver {
                    let decision = driver.next_chord(
                        self.current_state.tension,
                        self.current_state.valence,
                        rng,
                    );
                    let root_offset = driver.root_offset();
                    let quality = driver.to_basic_quality();
                    self.harmony.set_chord_context(root_offset, quality);
                    self.chord_root_offset = root_offset;
                    self.chord_is_minor = driver.is_minor();
                    self.chord_name = decision.next_chord.name();
                    self.current_chord_type = decision.next_chord.chord_type;
                }
            }

            // === BASS (CORELIB-3: decoupled from kick, musically varied) ===
            // Skip kick-triggered bass when walking bass already filled the bar
            if rhythm_enabled
                && !walking_bass
                && trigger_primary.kick
                && !self.musical_params.muted_channels.first().copied().unwrap_or(false)
            {
                let density = self.current_state.density;
                let smoothness = self.current_state.smoothness;

                // === REST INSERTION for bass (sparse at low density) ===
                let bass_rest_prob = (1.0 - density) * 0.15; // 0-15%
                if self.musical_params.fixed_kick || rng.next_f32() >= bass_rest_prob {
                    let root = 36i32 + self.chord_root_offset;

                    // === PITCH VARIETY: root, fifth, third, approach ===
                    let midi_note = if self.musical_params.fixed_kick {
                        36u8
                    } else {
                        let tpb = self.sequencer_primary.ticks_per_beat;
                        let beat_in_bar = if tpb > 0 { step / tpb } else { 0 };
                        let is_last_beat = step + tpb >= steps;
                        let is_minor = self.chord_is_minor;

                        if is_last_beat && rng.next_f32() < 0.3 {
                            // Approach note: chromatic approach to root (±1 semitone)
                            let approach = if rng.next_f32() < 0.5 { root - 1 } else { root + 1 };
                            approach.clamp(28, 60) as u8
                        } else if beat_in_bar % 2 == 1 && rng.next_f32() < 0.4 {
                            // Weak beats: fifth (root + 7 semitones)
                            (root + 7).clamp(28, 60) as u8
                        } else if beat_in_bar >= 2 && rng.next_f32() < 0.2 {
                            // Later beats: third (major +4, minor +3)
                            let third = if is_minor { 3 } else { 4 };
                            (root + third).clamp(28, 60) as u8
                        } else {
                            // Default: root
                            root.clamp(28, 60) as u8
                        }
                    };
                    let midi_note = self.musical_params.instrument_bass.apply(midi_note);

                    // === VARIABLE DURATION (not always staccato) ===
                    let duration = if smoothness > 0.5 && density < 0.5 {
                        // Smooth + sparse: longer bass notes (quarter note = 4 steps)
                        4.min(steps - step)
                    } else if density > 0.7 {
                        1 // High density: staccato punch
                    } else {
                        2.min(steps - step) // Default: eighth note
                    };

                    let articulation = if smoothness > 0.6 {
                        Articulation::Normal // Legato-ish
                    } else {
                        Articulation::Staccato
                    };

                    let base_vel = self.musical_params.vel_base_bass
                        + (self.current_state.arousal * 25.0) as u8;
                    let vel = self.shape_velocity(base_vel, step, steps, bar_index, 4, rng);

                    measure.add_note(
                        TrackId::Bass,
                        TimelineNote {
                            id: self.next_id(),
                            pitch: midi_note,
                            start_step: step,
                            duration_steps: duration,
                            velocity: vel,
                            articulation,
                        },
                    );
                }
            }

            // === LEAD (with voicing decision) ===
            let play_lead = melody_enabled
                && trigger_primary.lead
                && step >= skip_lead_until
                && !(is_high_tension && is_in_fill_zone)
                && !self.musical_params.muted_channels.get(1).copied().unwrap_or(false);

            if play_lead {
                let is_strong = trigger_primary.kick || trigger_primary.snare;
                let is_new_measure = step == 0;
                let density = self.current_state.density;

                // === CORELIB-2: REST INSERTION ===
                // Probability-based rests for natural phrasing.
                // Higher rest chance at low density, on weak beats, at phrase boundaries.
                let rest_prob = if is_strong {
                    density * 0.05 // 0-5% on strong beats
                } else if step == 0 && bar_index % 4 == 3 {
                    0.35 // 35% at phrase boundaries (last bar of 4-bar phrase)
                } else {
                    (1.0 - density) * 0.20 // 0-20% scaled inversely with density
                };

                if rng.next_f32() < rest_prob {
                    // Rest: advance melody state but don't emit a note
                    self.harmony.next_note_structured(is_strong, is_new_measure, rng);
                } else {
                    let freq = self.harmony.next_note_structured(is_strong, is_new_measure, rng);
                    let melody_midi = (69.0 + 12.0 * (freq / 440.0).log2()).round() as u8;
                    let melody_midi = self.musical_params.instrument_lead.apply(melody_midi);
                    let base_vel = (70.0 + self.current_state.arousal * 30.0) as u8;

                    // Determine available gap to next trigger
                    let raw_gap = self.calculate_lead_duration(
                        step,
                        steps,
                        &self.sequencer_primary.pattern,
                        is_high_tension,
                        fill_zone_start,
                    );

                    // === CELL VARIETY (CORELIB-13 wiring) ===
                    let variety = self.musical_params.variety.rhythmic_cell_variety;

                    // === LOOKAHEAD COMBINE ===
                    // Adjacent quarter-gaps can merge into a half-bar cell so we
                    // can emit clave-style patterns like [3,3,2] that don't fit
                    // in a single quarter. Probability scales with variety.
                    let combine_prob = (variety * 0.6).min(0.8);
                    let pattern_len = self.sequencer_primary.pattern.len();
                    let next_lead_step = step + raw_gap;
                    let combined = raw_gap == 4
                        && next_lead_step < steps
                        && next_lead_step < pattern_len
                        && self.sequencer_primary.pattern[next_lead_step].lead
                        && rng.next_f32() < combine_prob;
                    let gap = if combined { 8 } else { raw_gap };

                    // === CORELIB-1: RHYTHMIC CELL SUBDIVISION ===
                    // At higher density/tension, subdivide long notes into cells.
                    // Low density → sustain (single long note)
                    // Mid density → quarter+eighth or dotted patterns
                    // High density → eighth note runs, 16th pickups
                    let subdivide_prob = if gap >= 4 {
                        (density * 0.5 + self.current_state.tension * 0.25 + variety * 0.35)
                            .min(0.9)
                    } else if gap == 2 && variety > 0.0 {
                        // Occasionally split an eighth into two 16ths for flourish.
                        // Fully gated on variety so disabling it restores legacy behavior.
                        (variety * (0.3 + density * 0.2)).min(0.4)
                    } else {
                        0.0
                    };

                    if gap >= 2 && rng.next_f32() < subdivide_prob {
                        // Generate a rhythmic cell that fills the gap
                        let cell = Self::pick_rhythmic_cell(gap, density, variety, rng);
                        let mut cell_offset = 0;
                        for (i, &dur) in cell.iter().enumerate() {
                            if cell_offset >= gap {
                                break;
                            }
                            let actual_dur = dur.min(gap - cell_offset);

                            // First note of cell reuses the already-generated pitch;
                            // subsequent notes get new pitches
                            let (pitch, vel_step) = if i == 0 {
                                (melody_midi, step + cell_offset)
                            } else {
                                let f = self.harmony.next_note_structured(false, false, rng);
                                let m = (69.0 + 12.0 * (f / 440.0).log2()).round() as u8;
                                let m = self.musical_params.instrument_lead.apply(m);
                                (m, step + cell_offset)
                            };

                            let vel =
                                self.shape_velocity(base_vel, vel_step, steps, bar_index, 4, rng);
                            measure.add_note(
                                TrackId::Lead,
                                TimelineNote {
                                    id: self.next_id(),
                                    pitch,
                                    start_step: step + cell_offset,
                                    duration_steps: actual_dur,
                                    velocity: vel,
                                    articulation: Articulation::Normal,
                                },
                            );
                            cell_offset += dur;
                        }
                    } else {
                        // Single sustained note (original behavior)
                        let solo_vel =
                            self.shape_velocity(base_vel, step, steps, bar_index, 4, rng);
                        measure.add_note(
                            TrackId::Lead,
                            TimelineNote {
                                id: self.next_id(),
                                pitch: melody_midi,
                                start_step: step,
                                duration_steps: gap,
                                velocity: solo_vel,
                                articulation: Articulation::Normal,
                            },
                        );
                    }

                    // Suppress the swallowed trigger when we merged two gaps.
                    if combined {
                        skip_lead_until = step + gap;
                    }
                }
            }

            // === SNARE (with ghost notes and tom fills) ===
            if rhythm_enabled
                && trigger_primary.snare
                && !self.musical_params.muted_channels.get(2).copied().unwrap_or(false)
            {
                let mut snare_note = 38u8;
                let mut vel =
                    self.musical_params.vel_base_snare + (self.current_state.arousal * 30.0) as u8;

                // Ghost notes
                if trigger_primary.velocity < 0.7 {
                    vel = (vel as f32 * arr.ghost_velocity_factor) as u8;
                    if is_low_energy {
                        snare_note = 37; // Side Stick
                    }
                }

                // Tom fills
                if is_high_tension && is_in_fill_zone {
                    snare_note = match step % 3 {
                        0 => 41, // Low Tom
                        1 => 45, // Mid Tom
                        _ => 50, // High Tom
                    };
                    vel = (vel as f32 * arr.tom_velocity_boost).min(127.0) as u8;
                }

                measure.add_note(
                    TrackId::Snare,
                    TimelineNote {
                        id: self.next_id(),
                        pitch: snare_note,
                        start_step: step,
                        duration_steps: 0, // Trigger only
                        velocity: vel,
                        articulation: Articulation::Trigger,
                    },
                );
            }

            // === HAT (with cymbal variations) ===
            let play_hat = trigger_primary.hat || trigger_secondary.hat;
            if rhythm_enabled
                && play_hat
                && !self.musical_params.muted_channels.get(3).copied().unwrap_or(false)
            {
                let mut hat_note = 42u8; // Closed Hi-Hat
                let mut vel = 70 + (self.current_state.arousal * 30.0) as u8;

                // Crash on the "One"
                if step == 0 && is_high_energy {
                    hat_note = 49;
                    vel = arr.crash_velocity;
                }
                // Ride / Open Hat variation
                else if is_high_density {
                    if self.current_state.tension > 0.7 {
                        hat_note = 51; // Ride Cymbal
                    } else if !step.is_multiple_of(2) {
                        hat_note = 46; // Open Hi-Hat
                    }
                }
                // Pedal Hat (calm)
                else if is_low_energy {
                    hat_note = 44; // Pedal Hi-Hat
                }

                measure.add_note(
                    TrackId::Hat,
                    TimelineNote {
                        id: self.next_id(),
                        pitch: hat_note,
                        start_step: step,
                        duration_steps: 0,
                        velocity: vel,
                        articulation: Articulation::Trigger,
                    },
                );
            }
        }

        // Advance sequencer positions to match (they were read but not ticked)
        self.sequencer_primary.current_step = 0;
        if self.sequencer_primary.mode == RhythmMode::Euclidean {
            self.sequencer_secondary.current_step = 0;
        }

        measure
    }

    /// Handle barline logic: swap patterns, advance harmony, prepare next bar
    fn handle_barline(&mut self, bar_index: usize, rng: &mut dyn RngCore) {
        // Swap pattern buffers (replicates tick() bar_crossed logic)
        if let Some(next) = self.sequencer_primary.next_pattern.take() {
            self.sequencer_primary.pattern = next;
            self.sequencer_primary.steps = self.sequencer_primary.pattern.len();
        }
        if let Some(next) = self.sequencer_secondary.next_pattern.take() {
            self.sequencer_secondary.pattern = next;
            self.sequencer_secondary.steps = self.sequencer_secondary.pattern.len();
        }

        // Prepare next bar patterns
        self.sequencer_primary.prepare_next_bar();
        self.sequencer_secondary.prepare_next_bar();

        // === HARMONY & PROGRESSION ===
        if !self.musical_params.enable_harmony {
            return;
        }

        match self.harmony_mode {
            HarmonyMode::Basic => {
                self.advance_basic_harmony(bar_index);
            }
            HarmonyMode::Driver => {
                self.advance_driver_harmony(bar_index, rng);
            }
            HarmonyMode::Chart => {
                self.advance_chart_harmony(bar_index);
            }
        }
    }

    /// Compute tension-driven measures-per-chord (CORELIB-5)
    fn harmonic_rhythm_rate(&self, tension: f32) -> usize {
        let arr = &self.tuning.arrangement;
        if tension < arr.progression_tension_thresholds[0] {
            arr.progression_switch_interval_slow
        } else if tension < arr.progression_tension_thresholds[1] {
            arr.progression_switch_interval_normal
        } else {
            arr.progression_switch_interval_fast
        }
    }

    /// Advance harmony in Basic mode (quadrant-based progressions)
    fn advance_basic_harmony(&mut self, bar_index: usize) {
        // Palette selection with hysteresis (every 4 bars)
        if bar_index.is_multiple_of(4) {
            let valence_delta = (self.current_state.valence - self.last_valence_choice).abs();
            let tension_delta = (self.current_state.tension - self.last_tension_choice).abs();

            if valence_delta > 0.4 || tension_delta > 0.4 {
                self.current_progression = Progression::get_palette(
                    self.current_state.valence,
                    self.current_state.tension,
                    &self.tuning.emotional_quadrant,
                );
                self.progression_index = 0;
                self.last_valence_choice = self.current_state.valence;
                self.last_tension_choice = self.current_state.tension;
            }
        }

        // CORELIB-5: Dynamic harmonic rhythm based on tension
        let measures_per_chord = self.harmonic_rhythm_rate(self.current_state.tension);
        if bar_index.is_multiple_of(measures_per_chord) {
            self.progression_index = (self.progression_index + 1) % self.current_progression.len();
            let chord = &self.current_progression[self.progression_index];

            self.harmony.set_chord_context(chord.root_offset, chord.quality);
            self.chord_root_offset = chord.root_offset;
            self.chord_is_minor = matches!(chord.quality, ChordQuality::Minor);
            self.chord_name = format_chord_name(chord.root_offset, chord.quality);
            self.current_chord_type = match chord.quality {
                ChordQuality::Major => ChordType::Major7,
                ChordQuality::Minor => ChordType::Minor7,
                ChordQuality::Dominant7 => ChordType::Dominant7,
                ChordQuality::Diminished => ChordType::Diminished7,
                ChordQuality::Sus2 => ChordType::Sus2,
            };
        }
    }

    /// Advance harmony in Driver mode (Steedman + Neo-Riemannian + Parsimonious)
    fn advance_driver_harmony(&mut self, bar_index: usize, rng: &mut dyn RngCore) {
        // CORELIB-5: Dynamic harmonic rhythm
        let measures_per_chord = self.harmonic_rhythm_rate(self.current_state.tension);
        if bar_index.is_multiple_of(measures_per_chord) {
            if let Some(ref mut driver) = self.harmonic_driver {
                let decision =
                    driver.next_chord(self.current_state.tension, self.current_state.valence, rng);

                let root_offset = driver.root_offset();
                let quality = driver.to_basic_quality();
                self.harmony.set_chord_context(root_offset, quality);

                self.chord_root_offset = root_offset;
                self.chord_is_minor = driver.is_minor();
                self.chord_name = decision.next_chord.name();
                self.current_chord_type = decision.next_chord.chord_type;
            }
        }
    }

    /// Advance harmony in Chart mode — step through a fixed chord sequence, looping at end
    fn advance_chart_harmony(&mut self, bar_index: usize) {
        if self.chord_chart.is_empty() {
            return;
        }

        let measures_per_chord = self.musical_params.harmony_measures_per_chord;
        if bar_index.is_multiple_of(measures_per_chord) {
            let chord = &self.chord_chart[self.chart_index];
            let root_offset =
                ((i32::from(chord.root)) - i32::from(self.musical_params.key_root) + 12) % 12;
            let quality = chord.to_basic_quality();

            self.harmony.set_chord_context(root_offset, quality);
            self.chord_root_offset = root_offset;
            self.chord_is_minor = chord.chord_type.is_minor();
            self.chord_name = chord.name();
            self.current_chord_type = chord.chord_type;

            // Advance to next chord, wrapping at chart end
            self.chart_index = (self.chart_index + 1) % self.chord_chart.len();
        }
    }

    // apply_morphing removed — writehead uses params directly via snap_current_state

    /// Standard note durations in steps (for ticks_per_beat=4).
    /// These map 1:1 to VexFlow glyphs: 16=w, 12=hd, 8=h, 6=qd, 4=q, 3=8d, 2=8, 1=16
    const NOTATION_SAFE_STEPS: [usize; 8] = [16, 12, 8, 6, 4, 3, 2, 1];

    /// Pick one cell from a weighted table. Weights are relative; they are
    /// summed and the random sample is rescaled, so callers don't need to make
    /// them sum to 1.0. Always returns a cell — the last entry is the fallback.
    fn pick_weighted_cell(rng: &mut dyn RngCore, choices: &[(&[usize], f32)]) -> Vec<usize> {
        debug_assert!(!choices.is_empty(), "pick_weighted_cell needs at least one option");
        let total: f32 = choices.iter().map(|(_, w)| *w).sum();
        let mut r = rng.next_f32() * total;
        for (cell, w) in choices {
            if r < *w {
                return cell.to_vec();
            }
            r -= w;
        }
        choices.last().unwrap().0.to_vec()
    }

    /// Pick a rhythmic cell (sequence of durations in steps) to fill a gap (CORELIB-1).
    ///
    /// Returns a `Vec<usize>` of notation-safe durations that sum to approximately `gap`.
    /// Higher density → more subdivision, shorter notes. Higher variety → more
    /// asymmetric/clave-style cells over uniform ones.
    ///
    /// Each `gap` arm follows the same shape: roll variety-gated "special"
    /// branches (dotted, clave) first; if none fire, pick from the uniform
    /// weighted table. Branches read top-to-bottom in priority order.
    fn pick_rhythmic_cell(
        gap: usize,
        density: f32,
        variety: f32,
        rng: &mut dyn RngCore,
    ) -> Vec<usize> {
        // Fuzzy density: add randomness so same density doesn't always pick same branch
        let d = (density + (rng.next_f32() - 0.5) * 0.2).clamp(0.0, 1.0);

        match gap {
            2 => {
                // Tiny flourish: split eighth into two 16ths
                vec![1, 1]
            }
            4 => Self::pick_cell_gap4(d, variety, rng),
            6 => Self::pick_cell_gap6(rng),
            8 => Self::pick_cell_gap8(d, variety, rng),
            12..=usize::MAX => {
                let r = rng.next_f32();
                if d < 0.3 {
                    if r < 0.5 { vec![8, gap - 8] } else { vec![6, 2, gap - 8] }
                } else if r < 0.3 {
                    vec![4, 4, gap - 8]
                } else if r < 0.6 {
                    vec![2, 2, 4, gap - 8]
                } else {
                    let mut cell = Vec::new();
                    let mut remaining = gap;
                    while remaining > 0 {
                        let dur = if remaining >= 4 { 4 } else { remaining };
                        cell.push(dur);
                        remaining -= dur;
                    }
                    cell
                }
            }
            _ => vec![gap],
        }
    }

    /// Quarter-gap (4 steps) cell selection.
    ///
    /// Two-tier:
    /// 1. Variety-gated dotted/syncopated branch (`[3,1]`, `[1,3]`, `[4]`)
    /// 2. Density-aware uniform table (sustain at low density, subdivision otherwise)
    fn pick_cell_gap4(d: f32, variety: f32, rng: &mut dyn RngCore) -> Vec<usize> {
        const DOTTED: &[(&[usize], f32)] = &[
            (&[3, 1], 0.4),
            (&[1, 3], 0.4),
            (&[4],    0.2),
        ];
        const SUBDIV: &[(&[usize], f32)] = &[
            (&[2, 2],       0.30),
            (&[1, 1, 2],    0.20),
            (&[2, 1, 1],    0.20),
            (&[1, 1, 1, 1], 0.15),
        ];

        let dotted_chance = (0.20 + variety * 0.35).min(0.55);
        if rng.next_f32() < dotted_chance {
            return Self::pick_weighted_cell(rng, DOTTED);
        }
        if d < 0.4 {
            return vec![4]; // sustain at low density
        }
        Self::pick_weighted_cell(rng, SUBDIV)
    }

    /// Dotted-quarter gap (6 steps) cell selection.
    /// Used for compound meters (3/4, 6/8). No variety gating yet.
    fn pick_cell_gap6(rng: &mut dyn RngCore) -> Vec<usize> {
        const TABLE: &[(&[usize], f32)] = &[
            (&[4, 2],    0.25),
            (&[2, 4],    0.20),
            (&[2, 2, 2], 0.20),
            (&[3, 1, 2], 0.15),
            (&[1, 1, 4], 0.20),
        ];
        Self::pick_weighted_cell(rng, TABLE)
    }

    /// Half-bar gap (8 steps) cell selection.
    ///
    /// Three-tier:
    /// 1. Variety-gated clave/3+3+2 family (`[3,3,2]`, `[2,3,3]`, `[3,2,3]`)
    /// 2. Low-density sparse pair (`[6,2]`, `[2,6]`)
    /// 3. Density-aware uniform table
    fn pick_cell_gap8(d: f32, variety: f32, rng: &mut dyn RngCore) -> Vec<usize> {
        const CLAVE: &[(&[usize], f32)] = &[
            (&[3, 3, 2], 0.5),
            (&[2, 3, 3], 0.3),
            (&[3, 2, 3], 0.2),
        ];
        const SPARSE: &[(&[usize], f32)] = &[
            (&[6, 2], 0.5),
            (&[2, 6], 0.5),
        ];
        const DENSE: &[(&[usize], f32)] = &[
            (&[4, 4],       0.15),
            (&[4, 2, 2],    0.15),
            (&[2, 2, 4],    0.15),
            (&[2, 2, 2, 2], 0.20),
            (&[2, 4, 2],    0.15),
        ];

        let clave_chance = (0.20 + variety * 0.45).min(0.65);
        if rng.next_f32() < clave_chance {
            return Self::pick_weighted_cell(rng, CLAVE);
        }
        if d < 0.3 {
            return Self::pick_weighted_cell(rng, SPARSE);
        }
        Self::pick_weighted_cell(rng, DENSE)
    }

    /// Calculate lead note duration (until next lead trigger or end of bar),
    /// clamped down to the largest notation-safe value that fits.
    fn calculate_lead_duration(
        &self,
        current_step: usize,
        total_steps: usize,
        pattern: &[StepTrigger],
        is_high_tension: bool,
        fill_zone_start: usize,
    ) -> usize {
        // Look ahead for the next lead trigger
        let raw = 'outer: {
            for future_step in (current_step + 1)..total_steps {
                let trigger = if future_step < pattern.len() {
                    pattern[future_step]
                } else {
                    StepTrigger::default()
                };

                let would_play =
                    trigger.lead && !(is_high_tension && future_step >= fill_zone_start);
                if would_play {
                    break 'outer future_step - current_step;
                }
            }
            // Sustain until end of bar
            total_steps - current_step
        };

        // Clamp to largest notation-safe duration that fits within the raw gap
        Self::NOTATION_SAFE_STEPS.iter().copied().find(|&d| d <= raw).unwrap_or(1)
    }

    /// Update musical parameters (called when commands are processed)
    /// Snap `current_state` to match `musical_params` immediately (no morphing).
    /// Call after direct param changes to ensure newly generated measures
    /// use the correct tempo/density/etc. right away.
    pub fn snap_current_state(&mut self) {
        let mp = &self.musical_params;
        self.current_state.bpm = mp.bpm;
        self.current_state.density = mp.rhythm_density;
        self.current_state.tension = mp.harmony_tension;
        self.current_state.smoothness = mp.melody_smoothness;
        self.current_state.valence = mp.harmony_valence;
        self.current_state.arousal = (mp.bpm - 70.0) / 110.0;
    }

    pub fn update_params(&mut self, params: MusicalParams) {
        // Detect harmony mode changes
        if self.musical_params.harmony_mode != params.harmony_mode {
            self.harmony_mode = params.harmony_mode;
        }

        // Update sequencer density/tension FIRST — PerfectBalance and ClassicGroove
        // use these during pattern generation, so they must be set before any
        // regenerate_pattern() or prepare_next_bar() calls.
        self.sequencer_primary.tension = params.rhythm_tension;
        self.sequencer_primary.density = params.rhythm_density;

        // Track whether the current pattern needs regenerating
        let mode_changed = self.sequencer_primary.mode != params.rhythm_mode;
        let density_changed =
            (self.musical_params.rhythm_density - params.rhythm_density).abs() > f32::EPSILON;
        let tension_changed =
            (self.musical_params.rhythm_tension - params.rhythm_tension).abs() > f32::EPSILON;

        // Update sequencer mode and step count
        if mode_changed {
            self.sequencer_primary.mode = params.rhythm_mode;
            self.sequencer_primary.upgrade_to_steps(params.rhythm_steps);
        }

        let new_pulses = params.rhythm_pulses.min(self.sequencer_primary.steps);
        let pulses_changed = self.sequencer_primary.pulses != new_pulses;
        if pulses_changed {
            self.sequencer_primary.pulses = new_pulses;
        }

        let rotation_changed = self.sequencer_primary.rotation != params.rhythm_rotation;
        if rotation_changed {
            self.sequencer_primary.rotation = params.rhythm_rotation;
        }

        // Regenerate the CURRENT pattern when any pattern-affecting param changed.
        // Without this, bar 0 uses the stale pattern from construction while
        // subsequent bars get the correct pattern from prepare_next_bar().
        if mode_changed || density_changed || tension_changed || pulses_changed || rotation_changed
        {
            self.sequencer_primary.regenerate_pattern();
        }

        // Secondary sequencer
        self.sequencer_secondary.pulses =
            params.rhythm_secondary_pulses.min(params.rhythm_secondary_steps);
        self.sequencer_secondary.rotation = params.rhythm_secondary_rotation;

        // Melody
        self.harmony.set_hurst_factor(params.melody_smoothness);
        self.harmony.set_tension(params.harmony_tension);
        self.harmony.set_variety(
            params.variety.fractal_boost,
            params.variety.fractal_range,
            params.variety.motif_new_material_bias,
        );

        // Re-parse chord chart when it changes
        if self.musical_params.chord_chart != params.chord_chart {
            self.chord_chart = Self::parse_chord_chart(&params.chord_chart);
            self.chart_index = 0;
        }

        self.musical_params = params;
    }

    /// Reset chart index to 0 so the next generation starts from the first chord.
    /// Call this when regenerating the timeline from scratch.
    pub fn reset_chart_index(&mut self) {
        self.chart_index = 0;
        self.current_bar = 0;
    }

    /// Parse chord names into Chord structs, skipping invalid entries.
    fn parse_chord_chart(chart_names: &[arrayvec::ArrayString<16>]) -> Vec<crate::harmony::Chord> {
        chart_names
            .iter()
            .filter_map(|name| crate::harmony::Chord::from_name(name.as_str()).ok())
            .collect()
    }

    fn next_id(&mut self) -> NoteId {
        let id = self.next_note_id;
        self.next_note_id += 1;
        id
    }
}

/// Format a chord name using Roman numeral notation
fn format_chord_name(root_offset: i32, quality: ChordQuality) -> String {
    let roman = match root_offset {
        0 => "I",
        2 => "II",
        3 => "III",
        5 => "IV",
        7 => "V",
        8 => "VI",
        9 => "vi",
        10 => "VII",
        11 => "vii",
        _ => "?",
    };

    let quality_symbol = match quality {
        ChordQuality::Major => "",
        ChordQuality::Minor => "m",
        ChordQuality::Dominant7 => "7",
        ChordQuality::Diminished => "°",
        ChordQuality::Sus2 => "sus2",
    };

    format!("{roman}{quality_symbol}")
}

#[cfg(test)]
mod tests {
    use rust_music_theory::{note::PitchSymbol, scale::ScaleType};

    use super::*;
    use crate::params::TimeSignature;

    fn make_gen() -> TimelineGenerator {
        let seq_primary = Sequencer::new(16, 4, 120.0);
        let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
        let harmony = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
        let params = MusicalParams::default();
        let state = CurrentState {
            bpm: 120.0,
            density: 0.5,
            tension: 0.3,
            smoothness: 0.7,
            valence: 0.3,
            arousal: 0.5,
        };

        TimelineGenerator::new(
            seq_primary,
            seq_secondary,
            harmony,
            None, // No driver for basic tests
            params,
            state,
            TuningParams::default(),
        )
    }

    #[test]
    fn test_generate_measure_basic() {
        let mut tgen = make_gen();
        let mut rng = rand::thread_rng();

        let measure = tgen.generate_measure(1, &mut rng);

        assert_eq!(measure.index, 1);
        assert_eq!(measure.steps, 16);
        // Should have at least some notes (bass/lead/snare/hat from pattern)
        assert!(measure.total_notes() > 0, "Expected notes in generated measure");
    }

    #[test]
    fn test_generate_multiple_measures() {
        let mut tgen = make_gen();
        let mut rng = rand::thread_rng();

        let mut total_notes = 0;
        for i in 1..=8 {
            let measure = tgen.generate_measure(i, &mut rng);
            assert_eq!(measure.index, i);
            total_notes += measure.total_notes();
        }

        // 8 bars should produce a reasonable number of notes
        assert!(total_notes > 10, "Expected many notes across 8 bars, got {total_notes}");
    }

    #[test]
    fn test_generate_measure_with_driver() {
        let seq_primary = Sequencer::new(16, 4, 120.0);
        let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
        let harmony = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
        let driver = HarmonicDriver::new(0, &crate::tuning::HarmonyDriverParams::default()); // C
        let params = MusicalParams::default();
        let state = CurrentState {
            bpm: 120.0,
            density: 0.5,
            tension: 0.3,
            smoothness: 0.7,
            valence: 0.3,
            arousal: 0.5,
        };

        let mut tgen = TimelineGenerator::new(
            seq_primary,
            seq_secondary,
            harmony,
            Some(driver),
            params,
            state,
            TuningParams::default(),
        );

        let mut rng = rand::thread_rng();
        let measure = tgen.generate_measure(1, &mut rng);
        assert!(measure.total_notes() > 0);
    }

    #[test]
    fn test_generate_measure_rhythm_disabled() {
        let mut tgen = make_gen();
        tgen.musical_params.enable_rhythm = false;

        let mut rng = rand::thread_rng();
        let measure = tgen.generate_measure(1, &mut rng);

        // No bass, snare, or hat notes when rhythm is disabled
        assert_eq!(measure.notes_for_track(TrackId::Bass).len(), 0);
        assert_eq!(measure.notes_for_track(TrackId::Snare).len(), 0);
        assert_eq!(measure.notes_for_track(TrackId::Hat).len(), 0);
    }

    #[test]
    fn test_generate_measure_melody_disabled() {
        let mut tgen = make_gen();
        tgen.musical_params.enable_melody = false;

        let mut rng = rand::thread_rng();
        let measure = tgen.generate_measure(1, &mut rng);

        assert_eq!(measure.notes_for_track(TrackId::Lead).len(), 0);
    }

    #[test]
    fn test_note_ids_are_unique() {
        let mut tgen = make_gen();
        let mut rng = rand::thread_rng();

        let m1 = tgen.generate_measure(1, &mut rng);
        let m2 = tgen.generate_measure(2, &mut rng);

        let mut all_ids: Vec<NoteId> = Vec::new();
        for m in [&m1, &m2] {
            for track in &TrackId::ALL {
                for note in m.notes_for_track(*track) {
                    all_ids.push(note.id);
                }
            }
        }

        let original_len = all_ids.len();
        all_ids.sort();
        all_ids.dedup();
        assert_eq!(all_ids.len(), original_len, "All note IDs should be unique");
    }

    #[test]
    fn test_state_snapshot_captured() {
        let mut tgen = make_gen();
        tgen.current_state.tension = 0.8;

        let mut rng = rand::thread_rng();
        let measure = tgen.generate_measure(1, &mut rng);

        // State should be captured after morphing, so slightly less than 0.8
        // (morphing towards musical_params.harmony_tension=0.3)
        assert!(measure.state_snapshot.tension > 0.0);
    }

    #[test]
    fn test_lead_notes_within_instrument_range() {
        use crate::params::InstrumentConfig;

        let mut tgen = make_gen();
        tgen.musical_params.instrument_lead =
            InstrumentConfig { min_note: 60, max_note: 72, transposition_semitones: 0 };

        let mut rng = rand::thread_rng();
        for bar in 1..=8 {
            let measure = tgen.generate_measure(bar, &mut rng);
            for note in measure.notes_for_track(TrackId::Lead) {
                assert!(
                    note.pitch >= 60 && note.pitch <= 72,
                    "Lead note {} out of range [60, 72] at bar {bar}",
                    note.pitch,
                );
            }
        }
    }

    #[test]
    fn test_tenor_sax_lead_range() {
        use crate::params::InstrumentConfig;

        let mut tgen = make_gen();
        tgen.musical_params.instrument_lead = InstrumentConfig::tenor_sax();

        let mut rng = rand::thread_rng();
        for bar in 1..=16 {
            let measure = tgen.generate_measure(bar, &mut rng);
            for note in measure.notes_for_track(TrackId::Lead) {
                assert!(
                    note.pitch >= 56 && note.pitch <= 90,
                    "Tenor sax lead note {} out of range [56, 90] at bar {bar}",
                    note.pitch,
                );
            }
        }
    }

    #[test]
    fn test_different_time_signatures() {
        let mut tgen = make_gen();

        let mut rng = rand::thread_rng();

        // 3/4 time
        tgen.musical_params.time_signature = TimeSignature::new(3, 4);
        let measure = tgen.generate_measure(1, &mut rng);
        assert_eq!(measure.time_signature, TimeSignature::new(3, 4));
        assert_eq!(measure.steps, 12); // 3 beats * 4 ticks

        // 5/4 time
        tgen.musical_params.time_signature = TimeSignature::new(5, 4);
        let measure = tgen.generate_measure(2, &mut rng);
        assert_eq!(measure.steps, 20); // 5 beats * 4 ticks
    }

    /// Full pipeline integration test: TimelineGenerator with tenor sax config
    /// → generate measures → validate ranges → export MusicXML → validate XML structure.
    #[test]
    fn test_tenor_sax_full_pipeline() {
        use crate::{
            params::InstrumentConfig,
            timeline::{ScoreTimeline, export::timeline_to_musicxml_with_instruments},
        };

        let tenor = InstrumentConfig::tenor_sax();
        let mut tgen = make_gen();
        tgen.musical_params.instrument_lead = tenor;

        let mut rng = rand::thread_rng();
        let mut timeline = ScoreTimeline::new(20);

        for bar in 1..=16 {
            let measure = tgen.generate_measure(bar, &mut rng);

            // All lead notes must fall within tenor sax range
            for note in measure.notes_for_track(TrackId::Lead) {
                assert!(
                    note.pitch >= tenor.min_note && note.pitch <= tenor.max_note,
                    "Lead note MIDI {} out of tenor sax range [{}, {}] at bar {bar}",
                    note.pitch,
                    tenor.min_note,
                    tenor.max_note,
                );
            }

            timeline.push_measure(measure);
        }

        // Export with instrument config and validate XML structure
        let xml = timeline_to_musicxml_with_instruments(
            &timeline,
            "Integration Test - Tenor Sax",
            &tenor,
            &InstrumentConfig::default(),
        );

        // Part name
        assert!(xml.contains("<part-name>Tenor Saxophone</part-name>"));

        // Transpose element (Bb instrument: chromatic=-2, diatonic=-1)
        assert!(xml.contains("<transpose>"));
        assert!(xml.contains("<chromatic>-2</chromatic>"));
        assert!(xml.contains("<diatonic>-1</diatonic>"));

        // Bass part should have no transpose (default config)
        // Count occurrences of <transpose> — should be exactly 1 (lead only)
        let transpose_count = xml.matches("<transpose>").count();
        assert_eq!(transpose_count, 1, "Only the lead part should have <transpose>");

        // Valid MusicXML structure
        assert!(xml.contains("score-partwise"));
        assert!(xml.contains("<part-name>Bass</part-name>"));
        assert!(xml.contains("<part-name>Drums</part-name>"));
    }

    /// Validate that generated lead notes only use notation-safe durations
    /// across a range of density/tension parameters.
    #[test]
    fn test_lead_durations_are_notation_safe() {
        const NOTATION_SAFE: [usize; 8] = [16, 12, 8, 6, 4, 3, 2, 1];

        for &density in &[0.0, 0.25, 0.5, 0.75, 1.0] {
            for &tension in &[0.0, 0.5, 1.0] {
                let seq_primary = Sequencer::new(16, 4, 120.0);
                let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
                let harmony = HarmonyNavigator::new(
                    rust_music_theory::note::PitchSymbol::C,
                    rust_music_theory::scale::ScaleType::PentatonicMajor,
                    4,
                );
                let params = MusicalParams::default();
                let state = CurrentState {
                    bpm: 120.0,
                    density,
                    tension,
                    smoothness: 0.7,
                    valence: 0.3,
                    arousal: 0.5,
                };

                let mut tgen = TimelineGenerator::new(
                    seq_primary,
                    seq_secondary,
                    harmony,
                    None,
                    params,
                    state,
                    TuningParams::default(),
                );
                let mut rng = rand::thread_rng();

                for bar in 1..=8 {
                    let measure = tgen.generate_measure(bar, &mut rng);
                    for note in measure.notes_for_track(TrackId::Lead) {
                        assert!(
                            NOTATION_SAFE.contains(&note.duration_steps),
                            "density={density}, tension={tension}, bar={bar}: \
                             lead note at step {} has duration_steps={} which is not notation-safe",
                            note.start_step,
                            note.duration_steps,
                        );
                    }
                }
            }
        }
    }

    /// Diagnostic: Compare note density of bar 0 vs subsequent bars across
    /// all rhythm modes and density levels. Uses deterministic seed.
    #[test]
    fn test_first_bar_density_not_higher_than_subsequent() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let num_bars = 16;
        let seeds: [u64; 5] = [42, 123, 999, 7777, 54321];

        let configs: Vec<(&str, RhythmMode, f32, f32)> = vec![
            ("euclidean-default", RhythmMode::Euclidean, 0.5, 0.3),
            ("euclidean-high-density", RhythmMode::Euclidean, 0.8, 0.3),
            ("balanced-default", RhythmMode::PerfectBalance, 0.5, 0.3),
            ("balanced-high-density", RhythmMode::PerfectBalance, 0.8, 0.5),
            ("groove-default", RhythmMode::ClassicGroove, 0.5, 0.3),
            ("groove-high-density", RhythmMode::ClassicGroove, 0.8, 0.5),
        ];

        for (label, mode, density, tension) in &configs {
            for &seed in &seeds {
                let mut rng = ChaCha8Rng::seed_from_u64(seed);

                let mut seq_primary = Sequencer::new_with_mode(16, 4, 120.0, *mode);
                seq_primary.density = *density;
                seq_primary.tension = *tension;
                seq_primary.regenerate_pattern();

                let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
                let harmony = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
                let mut params = MusicalParams::default();
                params.rhythm_mode = *mode;
                params.rhythm_density = *density;
                params.rhythm_tension = *tension;
                let state = CurrentState {
                    bpm: 120.0,
                    density: *density,
                    tension: *tension,
                    smoothness: 0.7,
                    valence: 0.3,
                    arousal: 0.5,
                };

                let mut tgen = TimelineGenerator::new(
                    seq_primary,
                    seq_secondary,
                    harmony,
                    None,
                    params,
                    state,
                    TuningParams::default(),
                );

                let mut lead_counts: Vec<usize> = Vec::new();
                let mut total_counts: Vec<usize> = Vec::new();

                for bar in 0..num_bars {
                    let measure = tgen.generate_measure(bar, &mut rng);
                    lead_counts.push(measure.notes_for_track(TrackId::Lead).len());
                    total_counts.push(measure.total_notes());
                }

                let bar0_lead = lead_counts[0] as f64;
                let rest_lead_avg: f64 =
                    lead_counts[1..].iter().sum::<usize>() as f64 / (num_bars - 1) as f64;
                let bar0_total = total_counts[0] as f64;
                let rest_total_avg: f64 =
                    total_counts[1..].iter().sum::<usize>() as f64 / (num_bars - 1) as f64;

                if rest_lead_avg > 0.0 {
                    assert!(
                        bar0_lead <= rest_lead_avg * 1.5,
                        "{label} seed={seed}: Bar 0 lead ({bar0_lead}) > 1.5x avg ({rest_lead_avg:.1})"
                    );
                }
                if rest_total_avg > 0.0 {
                    assert!(
                        bar0_total <= rest_total_avg * 1.5,
                        "{label} seed={seed}: Bar 0 total ({bar0_total}) > 1.5x avg ({rest_total_avg:.1})"
                    );
                }
            }
        }
    }

    /// Reproduce the practice app initialization: composer creates with Euclidean,
    /// then update_params switches to PerfectBalance. Bar 0 should use the correct mode.
    #[test]
    fn test_mode_switch_bar0_uses_correct_pattern() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        // Step 1: Create with Euclidean (mimics MusicComposer::new)
        let seq_primary = Sequencer::new(16, 4, 120.0);
        let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
        let harmony = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
        let state = CurrentState {
            bpm: 120.0,
            density: 0.5,
            tension: 0.3,
            smoothness: 0.7,
            valence: 0.3,
            arousal: 0.5,
        };
        let mut params = MusicalParams::default();

        let mut tgen = TimelineGenerator::new(
            seq_primary,
            seq_secondary,
            harmony,
            None,
            params.clone(),
            state,
            TuningParams::default(),
        );

        // Step 2: Switch to PerfectBalance with high density (mimics sync_generator)
        params.rhythm_mode = RhythmMode::PerfectBalance;
        params.rhythm_density = 0.8;
        params.rhythm_tension = 0.5;
        // Disable rhythmic-cell variety so this test's strict count equality
        // depends only on pattern equality, not on RNG-driven cell choices.
        params.variety.rhythmic_cell_variety = 0.0;
        tgen.update_params(params);

        // Step 3: Generate bars — bar 0 should match bar 1+
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut total_counts: Vec<usize> = Vec::new();
        let mut lead_counts: Vec<usize> = Vec::new();

        for bar in 0..8 {
            let measure = tgen.generate_measure(bar, &mut rng);
            total_counts.push(measure.total_notes());
            lead_counts.push(measure.notes_for_track(TrackId::Lead).len());
        }

        // Bar 0 must NOT differ from subsequent bars
        assert_eq!(
            total_counts[0], total_counts[1],
            "Bar 0 total ({}) differs from bar 1 ({}) — \
             stale pattern leaked after mode switch",
            total_counts[0], total_counts[1],
        );
        assert_eq!(
            lead_counts[0], lead_counts[1],
            "Bar 0 lead ({}) differs from bar 1 ({}) — \
             stale pattern leaked after mode switch",
            lead_counts[0], lead_counts[1],
        );
    }

    /// Chart mode: fixed chord sequence cycles correctly
    #[test]
    fn test_chart_mode_cycles_chords() {
        use arrayvec::ArrayString;

        let seq_primary = Sequencer::new(16, 4, 120.0);
        let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
        let harmony = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
        let mut params = MusicalParams::default();
        params.harmony_mode = HarmonyMode::Chart;
        params.harmony_measures_per_chord = 1; // 1 chord per bar
        params.chord_chart = vec![
            ArrayString::from("Cmaj7").unwrap(),
            ArrayString::from("Dm7").unwrap(),
            ArrayString::from("G7").unwrap(),
            ArrayString::from("Cmaj7").unwrap(),
        ];
        let state = CurrentState {
            bpm: 120.0,
            density: 0.5,
            tension: 0.3,
            smoothness: 0.7,
            valence: 0.3,
            arousal: 0.5,
        };

        let mut tgen = TimelineGenerator::new(
            seq_primary,
            seq_secondary,
            harmony,
            None,
            params,
            state,
            TuningParams::default(),
        );
        let mut rng = rand::thread_rng();

        // Generate 8 bars — should cycle through 4-chord chart twice
        let mut chord_names: Vec<String> = Vec::new();
        for bar in 0..8 {
            let _measure = tgen.generate_measure(bar, &mut rng);
            chord_names.push(tgen.chord_name.clone());
        }

        // First 4 bars = first cycle, next 4 = second cycle
        assert_eq!(chord_names[0], "Cmaj7");
        assert_eq!(chord_names[1], "Dm7");
        assert_eq!(chord_names[2], "G7");
        assert_eq!(chord_names[3], "Cmaj7");
        // Loop
        assert_eq!(chord_names[4], "Cmaj7");
        assert_eq!(chord_names[5], "Dm7");
        assert_eq!(chord_names[6], "G7");
        assert_eq!(chord_names[7], "Cmaj7");
    }

    /// Chart mode with measures_per_chord=2: chord changes every 2 bars
    #[test]
    fn test_chart_mode_measures_per_chord() {
        use arrayvec::ArrayString;

        let seq_primary = Sequencer::new(16, 4, 120.0);
        let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
        let harmony = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
        let mut params = MusicalParams::default();
        params.harmony_mode = HarmonyMode::Chart;
        params.harmony_measures_per_chord = 2;
        params.chord_chart =
            vec![ArrayString::from("Am7").unwrap(), ArrayString::from("D7").unwrap()];
        let state = CurrentState {
            bpm: 120.0,
            density: 0.5,
            tension: 0.3,
            smoothness: 0.7,
            valence: 0.3,
            arousal: 0.5,
        };

        let mut tgen = TimelineGenerator::new(
            seq_primary,
            seq_secondary,
            harmony,
            None,
            params,
            state,
            TuningParams::default(),
        );
        let mut rng = rand::thread_rng();

        let mut chord_names: Vec<String> = Vec::new();
        for bar in 0..8 {
            let _measure = tgen.generate_measure(bar, &mut rng);
            chord_names.push(tgen.chord_name.clone());
        }

        // Am7 for bars 0-1, D7 for bars 2-3, Am7 for 4-5, D7 for 6-7
        assert_eq!(chord_names[0], "Am7");
        assert_eq!(chord_names[1], "Am7");
        assert_eq!(chord_names[2], "D7");
        assert_eq!(chord_names[3], "D7");
        assert_eq!(chord_names[4], "Am7");
        assert_eq!(chord_names[5], "Am7");
    }

    /// Chart mode with flat notation input normalizes to sharps
    #[test]
    fn test_chart_mode_flat_input() {
        use arrayvec::ArrayString;

        let seq_primary = Sequencer::new(16, 4, 120.0);
        let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
        let harmony = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
        let mut params = MusicalParams::default();
        params.harmony_mode = HarmonyMode::Chart;
        params.harmony_measures_per_chord = 1;
        params.chord_chart =
            vec![ArrayString::from("Bbmaj7").unwrap(), ArrayString::from("Ebm7").unwrap()];
        let state = CurrentState {
            bpm: 120.0,
            density: 0.5,
            tension: 0.3,
            smoothness: 0.7,
            valence: 0.3,
            arousal: 0.5,
        };

        let mut tgen = TimelineGenerator::new(
            seq_primary,
            seq_secondary,
            harmony,
            None,
            params,
            state,
            TuningParams::default(),
        );
        let mut rng = rand::thread_rng();

        let _measure = tgen.generate_measure(0, &mut rng);
        // Bb normalizes to A# in display
        assert_eq!(tgen.chord_name, "A#maj7");

        let _measure = tgen.generate_measure(1, &mut rng);
        // Eb normalizes to D#
        assert_eq!(tgen.chord_name, "D#m7");
    }

    /// Integration test: chart mode generates music with correct chord context
    /// in each measure AND lead notes are influenced by the active chord.
    #[test]
    fn test_chart_mode_integration_chord_context_and_notes() {
        use std::collections::HashSet;

        use arrayvec::ArrayString;
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        // ii-V-I in C major — a standard jazz progression
        let chart = vec![
            ArrayString::from("Dm7").unwrap(),   // D F A C
            ArrayString::from("G7").unwrap(),    // G B D F
            ArrayString::from("Cmaj7").unwrap(), // C E G B
        ];

        let seq_primary = Sequencer::new(16, 4, 120.0);
        let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
        let harmony = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
        let mut params = MusicalParams::default();
        params.harmony_mode = HarmonyMode::Chart;
        params.harmony_measures_per_chord = 1;
        params.chord_chart = chart;
        params.key_root = 0; // C
        let state = CurrentState {
            bpm: 120.0,
            density: 0.5,
            tension: 0.3,
            smoothness: 0.7,
            valence: 0.3,
            arousal: 0.5,
        };

        let mut tgen = TimelineGenerator::new(
            seq_primary,
            seq_secondary,
            harmony,
            None,
            params,
            state,
            TuningParams::default(),
        );
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let expected_chords = ["Dm7", "G7", "Cmaj7", "Dm7", "G7", "Cmaj7"];
        // Chord tones (pitch classes) for each chord
        let chord_tones: [HashSet<u8>; 3] = [
            [2, 5, 9, 0].into_iter().collect(),  // Dm7: D F A C
            [7, 11, 2, 5].into_iter().collect(), // G7: G B D F
            [0, 4, 7, 11].into_iter().collect(), // Cmaj7: C E G B
        ];

        let mut total_lead = 0usize;
        let mut total_chord_hits = 0usize;

        for (bar, expected_name) in expected_chords.iter().enumerate() {
            let measure = tgen.generate_measure(bar, &mut rng);

            // 1. Verify chord_context in the measure matches the chart
            assert_eq!(
                measure.chord_context.chord_name, *expected_name,
                "Bar {bar}: chord_context.chord_name should be \"{expected_name}\", \
                 got \"{}\"",
                measure.chord_context.chord_name
            );

            // 2. Verify the measure produces notes
            let lead_notes = measure.notes_for_track(TrackId::Lead);
            assert!(
                !lead_notes.is_empty(),
                "Bar {bar} ({expected_name}): expected lead notes but got none"
            );

            // 3. Collect chord-tone hits for aggregate check
            let active_tones = &chord_tones[bar % 3];
            let chord_tone_count =
                lead_notes.iter().filter(|n| active_tones.contains(&(n.pitch % 12))).count();
            total_lead += lead_notes.len();
            total_chord_hits += chord_tone_count;
        }

        // Verify chord influence across ALL bars (not per-bar, since individual bars
        // with few notes can randomly miss chord tones with broader interval weights)
        let overall_ratio = total_chord_hits as f64 / total_lead as f64;
        assert!(
            overall_ratio >= 0.15,
            "Overall: only {total_chord_hits}/{total_lead} lead notes ({:.0}%) are chord tones \
             — expected ≥15% across all bars",
            overall_ratio * 100.0,
        );
    }

    /// Integration test: chart mode through NativeHandle-like pipeline.
    /// Generates 12 bars with a 4-chord chart, verifies all measures have
    /// correct chord names and the chart loops correctly.
    #[test]
    fn test_chart_mode_full_pipeline_loop() {
        use arrayvec::ArrayString;
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let chart = vec![
            ArrayString::from("Am7").unwrap(),
            ArrayString::from("Dm7").unwrap(),
            ArrayString::from("G7").unwrap(),
            ArrayString::from("Cmaj7").unwrap(),
        ];

        let seq_primary = Sequencer::new(16, 4, 120.0);
        let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
        let harmony = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
        let mut params = MusicalParams::default();
        params.harmony_mode = HarmonyMode::Chart;
        params.harmony_measures_per_chord = 1;
        params.chord_chart = chart;
        let state = CurrentState {
            bpm: 120.0,
            density: 0.5,
            tension: 0.3,
            smoothness: 0.7,
            valence: 0.3,
            arousal: 0.5,
        };

        let mut tgen = TimelineGenerator::new(
            seq_primary,
            seq_secondary,
            harmony,
            None,
            params,
            state,
            TuningParams::default(),
        );
        let mut rng = ChaCha8Rng::seed_from_u64(123);

        let expected = ["Am7", "Dm7", "G7", "Cmaj7"];
        let mut total_notes = 0;

        for bar in 0..12 {
            let measure = tgen.generate_measure(bar, &mut rng);

            // Chord should cycle: bar % 4
            let expected_chord = expected[bar % 4];
            assert_eq!(
                measure.chord_context.chord_name, expected_chord,
                "Bar {bar}: expected \"{expected_chord}\", got \"{}\"",
                measure.chord_context.chord_name
            );

            // Every bar should produce music
            let notes = measure.total_notes();
            assert!(notes > 0, "Bar {bar} ({expected_chord}): no notes generated");
            total_notes += notes;
        }

        // 12 bars should produce substantial music
        assert!(total_notes > 50, "Expected >50 notes across 12 bars, got {total_notes}");
    }

    /// Verify that PerfectBalance mode produces only clean subdivisions
    /// (no dotted eighths) across the full density range.
    #[test]
    fn test_lead_durations_clean_subdivisions_balanced() {
        // CORELIB-1: With rhythmic cell subdivision, lead durations can now include
        // all notation-safe values (1,2,3,4,6,8,12,16) plus dotted patterns.
        // We verify durations are reasonable (1-16) and that subdivision
        // produces varied durations at higher densities.
        for &density in &[0.0, 0.25, 0.5, 0.75, 1.0] {
            let mut seq_primary =
                Sequencer::new_with_mode(16, 4, 120.0, RhythmMode::PerfectBalance);
            seq_primary.density = density;
            seq_primary.tension = 0.0;
            seq_primary.regenerate_pattern();
            let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
            let harmony = HarmonyNavigator::new(
                rust_music_theory::note::PitchSymbol::C,
                rust_music_theory::scale::ScaleType::PentatonicMajor,
                4,
            );
            let mut params = MusicalParams::default();
            params.rhythm_density = density;
            params.rhythm_mode = RhythmMode::PerfectBalance;
            let state = CurrentState {
                bpm: 120.0,
                density,
                tension: 0.0,
                smoothness: 0.7,
                valence: 0.3,
                arousal: 0.5,
            };

            let mut tgen = TimelineGenerator::new(
                seq_primary,
                seq_secondary,
                harmony,
                None,
                params,
                state,
                TuningParams::default(),
            );
            let mut rng = rand::thread_rng();
            let mut durations_seen = std::collections::HashSet::new();

            for bar in 1..=16 {
                let measure = tgen.generate_measure(bar, &mut rng);
                for note in measure.notes_for_track(TrackId::Lead) {
                    assert!(
                        note.duration_steps >= 1 && note.duration_steps <= 16,
                        "density={density}, bar={bar}: lead duration {} out of range [1,16]",
                        note.duration_steps,
                    );
                    durations_seen.insert(note.duration_steps);
                }
            }

            // Verify we got some notes (rests may skip some, but not all)
            assert!(
                !durations_seen.is_empty(),
                "density={density}: expected at least some lead notes",
            );
        }
    }
}

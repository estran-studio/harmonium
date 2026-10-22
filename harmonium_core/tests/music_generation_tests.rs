//! Music Generation Integration Tests
//!
//! These tests run the ACTUAL music generation (Sequencer + `HarmonicDriver`)
//! and export the results to `MusicXML` for visual review in `MuseScore`.
//!
//! Run with: `cargo test -p harmonium_core --test music_generation_tests -- --ignored --nocapture`
//!
//! Output files: `target/generated_music/`

use std::path::Path;

use harmonium_core::{
    events::AudioEvent,
    export::{ChordSymbol, write_musicxml_with_chords},
    harmony::driver::HarmonicDriver,
    params::{InstrumentConfig, MelodyScaleType, MusicalParams},
    sequencer::{RhythmMode, Sequencer},
};

/// Output directory for generated music
const OUTPUT_DIR: &str = "target/generated_music";

/// Standard samples per step at 120 BPM, 44100 Hz
const SAMPLES_PER_STEP: usize = 11025;

/// Simple deterministic RNG for reproducible tests
struct TestRng(u64);

impl TestRng {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.0 >> 16) & 0x7fff) as f32 / 32768.0
    }

    fn next_range(&mut self, range: std::ops::Range<usize>) -> usize {
        let f = self.next_f32();
        range.start + (f * (range.end - range.start) as f32) as usize
    }
}

impl harmonium_core::harmony::RngCore for TestRng {
    fn next_f32(&mut self) -> f32 {
        Self::next_f32(self)
    }

    fn next_range_usize(&mut self, range: std::ops::Range<usize>) -> usize {
        self.next_range(range)
    }
}

fn setup_output_dir() {
    std::fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");
}

/// Generate music and export to `MusicXML`
fn generate_and_export(name: &str, params: &MusicalParams, measures: usize, seed: u64) {
    setup_output_dir();

    let mut rng = TestRng::new(seed);
    let steps_per_measure = params.rhythm_steps;
    let total_steps = measures * steps_per_measure;

    // Setup sequencer
    let mut sequencer = Sequencer::new_with_mode(
        params.rhythm_steps,
        params.rhythm_pulses,
        params.bpm,
        params.rhythm_mode,
    );
    // Set density and tension, then regenerate pattern
    sequencer.density = params.rhythm_density;
    sequencer.tension = params.rhythm_tension;
    sequencer.regenerate_pattern();

    // Setup harmony driver
    let mut driver = HarmonicDriver::new(
        params.key_root,
        &harmonium_core::tuning::HarmonyDriverParams::default(),
    );

    // Collect events and chord symbols
    let mut events: Vec<(f64, AudioEvent)> = Vec::new(); // Changed to f64 for step-based timestamps
    let mut chord_symbols: Vec<ChordSymbol> = Vec::new();
    let mut current_chord = driver.current_chord().clone();
    let mut steps_in_chord = 0;
    let steps_per_chord = params.harmony_measures_per_chord * steps_per_measure;

    // Record initial chord at step 0
    chord_symbols.push(ChordSymbol::new(0, current_chord.root, current_chord.chord_type.suffix()));

    // Active notes for NoteOff tracking
    let mut active_bass: Option<u8> = None;
    let mut active_lead: Option<u8> = None;

    for step in 0..total_steps {
        let timestamp = step as f64; // Use step directly as timestamp

        // === HARMONY: Change chord every N measures ===
        if steps_in_chord >= steps_per_chord {
            let decision =
                driver.next_chord(params.harmony_tension, params.harmony_valence, &mut rng);
            current_chord = decision.next_chord;
            steps_in_chord = 0;

            // Record chord change
            chord_symbols.push(ChordSymbol::new(
                step,
                current_chord.root,
                current_chord.chord_type.suffix(),
            ));
        }
        steps_in_chord += 1;

        // === RHYTHM: Get triggers from sequencer ===
        let tick_result = sequencer.tick();
        let trigger = tick_result.trigger;

        // === BASS (Channel 2) ===
        if trigger.kick {
            // Release previous bass note
            if let Some(prev_note) = active_bass.take() {
                events.push((timestamp, AudioEvent::NoteOff { note: prev_note, channel: 2 }));
            }

            // Play new bass note (harmonized to current chord root)
            let bass_note = if params.fixed_kick {
                36 // Fixed C1 for drum kit mode
            } else {
                // Harmonized bass: chord root in octave 2
                36 + current_chord.root
            };
            let velocity = (trigger.velocity * 100.0) as u8;
            events.push((timestamp, AudioEvent::NoteOn { note: bass_note, velocity, channel: 2 }));
            active_bass = Some(bass_note);
        }

        // === SNARE / HEEL (Channel 3) ===
        if trigger.snare {
            let velocity = (trigger.velocity * 90.0) as u8;
            events.push((timestamp, AudioEvent::NoteOn { note: 38, velocity, channel: 3 }));
            // Short duration for drums (half step)
            let off_time = timestamp + 0.5;
            events.push((off_time, AudioEvent::NoteOff { note: 38, channel: 3 }));
        }

        // === HI-HAT / TOE (Channel 4) ===
        if trigger.hat {
            let velocity = (trigger.velocity * 70.0) as u8;
            events.push((timestamp, AudioEvent::NoteOn { note: 42, velocity, channel: 4 }));
            let off_time = timestamp + 0.25; // Quarter step
            events.push((off_time, AudioEvent::NoteOff { note: 42, channel: 4 }));
        }

        // === LEAD/MELODY (Channel 0) ===
        // Play melody on every other step for a simple pattern
        let is_melody_step = step % 2 == 0;
        if is_melody_step {
            // Release previous lead note
            if let Some(prev_note) = active_lead.take() {
                events.push((timestamp, AudioEvent::NoteOff { note: prev_note, channel: 0 }));
            }

            // Generate melody note from chord tones
            // melody_octave=4 means standard notation octave 4 (middle C = C4 = MIDI 60)
            // MIDI formula: 12 * (octave + 1) + pitch_class
            let chord_pitches = current_chord.pitch_classes();
            let melody_octave = params.melody_octave as u8;
            let base_pitch = chord_pitches[rng.next_range(0..chord_pitches.len())];
            let melody_note = 12 * (melody_octave + 1) + base_pitch; // +1 to convert to MIDI octave

            events.push((
                timestamp,
                AudioEvent::NoteOn { note: melody_note, velocity: 80, channel: 0 },
            ));
            active_lead = Some(melody_note);
        }
    }

    // Final NoteOffs
    let end_time = total_steps as f64;
    if let Some(note) = active_bass {
        events.push((end_time, AudioEvent::NoteOff { note, channel: 2 }));
    }
    if let Some(note) = active_lead {
        events.push((end_time, AudioEvent::NoteOff { note, channel: 0 }));
    }

    // Sort events by timestamp
    events.sort_by(|(t1, _), (t2, _)| t1.partial_cmp(t2).unwrap());

    // Export to MusicXML with chord symbols
    let path = Path::new(OUTPUT_DIR).join(format!("{name}.musicxml"));
    write_musicxml_with_chords(&events, &chord_symbols, params, SAMPLES_PER_STEP, &path)
        .unwrap_or_else(|_| panic!("Failed to write {name}"));

    println!(
        "Generated: {} ({} measures, {} events, {} chords)",
        path.display(),
        measures,
        events.len(),
        chord_symbols.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// RHYTHM MODE TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore]
fn test_euclidean_4_pulses() {
    let mut params = MusicalParams::default();
    params.rhythm_mode = RhythmMode::Euclidean;
    params.rhythm_steps = 16;
    params.rhythm_pulses = 4;
    params.key_root = 0; // C

    generate_and_export("rhythm_euclidean_4_pulses", &params, 4, 42);
}

#[test]
#[ignore]
fn test_euclidean_5_pulses() {
    let mut params = MusicalParams::default();
    params.rhythm_mode = RhythmMode::Euclidean;
    params.rhythm_steps = 16;
    params.rhythm_pulses = 5; // Creates interesting syncopation

    generate_and_export("rhythm_euclidean_5_pulses", &params, 4, 42);
}

#[test]
#[ignore]
fn test_euclidean_7_pulses() {
    let mut params = MusicalParams::default();
    params.rhythm_mode = RhythmMode::Euclidean;
    params.rhythm_steps = 16;
    params.rhythm_pulses = 7; // Afro-Cuban feel

    generate_and_export("rhythm_euclidean_7_pulses", &params, 4, 42);
}

#[test]
#[ignore]
fn test_perfect_balance_low_density() {
    let mut params = MusicalParams::default();
    params.rhythm_mode = RhythmMode::PerfectBalance;
    params.rhythm_steps = 48;
    params.rhythm_density = 0.3;
    params.rhythm_tension = 0.2;

    generate_and_export("rhythm_perfect_balance_low_density", &params, 4, 42);
}

#[test]
#[ignore]
fn test_perfect_balance_high_density() {
    let mut params = MusicalParams::default();
    params.rhythm_mode = RhythmMode::PerfectBalance;
    params.rhythm_steps = 48;
    params.rhythm_density = 0.8;
    params.rhythm_tension = 0.6;

    generate_and_export("rhythm_perfect_balance_high_density", &params, 4, 42);
}

#[test]
#[ignore]
fn test_classic_groove_half_time() {
    let mut params = MusicalParams::default();
    params.rhythm_mode = RhythmMode::ClassicGroove;
    params.rhythm_steps = 16;
    params.rhythm_density = 0.2; // Half-time

    generate_and_export("rhythm_classic_half_time", &params, 4, 42);
}

#[test]
#[ignore]
fn test_classic_groove_four_on_floor() {
    let mut params = MusicalParams::default();
    params.rhythm_mode = RhythmMode::ClassicGroove;
    params.rhythm_steps = 16;
    params.rhythm_density = 0.5; // Four-on-the-floor

    generate_and_export("rhythm_classic_four_on_floor", &params, 4, 42);
}

#[test]
#[ignore]
fn test_classic_groove_breakbeat() {
    let mut params = MusicalParams::default();
    params.rhythm_mode = RhythmMode::ClassicGroove;
    params.rhythm_steps = 16;
    params.rhythm_density = 0.9; // Breakbeat

    generate_and_export("rhythm_classic_breakbeat", &params, 4, 42);
}

// ═══════════════════════════════════════════════════════════════════════════
// HARMONY TENSION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore]
fn test_harmony_low_tension_steedman() {
    let mut params = MusicalParams::default();
    params.harmony_tension = 0.2; // Should use Steedman (functional harmony)
    params.harmony_valence = 0.5; // Major
    params.harmony_measures_per_chord = 1;

    generate_and_export("harmony_low_tension_steedman", &params, 8, 42);
}

#[test]
#[ignore]
fn test_harmony_medium_tension_parsimonious() {
    let mut params = MusicalParams::default();
    params.harmony_tension = 0.55; // Should use Parsimonious
    params.harmony_valence = 0.3;
    params.harmony_measures_per_chord = 1;

    generate_and_export("harmony_medium_tension_parsimonious", &params, 8, 42);
}

#[test]
#[ignore]
fn test_harmony_high_tension_neo_riemannian() {
    let mut params = MusicalParams::default();
    params.harmony_tension = 0.8; // Should use Neo-Riemannian
    params.harmony_valence = 0.0; // Neutral
    params.harmony_measures_per_chord = 1;

    generate_and_export("harmony_high_tension_neo_riemannian", &params, 8, 42);
}

#[test]
#[ignore]
fn test_harmony_minor_key() {
    let mut params = MusicalParams::default();
    params.harmony_tension = 0.3;
    params.harmony_valence = -0.7; // Minor
    params.key_root = 9; // A minor
    params.harmony_measures_per_chord = 2;

    generate_and_export("harmony_a_minor", &params, 8, 42);
}

#[test]
#[ignore]
fn test_harmony_major_key() {
    let mut params = MusicalParams::default();
    params.harmony_tension = 0.3;
    params.harmony_valence = 0.7; // Major
    params.key_root = 7; // G major
    params.harmony_measures_per_chord = 2;

    generate_and_export("harmony_g_major", &params, 8, 42);
}

// ═══════════════════════════════════════════════════════════════════════════
// KEY SIGNATURE TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore]
fn test_key_c_major() {
    let mut params = MusicalParams::default();
    params.key_root = 0;
    params.harmony_valence = 0.5;
    generate_and_export("key_c_major", &params, 4, 42);
}

#[test]
#[ignore]
fn test_key_d_major() {
    let mut params = MusicalParams::default();
    params.key_root = 2;
    params.harmony_valence = 0.5;
    generate_and_export("key_d_major", &params, 4, 42);
}

#[test]
#[ignore]
fn test_key_f_major() {
    let mut params = MusicalParams::default();
    params.key_root = 5;
    params.harmony_valence = 0.5;
    generate_and_export("key_f_major", &params, 4, 42);
}

#[test]
#[ignore]
fn test_key_bb_major() {
    let mut params = MusicalParams::default();
    params.key_root = 10;
    params.harmony_valence = 0.5;
    generate_and_export("key_bb_major", &params, 4, 42);
}

#[test]
#[ignore]
fn test_key_e_minor() {
    let mut params = MusicalParams::default();
    params.key_root = 4;
    params.harmony_valence = -0.5;
    generate_and_export("key_e_minor", &params, 4, 42);
}

// ═══════════════════════════════════════════════════════════════════════════
// COMBINED SCENARIO TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore]
fn test_scenario_calm_ambient() {
    let mut params = MusicalParams::default();
    params.bpm = 80.0;
    params.rhythm_mode = RhythmMode::Euclidean;
    params.rhythm_steps = 16;
    params.rhythm_pulses = 3;
    params.rhythm_density = 0.3;
    params.harmony_tension = 0.2;
    params.harmony_valence = 0.4;
    params.key_root = 0; // C major

    generate_and_export("scenario_calm_ambient", &params, 8, 123);
}

#[test]
#[ignore]
fn test_scenario_energetic_dance() {
    let mut params = MusicalParams::default();
    params.bpm = 128.0;
    params.rhythm_mode = RhythmMode::ClassicGroove;
    params.rhythm_steps = 16;
    params.rhythm_density = 0.5; // Four-on-the-floor
    params.harmony_tension = 0.5;
    params.harmony_valence = 0.6;
    params.key_root = 7; // G major

    generate_and_export("scenario_energetic_dance", &params, 8, 456);
}

#[test]
#[ignore]
fn test_scenario_dark_tense() {
    let mut params = MusicalParams::default();
    params.bpm = 100.0;
    params.rhythm_mode = RhythmMode::PerfectBalance;
    params.rhythm_steps = 48;
    params.rhythm_density = 0.6;
    params.rhythm_tension = 0.7;
    params.harmony_tension = 0.8;
    params.harmony_valence = -0.8; // Minor
    params.key_root = 2; // D minor

    generate_and_export("scenario_dark_tense", &params, 8, 789);
}

#[test]
#[ignore]
fn test_scenario_jazz_swing() {
    let mut params = MusicalParams::default();
    params.bpm = 140.0;
    params.rhythm_mode = RhythmMode::Euclidean;
    params.rhythm_steps = 12; // 3/4 or swing feel
    params.rhythm_pulses = 4;
    params.harmony_tension = 0.5; // Parsimonious for 7th chords
    params.harmony_valence = 0.2;
    params.harmony_measures_per_chord = 1;
    params.key_root = 5; // F major

    generate_and_export("scenario_jazz_swing", &params, 8, 999);
}

// ═══════════════════════════════════════════════════════════════════════════
// ODD METER TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore]
fn test_odd_meter_3_4() {
    let mut params = MusicalParams::default();
    params.time_signature = harmonium_core::params::TimeSignature::new(3, 4);
    params.rhythm_mode = RhythmMode::ClassicGroove;
    params.rhythm_steps = 12; // 3 beats * 4 ticks
    params.rhythm_density = 0.5;

    generate_and_export("odd_meter_3_4", &params, 4, 42);
}

#[test]
#[ignore]
fn test_odd_meter_5_4() {
    let mut params = MusicalParams::default();
    params.time_signature = harmonium_core::params::TimeSignature::new(5, 4);
    params.rhythm_mode = RhythmMode::ClassicGroove;
    params.rhythm_steps = 20; // 5 beats * 4 ticks
    params.rhythm_density = 0.6;

    generate_and_export("odd_meter_5_4", &params, 4, 42);
}

#[test]
#[ignore]
fn test_odd_meter_7_8() {
    let mut params = MusicalParams::default();
    params.time_signature = harmonium_core::params::TimeSignature::new(7, 8);
    params.rhythm_mode = RhythmMode::ClassicGroove;
    params.rhythm_steps = 14; // 7 * (4*4/8) = 14 ticks
    params.rhythm_density = 0.7;

    generate_and_export("odd_meter_7_8", &params, 4, 42);
}

// ═══════════════════════════════════════════════════════════════════════════
// GENERATE ALL
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore]
fn generate_all_music_tests() {
    println!("\n========================================");
    println!("Generating music with REAL generation...");
    println!("Output: {OUTPUT_DIR}/");
    println!("========================================\n");

    // Rhythm tests
    test_euclidean_4_pulses();
    test_euclidean_5_pulses();
    test_euclidean_7_pulses();
    test_perfect_balance_low_density();
    test_perfect_balance_high_density();
    test_classic_groove_half_time();
    test_classic_groove_four_on_floor();
    test_classic_groove_breakbeat();

    // Harmony tests
    test_harmony_low_tension_steedman();
    test_harmony_medium_tension_parsimonious();
    test_harmony_high_tension_neo_riemannian();
    test_harmony_minor_key();
    test_harmony_major_key();

    // Key tests
    test_key_c_major();
    test_key_d_major();
    test_key_f_major();
    test_key_bb_major();
    test_key_e_minor();

    // Scenarios
    test_scenario_calm_ambient();
    test_scenario_energetic_dance();
    test_scenario_dark_tense();
    test_scenario_jazz_swing();

    // Odd meters
    test_odd_meter_3_4();
    test_odd_meter_5_4();
    test_odd_meter_7_8();

    // Transposing instrument tests
    test_tenor_sax_lead_export();
    test_alto_sax_lead_export();

    println!("\n========================================");
    println!("Done! Open files in MuseScore to review.");
    println!("========================================\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// TRANSPOSING INSTRUMENT TESTS (Timeline-based export)
// ═══════════════════════════════════════════════════════════════════════════

/// Generate measures using TimelineGenerator and export with instrument config
fn generate_timeline_export(
    name: &str,
    params: &MusicalParams,
    instrument_lead: &InstrumentConfig,
    instrument_bass: &InstrumentConfig,
    num_measures: usize,
    seed: u64,
) {
    use harmonium_core::{
        harmony::melody::HarmonyNavigator,
        params::CurrentState,
        timeline::{
            ScoreTimeline, TimelineGenerator, TrackId, timeline_to_musicxml_with_instruments,
        },
    };
    use rust_music_theory::{note::PitchSymbol, scale::ScaleType};

    setup_output_dir();

    let seq_primary = Sequencer::new(params.rhythm_steps, params.rhythm_pulses, params.bpm);
    let seq_secondary = Sequencer::new_with_rotation(12, 3, params.bpm, 0);
    let harmony =
        HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, params.melody_octave);
    let driver = HarmonicDriver::new(
        params.key_root,
        &harmonium_core::tuning::HarmonyDriverParams::default(),
    );
    let state = CurrentState {
        bpm: params.bpm,
        density: params.rhythm_density,
        tension: params.harmony_tension,
        smoothness: params.melody_smoothness,
        valence: params.harmony_valence,
        arousal: 0.5,
    };

    let mut tgen = TimelineGenerator::new(
        seq_primary,
        seq_secondary,
        harmony,
        Some(driver),
        params.clone(),
        state,
        harmonium_core::TuningParams::default(),
    );

    let mut rng = TestRng::new(seed);
    let mut timeline = ScoreTimeline::new(num_measures + 1);

    for bar in 1..=num_measures {
        let measure = tgen.generate_measure(bar, &mut rng);
        timeline.push_measure(measure);
    }

    // Validate lead notes are within instrument range
    let mut lead_count = 0;
    for measure in timeline.measures() {
        for note in measure.notes_for_track(TrackId::Melody) {
            assert!(
                note.pitch >= instrument_lead.min_note && note.pitch <= instrument_lead.max_note,
                "Lead note {} out of instrument range [{}, {}]",
                note.pitch,
                instrument_lead.min_note,
                instrument_lead.max_note,
            );
            lead_count += 1;
        }
    }

    let xml = timeline_to_musicxml_with_instruments(
        &timeline,
        &format!("Harmonium - {name}"),
        instrument_lead,
        instrument_bass,
    );

    let path = Path::new(OUTPUT_DIR).join(format!("{name}.musicxml"));
    std::fs::write(&path, &xml).unwrap_or_else(|_| panic!("Failed to write {name}"));

    println!(
        "Generated: {} ({} measures, {} lead notes, {} bytes)",
        path.display(),
        num_measures,
        lead_count,
        xml.len(),
    );

    // Print first few lines for quick inspection
    for line in xml.lines().take(30) {
        println!("  {line}");
    }
    println!("  ...");
}

#[test]
#[ignore]
fn test_tenor_sax_lead_export() {
    let tenor = InstrumentConfig::tenor_sax();
    let params = MusicalParams::default().instrument_lead(tenor);

    generate_timeline_export(
        "tenor_sax_lead",
        &params,
        &tenor,
        &InstrumentConfig::default(),
        8,
        42,
    );
}

#[test]
#[ignore]
fn test_alto_sax_lead_export() {
    let alto = InstrumentConfig::alto_sax();
    let params = MusicalParams::default().instrument_lead(alto);

    generate_timeline_export("alto_sax_lead", &params, &alto, &InstrumentConfig::default(), 8, 42);
}

// ═══════════════════════════════════════════════════════════════════════════
// HARMONIUM_LAB EXPORTS (Timeline-based MIDI + JSON + MusicXML)
// ═══════════════════════════════════════════════════════════════════════════

/// Generate measures using TimelineGenerator and export MIDI, JSON, and MusicXML
/// for consumption by harmonium_lab evaluation pipeline.
fn generate_lab_export(name: &str, params: &MusicalParams, num_measures: usize, seed: u64) {
    use harmonium_core::{
        harmony::melody::HarmonyNavigator,
        key_root_to_pitch_symbol,
        params::CurrentState,
        report::MeasureSnapshot,
        timeline::{ScoreTimeline, TimelineGenerator, timeline_to_musicxml, write_midi},
    };

    setup_output_dir();

    let seq_primary = Sequencer::new(params.rhythm_steps, params.rhythm_pulses, params.bpm);
    let seq_secondary = Sequencer::new_with_rotation(12, 3, params.bpm, 0);
    // CORELIB-22: Use correct key root and scale type from params
    let pitch_symbol = key_root_to_pitch_symbol(params.key_root);
    let is_minor = params.harmony_valence < 0.0;
    let scale_type = params.melody_scale_type.to_rmt_scale_type(is_minor);
    let harmony = HarmonyNavigator::new(pitch_symbol, scale_type, params.melody_octave);
    let driver = HarmonicDriver::new(
        params.key_root,
        &harmonium_core::tuning::HarmonyDriverParams::default(),
    );
    let state = CurrentState {
        bpm: params.bpm,
        density: params.rhythm_density,
        tension: params.harmony_tension,
        smoothness: params.melody_smoothness,
        valence: params.harmony_valence,
        arousal: 0.5,
    };

    let mut tgen = TimelineGenerator::new(
        seq_primary,
        seq_secondary,
        harmony,
        Some(driver),
        params.clone(),
        state.clone(),
        harmonium_core::TuningParams::default(),
    );

    let mut rng = TestRng::new(seed);
    let mut timeline = ScoreTimeline::new(num_measures + 1);

    for bar in 1..=num_measures {
        let measure = tgen.generate_measure(bar, &mut rng);
        timeline.push_measure(measure);
    }

    // 1. Export MusicXML
    let xml = timeline_to_musicxml(&timeline, &format!("Harmonium Lab - {name}"));
    let xml_path = Path::new(OUTPUT_DIR).join(format!("{name}.musicxml"));
    std::fs::write(&xml_path, &xml).unwrap_or_else(|_| panic!("Failed to write {name}.musicxml"));

    // 2. Export MIDI
    let midi_path = Path::new(OUTPUT_DIR).join(format!("{name}.mid"));
    write_midi(&timeline, &midi_path).unwrap_or_else(|_| panic!("Failed to write {name}.mid"));

    // 3. Export MeasureSnapshot JSON (same format as golden test files)
    let snapshots: Vec<MeasureSnapshot> = timeline
        .measures()
        .iter()
        .map(|m| {
            let mut snapshot = MeasureSnapshot::from_measure(m);
            snapshot.composition_bpm = state.bpm;
            snapshot
        })
        .collect();
    let json = serde_json::to_string_pretty(&snapshots)
        .unwrap_or_else(|_| panic!("Failed to serialize {name} measures"));
    let json_path = Path::new(OUTPUT_DIR).join(format!("{name}.json"));
    std::fs::write(&json_path, &json).unwrap_or_else(|_| panic!("Failed to write {name}.json"));

    // 4. Export scenario metadata JSON
    let scenario_meta = serde_json::json!({
        "scenario": name,
        "params": {
            "bpm": state.bpm,
            "density": state.density,
            "tension": state.tension,
            "smoothness": state.smoothness,
            "valence": state.valence,
            "arousal": state.arousal,
        },
        "bars": num_measures,
        "seed": seed,
        "key_root": params.key_root,
        "rhythm_mode": format!("{:?}", params.rhythm_mode),
        "time_signature": format!("{}/{}", params.time_signature.numerator, params.time_signature.denominator),
    });
    let meta_path = Path::new(OUTPUT_DIR).join(format!("{name}_scenario.json"));
    std::fs::write(&meta_path, serde_json::to_string_pretty(&scenario_meta).unwrap())
        .unwrap_or_else(|_| panic!("Failed to write {name}_scenario.json"));

    println!(
        "Lab export: {} → .musicxml + .mid + .json + _scenario.json ({} measures)",
        name, num_measures,
    );
}

/// Helper: configure a scenario and export with multiple seeds.
fn lab_scenario(base_name: &str, params: &MusicalParams, bars: usize, seeds: &[u64]) {
    for &seed in seeds {
        let name =
            if seeds.len() == 1 { base_name.to_string() } else { format!("{base_name}_s{seed}") };
        generate_lab_export(&name, params, bars, seed);
    }
}

/// Standard seeds for multi-seed scenarios
const SEEDS: [u64; 4] = [42, 123, 456, 789];
/// Bars per scenario
const BARS: usize = 32;

#[test]
#[ignore]
fn generate_all_lab_exports() {
    println!("\n========================================");
    println!("Generating harmonium_lab exports...");
    println!("Output: {OUTPUT_DIR}/");
    println!("========================================\n");

    // ─── AMBIENT SCENARIOS (low tension, high smoothness) ─────────

    // Calm ambient — C major, Euclidean sparse
    let mut p = MusicalParams::default();
    p.bpm = 80.0;
    p.rhythm_mode = RhythmMode::Euclidean;
    p.rhythm_steps = 16;
    p.rhythm_pulses = 3;
    p.rhythm_density = 0.25;
    p.harmony_tension = 0.15;
    p.harmony_valence = 0.3;
    p.melody_smoothness = 0.8;
    p.melody_scale_type = MelodyScaleType::Pentatonic; // Ambient = safe pentatonic
    p.key_root = 0;
    lab_scenario("lab_ambient_calm_c", &p, BARS, &SEEDS);

    // Ambient warm — Eb major, even sparser
    p.bpm = 72.0;
    p.rhythm_pulses = 2;
    p.rhythm_density = 0.15;
    p.harmony_tension = 0.1;
    p.harmony_valence = 0.5;
    p.melody_smoothness = 0.9;
    p.key_root = 3; // Eb
    lab_scenario("lab_ambient_warm_eb", &p, BARS, &SEEDS);

    // Ambient dark — A minor, slight tension
    p.bpm = 76.0;
    p.rhythm_pulses = 3;
    p.rhythm_density = 0.2;
    p.harmony_tension = 0.2;
    p.harmony_valence = -0.4;
    p.melody_smoothness = 0.85;
    p.key_root = 9; // A
    lab_scenario("lab_ambient_dark_am", &p, BARS, &SEEDS);

    // ─── JAZZ CALM SCENARIOS (ballads, slow standards) ────────────

    // Jazz ballad — F major, ClassicGroove
    let mut p = MusicalParams::default();
    p.bpm = 90.0;
    p.rhythm_mode = RhythmMode::ClassicGroove;
    p.rhythm_steps = 16;
    p.rhythm_density = 0.4;
    p.harmony_tension = 0.35;
    p.harmony_valence = 0.2;
    p.melody_smoothness = 0.6;
    p.harmony_measures_per_chord = 1;
    p.key_root = 5; // F
    lab_scenario("lab_jazz_ballad_f", &p, BARS, &SEEDS);

    // Jazz ballad — Bb major, slower
    p.bpm = 76.0;
    p.rhythm_density = 0.35;
    p.harmony_tension = 0.3;
    p.harmony_valence = 0.3;
    p.melody_smoothness = 0.65;
    p.key_root = 10; // Bb
    lab_scenario("lab_jazz_ballad_bb", &p, BARS, &SEEDS);

    // Jazz ballad — D minor
    p.bpm = 84.0;
    p.harmony_tension = 0.35;
    p.harmony_valence = -0.5;
    p.melody_smoothness = 0.55;
    p.key_root = 2; // D minor
    lab_scenario("lab_jazz_ballad_dm", &p, BARS, &SEEDS);

    // ─── JAZZ MEDIUM SCENARIOS (swing, mid-tempo) ─────────────────

    // Jazz medium — G major, Euclidean swing
    let mut p = MusicalParams::default();
    p.bpm = 140.0;
    p.rhythm_mode = RhythmMode::Euclidean;
    p.rhythm_steps = 16;
    p.rhythm_pulses = 5;
    p.rhythm_density = 0.55;
    p.harmony_tension = 0.5;
    p.harmony_valence = 0.5;
    p.melody_smoothness = 0.4;
    p.harmony_measures_per_chord = 1;
    p.key_root = 7; // G
    lab_scenario("lab_jazz_medium_g", &p, BARS, &SEEDS);

    // Jazz medium — C major, ClassicGroove
    p.bpm = 130.0;
    p.rhythm_mode = RhythmMode::ClassicGroove;
    p.rhythm_density = 0.5;
    p.harmony_tension = 0.45;
    p.harmony_valence = 0.4;
    p.melody_smoothness = 0.45;
    p.key_root = 0; // C
    lab_scenario("lab_jazz_medium_c", &p, BARS, &SEEDS);

    // Jazz uptempo — F major
    p.bpm = 160.0;
    p.rhythm_mode = RhythmMode::Euclidean;
    p.rhythm_pulses = 7;
    p.rhythm_density = 0.65;
    p.harmony_tension = 0.55;
    p.harmony_valence = 0.6;
    p.melody_smoothness = 0.35;
    p.key_root = 5; // F
    lab_scenario("lab_jazz_uptempo_f", &p, BARS, &SEEDS);

    // ─── PRACTICE / TRAINING SCENARIOS ────────────────────────────

    // Practice easy — C major, steady
    let mut p = MusicalParams::default();
    p.bpm = 100.0;
    p.rhythm_mode = RhythmMode::ClassicGroove;
    p.rhythm_steps = 16;
    p.rhythm_density = 0.35;
    p.harmony_tension = 0.25;
    p.harmony_valence = 0.4;
    p.melody_smoothness = 0.6;
    p.harmony_measures_per_chord = 2;
    p.key_root = 0; // C
    lab_scenario("lab_practice_easy_c", &p, BARS, &SEEDS);

    // Practice medium — G major
    p.bpm = 110.0;
    p.rhythm_density = 0.45;
    p.harmony_tension = 0.35;
    p.harmony_valence = 0.5;
    p.melody_smoothness = 0.5;
    p.harmony_measures_per_chord = 1;
    p.key_root = 7; // G
    lab_scenario("lab_practice_medium_g", &p, BARS, &SEEDS);

    // Practice blues — Bb, higher tension
    p.bpm = 105.0;
    p.rhythm_density = 0.5;
    p.harmony_tension = 0.4;
    p.harmony_valence = -0.1;
    p.melody_smoothness = 0.45;
    p.melody_scale_type = MelodyScaleType::Blues; // Blues scale for bluesy practice
    p.key_root = 10; // Bb
    lab_scenario("lab_practice_blues_bb", &p, BARS, &SEEDS);

    // ─── DRAMATIC / HIGH-TENSION SCENARIOS ────────────────────────

    // Dramatic — D minor, Neo-Riemannian territory
    let mut p = MusicalParams::default();
    p.bpm = 120.0;
    p.rhythm_mode = RhythmMode::PerfectBalance;
    p.rhythm_steps = 48;
    p.rhythm_density = 0.6;
    p.rhythm_tension = 0.7;
    p.harmony_tension = 0.7;
    p.harmony_valence = -0.3;
    p.melody_smoothness = 0.3;
    p.melody_scale_type = MelodyScaleType::HarmonicMinor; // Dramatic = harmonic minor
    p.key_root = 2; // D
    lab_scenario("lab_dramatic_high_dm", &p, BARS, &SEEDS);

    // Dramatic intense — E minor, very high tension
    p.bpm = 132.0;
    p.rhythm_density = 0.7;
    p.rhythm_tension = 0.8;
    p.harmony_tension = 0.85;
    p.harmony_valence = -0.6;
    p.melody_smoothness = 0.2;
    // melody_scale_type inherited from dramatic block (HarmonicMinor)
    p.key_root = 4; // E
    lab_scenario("lab_dramatic_intense_em", &p, BARS, &SEEDS);

    // Cinematic — Ab major, wide voicings
    p.bpm = 108.0;
    p.rhythm_density = 0.45;
    p.rhythm_tension = 0.5;
    p.harmony_tension = 0.6;
    p.harmony_valence = 0.1;
    p.melody_smoothness = 0.5;
    p.key_root = 8; // Ab
    lab_scenario("lab_cinematic_ab", &p, BARS, &SEEDS);

    // ─── ODD METER SCENARIOS ──────────────────────────────────────

    // Triple meter 3/4 — Eb major
    let mut p = MusicalParams::default();
    p.time_signature = harmonium_core::params::TimeSignature::new(3, 4);
    p.bpm = 120.0;
    p.rhythm_mode = RhythmMode::ClassicGroove;
    p.rhythm_steps = 12;
    p.rhythm_density = 0.45;
    p.harmony_tension = 0.3;
    p.harmony_valence = 0.5;
    p.melody_smoothness = 0.55;
    p.harmony_measures_per_chord = 2;
    p.key_root = 3; // Eb
    lab_scenario("lab_triple_eb", &p, BARS, &SEEDS);

    // 5/4 groove — G minor
    p.time_signature = harmonium_core::params::TimeSignature::new(5, 4);
    p.bpm = 110.0;
    p.rhythm_steps = 20;
    p.rhythm_density = 0.5;
    p.harmony_tension = 0.45;
    p.harmony_valence = -0.2;
    p.melody_smoothness = 0.4;
    p.harmony_measures_per_chord = 1;
    p.key_root = 7; // G
    lab_scenario("lab_5_4_gm", &p, BARS, &SEEDS);

    // ─── KEY VARIETY (same params, different keys) ────────────────

    let mut p = MusicalParams::default();
    p.bpm = 110.0;
    p.rhythm_mode = RhythmMode::ClassicGroove;
    p.rhythm_steps = 16;
    p.rhythm_density = 0.45;
    p.harmony_tension = 0.35;
    p.harmony_valence = 0.3;
    p.melody_smoothness = 0.5;
    p.harmony_measures_per_chord = 1;

    for (key_root, key_name) in
        [(0, "c"), (2, "d"), (4, "e"), (5, "f"), (7, "g"), (9, "a"), (10, "bb")]
    {
        p.key_root = key_root;
        generate_lab_export(&format!("lab_keys_{key_name}"), &p, BARS, 42);
    }

    // ─── RHYTHM MODE COMPARISON (same harmony, different rhythms) ─

    let mut p = MusicalParams::default();
    p.bpm = 120.0;
    p.rhythm_density = 0.5;
    p.harmony_tension = 0.4;
    p.harmony_valence = 0.3;
    p.melody_smoothness = 0.5;
    p.key_root = 0;

    p.rhythm_mode = RhythmMode::Euclidean;
    p.rhythm_steps = 16;
    p.rhythm_pulses = 5;
    generate_lab_export("lab_rhythm_euclidean", &p, BARS, 42);

    p.rhythm_mode = RhythmMode::ClassicGroove;
    p.rhythm_steps = 16;
    generate_lab_export("lab_rhythm_classic", &p, BARS, 42);

    p.rhythm_mode = RhythmMode::PerfectBalance;
    p.rhythm_steps = 48;
    generate_lab_export("lab_rhythm_perfect_balance", &p, BARS, 42);

    // ─── TENSION SWEEP (same base, tension 0.1 → 0.9) ────────────

    let mut p = MusicalParams::default();
    p.bpm = 110.0;
    p.rhythm_mode = RhythmMode::ClassicGroove;
    p.rhythm_steps = 16;
    p.rhythm_density = 0.45;
    p.harmony_valence = 0.2;
    p.melody_smoothness = 0.5;
    p.key_root = 0;

    for tension_pct in [10, 25, 40, 55, 70, 85] {
        p.harmony_tension = tension_pct as f32 / 100.0;
        p.rhythm_tension = tension_pct as f32 / 200.0; // half of harmony tension
        generate_lab_export(&format!("lab_tension_{tension_pct:02}"), &p, BARS, 42);
    }

    // Count total files
    let count = std::fs::read_dir(OUTPUT_DIR)
        .map(|d| {
            d.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "mid"))
                .filter(|e| e.file_name().to_string_lossy().starts_with("lab_"))
                .count()
        })
        .unwrap_or(0);

    println!("\n========================================");
    println!("Done! {count} lab MIDI files generated.");
    println!("========================================\n");
}

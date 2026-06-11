//! Regression tests for issue #26 — every chord-related artifact (chord_name,
//! ScaleGuidance, bass pitches, melody chord context) must agree on the SAME
//! absolute root in a non-C session. Before the fix, the bass played
//! `36 + root_offset` (C-rooted) while labels/guidance/lead were in session-key
//! space — in G, a measure labelled Em7 walked its bass on A.

use harmonium_core::{
    harmony::{HarmonicDriver, melody::HarmonyNavigator},
    params::{CurrentState, MusicalParams},
    sequencer::Sequencer,
    timeline::{TrackId, generator::TimelineGenerator},
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_music_theory::{note::PitchSymbol, scale::ScaleType};

const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

fn pc(p: u8) -> &'static str {
    NOTE_NAMES[(p % 12) as usize]
}

/// Parse the root pitch class out of a chord name like "Em7" / "F#m7b5" / "Bb7".
fn root_pc_of_name(name: &str) -> u8 {
    let bytes = name.as_bytes();
    let natural = match bytes[0] {
        b'C' => 0i32,
        b'D' => 2,
        b'E' => 4,
        b'F' => 5,
        b'G' => 7,
        b'A' => 9,
        b'B' => 11,
        _ => panic!("chord name without a note root: {name}"),
    };
    let accidental = if bytes.len() > 1 {
        match bytes[1] {
            b'#' => 1,
            b'b' => -1,
            _ => 0,
        }
    } else {
        0
    };
    (natural + accidental).rem_euclid(12) as u8
}

fn make_g_session() -> TimelineGenerator {
    let seq_primary = Sequencer::new(16, 4, 120.0);
    let seq_secondary = Sequencer::new_with_rotation(12, 3, 120.0, 0);
    // Consistent session keyed to G (pc 7): navigator + driver + key_root
    let harmony = HarmonyNavigator::new(PitchSymbol::G, ScaleType::PentatonicMajor, 4);
    let driver = HarmonicDriver::new(7, &harmonium_core::tuning::HarmonyDriverParams::default());
    let mut params = MusicalParams::default();
    params.key_root = 7;
    // density/smoothness high enough for walking bass every bar
    let state = CurrentState {
        bpm: 120.0,
        density: 0.6,
        tension: 0.3,
        smoothness: 0.7,
        valence: 0.3,
        arousal: 0.5,
    };

    TimelineGenerator::new(
        seq_primary,
        seq_secondary,
        harmony,
        Some(driver),
        params,
        state,
        TuningParams::default(),
    )
}

#[test]
fn chord_artifacts_agree_in_g_session_driver_mode() {
    let mut tgen = make_g_session();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let mut checked_bars = 0;
    for bar in 1..=16 {
        let m = tgen.generate_measure(bar, &mut rng);
        let ctx = &m.chord_context;
        let guidance = ctx.scale_guidance.as_ref().expect("scale guidance present");

        // Initial placeholder "I" only allowed before the first chord decision
        if ctx.chord_name == "I" {
            continue;
        }

        let label_root = root_pc_of_name(&ctx.chord_name);
        let sounding_root = ((7 + ctx.root_offset).rem_euclid(12)) as u8;

        // 1. Label matches the sounding root (the original #26 complaint)
        assert_eq!(
            label_root,
            sounding_root,
            "bar {bar}: chord_name {} (root {}) != sounding root {}",
            ctx.chord_name,
            pc(label_root),
            pc(sounding_root)
        );

        // 2. Guidance agrees with the label (chord symbol + root)
        assert_eq!(
            guidance.chord_root % 12,
            sounding_root,
            "bar {bar}: guidance root {} != sounding root {}",
            pc(guidance.chord_root),
            pc(sounding_root)
        );
        assert_eq!(
            root_pc_of_name(&guidance.chord_symbol),
            sounding_root,
            "bar {bar}: guidance symbol {} != sounding root {}",
            guidance.chord_symbol,
            pc(sounding_root)
        );

        // 3. The bass actually plays in the chord: every bass note's pitch
        // class must be a chord tone, a scale-adjacent colour tone, or a
        // chromatic approach (the walking-bass palette is root/3rd/5th/6th/
        // 4th/approach ±1 relative to the SOUNDING root).
        let allowed: Vec<u8> = [0i32, 3, 4, 5, 7, 9, 1, 11, 2, 10]
            .iter()
            .map(|i| ((i32::from(sounding_root) + i).rem_euclid(12)) as u8)
            .collect();
        for n in m.notes_for_track(TrackId::Bass) {
            let bass_pc = n.pitch % 12;
            assert!(
                allowed.contains(&bass_pc),
                "bar {bar} ({}): bass note {} not in the chord palette rooted on {}",
                ctx.chord_name,
                pc(bass_pc),
                pc(sounding_root)
            );
        }

        // 4. On beat 1 the walking bass states the root (or its lower fifth /
        // octave — same pitch class family as the root, never the old
        // C-rooted offset). Beat-1 candidates: root, root-5 (fifth below),
        // root+12 — pitch classes root or root+7.
        if let Some(first) = m.notes_for_track(TrackId::Bass).iter().find(|n| n.start_step == 0) {
            let first_pc = first.pitch % 12;
            let fifth_below = (i32::from(sounding_root) + 7).rem_euclid(12) as u8;
            assert!(
                first_pc == sounding_root || first_pc == fifth_below,
                "bar {bar} ({}): beat-1 bass {} is neither root {} nor fifth {}",
                ctx.chord_name,
                pc(first_pc),
                pc(sounding_root),
                pc(fifth_below)
            );
        }

        // 5. Melody chord context is in absolute space: its first pitch class
        // is the sounding root.
        assert_eq!(
            tgen.harmony.current_chord_notes[0],
            sounding_root,
            "bar {bar}: melody chord-context root {} != sounding root {}",
            pc(tgen.harmony.current_chord_notes[0]),
            pc(sounding_root)
        );

        checked_bars += 1;
    }
    assert!(checked_bars >= 8, "expected most bars to carry a real chord, got {checked_bars}");
}

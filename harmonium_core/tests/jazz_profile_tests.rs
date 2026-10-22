//! Fixture synthétique, pas un style distribué ni un substitut de corpus jazz.
use harmonium_core::{
    StyleProfile, TuningParams,
    params::MusicalParams,
    report::MeasureSnapshot,
    theory::chord::{Chord, ChordType},
    trad::TradGenerator,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn load() -> Result<StyleProfile, Box<dyn std::error::Error>> {
    Ok(StyleProfile::load(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tests/fixtures/profiles/jazz_test"
    ))?)
}
fn stream(seed: u64) -> Result<Vec<MeasureSnapshot>, Box<dyn std::error::Error>> {
    let p = load()?;
    let mut params = MusicalParams::default();
    params.bpm = p.meter.default_bpm;
    params.swing_ratio = p.meter.default_swing_ratio;
    let mut g = TradGenerator::new(params, TuningParams { profile: p, ..Default::default() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=64)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect())
}
#[test]
fn synthetic_jazz_profile_publishes_aaba_swing_ii_v_i_and_euclidean_percussion()
-> Result<(), Box<dyn std::error::Error>> {
    for seed in 1..=10 {
        let snapshots = stream(seed)?;
        assert_eq!(
            serde_json::to_value(&snapshots)?,
            serde_json::to_value(stream(seed)?)?,
            "seed {seed}"
        );
        for tour in snapshots.chunks_exact(32) {
            let progression: Vec<_> =
                tour.iter().map(|s| Chord::from_name(&s.chord_name)).collect::<Result<_, _>>()?;
            let key = tour[0].position.unwrap().key;
            assert!(
                progression.windows(3).any(|w| w[0].root == (key + 2) % 12
                    && w[0].chord_type == ChordType::Minor7
                    && w[1].root == (key + 7) % 12
                    && w[1].chord_type == ChordType::Dominant7
                    && w[2].root == key),
                "ii-V-I seed {seed}"
            );
            let positions: Vec<_> = tour.iter().map(|s| s.position.unwrap()).collect();
            assert_eq!(
                positions.iter().step_by(8).map(|p| (p.part.0, p.pass)).collect::<Vec<_>>(),
                [(0, 1), (0, 2), (1, 1), (0, 3)]
            );
            for bar in 0..8 {
                assert_eq!(
                    serde_json::to_value(&tour[bar].notes)?,
                    serde_json::to_value(&tour[bar + 8].notes)?
                );
                assert_eq!(
                    serde_json::to_value(&tour[bar].notes)?,
                    serde_json::to_value(&tour[bar + 24].notes)?
                );
                assert_eq!(tour[bar].chord_name, tour[bar + 24].chord_name);
            }
        }
        for snapshot in &snapshots {
            assert_eq!(
                (snapshot.time_sig_numerator, snapshot.time_sig_denominator, snapshot.steps),
                (4, 4, 16)
            );
            assert_eq!((snapshot.pulse.steps_per_pulse, snapshot.pulse.pulses_per_bar), (4, 4));
            assert_eq!(snapshot.tempo, 120.0);
            assert!((snapshot.swing_ratio - 2.0 / 3.0).abs() < 1e-6);
            let scale = snapshot.scale_guidance.as_ref().unwrap();
            let chord = Chord::from_name(&snapshot.chord_name)?;
            assert_eq!(scale.chord_tones, chord.pitch_classes());
            assert_eq!(scale.chord_root, chord.root);
            for note in snapshot.notes.iter().filter(|n| n.track == 0) {
                assert!(
                    scale.chord_tones.contains(&(note.pitch % 12))
                        || scale.scale_notes.contains(&(note.pitch % 12)),
                    "seed {seed}, bar {}, pitch {}",
                    snapshot.index,
                    note.pitch
                );
            }
            let percussion: Vec<_> = snapshot
                .notes
                .iter()
                .filter(|n| n.track == 3)
                .map(|n| (n.pitch, n.start_step, n.duration_steps))
                .collect();
            assert_eq!(percussion, [(51, 1, 0), (51, 5, 0), (51, 9, 0), (51, 13, 0)]);
        }
    }
    Ok(())
}

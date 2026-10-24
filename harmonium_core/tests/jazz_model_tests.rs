//! Rythme harmonique, sélection des règles et réalisation mélodique sur le contrat public.
use harmonium_core::{
    StyleProfile, TuningParams,
    models::jazz::{RewriteRule, RomanNumeral, RuleCategory},
    params::MusicalParams,
    report::MeasureSnapshot,
    style_profile::KeyModeWeight,
    theory::chord::Chord,
    trad::{IntroConfig, Mode, Part, ReelArrangement, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn rule(lhs: RomanNumeral, rhs: Vec<RomanNumeral>, weight: f32) -> RewriteRule {
    RewriteRule {
        lhs,
        rhs,
        weight,
        min_valence: -1.0,
        max_valence: 1.0,
        category: RuleCategory::Cadential,
        max_recursion: 0,
    }
}
fn profile(period: usize, guidance: bool) -> StyleProfile {
    use RomanNumeral::{I, II, V};
    let mut p = StyleProfile::default();
    p.ornaments = Default::default();
    p.harmony.model = "jazz_functional".into();
    p.harmony.steps_per_chord = period;
    p.harmony.rules = vec![rule(I, vec![II], 1.0), rule(II, vec![V], 1.0), rule(V, vec![I], 1.0)];
    p.harmony.candidates.clear();
    p.harmony.scale_guidance = guidance;
    p.melody.key_modes = vec![KeyModeWeight { key: 7, mode: Mode::Ionian, weight: 1 }];
    if let Some(a) = &mut p.accompaniment {
        a.bass.low = 36;
        a.bass.high = 59;
    }
    p.arrangement = ReelArrangement::from_start(1, Part(0));
    p.arrangement.intro = IntroConfig::none();
    p
}
fn stream(p: StyleProfile, seed: u64) -> Result<Vec<MeasureSnapshot>, Box<dyn std::error::Error>> {
    let p = StyleProfile::from_toml(&p.to_toml()?)?;
    let mut g = TradGenerator::new(
        MusicalParams::default(),
        TuningParams { profile: p, ..Default::default() },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=32)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect())
}
#[test]
fn jazz_harmonic_slots_publish_correct_absolute_chords_and_melody()
-> Result<(), Box<dyn std::error::Error>> {
    for period in [4, 8, 16] {
        for guidance in [false, true] {
            let snapshots = stream(profile(period, guidance), 42)?;
            let other = stream(profile(period, guidance), 42)?;
            assert_eq!(serde_json::to_value(&snapshots)?, serde_json::to_value(other)?);
            let first_roots: Vec<_> = snapshots
                .iter()
                .take(3)
                .flat_map(|s| &s.chord_changes)
                .take(3)
                .map(|c| Chord::from_name(&c.context.chord_name).unwrap().root)
                .collect();
            assert_eq!(first_roots, [9, 2, 7]); // ii-V-I en G, pas en C.
            for snapshot in snapshots {
                assert_eq!(
                    snapshot.chord_changes.iter().map(|c| c.start_step).collect::<Vec<_>>(),
                    (0..16).step_by(period).collect::<Vec<_>>()
                );
                assert_eq!(snapshot.chord_name, snapshot.chord_changes[0].context.chord_name);
                for change in &snapshot.chord_changes {
                    let chord = Chord::from_name(&change.context.chord_name)?;
                    assert_eq!(
                        change.context.root_offset,
                        (i32::from(chord.root) - 7).rem_euclid(12)
                    );
                    assert_eq!(change.context.scale_guidance.is_some(), guidance);
                    if let Some(scale) = &change.context.scale_guidance {
                        assert_eq!(scale.chord_root, chord.root);
                        assert_eq!(scale.chord_tones, chord.pitch_classes());
                    }
                }
                for note in snapshot.notes.iter().filter(|n| n.track <= 2) {
                    let change = &snapshot.chord_changes[note.start_step / period];
                    let chord = Chord::from_name(&change.context.chord_name)?;
                    let pc = note.pitch % 12;
                    assert!(
                        chord.pitch_classes().contains(&pc)
                            || (note.track == 0
                                && change
                                    .context
                                    .scale_guidance
                                    .as_ref()
                                    .is_some_and(|s| s.scale_notes.contains(&pc)))
                    );
                    assert!(note.start_step + note.duration_steps <= change.start_step + period);
                }
            }
        }
    }
    Ok(())
}
#[test]
fn configured_rules_and_zero_weights_control_audible_chords()
-> Result<(), Box<dyn std::error::Error>> {
    use RomanNumeral::{I, II, IV};
    for (selected, root) in [(II, 9), (IV, 0)] {
        let mut p = profile(16, false);
        p.harmony.rules = vec![
            rule(I, vec![II], f32::from(selected == II)),
            rule(I, vec![IV], f32::from(selected == IV)),
        ];
        let snapshots = stream(p, 1)?;
        assert_eq!(Chord::from_name(&snapshots[0].chord_name)?.root, root);
    }
    let mut p = profile(16, false);
    p.harmony.rules[0].rhs.clear();
    assert!(p.validate().is_err());
    p = profile(16, false);
    p.harmony.rules.iter_mut().for_each(|r| r.weight = 0.0);
    assert!(p.validate().is_err());
    p = profile(16, false);
    p.harmony.rules[0].weight = f32::NAN;
    assert!(p.validate().is_err());
    p = profile(16, false);
    p.harmony.rules = vec![rule(I, vec![II], 1.5e38), rule(I, vec![IV], 1.5e38)];
    for rule in &mut p.harmony.rules {
        rule.category = RuleCategory::Preparation;
    }
    assert_eq!(p.validate().unwrap_err().field, "harmony.rules");
    Ok(())
}

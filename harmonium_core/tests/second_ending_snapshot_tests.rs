//! Voltas : une seule mesure différente, résolution mesurée, priorité de l'outro.
use harmonium_core::{
    StyleProfile, TuningParams,
    params::MusicalParams,
    report::MeasureSnapshot,
    style_profile::{ProfileRole, WeightedRole},
    trad::{ImposedRole, ReelArrangement, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn stream(mut profile: StyleProfile, seed: u64, bars: usize, outro: bool) -> Vec<MeasureSnapshot> {
    if let Some(a) = &mut profile.accompaniment {
        a.bass.passing = None;
        a.chords.strum = None;
        a.chords.strings = 3;
    }
    let mut params = MusicalParams::default();
    params.bpm = profile.meter.default_bpm;
    params.end_after_tour = outro;
    let mut generator = TradGenerator::new(params, TuningParams { profile });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect()
}
fn notes(snapshot: &MeasureSnapshot) -> Vec<(u8, u8, usize, usize, u8)> {
    snapshot
        .notes
        .iter()
        .map(|n| (n.track, n.pitch, n.start_step, n.duration_steps, n.velocity))
        .collect()
}
fn melody(snapshot: &MeasureSnapshot) -> Vec<(u8, usize, usize, u8)> {
    snapshot
        .notes
        .iter()
        .filter(|n| n.track == 0)
        .map(|n| (n.pitch, n.start_step, n.duration_steps, n.velocity))
        .collect()
}
fn end_category(snapshot: &MeasureSnapshot) -> usize {
    let last = snapshot.notes.iter().filter(|n| n.track == 0).max_by_key(|n| n.start_step).unwrap();
    let position = snapshot.position.unwrap();
    assert!(position.mode.contains_midi(position.key, last.pitch));
    match (last.pitch + 12 - position.key) % 12 {
        0 => 0,
        7 => 1,
        _ => 2,
    }
}
fn ending_weights(role: &ProfileRole) -> [f64; 3] {
    let ProfileRole::Choice { choose } = role else { panic!("distribution de fin requise") };
    let mut result = [0.0; 3];
    for choice in choose {
        let category = match choice.role.fixed().unwrap() {
            ImposedRole::CadTonic => 0,
            ImposedRole::CadFifth => 1,
            ImposedRole::CadOther => 2,
            role => panic!("classe finale inattendue : {role:?}"),
        };
        result[category] += f64::from(choice.weight);
    }
    let total: f64 = result.iter().sum();
    result.map(|weight| weight / total)
}
#[test]
fn distributed_final_endings_follow_weights_and_preserve_other_bars_over_200_tunes() {
    for distributed in StyleProfile::distributed() {
        let mut profile = distributed.clone();
        profile.performance = Default::default();
        profile.arrangement = ReelArrangement::from_start(2, Default::default());
        let mut counts = [[0usize; 3]; 2];
        for seed in 0..200 {
            let snapshots = stream(profile.clone(), seed, 64, false);
            for (part, start) in [0usize, 16].into_iter().enumerate() {
                counts[part][end_category(&snapshots[start + 15])] += 1;
                for tour in [0usize, 32] {
                    for offset in 0..7 {
                        let first = &snapshots[tour + start + offset];
                        let second = &snapshots[tour + start + offset + 8];
                        assert_eq!(
                            notes(first),
                            notes(second),
                            "{} seed {seed} part {part} bar {}",
                            profile.meta.id,
                            offset + 1
                        );
                        assert_eq!(
                            (&first.chord_name, first.chord_root_offset, first.chord_is_minor),
                            (&second.chord_name, second.chord_root_offset, second.chord_is_minor)
                        );
                    }
                }
                assert_eq!(notes(&snapshots[start + 15]), notes(&snapshots[start + 47]));
            }
            if seed < 3 {
                assert_eq!(
                    serde_json::to_value(&snapshots).unwrap(),
                    serde_json::to_value(stream(profile.clone(), seed, 64, false)).unwrap()
                );
            }
        }
        for part in 0..2 {
            let expected = ending_weights(profile.form.parts[part].second_ending.as_ref().unwrap());
            for category in 0..3 {
                let actual = counts[part][category] as f64 / 200.0;
                assert!(
                    (actual - expected[category]).abs() <= 0.10,
                    "{} part {part}, class {category}: {actual:.3} hors {:.3} ±0.10",
                    profile.meta.id,
                    expected[category]
                );
            }
            println!(
                "{} {}2 : {:?}/200 ; poids {:?}",
                profile.meta.id, profile.form.parts[part].name, counts[part], expected
            );
        }
    }
}
fn forced_profile() -> StyleProfile {
    let mut profile = StyleProfile::default();
    profile.performance = Default::default();
    profile.arrangement = ReelArrangement::from_start(1, Default::default());
    for part in &mut profile.form.parts {
        part.roles = vec![ImposedRole::Free.into(); 8];
        part.roles[7] = ImposedRole::CadOther.into();
        part.second_ending = Some(ImposedRole::CadTonic.into());
    }
    profile.validate().unwrap();
    profile
}
#[test]
fn terminal_pass_selects_the_alternative_even_in_an_interleaved_three_pass_form() {
    let mut profile = forced_profile();
    profile.form.sequence = vec![0, 1, 0, 1, 0];
    profile.validate().unwrap();
    let snapshots = stream(profile, 71, 40, false);
    for bar in [7usize, 15, 23] {
        assert_eq!(end_category(&snapshots[bar]), 2);
    }
    for bar in [31usize, 39] {
        assert_eq!(end_category(&snapshots[bar]), 0);
    }
    for bar in 0..7 {
        assert_eq!(notes(&snapshots[bar]), notes(&snapshots[bar + 16]));
        assert_eq!(notes(&snapshots[bar]), notes(&snapshots[bar + 32]));
    }
}
#[test]
fn fixed_second_ending_is_a_real_motif_operation_not_a_cadence_override() {
    let mut profile = forced_profile();
    for part in &mut profile.form.parts {
        part.second_ending = Some(ImposedRole::CopyOf(1).into());
    }
    profile.validate().unwrap();
    let snapshots = stream(profile, 83, 32, false);
    assert_eq!(melody(&snapshots[0]), melody(&snapshots[15]));
    assert_eq!(melody(&snapshots[16]), melody(&snapshots[31]));
}
#[test]
fn end_after_tour_has_priority_over_the_configured_non_tonic_second_ending() {
    let mut profile = forced_profile();
    for part in &mut profile.form.parts {
        part.second_ending = Some(ImposedRole::CadOther.into());
    }
    let normal = stream(profile.clone(), 17, 32, false);
    let final_tour = stream(profile, 17, 32, true);
    assert_eq!(end_category(&normal[31]), 2);
    assert_eq!(end_category(&final_tour[15]), 2);
    assert_eq!(end_category(&final_tour[31]), 0);
    for bar in 0..31 {
        assert_eq!(notes(&normal[bar]), notes(&final_tour[bar]));
    }
    assert_eq!(final_tour[31].chord_root_offset, 0);
}
#[test]
fn invalid_or_unreachable_second_endings_are_rejected_before_generation() {
    let invalid = [
        ImposedRole::CopyOf(8).into(),
        ProfileRole::Choice {
            choose: vec![WeightedRole { role: ImposedRole::CadTonic.into(), weight: 0 }],
        },
    ];
    for role in invalid {
        let mut profile = forced_profile();
        profile.form.parts[0].second_ending = Some(role);
        assert!(profile.validate().unwrap_err().field.starts_with("form.parts[0].second_ending"));
    }
    let mut profile = forced_profile();
    profile.form.sequence = vec![0, 1, 1];
    assert_eq!(profile.validate().unwrap_err().field, "form.parts[0].second_ending");
}
#[test]
fn second_ending_profile_round_trip_keeps_the_entire_snapshot_stream() {
    let profile = forced_profile();
    let decoded = StyleProfile::from_toml(&profile.to_toml().unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(stream(profile, 97, 64, false)).unwrap(),
        serde_json::to_value(stream(decoded, 97, 64, false)).unwrap()
    );
}

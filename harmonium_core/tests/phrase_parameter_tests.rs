//! Paramètres de structure : effets audibles et frontières sur le contrat public.
use harmonium_core::{
    StyleProfile, TuningParams,
    ornaments::{OrnamentKind, OrnamentRule},
    params::MusicalParams,
    report::MeasureSnapshot,
    style_profile::{
        PositionRhythm, ProfileRole, RhythmPattern, RoleResolution, VariationOperation,
        WeightedRole,
    },
    trad::{
        Cell, CellBank, CellRole, ImposedRole, ReelArrangement, TradGenerator, dynamics::Curve,
    },
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type Melody = Vec<(u8, usize, usize, u8)>;
fn melody(s: &MeasureSnapshot) -> Melody {
    s.notes
        .iter()
        .filter(|n| n.track == 0)
        .map(|n| (n.pitch, n.start_step, n.duration_steps, n.velocity))
        .collect()
}
fn stream(p: &StyleProfile, seed: u64, bars: usize) -> Vec<MeasureSnapshot> {
    p.validate().unwrap();
    let mut g = TradGenerator::new(
        MusicalParams { bpm: p.meter.default_bpm, ..Default::default() },
        TuningParams { profile: p.clone() },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars).map(|b| MeasureSnapshot::from_measure(&g.generate_measure(b, &mut rng))).collect()
}
fn fixture() -> StyleProfile {
    let mut p = StyleProfile::default();
    p.performance = Default::default();
    p.ornaments = Default::default();
    p.arrangement = ReelArrangement::from_start(2, Default::default());
    p.form.parts.truncate(1);
    p.form.sequence = vec![0, 0];
    p.form.parts[0].second_ending = None;
    p.form.parts[0].rhythm.clear();
    p.form.parts[0].roles = vec![ImposedRole::Open.into(); 8];
    p.form.parts[0].roles[3] = ImposedRole::CadHalf.into();
    p.form.parts[0].roles[7] = ImposedRole::CadTonic.into();
    p.melody.cells = CellBank {
        cells: [
            (CellRole::Open, "1 1 2 2 3 3 4 4"),
            (CellRole::Run, "2 3 4 5 6 7 5 6"),
            (CellRole::Arp, "1 3 5 3 2 4 6 4"),
            (CellRole::Turn, "3 4 3 2 3 4 3 2"),
            (CellRole::CadHalf, "1 2 3 4 5 - 5 -"),
            (CellRole::CadFull, "5 4 3 2 1 - - -"),
        ]
        .into_iter()
        .map(|(r, n)| Cell::from_notation(r, n).unwrap())
        .collect(),
    };
    p
}
fn ending(s: &MeasureSnapshot) -> (u8, usize) {
    let n = s.notes.iter().filter(|n| n.track == 0).max_by_key(|n| n.start_step).unwrap();
    (n.pitch % 12, n.duration_steps)
}
#[test]
fn repeat_variation_operations_preserve_cadences_and_tour_identity() {
    for operation in [
        VariationOperation::VariedEnd,
        VariationOperation::RepeatedNotes,
        VariationOperation::Octave,
    ] {
        let mut p = fixture();
        let fixed = stream(&p, 7, 32);
        for b in 0..8 {
            assert_eq!(melody(&fixed[b]), melody(&fixed[b + 8]));
        }
        p.form.repeat_variation = 1.0;
        p.form.variation_operations = vec![operation];
        let varied = stream(&p, 7, 32);
        assert!((0..8).any(|b| melody(&varied[b]) != melody(&varied[b + 8])), "{operation:?}");
        for b in [3, 7] {
            assert_eq!(ending(&varied[b]), ending(&varied[b + 8]), "{operation:?}");
        }
        for b in 0..16 {
            assert_eq!(melody(&varied[b]), melody(&varied[b + 16]));
        }
        assert_eq!(
            serde_json::to_value(&varied).unwrap(),
            serde_json::to_value(stream(&p, 7, 32)).unwrap()
        );
    }
}
#[test]
fn tour_variation_does_not_vary_first_tour_or_repeat_pass() {
    let mut p = fixture();
    let fixed = stream(&p, 11, 32);
    p.form.tour_variation = 1.0;
    p.form.variation_operations = vec![VariationOperation::Octave];
    let varied = stream(&p, 11, 32);
    for b in 0..16 {
        assert_eq!(melody(&fixed[b]), melody(&varied[b]));
    }
    assert!((0..16).any(|b| melody(&varied[b]) != melody(&varied[b + 16])));
    for b in [3, 7, 11, 15] {
        assert_eq!(ending(&varied[b]), ending(&varied[b + 16]));
    }
}
#[test]
fn first_measure_is_weighted_and_per_pass_resolution_changes_reprises() {
    let mut p = fixture();
    p.form.parts[0].roles[0] = ProfileRole::Choice {
        choose: vec![
            WeightedRole { role: ImposedRole::Open.into(), weight: 1 },
            WeightedRole { role: ImposedRole::CadTonic.into(), weight: 1 },
        ],
    };
    let mut tonic_openings = 0;
    let mut changed = 0;
    for seed in 0..100 {
        let fixed = stream(&p, seed, 16);
        assert_eq!(melody(&fixed[0]), melody(&fixed[8]));
        tonic_openings += usize::from(ending(&fixed[0]).0 == fixed[0].position.unwrap().key);
        let mut per_pass = p.clone();
        per_pass.form.resolve = RoleResolution::PerPass;
        let played = stream(&per_pass, seed, 32);
        changed += usize::from(ending(&played[0]) != ending(&played[8]));
        assert_eq!(
            serde_json::to_value(&played).unwrap(),
            serde_json::to_value(stream(&per_pass, seed, 32)).unwrap()
        );
    }
    assert!((35..65).contains(&tonic_openings), "{tonic_openings}");
    assert!((30..70).contains(&changed), "{changed}");
}
#[test]
fn rhythm_is_positional_preserves_copy_contract_and_final_degree() {
    let mut p = fixture();
    p.form.parts[0].roles[0] = ImposedRole::CadOpen.into();
    p.form.parts[0].roles[1] = ImposedRole::CopyOf(1).into();
    let fixed = stream(&p, 13, 8);
    p.form.parts[0].rhythm = vec![
        PositionRhythm {
            bar: 1,
            patterns: vec![RhythmPattern { durations: vec![2, 2, 4], weight: 1 }],
        },
        PositionRhythm { bar: 2, patterns: vec![RhythmPattern { durations: vec![8], weight: 1 }] },
    ];
    let played = stream(&p, 13, 8);
    assert_eq!(
        melody(&played[0]).iter().map(|n| (n.1, n.2)).collect::<Vec<_>>(),
        vec![(0, 4), (4, 4), (8, 8)]
    );
    assert_eq!(ending(&played[0]).0, ending(&fixed[0]).0);
    assert_eq!(melody(&played[0]), melody(&played[1]));
}
#[test]
fn varied_end_keep_controls_exactly_the_preserved_prefix() {
    for keep in [0.25, 0.75] {
        let mut p = fixture();
        p.form.varied_end_keep = keep;
        p.form.parts[0].roles[1] = ImposedRole::VariedEndOf(1).into();
        let s = stream(&p, 3, 2);
        let boundary = (keep * 16.0) as usize;
        let a = melody(&s[0]);
        let b = melody(&s[1]);
        assert_eq!(
            a.iter().filter(|n| n.1 < boundary).collect::<Vec<_>>(),
            b.iter().filter(|n| n.1 < boundary).collect::<Vec<_>>()
        );
        assert_ne!(
            a.iter().filter(|n| n.1 >= boundary).collect::<Vec<_>>(),
            b.iter().filter(|n| n.1 >= boundary).collect::<Vec<_>>()
        );
    }
}
#[test]
fn numeric_structure_controls_reach_the_snapshot_stream() {
    let controls: &[(&str, fn(&mut StyleProfile))] = &[
        ("avoid_adjacent", |p| p.form.avoid_adjacent = false),
        ("opening_retries", |p| p.form.opening_retries = 0),
        ("contour_window_semitones", |p| p.form.contour_window_semitones = 0.0),
        ("register_tolerance_semitones", |p| p.form.register_tolerance_semitones = 0.0),
        ("max_leap_degrees", |p| p.form.max_leap_degrees = 0),
        ("close_part_loop", |p| p.form.close_part_loop = false),
        ("link_previous", |p| p.form.parts[1].link_previous = false),
    ];
    for &(name, change) in controls {
        let mut p = StyleProfile::default();
        p.performance = Default::default();
        p.arrangement = ReelArrangement::from_start(1, Default::default());
        let mut modified = p.clone();
        change(&mut modified);
        assert!(
            (0..80).any(|seed| stream(&p, seed, 65)
                .iter()
                .zip(stream(&modified, seed, 65))
                .any(|(a, b)| melody(a) != melody(&b))),
            "{name} sans effet"
        );
    }
}
#[test]
fn phrase_grouping_controls_percussion_boundaries_and_arbitrary_lengths() {
    let mut p = StyleProfile::named("marche_britannique_coloniale").unwrap();
    p.performance = Default::default();
    p.arrangement = ReelArrangement::from_start(1, Default::default());
    p.form.parts.truncate(1);
    p.form.sequence = vec![0, 0];
    p.form.parts[0].roles.truncate(6);
    p.form.parts[0].second_ending = None;
    p.form.parts[0].rhythm.clear();
    p.form.parts[0].phrase_ends = vec![3, 6];
    let s = stream(&p, 19, 12);
    for (i, s) in s.iter().enumerate() {
        assert_eq!(s.position.unwrap().bar_in_strain, i % 6 + 1);
        let flam = s
            .notes
            .iter()
            .any(|n| n.pitch == 40 && n.track == harmonium_core::timeline::TrackId::Toe.channel());
        assert_eq!(flam, [2, 5].contains(&(i % 6)), "mesure {}", i + 1);
    }
}
#[test]
fn invalid_structure_values_name_the_field() {
    let bad: &[(&str, fn(&mut StyleProfile))] = &[
        ("form.repeat_variation", |p| p.form.repeat_variation = -0.1),
        ("form.tour_variation", |p| p.form.tour_variation = 1.1),
        ("form.repeat_variation", |p| p.form.repeat_variation = f32::NAN),
        ("form.variation_operations", |p| p.form.variation_operations.clear()),
        ("form.opening_retries", |p| p.form.opening_retries = 10001),
        ("form.contour_window_semitones", |p| p.form.contour_window_semitones = -1.0),
        ("form.register_tolerance_semitones", |p| {
            p.form.register_tolerance_semitones = f32::INFINITY
        }),
        ("form.max_leap_degrees", |p| p.form.max_leap_degrees = -1),
        ("form.varied_end_keep", |p| p.form.varied_end_keep = 1.0),
        ("form.cadence_min_units", |p| p.form.cadence_min_units = 0),
        ("form.cadence_search_from", |p| p.form.cadence_search_from = 1.1),
        ("form.parts[0].phrase_ends", |p| p.form.parts[0].phrase_ends = vec![9]),
        ("form.parts[0].rhythm[0]", |p| {
            p.form.parts[0].rhythm[0].patterns[0].durations = vec![0, 8]
        }),
    ];
    for &(field, mutate) in bad {
        let mut p = StyleProfile::default();
        p.performance = Default::default();
        mutate(&mut p);
        let error = p.validate().unwrap_err();
        assert_eq!(error.field, field);
    }
    let text = fixture().to_toml().unwrap().replace("per_tune", "each_note");
    assert!(StyleProfile::from_toml(&text).is_err());
}

#[test]
fn density_ornament_protection_and_minimum_duration_are_profile_rules() {
    let mut p = fixture();
    let passing = |min: f32| OrnamentRule {
        min_carrier_eighths: min,
        density_curve: Some(Curve(vec![[0.0, 1.0], [1.0, 1.0]])),
        ..Default::default()
    };
    p.ornaments.types.insert(OrnamentKind::Passing, passing(1.5));
    let protected = stream(&p, 7, 8);
    p.form.parts[0].ornament_protected_bars = Some(vec![]);
    let unprotected = stream(&p, 7, 8);
    assert_ne!(ending(&protected[7]).0, ending(&unprotected[7]).0);
    p.ornaments.types.insert(OrnamentKind::Passing, passing(64.0));
    let disabled = stream(&p, 7, 8);
    assert_eq!(ending(&protected[7]).0, ending(&disabled[7]).0);
    assert!(melody(&protected[3]).iter().any(|n| n.2 == 1));
    assert!(melody(&disabled[3]).iter().all(|n| n.2 >= 2));
    p.ornaments.types.insert(OrnamentKind::Passing, passing(65.0));
    assert_eq!(p.validate().unwrap_err().field, "ornaments.types.passing.min_carrier_eighths");
    p.ornaments.types.insert(OrnamentKind::Passing, passing(1.5));
    p.form.parts[0].ornament_protected_bars = Some(vec![0]);
    assert_eq!(p.validate().unwrap_err().field, "form.parts[0].ornament_protected_bars");
}

#[test]
fn combined_parameters_round_trip_preserves_performed_snapshots() {
    let mut p = fixture();
    p.form.resolve = RoleResolution::PerPass;
    p.form.repeat_variation = 0.4;
    p.form.tour_variation = 0.6;
    p.form.varied_end_keep = 0.25;
    p.form.parts[0].link_previous = false;
    p.form.parts[0].phrase_ends = vec![3, 8];
    p.form.parts[0].ornament_protected_bars = Some(vec![3, 8]);
    let decoded = StyleProfile::from_toml(&p.to_toml().unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(stream(&p, 71, 64)).unwrap(),
        serde_json::to_value(stream(&decoded, 71, 64)).unwrap()
    );
}

#[test]
fn rhythm_pattern_weights_reach_snapshot_attack_positions() {
    let mut p = fixture();
    p.form.parts[0].roles[0] = ImposedRole::CadOpen.into();
    p.form.parts[0].rhythm = vec![PositionRhythm {
        bar: 1,
        patterns: vec![
            RhythmPattern { durations: vec![2, 2, 4], weight: 3 },
            RhythmPattern { durations: vec![4, 4], weight: 1 },
        ],
    }];
    let count = (1..=200).filter(|&seed| melody(&stream(&p, seed, 1)[0]).len() == 3).count();
    assert!((count as f64 / 200.0 - 0.75).abs() < 0.10, "{count}/200");
}

#[test]
fn distributed_rhythmic_entropy_recovers_the_pre_phrase_form() {
    fn entropy(p: &StyleProfile) -> f64 {
        let mut counts = std::collections::BTreeMap::new();
        for seed in 1..=200 {
            let snapshots = stream(p, seed, 32);
            for index in (0..8).chain(16..24) {
                let onsets: Vec<_> = melody(&snapshots[index]).iter().map(|n| n.1).collect();
                *counts.entry(onsets).or_insert(0usize) += 1;
            }
        }
        counts
            .values()
            .map(|&n| {
                let probability = n as f64 / 3200.0;
                -probability * probability.log2()
            })
            .sum()
    }
    for profile in StyleProfile::distributed() {
        let mut current = profile.clone();
        current.arrangement = ReelArrangement::from_start(1, Default::default());
        current.ornaments = Default::default();
        let mut before = current.clone();
        for part in &mut before.form.parts {
            part.rhythm.clear();
            part.second_ending = None;
            part.roles = [
                ImposedRole::Open,
                ImposedRole::Free,
                ImposedRole::Free,
                ImposedRole::CadHalf,
                ImposedRole::CopyOf(1),
                ImposedRole::CopyOf(2),
                ImposedRole::Free,
                ImposedRole::CadFull,
            ]
            .map(Into::into)
            .to_vec();
        }
        let baseline = entropy(&before);
        let actual = entropy(&current);
        assert!(actual >= baseline, "{} : {actual:.4} < {baseline:.4}", profile.meta.id);
    }
}

#[test]
fn cadence_hold_and_search_window_control_the_published_rest() {
    let mut p = fixture();
    let original = stream(&p, 31, 8);
    p.form.cadence_min_units = 6;
    let long = stream(&p, 31, 8);
    assert!(ending(&long[7]).1 >= 12);
    assert!(ending(&long[7]).1 > ending(&original[7]).1);
    assert_eq!(ending(&long[7]).0, ending(&original[7]).0);
    p.form.cadence_min_units = 2;
    p.form.cadence_search_from = 0.9;
    let late = stream(&p, 31, 8);
    assert_eq!(ending(&late[7]).1, 4);
    assert!(ending(&late[7]).1 < ending(&original[7]).1);
    assert_eq!(ending(&late[7]).0, ending(&original[7]).0);
}

#[test]
fn cadence_chord_qualities_come_from_profile_candidates() {
    use harmonium_core::trad::{ChordDegree, ChordQuality, Mode};
    let mut p = fixture();
    p.melody.key_modes = vec![harmonium_core::style_profile::KeyModeWeight {
        key: 2,
        mode: Mode::Dorian,
        weight: 1,
    }];
    let original = stream(&p, 11, 8);
    for entry in &mut p.harmony.candidates {
        if entry.mode == Mode::Dorian {
            for chord in &mut entry.chords {
                if chord.degree == ChordDegree::V {
                    chord.quality = ChordQuality::Major;
                }
            }
        }
    }
    let changed = stream(&p, 11, 8);
    assert_eq!(original[7].chord_name, "Am");
    assert_eq!(changed[7].chord_name, "A");
    assert_eq!(melody(&original[7]), melody(&changed[7]));
}

#[test]
fn odd_cell_default_varied_end_keeps_the_historical_floor_half() {
    use harmonium_core::trad::CellNote;
    let mut p = fixture();
    p.melody.cell_units = 7;
    p.meter.steps_per_bar = 14;
    p.meter.steps_per_pulse = 7;
    p.harmony.steps_per_chord = 7;
    p.melody.weak_units.retain(|&unit| unit < 7);
    for pattern in &mut p.percussion.patterns {
        pattern.hits.retain(|hit| hit.step < 14);
    }
    for pattern in &mut p.accompaniment.as_mut().unwrap().chords.patterns {
        pattern.hits.retain(|hit| hit.step < 14);
    }
    for cell in &mut p.melody.cells.cells {
        cell.notes.truncate(7);
        if cell.role != CellRole::Open {
            cell.notes[3] = CellNote::Degree(5);
        }
    }
    p.form.parts[0].roles[1] = ImposedRole::VariedEndOf(1).into();
    let snapshots = stream(&p, 17, 2);
    let source = melody(&snapshots[0]);
    let varied = melody(&snapshots[1]);
    assert_eq!(&source[..3], &varied[..3]);
    assert_ne!(source[3].0 % 12, varied[3].0 % 12);
}

#[test]
fn varied_reprise_preserves_full_cadence_before_its_pickup() {
    let mut p = fixture();
    for cell in &mut p.melody.cells.cells {
        if cell.role == CellRole::CadFull {
            *cell = Cell::from_notation(CellRole::CadFull, "5 4 3 2 1 - 5, -").unwrap();
        }
    }
    p.form.parts[0].roles[7] = ImposedRole::CadFull.into();
    p.form.repeat_variation = 1.0;
    p.form.variation_operations = vec![VariationOperation::VariedEnd];
    for seed in 0..20 {
        let snapshots = stream(&p, seed, 16);
        let cadence =
            |s: &MeasureSnapshot| melody(s).into_iter().filter(|n| n.1 >= 8).collect::<Vec<_>>();
        assert_eq!(cadence(&snapshots[7]), cadence(&snapshots[15]));
    }
}

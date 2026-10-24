//! Couche phrase : probabilités et opérations observées sur le flux public.
use harmonium_core::{
    StyleProfile, TuningParams,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    style_profile::{
        DegreeShift, KeyModeWeight, PartReference, ProfileRole, SequenceRole, WeightedRole,
    },
    trad::{Cell, CellBank, CellRole, ImposedRole, Mode, ReelArrangement, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type Note = (u8, usize, usize, u8);
fn melody(snapshot: &MeasureSnapshot) -> Vec<Note> {
    snapshot
        .notes
        .iter()
        .filter(|n| n.track == 0)
        .map(|n| (n.pitch, n.start_step, n.duration_steps, n.velocity))
        .collect()
}
fn choice(roles: &[(ImposedRole, u32)]) -> ProfileRole {
    ProfileRole::Choice {
        choose: roles
            .iter()
            .map(|&(role, weight)| WeightedRole { role: role.into(), weight })
            .collect(),
    }
}
fn fixture() -> StyleProfile {
    let mut p = StyleProfile::default();
    p.performance = Default::default();
    p.ornaments = Default::default();
    p.arrangement = ReelArrangement::from_start(2, Default::default());
    p.set = Default::default();
    p.melody.key_modes = vec![KeyModeWeight { key: 2, mode: Mode::Dorian, weight: 1 }];
    p.melody.cells = CellBank {
        cells: [
            (CellRole::Open, "1 2 3 4 5 6 7 8"),
            (CellRole::Run, "2 3 4 5 6 7 5 6"),
            (CellRole::Arp, "1 3 5 3 2 4 6 4"),
            (CellRole::Turn, "3 4 3 2 3 4 3 2"),
            (CellRole::CadHalf, "1 2 3 4 5 - 5 -"),
            (CellRole::CadFull, "5 4 3 2 1 - - -"),
        ]
        .into_iter()
        .map(|(role, notes)| Cell::from_notation(role, notes).unwrap())
        .collect(),
    };
    for part in &mut p.form.parts {
        part.second_ending = None;
        part.roles = [
            ImposedRole::Open,
            ImposedRole::Free,
            ImposedRole::Free,
            ImposedRole::Free,
            ImposedRole::Free,
            ImposedRole::Free,
            ImposedRole::Free,
            ImposedRole::CadOpen,
        ]
        .map(Into::into)
        .to_vec();
    }
    p.form.parts[0].roles[1] = choice(&[(ImposedRole::CopyOf(1), 53), (ImposedRole::Free, 47)]);
    p.form.parts[0].roles[2] = ProfileRole::Sequence {
        sequence: SequenceRole {
            bar: 1,
            shift: vec![
                DegreeShift { degrees: -1, weight: 25 },
                DegreeShift { degrees: 2, weight: 75 },
            ],
        },
    };
    p.form.parts[0].roles[3] = ImposedRole::VariedEndOf(1).into();
    p.form.parts[0].roles[7] = choice(&[
        (ImposedRole::CadTonic, 22),
        (ImposedRole::CadHalf, 28),
        (ImposedRole::CadOpen, 50),
    ]);
    p.form.parts[1].roles[6] =
        ProfileRole::CrossPart { source: PartReference { part: "A".into(), bar: 7 } };
    p.validate().unwrap();
    p
}
fn stream(profile: StyleProfile, seed: u64, bars: usize) -> Vec<MeasureSnapshot> {
    let mut params = MusicalParams::default();
    params.bpm = profile.meter.default_bpm;
    let mut generator = TradGenerator::new(params, TuningParams { profile });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect()
}
fn last_melody(snapshot: &MeasureSnapshot) -> &NoteSnapshot {
    snapshot.notes.iter().filter(|n| n.track == 0).max_by_key(|n| n.start_step).unwrap()
}
fn diatonic_index(mode: Mode, key: u8, pitch: u8) -> i16 {
    (i16::from(pitch) - i16::from(key)).div_euclid(12) * 7
        + i16::from(mode.scale_degree_of_midi(key, pitch).unwrap())
        - 1
}
fn near(count: usize, expected: f64, label: &str) {
    let rate = count as f64 / 200.0;
    assert!((rate - expected).abs() <= 0.10, "{label}: {rate:.3} hors {expected:.3} ±0.10");
}
#[test]
fn weighted_operations_cadences_repeats_and_determinism_on_200_tunes() {
    let profile = fixture();
    let mut copies = 0;
    let mut down = 0;
    let mut cadences = [0usize; 3];
    for seed in 0..200 {
        let snapshots = stream(profile.clone(), seed, 64);
        assert_eq!(
            serde_json::to_value(&snapshots).unwrap(),
            serde_json::to_value(stream(profile.clone(), seed, 64)).unwrap()
        );
        copies += usize::from(melody(&snapshots[0]) == melody(&snapshots[1]));
        let source = melody(&snapshots[0]);
        let sequence = melody(&snapshots[2]);
        assert_eq!(
            source.iter().map(|n| (n.1, n.2, n.3)).collect::<Vec<_>>(),
            sequence.iter().map(|n| (n.1, n.2, n.3)).collect::<Vec<_>>()
        );
        let shifts: Vec<_> = source
            .iter()
            .zip(&sequence)
            .map(|(a, b)| {
                diatonic_index(Mode::Dorian, 2, b.0) - diatonic_index(Mode::Dorian, 2, a.0)
            })
            .collect();
        assert!(shifts.iter().all(|s| *s == shifts[0]), "contour non transposé: {shifts:?}");
        assert!(matches!(shifts[0].rem_euclid(7), 6 | 2));
        down += usize::from(shifts[0].rem_euclid(7) == 6);
        let varied = melody(&snapshots[3]);
        assert_eq!(
            source.iter().filter(|n| n.1 < 8).collect::<Vec<_>>(),
            varied.iter().filter(|n| n.1 < 8).collect::<Vec<_>>()
        );
        assert_ne!(
            source.iter().filter(|n| n.1 >= 8).collect::<Vec<_>>(),
            varied.iter().filter(|n| n.1 >= 8).collect::<Vec<_>>()
        );
        assert_eq!(melody(&snapshots[6]), melody(&snapshots[22]));
        for bar in 0..32 {
            assert_eq!(melody(&snapshots[bar]), melody(&snapshots[bar + 32]));
        }
        for start in [0usize, 16, 32, 48] {
            for bar in 0..8 {
                assert_eq!(melody(&snapshots[start + bar]), melody(&snapshots[start + bar + 8]));
            }
        }
        for snapshot in &snapshots {
            for note in snapshot.notes.iter().filter(|n| n.track == 0) {
                assert!((55..=88).contains(&note.pitch));
                assert!(Mode::Dorian.contains_midi(2, note.pitch));
            }
        }
        let last = last_melody(&snapshots[7]);
        match Mode::Dorian.scale_degree_of_midi(2, last.pitch).unwrap() {
            1 => {
                assert!(last.duration_steps >= 4);
                cadences[0] += 1;
            }
            5 => cadences[1] += 1,
            _ => cadences[2] += 1,
        }
    }
    near(copies, 0.53, "CopyOf");
    near(down, 0.25, "SequenceOf -1");
    for (count, expected) in cadences.into_iter().zip([0.22, 0.28, 0.50]) {
        near(count, expected, "cadence");
    }
}
#[test]
fn distributed_copy_and_tonic_weights_reach_snapshots_on_200_tunes() {
    for profile in StyleProfile::distributed() {
        let mut profile = profile.clone();
        profile.performance = Default::default();
        profile.arrangement = ReelArrangement::from_start(1, Default::default());
        let mut copies = [0usize; 2];
        let mut tonics = [0usize; 2];
        for seed in 0..200 {
            let snapshots = stream(profile.clone(), seed, 32);
            for (part, start) in [0usize, 16].into_iter().enumerate() {
                copies[part] +=
                    usize::from(melody(&snapshots[start]) == melody(&snapshots[start + 4]));
                let last = last_melody(&snapshots[start + 7]);
                let position = snapshots[start + 7].position.unwrap();
                tonics[part] += usize::from(
                    position.mode.scale_degree_of_midi(position.key, last.pitch) == Some(1)
                        && last.duration_steps >= 4,
                );
            }
        }
        for part in 0..2 {
            for (bar, target, count) in [
                (4, ImposedRole::CopyOf(1), copies[part]),
                (7, ImposedRole::CadTonic, tonics[part]),
            ] {
                let ProfileRole::Choice { choose } = &profile.form.parts[part].roles[bar] else {
                    panic!("rôle probabiliste requis")
                };
                let total: u32 = choose.iter().map(|c| c.weight).sum();
                let weight: u32 = choose
                    .iter()
                    .filter(|c| c.role.fixed() == Some(target))
                    .map(|c| c.weight)
                    .sum();
                near(
                    count,
                    f64::from(weight) / f64::from(total),
                    &format!("{}:{}:{target:?}", profile.meta.id, part),
                );
            }
        }
    }
}
#[test]
fn invalid_choices_and_forward_references_are_rejected() {
    let bad: Vec<ProfileRole> = vec![
        choice(&[(ImposedRole::Free, 0)]),
        choice(&[(ImposedRole::Free, u32::MAX), (ImposedRole::Open, 1)]),
        choice(&[(ImposedRole::CopyOf(3), 1), (ImposedRole::Free, 1)]),
        ImposedRole::VariedEndOf(2).into(),
        ProfileRole::Sequence {
            sequence: SequenceRole { bar: 3, shift: vec![DegreeShift { degrees: 1, weight: 1 }] },
        },
        ProfileRole::Sequence { sequence: SequenceRole { bar: 1, shift: vec![] } },
        ProfileRole::Sequence {
            sequence: SequenceRole { bar: 1, shift: vec![DegreeShift { degrees: 15, weight: 1 }] },
        },
        ProfileRole::Sequence {
            sequence: SequenceRole { bar: 1, shift: vec![DegreeShift { degrees: 1, weight: 0 }] },
        },
        ProfileRole::Choice {
            choose: vec![WeightedRole { role: choice(&[(ImposedRole::Free, 1)]), weight: 1 }],
        },
        ProfileRole::CrossPart { source: PartReference { part: "B".into(), bar: 1 } },
        ProfileRole::CrossPart { source: PartReference { part: "A".into(), bar: 1 } },
    ];
    for role in bad {
        let mut p = fixture();
        p.form.parts[0].roles[1] = role;
        assert!(p.validate().unwrap_err().field.starts_with("form.parts[0].roles[1]"));
    }
    let mut p = fixture();
    p.form.parts[1].roles[6] =
        ProfileRole::CrossPart { source: PartReference { part: "A".into(), bar: 9 } };
    assert_eq!(p.validate().unwrap_err().field, "form.parts[1].roles[6]");
    let mut p = fixture();
    p.form.sequence = vec![1, 1, 0, 0];
    assert_eq!(p.validate().unwrap_err().field, "form.parts[1].roles[6]");
    let mut p = fixture();
    for cell in &mut p.melody.cells.cells {
        cell.notes[4..].fill(harmonium_core::trad::CellNote::Hold);
    }
    assert_eq!(p.validate().unwrap_err().field, "melody.cells");
}
#[test]
fn weighted_profile_round_trip_preserves_snapshot_contract() {
    let p = fixture();
    let decoded = StyleProfile::from_toml(&p.to_toml().unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(stream(p, 89, 64)).unwrap(),
        serde_json::to_value(stream(decoded, 89, 64)).unwrap()
    );
}

#[test]
fn sequences_keep_diatonic_contour_across_modes_keys_and_octaves() {
    for mode in [Mode::Ionian, Mode::Mixolydian, Mode::Dorian, Mode::Aeolian] {
        for key in 0..12 {
            for degrees in [-14, -6, 0, 6, 14] {
                let mut profile = fixture();
                profile.form.parts.truncate(1);
                profile.form.sequence = vec![0, 0];
                profile.melody.params.tessitura_min = 65;
                profile.melody.params.tessitura_max = 89;
                profile.melody.key_modes = vec![KeyModeWeight { key, mode, weight: 1 }];
                profile.form.parts[0].roles[2] = ProfileRole::Sequence {
                    sequence: SequenceRole {
                        bar: 1,
                        shift: vec![DegreeShift { degrees, weight: 1 }],
                    },
                };
                profile.validate().unwrap();
                let snapshots = stream(profile, 42, 3);
                let source = melody(&snapshots[0]);
                let shifted = melody(&snapshots[2]);
                let actual: Vec<_> = source
                    .iter()
                    .zip(&shifted)
                    .map(|(a, b)| {
                        assert!((65..=89).contains(&b.0));
                        diatonic_index(mode, key, b.0) - diatonic_index(mode, key, a.0)
                    })
                    .collect();
                assert_eq!(source.len(), shifted.len());
                assert!(
                    actual.iter().all(|&shift| shift == actual[0]),
                    "{mode:?} {key} {degrees}: {actual:?}"
                );
                assert_eq!(actual[0].rem_euclid(7), i16::from(degrees).rem_euclid(7));
            }
        }
    }
}

#[test]
fn new_tunes_redraw_roles_even_when_key_and_mode_stay_the_same() {
    let mut same_color = 0usize;
    let mut changed = 0usize;
    for seed in 0..200 {
        let snapshots = stream(fixture(), seed, 128);
        let first = snapshots[0].position.unwrap();
        let next = snapshots[64].position.unwrap();
        if (first.key, first.mode) == (next.key, next.mode) {
            same_color += 1;
            let copied_first = melody(&snapshots[0]) == melody(&snapshots[1]);
            let copied_next = melody(&snapshots[64]) == melody(&snapshots[65]);
            changed += usize::from(copied_first != copied_next);
        }
    }
    let rate = changed as f64 / same_color as f64;
    assert!(
        (rate - 2.0 * 0.53 * 0.47).abs() <= 0.15,
        "{changed}/{same_color}: les choix doivent être retirés au prochain air"
    );
}

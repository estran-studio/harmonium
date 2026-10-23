//! Frontières du profil observées sur le contrat public de snapshots.
use harmonium_core::{
    StyleProfile,
    params::{MusicalParams, TimeSignature},
    report::MeasureSnapshot,
    trad::{Cell, CellBank, CellRole, FootHit, ImposedRole, Part, ReelArrangement, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn variable_profile() -> Result<StyleProfile, Box<dyn std::error::Error>> {
    let mut p = StyleProfile::default();
    p.performance = Default::default();
    for part in &mut p.form.parts {
        part.second_ending = None;
    }
    p.meta.id = "fixture".into();
    p.meter.time_signature = TimeSignature::new(6, 8);
    p.meter.steps_per_bar = 12;
    p.meter.steps_per_unit = 2;
    p.meter.steps_per_pulse = 6;
    p.meter.pulses_per_bar = 2;
    p.melody.cell_units = 6;
    p.melody.strong_units = vec![0, 3];
    p.melody.weak_units = vec![1, 2, 4, 5];
    p.melody.cells = CellBank {
        cells: [
            (CellRole::Open, "1 3 5 3 1 3"),
            (CellRole::Run, "1 2 3 4 5 6"),
            (CellRole::Arp, "1 3 5 3 1 3"),
            (CellRole::Turn, "3 4 3 2 3 4"),
            (CellRole::CadHalf, "1 2 3 4 5 -"),
            (CellRole::CadFull, "5 4 3 2 1 -"),
        ]
        .into_iter()
        .map(|(role, notes)| Cell::from_notation(role, notes))
        .collect::<Result<_, _>>()?,
    };
    for part in &mut p.form.parts {
        part.rhythm.clear();
        part.phrase_ends.clear();
    }
    let mut third = p.form.parts[0].clone();
    third.name = "C".into();
    third.roles = [ImposedRole::Open, ImposedRole::CadFull].map(Into::into).to_vec();
    p.form.parts[0].roles =
        [ImposedRole::Open, ImposedRole::Free, ImposedRole::CadFull].map(Into::into).to_vec();
    p.form.parts[1].roles =
        [ImposedRole::Open, ImposedRole::Free, ImposedRole::CopyOf(1), ImposedRole::CadFull]
            .map(Into::into)
            .to_vec();
    p.form.parts.push(third);
    p.form.sequence = vec![2, 0, 1, 0, 2];
    p.harmony.steps_per_chord = 12;
    if let Some(a) = &mut p.accompaniment {
        a.bass_steps = vec![0, 6];
        a.chop_steps = vec![3, 9];
        a.strong_steps = vec![0, 6];
    }
    for pattern in &mut p.percussion.patterns {
        pattern.hits = vec![FootHit::heel(0), FootHit::toe(3), FootHit::heel(6), FootHit::toe(9)];
    }
    p.dynamics.chop_patterns = vec![vec![3], vec![3, 9], vec![0, 3, 6, 9]];
    p.arrangement = ReelArrangement::from_start(1, Part(p.form.sequence[0]));
    p.validate()?;
    Ok(p)
}

fn notes(snapshot: &MeasureSnapshot) -> Vec<(u8, u8, usize, usize, u8)> {
    snapshot
        .notes
        .iter()
        .map(|n| (n.track, n.pitch, n.start_step, n.duration_steps, n.velocity))
        .collect()
}

#[test]
fn variable_grid_units_parts_sequence_copies_and_harmony_render()
-> Result<(), Box<dyn std::error::Error>> {
    let profile = variable_profile()?;
    let tuning = harmonium_core::TuningParams { profile, ..Default::default() };
    let mut generator = TradGenerator::new(MusicalParams::default(), tuning);
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let snapshots: Vec<_> = (1..=14)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect();
    for snapshot in &snapshots {
        assert_eq!(
            (snapshot.time_sig_numerator, snapshot.time_sig_denominator, snapshot.steps),
            (6, 8, 12)
        );
        assert_eq!((snapshot.pulse.steps_per_pulse, snapshot.pulse.pulses_per_bar), (6, 2));
        assert!(
            snapshot
                .notes
                .iter()
                .all(|n| n.start_step < 12 && n.start_step + n.duration_steps <= 12)
        );
        let bass: Vec<_> =
            snapshot.notes.iter().filter(|n| n.track == 2).map(|n| n.start_step).collect();
        assert_eq!(bass, [0, 6]);
    }
    assert_eq!(snapshots[0].position.unwrap().part, Part(2));
    assert_eq!(snapshots[2].position.unwrap().part, Part(0));
    assert_eq!(snapshots[5].position.unwrap().part, Part(1));
    assert_eq!(notes(&snapshots[0]), notes(&snapshots[12]));
    assert_eq!(notes(&snapshots[2]), notes(&snapshots[9]));
    let melody = |bar: usize| {
        snapshots[bar]
            .notes
            .iter()
            .filter(|n| n.track == 0)
            .map(|n| (n.pitch, n.start_step, n.duration_steps, n.velocity))
            .collect::<Vec<_>>()
    };
    assert_eq!(melody(5), melody(7));
    Ok(())
}

#[test]
fn cross_section_bounds_reject_before_generation() -> Result<(), Box<dyn std::error::Error>> {
    let mut p = variable_profile()?;
    p.percussion.patterns[0].hits[0].step = 12;
    assert_eq!(p.validate().unwrap_err().field, "percussion.patterns[0].hits");
    p = variable_profile()?;
    p.form.parts[0].roles[0] = ImposedRole::CopyOf(1).into();
    assert_eq!(p.validate().unwrap_err().field, "form.parts[0].roles[0]");
    p = variable_profile()?;
    p.melody.cells.cells[0].notes.pop();
    assert_eq!(p.validate().unwrap_err().field, "melody.cells[0].notes");
    p = variable_profile()?;
    p.arrangement.entries.melody.pass = 3;
    assert_eq!(p.validate().unwrap_err().field, "arrangement.entries.Melody.pass");
    Ok(())
}

#[test]
fn compound_meter_distributed_profile_contract() -> Result<(), Box<dyn std::error::Error>> {
    let profile = &StyleProfile::named("gigue_quebecoise")?;
    assert_eq!(profile.meter.time_signature, TimeSignature::new(6, 8));
    assert_eq!(&StyleProfile::named("gigue")?, profile);
    assert_eq!(&StyleProfile::named("jig")?, profile);

    assert_eq!(profile.meter.steps_per_bar, 12);
    assert_eq!(profile.meter.steps_per_unit, 2);
    assert_eq!(profile.meter.steps_per_pulse, 6);
    assert_eq!(profile.meter.pulses_per_bar, 2);
    assert_eq!(profile.meter.default_swing_ratio, 0.5);

    assert_eq!(profile.form.sequence, vec![0, 0, 1, 1]);
    assert_eq!(profile.form.parts.len(), 2);
    assert_eq!(profile.form.parts[0].roles.len(), 8);
    assert_eq!(profile.form.parts[1].roles.len(), 8);
    assert_eq!(profile.form.bars_per_tour(), 32);

    let tuning = harmonium_core::TuningParams { profile: profile.clone(), ..Default::default() };
    let mut params = MusicalParams::default();
    params.bpm = profile.meter.default_bpm;
    params.enable_melody = true;
    params.enable_harmony = true;
    params.enable_rhythm = true;

    let mut gen1 = TradGenerator::new(params.clone(), tuning.clone());
    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let snapshots1: Vec<_> = (1..=64)
        .map(|bar| MeasureSnapshot::from_measure(&gen1.generate_measure(bar, &mut rng1)))
        .collect();

    let mut gen2 = TradGenerator::new(params, tuning);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);
    let snapshots2: Vec<_> = (1..=64)
        .map(|bar| MeasureSnapshot::from_measure(&gen2.generate_measure(bar, &mut rng2)))
        .collect();

    for (s1, s2) in snapshots1.iter().zip(&snapshots2) {
        assert_eq!(notes(s1), notes(s2));
    }

    for snap in &snapshots1 {
        assert_eq!((snap.time_sig_numerator, snap.time_sig_denominator), (6, 8));
        assert_eq!(snap.steps, 12);
        assert_eq!((snap.pulse.steps_per_pulse, snap.pulse.pulses_per_bar), (6, 2));
        assert!(!snap.chord_name.is_empty());

        if let Some(pos) = snap.position {
            for note in snap.notes.iter().filter(|n| n.track == 0) {
                assert!(
                    pos.mode.contains_midi(pos.key, note.pitch),
                    "note {} hors du mode {:?} en tonalité {}",
                    note.pitch,
                    pos.mode,
                    pos.key
                );
            }
        }

        let bass_steps: Vec<_> =
            snap.notes.iter().filter(|n| n.track == 2).map(|n| n.start_step).collect();
        if !bass_steps.is_empty() {
            assert!(
                bass_steps.iter().all(|&step| step == 0 || step == 6),
                "pas basse inattendus: {:?}",
                bass_steps
            );
        }

        let chord_steps: Vec<_> =
            snap.notes.iter().filter(|n| n.track == 1).map(|n| n.start_step).collect();
        if !chord_steps.is_empty() {
            assert!(
                chord_steps.iter().all(|&step| step == 4 || step == 10),
                "pas accords inattendus: {:?}",
                chord_steps
            );
        }
    }

    let tour_bars: Vec<_> =
        snapshots1.iter().filter_map(|s| s.position).filter(|p| !p.is_intro()).collect();
    assert!(!tour_bars.is_empty());
    assert_eq!(tour_bars[0].part, Part(0));
    assert_eq!(tour_bars[0].pass, 1);
    assert_eq!(tour_bars[7].bar_in_strain, 8);
    assert_eq!(tour_bars[8].part, Part(0));
    assert_eq!(tour_bars[8].pass, 2);
    assert_eq!(tour_bars[16].part, Part(1));
    assert_eq!(tour_bars[16].pass, 1);
    assert_eq!(tour_bars[24].part, Part(1));
    assert_eq!(tour_bars[24].pass, 2);

    Ok(())
}

#[test]
fn triple_meter_distributed_profile_contract() -> Result<(), Box<dyn std::error::Error>> {
    let profile = &StyleProfile::named("valse_quebecoise")?;
    assert_eq!(profile.meter.time_signature, TimeSignature::new(3, 4));
    assert_eq!(&StyleProfile::named("valse")?, profile);
    assert_eq!(&StyleProfile::named("waltz")?, profile);

    assert_eq!(profile.meter.steps_per_bar, 12);
    assert_eq!(profile.meter.steps_per_unit, 2);
    assert_eq!(profile.meter.steps_per_pulse, 4);
    assert_eq!(profile.meter.pulses_per_bar, 3);
    assert_eq!(profile.meter.default_swing_ratio, 0.5);

    assert_eq!(profile.form.sequence, vec![0, 0, 1, 1]);
    assert_eq!(profile.form.parts.len(), 2);
    assert_eq!(profile.form.parts[0].roles.len(), 8);
    assert_eq!(profile.form.parts[1].roles.len(), 8);
    assert_eq!(profile.form.bars_per_tour(), 32);

    let tuning = harmonium_core::TuningParams { profile: profile.clone(), ..Default::default() };
    let mut params = MusicalParams::default();
    params.bpm = profile.meter.default_bpm;
    params.enable_melody = true;
    params.enable_harmony = true;
    params.enable_rhythm = true;

    let mut gen1 = TradGenerator::new(params.clone(), tuning.clone());
    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let snapshots1: Vec<_> = (1..=64)
        .map(|bar| MeasureSnapshot::from_measure(&gen1.generate_measure(bar, &mut rng1)))
        .collect();

    let mut gen2 = TradGenerator::new(params, tuning);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);
    let snapshots2: Vec<_> = (1..=64)
        .map(|bar| MeasureSnapshot::from_measure(&gen2.generate_measure(bar, &mut rng2)))
        .collect();

    for (s1, s2) in snapshots1.iter().zip(&snapshots2) {
        assert_eq!(notes(s1), notes(s2));
    }

    for snap in &snapshots1 {
        assert_eq!((snap.time_sig_numerator, snap.time_sig_denominator), (3, 4));
        assert_eq!(snap.steps, 12);
        assert_eq!((snap.pulse.steps_per_pulse, snap.pulse.pulses_per_bar), (4, 3));
        assert!(!snap.chord_name.is_empty());

        if let Some(pos) = snap.position {
            for note in snap.notes.iter().filter(|n| n.track == 0) {
                assert!(
                    pos.mode.contains_midi(pos.key, note.pitch),
                    "note {} hors du mode {:?} en tonalité {}",
                    note.pitch,
                    pos.mode,
                    pos.key
                );
            }
        }

        // Accompagnement oom-pah-pah : basse au pas 0, accords aux pas 4 et 8
        let bass_steps: Vec<_> =
            snap.notes.iter().filter(|n| n.track == 2).map(|n| n.start_step).collect();
        if !bass_steps.is_empty() {
            assert!(
                bass_steps.iter().all(|&step| step == 0),
                "pas basse inattendus: {:?}",
                bass_steps
            );
        }

        let chord_steps: Vec<_> =
            snap.notes.iter().filter(|n| n.track == 1).map(|n| n.start_step).collect();
        if !chord_steps.is_empty() {
            assert!(
                chord_steps.iter().all(|&step| step == 4 || step == 8),
                "pas accords inattendus: {:?}",
                chord_steps
            );
        }
    }

    let tour_bars: Vec<_> =
        snapshots1.iter().filter_map(|s| s.position).filter(|p| !p.is_intro()).collect();
    assert!(!tour_bars.is_empty());
    assert_eq!(tour_bars[0].part, Part(0));
    assert_eq!(tour_bars[0].pass, 1);
    assert_eq!(tour_bars[7].bar_in_strain, 8);
    assert_eq!(tour_bars[8].part, Part(0));
    assert_eq!(tour_bars[8].pass, 2);
    assert_eq!(tour_bars[16].part, Part(1));
    assert_eq!(tour_bars[16].pass, 1);
    assert_eq!(tour_bars[24].part, Part(1));
    assert_eq!(tour_bars[24].pass, 2);

    Ok(())
}
#[test]
fn fixed_roles_gigue_valse_notes_identical_to_main() -> Result<(), Box<dyn std::error::Error>> {
    for style_id in ["gigue_quebecoise", "valse_quebecoise"] {
        let mut profile = StyleProfile::named(style_id)?;
        profile.performance = Default::default();
        for part in &mut profile.form.parts {
            part.second_ending = None;
            part.rhythm.clear();
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
        let tuning =
            harmonium_core::TuningParams { profile: profile.clone(), ..Default::default() };
        let mut params = MusicalParams::default();
        params.bpm = profile.meter.default_bpm;
        params.enable_melody = true;
        params.enable_harmony = true;
        params.enable_rhythm = false; // --feet off
        let mut all_runs = serde_json::Map::new();
        for seed in 1..=3 {
            let mut generator = TradGenerator::new(params.clone(), tuning.clone());
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let bars: Vec<_> = (1..=64)
                .map(|bar| {
                    let m = generator.generate_measure(bar, &mut rng);
                    let s = MeasureSnapshot::from_measure(&m);
                    (bar, notes(&s))
                })
                .collect();
            all_runs.insert(format!("seed_{seed}"), serde_json::to_value(bars)?);
        }
        let golden: serde_json::Value = match style_id {
            "gigue_quebecoise" => {
                serde_json::from_str(include_str!("fixtures/main_golden_gigue_quebecoise.json"))?
            }
            "valse_quebecoise" => {
                serde_json::from_str(include_str!("fixtures/main_golden_valse_quebecoise.json"))?
            }
            _ => unreachable!(),
        };
        let actual = serde_json::Value::Object(all_runs);
        type Runs =
            std::collections::BTreeMap<String, Vec<(usize, Vec<(u8, u8, usize, usize, u8)>)>>;
        // Le contrat musical conserve tous les événements, pas l'ancien ordre
        // non chronologique d'une levée ajoutée après une fin Tail.
        let canonical = |value: serde_json::Value| -> Result<Runs, serde_json::Error> {
            let mut runs: Runs = serde_json::from_value(value)?;
            for bars in runs.values_mut() {
                for (_, notes) in bars {
                    notes.sort_unstable();
                }
            }
            Ok(runs)
        };
        assert_eq!(
            canonical(actual)?,
            canonical(golden)?,
            "notes de {style_id} différentes de main (graines 1..3, 64 mesures, feet off)"
        );
    }
    Ok(())
}

/// Chaque dossier de `profiles/` est distribué : un profil invalide ne doit pas disparaître en
/// silence du moteur (le chargeur embarqué ignore ce qu'il ne sait pas décoder).
#[test]
fn every_profile_directory_is_distributed() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../profiles");
    let mut on_disk: Vec<String> = std::fs::read_dir(&dir)?
        .flatten()
        .filter(|e| e.path().join("profile.toml").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    on_disk.sort();
    let mut distributed: Vec<String> =
        StyleProfile::distributed().iter().map(|p| p.meta.id.clone()).collect();
    distributed.sort();
    assert_eq!(distributed, on_disk);
    Ok(())
}

/// Un identifiant ou un alias désigne un seul style : sinon `--style gigue` dépendrait de l'ordre
/// d'embarquement des profils.
#[test]
fn ids_and_aliases_are_unique_across_distributed_profiles() {
    let mut seen = std::collections::HashMap::new();
    for profile in StyleProfile::distributed() {
        for name in std::iter::once(&profile.meta.id).chain(&profile.meta.aliases) {
            let previous = seen.insert(name.to_lowercase(), profile.meta.id.clone());
            assert!(previous.is_none(), "« {name} » désigne {previous:?} et {}", profile.meta.id);
        }
    }
}

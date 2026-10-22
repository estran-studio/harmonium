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
    let mut third = p.form.parts[0].clone();
    third.name = "C".into();
    third.roles = vec![ImposedRole::Open, ImposedRole::CadFull];
    p.form.parts[0].roles = vec![ImposedRole::Open, ImposedRole::Free, ImposedRole::CadFull];
    p.form.parts[1].roles =
        vec![ImposedRole::Open, ImposedRole::Free, ImposedRole::CopyOf(1), ImposedRole::CadFull];
    p.form.parts.push(third);
    p.form.sequence = vec![2, 0, 1, 0, 2];
    p.harmony.steps_per_chord = 12;
    p.accompaniment.bass_steps = vec![0, 6];
    p.accompaniment.chop_steps = vec![3, 9];
    p.accompaniment.strong_steps = vec![0, 6];
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
    p.form.parts[0].roles[0] = ImposedRole::CopyOf(1);
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
    let profile = StyleProfile::distributed()
        .iter()
        .find(|p| p.meter.time_signature == TimeSignature::new(6, 8))
        .expect("profil 6/8 distribué");

    assert_eq!(&StyleProfile::named(&profile.meta.id)?, profile);
    let prefix = profile.meta.id.split('_').next().unwrap();
    assert_eq!(&StyleProfile::named(prefix)?, profile);
    for tag in &profile.meta.tags {
        if StyleProfile::distributed().iter().filter(|p| p.meta.tags.contains(tag)).count() == 1 {
            assert_eq!(&StyleProfile::named(tag)?, profile);
        }
    }

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
    let profile = StyleProfile::distributed()
        .iter()
        .find(|p| p.meter.time_signature == TimeSignature::new(3, 4))
        .expect("profil 3/4 distribué");

    assert_eq!(&StyleProfile::named(&profile.meta.id)?, profile);
    let prefix = profile.meta.id.split('_').next().unwrap();
    assert_eq!(&StyleProfile::named(prefix)?, profile);
    for tag in &profile.meta.tags {
        if StyleProfile::distributed().iter().filter(|p| p.meta.tags.contains(tag)).count() == 1 {
            assert_eq!(&StyleProfile::named(tag)?, profile);
        }
    }

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

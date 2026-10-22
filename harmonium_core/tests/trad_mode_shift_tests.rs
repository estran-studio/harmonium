//! Contrat : la même ligne change de mode parallèle aux frontières musicales.
use harmonium_core::{
    params::MusicalParams,
    report::MeasureSnapshot,
    timeline::TrackId,
    trad::{
        Mode, ReelArrangement, TradGenerator,
        dynamics::{ColorChoice, Curve},
    },
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn tuning() -> TuningParams {
    let mut tuning = TuningParams::default();
    tuning.reel_arrangement = ReelArrangement::from_start(3);
    tuning.reel_intensity.color = vec![ColorChoice {
        key: 2,
        mode: Mode::Ionian,
        weight: Curve(vec![[0.0, 1.0], [1.0, 1.0]]),
    }];
    tuning
        .reel_intensity
        .parallel_modes
        .retain(|c| !(c.bright == Mode::Ionian && c.dark == Mode::Dorian));
    tuning
}

fn render(changes: &[(usize, f32)]) -> Vec<MeasureSnapshot> {
    let mut params = MusicalParams::default();
    params.harmony_valence = 0.4;
    let mut generator = TradGenerator::new(params.clone(), tuning());
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    (1..=16)
        .map(|bar| {
            if let Some((_, valence)) = changes.iter().find(|(at, _)| *at == bar) {
                params.harmony_valence = *valence;
                generator.invalidate_future(bar - 1);
                generator.update_params(params.clone());
            }
            MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng))
        })
        .collect()
}

fn modes(snapshots: &[MeasureSnapshot]) -> Vec<Mode> {
    snapshots.iter().map(|s| s.position.unwrap().mode).collect()
}

#[test]
fn parallel_mode_preserves_degrees_and_waits_for_next_phrase() {
    let original = render(&[]);
    let shifted = render(&[(3, -0.9), (7, 0.95)]);
    for (old, new) in original.iter().zip(&shifted) {
        let dark = (5..9).contains(&new.index);
        assert_eq!(new.position.unwrap().key, 2);
        assert_eq!(new.position.unwrap().mode, if dark { Mode::Aeolian } else { Mode::Ionian });
        let old_notes: Vec<_> =
            old.notes.iter().filter(|n| n.track == TrackId::Melody.channel()).collect();
        let new_notes: Vec<_> =
            new.notes.iter().filter(|n| n.track == TrackId::Melody.channel()).collect();
        assert_eq!(old_notes.len(), new_notes.len());
        for (before, after) in old_notes.iter().zip(new_notes) {
            let degree = Mode::Ionian.scale_degree_of_midi(2, before.pitch).unwrap();
            let mode = if dark { Mode::Aeolian } else { Mode::Ionian };
            assert_eq!(
                mode.scale_degree_of_midi(2, after.pitch),
                Some(degree),
                "bar {}",
                new.index
            );
            let expected_pitch = i16::from(before.pitch)
                + i16::from(mode.intervals()[usize::from(degree - 1)])
                - i16::from(Mode::Ionian.intervals()[usize::from(degree - 1)]);
            assert_eq!(
                i16::from(after.pitch),
                expected_pitch,
                "octave/contour mesure {}",
                new.index
            );
            assert_eq!(before.start_step, after.start_step);
            assert_eq!(before.duration_steps, after.duration_steps);
        }
    }
}

#[test]
fn hysteresis_retains_color_and_latches_threshold_crossings() {
    let band = [(3, -0.2), (5, 0.0), (7, -0.1), (9, 0.1), (13, -0.3)];
    assert_eq!(modes(&render(&band)), vec![Mode::Ionian; 16]);
    let latched = render(&[(3, -0.9), (4, -0.1), (6, 0.95), (7, 0.0)]);
    assert_eq!(
        modes(&latched),
        [vec![Mode::Ionian; 4], vec![Mode::Aeolian; 4], vec![Mode::Ionian; 8]].concat()
    );
    let dark_band = render(&[(3, -0.9), (6, -0.2), (8, 0.0), (12, 0.1)]);
    assert_eq!(modes(&dark_band)[4..], [Mode::Aeolian; 12]);
}

#[test]
fn accompaniment_and_dense_ornaments_stay_in_the_active_mode() {
    let mut params = MusicalParams::default();
    params.harmony_valence = 0.4;
    params.harmony_tension = 1.0;
    params.rhythm_density = 1.0;
    let mut generator = TradGenerator::new(params.clone(), tuning());
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    for bar in 1..=32 {
        if bar == 3 {
            params.harmony_valence = -0.9;
            generator.invalidate_future(2);
            generator.update_params(params.clone());
        }
        let snapshot = MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng));
        let position = snapshot.position.unwrap();
        for note in snapshot.notes.iter().filter(|n| n.track <= TrackId::Bass.channel()) {
            assert!(
                position.mode.contains_midi(position.key, note.pitch),
                "bar {bar}, track {}, pitch {}",
                note.track,
                note.pitch
            );
        }
    }
}

#[test]
fn part_boundary_waits_eight_bars_and_speculative_shift_can_be_cancelled() {
    let mut params = MusicalParams::default();
    params.harmony_valence = 0.4;
    let mut table = tuning();
    table.reel_intensity.mode_shift_boundary =
        harmonium_core::trad::dynamics::ModeShiftBoundary::Part;
    let mut generator = TradGenerator::new(params.clone(), table);
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    for bar in 1..=16 {
        if bar == 3 {
            params.harmony_valence = -0.9;
            generator.invalidate_future(2);
            generator.update_params(params.clone());
        }
        let s = MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng));
        assert_eq!(s.position.unwrap().mode, if bar < 9 { Mode::Ionian } else { Mode::Aeolian });
    }
    generator.invalidate_future(3);
    params.harmony_valence = 0.95;
    generator.update_params(params);
    for bar in 4..=16 {
        let s = MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng));
        assert_eq!(s.position.unwrap().mode, Mode::Ionian);
    }
}

#[test]
fn identical_automation_replays_identical_snapshots() {
    let changes = [(3, -0.9), (7, 0.95), (11, -0.8)];
    assert_eq!(
        serde_json::to_value(render(&changes)).unwrap(),
        serde_json::to_value(render(&changes)).unwrap()
    );
}

#[test]
fn invalid_mode_tables_are_rejected() {
    let valid = tuning();
    for mutate in [
        |d: &mut harmonium_core::trad::dynamics::StyleDynamics| d.dark_below = d.bright_above,
        |d: &mut harmonium_core::trad::dynamics::StyleDynamics| d.bright_above = f32::NAN,
        |d: &mut harmonium_core::trad::dynamics::StyleDynamics| d.parallel_modes[0].weight = 0,
        |d: &mut harmonium_core::trad::dynamics::StyleDynamics| d.parallel_modes.clear(),
        |d: &mut harmonium_core::trad::dynamics::StyleDynamics| {
            d.parallel_modes[0].dark = Mode::Ionian
        },
    ] {
        let mut invalid = valid.reel_intensity.clone();
        mutate(&mut invalid);
        assert!(invalid.validate().is_err());
    }
}

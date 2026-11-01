//! Critères d'écoute P1-T7 sur le flux de snapshots (20 seeds × 32 mesures) :
//! variété des débuts, registre (centre si4, lift B), percussion discrets par défaut,
//! ouvertures différentes entre tunes consécutifs.

use std::collections::BTreeSet;

use harmonium_core::{
    compose::{Arrangement, TuneGenerator, same_cell_up_to_octave},
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::TrackId,
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const NUM_SEEDS: u32 = 20;
const BARS_32: usize = 32;

/// Tuning « un seul tour, tout le monde dès la mesure 1 » : isole la forme, l'harmonie
/// et l'accompagnement (D4–D9) de l'arrangement progressif (D11), couvert par
/// `arrangement_tests`.
fn flat_tuning() -> TuningParams {
    TuningParams {
        profile: {
            let mut p = TuningParams::default().profile;
            p.arrangement = Arrangement::from_start(1, Default::default());
            p
        },
        ..TuningParams::default()
    }
}

fn tune_params(percussion_density: u8) -> MusicalParams {
    let tuning = TuningParams::default();
    let rf = &tuning.profile.meter;
    let mut params = MusicalParams::default();

    params.bpm = rf.default_bpm;
    params.set_swing_ratio(rf.default_swing_ratio);
    params.time_signature = rf.time_signature;
    params.set_percussion_density(percussion_density);
    params
}

fn generate_32_snapshots(
    seed: u64,
    percussion_density: u8,
) -> (TuneGenerator, Vec<MeasureSnapshot>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut tune_gen = TuneGenerator::new(tune_params(percussion_density), flat_tuning());
    let mut snapshots = Vec::with_capacity(BARS_32);
    for bar in 1..=BARS_32 {
        let m = tune_gen.generate_measure(bar, &mut rng);
        snapshots.push(MeasureSnapshot::from_measure(&m));
    }
    (tune_gen, snapshots)
}

fn notes_on_track(snapshot: &MeasureSnapshot, track: TrackId) -> Vec<&NoteSnapshot> {
    let ch = track.channel();
    snapshot.notes.iter().filter(|n| n.track == ch).collect()
}

fn melody_mean(snaps: &[MeasureSnapshot]) -> Option<f64> {
    let mut sum = 0u64;
    let mut count = 0u64;
    for snap in snaps {
        for n in notes_on_track(snap, TrackId::Melody) {
            sum += u64::from(n.pitch);
            count += 1;
        }
    }
    if count == 0 { None } else { Some(sum as f64 / count as f64) }
}

#[test]
fn p1t7_distinct_openings_and_register_20_seeds() -> Result<(), String> {
    let mut first_bars: BTreeSet<Vec<(u8, usize, usize)>> = BTreeSet::new();
    let mut b_minus_a_sum = 0.0;
    for seed_u32 in 1..=NUM_SEEDS {
        let seed = u64::from(seed_u32);
        let (_, snapshots) = generate_32_snapshots(seed, 1);
        assert_eq!(snapshots.len(), BARS_32);

        first_bars.insert(
            notes_on_track(&snapshots[0], TrackId::Melody)
                .iter()
                .map(|n| (n.pitch, n.start_step, n.duration_steps))
                .collect(),
        );

        let mean = melody_mean(&snapshots).ok_or("mélodie vide".to_string())?;
        assert!((68.0..=75.0).contains(&mean), "seed {seed}: moyenne {mean:.1} hors 68..=75");

        let mean_a = melody_mean(&snapshots[..16]).ok_or("part A vide".to_string())?;
        let mean_b = melody_mean(&snapshots[16..]).ok_or("part B vide".to_string())?;
        b_minus_a_sum += mean_b - mean_a;
    }

    assert!(first_bars.len() >= 16, "seulement {}/20 débuts distincts", first_bars.len());

    let b_minus_a = b_minus_a_sum / f64::from(NUM_SEEDS);
    assert!((0.0..=4.0).contains(&b_minus_a), "contraste B-A {b_minus_a:.2} hors 0..=4");
    Ok(())
}

#[test]
fn p1t7_percussion_quiet_by_default() {
    // Avec l'humanisation (±velocity_humanize), le pied doit rester discret
    // face à la mélodie (vélocité de base ~84) : plafond = base + humanisation.
    let table = TuningParams::default().profile.percussion;
    let percussion_low_ceiling = table.low_velocity.saturating_add(table.velocity_humanize);
    let percussion_high_ceiling = table.high_velocity.saturating_add(table.velocity_humanize);
    assert!(
        percussion_low_ceiling <= 84,
        "percussion grave trop fort par défaut : plafond {percussion_low_ceiling}"
    );
    assert!(
        percussion_high_ceiling <= 84,
        "percussion aiguë trop forte par défaut : plafond {percussion_high_ceiling}"
    );

    for density in 0..=2u8 {
        let (_, snapshots) = generate_32_snapshots(1, density);
        for snap in &snapshots {
            for n in notes_on_track(snap, TrackId::PercussionLow) {
                assert!(
                    n.velocity <= percussion_low_ceiling,
                    "percussion grave vélocité {} > {percussion_low_ceiling}",
                    n.velocity
                );
            }
            for n in notes_on_track(snap, TrackId::PercussionHigh) {
                assert!(
                    n.velocity <= percussion_high_ceiling,
                    "percussion aiguë vélocité {} > {percussion_high_ceiling}",
                    n.velocity
                );
            }
        }
    }
}

#[test]
fn p1t7_consecutive_tunes_start_differently() -> Result<(), String> {
    for seed_u32 in 1..=NUM_SEEDS {
        let seed = u64::from(seed_u32);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut tune_gen = TuneGenerator::new(tune_params(1), flat_tuning());
        for bar in 1..=64 {
            let _ = tune_gen.generate_measure(bar, &mut rng);
        }
        let tune1 = tune_gen.tune_for_bar(1).ok_or("tune 1 absent".to_string())?;
        let tune2 = tune_gen.tune_for_bar(33).ok_or("tune 2 absent".to_string())?;
        assert!(
            !same_cell_up_to_octave(
                &tune1.melody.cells_for_part(harmonium_core::compose::Part(0))[0],
                &tune2.melody.cells_for_part(harmonium_core::compose::Part(0))[0]
            ),
            "seed {seed}: tunes 1 et 2 commencent par la même cellule"
        );
    }
    Ok(())
}

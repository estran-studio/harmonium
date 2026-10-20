//! Critères d'écoute P1-T7 sur le flux de snapshots (20 seeds × 32 mesures) :
//! variété des débuts, registre (centre si4, lift B), pieds discrets par défaut,
//! ouvertures différentes entre tunes consécutifs.

use std::collections::BTreeSet;

use harmonium_core::{
    params::{MusicalParams, Style},
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::TrackId,
    trad::{ReelArrangement, TradGenerator, same_cell_up_to_octave},
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const NUM_SEEDS: u32 = 20;
const BARS_32: usize = 32;

/// Tuning « un seul tour, tout le monde dès la mesure 1 » : isole la forme, l'harmonie
/// et l'accompagnement (D4–D9) de l'arrangement progressif (D11), couvert par
/// `trad_arrangement_tests`.
fn flat_tuning() -> TuningParams {
    TuningParams { reel_arrangement: ReelArrangement::from_start(1), ..TuningParams::default() }
}

fn reel_params(feet_density: u8) -> MusicalParams {
    let tuning = TuningParams::default();
    let rf = &tuning.reel_fiddle;
    let mut params = MusicalParams::default();
    params.set_style(Style::Reel);
    params.bpm = rf.default_bpm;
    params.set_swing_ratio(rf.default_swing_ratio);
    params.time_signature = rf.default_time_signature;
    params.set_feet_density(feet_density);
    params
}

fn generate_32_snapshots(seed: u64, feet_density: u8) -> (TradGenerator, Vec<MeasureSnapshot>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut trad_gen = TradGenerator::new(reel_params(feet_density), flat_tuning());
    let mut snapshots = Vec::with_capacity(BARS_32);
    for bar in 1..=BARS_32 {
        let m = trad_gen.generate_measure(bar, &mut rng);
        snapshots.push(MeasureSnapshot::from_measure(&m));
    }
    (trad_gen, snapshots)
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

        let mean_a = melody_mean(&snapshots[..16]).ok_or("strain A vide".to_string())?;
        let mean_b = melody_mean(&snapshots[16..]).ok_or("strain B vide".to_string())?;
        b_minus_a_sum += mean_b - mean_a;
    }

    assert!(first_bars.len() >= 16, "seulement {}/20 débuts distincts", first_bars.len());

    let b_minus_a = b_minus_a_sum / f64::from(NUM_SEEDS);
    assert!((0.0..=4.0).contains(&b_minus_a), "contraste B-A {b_minus_a:.2} hors 0..=4");
    Ok(())
}

#[test]
fn p1t7_feet_quiet_by_default() {
    for density in 0..=2u8 {
        let (_, snapshots) = generate_32_snapshots(1, density);
        for snap in &snapshots {
            for n in notes_on_track(snap, TrackId::Heel) {
                assert!(n.velocity <= 60, "talon vélocité {} > 60", n.velocity);
            }
            for n in notes_on_track(snap, TrackId::Toe) {
                assert!(n.velocity <= 60, "pointe vélocité {} > 60", n.velocity);
            }
        }
    }
}

#[test]
fn p1t7_consecutive_tunes_start_differently() -> Result<(), String> {
    for seed_u32 in 1..=NUM_SEEDS {
        let seed = u64::from(seed_u32);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut trad_gen = TradGenerator::new(reel_params(1), flat_tuning());
        for bar in 1..=64 {
            let _ = trad_gen.generate_measure(bar, &mut rng);
        }
        let tune1 = trad_gen.tune_for_bar(1).ok_or("tune 1 absent".to_string())?;
        let tune2 = trad_gen.tune_for_bar(33).ok_or("tune 2 absent".to_string())?;
        assert!(
            !same_cell_up_to_octave(&tune1.fiddle.cells_a()[0], &tune2.fiddle.cells_a()[0]),
            "seed {seed}: tunes 1 et 2 commencent par la même cellule"
        );
    }
    Ok(())
}

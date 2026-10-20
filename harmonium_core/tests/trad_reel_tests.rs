//! Tests d'acceptation de `TradGenerator` sur 20 seeds × 64 mesures (D1, D3, D4, D6–D9).

use harmonium_core::{
    params::{MusicalParams, PulseSpec, Style},
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::{GenerationContext, TrackId},
    trad::{ReelArrangement, TradGenerator, TunePlan, is_same_or_relative_plan},
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_music_theory::{note::PitchSymbol, scale::ScaleType};

const NUM_SEEDS: u64 = 20;
const BARS_64: usize = 64;

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

fn generate_64_snapshots(seed: u64, feet_density: u8) -> (TradGenerator, Vec<MeasureSnapshot>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut trad_gen = TradGenerator::new(reel_params(feet_density), flat_tuning());
    let mut snapshots = Vec::with_capacity(BARS_64);
    for bar in 1..=BARS_64 {
        let m = trad_gen.generate_measure(bar, &mut rng);
        snapshots.push(MeasureSnapshot::from_measure(&m));
    }
    (trad_gen, snapshots)
}

fn notes_on_track(snapshot: &MeasureSnapshot, track: TrackId) -> Vec<&NoteSnapshot> {
    let ch = track.channel();
    snapshot.notes.iter().filter(|n| n.track == ch).collect()
}

fn note_tuples(snapshot: &MeasureSnapshot) -> Vec<(u8, u8, usize, usize, u8)> {
    snapshot
        .notes
        .iter()
        .map(|n| (n.track, n.pitch, n.start_step, n.duration_steps, n.velocity))
        .collect()
}

fn cadence_degree_bar8(plan: &TunePlan, melody: &[&NoteSnapshot]) -> Option<u8> {
    let last = melody.last()?;
    if plan.mode.scale_degree_of_midi(plan.key, last.pitch) == Some(1) {
        return Some(1);
    }
    if melody.len() >= 2 && last.start_step >= 12 {
        let penultimate = melody[melody.len() - 2];
        if penultimate.duration_steps >= 4 {
            return plan.mode.scale_degree_of_midi(plan.key, penultimate.pitch);
        }
    }
    plan.mode.scale_degree_of_midi(plan.key, last.pitch)
}

#[test]
fn reel_form_aabb_and_cadences_20_seeds_64_bars() {
    for seed in 0..NUM_SEEDS {
        let (trad_gen, snapshots) = generate_64_snapshots(seed, 1);
        assert_eq!(snapshots.len(), BARS_64);

        for (i, snap) in snapshots.iter().enumerate() {
            assert_eq!(snap.index, i + 1);
            assert_eq!(snap.time_sig_numerator, 2);
            assert_eq!(snap.time_sig_denominator, 2);
            assert_eq!(snap.steps, 16);
            assert_eq!(snap.pulse, PulseSpec { steps_per_pulse: 8, pulses_per_bar: 2 });
            assert!((snap.swing_ratio - 0.56).abs() < f32::EPSILON);
            assert!((snap.tempo - 230.0).abs() < f32::EPSILON);
            assert!(!snap.chord_name.is_empty());
        }

        // Vérifie chacun des deux tunes de 32 mesures (0..32 et 32..64)
        for tune_offset in [0usize, 32] {
            let info = trad_gen.bar_info(tune_offset + 1);
            assert!(info.is_some(), "seed {seed}: tune absent à {}", tune_offset + 1);
            let Some((plan, _)) = info else { continue };
            // Reprises identiques (A1 == A2 et B1 == B2) sur toutes les pistes et accords
            for b in 0..8 {
                let a1 = &snapshots[tune_offset + b];
                let a2 = &snapshots[tune_offset + 8 + b];
                assert_eq!(note_tuples(a1), note_tuples(a2), "seed {seed}: A1 m.{} != A2", b + 1);
                assert_eq!(a1.chord_name, a2.chord_name, "seed {seed}: accord A1 != A2");

                let b1 = &snapshots[tune_offset + 16 + b];
                let b2 = &snapshots[tune_offset + 24 + b];
                assert_eq!(note_tuples(b1), note_tuples(b2), "seed {seed}: B1 m.{} != B2", b + 1);
                assert_eq!(b1.chord_name, b2.chord_name, "seed {seed}: accord B1 != B2");
            }

            // Dans chaque strain, mesures 5-6 == 1-2 pour la mélodie, et cadences aux mesures 4 et 8
            for strain_start in [0usize, 8, 16, 24].map(|s| tune_offset + s) {
                let m1 = notes_on_track(&snapshots[strain_start], TrackId::Melody);
                let m2 = notes_on_track(&snapshots[strain_start + 1], TrackId::Melody);
                let m5 = notes_on_track(&snapshots[strain_start + 4], TrackId::Melody);
                let m6 = notes_on_track(&snapshots[strain_start + 5], TrackId::Melody);
                assert_eq!(
                    m5.iter()
                        .map(|n| (n.pitch, n.start_step, n.duration_steps, n.velocity))
                        .collect::<Vec<_>>(),
                    m1.iter()
                        .map(|n| (n.pitch, n.start_step, n.duration_steps, n.velocity))
                        .collect::<Vec<_>>(),
                    "seed {seed}: m.5 != m.1"
                );
                assert_eq!(
                    m6.iter()
                        .map(|n| (n.pitch, n.start_step, n.duration_steps, n.velocity))
                        .collect::<Vec<_>>(),
                    m2.iter()
                        .map(|n| (n.pitch, n.start_step, n.duration_steps, n.velocity))
                        .collect::<Vec<_>>(),
                    "seed {seed}: m.6 != m.2"
                );

                // Mesure 4 : demi-cadence sur degré 2, 5 ou 7
                let m4 = notes_on_track(&snapshots[strain_start + 3], TrackId::Melody);
                assert!(!m4.is_empty(), "seed {seed}: m.4 vide");
                let Some(last4) = m4.last() else { continue };
                let deg4 = plan.mode.scale_degree_of_midi(plan.key, last4.pitch);
                assert!(matches!(deg4, Some(2 | 5 | 7)), "seed {seed}: m.4 finit sur {deg4:?}");

                // Mesure 8 : cadence parfaite sur la tonique (degré 1) et accord d'attaque V (seconde moitié I)
                let m8 = notes_on_track(&snapshots[strain_start + 7], TrackId::Melody);
                assert_eq!(
                    cadence_degree_bar8(&plan, &m8),
                    Some(1),
                    "seed {seed}: m.8 ne finit pas sur le degré 1"
                );
                assert_eq!(
                    snapshots[strain_start + 7].chord_root_offset,
                    7,
                    "seed {seed}: m.8 1re moitié doit être sur V (+7 demi-tons)"
                );
            }
        }

        // Tune 2 est dans la même tonalité ou la relative de Tune 1
        let p1 = trad_gen.bar_info(1).map(|(p, _)| p);
        let p2 = trad_gen.bar_info(33).map(|(p, _)| p);
        if let (Some(plan1), Some(plan2)) = (p1, p2) {
            assert!(is_same_or_relative_plan(plan1, plan2));
        }
    }
}

#[test]
fn accompaniment_and_feet_positions_match_d8_d9() {
    for density in 0..=2u8 {
        let (_, snapshots) = generate_64_snapshots(1, density);
        for snap in &snapshots {
            // Basse : pas 0 et 8, durée 4
            let bass = notes_on_track(snap, TrackId::Bass);
            assert_eq!(
                bass.iter().map(|n| (n.start_step, n.duration_steps)).collect::<Vec<_>>(),
                vec![(0, 4), (8, 4)]
            );

            // Guitare (Chords) : 3 sons sur le pas 4, 3 sons sur le pas 12, durée 1
            let chords = notes_on_track(snap, TrackId::Chords);
            assert_eq!(chords.len(), 6);
            assert_eq!(
                chords.iter().filter(|n| n.start_step == 4 && n.duration_steps == 1).count(),
                3
            );
            assert_eq!(
                chords.iter().filter(|n| n.start_step == 12 && n.duration_steps == 1).count(),
                3
            );

            // Podorythmie D9
            let heel = notes_on_track(snap, TrackId::Heel);
            let toe = notes_on_track(snap, TrackId::Toe);
            assert!(heel.iter().all(|n| n.pitch == 35 && n.duration_steps == 0));
            assert!(toe.iter().all(|n| n.pitch == 37 && n.duration_steps == 0));

            let heel_steps: Vec<usize> = heel.iter().map(|n| n.start_step).collect();
            let toe_steps: Vec<usize> = toe.iter().map(|n| n.start_step).collect();
            match density {
                0 => {
                    assert_eq!(heel_steps, vec![0, 8]);
                    assert!(toe_steps.is_empty());
                }
                1 => {
                    assert_eq!(heel_steps, vec![0, 4, 8, 12]);
                    assert_eq!(toe_steps, vec![2, 6, 10, 14]);
                }
                2 => {
                    assert_eq!(heel_steps, vec![0, 4, 8, 12]);
                    assert_eq!(toe_steps, vec![2, 3, 6, 7, 10, 11, 14, 15]);
                }
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn trad_generator_silent_advance_matches_linear() {
    let seed = 7u64;
    let (_, linear) = generate_64_snapshots(seed, 1);

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut sought = TradGenerator::new(reel_params(1), flat_tuning());
    let ctx = GenerationContext {
        session_seed: seed,
        key: PitchSymbol::D,
        scale: ScaleType::Diatonic,
        key_pc: 2,
    };
    sought.reset_to_initial(&ctx);
    sought.silent_advance(40, &mut rng);

    for bar in 40..=BARS_64 {
        let m = sought.generate_measure(bar, &mut rng);
        let snap = MeasureSnapshot::from_measure(&m);
        let a = serde_json::to_string(&linear[bar - 1]).unwrap_or_default();
        let b = serde_json::to_string(&snap).unwrap_or_default();
        assert_eq!(a, b, "bar {bar} mismatch after silent_advance(40)");
    }
}

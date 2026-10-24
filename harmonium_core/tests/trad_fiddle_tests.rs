//! Tests d'acceptation observables pour P1-T2 — Ligne de violon d'un reel (D4, D6).
//!
//! Couvre les 6 critères d'acceptation sur 20 seeds déterministes (`ChaCha8Rng`)
//! ainsi qu'un test `#[ignore]` d'export MIDI dans `target/` pour l'écoute.

use std::path::PathBuf;

use harmonium_core::{
    timeline::{Measure, ScoreTimeline, TimelineNote, TrackId, write_midi},
    trad::{
        CellBank, FIFTH_SEMITONES, FiddleLine, ImposedRole, TunePlan, same_cell_up_to_octave, set,
    },
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

const NUM_SEEDS: u64 = 20;

fn generate_fiddle_tune(seed: u64) -> (TunePlan, FiddleLine, Vec<Vec<TimelineNote>>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let bank = CellBank::default();
    let mut profile = harmonium_core::StyleProfile::default();
    for part in &mut profile.form.parts {
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
    let candidates = set::candidates(&profile, profile.dynamics.neutral_valence);
    let (key, mode) = profile.set.choose(None, &candidates, false, rng.gen_range(0..1_000_000));
    let plan = TunePlan::with_profile(key, mode, std::sync::Arc::new(profile));
    let line = FiddleLine::new(plan.clone(), &bank, &mut rng);
    let tune = line.render_tune();
    (plan, line, tune)
}

/// Retourne le degré d'échelle (1..=7) de la note de cadence d'une mesure 8
/// (`CadFull`), en ignorant le pickup éventuel en fin de cellule (ex. `"5 4 3 2 1 - 5, -"`).
fn cadence_degree_bar8(plan: &TunePlan, notes: &[TimelineNote]) -> Option<u8> {
    let last = notes.last()?;
    if plan.mode.scale_degree_of_midi(plan.key, last.pitch) == Some(1) {
        return Some(1);
    }
    // Si la dernière note est un pickup de fin de mesure (après une note tenue sur le temps 2),
    // la note de cadence est l'avant-dernière note.
    if notes.len() >= 2 && last.start_step >= 12 {
        let penultimate = &notes[notes.len() - 2];
        if penultimate.duration_steps >= 4 {
            return plan.mode.scale_degree_of_midi(plan.key, penultimate.pitch);
        }
    }
    plan.mode.scale_degree_of_midi(plan.key, last.pitch)
}

/// Critère 1 : Mesures 5–6 = 1–2 (hauteurs et positions) dans chaque strain ;
/// passes 1 et 2 identiques (sur 20 seeds).
#[test]
fn acceptance_1_form_repeats_and_strain_passes_identical_20_seeds() {
    for seed in 0..NUM_SEEDS {
        let (_, _, tune) = generate_fiddle_tune(seed);
        assert_eq!(tune.len(), 32, "seed {seed}: un tune AABB doit compter 32 mesures");

        // Strain A : passe 1 (0..8) == passe 2 (8..16)
        for b in 0..8 {
            assert_eq!(
                tune[b],
                tune[b + 8],
                "seed {seed}: strain A passe 1 mesure {} != passe 2 mesure {}",
                b + 1,
                b + 9
            );
        }

        // Strain B : passe 1 (16..24) == passe 2 (24..32)
        for b in 16..24 {
            assert_eq!(
                tune[b],
                tune[b + 8],
                "seed {seed}: strain B passe 1 mesure {} != passe 2 mesure {}",
                b + 1,
                b + 9
            );
        }

        // Dans chaque strain (A passe 1, A passe 2, B passe 1, B passe 2), mesures 5-6 == 1-2
        for section_start in [0usize, 8, 16, 24] {
            assert_eq!(
                tune[section_start + 4],
                tune[section_start],
                "seed {seed}: section {section_start} mesure 5 != mesure 1"
            );
            assert_eq!(
                tune[section_start + 5],
                tune[section_start + 1],
                "seed {seed}: section {section_start} mesure 6 != mesure 2"
            );
        }
    }
}

/// Critère 2 : Mesure 4 : dernière note sur un degré 2, 5 ou 7 ;
/// mesure 8 : dernière note (hors pickup éventuel de la cellule) sur le degré 1 (sur 20 seeds).
#[test]
fn acceptance_2_half_and_full_cadences_20_seeds() {
    for seed in 0..NUM_SEEDS {
        let (plan, _, tune) = generate_fiddle_tune(seed);

        for section_start in [0usize, 8, 16, 24] {
            // Mesure 4 de la strain (indice section_start + 3)
            let bar4 = &tune[section_start + 3];
            assert!(!bar4.is_empty(), "seed {seed}: mesure 4 vide");
            let last_bar4 = &bar4[bar4.len() - 1];
            let deg4 = plan.mode.scale_degree_of_midi(plan.key, last_bar4.pitch);
            assert!(
                matches!(deg4, Some(2 | 5 | 7)),
                "seed {seed} section {section_start}: mesure 4 finit sur degré {deg4:?} (attendu 2, 5 ou 7)"
            );

            // Mesure 8 de la strain (indice section_start + 7)
            let bar8 = &tune[section_start + 7];
            assert!(!bar8.is_empty(), "seed {seed}: mesure 8 vide");
            let deg8 = cadence_degree_bar8(&plan, bar8);
            assert_eq!(
                deg8,
                Some(1),
                "seed {seed} section {section_start}: mesure 8 finit sur degré {deg8:?} (attendu 1)"
            );
        }
    }
}

/// Critère 3 : Toutes les hauteurs dans 55..=88 (G3–E6, P1-T7) ; aucune note hors du mode (sur 20 seeds).
#[test]
fn acceptance_3_tessitura_and_diatonic_mode_20_seeds() {
    for seed in 0..NUM_SEEDS {
        let (plan, _, tune) = generate_fiddle_tune(seed);

        for (bar_idx, bar_notes) in tune.iter().enumerate() {
            assert!(!bar_notes.is_empty(), "seed {seed} bar {bar_idx}: mesure vide");
            for note in bar_notes {
                assert!(
                    (55..=88).contains(&note.pitch),
                    "seed {seed} bar {bar_idx}: hauteur MIDI {} hors tessiture 55..=88",
                    note.pitch
                );
                assert!(
                    plan.mode.contains_midi(plan.key, note.pitch),
                    "seed {seed} bar {bar_idx}: hauteur MIDI {} hors du mode {:?} (tonique {})",
                    note.pitch,
                    plan.mode,
                    plan.key
                );
            }
        }
    }
}

/// Critère 4 : Continuité : saut entre dernière note d'une mesure et première
/// de la suivante ≤ une quinte (7 demi-tons) dans ≥ 95 % des cas (sur 20 seeds).
#[test]
fn acceptance_4_inter_bar_continuity_at_least_95_percent_20_seeds() {
    let mut total_transitions = 0usize;
    let mut within_fifth_transitions = 0usize;

    for seed in 0..NUM_SEEDS {
        let (_, _, tune) = generate_fiddle_tune(seed);
        let mut seed_transitions = 0usize;
        let mut seed_within_fifth = 0usize;

        for pair in tune.windows(2) {
            let prev_bar = &pair[0];
            let next_bar = &pair[1];
            let prev_last = prev_bar[prev_bar.len() - 1].pitch;
            let next_first = next_bar[0].pitch;
            let leap_semitones = (i16::from(next_first) - i16::from(prev_last)).abs();

            seed_transitions += 1;
            total_transitions += 1;
            if leap_semitones <= i16::from(FIFTH_SEMITONES) {
                seed_within_fifth += 1;
                within_fifth_transitions += 1;
            }
        }

        let seed_ratio = (seed_within_fifth as f64) / (seed_transitions as f64);
        assert!(
            seed_ratio >= 0.95,
            "seed {seed}: continuité ≤ quinte = {seed_within_fifth}/{seed_transitions} ({:.1}% < 95%)",
            seed_ratio * 100.0
        );
    }

    let global_ratio = (within_fifth_transitions as f64) / (total_transitions as f64);
    assert!(
        global_ratio >= 0.95,
        "continuité globale ≤ quinte = {within_fifth_transitions}/{total_transitions} ({:.1}% < 95%)",
        global_ratio * 100.0
    );
}

/// Critère 5 (P1-T7) : contraste partie B − partie A en moyenne sur 20 seeds
/// dans 0..=4 demi-tons (l'ancienne règle « B sur 6–8 » produisait +6 ;
/// cible `strain_b_lift_semitones` = +2 autour de `melody_center_midi`).
#[test]
fn acceptance_5_strain_b_lift_contrast_0_to_4_semitones_20_seeds() {
    let mut total_b_minus_a = 0.0;
    for seed in 0..NUM_SEEDS {
        let (_, _, tune) = generate_fiddle_tune(seed);

        let a_pitches: Vec<f64> =
            tune[0..16].iter().flat_map(|bar| bar.iter().map(|n| f64::from(n.pitch))).collect();
        let b_pitches: Vec<f64> =
            tune[16..32].iter().flat_map(|bar| bar.iter().map(|n| f64::from(n.pitch))).collect();

        let mean_a = a_pitches.iter().sum::<f64>() / (a_pitches.len() as f64);
        let mean_b = b_pitches.iter().sum::<f64>() / (b_pitches.len() as f64);
        total_b_minus_a += mean_b - mean_a;
    }

    let contrast = total_b_minus_a / (NUM_SEEDS as f64);
    assert!(
        (0.0..=4.0).contains(&contrast),
        "contraste B − A moyen {contrast:.2} hors 0..=4 demi-tons"
    );
}

/// Critère 6 : Même seed ⇒ mêmes notes ; deux seeds ⇒ tunes différents.
#[test]
fn acceptance_6_determinism_and_seed_diversity() {
    for seed in 0..NUM_SEEDS {
        let (plan_1, _, tune_1) = generate_fiddle_tune(seed);
        let (plan_2, _, tune_2) = generate_fiddle_tune(seed);
        assert_eq!(plan_1, plan_2, "seed {seed}: plans non déterministes");
        assert_eq!(tune_1, tune_2, "seed {seed}: notes non déterministes");
    }

    for seed in 0..(NUM_SEEDS - 1) {
        let (_, _, tune_a) = generate_fiddle_tune(seed);
        let (_, _, tune_b) = generate_fiddle_tune(seed + 1);
        assert_ne!(
            tune_a,
            tune_b,
            "seeds {seed} et {} ont produit exactement les mêmes 32 mesures",
            seed + 1
        );
    }
}

/// Anti-répétition sur les mesures libres (`ImposedRole::Free`) :
/// sur 20 seeds (et sur la seed 42 signalée à l'écoute), aucune mesure `Free`
/// (mesures 2, 3 et 7 de chaque strain) ne réutilise la cellule de la mesure précédente.
#[test]
fn acceptance_7_free_bars_never_repeat_previous_bar_cell_20_seeds() {
    for seed in (0..NUM_SEEDS).chain(std::iter::once(42)) {
        let (plan, line, tune) = generate_fiddle_tune(seed);
        for bar in 0..plan.total_bars() {
            let role = plan.bar_role(bar);
            if matches!(role.role, ImposedRole::Free) && bar > 0 {
                let prev_cell = line.cell_for_bar(bar - 1);
                let curr_cell = line.cell_for_bar(bar);
                assert!(
                    !same_cell_up_to_octave(curr_cell, prev_cell),
                    "seed {seed} bar {} (strain {:?} m.{}): la mesure Free a réutilisé la cellule de la mesure précédente",
                    bar + 1,
                    role.strain,
                    role.bar_in_strain
                );
                assert_ne!(
                    tune[bar],
                    tune[bar - 1],
                    "seed {seed} bar {}: notes identiques à la mesure précédente sur une mesure Free",
                    bar + 1
                );
            }
        }
    }
}

/// Export MIDI d'un tune complet de 32 mesures dans `target/trad_fiddle_demo.mid` pour écoute.
///
/// Commande :
/// `cargo test -p harmonium_core --test trad_fiddle_tests export_fiddle_tune_to_midi_for_listening -- --ignored --nocapture`
#[test]
#[ignore = "écrit un fichier MIDI de démonstration dans target/ pour écoute manuelle"]
fn export_fiddle_tune_to_midi_for_listening() {
    let (plan, _, tune) = generate_fiddle_tune(42);
    let mut timeline = ScoreTimeline::new(64);
    let time_sig = plan.profile.meter.time_signature;
    let steps = plan.profile.meter.steps_per_bar;
    let pulse = plan.profile.meter.pulse_spec();
    let swing = plan.profile.meter.default_swing_ratio;

    for (bar_idx, notes) in tune.into_iter().enumerate() {
        let mut measure = Measure::new(bar_idx + 1, time_sig, 230.0, steps);
        measure.pulse = pulse;
        measure.swing_ratio = swing;
        for mut note in notes {
            note.id = timeline.next_note_id();
            measure.add_note(TrackId::Melody, note);
        }
        timeline.push_measure(measure);
    }

    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("target"), |root| root.join("target"));
    let _ = std::fs::create_dir_all(&target_dir);
    let midi_path = target_dir.join("trad_fiddle_demo.mid");

    let res = write_midi(&timeline, &plan.profile, &midi_path);
    assert!(res.is_ok(), "échec de l'écriture MIDI: {res:?}");
    assert!(midi_path.exists(), "le fichier MIDI n'a pas été créé: {}", midi_path.display());
    println!("Tune MIDI exporté vers : {}", midi_path.display());
}

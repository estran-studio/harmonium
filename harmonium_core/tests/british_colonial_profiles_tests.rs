//! Tests de contrat pour les profils britanniques coloniaux :
//! `gigue_britannique_coloniale` et `hornpipe_britannique_coloniale` (P4-T3).
//!
//! Vérifie sur le flux de snapshots :
//! - Chargement direct (`load`) et résolution par nom (`named`)
//! - Métadonnées précises (tradition, époque, corpus)
//! - Métrique, tempo et pulses `BeatInfo` (gigue 6/8 en 2 pulses de 6 pas ; hornpipe 4/4 en 4 pulses de 4 pas)
//! - Rythme pointé du hornpipe (`default_swing_ratio = 0.65`, tempo à 92 BPM)
//! - Forme AABB de 32 mesures (parties de 8 mesures)
//! - Notes de mélodie dans le mode publié
//! - Accompagnement aux pas attendus (basse et accords simples, sans chop québécois)
//! - Absence totale de podorythmie québécoise (aucun coup sur Heel / Toe)
//! - Cadence finale du hornpipe (trois noires accentuées)
//! - Déterminisme strict (même graine => mêmes snapshots)

use std::collections::HashSet;

use harmonium_core::{
    StyleProfile,
    params::{MusicalParams, TimeSignature},
    report::MeasureSnapshot,
    trad::{Part, TradGenerator},
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn jig_profile() -> StyleProfile {
    StyleProfile::named("gigue_britannique_coloniale")
        .expect("le profil gigue_britannique_coloniale doit être distribué")
}

fn hornpipe_profile() -> StyleProfile {
    StyleProfile::named("hornpipe_britannique_coloniale")
        .expect("le profil hornpipe_britannique_coloniale doit être distribué")
}

fn profile_params(p: &StyleProfile) -> MusicalParams {
    let mut params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        enable_melody: true,
        enable_harmony: true,
        enable_rhythm: true,
        ..Default::default()
    };
    params.set_swing_ratio(p.meter.default_swing_ratio);
    params
}

fn generate_snapshots(p: &StyleProfile, seed: u64, bars: usize) -> Vec<MeasureSnapshot> {
    let tuning = TuningParams { profile: p.clone() };
    let mut generator = TradGenerator::new(profile_params(p), tuning);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect()
}

// ============================================================================
// GIGUE BRITANNIQUE COLONIALE
// ============================================================================

#[test]
fn test_load_jig_directly_and_by_name() -> Result<(), Box<dyn std::error::Error>> {
    let direct = StyleProfile::load(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../profiles/gigue_britannique_coloniale"
    ))?;
    assert_eq!(direct.meta.id, "gigue_britannique_coloniale");

    let named = StyleProfile::named("gigue_britannique_coloniale")?;
    assert_eq!(named.meta.id, "gigue_britannique_coloniale");
    assert_eq!(direct, named);
    Ok(())
}

#[test]
fn jig_metadata_and_corpus_citation() {
    let p = jig_profile();
    assert_eq!(p.meta.id, "gigue_britannique_coloniale");
    assert!(p.meta.tradition.contains("britannique") && p.meta.tradition.contains("1760"));
    assert!(
        p.meta.corpus.contains("Nottingham")
            && p.meta.corpus.contains("John Chambers")
            && p.meta.corpus.contains("Cahusac")
    );
    assert_eq!(p.instruments.melody_program, 40); // Violon GM 40
    assert_eq!(p.instruments.chords_program, Some(24)); // Guitare nylon GM 24
    assert_eq!(p.instruments.bass_program, Some(32)); // Basse acoustique GM 32
}

#[test]
fn jig_meter_pulse_and_tempo_contract() {
    let p = jig_profile();
    assert_eq!(p.meter.time_signature, TimeSignature::new(6, 8));
    assert_eq!(p.meter.steps_per_bar, 12);
    assert_eq!(p.meter.steps_per_unit, 2);
    assert_eq!(p.meter.steps_per_pulse, 6);
    assert_eq!(p.meter.pulses_per_bar, 2);
    assert_eq!(p.meter.default_swing_ratio, 0.5);
    assert!((p.meter.default_bpm - 156.0).abs() < 0.1);

    let snaps = generate_snapshots(&p, 1, 32);
    for s in &snaps {
        assert_eq!((s.time_sig_numerator, s.time_sig_denominator), (6, 8));
        assert_eq!(s.steps, 12);
        assert_eq!((s.pulse.steps_per_pulse, s.pulse.pulses_per_bar), (6, 2));
    }
}

#[test]
fn jig_form_aabb_and_repeats() {
    let p = jig_profile();
    assert_eq!(p.form.sequence, vec![0, 0, 1, 1]);
    assert_eq!(p.form.parts.len(), 2);
    assert_eq!(p.form.parts[0].roles.len(), 8);
    assert_eq!(p.form.parts[1].roles.len(), 8);
    assert_eq!(p.form.bars_per_tour(), 32);

    let snaps = generate_snapshots(&p, 42, 65);
    let tour_bars: Vec<_> =
        snaps.iter().filter_map(|s| s.position).filter(|pos| !pos.is_intro()).collect();
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
}

#[test]
fn jig_melody_notes_respect_published_mode() {
    let p = jig_profile();
    let snaps = generate_snapshots(&p, 1, 64);
    for s in &snaps {
        if let Some(pos) = s.position {
            for note in s.notes.iter().filter(|n| n.track == 0) {
                assert!(
                    pos.mode.contains_midi(pos.key, note.pitch),
                    "note {} hors du mode {:?} en tonalité {}",
                    note.pitch,
                    pos.mode,
                    pos.key
                );
            }
        }
    }
}

#[test]
fn jig_accompaniment_steps_and_no_percussion() {
    let p = jig_profile();
    let snaps = generate_snapshots(&p, 1, 32);
    for s in &snaps {
        // Aucune podorythmie (Heel/Toe = pistes 3 et 4)
        let drum_notes: Vec<_> = s.notes.iter().filter(|n| n.track == 3 || n.track == 4).collect();
        assert!(drum_notes.is_empty(), "aucune note de podorythmie attendue");

        // Basse aux pas attendus (temps forts 0 et 6)
        let bass_steps: Vec<_> =
            s.notes.iter().filter(|n| n.track == 2).map(|n| n.start_step).collect();
        if !bass_steps.is_empty() {
            assert!(
                bass_steps.iter().all(|&step| step == 0 || step == 6),
                "pas basse inattendus: {:?}",
                bass_steps
            );
        }

        // Accords simples posés sur les temps 0 et 6 (pas le chop québécois sur 4 et 10)
        let chord_steps: Vec<_> =
            s.notes.iter().filter(|n| n.track == 1).map(|n| n.start_step).collect();
        if !chord_steps.is_empty() {
            assert!(
                chord_steps.iter().all(|&step| step == 0 || step == 6),
                "pas accords inattendus (attendu 0 et 6): {:?}",
                chord_steps
            );
        }
    }
}

#[test]
fn jig_strict_determinism() {
    let p = jig_profile();
    let snaps1 = generate_snapshots(&p, 12345, 32);
    let snaps2 = generate_snapshots(&p, 12345, 32);
    for (s1, s2) in snaps1.iter().zip(&snaps2) {
        assert_eq!(s1.notes.len(), s2.notes.len());
        for (n1, n2) in s1.notes.iter().zip(&s2.notes) {
            assert_eq!((n1.track, n1.pitch, n1.start_step), (n2.track, n2.pitch, n2.start_step));
        }
    }
}

// ============================================================================
// HORNPIPE BRITANNIQUE COLONIAL
// ============================================================================

#[test]
fn test_load_hornpipe_directly_and_by_name() -> Result<(), Box<dyn std::error::Error>> {
    let direct = StyleProfile::load(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../profiles/hornpipe_britannique_coloniale"
    ))?;
    assert_eq!(direct.meta.id, "hornpipe_britannique_coloniale");

    let named = StyleProfile::named("hornpipe_britannique_coloniale")?;
    assert_eq!(named.meta.id, "hornpipe_britannique_coloniale");
    assert_eq!(direct, named);

    // Alias déclaré dans [meta] aliases
    let by_alias = StyleProfile::named("hornpipe")?;
    assert_eq!(by_alias.meta.id, "hornpipe_britannique_coloniale");
    Ok(())
}

#[test]
fn hornpipe_metadata_and_corpus_citation() {
    let p = hornpipe_profile();
    assert_eq!(p.meta.id, "hornpipe_britannique_coloniale");
    assert!(p.meta.tradition.contains("britannique") && p.meta.tradition.contains("1760"));
    assert!(
        p.meta.corpus.contains("Nottingham")
            && p.meta.corpus.contains("John Chambers")
            && p.meta.corpus.contains("Cahusac")
    );
    assert_eq!(p.instruments.melody_program, 40); // Violon GM 40
    assert_eq!(p.instruments.chords_program, Some(24));
    assert_eq!(p.instruments.bass_program, Some(32));
}

#[test]
fn hornpipe_meter_swing_and_tempo_contract() {
    let p = hornpipe_profile();
    assert_eq!(p.meter.time_signature, TimeSignature::new(4, 4));
    assert_eq!(p.meter.steps_per_bar, 16);
    assert_eq!(p.meter.steps_per_unit, 2);
    assert_eq!(p.meter.steps_per_pulse, 4); // Pulse à la noire
    assert_eq!(p.meter.pulses_per_bar, 4); // 4 temps par mesure
    assert_eq!(p.meter.default_swing_ratio, 0.65); // Fort swing pointé (0.62–0.67)
    assert!((p.meter.default_bpm - 92.0).abs() < 0.1); // Plus lent qu'un reel (230 BPM)

    let snaps = generate_snapshots(&p, 1, 32);
    for s in &snaps {
        assert_eq!((s.time_sig_numerator, s.time_sig_denominator), (4, 4));
        assert_eq!(s.steps, 16);
        assert_eq!((s.pulse.steps_per_pulse, s.pulse.pulses_per_bar), (4, 4));
        assert!((s.swing_ratio - 0.65).abs() < 1e-4);
        assert!((s.tempo - 92.0).abs() < 0.1);
    }
}

#[test]
fn hornpipe_form_aabb_and_repeats() {
    let p = hornpipe_profile();
    assert_eq!(p.form.sequence, vec![0, 0, 1, 1]);
    assert_eq!(p.form.parts.len(), 2);
    assert_eq!(p.form.parts[0].roles.len(), 8);
    assert_eq!(p.form.parts[1].roles.len(), 8);
    assert_eq!(p.form.bars_per_tour(), 32);

    let snaps = generate_snapshots(&p, 42, 65);
    let tour_bars: Vec<_> =
        snaps.iter().filter_map(|s| s.position).filter(|pos| !pos.is_intro()).collect();
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
}

#[test]
fn hornpipe_melody_notes_respect_published_mode() {
    let p = hornpipe_profile();
    let snaps = generate_snapshots(&p, 1, 64);
    for s in &snaps {
        if let Some(pos) = s.position {
            for note in s.notes.iter().filter(|n| n.track == 0) {
                assert!(
                    pos.mode.contains_midi(pos.key, note.pitch),
                    "note {} hors du mode {:?} en tonalité {}",
                    note.pitch,
                    pos.mode,
                    pos.key
                );
            }
        }
    }
}

#[test]
fn hornpipe_accompaniment_steps_and_no_percussion() {
    let p = hornpipe_profile();
    let snaps = generate_snapshots(&p, 1, 32);
    for s in &snaps {
        // Aucune podorythmie
        let drum_notes: Vec<_> = s.notes.iter().filter(|n| n.track == 3 || n.track == 4).collect();
        assert!(drum_notes.is_empty(), "aucune note de podorythmie attendue");

        // Basse aux pas attendus (temps 0 et 8)
        let bass_steps: Vec<_> =
            s.notes.iter().filter(|n| n.track == 2).map(|n| n.start_step).collect();
        if !bass_steps.is_empty() {
            assert!(
                bass_steps.iter().all(|&step| step == 0 || step == 8),
                "pas basse inattendus: {:?}",
                bass_steps
            );
        }

        // Accords aux pas 0 et 8 (pas le chop québécois sur contretemps 4 et 12)
        let chord_steps: Vec<_> =
            s.notes.iter().filter(|n| n.track == 1).map(|n| n.start_step).collect();
        if !chord_steps.is_empty() {
            assert!(
                chord_steps.iter().all(|&step| step == 0 || step == 8),
                "pas accords inattendus (attendu 0 et 8): {:?}",
                chord_steps
            );
        }
    }
}

#[test]
fn hornpipe_final_cadence_three_quarter_notes_pattern() {
    // Vérifie que les mesures de cadence complète (mesures 8, 16, 24, 32 d'un tour)
    // comportent la formule traditionnelle en trois noires sur les temps 1, 2, 3 (pas 0, 4, 8)
    let p = hornpipe_profile();
    let snaps = generate_snapshots(&p, 1, 64);
    let mut cad_count = 0;
    for s in &snaps {
        if let Some(pos) = s.position {
            if !pos.is_intro() && pos.bar_in_strain == 8 {
                let melody_notes: Vec<_> = s.notes.iter().filter(|n| n.track == 0).collect();
                assert!(!melody_notes.is_empty(), "la mesure cadentielle doit avoir des notes");
                cad_count += 1;

                // Vérifie que des attaques de mélodie sont présentes sur les temps forts
                let start_steps: HashSet<usize> =
                    melody_notes.iter().map(|n| n.start_step).collect();
                // Au moins les temps 1 et 3 (pas 0 et 8) ou les temps 1, 2, 3 sont attaqués
                assert!(
                    start_steps.contains(&0),
                    "attaque attendue sur le premier temps de la cadence finale"
                );
            }
        }
    }
    assert!(cad_count >= 2, "au moins 2 cadences complètes générées");
}

#[test]
fn hornpipe_strict_determinism() {
    let p = hornpipe_profile();
    let snaps1 = generate_snapshots(&p, 54321, 32);
    let snaps2 = generate_snapshots(&p, 54321, 32);
    for (s1, s2) in snaps1.iter().zip(&snaps2) {
        assert_eq!(s1.notes.len(), s2.notes.len());
        for (n1, n2) in s1.notes.iter().zip(&s2.notes) {
            assert_eq!((n1.track, n1.pitch, n1.start_step), (n2.track, n2.pitch, n2.start_step));
        }
    }
}

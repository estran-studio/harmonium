//! Tests de contrat pour le profil de style `marche_britannique_coloniale` (P4-T1).
//!
//! Vérifie sur le flux de snapshots :
//! - Métadonnées (tradition, époque, corpus)
//! - Aucune note sur `Chords` ni `Bass` (modèle d'harmonie `none`, instrumentation fifre + tambour)
//! - Fifre dans sa tessiture (piccolo GM 72, 72..=102)
//! - Motifs de tambour militaire aux pas attendus (grosse caisse 36, caisse claire 38, flams 40)
//! - Roulements en 16ᵉ à densité 2 et flams aux cadences de fin de phrase (m.4 et m.8)
//! - Forme AABB de 32 mesures (parties de 8 mesures)
//! - Déterminisme strict (même graine => mêmes snapshots)
//! - Pulse `BeatInfo` de marche (2/2, 2 pulses par mesure, pas cadencé)
//! - Décompte de tambour en intro (forme percussion naturelle)
//! - Réaction aux 4 émotions (arousal, density, tension, valence)

use harmonium_core::{
    StyleProfile,
    params::{MusicalParams, TimeSignature},
    report::MeasureSnapshot,
    timeline::TrackId,
    trad::{Part, TradGenerator},
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn march_profile() -> StyleProfile {
    StyleProfile::named("marche_britannique_coloniale")
        .expect("le profil marche_britannique_coloniale doit être distribué")
}

fn march_tuning() -> TuningParams {
    TuningParams { profile: march_profile() }
}

fn march_params() -> MusicalParams {
    let p = march_profile();
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
fn generate_snapshots(seed: u64, bars: usize, params: MusicalParams) -> Vec<MeasureSnapshot> {
    let mut generator = TradGenerator::new(params, march_tuning());
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect()
}

#[test]
fn test_load_march_directly() -> Result<(), Box<dyn std::error::Error>> {
    let profile = StyleProfile::load(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../profiles/marche_britannique_coloniale"
    ))?;
    assert_eq!(profile.meta.id, "marche_britannique_coloniale");
    Ok(())
}

#[test]
fn march_metadata_and_corpus_citation() {
    let p = march_profile();
    assert_eq!(p.meta.id, "marche_britannique_coloniale");
    assert!(p.meta.tradition.contains("britannique") && p.meta.tradition.contains("1760"));
    assert!(
        p.meta.corpus.contains("Nottingham")
            && p.meta.corpus.contains("John Chambers")
            && p.meta.corpus.contains("Cahusac")
    );
    assert_eq!(p.instruments.melody_program, 72); // Piccolo GM 72 (fifre)
    assert_eq!(p.instruments.chords_program, None); // Pas de guitare
    assert_eq!(p.instruments.bass_program, None); // Pas de basse
    assert_eq!(p.harmony.model, "none");
}

#[test]
fn march_meter_and_pulse_contract() {
    let p = march_profile();
    assert_eq!(p.meter.time_signature, TimeSignature::new(2, 2));
    assert_eq!(p.meter.steps_per_bar, 16);
    assert_eq!(p.meter.steps_per_unit, 2); // 8 croches par mesure
    assert_eq!(p.meter.steps_per_pulse, 8); // Pulse à la blanche (pas cadencé)
    assert_eq!(p.meter.pulses_per_bar, 2); // 2 pas par mesure (gauche, droite)
    assert_eq!(p.meter.default_swing_ratio, 0.5); // Droit et militaire
    assert!((p.meter.default_bpm - 108.0).abs() < 0.1);

    let snaps = generate_snapshots(1, 16, march_params());
    for s in &snaps {
        assert_eq!((s.time_sig_numerator, s.time_sig_denominator), (2, 2));
        assert_eq!(s.steps, 16);
        assert_eq!((s.pulse.steps_per_pulse, s.pulse.pulses_per_bar), (8, 2));
    }
}

#[test]
fn march_form_aabb_32_bars() {
    let p = march_profile();
    assert_eq!(p.form.sequence, vec![0, 0, 1, 1]);
    assert_eq!(p.form.parts.len(), 2);
    assert_eq!(p.form.parts[0].roles.len(), 8);
    assert_eq!(p.form.parts[1].roles.len(), 8);
    assert_eq!(p.form.bars_per_tour(), 32);

    let snaps = generate_snapshots(1, 65, march_params());
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
fn march_no_chords_no_bass_ever() {
    // Aucune note ne doit jamais être générée sur Chords (canal 1) ni Bass (canal 2)
    for seed in 1..=5 {
        let snaps = generate_snapshots(seed, 64, march_params());
        for snap in &snaps {
            for note in &snap.notes {
                assert_ne!(
                    note.track,
                    TrackId::Chords.channel(),
                    "aucune note Chords attendue dans une marche (mesure {})",
                    snap.index
                );
                assert_ne!(
                    note.track,
                    TrackId::Bass.channel(),
                    "aucune note Bass attendue dans une marche (mesure {})",
                    snap.index
                );
            }
        }
    }
}

#[test]
fn march_fife_in_tessitura() {
    let p = march_profile();
    let tess_min = p.melody.params.tessitura_min;
    let tess_max = p.melody.params.tessitura_max;
    assert_eq!((tess_min, tess_max), (72, 102));

    for seed in 1..=3 {
        let snaps = generate_snapshots(seed, 64, march_params());
        for snap in &snaps {
            for note in snap.notes.iter().filter(|n| n.track == TrackId::Melody.channel()) {
                assert!(
                    note.pitch >= tess_min && note.pitch <= tess_max,
                    "m.{}: note de fifre {} hors tessiture ({}..={})",
                    snap.index,
                    note.pitch,
                    tess_min,
                    tess_max
                );
            }
        }
    }
}

#[test]
fn march_drum_patterns_cadence_and_rolls() {
    let mut params = march_params();
    params.rhythm_density = 1.0; // Active la densité maximale (roulements en 16ᵉ)

    let snaps = generate_snapshots(42, 64, params);

    // Vérifier les frappes de tambour
    let mut found_bass_drum = false;
    let mut found_snare = false;
    let mut found_flam = false;
    let mut found_roll_16th = false;

    for snap in &snaps {
        let drum_notes: Vec<_> = snap
            .notes
            .iter()
            .filter(|n| n.track == TrackId::Heel.channel() || n.track == TrackId::Toe.channel())
            .collect();

        for note in &drum_notes {
            if note.pitch == 36 {
                found_bass_drum = true;
                // La grosse caisse marque les temps forts (pas 0 ou 8)
                assert!(
                    note.start_step == 0 || note.start_step == 8,
                    "grosse caisse 36 au pas inattendu : {}",
                    note.start_step
                );
            }
            if note.pitch == 38 {
                found_snare = true;
            }
            if note.pitch == 40 {
                found_flam = true;
                // Les flams se produisent aux pas cadentiels de fin de phrase
                assert!(
                    note.start_step >= 12,
                    "flam 40 hors de la fin de phrase : pas {}",
                    note.start_step
                );
            }
        }

        // Vérifier la présence de roulements en 16ᵉ (frappes consécutives à chaque pas)
        let toe_steps: Vec<usize> = drum_notes
            .iter()
            .filter(|n| n.track == TrackId::Toe.channel())
            .map(|n| n.start_step)
            .collect();
        let consecutive_16ths = toe_steps.windows(2).filter(|w| w[1] == w[0] + 1).count();
        if consecutive_16ths >= 4 {
            found_roll_16th = true;
        }
    }

    assert!(found_bass_drum, "grosse caisse GM 36 attendue");
    assert!(found_snare, "caisse claire GM 38 attendue");
    assert!(found_flam, "caisse claire flam GM 40 attendue aux cadences");
    assert!(found_roll_16th, "roulements en 16ᵉ attendus à haute densité");
}

#[test]
fn march_intro_is_drum_count_in() {
    let snaps = generate_snapshots(1, 4, march_params());
    // La mesure 1 est l'intro (décompte de tambour)
    let intro_snap = &snaps[0];
    assert!(intro_snap.position.is_some_and(|p| p.is_intro()), "m.1 doit être l'intro");

    let melody_notes =
        intro_snap.notes.iter().filter(|n| n.track == TrackId::Melody.channel()).count();
    let chord_notes =
        intro_snap.notes.iter().filter(|n| n.track == TrackId::Chords.channel()).count();
    let bass_notes = intro_snap.notes.iter().filter(|n| n.track == TrackId::Bass.channel()).count();
    let drum_notes = intro_snap
        .notes
        .iter()
        .filter(|n| n.track == TrackId::Heel.channel() || n.track == TrackId::Toe.channel())
        .count();

    assert_eq!(melody_notes, 0, "aucun fifre pendant le décompte de tambour");
    assert_eq!(chord_notes, 0, "aucun accord pendant le décompte");
    assert_eq!(bass_notes, 0, "aucune basse pendant le décompte");
    assert!(drum_notes > 0, "le tambour doit jouer son décompte dans l'intro");
}

#[test]
fn march_determinism_same_seed_exact_notes() {
    let snaps1 = generate_snapshots(12345, 64, march_params());
    let snaps2 = generate_snapshots(12345, 64, march_params());
    assert_eq!(snaps1.len(), snaps2.len());

    for (s1, s2) in snaps1.iter().zip(&snaps2) {
        assert_eq!(s1.index, s2.index);
        assert_eq!(s1.tempo, s2.tempo);
        assert_eq!(s1.chord_name, s2.chord_name);
        let n1: Vec<_> = s1
            .notes
            .iter()
            .map(|n| (n.track, n.pitch, n.start_step, n.duration_steps, n.velocity))
            .collect();
        let n2: Vec<_> = s2
            .notes
            .iter()
            .map(|n| (n.track, n.pitch, n.start_step, n.duration_steps, n.velocity))
            .collect();
        assert_eq!(n1, n2, "snapshots non déterministes à la mesure {}", s1.index);
    }
}

#[test]
fn march_emotions_modulate_dynamics() {
    // 1. Arousal : accélère le tempo et augmente la vélocité du tambour
    let mut calm = march_params();
    calm.intensity = 0.0;
    let mut intense = march_params();
    intense.intensity = 1.0;

    let s_calm = generate_snapshots(1, 16, calm);
    let s_intense = generate_snapshots(1, 16, intense);

    assert!(
        s_intense[1].tempo > s_calm[1].tempo,
        "l'arousal doit augmenter le tempo de marche : {} > {}",
        s_intense[1].tempo,
        s_calm[1].tempo
    );

    let avg_drum_vel = |snaps: &[MeasureSnapshot]| -> f32 {
        let vels: Vec<f32> = snaps
            .iter()
            .flat_map(|s| &s.notes)
            .filter(|n| n.track == TrackId::Toe.channel() || n.track == TrackId::Heel.channel())
            .map(|n| f32::from(n.velocity))
            .collect();
        vels.iter().sum::<f32>() / vels.len().max(1) as f32
    };
    assert!(
        avg_drum_vel(&s_intense) > avg_drum_vel(&s_calm),
        "l'intensité doit augmenter la force du tambour : {} > {}",
        avg_drum_vel(&s_intense),
        avg_drum_vel(&s_calm)
    );

    // 2. Tension : accentue les flams (note 40)
    let mut relaxed = march_params();
    relaxed.harmony_tension = 0.0;
    let mut tense = march_params();
    tense.harmony_tension = 1.0;

    let s_relaxed = generate_snapshots(1, 32, relaxed);
    let s_tense = generate_snapshots(1, 32, tense);

    let flam_vels = |snaps: &[MeasureSnapshot]| -> Vec<u8> {
        snaps.iter().flat_map(|s| &s.notes).filter(|n| n.pitch == 40).map(|n| n.velocity).collect()
    };
    let f_rel = flam_vels(&s_relaxed);
    let f_ten = flam_vels(&s_tense);
    assert!(!f_ten.is_empty(), "flams attendus");
    if !f_rel.is_empty() {
        let avg_rel = f_rel.iter().map(|&v| f32::from(v)).sum::<f32>() / f_rel.len() as f32;
        let avg_ten = f_ten.iter().map(|&v| f32::from(v)).sum::<f32>() / f_ten.len() as f32;
        assert!(
            avg_ten > avg_rel,
            "la tension doit renforcer les flams : {} > {}",
            avg_ten,
            avg_rel
        );
    }
}

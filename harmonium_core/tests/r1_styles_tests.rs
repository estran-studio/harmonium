//! Tests de contrat des trois styles ajoutés par R1 (phase 013), en données seulement :
//! `gigue_acadienne`, `reel_britannique_coloniale` et `valse_britannique_coloniale`.
//!
//! Vérifie sur le flux de snapshots : chargement direct et par nom, métrique et pulse publiés,
//! forme AABB de 32 mesures, notes dans le mode publié, absence de podorythmie des deux profils
//! britanniques, et accompagnement aux pas décrits par chaque profil (piano de la gigue acadienne).

use harmonium_core::{
    StyleProfile,
    params::{MusicalParams, TimeSignature},
    report::MeasureSnapshot,
    trad::{Part, TradGenerator},
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const IDS: [&str; 3] =
    ["gigue_acadienne", "reel_britannique_coloniale", "valse_britannique_coloniale"];

fn profile(id: &str) -> StyleProfile {
    StyleProfile::named(id).expect("le profil doit être distribué")
}

fn snapshots(p: &StyleProfile, seed: u64, bars: usize) -> Vec<MeasureSnapshot> {
    let mut params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        enable_melody: true,
        enable_harmony: true,
        enable_rhythm: true,
        ..Default::default()
    };
    params.set_swing_ratio(p.meter.default_swing_ratio);
    let mut generator = TradGenerator::new(params, TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect()
}

#[test]
fn load_directly_and_by_name() -> Result<(), Box<dyn std::error::Error>> {
    for id in IDS {
        let direct =
            StyleProfile::load(format!("{}/../profiles/{id}", env!("CARGO_MANIFEST_DIR")))?;
        assert_eq!(direct, StyleProfile::named(id)?, "{id}");
        assert_eq!(direct.meta.id, id);
        assert!(
            !direct.meta.tradition.to_lowercase().contains("autochton"),
            "{id} : aucun style autochtone sans consultation (principe VII)"
        );
        assert!(direct.meta.corpus.contains("harmonium_lab"), "{id} : provenance du corpus");
    }
    Ok(())
}

#[test]
fn meter_pulse_and_tempo_are_published() {
    // (id, signature, pas par mesure, pas par pulse, pulses par mesure, swing)
    let expected = [
        ("gigue_acadienne", (6, 8), 12, 6, 2, 0.5),
        ("reel_britannique_coloniale", (2, 2), 16, 8, 2, 0.5),
        ("valse_britannique_coloniale", (3, 4), 12, 4, 3, 0.5),
    ];
    for (id, (num, den), steps, per_pulse, pulses, swing) in expected {
        let p = profile(id);
        assert_eq!(p.meter.time_signature, TimeSignature::new(num, den), "{id}");
        assert_eq!(p.meter.default_swing_ratio, swing, "{id}");
        assert!(p.meter.min_bpm <= p.meter.default_bpm && p.meter.default_bpm <= p.meter.max_bpm);
        for s in snapshots(&p, 1, 32) {
            assert_eq!((s.time_sig_numerator, s.time_sig_denominator), (num, den), "{id}");
            assert_eq!(s.steps, steps, "{id}");
            assert_eq!(
                (s.pulse.steps_per_pulse, s.pulse.pulses_per_bar),
                (per_pulse, pulses),
                "{id}"
            );
            assert!((s.tempo - p.meter.default_bpm).abs() < 0.1, "{id}");
        }
    }
}

#[test]
fn form_is_aabb_of_thirty_two_bars_with_repeats_in_place() {
    for id in IDS {
        let p = profile(id);
        assert_eq!(p.form.sequence, vec![0, 0, 1, 1], "{id}");
        assert_eq!(p.form.bars_per_tour(), 32, "{id}");
        let tour: Vec<_> = snapshots(&p, 42, 65)
            .iter()
            .filter_map(|s| s.position)
            .filter(|pos| !pos.is_intro())
            .collect();
        assert_eq!((tour[0].part, tour[0].pass), (Part(0), 1), "{id}");
        assert_eq!((tour[8].part, tour[8].pass), (Part(0), 2), "{id}");
        assert_eq!((tour[16].part, tour[16].pass), (Part(1), 1), "{id}");
        assert_eq!((tour[24].part, tour[24].pass), (Part(1), 2), "{id}");
    }
}

#[test]
fn melody_stays_in_the_published_mode_and_tessitura() {
    for id in IDS {
        let p = profile(id);
        for seed in 1..=4 {
            for s in snapshots(&p, seed, 64) {
                let Some(pos) = s.position else { continue };
                for note in s.notes.iter().filter(|n| n.track == 0) {
                    assert!(
                        pos.mode.contains_midi(pos.key, note.pitch),
                        "{id} seed {seed} : {note:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn british_profiles_have_no_feet_but_the_acadian_jig_has() {
    for id in ["reel_britannique_coloniale", "valse_britannique_coloniale"] {
        let p = profile(id);
        for s in snapshots(&p, 1, 32) {
            assert!(
                s.notes.iter().all(|n| n.track != 3 && n.track != 4),
                "{id} : aucune podorythmie attendue"
            );
        }
    }
    // Les pieds de la gigue acadienne entrent au tour 2 (arrangement) : on lit leur table.
    let p = profile("gigue_acadienne");
    let heel_steps: Vec<_> = p
        .percussion
        .patterns
        .iter()
        .find(|f| f.density == 0)
        .map_or_else(Vec::new, |f| f.hits.iter().map(|hit| hit.step).collect());
    assert_eq!(heel_steps, vec![0, 6], "talon sur les pulses 1 et 4");
}

#[test]
fn accompaniment_follows_each_profile() {
    let jig = profile("gigue_acadienne");
    assert_eq!((jig.instruments.chords_program, jig.instruments.bass_program), (Some(0), Some(0)));
    for s in snapshots(&jig, 1, 32) {
        let steps: Vec<_> = s.notes.iter().filter(|n| n.track == 1).map(|n| n.start_step).collect();
        assert!(steps.iter().all(|step| [2, 4, 8, 10].contains(step)), "piano : accords {steps:?}");
    }
    let reel = profile("reel_britannique_coloniale");
    for s in snapshots(&reel, 1, 32) {
        let steps: Vec<_> = s.notes.iter().filter(|n| n.track == 1).map(|n| n.start_step).collect();
        assert!(
            steps.iter().all(|step| *step == 0 || *step == 8),
            "reel : accords tenus {steps:?}"
        );
    }
    let waltz = profile("valse_britannique_coloniale");
    for s in snapshots(&waltz, 1, 32) {
        let steps: Vec<_> = s.notes.iter().filter(|n| n.track == 1).map(|n| n.start_step).collect();
        assert!(
            steps.iter().all(|step| *step == 4 || *step == 8),
            "valse : accords sur 2 et 3 {steps:?}"
        );
    }
}

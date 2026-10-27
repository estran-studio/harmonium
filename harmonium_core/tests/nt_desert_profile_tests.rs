//! Profil distribué `nt_desert` (phase 014, G7) : désert de Nuclear Throne.
//! Données seulement ; le contrat est observé dans les snapshots et le
//! lecteur public, jamais dans un état privé.
use std::collections::{BTreeMap, BTreeSet};

use harmonium_core::{
    StyleProfile, TuningParams,
    events::AudioEvent,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::{Measure, Playhead},
    trad::TradGenerator,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn load() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!("{}/../profiles/nt_desert", env!("CARGO_MANIFEST_DIR")))?)
}

fn params(p: &StyleProfile) -> MusicalParams {
    MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        ..Default::default()
    }
}

fn stream(p: &StyleProfile, seed: u64, bars: usize) -> TestResult<Vec<MeasureSnapshot>> {
    p.validate()?;
    let mut g = TradGenerator::new(params(p), TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect())
}

fn notes(s: &MeasureSnapshot, track: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == track).collect()
}

#[test]
fn charge_par_nom_et_contenu() -> TestResult {
    let direct = load()?;
    let named = StyleProfile::named("nt_desert")?;
    assert_eq!(direct, named);
    assert_eq!(named.meta.id, "nt_desert");
    assert_eq!(named.meta.tradition, "Nuclear Throne, désert — Jukio Kallio, 2015");
    assert!(named.meta.corpus.contains("corpus/nuclear_throne/manifest.jsonl"));
    assert_eq!(
        (named.meter.time_signature.numerator, named.meter.time_signature.denominator),
        (12, 8)
    );
    assert_eq!(named.meter.steps_per_bar, 24);
    assert_eq!(
        (named.meter.steps_per_pulse, named.meter.pulses_per_bar),
        (6, 4),
        "quatre noires pointées par mesure"
    );
    assert_eq!(named.meter.default_bpm, 210.0, "140 noires pointées par minute");
    let instruments = &named.instruments;
    assert_eq!(instruments.melody_program, 78, "sifflement");
    assert_eq!(instruments.chords_program, Some(81), "scie tenue, décision d'écoute");
    assert_eq!(instruments.bass_program, Some(81), "scie tenue, décision d'écoute");
    assert_eq!((instruments.percussion_bank, instruments.percussion_program), (128, 0));
    // Le sifflement n'entre qu'au pont (partie 2), après 16 mesures (G4).
    let entries = &named.arrangement.entries;
    assert_eq!(entries.melody.map(|e| (e.tour, e.part.0, e.pass)), Some((1, 2, 1)));
    Ok(())
}

#[test]
fn arche_48_melodie_tardive() -> TestResult {
    let p = load()?;
    for seed in 1..=5 {
        let snapshots = stream(&p, seed, 64)?;
        assert_eq!(snapshots.len(), 64);
        for (i, s) in snapshots.iter().enumerate() {
            let bar = i + 1;
            assert!(!s.notes.is_empty(), "mesure {bar} vide (graine {seed})");
            assert_eq!((s.time_sig_numerator, s.time_sig_denominator), (12, 8));
        }
        // A1 + A2 (mes. 1–16) : pas de sifflement ; basse, scie et batterie jouent.
        for s in &snapshots[0..16] {
            assert!(notes(s, 0).is_empty(), "sifflement avant le pont, mesure {}", s.index);
            assert!(!notes(s, 1).is_empty());
            assert!(!notes(s, 2).is_empty());
        }
        // Le sifflement est entré au pont (mes. 17–32) et la scie y tient des triades.
        assert!(
            snapshots[16..32].iter().any(|s| !notes(s, 0).is_empty()),
            "sifflement absent du pont"
        );
        for s in &snapshots[16..32] {
            assert_eq!(notes(s, 1).len(), 3, "triade tenue, mesure {}", s.index);
        }
        // Batterie présente dans tout l'air, pont compris : le moteur n'a pas
        // de sourdine par partie en données (écart documenté au compte rendu G7).
        for s in &snapshots[0..48] {
            assert!(!notes(s, 3).is_empty(), "grosse caisse absente, mesure {}", s.index);
            assert!(!notes(s, 4).is_empty(), "caisse claire absente, mesure {}", s.index);
        }
        // Reprise A1' (mes. 41–48) : pédale et relance comme en A1.
        for (a, b) in [(41, 1), (44, 4), (48, 8)] {
            let (pa, pb) = (&snapshots[a - 1], &snapshots[b - 1]);
            let pitches = |s: &MeasureSnapshot| {
                notes(s, 2).iter().map(|n| (n.pitch, n.start_step)).collect::<Vec<_>>()
            };
            assert_eq!(pitches(pa), pitches(pb), "reprise A1' mesure {a} vs A1 mesure {b}");
        }
    }
    Ok(())
}

#[test]
fn pedale_liee_une_attaque_sans_orpheline() -> TestResult {
    let p = load()?;
    let signature = p.meter.time_signature;
    let mut g = TradGenerator::new(params(&p), TuningParams { profile: p });
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let mut measures: Vec<_> = (1..=8).map(|b| g.generate_measure(b, &mut rng)).collect();
    measures.push(Measure::new(9, signature, 210.0, 24));
    let mut reader = Playhead::new(44100.0, 4);
    let mut bass_attacks = Vec::new();
    let mut chords_attacks = Vec::new();
    let mut balance = BTreeMap::<(u8, u8), i32>::new();
    for mut m in measures {
        let bar = m.index;
        m.prepare_playback();
        reader.load_measure(Box::new(m));
        for step in 0..24 {
            reader.tick(None);
            for i in 0.. {
                let Some(ev) = reader.event(i) else { break };
                match &ev.event {
                    AudioEvent::NoteOn { channel, note, .. } => {
                        *balance.entry((*channel, *note)).or_default() += 1;
                        if (*channel, *note) == (2, 32) {
                            bass_attacks.push((bar, step));
                        }
                        if (*channel, *note) == (1, 44) {
                            chords_attacks.push((bar, step));
                        }
                    }
                    AudioEvent::NoteOff { channel, note } => {
                        *balance.entry((*channel, *note)).or_default() -= 1;
                    }
                    _ => {}
                }
            }
        }
        let _ = reader.take_finished();
    }
    // Pédale G#1 : une attaque pour les mesures 1–3 ; la relance mes. 4/8 ne
    // touche le G# qu'à sa résolution (pas 11), sans réattaque aux barres.
    assert_eq!(bass_attacks, vec![(1, 0), (4, 11), (8, 11)], "{bass_attacks:?}");
    // Scie G#2 : une attaque pour les huit mesures, liée sans réattaque.
    assert_eq!(chords_attacks, vec![(1, 0)], "{chords_attacks:?}");
    assert!(balance.values().all(|&v| v == 0), "notes orphelines : {balance:?}");
    Ok(())
}

#[test]
fn batterie_grille_6_2() -> TestResult {
    let p = load()?;
    let snapshots = stream(&p, 1, 16)?;
    for s in &snapshots {
        let heel: BTreeSet<usize> = notes(s, 3).iter().map(|n| n.start_step).collect();
        assert!(
            heel.is_subset(&BTreeSet::from([0, 8, 20])),
            "grosse caisse mes. {} : {heel:?}",
            s.index
        );
        assert!(!heel.is_empty());
        for n in notes(s, 4) {
            if n.pitch == 38 {
                assert!(
                    [2, 10, 14].contains(&n.start_step),
                    "caisse claire mes. {} : {n:?}",
                    s.index
                );
            } else {
                assert_eq!(n.pitch, 42, "charleston mes. {} : {n:?}", s.index);
                assert_eq!(n.start_step % 2, 0);
            }
        }
    }
    // Groove principal exact à densité neutre (mesure 1).
    let heel: BTreeSet<usize> = notes(&snapshots[0], 3).iter().map(|n| n.start_step).collect();
    assert_eq!(heel, BTreeSet::from([0, 8, 20]));
    let snare: BTreeSet<usize> =
        notes(&snapshots[0], 4).iter().filter(|n| n.pitch == 38).map(|n| n.start_step).collect();
    assert_eq!(snare, BTreeSet::from([2, 10, 14]));
    let hat: BTreeSet<usize> =
        notes(&snapshots[0], 4).iter().filter(|n| n.pitch == 42).map(|n| n.start_step).collect();
    assert_eq!(hat, (0..24).step_by(2).collect());
    Ok(())
}

#[test]
fn presets_banque_uniquement() -> TestResult {
    let p = load()?;
    let snapshots = stream(&p, 1, 48)?;
    for s in &snapshots {
        for n in &s.notes {
            assert!((0..=4).contains(&n.track), "pas de seconde voix : {n:?}");
            if n.track == 3 {
                assert_eq!(n.pitch, 36, "grosse caisse : {n:?}");
            } else if n.track == 4 {
                assert!([38, 42].contains(&n.pitch), "caisse claire ou charleston : {n:?}");
            }
        }
    }
    Ok(())
}

#[test]
fn density_basse_ne_laisse_que_la_scie() -> TestResult {
    let p = load()?;
    p.validate()?;
    let mut par = params(&p);
    par.rhythm_density = 0.2;
    let mut g = TradGenerator::new(par, TuningParams { profile: p });
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let snapshots: Vec<_> = (1..=48)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect();
    // Calme façon A Way : sifflement, basse et batterie se taisent, la scie reste.
    for s in &snapshots {
        assert!(notes(s, 0).is_empty(), "mesure {}", s.index);
        assert!(notes(s, 2).is_empty(), "mesure {}", s.index);
        assert!(notes(s, 3).is_empty(), "mesure {}", s.index);
        assert!(notes(s, 4).is_empty(), "mesure {}", s.index);
        assert!(!notes(s, 1).is_empty(), "mesure {}", s.index);
    }
    Ok(())
}

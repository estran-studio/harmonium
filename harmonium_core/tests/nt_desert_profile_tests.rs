//! Profil distribué `nt_desert` (phase 014, G7 puis G7c) : désert de Nuclear Throne.
//! Données seulement ; le contrat est observé dans les snapshots et le
//! lecteur public, jamais dans un état privé.
use std::collections::{BTreeMap, BTreeSet};

use harmonium_core::{
    StyleProfile, TuningParams,
    events::AudioEvent,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::{Articulation, Measure, Playhead},
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
    // G7c : le sifflement est retiré (pas de lead dans Fläshyn) ; le programme
    // 78 reste déclaré car le schéma l'exige, mais ne sonne jamais.
    assert_eq!(instruments.melody_program, 78, "sifflement déclaré, inutilisé");
    assert_eq!(instruments.chords_program, Some(89), "nappe chaude, essai G7c");
    assert_eq!(instruments.bass_program, Some(81), "scie grave, décision d'écoute");
    assert_eq!((instruments.percussion_bank, instruments.percussion_program), (128, 0));
    // La mélodie n'a aucune entrée d'arrangement : elle ne joue jamais (G4).
    assert!(named.arrangement.entries.melody.is_none(), "sifflement retiré (G7c)");
    Ok(())
}

#[test]
fn arche_48_sans_melodie() -> TestResult {
    let p = load()?;
    for seed in 1..=5 {
        let snapshots = stream(&p, seed, 64)?;
        assert_eq!(snapshots.len(), 64);
        for (i, s) in snapshots.iter().enumerate() {
            let bar = i + 1;
            assert!(!s.notes.is_empty(), "mesure {bar} vide (graine {seed})");
            assert_eq!((s.time_sig_numerator, s.time_sig_denominator), (12, 8));
            // G7c : aucun sifflement nulle part dans le désert A.
            assert!(notes(s, 0).is_empty(), "sifflement, mesure {} (graine {seed})", s.index);
        }
        // A1 (mes. 1–8) : pédale seule sur la nappe, basse, scie et batterie.
        for s in &snapshots[0..8] {
            assert_eq!(notes(s, 1).len(), 1, "pédale seule, mesure {}", s.index);
            assert!(!notes(s, 2).is_empty());
        }
        // A2 (mes. 9–16) : triades tenues sur la nappe.
        for s in &snapshots[8..16] {
            assert_eq!(notes(s, 1).len(), 3, "triade tenue, mesure {}", s.index);
        }
        // Le pont (mes. 17–32) tient des triades en tessiture médium.
        for s in &snapshots[16..32] {
            assert_eq!(notes(s, 1).len(), 3, "triade tenue, mesure {}", s.index);
            for n in notes(s, 1) {
                assert!(n.pitch >= 48, "nappe dans le grave, mesure {} : {n:?}", s.index);
            }
        }
        // Pont sans batterie ni basse (G7b, comme l'original) : sourdine par
        // partie en données ; la nappe reste seule.
        for s in &snapshots[16..32] {
            assert!(notes(s, 2).is_empty(), "basse au pont, mesure {}", s.index);
            assert!(notes(s, 3).is_empty(), "grosse caisse au pont, mesure {}", s.index);
            assert!(notes(s, 4).is_empty(), "pieds au pont, mesure {}", s.index);
            assert!(!notes(s, 1).is_empty(), "nappe absente au pont, mesure {}", s.index);
        }
        // Batterie et basse présentes avant (A1, A2) et après (B, A1') le pont :
        // basse à chaque mesure en A1/A2/A1', une mesure sur deux en B
        // (données : quasi-absence voulue).
        for s in snapshots[0..16].iter().chain(&snapshots[40..48]) {
            assert!(!notes(s, 2).is_empty(), "basse absente, mesure {}", s.index);
        }
        for s in &snapshots[32..48] {
            assert!(!notes(s, 3).is_empty(), "grosse caisse absente, mesure {}", s.index);
            assert!(
                notes(s, 4).iter().any(|n| n.pitch == 42),
                "charleston absent, mesure {}",
                s.index
            );
        }
        assert!(
            snapshots[32..40].iter().any(|s| !notes(s, 2).is_empty()),
            "basse absente de tout B"
        );
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
                        if (*channel, *note) == (1, 56) {
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
    // Nappe G#3 (G7c : tessiture médium, deux octaves au-dessus de la basse) :
    // une attaque pour les huit mesures, liée sans réattaque.
    assert_eq!(chords_attacks, vec![(1, 0)], "{chords_attacks:?}");
    assert!(balance.values().all(|&v| v == 0), "notes orphelines : {balance:?}");
    Ok(())
}

#[test]
fn batterie_groove_pulsations_accents() -> TestResult {
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
                // G7c : charleston sur les pulsations seulement, plus aucun
                // coup aux pas pairs intermédiaires.
                assert_eq!(n.pitch, 42, "charleston mes. {} : {n:?}", s.index);
                assert!(
                    [0, 6, 12, 18].contains(&n.start_step),
                    "charleston hors pulsation mes. {} : {n:?}",
                    s.index
                );
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
    assert_eq!(hat, BTreeSet::from([0, 6, 12, 18]));
    // Accents par temps (G7c, essai du propriétaire) : le temps fort sonne
    // plein et porte l'étiquette d'accent, les autres temps sont à ×0,8.
    let velocity = |pitch: u8, step: usize| {
        notes(&snapshots[0], if pitch == 36 { 3 } else { 4 })
            .iter()
            .find(|n| n.pitch == pitch && n.start_step == step)
            .map(|n| (n.velocity, n.articulation))
    };
    let (fort, articulation) = velocity(36, 0).expect("grosse caisse au temps fort");
    assert!(matches!(articulation, Articulation::Accent), "temps fort non accentué");
    for step in [8, 20] {
        let (faible, articulation) = velocity(36, step).expect("grosse caisse à accentuer");
        // Hors temps fort : déclenchement brut, sans étiquette d'accent.
        assert!(matches!(articulation, Articulation::Trigger), "accent hors temps fort");
        assert!(fort > faible, "temps fort {fort} <= temps {step} {faible}");
    }
    let (fort, _) = velocity(42, 0).expect("charleston au temps fort");
    for step in [6, 12, 18] {
        let (faible, _) = velocity(42, step).expect("charleston à accentuer");
        assert!(fort > faible, "charleston fort {fort} <= temps {step} {faible}");
    }
    // Micro-timing (G7c) : des décalages légers sont publiés sur les pieds,
    // bornés à ±0,02 pas.
    let mut decales = 0;
    for n in notes(&snapshots[0], 3).into_iter().chain(notes(&snapshots[0], 4)) {
        assert!((-20..=20).contains(&n.performance.offset), "décalage hors ±0,02 pas : {n:?}");
        decales += i32::from(n.performance.offset != 0);
    }
    assert!(decales > 0, "micro-timing nul sur toute la mesure 1");
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
fn density_basse_ne_laisse_que_la_nappe() -> TestResult {
    let p = load()?;
    p.validate()?;
    let mut par = params(&p);
    par.rhythm_density = 0.2;
    let mut g = TradGenerator::new(par, TuningParams { profile: p });
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let snapshots: Vec<_> = (1..=48)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect();
    // Calme façon A Way : basse et batterie se taisent, la nappe reste
    // (le sifflement est retiré en G7c).
    for s in &snapshots {
        assert!(notes(s, 0).is_empty(), "mesure {}", s.index);
        assert!(notes(s, 2).is_empty(), "mesure {}", s.index);
        assert!(notes(s, 3).is_empty(), "mesure {}", s.index);
        assert!(notes(s, 4).is_empty(), "mesure {}", s.index);
        assert!(!notes(s, 1).is_empty(), "mesure {}", s.index);
    }
    Ok(())
}

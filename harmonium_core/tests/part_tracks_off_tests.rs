//! Sourdine par partie (`form.parts[].tracks_off`, phase 014 G7b) : couper des
//! pistes sur une partie de la forme, en données. Le contrat est observé dans
//! les snapshots et le lecteur public, jamais dans un état privé.
use std::collections::BTreeMap;

use harmonium_core::{
    StyleProfile, TuningParams,
    events::AudioEvent,
    params::MusicalParams,
    report::MeasureSnapshot,
    style_profile::ProfilePart,
    timeline::{Measure, Playhead, TrackId},
    trad::{
        TradGenerator,
        accomp_profile::{BassMotif, BassMotifNote},
    },
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn load_distribué() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!("{}/../profiles/nt_desert", env!("CARGO_MANIFEST_DIR")))?)
}

fn load_fixture(id: &str) -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/{id}",
        env!("CARGO_MANIFEST_DIR")
    ))?)
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

fn notes(s: &MeasureSnapshot, track: TrackId) -> usize {
    s.notes.iter().filter(|n| n.track == track as u8).count()
}

#[test]
fn pont_muette_batterie_basse_puis_reprend_sans_orpheline() -> TestResult {
    let p = load_distribué()?;
    p.validate()?;
    let signature = p.meter.time_signature;
    let steps = p.meter.steps_per_bar;
    let mut g = TradGenerator::new(params(&p), TuningParams { profile: p });
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    // Mesures 1–40 (A1, A2, pont, B) puis une mesure vide qui purge les fins
    // en vol, comme le garde-fou des pédales liées.
    let mut measures: Vec<_> = (1..=40).map(|b| g.generate_measure(b, &mut rng)).collect();
    measures.push(Measure::new(41, signature, 210.0, steps));
    let mut reader = Playhead::new(44100.0, 4);
    let mut balance = BTreeMap::<(u8, u8), i32>::new();
    let mut on_par_mesure = BTreeMap::<usize, Vec<u8>>::new();
    for mut m in measures {
        let bar = m.index;
        m.prepare_playback();
        reader.load_measure(Box::new(m));
        for _ in 0..steps {
            reader.tick(None);
            for i in 0.. {
                let Some(ev) = reader.event(i) else { break };
                match &ev.event {
                    AudioEvent::NoteOn { channel, note, .. } => {
                        *balance.entry((*channel, *note)).or_default() += 1;
                        on_par_mesure.entry(bar).or_default().push(*channel);
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
    // Pont (mes. 17–32) : aucune attaque basse ni pieds. La scie liée y
    // tient sans réattaque : sa présence est prouvée au snapshot (G7).
    for bar in 17..=32 {
        let attaques = on_par_mesure.get(&bar).cloned().unwrap_or_default();
        assert!(
            !attaques.contains(&2) && !attaques.contains(&3) && !attaques.contains(&4),
            "attaque basse ou pieds au pont, mesure {bar} : {attaques:?}"
        );
    }
    // Reprise en B (mes. 33–40) : la basse et les pieds rejouent.
    let b: Vec<u8> = (33..=40)
        .filter_map(|bar| on_par_mesure.get(&bar))
        .flat_map(|v| v.iter().copied())
        .collect();
    assert!(b.contains(&2), "basse absente de tout B");
    assert!(b.contains(&3), "grosse caisse absente de tout B");
    assert!(b.contains(&4), "pieds absents de tout B");
    // Les tenues qui traversaient la frontière se sont fermées proprement.
    assert!(balance.values().all(|&v| v == 0), "notes orphelines : {balance:?}");
    Ok(())
}

#[test]
fn gigue_bourdon_partie_b_sans_pieds() -> TestResult {
    // Principe VIII : le même mécanisme dans un autre style (gigue 6/8).
    let p = load_fixture("gigue_bourdon")?;
    let snapshots = stream(&p, 1, 32)?;
    // Deux tours de 16 mesures : A = 1–8 et 17–24, B = 9–16 et 25–32.
    for (tour, debut) in [0, 16].iter().enumerate() {
        let a = &snapshots[*debut..*debut + 8];
        let b = &snapshots[*debut + 8..*debut + 16];
        assert!(
            a.iter().any(|s| notes(s, TrackId::Heel) + notes(s, TrackId::Toe) > 0),
            "pieds absents de tout A (tour {})",
            tour + 1
        );
        for s in b {
            assert_eq!(notes(s, TrackId::Heel), 0, "talon en B, mesure {}", s.index);
            assert_eq!(notes(s, TrackId::Toe), 0, "pointe en B, mesure {}", s.index);
            assert!(!s.notes.is_empty(), "mesure B vide {}", s.index);
        }
    }
    Ok(())
}

#[test]
fn validation_piste_inconnue_doublon_et_motif_sans_emploi() -> TestResult {
    // Piste inconnue : refusée au chargement avec le champ fautif.
    let err = toml::from_str::<ProfilePart>(
        "name = \"A\"\nroles = []\nmelody_center_offset = 0\ntracks_off = [\"Violon\"]",
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("tracks_off"), "champ fautif absent : {err}");
    // Doublon : refusé à la validation.
    let mut p = load_fixture("gigue_bourdon")?;
    p.form.parts[1].tracks_off.push(TrackId::Heel);
    assert_eq!(p.validate().unwrap_err().field, "form.parts[1].tracks_off");
    // Partie inexistante : un motif qui la nomme est refusé (garde existante).
    let mut p = load_distribué()?;
    let motif = || BassMotif {
        bars: 4,
        weight: 1,
        part: Some(String::new()),
        reference: Default::default(),
        notes: vec![BassMotifNote {
            step: 0,
            duration: 2,
            semitones: Some(0),
            degree: None,
            octave: 0,
            velocity: 45,
            tie: false,
            legato: false,
            doubling: Vec::new(),
        }],
    };
    let mut inconnu = motif();
    inconnu.part = Some("Zzz".to_string());
    let n = p.accompaniment.as_ref().ok_or("accompagnement absent")?.bass.motifs.len();
    p.accompaniment.as_mut().ok_or("accompagnement absent")?.bass.motifs.push(inconnu);
    assert_eq!(p.validate().unwrap_err().field, format!("accompaniment.bass.motifs[{n}].part"));
    // Motif dédié à une partie qui tait la basse : sans emploi, refusé.
    let mut p = load_distribué()?;
    let n = p.accompaniment.as_ref().ok_or("accompagnement absent")?.bass.motifs.len();
    let mut inutile = motif();
    inutile.part = Some("Pont".to_string());
    p.accompaniment.as_mut().ok_or("accompagnement absent")?.bass.motifs.push(inutile);
    assert_eq!(p.validate().unwrap_err().field, format!("accompaniment.bass.motifs[{n}].part"));
    // Le pont sans motif dédié est valide : la partie muette est exemptée.
    let p = load_distribué()?;
    assert!(
        p.accompaniment
            .as_ref()
            .ok_or("accompagnement absent")?
            .bass
            .motifs
            .iter()
            .all(|m| { m.part.as_deref() != Some("Pont") }),
        "le pont ne doit porter aucun motif"
    );
    p.validate()?;
    Ok(())
}

#[test]
fn gigue_bourdon_pieds_micro_timing_publie() -> TestResult {
    // Principe VIII (014-G7c) : le micro-timing des pieds employé par
    // `nt_desert` vit aussi dans un autre style (gigue 6/8), en données.
    let p = load_fixture("gigue_bourdon")?;
    p.validate()?;
    let snapshots = stream(&p, 1, 32)?;
    let mut pieds = 0;
    let mut decales = 0;
    for s in snapshots.iter().take(8) {
        for n in &s.notes {
            if n.track == TrackId::Heel as u8 || n.track == TrackId::Toe as u8 {
                pieds += 1;
                assert!(
                    (-20..=20).contains(&n.performance.offset),
                    "décalage hors ±0,02 pas, mesure {} : {n:?}",
                    s.index
                );
                decales += i32::from(n.performance.offset != 0);
            }
        }
    }
    assert!(pieds > 0, "aucun coup de pieds en A");
    assert!(decales > 0, "micro-timing nul sur tous les pieds");
    Ok(())
}

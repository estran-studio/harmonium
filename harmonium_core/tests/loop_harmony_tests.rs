//! Fixtures techniques G2 : aucune validation stylistique à l'écoute revendiquée.
//! Le contrat est observé exclusivement dans les snapshots et le lecteur public.
use std::collections::{BTreeMap, BTreeSet};

use harmonium_core::{
    StyleProfile, TuningParams,
    events::AudioEvent,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::{Articulation, Measure, Playhead},
    trad::{Mode, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn load(id: &str) -> TestResult<StyleProfile> {
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

fn notes(s: &MeasureSnapshot, track: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == track).collect()
}

fn pcs(s: &MeasureSnapshot, track: u8) -> BTreeSet<u8> {
    notes(s, track).iter().map(|n| n.pitch % 12).collect()
}

fn expected_nt(bar: usize, key: u8, mode: Mode) -> (i32, bool, BTreeSet<u8>) {
    let (offset, minor, intervals): (u8, bool, &[u8]) = match (bar - 1) % 12 {
        0..=3 => (0, false, &[0]),
        4..=5 => (11, false, &[0, 4, 7]),
        6..=7 => (0, false, &[0, 4, 7]),
        8..=9 => (0, true, &[0, 3, 7]),
        _ => (if mode == Mode::Phrygian { 1 } else { 2 }, false, &[0, 4, 7]),
    };
    (i32::from(offset), minor, intervals.iter().map(|i| (key + offset + i) % 12).collect())
}

fn check_nt(s: &MeasureSnapshot, key: u8, mode: Mode) {
    let (offset, minor, expected) = expected_nt(s.index, key, mode);
    assert_eq!((s.chord_root_offset, s.chord_is_minor), (offset, minor), "mesure {}", s.index);
    assert_eq!(pcs(s, 1), expected, "accord mesure {}", s.index);
    let root = (key + offset as u8) % 12;
    let bass = if (s.index - 1) % 2 == 0 {
        vec![(0, 3, root), (8, 1, (root + 1) % 12), (10, 3, root)]
    } else {
        vec![(0, 3, root), (8, 3, (root + 7) % 12)]
    };
    assert_eq!(
        notes(s, 2)
            .iter()
            .map(|n| (n.start_step, n.duration_steps, n.pitch % 12))
            .collect::<Vec<_>>(),
        bass,
        "motif G3 transposé mesure {}",
        s.index
    );
    assert!(notes(s, 0).is_empty());
    assert_eq!(notes(s, 1).len(), expected.len());
    for n in notes(s, 1) {
        assert_eq!((n.start_step, n.duration_steps, n.articulation), (0, 16, Articulation::Tied));
        assert!((28..=55).contains(&n.pitch), "registre grave : {n:?}");
    }
}

#[test]
fn nt_pedale_et_boucles_exactes_repetables() -> TestResult {
    let p = load("nt_pedale_boucle")?;
    for seed in [1, 2, 17] {
        let snapshots = stream(&p, seed, 72)?;
        for s in &snapshots {
            assert_eq!(s.position.map(|p| p.mode), Some(Mode::Phrygian));
            check_nt(s, 8, Mode::Phrygian);
        }
        assert_eq!(serde_json::to_value(&snapshots)?, serde_json::to_value(stream(&p, seed, 72)?)?);
    }
    Ok(())
}

#[test]
fn pedale_une_attaque_puis_fin_equilibree() -> TestResult {
    let p = load("nt_pedale_boucle")?;
    let signature = p.meter.time_signature;
    let mut g = TradGenerator::new(params(&p), TuningParams { profile: p });
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let mut measures: Vec<_> = (1..=4).map(|b| g.generate_measure(b, &mut rng)).collect();
    measures.push(Measure::new(5, signature, 120.0, 16));
    let mut reader = Playhead::new(44100.0, 4);
    let mut balance = BTreeMap::<u8, i32>::new();
    let mut attacks = Vec::new();
    for mut m in measures {
        let bar = m.index;
        m.prepare_playback();
        reader.load_measure(Box::new(m));
        for step in 0..16 {
            reader.tick(None);
            for i in 0.. {
                let Some(ev) = reader.event(i) else { break };
                match &ev.event {
                    AudioEvent::NoteOn { channel: 1, note, .. } => {
                        attacks.push((bar, step, *note));
                        *balance.entry(*note).or_default() += 1;
                    }
                    AudioEvent::NoteOff { channel: 1, note } => {
                        *balance.entry(*note).or_default() -= 1;
                    }
                    _ => {}
                }
            }
        }
        let _ = reader.take_finished();
    }
    assert_eq!(attacks.len(), 1, "une attaque sur quatre mesures : {attacks:?}");
    assert_eq!((attacks[0].0, attacks[0].1, attacks[0].2 % 12), (1, 0, 8));
    assert_eq!(balance.values().copied().collect::<Vec<_>>(), vec![0]);
    Ok(())
}

fn edited(
    p: &StyleProfile,
    change: impl FnOnce(&mut toml::Value) -> TestResult,
) -> TestResult<StyleProfile> {
    let mut value: toml::Value = toml::from_str(&p.to_toml()?)?;
    change(&mut value)?;
    Ok(StyleProfile::from_toml(&toml::to_string(&value)?)?)
}

fn insert(value: &mut toml::Value, key: &str, field: impl Into<toml::Value>) -> TestResult {
    value.as_table_mut().ok_or("table TOML requise")?.insert(key.into(), field.into());
    Ok(())
}

#[test]
fn transposition_publique_et_voisin_chromatique_optionnel() -> TestResult {
    let p = load("nt_pedale_boucle")?;
    for key in [0, 3, 11] {
        let transposed = edited(&p, |v| {
            v["melody"]["key_modes"][0]["key"] = key.into();
            v["dynamics"]["color"][0]["key"] = key.into();
            Ok(())
        })?;
        for s in stream(&transposed, 1, 24)? {
            check_nt(&s, key as u8, Mode::Phrygian);
        }
    }
    let neighbor =
        edited(&p, |v| insert(&mut v["harmony"]["parts"]["Pedale"], "neighbor_semitones", 1))?;
    for s in stream(&neighbor, 1, 4)? {
        assert_eq!(pcs(&s, 1), BTreeSet::from([8, 9]));
        assert_eq!(notes(&s, 1).len(), 2);
        let chord = notes(&s, 1);
        assert_eq!(chord[1].pitch - chord[0].pitch, 1, "b2 juste au-dessus de la pédale");
        assert_eq!(chord[0].pitch % 12, 8, "la fondamentale reste la voix grave");
        assert_eq!(notes(&s, 2)[0].pitch % 12, 8);
    }
    let fixed = edited(&neighbor, |v| {
        v["accompaniment"]["chords"]["voicing"] = "fixed_triad".into();
        v["accompaniment"]["chords"]["low"] = 36.into();
        v["accompaniment"]["chords"]["high"] = 48.into();
        Ok(())
    })?;
    for s in stream(&fixed, 1, 4)? {
        let chord = notes(&s, 1);
        assert_eq!(chord.iter().map(|n| n.pitch).collect::<Vec<_>>(), [44, 45]);
    }
    let mut too_narrow = fixed;
    too_narrow.accompaniment.as_mut().ok_or("accompagnement absent")?.chords.high = 47;
    assert_eq!(
        too_narrow.validate().err().ok_or("fenêtre trop étroite acceptée")?.field,
        "accompaniment.chords.high"
    );
    Ok(())
}

#[test]
fn valence_attend_frontiere_et_recalcule_les_degres_modaux() -> TestResult {
    let p = load("nt_pedale_boucle")?;
    let mut par = params(&p);
    let mut g = TradGenerator::new(par.clone(), TuningParams { profile: p });
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let mut seconds = BTreeSet::new();
    for bar in 1..=48 {
        if bar == 10 || bar == 22 {
            par.harmony_valence = if bar == 10 { 0.9 } else { -0.9 };
            g.invalidate_future(bar - 1);
            g.update_params(par.clone());
        }
        let s = MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng));
        let mode = if (13..=24).contains(&bar) { Mode::Mixolydian } else { Mode::Phrygian };
        assert_eq!(s.position.map(|p| p.mode), Some(mode), "mesure {bar}");
        check_nt(&s, 8, mode);
        if (bar - 1) % 12 >= 10 {
            seconds.insert(s.chord_root_offset);
        }
    }
    assert_eq!(seconds, BTreeSet::from([1, 2]));
    Ok(())
}

#[test]
fn validation_publique_refuse_les_parties_et_boucles_invalides() -> TestResult {
    let p = load("nt_pedale_boucle")?;
    let base: toml::Value = toml::from_str(&p.to_toml()?)?;
    let reject = |v: toml::Value| -> TestResult {
        let text = toml::to_string(&v)?;
        assert!(StyleProfile::from_toml(&text).is_err(), "profil invalide accepté");
        Ok(())
    };
    let mut v = base.clone();
    v["harmony"]["parts"].as_table_mut().ok_or("table parts requise")?.remove("Pedale");
    reject(v)?;
    let mut v = base.clone();
    let unknown = v["harmony"]["parts"]["Pedale"].clone();
    insert(&mut v["harmony"]["parts"], "Inconnue", unknown)?;
    reject(v)?;
    for duration in [0, 5, 65536] {
        let mut v = base.clone();
        v["harmony"]["parts"]["Modale"]["loops"][0]["chords"][0]["bars"] = duration.into();
        reject(v)?;
    }
    for degree in [0, 8] {
        let mut v = base.clone();
        v["harmony"]["parts"]["Pedale"]["degree"] = degree.into();
        reject(v)?;
    }
    let mut v = base.clone();
    insert(&mut v["harmony"]["parts"]["Pedale"], "semitones", 0)?;
    reject(v)?;
    let mut v = base.clone();
    v["harmony"]["parts"]["Pedale"].as_table_mut().ok_or("table pédale requise")?.remove("degree");
    reject(v)?;
    let mut v = base.clone();
    v["harmony"]["parts"]["Chromatique"]["loops"][0]["chords"][0]["semitones"] = 12.into();
    reject(v)?;
    let mut v = base.clone();
    v["harmony"]["parts"]["Modale"]["loops"][0]["chords"] = toml::Value::Array(Vec::new());
    reject(v)?;
    let mut v = base.clone();
    v["harmony"]["parts"]["Modale"]["loops"] = toml::Value::Array(Vec::new());
    reject(v)?;
    let mut v = base.clone();
    insert(&mut v["harmony"]["parts"]["Modale"]["loops"][0], "weight", 0)?;
    reject(v)?;
    let mut v = base.clone();
    v["harmony"]["parts"]["Modale"]["loops"][0]["chords"][0]["bars"] = 1.into();
    reject(v)?;
    for degree in [0, 8] {
        let mut v = base.clone();
        v["harmony"]["parts"]["Modale"]["loops"][0]["chords"][0]["degree"] = degree.into();
        reject(v)?;
    }
    let mut v = base.clone();
    insert(&mut v["harmony"]["parts"]["Modale"]["loops"][0]["chords"][0], "semitones", 0)?;
    reject(v)?;
    let mut v = base.clone();
    v["harmony"]["parts"]["Modale"]["loops"][0]["chords"][0]
        .as_table_mut()
        .ok_or("table accord requise")?
        .remove("degree");
    reject(v)?;
    let mut v = base.clone();
    v["harmony"]["parts"]["Modale"]["loops"][0]["chords"][0]["quality"] = "Pedal".into();
    reject(v)?;
    let mut v = base.clone();
    let legacy = v["harmony"]["parts"]["Modale"]["loops"][0]["chords"].clone();
    v["harmony"]["parts"]["Modale"].as_table_mut().ok_or("table boucle requise")?.remove("loops");
    insert(&mut v["harmony"]["parts"]["Modale"], "chords", legacy)?;
    reject(v)?;
    let mut v = base.clone();
    v["harmony"]["default_bars_per_chord"] = 0.into();
    reject(v)?;
    for neighbor in [0, 12] {
        let mut v = base.clone();
        insert(&mut v["harmony"]["parts"]["Pedale"], "neighbor_semitones", neighbor)?;
        reject(v)?;
    }
    let mut v = base.clone();
    v["harmony"]["parts"]["Modale"]["loops"][0]["chords"][0]
        .as_table_mut()
        .ok_or("table accord requise")?
        .remove("quality");
    reject(v)?;
    let mut v = base.clone();
    insert(&mut v["harmony"]["parts"]["Pedale"], "quality", "Minor")?;
    reject(v)?;
    let mut v = base;
    v["harmony"]["steps_per_chord"] = 3.into();
    reject(v)?;
    Ok(())
}

#[test]
fn alternative_unique_poids_neutre_sur_les_snapshots() -> TestResult {
    let p = load("nt_pedale_boucle")?;
    let weighted = edited(&p, |v| {
        for part in ["Chromatique", "Modale"] {
            insert(&mut v["harmony"]["parts"][part]["loops"][0], "weight", 123)?;
        }
        Ok(())
    })?;
    for seed in [1, 2, 17] {
        assert_eq!(
            serde_json::to_value(stream(&p, seed, 72)?)?,
            serde_json::to_value(stream(&weighted, seed, 72)?)?,
            "un candidat unique ne consomme aucun tirage, graine {seed}"
        );
    }
    Ok(())
}

#[test]
fn alternatives_conservent_le_choix_lors_des_changements_de_mode() -> TestResult {
    let p = load("nt_pedale_boucle")?;
    let bars =
        p.form.parts.iter().find(|p| p.name == "Modale").ok_or("partie absente")?.roles.len();
    let p = edited(&p, |v| {
        let alternatives: toml::Value = toml::from_str(&format!(
            "loops = [\
                {{ weight = 1, chords = [{{ degree = 2, quality = 'Major', bars = {bars} }}] }},\
                {{ weight = 3, chords = [{{ degree = 6, quality = 'Major', bars = {bars} }}] }}\
            ]"
        ))?;
        v["harmony"]["parts"]["Modale"]["loops"] = alternatives["loops"].clone();
        Ok(())
    })?;
    for seed in [1, 2, 17] {
        let mut par = params(&p);
        let mut g = TradGenerator::new(par.clone(), TuningParams { profile: p.clone() });
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut selected_dark_root = None;
        for bar in 1..=48 {
            if bar == 10 || bar == 22 {
                par.harmony_valence = if bar == 10 { 0.9 } else { -0.9 };
                g.invalidate_future(bar - 1);
                g.update_params(par.clone());
            }
            let s = MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng));
            let mode = if (13..=24).contains(&bar) { Mode::Mixolydian } else { Mode::Phrygian };
            assert_eq!(s.position.map(|p| p.mode), Some(mode));
            if (bar - 1) % 12 >= 8 {
                let root = *selected_dark_root.get_or_insert(s.chord_root_offset);
                assert!([1, 8].contains(&root));
                assert_eq!(
                    s.chord_root_offset,
                    root + i32::from(mode == Mode::Mixolydian),
                    "même alternative aux reprises, tours et retours au mode sombre : graine {seed}, mesure {bar}"
                );
            }
        }
    }
    Ok(())
}

//! S0 : l'accompagnement suit la graine de session, observé dans le flux de `MeasureSnapshot`
//! (principe VI).
//!
//! Comping, marche et notes de passage de la basse, motifs de basse tiraient leur graine de la
//! mélodie écrite de l'air. Sous une grille fixe, la banque de cellules minimale réduit cette
//! mélodie à deux valeurs : l'accompagnement ne dépendait presque pas de la graine de session.
//! Chaque air tire maintenant sa graine d'accompagnement dans le RNG injecté (après la mélodie,
//! l'intro et le matériel), et chaque mesure en dérive la sienne par son seul indice.
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams,
    compose::TuneGenerator,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const MELODY: u8 = 0;
const CHORDS: u8 = 1;
const BASS: u8 = 2;
const COUNTERMELODY: u8 = 5;
const BARS: usize = 24;
const SEEDS: u64 = 16;
/// Séquences distinctes exigées sur 16 graines (la ligne de base d'avant S0 en comptait 2).
const AT_LEAST: usize = 12;

fn fixture() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/grille_arc_chorus",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

/// `grille_arc_chorus` sans `[arc]` : une grille fixe de douze mesures, un seul profil de jeu.
fn fixed_grid() -> TestResult<StyleProfile> {
    let mut value: toml::Value = toml::from_str(&fixture()?.to_toml()?)?;
    let table = value.as_table_mut().ok_or("profil")?;
    table.remove("arc");
    if let Some(files) = table.get_mut("files").and_then(toml::Value::as_table_mut) {
        files.remove("arc");
    }
    Ok(StyleProfile::from_toml(&toml::to_string(&value)?)?)
}

fn params(p: &StyleProfile) -> MusicalParams {
    MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        intensity: 0.5,
        percussion_density: 1,
        ..Default::default()
    }
}

fn stream(p: &StyleProfile, seed: u64, bars: usize) -> Vec<MeasureSnapshot> {
    let mut g = TuneGenerator::new(params(p), TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect()
}

type Grid = Vec<(usize, u8, usize, usize)>;

/// Notes d'une piste, par mesure : (mesure, hauteur, pas, durée), vélocités ignorées.
fn grid(flow: &[MeasureSnapshot], track: u8) -> Grid {
    let mut notes: Grid = flow
        .iter()
        .flat_map(|s| {
            s.notes
                .iter()
                .filter(move |n: &&NoteSnapshot| n.track == track)
                .map(move |n| (s.index, n.pitch, n.start_step, n.duration_steps))
        })
        .collect();
    notes.sort_unstable();
    notes
}

fn distinct(flows: &[Vec<MeasureSnapshot>], track: u8) -> usize {
    flows.iter().map(|flow| grid(flow, track)).collect::<BTreeSet<_>>().len()
}

fn profiles() -> TestResult<Vec<(&'static str, StyleProfile)>> {
    Ok(vec![
        ("wjazzd_swing, grille par défaut", StyleProfile::named("wjazzd_swing")?),
        ("grille fixe, sans arc", fixed_grid()?),
        ("grille fixe, avec arc", fixture()?),
    ])
}

#[test]
fn des_graines_de_session_differentes_donnent_des_comping_et_des_basses_differents() -> TestResult {
    for (name, p) in profiles()? {
        let flows: Vec<_> = (1..=SEEDS).map(|seed| stream(&p, seed, BARS)).collect();
        for (layer, track) in [("comping", CHORDS), ("basse", BASS)] {
            let n = distinct(&flows, track);
            assert!(n >= AT_LEAST, "{name}, {layer} : {n} séquences distinctes sur {SEEDS}");
        }
    }
    Ok(())
}

#[test]
fn a_melodie_egale_la_graine_de_session_change_l_accompagnement() -> TestResult {
    // La mélodie écrite était la seule source de variété de l'accompagnement : on compare
    // des graines de session qui jouent la même.
    for (name, p) in profiles()? {
        let flows: Vec<_> = (1..=SEEDS).map(|seed| stream(&p, seed, BARS)).collect();
        let melodies: BTreeSet<_> = flows.iter().map(|flow| grid(flow, MELODY)).collect();
        for melody in &melodies {
            let same: Vec<_> =
                flows.iter().filter(|flow| &grid(flow, MELODY) == melody).cloned().collect();
            if same.len() < 4 {
                continue;
            }
            for (layer, track) in [("comping", CHORDS), ("basse", BASS)] {
                let n = distinct(&same, track);
                assert!(
                    n + 1 >= same.len(),
                    "{name}, {layer} : {n} séquences distinctes sur {} graines de même mélodie",
                    same.len()
                );
            }
        }
    }
    Ok(())
}

#[test]
fn meme_graine_meme_sortie() -> TestResult {
    for (name, p) in profiles()? {
        for seed in [1, 2, 17] {
            assert_eq!(
                serde_json::to_string(&stream(&p, seed, BARS))?,
                serde_json::to_string(&stream(&p, seed, BARS))?,
                "{name}, graine {seed}"
            );
        }
    }
    Ok(())
}

#[test]
fn une_mesure_se_rejoue_a_l_identique_apres_seek() -> TestResult {
    for (name, p) in profiles()? {
        let flow = stream(&p, 9, 36);
        for target in [13, 25, 31] {
            // Un générateur remis à zéro puis avancé en silence : la mesure se recalcule depuis
            // son indice et les graines de l'air, sans état d'une mesure à l'autre.
            let mut g = TuneGenerator::new(params(&p), TuningParams { profile: p.clone() });
            let mut warm = ChaCha8Rng::seed_from_u64(0);
            let _ = g.generate_measure(1, &mut warm);
            g.reset_to_initial();
            let mut rng = ChaCha8Rng::seed_from_u64(9);
            g.silent_advance(target, &mut rng);
            for bar in target..=36 {
                let again = MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng));
                assert_eq!(
                    serde_json::to_string(&again)?,
                    serde_json::to_string(&flow[bar - 1])?,
                    "{name}, seek {target}, mesure {bar}"
                );
            }
        }
    }
    Ok(())
}

/// `reel_quebecois` dont l'accompagnement ne tire plus au sort : passage de basse retiré,
/// probabilités des coups d'accords à 1, jitters du spread à 0.
fn reel_without_draws() -> TestResult<StyleProfile> {
    let mut value: toml::Value =
        toml::from_str(&StyleProfile::named("reel_quebecois")?.to_toml()?)?;
    let accompaniment = value
        .get_mut("accompaniment")
        .and_then(toml::Value::as_table_mut)
        .ok_or("accompaniment")?;
    accompaniment
        .get_mut("bass")
        .and_then(toml::Value::as_table_mut)
        .ok_or("bass")?
        .remove("passing");
    let chords =
        accompaniment.get_mut("chords").and_then(toml::Value::as_table_mut).ok_or("chords")?;
    for pattern in
        chords.get_mut("patterns").and_then(toml::Value::as_array_mut).ok_or("patterns")?
    {
        let hits = pattern.get_mut("hits").and_then(toml::Value::as_array_mut).ok_or("hits")?;
        for hit in hits {
            hit.as_table_mut().ok_or("hit")?.remove("probability");
        }
    }
    let spread = chords.get_mut("spread").and_then(toml::Value::as_table_mut).ok_or("spread")?;
    spread.insert("gap_jitter".into(), 0.0.into());
    spread.insert("velocity_jitter".into(), 0.into());
    Ok(StyleProfile::from_toml(&toml::to_string(&value)?)?)
}

#[test]
fn la_melodie_et_les_ornements_ne_dependent_pas_de_la_graine_d_accompagnement() -> TestResult {
    // La graine d'accompagnement est tirée après la mélodie, l'intro et le matériel de l'air :
    // que l'accompagnement tire ou non, la mélodie écrite et ses ornements (`material_seed`,
    // hétérophonie comprise) du premier air sont les mêmes notes.
    let with = StyleProfile::named("reel_quebecois")?;
    let without = reel_without_draws()?;
    let first_tune = |p: &StyleProfile, seed| -> Vec<MeasureSnapshot> {
        let mut flow = stream(p, seed, 96);
        flow.retain(|s| s.position.is_some_and(|p| p.tune_index == 0));
        flow
    };
    let mut accompaniment_differs = false;
    for seed in [1, 2, 3, 4] {
        let (a, b) = (first_tune(&with, seed), first_tune(&without, seed));
        assert!(a.len() >= 32, "graine {seed} : un air entier attendu");
        assert_eq!(a.len(), b.len(), "graine {seed}");
        for (x, y) in a.iter().zip(&b) {
            for track in [MELODY, COUNTERMELODY] {
                assert_eq!(
                    grid(std::slice::from_ref(x), track),
                    grid(std::slice::from_ref(y), track),
                    "graine {seed}, piste {track}, mesure {}",
                    x.index
                );
            }
        }
        accompaniment_differs |=
            [CHORDS, BASS].iter().any(|&track| grid(&a, track) != grid(&b, track));
    }
    // La variante change bien l'accompagnement : la comparaison ci-dessus n'est pas triviale.
    assert!(accompaniment_differs, "l'accompagnement sans tirage doit différer");
    Ok(())
}

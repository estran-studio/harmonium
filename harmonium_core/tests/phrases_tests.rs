//! A1 : phrases de plusieurs mesures (comping, batterie) et relances, observées dans le flux de
//! `MeasureSnapshot` (principe VI).
//!
//! Fixtures `grille_phrases` (blues de 12 mesures, quatre chorus, une partie par chorus) et
//! `grille_phrases_aaba` (32 mesures, parties A A B A de huit mesures, trois chorus) : copies des
//! fixtures de l'arc avec des phrases de comping (`accompaniment.chords.phrases`) et de batterie
//! (`percussion.phrases`, `percussion.fills`). Les tests réécrivent l'arbre TOML du profil
//! (`edited`) pour isoler un mécanisme, avec ou sans `[arc]`.
//!
//! Les six motifs de comping des fixtures se reconnaissent à leurs pas d'attaque (`MOTIFS`) : une
//! phrase de comping se lit donc comme une suite d'indices de motifs.
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams,
    compose::{PhraseAt, TuneGenerator},
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CHORDS: u8 = 1;
const PERCUSSION: [u8; 2] = [3, 4];
const SEEDS: [u64; 4] = [1, 2, 3, 17];
/// Pas d'attaque des six motifs de `comping` des fixtures (indices 0 à 5).
const MOTIFS: [&[usize]; 6] = [&[0, 6], &[4, 10, 14], &[0, 8, 14], &[6, 14], &[8], &[]];

fn fixture(name: &str) -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

/// Même profil après `change` sur son arbre TOML.
fn edited(
    p: &StyleProfile,
    change: impl FnOnce(&mut toml::Value) -> TestResult,
) -> TestResult<StyleProfile> {
    let mut value: toml::Value = toml::from_str(&p.to_toml()?)?;
    change(&mut value)?;
    Ok(StyleProfile::from_toml(&toml::to_string(&value)?)?)
}

fn table<'a>(v: &'a mut toml::Value, path: &[&str]) -> TestResult<&'a mut toml::Table> {
    let mut at = v;
    for key in path {
        at = at.get_mut(*key).ok_or_else(|| format!("table {key}"))?;
    }
    Ok(at.as_table_mut().ok_or("table")?)
}

fn parsed(toml_text: &str, key: &str) -> TestResult<toml::Value> {
    let value: toml::Value = toml::from_str(toml_text)?;
    Ok(value.get(key).ok_or("clé")?.clone())
}

/// Même profil sans `[arc]` : toutes les phrases concourent.
fn without_arc(p: &StyleProfile) -> TestResult<StyleProfile> {
    edited(p, |v| {
        v.as_table_mut().ok_or("profil")?.remove("arc");
        Ok(())
    })
}

fn with_arc(p: &StyleProfile, arc: &str) -> TestResult<StyleProfile> {
    edited(p, |v| {
        v.as_table_mut().ok_or("profil")?.insert("arc".into(), toml::from_str(arc)?);
        Ok(())
    })
}

fn stream(
    p: &StyleProfile,
    seed: u64,
    bars: usize,
    density: u8,
) -> TestResult<Vec<MeasureSnapshot>> {
    p.validate()?;
    let mut g = generator(p, density);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect())
}

fn generator(p: &StyleProfile, density: u8) -> TuneGenerator {
    let params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        intensity: 0.5,
        percussion_density: density,
        ..Default::default()
    };
    TuneGenerator::new(params, TuningParams { profile: p.clone() })
}

fn track(s: &MeasureSnapshot, id: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == id).collect()
}

fn attacks(s: &MeasureSnapshot, id: u8) -> Vec<usize> {
    track(s, id).iter().map(|n| n.start_step).collect::<BTreeSet<_>>().into_iter().collect()
}

/// Indice du motif de comping joué dans la mesure.
fn motif(s: &MeasureSnapshot) -> TestResult<usize> {
    let steps = attacks(s, CHORDS);
    MOTIFS
        .iter()
        .position(|m| **m == steps[..])
        .ok_or_else(|| format!("mesure {} : attaques {steps:?} hors des motifs", s.index).into())
}

/// Notes de percussion de la mesure : (pas, hauteur), dans l'ordre du flux.
fn drums(s: &MeasureSnapshot) -> Vec<(usize, u8)> {
    s.notes
        .iter()
        .filter(|n| PERCUSSION.contains(&n.track))
        .map(|n| (n.start_step, n.pitch))
        .collect()
}

fn drum_flow(flow: &[MeasureSnapshot]) -> Vec<Vec<(usize, u8)>> {
    flow.iter().map(drums).collect()
}

/// Séquences de motifs par passe de `width` mesures, `passes` passes.
fn passes(flow: &[MeasureSnapshot], width: usize, passes: usize) -> TestResult<Vec<Vec<usize>>> {
    (0..passes).map(|n| flow[n * width..(n + 1) * width].iter().map(motif).collect()).collect()
}

// --- Banque de comping lue dans le profil ---------------------------------------------------------

struct Phrase {
    motifs: Vec<usize>,
    at: PhraseAt,
    tag: Option<String>,
}

fn bank(p: &StyleProfile) -> TestResult<Vec<Phrase>> {
    let chords = &p.accompaniment.as_ref().ok_or("accompagnement")?.chords;
    Ok(chords
        .phrases
        .iter()
        .map(|phrase| Phrase {
            motifs: phrase.motifs.clone(),
            at: phrase.at,
            tag: phrase.tag.clone(),
        })
        .collect())
}

/// Vrai si `rest` se découpe en phrases de `bank` posées depuis son début (`PartStart` en tête).
fn opens(rest: &[usize], first: bool, bank: &[&Phrase]) -> bool {
    rest.is_empty()
        || bank.iter().any(|phrase| {
            let open = phrase.at == PhraseAt::Any || (first && phrase.at == PhraseAt::PartStart);
            open && rest.starts_with(&phrase.motifs)
                && opens(&rest[phrase.motifs.len()..], false, bank)
        })
}

/// Vrai si la passe se découpe en phrases de `bank` alignées sur ses frontières, la dernière
/// étant une phrase de fin de classe `close`.
fn tiles(pass: &[usize], close: PhraseAt, bank: &[&Phrase]) -> bool {
    let closers: Vec<_> = bank.iter().filter(|phrase| phrase.at == close).collect();
    if closers.is_empty() {
        return opens(pass, true, bank);
    }
    closers.iter().any(|phrase| {
        let start = pass.len().saturating_sub(phrase.motifs.len());
        pass.ends_with(&phrase.motifs) && opens(&pass[..start], true, bank)
    })
}

#[test]
fn les_phrases_de_comping_s_alignent_sur_les_frontieres_de_partie_et_de_chorus() -> TestResult {
    // Blues : une partie de 12 mesures = un chorus ; la mise en place est la phrase `part_end`.
    let blues = without_arc(&fixture("grille_phrases")?)?;
    let blues_bank = bank(&blues)?;
    let all: Vec<&Phrase> = blues_bank.iter().collect();
    for seed in SEEDS {
        let flow = stream(&blues, seed, 36, 1)?;
        for (n, pass) in passes(&flow, 12, 3)?.iter().enumerate() {
            assert!(
                tiles(pass, PhraseAt::PartEnd, &all),
                "blues graine {seed} chorus {}: {pass:?}",
                n + 1
            );
        }
    }
    // AABA : quatre passes de huit mesures par chorus ; les trois premières finissent sur une
    // phrase `part_end`, la dernière (fin du chorus) sur une phrase `chorus_end`, et aucune
    // phrase ne franchit une frontière de partie ni de chorus.
    let aaba = without_arc(&fixture("grille_phrases_aaba")?)?;
    let aaba_bank = bank(&aaba)?;
    let all: Vec<&Phrase> = aaba_bank.iter().collect();
    for seed in SEEDS {
        let flow = stream(&aaba, seed, 64, 1)?;
        for (n, pass) in passes(&flow, 8, 8)?.iter().enumerate() {
            let close = if n % 4 == 3 { PhraseAt::ChorusEnd } else { PhraseAt::PartEnd };
            assert!(tiles(pass, close, &all), "AABA graine {seed} passe {n}: {pass:?}");
        }
    }
    Ok(())
}

#[test]
fn les_phrases_remplacent_le_tirage_d_un_motif_par_mesure() -> TestResult {
    // Sans phrase, le même profil tire un motif par mesure : les passes ne se découpent plus
    // toujours en phrases de la banque.
    let phrased = without_arc(&fixture("grille_phrases")?)?;
    let bare = edited(&phrased, |v| {
        table(v, &["accompaniment", "chords"])?.remove("phrases");
        Ok(())
    })?;
    let all_phrases = bank(&phrased)?;
    let all: Vec<&Phrase> = all_phrases.iter().collect();
    let aligned = |p: &StyleProfile| -> TestResult<usize> {
        let mut count = 0;
        for seed in 1..=16 {
            let flow = stream(p, seed, 36, 1)?;
            count += passes(&flow, 12, 3)?
                .iter()
                .filter(|pass| tiles(pass, PhraseAt::PartEnd, &all))
                .count();
        }
        Ok(count)
    };
    assert_eq!(aligned(&phrased)?, 48, "toutes les passes sont des phrases");
    assert!(aligned(&bare)? < 24, "un tirage par mesure n'est presque jamais une suite de phrases");
    Ok(())
}

#[test]
fn l_arc_restreint_les_phrases_de_comping_par_tag() -> TestResult {
    let base = fixture("grille_phrases")?;
    for tag in ["sparse", "medium", "dense"] {
        let arc = format!(
            "[[stages]]\nchoruses = 4\nvariants = [{{ comping = [{{ tag = \"{tag}\" }}] }}]\n"
        );
        let p = with_arc(&base, &arc)?;
        let phrases = bank(&p)?;
        let named: Vec<&Phrase> =
            phrases.iter().filter(|x| x.tag.as_deref() == Some(tag)).collect();
        let others: Vec<&Phrase> =
            phrases.iter().filter(|x| x.tag.as_deref() != Some(tag)).collect();
        let mut foreign_tile = 0;
        for seed in SEEDS {
            let flow = stream(&p, seed, 36, 1)?;
            for pass in passes(&flow, 12, 3)? {
                assert!(tiles(&pass, PhraseAt::PartEnd, &named), "{tag} graine {seed} : {pass:?}");
                foreign_tile += usize::from(tiles(&pass, PhraseAt::PartEnd, &others));
            }
        }
        assert_eq!(foreign_tile, 0, "{tag} : aucune phrase d'une autre étiquette ne se joue");
    }
    Ok(())
}

// --- Batterie : phrases ---------------------------------------------------------------------------

/// (nombre de coups, pédale) du pattern de densité 1 joué : 3 → (6, non), 4 → (8, oui),
/// 5 → (6, oui), 6 → (9, non).
fn density_one_pattern(s: &MeasureSnapshot) -> Option<usize> {
    let notes = drums(s);
    let pedal = notes.iter().any(|(_, pitch)| *pitch == 44);
    match (notes.len(), pedal) {
        (6, false) => Some(3),
        (8, true) => Some(4),
        (6, true) => Some(5),
        (9, false) => Some(6),
        _ => None,
    }
}

fn drum_phrases(p: &StyleProfile, phrases: &str) -> TestResult<StyleProfile> {
    edited(&without_arc(p)?, |v| {
        let percussion = table(v, &["percussion"])?;
        percussion.remove("fills");
        percussion.insert("phrases".into(), parsed(phrases, "phrases")?);
        Ok(())
    })
}

#[test]
fn les_phrases_de_batterie_suivent_leurs_patterns_et_s_alignent_sur_la_partie() -> TestResult {
    // Une phrase de quatre mesures (patterns 3, 4, 5, 6 de densité 1) et une mise en place de deux
    // mesures en fin de partie : 4 + 4, deux mesures libres, puis la mise en place (12 mesures).
    let p = drum_phrases(
        &fixture("grille_phrases")?,
        "phrases = [\
         { density = 1, patterns = [3, 4, 5, 6] },\
         { density = 1, at = \"part_end\", patterns = [6, 5] }]",
    )?;
    for seed in SEEDS {
        let flow = stream(&p, seed, 36, 1)?;
        for chorus in flow.chunks(12) {
            let pattern: Vec<_> = chorus.iter().map(density_one_pattern).collect();
            assert_eq!(pattern[..4], [Some(3), Some(4), Some(5), Some(6)], "graine {seed}");
            assert_eq!(pattern[4..8], [Some(3), Some(4), Some(5), Some(6)], "graine {seed}");
            assert_eq!(pattern[10..], [Some(6), Some(5)], "graine {seed} : mise en place");
        }
    }
    Ok(())
}

#[test]
fn les_phrases_de_batterie_d_une_autre_densite_ne_jouent_pas() -> TestResult {
    let p = drum_phrases(
        &fixture("grille_phrases")?,
        "phrases = [{ density = 1, patterns = [3, 4, 5, 6] }]",
    )?;
    let plain = edited(&p, |v| {
        table(v, &["percussion"])?.remove("phrases");
        Ok(())
    })?;
    // Densité 0 : la phrase de densité 1 ne concourt pas, la mesure est celle du profil sans phrase.
    assert_eq!(drum_flow(&stream(&p, 3, 24, 0)?), drum_flow(&stream(&plain, 3, 24, 0)?));
    Ok(())
}

// --- Batterie : relances --------------------------------------------------------------------------

/// Profil de batterie à relances seules (`fills`), sans phrase, dont les fins de phrase de la
/// forme sont `ends` (mesures 1-based du blues).
fn fills_profile(fills: &str, ends: &[i64]) -> TestResult<StyleProfile> {
    edited(&without_arc(&fixture("grille_phrases")?)?, |v| {
        let percussion = table(v, &["percussion"])?;
        percussion.remove("phrases");
        percussion.insert("fills_none_weight".into(), 0.into());
        if fills.is_empty() {
            percussion.remove("fills");
        } else {
            percussion.insert("fills".into(), parsed(fills, "fills")?);
        }
        let ends: Vec<toml::Value> = ends.iter().map(|&e| e.into()).collect();
        v["form"]["parts"][0]["phrase_ends"] = toml::Value::Array(ends);
        Ok(())
    })
}

/// Mesure `n` (0-based) du chorus : notes de la relance (pas ≥ `from`) ou de tout le pattern.
fn tail(notes: &[(usize, u8)], from: usize) -> Vec<(usize, u8)> {
    notes.iter().copied().filter(|(step, _)| *step >= from).collect()
}

fn head(notes: &[(usize, u8)], until: usize) -> Vec<(usize, u8)> {
    notes.iter().copied().filter(|(step, _)| *step < until).collect()
}

const SNARE: u8 = 38;
const TOM: u8 = 43;
const KICK: u8 = 36;

#[test]
fn la_relance_d_une_mesure_part_au_pas_donne_a_chaque_fin_de_phrase() -> TestResult {
    let fills = "fills = [{ density = 1, start_step = 12, bars = [{ hits = [\
                 { step = 12, voice = \"snare\" }, { step = 14, voice = \"tom\" }] }] }]";
    let with = fills_profile(fills, &[4, 8, 12])?;
    let without = fills_profile("", &[4, 8, 12])?;
    for seed in SEEDS {
        let (a, b) =
            (drum_flow(&stream(&with, seed, 36, 1)?), drum_flow(&stream(&without, seed, 36, 1)?));
        for (bar, (relance, ordinaire)) in a.iter().zip(&b).enumerate() {
            if matches!(bar % 12, 3 | 7 | 11) {
                // Levée au temps 4 : le pattern joue jusqu'au pas 11, la relance reprend au pas 12.
                assert_eq!(
                    head(relance, 12),
                    head(ordinaire, 12),
                    "graine {seed} mesure {}",
                    bar + 1
                );
                assert_eq!(
                    tail(relance, 12),
                    [(12, SNARE), (14, TOM)],
                    "graine {seed} mesure {}",
                    bar + 1
                );
            } else {
                assert_eq!(relance, ordinaire, "graine {seed} mesure {}", bar + 1);
            }
        }
    }
    Ok(())
}

#[test]
fn la_relance_de_plusieurs_mesures_s_acheve_avec_la_phrase() -> TestResult {
    // Trois mesures à partir du pas 8 de la première : elles couvrent les mesures 2, 3 et 4 de la
    // phrase de quatre mesures ; la première garde le pattern jusqu'au pas 7.
    let fills = "fills = [{ density = 1, start_step = 8, bars = [\
        { hits = [{ step = 8, voice = \"snare\" }, { step = 10, voice = \"snare\" }] },\
        { hits = [{ step = 0, voice = \"tom\" }] },\
        { hits = [{ step = 0, voice = \"kick\" }, { step = 12, voice = \"snare\" }] }] }]";
    let with = fills_profile(fills, &[4, 8, 12])?;
    let without = fills_profile("", &[4, 8, 12])?;
    for seed in SEEDS {
        let (a, b) =
            (drum_flow(&stream(&with, seed, 36, 1)?), drum_flow(&stream(&without, seed, 36, 1)?));
        for (bar, (relance, ordinaire)) in a.iter().zip(&b).enumerate() {
            let at = format!("graine {seed} mesure {}", bar + 1);
            match bar % 4 {
                0 => assert_eq!(relance, ordinaire, "{at} : hors relance"),
                1 => {
                    assert_eq!(head(relance, 8), head(ordinaire, 8), "{at}");
                    assert_eq!(tail(relance, 8), [(8, SNARE), (10, SNARE)], "{at}");
                }
                2 => assert_eq!(relance[..], [(0, TOM)], "{at}"),
                _ => assert_eq!(relance[..], [(0, KICK), (12, SNARE)], "{at}"),
            }
        }
    }
    Ok(())
}

#[test]
fn une_relance_ne_deborde_pas_de_la_phrase_qu_elle_clot() -> TestResult {
    // Phrases de deux mesures : la relance de trois mesures ne tient dans aucune, celle de deux
    // joue sur les mesures paires et impaires qui précèdent chaque fin.
    let three = "{ density = 1, bars = [{ hits = [{ step = 0, voice = \"kick\" }] }, \
                 { hits = [{ step = 0, voice = \"kick\" }] }, { hits = [{ step = 0, voice = \"kick\" }] }] }";
    let two = "{ density = 1, bars = [{ hits = [{ step = 0, voice = \"tom\" }] }, \
               { hits = [{ step = 0, voice = \"snare\" }] }] }";
    let ends = [2, 4, 6, 8, 10, 12];
    let without = drum_flow(&stream(&fills_profile("", &ends)?, 5, 12, 1)?);
    let long = fills_profile(&format!("fills = [{three}]"), &ends)?;
    assert_eq!(
        drum_flow(&stream(&long, 5, 12, 1)?),
        without,
        "trois mesures dans deux : aucune relance"
    );
    let short =
        drum_flow(&stream(&fills_profile(&format!("fills = [{three}, {two}]"), &ends)?, 5, 12, 1)?);
    for (bar, notes) in short.iter().enumerate() {
        assert_eq!(
            notes[..],
            if bar % 2 == 0 { [(0, TOM)] } else { [(0, SNARE)] },
            "mesure {}",
            bar + 1
        );
    }
    Ok(())
}

#[test]
fn la_relance_de_fin_de_partie_ne_joue_qu_en_fin_de_partie() -> TestResult {
    // Fins de phrase aux mesures 4, 8 et 12 ; la relance `part_end` de deux mesures ne joue
    // qu'avant le changement de section, mesures 11 et 12.
    let fills = "fills = [{ density = 1, at = \"part_end\", start_step = 8, bars = [\
        { hits = [{ step = 8, voice = \"snare\" }] },\
        { hits = [{ step = 0, voice = \"kick\" }] }] }]";
    let with = fills_profile(fills, &[4, 8, 12])?;
    let without = fills_profile("", &[4, 8, 12])?;
    for seed in SEEDS {
        let (a, b) =
            (drum_flow(&stream(&with, seed, 36, 1)?), drum_flow(&stream(&without, seed, 36, 1)?));
        for (bar, (relance, ordinaire)) in a.iter().zip(&b).enumerate() {
            match bar % 12 {
                10 => {
                    assert_eq!(tail(relance, 8), [(8, SNARE)], "graine {seed} mesure {}", bar + 1)
                }
                11 => assert_eq!(relance[..], [(0, KICK)], "graine {seed} mesure {}", bar + 1),
                _ => assert_eq!(relance, ordinaire, "graine {seed} mesure {}", bar + 1),
            }
        }
    }
    Ok(())
}

#[test]
fn la_relance_de_fin_de_chorus_l_emporte_sur_celle_de_fin_de_partie() -> TestResult {
    // AABA : la dernière passe de la forme (mesures 25 à 32) joue la relance `chorus_end`, les
    // autres parties la relance `part_end`.
    let fills = "fills = [\
        { density = 1, at = \"part_end\", bars = [{ hits = [{ step = 0, voice = \"tom\" }] }] },\
        { density = 1, at = \"chorus_end\", bars = [{ hits = [{ step = 0, voice = \"snare\" }] }] }]";
    let p = edited(&without_arc(&fixture("grille_phrases_aaba")?)?, |v| {
        let percussion = table(v, &["percussion"])?;
        percussion.remove("phrases");
        percussion.insert("fills_none_weight".into(), 0.into());
        percussion.insert("fills".into(), parsed(fills, "fills")?);
        // Fins de phrase aux mesures 8 de chaque partie (`phrase_ends = [8]`).
        Ok(())
    })?;
    let flow = drum_flow(&stream(&p, 2, 64, 1)?);
    for (bar, notes) in flow.iter().enumerate() {
        let (in_chorus, end_of_part) = (bar % 32, bar % 8 == 7);
        if end_of_part && in_chorus == 31 {
            assert_eq!(notes[..], [(0, SNARE)], "mesure {} : fin de chorus", bar + 1);
        } else if end_of_part {
            assert_eq!(notes[..], [(0, TOM)], "mesure {} : fin de partie", bar + 1);
        } else {
            assert!(notes.len() > 1, "mesure {} : pattern ordinaire", bar + 1);
        }
    }
    Ok(())
}

#[test]
fn l_arc_restreint_les_relances_et_les_phrases_de_batterie_par_tag() -> TestResult {
    let base = edited(&fixture("grille_phrases")?, |v| {
        let percussion = table(v, &["percussion"])?;
        percussion.insert("fills_none_weight".into(), 0.into());
        percussion.insert(
            "fills".into(),
            parsed(
                "fills = [\
                 { density = 1, tag = \"hat\", start_step = 12, bars = [{ hits = [{ step = 14, voice = \"snare\" }] }] },\
                 { density = 1, tag = \"ride\", start_step = 12, bars = [{ hits = [{ step = 14, voice = \"tom\" }] }] }]",
                "fills",
            )?,
        );
        Ok(())
    })?;
    for (tag, relance, absent) in [("hat", SNARE, TOM), ("ride", TOM, SNARE)] {
        let arc = format!(
            "[[stages]]\nchoruses = 4\nvariants = [{{ percussion = [{{ tag = \"{tag}\" }}] }}]\n"
        );
        let p = with_arc(&base, &arc)?;
        let flow = drum_flow(&stream(&p, 4, 36, 1)?);
        let last = flow.chunks(12).map(|chorus| chorus[11].clone()).collect::<Vec<_>>();
        for notes in &last {
            let at14: Vec<u8> = notes.iter().filter(|(s, _)| *s == 14).map(|(_, p)| *p).collect();
            assert!(at14.contains(&relance), "{tag} : relance {relance} attendue, {notes:?}");
            assert!(!at14.contains(&absent), "{tag} : relance d'une autre étiquette, {notes:?}");
        }
    }
    Ok(())
}

// --- Tirage sans état, graines ---------------------------------------------------------------------

fn json(flow: &[MeasureSnapshot]) -> TestResult<String> {
    Ok(serde_json::to_string(flow)?)
}

#[test]
fn une_phrase_se_rejoue_a_l_identique_apres_un_seek() -> TestResult {
    // Un autre générateur, remis à zéro puis avancé en silence jusqu'à une mesure du milieu d'une
    // phrase ou d'une passe, rejoue la suite à l'identique (comping, batterie, relances).
    for (name, bars, starts) in
        [("grille_phrases", 48, [13, 18, 22, 35]), ("grille_phrases_aaba", 80, [10, 14, 33, 55])]
    {
        let p = fixture(name)?;
        let flow = stream(&p, 9, bars, 1)?;
        for start in starts {
            let mut g = generator(&p, 1);
            let mut rng = ChaCha8Rng::seed_from_u64(0);
            let _ = g.generate_measure(1, &mut rng);
            g.reset_to_initial();
            let mut rng = ChaCha8Rng::seed_from_u64(9);
            g.silent_advance(start, &mut rng);
            for bar in start..=bars - 1 {
                let again = MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng));
                assert_eq!(
                    serde_json::to_string(&again)?,
                    serde_json::to_string(&flow[bar - 1])?,
                    "{name} : mesure {bar} après un seek à {start}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn meme_graine_meme_sortie_graines_differentes_phrases_differentes() -> TestResult {
    for name in ["grille_phrases", "grille_phrases_aaba"] {
        let p = fixture(name)?;
        assert_eq!(json(&stream(&p, 5, 40, 1)?)?, json(&stream(&p, 5, 40, 1)?)?, "{name}");
    }
    // Sans arc : toute la variété vient des phrases. 16 graines × 36 mesures du blues.
    let p = without_arc(&fixture("grille_phrases")?)?;
    let mut comping = BTreeSet::new();
    let mut drums_seen = BTreeSet::new();
    for seed in 1..=16 {
        let flow = stream(&p, seed, 36, 2)?;
        comping.insert(passes(&flow, 12, 3)?);
        drums_seen.insert(drum_flow(&flow));
    }
    assert!(comping.len() >= 14, "16 graines, {} suites de phrases de comping", comping.len());
    assert!(drums_seen.len() >= 14, "16 graines, {} suites de batterie", drums_seen.len());
    // D'un chorus à l'autre, une même graine ne répète pas la même phrase partout.
    let varied = (1..=16u64).filter(|&seed| {
        let flow = stream(&p, seed, 36, 1).unwrap_or_default();
        let chorus = passes(&flow, 12, 3).unwrap_or_default();
        chorus.windows(2).any(|pair| pair[0] != pair[1])
    });
    assert!(varied.count() >= 14, "un chorus n'est pas la boucle du précédent");
    Ok(())
}

// --- Optionnel en données, schéma ------------------------------------------------------------------

#[test]
fn un_profil_sans_phrases_ne_les_serialise_pas() -> TestResult {
    let text = StyleProfile::default().to_toml()?;
    for key in ["phrases", "fills"] {
        assert!(!text.contains(key), "{key} dans la sérialisation d'un profil sans phrases");
    }
    // Un profil à phrases survit à l'aller-retour TOML.
    let p = fixture("grille_phrases_aaba")?;
    assert_eq!(StyleProfile::from_toml(&p.to_toml()?)?, p);
    Ok(())
}

fn invalid(
    p: &StyleProfile,
    change: impl FnOnce(&mut toml::Value) -> TestResult,
) -> TestResult<String> {
    let error = edited(p, change).err().ok_or("le profil devait être refusé")?;
    Ok(error.to_string())
}

#[test]
fn les_erreurs_de_schema_nomment_le_champ_des_phrases_et_des_relances() -> TestResult {
    let p = fixture("grille_phrases_aaba")?;
    let comping = |text: &'static str| {
        move |v: &mut toml::Value| -> TestResult {
            table(v, &["accompaniment", "chords"])?
                .insert("phrases".into(), parsed(text, "phrases")?);
            Ok(())
        }
    };
    let percussion = |key: &'static str, text: String| {
        move |v: &mut toml::Value| -> TestResult {
            let table = table(v, &["percussion"])?;
            table.insert(key.into(), parsed(&text, key)?);
            Ok(())
        }
    };
    let comping_cases = [
        ("phrases = [{ weight = 0, motifs = [0] }]", "accompaniment.chords.phrases[0].weight"),
        ("phrases = [{ motifs = [] }]", "accompaniment.chords.phrases[0].motifs"),
        (
            "phrases = [{ motifs = [0, 0, 0, 0, 0, 0, 0, 0, 0] }]",
            "accompaniment.chords.phrases[0].motifs",
        ),
        ("phrases = [{ motifs = [0, 9] }]", "accompaniment.chords.phrases[0].motifs[1]"),
        ("phrases = [{ motifs = [0], part = \"Z\" }]", "accompaniment.chords.phrases[0].part"),
        ("phrases = [{ motifs = [0] }, { motifs = [1], tag = \"x\", at = \"milieu\" }]", "at"),
    ];
    for (text, field) in comping_cases {
        let message = invalid(&p, comping(text))?;
        assert!(message.contains(field), "{text} : {message}");
    }
    let fill = |extra: &str| format!("fills = [{{ density = 1, {extra} }}]");
    let hit = |step: usize| format!("{{ hits = [{{ step = {step}, voice = \"snare\" }}] }}");
    let cases: Vec<(&str, String, &str)> = vec![
        (
            "phrases",
            "phrases = [{ density = 3, patterns = [0] }]".into(),
            "percussion.phrases[0].density",
        ),
        (
            "phrases",
            "phrases = [{ density = 0, weight = 0, patterns = [0] }]".into(),
            "percussion.phrases[0].weight",
        ),
        (
            "phrases",
            "phrases = [{ density = 0, patterns = [] }]".into(),
            "percussion.phrases[0].patterns",
        ),
        (
            "phrases",
            "phrases = [{ density = 0, patterns = [0, 99] }]".into(),
            "percussion.phrases[0].patterns[1]",
        ),
        // Le pattern 3 est de densité 1.
        (
            "phrases",
            "phrases = [{ density = 0, patterns = [0, 3] }]".into(),
            "percussion.phrases[0].patterns[1]",
        ),
        (
            "phrases",
            "phrases = [{ density = 0, patterns = [0], part = \"Z\" }]".into(),
            "percussion.phrases[0].part",
        ),
        ("fills", fill("bars = []"), "percussion.fills[0].bars"),
        (
            "fills",
            fill(&format!("bars = [{0}, {0}, {0}, {0}, {0}]", hit(0))),
            "percussion.fills[0].bars",
        ),
        ("fills", fill(&format!("weight = 0, bars = [{}]", hit(0))), "percussion.fills[0].weight"),
        (
            "fills",
            fill(&format!("start_step = 16, bars = [{}]", hit(0))),
            "percussion.fills[0].start_step",
        ),
        (
            "fills",
            fill(&format!("bars = [{}]", hit(16))),
            "percussion.fills[0].bars[0].hits[0].step",
        ),
        (
            "fills",
            fill(&format!("start_step = 8, bars = [{}]", hit(4))),
            "percussion.fills[0].bars[0].hits[0].step",
        ),
        (
            "fills",
            fill(&format!("at = \"part_start\", bars = [{}]", hit(0))),
            "percussion.fills[0].at",
        ),
    ];
    for (key, text, field) in cases {
        let message = invalid(&p, percussion(key, text.clone()))?;
        assert!(message.contains(field), "{text} : {message}");
    }
    // `cadence_patterns` et `fills` à la même densité : le profil choisit l'un ou l'autre.
    let message = invalid(&p, |v| {
        let table = table(v, &["percussion"])?;
        table.insert("fills".into(), parsed(&fill(&format!("bars = [{}]", hit(0))), "fills")?);
        table.insert(
            "cadence_patterns".into(),
            parsed("cadence_patterns = [{ density = 1, hits = [] }]", "cadence_patterns")?,
        );
        Ok(())
    })?;
    assert!(message.contains("percussion.fills[0].density"), "{message}");
    // Une étiquette portée par une phrase seule (aucun motif) est connue de l'arc.
    edited(&p, |v| {
        let chords = table(v, &["accompaniment", "chords"])?;
        let phrases =
            chords.get_mut("phrases").and_then(toml::Value::as_array_mut).ok_or("phrases")?;
        phrases[0].as_table_mut().ok_or("phrase")?.insert("tag".into(), "seule".into());
        v["arc"]["stages"][0]["variants"][0]
            .as_table_mut()
            .ok_or("variante")?
            .insert("comping".into(), parsed("comping = [{ tag = \"seule\" }]", "comping")?);
        Ok(())
    })?;
    Ok(())
}

#[test]
fn une_phrase_de_partie_ne_joue_que_dans_cette_partie() -> TestResult {
    // Une phrase de quatre mesures réservée à la partie B (mesures 17 à 24 de chaque chorus) et
    // de poids écrasant : elle ouvre B, jamais A.
    let p = edited(&without_arc(&fixture("grille_phrases_aaba")?)?, |v| {
        let chords = table(v, &["accompaniment", "chords"])?;
        let phrases =
            chords.get_mut("phrases").and_then(toml::Value::as_array_mut).ok_or("phrases")?;
        phrases.push(
            parsed(
                "phrases = [{ weight = 100000, part = \"B\", motifs = [4, 4, 4, 4] }]",
                "phrases",
            )?[0]
                .clone(),
        );
        Ok(())
    })?;
    for seed in SEEDS {
        let flow = stream(&p, seed, 64, 1)?;
        for chorus in [0, 32] {
            let bridge: Vec<usize> =
                (16..20).map(|n| motif(&flow[chorus + n])).collect::<TestResult<_>>()?;
            assert_eq!(bridge, [4, 4, 4, 4], "graine {seed} : ouverture de B");
        }
        let in_a: Vec<usize> = flow[..16].iter().map(motif).collect::<TestResult<_>>()?;
        assert!(in_a.iter().any(|&m| m != 4), "graine {seed} : la phrase de B ne joue pas dans A");
    }
    Ok(())
}

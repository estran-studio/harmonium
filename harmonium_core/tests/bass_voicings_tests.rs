//! A4 et A5 : basse plus riche et voicings plus variés, observés dans le flux de
//! `MeasureSnapshot` (principe VI).
//!
//! La fixture `grille_basse_voicings` est `grille_arc_chorus` (blues de douze mesures en fa,
//! quatre chorus) avec toutes les options de basse (`accompaniment.bass.walking.{arrival,
//! eighths, two_feel, approach.side, passing.chromatic_weight, passing.repeat_weight}`) et de
//! voicings (`accompaniment.chords.{color, register_drift}`) ; les nouvelles familles de voicings
//! (`drop3`, `fourths`, `upper_structure`, `shell`, `open`) se choisissent par `voicing`. Les tests
//! réécrivent la section voulue de la fixture, à la manière de `arc_chorus_tests`.
use std::collections::{BTreeMap, BTreeSet};

use harmonium_core::{
    StyleProfile, TuningParams,
    compose::TuneGenerator,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CHORDS: u8 = 1;
const BASS: u8 = 2;
const BLUES: usize = 12;
/// Fenêtre de marche de la fixture (`walk_low..=walk_high`) et d'arrivée (`low..=high`).
const WALK: (u8, u8) = (33, 50);
const ARRIVAL: (u8, u8) = (36, 47);
/// Gamme de fa majeur (classes de hauteur).
const F_MAJOR: [u8; 7] = [5, 7, 9, 10, 0, 2, 4];
const SEEDS: [u64; 6] = [1, 2, 3, 4, 17, 99];

fn profile(name: &str) -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

fn fixture() -> TestResult<StyleProfile> {
    profile("grille_basse_voicings")
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

fn table<'a>(v: &'a mut toml::Value, path: &[&str]) -> TestResult<&'a mut toml::value::Table> {
    let mut at = v;
    for key in path {
        at = at.get_mut(*key).ok_or_else(|| format!("section {key} absente"))?;
    }
    Ok(at.as_table_mut().ok_or("table attendue")?)
}

fn walking(v: &mut toml::Value) -> TestResult<&mut toml::value::Table> {
    table(v, &["accompaniment", "bass", "walking"])
}

fn chords(v: &mut toml::Value) -> TestResult<&mut toml::value::Table> {
    table(v, &["accompaniment", "chords"])
}

/// Retire `[arc]` : douze mesures de marche, quatre chorus identiques de forme.
fn without_arc(v: &mut toml::Value) -> TestResult {
    v.as_table_mut().ok_or("profil")?.remove("arc");
    if let Some(files) = v.get_mut("files").and_then(toml::Value::as_table_mut) {
        files.remove("arc");
    }
    Ok(())
}

/// Retire les options de basse d'A4.
fn without_bass_options(v: &mut toml::Value) -> TestResult {
    let w = walking(v)?;
    for key in ["arrival", "eighths", "two_feel"] {
        w.remove(key);
    }
    w["approach"].as_table_mut().ok_or("approach")?.remove("side");
    let passing = w["passing"].as_table_mut().ok_or("passing")?;
    passing.remove("chromatic_weight");
    passing.remove("repeat_weight");
    Ok(())
}

/// Retire les options de voicings d'A5.
fn without_voicing_options(v: &mut toml::Value) -> TestResult {
    let c = chords(v)?;
    c.remove("color");
    c.remove("register_drift");
    Ok(())
}

fn set_walking(key: &str, value: &str) -> impl FnOnce(&mut toml::Value) -> TestResult {
    let (key, value) = (key.to_owned(), value.to_owned());
    move |v| {
        let parsed: toml::Value = toml::from_str(&format!("x = {value}"))?;
        walking(v)?.insert(key, parsed["x"].clone());
        Ok(())
    }
}

fn stream(p: &StyleProfile, seed: u64, bars: usize) -> TestResult<Vec<MeasureSnapshot>> {
    p.validate()?;
    let mut g = generator(p);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect())
}

fn generator(p: &StyleProfile) -> TuneGenerator {
    let params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        intensity: 0.5,
        percussion_density: 1,
        ..Default::default()
    };
    TuneGenerator::new(params, TuningParams { profile: p.clone() })
}

fn json(flow: &[MeasureSnapshot]) -> TestResult<String> {
    Ok(serde_json::to_string(flow)?)
}

fn track(s: &MeasureSnapshot, id: u8) -> Vec<&NoteSnapshot> {
    let mut notes: Vec<_> = s.notes.iter().filter(|n| n.track == id).collect();
    notes.sort_by_key(|n| (n.start_step, n.pitch));
    notes
}

/// Les notes de basse des mesures, une liste par mesure.
fn bass_bars(flow: &[MeasureSnapshot]) -> Vec<Vec<&NoteSnapshot>> {
    flow.iter().map(|s| track(s, BASS)).collect()
}

/// Classes de hauteur de l'accord publié en vigueur au pas `step`.
fn chord_pcs_at(s: &MeasureSnapshot, step: usize) -> Vec<u8> {
    s.chord_changes
        .iter()
        .rev()
        .find(|c| c.start_step <= step)
        .map_or_else(Vec::new, |c| c.context.pitch_classes.clone())
}

fn bass_pc_at(s: &MeasureSnapshot, step: usize) -> Option<u8> {
    s.chord_changes.iter().rev().find(|c| c.start_step <= step).map(|c| c.context.bass_pc)
}

/// Pas des arrivées d'une mesure : les changements d'accord (le pas 0 en fait partie).
fn arrival_steps(s: &MeasureSnapshot) -> Vec<usize> {
    s.chord_changes.iter().map(|c| c.start_step).collect()
}

// --- A4 : basse -----------------------------------------------------------------------------

#[test]
fn sans_option_le_flux_est_celui_de_la_fixture_de_l_arc() -> TestResult {
    // Toutes les options absentes : `grille_basse_voicings` redevient `grille_arc_chorus`.
    // Les poids de base (approche, passage) sont ceux calibrés de la fixture : on rend ceux de
    // `grille_arc_chorus`.
    let plain = edited(&fixture()?, |v| {
        without_bass_options(v)?;
        without_voicing_options(v)?;
        let w = walking(v)?;
        w.insert(
            "approach".into(),
            toml::from_str("none_weight = 60\nchromatic_weight = 8\ndiatonic_weight = 8")?,
        );
        w.insert(
            "passing".into(),
            toml::from_str("chord_tone_weight = 35\nscale_step_weight = 65")?,
        );
        Ok(())
    })?;
    let reference = profile("grille_arc_chorus")?;
    for seed in SEEDS {
        assert_eq!(
            json(&stream(&plain, seed, 48)?)?,
            json(&stream(&reference, seed, 48)?)?,
            "graine {seed}"
        );
    }
    Ok(())
}

#[test]
fn chaque_option_de_basse_change_la_ligne_et_elle_seule() -> TestResult {
    let p = edited(&fixture()?, without_arc)?;
    let off = |change: fn(&mut toml::Value) -> TestResult| edited(&p, change);
    let drops: [(&str, StyleProfile); 5] = [
        (
            "arrival",
            off(|v| {
                let _ = walking(v)?.remove("arrival");
                Ok(())
            })?,
        ),
        (
            "eighths",
            off(|v| {
                let _ = walking(v)?.remove("eighths");
                Ok(())
            })?,
        ),
        (
            "approach.side",
            off(|v| {
                let _ = walking(v)?["approach"].as_table_mut().ok_or("a")?.remove("side");
                Ok(())
            })?,
        ),
        (
            "passing.chromatic",
            off(|v| {
                let passing = walking(v)?["passing"].as_table_mut().ok_or("p")?;
                passing.remove("chromatic_weight");
                Ok(())
            })?,
        ),
        (
            "passing.repeat",
            off(|v| {
                let passing = walking(v)?["passing"].as_table_mut().ok_or("p")?;
                passing.remove("repeat_weight");
                Ok(())
            })?,
        ),
    ];
    let full: Vec<String> =
        SEEDS.iter().map(|&s| json(&stream(&p, s, 48)?)).collect::<TestResult<_>>()?;
    for (name, variant) in &drops {
        let differs = SEEDS.iter().zip(&full).any(|(&s, f)| {
            stream(variant, s, 48).map(|flow| json(&flow).ok().as_ref() != Some(f)).unwrap_or(true)
        });
        assert!(differs, "retirer {name} ne change rien : l'option n'agit pas");
    }
    Ok(())
}

#[test]
fn les_notes_de_pulsation_gardent_la_fenetre_et_les_arrivees_sont_des_notes_d_accord() -> TestResult
{
    let p = edited(&fixture()?, without_arc)?;
    for seed in SEEDS {
        for s in &stream(&p, seed, 48)? {
            let bass = track(s, BASS);
            assert!(!bass.is_empty(), "graine {seed}, mesure {}", s.index);
            for n in &bass {
                assert!(
                    (WALK.0..=WALK.1).contains(&n.pitch),
                    "graine {seed}, mesure {} : {} hors fenêtre",
                    s.index,
                    n.pitch
                );
            }
            // Chaque arrivée (changement d'accord, premier pas) joue une note de l'accord publié,
            // dans le registre d'arrivée : aucune note hors accord sur un temps d'arrivée (D4).
            for step in arrival_steps(s) {
                let note = bass
                    .iter()
                    .find(|n| n.start_step == step)
                    .ok_or("attaque de basse à l'arrivée")?;
                assert!(
                    chord_pcs_at(s, step).contains(&(note.pitch % 12)),
                    "graine {seed}, mesure {}, pas {step} : {} hors de l'accord",
                    s.index,
                    note.pitch
                );
                assert!(
                    (ARRIVAL.0..=ARRIVAL.1).contains(&note.pitch),
                    "graine {seed}, mesure {} : arrivée {} hors registre",
                    s.index,
                    note.pitch
                );
            }
        }
    }
    Ok(())
}

/// Arrivées d'une marche (premier pas de chaque mesure) sur `seeds` graines.
fn first_degrees(p: &StyleProfile, seeds: &[u64]) -> TestResult<BTreeMap<&'static str, usize>> {
    let mut count = BTreeMap::new();
    for &seed in seeds {
        for s in &stream(p, seed, 48)? {
            let note = track(s, BASS).into_iter().next().ok_or("basse")?;
            let root = s.chord_root_pc;
            let degree = match (note.pitch + 12 - root) % 12 {
                0 => "root",
                3 | 4 => "third",
                6..=8 => "fifth",
                _ => "seventh",
            };
            *count.entry(degree).or_insert(0) += 1;
        }
    }
    Ok(count)
}

#[test]
fn l_arrivee_joue_la_tierce_la_quinte_ou_la_septieme_seulement_si_le_profil_le_dit() -> TestResult {
    let p = edited(&fixture()?, without_arc)?;
    let rich = first_degrees(&p, &SEEDS)?;
    for degree in ["root", "third", "fifth", "seventh"] {
        assert!(rich.get(degree).copied().unwrap_or(0) > 0, "{degree} jamais joué : {rich:?}");
    }
    let plain = edited(&p, |v| {
        let _ = walking(v)?.remove("arrival");
        Ok(())
    })?;
    let only_roots = first_degrees(&plain, &SEEDS)?;
    assert_eq!(only_roots.keys().copied().collect::<Vec<_>>(), vec!["root"], "{only_roots:?}");
    Ok(())
}

#[test]
fn un_accord_slash_garde_sa_basse_publiee() -> TestResult {
    // Chaque accord devient un accord slash sur sa quinte : la grille impose la basse, aucune
    // arrivée n'en change (D4).
    let p = edited(&fixture()?, |v| {
        without_arc(v)?;
        let loops = v["harmony"]["parts"]["Blues"]["loops"].as_array_mut().ok_or("loops")?;
        for chord in loops[0]["chords"].as_array_mut().ok_or("chords")? {
            let root = chord["semitones"].as_integer().ok_or("semitones")?;
            chord.as_table_mut().ok_or("accord")?.insert("bass".into(), ((root + 7) % 12).into());
        }
        Ok(())
    })?;
    for seed in SEEDS {
        for s in &stream(&p, seed, 24)? {
            let bass = track(s, BASS);
            for step in arrival_steps(s) {
                let note = bass.iter().find(|n| n.start_step == step).ok_or("arrivée")?;
                assert_eq!(
                    Some(note.pitch % 12),
                    bass_pc_at(s, step),
                    "graine {seed}, mesure {}, pas {step}",
                    s.index
                );
            }
        }
    }
    Ok(())
}

/// Profil de marche réduit à l'approche chromatique de chaque arrivée (fenêtre de marche large) :
/// la dernière note d'une mesure est à un demi-ton de la première de la suivante.
fn chromatic_approach_only(side: &str) -> TestResult<StyleProfile> {
    edited(&fixture()?, |v| {
        without_arc(v)?;
        let w = walking(v)?;
        w.remove("eighths");
        w.insert(
            "approach".into(),
            toml::from_str(&format!(
                "none_weight = 0\nchromatic_weight = 1\ndiatonic_weight = 0\nside = {side}"
            ))?,
        );
        Ok(())
    })
}

#[test]
fn l_approche_vise_l_arrivee_de_la_mesure_suivante_au_demi_ton() -> TestResult {
    // L'arrivée de la mesure suivante est tirée avec sa propre graine : la mesure précédente doit
    // la retrouver pour que son approche tombe à un demi-ton, tierce, quinte ou septième comprises.
    let p = chromatic_approach_only("{ from_weight = 1 }")?;
    for seed in SEEDS {
        let flow = stream(&p, seed, 36)?;
        let bars = bass_bars(&flow);
        for pair in bars.windows(2) {
            let (last, first) = (pair[0].last().ok_or("fin")?, pair[1].first().ok_or("début")?);
            assert_eq!(
                last.pitch.abs_diff(first.pitch),
                1,
                "graine {seed} : {} puis {}",
                last.pitch,
                first.pitch
            );
        }
    }
    Ok(())
}

#[test]
fn le_cote_d_approche_est_reglable() -> TestResult {
    for (side, above) in [("{ above_weight = 1 }", true), ("{ below_weight = 1 }", false)] {
        let p = chromatic_approach_only(side)?;
        for seed in SEEDS {
            let flow = stream(&p, seed, 36)?;
            let bars = bass_bars(&flow);
            for pair in bars.windows(2) {
                let (last, first) = (pair[0].last().ok_or("fin")?, pair[1].first().ok_or("début")?);
                assert_eq!(last.pitch > first.pitch, above, "graine {seed} {side}");
            }
        }
    }
    // Côté opposé à l'arrivée précédente : jamais le côté historique tant que la fenêtre le permet.
    let from = chromatic_approach_only("{ from_weight = 1 }")?;
    let other = chromatic_approach_only("{ other_weight = 1 }")?;
    let side = |p: &StyleProfile| -> TestResult<Vec<bool>> {
        let flow = stream(p, 1, 36)?;
        let bars = bass_bars(&flow);
        Ok(bars
            .windows(2)
            .map(|pair| pair[0].last().zip(pair[1].first()).is_some_and(|(a, b)| a.pitch > b.pitch))
            .collect())
    };
    assert_ne!(side(&from)?, side(&other)?, "le côté opposé change la ligne");
    Ok(())
}

/// Part des notes de passage (ni arrivée ni dernière note de la mesure) qui sont chromatiques
/// (hors gamme et hors accord), et part des notes qui répètent la précédente.
fn passing_shares(p: &StyleProfile) -> TestResult<(f64, f64)> {
    let (mut passing, mut chromatic, mut repeats, mut pairs) = (0usize, 0usize, 0usize, 0usize);
    for seed in SEEDS {
        for s in &stream(p, seed, 48)? {
            let bass = track(s, BASS);
            let arrivals = arrival_steps(s);
            for (i, n) in bass.iter().enumerate() {
                if i > 0 {
                    pairs += 1;
                    repeats += usize::from(bass[i - 1].pitch == n.pitch);
                }
                if arrivals.contains(&n.start_step) || i + 1 == bass.len() {
                    continue;
                }
                passing += 1;
                let pc = n.pitch % 12;
                chromatic += usize::from(
                    !F_MAJOR.contains(&pc) && !chord_pcs_at(s, n.start_step).contains(&pc),
                );
            }
        }
    }
    Ok((chromatic as f64 / passing as f64, repeats as f64 / pairs as f64))
}

#[test]
fn le_chromatisme_de_passage_et_les_notes_repetees_suivent_leurs_poids() -> TestResult {
    let p = edited(&fixture()?, |v| {
        without_arc(v)?;
        walking(v)?.remove("eighths");
        Ok(())
    })?;
    let plain = edited(&p, |v| {
        let passing = walking(v)?["passing"].as_table_mut().ok_or("passing")?;
        passing.remove("chromatic_weight");
        passing.remove("repeat_weight");
        Ok(())
    })?;
    let (chromatic, repeats) = passing_shares(&p)?;
    let (plain_chromatic, plain_repeats) = passing_shares(&plain)?;
    // Sans poids : jamais de passage chromatique et peu de notes répétées ; avec les poids de la
    // fixture (calibrés sur WJazzD : 29 % de passages chromatiques, 20 % de répétitions).
    assert!(plain_chromatic < 0.01, "pas de chromatisme sans poids : {plain_chromatic}");
    assert!(plain_repeats < 0.1, "répétitions sans poids : {plain_repeats}");
    assert!((0.15..=0.40).contains(&chromatic), "chromatisme de passage : {chromatic}");
    assert!((0.12..=0.30).contains(&repeats), "notes répétées : {repeats}");
    Ok(())
}

/// Les notes de pulsation d'une mesure (pas multiple de la pulsation de quatre pas).
fn on_pulse(s: &MeasureSnapshot) -> Vec<&NoteSnapshot> {
    track(s, BASS).into_iter().filter(|n| n.start_step % 4 == 0).collect()
}

#[test]
fn les_croches_s_ajoutent_sans_toucher_aux_notes_de_pulsation() -> TestResult {
    let with = edited(&fixture()?, without_arc)?;
    let without = edited(&with, |v| {
        let _ = walking(v)?.remove("eighths");
        Ok(())
    })?;
    let eighths = {
        let section = &with.accompaniment.as_ref().ok_or("accompagnement")?.bass.walking;
        section.as_ref().and_then(|w| w.eighths.clone()).ok_or("croches")?
    };
    let (mut passing, mut ghosts, mut pulses) = (0, 0, 0);
    for seed in SEEDS {
        let (a, b) = (stream(&with, seed, 48)?, stream(&without, seed, 48)?);
        for (s, t) in a.iter().zip(&b) {
            // La ligne des pulsations est celle du profil sans croches.
            let (long, plain) = (on_pulse(s), on_pulse(t));
            assert_eq!(
                long.iter().map(|n| (n.start_step, n.pitch)).collect::<Vec<_>>(),
                plain.iter().map(|n| (n.start_step, n.pitch)).collect::<Vec<_>>(),
                "graine {seed}, mesure {}",
                s.index
            );
            pulses += long.len();
            let bass = track(s, BASS);
            for n in bass.iter().filter(|n| n.start_step % 4 != 0) {
                assert_eq!(n.start_step % 4, 2, "une croche tombe à la demi-pulsation");
                let before = bass
                    .iter()
                    .rfind(|b| b.start_step < n.start_step)
                    .ok_or("note avant la croche")?;
                if n.velocity == eighths.ghost_velocity {
                    ghosts += 1;
                    assert_eq!(
                        n.pitch, before.pitch,
                        "la note fantôme répète la note de la pulsation"
                    );
                } else {
                    assert_eq!(n.velocity, eighths.velocity);
                    passing += 1;
                    assert!(n.pitch != before.pitch);
                    assert!(n.pitch.abs_diff(before.pitch) <= 12);
                }
                assert!(n.start_step + n.duration_steps <= (n.start_step / 4 + 1) * 4);
                assert!((WALK.0..=WALK.1).contains(&n.pitch));
            }
        }
    }
    assert!(passing > 0 && ghosts > 0, "croches de passage {passing}, notes fantômes {ghosts}");
    // Les poids de la fixture : 3 + 3 % de passage, 10 % de fantômes par pulsation éligible.
    let share = (passing + ghosts) as f64 / pulses as f64;
    assert!((0.05..=0.25).contains(&share), "part de pulsations suivies d'une croche : {share}");
    Ok(())
}

#[test]
fn les_croches_ne_jouent_pas_au_deux_temps() -> TestResult {
    let p = edited(&fixture()?, |v| {
        without_arc(v)?;
        let w = walking(v)?;
        w.insert(
            "eighths".into(),
            toml::from_str(
                "none_weight = 0\nghost_weight = 1\nvelocity = 50\nghost_velocity = 30",
            )?,
        );
        Ok(())
    })?;
    // Chorus 1 de la fixture (avec arc) joue le deux temps : aucune note hors pulsation.
    let with_arc = edited(&fixture()?, |v| {
        let w = walking(v)?;
        w.insert(
            "eighths".into(),
            toml::from_str(
                "none_weight = 0\nghost_weight = 1\nvelocity = 50\nghost_velocity = 30",
            )?,
        );
        Ok(())
    })?;
    for seed in SEEDS {
        for s in stream(&with_arc, seed, BLUES)?.iter() {
            assert!(track(s, BASS).iter().all(|n| n.start_step % 8 == 0), "mesure {}", s.index);
        }
        // Sans arc : toutes les pulsations éligibles reçoivent une fantôme (poids 1 sur 1).
        for s in stream(&p, seed, BLUES)?.iter() {
            let bass = track(s, BASS);
            let ghosts = bass.iter().filter(|n| n.start_step % 4 == 2).count();
            let beats = bass.iter().filter(|n| n.start_step % 4 == 0).count();
            assert!(
                ghosts >= beats - 1 && ghosts <= beats,
                "mesure {} : {ghosts}/{beats}",
                s.index
            );
        }
    }
    Ok(())
}

#[test]
fn le_deux_temps_joue_fondamentale_quinte() -> TestResult {
    // Chorus 1 de la fixture : deux temps ; approche tirée jamais (aucune note sur la fin de
    // l'étendue autre que la quinte), arrivées sur la basse.
    let p = edited(&fixture()?, |v| {
        let w = walking(v)?;
        w.remove("eighths");
        w.remove("arrival");
        w.insert(
            "approach".into(),
            toml::from_str("none_weight = 1\nchromatic_weight = 0\ndiatonic_weight = 0")?,
        );
        w.insert("two_feel".into(), toml::from_str("passing_weight = 0\nfifth_weight = 1")?);
        Ok(())
    })?;
    let mut fifths = 0;
    for seed in SEEDS {
        for s in stream(&p, seed, BLUES)?.iter() {
            let bass = track(s, BASS);
            // Mesures à un accord, hors stop-time : blanche à la fondamentale puis sa quinte.
            if s.chord_changes.len() == 1 && bass.len() == 2 {
                assert!(
                    (bass[1].pitch + 12 - bass[0].pitch) % 12 == 7,
                    "mesure {} : {} puis {}",
                    s.index,
                    bass[0].pitch,
                    bass[1].pitch
                );
                assert!(bass[0].pitch.abs_diff(bass[1].pitch) <= 7);
                fifths += 1;
            }
        }
    }
    assert!(fifths >= 6 * 5, "quintes jouées : {fifths}");
    // Sans l'option : la deuxième attaque est une note de passage, pas toujours une quinte.
    let plain = edited(&p, |v| {
        let _ = walking(v)?.remove("two_feel");
        Ok(())
    })?;
    let all_fifths = SEEDS.iter().all(|&seed| {
        stream(&plain, seed, BLUES).is_ok_and(|flow| {
            flow.iter().all(|s| {
                let bass = track(s, BASS);
                bass.len() != 2
                    || s.chord_changes.len() != 1
                    || (bass[1].pitch + 12 - bass[0].pitch) % 12 == 7
            })
        })
    });
    assert!(!all_fifths, "sans two_feel la deuxième attaque n'est pas toujours la quinte");
    Ok(())
}

#[test]
fn les_gabarits_expriment_octave_quinte_en_croches_et_motifs_syncopes() -> TestResult {
    // Basse des styles non swing : les gabarits (`accompaniment.bass.templates`) disent déjà
    // fondamentale-octave, fondamentale-quinte en croches et motifs syncopés.
    let templates = r#"
        [[templates]]
        part = "Blues"
        rhythm = [
          { step = 0, duration = 2, velocity = 80 }, { step = 2, duration = 2, velocity = 70 },
          { step = 4, duration = 2, velocity = 76 }, { step = 6, duration = 2, velocity = 70 },
          { step = 8, duration = 2, velocity = 80 }, { step = 10, duration = 2, velocity = 70 },
          { step = 12, duration = 2, velocity = 76 }, { step = 14, duration = 2, velocity = 70 },
        ]
        cycle = [{ degree = "root" }, { degree = "root", octave = 1 }]
    "#;
    let fifth_eighths = templates.replace(
        r#"cycle = [{ degree = "root" }, { degree = "root", octave = 1 }]"#,
        r#"cycle = [{ degree = "root" }, { degree = "fifth" }]"#,
    );
    let syncopated = r#"
        [[templates]]
        part = "Blues"
        rhythm = [
          { step = 0, duration = 5, velocity = 82 }, { step = 6, duration = 3, velocity = 70 },
          { step = 10, duration = 2, velocity = 74 }, { step = 14, duration = 2, velocity = 68 },
        ]
        cycle = [{ degree = "root" }, { degree = "fifth" }, { degree = "root", octave = 1 }]
    "#;
    let run = |text: &str| -> TestResult<Vec<MeasureSnapshot>> {
        let p = edited(&fixture()?, |v| {
            without_arc(v)?;
            table(v, &["accompaniment", "bass"])?.remove("walking");
            let parsed: toml::Value = toml::from_str(text)?;
            table(v, &["accompaniment", "bass"])?
                .insert("templates".into(), parsed["templates"].clone());
            Ok(())
        })?;
        stream(&p, 3, BLUES)
    };
    for (name, text, steps) in [
        ("octave", templates.to_owned(), vec![0, 2, 4, 6, 8, 10, 12, 14]),
        ("quinte", fifth_eighths, vec![0, 2, 4, 6, 8, 10, 12, 14]),
        ("syncopé", syncopated.to_owned(), vec![0, 6, 10, 14]),
    ] {
        let flow = run(&text)?;
        let mut checked = 0;
        for s in flow.iter().filter(|s| s.chord_changes.len() == 1) {
            let bass = track(s, BASS);
            assert_eq!(bass.iter().map(|n| n.start_step).collect::<Vec<_>>(), steps, "{name}");
            let root = s.chord_bass_pc;
            let degrees: Vec<u8> = bass.iter().map(|n| (n.pitch + 12 - root) % 12).collect();
            match name {
                "octave" => assert!(degrees.iter().all(|&d| d == 0), "{name} {degrees:?}"),
                "quinte" => {
                    assert!(degrees.iter().all(|&d| d == 0 || d == 7), "{name} {degrees:?}")
                }
                _ => assert!(degrees.iter().all(|&d| d == 0 || d == 7), "{name} {degrees:?}"),
            }
            if name == "octave" {
                assert!(bass.windows(2).all(|w| w[0].pitch.abs_diff(w[1].pitch) == 12), "{name}");
            }
            checked += 1;
        }
        assert!(checked >= 8, "{name}");
    }
    Ok(())
}

#[test]
fn une_mesure_se_rejoue_a_l_identique_apres_seek() -> TestResult {
    // Basse (arrivées tirées avec la graine de la mesure suivante), croches, voicings tirés et
    // dérive de registre : une mesure rejouée depuis un seek donne les mêmes notes.
    let p = fixture()?;
    let flow = stream(&p, 9, 48)?;
    for target in [2, 14, 31, 40] {
        let mut g = generator(&p);
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let _ = g.generate_measure(1, &mut rng);
        g.reset_to_initial();
        let mut rng = ChaCha8Rng::seed_from_u64(9);
        g.silent_advance(target, &mut rng);
        for bar in target..=48 {
            let again = MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng));
            assert_eq!(
                serde_json::to_string(&again)?,
                serde_json::to_string(&flow[bar - 1])?,
                "seek {target}, mesure {bar}"
            );
        }
    }
    Ok(())
}

#[test]
fn les_poids_de_basse_sont_valides_a_la_lecture() -> TestResult {
    let cases: [(&str, &str, &str); 6] = [
        ("arrival", "{ root_weight = 0 }", "accompaniment.bass.walking.arrival"),
        (
            "eighths",
            "{ none_weight = 0, velocity = 50, ghost_velocity = 30 }",
            "accompaniment.bass.walking.eighths",
        ),
        (
            "eighths",
            "{ none_weight = 1, velocity = 0, ghost_velocity = 30 }",
            "accompaniment.bass.walking.eighths.velocity",
        ),
        ("two_feel", "{ }", "accompaniment.bass.walking.two_feel"),
        ("approach", "{ none_weight = 1, side = { } }", "accompaniment.bass.walking.approach.side"),
        (
            "passing",
            "{ chord_tone_weight = 0, scale_step_weight = 0, repeat_weight = 5 }",
            "accompaniment.bass.walking.passing",
        ),
    ];
    for (key, value, field) in cases {
        let error = edited(&fixture()?, set_walking(key, value))
            .and_then(|p| p.validate().map_err(Into::into))
            .err()
            .ok_or_else(|| format!("{key} {value} devait être refusé"))?;
        assert!(error.to_string().contains(field), "{error}");
    }
    Ok(())
}

// --- A5 : voicings --------------------------------------------------------------------------

/// Comping sans attaque anticipée : chaque attaque joue l'accord en vigueur à son pas.
fn plain_comping(v: &mut toml::Value) -> TestResult {
    let c = chords(v)?;
    c.remove("phrases");
    c.insert(
        "comping".into(),
        toml::from_str::<toml::Value>(
            "x = [{ weight = 1, hits = [{ step = 0, duration = 6, velocity = 66 }, { step = 8, duration = 6, velocity = 60 }] }]",
        )?["x"]
            .clone(),
    );
    Ok(())
}

fn with_voicing(name: &'static str) -> impl FnOnce(&mut toml::Value) -> TestResult {
    move |v| {
        without_arc(v)?;
        plain_comping(v)?;
        chords(v)?.insert("voicing".into(), name.into());
        Ok(())
    }
}

/// Voicings d'une mesure : les notes d'accords groupées par attaque, hauteurs croissantes.
fn voicings(s: &MeasureSnapshot) -> Vec<(usize, Vec<u8>)> {
    let mut by_step: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    for n in track(s, CHORDS) {
        by_step.entry(n.start_step).or_default().push(n.pitch);
    }
    by_step.into_iter().collect()
}

/// Tensions naturelles (9, 11, 13) permises sans conflit avec les sons `owned` de l'accord :
/// la règle des notes à éviter, écrite ici indépendamment du moteur (demi-ton au-dessus d'un son
/// de l'accord, ou même degré altéré).
fn natural_tensions(owned: &[u8]) -> Vec<u8> {
    [(2u8, &[1u8, 2, 3][..]), (5, &[5, 6][..]), (9, &[8, 9][..])]
        .into_iter()
        .filter(|(t, same_degree)| {
            !owned.iter().any(|o| same_degree.contains(o) || (t + 12 - o) % 12 == 1)
        })
        .map(|(t, _)| t)
        .collect()
}

/// Sons permis d'un accord publié : ses classes de hauteur et ses tensions naturelles sans conflit.
fn allowed_pcs(s: &MeasureSnapshot, step: usize) -> BTreeSet<u8> {
    let pcs = chord_pcs_at(s, step);
    let root = s
        .chord_changes
        .iter()
        .rev()
        .find(|c| c.start_step <= step)
        .map_or(0, |c| c.context.root_pc);
    let owned: Vec<u8> = pcs.iter().map(|pc| (pc + 12 - root) % 12).collect();
    let mut allowed: BTreeSet<u8> = pcs.iter().copied().collect();
    allowed.extend(natural_tensions(&owned).into_iter().map(|t| (root + t) % 12));
    allowed
}

#[test]
fn chaque_famille_reste_dans_la_qualite_publiee() -> TestResult {
    // Sol-mineur-7, do-7 et fa-7 du blues : tous les sons d'un voicing sont des notes de l'accord
    // publié ou une tension naturelle que sa qualité permet ; le nombre de voix est celui de la
    // famille.
    for (family, voices) in [
        ("drop3", 4..=4),
        ("fourths", 4..=4),
        ("upper_structure", 3..=4),
        ("shell", 2..=2),
        ("open", 4..=4),
        ("drop2", 4..=4),
        ("rootless", 4..=4),
        ("guide_tones", 2..=3),
    ] {
        let p = edited(&fixture()?, with_voicing(family))?;
        let mut seen = 0;
        for seed in SEEDS {
            for s in &stream(&p, seed, 12)? {
                for (step, pitches) in voicings(s) {
                    let allowed = allowed_pcs(s, step);
                    assert!(voices.contains(&pitches.len()), "{family}: {pitches:?}");
                    assert!(
                        pitches.iter().all(|pitch| allowed.contains(&(pitch % 12))),
                        "{family}, graine {seed}, mesure {}, pas {step} : {pitches:?} hors de {allowed:?}",
                        s.index
                    );
                    assert!(pitches.windows(2).all(|w| w[0] < w[1]), "{family}: {pitches:?}");
                    seen += 1;
                }
            }
        }
        assert!(seen > 100, "{family}: {seen} voicings");
    }
    Ok(())
}

#[test]
fn les_familles_ont_la_forme_annoncee() -> TestResult {
    // Fa 7 du blues, sans tension de couleur : les intervalles caractérisent la famille.
    let shape = |family: &'static str| -> TestResult<BTreeSet<Vec<i16>>> {
        let p = edited(&fixture()?, |v| {
            with_voicing(family)(v)?;
            without_voicing_options(v)
        })?;
        let mut shapes = BTreeSet::new();
        for seed in SEEDS {
            for s in &stream(&p, seed, 12)? {
                for (_, pitches) in voicings(s) {
                    shapes.insert(
                        pitches.windows(2).map(|w| i16::from(w[1]) - i16::from(w[0])).collect(),
                    );
                }
            }
        }
        Ok(shapes)
    };
    // Quartes : trois quartes justes d'écart.
    assert!(
        shape("fourths")?.iter().all(|gaps| gaps.iter().all(|&g| g == 5)),
        "{:?}",
        shape("fourths")?
    );
    // Shell : deux voix, la tierce et la septième à une quarte, un triton, une quinte ou une sixte mineure.
    assert!(
        shape("shell")?.iter().all(|gaps| gaps.len() == 1 && matches!(gaps[0], 5..=8)),
        "{:?}",
        shape("shell")?
    );
    // Drop 3 et spread tiennent sur plus d'une octave ; drop 2 aussi, le serré non.
    let span = |family| -> TestResult<BTreeSet<i16>> {
        Ok(shape(family)?.iter().map(|gaps| gaps.iter().sum()).collect())
    };
    assert!(span("open")?.iter().all(|&s| s > 12), "{:?}", span("open")?);
    assert!(span("drop3")?.iter().all(|&s| s > 12 && s < 24), "{:?}", span("drop3")?);
    Ok(())
}

#[test]
fn la_tension_de_couleur_est_tiree_par_graine_sans_conflit_avec_la_grille() -> TestResult {
    // Une grille de qualités à altérations publiées : 7b9, 7#9, 7b13, 7#11, m7b5, maj7, m7, 13.
    let grid = [
        ("7b9", 0),
        ("7#9", 5),
        ("7b13", 7),
        ("7#11", 2),
        ("m7b5", 9),
        ("maj7", 0),
        ("m7", 2),
        ("13", 7),
    ];
    let p = edited(&fixture()?, |v| {
        with_voicing("rootless")(v)?;
        let loops = v["harmony"]["parts"]["Blues"]["loops"].as_array_mut().ok_or("loops")?;
        let chords: Vec<toml::Value> = (0..12)
            .map(|bar| {
                let (quality, root) = grid[bar % grid.len()];
                toml::from_str::<toml::Value>(&format!(
                    "semitones = {root}\nquality = \"{quality}\"\nsteps = 16"
                ))
                .map_err(Into::into)
            })
            .collect::<TestResult<_>>()?;
        loops[0]["chords"] = toml::Value::Array(chords);
        Ok(())
    })?;
    let plain = edited(&p, without_voicing_options)?;
    let mut distinct: BTreeMap<(usize, u8), BTreeSet<Vec<u8>>> = BTreeMap::new();
    let mut plain_distinct: BTreeMap<(usize, u8), BTreeSet<Vec<u8>>> = BTreeMap::new();
    for seed in 1..=16 {
        for (profile, bag) in [(&p, &mut distinct), (&plain, &mut plain_distinct)] {
            for s in &stream(profile, seed, 12)? {
                for (step, pitches) in voicings(s) {
                    let allowed = allowed_pcs(s, step);
                    assert!(
                        pitches.iter().all(|pitch| allowed.contains(&(pitch % 12))),
                        "graine {seed}, mesure {} : {pitches:?} hors de {allowed:?}",
                        s.index
                    );
                    // Jamais de 9 sur un accord qui publie b9 ou #9, de 13 sur b13, de 11 sur #11.
                    let root = s.chord_root_pc;
                    let rel: BTreeSet<u8> = pitches.iter().map(|p| (p + 12 - root) % 12).collect();
                    let name = &s.chord_name;
                    for (altered, natural) in [("b9", 2), ("#9", 2), ("b13", 9), ("#11", 5)] {
                        assert!(
                            !(name.contains(altered) && rel.contains(&natural)),
                            "{name} : {rel:?}"
                        );
                    }
                    bag.entry((s.index, step as u8)).or_default().insert(pitches);
                }
            }
        }
    }
    // Voicings distincts par accord sur 16 graines : la couleur tirée en ajoute.
    let total = |bag: &BTreeMap<(usize, u8), BTreeSet<Vec<u8>>>| -> usize {
        bag.values().map(BTreeSet::len).sum()
    };
    assert!(
        total(&distinct) > total(&plain_distinct),
        "{} contre {}",
        total(&distinct),
        total(&plain_distinct)
    );
    assert!(
        distinct.values().any(|v| v.len() >= 4),
        "au moins quatre voicings distincts pour un accord"
    );
    Ok(())
}

#[test]
fn la_derive_de_registre_est_bornee_et_varie() -> TestResult {
    let p = edited(&fixture()?, |v| with_voicing("rootless")(v))?;
    let window = {
        let c = &p.accompaniment.as_ref().ok_or("accompagnement")?.chords;
        c.register_drift.ok_or("dérive")?;
        (c.low, c.high)
    };
    let plain = edited(&p, |v| {
        let _ = chords(v)?.remove("register_drift");
        Ok(())
    })?;
    let means = |p: &StyleProfile| -> TestResult<Vec<f64>> {
        let mut means = Vec::new();
        for seed in SEEDS {
            for s in &stream(p, seed, 48)? {
                for (_, pitches) in voicings(s) {
                    means.push(
                        pitches.iter().map(|&x| f64::from(x)).sum::<f64>() / pitches.len() as f64,
                    );
                }
            }
        }
        Ok(means)
    };
    let spread = |m: &[f64]| {
        let mean = m.iter().sum::<f64>() / m.len() as f64;
        (m.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / m.len() as f64).sqrt()
    };
    let (with, without) = (means(&p)?, means(&plain)?);
    assert!(
        spread(&with) > spread(&without),
        "la dérive élargit le registre : {} contre {}",
        spread(&with),
        spread(&without)
    );
    // La fenêtre `low..=high` reste respectée.
    for seed in SEEDS {
        for s in &stream(&p, seed, 48)? {
            for (_, pitches) in voicings(s) {
                assert!(pitches[0] >= window.0 - 2 && pitches[pitches.len() - 1] <= window.1 + 2);
            }
        }
    }
    Ok(())
}

#[test]
fn les_options_de_voicings_sont_validees_a_la_lecture() -> TestResult {
    for (change, field) in [
        ("color", "accompaniment.chords.color"),
        ("register_drift", "accompaniment.chords.register_drift"),
    ] {
        let bad = |v: &mut toml::Value| -> TestResult {
            let value =
                if change == "color" { "{ none_weight = 0 }" } else { "{ step = 3, max = 2 }" };
            let parsed: toml::Value = toml::from_str(&format!("x = {value}"))?;
            chords(v)?.insert(change.into(), parsed["x"].clone());
            Ok(())
        };
        let error = edited(&fixture()?, bad)
            .and_then(|p| p.validate().map_err(Into::into))
            .err()
            .ok_or("refus attendu")?;
        assert!(error.to_string().contains(field), "{error}");
    }
    Ok(())
}

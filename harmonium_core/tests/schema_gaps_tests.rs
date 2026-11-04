//! A7 : manques de schéma comblés, observés dans le flux de `MeasureSnapshot` (principe VI).
//!
//! La fixture `grille_arc_chorus` (blues de douze mesures, quatre chorus) est réécrite par chaque
//! test : motifs de basse étiquetés sous l'arc, tirages harmoniques par chorus, accord de
//! puissance, feel de basse à trois pulsations, niveau de densité de batterie par chorus,
//! relance à plusieurs étiquettes, approche à la barre.
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

const BASS: u8 = 2;
const CHORDS: u8 = 1;
const PERCUSSION: [u8; 2] = [3, 4];
const BLUES: usize = 12;

fn fixture() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/grille_arc_chorus",
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

fn set(v: &mut toml::Value, path: &[&str], value: toml::Value) -> TestResult {
    let (last, parents) = path.split_last().ok_or("chemin vide")?;
    let mut table = v;
    for key in parents {
        table = table.get_mut(*key).ok_or_else(|| format!("section {key} absente"))?;
    }
    table.as_table_mut().ok_or("table")?.insert((*last).into(), value);
    Ok(())
}

fn parse(text: &str) -> TestResult<toml::Value> {
    Ok(toml::from_str::<toml::Table>(text)?.into())
}

fn stream_with(
    p: &StyleProfile,
    seed: u64,
    bars: usize,
    percussion_density: u8,
) -> TestResult<Vec<MeasureSnapshot>> {
    p.validate()?;
    let params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        intensity: 0.5,
        percussion_density,
        ..Default::default()
    };
    let mut g = TuneGenerator::new(params, TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect())
}

fn stream(p: &StyleProfile, seed: u64, bars: usize) -> TestResult<Vec<MeasureSnapshot>> {
    stream_with(p, seed, bars, 1)
}

fn chorus(flow: &[MeasureSnapshot], n: usize) -> &[MeasureSnapshot] {
    &flow[(n - 1) * BLUES..n * BLUES]
}

fn track(s: &MeasureSnapshot, id: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == id).collect()
}

/// Pas d'attaque de la basse, mesure par mesure.
fn bass_steps(bars: &[MeasureSnapshot]) -> Vec<Vec<usize>> {
    bars.iter().map(|s| track(s, BASS).iter().map(|n| n.start_step).collect()).collect()
}

fn percussion_notes(bars: &[MeasureSnapshot]) -> usize {
    bars.iter().flat_map(|s| &s.notes).filter(|n| PERCUSSION.contains(&n.track)).count()
}

/// Blues de basse motivique : une attaque (`sparse`), ou quatre attaques de deux dessins
/// (`dense`). La marche (et le `bass_feel` d'arc qui l'exige) est retirée.
fn motif_bass(p: &StyleProfile, arc: &str) -> TestResult<StyleProfile> {
    edited(p, |v| {
        v["accompaniment"]["bass"].as_table_mut().ok_or("basse")?.remove("walking");
        let motifs = parse(
            r#"motifs = [
  { tag = "sparse", notes = [{ step = 0, duration = 16, semitones = 0, velocity = 80 }] },
  { tag = "dense", notes = [
      { step = 0, duration = 4, semitones = 0, velocity = 80 },
      { step = 4, duration = 4, semitones = 0, velocity = 80 },
      { step = 8, duration = 4, semitones = 0, velocity = 80 },
      { step = 12, duration = 4, semitones = 0, velocity = 80 } ] },
  { tag = "dense", notes = [
      { step = 0, duration = 3, semitones = 0, velocity = 80 },
      { step = 3, duration = 3, semitones = 0, velocity = 80 },
      { step = 8, duration = 3, semitones = 0, velocity = 80 },
      { step = 11, duration = 3, semitones = 0, velocity = 80 } ] },
]"#,
        )?;
        set(v, &["accompaniment", "bass", "motifs"], motifs["motifs"].clone())?;
        set(v, &["arc"], parse(arc)?)
    })
}

const ARC_BY_TAG: &str = r#"
[[stages]]
choruses = 1
variants = [{ bass = [{ tag = "sparse" }] }]
[[stages]]
choruses = 3
variants = [{ bass = [{ tag = "dense" }] }]
"#;

const ARC_WITHOUT_BASS_TAGS: &str = r#"
[[stages]]
choruses = 4
variants = [{ intensity = 1.0 }]
"#;

#[test]
fn l_arc_restreint_la_basse_a_ses_etiquettes_par_chorus() -> TestResult {
    let p = motif_bass(&fixture()?, ARC_BY_TAG)?;
    let dense = [BTreeSet::from([0, 4, 8, 12]), BTreeSet::from([0, 3, 8, 11])];
    for seed in [1, 2, 3, 17] {
        let flow = stream(&p, seed, 4 * BLUES)?;
        for steps in bass_steps(chorus(&flow, 1)) {
            assert_eq!(steps, vec![0], "seed {seed} : l'exposition ne joue que `sparse`");
        }
        for n in 2..=4 {
            for steps in bass_steps(chorus(&flow, n)) {
                let steps: BTreeSet<usize> = steps.into_iter().collect();
                assert!(
                    dense.contains(&steps),
                    "seed {seed}, chorus {n} : {steps:?} n'est pas `dense`"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn sous_des_tags_chaque_chorus_tire_son_plan_de_basse() -> TestResult {
    let tagged = motif_bass(&fixture()?, ARC_BY_TAG)?;
    let plain = motif_bass(&fixture()?, ARC_WITHOUT_BASS_TAGS)?;
    let mut differs = 0;
    for seed in 1..=8 {
        let flow = stream(&tagged, seed, 4 * BLUES)?;
        // Chorus 2 et 3 : mêmes tags, tirages distincts.
        differs += usize::from(bass_steps(chorus(&flow, 2)) != bass_steps(chorus(&flow, 3)));
        // Sans tags de basse (ni dans l'arc ni dans la session) : le plan de l'air, rejoué à
        // l'identique à chaque chorus.
        let flow = stream(&plain, seed, 4 * BLUES)?;
        for n in 2..=4 {
            assert_eq!(
                bass_steps(chorus(&flow, 1)),
                bass_steps(chorus(&flow, n)),
                "seed {seed}, chorus {n}"
            );
        }
    }
    assert!(differs >= 6, "le plan ne varie d'un chorus à l'autre que sur {differs} graines sur 8");
    Ok(())
}

#[test]
fn le_plan_de_basse_par_chorus_se_rejoue_a_l_identique() -> TestResult {
    let p = motif_bass(&fixture()?, ARC_BY_TAG)?;
    let first = stream(&p, 5, 4 * BLUES)?;
    let again = stream(&p, 5, 4 * BLUES)?;
    assert_eq!(bass_steps(&first), bass_steps(&again));
    // Seek : le générateur rejoue depuis la mesure 1 et retrouve les mêmes tranches.
    p.validate()?;
    let params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        intensity: 0.5,
        percussion_density: 1,
        ..Default::default()
    };
    let mut g = TuneGenerator::new(params, TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(5);
    for bar in 1..=2 * BLUES {
        g.generate_measure(bar, &mut rng);
    }
    g.reset_to_initial();
    let mut rng = ChaCha8Rng::seed_from_u64(5);
    let replay: Vec<_> = (1..=3 * BLUES)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect();
    assert_eq!(bass_steps(&replay), bass_steps(&first[..3 * BLUES]));
    Ok(())
}

#[test]
fn un_tag_de_basse_inconnu_est_refuse_avec_son_chemin() -> TestResult {
    let p = motif_bass(&fixture()?, ARC_BY_TAG)?;
    let bad = edited(&p, |v| {
        set(
            v,
            &["arc"],
            parse("[[stages]]\nchoruses = 1\nvariants = [{ bass = [{ tag = \"absent\" }] }]")?,
        )
    });
    let error = bad.and_then(|p| p.validate().map_err(Into::into)).err().ok_or("tag accepté")?;
    assert!(error.to_string().contains("bass"), "{error}");
    Ok(())
}

/// Deux boucles de blues qui ne partagent ni accord ni fondamentale à la mesure 1.
fn two_loops(p: &StyleProfile, variants: i64) -> TestResult<StyleProfile> {
    edited(p, |v| {
        let part = &mut v["harmony"]["parts"]["Blues"];
        let alt = |root: u8, quality: &str| {
            format!(
                "{{ weight = 1, chords = [{{ semitones = {root}, quality = \"{quality}\", steps = 192 }}] }}"
            )
        };
        *part =
            parse(&format!("kind = \"chord_loop\"\nloops = [{}, {}]", alt(0, "7"), alt(7, "m7")))?;
        set(v, &["harmony", "tour_variants"], variants.into())
    })
}

fn chord_per_chorus(flow: &[MeasureSnapshot]) -> Vec<(i32, String)> {
    (1..=4)
        .map(|n| {
            let s = &chorus(flow, n)[0];
            (s.chord_root_offset, s.chord_name.clone())
        })
        .collect()
}

#[test]
fn tour_variants_change_la_boucle_d_un_chorus_au_suivant() -> TestResult {
    let single = two_loops(&fixture()?, 1)?;
    let varied = two_loops(&fixture()?, 2)?;
    let mut changes = 0;
    for seed in 1..=8 {
        let one = chord_per_chorus(&stream(&single, seed, 4 * BLUES)?);
        assert!(one.iter().all(|c| *c == one[0]), "seed {seed} : un seul tirage répété : {one:?}");
        let two = chord_per_chorus(&stream(&varied, seed, 4 * BLUES)?);
        // Le premier tirage est celui de l'air ; le cycle revient au premier un chorus sur deux.
        assert_eq!(two[0], one[0], "seed {seed} : le chorus 1 garde le tirage de l'air");
        assert_eq!(two[0], two[2], "seed {seed}");
        assert_eq!(two[1], two[3], "seed {seed}");
        changes += usize::from(two[0] != two[1]);
    }
    assert!(changes >= 2, "la boucle change d'un chorus au suivant sur {changes} graines sur 8");
    Ok(())
}

#[test]
fn une_boucle_limitee_a_un_mode_ne_se_tire_que_dans_ce_mode() -> TestResult {
    // La fixture est en ionien : la boucle réservée au dorien ne doit jamais sonner.
    let p = edited(&two_loops(&fixture()?, 4)?, |v| {
        let loops = v["harmony"]["parts"]["Blues"]["loops"].as_array_mut().ok_or("boucles")?;
        loops[1]
            .as_table_mut()
            .ok_or("boucle")?
            .insert("modes".into(), toml::Value::Array(vec!["dorian".into()]));
        Ok(())
    })?;
    for seed in 1..=8 {
        let flow = stream(&p, seed, 4 * BLUES)?;
        for (offset, _) in chord_per_chorus(&flow) {
            assert_eq!(offset, 0, "seed {seed} : la boucle dorienne sonne en ionien");
        }
    }
    Ok(())
}

#[test]
fn l_accord_de_puissance_joue_fondamentale_quinte_et_octave() -> TestResult {
    let p = edited(&fixture()?, |v| {
        let part = &mut v["harmony"]["parts"]["Blues"];
        *part = parse(
            "kind = \"chord_loop\"\nloops = [{ chords = [{ semitones = 0, quality = \"5\", steps = 192 }] }]",
        )?;
        Ok(())
    })?;
    let flow = stream(&p, 3, 2 * BLUES)?;
    let mut played = false;
    for s in &flow {
        assert_eq!(s.chord_quality.map(|q| q.to_string()), Some("5".to_string()));
        assert_eq!(s.chord_pitch_classes.len(), 2, "fondamentale et quinte seules");
        let pcs: BTreeSet<u8> = track(s, CHORDS).iter().map(|n| n.pitch % 12).collect();
        played |= !pcs.is_empty();
        assert!(
            pcs.iter().all(|pc| s.chord_pitch_classes.contains(pc)),
            "mesure {} : {pcs:?} hors de {:?}",
            s.index,
            s.chord_pitch_classes
        );
    }
    assert!(played, "le comping joue l'accord de puissance");
    Ok(())
}

#[test]
fn le_feel_a_trois_pulsations_ne_joue_qu_une_attaque_par_mesure() -> TestResult {
    let p = fixture()?;
    let feel = |name: &str| {
        edited(&p, |v| {
            set(
                v,
                &["arc"],
                parse(&format!(
                    "[[stages]]\nchoruses = 4\nvariants = [{{ bass_feel = \"{name}\" }}]"
                ))?,
            )
        })
    };
    // Quatre pulsations par mesure : « trois pulsations » joue les temps 1 et 4, `bar_feel` le un.
    let walking = stream(&feel("walking")?, 2, BLUES)?;
    let three = stream(&feel("three_feel")?, 2, BLUES)?;
    let bar = stream(&feel("bar_feel")?, 2, BLUES)?;
    for ((w, t), b) in walking.iter().zip(&three).zip(&bar) {
        let changes = w.chord_changes.len().max(1);
        assert!(track(b, BASS).len() <= changes + 1, "mesure {}", b.index);
        assert!(track(t, BASS).len() < track(w, BASS).len(), "mesure {}", t.index);
        assert!(track(b, BASS).len() <= track(t, BASS).len(), "mesure {}", b.index);
    }
    Ok(())
}

#[test]
fn le_niveau_de_densite_de_l_arc_remplace_celui_du_jeu() -> TestResult {
    let level = |density: u8| {
        edited(&fixture()?, |v| {
            set(
                v,
                &["arc"],
                parse(&format!(
                    "[[stages]]\nchoruses = 4\nvariants = [{{ percussion_density = {density} }}]"
                ))?,
            )
        })
    };
    // Quel que soit le niveau demandé par l'hôte, celui de l'arc décide.
    let mut counts = Vec::new();
    for density in [0, 1, 2] {
        let a = percussion_notes(&stream_with(&level(density)?, 4, 2 * BLUES, 0)?);
        let b = percussion_notes(&stream_with(&level(density)?, 4, 2 * BLUES, 2)?);
        assert_eq!(a, b, "niveau {density} : l'hôte ne décide plus");
        counts.push(a);
    }
    assert!(counts[0] < counts[2], "{counts:?}");
    let bad = edited(&fixture()?, |v| {
        set(
            v,
            &["arc"],
            parse("[[stages]]\nchoruses = 1\nvariants = [{ percussion_density = 3 }]")?,
        )
    });
    assert!(bad.and_then(|p| p.validate().map_err(Into::into)).is_err());
    Ok(())
}

#[test]
fn le_decalage_de_tempo_peut_depasser_trente_battements() -> TestResult {
    let wide = |offset: f64| {
        edited(&fixture()?, |v| {
            let curve = format!("[[0.0, {}], [0.5, 0.0], [1.0, {offset}]]", -offset);
            set(v, &["dynamics", "tempo_offset"], parse(&format!("c = {curve}"))?["c"].clone())
        })
    };
    let tempo = |offset: f64, intensity: f32| -> TestResult<f32> {
        let p = wide(offset)?;
        p.validate()?;
        let params = MusicalParams {
            bpm: p.meter.default_bpm,
            time_signature: p.meter.time_signature,
            intensity,
            ..Default::default()
        };
        let mut g = TuneGenerator::new(params, TuningParams { profile: p });
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        Ok(g.generate_measure(1, &mut rng).tempo)
    };
    assert!((tempo(60.0, 1.0)? - tempo(60.0, 0.0)? - 120.0).abs() < 1.0);
    assert!(wide(121.0).and_then(|p| p.validate().map_err(Into::into)).is_err());
    Ok(())
}

fn drawn(p: &StyleProfile, seed: u64, force: &[(&str, usize)]) -> TestResult<Vec<(String, usize)>> {
    p.validate()?;
    let params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        ..Default::default()
    };
    let mut g = TuneGenerator::new(params, TuningParams { profile: p.clone() });
    for (group, index) in force {
        g.force_session_variant(group, *index)?;
    }
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let measure = g.generate_measure(1, &mut rng);
    Ok(measure.session.iter().map(|v| (v.group.clone(), v.index)).collect())
}

#[test]
fn une_variante_imposee_l_emporte_sur_la_graine_sans_deplacer_les_autres_groupes() -> TestResult {
    let p = StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/grille_habillage",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    const BRUSHES: usize = 13;
    let mut sticks = 0;
    for seed in 1..=16 {
        let free = drawn(&p, seed, &[])?;
        let forced = drawn(&p, seed, &[("kit", BRUSHES)])?;
        sticks += usize::from(free.contains(&("kit".to_string(), 12)));
        assert!(forced.contains(&("kit".to_string(), BRUSHES)), "seed {seed} : {forced:?}");
        let others = |v: &[(String, usize)]| -> Vec<(String, usize)> {
            v.iter().filter(|(group, _)| group != "kit").cloned().collect()
        };
        assert_eq!(others(&free), others(&forced), "seed {seed} : un autre groupe a bougé");
    }
    assert!(sticks > 0, "le tirage libre doit donner des baguettes sur au moins une graine");
    // Variante d'un autre groupe ou hors de la liste : refusée, rien n'est imposé.
    let params = MusicalParams { bpm: p.meter.default_bpm, ..Default::default() };
    let mut g = TuneGenerator::new(params, TuningParams { profile: p });
    assert!(g.force_session_variant("kit", 3).is_err());
    assert!(g.force_session_variant("kit", 99).is_err());
    assert!(g.forced_session_variants().is_empty());
    Ok(())
}

fn percussion_signature(flow: &[MeasureSnapshot]) -> Vec<Vec<(u8, usize, u8)>> {
    flow.iter()
        .map(|s| {
            s.notes
                .iter()
                .filter(|n| PERCUSSION.contains(&n.track))
                .map(|n| (n.track, n.start_step, n.pitch))
                .collect()
        })
        .collect()
}

#[test]
fn une_relance_a_plusieurs_etiquettes_concourt_sous_chacune() -> TestResult {
    let phrases = StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/grille_phrases",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    // `tag = "x"` devient `tags = ["x"]` : même tirage, même batterie.
    let listed = edited(&phrases, |v| {
        for bank in ["fills", "phrases"] {
            for item in v["percussion"][bank].as_array_mut().ok_or("banque")? {
                let table = item.as_table_mut().ok_or("élément")?;
                if let Some(tag) = table.remove("tag") {
                    table.insert("tags".into(), toml::Value::Array(vec![tag]));
                }
            }
        }
        Ok(())
    })?;
    for seed in [1, 2, 3] {
        assert_eq!(
            percussion_signature(&stream(&phrases, seed, 4 * BLUES)?),
            percussion_signature(&stream(&listed, seed, 4 * BLUES)?),
            "seed {seed}"
        );
    }
    // Un tag qui n'est porté que par `tags` est connu de l'arc ; un tag porté par personne, non.
    let alias = edited(&listed, |v| {
        let fills = v["percussion"]["fills"].as_array_mut().ok_or("relances")?;
        fills[0]
            .as_table_mut()
            .ok_or("relance")?
            .insert("tags".into(), toml::Value::Array(vec!["alias".into()]));
        set(
            v,
            &["arc"],
            parse("[[stages]]\nchoruses = 4\nvariants = [{ percussion = [{ tag = \"alias\" }] }]")?,
        )
    })?;
    alias.validate()?;
    let unknown = edited(&alias, |v| {
        set(
            v,
            &["arc"],
            parse("[[stages]]\nchoruses = 4\nvariants = [{ percussion = [{ tag = \"nul\" }] }]")?,
        )
    });
    assert!(unknown.is_err(), "un tag que personne ne porte est accepté");
    Ok(())
}

#[test]
fn l_approche_a_la_barre_ne_precede_qu_un_changement_d_accord() -> TestResult {
    let p = motif_bass(&fixture()?, ARC_WITHOUT_BASS_TAGS)?;
    let with_passing = edited(&p, |v| {
        set(
            v,
            &["accompaniment", "bass", "passing"],
            parse(
                r#"none_weight = 0
density = [[0.0, 1.0], [1.0, 1.0]]
phrase = [[0.0, 1.0], [1.0, 1.0]]
avoid_melody_intervals = []
rules = [{ when = "chord_change_at_bar", before = [2], motion = "chromatic", weight = 1, duration = 2, velocity_delta = -10, min_leap = 3 }]"#,
            )?,
        )
    })?;
    let mut approached = 0;
    for seed in [1, 2, 3] {
        let flow = stream(&with_passing, seed, 2 * BLUES)?;
        // Mesures 1 à 11 d'un chorus : un seul accord par mesure (le blues change à la barre).
        for chorus_bars in [&flow[..BLUES - 1], &flow[BLUES..2 * BLUES - 1]] {
            for pair in chorus_bars.windows(2) {
                let (bar, next) = (&pair[0], &pair[1]);
                let changes = (bar.chord_root_offset, &bar.chord_name)
                    != (next.chord_root_offset, &next.chord_name);
                let approach = track(bar, BASS).iter().any(|n| n.start_step == 14);
                assert!(
                    !approach || changes,
                    "seed {seed}, mesure {} : approche sans changement",
                    bar.index
                );
                approached += usize::from(approach);
            }
        }
    }
    assert!(approached > 0, "aucune approche à la barre jouée");
    Ok(())
}

#[test]
fn une_variante_se_designe_par_son_nom() -> TestResult {
    let p = StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/grille_habillage",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let params = MusicalParams { bpm: p.meter.default_bpm, ..Default::default() };
    let mut g = TuneGenerator::new(params, TuningParams { profile: p });
    g.force_session_variant_named("kit", "balais")?;
    assert_eq!(g.forced_session_variants(), [("kit".to_string(), 13)]);
    let error = g.force_session_variant_named("kit", "absent").err().ok_or("accepté")?;
    assert!(error.contains("baguettes") && error.contains("balais"), "{error}");
    assert!(g.force_session_variant_named("nul", "balais").is_err());
    Ok(())
}

#[test]
fn un_break_hors_du_chorus_n_est_pas_refuse() -> TestResult {
    // L'arc de la fixture a des breaks `start = -1` et `-2` ; un break en positif au-delà des
    // douze mesures du blues est ignoré, un break plus long que le chorus est borné.
    let p = edited(&fixture()?, |v| {
        set(
            v,
            &["arc"],
            parse(
                "[[stages]]\nchoruses = 4\nvariants = [{ breaks = [{ start = 20, bars = 2 }, { start = 11, bars = 9 }, { start = -40, bars = 1 }] }]",
            )?,
        )
    })?;
    let flow = stream(&p, 1, 2 * BLUES)?;
    // Mesure 11 et 12 : silence de la basse (break `start = 11` borné), mesures 1 à 10 jouées.
    assert!(flow[..10].iter().any(|s| !track(s, BASS).is_empty()));
    Ok(())
}

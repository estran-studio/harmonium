//! A3 : l'arc sur plusieurs chorus, observé dans le flux de `MeasureSnapshot` (principe VI).
//!
//! La fixture `grille_arc_chorus` (copie de `wjazzd_swing` réduite au blues de douze mesures, avec
//! `arc.toml`) joue quatre chorus. Un chorus est une passe du blues : mesures 1 à 12, 13 à 24, 25
//! à 36, 37 à 48. Les tests variantes réécrivent la section `[arc]`, la durée ou le set.
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

const CHORDS: u8 = 1;
const BASS: u8 = 2;
const PERCUSSION: [u8; 2] = [3, 4];
const RIDE: u8 = 51;
const HAT: u8 = 42;
const BLUES: usize = 12;

fn fixture_dir() -> String {
    format!("{}/../tests/fixtures/profiles/grille_arc_chorus", env!("CARGO_MANIFEST_DIR"))
}

fn fixture() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(fixture_dir())?)
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

/// Même profil, `[arc]` remplacé par `arc` (TOML de la section).
fn with_arc(p: &StyleProfile, arc: &str) -> TestResult<StyleProfile> {
    edited(p, |v| {
        v.as_table_mut().ok_or("profil")?.insert("arc".into(), toml::from_str(arc)?);
        Ok(())
    })
}

/// Même profil joué `tours` chorus, dans un set de `tunes` airs (0 : set sans fin).
fn with_run(p: &StyleProfile, tours: i64, tunes: i64) -> TestResult<StyleProfile> {
    edited(p, |v| {
        v["arrangement"]["tours_per_tune"] = tours.into();
        let set = v["set"].as_table_mut().ok_or("set")?;
        if tunes == 0 {
            set.remove("length");
        } else {
            let length: toml::Value =
                toml::from_str(&format!("length = [{{ tunes = {tunes}, weight = 1 }}]"))?;
            set.insert("length".into(), length["length"].clone());
        }
        Ok(())
    })
}

fn stream(p: &StyleProfile, seed: u64, bars: usize) -> TestResult<Vec<MeasureSnapshot>> {
    p.validate()?;
    let params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        intensity: 0.5,
        percussion_density: 1,
        ..Default::default()
    };
    let mut g = TuneGenerator::new(params, TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect())
}

fn chorus_of(s: &MeasureSnapshot) -> usize {
    s.position.map_or(0, |p| p.chorus())
}

/// Les mesures du chorus `chorus` (1-based), dans l'ordre.
fn chorus(flow: &[MeasureSnapshot], chorus: usize) -> Vec<&MeasureSnapshot> {
    flow.iter().filter(|s| chorus_of(s) == chorus).collect()
}

/// Mesures du chorus sans la dernière (celle de la relance de batterie et du tournant).
fn body(flow: &[MeasureSnapshot], n: usize) -> Vec<&MeasureSnapshot> {
    let mut bars = chorus(flow, n);
    bars.truncate(BLUES - 1);
    bars
}

fn track(s: &MeasureSnapshot, id: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == id).collect()
}

fn percussion(s: &MeasureSnapshot) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| PERCUSSION.contains(&n.track)).collect()
}

fn attacks(s: &MeasureSnapshot, id: u8) -> BTreeSet<usize> {
    track(s, id).iter().map(|n| n.start_step).collect()
}

/// Attaques de basse par mesure : 2 au deux temps, 4 à la marche.
fn bass_per_bar(bars: &[&MeasureSnapshot]) -> Vec<usize> {
    bars.iter().map(|s| track(s, BASS).len()).collect()
}

fn plays(bars: &[&MeasureSnapshot], pitch: u8) -> bool {
    bars.iter().any(|s| percussion(s).iter().any(|n| n.pitch == pitch))
}

const SEEDS: [u64; 4] = [1, 2, 3, 17];

#[test]
fn le_numero_de_chorus_se_lit_dans_la_position_du_flux() -> TestResult {
    let p = fixture()?;
    let flow = stream(&p, 1, 48)?;
    for s in &flow {
        let position = s.position.ok_or("position publiée")?;
        assert_eq!(position.chorus(), (s.index - 1) / BLUES + 1, "mesure {}", s.index);
        assert_eq!(position.tour, position.chorus(), "un air : chorus = tour");
        assert_eq!(position.choruses(), Some(4), "mesure {}", s.index);
    }
    // Set sans fin : le numéro se compte quand même, le nombre de chorus est inconnu.
    let endless = stream(&with_run(&p, 4, 0)?, 1, 48)?;
    for s in &endless {
        let position = s.position.ok_or("position publiée")?;
        assert_eq!(position.chorus(), (s.index - 1) / BLUES + 1);
        assert_eq!(position.choruses(), None);
    }
    // Deux airs de deux chorus dans le même set : la numérotation continue d'un air à l'autre.
    let two_tunes = stream(&with_run(&p, 2, 2)?, 1, 48)?;
    for s in &two_tunes {
        let position = s.position.ok_or("position publiée")?;
        assert_eq!(position.chorus(), (s.index - 1) / BLUES + 1, "mesure {}", s.index);
        assert_eq!(position.choruses(), Some(4));
    }
    assert_eq!(two_tunes[24].position.map(|p| (p.tune_index, p.tour)), Some((1, 1)));
    Ok(())
}

#[test]
fn le_feel_de_basse_bascule_du_deux_temps_a_la_marche_puis_revient() -> TestResult {
    let p = fixture()?;
    for seed in SEEDS {
        let flow = stream(&p, seed, 48)?;
        // Chorus 1 et 4 (retombée) : blanches aux temps 1 et 3, tenues jusqu'à l'attaque suivante ;
        // le stop-time de la retombée garde le premier temps des deux dernières mesures.
        for n in [1, 4] {
            for s in chorus(&flow, n) {
                let bar = (s.index - 1) % BLUES + 1;
                let stopped = n == 4 && bar >= BLUES - 1;
                let expected = if stopped { vec![(0, 8)] } else { vec![(0, 8), (8, 8)] };
                assert_eq!(
                    track(s, BASS)
                        .iter()
                        .map(|b| (b.start_step, b.duration_steps))
                        .collect::<Vec<_>>(),
                    expected,
                    "graine {seed}, chorus {n}, mesure {}",
                    s.index
                );
            }
        }
        // Chorus 2 et 3 : une attaque par pulsation (hors la dernière mesure, où une variante
        // tire un break).
        for n in [2, 3] {
            for s in body(&flow, n) {
                assert_eq!(
                    track(s, BASS).iter().map(|b| b.start_step).collect::<Vec<_>>(),
                    vec![0, 4, 8, 12],
                    "graine {seed}, chorus {n}, mesure {}",
                    s.index
                );
            }
        }
    }
    Ok(())
}

#[test]
fn le_deux_temps_arrive_sur_la_basse_de_l_accord_a_chaque_changement() -> TestResult {
    let p = fixture()?;
    for seed in SEEDS {
        let flow = stream(&p, seed, 48)?;
        for s in chorus(&flow, 1) {
            let bass = track(s, BASS);
            for change in &s.chord_changes {
                let note = bass
                    .iter()
                    .find(|b| b.start_step == change.start_step)
                    .ok_or("attaque de basse sur le changement d'accord")?;
                assert_eq!(
                    note.pitch % 12,
                    change.context.bass_pc,
                    "graine {seed}, mesure {}, pas {}",
                    s.index,
                    change.start_step
                );
                assert!((36..=47).contains(&note.pitch), "arrivée dans le registre");
            }
            // Les attaques hors changement d'accord restent dans la fenêtre de marche.
            assert!(bass.iter().all(|b| (33..=50).contains(&b.pitch)));
        }
    }
    Ok(())
}

#[test]
fn le_comping_passe_du_clairseme_au_dense_selon_les_etapes() -> TestResult {
    let p = fixture()?;
    for seed in SEEDS {
        let flow = stream(&p, seed, 48)?;
        let most =
            |n: usize| body(&flow, n).iter().map(|s| attacks(s, CHORDS).len()).max().unwrap_or(0);
        // Exposition : motifs `sparse` et `medium`, au plus deux attaques par mesure.
        assert!(most(1) <= 2, "graine {seed}, chorus 1 : {} attaques", most(1));
        // Retombée : motifs `sparse` seulement, au plus une attaque.
        assert!(most(4) <= 1, "graine {seed}, chorus 4 : {} attaques", most(4));
        // Le corps joue au moins une mesure à trois attaques (motifs `dense`) sur ses deux chorus.
        assert!(most(2).max(most(3)) >= 3, "graine {seed}");
    }
    Ok(())
}

#[test]
fn la_batterie_change_de_pattern_et_de_densite_par_etape() -> TestResult {
    let p = fixture()?;
    for seed in SEEDS {
        let flow = stream(&p, seed, 48)?;
        for n in [1, 4] {
            let bars = body(&flow, n);
            assert!(
                plays(&bars, HAT) && !plays(&bars, RIDE),
                "graine {seed}, chorus {n} : charleston"
            );
        }
        for n in [2, 3] {
            let bars = body(&flow, n);
            assert!(plays(&bars, RIDE) && !plays(&bars, HAT), "graine {seed}, chorus {n} : ride");
        }
        // Densité : l'exposition (densité 0 du kit) joue moins de coups que le corps.
        let hits = |n: usize| body(&flow, n).iter().map(|s| percussion(s).len()).sum::<usize>();
        assert!(hits(1) < hits(2), "graine {seed} : {} contre {}", hits(1), hits(2));
    }
    Ok(())
}

#[test]
fn l_intensite_de_l_arc_change_les_velocites_sans_toucher_au_tempo() -> TestResult {
    let p = fixture()?;
    for seed in SEEDS {
        let flow = stream(&p, seed, 48)?;
        // Vélocité de l'arrivée du premier temps : intensité 0,6 à 0,7 (exposition) sous 1,0 à 1,2.
        let arrival = |n: usize| -> Vec<u8> {
            chorus(&flow, n)
                .iter()
                .filter_map(|s| {
                    track(s, BASS).iter().find(|b| b.start_step == 0).map(|b| b.velocity)
                })
                .collect()
        };
        let quiet = arrival(1).into_iter().max().ok_or("arrivées")?;
        let loud = arrival(2).into_iter().chain(arrival(3)).min().ok_or("arrivées")?;
        assert!(quiet < loud, "graine {seed} : {quiet} contre {loud}");
        assert!(flow.iter().all(|s| s.tempo == flow[0].tempo), "le tempo ne suit pas l'arc");
    }
    Ok(())
}

/// Chorus isolé : une seule variante, un seul break.
const BREAK_ARC: &str = r#"
[[stages]]
variants = [{ breaks = [{ start = 3, bars = 2, tracks = ["Chords", "Bass"] }] }]
"#;

#[test]
fn un_break_se_joue_aux_mesures_donnees_de_chaque_chorus() -> TestResult {
    let p = with_arc(&fixture()?, BREAK_ARC)?;
    let flow = stream(&p, 1, 48)?;
    for s in &flow {
        let bar = (s.index - 1) % BLUES + 1;
        let silent = track(s, CHORDS).is_empty() && track(s, BASS).is_empty();
        assert_eq!(silent, (3..=4).contains(&bar), "mesure {} (mesure {bar} du chorus)", s.index);
        assert!(!percussion(s).is_empty(), "la batterie continue, mesure {}", s.index);
    }
    Ok(())
}

#[test]
fn un_stop_time_ne_garde_que_les_pas_demandes_et_compte_depuis_la_fin() -> TestResult {
    let arc = r#"
[[stages]]
variants = [{ breaks = [{ start = -2, bars = 2, tracks = ["Chords", "Bass"], keep_steps = [0] }] }]
"#;
    let p = with_arc(&fixture()?, arc)?;
    let flow = stream(&p, 1, 48)?;
    for s in &flow {
        let bar = (s.index - 1) % BLUES + 1;
        for id in [CHORDS, BASS] {
            let steps = attacks(s, id);
            if bar >= BLUES - 1 {
                assert!(
                    steps.iter().all(|&step| step == 0),
                    "mesure {}, piste {id} : {steps:?}",
                    s.index
                );
            }
        }
        if bar == BLUES - 1 || bar == BLUES {
            assert!(
                !attacks(s, BASS).is_empty(),
                "le premier temps reste joué, mesure {}",
                s.index
            );
        }
        if bar < BLUES - 1 {
            assert_eq!(track(s, BASS).len(), 4, "mesure {} intacte", s.index);
        }
    }
    Ok(())
}

#[test]
fn un_break_tire_au_sort_n_arrive_que_certains_chorus_et_certaines_graines() -> TestResult {
    let arc = r#"
[[stages]]
variants = [{ breaks = [{ start = 1, bars = 1, chance = 0.5 }] }]
"#;
    let p = with_arc(&fixture()?, arc)?;
    let mut silent = Vec::new();
    for seed in 1..=8 {
        let flow = stream(&p, seed, 48)?;
        silent.push((1..=4).map(|n| chorus(&flow, n)[0].notes.is_empty()).collect::<Vec<bool>>());
        // Le tirage se fait par chorus : le reste du chorus joue toujours.
        assert!(chorus(&flow, 1).iter().skip(1).all(|s| !s.notes.is_empty()));
    }
    let breaks: usize = silent.iter().flatten().filter(|&&b| b).count();
    assert!((8..=24).contains(&breaks), "8 graines × 4 chorus à une chance sur deux : {breaks}");
    assert!(silent.iter().any(|s| s.iter().any(|&b| b) && s.iter().any(|&b| !b)), "{silent:?}");
    Ok(())
}

/// La variante de chaque chorus se distingue par le feel de basse, sans autre réglage.
const FALL_ARC: &str = r#"
[[stages]]
variants = [{ bass_feel = "walking" }]
[[fall]]
bass_feel = "two_feel"
"#;

#[test]
fn la_retombee_joue_au_dernier_chorus_quand_le_nombre_de_chorus_est_connu() -> TestResult {
    let p = with_arc(&with_run(&fixture()?, 3, 1)?, FALL_ARC)?;
    let flow = stream(&p, 1, 36)?;
    assert_eq!(bass_per_bar(&chorus(&flow, 1)), vec![4; BLUES]);
    assert_eq!(bass_per_bar(&chorus(&flow, 2)), vec![4; BLUES]);
    assert_eq!(bass_per_bar(&chorus(&flow, 3)), vec![2; BLUES], "dernier chorus : retombée");
    // Deux airs d'un set de deux : la retombée est au dernier chorus du set, pas de l'air.
    let p = with_arc(&with_run(&fixture()?, 2, 2)?, FALL_ARC)?;
    let flow = stream(&p, 1, 48)?;
    assert_eq!(bass_per_bar(&chorus(&flow, 2)), vec![4; BLUES], "dernier chorus du premier air");
    assert_eq!(bass_per_bar(&chorus(&flow, 3)), vec![4; BLUES]);
    assert_eq!(bass_per_bar(&chorus(&flow, 4)), vec![2; BLUES]);
    Ok(())
}

#[test]
fn sans_fin_connue_ou_avec_un_seul_chorus_il_n_y_a_pas_de_retombee() -> TestResult {
    // Set sans fin : aucun chorus n'est le dernier.
    let p = with_arc(&with_run(&fixture()?, 3, 0)?, FALL_ARC)?;
    let flow = stream(&p, 1, 36)?;
    for n in 1..=3 {
        assert_eq!(bass_per_bar(&chorus(&flow, n)), vec![4; BLUES], "chorus {n}");
    }
    // Un seul chorus : pas d'arc à retomber.
    let p = with_arc(&with_run(&fixture()?, 1, 1)?, FALL_ARC)?;
    assert_eq!(bass_per_bar(&chorus(&stream(&p, 1, 12)?, 1)), vec![4; BLUES]);
    Ok(())
}

#[test]
fn les_etapes_se_tiennent_ou_reprennent_sans_fin_connue() -> TestResult {
    // Trois étapes à un chorus : deux temps, marche, deux temps ; au-delà, la dernière se tient.
    let steps = r#"
[[stages]]
variants = [{ bass_feel = "two_feel" }]
[[stages]]
variants = [{ bass_feel = "walking" }]
[[stages]]
variants = [{ bass_feel = "two_feel" }]
"#;
    let feel = |flow: &[MeasureSnapshot], n| bass_per_bar(&chorus(flow, n))[0];
    let p = with_arc(&with_run(&fixture()?, 6, 0)?, steps)?;
    let flow = stream(&p, 1, 72)?;
    assert_eq!((1..=6).map(|n| feel(&flow, n)).collect::<Vec<_>>(), vec![2, 4, 2, 2, 2, 2]);
    // `repeat_from = 1` : après la dernière étape, retour à la marche.
    let looped = format!("repeat_from = 1\n{steps}");
    let p = with_arc(&with_run(&fixture()?, 6, 0)?, &looped)?;
    let flow = stream(&p, 1, 72)?;
    assert_eq!((1..=6).map(|n| feel(&flow, n)).collect::<Vec<_>>(), vec![2, 4, 2, 4, 2, 4]);
    // Une étape de deux chorus couvre deux chorus consécutifs.
    let long = r#"
[[stages]]
choruses = 2
variants = [{ bass_feel = "two_feel" }]
[[stages]]
variants = [{ bass_feel = "walking" }]
"#;
    let p = with_arc(&with_run(&fixture()?, 4, 0)?, long)?;
    let flow = stream(&p, 1, 48)?;
    assert_eq!((1..=4).map(|n| feel(&flow, n)).collect::<Vec<_>>(), vec![2, 2, 4, 4]);
    Ok(())
}

/// Signature d'un chorus qui ne lit que l'arc (jamais le comping ni la ligne de basse, que la
/// graine de session fait peu varier à grille égale) : feel de basse, charleston ou ride,
/// vélocité de l'arrivée (intensité tirée), dernière mesure silencieuse (break tiré).
fn fingerprint(flow: &[MeasureSnapshot], n: usize) -> (usize, bool, bool, u8) {
    let bars = body(flow, n);
    let velocity = track(bars[0], BASS).first().map_or(0, |b| b.velocity);
    let broken = chorus(flow, n).last().is_some_and(|s| s.notes.iter().all(|n| n.track >= 3));
    (bass_per_bar(&bars)[0], plays(&bars, HAT), broken, velocity)
}

#[test]
fn meme_graine_meme_sortie_graines_differentes_arcs_differents() -> TestResult {
    let p = fixture()?;
    let a = stream(&p, 5, 48)?;
    let b = stream(&p, 5, 48)?;
    assert_eq!(serde_json::to_string(&a)?, serde_json::to_string(&b)?);
    let arcs: BTreeSet<Vec<(usize, bool, bool, u8)>> = (1..=16)
        .map(|seed| {
            let flow = stream(&p, seed, 48)?;
            Ok((1..=4).map(|n| fingerprint(&flow, n)).collect())
        })
        .collect::<TestResult<_>>()?;
    assert!(arcs.len() >= 4, "16 graines, {} arcs distincts", arcs.len());
    // Les chorus d'une même graine ne se jouent pas tous pareil.
    let varied = (1..=16u64).any(|seed| {
        let flow = stream(&p, seed, 48).unwrap_or_default();
        fingerprint(&flow, 2) != fingerprint(&flow, 3)
    });
    assert!(varied, "un arc n'est pas une boucle");
    Ok(())
}

#[test]
fn une_mesure_se_rejoue_a_l_identique_depuis_son_indice_et_la_graine() -> TestResult {
    let p = fixture()?;
    let flow = stream(&p, 9, 48)?;
    // Un autre générateur, remis à zéro puis avancé en silence jusqu'à la mesure 31 (chorus 3).
    let params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        intensity: 0.5,
        percussion_density: 1,
        ..Default::default()
    };
    let mut g = TuneGenerator::new(params, TuningParams { profile: p });
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let _ = g.generate_measure(1, &mut rng);
    g.reset_to_initial();
    let mut rng = ChaCha8Rng::seed_from_u64(9);
    g.silent_advance(31, &mut rng);
    for bar in 31..=48 {
        let again = MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng));
        assert_eq!(
            serde_json::to_string(&again)?,
            serde_json::to_string(&flow[bar - 1])?,
            "mesure {bar}"
        );
    }
    Ok(())
}

// --- Validation du schéma ---------------------------------------------------------------------

fn invalid(arc: &str) -> TestResult<String> {
    let error = with_arc(&fixture()?, arc).err().ok_or("le profil devait être refusé")?;
    Ok(error.to_string())
}

#[test]
fn les_erreurs_de_schema_nomment_le_champ_de_l_arc() -> TestResult {
    let stage = |variant: &str| format!("[[stages]]\nvariants = [{variant}]\n");
    let cases: Vec<(String, &str)> = vec![
        ("stages = []".into(), "arc.stages"),
        (stage("{ intensity = 3.0 }"), "arc.stages[0].variants[0].intensity"),
        (stage("{ density = -1.0 }"), "arc.stages[0].variants[0].density"),
        (stage("{ weight = 0 }"), "arc.stages[0].variants[0].weight"),
        (
            stage("{ comping = [{ tag = \"inconnu\" }] }"),
            "arc.stages[0].variants[0].comping[0].tag",
        ),
        (
            stage("{ percussion = [{ tag = \"hat\", weight = 0 }] }"),
            "arc.stages[0].variants[0].percussion[0].weight",
        ),
        (
            stage("{ percussion = [{ tag = \"hat\" }, { tag = \"hat\" }] }"),
            "arc.stages[0].variants[0].percussion[1].tag",
        ),
        (stage("{ breaks = [{ start = 0 }] }"), "arc.stages[0].variants[0].breaks[0].start"),
        (
            stage("{ breaks = [{ start = 1, bars = 0 }] }"),
            "arc.stages[0].variants[0].breaks[0].bars",
        ),
        (
            stage("{ breaks = [{ start = 1, chance = 1.5 }] }"),
            "arc.stages[0].variants[0].breaks[0].chance",
        ),
        (
            stage("{ breaks = [{ start = 1, keep_steps = [16] }] }"),
            "arc.stages[0].variants[0].breaks[0].keep_steps",
        ),
        (
            stage("{ breaks = [{ start = 1, tracks = [\"Bass\", \"Bass\"] }] }"),
            "arc.stages[0].variants[0].breaks[0].tracks",
        ),
        (format!("repeat_from = 2\n{}", stage("{}")), "arc.repeat_from"),
        (format!("{}[[fall]]\nintensity = 9.0\n", stage("{}")), "arc.fall[0].intensity"),
        (format!("{}[[fall]]\nbass_feel = \"slow\"\n", stage("{}")), "arc.fall[0].bass_feel"),
        (format!("{}tempo = 1\n", stage("{}")), "arc"),
    ];
    for (arc, field) in cases {
        let message = invalid(&arc)?;
        assert!(message.contains(field), "{arc:?} : {message}");
    }
    Ok(())
}

#[test]
fn le_feel_de_basse_exige_une_marche() -> TestResult {
    // `reel_quebecois` n'a pas de marche de basse.
    let p = StyleProfile::default();
    let arc = "[[stages]]\nvariants = [{ bass_feel = \"two_feel\" }]\n";
    let error = with_arc(&p, arc).err().ok_or("refusé")?;
    assert!(error.to_string().contains("arc.stages[0].variants[0].bass_feel"), "{error}");
    Ok(())
}

#[test]
fn une_erreur_de_l_arc_est_situee_dans_son_fichier() -> TestResult {
    let dir = std::env::temp_dir().join(format!("harmonium_arc_{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    for entry in std::fs::read_dir(fixture_dir())? {
        let entry = entry?;
        std::fs::copy(entry.path(), dir.join(entry.file_name()))?;
    }
    let arc = std::fs::read_to_string(dir.join("arc.toml"))?;
    std::fs::write(dir.join("arc.toml"), arc.replace("intensity = 0.5", "intensity = 7.5"))?;
    let error = StyleProfile::load(&dir).err().ok_or("refusé");
    std::fs::remove_dir_all(&dir)?;
    let error = error?;
    assert_eq!(error.file.as_deref(), Some("arc.toml"), "{error}");
    assert_eq!(error.field, "arc.fall[0].intensity");
    assert!(error.line.is_some(), "{error}");
    Ok(())
}

#[test]
fn l_arc_survit_a_l_aller_retour_du_profil() -> TestResult {
    let p = fixture()?;
    assert_eq!(StyleProfile::from_toml(&p.to_toml()?)?, p);
    Ok(())
}

#[test]
fn une_forme_a_plusieurs_parties_est_un_chorus_de_trente_deux_mesures() -> TestResult {
    let p = StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/grille_arc_chorus_aaba",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let flow = stream(&p, 3, 96)?;
    for s in &flow {
        let position = s.position.ok_or("position publiée")?;
        assert_eq!(position.chorus(), (s.index - 1) / 32 + 1, "mesure {}", s.index);
        assert_eq!(position.choruses(), Some(3));
        // La passe de la forme `A A B A` se lit dans la même position.
        let part = [0, 0, 1, 0][((s.index - 1) % 32) / 8];
        assert_eq!(position.part.index(), part, "mesure {}", s.index);
    }
    let feel = |n: usize| -> Vec<usize> { bass_per_bar(&chorus(&flow, n)) };
    // Exposition (deux temps, deux accords par mesure : une arrivée chacun), corps (marche),
    // retombée (deux temps, stop-time sur les deux dernières mesures).
    assert!(feel(1).iter().all(|&n| n == 2), "{:?}", feel(1));
    assert!(feel(2).iter().all(|&n| n == 4 || n == 0), "{:?}", feel(2));
    let last = feel(3);
    assert!(last[..30].iter().all(|&n| n == 2), "{last:?}");
    assert_eq!(last[30..], [1, 1]);
    Ok(())
}

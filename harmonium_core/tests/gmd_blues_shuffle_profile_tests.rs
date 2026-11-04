//! B6 : contrat du profil distribué `gmd_blues_shuffle` (blues shuffle, feel ternaire sur douze pas), observé dans le flux de
//! `MeasureSnapshot` (principe VI) : grille de douze mesures (boucles relatives à la tonique), mètre ternaire et tempo de la
//! source, basse sur les changements d'accord, comping dans l'accord publié, batterie (grille de douze pas, densités,
//! relances), arc de cinq chorus, habillage par graine, variété entre graines.
//!
//! Les attendus sont écrits en clair. Basse, comping et grilles sont des règles de métier : aucun corpus ne les mesure, le
//! test ne vérifie donc que leur cohérence avec l'accord publié, pas une ressemblance à une source. Un chorus fait douze
//! mesures, cinq chorus par air.
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams,
    compose::TuneGenerator,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    style_profile::FixedTonality,
    timeline::ChordContext,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const ID: &str = "gmd_blues_shuffle";
const CHORDS: u8 = 1;
const BASS: u8 = 2;
const PERCUSSION_LOW: u8 = 3;
const PERCUSSION: [u8; 2] = [3, 4];
/// Mesures d'un chorus (douze) et d'un air (cinq chorus), pas par mesure.
const CHORUS: usize = 12;
const CHORUSES: usize = 5;
const TUNE: usize = CHORUSES * CHORUS;
const STEPS: usize = 12;
/// Plage de tempo de la source : les cinq fichiers ternaires du GMD vont de 72 à 134 BPM.
const BPM: (f32, f32) = (72.0, 134.0);
/// Rapport de swing médian des cinq fichiers ternaires du GMD.
const SWING: f32 = 0.664;
const TOMS: [u8; 2] = [43, 50];

/// Les cinq boucles écrites : fondamentales relatives à la tonique par mesure, et mesures dont la tierce est mineure.
const LOOPS: [([u8; CHORUS], &[usize]); 5] = [
    ([0, 0, 0, 0, 5, 5, 0, 0, 7, 5, 0, 7], &[]),
    ([0, 5, 0, 0, 5, 5, 0, 0, 7, 5, 0, 7], &[]),
    ([0, 5, 0, 0, 5, 5, 0, 9, 2, 7, 0, 7], &[8]),
    ([0, 0, 0, 0, 5, 5, 0, 0, 8, 7, 0, 7], &[0, 1, 2, 3, 4, 5, 6, 7, 10]),
    ([0, 0, 0, 0, 5, 5, 0, 0, 7, 5, 0, 0], &[]),
];

fn profile() -> TestResult<StyleProfile> {
    Ok(StyleProfile::named(ID)?)
}

/// Même profil sans arc ni session (qui pourrait choisir un arc nommé) : isole la base.
fn plain() -> TestResult<StyleProfile> {
    let mut p = profile()?;
    p.arc = None;
    p.session = Default::default();
    Ok(p)
}

fn stream_of(
    p: StyleProfile,
    seed: u64,
    bars: usize,
    intensity: f32,
    density: u8,
    bpm: Option<f32>,
) -> Vec<MeasureSnapshot> {
    let params = MusicalParams {
        bpm: bpm.unwrap_or(p.meter.default_bpm),
        bpm_explicit: bpm.is_some(),
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        intensity,
        percussion_density: density,
        ..Default::default()
    };
    let mut g = TuneGenerator::new(params, TuningParams { profile: p });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect()
}

fn track(s: &MeasureSnapshot, id: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == id).collect()
}

fn percussion(s: &MeasureSnapshot) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| PERCUSSION.contains(&n.track)).collect()
}

fn context_at(s: &MeasureSnapshot, step: usize) -> &ChordContext {
    &s.chord_changes[s.chord_changes.partition_point(|c| c.start_step <= step) - 1].context
}

/// Fondamentales relatives à la tonique, une par mesure (un accord par mesure dans toutes les grilles).
fn degrees(flow: &[MeasureSnapshot]) -> Vec<u8> {
    flow.iter()
        .map(|s| {
            let key = s.position.map_or(0, |p| p.key);
            (s.chord_changes[0].context.root_pc + 12 - key) % 12
        })
        .collect()
}

fn pcs(pitches: impl IntoIterator<Item = u8>) -> BTreeSet<u8> {
    pitches.into_iter().map(|p| p % 12).collect()
}

/// D4 : notes publiées et tensions naturelles sans conflit (convention de `bass_voicings_tests`).
fn allowed_pcs(chord: &ChordContext) -> BTreeSet<u8> {
    let owned: Vec<u8> =
        chord.pitch_classes.iter().map(|pc| (pc + 12 - chord.root_pc) % 12).collect();
    let mut allowed: BTreeSet<u8> = chord.pitch_classes.iter().copied().collect();
    for (tension, same_degree) in [(2u8, &[1u8, 2, 3][..]), (5, &[5, 6][..]), (9, &[8, 9][..])] {
        if !owned.iter().any(|o| same_degree.contains(o) || (tension + 12 - o) % 12 == 1) {
            allowed.insert((chord.root_pc + tension) % 12);
        }
    }
    allowed
}

/// Le groupe « registre » ne déplace que le comping, par octaves.
fn chord_register_offset(s: &MeasureSnapshot) -> i16 {
    match s.session.iter().find(|v| v.group == "registre").map(|v| v.name.as_str()) {
        Some("haut") => 12,
        Some("grave") => -12,
        _ => 0,
    }
}

/// Attaques de la piste des accords d'une mesure : (pas, durée, hauteurs).
fn attacks(s: &MeasureSnapshot, id: u8) -> Vec<(usize, usize, Vec<u8>)> {
    let mut out: Vec<(usize, usize, Vec<u8>)> = Vec::new();
    let mut notes = track(s, id);
    notes.sort_by_key(|n| (n.start_step, n.pitch));
    for n in notes {
        match out.last_mut() {
            Some((step, _, pitches)) if *step == n.start_step => pitches.push(n.pitch),
            _ => out.push((n.start_step, n.duration_steps, vec![n.pitch])),
        }
    }
    out
}

#[test]
fn la_grille_par_defaut_est_un_blues_de_douze_mesures_relatif_a_la_tonique() -> TestResult {
    let mut seen = BTreeSet::new();
    for seed in 1..=24 {
        let flow = stream_of(plain()?, seed, CHORUS, 0.5, 1, None);
        let d = degrees(&flow);
        let found = LOOPS.iter().position(|(l, _)| l[..] == d[..]);
        let Some(index) = found else {
            return Err(format!("graine {seed} : {d:?} hors des cinq boucles écrites").into());
        };
        seen.insert(index);
        // Un seul accord par mesure, au premier pas ; la tierce est celle de la boucle (septième de dominante ou mineure).
        for (bar, snap) in flow.iter().enumerate() {
            assert!(
                snap.chord_changes.len() == 1 && snap.chord_changes[0].start_step == 0,
                "graine {seed} mesure {}",
                bar + 1
            );
            let ctx = &snap.chord_changes[0].context;
            let minor = LOOPS[index].1.contains(&bar);
            let third = (ctx.root_pc + if minor { 3 } else { 4 }) % 12;
            assert!(
                ctx.pitch_classes.contains(&third),
                "graine {seed} mesure {} : {} sans la tierce {}",
                bar + 1,
                ctx.chord_name,
                if minor { "mineure" } else { "majeure" }
            );
            assert!(
                ctx.pitch_classes.contains(&((ctx.root_pc + 10) % 12)),
                "graine {seed} mesure {} : {} sans septième mineure",
                bar + 1,
                ctx.chord_name
            );
        }
        // Le chorus suivant rejoue la boucle (reprise de la partie).
        let again = stream_of(plain()?, seed, 2 * CHORUS, 0.5, 1, None);
        assert_eq!(degrees(&again)[CHORUS..], d[..], "graine {seed} : second chorus");
    }
    assert!(seen.len() >= 3, "{} boucles seulement sur 24 graines : {seen:?}", seen.len());
    Ok(())
}

#[test]
fn le_metre_est_ternaire_sur_douze_pas_et_le_tempo_celui_de_la_source() -> TestResult {
    let p = plain()?;
    assert_eq!(
        (p.meter.min_bpm, p.meter.max_bpm, p.meter.default_swing_ratio),
        (BPM.0, BPM.1, SWING)
    );
    for intensity in [0.0f32, 0.5, 1.0] {
        for snap in &stream_of(p.clone(), 1, 8, intensity, 1, None) {
            assert!(
                (BPM.0..=BPM.1).contains(&snap.tempo),
                "intensité {intensity} : tempo {} hors de {}..{}",
                snap.tempo,
                BPM.0,
                BPM.1
            );
            assert!(
                (snap.swing_ratio - SWING).abs() < 1e-6,
                "rapport de swing {}",
                snap.swing_ratio
            );
            assert_eq!(snap.steps, STEPS, "grille de douze pas");
        }
    }
    // Aucune attaque ne sort de la mesure, et les accords changent sur un temps (multiples de trois pas).
    for seed in [1u64, 2, 3] {
        for snap in &stream_of(profile()?, seed, TUNE, 0.5, 2, None) {
            assert!(snap.notes.iter().all(|n| n.start_step < STEPS));
            assert!(snap.chord_changes.iter().all(|c| c.start_step % 3 == 0));
        }
    }
    Ok(())
}

#[test]
fn la_basse_joue_le_shuffle_de_l_accord_publie_et_arrive_sur_sa_basse() -> TestResult {
    let mut arrivals = 0;
    let mut shuffle_pairs = 0;
    for seed in [1u64, 2, 17] {
        let flow = stream_of(profile()?, seed, TUNE + 1, 0.5, 1, None);
        for (bar, snap) in flow.iter().enumerate().take(TUNE) {
            let next = &flow[bar + 1].chord_changes[0].context;
            for n in track(snap, BASS) {
                let ctx = format!("graine {seed} mesure {} pas {}", bar + 1, n.start_step);
                let chord = context_at(snap, n.start_step);
                // Fondamentale, quinte, sixte, septième mineure du riff ; ou approche chromatique ou diatonique de
                // la basse de l'accord suivant (passage au temps 4, trois pas avant la barre).
                let own = (n.pitch % 12 + 12 - chord.bass_pc) % 12;
                let to_next = (n.pitch % 12 + 12 - next.bass_pc) % 12;
                let riff = [0, 7, 9, 10].contains(&own);
                let approach = n.start_step + 3 >= STEPS && [1, 2, 10, 11].contains(&to_next);
                assert!(
                    riff || approach,
                    "{ctx} : hauteur {} à l'intervalle {own} de la basse {} de {}",
                    n.pitch,
                    chord.bass_pc,
                    chord.chord_name
                );
                assert!(
                    (28..=53).contains(&n.pitch),
                    "{ctx} : hauteur {} hors de la fenêtre de marche 28..53",
                    n.pitch
                );
                if n.start_step == 0 {
                    assert_eq!(
                        n.pitch % 12,
                        chord.bass_pc,
                        "{ctx} : arrivée du premier temps sur la basse publiée"
                    );
                    assert!(
                        (30..=41).contains(&n.pitch),
                        "{ctx} : arrivée {} hors de 30..41",
                        n.pitch
                    );
                    arrivals += 1;
                }
            }
            // Shuffle : une attaque longue (deux pas) suivie d'une brève (un pas) sur un temps, quelque part dans la mesure.
            let attacks = attacks(snap, BASS);
            shuffle_pairs += usize::from(attacks.iter().any(|(s, d, _)| {
                s % 3 == 0 && *d == 2 && attacks.iter().any(|(t, e, _)| *t == s + 2 && *e == 1)
            }));
        }
    }
    // Une arrivée par mesure, sauf aux mesures de break (la basse se tait).
    assert!(arrivals * 10 >= 9 * 3 * TUNE, "{arrivals} arrivées seulement sur {}", 3 * TUNE);
    assert!(
        shuffle_pairs * 2 >= 3 * TUNE,
        "paires longue-brève dans {shuffle_pairs} mesures sur {}",
        3 * TUNE
    );
    Ok(())
}

#[test]
fn le_comping_respecte_la_qualite_publiee_et_ses_tensions_naturelles() -> TestResult {
    let p = profile()?;
    let mut total = 0;
    for seed in [1u64, 2, 17] {
        let flow = stream_of(p.clone(), seed, TUNE + 1, 0.5, 1, None);
        for (bar, snap) in flow.iter().enumerate().take(TUNE) {
            for (step, duration, pitches) in attacks(snap, CHORDS) {
                let ctx = format!("graine {seed} mesure {} pas {step}", bar + 1);
                // Une attaque anticipée finit sur la barre et joue l'accord de la mesure suivante.
                let current = context_at(snap, step);
                let played = pcs(pitches.iter().copied());
                let chord = if !played.is_subset(&allowed_pcs(current)) && step + duration == STEPS
                {
                    &flow[bar + 1].chord_changes[0].context
                } else {
                    current
                };
                let allowed = allowed_pcs(chord);
                assert!(
                    played.is_subset(&allowed),
                    "{ctx} : {played:?} hors de {} (sons permis {allowed:?})",
                    chord.chord_name
                );
                // Tessiture du comping (familles serrée ou ouverte, registre de la session : ± une octave).
                let offset = chord_register_offset(snap);
                assert!(
                    pitches.iter().all(|p| (40 + offset..=90 + offset).contains(&i16::from(*p))),
                    "{ctx} : tessiture {pitches:?} hors de {}..{}",
                    40 + offset,
                    90 + offset
                );
                total += 1;
            }
        }
    }
    assert!(total > 400, "{total} attaques de comping seulement");
    Ok(())
}

#[test]
fn la_batterie_monte_du_timekeeper_a_l_ossature_puis_aux_notes_fantomes() -> TestResult {
    let timekeepers = [42u8, 46, 51, 53];
    let mut counts = [0usize; 3];
    for density in [0u8, 1, 2] {
        for seed in [1u64, 2, 3, 17] {
            let flow = stream_of(plain()?, seed, TUNE, 0.5, density, None);
            let mut backbeats = 0;
            for (bar, snap) in flow.iter().enumerate() {
                let notes = percussion(snap);
                let ctx = format!("graine {seed} densité {density} mesure {}", bar + 1);
                assert!(notes.iter().all(|n| n.start_step < STEPS), "{ctx}");
                // La grosse caisse est seule sur la piste grave, tout le reste sur la piste aiguë.
                assert!(
                    notes.iter().all(|n| (n.pitch == 36) == (n.track == PERCUSSION_LOW)),
                    "{ctx} : pistes"
                );
                if density == 0 {
                    assert!(
                        notes.iter().all(|n| timekeepers.contains(&n.pitch)),
                        "{ctx} : densité 0 = timekeeper seul, reçu {:?}",
                        notes.iter().map(|n| n.pitch).collect::<BTreeSet<_>>()
                    );
                }
                backbeats += usize::from(notes.iter().any(|n| matches!(n.pitch, 37 | 38)));
                counts[usize::from(density)] += notes.len();
            }
            // Dès l'ossature, la caisse claire ou le cross-stick joue dans la plupart des mesures.
            if density >= 1 {
                assert!(
                    backbeats * 10 >= TUNE * 6,
                    "graine {seed} densité {density} : caisse claire dans {backbeats} mesures sur {TUNE}"
                );
            }
        }
    }
    assert!(counts[0] < counts[1] && counts[1] < counts[2], "coups par densité {counts:?}");
    Ok(())
}

#[test]
fn les_relances_de_fin_de_phrase_jouent_les_toms_du_corpus_ou_aucune() -> TestResult {
    let mut with_toms = 0;
    let mut at_density_zero = 0;
    for seed in 1..=8u64 {
        for (density, tally) in [(2u8, &mut with_toms), (0, &mut at_density_zero)] {
            let flow = stream_of(plain()?, seed, TUNE, 0.5, density, None);
            *tally += flow
                .iter()
                .filter(|s| percussion(s).iter().any(|n| TOMS.contains(&n.pitch)))
                .count();
        }
    }
    assert_eq!(at_density_zero, 0, "relance à la densité 0");
    assert!(with_toms >= 8, "{with_toms} mesures de relance aux toms sur 8 graines seulement");
    Ok(())
}

#[test]
fn l_arc_joue_exposition_corps_sommet_et_retombee_sur_les_cinq_chorus() -> TestResult {
    for seed in [1u64, 2, 3, 17] {
        // L'arc par défaut reste couvert indépendamment du choix d'un arc nommé de session.
        let mut p = profile()?;
        p.session = Default::default();
        let flow = stream_of(p, seed, TUNE, 0.5, 1, None);
        let ctx = |what: &str| format!("graine {seed} : {what}");
        let chorus = |c: usize| &flow[c * CHORUS..(c + 1) * CHORUS];
        let kicks = |bars: &[MeasureSnapshot]| {
            bars.iter().flat_map(percussion).filter(|n| n.pitch == 36).count()
        };
        let last = CHORUSES - 1;
        // Retombée : charleston seule (aucune grosse caisse), comping clairsemé, stop-time sur les deux dernières mesures.
        assert_eq!(kicks(chorus(last)), 0, "{}", ctx("grosse caisse à la retombée"));
        for snap in chorus(last) {
            assert!(attacks(snap, CHORDS).len() <= 3, "{}", ctx("comping clairsemé à la retombée"));
        }
        for snap in &chorus(last)[CHORUS - 2..] {
            assert!(
                snap.notes
                    .iter()
                    .filter(|n| n.track == CHORDS || n.track == BASS)
                    .all(|n| n.start_step == 0),
                "{}",
                ctx("stop-time")
            );
        }
        // Exposition : ossature de batterie (grosse caisse présente).
        assert!(kicks(chorus(0)) > 0, "{}", ctx("exposition sans grosse caisse"));
        // Le corps et le sommet jouent plus fort que l'exposition et la retombée : vélocité moyenne de la basse
        // (l'intensité de l'arc règle `bass_velocity` ; les notes fantômes de la batterie fausseraient la mesure).
        let level = |bars: &[MeasureSnapshot]| {
            let v: Vec<usize> =
                bars.iter().flat_map(|s| track(s, BASS)).map(|n| usize::from(n.velocity)).collect();
            v.iter().sum::<usize>() / v.len().max(1)
        };
        let levels: Vec<usize> = (0..CHORUSES).map(|c| level(chorus(c))).collect();
        for body in 1..=3 {
            assert!(
                levels[body] > levels[0] && levels[body] > levels[last],
                "{} : vélocités de basse par chorus {levels:?}",
                ctx("corps et sommet")
            );
        }
        // Le ride du corps (chorus 2 à 4) : absent de l'exposition et de la retombée, où la charleston reste le timekeeper.
        let rides = |bars: &[MeasureSnapshot]| {
            bars.iter().flat_map(percussion).filter(|n| n.pitch == 51).count()
        };
        assert!((1..=3).all(|c| rides(chorus(c)) > 0), "{}", ctx("ride du corps"));
        assert_eq!(rides(chorus(0)), 0, "{}", ctx("ride dans l'exposition"));
        assert_eq!(rides(chorus(last)), 0, "{}", ctx("ride à la retombée"));
    }
    Ok(())
}

#[test]
fn le_profil_rejoue_la_meme_graine_a_l_identique() -> TestResult {
    let a = stream_of(profile()?, 5, TUNE, 0.5, 1, None);
    let b = stream_of(profile()?, 5, TUNE, 0.5, 1, None);
    assert_eq!(serde_json::to_string(&a)?, serde_json::to_string(&b)?);
    Ok(())
}

#[test]
fn seize_graines_donnent_des_accompagnements_distincts() -> TestResult {
    let mut comping = BTreeSet::new();
    let mut bass = BTreeSet::new();
    let mut drums = BTreeSet::new();
    for seed in 1..=16u64 {
        let flow = stream_of(profile()?, seed, TUNE, 0.5, 1, None);
        comping.insert(
            flow.iter()
                .map(|s| attacks(s, CHORDS).iter().map(|a| a.0).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
        );
        bass.insert(
            flow.iter()
                .map(|s| track(s, BASS).iter().map(|n| (n.start_step, n.pitch)).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
        );
        drums.insert(
            flow.iter()
                .map(|s| percussion(s).iter().map(|n| (n.start_step, n.pitch)).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
        );
    }
    assert!(comping.len() >= 14, "{} suites de comping distinctes sur 16 graines", comping.len());
    assert!(bass.len() >= 12, "{} suites de basse distinctes sur 16 graines", bass.len());
    assert!(drums.len() >= 12, "{} suites de batterie distinctes sur 16 graines", drums.len());
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Dressing {
    tempo_cluster: usize,
    key: u8,
    mode: String,
    chords: Option<u8>,
    bass: Option<u8>,
    kit: Option<u8>,
    voicing: String,
}

/// Habillage d'une graine, lu dans sa première mesure ; les tempos se rangent ensuite en grappes de 16 BPM (± 8).
fn dressings(p: &StyleProfile) -> TestResult<Vec<(f32, Dressing)>> {
    let mut rows: Vec<(f32, Dressing)> = (1..=16u64)
        .map(|seed| -> TestResult<(f32, Dressing)> {
            let first = stream_of(p.clone(), seed, 1, 0.5, 1, None).remove(0);
            let position = first.position.ok_or("position publiée")?;
            let instruments = first.instruments.ok_or("programmes publiés sous une session")?;
            let voicing = first
                .session
                .iter()
                .find(|v| v.group == "voicing")
                .map(|v| v.name.clone())
                .unwrap_or_default();
            Ok((
                first.tempo,
                Dressing {
                    tempo_cluster: 0,
                    key: position.key,
                    mode: format!("{:?}", position.mode),
                    chords: instruments.chords_program,
                    bass: instruments.bass_program,
                    kit: Some(instruments.percussion_program),
                    voicing,
                },
            ))
        })
        .collect::<TestResult<_>>()?;
    let mut order: Vec<f32> = rows.iter().map(|(tempo, _)| *tempo).collect();
    order.sort_by(f32::total_cmp);
    let mut starts: Vec<f32> = Vec::new();
    for tempo in order {
        if starts.last().is_none_or(|start| tempo - start > 16.0) {
            starts.push(tempo);
        }
    }
    for (tempo, d) in &mut rows {
        d.tempo_cluster = starts.iter().rposition(|start| *tempo >= *start).unwrap_or(0);
    }
    Ok(rows)
}

#[test]
fn seize_graines_donnent_au_moins_huit_habillages_dans_les_bornes_du_style() -> TestResult {
    let p = profile()?;
    let rows = dressings(&p)?;
    let distinct: BTreeSet<_> = rows.iter().map(|(_, d)| d.clone()).collect();
    let clusters: BTreeSet<_> = rows.iter().map(|(_, d)| d.tempo_cluster).collect();
    let keys: BTreeSet<_> = rows.iter().map(|(_, d)| (d.key, d.mode.clone())).collect();
    let chords: BTreeSet<_> = rows.iter().map(|(_, d)| d.chords).collect();
    let bass: BTreeSet<_> = rows.iter().map(|(_, d)| d.bass).collect();
    let kits: BTreeSet<_> = rows.iter().map(|(_, d)| d.kit).collect();
    let voicings: BTreeSet<_> = rows.iter().map(|(_, d)| d.voicing.clone()).collect();
    assert!(distinct.len() >= 8, "{} habillages distincts sur 16 graines", distinct.len());
    assert!(clusters.len() >= 2, "tempos à ±8 BPM {rows:?}");
    assert!(keys.len() >= 3, "tonalités {keys:?}");
    assert!(
        chords.len() >= 3 && bass.len() >= 2 && kits.len() >= 3,
        "{chords:?} {bass:?} {kits:?}"
    );
    assert!(voicings.len() >= 2, "familles de voicings {voicings:?}");
    // Tonalités du blues : mixolydien sur une des six toniques écrites (mi, la, sol, do, si♭, ré).
    assert!(
        keys.iter().all(|(k, mode)| [4, 9, 7, 0, 10, 2].contains(k) && mode == "Mixolydian"),
        "tonalités {keys:?}"
    );
    for (tempo, _) in &rows {
        assert!((BPM.0..=BPM.1).contains(tempo), "{tempo} BPM hors de la plage de la source");
    }
    Ok(())
}

#[test]
fn l_hote_garde_la_tonalite_et_le_tempo_et_l_habillage_reste_visible() -> TestResult {
    let mut p = profile()?;
    let mode = p.melody.key_modes[0].mode;
    p.fixed_tonality = Some(FixedTonality { key: 5, mode });
    let mut kits = BTreeSet::new();
    for seed in 1..=16u64 {
        let snaps = stream_of(p.clone(), seed, 8, 0.5, 1, Some(100.0));
        let first = &snaps[0];
        assert_eq!(first.position.map(|p| p.key), Some(5), "graine {seed} : tonalité de l'hôte");
        assert!(snaps.iter().all(|s| s.tempo == 100.0), "graine {seed} : tempo de l'hôte");
        assert!(!first.session.is_empty(), "variante de session publiée");
        kits.insert(first.instruments.map(|i| i.percussion_program));
    }
    assert!(kits.len() >= 2, "le kit reste tiré par la graine sous la grille de l'hôte");
    Ok(())
}

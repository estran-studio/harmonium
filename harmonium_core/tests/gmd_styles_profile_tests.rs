//! B8 : contrat des sept profils distribués `gmd_*` (rock, pop, hip-hop, soul, reggae, disco, afrobeat), observé dans
//! le flux de `MeasureSnapshot` (principe VI) : grille par défaut (boucles écrites relatives à la tonique), feel et
//! tempo de la source, basse sur les changements d'accord, comping compatible avec la qualité publiée (D4),
//! batterie (voix, densités, relances), arc de base sur quatre chorus, variété entre graines.
//!
//! Les attendus sont écrits en clair (tableau `STYLES`). Basse, comping et grilles sont des règles de métier : aucun
//! corpus ne les mesure, le test ne vérifie donc que leur cohérence avec l'accord publié, pas une ressemblance à une
//! source. Un chorus fait 16 mesures (parties A et B de 8 mesures), quatre chorus par air.
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams,
    compose::TuneGenerator,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::ChordContext,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CHORDS: u8 = 1;
const BASS: u8 = 2;
const PERCUSSION_LOW: u8 = 3;
const PERCUSSION: [u8; 2] = [3, 4];
/// Mesures d'un chorus (A + B) et d'un air (quatre chorus).
const CHORUS: usize = 16;
const TUNE: usize = 4 * CHORUS;
const TOMS: [u8; 5] = [41, 43, 45, 47, 50];

struct Spec {
    id: &'static str,
    /// Plage de tempo de la source : centile 10, médiane, centile 90 des fichiers `beat` retenus.
    bpm: (f32, f32, f32),
    /// Boucles de la partie A et de la partie B : degrés chromatiques des fondamentales, une par mesure.
    a: &'static [[u8; 8]],
    b: &'static [[u8; 8]],
    /// Intervalles (modulo 12) que la basse joue au-dessus de la basse publiée, et fenêtre des fondamentales.
    bass_intervals: &'static [u8],
    bass_window: (u8, u8),
    /// Le style a des fichiers `fill` dans le GMD (donc des relances aux toms).
    fills: bool,
    /// Ride (51) des chorus du corps de l'arc, quand le profil a des groupes de ride (sinon la charleston reste le timekeeper).
    loud_timekeeper: Option<u8>,
    /// Variété minimale entre 16 graines (suites distinctes de 64 mesures) : comping, basse, batterie.
    distinct: (usize, usize, usize),
}

const STYLES: [Spec; 7] = [
    Spec {
        id: "gmd_rock",
        bpm: (90.0, 105.0, 130.0),
        a: &[
            [0, 0, 5, 5, 0, 0, 7, 5],
            [0, 7, 9, 5, 0, 7, 9, 5],
            [0, 10, 5, 0, 0, 10, 5, 7],
            [9, 5, 0, 7, 9, 5, 0, 7],
        ],
        b: &[
            [5, 7, 0, 0, 5, 7, 9, 7],
            [5, 0, 7, 9, 5, 0, 7, 7],
            [0, 5, 7, 5, 0, 5, 7, 5],
            [8, 10, 0, 0, 8, 10, 0, 0],
        ],
        bass_intervals: &[0, 7],
        bass_window: (28, 39),
        fills: true,
        loud_timekeeper: Some(51),
        distinct: (14, 10, 10),
    },
    Spec {
        id: "gmd_pop",
        bpm: (83.0, 135.0, 142.0),
        a: &[
            [0, 7, 9, 5, 0, 7, 9, 5],
            [9, 5, 0, 7, 9, 5, 0, 7],
            [0, 9, 5, 7, 0, 9, 5, 7],
            [0, 5, 9, 7, 0, 5, 9, 7],
        ],
        b: &[
            [5, 0, 7, 9, 5, 0, 7, 9],
            [5, 7, 4, 9, 5, 7, 0, 0],
            [9, 7, 5, 0, 9, 7, 5, 0],
            [0, 7, 5, 5, 0, 7, 9, 7],
        ],
        bass_intervals: &[0, 7],
        bass_window: (28, 39),
        fills: true,
        loud_timekeeper: Some(51),
        distinct: (14, 10, 8),
    },
    Spec {
        id: "gmd_hiphop",
        bpm: (70.0, 91.0, 100.0),
        a: &[
            [0, 8, 3, 10, 0, 8, 3, 10],
            [0, 0, 5, 5, 0, 0, 5, 5],
            [0, 10, 8, 7, 0, 10, 8, 7],
            [0, 3, 10, 5, 0, 3, 10, 7],
        ],
        b: &[
            [8, 10, 0, 0, 8, 10, 0, 0],
            [5, 0, 8, 7, 5, 0, 8, 7],
            [3, 10, 0, 0, 3, 10, 5, 7],
            [0, 0, 0, 0, 8, 8, 10, 10],
        ],
        bass_intervals: &[0, 7],
        bass_window: (24, 35),
        fills: true,
        loud_timekeeper: None,
        distinct: (14, 10, 8),
    },
    Spec {
        id: "gmd_soul",
        bpm: (80.0, 102.0, 105.0),
        a: &[
            [0, 9, 2, 7, 0, 9, 2, 7],
            [0, 0, 5, 5, 0, 9, 2, 7],
            [0, 5, 0, 0, 5, 5, 0, 7],
            [2, 7, 0, 9, 2, 7, 0, 0],
        ],
        b: &[[5, 4, 2, 7, 5, 4, 2, 7], [5, 7, 4, 9, 5, 7, 0, 0], [9, 2, 7, 0, 9, 2, 7, 0]],
        bass_intervals: &[0, 7],
        bass_window: (28, 39),
        fills: true,
        loud_timekeeper: Some(51),
        distinct: (14, 10, 8),
    },
    Spec {
        id: "gmd_reggae",
        bpm: (83.0, 102.0, 121.0),
        a: &[
            [0, 0, 5, 5, 0, 0, 5, 5],
            [0, 5, 0, 7, 0, 5, 0, 7],
            [0, 9, 5, 7, 0, 9, 5, 7],
            [9, 9, 5, 5, 9, 9, 4, 4],
        ],
        b: &[[5, 5, 0, 0, 5, 5, 7, 7], [0, 7, 9, 5, 0, 7, 9, 5], [5, 0, 5, 0, 5, 7, 0, 0]],
        bass_intervals: &[0, 7],
        bass_window: (28, 39),
        fills: true,
        loud_timekeeper: None,
        distinct: (12, 8, 4),
    },
    Spec {
        id: "gmd_dance_disco",
        bpm: (120.0, 120.0, 137.0),
        a: &[
            [0, 0, 5, 5, 0, 0, 5, 5],
            [0, 5, 10, 3, 0, 5, 10, 3],
            [0, 10, 8, 7, 0, 10, 8, 7],
            [0, 0, 0, 0, 5, 5, 7, 7],
        ],
        b: &[[8, 10, 0, 0, 8, 10, 0, 7], [3, 8, 10, 0, 3, 8, 10, 0], [5, 5, 0, 0, 5, 5, 7, 7]],
        bass_intervals: &[0, 7, 10],
        bass_window: (28, 39),
        fills: false,
        loud_timekeeper: None,
        distinct: (14, 10, 6),
    },
    Spec {
        id: "gmd_afrobeat",
        bpm: (86.0, 98.0, 113.0),
        a: &[
            [0, 0, 0, 0, 5, 5, 5, 5],
            [0, 0, 5, 5, 0, 0, 5, 5],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 10, 0, 10, 0, 10, 5, 7],
        ],
        b: &[[5, 5, 0, 0, 5, 5, 0, 0], [10, 10, 5, 5, 0, 0, 0, 0], [0, 5, 0, 5, 0, 5, 0, 5]],
        bass_intervals: &[0, 7, 10],
        bass_window: (28, 39),
        fills: false,
        loud_timekeeper: Some(51),
        distinct: (14, 10, 8),
    },
];

fn profile(spec: &Spec) -> TestResult<StyleProfile> {
    Ok(StyleProfile::named(spec.id)?)
}

/// Même profil sans arc ni session (qui pourrait choisir un arc nommé) : isole la base.
fn plain(spec: &Spec) -> TestResult<StyleProfile> {
    let mut p = profile(spec)?;
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
) -> Vec<MeasureSnapshot> {
    let params = MusicalParams {
        bpm: p.meter.default_bpm,
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

/// Le groupe « registre » de ces profils ne déplace que le comping, par octaves.
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
fn la_grille_par_defaut_est_une_boucle_ecrite_de_chaque_partie_relative_a_la_tonique() -> TestResult
{
    for spec in &STYLES {
        let mut seen_a = BTreeSet::new();
        let mut seen_b = BTreeSet::new();
        for seed in 1..=16 {
            let flow = stream_of(plain(spec)?, seed, CHORUS, 0.5, 1);
            let d = degrees(&flow);
            let a = spec.a.iter().position(|l| l[..] == d[..8]);
            let b = spec.b.iter().position(|l| l[..] == d[8..]);
            assert!(
                a.is_some(),
                "{} graine {seed} : partie A {:?} hors des boucles écrites",
                spec.id,
                &d[..8]
            );
            assert!(
                b.is_some(),
                "{} graine {seed} : partie B {:?} hors des boucles écrites",
                spec.id,
                &d[8..]
            );
            seen_a.extend(a);
            seen_b.extend(b);
            // Un seul accord par mesure, au premier pas ; le chorus suivant rejoue les mêmes boucles (reprise de la partie).
            assert!(
                flow.iter()
                    .all(|s| s.chord_changes.len() == 1 && s.chord_changes[0].start_step == 0),
                "{}",
                spec.id
            );
            let again = stream_of(plain(spec)?, seed, TUNE, 0.5, 1);
            assert_eq!(
                degrees(&again)[CHORUS..2 * CHORUS],
                d[..],
                "{} graine {seed} : second chorus",
                spec.id
            );
        }
        assert!(
            seen_a.len() >= 2 && seen_b.len() >= 2,
            "{} : une seule boucle sur 16 graines",
            spec.id
        );
    }
    Ok(())
}

#[test]
fn le_tempo_et_le_feel_sont_ceux_de_la_source() -> TestResult {
    for spec in &STYLES {
        for intensity in [0.0f32, 0.5, 1.0] {
            for snap in &stream_of(plain(spec)?, 1, 8, intensity, 1) {
                assert!(
                    (spec.bpm.0..=spec.bpm.2).contains(&snap.tempo),
                    "{} intensité {intensity} : tempo {} hors de {}..{}",
                    spec.id,
                    snap.tempo,
                    spec.bpm.0,
                    spec.bpm.2
                );
                assert!((snap.swing_ratio - 0.5).abs() < 1e-6, "{} : base droite", spec.id);
                assert_eq!(snap.steps, 16, "{} : grille 4/4", spec.id);
            }
        }
    }
    Ok(())
}

#[test]
fn la_basse_joue_l_accord_publie_dans_sa_fenetre_et_arrive_sur_sa_basse() -> TestResult {
    for spec in &STYLES {
        let (low, high) = spec.bass_window;
        let mut arrivals = 0;
        for seed in [1u64, 2, 17] {
            let flow = stream_of(profile(spec)?, seed, TUNE + 1, 0.5, 1);
            for (bar, snap) in flow.iter().enumerate().take(TUNE) {
                for n in track(snap, BASS) {
                    let ctx = format!(
                        "{} graine {seed} mesure {} pas {}",
                        spec.id,
                        bar + 1,
                        n.start_step
                    );
                    let chord = context_at(snap, n.start_step);
                    let end = n.start_step + n.duration_steps;
                    // Relance vers l'accord suivant (pickup) : note courte qui finit sur la barre et joue la basse suivante.
                    let next = &flow[bar + 1].chord_changes[0].context;
                    let target = if end == 16
                        && n.pitch % 12 == next.bass_pc
                        && n.pitch % 12 != chord.bass_pc
                    {
                        next
                    } else {
                        chord
                    };
                    let interval = (n.pitch % 12 + 12 - target.bass_pc) % 12;
                    assert!(
                        spec.bass_intervals.contains(&interval) || interval == 0,
                        "{ctx} : hauteur {} à l'intervalle {interval} de la basse {} de {}",
                        n.pitch,
                        target.bass_pc,
                        target.chord_name
                    );
                    assert!(
                        (low..=high + 12).contains(&n.pitch),
                        "{ctx} : hauteur {} hors de {low}..{}",
                        n.pitch,
                        high + 12
                    );
                    if n.start_step == 0 {
                        assert_eq!(
                            n.pitch % 12,
                            chord.bass_pc,
                            "{ctx} : arrivée du premier temps sur la basse publiée"
                        );
                        assert!(
                            (low..=high).contains(&n.pitch),
                            "{ctx} : arrivée {} hors de {low}..{high}",
                            n.pitch
                        );
                        arrivals += 1;
                    }
                }
            }
        }
        assert!(arrivals > TUNE, "{} : {arrivals} arrivées seulement", spec.id);
    }
    Ok(())
}

#[test]
fn le_comping_respecte_la_qualite_publiee_et_ses_tensions_naturelles() -> TestResult {
    for spec in &STYLES {
        let p = profile(spec)?;
        let mut total = 0;
        for seed in [1u64, 2, 17] {
            let flow = stream_of(p.clone(), seed, TUNE + 1, 0.5, 1);
            for (bar, snap) in flow.iter().enumerate().take(TUNE) {
                for (step, duration, pitches) in attacks(snap, CHORDS) {
                    let ctx = format!("{} graine {seed} mesure {} pas {step}", spec.id, bar + 1);
                    // Une attaque anticipée finit sur la barre et joue l'accord de la mesure suivante.
                    let current = context_at(snap, step);
                    let played = pcs(pitches.iter().copied());
                    let chord = if !played.is_subset(&allowed_pcs(current)) && step + duration == 16
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
                    // Tessiture du comping (famille serrée ou ouverte, registre de la session : ± une octave).
                    let offset = chord_register_offset(snap);
                    assert!(
                        pitches
                            .iter()
                            .all(|p| (40 + offset..=90 + offset).contains(&i16::from(*p))),
                        "{ctx} : tessiture {pitches:?} hors de {}..{}",
                        40 + offset,
                        90 + offset
                    );
                    total += 1;
                }
            }
        }
        assert!(total > 150, "{} : {total} attaques de comping seulement", spec.id);
    }
    Ok(())
}

#[test]
fn la_batterie_monte_de_la_charleston_a_l_ossature_puis_aux_notes_fantomes() -> TestResult {
    for spec in &STYLES {
        let timekeepers = [42u8, 46, 51, 53];
        let mut counts = [0usize; 3];
        for density in [0u8, 1, 2] {
            for seed in [1u64, 2, 3, 17] {
                let flow = stream_of(plain(spec)?, seed, TUNE, 0.5, density);
                let mut backbeats = 0;
                for (bar, snap) in flow.iter().enumerate() {
                    let notes = percussion(snap);
                    let ctx =
                        format!("{} graine {seed} densité {density} mesure {}", spec.id, bar + 1);
                    assert!(notes.iter().all(|n| n.start_step < 16), "{ctx}");
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
                        "{} graine {seed} densité {density} : caisse claire dans {backbeats} mesures sur {TUNE}",
                        spec.id
                    );
                }
            }
        }
        assert!(
            counts[0] < counts[1] && counts[1] < counts[2],
            "{} : coups par densité {counts:?}",
            spec.id
        );
    }
    Ok(())
}

#[test]
fn les_relances_de_fin_de_phrase_jouent_les_toms_du_corpus_ou_aucune() -> TestResult {
    for spec in &STYLES {
        let mut with_toms = 0;
        let mut at_density_zero = 0;
        for seed in 1..=8u64 {
            for (density, tally) in [(2u8, &mut with_toms), (0, &mut at_density_zero)] {
                let flow = stream_of(plain(spec)?, seed, TUNE, 0.5, density);
                *tally += flow
                    .iter()
                    .filter(|s| percussion(s).iter().any(|n| TOMS.contains(&n.pitch)))
                    .count();
            }
        }
        assert_eq!(at_density_zero, 0, "{} : relance à la densité 0", spec.id);
        if spec.fills {
            assert!(
                with_toms >= 8,
                "{} : {with_toms} mesures de relance aux toms sur 8 graines seulement",
                spec.id
            );
        } else {
            assert_eq!(
                with_toms, 0,
                "{} : aucun fichier `fill` dans le GMD, donc aucune relance",
                spec.id
            );
        }
    }
    Ok(())
}

#[test]
fn l_arc_joue_exposition_corps_et_retombee_sur_les_quatre_chorus() -> TestResult {
    for spec in &STYLES {
        for seed in [1u64, 2, 3, 17] {
            // L'arc par défaut reste couvert indépendamment du choix d'un arc nommé de session.
            let mut p = profile(spec)?;
            p.session = Default::default();
            let flow = stream_of(p, seed, TUNE, 0.5, 1);
            let ctx = |what: &str| format!("{} graine {seed} : {what}", spec.id);
            let chorus = |c: usize| &flow[c * CHORUS..(c + 1) * CHORUS];
            let kicks = |bars: &[MeasureSnapshot]| {
                bars.iter().flat_map(percussion).filter(|n| n.pitch == 36).count()
            };
            // Retombée : charleston seule (aucune grosse caisse), comping clairsemé, stop-time sur les deux dernières mesures.
            assert_eq!(kicks(chorus(3)), 0, "{}", ctx("grosse caisse à la retombée"));
            for snap in chorus(3) {
                assert!(
                    attacks(snap, CHORDS).len() <= 2,
                    "{}",
                    ctx("comping clairsemé à la retombée")
                );
            }
            for snap in &chorus(3)[CHORUS - 2..] {
                assert!(
                    snap.notes
                        .iter()
                        .filter(|n| n.track == CHORDS || n.track == BASS)
                        .all(|n| n.start_step == 0),
                    "{}",
                    ctx("stop-time")
                );
            }
            // Exposition : ossature de batterie (grosse caisse présente), comping de cinq attaques au plus.
            assert!(kicks(chorus(0)) > 0, "{}", ctx("exposition sans grosse caisse"));
            for snap in chorus(0) {
                assert!(attacks(snap, CHORDS).len() <= 5, "{}", ctx("comping de l'exposition"));
            }
            // Le corps joue plus fort que l'exposition et que la retombée : vélocité moyenne de la basse
            // (l'intensité de l'arc règle `bass_velocity` ; les notes fantômes de la batterie fausseraient la mesure).
            let level = |bars: &[MeasureSnapshot]| {
                let v: Vec<usize> = bars
                    .iter()
                    .flat_map(|s| track(s, BASS))
                    .map(|n| usize::from(n.velocity))
                    .collect();
                v.iter().sum::<usize>() / v.len().max(1)
            };
            let levels: Vec<usize> = (0..4).map(|c| level(chorus(c))).collect();
            assert!(
                levels[1] > levels[0] && levels[2] > levels[0],
                "{} : vélocités de basse par chorus {levels:?}",
                ctx("corps")
            );
            assert!(
                levels[1] > levels[3] && levels[2] > levels[3],
                "{} : vélocités de basse par chorus {levels:?}",
                ctx("retombée")
            );
            if let Some(pitch) = spec.loud_timekeeper {
                let count = |bars: &[MeasureSnapshot]| {
                    bars.iter().flat_map(percussion).filter(|n| n.pitch == pitch).count()
                };
                assert!(
                    count(chorus(1)) > 0 && count(chorus(2)) > 0,
                    "{}",
                    ctx("timekeeper du corps")
                );
                assert_eq!(count(chorus(0)), 0, "{}", ctx("timekeeper du corps dans l'exposition"));
                assert_eq!(count(chorus(3)), 0, "{}", ctx("timekeeper du corps à la retombée"));
            }
        }
    }
    Ok(())
}

#[test]
fn le_profil_rejoue_la_meme_graine_a_l_identique() -> TestResult {
    for spec in &STYLES {
        let a = stream_of(profile(spec)?, 5, TUNE, 0.5, 1);
        let b = stream_of(profile(spec)?, 5, TUNE, 0.5, 1);
        assert_eq!(serde_json::to_string(&a)?, serde_json::to_string(&b)?, "{}", spec.id);
    }
    Ok(())
}

#[test]
fn seize_graines_donnent_des_accompagnements_distincts() -> TestResult {
    for spec in &STYLES {
        let mut comping = BTreeSet::new();
        let mut bass = BTreeSet::new();
        let mut drums = BTreeSet::new();
        let mut shapes = BTreeSet::new();
        for seed in 1..=16u64 {
            let flow = stream_of(profile(spec)?, seed, TUNE, 0.5, 1);
            comping.insert(
                flow.iter()
                    .map(|s| attacks(s, CHORDS).iter().map(|a| a.0).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
            );
            bass.insert(
                flow.iter()
                    .map(|s| {
                        track(s, BASS).iter().map(|n| (n.start_step, n.pitch)).collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            );
            drums.insert(
                flow.iter()
                    .map(|s| {
                        percussion(s).iter().map(|n| (n.start_step, n.pitch)).collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            );
            // Forme : boucles de A et de B, intensité de chaque chorus (vélocité moyenne de la batterie).
            let d = degrees(&flow[..CHORUS]);
            let level: Vec<usize> = (0..4)
                .map(|c| {
                    let v: Vec<usize> = flow[c * CHORUS..(c + 1) * CHORUS]
                        .iter()
                        .flat_map(percussion)
                        .map(|n| usize::from(n.velocity))
                        .collect();
                    v.iter().sum::<usize>() / v.len().max(1) / 4
                })
                .collect();
            shapes.insert((d, level));
        }
        let (c, b, d) = spec.distinct;
        assert!(
            comping.len() >= c,
            "{} : {} suites de comping distinctes sur 16 graines (attendu {c})",
            spec.id,
            comping.len()
        );
        assert!(
            bass.len() >= b,
            "{} : {} suites de basse distinctes sur 16 graines (attendu {b})",
            spec.id,
            bass.len()
        );
        assert!(
            drums.len() >= d,
            "{} : {} suites de batterie distinctes sur 16 graines (attendu {d})",
            spec.id,
            drums.len()
        );
        assert!(
            shapes.len() >= 8,
            "{} : {} formes distinctes sur 16 graines",
            spec.id,
            shapes.len()
        );
    }
    Ok(())
}

//! E7 : contrat du profil distribué `wjazzd_swing`, observé dans le flux de `MeasureSnapshot`
//! (principe VI). Les attendus sont écrits en clair, relatifs à la tonique (fa, `key` 5) : la
//! grille par défaut, la marche de basse (quatre noires, arrivée sur les changements, fenêtre),
//! le comping dans l'accord publié, la batterie (ride à chaque temps, pédale aux temps 2 et 4,
//! relance de fin de phrase) et le tempo dans la plage du profil.
//!
//! La séquence du profil est `Blues A A B A` : 12 + 8 + 8 + 8 + 8 = 44 mesures, deux tours, donc
//! 88 mesures.
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

const SEEDS: [u64; 3] = [1, 2, 17];
const CHORDS: u8 = 1;
const BASS: u8 = 2;
const PERCUSSION: [u8; 2] = [3, 4];
const KEY: u8 = 5;
/// Un tour : 12 + 8 + 8 + 8 + 8 mesures.
const TOUR: usize = 44;

fn profile() -> TestResult<StyleProfile> {
    Ok(StyleProfile::named("wjazzd_swing")?)
}

fn stream(seed: u64, bars: usize, intensity: f32, density: u8) -> TestResult<Vec<MeasureSnapshot>> {
    let p = profile()?;
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
    Ok((1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect())
}

fn neutral(seed: u64, bars: usize) -> TestResult<Vec<MeasureSnapshot>> {
    stream(seed, bars, 0.5, 1)
}

fn track(s: &MeasureSnapshot, id: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == id).collect()
}

fn percussion(s: &MeasureSnapshot) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| PERCUSSION.contains(&n.track)).collect()
}

/// Fondamentale d'un accord publié, relative à la tonique.
fn degree(c: &ChordContext) -> u8 {
    (c.root_pc + 12 - KEY) % 12
}

fn context_at(s: &MeasureSnapshot, step: usize) -> &ChordContext {
    &s.chord_changes[s.chord_changes.partition_point(|c| c.start_step <= step) - 1].context
}

/// Fondamentales relatives (une par changement d'accord de la mesure, pas, degré).
fn changes(s: &MeasureSnapshot) -> Vec<(usize, u8)> {
    s.chord_changes.iter().map(|c| (c.start_step, degree(&c.context))).collect()
}

fn pcs(pitches: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = pitches.iter().map(|p| p % 12).collect();
    out.sort_unstable();
    out.dedup();
    out
}

#[test]
fn la_grille_par_defaut_est_le_blues_puis_la_forme_aaba_ecrits_en_fa() -> TestResult {
    // (pas, degré) de chaque changement, par mesure, relatifs à la tonique.
    let blues: [&[(usize, u8)]; 12] = [
        &[(0, 0)],
        &[(0, 5)],
        &[(0, 0)],
        &[(0, 0)],
        &[(0, 5)],
        &[(0, 5)],
        &[(0, 0)],
        &[(0, 0)],
        &[(0, 2)],
        &[(0, 7)],
        &[(0, 0)],
        &[(0, 0), (8, 7)],
    ];
    let a: [&[(usize, u8)]; 8] = [
        &[(0, 0), (8, 9)],
        &[(0, 2), (8, 7)],
        &[(0, 4), (8, 9)],
        &[(0, 2), (8, 7)],
        &[(0, 7), (8, 0)],
        &[(0, 5), (8, 5)],
        &[(0, 0), (8, 9)],
        &[(0, 2), (8, 7)],
    ];
    let b: [&[(usize, u8)]; 8] =
        [&[(0, 4)], &[(0, 4)], &[(0, 9)], &[(0, 9)], &[(0, 2)], &[(0, 2)], &[(0, 7)], &[(0, 7)]];
    // Séquence du profil `[Blues, A, A, B, A]`, deux tours.
    let mut expected: Vec<&[(usize, u8)]> = Vec::new();
    for _ in 0..2 {
        expected.extend(blues);
        expected.extend(a);
        expected.extend(a);
        expected.extend(b);
        expected.extend(a);
    }
    for seed in SEEDS {
        let flow = neutral(seed, 2 * TOUR)?;
        for (index, snap) in flow.iter().enumerate() {
            assert_eq!(
                changes(snap),
                expected[index],
                "graine {seed} mesure {} ({})",
                index + 1,
                snap.chord_name
            );
        }
    }
    Ok(())
}

#[test]
fn la_marche_joue_quatre_noires_par_mesure_et_arrive_sur_la_basse_de_chaque_changement()
-> TestResult {
    for seed in SEEDS {
        let flow = neutral(seed, 2 * TOUR)?;
        for (index, snap) in flow.iter().enumerate() {
            let ctx = format!("graine {seed} mesure {}", index + 1);
            let mut bass = track(snap, BASS);
            bass.sort_by_key(|n| n.start_step);
            let steps: Vec<usize> = bass.iter().map(|n| n.start_step).collect();
            assert_eq!(steps, [0, 4, 8, 12], "{ctx} : quatre noires");
            assert!(bass.iter().all(|n| n.duration_steps == 4), "{ctx} : noires pleines");
            assert!(
                bass.iter().all(|n| (33..=50).contains(&n.pitch)),
                "{ctx} : hauteurs {:?} hors de la fenêtre 33..50 des statistiques de basse",
                bass.iter().map(|n| n.pitch).collect::<Vec<_>>()
            );
            // Arrivée : premier temps et chaque pulsation où l'accord change → basse publiée.
            for note in &bass {
                let arrival = note.start_step == 0
                    || snap.chord_changes.iter().any(|c| c.start_step == note.start_step);
                if arrival {
                    let chord = context_at(snap, note.start_step);
                    assert_eq!(
                        note.pitch % 12,
                        chord.bass_pc,
                        "{ctx} pas {} : arrivée sur {} attendue sur {}",
                        note.start_step,
                        note.pitch % 12,
                        chord.bass_pc
                    );
                    assert!(
                        (36..=47).contains(&note.pitch),
                        "{ctx} pas {} : arrivée {} hors de 36..47",
                        note.start_step,
                        note.pitch
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn l_accord_slash_de_la_forme_aaba_est_joue_sur_sa_basse() -> TestResult {
    // Mesure 19 (A, septième mesure) : I6 sur la quinte (F6/C), puis D7.
    let flow = neutral(1, 30)?;
    let bar = &flow[12 + 6];
    assert_eq!(bar.chord_changes[0].context.bass_pc, (KEY + 7) % 12, "basse publiée do");
    let first = track(bar, BASS).into_iter().find(|n| n.start_step == 0).map(|n| n.pitch % 12);
    assert_eq!(first, Some((KEY + 7) % 12));
    Ok(())
}

#[test]
fn le_comping_ne_joue_que_des_notes_de_l_accord_publie_sans_sa_fondamentale() -> TestResult {
    for seed in SEEDS {
        let flow = neutral(seed, TOUR + 1)?;
        let mut attacks = 0;
        for (bar, snap) in flow.iter().enumerate().take(TOUR) {
            let mut at: Vec<(usize, usize, Vec<u8>)> = Vec::new();
            for n in track(snap, CHORDS) {
                match at.last_mut() {
                    Some((step, _, pitches)) if *step == n.start_step => pitches.push(n.pitch),
                    _ => at.push((n.start_step, n.duration_steps, vec![n.pitch])),
                }
            }
            for (step, duration, mut pitches) in at {
                pitches.sort_unstable();
                let ctx = format!("graine {seed} mesure {} pas {step}", bar + 1);
                // Les attaques anticipées du profil (pas 14, durée 2) jouent l'accord de la mesure
                // suivante ; la blanche du pas 8 finit aussi sur la barre mais n'est pas anticipée.
                let chord = if step == 14 && duration == 2 {
                    &flow[bar + 1].chord_changes[0].context
                } else {
                    context_at(snap, step)
                };
                let played = pcs(&pitches);
                assert!(
                    played.iter().all(|pc| chord.pitch_classes.contains(pc)),
                    "{ctx} : {played:?} hors de {} {:?}",
                    chord.chord_name,
                    chord.pitch_classes
                );
                assert!(!played.contains(&chord.root_pc), "{ctx} : fondamentale jouée");
                assert!(pitches.len() >= 3, "{ctx} : {pitches:?}");
                assert!(
                    pitches[0] >= 48 && pitches[pitches.len() - 1] <= 80,
                    "{ctx} : tessiture {pitches:?}"
                );
                attacks += 1;
            }
        }
        assert!(attacks > 60, "graine {seed} : {attacks} attaques de comping seulement");
    }
    Ok(())
}

#[test]
fn le_ride_marque_chaque_temps_et_la_pedale_les_temps_2_et_4() -> TestResult {
    const RIDE: u8 = 51;
    const PEDAL: u8 = 44;
    for seed in SEEDS {
        for density in [0u8, 1, 2] {
            let flow = stream(seed, 2 * TOUR, 0.5, density)?;
            for (index, snap) in flow.iter().enumerate() {
                // Mesure de fin de phrase (12ᵉ du blues, 8ᵉ de A et de B) : relance, sans ride.
                let phrase_end = matches!(index % TOUR, 11 | 19 | 27 | 35 | 43);
                let ctx = format!("graine {seed} densité {density} mesure {}", index + 1);
                let notes = percussion(snap);
                let ride: Vec<usize> =
                    notes.iter().filter(|n| n.pitch == RIDE).map(|n| n.start_step).collect();
                if density == 2 && phrase_end {
                    assert!(ride.is_empty(), "{ctx} : relance de fin de phrase avec ride");
                    assert!(
                        notes.iter().any(|n| n.pitch == 38) && notes.iter().any(|n| n.pitch == 36),
                        "{ctx} : la relance joue caisse claire et grosse caisse"
                    );
                    continue;
                }
                for beat in [0, 4, 8, 12] {
                    assert!(ride.contains(&beat), "{ctx} : pas de ride au pas {beat} ({ride:?})");
                }
                for n in notes.iter().filter(|n| n.pitch == PEDAL) {
                    assert!(
                        density >= 1 && (n.start_step == 4 || n.start_step == 12),
                        "{ctx} : pédale au pas {}",
                        n.start_step
                    );
                }
                if density >= 1 {
                    assert!(
                        notes.iter().filter(|n| n.pitch == PEDAL).count() <= 2,
                        "{ctx} : trop de coups de pédale"
                    );
                }
                if density < 2 {
                    assert!(
                        notes.iter().all(|n| n.pitch != 36),
                        "{ctx} : grosse caisse hors densité 2"
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn le_tempo_reste_dans_la_plage_et_le_swing_est_celui_du_profil() -> TestResult {
    let p = profile()?;
    assert!((p.meter.min_bpm..=p.meter.max_bpm).contains(&p.meter.default_bpm));
    for intensity in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
        let flow = stream(1, 14, intensity, 1)?;
        for snap in &flow {
            assert!(
                (p.meter.min_bpm..=p.meter.max_bpm).contains(&snap.tempo),
                "intensité {intensity} : tempo {} hors de {}..{}",
                snap.tempo,
                p.meter.min_bpm,
                p.meter.max_bpm
            );
            assert!(
                (snap.swing_ratio - p.meter.default_swing_ratio).abs() < 1e-6,
                "swing {} au lieu de {}",
                snap.swing_ratio,
                p.meter.default_swing_ratio
            );
        }
    }
    // Les rapports de swing du corpus (JTD) vont de 0,58 à 0,72 : le défaut y est.
    assert!((0.58..=0.72).contains(&p.meter.default_swing_ratio));
    Ok(())
}

#[test]
fn le_profil_rejoue_la_meme_graine_a_l_identique_et_varie_d_une_graine_a_l_autre() -> TestResult {
    let a = neutral(1, TOUR)?;
    let again = neutral(1, TOUR)?;
    assert_eq!(serde_json::to_string(&a)?, serde_json::to_string(&again)?);
    let bass_line = |flow: &[MeasureSnapshot]| -> Vec<u8> {
        flow.iter().flat_map(|s| track(s, BASS).into_iter().map(|n| n.pitch)).collect()
    };
    let other = neutral(2, TOUR)?;
    assert_ne!(bass_line(&a), bass_line(&other), "la marche ne dépend pas de la graine");
    Ok(())
}

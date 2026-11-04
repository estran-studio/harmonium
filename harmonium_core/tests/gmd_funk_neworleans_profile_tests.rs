//! B4 et B7 (phase 019) : contrat des profils distribués `gmd_funk` et
//! `gmd_neworleans_funk_secondline`, observé dans le flux de `MeasureSnapshot` (principe VI).
//!
//! Chaque contrat s'écrit une fois et se joue sur les deux profils (`Spec`). Les attendus sont écrits
//! en clair, relatifs à la tonique : les grilles par défaut (une par boucle écrite dans le profil),
//! la basse sur les changements d'accord, le comping dans l'accord publié, le kit et le backbeat
//! de la batterie, le feel droit en 16 pas, le tempo dans la plage, l'arc sur cinq chorus.
//!
//! `gmd_funk` : un chorus de seize mesures, cinq chorus par air (80 mesures).
//! `gmd_neworleans_funk_secondline` : douze mesures, cinq chorus (60 mesures). La tonique est tirée par la session.
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
const PERCUSSION: [u8; 2] = [3, 4];
/// Notes du kit GM que les patterns, phrases et relances des deux profils peuvent jouer.
const KIT: [u8; 12] = [36, 37, 38, 42, 43, 44, 45, 46, 48, 49, 51, 53];
const SNARE: u8 = 38;
const SIDESTICK: u8 = 37;
const KICK: u8 = 36;
const CHORUSES: usize = 5;
const SEEDS: [u64; 4] = [1, 2, 3, 17];

/// Une boucle de grille : (demi-tons au-dessus de la tonique, durée en pas), `steps_per_bar` = 16.
type Loop = &'static [(u8, usize)];

struct Spec {
    id: &'static str,
    /// Mesures d'un chorus.
    bars: usize,
    loops: &'static [Loop],
    bpm: (f32, f32, f32),
    /// Fenêtre de la fondamentale de basse (`low..=high` du profil) et plus haute note jouée.
    root_window: (u8, u8),
    bass_ceiling: u8,
    /// Le comping joue la fondamentale de l'accord (`drop2`) ou non (`guide_tones`).
    comping_plays_root: bool,
    comping_window: (u8, u8),
}

const FUNK: Spec = Spec {
    id: "gmd_funk",
    bars: 16,
    loops: &[
        &[(0, 256)],
        &[(0, 64), (5, 64), (0, 64), (5, 32), (0, 32)],
        &[(0, 64), (5, 64), (0, 64), (5, 32), (10, 32)],
        &[(0, 32), (10, 32), (0, 32), (10, 32), (0, 32), (10, 32), (0, 32), (10, 32)],
        &[
            (0, 32),
            (5, 32),
            (0, 32),
            (10, 16),
            (7, 8),
            (10, 8),
            (0, 32),
            (5, 32),
            (0, 32),
            (10, 16),
            (7, 8),
            (10, 8),
        ],
    ],
    bpm: (80.0, 96.0, 130.0),
    root_window: (30, 41),
    bass_ceiling: 53,
    comping_plays_root: false,
    comping_window: (52, 80),
};

const NEW_ORLEANS: Spec = Spec {
    id: "gmd_neworleans_funk_secondline",
    bars: 12,
    loops: &[
        &[(0, 16), (5, 16), (0, 32), (5, 32), (0, 32), (7, 16), (5, 16), (0, 24), (7, 8)],
        &[(0, 16), (5, 16), (0, 32), (5, 32), (0, 32), (7, 16), (5, 16), (0, 32)],
        &[(0, 128), (5, 64)],
        &[(0, 192)],
        &[(0, 32), (7, 32), (0, 32), (5, 32), (0, 32), (7, 16), (0, 16)],
    ],
    bpm: (82.0, 93.0, 124.0),
    root_window: (28, 39),
    bass_ceiling: 51,
    comping_plays_root: true,
    comping_window: (46, 80),
};

fn profile(spec: &Spec) -> TestResult<StyleProfile> {
    Ok(StyleProfile::named(spec.id)?)
}

fn stream(
    spec: &Spec,
    seed: u64,
    bars: usize,
    intensity: f32,
    density: u8,
) -> TestResult<Vec<MeasureSnapshot>> {
    let p = profile(spec)?;
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

/// Les cinq chorus de l'air, intensité et densité neutres.
fn air(spec: &Spec, seed: u64) -> TestResult<Vec<MeasureSnapshot>> {
    stream(spec, seed, CHORUSES * spec.bars, 0.5, 1)
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

/// Mesures du chorus `n` (1-based).
fn chorus<'a>(flow: &'a [MeasureSnapshot], spec: &Spec, n: usize) -> Vec<&'a MeasureSnapshot> {
    flow[(n - 1) * spec.bars..n * spec.bars].iter().collect()
}

/// Degré (demi-tons au-dessus de la tonique tirée par la session) d'un accord publié.
fn degree(key: u8, c: &ChordContext) -> u8 {
    (c.root_pc + 12 - key) % 12
}

/// D4 : notes publiées et tensions naturelles que la couleur du comping peut ajouter, sans conflit avec la grille.
fn allowed_pcs(chord: &ChordContext) -> Vec<u8> {
    let owned: Vec<u8> =
        chord.pitch_classes.iter().map(|pc| (pc + 12 - chord.root_pc) % 12).collect();
    let mut allowed: Vec<u8> = chord.pitch_classes.clone();
    for (tension, same_degree) in [(2u8, &[1u8, 2, 3][..]), (5, &[5, 6][..]), (9, &[8, 9][..])] {
        if !owned.iter().any(|o| same_degree.contains(o) || (tension + 12 - o) % 12 == 1) {
            allowed.push((chord.root_pc + tension) % 12);
        }
    }
    allowed
}

/// Décalage d'octaves du comping tiré par le groupe « registre » de la session.
fn register_offset(s: &MeasureSnapshot) -> i16 {
    match s.session.iter().find(|v| v.group == "registre").map(|v| v.name.as_str()) {
        Some("haut") => 12,
        Some("grave") => -12,
        _ => 0,
    }
}

fn context_at(s: &MeasureSnapshot, step: usize) -> &ChordContext {
    &s.chord_changes[s.chord_changes.partition_point(|c| c.start_step <= step) - 1].context
}

/// (pas, degré) de chaque changement publié d'une mesure.
fn changes(s: &MeasureSnapshot) -> Vec<(usize, u8)> {
    let key = s.position.map_or(0, |p| p.key);
    s.chord_changes.iter().map(|c| (c.start_step, degree(key, &c.context))).collect()
}

/// Les changements par mesure d'une boucle écrite : un pas 0 par mesure, puis chaque changement de
/// fondamentale tombant dans la mesure.
fn expected_changes(chords: Loop) -> Vec<Vec<(usize, u8)>> {
    let mut flat: Vec<(u8, usize)> = Vec::new();
    for &(degree, steps) in chords {
        match flat.last_mut() {
            Some((last, total)) if *last == degree => *total += steps,
            _ => flat.push((degree, steps)),
        }
    }
    let total: usize = flat.iter().map(|c| c.1).sum();
    let mut bars = vec![Vec::new(); total / 16];
    let mut start = 0;
    for &(degree, steps) in &flat {
        let (bar, step) = (start / 16, start % 16);
        bars[bar].push((step, degree));
        // Une mesure traversée par un accord long publie quand même son pas 0.
        let crossed = (start + steps).div_ceil(16);
        for later in bars.iter_mut().take(crossed).skip(bar + 1) {
            later.push((0, degree));
        }
        start += steps;
    }
    bars
}

fn pcs(pitches: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = pitches.iter().map(|p| p % 12).collect();
    out.sort_unstable();
    out.dedup();
    out
}

fn sequence(flow: &[MeasureSnapshot], f: impl Fn(&MeasureSnapshot) -> String) -> String {
    flow.iter().map(f).collect::<Vec<_>>().join("|")
}

#[test]
fn la_grille_par_defaut_est_une_des_boucles_ecrites_et_la_graine_en_choisit_plusieurs() -> TestResult
{
    for spec in [&FUNK, &NEW_ORLEANS] {
        let expected: Vec<Vec<Vec<(usize, u8)>>> =
            spec.loops.iter().map(|l| expected_changes(l)).collect();
        for e in &expected {
            assert_eq!(
                e.len(),
                spec.bars,
                "{} : une boucle écrite fait {} mesures",
                spec.id,
                spec.bars
            );
        }
        let mut seen = BTreeSet::new();
        for seed in 1..=16u64 {
            let flow = air(spec, seed)?;
            let first: Vec<Vec<(usize, u8)>> = flow[..spec.bars].iter().map(changes).collect();
            let which = expected.iter().position(|e| *e == first).ok_or_else(|| {
                format!("{} graine {seed} : grille hors des boucles écrites : {first:?}", spec.id)
            })?;
            seen.insert(which);
            // Chaque chorus joue une des boucles écrites (un tirage par chorus, `tour_variants`,
            // A7) ; le chorus 5 rejoue le tirage du chorus 1 (cycle de quatre).
            let mut chorus_choices = Vec::new();
            for n in 1..=CHORUSES {
                let again: Vec<Vec<(usize, u8)>> =
                    chorus(&flow, spec, n).iter().map(|s| changes(s)).collect();
                let which = expected.iter().position(|e| *e == again).ok_or_else(|| {
                    format!("{} graine {seed} chorus {n} : hors des boucles écrites", spec.id)
                })?;
                chorus_choices.push(which);
            }
            if CHORUSES >= 5 {
                assert_eq!(chorus_choices[0], chorus_choices[4], "{} graine {seed}", spec.id);
            }
        }
        assert!(seen.len() >= 3, "{} : 16 graines n'ont tiré que {} grilles", spec.id, seen.len());
    }
    Ok(())
}

#[test]
fn la_basse_arrive_sur_la_basse_publiee_a_chaque_changement_dans_la_fenetre_du_profil() -> TestResult
{
    for spec in [&FUNK, &NEW_ORLEANS] {
        for seed in SEEDS {
            let flow = air(spec, seed)?;
            for s in &flow {
                let ctx = format!("{} graine {seed} mesure {}", spec.id, s.index);
                let bass = track(s, BASS);
                // Les deux dernières mesures d'un chorus peuvent porter un break ou un stop-time de
                // l'arc (la basse se tait ou ne garde que le temps 1) : les arrivées se lisent ailleurs.
                let in_chorus = (s.index - 1) % spec.bars;
                let broken = in_chorus >= spec.bars - 2 && s.index > spec.bars;
                for note in &bass {
                    assert!(
                        (spec.root_window.0.saturating_sub(2)..=spec.bass_ceiling)
                            .contains(&note.pitch),
                        "{ctx} : hauteur {} hors de la fenêtre de basse",
                        note.pitch
                    );
                }
                for change in s.chord_changes.iter().filter(|_| !broken) {
                    let arrival =
                        bass.iter().find(|n| n.start_step == change.start_step).ok_or_else(
                            || format!("{ctx} pas {} : pas d'attaque de basse", change.start_step),
                        )?;
                    assert_eq!(
                        arrival.pitch % 12,
                        change.context.bass_pc,
                        "{ctx} pas {} : arrivée sur {} au lieu de la basse publiée",
                        change.start_step,
                        arrival.pitch % 12
                    );
                    assert!(
                        (spec.root_window.0..=spec.root_window.1).contains(&arrival.pitch),
                        "{ctx} pas {} : arrivée {} hors de {}..={}",
                        change.start_step,
                        arrival.pitch,
                        spec.root_window.0,
                        spec.root_window.1
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn la_basse_ne_joue_que_la_fondamentale_la_quinte_la_septieme_ou_une_approche() -> TestResult {
    for spec in [&FUNK, &NEW_ORLEANS] {
        let mut approaches = 0;
        for seed in SEEDS {
            let flow = air(spec, seed)?;
            for (index, s) in flow.iter().enumerate() {
                let mut bass = track(s, BASS);
                bass.sort_by_key(|n| n.start_step);
                for (rank, note) in bass.iter().enumerate() {
                    let chord = context_at(s, note.start_step);
                    let rel = (note.pitch + 12 - chord.bass_pc) % 12;
                    if [0, 7, 10].contains(&rel) {
                        continue;
                    }
                    // Sinon c'est une approche : à un ou deux demi-tons de la note de basse suivante
                    // (de la mesure suivante pour la dernière).
                    let next = bass.get(rank + 1).map(|n| n.pitch).or_else(|| {
                        flow.get(index + 1).and_then(|n| {
                            track(n, BASS)
                                .iter()
                                .map(|b| (b.start_step, b.pitch))
                                .min()
                                .map(|b| b.1)
                        })
                    });
                    let next = next.ok_or("cible d'approche")?;
                    let distance = note.pitch.abs_diff(next);
                    assert!(
                        (1..=2).contains(&distance),
                        "{} graine {seed} mesure {} pas {} : {} n'est ni riff ni approche de {next}",
                        spec.id,
                        s.index,
                        note.start_step,
                        note.pitch
                    );
                    approaches += 1;
                }
            }
        }
        assert!(approaches > 0, "{} : aucune approche sur {} graines", spec.id, SEEDS.len());
    }
    Ok(())
}

#[test]
fn le_comping_ne_joue_que_des_notes_de_l_accord_publie() -> TestResult {
    for spec in [&FUNK, &NEW_ORLEANS] {
        for seed in SEEDS {
            let flow = air(spec, seed)?;
            let mut attack_count = 0;
            for (index, snap) in flow.iter().enumerate() {
                let mut at: Vec<(usize, usize, Vec<u8>)> = Vec::new();
                for n in track(snap, CHORDS) {
                    match at.last_mut() {
                        Some((step, _, pitches)) if *step == n.start_step => pitches.push(n.pitch),
                        _ => at.push((n.start_step, n.duration_steps, vec![n.pitch])),
                    }
                }
                for (step, duration, mut pitches) in at {
                    pitches.sort_unstable();
                    let ctx = format!("{} graine {seed} mesure {} pas {step}", spec.id, snap.index);
                    let here = context_at(snap, step);
                    // Une attaque qui finit sur la barre peut être anticipée : elle joue alors
                    // l'accord de la mesure suivante.
                    let ahead = (step + duration == 16).then(|| flow.get(index + 1)).flatten();
                    let played = pcs(&pitches);
                    let fits = |c: &ChordContext| {
                        let allowed = allowed_pcs(c);
                        played.iter().all(|pc| allowed.contains(pc))
                    };
                    let sounding = if fits(here) {
                        here
                    } else {
                        ahead.map(|a| &a.chord_changes[0].context).filter(|c| fits(c)).ok_or_else(
                            || {
                                format!(
                                    "{ctx} : {played:?} hors de {} {:?}",
                                    here.chord_name,
                                    allowed_pcs(here)
                                )
                            },
                        )?
                    };
                    // Seule la famille `drop2` de la session joue la fondamentale (notes guides et shell : jamais).
                    let drop2 =
                        snap.session.iter().any(|v| v.group == "voicing" && v.name == "drop_2");
                    if !spec.comping_plays_root && !drop2 {
                        assert!(!played.contains(&sounding.root_pc), "{ctx} : fondamentale jouée");
                    }
                    let shift = register_offset(snap);
                    assert!(
                        i16::from(pitches[0]) >= i16::from(spec.comping_window.0) + shift
                            && i16::from(pitches[pitches.len() - 1])
                                <= i16::from(spec.comping_window.1) + shift,
                        "{ctx} : tessiture {pitches:?} (registre {shift:+})"
                    );
                    attack_count += 1;
                }
            }
            assert!(
                attack_count > 100,
                "{} graine {seed} : {attack_count} attaques de comping",
                spec.id
            );
        }
    }
    Ok(())
}

#[test]
fn la_batterie_reste_dans_le_kit_et_pose_un_backbeat_sur_les_temps_2_et_4() -> TestResult {
    for spec in [&FUNK, &NEW_ORLEANS] {
        for seed in SEEDS {
            let flow = air(spec, seed)?;
            let (mut backbeat, mut kick) = (0, 0);
            for s in &flow {
                let notes = percussion(s);
                assert!(
                    notes.iter().all(|n| KIT.contains(&n.pitch)),
                    "{} graine {seed} mesure {} : note hors du kit : {:?}",
                    spec.id,
                    s.index,
                    notes.iter().map(|n| n.pitch).collect::<Vec<_>>()
                );
                kick += usize::from(notes.iter().any(|n| n.pitch == KICK));
                if notes.iter().any(|n| {
                    (n.pitch == SNARE || n.pitch == SIDESTICK)
                        && (n.start_step == 4 || n.start_step == 12)
                }) {
                    backbeat += 1;
                }
            }
            // Le groupe de mesures d'un pattern retenu a une grosse caisse, et pour le funk une caisse
            // claire au temps 2 ou 4 ; seules les relances (fins de phrase) les remplacent.
            assert!(
                kick * 100 >= flow.len() * 70,
                "{} graine {seed} : grosse caisse sur {kick} mesures",
                spec.id
            );
            if spec.comping_plays_root {
                continue;
            }
            assert!(
                backbeat * 100 >= flow.len() * 70,
                "{} graine {seed} : backbeat sur {backbeat} mesures de {}",
                spec.id,
                flow.len()
            );
        }
    }
    Ok(())
}

#[test]
fn le_feel_est_droit_en_seize_pas_et_le_tempo_reste_dans_la_plage() -> TestResult {
    for spec in [&FUNK, &NEW_ORLEANS] {
        let p = profile(spec)?;
        assert_eq!(
            (p.meter.min_bpm, p.meter.default_bpm, p.meter.max_bpm),
            spec.bpm,
            "{}",
            spec.id
        );
        assert!((p.meter.default_swing_ratio - 0.5).abs() < 1e-6, "{} : feel droit", spec.id);
        for intensity in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
            let flow = stream(spec, 1, spec.bars + 2, intensity, 1)?;
            for s in &flow {
                assert_eq!(s.steps, 16, "{} : seize pas par mesure", spec.id);
                assert!(
                    (spec.bpm.0..=spec.bpm.2).contains(&s.tempo),
                    "{} intensité {intensity} : tempo {} hors de {}..{}",
                    spec.id,
                    s.tempo,
                    spec.bpm.0,
                    spec.bpm.2
                );
                assert!(
                    (s.swing_ratio - 0.5).abs() < 1e-6,
                    "{} : swing {}",
                    spec.id,
                    s.swing_ratio
                );
            }
        }
        // Doubles croches : à la densité 2 une mesure au moins attaque un pas impair (le 16ᵉ).
        let flow = stream(spec, 2, 4 * spec.bars, 1.0, 2)?;
        assert!(
            flow.iter().any(|s| percussion(s).iter().any(|n| n.start_step % 2 == 1)),
            "{} : aucune double croche à la batterie",
            spec.id
        );
    }
    Ok(())
}

#[test]
fn l_arc_creuse_l_ecart_entre_l_exposition_le_sommet_et_la_retombee() -> TestResult {
    for spec in [&FUNK, &NEW_ORLEANS] {
        for seed in SEEDS {
            let flow = air(spec, seed)?;
            let notes = |n: usize| -> usize {
                chorus(&flow, spec, n)
                    .iter()
                    .map(|s| track(s, CHORDS).len() + percussion(s).len())
                    .sum()
            };
            let (first, peak, fall) = (notes(1), notes(4), notes(5));
            assert!(
                first < peak,
                "{} graine {seed} : exposition {first} contre sommet {peak}",
                spec.id
            );
            assert!(
                fall < peak,
                "{} graine {seed} : retombée {fall} contre sommet {peak}",
                spec.id
            );
            // Retombée : stop-time sur les deux dernières mesures, accords et basse sur le un seulement.
            for s in chorus(&flow, spec, CHORUSES).iter().rev().take(2) {
                for id in [CHORDS, BASS] {
                    assert!(
                        attacks(s, id).iter().all(|&step| step == 0),
                        "{} graine {seed} mesure {} piste {id} : {:?}",
                        spec.id,
                        s.index,
                        attacks(s, id)
                    );
                }
                assert!(
                    !percussion(s).is_empty(),
                    "{} mesure {} : la batterie continue",
                    spec.id,
                    s.index
                );
            }
        }
    }
    Ok(())
}

#[test]
fn la_graine_change_le_comping_la_batterie_et_la_basse_et_la_meme_graine_rejoue_pareil()
-> TestResult {
    for spec in [&FUNK, &NEW_ORLEANS] {
        let a = air(spec, 1)?;
        let again = air(spec, 1)?;
        assert_eq!(serde_json::to_string(&a)?, serde_json::to_string(&again)?, "{}", spec.id);
        let mut comping = BTreeSet::new();
        let mut drums = BTreeSet::new();
        let mut bass = BTreeSet::new();
        for seed in 1..=16u64 {
            let flow = air(spec, seed)?;
            comping.insert(sequence(&flow, |s| {
                track(s, CHORDS)
                    .iter()
                    .map(|n| format!("{}", n.start_step))
                    .collect::<Vec<_>>()
                    .join(",")
            }));
            drums.insert(sequence(&flow, |s| {
                percussion(s)
                    .iter()
                    .map(|n| format!("{}.{}", n.start_step, n.pitch))
                    .collect::<Vec<_>>()
                    .join(",")
            }));
            bass.insert(sequence(&flow, |s| {
                track(s, BASS)
                    .iter()
                    .map(|n| format!("{}.{}", n.start_step, n.pitch))
                    .collect::<Vec<_>>()
                    .join(",")
            }));
        }
        assert!(
            comping.len() >= 14,
            "{} : {} comping distincts sur 16 graines",
            spec.id,
            comping.len()
        );
        assert!(
            drums.len() >= 14,
            "{} : {} batteries distinctes sur 16 graines",
            spec.id,
            drums.len()
        );
        assert!(bass.len() >= 12, "{} : {} basses distinctes sur 16 graines", spec.id, bass.len());
    }
    Ok(())
}

//! B3 (phase 019) : contrat du profil distribué `gmd_latin_brazilian_bossa`, observé dans le flux de
//! `MeasureSnapshot` (principe VI).
//!
//! Bossa nova sous une grille : guitare nylon au comping syncopé (cycle de deux mesures, attaque anticipée sur le
//! « et » du quatre), basse fondamentale-quinte anticipée (`pickup`), batterie du kit (charleston en croches à
//! l'exposition, ride au corps, cross-stick, shaker), arc sur cinq chorus et habillage par graine. Les attendus sont
//! écrits en clair, relatifs à la tonique : les grilles par défaut (une par boucle écrite dans le profil), la basse
//! sur les changements d'accord, le comping dans l'accord publié, le kit, le feel droit, le tempo de chaque variante.
//!
//! Le profil n'a pas de `[clave]` : le « cycle de deux mesures » du comping est un cycle de phrases, pas une clave
//! contrainte ni mesurée (règle de métier, README du profil).
//!
//! Un chorus de seize mesures, cinq chorus par air (80 mesures), tonique tirée par la graine (`position.key`).
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

const ID: &str = "gmd_latin_brazilian_bossa";
const CHORDS: u8 = 1;
const BASS: u8 = 2;
const PERCUSSION_LOW: u8 = 3;
const PERCUSSION_HIGH: u8 = 4;
const PERCUSSION: [u8; 2] = [PERCUSSION_LOW, PERCUSSION_HIGH];
/// Notes du kit GM que les patterns, phrases et relances du profil peuvent jouer : grosse caisse, cross-stick, caisse
/// claire, charleston fermé, pédale, tom médium, ride, shaker.
const KIT: [u8; 8] = [36, 37, 38, 42, 44, 47, 51, 82];
const KICK: u8 = 36;
const SIDESTICK: u8 = 37;
const HAT: u8 = 42;
const PEDAL: u8 = 44;
const RIDE: u8 = 51;
const SHAKER: u8 = 82;
const BARS: usize = 16;
const CHORUSES: usize = 5;
const SEEDS: [u64; 4] = [1, 2, 3, 17];
/// Fenêtre de la fondamentale de basse (`low..=high` du profil) et étendue des notes jouées (WJazzD LATIN : 28 à 53).
const ROOT_WINDOW: (u8, u8) = (34, 45);
const BASS_RANGE: (u8, u8) = (28, 53);
/// Plages de BPM des trois variantes de tempo (`session.toml`).
const TEMPOS: [(&str, f32, f32); 3] =
    [("lent", 104.0, 112.0), ("medium", 122.0, 132.0), ("vif", 152.0, 180.0)];
/// Tonalités des variantes (tonique en demi-tons au-dessus de do) : do, ré, fa, sol, si♭, mi♭.
const KEYS: [u8; 6] = [0, 2, 5, 7, 10, 3];

/// Une boucle de grille : (demi-tons au-dessus de la tonique, durée en pas), `steps_per_bar` = 16.
type Loop = &'static [(u8, usize)];

const TOURNE: Loop = &[
    (0, 16),
    (9, 16),
    (2, 16),
    (7, 16),
    (0, 16),
    (9, 16),
    (2, 16),
    (7, 16),
    (0, 16),
    (9, 16),
    (2, 16),
    (7, 16),
    (0, 16),
    (9, 16),
    (2, 16),
    (7, 16),
];
const II_V_I: Loop = &[
    (2, 16),
    (7, 16),
    (0, 32),
    (2, 16),
    (7, 16),
    (0, 32),
    (2, 16),
    (7, 16),
    (0, 32),
    (2, 16),
    (7, 16),
    (0, 32),
];
const TOURNE_A_LA_BLANCHE: Loop = &[
    (0, 16),
    (9, 16),
    (2, 8),
    (7, 8),
    (0, 16),
    (9, 16),
    (2, 8),
    (7, 8),
    (0, 16),
    (9, 16),
    (2, 8),
    (7, 8),
    (0, 16),
    (9, 16),
    (2, 8),
    (7, 8),
    (0, 16),
    (9, 16),
    (2, 8),
    (7, 8),
    (0, 16),
];
const HUIT_MESURES: Loop = &[
    (0, 32),
    (5, 16),
    (2, 16),
    (7, 16),
    (0, 16),
    (9, 16),
    (2, 8),
    (7, 8),
    (0, 32),
    (5, 16),
    (2, 16),
    (7, 16),
    (0, 16),
    (9, 16),
    (2, 8),
    (7, 8),
];
const II_V_I_IV: Loop = &[
    (2, 16),
    (7, 16),
    (0, 16),
    (5, 16),
    (2, 16),
    (7, 16),
    (0, 16),
    (5, 16),
    (2, 16),
    (7, 16),
    (0, 16),
    (5, 16),
    (2, 16),
    (7, 16),
    (0, 16),
    (5, 16),
];
const LOOPS: [Loop; 5] = [TOURNE, II_V_I, TOURNE_A_LA_BLANCHE, HUIT_MESURES, II_V_I_IV];

fn profile() -> TestResult<StyleProfile> {
    Ok(StyleProfile::named(ID)?)
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

/// Les cinq chorus de l'air, intensité et densité neutres.
fn air(seed: u64) -> TestResult<Vec<MeasureSnapshot>> {
    stream(seed, CHORUSES * BARS, 0.5, 1)
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
fn chorus(flow: &[MeasureSnapshot], n: usize) -> Vec<&MeasureSnapshot> {
    flow[(n - 1) * BARS..n * BARS].iter().collect()
}

/// Tonique tirée de l'air (la même sur toutes les mesures).
fn key_of(s: &MeasureSnapshot) -> TestResult<u8> {
    Ok(s.position.ok_or("position")?.key)
}

/// Degré (demi-tons au-dessus de la tonique) d'un accord publié.
fn degree(key: u8, c: &ChordContext) -> u8 {
    (c.root_pc + 12 - key) % 12
}

fn context_at(s: &MeasureSnapshot, step: usize) -> &ChordContext {
    &s.chord_changes[s.chord_changes.partition_point(|c| c.start_step <= step) - 1].context
}

/// (pas, degré) de chaque changement publié d'une mesure.
fn changes(key: u8, s: &MeasureSnapshot) -> Vec<(usize, u8)> {
    s.chord_changes.iter().map(|c| (c.start_step, degree(key, &c.context))).collect()
}

/// Les changements par mesure d'une boucle écrite : un pas 0 par mesure, puis chaque changement de fondamentale
/// tombant dans la mesure.
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

fn drawn(s: &MeasureSnapshot, group: &str) -> String {
    s.session.iter().find(|v| v.group == group).map(|v| v.name.clone()).unwrap_or_default()
}

/// Les deux dernières mesures d'un chorus (après le premier) peuvent porter un break ou un stop-time de l'arc.
fn may_be_broken(s: &MeasureSnapshot) -> bool {
    let in_chorus = (s.index - 1) % BARS;
    in_chorus >= BARS - 2 && s.index > BARS
}

#[test]
fn la_grille_par_defaut_est_une_des_boucles_ecrites_a_la_tonique_tiree_et_la_graine_en_choisit_plusieurs()
-> TestResult {
    let expected: Vec<Vec<Vec<(usize, u8)>>> = LOOPS.iter().map(|l| expected_changes(l)).collect();
    for e in &expected {
        assert_eq!(e.len(), BARS, "une boucle écrite fait {BARS} mesures");
    }
    let mut seen = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let key = key_of(&flow[0])?;
        keys.insert(key);
        assert!(KEYS.contains(&key), "graine {seed} : tonique {key} hors des variantes");
        let first: Vec<Vec<(usize, u8)>> = flow[..BARS].iter().map(|s| changes(key, s)).collect();
        let which = expected.iter().position(|e| *e == first).ok_or_else(|| {
            format!("graine {seed} : grille hors des boucles écrites : {first:?}")
        })?;
        seen.insert(which);
        // Les chorus suivants rejouent la même boucle, dans la même tonalité et le même mode.
        for n in 2..=CHORUSES {
            let again: Vec<Vec<(usize, u8)>> =
                chorus(&flow, n).iter().map(|s| changes(key, s)).collect();
            assert_eq!(again, first, "graine {seed} chorus {n}");
        }
        for s in &flow {
            assert_eq!(key_of(s)?, key, "graine {seed} : la tonique ne bouge pas");
            assert_eq!(
                format!("{:?}", s.position.ok_or("position")?.mode),
                "Ionian",
                "graine {seed} : mode ionien"
            );
        }
    }
    assert!(seen.len() >= 3, "16 graines n'ont tiré que {} grilles", seen.len());
    assert!(keys.len() >= 4, "16 graines n'ont tiré que {} tonalités : {keys:?}", keys.len());
    Ok(())
}

#[test]
fn la_basse_arrive_sur_la_basse_publiee_a_chaque_changement_dans_la_fenetre_du_profil() -> TestResult
{
    for seed in SEEDS {
        let flow = air(seed)?;
        for s in &flow {
            let ctx = format!("graine {seed} mesure {}", s.index);
            let bass = track(s, BASS);
            let broken = may_be_broken(s);
            for note in &bass {
                assert!(
                    (BASS_RANGE.0..=BASS_RANGE.1).contains(&note.pitch),
                    "{ctx} : hauteur {} hors de l'étendue de basse",
                    note.pitch
                );
            }
            // Un break complet de l'arc (basse et accords se taisent) ne laisse aucune attaque, pas même au un.
            if broken && bass.is_empty() {
                continue;
            }
            for change in s.chord_changes.iter().filter(|c| !broken || c.start_step == 0) {
                let arrival =
                    bass.iter().find(|n| n.start_step == change.start_step).ok_or_else(|| {
                        format!("{ctx} pas {} : pas d'attaque de basse", change.start_step)
                    })?;
                assert_eq!(
                    arrival.pitch % 12,
                    change.context.bass_pc,
                    "{ctx} pas {} : arrivée sur {} au lieu de la basse publiée",
                    change.start_step,
                    arrival.pitch % 12
                );
                assert!(
                    (ROOT_WINDOW.0..=ROOT_WINDOW.1).contains(&arrival.pitch),
                    "{ctx} pas {} : arrivée {} hors de {}..={}",
                    change.start_step,
                    arrival.pitch,
                    ROOT_WINDOW.0,
                    ROOT_WINDOW.1
                );
            }
        }
    }
    Ok(())
}

#[test]
fn la_basse_joue_fondamentale_quinte_anticipation_de_la_mesure_suivante_ou_approche() -> TestResult
{
    let (mut pickups, mut bars_with_pickup, mut approaches, mut fifths) = (0, 0, 0, 0);
    for seed in SEEDS {
        let flow = air(seed)?;
        for (index, s) in flow.iter().enumerate() {
            let mut bass = track(s, BASS);
            bass.sort_by_key(|n| n.start_step);
            let next_bar = flow.get(index + 1);
            for (rank, note) in bass.iter().enumerate() {
                let chord = context_at(s, note.start_step);
                let rel = (note.pitch + 12 - chord.bass_pc) % 12;
                if rel == 0 {
                    continue;
                }
                if rel == 7 {
                    fifths += 1;
                    continue;
                }
                // Anticipation (`pickup`, `when = "bar_end"`) : au pas 14, la basse de l'accord de la mesure suivante.
                if note.start_step == 14 {
                    let target =
                        next_bar.ok_or("mesure suivante")?.chord_changes[0].context.bass_pc;
                    if note.pitch % 12 == target {
                        pickups += 1;
                        continue;
                    }
                }
                // Sinon une approche : à un ou deux demi-tons de la note de basse suivante (de la mesure
                // suivante pour la dernière).
                let next = bass.get(rank + 1).map(|n| n.pitch).or_else(|| {
                    next_bar.and_then(|n| {
                        track(n, BASS).iter().map(|b| (b.start_step, b.pitch)).min().map(|b| b.1)
                    })
                });
                let next = next.ok_or("cible d'approche")?;
                assert!(
                    (1..=2).contains(&note.pitch.abs_diff(next)),
                    "graine {seed} mesure {} pas {} : {} n'est ni fondamentale, ni quinte, ni anticipation, ni approche de {next}",
                    s.index,
                    note.start_step,
                    note.pitch
                );
                approaches += 1;
            }
            // Hors break et hors dernière mesure de l'air, la mesure se termine par l'anticipation.
            if !may_be_broken(s) && next_bar.is_some() && !bass.is_empty() {
                bars_with_pickup += 1;
                let target = next_bar.ok_or("mesure suivante")?.chord_changes[0].context.bass_pc;
                assert!(
                    bass.iter().any(|n| n.start_step == 14 && n.pitch % 12 == target),
                    "graine {seed} mesure {} : pas d'anticipation de la basse suivante au pas 14",
                    s.index
                );
            }
        }
    }
    assert!(
        pickups >= bars_with_pickup / 2 && bars_with_pickup > 200,
        "{pickups} anticipations pour {bars_with_pickup} mesures"
    );
    assert!(approaches > 0, "aucune approche sur {} graines", SEEDS.len());
    assert!(fifths > 100, "{fifths} quintes seulement : la basse est fondamentale-quinte");
    Ok(())
}

#[test]
fn le_comping_ne_joue_que_des_notes_de_l_accord_publie_ou_de_sa_mesure_suivante_anticipee()
-> TestResult {
    let mut anticipated = 0;
    let mut colored = 0;
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let voicing = drawn(&flow[0], "voicing");
        let register = drawn(&flow[0], "registre");
        // Fenêtre `low..high` du profil (48..78), décalée d'une octave par le registre « grave ».
        let shift: i16 = if register == "grave" { -12 } else { 0 };
        let window = (48 + shift, 78 + shift);
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
                let ctx = format!("graine {seed} ({voicing}) mesure {} pas {step}", snap.index);
                let here = context_at(snap, step);
                let played = pcs(&pitches);
                // Une tension naturelle (neuvième, onzième, treizième) de la couleur tirée peut n'être pas publiée.
                let fits = |c: &ChordContext| {
                    played.iter().all(|pc| {
                        c.pitch_classes.contains(pc)
                            || [2, 5, 9].contains(&((pc + 12 - c.root_pc) % 12))
                    })
                };
                let strict =
                    |c: &ChordContext| played.iter().all(|pc| c.pitch_classes.contains(pc));
                // Une attaque qui finit sur la barre peut être anticipée : elle joue alors l'accord de la
                // mesure suivante.
                let ahead = (step + duration == 16).then(|| flow.get(index + 1)).flatten();
                // Toute attaque du profil qui finit sur la barre est `anticipate` : l'accord suivant prime.
                let next = ahead.map(|a| &a.chord_changes[0].context);
                let sounding = match next.filter(|c| fits(c)) {
                    Some(c) => c,
                    None if fits(here) => here,
                    None => {
                        return Err(format!(
                            "{ctx} : {played:?} hors de {} {:?}",
                            here.chord_name, here.pitch_classes
                        )
                        .into());
                    }
                };
                if !std::ptr::eq(sounding, here) {
                    anticipated += 1;
                }
                colored += usize::from(!strict(sounding));
                // Notes guides, shell et sans fondamentale ne jouent jamais la fondamentale ; le drop 2 (accord serré) et la
                // structure supérieure (triade bâtie sur un autre degré, qui peut contenir la fondamentale) le peuvent.
                if !["drop_2", "structure_superieure"].contains(&voicing.as_str()) {
                    assert!(
                        !played.contains(&sounding.root_pc),
                        "{ctx} : fondamentale jouée : {pitches:?} sur {} {:?} (mesure suivante : {:?})",
                        sounding.chord_name,
                        sounding.pitch_classes,
                        next.map(|c| &c.chord_name)
                    );
                }
                assert!(
                    i16::from(pitches[0]) >= window.0
                        && i16::from(pitches[pitches.len() - 1]) <= window.1,
                    "{ctx} : tessiture {pitches:?} hors de {window:?}"
                );
            }
        }
    }
    // Les attaques anticipées jouent bien l'accord SUIVANT quand il change à la barre (un profil sans `anticipate`
    // n'en a aucune) ; la couleur tirée ajoute des tensions non publiées.
    assert!(anticipated >= 20, "{anticipated} attaques anticipées sur 16 graines");
    assert!(colored > 0, "aucune tension de couleur sur 16 graines");
    Ok(())
}

#[test]
fn la_bossa_joue_son_cycle_de_deux_mesures_trois_puis_deux() -> TestResult {
    // Rythme « trois » (pas 0, 6, 12) puis « deux » (pas 4, 8, 14) : le cycle de deux mesures des phrases de comping
    // `medium`. Les phrases de 2 et 4 mesures se découpent depuis le début de la passe : un « trois » tombe sur une
    // mesure impaire du chorus, un « deux » sur une mesure paire, et le « trois » est le plus souvent suivi d'un
    // « deux » (les variations des phrases de quatre mesures le font suivre d'une autre mesure).
    let three: BTreeSet<usize> = BTreeSet::from([0, 6, 12]);
    let two: BTreeSet<usize> = BTreeSet::from([4, 8, 14]);
    let (mut pairs, mut threes) = (0, 0);
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        for (index, s) in flow.iter().enumerate() {
            let played = attacks(s, CHORDS);
            if played == two {
                assert_eq!(
                    index % 2,
                    1,
                    "graine {seed} mesure {} : « deux » sur une mesure impaire",
                    s.index
                );
            }
            if played != three {
                continue;
            }
            threes += 1;
            assert_eq!(
                index % 2,
                0,
                "graine {seed} mesure {} : « trois » sur une mesure paire",
                s.index
            );
            pairs +=
                usize::from(flow.get(index + 1).is_some_and(|next| attacks(next, CHORDS) == two));
        }
    }
    assert!(threes >= 20, "{threes} mesures « trois » sur 16 graines");
    assert!(pairs * 10 >= threes * 7, "« trois » suivi de « deux » : {pairs} sur {threes}");
    Ok(())
}

#[test]
fn la_batterie_reste_dans_le_kit_pose_la_grosse_caisse_de_la_bossa_et_route_les_voix() -> TestResult
{
    for seed in SEEDS {
        let flow = air(seed)?;
        let (mut kick, mut pedal, mut full, mut sidestick, mut shaker) = (0, 0, 0, 0, 0);
        for s in &flow {
            let notes = percussion(s);
            assert!(
                notes.iter().all(|n| KIT.contains(&n.pitch)),
                "graine {seed} mesure {} : note hors du kit : {:?}",
                s.index,
                notes.iter().map(|n| n.pitch).collect::<Vec<_>>()
            );
            // Routage : la grosse caisse sur `PercussionLow`, tout le reste (dont le shaker GM) sur `PercussionHigh`.
            for n in &notes {
                let expected = if n.pitch == KICK { PERCUSSION_LOW } else { PERCUSSION_HIGH };
                assert_eq!(
                    n.track, expected,
                    "graine {seed} mesure {} : note {}",
                    s.index, n.pitch
                );
            }
            let kick_steps: BTreeSet<usize> =
                notes.iter().filter(|n| n.pitch == KICK).map(|n| n.start_step).collect();
            // Grosse caisse mesurée dans le GMD : pas 0, 6, 8, 14 (86 à 90 % des mesures) ; seule une relance
            // (fin de phrase, à partir du pas 8 ou 12) la remplace.
            kick += usize::from([0, 6, 8, 14].iter().all(|s| kick_steps.contains(s)));
            let pedal_steps: BTreeSet<usize> =
                notes.iter().filter(|n| n.pitch == PEDAL).map(|n| n.start_step).collect();
            pedal += usize::from(pedal_steps.contains(&4) && pedal_steps.contains(&12));
            full += usize::from(!notes.is_empty());
            sidestick += usize::from(notes.iter().any(|n| n.pitch == SIDESTICK));
            shaker += usize::from(notes.iter().any(|n| n.pitch == SHAKER));
        }
        assert_eq!(full, flow.len(), "graine {seed} : la batterie ne se tait jamais");
        assert!(
            kick * 100 >= flow.len() * 70,
            "graine {seed} : grosse caisse [0, 6, 8, 14] sur {kick} mesures"
        );
        assert!(
            pedal * 100 >= flow.len() * 70,
            "graine {seed} : pédale aux temps 2 et 4 sur {pedal} mesures"
        );
        assert!(sidestick > 0, "graine {seed} : aucun cross-stick");
        // Le shaker (métier) n'entre qu'à la densité 2 : aucun, ou à faible dose, au niveau neutre.
        assert!(shaker * 2 <= flow.len(), "graine {seed} : shaker sur {shaker} mesures");
    }
    // Densité 2 : le kit reste le même (caisse claire fantôme, tom médium, shaker GM sur `PercussionHigh`).
    let flow = stream(2, CHORUSES * BARS, 1.0, 2)?;
    let shaker = flow.iter().filter(|s| percussion(s).iter().any(|n| n.pitch == SHAKER)).count();
    assert!(shaker * 4 >= flow.len(), "densité 2 : shaker sur {shaker} mesures de {}", flow.len());
    for s in &flow {
        for n in percussion(s) {
            assert!(
                KIT.contains(&n.pitch),
                "densité 2, mesure {} : note {} hors du kit",
                s.index,
                n.pitch
            );
            let expected = if n.pitch == KICK { PERCUSSION_LOW } else { PERCUSSION_HIGH };
            assert_eq!(n.track, expected, "densité 2, mesure {} : note {}", s.index, n.pitch);
        }
    }
    Ok(())
}

#[test]
fn le_charleston_porte_l_exposition_et_la_retombee_le_ride_le_corps_et_le_sommet() -> TestResult {
    for seed in SEEDS {
        let flow = air(seed)?;
        let count = |n: usize, pitch: u8| -> usize {
            chorus(&flow, n)
                .iter()
                .map(|s| percussion(s).iter().filter(|x| x.pitch == pitch).count())
                .sum()
        };
        // Exposition et retombée : étiquette `hat`, charleston en croches, aucun ride.
        for n in [1, CHORUSES] {
            assert_eq!(
                count(n, RIDE),
                0,
                "graine {seed} chorus {n} : ride à l'exposition ou à la retombée"
            );
            assert!(
                count(n, HAT) >= 8 * BARS / 2,
                "graine {seed} chorus {n} : {} coups de charleston",
                count(n, HAT)
            );
        }
        // Sommet : étiquette `ride` seule, plus aucun charleston fermé.
        assert_eq!(count(4, HAT), 0, "graine {seed} : charleston au sommet");
        assert!(
            count(4, RIDE) >= 8 * BARS / 2,
            "graine {seed} : {} coups de ride au sommet",
            count(4, RIDE)
        );
    }
    Ok(())
}

#[test]
fn le_feel_est_droit_en_seize_pas_et_le_tempo_reste_dans_la_plage_de_la_variante() -> TestResult {
    let p = profile()?;
    assert_eq!((p.meter.min_bpm, p.meter.default_bpm, p.meter.max_bpm), (104.0, 127.0, 180.0));
    assert!((p.meter.default_swing_ratio - 0.5).abs() < 1e-6, "feel droit");
    let mut names = BTreeSet::new();
    for seed in 1..=16u64 {
        for intensity in [0.0f32, 0.5, 1.0] {
            let flow = stream(seed, BARS + 2, intensity, 1)?;
            let name = drawn(&flow[0], "tempo");
            let &(_, low, high) = TEMPOS
                .iter()
                .find(|(n, ..)| *n == name)
                .ok_or_else(|| format!("variante de tempo « {name} »"))?;
            names.insert(name.clone());
            for s in &flow {
                assert_eq!(s.steps, 16, "seize pas par mesure");
                assert!(
                    (low - 4.0..=high + 4.0).contains(&s.tempo),
                    "graine {seed} intensité {intensity} ({name}) : tempo {} hors de {low}..{high} ± 4",
                    s.tempo
                );
                assert_eq!(s.tempo, flow[0].tempo, "le tempo d'une session ne bouge pas");
                assert_eq!(s.composition_bpm.fract(), 0.0, "BPM entier tiré");
                assert!((s.swing_ratio - 0.5).abs() < 1e-6, "swing {}", s.swing_ratio);
            }
        }
    }
    assert_eq!(names.len(), 3, "les trois tempos sont tirés sur 16 graines : {names:?}");
    // Doubles croches : à la densité 2 une mesure au moins attaque un pas impair (grosse caisse ou caisse claire fantôme).
    let flow = stream(2, 4 * BARS, 1.0, 2)?;
    assert!(
        flow.iter().any(|s| percussion(s).iter().any(|n| n.start_step % 2 == 1)),
        "aucune double croche à la batterie"
    );
    Ok(())
}

#[test]
fn l_arc_creuse_l_ecart_entre_l_exposition_le_sommet_et_la_retombee() -> TestResult {
    for seed in SEEDS {
        let flow = air(seed)?;
        let notes = |n: usize| -> usize {
            chorus(&flow, n).iter().map(|s| track(s, CHORDS).len() + percussion(s).len()).sum()
        };
        let (first, peak, fall) = (notes(1), notes(4), notes(5));
        assert!(first < peak, "graine {seed} : exposition {first} contre sommet {peak}");
        assert!(fall < peak, "graine {seed} : retombée {fall} contre sommet {peak}");
        // Retombée : stop-time sur les deux dernières mesures, accords et basse sur le un seulement.
        for s in chorus(&flow, CHORUSES).iter().rev().take(2) {
            for id in [CHORDS, BASS] {
                assert!(
                    attacks(s, id).iter().all(|&step| step == 0),
                    "graine {seed} mesure {} piste {id} : {:?}",
                    s.index,
                    attacks(s, id)
                );
            }
            assert!(!percussion(s).is_empty(), "mesure {} : la batterie continue", s.index);
        }
    }
    Ok(())
}

#[test]
fn l_arc_orchestre_change_les_instruments_a_chaque_frontiere_de_chorus() -> TestResult {
    let mut checked = 0;
    for seed in 1..=40u64 {
        let flow = air(seed)?;
        if drawn(&flow[0], "arc") != "orchestre" {
            continue;
        }
        checked += 1;
        let programs = |n: usize| chorus(&flow, n)[0].instruments;
        let first = programs(1).ok_or("instruments")?;
        let (second, third) =
            (programs(2).ok_or("instruments")?, programs(3).ok_or("instruments")?);
        let (peak, fall) = (programs(4).ok_or("instruments")?, programs(5).ok_or("instruments")?);
        assert_eq!(first.percussion_program, 40, "graine {seed} : exposition aux balais");
        assert_eq!(
            second, third,
            "graine {seed} : les chorus de corps gardent les instruments de la session"
        );
        assert_eq!(peak.chords_program, Some(4), "graine {seed} : le piano électrique au sommet");
        assert_eq!(
            (fall.chords_program, fall.percussion_program),
            (Some(24), 40),
            "graine {seed} : retombée"
        );
        // Un seul programme par chorus : tous les snapshots d'un chorus portent les mêmes.
        for n in 1..=CHORUSES {
            for s in chorus(&flow, n) {
                assert_eq!(
                    s.instruments,
                    chorus(&flow, n)[0].instruments,
                    "graine {seed} chorus {n}"
                );
            }
        }
        // La basse garde le programme de la session à tous les chorus.
        assert!([32, 33, 35].contains(&first.bass_program.ok_or("basse")?));
        assert!([first, second, peak, fall].iter().all(|i| i.bass_program == first.bass_program));
    }
    assert!(checked >= 3, "{checked} graines à arc « orchestre » parmi 40");
    Ok(())
}

/// Un habillage tel que les snapshots du premier chorus le montrent.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Dressing {
    tempo: String,
    key: u8,
    chords: u8,
    bass: u8,
    kit: u8,
    voicing: String,
    register: String,
    energy: String,
    arc: String,
}

#[test]
fn des_graines_differentes_donnent_des_habillages_differents() -> TestResult {
    let mut all: Vec<(f32, Dressing)> = Vec::new();
    for seed in 1..=16u64 {
        let flow = stream(seed, BARS, 0.5, 1)?;
        let first = &flow[0];
        let instruments = first.instruments.ok_or("instruments publiés")?;
        assert_eq!(first.session.len(), 9, "un tirage par groupe : {:?}", first.session);
        all.push((
            first.tempo,
            Dressing {
                tempo: drawn(first, "tempo"),
                key: key_of(first)?,
                chords: instruments.chords_program.ok_or("comping")?,
                bass: instruments.bass_program.ok_or("basse")?,
                kit: instruments.percussion_program,
                voicing: drawn(first, "voicing"),
                register: drawn(first, "registre"),
                energy: drawn(first, "energie"),
                arc: drawn(first, "arc"),
            },
        ));
    }
    let distinct: BTreeSet<_> = all.iter().map(|(_, d)| d.clone()).collect();
    let keys: BTreeSet<_> = all.iter().map(|(_, d)| d.key).collect();
    let chords: BTreeSet<_> = all.iter().map(|(_, d)| d.chords).collect();
    let bass: BTreeSet<_> = all.iter().map(|(_, d)| d.bass).collect();
    let kits: BTreeSet<_> = all.iter().map(|(_, d)| d.kit).collect();
    let voicings: BTreeSet<_> = all.iter().map(|(_, d)| d.voicing.clone()).collect();
    let tempos: BTreeSet<_> = all.iter().map(|(_, d)| d.tempo.clone()).collect();
    // ± 8 BPM : des grappes de 16 BPM de large au plus ; les plages lent / médium / vif sont séparées de plus de 8 BPM.
    let mut sorted: Vec<f32> = all.iter().map(|(t, _)| *t).collect();
    sorted.sort_by(f32::total_cmp);
    let mut clusters = 0;
    let mut start = f32::NEG_INFINITY;
    for t in &sorted {
        if t - start > 8.0 {
            clusters += 1;
            start = *t;
        }
    }
    println!(
        "16 graines : {} habillages distincts ; tempos {sorted:?} ; tonalités {keys:?} ; comping {chords:?} ; basse {bass:?} ; kit {kits:?} ; voicings {voicings:?}",
        distinct.len()
    );
    assert!(distinct.len() >= 8, "{} habillages distincts sur 16 graines", distinct.len());
    assert_eq!(tempos.len(), 3, "trois tempos tirés : {tempos:?}");
    assert!(clusters >= 3, "{clusters} grappes de tempo à ±8 BPM : {sorted:?}");
    assert!(keys.len() >= 4 && keys.iter().all(|k| KEYS.contains(k)), "tonalités : {keys:?}");
    assert!(
        chords.len() >= 3 && chords.iter().all(|p| [24, 25, 26, 0, 4].contains(p)),
        "comping : {chords:?}"
    );
    assert!(bass.len() >= 2 && bass.iter().all(|p| [32, 33, 35].contains(p)), "basse : {bass:?}");
    assert!(kits.len() >= 2 && kits.iter().all(|p| [0, 8, 32, 40].contains(p)), "kits : {kits:?}");
    assert!(voicings.len() >= 3, "familles de voicings : {voicings:?}");
    Ok(())
}

#[test]
fn la_graine_change_le_comping_la_batterie_et_la_basse_et_la_meme_graine_rejoue_pareil()
-> TestResult {
    let a = air(1)?;
    let again = air(1)?;
    assert_eq!(serde_json::to_string(&a)?, serde_json::to_string(&again)?);
    let mut comping = BTreeSet::new();
    let mut drums = BTreeSet::new();
    let mut bass = BTreeSet::new();
    for seed in 1..=16u64 {
        let flow = air(seed)?;
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
    assert!(comping.len() >= 14, "{} comping distincts sur 16 graines", comping.len());
    assert!(drums.len() >= 14, "{} batteries distinctes sur 16 graines", drums.len());
    assert!(bass.len() >= 12, "{} basses distinctes sur 16 graines", bass.len());
    Ok(())
}

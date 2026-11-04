//! B3 (phase 019) : contrat du profil distribué `gmd_afrocuban`, observé dans le flux de `MeasureSnapshot`
//! (principe VI).
//!
//! Les attendus sont écrits en clair : les cinq grilles par défaut (relatives à la tonique tirée par la
//! session), la clave son de deux mesures (« trois » aux pas 0, 6, 12, « deux » aux pas 4, 8), les montunos
//! de chaque côté de la clave, le tumbao de la basse (pas 6 et 12, anticipation au pas 14), les voix
//! latines du kit sur leurs pistes, le tempo des trois variantes mesurées dans la source, l'arc de cinq
//! chorus (exposition, deux chorus de corps, sommet, retombée) et l'habillage de la graine.
//!
//! Le profil est lu depuis son dossier (`profiles/gmd_afrocuban`) : un seul test vérifie que la copie
//! distribuée dans le binaire est la même.
use std::collections::{BTreeMap, BTreeSet};

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

const ID: &str = "gmd_afrocuban";
const CHORDS: u8 = 1;
const BASS: u8 = 2;
const LOW: u8 = 3;
const HIGH: u8 = 4;
const PERCUSSION: [u8; 2] = [LOW, HIGH];
/// Mesures d'un chorus et chorus d'un air.
const BARS: usize = 16;
const CHORUSES: usize = 5;
const SEEDS: [u64; 4] = [1, 2, 3, 17];

/// Notes du kit GM de GeneralUser GS que le profil peut jouer, avec leur piste.
const CLAVES: u8 = 75;
const RIDE_BELL: u8 = 53;
const COWBELL: u8 = 56;
const TOM_HIGH: u8 = 50;
/// `PercussionLow` : grosse caisse, congas (62 sourde, 63 ouverte, 64 grave), timbale grave.
const LOW_NOTES: [u8; 5] = [36, 62, 63, 64, 66];
/// `PercussionHigh` : le reste du kit et les voix latines aiguës.
const HIGH_NOTES: [u8; 17] = [37, 38, 42, 43, 44, 50, 51, 53, 56, 60, 61, 65, 70, 73, 74, 75, 82];

/// Une boucle de grille : (demi-tons au-dessus de la tonique, durée en pas), seize pas par mesure.
type Loop = &'static [(u8, usize)];
const LOOPS: [Loop; 5] = [
    &[(0, 256)],
    &[(0, 64), (5, 64), (7, 64), (0, 64)],
    &[(0, 32), (5, 32), (0, 32), (5, 32), (0, 32), (5, 32), (0, 32), (5, 32)],
    &[(2, 32), (7, 32), (0, 64), (2, 32), (7, 32), (0, 64)],
    &[(0, 32), (10, 32), (8, 32), (7, 32), (0, 32), (10, 32), (8, 32), (7, 32)],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Three,
    Two,
}

/// Attaques d'un motif de montuno : (toutes, obligatoires) ; les coups probabilistes sont facultatifs.
type Montuno = (&'static [usize], &'static [usize]);
/// Montunos du côté « trois » (autour des pas 0, 6, 12), écrits dans `profile.toml`.
const MONTUNO_THREE: [Montuno; 6] = [
    (&[0, 6, 12], &[0, 6, 12]),
    (&[0, 8, 14], &[0, 8, 14]),
    (&[0, 6, 10, 12], &[0, 6, 10, 12]),
    (&[2, 6, 8, 12], &[2, 6, 8, 12]),
    (&[0, 2, 6, 8, 10, 12], &[0, 6, 10, 12]),
    (&[0, 3, 6, 8, 11, 12, 14], &[0, 6, 8, 12, 14]),
];
/// Montunos du côté « deux » (autour des pas 4, 8).
const MONTUNO_TWO: [Montuno; 6] = [
    (&[4, 8, 14], &[4, 8, 14]),
    (&[2, 6, 12], &[2, 6, 12]),
    (&[2, 4, 8, 14], &[2, 4, 8, 14]),
    (&[0, 4, 7, 10, 14], &[0, 4, 7, 10, 14]),
    (&[2, 4, 6, 8, 10, 14], &[2, 4, 8, 14]),
    (&[0, 2, 4, 7, 8, 12, 14], &[2, 4, 8, 14]),
];

fn profile() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!("{}/../profiles/{ID}", env!("CARGO_MANIFEST_DIR")))?)
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

/// Pas d'attaque d'une note de percussion.
fn drum_steps(s: &MeasureSnapshot, pitch: u8) -> BTreeSet<usize> {
    percussion(s).iter().filter(|n| n.pitch == pitch).map(|n| n.start_step).collect()
}

/// Mesures du chorus `n` (1-based).
fn chorus(flow: &[MeasureSnapshot], n: usize) -> &[MeasureSnapshot] {
    &flow[(n - 1) * BARS..n * BARS]
}

fn key_of(s: &MeasureSnapshot) -> TestResult<u8> {
    Ok(s.position.as_ref().ok_or("position de l'air")?.key)
}

fn degree(s: &MeasureSnapshot, c: &ChordContext) -> TestResult<u8> {
    Ok((c.root_pc + 12 - key_of(s)?) % 12)
}

fn context_at(s: &MeasureSnapshot, step: usize) -> &ChordContext {
    &s.chord_changes[s.chord_changes.partition_point(|c| c.start_step <= step) - 1].context
}

fn changes(s: &MeasureSnapshot) -> TestResult<Vec<(usize, u8)>> {
    s.chord_changes.iter().map(|c| Ok((c.start_step, degree(s, &c.context)?))).collect()
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

/// Côté de la clave que la mesure joue, lu sur la voix `claves` seule.
fn clave_side(s: &MeasureSnapshot) -> TestResult<Side> {
    let steps = drum_steps(s, CLAVES);
    if steps == BTreeSet::from([0, 6, 12]) {
        Ok(Side::Three)
    } else if steps == BTreeSet::from([4, 8]) {
        Ok(Side::Two)
    } else {
        Err(format!("mesure {} : claves aux pas {steps:?}, ni 0 6 12 ni 4 8", s.index).into())
    }
}

/// Mesure de stop-time ou de break de l'arc : les deux dernières mesures des chorus 4 et 5 (accords et basse sur
/// le un seulement) ; la dernière des chorus 2 et 3 peut taire les accords et la basse ; la mesure 10 des chorus 2
/// et 3 peut taire les accords.
fn chorus_of(s: &MeasureSnapshot) -> (usize, usize) {
    ((s.index - 1) / BARS + 1, (s.index - 1) % BARS)
}

#[test]
fn le_profil_distribue_est_celui_du_dossier() -> TestResult {
    assert_eq!(StyleProfile::named(ID)?, profile()?);
    Ok(())
}

#[test]
fn la_grille_par_defaut_est_une_des_boucles_ecrites_et_se_reprend_a_l_identique_a_chaque_chorus()
-> TestResult {
    let expected: Vec<Vec<Vec<(usize, u8)>>> = LOOPS.iter().map(|l| expected_changes(l)).collect();
    for e in &expected {
        assert_eq!(e.len(), BARS, "une boucle écrite fait {BARS} mesures");
    }
    let mut seen = BTreeSet::new();
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let first = chorus(&flow, 1).iter().map(changes).collect::<TestResult<Vec<_>>>()?;
        let which = expected.iter().position(|e| *e == first).ok_or_else(|| {
            format!("graine {seed} : grille hors des boucles écrites : {first:?}")
        })?;
        seen.insert(which);
        for n in 2..=CHORUSES {
            let again = chorus(&flow, n).iter().map(changes).collect::<TestResult<Vec<_>>>()?;
            assert_eq!(again, first, "graine {seed} chorus {n} : la boucle se reprend");
        }
    }
    assert!(seen.len() >= 4, "16 graines n'ont tiré que {} grilles sur 5", seen.len());
    Ok(())
}

#[test]
fn la_clave_alterne_trois_deux_mesure_par_mesure_dans_les_deux_sens() -> TestResult {
    let mut directions = BTreeMap::new();
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let first = clave_side(&flow[0])?;
        for (i, s) in flow.iter().enumerate() {
            let expected = if i % 2 == 0 { first } else { opposite(first) };
            assert_eq!(
                clave_side(s)?,
                expected,
                "graine {seed} mesure {} : la clave est un cycle de deux mesures, continu d'un chorus à l'autre",
                s.index
            );
        }
        *directions.entry(format!("{first:?}")).or_insert(0usize) += 1;
    }
    assert_eq!(directions.len(), 2, "3-2 et 2-3 doivent sortir sur 16 graines : {directions:?}");
    // Chaque densité de batterie (patterns des trois densités, sous toutes les étiquettes de l'arc) porte la
    // même clave : intensité et densité de la commande ne la déplacent pas.
    for density in [0u8, 1, 2] {
        for intensity in [0.0f32, 0.5, 1.0] {
            for seed in [1u64, 2] {
                let flow = stream(seed, CHORUSES * BARS, intensity, density)?;
                let first = clave_side(&flow[0])?;
                for (i, s) in flow.iter().enumerate() {
                    let expected = if i % 2 == 0 { first } else { opposite(first) };
                    assert_eq!(
                        clave_side(s)?,
                        expected,
                        "graine {seed} densité {density} intensité {intensity} mesure {}",
                        s.index
                    );
                }
            }
        }
    }
    Ok(())
}

fn opposite(side: Side) -> Side {
    match side {
        Side::Three => Side::Two,
        Side::Two => Side::Three,
    }
}

/// `true` si l'ensemble d'attaques `played` est un montuno de la famille.
fn matches_family(played: &BTreeSet<usize>, family: &[Montuno]) -> bool {
    family.iter().any(|(all, required)| {
        played.iter().all(|s| all.contains(s)) && required.iter().all(|s| played.contains(s))
    })
}

#[test]
fn les_montunos_des_deux_cotes_ne_se_confondent_pas() {
    // Garde du test suivant : aucune suite d'attaques ne peut passer pour les deux côtés.
    for (all_a, req_a) in MONTUNO_THREE {
        for (all_b, req_b) in MONTUNO_TWO {
            let both: BTreeSet<usize> = req_a.iter().chain(req_b.iter()).copied().collect();
            let confusable = both.iter().all(|s| all_a.contains(s) && all_b.contains(s));
            assert!(!confusable, "{all_a:?} et {all_b:?} peuvent se confondre");
        }
    }
}

#[test]
fn le_comping_et_la_batterie_jouent_du_meme_cote_que_la_clave() -> TestResult {
    let (mut checked, mut cowbells) = (0, 0);
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        for s in &flow {
            let side = clave_side(s)?;
            let ctx = format!("graine {seed} mesure {}", s.index);
            // Comping : un motif du côté de la clave (les mesures en stop-time ou tues n'ont que le un).
            let played = attacks(s, CHORDS);
            if played.iter().any(|&step| step != 0) {
                let (mine, other) = match side {
                    Side::Three => (&MONTUNO_THREE, &MONTUNO_TWO),
                    Side::Two => (&MONTUNO_TWO, &MONTUNO_THREE),
                };
                assert!(
                    matches_family(&played, mine) && !matches_family(&played, other),
                    "{ctx} : comping {played:?} pas du côté {side:?}"
                );
                checked += 1;
            }
            // Batterie : la cloche porte l'asymétrie des deux côtés (pas 6 côté « trois », pas 14
            // côté « deux »), cloche de ride 53 dans le corps, cowbell 56 au sommet.
            for bell in [RIDE_BELL, COWBELL] {
                let steps = drum_steps(s, bell);
                if bell == RIDE_BELL && !steps.contains(&6) && !steps.contains(&14) {
                    continue;
                }
                if bell == COWBELL && steps.is_empty() {
                    continue;
                }
                cowbells += usize::from(bell == COWBELL);
                let (here, there) = match side {
                    Side::Three => (6, 14),
                    Side::Two => (14, 6),
                };
                // Une relance de fin de chorus reprend la main au pas 13 : elle retire le coup du pas 14.
                let filled = chorus_of(s).1 == BARS - 1;
                assert!(
                    (steps.contains(&here) || filled) && !steps.contains(&there),
                    "{ctx} : cloche {bell} aux pas {steps:?}, côté {side:?}"
                );
            }
        }
    }
    assert!(checked > 600, "{checked} mesures de comping lues");
    assert!(cowbells > 100, "{cowbells} mesures à cowbell lues");
    Ok(())
}

#[test]
fn le_tempo_reste_dans_la_plage_mesuree_de_la_source_et_de_chaque_variante() -> TestResult {
    let p = profile()?;
    assert_eq!((p.meter.min_bpm, p.meter.default_bpm, p.meter.max_bpm), (105.0, 105.0, 128.0));
    let by_name = BTreeMap::from([("droit_105", 105.0), ("rhumba_110", 110.0), ("vif_128", 128.0)]);
    let mut seen = BTreeSet::new();
    for seed in 1..=16u64 {
        for intensity in [0.0f32, 0.5, 1.0] {
            let flow = stream(seed, BARS, intensity, 1)?;
            let name = drawn(&flow[0], "tempo");
            let bpm = *by_name.get(name.as_str()).ok_or_else(|| format!("variante {name}"))?;
            for s in &flow {
                assert_eq!(s.tempo, bpm, "graine {seed} intensité {intensity} : {name}");
                assert_eq!(s.composition_bpm, bpm);
                assert!((p.meter.min_bpm..=p.meter.max_bpm).contains(&s.tempo));
            }
            seen.insert(name);
        }
    }
    assert_eq!(seen.len(), 3, "les trois tempos mesurés sortent sur 16 graines : {seen:?}");
    Ok(())
}

#[test]
fn la_basse_arrive_sur_la_basse_publiee_a_chaque_changement_dans_la_fenetre_du_profil() -> TestResult
{
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        for s in &flow {
            let ctx = format!("graine {seed} mesure {}", s.index);
            let (chorus, in_chorus) = chorus_of(s);
            // La dernière mesure des chorus 2 et 3 peut être un break des accords et de la basse.
            let broken = (chorus == 2 || chorus == 3) && in_chorus == BARS - 1;
            let bass = track(s, BASS);
            for note in &bass {
                assert!(
                    (28..=53).contains(&note.pitch),
                    "{ctx} : hauteur {} hors de l'étendue de WJazzD LATIN (28..53)",
                    note.pitch
                );
            }
            for change in s.chord_changes.iter().filter(|_| !broken) {
                let arrival =
                    bass.iter().find(|n| n.start_step == change.start_step).ok_or_else(|| {
                        format!("{ctx} pas {} : pas d'attaque de basse", change.start_step)
                    })?;
                assert_eq!(
                    arrival.pitch % 12,
                    change.context.bass_pc,
                    "{ctx} pas {} : arrivée hors de la basse publiée",
                    change.start_step
                );
                assert!(
                    (35..=46).contains(&arrival.pitch),
                    "{ctx} : arrivée {} hors de la fenêtre 35..=46",
                    arrival.pitch
                );
            }
        }
    }
    Ok(())
}

#[test]
fn la_basse_joue_le_tumbao_pas_6_et_12_avec_l_anticipation_du_changement_au_pas_14() -> TestResult {
    let (mut bars, mut with_six, mut with_twelve, mut anticipated) = (0, 0, 0, 0);
    let mut approaches = 0;
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        for (index, s) in flow.iter().enumerate() {
            let (chorus, in_chorus) = chorus_of(s);
            let stopped = (chorus >= 4 && in_chorus >= BARS - 2)
                || ((chorus == 2 || chorus == 3) && in_chorus == BARS - 1);
            if stopped {
                continue;
            }
            let mut bass = track(s, BASS);
            bass.sort_by_key(|n| n.start_step);
            let next_bar = flow.get(index + 1);
            bars += 1;
            with_six += usize::from(bass.iter().any(|n| n.start_step == 6));
            with_twelve += usize::from(bass.iter().any(|n| n.start_step == 12));
            for (rank, note) in bass.iter().enumerate() {
                let ctx = format!("graine {seed} mesure {} pas {}", s.index, note.start_step);
                let chord = context_at(s, note.start_step);
                let relative = (note.pitch + 12 - chord.bass_pc) % 12;
                if note.start_step == 14 {
                    // Anticipation : la basse de l'accord de la mesure suivante.
                    let next = next_bar.ok_or("mesure suivante")?;
                    assert_eq!(
                        note.pitch % 12,
                        next.chord_changes[0].context.bass_pc,
                        "{ctx} : anticipation"
                    );
                    anticipated += 1;
                    continue;
                }
                if relative == 0 || relative == 7 {
                    continue;
                }
                // Sinon une note de passage : à un ou deux demi-tons de la note suivante (de la mesure
                // suivante pour la dernière).
                let next = bass
                    .get(rank + 1)
                    .map(|n| n.pitch)
                    .or_else(|| {
                        next_bar.and_then(|n| {
                            track(n, BASS)
                                .iter()
                                .map(|b| (b.start_step, b.pitch))
                                .min()
                                .map(|b| b.1)
                        })
                    })
                    .ok_or("cible d'approche")?;
                assert!(
                    (1..=2).contains(&note.pitch.abs_diff(next)),
                    "{ctx} : {} n'est ni fondamentale, ni quinte, ni anticipation, ni approche de {next}",
                    note.pitch
                );
                approaches += 1;
            }
        }
    }
    assert!(with_six * 100 >= bars * 95, "attaque au pas 6 sur {with_six} mesures de {bars}");
    assert!(
        with_twelve * 100 >= bars * 70,
        "attaque au pas 12 sur {with_twelve} mesures de {bars}"
    );
    assert!(
        anticipated * 100 >= bars * 55,
        "anticipation au pas 14 sur {anticipated} mesures de {bars}"
    );
    assert!(approaches > 20, "{approaches} notes de passage sur 16 graines");
    Ok(())
}

#[test]
fn le_comping_ne_joue_que_des_notes_de_l_accord_publie_dans_la_tessiture_de_la_variante()
-> TestResult {
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        // Registre « grave » de la session : une octave plus bas.
        let window = if drawn(&flow[0], "registre") == "grave" { (36, 72) } else { (48, 84) };
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
                let ctx = format!("graine {seed} mesure {} pas {step}", snap.index);
                let here = context_at(snap, step);
                // Une attaque qui finit sur la barre peut être anticipée : elle joue alors l'accord de la
                // mesure suivante.
                let ahead = (step + duration == 16).then(|| flow.get(index + 1)).flatten();
                let played = pcs(&pitches);
                let fits = |c: &ChordContext| played.iter().all(|pc| c.pitch_classes.contains(pc));
                if !fits(here) {
                    ahead.map(|a| &a.chord_changes[0].context).filter(|c| fits(c)).ok_or_else(
                        || {
                            format!(
                                "{ctx} : {played:?} hors de {} {:?}",
                                here.chord_name, here.pitch_classes
                            )
                        },
                    )?;
                }
                assert!(
                    pitches[0] >= window.0 && pitches[pitches.len() - 1] <= window.1,
                    "{ctx} : tessiture {pitches:?} hors de {window:?}"
                );
                attack_count += 1;
            }
        }
        assert!(attack_count > 100, "graine {seed} : {attack_count} attaques de comping");
    }
    Ok(())
}

#[test]
fn la_batterie_reste_dans_le_kit_les_voix_latines_sont_sur_leurs_pistes_et_la_cloche_suit_l_arc()
-> TestResult {
    for seed in SEEDS {
        let flow = air(seed)?;
        let (mut kick, mut congas) = (0, 0);
        for s in &flow {
            for n in percussion(s) {
                let allowed = if n.track == LOW { &LOW_NOTES[..] } else { &HIGH_NOTES[..] };
                assert!(
                    allowed.contains(&n.pitch),
                    "graine {seed} mesure {} : note {} hors du kit de la piste {}",
                    s.index,
                    n.pitch,
                    n.track
                );
            }
            kick += usize::from(percussion(s).iter().any(|n| n.pitch == 36));
            congas += usize::from(percussion(s).iter().any(|n| [62, 63, 64].contains(&n.pitch)));
            assert!(!drum_steps(s, CLAVES).is_empty(), "mesure {} : pas de clave", s.index);
        }
        assert!(kick * 100 >= flow.len() * 90, "graine {seed} : grosse caisse sur {kick} mesures");
        assert_eq!(congas, flow.len(), "graine {seed} : congas sur chaque mesure");
        // Arc : pas de cloche à l'exposition, cloche de ride au corps, cowbell au sommet.
        let bell_at = |n: usize, note: u8| {
            chorus(&flow, n).iter().filter(|s| !drum_steps(s, note).is_empty()).count()
        };
        let ride_bell_pushes = |n: usize| {
            chorus(&flow, n)
                .iter()
                .filter(|s| {
                    // La cloche de ride du corps frappe les temps 2 et 3 (le kit mesuré n'y joue pas de cloche).
                    let steps = drum_steps(s, RIDE_BELL);
                    steps.contains(&4) && steps.contains(&8)
                })
                .count()
        };
        assert_eq!(
            bell_at(1, COWBELL) + ride_bell_pushes(1),
            0,
            "graine {seed} : cloche à l'exposition"
        );
        for n in [2, 3] {
            assert_eq!(bell_at(n, COWBELL), 0, "graine {seed} chorus {n} : cowbell au corps");
            assert_eq!(
                ride_bell_pushes(n),
                BARS,
                "graine {seed} chorus {n} : cloche de ride au corps"
            );
        }
        assert_eq!(bell_at(4, COWBELL), BARS, "graine {seed} : cowbell au sommet");
        assert_eq!(ride_bell_pushes(4), 0, "graine {seed} : plus de cloche de ride au sommet");
        assert_eq!(
            bell_at(5, COWBELL) + ride_bell_pushes(5),
            0,
            "graine {seed} : cloche à la retombée"
        );
    }
    Ok(())
}

#[test]
fn la_relance_ne_touche_pas_la_clave_et_tombe_en_fin_de_chorus() -> TestResult {
    let (mut fills, mut elsewhere) = (0, 0);
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        for s in &flow {
            // Les toms aigus au pas 13 ne viennent que des relances du GMD rhumba (le kit n'en joue pas là).
            if drum_steps(s, TOM_HIGH).contains(&13) {
                if chorus_of(s).1 == BARS - 1 {
                    fills += 1;
                } else {
                    elsewhere += 1;
                }
            }
            // Quelle que soit la relance, la clave de la mesure est entière.
            clave_side(s)?;
        }
    }
    assert!(fills >= 3, "{fills} relances vues sur 16 graines");
    assert_eq!(elsewhere, 0, "une relance hors de la fin de chorus");
    Ok(())
}

#[test]
fn le_feel_est_droit_en_seize_pas() -> TestResult {
    let p = profile()?;
    assert!((p.meter.default_swing_ratio - 0.5).abs() < 1e-6, "feel droit");
    for seed in [1u64, 2] {
        for intensity in [0.0f32, 0.5, 1.0] {
            for s in stream(seed, BARS + 2, intensity, 1)? {
                assert_eq!(s.steps, 16, "seize pas par mesure");
                assert!((s.swing_ratio - 0.5).abs() < 1e-6, "swing {}", s.swing_ratio);
            }
        }
    }
    // Aucune piste de mélodie : le profil joue sous une grille.
    for s in air(1)? {
        assert!(track(&s, 0).is_empty(), "mesure {} : mélodie jouée", s.index);
    }
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
            assert!(!attacks(s, BASS).is_empty(), "mesure {} : la basse joue le un", s.index);
        }
    }
    Ok(())
}

#[test]
fn la_graine_change_le_comping_la_batterie_et_la_basse_et_la_meme_graine_rejoue_pareil()
-> TestResult {
    let a = air(1)?;
    assert_eq!(serde_json::to_string(&a)?, serde_json::to_string(&air(1)?)?);
    let (mut comping, mut drums, mut bass) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        comping.insert(sequence(&flow, |s| {
            track(s, CHORDS)
                .iter()
                .map(|n| format!("{}.{}", n.start_step, n.pitch))
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
    assert_eq!(comping.len(), 16, "{} comping distincts sur 16 graines", comping.len());
    assert_eq!(drums.len(), 16, "{} batteries distinctes sur 16 graines", drums.len());
    assert_eq!(bass.len(), 16, "{} basses distinctes sur 16 graines", bass.len());
    Ok(())
}

/// Un habillage tel que les snapshots du premier chorus le montrent.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Dressing {
    tempo_cluster: usize,
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
fn l_habillage_de_la_graine_change_au_moins_huit_fois_sur_seize_graines() -> TestResult {
    let mut rows = Vec::new();
    for seed in 1..=16u64 {
        let flow = stream(seed, 2 * BARS, 0.5, 1)?;
        let first = &flow[0];
        // Les programmes de la session sont ceux du deuxième chorus : l'arc « orchestre » remplace ceux du
        // premier (balais) et du sommet.
        let programs = chorus(&flow, 2)[0].instruments.ok_or("programmes publiés")?;
        first.instruments.ok_or("programmes publiés dès la première mesure")?;
        rows.push((first.tempo, key_of(first)?, programs, first.session.clone()));
        assert_eq!(first.session.len(), 9, "un tirage par groupe : {:?}", first.session);
    }
    // Grappes de tempo de 8 BPM (le premier membre ouvre la grappe).
    let mut tempos: Vec<f32> = rows.iter().map(|r| r.0).collect();
    tempos.sort_by(f32::total_cmp);
    let mut starts: Vec<f32> = Vec::new();
    for t in tempos {
        if starts.last().is_none_or(|&s| t - s > 8.0) {
            starts.push(t);
        }
    }
    let mut dressings = BTreeSet::new();
    let (mut keys, mut chords, mut basses, mut kits, mut voicings) =
        (BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    for (tempo, key, programs, session) in &rows {
        let name = |group: &str| {
            session.iter().find(|v| v.group == group).map(|v| v.name.clone()).unwrap_or_default()
        };
        let cluster = starts.iter().rposition(|&s| *tempo >= s).unwrap_or(0);
        keys.insert(*key);
        chords.insert(programs.chords_program);
        basses.insert(programs.bass_program);
        kits.insert((programs.percussion_bank, programs.percussion_program));
        voicings.insert(name("voicing"));
        dressings.insert(Dressing {
            tempo_cluster: cluster,
            key: *key,
            chords: programs.chords_program.unwrap_or(0),
            bass: programs.bass_program.unwrap_or(0),
            kit: programs.percussion_program,
            voicing: name("voicing"),
            register: name("registre"),
            energy: name("energie"),
            arc: name("arc"),
        });
    }
    assert!(dressings.len() >= 8, "{} habillages distincts sur 16 graines", dressings.len());
    assert!(starts.len() >= 2, "tempos à ± 8 BPM : {starts:?}");
    assert!(keys.len() >= 4, "{} tonalités : {keys:?}", keys.len());
    assert!(chords.len() >= 3, "{} instruments de comping : {chords:?}", chords.len());
    assert!(basses.len() >= 2, "{} basses : {basses:?}", basses.len());
    assert!(kits.len() >= 2, "{} kits : {kits:?}", kits.len());
    assert!(voicings.len() >= 3, "{} familles de voicings : {voicings:?}", voicings.len());
    Ok(())
}

#[test]
fn l_arc_orchestre_change_les_programmes_par_chorus_et_l_arc_classique_les_garde() -> TestResult {
    let kit_of = |name: &str| match name {
        "standard" => 0,
        "room" => 8,
        "power" => 16,
        _ => 32,
    };
    let chords_of = |name: &str| match name {
        "piano" => 0,
        "piano_electrique" => 4,
        _ => 24,
    };
    let (mut orchestre, mut classique) = (0, 0);
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let programs = |n: usize| chorus(&flow, n)[0].instruments.ok_or("programmes");
        let (kit, comping) =
            (kit_of(&drawn(&flow[0], "kit")), chords_of(&drawn(&flow[0], "comping")));
        if drawn(&flow[0], "arc") == "orchestre" {
            orchestre += 1;
            assert_eq!(
                programs(1)?.percussion_program,
                40,
                "graine {seed} : exposition aux balais"
            );
            assert_eq!(
                programs(2)?.percussion_program,
                kit,
                "graine {seed} : le corps rend le kit de la session"
            );
            assert_eq!(programs(3)?.percussion_program, kit);
            assert_eq!(programs(4)?.chords_program, Some(61), "graine {seed} : cuivres au sommet");
            assert_eq!(programs(4)?.percussion_program, kit);
            assert_eq!(programs(5)?.chords_program, Some(comping), "graine {seed} : retombée");
            assert_eq!(programs(5)?.percussion_program, 40, "graine {seed} : retombée aux balais");
        } else {
            classique += 1;
            for n in 1..=CHORUSES {
                let p = programs(n)?;
                assert_eq!(
                    (p.percussion_program, p.chords_program),
                    (kit, Some(comping)),
                    "graine {seed} chorus {n}"
                );
            }
        }
    }
    assert!(orchestre >= 2 && classique >= 2, "orchestre {orchestre}, classique {classique}");
    Ok(())
}

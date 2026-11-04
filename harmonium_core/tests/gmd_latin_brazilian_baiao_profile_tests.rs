//! B3 (phase 019) : contrat du profil distribué `gmd_latin_brazilian_baiao`, observé dans le flux de
//! `MeasureSnapshot` (principe VI).
//!
//! Les attendus sont écrits en clair, relatifs à la tonique : les quatre grilles par défaut (vamps modaux de
//! seize mesures), la basse en fondamentale-quinte sur les changements d'accord, le comping dans l'accord
//! publié, la zabumba (grosse caisse) aux pas 0, 3, 8, 11, le triangle et l'agogô sur la piste haute, le
//! feel droit, le tempo de la source (GMD : 95, 95, 110 BPM), l'arc sur cinq chorus et l'habillage par graine
//! (`session.toml`). Aucune clave : le baiao n'en a pas.
//!
//! Un chorus = seize mesures, cinq chorus par air (80 mesures). La tonique vient de l'habillage (`position.key`).
use std::collections::BTreeSet;

use harmonium_core::{
    compose::TuneGenerator,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::ChordContext,
    StyleProfile, TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const ID: &str = "gmd_latin_brazilian_baiao";
const CHORDS: u8 = 1;
const BASS: u8 = 2;
const LOW: u8 = 3;
const HIGH: u8 = 4;
const BARS: usize = 16;
const CHORUSES: usize = 5;
const SEEDS: [u64; 4] = [1, 2, 3, 17];
/// Tempo de la source (GMD : fichiers à 95, 95 et 110 BPM) : le profil n'en sort pas.
const BPM: (f32, f32, f32) = (95.0, 95.0, 110.0);
/// Fenêtre de la fondamentale de basse (`low..=high` du profil) et plus haute note jouée (WJazzD `LATIN` : 28..53).
const ROOT_WINDOW: (u8, u8) = (33, 44);
const BASS_CEILING: u8 = 53;
/// Notes du kit : grosse caisse sur `PercussionLow`, le reste sur `PercussionHigh`.
const KICK: u8 = 36;
const HIGH_KIT: [u8; 11] = [37, 38, 42, 43, 44, 50, 51, 67, 68, 80, 81];
const TRIANGLE_MUTE: u8 = 80;
const TRIANGLE_OPEN: u8 = 81;
const AGOGO: [u8; 2] = [67, 68];
/// Pas de la zabumba mesurés par le GMD (grosse caisse, probabilité par pas ≥ 0,94) et pas de ses variantes.
const ZABUMBA: [usize; 4] = [0, 3, 8, 11];
const KICK_STEPS: [usize; 6] = [0, 3, 6, 8, 11, 14];

/// Une boucle de grille : (demi-tons au-dessus de la tonique, durée en pas).
type Loop = &'static [(u8, usize)];

/// Vamps modaux : I7 seize mesures ; I7 / bVII par quatre ; I7 / IV par quatre ; huit mesures de I7, bVII, I7, IV
/// (deux mesures chacun), deux fois.
const LOOPS: [Loop; 4] = [
    &[(0, 256)],
    &[(0, 64), (10, 64), (0, 64), (10, 64)],
    &[(0, 64), (5, 64), (0, 64), (5, 64)],
    &[(0, 32), (10, 32), (0, 32), (5, 32), (0, 32), (10, 32), (0, 32), (5, 32)],
];
/// Tonalités de l'habillage (G, D, C, F, A, Si bémol), toutes mixolydiennes, tempos (nom → plage) et programmes.
const KEYS: [u8; 6] = [7, 2, 0, 5, 9, 10];
const TEMPOS: [(&str, f32, f32); 3] =
    [("pose", 95.0, 98.0), ("intermediaire", 101.0, 104.0), ("allant", 107.0, 110.0)];
/// Variantes de l'habillage → programme General MIDI (GeneralUser GS) : comping, basse, kit (banque 128).
const COMPING_PROGRAMS: [(&str, u8); 3] = [("accordeon", 21), ("guitare_nylon", 24), ("piano", 0)];
const BASS_PROGRAMS: [(&str, u8); 2] = [("contrebasse", 32), ("basse_aux_doigts", 33)];
const KIT_PROGRAMS: [(&str, u8); 4] = [("standard", 0), ("salle", 8), ("jazz", 32), ("balais", 40)];
const GROUPS: [&str; 9] =
    ["tempo", "tonalite", "comping", "basse", "kit", "voicing", "registre", "energie", "arc"];

fn stream(seed: u64, bars: usize, intensity: f32, density: u8) -> TestResult<Vec<MeasureSnapshot>> {
    let p = StyleProfile::named(ID)?;
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
    s.notes.iter().filter(|n| n.track == LOW || n.track == HIGH).collect()
}

fn attacks(s: &MeasureSnapshot, id: u8) -> BTreeSet<usize> {
    track(s, id).iter().map(|n| n.start_step).collect()
}

/// Mesures du chorus `n` (1-based).
fn chorus(flow: &[MeasureSnapshot], n: usize) -> Vec<&MeasureSnapshot> {
    flow[(n - 1) * BARS..n * BARS].iter().collect()
}

/// Nom de la variante tirée pour un groupe de l'habillage.
fn variant<'a>(flow: &'a [MeasureSnapshot], group: &str) -> TestResult<&'a str> {
    flow[0]
        .session
        .iter()
        .find(|v| v.group == group)
        .map(|v| v.name.as_str())
        .ok_or_else(|| format!("groupe d'habillage absent : {group}").into())
}

/// Programme que la variante tirée du groupe donne : (comping, basse, kit).
fn session_programs(flow: &[MeasureSnapshot]) -> TestResult<(u8, u8, u8)> {
    let find = |table: &[(&str, u8)], group: &str| -> TestResult<u8> {
        let name = variant(flow, group)?;
        table
            .iter()
            .find(|t| t.0 == name)
            .map(|t| t.1)
            .ok_or_else(|| format!("variante inconnue : {group} {name}").into())
    };
    Ok((
        find(&COMPING_PROGRAMS, "comping")?,
        find(&BASS_PROGRAMS, "basse")?,
        find(&KIT_PROGRAMS, "kit")?,
    ))
}

/// Tonique de l'air tirée par l'habillage (`position.key`).
fn key(flow: &[MeasureSnapshot]) -> TestResult<u8> {
    Ok(flow[0].position.ok_or("position absente")?.key)
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

/// Les deux dernières mesures d'un chorus (après le premier) peuvent porter un break ou un stop-time de l'arc :
/// la basse et les accords ne gardent alors que le temps 1.
fn broken(s: &MeasureSnapshot) -> bool {
    (s.index - 1) % BARS >= BARS - 2 && s.index > BARS
}

#[test]
fn la_grille_par_defaut_est_un_vamp_modal_ecrit_et_la_graine_en_choisit_plusieurs() -> TestResult {
    let expected: Vec<Vec<Vec<(usize, u8)>>> = LOOPS.iter().map(|l| expected_changes(l)).collect();
    for e in &expected {
        assert_eq!(e.len(), BARS, "une boucle écrite fait {BARS} mesures");
    }
    let (mut seen, mut keys) = (BTreeSet::new(), BTreeSet::new());
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let tonic = key(&flow)?;
        assert!(KEYS.contains(&tonic), "graine {seed} : tonique {tonic} hors de l'habillage");
        let p = flow[0].position.ok_or("position")?;
        assert_eq!(format!("{:?}", p.mode), "Mixolydian", "graine {seed} : mode");
        keys.insert(tonic);
        let first: Vec<Vec<(usize, u8)>> = flow[..BARS].iter().map(|s| changes(tonic, s)).collect();
        let which = expected.iter().position(|e| *e == first).ok_or_else(|| {
            format!("graine {seed} : grille hors des boucles écrites : {first:?}")
        })?;
        seen.insert(which);
        // Les chorus suivants rejouent la même boucle (le tirage est par air).
        for n in 2..=CHORUSES {
            let again: Vec<Vec<(usize, u8)>> =
                chorus(&flow, n).iter().map(|s| changes(tonic, s)).collect();
            assert_eq!(again, first, "graine {seed} chorus {n}");
        }
    }
    assert!(seen.len() >= 3, "16 graines n'ont tiré que {} grilles", seen.len());
    assert!(keys.len() >= 4, "16 graines n'ont tiré que {} tonalités", keys.len());
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
            for note in &bass {
                assert!(
                    (ROOT_WINDOW.0.saturating_sub(3)..=BASS_CEILING).contains(&note.pitch),
                    "{ctx} : hauteur {} hors de la fenêtre de basse",
                    note.pitch
                );
            }
            for change in s.chord_changes.iter().filter(|c| !broken(s) || c.start_step == 0) {
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
fn la_basse_joue_la_fondamentale_sur_le_un_et_le_trois_et_la_quinte_en_contretemps() -> TestResult {
    let (mut approaches, mut fifths, mut counted) = (0, 0, 0);
    for seed in SEEDS {
        let flow = air(seed)?;
        for (index, s) in flow.iter().enumerate() {
            let mut bass = track(s, BASS);
            bass.sort_by_key(|n| n.start_step);
            for (rank, note) in bass.iter().enumerate() {
                let chord = context_at(s, note.start_step);
                let rel = (note.pitch + 12 - chord.bass_pc) % 12;
                if [0, 7].contains(&rel) {
                    continue;
                }
                // Sinon c'est une approche : à un ou deux demi-tons de la note de basse suivante
                // (de la mesure suivante pour la dernière).
                let next = bass.get(rank + 1).map(|n| n.pitch).or_else(|| {
                    flow.get(index + 1).and_then(|n| {
                        track(n, BASS).iter().map(|b| (b.start_step, b.pitch)).min().map(|b| b.1)
                    })
                });
                let next = next.ok_or("cible d'approche")?;
                let distance = note.pitch.abs_diff(next);
                assert!(
                    (1..=2).contains(&distance),
                    "graine {seed} mesure {} pas {} : {} n'est ni fondamentale, ni quinte, ni approche de {next}",
                    s.index,
                    note.start_step,
                    note.pitch
                );
                approaches += 1;
            }
            if broken(s) {
                continue;
            }
            counted += 1;
            // Un et trois : la fondamentale de l'accord qui sonne.
            for step in [0, 8] {
                let n = bass.iter().find(|n| n.start_step == step).ok_or_else(|| {
                    format!("graine {seed} mesure {} : pas de basse au pas {step}", s.index)
                })?;
                assert_eq!(
                    n.pitch % 12,
                    context_at(s, step).bass_pc,
                    "graine {seed} mesure {} pas {step}",
                    s.index
                );
            }
            // Contretemps : la quinte au pas 6.
            fifths +=
                usize::from(bass.iter().any(|n| {
                    n.start_step == 6 && (n.pitch + 12 - context_at(s, 6).bass_pc) % 12 == 7
                }));
        }
    }
    assert!(approaches > 0, "aucune approche sur {} graines", SEEDS.len());
    assert!(fifths * 100 >= counted * 90, "quinte au pas 6 dans {fifths} mesures sur {counted}");
    Ok(())
}

#[test]
fn le_comping_ne_joue_que_des_notes_de_l_accord_publie_dans_la_tessiture_du_registre() -> TestResult
{
    for seed in SEEDS {
        let flow = air(seed)?;
        // Registre normal : fenêtre du profil 50..=80 ; « grave » : une octave plus bas.
        let window = match variant(&flow, "registre")? {
            "normal" => (50u8, 80u8),
            "grave" => (38, 68),
            other => return Err(format!("registre inconnu : {other}").into()),
        };
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
fn la_batterie_joue_la_zabumba_le_triangle_et_l_agogo_sur_les_bonnes_pistes() -> TestResult {
    let mut agogo = 0;
    for seed in SEEDS {
        let flow = air(seed)?;
        let (mut zabumba, mut counted) = (0, 0);
        for s in &flow {
            let ctx = format!("graine {seed} mesure {}", s.index);
            for n in percussion(s) {
                let on_low = n.track == LOW;
                assert_eq!(
                    on_low,
                    n.pitch == KICK,
                    "{ctx} : note {} sur la piste {}",
                    n.pitch,
                    n.track
                );
                assert!(
                    on_low || HIGH_KIT.contains(&n.pitch),
                    "{ctx} : note hors du kit : {}",
                    n.pitch
                );
                if on_low {
                    assert!(
                        KICK_STEPS.contains(&n.start_step),
                        "{ctx} : grosse caisse au pas {}",
                        n.start_step
                    );
                }
                // Triangle : muet sur les temps, ouvert sur les contretemps ; agogô : sur la piste haute.
                match n.pitch {
                    TRIANGLE_MUTE => assert_eq!(
                        n.start_step % 4,
                        0,
                        "{ctx} : triangle muet au pas {}",
                        n.start_step
                    ),
                    TRIANGLE_OPEN => assert_eq!(
                        n.start_step % 4,
                        2,
                        "{ctx} : triangle ouvert au pas {}",
                        n.start_step
                    ),
                    p if AGOGO.contains(&p) => agogo += 1,
                    _ => {}
                }
            }
            counted += 1;
            let kicks = attacks_of(s, KICK);
            zabumba += usize::from(ZABUMBA.iter().all(|p| kicks.contains(p)));
        }
        // La zabumba (0, 3, 8, 11) sonne dans presque toutes les mesures : seules les relances la remplacent.
        assert!(
            zabumba * 100 >= counted * 95,
            "graine {seed} : zabumba complète dans {zabumba} mesures sur {counted}"
        );
        // Le triangle joue dans toutes les mesures sauf les relances (une mesure de caisse claire en doubles croches
        // avec la zabumba, tirée du GMD) : une mesure sans triangle est une relance.
        let mut without = 0;
        for s in &flow {
            if !percussion(s).iter().any(|n| n.pitch == TRIANGLE_MUTE) {
                let snare = percussion(s).iter().filter(|n| n.pitch == 38).count();
                assert!(
                    snare >= 8,
                    "graine {seed} mesure {} : ni triangle ni relance ({snare} caisses claires)",
                    s.index
                );
                without += 1;
            }
        }
        assert!(
            without * 100 <= counted * 25,
            "graine {seed} : {without} relances sur {counted} mesures"
        );
    }
    assert!(agogo > 0, "aucun agogô sur {} graines", SEEDS.len());
    Ok(())
}

fn attacks_of(s: &MeasureSnapshot, pitch: u8) -> BTreeSet<usize> {
    percussion(s).iter().filter(|n| n.pitch == pitch).map(|n| n.start_step).collect()
}

#[test]
fn le_feel_est_droit_en_seize_pas_et_le_tempo_reste_dans_la_source_et_dans_la_variante(
) -> TestResult {
    let p = StyleProfile::named(ID)?;
    assert_eq!((p.meter.min_bpm, p.meter.default_bpm, p.meter.max_bpm), BPM);
    assert!((p.meter.default_swing_ratio - 0.5).abs() < 1e-6, "feel droit");
    let mut ranges = BTreeSet::new();
    for seed in 1..=16u64 {
        for intensity in [0.0f32, 0.5, 1.0] {
            let flow = stream(seed, BARS + 2, intensity, 1)?;
            let name = variant(&flow, "tempo")?;
            let &(_, low, high) = TEMPOS
                .iter()
                .find(|t| t.0 == name)
                .ok_or_else(|| format!("tempo inconnu : {name}"))?;
            ranges.insert(name.to_string());
            for s in &flow {
                assert_eq!(s.steps, 16, "seize pas par mesure");
                assert!(
                    (BPM.0..=BPM.2).contains(&s.tempo) && (low..=high).contains(&s.tempo),
                    "graine {seed} intensité {intensity} : tempo {} hors de la variante {name} ({low}..{high}) ou de la source",
                    s.tempo
                );
                assert!((s.swing_ratio - 0.5).abs() < 1e-6, "swing {}", s.swing_ratio);
            }
        }
    }
    assert_eq!(ranges.len(), TEMPOS.len(), "16 graines n'ont tiré que {ranges:?}");
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
fn la_graine_change_le_comping_la_batterie_et_la_basse_et_la_meme_graine_rejoue_pareil(
) -> TestResult {
    let a = air(1)?;
    let again = air(1)?;
    assert_eq!(serde_json::to_string(&a)?, serde_json::to_string(&again)?);
    let (mut comping, mut drums, mut bass) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
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

#[test]
fn l_habillage_par_graine_donne_au_moins_huit_variantes_distinctes_sur_seize_graines() -> TestResult
{
    let mut distinct = BTreeSet::new();
    let (mut keys, mut kits, mut voicings, mut comping, mut basses, mut tempos) = (
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::new(),
    );
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let first = &flow[0];
        // Un tirage par groupe, dans l'ordre du fichier, les mêmes sur toutes les mesures.
        let groups: Vec<&str> = first.session.iter().map(|v| v.group.as_str()).collect();
        assert_eq!(groups, GROUPS, "graine {seed}");
        assert!(
            flow.iter().all(|s| s.session == first.session),
            "graine {seed} : habillage changeant"
        );
        let inst = first.instruments.ok_or("instruments non publiés")?;
        // Les programmes qui sonnent à la première mesure sont ceux des variantes tirées, sauf le kit à l'exposition
        // de l'arc « orchestre » (balais) : voir le test de l'arc.
        let (chords_program, bass_program, kit) = session_programs(&flow)?;
        assert_eq!(inst.chords_program, Some(chords_program), "graine {seed} comping");
        assert_eq!(inst.bass_program, Some(bass_program), "graine {seed} basse");
        assert_eq!(inst.percussion_bank, 128, "kit GM, sans Odin");
        let expected_kit = if variant(&flow, "arc")? == "orchestre" { 40 } else { kit };
        assert_eq!(inst.percussion_program, expected_kit, "graine {seed} kit");
        keys.insert(key(&flow)?);
        kits.insert(kit);
        comping.insert(chords_program);
        basses.insert(bass_program);
        voicings.insert(variant(&flow, "voicing")?.to_string());
        tempos.insert(variant(&flow, "tempo")?.to_string());
        distinct.insert(
            first
                .session
                .iter()
                .map(|v| format!("{}={}", v.group, v.name))
                .collect::<Vec<_>>()
                .join("/"),
        );
    }
    assert!(distinct.len() >= 8, "{} habillages distincts sur 16 graines", distinct.len());
    assert!(keys.len() >= 4, "{} tonalités ({keys:?})", keys.len());
    assert!(comping.len() >= 3, "{} programmes de comping", comping.len());
    assert!(basses.len() >= 2, "{} basses", basses.len());
    assert!(kits.len() >= 2, "{} kits ({kits:?})", kits.len());
    assert!(voicings.len() >= 3, "{} familles de voicings ({voicings:?})", voicings.len());
    assert!(tempos.len() >= 2, "{} plages de tempo ({tempos:?})", tempos.len());
    Ok(())
}

#[test]
fn l_arc_orchestre_change_le_kit_a_l_exposition_et_le_comping_au_sommet() -> TestResult {
    let mut orchestrated = 0;
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let (chords, bass, kit) = session_programs(&flow)?;
        let program = |n: usize| flow[(n - 1) * BARS].instruments.ok_or("instruments non publiés");
        // Programmes de la session (comping, basse, kit) ; l'arc « orchestre » en remplace deux, le temps d'un chorus.
        let mut expected = [(chords, kit); CHORUSES];
        if variant(&flow, "arc")? == "orchestre" {
            orchestrated += 1;
            // Exposition : balais (40) ; sommet (chorus 4) : accordéon (21) ; les autres chorus rendent les programmes
            // de la session (le changement se défait), la retombée aussi.
            expected[0].1 = 40;
            expected[3].0 = 21;
        }
        for (n, &(chords, kit)) in expected.iter().enumerate() {
            let heard = program(n + 1)?;
            assert_eq!(
                (heard.chords_program, heard.bass_program, heard.percussion_program),
                (Some(chords), Some(bass), kit),
                "graine {seed} chorus {}",
                n + 1
            );
        }
    }
    assert!(orchestrated >= 4, "{orchestrated} graines à l'arc « orchestre » sur 16");
    Ok(())
}

/// Une mesure de `caixa` : la caisse claire en doubles croches tient le temps (au moins dix coups), sous le triangle et
/// la charleston à la pédale ; une relance a la même caisse claire mais ni triangle ni pédale.
fn caixa(s: &MeasureSnapshot) -> bool {
    let kit = percussion(s);
    kit.iter().filter(|n| n.pitch == 38).count() >= 10
        && kit.iter().any(|n| n.pitch == TRIANGLE_MUTE)
        && kit.iter().any(|n| n.pitch == 44)
}

#[test]
fn la_caixa_ne_joue_que_dans_les_chorus_ou_l_arc_la_nomme() -> TestResult {
    let mut per_chorus = [0usize; CHORUSES];
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        for n in 1..=CHORUSES {
            per_chorus[n - 1] += chorus(&flow, n).iter().filter(|s| caixa(s)).count();
        }
    }
    // L'exposition (chorus 1) et la retombée (chorus 5) jouent à la charleston : jamais la caixa.
    assert_eq!(per_chorus[0], 0, "caixa à l'exposition : {per_chorus:?}");
    assert_eq!(per_chorus[CHORUSES - 1], 0, "caixa à la retombée : {per_chorus:?}");
    // Le corps (chorus 2 et 3, une variante sur deux) et le sommet (chorus 4) la tirent.
    assert!(
        per_chorus[1..CHORUSES - 1].iter().all(|&c| c > 0),
        "caixa absente d'un chorus : {per_chorus:?}"
    );
    Ok(())
}

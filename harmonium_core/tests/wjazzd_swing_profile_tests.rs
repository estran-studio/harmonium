//! E7 puis B1 (phase 019) : contrat du profil distribué `wjazzd_swing`, observé dans le flux de
//! `MeasureSnapshot` (principe VI). Les attendus sont écrits en clair, relatifs à la tonique du
//! snapshot : les grilles par défaut (quatre blues, deux A, deux B), la basse (arrivées dans l'accord
//! publié, fenêtre), le comping dans l'accord publié, la batterie couplée au kit, l'habillage par
//! graine (tempo, tonalité, programmes, voicings), le rapport de swing selon le tempo et l'arc sur
//! quatre chorus, sous la grille par défaut et sous une grille d'hôte de huit mesures.
//!
//! Chorus par défaut : `Blues A A B A`, 12 + 8 + 8 + 8 + 8 = 44 mesures ; quatre chorus (176 mesures).
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
const RIDE: u8 = 51;
const SWIRL: u8 = 40;
const CHORUS: usize = 44;
const CHORUSES: usize = 4;
/// Plages de tempo des variantes (`session.toml`, groupe `rythme`), en noires par minute.
const TEMPO_RANGES: [(f32, f32); 5] =
    [(100.0, 118.0), (122.0, 142.0), (148.0, 186.0), (192.0, 225.0), (232.0, 265.0)];

fn profile() -> TestResult<StyleProfile> {
    Ok(StyleProfile::named("wjazzd_swing")?)
}

fn stream_of(p: &StyleProfile, seed: u64, bars: usize) -> TestResult<Vec<MeasureSnapshot>> {
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

fn stream(seed: u64, bars: usize) -> TestResult<Vec<MeasureSnapshot>> {
    stream_of(&profile()?, seed, bars)
}

fn track(s: &MeasureSnapshot, id: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == id).collect()
}

fn percussion(s: &MeasureSnapshot) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| PERCUSSION.contains(&n.track)).collect()
}

fn key_of(s: &MeasureSnapshot) -> u8 {
    s.position.map_or(0, |p| p.key)
}

fn chorus_of(s: &MeasureSnapshot) -> usize {
    s.position.map_or(0, |p| p.chorus())
}

fn drawn(s: &MeasureSnapshot, group: &str) -> String {
    s.session.iter().find(|v| v.group == group).map(|v| v.name.clone()).unwrap_or_default()
}

/// Fondamentale d'un accord publié, relative à la tonique de la mesure.
fn degree(s: &MeasureSnapshot, c: &ChordContext) -> u8 {
    (c.root_pc + 12 - key_of(s)) % 12
}

fn context_at(s: &MeasureSnapshot, step: usize) -> &ChordContext {
    &s.chord_changes[s.chord_changes.partition_point(|c| c.start_step <= step) - 1].context
}

/// Fondamentales relatives (une par changement d'accord de la mesure : pas, degré).
fn changes(s: &MeasureSnapshot) -> Vec<(usize, u8)> {
    s.chord_changes.iter().map(|c| (c.start_step, degree(s, &c.context))).collect()
}

/// Premier snapshot d'une graine dont le groupe `rythme` porte `name`, parmi les graines 1..=200.
fn seed_with(rhythm: &str) -> TestResult<u64> {
    let p = profile()?;
    for seed in 1..=200 {
        if drawn(&stream_of(&p, seed, 1)?[0], "rythme") == rhythm {
            return Ok(seed);
        }
    }
    Err(format!("aucune graine de 1 à 200 ne tire {rhythm}").into())
}

// ---------------------------------------------------------------------------
// Grilles par défaut
// ---------------------------------------------------------------------------

type Bar = &'static [(usize, u8)];

/// Les quatre blues de douze mesures (simple, changement rapide, de jazz, sous-dominante mineure).
const BLUES: [[Bar; 12]; 4] = [
    [
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
    ],
    [
        &[(0, 0)],
        &[(0, 5)],
        &[(0, 0)],
        &[(0, 0)],
        &[(0, 5)],
        &[(0, 5)],
        &[(0, 0)],
        &[(0, 9)],
        &[(0, 2)],
        &[(0, 7)],
        &[(0, 0)],
        &[(0, 0), (8, 7)],
    ],
    [
        &[(0, 0)],
        &[(0, 5)],
        &[(0, 0)],
        &[(0, 7), (8, 0)],
        &[(0, 5)],
        &[(0, 6)],
        &[(0, 0)],
        &[(0, 4), (8, 9)],
        &[(0, 2)],
        &[(0, 7)],
        &[(0, 0), (8, 9)],
        &[(0, 2), (8, 7)],
    ],
    [
        &[(0, 0)],
        &[(0, 5)],
        &[(0, 0)],
        &[(0, 0)],
        &[(0, 5)],
        &[(0, 5), (8, 10)],
        &[(0, 0)],
        &[(0, 9)],
        &[(0, 2)],
        &[(0, 7)],
        &[(0, 0)],
        &[(0, 0), (8, 7)],
    ],
];
/// Les deux versions de A (*rhythm changes*), huit mesures.
const A: [[Bar; 8]; 2] = [
    [
        &[(0, 0), (8, 9)],
        &[(0, 2), (8, 7)],
        &[(0, 4), (8, 9)],
        &[(0, 2), (8, 7)],
        &[(0, 7), (8, 0)],
        &[(0, 5), (8, 5)],
        &[(0, 0), (8, 9)],
        &[(0, 2), (8, 7)],
    ],
    [
        &[(0, 0), (8, 9)],
        &[(0, 2), (8, 7)],
        &[(0, 0), (8, 9)],
        &[(0, 2), (8, 7)],
        &[(0, 7), (8, 0)],
        &[(0, 5), (8, 5)],
        &[(0, 0), (8, 9)],
        &[(0, 2), (8, 7)],
    ],
];
/// Les deux versions de B : cycle des dominantes, puis ii-V une mesure par accord.
const B: [[Bar; 8]; 2] = [
    [&[(0, 4)], &[(0, 4)], &[(0, 9)], &[(0, 9)], &[(0, 2)], &[(0, 2)], &[(0, 7)], &[(0, 7)]],
    [
        &[(0, 11), (8, 4)],
        &[(0, 11), (8, 4)],
        &[(0, 4), (8, 9)],
        &[(0, 4), (8, 9)],
        &[(0, 9), (8, 2)],
        &[(0, 9), (8, 2)],
        &[(0, 2), (8, 7)],
        &[(0, 2), (8, 7)],
    ],
];

/// Indice de l'alternative que `bars` joue, `None` si aucune ne correspond.
fn matching<const N: usize>(bars: &[MeasureSnapshot], options: &[[Bar; N]]) -> Option<usize> {
    options.iter().position(|o| bars.iter().zip(o).all(|(s, expected)| changes(s) == *expected))
}

#[test]
fn la_grille_par_defaut_est_un_blues_puis_l_aaba_dont_les_variantes_se_tirent_par_graine()
-> TestResult {
    let mut blues_seen = BTreeSet::new();
    let mut a_seen = BTreeSet::new();
    let mut b_seen = BTreeSet::new();
    for seed in 1..=16 {
        let flow = stream(seed, CHORUS)?;
        let ctx = |part: &str| format!("graine {seed} : {part} hors des grilles écrites");
        let blues = matching(&flow[..12], &BLUES).ok_or_else(|| ctx("blues"))?;
        // Les trois A du chorus rejouent la même version, le B la sienne.
        let a = matching(&flow[12..20], &A).ok_or_else(|| ctx("A"))?;
        assert_eq!(matching(&flow[20..28], &A), Some(a), "graine {seed} : deuxième A");
        assert_eq!(matching(&flow[36..44], &A), Some(a), "graine {seed} : dernier A");
        let b = matching(&flow[28..36], &B).ok_or_else(|| ctx("B"))?;
        blues_seen.insert(blues);
        a_seen.insert(a);
        b_seen.insert(b);
    }
    assert!(blues_seen.len() >= 3, "blues tirés sur 16 graines : {blues_seen:?}");
    assert_eq!(a_seen.len(), 2, "versions de A : {a_seen:?}");
    assert_eq!(b_seen.len(), 2, "versions de B : {b_seen:?}");
    Ok(())
}

// ---------------------------------------------------------------------------
// Basse, comping, batterie
// ---------------------------------------------------------------------------

#[test]
fn la_basse_arrive_dans_l_accord_publie_et_reste_dans_la_fenetre_des_solos() -> TestResult {
    for seed in [1, 2, 5, 9] {
        let flow = stream(seed, CHORUSES * CHORUS)?;
        let mut arrivals = 0;
        for (index, snap) in flow.iter().enumerate() {
            let ctx = format!("graine {seed} mesure {}", index + 1);
            for note in track(snap, BASS) {
                assert!(
                    (33..=50).contains(&note.pitch),
                    "{ctx} : hauteur {} hors de 33..50",
                    note.pitch
                );
                let arrival = note.start_step == 0
                    || snap.chord_changes.iter().any(|c| c.start_step == note.start_step);
                if arrival {
                    let chord = context_at(snap, note.start_step);
                    assert!(
                        chord.pitch_classes.contains(&(note.pitch % 12)),
                        "{ctx} pas {} : arrivée {} hors de {}",
                        note.start_step,
                        note.pitch % 12,
                        chord.chord_name
                    );
                    assert!(
                        (36..=47).contains(&note.pitch),
                        "{ctx} : arrivée {} hors de 36..47",
                        note.pitch
                    );
                    arrivals += 1;
                }
            }
        }
        assert!(arrivals > 200, "graine {seed} : {arrivals} arrivées seulement");
    }
    Ok(())
}

#[test]
fn l_accord_slash_de_la_forme_aaba_est_joue_sur_sa_basse() -> TestResult {
    // Mesure 19 du chorus (A, septième mesure) : I6 sur la quinte, puis VI7. Premier chorus : l'exposition
    // et les chorus sans coupure jouent leur premier temps sur la basse publiée.
    for seed in [1, 2, 3] {
        let flow = stream(seed, CHORUS)?;
        let bar = &flow[12 + 6];
        let slash = bar.chord_changes[0].context.bass_pc;
        assert_eq!(slash, (key_of(bar) + 7) % 12, "graine {seed} : basse publiée = la quinte");
        let first = track(bar, BASS).into_iter().find(|n| n.start_step == 0).map(|n| n.pitch % 12);
        assert_eq!(first, Some(slash), "graine {seed}");
    }
    Ok(())
}

#[test]
fn le_comping_reste_dans_l_accord_publie_et_dans_la_tessiture() -> TestResult {
    for seed in 1..=8 {
        let flow = stream(seed, CHORUSES * CHORUS)?;
        let (mut inside, mut total, mut attacks) = (0, 0, 0);
        for (bar, snap) in flow.iter().enumerate() {
            for n in track(snap, CHORDS) {
                // Une attaque anticipée (pas 14, durée 2) joue l'accord de la mesure suivante.
                let chord = if n.start_step == 14 && n.duration_steps == 2 && bar + 1 < flow.len() {
                    &flow[bar + 1].chord_changes[0].context
                } else {
                    context_at(snap, n.start_step)
                };
                total += 1;
                inside += usize::from(chord.pitch_classes.contains(&(n.pitch % 12)));
                assert!((36..=96).contains(&n.pitch), "graine {seed} mesure {} : {n:?}", bar + 1);
            }
            attacks += usize::from(!track(snap, CHORDS).is_empty());
        }
        // Les tensions de couleur (9, 11, 13) sortent de l'accord publié (jusqu'à 48 % des notes en quartes, A5) ;
        // un comping sur le mauvais accord n'en garderait que le quart.
        assert!(
            inside * 100 >= total * 50,
            "graine {seed} ({}) : {inside}/{total} notes dans l'accord",
            drawn(&flow[0], "voicing")
        );
        assert!(attacks > 120, "graine {seed} : {attacks} mesures avec accords seulement");
    }
    Ok(())
}

#[test]
fn le_kit_et_les_patterns_vont_ensemble_balais_et_baguettes() -> TestResult {
    // `lent_balais_exposition` : exposition aux balais (balayage, jamais de ride), baguettes au sommet.
    let seed = seed_with("lent_balais_exposition")?;
    let flow = stream(seed, CHORUSES * CHORUS)?;
    let pitches = |chorus: usize| -> BTreeSet<u8> {
        flow.iter()
            .filter(|s| chorus_of(s) == chorus)
            .flat_map(|s| percussion(s).into_iter().map(|n| n.pitch))
            .collect()
    };
    let kit = |chorus: usize| {
        flow.iter()
            .find(|s| chorus_of(s) == chorus)
            .and_then(|s| s.instruments)
            .map(|i| i.percussion_program)
    };
    assert_eq!(kit(1), Some(40), "exposition aux balais");
    assert!(pitches(1).contains(&SWIRL) && !pitches(1).contains(&RIDE), "{:?}", pitches(1));
    assert_eq!(kit(3), Some(32), "sommet aux baguettes");
    assert!(pitches(3).contains(&RIDE), "{:?}", pitches(3));
    assert_eq!(kit(4), Some(40), "retombée : le kit de la session revient");
    // Aux baguettes, aucun balayage.
    let seed = seed_with("medium_baguettes_exposition")?;
    let flow = stream(seed, CHORUSES * CHORUS)?;
    assert!(flow.iter().flat_map(percussion).all(|n| n.pitch != SWIRL));
    Ok(())
}

// ---------------------------------------------------------------------------
// Habillage par graine
// ---------------------------------------------------------------------------

#[test]
fn seize_graines_donnent_au_moins_douze_habillages_distincts_dans_les_bornes_du_style() -> TestResult
{
    let p = profile()?;
    let mut dressings = BTreeSet::new();
    let (mut keys, mut chords, mut kits, mut voicings, mut clusters) =
        (BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    for seed in 1..=16 {
        let first = stream_of(&p, seed, 1)?.remove(0);
        let nominal = first.composition_bpm;
        assert!(
            TEMPO_RANGES.iter().any(|(lo, hi)| (*lo..=*hi).contains(&nominal)),
            "graine {seed} : tempo {nominal} hors des plages du profil"
        );
        let instruments = first.instruments.ok_or("programmes publiés")?;
        let cluster = TEMPO_RANGES.iter().position(|(lo, hi)| (*lo..=*hi).contains(&nominal));
        let dressing = (
            (nominal / 16.0) as u32,
            key_of(&first),
            instruments.chords_program,
            instruments.bass_program,
            instruments.percussion_program,
            drawn(&first, "voicing"),
        );
        keys.insert(key_of(&first));
        chords.insert(instruments.chords_program);
        kits.insert(instruments.percussion_program);
        voicings.insert(drawn(&first, "voicing"));
        clusters.insert(cluster);
        dressings.insert(dressing);
        assert_eq!(first.session.len(), 7, "un tirage par groupe");
    }
    assert!(dressings.len() >= 12, "{} habillages distincts sur 16 graines", dressings.len());
    assert!(keys.len() >= 5, "tonalités : {keys:?}");
    assert!(chords.len() >= 3, "comping : {chords:?}");
    assert!(kits.is_subset(&BTreeSet::from([32, 40])), "kits : {kits:?}");
    assert!(voicings.len() >= 4, "voicings : {voicings:?}");
    assert!(clusters.len() >= 4, "classes de tempo : {clusters:?}");
    Ok(())
}

#[test]
fn les_balais_sont_plus_probables_aux_tempos_lents_et_absents_au_dela_de_cent_quarante_deux()
-> TestResult {
    let p = profile()?;
    let (mut slow, mut slow_brush, mut fast_brush) = (0, 0, 0);
    for seed in 1..=200 {
        let first = stream_of(&p, seed, 1)?.remove(0);
        let brush = drawn(&first, "rythme").contains("balais");
        if first.composition_bpm < 120.0 {
            slow += 1;
            slow_brush += usize::from(brush);
        } else if first.composition_bpm > 142.0 {
            fast_brush += usize::from(brush);
        }
    }
    assert!(slow > 15, "{slow} graines lentes seulement");
    assert!(slow_brush * 10 >= slow * 4, "balais aux tempos lents : {slow_brush}/{slow}");
    assert_eq!(fast_brush, 0, "des balais au-delà de 142 BPM");
    Ok(())
}

#[test]
fn le_rapport_de_swing_suit_le_tempo_et_la_courbe_de_la_batterie() -> TestResult {
    let p = profile()?;
    let mut by_tempo: Vec<(f32, f32)> = Vec::new();
    for seed in 1..=40 {
        let first = stream_of(&p, seed, 1)?.remove(0);
        // Batterie JTD : plateau 0,7213 jusqu'à 140 BPM, puis −0,0976 par 100 BPM.
        let expected = 0.7213 - 0.000_976 * (first.tempo - 140.0).max(0.0);
        assert!(
            (first.swing_ratio - expected).abs() < 0.002,
            "graine {seed} : swing {} à {} BPM, attendu {expected}",
            first.swing_ratio,
            first.tempo
        );
        by_tempo.push((first.tempo, first.swing_ratio));
    }
    by_tempo.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert!(
        by_tempo.windows(2).all(|w| w[1].1 <= w[0].1 + 1e-6),
        "swing non décroissant : {by_tempo:?}"
    );
    let (slowest, fastest) = (by_tempo[0], by_tempo[by_tempo.len() - 1]);
    assert!(slowest.1 - fastest.1 > 0.05, "{slowest:?} contre {fastest:?}");
    Ok(())
}

// ---------------------------------------------------------------------------
// Arc sur quatre chorus
// ---------------------------------------------------------------------------

fn mean_notes(flow: &[MeasureSnapshot], chorus: usize, id: u8) -> f64 {
    let bars: Vec<_> = flow.iter().filter(|s| chorus_of(s) == chorus).collect();
    bars.iter().map(|s| track(s, id).len()).sum::<usize>() as f64 / bars.len() as f64
}

#[test]
fn l_arc_passe_du_deux_temps_a_la_marche_puis_au_sommet_et_retombe_en_stop_time() -> TestResult {
    let seed = seed_with("medium_baguettes_exposition")?;
    let flow = stream(seed, CHORUSES * CHORUS)?;
    let bass_steps = |s: &MeasureSnapshot| -> BTreeSet<usize> {
        track(s, BASS).iter().map(|n| n.start_step).collect()
    };
    // Chorus 1 : deux temps (attaques aux temps 1 et 3, plus les changements d'accord).
    for s in flow.iter().filter(|s| chorus_of(s) == 1) {
        let allowed: BTreeSet<usize> =
            [0, 8].into_iter().chain(s.chord_changes.iter().map(|c| c.start_step)).collect();
        assert!(bass_steps(s).is_subset(&allowed), "mesure {} : {:?}", s.index, bass_steps(s));
    }
    // Chorus 2 : la marche (une note par temps dans la grande majorité des mesures).
    let walking = flow
        .iter()
        .filter(|s| chorus_of(s) == 2 && [0, 4, 8, 12].iter().all(|st| bass_steps(s).contains(st)))
        .count();
    assert!(walking * 10 >= CHORUS * 8, "marche sur {walking}/{CHORUS} mesures");
    // Le comping s'étoffe au sommet (chorus 3) puis retombe (chorus 4).
    let (c1, c3, c4) =
        (mean_notes(&flow, 1, CHORDS), mean_notes(&flow, 3, CHORDS), mean_notes(&flow, 4, CHORDS));
    assert!(
        c3 > c1 * 1.5 && c4 < c3 * 0.6,
        "notes d'accords par mesure : {c1:.1} / {c3:.1} / {c4:.1}"
    );
    // Retombée : deux temps, et stop-time à l'avant-dernière mesure (accords et basse sur le premier temps).
    let last = &flow[CHORUSES * CHORUS - 2];
    for id in [CHORDS, BASS] {
        assert!(
            track(last, id).iter().all(|n| n.start_step == 0),
            "piste {id} : {:?}",
            track(last, id)
        );
    }
    assert!(!track(last, BASS).is_empty() || !track(last, CHORDS).is_empty());
    Ok(())
}

#[test]
fn le_kit_change_a_la_frontiere_des_chorus_de_l_arc_orchestre() -> TestResult {
    // `lent_orchestre` : balais en exposition, baguettes au corps, retour aux balais à la retombée.
    let seed = seed_with("lent_orchestre")?;
    let flow = stream(seed, CHORUSES * CHORUS)?;
    let kits: Vec<_> = (1..=CHORUSES)
        .map(|c| {
            flow.iter()
                .find(|s| chorus_of(s) == c)
                .and_then(|s| s.instruments)
                .map(|i| i.percussion_program)
        })
        .collect();
    assert_eq!(kits, [Some(40), Some(32), Some(32), Some(40)]);
    Ok(())
}

// ---------------------------------------------------------------------------
// Sous une grille d'hôte
// ---------------------------------------------------------------------------

/// Le profil sous une grille de huit mesures (une partie répétée), comme le fait un hôte.
fn under_host_grid(fixed_key: Option<u8>) -> TestResult<StyleProfile> {
    let p = profile()?;
    let mut value: toml::Value = toml::from_str(&p.to_toml()?)?;
    let table = value.as_table_mut().ok_or("profil")?;
    table.insert(
        "form".into(),
        toml::from_str(
            r#"sequence = [0]
[[parts]]
name = "Grille"
melody_center_offset = 0
phrase_ends = [8]
roles = ["Open", "Free", "Free", "Free", "Free", "Free", "Free", "Free"]"#,
        )?,
    );
    table.insert(
        "harmony".into(),
        toml::from_str(
            r#"model = "loop"
steps_per_chord = 4
[parts.Grille]
kind = "chord_loop"
loops = [{ weight = 1, chords = [
  { semitones = 0, quality = "6", steps = 16 }, { semitones = 9, quality = "7", steps = 16 },
  { semitones = 2, quality = "m7", steps = 16 }, { semitones = 7, quality = "7", steps = 16 },
  { semitones = 4, quality = "m7", steps = 16 }, { semitones = 9, quality = "7", steps = 16 },
  { semitones = 2, quality = "m7", steps = 8 }, { semitones = 7, quality = "7", steps = 8 },
  { semitones = 0, quality = "6", steps = 16 },
] }]"#,
        )?,
    );
    if let Some(key) = fixed_key {
        table.insert(
            "fixed_tonality".into(),
            toml::from_str(&format!("key = {key}\nmode = \"ionian\""))?,
        );
    }
    Ok(StyleProfile::from_toml(&toml::to_string(&value)?)?)
}

#[test]
fn sous_une_grille_de_huit_mesures_la_graine_habille_et_la_grille_ne_bouge_pas() -> TestResult {
    let p = under_host_grid(None)?;
    let grid: [Bar; 8] = [
        &[(0, 0)],
        &[(0, 9)],
        &[(0, 2)],
        &[(0, 7)],
        &[(0, 4)],
        &[(0, 9)],
        &[(0, 2), (8, 7)],
        &[(0, 0)],
    ];
    let mut keys = BTreeSet::new();
    let mut tempos = BTreeSet::new();
    for seed in 1..=8 {
        let flow = stream_of(&p, seed, 4 * 8)?;
        for (index, s) in flow.iter().enumerate() {
            assert_eq!(
                changes(s),
                grid[index % 8],
                "graine {seed} mesure {} : la grille de l'hôte bouge",
                index + 1
            );
        }
        keys.insert(key_of(&flow[0]));
        tempos.insert(flow[0].composition_bpm as u32 / 16);
        // L'arc joue sur des chorus de huit mesures : le comping change d'un chorus au suivant.
        assert!(
            (1..=3)
                .any(|c| (mean_notes(&flow, c, CHORDS) - mean_notes(&flow, c + 1, CHORDS)).abs()
                    > 0.5),
            "graine {seed} : aucun contraste de chorus"
        );
    }
    assert!(keys.len() >= 4 && tempos.len() >= 4, "tonalités {keys:?} ; tempos {tempos:?}");
    Ok(())
}

#[test]
fn la_tonalite_de_l_hote_gagne_sur_celle_de_la_graine_et_le_reste_de_la_variante_tient()
-> TestResult {
    let p = under_host_grid(Some(9))?;
    let free = under_host_grid(None)?;
    for seed in 1..=6 {
        let flow = stream_of(&p, seed, 16)?;
        assert!(flow.iter().all(|s| key_of(s) == 9), "graine {seed}");
        assert!(flow.iter().step_by(8).all(|s| s.chord_root_pc == 9));
        let loose = stream_of(&free, seed, 16)?;
        assert_eq!(flow[0].session, loose[0].session, "graine {seed} : la variante reste tirée");
        assert_eq!(flow[0].instruments, loose[0].instruments);
        assert_eq!(flow[0].composition_bpm, loose[0].composition_bpm);
    }
    Ok(())
}

#[test]
fn le_profil_rejoue_la_meme_graine_a_l_identique_et_varie_d_une_graine_a_l_autre() -> TestResult {
    let a = stream(1, CHORUS)?;
    let again = stream(1, CHORUS)?;
    assert_eq!(serde_json::to_string(&a)?, serde_json::to_string(&again)?);
    let bass_line = |flow: &[MeasureSnapshot]| -> Vec<u8> {
        flow.iter().flat_map(|s| track(s, BASS).into_iter().map(|n| n.pitch)).collect()
    };
    assert_ne!(
        bass_line(&a),
        bass_line(&stream(2, CHORUS)?),
        "la marche ne dépend pas de la graine"
    );
    Ok(())
}

/// Piste de la mélodie dans les snapshots.
const MELODY: u8 = 0;

#[test]
fn la_melodie_swing_joue_dans_l_accord_et_la_tessiture_du_profil_et_se_coupe() -> TestResult {
    let p = profile()?;
    let (low, high) = (p.melody.params.tessitura_min, p.melody.params.tessitura_max);
    for seed in 1..=8 {
        let flow = stream(seed, CHORUS)?;
        let played = flow.iter().filter(|s| !track(s, MELODY).is_empty()).count();
        assert!(played >= CHORUS - 2, "graine {seed} : {played} mesures sur {CHORUS} avec mélodie");
        let mut distinct = BTreeSet::new();
        for (bar, snap) in flow.iter().enumerate() {
            for n in track(snap, MELODY) {
                assert!(
                    (low..=high).contains(&n.pitch),
                    "graine {seed} mesure {} : {n:?}",
                    bar + 1
                );
                // Sous une grille, chaque note est ramenée à une note de l'accord publié (modèle `loop`).
                assert!(
                    context_at(snap, n.start_step).pitch_classes.contains(&(n.pitch % 12)),
                    "graine {seed} mesure {} : {n:?} hors de l'accord",
                    bar + 1
                );
                distinct.insert(n.pitch);
            }
        }
        assert!(distinct.len() >= 6, "graine {seed} : {} hauteurs seulement", distinct.len());
    }
    // `EnableMelody(false)` : plus aucune note de mélodie, le reste de l'accompagnement joue.
    let mut g = TuneGenerator::new(
        MusicalParams {
            bpm: p.meter.default_bpm,
            time_signature: p.meter.time_signature,
            enable_melody: false,
            ..Default::default()
        },
        TuningParams { profile: p.clone() },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    for bar in 1..=CHORUS {
        let snap = MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng));
        assert!(track(&snap, MELODY).is_empty(), "mesure {bar} : mélodie coupée");
        assert!(!track(&snap, BASS).is_empty(), "mesure {bar} : la basse joue");
    }
    Ok(())
}

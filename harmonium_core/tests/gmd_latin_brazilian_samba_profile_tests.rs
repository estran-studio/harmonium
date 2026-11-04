//! B3 (phase 019) : contrat du profil distribué `gmd_latin_brazilian_samba`, observé dans le flux de
//! `MeasureSnapshot` (principe VI).
//!
//! Samba en 4/4 sur 16 pas (deux mesures de 2/4 de 8 pas), feel droit, aucune clave, aucune mélodie.
//! Un chorus de seize mesures, cinq chorus par air (80 mesures). Les attendus sont écrits en clair, relatifs
//! à la tonique : les quatre grilles par défaut, la basse sur les changements d'accord (deux appuis par mesure),
//! le comping dans l'accord publié, le kit et les voix latines de la batterie (surdo aux pas 4 et 12 seulement,
//! orchestration surdo -> ensemble -> batucada par chorus), le tempo dans la plage de la source, l'arc et
//! l'habillage par graine (`session.toml`).
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

const ID: &str = "gmd_latin_brazilian_samba";
const CHORDS: u8 = 1;
const BASS: u8 = 2;
const PERCUSSION_LOW: u8 = 3;
const PERCUSSION_HIGH: u8 = 4;
const BARS: usize = 16;
const CHORUSES: usize = 5;
const SEEDS: [u64; 4] = [1, 2, 3, 17];

// Notes GM du kit (percussion.toml).
const KICK: u8 = 36;
const SIDESTICK: u8 = 37;
const CAIXA: u8 = 38;
const SURDO: u8 = 43;
const PEDAL: u8 = 44;
const TOM_MEDIUM: u8 = 47;
const TOM_HIGH: u8 = 50;
const PANDEIRO: u8 = 54;
const AGOGO: [u8; 2] = [67, 68];
const TAMBORIM: u8 = 76;
const SHAKER: u8 = 82;
/// Tout ce que le profil peut jouer à la batterie.
const KIT: [u8; 12] = [
    KICK, SIDESTICK, CAIXA, SURDO, PEDAL, TOM_MEDIUM, TOM_HIGH, PANDEIRO, 67, 68, TAMBORIM, SHAKER,
];
/// Voix de la piste grave (grosse caisse et surdo) ; toutes les autres sont sur la piste aiguë.
const LOW_VOICES: [u8; 2] = [KICK, SURDO];

/// Une boucle de grille : (demi-tons au-dessus de la tonique, durée en pas), `steps_per_bar` = 16.
type Loop = &'static [(u8, usize)];

const LOOPS: [Loop; 4] = [
    // I-vi-ii-V quatre fois ; ii et V à la blanche à la mesure 15, V7 tient la mesure 16.
    &[
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
        (2, 8),
        (7, 24),
    ],
    // I-IV-V.
    &[(0, 32), (5, 32), (7, 32), (0, 32), (0, 16), (5, 32), (7, 16), (5, 8), (7, 24), (0, 32)],
    // Cycle de quintes diatonique, deux fois huit mesures.
    &[
        (0, 16),
        (5, 16),
        (11, 16),
        (4, 16),
        (9, 16),
        (2, 16),
        (7, 16),
        (0, 16),
        (0, 16),
        (5, 16),
        (11, 16),
        (4, 16),
        (9, 16),
        (2, 8),
        (7, 8),
        (0, 32),
    ],
    // vi-ii-V-I, quatre fois quatre mesures.
    &[
        (9, 16),
        (2, 8),
        (7, 8),
        (0, 32),
        (9, 16),
        (2, 8),
        (7, 8),
        (0, 32),
        (9, 16),
        (2, 8),
        (7, 8),
        (0, 32),
        (9, 16),
        (2, 8),
        (7, 8),
        (0, 32),
    ],
];

/// Plages de tempo de l'habillage (GMD 89 à 110, latin/samba 116) et écart de l'arousal.
const TEMPO_RANGES: [(&str, f32, f32); 3] =
    [("pose", 89.0, 92.0), ("mesure", 101.0, 104.0), ("vif", 113.0, 116.0)];
const AROUSAL_OFFSET: f32 = 4.0;
const BPM_BOUNDS: (f32, f32, f32) = (85.0, 89.0, 120.0);
const KEYS: [(&str, u8); 6] =
    [("do", 0), ("re", 2), ("fa", 5), ("sol", 7), ("la", 9), ("si_bemol", 10)];
const COMPING: [(&str, u8); 4] =
    [("guitare_nylon", 24), ("guitare_acier", 25), ("piano", 0), ("piano_tine", 4)];
const BASSES: [(&str, u8); 3] =
    [("contrebasse", 32), ("basse_aux_doigts", 33), ("basse_fretless", 35)];
const KITS: [(&str, u8); 4] = [("standard", 0), ("room", 8), ("jazz", 32), ("balais", 40)];

fn profile() -> TestResult<StyleProfile> {
    Ok(StyleProfile::named(ID)?)
}

fn stream_with(
    seed: u64,
    bars: usize,
    intensity: f32,
    density: u8,
) -> TestResult<Vec<MeasureSnapshot>> {
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
    stream_with(seed, CHORUSES * BARS, 0.5, 1)
}

fn track(s: &MeasureSnapshot, id: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == id).collect()
}

fn percussion(s: &MeasureSnapshot) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == PERCUSSION_LOW || n.track == PERCUSSION_HIGH).collect()
}

fn drum_pitches(s: &MeasureSnapshot) -> BTreeSet<u8> {
    percussion(s).iter().map(|n| n.pitch).collect()
}

fn attacks(s: &MeasureSnapshot, id: u8) -> BTreeSet<usize> {
    track(s, id).iter().map(|n| n.start_step).collect()
}

/// Mesures du chorus `n` (1-based).
fn chorus(flow: &[MeasureSnapshot], n: usize) -> &[MeasureSnapshot] {
    &flow[(n - 1) * BARS..n * BARS]
}

fn drawn(s: &MeasureSnapshot, group: &str) -> String {
    s.session.iter().find(|v| v.group == group).map(|v| v.name.clone()).unwrap_or_default()
}

fn key_of(s: &MeasureSnapshot) -> TestResult<u8> {
    Ok(s.position.ok_or("position")?.key)
}

/// Degré (demi-tons au-dessus de la tonique de la graine) d'un accord publié.
fn degree(key: u8, c: &ChordContext) -> u8 {
    (c.root_pc + 12 - key) % 12
}

fn context_at(s: &MeasureSnapshot, step: usize) -> &ChordContext {
    &s.chord_changes[s.chord_changes.partition_point(|c| c.start_step <= step) - 1].context
}

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

#[test]
fn la_grille_par_defaut_est_une_des_boucles_ecrites_reprise_a_l_identique_a_chaque_chorus()
-> TestResult {
    let expected: Vec<_> = LOOPS.iter().map(|l| expected_changes(l)).collect();
    for e in &expected {
        assert_eq!(e.len(), BARS, "une boucle écrite fait {BARS} mesures");
    }
    let (mut seen, mut keys) = (BTreeSet::new(), BTreeSet::new());
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let key = key_of(&flow[0])?;
        // La tonique publiée est celle de la variante de tonalité tirée.
        let named =
            KEYS.iter().find(|(name, _)| *name == drawn(&flow[0], "tonalite")).ok_or("tonalité")?;
        assert_eq!(key, named.1, "graine {seed} : tonique publiée et variante tirée");
        keys.insert(key);
        let first: Vec<_> = flow[..BARS].iter().map(|s| changes(key, s)).collect();
        let which = expected.iter().position(|e| *e == first).ok_or_else(|| {
            format!("graine {seed} : grille hors des boucles écrites : {first:?}")
        })?;
        seen.insert(which);
        for n in 2..=CHORUSES {
            let again: Vec<_> = chorus(&flow, n).iter().map(|s| changes(key, s)).collect();
            assert_eq!(
                again, first,
                "graine {seed} chorus {n} : la boucle se reprend à l'identique"
            );
        }
    }
    assert!(seen.len() >= 3, "16 graines n'ont tiré que {} grilles", seen.len());
    assert!(keys.len() >= 4, "16 graines n'ont tiré que {} tonalités", keys.len());
    Ok(())
}

#[test]
fn le_profil_n_a_ni_clave_ni_melodie_ni_swing_et_reste_en_seize_pas_droits() -> TestResult {
    let p = profile()?;
    assert!(p.clave.is_none(), "la samba n'a pas de clave");
    assert!((p.meter.default_swing_ratio - 0.5).abs() < 1e-6, "feel droit");
    assert_eq!(
        (p.meter.min_bpm, p.meter.default_bpm, p.meter.max_bpm),
        BPM_BOUNDS,
        "tempos mesurés 89 à 116, ± 4 de l'arousal"
    );
    for seed in SEEDS {
        for s in &air(seed)? {
            assert_eq!(s.steps, 16, "graine {seed} mesure {} : seize pas", s.index);
            assert!((s.swing_ratio - 0.5).abs() < 1e-6, "graine {seed} : swing {}", s.swing_ratio);
            assert!(track(s, 0).is_empty(), "graine {seed} mesure {} : aucune mélodie", s.index);
            // Aucune attaque entre deux pas : la grille est droite.
            for n in &s.notes {
                assert!(n.start_step < 16, "graine {seed} : attaque au pas {}", n.start_step);
            }
        }
    }
    Ok(())
}

#[test]
fn le_tempo_reste_dans_la_plage_de_la_variante_tiree_et_de_la_source() -> TestResult {
    let mut seen = BTreeSet::new();
    for seed in 1..=16u64 {
        let first = stream_with(seed, 1, 0.5, 1)?.remove(0);
        let name = drawn(&first, "tempo");
        let &(_, low, high) =
            TEMPO_RANGES.iter().find(|(n, ..)| *n == name).ok_or("variante de tempo")?;
        seen.insert(name);
        assert!(
            (low..=high).contains(&first.composition_bpm),
            "graine {seed} : tempo nominal {} hors de {low}..{high}",
            first.composition_bpm
        );
        // Toutes les intensités : le tempo bouge de ± 4 BPM au plus autour de celui de la variante.
        for intensity in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
            for s in &stream_with(seed, BARS + 2, intensity, 1)? {
                assert!(
                    (low - AROUSAL_OFFSET..=high + AROUSAL_OFFSET).contains(&s.tempo)
                        && (BPM_BOUNDS.0..=BPM_BOUNDS.2).contains(&s.tempo),
                    "graine {seed} intensité {intensity} : tempo {} hors de {low}..{high} ± 4",
                    s.tempo
                );
            }
        }
    }
    assert_eq!(seen.len(), 3, "16 graines tirent les trois plages de tempo : {seen:?}");
    Ok(())
}

#[test]
fn la_basse_joue_deux_appuis_et_arrive_sur_la_basse_publiee_a_chaque_changement() -> TestResult {
    // Seize graines et la dix-septième : un changement d'accord au temps 3 sous une quinte n'est apparu
    // qu'à la graine 17 (cycle `fifth_if_held`).
    for seed in (1..=16u64).chain([17]) {
        let flow = air(seed)?;
        for (index, s) in flow.iter().enumerate() {
            let ctx = format!("graine {seed} mesure {}", s.index);
            let bass = track(s, BASS);
            // Les deux dernières mesures d'un chorus peuvent porter un break ou un stop-time de l'arc
            // (la basse ne garde que le temps 1) ; la mesure 9 d'un chorus peut taire la guitare seule.
            let in_chorus = index % BARS;
            let broken = in_chorus >= BARS - 2 && index >= BARS;
            for note in &bass {
                assert!(
                    (28..=53).contains(&note.pitch),
                    "{ctx} : hauteur {} hors de l'étendue mesurée de WJazzD LATIN (28..53)",
                    note.pitch
                );
            }
            if broken {
                continue;
            }
            // Deux appuis : la fondamentale sur le un de chaque 2/4.
            let starts = attacks(s, BASS);
            assert!(starts.contains(&0) && starts.contains(&8), "{ctx} : appuis {starts:?}");
            for change in &s.chord_changes {
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
                    (35..=46).contains(&arrival.pitch),
                    "{ctx} pas {} : arrivée {} hors de 35..=46",
                    change.start_step,
                    arrival.pitch
                );
            }
        }
    }
    Ok(())
}

#[test]
fn la_basse_ne_joue_que_fondamentale_quinte_anticipation_ou_approche() -> TestResult {
    let (mut pickups, mut approaches, mut fifths) = (0, 0, 0);
    for seed in SEEDS {
        let flow = air(seed)?;
        for (index, s) in flow.iter().enumerate() {
            let mut bass = track(s, BASS);
            bass.sort_by_key(|n| n.start_step);
            for (rank, note) in bass.iter().enumerate() {
                let here = context_at(s, note.start_step);
                let rel = (note.pitch + 12 - here.bass_pc) % 12;
                if rel == 0 {
                    continue;
                }
                if rel == 7 {
                    fifths += 1;
                    continue;
                }
                // Anticipation : la basse de l'accord qui commence quand la note s'achève (la barre ou la
                // blanche), `lead` de un ou deux pas.
                let end = note.start_step + note.duration_steps;
                let ahead = if end >= 16 {
                    flow.get(index + 1).map(|n| &n.chord_changes[0].context)
                } else {
                    Some(context_at(s, end))
                };
                if ahead.is_some_and(|c| c.bass_pc == note.pitch % 12 && c.bass_pc != here.bass_pc)
                    && note.duration_steps <= 2
                {
                    pickups += 1;
                    continue;
                }
                // Sinon une approche : à un ou deux demi-tons de la note de basse suivante (de la mesure
                // suivante pour la dernière).
                let next = bass.get(rank + 1).map(|n| n.pitch).or_else(|| {
                    flow.get(index + 1)
                        .and_then(|n| track(n, BASS).iter().map(|b| (b.start_step, b.pitch)).min())
                        .map(|b| b.1)
                });
                let next = next.ok_or("cible d'approche")?;
                let distance = note.pitch.abs_diff(next);
                assert!(
                    (1..=2).contains(&distance),
                    "graine {seed} mesure {} pas {} : {} n'est ni appui, ni quinte, ni anticipation, ni approche de {next}",
                    s.index,
                    note.start_step,
                    note.pitch
                );
                approaches += 1;
            }
        }
    }
    assert!(fifths > 100, "{fifths} quintes");
    assert!(pickups > 20, "{pickups} anticipations du changement d'accord");
    assert!(approaches > 0, "aucune approche sur {} graines", SEEDS.len());
    Ok(())
}

#[test]
fn le_comping_ne_joue_que_des_notes_de_l_accord_publie_dans_la_famille_et_le_registre_tires()
-> TestResult {
    let mut grave = BTreeSet::new();
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let family = drawn(&flow[0], "voicing");
        let octave_down = drawn(&flow[0], "registre") == "grave";
        grave.insert(octave_down);
        let shift = if octave_down { 12 } else { 0 };
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
                let ctx = format!("graine {seed} ({family}) mesure {} pas {step}", snap.index);
                let here = context_at(snap, step);
                // Une attaque qui finit sur la barre peut être anticipée : elle joue alors l'accord de la
                // mesure suivante.
                let ahead = (step + duration == 16).then(|| flow.get(index + 1)).flatten();
                let played = pcs(&pitches);
                let fits = |c: &ChordContext| played.iter().all(|pc| c.pitch_classes.contains(pc));
                // L'accord qui sonne : celui du pas, ou celui de la mesure suivante pour une anticipation.
                let next = ahead.map(|a| &a.chord_changes[0].context);
                let fitting: Vec<&ChordContext> =
                    std::iter::once(here).chain(next).filter(|c| fits(c)).collect();
                assert!(
                    !fitting.is_empty(),
                    "{ctx} : {played:?} hors de {} {:?}",
                    here.chord_name,
                    here.pitch_classes
                );
                if family != "drop_2" {
                    assert!(
                        fitting.iter().any(|c| !played.contains(&c.root_pc)),
                        "{ctx} : fondamentale jouée"
                    );
                }
                if family == "shell" {
                    assert!(pitches.len() <= 2, "{ctx} : shell à {} sons", pitches.len());
                }
                assert!(
                    pitches[0] + shift >= 46 && pitches[pitches.len() - 1] + shift <= 78,
                    "{ctx} : tessiture {pitches:?} (registre {octave_down})"
                );
                attack_count += 1;
            }
        }
        assert!(attack_count > 60, "graine {seed} : {attack_count} attaques de comping");
    }
    assert_eq!(grave.len(), 2, "16 graines montrent les deux registres");
    Ok(())
}

#[test]
fn la_batterie_reste_dans_le_kit_et_les_voix_latines_sont_sur_les_bonnes_pistes() -> TestResult {
    let mut seen = BTreeSet::new();
    for seed in SEEDS {
        let flow = air(seed)?;
        for s in &flow {
            let ctx = format!("graine {seed} mesure {}", s.index);
            for n in percussion(s) {
                assert!(KIT.contains(&n.pitch), "{ctx} : note {} hors du kit", n.pitch);
                // Grosse caisse et surdo sur la piste grave, tout le reste (tamborim, agogô, chocalho,
                // pandeiro, caisse claire, charleston au pied, toms) sur la piste aiguë.
                let low = LOW_VOICES.contains(&n.pitch);
                assert_eq!(
                    n.track == PERCUSSION_LOW,
                    low,
                    "{ctx} : note {} sur la piste {}",
                    n.pitch,
                    n.track
                );
                seen.insert(n.pitch);
            }
        }
    }
    for note in [KICK, SURDO, PEDAL, CAIXA, TAMBORIM, AGOGO[0], AGOGO[1], SHAKER, PANDEIRO] {
        assert!(seen.contains(&note), "note {note} jamais jouée sur {} graines", SEEDS.len());
    }
    Ok(())
}

#[test]
fn le_surdo_est_ouvert_aux_temps_deux_et_la_batterie_s_etoffe_chorus_apres_chorus() -> TestResult {
    let mut fills = 0;
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let ctx = |n: usize| format!("graine {seed} chorus {n}");
        // Chorus 1 (exposition) et 5 (retombée) : surdo, grosse caisse douce et chocalho seulement.
        for n in [1, CHORUSES] {
            for s in chorus(&flow, n) {
                let notes = drum_pitches(s);
                assert!(
                    notes.is_subset(&BTreeSet::from([KICK, SURDO, SHAKER])),
                    "{} mesure {} : {notes:?}",
                    ctx(n),
                    s.index
                );
                assert!(
                    !track_steps(s, SURDO).is_empty(),
                    "{} mesure {} : pas de surdo",
                    ctx(n),
                    s.index
                );
            }
        }
        // Le surdo s'ouvre aux pas 4 et 12 (temps 2 de chaque 2/4), jamais ailleurs hors relance.
        for n in [1, 2, CHORUSES] {
            for s in chorus(&flow, n) {
                assert_eq!(
                    track_steps(s, SURDO),
                    BTreeSet::from([4, 12]),
                    "{} mesure {} : pas du surdo",
                    ctx(n),
                    s.index
                );
            }
        }
        // Chorus 2 : l'ensemble entre (tamborim au pas 0 à chaque mesure, charleston au pied presque
        // toujours), sans caisse claire ni pandeiro.
        let two = chorus(&flow, 2);
        let pedal = two.iter().filter(|s| drum_pitches(s).contains(&PEDAL)).count();
        for s in two {
            let notes = drum_pitches(s);
            assert!(
                track_steps(s, TAMBORIM).contains(&0),
                "{} mesure {} : tamborim",
                ctx(2),
                s.index
            );
            assert!(
                !notes.contains(&CAIXA)
                    && !notes.contains(&PANDEIRO)
                    && !notes.contains(&SIDESTICK),
                "{} mesure {} : {notes:?}",
                ctx(2),
                s.index
            );
        }
        assert!(
            pedal * 10 >= two.len() * 9,
            "{} : charleston au pied dans {pedal} mesures",
            ctx(2)
        );
        // Chorus 4 (sommet) : la batucada au complet, caisse claire et pandeiro ; les relances (une mesure
        // sans tamborim au pas 0) coupent l'ensemble.
        let peak = chorus(&flow, 4);
        let caixa = peak.iter().filter(|s| drum_pitches(s).contains(&CAIXA)).count();
        let pandeiro = peak.iter().filter(|s| drum_pitches(s).contains(&PANDEIRO)).count();
        assert!(caixa * 10 >= peak.len() * 8, "{} : caisse claire dans {caixa} mesures", ctx(4));
        assert!(pandeiro * 10 >= peak.len() * 6, "{} : pandeiro dans {pandeiro} mesures", ctx(4));
        let agogo = peak.iter().any(|s| AGOGO.iter().any(|a| drum_pitches(s).contains(a)));
        assert!(agogo, "{} : agogô", ctx(4));
        fills += peak.iter().filter(|s| !track_steps(s, TAMBORIM).contains(&0)).count();
    }
    assert!(fills >= 8, "{fills} mesures de relance au sommet sur 16 graines");
    Ok(())
}

fn track_steps(s: &MeasureSnapshot, pitch: u8) -> BTreeSet<usize> {
    percussion(s).iter().filter(|n| n.pitch == pitch).map(|n| n.start_step).collect()
}

#[test]
fn l_arc_creuse_l_ecart_entre_l_exposition_le_sommet_et_la_retombee_avec_stop_time() -> TestResult {
    for seed in SEEDS {
        let flow = air(seed)?;
        let notes = |n: usize| -> usize {
            chorus(&flow, n).iter().map(|s| track(s, CHORDS).len() + percussion(s).len()).sum()
        };
        let (first, peak, fall) = (notes(1), notes(4), notes(5));
        assert!(first < peak, "graine {seed} : exposition {first} contre sommet {peak}");
        assert!(fall < peak, "graine {seed} : retombée {fall} contre sommet {peak}");
        assert!(
            first * 10 <= peak * 7,
            "graine {seed} : exposition {first} contre sommet {peak} (70 %)"
        );
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
            assert!(
                !percussion(s).is_empty(),
                "graine {seed} mesure {} : la batterie continue",
                s.index
            );
        }
        // L'arc ne change pas le tempo : il est le même à chaque chorus.
        let tempos: BTreeSet<u32> = flow.iter().map(|s| (s.tempo * 100.0) as u32).collect();
        assert_eq!(tempos.len(), 1, "graine {seed} : tempos {tempos:?}");
    }
    Ok(())
}

#[test]
fn la_meme_graine_rejoue_pareil_et_les_graines_changent_comping_batterie_et_basse() -> TestResult {
    let a = air(1)?;
    let again = air(1)?;
    assert_eq!(serde_json::to_string(&a)?, serde_json::to_string(&again)?);
    let (mut comping, mut drums, mut bass) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        comping.insert(sequence(&flow, |s| {
            track(s, CHORDS).iter().map(|n| n.start_step.to_string()).collect::<Vec<_>>().join(",")
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

/// Un habillage tel que le premier chorus le montre : tempo (grappe à ± 8 BPM), tonalité, programmes de
/// l'habillage (variantes tirées), famille de voicings, registre, énergie, arc.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Dressing {
    tempo_class: usize,
    key: u8,
    comping: String,
    bass: String,
    kit: String,
    voicing: String,
    register: String,
    energy: String,
    arc: String,
}

#[test]
fn des_graines_differentes_donnent_au_moins_huit_habillages_distincts_sur_seize() -> TestResult {
    let mut rows = Vec::new();
    for seed in 1..=16u64 {
        let flow = air(seed)?;
        let first = &flow[0];
        let groups: BTreeSet<_> = first.session.iter().map(|v| v.group.as_str()).collect();
        assert_eq!(groups.len(), 9, "graine {seed} : un tirage par groupe : {:?}", first.session);
        rows.push((first.composition_bpm, seed, flow));
    }
    // Grappes de tempo à ± 8 BPM : un tempo rejoint la grappe dont le premier membre est à moins de 8 BPM.
    let mut tempos: Vec<f32> = rows.iter().map(|r| r.0).collect();
    tempos.sort_by(f32::total_cmp);
    let mut starts: Vec<f32> = Vec::new();
    for t in tempos {
        if starts.last().is_none_or(|first| t - first > 8.0) {
            starts.push(t);
        }
    }
    let mut dressings = BTreeSet::new();
    let (mut chords, mut bass, mut kits, mut voicings, mut keys) =
        (BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    let mut orchestrated = 0;
    for (bpm, seed, flow) in &rows {
        let first = &flow[0];
        let tempo_class = starts.iter().rposition(|s| s <= bpm).ok_or("grappe")?;
        let lookup = |table: &[(&str, u8)], group: &str| -> TestResult<u8> {
            let name = drawn(first, group);
            Ok(table.iter().find(|(n, _)| *n == name).ok_or(format!("variante {name}"))?.1)
        };
        // Les programmes publiés sont ceux de l'habillage ; l'arc « orchestre » en change par chorus.
        let (comping, basse, kit) =
            (lookup(&COMPING, "comping")?, lookup(&BASSES, "basse")?, lookup(&KITS, "kit")?);
        let orchestre = drawn(first, "arc") == "orchestre";
        let mut by_chorus: BTreeMap<usize, (Option<u8>, Option<u8>, u16, u8)> = BTreeMap::new();
        for s in flow {
            let i = s.instruments.ok_or("programmes publiés")?;
            let c = s.position.ok_or("position")?.chorus();
            let entry = (i.chords_program, i.bass_program, i.percussion_bank, i.percussion_program);
            assert_eq!(
                *by_chorus.entry(c).or_insert(entry),
                entry,
                "graine {seed} : un seul jeu par chorus"
            );
        }
        for (&c, &(ch, ba, bank, pr)) in &by_chorus {
            assert_eq!(ba, Some(basse), "graine {seed} chorus {c} : basse");
            assert_eq!(bank, 128, "graine {seed} chorus {c} : banque du kit");
            let (want_chords, want_kit) = if orchestre {
                match c {
                    1 | 5 => (24, 40),
                    3 => (0, kit),
                    4 => (25, 16),
                    _ => (comping, kit),
                }
            } else {
                (comping, kit)
            };
            assert_eq!(ch, Some(want_chords), "graine {seed} chorus {c} : comping");
            assert_eq!(pr, want_kit, "graine {seed} chorus {c} : kit");
        }
        orchestrated += usize::from(
            by_chorus.values().collect::<BTreeSet<_>>().len() > 1
                && by_chorus.values().map(|v| v.0).collect::<BTreeSet<_>>().len() > 1,
        );
        let d = Dressing {
            tempo_class,
            key: key_of(first)?,
            comping: drawn(first, "comping"),
            bass: drawn(first, "basse"),
            kit: drawn(first, "kit"),
            voicing: drawn(first, "voicing"),
            register: drawn(first, "registre"),
            energy: drawn(first, "energie"),
            arc: drawn(first, "arc"),
        };
        chords.insert(d.comping.clone());
        bass.insert(d.bass.clone());
        kits.insert(d.kit.clone());
        voicings.insert(d.voicing.clone());
        keys.insert(d.key);
        dressings.insert(d);
    }
    println!(
        "16 graines : {} habillages distincts ; {} grappes de tempo ; tonalités {keys:?} ; comping {chords:?} ; basse {bass:?} ; kit {kits:?} ; voicings {voicings:?} ; {orchestrated} orchestrées",
        dressings.len(),
        starts.len()
    );
    assert_eq!(starts.len(), 3, "trois tempos à ± 8 BPM près : {starts:?}");
    assert!(keys.len() >= 4, "tonalités : {keys:?}");
    assert!(chords.len() >= 3, "comping : {chords:?}");
    assert!(bass.len() >= 2, "basses : {bass:?}");
    assert!(kits.len() >= 2, "kits : {kits:?}");
    assert!(voicings.len() >= 3, "voicings : {voicings:?}");
    assert!(orchestrated >= 1, "aucune graine n'orchestre par chorus");
    assert!(dressings.len() >= 8, "{} habillages distincts sur 16 graines", dressings.len());
    Ok(())
}

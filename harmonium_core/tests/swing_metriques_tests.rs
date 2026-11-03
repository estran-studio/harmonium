//! A6 : swing selon le tempo et par piste, feels ternaires et métriques (12 pas en 4/4, 3/4, 6/8),
//! micro-timing par voix de batterie, coups probabilistes, clave.
//!
//! Les mesures sont lues à l'instant où le lecteur les joue : `TimePlan` (le plan partagé que suit
//! le playback) donne l'instant de chaque pas, et le décalage de l'attaque (millièmes de pas) se
//! place dans la durée du pas qui la contient, comme `PlaybackEngine::event_frame`. Le rapport de
//! swing d'une piste est la fraction de la noire où tombent ses attaques de contretemps.
//!
//! Fixtures (`tests/fixtures/profiles/`, sans Odin) : `grille_shuffle_ternaire` (4/4 sur 12 pas),
//! `grille_valse_jazz` (3/4, arc et phrases), `grille_six_huit` (6/8, arc et phrases),
//! `grille_clave` (clave son 3-2 / 2-3). Courbes : `references/jtd-swing-tempo` du labo.
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams,
    compose::{PhraseAt, TuneGenerator},
    params::MusicalParams,
    time_plan::{BarTiming, TimePlan},
    timeline::{Measure, TimelineNote, TrackId},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const RIDE: u8 = 51;
const SNARE: u8 = 38;
const CLAVES: u8 = 75;

fn fixture(name: &str) -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))?)
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

fn root(v: &mut toml::Value) -> TestResult<&mut toml::Table> {
    Ok(v.as_table_mut().ok_or("profil")?)
}

fn without(p: &StyleProfile, keys: &[&str]) -> TestResult<StyleProfile> {
    edited(p, |v| {
        for key in keys {
            root(v)?.remove(*key);
        }
        Ok(())
    })
}

/// Réglages du générateur : `bpm` fixe le tempo (commande directe du jeu) ; `swing` fixe le rapport
/// comme le ferait `SetSwing` de l'hôte.
#[derive(Clone, Copy, Default)]
struct Setup {
    bpm: Option<f32>,
    swing: Option<f32>,
    /// Écart ajouté au rapport par défaut du profil (dosage), sans commande directe.
    dose: f32,
    density: u8,
}

fn generator(p: &StyleProfile, setup: Setup) -> TuneGenerator {
    let mut params = MusicalParams {
        bpm: setup.bpm.unwrap_or(p.meter.default_bpm),
        bpm_explicit: setup.bpm.is_some(),
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        intensity: 0.5,
        percussion_density: setup.density,
        ..Default::default()
    };
    params.percussion_density = setup.density;
    params.set_swing_ratio(p.meter.default_swing_ratio + setup.dose);
    if let Some(ratio) = setup.swing {
        params.set_swing_ratio(ratio);
        params.swing_explicit = true;
    }
    TuneGenerator::new(params, TuningParams { profile: p.clone() })
}

/// `bars` mesures générées, plus une de plus pour borner la dernière.
fn measures(p: &StyleProfile, seed: u64, bars: usize, setup: Setup) -> TestResult<Vec<Measure>> {
    p.validate()?;
    let mut g = generator(p, setup);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars + 1).map(|bar| g.generate_measure(bar, &mut rng)).collect())
}

fn plan_of(measures: &[Measure]) -> TimePlan {
    let mut plan = TimePlan::new();
    for measure in measures {
        plan.push(BarTiming::of(measure));
    }
    plan
}

/// Instant (flicks) de l'attaque de `note` dans la mesure 1-based `bar` ; `None` avant le pas 0.
fn instant(plan: &TimePlan, bar: usize, note: &TimelineNote) -> Option<f64> {
    let steps = i64::from(plan.timing(bar)?.steps);
    let milli = note.start_step as i64 * 1000 + i64::from(note.performance.offset);
    let (step, frac) = (milli.div_euclid(1000), milli.rem_euclid(1000) as f64 / 1000.0);
    if step < 0 {
        return None;
    }
    let start = plan.step_instant(bar, step as u16)?.0 as f64;
    let next = if step + 1 < steps {
        plan.step_instant(bar, step as u16 + 1)?
    } else {
        plan.bar_start(bar + 1)?
    };
    Some(start + frac * (next.0 as f64 - start))
}

/// Instant du pas entier `step` de la mesure `bar` ; `step == steps` : début de la mesure suivante.
fn step_instant(plan: &TimePlan, bar: usize, step: usize) -> Option<f64> {
    if step >= usize::from(plan.timing(bar)?.steps) {
        return Some(plan.bar_start(bar + 1)?.0 as f64);
    }
    Some(plan.step_instant(bar, step as u16)?.0 as f64)
}

/// Rapport de swing de chaque attaque de contretemps d'une piste (note retenue par `keep`) : la
/// fraction de sa noire où elle tombe. Contretemps = pas de rang `⌈pas par noire / 2⌉` dans la
/// noire (le milieu de la noire sur 4 pas, le 3e pas sur 3).
fn swing_ratios(
    measures: &[Measure],
    track: TrackId,
    keep: impl Fn(&TimelineNote) -> bool,
) -> Vec<f64> {
    let plan = plan_of(measures);
    let mut out = Vec::new();
    for (index, measure) in measures[..measures.len() - 1].iter().enumerate() {
        let bar = index + 1;
        let Some(unit) = BarTiming::of(measure).swing_unit_steps().map(|u| u as usize) else {
            continue;
        };
        let knot = unit.div_ceil(2);
        for note in measure.notes_for_track(track).iter().filter(|n| keep(n)) {
            if note.start_step % unit != knot {
                continue;
            }
            let first = note.start_step - knot;
            let (Some(a), Some(b), Some(at)) = (
                step_instant(&plan, bar, first),
                step_instant(&plan, bar, first + unit),
                instant(&plan, bar, note),
            ) else {
                continue;
            };
            out.push((at - a) / (b - a));
        }
    }
    out
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    values.sort_by(f64::total_cmp);
    values.get(values.len() / 2).copied()
}

fn track_swing(
    measures: &[Measure],
    track: TrackId,
    keep: impl Fn(&TimelineNote) -> bool,
) -> TestResult<f64> {
    let ratios = swing_ratios(measures, track, keep);
    assert!(ratios.len() >= 8, "{track:?} : {} attaques de contretemps seulement", ratios.len());
    median(ratios).ok_or_else(|| "aucun contretemps".into())
}

fn any(_: &TimelineNote) -> bool {
    true
}

fn ride(note: &TimelineNote) -> bool {
    note.pitch == RIDE
}

// --- Courbes JTD (`a6_table.curve`, points à 3 décimales) ----------------------------------------

/// `(tempo, batterie, basse, piano)` : `references/jtd-swing-tempo/measures-swing-tempo.json`.
const JTD: [(f32, f64, f64, f64); 5] = [
    (110.0, 0.7213, 0.7148, 0.7021),
    (150.0, 0.712, 0.706, 0.702),
    (190.0, 0.673, 0.688, 0.673),
    (230.0, 0.633, 0.671, 0.644),
    (270.0, 0.595, 0.653, 0.615),
];
/// Tolérance sur le rapport mesuré : arrondi du décalage (millième de pas) et des points JTD.
const TOLERANCE: f64 = 0.004;

#[test]
fn chaque_piste_suit_sa_courbe_de_swing_selon_le_tempo_sur_une_grille_ternaire() -> TestResult {
    // 4/4 sur 12 pas : le contretemps est le 3e pas de la noire. Sans le micro-timing de voix, que
    // le rapport mesuré ne doit pas contenir.
    let p = without(&fixture("grille_shuffle_ternaire")?, &["performance"])?;
    for (bpm, drums, bass, piano) in JTD {
        let flow = measures(&p, 4, 60, Setup { bpm: Some(bpm), ..Setup::default() })?;
        assert!((f64::from(flow[0].swing_ratio) - drums).abs() < 0.001, "grille à {bpm} BPM");
        let got = [
            ("batterie", track_swing(&flow, TrackId::PercussionHigh, ride)?, drums),
            ("basse", track_swing(&flow, TrackId::Bass, any)?, bass),
            ("comping", track_swing(&flow, TrackId::Chords, any)?, piano),
        ];
        for (name, measured, expected) in got {
            assert!(
                (measured - expected).abs() < TOLERANCE,
                "{name} à {bpm} BPM : {measured:.4}, JTD {expected:.4}"
            );
        }
    }
    Ok(())
}

#[test]
fn chaque_piste_suit_sa_courbe_de_swing_selon_le_tempo_sous_une_grille_en_3_4() -> TestResult {
    let p = fixture("grille_valse_jazz")?;
    for (bpm, drums, _, piano) in JTD {
        let flow = measures(&p, 2, 72, Setup { bpm: Some(bpm), ..Setup::default() })?;
        let ride_swing = track_swing(&flow, TrackId::PercussionHigh, ride)?;
        let comping_swing = track_swing(&flow, TrackId::Chords, any)?;
        assert!((ride_swing - drums).abs() < TOLERANCE, "ride à {bpm} : {ride_swing:.4}");
        assert!((comping_swing - piano).abs() < TOLERANCE, "comping à {bpm} : {comping_swing:.4}");
    }
    Ok(())
}

#[test]
fn la_courbe_est_decroissante_et_bornee_aux_tempos_mesures() -> TestResult {
    let p = fixture("grille_shuffle_ternaire")?;
    let swing = p.swing.as_ref().ok_or("section swing")?;
    let mut previous = [1.0_f32; 3];
    for bpm in (60..=300).step_by(10) {
        let at =
            swing.resolve(bpm as f32, p.meter.default_swing_ratio, p.meter.default_swing_ratio);
        let now = [at.grid, at.of(TrackId::Bass), at.of(TrackId::Chords)];
        for (n, before) in now.iter().zip(previous) {
            assert!(*n <= before + 1e-6, "monotone non croissante à {bpm} BPM");
            assert!((0.5..=0.75).contains(n));
        }
        previous = now;
    }
    // Hors plage mesurée (< 102 et > 299 BPM) : valeur de l'extrémité, jamais d'extrapolation.
    let low = swing.resolve(40.0, 0.7213, 0.7213);
    let high = swing.resolve(300.0, 0.7213, 0.7213);
    assert!((low.grid - 0.7213).abs() < 1e-6 && (high.grid - 0.5658).abs() < 1e-6);
    Ok(())
}

#[test]
fn la_commande_directe_de_l_hote_l_emporte_sur_la_courbe_pour_toutes_les_pistes() -> TestResult {
    let p = without(&fixture("grille_shuffle_ternaire")?, &["performance"])?;
    for bpm in [110.0, 270.0] {
        let setup = Setup { bpm: Some(bpm), swing: Some(0.6), ..Setup::default() };
        let flow = measures(&p, 4, 48, setup)?;
        assert!(flow.iter().all(|m| (m.swing_ratio - 0.6).abs() < f32::EPSILON), "{bpm} BPM");
        for (track, keep) in [
            (TrackId::PercussionHigh, ride as fn(&TimelineNote) -> bool),
            (TrackId::Bass, any),
            (TrackId::Chords, any),
        ] {
            let measured = track_swing(&flow, track, keep)?;
            assert!((measured - 0.6).abs() < 0.002, "{track:?} à {bpm} BPM : {measured:.4}");
        }
        // Un seul rapport : aucun décalage de swing par piste.
        let shifted = flow
            .iter()
            .flat_map(|m| m.tracks.values().flatten())
            .filter(|n| n.performance.offset != 0)
            .count();
        assert_eq!(shifted, 0);
    }
    Ok(())
}

#[test]
fn l_ecart_au_defaut_du_profil_decale_toutes_les_courbes_sans_commande_directe() -> TestResult {
    let p = without(&fixture("grille_shuffle_ternaire")?, &["performance"])?;
    let base = measures(&p, 4, 60, Setup { bpm: Some(190.0), ..Setup::default() })?;
    let dosed = measures(&p, 4, 60, Setup { bpm: Some(190.0), dose: -0.03, ..Setup::default() })?;
    for (track, keep) in
        [(TrackId::PercussionHigh, ride as fn(&TimelineNote) -> bool), (TrackId::Bass, any)]
    {
        let (a, b) = (track_swing(&base, track, keep)?, track_swing(&dosed, track, keep)?);
        assert!((a - b - 0.03).abs() < TOLERANCE, "{track:?} : {a:.4} → {b:.4}");
    }
    Ok(())
}

#[test]
fn sans_courbe_un_seul_rapport_quel_que_soit_le_tempo() -> TestResult {
    let p = without(&fixture("grille_valse_jazz")?, &["swing"])?;
    assert!(p.swing.is_none());
    for bpm in [110.0, 270.0] {
        let flow = measures(&p, 2, 36, Setup { bpm: Some(bpm), ..Setup::default() })?;
        assert!(flow.iter().all(|m| (m.swing_ratio - p.meter.default_swing_ratio).abs() < 1e-6));
        let measured = track_swing(&flow, TrackId::Chords, any)?;
        assert!((measured - f64::from(p.meter.default_swing_ratio)).abs() < 0.002, "{bpm} BPM");
    }
    Ok(())
}

// --- Ternaire, 3/4, 6/8 ---------------------------------------------------------------------------

#[test]
fn la_grille_de_12_pas_en_4_4_place_les_attaques_sur_les_triolets() -> TestResult {
    let p = without(&fixture("grille_shuffle_ternaire")?, &["performance"])?;
    // Rapport 2/3 : les croches de contretemps tombent exactement sur le triolet.
    let flow = measures(
        &p,
        1,
        24,
        Setup { bpm: Some(100.0), swing: Some(0.6667), density: 2, ..Setup::default() },
    )?;
    let plan = plan_of(&flow);
    let quarter =
        plan.step_instant(1, 3).ok_or("pas 3")?.0 - plan.step_instant(1, 0).ok_or("pas 0")?.0;
    let bar = plan.bar_start(2).ok_or("mesure 2")?.0 - plan.bar_start(1).ok_or("mesure 1")?.0;
    assert_eq!(bar, 4 * quarter, "4 noires par mesure");
    for beat in 0..4u16 {
        let at = plan.step_instant(3, beat * 3).ok_or("temps")?.0;
        let expected = plan.bar_start(3).ok_or("mesure")?.0 + u64::from(beat) * quarter;
        assert!(at.abs_diff(expected) <= 1, "temps {beat}");
    }
    for (track, keep) in [
        (TrackId::PercussionHigh, ride as fn(&TimelineNote) -> bool),
        (TrackId::Bass, any),
        (TrackId::Chords, any),
    ] {
        let measured = track_swing(&flow, track, keep)?;
        assert!((measured - 2.0 / 3.0).abs() < 0.001, "{track:?} : {measured:.4}");
    }
    // Les attaques de la batterie n'occupent que les pas du shuffle : jamais le pas du milieu du
    // triolet (1, 4, 7, 10), comme sur une grille de seizièmes.
    let steps: BTreeSet<usize> = flow
        .iter()
        .flat_map(|m| m.notes_for_track(TrackId::PercussionHigh))
        .filter(|n| n.pitch == RIDE)
        .map(|n| n.start_step)
        .collect();
    assert_eq!(steps, BTreeSet::from([0, 2, 3, 5, 6, 8, 9, 11]));
    Ok(())
}

#[test]
fn le_swing_deplace_le_dernier_pas_du_triolet_et_garde_les_temps() -> TestResult {
    let p = without(&fixture("grille_shuffle_ternaire")?, &["performance"])?;
    for (ratio, expected) in [(0.5, 2.0 / 3.0), (0.6, 0.6), (0.72, 0.72)] {
        let flow = measures(
            &p,
            1,
            12,
            Setup { bpm: Some(120.0), swing: Some(ratio), ..Setup::default() },
        )?;
        let plan = plan_of(&flow);
        let quarter = plan.step_instant(2, 3).ok_or("pas")?.0 as f64
            - plan.step_instant(2, 0).ok_or("pas")?.0 as f64;
        let third = plan.step_instant(2, 2).ok_or("pas")?.0 as f64
            - plan.step_instant(2, 0).ok_or("pas")?.0 as f64;
        assert!((third / quarter - expected).abs() < 1e-4, "swing {ratio}");
        let beat = plan.step_instant(2, 6).ok_or("pas")?.0 as f64
            - plan.step_instant(2, 0).ok_or("pas")?.0 as f64;
        assert!((beat / quarter - 2.0).abs() < 1e-6, "les temps ne bougent pas");
    }
    Ok(())
}

#[test]
fn le_jazz_waltz_en_3_4_compte_trois_noires_et_swingue_sur_12_pas() -> TestResult {
    let p = fixture("grille_valse_jazz")?;
    let flow = measures(&p, 5, 48, Setup { bpm: Some(160.0), ..Setup::default() })?;
    let plan = plan_of(&flow);
    let quarter = plan.step_instant(1, 4).ok_or("pas")?.0 - plan.step_instant(1, 0).ok_or("pas")?.0;
    let bar = plan.bar_start(2).ok_or("mesure")?.0 - plan.bar_start(1).ok_or("mesure")?.0;
    assert_eq!(bar, 3 * quarter, "3/4 : trois noires");
    assert_eq!(BarTiming::of(&flow[0]).swing_unit_steps(), Some(4));
    let ride_steps: BTreeSet<usize> = flow
        .iter()
        .flat_map(|m| m.notes_for_track(TrackId::PercussionHigh))
        .filter(|n| n.pitch == RIDE)
        .map(|n| n.start_step)
        .collect();
    assert_eq!(ride_steps, BTreeSet::from([0, 4, 6, 8]));
    let measured = track_swing(&flow, TrackId::PercussionHigh, ride)?;
    // 160 BPM : 0,7213 − 0,000976 × 20.
    assert!((measured - 0.7018).abs() < 0.002, "{measured:.4}");
    Ok(())
}

#[test]
fn le_6_8_garde_ses_douze_pas_droits_meme_sous_un_swing_demande() -> TestResult {
    let p = fixture("grille_six_huit")?;
    for swing in [None, Some(0.7)] {
        let flow = measures(&p, 5, 24, Setup { bpm: Some(100.0), swing, ..Setup::default() })?;
        assert_eq!(BarTiming::of(&flow[0]).swing_unit_steps(), None);
        let plan = plan_of(&flow);
        let quarter =
            (plan.bar_start(2).ok_or("mesure")?.0 - plan.bar_start(1).ok_or("mesure")?.0) / 3;
        let bar = plan.bar_start(2).ok_or("mesure")?.0 - plan.bar_start(1).ok_or("mesure")?.0;
        assert_eq!(bar, 3 * quarter, "6/8 : six croches = trois noires");
        let start = plan.bar_start(4).ok_or("mesure")?.0;
        for step in 0..12u64 {
            let at = plan.step_instant(4, step as u16).ok_or("pas")?.0;
            assert!(at.abs_diff(start + step * bar / 12) <= 1, "pas {step} droit");
        }
        // Deux pulsations de six pas : la pulse est une noire pointée.
        assert_eq!(plan.step_instant(4, 6).ok_or("pas")?.0.abs_diff(start + bar / 2), 0);
        let ride_steps: BTreeSet<usize> = flow
            .iter()
            .flat_map(|m| m.notes_for_track(TrackId::PercussionHigh))
            .filter(|n| n.pitch == RIDE)
            .map(|n| n.start_step)
            .collect();
        assert_eq!(ride_steps, BTreeSet::from([0, 2, 4, 6, 8, 10]));
    }
    Ok(())
}

// --- Arc et phrases sous grille de 12 pas ---------------------------------------------------------

fn pitches(flow: &[Measure], bars: std::ops::Range<usize>, track: TrackId) -> BTreeSet<u8> {
    flow[bars].iter().flat_map(|m| m.notes_for_track(track)).map(|n| n.pitch).collect()
}

fn bass_attacks_per_bar(flow: &[Measure], bars: std::ops::Range<usize>) -> f64 {
    let count = bars.len();
    flow[bars].iter().map(|m| m.notes_for_track(TrackId::Bass).len()).sum::<usize>() as f64
        / count as f64
}

#[test]
fn l_arc_de_chorus_joue_sous_les_grilles_de_3_4_et_de_6_8() -> TestResult {
    for (name, pulses) in [("grille_valse_jazz", 3.0), ("grille_six_huit", 2.0)] {
        let p = fixture(name)?;
        for seed in [1, 2, 3] {
            let flow =
                measures(&p, seed, 48, Setup { bpm: Some(150.0), density: 1, ..Setup::default() })?;
            // Chorus 1 : étape d'exposition, charleston (`hat`) ; chorus 2 et 3 : ride.
            let first = pitches(&flow, 0..12, TrackId::PercussionHigh);
            let body = pitches(&flow, 12..36, TrackId::PercussionHigh);
            assert!(
                !first.contains(&RIDE) && first.contains(&42),
                "{name} graine {seed} : {first:?}"
            );
            assert!(body.contains(&RIDE), "{name} graine {seed} : {body:?}");
            // Deux temps à l'exposition (une attaque toutes les deux pulses), marche ensuite.
            let two = bass_attacks_per_bar(&flow, 0..10);
            let walking = bass_attacks_per_bar(&flow, 12..34);
            assert!(
                two < walking,
                "{name} graine {seed} : {two} puis {walking} attaques par mesure"
            );
            assert!((walking - pulses).abs() < 0.8, "{name} : marche à {walking} par mesure");
        }
    }
    Ok(())
}

/// Indice du motif de comping d'une mesure d'après ses pas d'attaque.
fn motif(measure: &Measure, motifs: &[&[usize]]) -> TestResult<usize> {
    let steps: Vec<usize> = measure
        .notes_for_track(TrackId::Chords)
        .iter()
        .map(|n| n.start_step)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    motifs.iter().position(|m| **m == steps[..]).ok_or_else(|| {
        format!("mesure {} : attaques {steps:?} hors des motifs", measure.index).into()
    })
}

struct Phrase {
    motifs: Vec<usize>,
    at: PhraseAt,
}

/// Vrai si `rest` se découpe en phrases de `bank` posées depuis son début.
fn opens(rest: &[usize], first: bool, bank: &[&Phrase]) -> bool {
    rest.is_empty()
        || bank.iter().any(|phrase| {
            let open = phrase.at == PhraseAt::Any || (first && phrase.at == PhraseAt::PartStart);
            open && rest.starts_with(&phrase.motifs)
                && opens(&rest[phrase.motifs.len()..], false, bank)
        })
}

fn tiles(pass: &[usize], close: PhraseAt, bank: &[&Phrase]) -> bool {
    bank.iter().filter(|phrase| phrase.at == close).any(|phrase| {
        let start = pass.len().saturating_sub(phrase.motifs.len());
        pass.ends_with(&phrase.motifs) && opens(&pass[..start], true, bank)
    })
}

#[test]
fn les_phrases_de_comping_se_posent_sous_les_grilles_de_3_4_et_de_6_8() -> TestResult {
    let waltz: [&[usize]; 6] = [&[0, 8], &[4, 8, 10], &[0, 4, 10], &[4, 10], &[8], &[]];
    let six: [&[usize]; 6] = [&[0, 6], &[2, 6, 10], &[0, 6, 10], &[6, 10], &[6], &[]];
    for (name, motifs) in [("grille_valse_jazz", waltz), ("grille_six_huit", six)] {
        let p = without(&fixture(name)?, &["arc"])?;
        let chords = &p.accompaniment.as_ref().ok_or("accompagnement")?.chords;
        let bank: Vec<Phrase> =
            chords.phrases.iter().map(|x| Phrase { motifs: x.motifs.clone(), at: x.at }).collect();
        let all: Vec<&Phrase> = bank.iter().collect();
        for seed in 1..=8 {
            let flow = measures(&p, seed, 36, Setup { bpm: Some(150.0), ..Setup::default() })?;
            for chorus in 0..3 {
                let pass: Vec<usize> = flow[chorus * 12..(chorus + 1) * 12]
                    .iter()
                    .map(|m| motif(m, &motifs))
                    .collect::<TestResult<_>>()?;
                assert!(tiles(&pass, PhraseAt::PartEnd, &all), "{name} graine {seed} : {pass:?}");
            }
        }
    }
    Ok(())
}

// --- Micro-timing par voix de batterie ------------------------------------------------------------

fn drum_notes(measure: &Measure) -> impl Iterator<Item = &TimelineNote> {
    measure
        .notes_for_track(TrackId::PercussionLow)
        .iter()
        .chain(measure.notes_for_track(TrackId::PercussionHigh))
}

#[test]
fn chaque_voix_de_batterie_a_son_micro_timing() -> TestResult {
    let p = fixture("grille_shuffle_ternaire")?;
    let flow = measures(&p, 2, 12, Setup { bpm: Some(100.0), density: 2, ..Setup::default() })?;
    let offsets = |pitch: u8, on_beat: bool| -> BTreeSet<i16> {
        flow.iter()
            .flat_map(drum_notes)
            .filter(|n| n.pitch == pitch && (n.start_step % 3 == 0) == on_beat)
            .map(|n| n.performance.offset)
            .collect()
    };
    // Ride : +0,045 pas sur le temps, +0,07 hors temps ; pédale : −0,07 ; caisse claire : −0,02 ;
    // grosse caisse : −0,025 ; aucune dispersion (jitter nul).
    assert_eq!(offsets(RIDE, true), BTreeSet::from([45]));
    assert_eq!(offsets(RIDE, false), BTreeSet::from([70]));
    assert_eq!(offsets(44, true), BTreeSet::from([-70]));
    assert_eq!(offsets(SNARE, true), BTreeSet::from([-20]));
    assert_eq!(offsets(36, true), BTreeSet::from([-25]));
    // Un temps où plusieurs voix jouent ensemble : chacune garde le sien.
    let together = flow.iter().find(|m| {
        let on = |pitch: u8| {
            m.notes_for_track(TrackId::PercussionHigh)
                .iter()
                .any(|n| n.pitch == pitch && n.start_step == 3)
        };
        on(RIDE) && on(44) && on(SNARE)
    });
    let measure = together.ok_or("un temps où ride, pédale et caisse claire jouent")?;
    let at = |pitch: u8| {
        measure
            .notes_for_track(TrackId::PercussionHigh)
            .iter()
            .find(|n| n.pitch == pitch && n.start_step == 3)
            .map(|n| n.performance.offset)
    };
    assert_eq!((at(RIDE), at(44), at(SNARE)), (Some(45), Some(-70), Some(-20)));
    Ok(())
}

#[test]
fn un_profil_sans_micro_timing_de_voix_garde_une_attaque_par_pas() -> TestResult {
    let p = without(&fixture("grille_shuffle_ternaire")?, &["performance"])?;
    let flow = measures(&p, 2, 12, Setup { density: 2, ..Setup::default() })?;
    assert!(flow.iter().flat_map(drum_notes).all(|n| n.performance.offset == 0));
    Ok(())
}

// --- Coups probabilistes --------------------------------------------------------------------------

fn ghosts(flow: &[Measure], step: usize) -> Vec<bool> {
    flow[..flow.len() - 1]
        .iter()
        .map(|m| {
            m.notes_for_track(TrackId::PercussionHigh)
                .iter()
                .any(|n| n.pitch == SNARE && n.start_step == step)
        })
        .collect()
}

#[test]
fn un_coup_probabiliste_tombe_a_sa_frequence_et_se_rejoue_par_graine() -> TestResult {
    let p = fixture("grille_shuffle_ternaire")?;
    let setup = Setup { bpm: Some(100.0), density: 2, ..Setup::default() };
    let mut seen = BTreeSet::new();
    let (mut five, mut eleven, mut bars) = (0, 0, 0);
    for seed in 1..=10 {
        let flow = measures(&p, seed, 48, setup)?;
        let again = measures(&p, seed, 48, setup)?;
        assert_eq!(ghosts(&flow, 5), ghosts(&again, 5), "même graine, mêmes coups");
        five += ghosts(&flow, 5).iter().filter(|&&on| on).count();
        eleven += ghosts(&flow, 11).iter().filter(|&&on| on).count();
        bars += 48;
        seen.insert(ghosts(&flow, 5));
    }
    // Probabilités du fichier : 20 % au pas 5, 10 % au pas 11.
    let (f5, f11) = (five as f64 / bars as f64, eleven as f64 / bars as f64);
    assert!((0.15..0.25).contains(&f5), "pas 5 : {f5:.3}");
    assert!((0.06..0.14).contains(&f11), "pas 11 : {f11:.3}");
    assert!(seen.len() >= 9, "dix graines, {} suites de coups distinctes", seen.len());
    // Sans probabilité (densité 1), aucune note fantôme.
    let flow = measures(&p, 1, 48, Setup { bpm: Some(100.0), density: 1, ..Setup::default() })?;
    assert!(ghosts(&flow, 5).iter().all(|&on| !on));
    Ok(())
}

#[test]
fn un_coup_probabiliste_se_rejoue_a_l_identique_apres_un_seek() -> TestResult {
    let p = fixture("grille_shuffle_ternaire")?;
    let setup = Setup { bpm: Some(100.0), density: 2, ..Setup::default() };
    let flow = measures(&p, 9, 48, setup)?;
    for start in [7, 19, 33] {
        let mut g = generator(&p, setup);
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let _ = g.generate_measure(1, &mut rng);
        g.reset_to_initial();
        let mut rng = ChaCha8Rng::seed_from_u64(9);
        g.silent_advance(start, &mut rng);
        for bar in start..=47 {
            let again = g.generate_measure(bar, &mut rng);
            let drums = |m: &Measure| -> Vec<(u8, usize, u8, i16)> {
                let mut all: Vec<_> = TrackId::ALL
                    .iter()
                    .flat_map(|t| m.notes_for_track(*t).iter().map(move |n| (t.channel(), n)))
                    .map(|(channel, n)| (channel, n.start_step, n.pitch, n.performance.offset))
                    .collect();
                all.sort_unstable();
                all
            };
            assert_eq!(
                drums(&again),
                drums(&flow[bar - 1]),
                "mesure {bar} après un seek à {start}"
            );
        }
    }
    Ok(())
}

// --- Clave ----------------------------------------------------------------------------------------

/// Côté de la clave d'une mesure d'après les attaques de ses claves : « trois » aux pas 0, 6, 12 ;
/// « deux » aux pas 4, 8 ; `None` hors clave.
fn clave_side(measure: &Measure) -> Option<bool> {
    let steps: Vec<usize> = measure
        .notes_for_track(TrackId::PercussionHigh)
        .iter()
        .filter(|n| n.pitch == CLAVES)
        .map(|n| n.start_step)
        .collect();
    match steps[..] {
        [0, 6, 12] => Some(true),
        [4, 8] => Some(false),
        _ => None,
    }
}

/// Même côté pour le comping : motif « trois » aux pas 0, 6, 12 ; « deux » aux pas 4, 8, 14.
fn comping_side(measure: &Measure) -> Option<bool> {
    let steps: Vec<usize> = measure
        .notes_for_track(TrackId::Chords)
        .iter()
        .map(|n| n.start_step)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    match steps[..] {
        [0, 6, 12] => Some(true),
        [4, 8, 14] => Some(false),
        _ => None,
    }
}

#[test]
fn les_phrases_s_alignent_sur_la_clave_3_2_ou_2_3_tiree_par_air() -> TestResult {
    let p = fixture("grille_clave")?;
    let mut first_sides = BTreeSet::new();
    for seed in 1..=16 {
        let flow =
            measures(&p, seed, 48, Setup { bpm: Some(110.0), density: 1, ..Setup::default() })?;
        let sides: Vec<Option<bool>> = flow[..48].iter().map(clave_side).collect();
        assert!(sides.iter().all(Option::is_some), "graine {seed} : mesure hors clave");
        // La clave est un cycle de deux mesures : « trois » et « deux » alternent, d'un bout à
        // l'autre de l'air, chorus compris.
        for pair in sides.windows(2) {
            assert_ne!(pair[0], pair[1], "graine {seed} : {sides:?}");
        }
        // Le comping est sur le même côté que les claves, mesure par mesure.
        for (n, measure) in flow[..48].iter().enumerate() {
            assert_eq!(comping_side(measure), sides[n], "graine {seed} mesure {}", n + 1);
        }
        first_sides.insert(sides[0]);
    }
    assert_eq!(first_sides.len(), 2, "16 graines : des claves 3-2 et des claves 2-3");
    Ok(())
}

#[test]
fn sans_contrainte_de_clave_les_phrases_ne_s_alignent_plus() -> TestResult {
    let p = fixture("grille_clave")?;
    let free = edited(&p, |v| {
        root(v)?.remove("clave");
        let accompaniment = v
            .get_mut("accompaniment")
            .and_then(|a| a.get_mut("chords"))
            .and_then(toml::Value::as_table_mut)
            .ok_or("chords")?;
        for phrase in
            accompaniment.get_mut("phrases").and_then(toml::Value::as_array_mut).ok_or("phrases")?
        {
            phrase.as_table_mut().ok_or("phrase")?.remove("clave");
        }
        let percussion =
            v.get_mut("percussion").and_then(toml::Value::as_table_mut).ok_or("percussion")?;
        for phrase in
            percussion.get_mut("phrases").and_then(toml::Value::as_array_mut).ok_or("phrases")?
        {
            phrase.as_table_mut().ok_or("phrase")?.remove("clave");
        }
        Ok(())
    })?;
    let mut misaligned = 0;
    for seed in 1..=16 {
        let flow =
            measures(&free, seed, 48, Setup { bpm: Some(110.0), density: 1, ..Setup::default() })?;
        misaligned += flow[..48].iter().filter(|m| comping_side(m) != clave_side(m)).count();
    }
    assert!(misaligned > 100, "témoin : {misaligned} mesures où comping et claves divergent");
    Ok(())
}

#[test]
fn le_comping_et_la_basse_anticipent_le_prochain_accord() -> TestResult {
    let p = fixture("grille_clave")?;
    let flow = measures(&p, 3, 36, Setup { bpm: Some(110.0), density: 1, ..Setup::default() })?;
    let mut comping_anticipations = 0;
    let mut bass_anticipations = 0;
    let mut changes = 0;
    for pair in flow[..36].windows(2) {
        let (now, next) = (&pair[0], &pair[1]);
        if now.chord_context.root_pc == next.chord_context.root_pc {
            continue;
        }
        changes += 1;
        // Basse : la relance du gabarit joue la fondamentale de l'accord suivant au pas 14.
        let pickup = now
            .notes_for_track(TrackId::Bass)
            .iter()
            .find(|n| n.start_step == 14)
            .ok_or_else(|| format!("mesure {} : pas de relance de basse", now.index))?;
        assert_eq!(pickup.pitch % 12, next.chord_context.root_pc, "mesure {}", now.index);
        bass_anticipations += 1;
        // Comping : l'attaque anticipée du pas 14 joue l'accord suivant, celui que la mesure
        // suivante attaque (mêmes classes de hauteur), et non celui de la mesure qui s'achève.
        let pcs = |measure: &Measure, step: Option<usize>| -> BTreeSet<u8> {
            let notes = measure.notes_for_track(TrackId::Chords);
            let at = step.or_else(|| notes.iter().map(|n| n.start_step).min());
            notes.iter().filter(|n| Some(n.start_step) == at).map(|n| n.pitch % 12).collect()
        };
        let anticipated = pcs(now, Some(14));
        if !anticipated.is_empty() {
            assert_eq!(anticipated, pcs(next, None), "mesure {}", now.index);
            assert_ne!(anticipated, pcs(now, Some(4)), "mesure {}", now.index);
            comping_anticipations += 1;
        }
    }
    assert!(changes >= 6, "{changes} changements d'accord");
    assert_eq!(bass_anticipations, changes);
    assert!(comping_anticipations >= 1, "au moins une anticipation du comping");
    Ok(())
}

// --- Schéma ---------------------------------------------------------------------------------------

#[test]
fn les_erreurs_de_schema_nomment_le_champ() -> TestResult {
    let p = fixture("grille_clave")?;
    let swung = fixture("grille_valse_jazz")?;
    let error = |profile: Result<StyleProfile, Box<dyn std::error::Error>>| -> String {
        match profile {
            Ok(_) => String::new(),
            Err(e) => e.to_string(),
        }
    };
    // Côté de clave sans section [clave].
    let no_clave = edited(&p, |v| {
        root(v)?.remove("clave");
        Ok(())
    });
    assert!(error(no_clave).contains("clave"), "[clave] requise");
    // Probabilité hors de ]0, 1].
    let text = std::fs::read_to_string(format!(
        "{}/../tests/fixtures/profiles/grille_clave/percussion.toml",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let bad = text.replacen("probability = 0.25", "probability = 1.5", 1);
    assert!(toml::from_str::<harmonium_core::compose::PercussionTable>(&bad).is_err());
    // Courbes de swing : tempos non croissants, rapport hors plage, écart à la référence.
    for (points, field) in [
        ("[[100.0, 0.7], [100.0, 0.6]]", "swing.tracks.Bass.points"),
        ("[[100.0, 0.9]]", "swing.tracks.Bass.points[0]"),
        ("[[100.0, 0.5]]", "swing.tracks.Bass"),
    ] {
        let bad = edited(&swung, |v| {
            let tracks = v
                .get_mut("swing")
                .and_then(|s| s.get_mut("tracks"))
                .and_then(toml::Value::as_table_mut)
                .ok_or("swing.tracks")?;
            tracks.insert("Bass".into(), toml::from_str(&format!("points = {points}"))?);
            Ok(())
        });
        let message = error(bad);
        assert!(message.contains(field), "{points} : {message}");
    }
    // Micro-timing de voix : une seule entrée par note.
    let dup = edited(&fixture("grille_shuffle_ternaire")?, |v| {
        let voices = v
            .get_mut("performance")
            .and_then(|p| p.get_mut("tracks"))
            .and_then(|t| t.get_mut("PercussionHigh"))
            .and_then(|t| t.get_mut("voices"))
            .and_then(toml::Value::as_array_mut)
            .ok_or("voices")?;
        let first = voices[0].clone();
        voices.push(first);
        Ok(())
    });
    assert!(error(dup).contains("voices[3].note"));
    Ok(())
}

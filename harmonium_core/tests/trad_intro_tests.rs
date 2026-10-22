//! Tests d'acceptation des intros idiomatiques (P1-T11) sur le flux de snapshots :
//! formes d'intro, levée (anacrouse), déterminisme, `each_tune`, validation.

use harmonium_core::{
    params::{MusicalParams, Style},
    report::{MeasureSnapshot, NoteSnapshot},
    trad::{
        IntroConfig, IntroForm, IntroRole, ReelArrangement, Section, TradGenerator, TunePosition,
    },
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn reel_params() -> MusicalParams {
    let rf = TuningParams::default().reel_fiddle;
    let mut params = MusicalParams::default();
    params.set_style(Style::Reel);
    params.bpm = rf.default_bpm;
    params.set_swing_ratio(rf.default_swing_ratio);
    params.time_signature = rf.default_time_signature;
    params
}

fn stream_with(seed: u64, bars: usize, tuning: TuningParams) -> Vec<MeasureSnapshot> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut trad_gen = TradGenerator::new(reel_params(), tuning);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&trad_gen.generate_measure(bar, &mut rng)))
        .collect()
}

/// Tuning avec une seule forme d'intro (pour tester chaque forme isolément).
fn tuning_single_form(role: IntroRole, bars: usize, pickup_eighths: u8) -> TuningParams {
    let mut tuning = TuningParams::default();
    tuning.reel_arrangement.intro = IntroConfig {
        forms: vec![IntroForm { role, weight: 1, bars }],
        pickup_eighths,
        each_tune: false,
    };
    tuning
}

fn track<'a>(
    snap: &'a MeasureSnapshot,
    t: harmonium_core::timeline::TrackId,
) -> Vec<&'a NoteSnapshot> {
    snap.notes.iter().filter(|n| n.track == t.channel()).collect()
}

fn has(snap: &MeasureSnapshot, t: harmonium_core::timeline::TrackId) -> bool {
    !track(snap, t).is_empty()
}

fn bar(snaps: &[MeasureSnapshot], b: usize) -> &MeasureSnapshot {
    &snaps[b - 1]
}

fn position(snaps: &[MeasureSnapshot], b: usize) -> TunePosition {
    bar(snaps, b).position.expect("position absente")
}

use harmonium_core::timeline::TrackId;

// ---------------------------------------------------------------------------
// Formes
// ---------------------------------------------------------------------------

#[test]
fn percussion_intro_has_heel_only() {
    let snaps = stream_with(1, 3, tuning_single_form(IntroRole::Percussion, 1, 0));
    // 1 mesure d'intro : percussion (talon) seul
    assert!(has(bar(&snaps, 1), TrackId::Heel), "talon attendu dans l'intro");
    assert!(!has(bar(&snaps, 1), TrackId::Toe), "pas de pointe dans l'intro percussion");
    assert!(!has(bar(&snaps, 1), TrackId::Melody), "pas de violon dans percussion (levée 0)");
    assert!(!has(bar(&snaps, 1), TrackId::Chords), "pas de guitare dans percussion");
    // L'air commence à la mesure 2 (A1)
    let p = position(&snaps, 2);
    assert_eq!(p.section, Section::Tour);
    assert_eq!((p.tour, p.part, p.pass, p.bar_in_strain), (1, harmonium_core::trad::Part::A, 1, 1));
}

#[test]
fn tail_intro_has_melody_only() {
    let snaps = stream_with(1, 4, tuning_single_form(IntroRole::Tail, 2, 0));
    // 2 mesures d'intro : violon seul (les 2 dernières mesures de l'air)
    for b in 1..=2 {
        assert!(has(bar(&snaps, b), TrackId::Melody), "m.{b}: violon attendu");
        assert!(!has(bar(&snaps, b), TrackId::Chords));
        assert!(!has(bar(&snaps, b), TrackId::Bass));
        assert!(!has(bar(&snaps, b), TrackId::Heel));
    }
    // L'air commence à la mesure 3
    assert_eq!(position(&snaps, 3).section, Section::Tour);
}

#[test]
fn vamp_intro_has_chords_only() {
    let snaps = stream_with(1, 4, tuning_single_form(IntroRole::Vamp, 2, 0));
    for b in 1..=2 {
        assert!(has(bar(&snaps, b), TrackId::Chords), "m.{b}: guitare attendue");
        assert!(!has(bar(&snaps, b), TrackId::Melody));
        assert!(!has(bar(&snaps, b), TrackId::Heel));
    }
    assert_eq!(position(&snaps, 3).section, Section::Tour);
}

/// Écoute 2026-10-22 : le décompte aux pieds s'arrêtait quand le violon entrait.
/// Une piste qui a lancé l'air continue pendant le premier tour, même si
/// l'arrangement la fait normalement entrer plus tard.
#[test]
fn intro_instruments_keep_playing_into_the_tune() {
    let snaps = stream_with(1, 10, tuning_single_form(IntroRole::Percussion, 1, 0));
    for b in 2..=10 {
        assert!(has(bar(&snaps, b), TrackId::Heel), "m.{b}: le talon du décompte doit continuer");
    }
    let snaps = stream_with(1, 12, tuning_single_form(IntroRole::Vamp, 2, 0));
    for b in 3..=12 {
        assert!(has(bar(&snaps, b), TrackId::Chords), "m.{b}: la guitare du vamp doit continuer");
    }
}

// ---------------------------------------------------------------------------
// Déterminisme et décalage
// ---------------------------------------------------------------------------

#[test]
fn same_seed_same_intro() {
    let t = TuningParams::default();
    let a = stream_with(42, 3, t.clone());
    let b = stream_with(42, 3, t);
    for i in 0..3 {
        assert_eq!(
            serde_json::to_string(&a[i]).unwrap(),
            serde_json::to_string(&b[i]).unwrap(),
            "m.{}",
            i + 1
        );
    }
}

#[test]
fn melody_after_intro_identical_to_no_intro() {
    // Même graine : les notes Melody de l'air (après l'intro) sont identiques
    // à celles d'un air sans intro.
    let flat_tuning = TuningParams {
        reel_arrangement: ReelArrangement::from_start(3),
        ..TuningParams::default()
    };
    let intro_tuning = TuningParams::default();

    let flat = stream_with(7, 32, flat_tuning);
    // Générer intro + 32 mesures, puis lire intro_bars via la première mesure Tour
    let mut rng = ChaCha8Rng::seed_from_u64(7);
    let mut trad_gen = TradGenerator::new(reel_params(), intro_tuning.clone());
    for b in 1..=40 {
        let _ = trad_gen.generate_measure(b, &mut rng);
    }
    // Trouver la première mesure Tour (fin de l'intro)
    let intro_bars = {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let mut g = TradGenerator::new(reel_params(), intro_tuning);
        let mut n = 0usize;
        for b in 1..=8 {
            let m = g.generate_measure(b, &mut rng);
            if m.position.map(|p| p.section == Section::Tour).unwrap_or(false) {
                n = b - 1;
                break;
            }
        }
        n
    };

    let mut rng2 = ChaCha8Rng::seed_from_u64(7);
    let mut trad_gen2 = TradGenerator::new(reel_params(), TuningParams::default());
    for tune_bar in 1..=32 {
        let m = trad_gen2.generate_measure(tune_bar + intro_bars, &mut rng2);
        let mel: Vec<u8> = m.notes_for_track(TrackId::Melody).iter().map(|n| n.pitch).collect();
        let flat_mel: Vec<u8> =
            track(&flat[tune_bar - 1], TrackId::Melody).iter().map(|n| n.pitch).collect();
        assert_eq!(flat_mel, mel, "mélodie mesure {tune_bar} après intro");
    }
}

// ---------------------------------------------------------------------------
// Levée (anacrouse)
// ---------------------------------------------------------------------------

#[test]
fn pickup_ascends_to_first_a1_note_with_small_leaps() {
    // percussion (1 mesure, pas de violon) + levée 2 : la seule mélodie
    // de l'intro est la levée.
    let tuning = tuning_single_form(IntroRole::Percussion, 1, 2);
    let snaps = stream_with(3, 6, tuning);

    // La mesure d'intro est la 1 ; l'air commence à la mesure 2.
    assert_eq!(position(&snaps, 1).section, Section::Intro);
    assert_eq!(position(&snaps, 2).section, Section::Tour);

    let pickup: Vec<u8> = track(bar(&snaps, 1), TrackId::Melody).iter().map(|n| n.pitch).collect();
    let first_note_a1: Vec<u8> =
        track(bar(&snaps, 2), TrackId::Melody).iter().map(|n| n.pitch).take(1).collect();

    assert!(!pickup.is_empty(), "la levée doit exister (pickup_eighths = 2)");

    // La dernière note de levée monte vers la première note de A1 (même pitch class)
    let last_pickup = *pickup.last().unwrap();
    let target = *first_note_a1.first().unwrap();
    assert_eq!(last_pickup % 12, target % 12, "la levée doit arriver sur la 1ʳᵉ note de A1");

    // Sauts ≤ quinte (7 demi-tons) et ascendante
    let mut prev = pickup[0];
    for &m in &pickup[1..] {
        assert!((m as i16 - prev as i16).abs() <= 7, "saut > quinte: {prev}→{m}");
        assert!(m >= prev, "levée non ascendante");
        prev = m;
    }
}

// ---------------------------------------------------------------------------
// each_tune
// ---------------------------------------------------------------------------

#[test]
fn each_tune_false_second_tune_has_pickup_only() {
    // Par défaut each_tune = false : seul le premier air a une intro complète.
    let tuning = TuningParams::default();
    // Générer jusqu'au 2ᵉ air (intro + 96 mesures + marge)
    let snaps = stream_with(11, 110, tuning);

    // Localiser le début du 2ᵉ air : première mesure Tour avec tune_index = 1
    let second_tune_start = (2..=110)
        .find(|&b| {
            let p = position(&snaps, b);
            p.tune_index == 1 && p.section == Section::Tour && p.bar_in_strain == 1
        })
        .expect("début du 2ᵉ air");

    // La mesure juste avant est soit la dernière du tune 0, soit une levée seule.
    let prev = position(&snaps, second_tune_start - 1);
    assert!(prev.tune_index == 0 || prev.section == Section::Intro);
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[test]
fn invalid_intro_tables_are_rejected() {
    // pickup_eighths hors limites
    let mut tuning = TuningParams::default();
    tuning.reel_arrangement.intro.pickup_eighths = 4;
    let errs = tuning.validate().err().expect("validation doit échouer");
    assert!(errs.iter().any(|e| e.field == "reel_arrangement"), "erreur de validation attendue");

    // bars hors limites
    let mut tuning = TuningParams::default();
    tuning.reel_arrangement.intro.forms[0].bars = 10;
    assert!(tuning.validate().is_err());

    // weight nul
    let mut tuning = TuningParams::default();
    tuning.reel_arrangement.intro.forms[0].weight = 0;
    assert!(tuning.validate().is_err());
}

#[test]
fn end_after_tour_ends_last_measure_with_tonic() {
    use harmonium_core::timeline::TrackId;

    // Tuning sans intro pour un calcul simple : 3 tours = 96 mesures.
    let mut tuning = TuningParams::default();
    tuning.reel_arrangement.intro = IntroConfig::none();
    tuning.reel_arrangement.tours_per_tune = 3;

    let mut params = reel_params();
    params.end_after_tour = true;

    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let mut g = TradGenerator::new(params, tuning);
    let snaps: Vec<MeasureSnapshot> =
        (1..=97).map(|b| MeasureSnapshot::from_measure(&g.generate_measure(b, &mut rng))).collect();

    // La dernière mesure du dernier tour (mesure 96) est une fin : tonique
    // accentuée au pas 0, puis silence.
    let fin = bar(&snaps, 96);
    assert_eq!(fin.position.unwrap().is_last_tour(), true);
    for t in [TrackId::Melody, TrackId::Chords, TrackId::Bass, TrackId::Heel] {
        assert!(has(fin, t), "{t:?} attendu dans la fin");
    }
    // Une seule note par piste (tonique), au pas 0.
    let mel = track(fin, TrackId::Melody);
    assert_eq!(mel.len(), 1, "une seule note de violon dans la fin");
    assert_eq!(mel[0].start_step, 0);

    // Après la fin (mesure 97), plus de nouvel air : mesure vide.
    let after = bar(&snaps, 97);
    assert!(!has(after, TrackId::Melody), "pas de nouvel air après end_after_tour");
}

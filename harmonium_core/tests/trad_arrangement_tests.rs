//! Tests d'acceptation de l'arrangement du reel (D11) sur le flux de snapshots :
//! tours d'un air, entrées progressives des pistes, plafonds `enable_*`, position exposée.

use harmonium_core::{
    params::{MusicalParams, Style},
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::{GenerationContext, TrackId},
    trad::{
        ArrangementError, Part, REEL_ARRANGEMENT_TOML, ReelArrangement, TradGenerator, TunePosition,
    },
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_music_theory::{note::PitchSymbol, scale::ScaleType};

const SEEDS: [u64; 5] = [1, 2, 3, 7, 11];

fn reel_params() -> MusicalParams {
    let rf = TuningParams::default().reel_fiddle;
    let mut params = MusicalParams::default();
    params.set_style(Style::Reel);
    params.bpm = rf.default_bpm;
    params.set_swing_ratio(rf.default_swing_ratio);
    params.time_signature = rf.default_time_signature;
    params
}

fn stream_with(
    seed: u64,
    bars: usize,
    params: MusicalParams,
    tuning: TuningParams,
) -> Vec<MeasureSnapshot> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut trad_gen = TradGenerator::new(params, tuning);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&trad_gen.generate_measure(bar, &mut rng)))
        .collect()
}

fn stream(seed: u64, bars: usize) -> Vec<MeasureSnapshot> {
    stream_with(seed, bars, reel_params(), TuningParams::default())
}

fn track(snap: &MeasureSnapshot, track: TrackId) -> Vec<&NoteSnapshot> {
    snap.notes.iter().filter(|n| n.track == track.channel()).collect()
}

fn has(snap: &MeasureSnapshot, t: TrackId) -> bool {
    !track(snap, t).is_empty()
}

/// Mesure 1-based `bar` du flux.
fn bar(snaps: &[MeasureSnapshot], bar: usize) -> &MeasureSnapshot {
    &snaps[bar - 1]
}

fn position(snaps: &[MeasureSnapshot], b: usize) -> Result<TunePosition, String> {
    bar(snaps, b).position.ok_or_else(|| format!("m.{b}: position absente"))
}

fn melody_tuples(snap: &MeasureSnapshot) -> Vec<(u8, usize, usize, u8)> {
    track(snap, TrackId::Melody)
        .iter()
        .map(|n| (n.pitch, n.start_step, n.duration_steps, n.velocity))
        .collect()
}

#[test]
fn default_entries_violin_alone_then_accompaniment_then_feet() {
    for seed in SEEDS {
        let snaps = stream(seed, 97);
        for b in 1..=8 {
            let s = bar(&snaps, b);
            assert!(has(s, TrackId::Melody), "seed {seed} m.{b}: violon attendu");
            for t in [TrackId::Chords, TrackId::Bass, TrackId::Heel, TrackId::Toe] {
                assert!(!has(s, t), "seed {seed} m.{b}: {t:?} ne doit pas jouer");
            }
        }
        for b in 9..=32 {
            let s = bar(&snaps, b);
            assert!(has(s, TrackId::Chords) && has(s, TrackId::Bass), "seed {seed} m.{b}");
            assert!(!has(s, TrackId::Heel) && !has(s, TrackId::Toe), "seed {seed} m.{b}");
        }
        for b in 33..=96 {
            let s = bar(&snaps, b);
            for t in TrackId::ALL {
                assert!(has(s, t), "seed {seed} m.{b}: {t:?} attendu");
            }
        }
        // À partir du 2e air : tout le monde dès A1 (intro_each_tune = false).
        for t in TrackId::ALL {
            assert!(has(bar(&snaps, 97), t), "seed {seed} m.97: {t:?} attendu");
        }
    }
}

#[test]
fn tours_replay_the_same_melody_and_the_next_tune_starts_at_bar_97() {
    for seed in SEEDS {
        let snaps = stream(seed, 97);
        for b in 1..=32 {
            for tour in [1usize, 2] {
                assert_eq!(
                    melody_tuples(bar(&snaps, b)),
                    melody_tuples(bar(&snaps, b + 32 * tour)),
                    "seed {seed}: m.{b} != m.{} (tour {})",
                    b + 32 * tour,
                    tour + 1
                );
            }
        }
        // Harmonie identique d'un tour à l'autre.
        for b in 1..=32 {
            assert_eq!(bar(&snaps, b).chord_name, bar(&snaps, b + 32).chord_name, "seed {seed}");
        }
    }
    // Le tirage du nouvel air change la mélodie (au moins un seed sur ceux testés).
    let changed = SEEDS.iter().any(|&seed| {
        let snaps = stream(seed, 97);
        (1..=8).any(|b| melody_tuples(bar(&snaps, b)) != melody_tuples(bar(&snaps, b + 96)))
    });
    assert!(changed, "la mesure 97 doit ouvrir un nouvel air");
}

#[test]
fn later_tours_do_not_consume_the_rng() {
    // Même graine : le 2e air est identique que l'air 1 dure 3 tours ou qu'on saute
    // directement à la mesure 97 par silent_advance.
    for seed in SEEDS {
        let linear = stream(seed, 100);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut sought = TradGenerator::new(reel_params(), TuningParams::default());
        sought.reset_to_initial(&GenerationContext {
            session_seed: seed,
            key: PitchSymbol::D,
            scale: ScaleType::Diatonic,
            key_pc: 2,
        });
        sought.silent_advance(97, &mut rng);
        for b in 97..=100 {
            let snap = MeasureSnapshot::from_measure(&sought.generate_measure(b, &mut rng));
            assert_eq!(
                serde_json::to_string(&snap).unwrap_or_default(),
                serde_json::to_string(bar(&linear, b)).unwrap_or_default(),
                "seed {seed} m.{b}"
            );
        }
    }
}

#[test]
fn invalid_arrangement_tables_are_rejected_with_explicit_errors() {
    let with = |from: &str, to: &str| {
        assert!(REEL_ARRANGEMENT_TOML.contains(from), "table embarquée sans « {from} »");
        ReelArrangement::from_toml(&REEL_ARRANGEMENT_TOML.replace(from, to))
    };
    assert_eq!(
        with("Heel = { tour = 2", "Heel = { tour = 0"),
        Err(ArrangementError::TourOutOfRange { track: TrackId::Heel, tour: 0 })
    );
    assert_eq!(
        with("Toe = { tour = 2, part = \"A\"", "Toe = { tour = 2, part = \"C\""),
        Err(ArrangementError::UnknownPart { track: TrackId::Toe, part: "C".to_owned() })
    );
    assert_eq!(
        with("Toe = {", "Cymbale = {"),
        Err(ArrangementError::UnknownTrack("Cymbale".to_owned()))
    );
    assert_eq!(
        with("tours_per_tune = 3", "tours_per_tune = 0"),
        Err(ArrangementError::ToursPerTuneOutOfRange(0))
    );

    let mut tuning = TuningParams::default();
    tuning.reel_arrangement.entries.heel.tour = 0;
    let errs = tuning.validate().err().unwrap_or_default();
    assert!(errs.iter().any(|e| e.field == "reel_arrangement" && e.message.contains("Heel")));
}

#[test]
fn game_flags_cap_the_arrangement() {
    let ceiling = |edit: fn(&mut MusicalParams)| {
        let mut params = reel_params();
        edit(&mut params);
        stream_with(1, 97, params, TuningParams::default())
    };
    let no_rhythm = ceiling(|p| p.enable_rhythm = false);
    let no_harmony = ceiling(|p| p.enable_harmony = false);
    let no_melody = ceiling(|p| p.enable_melody = false);
    for b in 1..=97 {
        assert!(!has(bar(&no_rhythm, b), TrackId::Heel) && !has(bar(&no_rhythm, b), TrackId::Toe));
        assert!(!has(bar(&no_harmony, b), TrackId::Chords));
        assert!(!has(bar(&no_harmony, b), TrackId::Bass));
        assert!(!has(bar(&no_melody, b), TrackId::Melody));
    }
    // Les autres pistes suivent toujours l'arrangement.
    assert!(has(bar(&no_rhythm, 40), TrackId::Bass) && has(bar(&no_rhythm, 40), TrackId::Melody));
    assert!(has(bar(&no_harmony, 40), TrackId::Heel));
    assert!(!has(bar(&no_melody, 1), TrackId::Melody) && has(bar(&no_melody, 40), TrackId::Toe));

    // feet_density 0 : plus de pointe, le talon reste.
    let mut params = reel_params();
    params.set_feet_density(0);
    let snaps = stream_with(1, 40, params, TuningParams::default());
    assert!(has(bar(&snaps, 40), TrackId::Heel) && !has(bar(&snaps, 40), TrackId::Toe));
}

#[test]
fn exposed_position_matches_bar_numbers() -> Result<(), String> {
    let snaps = stream(1, 97);
    for b in 1..=snaps.len() {
        let pos = position(&snaps, b)?;
        let in_tune = (b - 1) % 96;
        let in_tour = in_tune % 32;
        assert_eq!(pos.tune_index, (b - 1) / 96, "m.{b}");
        assert_eq!(pos.tours, 3, "m.{b}");
        assert_eq!(pos.tour, in_tune / 32 + 1, "m.{b}");
        assert_eq!(pos.part, if in_tour < 16 { Part::A } else { Part::B }, "m.{b}");
        assert_eq!(pos.pass, (in_tour % 16) / 8 + 1, "m.{b}");
        assert_eq!(pos.bar_in_strain, in_tour % 8 + 1, "m.{b}");
        assert_eq!(pos.is_last_tour(), pos.tour == 3, "m.{b}");
    }
    // Repères lisibles par le gameplay : les pieds entrent à la mesure 33 (tour 2, A, passe 1).
    let first_feet = snaps.iter().position(|s| has(s, TrackId::Heel)).map(|i| i + 1);
    assert_eq!(first_feet, Some(33));
    let pos = position(&snaps, 33)?;
    assert_eq!((pos.tour, pos.part, pos.pass, pos.bar_in_strain), (2, Part::A, 1, 1));
    assert!(position(&snaps, 65)?.is_last_tour());
    Ok(())
}

#[test]
fn intro_each_tune_replays_the_progressive_entries() {
    let mut tuning = TuningParams::default();
    tuning.reel_arrangement.intro_each_tune = true;
    let snaps = stream_with(1, 106, reel_params(), tuning);
    for t in [TrackId::Chords, TrackId::Bass, TrackId::Heel, TrackId::Toe] {
        assert!(!has(bar(&snaps, 97), t), "{t:?} ne doit pas jouer à la mesure 97");
    }
    assert!(has(bar(&snaps, 97), TrackId::Melody));
    assert!(has(bar(&snaps, 106), TrackId::Bass));
}

#[test]
fn tours_per_tune_controls_the_tune_length() -> Result<(), String> {
    let mut tuning = TuningParams::default();
    tuning.reel_arrangement.tours_per_tune = 1;
    let snaps = stream_with(1, 33, reel_params(), tuning);
    let pos = position(&snaps, 33)?;
    assert_eq!((pos.tune_index, pos.tour, pos.tours), (1, 1, 1));
    // Un seul tour : l'entrée « tour 2 » des pieds est ramenée au dernier tour (le 1er).
    assert!(has(bar(&snaps, 1), TrackId::Heel));
    Ok(())
}

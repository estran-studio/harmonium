//! Raccords, clôture et entrées observés sur le flux public de snapshots.
use harmonium_core::{
    StyleProfile, TuningParams,
    params::MusicalParams,
    report::MeasureSnapshot,
    trad::{
        TradGenerator,
        arrangement::{IntroForm, IntroRole},
        set::{SetGap, SetLength, SetTransition, TransitionKind},
    },
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn tuning(kind: TransitionKind) -> TuningParams {
    let mut profile = StyleProfile::default();
    profile.arrangement.tours_per_tune = 2;
    profile.arrangement.intro.forms =
        vec![IntroForm { role: IntroRole::Percussion, weight: 1, bars: 1 }];
    profile.set.signature = Default::default();
    profile.set.length = vec![SetLength { tunes: 2, weight: 1 }];
    profile.set.transitions = vec![SetTransition {
        kind,
        weight: 1,
        bars: matches!(kind, TransitionKind::Call | TransitionKind::Silence).then_some(2),
    }];
    profile.set.end = Default::default();
    profile.performance = Default::default();
    profile.ornaments = Default::default();
    TuningParams { profile }
}

fn stream(tuning: &TuningParams) -> Vec<MeasureSnapshot> {
    let mut generator = TradGenerator::new(MusicalParams::default(), tuning.clone());
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    (1..=330)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect()
}

#[test]
fn transitions_keep_the_grid_and_separate_call_silence_and_pickup() {
    for kind in [
        TransitionKind::Direct,
        TransitionKind::Call,
        TransitionKind::Held,
        TransitionKind::Silence,
    ] {
        let snapshots = stream(&tuning(kind));
        assert!(snapshots.iter().enumerate().all(|(i, s)| s.index == i + 1));
        let intro: Vec<_> = snapshots
            .iter()
            .filter(|s| s.position.is_some_and(|p| p.tune_index == 1 && p.is_intro()))
            .collect();
        let prelude =
            usize::from(matches!(kind, TransitionKind::Call | TransitionKind::Silence)) * 2;
        assert_eq!(intro.len(), prelude + 1, "{kind:?}");
        for s in &intro[..prelude] {
            if kind == TransitionKind::Silence {
                assert!(s.notes.is_empty());
            } else {
                assert!(s.notes.iter().any(|n| n.track == 3));
                assert!(s.notes.iter().all(|n| n.track >= 3));
            }
        }
        let pickup = intro.last().unwrap();
        assert_eq!(pickup.notes.len(), 2);
        assert!(pickup.notes.iter().all(|n| n.track == 0 && n.start_step >= 12));
        let previous =
            snapshots.iter().rev().find(|s| s.position.is_some_and(|p| p.tune_index == 0)).unwrap();
        if kind == TransitionKind::Held {
            let melody: Vec<_> = previous.notes.iter().filter(|n| n.track == 0).collect();
            assert_eq!(melody.len(), 1);
            assert_eq!(melody[0].start_step, 0);
            assert_eq!(melody[0].pitch % 12, previous.position.unwrap().key);
        }
        assert!(
            snapshots
                .iter()
                .any(|s| s.position.is_some_and(|p| p.tune_index == 2 && !p.is_intro()))
        );
    }
}

#[test]
fn set_end_has_its_own_outro_gap_and_progressive_entries() {
    let mut config = tuning(TransitionKind::Direct);
    config.profile.set.end.outro = true;
    config.profile.set.end.gap_bars = vec![SetGap { bars: 2, weight: 1 }];
    let snapshots = stream(&config);
    let last = |tune| {
        snapshots.iter().rev().find(|s| s.position.is_some_and(|p| p.tune_index == tune)).unwrap()
    };
    assert!(last(0).notes.iter().filter(|n| n.track == 0).count() > 1);
    let outro = last(1);
    assert_eq!(outro.notes.iter().filter(|n| n.track == 0).count(), 1);
    assert!(outro.notes.iter().all(|n| n.start_step == 0));
    let intro: Vec<_> = snapshots
        .iter()
        .filter(|s| s.position.is_some_and(|p| p.tune_index == 2 && p.is_intro()))
        .collect();
    assert_eq!(intro.len(), 3);
    assert!(intro[..2].iter().all(|s| s.notes.is_empty()));
    assert!(intro[2].notes.iter().any(|n| n.track == 3));
    for tune in [0, 2, 4] {
        let first = snapshots
            .iter()
            .find(|s| s.position.is_some_and(|p| p.tune_index == tune && !p.is_intro()))
            .unwrap();
        assert!(
            !first.notes.iter().any(|n| n.track == 2),
            "la basse attend son entrée à chaque set"
        );
    }
    for tune in [1, 3] {
        let first = snapshots
            .iter()
            .find(|s| s.position.is_some_and(|p| p.tune_index == tune && !p.is_intro()))
            .unwrap();
        assert!(first.notes.iter().any(|n| n.track == 2));
        assert!(first.notes.iter().any(|n| n.track == 3));
    }
}

#[test]
fn transition_and_gap_validation_reports_the_field() {
    let mut config = tuning(TransitionKind::Call);
    config.profile.set.transitions[0].bars = Some(0);
    assert_eq!(config.profile.validate().unwrap_err().field, "set.transitions[0].bars");
    config.profile.set.transitions[0].bars = Some(1);
    config.profile.set.transitions[0].weight = 0;
    assert_eq!(config.profile.validate().unwrap_err().field, "set.transitions");
    config.profile.set.transitions[0].weight = 1;
    config.profile.set.end.gap_bars = vec![SetGap { bars: 5, weight: 1 }];
    assert_eq!(config.profile.validate().unwrap_err().field, "set.end.gap_bars[0].bars");
}

#[test]
fn game_stop_finishes_the_current_tune_even_after_the_first_set() {
    let config = tuning(TransitionKind::Direct);
    let mut generator = TradGenerator::new(MusicalParams::default(), config);
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    let mut bar = 1;
    loop {
        let m = generator.generate_measure(bar, &mut rng);
        if m.position.is_some_and(|p| p.tune_index == 2 && !p.is_intro()) {
            break;
        }
        bar += 1;
    }
    // Le compositeur a déjà écrit plusieurs airs d'avance quand le jeu demande l'arrêt.
    for future in bar + 1..=bar + 200 {
        let _ = generator.generate_measure(future, &mut rng);
    }
    let mut params = generator.musical_params.clone();
    params.end_after_tour = true;
    generator.update_params(params);
    generator.invalidate_future(bar);
    let tail: Vec<_> = (bar + 1..bar + 140)
        .map(|b| MeasureSnapshot::from_measure(&generator.generate_measure(b, &mut rng)))
        .collect();
    let ending = tail.iter().rev().find(|s| s.position.is_some_and(|p| p.tune_index == 2)).unwrap();
    assert_eq!(ending.notes.iter().filter(|n| n.track == 0).count(), 1);
    assert!(
        tail.iter()
            .filter(|s| s.position.is_some_and(|p| p.tune_index > 2))
            .all(|s| s.notes.is_empty())
    );
}

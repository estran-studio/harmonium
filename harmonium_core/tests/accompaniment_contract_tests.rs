//! Contrats d'accompagnement observés au chargement et sur les mesures publiées.
use harmonium_core::{
    StyleProfile, TuningParams, params::MusicalParams, timeline::Measure, trad::TradGenerator,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn stream(profile: &StyleProfile, seed: u64, bars: usize) -> Vec<Measure> {
    profile.validate().unwrap();
    let mut g = TradGenerator::new(
        MusicalParams { bpm: profile.meter.default_bpm, ..Default::default() },
        TuningParams { profile: profile.clone() },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars).map(|bar| g.generate_measure(bar, &mut rng)).collect()
}

#[test]
fn invalid_voice_fields_are_rejected_at_loading_with_their_path() {
    let profile = StyleProfile::default();
    for (field, mutate) in [
        (
            "bass.low",
            (|p: &mut StyleProfile| p.accompaniment.as_mut().unwrap().bass.low = 128)
                as fn(&mut StyleProfile),
        ),
        ("bass.hits[1].step", |p| p.accompaniment.as_mut().unwrap().bass.hits[1].step = 0),
        ("bass.hits[0].duration", |p| p.accompaniment.as_mut().unwrap().bass.hits[0].duration = 0),
        ("bass.hits[0].velocity", |p| {
            p.accompaniment.as_mut().unwrap().bass.hits[0].velocity = 128
        }),
        ("chords.default_pattern", |p| {
            p.accompaniment.as_mut().unwrap().chords.default_pattern = 100
        }),
        ("chords.patterns[1].hits[1].step", |p| {
            p.accompaniment.as_mut().unwrap().chords.patterns[1].hits[1].step = 4
        }),
        ("chords.patterns[0].hits[0].duration", |p| {
            p.accompaniment.as_mut().unwrap().chords.patterns[0].hits[0].duration = 100
        }),
        ("chords.center", |p| p.accompaniment.as_mut().unwrap().chords.center = 128),
    ] {
        let mut bad = profile.clone();
        mutate(&mut bad);
        let error = StyleProfile::from_toml(&bad.to_toml().unwrap()).unwrap_err().to_string();
        assert!(error.contains(&format!("accompaniment.{field}")), "{error}");
    }
    let text = profile.to_toml().unwrap();
    let error = StyleProfile::from_toml(&text.replace("fifth_if_held", "unknown_degree"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("degree"), "{error}");
}

fn arranged() -> StyleProfile {
    let mut p = StyleProfile::default();
    p.arrangement =
        harmonium_core::trad::ReelArrangement::from_start(2, harmonium_core::trad::Part(0));
    p.arrangement.intro = harmonium_core::trad::IntroConfig::none();
    p
}

fn note_values(m: &Measure, track: harmonium_core::timeline::TrackId) -> serde_json::Value {
    let snapshot = harmonium_core::report::MeasureSnapshot::from_measure(m);
    serde_json::to_value(
        snapshot.notes.into_iter().filter(|n| n.track == track as u8).collect::<Vec<_>>(),
    )
    .unwrap()
}

#[test]
fn passing_is_inert_without_positive_rules_or_density_and_draws_nothing() {
    for p in StyleProfile::distributed() {
        let Some(a) = &p.accompaniment else {
            continue;
        };
        if a.bass.passing.is_none() {
            continue;
        }
        let mut absent = p.clone();
        absent.accompaniment.as_mut().unwrap().bass.passing = None;
        let mut zero_weight = p.clone();
        for r in
            &mut zero_weight.accompaniment.as_mut().unwrap().bass.passing.as_mut().unwrap().rules
        {
            r.weight = 0;
        }
        let mut zero_density = p.clone();
        for point in &mut zero_density
            .accompaniment
            .as_mut()
            .unwrap()
            .bass
            .passing
            .as_mut()
            .unwrap()
            .density
            .0
        {
            point[1] = 0.0;
        }
        for seed in [3, 11] {
            let reference = serde_json::to_value(stream(&absent, seed, 100)).unwrap();
            assert_eq!(
                reference,
                serde_json::to_value(stream(&zero_weight, seed, 100)).unwrap(),
                "{}",
                p.meta.id
            );
            assert_eq!(
                reference,
                serde_json::to_value(stream(&zero_density, seed, 100)).unwrap(),
                "{}",
                p.meta.id
            );
        }
    }
}

#[test]
fn passing_keeps_neighbour_tracks_and_strong_bass_unchanged() {
    use harmonium_core::timeline::TrackId;
    for p in StyleProfile::distributed() {
        let Some(a) = &p.accompaniment else {
            continue;
        };
        let mut plain = p.clone();
        plain.accompaniment.as_mut().unwrap().bass.passing = None;
        for seed in 1..=5 {
            let before = stream(&plain, seed, 100);
            let after = stream(&p, seed, 100);
            for (x, y) in before.iter().zip(&after) {
                for track in [TrackId::Melody, TrackId::Heel, TrackId::Toe, TrackId::Chords] {
                    assert_eq!(
                        note_values(x, track),
                        note_values(y, track),
                        "{} {seed} m{} {track:?}",
                        p.meta.id,
                        x.index
                    );
                }
                let strong = |m: &Measure| {
                    m.notes_for_track(TrackId::Bass)
                        .iter()
                        .filter(|n| a.strong_steps.contains(&n.start_step))
                        .map(|n| (n.start_step, n.pitch))
                        .collect::<Vec<_>>()
                };
                assert_eq!(strong(x), strong(y));
            }
        }
    }
}

#[test]
fn every_passing_motion_stays_in_window_approaches_target_and_vetoes_melody() {
    use harmonium_core::{timeline::TrackId, trad::accomp_profile::PassingMotion};
    for motion in [PassingMotion::ScaleStep, PassingMotion::ScaleWalk, PassingMotion::Chromatic] {
        let mut p = arranged();
        let bass = &mut p.accompaniment.as_mut().unwrap().bass;
        let passing = bass.passing.as_mut().unwrap();
        passing.rules.retain(|r| r.motion == motion);
        passing.none_weight = 0;
        passing.density.0 = vec![[0.0, 1.0], [1.0, 1.0]];
        let bass = p.accompaniment.as_ref().unwrap().bass.clone();
        let mut count = 0;
        for seed in 1..=20 {
            let measures = stream(&p, seed, 64);
            for (i, m) in measures.iter().enumerate() {
                let notes = m.notes_for_track(TrackId::Bass);
                for (index, n) in notes
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| !bass.hits.iter().any(|h| h.step == n.start_step))
                {
                    count += 1;
                    assert!((bass.walk_low.unwrap()..=bass.walk_high.unwrap()).contains(&n.pitch));
                    assert!(
                        !p.accompaniment.as_ref().unwrap().strong_steps.contains(&n.start_step)
                    );
                    if index > 0 {
                        assert_ne!(notes[index - 1].pitch, n.pitch);
                    }
                    let next = notes[index + 1..]
                        .iter()
                        .find(|n| bass.hits.iter().any(|h| h.step == n.start_step))
                        .or_else(|| {
                            measures
                                .get(i + 1)
                                .and_then(|m| m.notes_for_track(TrackId::Bass).first())
                        });
                    if let Some(target) = next {
                        assert_ne!(n.pitch, target.pitch);
                        if motion != PassingMotion::ScaleWalk {
                            assert!(n.pitch.abs_diff(target.pitch) <= 2);
                        }
                    }
                    if motion != PassingMotion::Chromatic {
                        let position = m.position.unwrap();
                        assert!(position.mode.contains_midi(position.key, n.pitch));
                    }
                    let at = n.start_step as i64 * 1000;
                    for melody in m.notes_for_track(TrackId::Melody) {
                        let start =
                            melody.start_step as i64 * 1000 + i64::from(melody.performance.offset);
                        if start <= at
                            && at < start + melody.performance.duration(melody.duration_steps)
                        {
                            assert!(![1, 6, 11].contains(
                                &((i16::from(melody.pitch) - i16::from(n.pitch)).rem_euclid(12)
                                    as u8)
                            ));
                        }
                    }
                }
            }
        }
        assert!(count > 100, "{motion:?}: {count}"); // Chaque frontière/mouvement est réellement exercé.
    }
}

#[test]
fn passing_is_deterministic_after_emotional_invalidation() {
    use harmonium_core::{report::MeasureSnapshot, timeline::TrackId};
    let p = arranged();
    let params = MusicalParams { bpm: p.meter.default_bpm, ..Default::default() };
    let mut g = TradGenerator::new(params, TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    let before: Vec<_> = (1..=40).map(|bar| g.generate_measure(bar, &mut rng)).collect();
    g.invalidate_future(4);
    for original in &before[4..40] {
        let replay = g.generate_measure(original.index, &mut rng);
        assert_eq!(
            serde_json::to_value(MeasureSnapshot::from_measure(original)).unwrap(),
            serde_json::to_value(MeasureSnapshot::from_measure(&replay)).unwrap()
        );
        assert_eq!(note_values(original, TrackId::Bass), note_values(&replay, TrackId::Bass));
    }
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(stream(&p, 3, 40)).unwrap()
    );
}

#[test]
fn passing_curves_and_arousal_modulate_the_realized_density() {
    use harmonium_core::timeline::TrackId;
    let p = arranged();
    let counts = |density: f32, arousal: f32| -> usize {
        (1..=30)
            .map(|seed| {
                let mut g = TradGenerator::new(
                    MusicalParams {
                        rhythm_density: density,
                        arousal,
                        bpm: p.meter.default_bpm,
                        ..Default::default()
                    },
                    TuningParams { profile: p.clone() },
                );
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                (1..=32)
                    .map(|bar| {
                        g.generate_measure(bar, &mut rng)
                            .notes_for_track(TrackId::Bass)
                            .iter()
                            .filter(|n| ![0, 8].contains(&n.start_step))
                            .count()
                    })
                    .sum::<usize>()
            })
            .sum()
    };
    assert_eq!(counts(0.0, 1.0), 0);
    let low = counts(0.5, 0.0);
    let high = counts(0.5, 1.0);
    assert!(low > 30 && high > low * 3 / 2, "{low} → {high}");
}

#[test]
fn invalid_passing_rules_are_rejected_with_field_paths() {
    let p = arranged();
    for (field, mutate) in [
        (
            "bass.walk_low",
            (|p: &mut StyleProfile| p.accompaniment.as_mut().unwrap().bass.walk_low = Some(51))
                as fn(&mut StyleProfile),
        ),
        ("bass.walk_high", |p| p.accompaniment.as_mut().unwrap().bass.walk_high = Some(128)),
        ("bass.passing.rules[0].before", |p| {
            p.accompaniment.as_mut().unwrap().bass.passing.as_mut().unwrap().rules[0].before =
                vec![0]
        }),
        ("bass.passing.rules[1].before", |p| {
            p.accompaniment.as_mut().unwrap().bass.passing.as_mut().unwrap().rules[1].before =
                vec![2, 4]
        }),
        ("bass.passing.rules[0].duration", |p| {
            p.accompaniment.as_mut().unwrap().bass.passing.as_mut().unwrap().rules[0].duration = 10
        }),
        ("bass.passing.rules[0].weight", |p| {
            p.accompaniment.as_mut().unwrap().bass.passing.as_mut().unwrap().rules[0].weight =
                u32::MAX
        }),
        ("bass.passing.density", |p| {
            p.accompaniment.as_mut().unwrap().bass.passing.as_mut().unwrap().density.0[0][1] = -1.0
        }),
        ("bass.passing.avoid_melody_intervals", |p| {
            p.accompaniment
                .as_mut()
                .unwrap()
                .bass
                .passing
                .as_mut()
                .unwrap()
                .avoid_melody_intervals
                .push(12)
        }),
    ] {
        let mut bad = p.clone();
        mutate(&mut bad);
        let error = StyleProfile::from_toml(&bad.to_toml().unwrap()).unwrap_err().to_string();
        assert!(error.contains(&format!("accompaniment.{field}")), "{error}");
    }
    let text = p.to_toml().unwrap();
    for (from, to, field) in
        [("scale_step", "bad_motion", "motion"), ("next_target_differs", "bad_condition", "when")]
    {
        let error = StyleProfile::from_toml(&text.replace(from, to)).unwrap_err().to_string();
        assert!(error.contains(field), "{error}");
    }
}

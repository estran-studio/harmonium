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
    p.set = Default::default();
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

fn strummed() -> StyleProfile {
    use harmonium_core::trad::{accomp_profile::Stroke, dynamics::Curve};
    let mut p = arranged();
    p.dynamics.chop_intensity = Curve(vec![[0.0, 0.0], [1.0, 0.0]]);
    p.dynamics.density_chop = Curve(vec![[0.0, 0.0], [1.0, 0.0]]);
    let c = &mut p.accompaniment.as_mut().unwrap().chords;
    c.strings = 6;
    c.patterns[c.default_pattern].hits[1].stroke = Stroke::Up;
    p
}

fn chord_groups(m: &Measure) -> Vec<&[harmonium_core::timeline::TimelineNote]> {
    let notes = m.notes_for_track(harmonium_core::timeline::TrackId::Chords);
    let mut groups = Vec::new();
    let mut i = 0;
    while i < notes.len() {
        let count = notes[i..].iter().take_while(|n| n.start_step == notes[i].start_step).count();
        groups.push(&notes[i..i + count]);
        i += count;
    }
    groups
}

#[test]
fn strings_attack_in_vector_order_with_bounded_ms_gaps_at_three_tempos() {
    for bpm in [92.0, 172.5, 230.0] {
        let mut p = strummed();
        p.meter.min_bpm = 40.0;
        p.meter.max_bpm = 300.0;
        p.meter.default_bpm = bpm;
        let s = p.accompaniment.as_ref().unwrap().chords.strum.as_ref().unwrap();
        let measures = stream(&p, 11, 200);
        let mut up = 0;
        let mut down = 0;
        for m in &measures {
            let step_ms = 60000.0 / f64::from(m.tempo) / m.steps_per_quarter();
            for group in chord_groups(m) {
                let ascending = group[0].pitch < group[1].pitch;
                if ascending {
                    down += 1;
                    assert_eq!(group.len(), 6);
                } else {
                    up += 1;
                    assert_eq!(group.len(), 3);
                }
                let gap = f64::from(if ascending { s.down_gap_ms } else { s.up_gap_ms });
                for pair in group.windows(2) {
                    assert!(pair[0].id < pair[1].id);
                    assert!(if ascending {
                        pair[0].pitch < pair[1].pitch
                    } else {
                        pair[0].pitch > pair[1].pitch
                    });
                    assert!(pair[0].performance.offset < pair[1].performance.offset);
                    let ms = f64::from(pair[1].performance.offset - pair[0].performance.offset)
                        * step_ms
                        / 1000.0;
                    assert!(ms >= gap * (1.0 - f64::from(s.gap_jitter)) - step_ms / 1000.0);
                    assert!(ms <= gap * (1.0 + f64::from(s.gap_jitter)) + step_ms / 1000.0);
                }
                assert!(group.iter().all(|n| n.performance.offset.abs() <= 490));
                let spread = i32::from(group.last().unwrap().performance.offset)
                    - i32::from(group[0].performance.offset);
                assert!(spread as f32 <= s.max_spread_ratio * 8000.0 + 1.0);
                assert_eq!(
                    group.iter().map(|n| n.pitch).collect::<std::collections::HashSet<_>>().len(),
                    group.len()
                );
            }
        }
        assert!(up > 100 && down > 100);
    }
}

#[test]
fn strum_composes_with_atomic_v2_attack_and_configurable_anchor() {
    use harmonium_core::{
        performance::{ArticulationRule, TrackPerformance},
        timeline::{Articulation, TrackId},
    };
    for anchor in [0.0, 0.5, 1.0] {
        let mut p = strummed();
        let strum = p.accompaniment.as_mut().unwrap().chords.strum.as_mut().unwrap();
        strum.anchor = anchor;
        strum.gap_jitter = 0.0;
        strum.velocity_jitter = 0;
        strum.velocity_slope = 0;
        let rule = |articulation, weight, shorten| ArticulationRule {
            articulation,
            weight,
            steps: vec![],
            min_duration: 0,
            max_duration: None,
            context: Default::default(),
            shorten,
            accent: 0.0,
        };
        p.performance.tracks.insert(
            "Chords".into(),
            TrackPerformance {
                offsets: vec![0.05; p.meter.steps_per_bar],
                jitter: 0.1,
                breath: 0.2,
                rules: vec![
                    rule(Articulation::Normal, 1.0, 0.0),
                    rule(Articulation::Detached, 1.0, 0.2),
                ],
                ..Default::default()
            },
        );
        p.validate().unwrap(); // Un jitter V2 supérieur au demi-écart est désormais sûr : tirage commun.
        let mut block = p.clone();
        block.accompaniment.as_mut().unwrap().chords.strum = None;
        let baseline = stream(&block, 7, 200);
        let realized = stream(&p, 7, 200);
        for (a, b) in baseline.iter().zip(&realized) {
            for group in chord_groups(b) {
                let first = &group[0];
                let base = a
                    .notes_for_track(TrackId::Chords)
                    .iter()
                    .find(|n| n.start_step == first.start_step)
                    .unwrap();
                let spread =
                    f32::from(group.last().unwrap().performance.offset - first.performance.offset);
                let mean = group.iter().map(|n| f32::from(n.performance.offset)).sum::<f32>()
                    / group.len() as f32;
                assert!(
                    (mean - f32::from(base.performance.offset) - (0.5 - anchor) * spread).abs()
                        <= 1.0
                );
                assert!(group.iter().all(|n| n.articulation == first.articulation
                    && n.performance.gate == first.performance.gate
                    && n.performance.breath == first.performance.breath));
                if anchor == 0.0 {
                    assert_eq!(first.performance.offset, base.performance.offset);
                }
                if anchor == 1.0 {
                    assert_eq!(group.last().unwrap().performance.offset, base.performance.offset);
                }
            }
        }
    }
}

#[test]
fn strum_caps_the_whole_gesture_near_offset_and_neighbour_boundaries() {
    use harmonium_core::{performance::TrackPerformance, trad::accomp_profile::Stroke};
    let mut p = strummed();
    let c = &mut p.accompaniment.as_mut().unwrap().chords;
    c.patterns[c.default_pattern].hits[1].step = 5;
    for hit in &mut c.patterns[c.default_pattern].hits {
        hit.stroke = Stroke::Down;
    }
    let s = c.strum.as_mut().unwrap();
    s.down_gap_ms = 40.0;
    s.gap_jitter = 0.0;
    s.anchor = 1.0;
    p.performance.tracks.insert(
        "Chords".into(),
        TrackPerformance {
            offsets: vec![-0.2; p.meter.steps_per_bar],
            emotions: harmonium_core::performance::PerformanceEmotions {
                tension_advance: 0.2,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let params =
        MusicalParams { bpm: p.meter.default_bpm, harmony_tension: 1.0, ..Default::default() };
    let mut g = TradGenerator::new(params, TuningParams { profile: p });
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    for bar in 1..=200 {
        let m = g.generate_measure(bar, &mut rng);
        for group in chord_groups(&m) {
            assert!(group.iter().all(|n| (-490..=490).contains(&n.performance.offset)));
            assert!(group.windows(2).all(|w| w[0].performance.offset < w[1].performance.offset));
            assert_eq!(group.last().unwrap().performance.offset, -400);
            assert!(group.last().unwrap().performance.offset - group[0].performance.offset <= 90);
        }
    }
}

#[test]
fn optional_hits_draw_independently_of_neighbour_tracks_and_replay() {
    use harmonium_core::timeline::TrackId;
    let mut p = strummed();
    p.accompaniment.as_mut().unwrap().chords.patterns[1].hits[1].probability = 0.4;
    let mut absent = p.clone();
    absent.accompaniment.as_mut().unwrap().chords.patterns[1].hits[1].probability = 0.0;
    let before = stream(&absent, 31, 1000);
    let after = stream(&p, 31, 1000);
    let observed = after
        .iter()
        .filter(|m| m.notes_for_track(TrackId::Chords).iter().any(|n| n.start_step == 12))
        .count();
    assert!((350..=450).contains(&observed), "{observed}/1000");
    for (a, b) in before.iter().zip(&after) {
        for track in [TrackId::Melody, TrackId::Heel, TrackId::Toe, TrackId::Bass] {
            assert_eq!(note_values(a, track), note_values(b, track));
        }
    }
}

#[test]
fn invalid_strum_fields_and_strokes_are_rejected_with_paths() {
    let p = strummed();
    for (field, mutate) in [
        (
            "strings",
            (|p: &mut StyleProfile| p.accompaniment.as_mut().unwrap().chords.strings = 7)
                as fn(&mut StyleProfile),
        ),
        ("strum.down_gap_ms", |p| {
            p.accompaniment.as_mut().unwrap().chords.strum.as_mut().unwrap().down_gap_ms = 41.0
        }),
        ("strum.up_gap_ms", |p| {
            p.accompaniment.as_mut().unwrap().chords.strum.as_mut().unwrap().up_gap_ms = -1.0
        }),
        ("strum.up_strings", |p| {
            p.accompaniment.as_mut().unwrap().chords.strum.as_mut().unwrap().up_strings = 7
        }),
        ("strum.gap_jitter", |p| {
            p.accompaniment.as_mut().unwrap().chords.strum.as_mut().unwrap().gap_jitter = 0.9
        }),
        ("strum.anchor", |p| {
            p.accompaniment.as_mut().unwrap().chords.strum.as_mut().unwrap().anchor = 1.1
        }),
        ("strum.tempo_exponent", |p| {
            p.accompaniment.as_mut().unwrap().chords.strum.as_mut().unwrap().tempo_exponent = -1.0
        }),
        ("strum.ref_bpm", |p| {
            p.accompaniment.as_mut().unwrap().chords.strum.as_mut().unwrap().ref_bpm = 0.0
        }),
        ("strum.max_spread_ratio", |p| {
            p.accompaniment.as_mut().unwrap().chords.strum.as_mut().unwrap().max_spread_ratio = 0.5
        }),
        ("strum.velocity_slope", |p| {
            p.accompaniment.as_mut().unwrap().chords.strum.as_mut().unwrap().velocity_slope = 21
        }),
        ("strum.velocity_jitter", |p| {
            p.accompaniment.as_mut().unwrap().chords.strum.as_mut().unwrap().velocity_jitter = 21
        }),
        ("patterns[0].hits[0].probability", |p| {
            p.accompaniment.as_mut().unwrap().chords.patterns[0].hits[0].probability = 1.1
        }),
    ] {
        let mut bad = p.clone();
        mutate(&mut bad);
        let error = StyleProfile::from_toml(&bad.to_toml().unwrap()).unwrap_err().to_string();
        assert!(error.contains(&format!("accompaniment.chords.{field}")), "{error}");
    }
    let error = StyleProfile::from_toml(
        &p.to_toml().unwrap().replace("stroke = \"down\"", "stroke = \"unknown\""),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("stroke"), "{error}");
}

#[test]
fn generated_strum_offsets_and_vector_order_are_present_in_static_midi() {
    use harmonium_core::timeline::{ScoreTimeline, TrackId, timeline_to_midi};
    use midly::{MidiMessage, Smf, TrackEventKind};
    let p = strummed();
    let measures = stream(&p, 3, 32);
    let mut expected = Vec::new();
    let mut score = ScoreTimeline::with_default_capacity();
    for m in measures {
        for note in m.notes_for_track(TrackId::Chords) {
            let tick = ((m.index - 1) as f64 * 1920.0
                + note.start_step as f64 * 120.0
                + f64::from(note.performance.offset) * 0.12)
                .round()
                .max(0.0) as u32;
            expected.push((tick, note.pitch));
        }
        score.push_measure(m);
    }
    expected.sort_by_key(|n| n.0);
    let bytes = timeline_to_midi(&score, &p.mix).unwrap();
    let midi = Smf::parse(&bytes).unwrap();
    let mut tick = 0;
    let mut observed = Vec::new();
    for event in &midi.tracks[2] {
        tick += event.delta.as_int();
        if let TrackEventKind::Midi { message: MidiMessage::NoteOn { key, vel }, .. } = event.kind
            && vel.as_int() > 0
        {
            observed.push((tick, key.as_int()));
        }
    }
    assert_eq!(observed, expected);
    assert!(observed.windows(2).any(|w| w[1].0 - w[0].0 > 5 && w[1].0 - w[0].0 < 20));
}

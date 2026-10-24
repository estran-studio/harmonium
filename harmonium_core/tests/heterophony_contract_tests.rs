//! Contrats V4 exclusivement sur les mesures publiées.
use harmonium_core::{
    StyleProfile, TuningParams,
    heterophony::{Operation, Variation},
    ornaments::{BeatClass, PhraseEnd},
    params::MusicalParams,
    report::MeasureSnapshot,
    timeline::{Measure, TrackId},
    trad::{ReelArrangement, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn stream(profile: &StyleProfile, seed: u64, bars: usize, arousal: f32) -> Vec<Measure> {
    profile.validate().unwrap();
    let mut generator = TradGenerator::new(
        MusicalParams {
            bpm: profile.meter.default_bpm,
            arousal,
            rhythm_density: 0.5,
            ..Default::default()
        },
        TuningParams { profile: profile.clone() },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars).map(|bar| generator.generate_measure(bar, &mut rng)).collect()
}
fn strong_collisions(measure: &Measure) -> usize {
    let active = |n: &harmonium_core::timeline::TimelineNote, time: i64| {
        let on = n.start_step as i64 * 1000 + i64::from(n.performance.offset);
        on <= time && time < on + n.performance.duration(n.duration_steps)
    };
    (0..measure.steps)
        .step_by(usize::from(measure.pulse.steps_per_pulse))
        .map(|step| {
            let time = step as i64 * 1000;
            measure
                .notes_for_track(TrackId::Countermelody)
                .iter()
                .filter(|n| active(n, time))
                .map(|n| {
                    measure
                        .notes_for_track(TrackId::Melody)
                        .iter()
                        .filter(|lead| {
                            active(lead, time) && matches!(n.pitch.abs_diff(lead.pitch) % 12, 1 | 2)
                        })
                        .count()
                })
                .sum::<usize>()
        })
        .sum()
}

#[test]
fn two_hundred_airs_keep_cadences_register_and_strong_beats() {
    for profile in StyleProfile::distributed() {
        let Some(h) = &profile.heterophony else { continue };
        for seed in 0..200 {
            for measure in stream(&profile, seed, 66, (seed % 3) as f32 / 2.0) {
                assert_eq!(
                    strong_collisions(&measure),
                    0,
                    "{} seed {seed} bar {}",
                    profile.meta.id,
                    measure.index
                );
                let second = measure.notes_for_track(TrackId::Countermelody);
                assert!(second.iter().all(|n| (h.low..=h.high).contains(&n.pitch)));
                let Some(position) = measure.position else { continue };
                if position.is_intro() || position.tour < 2 {
                    assert!(second.is_empty());
                    continue;
                }
                let part = &profile.form.parts[position.part.0];
                if part.phrase_ends.contains(&position.bar_in_strain) && !second.is_empty() {
                    let lead = measure.notes_for_track(TrackId::Melody);
                    let tail = second.last().unwrap();
                    let last_step = lead.iter().map(|n| n.start_step).max().unwrap();
                    assert!(
                        lead.iter()
                            .any(|n| n.start_step == last_step && n.pitch % 12 == tail.pitch % 12),
                        "{} seed {seed} bar {}: cadence {}",
                        profile.meta.id,
                        measure.index,
                        tail.pitch
                    );
                }
            }
        }
    }
}

#[test]
fn deterministic_and_no_changes_to_the_existing_voices() {
    for profile in StyleProfile::distributed() {
        let mut absent = profile.clone();
        absent.heterophony = None;
        for seed in [3, 42] {
            let live = stream(&profile, seed, 160, 0.5);
            let repeat = stream(&profile, seed, 160, 0.5);
            assert_eq!(
                serde_json::to_value(&live).unwrap(),
                serde_json::to_value(&repeat).unwrap()
            );
            let plain = stream(&absent, seed, 160, 0.5);
            for (a, b) in live.iter().zip(&plain) {
                let mut x = MeasureSnapshot::from_measure(a);
                x.notes.retain(|n| n.track != TrackId::Countermelody.channel());
                let y = MeasureSnapshot::from_measure(b);
                assert_eq!(
                    serde_json::to_value(x).unwrap(),
                    serde_json::to_value(y).unwrap(),
                    "{} seed {seed} bar {}",
                    profile.meta.id,
                    a.index
                );
            }
        }
    }
}

#[test]
fn configured_weights_control_octaves_and_weak_note_silences_on_two_hundred_airs() {
    for operation in [Operation::Octave, Operation::Rest] {
        let mut profile = StyleProfile::default();
        profile.performance = Default::default();
        profile.ornaments = Default::default();
        profile.arrangement = ReelArrangement::from_start(2, Default::default());
        let h = profile.heterophony.as_mut().unwrap();
        h.low = 0;
        h.high = 127;
        h.arousal_low = 1.0;
        h.arousal_high = 1.0;
        h.density_low = 1.0;
        h.density_high = 1.0;
        h.operations = vec![
            Variation {
                operation: Operation::Unison,
                weight: 3,
                beats: vec![],
                phrase_end: PhraseEnd::Any,
                bars: vec![],
            },
            Variation {
                operation,
                weight: 1,
                beats: if operation == Operation::Rest { vec![BeatClass::Weak] } else { vec![] },
                phrase_end: PhraseEnd::Never,
                bars: vec![],
            },
        ];
        let (mut eligible, mut varied) = (0usize, 0usize);
        for seed in 0..200 {
            for measure in stream(&profile, seed, 32, 0.5) {
                let position = measure.position.unwrap();
                if profile.form.parts[position.part.0].phrase_ends.contains(&position.bar_in_strain)
                {
                    continue;
                }
                for lead in measure.notes_for_track(TrackId::Melody) {
                    if operation == Operation::Rest
                        && lead.start_step.is_multiple_of(profile.meter.steps_per_pulse)
                    {
                        continue;
                    }
                    eligible += 1;
                    let second = measure
                        .notes_for_track(TrackId::Countermelody)
                        .iter()
                        .find(|n| n.start_step == lead.start_step);
                    varied += usize::from(if operation == Operation::Rest {
                        second.is_none()
                    } else {
                        second.is_some_and(|n| n.pitch == lead.pitch + 12)
                    });
                }
            }
        }
        let rate = varied as f64 / eligible as f64;
        assert!((0.23..=0.27).contains(&rate), "{operation:?}: {varied}/{eligible} = {rate}");
    }
}

#[test]
fn invalid_sections_are_rejected_by_the_public_loader() {
    let raw = toml::Value::try_from(StyleProfile::default()).unwrap();
    for (field, value) in [
        ("source", toml::Value::String("Bass".into())),
        ("octave", toml::Value::Integer(3)),
        ("arousal_high", toml::Value::Float(f64::NAN)),
        ("density_high", toml::Value::Float(4.1)),
        ("low", toml::Value::Integer(100)),
    ] {
        let mut bad = raw.clone();
        bad["heterophony"][field] = value;
        let error = StyleProfile::from_toml(&toml::to_string(&bad).unwrap()).unwrap_err();
        assert!(error.to_string().contains("heterophony"), "{field}: {error}");
    }
    let mut bad = raw;
    bad["heterophony"]["operations"][0]["weight"] = toml::Value::Integer(0);
    assert!(StyleProfile::from_toml(&toml::to_string(&bad).unwrap()).is_err());
}

fn isolated(operation: Operation) -> StyleProfile {
    let mut profile = StyleProfile::default();
    profile.performance = Default::default();
    profile.ornaments = Default::default();
    profile.arrangement = ReelArrangement::from_start(2, Default::default());
    let h = profile.heterophony.as_mut().unwrap();
    h.low = 0;
    h.high = 127;
    h.arousal_low = 1.0;
    h.arousal_high = 1.0;
    h.density_low = 1.0;
    h.density_high = 1.0;
    h.operations = vec![
        Variation {
            operation: Operation::Unison,
            weight: 3,
            beats: vec![],
            phrase_end: PhraseEnd::Any,
            bars: vec![],
        },
        Variation {
            operation,
            weight: 1,
            beats: vec![],
            phrase_end: PhraseEnd::Only,
            bars: vec![],
        },
    ];
    profile
}

#[test]
fn thirds_and_sixths_follow_weights_only_on_harmonic_cadence_targets() {
    for operation in [Operation::Third, Operation::Sixth] {
        let profile = isolated(operation);
        let (mut eligible, mut varied) = (0, 0);
        for seed in 0..200 {
            for measure in stream(&profile, seed, 32, 0.5) {
                let position = measure.position.unwrap();
                if !profile.form.parts[position.part.0]
                    .phrase_ends
                    .contains(&position.bar_in_strain)
                {
                    continue;
                }
                let melody = measure.notes_for_track(TrackId::Melody);
                for lead in melody.iter().take(melody.len().saturating_sub(1)) {
                    let mut pitch = lead.pitch;
                    for _ in 0..if operation == Operation::Third { 2 } else { 5 } {
                        pitch = (pitch + 1..=127)
                            .find(|p| position.mode.contains_midi(position.key, *p))
                            .unwrap();
                    }
                    let chord = &measure.chord_context;
                    let root = (i32::from(position.key) + chord.root_offset).rem_euclid(12);
                    if ![0, if chord.is_minor { 3 } else { 4 }, 7]
                        .contains(&(i32::from(pitch) - root).rem_euclid(12))
                    {
                        continue;
                    }
                    eligible += 1;
                    varied += usize::from(
                        measure
                            .notes_for_track(TrackId::Countermelody)
                            .iter()
                            .any(|n| n.start_step == lead.start_step && n.pitch == pitch),
                    );
                }
            }
        }
        let rate = varied as f64 / eligible as f64;
        assert!((0.22..=0.28).contains(&rate), "{operation:?}: {varied}/{eligible} = {rate}");
    }
}

#[test]
fn plain_omits_ornaments_and_unison_reuses_v3b_while_independent_seed_varies_it() {
    use harmonium_core::ornaments::{OrnamentKind, OrnamentRule};
    let mut unison = isolated(Operation::Unison);
    // Même tessiture que la mélodie : le voisin d'un agrément est borné de la même façon.
    let (low, high) = (unison.melody.params.tessitura_min, unison.melody.params.tessitura_max);
    let h = unison.heterophony.as_mut().unwrap();
    (h.low, h.high) = (low, high);
    h.operations.truncate(1);
    unison.ornaments.types.insert(
        OrnamentKind::GraceSingle,
        OrnamentRule { written_per_100_notes: 100.0, ..Default::default() },
    );
    let mut plain = unison.clone();
    plain.heterophony.as_mut().unwrap().operations.push(Variation {
        operation: Operation::Plain,
        weight: 1_000_000,
        beats: vec![],
        phrase_end: PhraseEnd::Any,
        bars: vec![],
    });
    let mut independent = plain.clone();
    independent.heterophony.as_mut().unwrap().operations[1].operation = Operation::Ornament;
    let signature = |m: &Measure, track| {
        m.notes_for_track(track)
            .iter()
            .map(|n| (n.pitch, n.start_step, n.duration_steps, n.performance))
            .collect::<Vec<_>>()
    };
    let (mut opportunities, mut different, mut absent) = (0, 0, 0);
    for seed in 0..200 {
        let a = stream(&unison, seed, 8, 0.5);
        let b = stream(&plain, seed, 8, 0.5);
        let c = stream(&independent, seed, 8, 0.5);
        for ((a, b), c) in a.iter().zip(&b).zip(&c) {
            let position = a.position.unwrap();
            if unison.form.parts[position.part.0].phrase_ends.contains(&position.bar_in_strain) {
                continue;
            }
            assert_eq!(
                signature(a, TrackId::Melody),
                signature(a, TrackId::Countermelody),
                "seed {seed} mesure {}",
                a.index
            );
            opportunities += 1;
            different +=
                usize::from(signature(c, TrackId::Melody) != signature(c, TrackId::Countermelody));
            absent += usize::from(
                b.notes_for_track(TrackId::Countermelody).len()
                    < b.notes_for_track(TrackId::Melody).len(),
            );
        }
    }
    assert!(different * 2 > opportunities, "graine indépendante : {different}/{opportunities}");
    assert!(absent * 10 > opportunities * 9, "sans agréments : {absent}/{opportunities}");
}

#[test]
fn emotion_multiplier_and_position_filters_bound_the_realized_variation() {
    let mut profile = isolated(Operation::Octave);
    let h = profile.heterophony.as_mut().unwrap();
    h.arousal_low = 0.0;
    h.operations[1].phrase_end = PhraseEnd::Never;
    h.operations[1].bars = vec![2, 6];
    h.operations[1].beats = vec![BeatClass::Weak];
    let mut high_changes = 0;
    for seed in 0..200 {
        for (low, high) in
            stream(&profile, seed, 32, 0.0).iter().zip(stream(&profile, seed, 32, 1.0))
        {
            for measure in [low, &high] {
                let position = measure.position.unwrap();
                for n in measure.notes_for_track(TrackId::Countermelody) {
                    let lead = measure
                        .notes_for_track(TrackId::Melody)
                        .iter()
                        .find(|lead| lead.start_step == n.start_step)
                        .unwrap();
                    if n.pitch != lead.pitch {
                        assert_eq!(n.pitch, lead.pitch + 12);
                        assert!([2, 6].contains(&position.bar_in_strain));
                        assert!(!n.start_step.is_multiple_of(profile.meter.steps_per_pulse));
                        assert_ne!(measure.state_snapshot.arousal, 0.0);
                        high_changes += 1;
                    }
                }
            }
        }
    }
    assert!(high_changes > 100);
}

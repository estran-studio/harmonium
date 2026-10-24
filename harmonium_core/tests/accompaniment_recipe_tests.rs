//! Les recettes publiées sont chargées sans noms de recettes dans le moteur.
use harmonium_core::{
    StyleProfile, TuningParams,
    params::MusicalParams,
    timeline::{Measure, TrackId},
    trad::{IntroConfig, Part, ReelArrangement, TradGenerator, accomp_profile::BassDoubling},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn recipe(name: &str) -> StyleProfile {
    let readme = include_str!("../../profiles/README.md");
    let marker = format!("# recette: {name}\n");
    let text = readme.split_once(&marker).unwrap().1.split_once("\n```").unwrap().0;
    let snippet: toml::Value = toml::from_str(text).unwrap();
    let mut profile: toml::Value =
        toml::from_str(&StyleProfile::default().to_toml().unwrap()).unwrap();
    for voice in ["bass", "chords"] {
        profile["accompaniment"][voice] = snippet["accompaniment"][voice].clone();
    }
    for (key, value) in snippet["instruments"].as_table().unwrap() {
        profile["instruments"][key] = value.clone();
    }
    let mut profile = StyleProfile::from_toml(&toml::to_string(&profile).unwrap()).unwrap();
    profile.arrangement = ReelArrangement::from_start(2, Part(0));
    profile.arrangement.intro = IntroConfig::none();
    profile
}

fn stream(p: &StyleProfile, seed: u64, bars: usize) -> Vec<Measure> {
    p.validate().unwrap();
    let mut g = TradGenerator::new(
        MusicalParams { bpm: p.meter.default_bpm, ..Default::default() },
        TuningParams { profile: p.clone() },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars).map(|bar| g.generate_measure(bar, &mut rng)).collect()
}

#[test]
fn piano_recipe_octaves_tenths_and_runs_follow_the_low_bass_targets() {
    for doubling in [BassDoubling::Octave, BassDoubling::Tenth] {
        let mut p = recipe("piano_main_gauche");
        for hit in &mut p.accompaniment.as_mut().unwrap().bass.hits {
            hit.doubling = vec![doubling];
        }
        p.performance.tracks.insert(
            "Bass".into(),
            harmonium_core::performance::TrackPerformance {
                jitter: 0.04,
                breath: 0.1,
                emotions: harmonium_core::performance::PerformanceEmotions {
                    tension_detach: 0.2,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        let mut runs = 0;
        for seed in 1..=20 {
            let measures = stream(&p, seed, 64);
            for (bar, m) in measures.iter().enumerate() {
                let notes = m.notes_for_track(TrackId::Bass);
                for step in [0, 8] {
                    let hit: Vec<_> = notes.iter().filter(|n| n.start_step == step).collect();
                    assert_eq!(hit.len(), 2);
                    let (primary, upper) = (hit[0], hit[1]);
                    assert_eq!(primary.performance, upper.performance);
                    assert_eq!(
                        (primary.duration_steps, primary.velocity),
                        (upper.duration_steps, upper.velocity)
                    );
                    let interval = upper.pitch - primary.pitch;
                    if doubling == BassDoubling::Octave {
                        assert_eq!(interval, 12);
                    } else {
                        assert!([15, 16].contains(&interval));
                        let pos = m.position.unwrap();
                        assert!(pos.mode.contains_midi(pos.key, upper.pitch));
                    }
                }
                for passing in notes.iter().filter(|n| ![0, 8].contains(&n.start_step)) {
                    runs += 1;
                    let source_step = if passing.start_step < 8 { 0 } else { 8 };
                    let source = notes.iter().find(|n| n.start_step == source_step).unwrap();
                    let target = if source_step == 0 {
                        notes.iter().find(|n| n.start_step == 8)
                    } else {
                        measures
                            .get(bar + 1)
                            .and_then(|next| next.notes_for_track(TrackId::Bass).first())
                    };
                    if let Some(target) = target {
                        assert!(
                            (source.pitch < passing.pitch && passing.pitch < target.pitch)
                                || (target.pitch < passing.pitch && passing.pitch < source.pitch)
                        );
                    }
                    assert!((34..=50).contains(&passing.pitch));
                }
                for chord in m.notes_for_track(TrackId::Chords) {
                    assert_eq!(chord.performance.offset, 0);
                }
            }
        }
        assert!(runs > 100, "{doubling:?}: {runs}");
    }
}

#[test]
fn accordion_recipe_has_fixed_buttons_and_no_walking_or_strum() {
    let mut p = recipe("accordeon_basse_accord");
    p.harmony.steps_per_chord = p.meter.steps_per_bar;
    let mut buttons = std::collections::BTreeMap::new();
    let mut repetitions = 0;
    for seed in 1..=20 {
        for m in stream(&p, seed, 64) {
            let pos = m.position.unwrap();
            let key = (pos.key, format!("{:?}", pos.mode), m.chord_context.chord_name.clone());
            let notes = m.notes_for_track(TrackId::Chords);
            for group in notes.chunks(3) {
                assert_eq!(group.len(), 3);
                assert!(
                    group.iter().all(|n| (52..=64).contains(&n.pitch) && n.performance.offset == 0)
                );
                let pitches: Vec<_> = group.iter().map(|n| n.pitch).collect();
                if let Some(previous) = buttons.insert(key.clone(), pitches.clone()) {
                    repetitions += 1;
                    assert_eq!(previous, pitches, "{key:?}");
                }
            }
            let bass: Vec<_> =
                m.notes_for_track(TrackId::Bass).iter().map(|n| n.start_step).collect();
            assert_eq!(bass, [0, 8]);
        }
    }
    assert!(repetitions > 100);
}

#[test]
fn recipe_registers_and_doublings_are_checked_before_publication() {
    for (field, mutate) in [
        (
            "bass.hits[0].doubling",
            (|p: &mut StyleProfile| {
                p.accompaniment.as_mut().unwrap().bass.hits[0].doubling.push(BassDoubling::Octave)
            }) as fn(&mut StyleProfile),
        ),
        ("chords.low", |p| {
            let c = &mut p.accompaniment.as_mut().unwrap().chords;
            c.low = 128;
        }),
        ("chords.high", |p| {
            let c = &mut p.accompaniment.as_mut().unwrap().chords;
            c.voicing = harmonium_core::trad::accomp_profile::ChordVoicing::FixedTriad;
            c.high = c.low + 5;
        }),
    ] {
        let mut p = recipe("piano_main_gauche");
        mutate(&mut p);
        let error = StyleProfile::from_toml(&p.to_toml().unwrap()).unwrap_err().to_string();
        assert!(error.contains(&format!("accompaniment.{field}")), "{error}");
    }
    let mut p = recipe("piano_main_gauche");
    let b = &mut p.accompaniment.as_mut().unwrap().bass;
    b.low = 116;
    b.high = 127;
    b.walk_low = None;
    b.walk_high = None;
    let error = p.validate().unwrap_err().to_string();
    assert!(error.contains("bass.hits[0].doubling"), "{error}");
    let mut data: toml::Value =
        toml::from_str(&recipe("piano_main_gauche").to_toml().unwrap()).unwrap();
    data["accompaniment"]["bass"]["hits"][0]["doubling"] =
        toml::Value::Array(vec![toml::Value::String("unknown".into())]);
    assert!(
        StyleProfile::from_toml(&toml::to_string(&data).unwrap())
            .unwrap_err()
            .to_string()
            .contains("doubling")
    );
    data["accompaniment"]["bass"]["hits"][0]["doubling"] = toml::Value::Array(vec![]);
    data["accompaniment"]["chords"]["voicing"] = toml::Value::String("unknown".into());
    assert!(
        StyleProfile::from_toml(&toml::to_string(&data).unwrap())
            .unwrap_err()
            .to_string()
            .contains("voicing")
    );
}

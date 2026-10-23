//! V2 : décisions observables dans les snapshots, sans inspecter le générateur.
use harmonium_core::{
    StyleProfile, TuningParams, params::MusicalParams, performance::Performance,
    report::MeasureSnapshot, timeline::TrackId, trad::TradGenerator,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const LIVE: &str = r#"
[tracks.Melody]
offsets = [-0.02,0.0,0.01,0.0,-0.02,0.0,0.01,0.0,-0.02,0.0,0.01,0.0,-0.02,0.0,0.01,0.0]
jitter = 0.02
breath = 0.12
phrase = [[0.0,-2.0],[0.5,3.0],[1.0,-3.0]]
velocity_min = 30
velocity_max = 120
[[tracks.Melody.rules]]
articulation = "Accent"
weight = 1.0
accent = 2.0
shorten = 0.05
[tracks.Melody.emotions]
tension_advance = 0.10
tension_detach = 0.20
arousal_accent = 5.0
"#;
fn generator(performance: Performance, tension: f32, arousal: f32) -> TradGenerator {
    let mut profile = StyleProfile::builtin().clone();
    profile.performance = performance;
    for part in &mut profile.form.parts {
        part.phrase_ends = vec![3, 8];
    }
    profile.validate().unwrap();
    let params = MusicalParams { harmony_tension: tension, arousal, ..Default::default() };
    TradGenerator::new(params, TuningParams { profile, ..Default::default() })
}
fn stream(performance: Performance, tension: f32, arousal: f32) -> Vec<MeasureSnapshot> {
    let mut generator = generator(performance, tension, arousal);
    let mut rng = ChaCha8Rng::seed_from_u64(37);
    (1..=100)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect()
}

/// Un seek ne rejoue pas les mesures déjà en mémoire : l'interprétation de la
/// mesure `n` ne peut dépendre ni des mesures générées avant elle ni de leur nombre.
#[test]
fn a_bar_is_performed_the_same_whatever_was_generated_before() {
    let linear = stream(live(), 0.4, 0.5);
    let mut generator = generator(live(), 0.4, 0.5);
    let mut rng = ChaCha8Rng::seed_from_u64(37);
    let _ = generator.generate_measure(1, &mut rng);
    for bar in [9, 17, 41] {
        let direct = MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng));
        assert_eq!(json(&[direct]), json(&linear[bar - 1..bar]), "mesure {bar}");
    }
    assert!(linear[8].notes.iter().any(|n| n.performance.offset != 0));
}
fn live() -> Performance {
    toml::from_str(LIVE).unwrap()
}
fn json(stream: &[MeasureSnapshot]) -> String {
    serde_json::to_string(stream).unwrap()
}

#[test]
fn absent_and_explicit_zero_preserve_entire_stream_and_rng() {
    let absent = stream(Performance::default(), 0.4, 0.5);
    let zero: Performance = toml::from_str(
        r#"
[tracks.Melody]
offsets = [0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0]
jitter = 0.0
breath = 0.0
phrase = [[0.0,0.0],[1.0,0.0]]
[[tracks.Melody.rules]]
articulation = "Detached"
weight = 1.0
shorten = 0.0
accent = 0.0
"#,
    )
    .unwrap();
    assert_eq!(json(&absent), json(&stream(zero, 0.4, 0.5)));
}
#[test]
fn deterministic_bounded_and_breathes_at_each_configured_boundary() {
    let first = stream(live(), 0.4, 0.5);
    assert_eq!(json(&first), json(&stream(live(), 0.4, 0.5)));
    let mut boundaries = 0;
    for measure in &first {
        let position = measure.position.unwrap();
        if position.is_intro() {
            continue;
        }
        let notes: Vec<_> =
            measure.notes.iter().filter(|n| n.track == TrackId::Melody.channel()).collect();
        let last = notes.iter().map(|n| n.start_step).max();
        for n in notes {
            assert!((-80..=0).contains(&n.performance.offset), "offset {}", n.performance.offset);
            assert_eq!(n.performance.gate, 870);
            let breathes = [3, 8].contains(&position.bar_in_strain) && Some(n.start_step) == last;
            assert_eq!(
                n.performance.breath > 0,
                breathes,
                "bar {} step {}",
                measure.index,
                n.start_step
            );
            if breathes {
                boundaries += 1;
            }
        }
    }
    assert!(boundaries >= 20);
}
#[test]
fn tension_advances_attacks_and_shortens_gates_arousal_marks_accents() {
    let low = stream(live(), 0.0, 0.0);
    let high = stream(live(), 1.0, 0.0);
    let mean = |s: &[MeasureSnapshot], f: fn(&harmonium_core::report::NoteSnapshot) -> f64| {
        let values: Vec<_> = s
            .iter()
            .flat_map(|m| &m.notes)
            .filter(|n| n.track == 0 && !n.performance.is_neutral())
            .map(f)
            .collect();
        values.iter().sum::<f64>() / values.len() as f64
    };
    assert!(
        mean(&high, |n| f64::from(n.performance.offset))
            < mean(&low, |n| f64::from(n.performance.offset)) - 80.0
    );
    assert!(
        mean(&high, |n| f64::from(n.performance.gate))
            < mean(&low, |n| f64::from(n.performance.gate)) - 150.0
    );
    let energetic = stream(live(), 0.0, 1.0);
    assert!(
        mean(&energetic, |n| f64::from(n.velocity)) > mean(&low, |n| f64::from(n.velocity)) + 4.0
    );
}
#[test]
fn invalid_profile_values_name_the_field() {
    for (from, to, field) in [
        ("jitter = 0.02", "jitter = 0.11", "jitter"),
        ("breath = 0.12", "breath = nan", "breath"),
        ("tension_advance = 0.10", "tension_advance = -0.1", "tension_advance"),
        ("shorten = 0.05", "shorten = 1.1", "shorten"),
        ("velocity_max = 120", "velocity_max = 128", "velocity_max"),
        ("[1.0,-3.0]", "[0.5,-3.0]", "phrase"),
    ] {
        let mut profile = StyleProfile::builtin().clone();
        profile.performance = toml::from_str(&LIVE.replace(from, to)).unwrap();
        let error = profile.validate().unwrap_err().to_string();
        assert!(error.contains("performance.tracks.Melody") && error.contains(field), "{error}");
    }
}

#[test]
fn weighted_rules_obey_position_duration_and_melodic_context() {
    use harmonium_core::timeline::Articulation;
    for context in ["repeated", "ascending", "descending"] {
        let performance: Performance = toml::from_str(&format!(
            r#"
[[tracks.Melody.rules]]
articulation = "Detached"
weight = 1.0
steps = [2,6,10,14]
min_duration = 1
max_duration = 2
context = "{context}"
shorten = 0.25
"#
        ))
        .unwrap();
        let snapshots = stream(performance, 0.0, 0.0);
        let mut selected = 0;
        for measure in snapshots {
            if measure.position.unwrap().is_intro() {
                continue;
            }
            let mut previous = None;
            for note in measure.notes.iter().filter(|n| n.track == 0) {
                let applicable = previous.is_some_and(|pitch| match context {
                    "repeated" => pitch == note.pitch,
                    "ascending" => pitch < note.pitch,
                    _ => pitch > note.pitch,
                }) && [2, 6, 10, 14].contains(&note.start_step)
                    && (1..=2).contains(&note.duration_steps);
                assert_eq!(note.articulation == Articulation::Detached, applicable);
                assert_eq!(note.performance.gate, if applicable { 750 } else { 1000 });
                selected += usize::from(applicable);
                previous = Some(note.pitch);
            }
        }
        assert!(selected > 20, "{context} doit être exercé sur le flux réel");
    }
}

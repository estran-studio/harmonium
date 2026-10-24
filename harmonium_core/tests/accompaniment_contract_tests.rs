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

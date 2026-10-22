//! Les motifs déclaratifs euclidiens atteignent les snapshots sans tirages supplémentaires.
use harmonium_core::{
    StyleProfile, TuningParams,
    params::MusicalParams,
    report::MeasureSnapshot,
    trad::{FeetTable, IntroConfig, Part, ReelArrangement, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn profile(
    rotation: i32,
    pulses: usize,
    steps: usize,
) -> Result<StyleProfile, Box<dyn std::error::Error>> {
    let mut p = StyleProfile::default();
    p.arrangement = ReelArrangement::from_start(1, Part(0));
    p.arrangement.intro = IntroConfig::none();
    p.percussion = FeetTable::from_toml(&format!(
        r#"
velocity_humanize = 0
patterns = [
 {{ density = 0, euclid = {{ pulses = {pulses}, steps = {steps}, rotation = {rotation} }}, track = "Toe" }},
 {{ density = 1, euclid = {{ pulses = {pulses}, steps = {steps}, rotation = {rotation} }}, track = "Toe" }},
 {{ density = 2, euclid = {{ pulses = {pulses}, steps = {steps}, rotation = {rotation} }}, track = "Toe" }}
]
"#
    ))?;
    Ok(StyleProfile::from_toml(&p.to_toml()?)?)
}

#[test]
fn rotated_euclidean_profile_publishes_exact_onsets() -> Result<(), Box<dyn std::error::Error>> {
    for (rotation, expected) in [
        (0, vec![0, 4, 8, 12]),
        (1, vec![1, 5, 9, 13]),
        (-1, vec![3, 7, 11, 15]),
        (33, vec![1, 5, 9, 13]),
    ] {
        let p = profile(rotation, 4, 16)?;
        let mut g = TradGenerator::new(
            MusicalParams::default(),
            TuningParams { profile: p, ..Default::default() },
        );
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for bar in 1..=32 {
            let snap = MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng));
            let onsets: Vec<_> =
                snap.notes.iter().filter(|n| n.track == 4).map(|n| n.start_step).collect();
            assert_eq!(onsets, expected, "bar {bar}, rotation {rotation}");
            assert!(!snap.notes.iter().any(|n| n.track == 3));
        }
    }
    Ok(())
}

#[test]
fn euclidean_bounds_reject_before_generation() {
    for (pulses, steps) in [(1, 0), (17, 16), (1, 4097), (0, 8)] {
        assert!(profile(0, pulses, steps).is_err(), "{pulses}/{steps}");
    }
    let ambiguous = r#"patterns = [{density=0,hits=[],euclid={pulses=4,steps=16}}]"#;
    assert!(FeetTable::from_toml(ambiguous).is_err());
}

//! Paliers de tempo, bornes et priorité des commandes du jeu.
use harmonium_core::{StyleProfile, TuningParams, params::MusicalParams, report::MeasureSnapshot,
    trad::{TradGenerator, dynamics::Curve, set::{SetLength, SetRamp}}};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn ramp() -> SetRamp {
    SetRamp { tempo_bpm: Curve(vec![[0.0,0.0],[1.0,8.0]]),
        intensity_lift: Curve(vec![[0.0,0.0],[1.0,0.08]]),
        gate: Curve(vec![[0.0,0.0],[0.2,1.0],[0.8,1.0],[1.0,0.0]]), ceiling_bpm: Some(240.0) }
}
fn tuning() -> TuningParams {
    let mut p = StyleProfile::default();
    p.arrangement.tours_per_tune = 1;
    p.set.signature = Default::default();
    p.set.length = vec![SetLength { tunes: 3, weight: 1 }];
    p.set.ramp = Some(ramp());
    TuningParams { profile: p }
}
fn stream(config: &TuningParams, intensity: f32, bpm: f32, explicit: bool) -> Vec<MeasureSnapshot> {
    let params = MusicalParams { intensity, bpm, bpm_explicit: explicit, ..Default::default() };
    let mut g = TradGenerator::new(params, config.clone());
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    (1..=300).map(|b| MeasureSnapshot::from_measure(&g.generate_measure(b, &mut rng))).collect()
}

#[test]
fn ramp_is_monotone_by_tune_and_constant_inside_each_tune() {
    let config = tuning();
    for intensity in [0.2,0.5,0.8] {
        let snapshots = stream(&config, intensity, 225.0, false);
        let mut rises = 0;
        for pair in snapshots.windows(2) {
            let (a,b) = (&pair[0],&pair[1]);
            let (pa,pb) = (a.position.unwrap(), b.position.unwrap());
            assert!(b.tempo <= 240.0);
            if pa.tune_index == pb.tune_index { assert_eq!(a.tempo,b.tempo); }
            else if pa.set_index == pb.set_index { assert!(b.tempo >= a.tempo); rises += usize::from(b.tempo > a.tempo); }
        }
        assert!(rises >= 4);
    }
}

#[test]
fn extreme_intensity_erases_the_ramp_including_notes() {
    let config = tuning();
    let mut baseline = config.clone();
    baseline.profile.set.ramp = None;
    for intensity in [0.0,1.0] {
        assert_eq!(serde_json::to_value(stream(&config,intensity,230.0,false)).unwrap(),
            serde_json::to_value(stream(&baseline,intensity,230.0,false)).unwrap());
    }
}

#[test]
fn game_tempo_wins_and_the_ceiling_never_reduces_it() {
    let config = tuning();
    for intensity in [0.0,0.5,1.0] {
        assert!(stream(&config,intensity,197.0,true).iter().all(|s| s.tempo == 197.0));
    }
    let mut baseline = config.clone();
    baseline.profile.set.ramp = None;
    let plain = stream(&baseline,0.5,270.0,false);
    let lifted = stream(&config,0.5,270.0,false);
    assert!(plain.iter().zip(&lifted).all(|(a,b)| a.tempo == b.tempo && b.tempo > 240.0));
    // Le lift d'intensité lui-même ne doit pas franchir le plafond si le jeu reste dessous.
    let mut steep = config.clone();
    steep.profile.dynamics.tempo_offset = Curve(vec![[0.0,0.0],[1.0,200.0]]);
    assert!(stream(&steep,0.5,135.0,false).iter().all(|s| s.tempo <= 240.0));
}

#[test]
fn invalid_ramps_name_the_field() {
    for (field, mutate) in [
        ("set.ramp.tempo_bpm", (|r: &mut SetRamp| r.tempo_bpm.0[1][1] = -1.0) as fn(&mut SetRamp)),
        ("set.ramp.intensity_lift", |r: &mut SetRamp| r.intensity_lift.0[1][1] = f32::NAN),
        ("set.ramp.gate", |r: &mut SetRamp| r.gate.0[0][1] = 0.1),
        ("set.ramp.ceiling_bpm", |r: &mut SetRamp| r.ceiling_bpm = Some(20.0)),
    ] {
        let mut config = tuning();
        mutate(config.profile.set.ramp.as_mut().unwrap());
        assert_eq!(config.profile.validate().unwrap_err().field, field);
    }
}

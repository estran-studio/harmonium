//! Retour du matériel écrit, fenêtre inter-sets et veto de la couleur du jeu.
use harmonium_core::{
    StyleProfile, TuningParams,
    params::MusicalParams,
    report::MeasureSnapshot,
    trad::{
        Mode, TradGenerator,
        arrangement::IntroConfig,
        dynamics::{ColorChoice, Curve},
        set::{SetLength, SetSignature},
    },
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn tuning(rate: f32) -> TuningParams {
    let mut p = StyleProfile::default();
    p.arrangement.tours_per_tune = 1;
    p.arrangement.intro = IntroConfig::none();
    p.set.length = vec![SetLength { tunes: 3, weight: 1 }];
    p.set.ramp = None;
    p.set.end = Default::default();
    p.set.transitions.clear();
    p.set.signature = SetSignature { return_rate: rate, memory: 1 };
    p.performance = Default::default();
    p.ornaments = Default::default();
    TuningParams { profile: p }
}

fn run(
    config: &TuningParams,
    seed: u64,
    changed: Option<(usize, f32, f32)>,
) -> Vec<MeasureSnapshot> {
    let mut g = TradGenerator::new(MusicalParams::default(), config.clone());
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=288)
        .map(|bar| {
            if let Some((at, intensity, valence)) = changed {
                if bar == at {
                    let mut p = g.musical_params.clone();
                    p.intensity = intensity;
                    p.harmony_valence = valence;
                    g.update_params(p);
                    g.invalidate_future(bar - 1);
                }
            }
            MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng))
        })
        .collect()
}
fn melody(s: &[MeasureSnapshot], tune: usize) -> Vec<(usize, u8, usize, usize, u8)> {
    s.iter()
        .filter(|s| s.position.is_some_and(|p| p.tune_index == tune && !p.is_intro()))
        .flat_map(|s| {
            s.notes.iter().filter(|n| n.track == 0).map(move |n| {
                (
                    s.position.unwrap().bar_in_tour(),
                    n.pitch,
                    n.start_step,
                    n.duration_steps,
                    n.velocity,
                )
            })
        })
        .collect()
}

#[test]
fn signature_repeats_previous_set_material_but_never_the_current_set() {
    for seed in 1..=12 {
        let yes = run(&tuning(1.0), seed, None);
        assert_eq!(
            melody(&yes, 4),
            melody(&yes, 2),
            "seed {seed}: mémoire = dernier air du set précédent"
        );
        assert_ne!(melody(&yes, 5), melody(&yes, 4), "pas deux fois le même matériel dans le set");
        assert_ne!(melody(&yes, 1), melody(&yes, 0), "aucune mémoire du set courant");
        let no = run(&tuning(0.0), seed, None);
        assert_ne!(melody(&no, 4), melody(&no, 2), "taux nul : pas de retour");
    }
}

#[test]
fn signature_keeps_written_pitches_when_construction_emotions_change() {
    let mut config = tuning(1.0);
    // Même couleur autorisée, registre émotionnel différent : une graine rejouée ne suffit pas.
    for c in &mut config.profile.melody.key_modes {
        c.key = 2;
        c.mode = Mode::Ionian;
    }
    config.profile.dynamics.color = vec![ColorChoice {
        key: 2,
        mode: Mode::Ionian,
        weight: Curve(vec![[0.0, 1.0], [1.0, 1.0]]),
    }];
    config.profile.dynamics.color_register = Curve(vec![[0.0, -24.0], [1.0, 24.0]]);
    let snapshots = run(&config, 3, Some((97, 0.8, 0.6)));
    let written = |tune| {
        melody(&snapshots, tune)
            .into_iter()
            .map(|(bar, pitch, step, _, _)| (bar, pitch, step))
            .collect::<Vec<_>>()
    };
    assert_eq!(written(4), written(2));
}

#[test]
fn zero_color_weight_excludes_the_memorized_tune() {
    let mut config = tuning(1.0);
    for c in &mut config.profile.melody.key_modes {
        c.key = 2;
        c.mode = Mode::Ionian;
    }
    config.profile.dynamics.color = vec![
        ColorChoice { key: 2, mode: Mode::Ionian, weight: Curve(vec![[0.0, 0.0], [1.0, 0.0]]) },
        ColorChoice { key: 7, mode: Mode::Ionian, weight: Curve(vec![[0.0, 1.0], [1.0, 1.0]]) },
    ];
    let snapshots = run(&config, 3, Some((97, 0.5, 0.6)));
    assert!(
        snapshots
            .iter()
            .filter_map(|s| s.position)
            .filter(|p| p.tune_index >= 3)
            .all(|p| p.key == 7)
    );
    assert_ne!(melody(&snapshots, 4), melody(&snapshots, 2));
}

#[test]
fn seek_rebuilds_signature_memory_and_the_same_snapshots() {
    let config = tuning(1.0);
    let continuous = run(&config, 3, None);
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    let mut g = TradGenerator::new(MusicalParams::default(), config);
    g.silent_advance(150, &mut rng);
    let replay: Vec<_> = (150..=288)
        .map(|b| MeasureSnapshot::from_measure(&g.generate_measure(b, &mut rng)))
        .collect();
    assert_eq!(
        serde_json::to_value(replay).unwrap(),
        serde_json::to_value(&continuous[149..]).unwrap()
    );
}

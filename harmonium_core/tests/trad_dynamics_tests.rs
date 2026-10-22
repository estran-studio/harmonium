//! Contrats musicaux du pilotage continu, observés sur les snapshots.
use harmonium_core::{
    params::{MusicalParams, Style},
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::TrackId,
    trad::{
        ReelArrangement, TradGenerator,
        dynamics::{Curve, REEL_INTENSITY_TOML, StyleDynamics},
    },
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn generator() -> (TradGenerator, ChaCha8Rng) {
    let tuning = TuningParams {
        profile: {
            let mut p = TuningParams::default().profile;
            p.arrangement = ReelArrangement::from_start(1, Default::default());
            p
        },
        ..Default::default()
    };
    let params = MusicalParams {
        style: Style::Reel,
        bpm: tuning.profile.meter.default_bpm,
        ..Default::default()
    };
    (TradGenerator::new(params, tuning), ChaCha8Rng::seed_from_u64(1))
}
fn stream(change: impl Fn(&mut MusicalParams)) -> Vec<MeasureSnapshot> {
    let (mut generator, mut rng) = generator();
    // L'air courant est déjà choisi avant le changement du jeu.
    generator.generate_measure(1, &mut rng);
    change(&mut generator.musical_params);
    (1..=32)
        .map(|b| MeasureSnapshot::from_measure(&generator.generate_measure(b, &mut rng)))
        .collect()
}
fn notes(s: &[MeasureSnapshot], track: TrackId) -> Vec<&NoteSnapshot> {
    s.iter().flat_map(|b| b.notes.iter()).filter(|n| n.track == track.channel()).collect()
}
fn mean(notes: &[&NoteSnapshot], f: impl Fn(&NoteSnapshot) -> f64) -> f64 {
    notes.iter().map(|n| f(n)).sum::<f64>() / notes.len() as f64
}
fn melody(s: &MeasureSnapshot) -> Vec<(usize, u8)> {
    s.notes
        .iter()
        .filter(|n| n.track == TrackId::Melody.channel())
        .map(|n| (n.start_step, n.pitch))
        .collect()
}

#[test]
fn intensity_changes_energy_and_articulation_not_the_current_tune() {
    let low = stream(|p| p.intensity = 0.0);
    let high = stream(|p| p.intensity = 1.0);
    for (a, b) in low.iter().zip(&high) {
        assert_eq!(melody(a), melody(b));
    }
    for track in [TrackId::Melody, TrackId::Heel] {
        assert!(
            mean(&notes(&high, track), |n| f64::from(n.velocity))
                > mean(&notes(&low, track), |n| f64::from(n.velocity))
        );
    }
    assert!(
        mean(&notes(&high, TrackId::Melody), |n| n.duration_steps as f64)
            < mean(&notes(&low, TrackId::Melody), |n| n.duration_steps as f64)
    );
    assert!(
        notes(&high, TrackId::Heel).len() + notes(&high, TrackId::Toe).len()
            > notes(&low, TrackId::Heel).len() + notes(&low, TrackId::Toe).len()
    );
    assert!(notes(&high, TrackId::Chords).len() > notes(&low, TrackId::Chords).len());
    assert_eq!(low[0].tempo, 220.0);
    assert_eq!(high[0].tempo, 245.0);
}

#[test]
fn intensity_automation_preserves_aabb_pitches() {
    let (mut generator, mut rng) = generator();
    let snapshots: Vec<_> = (1..=32)
        .map(|bar| {
            generator.musical_params.intensity = (bar - 1) as f32 / 31.0;
            MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng))
        })
        .collect();
    for start in [0, 16] {
        for i in 0..8 {
            assert_eq!(melody(&snapshots[start + i]), melody(&snapshots[start + i + 8]));
        }
    }
}

#[test]
fn color_bias_is_a_majority_on_fifty_tunes_and_waits_for_the_next_tune() {
    for (valence, minor) in [(-1.0, true), (1.0, false)] {
        let (mut generator, mut rng) = generator();
        generator.generate_measure(1, &mut rng);
        let original = generator.tune_for_bar(1).unwrap().plan.clone();
        generator.musical_params.harmony_valence = valence;
        generator.generate_measure(2, &mut rng);
        assert_eq!(generator.tune_for_bar(2).unwrap().plan, original);
        let matching = (1..=50)
            .filter(|i| {
                let bar = 32 * i + 1;
                generator.generate_measure(bar, &mut rng);
                generator.tune_for_bar(bar).unwrap().plan.mode.is_minor() == minor
            })
            .count();
        assert!(matching > 25, "valence {valence}: {matching}/50");
    }
}

#[test]
fn density_changes_texture_and_tension_changes_offbeats_and_harmony() {
    let sparse = stream(|p| p.rhythm_density = 0.0);
    let dense = stream(|p| p.rhythm_density = 1.0);
    assert!(notes(&sparse, TrackId::Bass).is_empty());
    assert!(!notes(&dense, TrackId::Bass).is_empty());
    assert!(notes(&dense, TrackId::Melody).len() > notes(&sparse, TrackId::Melody).len());
    assert!(notes(&dense, TrackId::Chords).len() > notes(&sparse, TrackId::Chords).len());
    let relaxed = stream(|p| p.harmony_tension = 0.0);
    let tense = stream(|p| p.harmony_tension = 1.0);
    for (a, b) in relaxed.iter().zip(&tense) {
        assert_eq!(melody(a), melody(b));
    }
    let offbeat_mean = |s: &[MeasureSnapshot]| {
        let n: Vec<_> =
            notes(s, TrackId::Melody).into_iter().filter(|n| n.start_step % 4 != 0).collect();
        mean(&n, |n| f64::from(n.velocity))
    };
    assert!(offbeat_mean(&tense) > offbeat_mean(&relaxed));
    assert!(
        mean(&notes(&tense, TrackId::Chords), |n| n.duration_steps as f64)
            > mean(&notes(&relaxed, TrackId::Chords), |n| n.duration_steps as f64)
    );
    assert!(relaxed.iter().zip(&tense).any(|(a, b)| {
        a.notes
            .iter()
            .filter(|n| n.track == TrackId::Chords.channel())
            .map(|n| n.pitch)
            .collect::<Vec<_>>()
            != b.notes
                .iter()
                .filter(|n| n.track == TrackId::Chords.channel())
                .map(|n| n.pitch)
                .collect::<Vec<_>>()
    }));
}

#[test]
fn style_curves_reject_invalid_profiles_and_interpolate() {
    let mut dynamics = StyleDynamics::from_toml(REEL_INTENSITY_TOML).unwrap();
    assert_eq!(Curve(vec![[0.0, 10.0], [0.5, 20.0], [1.0, 0.0]]).eval(0.75), 10.0);
    dynamics.tempo_offset.0[1][0] = 0.0;
    assert!(dynamics.validate().unwrap_err().contains("tempo_offset"));
    let mut dynamics = StyleDynamics::default();
    dynamics.feet_velocity.0[0][1] = f32::NAN;
    assert!(dynamics.validate().unwrap_err().contains("feet_velocity"));
    let mut dynamics = StyleDynamics::default();
    for c in &mut dynamics.color {
        c.weight = Curve(vec![[0.0, 0.0], [1.0, 0.0]]);
    }
    assert!(dynamics.validate().unwrap_err().contains("poids total nul"));
}

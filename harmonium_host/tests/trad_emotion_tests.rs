//! Pilotage du jeu : quatre axes indépendants et réponse à la mesure suivante.
use std::time::Instant;

use harmonium::{audio::create_offline_engine, playback::PlaybackCommand};
use harmonium_core::{
    report::MeasureSnapshot, timeline::TrackId, trad::ReelArrangement, tuning::TuningParams,
};

fn melody(snapshot: &MeasureSnapshot) -> Vec<(usize, u8)> {
    snapshot
        .notes
        .iter()
        .filter(|n| n.track == TrackId::Melody.channel())
        .map(|n| (n.start_step, n.pitch))
        .collect()
}
fn mean_velocity(snapshot: &MeasureSnapshot) -> f32 {
    let notes: Vec<_> =
        snapshot.notes.iter().filter(|n| n.track == TrackId::Melody.channel()).collect();
    notes.iter().map(|n| f32::from(n.velocity)).sum::<f32>() / notes.len() as f32
}

#[test]
fn each_emotion_changes_the_next_bar_without_touching_current_bar_or_pulse() -> Result<(), String> {
    for (name, axes) in [
        ("arousal", [1.0, 0.3, 0.5, 0.3]),
        ("valence", [0.5, 1.0, 0.5, 0.3]),
        ("density", [0.5, 0.3, 0.0, 0.3]),
        ("tension", [0.5, 0.3, 0.5, 1.0]),
    ] {
        let (mut composer, mut playback, mut tx, _reports, mut clock, _recordings) =
            create_offline_engine(None, None, &[], 44100.0)?;
        composer.set_seed(1);

        composer.set_tuning(TuningParams {
            profile: {
                let mut p = TuningParams::default().profile;
                p.arrangement = ReelArrangement::from_start(3, Default::default());
                p
            },
            ..Default::default()
        });
        composer.use_emotion_mode();
        composer.set_emotions(0.5, 0.3, 0.5, 0.3);
        composer.invalidate_future();
        if let Some((bar, _)) = composer.take_pending_drop() {
            tx.push(PlaybackCommand::DropFrom { bar }).map_err(|_| "drop queue full")?;
        }
        composer.generate_bars(8);
        let before = composer.take_snapshots();
        let mut buffer = [0.0; 128];
        // Appeler au milieu de la première pulse, pas à une frontière favorable.
        let call_clock = loop {
            playback.process_buffer(&mut buffer, 2, Instant::now());
            let c = clock.read();
            if c.position.bar == 1 && c.position.step >= 3 {
                break c;
            }
        };
        composer.set_emotions(axes[0], axes[1], axes[2], axes[3]);
        composer.invalidate_future();
        let (drop_bar, _) = composer.take_pending_drop().ok_or("drop absent")?;
        assert_eq!(drop_bar, 2, "{name}: la mesure audible ne doit pas être invalidée");
        tx.push(PlaybackCommand::DropFrom { bar: drop_bar }).map_err(|_| "drop queue full")?;
        composer.generate_bars(8);
        let after = composer.take_snapshots();
        let next = after.iter().find(|b| b.index == 2).ok_or("mesure 2 absente")?;
        let old = before.iter().find(|b| b.index == 2).ok_or("mesure 2 initiale absente")?;
        match name {
            "arousal" => {
                assert!(mean_velocity(next) > mean_velocity(old));
                assert!((220.0..=245.0).contains(&next.tempo));
                assert_eq!(melody(next), melody(old));
            }
            "valence" => {
                assert!(mean_velocity(next) > mean_velocity(old));
                assert_eq!(melody(next), melody(old));
            }
            "density" => assert!(next.notes.len() < old.notes.len()),
            "tension" => assert!(
                next.notes
                    .iter()
                    .filter(|n| n.track == TrackId::Melody.channel())
                    .map(|n| n.duration_steps)
                    .sum::<usize>()
                    < old
                        .notes
                        .iter()
                        .filter(|n| n.track == TrackId::Melody.channel())
                        .map(|n| n.duration_steps)
                        .sum::<usize>()
            ),
            _ => unreachable!(),
        }
        let mut previous = call_clock;
        let mut pulse_lengths = Vec::new();
        let mut reached = false;
        for _ in 0..2000 {
            playback.process_buffer(&mut buffer, 2, Instant::now());
            let current = clock.read();
            assert!(current.position.bar >= previous.position.bar, "{name}: recul du playhead");
            assert!(current.position.bar <= previous.position.bar + 1, "{name}: saut de mesure");
            if current.pulse_start_frame == previous.pulse_start_frame {
                assert_eq!(
                    current.next_pulse_frame, previous.next_pulse_frame,
                    "{name}: pulse en cours déformée"
                );
            } else {
                assert_eq!(
                    current.pulse_start_frame, previous.next_pulse_frame,
                    "{name}: pulse discontinue"
                );
                if current.position.bar == 2 {
                    pulse_lengths.push(current.next_pulse_frame - current.pulse_start_frame);
                }
            }
            if current.position.bar == 3 {
                reached = true;
                break;
            }
            previous = current;
        }
        assert!(reached, "{name}: rendu bloqué");
        assert_eq!(pulse_lengths.len(), 2, "{name}: les deux pulses restent présentes");
        if name == "arousal" {
            assert!(
                pulse_lengths[0] > pulse_lengths[1],
                "tempo doit monter par rampe, pas sauter: {pulse_lengths:?}"
            );
            assert_eq!(pulse_lengths[1], 2 * (44100.0_f64 * 60.0 / 245.0).round() as u64);
        }
        println!(
            "{name}: appel mesure {}, premier effet mesure {}, latence 1 mesure",
            call_clock.position.bar, next.index
        );
    }
    Ok(())
}

#[test]
fn emotion_mapping_reads_style_tempo_data_and_preserves_runtime_contract() -> Result<(), String> {
    let (mut composer, _playback, _tx, _rx, _clock, _recordings) =
            create_offline_engine(None, None, &[], 44100.0)?;

    let mut tuning = TuningParams::default();
    tuning.profile.meter.default_bpm = 250.0;
    composer.set_tuning(tuning);
    composer.set_swing_ratio(0.61);
    composer.set_feet_density(2);
    composer.set_end_after_tour(true);
    composer.use_emotion_mode();
    composer.set_emotions(1.0, 0.0, 0.8, 0.9);
    composer.invalidate_future();
    composer.generate_bars(1);
    let params = composer.musical_params();
    assert_eq!(params.bpm, 250.0);
    assert_eq!(params.swing_ratio, 0.61);
    assert_eq!(params.time_signature.numerator, 2);
    assert!(params.end_after_tour);
    assert_eq!(params.feet_density, 2);
    let first = composer.take_snapshots().into_iter().next().ok_or("snapshot absent")?;
    assert_eq!(first.tempo, 265.0);
    Ok(())
}

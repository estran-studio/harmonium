//! Invalidation pendant une phrase : mode audible et horloge sans discontinuité.
use std::time::Instant;

use harmonium::{audio::create_offline_engine, playback::PlaybackCommand};
use harmonium_core::{
    ornaments::OrnamentKind,
    timeline::TrackId,
    trad::{Mode, ReelArrangement},
    tuning::TuningParams,
};

#[test]
fn mode_changes_at_phrase_boundary_without_restarting_the_pulse() -> Result<(), String> {
    let (mut composer, mut playback, mut tx, _reports, mut clock, _recordings) =
        create_offline_engine(None, None, 44100.0)?;
    composer.set_seed(1);

    composer.set_tuning(TuningParams {
        profile: {
            let mut p = TuningParams::default().profile;
            p.arrangement = ReelArrangement::from_start(3, Default::default());
            p.ornaments.types.retain(|kind, _| *kind == OrnamentKind::Passing);
            p
        },
        ..Default::default()
    });
    composer.use_emotion_mode();
    composer.set_emotions(0.5, 0.4, 0.5, 0.3);
    composer.invalidate_future();
    if let Some((bar, _)) = composer.take_pending_drop() {
        tx.push(PlaybackCommand::DropFrom { bar }).map_err(|_| "drop queue full")?;
    }
    composer.generate_bars(16);
    let original = composer.take_snapshots();
    let initial = original[0].position.ok_or("position absent")?;
    assert!(!initial.mode.is_minor());
    let mut buffer = [0.0; 128];
    let mut previous = None;
    let mut dark_applied = false;
    let mut bright_applied = false;
    let mut reached = false;
    for _ in 0..30000 {
        playback.process_buffer(&mut buffer, 2, Instant::now());
        let current = clock.read();
        if let Some(old) = previous {
            let old: harmonium::TransportClock = old;
            assert!(current.position.bar >= old.position.bar);
            assert!(current.position.bar <= old.position.bar + 1);
            // La publication initiale précède la première pulse audio.
            if old.next_pulse_frame > 0 {
                if current.pulse_start_frame == old.pulse_start_frame {
                    assert_eq!(current.next_pulse_frame, old.next_pulse_frame);
                } else {
                    assert_eq!(current.pulse_start_frame, old.next_pulse_frame);
                }
            }
        }
        let change = if !dark_applied && current.position.bar == 3 && current.position.step >= 3 {
            dark_applied = true;
            Some((-0.9, 5, true))
        } else if !bright_applied && current.position.bar == 7 && current.position.step >= 3 {
            bright_applied = true;
            Some((0.95, 9, false))
        } else {
            None
        };
        if let Some((valence, boundary, dark)) = change {
            composer.set_emotions(0.5, valence, 0.5, 0.3);
            composer.invalidate_future();
            let (drop_bar, _) = composer.take_pending_drop().ok_or("drop absent")?;
            assert_eq!(drop_bar, current.position.bar + 1);
            tx.push(PlaybackCommand::DropFrom { bar: drop_bar }).map_err(|_| "drop queue full")?;
            composer.generate_bars(12);
            let regenerated = composer.take_snapshots();
            let next = regenerated.iter().find(|s| s.index == boundary).ok_or("boundary absent")?;
            let position = next.position.ok_or("position absent")?;
            assert_eq!(position.key, initial.key);
            assert_eq!(position.mode.is_minor(), dark);
            for s in regenerated.iter().filter(|s| s.index < boundary) {
                assert_eq!(s.position.ok_or("position absent")?.mode.is_minor(), !dark);
            }
            let reference =
                original.iter().find(|s| s.index == boundary).ok_or("reference absent")?;
            let before: Vec<_> =
                reference.notes.iter().filter(|n| n.track == TrackId::Melody.channel()).collect();
            let after: Vec<_> =
                next.notes.iter().filter(|n| n.track == TrackId::Melody.channel()).collect();
            assert_eq!(before.len(), after.len());
            for (a, b) in before.iter().zip(after) {
                assert_eq!(
                    initial.mode.scale_degree_of_midi(initial.key, a.pitch),
                    position.mode.scale_degree_of_midi(position.key, b.pitch)
                );
            }
            println!(
                "valence {valence}: appel M{}, mode {:?} M{boundary}, latence {} mesures",
                current.position.bar,
                position.mode,
                boundary - current.position.bar
            );
        }
        if current.position.bar == 10 {
            reached = true;
            break;
        }
        previous = Some(current);
    }
    assert!(dark_applied && bright_applied && reached);
    assert!(matches!(initial.mode, Mode::Ionian | Mode::Mixolydian));
    Ok(())
}

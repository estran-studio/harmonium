//! La pulse publiée reste contiguë au travers des raccords et pauses de set.
use std::time::Instant;

use harmonium::{audio::create_offline_engine, playback::PlaybackCommand};
use harmonium_core::{
    StyleProfile,
    trad::set::{SetGap, SetLength, SetTransition, TransitionKind},
};

#[test]
fn beat_info_crosses_every_transition_and_set_gap_without_a_hole() -> Result<(), String> {
    for kind in [
        TransitionKind::Direct,
        TransitionKind::Call,
        TransitionKind::Held,
        TransitionKind::Silence,
    ] {
        let mut profile = StyleProfile::default();
        profile.arrangement.tours_per_tune = 1;
        profile.set.length = vec![SetLength { tunes: 2, weight: 1 }];
        profile.set.signature = Default::default();
        profile.set.transitions = vec![SetTransition {
            kind,
            weight: 1,
            bars: matches!(kind, TransitionKind::Call | TransitionKind::Silence).then_some(2),
        }];
        profile.set.end.outro = true;
        profile.set.end.gap_bars = vec![SetGap { bars: 2, weight: 1 }];
        let (mut composer, mut playback, mut commands, mut reports, mut clock_reader, _recordings) =
            create_offline_engine(None, None, &[], 44100.0)?;
        composer.set_profile(profile)?;
        composer.set_seed(3);
        composer.sync_generator();
        if let Some((bar, _)) = composer.take_pending_drop() {
            commands.push(PlaybackCommand::DropFrom { bar }).map_err(|_| "file pleine")?;
        }
        composer.set_writehead_lookahead(16);
        composer.generate_bars(8);
        commands
            .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()))
            .map_err(|_| "file pleine")?;
        let mut buffer = [0.0; 2048];
        let mut last_bar = 1;
        let mut previous_pulse = None;
        let mut previous_prediction = None;
        let mut boundaries = 0;
        for _ in 0..12000 {
            composer.generate_ahead();
            playback.process_buffer(&mut buffer, 2, Instant::now());
            let clock = clock_reader.read();
            let _ = composer.take_snapshots();
            while reports.pop().is_ok() {}
            assert!(
                clock.position.bar == last_bar || clock.position.bar == last_bar + 1,
                "{kind:?}: mesures non contiguës {last_bar} -> {}",
                clock.position.bar
            );
            last_bar = clock.position.bar;
            if clock.next_pulse_frame == 0 {
                continue;
            }
            if previous_pulse != Some(clock.pulse_start_frame) {
                if let Some(predicted) = previous_prediction {
                    assert_eq!(clock.pulse_start_frame, predicted, "{kind:?}: trou de pulse");
                    boundaries += 1;
                }
                previous_prediction = Some(clock.next_pulse_frame);
                previous_pulse = Some(clock.pulse_start_frame);
            }
            if last_bar >= 150 {
                break;
            }
        }
        assert!(
            last_bar >= 150 && boundaries > 250,
            "{kind:?}: clock {last_bar}, playhead {}, writehead {}, pulses {boundaries}",
            composer.playhead_bar(),
            composer.writehead_position()
        );
    }
    Ok(())
}

//! La grille d'un profil pilote aussi le playhead et la prédiction de pulse.
use harmonium::{audio::create_offline_engine, playback::PlaybackCommand};
use harmonium_core::{
    StyleProfile,
    trad::{IntroConfig, Part, ReelArrangement},
};

#[test]
fn larger_profile_grid_plays_every_step_and_predicts_long_pulses() -> Result<(), String> {
    for pulses_per_bar in [1, 2] {
        let mut profile = StyleProfile::default();
        profile.meta.id = "double_grid".into();
        profile.meter.steps_per_bar *= 2;
        profile.meter.steps_per_unit *= 2;
        profile.meter.pulses_per_bar = pulses_per_bar;
        profile.meter.steps_per_pulse = profile.meter.steps_per_bar / pulses_per_bar;
        profile.meter.default_swing_ratio = 0.5;
        profile.harmony.steps_per_chord *= 2;
        profile.accompaniment.bass_steps.iter_mut().for_each(|s| *s *= 2);
        profile.accompaniment.chop_steps.iter_mut().for_each(|s| *s *= 2);
        profile.accompaniment.strong_steps.iter_mut().for_each(|s| *s *= 2);
        profile.accompaniment.bass_duration_steps *= 2;
        profile.accompaniment.chop_duration_steps *= 2;
        for hit in profile.percussion.patterns.iter_mut().flat_map(|p| &mut p.hits) {
            hit.step *= 2;
        }
        for step in profile.dynamics.chop_patterns.iter_mut().flatten() {
            *step *= 2;
        }
        profile.arrangement = ReelArrangement::from_start(1, Part(profile.form.sequence[0]));
        profile.arrangement.intro = IntroConfig::none();
        let meter = profile.meter.clone();
        let step_frames = (44100.0 * 60.0 / f64::from(meter.default_bpm) / 8.0) as u64;
        let expected_pulse_frames = step_frames * meter.steps_per_pulse as u64;
        let (mut composer, mut playback, mut commands, _reports, mut clock, _recordings) =
            create_offline_engine(None, None, 44100.0)?;
        composer.set_profile(profile)?;
        composer.set_seed(1);
        composer.generate_bars(3);
        commands
            .push(PlaybackCommand::UpdateMusicalParams(Box::new(composer.musical_params().clone())))
            .map_err(|_| "file pleine")?;
        let mut buffer = [0.0f32; 128];
        let mut last_step = 0;
        let mut saw_last_pulse = false;
        let mut saw_second_bar = false;
        for _ in 0..2000 {
            playback.process_buffer(&mut buffer, 2, std::time::Instant::now());
            let state = clock.read();
            if state.next_pulse_frame == 0 {
                continue;
            }
            if state.position.bar == 1 {
                last_step = last_step.max(state.position.step);
                saw_last_pulse |= usize::from(state.pulse_in_bar) + 1 == pulses_per_bar;
                assert_eq!(state.next_pulse_frame - state.pulse_start_frame, expected_pulse_frames);
            } else if state.position.bar == 2 {
                saw_second_bar = true;
                break;
            }
        }
        assert!(saw_second_bar);
        assert!(saw_last_pulse);
    }
    Ok(())
}

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
        if let Some(a) = &mut profile.accompaniment {
            a.bass_steps.iter_mut().for_each(|s| *s *= 2);
            a.chop_steps.iter_mut().for_each(|s| *s *= 2);
            a.strong_steps.iter_mut().for_each(|s| *s *= 2);
            a.bass_duration_steps *= 2;
            a.chop_duration_steps *= 2;
        }
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

#[test]
fn compound_meter_beat_info_contract() -> Result<(), String> {
    let profile = StyleProfile::distributed()
        .iter()
        .find(|p| p.meter.time_signature == harmonium_core::params::TimeSignature::new(6, 8))
        .expect("profil 6/8 distribué");
    let (mut composer, mut playback, mut commands, _reports, mut clock, _recordings) =
        create_offline_engine(None, None, 44100.0)?;
    composer.set_profile(profile.clone())?;
    composer.set_seed(1);
    composer.generate_bars(4);
    commands
        .push(PlaybackCommand::UpdateMusicalParams(Box::new(composer.musical_params().clone())))
        .map_err(|_| "file pleine")?;
    let mut buffer = [0.0f32; 128];
    let mut saw_downbeat = false;
    let mut saw_backbeat = false;
    for _ in 0..4000 {
        playback.process_buffer(&mut buffer, 2, std::time::Instant::now());
        let state = clock.read();
        if state.next_pulse_frame == 0 {
            continue;
        }
        let beat_info = harmonium::clock::BeatInfo::from_clock(&state, std::time::Instant::now());
        assert_eq!(beat_info.pulses_per_bar, 2);
        assert!(beat_info.pulse == 1 || beat_info.pulse == 2);
        if beat_info.is_downbeat {
            saw_downbeat = true;
            assert_eq!(beat_info.pulse, 1);
        }
        if beat_info.is_backbeat {
            saw_backbeat = true;
            assert_eq!(beat_info.pulse, 2);
        }
        if state.position.bar >= 2 {
            break;
        }
    }
    assert!(saw_downbeat, "doit observer un downbeat (pulse 1)");
    assert!(saw_backbeat, "doit observer un backbeat (pulse 2)");
    Ok(())
}

#[test]
fn triple_meter_beat_info_contract() -> Result<(), String> {
    let profile = StyleProfile::distributed()
        .iter()
        .find(|p| p.meter.time_signature == harmonium_core::params::TimeSignature::new(3, 4))
        .expect("profil 3/4 distribué");
    let (mut composer, mut playback, mut commands, _reports, mut clock, _recordings) =
        create_offline_engine(None, None, 44100.0)?;
    composer.set_profile(profile.clone())?;
    composer.set_seed(1);
    composer.generate_bars(4);
    commands
        .push(PlaybackCommand::UpdateMusicalParams(Box::new(composer.musical_params().clone())))
        .map_err(|_| "file pleine")?;
    let mut buffer = [0.0f32; 128];
    let mut saw_downbeat = false;
    let mut saw_pulse2 = false;
    let mut saw_pulse3 = false;
    for _ in 0..4000 {
        playback.process_buffer(&mut buffer, 2, std::time::Instant::now());
        let state = clock.read();
        if state.next_pulse_frame == 0 {
            continue;
        }
        let beat_info = harmonium::clock::BeatInfo::from_clock(&state, std::time::Instant::now());
        assert_eq!(beat_info.pulses_per_bar, 3);
        assert!((1..=3).contains(&beat_info.pulse));
        if beat_info.is_downbeat {
            saw_downbeat = true;
            assert_eq!(beat_info.pulse, 1);
        }
        if beat_info.pulse == 2 {
            saw_pulse2 = true;
        }
        if beat_info.pulse == 3 {
            saw_pulse3 = true;
        }
        if state.position.bar >= 2 {
            break;
        }
    }
    assert!(saw_downbeat, "doit observer un downbeat (pulse 1)");
    assert!(saw_pulse2, "doit observer pulse 2");
    assert!(saw_pulse3, "doit observer pulse 3");
    Ok(())
}

/// Changer de profil applique ses défauts (tempo, swing, signature) et met en file les
/// programmes SoundFont de ses instruments ; revenir au reel restaure les siens.
#[test]
fn switching_profiles_applies_defaults_and_queues_programs() -> Result<(), String> {
    use harmonium_core::params::TimeSignature;

    let (mut composer, mut playback, mut commands, _reports, _clock, _recordings) =
        create_offline_engine(None, None, 44100.0)?;

    composer.set_profile_id("gigue_quebecoise")?;
    let params = composer.musical_params();
    assert!((params.bpm - 172.5).abs() < f32::EPSILON);
    assert!((params.swing_ratio - 0.5).abs() < f32::EPSILON);
    assert_eq!(params.time_signature, TimeSignature::new(6, 8));
    let _ = composer.take_pending_program_changes();

    let mut custom = StyleProfile::named("gigue_quebecoise").map_err(|e| e.to_string())?;
    custom.meta.id = "gigue_flute".into();
    custom.instruments.melody_program = 73;
    composer.set_profile(custom)?;
    let programs = composer.take_pending_program_changes();
    assert!(programs.contains(&(0, 73)), "programme mélodie attendu : {programs:?}");
    for (channel, program) in programs {
        commands
            .push(PlaybackCommand::ProgramChange { channel, program })
            .map_err(|_| "file de commandes pleine")?;
    }
    let mut buffer = [0.0f32; 512];
    playback.process_buffer(&mut buffer, 2, std::time::Instant::now());

    composer.set_profile_id("reel")?;
    let params = composer.musical_params();
    assert!((params.bpm - 230.0).abs() < f32::EPSILON);
    assert!((params.swing_ratio - 0.56).abs() < f32::EPSILON);
    assert_eq!(params.time_signature, TimeSignature::new(2, 2));
    assert!(composer.take_pending_program_changes().contains(&(0, 40)));
    Ok(())
}

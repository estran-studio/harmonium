//! Le fichier MIDI observable porte le même état de mixage que la lecture.
use std::time::Instant;

use harmonium::{audio::create_offline_engine, playback::PlaybackCommand};
use harmonium_core::{
    StyleProfile,
    events::{AudioRecordFormat, RecordFormat},
    mix::TrackMix,
};
use midly::{MidiMessage, Smf, TrackEventKind};

#[test]
fn midi_mix_initializes_every_track_and_reapplies_on_profile_change() -> Result<(), String> {
    let (mut composer, mut playback, mut commands, _, _, finished) =
        create_offline_engine(None, None, 44100.0)?;
    let mut profile = StyleProfile::builtin().clone();
    profile.mix.tracks.melody = TrackMix { level_db: -12.0, pan: -1.0, reverb_send: 1.0 };
    profile.mix.tracks.chords = TrackMix { level_db: -6.0, pan: 1.0, reverb_send: 0.5 };
    profile.mix.tracks.bass = TrackMix { level_db: 0.0, pan: 0.0, reverb_send: 0.0 };
    profile.mix.tracks.heel = TrackMix { level_db: -12.0, pan: -0.5, reverb_send: 0.25 };
    profile.mix.tracks.toe = TrackMix { level_db: -6.0, pan: 0.5, reverb_send: 0.75 };
    composer.set_profile(profile.clone())?;
    commands
        .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()))
        .map_err(|_| "commands full")?;
    commands
        .push(PlaybackCommand::StartRecording(AudioRecordFormat::Midi))
        .map_err(|_| "commands full")?;
    let mut buffer = [0.0; 512];
    playback.process_buffer(&mut buffer, 2, Instant::now());
    // REPL gain updates the same controller, preserving pan and reverb.
    commands
        .push(PlaybackCommand::SetChannelGain { channel: 0, gain: 0.0 })
        .map_err(|_| "commands full")?;
    playback.process_buffer(&mut buffer, 2, Instant::now());
    profile.mix.tracks.melody = TrackMix { level_db: 0.0, pan: 1.0, reverb_send: 0.0 };
    composer.set_profile(profile)?;
    commands
        .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()))
        .map_err(|_| "commands full")?;
    playback.process_buffer(&mut buffer, 2, Instant::now());
    commands
        .push(PlaybackCommand::StopRecording(AudioRecordFormat::Midi))
        .map_err(|_| "commands full")?;
    playback.process_buffer(&mut buffer, 2, Instant::now());
    let queue = finished.lock().map_err(|e| e.to_string())?;
    let bytes =
        &queue.iter().find(|(format, _)| *format == RecordFormat::Midi).ok_or("MIDI absent")?.1;
    let smf = Smf::parse(bytes).map_err(|e| e.to_string())?;
    let mut controllers = Vec::new();
    let mut tick = 0;
    for event in &smf.tracks[0] {
        tick += event.delta.as_int();
        if let TrackEventKind::Midi {
            channel,
            message: MidiMessage::Controller { controller, value },
        } = event.kind
        {
            controllers.push((tick, channel.as_int(), controller.as_int(), value.as_int()));
        }
    }
    let expected = [
        (0, [64, 0, 127]),
        (1, [90, 127, 64]),
        (2, [127, 64, 0]),
        (9, [64, 32, 32]),
        (9, [90, 96, 95]),
    ];
    let initial: Vec<_> = expected
        .into_iter()
        .flat_map(|(channel, values)| {
            [7, 10, 91].into_iter().zip(values).map(move |(cc, value)| (0, channel, cc, value))
        })
        .collect();
    assert_eq!(&controllers[..15], &initial, "CC7/10/91 avant la première note, Toe gagne sur ch9");
    let melody: Vec<_> = controllers
        .into_iter()
        .filter(|(_, channel, _, _)| *channel == 0)
        .map(|(_, _, cc, value)| (cc, value))
        .collect();
    assert_eq!(
        melody,
        [(7, 64), (10, 0), (91, 127), (7, 0), (10, 0), (91, 127), (7, 127), (10, 127), (91, 0)]
    );
    Ok(())
}

#[test]
fn next_tune_restores_profile_levels_but_preserves_solo_mutes() -> Result<(), String> {
    let (mut composer, mut playback, mut commands, mut reports, _, finished) =
        create_offline_engine(None, None, 44100.0)?;
    composer.set_seed(3);
    composer.set_tours_per_tune(1)?;
    composer.generate_bars(40);
    commands
        .push(PlaybackCommand::StartRecording(AudioRecordFormat::Midi))
        .map_err(|_| "commands full")?;
    let mut buffer = [0.0; 512];
    playback.process_buffer(&mut buffer, 2, Instant::now());
    commands
        .push(PlaybackCommand::SetChannelGain { channel: 0, gain: 0.0 })
        .map_err(|_| "commands full")?;
    commands
        .push(PlaybackCommand::SetChannelMute { channel: 1, muted: true })
        .map_err(|_| "commands full")?;
    while composer.playhead_bar() < 38 {
        composer.generate_ahead();
        playback.process_buffer(&mut buffer, 2, Instant::now());
        while reports.pop().is_ok() {}
    }
    commands
        .push(PlaybackCommand::StopRecording(AudioRecordFormat::Midi))
        .map_err(|_| "commands full")?;
    playback.process_buffer(&mut buffer, 2, Instant::now());
    let queue = finished.lock().map_err(|e| e.to_string())?;
    let bytes =
        &queue.iter().find(|(format, _)| *format == RecordFormat::Midi).ok_or("MIDI absent")?.1;
    let smf = Smf::parse(bytes).map_err(|e| e.to_string())?;
    let mut volumes = Vec::new();
    for event in &smf.tracks[0] {
        match event.kind {
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::Controller { controller, value },
            } if channel.as_int() == 0 && controller.as_int() == 7 => volumes.push(value.as_int()),
            TrackEventKind::Midi { channel, message: MidiMessage::NoteOn { .. } } => assert_ne!(
                channel.as_int(),
                1,
                "le changement d'air ne réactive pas la piste mutée"
            ),
            _ => {}
        }
    }
    volumes.dedup();
    assert_eq!(volumes, [127, 0, 127], "le nouvel air restaure le niveau du profil");
    Ok(())
}

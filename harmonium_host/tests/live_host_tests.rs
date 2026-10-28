//! L'hôte vivant exécute les `EngineCommand` : chaque commande a son effet
//! observable sur la sortie publiée (paramètres, transport, horloge, fichiers).
//!
//! Hôte sans carte son (`LiveHost::detached`) : le test fait tourner le
//! `PlaybackEngine` lui-même, tampon par tampon.
use std::time::Instant;

use harmonium::{LiveHost, playback::PlaybackEngine};
use harmonium_core::{EngineCommand, events::RecordFormat};

const SAMPLE_RATE: f64 = 44100.0;

fn host() -> Result<(LiveHost, PlaybackEngine), String> {
    LiveHost::detached(None, None, &[], SAMPLE_RATE)
}

/// Compose et joue `buffers` tampons.
fn run(host: &mut LiveHost, playback: &mut PlaybackEngine, buffers: usize) {
    let mut buffer = [0.0f32; 1024];
    for _ in 0..buffers {
        host.pump();
        playback.process_buffer(&mut buffer, 2, Instant::now());
    }
}

#[test]
fn emotion_command_enters_emotion_mode_and_maps_the_four_axes() -> Result<(), String> {
    let (mut host, mut playback) = host()?;
    run(&mut host, &mut playback, 4);
    let before = host.emotions();

    host.apply(EngineCommand::SetEmotionParams {
        arousal: 0.95,
        valence: 0.2,
        density: 0.7,
        tension: 0.3,
    })?;
    assert_ne!(host.emotions(), before);
    assert_eq!(host.emotions(), [0.95, 0.2, 0.7, 0.3]);
    let params = host.musical_params();
    assert_eq!(params.harmony_valence, 0.2);
    assert_eq!(params.rhythm_density, 0.7);
    assert_eq!(params.harmony_tension, 0.3);
    Ok(())
}

#[test]
fn generation_commands_change_the_parameters_and_the_published_report() -> Result<(), String> {
    let (mut host, mut playback) = host()?;
    host.apply(EngineCommand::SetSwing(0.9))?;
    host.apply(EngineCommand::SetFeetDensity(2))?;
    host.apply(EngineCommand::SetIntensity(0.8))?;
    host.apply(EngineCommand::EnableMelody(false))?;
    host.apply(EngineCommand::SetTimeSignature { numerator: 3, denominator: 4 })?;
    let params = host.musical_params();
    assert_eq!(params.swing_ratio, 0.75, "le swing est borné par l'hôte");
    assert_eq!(params.feet_density, 2);
    assert!((params.intensity - 0.8).abs() < 1e-6);
    assert!(!params.enable_melody && params.enable_rhythm && params.enable_harmony);
    assert_eq!((params.time_signature.numerator, params.time_signature.denominator), (3, 4));

    run(&mut host, &mut playback, 100);
    let report = host.poll_state().ok_or("aucun rapport publié")?;
    assert_eq!(report.musical_params.swing_ratio, 0.75);
    assert_eq!(report.musical_params.feet_density, 2);
    assert!(!report.musical_params.enable_melody);
    Ok(())
}

#[test]
fn seed_commands_set_then_renew_the_seed() -> Result<(), String> {
    let (mut host, _playback) = host()?;
    host.apply(EngineCommand::SetSeed(7))?;
    assert_eq!(host.session_seed(), 7);
    host.apply(EngineCommand::NewMelody)?;
    assert_ne!(host.session_seed(), 7);
    Ok(())
}

#[test]
fn seek_moves_the_transport_and_soft_seek_replays_composed_bars() -> Result<(), String> {
    let (mut host, mut playback) = host()?;
    host.apply(EngineCommand::SetWriteheadLookahead(8))?;
    run(&mut host, &mut playback, 8);

    host.apply(EngineCommand::Seek(6))?;
    run(&mut host, &mut playback, 1);
    assert_eq!(host.transport_position().bar, 6);

    // Bars 1..=6 stay composed: the soft seek only moves the playhead.
    host.apply(EngineCommand::SeekPlayhead(3))?;
    let mut buffer = [0.0f32; 64];
    playback.process_buffer(&mut buffer, 2, Instant::now());
    assert_eq!(host.transport_position().bar, 3);
    assert!(host.poll_state().is_none(), "reports from before the jump are dropped");
    Ok(())
}

#[test]
fn loop_beyond_the_playback_window_is_refused() -> Result<(), String> {
    let (mut host, _playback) = host()?;
    assert!(host.apply(EngineCommand::SetLoop { start_bar: 1, end_bar: 40 }).is_err());
    host.apply(EngineCommand::SetLoop { start_bar: 2, end_bar: 4 })?;
    host.apply(EngineCommand::ClearLoop)
}

#[test]
fn audible_tempo_is_taken_then_cleared_at_pulse_boundaries() -> Result<(), String> {
    let (mut host, mut playback) = host()?;
    let mut clock = host.take_clock_reader().ok_or("horloge absente")?;
    host.apply(EngineCommand::SetWriteheadLookahead(8))?;
    let mut buffer = [0.0f32; 512];
    let mut period = |host: &mut LiveHost, playback: &mut PlaybackEngine| {
        for _ in 0..400 {
            host.pump();
            playback.process_buffer(&mut buffer, 2, Instant::now());
        }
        let now = clock.read();
        now.next_pulse_frame - now.pulse_start_frame
    };

    let measure_period = period(&mut host, &mut playback);
    assert!(measure_period > 0);

    host.apply(EngineCommand::SetAudibleTempo { bpm: 300.0, ramp_pulses: 0 })?;
    let fast = period(&mut host, &mut playback);
    assert!(fast < measure_period, "{fast} < {measure_period}");

    host.apply(EngineCommand::ClearAudibleTempo)?;
    assert_eq!(period(&mut host, &mut playback), measure_period);
    Ok(())
}

#[test]
fn recording_commands_publish_the_file() -> Result<(), String> {
    let (mut host, mut playback) = host()?;
    host.apply(EngineCommand::StartRecording(RecordFormat::Midi))?;
    run(&mut host, &mut playback, 40);
    host.apply(EngineCommand::StopRecording(RecordFormat::Midi))?;
    run(&mut host, &mut playback, 2);
    let recordings = host.take_recordings();
    assert!(
        recordings.iter().any(|(format, bytes)| *format == RecordFormat::Midi && !bytes.is_empty())
    );
    assert!(host.take_recordings().is_empty(), "taken once");
    Ok(())
}

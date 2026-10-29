//! L'hôte vivant exécute les `EngineCommand` : chaque commande a son effet
//! observable sur la sortie publiée (paramètres, transport, horloge, fichiers).
//!
//! Hôte sans carte son (`LiveHost::detached`) : le test fait tourner le
//! `PlaybackEngine` lui-même, tampon par tampon.
use std::{collections::BTreeMap, time::Instant};

use harmonium::{LiveHost, playback::PlaybackEngine};
use harmonium_core::{EngineCommand, StyleProfile, events::RecordFormat, report::MeasureSnapshot};

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

/// Joue `buffers` tampons et garde, par mesure, le dernier snapshot publié.
fn play_bars(
    host: &mut LiveHost,
    playback: &mut PlaybackEngine,
    buffers: usize,
) -> BTreeMap<usize, MeasureSnapshot> {
    let mut bars = BTreeMap::new();
    let mut buffer = [0.0f32; 1024];
    for _ in 0..buffers {
        for snapshot in host.pump() {
            bars.insert(snapshot.index, snapshot);
        }
        playback.process_buffer(&mut buffer, 2, Instant::now());
    }
    bars
}

/// `reset` rend les défauts du profil actif et recommence à la mesure 1 avec la même graine :
/// la musique repart telle qu'au premier passage.
#[test]
fn reset_restarts_the_active_profile_from_bar_one() -> Result<(), String> {
    let (mut host, mut playback) = host()?;
    let profile = StyleProfile::named("nt_desert").map_err(|e| e.to_string())?;
    host.apply(EngineCommand::SetStyleProfile(Box::new(profile)))?;
    host.apply(EngineCommand::SetSeed(5))?;
    let first = play_bars(&mut host, &mut playback, 600);
    assert!(host.transport_position().bar > 3, "la lecture a avancé");
    let defaults = host.musical_params().clone();

    host.apply(EngineCommand::SetEmotionParams {
        arousal: 0.95,
        valence: -0.8,
        density: 0.9,
        tension: 0.9,
    })?;
    host.apply(EngineCommand::SetSwing(0.7))?;
    host.apply(EngineCommand::SetBpm(90.0))?;
    host.apply(EngineCommand::Reset)?;

    let params = host.musical_params();
    assert_eq!(&*params.profile_id, "nt_desert", "le profil actif reste");
    assert_eq!(params.bpm, defaults.bpm);
    assert_eq!(params.swing_ratio, defaults.swing_ratio);
    assert_eq!(params.harmony_valence, defaults.harmony_valence);
    assert_eq!(host.session_seed(), 5, "la graine reste");

    let again = play_bars(&mut host, &mut playback, 1);
    assert_eq!(host.transport_position().bar, 1, "la tête de lecture revient à la mesure 1");
    let again = again
        .into_iter()
        .chain(play_bars(&mut host, &mut playback, 100))
        .collect::<BTreeMap<_, _>>();
    let notes = |bars: &BTreeMap<usize, MeasureSnapshot>, bar: usize| {
        bars.get(&bar).map(|measure| format!("{:?}", measure.notes))
    };
    for bar in 1..=3 {
        assert!(notes(&again, bar).is_some(), "mesure {bar} recomposée");
        assert_eq!(notes(&again, bar), notes(&first, bar), "mesure {bar}");
    }
    Ok(())
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

/// Joue 400 tampons (~4 s) sur la banque de test, graine 3, au volume `volume` ;
/// rend le signal entendu et l'enregistrement WAV pris dans la même lecture.
fn listen(volume: Option<f32>) -> Result<(Vec<f32>, Vec<u8>), String> {
    let font = include_bytes!("../../assets/test.sf2");
    let (mut host, mut playback) = LiveHost::detached(Some(font), None, &[], SAMPLE_RATE)?;
    host.apply(EngineCommand::SetSeed(3))?;
    if let Some(volume) = volume {
        host.apply(EngineCommand::SetMasterVolume(volume))?;
    }
    host.apply(EngineCommand::StartRecording(RecordFormat::Wav))?;
    let mut heard = Vec::new();
    let mut buffer = [0.0f32; 1024];
    for _ in 0..400 {
        host.pump();
        playback.process_buffer(&mut buffer, 2, Instant::now());
        heard.extend_from_slice(&buffer);
    }
    host.apply(EngineCommand::StopRecording(RecordFormat::Wav))?;
    host.pump();
    playback.process_buffer(&mut buffer, 2, Instant::now());
    let wav = host
        .take_recordings()
        .into_iter()
        .find(|(format, _)| *format == RecordFormat::Wav)
        .map(|(_, bytes)| bytes)
        .ok_or("WAV absent")?;
    Ok((heard, wav))
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0, |peak, s| peak.max(s.abs()))
}

#[test]
fn listening_volume_scales_what_is_heard_but_not_what_is_recorded() -> Result<(), String> {
    let (full, full_wav) = listen(None)?;
    let (half, half_wav) = listen(Some(0.5))?;
    let (mute, mute_wav) = listen(Some(0.0))?;

    let full_peak = peak(&full);
    assert!(full_peak > 0.01, "la banque de test doit sonner : {full_peak}");
    let ratio = peak(&half) / full_peak;
    assert!((ratio - 0.5).abs() < 0.01, "crête à 0,5 / crête à 1,0 = {ratio}");

    // Past the ramp (10 ms), volume 0 is exact silence.
    assert!(mute[2000..].iter().all(|s| *s == 0.0));

    // Recordings are taken before the listening volume.
    assert_eq!(half_wav, full_wav);
    assert_eq!(mute_wav, full_wav);
    Ok(())
}

#[test]
fn listening_volume_is_bounded_and_ignores_non_finite_values() -> Result<(), String> {
    let (loud, _) = listen(Some(3.0))?;
    let (full, _) = listen(None)?;
    assert_eq!(peak(&loud), peak(&full), "borné à 1,0");
    let (nan, _) = listen(Some(f32::NAN))?;
    assert_eq!(peak(&nan), peak(&full), "NaN ignoré");
    Ok(())
}

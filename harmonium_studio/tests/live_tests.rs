//! L'écran Live sans fenêtre : une session sur un hôte détaché (aucune carte son), que le test
//! fait jouer tampon par tampon. Chaque test regarde ce que le moteur publie (snapshots,
//! transport, horloge, signal rendu, fichiers), pas le câblage.

use std::{net::UdpSocket, time::Duration};

use harmonium::{LiveHost, playback::PlaybackEngine};
use harmonium_core::{
    EngineCommand, MeasureSnapshot, StyleProfile,
    events::RecordFormat,
    report::NoteSnapshot,
    timeline::{Articulation, TrackId},
    trad::pitch_class_name,
};
use harmonium_studio::{
    live::{Intent, LiveView, Score, Session, TRACKS, score_view},
    platform::{RecordingSink, SessionHandle, start_osc},
};
use web_time::Instant;

const SAMPLE_RATE: f64 = 44100.0;
const SEED: u64 = 3;
const PROFILE: &str = "reel_quebecois";
const TEST_BANK: &[u8] = include_bytes!("../../assets/test.sf2");

fn command(session: &mut Session, command: EngineCommand) -> Result<(), String> {
    session.perform(Intent::Command(command))
}

/// Les commandes de départ du studio (`open_session`), plus une graine fixe.
fn setup() -> Result<Vec<EngineCommand>, String> {
    let profile = StyleProfile::named(PROFILE).map_err(|e| e.to_string())?;
    Ok(vec![
        EngineCommand::SetWriteheadLookahead(16),
        EngineCommand::SetStyleProfile(Box::new(profile)),
        EngineCommand::SetSeed(SEED),
    ])
}

fn session(bank: Option<&[u8]>) -> Result<(Session, PlaybackEngine), String> {
    let (host, playback) = LiveHost::detached(bank, None, &[], SAMPLE_RATE)?;
    let mut session = Session::new(host);
    for step in setup()? {
        command(&mut session, step)?;
    }
    Ok((session, playback))
}

/// Une session sur un autre profil distribué.
fn session_of(profile: &str) -> Result<(Session, PlaybackEngine), String> {
    let (host, playback) = LiveHost::detached(None, None, &[], SAMPLE_RATE)?;
    let mut session = Session::new(host);
    let profile = StyleProfile::named(profile).map_err(|e| e.to_string())?;
    command(&mut session, EngineCommand::SetWriteheadLookahead(16))?;
    command(&mut session, EngineCommand::SetStyleProfile(Box::new(profile)))?;
    command(&mut session, EngineCommand::SetSeed(SEED))?;
    Ok((session, playback))
}

/// Compose et joue `buffers` tampons de 512 images ; rend le signal entendu.
fn play(session: &mut Session, playback: &mut PlaybackEngine, buffers: usize) -> Vec<f32> {
    let mut heard = Vec::with_capacity(buffers * 1024);
    let mut buffer = [0.0f32; 1024];
    for _ in 0..buffers {
        session.pump();
        playback.process_buffer(&mut buffer, 2, Instant::now());
        heard.extend_from_slice(&buffer);
    }
    heard
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0, |peak, sample| peak.max(sample.abs()))
}

fn view(session: &mut Session) -> LiveView {
    session.view(Instant::now())
}

#[test]
fn the_score_view_shows_the_published_measures_and_the_playhead() -> Result<(), String> {
    let (mut session, mut playback) = session(None)?;
    play(&mut session, &mut playback, 400);

    // Référence : le même hôte, piloté à la main, dont on garde chaque snapshot publié.
    let (mut host, mut reference) = LiveHost::detached(None, None, &[], SAMPLE_RATE)?;
    for step in setup()? {
        host.apply(step)?;
    }
    let mut published: std::collections::BTreeMap<usize, MeasureSnapshot> = Default::default();
    let mut buffer = [0.0f32; 1024];
    for _ in 0..400 {
        for snapshot in host.pump() {
            published.insert(snapshot.index, snapshot);
        }
        reference.process_buffer(&mut buffer, 2, Instant::now());
    }

    let view = view(&mut session);
    let score = &view.score;
    let transport = session.host().transport_position().bar;
    assert!(transport > 1, "la lecture avance : mesure {transport}");
    assert_eq!(score.current_bar, transport);
    assert!(score.playhead >= (transport - score.first_bar) as f32);
    let composed = score.composed_through.ok_or("rien de composé")?;
    assert!(composed >= transport + 8, "mesures d'avance : {composed} pour {transport}");

    let mut shown = 0;
    for bar in &score.bars {
        let Some(content) = &bar.content else { continue };
        let snapshot = published.get(&bar.index).ok_or(format!("mesure {} absente", bar.index))?;
        let notes: Vec<_> = snapshot
            .notes
            .iter()
            .map(|n| (n.track, n.pitch, n.start_step, n.duration_steps, n.velocity))
            .collect();
        let drawn: Vec<_> = content
            .notes
            .iter()
            .map(|n| (n.track.channel(), n.pitch, n.start_step, n.duration_steps, n.velocity))
            .collect();
        assert_eq!(drawn, notes, "notes de la mesure {}", bar.index);
        let articulations: Vec<_> = snapshot.notes.iter().map(|n| n.articulation).collect();
        let drawn_articulations: Vec<_> = content.notes.iter().map(|n| n.articulation).collect();
        assert_eq!(drawn_articulations, articulations, "articulations de la mesure {}", bar.index);
        assert!(!content.chords.is_empty(), "accords de la mesure {}", bar.index);
        let position = snapshot.position.ok_or("position publiée")?;
        let form = content.form.as_ref().ok_or("forme")?;
        assert_eq!(form.tune_in_set, position.tune_in_set);
        assert_eq!(form.bar_in_strain, position.bar_in_strain);
        shown += 1;
    }
    assert!(shown >= 10, "{shown} mesures montrées");
    assert!(
        score.bars.iter().any(|bar| bar.content.as_ref().is_some_and(|c| !c.notes.is_empty())),
        "des notes à dessiner"
    );

    // Horloge : celle du transport, et un tempo entendu.
    let beat = view.beat.as_ref().ok_or("aucune pulsation")?;
    assert!(beat.bar.abs_diff(transport) <= 1, "pulsation {} / transport {transport}", beat.bar);
    assert!(beat.pulse >= 1 && beat.pulse <= beat.pulses_per_bar);
    assert!(view.audible_bpm.is_some_and(|bpm| bpm > 30.0));

    // Panneau d'état : tonalité et mode de la position publiée.
    let style = view.state.iter().find(|group| group.title == "Style").ok_or("groupe Style")?;
    assert!(style.rows.iter().any(|(label, _)| *label == "Tune key/mode"));
    Ok(())
}

#[test]
fn an_emotion_change_recomposes_the_bars_ahead() -> Result<(), String> {
    let (mut session, mut playback) = session(None)?;
    command(
        &mut session,
        EngineCommand::SetEmotionParams { arousal: 0.1, valence: -0.6, density: 0.2, tension: 0.1 },
    )?;
    play(&mut session, &mut playback, 200);
    let current = session.host().transport_position().bar;
    let ahead = current + 3;
    let notes = |session: &Session, bar: usize| {
        session.score().bar(bar).map(|measure| {
            measure
                .notes
                .iter()
                .map(|n| (n.track, n.pitch, n.start_step, n.duration_steps, n.velocity))
                .collect::<Vec<_>>()
        })
    };
    let (playing, before) = (notes(&session, current), notes(&session, ahead));
    assert!(before.is_some(), "mesure {ahead} composée d'avance");

    command(
        &mut session,
        EngineCommand::SetEmotionParams { arousal: 0.95, valence: 0.8, density: 0.9, tension: 0.9 },
    )?;
    session.pump();
    assert_eq!(view(&mut session).emotions, [0.95, 0.8, 0.9, 0.9]);
    assert_eq!(notes(&session, current), playing, "la mesure en cours est gardée");
    let after = notes(&session, ahead);
    assert!(after.is_some() && after != before, "la mesure {ahead} est recomposée");
    Ok(())
}

#[test]
fn solo_silences_the_other_tracks_and_ending_it_restores_them() -> Result<(), String> {
    let (mut session, mut playback) = session(Some(TEST_BANK))?;
    command(&mut session, EngineCommand::SetFeetDensity(0))?;
    assert!(peak(&play(&mut session, &mut playback, 300)) > 0.01, "la banque de test sonne");

    // Sans pieds, la pointe ne joue pas : en solo, elle laisse le silence.
    assert!(
        view(&mut session)
            .score
            .bars
            .iter()
            .filter_map(|bar| bar.content.as_ref())
            .all(|content| content.notes.iter().all(|note| note.track != TrackId::Toe))
    );
    session.perform(Intent::Solo(Some(TrackId::Toe)))?;
    let solo = play(&mut session, &mut playback, 300);
    // Reste la queue de la réverbération, sous −80 dB.
    let tail = peak(&solo[solo.len() / 2..]);
    assert!(tail < 1e-4, "solo sur une piste muette : silence ({tail})");
    let view = view(&mut session);
    for strip in &view.mixer {
        assert_eq!(strip.audible, strip.track == TrackId::Toe, "{:?}", strip.track);
    }

    // Une sourdine à la main survit à la fin du solo.
    session.perform(Intent::Mute(TrackId::Melody, true))?;
    session.perform(Intent::Solo(None))?;
    let back = play(&mut session, &mut playback, 300);
    assert!(peak(&back) > 0.01, "fin du solo : le son revient");
    let view = self::view(&mut session);
    for strip in &view.mixer {
        assert_eq!(strip.audible, strip.track != TrackId::Melody, "{:?}", strip.track);
    }
    Ok(())
}

#[test]
fn a_loop_holds_the_playhead_and_a_long_loop_is_refused() -> Result<(), String> {
    let (mut session, mut playback) = session(None)?;
    // Glissé de droite à gauche : bornes remises dans l'ordre.
    session.perform(Intent::Loop(4, 2))?;
    assert_eq!(view(&mut session).score.loop_range, Some((2, 4)));
    play(&mut session, &mut playback, 600);
    for _ in 0..40 {
        play(&mut session, &mut playback, 20);
        let bar = session.host().transport_position().bar;
        assert!((2..=4).contains(&bar), "mesure {bar} hors de la boucle");
    }

    let refused = session.perform(Intent::Loop(1, 40)).err().ok_or("boucle de 40 acceptée")?;
    assert!(refused.contains("Loop too long"), "{refused}");
    assert_eq!(view(&mut session).score.loop_range, Some((2, 4)), "boucle inchangée");

    command(&mut session, EngineCommand::ClearLoop)?;
    assert_eq!(view(&mut session).score.loop_range, None);
    Ok(())
}

#[test]
fn seek_moves_the_playhead_hard_and_soft() -> Result<(), String> {
    let (mut session, mut playback) = session(None)?;
    play(&mut session, &mut playback, 100);
    command(&mut session, EngineCommand::Seek(9))?;
    play(&mut session, &mut playback, 2);
    assert_eq!(view(&mut session).score.current_bar, 9);
    command(&mut session, EngineCommand::SeekPlayhead(3))?;
    play(&mut session, &mut playback, 1);
    assert_eq!(view(&mut session).score.current_bar, 3);
    Ok(())
}

#[test]
fn the_view_follows_the_commands_of_the_parity_table() -> Result<(), String> {
    let (mut session, mut playback) = session(None)?;
    for step in [
        EngineCommand::SetSwing(0.62),
        EngineCommand::SetFeetDensity(2),
        EngineCommand::SetIntensity(0.8),
        EngineCommand::SetBpm(150.0),
        EngineCommand::SetTimeSignature { numerator: 3, denominator: 4 },
        EngineCommand::EnableHarmony(false),
        EngineCommand::UseDirectMode,
        EngineCommand::SetMasterVolume(0.4),
        EngineCommand::SetOutputMute(true),
        EngineCommand::SetChannelRoute { channel: 2, bank_id: 1 },
        EngineCommand::SetChannelGain { channel: 0, gain: 0.25 },
        EngineCommand::SetWriteheadLookahead(8),
        EngineCommand::SetAudibleTempo { bpm: 180.0, ramp_pulses: 4 },
    ] {
        command(&mut session, step)?;
    }
    play(&mut session, &mut playback, 50);
    let view = view(&mut session);
    assert!((view.swing - 0.62).abs() < 1e-6);
    assert_eq!(view.feet, 2);
    assert!((view.intensity - 0.8).abs() < 1e-6);
    assert!((view.nominal_bpm - 150.0).abs() < 1e-6);
    assert_eq!(view.time_signature, (3, 4));
    assert_eq!(view.modules, [true, false, true]);
    assert!(!view.emotion_mode);
    assert!((view.desk.master_volume - 0.4).abs() < 1e-6);
    assert!(view.desk.output_muted);
    assert_eq!(view.desk.lookahead, Some(8));
    assert_eq!(view.desk.audible_tempo, Some((180.0, 4)));
    let melody = view.mixer.iter().find(|s| s.track == TrackId::Melody).ok_or("Melody")?;
    assert!((melody.gain - 0.25).abs() < 1e-6);
    let bass = view.mixer.iter().find(|s| s.track == TrackId::Bass).ok_or("Bass")?;
    assert_eq!(bass.route, 1);
    let harmony = view.state.iter().find(|g| g.title == "Modules").ok_or("Modules")?;
    assert!(harmony.rows.contains(&("Harmony", "off".to_string())));

    command(&mut session, EngineCommand::UseEmotionMode)?;
    command(&mut session, EngineCommand::ClearAudibleTempo)?;
    command(&mut session, EngineCommand::ResetBpm)?;
    let view = self::view(&mut session);
    assert!(view.emotion_mode);
    assert_eq!(view.desk.audible_tempo, None);
    assert!((view.nominal_bpm - 150.0).abs() > 1e-3, "tempo nominal rendu aux émotions");

    // Un autre profil rend à chaque piste son niveau : le gain choisi ne vaut plus.
    let other = StyleProfile::named("valse_quebecoise").map_err(|e| e.to_string())?;
    command(&mut session, EngineCommand::SetStyleProfile(Box::new(other)))?;
    let view = self::view(&mut session);
    assert_eq!(view.profile_id, "valse_quebecoise");
    let melody = view.mixer.iter().find(|s| s.track == TrackId::Melody).ok_or("Melody")?;
    assert!((melody.gain - 0.25).abs() > 1e-3, "niveau du profil : {}", melody.gain);
    assert_eq!(view.mixer.len(), TRACKS.len());
    Ok(())
}

#[test]
fn seed_commands_restart_the_score() -> Result<(), String> {
    let (mut session, mut playback) = session(None)?;
    play(&mut session, &mut playback, 100);
    command(&mut session, EngineCommand::SetSeed(11))?;
    play(&mut session, &mut playback, 2);
    let view = view(&mut session);
    assert_eq!(view.seed, 11);
    assert_eq!(view.score.current_bar, 1);
    command(&mut session, EngineCommand::NewMelody)?;
    assert_ne!(self::view(&mut session).seed, 11);
    Ok(())
}

#[test]
fn recordings_are_named_and_saved_without_overwriting() -> Result<(), String> {
    let (mut session, mut playback) = session(None)?;
    command(&mut session, EngineCommand::StartRecording(RecordFormat::Midi))?;
    assert!(view(&mut session).desk.recording[1]);
    play(&mut session, &mut playback, 200);
    command(&mut session, EngineCommand::StopRecording(RecordFormat::Midi))?;
    play(&mut session, &mut playback, 2);
    assert!(!view(&mut session).desk.recording[1]);

    let recordings = session.take_recordings();
    let [recording] = recordings.as_slice() else {
        return Err(format!("{} enregistrements", recordings.len()));
    };
    assert_eq!(recording.file_name, format!("harmonium-{PROFILE}-seed{SEED}.mid"));
    assert!(recording.bytes.starts_with(b"MThd"));

    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let sink = RecordingSink::new(dir.path().join("out"));
    let first = sink.save(recording)?;
    let second = sink.save(recording)?;
    let expected = dir.path().join("out").join(&recording.file_name);
    assert!(first.ends_with(&expected.display().to_string()), "{first}");
    assert!(second.ends_with("-2.mid"), "{second}");
    assert_eq!(std::fs::read(&expected).map_err(|e| e.to_string())?, recording.bytes);
    Ok(())
}

/// Message OSC `/harmonium/params` à quatre flottants, encodé à la main.
fn osc_params(values: [f32; 4]) -> Vec<u8> {
    fn padded(text: &str) -> Vec<u8> {
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0);
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        bytes
    }
    let mut packet = padded("/harmonium/params");
    packet.extend(padded(",ffff"));
    for value in values {
        packet.extend(value.to_be_bytes());
    }
    packet
}

#[test]
fn the_osc_listener_drives_the_emotions_of_the_session() -> Result<(), String> {
    let (session, _playback) = session(None)?;
    let handle = SessionHandle::new(session);
    let osc = start_osc(&handle, "127.0.0.1:0")?;
    let client = UdpSocket::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    client.send_to(&osc_params([0.9, -0.2, 0.4, 0.6]), osc.address()).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if handle.with(|session| session.host().emotions()) == [0.9, -0.2, 0.4, 0.6] {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err(format!("émotions : {:?}", handle.with(|session| session.host().emotions())))
}

#[test]
fn the_screen_shows_what_plays_not_the_session_leftovers() -> Result<(), String> {
    let (mut session, mut playback) = session(None)?;
    play(&mut session, &mut playback, 300);
    let view = view(&mut session);
    let measure = session.score().bar(view.score.current_bar).ok_or("mesure qui sonne")?;

    // Tempo : celui de la pulsation entendue, égal à celui de la mesure qui sonne (230 pour le
    // reel québécois), pas le `BPM: 120` de la session par défaut.
    let audible = view.audible_bpm.ok_or("tempo entendu")?;
    assert!(
        (audible - measure.tempo).abs() < 1.0,
        "{audible} BPM pour des mesures à {}",
        measure.tempo
    );
    assert!((audible - 120.0).abs() > 10.0, "{audible}");

    // Tonalité et mode : ceux de `TunePosition`, pas `session_key` / `session_scale`.
    let position = measure.position.ok_or("position publiée")?;
    let expected = format!("{} {:?}", pitch_class_name(position.key), position.mode);
    let style = view.state.iter().find(|group| group.title == "Style").ok_or("groupe Style")?;
    assert!(style.rows.contains(&("Tune key/mode", expected.clone())), "{:?}", style.rows);
    assert_eq!(
        view.score.bars.iter().find(|bar| bar.index == view.score.current_bar).and_then(|bar| bar
            .content
            .as_ref()
            .and_then(|content| content.form.as_ref())
            .map(|form| form.key_mode.clone())),
        Some(expected)
    );
    let shown: Vec<&str> = view
        .state
        .iter()
        .flat_map(|group| group.rows.iter())
        .flat_map(|(label, value)| [*label, value.as_str()])
        .collect();
    assert!(
        shown.iter().all(|text| !text.contains("Pentatonic") && !text.contains("Session")),
        "{shown:?}"
    );
    Ok(())
}

#[test]
fn every_articulation_reaches_the_piano_roll() -> Result<(), String> {
    // Une mesure publiée portant une note de chaque articulation (`Articulation` entier).
    let (mut session, mut playback) = session(None)?;
    play(&mut session, &mut playback, 100);
    let mut measure = session.score().bar(1).ok_or("mesure 1")?.clone();
    let variants = [
        Articulation::Normal,
        Articulation::Detached,
        Articulation::Linked,
        Articulation::Accent,
        Articulation::Tied,
        Articulation::Legato,
        Articulation::Trigger,
    ];
    measure.notes = variants
        .iter()
        .enumerate()
        .map(|(i, articulation)| NoteSnapshot {
            track: TrackId::Bass.channel(),
            pitch: 40 + i as u8,
            start_step: i,
            duration_steps: 1,
            velocity: 90,
            performance: Default::default(),
            articulation: *articulation,
        })
        .collect();
    let mut score = Score::default();
    score.absorb(vec![measure]);
    let position = session.host().transport_position();
    let shown = score_view(&score, position, &[], None, score.last());
    let content = shown
        .bars
        .iter()
        .find(|bar| bar.index == 1)
        .and_then(|bar| bar.content.as_ref())
        .ok_or("mesure 1 montrée")?;
    let drawn: Vec<_> = content.notes.iter().map(|note| note.articulation).collect();
    assert_eq!(drawn, variants);
    Ok(())
}

#[test]
fn legato_bass_and_tied_chords_of_isaac_are_in_the_view() -> Result<(), String> {
    let (mut session, mut playback) = session_of("isaac_basement")?;
    play(&mut session, &mut playback, 300);
    let view = view(&mut session);
    let notes = |track: TrackId, articulation: Articulation| {
        view.score
            .bars
            .iter()
            .filter_map(|bar| bar.content.as_ref())
            .flat_map(|content| content.notes.iter())
            .filter(|note| note.track == track && note.articulation == articulation)
            .count()
    };
    assert!(notes(TrackId::Bass, Articulation::Legato) > 0, "basse legato dans la fenêtre");
    assert!(notes(TrackId::Chords, Articulation::Tied) > 0, "accords liés dans la fenêtre");
    Ok(())
}

/// Le mixeur dit quelles pistes la densité courante écarte (`density_tracks` du profil) : ce sont
/// exactement celles qui n'ont plus de note dans les mesures recomposées.
#[test]
fn the_mixer_names_the_tracks_the_density_leaves_out() -> Result<(), String> {
    let (mut session, mut playback) = session_of("isaac_basement")?;
    let gate = |view: &LiveView, track: TrackId| {
        view.mixer.iter().find(|strip| strip.track == track).and_then(|strip| strip.density_gate)
    };
    let [arousal, valence, _, tension] = view(&mut session).emotions;
    command(
        &mut session,
        EngineCommand::SetEmotionParams { arousal, valence, density: 0.2, tension },
    )?;
    play(&mut session, &mut playback, 300);
    let thin = view(&mut session);
    let heard_ahead = |view: &LiveView, track: TrackId| {
        view.score
            .bars
            .iter()
            .filter(|bar| bar.index > view.score.current_bar)
            .filter_map(|bar| bar.content.as_ref())
            .any(|content| content.notes.iter().any(|note| note.track == track))
    };
    for track in [TrackId::Melody, TrackId::Bass, TrackId::Heel, TrackId::Toe] {
        assert!(gate(&thin, track).is_some_and(|minimum| minimum > 0.2), "{track:?} écartée");
        assert!(!heard_ahead(&thin, track), "{track:?} sans note à venir");
    }
    assert_eq!(gate(&thin, TrackId::Chords), None, "la nappe reste");

    command(
        &mut session,
        EngineCommand::SetEmotionParams { arousal, valence, density: 0.45, tension },
    )?;
    play(&mut session, &mut playback, 300);
    let full = view(&mut session);
    assert!(full.mixer.iter().all(|strip| strip.density_gate.is_none()), "{:?}", full.mixer);
    Ok(())
}

#[test]
fn reset_returns_to_the_active_profile_defaults_at_bar_one() -> Result<(), String> {
    let (mut session, mut playback) = session_of("isaac_basement")?;
    play(&mut session, &mut playback, 600);
    let initial = view(&mut session);
    assert!(initial.beat.as_ref().is_some_and(|beat| beat.bar > 2), "la lecture a avancé");
    for step in [
        EngineCommand::SetSwing(0.7),
        EngineCommand::SetFeetDensity(2),
        EngineCommand::SetBpm(150.0),
        EngineCommand::SetIntensity(0.9),
        EngineCommand::SetLoop { start_bar: 2, end_bar: 4 },
        EngineCommand::SetAudibleTempo { bpm: 90.0, ramp_pulses: 0 },
    ] {
        command(&mut session, step)?;
    }
    let changed = view(&mut session);
    assert!((changed.swing - initial.swing).abs() > 0.05, "swing commandé");
    command(&mut session, EngineCommand::Reset)?;
    play(&mut session, &mut playback, 5);
    let view = view(&mut session);
    assert_eq!(view.profile_id, "isaac_basement", "le profil choisi reste");
    assert_eq!(view.beat.as_ref().map(|beat| beat.bar), Some(1), "retour à la mesure 1");
    assert_eq!(view.desk.loop_range, None);
    assert_eq!(view.desk.audible_tempo, None);
    assert!((view.swing - initial.swing).abs() < 1e-6, "swing {}", view.swing);
    assert!((view.nominal_bpm - initial.nominal_bpm).abs() < 1e-3, "tempo {}", view.nominal_bpm);
    assert!((view.intensity - initial.intensity).abs() < 1e-6, "intensité {}", view.intensity);
    assert_eq!(view.feet, initial.feet);
    assert!(view.score.bars.iter().any(|bar| bar.content.is_some()), "des mesures recomposées");
    Ok(())
}

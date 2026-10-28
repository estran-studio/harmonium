//! Contrats du synthé Odin par piste (option de compilation `odin`).
//!
//! Lancer : `cargo test -p harmonium --features odin --test odin_contract_tests`.
//!
//! Les presets de test sont construits ici, octet par octet, au format des
//! fichiers `.odin` (en-tête `Odin` puis entrées `PARAM`) : aucun fichier
//! `.odin` ni code GPL dans ce dépôt (licence MIT OR Apache-2.0). Aucune
//! SoundFont n'est chargée : tout son audible vient donc d'Odin, et le repli
//! SoundFont d'une piste sans preset est le silence.
//!
//! Le callback complet (`PlaybackEngine::process_buffer`) tourne sous la garde
//! `RTCheckAllocator` du binaire de test : une allocation, une libération ou
//! un verrou dans le chemin Odin (notes, morphing, rendu) panique ici.
#![cfg(feature = "odin")]

use std::time::Instant;

use harmonium::{audio::create_offline_engine, playback::PlaybackCommand};
use harmonium_audio::odin_source::OdinPresetSource;
use harmonium_core::{
    StyleProfile,
    events::{AudioRecordFormat, RecordFormat},
    report::MeasureSnapshot,
    timeline::TrackId,
};
use midly::{MidiMessage, Smf, TrackEventKind};

const SAMPLE_RATE: f64 = 44100.0;
const BUFFER_FRAMES: usize = 256;
const BASE_EMOTIONS: [f32; 4] = [0.5, 0.3, 0.2, 0.4];

/// Entrée `PARAM` du format `.odin` : nom du paramètre, valeur `f64`.
fn push_param(bytes: &mut Vec<u8>, name: &str, value: f64) {
    bytes.extend_from_slice(b"PARAM\0\x01\x02id\0\x01\x00\x05");
    bytes.extend_from_slice(name.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(b"value\0\x01\x00\x04");
    bytes.extend_from_slice(&value.to_le_bytes());
}

/// Preset de nappe : un oscillateur audible, filtre passe-bas ouvert à
/// `filter_hz`, sustain plein et relâchement court.
fn test_preset(filter_hz: f64) -> Vec<u8> {
    let mut bytes = b"Odin".to_vec();
    for (name, value) in [
        ("osc1_vol", 0.0),
        ("osc2_vol", -100.0),
        ("osc3_vol", -100.0),
        ("master", 0.0),
        ("fil1_freq", filter_hz),
        ("fil1_res", 0.2),
        ("fil1_env", 0.0),
        ("env1_attack", 0.01),
        ("env1_decay", 0.1),
        ("env1_sustain", 1.0),
        ("env1_release", 0.1),
    ] {
        push_param(&mut bytes, name, value);
    }
    bytes
}

fn load(id: &str) -> StyleProfile {
    StyleProfile::load(format!("{}/../tests/fixtures/profiles/{id}", env!("CARGO_MANIFEST_DIR")))
        .unwrap()
}

struct Take {
    audio: Vec<f32>,
    snapshots: Vec<MeasureSnapshot>,
    midi: Vec<u8>,
}

impl Take {
    fn peak(&self) -> f32 {
        self.audio.iter().fold(0.0, |peak, s| peak.max(s.abs()))
    }

    /// Événements MIDI `(note, attaque ?)` du canal de la piste `Chords`.
    fn chord_events(&self) -> Vec<(u8, bool)> {
        let smf = Smf::parse(&self.midi).unwrap();
        smf.tracks[0]
            .iter()
            .filter_map(|event| match event.kind {
                TrackEventKind::Midi { channel, message }
                    if channel.as_int() == TrackId::Chords.channel() =>
                {
                    match message {
                        MidiMessage::NoteOn { key, vel } => Some((key.as_int(), vel.as_int() > 0)),
                        MidiMessage::NoteOff { key, .. } => Some((key.as_int(), false)),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect()
    }
}

/// Rend `bars` mesures de la seule piste `Chords` d'un profil, graine 1.
/// `audio_emotions` pilote le morphing sans toucher aux notes générées ;
/// `silence_after` coupe la piste (muet) après ce nombre de mesures.
fn render(
    profile: &str,
    presets: &[(&str, Vec<u8>)],
    audio_emotions: [f32; 4],
    bars: usize,
    silence_after: Option<usize>,
) -> Take {
    let sources: Vec<OdinPresetSource> =
        presets.iter().map(|(name, bytes)| OdinPresetSource { name, bytes }).collect();
    let (mut composer, mut playback, mut cmd_tx, _reports, _clock, finished) =
        create_offline_engine(None, None, &sources, SAMPLE_RATE).unwrap();
    composer.set_seed(1);
    composer.set_profile(load(profile)).unwrap();
    composer.invalidate_future();
    if let Some((bar, _)) = composer.take_pending_drop() {
        cmd_tx.push(PlaybackCommand::DropFrom { bar }).unwrap();
    }
    cmd_tx
        .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()))
        .unwrap();
    cmd_tx.push(PlaybackCommand::SetEmotions(audio_emotions)).unwrap();
    for track in TrackId::ALL {
        cmd_tx
            .push(PlaybackCommand::SetChannelMute {
                channel: track.channel(),
                muted: track != TrackId::Chords,
            })
            .unwrap();
    }
    composer.set_writehead_lookahead(bars + 6);
    composer.generate_bars(bars + 2);
    let snapshots = composer.take_snapshots();
    cmd_tx.push(PlaybackCommand::StartRecording(AudioRecordFormat::Midi)).unwrap();

    let mut audio = Vec::new();
    let mut buffer = vec![0.0f32; BUFFER_FRAMES * 2];
    let mut muted = false;
    let mut tail_frames = 0usize;
    for _ in 0..200_000 {
        composer.generate_ahead();
        if !muted && silence_after.is_some_and(|after| composer.playhead_bar() > after) {
            let mut mask = [false; 16];
            mask[usize::from(TrackId::Chords.channel())] = true;
            cmd_tx.push(PlaybackCommand::SetMutedChannels(mask)).unwrap();
            muted = true;
        }
        if composer.playhead_bar() > bars && !muted {
            break;
        }
        if muted {
            tail_frames += BUFFER_FRAMES;
            if tail_frames > 3 * SAMPLE_RATE as usize {
                break;
            }
        }
        buffer.fill(0.0);
        playback.process_buffer(&mut buffer, 2, Instant::now());
        audio.extend_from_slice(&buffer);
    }
    cmd_tx.push(PlaybackCommand::StopRecording(AudioRecordFormat::Midi)).unwrap();
    buffer.fill(0.0);
    playback.process_buffer(&mut buffer, 2, Instant::now());
    let deadline = Instant::now() + std::time::Duration::from_secs(20);
    let midi = loop {
        if let Some((_, bytes)) =
            finished.lock().unwrap().iter().find(|(format, _)| *format == RecordFormat::Midi)
        {
            break bytes.clone();
        }
        assert!(Instant::now() < deadline, "le MIDI enregistré est publié");
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    Take { audio, snapshots, midi }
}

fn nappe() -> Vec<(&'static str, Vec<u8>)> {
    vec![("nappe", test_preset(900.0))]
}

#[test]
fn a_track_with_a_provided_preset_plays_odin_otherwise_its_soundfont_program() {
    let with_preset = render("nappe_pad", &nappe(), BASE_EMOTIONS, 6, None);
    assert!(with_preset.peak() > 0.01, "la nappe Odin produit du son : {}", with_preset.peak());

    // Aucun preset fourni (ou nom absent) : la piste reste sur son programme
    // SoundFont ; sans banque chargée, c'est le silence, jamais de son Odin.
    let none = render("nappe_pad", &[], BASE_EMOTIONS, 6, None);
    assert_eq!(none.peak(), 0.0);
    let other = render("nappe_pad", &[("autre", test_preset(900.0))], BASE_EMOTIONS, 6, None);
    assert_eq!(other.peak(), 0.0);
}

#[test]
fn a_second_style_uses_the_same_mechanism_in_three_four() {
    // Les accords de la valse entrent à la seconde passe de la partie.
    let take = render("valse_nappe", &nappe(), BASE_EMOTIONS, 24, None);
    assert!(take.snapshots.iter().all(|s| s.time_sig_numerator == 3));
    assert!(take.peak() > 0.01, "la valse joue sa nappe Odin : {}", take.peak());
}

#[test]
fn morphing_is_deterministic_and_each_emotion_changes_the_sound() {
    let base = render("nappe_pad", &nappe(), BASE_EMOTIONS, 4, None);
    let again = render("nappe_pad", &nappe(), BASE_EMOTIONS, 4, None);
    assert_eq!(base.audio, again.audio, "mêmes entrées, même son");

    for (axis, name) in ["arousal", "valence", "density", "tension"].iter().enumerate() {
        let mut moved = BASE_EMOTIONS;
        moved[axis] = if moved[axis] < 0.6 { 1.0 } else { 0.0 };
        let take = render("nappe_pad", &nappe(), moved, 4, None);
        assert_eq!(
            serde_json::to_string(&take.snapshots).unwrap(),
            serde_json::to_string(&base.snapshots).unwrap(),
            "{name} : les notes ne changent pas"
        );
        assert_ne!(take.audio, base.audio, "{name} change le rendu Odin");
    }
}

#[test]
fn tied_chords_are_never_reattacked_and_no_pitch_is_attacked_twice() {
    let take = render("nappe_pad", &nappe(), BASE_EMOTIONS, 8, None);
    let events = take.chord_events();
    let mut sounding = std::collections::BTreeSet::new();
    let mut attacks = 0;
    for (note, on) in events {
        if on {
            attacks += 1;
            assert!(sounding.insert(note), "hauteur {note} réattaquée sans NoteOff");
        } else {
            sounding.remove(&note);
        }
    }
    let published = take
        .snapshots
        .iter()
        .filter(|snapshot| snapshot.index <= 8)
        .flat_map(|snapshot| snapshot.notes.iter())
        .filter(|note| note.track == TrackId::Chords as u8)
        .count();
    assert!(attacks > 0 && attacks < published, "{attacks} attaques pour {published} notes");
}

#[test]
fn muting_a_track_leaves_no_orphan_note_in_odin() {
    let take = render("nappe_pad", &nappe(), BASE_EMOTIONS, 8, Some(3));
    assert!(take.peak() > 0.01);
    let tail = &take.audio[take.audio.len() - SAMPLE_RATE as usize / 2 * 2..];
    let tail_peak = tail.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
    assert!(tail_peak < 1e-4, "aucune voix Odin ne reste coincée : {tail_peak}");
}

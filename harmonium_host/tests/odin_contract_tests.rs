//! Contrats du synthé Odin par piste (option de compilation `odin`).
//!
//! Lancer : `cargo test -p harmonium --features odin --test odin_contract_tests`.
//!
//! Les presets de test sont construits ici au format binaire JUCE ValueTree :
//! racine `Odin`, nœuds `PARAM`, `osc` et `misc`, propriétés typées et comptes
//! d'enfants. Aucun fichier `.odin` ni code GPL n'est embarqué. Hors du test
//! de bascule, aucune SoundFont n'est chargée : tout son vient donc d'Odin.
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
    odin::{OdinMorphParam, OdinMorphRule},
    report::MeasureSnapshot,
    timeline::TrackId,
};
use midly::{MidiMessage, Smf, TrackEventKind};

const SAMPLE_RATE: f64 = 44100.0;
const BUFFER_FRAMES: usize = 256;
const BASE_EMOTIONS: [f32; 4] = [0.5, 0.3, 0.2, 0.4];

/// Entier compressé JUCE ; les petits arbres de ce test tiennent sur un octet.
fn push_count(bytes: &mut Vec<u8>, count: usize) {
    if count == 0 {
        bytes.push(0);
    } else {
        bytes.extend_from_slice(&[1, u8::try_from(count).unwrap()]);
    }
}

fn push_number(bytes: &mut Vec<u8>, name: &str, value: f64) {
    bytes.extend_from_slice(name.as_bytes());
    bytes.push(0);
    push_count(bytes, 9);
    bytes.push(4); // Variante double : marqueur puis huit octets.
    bytes.extend_from_slice(&value.to_le_bytes());
}

/// Entrée `PARAM` avec deux propriétés et aucun enfant.
fn push_param(bytes: &mut Vec<u8>, name: &str, value: f64) {
    bytes.extend_from_slice(b"PARAM\0");
    push_count(bytes, 2);
    bytes.extend_from_slice(b"id\0");
    push_count(bytes, name.len() + 2);
    bytes.push(5); // Variante chaîne, zéro terminal compris dans sa longueur.
    bytes.extend_from_slice(name.as_bytes());
    bytes.push(0);
    push_number(bytes, "value", value);
    push_count(bytes, 0);
}

fn push_section(bytes: &mut Vec<u8>, name: &str, properties: &[(&str, f64)]) {
    bytes.extend_from_slice(name.as_bytes());
    bytes.push(0);
    push_count(bytes, properties.len());
    for &(name, value) in properties {
        push_number(bytes, name, value);
    }
    push_count(bytes, 0);
}

/// Dent de scie analogique, LP24 routé directement vers l'ampli, enveloppe
/// tenue et relâchement court. Les identifiants sont ceux du schéma upstream.
fn test_preset(filter_hz: f64) -> Vec<u8> {
    let parameters = [
        ("osc1_vol", 0.0),
        ("osc1_reset", 1.0),
        ("osc2_vol", -60.0),
        ("osc3_vol", -60.0),
        ("master", -12.0),
        ("fil1_freq", filter_hz),
        ("fil1_res", 0.2),
        ("fil1_env", 0.0),
        ("fil1_osc1", 1.0),
        ("fil1_to_amp", 1.0),
        ("fil2_to_amp", 0.0),
        ("env1_attack", 0.01),
        ("env1_decay", 0.1),
        ("env1_sustain", 1.0),
        ("env1_release", 0.1),
    ];
    let mut bytes = b"Odin\0".to_vec();
    push_count(&mut bytes, 0);
    push_count(&mut bytes, parameters.len() + 2);
    for (name, value) in parameters {
        push_param(&mut bytes, name, value);
    }
    push_section(
        &mut bytes,
        "osc",
        &[("osc1_type", 2.0), ("osc1_analog_wave", 0.0), ("osc2_type", 1.0), ("osc3_type", 1.0)],
    );
    push_section(
        &mut bytes,
        "misc",
        &[
            ("patch_migration_version", 5.0),
            ("fil1_type", 2.0),
            ("fil2_type", 1.0),
            ("fil3_type", 1.0),
        ],
    );
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

/// Profil dont la piste `Chords` garde son preset avec ces règles de morphing.
fn with_chords_morph(mut profile: StyleProfile, morph: Vec<OdinMorphRule>) -> StyleProfile {
    profile.odin.get_mut(&TrackId::Chords).expect("la fixture déclare Chords").morph = morph;
    profile
}

/// Rend `bars` mesures de la seule piste `Chords` d'un profil, graine 1.
/// `emotions` est l'unique entrée du morphing (préparé par le compositeur) ;
/// `silence_after` coupe la piste (muet) après ce nombre de mesures.
fn render(
    profile: StyleProfile,
    presets: &[(&str, Vec<u8>)],
    emotions: [f32; 4],
    bars: usize,
    silence_after: Option<usize>,
) -> Take {
    render_inner(profile, presets, emotions, bars, silence_after, None)
}

/// Comme [`render`], avec un changement d'émotions à la mesure 3 : le
/// compositeur prépare de nouveaux presets morphés, envoyés au callback.
fn render_with_emotion_change(
    profile: StyleProfile,
    presets: &[(&str, Vec<u8>)],
    bars: usize,
    after: [f32; 4],
) -> Take {
    render_inner(profile, presets, BASE_EMOTIONS, bars, None, Some((3, after)))
}

fn render_inner(
    profile: StyleProfile,
    presets: &[(&str, Vec<u8>)],
    emotions: [f32; 4],
    bars: usize,
    silence_after: Option<usize>,
    mut change: Option<(usize, [f32; 4])>,
) -> Take {
    let sources: Vec<OdinPresetSource> =
        presets.iter().map(|(name, bytes)| OdinPresetSource { name, bytes }).collect();
    let (mut composer, mut playback, mut cmd_tx, _reports, _clock, finished) =
        create_offline_engine(None, None, &sources, SAMPLE_RATE).unwrap();
    composer.set_seed(1);
    composer.set_profile(profile).unwrap();
    composer.use_emotion_mode();
    composer.set_emotions(emotions[0], emotions[1], emotions[2], emotions[3]);
    composer.invalidate_future();
    if let Some((bar, _)) = composer.take_pending_drop() {
        cmd_tx.push(PlaybackCommand::DropFrom { bar }).unwrap();
    }
    cmd_tx
        .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()))
        .unwrap();
    composer.send_odin(&mut cmd_tx).unwrap();
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
        if change.is_some_and(|(bar, _)| composer.playhead_bar() >= bar)
            && let Some((_, after)) = change.take()
        {
            composer.set_emotions(after[0], after[1], after[2], after[3]);
            composer.send_odin(&mut cmd_tx).unwrap();
        }
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
    let with_preset = render(load("nappe_pad"), &nappe(), BASE_EMOTIONS, 6, None);
    assert!(with_preset.peak() > 0.01, "la nappe Odin produit du son : {}", with_preset.peak());

    // Aucun preset fourni (ou nom absent) : la piste reste sur son programme
    // SoundFont ; sans banque chargée, c'est le silence, jamais de son Odin.
    let none = render(load("nappe_pad"), &[], BASE_EMOTIONS, 6, None);
    assert_eq!(none.peak(), 0.0);
    let other = render(load("nappe_pad"), &[("autre", test_preset(900.0))], BASE_EMOTIONS, 6, None);
    assert_eq!(other.peak(), 0.0);
}

#[test]
fn a_second_style_uses_the_same_mechanism_in_three_four() {
    // Les accords de la valse entrent à la seconde passe de la partie.
    let take = render(load("valse_nappe"), &nappe(), [0.5; 4], 24, None);
    assert!(take.snapshots.iter().all(|s| s.time_sig_numerator == 3));
    assert!(take.peak() > 0.01, "la valse joue sa nappe Odin : {}", take.peak());
}

#[test]
fn morphing_is_deterministic_and_each_emotion_changes_the_sound() {
    // Même profil, mêmes notes ; seule la règle de morphing change. Les
    // émotions du compositeur sont l'unique entrée du morphing, préparé sur
    // le thread principal : le callback ne fait qu'appliquer les valeurs Copy.
    let fixed = with_chords_morph(load("nappe_pad"), Vec::new());
    let base = render(fixed.clone(), &nappe(), BASE_EMOTIONS, 4, None);
    let again = render(fixed, &nappe(), BASE_EMOTIONS, 4, None);
    assert_eq!(base.audio, again.audio, "mêmes entrées, même son");

    for (axis, name) in ["arousal", "valence", "density", "tension"].iter().enumerate() {
        let mut rule = OdinMorphRule {
            param: OdinMorphParam::Fil1Freq,
            arousal: 0.0,
            valence: 0.0,
            density: 0.0,
            tension: 0.0,
        };
        *[&mut rule.arousal, &mut rule.valence, &mut rule.density, &mut rule.tension][axis] =
            -600.0;
        let take = render(
            with_chords_morph(load("nappe_pad"), vec![rule]),
            &nappe(),
            BASE_EMOTIONS,
            4,
            None,
        );
        assert_eq!(
            serde_json::to_string(&take.snapshots).unwrap(),
            serde_json::to_string(&base.snapshots).unwrap(),
            "{name} : les notes ne changent pas"
        );
        assert_ne!(take.audio, base.audio, "{name} change le rendu Odin");
        assert_eq!(take.midi, base.midi, "{name} ne change pas les événements MIDI");
    }
}

#[test]
fn an_emotion_change_in_flight_remorphs_the_playing_track() {
    let steady = render(load("nappe_pad"), &nappe(), BASE_EMOTIONS, 6, None);
    let moved = render_with_emotion_change(load("nappe_pad"), &nappe(), 6, [0.9, 0.9, 0.9, 0.9]);
    assert!(moved.peak() > 0.01);
    assert_ne!(moved.audio, steady.audio, "le changement d'émotion change le son");
    assert_eq!(
        serde_json::to_string(&moved.snapshots).unwrap(),
        serde_json::to_string(&steady.snapshots).unwrap(),
        "le morphing en vol conserve les snapshots déjà générés"
    );
    assert_eq!(moved.midi, steady.midi, "le morphing en vol conserve le MIDI");
}

#[test]
fn tied_chords_are_never_reattacked_and_no_pitch_is_attacked_twice() {
    let take = render(load("nappe_pad"), &nappe(), BASE_EMOTIONS, 8, None);
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
    let take = render(load("nappe_pad"), &nappe(), BASE_EMOTIONS, 8, Some(3));
    assert!(take.peak() > 0.01);
    let tail = &take.audio[take.audio.len() - SAMPLE_RATE as usize / 2 * 2..];
    let tail_peak = tail.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
    assert!(tail_peak < 1e-4, "aucune voix Odin ne reste coincée : {tail_peak}");
}

/// Rend une durée murale audio, sans génération ni allocation dans le callback.
fn render_seconds(playback: &mut harmonium::playback::PlaybackEngine, seconds: usize) -> Vec<f32> {
    let buffers = (seconds * SAMPLE_RATE as usize).div_ceil(BUFFER_FRAMES);
    let mut audio = Vec::with_capacity(buffers * BUFFER_FRAMES * 2);
    let mut buffer = [0.0; BUFFER_FRAMES * 2];
    for _ in 0..buffers {
        buffer.fill(0.0);
        playback.process_buffer(&mut buffer, 2, Instant::now());
        audio.extend_from_slice(&buffer);
    }
    audio
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0f32, |peak, sample| peak.max(sample.abs()))
}

fn tail_peak(samples: &[f32]) -> f32 {
    peak(&samples[samples.len() - SAMPLE_RATE as usize..])
}

#[test]
fn changing_presets_changes_audio_without_changing_snapshots_or_midi() {
    let dark = with_chords_morph(load("nappe_pad"), Vec::new());
    let mut bright = dark.clone();
    bright.odin.get_mut(&TrackId::Chords).unwrap().preset = "clair".into();
    let presets = [("nappe", test_preset(300.0)), ("clair", test_preset(6000.0))];
    let dark = render(dark, &presets, BASE_EMOTIONS, 4, None);
    let bright = render(bright, &presets, BASE_EMOTIONS, 4, None);
    assert!(dark.peak() > 0.01 && bright.peak() > 0.01, "les deux presets sont audibles");
    assert_ne!(dark.audio, bright.audio, "le preset choisi change le timbre");
    assert_eq!(
        serde_json::to_string(&dark.snapshots).unwrap(),
        serde_json::to_string(&bright.snapshots).unwrap(),
        "changer d'instrument ne change pas les notes générées"
    );
    assert_eq!(dark.midi, bright.midi, "changer d'instrument ne change pas le MIDI");
}

#[test]
fn a_non_finite_preset_is_rejected_before_entering_the_callback() {
    let invalid =
        render(load("nappe_pad"), &[("nappe", test_preset(f64::NAN))], BASE_EMOTIONS, 2, None);
    let missing = render(load("nappe_pad"), &[], BASE_EMOTIONS, 2, None);
    assert_eq!(invalid.audio, missing.audio, "le preset invalide n'entre pas dans le moteur");
    assert_eq!(invalid.midi, missing.midi);
}

#[test]
fn runtime_switches_and_queued_presets_recover_from_backpressure_without_orphan_notes() {
    let presets = [("nappe", test_preset(300.0)), ("clair", test_preset(6000.0))];
    let sources: Vec<_> =
        presets.iter().map(|(name, bytes)| OdinPresetSource { name, bytes }).collect();
    let mut dark = with_chords_morph(load("nappe_pad"), Vec::new());
    dark.instruments.chords_program = Some(0);
    // La bascule coupe les voix du preset précédent (contrat G6c).
    // Ici, des attaques réelles doivent suivre ; les tenues Tied sont
    // vérifiées séparément, sans leur imposer de réattaque artificielle.
    if let Some(params) = &mut dark.accompaniment {
        for pattern in &mut params.chords.patterns {
            for hit in &mut pattern.hits {
                hit.tie = false;
            }
        }
    }
    let mut bright = dark.clone();
    bright.odin.get_mut(&TrackId::Chords).unwrap().preset = "clair".into();
    let mut soundfont = dark.clone();
    soundfont.odin.clear();

    // Avec banque : vraie bascule SF↔Odin. Sans banque : le repli silencieux
    // rend observable toute voix Odin restée orpheline après la bascule.
    for font in [None, Some(include_bytes!("../../assets/test.sf2").as_slice())] {
        let (mut composer, mut playback, mut commands, _reports, _clock, _finished) =
            create_offline_engine(font, None, &sources, SAMPLE_RATE).unwrap();
        composer.set_seed(1);
        composer.set_profile(soundfont.clone()).unwrap();
        if let Some((bar, _)) = composer.take_pending_drop() {
            commands.push(PlaybackCommand::DropFrom { bar }).unwrap();
        }
        commands
            .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()))
            .unwrap();
        composer.send_odin(&mut commands).unwrap();
        let mut muted = [true; 16];
        muted[usize::from(TrackId::Chords.channel())] = false;
        commands.push(PlaybackCommand::SetMutedChannels(muted)).unwrap();
        // Les mesures déjà transmises restent fixes : seules les ressources
        // changent, sans DropFrom ni nouvelle génération entre les bascules.
        composer.set_writehead_lookahead(24);
        composer.generate_bars(24);
        let initial = render_seconds(&mut playback, 8);
        if font.is_some() {
            assert!(peak(&initial) > 0.001, "le programme SoundFont est réellement audible");
        } else {
            assert_eq!(peak(&initial), 0.0, "sans banque, le programme SF est silencieux");
        }

        composer.set_profile(dark.clone()).unwrap();
        while commands.push(PlaybackCommand::SetMasterVolume(1.0)).is_ok() {}
        assert!(
            composer.send_odin(&mut commands).is_err(),
            "une file pleine refuse l'update sans le considérer comme transmis"
        );
        let mut buffer = [0.0; BUFFER_FRAMES * 2];
        playback.process_buffer(&mut buffer, 2, Instant::now());
        composer.send_odin(&mut commands).unwrap();
        let first_odin = render_seconds(&mut playback, 8);
        assert!(peak(&first_odin) > 0.01, "la reprise transmet bien le preset refusé");

        // Plusieurs remplacements possèdent simultanément leurs ressources
        // en vol. Le dernier doit gagner ; le callback gardé retourne les
        // anciennes instances au main, sans les libérer sur le thread audio.
        for _ in 0..3 {
            for profile in [&bright, &soundfont, &dark] {
                composer.set_profile(profile.clone()).unwrap();
                composer.send_odin(&mut commands).unwrap();
            }
        }
        composer.set_profile(bright.clone()).unwrap();
        composer.send_odin(&mut commands).unwrap();
        let after_burst = render_seconds(&mut playback, 8);
        assert!(peak(&after_burst) > 0.01, "le dernier preset en vol reste jouable");

        for profile in [&dark, &bright, &soundfont] {
            composer.set_profile(profile.clone()).unwrap();
            composer.send_odin(&mut commands).unwrap();
        }
        let back_to_sf = render_seconds(&mut playback, 8);
        if font.is_some() {
            assert!(peak(&back_to_sf) > 0.001, "le programme SF rejoue après Odin");
        } else {
            assert!(
                tail_peak(&back_to_sf) < 1e-4,
                "aucune voix Odin orpheline après retour SF : {}",
                tail_peak(&back_to_sf)
            );
        }

        commands.push(PlaybackCommand::SetMutedChannels([true; 16])).unwrap();
        let silence = render_seconds(&mut playback, 4);
        assert!(
            tail_peak(&silence) < 1e-4,
            "les bascules ne laissent aucune note coincée : {}",
            tail_peak(&silence)
        );
        // Draine aussi le dernier retour sur le thread principal.
        composer.send_odin(&mut commands).unwrap();
    }
}

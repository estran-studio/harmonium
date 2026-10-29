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
    odin::{OdinMorphParam, OdinMorphRule, OdinZone, OdinZoneWhen},
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
    bar_frames: std::collections::BTreeMap<usize, usize>,
    diagnostics: harmonium_audio::odin_voices::OdinDiagnostics,
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

    /// Attaques musicales et tenues qui traversent réellement les frontières ;
    /// les fins exactement sur la barre sont exclues du transfert.
    fn expected_odin_attacks(&self, boundaries: &[usize]) -> usize {
        let smf = Smf::parse(&self.midi).unwrap();
        let midly::Timing::Metrical(ticks) = smf.header.timing else {
            panic!("le contrat enregistre un MIDI métrique");
        };
        let mut time = 0u32;
        let mut events = Vec::new();
        for event in &smf.tracks[0] {
            time += event.delta.as_int();
            if let TrackEventKind::Midi { channel, message } = event.kind
                && channel.as_int() == TrackId::Chords.channel()
            {
                match message {
                    MidiMessage::NoteOn { key, vel } => {
                        events.push((time, key.as_int(), vel.as_int() > 0));
                    }
                    MidiMessage::NoteOff { key, .. } => {
                        events.push((time, key.as_int(), false));
                    }
                    _ => {}
                }
            }
        }
        let musical = events.iter().filter(|(_, _, on)| *on).count();
        let transferred: usize = boundaries
            .iter()
            .map(|bar| {
                let boundary_tick = (*bar as u32 - 1) * 4 * u32::from(ticks.as_int());
                let mut active = std::collections::BTreeSet::new();
                for &(at, note, on) in &events {
                    if at < boundary_tick && on {
                        active.insert(note);
                    } else if at <= boundary_tick && !on {
                        active.remove(&note);
                    }
                }
                active.len()
            })
            .sum();
        assert!(transferred > 0, "la fixture exerce de vraies tenues traversant une frontière");
        eprintln!("{musical} attaques musicales + {transferred} tenues transférées");
        musical + transferred
    }
}

/// Profil dont la piste `Chords` garde son preset avec ces règles de morphing.
fn with_chords_morph(mut profile: StyleProfile, morph: Vec<OdinMorphRule>) -> StyleProfile {
    for zone in
        &mut profile.odin.get_mut(&TrackId::Chords).expect("la fixture déclare Chords").zones
    {
        zone.morph.clone_from(&morph);
    }
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
    render_inner(profile, presets, emotions, bars, silence_after, Vec::new())
}

/// Comme [`render`], avec un changement d'émotions à la mesure 3 : le
/// compositeur prépare de nouveaux presets morphés, envoyés au callback.
fn render_with_emotion_change(
    profile: StyleProfile,
    presets: &[(&str, Vec<u8>)],
    bars: usize,
    after: [f32; 4],
) -> Take {
    render_inner(profile, presets, BASE_EMOTIONS, bars, None, vec![(3, after)])
}

fn render_inner(
    profile: StyleProfile,
    presets: &[(&str, Vec<u8>)],
    emotions: [f32; 4],
    bars: usize,
    silence_after: Option<usize>,
    changes: Vec<(usize, [f32; 4])>,
) -> Take {
    let sources: Vec<OdinPresetSource> =
        presets.iter().map(|(name, bytes)| OdinPresetSource { name, bytes }).collect();
    let (mut composer, mut playback, mut cmd_tx, _reports, mut clock, finished) =
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
    let mut bar_frames = std::collections::BTreeMap::new();
    let mut changes = changes.into_iter().peekable();
    for _ in 0..200_000 {
        composer.generate_ahead();
        if changes.peek().is_some_and(|(bar, _)| composer.playhead_bar() >= *bar)
            && let Some((_, after)) = changes.next()
        {
            composer.set_emotions(after[0], after[1], after[2], after[3]);
            composer.send_odin(&mut cmd_tx).unwrap();
        }
        composer.send_odin(&mut cmd_tx).unwrap();
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
        let position = clock.read();
        if position.pulse_in_bar == 0 {
            bar_frames.entry(position.position.bar).or_insert(position.pulse_start_frame as usize);
        }
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
    Take {
        audio,
        snapshots,
        midi,
        bar_frames,
        diagnostics: playback.odin_diagnostics().expect("le renderer hors ligne expose Odin"),
    }
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

/// Pompe l'hôte direct, notamment les retours Odin et les cibles différées.
fn render_seconds(
    composer: &mut harmonium::composer::MusicComposer,
    playback: &mut harmonium::playback::PlaybackEngine,
    commands: &mut rtrb::Producer<PlaybackCommand>,
    seconds: usize,
) -> Vec<f32> {
    let buffers = (seconds * SAMPLE_RATE as usize).div_ceil(BUFFER_FRAMES);
    let mut audio = Vec::with_capacity(buffers * BUFFER_FRAMES * 2);
    let mut buffer = [0.0; BUFFER_FRAMES * 2];
    for _ in 0..buffers {
        if let Some((bar, _)) = composer.take_pending_drop() {
            commands.push(PlaybackCommand::DropFrom { bar }).unwrap();
        }
        composer.generate_ahead();
        composer.send_odin(commands).unwrap();
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
    bright.odin.get_mut(&TrackId::Chords).unwrap().zones[0].preset = "clair".into();
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
    // Une partie dure 6,4 s : chaque fenêtre de 8 s traverse une frontière
    // puis la totalité du fondu, sans supposer une bascule immédiate.
    dark.meter.default_bpm = 300.0;
    dark.meter.max_bpm = 300.0;
    // Ici les accords réattaquent normalement ; le contrat de zones ci-dessous
    // couvre séparément les tenues traversant une frontière de partie.
    if let Some(params) = &mut dark.accompaniment {
        for pattern in &mut params.chords.patterns {
            for hit in &mut pattern.hits {
                hit.tie = false;
            }
        }
    }
    let mut bright = dark.clone();
    bright.odin.get_mut(&TrackId::Chords).unwrap().zones[0].preset = "clair".into();
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
        composer.set_writehead_lookahead(4);
        composer.generate_bars(6);
        let initial = render_seconds(&mut composer, &mut playback, &mut commands, 8);
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
        let first_odin = render_seconds(&mut composer, &mut playback, &mut commands, 8);
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
        let after_burst = render_seconds(&mut composer, &mut playback, &mut commands, 8);
        assert!(peak(&after_burst) > 0.01, "le dernier preset en vol reste jouable");

        for profile in [&dark, &bright, &soundfont] {
            composer.set_profile(profile.clone()).unwrap();
            composer.send_odin(&mut commands).unwrap();
        }
        let back_to_sf = render_seconds(&mut composer, &mut playback, &mut commands, 8);
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
        let silence = render_seconds(&mut composer, &mut playback, &mut commands, 4);
        assert!(
            tail_peak(&silence) < 1e-4,
            "les bascules ne laissent aucune note coincée : {}",
            tail_peak(&silence)
        );
        // Draine aussi le dernier retour sur le thread principal.
        composer.send_odin(&mut commands).unwrap();
    }
}

/// Deux vrais instruments, jamais un mélange de leurs paramètres.
fn zoned_pad() -> StyleProfile {
    let mut profile = with_chords_morph(load("nappe_pad"), Vec::new());
    let spec = profile.odin.get_mut(&TrackId::Chords).unwrap();
    let default = spec.zones.last().unwrap().clone();
    spec.zones = vec![
        OdinZone {
            when: Some(OdinZoneWhen { tension: Some([0.6, 1.0]), ..Default::default() }),
            preset: "clair".into(),
            level_db: -3.0,
            morph: Vec::new(),
        },
        default,
    ];
    profile
}

fn tension(value: f32) -> [f32; 4] {
    [BASE_EMOTIONS[0], BASE_EMOTIONS[1], BASE_EMOTIONS[2], value]
}

#[test]
fn emotion_zones_switch_at_the_next_part_and_render_deterministically_without_clicks() {
    let presets = [("nappe", test_preset(300.0)), ("clair", test_preset(6000.0))];
    let automation = vec![(3, tension(0.85)), (12, tension(0.4))];
    let run =
        || render_inner(zoned_pad(), &presets, BASE_EMOTIONS, 20, Some(20), automation.clone());
    let moved = run();
    let replay = run();
    assert_eq!(moved.audio, replay.audio, "même automation, mêmes échantillons");

    let steady = render(zoned_pad(), &presets, BASE_EMOTIONS, 20, Some(20));
    let boundary = moved.bar_frames[&9];
    assert_eq!(
        moved.audio[..boundary * 2],
        steady.audio[..boundary * 2],
        "l'entrée de zone à la mesure 3 ne change rien avant la reprise de partie à 9"
    );
    let after_fade = boundary + SAMPLE_RATE as usize;
    assert_ne!(
        moved.audio[after_fade * 2..after_fade * 2 + 2048],
        steady.audio[after_fade * 2..after_fade * 2 + 2048],
        "le nouvel instrument devient audible après la frontière"
    );
    for bar in [9, 17] {
        let frame = moved.bar_frames[&bar];
        let radius = (SAMPLE_RATE * 0.35) as usize;
        let window = &moved.audio[(frame - radius) * 2..(frame + radius) * 2];
        let delta = window
            .windows(4)
            .step_by(2)
            .map(|pair| (pair[2] - pair[0]).abs().max((pair[3] - pair[1]).abs()))
            .fold(0.0f32, f32::max);
        eprintln!("frontière mesure {bar}, frame {frame}, crête inter-échantillon {delta}");
        assert!(delta < 0.1, "pas de saut brutal autour de la frontière {bar} : {delta}");
    }
    assert!(
        tail_peak(&moved.audio) < 1e-4,
        "les tenues et les changements ne laissent aucune note après le mute final"
    );
    let diagnostic = &moved.diagnostics;
    assert_eq!(diagnostic.history_overflow, 0, "l'audit couvre toutes les instances");
    assert_eq!(diagnostic.retired_count, 2, "un seul changement à chaque frontière demandée");
    assert!(diagnostic.retiring.iter().all(Option::is_none), "les fondus sont terminés");
    let instances: Vec<_> = diagnostic
        .retired
        .iter()
        .chain(&diagnostic.active)
        .flatten()
        .filter(|instance| instance.channel == usize::from(TrackId::Chords.channel()))
        .collect();
    assert_eq!(instances.len(), 3, "A initial, B, puis nouvelle instance A");
    for instance in &instances {
        assert_eq!(
            instance.note_on, instance.note_off,
            "instance {} : toutes les fins",
            instance.id
        );
        assert_eq!(instance.active_notes, 0, "instance {} : aucune note orpheline", instance.id);
        eprintln!(
            "instance {} zone {} installée mesure {} frame {} : {} NoteOn / {} NoteOff",
            instance.id,
            instance.zone,
            instance.installed_bar,
            instance.installed_frame,
            instance.note_on,
            instance.note_off
        );
    }
    let transferred_attacks: u64 = instances.iter().map(|instance| instance.note_on).sum();
    assert_eq!(
        transferred_attacks as usize,
        moved.expected_odin_attacks(&[9, 17]),
        "une seule attaque par note musicale, plus une par vraie tenue transférée"
    );
    for (instance, bar) in instances.iter().skip(1).zip([9, 17]) {
        assert_eq!(instance.installed_bar, bar);
        assert_eq!(instance.installed_frame, moved.bar_frames[&bar] as u64);
    }
    for pair in instances.windows(2) {
        assert_eq!(
            pair[0].retired_frame,
            Some(pair[1].installed_frame + (SAMPLE_RATE * 0.3).round() as u64),
            "l'ancienne instance ne quitte le callback qu'après 300 ms de fondu"
        );
    }
}

#[test]
fn oscillating_around_a_zone_edge_does_not_schedule_a_burst_of_switches() {
    let presets = [("nappe", test_preset(300.0)), ("clair", test_preset(6000.0))];
    let changes: Vec<_> =
        (3..20).map(|bar| (bar, tension(if bar % 2 == 0 { 0.59 } else { 0.61 }))).collect();
    let oscillating = render_inner(zoned_pad(), &presets, BASE_EMOTIONS, 20, None, changes);
    let steady = render(zoned_pad(), &presets, BASE_EMOTIONS, 20, None);
    assert_eq!(
        oscillating.audio, steady.audio,
        "le bruit ±0,01 autour du seuil 0,6 ne quitte pas le défaut"
    );
}

#[test]
fn a_zone_level_scales_the_track_without_changing_the_preset_master() {
    let mut zoned = zoned_pad();
    let spec = zoned.odin.get_mut(&TrackId::Chords).unwrap();
    spec.zones[0].preset = "nappe".into();
    spec.zones[0].level_db = -6.0;
    let mut unchanged = zoned.clone();
    unchanged.odin.get_mut(&TrackId::Chords).unwrap().zones.remove(0);
    let reduced = render(zoned, &nappe(), tension(0.85), 4, None);
    let original = render(unchanged, &nappe(), tension(0.85), 4, None);
    let gain = 10.0_f32.powf(-6.0 / 20.0);
    let error = reduced
        .audio
        .iter()
        .zip(&original.audio)
        .map(|(actual, base)| (actual - base * gain).abs())
        .fold(0.0f32, f32::max);
    assert!(original.peak() > 0.01);
    assert!(error < 1e-6, "le gain de zone est extérieur au synthétiseur : {error}");
}

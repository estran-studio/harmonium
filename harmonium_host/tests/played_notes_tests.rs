//! Notes jouées par le client : vraie banque, vrai callback et allocateur de
//! contrôle du host, sans périphérique audio ni attente d'horloge murale.

// Les préparations fallibles sont des assertions de test, jamais du code temps réel.
#![allow(clippy::expect_used)]

use std::{thread, time::Instant};

use harmonium::{LiveHost, LiveNoteError, playback::PlaybackEngine};
use harmonium_core::{EngineCommand, MeasureSnapshot, StyleProfile};

const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: usize = 2;
const FRAMES: usize = 512;
const SOUND_FONT: &[u8] = include_bytes!("../../assets/test.sf2");
const AUDIBLE_ENERGY: f64 = 1e-8;
const SILENT_ENERGY: f64 = 1e-12;

fn host() -> (LiveHost, PlaybackEngine) {
    LiveHost::detached(Some(SOUND_FONT), None, &[], SAMPLE_RATE)
        .expect("création de l'hôte sans périphérique")
}

fn render(playback: &mut PlaybackEngine) -> [f32; FRAMES * CHANNELS] {
    let mut buffer = [0.0; FRAMES * CHANNELS];
    playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
    buffer
}

fn energy(buffer: &[f32]) -> f64 {
    assert!(buffer.iter().all(|sample| sample.is_finite()), "signal audio fini");
    buffer.iter().map(|sample| f64::from(*sample).powi(2)).sum()
}

/// Le programme 15 de la banque réelle est un orgue soutenu. La borne dépend
/// du nombre d'échantillons rendus, jamais du temps pris par la machine.
fn assert_released(playback: &mut PlaybackEngine) {
    let buffers = SAMPLE_RATE as usize * 5 / FRAMES;
    let mut tail_energy = 0.0_f64;
    for index in 0..buffers {
        let buffer = render(playback);
        if index >= buffers - 8 {
            tail_energy = tail_energy.max(energy(&buffer));
        }
    }
    assert!(tail_energy < SILENT_ENERGY, "relâchement silencieux avant cinq secondes");
}

#[test]
fn la_poignee_joue_et_relache_depuis_un_thread_sans_hote_ni_pump() {
    let (mut host, mut playback) = host();
    let sender = host.take_note_sender().expect("poignée initiale disponible");
    assert!(host.take_note_sender().is_none(), "la poignée ne se prend qu'une fois");
    drop(host);

    // Déplacer la vraie poignée dans le thread prouve Send sans partager l'hôte.
    let sender = thread::spawn(move || {
        let mut sender = sender;
        sender.set_program(15).expect("programme orgue accepté");
        sender.note_on(9, 60, 110).expect("le canal 9 est mélodique pour le client");
        sender
    })
    .join()
    .expect("thread de notes terminé");

    assert!(energy(&render(&mut playback)) > AUDIBLE_ENERGY, "attaque au premier tampon sans pump");
    for _ in 0..16 {
        assert!(energy(&render(&mut playback)) > AUDIBLE_ENERGY, "l'orgue reste soutenu");
    }

    let _sender = thread::spawn(move || {
        let mut sender = sender;
        sender.note_off(9, 60).expect("relâchement accepté sans hôte");
        sender
    })
    .join()
    .expect("thread de relâchement terminé");
    assert_released(&mut playback);
}

#[test]
fn les_commandes_jouent_sans_mesure_et_note_off_relache() {
    let (mut host, mut playback) = host();
    host.apply(EngineCommand::SetLiveProgram(15)).expect("programme orgue accepté");
    host.apply(EngineCommand::NoteOn { channel: 0, note: 60, velocity: 110 })
        .expect("attaque acceptée");
    assert!(energy(&render(&mut playback)) > AUDIBLE_ENERGY, "attaque au premier tampon sans pump");
    for _ in 0..16 {
        assert!(energy(&render(&mut playback)) > AUDIBLE_ENERGY, "note tenue avant le relâchement");
    }
    host.apply(EngineCommand::NoteOff { channel: 0, note: 60 }).expect("relâchement accepté");
    assert_released(&mut playback);
}

#[test]
fn tous_les_canaux_clients_sont_melodiques_et_all_notes_off_coupe_immediatement() {
    let (mut host, mut playback) = host();
    let mut sender = host.take_note_sender().expect("poignée disponible");
    sender.set_program(15).expect("programme orgue accepté");
    for channel in 0..16 {
        sender.note_on(channel, 60, 110).expect("canal MIDI accepté");
        assert!(energy(&render(&mut playback)) > AUDIBLE_ENERGY, "canal {channel} audible");
        sender.all_notes_off().expect("coupure générale acceptée");
        for _ in 0..4 {
            assert!(
                energy(&render(&mut playback)) < SILENT_ENERGY,
                "canal {channel} coupé sans queue"
            );
        }
    }

    // La commande doit couper aussi une note venue de l'autre producteur.
    sender.note_on(15, 67, 110).expect("attaque acceptée");
    host.apply(EngineCommand::NoteOn { channel: 9, note: 60, velocity: 110 })
        .expect("seconde attaque acceptée");
    assert!(energy(&render(&mut playback)) > AUDIBLE_ENERGY, "deux producteurs audibles");
    host.apply(EngineCommand::AllNotesOff).expect("coupure par commande acceptée");
    for _ in 0..4 {
        assert!(energy(&render(&mut playback)) < SILENT_ENERGY, "toutes les notes sont coupées");
    }
}

fn program_audio(program: Option<u8>, commands: bool, channel: u8) -> Vec<f32> {
    let (mut host, mut playback) = host();
    let mut sender = host.take_note_sender().expect("poignée disponible");
    if commands {
        if let Some(program) = program {
            host.apply(EngineCommand::SetLiveProgram(program)).expect("programme accepté");
        }
        host.apply(EngineCommand::NoteOn { channel, note: 60, velocity: 110 })
            .expect("attaque acceptée");
    } else {
        if let Some(program) = program {
            sender.set_program(program).expect("programme accepté");
        }
        sender.note_on(channel, 60, 110).expect("attaque acceptée");
    }
    let mut audio = Vec::new();
    for _ in 0..16 {
        audio.extend_from_slice(&render(&mut playback));
    }
    assert!(energy(&audio) > AUDIBLE_ENERGY, "programme réel audible");
    audio
}

#[test]
fn le_programme_global_change_le_timbre_pendant_la_session_et_sur_tous_les_canaux() {
    let piano = program_audio(Some(0), false, 0);
    let organ = program_audio(Some(15), false, 0);
    assert_ne!(organ, piano, "le changement de programme change réellement le son");
    assert_eq!(
        program_audio(Some(15), true, 0),
        organ,
        "commande et poignée rendent le même orgue"
    );
    assert_eq!(program_audio(Some(15), false, 9), organ, "programme global jusque sur le canal 9");
    assert_eq!(program_audio(Some(15), true, 15), organ, "programme global jusque sur le canal 15");

    let (mut piano_host, mut piano_playback) = host();
    let mut piano_sender = piano_host.take_note_sender().expect("poignée témoin disponible");
    piano_sender.note_on(0, 60, 110).expect("première attaque témoin au piano");
    render(&mut piano_playback);
    piano_sender.all_notes_off().expect("coupure du piano témoin");
    piano_sender.set_program(0).expect("programme piano conservé dans le témoin");
    piano_sender.note_on(0, 60, 110).expect("seconde attaque témoin au piano");
    let mut restarted_piano = Vec::new();
    for _ in 0..16 {
        restarted_piano.extend_from_slice(&render(&mut piano_playback));
    }

    let (mut host, mut playback) = host();
    let mut sender = host.take_note_sender().expect("poignée disponible");
    sender.note_on(0, 60, 110).expect("première attaque au piano");
    render(&mut playback);
    sender.all_notes_off().expect("coupure du piano");
    sender.set_program(15).expect("changement de programme pendant la session");
    sender.note_on(0, 60, 110).expect("seconde attaque à l'orgue");
    let mut changed = Vec::new();
    for _ in 0..16 {
        changed.extend_from_slice(&render(&mut playback));
    }
    assert!(energy(&changed) > AUDIBLE_ENERGY, "nouveau programme audible dans la même session");
    assert_ne!(changed, restarted_piano, "le changement n'est pas limité à l'initialisation");
}

#[test]
fn une_velocite_nulle_equivaut_exactement_a_note_off_pour_les_deux_entrees() {
    for commands in [false, true] {
        let (mut normal, mut normal_playback) = host();
        let (mut zero, mut zero_playback) = host();
        let mut normal_sender = normal.take_note_sender().expect("poignée normale disponible");
        let mut zero_sender = zero.take_note_sender().expect("poignée zéro disponible");
        normal_sender.set_program(15).expect("programme accepté");
        zero_sender.set_program(15).expect("programme accepté");
        normal_sender.note_on(0, 60, 110).expect("attaque acceptée");
        zero_sender.note_on(0, 60, 110).expect("attaque acceptée");
        for _ in 0..16 {
            assert_eq!(render(&mut normal_playback), render(&mut zero_playback), "même note tenue");
        }
        if commands {
            normal.apply(EngineCommand::NoteOff { channel: 0, note: 60 }).expect("arrêt accepté");
            zero.apply(EngineCommand::NoteOn { channel: 0, note: 60, velocity: 0 })
                .expect("vélocité nulle acceptée");
        } else {
            normal_sender.note_off(0, 60).expect("arrêt accepté");
            zero_sender.note_on(0, 60, 0).expect("vélocité nulle acceptée");
        }
        for _ in 0..32 {
            assert_eq!(
                render(&mut normal_playback),
                render(&mut zero_playback),
                "relâchements identiques"
            );
        }
        assert_released(&mut zero_playback);
    }
}

#[test]
fn les_donnees_midi_invalides_sont_refusees_sans_polluer_le_rendu() {
    let (mut host, mut playback) = host();
    let mut sender = host.take_note_sender().expect("poignée disponible");
    for (channel, note, velocity) in [(16, 60, 100), (0, 128, 100), (0, 60, 128)] {
        assert_eq!(sender.note_on(channel, note, velocity), Err(LiveNoteError::InvalidMidiData));
        assert!(
            host.apply(EngineCommand::NoteOn { channel, note, velocity }).is_err(),
            "MIDI invalide refusé"
        );
    }
    for (channel, note) in [(16, 60), (0, 128)] {
        assert_eq!(sender.note_off(channel, note), Err(LiveNoteError::InvalidMidiData));
        assert!(
            host.apply(EngineCommand::NoteOff { channel, note }).is_err(),
            "arrêt invalide refusé"
        );
    }
    assert_eq!(sender.set_program(128), Err(LiveNoteError::InvalidMidiData));
    assert!(host.apply(EngineCommand::SetLiveProgram(128)).is_err(), "programme invalide refusé");
    assert!(
        energy(&render(&mut playback)) < SILENT_ENERGY,
        "aucune note invalide n'a atteint le son"
    );
    sender.set_program(127).expect("borne supérieure du programme acceptée");
    sender.note_on(15, 127, 127).expect("bornes supérieures MIDI acceptées");
    sender.note_off(15, 127).expect("arrêt à la borne supérieure accepté");
    sender.all_notes_off().expect("coupure acceptée");
    render(&mut playback);
}

#[test]
fn la_file_de_la_poignee_refuse_le_debordement_et_recupere_apres_un_callback() {
    let (mut host, mut playback) = host();
    let mut sender = host.take_note_sender().expect("poignée disponible");
    // Aucun callback concurrent : attendre de la place bloquerait ce test.
    for _ in 0..255 {
        sender.set_program(15).expect("place disponible avant la capacité 256");
    }
    sender.note_on(0, 60, 110).expect("dernier emplacement disponible");
    assert_eq!(sender.note_off(0, 60), Err(LiveNoteError::Full));
    assert_eq!(sender.all_notes_off(), Err(LiveNoteError::Full));
    assert!(energy(&render(&mut playback)) > AUDIBLE_ENERGY, "la note acceptée survit aux refus");
    sender.all_notes_off().expect("place récupérée après le drain");
    assert!(
        energy(&render(&mut playback)) < SILENT_ENERGY,
        "la reprise atteint effectivement le synthé"
    );
}

#[test]
fn la_file_apply_refuse_le_debordement_et_recupere_apres_un_callback() {
    let (mut host, mut playback) = host();
    for _ in 0..255 {
        host.apply(EngineCommand::SetLiveProgram(15))
            .expect("place disponible avant la capacité 256");
    }
    host.apply(EngineCommand::NoteOn { channel: 0, note: 60, velocity: 110 })
        .expect("dernier emplacement disponible");
    assert!(
        host.apply(EngineCommand::NoteOff { channel: 0, note: 60 }).is_err(),
        "file pleine refusée sans attente"
    );
    assert!(host.apply(EngineCommand::AllNotesOff).is_err(), "second refus sans attente");
    assert!(energy(&render(&mut playback)) > AUDIBLE_ENERGY, "la note acceptée survit aux refus");
    host.apply(EngineCommand::AllNotesOff).expect("place récupérée après le drain");
    assert!(
        energy(&render(&mut playback)) < SILENT_ENERGY,
        "la reprise atteint effectivement le synthé"
    );
}

#[test]
fn les_deux_files_sont_independantes_et_apply_est_draine_avant_la_poignee() {
    let (mut host, mut playback) = host();
    let mut sender = host.take_note_sender().expect("poignée disponible");
    for _ in 0..255 {
        sender.set_program(15).expect("place disponible dans la file MIDI");
    }
    sender.note_on(0, 60, 110).expect("attaque dans le dernier emplacement MIDI");
    host.apply(EngineCommand::AllNotesOff).expect("la file apply reste disponible");
    assert!(
        energy(&render(&mut playback)) > AUDIBLE_ENERGY,
        "apply coupe avant l'attaque MIDI, pas après"
    );

    sender.all_notes_off().expect("coupure MIDI acceptée");
    host.apply(EngineCommand::NoteOn { channel: 0, note: 67, velocity: 110 })
        .expect("attaque apply acceptée");
    assert!(
        energy(&render(&mut playback)) < SILENT_ENERGY,
        "la coupure MIDI arrive après l'attaque apply"
    );
}

fn snapshots(with_played_notes: bool) -> Vec<MeasureSnapshot> {
    let (mut host, mut playback) = host();
    let mut sender = host.take_note_sender().expect("poignée disponible");
    host.apply(EngineCommand::SetSeed(47)).expect("graine fixée");
    host.apply(EngineCommand::SetWriteheadLookahead(8)).expect("avance fixée");
    host.apply(EngineCommand::SetBpm(120.0)).expect("tempo fixé");
    let mut snapshots = Vec::new();
    for index in 0..600 {
        match index {
            90 => host.apply(EngineCommand::SetSwing(0.63)).expect("swing modifié"),
            180 => host
                .apply(EngineCommand::SetEmotionParams {
                    arousal: 0.7,
                    valence: 0.4,
                    density: 0.8,
                    tension: 0.3,
                })
                .expect("émotions modifiées"),
            300 => host.apply(EngineCommand::SetBpm(145.0)).expect("tempo modifié"),
            _ => {}
        }
        if with_played_notes {
            match index % 40 {
                0 => {
                    sender.set_program(15).expect("programme MIDI accepté");
                    sender.note_on(0, 60, 110).expect("attaque MIDI acceptée");
                    host.apply(EngineCommand::NoteOn { channel: 9, note: 67, velocity: 100 })
                        .expect("attaque apply acceptée");
                }
                10 => {
                    sender.note_off(0, 60).expect("relâchement MIDI accepté");
                    host.apply(EngineCommand::NoteOff { channel: 9, note: 67 })
                        .expect("relâchement apply accepté");
                }
                20 => {
                    host.apply(EngineCommand::SetLiveProgram(0)).expect("programme apply accepté");
                    sender.note_on(15, 72, 100).expect("nouvelle attaque acceptée");
                }
                30 => host.apply(EngineCommand::AllNotesOff).expect("coupure acceptée"),
                _ => {}
            }
        }
        snapshots.extend(host.pump());
        render(&mut playback);
    }
    snapshots
}

#[test]
fn les_snapshots_restent_strictement_identiques_avec_les_memes_commandes_de_composition() {
    let without = snapshots(false);
    let with = snapshots(true);
    assert!(without.len() >= 8, "plusieurs mesures réellement composées");
    assert!(without.iter().any(|snapshot| !snapshot.notes.is_empty()), "notes générées présentes");
    assert_eq!(
        serde_json::to_vec(&with).expect("sérialisation des snapshots avec jeu"),
        serde_json::to_vec(&without).expect("sérialisation des snapshots sans jeu"),
        "aucun champ ni ordre du flux composé ne dépend des notes jouées"
    );
}

#[test]
fn mutes_gains_et_seek_generes_ne_coupent_pas_une_note_jouee() {
    let (mut reference, mut reference_playback) = host();
    let (mut changed, mut changed_playback) = host();
    let mut reference_sender =
        reference.take_note_sender().expect("poignée de référence disponible");
    let mut changed_sender = changed.take_note_sender().expect("poignée modifiée disponible");
    reference_sender.set_program(15).expect("programme accepté");
    changed_sender.set_program(15).expect("programme accepté");
    reference_sender.note_on(0, 60, 110).expect("attaque acceptée");
    changed_sender.note_on(0, 60, 110).expect("attaque acceptée");
    for _ in 0..16 {
        assert_eq!(
            render(&mut reference_playback),
            render(&mut changed_playback),
            "notes initiales identiques"
        );
    }
    for channel in 0..16 {
        changed
            .apply(EngineCommand::SetChannelMute { channel, muted: true })
            .expect("mute généré accepté");
        changed
            .apply(EngineCommand::SetChannelGain { channel, gain: 0.0 })
            .expect("gain généré accepté");
    }
    changed.apply(EngineCommand::Seek(4)).expect("seek généré accepté");
    for _ in 0..32 {
        let reference_buffer = render(&mut reference_playback);
        assert!(energy(&reference_buffer) > AUDIBLE_ENERGY, "orgue toujours tenu");
        // Le seek met en file la mesure cible : le moteur généré la joue, coupé et à gain
        // nul, ce qui laisse un résidu de l'ordre de 1e-8. La note jouée, elle, reste intacte.
        let changed_buffer = render(&mut changed_playback);
        let worst = reference_buffer
            .iter()
            .zip(&changed_buffer)
            .map(|(reference, changed)| (reference - changed).abs())
            .fold(0.0_f32, f32::max);
        assert!(worst < 1e-6, "le jeu ignore le mixage et le seek générés (écart {worst:e})");
    }
}

#[test]
fn note_off_et_all_notes_off_clients_ne_relachent_pas_la_meme_note_generee() {
    let (mut reference, mut reference_playback) = host();
    let (mut changed, mut changed_playback) = host();
    let mut profile =
        StyleProfile::named("isaac_basement").expect("profil à accords tenus disponible");
    profile.odin.clear();
    profile.instruments.melody_program = 15;
    profile.instruments.chords_program = Some(15);
    profile.instruments.bass_program = Some(15);
    for host in [&mut reference, &mut changed] {
        host.apply(EngineCommand::SetStyleProfile(Box::new(profile.clone())))
            .expect("profil orgue accepté");
        host.apply(EngineCommand::SetSeed(3)).expect("graine fixée");
    }
    let reference_snapshots = reference.pump();
    let changed_snapshots = changed.pump();
    assert_eq!(
        serde_json::to_vec(&reference_snapshots).expect("snapshots de référence"),
        serde_json::to_vec(&changed_snapshots).expect("snapshots comparés"),
        "même partition avant le test audio"
    );
    let opening = reference_snapshots
        .iter()
        .find(|snapshot| snapshot.index == 1)
        .expect("première mesure composée");
    let note = opening
        .notes
        .iter()
        .find(|note| note.track <= 2 && note.start_step == 0 && note.duration_steps >= 2)
        .expect("note mélodique générée tenue au début de la mesure");
    for host in [&mut reference, &mut changed] {
        for channel in 0..16 {
            if channel != note.track {
                host.apply(EngineCommand::SetChannelMute { channel, muted: true })
                    .expect("autres canaux générés rendus muets");
            }
        }
    }
    for _ in 0..4 {
        let buffer = render(&mut reference_playback);
        assert_eq!(render(&mut changed_playback), buffer, "attaque générée identique");
    }
    let mut sender = changed.take_note_sender().expect("poignée disponible");
    // L'arrêt vise exactement le couple canal/hauteur publié, sans voix client.
    sender.note_off(note.track, note.pitch).expect("arrêt client accepté");
    changed
        .apply(EngineCommand::NoteOff { channel: note.track, note: note.pitch })
        .expect("arrêt apply accepté");
    for _ in 0..4 {
        let buffer = render(&mut reference_playback);
        assert!(energy(&buffer) > AUDIBLE_ENERGY, "note générée isolée toujours audible");
        assert_eq!(
            render(&mut changed_playback),
            buffer,
            "note_off client n'affecte aucun échantillon généré"
        );
    }
    sender.all_notes_off().expect("coupure client acceptée");
    changed.apply(EngineCommand::AllNotesOff).expect("coupure apply acceptée");
    for _ in 0..4 {
        assert_eq!(
            render(&mut changed_playback),
            render(&mut reference_playback),
            "all_notes_off client ne coupe pas les voix générées"
        );
    }
    // Dépasser les 256 voix de la voie jouée ne doit voler aucune voix générée.
    for index in 0..256 {
        let channel = (index / 128) as u8;
        let note = (index % 128) as u8;
        sender.note_on(channel, note, 110).expect("rafale MIDI acceptée");
        changed
            .apply(EngineCommand::NoteOn { channel: channel + 2, note, velocity: 110 })
            .expect("rafale apply acceptée");
    }
    render(&mut changed_playback);
    render(&mut reference_playback);
    changed.apply(EngineCommand::AllNotesOff).expect("coupure de la voie saturée");
    for _ in 0..4 {
        let reference_buffer = render(&mut reference_playback);
        assert!(
            energy(&reference_buffer) > AUDIBLE_ENERGY,
            "voix générée toujours audible après saturation"
        );
        assert_eq!(
            render(&mut changed_playback),
            reference_buffer,
            "aucun vol de voix entre les deux synthés"
        );
    }
}

#[test]
fn le_mute_et_le_volume_de_sortie_restent_globaux() {
    let (mut host, mut playback) = host();
    let mut sender = host.take_note_sender().expect("poignée disponible");
    sender.set_program(15).expect("programme accepté");
    sender.note_on(0, 60, 110).expect("attaque acceptée");
    assert!(energy(&render(&mut playback)) > AUDIBLE_ENERGY, "note initialement audible");
    host.apply(EngineCommand::SetOutputMute(true)).expect("mute de sortie accepté");
    assert!(energy(&render(&mut playback)) < SILENT_ENERGY, "le mute global coupe aussi le jeu");
    host.apply(EngineCommand::SetOutputMute(false)).expect("sortie rétablie");
    assert!(energy(&render(&mut playback)) > AUDIBLE_ENERGY, "le mute n'a pas relâché la note");
    host.apply(EngineCommand::SetMasterVolume(0.0)).expect("volume global nul accepté");
    // La rampe de volume a le temps d'atteindre zéro en nombre d'échantillons.
    for _ in 0..32 {
        render(&mut playback);
    }
    assert!(energy(&render(&mut playback)) < SILENT_ENERGY, "le volume global coupe aussi le jeu");
    host.apply(EngineCommand::SetMasterVolume(1.0)).expect("volume global rétabli");
    for _ in 0..32 {
        render(&mut playback);
    }
    assert!(
        energy(&render(&mut playback)) > AUDIBLE_ENERGY,
        "la note reste tenue après la rampe globale"
    );
}

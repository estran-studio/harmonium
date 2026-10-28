//! Callback complet sous `RTCheckAllocator` : moteur, enregistreur WAV + MIDI,
//! synthé réel, commandes, rapports et début/fin d'enregistrement.
//!
//! `PlaybackEngine::process_buffer` s'exécute entièrement sous la garde du
//! moteur : une allocation, une libération ou un verrou pris par le callback
//! panique ici (profil de débogage, allocateur de contrôle de `harmonium`).
//! Le thread d'enregistrement alloue librement : le test attend ses fichiers.
//!
//! Exception documentée, hors de ce test : le `Vec` que `oxisynth` alloue à
//! chaque NoteOn (`allow_allocation` dans `harmonium_audio`).

use std::{
    sync::{Arc, Mutex, atomic::AtomicU64},
    time::{Duration, Instant},
};

use harmonium::{
    composer::MusicComposer,
    playback::{MEASURE_RING_CAP, PlaybackCommand, PlaybackEngine, RETURN_RING_CAP, TaggedMeasure},
};
use harmonium_audio::backend::{recorder::RecorderBackend, synth_backend::SynthBackend};
use harmonium_core::events::{AudioRecordFormat, RecordFormat};
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;
const BUFFER_FRAMES: usize = 256;
const BUFFERS: usize = 900;

/// Two full takes (WAV + MIDI each): `(start buffer, stop buffer)`.
const TAKES: [(usize, usize); 2] = [(2, 400), (450, 890)];

#[test]
fn whole_callback_with_wav_and_midi_recording_is_allocation_free() -> Result<(), String> {
    let font = include_bytes!("../../assets/test.sf2");
    let finished = Arc::new(Mutex::new(Vec::new()));
    let synth = SynthBackend::new(SAMPLE_RATE, Some(font), None, &[]);
    let recorder = RecorderBackend::new(Box::new(synth), finished.clone(), SAMPLE_RATE as u32);
    let stats = recorder.stats();

    let (mut cmd_tx, cmd_rx) = rtrb::RingBuffer::<PlaybackCommand>::new(256);
    let (report_tx, mut report_rx) = rtrb::RingBuffer::<harmonium_core::EngineReport>::new(256);
    let (measure_tx, measure_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(MEASURE_RING_CAP);
    let (return_tx, return_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(RETURN_RING_CAP);
    let transport = Arc::new(AtomicU64::new(harmonium::playback::pack(1, 0)));
    let mut composer =
        MusicComposer::new_with_seed(SAMPLE_RATE, measure_tx, return_rx, transport.clone(), 7);
    let mut profile = harmonium_core::StyleProfile::builtin().clone();
    profile.arrangement = harmonium_core::trad::ReelArrangement::from_start(2, Default::default());
    profile.instruments.countermelody_program = Some(0);
    composer.set_profile(profile)?;
    composer.set_writehead_lookahead(12);
    composer.generate_bars(12);
    let (clock_tx, _clock_output) =
        triple_buffer::TripleBuffer::new(&harmonium::TransportClock::initial(SAMPLE_RATE as u32))
            .split();
    let mut engine = PlaybackEngine::new(
        SAMPLE_RATE,
        Box::new(recorder),
        measure_rx,
        return_tx,
        cmd_rx,
        report_tx,
        transport,
        clock_tx,
    );

    let mut buffer = vec![0.0f32; BUFFER_FRAMES * CHANNELS];
    let mut reports = 0;
    for i in 0..BUFFERS {
        composer.generate_ahead();
        for (start, stop) in TAKES {
            if i == start {
                let _ = cmd_tx.push(PlaybackCommand::UpdateMusicalParams(
                    composer.musical_params().playback_params(),
                ));
                let _ = cmd_tx.push(PlaybackCommand::StartRecording(AudioRecordFormat::Wav));
                let _ = cmd_tx.push(PlaybackCommand::StartRecording(AudioRecordFormat::Midi));
            }
            if i == stop {
                let _ = cmd_tx.push(PlaybackCommand::StopRecording(AudioRecordFormat::Midi));
                let _ = cmd_tx.push(PlaybackCommand::StopRecording(AudioRecordFormat::Wav));
            }
        }
        match i {
            100 => {
                let _ = cmd_tx.push(PlaybackCommand::SetChannelGain { channel: 0, gain: 0.5 });
                let _ = cmd_tx.push(PlaybackCommand::SetChannelGain { channel: 5, gain: 0.5 });
                let _ = cmd_tx.push(PlaybackCommand::SetTempo { bpm: 180.0, ramp_pulses: 4 });
            }
            200 => {
                let _ = cmd_tx.push(PlaybackCommand::SetChannelMute { channel: 1, muted: true });
                let _ = cmd_tx.push(PlaybackCommand::SetChannelMute { channel: 5, muted: true });
                let _ = cmd_tx.push(PlaybackCommand::SetMutedChannels([false; 16]));
                let _ = cmd_tx.push(PlaybackCommand::UpdateMusicalParams(
                    composer.musical_params().playback_params(),
                ));
            }
            300 => {
                let _ = cmd_tx.push(PlaybackCommand::ClearTempo);
                let _ = cmd_tx.push(PlaybackCommand::SetOutputMute(true));
            }
            320 => {
                let _ = cmd_tx.push(PlaybackCommand::SetOutputMute(false));
            }
            600 => {
                let _ = cmd_tx.push(PlaybackCommand::SetLoop { start_bar: 2, end_bar: 4 });
            }
            700 => {
                let _ = cmd_tx.push(PlaybackCommand::ClearLoop);
                let _ = cmd_tx.push(PlaybackCommand::SeekPlayhead(2));
            }
            _ => {}
        }
        buffer.fill(0.0);
        engine.process_buffer(&mut buffer, CHANNELS, Instant::now());
        while report_rx.pop().is_ok() {
            reports += 1;
        }
    }
    assert!(reports > 0, "des rapports périodiques ont été publiés");

    let deadline = Instant::now() + Duration::from_secs(20);
    while finished.lock().map_err(|e| e.to_string())?.len() < 2 * TAKES.len() {
        assert!(Instant::now() < deadline, "le thread d'enregistrement publie les 4 fichiers");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(stats.is_lossless(), "aucune perte : {stats:?}");

    let queue = finished.lock().map_err(|e| e.to_string())?;
    let wavs: Vec<_> = queue.iter().filter(|(f, _)| *f == RecordFormat::Wav).collect();
    let midis: Vec<_> = queue.iter().filter(|(f, _)| *f == RecordFormat::Midi).collect();
    assert_eq!((wavs.len(), midis.len()), (2, 2));
    let mut lengths: Vec<usize> = wavs
        .iter()
        .map(|(_, bytes)| {
            hound::WavReader::new(std::io::Cursor::new(bytes)).map(|r| r.len() as usize)
        })
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    lengths.sort_unstable();
    let mut expected: Vec<usize> =
        TAKES.iter().map(|(a, b)| (b - a) * BUFFER_FRAMES * CHANNELS).collect();
    expected.sort_unstable();
    assert_eq!(lengths, expected, "WAV : une prise = les buffers entre début et fin");
    for (_, bytes) in midis {
        let smf = Smf::parse(bytes).map_err(|e| e.to_string())?;
        let notes = smf.tracks[0]
            .iter()
            .filter(|e| {
                matches!(e.kind, TrackEventKind::Midi { message: MidiMessage::NoteOn { vel, .. }, .. } if vel.as_int() > 0)
            })
            .count();
        assert!(notes > 10, "MIDI : {notes} attaques");
        assert!(
            smf.tracks[0].iter().any(|event| matches!(
                event.kind,
                TrackEventKind::Midi { channel, message: MidiMessage::NoteOn { vel, .. } }
                    if channel.as_int() == 5 && vel.as_int() > 0
            )),
            "la seconde voix traverse le callback gardé et l'enregistreur"
        );
        assert!(
            smf.tracks[0]
                .iter()
                .any(|e| matches!(e.kind, TrackEventKind::Meta(MetaMessage::Tempo(_))))
        );
    }
    Ok(())
}

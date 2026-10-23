//! Enregistrement WAV + MIDI complet sous `RTCheckAllocator` : début, notes,
//! changement de mix et de tempo, fin — aucune allocation ni libération dans
//! le callback (le thread d'enregistrement, lui, alloue librement).
use std::sync::{Arc, Mutex};

use harmonium_audio::{
    backend::{AudioRenderer, recorder::RecorderBackend, synth_backend::SynthBackend},
    realtime::rt_check,
};
use harmonium_core::{
    events::{AudioEvent, AudioRecordFormat, RecordFormat},
    mix::ProfileMix,
    params::{PlaybackParams, TimeSignature},
};
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};

#[cfg(debug_assertions)]
#[global_allocator]
static ALLOCATOR: rt_check::RTCheckAllocator = rt_check::RTCheckAllocator;

const FRAMES: usize = 480;
const BUFFERS: usize = 400;

#[test]
fn full_recording_callback_is_allocation_free() -> Result<(), String> {
    let font = include_bytes!("../../assets/test.sf2");
    let finished = Arc::new(Mutex::new(Vec::new()));
    let synth = SynthBackend::new(44100.0, Some(font), None, &[]);
    let mut recorder = RecorderBackend::new(Box::new(synth), finished.clone(), 44100);
    let stats = recorder.stats();

    // Préparé hors callback, comme le fait le thread principal.
    let params = PlaybackParams { bpm: 150.0, ..PlaybackParams::default() };
    let faster = PlaybackParams { bpm: 180.0, time_signature: TimeSignature::new(6, 8), ..params };
    let mut profile = ProfileMix::default();
    profile.room.level = 0.3;
    let mix = profile.prepare(44100.0);
    let mut buffer = vec![0.0f32; FRAMES * 2];

    rt_check::enter_audio_context();
    recorder.handle_event(AudioEvent::UpdateMusicalParams { params });
    recorder.handle_event(AudioEvent::SetMix { mix });
    recorder.handle_event(AudioEvent::MidiClock { ticks_per_frame: 0.01 });
    recorder.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Wav });
    recorder.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Midi });
    for i in 0..BUFFERS {
        if i % 20 == 0 {
            let note = 48 + (i / 20) as u8;
            recorder.handle_event(AudioEvent::NoteOn { note, velocity: 100, channel: 0 });
            recorder.handle_event(AudioEvent::NoteOn { note: 40, velocity: 90, channel: 3 });
        }
        if i % 20 == 10 {
            let note = 48 + (i / 20) as u8;
            recorder.handle_event(AudioEvent::NoteOff { note, channel: 0 });
        }
        if i == 150 {
            recorder.handle_event(AudioEvent::SetMix { mix });
            recorder.handle_event(AudioEvent::UpdateMusicalParams { params: faster });
        }
        if i == 300 {
            recorder.handle_event(AudioEvent::AllNotesOff { channel: 0 });
        }
        recorder.process_buffer(&mut buffer, 2);
    }
    recorder.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Midi });
    recorder.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Wav });
    rt_check::exit_audio_context();

    recorder.wait_idle();
    assert!(stats.is_lossless(), "aucune perte à ce débit");
    let queue = finished.lock().map_err(|e| e.to_string())?;
    let wav = queue.iter().find(|(f, _)| *f == RecordFormat::Wav).ok_or("WAV publié")?;
    let reader = hound::WavReader::new(std::io::Cursor::new(&wav.1)).map_err(|e| e.to_string())?;
    assert_eq!(reader.len() as usize, FRAMES * 2 * BUFFERS);
    let midi = queue.iter().find(|(f, _)| *f == RecordFormat::Midi).ok_or("MIDI publié")?;
    let smf = Smf::parse(&midi.1).map_err(|e| e.to_string())?;
    let tempos = smf.tracks[0]
        .iter()
        .filter(|e| matches!(e.kind, TrackEventKind::Meta(MetaMessage::Tempo(_))))
        .count();
    assert_eq!(tempos, 2, "tempo initial + changement en cours de prise");
    let ons = smf.tracks[0]
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                TrackEventKind::Midi { message: MidiMessage::NoteOn { vel, .. }, .. } if vel.as_int() > 0
            )
        })
        .count();
    assert_eq!(ons, 2 * (BUFFERS / 20));
    Ok(())
}

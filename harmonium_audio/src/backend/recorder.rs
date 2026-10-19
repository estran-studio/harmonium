use std::{
    io::{Cursor, Seek, Write},
    sync::{Arc, Mutex},
};

use harmonium_core::{
    events::{AudioEvent, RecordFormat},
    params::MusicalParams,
};
use hound::{WavSpec, WavWriter};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

use crate::backend::AudioRenderer;

type FinishedRecordings = Arc<Mutex<Vec<(RecordFormat, Vec<u8>)>>>;

#[derive(Clone)]
struct SharedWriter {
    buffer: Arc<Mutex<Cursor<Vec<u8>>>>,
}

impl SharedWriter {
    fn new() -> Self {
        Self { buffer: Arc::new(Mutex::new(Cursor::new(Vec::with_capacity(1024 * 1024)))) }
    }
}

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut guard = match self.buffer.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        let mut guard = match self.buffer.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.flush()
    }
}

impl Seek for SharedWriter {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let mut guard = match self.buffer.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.seek(pos)
    }
}

pub struct RecorderBackend {
    inner: Box<dyn AudioRenderer>,
    // Shared storage for finished recordings
    finished_recordings: FinishedRecordings,

    // WAV State
    wav_writer: Option<WavWriter<SharedWriter>>,
    wav_output: Option<SharedWriter>,
    sample_rate: u32,

    // MIDI State
    midi_track: Option<Vec<TrackEvent<'static>>>,
    midi_steps_since_last: f64, // Changed from midi_samples_since_last: u64
    current_samples_per_step: usize,
    channel_programs: [u8; 16],
    musicxml_events: Option<Vec<(f64, AudioEvent)>>, // Changed from Vec<(u64, AudioEvent)>
    musicxml_steps_elapsed: f64,                     // Changed from musicxml_samples_elapsed: u64
    musicxml_params: MusicalParams,
}

impl RecorderBackend {
    pub fn new(
        inner: Box<dyn AudioRenderer>,
        finished_recordings: FinishedRecordings,
        sample_rate: u32,
    ) -> Self {
        Self {
            inner,
            finished_recordings,
            wav_writer: None,
            wav_output: None,
            sample_rate,
            midi_track: None,
            midi_steps_since_last: 0.0, // Changed to f64
            current_samples_per_step: 11025,
            channel_programs: {
                let mut progs = [0u8; 16];
                for (ch, prog) in
                    crate::voice_manager::gm_programs_for_style(harmonium_core::params::Style::Legacy)
                {
                    progs[ch as usize] = prog;
                }
                progs
            },
            musicxml_events: None,
            musicxml_steps_elapsed: 0.0, // Changed to f64
            musicxml_params: MusicalParams::default(),
        }
    }

    /// Access the inner backend for downcasting
    pub fn inner_mut(&mut self) -> &mut dyn AudioRenderer {
        self.inner.as_mut()
    }

    fn start_wav(&mut self) {
        let spec = WavSpec {
            channels: 2,
            sample_rate: self.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let shared = SharedWriter::new();
        self.wav_output = Some(shared.clone());
        if let Ok(writer) = WavWriter::new(shared, spec) {
            self.wav_writer = Some(writer);
        }
    }

    fn stop_wav(&mut self) {
        // Drop writer to finalize
        self.wav_writer = None;

        if let Some(shared) = self.wav_output.take()
            && let Ok(mutex) = Arc::try_unwrap(shared.buffer)
            && let Ok(cursor) = mutex.into_inner()
        {
            let data = cursor.into_inner();
            if let Ok(mut queue) = self.finished_recordings.lock() {
                queue.push((RecordFormat::Wav, data));
            }
        }
    }

    fn start_midi(&mut self) {
        // Use actual BPM from musical params, falling back to 120 if unset
        let bpm = if self.musicxml_params.bpm > 0.0 { self.musicxml_params.bpm } else { 120.0 };
        let microseconds_per_beat = (60_000_000.0 / bpm) as u32;
        let mut track = vec![TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(microseconds_per_beat.into())),
        }];

        // Émet les programmes GM initiaux pour Melody (0), Chords (1) et Bass (2)
        for ch in 0..3u8 {
            track.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Midi {
                    channel: ch.into(),
                    message: MidiMessage::ProgramChange {
                        program: self.channel_programs[ch as usize].into(),
                    },
                },
            });
        }

        self.midi_track = Some(track);
        self.midi_steps_since_last = 0.0;
    }

    fn stop_midi(&mut self) {
        if let Some(mut track_events) = self.midi_track.take() {
            // Add End of Track meta event (required by MIDI spec)
            track_events.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
            });

            let header = Header::new(Format::SingleTrack, Timing::Metrical(480.into()));
            let mut smf = Smf::new(header);
            smf.tracks.push(track_events);

            let mut buffer = Vec::new();
            if smf.write(&mut buffer).is_ok()
                && let Ok(mut queue) = self.finished_recordings.lock()
            {
                queue.push((RecordFormat::Midi, buffer));
            }
        }
    }

    fn start_musicxml(&mut self) {
        self.musicxml_events = Some(Vec::new());
        self.musicxml_steps_elapsed = 0.0; // Reset to f64
    }

    fn stop_musicxml(&mut self) {
        if let Some(events) = self.musicxml_events.take() {
            // Debug: Count captured events by type
            let note_on_count =
                events.iter().filter(|(_, e)| matches!(e, AudioEvent::NoteOn { .. })).count();
            let note_off_count =
                events.iter().filter(|(_, e)| matches!(e, AudioEvent::NoteOff { .. })).count();
            eprintln!(
                "MusicXML recorder: Captured {} NoteOn events, {} NoteOff events (total {} events)",
                note_on_count,
                note_off_count,
                events.len()
            );

            let xml = harmonium_core::export::to_musicxml(
                &events,
                &self.musicxml_params,
                1, // Dummy value - no longer used for conversion with step-based events
            );
            if let Ok(mut queue) = self.finished_recordings.lock() {
                queue.push((RecordFormat::MusicXml, xml.into_bytes()));
            }
        }
    }

    fn steps_to_ticks(&self, steps: f64) -> u32 {
        // 1 step = 1/4 beat (16th note)
        // Standard MIDI: 480 ticks per quarter note
        // Therefore: 120 ticks per step
        (steps * 120.0).round() as u32
    }
}

impl AudioRenderer for RecorderBackend {
    fn handle_event(&mut self, event: AudioEvent) {
        // Intercept recording commands
        match &event {
            AudioEvent::StartRecording { format } => match format {
                RecordFormat::Wav => self.start_wav(),
                RecordFormat::Midi => self.start_midi(),
                RecordFormat::MusicXml => self.start_musicxml(),
            },
            AudioEvent::StopRecording { format } => match format {
                RecordFormat::Wav => self.stop_wav(),
                RecordFormat::Midi => self.stop_midi(),
                RecordFormat::MusicXml => self.stop_musicxml(),
            },
            AudioEvent::TimingUpdate { samples_per_step } => {
                self.current_samples_per_step = *samples_per_step;
            }
            AudioEvent::UpdateMusicalParams { params } => {
                // Insert MIDI tempo change if BPM changed mid-recording
                if self.midi_track.is_some() && (params.bpm - self.musicxml_params.bpm).abs() > 0.01
                {
                    let bpm = if params.bpm > 0.0 { params.bpm } else { 120.0 };
                    let microseconds_per_beat = (60_000_000.0 / bpm) as u32;
                    let delta = self.steps_to_ticks(self.midi_steps_since_last);
                    if let Some(track) = &mut self.midi_track {
                        self.midi_steps_since_last = 0.0;
                        track.push(TrackEvent {
                            delta: delta.into(),
                            kind: TrackEventKind::Meta(MetaMessage::Tempo(
                                microseconds_per_beat.into(),
                            )),
                        });
                    }
                }
                for (ch, prog) in crate::voice_manager::gm_programs_for_style(params.style) {
                    self.channel_programs[ch as usize] = prog;
                }
                self.musicxml_params = *params.clone();
            }
            AudioEvent::NoteOn { .. } | AudioEvent::NoteOff { .. } => {
                // Capture MusicXML with step timestamp
                if let Some(events) = &mut self.musicxml_events {
                    events.push((self.musicxml_steps_elapsed, event.clone()));
                }
            }
            _ => {}
        }

        // MIDI recording logic - convert step delta to ticks (Heel=3, Toe=4 mapped to GM ch 9)
        let map_midi_ch = |ch: u8| -> u8 {
            if ch == 3 || ch == 4 { crate::voice_manager::GM_DRUM_CHANNEL } else { ch }
        };
        match &event {
            AudioEvent::ProgramChange { channel, program } => {
                if (*channel as usize) < 16 {
                    self.channel_programs[*channel as usize] = *program;
                }
                let midi_ch = map_midi_ch(*channel);
                let delta = self.steps_to_ticks(self.midi_steps_since_last);
                if let Some(track) = &mut self.midi_track {
                    self.midi_steps_since_last = 0.0;
                    track.push(TrackEvent {
                        delta: delta.into(),
                        kind: TrackEventKind::Midi {
                            channel: midi_ch.into(),
                            message: MidiMessage::ProgramChange { program: (*program).into() },
                        },
                    });
                }
            }
            AudioEvent::NoteOn { note, velocity, channel } => {
                let midi_ch = map_midi_ch(*channel);
                // Compute delta before mutable borrow
                let delta = self.steps_to_ticks(self.midi_steps_since_last);
                if let Some(track) = &mut self.midi_track {
                    self.midi_steps_since_last = 0.0; // Reset to 0.0
                    track.push(TrackEvent {
                        delta: delta.into(),
                        kind: TrackEventKind::Midi {
                            channel: midi_ch.into(),
                            message: MidiMessage::NoteOn {
                                key: (*note).into(),
                                vel: (*velocity).into(),
                            },
                        },
                    });
                }
            }
            AudioEvent::NoteOff { note, channel } => {
                let midi_ch = map_midi_ch(*channel);
                // Compute delta before mutable borrow
                let delta = self.steps_to_ticks(self.midi_steps_since_last);
                if let Some(track) = &mut self.midi_track {
                    self.midi_steps_since_last = 0.0; // Reset to 0.0
                    track.push(TrackEvent {
                        delta: delta.into(),
                        kind: TrackEventKind::Midi {
                            channel: midi_ch.into(),
                            message: MidiMessage::NoteOff { key: (*note).into(), vel: 0.into() },
                        },
                    });
                }
            }
            _ => {}
        }
        self.inner.handle_event(event);
    }

    fn process_buffer(&mut self, output: &mut [f32], channels: usize) {
        self.inner.process_buffer(output, channels);

        // Capture WAV
        if let Some(writer) = &mut self.wav_writer {
            for sample in output.iter() {
                writer.write_sample(*sample).ok();
            }
        }

        // Integrate samples → steps continuously
        // This is where tempo-awareness happens: when samples_per_step changes (tempo change),
        // the next buffer adds steps at a different rate, but accumulated history is preserved.
        if self.current_samples_per_step > 0 {
            let frames = (output.len() / channels) as f64;
            let steps_in_buffer = frames / self.current_samples_per_step as f64;

            // Advance MIDI time (steps)
            if self.midi_track.is_some() {
                self.midi_steps_since_last += steps_in_buffer;
            }

            // Advance MusicXML time (steps)
            if self.musicxml_events.is_some() {
                self.musicxml_steps_elapsed += steps_in_buffer;
            }
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

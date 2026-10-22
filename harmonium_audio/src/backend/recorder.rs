use std::{
    io::{Cursor, Seek, Write},
    sync::{Arc, Mutex},
};

use harmonium_core::{
    events::{AudioEvent, AudioRecordFormat, RecordFormat},
    params::{MusicalParams, TimeSignature},
};
use hound::{WavSpec, WavWriter};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

use crate::backend::AudioRenderer;

type FinishedRecordings = Arc<Mutex<Vec<(RecordFormat, Vec<u8>)>>>;

/// Map a logical channel to its MIDI channel: Heel (3) and Toe (4) share
/// the GM percussion channel 9.
fn map_midi_channel(channel: u8) -> u8 {
    if channel == 3 || channel == 4 { crate::voice_manager::GM_DRUM_CHANNEL } else { channel }
}

/// Build a MIDI time-signature meta event with the given delta.
fn time_signature_event(delta: u32, signature: &TimeSignature) -> TrackEvent<'static> {
    TrackEvent {
        delta: delta.into(),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
            signature.numerator as u8,
            signature.denominator.trailing_zeros() as u8,
            24,
            8,
        )),
    }
}

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
    midi_steps_since_last: f64,
    /// Notes currently sounding in the MIDI stream (`(channel, pitch)` after
    /// drum mapping). Closed with real `NoteOff`s on `AllNotesOff` and
    /// flushed at `stop_midi`, so every recorded `NoteOn` pairs with one
    /// `NoteOff` even when the export truncates a ringing note.
    midi_active: Vec<(u8, u8)>,
    current_samples_per_step: usize,
    channel_programs: [u8; 16],
    musical_params: MusicalParams,
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
            midi_steps_since_last: 0.0,
            midi_active: Vec::new(),
            current_samples_per_step: 11025,
            channel_programs: {
                let mut progs = [0u8; 16];
                for (ch, prog) in crate::voice_manager::gm_programs_for_profile(
                    &harmonium_core::StyleProfile::builtin().instruments,
                ) {
                    progs[ch as usize] = prog;
                }
                progs
            },
            musical_params: MusicalParams::default(),
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
        // Real tempo from the musical params (convoyed by `UpdateMusicalParams`
        // before recording starts), falling back to 120 if unset.
        let bpm = if self.musical_params.bpm > 0.0 { self.musical_params.bpm } else { 120.0 };
        let microseconds_per_beat = (60_000_000.0 / bpm) as u32;
        let mut track = vec![TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(microseconds_per_beat.into())),
        }];
        track.push(time_signature_event(0, &self.musical_params.time_signature));

        // Émet les programmes GM initiaux pour Melody (0) et éventuellement Chords (1) et Bass (2)
        for (ch, _) in
            crate::voice_manager::gm_programs_for_profile(&self.musical_params.profile_instruments)
        {
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
        self.midi_active.clear();
    }

    fn stop_midi(&mut self) {
        // Close every note still sounding (export truncated its ring): the
        // stream guarantees one NoteOff per NoteOn.
        let still_sounding = std::mem::take(&mut self.midi_active);
        for (channel, pitch) in still_sounding {
            self.push_midi_event(
                channel,
                MidiMessage::NoteOff { key: pitch.into(), vel: 0.into() },
            );
        }
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

    /// Append a MIDI channel event to the recorded track, consuming the
    /// steps elapsed since the previous event as its delta.
    fn push_midi_event(&mut self, channel: u8, message: MidiMessage) {
        let delta = self.steps_to_ticks(self.midi_steps_since_last);
        if let Some(track) = &mut self.midi_track {
            self.midi_steps_since_last = 0.0;
            track.push(TrackEvent {
                delta: delta.into(),
                kind: TrackEventKind::Midi { channel: channel.into(), message },
            });
        }
    }

    /// Close every MIDI-tracked note sounding on `channel` (answers an
    /// `AllNotesOff` so the file mirrors what the synth cut).
    fn midi_all_notes_off(&mut self, channel: u8) {
        let mut i = 0;
        while i < self.midi_active.len() {
            if self.midi_active[i].0 == channel {
                let (_, pitch) = self.midi_active.swap_remove(i);
                self.push_midi_event(
                    channel,
                    MidiMessage::NoteOff { key: pitch.into(), vel: 0.into() },
                );
            } else {
                i += 1;
            }
        }
    }

    fn update_timing(&mut self, bpm: f32, signature: TimeSignature, swing_ratio: f32) {
        if self.midi_track.is_some() {
            if (bpm - self.musical_params.bpm).abs() > 0.01 {
                let bpm = if bpm > 0.0 { bpm } else { 120.0 };
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
            if signature != self.musical_params.time_signature {
                let delta = self.steps_to_ticks(self.midi_steps_since_last);
                if let Some(track) = &mut self.midi_track {
                    self.midi_steps_since_last = 0.0;
                    track.push(time_signature_event(delta, &signature));
                }
            }
        }
        self.musical_params.bpm = bpm;
        self.musical_params.time_signature = signature;
        self.musical_params.swing_ratio = swing_ratio;
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
                AudioRecordFormat::Wav => self.start_wav(),
                AudioRecordFormat::Midi => self.start_midi(),
            },
            AudioEvent::StopRecording { format } => match format {
                AudioRecordFormat::Wav => self.stop_wav(),
                AudioRecordFormat::Midi => self.stop_midi(),
            },
            AudioEvent::TimingUpdate { samples_per_step } => {
                self.current_samples_per_step = *samples_per_step;
            }
            AudioEvent::UpdateMusicalParams { params } => {
                self.update_timing(params.bpm, params.time_signature, params.swing_ratio);
                for (ch, prog) in
                    crate::voice_manager::gm_programs_for_profile(&params.profile_instruments)
                {
                    self.channel_programs[ch as usize] = prog;
                }
                self.musical_params = *params.clone();
            }
            AudioEvent::MeasureTiming { bpm, time_signature, swing_ratio } => {
                self.update_timing(*bpm, *time_signature, *swing_ratio);
            }
            _ => {}
        }

        // MIDI recording: step deltas become ticks (Heel=3, Toe=4 on GM ch 9).
        match &event {
            AudioEvent::ProgramChange { channel, program } => {
                if (*channel as usize) < 16 {
                    self.channel_programs[*channel as usize] = *program;
                }
                self.push_midi_event(
                    map_midi_channel(*channel),
                    MidiMessage::ProgramChange { program: (*program).into() },
                );
            }
            AudioEvent::NoteOn { note, velocity, channel } => {
                let midi_ch = map_midi_channel(*channel);
                self.push_midi_event(
                    midi_ch,
                    MidiMessage::NoteOn { key: (*note).into(), vel: (*velocity).into() },
                );
                if self.midi_track.is_some() && !self.midi_active.contains(&(midi_ch, *note)) {
                    self.midi_active.push((midi_ch, *note));
                }
            }
            AudioEvent::NoteOff { note, channel } => {
                let midi_ch = map_midi_channel(*channel);
                self.push_midi_event(
                    midi_ch,
                    MidiMessage::NoteOff { key: (*note).into(), vel: 0.into() },
                );
                if let Some(pos) =
                    self.midi_active.iter().position(|&(ch, p)| ch == midi_ch && p == *note)
                {
                    self.midi_active.swap_remove(pos);
                }
            }
            AudioEvent::AllNotesOff { channel } => {
                self.midi_all_notes_off(map_midi_channel(*channel));
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
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use harmonium_core::params::TimeSignature;

    use super::*;

    struct NullRenderer;

    impl AudioRenderer for NullRenderer {
        fn handle_event(&mut self, _event: AudioEvent) {}
        fn process_buffer(&mut self, _output: &mut [f32], _channels: usize) {}
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    fn params_at(bpm: f32, numerator: usize, denominator: usize) -> MusicalParams {
        MusicalParams {
            bpm,
            time_signature: TimeSignature { numerator, denominator },
            ..MusicalParams::default()
        }
    }

    fn advance(rec: &mut RecorderBackend, steps: f64) {
        // `TimingUpdate` then a silent buffer: `steps` sixteenth-steps elapse.
        rec.handle_event(AudioEvent::TimingUpdate { samples_per_step: 1000 });
        let frames = (steps * 1000.0) as usize;
        rec.process_buffer(&mut vec![0.0f32; frames * 2], 2);
    }

    /// The recorded MIDI mirrors the played stream: real tempo and time
    /// signature metas, one `NoteOff` per `NoteOn` (same channel and pitch),
    /// `AllNotesOff` translated, truncated rings closed at stop.
    #[test]
    fn midi_mirrors_tempo_signature_and_note_lifetimes() -> Result<(), String> {
        let finished: FinishedRecordings = Arc::new(Mutex::new(Vec::new()));
        let mut rec = RecorderBackend::new(Box::new(NullRenderer), finished.clone(), 44100);

        rec.handle_event(AudioEvent::UpdateMusicalParams {
            params: Box::new(params_at(230.0, 2, 2)),
        });
        rec.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Midi });
        rec.handle_event(AudioEvent::NoteOn { note: 60, velocity: 90, channel: 0 });
        advance(&mut rec, 2.0);
        rec.handle_event(AudioEvent::NoteOff { note: 60, channel: 0 });
        rec.handle_event(AudioEvent::NoteOn { note: 62, velocity: 90, channel: 0 });
        advance(&mut rec, 1.0);
        // Legato cut: the synth drops 62 here, so the file must too.
        rec.handle_event(AudioEvent::AllNotesOff { channel: 0 });
        // Truncated ring: no NoteOff before stop.
        rec.handle_event(AudioEvent::NoteOn { note: 40, velocity: 90, channel: 3 });
        rec.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Midi });

        let queue = finished.lock().map_err(|e| e.to_string())?;
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].0, RecordFormat::Midi);
        let smf = midly::Smf::parse(&queue[0].1).map_err(|e| e.to_string())?;
        assert_eq!(smf.tracks.len(), 1);
        let track = &smf.tracks[0];

        let tempos: Vec<u32> = track
            .iter()
            .filter_map(|e| match e.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(t)) => Some(t.as_int()),
                _ => None,
            })
            .collect();
        assert_eq!(tempos, vec![(60_000_000.0 / 230.0) as u32], "tempo meta: {tempos:?}");
        let sigs: Vec<(u8, u8)> = track
            .iter()
            .filter_map(|e| match e.kind {
                TrackEventKind::Meta(MetaMessage::TimeSignature(n, d, _, _)) => Some((n, d)),
                _ => None,
            })
            .collect();
        assert_eq!(sigs, vec![(2, 1)], "time signature 2/2: {sigs:?}");

        let mut ons: Vec<(u8, u8)> = Vec::new();
        let mut offs: Vec<(u8, u8)> = Vec::new();
        for e in track {
            match e.kind {
                TrackEventKind::Midi { channel, message: MidiMessage::NoteOn { key, vel } }
                    if vel.as_int() > 0 =>
                {
                    ons.push((channel.as_int(), key.as_int()))
                }
                TrackEventKind::Midi { channel, message: MidiMessage::NoteOff { key, .. } } => {
                    offs.push((channel.as_int(), key.as_int()))
                }
                _ => {}
            }
        }
        ons.sort_unstable();
        offs.sort_unstable();
        // Ch 3 (heel) is mapped to GM drums (9); the truncated 40 still closes.
        assert_eq!(ons, vec![(0, 60), (0, 62), (9, 40)], "ons: {ons:?}");
        assert_eq!(offs, vec![(0, 60), (0, 62), (9, 40)], "offs: {offs:?}");
        Ok(())
    }

    /// A tempo or signature change mid-recording lands in the file.
    #[test]
    fn midi_follows_mid_recording_tempo_change() -> Result<(), String> {
        let finished: FinishedRecordings = Arc::new(Mutex::new(Vec::new()));
        let mut rec = RecorderBackend::new(Box::new(NullRenderer), finished.clone(), 44100);

        rec.handle_event(AudioEvent::UpdateMusicalParams {
            params: Box::new(params_at(200.0, 2, 2)),
        });
        rec.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Midi });
        advance(&mut rec, 4.0);
        rec.handle_event(AudioEvent::UpdateMusicalParams {
            params: Box::new(params_at(240.0, 2, 2)),
        });
        rec.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Midi });

        let queue = finished.lock().map_err(|e| e.to_string())?;
        let smf = midly::Smf::parse(&queue[0].1).map_err(|e| e.to_string())?;
        let tempos: Vec<u32> = smf.tracks[0]
            .iter()
            .filter_map(|e| match e.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(t)) => Some(t.as_int()),
                _ => None,
            })
            .collect();
        assert_eq!(
            tempos,
            vec![(60_000_000.0 / 200.0) as u32, (60_000_000.0 / 240.0) as u32],
            "tempos: {tempos:?}"
        );
        Ok(())
    }
}

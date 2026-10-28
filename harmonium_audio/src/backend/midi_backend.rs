use std::sync::{Arc, Mutex};

use harmonium_core::events::AudioEvent;
use midly::{Format, Header, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind};

use crate::backend::AudioRenderer;

pub struct MidiBackend {
    track: Arc<Mutex<Vec<TrackEvent<'static>>>>,
    samples_since_last_event: u64,
    current_samples_per_step: usize,
}

impl Default for MidiBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiBackend {
    #[must_use]
    pub fn new() -> Self {
        Self {
            track: Arc::new(Mutex::new(Vec::new())),
            samples_since_last_event: 0,
            current_samples_per_step: 11025, // Default fallback
        }
    }

    fn samples_to_ticks(&self, samples: u64) -> u32 {
        if self.current_samples_per_step == 0 {
            return 0;
        }
        // 1 step = 1/4 beat (16th note)
        // ticks per beat = 480
        // ticks per step = 120
        ((samples as f64 * 120.0) / self.current_samples_per_step as f64) as u32
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let track_guard = match self.track.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        // Clone events to create a track
        let track: Track = track_guard.clone();

        let header = Header::new(Format::SingleTrack, Timing::Metrical(480.into()));
        let mut smf = Smf::new(header);
        smf.tracks.push(track);

        smf.save(path)?;
        Ok(())
    }
}

impl AudioRenderer for MidiBackend {
    fn handle_event(&mut self, event: AudioEvent) {
        match event {
            AudioEvent::TimingUpdate { samples_per_step } => {
                self.current_samples_per_step = samples_per_step;
            }
            AudioEvent::UpdateMusicalParams { .. }
            | AudioEvent::EmotionUpdate { .. }
            | AudioEvent::MidiClock { .. }
            | AudioEvent::MeasureTiming { .. } => {
                // Ignore - only RecorderBackend needs this
            }
            AudioEvent::NoteOn { note, velocity, channel } => {
                let delta = self.samples_to_ticks(self.samples_since_last_event);
                self.samples_since_last_event = 0;

                let mut track = match self.track.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                track.push(TrackEvent {
                    delta: delta.into(),
                    kind: TrackEventKind::Midi {
                        channel: channel.into(),
                        message: MidiMessage::NoteOn { key: note.into(), vel: velocity.into() },
                    },
                });
            }
            AudioEvent::NoteOff { note, channel } => {
                let delta = self.samples_to_ticks(self.samples_since_last_event);
                self.samples_since_last_event = 0;

                let mut track = match self.track.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                track.push(TrackEvent {
                    delta: delta.into(),
                    kind: TrackEventKind::Midi {
                        channel: channel.into(),
                        message: MidiMessage::NoteOff { key: note.into(), vel: 0.into() },
                    },
                });
            }
            _ => {}
        }
    }

    fn process_buffer(&mut self, output: &mut [f32], channels: usize) {
        self.samples_since_last_event += (output.len() / channels) as u64;
        output.fill(0.0);
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

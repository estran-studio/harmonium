//! Writehead and Playhead - the two sides of the timeline architecture
//!
//! - **Writehead** (main thread): Generates measures ahead of playback using the
//!   same algorithms as the legacy engine (Sequencer, HarmonicDriver, HarmonyNavigator).
//!   Writes completed `Measure` structs to a ring buffer for the Playhead.
//!
//! - **Playhead** (audio thread): Reads measures from the ring buffer, converts
//!   musical time (steps) to sample time, and emits `AudioEvent`s to the renderer.
//!   Zero allocations in steady state.

use super::{Measure, MusicalPosition, ScoreTimeline, TempoMap, TrackId};
use crate::{events::AudioEvent, params::TimeSignature};

/// Writehead state - generates measures ahead of playback.
///
/// Lives on the main thread. Uses TimelineGenerator (Phase 2) to produce
/// `Measure` structs and pushes them to the Playhead via ring buffer.
pub struct Writehead {
    /// Current write position (next measure to generate)
    pub current_bar: usize,
    /// How many measures to generate ahead of the playhead
    pub lookahead: usize,
    /// Master copy of the score
    pub timeline: ScoreTimeline,
    /// Tempo map for timing calculations
    pub tempo_map: TempoMap,
}

impl Writehead {
    /// Create a new writehead
    #[must_use]
    pub fn new(sample_rate: f64, ticks_per_beat: usize) -> Self {
        Self {
            current_bar: 1,
            lookahead: 4,
            timeline: ScoreTimeline::with_default_capacity(),
            tempo_map: TempoMap::new(sample_rate, ticks_per_beat),
        }
    }

    /// Check if we need to generate more measures
    #[must_use]
    pub fn needs_generation(&self, playhead_bar: usize) -> bool {
        self.current_bar < playhead_bar + self.lookahead
    }

    /// Store a generated measure in the timeline and advance the write position
    pub fn commit_measure(&mut self, measure: Measure) {
        let idx = measure.index;
        self.timeline.push_measure(measure);
        self.current_bar = idx + 1;
    }

    /// Reset the writehead to bar 1
    pub fn reset(&mut self) {
        self.current_bar = 1;
        self.timeline.clear();
    }
}

/// Playhead state - reads measures and emits audio events.
///
/// Lives on the audio thread. Reads pre-generated `Measure` structs and
/// converts them to sample-accurate `AudioEvent`s.
///
/// Critical constraint: ZERO allocations after initialization.
///
/// Ownership: the currently-played measure is held as a `Box<Measure>` so
/// the audio thread only moves a pointer (no clone, no alloc). A measure
/// the playhead leaves is NOT dropped here — it moves to the `finished`
/// slot, and the caller reclaims it with [`Playhead::take_finished`] to
/// hand it back to the main thread (which alone may deallocate it).
pub struct Playhead {
    /// Current playback position
    pub position: MusicalPosition,
    /// Current measure being played (pointer-moved in, never cloned)
    current_measure: Option<Box<Measure>>,
    /// Measure just left by the playhead, awaiting reclaim by the caller.
    /// Never dropped on this thread: reclaim it and send it back.
    finished_measure: Option<Box<Measure>>,
    /// Samples accumulated since last step
    sample_counter: usize,
    /// Samples per step at current tempo
    samples_per_step: usize,
    /// Sample rate (samples per second), fixed at construction
    sample_rate: f64,
    /// Time signature for current measure
    time_signature: TimeSignature,
    /// Ticks per beat (resolution)
    ticks_per_beat: usize,
    /// Pre-allocated event buffer (reused each tick)
    events: Vec<AudioEvent>,
    /// Active bass note for NoteOff tracking
    active_bass_note: Option<u8>,
    /// Active lead notes for NoteOff tracking
    active_lead_notes: Vec<u8>,
    /// Next note indices per track (to avoid re-scanning)
    track_cursors: [usize; TrackId::ALL.len()],
}

impl Playhead {
    /// Create a new playhead
    #[must_use]
    pub fn new(sample_rate: f64, ticks_per_beat: usize) -> Self {
        let samples_per_step = (sample_rate * 60.0 / 120.0 / ticks_per_beat as f64) as usize;
        Self {
            position: MusicalPosition::new(1, 1, 0),
            current_measure: None,
            finished_measure: None,
            sample_counter: 0,
            samples_per_step,
            sample_rate,
            time_signature: TimeSignature::default(),
            ticks_per_beat,
            events: Vec::with_capacity(8),
            active_bass_note: None,
            active_lead_notes: Vec::with_capacity(8),
            track_cursors: [0; TrackId::ALL.len()],
        }
    }

    /// Load a measure for playback.
    ///
    /// Called when the playhead crosses a barline and needs the next measure
    /// ([`Playhead::needs_measure`] is true, so no box is currently held).
    /// Takes the box by pointer move: no clone, no allocation.
    pub fn load_measure(&mut self, measure: Box<Measure>) {
        debug_assert!(
            self.current_measure.is_none(),
            "load_measure called while a measure is still loaded"
        );
        self.samples_per_step = {
            let steps_per_beat = self.ticks_per_beat as f64;
            let bpm = measure.tempo as f64;
            (self.sample_rate * 60.0 / bpm / steps_per_beat) as usize
        };
        self.time_signature = measure.time_signature;
        self.current_measure = Some(measure);
        self.track_cursors = [0; TrackId::ALL.len()];
    }

    /// Unload the currently-held measure without dropping it.
    ///
    /// The caller owns the returned box (it goes back to the main thread
    /// through the return ring). After this call the playhead needs a new
    /// measure. Must be called before [`Playhead::seek_to_bar`].
    pub fn unload_current(&mut self) -> Option<Box<Measure>> {
        self.current_measure.take()
    }

    /// Reclaim the measure the playhead just left at a barline.
    ///
    /// Returns the exhausted box so the caller can hand it back to the main
    /// thread through the return ring. Returns `None` if the playhead has
    /// not crossed a barline since the last reclaim.
    pub fn take_finished(&mut self) -> Option<Box<Measure>> {
        self.finished_measure.take()
    }

    /// Process one step's worth of audio events from the current measure.
    ///
    /// Returns a slice of events to emit. The caller should send these
    /// to the audio renderer.
    ///
    /// This is the hot path - must be allocation-free.
    pub fn tick(&mut self) -> &[AudioEvent] {
        self.events.clear();

        let current_step = self.position.step_in_bar(self.ticks_per_beat);

        if let Some(ref measure) = self.current_measure {
            // Collect notes to emit (avoid borrow conflict with self.emit_note_on)
            // We process each track inline to avoid the double-borrow
            for (track_idx, &track_id) in TrackId::ALL.iter().enumerate() {
                let notes = measure.notes_for_track(track_id);
                let cursor = self.track_cursors[track_idx];

                // Find the range of notes at current_step
                let mut new_cursor = cursor;

                // Skip past notes before current step
                while new_cursor < notes.len() && notes[new_cursor].start_step < current_step {
                    new_cursor += 1;
                }

                // Emit all notes starting at this step
                while new_cursor < notes.len() && notes[new_cursor].start_step == current_step {
                    let note = &notes[new_cursor];
                    let channel = track_id.channel();

                    match track_id {
                        TrackId::Bass => {
                            if let Some(old_note) = self.active_bass_note.take() {
                                self.events.push(AudioEvent::NoteOff { note: old_note, channel });
                            }
                            self.events.push(AudioEvent::NoteOn {
                                note: note.pitch,
                                velocity: note.velocity,
                                channel,
                            });
                            self.active_bass_note = Some(note.pitch);
                        }
                        TrackId::Melody => {
                            if !self.active_lead_notes.is_empty() {
                                self.events.push(AudioEvent::AllNotesOff { channel });
                                self.active_lead_notes.clear();
                            }
                            self.events.push(AudioEvent::NoteOn {
                                note: note.pitch,
                                velocity: note.velocity,
                                channel,
                            });
                            self.active_lead_notes.push(note.pitch);
                        }
                        TrackId::Chords | TrackId::Heel | TrackId::Toe => {
                            self.events.push(AudioEvent::NoteOn {
                                note: note.pitch,
                                velocity: note.velocity,
                                channel,
                            });
                        }
                    }

                    new_cursor += 1;
                }

                self.track_cursors[track_idx] = new_cursor;
            }
        }

        // Advance position
        self.advance_position();

        &self.events
    }

    /// Advance the musical position by one step
    fn advance_position(&mut self) {
        self.position.tick += 1;
        if self.position.tick >= self.ticks_per_beat {
            self.position.tick = 0;
            self.position.beat += 1;
            // A bar is `steps_per_bar` grid steps, not `numerator ×
            // ticks_per_beat`: 2/2 has four quarter-note beats (two half
            // notes), 6/8 has three (six eighths grouped in twos). For x/4
            // the numerator already equals the grid-beat count, so 4/4 and
            // 3/4 are unchanged. `.max(1)` keeps an exotic sub-grid meter
            // from wrapping at beat 0.
            let beats_per_bar = (self.time_signature.steps_per_bar(self.ticks_per_beat)
                / self.ticks_per_beat)
                .max(1);
            if self.position.beat > beats_per_bar {
                self.position.beat = 1;
                self.position.bar += 1;
                // Current measure is exhausted: park it for the caller to
                // reclaim with `take_finished` (never dropped on this thread).
                // The slot is empty here: the caller reclaims after every tick.
                debug_assert!(
                    self.finished_measure.is_none(),
                    "finished measure was not reclaimed"
                );
                self.finished_measure = self.current_measure.take();
            }
        }
    }

    /// Whether the playhead needs a new measure loaded
    #[must_use]
    pub fn needs_measure(&self) -> bool {
        self.current_measure.is_none()
    }

    /// Current bar number being played
    #[must_use]
    pub fn current_bar(&self) -> usize {
        self.position.bar
    }

    /// Seek to a specific bar (for rewind/replay).
    ///
    /// Resets the position, the per-track cursors and the active notes, but
    /// never touches the held boxes: the caller must [`Playhead::unload_current`]
    /// (and [`Playhead::take_finished`]) before seeking, or deliberately keep
    /// the loaded box when re-seeking its own bar (bar restart).
    pub fn seek_to_bar(&mut self, bar: usize) {
        self.position = MusicalPosition::new(bar, 1, 0);
        self.track_cursors = [0; TrackId::ALL.len()];
        // Stop any active notes
        self.active_bass_note = None;
        self.active_lead_notes.clear();
    }

    /// Reset the playhead to bar 1
    pub fn reset(&mut self) {
        self.seek_to_bar(1);
        self.sample_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::{Articulation, TimelineNote};

    #[test]
    fn test_writehead_creation() {
        let wh = Writehead::new(44100.0, 4);
        assert_eq!(wh.current_bar, 1);
        assert_eq!(wh.lookahead, 4);
        assert!(wh.timeline.is_empty());
    }

    #[test]
    fn test_writehead_needs_generation() {
        let mut wh = Writehead::new(44100.0, 4);
        assert!(wh.needs_generation(1)); // Playhead at 1, write at 1, need 4 ahead

        // Generate 5 measures (bars 1-5), write position advances to 6
        for i in 1..=5 {
            wh.commit_measure(Measure::new(i, TimeSignature::default(), 120.0, 16));
        }
        // current_bar=6, playhead=1, 6 >= 1+4=5, so no generation needed
        assert!(!wh.needs_generation(1));
        // But if playhead advances to bar 3, need up to bar 7, so generation needed again
        assert!(wh.needs_generation(3));
    }

    #[test]
    fn test_writehead_commit_measure() {
        let mut wh = Writehead::new(44100.0, 4);
        let measure = Measure::new(1, TimeSignature::default(), 120.0, 16);
        wh.commit_measure(measure);
        assert_eq!(wh.current_bar, 2);
        assert_eq!(wh.timeline.len(), 1);
    }

    #[test]
    fn test_playhead_creation() {
        let ph = Playhead::new(44100.0, 4);
        assert_eq!(ph.position.bar, 1);
        assert!(ph.needs_measure());
    }

    #[test]
    fn test_playhead_load_and_tick() {
        let mut ph = Playhead::new(44100.0, 4);

        let mut measure = Measure::new(1, TimeSignature::default(), 120.0, 16);
        measure.add_note(
            TrackId::Bass,
            TimelineNote {
                id: 1,
                pitch: 36,
                start_step: 0,
                duration_steps: 1,
                velocity: 100,
                articulation: Articulation::Staccato,
            },
        );

        ph.load_measure(Box::new(measure));
        assert!(!ph.needs_measure());

        // First tick should emit the bass note
        let events = ph.tick();
        assert!(!events.is_empty());
        // Should have a NoteOn for bass
        let has_note_on = events
            .iter()
            .any(|e| matches!(e, AudioEvent::NoteOn { note: 36, velocity: 100, channel: 2 }));
        assert!(has_note_on, "Expected NoteOn for bass at step 0");
    }

    #[test]
    fn test_playhead_load_measure_uses_real_sample_rate() {
        let mut ph = Playhead::new(48000.0, 4);
        let measure = Measure::new(1, TimeSignature::default(), 120.0, 16);
        ph.load_measure(Box::new(measure));

        // 48000 samples/s * 60 s/min / 120 bpm / 4 steps-per-beat = 6000.
        assert_eq!(ph.samples_per_step, 6000);
    }

    #[test]
    fn test_playhead_seek() {
        let mut ph = Playhead::new(44100.0, 4);
        ph.seek_to_bar(10);
        assert_eq!(ph.current_bar(), 10);
        assert!(ph.needs_measure());
    }

    #[test]
    fn test_playhead_bar_crossing() {
        let mut ph = Playhead::new(44100.0, 4);
        let measure = Measure::new(1, TimeSignature::default(), 120.0, 16);
        ph.load_measure(Box::new(measure));

        // Tick through an entire 4/4 bar (16 steps)
        for _ in 0..16 {
            ph.tick();
        }

        // Should need a new measure now; the exhausted box is parked for
        // reclaim instead of being dropped.
        assert!(ph.needs_measure());
        assert_eq!(ph.current_bar(), 2);
        let finished = ph.take_finished();
        assert!(finished.is_some_and(|m| m.index == 1));
        assert!(ph.take_finished().is_none());
    }
}

//! Writehead and Playhead - the two sides of the timeline architecture
//!
//! - **Writehead** (main thread): Generates measures ahead of playback from a validated profile.
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

/// A sounding note awaiting its `NoteOff` at an absolute `(bar, step)` position.
///
/// Bars are 1-based, steps 0-based within the bar. Ends spilling past the
/// barline are stored against the future bar and fired there, so a note's
/// MIDI `NoteOff` lands exactly at its written duration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingOff {
    channel: u8,
    pitch: u8,
    end_bar: usize,
    end_step: usize,
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
    /// Sounding notes awaiting their `NoteOff`, scheduled at an absolute
    /// `(bar, step)`. Fired by [`Playhead::tick`] at the note's real end
    /// (`start_step + duration_steps`, triggers ring one step), including
    /// across barlines. Reused across ticks: no steady-state allocation.
    pending_offs: Vec<PendingOff>,
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
            events: Vec::with_capacity(2 * 64 + 7),
            // Pré-alloué hors callback : 5 pistes × notes tenues simultanées
            // (accords, croisements de barre) restent bien sous 64.
            pending_offs: Vec::with_capacity(64),
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
            let steps_per_beat = measure.steps_per_quarter();
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

        let bar = self.position.bar;
        let current_step = self.position.step_in_bar(self.ticks_per_beat);

        // NoteOffs before NoteOns: a note ending where another starts
        // releases first (same convention as the offline MIDI export).
        // Overdue ends (`end_bar < bar`, shorter follow-up measure) also
        // fire here, even with no measure loaded.
        let mut i = 0;
        while i < self.pending_offs.len() {
            let p = self.pending_offs[i];
            if p.end_bar < bar || (p.end_bar == bar && p.end_step == current_step) {
                self.events.push(AudioEvent::NoteOff { note: p.pitch, channel: p.channel });
                self.pending_offs.swap_remove(i);
            } else {
                i += 1;
            }
        }

        if let Some(measure) = &self.current_measure {
            let steps = measure.steps.max(1);
            // Collect notes to emit (avoid borrow conflict with self.events /
            // self.pending_offs): each track is processed inline.
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

                    // Retrigger: same pitch still sounding (legato overlap)
                    // closes now, so every NoteOn pairs with exactly one NoteOff.
                    if let Some(pos) = self
                        .pending_offs
                        .iter()
                        .position(|p| p.channel == channel && p.pitch == note.pitch)
                    {
                        let p = self.pending_offs.swap_remove(pos);
                        self.events.push(AudioEvent::NoteOff { note: p.pitch, channel: p.channel });
                    }
                    self.events.push(AudioEvent::NoteOn {
                        note: note.pitch,
                        velocity: note.velocity,
                        channel,
                    });
                    // Triggers (percussion, duration 0) ring one step so the
                    // MIDI stream still closes them; pitched notes ring their
                    // written duration, spilling past the barline if needed.
                    let end = current_step + note.duration_steps.max(1);
                    self.pending_offs.push(PendingOff {
                        channel,
                        pitch: note.pitch,
                        end_bar: bar + end / steps,
                        end_step: end % steps,
                    });

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
        let next_step = self.position.step_in_bar(self.ticks_per_beat) + 1;
        let steps = self
            .current_measure
            .as_ref()
            .map_or_else(|| self.time_signature.steps_per_bar(self.ticks_per_beat), |m| m.steps);
        if next_step >= steps.max(1) {
            self.position.tick = 0;
            self.position.beat = 1;
            self.position.bar += 1;
            debug_assert!(self.finished_measure.is_none(), "finished measure was not reclaimed");
            self.finished_measure = self.current_measure.take();
        } else {
            self.position.tick += 1;
            if self.position.tick >= self.ticks_per_beat {
                self.position.tick = 0;
                self.position.beat += 1;
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
        // Drop scheduled NoteOffs (the engine voices AllNotesOff on seeks).
        self.pending_offs.clear();
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
    fn note(kind: &AudioEvent) -> Option<(bool, u8, u8)> {
        match *kind {
            AudioEvent::NoteOn { note, channel, .. } => Some((true, channel, note)),
            AudioEvent::NoteOff { note, channel } => Some((false, channel, note)),
            _ => None,
        }
    }

    #[test]
    fn test_playhead_emits_noteoff_at_written_duration() {
        let mut ph = Playhead::new(44100.0, 4);
        let mut measure = Measure::new(1, TimeSignature::default(), 120.0, 16);
        let add =
            |m: &mut Measure, track: TrackId, id: u64, pitch: u8, start: usize, dur: usize| {
                m.add_note(
                    track,
                    TimelineNote {
                        id,
                        pitch,
                        start_step: start,
                        duration_steps: dur,
                        velocity: 90,
                        articulation: Articulation::Normal,
                    },
                );
            };
        add(&mut measure, TrackId::Melody, 1, 60, 0, 2);
        add(&mut measure, TrackId::Chords, 2, 64, 4, 1);
        add(&mut measure, TrackId::Bass, 3, 36, 0, 4);
        add(&mut measure, TrackId::Heel, 4, 38, 6, 0);
        // Spills past the barline (14 + 4 = step 2 of bar 2).
        add(&mut measure, TrackId::Melody, 5, 62, 14, 4);
        ph.load_measure(Box::new(measure));

        let mut log: Vec<(usize, bool, u8, u8)> = Vec::new();
        for step in 0..16 {
            for e in ph.tick() {
                if let Some((on, ch, pitch)) = note(e) {
                    log.push((step, on, ch, pitch));
                }
            }
        }
        assert!(log.contains(&(0, true, 0, 60)), "melody on: {log:?}");
        assert!(log.contains(&(2, false, 0, 60)), "melody off at start+dur: {log:?}");
        assert!(log.contains(&(4, true, 1, 64)), "chords on: {log:?}");
        assert!(log.contains(&(5, false, 1, 64)), "chords off: {log:?}");
        assert!(log.contains(&(0, true, 2, 36)), "bass on: {log:?}");
        assert!(log.contains(&(4, false, 2, 36)), "bass off: {log:?}");
        assert!(log.contains(&(6, true, 3, 38)), "heel on: {log:?}");
        assert!(log.contains(&(7, false, 3, 38)), "trigger off one step later: {log:?}");

        // Cross-barline end fires in the next measure.
        let measure2 = Measure::new(2, TimeSignature::default(), 120.0, 16);
        ph.take_finished();
        ph.load_measure(Box::new(measure2));
        let mut log2: Vec<(usize, bool, u8, u8)> = Vec::new();
        for step in 0..4 {
            for e in ph.tick() {
                if let Some((on, ch, pitch)) = note(e) {
                    log2.push((step, on, ch, pitch));
                }
            }
        }
        assert!(log2.contains(&(2, false, 0, 62)), "spilled off in bar 2: {log2:?}");
        // Every NoteOn pairs with exactly one NoteOff.
        let ons = log.iter().filter(|e| e.1).count() + log2.iter().filter(|e| e.1).count();
        let offs = log.iter().filter(|e| !e.1).count() + log2.iter().filter(|e| !e.1).count();
        assert_eq!(ons, 5, "ons: {log:?} {log2:?}");
        assert_eq!(offs, 5, "offs: {log:?} {log2:?}");
    }
}

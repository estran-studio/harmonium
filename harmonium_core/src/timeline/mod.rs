//! Timeline-based score architecture for Harmonium
//!
//! This module provides a seekable, replayable score representation where:
//! - A **Writehead** (main thread) generates `Measure` structs ahead of playback
//! - A **Playhead** (audio thread) reads measures and emits `AudioEvent`s
//! - A **ScoreTimeline** stores the master copy with bounded sliding window
//!
//! The algorithms (Sequencer, HarmonicDriver, HarmonyNavigator, Voicer) stay
//! identical to the legacy engine - only separated into composer and performer roles.

pub mod export;
pub mod generator;
pub mod midi_export;
mod pointers;

use std::collections::HashMap;

pub use export::{timeline_to_musicxml, timeline_to_musicxml_with_instruments};
pub use generator::TimelineGenerator;
pub use midi_export::{timeline_to_midi, write_midi};
pub use pointers::{Playhead, Writehead};
use rust_music_theory::{note::PitchSymbol, scale::ScaleType};
use serde::{Deserialize, Serialize};

use crate::params::{CurrentState, TimeSignature};

/// Context needed to deterministically reconstruct a generation session from scratch.
///
/// Stores the master seed and the derived key/scale that were randomly chosen
/// at session start. Used by `TimelineGenerator::reset_to_initial()` to replay
/// from bar 1 for deterministic seek.
#[derive(Clone, Debug)]
pub struct GenerationContext {
    pub session_seed: u64,
    pub key: PitchSymbol,
    pub scale: ScaleType,
    pub key_pc: u8,
}

/// Unique identifier for each note in the timeline
pub type NoteId = u64;

/// Track identifier for per-instrument separation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrackId {
    /// Bass / Kick (MIDI channel 0)
    Bass,
    /// Lead / Melody (MIDI channel 1)
    Lead,
    /// Snare / Percussion (MIDI channel 2)
    Snare,
    /// Hi-Hat / Cymbals (MIDI channel 3)
    Hat,
}

impl TrackId {
    /// Convert to MIDI channel number
    #[must_use]
    pub const fn channel(&self) -> u8 {
        match self {
            Self::Bass => 0,
            Self::Lead => 1,
            Self::Snare => 2,
            Self::Hat => 3,
        }
    }

    /// All track IDs in canonical order
    pub const ALL: [TrackId; 4] = [Self::Bass, Self::Lead, Self::Snare, Self::Hat];
}

/// Musical position within the score (bar, beat, tick)
///
/// Uses 1-based bar/beat numbering (musical convention).
/// Ticks are 0-based subdivisions within a beat.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MusicalPosition {
    /// Bar number (1-based)
    pub bar: usize,
    /// Beat within bar (1-based, 1..=time_sig.numerator)
    pub beat: usize,
    /// Tick within beat (0-based, 0..ticks_per_beat)
    pub tick: usize,
}

impl MusicalPosition {
    /// Create a new position
    #[must_use]
    pub const fn new(bar: usize, beat: usize, tick: usize) -> Self {
        Self { bar, beat, tick }
    }

    /// Convert to an absolute step index within a measure (0-based)
    /// For 4/4 time with 4 ticks/beat: beat 1 tick 0 = step 0, beat 2 tick 0 = step 4, etc.
    #[must_use]
    pub const fn step_in_bar(&self, ticks_per_beat: usize) -> usize {
        (self.beat.saturating_sub(1)) * ticks_per_beat + self.tick
    }

    /// Total steps from bar start to this position
    #[must_use]
    pub fn total_steps_from_bar_start(&self, ticks_per_beat: usize) -> usize {
        self.step_in_bar(ticks_per_beat)
    }
}

impl std::fmt::Display for MusicalPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}.{}", self.bar, self.beat, self.tick)
    }
}

/// Articulation hint for the playhead
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Articulation {
    /// Normal sustain (play for full duration)
    Normal,
    /// Staccato (short, detached - bass uses this)
    Staccato,
    /// Trigger only, no NoteOff needed (percussion)
    Trigger,
}

impl Default for Articulation {
    fn default() -> Self {
        Self::Normal
    }
}

/// A note with explicit duration, stored in the timeline.
///
/// Unlike ephemeral NoteOn/NoteOff events, this captures full intent:
/// pitch, start position, duration in ticks, velocity, and articulation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineNote {
    /// Unique identifier (monotonically increasing)
    pub id: NoteId,
    /// MIDI note number (0-127)
    pub pitch: u8,
    /// Start position within the measure (step index, 0-based)
    pub start_step: usize,
    /// Duration in steps (0 = trigger-only for percussion)
    pub duration_steps: usize,
    /// MIDI velocity (0-127)
    pub velocity: u8,
    /// Articulation hint for the playhead
    pub articulation: Articulation,
}

/// Snapshot of the engine's morphed state at the time a measure was generated.
///
/// Captures the `CurrentState` values that influenced generation decisions,
/// allowing the Playhead to optionally apply velocity/articulation adjustments.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub bpm: f32,
    pub density: f32,
    pub tension: f32,
    pub smoothness: f32,
    pub valence: f32,
    pub arousal: f32,
}

impl From<&CurrentState> for StateSnapshot {
    fn from(state: &CurrentState) -> Self {
        Self {
            bpm: state.bpm,
            density: state.density,
            tension: state.tension,
            smoothness: state.smoothness,
            valence: state.valence,
            arousal: state.arousal,
        }
    }
}

/// Chord context captured at generation time
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChordContext {
    /// Root offset from key (semitones)
    pub root_offset: i32,
    /// Whether the chord is minor
    pub is_minor: bool,
    /// Chord name for display
    pub chord_name: String,
    /// Improv scale guidance (LCC-derived) for this measure's chord.
    /// `None` for measures generated before this field existed or when the
    /// chord/key context is unavailable.
    #[serde(default)]
    pub scale_guidance: Option<crate::harmony::ScaleGuidance>,
}

/// A complete measure of music across all tracks.
///
/// This is the atomic unit of the timeline - generated by the Writehead,
/// transferred via ring buffer to the Playhead, and stored in the ScoreTimeline.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Measure {
    /// Absolute measure index (1-based)
    pub index: usize,
    /// Time signature for this measure
    pub time_signature: TimeSignature,
    /// Tempo (BPM) at generation time
    pub tempo: f32,
    /// Number of steps in this measure (time_sig.steps_per_bar(ticks_per_beat))
    pub steps: usize,
    /// Chord context for this measure
    pub chord_context: ChordContext,
    /// State snapshot at generation time
    pub state_snapshot: StateSnapshot,
    /// Notes per track
    pub tracks: HashMap<TrackId, Vec<TimelineNote>>,
}

impl Measure {
    /// Create an empty measure with the given parameters
    #[must_use]
    pub fn new(index: usize, time_signature: TimeSignature, tempo: f32, steps: usize) -> Self {
        let mut tracks = HashMap::with_capacity(4);
        for &track_id in &TrackId::ALL {
            tracks.insert(track_id, Vec::new());
        }

        Self {
            index,
            time_signature,
            tempo,
            steps,
            chord_context: ChordContext::default(),
            state_snapshot: StateSnapshot::default(),
            tracks,
        }
    }

    /// Add a note to a specific track
    pub fn add_note(&mut self, track: TrackId, note: TimelineNote) {
        self.tracks.entry(track).or_default().push(note);
    }

    /// Get all notes for a track, sorted by start_step
    #[must_use]
    pub fn notes_for_track(&self, track: TrackId) -> &[TimelineNote] {
        self.tracks.get(&track).map_or(&[], Vec::as_slice)
    }

    /// Total number of notes across all tracks
    #[must_use]
    pub fn total_notes(&self) -> usize {
        self.tracks.values().map(Vec::len).sum()
    }
}

/// The master score timeline - a bounded sliding window of measures.
///
/// Stores up to `max_measures` measures. When the limit is reached,
/// the oldest measures are dropped (sliding window).
pub struct ScoreTimeline {
    /// Stored measures (sorted by index)
    measures: Vec<Measure>,
    /// Maximum number of measures to retain
    max_measures: usize,
    /// Next note ID to assign (monotonically increasing)
    next_note_id: NoteId,
}

impl ScoreTimeline {
    /// Create a new timeline with the given capacity
    #[must_use]
    pub fn new(max_measures: usize) -> Self {
        Self { measures: Vec::with_capacity(max_measures), max_measures, next_note_id: 1 }
    }

    /// Default capacity (100 bars)
    #[must_use]
    pub fn with_default_capacity() -> Self {
        Self::new(100)
    }

    /// Allocate the next unique note ID
    pub fn next_note_id(&mut self) -> NoteId {
        let id = self.next_note_id;
        self.next_note_id += 1;
        id
    }

    /// Append a measure to the timeline
    ///
    /// If the timeline is at capacity, the oldest measure is dropped.
    pub fn push_measure(&mut self, measure: Measure) {
        if self.measures.len() >= self.max_measures {
            self.measures.remove(0);
        }
        self.measures.push(measure);
    }

    /// Get a measure by its absolute index
    #[must_use]
    pub fn get_measure(&self, index: usize) -> Option<&Measure> {
        self.measures.iter().find(|m| m.index == index)
    }

    /// Get a mutable reference to a measure by its absolute index
    pub fn get_measure_mut(&mut self, index: usize) -> Option<&mut Measure> {
        self.measures.iter_mut().find(|m| m.index == index)
    }

    /// The index of the first (oldest) measure in the timeline
    #[must_use]
    pub fn first_index(&self) -> Option<usize> {
        self.measures.first().map(|m| m.index)
    }

    /// The index of the last (newest) measure in the timeline
    #[must_use]
    pub fn last_index(&self) -> Option<usize> {
        self.measures.last().map(|m| m.index)
    }

    /// Number of measures currently stored
    #[must_use]
    pub fn len(&self) -> usize {
        self.measures.len()
    }

    /// Whether the timeline is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.measures.is_empty()
    }

    /// Clear all measures
    pub fn clear(&mut self) {
        self.measures.clear();
    }

    /// Get all measures as a slice
    #[must_use]
    pub fn measures(&self) -> &[Measure] {
        &self.measures
    }

    /// Check if a measure index is within the timeline window
    #[must_use]
    pub fn contains_index(&self, index: usize) -> bool {
        self.measures.iter().any(|m| m.index == index)
    }

    /// Invalidate (remove) all measures from the given index onward.
    /// Used when parameters change and future measures need regeneration.
    pub fn invalidate_from(&mut self, from_index: usize) {
        self.measures.retain(|m| m.index < from_index);
    }
}

/// Tempo map for converting between musical time and sample time
#[derive(Clone, Debug)]
pub struct TempoMap {
    /// Sample rate (e.g., 44100.0)
    pub sample_rate: f64,
    /// Ticks per beat (resolution, typically 4 for 16th notes)
    pub ticks_per_beat: usize,
}

impl TempoMap {
    /// Create a new tempo map
    #[must_use]
    pub const fn new(sample_rate: f64, ticks_per_beat: usize) -> Self {
        Self { sample_rate, ticks_per_beat }
    }

    /// Calculate samples per step at a given BPM
    #[must_use]
    pub fn samples_per_step(&self, bpm: f32) -> usize {
        let steps_per_beat = self.ticks_per_beat as f64;
        (self.sample_rate * 60.0 / (bpm as f64) / steps_per_beat) as usize
    }

    /// Calculate steps per bar for a given time signature
    #[must_use]
    pub fn steps_per_bar(&self, time_sig: TimeSignature) -> usize {
        time_sig.steps_per_bar(self.ticks_per_beat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_musical_position_ordering() {
        let a = MusicalPosition::new(1, 1, 0);
        let b = MusicalPosition::new(1, 2, 0);
        let c = MusicalPosition::new(2, 1, 0);
        assert!(a < b);
        assert!(b < c);
    }

    #[test]
    fn test_musical_position_step_in_bar() {
        let pos = MusicalPosition::new(1, 2, 1);
        // Beat 2, tick 1, with 4 ticks/beat = step 5
        assert_eq!(pos.step_in_bar(4), 5);
    }

    #[test]
    fn test_musical_position_display() {
        let pos = MusicalPosition::new(3, 2, 1);
        assert_eq!(format!("{pos}"), "3:2.1");
    }

    #[test]
    fn test_track_id_channel() {
        assert_eq!(TrackId::Bass.channel(), 0);
        assert_eq!(TrackId::Lead.channel(), 1);
        assert_eq!(TrackId::Snare.channel(), 2);
        assert_eq!(TrackId::Hat.channel(), 3);
    }

    #[test]
    fn test_measure_creation() {
        let measure = Measure::new(1, TimeSignature::default(), 120.0, 16);
        assert_eq!(measure.index, 1);
        assert_eq!(measure.steps, 16);
        assert_eq!(measure.total_notes(), 0);
        assert_eq!(measure.tracks.len(), 4);
    }

    #[test]
    fn test_measure_add_note() {
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
        assert_eq!(measure.total_notes(), 1);
        assert_eq!(measure.notes_for_track(TrackId::Bass).len(), 1);
        assert_eq!(measure.notes_for_track(TrackId::Lead).len(), 0);
    }

    #[test]
    fn test_score_timeline_push_and_get() {
        let mut timeline = ScoreTimeline::new(10);
        let measure = Measure::new(1, TimeSignature::default(), 120.0, 16);
        timeline.push_measure(measure);

        assert_eq!(timeline.len(), 1);
        assert!(timeline.get_measure(1).is_some());
        assert!(timeline.get_measure(2).is_none());
    }

    #[test]
    fn test_score_timeline_sliding_window() {
        let mut timeline = ScoreTimeline::new(3);

        for i in 1..=5 {
            timeline.push_measure(Measure::new(i, TimeSignature::default(), 120.0, 16));
        }

        // Only last 3 should remain
        assert_eq!(timeline.len(), 3);
        assert_eq!(timeline.first_index(), Some(3));
        assert_eq!(timeline.last_index(), Some(5));
        assert!(timeline.get_measure(1).is_none()); // Evicted
        assert!(timeline.get_measure(3).is_some());
    }

    #[test]
    fn test_score_timeline_invalidate() {
        let mut timeline = ScoreTimeline::new(10);
        for i in 1..=5 {
            timeline.push_measure(Measure::new(i, TimeSignature::default(), 120.0, 16));
        }

        timeline.invalidate_from(3);
        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline.last_index(), Some(2));
    }

    #[test]
    fn test_tempo_map_samples_per_step() {
        let map = TempoMap::new(44100.0, 4);
        // At 120 BPM, 4 ticks/beat:
        // beat = 0.5s, step = 0.125s, samples = 44100 * 0.125 = 5512.5
        let sps = map.samples_per_step(120.0);
        assert_eq!(sps, 5512);
    }

    #[test]
    fn test_tempo_map_steps_per_bar() {
        let map = TempoMap::new(44100.0, 4);
        assert_eq!(map.steps_per_bar(TimeSignature::new(4, 4)), 16);
        assert_eq!(map.steps_per_bar(TimeSignature::new(3, 4)), 12);
        assert_eq!(map.steps_per_bar(TimeSignature::new(5, 4)), 20);
    }

    #[test]
    fn test_note_id_monotonic() {
        let mut timeline = ScoreTimeline::new(10);
        let id1 = timeline.next_note_id();
        let id2 = timeline.next_note_id();
        let id3 = timeline.next_note_id();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_state_snapshot_from_current_state() {
        let state = CurrentState {
            bpm: 120.0,
            density: 0.5,
            tension: 0.3,
            smoothness: 0.7,
            valence: 0.2,
            arousal: 0.6,
        };
        let snapshot = StateSnapshot::from(&state);
        assert!((snapshot.bpm - 120.0).abs() < f32::EPSILON);
        assert!((snapshot.tension - 0.3).abs() < f32::EPSILON);
    }
}

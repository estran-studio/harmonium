//! Unified report from engine to UI
//!
//! EngineReport replaces HarmonyState + VisualizationEvent with a single
//! unified structure. Uses fixed-size arrays to avoid allocations in the audio thread.

use arrayvec::ArrayString;
use serde::{Deserialize, Serialize};

use crate::params::{MusicalParams, PulseSpec, TimeSignature};

/// Lightweight snapshot of a note for score rendering (e.g., VexFlow).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoteSnapshot {
    /// Track: 0=Melody, 1=Chords, 2=Bass, 3=Heel, 4=Toe
    pub track: u8,
    /// MIDI note number (0-127)
    pub pitch: u8,
    /// Start position within the measure (step index, 0-based)
    pub start_step: usize,
    /// Duration in steps (0 = trigger-only for percussion)
    pub duration_steps: usize,
    /// MIDI velocity (0-127)
    pub velocity: u8,
}

/// Lightweight snapshot of a generated measure for score rendering.
///
/// Pushed to the frontend whenever the Writehead generates a new bar,
/// so a score renderer (e.g., VexFlow) can display upcoming music
/// before it plays.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeasureSnapshot {
    /// Absolute measure index (1-based)
    pub index: usize,
    /// Tempo at generation time
    pub tempo: f32,
    /// Time signature numerator
    pub time_sig_numerator: usize,
    /// Time signature denominator
    pub time_sig_denominator: usize,
    /// Number of steps in this measure
    pub steps: usize,
    /// Chord name for display (e.g., "Imaj7", "iv")
    pub chord_name: String,
    /// Root offset from key (semitones)
    pub chord_root_offset: i32,
    /// Whether the chord is minor
    pub chord_is_minor: bool,
    /// All notes in this measure (flattened across tracks)
    pub notes: Vec<NoteSnapshot>,
    /// Emotion-mapped BPM (composition tempo, ignoring user override).
    /// Used for tempo markings on the score.
    pub composition_bpm: f32,
    /// Improv scale guidance (LCC-derived) for this measure's chord — the
    /// suggested scale + chord tones the improv-coach UI displays. `None` when
    /// the chord/key context was unavailable at generation time.
    #[serde(default)]
    pub scale_guidance: Option<crate::theory::lydian_chromatic::ScaleGuidance>,
    /// Changements harmoniques aux pas exacts ; vide pour les anciens modèles.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chord_changes: Vec<crate::timeline::ChordChange>,
    /// Swing ratio captured at generation time (0.5 = straight). Ignored in
    /// compound meter.
    #[serde(default = "crate::params::default_swing_ratio")]
    pub swing_ratio: f32,
    /// Dance pulse captured at generation time.
    #[serde(default)]
    pub pulse: PulseSpec,
    /// Position dans l'air (tour, partie A/B, passe 1/2, mesure de la passe) pour
    /// caler le gameplay sur « les pieds entrent » ou « dernier tour ».
    /// `None` hors style trad.
    #[serde(default)]
    pub position: Option<crate::trad::TunePosition>,
}

impl MeasureSnapshot {
    /// Create a snapshot from a full Measure (used by the engine)
    pub fn from_measure(measure: &crate::timeline::Measure) -> Self {
        use crate::timeline::TrackId;

        let mut notes = Vec::new();
        for &track_id in &TrackId::ALL {
            let channel = track_id.channel();
            for note in measure.notes_for_track(track_id) {
                notes.push(NoteSnapshot {
                    track: channel,
                    pitch: note.pitch,
                    start_step: note.start_step,
                    duration_steps: note.duration_steps,
                    velocity: note.velocity,
                });
            }
        }

        Self {
            index: measure.index,
            tempo: measure.tempo,
            time_sig_numerator: measure.time_signature.numerator,
            time_sig_denominator: measure.time_signature.denominator,
            steps: measure.steps,
            chord_name: measure.chord_context.chord_name.clone(),
            chord_root_offset: measure.chord_context.root_offset,
            chord_is_minor: measure.chord_context.is_minor,
            notes,
            // Default to measure tempo; overridden by composer with emotion-mapped BPM
            composition_bpm: measure.tempo,
            scale_guidance: measure.chord_context.scale_guidance.clone(),
            chord_changes: measure.chord_changes.clone(),
            swing_ratio: measure.swing_ratio,
            pulse: measure.pulse,
            position: measure.position,
        }
    }
}

/// Note event for real-time visualization
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoteEvent {
    /// MIDI note number
    pub note_midi: u8,

    /// MIDI velocity (0-127)
    pub velocity: u8,

    /// MIDI channel (0=Melody, 1=Chords, 2=Bass, 3=Heel, 4=Toe)
    pub channel: u8,

    /// true = NoteOn, false = NoteOff
    pub is_note_on: bool,
}

/// Unified report from engine to UI
/// Replaces HarmonyState + VisualizationEvent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineReport {
    // === TIMING ===
    /// Current bar number (1-based)
    pub current_bar: usize,

    /// Current beat in the bar (0-based)
    pub current_beat: usize,

    /// Current step in the sequencer (0-based)
    pub current_step: usize,

    /// Current time signature
    pub time_signature: TimeSignature,

    // === HARMONY STATE ===
    /// Current chord name (e.g., "Imaj7", "iv")
    pub current_chord: ArrayString<64>,

    /// Root offset from key (semitones)
    pub chord_root_offset: i32,

    /// Whether current chord is minor
    pub chord_is_minor: bool,

    /// Progression name (e.g., "Hopeful", "Dark")
    pub progression_name: ArrayString<64>,

    /// Length of current progression
    pub progression_length: usize,

    // === NOTES TRIGGERED (this tick) ===
    /// Notes triggered since the last report (pre-allocated capacity)
    pub notes: Vec<NoteEvent>,

    /// Sample offset within the audio buffer where these notes were triggered.
    /// Used by the VST plugin for sample-accurate MIDI timing.
    pub sample_offset: u32,

    // === NEW MEASURES (pushed when Writehead generates) ===
    /// Newly generated measures since the last report.
    /// Frontend should append these to its score cache for rendering.
    pub new_measures: Vec<MeasureSnapshot>,

    // === CURRENT PARAMS (echoed back) ===
    /// Current musical parameters
    pub musical_params: MusicalParams,

    // === SESSION INFO ===
    /// Session key (e.g., "C")
    pub session_key: ArrayString<8>,

    /// Session scale (e.g., "major", "minor")
    pub session_scale: ArrayString<32>,
}

impl Default for EngineReport {
    fn default() -> Self {
        Self {
            current_bar: 1,
            current_beat: 0,
            current_step: 0,
            time_signature: TimeSignature::default(),
            current_chord: ArrayString::new(),
            chord_root_offset: 0,
            chord_is_minor: false,
            progression_name: ArrayString::new(),
            progression_length: 0,
            notes: Vec::with_capacity(16), // Pre-allocated
            sample_offset: 0,
            new_measures: Vec::new(),
            musical_params: MusicalParams::default(),
            session_key: ArrayString::new(),
            session_scale: ArrayString::new(),
        }
    }
}

impl EngineReport {
    /// Create a new report with pre-allocated note buffer
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear notes vector for reuse (doesn't deallocate)
    pub fn clear_notes(&mut self) {
        self.notes.clear();
    }

    /// Add a note event (reuses pre-allocated capacity)
    pub fn add_note(&mut self, note_midi: u8, velocity: u8, channel: u8, is_note_on: bool) {
        self.notes.push(NoteEvent { note_midi, velocity, channel, is_note_on });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_default() {
        let report = EngineReport::default();
        assert_eq!(report.current_bar, 1);
        assert_eq!(report.notes.capacity(), 16);
    }

    #[test]
    fn test_report_serde() {
        let report = EngineReport::default();
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: EngineReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.current_bar, report.current_bar);
    }

    #[test]
    fn test_add_note() {
        let mut report = EngineReport::default();
        report.add_note(60, 100, 0, true);
        assert_eq!(report.notes.len(), 1);
        assert_eq!(report.notes[0].note_midi, 60);
        assert_eq!(report.notes[0].velocity, 100);
        assert_eq!(report.notes[0].channel, 0);
        assert!(report.notes[0].is_note_on);
    }

    #[test]
    fn test_clear_notes() {
        let mut report = EngineReport::default();
        report.add_note(60, 100, 0, true);
        report.add_note(64, 100, 0, true);
        assert_eq!(report.notes.len(), 2);

        report.clear_notes();
        assert_eq!(report.notes.len(), 0);
        assert_eq!(report.notes.capacity(), 16); // Capacity preserved
    }
}

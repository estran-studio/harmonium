//! Musical DNA - Intermediate Representation for Algorithm Tuning
//!
//! This module defines the "Musical DNA" structure which captures the geometric
//! and topological properties of a musical piece. The DNA serves as an
//! intermediate representation (IR) for:
//!
//! - Comparing generated music against reference corpora
//! - Enabling LLM-assisted algorithm tuning
//! - Analyzing harmonic and rhythmic characteristics
//!
//! Based on Dmitri Tymoczko's Geometrical Music Theory.

use harmonium_core::{
    events::AudioEvent,
    harmony::parsimonious::TRQ,
    params::MusicalParams,
    sequencer::RhythmMode,
    theory::{PitchClass, chord::Chord},
};
use serde::{Deserialize, Serialize};

/// The "Ground Truth" of a recording session (moved from harmonium_core::truth).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordingTruth {
    /// Version of the engine used for this recording
    pub version: String,
    /// Git SHA of the engine
    pub git_sha: String,
    /// The musical parameters active during the recording
    pub params: MusicalParams,
    /// Sequential history of all musical events with step-based timestamps
    pub events: Vec<(f64, AudioEvent)>,
    /// Sample rate used during the recording
    pub sample_rate: u32,
}

impl RecordingTruth {
    /// Create a new recording truth from events and parameters
    pub fn new(events: Vec<(f64, AudioEvent)>, params: MusicalParams, sample_rate: u32) -> Self {
        Self { version: String::new(), git_sha: String::new(), params, events, sample_rate }
    }
}

/// Complete Musical DNA capturing harmonic and rhythmic characteristics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MusicalDNA {
    /// Original recording truth (parameters + events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truth: Option<RecordingTruth>,

    /// Sequence of harmonic frames (chord-by-chord analysis)
    pub harmonic_profile: Vec<HarmonicFrame>,

    /// Rhythmic DNA (pattern characteristics)
    pub rhythmic_profile: RhythmicDNA,

    /// Global metrics (aggregated statistics)
    pub global_metrics: GlobalMetrics,
}

impl MusicalDNA {
    /// Create a new MusicalDNA with default empty values
    #[must_use]
    pub fn new() -> Self {
        Self {
            truth: None,
            harmonic_profile: Vec::new(),
            rhythmic_profile: RhythmicDNA::default(),
            global_metrics: GlobalMetrics::default(),
        }
    }

    /// Create MusicalDNA from a RecordingTruth
    #[must_use]
    pub fn from_truth(truth: RecordingTruth) -> Self {
        Self {
            truth: Some(truth),
            harmonic_profile: Vec::new(),
            rhythmic_profile: RhythmicDNA::default(),
            global_metrics: GlobalMetrics::default(),
        }
    }
}

impl Default for MusicalDNA {
    fn default() -> Self {
        Self::new()
    }
}

/// A single frame in the harmonic profile (one chord transition)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HarmonicFrame {
    /// Timestamp in beats (or seconds)
    pub timestamp: f64,

    /// Duration of this chord in beats
    pub duration: f64,

    /// Chord name (e.g., "Cmaj7", "Dm7")
    pub chord: String,

    /// Pitch class set (0-11 values)
    pub pitch_class_set: Vec<PitchClass>,

    /// Tension/Release Quotient for this transition
    pub trq: SerializableTRQ,

    /// Voice leading distance (L1 norm) from previous chord
    pub voice_leading_distance: u32,

    /// Lydian Chromatic Concept level (1-12)
    pub lcc_level: u8,
}

impl Default for HarmonicFrame {
    fn default() -> Self {
        Self {
            timestamp: 0.0,
            duration: 1.0,
            chord: "C".to_string(),
            pitch_class_set: vec![0, 4, 7],
            trq: SerializableTRQ::default(),
            voice_leading_distance: 0,
            lcc_level: 1,
        }
    }
}

/// Serializable version of TRQ (Tension/Release Quotient)
///
/// This wraps the core TRQ struct to add serialization support
/// without modifying the original parsimonious.rs module.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SerializableTRQ {
    /// Tension component (0.0 - 1.0): dissonance, instability
    pub tension: f32,
    /// Release component (0.0 - 1.0): tendency toward resolution
    pub release: f32,
}

impl SerializableTRQ {
    /// Create from core TRQ
    #[must_use]
    pub fn from_trq(trq: TRQ) -> Self {
        Self { tension: trq.tension, release: trq.release }
    }

    /// Convert to core TRQ
    #[must_use]
    pub fn to_trq(self) -> TRQ {
        TRQ::new(self.tension, self.release)
    }

    /// Net tension (positive = tense, negative = relaxed)
    #[must_use]
    pub fn net(&self) -> f32 {
        self.tension - self.release
    }
}

impl Default for SerializableTRQ {
    fn default() -> Self {
        Self { tension: 0.5, release: 0.5 }
    }
}

impl From<TRQ> for SerializableTRQ {
    fn from(trq: TRQ) -> Self {
        Self::from_trq(trq)
    }
}

/// Rhythmic DNA capturing pattern characteristics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RhythmicDNA {
    /// Rhythm generation mode
    pub mode: String,

    /// Extracted polygon signatures (kick, snare, hat patterns)
    pub polygons: Vec<PolygonSignature>,

    /// Syncopation score (0.0 = on-beat, 1.0 = highly syncopated)
    pub syncopation_score: f32,

    /// Density curve per measure (values 0.0 - 1.0)
    pub density_curve: Vec<f32>,

    /// Average micro-timing deviation (swing/rubato)
    pub micro_timing_deviation: f32,
}

impl Default for RhythmicDNA {
    fn default() -> Self {
        Self {
            mode: "Euclidean".to_string(),
            polygons: Vec::new(),
            syncopation_score: 0.0,
            density_curve: Vec::new(),
            micro_timing_deviation: 0.0,
        }
    }
}

impl RhythmicDNA {
    /// Create from a RhythmMode
    #[must_use]
    pub fn from_mode(mode: RhythmMode) -> Self {
        let mode_str = match mode {
            RhythmMode::Euclidean => "Euclidean",
            RhythmMode::PerfectBalance => "PerfectBalance",
            RhythmMode::ClassicGroove => "ClassicGroove",
        };
        Self { mode: mode_str.to_string(), ..Default::default() }
    }
}

/// Signature of a rhythmic polygon (from Perfect Balance algorithm)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolygonSignature {
    /// Layer name (e.g., "kick", "snare", "hat", "bass", "lead")
    pub layer: String,

    /// Number of vertices in the polygon
    pub vertices: usize,

    /// Rotation offset in steps
    pub rotation_offset: usize,

    /// Inter-onset intervals (IOIs) between hits
    pub interval_vector: Vec<usize>,

    /// Velocity/amplitude of this layer
    pub velocity: f32,
}

impl PolygonSignature {
    /// Create a new polygon signature
    #[must_use]
    pub fn new(layer: impl Into<String>, vertices: usize, rotation_offset: usize) -> Self {
        Self {
            layer: layer.into(),
            vertices,
            rotation_offset,
            interval_vector: Vec::new(),
            velocity: 1.0,
        }
    }
}

/// Global metrics aggregated from the entire piece
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalMetrics {
    /// Average voice leading effort (L1 distance per transition)
    pub average_voice_leading_effort: f32,

    /// Variance in tension across the piece
    pub tension_variance: f32,

    /// Average TRQ.net() - balance between tension and release
    pub tension_release_balance: f32,

    /// Percentage of notes in diatonic scale (0.0 - 100.0)
    pub diatonic_percentage: f32,

    /// Harmonic rhythm (average chords per measure)
    pub harmonic_rhythm: f32,

    /// Total duration in beats
    pub total_duration_beats: f32,

    /// Number of chord changes
    pub chord_change_count: usize,
}

impl Default for GlobalMetrics {
    fn default() -> Self {
        Self {
            average_voice_leading_effort: 0.0,
            tension_variance: 0.0,
            tension_release_balance: 0.0,
            diatonic_percentage: 100.0,
            harmonic_rhythm: 1.0,
            total_duration_beats: 0.0,
            chord_change_count: 0,
        }
    }
}

impl GlobalMetrics {
    /// Calculate metrics from a sequence of harmonic frames
    /// Uses duration-weighted averaging to prioritize longer (more harmonically significant) chords
    #[must_use]
    pub fn from_frames(frames: &[HarmonicFrame]) -> Self {
        if frames.is_empty() {
            return Self::default();
        }

        let n = frames.len() as f32;

        // Total duration for weighting
        let total_duration_beats: f32 = frames.iter().map(|f| f.duration as f32).sum();

        // Use duration-weighted averaging for more accurate metrics
        // This reduces the impact of short passing tones and emphasizes structural harmonies
        let (average_voice_leading_effort, mean_tension, tension_release_balance) =
            if total_duration_beats > 0.0 {
                // Duration-weighted voice leading effort
                let weighted_vl: f32 = frames
                    .iter()
                    .map(|f| f.voice_leading_distance as f32 * f.duration as f32)
                    .sum();
                let avg_vl = weighted_vl / total_duration_beats;

                // Duration-weighted mean tension
                let weighted_tension: f32 =
                    frames.iter().map(|f| f.trq.tension * f.duration as f32).sum();
                let mean_t = weighted_tension / total_duration_beats;

                // Duration-weighted tension/release balance
                let weighted_balance: f32 =
                    frames.iter().map(|f| f.trq.net() * f.duration as f32).sum();
                let balance = weighted_balance / total_duration_beats;

                (avg_vl, mean_t, balance)
            } else {
                // Fallback to simple averaging
                let total_vl: u32 = frames.iter().map(|f| f.voice_leading_distance).sum();
                let avg_vl = total_vl as f32 / n;
                let mean_t: f32 = frames.iter().map(|f| f.trq.tension).sum::<f32>() / n;
                let balance: f32 = frames.iter().map(|f| f.trq.net()).sum::<f32>() / n;
                (avg_vl, mean_t, balance)
            };

        // Duration-weighted tension variance
        let tension_variance: f32 = if total_duration_beats > 0.0 {
            frames
                .iter()
                .map(|f| (f.trq.tension - mean_tension).powi(2) * f.duration as f32)
                .sum::<f32>()
                / total_duration_beats
        } else {
            let tensions: Vec<f32> = frames.iter().map(|f| f.trq.tension).collect();
            tensions.iter().map(|t| (t - mean_tension).powi(2)).sum::<f32>() / n
        };

        // Harmonic rhythm (assuming 4 beats per measure)
        let harmonic_rhythm =
            if total_duration_beats > 0.0 { n / (total_duration_beats / 4.0) } else { 1.0 };

        Self {
            average_voice_leading_effort,
            tension_variance,
            tension_release_balance,
            diatonic_percentage: 100.0, // TODO: Calculate from pitch class sets
            harmonic_rhythm,
            total_duration_beats,
            chord_change_count: frames.len(),
        }
    }
}

// ============================================================================
// DNA EXTRACTION
// ============================================================================

/// Channel assignments in Harmonium (`TrackId::channel`)
const CHANNEL_LEAD: u8 = harmonium_core::timeline::TrackId::Melody.channel();
const CHANNEL_BASS: u8 = harmonium_core::timeline::TrackId::Bass.channel();
const CHANNEL_SNARE: u8 = harmonium_core::timeline::TrackId::Heel.channel();
const CHANNEL_HAT: u8 = harmonium_core::timeline::TrackId::Toe.channel();

/// Time window (in beats) for grouping simultaneous notes into chords
const CHORD_GROUPING_WINDOW: f64 = 0.1;

/// DNA Extractor - extracts Musical DNA from RecordingTruth
///
/// This analyzer processes recorded events to build a complete
/// Musical DNA profile including harmonic and rhythmic characteristics.
#[derive(Clone, Debug)]
pub struct DNAExtractor {
    /// BPM for timing calculations
    bpm: f32,
    /// Steps per beat (typically 4 for 16th notes)
    steps_per_beat: usize,
}

impl Default for DNAExtractor {
    fn default() -> Self {
        Self::new(120.0, 4)
    }
}

impl DNAExtractor {
    /// Create a new DNA extractor
    #[must_use]
    pub const fn new(bpm: f32, steps_per_beat: usize) -> Self {
        Self { bpm, steps_per_beat }
    }

    /// Extract complete Musical DNA from a RecordingTruth
    #[must_use]
    pub fn extract(&self, truth: &RecordingTruth) -> MusicalDNA {
        let bpm = truth.params.bpm;
        let extractor = Self::new(bpm, 4);

        // Extract harmonic profile from lead channel
        let harmonic_profile = extractor.extract_harmonic_profile(&truth.events);

        // Extract rhythmic DNA from rhythm channels
        let rhythmic_profile = extractor.extract_rhythmic_dna(&truth.events, &truth.params);

        // Calculate global metrics
        let global_metrics = GlobalMetrics::from_frames(&harmonic_profile);

        MusicalDNA {
            truth: Some(truth.clone()),
            harmonic_profile,
            rhythmic_profile,
            global_metrics,
        }
    }

    /// Extract harmonic profile from events
    ///
    /// Groups notes by timestamp to detect chords, then analyzes
    /// transitions using TRQ and voice leading distance.
    fn extract_harmonic_profile(&self, events: &[(f64, AudioEvent)]) -> Vec<HarmonicFrame> {
        // Group notes by time window (chord detection)
        let chord_groups = self.group_notes_into_chords(events);

        if chord_groups.is_empty() {
            return Vec::new();
        }

        let mut frames = Vec::new();
        let mut prev_chord: Option<Chord> = None;

        for (i, (timestamp, pitch_classes)) in chord_groups.iter().enumerate() {
            // Try to identify the chord
            let chord = Chord::identify(pitch_classes);

            // Calculate duration (time until next chord or default)
            let duration = if i + 1 < chord_groups.len() {
                chord_groups[i + 1].0 - timestamp
            } else {
                4.0 // Default: 1 measure
            };

            // Calculate TRQ and voice leading distance
            let (trq, vl_distance) = if let (Some(prev), Some(curr)) = (&prev_chord, &chord) {
                let trq = TRQ::for_transition(prev, curr);
                let vl = prev.voice_leading_distance(curr);
                (SerializableTRQ::from_trq(trq), vl)
            } else {
                (SerializableTRQ::default(), 0)
            };

            let chord_name = chord
                .as_ref()
                .map_or_else(|| format!("Unknown({:?})", pitch_classes), |c| c.name());

            frames.push(HarmonicFrame {
                timestamp: *timestamp,
                duration,
                chord: chord_name,
                pitch_class_set: pitch_classes.clone(),
                trq,
                voice_leading_distance: vl_distance,
                lcc_level: 1, // TODO: Calculate from LCC context
            });

            prev_chord = chord;
        }

        frames
    }

    /// Group notes into chords based on temporal proximity
    ///
    /// Returns a list of (timestamp, pitch_classes) tuples
    fn group_notes_into_chords(&self, events: &[(f64, AudioEvent)]) -> Vec<(f64, Vec<PitchClass>)> {
        // Filter to lead channel NoteOn events only
        let lead_notes: Vec<(f64, u8)> = events
            .iter()
            .filter_map(|(ts, event)| {
                if let AudioEvent::NoteOn { note, channel, .. } = event {
                    if *channel == CHANNEL_LEAD {
                        return Some((*ts, *note));
                    }
                }
                None
            })
            .collect();

        if lead_notes.is_empty() {
            return Vec::new();
        }

        let mut groups: Vec<(f64, Vec<PitchClass>)> = Vec::new();
        let mut current_group_start = lead_notes[0].0;
        let mut current_pcs: Vec<PitchClass> = Vec::new();

        for (ts, note) in lead_notes {
            let pc = note % 12;

            if ts - current_group_start > CHORD_GROUPING_WINDOW {
                // Start new group
                if !current_pcs.is_empty() {
                    // Deduplicate and sort pitch classes
                    current_pcs.sort_unstable();
                    current_pcs.dedup();
                    groups.push((current_group_start, current_pcs));
                }
                current_group_start = ts;
                current_pcs = vec![pc];
            } else {
                // Add to current group
                if !current_pcs.contains(&pc) {
                    current_pcs.push(pc);
                }
            }
        }

        // Don't forget last group
        if !current_pcs.is_empty() {
            current_pcs.sort_unstable();
            current_pcs.dedup();
            groups.push((current_group_start, current_pcs));
        }

        groups
    }

    /// Extract rhythmic DNA from events
    fn extract_rhythmic_dna(
        &self,
        events: &[(f64, AudioEvent)],
        params: &harmonium_core::params::MusicalParams,
    ) -> RhythmicDNA {
        let mode = RhythmicDNA::from_mode(params.rhythm_mode);

        // Extract onset patterns per channel
        let kick_onsets = self.extract_channel_onsets(events, CHANNEL_BASS);
        let snare_onsets = self.extract_channel_onsets(events, CHANNEL_SNARE);
        let hat_onsets = self.extract_channel_onsets(events, CHANNEL_HAT);

        // Build polygon signatures
        let mut polygons = Vec::new();

        if !kick_onsets.is_empty() {
            polygons.push(self.build_polygon_signature("kick", &kick_onsets, events));
        }
        if !snare_onsets.is_empty() {
            polygons.push(self.build_polygon_signature("snare", &snare_onsets, events));
        }
        if !hat_onsets.is_empty() {
            polygons.push(self.build_polygon_signature("hat", &hat_onsets, events));
        }

        // Calculate syncopation score
        let syncopation_score = self.calculate_syncopation(&kick_onsets, &snare_onsets);

        // Calculate density curve (hits per measure)
        let density_curve = self.calculate_density_curve(events);

        // Calculate micro-timing deviation
        let micro_timing_deviation = self.calculate_micro_timing_deviation(&kick_onsets);

        RhythmicDNA {
            mode: mode.mode,
            polygons,
            syncopation_score,
            density_curve,
            micro_timing_deviation,
        }
    }

    /// Extract onset times for a specific channel
    fn extract_channel_onsets(&self, events: &[(f64, AudioEvent)], channel: u8) -> Vec<f64> {
        events
            .iter()
            .filter_map(|(ts, event)| {
                if let AudioEvent::NoteOn { channel: ch, .. } = event {
                    if *ch == channel {
                        return Some(*ts);
                    }
                }
                None
            })
            .collect()
    }

    /// Build a polygon signature from onset times
    fn build_polygon_signature(
        &self,
        layer: &str,
        onsets: &[f64],
        events: &[(f64, AudioEvent)],
    ) -> PolygonSignature {
        let vertices = onsets.len().min(16); // Cap at 16 vertices

        // Calculate inter-onset intervals (IOIs)
        let mut interval_vector: Vec<usize> = Vec::new();
        if onsets.len() >= 2 {
            let step_duration = 60.0 / self.bpm as f64 / self.steps_per_beat as f64;
            for i in 1..onsets.len().min(17) {
                let ioi = onsets[i] - onsets[i - 1];
                let steps = (ioi / step_duration).round() as usize;
                interval_vector.push(steps.max(1));
            }
        }

        // Calculate average velocity
        let velocities: Vec<u8> = events
            .iter()
            .filter_map(|(_, event)| {
                if let AudioEvent::NoteOn { velocity, .. } = event { Some(*velocity) } else { None }
            })
            .collect();

        let avg_velocity = if velocities.is_empty() {
            1.0
        } else {
            velocities.iter().map(|&v| v as f32).sum::<f32>() / velocities.len() as f32 / 127.0
        };

        // Estimate rotation offset (first onset position in step grid)
        let rotation_offset = if let Some(&first) = onsets.first() {
            let step_duration = 60.0 / self.bpm as f64 / self.steps_per_beat as f64;
            (first / step_duration).round() as usize % 16
        } else {
            0
        };

        PolygonSignature {
            layer: layer.to_string(),
            vertices,
            rotation_offset,
            interval_vector,
            velocity: avg_velocity,
        }
    }

    /// Calculate syncopation score based on off-beat hits
    ///
    /// Score ranges from 0.0 (all on-beat) to 1.0 (highly syncopated)
    fn calculate_syncopation(&self, kick_onsets: &[f64], snare_onsets: &[f64]) -> f32 {
        let step_duration = 60.0 / self.bpm as f64 / self.steps_per_beat as f64;
        let beat_duration = 60.0 / self.bpm as f64;

        let all_onsets: Vec<f64> = kick_onsets.iter().chain(snare_onsets.iter()).copied().collect();

        if all_onsets.is_empty() {
            return 0.0;
        }

        let mut off_beat_count = 0;
        for onset in &all_onsets {
            // Check if onset is on a strong beat (1 or 3 in 4/4)
            let beat_position = (onset / beat_duration) % 4.0;
            let is_strong_beat = beat_position < 0.1 || (beat_position - 2.0).abs() < 0.1;

            // Check if onset is on a beat at all
            let step_position = onset / step_duration;
            let is_on_beat = (step_position - step_position.round()).abs() < 0.1;

            if !is_strong_beat && is_on_beat {
                off_beat_count += 1;
            }
        }

        off_beat_count as f32 / all_onsets.len() as f32
    }

    /// Calculate density curve (hits per measure)
    fn calculate_density_curve(&self, events: &[(f64, AudioEvent)]) -> Vec<f32> {
        let measure_duration = 60.0 / self.bpm as f64 * 4.0; // 4 beats per measure

        // Find total duration
        let max_time = events.iter().map(|(ts, _)| *ts).fold(0.0f64, |a, b| a.max(b));

        let num_measures = ((max_time / measure_duration).ceil() as usize).max(1);
        let mut density_curve = vec![0.0f32; num_measures];

        // Count rhythm channel hits per measure
        for (ts, event) in events {
            if let AudioEvent::NoteOn { channel, .. } = event {
                if *channel == CHANNEL_BASS || *channel == CHANNEL_SNARE || *channel == CHANNEL_HAT
                {
                    let measure_idx = (ts / measure_duration) as usize;
                    if measure_idx < num_measures {
                        density_curve[measure_idx] += 1.0;
                    }
                }
            }
        }

        // Normalize to 0.0-1.0 range (assuming max ~32 hits per measure)
        for d in &mut density_curve {
            *d = (*d / 32.0).min(1.0);
        }

        density_curve
    }

    /// Calculate micro-timing deviation (swing/rubato)
    ///
    /// Measures how much onsets deviate from perfect grid alignment
    fn calculate_micro_timing_deviation(&self, onsets: &[f64]) -> f32 {
        if onsets.len() < 2 {
            return 0.0;
        }

        let step_duration = 60.0 / self.bpm as f64 / self.steps_per_beat as f64;

        let mut total_deviation = 0.0f64;
        for onset in onsets {
            let expected_step = (onset / step_duration).round();
            let expected_time = expected_step * step_duration;
            let deviation = (onset - expected_time).abs();
            total_deviation += deviation;
        }

        // Normalize: deviation as fraction of step duration
        let avg_deviation = total_deviation / onsets.len() as f64;
        (avg_deviation / step_duration).min(1.0) as f32
    }
}

impl MusicalDNA {
    /// Extract DNA from a RecordingTruth
    ///
    /// This is a convenience method that creates a DNAExtractor
    /// and extracts the full DNA profile.
    #[must_use]
    pub fn extract(truth: &RecordingTruth) -> Self {
        DNAExtractor::default().extract(truth)
    }

    /// Serialize DNA to JSON string
    ///
    /// # Errors
    /// Returns error if serialization fails
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize DNA to JSON string (compact)
    ///
    /// # Errors
    /// Returns error if serialization fails
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize DNA from JSON string
    ///
    /// # Errors
    /// Returns error if deserialization fails
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use harmonium_core::params::MusicalParams;

    use super::*;

    #[test]
    fn test_serializable_trq_roundtrip() {
        let trq = TRQ::new(0.7, 0.3);
        let serializable = SerializableTRQ::from_trq(trq);
        let back = serializable.to_trq();

        assert!((back.tension - 0.7).abs() < 0.001);
        assert!((back.release - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_global_metrics_calculation() {
        let frames = vec![
            HarmonicFrame {
                timestamp: 0.0,
                duration: 4.0,
                chord: "C".to_string(),
                pitch_class_set: vec![0, 4, 7],
                trq: SerializableTRQ { tension: 0.3, release: 0.5 },
                voice_leading_distance: 0,
                lcc_level: 1,
            },
            HarmonicFrame {
                timestamp: 4.0,
                duration: 4.0,
                chord: "Am".to_string(),
                pitch_class_set: vec![9, 0, 4],
                trq: SerializableTRQ { tension: 0.4, release: 0.6 },
                voice_leading_distance: 2,
                lcc_level: 2,
            },
        ];

        let metrics = GlobalMetrics::from_frames(&frames);

        assert_eq!(metrics.chord_change_count, 2);
        assert!((metrics.average_voice_leading_effort - 1.0).abs() < 0.001);
        assert!((metrics.total_duration_beats - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_musical_dna_serialization() {
        let dna = MusicalDNA::new();
        let json = serde_json::to_string(&dna).unwrap();
        let parsed: MusicalDNA = serde_json::from_str(&json).unwrap();

        assert!(parsed.harmonic_profile.is_empty());
        assert!(parsed.truth.is_none());
    }

    // ========================================================================
    // DNA EXTRACTION TESTS
    // ========================================================================

    fn create_test_truth() -> RecordingTruth {
        let params = MusicalParams::default();
        let events = vec![
            // Kick pattern (channel 2 = Bass)
            (0.0, AudioEvent::NoteOn { note: 36, velocity: 100, channel: 2 }),
            (0.5, AudioEvent::NoteOn { note: 36, velocity: 100, channel: 2 }),
            (1.0, AudioEvent::NoteOn { note: 36, velocity: 100, channel: 2 }),
            (1.5, AudioEvent::NoteOn { note: 36, velocity: 100, channel: 2 }),
            // Lead melody (channel 0 = Melody) - C major chord tones
            (0.0, AudioEvent::NoteOn { note: 60, velocity: 90, channel: 0 }), // C
            (0.0, AudioEvent::NoteOn { note: 64, velocity: 90, channel: 0 }), // E
            (0.0, AudioEvent::NoteOn { note: 67, velocity: 90, channel: 0 }), // G
            // Transition to Am
            (1.0, AudioEvent::NoteOn { note: 69, velocity: 90, channel: 0 }), // A
            (1.0, AudioEvent::NoteOn { note: 60, velocity: 90, channel: 0 }), // C
            (1.0, AudioEvent::NoteOn { note: 64, velocity: 90, channel: 0 }), // E
            // Snare / Heel (channel 3)
            (0.5, AudioEvent::NoteOn { note: 38, velocity: 110, channel: 3 }),
            (1.5, AudioEvent::NoteOn { note: 38, velocity: 110, channel: 3 }),
            // Hat / Toe (channel 4)
            (0.0, AudioEvent::NoteOn { note: 42, velocity: 70, channel: 4 }),
            (0.25, AudioEvent::NoteOn { note: 42, velocity: 70, channel: 4 }),
            (0.5, AudioEvent::NoteOn { note: 42, velocity: 70, channel: 4 }),
            (0.75, AudioEvent::NoteOn { note: 42, velocity: 70, channel: 4 }),
        ];
        RecordingTruth::new(events, params, 44100)
    }

    #[test]
    fn test_dna_extraction_basic() {
        let truth = create_test_truth();
        let dna = MusicalDNA::extract(&truth);

        // Should have extracted some harmonic frames
        assert!(!dna.harmonic_profile.is_empty());

        // Should have rhythmic DNA
        assert!(!dna.rhythmic_profile.polygons.is_empty());

        // Should have global metrics
        assert!(dna.global_metrics.chord_change_count > 0);
    }

    #[test]
    fn test_chord_grouping() {
        let extractor = DNAExtractor::new(120.0, 4);
        let events = vec![
            // C major chord at time 0
            (0.0, AudioEvent::NoteOn { note: 60, velocity: 90, channel: 0 }),
            (0.0, AudioEvent::NoteOn { note: 64, velocity: 90, channel: 0 }),
            (0.0, AudioEvent::NoteOn { note: 67, velocity: 90, channel: 0 }),
            // A minor chord at time 1.0
            (1.0, AudioEvent::NoteOn { note: 69, velocity: 90, channel: 0 }),
            (1.0, AudioEvent::NoteOn { note: 60, velocity: 90, channel: 0 }),
            (1.0, AudioEvent::NoteOn { note: 64, velocity: 90, channel: 0 }),
        ];

        let groups = extractor.group_notes_into_chords(&events);

        assert_eq!(groups.len(), 2);
        // First group should be C, E, G (pitch classes 0, 4, 7)
        assert_eq!(groups[0].1, vec![0, 4, 7]);
        // Second group should be C, E, A (pitch classes 0, 4, 9)
        assert_eq!(groups[1].1, vec![0, 4, 9]);
    }

    #[test]
    fn test_harmonic_profile_extraction() {
        let truth = create_test_truth();
        let dna = MusicalDNA::extract(&truth);

        // First chord should be C major
        if let Some(first_frame) = dna.harmonic_profile.first() {
            assert!(
                first_frame.chord.contains('C') || first_frame.chord.contains("Unknown"),
                "First chord should be C-based: {}",
                first_frame.chord
            );
        }
    }

    #[test]
    fn test_rhythmic_dna_extraction() {
        let truth = create_test_truth();
        let dna = MusicalDNA::extract(&truth);

        // Should have kick, snare, and hat polygons
        let layer_names: Vec<&str> =
            dna.rhythmic_profile.polygons.iter().map(|p| p.layer.as_str()).collect();

        assert!(layer_names.contains(&"kick"), "Should have kick polygon");
        assert!(layer_names.contains(&"snare"), "Should have snare polygon");
        assert!(layer_names.contains(&"hat"), "Should have hat polygon");
    }

    #[test]
    fn test_syncopation_calculation() {
        let extractor = DNAExtractor::new(120.0, 4);

        // On-beat pattern (no syncopation)
        let on_beat = vec![0.0, 0.5, 1.0, 1.5];
        let sync_score = extractor.calculate_syncopation(&on_beat, &[]);

        // Syncopation should be relatively low for on-beat pattern
        assert!(sync_score < 0.8, "On-beat pattern should have low syncopation: {}", sync_score);
    }

    #[test]
    fn test_density_curve_calculation() {
        let extractor = DNAExtractor::new(120.0, 4);
        let events = vec![
            // First measure: 4 hits
            (0.0, AudioEvent::NoteOn { note: 36, velocity: 100, channel: 2 }),
            (0.5, AudioEvent::NoteOn { note: 38, velocity: 100, channel: 3 }),
            (1.0, AudioEvent::NoteOn { note: 36, velocity: 100, channel: 2 }),
            (1.5, AudioEvent::NoteOn { note: 38, velocity: 100, channel: 3 }),
            // Second measure: 8 hits
            (2.0, AudioEvent::NoteOn { note: 36, velocity: 100, channel: 2 }),
            (2.25, AudioEvent::NoteOn { note: 42, velocity: 70, channel: 4 }),
            (2.5, AudioEvent::NoteOn { note: 38, velocity: 100, channel: 3 }),
            (2.75, AudioEvent::NoteOn { note: 42, velocity: 70, channel: 4 }),
            (3.0, AudioEvent::NoteOn { note: 36, velocity: 100, channel: 2 }),
            (3.25, AudioEvent::NoteOn { note: 42, velocity: 70, channel: 4 }),
            (3.5, AudioEvent::NoteOn { note: 38, velocity: 100, channel: 3 }),
            (3.75, AudioEvent::NoteOn { note: 42, velocity: 70, channel: 4 }),
        ];

        let density = extractor.calculate_density_curve(&events);

        // Should have 2 measures
        assert_eq!(density.len(), 2);
        // Second measure should be denser
        assert!(density[1] > density[0], "Second measure should be denser");
    }

    #[test]
    fn test_dna_json_roundtrip() {
        let truth = create_test_truth();
        let dna = MusicalDNA::extract(&truth);

        // Serialize to JSON
        let json = dna.to_json().expect("Should serialize to JSON");

        // Deserialize back
        let parsed = MusicalDNA::from_json(&json).expect("Should deserialize from JSON");

        // Verify key fields match
        assert_eq!(dna.harmonic_profile.len(), parsed.harmonic_profile.len());
        assert_eq!(dna.rhythmic_profile.polygons.len(), parsed.rhythmic_profile.polygons.len());
        assert_eq!(dna.global_metrics.chord_change_count, parsed.global_metrics.chord_change_count);
    }

    #[test]
    fn test_polygon_signature_creation() {
        let extractor = DNAExtractor::new(120.0, 4);
        let onsets = vec![0.0, 0.5, 1.0, 1.5]; // Quarter note pattern
        let events: Vec<(f64, AudioEvent)> = onsets
            .iter()
            .map(|&t| (t, AudioEvent::NoteOn { note: 36, velocity: 100, channel: 0 }))
            .collect();

        let polygon = extractor.build_polygon_signature("kick", &onsets, &events);

        assert_eq!(polygon.layer, "kick");
        assert_eq!(polygon.vertices, 4);
        // IOIs should be approximately equal for regular pattern
        assert!(!polygon.interval_vector.is_empty());
    }

    #[test]
    fn test_micro_timing_deviation() {
        let extractor = DNAExtractor::new(120.0, 4);

        // Perfect grid alignment
        let perfect_onsets = vec![0.0, 0.125, 0.25, 0.375, 0.5];
        let deviation = extractor.calculate_micro_timing_deviation(&perfect_onsets);

        // Should have very low deviation for grid-aligned notes
        assert!(deviation < 0.1, "Grid-aligned notes should have low deviation: {}", deviation);
    }
}

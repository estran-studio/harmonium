//! MusicXML Parser and DNA Extractor
//!
//! Parses MusicXML files (including compressed .mxl) and extracts
//! Musical DNA profiles for analysis.

use std::path::{Path, PathBuf};

use harmonium_core::theory::{PitchClass, chord::Chord};
use thiserror::Error;
use walkdir::WalkDir;

use crate::dna_types::{
    GlobalMetrics, HarmonicFrame, MusicalDNA, PolygonSignature, RhythmicDNA, SerializableTRQ,
};

/// Errors that can occur during MusicXML ingestion
#[derive(Error, Debug)]
pub enum IngestError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("MusicXML parse error: {0}")]
    Parse(String),

    #[error("No valid parts found in score")]
    NoPartsFound,

    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
}

/// MusicXML ingester for extracting DNA from score files
#[derive(Clone, Debug, Default)]
pub struct MusicXMLIngester {
    /// Default BPM to use when tempo is not specified
    default_bpm: f32,
}

impl MusicXMLIngester {
    /// Create a new MusicXML ingester
    #[must_use]
    pub fn new() -> Self {
        Self { default_bpm: 120.0 }
    }

    /// Create with custom default BPM
    #[must_use]
    pub const fn with_default_bpm(mut self, bpm: f32) -> Self {
        self.default_bpm = bpm;
        self
    }

    /// Find all MusicXML files in a directory
    ///
    /// # Errors
    /// Returns error if directory cannot be read
    pub fn find_musicxml_files(
        &self,
        dir: &Path,
        recursive: bool,
    ) -> Result<Vec<PathBuf>, IngestError> {
        let mut files = Vec::new();

        let walker = if recursive { WalkDir::new(dir) } else { WalkDir::new(dir).max_depth(1) };

        for entry in walker.into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if ext_lower == "xml" || ext_lower == "musicxml" || ext_lower == "mxl" {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }

        Ok(files)
    }

    /// Ingest a single MusicXML file and extract DNA
    /// Supports both .xml/.musicxml (plain XML) and .mxl (compressed) formats
    ///
    /// # Errors
    /// Returns error if file cannot be parsed
    pub fn ingest_file(&self, path: &Path) -> Result<MusicalDNA, IngestError> {
        let content = if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if ext_lower == "mxl" {
                // Handle compressed MusicXML
                self.read_mxl_file(path)?
            } else {
                // Handle plain XML
                std::fs::read_to_string(path)?
            }
        } else {
            std::fs::read_to_string(path)?
        };

        // Parse MusicXML
        self.ingest_string(&content)
    }

    /// Read and decompress a .mxl file
    fn read_mxl_file(&self, path: &Path) -> Result<String, IngestError> {
        use std::io::Read;

        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| IngestError::Parse(format!("Failed to open MXL archive: {}", e)))?;

        // Look for the main MusicXML file
        // First, try to find container.xml to get the rootfile path
        let rootfile_path: Option<String> = {
            if let Ok(mut container) = archive.by_name("META-INF/container.xml") {
                let mut container_content = String::new();
                if container.read_to_string(&mut container_content).is_ok() {
                    // Parse rootfile path from container.xml
                    if let Some(rootfile_start) = container_content.find("full-path=\"") {
                        let start = rootfile_start + 11;
                        if let Some(end) = container_content[start..].find('"') {
                            Some(container_content[start..start + end].to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        // If we found a rootfile path, read it
        if let Some(ref rootpath) = rootfile_path {
            if let Ok(mut rootfile) = archive.by_name(rootpath) {
                let mut content = String::new();
                rootfile
                    .read_to_string(&mut content)
                    .map_err(|e| IngestError::Parse(format!("Failed to read rootfile: {}", e)))?;
                return Ok(content);
            }
        }

        // Fallback: look for any .xml file that looks like MusicXML
        for i in 0..archive.len() {
            let Ok(mut file) = archive.by_index(i) else {
                continue;
            };
            let Ok(name) = file.name() else {
                continue;
            };
            if name.ends_with(".xml") && !name.contains("META-INF") {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() && content.contains("<score-partwise")
                {
                    return Ok(content);
                }
            }
        }

        Err(IngestError::Parse("No valid MusicXML file found in archive".to_string()))
    }

    /// Ingest MusicXML from a string
    ///
    /// # Errors
    /// Returns error if content cannot be parsed
    pub fn ingest_string(&self, content: &str) -> Result<MusicalDNA, IngestError> {
        // Extract note events from MusicXML
        let events = self.extract_note_events(content)?;

        if events.is_empty() {
            return Err(IngestError::NoPartsFound);
        }

        // Build harmonic profile
        let harmonic_profile = self.build_harmonic_profile(&events);

        // Build rhythmic profile
        let rhythmic_profile = self.build_rhythmic_profile(&events);

        // Calculate global metrics
        let global_metrics = GlobalMetrics::from_frames(&harmonic_profile);

        Ok(MusicalDNA { truth: None, harmonic_profile, rhythmic_profile, global_metrics })
    }

    /// Extract note events from MusicXML content
    /// Handles multi-part scores by parsing each part separately and combining by timestamp
    fn extract_note_events(&self, content: &str) -> Result<Vec<NoteEvent>, IngestError> {
        let mut all_events = Vec::new();

        // Find all parts in the score
        let mut part_pos = 0;
        while let Some(part_start) = content[part_pos..].find("<part ") {
            let abs_part_start = part_pos + part_start;
            if let Some(part_end) = content[abs_part_start..].find("</part>") {
                let part_xml = &content[abs_part_start..abs_part_start + part_end + 7];

                // Parse this part
                let part_events = self.extract_events_from_part(part_xml);
                all_events.extend(part_events);

                part_pos = abs_part_start + part_end + 7;
            } else {
                break;
            }
        }

        // Sort all events by time
        all_events.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));

        Ok(all_events)
    }

    /// Extract note events from a single part
    /// Handles pickup measures correctly by calculating actual measure duration from content
    fn extract_events_from_part(&self, part_xml: &str) -> Vec<NoteEvent> {
        let mut events = Vec::new();
        let mut divisions = 4; // Default divisions per quarter note
        let mut measure_start_time = 0.0;
        let mut beats_per_measure = 4.0;

        // Parse measures within this part
        let mut measure_pos = 0;
        while let Some(measure_start) = part_xml[measure_pos..].find("<measure") {
            let abs_measure_start = measure_pos + measure_start;
            if let Some(measure_end) = part_xml[abs_measure_start..].find("</measure>") {
                let measure_xml =
                    &part_xml[abs_measure_start..abs_measure_start + measure_end + 10];

                // Check for divisions update (must be read before parsing notes)
                if let Some(div_start) = measure_xml.find("<divisions>") {
                    if let Some(div_end) = measure_xml[div_start..].find("</divisions>") {
                        let div_str = &measure_xml[div_start + 11..div_start + div_end];
                        if let Ok(d) = div_str.trim().parse::<i32>() {
                            divisions = d;
                        }
                    }
                }

                // Check for time signature update
                if let Some(beats_start) = measure_xml.find("<beats>") {
                    if let Some(beats_end) = measure_xml[beats_start..].find("</beats>") {
                        let beats_str = &measure_xml[beats_start + 7..beats_start + beats_end];
                        if let Ok(b) = beats_str.trim().parse::<f64>() {
                            beats_per_measure = b;
                        }
                    }
                }

                // Parse notes in this measure
                let measure_events =
                    self.extract_events_from_measure(measure_xml, measure_start_time, divisions);

                // Calculate actual measure duration from parsed events
                // This correctly handles pickup measures which are shorter than the time signature
                let actual_measure_duration =
                    self.calculate_measure_duration(measure_xml, divisions, beats_per_measure);

                events.extend(measure_events);

                // Advance to next measure using actual duration, not assumed time signature
                measure_start_time += actual_measure_duration;
                measure_pos = abs_measure_start + measure_end + 10;
            } else {
                break;
            }
        }

        events
    }

    /// Calculate the actual duration of a measure in beats
    /// This is important for pickup measures which don't fill the full time signature
    fn calculate_measure_duration(
        &self,
        measure_xml: &str,
        divisions: i32,
        default_beats: f64,
    ) -> f64 {
        let mut max_time = 0.0;
        let mut current_time = 0.0;

        let mut pos = 0;
        while let Some(note_start) = measure_xml[pos..].find("<note") {
            let abs_start = pos + note_start;
            if let Some(note_end) = measure_xml[abs_start..].find("</note>") {
                let note_xml = &measure_xml[abs_start..abs_start + note_end + 7];

                // Check for backup/forward before this note
                let before_note = &measure_xml[pos..abs_start];
                if let Some(backup_duration) = self.extract_backup_forward(before_note, divisions) {
                    current_time += backup_duration;
                }

                let _is_rest = note_xml.contains("<rest");
                let is_chord = note_xml.contains("<chord");
                let duration = self.extract_duration(note_xml, divisions);

                // Only advance time for non-chord notes
                if !is_chord {
                    let note_end_time = current_time + duration;
                    if note_end_time > max_time {
                        max_time = note_end_time;
                    }
                    current_time = note_end_time;
                }

                pos = abs_start + note_end + 7;
            } else {
                break;
            }
        }

        // Use calculated duration if we found notes, otherwise fall back to time signature
        if max_time > 0.0 { max_time } else { default_beats }
    }

    /// Extract note events from a single measure
    fn extract_events_from_measure(
        &self,
        measure_xml: &str,
        measure_start_time: f64,
        divisions: i32,
    ) -> Vec<NoteEvent> {
        let mut events = Vec::new();
        let mut current_time = measure_start_time;
        let mut chord_start_time = measure_start_time;

        let mut pos = 0;
        while let Some(note_start) = measure_xml[pos..].find("<note") {
            let abs_start = pos + note_start;
            if let Some(note_end) = measure_xml[abs_start..].find("</note>") {
                let note_xml = &measure_xml[abs_start..abs_start + note_end + 7];

                // Check for backup/forward elements before this note
                // (These affect timing within a measure for multiple voices)
                let before_note = &measure_xml[pos..abs_start];
                if let Some(backup_duration) = self.extract_backup_forward(before_note, divisions) {
                    current_time += backup_duration;
                    chord_start_time = current_time;
                }

                let is_rest = note_xml.contains("<rest");
                let is_chord = note_xml.contains("<chord");

                let pitch = if is_rest { None } else { self.extract_pitch(note_xml) };

                let duration = self.extract_duration(note_xml, divisions);

                let note_time = if is_chord {
                    chord_start_time
                } else {
                    chord_start_time = current_time;
                    current_time
                };

                if let Some(midi_pitch) = pitch {
                    events.push(NoteEvent {
                        time: note_time,
                        pitch: midi_pitch,
                        duration,
                        velocity: 80,
                        voice: 0,
                    });
                }

                if !is_chord && !is_rest {
                    current_time += duration;
                } else if is_rest {
                    current_time += duration;
                    chord_start_time = current_time;
                }

                pos = abs_start + note_end + 7;
            } else {
                break;
            }
        }

        events
    }

    /// Extract backup/forward timing adjustments
    fn extract_backup_forward(&self, xml: &str, divisions: i32) -> Option<f64> {
        // Look for <backup> element
        if let Some(backup_start) = xml.rfind("<backup>") {
            if let Some(dur_start) = xml[backup_start..].find("<duration>") {
                if let Some(dur_end) = xml[backup_start + dur_start..].find("</duration>") {
                    let dur_str =
                        &xml[backup_start + dur_start + 10..backup_start + dur_start + dur_end];
                    if let Ok(dur) = dur_str.trim().parse::<i32>() {
                        return Some(-(dur as f64 / divisions as f64));
                    }
                }
            }
        }

        // Look for <forward> element
        if let Some(forward_start) = xml.rfind("<forward>") {
            if let Some(dur_start) = xml[forward_start..].find("<duration>") {
                if let Some(dur_end) = xml[forward_start + dur_start..].find("</duration>") {
                    let dur_str =
                        &xml[forward_start + dur_start + 10..forward_start + dur_start + dur_end];
                    if let Ok(dur) = dur_str.trim().parse::<i32>() {
                        return Some(dur as f64 / divisions as f64);
                    }
                }
            }
        }

        None
    }

    /// Extract pitch from a note XML element
    fn extract_pitch(&self, note_xml: &str) -> Option<u8> {
        // Look for <pitch> element
        let pitch_start = note_xml.find("<pitch>")?;
        let pitch_end = note_xml.find("</pitch>")?;
        let pitch_xml = &note_xml[pitch_start..pitch_end + 8];

        // Extract step (C, D, E, F, G, A, B)
        let step_start = pitch_xml.find("<step>")? + 6;
        let step_end = pitch_xml[step_start..].find("</step>")?;
        let step = &pitch_xml[step_start..step_start + step_end];

        // Extract octave
        let octave_start = pitch_xml.find("<octave>")? + 8;
        let octave_end = pitch_xml[octave_start..].find("</octave>")?;
        let octave: i32 = pitch_xml[octave_start..octave_start + octave_end].trim().parse().ok()?;

        // Extract alter (sharps/flats)
        let alter: i32 = if let Some(alter_start) = pitch_xml.find("<alter>") {
            let alter_end = pitch_xml[alter_start..].find("</alter>")?;
            pitch_xml[alter_start + 7..alter_start + alter_end].trim().parse().unwrap_or(0)
        } else {
            0
        };

        // Convert to MIDI note number
        let step_semitones = match step {
            "C" => 0,
            "D" => 2,
            "E" => 4,
            "F" => 5,
            "G" => 7,
            "A" => 9,
            "B" => 11,
            _ => return None,
        };

        // MIDI note = (octave + 1) * 12 + step + alter
        let midi = (octave + 1) * 12 + step_semitones + alter;

        if midi >= 0 && midi <= 127 { Some(midi as u8) } else { None }
    }

    /// Extract duration from a note XML element
    fn extract_duration(&self, note_xml: &str, divisions: i32) -> f64 {
        if let Some(dur_start) = note_xml.find("<duration>") {
            if let Some(dur_end) = note_xml[dur_start..].find("</duration>") {
                let dur_str = &note_xml[dur_start + 10..dur_start + dur_end];
                if let Ok(dur) = dur_str.trim().parse::<i32>() {
                    // Duration in beats = duration / divisions
                    return dur as f64 / divisions as f64;
                }
            }
        }
        1.0 // Default: quarter note
    }

    /// Build harmonic profile from note events
    fn build_harmonic_profile(&self, events: &[NoteEvent]) -> Vec<HarmonicFrame> {
        if events.is_empty() {
            return Vec::new();
        }

        // Group notes by time window (chord detection)
        let chord_groups = self.group_notes_into_chords(events);

        let mut frames = Vec::new();
        let mut prev_chord: Option<Chord> = None;

        for (i, (timestamp, pitch_classes)) in chord_groups.iter().enumerate() {
            // Try to identify the chord
            let chord = Chord::identify(pitch_classes);

            // Calculate duration
            let duration =
                if i + 1 < chord_groups.len() { chord_groups[i + 1].0 - timestamp } else { 4.0 };

            // Calculate TRQ and voice leading distance
            let (trq, vl_distance) = if let (Some(prev), Some(curr)) = (&prev_chord, &chord) {
                let trq = SerializableTRQ::for_transition(prev, curr);
                let vl = prev.voice_leading_distance(curr);
                (trq, vl)
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
                lcc_level: 1,
            });

            prev_chord = chord;
        }

        frames
    }

    /// Extract chords using harmonic slicing (what notes are SOUNDING at each position)
    /// This is the correct approach for SATB and multi-voice textures.
    /// Unlike onset-based grouping, this considers note durations to build proper vertical slices.
    fn group_notes_into_chords(&self, events: &[NoteEvent]) -> Vec<(f64, Vec<PitchClass>)> {
        // Minimum note duration to be considered harmonic (filters ornaments)
        const MIN_HARMONIC_DURATION: f64 = 0.125;
        // Minimum slice duration to avoid creating too many micro-chords
        const MIN_SLICE_DURATION: f64 = 0.25;

        if events.is_empty() {
            return Vec::new();
        }

        // Filter out very short notes (likely non-harmonic tones)
        let harmonic_events: Vec<&NoteEvent> =
            events.iter().filter(|e| e.duration >= MIN_HARMONIC_DURATION).collect();

        if harmonic_events.is_empty() {
            return self.group_notes_into_chords_unfiltered(events);
        }

        // Collect all unique time points where the sounding notes change
        // (note onsets and note endings)
        let mut time_points: Vec<f64> = Vec::new();
        for event in &harmonic_events {
            time_points.push(event.time);
            time_points.push(event.time + event.duration);
        }
        time_points.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        time_points.dedup_by(|a, b| (*a - *b).abs() < 0.001);

        // Build chord slices at each time point
        let mut slices: Vec<(f64, Vec<PitchClass>)> = Vec::new();

        for &t in &time_points {
            // Find all notes that are SOUNDING at time t
            // (started at or before t, and end after t)
            let mut pitch_classes: Vec<PitchClass> = harmonic_events
                .iter()
                .filter(|e| e.time <= t + 0.001 && e.time + e.duration > t + 0.001)
                .map(|e| e.pitch % 12)
                .collect();

            pitch_classes.sort_unstable();
            pitch_classes.dedup();

            // Only create a new slice if pitch classes changed
            if !pitch_classes.is_empty() {
                let should_add =
                    slices.last().map_or(true, |(_, prev_pcs)| prev_pcs != &pitch_classes);

                if should_add {
                    slices.push((t, pitch_classes));
                }
            }
        }

        // Merge very short slices into longer ones (avoid micro-fragmentation)
        self.merge_short_slices(&mut slices, MIN_SLICE_DURATION);

        slices
    }

    /// Merge slices that are too short into neighboring slices
    fn merge_short_slices(&self, slices: &mut Vec<(f64, Vec<PitchClass>)>, min_duration: f64) {
        if slices.len() < 2 {
            return;
        }

        let mut i = 0;
        while i < slices.len() - 1 {
            let duration = slices[i + 1].0 - slices[i].0;
            if duration < min_duration {
                // Merge this slice into the next one by removing it
                slices.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Fallback grouping without duration filtering
    fn group_notes_into_chords_unfiltered(
        &self,
        events: &[NoteEvent],
    ) -> Vec<(f64, Vec<PitchClass>)> {
        const CHORD_WINDOW: f64 = 0.25;

        let mut groups: Vec<(f64, Vec<PitchClass>)> = Vec::new();
        let mut current_time = events[0].time;
        let mut current_pcs: Vec<PitchClass> = Vec::new();

        for event in events {
            let pc = event.pitch % 12;

            if event.time - current_time > CHORD_WINDOW {
                if !current_pcs.is_empty() {
                    current_pcs.sort_unstable();
                    current_pcs.dedup();
                    groups.push((current_time, current_pcs));
                }
                current_time = event.time;
                current_pcs = vec![pc];
            } else if !current_pcs.contains(&pc) {
                current_pcs.push(pc);
            }
        }

        if !current_pcs.is_empty() {
            current_pcs.sort_unstable();
            current_pcs.dedup();
            groups.push((current_time, current_pcs));
        }

        groups
    }

    /// Build rhythmic profile from note events
    fn build_rhythmic_profile(&self, events: &[NoteEvent]) -> RhythmicDNA {
        if events.is_empty() {
            return RhythmicDNA::default();
        }

        // Extract onset times
        let onsets: Vec<f64> = events.iter().map(|e| e.time).collect();

        // Calculate density curve (hits per measure)
        let density_curve = self.calculate_density_curve(&onsets);

        // Calculate syncopation
        let syncopation_score = self.calculate_syncopation(&onsets);

        // Build a simple polygon signature
        let mut polygons = Vec::new();
        if !onsets.is_empty() {
            polygons.push(PolygonSignature {
                layer: "melody".to_string(),
                vertices: onsets.len().min(16),
                rotation_offset: 0,
                interval_vector: self.calculate_intervals(&onsets),
                velocity: 0.8,
            });
        }

        RhythmicDNA {
            mode: "MusicXML".to_string(),
            polygons,
            syncopation_score,
            density_curve,
            micro_timing_deviation: 0.0, // MusicXML is quantized
        }
    }

    /// Calculate density curve
    fn calculate_density_curve(&self, onsets: &[f64]) -> Vec<f32> {
        if onsets.is_empty() {
            return vec![];
        }

        let max_time = onsets.iter().fold(0.0f64, |a, &b| a.max(b));
        let num_measures = ((max_time / 4.0).ceil() as usize).max(1);
        let mut density = vec![0.0f32; num_measures];

        for &onset in onsets {
            let measure_idx = (onset / 4.0) as usize;
            if measure_idx < num_measures {
                density[measure_idx] += 1.0;
            }
        }

        // Normalize
        for d in &mut density {
            *d = (*d / 16.0).min(1.0);
        }

        density
    }

    /// Calculate syncopation score
    fn calculate_syncopation(&self, onsets: &[f64]) -> f32 {
        if onsets.is_empty() {
            return 0.0;
        }

        let mut off_beat_count = 0;
        for &onset in onsets {
            let beat_position = onset % 4.0;
            let is_strong_beat = beat_position < 0.1
                || (beat_position - 1.0).abs() < 0.1
                || (beat_position - 2.0).abs() < 0.1
                || (beat_position - 3.0).abs() < 0.1;

            if !is_strong_beat {
                off_beat_count += 1;
            }
        }

        off_beat_count as f32 / onsets.len() as f32
    }

    /// Calculate inter-onset intervals
    fn calculate_intervals(&self, onsets: &[f64]) -> Vec<usize> {
        if onsets.len() < 2 {
            return vec![];
        }

        let mut intervals = Vec::new();
        for i in 1..onsets.len().min(17) {
            let ioi = onsets[i] - onsets[i - 1];
            // Quantize to 16th notes (0.25 beats)
            let steps = (ioi / 0.25).round() as usize;
            intervals.push(steps.max(1));
        }

        intervals
    }
}

/// A note event extracted from MusicXML
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct NoteEvent {
    /// Time in beats from start
    time: f64,
    /// MIDI pitch (0-127)
    pitch: u8,
    /// Duration in beats
    duration: f64,
    /// Velocity (0-127)
    velocity: u8,
    /// Voice number
    voice: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_MUSICXML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 3.1 Partwise//EN" "http://www.musicxml.org/dtds/partwise.dtd">
<score-partwise version="3.1">
  <part-list>
    <score-part id="P1">
      <part-name>Piano</part-name>
    </score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <key>
          <fifths>0</fifths>
        </key>
        <time>
          <beats>4</beats>
          <beat-type>4</beat-type>
        </time>
      </attributes>
      <note>
        <pitch>
          <step>C</step>
          <octave>4</octave>
        </pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <chord/>
        <pitch>
          <step>E</step>
          <octave>4</octave>
        </pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <chord/>
        <pitch>
          <step>G</step>
          <octave>4</octave>
        </pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <pitch>
          <step>A</step>
          <octave>4</octave>
        </pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <chord/>
        <pitch>
          <step>C</step>
          <octave>5</octave>
        </pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <chord/>
        <pitch>
          <step>E</step>
          <octave>5</octave>
        </pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

    #[test]
    fn test_extract_pitch() {
        let ingester = MusicXMLIngester::new();

        let note_xml = r#"<note>
            <pitch>
                <step>C</step>
                <octave>4</octave>
            </pitch>
        </note>"#;

        let pitch = ingester.extract_pitch(note_xml);
        assert_eq!(pitch, Some(60)); // C4 = MIDI 60
    }

    #[test]
    fn test_extract_pitch_with_sharp() {
        let ingester = MusicXMLIngester::new();

        let note_xml = r#"<note>
            <pitch>
                <step>F</step>
                <alter>1</alter>
                <octave>4</octave>
            </pitch>
        </note>"#;

        let pitch = ingester.extract_pitch(note_xml);
        assert_eq!(pitch, Some(66)); // F#4 = MIDI 66
    }

    #[test]
    fn test_ingest_simple_musicxml() {
        let ingester = MusicXMLIngester::new();
        let dna = ingester.ingest_string(SIMPLE_MUSICXML).unwrap();

        // Should have detected some chords
        assert!(!dna.harmonic_profile.is_empty());

        // First chord should be C major (C, E, G)
        let first_frame = &dna.harmonic_profile[0];
        assert!(
            first_frame.pitch_class_set.contains(&0) // C
                && first_frame.pitch_class_set.contains(&4) // E
                && first_frame.pitch_class_set.contains(&7), // G
            "First chord should contain C, E, G"
        );
    }

    #[test]
    fn test_group_notes_into_chords() {
        let ingester = MusicXMLIngester::new();

        let events = vec![
            NoteEvent { time: 0.0, pitch: 60, duration: 1.0, velocity: 80, voice: 0 },
            NoteEvent { time: 0.0, pitch: 64, duration: 1.0, velocity: 80, voice: 0 },
            NoteEvent { time: 0.0, pitch: 67, duration: 1.0, velocity: 80, voice: 0 },
            NoteEvent { time: 1.0, pitch: 69, duration: 1.0, velocity: 80, voice: 0 },
            NoteEvent { time: 1.0, pitch: 72, duration: 1.0, velocity: 80, voice: 0 },
        ];

        let groups = ingester.group_notes_into_chords(&events);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].1, vec![0, 4, 7]); // C, E, G
        assert_eq!(groups[1].1, vec![0, 9]); // A, C
    }

    #[test]
    fn test_dna_serialization() {
        let ingester = MusicXMLIngester::new();
        let dna = ingester.ingest_string(SIMPLE_MUSICXML).unwrap();

        let json = dna.to_json().unwrap();
        let parsed: crate::dna_types::MusicalDNA = serde_json::from_str(&json).unwrap();

        assert_eq!(dna.harmonic_profile.len(), parsed.harmonic_profile.len());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_mxl_file() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("mxl_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let mxl_path = dir.join("test.mxl");

        let file = std::fs::File::create(&mxl_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();

        zip.start_file("META-INF/container.xml", options).unwrap();
        zip.write_all(br#"<?xml version="1.0"?><container><rootfiles><rootfile full-path="score.xml"/></rootfiles></container>"#).unwrap();

        zip.start_file("score.xml", options).unwrap();
        zip.write_all(SIMPLE_MUSICXML.as_bytes()).unwrap();

        zip.finish().unwrap();

        let ingester = MusicXMLIngester::new();
        let content = ingester.read_mxl_file(&mxl_path).unwrap();
        assert_eq!(content, SIMPLE_MUSICXML);

        let _ = std::fs::remove_file(&mxl_path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_mxl_file_fallback() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("mxl_test_fallback_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let mxl_path = dir.join("test_fallback.mxl");

        let file = std::fs::File::create(&mxl_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();

        zip.start_file("myscore.xml", options).unwrap();
        zip.write_all(SIMPLE_MUSICXML.as_bytes()).unwrap();

        zip.finish().unwrap();

        let ingester = MusicXMLIngester::new();
        let content = ingester.read_mxl_file(&mxl_path).unwrap();
        assert_eq!(content, SIMPLE_MUSICXML);

        let _ = std::fs::remove_file(&mxl_path);
        let _ = std::fs::remove_dir(&dir);
    }
}

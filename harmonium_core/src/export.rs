//! `MusicXML` Export Module
//!
//! Lightweight `MusicXML` generation for testing and reviewing music generation quality.
//! No audio dependencies - works purely with event history and musical parameters.
//!
//! # Example
//! ```ignore
//! use harmonium_core::export::{to_musicxml_with_chords, ChordSymbol};
//! use harmonium_core::params::MusicalParams;
//! use harmonium_core::events::AudioEvent;
//!
//! let events: Vec<(u64, AudioEvent)> = vec![/* ... */];
//! let chords = vec![
//!     ChordSymbol { step: 0, root: 0, kind: "major".into(), text: "C".into() },
//!     ChordSymbol { step: 16, root: 7, kind: "major".into(), text: "G".into() },
//! ];
//! let params = MusicalParams::default();
//! let xml = to_musicxml_with_chords(&events, &chords, &params, 11025);
//! std::fs::write("output.musicxml", xml).unwrap();
//! ```

use crate::events::AudioEvent;

/// Escape special XML characters to produce valid XML output.
/// Handles: & < > " '
fn xml_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}
use std::{
    collections::HashMap,
    fmt::Write as FmtWrite,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::params::MusicalParams;

/// Get current date in YYYY-MM-DD format (no external dependencies)
fn chrono_date() -> String {
    {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let secs = duration.as_secs();

        // Simple date calculation (no leap second handling, good enough for metadata)
        let days_since_epoch = secs / 86400;
        let mut year = 1970;
        let mut remaining_days = days_since_epoch.cast_signed(); // Explicitly use i64 for calculation

        loop {
            let days_in_year = if is_leap_year(year) { 366 } else { 365 };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            year += 1;
        }

        let days_in_months: [i64; 12] = if is_leap_year(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };

        let mut month = 1;
        for &days in &days_in_months {
            if remaining_days < days {
                break;
            }
            remaining_days -= days;
            month += 1;
        }

        let day = remaining_days + 1;
        format!("{year:04}-{month:02}-{day:02}")
    }
}

const fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Git version info (tag and short SHA)
///
/// Version information is captured at compile time via build.rs, making the binary
/// portable without requiring git to be installed at runtime.
#[derive(Clone, Debug)]
pub struct GitVersion {
    /// Version tag (e.g., "v0.1.0" or crate version if no tag)
    pub tag: String,
    /// Short commit SHA (e.g., "abc1234")
    pub sha: String,
}

impl std::fmt::Display for GitVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.tag, self.sha)
    }
}

impl GitVersion {
    /// Get git version captured at compile time
    ///
    /// This uses environment variables set by build.rs, avoiding the need for
    /// git to be installed at runtime. Falls back to crate version if git info
    /// was not available at compile time.
    #[must_use]
    pub fn detect() -> Self {
        // These are set at compile time by build.rs
        let tag = env!("GIT_VERSION_TAG").to_string();
        let sha = env!("GIT_VERSION_SHA").to_string();
        Self { tag, sha }
    }
}

impl Default for GitVersion {
    fn default() -> Self {
        Self::detect()
    }
}

/// Chord symbol for harmony annotation in `MusicXML`
#[derive(Clone, Debug)]
pub struct ChordSymbol {
    /// Step number where chord starts (0-indexed)
    pub step: usize,
    /// Root pitch class (0-11, where 0=C)
    pub root: u8,
    /// `MusicXML` chord kind (e.g., "major", "minor", "dominant", "major-seventh")
    pub kind: String,
    /// Display text (e.g., "Cmaj7", "Am")
    pub text: String,
}

impl ChordSymbol {
    /// Create a chord symbol from a root pitch class and chord type suffix
    #[must_use]
    pub fn new(step: usize, root: u8, chord_type: &str) -> Self {
        let (kind, text) = Self::type_to_musicxml_kind(root, chord_type);
        Self { step, root: root % 12, kind, text }
    }

    /// Convert chord type suffix to `MusicXML` kind and display text
    fn type_to_musicxml_kind(root: u8, chord_type: &str) -> (String, String) {
        let root_name = Self::root_name(root);

        let (kind, suffix) = match chord_type {
            "Minor" | "m" | "min" => ("minor", "m"),
            "Dominant" | "7" | "dom" | "Dominant7" => ("dominant", "7"),
            "dim" | "Diminished" => ("diminished", "dim"),
            "maj7" | "Major7" => ("major-seventh", "maj7"),
            "m7" | "Minor7" => ("minor-seventh", "m7"),
            "m7b5" | "HalfDiminished" => ("half-diminished", "m7b5"),
            "dim7" | "Diminished7" => ("diminished-seventh", "dim7"),
            "sus2" | "Sus2" => ("suspended-second", "sus2"),
            "sus4" | "Sus4" => ("suspended-fourth", "sus4"),
            "mMaj7" | "MinorMajor7" => ("major-minor", "mMaj7"),
            "7#5" | "Augmented7" => ("augmented-seventh", "7#5"),
            "6" | "Major6" => ("major-sixth", "6"),
            "m6" | "Minor6" => ("minor-sixth", "m6"),
            "7sus4" | "Dominant7Sus4" => ("dominant-11th", "7sus4"),
            "add9" | "Add9" => ("major-ninth", "add9"),
            _ => ("major", ""),
        };

        (kind.to_string(), format!("{root_name}{suffix}"))
    }

    /// Get root note name with proper enharmonic spelling
    const fn root_name(root: u8) -> &'static str {
        match root % 12 {
            10 => "A#",
            11 => "B",
            _ => "C",
        }
    }

    /// Get `MusicXML` root-step and root-alter
    const fn root_step_alter(&self) -> (&'static str, i8) {
        match self.root % 12 {
            10 => ("B", -1), // Bb
            11 => ("B", 0),
            _ => ("C", 0),
        }
    }
}

/// A note computed from NoteOn/NoteOff pair with duration
#[derive(Clone, Debug)]
pub struct ScoreNote {
    /// MIDI pitch (0-127)
    pub pitch: u8,
    /// Start time in steps
    pub start_step: usize,
    /// Duration in steps (from `NoteOn` to `NoteOff`)
    pub duration_steps: usize,
    /// Channel (0=Bass, 1=Lead, 2=Snare, 3=Hat)
    pub channel: u8,
    /// Velocity (for dynamics notation)
    pub velocity: u8,
}

/// Key signature mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyMode {
    Major,
    Minor,
}

/// Clef type for a part
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClefType {
    Treble,
    Bass,
    Percussion,
}

/// Calculate circle of fifths position from `key_root` and mode
///
/// Returns the `<fifths>` value for `MusicXML` (-7 to +7)
/// - Positive values = sharps (G=1, D=2, A=3, E=4, B=5, F#=6, C#=7)
/// - Negative values = flats (F=-1, Bb=-2, Eb=-3, Ab=-4, Db=-5, Gb=-6, Cb=-7)
/// - Zero = C major / A minor
#[must_use]
pub const fn fifths_from_key(key_root: u8, is_minor: bool) -> i8 {
    let root = key_root % 12;

    // Major key fifths mapping
    // This maps pitch class to circle of fifths position
    let major_fifths: [i8; 12] = [
        0,  // C  = 0 fifths
        -5, // Db = -5 fifths (prefer flat over C#=7)
        2,  // D  = 2 fifths
        -3, // Eb = -3 fifths (prefer flat over D#=9)
        4,  // E  = 4 fifths
        -1, // F  = -1 fifths
        6,  // F# = 6 fifths (or Gb=-6, prefer sharp for symmetry)
        1,  // G  = 1 fifth
        -4, // Ab = -4 fifths (prefer flat over G#=8)
        3,  // A  = 3 fifths
        -2, // Bb = -2 fifths (prefer flat over A#=10)
        5,  // B  = 5 fifths
    ];

    if is_minor {
        // Minor keys use the same key signature as their relative major
        // Relative major is 3 semitones up from minor root
        let relative_major = (root + 3) % 12;
        major_fifths[relative_major as usize]
    } else {
        major_fifths[root as usize]
    }
}

/// Get the key name string for display
#[must_use]
pub const fn key_name(key_root: u8, is_minor: bool) -> &'static str {
    let root = key_root % 12;
    if is_minor {
        match root {
            0 => "C minor",
            1 => "C# minor",
            2 => "D minor",
            3 => "Eb minor",
            4 => "E minor",
            5 => "F minor",
            6 => "F# minor",
            7 => "G minor",
            8 => "G# minor",
            9 => "A minor",
            10 => "Bb minor",
            11 => "B minor",
            _ => "Unknown",
        }
    } else {
        match root {
            0 => "C major",
            1 => "Db major",
            2 => "D major",
            3 => "Eb major",
            4 => "E major",
            5 => "F major",
            6 => "F# major",
            7 => "G major",
            8 => "Ab major",
            9 => "A major",
            10 => "Bb major",
            11 => "B major",
            _ => "Unknown",
        }
    }
}

/// Infer time signature from `rhythm_steps`
///
/// Returns (beats, `beat_type`) e.g., (4, 4) for 4/4 time
#[must_use]
pub const fn time_signature_from_steps(rhythm_steps: usize) -> (u8, u8) {
    match rhythm_steps {
        12 => (3, 4), // 3/4 time
        24 => (6, 8), // 6/8 time (compound duple)
        _ => (4, 4),  // Default to 4/4 (includes 16, 48, 96, 192)
    }
}

/// Calculate steps per quarter note based on `rhythm_steps`
const fn steps_per_quarter(rhythm_steps: usize) -> usize {
    match rhythm_steps {
        12 => 4,   // 3/4: 12 steps / 3 beats = 4 steps per quarter
        16 => 4,   // 4/4: 16 steps / 4 beats = 4 steps per quarter
        24 => 4,   // 6/8: treat as 4 steps per quarter (compound time)
        48 => 12,  // 4/4 high-res: 48 steps / 4 beats = 12 steps per quarter
        96 => 24,  // 4/4 very high-res
        192 => 48, // 4/4 ultra high-res
        _ => 4,    // Default
    }
}

/// Main builder for `MusicXML` output
pub struct MusicXmlBuilder {
    /// Event history (timestamp in musical steps, event)
    events: Vec<(f64, AudioEvent)>, // Changed from u64 to f64
    /// Musical parameters
    params: MusicalParams,
    /// Computed score notes
    notes: Vec<ScoreNote>,
    /// Chord symbols for harmony annotations
    chord_symbols: Vec<ChordSymbol>,
    /// Key signature fifths value
    fifths: i8,
    /// Key mode (major/minor)
    mode: KeyMode,
    /// Time signature (beats, `beat_type`)
    time_sig: (u8, u8),
    /// Steps per quarter note (`MusicXML` divisions)
    divisions: usize,
}

impl MusicXmlBuilder {
    /// Create a new `MusicXML` builder from event history
    #[must_use]
    pub fn new(
        events: Vec<(f64, AudioEvent)>, // Changed to f64
        params: &MusicalParams,
        samples_per_step: usize,
    ) -> Self {
        Self::with_chords(events, Vec::new(), params, samples_per_step)
    }

    /// Create a new `MusicXML` builder with chord symbols
    #[must_use]
    pub fn with_chords(
        events: Vec<(f64, AudioEvent)>, // Changed to f64
        chord_symbols: Vec<ChordSymbol>,
        params: &MusicalParams,
        _samples_per_step: usize,
    ) -> Self {
        let is_minor = params.harmony_valence < 0.0;
        let fifths = fifths_from_key(params.key_root, is_minor);
        let mode = if is_minor { KeyMode::Minor } else { KeyMode::Major };
        let time_sig = time_signature_from_steps(params.rhythm_steps);
        let divisions = steps_per_quarter(params.rhythm_steps);

        let mut builder = Self {
            events,
            params: params.clone(),
            notes: Vec::new(),
            chord_symbols,
            fifths,
            mode,
            time_sig,
            divisions,
        };
        builder.compute_notes();
        builder
    }

    /// Build the `MusicXML` string (auto-detects git version)
    #[must_use]
    pub fn build(&self) -> String {
        let git = GitVersion::detect();
        self.build_with_version(&git.tag, &git.sha)
    }

    /// Build the `MusicXML` string with version info
    ///
    /// Note: Writing to a String in Rust cannot fail (except for OOM which would panic anyway),
    /// so we use `let _ =` to acknowledge write results without panicking behavior.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn build_with_version(&self, version: &str, git_sha: &str) -> String {
        let mut xml = String::new();

        // Escape version info to prevent XML injection
        let version_escaped = xml_escape(version);
        let git_sha_escaped = xml_escape(git_sha);

        // XML header
        let _ = writeln!(xml, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        let _ = writeln!(
            xml,
            r#"<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 4.0 Partwise//EN" "http://www.musicxml.org/dtds/partwise.dtd">"#
        );
        let _ = writeln!(xml, r#"<score-partwise version="4.0">"#);

        // Work title with version
        // Handle version with or without 'v' prefix
        let version_display = if version_escaped.starts_with('v') {
            version_escaped
        } else {
            format!("v{version_escaped}")
        };
        let _ = writeln!(xml, "  <work>");
        let _ = writeln!(
            xml,
            "    <work-title>Harmonium {version_display}-{git_sha_escaped}</work-title>"
        );
        let _ = writeln!(xml, "  </work>");

        // Identification
        let key_name = key_name(self.params.key_root, self.mode == KeyMode::Minor);
        let _ = writeln!(xml, "  <identification>");
        let _ = writeln!(xml, r#"    <creator type="composer">Harmonium</creator>"#);
        let _ = writeln!(xml, "    <encoding>");
        let _ = writeln!(
            xml,
            "      <software>harmonium_core {version_display}-{git_sha_escaped}</software>"
        );
        let _ = writeln!(xml, "      <encoding-date>{}</encoding-date>", chrono_date());
        let _ = writeln!(xml, "    </encoding>");
        let _ = writeln!(xml, "    <miscellaneous>");
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"key\">{key_name}</miscellaneous-field>"
        );
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"bpm\">{}</miscellaneous-field>",
            self.params.bpm
        );
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"time_signature\">{}/{}</miscellaneous-field>",
            self.time_sig.0, self.time_sig.1
        );
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"rhythm_mode\">{:?}</miscellaneous-field>",
            self.params.rhythm_mode
        );
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"rhythm_steps\">{}</miscellaneous-field>",
            self.params.rhythm_steps
        );
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"rhythm_pulses\">{}</miscellaneous-field>",
            self.params.rhythm_pulses
        );
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"rhythm_density\">{:.2}</miscellaneous-field>",
            self.params.rhythm_density
        );
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"rhythm_tension\">{:.2}</miscellaneous-field>",
            self.params.rhythm_tension
        );
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"harmony_tension\">{:.2}</miscellaneous-field>",
            self.params.harmony_tension
        );
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"harmony_valence\">{:.2}</miscellaneous-field>",
            self.params.harmony_valence
        );
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"harmony_measures_per_chord\">{}</miscellaneous-field>",
            self.params.harmony_measures_per_chord
        );
        let _ = writeln!(
            xml,
            "      <miscellaneous-field name=\"melody_octave\">{}</miscellaneous-field>",
            self.params.melody_octave
        );
        let _ = writeln!(xml, "    </miscellaneous>");
        let _ = writeln!(xml, "  </identification>");

        // Credits for display on score (MuseScore shows these)
        // Compact title with version and date - positioned at top
        let _ = writeln!(xml, "  <credit page=\"1\">");
        let _ = writeln!(xml, "    <credit-type>title</credit-type>");
        let _ = writeln!(
            xml,
            r#"    <credit-words default-x="600" default-y="1550" font-size="14" justify="center" valign="top">Harmonium {}-{} | {}</credit-words>"#,
            version_display,
            git_sha_escaped,
            chrono_date()
        );
        let _ = writeln!(xml, "  </credit>");

        // Compact parameters (single line) - positioned below title
        // Format: Key | BPM | Time | Mode(steps,pulses,d,t) | Harm(t,v,m)
        let mode_full = format!("{:?}", self.params.rhythm_mode);
        let mode_short = match mode_full.as_str() {
            "Euclidean" => "Euc",
            "PerfectBalance" => "PB",
            "ClassicGroove" => "CG",
            _ => &mode_full,
        };
        let _ = writeln!(xml, "  <credit page=\"1\">");
        let _ = writeln!(xml, "    <credit-type>subtitle</credit-type>");
        let _ = writeln!(
            xml,
            r#"    <credit-words default-x="600" default-y="1520" font-size="9" justify="center" valign="top">{} | {} BPM | {}/{} | {}({},{},{:.1},{:.1}) | H({:.1},{:.1},{})</credit-words>"#,
            key_name,
            self.params.bpm,
            self.time_sig.0,
            self.time_sig.1,
            mode_short,
            self.params.rhythm_steps,
            self.params.rhythm_pulses,
            self.params.rhythm_density,
            self.params.rhythm_tension,
            self.params.harmony_tension,
            self.params.harmony_valence,
            self.params.harmony_measures_per_chord
        );
        let _ = writeln!(xml, "  </credit>");

        // Part list
        let _ = writeln!(xml, "  <part-list>");
        let _ =
            writeln!(xml, r#"    <score-part id="P1"><part-name>Lead</part-name></score-part>"#);
        let _ =
            writeln!(xml, r#"    <score-part id="P2"><part-name>Bass</part-name></score-part>"#);
        let _ =
            writeln!(xml, r#"    <score-part id="P3"><part-name>Drums</part-name></score-part>"#);
        let _ = writeln!(xml, "  </part-list>");

        // Parts
        self.write_part(&mut xml, "P1", 1, ClefType::Treble);
        self.write_part(&mut xml, "P2", 0, ClefType::Bass);
        self.write_drum_part(&mut xml, "P3");

        let _ = writeln!(xml, "</score-partwise>");
        xml
    }

    /// Compute `ScoreNotes` from NoteOn/NoteOff events
    fn compute_notes(&mut self) {
        eprintln!("MusicXML export: Processing {} total events", self.events.len());

        // Map of (channel, pitch) -> (start_step, velocity)
        let mut pending: HashMap<(u8, u8), (usize, u8)> = HashMap::new();
        let mut notes = Vec::new();

        for (step_timestamp, event) in &self.events {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let step = step_timestamp.round() as usize;

            match event {
                AudioEvent::NoteOn { note, velocity, channel } => {
                    if *velocity > 0 {
                        // If this note is already pending (common for percussion), finalize it first
                        if let Some((old_start, old_vel)) = pending.get(&(*channel, *note)) {
                            // Give it a short default duration (percussion hits)
                            let default_duration = if *channel == 2 || *channel == 3 {
                                2 // Short percussion hit
                            } else {
                                4 // Quarter note for other instruments
                            };

                            notes.push(ScoreNote {
                                pitch: *note,
                                start_step: *old_start,
                                duration_steps: default_duration,
                                channel: *channel,
                                velocity: *old_vel,
                            });
                        }
                        pending.insert((*channel, *note), (step, *velocity));
                    } else {
                        // Velocity 0 = NoteOff
                        if let Some((start, vel)) = pending.remove(&(*channel, *note)) {
                            notes.push(ScoreNote {
                                pitch: *note,
                                start_step: start,
                                duration_steps: step.saturating_sub(start).max(1),
                                channel: *channel,
                                velocity: vel,
                            });
                        }
                    }
                }
                AudioEvent::NoteOff { note, channel } => {
                    if let Some((start, vel)) = pending.remove(&(*channel, *note)) {
                        notes.push(ScoreNote {
                            pitch: *note,
                            start_step: start,
                            duration_steps: step.saturating_sub(start).max(1),
                            channel: *channel,
                            velocity: vel,
                        });
                    }
                }
                _ => {}
            }
        }

        // Handle pending notes (no NoteOff received)
        let pending_count = pending.len();
        for ((channel, pitch), (start, vel)) in pending {
            // Percussion channels (2=snare, 3=hat) get short default duration
            // Other channels get full measure duration
            let default_duration = if channel == 2 || channel == 3 {
                2 // Short percussion hit (equivalent to 8th note at 16 steps/measure)
            } else {
                self.params.rhythm_steps // Full measure for sustained notes
            };

            notes.push(ScoreNote {
                pitch,
                start_step: start,
                duration_steps: default_duration,
                channel,
                velocity: vel,
            });
        }

        if pending_count > 0 {
            eprintln!("MusicXML export: {pending_count} notes still pending (no NoteOff received)");
        }

        notes.sort_by_key(|n| (n.start_step, n.channel, n.pitch));

        // Debug: Count notes per channel
        let mut channel_counts = [0; 16];
        for note in &notes {
            channel_counts[note.channel as usize] += 1;
        }
        eprintln!("MusicXML: Created {} total notes from events", notes.len());
        eprintln!("MusicXML: Note counts by channel:");
        for (ch, count) in channel_counts.iter().enumerate() {
            if *count > 0 {
                eprintln!("  Channel {ch}: {count} notes");
            }
        }

        self.notes = notes;
    }

    /// Write a pitched part (Lead or Bass)
    /// Lead part (channel 1) includes chord symbols
    fn write_part(&self, xml: &mut String, part_id: &str, channel: u8, clef: ClefType) {
        let _ = writeln!(xml, r#"  <part id="{part_id}">"#);

        let part_notes: Vec<&ScoreNote> =
            self.notes.iter().filter(|n| n.channel == channel).collect();

        let steps_per_measure = self.steps_per_measure();
        let total_measures = self.calculate_total_measures(&part_notes);

        // Only show chord symbols on Lead part (channel 1)
        let show_chords = channel == 1;

        for measure in 0..total_measures.max(1) {
            let _ = writeln!(xml, r#"    <measure number="{}">"#, measure + 1);

            // Attributes on first measure
            if measure == 0 {
                self.write_attributes(xml, clef);
            }

            // Get notes for this measure
            let measure_start = measure * steps_per_measure;
            let measure_end = (measure + 1) * steps_per_measure;

            // Get chord symbols for this measure (only for Lead)
            let chords_in_measure: Vec<&ChordSymbol> = if show_chords {
                self.chord_symbols
                    .iter()
                    .filter(|c| c.step >= measure_start && c.step < measure_end)
                    .collect()
            } else {
                Vec::new()
            };

            let notes_in_measure: Vec<&&ScoreNote> = part_notes
                .iter()
                .filter(|n| n.start_step >= measure_start && n.start_step < measure_end)
                .collect();

            let mut current_pos = measure_start;
            let mut chord_idx = 0;

            // Group notes by start_step for chord detection
            // Use a tolerance of 1 step for chord grouping (real-time recording can have slight timing differences)
            const CHORD_TOLERANCE: usize = 1;
            let mut i = 0;
            while i < notes_in_measure.len() {
                let note = notes_in_measure[i];

                // Skip notes that start at or after the measure end
                if note.start_step >= measure_end {
                    i += 1;
                    continue;
                }

                // Write any chord symbols that come before or at this note
                while chord_idx < chords_in_measure.len()
                    && chords_in_measure[chord_idx].step <= note.start_step
                {
                    self.write_harmony(xml, chords_in_measure[chord_idx]);
                    chord_idx += 1;
                }

                // Calculate remaining space in measure
                let remaining = measure_end.saturating_sub(current_pos);
                if remaining == 0 {
                    // Measure is full, skip remaining notes
                    break;
                }

                // Fill rest before note (only if there's a significant gap)
                let note_pos = note.start_step.max(current_pos);
                if note_pos > current_pos {
                    let rest_duration = (note_pos - current_pos).min(remaining);
                    if rest_duration > 0 {
                        self.write_rest(xml, rest_duration);
                        current_pos += rest_duration;
                    }
                }

                // Recalculate remaining after rest
                let remaining = measure_end.saturating_sub(current_pos);
                if remaining == 0 {
                    break;
                }

                // Check for simultaneous notes (chord) - use tolerance for real-time recordings
                let mut chord_notes = vec![note];
                let mut j = i + 1;
                while j < notes_in_measure.len() {
                    let time_diff = notes_in_measure[j].start_step.saturating_sub(note.start_step);
                    if time_diff <= CHORD_TOLERANCE {
                        chord_notes.push(notes_in_measure[j]);
                        j += 1;
                    } else {
                        break;
                    }
                }

                // Clamp note duration to fit in remaining measure space
                let clamped_duration = chord_notes[0].duration_steps.min(remaining);
                if clamped_duration == 0 {
                    i = j;
                    continue;
                }

                // Write chord notes (all with same duration)
                for (idx, chord_note) in chord_notes.iter().enumerate() {
                    self.write_pitched_note_with_duration(
                        xml,
                        chord_note,
                        idx > 0,
                        clamped_duration,
                    );
                }

                current_pos += clamped_duration;
                i = j;
            }

            // Write any remaining chord symbols at end of measure
            while chord_idx < chords_in_measure.len() {
                self.write_harmony(xml, chords_in_measure[chord_idx]);
                chord_idx += 1;
            }

            // Fill rest at end of measure
            if current_pos < measure_end {
                self.write_rest(xml, measure_end - current_pos);
            }

            let _ = writeln!(xml, "    </measure>");
        }

        let _ = writeln!(xml, "  </part>");
    }

    /// Write a harmony (chord symbol) element
    #[allow(clippy::unused_self)]
    fn write_harmony(&self, xml: &mut String, chord: &ChordSymbol) {
        let (root_step, root_alter) = chord.root_step_alter();
        // Escape chord text to prevent XML injection
        let text_escaped = xml_escape(&chord.text);
        let kind_escaped = xml_escape(&chord.kind);

        let _ = writeln!(xml, "      <harmony>");
        let _ = writeln!(xml, "        <root>");
        let _ = writeln!(xml, "          <root-step>{root_step}</root-step>");
        if root_alter != 0 {
            let _ = writeln!(xml, "          <root-alter>{root_alter}</root-alter>");
        }
        let _ = writeln!(xml, "        </root>");
        let _ = writeln!(xml, "        <kind text=\"{text_escaped}\">{kind_escaped}</kind>");
        let _ = writeln!(xml, "      </harmony>");
    }

    /// Write the drum part (channels 2 and 3 combined)
    fn write_drum_part(&self, xml: &mut String, part_id: &str) {
        let _ = writeln!(xml, r#"  <part id="{part_id}">"#);

        let drum_notes: Vec<&ScoreNote> =
            self.notes.iter().filter(|n| n.channel == 2 || n.channel == 3).collect();

        let steps_per_measure = self.steps_per_measure();
        let total_measures = self.calculate_total_measures(&drum_notes);

        for measure in 0..total_measures.max(1) {
            let _ = writeln!(xml, r#"    <measure number="{}">"#, measure + 1);

            // Attributes on first measure (percussion clef)
            if measure == 0 {
                let _ = writeln!(xml, "      <attributes>");
                let _ = writeln!(xml, "        <divisions>{}</divisions>", self.divisions);
                let _ = writeln!(xml, "        <key><fifths>0</fifths></key>");
                let _ = writeln!(
                    xml,
                    "        <time><beats>{}</beats><beat-type>{}</beat-type></time>",
                    self.time_sig.0, self.time_sig.1
                );
                let _ = writeln!(xml, "        <clef><sign>percussion</sign><line>2</line></clef>");
                let _ = writeln!(xml, "      </attributes>");
            }

            let measure_start = measure * steps_per_measure;
            let measure_end = (measure + 1) * steps_per_measure;

            let notes_in_measure: Vec<&&ScoreNote> = drum_notes
                .iter()
                .filter(|n| n.start_step >= measure_start && n.start_step < measure_end)
                .collect();

            let mut current_pos = measure_start;

            // Use same chord tolerance as pitched parts
            const CHORD_TOLERANCE: usize = 1;
            let mut i = 0;
            while i < notes_in_measure.len() {
                let note = notes_in_measure[i];

                // Skip notes that start at or after the measure end
                if note.start_step >= measure_end {
                    i += 1;
                    continue;
                }

                // Calculate remaining space in measure
                let remaining = measure_end.saturating_sub(current_pos);
                if remaining == 0 {
                    break;
                }

                // Fill rest before note
                let note_pos = note.start_step.max(current_pos);
                if note_pos > current_pos {
                    let rest_duration = (note_pos - current_pos).min(remaining);
                    if rest_duration > 0 {
                        self.write_rest(xml, rest_duration);
                        current_pos += rest_duration;
                    }
                }

                // Recalculate remaining after rest
                let remaining = measure_end.saturating_sub(current_pos);
                if remaining == 0 {
                    break;
                }

                // Check for simultaneous drum hits - use tolerance
                let mut chord_notes = vec![note];
                let mut j = i + 1;
                while j < notes_in_measure.len() {
                    let time_diff = notes_in_measure[j].start_step.saturating_sub(note.start_step);
                    if time_diff <= CHORD_TOLERANCE {
                        chord_notes.push(notes_in_measure[j]);
                        j += 1;
                    } else {
                        break;
                    }
                }

                // Clamp duration to fit in remaining measure space
                let clamped_duration = chord_notes[0].duration_steps.min(remaining);
                if clamped_duration == 0 {
                    i = j;
                    continue;
                }

                // Write drum notes (all with same duration)
                for (idx, chord_note) in chord_notes.iter().enumerate() {
                    self.write_drum_note_with_duration(xml, chord_note, idx > 0, clamped_duration);
                }

                current_pos += clamped_duration;
                i = j;
            }

            if current_pos < measure_end {
                self.write_rest(xml, measure_end - current_pos);
            }

            let _ = writeln!(xml, "    </measure>");
        }

        let _ = writeln!(xml, "  </part>");
    }

    /// Write measure attributes (key, time, clef)
    fn write_attributes(&self, xml: &mut String, clef: ClefType) {
        let _ = writeln!(xml, "      <attributes>");
        let _ = writeln!(xml, "        <divisions>{}</divisions>", self.divisions);

        // Key signature
        let mode_str = match self.mode {
            KeyMode::Major => "major",
            KeyMode::Minor => "minor",
        };
        let _ = writeln!(
            xml,
            "        <key><fifths>{}</fifths><mode>{}</mode></key>",
            self.fifths, mode_str
        );

        // Time signature
        let _ = writeln!(
            xml,
            "        <time><beats>{}</beats><beat-type>{}</beat-type></time>",
            self.time_sig.0, self.time_sig.1
        );

        // Clef
        let (sign, line) = match clef {
            ClefType::Treble => ("G", 2),
            ClefType::Bass => ("F", 4),
            ClefType::Percussion => ("percussion", 2),
        };
        let _ = writeln!(xml, "        <clef><sign>{sign}</sign><line>{line}</line></clef>");

        let _ = writeln!(xml, "      </attributes>");
    }

    /// Write a pitched note with explicit duration (for measure clamping)
    fn write_pitched_note_with_duration(
        &self,
        xml: &mut String,
        note: &ScoreNote,
        is_chord: bool,
        duration: usize,
    ) {
        let (step, alter, octave) = self.midi_to_pitch(note.pitch);
        let (note_type, dots) = self.duration_to_type(duration);

        let _ = writeln!(xml, "      <note>");
        if is_chord {
            let _ = writeln!(xml, "        <chord/>");
        }
        let _ = writeln!(xml, "        <pitch>");
        let _ = writeln!(xml, "          <step>{step}</step>");
        if alter != 0 {
            let _ = writeln!(xml, "          <alter>{alter}</alter>");
        }
        let _ = writeln!(xml, "          <octave>{octave}</octave>");
        let _ = writeln!(xml, "        </pitch>");
        let _ = writeln!(xml, "        <duration>{duration}</duration>");
        let _ = writeln!(xml, "        <type>{note_type}</type>");
        for _ in 0..dots {
            let _ = writeln!(xml, "        <dot/>");
        }
        // Add accidental for visual display if altered
        if alter != 0 {
            let acc = if alter > 0 { "sharp" } else { "flat" };
            let _ = writeln!(xml, "        <accidental>{acc}</accidental>");
        }
        let _ = writeln!(xml, "      </note>");
    }

    /// Write a drum note with explicit duration (for measure clamping)
    fn write_drum_note_with_duration(
        &self,
        xml: &mut String,
        note: &ScoreNote,
        is_chord: bool,
        duration: usize,
    ) {
        // Map channel to display position
        // Channel 2 = Snare (middle of staff, E4)
        // Channel 3 = Hi-hat (above staff, G5)
        let (display_step, display_octave) = match note.channel {
            2 => ("E", 4), // Snare
            3 => ("G", 5), // Hi-hat
            _ => ("F", 4), // Default (kick would be here)
        };

        let (note_type, dots) = self.duration_to_type(duration);

        let _ = writeln!(xml, "      <note>");
        if is_chord {
            let _ = writeln!(xml, "        <chord/>");
        }
        let _ = writeln!(xml, "        <unpitched>");
        let _ = writeln!(xml, "          <display-step>{display_step}</display-step>");
        let _ = writeln!(xml, "          <display-octave>{display_octave}</display-octave>");
        let _ = writeln!(xml, "        </unpitched>");
        let _ = writeln!(xml, "        <duration>{duration}</duration>");
        let _ = writeln!(xml, "        <type>{note_type}</type>");
        for _ in 0..dots {
            let _ = writeln!(xml, "        <dot/>");
        }
        let _ = writeln!(xml, "      </note>");
    }

    /// Write a rest
    fn write_rest(&self, xml: &mut String, duration: usize) {
        if duration == 0 {
            return;
        }
        let (note_type, dots) = self.duration_to_type(duration);

        let _ = writeln!(xml, "      <note>");
        let _ = writeln!(xml, "        <rest/>");
        let _ = writeln!(xml, "        <duration>{duration}</duration>");
        let _ = writeln!(xml, "        <type>{note_type}</type>");
        for _ in 0..dots {
            let _ = writeln!(xml, "        <dot/>");
        }
        let _ = writeln!(xml, "      </note>");
    }

    /// Convert MIDI pitch to `MusicXML` pitch components
    /// Returns (step, alter, octave)
    const fn midi_to_pitch(&self, midi: u8) -> (&'static str, i8, u8) {
        let octave = (midi / 12).saturating_sub(1);
        let pitch_class = midi % 12;

        // Use key signature to determine enharmonic spelling
        // Sharp keys (fifths >= 0): prefer sharps
        // Flat keys (fifths < 0): prefer flats
        let use_sharps = self.fifths >= 0;

        let (step, alter) = match pitch_class {
            0 => ("C", 0),
            1 => {
                if use_sharps {
                    ("C", 1)
                } else {
                    ("D", -1)
                }
            }
            2 => ("D", 0),
            3 => {
                if use_sharps {
                    ("D", 1)
                } else {
                    ("E", -1)
                }
            }
            4 => ("E", 0),
            5 => ("F", 0),
            6 => {
                if use_sharps {
                    ("F", 1)
                } else {
                    ("G", -1)
                }
            }
            7 => ("G", 0),
            8 => {
                if use_sharps {
                    ("G", 1)
                } else {
                    ("A", -1)
                }
            }
            9 => ("A", 0),
            10 => {
                if use_sharps {
                    ("A", 1)
                } else {
                    ("B", -1)
                }
            }
            11 => ("B", 0),
            _ => ("C", 0),
        };

        (step, alter, octave)
    }

    /// Convert duration in steps to `MusicXML` note type
    /// Returns (`type_name`, `dot_count`)
    fn duration_to_type(&self, duration: usize) -> (&'static str, u8) {
        #[allow(clippy::cast_precision_loss)]
        // Calculate ratio to quarter note
        let quarters = duration as f32 / self.divisions as f32;

        // Standard note types: whole=4, half=2, quarter=1, eighth=0.5, 16th=0.25
        if quarters >= 4.0 {
            ("whole", 0)
        } else if quarters >= 3.0 {
            ("half", 1) // dotted half
        } else if quarters >= 2.0 {
            ("half", 0)
        } else if quarters >= 1.5 {
            ("quarter", 1) // dotted quarter
        } else if quarters >= 1.0 {
            ("quarter", 0)
        } else if quarters >= 0.75 {
            ("eighth", 1) // dotted eighth
        } else if quarters >= 0.5 {
            ("eighth", 0)
        } else if quarters >= 0.375 {
            ("16th", 1) // dotted 16th
        } else if quarters >= 0.25 {
            ("16th", 0)
        } else {
            ("32nd", 0)
        }
    }

    /// Calculate steps per measure
    const fn steps_per_measure(&self) -> usize {
        self.params.rhythm_steps
    }

    /// Calculate total measures needed based on ALL notes (not just one part)
    /// All parts must have the same number of measures in `MusicXML`
    fn calculate_total_measures(&self, _notes: &[&ScoreNote]) -> usize {
        let steps_per_measure = self.steps_per_measure();
        // Calculate based on ALL notes across ALL parts
        let max_step =
            self.notes.iter().map(|n| n.start_step + n.duration_steps).max().unwrap_or(0);
        max_step.div_ceil(steps_per_measure)
    }
}

/// Convert event history + musical params to `MusicXML` string
///
/// # Arguments
/// * `events` - Vector of (`timestamp_samples`, `AudioEvent`) tuples
/// * `params` - Musical parameters containing key, time signature info
/// * `samples_per_step` - Number of audio samples per sequencer step
///
/// # Returns
/// A complete `MusicXML` 4.0 string ready to be saved or opened in notation software
#[must_use]
pub fn to_musicxml(
    events: &[(f64, AudioEvent)], // Changed from u64 to f64
    params: &MusicalParams,
    samples_per_step: usize, // Kept for backward compatibility
) -> String {
    let builder = MusicXmlBuilder::new(events.to_vec(), params, samples_per_step);
    builder.build()
}

/// Write `MusicXML` to a file
///
/// # Arguments
/// * `events` - Vector of (`timestamp_samples`, `AudioEvent`) tuples
/// * `params` - Musical parameters containing key, time signature info
/// * `samples_per_step` - Number of audio samples per sequencer step
/// * `path` - Output file path
pub fn write_musicxml(
    events: &[(f64, AudioEvent)], // Changed to f64
    params: &MusicalParams,
    samples_per_step: usize,
    path: &Path,
) -> std::io::Result<()> {
    let xml = to_musicxml(events, params, samples_per_step);
    let mut file = std::fs::File::create(path)?;
    file.write_all(xml.as_bytes())?;
    Ok(())
}

/// Convert event history + chord symbols + musical params to `MusicXML` string
///
/// # Arguments
/// * `events` - Vector of (`timestamp_samples`, `AudioEvent`) tuples
/// * `chords` - Vector of chord symbols with step positions
/// * `params` - Musical parameters containing key, time signature info
/// * `samples_per_step` - Number of audio samples per sequencer step
///
/// # Returns
/// A complete `MusicXML` 4.0 string with chord symbols above the lead part
#[must_use]
pub fn to_musicxml_with_chords(
    events: &[(f64, AudioEvent)], // Changed from u64 to f64
    chords: &[ChordSymbol],
    params: &MusicalParams,
    samples_per_step: usize, // Kept for backward compatibility
) -> String {
    let builder =
        MusicXmlBuilder::with_chords(events.to_vec(), chords.to_vec(), params, samples_per_step);
    builder.build()
}

/// Write `MusicXML` with chord symbols to a file
///
/// # Arguments
/// * `events` - Vector of (`timestamp_samples`, `AudioEvent`) tuples
/// * `chords` - Vector of chord symbols with step positions
/// * `params` - Musical parameters containing key, time signature info
/// * `samples_per_step` - Number of audio samples per sequencer step
/// * `path` - Output file path
pub fn write_musicxml_with_chords(
    events: &[(f64, AudioEvent)], // Changed to f64
    chords: &[ChordSymbol],
    params: &MusicalParams,
    samples_per_step: usize,
    path: &Path,
) -> std::io::Result<()> {
    let xml = to_musicxml_with_chords(events, chords, params, samples_per_step);
    let mut file = std::fs::File::create(path)?;
    file.write_all(xml.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create NoteOn event
    fn note_on(time: f64, note: u8, vel: u8, channel: u8) -> (f64, AudioEvent) {
        (time, AudioEvent::NoteOn { note, velocity: vel, channel })
    }

    // Helper to create NoteOff event
    fn note_off(time: f64, note: u8, channel: u8) -> (f64, AudioEvent) {
        (time, AudioEvent::NoteOff { note, channel })
    }

    #[test]
    fn test_fifths_from_key_c_major() {
        assert_eq!(fifths_from_key(0, false), 0); // C major = 0 sharps/flats
    }

    #[test]
    fn test_fifths_from_key_g_major() {
        assert_eq!(fifths_from_key(7, false), 1); // G major = 1 sharp
    }

    #[test]
    fn test_fifths_from_key_d_major() {
        assert_eq!(fifths_from_key(2, false), 2); // D major = 2 sharps
    }

    #[test]
    fn test_fifths_from_key_f_major() {
        assert_eq!(fifths_from_key(5, false), -1); // F major = 1 flat
    }

    #[test]
    fn test_fifths_from_key_bb_major() {
        assert_eq!(fifths_from_key(10, false), -2); // Bb major = 2 flats
    }

    #[test]
    fn test_fifths_from_key_a_minor() {
        // A minor (root=9) has same signature as C major (relative major)
        assert_eq!(fifths_from_key(9, true), 0);
    }

    #[test]
    fn test_fifths_from_key_e_minor() {
        // E minor (root=4) has same signature as G major (1 sharp)
        assert_eq!(fifths_from_key(4, true), 1);
    }

    #[test]
    fn test_fifths_from_key_d_minor() {
        // D minor (root=2) has same signature as F major (1 flat)
        assert_eq!(fifths_from_key(2, true), -1);
    }

    #[test]
    fn test_time_signature_16_steps() {
        assert_eq!(time_signature_from_steps(16), (4, 4));
    }

    #[test]
    fn test_time_signature_12_steps() {
        assert_eq!(time_signature_from_steps(12), (3, 4));
    }

    #[test]
    fn test_time_signature_24_steps() {
        assert_eq!(time_signature_from_steps(24), (6, 8));
    }

    #[test]
    fn test_pitch_c4() {
        let params = MusicalParams::default();
        let builder = MusicXmlBuilder::new(vec![], &params, 11025);
        let (step, alter, octave) = builder.midi_to_pitch(60);
        assert_eq!((step, alter, octave), ("C", 0, 4));
    }

    #[test]
    fn test_pitch_c_sharp_in_sharp_key() {
        let mut params = MusicalParams::default();
        params.key_root = 7; // G major (sharp key)
        params.harmony_valence = 0.5;
        let builder = MusicXmlBuilder::new(vec![], &params, 11025);
        let (step, alter, _) = builder.midi_to_pitch(61); // C#4
        assert_eq!((step, alter), ("C", 1)); // Should be C#, not Db
    }

    #[test]
    fn test_pitch_d_flat_in_flat_key() {
        let mut params = MusicalParams::default();
        params.key_root = 5; // F major (flat key)
        params.harmony_valence = 0.5;
        let builder = MusicXmlBuilder::new(vec![], &params, 11025);
        let (step, alter, _) = builder.midi_to_pitch(61); // Db4
        assert_eq!((step, alter), ("D", -1)); // Should be Db, not C#
    }

    #[test]
    fn test_duration_whole_note() {
        let params = MusicalParams::default(); // rhythm_steps = 16, divisions = 4
        let builder = MusicXmlBuilder::new(vec![], &params, 11025);
        let (note_type, dots) = builder.duration_to_type(16); // 16 steps = 4 quarters = whole
        assert_eq!((note_type, dots), ("whole", 0));
    }

    #[test]
    fn test_duration_quarter_note() {
        let params = MusicalParams::default();
        let builder = MusicXmlBuilder::new(vec![], &params, 11025);
        let (note_type, dots) = builder.duration_to_type(4); // 4 steps = 1 quarter
        assert_eq!((note_type, dots), ("quarter", 0));
    }

    #[test]
    fn test_duration_eighth_note() {
        let params = MusicalParams::default();
        let builder = MusicXmlBuilder::new(vec![], &params, 11025);
        let (note_type, dots) = builder.duration_to_type(2); // 2 steps = 0.5 quarter = eighth
        assert_eq!((note_type, dots), ("eighth", 0));
    }

    #[test]
    fn test_simple_note_produces_valid_xml() {
        let samples_per_step = 11025;
        let events = vec![note_on(0.0, 60, 100, 1), note_off(samples_per_step as f64, 60, 1)];

        let params = MusicalParams::default();
        let xml = to_musicxml(&events, &params, samples_per_step);

        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<score-partwise"));
        assert!(xml.contains("<part-list>"));
        assert!(xml.contains("<pitch>"));
        assert!(xml.contains("<step>C</step>"));
        assert!(xml.contains("<octave>4</octave>"));
    }

    #[test]
    fn test_bass_channel_uses_bass_clef() {
        let events = vec![
            note_on(0.0, 36, 100, 0), // Channel 0 = Bass
            note_off(11025.0, 36, 0),
        ];

        let params = MusicalParams::default();
        let xml = to_musicxml(&events, &params, 11025);

        // Check Bass part has F clef
        assert!(xml.contains(r#"<part id="P2">"#)); // Bass is P2
        assert!(xml.contains("<sign>F</sign>"));
        assert!(xml.contains("<line>4</line>"));
    }

    #[test]
    fn test_drums_use_percussion_clef() {
        let events = vec![
            note_on(0.0, 38, 100, 2), // Channel 2 = Snare
            note_off(11025.0, 38, 2),
        ];

        let params = MusicalParams::default();
        let xml = to_musicxml(&events, &params, 11025);

        assert!(xml.contains("<sign>percussion</sign>"));
        assert!(xml.contains("<unpitched>"));
        assert!(xml.contains("<display-step>E</display-step>")); // Snare on E
    }

    #[test]
    fn test_key_signature_g_major() {
        let events = vec![note_on(0.0, 67, 100, 1), note_off(11025.0, 67, 1)];

        let mut params = MusicalParams::default();
        params.key_root = 7; // G
        params.harmony_valence = 0.5; // Major

        let xml = to_musicxml(&events, &params, 11025);
        assert!(xml.contains("<fifths>1</fifths>")); // One sharp
        assert!(xml.contains("<mode>major</mode>"));
    }

    #[test]
    fn test_key_signature_d_minor() {
        let events = vec![note_on(0.0, 62, 100, 1), note_off(11025.0, 62, 1)];

        let mut params = MusicalParams::default();
        params.key_root = 2; // D
        params.harmony_valence = -0.5; // Minor

        let xml = to_musicxml(&events, &params, 11025);
        assert!(xml.contains("<fifths>-1</fifths>")); // One flat (relative major = F)
        assert!(xml.contains("<mode>minor</mode>"));
    }

    #[test]
    fn test_time_signature_3_4() {
        let events = vec![note_on(0.0, 60, 100, 1), note_off(11025.0, 60, 1)];

        let mut params = MusicalParams::default();
        params.rhythm_steps = 12; // 3/4 time

        let xml = to_musicxml(&events, &params, 11025);
        assert!(xml.contains("<beats>3</beats>"));
        assert!(xml.contains("<beat-type>4</beat-type>"));
    }

    #[test]
    fn test_chord_notation() {
        let events = vec![
            note_on(0.0, 60, 100, 1), // C
            note_on(0.0, 64, 100, 1), // E (same time = chord)
            note_on(0.0, 67, 100, 1), // G (same time = chord)
            note_off(11025.0, 60, 1),
            note_off(11025.0, 64, 1),
            note_off(11025.0, 67, 1),
        ];

        let xml = to_musicxml(&events, &MusicalParams::default(), 11025);

        // Second and third notes in chord should have <chord/> element
        let chord_count = xml.matches("<chord/>").count();
        assert_eq!(chord_count, 2);
    }

    #[test]
    fn test_rest_generation() {
        // Note starts on step 4 (after a rest)
        let samples_per_step = 11025;
        let samples_per_step_f64 = samples_per_step as f64;
        let events = vec![
            note_on(4.0 * samples_per_step_f64, 60, 100, 1),
            note_off(5.0 * samples_per_step_f64, 60, 1),
        ];

        let xml = to_musicxml(&events, &MusicalParams::default(), samples_per_step);
        assert!(xml.contains("<rest/>"));
    }

    #[test]
    fn test_empty_events_produces_valid_xml() {
        let events: Vec<(f64, AudioEvent)> = vec![];
        let xml = to_musicxml(&events, &MusicalParams::default(), 11025);

        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<score-partwise"));
        assert!(xml.contains("</score-partwise>"));
    }

    #[test]
    fn test_key_name() {
        assert_eq!(key_name(0, false), "C major");
        assert_eq!(key_name(7, false), "G major");
        assert_eq!(key_name(5, false), "F major");
        assert_eq!(key_name(9, true), "A minor");
        assert_eq!(key_name(4, true), "E minor");
    }
}

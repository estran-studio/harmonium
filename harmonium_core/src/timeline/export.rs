//! MusicXML export from ScoreTimeline
//!
//! Generates MusicXML directly from `Measure` structs with explicit note durations.
//! This replaces the legacy NoteOn/NoteOff-pair reconstruction approach.

use std::fmt::Write;

use super::{Measure, ScoreTimeline, TimelineNote, TrackId};
use crate::params::InstrumentConfig;

/// MIDI note number to MusicXML pitch mapping
struct MxlPitch {
    step: &'static str,
    alter: i32,
    octave: i32,
}

/// Convert MIDI note number to MusicXML pitch components
fn midi_to_pitch(midi: u8) -> MxlPitch {
    let octave = (midi as i32 / 12) - 1;
    let pc = midi % 12;
    let (step, alter) = match pc {
        0 => ("C", 0),
        1 => ("C", 1),
        2 => ("D", 0),
        3 => ("D", 1),
        4 => ("E", 0),
        5 => ("F", 0),
        6 => ("F", 1),
        7 => ("G", 0),
        8 => ("G", 1),
        9 => ("A", 0),
        10 => ("A", 1),
        11 => ("B", 0),
        _ => ("C", 0),
    };
    MxlPitch { step, alter, octave }
}

/// Convert step duration to MusicXML note type and duration divisions.
///
/// Assumes 4 ticks per beat (16th note resolution) and `divisions = 4`.
/// Duration in divisions = duration_steps (since 1 step = 1 division).
fn step_duration_to_type(duration_steps: usize) -> (&'static str, bool) {
    match duration_steps {
        0 | 1 => ("16th", false),
        2 => ("eighth", false),
        3 => ("eighth", true), // dotted eighth
        4 => ("quarter", false),
        6 => ("quarter", true), // dotted quarter
        8 => ("half", false),
        12 => ("half", true), // dotted half
        16 => ("whole", false),
        _ => {
            // Best approximation
            if duration_steps <= 2 {
                ("16th", false)
            } else if duration_steps <= 4 {
                ("eighth", false)
            } else if duration_steps <= 8 {
                ("quarter", false)
            } else if duration_steps <= 16 {
                ("half", false)
            } else {
                ("whole", false)
            }
        }
    }
}

/// Export a ScoreTimeline to MusicXML
///
/// Generates a complete MusicXML document with separate parts for each track.
/// Notes have explicit durations from the timeline (no reconstruction needed).
///
/// When `instrument_lead` or `instrument_bass` have non-zero transposition,
/// the exported MusicXML includes `<transpose>` elements so notation software
/// (MuseScore, Finale, etc.) renders the correct written pitch for transposing
/// instruments (e.g., tenor sax, trumpet).
pub fn timeline_to_musicxml(timeline: &ScoreTimeline, title: &str) -> String {
    timeline_to_musicxml_with_instruments(
        timeline,
        title,
        &InstrumentConfig::default(),
        &InstrumentConfig::default(),
    )
}

/// Export a ScoreTimeline to MusicXML with instrument transposition support.
///
/// Part names and `<transpose>` elements are derived from the configs.
pub fn timeline_to_musicxml_with_instruments(
    timeline: &ScoreTimeline,
    title: &str,
    instrument_lead: &InstrumentConfig,
    instrument_bass: &InstrumentConfig,
) -> String {
    let measures = timeline.measures();
    if measures.is_empty() {
        return empty_musicxml(title);
    }

    let mut xml = String::with_capacity(measures.len() * 2000);

    // XML header
    write_header(&mut xml, title);

    // Part names (use instrument name if available)
    let bass_name = instrument_bass.instrument_name().unwrap_or("Bass");
    let lead_name = instrument_lead.instrument_name().unwrap_or("Lead");

    // Part list
    writeln!(xml, "  <part-list>").unwrap();
    writeln!(xml, "    <score-part id=\"P1\"><part-name>{bass_name}</part-name></score-part>")
        .unwrap();
    writeln!(xml, "    <score-part id=\"P2\"><part-name>{lead_name}</part-name></score-part>")
        .unwrap();
    writeln!(xml, "    <score-part id=\"P3\"><part-name>Chords</part-name></score-part>").unwrap();
    writeln!(xml, "    <score-part id=\"P4\"><part-name>Drums</part-name></score-part>").unwrap();
    writeln!(xml, "  </part-list>").unwrap();

    // Bass part
    write_part(&mut xml, "P1", measures, TrackId::Bass, instrument_bass);

    // Lead / Melody part
    write_part(&mut xml, "P2", measures, TrackId::Melody, instrument_lead);

    // Chords part
    write_part(&mut xml, "P3", measures, TrackId::Chords, &InstrumentConfig::default());

    // Drums part (combined heel + toe)
    write_drum_part(&mut xml, "P4", measures);

    writeln!(xml, "</score-partwise>").unwrap();
    xml
}

fn write_header(xml: &mut String, title: &str) {
    writeln!(xml, r#"<?xml version="1.0" encoding="UTF-8"?>"#).unwrap();
    writeln!(xml, r#"<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 4.0 Partwise//EN" "http://www.musicxml.org/dtds/partwise.dtd">"#).unwrap();
    writeln!(xml, r#"<score-partwise version="4.0">"#).unwrap();
    writeln!(xml, "  <work>").unwrap();
    writeln!(xml, "    <work-title>{}</work-title>", xml_escape(title)).unwrap();
    writeln!(xml, "  </work>").unwrap();
    writeln!(xml, "  <identification>").unwrap();
    writeln!(xml, "    <creator type=\"composer\">Harmonium</creator>").unwrap();
    writeln!(xml, "    <encoding>").unwrap();
    writeln!(xml, "      <software>Harmonium Timeline Engine</software>").unwrap();
    writeln!(xml, "    </encoding>").unwrap();
    writeln!(xml, "  </identification>").unwrap();
}

fn write_part(
    xml: &mut String,
    part_id: &str,
    measures: &[Measure],
    track: TrackId,
    instrument: &InstrumentConfig,
) {
    writeln!(xml, "  <part id=\"{part_id}\">").unwrap();

    for (i, measure) in measures.iter().enumerate() {
        writeln!(xml, "    <measure number=\"{}\">", i + 1).unwrap();

        // Attributes on first measure
        if i == 0 {
            writeln!(xml, "      <attributes>").unwrap();
            writeln!(xml, "        <divisions>4</divisions>").unwrap();
            writeln!(xml, "        <time>").unwrap();
            writeln!(xml, "          <beats>{}</beats>", measure.time_signature.numerator).unwrap();
            writeln!(
                xml,
                "          <beat-type>{}</beat-type>",
                measure.time_signature.denominator
            )
            .unwrap();
            writeln!(xml, "        </time>").unwrap();
            writeln!(xml, "        <clef>").unwrap();
            if track == TrackId::Bass {
                writeln!(xml, "          <sign>F</sign>").unwrap();
                writeln!(xml, "          <line>4</line>").unwrap();
            } else {
                writeln!(xml, "          <sign>G</sign>").unwrap();
                writeln!(xml, "          <line>2</line>").unwrap();
            }
            writeln!(xml, "        </clef>").unwrap();
            // Transposition for transposing instruments (e.g., tenor sax, trumpet)
            if let Some((chromatic, diatonic)) = instrument.musicxml_transpose() {
                writeln!(xml, "        <transpose>").unwrap();
                writeln!(xml, "          <diatonic>{diatonic}</diatonic>").unwrap();
                writeln!(xml, "          <chromatic>{chromatic}</chromatic>").unwrap();
                writeln!(xml, "        </transpose>").unwrap();
            }
            writeln!(xml, "      </attributes>").unwrap();

            // Tempo marking
            writeln!(xml, "      <direction placement=\"above\">").unwrap();
            writeln!(xml, "        <direction-type>").unwrap();
            writeln!(xml, "          <metronome>").unwrap();
            writeln!(xml, "            <beat-unit>quarter</beat-unit>").unwrap();
            writeln!(xml, "            <per-minute>{}</per-minute>", measure.tempo as u32).unwrap();
            writeln!(xml, "          </metronome>").unwrap();
            writeln!(xml, "        </direction-type>").unwrap();
            writeln!(xml, "        <sound tempo=\"{}\"/>", measure.tempo as u32).unwrap();
            writeln!(xml, "      </direction>").unwrap();
        }

        // Chord annotation
        if !measure.chord_context.chord_name.is_empty() {
            writeln!(xml, "      <direction placement=\"above\">").unwrap();
            writeln!(xml, "        <direction-type>").unwrap();
            writeln!(
                xml,
                "          <words>{}</words>",
                xml_escape(&measure.chord_context.chord_name)
            )
            .unwrap();
            writeln!(xml, "        </direction-type>").unwrap();
            writeln!(xml, "      </direction>").unwrap();
        }

        let notes = measure.notes_for_track(track);
        write_notes_for_measure(xml, notes, measure.steps);

        writeln!(xml, "    </measure>").unwrap();
    }

    writeln!(xml, "  </part>").unwrap();
}

fn write_drum_part(xml: &mut String, part_id: &str, measures: &[Measure]) {
    writeln!(xml, "  <part id=\"{part_id}\">").unwrap();

    for (i, measure) in measures.iter().enumerate() {
        writeln!(xml, "    <measure number=\"{}\">", i + 1).unwrap();

        if i == 0 {
            writeln!(xml, "      <attributes>").unwrap();
            writeln!(xml, "        <divisions>4</divisions>").unwrap();
            writeln!(xml, "        <time>").unwrap();
            writeln!(xml, "          <beats>{}</beats>", measure.time_signature.numerator).unwrap();
            writeln!(
                xml,
                "          <beat-type>{}</beat-type>",
                measure.time_signature.denominator
            )
            .unwrap();
            writeln!(xml, "        </time>").unwrap();
            writeln!(xml, "        <clef>").unwrap();
            writeln!(xml, "          <sign>percussion</sign>").unwrap();
            writeln!(xml, "        </clef>").unwrap();
            writeln!(xml, "      </attributes>").unwrap();
        }

        // Merge heel and toe notes by start_step
        let heel_notes = measure.notes_for_track(TrackId::Heel);
        let toe_notes = measure.notes_for_track(TrackId::Toe);

        let mut all_notes: Vec<&TimelineNote> = heel_notes.iter().chain(toe_notes.iter()).collect();
        all_notes.sort_by_key(|n| n.start_step);

        if all_notes.is_empty() {
            // Full measure rest
            let total_duration = measure.steps;
            writeln!(xml, "      <note>").unwrap();
            writeln!(xml, "        <rest/>").unwrap();
            writeln!(xml, "        <duration>{total_duration}</duration>").unwrap();
            writeln!(xml, "      </note>").unwrap();
        } else {
            let mut current_step = 0;
            for note in &all_notes {
                // Rest before this note
                if note.start_step > current_step {
                    let rest_dur = note.start_step - current_step;
                    writeln!(xml, "      <note>").unwrap();
                    writeln!(xml, "        <rest/>").unwrap();
                    writeln!(xml, "        <duration>{rest_dur}</duration>").unwrap();
                    writeln!(xml, "      </note>").unwrap();
                }

                // Unpitched percussion note
                writeln!(xml, "      <note>").unwrap();
                writeln!(xml, "        <unpitched>").unwrap();
                writeln!(xml, "          <display-step>E</display-step>").unwrap();
                writeln!(xml, "          <display-octave>4</display-octave>").unwrap();
                writeln!(xml, "        </unpitched>").unwrap();
                let dur = note.duration_steps.max(1);
                writeln!(xml, "        <duration>{dur}</duration>").unwrap();
                let (note_type, _dotted) = step_duration_to_type(dur);
                writeln!(xml, "        <type>{note_type}</type>").unwrap();
                writeln!(xml, "      </note>").unwrap();

                current_step = note.start_step + dur;
            }

            // Trailing rest
            if current_step < measure.steps {
                let rest_dur = measure.steps - current_step;
                writeln!(xml, "      <note>").unwrap();
                writeln!(xml, "        <rest/>").unwrap();
                writeln!(xml, "        <duration>{rest_dur}</duration>").unwrap();
                writeln!(xml, "      </note>").unwrap();
            }
        }

        writeln!(xml, "    </measure>").unwrap();
    }

    writeln!(xml, "  </part>").unwrap();
}

fn write_notes_for_measure(xml: &mut String, notes: &[TimelineNote], total_steps: usize) {
    if notes.is_empty() {
        // Full measure rest
        writeln!(xml, "      <note>").unwrap();
        writeln!(xml, "        <rest/>").unwrap();
        writeln!(xml, "        <duration>{total_steps}</duration>").unwrap();
        writeln!(xml, "      </note>").unwrap();
        return;
    }

    let mut current_step = 0;

    for note in notes {
        // Insert rest if there's a gap
        if note.start_step > current_step {
            let rest_duration = note.start_step - current_step;
            writeln!(xml, "      <note>").unwrap();
            writeln!(xml, "        <rest/>").unwrap();
            writeln!(xml, "        <duration>{rest_duration}</duration>").unwrap();
            writeln!(xml, "      </note>").unwrap();
        }

        let duration = note.duration_steps.max(1);
        let pitch = midi_to_pitch(note.pitch);
        let (note_type, dotted) = step_duration_to_type(duration);

        writeln!(xml, "      <note>").unwrap();
        writeln!(xml, "        <pitch>").unwrap();
        writeln!(xml, "          <step>{}</step>", pitch.step).unwrap();
        if pitch.alter != 0 {
            writeln!(xml, "          <alter>{}</alter>", pitch.alter).unwrap();
        }
        writeln!(xml, "          <octave>{}</octave>", pitch.octave).unwrap();
        writeln!(xml, "        </pitch>").unwrap();
        writeln!(xml, "        <duration>{duration}</duration>").unwrap();
        writeln!(xml, "        <type>{note_type}</type>").unwrap();
        if dotted {
            writeln!(xml, "        <dot/>").unwrap();
        }
        // Dynamics from velocity
        if note.velocity > 0 {
            let dynamics = velocity_to_dynamics(note.velocity);
            writeln!(xml, "        <dynamics><{dynamics}/></dynamics>").unwrap();
        }
        writeln!(xml, "      </note>").unwrap();

        current_step = note.start_step + duration;
    }

    // Trailing rest if notes don't fill the measure
    if current_step < total_steps {
        let rest_duration = total_steps - current_step;
        writeln!(xml, "      <note>").unwrap();
        writeln!(xml, "        <rest/>").unwrap();
        writeln!(xml, "        <duration>{rest_duration}</duration>").unwrap();
        writeln!(xml, "      </note>").unwrap();
    }
}

fn velocity_to_dynamics(velocity: u8) -> &'static str {
    match velocity {
        0..=31 => "pp",
        32..=63 => "p",
        64..=89 => "mp",
        90..=110 => "mf",
        111..=126 => "f",
        127 => "ff",
        _ => "mf",
    }
}

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

fn empty_musicxml(title: &str) -> String {
    let mut xml = String::new();
    write_header(&mut xml, title);
    writeln!(xml, "  <part-list>").unwrap();
    writeln!(xml, "    <score-part id=\"P1\"><part-name>Empty</part-name></score-part>").unwrap();
    writeln!(xml, "  </part-list>").unwrap();
    writeln!(xml, "  <part id=\"P1\">").unwrap();
    writeln!(xml, "    <measure number=\"1\">").unwrap();
    writeln!(xml, "      <attributes>").unwrap();
    writeln!(xml, "        <divisions>4</divisions>").unwrap();
    writeln!(xml, "      </attributes>").unwrap();
    writeln!(xml, "      <note><rest/><duration>16</duration></note>").unwrap();
    writeln!(xml, "    </measure>").unwrap();
    writeln!(xml, "  </part>").unwrap();
    writeln!(xml, "</score-partwise>").unwrap();
    xml
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{params::TimeSignature, timeline::Articulation};

    #[test]
    fn test_midi_to_pitch() {
        let p = midi_to_pitch(60); // Middle C
        assert_eq!(p.step, "C");
        assert_eq!(p.alter, 0);
        assert_eq!(p.octave, 4);

        let p = midi_to_pitch(61); // C#4
        assert_eq!(p.step, "C");
        assert_eq!(p.alter, 1);
        assert_eq!(p.octave, 4);

        let p = midi_to_pitch(36); // C2
        assert_eq!(p.step, "C");
        assert_eq!(p.octave, 2);
    }

    #[test]
    fn test_step_duration_to_type() {
        assert_eq!(step_duration_to_type(1), ("16th", false));
        assert_eq!(step_duration_to_type(2), ("eighth", false));
        assert_eq!(step_duration_to_type(4), ("quarter", false));
        assert_eq!(step_duration_to_type(8), ("half", false));
        assert_eq!(step_duration_to_type(16), ("whole", false));
        assert_eq!(step_duration_to_type(6), ("quarter", true)); // dotted quarter
    }

    #[test]
    fn test_empty_timeline_export() {
        let timeline = ScoreTimeline::with_default_capacity();
        let xml = timeline_to_musicxml(&timeline, "Test");
        assert!(xml.contains("score-partwise"));
        assert!(xml.contains("Empty"));
    }

    #[test]
    fn test_timeline_with_notes_export() {
        let mut timeline = ScoreTimeline::new(10);
        let mut measure = Measure::new(1, TimeSignature::default(), 120.0, 16);
        measure.add_note(
            TrackId::Bass,
            TimelineNote {
                id: 1,
                pitch: 36,
                start_step: 0,
                duration_steps: 4,
                velocity: 100,
                articulation: Articulation::Normal,
            },
        );
        measure.add_note(
            TrackId::Melody,
            TimelineNote {
                id: 2,
                pitch: 60,
                start_step: 0,
                duration_steps: 8,
                velocity: 80,
                articulation: Articulation::Normal,
            },
        );
        timeline.push_measure(measure);

        let xml = timeline_to_musicxml(&timeline, "Test Song");
        assert!(xml.contains("Test Song"));
        assert!(xml.contains("<step>C</step>"));
        assert!(xml.contains("P1")); // Bass part
        assert!(xml.contains("P2")); // Lead part
        assert!(xml.contains("P3")); // Chords part
        assert!(xml.contains("P4")); // Drums part
    }

    #[test]
    fn test_tenor_sax_export_has_transpose() {
        let mut timeline = ScoreTimeline::new(10);
        let mut measure = Measure::new(1, TimeSignature::default(), 120.0, 16);
        measure.add_note(
            TrackId::Melody,
            TimelineNote {
                id: 1,
                pitch: 62, // D4 (written pitch for tenor sax playing concert C4)
                start_step: 0,
                duration_steps: 4,
                velocity: 80,
                articulation: Articulation::Normal,
            },
        );
        timeline.push_measure(measure);

        let tenor = InstrumentConfig::tenor_sax();
        let xml = timeline_to_musicxml_with_instruments(
            &timeline,
            "Tenor Sax Test",
            &tenor,
            &InstrumentConfig::default(),
        );

        // Part name should be "Tenor Saxophone"
        assert!(xml.contains("Tenor Saxophone"), "Expected 'Tenor Saxophone' in part name");

        // Should have <transpose> element with chromatic=-2 (Bb instrument)
        assert!(xml.contains("<transpose>"), "Expected <transpose> element");
        assert!(
            xml.contains("<chromatic>-2</chromatic>"),
            "Expected chromatic=-2 for Bb instrument"
        );
        assert!(xml.contains("<diatonic>-1</diatonic>"), "Expected diatonic=-1 for Bb instrument");

        // The note D4 should be written as-is (written pitch)
        assert!(xml.contains("<step>D</step>"), "Expected written pitch D");
    }

    #[test]
    fn test_alto_sax_export_has_transpose() {
        let mut timeline = ScoreTimeline::new(10);
        let mut measure = Measure::new(1, TimeSignature::default(), 120.0, 16);
        measure.add_note(
            TrackId::Melody,
            TimelineNote {
                id: 1,
                pitch: 57, // A3 (written pitch for alto sax)
                start_step: 0,
                duration_steps: 4,
                velocity: 80,
                articulation: Articulation::Normal,
            },
        );
        timeline.push_measure(measure);

        let alto = InstrumentConfig::alto_sax();
        let xml = timeline_to_musicxml_with_instruments(
            &timeline,
            "Alto Sax Test",
            &alto,
            &InstrumentConfig::default(),
        );

        assert!(xml.contains("Alto Saxophone"));
        assert!(xml.contains("<chromatic>3</chromatic>"), "Expected chromatic=3 for Eb instrument");
        assert!(
            xml.contains("<diatonic>2</diatonic>"),
            "Expected diatonic=2 for Eb instrument (minor 3rd)"
        );
    }

    #[test]
    fn test_default_config_no_transpose() {
        let mut timeline = ScoreTimeline::new(10);
        let mut measure = Measure::new(1, TimeSignature::default(), 120.0, 16);
        measure.add_note(
            TrackId::Melody,
            TimelineNote {
                id: 1,
                pitch: 60,
                start_step: 0,
                duration_steps: 4,
                velocity: 80,
                articulation: Articulation::Normal,
            },
        );
        timeline.push_measure(measure);

        let xml = timeline_to_musicxml(&timeline, "Concert Pitch Test");

        // Default config: no transpose element, standard part names
        assert!(!xml.contains("<transpose>"), "Default config should not have <transpose>");
        assert!(xml.contains("<part-name>Lead</part-name>"));
        assert!(xml.contains("<part-name>Bass</part-name>"));
    }

    #[test]
    fn test_velocity_to_dynamics() {
        assert_eq!(velocity_to_dynamics(20), "pp");
        assert_eq!(velocity_to_dynamics(50), "p");
        assert_eq!(velocity_to_dynamics(80), "mp");
        assert_eq!(velocity_to_dynamics(100), "mf");
        assert_eq!(velocity_to_dynamics(120), "f");
        assert_eq!(velocity_to_dynamics(127), "ff");
    }
}

//! MIDI (Standard MIDI File) export from ScoreTimeline
//!
//! Generates a Type 1 (parallel tracks) MIDI file from `Measure` structs.
//! Uses 480 ticks per quarter note. Each step in the timeline corresponds
//! to 120 MIDI ticks (16th note at 480 PPQ with 4 ticks/beat).

use std::path::Path;

use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind,
    num::{u4, u7, u15, u24, u28},
};

use super::{Measure, ScoreTimeline, TrackId};

/// Ticks per quarter note in the output MIDI file.
const TICKS_PER_QUARTER: u16 = 480;

/// MIDI ticks per timeline step (480 PPQ / 4 ticks_per_beat = 120).
const TICKS_PER_STEP: u32 = 120;

/// Default MIDI velocity for notes with velocity 0 (trigger events).
const DEFAULT_TRIGGER_VELOCITY: u8 = 80;

/// General MIDI percussion channel (0-indexed).
const GM_DRUM_CHANNEL: u8 = 9;

/// GM percussion note for snare drum.
const GM_SNARE: u8 = 38;

/// GM percussion note for closed hi-hat.
const GM_HIHAT: u8 = 42;

/// Convert a ScoreTimeline to a Standard MIDI File byte vector.
///
/// Produces parallel tracks from the canonical track list, plus a tempo map.
pub fn timeline_to_midi(
    timeline: &ScoreTimeline,
    profile: &crate::StyleProfile,
) -> std::io::Result<Vec<u8>> {
    let measures = timeline.measures();
    if measures.is_empty() {
        return empty_midi();
    }

    let header = Header::new(Format::Parallel, Timing::Metrical(u15::new(TICKS_PER_QUARTER)));
    let mut smf = Smf::new(header);

    // Track 0: tempo map + time signatures
    smf.tracks.push(build_tempo_track(measures));

    for track in TrackId::ALL {
        if track == TrackId::Heel {
            smf.tracks.push(build_drum_track(measures, profile.mix.tracks.toe.midi()));
            continue;
        }
        if track == TrackId::Toe {
            continue;
        }
        smf.tracks.push(build_instrument_track(
            measures,
            track,
            track.channel(),
            track.name(),
            profile.mix.tracks.get(track).midi(),
            profile.instruments.program(track),
        ));
    }

    let mut buf = Vec::new();
    smf.write_std(&mut buf)?;
    Ok(buf)
}

/// Write a ScoreTimeline to a MIDI file.
pub fn write_midi(
    timeline: &ScoreTimeline,
    profile: &crate::StyleProfile,
    path: &Path,
) -> std::io::Result<()> {
    let bytes = timeline_to_midi(timeline, profile)?;
    std::fs::write(path, bytes)
}

/// Produce a minimal empty MIDI file.
fn empty_midi() -> std::io::Result<Vec<u8>> {
    let header = Header::new(Format::Parallel, Timing::Metrical(u15::new(TICKS_PER_QUARTER)));
    let mut smf = Smf::new(header);
    // Single empty track with just EndOfTrack
    smf.tracks.push(vec![TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    }]);
    let mut buf = Vec::new();
    smf.write_std(&mut buf)?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Tempo track (track 0)
// ---------------------------------------------------------------------------

fn build_tempo_track(measures: &[Measure]) -> Vec<TrackEvent<'static>> {
    let mut events: Vec<TrackEvent<'static>> = Vec::new();
    let mut current_tick: u32 = 0;
    let mut last_tempo: Option<u32> = None;
    let mut last_time_sig: Option<(usize, usize)> = None;

    // Track name
    events.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(b"Tempo Map")),
    });

    for measure in measures {
        let measure_start_tick = measure_start(measure);

        // Time signature (emit on change or first measure)
        let ts = (measure.time_signature.numerator, measure.time_signature.denominator);
        if last_time_sig != Some(ts) {
            let delta = measure_start_tick.saturating_sub(current_tick);
            events.push(TrackEvent {
                delta: u28::new(delta),
                kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
                    ts.0 as u8,
                    denom_to_power(ts.1),
                    24, // MIDI clocks per metronome click
                    8,  // 32nd notes per quarter note
                )),
            });
            current_tick = measure_start_tick;
            last_time_sig = Some(ts);
        }

        // Tempo (emit on change or first measure)
        let tempo_uspq = bpm_to_microseconds_per_quarter(measure.tempo);
        if last_tempo != Some(tempo_uspq) {
            let delta = measure_start_tick.saturating_sub(current_tick);
            events.push(TrackEvent {
                delta: u28::new(delta),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(tempo_uspq))),
            });
            current_tick = measure_start_tick;
            last_tempo = Some(tempo_uspq);
        }
    }

    // End of track
    events.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    events
}

// ---------------------------------------------------------------------------
fn mix_events(
    channel: u8,
    mix: crate::mix::MidiTrackMix,
) -> impl Iterator<Item = TrackEvent<'static>> {
    mix.controllers().into_iter().map(move |(controller, value)| TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Midi {
            channel: channel.into(),
            message: MidiMessage::Controller { controller: controller.into(), value: value.into() },
        },
    })
}

// Instrument tracks (bass, lead)
// ---------------------------------------------------------------------------

fn build_instrument_track(
    measures: &[Measure],
    track_id: TrackId,
    channel: u8,
    name: &'static str,
    mix: crate::mix::MidiTrackMix,
    program: Option<u8>,
) -> Vec<TrackEvent<'static>> {
    let mut events: Vec<(u32, TrackEventKind<'static>)> = Vec::new();

    // Collect all note events with absolute tick positions
    for measure in measures {
        let base_tick = measure_start(measure);
        for note in measure.notes_for_track(track_id) {
            let (on_tick, off_tick) = note_ticks(base_tick, note);
            let vel = if note.velocity == 0 { DEFAULT_TRIGGER_VELOCITY } else { note.velocity };
            // Même gate explicite que la lecture ; arrondi au tick MIDI.

            events.push((
                on_tick,
                TrackEventKind::Midi {
                    channel: u4::new(channel),
                    message: MidiMessage::NoteOn { key: u7::new(note.pitch), vel: u7::new(vel) },
                },
            ));
            events.push((
                off_tick,
                TrackEventKind::Midi {
                    channel: u4::new(channel),
                    message: MidiMessage::NoteOff { key: u7::new(note.pitch), vel: u7::new(0) },
                },
            ));
        }
    }

    // Sort by tick, NoteOff before NoteOn at same tick
    events.sort_by(|a, b| {
        a.0.cmp(&b.0).then_with(|| {
            let a_off =
                matches!(a.1, TrackEventKind::Midi { message: MidiMessage::NoteOff { .. }, .. });
            let b_off =
                matches!(b.1, TrackEventKind::Midi { message: MidiMessage::NoteOff { .. }, .. });
            b_off.cmp(&a_off)
        })
    });

    // Convert to delta-time TrackEvents
    let mut track: Vec<TrackEvent<'static>> = Vec::with_capacity(events.len() + 2);

    // Track name
    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(name.as_bytes())),
    });
    track.extend(mix_events(channel, mix));
    if let Some(program) = program {
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::ProgramChange { program: program.into() },
            },
        });
    }

    let mut prev_tick: u32 = 0;
    for (tick, kind) in events {
        let delta = tick.saturating_sub(prev_tick);
        track.push(TrackEvent { delta: u28::new(delta), kind });
        prev_tick = tick;
    }

    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    track
}

// ---------------------------------------------------------------------------
// Drum track (snare + hat on GM channel 9)
// ---------------------------------------------------------------------------

fn build_drum_track(
    measures: &[Measure],
    mix: crate::mix::MidiTrackMix,
) -> Vec<TrackEvent<'static>> {
    let mut events: Vec<(u32, TrackEventKind<'static>)> = Vec::new();

    for measure in measures {
        let base_tick = measure_start(measure);

        // Heel
        for note in measure.notes_for_track(TrackId::Heel) {
            let (on_tick, off_tick) = note_ticks(base_tick, note);
            let vel = if note.velocity == 0 { DEFAULT_TRIGGER_VELOCITY } else { note.velocity };
            let key = if note.pitch != 0 { note.pitch } else { GM_SNARE };
            events.push((
                on_tick,
                TrackEventKind::Midi {
                    channel: u4::new(GM_DRUM_CHANNEL),
                    message: MidiMessage::NoteOn { key: u7::new(key), vel: u7::new(vel) },
                },
            ));
            events.push((
                off_tick,
                TrackEventKind::Midi {
                    channel: u4::new(GM_DRUM_CHANNEL),
                    message: MidiMessage::NoteOff { key: u7::new(key), vel: u7::new(0) },
                },
            ));
        }

        // Toe
        for note in measure.notes_for_track(TrackId::Toe) {
            let (on_tick, off_tick) = note_ticks(base_tick, note);
            let vel = if note.velocity == 0 { DEFAULT_TRIGGER_VELOCITY } else { note.velocity };
            let key = if note.pitch != 0 { note.pitch } else { GM_HIHAT };
            events.push((
                on_tick,
                TrackEventKind::Midi {
                    channel: u4::new(GM_DRUM_CHANNEL),
                    message: MidiMessage::NoteOn { key: u7::new(key), vel: u7::new(vel) },
                },
            ));
            events.push((
                off_tick,
                TrackEventKind::Midi {
                    channel: u4::new(GM_DRUM_CHANNEL),
                    message: MidiMessage::NoteOff { key: u7::new(key), vel: u7::new(0) },
                },
            ));
        }
    }

    // Sort by tick, NoteOff before NoteOn at same tick
    events.sort_by(|a, b| {
        a.0.cmp(&b.0).then_with(|| {
            let a_off =
                matches!(a.1, TrackEventKind::Midi { message: MidiMessage::NoteOff { .. }, .. });
            let b_off =
                matches!(b.1, TrackEventKind::Midi { message: MidiMessage::NoteOff { .. }, .. });
            b_off.cmp(&a_off)
        })
    });

    let mut track: Vec<TrackEvent<'static>> = Vec::with_capacity(events.len() + 2);

    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(b"Drums")),
    });
    track.extend(mix_events(GM_DRUM_CHANNEL, mix));

    let mut prev_tick: u32 = 0;
    for (tick, kind) in events {
        let delta = tick.saturating_sub(prev_tick);
        track.push(TrackEvent { delta: u28::new(delta), kind });
        prev_tick = tick;
    }

    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    track
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn note_ticks(base: u32, note: &super::TimelineNote) -> (u32, u32) {
    let start = i64::from(base) * 1000
        + (note.start_step as i64 * 1000 + i64::from(note.performance.offset))
            * i64::from(TICKS_PER_STEP);
    let end = start + note.performance.duration(note.duration_steps) * i64::from(TICKS_PER_STEP);
    let on = ((start + 500) / 1000).max(0) as u32;
    let off = (((end + 500) / 1000).max(0) as u32).max(on + 1);
    (on, off)
}

/// Calculate the absolute MIDI tick for the start of a measure.
///
/// Since measures can have different step counts (different time signatures),
/// we compute from measure index and step count.
/// For now assumes all preceding measures have the same step count (simple case).
/// TODO: Handle mixed time signatures by accumulating from measure 0.
fn measure_start(measure: &Measure) -> u32 {
    ((measure.index - 1) as u32) * (measure.steps as u32) * TICKS_PER_STEP
}

/// Convert BPM to microseconds per quarter note for MIDI tempo meta event.
fn bpm_to_microseconds_per_quarter(bpm: f32) -> u32 {
    if bpm <= 0.0 {
        500_000 // Default: 120 BPM
    } else {
        (60_000_000.0 / bpm as f64) as u32
    }
}

/// Convert time signature denominator to its power-of-2 representation.
/// MIDI time signature uses log2(denominator): 4 → 2, 8 → 3, etc.
fn denom_to_power(denom: usize) -> u8 {
    match denom {
        1 => 0,
        2 => 1,
        4 => 2,
        8 => 3,
        16 => 4,
        _ => 2, // Default to quarter note
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        params::TimeSignature,
        timeline::{Articulation, ChordContext, StateSnapshot, TimelineNote},
    };

    /// Helper: create a minimal timeline with known content.
    fn make_test_timeline() -> ScoreTimeline {
        let mut timeline = ScoreTimeline::new(10);

        let mut measure = Measure::new(1, TimeSignature::new(4, 4), 120.0, 16);
        measure.chord_context = ChordContext {
            root_offset: 0,
            is_minor: false,
            chord_name: "I".to_string(),
            ..Default::default()
        };
        measure.state_snapshot = StateSnapshot {
            bpm: 120.0,
            density: 0.5,
            tension: 0.3,
            smoothness: 0.7,
            valence: 0.2,
            arousal: 0.5,
        };

        // Bass: kick on beats 1 and 3
        measure.add_note(
            TrackId::Bass,
            TimelineNote {
                id: 1,
                pitch: 36,
                start_step: 0,
                duration_steps: 1,
                velocity: 96,
                articulation: Articulation::Detached,
                performance: Default::default(),
            },
        );
        measure.add_note(
            TrackId::Bass,
            TimelineNote {
                id: 2,
                pitch: 36,
                start_step: 8,
                duration_steps: 1,
                velocity: 96,
                articulation: Articulation::Detached,
                performance: Default::default(),
            },
        );

        // Lead: quarter note melody
        for (i, pitch) in [60u8, 64, 67, 72].iter().enumerate() {
            measure.add_note(
                TrackId::Melody,
                TimelineNote {
                    id: 10 + i as u64,
                    pitch: *pitch,
                    start_step: i * 4,
                    duration_steps: 4,
                    velocity: 80,
                    articulation: Articulation::Normal,
                    performance: Default::default(),
                },
            );
        }

        // Snare on beat 2 and 4
        measure.add_note(
            TrackId::Heel,
            TimelineNote {
                id: 20,
                pitch: 38,
                start_step: 4,
                duration_steps: 0,
                velocity: 100,
                articulation: Articulation::Trigger,
                performance: Default::default(),
            },
        );
        measure.add_note(
            TrackId::Heel,
            TimelineNote {
                id: 21,
                pitch: 38,
                start_step: 12,
                duration_steps: 0,
                velocity: 100,
                articulation: Articulation::Trigger,
                performance: Default::default(),
            },
        );

        // Hat on every beat
        for step in (0..16).step_by(4) {
            measure.add_note(
                TrackId::Toe,
                TimelineNote {
                    id: 30 + step as u64 / 4,
                    pitch: 42,
                    start_step: step,
                    duration_steps: 0,
                    velocity: 70,
                    articulation: Articulation::Trigger,
                    performance: Default::default(),
                },
            );
        }

        timeline.push_measure(measure);
        timeline
    }

    #[test]
    fn test_empty_timeline_produces_valid_midi() {
        let timeline = ScoreTimeline::new(10);
        let bytes = timeline_to_midi(&timeline, &crate::StyleProfile::builtin()).unwrap();
        assert!(!bytes.is_empty());
        // Should start with MIDI header "MThd"
        assert_eq!(&bytes[0..4], b"MThd");
    }

    #[test]
    fn test_single_measure_roundtrip() {
        let timeline = make_test_timeline();
        let bytes = timeline_to_midi(&timeline, &crate::StyleProfile::builtin()).unwrap();

        // Parse it back
        let smf = Smf::parse(&bytes).expect("generated MIDI should parse");

        // Verify header
        assert_eq!(smf.header.format, Format::Parallel);
        assert_eq!(smf.header.timing, Timing::Metrical(u15::new(TICKS_PER_QUARTER)));
    }

    #[test]
    fn test_note_counts() {
        let timeline = make_test_timeline();
        let bytes = timeline_to_midi(&timeline, &crate::StyleProfile::builtin()).unwrap();
        let smf = Smf::parse(&bytes).expect("generated MIDI should parse");

        // Count NoteOn events per track (excluding track 0 which is tempo)
        let count_note_ons = |track: &[TrackEvent]| -> usize {
            track
                .iter()
                .filter(|e| {
                    matches!(
                        e.kind,
                        TrackEventKind::Midi { message: MidiMessage::NoteOn { .. }, .. }
                    )
                })
                .count()
        };

        // Track 1 = Melody: 4 notes
        assert_eq!(count_note_ons(&smf.tracks[1]), 4);
        // Track 2 = Chords: 0 notes
        assert_eq!(count_note_ons(&smf.tracks[2]), 0);
        // Track 3 = Bass: 2 notes
        assert_eq!(count_note_ons(&smf.tracks[3]), 2);
        // Track 4 = Drums: 2 heel + 4 toe = 6 notes
        assert_eq!(count_note_ons(&smf.tracks[4]), 6);
    }

    #[test]
    fn test_tempo_meta_event() {
        let timeline = make_test_timeline();
        let bytes = timeline_to_midi(&timeline, &crate::StyleProfile::builtin()).unwrap();
        let smf = Smf::parse(&bytes).expect("generated MIDI should parse");

        // Track 0 should have a tempo event for 120 BPM = 500000 μs/quarter
        let has_tempo = smf.tracks[0].iter().any(|e| {
            matches!(
                e.kind,
                TrackEventKind::Meta(MetaMessage::Tempo(t)) if t.as_int() == 500_000
            )
        });
        assert!(has_tempo, "Expected tempo meta event for 120 BPM");
    }

    #[test]
    fn test_time_signature_meta_event() {
        let timeline = make_test_timeline();
        let bytes = timeline_to_midi(&timeline, &crate::StyleProfile::builtin()).unwrap();
        let smf = Smf::parse(&bytes).expect("generated MIDI should parse");

        // Track 0 should have 4/4 time signature
        let has_ts = smf.tracks[0].iter().any(|e| {
            matches!(e.kind, TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, _, _)))
        });
        assert!(has_ts, "Expected 4/4 time signature");
    }

    #[test]
    fn test_drum_channel() {
        let timeline = make_test_timeline();
        let bytes = timeline_to_midi(&timeline, &crate::StyleProfile::builtin()).unwrap();
        let smf = Smf::parse(&bytes).expect("generated MIDI should parse");

        // All MIDI events in drum track should be on channel 9
        for event in &smf.tracks[4] {
            if let TrackEventKind::Midi { channel, .. } = event.kind {
                assert_eq!(channel.as_int(), GM_DRUM_CHANNEL, "Drum events should be on channel 9");
            }
        }
    }

    #[test]
    fn test_multi_measure() {
        let mut timeline = ScoreTimeline::new(10);

        for i in 1..=4 {
            let mut measure = Measure::new(i, TimeSignature::new(4, 4), 120.0, 16);
            measure.add_note(
                TrackId::Melody,
                TimelineNote {
                    id: i as u64,
                    pitch: 60,
                    start_step: 0,
                    duration_steps: 4,
                    velocity: 80,
                    articulation: Articulation::Normal,
                    performance: Default::default(),
                },
            );
            timeline.push_measure(measure);
        }

        let bytes = timeline_to_midi(&timeline, &crate::StyleProfile::builtin()).unwrap();
        let smf = Smf::parse(&bytes).expect("generated MIDI should parse");

        // 4 notes across 4 measures
        let note_ons: usize = smf.tracks[1]
            .iter()
            .filter(|e| {
                matches!(e.kind, TrackEventKind::Midi { message: MidiMessage::NoteOn { .. }, .. })
            })
            .count();
        assert_eq!(note_ons, 4);
    }

    #[test]
    fn test_write_midi_file() {
        let timeline = make_test_timeline();
        let dir = std::env::temp_dir().join("harmonium_midi_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_output.mid");

        write_midi(&timeline, &crate::StyleProfile::builtin(), &path)
            .expect("should write MIDI file");
        assert!(path.exists());

        // Read back and verify
        let bytes = std::fs::read(&path).unwrap();
        let smf = Smf::parse(&bytes).expect("written file should parse");
        // Tempo, une piste par logique, pieds regroupés en une seule percussion.
        assert_eq!(smf.tracks.len(), TrackId::ALL.len());

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }
}

//! Le MIDI statique porte la même intention que les snapshots (arrondi au tick).
use harmonium_core::{
    params::TimeSignature,
    performance::NotePerformance,
    timeline::{Articulation, Measure, ScoreTimeline, TimelineNote, TrackId, timeline_to_midi},
};
use midly::{MidiMessage, Smf, TrackEventKind};
#[test]
fn midi_offsets_gates_and_negative_barline_are_explicit() {
    let mut timeline = ScoreTimeline::with_default_capacity();
    for bar in 1..=2 {
        let mut measure = Measure::new(bar, TimeSignature::new(4, 4), 120.0, 16);
        let notes = if bar == 1 {
            vec![(1, 60, 0, -100, 500), (2, 62, 2, 125, 750)]
        } else {
            vec![(3, 64, 0, -200, 800)]
        };
        for (id, pitch, start_step, offset, gate) in notes {
            measure.add_note(
                TrackId::Melody,
                TimelineNote {
                    id,
                    pitch,
                    start_step,
                    duration_steps: 2,
                    velocity: 90,
                    articulation: Articulation::Linked,
                    performance: NotePerformance { offset, gate, breath: 0 },
                },
            );
        }
        timeline.push_measure(measure);
    }
    let bytes = timeline_to_midi(&timeline, &Default::default()).unwrap();
    let midi = Smf::parse(&bytes).unwrap();
    let mut tick = 0;
    let mut notes = Vec::new();
    for event in &midi.tracks[1] {
        tick += event.delta.as_int();
        match event.kind {
            TrackEventKind::Midi { message: MidiMessage::NoteOn { key, .. }, .. } => {
                notes.push((tick, true, key.as_int()))
            }
            TrackEventKind::Midi { message: MidiMessage::NoteOff { key, .. }, .. } => {
                notes.push((tick, false, key.as_int()))
            }
            _ => {}
        }
    }
    assert_eq!(
        notes,
        vec![
            (0, true, 60),
            (108, false, 60),
            (255, true, 62),
            (435, false, 62),
            (1896, true, 64),
            (2088, false, 64)
        ]
    );
}

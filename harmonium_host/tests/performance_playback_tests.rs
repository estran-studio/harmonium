//! Attaques/gates aux frames exactes, sans déplacer l'horloge de grille.
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use harmonium::{
    TransportClock,
    backend::AudioRenderer,
    events::AudioEvent,
    playback::{LiveMidiEvent, PlaybackCommand, PlaybackEngine, TaggedMeasure, pack},
};
use harmonium_core::{
    params::{PulseSpec, TimeSignature},
    performance::NotePerformance,
    timeline::{Articulation, Measure, TimelineNote, TrackId},
};

/// Journal atomique borné : aucun verrou ni allocation, même pour la preuve.
struct Capture {
    frame: u64,
    index: usize,
    events: Arc<[AtomicU64; 128]>,
}
impl AudioRenderer for Capture {
    fn handle_event(&mut self, event: AudioEvent) {
        let note = match event {
            AudioEvent::NoteOn { note, .. } => Some((true, note)),
            AudioEvent::NoteOff { note, .. } => Some((false, note)),
            _ => None,
        };
        if let Some((on, pitch)) = note {
            self.events[self.index].store(
                (self.frame << 8) | (u64::from(on) << 7) | u64::from(pitch),
                Ordering::Relaxed,
            );
            self.index += 1;
        }
    }
    fn process_buffer(&mut self, output: &mut [f32], channels: usize) {
        self.frame += (output.len() / channels) as u64;
        output.fill(0.0);
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
fn note(id: u64, pitch: u8, step: usize, duration: usize, offset: i16, gate: u16) -> TimelineNote {
    TimelineNote {
        id,
        pitch,
        start_step: step,
        duration_steps: duration,
        velocity: 90,
        articulation: Articulation::Linked,
        performance: NotePerformance { offset, gate, breath: 0 },
    }
}
#[test]
fn fractional_attacks_gates_barline_and_opening_keep_clock_on_grid() {
    let (mut tx, rx) = rtrb::RingBuffer::<TaggedMeasure>::new(16);
    let (return_tx, _return_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(64);
    let (_commands, commands) = rtrb::RingBuffer::<PlaybackCommand>::new(16);
    let (_live, live) = rtrb::RingBuffer::<LiveMidiEvent>::new(1);
    let (reports, _reports) = rtrb::RingBuffer::new(128);
    let recorded = Arc::new(std::array::from_fn(|_| AtomicU64::new(0)));
    let initial = TransportClock::initial(8000);
    let (clock_tx, mut clock_rx) = triple_buffer::TripleBuffer::new(&initial).split();
    for bar in 1..=3 {
        let mut measure = Measure::new(bar, TimeSignature::new(4, 4), 120.0, 16);
        measure.pulse = PulseSpec { steps_per_pulse: 4, pulses_per_bar: 4 };
        if bar == 1 {
            measure.add_note(TrackId::Melody, note(1, 60, 0, 2, -100, 500));
            measure.add_note(TrackId::Melody, note(2, 62, 2, 2, 125, 750));
            measure.add_note(TrackId::Melody, note(3, 64, 4, 1, -250, 1000));
        } else if bar == 2 {
            measure.add_note(TrackId::Melody, note(4, 65, 0, 2, -200, 800));
            // Note traversant une barre, puis répétition du même pitch avant
            // sa fin initiale : aucune ancienne fin ne doit couper le retrigger.
            measure.add_note(TrackId::Melody, note(5, 67, 15, 4, 0, 1000));
        } else {
            measure.add_note(TrackId::Melody, note(6, 67, 1, 4, 100, 1000));
        }
        measure.prepare_playback();
        assert!(tx.push(TaggedMeasure { epoch: 0, measure: Box::new(measure) }).is_ok());
    }
    let mut engine = PlaybackEngine::new(
        8000.0,
        Box::new(Capture { frame: 0, index: 0, events: recorded.clone() }),
        rx,
        return_tx,
        commands,
        live,
        reports,
        Arc::new(AtomicU64::new(pack(1, 0))),
        clock_tx,
    );
    let mut buffer = [0.0; 128];
    for _ in 0..760 {
        engine.process_buffer(&mut buffer, 2, Instant::now());
        let clock = *clock_rx.read();
        assert_eq!(clock.position.beat, clock.position.step as f32 / 4.0 + 1.0);
    }
    let log: Vec<_> = recorded
        .iter()
        .map(|entry| entry.load(Ordering::Relaxed))
        .take_while(|&entry| entry != 0)
        .map(|entry| (entry >> 8, entry & 128 != 0, (entry & 127) as u8))
        .collect();
    assert_eq!(
        log,
        vec![
            (0, true, 60),
            (900, false, 60),
            (2125, true, 62),
            (3625, false, 62),
            (3750, true, 64),
            (4750, false, 64),
            (15800, true, 65),
            (17400, false, 65),
            (31000, true, 67),
            (33100, false, 67),
            (33100, true, 67),
            (37100, false, 67)
        ]
    );
}

#[test]
fn prepared_playhead_path_is_allocation_free_under_debug_allocator() {
    use harmonium_audio::realtime::rt_check::{enter_audio_context, exit_audio_context};
    use harmonium_core::timeline::Playhead;
    let mut measure = Measure::new(1, TimeSignature::new(4, 4), 120.0, 16);
    for i in 0..16 {
        measure.add_note(
            TrackId::Melody,
            note(i as u64, 60 + (i % 3) as u8, i, 2, if i % 2 == 0 { -120 } else { 130 }, 750),
        );
    }
    measure.prepare_playback();
    let boxed = Box::new(measure);
    let mut playhead = Playhead::new(44100.0, 4);
    enter_audio_context();
    playhead.load_measure(boxed);
    for _ in 0..16 {
        playhead.tick(None);
    }
    let finished = playhead.take_finished();
    exit_audio_context();
    assert!(finished.is_some());
}

#[test]
fn prepared_strum_and_bass_walk_cross_the_bar_under_rt_allocator() {
    let (mut tx, rx) = rtrb::RingBuffer::<TaggedMeasure>::new(16);
    let (return_tx, _return_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(64);
    let (_commands, commands) = rtrb::RingBuffer::<PlaybackCommand>::new(16);
    let (_live, live) = rtrb::RingBuffer::<LiveMidiEvent>::new(1);
    let (reports, _reports) = rtrb::RingBuffer::new(128);
    let recorded = Arc::new(std::array::from_fn(|_| AtomicU64::new(0)));
    let initial = TransportClock::initial(8000);
    let (clock_tx, _clock_rx) = triple_buffer::TripleBuffer::new(&initial).split();
    for bar in 1..=3 {
        let mut m = Measure::new(bar, TimeSignature::new(4, 4), 120.0, 16);
        m.pulse = PulseSpec { steps_per_pulse: 4, pulses_per_bar: 4 };
        if bar == 1 {
            m.add_note(TrackId::Bass, note(1, 43, 15, 4, 0, 1000));
            for (id, pitch, offset) in [(2, 48, 100), (3, 52, 150), (4, 55, 200)] {
                m.add_note(TrackId::Chords, note(id, pitch, 15, 1, offset, 1000));
            }
        } else if bar == 2 {
            for (id, pitch, offset) in [(5, 57, -150), (6, 60, -100), (7, 64, -50)] {
                m.add_note(TrackId::Chords, note(id, pitch, 0, 1, offset, 1000));
            }
            m.add_note(TrackId::Bass, note(8, 43, 1, 4, 0, 1000));
        }
        m.prepare_playback();
        assert!(tx.push(TaggedMeasure { epoch: 0, measure: Box::new(m) }).is_ok());
    }
    let mut engine = PlaybackEngine::new(
        8000.0,
        Box::new(Capture { frame: 0, index: 0, events: recorded.clone() }),
        rx,
        return_tx,
        commands,
        live,
        reports,
        Arc::new(AtomicU64::new(pack(1, 0))),
        clock_tx,
    );
    let mut buffer = [0.0; 128];
    // process_buffer est entièrement sous RtGuard depuis RT1.
    for _ in 0..600 {
        engine.process_buffer(&mut buffer, 2, Instant::now());
    }
    let log: Vec<_> = recorded
        .iter()
        .map(|e| e.load(Ordering::Relaxed))
        .take_while(|&e| e != 0)
        .map(|e| (e >> 8, e & 128 != 0, (e & 127) as u8))
        .collect();
    let on: Vec<_> = log.iter().filter(|e| e.1).map(|e| (e.0, e.2)).collect();
    assert_eq!(
        on,
        [
            (15000, 43),
            (15100, 48),
            (15150, 52),
            (15200, 55),
            (15850, 57),
            (15900, 60),
            (15950, 64),
            (17000, 43)
        ]
    );
    let bass_off: Vec<_> = log.iter().filter(|e| !e.1 && e.2 == 43).map(|e| e.0).collect();
    assert_eq!(bass_off, [17000, 21000]); // L'ancienne fin à 19000 ne coupe pas le retrigger.
}

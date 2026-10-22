//! Swing (D3) and tempo-ramp (D4) acceptance tests, phase 0.
//!
//! Drives the real offline `PlaybackEngine` and verifies the two timing
//! features on whole frames: a swung 4/4 pulse is exactly one rounded quarter
//! (22050 frames at 120 BPM, measure 88200), and a `SetTempo { 240, 8 }` ramp
//! lands exactly on a 240 BPM quarter (11025 frames) then `ClearTempo`
//! reverts. The `swing_unit` pure function is also unit-tested.
//!
//! The coincidence tests below drive the engine directly (hand-built measures,
//! one Lead note per step, recording renderer) and prove the clock/audio
//! invariant: at swing 0.6 the NoteOn gaps are 6615, 6615, 4410, 4410 and
//! every pulse's first NoteOn lands exactly on its `pulse_start_frame` —
//! including during a tempo ramp.

use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex, atomic::AtomicU64},
    time::Instant,
};

use harmonium::{
    TransportClock,
    audio::create_offline_engine,
    backend::AudioRenderer,
    events::AudioEvent,
    playback::{
        LiveMidiEvent, MEASURE_RING_CAP, PlaybackCommand, PlaybackEngine, RETURN_RING_CAP,
        TaggedMeasure, pack, swing_unit,
    },
};
use harmonium_core::{
    params::{PulseSpec, TimeSignature},
    timeline::{Articulation, Measure, TimelineNote, TrackId},
};

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;

fn quarter_profile() -> harmonium_core::StyleProfile {
    let mut profile = harmonium_core::StyleProfile::builtin().clone();
    profile.meter.time_signature = TimeSignature::new(4, 4);
    profile.meter.steps_per_pulse = 4;
    profile.meter.pulses_per_bar = 4;
    profile
}
/// Acceptance (2) — the swing unit: at ratio 0.6, 120 BPM, 44100 Hz, the step
/// durations are exactly [6615, 6615, 4410, 4410] (step starts 0, 13230,
/// 22050) and the unit sums to the rounded quarter (22050).
#[test]
fn swing_unit_has_exact_durations() {
    let quarter = 22050usize;
    let unit = swing_unit(quarter, 0.6);
    assert_eq!(unit, [6615, 6615, 4410, 4410]);
    assert_eq!(unit[0], 6615);
    assert_eq!(unit[0] + unit[1], 13230);
    assert_eq!(unit.iter().sum::<usize>(), quarter);

    // A straight (0.5) unit under a ramp still sums to the rounded quarter.
    let straight = swing_unit(11025, 0.5);
    assert_eq!(straight.iter().sum::<usize>(), 11025);
}

/// Acceptance (2) — on the engine: every swung 4/4 pulse lasts exactly one
/// quarter (22050 frames at 120 BPM), so a measure lasts 88200 frames.
#[test]
fn swing_quarter_lasts_exactly_22050_frames() -> Result<(), String> {
    let (mut composer, mut playback, _cmd_tx, _report_rx, mut clock_reader, _recordings) =
        create_offline_engine(None, None, SAMPLE_RATE)?;

    composer.set_profile(quarter_profile())?;
    composer.set_bpm(120.0);
    composer.set_swing_ratio(0.6);
    composer.sync_generator();
    composer.set_writehead_lookahead(8);
    composer.generate_bars(8);

    let mut buffer = vec![0.0f32; 512 * CHANNELS];
    let mut checked = 0usize;
    let mut last_next = 0u64;
    // Bar-start frames: consecutive entries must be exactly one swung
    // measure apart (4 × 22050 = 88200 frames).
    let mut bar_starts: Vec<u64> = Vec::new();
    let mut last_bar = 0usize;
    for _ in 0..800 {
        composer.generate_ahead();
        buffer.fill(0.0);
        playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
        let clock = clock_reader.read();
        if clock.pulse_start_frame > 0 && clock.next_pulse_frame != last_next {
            assert_eq!(
                clock.next_pulse_frame - clock.pulse_start_frame,
                22050,
                "swung pulse must be exactly one quarter (22050 frames)"
            );
            last_next = clock.next_pulse_frame;
            checked += 1;
        }
        if clock.position.bar != last_bar {
            last_bar = clock.position.bar;
            if clock.pulse_start_frame > 0 {
                bar_starts.push(clock.pulse_start_frame);
            }
        }
        if clock.position.bar > 3 {
            break;
        }
    }
    assert!(checked >= 12, "only checked {checked} pulses");
    assert!(bar_starts.len() >= 3, "only saw {} bar starts", bar_starts.len());
    for pair in bar_starts.windows(2) {
        assert_eq!(pair[1] - pair[0], 88200, "swung 4/4 measure must last exactly 88200 frames");
    }
    Ok(())
}

/// Acceptance (3) — tempo ramp: `SetTempo { 240, 8 }` received during a pulse
/// makes the next 8 pulse durations strictly decrease, the 8th landing exactly
/// on a 240 BPM quarter (11025 frames); `ClearTempo` reverts to the measure
/// tempo at the next boundary.
#[test]
fn tempo_ramp_reaches_target_then_clears() -> Result<(), String> {
    let (mut composer, mut playback, mut cmd_tx, _report_rx, mut clock_reader, _recordings) =
        create_offline_engine(None, None, SAMPLE_RATE)?;

    composer.set_profile(quarter_profile())?;
    composer.set_swing_ratio(0.5);
    composer.set_bpm(120.0);
    composer.sync_generator();
    composer.set_writehead_lookahead(8);
    composer.generate_bars(8);

    let mut buffer = vec![0.0f32; 512 * CHANNELS];
    let mut durations = Vec::new();
    let mut last_next = 0u64;
    let mut sent = false;

    // Phase 1: drive the ramp. Collect one duration per pulse (the current
    // pulse's frozen length), sending the ramp once pulse 0 is frozen.
    loop {
        composer.generate_ahead();
        buffer.fill(0.0);
        playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
        let clock = clock_reader.read();

        if clock.next_pulse_frame > 0 && clock.next_pulse_frame != last_next {
            durations.push(clock.next_pulse_frame - clock.pulse_start_frame);
            last_next = clock.next_pulse_frame;

            if !sent && durations.len() == 1 {
                let _ = cmd_tx.push(PlaybackCommand::SetTempo { bpm: 240.0, ramp_pulses: 8 });
                sent = true;
            }
            if durations.len() >= 9 {
                break;
            }
        }
    }

    // durations = [pulse0=22048, pulse1=19600, ..., pulse8=11025] — the ramp
    // is pulses 1..=8 (received during pulse 0).
    let ramp = &durations[1..9];
    for pair in ramp.windows(2) {
        assert!(pair[1] < pair[0], "ramp must be strictly decreasing: {ramp:?}");
    }
    assert_eq!(ramp[7], 11025, "pulse k+8 must be exactly a quarter at 240 BPM");

    // Phase 2: ClearTempo reverts to the measure tempo at the next pulse
    // boundary (120 BPM, straight = 22048).
    let _ = cmd_tx.push(PlaybackCommand::ClearTempo);
    let cleared_duration = loop {
        composer.generate_ahead();
        buffer.fill(0.0);
        playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
        let clock = clock_reader.read();
        if clock.next_pulse_frame > 0 && clock.next_pulse_frame != last_next {
            break clock.next_pulse_frame - clock.pulse_start_frame;
        }
    };
    assert_eq!(cleared_duration, 22048, "ClearTempo must revert to the 120 BPM measure tempo");
    Ok(())
}

// ─── Clock/audio coincidence: NoteOn frames vs `pulse_start_frame` ───

/// Renderer that voices nothing but records the engine frame of every Lead
/// NoteOn. `frame` counts rendered frames, so it lives in the same frame
/// space as `TransportClock`. The note log is pre-sized: nothing allocates
/// on the audio thread, even in debug builds with `RTCheckAllocator`.
struct FrameRecorder {
    frame: u64,
    notes: Arc<Mutex<Vec<(u64, u8)>>>,
}

impl AudioRenderer for FrameRecorder {
    fn handle_event(&mut self, event: AudioEvent) {
        if let AudioEvent::NoteOn { note, channel: 0, .. } = event
            && let Ok(mut notes) = self.notes.lock()
            && notes.len() < notes.capacity()
        {
            notes.push((self.frame, note));
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

/// Harness driving a `PlaybackEngine` directly: 4 hand-built 4/4 bars, one
/// Lead note per step, swing 0.6 at 120 BPM — no generation involved.
struct NoteHarness {
    engine: PlaybackEngine,
    cmd_tx: rtrb::Producer<PlaybackCommand>,
    clock_output: triple_buffer::Output<TransportClock>,
    notes: Arc<Mutex<Vec<(u64, u8)>>>,
    _measure_tx: rtrb::Producer<TaggedMeasure>,
    _live_tx: rtrb::Producer<LiveMidiEvent>,
    _report_rx: rtrb::Consumer<harmonium_core::EngineReport>,
    _return_rx: rtrb::Consumer<TaggedMeasure>,
    _transport: Arc<AtomicU64>,
}

fn make_note_harness() -> NoteHarness {
    let (cmd_tx, cmd_rx) = rtrb::RingBuffer::<PlaybackCommand>::new(256);
    let (_live_tx, live_rx) = rtrb::RingBuffer::<LiveMidiEvent>::new(1);
    let (report_tx, report_rx) = rtrb::RingBuffer::<harmonium_core::EngineReport>::new(256);
    let (measure_tx, measure_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(MEASURE_RING_CAP);
    let (return_tx, return_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(RETURN_RING_CAP);

    let ts = TimeSignature::new(4, 4);
    let pulse = PulseSpec::from_time_signature(ts, 4);
    let mut measure_tx = measure_tx;
    for i in 1..=4usize {
        let mut measure = Measure::new(i, ts, 120.0, 16);
        measure.swing_ratio = 0.6;
        measure.pulse = pulse;
        for s in 0..16usize {
            measure.add_note(
                TrackId::Melody,
                TimelineNote {
                    id: i as u64 * 100 + s as u64,
                    pitch: 60 + s as u8,
                    start_step: s,
                    duration_steps: 1,
                    velocity: 90,
                    articulation: Articulation::Normal,
                },
            );
        }
        assert!(
            measure_tx.push(TaggedMeasure { epoch: 0, measure: Box::new(measure) }).is_ok(),
            "staging bar {i} must fit the ring"
        );
    }

    let transport = Arc::new(AtomicU64::new(pack(1, 0)));
    let initial = TransportClock::initial(SAMPLE_RATE as u32);
    let (clock_tx, clock_output) = triple_buffer::TripleBuffer::new(&initial).split();
    let notes = Arc::new(Mutex::new(Vec::with_capacity(512)));
    let engine = PlaybackEngine::new(
        SAMPLE_RATE,
        Box::new(FrameRecorder { frame: 0, notes: notes.clone() }),
        measure_rx,
        return_tx,
        cmd_rx,
        live_rx,
        report_tx,
        transport.clone(),
        clock_tx,
    );
    NoteHarness {
        engine,
        cmd_tx,
        clock_output,
        notes,
        _measure_tx: measure_tx,
        _live_tx,
        _report_rx: report_rx,
        _return_rx: return_rx,
        _transport: transport,
    }
}

impl NoteHarness {
    /// Render `frames` samples in 64-frame buffers, snapshotting the clock
    /// after each buffer.
    fn render(&mut self, frames: u64) -> Vec<TransportClock> {
        let mut snaps = Vec::new();
        let mut buffer = vec![0.0f32; 64 * CHANNELS];
        let mut done = 0u64;
        while done < frames {
            buffer.fill(0.0);
            self.engine.process_buffer(&mut buffer, CHANNELS, Instant::now());
            snaps.push(*self.clock_output.read());
            done += 64;
        }
        snaps
    }

    fn note_frames(&self) -> Vec<(u64, u8)> {
        self.notes.lock().map(|guard| guard.clone()).unwrap_or_default()
    }
}

/// Défauts 1+2 — at swing 0.6 the consecutive NoteOn gaps are 6615, 6615,
/// 4410, 4410 (the duration of the step being entered, not the next), and
/// every pulse's first NoteOn lands exactly on its `pulse_start_frame`.
#[test]
fn swung_noteons_match_clock_pulses() -> Result<(), String> {
    let mut h = make_note_harness();
    let snaps = h.render(4 * 88200 + 128);
    let notes = h.note_frames();
    assert_eq!(notes.len(), 64, "one NoteOn per step over 4 bars, got {}", notes.len());

    // Measure 2 starts exactly at frame 88200; its 16 NoteOn offsets are the
    // cumulative swung steps 0, 6615, 13230, 17640, 22050, …
    let m2: Vec<u64> =
        notes.iter().map(|(f, _)| *f).filter(|f| (88200..176400).contains(f)).collect();
    assert_eq!(m2.len(), 16, "measure 2 must hold 16 NoteOn, got {}", m2.len());
    let base = m2[0];
    let expected = [
        0, 6615, 13230, 17640, 22050, 28665, 35280, 39690, 44100, 50715, 57330, 61740, 66150,
        72765, 79380, 83790,
    ];
    for (i, (got, want)) in m2.iter().zip(expected.iter()).enumerate() {
        assert_eq!(got - base, *want, "measure-2 step {i} NoteOn offset");
    }
    // … and measure 3 starts exactly one measure later.
    let m3 = notes.iter().map(|(f, _)| *f).find(|f| *f >= 176400).ok_or("missing bar 3")?;
    assert_eq!(m3 - base, 88200, "measure 3 must start one measure after measure 2");

    // Every pulse start published in measure 2 coincides with a NoteOn frame.
    let note_set: BTreeSet<u64> = notes.iter().map(|(f, _)| *f).collect();
    let mut starts = BTreeSet::new();
    for snap in &snaps {
        if (88200..176400).contains(&snap.pulse_start_frame) {
            starts.insert(snap.pulse_start_frame);
        }
    }
    assert_eq!(starts, BTreeSet::from([88200, 110250, 132300, 154350]));
    for start in &starts {
        assert!(note_set.contains(start), "pulse start {start} has no NoteOn");
    }
    Ok(())
}

/// Same coincidence during a ramp: after `SetTempo { 240, 8 }` every pulse's
/// first NoteOn still lands on its `pulse_start_frame`, while the 8 pulse
/// durations decrease strictly and land on 11025.
#[test]
fn ramp_noteons_match_clock_pulses() -> Result<(), String> {
    let mut h = make_note_harness();
    // Drive into bar 2, then launch the ramp mid-pulse.
    let mut pre = h.render(88200 + 22050 + 64);
    let frozen = *pre.last().ok_or("no clock snapshot")?;
    assert_eq!(frozen.position.bar, 2);
    assert!(
        h.cmd_tx.push(PlaybackCommand::SetTempo { bpm: 240.0, ramp_pulses: 8 }).is_ok(),
        "SetTempo must fit the command ring"
    );
    let post = h.render(3 * 88200);
    pre.extend(post);

    // The 8 pulses after the send: strictly decreasing durations, landing on
    // exactly a 240 BPM quarter (11025 frames).
    let mut pulses: Vec<(u64, u64)> = Vec::new();
    let mut last = 0u64;
    for snap in &pre {
        if snap.pulse_start_frame > frozen.pulse_start_frame && snap.next_pulse_frame != last {
            pulses.push((snap.pulse_start_frame, snap.next_pulse_frame));
            last = snap.next_pulse_frame;
        }
    }
    assert!(pulses.len() >= 8, "only {} pulses after the ramp send", pulses.len());
    let durations: Vec<u64> = pulses.iter().take(8).map(|(s, n)| n - s).collect();
    for pair in durations.windows(2) {
        assert!(pair[1] < pair[0], "ramp must be strictly decreasing: {durations:?}");
    }
    assert_eq!(durations[7], 11025, "ramp must land on a 240 BPM quarter");

    // … and each of those pulses still opens exactly on a NoteOn.
    let note_set: BTreeSet<u64> = h.note_frames().iter().map(|(f, _)| *f).collect();
    for (start, _) in pulses.iter().take(8) {
        assert!(note_set.contains(start), "ramp pulse start {start} has no NoteOn");
    }
    Ok(())
}

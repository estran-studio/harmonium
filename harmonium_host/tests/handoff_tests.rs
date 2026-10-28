//! Lock-free measure handoff tests (phase 0, D5).
//!
//! A real [`PlaybackEngine`] is driven offline against a real `MusicComposer`
//! (no mocks): bars are generated, staged through the SPSC rings, played,
//! and handed back through the return ring. Notes are observed at the
//! renderer and checked step by step against the composer's timeline — the
//! engine must never play a measure whose index differs from the playhead
//! bar (missing boxes mean silence, as before).
//!
//! Real-time proof: every handoff op runs under the debug
//! `RTCheckAllocator` guard (see `RtGuard` in `playback.rs`), so any
//! allocation, deallocation or lock on the audio path panics and fails
//! these tests. Completing this file in debug is acceptance (c).

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

use harmonium::{
    composer::MusicComposer,
    playback::{
        MEASURE_RING_CAP, PlaybackCommand, PlaybackEngine, RETURN_RING_CAP, TaggedMeasure,
        check_loop_len, unpack,
    },
};
use harmonium_audio::backend::AudioRenderer;
use harmonium_core::{events::AudioEvent, timeline::TrackId};

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;
/// Samples per grid step at 126 BPM (a whole number: step-sized buffers stay
/// aligned with the plan's exact step instants).
const SAMPLES_PER_STEP: usize = 5250;
const STEPS_PER_BAR: usize = 16;

/// Shared event log: the renderer records every event it receives; the test
/// drains it after each step and attributes notes to (bar, step). The log is
/// pre-sized and keeps its capacity when drained: the audio thread never
/// allocates, the whole `process_buffer` runs under `RTCheckAllocator`.
#[derive(Clone)]
struct EventLog(Arc<Mutex<Vec<AudioEvent>>>);

impl Default for EventLog {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Vec::with_capacity(1 << 16))))
    }
}

impl EventLog {
    fn drain(&self) -> Vec<AudioEvent> {
        if let Ok(mut guard) = self.0.lock() { guard.drain(..).collect() } else { Vec::new() }
    }

    fn push(&self, event: AudioEvent) {
        if let Ok(mut guard) = self.0.lock()
            && guard.len() < guard.capacity()
        {
            guard.push(event);
        }
    }
}

struct RecordingRenderer {
    log: EventLog,
}

impl AudioRenderer for RecordingRenderer {
    fn handle_event(&mut self, event: AudioEvent) {
        self.log.push(event);
    }
    fn process_buffer(&mut self, output: &mut [f32], _channels: usize) {
        output.fill(0.0);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct Harness {
    composer: MusicComposer,
    engine: PlaybackEngine,
    transport: Arc<AtomicU64>,
    cmd_tx: rtrb::Producer<PlaybackCommand>,
    report_rx: rtrb::Consumer<harmonium_core::EngineReport>,
    log: EventLog,
}

fn make_harness(seed: u64) -> Harness {
    let (cmd_tx, cmd_rx) = rtrb::RingBuffer::<PlaybackCommand>::new(256);
    let (report_tx, report_rx) = rtrb::RingBuffer::<harmonium_core::EngineReport>::new(256);
    let (measure_tx, measure_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(MEASURE_RING_CAP);
    let (return_tx, return_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(RETURN_RING_CAP);

    let transport = Arc::new(AtomicU64::new(harmonium::playback::pack(1, 0)));
    let mut composer =
        MusicComposer::new_with_seed(SAMPLE_RATE, measure_tx, return_rx, transport.clone(), seed);
    // Grille exacte : ce test vérifie les index de mesure, pas l'interprétation.
    let mut profile = composer.profile().clone();
    profile.performance = Default::default();
    profile.ornaments = Default::default();
    profile.accompaniment.as_mut().unwrap().chords.strum = None;
    assert!(composer.set_profile(profile).is_ok());
    let _ = composer.take_pending_drop();
    composer.set_bpm(126.0);
    composer.set_swing_ratio(0.5);
    composer.sync_generator();
    let log = EventLog::default();
    let initial_clock = harmonium::TransportClock::initial(SAMPLE_RATE as u32);
    let (clock_tx, _clock_output) = triple_buffer::TripleBuffer::new(&initial_clock).split();
    let engine = PlaybackEngine::new(
        SAMPLE_RATE,
        Box::new(RecordingRenderer { log: log.clone() }),
        measure_rx,
        return_tx,
        cmd_rx,
        report_tx,
        transport.clone(),
        clock_tx,
    );

    Harness { composer, engine, transport, cmd_tx, report_rx, log }
}

/// Expected NoteOn multiset at `(bar, step)`, as sorted `(pitch, velocity,
/// channel)` triples read from the composer's timeline (source of truth).
fn expected_step_notes(composer: &MusicComposer, bar: usize, step: usize) -> Vec<(u8, u8, u8)> {
    let mut notes = Vec::new();
    if let Some(measure) = composer.timeline_measure(bar) {
        for track in TrackId::ALL {
            for note in measure.notes_for_track(track) {
                if note.start_step == step {
                    notes.push((note.pitch, note.velocity, track.channel()));
                }
            }
        }
    }
    notes.sort_unstable();
    notes
}

/// NoteOns received since the last drain, as sorted `(pitch, velocity,
/// channel)` triples.
fn played_step_notes(log: &EventLog) -> Vec<(u8, u8, u8)> {
    let mut notes = Vec::new();
    for event in log.drain() {
        if let AudioEvent::NoteOn { note, velocity, channel } = event {
            notes.push((note, velocity, channel));
        }
    }
    notes.sort_unstable();
    notes
}

impl Harness {
    /// Render one grid step, then check the emitted NoteOns against the
    /// timeline content for `(expect_bar, step)`. An empty play against an
    /// empty expectation is an underflow (silence) — legal; any divergence
    /// is a stale/wrong-index box and fails.
    fn render_checked_step(&mut self, expect_bar: usize, step: usize) {
        self.composer.generate_ahead();
        let mut buffer = vec![0.0f32; SAMPLES_PER_STEP * CHANNELS];
        self.engine.process_buffer(&mut buffer, CHANNELS, std::time::Instant::now());
        while self.report_rx.pop().is_ok() {}
        let played = played_step_notes(&self.log);
        let expected = expected_step_notes(&self.composer, expect_bar, step);
        assert_eq!(
            played, expected,
            "bar {expect_bar} step {step}: played notes diverge from the timeline"
        );
    }

    /// Render a full bar step by step and check the transport advanced past
    /// it — the same positions the engine published before.
    fn render_checked_bar(&mut self, expect_bar: usize) {
        for step in 0..STEPS_PER_BAR {
            self.render_checked_step(expect_bar, step);
        }
        let (bar, step) = unpack(self.transport.load(Ordering::Relaxed));
        assert_eq!(
            (bar, step),
            (expect_bar + 1, 0),
            "transport must advance past bar {expect_bar}"
        );
    }

    /// Render a full bar and return all its NoteOns (for repetition checks).
    fn render_collected_bar(&mut self, expect_bar: usize) -> Vec<(u8, u8, u8)> {
        let mut collected = Vec::new();
        for _ in 0..STEPS_PER_BAR {
            self.composer.generate_ahead();
            let mut buffer = vec![0.0f32; SAMPLES_PER_STEP * CHANNELS];
            self.engine.process_buffer(&mut buffer, CHANNELS, std::time::Instant::now());
            while self.report_rx.pop().is_ok() {}
            collected.extend(played_step_notes(&self.log));
        }
        collected.sort_unstable();
        let (bar, step) = unpack(self.transport.load(Ordering::Relaxed));
        assert_eq!(
            (bar, step),
            (expect_bar + 1, 0),
            "transport must advance past bar {expect_bar}"
        );
        collected
    }
}

// ─── (a) seek then invalidation during playback ───

/// 32 bars with a deterministic seek and a mid-read invalidation: every
/// played step matches the timeline bar under the playhead — never a stale
/// box, silence on underflow, same positions as before.
#[test]
fn seek_then_invalidate_never_plays_stale_bar() {
    let mut h = make_harness(42);
    h.composer.set_writehead_lookahead(8);
    h.composer.generate_bars(8);

    // Bars 1-2, checked note by note against the timeline.
    h.render_checked_bar(1);
    h.render_checked_bar(2);

    // Deterministic seek to bar 5, exactly like `LiveHost::apply(Seek)`:
    // composer reset, DropFrom, then Seek.
    h.composer.deterministic_seek(5);
    let pending = h.composer.take_pending_drop();
    assert!(matches!(pending, Some((5, _))), "seek must request an eviction from 5");
    if let Some((drop_bar, _)) = pending {
        let _ = h.cmd_tx.push(PlaybackCommand::DropFrom { bar: drop_bar });
    }
    let _ = h.cmd_tx.push(PlaybackCommand::Seek(5));

    // Bars 5-6 from the replayed timeline.
    h.render_checked_bar(5);
    h.render_checked_bar(6);

    // Invalidate mid-read with a param change at constant tempo (so the
    // grid step size is unchanged and per-step checks stay exact).
    h.composer.set_rhythm_density(0.9);
    h.composer.invalidate_future();
    let pending = h.composer.take_pending_drop();
    assert!(matches!(pending, Some((8, _))), "invalidate must request an eviction from 8");
    if let Some((drop_bar, _)) = pending {
        let _ = h.cmd_tx.push(PlaybackCommand::DropFrom { bar: drop_bar });
    }
    h.composer.generate_bars(8);

    // The kept playhead bar, then regenerated content: 32 bars in total.
    for bar in 7..=34 {
        h.render_checked_bar(bar);
    }
}

// ─── (b) loop repetition then linear resume ───

/// A 4-bar loop played 5 times sounds identical on every pass, then
/// `ClearLoop` resumes linearly. Loop-length validation is enforced on the
/// main thread (over-long loops are refused, never played).
#[test]
fn loop_replays_identical_passes_then_resumes() {
    let mut h = make_harness(7);
    h.composer.set_writehead_lookahead(8);
    h.composer.generate_bars(12);

    let _ = h.cmd_tx.push(PlaybackCommand::SetLoop { start_bar: 2, end_bar: 5 });

    // Bar 1, then 5 passes of bars 2-5.
    let first_bar = h.render_collected_bar(1);
    let mut first_bar_timeline = Vec::new();
    for step in 0..STEPS_PER_BAR {
        first_bar_timeline.extend(expected_step_notes(&h.composer, 1, step));
    }
    first_bar_timeline.sort_unstable();
    assert_eq!(first_bar, first_bar_timeline, "bar 1 must match the timeline");

    let mut passes = Vec::new();
    for _ in 0..5 {
        let mut pass = Vec::new();
        for bar in 2..=5 {
            pass.extend(h.render_collected_bar(bar));
        }
        pass.sort_unstable();
        passes.push(pass);
    }
    for (i, pass) in passes.iter().enumerate() {
        assert_eq!(&passes[0], pass, "loop pass {i} differs from the first pass");
    }

    // The repeated pass is the timeline content, not a stuck box: it must
    // equal bars 2-5 of the timeline.
    let mut loop_timeline = Vec::new();
    for bar in 2..=5 {
        for step in 0..STEPS_PER_BAR {
            loop_timeline.extend(expected_step_notes(&h.composer, bar, step));
        }
    }
    loop_timeline.sort_unstable();
    assert_eq!(passes[0], loop_timeline, "loop passes must match timeline bars 2-5");

    // ClearLoop resumes linearly from bar 6.
    let _ = h.cmd_tx.push(PlaybackCommand::ClearLoop);
    for bar in 6..=10 {
        h.render_checked_bar(bar);
    }

    // Main-thread validation: 17 bars do not fit the window, 16 do.
    assert!(!check_loop_len(1, 17));
    assert!(check_loop_len(1, 16));
    assert!(check_loop_len(2, 5));
}

// ─── (c) control storm under the RT guard ───

/// Seek / DropFrom / loop / invalidate / reset churn while rendering: no
/// RTCheckAllocator panic may fire on the handoff path. Content is not
/// checked here (the storm may legitimately underflow); completing the run
/// in debug is the assertion.
#[test]
fn control_storm_stays_rt_clean() {
    let mut h = make_harness(99);
    h.composer.set_writehead_lookahead(8);
    h.composer.generate_bars(8);

    let mut buffer = vec![0.0f32; 1024 * CHANNELS];
    for i in 0..64 {
        h.composer.generate_ahead();
        if i % 8 == 0 {
            h.composer.invalidate_after_preview(2);
            if let Some((bar, _)) = h.composer.take_pending_drop() {
                let _ = h.cmd_tx.push(PlaybackCommand::DropFrom { bar });
            }
        }
        if i % 16 == 8 {
            let _ = h.cmd_tx.push(PlaybackCommand::SeekPlayhead(1 + (i % 4)));
        }
        if i % 32 == 0 {
            let _ = h.cmd_tx.push(PlaybackCommand::SetLoop { start_bar: 1, end_bar: 4 });
        }
        if i % 32 == 16 {
            let _ = h.cmd_tx.push(PlaybackCommand::ClearLoop);
        }
        if i == 48 {
            h.composer.reset_timeline();
            if let Some((bar, _)) = h.composer.take_pending_drop() {
                let _ = h.cmd_tx.push(PlaybackCommand::DropFrom { bar });
            }
            let _ = h.cmd_tx.push(PlaybackCommand::Seek(1));
        }
        buffer.fill(0.0);
        h.engine.process_buffer(&mut buffer, CHANNELS, std::time::Instant::now());
        while h.report_rx.pop().is_ok() {}
        h.log.drain();
    }
}

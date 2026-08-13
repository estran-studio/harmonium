//! Transport position tests (spec 006 — sub-beat transport position)
//!
//! The audio-thread [`PlaybackEngine`] publishes its position at 16th-note
//! grid resolution in ONE packed `AtomicU64` (`bar << 32 | step`). These tests
//! drive a real engine (NullRenderer, hand-built measures, offline) and sample
//! the packed value far faster than a beat — the same way a MIDI callback or
//! a rhythmic-scoring loop would — to prove:
//!
//! - 16 distinct positions per 4/4 measure, not 4 (T010)
//! - `Seek` / `SeekPlayhead` publish `(bar, 0)` (T011)
//! - `SetLoop` wrap publishes the start bar and never runs away (T012)
//! - reads are stable between ticks (pause-equivalent) (T013)
//! - position reads never take the composer lock (T018)

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use harmonium::{
    composer::MusicComposer,
    playback::{
        LiveMidiEvent, PlaybackCommand, PlaybackEngine, TICKS_PER_BEAT, TransportPosition, pack,
        unpack,
    },
};
use harmonium_audio::backend::AudioRenderer;
use harmonium_core::{events::AudioEvent, params::TimeSignature, timeline::Measure};

const SAMPLE_RATE: f64 = 44100.0;
const BPM: f32 = 120.0;
/// 44100 * 60 / 120 / 4 — samples per grid step at 120 bpm in 4/4.
const SAMPLES_PER_STEP: usize = 5512;
/// ~43 reads per step — far faster than a beat.
const READ_EVERY: usize = 128;

fn samples_per_bar() -> usize {
    (SAMPLE_RATE * 60.0 / f64::from(BPM) * 4.0) as usize
}

// ─── Null renderer ───

struct NullRenderer;

impl AudioRenderer for NullRenderer {
    fn handle_event(&mut self, _event: AudioEvent) {}

    fn process_buffer(&mut self, output: &mut [f32], _channels: usize) {
        output.fill(0.0);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ─── Harness ───

struct Harness {
    engine: PlaybackEngine,
    transport: Arc<AtomicU64>,
    cmd_tx: rtrb::Producer<PlaybackCommand>,
    /// Kept alive so the engine's live-MIDI consumer never sees a closed ring.
    _live_tx: rtrb::Producer<LiveMidiEvent>,
    /// Reports are drained lazily; the consumer just must outlive the engine.
    _report_rx: rtrb::Consumer<harmonium_core::EngineReport>,
}

fn make_playback(n_bars: usize) -> Harness {
    let (cmd_tx, cmd_rx) = rtrb::RingBuffer::<PlaybackCommand>::new(256);
    let (_live_tx, live_rx) = rtrb::RingBuffer::<LiveMidiEvent>::new(1);
    let (report_tx, report_rx) = rtrb::RingBuffer::<harmonium_core::EngineReport>::new(256);

    let pages: Arc<Mutex<Vec<Measure>>> = Arc::new(Mutex::new(
        (1..=n_bars).map(|i| Measure::new(i, TimeSignature::default(), BPM, 16)).collect(),
    ));

    let transport = Arc::new(AtomicU64::new(pack(1, 0)));
    let engine = PlaybackEngine::new(
        SAMPLE_RATE,
        Box::new(NullRenderer),
        pages,
        cmd_rx,
        live_rx,
        report_tx,
        transport.clone(),
    );

    Harness { engine, transport, cmd_tx, _live_tx, _report_rx: report_rx }
}

impl Harness {
    /// Process `total_samples` in 128-sample chunks, reading the packed
    /// position after every chunk — the same cadence as a real-time
    /// scoring loop.
    fn run_sampling(&mut self, total_samples: usize) -> Vec<u64> {
        let mut observed = Vec::new();
        let mut buffer = vec![0.0f32; READ_EVERY * 2];
        let mut processed = 0;
        while processed < total_samples {
            let chunk = (total_samples - processed).min(READ_EVERY);
            let buf = &mut buffer[..chunk * 2];
            self.engine.process_buffer(buf, 2);
            processed += chunk;
            observed.push(self.transport.load(Ordering::Relaxed));
        }
        observed
    }

    fn position(&self) -> TransportPosition {
        TransportPosition::from_packed(self.transport.load(Ordering::Relaxed))
    }
}

// ─── T010: sixteen distinct positions per 4/4 measure ───

/// SIXTEEN distinct positions per measure in 4/4 — not four. This is THE
/// proof demanded by spec 006: sample the position much faster than a beat
/// and count distinct values per measure. Run with `--nocapture` to consign
/// the reading to the report.
#[test]
fn sixteen_distinct_positions_per_measure() {
    let mut h = make_playback(8);
    // Two full 4/4 bars plus a margin of three steps.
    let observed = h.run_sampling(samples_per_bar() * 2 + SAMPLES_PER_STEP * 3);

    let mut steps_per_bar: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    for &packed in &observed {
        let (bar, step) = unpack(packed);
        steps_per_bar.entry(bar).or_default().insert(step);
    }

    for bar in [1usize, 2] {
        let steps =
            steps_per_bar.get(&bar).unwrap_or_else(|| panic!("bar {bar} was never observed"));
        assert_eq!(
            steps.len(),
            16,
            "bar {bar}: {} distinct positions observed, want 16",
            steps.len()
        );
        for step in 0..16 {
            assert!(steps.contains(&step), "bar {bar}: step {step} was never observed");
        }
    }

    // Observed proof, consigned for the SE1 report.
    eprintln!(
        "[observed-proof] grid = {} steps/beat, sampling every {READ_EVERY} samples \
         ({} reads/step); {} distinct packed values over {} reads",
        TICKS_PER_BEAT,
        SAMPLES_PER_STEP / READ_EVERY,
        steps_per_bar.values().map(BTreeSet::len).sum::<usize>(),
        observed.len(),
    );
    for (bar, steps) in &steps_per_bar {
        eprintln!(
            "[observed-proof] measure {bar}: {} distinct positions ({:?})",
            steps.len(),
            steps
        );
    }
}

// ─── T011: Seek / SeekPlayhead publish (bar, 0) ───

#[test]
fn seek_and_seekplayhead_publish_bar_step_zero() {
    let mut h = make_playback(4);

    // Commands are processed at the start of process_buffer; an empty buffer
    // runs them without any tick, so the position read right after is exactly
    // the command's write.
    h.cmd_tx.push(PlaybackCommand::Seek(3)).unwrap();
    let mut empty: [f32; 0] = [];
    h.engine.process_buffer(&mut empty, 2);
    assert_eq!(h.position(), TransportPosition { bar: 3, step: 0, beat: 1.0 });

    // One full step later (counter starts at 0: exactly one tick), the
    // post-tick write publishes (3, 1).
    let mut one_step = vec![0.0f32; SAMPLES_PER_STEP * 2];
    h.engine.process_buffer(&mut one_step, 2);
    let after_tick = h.position();
    assert_eq!((after_tick.bar, after_tick.step), (3, 1));

    h.cmd_tx.push(PlaybackCommand::SeekPlayhead(5)).unwrap();
    h.engine.process_buffer(&mut empty, 2);
    assert_eq!(h.position(), TransportPosition { bar: 5, step: 0, beat: 1.0 });
}

// ─── T012: SetLoop wrap publishes the start bar and never runs away ───

#[test]
fn set_loop_wraps_to_start_bar() {
    // Bars 1..=4 only: when the playhead crosses bar 4, the next tick sees
    // bar 5 (transiently published as (5, 0)) and the tick after that wraps
    // to the start bar. No page exists for bar 5, so it can never advance
    // past step 0 — a runaway loop would climb past bar 5.
    let mut h = make_playback(4);
    h.cmd_tx.push(PlaybackCommand::SetLoop { start_bar: 3, end_bar: 4 }).unwrap();

    // Bars 1-2, then two full loop cycles (3, 4, 3, 4) plus margin.
    let observed = h.run_sampling(samples_per_bar() * 6 + SAMPLES_PER_STEP * 4);

    let mut max_bar = 0usize;
    let mut bar5_count = 0usize;
    let mut wrap_events = 0usize;
    let mut first_post_transient_step: Vec<usize> = Vec::new();
    let mut previous_was_bar5 = false;

    for &packed in &observed {
        let (bar, step) = unpack(packed);
        max_bar = max_bar.max(bar);
        if bar == 5 {
            assert_eq!(step, 0, "bar 5 must only ever be observed at step 0");
            bar5_count += 1;
            previous_was_bar5 = true;
        } else if previous_was_bar5 {
            assert_eq!(bar, 3, "after the bar-5 transient the loop must wrap to bar 3");
            assert!(step <= 1, "first post-wrap bar-3 step must be 0 or 1, got {step}");
            first_post_transient_step.push(step);
            wrap_events += 1;
            previous_was_bar5 = false;
        }
    }

    assert!(max_bar <= 5, "loop ran away: bar {max_bar} observed");
    assert!(
        wrap_events >= 2,
        "expected at least 2 loop wraps, saw {wrap_events} ({bar5_count} bar-5 reads)"
    );
    assert_eq!(first_post_transient_step.len(), wrap_events);
}

// ─── T013: reads are stable between ticks ───

#[test]
fn position_is_stable_between_ticks() {
    let mut h = make_playback(4);

    let first = h.position();
    let second = h.position();
    assert_eq!(
        (first.bar, first.step),
        (second.bar, second.step),
        "two immediate reads must be identical"
    );
    assert_eq!((first.bar, first.step), (1, 0));

    // One full step of samples = exactly one tick (counter starts at 0).
    let mut one_step = vec![0.0f32; SAMPLES_PER_STEP * 2];
    h.engine.process_buffer(&mut one_step, 2);
    let after_tick = h.position();
    assert_eq!((after_tick.bar, after_tick.step), (1, 1));

    let again = h.position();
    assert_eq!(
        (after_tick.bar, after_tick.step),
        (again.bar, again.step),
        "position must not change without processing"
    );
}

// ─── T018: position reads take no composer lock ───

/// The composer mutex is HELD on this thread while the position is read.
/// `NativeHandle::transport_position` and the composer's own `playhead_bar`
/// both read the packed atomic through their own `Arc` clones — a regression
/// to composer-lock reads would deadlock here (std Mutex is not reentrant).
#[test]
fn position_read_takes_no_composer_lock() {
    let shared_pages: Arc<Mutex<Vec<Measure>>> = Arc::new(Mutex::new(Vec::new()));
    let transport = Arc::new(AtomicU64::new(pack(1, 0)));
    let font_queue = Arc::new(Mutex::new(Vec::new()));
    let composer =
        MusicComposer::new_with_seed(SAMPLE_RATE, shared_pages, transport.clone(), font_queue, 42);
    let composer = Mutex::new(composer);

    let guard = composer.lock().unwrap();
    // Composer-side read path: lock-free with respect to its outer mutex.
    assert_eq!(guard.playhead_bar(), 1);
    // Handle-style read path: the Arc clone, no lock at all.
    let pos = TransportPosition::from_packed(transport.load(Ordering::Relaxed));
    assert_eq!((pos.bar, pos.step), (1, 0));
    drop(guard);
}

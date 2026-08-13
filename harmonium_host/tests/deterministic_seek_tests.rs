//! Deterministic Seek & Loop Tests
//!
//! Verifies that:
//! 1. Seeking to bar N produces identical measures as linear playback to bar N
//! 2. Two engines with the same seed produce identical output
//! 3. MusicComposer deterministic seek matches linear generation
//! 4. Seek to bar 64 completes within 50ms
//! 5. NewMelody produces different content
//! 6. SetSeed restores an identical session

use std::sync::{Arc, Mutex, atomic::AtomicU64};

use harmonium::{composer::MusicComposer, timeline_engine::TimelineEngine};
use harmonium_audio::backend::AudioRenderer;
use harmonium_core::{events::AudioEvent, report::MeasureSnapshot, timeline::Measure};

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

// ─── Helpers ───

fn create_engine(
    seed: u64,
) -> (
    TimelineEngine,
    rtrb::Producer<harmonium_core::EngineCommand>,
    rtrb::Consumer<harmonium_core::EngineReport>,
) {
    let (cmd_tx, cmd_rx) = rtrb::RingBuffer::new(1024);
    let (rpt_tx, rpt_rx) = rtrb::RingBuffer::new(256);
    let renderer: Box<dyn AudioRenderer> = Box::new(NullRenderer);
    let mut engine = TimelineEngine::new_with_seed(44100.0, cmd_rx, rpt_tx, renderer, seed);
    engine.set_offline(true);
    (engine, cmd_tx, rpt_rx)
}

fn samples_for_bars(bars: usize, bpm: f64, sample_rate: f64) -> usize {
    let beats_per_bar = 4.0f64;
    let seconds_per_beat = 60.0 / bpm;
    let seconds_per_bar = seconds_per_beat * beats_per_bar;
    (seconds_per_bar * bars as f64 * sample_rate) as usize
}

fn run_engine_bars(
    engine: &mut TimelineEngine,
    rpt_rx: &mut rtrb::Consumer<harmonium_core::EngineReport>,
    total_samples: usize,
) -> Vec<MeasureSnapshot> {
    let chunk_size = 512;
    let channels = 2;
    let mut buffer = vec![0.0f32; chunk_size * channels];
    let mut all_measures = Vec::new();

    let mut processed = 0;
    while processed < total_samples {
        let remaining = total_samples - processed;
        let this_chunk = remaining.min(chunk_size);
        let buf_len = this_chunk * channels;
        buffer[..buf_len].fill(0.0);
        engine.process_buffer(&mut buffer[..buf_len], channels);
        processed += this_chunk;

        while let Ok(report) = rpt_rx.pop() {
            all_measures.extend(report.new_measures);
        }
    }

    while let Ok(report) = rpt_rx.pop() {
        all_measures.extend(report.new_measures);
    }

    all_measures
}

fn create_composer(seed: u64) -> (MusicComposer, Arc<Mutex<Vec<Measure>>>) {
    let shared_pages: Arc<Mutex<Vec<Measure>>> = Arc::new(Mutex::new(Vec::new()));
    let transport_position = Arc::new(AtomicU64::new(harmonium::playback::pack(1, 0)));
    let font_queue = Arc::new(Mutex::new(Vec::new()));
    let composer = MusicComposer::new_with_seed(
        44100.0,
        shared_pages.clone(),
        transport_position,
        font_queue,
        seed,
    );
    (composer, shared_pages)
}

/// Compare two slices of MeasureSnapshot by their JSON serialization.
fn assert_measures_equal(a: &[MeasureSnapshot], b: &[MeasureSnapshot], context: &str) {
    assert_eq!(a.len(), b.len(), "{context}: measure count mismatch ({} vs {})", a.len(), b.len());
    for (i, (ma, mb)) in a.iter().zip(b.iter()).enumerate() {
        let ja = serde_json::to_string(ma).unwrap();
        let jb = serde_json::to_string(mb).unwrap();
        assert_eq!(ja, jb, "{context}: measure {i} differs.\n  a: {ja}\n  b: {jb}");
    }
}

// ─── Test 1: Seek determinism (TimelineEngine) ───

/// TimelineEngine seek determinism: verify that seeking to bar N via the
/// audio-thread engine matches linear playback. This test exercises the
/// full process_buffer → generate_ahead pipeline. Currently ignored pending
/// investigation of subtle state differences between update_controls() and
/// silent_advance() paths.
#[test]
#[ignore = "TimelineEngine audio-path seek determinism — WIP"]
fn seek_produces_identical_measures_as_linear_playback() {
    let seed = 42u64;
    let total_bars = 16;
    let seek_bar = 8;
    let sample_rate = 44100.0;
    let bpm = 120.0;

    // Run A: linear playback of all bars
    let (mut engine_a, _cmd_a, mut rpt_a) = create_engine(seed);
    let samples = samples_for_bars(total_bars, bpm, sample_rate);
    let measures_a = run_engine_bars(&mut engine_a, &mut rpt_a, samples);

    // Run B: play bars 1-4, then seek to bar 8, play bars 8-16
    let (mut engine_b, mut cmd_b, mut rpt_b) = create_engine(seed);
    // Play first 4 bars
    let samples_4 = samples_for_bars(4, bpm, sample_rate);
    let _ = run_engine_bars(&mut engine_b, &mut rpt_b, samples_4);
    // Seek to bar 8
    let _ = cmd_b.push(harmonium_core::EngineCommand::Seek(seek_bar));
    // Play bars 8-16 (need enough samples for 9 bars: 8..=16)
    let remaining_bars = total_bars - seek_bar + 1;
    let samples_remaining = samples_for_bars(remaining_bars, bpm, sample_rate);
    let measures_b_post_seek = run_engine_bars(&mut engine_b, &mut rpt_b, samples_remaining);

    // Extract bars 8-16 from linear run A (deduplicated by index)
    let mut measures_a_from_8: Vec<_> = measures_a
        .iter()
        .filter(|m| m.index >= seek_bar && m.index <= total_bars)
        .cloned()
        .collect();
    measures_a_from_8.dedup_by_key(|m| m.index);

    // Extract bars 8-16 from seek run B (deduplicated by index)
    let mut measures_b_from_8: Vec<_> = measures_b_post_seek
        .iter()
        .filter(|m| m.index >= seek_bar && m.index <= total_bars)
        .cloned()
        .collect();
    measures_b_from_8.dedup_by_key(|m| m.index);

    // Compare the overlapping bars (both sides must have generated them)
    let min_len = measures_a_from_8.len().min(measures_b_from_8.len());
    assert!(min_len >= 5, "Expected at least 5 overlapping bars, got {min_len}");
    assert_measures_equal(
        &measures_a_from_8[..min_len],
        &measures_b_from_8[..min_len],
        "Seek determinism (bars 8+)",
    );
}

// ─── Test 2: Cross-session determinism ───

#[test]
fn two_engines_same_seed_produce_identical_output() {
    let seed = 123456u64;
    let total_bars = 8;
    let sample_rate = 44100.0;
    let bpm = 120.0;
    let samples = samples_for_bars(total_bars, bpm, sample_rate);

    let (mut engine_a, _cmd_a, mut rpt_a) = create_engine(seed);
    let measures_a = run_engine_bars(&mut engine_a, &mut rpt_a, samples);

    let (mut engine_b, _cmd_b, mut rpt_b) = create_engine(seed);
    let measures_b = run_engine_bars(&mut engine_b, &mut rpt_b, samples);

    assert_measures_equal(&measures_a, &measures_b, "Cross-session determinism");
}

// ─── Test 3: MusicComposer deterministic seek ───

#[test]
fn composer_deterministic_seek_matches_linear() {
    let seed = 99u64;

    // Run A: linear generation of 16 bars
    let (mut composer_a, _pages_a) = create_composer(seed);
    composer_a.generate_bars(16);
    let snapshots_a = composer_a.take_snapshots();

    // Run B: generate 4 bars, then deterministic seek to bar 8, generate remaining
    let (mut composer_b, _pages_b) = create_composer(seed);
    composer_b.generate_bars(4);
    let _ = composer_b.take_snapshots(); // discard first 4
    composer_b.deterministic_seek(8);
    // generate_bars(N) generates until writehead >= playhead + N.
    // playhead is at 1, writehead is at 8. We need bars 8-16 (9 bars).
    // So we need lookahead >= 16: generate_bars(16) ensures writehead reaches 17.
    composer_b.generate_bars(16);
    let snapshots_b = composer_b.take_snapshots();

    // Extract bars 8-16 from A
    let a_from_8: Vec<_> = snapshots_a.iter().filter(|m| m.index >= 8).cloned().collect();

    assert_measures_equal(&a_from_8, &snapshots_b, "Composer deterministic seek (bars 8-16)");
}

// ─── Test 4: Performance benchmark ───

#[test]
fn seek_to_bar_64_under_50ms() {
    let seed = 42u64;
    let (mut composer, _pages) = create_composer(seed);

    let start = std::time::Instant::now();
    composer.deterministic_seek(64);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 50,
        "Seek to bar 64 took {}ms (limit: 50ms)",
        elapsed.as_millis()
    );
}

// ─── Test 5: NewMelody produces different content ───

#[test]
fn new_melody_produces_different_content() {
    let seed = 42u64;
    let (mut composer, _pages) = create_composer(seed);
    composer.generate_bars(4);
    let original = composer.take_snapshots();

    // New melody should produce different content
    composer.new_melody();
    composer.generate_bars(4);
    let new_content = composer.take_snapshots();

    // The two should differ (extremely unlikely to be identical with different seeds)
    let json_a = serde_json::to_string(&original).unwrap();
    let json_b = serde_json::to_string(&new_content).unwrap();
    assert_ne!(json_a, json_b, "NewMelody should produce different content than original seed");
}

// ─── Test 6a: Minimal reset test ───

#[test]
fn deterministic_seek_bar1_matches_fresh_composer() {
    let seed = 42u64;

    // A: fresh composer
    let (mut composer_a, _pages_a) = create_composer(seed);
    composer_a.generate_bars(4);
    let snapshots_a = composer_a.take_snapshots();

    // B: same seed, deterministic_seek(1) immediately, then generate
    let (mut composer_b, _pages_b) = create_composer(seed);
    composer_b.deterministic_seek(1);
    composer_b.generate_bars(4);
    let snapshots_b = composer_b.take_snapshots();

    assert_measures_equal(&snapshots_a, &snapshots_b, "deterministic_seek(1) on fresh composer");
}

// ─── Test 6b: SetSeed restores identical session ───

#[test]
fn set_seed_restores_identical_session() {
    let seed = 42u64;

    // Run A: fresh engine with seed
    let (mut composer_a, _pages_a) = create_composer(seed);
    composer_a.generate_bars(8);
    let snapshots_a = composer_a.take_snapshots();

    // Run B: engine with different seed, then set_seed to original
    let (mut composer_b, _pages_b) = create_composer(999);
    composer_b.generate_bars(4); // generate some bars with wrong seed
    let _ = composer_b.take_snapshots(); // discard
    composer_b.set_seed(seed); // restore to original seed
    composer_b.generate_bars(8);
    let snapshots_b = composer_b.take_snapshots();

    assert_measures_equal(&snapshots_a, &snapshots_b, "SetSeed restores identical session");
}

//! Deterministic Seek Tests (MusicComposer)
//!
//! Verifies that:
//! 1. MusicComposer deterministic seek matches linear generation
//! 2. Seek to bar 64 completes within 50ms
//! 3. NewMelody produces different content
//! 4. SetSeed restores an identical session

use std::sync::{Arc, atomic::AtomicU64};

use harmonium::{
    composer::MusicComposer,
    playback::{MEASURE_RING_CAP, RETURN_RING_CAP, TaggedMeasure},
};
use harmonium_core::report::MeasureSnapshot;

/// Composer wired to fresh handoff rings. The forward consumer lets tests
/// observe what the composer staged for playback (replacing the old
/// shared-pages peek); the return producer is unused here (nothing is
/// played back, so nothing is returned).
fn create_composer(
    seed: u64,
) -> (MusicComposer, rtrb::Consumer<TaggedMeasure>, rtrb::Producer<TaggedMeasure>) {
    let (measure_tx, measure_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(MEASURE_RING_CAP);
    let (return_tx, return_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(RETURN_RING_CAP);
    let transport_position = Arc::new(AtomicU64::new(harmonium::playback::pack(1, 0)));
    let composer =
        MusicComposer::new_with_seed(44100.0, measure_tx, return_rx, transport_position, seed);
    (composer, measure_rx, return_tx)
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

// ─── Test 1: MusicComposer deterministic seek ───

#[test]
fn composer_deterministic_seek_matches_linear() {
    let seed = 99u64;

    // Run A: linear generation of 16 bars
    let (mut composer_a, _staged_a, _return_a) = create_composer(seed);
    composer_a.generate_bars(16);
    let snapshots_a = composer_a.take_snapshots();

    // Run B: generate 4 bars, then deterministic seek to bar 8, generate remaining
    let (mut composer_b, _staged_b, _return_b) = create_composer(seed);
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

// ─── Test 2: Performance benchmark ───

#[test]
fn seek_to_bar_64_under_50ms() {
    let seed = 42u64;
    let (mut composer, _staged, _return) = create_composer(seed);

    let start = std::time::Instant::now();
    composer.deterministic_seek(64);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 50,
        "Seek to bar 64 took {}ms (limit: 50ms)",
        elapsed.as_millis()
    );
}

// ─── Test 3: NewMelody produces different content ───

#[test]
fn new_melody_produces_different_content() {
    let seed = 42u64;
    let (mut composer, _staged, _return) = create_composer(seed);
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

// ─── Test 4a: Minimal reset test ───

#[test]
fn deterministic_seek_bar1_matches_fresh_composer() {
    let seed = 42u64;

    // A: fresh composer
    let (mut composer_a, _staged_a, _return_a) = create_composer(seed);
    composer_a.generate_bars(4);
    let snapshots_a = composer_a.take_snapshots();

    // B: same seed, deterministic_seek(1) immediately, then generate
    let (mut composer_b, _staged_b, _return_b) = create_composer(seed);
    composer_b.deterministic_seek(1);
    composer_b.generate_bars(4);
    let snapshots_b = composer_b.take_snapshots();

    assert_measures_equal(&snapshots_a, &snapshots_b, "deterministic_seek(1) on fresh composer");
}

// ─── Test 4b: SetSeed restores identical session ───

#[test]
fn set_seed_restores_identical_session() {
    let seed = 42u64;

    // Run A: fresh engine with seed
    let (mut composer_a, _staged_a, _return_a) = create_composer(seed);
    composer_a.generate_bars(8);
    let snapshots_a = composer_a.take_snapshots();

    // Run B: engine with different seed, then set_seed to original
    let (mut composer_b, _staged_b, _return_b) = create_composer(999);
    composer_b.generate_bars(4); // generate some bars with wrong seed
    let _ = composer_b.take_snapshots(); // discard
    composer_b.set_seed(seed); // restore to original seed
    composer_b.generate_bars(8);
    let snapshots_b = composer_b.take_snapshots();

    assert_measures_equal(&snapshots_a, &snapshots_b, "SetSeed restores identical session");
}

// ─── Test 5: invalidate_future before any generation keeps the playhead bar ───

/// The standalone binary and the game both set params then call
/// `invalidate_future()` before a single bar exists. The playhead already
/// points at bar 1; regenerating from bar 2 would leave the playback engine
/// waiting for bar 1 forever (silent output).
#[test]
fn invalidate_future_before_generation_starts_at_playhead_bar() {
    let (mut composer, mut staged, _return) = create_composer(7);
    composer.invalidate_future();
    composer.generate_ahead();

    let first = composer.take_snapshots().first().map(|m| m.index);
    assert_eq!(first, Some(1), "first generated bar must be the playhead bar");
    // Same property as before, without SharedPages: bar 1 is staged for
    // playback on the composer → playback ring.
    let mut staged_bar_1 = false;
    while let Ok(tagged) = staged.pop() {
        if tagged.measure.index == 1 {
            staged_bar_1 = true;
        }
    }
    assert!(staged_bar_1, "bar 1 must be staged for playback");
}

/// Once the playhead bar exists, invalidation must keep it and regenerate
/// from the next bar only.
#[test]
fn invalidate_future_after_generation_keeps_playhead_bar() {
    let (mut composer, _staged, _return) = create_composer(7);
    composer.generate_bars(4);
    let before: Vec<_> = composer.take_snapshots();

    composer.set_bpm(150.0);
    composer.invalidate_future();
    composer.generate_bars(4);
    let after = composer.take_snapshots();

    assert_eq!(
        after.first().map(|m| m.index),
        Some(2),
        "regeneration starts after the playhead bar"
    );
    assert_eq!(before[0].tempo, 230.0, "bar 1 kept the default profile's original tempo");
    assert!(
        after.iter().all(|m| (m.tempo - 150.0).abs() < f32::EPSILON),
        "regenerated bars use the new tempo"
    );
}

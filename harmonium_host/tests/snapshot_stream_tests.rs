//! Non-regression filet for the decoupled engine's snapshot stream (spec 006).
//!
//! The decoupled path (MusicComposer + PlaybackEngine) is driven through
//! `create_offline_engine` at a fixed seed, exactly like the real-time path:
//! `composer.generate_ahead()` interleaved with `playback.process_buffer()`.
//! The full `MeasureSnapshot` stream is frozen as a golden JSON reference.
//!
//! This test must stay green across the transport-position change: that
//! change is an addition of a read of the shared position, not a
//! modification of generation (FR-004). It runs against the shared atomic
//! on both sides — the composer reads it during generation-on-demand, the
//! playback engine writes it every buffer — so a wrong bar derivation on
//! either side changes *how many* bars get generated and breaks the golden.
//!
//! To regenerate golden files, delete `tests/golden_measures/` and re-run tests.

use harmonium::audio::{AudioBackendType, create_offline_engine};
use harmonium_core::report::MeasureSnapshot;

const SEED: u64 = 42;
const BARS: usize = 8;
const SAMPLE_RATE: f64 = 44100.0;
const BPM: f64 = 120.0;

/// Total samples for `bars` of 4/4 at `BPM`.
fn samples_for_bars(bars: usize) -> usize {
    let beats_per_bar = 4.0f64;
    let seconds_per_beat = 60.0 / BPM;
    let seconds_per_bar = seconds_per_beat * beats_per_bar;
    (seconds_per_bar * bars as f64 * SAMPLE_RATE) as usize
}

/// Directory for golden measure JSON files (relative to crate root).
fn golden_dir() -> std::path::PathBuf {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.join("tests").join("golden_measures")
}

/// Compare collected measures against a golden file, or create it if missing.
fn assert_golden(name: &str, measures: &[MeasureSnapshot]) {
    let dir = golden_dir();
    let path = dir.join(format!("{name}.json"));

    let actual_json = serde_json::to_string_pretty(measures).expect("Failed to serialize measures");

    if path.exists() {
        let expected_json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read golden file {}: {e}", path.display()));

        if actual_json != expected_json {
            // Write actual output next to golden for diffing
            let actual_path = dir.join(format!("{name}.actual.json"));
            std::fs::write(&actual_path, &actual_json).ok();

            // Find first divergence for a useful error message
            let (line, col) = first_diff_location(&expected_json, &actual_json);
            panic!(
                "Golden file mismatch for '{name}' at line {line}, col {col}.\n\
                 Expected: {}\n\
                 Actual:   {}\n\
                 Run `diff {} {}` to inspect.",
                path.display(),
                actual_path.display(),
                path.display(),
                actual_path.display(),
            );
        }
    } else {
        // First run: create golden file
        std::fs::create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("Failed to create golden dir {}: {e}", dir.display()));
        std::fs::write(&path, &actual_json)
            .unwrap_or_else(|e| panic!("Failed to write golden file {}: {e}", path.display()));
        eprintln!("Created golden file: {} ({} measures)", path.display(), measures.len());
    }
}

fn first_diff_location(a: &str, b: &str) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (ca, cb) in a.chars().zip(b.chars()) {
        if ca != cb {
            return (line, col);
        }
        if ca == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    // Lengths differ
    (line, col)
}

/// The filet: the snapshot stream of the decoupled engine is unchanged at a
/// fixed seed. Must be green on unmodified code, and stay green after the
/// transport-position change.
#[test]
fn snapshot_stream_unchanged() {
    let (mut composer, mut playback, _cmd_tx, _report_rx, _recordings) =
        create_offline_engine(None, AudioBackendType::FundSP, SAMPLE_RATE)
            .expect("create_offline_engine failed");

    // Fixed seed: reproducible across runs and machines. set_seed resets
    // writehead, shared pages and the RNG-driven session (key/scale) —
    // proven by `set_seed_restores_identical_session` in
    // deterministic_seek_tests.rs.
    composer.set_seed(SEED);
    composer.set_writehead_lookahead(16);

    // Drive playback for 8 bars, generating ahead before every buffer like
    // the real-time path does. The composer reads the shared playhead
    // position inside generate_ahead(); the playback engine writes it.
    let total_samples = samples_for_bars(BARS);
    let chunk_size = 1024;
    let channels = 2;
    let mut buffer = vec![0.0f32; chunk_size * channels];

    let mut processed = 0;
    while processed < total_samples {
        let remaining = total_samples - processed;
        let this_chunk = remaining.min(chunk_size);
        let buf_len = this_chunk * channels;

        composer.generate_ahead();
        buffer[..buf_len].fill(0.0);
        playback.process_buffer(&mut buffer[..buf_len], channels);
        processed += this_chunk;
    }

    // Freeze the FULL stream, not just the played bars: generation runs
    // ahead of the playhead, and how far it runs depends on the shared
    // position read by the composer — the interaction this change touches.
    let snapshots = composer.take_snapshots();
    assert!(snapshots.len() >= BARS, "Expected at least {BARS} snapshots, got {}", snapshots.len());
    assert_golden("snapshot_stream_seed42_8bars", &snapshots);
}

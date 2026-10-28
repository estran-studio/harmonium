//! Predictive-clock acceptance tests (phase 0, D1/D2).
//!
//! Drives the real offline `PlaybackEngine` with composer-generated measures
//! and verifies the D1 planning invariant end to end: the `next_pulse_frame`
//! published during a pulse is *exactly* the global frame where the playhead
//! enters the next pulse (0-frame error), across time signatures, buffer
//! sizes and a nominal BPM change between measures. Also unit-tests
//! [`BeatInfo::from_clock`] (phase interpolation and wall-clock conversion).

use std::time::{Duration, Instant};

use harmonium::{
    BeatInfo, TransportClock,
    audio::create_offline_engine,
    playback::{PlaybackCommand, TransportPosition},
};
use harmonium_core::params::{PulseSpec, TimeSignature};

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;
const N_BARS: usize = 64;
/// Nominal BPMs for the two halves of the run (both give whole-frame step
/// durations at 44.1 kHz, so the frame budget is exact).
const BPM1: f64 = 100.0;
const BPM2: f64 = 180.0;
/// Grid resolution (ticks per beat) the engine uses.
const TICKS_PER_BEAT: usize = 4;

/// Samples per grid step at a nominal BPM (the engine truncates to usize).
fn samples_per_step(bpm: f64) -> usize {
    (SAMPLE_RATE * 60.0 / bpm / 4.0) as usize
}

fn profile_for_meter(signature: TimeSignature) -> harmonium_core::StyleProfile {
    let mut profile = match (signature.numerator, signature.denominator) {
        (6, 8) => harmonium_core::StyleProfile::named("gigue_quebecoise").unwrap(),
        (3, 4) => harmonium_core::StyleProfile::named("valse_quebecoise").unwrap(),
        _ => harmonium_core::StyleProfile::builtin().clone(),
    };
    let pulse = PulseSpec::from_time_signature(signature, TICKS_PER_BEAT);
    profile.meter.time_signature = signature;
    profile.meter.steps_per_pulse = usize::from(pulse.steps_per_pulse);
    profile.meter.pulses_per_bar = usize::from(pulse.pulses_per_bar);
    profile
}

/// Drive 64 bars of `time_sig` through the offline engine at `buffer_size`
/// frames per `process_buffer`, with a BPM change between bars 1 and 2.
///
/// Returns `(pulse_transitions, max_bar)` — the number of pulse-boundary
/// transitions verified and the highest bar the playhead reached.
fn run_prediction(time_sig: TimeSignature, buffer_size: usize, swing_ratio: f32) -> (usize, usize) {
    let pulse = PulseSpec::from_time_signature(time_sig, TICKS_PER_BEAT);
    let steps_per_pulse = usize::from(pulse.steps_per_pulse);
    let pulses_per_bar = usize::from(pulse.pulses_per_bar);
    let steps_per_bar = time_sig.steps_per_bar(TICKS_PER_BEAT);
    assert_eq!(steps_per_pulse * pulses_per_bar, steps_per_bar, "pulse must tile the bar");

    let (mut composer, mut playback, mut cmd_tx, _report_rx, mut clock_reader, _recordings) =
        create_offline_engine(None, None, &[], SAMPLE_RATE).expect("create_offline_engine failed");

    // Bars 1..=32 at BPM1, then set_bpm + invalidation for bars 2..=64 at BPM2.
    composer.set_profile(profile_for_meter(time_sig)).expect("valid clock-test profile");
    composer.set_bpm(BPM1 as f32);
    composer.set_swing_ratio(swing_ratio);
    composer.sync_generator();
    composer.set_writehead_lookahead(64);
    composer.generate_bars(32);

    composer.set_bpm(BPM2 as f32);
    composer.invalidate_future();
    if let Some((bar, _)) = composer.take_pending_drop() {
        let _ = cmd_tx.push(PlaybackCommand::DropFrom { bar });
    }
    composer.generate_bars(N_BARS);

    // Exact frame budget for 64 bars: bar 1 at BPM1, bars 2..=64 at BPM2.
    let sps1 = samples_per_step(BPM1);
    let sps2 = samples_per_step(BPM2);
    let total_frames = steps_per_bar as u64 * sps1 as u64
        + (N_BARS as u64 - 1) * steps_per_bar as u64 * sps2 as u64;

    let mut buffer = vec![0.0f32; buffer_size * CHANNELS];
    let mut frames_driven = 0u64;
    let mut last_pulse_start: Option<u64> = None;
    let mut last_next_pulse: Option<u64> = None;
    let mut pulse_transitions = 0usize;
    let mut max_bar = 0usize;
    let mut ramp_sent = false;

    while frames_driven < total_frames {
        composer.generate_ahead();
        buffer.fill(0.0);
        playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
        frames_driven += buffer_size as u64;

        let clock = clock_reader.read();

        // Launch a tempo ramp mid-measure (bar 2, pulse 1): the prediction
        // must stay exact *during* the ramp (D4 + D1 invariant).
        if !ramp_sent && clock.position.bar == 2 && clock.pulse_in_bar == 1 {
            let _ = cmd_tx.push(PlaybackCommand::SetTempo { bpm: 240.0, ramp_pulses: 8 });
            ramp_sent = true;
        }

        // The D1 invariant: when the playhead enters a new pulse, its start
        // frame equals the previous pulse's `next_pulse_frame` — 0-frame error.
        if let (Some(prev_start), Some(prev_next)) = (last_pulse_start, last_next_pulse)
            && clock.pulse_start_frame > prev_start
        {
            assert_eq!(
                clock.pulse_start_frame,
                prev_next,
                "{}:{} swing={swing_ratio} buffer={}: predicted next_pulse_frame {} != actual {}",
                time_sig.numerator,
                time_sig.denominator,
                buffer_size,
                prev_next,
                clock.pulse_start_frame,
            );
            pulse_transitions += 1;
        }

        // Skip the pre-first-freeze state (pulse_start_frame == 0); verify
        // pulse/is_downbeat consistency against bar/step for every real pulse.
        if clock.pulse_start_frame > 0 {
            last_pulse_start = Some(clock.pulse_start_frame);
            last_next_pulse = Some(clock.next_pulse_frame);

            assert_eq!(
                usize::from(clock.pulse_in_bar),
                clock.position.step / steps_per_pulse,
                "{}:{}: pulse_in_bar {} inconsistent with step {}",
                time_sig.numerator,
                time_sig.denominator,
                clock.pulse_in_bar,
                clock.position.step,
            );

            let beat = BeatInfo::from_clock(&clock, Instant::now());
            assert_eq!(beat.bar, clock.position.bar);
            assert_eq!(beat.pulse, clock.pulse_in_bar + 1);
            assert_eq!(beat.pulses_per_bar, pulse.pulses_per_bar);
            assert!(beat.pulse <= beat.pulses_per_bar);
            assert_eq!(beat.is_downbeat, beat.pulse == 1);
            assert_eq!(beat.is_backbeat, beat.pulse.is_multiple_of(2));
            assert_eq!(beat.next_pulse_frame, clock.next_pulse_frame);
        }

        max_bar = max_bar.max(clock.position.bar);
    }

    assert!(
        max_bar >= N_BARS,
        "{}:{} buffer={}: playhead only reached bar {max_bar}, want {N_BARS}",
        time_sig.numerator,
        time_sig.denominator,
        buffer_size,
    );
    assert!(
        pulse_transitions >= (N_BARS - 1) * pulses_per_bar,
        "{}:{} buffer={}: only {pulse_transitions} pulse transitions verified",
        time_sig.numerator,
        time_sig.denominator,
        buffer_size,
    );

    (pulse_transitions, max_bar)
}

/// Acceptance (1): `next_pulse_frame` is exact (0-frame error) across 4/4,
/// 2/2, 6/8, 3/4, on 64 measures, at buffer sizes 64, 333 and 1024, at swing
/// 0.5 and 0.62, with a nominal BPM change between measures (`set_bpm` +
/// invalidation) and a `SetTempo { 240, 8 }` ramp launched mid-measure.
#[test]
fn prediction_is_exact_across_meters_and_buffer_sizes() {
    let signatures = [
        TimeSignature::new(4, 4),
        TimeSignature::new(2, 2),
        TimeSignature::new(6, 8),
        TimeSignature::new(3, 4),
    ];
    for ts in signatures {
        for &swing in &[0.5f32, 0.62] {
            for &buffer_size in &[64usize, 333, 1024] {
                let (transitions, max_bar) = run_prediction(ts, buffer_size, swing);
                eprintln!(
                    "[observed-proof] {}/{} swing={swing} buffer={}: {} pulse transitions, playhead reached bar {}",
                    ts.numerator, ts.denominator, buffer_size, transitions, max_bar
                );
            }
        }
    }
}

/// Unit test for [`BeatInfo::from_clock`]: phase at 0, mid-pulse, just before
/// the boundary (bounded < 1.0), and `next_pulse_at` consistent with
/// `sample_rate`.
#[test]
fn beat_info_from_clock_phase_and_timing() {
    let sample_rate = 48_000u32;
    let anchor = Instant::now();

    // Pulse 2 (index 1) of bar 3: frames [1000, 4000), period 3000 frames.
    let clock = TransportClock {
        position: TransportPosition { bar: 3, step: 4, beat: 2.0 },
        pulse_in_bar: 1,
        pulses_per_bar: 4,
        pulse_start_frame: 1000,
        next_pulse_frame: 4000,
        buffer_start_frame: 0,
        buffer_start_instant: anchor,
        sample_rate,
        ..TransportClock::initial(sample_rate)
    };

    let frame_to_instant =
        |f: u64| anchor + Duration::from_secs_f64(f as f64 / f64::from(sample_rate));

    // Phase 0 at the pulse start.
    let at_start = BeatInfo::from_clock(&clock, frame_to_instant(1000));
    assert_eq!(at_start.phase, 0.0);
    assert_eq!(at_start.pulse, 2);
    assert_eq!(at_start.pulses_per_bar, 4);
    assert_eq!(at_start.bar, 3);
    assert!(!at_start.is_downbeat, "pulse 2 is not a downbeat");
    assert!(at_start.is_backbeat, "pulse 2 is even (backbeat)");
    assert_eq!(at_start.next_pulse_frame, 4000);

    // Mid-pulse: phase 0.5.
    let at_mid = BeatInfo::from_clock(&clock, frame_to_instant(2500));
    assert!((at_mid.phase - 0.5).abs() < 1e-6, "mid phase {}", at_mid.phase);

    // Just before the boundary: phase < 1.0 and close to it.
    let at_before = BeatInfo::from_clock(&clock, frame_to_instant(3999));
    assert!(at_before.phase < 1.0, "phase must be bounded < 1.0, got {}", at_before.phase);
    assert!((at_before.phase - 2999.0 / 3000.0).abs() < 1e-3);

    // Past the boundary: clamped back to < 1.0 (the next pulse is a re-read).
    let at_after = BeatInfo::from_clock(&clock, frame_to_instant(5000));
    assert!(at_after.phase < 1.0);

    // `next_pulse_at` and `pulse_period` follow the sample rate exactly.
    assert_eq!(at_start.next_pulse_at, frame_to_instant(4000));
    assert_eq!(at_start.pulse_period, Duration::from_secs_f64(3000.0 / f64::from(sample_rate)));
}

/// Acceptance (3): On a 4/4 → 2/2 transition (`set_time_signature` + `invalidate_future`
/// during playback), `pulses_per_bar` read during each pulse equals
/// `Measure.pulse.pulses_per_bar` of the played measure, and `pulse <= pulses_per_bar`
/// always holds.
#[test]
fn time_signature_transition_preserves_pulses_per_bar_consistency() {
    let (mut composer, mut playback, mut cmd_tx, _report_rx, mut clock_reader, _recordings) =
        create_offline_engine(None, None, &[], SAMPLE_RATE).expect("create_offline_engine failed");

    composer.set_profile(profile_for_meter(TimeSignature::new(4, 4))).unwrap();
    composer.set_bpm(120.0);
    composer.set_swing_ratio(0.5);
    composer.sync_generator();
    composer.set_writehead_lookahead(8);
    composer.generate_bars(8);

    let buffer_size = 256usize;
    let mut buffer = vec![0.0f32; buffer_size * CHANNELS];
    let mut transition_sent = false;
    let mut saw_4_4 = false;
    let mut saw_2_2 = false;
    let mut verified_pulses = 0usize;

    while clock_reader.read().position.bar < 6 {
        composer.generate_ahead();
        buffer.fill(0.0);
        playback.process_buffer(&mut buffer, CHANNELS, Instant::now());

        let clock = clock_reader.read();

        if !transition_sent && clock.position.bar >= 2 {
            composer.set_profile(profile_for_meter(TimeSignature::new(2, 2))).unwrap();
            composer.set_bpm(120.0);
            composer.set_swing_ratio(0.5);
            composer.invalidate_future();
            if let Some((bar, _)) = composer.take_pending_drop() {
                let _ = cmd_tx.push(PlaybackCommand::DropFrom { bar });
            }
            composer.generate_ahead();
            transition_sent = true;
        }

        if clock.pulse_start_frame > 0 {
            let beat = BeatInfo::from_clock(&clock, Instant::now());
            let measure = composer
                .timeline_measure(clock.position.bar)
                .expect("played bar must exist in timeline");

            assert_eq!(
                beat.pulses_per_bar, measure.pulse.pulses_per_bar,
                "bar {}: beat.pulses_per_bar {} != measure.pulse.pulses_per_bar {}",
                beat.bar, beat.pulses_per_bar, measure.pulse.pulses_per_bar
            );
            assert_eq!(
                clock.pulses_per_bar, measure.pulse.pulses_per_bar,
                "bar {}: clock.pulses_per_bar {} != measure.pulse.pulses_per_bar {}",
                clock.position.bar, clock.pulses_per_bar, measure.pulse.pulses_per_bar
            );
            assert!(
                beat.pulse <= beat.pulses_per_bar,
                "bar {}: pulse {} exceeds pulses_per_bar {}",
                beat.bar,
                beat.pulse,
                beat.pulses_per_bar
            );
            assert!(
                clock.pulse_in_bar < clock.pulses_per_bar,
                "bar {}: pulse_in_bar {} >= pulses_per_bar {}",
                clock.position.bar,
                clock.pulse_in_bar,
                clock.pulses_per_bar
            );

            if beat.pulses_per_bar == 4 {
                saw_4_4 = true;
            } else if beat.pulses_per_bar == 2 {
                saw_2_2 = true;
            }
            verified_pulses += 1;
        }
    }

    assert!(transition_sent, "transition must have been triggered");
    assert!(saw_4_4, "must have observed 4/4 pulses before transition");
    assert!(saw_2_2, "must have observed 2/2 pulses after transition");
    assert!(verified_pulses > 0, "must have verified pulses");
}

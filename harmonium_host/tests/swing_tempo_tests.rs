//! Swing (D3) and tempo-ramp (D4) acceptance tests, phase 0.
//!
//! Drives the real offline `PlaybackEngine` and verifies the two timing
//! features on whole frames: a swung 4/4 pulse is exactly one rounded quarter
//! (22050 frames at 120 BPM, measure 88200), and a `SetTempo { 240, 8 }` ramp
//! lands exactly on a 240 BPM quarter (11025 frames) then `ClearTempo`
//! reverts. The `swing_unit` pure function is also unit-tested.

use std::time::Instant;

use harmonium::{
    audio::create_offline_engine,
    playback::{PlaybackCommand, swing_unit},
};

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;

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
fn swing_quarter_lasts_exactly_22050_frames() {
    let (mut composer, mut playback, _cmd_tx, _report_rx, mut clock_reader, _recordings) =
        create_offline_engine(None, SAMPLE_RATE).expect("create_offline_engine failed");

    composer.set_time_signature(4, 4);
    composer.set_swing_ratio(0.6);
    composer.sync_generator();
    composer.set_writehead_lookahead(8);
    composer.generate_bars(8);

    let mut buffer = vec![0.0f32; 512 * CHANNELS];
    let mut checked = 0usize;
    let mut last_next = 0u64;
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
        if clock.position.bar > 3 {
            break;
        }
    }
    assert!(checked >= 12, "only checked {checked} pulses");
}

/// Acceptance (3) — tempo ramp: `SetTempo { 240, 8 }` received during a pulse
/// makes the next 8 pulse durations strictly decrease, the 8th landing exactly
/// on a 240 BPM quarter (11025 frames); `ClearTempo` reverts to the measure
/// tempo at the next boundary.
#[test]
fn tempo_ramp_reaches_target_then_clears() {
    let (mut composer, mut playback, mut cmd_tx, _report_rx, mut clock_reader, _recordings) =
        create_offline_engine(None, SAMPLE_RATE).expect("create_offline_engine failed");

    composer.set_time_signature(4, 4);
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
}

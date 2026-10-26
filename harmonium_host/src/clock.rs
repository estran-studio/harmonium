//! Predictive transport clock and the gameplay-facing `BeatInfo`.
//!
//! The audio callback publishes a [`TransportClock`] into a lock-free
//! [`triple_buffer`] (single writer). Any third thread — the game — reads it
//! through a [`ClockReader`] and derives a [`BeatInfo`] with
//! [`BeatInfo::from_clock`], interpolating the pulse phase in *frames* from
//! the wall clock. `Instant` therefore only appears in publication (the cpal
//! path) and in the reader; generation and playback never read it
//! (constitution II).

use std::time::{Duration, Instant};

use harmonium_core::time_plan::Flicks;

use crate::playback::TransportPosition;

/// Predictive clock published by the audio callback, lock-free.
///
/// The pulse fields are frozen at each pulse boundary (the D1 planning
/// invariant): `next_pulse_frame` is therefore exact by construction, not an
/// estimate. `buffer_start_frame`/`buffer_start_instant` anchor the clock to
/// the wall clock; everything else is expressed in engine frames (the
/// testable truth).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportClock {
    /// Transport position (bar, step, beat) — `step` is the step currently
    /// being played (one behind the packed `(bar, step)` atomic, which
    /// reports the upcoming step).
    pub position: TransportPosition,
    /// Dance-pulse index within the bar, 0-based.
    pub pulse_in_bar: u8,
    /// Total number of dance pulses in the current bar.
    pub pulses_per_bar: u8,
    /// Global frame counter at the start of the current pulse.
    pub pulse_start_frame: u64,
    /// Exact frame where the next pulse begins.
    pub next_pulse_frame: u64,
    /// First frame of the buffer in progress.
    pub buffer_start_frame: u64,
    /// Audible wall-clock instant of [`TransportClock::buffer_start_frame`].
    pub buffer_start_instant: Instant,
    /// Samples per second, for frame ↔ time conversions.
    pub sample_rate: u32,
    /// Global frame where the step in `position` started (its notes' frame).
    pub step_start_frame: u64,
    /// Instant of that step in the shared time plan (`harmonium_core::time_plan`),
    /// along the played measures: what the simulation computes for the same step.
    pub plan_instant: Flicks,
    /// Measured gap between the playhead and the plan, in frames: `step_start_frame`
    /// minus the plan's frame for `plan_instant` (anchored at the first played step).
    /// Non-zero after an underflow; published, never corrected silently.
    pub plan_gap_frames: i64,
}

impl TransportClock {
    /// The clock a fresh engine publishes before it has processed a buffer.
    /// Pulse fields are zero ("no pulse yet"); the first pulse boundary in
    /// the first buffer freezes them.
    #[must_use]
    pub fn initial(sample_rate: u32) -> Self {
        Self {
            position: TransportPosition { bar: 1, step: 0, beat: 1.0 },
            pulse_in_bar: 0,
            pulses_per_bar: 4,
            pulse_start_frame: 0,
            next_pulse_frame: 0,
            buffer_start_frame: 0,
            buffer_start_instant: Instant::now(),
            sample_rate,
            step_start_frame: 0,
            plan_instant: Flicks::ZERO,
            plan_gap_frames: 0,
        }
    }
}

/// The dance-pulse clock the gameplay acts on, derived from a
/// [`TransportClock`] at a given wall-clock instant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatInfo {
    /// 1-based bar.
    pub bar: usize,
    /// 1-based pulse within the bar.
    pub pulse: u8,
    /// Total number of dance pulses in the current bar.
    pub pulses_per_bar: u8,
    /// Phase within the pulse, `0.0..1.0`, interpolated in frames.
    pub phase: f32,
    /// `pulse == 1`.
    pub is_downbeat: bool,
    /// Even pulse (backbeat).
    pub is_backbeat: bool,
    /// Exact next-pulse frame (the prediction).
    pub next_pulse_frame: u64,
    /// Wall-clock instant of the next pulse.
    pub next_pulse_at: Instant,
    /// Duration of the current pulse.
    pub pulse_period: Duration,
    /// Playhead gap with the shared time plan, in frames (see
    /// [`TransportClock::plan_gap_frames`]).
    pub plan_gap_frames: i64,
}

impl BeatInfo {
    /// Derive the gameplay beat from a published clock at `now`.
    ///
    /// Frames are converted to wall time through the buffer anchor
    /// (`buffer_start_frame` / `buffer_start_instant`); the pulse phase is
    /// interpolated from `now`, clamped to `[0, 1)`.
    #[must_use]
    pub fn from_clock(clock: &TransportClock, now: Instant) -> Self {
        let sr = f64::from(clock.sample_rate);
        let period_frames = clock.next_pulse_frame.saturating_sub(clock.pulse_start_frame);
        let pulse_period = Duration::from_secs_f64(period_frames as f64 / sr);

        // Wall-clock elapsed since the buffer anchor, expressed in frames.
        let elapsed = now.saturating_duration_since(clock.buffer_start_instant);
        let current_frame = clock.buffer_start_frame as f64 + elapsed.as_secs_f64() * sr;

        let phase = if period_frames == 0 {
            0.0
        } else {
            let raw = (current_frame - clock.pulse_start_frame as f64) / period_frames as f64;
            (raw.clamp(0.0, 1.0) as f32).min(1.0 - f32::EPSILON)
        };

        let next_pulse_at = clock.buffer_start_instant
            + Duration::from_secs_f64(
                clock.next_pulse_frame.saturating_sub(clock.buffer_start_frame) as f64 / sr,
            );

        let pulse = clock.pulse_in_bar + 1;
        Self {
            bar: clock.position.bar,
            pulse,
            pulses_per_bar: clock.pulses_per_bar,
            phase,
            is_downbeat: pulse == 1,
            is_backbeat: pulse.is_multiple_of(2),
            next_pulse_frame: clock.next_pulse_frame,
            next_pulse_at,
            pulse_period,
            plan_gap_frames: clock.plan_gap_frames,
        }
    }
}

/// Reusable lock-free reader for the published [`TransportClock`], safe to
/// hand to a third thread (the game).
///
/// Wraps the `triple_buffer::Output` half of a single-producer/single-consumer
/// buffer: reading advances the consumer index, so [`ClockReader::read`] takes
/// `&mut self`. `Send` so the game thread can own it.
pub struct ClockReader {
    output: triple_buffer::Output<TransportClock>,
}

impl ClockReader {
    /// Wrap a `triple_buffer::Output` for lock-free reading (internal wiring).
    pub(crate) fn from_output(output: triple_buffer::Output<TransportClock>) -> Self {
        Self { output }
    }

    /// Read the latest published clock, copying it out.
    ///
    /// Lock-free: an atomic swap on the shared back-buffer index, no lock, no
    /// allocation. Returns the most recently published value.
    #[must_use]
    pub fn read(&mut self) -> TransportClock {
        *self.output.read()
    }
}

//! PlaybackEngine - Audio-thread playback, decoupled from music generation.
//!
//! Minimal audio-thread component: reads pre-generated measures from a ring buffer,
//! ticks the playhead, renders audio events. No generation, no allocations in steady state.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use arrayvec::{ArrayString, ArrayVec};
use harmonium_audio::backend::AudioRenderer;
use harmonium_core::{
    events::AudioEvent,
    log,
    params::{MusicalParams, PulseSpec, TimeSignature},
    timeline::{Measure, Playhead},
};

use crate::clock::TransportClock;

/// Capacity of the composer → playback staged-measure ring and of the
/// playback prefetch window. A loop longer than this is refused on the main
/// thread (see [`check_loop_len`]).
pub const MEASURE_WINDOW_CAP: usize = 16;
/// Capacity of the composer → playback ring (same bound as the window: the
/// composer stops staging when the ring is full and retries later).
pub const MEASURE_RING_CAP: usize = 16;
/// Capacity of the playback → composer return ring. It covers every box the
/// engine can hold at once (window + staged ring + held boxes), so returning
/// a box on the audio thread cannot fail.
pub const RETURN_RING_CAP: usize = 64;

/// A measure staged for the audio thread, tagged with the composer's
/// generation epoch. Boxes published before an invalidation carry an older
/// epoch and are treated as stale: never loaded, handed straight back.
/// Moving this value is a pointer move — no allocation, no deallocation.
pub struct TaggedMeasure {
    /// Generation epoch at stage time (see `MusicComposer::epoch`).
    pub epoch: u64,
    /// The staged measure. Only the main thread may drop it.
    pub measure: Box<Measure>,
}

/// Whether a loop region fits the playback window.
///
/// Loops longer than [`MEASURE_WINDOW_CAP`] bars are refused by the caller
/// (main thread) — the playback engine could not hold them without
/// underflowing. Bar numbers are normalized the same way as
/// `PlaybackCommand::SetLoop` (1-based, `end >= start`).
#[must_use]
pub fn check_loop_len(start_bar: usize, end_bar: usize) -> bool {
    let start = start_bar.max(1);
    let end = end_bar.max(start);
    end - start < MEASURE_WINDOW_CAP
}

/// RAII marker for the real-time handoff section: entering flags the audio
/// context (so `RTCheckAllocator` panics on alloc/dealloc in debug builds),
/// dropping clears it. Every handoff op (ring pop/push, window insert/remove,
/// box moves) must run inside this guard.
struct RtGuard;

impl RtGuard {
    fn enter() -> Self {
        harmonium_audio::realtime::rt_check::enter_audio_context();
        Self
    }
}

impl Drop for RtGuard {
    fn drop(&mut self) {
        harmonium_audio::realtime::rt_check::exit_audio_context();
    }
}

/// Grid resolution of the transport, in steps per beat (sixteenth notes).
/// `TransportPosition::beat` is derived from the step on this grid — it is
/// never stored separately.
pub const TICKS_PER_BEAT: usize = 4;

/// Transport position at the engine's grid resolution, packed into a single
/// lock-free shared value ([`pack`]). `bar` is 1-based, `step` is 0-based
/// within the measure, and `beat` is derived: `step / TICKS_PER_BEAT + 1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportPosition {
    /// 1-based bar.
    pub bar: usize,
    /// Step within the measure, 0-based, on the [`TICKS_PER_BEAT`] grid.
    pub step: usize,
    /// Derived beat: `step / TICKS_PER_BEAT + 1.0` (1.0 = first beat).
    pub beat: f32,
}

/// Pack a transport position into one 64-bit value: bar in the high 32
/// bits, step in the low 32 bits. One atomic, one load — a pair read this
/// way is coherent by construction (spec 006, R3).
pub const fn pack(bar: usize, step: usize) -> u64 {
    ((bar as u64) << 32) | (step as u64 & 0xFFFF_FFFF)
}

/// Unpack a packed transport position into `(bar, step)`.
pub const fn unpack(packed: u64) -> (usize, usize) {
    ((packed >> 32) as usize, (packed & 0xFFFF_FFFF) as usize)
}

/// Swing timing for one quarter note (unit = 4 steps).
///
/// Steps 0-1 last `ratio × quarter / 2` frames each, steps 2-3
/// `(1 − ratio) × quarter / 2`, rounded to whole frames with the rounding
/// remainder folded into the last step — so the unit sums to exactly
/// `quarter_frames` (no drift, the D3 invariant). `ratio = 0.5` yields a
/// straight quarter (four near-equal steps). Pure and deterministic.
#[must_use]
pub fn swing_unit(quarter_frames: usize, ratio: f32) -> [usize; 4] {
    let q = quarter_frames as f64;
    let r = f64::from(ratio);
    let first = (r * q / 2.0).round() as i64;
    let second = ((1.0 - r) * q / 2.0).round() as i64;
    let mut unit = [first, first, second, second];
    let sum: i64 = unit.iter().sum();
    unit[3] += quarter_frames as i64 - sum;
    unit.map(|d| d as usize)
}

impl TransportPosition {
    /// Decode a packed transport value into a [`TransportPosition`],
    /// deriving `beat` from the step.
    #[must_use]
    pub fn from_packed(packed: u64) -> Self {
        let (bar, step) = unpack(packed);
        Self { bar, step, beat: step as f32 / TICKS_PER_BEAT as f32 + 1.0 }
    }
}

/// MIDI channel reserved for monitoring the player's own input ("hear
/// yourself"). Channels 0–4 are the sequenced tracks (melody/chords/bass/heel/toe)
/// and 9 is GM percussion, so 8 is free and tonal.
pub const MONITOR_CHANNEL: u8 = 8;

/// A live note the player pressed on their MIDI controller, forwarded from
/// the MIDI input thread to the audio thread so it can be voiced through the
/// same synth as the backing track. Single-producer/single-consumer: the MIDI
/// callback owns the producer, the audio thread the consumer.
#[derive(Clone, Copy, Debug)]
pub struct LiveMidiEvent {
    /// true = note-on, false = note-off.
    pub on: bool,
    pub note: u8,
    pub velocity: u8,
}

/// Commands sent from the main thread to the PlaybackEngine (audio thread).
pub enum PlaybackCommand {
    SetChannelGain {
        channel: u8,
        gain: f32,
    },
    SetChannelMute {
        channel: u8,
        muted: bool,
    },
    SetChannelRoute {
        channel: u8,
        bank_id: i32,
    },
    SetVelocityBase {
        channel: u8,
        velocity: u8,
    },
    SetOutputMute(bool),
    SetMasterVolume(f32),
    Seek(usize),
    SeekPlayhead(usize),
    /// Evict every staged measure with `index >= bar`, handing the boxes
    /// back to the composer through the return ring. Sent by the main thread
    /// after `invalidate_future` / `invalidate_after_preview` / `reset` /
    /// `reset_timeline` / seek, before the composer publishes replacements
    /// (epoch tags make the order irrelevant for correctness).
    DropFrom {
        bar: usize,
    },
    SetLoop {
        start_bar: usize,
        end_bar: usize,
    },
    ClearLoop,
    StartRecording(harmonium_core::events::RecordFormat),
    StopRecording(harmonium_core::events::RecordFormat),
    MixerGains {
        lead: f32,
        chords: f32,
        bass: f32,
        snare: f32,
        hat: f32,
    },
    LoadFont {
        id: u32,
        bytes: Vec<u8>,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    /// Update the muted channels mask from the composer's musical params
    SetMutedChannels(Vec<bool>),
    /// Update musical params snapshot for recording/reporting
    UpdateMusicalParams(Box<MusicalParams>),
    /// Start (or ramp to) an audible tempo override (D4).
    ///
    /// `ramp_pulses == 0` jumps to `bpm` at the next pulse boundary;
    /// otherwise the audible tempo advances linearly over `ramp_pulses` pulse
    /// boundaries and lands exactly on `bpm`. Overrides the measure tempo
    /// until [`PlaybackCommand::ClearTempo`].
    SetTempo {
        bpm: f32,
        ramp_pulses: u8,
    },
    /// Revert to the measure tempo at the next pulse boundary.
    ClearTempo,
}

/// PlaybackEngine runs on the audio thread inside the CPAL closure.
///
/// It reads staged measures from the composer → playback ring into an
/// ordered prefetch window, ticks the playhead, and renders audio.
/// Generation is NOT done here — that's MusicComposer's job.
///
/// Real-time contract: the handoff (ring pop/push, window insert/remove, box
/// moves) performs no allocation, no deallocation and takes no lock. Played
/// boxes travel back through the return ring so only the main thread drops
/// them. A box is only ever loaded when its index equals the playhead bar;
/// otherwise the engine underflows (silence), exactly like a missing page
/// before.
pub struct PlaybackEngine {
    // Playhead
    playhead: Playhead,

    // Audio renderer
    renderer: Box<dyn AudioRenderer>,
    sample_rate: f64,

    // Timing
    sample_counter: usize,
    samples_per_step: usize,
    // Frozen frame length of the step being rendered, captured at its entry
    // (`enter_step`): the next entry renders a new capture, so a pulse
    // freeze mid-step never corrupts the rendering in flight.
    current_step_frames: usize,
    // Whether an entered step is rendering: its position/clock publication
    // happens at its completion (`complete_step`), mirroring the historical
    // post-step publication timing. False while underflowing (free-run) or
    // right after a seek (the in-flight step is cut by the jump).
    step_open: bool,
    // The step being rendered (bar 1-based, step 0-based in bar), captured at
    // its entry: `clock.position` reports it, the packed atomic the upcoming
    // step. Updated before the playhead advances, published at completion.
    entered_bar: usize,
    entered_step: usize,
    // Global frame counter: total frames rendered since engine start. The
    // clock's pulse/buffer fields live in this frame space.
    frame_counter: u64,
    // Per-step timing (D3 swing + D4 ramp): `step_durations[..step_count]`
    // is the frozen frame length of each step of the current pulse (the D1
    // invariant), `step_index` the step currently being rendered.
    step_durations: [usize; u8::MAX as usize],
    steps_per_quarter: f64,
    step_count: usize,
    step_index: usize,
    // Swing ratio of the measure under the playhead (0.5 = straight).
    swing_ratio: f32,
    // Nominal tempo of the measure under the playhead (Measure.tempo) and the
    // audible tempo actually used for timing (overridden by a tempo ramp).
    measure_bpm: f32,
    audible_bpm: f32,
    // Active tempo override (D4). `None` = follow the measure tempo.
    tempo_override: Option<TempoRamp>,
    // An explicit SetTempo wins over data-driven measure ramps.
    tempo_override_explicit: bool,

    // Predictive clock (D1): the current state plus the triple-buffer
    // producer half. `clock` is copied into the buffer on each publish (a
    // plain memcpy, no allocation, no lock). `current_pulse` is the
    // dance-pulse spec of the measure under the playhead, captured at load.
    clock: TransportClock,
    clock_tx: triple_buffer::Input<TransportClock>,
    current_pulse: PulseSpec,

    // Handoff: composer → playback staged measures (epoch-tagged boxes).
    measure_rx: rtrb::Consumer<TaggedMeasure>,
    // Handoff: playback → composer return ring (main thread deallocates).
    return_tx: rtrb::Producer<TaggedMeasure>,
    // Prefetch window, ordered by measure index.
    window: ArrayVec<WindowSlot, MEASURE_WINDOW_CAP>,
    // Highest epoch staged by the composer; older boxes are stale.
    epoch: u64,
    // Epoch of the box held by the playhead (current or finished slot).
    loaded_epoch: Option<u64>,

    // Command ring buffer (main thread → audio thread)
    cmd_rx: rtrb::Consumer<PlaybackCommand>,

    // Live MIDI monitor ring buffer (MIDI input thread → audio thread).
    // Notes the player presses, voiced through the synth on MONITOR_CHANNEL.
    live_rx: rtrb::Consumer<LiveMidiEvent>,

    // Report ring buffer (audio thread → main thread)
    report_tx: rtrb::Producer<harmonium_core::EngineReport>,

    // Shared transport position, packed (bar, step) — audio thread writes,
    // composer and lock-free readers read
    transport_position: Arc<AtomicU64>,

    // Mute state
    output_muted: bool,
    muted_channels: Vec<bool>,
    #[allow(dead_code)]
    last_muted_channels: Vec<bool>,

    // Loop region
    loop_region: Option<(usize, usize)>,

    // Recording state
    is_recording_wav: bool,
    is_recording_midi: bool,
    is_recording_musicxml: bool,

    // Mixer gains (cached for reporting)
    gain_lead: f32,
    gain_chords: f32,
    gain_bass: f32,
    gain_snare: f32,
    gain_hat: f32,

    // Musical params snapshot for reports
    musical_params: MusicalParams,
    // Signature of the measure under the playhead, captured at load.
    measure_sig: TimeSignature,
    // Tempo and signature last relayed to the renderer (MIDI file metas).
    // Compared at each measure load; the relay fires only on change, so
    // steady-state playback stays allocation-free.
    reported_bpm: f32,
    reported_signature: TimeSignature,

    // Report cache
    last_chord_name: ArrayString<64>,
    last_chord_root_offset: i32,
    last_chord_is_minor: bool,

    // True once the playhead has loaded its first measure.
    // Before that, output is zeroed to suppress DSP-graph init transients.
    first_measure_loaded: bool,
}

/// One window entry: a staged box plus its generation epoch.
struct WindowSlot {
    epoch: u64,
    measure: Box<Measure>,
}

/// An in-flight tempo ramp (D4): the audible tempo advances by `step` BPM at
/// each pulse boundary until `remaining` reaches zero, landing exactly on
/// `target`.
struct TempoRamp {
    target: f32,
    remaining: u8,
    step: f32,
}

impl PlaybackEngine {
    /// Create a new PlaybackEngine.
    ///
    /// Nine arguments is the whole wiring (renderer, three rings,
    /// transport, clock producer): bundling them would only rename the tuple.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sample_rate: f64,
        mut renderer: Box<dyn AudioRenderer>,
        measure_rx: rtrb::Consumer<TaggedMeasure>,
        return_tx: rtrb::Producer<TaggedMeasure>,
        cmd_rx: rtrb::Consumer<PlaybackCommand>,
        live_rx: rtrb::Consumer<LiveMidiEvent>,
        report_tx: rtrb::Producer<harmonium_core::EngineReport>,
        transport_position: Arc<AtomicU64>,
        clock_tx: triple_buffer::Input<TransportClock>,
    ) -> Self {
        let bpm = 120.0;
        let samples_per_step = (sample_rate * 60.0 / (bpm as f64) / 4.0) as usize;
        renderer.handle_event(AudioEvent::TimingUpdate { samples_per_step });

        Self {
            playhead: Playhead::new(sample_rate, 4),
            renderer,
            sample_rate,
            sample_counter: 0,
            samples_per_step,
            current_step_frames: samples_per_step,
            entered_bar: 1,
            entered_step: 0,
            step_open: false,
            frame_counter: 0,
            step_durations: [samples_per_step; u8::MAX as usize],
            steps_per_quarter: 4.0,
            step_count: 4,
            step_index: 0,
            swing_ratio: 0.5,
            measure_bpm: bpm,
            audible_bpm: bpm,
            tempo_override: None,
            tempo_override_explicit: false,
            clock: TransportClock::initial(sample_rate as u32),
            clock_tx,
            current_pulse: PulseSpec::default(),
            measure_rx,
            return_tx,
            window: ArrayVec::new(),
            epoch: 0,
            loaded_epoch: None,
            cmd_rx,
            live_rx,
            report_tx,
            transport_position,
            output_muted: false,
            muted_channels: vec![false; 16],
            last_muted_channels: vec![false; 16],
            loop_region: None,
            is_recording_wav: false,
            is_recording_midi: false,
            is_recording_musicxml: false,
            gain_lead: 1.0,
            gain_chords: 0.6,
            gain_bass: 0.6,
            gain_snare: 0.5,
            gain_hat: 0.4,
            musical_params: MusicalParams::default(),
            measure_sig: TimeSignature::default(),
            // 0.0 BPM never matches a real measure: the first load relays once.
            reported_bpm: 0.0,
            reported_signature: TimeSignature::default(),
            last_chord_name: ArrayString::from("I").unwrap_or_default(),
            last_chord_root_offset: 0,
            last_chord_is_minor: false,
            first_measure_loaded: false,
        }
    }

    /// Audio thread: process a buffer of audio samples.
    ///
    /// `buffer_start_instant` is the audible wall-clock instant of this
    /// buffer's first frame — computed by the cpal path from the
    /// `OutputCallbackInfo` timestamps (fallback `Instant::now()`), or just
    /// `Instant::now()` for offline rendering.
    pub fn process_buffer(
        &mut self,
        output: &mut [f32],
        channels: usize,
        buffer_start_instant: Instant,
    ) {
        // Process commands first (outside rt context)
        self.process_commands();
        // Voice any notes the player pressed on their MIDI controller.
        self.process_live_midi();

        // Publish the buffer anchor once per buffer: the wall-clock/frame
        // mapping the reader interpolates from.
        self.clock.buffer_start_frame = self.frame_counter;
        self.clock.buffer_start_instant = buffer_start_instant;
        self.publish_clock();

        // Pre-load the first measure (or the measure after a seek) before
        // rendering, so the opening pulse is frozen at the right frame and the
        // first step's notes are emitted.
        if self.playhead.needs_measure() {
            let _loaded = {
                let _rt = RtGuard::enter();
                self.drain_staged();
                self.try_load()
            };
            self.relay_measure_params();
        }

        let total_samples = output.len() / channels;
        let mut processed = 0;

        while processed < total_samples {
            // Step entry: freeze the pulse if this step opens one (the D1
            // invariant), voice the step's notes at the exact frame where the
            // step begins (clock/audio coincidence), then render the entered
            // step's frozen duration.
            if self.sample_counter == 0 {
                match self.enter_step() {
                    Some(frames) => {
                        self.current_step_frames = frames.max(1);
                        self.step_open = true;
                    }
                    // Underflow — nothing entered: free-run the stale duration
                    // and retry the load at the next entry (no publication).
                    None => self.step_open = false,
                }
            }
            let remaining = total_samples - processed;
            let samples_until_entry = self.current_step_frames - self.sample_counter;

            let chunk_size = std::cmp::min(remaining, samples_until_entry);
            let start_idx = processed * channels;
            let end_idx = (processed + chunk_size) * channels;
            let chunk = &mut output[start_idx..end_idx];

            self.renderer.process_buffer(chunk, channels);
            self.sample_counter += chunk_size;
            self.frame_counter += chunk_size as u64;
            processed += chunk_size;

            if self.sample_counter >= self.current_step_frames {
                self.sample_counter = 0;
                // Step completion: publish the entered step's position, at the
                // same frame the historical post-step tick published it.
                if self.step_open {
                    self.step_open = false;
                    self.complete_step();
                }
            }
        }

        // Zero output when muted or before the first measure is loaded
        // (suppresses DSP-graph initialization transients)
        if self.output_muted || !self.first_measure_loaded {
            for sample in output.iter_mut() {
                *sample = 0.0;
            }
        }
    }

    /// Step entry: run the per-step bookkeeping for the step the playhead
    /// currently sits on. Returns how many frames to render before the next
    /// entry, or `None` on underflow (nothing entered: the grid does not
    /// advance, the next entry retries the load).
    ///
    /// At the tick that enters step S: (a) the pulse is frozen if S opens one
    /// (the D1 invariant — `next_pulse_frame` exact by construction), (b) S's
    /// notes are voiced from the current frame on (the NoteOn frame IS the
    /// grid position: clock/audio coincidence), (c) the returned duration is
    /// S's frozen length. Position/clock publication happens at the step's
    /// completion (`complete_step`), one step duration later.
    fn enter_step(&mut self) -> Option<usize> {
        {
            let _rt = RtGuard::enter();
            self.drain_staged();
        }

        // Check loop region
        if let Some((start_bar, end_bar)) = self.loop_region {
            if self.playhead.current_bar() > end_bar {
                for ch in 0..5u8 {
                    self.renderer.handle_event(AudioEvent::AllNotesOff { channel: ch });
                }
                {
                    let _rt = RtGuard::enter();
                    self.evict_held();
                }
                self.playhead.seek_to_bar(start_bar);
                self.transport_position.store(pack(start_bar, 0), Ordering::Relaxed);
            }
        }

        // If a seek (loop wrap or command) left the playhead at a barline with
        // nothing loaded, load the measure before emitting the step's notes.
        if self.playhead.needs_measure() {
            let loaded = {
                let _rt = RtGuard::enter();
                self.try_load()
            };
            if !loaded {
                // Underflow — measure not yet staged. The grid does not
                // advance; the next entry retries the load.
                return None;
            }
            // The freshly loaded step 0 always opens a pulse: fall through to
            // the freeze below, which stamps the true boundary.
        }

        // The entered step, captured before the playhead advances: it is what
        // renders now (notes, duration) and what `clock.position` reports.
        self.entered_bar = self.playhead.current_bar();
        self.entered_step = self.playhead.position.step_in_bar(TICKS_PER_BEAT);

        // Pulse freeze for the ENTERED step (the D1 invariant): if S opens a
        // pulse, its whole-step table is frozen now — before S renders — so
        // `pulse_start_frame` is S's exact entry frame and `next_pulse_frame`
        // the pulse's exact end. (Freezing on the advanced step would stamp
        // the previous step's frame, one step early.)
        let step = self.playhead.position.step_in_bar(TICKS_PER_BEAT);
        let steps_per_pulse = usize::from(self.current_pulse.steps_per_pulse).max(1);
        if step.is_multiple_of(steps_per_pulse) {
            self.freeze_pulse(step, steps_per_pulse);
        }
        self.relay_measure_params();

        // Frozen duration of the entered step, from its pulse's fresh table.
        let duration = self.step_durations[self.step_index];

        // Tick the playhead: emits the entered step, advances, and parks the
        // finished measure when a barline is crossed.
        let events = self.playhead.tick();

        // Forward events to renderer, filtering muted channels. Done first so
        // the `events` borrow of the playhead is released before the handoff.
        for event in events {
            if let AudioEvent::NoteOn { channel, .. } = &event {
                if self.muted_channels.get(*channel as usize).copied().unwrap_or(false) {
                    continue;
                }
            }
            self.renderer.handle_event(event.clone());
        }

        // Reclaim the finished measure, then load the next one — in that
        // order, so the epoch tags stay aligned. Runs under RtGuard.
        {
            let _rt = RtGuard::enter();
            self.reclaim_finished();
            if self.playhead.needs_measure() && !self.try_load() {
                // Underflow at a barline — the entered step was emitted and
                // its duration is returned, but the grid freezes here: no
                // pulse freeze, no cursor advance. Publication (of the frozen
                // grid position) happens at the step's completion.
                self.step_index = 0;
                return Some(duration);
            }
        }

        // Cursor for the next entry: the advanced step's slot in the current
        // table (a newly opened pulse is frozen at its own entry above).
        let next = self.playhead.position.step_in_bar(TICKS_PER_BEAT);
        if next.is_multiple_of(steps_per_pulse) {
            self.step_index = 0;
        } else {
            self.step_index += 1;
        }

        // The clock position is the RENDERED step (what sounds now); the
        // packed atomic below reports the upcoming step (see `complete_step`
        // and the `TransportClock.position` contract).
        self.clock.position = TransportPosition {
            bar: self.entered_bar,
            step: self.entered_step,
            beat: self.entered_step as f32 / TICKS_PER_BEAT as f32 + 1.0,
        };

        Some(duration)
    }

    /// Métadonnées de la pulse audible : payload Copy, sans Box ni clone.
    fn relay_measure_params(&mut self) {
        if (self.audible_bpm - self.reported_bpm).abs() > 0.01
            || self.measure_sig != self.reported_signature
        {
            self.reported_bpm = self.audible_bpm;
            self.reported_signature = self.measure_sig;
            self.renderer.handle_event(AudioEvent::MeasureTiming {
                bpm: self.audible_bpm,
                time_signature: self.measure_sig,
                swing_ratio: self.swing_ratio,
            });
        }
    }

    /// Step completion: publish the entered step, exactly one step duration
    /// after its notes were voiced. This preserves the historical post-step
    /// publication timing. The packed atomic reports the upcoming step (fast
    /// scoring loops, spec 006); the clock position reports the rendered step
    /// (gameplay truth, alongside the pulse fields).
    fn complete_step(&mut self) {
        // Update shared transport position at grid resolution, after the tick.
        self.transport_position.store(
            pack(self.playhead.current_bar(), self.playhead.position.step_in_bar(TICKS_PER_BEAT)),
            Ordering::Relaxed,
        );

        // Publish the clock once per step (position + pulse fields).
        self.publish_clock();

        // Send report periodically
        let current_step = self.playhead.position.step_in_bar(4);
        if current_step.is_multiple_of(4) {
            self.send_report();
        }
    }

    fn start_tempo_ramp(&mut self, bpm: f32, ramp_pulses: u8) {
        let target = bpm.clamp(40.0, 300.0);
        let remaining = ramp_pulses.max(1);
        let step = (target - self.audible_bpm) / f32::from(remaining);
        self.tempo_override = Some(TempoRamp { target, remaining, step });
    }
    /// Freeze the pulse timing at a pulse boundary: apply one tempo-ramp step
    /// (D4), compute the pulse's whole-frame step durations (swing, D3) and
    /// fix `pulse_start_frame` / `next_pulse_frame` (the D1 invariant — exact
    /// by construction).
    fn freeze_pulse(&mut self, step: usize, steps_per_pulse: usize) {
        if let Some(ramp) = &mut self.tempo_override
            && ramp.remaining > 0
        {
            ramp.remaining -= 1;
            self.audible_bpm =
                if ramp.remaining == 0 { ramp.target } else { self.audible_bpm + ramp.step };
        }
        self.compute_pulse_steps(steps_per_pulse);
        self.step_index = 0;

        let pulse_duration = self.step_durations[..self.step_count].iter().sum::<usize>() as u64;
        self.clock.pulse_in_bar = (step / steps_per_pulse) as u8;
        self.clock.pulses_per_bar = self.current_pulse.pulses_per_bar;
        self.clock.pulse_start_frame = self.frame_counter;
        self.clock.next_pulse_frame = self.frame_counter + pulse_duration;
    }

    /// Compute the whole-frame duration of each step of a `steps_per_pulse`
    /// pulse at the current audible tempo and swing ratio (D3), frozen in
    /// `step_durations`. Compound pulses (6 steps) ignore the swing.
    fn compute_pulse_steps(&mut self, steps_per_pulse: usize) {
        let truncated_step =
            (self.sample_rate * 60.0 / f64::from(self.audible_bpm) / self.steps_per_quarter)
                .max(1.0) as usize;
        // Swing or a tempo ramp use the *rounded* quarter (no truncation
        // drift); a plain straight pulse keeps the pre-swing truncated step,
        // so the snapshot stream is unchanged.
        let rounded = self.swing_ratio > 0.5 || self.tempo_override.is_some();
        let quarter = (self.sample_rate * 60.0 / f64::from(self.audible_bpm)).round() as usize;

        let quarter_steps = self.steps_per_quarter as usize;
        if rounded
            && quarter_steps > 0
            && quarter_steps.is_multiple_of(2)
            && self.steps_per_quarter == quarter_steps as f64
            && steps_per_pulse.is_multiple_of(quarter_steps)
        {
            let half = quarter_steps / 2;
            let first =
                (f64::from(self.swing_ratio) * quarter as f64 / half as f64).round() as usize;
            let second = ((1.0 - f64::from(self.swing_ratio)) * quarter as f64 / half as f64)
                .round() as usize;
            for unit in self.step_durations[..steps_per_pulse].chunks_exact_mut(quarter_steps) {
                unit[..half].fill(first);
                unit[half..].fill(second);
                let sum = half * (first + second);
                unit[quarter_steps - 1] = (unit[quarter_steps - 1] as isize + quarter as isize
                    - sum as isize)
                    .max(1) as usize;
            }
        } else {
            // Straight without swing/ramp, or compound (swing ignored):
            // uniform truncated steps.
            for slot in &mut self.step_durations[..steps_per_pulse] {
                *slot = truncated_step;
            }
        }
        self.step_count = steps_per_pulse;

        // Base step duration (the recorder's TimingUpdate) stays the nominal
        // truncated step regardless of swing.
        if truncated_step != self.samples_per_step {
            self.samples_per_step = truncated_step;
            self.renderer
                .handle_event(AudioEvent::TimingUpdate { samples_per_step: truncated_step });
        }
    }

    /// Publish the current clock into the triple buffer.
    ///
    /// Runs under [`RtGuard`]: the producer `write` is a memcpy + atomic
    /// swap (no allocation, no lock), and the guard makes a debug build
    /// panic if that ever regresses.
    fn publish_clock(&mut self) {
        let _rt = RtGuard::enter();
        self.clock_tx.write(self.clock);
    }

    /// Drain the staged ring into the window. Must run under [`RtGuard`]:
    /// pointer moves only. Stops draining while the window is full (the
    /// composer sees ring backpressure and retries later).
    fn drain_staged(&mut self) {
        while self.window.len() < MEASURE_WINDOW_CAP {
            let item = match self.measure_rx.pop() {
                Ok(item) => item,
                Err(_) => break,
            };
            // Adopt newer epochs on sight: boxes staged before an
            // invalidation carry an older epoch and are stale.
            if item.epoch > self.epoch {
                self.epoch = item.epoch;
            }
            if item.epoch < self.epoch {
                self.stage_return(item.epoch, item.measure);
                continue;
            }
            let index = item.measure.index;
            // Duplicate of the currently-playing box: keep playing it,
            // hand the copy back.
            if !self.playhead.needs_measure() && self.playhead.current_bar() == index {
                self.stage_return(item.epoch, item.measure);
                continue;
            }
            if let Some(pos) = self.window.iter().position(|s| s.measure.index == index) {
                // Same bar staged twice (republish race): keep the newer epoch.
                if item.epoch > self.window[pos].epoch {
                    let old = std::mem::replace(&mut self.window[pos].measure, item.measure);
                    let old_epoch = std::mem::replace(&mut self.window[pos].epoch, item.epoch);
                    self.stage_return(old_epoch, old);
                } else {
                    self.stage_return(item.epoch, item.measure);
                }
                continue;
            }
            let pos = self
                .window
                .iter()
                .position(|s| s.measure.index > index)
                .unwrap_or(self.window.len());
            self.window.insert(pos, WindowSlot { epoch: item.epoch, measure: item.measure });
        }
    }

    /// Reclaim the box the playhead left at the last barline. Must run under
    /// [`RtGuard`]. A box inside the active loop region goes back into the
    /// window for the next pass; any other box goes to the return ring.
    fn reclaim_finished(&mut self) {
        if let Some(measure) = self.playhead.take_finished() {
            let epoch = self.loaded_epoch.take().unwrap_or(self.epoch);
            self.route_reclaimed(epoch, measure);
        }
    }

    /// Evict every box the playhead holds (current + finished). Must run
    /// under [`RtGuard`]. Used before a jump (seek, loop wrap) so no stale
    /// box survives the position change.
    fn evict_held(&mut self) {
        if let Some(measure) = self.playhead.unload_current() {
            let epoch = self.loaded_epoch.take().unwrap_or(self.epoch);
            self.route_reclaimed(epoch, measure);
        }
        self.reclaim_finished();
    }

    /// Route a reclaimed box: keep loop-region boxes of the current epoch in
    /// the window, hand everything else back. Must run under [`RtGuard`].
    fn route_reclaimed(&mut self, epoch: u64, measure: Box<Measure>) {
        if epoch >= self.epoch && self.in_loop(measure.index) {
            // Make room by evicting one non-loop prefetch (the composer
            // republishes it on demand — it lies ahead of the playhead).
            if self.window.len() >= MEASURE_WINDOW_CAP
                && let Some(pos) = self.window.iter().rposition(|s| !self.in_loop(s.measure.index))
            {
                let slot = self.window.remove(pos);
                self.stage_return(slot.epoch, slot.measure);
            }
            if self.window.len() < MEASURE_WINDOW_CAP {
                let index = measure.index;
                let pos = self
                    .window
                    .iter()
                    .position(|s| s.measure.index > index)
                    .unwrap_or(self.window.len());
                self.window.insert(pos, WindowSlot { epoch, measure });
                return;
            }
            // Window full of loop boxes: the loop is longer than the window
            // (refused upstream, defensive) — hand back, republish later.
        }
        self.stage_return(epoch, measure);
    }

    /// Load the window box matching the playhead bar. Stale boxes found on
    /// the way are handed back. Must run under [`RtGuard`]. Returns false on
    /// underflow: the caller stays silent, as with a missing page before.
    fn try_load(&mut self) -> bool {
        let bar = self.playhead.current_bar();
        let mut found: Option<usize> = None;
        let mut i = 0;
        while i < self.window.len() {
            if self.window[i].epoch < self.epoch {
                let slot = self.window.remove(i);
                self.stage_return(slot.epoch, slot.measure);
                continue;
            }
            if found.is_none() && self.window[i].measure.index == bar {
                found = Some(i);
            }
            i += 1;
        }
        if let Some(pos) = found {
            let slot = self.window.remove(pos);
            debug_assert!(slot.epoch == self.epoch);

            // Capture the measure's nominal tempo, swing ratio, signature and
            // dance-pulse spec before the box moves into the playhead (Copy
            // fields). Sounding tempo/signature reach the renderer through
            // `relay_measure_params` at the next unguarded point: this
            // function runs under `RtGuard`, where allocating (Box, clone)
            // would panic in debug builds.
            self.measure_bpm = slot.measure.tempo;
            self.swing_ratio = slot.measure.swing_ratio;
            self.current_pulse = slot.measure.pulse;
            if !self.tempo_override_explicit {
                if self.first_measure_loaded
                    && slot.measure.position.is_some()
                    && self.measure_bpm != self.audible_bpm
                {
                    // Même rampe que SetTempo, sur une mesure de pulses :
                    // aucune coupure, aucun seek et aucune horloge reconstruite.
                    self.start_tempo_ramp(self.measure_bpm, self.current_pulse.pulses_per_bar);
                } else {
                    self.tempo_override = None;
                    self.audible_bpm = self.measure_bpm;
                }
            }
            self.measure_sig = slot.measure.time_signature;
            self.steps_per_quarter = slot.measure.steps_per_quarter();

            // Update chord info for reports
            self.last_chord_name =
                ArrayString::from(&slot.measure.chord_context.chord_name).unwrap_or_default();
            self.last_chord_root_offset = slot.measure.chord_context.root_offset;
            self.last_chord_is_minor = slot.measure.chord_context.is_minor;

            self.playhead.load_measure(slot.measure);
            self.loaded_epoch = Some(slot.epoch);

            // The opening pulse is NOT frozen here: `enter_step` freezes the
            // entered step's pulse before it renders — including this freshly
            // loaded step 0, which always opens a pulse — so the pulse fields
            // are stamped at the true boundary, never at the load frame (a
            // post-crossing load precedes its barline).
            self.first_measure_loaded = true;
            true
        } else {
            false
        }
    }

    /// Hand a box back to the composer. Must run under [`RtGuard`]. The
    /// return ring covers every box the engine can hold at once, so the push
    /// cannot fail; on the impossible error path the `PushError` still owns
    /// the box and drops it, which panics loudly in debug builds.
    fn stage_return(&mut self, epoch: u64, measure: Box<Measure>) {
        let _ = self.return_tx.push(TaggedMeasure { epoch, measure });
    }

    /// Whether `bar` lies in the active loop region.
    fn in_loop(&self, bar: usize) -> bool {
        self.loop_region.is_some_and(|(start, end)| bar >= start && bar <= end)
    }
    /// Prepare a jump to `target_bar`. Must run under [`RtGuard`], before
    /// [`Playhead::seek_to_bar`]. Re-seeking the loaded bar restarts it
    /// (the box stays, cursors reset in `seek_to_bar`); any other jump
    /// unloads the held box, and prefetch behind the target — outside the
    /// loop region — is handed back. Prefetch at or past the target stays:
    /// it is still valid (exact-index load guarantees no stale play).
    fn evict_for_seek(&mut self, target_bar: usize) {
        if self.playhead.current_bar() == target_bar && !self.playhead.needs_measure() {
            // Same bar, box loaded: keep it, restart from step 0.
            // A parked finished box (crossed mid-buffer) is already played.
            self.reclaim_finished();
        } else if let Some(measure) = self.playhead.unload_current() {
            let epoch = self.loaded_epoch.take().unwrap_or(self.epoch);
            self.stage_return(epoch, measure);
            self.reclaim_finished();
        } else {
            self.reclaim_finished();
        }
        self.evict_behind_at(target_bar);
    }

    /// Hand back every window box behind the playhead bar that is outside
    /// the loop region. Must run under [`RtGuard`].
    fn evict_behind(&mut self) {
        let bar = self.playhead.current_bar();
        self.evict_behind_at(bar);
    }

    /// Hand back every window box with `index < bar` that is outside the
    /// loop region. Must run under [`RtGuard`].
    fn evict_behind_at(&mut self, bar: usize) {
        let mut i = 0;
        while i < self.window.len() {
            let index = self.window[i].measure.index;
            if index < bar && !self.in_loop(index) {
                let slot = self.window.remove(i);
                self.stage_return(slot.epoch, slot.measure);
            } else {
                i += 1;
            }
        }
    }

    /// Drain the live-MIDI ring and voice the player's notes through the
    /// synth on the dedicated monitor channel. Note-off respects an empty
    /// ring (producer may not exist when MIDI input is off) — it simply
    /// no-ops, so this is safe to call every buffer.
    fn process_live_midi(&mut self) {
        while let Ok(ev) = self.live_rx.pop() {
            let event = if ev.on {
                AudioEvent::NoteOn {
                    note: ev.note,
                    velocity: ev.velocity,
                    channel: MONITOR_CHANNEL,
                }
            } else {
                AudioEvent::NoteOff { note: ev.note, channel: MONITOR_CHANNEL }
            };
            self.renderer.handle_event(event);
        }
    }

    fn process_commands(&mut self) {
        while let Ok(cmd) = self.cmd_rx.pop() {
            match cmd {
                PlaybackCommand::SetChannelGain { channel, gain } => {
                    let gain = gain.clamp(0.0, 1.0);
                    match channel {
                        0 => self.gain_lead = gain,
                        1 => self.gain_chords = gain,
                        2 => self.gain_bass = gain,
                        3 => self.gain_snare = gain,
                        4 => self.gain_hat = gain,
                        _ => {}
                    }
                    self.renderer.handle_event(AudioEvent::SetMixerGains {
                        lead: self.gain_lead,
                        chords: self.gain_chords,
                        bass: self.gain_bass,
                        snare: self.gain_snare,
                        hat: self.gain_hat,
                    });
                }
                PlaybackCommand::SetChannelMute { channel, muted } => {
                    if (channel as usize) < self.muted_channels.len() {
                        self.muted_channels[channel as usize] = muted;
                        if muted {
                            self.renderer.handle_event(AudioEvent::AllNotesOff { channel });
                        }
                    }
                }
                PlaybackCommand::SetChannelRoute { channel, bank_id } => {
                    if (channel as usize) < 16 {
                        self.renderer
                            .handle_event(AudioEvent::SetChannelRoute { channel, bank: bank_id });
                    }
                }
                PlaybackCommand::SetVelocityBase { .. } => {
                    // Velocity base is handled in generation, not playback
                }
                PlaybackCommand::SetOutputMute(muted) => {
                    self.output_muted = muted;
                    if muted {
                        for ch in 0..5u8 {
                            self.renderer.handle_event(AudioEvent::AllNotesOff { channel: ch });
                        }
                    }
                }
                PlaybackCommand::SetMasterVolume(_volume) => {
                    // Master volume is applied during rendering
                }
                PlaybackCommand::Seek(bar) => {
                    let target_bar = bar.max(1);
                    log::info(&format!("Seeking to bar {target_bar}"));
                    for ch in 0..5u8 {
                        self.renderer.handle_event(AudioEvent::AllNotesOff { channel: ch });
                    }
                    {
                        let _rt = RtGuard::enter();
                        self.evict_for_seek(target_bar);
                    }
                    self.playhead.seek_to_bar(target_bar);
                    self.transport_position.store(pack(target_bar, 0), Ordering::Relaxed);
                    // A jump cuts the in-flight step: drop its remainder and its
                    // pending completion publication, so the next entry starts
                    // the target bar from step 0.
                    self.sample_counter = 0;
                    self.step_open = false;
                }
                PlaybackCommand::SeekPlayhead(bar) => {
                    let target_bar = bar.max(1);
                    log::info(&format!("SeekPlayhead to bar {target_bar}"));
                    for ch in 0..5u8 {
                        self.renderer.handle_event(AudioEvent::AllNotesOff { channel: ch });
                    }
                    {
                        let _rt = RtGuard::enter();
                        self.evict_for_seek(target_bar);
                    }
                    self.playhead.seek_to_bar(target_bar);
                    self.transport_position.store(pack(target_bar, 0), Ordering::Relaxed);
                    // Same cut as `Seek` above: fresh entry at the target bar.
                    self.sample_counter = 0;
                    self.step_open = false;
                }
                PlaybackCommand::DropFrom { bar } => {
                    let bar = bar.max(1);
                    // The finished box was already played: always hand it back.
                    // Window boxes at or past the cut are stale by definition.
                    let _rt = RtGuard::enter();
                    if let Some(measure) = self.playhead.take_finished() {
                        let epoch = self.loaded_epoch.take().unwrap_or(self.epoch);
                        self.stage_return(epoch, measure);
                    }
                    let mut i = 0;
                    while i < self.window.len() {
                        if self.window[i].measure.index >= bar {
                            let slot = self.window.remove(i);
                            self.stage_return(slot.epoch, slot.measure);
                        } else {
                            i += 1;
                        }
                    }
                }
                PlaybackCommand::SetLoop { start_bar, end_bar } => {
                    let start = start_bar.max(1);
                    let end = end_bar.max(start);
                    log::info(&format!("Loop set: bars {start}-{end}"));
                    self.loop_region = Some((start, end));
                    // Loops longer than the window are refused on the main
                    // thread; shed prefetch behind the playhead that the new
                    // region no longer needs.
                    let _rt = RtGuard::enter();
                    self.evict_behind();
                }
                PlaybackCommand::ClearLoop => {
                    log::info("Loop cleared");
                    self.loop_region = None;
                    // Former loop boxes behind the playhead will never play
                    // again: hand them back now instead of occupying the window.
                    let _rt = RtGuard::enter();
                    self.reclaim_finished();
                    self.evict_behind();
                }
                PlaybackCommand::StartRecording(format) => {
                    match format {
                        harmonium_core::events::RecordFormat::Wav => self.is_recording_wav = true,
                        harmonium_core::events::RecordFormat::Midi => self.is_recording_midi = true,
                        harmonium_core::events::RecordFormat::MusicXml => {
                            self.is_recording_musicxml = true;
                            self.renderer.handle_event(AudioEvent::UpdateMusicalParams {
                                params: Box::new(self.musical_params.clone()),
                            });
                        }
                    }
                    self.renderer.handle_event(AudioEvent::StartRecording { format });
                }
                PlaybackCommand::StopRecording(format) => {
                    match format {
                        harmonium_core::events::RecordFormat::Wav => self.is_recording_wav = false,
                        harmonium_core::events::RecordFormat::Midi => {
                            self.is_recording_midi = false
                        }
                        harmonium_core::events::RecordFormat::MusicXml => {
                            self.is_recording_musicxml = false
                        }
                    }
                    self.renderer.handle_event(AudioEvent::StopRecording { format });
                }
                PlaybackCommand::MixerGains { lead, chords, bass, snare, hat } => {
                    self.gain_lead = lead;
                    self.gain_chords = chords;
                    self.gain_bass = bass;
                    self.gain_snare = snare;
                    self.gain_hat = hat;
                    self.renderer.handle_event(AudioEvent::SetMixerGains {
                        lead,
                        chords,
                        bass,
                        snare,
                        hat,
                    });
                }
                PlaybackCommand::LoadFont { id, bytes } => {
                    self.renderer.handle_event(AudioEvent::LoadFont { id, bytes });
                }
                PlaybackCommand::ProgramChange { channel, program } => {
                    self.renderer.handle_event(AudioEvent::ProgramChange { channel, program });
                }
                PlaybackCommand::SetMutedChannels(channels) => {
                    for (i, &muted) in channels.iter().enumerate() {
                        if i < self.muted_channels.len() {
                            if muted && !self.muted_channels[i] {
                                self.renderer
                                    .handle_event(AudioEvent::AllNotesOff { channel: i as u8 });
                            }
                            self.muted_channels[i] = muted;
                        }
                    }
                }
                PlaybackCommand::UpdateMusicalParams(params) => {
                    // Forwarded so recorders (MIDI tempo/signature metas,
                    // MusicXML) track the commanded params; the measure load
                    // path refines them to the sounding values on change.
                    self.renderer
                        .handle_event(AudioEvent::UpdateMusicalParams { params: params.clone() });
                    self.reported_bpm = params.bpm;
                    self.reported_signature = params.time_signature;
                    self.musical_params = *params;
                }
                PlaybackCommand::SetTempo { bpm, ramp_pulses } => {
                    // D4: the audible tempo advances at pulse boundaries only
                    // (the D1 invariant); `ramp_pulses == 0` is a jump applied
                    // at the next boundary.
                    self.tempo_override_explicit = true;
                    self.start_tempo_ramp(bpm, ramp_pulses);
                }
                PlaybackCommand::ClearTempo => {
                    self.tempo_override_explicit = false;
                    self.tempo_override = None;
                    self.audible_bpm = self.measure_bpm;
                }
            }
        }
    }

    fn send_report(&mut self) {
        use harmonium_core::EngineReport;

        let mut report = EngineReport::new();

        report.current_bar = self.playhead.current_bar();
        report.current_beat = self.playhead.position.beat;
        report.current_step = self.playhead.position.step_in_bar(4);
        report.time_signature = self.musical_params.time_signature;

        report.current_chord = self.last_chord_name;
        report.chord_root_offset = self.last_chord_root_offset;
        report.chord_is_minor = self.last_chord_is_minor;
        report.harmony_mode = self.musical_params.harmony_mode;

        report.rhythm_mode = self.musical_params.rhythm_mode;
        report.primary_steps = self.musical_params.rhythm_steps;
        report.primary_pulses = self.musical_params.rhythm_pulses;
        report.secondary_steps = self.musical_params.rhythm_secondary_steps;
        report.secondary_pulses = self.musical_params.rhythm_secondary_pulses;

        report.musical_params = self.musical_params.clone();
        // The audible tempo (override/ramp) is the truth for the game, not the
        // measure's nominal tempo.
        report.musical_params.bpm = self.audible_bpm;

        let _ = self.report_tx.push(report);
    }
}

#[cfg(test)]
mod transport_position_tests {
    use super::*;

    #[test]
    fn pack_unpack_roundtrip() {
        for (bar, step) in [(1, 0), (5, 15), (12, 0), (42, 3), (1000, 15)] {
            assert_eq!(unpack(pack(bar, step)), (bar, step));
        }
    }

    #[test]
    fn pack_unpack_at_bounds() {
        // Step is masked to 32 bits; bar keeps its own bits.
        assert_eq!(unpack(pack(7, u32::MAX as usize)), (7, u32::MAX as usize));
        assert_eq!(unpack(pack(0, 0)), (0, 0));
    }

    #[test]
    fn initial_value_is_bar1_step0() {
        assert_eq!(pack(1, 0), 1u64 << 32);
        assert_eq!(unpack(pack(1, 0)), (1, 0));
    }

    #[test]
    fn beat_is_derived_from_step() {
        let at = |bar: usize, step: usize| TransportPosition::from_packed(pack(bar, step));
        assert_eq!(at(1, 0).beat, 1.0);
        assert_eq!(at(1, 2).beat, 1.5);
        assert_eq!(at(1, 6).beat, 2.5);
        assert_eq!(at(3, 12).beat, 4.0);
        assert_eq!(at(3, 15).beat, 4.75);
    }

    #[test]
    fn step_never_leaks_into_bar_bits() {
        assert_eq!(TransportPosition::from_packed(pack(7, 12)).bar, 7);
        assert_eq!(unpack(pack(7, u32::MAX as usize)).0, 7);
    }
}

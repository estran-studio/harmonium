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
    events::{AudioEvent, MIDI_TICKS_PER_STEP},
    params::{PlaybackParams, PulseSpec, TimeSignature},
    time_plan::{BarTiming, Flicks, Tempo, ramp_tempo},
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

impl TransportPosition {
    /// Decode a packed transport value into a [`TransportPosition`],
    /// deriving `beat` from the step.
    #[must_use]
    pub fn from_packed(packed: u64) -> Self {
        let (bar, step) = unpack(packed);
        Self { bar, step, beat: step as f32 / TICKS_PER_BEAT as f32 + 1.0 }
    }
}

/// Commands sent from the main thread to the PlaybackEngine (audio thread).
// Variantes volumineuses gardées en ligne : un `Box` serait libéré sur le thread audio
// (allocation interdite dans le callback, principe III).
#[allow(clippy::large_enum_variant)]
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
    StartRecording(harmonium_core::events::AudioRecordFormat),
    StopRecording(harmonium_core::events::AudioRecordFormat),
    ProgramChange {
        channel: u8,
        program: u8,
    },
    /// Update the muted channels mask from the composer's musical params
    /// (one flag per logical channel; fixed size, nothing to free on the audio thread).
    SetMutedChannels([bool; 16]),
    /// Update musical params snapshot for recording/reporting. `Copy`,
    /// prepared on the main thread with `MusicalParams::playback_params`.
    UpdateMusicalParams(PlaybackParams),
    /// Émotions `[arousal, valence, density, tension]` pour le morphing Odin
    /// (`AudioEvent::EmotionUpdate`) ; `Copy`, sans allocation.
    SetEmotions([f32; 4]),
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

#[derive(Clone, Copy, Default)]
struct TrackState {
    mix: harmonium_core::mix::MidiTrackMix,
    muted: bool,
}

/// PlaybackEngine runs on the audio thread inside the CPAL closure.
///
/// It reads staged measures from the composer → playback ring into an
/// ordered prefetch window, ticks the playhead, and renders audio.
/// Generation is NOT done here — that's MusicComposer's job.
///
/// Real-time contract: the whole of [`PlaybackEngine::process_buffer`] runs
/// under the debug `RTCheckAllocator` guard — commands, profile updates,
/// reports, rendering and recording start/stop included. The handoff (ring
/// pop/push, window insert/remove, box moves) performs no allocation, no
/// deallocation and takes no lock. Played boxes travel back through the
/// return ring so only the main thread drops them; commands and reports carry
/// `Copy` payloads. Documented exceptions (SoundFont loading, oxisynth's
/// per-note `Vec`) live in `harmonium_audio` under `allow_allocation`. A box is only ever loaded when its index equals the playhead bar;
/// otherwise the engine underflows (silence), exactly like a missing page
/// before.
pub struct PlaybackEngine {
    // Playhead
    playhead: Playhead,

    // Audio renderer
    renderer: Box<dyn AudioRenderer>,

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
    event_index: usize,
    // The step being rendered (bar 1-based, step 0-based in bar), captured at
    // its entry: `clock.position` reports it, the packed atomic the upcoming
    // step. Updated before the playhead advances, published at completion.
    entered_bar: usize,
    entered_step: usize,
    // Global frame counter: total frames rendered since engine start. The
    // clock's pulse/buffer fields live in this frame space.
    frame_counter: u64,
    // Integer sample rate for plan ↔ frame conversions.
    sample_rate_hz: u32,
    // Per-step timing (D3 swing + D4 ramp), derived from the shared time plan
    // (`harmonium_core::time_plan`): `step_durations[..step_count]` is the
    // frozen frame length of each step of the current pulse (the D1
    // invariant), `step_index` the step currently being rendered.
    step_durations: [usize; u8::MAX as usize],
    // MIDI ticks per frame last announced to the renderer (`MidiClock`).
    midi_ticks_per_frame: f64,
    step_count: usize,
    step_index: usize,
    // Swing ratio of the measure under the playhead (relayed to the renderer).
    swing_ratio: f32,
    // Timing header of the measure under the playhead, captured at load.
    bar_timing: BarTiming,
    // Header and tempo frozen with the current pulse, and the plan instant
    // (flicks since the anchor step) where the pulse starts.
    pulse_timing: BarTiming,
    pulse_tempo: Tempo,
    pulse_plan_start: u64,
    // Plan instant of the next step entry along the played path.
    next_step_plan: u64,
    // Frame of plan instant zero (the first entered step). The playhead's
    // gap with the plan is measured from it, never corrected.
    plan_anchor_frame: Option<u64>,
    // Nominal tempo of the measure under the playhead (Measure.tempo) and the
    // audible tempo actually used for timing (overridden by a tempo ramp).
    measure_tempo: Tempo,
    audible_tempo: Tempo,
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

    // Report ring buffer (audio thread → main thread)
    report_tx: rtrb::Producer<harmonium_core::EngineReport>,

    // Shared transport position, packed (bar, step) — audio thread writes,
    // composer and lock-free readers read
    transport_position: Arc<AtomicU64>,

    // One runtime state per logical track: REPL gain and mute edit these slots.
    tracks: [TrackState; 16],
    output_muted: bool,
    mix_tune: Option<usize>,
    pending_mix_reset: bool,

    // Loop region
    loop_region: Option<(usize, usize)>,

    // Musical params snapshot for reports (Copy: replacing it frees nothing)
    musical_params: PlaybackParams,
    // Signature of the measure under the playhead, captured at load.
    measure_sig: TimeSignature,
    // Tempo and signature last relayed to the renderer (MIDI file metas).
    // Compared at each measure load; the relay fires only on change, so
    // steady-state playback stays allocation-free.
    reported_tempo: Option<Tempo>,
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

/// An in-flight tempo ramp (D4): at each pulse boundary the audible tempo
/// takes the next value of [`ramp_tempo`] from `from` to `target`, landing
/// exactly on `target` after `total` pulses (the shared plan's rule).
#[derive(Clone, Copy)]
struct TempoRamp {
    from: Tempo,
    target: Tempo,
    total: u32,
    done: u32,
}

impl PlaybackEngine {
    /// Create a new PlaybackEngine.
    ///
    /// Eight arguments is the whole wiring (renderer, rings, transport,
    /// clock producer): bundling them would only rename the tuple.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sample_rate: f64,
        mut renderer: Box<dyn AudioRenderer>,
        measure_rx: rtrb::Consumer<TaggedMeasure>,
        return_tx: rtrb::Producer<TaggedMeasure>,
        cmd_rx: rtrb::Consumer<PlaybackCommand>,
        report_tx: rtrb::Producer<harmonium_core::EngineReport>,
        transport_position: Arc<AtomicU64>,
        clock_tx: triple_buffer::Input<TransportClock>,
    ) -> Self {
        let tempo = Tempo::from_milli_bpm(120_000);
        let sample_rate_hz = sample_rate.round() as u32;
        let samples_per_step = (tempo.quarter().to_frames(sample_rate_hz) / 4) as usize;
        renderer.handle_event(AudioEvent::TimingUpdate { samples_per_step });
        let midi_ticks_per_frame = MIDI_TICKS_PER_STEP as f64 / samples_per_step as f64;
        renderer.handle_event(AudioEvent::MidiClock { ticks_per_frame: midi_ticks_per_frame });
        let bar_timing = BarTiming {
            tempo,
            swing: harmonium_core::time_plan::Swing::STRAIGHT,
            numerator: 4,
            denominator: 4,
            steps: 16,
            steps_per_pulse: 4,
            pulses_per_bar: 4,
            ramp_in: false,
        };
        let musical_params = PlaybackParams {
            mix: harmonium_core::StyleProfile::builtin().mix.prepare(sample_rate),
            ..PlaybackParams::default()
        };
        renderer.handle_event(AudioEvent::SetMix { mix: musical_params.mix });
        let mut tracks = [TrackState::default(); 16];
        for (track, mix) in tracks.iter_mut().zip(musical_params.mix.tracks) {
            track.mix = mix;
        }

        Self {
            playhead: Playhead::new(sample_rate, 4),
            renderer,
            sample_rate_hz,
            sample_counter: 0,
            samples_per_step,
            event_index: 0,
            current_step_frames: samples_per_step,
            entered_bar: 1,
            entered_step: 0,
            step_open: false,
            frame_counter: 0,
            step_durations: [samples_per_step; u8::MAX as usize],
            midi_ticks_per_frame,
            step_count: 4,
            step_index: 0,
            swing_ratio: 0.5,
            bar_timing,
            pulse_timing: bar_timing,
            pulse_tempo: tempo,
            pulse_plan_start: 0,
            next_step_plan: 0,
            plan_anchor_frame: None,
            measure_tempo: tempo,
            audible_tempo: tempo,
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
            report_tx,
            transport_position,
            output_muted: false,
            tracks,
            mix_tune: None,
            pending_mix_reset: false,
            loop_region: None,
            musical_params,
            measure_sig: TimeSignature::default(),
            // Nothing relayed yet: the first load relays once.
            reported_tempo: None,
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
    ///
    /// The whole call runs under the real-time guard (see the type docs).
    pub fn process_buffer(
        &mut self,
        output: &mut [f32],
        channels: usize,
        buffer_start_instant: Instant,
    ) {
        let _rt = RtGuard::enter();
        // Process commands first
        self.process_commands();

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
            // Split at every scheduled attack/release, not only grid entries.
            while let Some(event) = self.playhead.event(self.event_index) {
                if self.event_frame(event.offset) > self.sample_counter {
                    break;
                }
                if !matches!(&event.event, AudioEvent::NoteOn { channel, .. }
                    if self.tracks.get(*channel as usize).is_some_and(|track| track.muted))
                {
                    self.renderer.handle_event(event.event.clone());
                }
                self.event_index += 1;
            }
            let until_event = self
                .playhead
                .event(self.event_index)
                .map_or(self.current_step_frames, |event| self.event_frame(event.offset));
            let samples_until_entry = self.current_step_frames - self.sample_counter;
            let chunk_size = remaining
                .min(samples_until_entry)
                .min(until_event.saturating_sub(self.sample_counter).max(1));
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

    /// Frame, dans le pas gelé courant, d'un instant en millièmes de pas ;
    /// arrondi à l'échantillon, jamais au-delà de la dernière frame du pas.
    fn event_frame(&self, offset: u16) -> usize {
        ((usize::from(offset) * self.current_step_frames + 500) / 1000)
            .min(self.current_step_frames.saturating_sub(1))
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
                for track in harmonium_core::timeline::TrackId::ALL {
                    self.renderer
                        .handle_event(AudioEvent::AllNotesOff { channel: track.channel() });
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

        // The clock position is the RENDERED step (what sounds now); the
        // packed atomic reports the upcoming step (see `complete_step` and the
        // `TransportClock.position` contract). Set before the playhead
        // advances, so an underflow at the barline still reports this step.
        self.clock.position = TransportPosition {
            bar: self.entered_bar,
            step: self.entered_step,
            beat: self.entered_step as f32 / TICKS_PER_BEAT as f32 + 1.0,
        };
        self.enter_plan_step();

        // Frozen duration of the entered step, from its pulse's fresh table.
        let duration = self.step_durations[self.step_index];

        // The next box is already prepared by the composer. Its early attacks
        // belong to this bar's final step; ownership remains in the window.
        let next_bar = self
            .loop_region
            .filter(|(_, end)| *end == self.entered_bar)
            .map_or(self.entered_bar + 1, |(start, _)| start);
        let next = self
            .window
            .iter()
            .find(|slot| slot.measure.index == next_bar)
            .map(|slot| slot.measure.as_ref());
        {
            let _rt = RtGuard::enter();
            self.playhead.tick(next);
        }
        self.event_index = 0;

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

        Some(duration)
    }

    /// Métadonnées de la pulse audible : payload Copy, sans Box ni clone.
    fn relay_measure_params(&mut self) {
        if self.pending_mix_reset {
            self.pending_mix_reset = false;
            self.reset_profile_mix();
        }
        if self.reported_tempo != Some(self.audible_tempo)
            || self.measure_sig != self.reported_signature
        {
            self.reported_tempo = Some(self.audible_tempo);
            self.reported_signature = self.measure_sig;
            self.renderer.handle_event(AudioEvent::MeasureTiming {
                bpm: self.audible_tempo.bpm(),
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

    /// Start a tempo ramp from the audible tempo, landing on `target` after
    /// `pulses` pulse boundaries (`0` jumps at the next boundary).
    fn start_tempo_ramp(&mut self, target: Tempo, pulses: u8) {
        self.tempo_override = Some(TempoRamp {
            from: self.audible_tempo,
            target,
            total: u32::from(pulses.max(1)),
            done: 0,
        });
    }

    /// Freeze the pulse timing at a pulse boundary: apply one tempo-ramp step
    /// (D4), derive the pulse's whole-frame step durations from the shared
    /// time plan (swing D3 included) and fix `pulse_start_frame` /
    /// `next_pulse_frame` (the D1 invariant — exact by construction).
    fn freeze_pulse(&mut self, step: usize, steps_per_pulse: usize) {
        if let Some(ramp) = &mut self.tempo_override
            && ramp.done < ramp.total
        {
            ramp.done += 1;
            self.audible_tempo = ramp_tempo(ramp.from, ramp.target, ramp.done, ramp.total);
        }
        self.compute_pulse_steps(steps_per_pulse);
        self.step_index = 0;

        let pulse_duration = self.step_durations[..self.step_count].iter().sum::<usize>() as u64;
        self.clock.pulse_in_bar = (step / steps_per_pulse) as u8;
        self.clock.pulses_per_bar = self.current_pulse.pulses_per_bar;
        self.clock.pulse_start_frame = self.frame_counter;
        self.clock.next_pulse_frame = self.frame_counter + pulse_duration;
    }

    /// Whole-frame duration of each step of the pulse opening at the next
    /// plan instant: frames between consecutive plan instants
    /// ([`BarTiming::step_offset`] at the audible tempo), rounded on the
    /// cumulative instant so durations never drift from the plan.
    fn compute_pulse_steps(&mut self, steps_per_pulse: usize) {
        self.pulse_timing = self.bar_timing;
        self.pulse_tempo = self.audible_tempo;
        self.pulse_plan_start = self.next_step_plan;
        let (base, rate, timing, tempo) =
            (self.pulse_plan_start, self.sample_rate_hz, self.pulse_timing, self.pulse_tempo);
        let frame_of = |offset: u64| Flicks(base + offset).to_frames(rate);
        let start = frame_of(0);
        let mut previous = start;
        for (k, slot) in self.step_durations[..steps_per_pulse].iter_mut().enumerate() {
            let end = frame_of(timing.step_offset(tempo, k as u32 + 1));
            *slot = (end - previous) as usize;
            previous = end;
        }
        self.step_count = steps_per_pulse;

        // Nominal step (the MIDI backend's TimingUpdate) and recorder cadence:
        // the pulse spans exactly `steps × 120` ticks over its frames.
        let pulse_frames = (previous - start).max(1);
        let samples_per_step = (pulse_frames / steps_per_pulse as u64).max(1) as usize;
        if samples_per_step != self.samples_per_step {
            self.samples_per_step = samples_per_step;
            self.renderer.handle_event(AudioEvent::TimingUpdate { samples_per_step });
        }
        let ticks_per_frame =
            f64::from(MIDI_TICKS_PER_STEP) * steps_per_pulse as f64 / pulse_frames as f64;
        if ticks_per_frame != self.midi_ticks_per_frame {
            self.midi_ticks_per_frame = ticks_per_frame;
            self.renderer.handle_event(AudioEvent::MidiClock { ticks_per_frame });
        }
    }

    /// Plan bookkeeping of the entered step: its plan instant along the played
    /// path, and the playhead's measured gap with the plan (frames rendered
    /// minus the plan's frame for that instant). The gap is published, never
    /// corrected here: underflows and seeks show up in it.
    fn enter_plan_step(&mut self) {
        let k = self.step_index as u32;
        let instant = self.pulse_plan_start + self.pulse_timing.step_offset(self.pulse_tempo, k);
        self.next_step_plan =
            self.pulse_plan_start + self.pulse_timing.step_offset(self.pulse_tempo, k + 1);
        let planned = Flicks(instant).to_frames(self.sample_rate_hz);
        let anchor =
            *self.plan_anchor_frame.get_or_insert(self.frame_counter.saturating_sub(planned));
        self.clock.step_start_frame = self.frame_counter;
        self.clock.plan_instant = Flicks(instant);
        self.clock.plan_gap_frames = self.frame_counter as i64 - (anchor + planned) as i64;
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
            self.bar_timing = BarTiming::of(&slot.measure);
            self.measure_tempo = self.bar_timing.tempo;
            if let Some(position) = slot.measure.position
                && self.mix_tune != Some(position.tune_index)
            {
                self.mix_tune = Some(position.tune_index);
                self.pending_mix_reset = true;
            }
            self.swing_ratio = slot.measure.swing_ratio;
            self.current_pulse = slot.measure.pulse;
            // Same rule as the shared plan (`time_plan::TimePlan`): a tempo
            // change on a tune measure ramps over the measure's pulses from
            // the audible tempo; anything else jumps.
            if !self.tempo_override_explicit {
                if self.first_measure_loaded
                    && self.bar_timing.ramp_in
                    && self.measure_tempo != self.audible_tempo
                {
                    // Même rampe que SetTempo, sur une mesure de pulses :
                    // aucune coupure, aucun seek et aucune horloge reconstruite.
                    self.start_tempo_ramp(self.measure_tempo, self.current_pulse.pulses_per_bar);
                } else {
                    self.tempo_override = None;
                    self.audible_tempo = self.measure_tempo;
                }
            }
            self.measure_sig = slot.measure.time_signature;

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

    fn emit_mix(&mut self) {
        let mut mix = self.musical_params.mix;
        for (control, track) in mix.tracks.iter_mut().zip(&self.tracks) {
            *control = track.mix;
        }
        self.renderer.handle_event(AudioEvent::SetMix { mix });
    }

    fn reset_profile_mix(&mut self) {
        for (track, mix) in self.tracks.iter_mut().zip(self.musical_params.mix.tracks) {
            track.mix = mix;
        }
        self.emit_mix();
    }

    fn process_commands(&mut self) {
        while let Ok(cmd) = self.cmd_rx.pop() {
            match cmd {
                PlaybackCommand::SetChannelGain { channel, gain } => {
                    if let Some(track) = self.tracks.get_mut(channel as usize) {
                        // gain is a linear amplitude, not a linear CC7 value.
                        track.mix.volume = (127.0 * gain.clamp(0.0, 1.0).sqrt()).round() as u8;
                        self.emit_mix();
                    }
                }
                PlaybackCommand::SetChannelMute { channel, muted } => {
                    if let Some(track) = self.tracks.get_mut(channel as usize) {
                        track.muted = muted;
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
                PlaybackCommand::SetOutputMute(muted) => {
                    self.output_muted = muted;
                    if muted {
                        for track in harmonium_core::timeline::TrackId::ALL {
                            self.renderer
                                .handle_event(AudioEvent::AllNotesOff { channel: track.channel() });
                        }
                    }
                }
                PlaybackCommand::SetMasterVolume(_volume) => {
                    // Master volume is applied during rendering
                }
                PlaybackCommand::Seek(bar) => {
                    let target_bar = bar.max(1);
                    for track in harmonium_core::timeline::TrackId::ALL {
                        self.renderer
                            .handle_event(AudioEvent::AllNotesOff { channel: track.channel() });
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
                    for track in harmonium_core::timeline::TrackId::ALL {
                        self.renderer
                            .handle_event(AudioEvent::AllNotesOff { channel: track.channel() });
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
                    self.loop_region = Some((start, end));
                    // Loops longer than the window are refused on the main
                    // thread; shed prefetch behind the playhead that the new
                    // region no longer needs.
                    let _rt = RtGuard::enter();
                    self.evict_behind();
                }
                PlaybackCommand::ClearLoop => {
                    self.loop_region = None;
                    // Former loop boxes behind the playhead will never play
                    // again: hand them back now instead of occupying the window.
                    let _rt = RtGuard::enter();
                    self.reclaim_finished();
                    self.evict_behind();
                }
                PlaybackCommand::StartRecording(format) => {
                    self.renderer.handle_event(AudioEvent::StartRecording { format });
                }
                PlaybackCommand::StopRecording(format) => {
                    self.renderer.handle_event(AudioEvent::StopRecording { format });
                }
                PlaybackCommand::ProgramChange { channel, program } => {
                    self.renderer.handle_event(AudioEvent::ProgramChange { channel, program });
                }
                PlaybackCommand::SetMutedChannels(channels) => {
                    for (track, &muted) in self.tracks.iter_mut().zip(channels.iter()) {
                        track.muted = muted;
                    }
                    for (channel, track) in self.tracks.iter().enumerate() {
                        if track.muted {
                            self.renderer
                                .handle_event(AudioEvent::AllNotesOff { channel: channel as u8 });
                        }
                    }
                }
                PlaybackCommand::UpdateMusicalParams(params) => {
                    // Forwarded so recorders (MIDI tempo/signature metas,
                    // MusicXML) track the commanded params; the measure load
                    // path refines them to the sounding values on change.
                    let mix_changed = self.musical_params.profile_id != params.profile_id
                        || self.musical_params.mix != params.mix;
                    self.renderer.handle_event(AudioEvent::UpdateMusicalParams { params });
                    self.reported_tempo = Some(Tempo::from_bpm(params.bpm));
                    self.reported_signature = params.time_signature;
                    self.musical_params = params;
                    if mix_changed {
                        self.reset_profile_mix();
                    }
                }
                PlaybackCommand::SetEmotions(emotions) => {
                    self.renderer.handle_event(AudioEvent::EmotionUpdate { emotions });
                }
                PlaybackCommand::SetTempo { bpm, ramp_pulses } => {
                    // D4: the audible tempo advances at pulse boundaries only
                    // (the D1 invariant); `ramp_pulses == 0` is a jump applied
                    // at the next boundary.
                    self.tempo_override_explicit = true;
                    self.start_tempo_ramp(Tempo::from_bpm(bpm), ramp_pulses);
                }
                PlaybackCommand::ClearTempo => {
                    self.tempo_override_explicit = false;
                    self.tempo_override = None;
                    self.audible_tempo = self.measure_tempo;
                }
            }
        }
    }

    fn send_report(&mut self) {
        use harmonium_core::EngineReport;

        // Fixed-size snapshot: no Vec, no Box, no clone of heap data.
        let mut report = EngineReport::realtime(self.musical_params);

        report.current_bar = self.playhead.current_bar();
        report.current_beat = self.playhead.position.beat;
        report.current_step = self.playhead.position.step_in_bar(4);
        report.time_signature = self.musical_params.time_signature;

        report.current_chord = self.last_chord_name;
        report.chord_root_offset = self.last_chord_root_offset;
        report.chord_is_minor = self.last_chord_is_minor;

        // The audible tempo (override/ramp) is the truth for the game, not the
        // measure's nominal tempo.
        report.musical_params.bpm = self.audible_tempo.bpm();

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

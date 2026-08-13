//! PlaybackEngine - Audio-thread playback, decoupled from music generation.
//!
//! Minimal audio-thread component: reads pre-generated measures from a ring buffer,
//! ticks the playhead, renders audio events. No generation, no allocations in steady state.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use arrayvec::ArrayString;
use harmonium_audio::backend::AudioRenderer;
use harmonium_core::{events::AudioEvent, log, params::MusicalParams, timeline::Playhead};

/// Grid resolution of the transport, in steps per beat (sixteenth notes).
/// `TransportPosition::beat` is derived from the step on this grid — it is
/// never stored separately.
pub const TICKS_PER_BEAT: usize = 4;

/// Transport position at the engine's grid resolution, packed into a single
/// lock-free shared value ([`pack`]). `bar` is 1-based, `step` is 0-based
/// within the measure, and `beat` is derived: `step / TICKS_PER_BEAT + 1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
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

/// MIDI channel reserved for monitoring the player's own input ("hear
/// yourself"). Channels 0–3 are the sequenced tracks (lead/bass/kick/hat)
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
    SetLoop {
        start_bar: usize,
        end_bar: usize,
    },
    ClearLoop,
    StartRecording(harmonium_core::events::RecordFormat),
    StopRecording(harmonium_core::events::RecordFormat),
    MixerGains {
        lead: f32,
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
}

/// PlaybackEngine runs on the audio thread inside the CPAL closure.
///
/// It reads measures from the ring buffer, ticks the playhead, and renders audio.
/// Generation is NOT done here — that's MusicComposer's job.
pub struct PlaybackEngine {
    // Playhead
    playhead: Playhead,

    // Audio renderer
    renderer: Box<dyn AudioRenderer>,
    sample_rate: f64,

    // Timing
    sample_counter: usize,
    samples_per_step: usize,

    // Shared pages (Composer writes by index, Playback reads by index)
    shared_pages: crate::SharedPages,

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
    gain_bass: f32,
    gain_snare: f32,
    gain_hat: f32,

    // Musical params snapshot for reports
    musical_params: MusicalParams,

    // Report cache
    last_chord_name: ArrayString<64>,
    last_chord_root_offset: i32,
    last_chord_is_minor: bool,

    // True once the playhead has loaded its first measure.
    // Before that, output is zeroed to suppress DSP-graph init transients.
    first_measure_loaded: bool,
}

impl PlaybackEngine {
    /// Create a new PlaybackEngine.
    pub fn new(
        sample_rate: f64,
        mut renderer: Box<dyn AudioRenderer>,
        shared_pages: crate::SharedPages,
        cmd_rx: rtrb::Consumer<PlaybackCommand>,
        live_rx: rtrb::Consumer<LiveMidiEvent>,
        report_tx: rtrb::Producer<harmonium_core::EngineReport>,
        transport_position: Arc<AtomicU64>,
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
            shared_pages,
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
            gain_bass: 0.6,
            gain_snare: 0.5,
            gain_hat: 0.4,
            musical_params: MusicalParams::default(),
            last_chord_name: ArrayString::from("I").unwrap_or_default(),
            last_chord_root_offset: 0,
            last_chord_is_minor: false,
            first_measure_loaded: false,
        }
    }

    /// Audio thread: process a buffer of audio samples.
    pub fn process_buffer(&mut self, output: &mut [f32], channels: usize) {
        // Process commands first (outside rt context)
        self.process_commands();
        // Voice any notes the player pressed on their MIDI controller.
        self.process_live_midi();

        let total_samples = output.len() / channels;
        let mut processed = 0;

        while processed < total_samples {
            let remaining = total_samples - processed;
            let samples_until_tick = if self.samples_per_step > self.sample_counter {
                self.samples_per_step - self.sample_counter
            } else {
                1
            };

            let chunk_size = std::cmp::min(remaining, samples_until_tick);
            let start_idx = processed * channels;
            let end_idx = (processed + chunk_size) * channels;
            let chunk = &mut output[start_idx..end_idx];

            self.renderer.process_buffer(chunk, channels);
            self.sample_counter += chunk_size;
            processed += chunk_size;

            if self.sample_counter >= self.samples_per_step {
                self.sample_counter = 0;
                self.tick();
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

    /// Tick: read from playhead and emit events.
    fn tick(&mut self) {
        // Check loop region
        if let Some((start_bar, end_bar)) = self.loop_region {
            if self.playhead.current_bar() > end_bar {
                for ch in 0..4u8 {
                    self.renderer.handle_event(AudioEvent::AllNotesOff { channel: ch });
                }
                self.playhead.seek_to_bar(start_bar);
                self.transport_position.store(pack(start_bar, 0), Ordering::Relaxed);
            }
        }

        // If playhead needs a new measure, read from shared pages by index
        if self.playhead.needs_measure() {
            let bar = self.playhead.current_bar();
            let measure_opt = if let Ok(pages) = self.shared_pages.lock() {
                pages.iter().find(|m| m.index == bar).cloned()
            } else {
                None
            };

            if let Some(measure) = measure_opt {
                // Update timing from measure tempo
                let steps_per_beat = 4.0f64;
                let new_sps =
                    (self.sample_rate * 60.0 / (measure.tempo as f64) / steps_per_beat) as usize;
                if new_sps != self.samples_per_step {
                    self.samples_per_step = new_sps;
                    self.renderer
                        .handle_event(AudioEvent::TimingUpdate { samples_per_step: new_sps });
                }

                // Update chord info for reports
                self.last_chord_name =
                    ArrayString::from(&measure.chord_context.chord_name).unwrap_or_default();
                self.last_chord_root_offset = measure.chord_context.root_offset;
                self.last_chord_is_minor = measure.chord_context.is_minor;

                self.playhead.load_measure(measure);
                self.first_measure_loaded = true;
            } else {
                return; // Underflow — measure not yet generated
            }
        }

        // Tick the playhead
        let events = self.playhead.tick();

        // Forward events to renderer, filtering muted channels
        for event in events {
            if let AudioEvent::NoteOn { channel, .. } = &event {
                if self.muted_channels.get(*channel as usize).copied().unwrap_or(false) {
                    continue;
                }
            }
            self.renderer.handle_event(event.clone());
        }

        // Update shared transport position at grid resolution, after the
        // tick: (current bar, step within the measure)
        self.transport_position.store(
            pack(self.playhead.current_bar(), self.playhead.position.step_in_bar(TICKS_PER_BEAT)),
            Ordering::Relaxed,
        );

        // Send report periodically
        let current_step = self.playhead.position.step_in_bar(4);
        if current_step.is_multiple_of(4) {
            self.send_report();
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
                        0 => self.gain_bass = gain,
                        1 => self.gain_lead = gain,
                        2 => self.gain_snare = gain,
                        3 => self.gain_hat = gain,
                        _ => {}
                    }
                    self.renderer.handle_event(AudioEvent::SetMixerGains {
                        lead: self.gain_lead,
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
                        for ch in 0..4u8 {
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
                    for ch in 0..4u8 {
                        self.renderer.handle_event(AudioEvent::AllNotesOff { channel: ch });
                    }
                    self.playhead.seek_to_bar(target_bar);
                    self.transport_position.store(pack(target_bar, 0), Ordering::Relaxed);
                }
                PlaybackCommand::SeekPlayhead(bar) => {
                    let target_bar = bar.max(1);
                    log::info(&format!("SeekPlayhead to bar {target_bar}"));
                    for ch in 0..4u8 {
                        self.renderer.handle_event(AudioEvent::AllNotesOff { channel: ch });
                    }
                    self.playhead.seek_to_bar(target_bar);
                    self.transport_position.store(pack(target_bar, 0), Ordering::Relaxed);
                }
                PlaybackCommand::SetLoop { start_bar, end_bar } => {
                    let start = start_bar.max(1);
                    let end = end_bar.max(start);
                    log::info(&format!("Loop set: bars {start}-{end}"));
                    self.loop_region = Some((start, end));
                }
                PlaybackCommand::ClearLoop => {
                    log::info("Loop cleared");
                    self.loop_region = None;
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
                PlaybackCommand::MixerGains { lead, bass, snare, hat } => {
                    self.gain_lead = lead;
                    self.gain_bass = bass;
                    self.gain_snare = snare;
                    self.gain_hat = hat;
                    self.renderer.handle_event(AudioEvent::SetMixerGains {
                        lead,
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
                    self.musical_params = *params;
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

use std::io::Cursor;

use harmonium_core::events::AudioEvent;
use rand::Rng;

/// Routing of a logical channel to a SoundFont bank.
///
/// Every channel is rendered by oxisynth; `bank` selects the SoundFont bank
/// (CC 0) used for the channel. A negative bank id in `channel_routing` /
/// `SetChannelRoute` means bank 0.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChannelRoute {
    pub bank: u32,
}

impl ChannelRoute {
    /// Build a route from a signed bank id: `-1` (or any negative) ⇒ bank 0.
    #[must_use]
    pub const fn from_bank_id(bank_id: i32) -> Self {
        Self { bank: if bank_id < 0 { 0 } else { bank_id as u32 } }
    }
}

pub struct VoiceManager {
    pub synth: oxisynth::Synth,
    pub channel_routing: [ChannelRoute; 16],
    /// Bank currently selected on each logical channel; `None` until the first
    /// `set_channel_route` so the initial Bank Select / Program Change is sent.
    pub current_banks: [Option<u32>; 16],
    /// Maps logical channels to MIDI channels (e.g. drums ch2/ch3 → MIDI ch9)
    pub midi_channel_map: [u8; 16],
}

/// Default identity map: logical channel N → MIDI channel N
fn default_midi_channel_map() -> [u8; 16] {
    let mut map = [0u8; 16];
    for (i, slot) in map.iter_mut().enumerate() {
        *slot = i as u8;
    }
    map
}

impl VoiceManager {
    #[must_use]
    pub fn new(sf2_bytes: Option<&[u8]>, sample_rate: f32) -> Self {
        let mut synth = oxisynth::Synth::default();
        synth.set_sample_rate(sample_rate);

        let mut vm = Self {
            synth,
            channel_routing: [ChannelRoute { bank: 0 }; 16],
            current_banks: [None; 16],
            midi_channel_map: default_midi_channel_map(),
        };

        if let Some(bytes) = sf2_bytes {
            vm.add_font(0, bytes);
        }

        vm
    }

    pub fn add_font(&mut self, bank_id: u32, bytes: &[u8]) {
        harmonium_core::log::info(&format!(
            "Loading SoundFont: {} bytes, bank_id={}",
            bytes.len(),
            bank_id
        ));

        // Catch panics from OxiSynth decompression (corrupt SF2/SF3 data)
        let font_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut cursor = Cursor::new(bytes);
            oxisynth::SoundFont::load(&mut cursor)
        }));

        match font_result {
            Ok(Ok(font)) => {
                let font_id = self.synth.add_font(font, true);
                self.synth.set_bank_offset(font_id, bank_id);
                harmonium_core::log::info("SoundFont loaded successfully");
            }
            Ok(Err(e)) => {
                harmonium_core::log::error(&format!("SoundFont loading failed: {e}"));
            }
            Err(panic_info) => {
                let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                    (*s).to_string()
                } else {
                    "unknown panic".to_string()
                };
                harmonium_core::log::error(&format!(
                    "SoundFont loading panicked (corrupt data?): {msg}"
                ));
            }
        }
    }

    /// Configure GM-standard program assignments and drum channel mapping.
    ///
    /// When `gm_compatible` is true (full GM SoundFont like MuseScore General):
    /// - Channel 0 → Melody / Lead (GM program 0, Piano)
    /// - Channel 1 → Chords (GM program 24, Acoustic Guitar steel)
    /// - Channel 2 → Bass (GM program 32, Acoustic Bass)
    /// - Channels 3,4 → Drums / Feet on MIDI channel 9 (GM bank 128, program 0)
    ///
    /// When `gm_compatible` is false (small/test SoundFonts):
    /// - All channels use program 0 (first available patch)
    /// - No drum remapping (drums play as pitched notes)
    pub fn configure_gm_defaults(&mut self, gm_compatible: bool) {
        if gm_compatible {
            // Remap drum logical channels (heel=3, toe=4) to MIDI channel 9 (GM drums)
            self.midi_channel_map[3] = 9;
            self.midi_channel_map[4] = 9;

            // Set up MIDI channel 9 as drum bank (GM standard)
            let _ = self.synth.send_event(oxisynth::MidiEvent::ControlChange {
                channel: 9,
                ctrl: 0,
                value: 128,
            });
            let _ = self
                .synth
                .send_event(oxisynth::MidiEvent::ProgramChange { channel: 9, program_id: 0 });

            // Melody / Lead: GM program 0 (Acoustic Grand Piano)
            self.set_channel_program(0, 0);
            // Chords: GM program 24 (Acoustic Guitar steel)
            self.set_channel_program(1, 24);
            // Bass: GM program 32 (Acoustic Bass)
            self.set_channel_program(2, 32);
        } else {
            // Non-GM: all channels use program 0
            for ch in 0..5 {
                self.set_channel_program(ch, 0);
            }
        }
    }

    /// Send a GM Program Change to the MIDI channel mapped from a logical channel.
    pub fn set_channel_program(&mut self, logical_channel: usize, program: u8) {
        if logical_channel < 16 {
            let midi_ch = self.midi_channel_map[logical_channel];
            let _ = self.synth.send_event(oxisynth::MidiEvent::ProgramChange {
                channel: midi_ch,
                program_id: program,
            });
        }
    }

    /// Route a logical channel to a SoundFont bank. Sends Bank Select (CC 0)
    /// and Program Change 0 only when the bank actually changes.
    pub fn set_channel_route(&mut self, channel: usize, route: ChannelRoute) {
        if channel < 16 {
            self.channel_routing[channel] = route;

            if self.current_banks[channel] != Some(route.bank) {
                let midi_ch = self.midi_channel_map[channel];
                let _ = self.synth.send_event(oxisynth::MidiEvent::ControlChange {
                    channel: midi_ch,
                    ctrl: 0,
                    value: route.bank as u8,
                });
                let _ = self.synth.send_event(oxisynth::MidiEvent::ProgramChange {
                    channel: midi_ch,
                    program_id: 0,
                });
                self.current_banks[channel] = Some(route.bank);
            }
        }
    }

    /// Mixer gains for the five instrument channels (melody=0, chords=1,
    /// bass=2, heel=3, toe=4), applied as MIDI Channel Volume (CC 7). In GM
    /// mode heel and toe share MIDI channel 9, so the toe gain is applied last
    /// and wins.
    pub fn set_gains(&mut self, lead: f32, chords: f32, bass: f32, snare: f32, hat: f32) {
        for (channel, gain) in [(0usize, lead), (1, chords), (2, bass), (3, snare), (4, hat)] {
            let midi_ch = self.midi_channel_map[channel];
            let _ = self.synth.send_event(oxisynth::MidiEvent::ControlChange {
                channel: midi_ch,
                ctrl: 7,
                value: (gain.clamp(0.0, 1.0) * 127.0) as u8,
            });
        }
    }

    pub fn process_event(&mut self, event: AudioEvent) {
        match event {
            AudioEvent::NoteOn { note, velocity, channel } => {
                let midi_ch = self.midi_channel_map[channel as usize];
                let _ = self.synth.send_event(oxisynth::MidiEvent::NoteOn {
                    channel: midi_ch,
                    key: note,
                    vel: velocity,
                });
            }
            AudioEvent::NoteOff { note, channel } => {
                let midi_ch = self.midi_channel_map[channel as usize];
                let _ = self
                    .synth
                    .send_event(oxisynth::MidiEvent::NoteOff { channel: midi_ch, key: note });
            }
            AudioEvent::AllNotesOff { channel } => {
                let midi_ch = self.midi_channel_map[channel as usize];
                // Sustain Off (CC 64) + All Notes Off (CC 123) pour vraiment couper
                let _ = self.synth.send_event(oxisynth::MidiEvent::ControlChange {
                    channel: midi_ch,
                    ctrl: 64,
                    value: 0,
                });
                let _ = self.synth.send_event(oxisynth::MidiEvent::ControlChange {
                    channel: midi_ch,
                    ctrl: 123,
                    value: 0,
                });
            }
            AudioEvent::ControlChange { ctrl, value, channel } => {
                let midi_ch = self.midi_channel_map[channel as usize];
                let _ = self.synth.send_event(oxisynth::MidiEvent::ControlChange {
                    channel: midi_ch,
                    ctrl,
                    value,
                });

                if ctrl == 1 {
                    // Modulation / Tension: filter, resonance and pitch
                    // instability on the five instrument channels.
                    let val_norm = f32::from(value) / 127.0;
                    let cutoff_val = ((30.0 + val_norm * 97.0) as i32).clamp(0, 127) as u8;
                    let res_val = ((val_norm * 90.0) as i32).clamp(0, 127) as u8;
                    for ch in 0..5 {
                        let mapped_ch = self.midi_channel_map[ch];
                        let _ = self.synth.send_event(oxisynth::MidiEvent::ControlChange {
                            channel: mapped_ch,
                            ctrl: 74,
                            value: cutoff_val,
                        });
                        let _ = self.synth.send_event(oxisynth::MidiEvent::ControlChange {
                            channel: mapped_ch,
                            ctrl: 71,
                            value: res_val,
                        });

                        // Horror: Pitch Instability (Detune)
                        let bend_val = if val_norm > 0.6 {
                            let mut rng = rand::thread_rng();
                            let wobble_amount = (val_norm - 0.6) * 2000.0;
                            let random_bend = rng.gen_range(-0.5..0.5) * wobble_amount;
                            (8192.0 + random_bend) as i32
                        } else {
                            8192
                        };
                        let _ = self.synth.send_event(oxisynth::MidiEvent::PitchBend {
                            channel: mapped_ch,
                            value: bend_val.clamp(0, 16383) as u16,
                        });
                    }
                }
            }
            AudioEvent::ProgramChange { channel, program } => {
                self.set_channel_program(channel as usize, program);
            }
            AudioEvent::TimingUpdate { .. }
            | AudioEvent::UpdateMusicalParams { .. }
            | AudioEvent::LoadFont { .. }
            | AudioEvent::SetChannelRoute { .. }
            | AudioEvent::StartRecording { .. }
            | AudioEvent::StopRecording { .. }
            | AudioEvent::SetMixerGains { .. } => {}
        }
    }

    /// Render one interleaved stereo buffer. Panics inside oxisynth are caught
    /// so a corrupt font or bad state cannot take down the audio thread.
    pub fn render(&mut self, output: &mut [f32]) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.synth.write(&mut *output);
        }));
        if result.is_err() {
            output.fill(0.0);
        }
    }
}

use harmonium_core::events::AudioEvent;

use crate::{
    backend::AudioRenderer,
    voice_manager::{ChannelRoute, FEET_BANK_OFFSET, VoiceManager},
};

/// SoundFont renderer backed by oxisynth.
pub struct SynthBackend {
    voice_manager: VoiceManager,
    mix: harmonium_core::mix::PreparedMix,
    eq_state: [[[f32; 2]; 2]; 2],
}

impl SynthBackend {
    /// Build the backend. `sf2_bytes = None` starts without any bank loaded:
    /// the engine renders silence until a `LoadFont` event arrives.
    /// `feet_sf2_bytes` charge la banque de pieds dédiée (décalage
    /// [`FEET_BANK_OFFSET`]) et route Heel/Toe dessus ; `None` conserve le
    /// comportement GM (canal 9) ou non-GM.
    /// `initial_routing` holds signed bank ids per logical channel
    /// (negative ⇒ bank 0); entries beyond channel 15 are ignored.
    #[must_use]
    pub fn new(
        sample_rate: f64,
        sf2_bytes: Option<&[u8]>,
        feet_sf2_bytes: Option<&[u8]>,
        initial_routing: &[i32],
    ) -> Self {
        let mut voice_manager = VoiceManager::new(sf2_bytes, sample_rate as f32);

        for (i, &bank_id) in initial_routing.iter().enumerate().take(16) {
            voice_manager.set_channel_route(i, ChannelRoute::from_bank_id(bank_id));
        }

        // After routing is established, configure GM defaults if a SoundFont is loaded.
        // This must happen after set_channel_route() because that sends ProgramChange(0)
        // which would overwrite the correct GM programs.
        // Détection réelle de la conformité GM : présence d'un preset en banque 128 (kit de batterie).
        // Calculé à l'initialisation, strictement hors du callback temps réel.
        let gm_compatible = if sf2_bytes.is_some() {
            let gm = voice_manager.has_bank_128_preset();
            if !gm {
                let missing = voice_manager.missing_gm_programs();
                let missing_list =
                    if missing.is_empty() { "aucun".to_string() } else { missing.join(", ") };
                let warning_msg = format!(
                    "Avertissement : la banque SoundFont n'est pas compatible General MIDI \
                     (aucun kit de batterie en banque 128). \
                     Programmes GM demandés absents : {missing_list}. \
                     Le canal 9 ne sera pas remappé vers la banque 128."
                );
                eprintln!("{warning_msg}");
                harmonium_core::log::warn(&warning_msg);
            }
            gm
        } else {
            false
        };

        // Banque de pieds dédiée (SK2) : chargée une seule fois, hors callback audio.
        if let Some(bytes) = feet_sf2_bytes {
            voice_manager.add_font(FEET_BANK_OFFSET, bytes);
        }

        voice_manager.configure_gm_defaults(gm_compatible, feet_sf2_bytes.is_some());

        let mix = harmonium_core::StyleProfile::builtin().mix.prepare(sample_rate);
        let mut backend = Self { voice_manager, mix, eq_state: [[[0.0; 2]; 2]; 2] };
        backend.apply_mix(mix);
        backend
    }

    pub fn add_font(&mut self, id: u32, bytes: &[u8]) {
        self.voice_manager.add_font(id, bytes);
    }

    pub fn set_channel_route(&mut self, channel: usize, route: ChannelRoute) {
        self.voice_manager.set_channel_route(channel, route);
    }

    fn apply_mix(&mut self, mix: harmonium_core::mix::PreparedMix) {
        if self.mix.eq != mix.eq {
            self.eq_state = [[[0.0; 2]; 2]; 2];
        }
        self.mix = mix;
        self.voice_manager.synth.set_reverb_params(&oxisynth::ReverbParams {
            roomsize: mix.room.room_size,
            damp: mix.room.damping,
            width: mix.room.width,
            level: mix.room.level,
        });
        // En GM les pieds partagent le canal 9 : Toe est envoyé en dernier et gagne.
        for (channel, track) in mix.tracks.into_iter().enumerate() {
            for (ctrl, value) in track.controllers() {
                self.voice_manager.process_event(AudioEvent::ControlChange {
                    channel: channel as u8,
                    ctrl,
                    value,
                });
            }
        }
    }
}

impl AudioRenderer for SynthBackend {
    fn handle_event(&mut self, event: AudioEvent) {
        match event {
            AudioEvent::LoadFont { id, bytes } => {
                self.voice_manager.add_font(id, &bytes);
            }
            AudioEvent::SetChannelRoute { channel, bank } => {
                self.voice_manager
                    .set_channel_route(channel as usize, ChannelRoute::from_bank_id(bank));
            }
            AudioEvent::SetMix { mix } => self.apply_mix(mix),
            AudioEvent::UpdateMusicalParams { params } => {
                self.voice_manager.apply_profile_programs(&params.profile_instruments);
            }
            AudioEvent::TimingUpdate { .. } | AudioEvent::MeasureTiming { .. } => {}
            _ => self.voice_manager.process_event(event),
        }
    }

    /// Allocation-free: oxisynth writes interleaved stereo straight into
    /// `output`. Non-stereo layouts are not rendered and yield silence.
    fn process_buffer(&mut self, output: &mut [f32], channels: usize) {
        if channels == 2 {
            self.voice_manager.render(output);
            let (frames, _) = output.as_chunks_mut::<2>();
            for frame in frames {
                for (channel, sample) in frame.iter_mut().enumerate() {
                    let mut x = *sample;
                    for (band, [b0, b1, b2, a1, a2]) in self.mix.eq.into_iter().enumerate() {
                        let state = &mut self.eq_state[band][channel];
                        let y = b0 * x + state[0];
                        state[0] = b1 * x - a1 * y + state[1];
                        state[1] = b2 * x - a2 * y;
                        x = y;
                    }
                    *sample = x * self.mix.master_gain;
                }
            }
        } else {
            output.fill(0.0);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

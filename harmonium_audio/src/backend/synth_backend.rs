use harmonium_core::events::AudioEvent;

use crate::{
    backend::AudioRenderer,
    voice_manager::{ChannelRoute, FEET_BANK_OFFSET, VoiceManager},
};

/// SoundFont renderer backed by oxisynth.
pub struct SynthBackend {
    voice_manager: VoiceManager,
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

        Self { voice_manager }
    }

    pub fn add_font(&mut self, id: u32, bytes: &[u8]) {
        self.voice_manager.add_font(id, bytes);
    }

    pub fn set_channel_route(&mut self, channel: usize, route: ChannelRoute) {
        self.voice_manager.set_channel_route(channel, route);
    }

    pub fn set_gains(&mut self, lead: f32, chords: f32, bass: f32, snare: f32, hat: f32) {
        self.voice_manager.set_gains(lead, chords, bass, snare, hat);
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
            AudioEvent::SetMixerGains { lead, chords, bass, snare, hat } => {
                self.voice_manager.set_gains(lead, chords, bass, snare, hat);
            }
            AudioEvent::TimingUpdate { .. }
            | AudioEvent::UpdateMusicalParams { .. }
            | AudioEvent::MeasureTiming { .. } => {}
            _ => self.voice_manager.process_event(event),
        }
    }

    /// Allocation-free: oxisynth writes interleaved stereo straight into
    /// `output`. Non-stereo layouts are not rendered and yield silence.
    fn process_buffer(&mut self, output: &mut [f32], channels: usize) {
        if channels == 2 {
            self.voice_manager.render(output);
        } else {
            output.fill(0.0);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

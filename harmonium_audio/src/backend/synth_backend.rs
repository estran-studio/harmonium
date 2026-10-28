use harmonium_core::events::AudioEvent;

use crate::{
    backend::AudioRenderer,
    voice_manager::{ChannelRoute, FEET_BANK_OFFSET, VoiceManager},
};

/// SoundFont renderer backed by oxisynth, avec voix Odin par piste derrière
/// l'option `odin`. Sans `odin` (ou sans preset fourni), tout passe par oxisynth.
pub struct SynthBackend {
    voice_manager: VoiceManager,
    mix: harmonium_core::mix::PreparedMix,
    eq_state: [[[f32; 2]; 2]; 2],
    #[cfg(feature = "odin")]
    odin: crate::odin_voices::OdinMixer,
}

impl SynthBackend {
    /// Build the backend. `sf2_bytes = None` starts without any bank loaded:
    /// the engine renders silence until a `LoadFont` event arrives.
    /// `feet_sf2_bytes` charge la banque de pieds en banque
    /// [`FEET_BANK_OFFSET`] ; le profil décide qui l'emploie
    /// (`apply_profile_programs`, un seul chemin).
    /// `initial_routing` holds signed bank ids per logical channel
    /// (negative ⇒ bank 0); entries beyond channel 15 are ignored.
    /// `odin_presets` : presets `.odin` fournis par l'hôte (nom + octets),
    /// analysés à la construction hors callback quand compilé avec `odin` ;
    /// ignorés sinon (chaque piste joue son programme SoundFont).
    #[must_use]
    pub fn new(
        sample_rate: f64,
        sf2_bytes: Option<&[u8]>,
        feet_sf2_bytes: Option<&[u8]>,
        initial_routing: &[i32],
        odin_presets: &[crate::odin_source::OdinPresetSource],
    ) -> Self {
        let mut voice_manager = VoiceManager::new(sf2_bytes, sample_rate as f32);

        for (i, &bank_id) in initial_routing.iter().enumerate().take(16) {
            voice_manager.set_channel_route(i, ChannelRoute::from_bank_id(bank_id));
        }

        // Diagnostic de conformité GM (kit banque 128, programmes demandés),
        // strictement hors du callback temps réel. Le routage, lui, suit le
        // profil (`apply_profile_programs`, un seul chemin).
        if sf2_bytes.is_some() && !voice_manager.has_bank_128_preset() {
            let missing = voice_manager.missing_gm_programs();
            let missing_list =
                if missing.is_empty() { "aucun".to_string() } else { missing.join(", ") };
            let warning_msg = format!(
                "Avertissement : la banque SoundFont n'est pas compatible General MIDI \
                 (aucun kit de batterie en banque 128). \
                 Programmes GM demandés absents : {missing_list}."
            );
            eprintln!("{warning_msg}");
            harmonium_core::log::warn(&warning_msg);
        }

        // Préparation SF2 strictement à l'initialisation : les banques natives
        // évitent le saturating_sub des offsets oxisynth, qui masquerait GM 0.
        if let Some(bytes) = feet_sf2_bytes {
            match relocate_font_banks(bytes, FEET_BANK_OFFSET as u16) {
                Ok(prepared) => voice_manager.add_font(0, &prepared),
                Err(error) => harmonium_core::log::error(error),
            }
        }

        voice_manager
            .apply_profile_programs(&harmonium_core::StyleProfile::builtin().instruments);
        #[cfg(feature = "odin")]
        let odin = crate::odin_voices::OdinMixer::new(sample_rate as f32, odin_presets);
        #[cfg(not(feature = "odin"))]
        let _ = odin_presets;

        let mix = harmonium_core::StyleProfile::builtin().mix.prepare(sample_rate);
        #[cfg(feature = "odin")]
        let mut backend = Self { voice_manager, mix, eq_state: [[[0.0; 2]; 2]; 2], odin };
        #[cfg(not(feature = "odin"))]
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
        #[cfg(feature = "odin")]
        self.odin.apply_mix(mix);
    }
    /// Bascule SoundFont ↔ Odin d'après le profil : applique puis coupe les
    /// notes des canaux qui changent de côté (aucune note pendante).
    /// Sans `odin`, aucun canal ne bascule.
    #[cfg(feature = "odin")]
    fn update_routing(&mut self, params: &harmonium_core::params::PlaybackParams) {
        let mut was = [false; harmonium_core::timeline::TrackId::ALL.len()];
        for (index, slot) in was.iter_mut().enumerate() {
            *slot = self.odin.is_odin_channel(index);
        }
        self.odin.apply_descs(&params.odin);
        for (index, was_odin) in was.iter().enumerate() {
            if *was_odin != self.odin.is_odin_channel(index) {
                let channel = index as u8;
                self.voice_manager.process_event(AudioEvent::AllNotesOff { channel });
                self.odin.all_notes_off(index);
            }
        }
    }

    #[cfg(not(feature = "odin"))]
    fn update_routing(&mut self, _params: &harmonium_core::params::PlaybackParams) {}
}

impl AudioRenderer for SynthBackend {
    fn handle_event(&mut self, event: AudioEvent) {
        match event {
            AudioEvent::LoadFont { id, bytes } => {
                // Dette temps réel assumée : analyser et insérer une SoundFont
                // alloue (oxisynth) ; c'est une opération rare, hors régime
                // permanent. Le tampon est libéré dans la même portée.
                let _allowed = crate::realtime::rt_check::allow_allocation();
                self.voice_manager.add_font(id, &bytes);
                drop(bytes);
            }
            AudioEvent::SetChannelRoute { channel, bank } => {
                self.voice_manager
                    .set_channel_route(channel as usize, ChannelRoute::from_bank_id(bank));
            }
            AudioEvent::SetMix { mix } => self.apply_mix(mix),
            AudioEvent::UpdateMusicalParams { params } => {
                self.update_routing(&params);
                self.voice_manager.apply_profile_programs(&params.instruments);
            }
            #[cfg(feature = "odin")]
            AudioEvent::EmotionUpdate { emotions } => self.odin.apply_emotions(emotions),
            #[cfg(not(feature = "odin"))]
            AudioEvent::EmotionUpdate { .. } => {}
            AudioEvent::NoteOn { note, velocity, channel } => {
                #[cfg(feature = "odin")]
                if self.odin.is_odin_channel(usize::from(channel)) {
                    self.odin.note_on(usize::from(channel), note, velocity);
                    return;
                }
                self.voice_manager.process_event(AudioEvent::NoteOn { note, velocity, channel });
            }
            AudioEvent::NoteOff { note, channel } => {
                #[cfg(feature = "odin")]
                if self.odin.is_odin_channel(usize::from(channel)) {
                    self.odin.note_off(usize::from(channel), note);
                    return;
                }
                self.voice_manager.process_event(AudioEvent::NoteOff { note, channel });
            }
            AudioEvent::AllNotesOff { channel } => {
                // Les deux côtés : une bascule SoundFont ↔ Odin ne doit jamais
                // laisser une note pendante.
                self.voice_manager.process_event(AudioEvent::AllNotesOff { channel });
                #[cfg(feature = "odin")]
                self.odin.all_notes_off(usize::from(channel));
            }
            AudioEvent::TimingUpdate { .. }
            | AudioEvent::MidiClock { .. }
            | AudioEvent::MeasureTiming { .. } => {}
            // Contrôleurs et programmes : état SoundFont seulement (le preset
            // Odin et le mix de piste viennent du profil).
            _ => self.voice_manager.process_event(event),
        }
    }

    /// Allocation-free: oxisynth writes interleaved stereo straight into
    /// `output`. Non-stereo layouts are not rendered and yield silence.
    /// Les voix Odin actives s'ajoutent après EQ et gain master.
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
            #[cfg(feature = "odin")]
            self.odin.render_add(output, self.mix.master_gain);
        } else {
            output.fill(0.0);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Décale les banques des en-têtes SF2, jamais les samples ni les programmes.
/// Appelé uniquement par le constructeur, avant le démarrage du callback.
fn relocate_font_banks(bytes: &[u8], offset: u16) -> Result<Vec<u8>, &'static str> {
    if bytes.get(..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"sfbk") {
        return Err("Banque de pieds : en-tête RIFF/sfbk invalide");
    }
    let size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let end = size.checked_add(8).ok_or("Banque de pieds : taille RIFF excessive")?;
    let chunks = bytes.get(12..end).ok_or("Banque de pieds : RIFF tronqué")?;
    let pdta = find_sf2_chunk(chunks, b"LIST", Some(b"pdta"))?;
    let phdr = find_sf2_chunk(&chunks[pdta.start + 4..pdta.end], b"phdr", None)?;
    let start = 12 + pdta.start + 4 + phdr.start;
    let end = 12 + pdta.start + 4 + phdr.end;
    let headers = &bytes[start..end];
    if headers.len() < 76 || !headers.len().is_multiple_of(38) {
        return Err("Banque de pieds : en-têtes de presets SF2 invalides");
    }
    let mut prepared = bytes.to_vec();
    // Inclure le terminal : certaines banques omettent son nom EOP et
    // soundfont 0.1.0 l'importe alors comme un preset vide.
    let (headers, _) = prepared[start..end].as_chunks_mut::<38>();
    for header in headers {
        let bank = u16::from_le_bytes([header[22], header[23]]);
        let bank = bank.checked_add(offset).ok_or("Banque de pieds : numéro de banque excessif")?;
        header[22..24].copy_from_slice(&bank.to_le_bytes());
    }
    Ok(prepared)
}

fn find_sf2_chunk(
    chunks: &[u8],
    id: &[u8; 4],
    kind: Option<&[u8; 4]>,
) -> Result<std::ops::Range<usize>, &'static str> {
    let mut position = 0;
    while position < chunks.len() {
        let header = chunks
            .get(position..)
            .and_then(|remaining| remaining.get(..8))
            .ok_or("Banque de pieds : en-tête de bloc tronqué")?;
        let size = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
        let start = position + 8;
        let end = start
            .checked_add(size)
            .filter(|&end| end <= chunks.len())
            .ok_or("Banque de pieds : bloc SF2 tronqué")?;
        let data = &chunks[start..end];
        if &header[..4] == id && kind.is_none_or(|kind| data.get(..4) == Some(kind.as_slice())) {
            return Ok(start..end);
        }
        position = end
            .checked_add(size % 2)
            .filter(|&next| next <= chunks.len())
            .ok_or("Banque de pieds : alignement SF2 tronqué")?;
    }
    Err("Banque de pieds : bloc SF2 requis absent")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named_font(name: &[u8]) -> Vec<u8> {
        let mut bytes = include_bytes!("../../../assets/test.sf2").to_vec();
        let phdr = bytes.windows(4).position(|chunk| chunk == b"phdr").unwrap() + 8;
        bytes[phdr..phdr + 20].fill(0);
        bytes[phdr..phdr + name.len()].copy_from_slice(name);
        bytes
    }

    #[test]
    fn gm_piano_is_not_masked_by_the_feet_font() {
        let gm = named_font(b"GM piano");
        let mut feet = named_font(b"Dedicated feet");
        // Même terminal anonyme que la banque harmonium_trad fournie.
        let phdr = feet.windows(4).position(|chunk| chunk == b"phdr").unwrap();
        let size = u32::from_le_bytes(feet[phdr + 4..phdr + 8].try_into().unwrap()) as usize;
        feet[phdr + 8 + size - 38..phdr + 8 + size - 18].fill(0);
        let mut backend = SynthBackend::new(44100.0, Some(&gm), Some(&feet), &[0; 5], &[]);
        backend.handle_event(AudioEvent::ProgramChange { channel: 1, program: 0 });
        let piano = backend.voice_manager.synth.channel_preset(1).unwrap();
        assert_eq!(piano.name(), "GM piano");
        assert_eq!(piano.banknum(), 0);
        for channel in [3, 4] {
            let preset = backend.voice_manager.synth.channel_preset(channel).unwrap();
            assert_eq!(preset.name(), "Dedicated feet");
            assert_eq!(preset.banknum(), FEET_BANK_OFFSET);
            assert_eq!(backend.voice_manager.midi_channel_map[channel as usize], channel);
        }

        let mut reference = SynthBackend::new(44100.0, Some(&gm), None, &[0; 5], &[]);
        reference.handle_event(AudioEvent::ProgramChange { channel: 1, program: 0 });
        for synth in [&mut backend, &mut reference] {
            synth.handle_event(AudioEvent::NoteOn { note: 60, velocity: 100, channel: 1 });
        }
        let mut actual = [0.0; 4096];
        let mut expected = [0.0; 4096];
        backend.process_buffer(&mut actual, 2);
        reference.process_buffer(&mut expected, 2);
        assert!(expected.iter().any(|sample| sample.abs() > 0.001));
        assert_eq!(actual, expected);
    }

    #[test]
    fn malformed_or_overflowing_font_banks_are_rejected() {
        let bytes = named_font(b"Feet");
        for end in [0, 4, 11, bytes.len() - 1] {
            assert!(relocate_font_banks(&bytes[..end], 1).is_err());
        }
        let mut overflowing = bytes.clone();
        let phdr = overflowing.windows(4).position(|chunk| chunk == b"phdr").unwrap() + 8;
        overflowing[phdr + 22..phdr + 24].copy_from_slice(&u16::MAX.to_le_bytes());
        assert!(relocate_font_banks(&overflowing, 1).is_err());
        let mut truncated_chunk = bytes;
        let phdr = truncated_chunk.windows(4).position(|chunk| chunk == b"phdr").unwrap();
        truncated_chunk[phdr + 4..phdr + 8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(relocate_font_banks(&truncated_chunk, 1).is_err());
    }
}

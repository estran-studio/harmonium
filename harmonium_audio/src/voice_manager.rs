use std::io::Cursor;

use harmonium_core::events::AudioEvent;

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

/// Programme GM du violon (40, index 0-based).
pub const GM_PROGRAM_VIOLIN: u8 = 40;
/// Programme GM pour la guitare acoustique cordes acier (`Chords`, D10 : 25 1-based = 24 0-based).
pub const GM_PROGRAM_STEEL_GUITAR: u8 = 24;
/// Programme GM pour la basse acoustique (`Bass`, D10 : 33 1-based = 32 0-based).
pub const GM_PROGRAM_ACOUSTIC_BASS: u8 = 32;
/// Canal MIDI GM dédié aux percussions / podorythmie (`Heel` et `Toe`, D10).
pub const GM_DRUM_CHANNEL: u8 = 9;

/// Décalage de banque dédié à la SoundFont des pieds (`harmonium_trad.sf2`, SK2).
/// Quand cette banque est chargée, Heel/Toe sélectionnent la banque
/// `FEET_BANK_OFFSET` (programme 0) sur leurs canaux MIDI propres (3 et 4).
pub const FEET_BANK_OFFSET: u32 = 1;

/// Programmes des pistes à hauteur définie, résolus depuis la liste canonique.
/// (sur la pile : utilisable dans le callback sans allouer).
#[must_use]
pub fn gm_programs_for_profile(
    instruments: &harmonium_core::style_profile::ProfileInstruments,
) -> arrayvec::ArrayVec<(u8, u8), { harmonium_core::timeline::TrackId::ALL.len() }> {
    use harmonium_core::timeline::TrackId;
    let mut progs = arrayvec::ArrayVec::new();
    for track in TrackId::ALL {
        if !matches!(track, TrackId::Heel | TrackId::Toe)
            && let Some(program) = instruments.program(track)
        {
            progs.push((track.channel(), program));
        }
    }
    progs
}
/// Détermine si une séquence de numéros de banques contient la banque 128 (kit de batterie GM).
#[must_use]
pub fn has_bank_128_in_presets(banks: impl IntoIterator<Item = u32>) -> bool {
    banks.into_iter().any(|bank| bank == 128)
}

pub struct VoiceManager {
    pub synth: oxisynth::Synth,
    pub channel_routing: [ChannelRoute; 16],
    /// Bank currently selected on each logical channel; `None` until the first
    /// `set_channel_route` so the initial Bank Select / Program Change is sent.
    pub current_banks: [Option<u32>; 16],
    /// Program currently selected on each logical channel.
    pub current_programs: [u8; 16],
    /// Maps logical channels to MIDI channels. Identité par défaut ; la
    /// banque de percussion du profil décide (`128` → canal GM 9, sinon
    /// canaux propres, voir `apply_profile_programs`).
    pub midi_channel_map: [u8; 16],
}

/// Default map: logical channel N → MIDI channel N. Le profil décide du
/// routage des pieds via `apply_profile_programs` (un seul chemin).
fn default_midi_channel_map() -> [u8; 16] {
    let mut map = [0u8; 16];
    for (i, slot) in map.iter_mut().enumerate() {
        *slot = i as u8;
    }
    map
}

fn default_channel_programs() -> [u8; 16] {
    let mut progs = [0u8; 16];
    for (channel, program) in
        gm_programs_for_profile(&harmonium_core::StyleProfile::builtin().instruments)
    {
        progs[channel as usize] = program;
    }
    progs
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
            current_programs: default_channel_programs(),
            midi_channel_map: default_midi_channel_map(),
        };
        if let Some(bytes) = sf2_bytes {
            vm.add_font(0, bytes);
        }

        vm
    }

    pub(crate) fn add_font(&mut self, bank_id: u32, bytes: &[u8]) -> bool {
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
                true
            }
            Ok(Err(e)) => {
                harmonium_core::log::error(&format!("SoundFont loading failed: {e}"));
                false
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
                false
            }
        }
    }

    /// Applique l'instrumentarium déjà résolu, sans allocation dans le callback.
    ///
    /// Un seul chemin pour les pieds : la banque de percussion du profil
    /// décide. Banque `128` (kit GM : marche, `nt_desert`) : Heel (3) et Toe
    /// (4) partagent le canal MIDI 9 avec cette banque. Toute autre banque
    /// (pieds `harmonium_trad.sf2` en banque `1`) : canaux MIDI propres (3
    /// et 4) avec la banque et le programme du profil. Sans la banque
    /// demandée, les pieds sont silencieux (avertissement au chargement).
    pub fn apply_profile_programs(
        &mut self,
        instruments: &harmonium_core::style_profile::ProfileInstruments,
    ) {
        for (channel, program) in gm_programs_for_profile(instruments) {
            self.set_channel_program(usize::from(channel), program);
        }
        let bank = u32::from(instruments.percussion_bank);
        if bank == 128 {
            self.midi_channel_map[3] = GM_DRUM_CHANNEL;
            self.midi_channel_map[4] = GM_DRUM_CHANNEL;
        } else {
            self.midi_channel_map[3] = 3;
            self.midi_channel_map[4] = 4;
        }
        // La carte précède les ordres : route et programme partent sur le
        // canal MIDI résolu ci-dessus.
        for channel in [3, 4] {
            self.set_channel_route(channel, ChannelRoute { bank });
            self.set_channel_program(channel, instruments.percussion_program);
        }
    }

    /// Détermine si la SoundFont chargée dans oxisynth contient un preset en banque 128 (kit de batterie GM).
    pub fn has_bank_128_preset(&mut self) -> bool {
        // Le canal 9 est configuré avec banknum 128 par défaut dans oxisynth.
        // Si le SoundFont possède le preset 0 en banque 128 (Standard Drum Kit),
        // channel_preset(9) retourne Some(...) avec un banknum de 128.
        if let Some(preset) = self.synth.channel_preset(GM_DRUM_CHANNEL) {
            if preset.banknum() == 128 {
                return true;
            }
        }

        // Si le preset 0 n'est pas présent, teste les programmes 1..=127 au cas où un kit spécialisé est utilisé
        for prog in 1..=127 {
            let _ = self.synth.send_event(oxisynth::MidiEvent::ProgramChange {
                channel: GM_DRUM_CHANNEL,
                program_id: prog,
            });
            if let Some(preset) = self.synth.channel_preset(GM_DRUM_CHANNEL) {
                if preset.banknum() == 128 {
                    let _ = self.synth.send_event(oxisynth::MidiEvent::ProgramChange {
                        channel: GM_DRUM_CHANNEL,
                        program_id: 0,
                    });
                    return true;
                }
            }
        }

        let _ = self.synth.send_event(oxisynth::MidiEvent::ProgramChange {
            channel: GM_DRUM_CHANNEL,
            program_id: 0,
        });
        false
    }

    /// Liste les programmes GM demandés par Harmonium absents de la SoundFont chargée.
    ///
    /// Calculé lors de l'initialisation hors du callback temps réel.
    pub fn missing_gm_programs(&mut self) -> Vec<&'static str> {
        let mut missing = Vec::new();

        if !self.has_bank_128_preset() {
            missing.push("kit de batterie (banque 128)");
        }

        // Teste les instruments mélodiques et harmoniques en banque 0 sur le canal 15
        let _ = self.synth.send_event(oxisynth::MidiEvent::ControlChange {
            channel: 15,
            ctrl: 0,
            value: 0,
        });

        let required = [
            (GM_PROGRAM_STEEL_GUITAR, "guitare acier (programme 24)"),
            (GM_PROGRAM_ACOUSTIC_BASS, "basse acoustique (programme 32)"),
            (GM_PROGRAM_VIOLIN, "violon (programme 40)"),
        ];

        for (prog, name) in required {
            let _ = self
                .synth
                .send_event(oxisynth::MidiEvent::ProgramChange { channel: 15, program_id: prog });
            let present = match self.synth.channel_preset(15) {
                Some(p) => p.banknum() == 0 && p.num() == u32::from(prog),
                None => false,
            };
            if !present {
                missing.push(name);
            }
        }

        // Rétablit le canal 15 selon son état nominal
        let route = self.channel_routing[15];
        let prog = self.current_programs[15];
        let _ = self.synth.send_event(oxisynth::MidiEvent::ControlChange {
            channel: 15,
            ctrl: 0,
            value: route.bank as u8,
        });
        let _ = self
            .synth
            .send_event(oxisynth::MidiEvent::ProgramChange { channel: 15, program_id: prog });

        missing
    }

    /// Send a GM Program Change to the MIDI channel mapped from a logical channel.
    pub fn set_channel_program(&mut self, logical_channel: usize, program: u8) {
        if logical_channel < 16 {
            self.current_programs[logical_channel] = program;
            let midi_ch = self.midi_channel_map[logical_channel];
            let _ = self.synth.send_event(oxisynth::MidiEvent::ProgramChange {
                channel: midi_ch,
                program_id: program,
            });
        }
    }

    /// Retourne le programme GM courant d'un canal logique (`0..16`).
    #[must_use]
    pub const fn channel_program(&self, logical_channel: usize) -> Option<u8> {
        if logical_channel < 16 { Some(self.current_programs[logical_channel]) } else { None }
    }

    /// Route a logical channel to a SoundFont bank. Sends Bank Select (CC 0)
    /// and the channel's active Program Change only when the bank actually changes.
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
                    program_id: self.current_programs[channel],
                });
                self.current_banks[channel] = Some(route.bank);
            }
        }
    }

    pub fn process_event(&mut self, event: AudioEvent) {
        match event {
            AudioEvent::NoteOn { note, velocity, channel } => {
                // Legato monophonique (G7e) : réalisé par le chevauchement de
                // notes du playhead (NoteOn de la nouvelle hauteur, puis NoteOff
                // de l'ancienne légèrement après), pas ici. `oxisynth` (dérivé de
                // FluidSynth) stocke les contrôleurs CC 126 (mode mono) et CC 5/65
                // (portamento) mais ne les interprète pas : aucun mode mono ni
                // glissando de hauteur n'est produit par cet appel. La basse reste
                // polyphonique dans le rendu SoundFont ; le vrai glissando est
                // renvoyé au moteur Odin (TODO G7e, après G6d).
                let midi_ch = self.midi_channel_map[channel as usize];
                // Dette tierce assumée et circonscrite : `oxisynth` alloue puis libère
                // un `Vec` de 64 pointeurs à chaque NoteOn (`core/midi.rs`, `inner_noteon`).
                // Seul cet appel échappe à la garde ; rendu, NoteOff, CC et tout le code
                // de ce dépôt restent sous `RTCheckAllocator`.
                let _allowed = crate::realtime::rt_check::allow_allocation();
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
            }
            AudioEvent::ProgramChange { channel, program } => {
                self.set_channel_program(channel as usize, program);
            }
            AudioEvent::UpdateMusicalParams { params } => {
                self.apply_profile_programs(&params.instruments);
            }
            AudioEvent::TimingUpdate { .. }
            | AudioEvent::MidiClock { .. }
            | AudioEvent::MeasureTiming { .. }
            | AudioEvent::SetChannelRoute { .. }
            | AudioEvent::StartRecording { .. }
            | AudioEvent::StopRecording { .. }
            | AudioEvent::SetMix { .. } => {}
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bank_128_detection_synthetic() {
        let mut bytes = match std::fs::read("../assets/test.sf2") {
            Ok(b) => b,
            Err(_) => return,
        };
        let phdr_pos = match bytes.windows(4).position(|w| w == b"phdr") {
            Some(pos) => pos,
            None => return,
        };
        let bank_offset = phdr_pos + 8 + 22;
        bytes[bank_offset] = 128;
        bytes[bank_offset + 1] = 0;

        let mut vm = VoiceManager::new(Some(&bytes), 44100.0);
        assert!(
            vm.has_bank_128_preset(),
            "La SoundFont synthétique avec banque 128 doit être détectée GM"
        );
    }

    #[test]
    fn test_bank_128_detection_test_sf2_is_false() {
        let bytes = match std::fs::read("../assets/test.sf2") {
            Ok(b) => b,
            Err(_) => return,
        };
        let mut vm = VoiceManager::new(Some(&bytes), 44100.0);
        assert!(!vm.has_bank_128_preset(), "test.sf2 ne contient pas de banque 128");

        let missing = vm.missing_gm_programs();
        assert!(missing.contains(&"kit de batterie (banque 128)"));
        assert!(missing.contains(&"guitare acier (programme 24)"));
        assert!(missing.contains(&"basse acoustique (programme 32)"));
        assert!(missing.contains(&"violon (programme 40)"));
        assert!(!missing.contains(&"piano acoustique (programme 0)"));
    }

    #[test]
    fn test_bank_128_detection_generaluser_gs() {
        if let Ok(bytes) = std::fs::read("../assets/GeneralUser-GS.sf2") {
            let mut vm = VoiceManager::new(Some(&bytes), 44100.0);
            assert!(vm.has_bank_128_preset(), "GeneralUser-GS doit être détecté GM");
            let missing = vm.missing_gm_programs();
            assert!(
                missing.is_empty(),
                "GeneralUser-GS contient tous les programmes requis: {missing:?}"
            );
        }
    }

    #[test]
    fn test_has_bank_128_in_presets_list() {
        assert!(!has_bank_128_in_presets([0, 1, 2]));
        assert!(has_bank_128_in_presets([0, 128, 1]));
        assert!(has_bank_128_in_presets(vec![128]));
        assert!(!has_bank_128_in_presets(Vec::<u32>::new()));
    }

    #[test]
    fn test_percussion_routing_follows_profile_bank_128() {
        let mut vm = VoiceManager::new(None, 44100.0);
        let mut instruments = harmonium_core::StyleProfile::builtin().instruments;
        instruments.percussion_bank = 128;
        instruments.percussion_program = 0;
        vm.apply_profile_programs(&instruments);
        assert_eq!(vm.midi_channel_map[3], GM_DRUM_CHANNEL, "kit GM sur canal 9");
        assert_eq!(vm.midi_channel_map[4], GM_DRUM_CHANNEL, "kit GM sur canal 9");
        assert_eq!(vm.channel_routing[3].bank, 128, "Heel → banque du profil");
        assert_eq!(vm.channel_routing[4].bank, 128, "Toe → banque du profil");
    }

    #[test]
    fn test_percussion_routing_follows_profile_feet_bank() {
        let mut vm = VoiceManager::new(None, 44100.0);
        let mut instruments = harmonium_core::StyleProfile::builtin().instruments;
        instruments.percussion_bank = 1;
        instruments.percussion_program = 0;
        vm.apply_profile_programs(&instruments);
        assert_eq!(vm.midi_channel_map[3], 3, "pieds sur canaux propres");
        assert_eq!(vm.midi_channel_map[4], 4, "pieds sur canaux propres");
        assert_eq!(vm.channel_routing[3].bank, FEET_BANK_OFFSET, "Heel → banque du profil");
        assert_eq!(vm.channel_routing[4].bank, FEET_BANK_OFFSET, "Toe → banque du profil");
        assert_eq!(vm.channel_program(3), Some(0), "Heel programme du profil");
        assert_eq!(vm.channel_program(4), Some(0), "Toe programme du profil");
    }
}

//! Voix Odin par piste, derrière l'option `odin` (GPLv3 via `odin2-core`).
//!
//! Une voix par [`TrackId`](harmonium_core::timeline::TrackId) (6 moteurs
//! créés à la construction, hors callback) : aucun canal fixe, aucun nom de
//! style. Le preset vient des octets fournis par l'hôte, analysés à la
//! construction ; le morphing (bases + règles du profil × émotions) ne fait
//! que des `f32` et des copies, sans allocation ni verrou.
//!
//! Liaisons `Tied` : le lecteur n'émet pas de `NoteOn` pour une attaque liée
//! qui prolonge une hauteur active — Odin ne réattaque donc jamais sans ordre
//! explicite. Une `NoteOn` sur une hauteur déjà active referme d'abord
//! l'ancienne voix (pas de note orpheline dans le moteur).

use arrayvec::ArrayString;
use harmonium_core::{
    odin::{ODIN_PRESET_NAME_CAP, OdinBaseParams, OdinTrackDesc, OdinVoiceParams},
    timeline::TrackId,
};
use odin2_core::{OdinEngine, SynthEngine, engine::PresetConfig, preset::OdinPreset};

use crate::odin_source::OdinPresetSource;

/// Trames stéréo rendues par appel moteur (tampon fixe, jamais réalloué).
const SCRATCH_FRAMES: usize = 1024;

/// Champs discrets du preset (octaves, demi-tons) : sélectionnés, jamais interpolés.
#[derive(Clone, Copy, Debug, Default)]
struct DiscretePreset {
    octaves: [i32; 3],
    semitones: [i32; 3],
}

/// Socle analysé d'un preset fourni par l'hôte (construction, hors callback).
struct BankEntry {
    name: ArrayString<ODIN_PRESET_NAME_CAP>,
    base: OdinBaseParams,
    discrete: DiscretePreset,
}

/// Une voix Odin : moteur + socle + description du profil + notes actives.
pub struct OdinTrackVoice {
    engine: OdinEngine,
    base: OdinBaseParams,
    discrete: DiscretePreset,
    desc: OdinTrackDesc,
    active: [bool; 128],
    gain: f32,
    pan_left: f32,
    pan_right: f32,
}

impl OdinTrackVoice {
    fn new(sample_rate: f32) -> Self {
        let base = OdinBaseParams {
            osc_volumes: [0.0; 3],
            osc_detune: 0.0,
            filter_frequency: 1000.0,
            filter_resonance: 0.2,
            filter_env_amount: 0.0,
            amp_attack: 0.01,
            amp_decay: 0.1,
            amp_sustain: 0.7,
            amp_release: 0.2,
            filter_attack: 0.01,
            filter_decay: 0.1,
            filter_sustain: 0.7,
            filter_release: 0.2,
            master_volume: 0.0,
        };
        Self {
            engine: OdinEngine::new(sample_rate),
            base,
            discrete: DiscretePreset::default(),
            desc: OdinTrackDesc::default(),
            active: [false; 128],
            gain: 1.0,
            pan_left: 1.0,
            pan_right: 1.0,
        }
    }

    /// Vrai si le profil déclare un preset résolu pour cette piste.
    fn is_active(&self) -> bool {
        !self.desc.is_empty()
    }

    /// Applique le morphing courant au moteur : copies et `f32` purs.
    fn refresh(&mut self, emotions: [f32; 4]) {
        let morphed = OdinVoiceParams::morphed(&self.base, self.desc.rules(), emotions);
        self.engine.load_config(PresetConfig {
            osc_volumes: morphed.osc_volumes,
            osc_octaves: self.discrete.octaves,
            osc_semitones: self.discrete.semitones,
            osc_detune: morphed.osc_detune,
            filter_frequency: morphed.filter_frequency,
            filter_resonance: morphed.filter_resonance,
            filter_env_amount: morphed.filter_env_amount,
            amp_attack: morphed.amp_attack,
            amp_decay: morphed.amp_decay,
            amp_sustain: morphed.amp_sustain,
            amp_release: morphed.amp_release,
            filter_attack: morphed.filter_attack,
            filter_decay: morphed.filter_decay,
            filter_sustain: morphed.filter_sustain,
            filter_release: morphed.filter_release,
            master_volume: morphed.master_volume,
        });
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        if velocity == 0 {
            self.note_off(note);
            return;
        }
        if self.active[usize::from(note)] {
            self.engine.note_off(note);
        }
        self.engine.note_on(note, velocity);
        self.active[usize::from(note)] = true;
    }

    fn note_off(&mut self, note: u8) {
        self.engine.note_off(note);
        self.active[usize::from(note)] = false;
    }

    fn all_notes_off(&mut self) {
        for note in 0..128 {
            self.engine.note_off(note);
        }
        self.active = [false; 128];
    }
}

/// Mixeur Odin : banque analysée + 6 voix + émotions courantes.
pub struct OdinMixer {
    bank: Vec<BankEntry>,
    voices: [OdinTrackVoice; TrackId::ALL.len()],
    emotions: [f32; 4],
    scratch: Box<[f32; SCRATCH_FRAMES * 2]>,
}

impl OdinMixer {
    /// Analyse les presets fournis (hors callback) ; les octets illisibles
    /// sont ignorés avec un avertissement (la piste joue SoundFont).
    #[must_use]
    pub fn new(sample_rate: f32, presets: &[OdinPresetSource]) -> Self {
        let mut bank = Vec::new();
        for source in presets {
            let mut name = ArrayString::new();
            let mut too_long = false;
            for ch in source.name.chars() {
                if name.try_push(ch).is_err() {
                    too_long = true;
                    break;
                }
            }
            if too_long || name.is_empty() {
                harmonium_core::log::warn(&format!(
                    "Preset Odin ignoré : nom vide ou trop long ({} caractères max)",
                    ODIN_PRESET_NAME_CAP
                ));
                continue;
            }
            match OdinPreset::from_bytes(source.bytes) {
                Ok(preset) => {
                    let config = PresetConfig::from_preset(&preset);
                    bank.push(BankEntry {
                        name,
                        base: OdinBaseParams {
                            osc_volumes: config.osc_volumes,
                            osc_detune: config.osc_detune,
                            filter_frequency: config.filter_frequency,
                            filter_resonance: config.filter_resonance,
                            filter_env_amount: config.filter_env_amount,
                            amp_attack: config.amp_attack,
                            amp_decay: config.amp_decay,
                            amp_sustain: config.amp_sustain,
                            amp_release: config.amp_release,
                            filter_attack: config.filter_attack,
                            filter_decay: config.filter_decay,
                            filter_sustain: config.filter_sustain,
                            filter_release: config.filter_release,
                            master_volume: config.master_volume,
                        },
                        discrete: DiscretePreset {
                            octaves: config.osc_octaves,
                            semitones: config.osc_semitones,
                        },
                    });
                    harmonium_core::log::info(&format!("Preset Odin chargé : {name}"));
                }
                Err(error) => {
                    harmonium_core::log::warn(&format!(
                        "Preset Odin `{name}` illisible, piste en SoundFont : {error}"
                    ));
                }
            }
        }
        Self {
            bank,
            voices: TrackId::ALL.map(|_| OdinTrackVoice::new(sample_rate)),
            emotions: harmonium_core::params::EngineParams::default().emotions(),
            scratch: Box::new([0.0; SCRATCH_FRAMES * 2]),
        }
    }

    /// Piste jouée par Odin (preset déclaré et résolu) ?
    #[must_use]
    pub fn is_odin_channel(&self, channel: usize) -> bool {
        self.voices.get(channel).is_some_and(|voice| voice.is_active())
    }

    /// Bascule SoundFont ↔ Odin d'après le profil (commande audio, sans allocation).
    pub fn apply_descs(&mut self, descs: &[OdinTrackDesc; TrackId::ALL.len()]) {
        for (voice, desc) in self.voices.iter_mut().zip(descs.iter()) {
            let was_active = voice.is_active();
            if desc.is_empty() {
                if was_active {
                    voice.all_notes_off();
                }
                voice.desc = OdinTrackDesc::default();
                continue;
            }
            let resolved = self.bank.iter().find(|entry| entry.name == desc.preset);
            match resolved {
                Some(entry) => {
                    let preset_changed = voice.desc.preset != desc.preset;
                    if was_active && preset_changed {
                        voice.all_notes_off();
                    }
                    voice.base = entry.base;
                    voice.discrete = entry.discrete;
                    voice.desc = *desc;
                    voice.refresh(self.emotions);
                }
                // Preset absent des octets fournis par l'hôte : la piste joue
                // son programme SoundFont (aucun journal ici : le callback
                // n'alloue pas).
                None => {
                    if was_active {
                        voice.all_notes_off();
                    }
                    voice.desc = OdinTrackDesc::default();
                }
            }
        }
    }

    /// Émotions courantes : remorphe les voix actives (copies et `f32` purs).
    pub fn apply_emotions(&mut self, emotions: [f32; 4]) {
        self.emotions = emotions;
        for voice in &mut self.voices {
            if voice.is_active() {
                voice.refresh(emotions);
            }
        }
    }

    /// Niveaux du mix par piste (volume quadratique, panoramique à puissance
    /// égale) ; réverb et EQ du mix restent réservés aux pistes SoundFont.
    pub fn apply_mix(&mut self, mix: harmonium_core::mix::PreparedMix) {
        for (voice, track) in self.voices.iter_mut().zip(mix.tracks.iter()) {
            let volume = f32::from(track.volume) / 127.0;
            voice.gain = volume * volume;
            let angle = f32::from(track.pan) / 127.0 * std::f32::consts::FRAC_PI_2;
            voice.pan_left = angle.cos();
            voice.pan_right = angle.sin();
        }
    }

    pub fn note_on(&mut self, channel: usize, note: u8, velocity: u8) {
        if let Some(voice) = self.voices.get_mut(channel) {
            voice.note_on(note, velocity);
        }
    }

    pub fn note_off(&mut self, channel: usize, note: u8) {
        if let Some(voice) = self.voices.get_mut(channel) {
            voice.note_off(note);
        }
    }

    pub fn all_notes_off(&mut self, channel: usize) {
        if let Some(voice) = self.voices.get_mut(channel) {
            voice.all_notes_off();
        }
    }

    /// Rend les voix actives et les ajoute à `output` (stéréo entrelacée),
    /// par tranches du tampon fixe.
    pub fn render_add(&mut self, output: &mut [f32]) {
        if !output.len().is_multiple_of(2) {
            return;
        }
        for voice in &mut self.voices {
            if !voice.is_active() || voice.gain < 0.001 {
                continue;
            }
            let gain = voice.gain;
            let (pan_left, pan_right) = (voice.pan_left, voice.pan_right);
            let mut offset = 0;
            while offset < output.len() {
                let frames = (output.len() - offset) / 2;
                let chunk = frames.min(SCRATCH_FRAMES);
                let len = chunk * 2;
                {
                    let (scratch, engine) = (&mut self.scratch[..len], &mut voice.engine);
                    engine.process(scratch, 2);
                    let out = &mut output[offset..offset + len];
                    for (index, sample) in scratch.iter().enumerate() {
                        let pan = if index % 2 == 0 { pan_left } else { pan_right };
                        out[index] += sample * gain * pan;
                    }
                }
                offset += len;
            }
        }
    }
}

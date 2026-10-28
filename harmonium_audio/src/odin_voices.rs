//! Voix Odin par piste, derrière l'option `odin` (GPLv3 via `odin2-core`).
//!
//! Une voix par [`TrackId`](harmonium_core::timeline::TrackId) (6 moteurs
//! créés à la construction, hors callback) : aucun canal fixe, aucun nom de
//! style. Le callback ne calcule aucun morphing : le thread principal lui
//! envoie le preset déjà morphé de chaque piste
//! ([`OdinPatches`](harmonium_core::odin::OdinPatches), `Copy`), qu'il copie
//! dans le moteur sans allocation ni verrou.
//!
//! Liaisons `Tied` : le lecteur n'émet pas de `NoteOn` pour une attaque liée
//! qui prolonge une hauteur active — Odin ne réattaque donc jamais sans ordre
//! explicite. Une `NoteOn` sur une hauteur déjà active referme d'abord
//! l'ancienne voix (pas de note orpheline dans le moteur).

use harmonium_core::{
    mix::PreparedMix,
    odin::{OdinPatches, OdinTrackPatch},
    timeline::TrackId,
};
use odin2_core::{OdinEngine, SynthEngine, engine::PresetConfig};

/// Trames stéréo rendues par appel moteur (tampon fixe, jamais réalloué).
const SCRATCH_FRAMES: usize = 1024;

/// Unique écriture d'un preset dans `odin2-core` : charge préparée par le
/// thread principal → `PresetConfig` (copie de champs, aucun calcul).
fn preset_config(patch: &OdinTrackPatch) -> PresetConfig {
    let params = &patch.params;
    PresetConfig {
        osc_volumes: params.osc_volumes,
        osc_octaves: patch.discrete.octaves,
        osc_semitones: patch.discrete.semitones,
        osc_detune: params.osc_detune,
        filter_frequency: params.filter_frequency,
        filter_resonance: params.filter_resonance,
        filter_env_amount: params.filter_env_amount,
        amp_attack: params.amp_attack,
        amp_decay: params.amp_decay,
        amp_sustain: params.amp_sustain,
        amp_release: params.amp_release,
        filter_attack: params.filter_attack,
        filter_decay: params.filter_decay,
        filter_sustain: params.filter_sustain,
        filter_release: params.filter_release,
        master_volume: params.master_volume,
    }
}

/// Une voix Odin : moteur + rang du preset joué + notes actives.
pub struct OdinTrackVoice {
    engine: OdinEngine,
    /// Rang du preset de la banque de l'hôte ; `None` : piste SoundFont.
    preset: Option<usize>,
    active: [bool; 128],
    gain: f32,
    pan_left: f32,
    pan_right: f32,
}

impl OdinTrackVoice {
    fn new(sample_rate: f32) -> Self {
        Self {
            engine: OdinEngine::new(sample_rate),
            preset: None,
            active: [false; 128],
            gain: 1.0,
            pan_left: 1.0,
            pan_right: 1.0,
        }
    }

    /// Vrai si la piste joue Odin (preset reçu du thread principal).
    fn is_active(&self) -> bool {
        self.preset.is_some()
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

/// Mixeur Odin : 6 voix et tampon de rendu, créés à la construction.
pub struct OdinMixer {
    voices: [OdinTrackVoice; TrackId::ALL.len()],
    scratch: Box<[f32; SCRATCH_FRAMES * 2]>,
}

impl OdinMixer {
    #[must_use]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            voices: TrackId::ALL.map(|_| OdinTrackVoice::new(sample_rate)),
            scratch: Box::new([0.0; SCRATCH_FRAMES * 2]),
        }
    }

    /// Piste jouée par Odin (preset reçu) ?
    #[must_use]
    pub fn is_odin_channel(&self, channel: usize) -> bool {
        self.voices.get(channel).is_some_and(|voice| voice.is_active())
    }

    /// Applique les presets morphés (commande audio : copies seulement). Une
    /// piste qui quitte Odin ou change de preset coupe ses notes ; l'appelant
    /// coupe le côté SoundFont d'une bascule.
    pub fn apply_patches(&mut self, patches: &OdinPatches) {
        for (voice, patch) in self.voices.iter_mut().zip(patches.iter()) {
            match patch {
                Some(patch) => {
                    if voice.preset.is_some_and(|current| current != patch.preset) {
                        voice.all_notes_off();
                    }
                    voice.preset = Some(patch.preset);
                    voice.engine.load_config(preset_config(patch));
                }
                None => {
                    if voice.is_active() {
                        voice.all_notes_off();
                    }
                    voice.preset = None;
                }
            }
        }
    }

    /// Niveaux du mix par piste (volume quadratique, panoramique à puissance
    /// égale) ; réverb et EQ du mix restent réservés aux pistes SoundFont.
    pub fn apply_mix(&mut self, mix: PreparedMix) {
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

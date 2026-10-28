//! Voix Odin complètes par piste. Construction et destruction sur le thread
//! principal ; échange de boîtes et paramètres numériques seulement ici.
//! Les notes Tied restent gouvernées par le lecteur G5, sans réattaque.

use harmonium_core::{mix::PreparedMix, odin::OdinMorphParam, timeline::TrackId};
use odin2_core::{OdinEngine, ParameterId, SynthEngine};

use crate::odin_bank::OdinUpdate;

const SCRATCH_FRAMES: usize = 1024;

struct OdinTrackVoice {
    engine: Option<Box<OdinEngine>>,
    preset: Option<usize>,
    active: [bool; 128],
    gain: f32,
    pan_left: f32,
    pan_right: f32,
}

impl OdinTrackVoice {
    fn new() -> Self {
        Self {
            engine: None,
            preset: None,
            active: [false; 128],
            gain: 1.0,
            pan_left: 1.0,
            pan_right: 1.0,
        }
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        if velocity == 0 {
            self.note_off(note);
            return;
        }
        if let Some(engine) = &mut self.engine {
            if self.active[usize::from(note)] {
                engine.note_off(note);
            }
            engine.note_on(note, velocity);
            self.active[usize::from(note)] = true;
        }
    }

    fn note_off(&mut self, note: u8) {
        if let Some(engine) = &mut self.engine {
            engine.note_off(note);
        }
        self.active[usize::from(note)] = false;
    }

    fn all_notes_off(&mut self) {
        if let Some(engine) = &mut self.engine {
            engine.all_sound_off();
        }
        self.active.fill(false);
    }
}

/// Mixeur sans construction ni chargement de moteur dans le callback.
pub struct OdinMixer {
    voices: [OdinTrackVoice; TrackId::ALL.len()],
    ids: [ParameterId; OdinMorphParam::COUNT],
    scratch: Box<[f32; SCRATCH_FRAMES * 2]>,
    tempo: f32,
}

impl Default for OdinMixer {
    fn default() -> Self {
        Self::new()
    }
}

impl OdinMixer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            voices: TrackId::ALL.map(|_| OdinTrackVoice::new()),
            ids: OdinMorphParam::ALL.map(|param| {
                ParameterId::from_name(param.name()).expect("identifiant du catalogue Odin épinglé")
            }),
            scratch: Box::new([0.0; SCRATCH_FRAMES * 2]),
            tempo: 120.0,
        }
    }

    #[must_use]
    pub fn is_odin_channel(&self, channel: usize) -> bool {
        self.voices.get(channel).is_some_and(|voice| voice.preset.is_some())
    }

    /// Échange atomique du point de vue du flux de commandes. La transaction
    /// repart avec les instances retirées ; aucun Drop de moteur ici.
    pub fn apply_update(&mut self, update: &mut OdinUpdate) {
        for (index, voice) in self.voices.iter_mut().enumerate() {
            let preset = update.patches[index].map(|patch| patch.preset);
            if preset != voice.preset {
                assert_eq!(
                    update.engines[index].is_some(),
                    preset.is_some(),
                    "instance Odin manquante dans la transaction"
                );
                std::mem::swap(&mut voice.engine, &mut update.engines[index]);
                voice.preset = preset;
                voice.active.fill(false);
                if let Some(engine) = &mut voice.engine {
                    engine.set_tempo(self.tempo);
                }
            } else {
                assert!(
                    update.engines[index].is_none(),
                    "instance Odin inattendue dans la transaction"
                );
            }
            if let (Some(patch), Some(engine)) = (update.patches[index], &mut voice.engine) {
                let values: [_; OdinMorphParam::COUNT] =
                    std::array::from_fn(|i| (self.ids[i], patch.params.values[i]));
                engine
                    .set_parameters(&values)
                    .expect("paramètres Odin finis préparés sur le thread principal");
            }
        }
    }

    /// Tempo du transport pour les effets et LFO synchronisés du preset.
    pub fn set_tempo(&mut self, bpm: f32) {
        if !bpm.is_finite() || bpm <= 0.0 || bpm == self.tempo {
            return;
        }
        self.tempo = bpm;
        for voice in &mut self.voices {
            if let Some(engine) = &mut voice.engine {
                engine.set_tempo(bpm);
            }
        }
    }

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

    pub fn render_add(&mut self, output: &mut [f32]) {
        if !output.len().is_multiple_of(2) {
            return;
        }
        for voice in &mut self.voices {
            let Some(engine) = &mut voice.engine else {
                continue;
            };
            if voice.gain < 0.001 {
                continue;
            }
            for out in output.chunks_mut(SCRATCH_FRAMES * 2) {
                let scratch = &mut self.scratch[..out.len()];
                engine.process(scratch, 2);
                for (index, (sample, target)) in scratch.iter().zip(out.iter_mut()).enumerate() {
                    let pan = if index % 2 == 0 { voice.pan_left } else { voice.pan_right };
                    *target += sample * voice.gain * pan;
                }
            }
        }
    }
}

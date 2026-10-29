//! Deux emplacements Odin par piste : courant et entrant/sortant. Les moteurs
//! arrivent préparés et repartent par SPSC ; le callback ne les détruit jamais.

use harmonium_core::{
    mix::PreparedMix,
    odin::{OdinMorphParam, OdinPatches, OdinTrackPatch},
    timeline::TrackId,
};
use odin2_core::{OdinEngine, ParameterId, SynthEngine};

use crate::odin_bank::OdinUpdate;

const SCRATCH_FRAMES: usize = 1024;
pub const ODIN_DIAGNOSTIC_HISTORY: usize = 128;

/// Contrat de diagnostic hors ligne : une entrée par instance effectivement
/// installée, y compris celles déjà retournées à leur propriétaire.
#[derive(Clone, Copy, Debug, Default)]
pub struct OdinInstanceDiagnostic {
    pub id: u64,
    pub channel: usize,
    pub zone: usize,
    pub preset: usize,
    pub installed_bar: usize,
    pub installed_frame: u64,
    pub retired_frame: Option<u64>,
    pub note_on: u64,
    pub note_off: u64,
    pub active_notes: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct OdinDiagnostics {
    pub active: [Option<OdinInstanceDiagnostic>; TrackId::ALL.len()],
    pub retiring: [Option<OdinInstanceDiagnostic>; TrackId::ALL.len()],
    pub retired: [Option<OdinInstanceDiagnostic>; ODIN_DIAGNOSTIC_HISTORY],
    pub retired_count: usize,
    pub history_overflow: u64,
}

struct OdinInstance {
    engine: Box<OdinEngine>,
    patch: OdinTrackPatch,
    velocity: [u8; 128],
    diagnostic: OdinInstanceDiagnostic,
}

impl OdinInstance {
    fn key(&self) -> (usize, usize) {
        (self.patch.zone, self.patch.preset)
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        if velocity == 0 {
            self.note_off(note);
            return;
        }
        // La politique historique de réattaque ferme d'abord l'ancienne note.
        if self.velocity[usize::from(note)] != 0 {
            self.note_off(note);
        }
        self.engine.note_on(note, velocity);
        self.velocity[usize::from(note)] = velocity;
        self.diagnostic.note_on += 1;
        self.diagnostic.active_notes += 1;
    }

    fn note_off(&mut self, note: u8) {
        if self.velocity[usize::from(note)] == 0 {
            return;
        }
        self.engine.note_off(note);
        self.velocity[usize::from(note)] = 0;
        self.diagnostic.note_off += 1;
        self.diagnostic.active_notes -= 1;
    }

    fn release(&mut self) {
        for note in 0..128 {
            self.note_off(note);
        }
    }

    fn apply_parameters(&mut self, ids: &[ParameterId; OdinMorphParam::COUNT]) {
        let values: [_; OdinMorphParam::COUNT] =
            std::array::from_fn(|i| (ids[i], self.patch.params.values[i]));
        self.engine
            .set_parameters(&values)
            .expect("paramètres Odin finis préparés sur le thread principal");
    }

    fn morph(&mut self, patch: OdinTrackPatch, ids: &[ParameterId; OdinMorphParam::COUNT]) {
        let changed = self.patch.params != patch.params;
        self.patch = patch;
        if changed {
            self.apply_parameters(ids);
        }
    }
}

struct OdinTrackVoice {
    current: Option<OdinInstance>,
    retiring: Option<OdinInstance>,
    held: [u8; 128],
    fade_elapsed: usize,
    fading: bool,
    gain: f32,
    pan_left: f32,
    pan_right: f32,
}

impl OdinTrackVoice {
    fn new() -> Self {
        Self {
            current: None,
            retiring: None,
            held: [0; 128],
            fade_elapsed: 0,
            fading: false,
            gain: 1.0,
            pan_left: 1.0,
            pan_right: 1.0,
        }
    }
}

/// Mixeur sans construction, chargement ni destruction de moteur dans le callback.
pub struct OdinMixer {
    voices: [OdinTrackVoice; TrackId::ALL.len()],
    ids: [ParameterId; OdinMorphParam::COUNT],
    scratch: Box<[f32; SCRATCH_FRAMES * 2]>,
    tempo: f32,
    fade_frames: usize,
    frame: u64,
    retired: [Option<OdinInstanceDiagnostic>; ODIN_DIAGNOSTIC_HISTORY],
    retired_count: usize,
    history_overflow: u64,
}

impl OdinMixer {
    #[must_use]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            voices: TrackId::ALL.map(|_| OdinTrackVoice::new()),
            ids: OdinMorphParam::ALL.map(|param| {
                ParameterId::from_name(param.name()).expect("identifiant du catalogue Odin épinglé")
            }),
            scratch: Box::new([0.0; SCRATCH_FRAMES * 2]),
            tempo: 120.0,
            fade_frames: (sample_rate * 0.3).round().max(1.0) as usize,
            frame: 0,
            retired: [None; ODIN_DIAGNOSTIC_HISTORY],
            retired_count: 0,
            history_overflow: 0,
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> OdinDiagnostics {
        OdinDiagnostics {
            active: self
                .voices
                .each_ref()
                .map(|voice| voice.current.as_ref().map(|v| v.diagnostic)),
            retiring: self
                .voices
                .each_ref()
                .map(|voice| voice.retiring.as_ref().map(|v| v.diagnostic)),
            retired: self.retired,
            retired_count: self.retired_count,
            history_overflow: self.history_overflow,
        }
    }

    #[must_use]
    pub fn is_odin_channel(&self, channel: usize) -> bool {
        self.voices.get(channel).is_some_and(|voice| voice.current.is_some())
    }

    #[must_use]
    pub fn is_fading(&self) -> bool {
        self.voices.iter().any(|voice| voice.fading)
    }

    /// Appelé uniquement à la vraie entrée de partie (ou pour l'installation
    /// initiale). Les tenues sont fermées sur l'ancienne instance puis ouvertes
    /// une seule fois sur la nouvelle ; l'ancienne ne reçoit plus de NoteOn.
    pub fn apply_update(&mut self, update: &mut OdinUpdate) {
        assert!(!self.is_fading(), "second emplacement Odin encore occupé");
        for (index, voice) in self.voices.iter_mut().enumerate() {
            assert!(voice.retiring.is_none(), "instance Odin non retournée");
            let target = update.patches[index];
            let key = target.map(|patch| (patch.zone, patch.preset));
            if key != voice.current.as_ref().map(OdinInstance::key) {
                let held = voice.current.as_ref().map_or([0; 128], |instance| instance.velocity);
                if let Some(mut old) = voice.current.take() {
                    old.release();
                    voice.retiring = Some(old);
                    voice.fade_elapsed = 0;
                    voice.fading = update.at_bar != 0;
                    if !voice.fading
                        && let Some(instance) = &mut voice.retiring
                    {
                        instance.diagnostic.retired_frame = Some(update.at_frame);
                    }
                }
                if let Some(patch) = target {
                    let mut instance = OdinInstance {
                        engine: update.engines[index].take().expect("instance Odin préparée"),
                        patch,
                        velocity: [0; 128],
                        diagnostic: OdinInstanceDiagnostic {
                            id: update.instance_ids[index],
                            channel: index,
                            zone: patch.zone,
                            preset: patch.preset,
                            installed_bar: update.at_bar,
                            installed_frame: update.at_frame,
                            ..OdinInstanceDiagnostic::default()
                        },
                    };
                    instance.engine.set_tempo(self.tempo);
                    instance.apply_parameters(&self.ids);
                    // Les événements offset 0 éliminent les notes qui finissent
                    // à la frontière avant de réouvrir les seules vraies tenues.
                    voice.held = held;
                    voice.current = Some(instance);
                }
            } else if let (Some(patch), Some(instance)) = (target, &mut voice.current) {
                instance.morph(patch, &self.ids);
            }
        }
    }

    /// Le propriétaire garde la boîte jusqu'à la fin du fondu. Le déplacement
    /// des moteurs vers cette boîte ne libère aucune ressource dans le callback.
    pub fn finish_update(&mut self, update: &mut OdinUpdate) -> bool {
        if self.is_fading() {
            return false;
        }
        for (index, voice) in self.voices.iter_mut().enumerate() {
            if let Some(instance) = voice.retiring.take() {
                assert!(update.engines[index].is_none());
                let slot = self.retired_count % ODIN_DIAGNOSTIC_HISTORY;
                self.retired[slot] = Some(instance.diagnostic);
                self.retired_count += 1;
                if self.retired_count > ODIN_DIAGNOSTIC_HISTORY {
                    self.history_overflow += 1;
                }
                update.engines[index] = Some(instance.engine);
            }
        }
        true
    }

    /// Morphing de l'instance courante seulement. La queue de réverbération de
    /// l'ancienne conserve son preset et son gain jusqu'à son retrait.
    pub fn morph(&mut self, patches: &[OdinPatches; 2]) {
        for (index, voice) in self.voices.iter_mut().enumerate() {
            if let Some(instance) = &mut voice.current
                && let Some(patch) = patches
                    .iter()
                    .rev()
                    .filter_map(|patches| patches[index])
                    .find(|patch| (patch.zone, patch.preset) == instance.key())
            {
                instance.morph(patch, &self.ids);
            }
        }
    }

    pub fn set_tempo(&mut self, bpm: f32) {
        if !bpm.is_finite() || bpm <= 0.0 || bpm == self.tempo {
            return;
        }
        self.tempo = bpm;
        for voice in &mut self.voices {
            for instance in [&mut voice.current, &mut voice.retiring].into_iter().flatten() {
                instance.engine.set_tempo(bpm);
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
        if let Some(voice) = self.voices.get_mut(channel)
            && let Some(instance) = &mut voice.current
        {
            voice.held[usize::from(note)] = 0;
            instance.note_on(note, velocity);
        }
    }

    pub fn note_off(&mut self, channel: usize, note: u8) {
        if let Some(voice) = self.voices.get_mut(channel)
            && let Some(instance) = &mut voice.current
        {
            voice.held[usize::from(note)] = 0;
            instance.note_off(note);
        }
    }

    pub fn all_notes_off(&mut self, channel: usize) {
        if let Some(voice) = self.voices.get_mut(channel) {
            voice.held.fill(0);
            for instance in [&mut voice.current, &mut voice.retiring].into_iter().flatten() {
                instance.release();
                // Seek/mute conservent la sémantique historique d'arrêt net.
                // Ce chemin n'est jamais utilisé par une transition de zone.
                instance.engine.all_sound_off();
            }
        }
    }

    pub fn render_add(&mut self, output: &mut [f32]) {
        if !output.len().is_multiple_of(2) {
            return;
        }
        for voice in &mut self.voices {
            if voice.current.is_none() && voice.retiring.is_none() {
                continue;
            }
            if let Some(instance) = &mut voice.current {
                for (note, velocity) in voice.held.iter_mut().enumerate() {
                    if *velocity != 0 {
                        instance.note_on(note as u8, *velocity);
                        *velocity = 0;
                    }
                }
            }
            // Même chemin et même ordre d'opérations pour la zone unique.
            if voice.gain < 0.001 && !voice.fading {
                continue;
            }
            let mut offset = 0;
            while offset < output.len() {
                let frames =
                    ((output.len() - offset) / 2).min(SCRATCH_FRAMES).min(if voice.fading {
                        self.fade_frames - voice.fade_elapsed
                    } else {
                        usize::MAX
                    });
                let out = &mut output[offset..offset + frames * 2];
                let scratch = &mut self.scratch[..out.len()];
                for (current, instance) in
                    [(true, &mut voice.current), (false, &mut voice.retiring)]
                {
                    let Some(instance) = instance else { continue };
                    if !current && !voice.fading {
                        continue;
                    }
                    instance.engine.process(scratch, 2);
                    for (index, (sample, target)) in scratch.iter().zip(out.iter_mut()).enumerate()
                    {
                        let pan = if index % 2 == 0 { voice.pan_left } else { voice.pan_right };
                        let mut sample = sample * voice.gain;
                        if instance.patch.gain != 1.0 {
                            sample *= instance.patch.gain;
                        }
                        if voice.fading {
                            let rise =
                                (voice.fade_elapsed + index / 2) as f32 / self.fade_frames as f32;
                            sample *= if current { rise } else { 1.0 - rise };
                        }
                        *target += sample * pan;
                    }
                }
                if voice.fading {
                    voice.fade_elapsed += frames;
                    if voice.fade_elapsed == self.fade_frames {
                        voice.fading = false;
                        if let Some(instance) = &mut voice.retiring {
                            instance.diagnostic.retired_frame =
                                Some(self.frame + (offset / 2 + frames) as u64);
                        }
                    }
                }
                offset += frames * 2;
            }
        }
        self.frame += (output.len() / 2) as u64;
    }
}

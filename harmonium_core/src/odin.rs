//! Instrument Odin par piste, en données.
//!
//! Chaque piste garde son programme SoundFont (rendu sans `odin`) et peut
//! déclarer un preset Odin (nom fourni par l'hôte comme les octets SF2) avec
//! ses réglages de morphing par les 4 émotions. Sans `odin` compilé ou sans
//! preset fourni, la piste joue son programme SoundFont (avertissement au
//! chargement du profil) ; avec `odin` et le preset présent, elle joue Odin.
//! Le thread principal connaît les socles des presets ([`OdinBank`]), calcule
//! le preset morphé de chaque piste à chaque changement d'émotion et envoie au
//! callback une charge `Copy` ([`OdinPatches`]) : le fil audio ne calcule
//! rien. Aucun nom de style ni canal fixe ici : tout est clé par
//! [`TrackId`](crate::timeline::TrackId).
//!
//! ```toml
//! [odin.Bass]
//! preset = "basse_saturee"
//! morph = [
//!   { param = "filter_frequency", arousal = 3500.0, tension = 1500.0 },
//!   { param = "filter_resonance", tension = 0.35 },
//! ]
//! ```

use serde::{Deserialize, Serialize};

use crate::timeline::TrackId;

/// Nom de preset maximal (octets SF2 exclus, nom court fourni par l'hôte).
pub const ODIN_PRESET_NAME_CAP: usize = 64;
/// Règles de morphing maximales par piste (borne du schéma `[odin.<Piste>]`).
pub const ODIN_MAX_RULES: usize = 8;

/// Paramètre continu du preset pilotable par les 4 émotions.
///
/// Sous-ensemble des champs de `PresetConfig` d'`odin2-core` (ceux que le
/// moteur expose : pas de drive ni d'effets dans la configuration
/// simplifiée). Les champs discrets (octaves, demi-tons) ne se morphent pas :
/// ils resteraient faux entre deux valeurs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OdinMorphParam {
    FilterFrequency,
    FilterResonance,
    FilterEnvAmount,
    AmpAttack,
    AmpDecay,
    AmpSustain,
    AmpRelease,
    FilterAttack,
    FilterDecay,
    FilterSustain,
    FilterRelease,
    OscDetune,
    Osc1Volume,
    Osc2Volume,
    Osc3Volume,
    MasterVolume,
}

/// Une règle de morphing : gains additifs par émotion (émotions `0..1`).
///
/// `valeur = base + arousal × gain + valence × gain + density × gain +
/// tension × gain`, bornée par paramètre (voir [`OdinVoiceParams::apply`]).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OdinMorphRule {
    pub param: OdinMorphParam,
    #[serde(default)]
    pub arousal: f32,
    #[serde(default)]
    pub valence: f32,
    #[serde(default)]
    pub density: f32,
    #[serde(default)]
    pub tension: f32,
}

/// Instrument Odin d'une piste, en données du profil.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OdinTrackSpec {
    /// Nom du preset, fourni par l'hôte (`<nom>.odin`) ; sans `/` ni `\`.
    pub preset: String,
    /// Règles de morphing (défaut : preset fixe, insensible aux émotions).
    #[serde(default)]
    pub morph: Vec<OdinMorphRule>,
}

/// Paramètres continus d'une voix Odin : socle du preset (lu du `.odin` par
/// l'hôte, hors callback) comme valeurs après morphing.
///
/// Mêmes champs continus que `PresetConfig` d'`odin2-core`, en `f32` purs :
/// aucune dépendance GPL dans ce module.
///
/// **Ajouter un paramètre** (drive, effets : l'API d'`odin2-core` s'élargit) :
/// un champ ici (+ sa borne dans [`Self::clamp`]), une variante de
/// [`OdinMorphParam`] et son bras dans [`Self::slot_mut`] ; côté audio, sa
/// lecture dans `odin_bank::read_preset` et son écriture dans
/// `odin_voices::preset_config`. Rien d'autre : ni profil, ni lecteur de
/// snapshots, ni message audio.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OdinVoiceParams {
    pub osc_volumes: [f32; 3],
    pub osc_detune: f32,
    pub filter_frequency: f32,
    pub filter_resonance: f32,
    pub filter_env_amount: f32,
    pub amp_attack: f32,
    pub amp_decay: f32,
    pub amp_sustain: f32,
    pub amp_release: f32,
    pub filter_attack: f32,
    pub filter_decay: f32,
    pub filter_sustain: f32,
    pub filter_release: f32,
    pub master_volume: f32,
}

impl OdinVoiceParams {
    /// Morphing déterministe, sans allocation : optique `f32` pure. `self` est
    /// le socle du preset ; le résultat est borné par [`Self::clamp`].
    #[must_use]
    pub fn morphed(&self, rules: &[OdinMorphRule], emotions: [f32; 4]) -> Self {
        let [arousal, valence, density, tension] = emotions;
        let mut out = *self;
        for rule in rules {
            let delta = arousal.mul_add(
                rule.arousal,
                valence
                    .mul_add(rule.valence, density.mul_add(rule.density, tension * rule.tension)),
            );
            *out.slot_mut(rule.param) += delta;
        }
        out.clamp()
    }

    /// Champ piloté par un paramètre de règle : l'unique table
    /// paramètre → champ.
    fn slot_mut(&mut self, param: OdinMorphParam) -> &mut f32 {
        match param {
            OdinMorphParam::FilterFrequency => &mut self.filter_frequency,
            OdinMorphParam::FilterResonance => &mut self.filter_resonance,
            OdinMorphParam::FilterEnvAmount => &mut self.filter_env_amount,
            OdinMorphParam::AmpAttack => &mut self.amp_attack,
            OdinMorphParam::AmpDecay => &mut self.amp_decay,
            OdinMorphParam::AmpSustain => &mut self.amp_sustain,
            OdinMorphParam::AmpRelease => &mut self.amp_release,
            OdinMorphParam::FilterAttack => &mut self.filter_attack,
            OdinMorphParam::FilterDecay => &mut self.filter_decay,
            OdinMorphParam::FilterSustain => &mut self.filter_sustain,
            OdinMorphParam::FilterRelease => &mut self.filter_release,
            OdinMorphParam::OscDetune => &mut self.osc_detune,
            OdinMorphParam::Osc1Volume => &mut self.osc_volumes[0],
            OdinMorphParam::Osc2Volume => &mut self.osc_volumes[1],
            OdinMorphParam::Osc3Volume => &mut self.osc_volumes[2],
            OdinMorphParam::MasterVolume => &mut self.master_volume,
        }
    }

    /// Bornes du moteur : le filtre reste audible, les enveloppes valides.
    #[must_use]
    pub fn clamp(mut self) -> Self {
        self.filter_frequency = self.filter_frequency.clamp(20.0, 20_000.0);
        self.filter_resonance = self.filter_resonance.clamp(0.0, 1.0);
        self.filter_env_amount = self.filter_env_amount.clamp(-1.0, 1.0);
        self.amp_attack = self.amp_attack.clamp(0.001, 5.0);
        self.amp_decay = self.amp_decay.clamp(0.001, 5.0);
        self.amp_sustain = self.amp_sustain.clamp(0.0, 1.0);
        self.amp_release = self.amp_release.clamp(0.01, 10.0);
        self.filter_attack = self.filter_attack.clamp(0.001, 5.0);
        self.filter_decay = self.filter_decay.clamp(0.001, 5.0);
        self.filter_sustain = self.filter_sustain.clamp(0.0, 1.0);
        self.filter_release = self.filter_release.clamp(0.01, 10.0);
        self.osc_detune = self.osc_detune.clamp(0.0, 1.0);
        for volume in &mut self.osc_volumes {
            *volume = volume.clamp(0.0, 2.0);
        }
        self.master_volume = self.master_volume.clamp(0.0, 2.0);
        self
    }
}

/// Champs discrets du preset (octaves, demi-tons) : repris tels quels du
/// preset, jamais morphés.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OdinDiscreteParams {
    pub octaves: [i32; 3],
    pub semitones: [i32; 3],
}

/// Socle d'un preset fourni par l'hôte, analysé hors callback.
#[derive(Clone, Debug, PartialEq)]
pub struct OdinPresetBase {
    /// Nom court (celui que les profils désignent, `[odin.<Piste>].preset`).
    pub name: String,
    pub base: OdinVoiceParams,
    pub discrete: OdinDiscreteParams,
}

/// Charge `Copy` prête à appliquer pour une piste Odin : le preset morphé
/// par le thread principal. Le callback audio ne calcule rien, il copie.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OdinTrackPatch {
    /// Rang du preset dans la [`OdinBank`] de l'hôte : un changement de rang
    /// coupe les notes de la voix.
    pub preset: usize,
    pub params: OdinVoiceParams,
    pub discrete: OdinDiscreteParams,
}

/// Charges des 6 pistes, indexées par [`TrackId`] (`None` : piste SoundFont).
pub type OdinPatches = [Option<OdinTrackPatch>; TrackId::ALL.len()];

/// Presets fournis par l'hôte, connus du thread principal : socles à partir
/// desquels il prépare les charges du callback.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OdinBank {
    presets: Vec<OdinPresetBase>,
}

impl OdinBank {
    #[must_use]
    pub fn new(presets: Vec<OdinPresetBase>) -> Self {
        Self { presets }
    }

    /// Preset chargé sous ce nom.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.presets.iter().any(|preset| preset.name == name)
    }

    /// Charges de toutes les pistes pour ces émotions : les pistes dont le
    /// profil déclare un preset chargé jouent Odin, les autres `None`
    /// (SoundFont ; l'absence d'un preset déclaré est signalée au chargement
    /// du profil, voir `host_resources`).
    #[must_use]
    pub fn patches(
        &self,
        specs: &std::collections::BTreeMap<TrackId, OdinTrackSpec>,
        emotions: [f32; 4],
    ) -> OdinPatches {
        let mut patches = [None; TrackId::ALL.len()];
        for (track, spec) in specs {
            let found = self.presets.iter().enumerate().find(|(_, p)| p.name == spec.preset);
            if let (Some((index, preset)), Some(slot)) =
                (found, patches.get_mut(usize::from(track.channel())))
            {
                *slot = Some(OdinTrackPatch {
                    preset: index,
                    params: preset.base.morphed(&spec.morph, emotions),
                    discrete: preset.discrete,
                });
            }
        }
        patches
    }
}

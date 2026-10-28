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
//!   { param = "fil1_freq", arousal = 3500.0, tension = 1500.0 },
//!   { param = "fil1_res", tension = 0.35 },
//! ]
//! ```

use serde::{Deserialize, Serialize};

use crate::timeline::TrackId;

/// Nom de preset maximal (octets SF2 exclus, nom court fourni par l'hôte).
pub const ODIN_PRESET_NAME_CAP: usize = 64;
/// Règles de morphing maximales par piste (borne du schéma `[odin.<Piste>]`).
pub const ODIN_MAX_RULES: usize = 16;

// Noms et bornes du catalogue odin-rs 4718189. La table reste indépendante
// de la dépendance GPL et définit aussi l'ordre du tableau transmis au rendu.
macro_rules! morph_params {
    ($($variant:ident => ($name:literal, $min:expr, $max:expr)),+ $(,)?) => {
        /// Paramètre continu du catalogue Odin, dans son unité physique.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[repr(usize)]
        pub enum OdinMorphParam {
            $(#[serde(rename = $name)] $variant),+
        }

        impl OdinMorphParam {
            pub const ALL: [Self; Self::COUNT] = [$(Self::$variant),+];
            pub const COUNT: usize = [$(stringify!($variant)),+].len();

            #[must_use]
            pub const fn index(self) -> usize { self as usize }

            #[must_use]
            pub const fn name(self) -> &'static str {
                match self { $(Self::$variant => $name),+ }
            }

            #[must_use]
            pub const fn bounds(self) -> (f32, f32) {
                match self { $(Self::$variant => ($min, $max)),+ }
            }
        }
    };
}

morph_params! {
    Osc1Vol => ("osc1_vol", -60.0, 12.0),
    Osc2Vol => ("osc2_vol", -60.0, 12.0),
    Osc3Vol => ("osc3_vol", -60.0, 12.0),
    Osc1Detune => ("osc1_detune", 0.0, 1.0),
    Osc2Detune => ("osc2_detune", 0.0, 1.0),
    Osc3Detune => ("osc3_detune", 0.0, 1.0),
    Fil1Freq => ("fil1_freq", 20.0, 20_000.0),
    Fil1Res => ("fil1_res", 0.0, 1.0),
    Fil1Env => ("fil1_env", -1.0, 1.0),
    Fil1Saturation => ("fil1_saturation", 0.0, 1.0),
    Fil1Gain => ("fil1_gain", -60.0, 12.0),
    Fil2Freq => ("fil2_freq", 20.0, 20_000.0),
    Fil2Res => ("fil2_res", 0.0, 1.0),
    Fil2Env => ("fil2_env", -1.0, 1.0),
    Fil2Saturation => ("fil2_saturation", 0.0, 1.0),
    Fil2Gain => ("fil2_gain", -60.0, 12.0),
    Fil3Freq => ("fil3_freq", 20.0, 20_000.0),
    Fil3Res => ("fil3_res", 0.0, 1.0),
    Fil3Env => ("fil3_env", -1.0, 1.0),
    Fil3Saturation => ("fil3_saturation", 0.0, 1.0),
    Fil3Gain => ("fil3_gain", -60.0, 12.0),
    Env1Attack => ("env1_attack", 0.001, 10.0),
    Env1Decay => ("env1_decay", 0.001, 10.0),
    Env1Sustain => ("env1_sustain", 0.0, 1.0),
    Env1Release => ("env1_release", 0.001, 10.0),
    Env2Attack => ("env2_attack", 0.001, 10.0),
    Env2Decay => ("env2_decay", 0.001, 10.0),
    Env2Sustain => ("env2_sustain", 0.0, 1.0),
    Env2Release => ("env2_release", 0.001, 10.0),
    Env3Attack => ("env3_attack", 0.001, 10.0),
    Env3Decay => ("env3_decay", 0.001, 10.0),
    Env3Sustain => ("env3_sustain", 0.0, 1.0),
    Env3Release => ("env3_release", 0.001, 10.0),
    Env4Attack => ("env4_attack", 0.001, 10.0),
    Env4Decay => ("env4_decay", 0.001, 10.0),
    Env4Sustain => ("env4_sustain", 0.0, 1.0),
    Env4Release => ("env4_release", 0.001, 10.0),
    Master => ("master", -50.0, 12.0),
    DistBoost => ("dist_boost", 0.0, 1.0),
    DistDrywet => ("dist_drywet", 0.0, 1.0),
    DelayWet => ("delay_wet", 0.0, 1.0),
    PhaserDrywet => ("phaser_drywet", 0.0, 1.0),
    FlangerDrywet => ("flanger_drywet", 0.0, 1.0),
    ChorusDrywet => ("chorus_drywet", 0.0, 1.0),
    RevDrywet => ("rev_drywet", 0.0, 1.0),
}

/// Une règle de morphing : gains additifs par émotion (émotions `0..1`).
///
/// `valeur = base + arousal × gain + valence × gain + density × gain +
/// tension × gain`, bornée par paramètre (voir [`OdinVoiceParams::morphed`]).
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
/// Tableau fixe indexé par [`OdinMorphParam::index`], sans dépendance GPL.
/// Les valeurs sont celles du catalogue (Hz, secondes, dB ou rapports).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OdinVoiceParams {
    pub values: [f32; OdinMorphParam::COUNT],
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
            out.values[rule.param.index()] += delta;
        }
        out.clamp()
    }

    /// Bornes physiques du catalogue, identiques pour socles et morphing.
    #[must_use]
    pub fn clamp(mut self) -> Self {
        for param in OdinMorphParam::ALL {
            let (min, max) = param.bounds();
            self.values[param.index()] = self.values[param.index()].clamp(min, max);
        }
        self
    }
}

/// Socle d'un preset fourni par l'hôte, analysé hors callback.
#[derive(Clone, Debug, PartialEq)]
pub struct OdinPresetBase {
    /// Nom court (celui que les profils désignent, `[odin.<Piste>].preset`).
    pub name: String,
    pub base: OdinVoiceParams,
}

/// Charge `Copy` prête à appliquer pour une piste Odin : le preset morphé
/// par le thread principal. Le callback audio ne calcule rien, il copie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OdinTrackPatch {
    /// Rang du preset dans la [`OdinBank`] de l'hôte : un changement de rang
    /// coupe les notes de la voix.
    pub preset: usize,
    pub params: OdinVoiceParams,
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
                });
            }
        }
        patches
    }
}

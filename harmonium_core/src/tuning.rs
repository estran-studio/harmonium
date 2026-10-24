//! Profil actif et paramètres des modèles musicaux.
use serde::{Deserialize, Serialize};

use crate::{
    style_profile::StyleProfile,
    trad::{
        accomp_profile::{BassVoice, ChordVoice},
        harmony::ChordDegree,
    },
};

/// Version du contrat : ornements idiomatiques `[ornaments]` (V3b).
pub const TUNING_VERSION: u32 = 16;

/// Paramètres de réalisation de la mélodie du profil.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReelFiddleParams {
    pub weight_role_run: u32,
    pub weight_role_arp: u32,
    pub weight_role_turn: u32,
    pub max_continuity_step_distance: i8,
    pub continuity_max_retries: usize,
    pub melody_center_midi: u8,
    pub tonic_midi_min: u8,
    pub tonic_midi_max: u8,
    pub tessitura_min: u8,
    pub tessitura_max: u8,
    pub base_velocity: u8,
    pub strong_eighth_velocity_offset: i8,
    pub even_eighth_velocity_offset: i8,
}
impl Default for ReelFiddleParams {
    fn default() -> Self {
        StyleProfile::default().melody.params
    }
}

/// Harmonisation, tessitures et motifs d'accompagnement du profil.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TradHarmonyParams {
    pub strong_steps: Vec<usize>,
    pub strong_weight: u32,
    pub weak_weight: u32,
    pub tie_priority: Vec<ChordDegree>,
    pub bass: BassVoice,
    pub chords: ChordVoice,
}
impl Default for TradHarmonyParams {
    fn default() -> Self {
        match StyleProfile::default().accompaniment {
            Some(a) => a,
            None => Self {
                strong_steps: Vec::new(),
                strong_weight: 0,
                weak_weight: 0,
                tie_priority: Vec::new(),
                bass: BassVoice::default(),
                chords: ChordVoice::default(),
            },
        }
    }
}

/// Source unique des données musicales du générateur.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TuningParams {
    #[serde(default)]
    pub profile: StyleProfile,
}

#[derive(Clone, Debug)]
pub struct ValidationError {
    pub field: &'static str,
    pub message: String,
}
impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}
impl TuningParams {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        self.profile
            .validate()
            .map_err(|error| vec![ValidationError { field: "profile", message: error.to_string() }])
    }
}

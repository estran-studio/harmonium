//! Profil actif et paramètres des modèles musicaux.
use serde::{Deserialize, Serialize};

use crate::{style_profile::StyleProfile, trad::harmony::ChordDegree};

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
pub struct TradHarmonyParams {
    pub strong_steps: Vec<usize>,
    pub strong_weight: u32,
    pub weak_weight: u32,
    pub tie_priority: Vec<ChordDegree>,
    pub bass_steps: Vec<usize>,
    pub bass_duration_steps: usize,
    pub bass_velocity: u8,
    pub bass_low: u8,
    pub bass_high: u8,
    pub chop_steps: Vec<usize>,
    pub chop_duration_steps: usize,
    pub chop_velocity: u8,
    pub guitar_center: u8,
    pub guitar_max_drift: u8,
}
impl Default for TradHarmonyParams {
    fn default() -> Self {
        match StyleProfile::default().accompaniment {
            Some(a) => a,
            None => Self {
                strong_steps: vec![0, 4, 8, 12],
                strong_weight: 2,
                weak_weight: 1,
                tie_priority: Vec::new(),
                bass_steps: vec![0, 8],
                bass_duration_steps: 4,
                bass_velocity: 80,
                bass_low: 36,
                bass_high: 59,
                chop_steps: vec![4, 12],
                chop_duration_steps: 2,
                chop_velocity: 70,
                guitar_center: 60,
                guitar_max_drift: 6,
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

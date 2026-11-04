//! Profil actif et paramètres des modèles musicaux.
use serde::{Deserialize, Serialize};

use crate::{
    compose::{
        CellRole,
        accomp_profile::{BassVoice, ChordVoice},
        harmony::ChordDegree,
    },
    style_profile::StyleProfile,
};

/// Version du contrat : R2-jeux (019, 27) active les habillages par graine, arcs et phrases des
/// cinq profils de jeu ; `isaac_cellar` reçoit aussi des boucles générées et pédales alternatives.
/// S0 (26) dérive l'accompagnement de la graine de session ; R1 (25) intègre P1/P2.
pub const TUNING_VERSION: u32 = 27;

/// Paramètres de réalisation de la mélodie du profil.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MelodyParams {
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
impl Default for MelodyParams {
    fn default() -> Self {
        StyleProfile::default().melody.params
    }
}
impl MelodyParams {
    /// Poids des rôles de cellule libres, tirés par `Free`, `CadOpen` et `CadOther`.
    /// Un poids nul retire le rôle du tirage : sa banque n'est pas exigée.
    #[must_use]
    pub const fn free_role_weights(&self) -> [(CellRole, u32); 3] {
        [
            (CellRole::Run, self.weight_role_run),
            (CellRole::Arp, self.weight_role_arp),
            (CellRole::Turn, self.weight_role_turn),
        ]
    }
}

/// Harmonisation, tessitures et motifs d'accompagnement du profil.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccompanimentParams {
    pub strong_steps: Vec<usize>,
    pub strong_weight: u32,
    pub weak_weight: u32,
    pub tie_priority: Vec<ChordDegree>,
    pub bass: BassVoice,
    pub chords: ChordVoice,
}
impl Default for AccompanimentParams {
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

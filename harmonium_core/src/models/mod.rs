//! Modèles sélectionnés par les données du profil, indépendants du driver Legacy.
pub mod jazz;

use crate::{theory::chord::Chord, trad::TuneHarmony};

/// Plan harmonique précalculé sur le thread de génération.
#[derive(Clone, Debug, PartialEq)]
pub enum HarmonyPlan {
    TradFunctional(TuneHarmony),
    JazzFunctional(Vec<Vec<Chord>>),
    None,
}

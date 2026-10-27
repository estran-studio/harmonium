//! Modèles sélectionnés par les données du profil, indépendants du driver Legacy.
pub mod chord_loop;
pub mod jazz;

use crate::{theory::chord::Chord, trad::TuneHarmony};

/// Plan harmonique précalculé sur le thread de génération.
#[derive(Clone, Debug, PartialEq)]
pub enum HarmonyPlan {
    TradFunctional(TuneHarmony),
    Loop(chord_loop::LoopHarmony),
    JazzFunctional(Vec<Vec<Chord>>),
    None,
}
impl HarmonyPlan {
    /// Accords relatifs au mode, partagés par les voix d'accompagnement.
    pub(crate) const fn uses_modal_chords(&self) -> bool {
        matches!(self, Self::TradFunctional(_) | Self::Loop(_))
    }
}

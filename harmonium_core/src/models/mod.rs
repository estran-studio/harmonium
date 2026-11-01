//! Modèles sélectionnés par les données du profil, indépendants du driver Legacy.
pub mod chord_loop;
pub mod rewrite;

use crate::{compose::TuneHarmony, theory::chord::Chord};

/// Plan harmonique précalculé sur le thread de génération.
#[derive(Clone, Debug, PartialEq)]
pub enum HarmonyPlan {
    MelodyFunctional(TuneHarmony),
    Loop(chord_loop::LoopHarmony),
    RewriteFunctional(Vec<Vec<Chord>>),
    None,
}
impl HarmonyPlan {
    /// Accords relatifs au mode, partagés par les voix d'accompagnement.
    pub(crate) const fn uses_modal_chords(&self) -> bool {
        matches!(self, Self::MelodyFunctional(_) | Self::Loop(_))
    }
}

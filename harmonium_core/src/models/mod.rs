//! Modèles sélectionnés par les données du profil, indépendants du driver Legacy.
pub mod chord_loop;
pub(crate) mod loop_generation;
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
    /// Accords relatifs au mode dont les contextes publiés se calculent depuis un
    /// `DegreeChord` (`rewrite_functional` publie les siens, absolus).
    pub(crate) const fn uses_modal_chords(&self) -> bool {
        matches!(self, Self::MelodyFunctional(_) | Self::Loop(_))
    }

    /// Les voix d'accompagnement jouent des accords sous ce plan (tout sauf `none`).
    pub(crate) const fn has_chords(&self) -> bool {
        !matches!(self, Self::None)
    }
}

//! Module Harmonique Unifié - Harmonium
//!
//! Ce module contient toute la logique harmonique:
//! - `basic`: Système `BasicHarmony` (quadrants émotionnels Russell)
//! - `melody`: `HarmonyNavigator` (génération mélodique Markov+Fractal)
//! - `chord`: Représentation enrichie des accords
//! - `lydian_chromatic`: Lydian Chromatic Concept (12 niveaux)
//! - `neo_riemannian`: Transformations P, L, R (triades uniquement)
//! - `parsimonious`: Voice-leading parsimonieux (tous types d'accords)
//! - `steedman_grammar`: Grammaire générative de progressions
//! - `pivot`: Système de transition entre stratégies
//! - `voice_leading`: Optimisation du voice-leading
//! - `driver`: `HarmonicDriver` (orchestrateur principal)

pub mod basic;
pub mod chord;
pub mod driver;
pub mod lydian_chromatic;
pub mod melody;
pub mod neo_riemannian;
pub mod parsimonious;
pub mod pivot;
pub mod steedman_grammar;
pub mod voice_leading;

// Re-exports pour compatibilité
pub use basic::{ChordQuality, ChordStep, Progression};
pub use chord::{Chord, ChordType, PitchClass};
pub use driver::HarmonicDriver;
pub use lydian_chromatic::{LccLevel, LydianChromaticConcept, ScaleGuidance};
pub use melody::HarmonyNavigator;
pub use parsimonious::{Neighbor, ParsimoniousDriver, ParsimoniousTransform, TRQ};
use rand::Rng;

/// Mode d'harmonie sélectionné
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum HarmonyMode {
    /// Système `BasicHarmony` basé sur les quadrants émotionnels (Russell Circumplex)
    #[default]
    Basic,
    /// `HarmonicDriver` avancé (Steedman + Neo-Riemannian + LCC)
    Driver,
    /// Fixed chord chart — cycles through a user-provided chord sequence
    Chart,
}

/// Contexte harmonique passé aux stratégies
#[derive(Clone, Debug)]
pub struct HarmonyContext {
    /// Accord courant
    pub current_chord: Chord,
    /// Tonique globale du morceau (0-11)
    pub global_key: PitchClass,
    /// Tension (0.0-1.0) - contrôle la stratégie (Steedman vs Neo-Riemannian)
    pub tension: f32,
    /// Valence (-1.0 à 1.0) - contrôle majeur/mineur
    pub valence: f32,
    /// Position dans la phrase (mesure)
    pub measure_in_phrase: usize,
    /// Position dans la mesure (beat)
    pub beat_in_measure: usize,
}

impl Default for HarmonyContext {
    fn default() -> Self {
        Self {
            current_chord: Chord::default(),
            global_key: 0,
            tension: 0.5,
            valence: 0.0,
            measure_in_phrase: 0,
            beat_in_measure: 0,
        }
    }
}

/// Résultat d'une décision harmonique
#[derive(Clone, Debug)]
pub struct HarmonyDecision {
    /// Prochain accord
    pub next_chord: Chord,
    /// Type de transition utilisée
    pub transition_type: TransitionType,
    /// Gamme suggérée pour la mélodie (pitch classes)
    pub suggested_scale: Vec<PitchClass>,
}

/// Type de transition entre accords
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionType {
    /// Harmonie fonctionnelle (Steedman): V->I, ii-V, etc.
    Functional,
    /// Transformation géométrique (Neo-Riemannian): P, L, R
    Transformational,
    /// Accord pivot (dim7, aug, sus4) pour transition entre stratégies
    Pivot,
    /// Pas de changement d'accord
    Static,
}

impl TransitionType {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Functional => "Functional",
            Self::Transformational => "Transformational",
            Self::Pivot => "Pivot",
            Self::Static => "Static",
        }
    }
}

/// Trait pour les stratégies de génération harmonique
pub trait HarmonyStrategy: Send + Sync {
    /// Génère le prochain accord basé sur le contexte
    fn next_chord(&self, ctx: &HarmonyContext, rng: &mut dyn RngCore) -> HarmonyDecision;

    /// Nom de la stratégie pour le debug/UI
    fn name(&self) -> &'static str;
}

/// Trait pour RNG compatible avec dyn dispatch
pub trait RngCore {
    fn next_f32(&mut self) -> f32;
    fn next_range_usize(&mut self, range: std::ops::Range<usize>) -> usize;
}

impl<R: Rng> RngCore for R {
    fn next_f32(&mut self) -> f32 {
        self.r#gen::<f32>()
    }

    fn next_range_usize(&mut self, range: std::ops::Range<usize>) -> usize {
        self.gen_range(range)
    }
}

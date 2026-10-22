//! Pivot System - Transition entre stratégies harmoniques
//!
//! Gère la transition entre Steedman Grammar et Neo-Riemannian
//! en utilisant des accords ambigus (dim7, aug, sus4) comme pivots.

use std::sync::Arc;

use super::lydian_chromatic::LydianChromaticConcept;
use crate::theory::chord::{Chord, ChordType};

/// Seuil de tension pour basculer vers Steedman (en dessous)
pub const STEEDMAN_THRESHOLD: f32 = 0.5;

/// Seuil de tension pour basculer vers Neo-Riemannian (au dessus)
pub const NEO_RIEMANNIAN_THRESHOLD: f32 = 0.7;

/// Type de pivot
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PivotType {
    /// Pas un accord pivot
    None,
    /// Accord faiblement ambigu (sus4)
    Weak,
    /// Accord modérément ambigu (demi-diminué)
    Moderate,
    /// Accord fortement ambigu (dim7, augmenté)
    Strong,
}

/// Détecteur et générateur d'accords pivot
pub struct PivotDetector {
    lcc: Arc<LydianChromaticConcept>,
}

impl PivotDetector {
    #[must_use]
    pub const fn new(lcc: Arc<LydianChromaticConcept>) -> Self {
        Self { lcc }
    }

    /// Vérifie si un accord peut servir de pivot
    #[must_use]
    pub const fn is_pivot_chord(&self, chord: &Chord) -> PivotType {
        match chord.chord_type {
            // Accords symétriques: parfaits pour les pivots
            ChordType::Diminished7 => PivotType::Strong,
            ChordType::Augmented => PivotType::Strong,

            // Sus4: pas de tierce, ambigu
            ChordType::Sus4 => PivotType::Weak,
            ChordType::Sus2 => PivotType::Weak,

            // Demi-diminué: peut fonctionner
            ChordType::HalfDiminished => PivotType::Moderate,

            // Diminué simple
            ChordType::Diminished => PivotType::Moderate,

            // Accords clairs: pas des pivots
            _ => PivotType::None,
        }
    }

    /// Calcule les poids de crossfade entre les deux stratégies
    /// Retourne (`steedman_weight`, `neo_riemannian_weight`)
    #[must_use]
    pub fn crossfade_weight(&self, tension: f32) -> (f32, f32) {
        if tension < STEEDMAN_THRESHOLD {
            // Zone Steedman pure
            (1.0, 0.0)
        } else if tension > NEO_RIEMANNIAN_THRESHOLD {
            // Zone Neo-Riemannian pure
            (0.0, 1.0)
        } else {
            // Zone de transition: crossfade linéaire
            let t =
                (tension - STEEDMAN_THRESHOLD) / (NEO_RIEMANNIAN_THRESHOLD - STEEDMAN_THRESHOLD);
            (1.0 - t, t)
        }
    }

    /// Calcule les poids de crossfade à trois voies avec support d'hystérésis
    /// Retourne (`steedman_weight`, `parsimonious_weight`, `neo_riemannian_weight`)
    ///
    /// Cette méthode utilise des zones d'hystérésis pour éviter les basculements chaotiques
    /// entre les stratégies lorsque la tension fluctue autour des seuils.
    ///
    /// # Arguments
    /// * `tension` - La tension actuelle (0.0 à 1.0)
    /// * `steedman_lower` - Seuil inférieur pour Steedman (ex: 0.45)
    /// * `steedman_upper` - Seuil supérieur pour Steedman (ex: 0.55)
    /// * `neo_lower` - Seuil inférieur pour Neo-Riemannian (ex: 0.65)
    /// * `neo_upper` - Seuil supérieur pour Neo-Riemannian (ex: 0.75)
    ///
    /// # Zones
    /// 1. Pure Steedman: tension ≤ `steedman_lower`
    /// 2. Hystérésis Steedman: `steedman_lower` < tension < `steedman_upper`
    /// 3. Pure Parsimonious: `steedman_upper` ≤ tension ≤ `neo_lower`
    /// 4. Hystérésis Neo-Riemannian: `neo_lower` < tension < `neo_upper`
    /// 5. Pure Neo-Riemannian: tension ≥ `neo_upper`
    #[must_use]
    pub fn crossfade_weight_three_hysteresis(
        &self,
        tension: f32,
        steedman_lower: f32,
        steedman_upper: f32,
        neo_lower: f32,
        neo_upper: f32,
    ) -> (f32, f32, f32) {
        if tension <= steedman_lower {
            // Zone Steedman pure
            (1.0, 0.0, 0.0)
        } else if tension < steedman_upper {
            // Zone d'hystérésis Steedman: fade Steedman → Parsimonious
            let t = (tension - steedman_lower) / (steedman_upper - steedman_lower);
            (1.0 - t, t, 0.0)
        } else if tension <= neo_lower {
            // Zone Parsimonious pure
            (0.0, 1.0, 0.0)
        } else if tension < neo_upper {
            // Zone d'hystérésis Neo-Riemannian: fade Parsimonious → Neo-Riemannian
            let t = (tension - neo_lower) / (neo_upper - neo_lower);
            (0.0, 1.0 - t, t)
        } else {
            // Zone Neo-Riemannian pure
            (0.0, 0.0, 1.0)
        }
    }

    /// Vérifie si on est dans la zone de transition
    #[must_use]
    pub fn is_in_transition_zone(&self, tension: f32) -> bool {
        (STEEDMAN_THRESHOLD..=NEO_RIEMANNIAN_THRESHOLD).contains(&tension)
    }

    /// Génère un accord pivot approprié pour la transition
    #[must_use]
    pub fn create_pivot(&self, from: &Chord, to: &Chord, tension: f32) -> Chord {
        // Si la tension monte (vers Neo-Riemannian), utiliser dim7 ou aug
        // Si la tension descend (vers Steedman), utiliser sus4

        if tension > 0.6 {
            // Haute tension: accord symétrique
            // Diminué 7 basé sur la fondamentale de "from"
            Chord::new(from.root, ChordType::Diminished7)
        } else if tension > 0.5 {
            // Tension moyenne: augmenté
            Chord::new(from.root, ChordType::Augmented)
        } else {
            // Basse tension: sus4 (neutre, préparation à la résolution)
            Chord::new(to.root, ChordType::Sus4)
        }
    }

    /// Suggère le meilleur pivot entre deux accords
    #[must_use]
    pub fn suggest_pivot(&self, from: &Chord, to: &Chord) -> Option<Chord> {
        // Calculer la distance
        let distance = from.voice_leading_distance(to);

        // Si la distance est petite, pas besoin de pivot
        if distance <= 2 {
            return None;
        }

        // Trouver une note commune ou proche
        let from_pcs = from.pitch_classes();
        let to_pcs = to.pitch_classes();

        // Chercher une intersection
        for &from_pc in &from_pcs {
            if to_pcs.contains(&from_pc) {
                // Note commune: construire un pivot dessus
                // Dim7 si les accords sont éloignés, sus4 sinon
                if distance > 4 {
                    return Some(Chord::new(from_pc, ChordType::Diminished7));
                }
                return Some(Chord::new(from_pc, ChordType::Sus4));
            }
        }

        // Pas de note commune: utiliser la moyenne des fondamentales
        let avg_root = i32::midpoint(i32::from(from.root), i32::from(to.root)) as u8 % 12;
        Some(Chord::new(avg_root, ChordType::Augmented))
    }

    /// Retourne la gamme LCC suggérée pour un accord pivot
    #[must_use]
    pub fn get_pivot_scale(&self, pivot: &Chord, tension: f32) -> Vec<crate::theory::PitchClass> {
        let parent = self.lcc.parent_lydian(pivot);
        let level = self.lcc.level_for_tension(tension);
        self.lcc.get_scale(parent, level)
    }

    /// Détermine quelle stratégie utiliser basée sur la tension
    #[must_use]
    pub fn preferred_strategy(&self, tension: f32) -> PreferredStrategy {
        if tension < STEEDMAN_THRESHOLD {
            PreferredStrategy::Steedman
        } else if tension > NEO_RIEMANNIAN_THRESHOLD {
            PreferredStrategy::NeoRiemannian
        } else {
            let (s, n) = self.crossfade_weight(tension);
            if s > n {
                PreferredStrategy::TransitionToSteedman
            } else {
                PreferredStrategy::TransitionToNeoRiemannian
            }
        }
    }
}

/// Stratégie préférée
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferredStrategy {
    Steedman,
    NeoRiemannian,
    TransitionToSteedman,
    TransitionToNeoRiemannian,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_detector() -> PivotDetector {
        PivotDetector::new(Arc::new(LydianChromaticConcept::new()))
    }

    #[test]
    fn test_pivot_detection() {
        let detector = make_detector();

        // Dim7 = Strong
        assert_eq!(
            detector.is_pivot_chord(&Chord::new(0, ChordType::Diminished7)),
            PivotType::Strong
        );

        // Augmented = Strong
        assert_eq!(
            detector.is_pivot_chord(&Chord::new(0, ChordType::Augmented)),
            PivotType::Strong
        );

        // Sus4 = Weak
        assert_eq!(detector.is_pivot_chord(&Chord::new(0, ChordType::Sus4)), PivotType::Weak);

        // Major = None
        assert_eq!(detector.is_pivot_chord(&Chord::new(0, ChordType::Major)), PivotType::None);
    }

    #[test]
    fn test_crossfade_weights() {
        let detector = make_detector();

        // Basse tension: 100% Steedman
        let (s, n) = detector.crossfade_weight(0.3);
        assert!((s - 1.0).abs() < f32::EPSILON);
        assert!(n.abs() < f32::EPSILON);

        // Haute tension: 100% Neo-Riemannian
        let (s, n) = detector.crossfade_weight(0.9);
        assert!(s.abs() < f32::EPSILON);
        assert!((n - 1.0).abs() < f32::EPSILON);

        // Milieu: mélange
        let (s, n) = detector.crossfade_weight(0.6);
        assert!(s > 0.0 && s < 1.0);
        assert!(n > 0.0 && n < 1.0);
        assert!((s + n - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_transition_zone() {
        let detector = make_detector();

        assert!(!detector.is_in_transition_zone(0.3));
        assert!(detector.is_in_transition_zone(0.6));
        assert!(!detector.is_in_transition_zone(0.9));
    }
}

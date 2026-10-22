//! Module `BasicHarmony` - Progressions harmoniques basées sur le modèle Circumplex de Russell
//!
//! Système de sélection de progressions d'accords basé sur les quadrants émotionnels:
//! - Valence (positif/négatif) × Tension (calme/tendu)

use crate::{theory::chord::ChordType, tuning::EmotionalQuadrantParams};

impl crate::theory::chord::Chord {
    /// Convertit vers le `ChordQuality` de l'ancien système (pour compatibilité)
    #[must_use]
    pub const fn to_basic_quality(&self) -> super::basic::ChordQuality {
        match self.chord_type {
            ChordType::Major
            | ChordType::Major7
            | ChordType::Augmented
            | ChordType::Major6
            | ChordType::Augmented7
            | ChordType::Add9 => super::basic::ChordQuality::Major,
            ChordType::Minor
            | ChordType::Minor7
            | ChordType::HalfDiminished
            | ChordType::MinorMajor7
            | ChordType::Minor6 => super::basic::ChordQuality::Minor,
            ChordType::Diminished | ChordType::Diminished7 => {
                super::basic::ChordQuality::Diminished
            }
            ChordType::Dominant7 | ChordType::Dominant7Sus4 => {
                super::basic::ChordQuality::Dominant7
            }
            ChordType::Sus2 | ChordType::Sus4 => super::basic::ChordQuality::Sus2,
        }
    }
}

/// Qualité d'accord (pour compatibilité avec l'ancien système)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Dominant7,
    Sus2, // Pour neutralité/ambiance
}

/// Étape d'accord dans une progression
#[derive(Clone, Copy, Debug)]
pub struct ChordStep {
    pub root_offset: i32, // Décalage en demi-tons depuis tonique (0=I, 7=V, etc.)
    pub quality: ChordQuality,
}

/// Progression d'accords avec nom
pub struct Progression {
    pub steps: Vec<ChordStep>,
    pub name: &'static str,
}

impl Progression {
    /// Sélectionne la palette harmonique selon l'état émotionnel
    /// Basé sur Russell's Circumplex Model (Valence × Arousal)
    #[must_use]
    pub fn get_palette(valence: f32, tension: f32, eq: &EmotionalQuadrantParams) -> Vec<ChordStep> {
        // === QUADRANT 1: HEUREUX & ÉNERGIQUE (Valence > 0, Tension > 0.5) ===
        if valence > eq.happy_valence_threshold && tension > eq.energetic_tension_threshold {
            // Progression pop énergique: I - V - vi - IV
            // (Journey, U2, Red Hot Chili Peppers)
            vec![
                ChordStep { root_offset: 0, quality: ChordQuality::Major }, // I
                ChordStep { root_offset: 7, quality: ChordQuality::Dominant7 }, // V7 (tension)
                ChordStep { root_offset: 9, quality: ChordQuality::Minor }, // vi (contraste)
                ChordStep { root_offset: 5, quality: ChordQuality::Major }, // IV
            ]
        }
        // === QUADRANT 2: HEUREUX & CALME (Valence > 0, Tension < 0.5) ===
        else if valence > eq.happy_valence_threshold {
            // Progression paisible: I - IV - I - V
            // (Folk, ballades apaisantes)
            vec![
                ChordStep { root_offset: 0, quality: ChordQuality::Major }, // I (repos)
                ChordStep { root_offset: 5, quality: ChordQuality::Major }, // IV (préparation douce)
                ChordStep { root_offset: 0, quality: ChordQuality::Major }, // I (retour)
                ChordStep { root_offset: 7, quality: ChordQuality::Major }, // V (résolution douce)
            ]
        }
        // === QUADRANT 3: TRISTE & TENDU (Valence < 0, Tension > 0.6) ===
        else if valence < eq.sad_valence_threshold && tension > eq.energetic_tension_threshold {
            // Progression dramatique: i - V7 - VI - vii°
            // (Film noir, suspense, tragédie)
            vec![
                ChordStep { root_offset: 0, quality: ChordQuality::Minor }, // i (sombre)
                ChordStep { root_offset: 7, quality: ChordQuality::Dominant7 }, // V7 (tension forte)
                ChordStep { root_offset: 8, quality: ChordQuality::Major },     // VI (échappatoire)
                ChordStep { root_offset: 11, quality: ChordQuality::Diminished }, // vii° (instabilité)
            ]
        }
        // === QUADRANT 4: TRISTE & CALME (Valence < 0, Tension < 0.5) ===
        else if valence < eq.sad_valence_threshold {
            // Progression mélancolique: i - III - VII - i
            // (Post-rock, ambient mélancolique)
            vec![
                ChordStep { root_offset: 0, quality: ChordQuality::Minor }, // i (mélancolie)
                ChordStep { root_offset: 3, quality: ChordQuality::Major }, // III (lumière douce)
                ChordStep { root_offset: 10, quality: ChordQuality::Major }, // VII (sous-tonique)
                ChordStep { root_offset: 0, quality: ChordQuality::Minor }, // i (retour)
            ]
        }
        // === CENTRE: NEUTRE/AMBIENT (Valence ≈ 0) ===
        else if tension > eq.energetic_tension_threshold {
            // Progression modale tendue: i - iv - i - v
            // (Musique modale, Dorien, ambiguïté harmonique)
            vec![
                ChordStep { root_offset: 0, quality: ChordQuality::Minor }, // i
                ChordStep { root_offset: 5, quality: ChordQuality::Minor }, // iv (sous-dominante mineure)
                ChordStep { root_offset: 0, quality: ChordQuality::Sus2 },  // I sus2 (ambiguïté)
                ChordStep { root_offset: 7, quality: ChordQuality::Minor }, // v (dominante mineure)
            ]
        } else {
            // Progression drone/ambient: i - iv (minimaliste)
            // (Brian Eno, ambient minimale)
            vec![
                ChordStep { root_offset: 0, quality: ChordQuality::Minor }, // i (statique)
                ChordStep { root_offset: 5, quality: ChordQuality::Minor }, // iv (mouvement minimal)
            ]
        }
    }

    /// Retourne le nom de la progression basé sur le contexte émotionnel
    #[must_use]
    pub fn get_progression_name(
        valence: f32,
        tension: f32,
        eq: &EmotionalQuadrantParams,
    ) -> &'static str {
        if valence > eq.happy_valence_threshold && tension > eq.energetic_tension_threshold {
            "Pop Energetic (I-V-vi-IV)"
        } else if valence > eq.happy_valence_threshold {
            "Folk Peaceful (I-IV-I-V)"
        } else if valence < eq.sad_valence_threshold && tension > eq.energetic_tension_threshold {
            "Dramatic Minor (i-V7-VI-vii°)"
        } else if valence < eq.sad_valence_threshold {
            "Melancholic (i-III-VII-i)"
        } else if tension > eq.energetic_tension_threshold {
            "Modal Tense (i-iv-Isus2-v)"
        } else {
            "Ambient Drone (i-iv)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_eq() -> EmotionalQuadrantParams {
        EmotionalQuadrantParams::default()
    }

    #[test]
    fn test_happy_energetic_quadrant() {
        let palette = Progression::get_palette(0.7, 0.8, &default_eq());
        assert_eq!(palette.len(), 4);
        assert_eq!(palette[0].quality, ChordQuality::Major);
        assert_eq!(palette[1].quality, ChordQuality::Dominant7);
    }

    #[test]
    fn test_sad_calm_quadrant() {
        let palette = Progression::get_palette(-0.6, 0.3, &default_eq());
        assert_eq!(palette.len(), 4);
        assert_eq!(palette[0].quality, ChordQuality::Minor);
    }

    #[test]
    fn test_neutral_ambient() {
        let palette = Progression::get_palette(0.0, 0.2, &default_eq());
        assert_eq!(palette.len(), 2); // Drone minimal
    }
}

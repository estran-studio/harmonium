//! Voice Leading - Optimisation du mouvement des voix
//!
//! Algorithmes pour minimiser le mouvement mélodique entre accords
//! et assurer des transitions lisses.

use super::{PitchClass, chord::Chord};

/// Calcule la distance minimale entre deux pitch classes sur le cercle chromatique
#[must_use]
pub fn pitch_class_distance(a: PitchClass, b: PitchClass) -> u8 {
    let d1 = (i16::from(b) - i16::from(a)).unsigned_abs() as u8;
    let d2 = 12 - d1;
    d1.min(d2)
}

/// Calcule la distance totale de voice-leading entre deux accords
/// (somme des mouvements minimaux pour chaque voix)
#[must_use]
pub fn total_voice_leading_distance(from: &Chord, to: &Chord) -> u32 {
    from.voice_leading_distance(to)
}

/// Trouve l'assignation optimale des voix pour minimiser le mouvement
#[must_use]
pub fn optimal_voice_assignment(from: &[PitchClass], to: &[PitchClass]) -> Vec<(usize, usize)> {
    let n = from.len().min(to.len());
    let mut assignments = Vec::with_capacity(n);
    let mut used_to = vec![false; to.len()];

    // Algorithme glouton: pour chaque note de "from",
    // trouver la note la plus proche dans "to"
    for (i, &from_pc) in from.iter().enumerate() {
        let mut min_dist = 12u8;
        let mut min_idx = 0;

        for (j, &to_pc) in to.iter().enumerate() {
            if used_to[j] {
                continue;
            }
            let dist = pitch_class_distance(from_pc, to_pc);
            if dist < min_dist {
                min_dist = dist;
                min_idx = j;
            }
        }

        if min_dist < 12 && min_idx < to.len() {
            used_to[min_idx] = true;
            assignments.push((i, min_idx));
        }
    }

    assignments
}

/// Évalue la qualité d'une transition harmonique
/// Score bas = bonne transition, score haut = transition difficile
#[must_use]
pub fn transition_quality_score(from: &Chord, to: &Chord) -> f32 {
    let base_distance = from.voice_leading_distance(to) as f32;

    // Pénalités supplémentaires
    let mut penalty = 0.0;

    // Pénalité pour mouvement de triton (instable)
    let from_pcs = from.pitch_classes();
    let to_pcs = to.pitch_classes();
    for &from_pc in &from_pcs {
        for &to_pc in &to_pcs {
            if pitch_class_distance(from_pc, to_pc) == 6 {
                penalty += 0.5;
            }
        }
    }

    // Pénalité pour changement de mode (majeur -> mineur ou vice versa)
    if from.chord_type.is_major() != to.chord_type.is_major() {
        penalty += 0.3;
    }

    // Bonus pour note commune
    let common_notes = from_pcs.iter().filter(|pc| to_pcs.contains(pc)).count();
    let bonus = common_notes as f32 * 0.5;

    (base_distance + penalty - bonus).max(0.0)
}

/// Suggère un accord intermédiaire pour améliorer une transition difficile
#[must_use]
pub fn suggest_passing_chord(from: &Chord, to: &Chord) -> Option<Chord> {
    let score = transition_quality_score(from, to);

    // Si la transition est déjà bonne, pas besoin de passing chord
    if score < 2.0 {
        return None;
    }

    // Chercher une note commune pour construire le passing chord
    let from_pcs = from.pitch_classes();
    let to_pcs = to.pitch_classes();

    for &pc in &from_pcs {
        if to_pcs.contains(&pc) {
            // Construire un accord sus4 sur cette note
            return Some(Chord::new(pc, crate::theory::chord::ChordType::Sus4));
        }
    }

    // Pas de note commune: utiliser un accord diminué sur la moyenne
    let avg_root = u16::midpoint(u16::from(from.root), u16::from(to.root)) as u8 % 12;
    Some(Chord::new(avg_root, crate::theory::chord::ChordType::Diminished))
}

/// Calcule le "smoothness" d'une progression (moyenne des distances)
#[must_use]
pub fn progression_smoothness(chords: &[Chord]) -> f32 {
    if chords.len() < 2 {
        return 1.0;
    }

    let total_distance: u32 = chords.windows(2).map(|w| w[0].voice_leading_distance(&w[1])).sum();

    let avg = total_distance as f32 / (chords.len() - 1) as f32;

    // Convertir en score de 0 (très lisse) à 1 (très disjoint)
    // Distance de 1-2 = très lisse, distance > 6 = disjoint
    (avg / 6.0).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theory::chord::ChordType;

    #[test]
    fn test_pitch_class_distance() {
        // C à D = 2 demi-tons
        assert_eq!(pitch_class_distance(0, 2), 2);

        // C à B = 1 demi-ton (en passant par le haut)
        assert_eq!(pitch_class_distance(0, 11), 1);

        // F à B = 6 demi-tons (triton)
        assert_eq!(pitch_class_distance(5, 11), 6);
    }

    #[test]
    fn test_optimal_assignment() {
        // C Major (C, E, G) -> A Minor (A, C, E)
        let from = vec![0, 4, 7]; // C, E, G
        let to = vec![9, 0, 4]; // A, C, E

        let assignments = optimal_voice_assignment(&from, &to);
        assert_eq!(assignments.len(), 3);

        // C devrait aller vers C (distance 0)
        // E devrait aller vers E (distance 0)
        // G devrait aller vers A (distance 2)
    }

    #[test]
    fn test_transition_quality() {
        // C Major -> A Minor (relative): très lisse
        let c_maj = Chord::new(0, ChordType::Major);
        let a_min = Chord::new(9, ChordType::Minor);
        let score1 = transition_quality_score(&c_maj, &a_min);

        // C Major -> F# Major: plus difficile
        let fs_maj = Chord::new(6, ChordType::Major);
        let score2 = transition_quality_score(&c_maj, &fs_maj);

        assert!(score1 < score2);
    }

    #[test]
    fn test_progression_smoothness() {
        // Progression très lisse: I - vi - IV - V (en C)
        let smooth = vec![
            Chord::new(0, ChordType::Major), // C
            Chord::new(9, ChordType::Minor), // Am
            Chord::new(5, ChordType::Major), // F
            Chord::new(7, ChordType::Major), // G
        ];

        // Progression chaotique: sauts chromatiques
        let chaotic = vec![
            Chord::new(0, ChordType::Major), // C
            Chord::new(6, ChordType::Major), // F#
            Chord::new(1, ChordType::Minor), // C#m
            Chord::new(8, ChordType::Major), // G#
        ];

        let smooth_score = progression_smoothness(&smooth);
        let chaotic_score = progression_smoothness(&chaotic);

        assert!(smooth_score < chaotic_score);
    }
}

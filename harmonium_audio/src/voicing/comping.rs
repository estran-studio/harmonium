//! Pattern de comping (accompagnement rythmique)
//!
//! Utilise l'algorithme euclidien pour créer des patterns de jeu syncopés
//! plutôt que de jouer sur chaque beat.

use harmonium_core::algo::euclidean::generate_euclidean_bools;

/// Pattern de comping basé sur l'algorithme euclidien
#[derive(Clone, Debug)]
pub struct CompingPattern {
    /// Pattern booléen: true = jouer, false = silence
    pattern: Vec<bool>,
    /// Nombre total de steps
    steps: usize,
    /// Densité actuelle (pour recalcul)
    density: f32,
}

impl Default for CompingPattern {
    fn default() -> Self {
        Self::euclidean(8, 0.5)
    }
}

impl CompingPattern {
    /// Crée un pattern de comping euclidien
    ///
    /// # Arguments
    /// * `steps` - Nombre de steps dans le pattern (8, 16, etc.)
    /// * `density` - Densité du pattern (0.0 = sparse, 1.0 = dense)
    #[must_use]
    pub fn euclidean(steps: usize, density: f32) -> Self {
        let density = density.clamp(0.0, 1.0);

        // Calculer le nombre de pulses basé sur la densité
        // Minimum 1 pulse, maximum steps-1 (laisser au moins un silence)
        let pulses = ((density * (steps as f32 - 1.0)).round() as usize).max(1);

        let pattern = generate_euclidean_bools(steps, pulses);

        Self { pattern, steps, density }
    }

    /// Vérifie si le step donné est actif (doit jouer)
    #[must_use]
    pub fn is_active(&self, step: usize) -> bool {
        if self.pattern.is_empty() {
            return true; // Fallback: toujours jouer
        }
        self.pattern[step % self.steps]
    }

    /// Met à jour le pattern avec une nouvelle densité
    pub fn update_density(&mut self, new_density: f32) {
        if (new_density - self.density).abs() > 0.05 {
            *self = Self::euclidean(self.steps, new_density);
        }
    }

    /// Retourne le nombre de pulses actifs
    #[must_use]
    pub fn pulse_count(&self) -> usize {
        self.pattern.iter().filter(|&&b| b).count()
    }

    /// Retourne le pattern complet (pour debug/visualisation)
    #[must_use]
    pub fn pattern(&self) -> &[bool] {
        &self.pattern
    }

    /// Retourne le nombre de steps
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_pattern() {
        // E(3, 8) classique: X..X..X.
        let pattern = CompingPattern::euclidean(8, 3.0 / 7.0);

        assert_eq!(pattern.steps(), 8);
        assert_eq!(pattern.pulse_count(), 3);
    }

    #[test]
    fn test_density_extremes() {
        // Densité minimale
        let sparse = CompingPattern::euclidean(8, 0.0);
        assert_eq!(sparse.pulse_count(), 1);

        // Densité maximale
        let dense = CompingPattern::euclidean(8, 1.0);
        assert_eq!(dense.pulse_count(), 7); // steps - 1
    }

    #[test]
    fn test_is_active_wraps() {
        let pattern = CompingPattern::euclidean(4, 0.5);

        // Vérifie que ça wrap correctement
        let step_4_active = pattern.is_active(4);
        let step_0_active = pattern.is_active(0);
        assert_eq!(step_4_active, step_0_active);
    }
}

//! Shell Voicer (Guide Tones Style)
//!
//! Style minimaliste utilisé en Be-Bop où la main gauche joue
//! uniquement la tierce et la septième (guide tones) de l'accord.
//! Cela laisse plus d'espace pour la mélodie et la ligne de basse.

use super::{
    comping::CompingPattern,
    voicer::{VoicedNote, Voicer, VoicerContext, get_guide_tones},
};

/// Voicer style "Shell" (Guide Tones)
///
/// Style minimaliste: mélodie + tierce + septième.
/// Populaire en jazz Be-Bop car il laisse de l'espace pour
/// les lignes mélodiques complexes.
pub struct ShellVoicer {
    /// Pattern de comping euclidien
    comping: CompingPattern,
    /// Durée des notes (en steps) - plus long que block chords
    note_duration: u8,
}

impl Default for ShellVoicer {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellVoicer {
    /// Crée un nouveau `ShellVoicer`
    #[must_use]
    pub fn new() -> Self {
        Self {
            comping: CompingPattern::euclidean(8, 0.4), // Plus sparse par défaut
            note_duration: 4,                           // Notes plus longues
        }
    }

    /// Définit la durée des notes
    pub fn set_note_duration(&mut self, duration: u8) {
        self.note_duration = duration.max(1);
    }
}

impl Voicer for ShellVoicer {
    fn name(&self) -> &'static str {
        "Shell Voicings"
    }

    fn process_note(
        &mut self,
        melody_note: u8,
        base_velocity: u8,
        ctx: &VoicerContext,
    ) -> Vec<VoicedNote> {
        let mut notes = Vec::with_capacity(3);

        // 1. La mélodie
        notes.push(VoicedNote::new(melody_note, base_velocity, self.note_duration));

        // 2. Guide tones (tierce et septième)
        let (third, seventh) = get_guide_tones(ctx.chord_root_midi, ctx.chord_type, melody_note);

        // Tierce avec vélocité réduite (accompagnement)
        notes.push(VoicedNote::new(third, base_velocity.saturating_sub(15), self.note_duration));

        // Septième avec vélocité encore plus réduite
        notes.push(VoicedNote::new(seventh, base_velocity.saturating_sub(20), self.note_duration));

        notes
    }

    fn on_step(&mut self, _ctx: &VoicerContext) {
        // Pas d'état à mettre à jour
    }

    fn should_voice(&self, ctx: &VoicerContext) -> bool {
        self.comping.is_active(ctx.current_step)
    }

    fn on_density_change(&mut self, new_density: f32, steps: usize) {
        // Shell voicings sont naturellement plus sparse
        let adjusted_density = new_density * 0.7;
        self.comping = CompingPattern::euclidean(steps, adjusted_density);
    }
}

#[cfg(test)]
mod tests {
    use harmonium_core::theory::chord::ChordType;

    use super::*;

    #[test]
    fn test_shell_voicing() {
        let mut voicer = ShellVoicer::new();

        let ctx = VoicerContext {
            chord_root_midi: 60, // C4
            chord_type: ChordType::Major7,
            lcc_scale: vec![0, 2, 4, 6, 7, 9, 11],
            tension: 0.3,
            density: 0.5,
            current_step: 0,
            total_steps: 16,
        };

        // G5 comme mélodie
        let voiced = voicer.process_note(79, 100, &ctx);

        // Shell = 3 notes: mélodie, tierce, septième
        assert_eq!(voiced.len(), 3);

        // Mélodie en premier
        assert_eq!(voiced[0].midi, 79);

        // Tierce (E) et septième (B) de Cmaj7
        let third_pc = voiced[1].midi % 12;
        let seventh_pc = voiced[2].midi % 12;

        assert_eq!(third_pc, 4); // E
        assert_eq!(seventh_pc, 11); // B
    }

    #[test]
    fn test_shell_velocities() {
        let mut voicer = ShellVoicer::new();
        let ctx = VoicerContext::default();

        let voiced = voicer.process_note(72, 100, &ctx);

        // Vélocités décroissantes
        assert!(voiced[0].velocity > voiced[1].velocity);
        assert!(voiced[1].velocity > voiced[2].velocity);
    }
}

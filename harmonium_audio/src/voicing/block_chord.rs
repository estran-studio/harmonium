//! Block Chord Voicer (Locked Hands / George Shearing Style)
//!
//! Harmonise la mélodie avec des accords "bloqués" où la mélodie
//! est la note supérieure et les harmonies sont empilées en dessous.

use super::{
    comping::CompingPattern,
    voicer::{VoicedNote, Voicer, VoicerContext, find_scale_notes_below},
};

/// Voicer style "Block Chords" (Locked Hands)
///
/// Technique popularisée par George Shearing et Milt Buckner.
/// La mélodie est doublée à l'octave inférieure avec des harmonies
/// de la gamme LCC entre les deux.
pub struct BlockChordVoicer {
    /// Pattern de comping euclidien
    comping: CompingPattern,
    /// Nombre de voix (3-5)
    num_voices: usize,
    /// Durée des notes (en steps)
    note_duration: u8,
}

impl Default for BlockChordVoicer {
    fn default() -> Self {
        Self::new(4)
    }
}

impl BlockChordVoicer {
    /// Crée un nouveau `BlockChordVoicer`
    ///
    /// # Arguments
    /// * `num_voices` - Nombre de voix (3-5 recommandé)
    #[must_use]
    pub fn new(num_voices: usize) -> Self {
        Self {
            // Pattern jazz: ~5-6 accords sur 16 steps, reste = mélodie seule
            comping: CompingPattern::euclidean(16, 0.4),
            num_voices: num_voices.clamp(3, 5),
            note_duration: 2, // Croches (le legato vient du NoteOff au step suivant)
        }
    }

    /// Définit le nombre de voix
    pub fn set_num_voices(&mut self, num: usize) {
        self.num_voices = num.clamp(3, 5);
    }

    /// Définit la durée des notes
    pub fn set_note_duration(&mut self, duration: u8) {
        self.note_duration = duration.max(1);
    }
}

impl Voicer for BlockChordVoicer {
    fn name(&self) -> &'static str {
        "Block Chords"
    }

    fn process_note(
        &mut self,
        melody_note: u8,
        base_velocity: u8,
        ctx: &VoicerContext,
    ) -> Vec<VoicedNote> {
        let mut notes = Vec::with_capacity(self.num_voices);

        // 1. La mélodie est toujours la note supérieure
        notes.push(VoicedNote::new(melody_note, base_velocity, self.note_duration));

        // 2. Trouver les notes de la gamme LCC sous la mélodie
        let harmony_notes =
            find_scale_notes_below(melody_note, &ctx.lcc_scale, self.num_voices - 1);

        // 3. Ajouter les notes d'harmonie avec vélocité légèrement réduite
        for (i, note) in harmony_notes.into_iter().enumerate() {
            // Vélocité décroissante pour les notes plus graves
            let vel = base_velocity.saturating_sub(5 + (i as u8 * 3));
            notes.push(VoicedNote::new(note, vel.max(40), self.note_duration));
        }

        notes
    }

    fn on_step(&mut self, _ctx: &VoicerContext) {
        // Pas d'état à mettre à jour pour le block chord basique
    }

    fn should_voice(&self, ctx: &VoicerContext) -> bool {
        self.comping.is_active(ctx.current_step)
    }

    fn on_density_change(&mut self, new_density: f32, steps: usize) {
        self.comping = CompingPattern::euclidean(steps, new_density);
    }
}

#[cfg(test)]
mod tests {
    use harmonium_core::theory::chord::ChordType;

    use super::*;

    #[test]
    fn test_block_chord_voicing() {
        let mut voicer = BlockChordVoicer::new(4);

        let ctx = VoicerContext {
            chord_root_midi: 60, // C4
            chord_type: ChordType::Major7,
            lcc_scale: vec![0, 2, 4, 6, 7, 9, 11], // C Lydian
            tension: 0.3,
            density: 0.5,
            current_step: 0,
            total_steps: 16,
        };

        // G5 (MIDI 79) comme note mélodique
        let voiced = voicer.process_note(79, 100, &ctx);

        // Devrait avoir 4 notes
        assert_eq!(voiced.len(), 4);

        // La première note est la mélodie
        assert_eq!(voiced[0].midi, 79);

        // Les autres notes sont sous la mélodie
        for note in voiced.iter().skip(1) {
            assert!(note.midi < 79);
        }
    }
}

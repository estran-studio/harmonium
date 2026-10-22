//! Trait Voicer - Abstraction pour les stratégies de voicing
//!
//! Permet de swapper dynamiquement entre différents styles de voicing
//! (Block Chords, Shell, Drop-2, etc.) selon le genre musical.

use harmonium_core::theory::chord::ChordType;

/// Contexte partagé pour tous les voicers
#[derive(Clone, Debug)]
pub struct VoicerContext {
    /// Root de l'accord courant (note MIDI, pas pitch class)
    pub chord_root_midi: u8,
    /// Type de l'accord courant
    pub chord_type: ChordType,
    /// Gamme LCC active (pitch classes 0-11)
    pub lcc_scale: Vec<u8>,
    /// Tension actuelle (0.0 - 1.0)
    pub tension: f32,
    /// Densité actuelle (0.0 - 1.0)
    pub density: f32,
    /// Step courant dans le séquenceur
    pub current_step: usize,
    /// Nombre total de steps
    pub total_steps: usize,
}

impl Default for VoicerContext {
    fn default() -> Self {
        Self {
            chord_root_midi: 60, // C4
            chord_type: ChordType::Major,
            lcc_scale: vec![0, 2, 4, 6, 7, 9, 11], // C Lydian par défaut
            tension: 0.3,
            density: 0.5,
            current_step: 0,
            total_steps: 16,
        }
    }
}

/// Une note avec sa durée et vélocité
#[derive(Clone, Debug)]
pub struct VoicedNote {
    /// Note MIDI (0-127)
    pub midi: u8,
    /// Vélocité (0-127)
    pub velocity: u8,
    /// Durée en nombre de steps (1 = croche, 4 = noire, etc.)
    pub duration_steps: u8,
}

impl VoicedNote {
    #[must_use]
    pub const fn new(midi: u8, velocity: u8, duration_steps: u8) -> Self {
        Self { midi, velocity, duration_steps }
    }
}

/// Trait principal pour les voicers - swappable dynamiquement
pub trait Voicer: Send + Sync {
    /// Nom du voicer (pour debug/UI)
    fn name(&self) -> &'static str;

    /// Transforme une note mélodique en voicing complet
    fn process_note(
        &mut self,
        melody_note: u8,
        base_velocity: u8,
        ctx: &VoicerContext,
    ) -> Vec<VoicedNote>;

    /// Callback appelé à chaque step (pour mettre à jour l'état interne)
    fn on_step(&mut self, ctx: &VoicerContext);

    /// Doit-on jouer un voicing à ce step? (comping mask)
    fn should_voice(&self, ctx: &VoicerContext) -> bool;

    /// Mise à jour quand la density change (recalcule le pattern de comping)
    fn on_density_change(&mut self, new_density: f32, steps: usize);
}

// === Fonctions utilitaires pour les voicers ===

/// Trouve les N notes de la gamme LCC situées immédiatement sous la note mélodique
#[must_use]
pub fn find_scale_notes_below(melody_midi: u8, lcc_scale: &[u8], count: usize) -> Vec<u8> {
    if lcc_scale.is_empty() || count == 0 {
        return vec![];
    }

    let mut notes = Vec::with_capacity(count);
    let _melody_pc = melody_midi % 12;
    let _melody_octave = melody_midi / 12;

    // Trouver la position de départ (juste sous la mélodie)
    let mut current_midi = melody_midi.saturating_sub(1);

    while notes.len() < count && current_midi >= 24 {
        let pc = current_midi % 12;
        if lcc_scale.contains(&pc) {
            notes.push(current_midi);
        }
        current_midi = current_midi.saturating_sub(1);
    }

    notes
}

/// Retourne les guide tones (tierce et septième) pour un accord donné
/// Positionne les notes dans une octave appropriée sous la mélodie
#[must_use]
pub fn get_guide_tones(chord_root_midi: u8, chord_type: ChordType, below_note: u8) -> (u8, u8) {
    let intervals = chord_type.intervals();

    // Tierce: 2ème intervalle (index 1)
    let third_interval = intervals.get(1).copied().unwrap_or(4);
    // Septième: 4ème intervalle (index 3), ou on utilise b7 par défaut
    let seventh_interval = intervals.get(3).copied().unwrap_or(10);

    let root_pc = chord_root_midi % 12;
    let third_pc = (root_pc + third_interval) % 12;
    let seventh_pc = (root_pc + seventh_interval) % 12;

    // Placer les notes sous la mélodie dans une octave appropriée
    let target_octave = (below_note / 12).saturating_sub(1);

    let third_midi = target_octave * 12 + third_pc;
    let seventh_midi = target_octave * 12 + seventh_pc;

    // S'assurer que les notes sont sous la mélodie
    let third_final =
        if third_midi >= below_note && third_midi >= 12 { third_midi - 12 } else { third_midi };

    let seventh_final = if seventh_midi >= below_note && seventh_midi >= 12 {
        seventh_midi - 12
    } else {
        seventh_midi
    };

    (third_final.max(24), seventh_final.max(24)) // Minimum C1
}

/// Applique un Drop-2 voicing: descend la 2ème note la plus haute d'une octave
pub fn apply_drop_two(notes: &mut [VoicedNote]) {
    if notes.len() >= 2 {
        // Trier par hauteur (décroissant)
        notes.sort_by(|a, b| b.midi.cmp(&a.midi));
        // La 2ème note la plus haute (index 1) descend d'une octave
        if notes[1].midi >= 12 {
            notes[1].midi -= 12;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_scale_notes_below() {
        // C Lydian: C D E F# G A B = 0, 2, 4, 6, 7, 9, 11
        let scale = vec![0, 2, 4, 6, 7, 9, 11];

        // G5 (MIDI 79) - on veut 3 notes en dessous
        let notes = find_scale_notes_below(79, &scale, 3);

        assert_eq!(notes.len(), 3);
        // Devrait être F#5, E5, D5 ou similaire (notes de la gamme sous G)
        for note in &notes {
            assert!(note < &79);
            assert!(scale.contains(&(note % 12)));
        }
    }

    #[test]
    fn test_get_guide_tones() {
        // Cmaj7: tierce = E (4), septième = B (11)
        let (third, seventh) = get_guide_tones(60, ChordType::Major7, 72);

        assert_eq!(third % 12, 4); // E
        assert_eq!(seventh % 12, 11); // B
        assert!(third < 72);
        assert!(seventh < 72);
    }
}

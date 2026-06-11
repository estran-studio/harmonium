//! Module de génération mélodique - `HarmonyNavigator`
//!
//! Génération de mélodies basée sur:
//! - Chaînes de Markov (probabilités conditionnelles)
//! - Bruit fractal 1/f (Pink Noise)
//! - Hybride Markov+Fractal

use rust_music_theory::{
    note::{Notes, Pitch, PitchSymbol},
    scale::{Direction, Scale, ScaleType},
};

use super::{RngCore, basic::ChordQuality};
use crate::{fractal::PinkNoise, tuning::MelodyParams};

pub struct HarmonyNavigator {
    pub current_scale: Scale,
    pub current_index: i32,
    pub octave: i32,
    scale_len: usize,
    last_step: i32, // Mémoire mélodique pour "Gap Fill" (Temperley)
    // === PROGRESSION HARMONIQUE: Contexte d'accord local ===
    pub current_chord_notes: Vec<u8>, // Pitch classes de l'accord actuel (ex: [0,4,7] pour Do Maj)
    pub global_key_root: u8,          // Tonique globale du morceau (0=C, 1=C#, etc.)
    // === FRACTAL NOISE ===
    pink_noise: PinkNoise,
    hurst_factor: f32,
    // === MOTIF MEMORY ===
    motif_buffer: Vec<i32>,
    motif_index: usize,
    playing_motif: bool,
    // === CONTOUR CONTROL (CORELIB-21) ===
    consecutive_direction: i32, // Count of consecutive same-direction steps (+/-)
    // === CHROMATIC PASSING TONES (CORELIB-22) ===
    chromatic_offset: i32, // ±1 semitone offset applied to next note (0 = diatonic)
    tension: f32,          // Harmony tension (0.0-1.0), controls chromatic probability
    // === REGISTER EXPLORATION (CORELIB-12) ===
    bars_in_register: u8, // How many bars spent near current octave
    // === VARIETY PARAMS (tunable, set from VarietyParams) ===
    fractal_boost: f32,           // Fractal GPS influence multiplier (default 1.8)
    fractal_range: f32,           // Fractal target amplitude (default 22.0)
    motif_new_material_bias: f32, // 0.0=all replay, 1.0=all new (default 0.6)
}

impl HarmonyNavigator {
    #[must_use]
    pub fn new(root_note: PitchSymbol, scale_type: ScaleType, octave: i32) -> Self {
        let pitch = Pitch::from(root_note);
        let scale = Scale::new(scale_type, pitch, octave as u8, None, Direction::Ascending)
            .unwrap_or_else(|_| {
                Scale::new(
                    ScaleType::PentatonicMajor,
                    Pitch::from(PitchSymbol::C),
                    4,
                    None,
                    Direction::Ascending,
                )
                .unwrap_or_else(|_| {
                    #[allow(clippy::panic)]
                    {
                        panic!("Invariant: Default scale must be valid")
                    }
                })
            });
        let scale_len = scale.notes().len();

        // Départ: accord I majeur (tonique, tierce majeure, quinte, septième majeure)
        // — en pitch classes absolues, ancrées sur la tonique (issue #26)
        let global_key_root = pitch.into_u8();
        let k = global_key_root;
        let current_chord_notes = vec![k % 12, (k + 4) % 12, (k + 7) % 12, (k + 11) % 12]; // I Maj7

        Self {
            current_scale: scale,
            current_index: 0,
            octave,
            scale_len,
            last_step: 0,
            current_chord_notes,
            global_key_root,
            pink_noise: PinkNoise::new(5), // 5 octaves de profondeur
            hurst_factor: 0.7,             // Valeur par défaut pour une mélodie "chantante"
            motif_buffer: Vec::new(),
            motif_index: 0,
            playing_motif: false,
            consecutive_direction: 0,
            chromatic_offset: 0,
            tension: 0.3,
            bars_in_register: 0,
            fractal_boost: 1.8,
            fractal_range: 22.0,
            motif_new_material_bias: 0.6,
        }
    }

    /// Apply melody tuning parameters. Replaces hardcoded defaults for fractal,
    /// motif, and pink noise settings.
    pub fn set_melody_params(&mut self, params: &MelodyParams) {
        self.pink_noise = PinkNoise::new(params.pink_noise_depth);
        self.hurst_factor = params.default_hurst_factor;
        self.fractal_boost = params.fractal_boost;
        self.fractal_range = params.fractal_range;
        self.motif_new_material_bias = params.motif_new_material_bias;
    }

    /// Change le contexte harmonique (accord courant)
    /// `root_pc`: pitch class ABSOLUE (0=C … 11=B) de la fondamentale de
    /// l'accord qui sonne — le même espace que les notes de `current_scale`,
    /// pour que le biais notes-d'accord vise les bonnes hauteurs (issue #26).
    /// L'appelant convertit son offset relatif via `key_root + offset`.
    ///
    /// Basé sur la théorie des progressions fonctionnelles (Tonique-Sous-Dominante-Dominante)
    pub fn set_chord_context(&mut self, root_pc: i32, quality: ChordQuality) {
        // Construction de l'accord selon sa qualité
        let (third, fifth, seventh) = match quality {
            ChordQuality::Major => (4, 7, 11),     // Maj7: 1-3-5-7
            ChordQuality::Minor => (3, 7, 10),     // m7: 1-♭3-5-♭7
            ChordQuality::Dominant7 => (4, 7, 10), // 7: 1-3-5-♭7 (tension)
            ChordQuality::Diminished => (3, 6, 9), // dim7: 1-♭3-♭5-♭♭7
            ChordQuality::Sus2 => (2, 7, 0),       // sus2: 1-2-5 (pas de tierce)
        };

        // Notes de l'accord en pitch classes (modulo 12)
        self.current_chord_notes = if quality == ChordQuality::Sus2 {
            // Sus2 n'a que 3 notes (pas de 7ème)
            vec![
                (root_pc % 12) as u8,
                ((root_pc + third) % 12) as u8,
                ((root_pc + fifth) % 12) as u8,
            ]
        } else {
            vec![
                (root_pc % 12) as u8,             // Fondamentale
                ((root_pc + third) % 12) as u8,   // Tierce (ou 2nde pour sus2)
                ((root_pc + fifth) % 12) as u8,   // Quinte (juste ou diminuée)
                ((root_pc + seventh) % 12) as u8, // Septième
            ]
        };
    }

    /// Vérifie si une note de la gamme fait partie de l'accord courant
    /// Ceci permet de distinguer notes d'accord (stables) vs notes de passage (transitoires)
    fn is_in_current_chord(&self, scale_degree: i32) -> bool {
        let notes = self.current_scale.notes();
        let len = notes.len() as i32;

        // Obtenir la note de la gamme à cette position
        let index = scale_degree.rem_euclid(len);
        let note = &notes[index as usize];

        // Convertir en pitch class (modulo 12)
        let pitch_class = note.pitch.into_u8();

        // Vérifier si cette pitch class est dans l'accord courant
        self.current_chord_notes.contains(&pitch_class)
    }

    /// Génère la prochaine note en utilisant des probabilités conditionnelles (Chaînes de Markov)
    /// Basé sur "Music and Probability" (David Temperley)
    pub fn next_note(&mut self, is_strong_beat: bool, rng: &mut dyn RngCore) -> f32 {
        // Position normalisée dans la gamme (0 = tonique, 1 = 2ème degré, etc.)
        let normalized_index = self.current_index.rem_euclid(self.scale_len as i32);

        // === CHAÎNES DE MARKOV: Probabilités conditionnelles ===
        let (steps, weights) = self.get_weighted_steps(normalized_index, is_strong_beat);

        // Sélection pondérée
        let chosen_step = weighted_sample(&steps, &weights, rng);

        // === GAP FILL (Temperley): Après un GRAND saut, tendance à compenser ===
        // CORELIB-21: Softened threshold from >2 to >=5, and only nudge (don't force)
        let final_step =
            if self.last_step.abs() >= 5 && chosen_step.signum() == self.last_step.signum() {
                // Very large leap in same direction: reverse by step
                if self.last_step > 0 { -1 } else { 1 }
            } else {
                chosen_step
            };

        self.last_step = final_step; // Mémoriser pour la prochaine fois
        self.current_index += final_step;

        // Contrainte: rester dans une tessiture raisonnable (± 3 octaves)
        self.current_index =
            self.current_index.clamp(-(self.scale_len as i32 * 3), self.scale_len as i32 * 3);

        self.get_frequency()
    }

    /// Génère la prochaine note en utilisant du bruit fractal (1/f)
    /// Cela crée des mélodies plus organiques et structurées que les chaînes de Markov
    pub fn next_note_fractal(&mut self, rng: &mut dyn RngCore) -> f32 {
        // 1. Obtenir une valeur "tendance" du bruit fractal
        let fractal_drift = self.pink_noise.next_value(rng);

        // 2. Mapper cette valeur à un index dans la gamme (autour d'un centre)
        // Cela remplace la marche aléatoire pure par une évolution structurée
        let center_index = 0; // Tonique centrale
        let target_index = center_index + (fractal_drift * 10.0) as i32; // Amplitude de ±10 degrés

        // 3. Lisser le mouvement (simulation de l'Exposant de Hurst)
        // Au lieu de sauter directement au target, on s'en rapproche
        // Un facteur faible (0.1) = très lisse (Hurst élevé), facteur fort (1.0) = erratique

        let diff = target_index - self.current_index;

        // Utilisation de hurst_factor pour contrôler la taille maximale du saut
        // Si hurst_factor est petit (ex: 0.1), on force des petits pas (mouvement conjoint)
        // Si hurst_factor est grand (ex: 1.0), on autorise des sauts plus grands vers la cible
        let max_step = (self.hurst_factor * 5.0).max(1.0) as i32;
        let step = diff.clamp(-max_step, max_step);

        self.current_index += step;

        // ... contraintes de gamme et conversion en fréquence ...
        self.get_frequency()
    }

    /// Génère un intervalle mélodique basé sur la logique Hybride (Markov + Fractal)
    /// Retourne le saut (step) en degrés, pas la fréquence
    fn generate_hybrid_step(&mut self, is_strong_beat: bool, rng: &mut dyn RngCore) -> i32 {
        // 1. LE GPS (Bruit Fractal) : Quelle est la "tendance" globale ?
        let fractal_drift = self.pink_noise.next_value(rng);
        let center_index = 0;
        let target_index = center_index + (fractal_drift * self.fractal_range) as i32;

        // 2. LE CONDUCTEUR (Markov) : Quels sont les mouvements musicaux valides ?
        let normalized_index = self.current_index.rem_euclid(self.scale_len as i32);
        let (steps, original_weights) = self.get_weighted_steps(normalized_index, is_strong_beat);

        // 3. LA FUSION : Fractal gently nudges the Markov distribution
        // CORELIB-21: The fractal should act as a *tiebreaker*, not override Markov.
        // Steps toward fractal target get a mild boost; others are unchanged (not penalized).
        // This preserves the broadened interval distribution from get_weighted_steps.
        let mut final_weights = Vec::with_capacity(original_weights.len());
        let current_dist = (target_index - self.current_index).abs();
        let fractal_boost = self.fractal_boost;

        for (i, &step) in steps.iter().enumerate() {
            let predicted_index = self.current_index + step;
            let new_dist = (target_index - predicted_index).abs();
            let weight = original_weights[i] as f32;

            if new_dist < current_dist {
                final_weights.push(weight * fractal_boost);
            } else {
                final_weights.push(weight); // No penalty — preserve Markov distribution
            }
        }

        // 4. SÉLECTION PONDÉRÉE
        let chosen_step = weighted_sample_f32(&steps, &final_weights, rng);

        // === Gap Fill (Temperley) — softened for CORELIB-21 ===
        // Only trigger after very large leaps (>=5) continuing in same direction
        let step = if self.last_step.abs() >= 5
            && chosen_step.abs() >= 3
            && chosen_step.signum() == self.last_step.signum()
        {
            if chosen_step > 0 { -1 } else { 1 }
        } else {
            chosen_step
        };

        // === MAX-RUN-LENGTH: Force direction change after too many same-direction moves ===
        // CORELIB-21: Addresses low direction_changes metric
        if step != 0 {
            let step_dir = step.signum();
            if step_dir == self.consecutive_direction.signum() {
                self.consecutive_direction += step_dir;
            } else {
                self.consecutive_direction = step_dir;
            }

            // After 6 consecutive same-direction steps, force a reversal
            if self.consecutive_direction.abs() >= 6 {
                self.consecutive_direction = 0;
                return if step > 0 { -step.min(3) } else { (-step).min(3) };
            }
        }

        step
    }

    /// Applique le saut mélodique, met à jour l'historique et calcule la fréquence finale
    fn apply_step_and_get_freq(&mut self, step: i32) -> f32 {
        self.last_step = step;
        self.current_index += step;

        // Contraintes physiques (Tessiture) — CORELIB-21: widened from ±2 to ±3 octaves
        self.current_index =
            self.current_index.clamp(-(self.scale_len as i32 * 3), self.scale_len as i32 * 3);

        let freq = self.get_frequency();

        // CORELIB-22: Apply chromatic offset (±1 semitone) if set
        if self.chromatic_offset != 0 {
            let offset = self.chromatic_offset;
            self.chromatic_offset = 0;
            // Shift by semitones: multiply frequency by 2^(offset/12)
            freq * (offset as f32 / 12.0).exp2()
        } else {
            freq
        }
    }

    /// Version structurée avec mémoire de motifs (Call & Response, Répétition)
    pub fn next_note_structured(
        &mut self,
        is_strong_beat: bool,
        is_new_measure: bool,
        rng: &mut dyn RngCore,
    ) -> f32 {
        // Au début d'une mesure, on décide si on réutilise le motif précédent
        if is_new_measure {
            self.bars_in_register += 1;

            // === REGISTER EXPLORATION (CORELIB-12) ===
            // After 3+ bars in same register, jump to a different octave.
            // Widens pitch_range toward reference (~80 semitones).
            let octave = self.scale_len as i32;
            if self.bars_in_register >= 3 && rng.next_f32() < 0.5 {
                // Choose jump size: 1 or 2 octaves
                let jump_size = if rng.next_f32() < 0.3 { octave * 2 } else { octave };
                // Bias direction away from current position for exploration
                let jump = if self.current_index > octave {
                    -jump_size
                } else if self.current_index < -octave {
                    jump_size
                } else if rng.next_f32() < 0.5 {
                    jump_size
                } else {
                    -jump_size
                };
                self.current_index += jump;
                let max_range = self.scale_len as i32 * 3;
                self.current_index = self.current_index.clamp(-max_range, max_range);
                self.bars_in_register = 0;
            }

            // === MOTIF DEVELOPMENT (CORELIB-8) ===
            // At each new measure, choose how to handle the motif:
            // - New material: generate fresh (clears buffer)
            // - Exact repeat: replay motif_buffer verbatim
            // - Inversion: flip all intervals (ascending↔descending)
            // - Fragmentation: replay only first half of motif
            // - Sequence: replay motif transposed up/down by 1-2 scale degrees
            // Probabilities shift with tension: high tension → more new/fragmentation
            self.motif_index = 0;
            if self.motif_buffer.is_empty() {
                self.playing_motif = false;
            } else {
                let r = rng.next_f32();
                let t = self.tension;
                let bias = self.motif_new_material_bias; // 0.6 = 60% new material

                // Replay budget = 1.0 - bias (e.g. 0.4 at default)
                let replay_budget = (1.0 - bias).max(0.0);
                // Distribute replay budget across techniques
                let exact_thresh = replay_budget * (0.15 - t * 0.05);
                let invert_thresh = exact_thresh + replay_budget * 0.12;
                let fragment_thresh = invert_thresh + replay_budget * (0.10 + t * 0.08);
                let sequence_thresh = fragment_thresh + replay_budget * 0.12;
                let embellish_thresh = sequence_thresh + replay_budget * 0.08;
                // Remainder = new material = bias + leftover

                if r < exact_thresh {
                    // Exact repeat
                    self.playing_motif = true;
                } else if r < invert_thresh {
                    // Inversion: flip intervals
                    self.motif_buffer = self.motif_buffer.iter().map(|&s| -s).collect();
                    self.playing_motif = true;
                } else if r < fragment_thresh {
                    // Fragmentation: keep only first half
                    let half = (self.motif_buffer.len() / 2).max(1);
                    self.motif_buffer.truncate(half);
                    self.playing_motif = true;
                } else if r < sequence_thresh {
                    // Sequence: transpose by +1 or +2
                    let shift = if rng.next_f32() < 0.5 { 1 } else { 2 };
                    self.current_index += shift;
                    self.playing_motif = true;
                } else if r < embellish_thresh {
                    // Embellished replay: perturb each interval by ±1
                    self.motif_buffer = self
                        .motif_buffer
                        .iter()
                        .map(|&s| {
                            let perturbation = if rng.next_f32() < 0.3 {
                                1
                            } else if rng.next_f32() < 0.5 {
                                -1
                            } else {
                                0
                            };
                            s + perturbation
                        })
                        .collect();
                    self.playing_motif = true;
                } else {
                    // New material
                    self.motif_buffer.clear();
                    self.playing_motif = false;
                }
            }
        }

        let step = if self.playing_motif && self.motif_index < self.motif_buffer.len() {
            // DÉVELOPPEMENT : replay (possibly transformed) motif interval
            self.motif_buffer[self.motif_index]
        } else {
            // GÉNÉRATION : fresh material via Markov+Fractal hybrid
            let generated_step = self.generate_hybrid_step(is_strong_beat, rng);
            if !self.playing_motif {
                self.motif_buffer.push(generated_step);
            }
            generated_step
        };

        // === CHROMATIC PASSING TONES (CORELIB-22) ===
        // On weak beats with tension > 0.3, occasionally inject ±1 semitone offset.
        // Probability scales with tension: 0% at 0.3, ~20% at 1.0.
        // Strong beats stay diatonic for stability.
        self.chromatic_offset = if !is_strong_beat && self.tension > 0.3 {
            let chromatic_prob = (self.tension - 0.3) * 0.3; // 0-21% range
            if rng.next_f32() < chromatic_prob {
                if rng.next_f32() < 0.5 { 1 } else { -1 }
            } else {
                0
            }
        } else {
            0
        };

        self.motif_index += 1;
        self.apply_step_and_get_freq(step)
    }

    /// GÉNÉRATION HYBRIDE : Le Bruit Rose (GPS) guide les choix de Markov (Conducteur).
    /// `is_strong_beat` : Permet de favoriser les notes de l'accord sur les temps forts
    pub fn next_note_hybrid(&mut self, is_strong_beat: bool, rng: &mut dyn RngCore) -> f32 {
        let step = self.generate_hybrid_step(is_strong_beat, rng);
        self.apply_step_and_get_freq(step)
    }

    /// Calcule les probabilités de mouvement selon la théorie musicale
    /// CORELIB-21: Broadened intervals for pitch variety + reduced stepwise dominance
    /// - Steps (±1): ~30-40% (was 80-90%) to match reference step_ratio ~25%
    /// - Leaps (±3,±4,±5): significant weight for pitch class entropy
    /// - Octave jumps: present in all cases for pitch range
    fn get_weighted_steps(
        &self,
        normalized_index: i32,
        is_strong_beat: bool,
    ) -> (Vec<i32>, Vec<u32>) {
        let is_chord_tone = self.is_in_current_chord(normalized_index);
        let is_tonic = normalized_index == 0;
        let is_leading_tone = self.scale_len == 7 && normalized_index == 6;
        let octave_jump = self.scale_len as i32;

        // === CAS 1: TONIQUE — affirmer l'accord via arpège et leaps ===
        if is_tonic {
            if is_strong_beat {
                // Arpège + large leaps for variety
                (
                    vec![0, 2, 4, -3, -5, 3, 5, octave_jump, -octave_jump],
                    vec![5, 20, 18, 12, 8, 10, 7, 10, 10],
                )
            } else {
                // Mix of steps and leaps
                (vec![1, -1, 2, -2, 3, -3, 4, -4], vec![15, 15, 15, 15, 12, 12, 8, 8])
            }
        }
        // === CAS 2: SENSIBLE — strong resolution but allow occasional escape ===
        else if is_leading_tone {
            (vec![1, -1, -2, 2, -3, 3], vec![50, 15, 10, 10, 8, 7])
        }
        // === CAS 3: NOTES D'ACCORD — navigate with mix of steps and leaps ===
        else if is_chord_tone {
            if is_strong_beat {
                (
                    vec![0, -2, 2, -4, 4, 1, -1, 3, -3, -5, 5, octave_jump, -octave_jump],
                    vec![5, 14, 14, 10, 8, 8, 8, 8, 8, 5, 5, 4, 3],
                )
            } else {
                (vec![1, -1, 2, -2, 3, -3, 4, -4, 5, -5], vec![14, 14, 14, 14, 10, 10, 8, 8, 4, 4])
            }
        }
        // === CAS 4: NOTES DE PASSAGE — resolve but allow leaps to non-adjacent tones ===
        else {
            (vec![1, -1, 2, -2, 3, -3, 4, -4], vec![18, 18, 14, 14, 10, 10, 8, 8])
        }
    }

    fn get_frequency(&self) -> f32 {
        let notes = self.current_scale.notes();
        let len = notes.len() as i32;

        // Calculate the actual note index and octave shift
        let mut index = self.current_index;
        let mut octave_shift = 0;

        while index < 0 {
            index += len;
            octave_shift -= 1;
        }
        while index >= len {
            index -= len;
            octave_shift += 1;
        }

        let note = &notes[index as usize];

        let pc_val = i32::from(note.pitch.into_u8());
        let note_octave = i32::from(note.octave) + octave_shift;

        let midi_note = (note_octave + 1) * 12 + pc_val;

        440.0 * ((midi_note as f32 - 69.0) / 12.0).exp2()
    }

    pub const fn set_hurst_factor(&mut self, factor: f32) {
        self.hurst_factor = factor.clamp(0.0, 1.0);
    }

    /// Set harmony tension level — controls chromatic passing tone probability (CORELIB-22)
    pub const fn set_tension(&mut self, tension: f32) {
        self.tension = tension.clamp(0.0, 1.0);
    }

    /// Set variety params from MusicalParams (tunable for CORELIB-13)
    pub fn set_variety(&mut self, fractal_boost: f32, fractal_range: f32, motif_bias: f32) {
        self.fractal_boost = fractal_boost.clamp(0.5, 4.0);
        self.fractal_range = fractal_range.clamp(5.0, 40.0);
        self.motif_new_material_bias = motif_bias.clamp(0.0, 1.0);
    }
}

/// Weighted sampling using cumulative distribution with u32 weights
fn weighted_sample(items: &[i32], weights: &[u32], rng: &mut dyn RngCore) -> i32 {
    let total: u32 = weights.iter().sum();
    if total == 0 || items.is_empty() {
        return items.first().copied().unwrap_or(0);
    }
    let threshold = rng.next_f32() * total as f32;
    let mut cumulative = 0.0;
    for (i, &w) in weights.iter().enumerate() {
        cumulative += w as f32;
        if threshold < cumulative {
            return items[i];
        }
    }
    items[items.len() - 1]
}

/// Weighted sampling using cumulative distribution with f32 weights
fn weighted_sample_f32(items: &[i32], weights: &[f32], rng: &mut dyn RngCore) -> i32 {
    let total: f32 = weights.iter().sum();
    if total <= 0.0 || items.is_empty() {
        return items.first().copied().unwrap_or(0);
    }
    let threshold = rng.next_f32() * total;
    let mut cumulative = 0.0;
    for (i, &w) in weights.iter().enumerate() {
        cumulative += w;
        if threshold < cumulative {
            return items[i];
        }
    }
    items[items.len() - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test RNG for deterministic tests
    struct TestRng {
        values: Vec<f32>,
        index: usize,
    }

    impl TestRng {
        fn new(values: Vec<f32>) -> Self {
            Self { values, index: 0 }
        }
    }

    impl RngCore for TestRng {
        fn next_f32(&mut self) -> f32 {
            let val = self.values[self.index % self.values.len()];
            self.index += 1;
            val
        }

        fn next_range_usize(&mut self, range: std::ops::Range<usize>) -> usize {
            let val = self.next_f32();
            let len = range.end - range.start;
            range.start + (val * len as f32) as usize % len
        }
    }

    #[test]
    fn test_weighted_steps_tonic_strong_beat() {
        let navigator = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
        let (steps, weights) = navigator.get_weighted_steps(0, true);

        // Sur tonique + temps fort: MOUVEMENT favorisé (arpège) plutôt qu'immobilité
        // Les sauts d'arpège (+2 tierce, +4 quinte) doivent avoir plus de poids que "0"
        let stay_weight = steps.iter().position(|&s| s == 0).map_or(0, |i| weights[i]);
        let arpeggiate_weight: u32 = steps
            .iter()
            .enumerate()
            .filter(|&(_, &s)| s == 2 || s == 4)
            .map(|(i, _)| weights[i])
            .sum();

        assert!(
            arpeggiate_weight > stay_weight,
            "Les arpèges ({arpeggiate_weight}) doivent dominer l'immobilité ({stay_weight})"
        );
    }

    #[test]
    fn test_weighted_steps_chord_tone() {
        let navigator = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
        let (steps, weights) = navigator.get_weighted_steps(2, true); // 3ème degré = note d'accord

        // Note d'accord sur temps fort: doit have mix of steps AND leaps
        assert!(steps.contains(&0)); // Peut rester
        assert!(steps.contains(&1) || steps.contains(&-1)); // Steps
        assert!(steps.contains(&3) || steps.contains(&-3)); // Leaps also present
        assert!(weights.iter().sum::<u32>() == 100); // Total des poids = 100%
    }

    #[test]
    fn test_probabilistic_movement_distribution() {
        let mut navigator = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);
        let mut rng = rand::thread_rng();

        // Générer 100 notes et vérifier la distribution
        let mut movements = Vec::new();
        for _ in 0..100 {
            let prev_index = navigator.current_index;
            navigator.next_note(false, &mut rng);
            movements.push(navigator.current_index - prev_index);
        }

        // CORELIB-21: With broadened intervals, we expect a healthy mix of steps AND leaps
        // Steps (±1) should still be present but leaps (>±2) should also appear frequently
        let steps = movements.iter().filter(|&&m| m.abs() <= 1).count();
        let leaps = movements.iter().filter(|&&m| m.abs() >= 3).count();

        assert!(steps > 10, "Should still have some stepwise motion, got {steps}");
        assert!(leaps > 10, "Should have significant leaps for variety, got {leaps}");
    }

    #[test]
    fn test_chord_context_changes_stability() {
        let mut navigator = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);

        // Test 1: Sur accord I (C Maj: C, E, G, B), la note C (degré 0) est stable
        navigator.set_chord_context(0, ChordQuality::Major); // I Maj
        assert!(navigator.is_in_current_chord(0), "C devrait être dans l'accord I");

        // Test 2: Sur accord vi (A Min: A, C, E, G), la note C (degré 0) est TOUJOURS stable
        // Parce que C fait partie de l'accord de La mineur
        navigator.set_chord_context(9, ChordQuality::Minor); // vi Min (A = +9 demi-tons depuis C)
        // Note: En pentatonique C majeur, les degrés sont C, D, E, G, A
        // L'accord de A mineur contient A, C, E, G
        // Donc le degré 0 (C) devrait toujours être dedans
        assert!(
            navigator.is_in_current_chord(0),
            "C devrait être dans l'accord vi (A mineur contient C)"
        );

        // Test 3: Vérifier que les pitch classes sont correctement calculées
        navigator.set_chord_context(5, ChordQuality::Major); // IV (F Maj: F, A, C, E)
        let chord_notes = &navigator.current_chord_notes;
        assert_eq!(chord_notes.len(), 4, "L'accord devrait avoir 4 notes");
        assert!(chord_notes.contains(&5), "F (pitch class 5) devrait être dans F Maj");
        assert!(chord_notes.contains(&9), "A (pitch class 9) devrait être dans F Maj");
        assert!(chord_notes.contains(&0), "C (pitch class 0) devrait être dans F Maj");
    }

    #[test]
    fn test_chord_progression_cycle() {
        let mut navigator = HarmonyNavigator::new(PitchSymbol::C, ScaleType::PentatonicMajor, 4);

        // Simuler la progression I-vi-IV-V
        let progression = [
            (0, ChordQuality::Major), // I
            (9, ChordQuality::Minor), // vi
            (5, ChordQuality::Major), // IV
            (7, ChordQuality::Major), // V
        ];

        for (root_offset, quality) in &progression {
            navigator.set_chord_context(*root_offset, *quality);

            // Vérifier que les notes de l'accord sont bien définies
            assert_eq!(
                navigator.current_chord_notes.len(),
                4,
                "Chaque accord devrait avoir 4 notes (1, 3, 5, 7)"
            );

            // Vérifier que les pitch classes sont dans la plage [0, 11]
            for &pc in &navigator.current_chord_notes {
                assert!(pc < 12, "Pitch class {pc} devrait être < 12");
            }
        }
    }
}

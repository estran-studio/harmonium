//! Parsimonious Voice-Leading Driver
//!
//! Implémente des transitions harmoniques fluides entre TOUS les types d'accords
//! en trouvant les voisins à 1-2 demi-tons de distance par voix.
//!
//! Étend au-delà du Neo-Riemannien P/L/R pour supporter les accords de 7ème,
//! les 6èmes, et le morphing de cardinalité (triade ↔ tétracorde).
//!
//! Basé sur les travaux de Dmitri Tymoczko sur la géométrie des espaces d'accords.

use std::sync::Arc;

use super::{HarmonyContext, HarmonyDecision, HarmonyStrategy, RngCore, TransitionType};
use crate::{
    theory::{
        PitchClass,
        chord::{Chord, ChordType},
        lydian_chromatic::LydianChromaticConcept,
    },
    tuning::VoiceLeadingParams,
};

/// Mouvement maximum de voix en demi-tons pour le mouvement parsimonieux
pub const MAX_SEMITONE_MOVEMENT: u8 = 2;

/// Tension/Release Quotient pour la sélection automatisée
///
/// Inspiré du modèle circumplex de Russell, le TRQ quantifie
/// la "direction émotionnelle" d'une transition harmonique.
#[derive(Clone, Copy, Debug)]
pub struct TRQ {
    /// Composante de tension (0.0 - 1.0): dissonance, instabilité
    pub tension: f32,
    /// Composante de release (0.0 - 1.0): tendance à la résolution
    pub release: f32,
}

impl TRQ {
    #[must_use]
    pub const fn new(tension: f32, release: f32) -> Self {
        Self { tension: tension.clamp(0.0, 1.0), release: release.clamp(0.0, 1.0) }
    }

    /// Tension nette (positive = tendu, négative = détendu)
    #[must_use]
    pub fn net(&self) -> f32 {
        self.tension - self.release
    }

    /// Calcule le TRQ pour une transition d'accord
    #[must_use]
    pub fn for_transition(from: &Chord, to: &Chord) -> Self {
        // Facteurs de tension:
        // - Le mouvement chromatique augmente la tension
        // - Les intervalles de triton augmentent la tension
        // - Aller vers une dominante augmente la tension

        let voice_distance = from.voice_leading_distance(to) as f32;
        let tension = (voice_distance / 6.0).clamp(0.0, 1.0);

        // Facteurs de release:
        // - Aller vers une tonique diminue la tension
        // - Le mouvement par degrés conjoints apporte du release
        // - Les notes communes apportent de la stabilité

        let common_tones = Self::count_common_tones(from, to);
        let release = (common_tones as f32 / 3.0).clamp(0.0, 1.0);

        Self::new(tension, release)
    }

    fn count_common_tones(a: &Chord, b: &Chord) -> usize {
        let a_pcs = a.pitch_classes();
        let b_pcs = b.pitch_classes();
        a_pcs.iter().filter(|pc| b_pcs.contains(pc)).count()
    }
}

impl Default for TRQ {
    fn default() -> Self {
        Self::new(0.5, 0.5)
    }
}

/// Un accord voisin avec son score de voice-leading
#[derive(Clone, Debug)]
pub struct Neighbor {
    /// L'accord voisin
    pub chord: Chord,
    /// Distance de voice-leading (plus bas = plus fluide)
    pub voice_leading_distance: u32,
    /// TRQ pour cette transition
    pub trq: TRQ,
    /// Type de transformation utilisée
    pub transformation: ParsimoniousTransform,
}

/// Types de transformations parsimonieuses
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParsimoniousTransform {
    /// Une voix bouge d'un demi-ton
    SingleSemitone,
    /// Une voix bouge d'un ton entier
    SingleWholeTone,
    /// Deux voix bougent d'un demi-ton chacune
    DoubleSemitone,
    /// Changement de cardinalité: triade vers tétracorde
    CardinalityExpand,
    /// Changement de cardinalité: tétracorde vers triade
    CardinalityContract,
    /// Pas de changement (identité)
    Identity,
}

/// Le moteur de Voice-Leading Parsimonieux
///
/// Contrairement au `NeoRiemannianEngine` qui utilise des tables de lookup
/// pré-calculées, ce moteur explore dynamiquement l'espace des accords
/// pour trouver les voisins les plus proches.
pub struct ParsimoniousDriver {
    /// Contexte LCC pour les suggestions de gammes
    lcc: Arc<LydianChromaticConcept>,
    /// Mouvement maximum par voix en demi-tons
    max_movement: u8,
    /// Activer le morphing de cardinalité
    allow_cardinality_morph: bool,
    /// Seuil TRQ pour la sélection (préfère les voisins avec `TRQ.net()` < seuil)
    trq_threshold: f32,
    /// Tension threshold above which high-tension neighbor filtering applies
    high_tension_threshold: f32,
    /// Tension threshold below which low-tension neighbor filtering applies
    low_tension_threshold: f32,
}

impl Default for ParsimoniousDriver {
    fn default() -> Self {
        Self::new(Arc::new(LydianChromaticConcept::new()))
    }
}

impl ParsimoniousDriver {
    #[must_use]
    pub const fn new(lcc: Arc<LydianChromaticConcept>) -> Self {
        Self {
            lcc,
            max_movement: MAX_SEMITONE_MOVEMENT,
            allow_cardinality_morph: true,
            trq_threshold: 0.5,
            high_tension_threshold: 0.6,
            low_tension_threshold: 0.4,
        }
    }

    /// Update voice-leading parameters from tuning.
    pub fn set_params(&mut self, params: &VoiceLeadingParams) {
        self.max_movement = params.max_semitone_movement;
        self.allow_cardinality_morph = params.allow_cardinality_morph;
        self.trq_threshold = params.trq_threshold;
        self.high_tension_threshold = params.high_tension_threshold;
        self.low_tension_threshold = params.low_tension_threshold;
    }

    /// Définit le mouvement maximum par voix
    #[must_use]
    pub fn with_max_movement(mut self, semitones: u8) -> Self {
        self.max_movement = semitones.min(3);
        self
    }

    /// Active/désactive le morphing de cardinalité
    #[must_use]
    pub const fn with_cardinality_morph(mut self, enabled: bool) -> Self {
        self.allow_cardinality_morph = enabled;
        self
    }

    /// Définit le seuil TRQ pour la sélection
    #[must_use]
    pub const fn with_trq_threshold(mut self, threshold: f32) -> Self {
        self.trq_threshold = threshold;
        self
    }

    /// Trouve tous les accords voisins valides dans la distance de voice-leading spécifiée
    ///
    /// C'est le cœur de l'algorithme: au lieu de suivre des rails prédéfinis (P, L, R),
    /// on explore l'espace des accords pour trouver les voisins les plus proches.
    #[must_use]
    pub fn find_neighbors(&self, chord: &Chord) -> Vec<Neighbor> {
        let mut neighbors = Vec::new();
        let current_pcs = chord.pitch_classes();

        // Stratégie 1: Bouger une seule voix de 1-2 demi-tons
        neighbors.extend(self.find_single_voice_neighbors(chord, &current_pcs));

        // Stratégie 2: Bouger deux voix d'un demi-ton chacune
        neighbors.extend(self.find_double_voice_neighbors(chord, &current_pcs));

        // Stratégie 3: Morphing de cardinalité (si activé)
        if self.allow_cardinality_morph {
            neighbors.extend(self.find_cardinality_neighbors(chord));
        }

        // Supprimer les doublons et trier par distance de voice-leading
        // On garde les transformations distinctes pour le même accord
        neighbors.sort_by(|a, b| a.voice_leading_distance.cmp(&b.voice_leading_distance));
        neighbors.dedup_by(|a, b| {
            a.chord.root == b.chord.root
                && a.chord.chord_type == b.chord.chord_type
                && a.transformation == b.transformation
        });

        neighbors
    }

    /// Trouve les voisins en bougeant une seule voix
    fn find_single_voice_neighbors(&self, original: &Chord, pcs: &[PitchClass]) -> Vec<Neighbor> {
        let mut neighbors = Vec::new();

        for (voice_idx, &pc) in pcs.iter().enumerate() {
            // Essayer de bouger cette voix de +1, -1, +2, -2 demi-tons
            for delta in &[-2i8, -1, 1, 2] {
                if delta.unsigned_abs() > self.max_movement {
                    continue;
                }

                let new_pc = ((pc as i8 + delta + 12) % 12) as u8;

                // Créer le nouveau set de pitch classes
                let mut new_pcs = pcs.to_vec();
                new_pcs[voice_idx] = new_pc;

                // Essayer d'identifier l'accord résultant
                if let Some(new_chord) = Chord::identify(&new_pcs) {
                    // Éviter l'accord identique
                    if new_chord.root == original.root
                        && new_chord.chord_type == original.chord_type
                    {
                        continue;
                    }

                    let transform = if delta.abs() == 1 {
                        ParsimoniousTransform::SingleSemitone
                    } else {
                        ParsimoniousTransform::SingleWholeTone
                    };

                    neighbors.push(Neighbor {
                        voice_leading_distance: original.voice_leading_distance(&new_chord),
                        trq: TRQ::for_transition(original, &new_chord),
                        chord: new_chord,
                        transformation: transform,
                    });
                }
            }
        }

        neighbors
    }

    /// Trouve les voisins en bougeant deux voix d'un demi-ton chacune
    fn find_double_voice_neighbors(&self, original: &Chord, pcs: &[PitchClass]) -> Vec<Neighbor> {
        let mut neighbors = Vec::new();
        let n = pcs.len();

        if n < 2 {
            return neighbors;
        }

        // Essayer toutes les paires de voix
        for i in 0..n {
            for j in (i + 1)..n {
                // Essayer toutes les combinaisons de demi-ton haut/bas
                for d1 in &[-1i8, 1] {
                    for d2 in &[-1i8, 1] {
                        let mut new_pcs = pcs.to_vec();
                        new_pcs[i] = ((pcs[i] as i8 + d1 + 12) % 12) as u8;
                        new_pcs[j] = ((pcs[j] as i8 + d2 + 12) % 12) as u8;

                        if let Some(new_chord) = Chord::identify(&new_pcs) {
                            // Éviter l'accord identique
                            if new_chord.root == original.root
                                && new_chord.chord_type == original.chord_type
                            {
                                continue;
                            }

                            neighbors.push(Neighbor {
                                voice_leading_distance: original.voice_leading_distance(&new_chord),
                                trq: TRQ::for_transition(original, &new_chord),
                                chord: new_chord,
                                transformation: ParsimoniousTransform::DoubleSemitone,
                            });
                        }
                    }
                }
            }
        }

        neighbors
    }

    /// Trouve les voisins par morphing de cardinalité (triade ↔ tétracorde)
    fn find_cardinality_neighbors(&self, original: &Chord) -> Vec<Neighbor> {
        let mut neighbors = Vec::new();

        if original.chord_type.is_triad() {
            // Expansion: triade vers tétracorde en ajoutant une 7ème
            neighbors.extend(self.expand_triad_to_tetrad(original));
        } else if original.chord_type.is_tetrad() {
            // Contraction: tétracorde vers triade en supprimant une note
            neighbors.extend(self.contract_tetrad_to_triad(original));
        }

        neighbors
    }

    /// Expansion d'une triade vers différents types de tétracordes
    fn expand_triad_to_tetrad(&self, original: &Chord) -> Vec<Neighbor> {
        let mut neighbors = Vec::new();
        let root = original.root;

        // Déterminer les expansions valides selon la qualité de la triade
        let expansions: Vec<ChordType> = match original.chord_type {
            ChordType::Major => vec![ChordType::Major7, ChordType::Dominant7, ChordType::Major6],
            ChordType::Minor => vec![ChordType::Minor7, ChordType::MinorMajor7, ChordType::Minor6],
            ChordType::Diminished => vec![ChordType::HalfDiminished, ChordType::Diminished7],
            ChordType::Augmented => vec![ChordType::Augmented7],
            ChordType::Sus4 => vec![ChordType::Dominant7Sus4],
            _ => vec![],
        };

        for chord_type in expansions {
            let new_chord = Chord::new(root, chord_type);
            neighbors.push(Neighbor {
                voice_leading_distance: original.voice_leading_distance(&new_chord),
                trq: TRQ::for_transition(original, &new_chord),
                chord: new_chord,
                transformation: ParsimoniousTransform::CardinalityExpand,
            });
        }

        neighbors
    }

    /// Contraction d'un tétracorde vers une triade
    fn contract_tetrad_to_triad(&self, original: &Chord) -> Vec<Neighbor> {
        let mut neighbors = Vec::new();
        let root = original.root;

        // Déterminer les contractions valides
        let contractions: Vec<ChordType> = match original.chord_type {
            ChordType::Major7 | ChordType::Dominant7 | ChordType::Major6 => vec![ChordType::Major],
            ChordType::Minor7 | ChordType::MinorMajor7 | ChordType::Minor6 => {
                vec![ChordType::Minor]
            }
            ChordType::HalfDiminished | ChordType::Diminished7 => vec![ChordType::Diminished],
            ChordType::Augmented7 => vec![ChordType::Augmented],
            ChordType::Dominant7Sus4 => vec![ChordType::Sus4],
            _ => vec![],
        };

        for chord_type in contractions {
            let new_chord = Chord::new(root, chord_type);
            neighbors.push(Neighbor {
                voice_leading_distance: original.voice_leading_distance(&new_chord),
                trq: TRQ::for_transition(original, &new_chord),
                chord: new_chord,
                transformation: ParsimoniousTransform::CardinalityContract,
            });
        }

        neighbors
    }

    /// Sélectionne le meilleur voisin basé sur le contexte et le TRQ
    pub fn select_neighbor(
        &self,
        neighbors: &[Neighbor],
        ctx: &HarmonyContext,
        rng: &mut dyn RngCore,
    ) -> Option<Neighbor> {
        if neighbors.is_empty() {
            return None;
        }

        // Filtrer les voisins selon la préférence de tension
        let filtered: Vec<&Neighbor> = if ctx.tension > self.high_tension_threshold {
            // Haute tension: préférer les voisins avec TRQ positif (plus de tension)
            neighbors.iter().filter(|n| n.trq.net() > 0.0).collect()
        } else if ctx.tension < self.low_tension_threshold {
            // Basse tension: préférer les voisins avec TRQ négatif (release)
            neighbors.iter().filter(|n| n.trq.net() <= 0.0).collect()
        } else {
            // Tension moyenne: considérer tous
            neighbors.iter().collect()
        };

        let candidates =
            if filtered.is_empty() { neighbors.iter().collect::<Vec<_>>() } else { filtered };

        // Sélection aléatoire pondérée basée sur la distance de voice-leading
        // Distance plus basse = poids plus élevé
        let total_weight: f32 =
            candidates.iter().map(|n| 1.0 / (n.voice_leading_distance as f32 + 1.0)).sum();

        let mut choice = rng.next_f32() * total_weight;

        for neighbor in &candidates {
            let weight = 1.0 / (neighbor.voice_leading_distance as f32 + 1.0);
            choice -= weight;
            if choice <= 0.0 {
                return Some((*neighbor).clone());
            }
        }

        Some(candidates[0].clone())
    }

    /// Trouve le chemin le plus court entre deux accords (BFS)
    ///
    /// Similaire à `find_path()` de `NeoRiemannianEngine` mais fonctionne
    /// avec tous les types d'accords.
    #[must_use]
    pub fn find_path(&self, from: &Chord, to: &Chord, max_depth: usize) -> Vec<Chord> {
        use std::collections::{HashSet, VecDeque};

        let target_key = (to.root, to.chord_type);
        let start_key = (from.root, from.chord_type);

        if start_key == target_key {
            return vec![from.clone()];
        }

        let mut visited: HashSet<(PitchClass, ChordType)> = HashSet::new();
        let mut queue: VecDeque<(Chord, Vec<Chord>)> = VecDeque::new();

        visited.insert(start_key);
        queue.push_back((from.clone(), vec![from.clone()]));

        while let Some((current, path)) = queue.pop_front() {
            if path.len() > max_depth {
                continue;
            }

            let neighbors = self.find_neighbors(&current);

            for neighbor in neighbors {
                let next_key = (neighbor.chord.root, neighbor.chord.chord_type);

                if next_key == target_key {
                    let mut full_path = path;
                    full_path.push(neighbor.chord);
                    return full_path;
                }

                if !visited.contains(&next_key) {
                    visited.insert(next_key);
                    let mut new_path = path.clone();
                    new_path.push(neighbor.chord.clone());
                    queue.push_back((neighbor.chord, new_path));
                }
            }
        }

        vec![] // Pas de chemin trouvé
    }
}

impl HarmonyStrategy for ParsimoniousDriver {
    fn next_chord(&self, ctx: &HarmonyContext, rng: &mut dyn RngCore) -> HarmonyDecision {
        // Trouver tous les voisins valides
        let neighbors = self.find_neighbors(&ctx.current_chord);

        // Sélectionner le meilleur basé sur le contexte
        let next_chord = if let Some(neighbor) = self.select_neighbor(&neighbors, ctx, rng) {
            neighbor.chord
        } else {
            // Fallback: retourner le même accord
            ctx.current_chord.clone()
        };

        // Obtenir la gamme LCC
        let parent = self.lcc.parent_lydian(&next_chord);
        let level = self.lcc.level_for_tension(ctx.tension);
        let suggested_scale = self.lcc.get_scale(parent, level);

        HarmonyDecision {
            next_chord,
            transition_type: TransitionType::Transformational,
            suggested_scale,
        }
    }

    fn name(&self) -> &'static str {
        "Parsimonious"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_driver() -> ParsimoniousDriver {
        ParsimoniousDriver::new(Arc::new(LydianChromaticConcept::new()))
    }

    #[test]
    fn test_find_neighbors_major_triad() {
        let driver = make_driver();
        let c_major = Chord::new(0, ChordType::Major);

        let neighbors = driver.find_neighbors(&c_major);

        // Devrait trouver au moins les transformations P, L, R équivalentes
        assert!(neighbors.len() >= 3);

        // C Major -> C Minor devrait être un voisin (transformation P)
        assert!(
            neighbors.iter().any(|n| n.chord.root == 0 && n.chord.chord_type == ChordType::Minor)
        );
    }

    #[test]
    fn test_find_neighbors_tetrad() {
        let driver = make_driver();
        let cmaj7 = Chord::new(0, ChordType::Major7);

        let neighbors = driver.find_neighbors(&cmaj7);

        // Devrait trouver des voisins incluant la contraction de cardinalité
        assert!(neighbors.iter().any(|n| n.chord.chord_type.is_triad()
            && n.transformation == ParsimoniousTransform::CardinalityContract));
    }

    #[test]
    fn test_cardinality_expansion() {
        let driver = make_driver();
        let c_major = Chord::new(0, ChordType::Major);

        let neighbors = driver.find_neighbors(&c_major);

        // Devrait pouvoir s'étendre vers Cmaj7, C7, C6
        assert!(
            neighbors
                .iter()
                .filter(|n| n.transformation == ParsimoniousTransform::CardinalityExpand)
                .count()
                >= 2
        );
    }

    #[test]
    fn test_trq_calculation() {
        let c_major = Chord::new(0, ChordType::Major);
        let c_minor = Chord::new(0, ChordType::Minor); // Parallèle
        let g_major = Chord::new(7, ChordType::Major); // Quinte

        let trq_to_parallel = TRQ::for_transition(&c_major, &c_minor);
        let trq_to_fifth = TRQ::for_transition(&c_major, &g_major);

        // Le parallèle a une note commune (G), devrait avoir plus de release
        assert!(trq_to_parallel.release > 0.0);

        // Vérifier que les valeurs sont dans les limites
        assert!(trq_to_parallel.tension >= 0.0 && trq_to_parallel.tension <= 1.0);
        assert!(trq_to_fifth.release >= 0.0 && trq_to_fifth.release <= 1.0);
    }

    #[test]
    fn test_trq_net() {
        let trq = TRQ::new(0.8, 0.2);
        assert!((trq.net() - 0.6).abs() < 0.001);

        let trq2 = TRQ::new(0.3, 0.7);
        assert!(trq2.net() < 0.0); // Release > Tension
    }

    #[test]
    fn test_find_path() {
        let driver = make_driver();
        let c_major = Chord::new(0, ChordType::Major);
        let a_minor = Chord::new(9, ChordType::Minor);

        let path = driver.find_path(&c_major, &a_minor, 6);

        // Un chemin devrait exister (R transformation)
        assert!(!path.is_empty());
        assert_eq!(path.first().unwrap().root, 0); // Commence par C
        assert_eq!(path.last().unwrap().root, 9); // Finit par A
    }

    #[test]
    fn test_no_self_neighbors() {
        let driver = make_driver();
        let c_major = Chord::new(0, ChordType::Major);

        let neighbors = driver.find_neighbors(&c_major);

        // Aucun voisin ne devrait être l'accord lui-même
        assert!(
            !neighbors.iter().any(|n| n.chord.root == 0 && n.chord.chord_type == ChordType::Major)
        );
    }
}

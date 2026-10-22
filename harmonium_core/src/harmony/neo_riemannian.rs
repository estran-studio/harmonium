//! Neo-Riemannian Theory - Transformations P, L, R
//!
//! Implémentation des transformations géométriques sur le Tonnetz:
//! - P (Parallel): Échange majeur/mineur sur la même fondamentale
//! - L (Leading-tone): Échange via le mouvement de la sensible
//! - R (Relative): Échange majeur/mineur relatif
//!
//! Ces transformations permettent des progressions chromatiques lisses
//! sans fonctionnalité tonale traditionnelle.

use std::sync::Arc;

use super::{HarmonyContext, HarmonyDecision, HarmonyStrategy, RngCore, TransitionType};
use crate::{
    theory::{
        PitchClass,
        chord::{Chord, ChordType},
        lydian_chromatic::LydianChromaticConcept,
    },
    tuning::NeoRiemannianParams,
};

/// Les trois opérations fondamentales Neo-Riemannian
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeoRiemannianOp {
    /// Parallel: C Major <-> C Minor (déplacer la tierce d'un demi-ton)
    P,
    /// Leading-tone exchange: C Major <-> E Minor (la fondamentale descend vers la sensible)
    L,
    /// Relative: C Major <-> A Minor (relatif majeur/mineur)
    R,
}

impl NeoRiemannianOp {
    /// Nom de l'opération
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::P => "P",
            Self::L => "L",
            Self::R => "R",
        }
    }
}

/// Opérations composées (séquences de P, L, R)
#[derive(Clone, Debug)]
pub enum CompositeOp {
    /// Opération simple
    Single(NeoRiemannianOp),
    /// P puis L: C -> Cm -> Ab
    PL,
    /// P puis R: C -> Cm -> Eb
    PR,
    /// L puis R: C -> Em -> G
    LR,
    /// R puis P: C -> Am -> A
    RP,
    /// L puis P: C -> Em -> E
    LP,
    /// Cycle complet PLR
    PLR,
    /// Cycle hexatonique
    Hexatonic(Vec<NeoRiemannianOp>),
}

/// Moteur Neo-Riemannian avec tables de lookup pré-calculées
pub struct NeoRiemannianEngine {
    /// Table P: index = root * 2 + (`is_minor` ? 1 : 0)
    p_table: [(PitchClass, bool); 24],
    /// Table L
    l_table: [(PitchClass, bool); 24],
    /// Table R
    r_table: [(PitchClass, bool); 24],
    /// Contexte LCC pour les gammes
    lcc: Arc<LydianChromaticConcept>,
    /// Tuning parameters for valence/tension thresholds and P/L/R probabilities
    params: NeoRiemannianParams,
}

impl Default for NeoRiemannianEngine {
    fn default() -> Self {
        Self::new(Arc::new(LydianChromaticConcept::new()))
    }
}

impl NeoRiemannianEngine {
    /// Crée un nouveau moteur avec tables pré-calculées
    #[must_use]
    pub fn new(lcc: Arc<LydianChromaticConcept>) -> Self {
        let mut p_table = [(0u8, false); 24];
        let mut l_table = [(0u8, false); 24];
        let mut r_table = [(0u8, false); 24];

        // Pré-calculer toutes les transformations pour les 24 triades
        // (12 majeures + 12 mineures)
        for root in 0u8..12 {
            // Index pour majeur
            let maj_idx = (root * 2) as usize;
            // Index pour mineur
            let min_idx = (root * 2 + 1) as usize;

            // === P (Parallel) ===
            // Majeur -> Mineur sur même root (et vice versa)
            p_table[maj_idx] = (root, true); // C -> Cm
            p_table[min_idx] = (root, false); // Cm -> C

            // === L (Leading-tone exchange) ===
            // C Major (C-E-G) -> E Minor (E-G-B): root descend d'un demi-ton, devient quinte
            // Majeur: nouveau root = old 3rd (root + 4)
            // Mineur: nouveau root = old root - 1 = old 5th + 4
            l_table[maj_idx] = ((root + 4) % 12, true); // C -> Em
            l_table[min_idx] = ((root + 8) % 12, false); // Cm -> Ab (root + 8 = maj6 = new root)

            // === R (Relative) ===
            // C Major -> A Minor (relatif mineur)
            // Majeur: nouveau root = old root - 3 (min 6th)
            // Mineur: nouveau root = old root + 3 (min 3rd up)
            r_table[maj_idx] = ((root + 9) % 12, true); // C -> Am (C + 9 = A)
            r_table[min_idx] = ((root + 3) % 12, false); // Am -> C (A + 3 = C)
        }

        Self { p_table, l_table, r_table, lcc, params: NeoRiemannianParams::default() }
    }

    /// Update tuning parameters.
    pub fn set_params(&mut self, params: NeoRiemannianParams) {
        self.params = params;
    }

    /// Applique une transformation P, L ou R à un accord
    #[must_use]
    pub fn apply(&self, chord: &Chord, op: NeoRiemannianOp) -> Chord {
        // Convertir l'accord en index de lookup
        let is_minor = chord.chord_type.is_minor();
        let idx = (chord.root * 2 + u8::from(is_minor)) as usize;

        let (new_root, new_is_minor) = match op {
            NeoRiemannianOp::P => self.p_table[idx],
            NeoRiemannianOp::L => self.l_table[idx],
            NeoRiemannianOp::R => self.r_table[idx],
        };

        let new_type = if new_is_minor { ChordType::Minor } else { ChordType::Major };

        Chord::new(new_root, new_type)
    }

    /// Applique une opération composée
    #[must_use]
    pub fn apply_composite(&self, chord: &Chord, op: &CompositeOp) -> Chord {
        match op {
            CompositeOp::Single(single_op) => self.apply(chord, *single_op),
            CompositeOp::PL => {
                let after_p = self.apply(chord, NeoRiemannianOp::P);
                self.apply(&after_p, NeoRiemannianOp::L)
            }
            CompositeOp::PR => {
                let after_p = self.apply(chord, NeoRiemannianOp::P);
                self.apply(&after_p, NeoRiemannianOp::R)
            }
            CompositeOp::LR => {
                let after_l = self.apply(chord, NeoRiemannianOp::L);
                self.apply(&after_l, NeoRiemannianOp::R)
            }
            CompositeOp::RP => {
                let after_r = self.apply(chord, NeoRiemannianOp::R);
                self.apply(&after_r, NeoRiemannianOp::P)
            }
            CompositeOp::LP => {
                let after_l = self.apply(chord, NeoRiemannianOp::L);
                self.apply(&after_l, NeoRiemannianOp::P)
            }
            CompositeOp::PLR => {
                let after_p = self.apply(chord, NeoRiemannianOp::P);
                let after_pl = self.apply(&after_p, NeoRiemannianOp::L);
                self.apply(&after_pl, NeoRiemannianOp::R)
            }
            CompositeOp::Hexatonic(ops) => {
                let mut result = chord.clone();
                for op in ops {
                    result = self.apply(&result, *op);
                }
                result
            }
        }
    }

    /// Génère une marche aléatoire sur le Tonnetz
    pub fn random_walk(&self, start: &Chord, steps: usize, rng: &mut dyn RngCore) -> Vec<Chord> {
        let mut path = vec![start.clone()];
        let mut current = start.clone();

        for _ in 0..steps {
            let op = match rng.next_range_usize(0..3) {
                0 => NeoRiemannianOp::P,
                1 => NeoRiemannianOp::L,
                _ => NeoRiemannianOp::R,
            };
            current = self.apply(&current, op);
            path.push(current.clone());
        }

        path
    }

    /// Trouve le chemin le plus court entre deux accords sur le Tonnetz
    #[must_use]
    pub fn find_path(&self, from: &Chord, to: &Chord) -> Vec<NeoRiemannianOp> {
        // BFS simplifié sur le Tonnetz
        use std::collections::{HashSet, VecDeque};

        let target = (to.root, to.chord_type.is_minor());
        let start = (from.root, from.chord_type.is_minor());

        if start == target {
            return vec![];
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(start);
        queue.push_back((from.clone(), vec![]));

        while let Some((current, path)) = queue.pop_front() {
            for op in [NeoRiemannianOp::P, NeoRiemannianOp::L, NeoRiemannianOp::R] {
                let next = self.apply(&current, op);
                let next_key = (next.root, next.chord_type.is_minor());

                if next_key == target {
                    let mut full_path = path;
                    full_path.push(op);
                    return full_path;
                }

                if !visited.contains(&next_key) && path.len() < 6 {
                    visited.insert(next_key);
                    let mut new_path = path.clone();
                    new_path.push(op);
                    queue.push_back((next, new_path));
                }
            }
        }

        vec![] // Pas de chemin trouvé (ne devrait pas arriver)
    }

    /// Choisit une transformation basée sur la valence
    /// Valence positive: préfère R (diatonique, familier)
    /// Valence négative: préfère L (chromatique, étrange)
    fn choose_op_by_valence(&self, valence: f32, rng: &mut dyn RngCore) -> NeoRiemannianOp {
        let r = rng.next_f32();
        let p = &self.params;

        if valence > p.positive_valence_threshold {
            if r < p.positive_r_prob {
                NeoRiemannianOp::R
            } else if r < p.positive_p_cumulative {
                NeoRiemannianOp::P
            } else {
                NeoRiemannianOp::L
            }
        } else if valence < p.negative_valence_threshold {
            if r < p.negative_l_prob {
                NeoRiemannianOp::L
            } else if r < p.negative_p_cumulative {
                NeoRiemannianOp::P
            } else {
                NeoRiemannianOp::R
            }
        } else if r < p.neutral_p_prob {
            NeoRiemannianOp::P
        } else if r < p.neutral_l_cumulative {
            NeoRiemannianOp::L
        } else {
            NeoRiemannianOp::R
        }
    }
}

impl HarmonyStrategy for NeoRiemannianEngine {
    fn next_chord(&self, ctx: &HarmonyContext, rng: &mut dyn RngCore) -> HarmonyDecision {
        // Choisir l'opération basée sur la valence
        let op = self.choose_op_by_valence(ctx.valence, rng);

        // À haute tension, on peut enchaîner plusieurs opérations
        let next_chord = if ctx.tension > self.params.composite_tension_threshold
            && rng.next_f32() < self.params.composite_probability
        {
            // 50% de chance d'opération composée à très haute tension
            let composite = match rng.next_range_usize(0..3) {
                0 => CompositeOp::PL,
                1 => CompositeOp::PR,
                _ => CompositeOp::LR,
            };
            self.apply_composite(&ctx.current_chord, &composite)
        } else {
            self.apply(&ctx.current_chord, op)
        };

        // Obtenir la gamme LCC suggérée
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
        "Neo-Riemannian"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> NeoRiemannianEngine {
        NeoRiemannianEngine::new(Arc::new(LydianChromaticConcept::new()))
    }

    #[test]
    fn test_parallel() {
        let engine = make_engine();

        // C Major -> C Minor
        let c_maj = Chord::new(0, ChordType::Major);
        let result = engine.apply(&c_maj, NeoRiemannianOp::P);
        assert_eq!(result.root, 0);
        assert_eq!(result.chord_type, ChordType::Minor);

        // C Minor -> C Major (involutif)
        let c_min = Chord::new(0, ChordType::Minor);
        let result = engine.apply(&c_min, NeoRiemannianOp::P);
        assert_eq!(result.root, 0);
        assert_eq!(result.chord_type, ChordType::Major);
    }

    #[test]
    fn test_leading_tone() {
        let engine = make_engine();

        // C Major -> E Minor
        let c_maj = Chord::new(0, ChordType::Major);
        let result = engine.apply(&c_maj, NeoRiemannianOp::L);
        assert_eq!(result.root, 4); // E
        assert_eq!(result.chord_type, ChordType::Minor);

        // E Minor -> C Major (involutif)
        let e_min = Chord::new(4, ChordType::Minor);
        let result = engine.apply(&e_min, NeoRiemannianOp::L);
        assert_eq!(result.root, 0); // C
        assert_eq!(result.chord_type, ChordType::Major);
    }

    #[test]
    fn test_relative() {
        let engine = make_engine();

        // C Major -> A Minor
        let c_maj = Chord::new(0, ChordType::Major);
        let result = engine.apply(&c_maj, NeoRiemannianOp::R);
        assert_eq!(result.root, 9); // A
        assert_eq!(result.chord_type, ChordType::Minor);

        // A Minor -> C Major (involutif)
        let a_min = Chord::new(9, ChordType::Minor);
        let result = engine.apply(&a_min, NeoRiemannianOp::R);
        assert_eq!(result.root, 0); // C
        assert_eq!(result.chord_type, ChordType::Major);
    }

    #[test]
    fn test_involution() {
        let engine = make_engine();
        let c_maj = Chord::new(0, ChordType::Major);

        // Chaque opération appliquée deux fois revient au départ
        for op in [NeoRiemannianOp::P, NeoRiemannianOp::L, NeoRiemannianOp::R] {
            let after_one = engine.apply(&c_maj, op);
            let after_two = engine.apply(&after_one, op);
            assert_eq!(after_two.root, c_maj.root);
            assert_eq!(after_two.chord_type, c_maj.chord_type);
        }
    }

    #[test]
    fn test_find_path() {
        let engine = make_engine();

        // C Major -> A Minor devrait être R
        let c_maj = Chord::new(0, ChordType::Major);
        let a_min = Chord::new(9, ChordType::Minor);
        let path = engine.find_path(&c_maj, &a_min);
        assert_eq!(path, vec![NeoRiemannianOp::R]);

        // C Major -> C Minor devrait être P
        let c_min = Chord::new(0, ChordType::Minor);
        let path = engine.find_path(&c_maj, &c_min);
        assert_eq!(path, vec![NeoRiemannianOp::P]);
    }

    #[test]
    fn test_composite_pl() {
        let engine = make_engine();

        // C Major -> PL -> Ab Major
        // C Major -P-> C Minor -L-> Ab Major
        let c_maj = Chord::new(0, ChordType::Major);
        let result = engine.apply_composite(&c_maj, &CompositeOp::PL);

        // C Minor (0, minor) -L-> ?
        // L sur mineur: (root + 8) % 12, major = (0 + 8) % 12 = 8 = G#/Ab, major
        assert_eq!(result.root, 8); // Ab
        assert_eq!(result.chord_type, ChordType::Major);
    }
}

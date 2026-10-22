//! Steedman Grammar - Grammaire générative pour progressions harmoniques
//!
//! Implémentation basée sur les travaux de Mark Steedman sur les grammaires
//! combinatoires pour la musique jazz/pop.
//!
//! Règles de réécriture: V -> ii-V, I -> vi-I, etc.

use std::sync::{Arc, RwLock};

use super::{
    HarmonyContext, HarmonyDecision, HarmonyStrategy, RngCore, TransitionType,
    lydian_chromatic::LydianChromaticConcept,
};
use crate::theory::{
    PitchClass,
    chord::{Chord, ChordType},
};

/// Chiffres romains (degrés de la gamme) - Version étendue
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RomanNumeral {
    I,
    II,
    III,
    IV,
    V,
    VI,
    VII,
    /// Dominante secondaire: V/V
    VofV,
    /// Dominante secondaire: V/ii
    VofII,
    /// Dominante secondaire: V/IV
    VofIV,
    // === NOUVEAUX (V2) ===
    /// Dominante secondaire: V/vi (E7 en C)
    VofVI,
    /// Dominante secondaire: V/iii (B7 en C)
    VofIII,
    /// Substitution tritonique: bII7 (Db7 en C)
    FlatII,
    /// Modal interchange: bVI (Ab en C)
    FlatVI,
    /// Modal interchange: bVII (Bb en C)
    FlatVII,
}

impl RomanNumeral {
    /// Intervalle en demi-tons depuis la tonique
    #[must_use]
    pub const fn interval(&self) -> u8 {
        match self {
            Self::I => 0,
            Self::II => 2,
            Self::III => 4,
            Self::IV => 5,
            Self::V => 7,
            Self::VI => 9,
            Self::VII => 11,
            Self::VofV => 2,  // V/V = II7 (D7 en C)
            Self::VofII => 9, // V/ii = VI7 (A7 en C)
            Self::VofIV => 0, // V/IV = I7 (C7 en C)
            // Nouveaux V2
            Self::VofVI => 4,    // V/vi = III7 (E7 en C)
            Self::VofIII => 11,  // V/iii = VII7 (B7 en C)
            Self::FlatII => 1,   // bII7 (Db7 en C) - substitution tritonique
            Self::FlatVI => 8,   // bVI (Ab en C)
            Self::FlatVII => 10, // bVII (Bb en C)
        }
    }

    /// Nom pour affichage
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::I => "I",
            Self::II => "ii",
            Self::III => "iii",
            Self::IV => "IV",
            Self::V => "V",
            Self::VI => "vi",
            Self::VII => "vii°",
            Self::VofV => "V/V",
            Self::VofII => "V/ii",
            Self::VofIV => "V/IV",
            // Nouveaux V2
            Self::VofVI => "V/vi",
            Self::VofIII => "V/iii",
            Self::FlatII => "bII7",
            Self::FlatVI => "bVI",
            Self::FlatVII => "bVII",
        }
    }
}

/// Catégories de règles pour les probabilités spécifiques au style
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    /// Mouvement cadentiel de base (V-I, IV-I)
    Cadential,
    /// Préparation ii-V
    Preparation,
    /// Back-cycling récursif (III-VI-II-V)
    BackCycle,
    /// Substitution tritonique
    TritoneSubstitution,
    /// Mouvement déceptif/chromatique
    Deceptive,
    /// Modal interchange
    ModalInterchange,
}

/// Presets de style avec distributions de probabilité
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum GrammarStyle {
    /// Jazz standard (bebop)
    #[default]
    Jazz,
    /// Pop/Rock
    Pop,
    /// Classique/Romantique
    Classical,
    /// Contemporain/Neo-Soul
    Contemporary,
}

impl GrammarStyle {
    /// Obtient le multiplicateur de probabilité pour chaque catégorie de règle
    #[must_use]
    pub const fn category_weight(&self, category: RuleCategory) -> f32 {
        match (self, category) {
            // Jazz: ii-V, back-cycling, tritone subs
            (Self::Jazz, RuleCategory::Preparation) => 1.5,
            (Self::Jazz, RuleCategory::BackCycle) => 1.3,
            (Self::Jazz, RuleCategory::TritoneSubstitution) => 1.2,
            (Self::Jazz, RuleCategory::Cadential) => 0.8,

            // Pop: cadences basiques, moins de complexité
            (Self::Pop, RuleCategory::Cadential) => 1.5,
            (Self::Pop, RuleCategory::Preparation) => 0.8,
            (Self::Pop, RuleCategory::BackCycle) => 0.3,
            (Self::Pop, RuleCategory::TritoneSubstitution) => 0.2,

            // Classical: cadentiel, déceptif
            (Self::Classical, RuleCategory::Cadential) => 1.4,
            (Self::Classical, RuleCategory::Deceptive) => 1.2,
            (Self::Classical, RuleCategory::TritoneSubstitution) => 0.1,

            // Contemporary: modal, chromatique
            (Self::Contemporary, RuleCategory::ModalInterchange) => 1.4,
            (Self::Contemporary, RuleCategory::Deceptive) => 1.3,
            (Self::Contemporary, RuleCategory::BackCycle) => 1.1,

            // Défaut
            _ => 1.0,
        }
    }
}

/// Règle de réécriture de la grammaire (version étendue)
#[derive(Clone, Debug)]
pub struct RewriteRule {
    /// Symbole de gauche (ce qu'on remplace)
    pub lhs: RomanNumeral,
    /// Symbole(s) de droite (par quoi on remplace)
    pub rhs: Vec<RomanNumeral>,
    /// Poids de la règle (probabilité relative)
    pub weight: f32,
    /// Valence minimale pour appliquer cette règle
    pub min_valence: f32,
    /// Valence maximale
    pub max_valence: f32,
    /// Catégorie de la règle pour la sélection par style
    pub category: RuleCategory,
    /// Profondeur de récursion maximale pour cette règle (0 = pas de récursion)
    pub max_recursion: u8,
}

/// Grammaire de Steedman pour progressions harmoniques
///
/// Utilise `RwLock` pour permettre la mutabilité intérieure thread-safe,
/// ce qui permet de maintenir l'état des expansions en cours
/// (ex: V → ii-V garde le V en attente après avoir joué ii)
pub struct SteedmanGrammar {
    /// Règles de réécriture
    rules: Vec<RewriteRule>,
    /// Contexte LCC
    lcc: Arc<LydianChromaticConcept>,
    /// État mutable: numeral courant + progression en attente
    state: RwLock<GrammarState>,
}

/// État interne mutable de la grammaire (version étendue)
#[derive(Clone, Debug)]
struct GrammarState {
    /// Accord courant (en numéraux romains)
    current_numeral: RomanNumeral,
    /// Progression générée en attente (ex: après V→ii-V, contient [V])
    pending_progression: Vec<RomanNumeral>,
    /// Profondeur de récursion courante par catégorie
    recursion_depth: std::collections::HashMap<RuleCategory, u8>,
    /// Profondeur de récursion maximale autorisée
    max_recursion_depth: u8,
    /// Style courant
    style: GrammarStyle,
}

impl GrammarState {
    fn new() -> Self {
        Self {
            current_numeral: RomanNumeral::I,
            pending_progression: Vec::new(),
            recursion_depth: std::collections::HashMap::new(),
            max_recursion_depth: 2, // Défaut: jusqu'à 2 niveaux de back-cycling
            style: GrammarStyle::Jazz,
        }
    }

    fn can_recurse(&self, category: RuleCategory) -> bool {
        let current = self.recursion_depth.get(&category).copied().unwrap_or(0);
        current < self.max_recursion_depth
    }

    fn increment_recursion(&mut self, category: RuleCategory) {
        let current = self.recursion_depth.entry(category).or_insert(0);
        *current += 1;
    }

    fn reset_recursion(&mut self) {
        self.recursion_depth.clear();
    }
}

impl SteedmanGrammar {
    /// Crée une nouvelle grammaire avec les règles jazz/pop standard
    #[must_use]
    pub fn new(lcc: Arc<LydianChromaticConcept>) -> Self {
        let mut grammar = Self { rules: Vec::new(), lcc, state: RwLock::new(GrammarState::new()) };

        grammar.init_rules();
        grammar
    }

    /// Initialise les règles de réécriture (version étendue avec Rule 3, 4)
    fn init_rules(&mut self) {
        // === RÈGLE 1: CADENCES DE BASE ===
        self.add_rule_ext(
            RomanNumeral::V,
            vec![RomanNumeral::I],
            0.9,
            -1.0,
            1.0,
            RuleCategory::Cadential,
            0,
        );

        self.add_rule_ext(
            RomanNumeral::IV,
            vec![RomanNumeral::I],
            0.5,
            0.2,
            1.0,
            RuleCategory::Cadential,
            0,
        );

        // === RÈGLE 2: PRÉPARATION ii-V ===
        // Règle 2a: V -> ii7-V (ii mineur, le standard jazz)
        self.add_rule_ext(
            RomanNumeral::V,
            vec![RomanNumeral::II, RomanNumeral::V],
            0.85,
            -1.0,
            1.0,
            RuleCategory::Preparation,
            0,
        );

        // V -> IV-V (approche plagale, rock/pop)
        self.add_rule_ext(
            RomanNumeral::V,
            vec![RomanNumeral::IV, RomanNumeral::V],
            0.5,
            0.0,
            1.0,
            RuleCategory::Preparation,
            0,
        );

        // === RÈGLE 3: BACK-CYCLING RÉCURSIF ===
        // Règle 3: V -> VI-II-V (vi-ii-V)
        self.add_rule_ext(
            RomanNumeral::V,
            vec![RomanNumeral::VI, RomanNumeral::II, RomanNumeral::V],
            0.4,
            -1.0,
            1.0,
            RuleCategory::BackCycle,
            1,
        );

        // Règle 3 étendue: V -> III-VI-II-V (iii-vi-ii-V)
        self.add_rule_ext(
            RomanNumeral::V,
            vec![RomanNumeral::III, RomanNumeral::VI, RomanNumeral::II, RomanNumeral::V],
            0.2,
            -0.5,
            1.0,
            RuleCategory::BackCycle,
            0,
        );

        // Règle 3b: Dominante back-cycle (II7 - V7 pour le blues)
        self.add_rule_ext(
            RomanNumeral::V,
            vec![RomanNumeral::VofV, RomanNumeral::V],
            0.3,
            -1.0,
            0.3,
            RuleCategory::BackCycle,
            0,
        );

        // === RÈGLE 4: SUBSTITUTION TRITONIQUE ===
        // Règle 4: V7 -> bII7 (tritone sub)
        self.add_rule_ext(
            RomanNumeral::V,
            vec![RomanNumeral::FlatII],
            0.35,
            -1.0,
            0.5,
            RuleCategory::TritoneSubstitution,
            0,
        );

        // Règle 4 étendue: ii-bII-I (sub-dominant tritone)
        self.add_rule_ext(
            RomanNumeral::V,
            vec![RomanNumeral::II, RomanNumeral::FlatII],
            0.25,
            -1.0,
            0.3,
            RuleCategory::TritoneSubstitution,
            0,
        );

        // === DOMINANTES SECONDAIRES ===
        self.add_rule_ext(
            RomanNumeral::VI,
            vec![RomanNumeral::VofVI, RomanNumeral::VI],
            0.3,
            -1.0,
            1.0,
            RuleCategory::Preparation,
            0,
        );

        self.add_rule_ext(
            RomanNumeral::II,
            vec![RomanNumeral::VofII, RomanNumeral::II],
            0.25,
            -1.0,
            1.0,
            RuleCategory::Preparation,
            0,
        );

        // === MOUVEMENT DÉCEPTIF ===
        self.add_rule_ext(
            RomanNumeral::V,
            vec![RomanNumeral::VI],
            0.3,
            -0.5,
            0.5,
            RuleCategory::Deceptive,
            0,
        );

        self.add_rule_ext(
            RomanNumeral::V,
            vec![RomanNumeral::FlatVI],
            0.2,
            -1.0,
            0.0,
            RuleCategory::Deceptive,
            0,
        );

        // === MODAL INTERCHANGE ===
        self.add_rule_ext(
            RomanNumeral::IV,
            vec![RomanNumeral::FlatVII, RomanNumeral::IV],
            0.2,
            -0.5,
            0.5,
            RuleCategory::ModalInterchange,
            0,
        );

        // I -> bVII-IV-I (backdoor progression)
        self.add_rule_ext(
            RomanNumeral::I,
            vec![RomanNumeral::FlatVII, RomanNumeral::IV, RomanNumeral::I],
            0.15,
            -0.5,
            0.5,
            RuleCategory::ModalInterchange,
            0,
        );

        // === ÉLABORATIONS DE TONIQUE ===
        self.add_rule_ext(
            RomanNumeral::I,
            vec![RomanNumeral::VI, RomanNumeral::I],
            0.4,
            -0.5,
            0.5,
            RuleCategory::Cadential,
            0,
        );

        self.add_rule_ext(
            RomanNumeral::I,
            vec![RomanNumeral::IV, RomanNumeral::I],
            0.6,
            0.2,
            1.0,
            RuleCategory::Cadential,
            0,
        );

        self.add_rule_ext(
            RomanNumeral::I,
            vec![RomanNumeral::III, RomanNumeral::I],
            0.3,
            -1.0,
            0.3,
            RuleCategory::Cadential,
            0,
        );

        // === PRÉPARATIONS DE SOUS-DOMINANTE ===
        self.add_rule_ext(
            RomanNumeral::IV,
            vec![RomanNumeral::II, RomanNumeral::IV],
            0.4,
            -1.0,
            1.0,
            RuleCategory::Preparation,
            0,
        );
    }

    /// Ajoute une règle de réécriture (version étendue)
    #[allow(clippy::too_many_arguments)]
    fn add_rule_ext(
        &mut self,
        lhs: RomanNumeral,
        rhs: Vec<RomanNumeral>,
        weight: f32,
        min_valence: f32,
        max_valence: f32,
        category: RuleCategory,
        max_recursion: u8,
    ) {
        self.rules.push(RewriteRule {
            lhs,
            rhs,
            weight,
            min_valence,
            max_valence,
            category,
            max_recursion,
        });
    }

    /// Définit le style de la grammaire
    pub fn set_style(&self, style: GrammarStyle) {
        if let Ok(mut state) = self.state.write() {
            state.style = style;
        }
    }

    /// Définit la profondeur de récursion maximale pour le back-cycling
    pub fn set_max_recursion(&self, depth: u8) {
        if let Ok(mut state) = self.state.write() {
            state.max_recursion_depth = depth;
        }
    }

    /// Retourne le style courant
    pub fn style(&self) -> GrammarStyle {
        self.state.read().map(|s| s.style).unwrap_or_default()
    }

    /// Trouve les règles applicables pour un symbole et une valence
    fn applicable_rules(&self, symbol: RomanNumeral, valence: f32) -> Vec<&RewriteRule> {
        self.rules
            .iter()
            .filter(|r| r.lhs == symbol && valence >= r.min_valence && valence <= r.max_valence)
            .collect()
    }

    /// Sélectionne une règle avec pondération par style et contrôle de récursion
    fn select_rule_styled(
        &self,
        rules: &[&RewriteRule],
        state: &GrammarState,
        rng: &mut dyn RngCore,
    ) -> Option<(Vec<RomanNumeral>, RuleCategory)> {
        if rules.is_empty() {
            return None;
        }

        // Appliquer les poids de style et les contraintes de récursion
        let weighted_rules: Vec<(&RewriteRule, f32)> = rules
            .iter()
            .filter(|r| {
                // Ignorer les règles qui dépassent la limite de récursion
                if r.max_recursion > 0 && !state.can_recurse(r.category) {
                    return false;
                }
                true
            })
            .map(|r| {
                let style_weight = state.style.category_weight(r.category);
                (*r, r.weight * style_weight)
            })
            .collect();

        if weighted_rules.is_empty() {
            return None;
        }

        let total_weight: f32 = weighted_rules.iter().map(|(_, w)| w).sum();
        let mut choice = rng.next_f32() * total_weight;

        for (rule, weight) in &weighted_rules {
            choice -= weight;
            if choice <= 0.0 {
                return Some((rule.rhs.clone(), rule.category));
            }
        }

        weighted_rules.first().map(|(r, _)| (r.rhs.clone(), r.category))
    }

    /// Convertit un numeral en accord concret (version étendue avec style)
    pub fn realize(&self, numeral: RomanNumeral, key: PitchClass, valence: f32) -> Chord {
        let style = self.style();
        self.realize_with_style(numeral, key, valence, style)
    }

    /// Convertit un numeral en accord concret avec style explicite
    pub fn realize_with_style(
        &self,
        numeral: RomanNumeral,
        key: PitchClass,
        valence: f32,
        style: GrammarStyle,
    ) -> Chord {
        let interval = numeral.interval();
        let root = (key + interval) % 12;

        let chord_type = match numeral {
            RomanNumeral::I => {
                if valence > 0.3 {
                    ChordType::Major7
                } else {
                    ChordType::Major
                }
            }
            // Règle 3a/3b: ii réalisation dépend du style
            RomanNumeral::II => match style {
                GrammarStyle::Jazz | GrammarStyle::Contemporary => ChordType::Minor7,
                GrammarStyle::Pop => ChordType::Minor,
                GrammarStyle::Classical => {
                    if valence < -0.3 {
                        ChordType::Dominant7 // Saveur napolitaine
                    } else {
                        ChordType::Minor
                    }
                }
            },
            RomanNumeral::III => match style {
                GrammarStyle::Jazz => ChordType::Minor7,
                _ => ChordType::Minor,
            },
            RomanNumeral::IV => {
                if valence > 0.3 && matches!(style, GrammarStyle::Jazz) {
                    ChordType::Major7
                } else {
                    ChordType::Major
                }
            }
            RomanNumeral::V => ChordType::Dominant7,
            RomanNumeral::VI => match style {
                GrammarStyle::Jazz => ChordType::Minor7,
                _ => ChordType::Minor,
            },
            RomanNumeral::VII => ChordType::HalfDiminished,
            // Substitution tritonique: toujours dominant 7
            RomanNumeral::FlatII => ChordType::Dominant7,
            // Modal interchange
            RomanNumeral::FlatVI => match style {
                GrammarStyle::Jazz | GrammarStyle::Contemporary => ChordType::Major7,
                _ => ChordType::Major,
            },
            RomanNumeral::FlatVII => ChordType::Dominant7,
            // Dominantes secondaires: toujours dominant 7
            RomanNumeral::VofV
            | RomanNumeral::VofII
            | RomanNumeral::VofIV
            | RomanNumeral::VofVI
            | RomanNumeral::VofIII => ChordType::Dominant7,
        };

        Chord::new(root, chord_type)
    }

    /// Génère le prochain accord de la progression (stateful via `RwLock`)
    fn generate_next_stateful(&self, ctx: &HarmonyContext, rng: &mut dyn RngCore) -> RomanNumeral {
        let mut state = match self.state.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // If the lock is poisoned, we can try to recover the data
                // or just return a default value to prevent a crash.
                // For a music generator, robustness is preferred.
                poisoned.into_inner()
            }
        };

        // Si on a une progression en attente, la consommer
        if !state.pending_progression.is_empty() {
            let next = state.pending_progression.remove(0);
            state.current_numeral = next;
            return next;
        }

        // Sinon, essayer d'étendre le symbole courant
        let rules = self.applicable_rules(state.current_numeral, ctx.valence);

        let next_numeral =
            if let Some((expansion, category)) = self.select_rule_styled(&rules, &state, rng) {
                // Tracker la récursion pour les règles de back-cycling
                if category == RuleCategory::BackCycle {
                    state.increment_recursion(category);
                }

                if expansion.len() > 1 {
                    // Stocker les symboles suivants dans la progression en attente
                    // Ex: V → ii-V devient: retourne ii, garde [V] en attente
                    state.pending_progression = expansion[1..].to_vec();
                    expansion[0]
                } else if !expansion.is_empty() {
                    expansion[0]
                } else {
                    self.default_progression_for(state.current_numeral, ctx, rng)
                }
            } else {
                // Fallback: mouvement par quinte ou progression standard
                self.default_progression_for(state.current_numeral, ctx, rng)
            };

        // Reset la récursion quand on arrive à une résolution (I)
        if next_numeral == RomanNumeral::I {
            state.reset_recursion();
        }

        state.current_numeral = next_numeral;
        next_numeral
    }

    /// Progression par défaut quand aucune règle ne s'applique
    fn default_progression_for(
        &self,
        current: RomanNumeral,
        ctx: &HarmonyContext,
        rng: &mut dyn RngCore,
    ) -> RomanNumeral {
        let choice = rng.next_f32();

        // Progression cyclique basée sur le cercle des quintes
        match current {
            RomanNumeral::I => {
                if ctx.valence > 0.0 && choice < 0.4 {
                    RomanNumeral::V
                } else if choice < 0.7 {
                    RomanNumeral::IV
                } else {
                    RomanNumeral::VI
                }
            }
            RomanNumeral::II => RomanNumeral::V,
            RomanNumeral::III => RomanNumeral::VI,
            RomanNumeral::IV => {
                if choice < 0.6 {
                    RomanNumeral::V
                } else {
                    RomanNumeral::I
                }
            }
            RomanNumeral::V => RomanNumeral::I,
            RomanNumeral::VI => {
                if choice < 0.5 {
                    RomanNumeral::II
                } else {
                    RomanNumeral::IV
                }
            }
            RomanNumeral::VII => RomanNumeral::I,
            // Dominantes secondaires
            RomanNumeral::VofV => RomanNumeral::V,
            RomanNumeral::VofII => RomanNumeral::II,
            RomanNumeral::VofIV => RomanNumeral::IV,
            RomanNumeral::VofVI => RomanNumeral::VI,
            RomanNumeral::VofIII => RomanNumeral::III,
            // Substitution tritonique et modal interchange
            RomanNumeral::FlatII => RomanNumeral::I, // bII7 résout sur I
            RomanNumeral::FlatVI => {
                if choice < 0.5 {
                    RomanNumeral::V // bVI -> V (backdoor)
                } else {
                    RomanNumeral::I // bVI -> I (plagal mineur)
                }
            }
            RomanNumeral::FlatVII => RomanNumeral::IV, // bVII -> IV
        }
    }

    /// Retourne true si une expansion est en attente
    pub fn has_pending(&self) -> bool {
        self.state.read().map(|s| !s.pending_progression.is_empty()).unwrap_or(false)
    }

    /// Retourne le numeral courant (pour debug/UI)
    pub fn current_numeral(&self) -> RomanNumeral {
        self.state.read().map(|s| s.current_numeral).unwrap_or(RomanNumeral::I)
    }
}

impl HarmonyStrategy for SteedmanGrammar {
    fn next_chord(&self, ctx: &HarmonyContext, rng: &mut dyn RngCore) -> HarmonyDecision {
        // Utilise la méthode stateful avec RefCell pour maintenir l'état
        // des expansions en cours (ex: V → ii-V garde le V en attente)
        let next_numeral = self.generate_next_stateful(ctx, rng);
        let next_chord = self.realize(next_numeral, ctx.global_key, ctx.valence);

        // Obtenir la gamme LCC
        let parent = self.lcc.parent_lydian(&next_chord);
        let level = self.lcc.level_for_tension(ctx.tension);
        let suggested_scale = self.lcc.get_scale(parent, level);

        HarmonyDecision { next_chord, transition_type: TransitionType::Functional, suggested_scale }
    }

    fn name(&self) -> &'static str {
        "Steedman Grammar"
    }
}

impl SteedmanGrammar {
    /// Déduit le numeral romain d'un accord basé sur la tonique globale
    /// (Utilisé pour les tests et potentiellement pour analyse future)
    #[allow(dead_code)]
    const fn deduce_numeral(&self, chord: &Chord, key: PitchClass) -> RomanNumeral {
        let interval = (chord.root + 12 - key) % 12;

        match interval {
            0 => RomanNumeral::I,
            2 => RomanNumeral::II,
            4 => RomanNumeral::III,
            5 => RomanNumeral::IV,
            7 => RomanNumeral::V,
            9 => RomanNumeral::VI,
            11 => RomanNumeral::VII,
            _ => RomanNumeral::I, // Fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grammar() -> SteedmanGrammar {
        SteedmanGrammar::new(Arc::new(LydianChromaticConcept::new()))
    }

    #[test]
    fn test_realize_basic() {
        let grammar = make_grammar();

        // I en C = C Major
        let chord = grammar.realize(RomanNumeral::I, 0, 0.5);
        assert_eq!(chord.root, 0);
        assert!(matches!(chord.chord_type, ChordType::Major | ChordType::Major7));

        // V en C = G (7 ou 12)
        let chord = grammar.realize(RomanNumeral::V, 0, 0.5);
        assert_eq!(chord.root, 7);

        // ii en C = D Minor
        let chord = grammar.realize(RomanNumeral::II, 0, 0.0);
        assert_eq!(chord.root, 2);
        assert!(chord.chord_type.is_minor());
    }

    #[test]
    fn test_deduce_numeral() {
        let grammar = make_grammar();

        // C en key C = I
        assert_eq!(grammar.deduce_numeral(&Chord::new(0, ChordType::Major), 0), RomanNumeral::I);

        // G en key C = V
        assert_eq!(grammar.deduce_numeral(&Chord::new(7, ChordType::Major), 0), RomanNumeral::V);

        // Am en key C = vi
        assert_eq!(grammar.deduce_numeral(&Chord::new(9, ChordType::Minor), 0), RomanNumeral::VI);
    }

    #[test]
    fn test_applicable_rules() {
        let grammar = make_grammar();

        // V devrait avoir plusieurs règles applicables
        let rules = grammar.applicable_rules(RomanNumeral::V, 0.0);
        assert!(!rules.is_empty());

        // La règle V -> ii-V devrait être présente
        let has_ii_v = rules.iter().any(|r| r.rhs == vec![RomanNumeral::II, RomanNumeral::V]);
        assert!(has_ii_v);
    }
}

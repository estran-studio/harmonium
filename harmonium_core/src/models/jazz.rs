//! Grammaire fonctionnelle de Steedman : état local, aléa injecté, aucune dépendance Legacy.
use std::collections::{HashMap, VecDeque};

use crate::theory::{
    PitchClass,
    chord::{Chord, ChordType},
};

/// Chiffres romains (degrés de la gamme) - Version étendue
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
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

/// Instance locale de la grammaire ; toutes les décisions utilisent l'aléa fourni.
#[derive(Clone, Debug)]
pub struct SteedmanGrammar {
    pub rules: Vec<RewriteRule>,
    current: RomanNumeral,
    pending: VecDeque<RomanNumeral>,
    recursion: HashMap<RuleCategory, u8>,
    max_recursion: u8,
    style: GrammarStyle,
}
impl Default for SteedmanGrammar {
    fn default() -> Self {
        let mut grammar = Self::new(Vec::new(), 2);
        grammar.init_rules();
        grammar
    }
}
impl SteedmanGrammar {
    pub fn new(rules: Vec<RewriteRule>, max_recursion: u8) -> Self {
        Self {
            rules,
            current: RomanNumeral::I,
            pending: VecDeque::new(),
            recursion: HashMap::new(),
            max_recursion,
            style: GrammarStyle::Jazz,
        }
    }
    pub fn set_style(&mut self, style: GrammarStyle) {
        self.style = style;
    }
    pub fn set_max_recursion(&mut self, depth: u8) {
        self.max_recursion = depth;
    }
    pub const fn style(&self) -> GrammarStyle {
        self.style
    }
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
    pub const fn current_numeral(&self) -> RomanNumeral {
        self.current
    }

    pub fn next_numeral(&mut self, valence: f32, random: &mut dyn FnMut() -> f32) -> RomanNumeral {
        if let Some(next) = self.pending.pop_front() {
            self.current = next;
            return next;
        }
        let eligible = |rule: &&RewriteRule| {
            rule.lhs == self.current
                && valence >= rule.min_valence
                && valence <= rule.max_valence
                && (rule.max_recursion == 0
                    || self.recursion.get(&rule.category).copied().unwrap_or(0)
                        < self.max_recursion)
                && rule.weight > 0.0
        };
        let total: f32 = self
            .rules
            .iter()
            .filter(eligible)
            .map(|r| r.weight * self.style.category_weight(r.category))
            .sum();
        let mut selected = None;
        if total > 0.0 {
            let mut choice = random() * total;
            for rule in self.rules.iter().filter(eligible) {
                choice -= rule.weight * self.style.category_weight(rule.category);
                if choice <= 0.0 {
                    selected = Some(rule);
                    break;
                }
            }
        }
        let next = if let Some(rule) = selected {
            if rule.category == RuleCategory::BackCycle {
                *self.recursion.entry(rule.category).or_insert(0) += 1;
            }
            self.pending.extend(rule.rhs.iter().skip(1).copied());
            rule.rhs
                .first()
                .copied()
                .unwrap_or_else(|| Self::default_progression_for(self.current, valence, random))
        } else {
            Self::default_progression_for(self.current, valence, random)
        };
        if next == RomanNumeral::I {
            self.recursion.clear();
        }
        self.current = next;
        next
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

    /// Convertit un numeral en accord concret avec style explicite
    pub fn realize(
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

    /// Progression par défaut quand aucune règle ne s'applique
    fn default_progression_for(
        current: RomanNumeral,
        valence: f32,
        random: &mut dyn FnMut() -> f32,
    ) -> RomanNumeral {
        let choice = random();

        // Progression cyclique basée sur le cercle des quintes
        match current {
            RomanNumeral::I => {
                if valence > 0.0 && choice < 0.4 {
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
}

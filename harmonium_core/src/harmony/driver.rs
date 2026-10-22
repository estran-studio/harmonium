//! `HarmonicDriver` - Orchestrateur principal du système harmonique
//!
//! Le `HarmonicDriver` choisit dynamiquement entre:
//! - Steedman Grammar (Tension < 0.5): Progressions fonctionnelles narratives
//! - Neo-Riemannian (Tension > 0.7, triades): Transformations géométriques P/L/R
//! - Parsimonious (Tension > 0.7, tétracordes): Voice-leading parsimonieux
//!
//! Le Lydian Chromatic Concept (LCC) agit comme filtre vertical global.

use std::sync::Arc;

use super::{
    HarmonyContext, HarmonyDecision, HarmonyStrategy, RngCore, TransitionType,
    lydian_chromatic::LydianChromaticConcept,
    neo_riemannian::NeoRiemannianEngine,
    parsimonious::ParsimoniousDriver,
    pivot::{PivotDetector, PivotType},
    steedman_grammar::{GrammarStyle, SteedmanGrammar},
};
use crate::{
    theory::{
        PitchClass,
        chord::{Chord, ChordType},
    },
    tuning::HarmonyDriverParams,
};

/// Mode de stratégie actuel (version V2 avec Parsimonious)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrategyMode {
    /// Utilise Steedman Grammar (harmonie fonctionnelle)
    Steedman,
    /// Utilise Neo-Riemannian (transformations géométriques pour triades)
    NeoRiemannian,
    /// Utilise Parsimonious Driver (voice-leading pour tous types d'accords)
    Parsimonious,
    /// En transition entre les stratégies
    Transitioning { progress: f32 },
}

impl StrategyMode {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Steedman => "Steedman",
            Self::NeoRiemannian => "Neo-Riemannian",
            Self::Parsimonious => "Parsimonious",
            Self::Transitioning { .. } => "Transitioning",
        }
    }
}

/// Orchestrateur principal du système harmonique V3
///
/// Gère trois stratégies harmoniques:
/// - Steedman: Harmonie fonctionnelle (basse tension)
/// - Neo-Riemannian: Transformations géométriques pour triades (haute tension)
/// - Parsimonious: Voice-leading parsimonieux pour tous accords (haute tension + tétracordes)
pub struct HarmonicDriver {
    /// Moteur Steedman Grammar (enhanced V2)
    steedman: SteedmanGrammar,
    /// Moteur Neo-Riemannian (triades uniquement - fast path)
    neo_riemannian: NeoRiemannianEngine,
    /// Moteur Parsimonious (tous types d'accords)
    parsimonious: ParsimoniousDriver,
    /// Contexte LCC partagé
    lcc: Arc<LydianChromaticConcept>,
    /// Détecteur de pivots
    pivot: PivotDetector,

    // === État ===
    /// Accord courant
    current_chord: Chord,
    /// Historique des accords récents (Taboo List - fenêtre glissante de 2 accords)
    /// Permet d'éviter les boucles immédiates (E->C->E->C)
    chord_history: Vec<Chord>,
    /// Stratégie actuelle
    current_strategy: StrategyMode,
    /// Position dans la phrase (en mesures)
    phrase_position: usize,
    /// Dernière tension observée
    last_tension: f32,
    /// Tonique globale
    global_key: PitchClass,

    // === Hysteresis ===
    /// Dernière stratégie sélectionnée (pour l'hystérésis)
    last_strategy: StrategyMode,

    /// Seuils d'hystérésis configurables
    /// Seuil inférieur pour Steedman (reste en Steedman jusqu'à ce que la tension dépasse ce seuil)
    pub steedman_lower: f32,
    /// Seuil supérieur pour Steedman (entre en Steedman seulement si la tension tombe en dessous)
    pub steedman_upper: f32,
    /// Seuil inférieur pour Neo-Riemannian (entre en Neo-Riemannian seulement si la tension dépasse ce seuil)
    pub neo_lower: f32,
    /// Seuil supérieur pour Neo-Riemannian (reste en Neo-Riemannian jusqu'à ce que la tension tombe en dessous)
    pub neo_upper: f32,

    // === TuningParams-driven fields ===
    /// Bias added to previous strategy weight for stability.
    hysteresis_boost: f32,
    /// Upper tension threshold for dramatic drop detection.
    dramatic_tension_drop_upper: f32,
    /// Lower tension threshold for dramatic drop detection.
    dramatic_tension_drop_lower: f32,
    /// Probability of resolving directly to tonic (vs V-I).
    cadential_resolution_probability: f32,
    /// Max chord generation retries to avoid A-B-A loops.
    max_retries: usize,
}

impl HarmonicDriver {
    /// Crée un nouveau `HarmonicDriver` with tuning parameters.
    #[must_use]
    pub fn new(initial_key: PitchClass, params: &HarmonyDriverParams) -> Self {
        let lcc = Arc::new(LydianChromaticConcept::new());

        Self {
            steedman: SteedmanGrammar::new(lcc.clone()),
            neo_riemannian: NeoRiemannianEngine::new(lcc.clone()),
            parsimonious: ParsimoniousDriver::new(lcc.clone()),
            pivot: PivotDetector::new(lcc.clone()),
            lcc,
            current_chord: Chord::new(initial_key, ChordType::Major),
            chord_history: Vec::new(),
            current_strategy: StrategyMode::Steedman,
            phrase_position: 0,
            last_tension: 0.5,
            global_key: initial_key,
            last_strategy: StrategyMode::Steedman,
            steedman_lower: params.steedman_lower,
            steedman_upper: params.steedman_upper,
            neo_lower: params.neo_lower,
            neo_upper: params.neo_upper,
            hysteresis_boost: params.hysteresis_boost,
            dramatic_tension_drop_upper: params.dramatic_tension_drop_upper,
            dramatic_tension_drop_lower: params.dramatic_tension_drop_lower,
            cadential_resolution_probability: params.cadential_resolution_probability,
            max_retries: params.max_retries,
        }
    }

    /// Définit la tonique globale
    pub const fn set_key(&mut self, key: PitchClass) {
        self.global_key = key % 12;
    }

    /// Définit le style de la grammaire Steedman
    pub fn set_grammar_style(&self, style: GrammarStyle) {
        self.steedman.set_style(style);
    }

    /// Configure les seuils d'hystérésis pour la sélection de stratégie
    ///
    /// Les zones d'hystérésis permettent d'éviter les basculements chaotiques entre stratégies
    /// lorsque la tension fluctue autour des seuils.
    ///
    /// # Arguments
    /// * `steedman_lower` - Seuil inférieur pour Steedman (reste en Steedman tant que tension > ce seuil)
    /// * `steedman_upper` - Seuil supérieur pour Steedman (entre en Steedman si tension < ce seuil)
    /// * `neo_lower` - Seuil inférieur pour Neo-Riemannian (entre en Neo-Riemannian si tension > ce seuil)
    /// * `neo_upper` - Seuil supérieur pour Neo-Riemannian (reste en Neo-Riemannian tant que tension < ce seuil)
    ///
    /// # Panics
    /// Panique si les seuils ne sont pas dans l'ordre croissant valide:
    /// `steedman_lower` < `steedman_upper` ≤ `neo_lower` < `neo_upper`
    ///
    /// # Example
    /// ```
    /// use harmonium_core::harmony::driver::HarmonicDriver;
    /// use harmonium_core::tuning::HarmonyDriverParams;
    /// let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default());
    /// driver.set_hysteresis_thresholds(0.45, 0.55, 0.65, 0.75);
    /// ```
    pub fn set_hysteresis_thresholds(
        &mut self,
        steedman_lower: f32,
        steedman_upper: f32,
        neo_lower: f32,
        neo_upper: f32,
    ) {
        assert!(
            steedman_lower < steedman_upper,
            "steedman_lower ({steedman_lower}) doit être < steedman_upper ({steedman_upper})"
        );
        assert!(
            steedman_upper <= neo_lower,
            "steedman_upper ({steedman_upper}) doit être ≤ neo_lower ({neo_lower})"
        );
        assert!(
            neo_lower < neo_upper,
            "neo_lower ({neo_lower}) doit être < neo_upper ({neo_upper})"
        );

        self.steedman_lower = steedman_lower;
        self.steedman_upper = steedman_upper;
        self.neo_lower = neo_lower;
        self.neo_upper = neo_upper;
    }

    /// Retourne le nom de la stratégie actuelle
    pub const fn current_strategy_name(&self) -> &'static str {
        self.current_strategy.name()
    }

    /// Retourne l'accord courant
    pub const fn current_chord(&self) -> &Chord {
        &self.current_chord
    }

    /// Sélectionne la stratégie appropriée basée sur le contexte (implémentation legacy)
    ///
    /// Cette méthode est l'implémentation originale avec des seuils stricts.
    /// Elle est conservée pour compatibilité et testing, mais `select_strategy_probabilistic()`
    /// est recommandée pour une transition plus fluide entre stratégies.
    #[allow(dead_code)]
    fn select_strategy(&self, ctx: &HarmonyContext) -> StrategyMode {
        let is_tetrad = ctx.current_chord.chord_type.is_tetrad();
        let (steedman_w, _neo_w) = self.pivot.crossfade_weight(ctx.tension);

        if steedman_w > 0.99 {
            // Zone Steedman pure (tension < 0.5)
            StrategyMode::Steedman
        } else if ctx.tension > 0.7 {
            // Haute tension: choisir selon le type d'accord
            if is_tetrad {
                // Tétracordes: utiliser Parsimonious (peut gérer les 7èmes)
                StrategyMode::Parsimonious
            } else {
                // Triades: utiliser Neo-Riemannian (fast path)
                StrategyMode::NeoRiemannian
            }
        } else {
            // Zone de transition (0.5 - 0.7): utiliser Parsimonious comme bridge
            StrategyMode::Parsimonious
        }
    }

    /// Sélectionne la stratégie avec hystérésis et blending probabiliste
    ///
    /// Cette méthode améliore `select_strategy()` en:
    /// 1. Utilisant des zones d'hystérésis pour éviter les basculements chaotiques
    /// 2. Appliquant une sélection probabiliste pondérée dans les zones de transition
    /// 3. Biaisant légèrement vers la stratégie précédente pour plus de stabilité
    ///
    /// # Arguments
    /// * `ctx` - Le contexte harmonique actuel
    /// * `rng` - Générateur de nombres aléatoires pour la sélection probabiliste
    ///
    /// # Returns
    /// La stratégie sélectionnée basée sur la tension et l'hystérésis
    fn select_strategy_probabilistic(
        &mut self,
        ctx: &HarmonyContext,
        rng: &mut dyn RngCore,
    ) -> StrategyMode {
        let is_tetrad = ctx.current_chord.chord_type.is_tetrad();

        // Obtenir les poids à trois voies avec hystérésis
        let (mut steedman_w, mut parsimonious_w, mut neo_w) =
            self.pivot.crossfade_weight_three_hysteresis(
                ctx.tension,
                self.steedman_lower,
                self.steedman_upper,
                self.neo_lower,
                self.neo_upper,
            );

        // Gérer les tétracordes: Neo-Riemannian ne peut pas les traiter
        // Redistribuer neo_w vers parsimonious_w
        if is_tetrad && neo_w > 0.0 {
            parsimonious_w += neo_w;
            neo_w = 0.0;
        }

        // Zones pures: déterministe
        if steedman_w >= 0.99 {
            self.last_strategy = StrategyMode::Steedman;
            return StrategyMode::Steedman;
        }
        if parsimonious_w >= 0.99 {
            self.last_strategy = StrategyMode::Parsimonious;
            return StrategyMode::Parsimonious;
        }
        if neo_w >= 0.99 {
            let strategy =
                if is_tetrad { StrategyMode::Parsimonious } else { StrategyMode::NeoRiemannian };
            self.last_strategy = strategy;
            return strategy;
        }

        // Zones d'hystérésis: probabiliste avec biais vers la dernière stratégie
        let hysteresis_boost = self.hysteresis_boost;

        match self.last_strategy {
            StrategyMode::Steedman => {
                let boost = steedman_w * hysteresis_boost;
                steedman_w += boost;
                parsimonious_w = boost.mul_add(-0.5, parsimonious_w).max(0.0);
                neo_w = boost.mul_add(-0.5, neo_w).max(0.0);
            }
            StrategyMode::Parsimonious => {
                let boost = parsimonious_w * hysteresis_boost;
                parsimonious_w += boost;
                steedman_w = boost.mul_add(-0.5, steedman_w).max(0.0);
                neo_w = boost.mul_add(-0.5, neo_w).max(0.0);
            }
            StrategyMode::NeoRiemannian => {
                let boost = neo_w * hysteresis_boost;
                neo_w += boost;
                steedman_w = boost.mul_add(-0.5, steedman_w).max(0.0);
                parsimonious_w = boost.mul_add(-0.5, parsimonious_w).max(0.0);
            }
            StrategyMode::Transitioning { .. } => {
                // Pas de biais pour l'état de transition
            }
        }

        // Normaliser les poids
        let total = steedman_w + parsimonious_w + neo_w;
        let (steedman_w_norm, parsimonious_w_norm, neo_w_norm) = if total > 0.0 {
            (steedman_w / total, parsimonious_w / total, neo_w / total)
        } else {
            // Fallback si tous les poids sont nuls
            (0.33, 0.34, 0.33)
        };

        // Sélection aléatoire pondérée
        let rand_val = rng.next_f32();

        let strategy = if rand_val < steedman_w_norm {
            StrategyMode::Steedman
        } else if rand_val < steedman_w_norm + parsimonious_w_norm || is_tetrad {
            StrategyMode::Parsimonious
        } else {
            StrategyMode::NeoRiemannian
        };

        // Debug logging: afficher les changements de stratégie
        if cfg!(debug_assertions) {
            eprintln!(
                "[Harmony] tension={:.3} | weights=(S:{:.2}, P:{:.2}, N:{:.2}) | rand={:.2} | prev={:?} → selected={:?}",
                ctx.tension,
                steedman_w_norm,
                parsimonious_w_norm,
                neo_w_norm,
                rand_val,
                self.last_strategy,
                strategy
            );
        }

        // Mettre à jour last_strategy pour le prochain appel
        self.last_strategy = strategy;

        strategy
    }

    /// Génère le prochain accord basé sur l'état émotionnel
    pub fn next_chord(
        &mut self,
        tension: f32,
        valence: f32,
        rng: &mut dyn RngCore,
    ) -> HarmonyDecision {
        // Détecter une chute dramatique de tension
        let dramatic_tension_drop = self.last_tension > self.dramatic_tension_drop_upper
            && tension < self.dramatic_tension_drop_lower;

        // Construire le contexte
        let ctx = HarmonyContext {
            current_chord: self.current_chord.clone(),
            global_key: self.global_key,
            tension,
            valence,
            measure_in_phrase: self.phrase_position / 4,
            beat_in_measure: self.phrase_position % 4,
        };

        // Sélectionner la stratégie appropriée avec hystérésis et blending probabiliste
        let selected_strategy = self.select_strategy_probabilistic(&ctx, rng);
        let (steedman_w, neo_w) = self.pivot.crossfade_weight(tension);

        // Générer une décision avec protection contre les boucles A->B->A
        let mut decision = match selected_strategy {
            StrategyMode::Steedman => {
                self.current_strategy = StrategyMode::Steedman;

                // Si chute dramatique de tension, forcer une résolution cadentielle
                if dramatic_tension_drop {
                    self.force_cadential_resolution(&ctx, rng)
                } else {
                    self.steedman.next_chord(&ctx, rng)
                }
            }
            StrategyMode::NeoRiemannian => {
                // Vérifier que l'accord courant est une triade
                if ctx.current_chord.chord_type.is_triad() {
                    self.current_strategy = StrategyMode::NeoRiemannian;
                    self.neo_riemannian.next_chord(&ctx, rng)
                } else {
                    // Fallback: utiliser Parsimonious pour les tétracordes
                    self.current_strategy = StrategyMode::Parsimonious;
                    self.parsimonious.next_chord(&ctx, rng)
                }
            }
            StrategyMode::Parsimonious => {
                self.current_strategy = StrategyMode::Parsimonious;
                self.parsimonious.next_chord(&ctx, rng)
            }
            StrategyMode::Transitioning { .. } => {
                // Zone de transition avec pivot
                self.current_strategy = StrategyMode::Transitioning { progress: neo_w };
                self.handle_transition(&ctx, steedman_w, neo_w, rng)
            }
        };

        // Vérifier la boucle A->B->A et réessayer si nécessaire
        for retry in 0..self.max_retries {
            if !self.would_create_aba_loop(&decision.next_chord) {
                break;
            }

            // Si on crée une boucle, réessayer avec la même stratégie
            if cfg!(debug_assertions) {
                eprintln!(
                    "  [Taboo] Évitement A->B->A: {:?} {:?} → {:?} {:?} (retry {})",
                    self.current_chord.root,
                    self.current_chord.chord_type,
                    decision.next_chord.root,
                    decision.next_chord.chord_type,
                    retry + 1
                );
            }

            decision = match selected_strategy {
                StrategyMode::Steedman => {
                    if dramatic_tension_drop {
                        self.force_cadential_resolution(&ctx, rng)
                    } else {
                        self.steedman.next_chord(&ctx, rng)
                    }
                }
                StrategyMode::NeoRiemannian => {
                    if ctx.current_chord.chord_type.is_triad() {
                        self.neo_riemannian.next_chord(&ctx, rng)
                    } else {
                        self.parsimonious.next_chord(&ctx, rng)
                    }
                }
                StrategyMode::Parsimonious => self.parsimonious.next_chord(&ctx, rng),
                StrategyMode::Transitioning { .. } => {
                    self.handle_transition(&ctx, steedman_w, neo_w, rng)
                }
            };
        }

        // Debug logging: afficher les changements d'accord
        if cfg!(debug_assertions) {
            eprintln!(
                "  → Chord: {:?} {:?} → {:?} {:?} | strategy={:?}",
                self.current_chord.root,
                self.current_chord.chord_type,
                decision.next_chord.root,
                decision.next_chord.chord_type,
                selected_strategy
            );
        }

        // Mettre à jour l'état et l'historique (Taboo List avec fenêtre glissante de 2)
        self.chord_history.push(self.current_chord.clone());
        if self.chord_history.len() > 2 {
            self.chord_history.remove(0); // Maintenir une fenêtre de 2 accords
        }

        self.current_chord = decision.next_chord.clone();
        self.phrase_position += 1;
        self.last_tension = tension;

        decision
    }

    /// Gère la transition entre stratégies
    fn handle_transition(
        &self,
        ctx: &HarmonyContext,
        steedman_w: f32,
        neo_w: f32,
        rng: &mut dyn RngCore,
    ) -> HarmonyDecision {
        // Vérifier si l'accord courant est un pivot
        let pivot_type = self.pivot.is_pivot_chord(&ctx.current_chord);

        if pivot_type == PivotType::None {
            // Pas de pivot: en créer un si nécessaire
            let target = if neo_w > steedman_w {
                self.neo_riemannian.next_chord(ctx, rng).next_chord
            } else {
                self.steedman.next_chord(ctx, rng).next_chord
            };

            // Générer un accord pivot
            let pivot_chord = self.pivot.create_pivot(&ctx.current_chord, &target, ctx.tension);

            // Obtenir la gamme LCC pour le pivot
            let parent = self.lcc.parent_lydian(&pivot_chord);
            let level = self.lcc.level_for_tension(ctx.tension);
            let suggested_scale = self.lcc.get_scale(parent, level);

            HarmonyDecision {
                next_chord: pivot_chord,
                transition_type: TransitionType::Pivot,
                suggested_scale,
            }
        } else {
            // On est sur un pivot: laisser la stratégie dominante prendre le relais
            if neo_w > steedman_w {
                self.neo_riemannian.next_chord(ctx, rng)
            } else {
                self.steedman.next_chord(ctx, rng)
            }
        }
    }

    /// Retourne la gamme LCC actuelle pour la génération mélodique
    pub fn get_current_scale(&self, tension: f32) -> Vec<PitchClass> {
        let parent = self.lcc.parent_lydian(&self.current_chord);
        let level = self.lcc.level_for_tension(tension);
        self.lcc.get_scale(parent, level)
    }

    /// Vérifie si une note mélodique est valide dans le contexte actuel
    pub fn is_valid_melody_note(&self, note: PitchClass, tension: f32) -> bool {
        self.lcc.is_valid_note(note, &self.current_chord, tension)
    }

    /// Retourne le poids d'une note pour la génération mélodique
    pub fn melody_note_weight(&self, note: PitchClass, tension: f32) -> f32 {
        self.lcc.note_weight(note, &self.current_chord, tension)
    }

    /// Réinitialise la position dans la phrase
    pub const fn reset_phrase(&mut self) {
        self.phrase_position = 0;
    }

    /// Vérifie si un accord est dans la Taboo List (historique des 2 derniers accords)
    /// Retourne true si l'accord proposé est identique à un accord dans l'historique récent
    /// Exception: le retour à la tonique (I) est toujours autorisé
    fn would_create_aba_loop(&self, proposed_chord: &Chord) -> bool {
        // Permettre le retour à la tonique (I) - toujours autorisé pour les cadences
        if proposed_chord.root == self.global_key {
            return false;
        }

        // Vérifier si l'accord proposé est dans l'historique récent (Taboo List)
        self.chord_history.iter().any(|historical_chord| {
            proposed_chord.root == historical_chord.root
                && proposed_chord.chord_type == historical_chord.chord_type
        })
    }

    /// Force une résolution cadentielle (V -> I ou directement I)
    /// Utilisé lors d'une chute dramatique de tension pour donner un sentiment de résolution
    fn force_cadential_resolution(
        &self,
        ctx: &HarmonyContext,
        rng: &mut dyn RngCore,
    ) -> HarmonyDecision {
        // Calculer la dominante (V = +7 demi-tons depuis la tonique)
        let dominant_root = (self.global_key + 7) % 12;

        // Si on est déjà sur la dominante, résoudre vers la tonique
        if ctx.current_chord.root == dominant_root {
            let tonic_chord = Chord::new(self.global_key, ChordType::Major);
            let parent = self.lcc.parent_lydian(&tonic_chord);
            let level = self.lcc.level_for_tension(ctx.tension);
            let suggested_scale = self.lcc.get_scale(parent, level);

            if cfg!(debug_assertions) {
                eprintln!("  [Resolution] Chute de tension: V → I ({})", tonic_chord.name());
            }

            return HarmonyDecision {
                next_chord: tonic_chord,
                transition_type: TransitionType::Functional,
                suggested_scale,
            };
        }

        // Si on n'est pas sur la dominante, choisir aléatoirement entre:
        // 1. Aller directement à la tonique (60%)
        // 2. Aller à la dominante pour préparer la résolution (40%)
        let go_to_tonic = rng.next_f32() < self.cadential_resolution_probability;

        let target_root = if go_to_tonic { self.global_key } else { dominant_root };

        // Choisir le type d'accord basé sur la valence
        let chord_type = if ctx.valence > 0.0 { ChordType::Major } else { ChordType::Minor };

        let resolution_chord = Chord::new(target_root, chord_type);
        let parent = self.lcc.parent_lydian(&resolution_chord);
        let level = self.lcc.level_for_tension(ctx.tension);
        let suggested_scale = self.lcc.get_scale(parent, level);

        if cfg!(debug_assertions) {
            eprintln!(
                "  [Resolution] Chute de tension: {} → {} (cadence forcée)",
                ctx.current_chord.name(),
                resolution_chord.name()
            );
        }

        HarmonyDecision {
            next_chord: resolution_chord,
            transition_type: TransitionType::Functional,
            suggested_scale,
        }
    }

    /// Retourne le décalage root en demi-tons depuis la tonique globale
    pub fn root_offset(&self) -> i32 {
        (i32::from(self.current_chord.root) - i32::from(self.global_key) + 12) % 12
    }

    /// Retourne si l'accord courant est mineur
    pub const fn is_minor(&self) -> bool {
        self.current_chord.chord_type.is_minor()
    }

    /// Convertit vers le format `ChordQuality` de l'ancien système (pour compatibilité)
    pub const fn to_basic_quality(&self) -> super::basic::ChordQuality {
        self.current_chord.to_basic_quality()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRng(u64);

    impl RngCore for TestRng {
        fn next_f32(&mut self) -> f32 {
            self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12345);
            ((self.0 >> 16) & 0x7fff) as f32 / 32768.0
        }

        fn next_range_usize(&mut self, range: std::ops::Range<usize>) -> usize {
            let f = self.next_f32();
            range.start + (f * (range.end - range.start) as f32) as usize
        }
    }

    #[test]
    fn test_low_tension_uses_steedman() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        let mut rng = TestRng(42);

        // Tension basse -> Steedman
        let _decision = driver.next_chord(0.3, 0.5, &mut rng);
        assert_eq!(driver.current_strategy, StrategyMode::Steedman);
    }

    #[test]
    fn test_high_tension_uses_neo_riemannian() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        let mut rng = TestRng(42);

        // Tension haute -> Neo-Riemannian
        let _decision = driver.next_chord(0.9, 0.5, &mut rng);
        assert_eq!(driver.current_strategy, StrategyMode::NeoRiemannian);
    }

    #[test]
    fn test_transition_zone() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        let mut rng = TestRng(42);

        // Tension moyenne -> Parsimonious (bridge entre Steedman et Neo-Riemannian)
        let _decision = driver.next_chord(0.6, 0.5, &mut rng);
        // En zone de transition (0.5-0.7), Parsimonious sert de bridge
        assert!(matches!(driver.current_strategy, StrategyMode::Parsimonious));
    }

    #[test]
    fn test_scale_generation() {
        let driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C

        // Basse tension -> gamme Lydienne (consonante)
        let scale = driver.get_current_scale(0.0);
        assert!(scale.contains(&0)); // C
        assert!(scale.contains(&6)); // F# (caractéristique du Lydien)

        // Haute tension -> gamme chromatique
        let scale = driver.get_current_scale(1.0);
        assert_eq!(scale.len(), 12);
    }

    #[test]
    fn test_root_offset() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C

        // Après quelques accords, vérifier que root_offset est correct
        driver.current_chord = Chord::new(7, ChordType::Major); // G
        assert_eq!(driver.root_offset(), 7);

        driver.current_chord = Chord::new(5, ChordType::Major); // F
        assert_eq!(driver.root_offset(), 5);
    }

    // ========================================
    // Tests de progression (pas de X → X)
    // ========================================

    #[test]
    fn test_steedman_produces_chord_changes() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        let mut rng = TestRng(42);

        // Basse tension = Steedman
        let tension = 0.3;
        let valence = 0.5;

        let mut changes = 0;
        let iterations = 10;

        for _ in 0..iterations {
            let old_chord = driver.current_chord.clone();
            let decision = driver.next_chord(tension, valence, &mut rng);

            if decision.next_chord.root != old_chord.root
                || decision.next_chord.chord_type != old_chord.chord_type
            {
                changes += 1;
            }
        }

        // Au moins 50% des itérations devraient produire un changement
        assert!(
            changes >= iterations / 2,
            "Steedman devrait produire des changements d'accords: {changes} changes sur {iterations}"
        );
    }

    #[test]
    fn test_neo_riemannian_produces_chord_changes() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        driver.current_chord = Chord::new(0, ChordType::Major); // Triade pour Neo-Riemannian
        let mut rng = TestRng(123);

        // Haute tension + triade = Neo-Riemannian
        let tension = 0.9;
        let valence = 0.0;

        let mut changes = 0;
        let iterations = 10;

        for _ in 0..iterations {
            let old_chord = driver.current_chord.clone();
            let decision = driver.next_chord(tension, valence, &mut rng);

            if decision.next_chord.root != old_chord.root
                || decision.next_chord.chord_type != old_chord.chord_type
            {
                changes += 1;
            }
        }

        // Neo-Riemannian devrait TOUJOURS changer l'accord (P, L, R transforment)
        assert!(
            changes >= iterations * 8 / 10,
            "Neo-Riemannian devrait produire des changements: {changes} changes sur {iterations}"
        );
    }

    #[test]
    fn test_parsimonious_produces_chord_changes() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        driver.current_chord = Chord::new(0, ChordType::Major7); // Tétracorde pour Parsimonious
        let mut rng = TestRng(456);

        // Haute tension + tétracorde = Parsimonious
        let tension = 0.85;
        let valence = 0.5;

        let mut changes = 0;
        let iterations = 10;

        for _ in 0..iterations {
            let old_chord = driver.current_chord.clone();
            let decision = driver.next_chord(tension, valence, &mut rng);

            if decision.next_chord.root != old_chord.root
                || decision.next_chord.chord_type != old_chord.chord_type
            {
                changes += 1;
            }
        }

        // Parsimonious devrait produire des changements (voisins trouvés)
        assert!(
            changes >= iterations / 2,
            "Parsimonious devrait produire des changements: {changes} changes sur {iterations}"
        );
    }

    #[test]
    fn test_progression_variety() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        let mut rng = TestRng(789);

        // Collecter les accords sur 20 itérations avec tension variable
        let mut chords_seen: std::collections::HashSet<(u8, ChordType)> =
            std::collections::HashSet::new();

        for i in 0..20 {
            // Varier la tension pour tester différentes stratégies
            let tension = (i as f32 / 20.0).mul_add(0.8, 0.1); // 0.1 à 0.9
            let valence = if i % 2 == 0 { 0.5 } else { -0.5 };

            let decision = driver.next_chord(tension, valence, &mut rng);
            chords_seen.insert((decision.next_chord.root, decision.next_chord.chord_type));
        }

        // On devrait voir au moins 5 accords différents sur 20 itérations
        assert!(
            chords_seen.len() >= 5,
            "La progression devrait avoir de la variété: {} accords uniques sur 20",
            chords_seen.len()
        );
    }

    #[test]
    fn test_strategy_switch_continues_progression() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        let mut rng = TestRng(999);

        // Commencer avec Steedman (basse tension)
        let decision1 = driver.next_chord(0.3, 0.5, &mut rng);
        assert_eq!(driver.current_strategy, StrategyMode::Steedman);
        let chord_after_steedman = decision1.next_chord;

        // Passer à Neo-Riemannian (haute tension, garder triade)
        driver.current_chord = Chord::new(chord_after_steedman.root, ChordType::Major);
        let chord_before_neo = driver.current_chord.clone(); // Capturer AVANT

        let decision2 = driver.next_chord(0.9, 0.5, &mut rng);
        assert_eq!(driver.current_strategy, StrategyMode::NeoRiemannian);

        // Neo-Riemannian devrait produire un accord différent
        assert!(
            decision2.next_chord.root != chord_before_neo.root
                || decision2.next_chord.chord_type != chord_before_neo.chord_type,
            "Le switch vers Neo-Riemannian devrait changer l'accord: {} → {}",
            chord_before_neo.name(),
            decision2.next_chord.name()
        );
    }

    #[test]
    fn test_tetrad_high_tension_uses_parsimonious() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        driver.current_chord = Chord::new(0, ChordType::Dominant7); // Tétracorde
        let mut rng = TestRng(111);

        // Haute tension + tétracorde = Parsimonious (pas Neo-Riemannian)
        let decision = driver.next_chord(0.85, 0.5, &mut rng);

        assert_eq!(
            driver.current_strategy,
            StrategyMode::Parsimonious,
            "Tétracorde à haute tension devrait utiliser Parsimonious"
        );

        // Devrait produire un changement
        assert!(
            decision.next_chord.root != 0 || decision.next_chord.chord_type != ChordType::Dominant7,
            "Parsimonious devrait changer l'accord"
        );
    }

    #[test]
    fn test_no_stuck_on_same_chord() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        let mut rng = TestRng(222);

        // Test avec différentes configurations
        let configs = [
            (0.2, 0.5, ChordType::Major),    // Steedman
            (0.6, 0.0, ChordType::Minor),    // Parsimonious bridge
            (0.9, 0.5, ChordType::Major),    // Neo-Riemannian
            (0.85, -0.5, ChordType::Minor7), // Parsimonious tetrad
        ];

        for (tension, valence, start_type) in configs {
            driver.current_chord = Chord::new(0, start_type);

            let mut same_count = 0;
            for _ in 0..5 {
                let old = driver.current_chord.clone();
                let decision = driver.next_chord(tension, valence, &mut rng);

                if decision.next_chord.root == old.root
                    && decision.next_chord.chord_type == old.chord_type
                {
                    same_count += 1;
                }
            }

            // Ne devrait pas rester bloqué plus de 2 fois sur 5
            assert!(
                same_count <= 2,
                "Config ({tension}, {valence}, {start_type:?}): bloqué {same_count} fois sur 5"
            );
        }
    }

    #[test]
    fn test_taboo_list_prevents_aba_loops() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        let mut rng = TestRng(333);

        // Démarrer avec un accord spécifique et historique vide
        driver.current_chord = Chord::new(4, ChordType::Major); // E Major
        driver.chord_history.clear(); // Historique vide au départ

        // Premier mouvement: E -> quelque part
        let decision1 = driver.next_chord(0.3, 0.5, &mut rng);
        let chord_after_first = decision1.next_chord;

        // Si on a bougé (pas resté sur E), le système devrait éviter de retourner immédiatement à E
        if chord_after_first.root != 4 || chord_after_first.chord_type != ChordType::Major {
            // Faire plusieurs tentatives pour voir si on retourne à E
            let mut returned_to_e = false;
            for _ in 0..5 {
                let decision = driver.next_chord(0.3, 0.5, &mut rng);
                if decision.next_chord.root == 4
                    && decision.next_chord.chord_type == ChordType::Major
                {
                    returned_to_e = true;
                    break;
                }
            }

            // On ne devrait pas retourner immédiatement à E (sauf si c'est la tonique)
            assert!(
                !returned_to_e || 4 == 0, // Permettre le retour si E est la tonique
                "Le système a créé une boucle A->B->A"
            );
        }
    }

    #[test]
    fn test_dramatic_tension_drop_forces_resolution() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        let mut rng = TestRng(444);

        // Commencer avec haute tension
        driver.current_chord = Chord::new(4, ChordType::Major); // E Major
        driver.last_tension = 0.85; // Haute tension

        // Forcer une chute dramatique de tension (0.85 -> 0.3)
        let decision = driver.next_chord(0.3, 0.5, &mut rng);

        // Le système devrait forcer une résolution vers I (C) ou V (G)
        let tonic = 0; // C
        let dominant = 7; // G

        assert!(
            decision.next_chord.root == tonic || decision.next_chord.root == dominant,
            "Après une chute dramatique de tension, l'accord devrait résoudre vers I ({}) ou V ({}), mais a résolu vers {}",
            tonic,
            dominant,
            decision.next_chord.root
        );
    }

    #[test]
    fn test_taboo_list_sliding_window() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C (tonique)

        // === Test 1: Après E -> C, retourner à E devrait être bloqué ===
        // État simulé: progression E -> C (actuel)
        driver.current_chord = Chord::new(0, ChordType::Major); // C Major (actuel)
        driver.chord_history.clear();
        driver.chord_history.push(Chord::new(4, ChordType::Major)); // E Major (dans l'historique)

        let proposed_e = Chord::new(4, ChordType::Major);
        assert!(
            driver.would_create_aba_loop(&proposed_e),
            "Après E -> C, retourner à E devrait être bloqué (E est dans la Taboo List)"
        );

        // === Test 2: Après E -> C -> F, retourner à E devrait être autorisé ===
        // Simuler la progression E -> C -> F
        // État initial: current_chord = C, chord_history = [E]

        // Transition vers F: on ajoute C à l'historique, puis on change current_chord
        driver.chord_history.push(driver.current_chord.clone()); // Ajouter C: [E, C]
        if driver.chord_history.len() > 2 {
            driver.chord_history.remove(0); // Maintenir taille 2
        }
        driver.current_chord = Chord::new(5, ChordType::Major); // F Major (nouveau actuel)

        // État après transition vers F: current_chord = F, chord_history = [E, C]
        // Pour aller vers G (un 3e mouvement), on ajoute F à l'historique et E est éjecté
        driver.chord_history.push(driver.current_chord.clone()); // Ajouter F: [E, C, F]
        if driver.chord_history.len() > 2 {
            driver.chord_history.remove(0); // Éjecter E: [C, F]
        }
        driver.current_chord = Chord::new(7, ChordType::Major); // G Major

        // État final: current_chord = G, chord_history = [C, F]
        // E devrait maintenant être autorisé car il n'est plus dans l'historique
        assert!(
            !driver.would_create_aba_loop(&proposed_e),
            "Après E -> C -> F -> G, retourner à E devrait être autorisé (E n'est plus dans la Taboo List)"
        );

        // === Test 3: La tonique (C) est toujours autorisée même dans la Taboo List ===
        let proposed_c = Chord::new(0, ChordType::Major);
        assert!(
            !driver.would_create_aba_loop(&proposed_c),
            "La tonique (C) devrait toujours être autorisée, même si elle est dans la Taboo List"
        );

        // === Test 4: Les accords dans l'historique (non-toniques) devraient être bloqués ===
        // L'historique contient actuellement [C, F]
        // F n'est pas la tonique, donc proposer F devrait être bloqué
        let proposed_f = Chord::new(5, ChordType::Major);
        assert!(
            driver.would_create_aba_loop(&proposed_f),
            "Proposer F devrait être bloqué car F est dans la Taboo List"
        );
    }

    #[test]
    fn test_taboo_list_prevents_immediate_repetition() {
        let mut driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C Major (tonique)
        let mut rng = TestRng(555);

        // Démarrer avec un accord non-tonique pour tester la Taboo List
        driver.current_chord = Chord::new(4, ChordType::Minor); // E Minor

        // Générer plusieurs accords et vérifier qu'on ne retourne pas immédiatement
        // au même accord (boucle A -> B -> A)
        let mut previous_two_chords: Vec<(u8, ChordType)> =
            vec![(driver.current_chord.root, driver.current_chord.chord_type)];

        for i in 0..10 {
            let old_chord = driver.current_chord.clone();
            let decision = driver.next_chord(0.6, 0.0, &mut rng); // Tension moyenne

            // Vérifier qu'on n'a pas créé une boucle avec les 2 derniers accords
            // (sauf si c'est la tonique qui est toujours autorisée)
            if decision.next_chord.root != 0 {
                // Si ce n'est pas la tonique
                let new_chord_sig = (decision.next_chord.root, decision.next_chord.chord_type);

                // Le nouvel accord ne devrait pas être dans les 2 derniers
                // (previous_two_chords contient au maximum les 2 derniers)
                let is_in_recent_history = previous_two_chords
                    .iter()
                    .any(|&(r, t)| r == new_chord_sig.0 && t == new_chord_sig.1);

                assert!(
                    !is_in_recent_history,
                    "Itération {}: Le système a créé une répétition immédiate: {:?} -> {:?} (historique: {:?})",
                    i,
                    (old_chord.root, old_chord.chord_type),
                    new_chord_sig,
                    previous_two_chords
                );
            }

            // Mettre à jour l'historique des 2 derniers accords
            previous_two_chords.push((old_chord.root, old_chord.chord_type));
            if previous_two_chords.len() > 2 {
                previous_two_chords.remove(0);
            }
        }
    }

    #[test]
    fn test_tonic_return_always_allowed() {
        let driver = HarmonicDriver::new(0, &HarmonyDriverParams::default()); // C
        let mut driver = driver;

        // Configurer: C -> G -> (devrait pouvoir retourner à C)
        driver.current_chord = Chord::new(7, ChordType::Major); // G Major
        driver.chord_history.push(Chord::new(0, ChordType::Major)); // C Major dans l'historique

        // Le retour à la tonique (I = C) devrait toujours être autorisé
        // même si cela crée un pattern A->B->A
        let proposed_tonic = Chord::new(0, ChordType::Major);
        let would_loop = driver.would_create_aba_loop(&proposed_tonic);

        assert!(!would_loop, "Le retour à la tonique devrait toujours être autorisé");
    }
}

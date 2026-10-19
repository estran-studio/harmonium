//! `EmotionMapper` - Traducteur Émotions → Paramètres Musicaux
//!
//! Ce module contient toute la logique de mapping qui était auparavant
//! dispersée dans `update_controls()`. C'est un composant pur, sans état,
//! facile à tester unitairement.
//!
//! ## Principe de Russell's Circumplex Model
//! ```text
//!              High Arousal
//!                   ↑
//!                   |
//!     Angry    ←────┼────→    Excited
//!  (-, high)        |        (+, high)
//!                   |
//!  Low Valence ←────┼────→ High Valence
//!                   |
//!     Sad      ←────┼────→    Relaxed
//!  (-, low)         |        (+, low)
//!                   |
//!                   ↓
//!              Low Arousal
//! ```

use crate::{
    params::{EngineParams, HarmonyStrategy, MusicalParams, TensionState},
    sequencer::RhythmMode,
};

/// Configuration du mapper (seuils, courbes, etc.)
#[derive(Clone, Debug)]
pub struct MapperConfig {
    /// BPM minimum (arousal = 0.0)
    pub bpm_min: f32,
    /// BPM maximum (arousal = 1.0)
    pub bpm_max: f32,
    /// Seuil de tension pour passer en Neo-Riemannian/Parsimonious
    pub tension_threshold_high: f32,
    /// Seuil de tension pour revenir en Steedman
    pub tension_threshold_low: f32,
    /// Seuil de tension pour changer d'accord plus vite
    pub fast_chord_change_threshold: f32,
}

impl Default for MapperConfig {
    fn default() -> Self {
        Self {
            bpm_min: 70.0,
            bpm_max: 180.0,
            tension_threshold_high: 0.7,
            tension_threshold_low: 0.5,
            fast_chord_change_threshold: 0.6,
        }
    }
}

/// Traducteur d'émotions vers paramètres musicaux
pub struct EmotionMapper {
    config: MapperConfig,
}

impl Default for EmotionMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionMapper {
    #[must_use]
    pub fn new() -> Self {
        Self { config: MapperConfig::default() }
    }

    #[must_use]
    pub const fn with_config(config: MapperConfig) -> Self {
        Self { config }
    }

    /// Traduit les paramètres émotionnels en paramètres musicaux concrets
    ///
    /// C'est la fonction principale qui contient toute la logique de mapping.
    /// Elle est pure (pas d'effets de bord) et peut être testée unitairement.
    #[must_use]
    pub fn map(&self, emotions: &EngineParams) -> MusicalParams {
        // ═══════════════════════════════════════════════════════════════════
        // BPM: Arousal → Tempo
        // ═══════════════════════════════════════════════════════════════════
        // Faible arousal = calme = tempo lent
        // Haute arousal = excité = tempo rapide
        let bpm = emotions
            .arousal
            .mul_add(self.config.bpm_max - self.config.bpm_min, self.config.bpm_min);

        let mut params = MusicalParams { bpm, ..MusicalParams::default() };

        // ═══════════════════════════════════════════════════════════════════
        // RYTHME: Density + Tension → Pattern
        // ═══════════════════════════════════════════════════════════════════
        params.rhythm_mode = emotions.algorithm;
        params.rhythm_density = emotions.density;
        params.rhythm_tension = emotions.tension;

        // Pulses primaires: density contrôle la "densité" d'accents
        // density 0.0 → 1 pulse, density 1.0 → 12 pulses (sur 16 steps)
        if emotions.algorithm == RhythmMode::Euclidean {
            params.rhythm_steps = 16;
            params.rhythm_pulses = ((emotions.density * 11.0) as usize + 1).min(16);
        } else {
            params.rhythm_steps = emotions.poly_steps;
            // PerfectBalance garde les pulses internes
        }

        // Rotation: tension contrôle le décalage de phase (syncopation)
        let max_rotation = if emotions.algorithm == RhythmMode::PerfectBalance {
            params.rhythm_steps / 2
        } else {
            8
        };
        params.rhythm_rotation = (emotions.tension * max_rotation as f32) as usize;

        // Séquenceur secondaire (polyrythme 4:3)
        params.rhythm_secondary_steps = 12;
        params.rhythm_secondary_pulses = ((emotions.density * 8.0) as usize + 1).min(12);
        params.rhythm_secondary_rotation = 8_usize.saturating_sub(params.rhythm_rotation % 8);

        // ═══════════════════════════════════════════════════════════════════
        // HARMONIE: Tension + Valence → Stratégie + Palette
        // ═══════════════════════════════════════════════════════════════════
        params.harmony_mode = emotions.harmony_mode;
        params.harmony_tension = emotions.tension;
        params.harmony_valence = emotions.valence;

        // Sélection automatique de stratégie selon tension
        params.harmony_strategy = if emotions.tension > self.config.tension_threshold_high {
            // Haute tension → transformations géométriques (plus imprévisible)
            HarmonyStrategy::NeoRiemannian
        } else if emotions.tension < self.config.tension_threshold_low {
            // Basse tension → harmonie fonctionnelle (prévisible, stable)
            HarmonyStrategy::Steedman
        } else {
            // Zone intermédiaire → parsimonieux (transition douce)
            HarmonyStrategy::Parsimonious
        };

        // Vitesse de changement d'accords
        params.harmony_measures_per_chord =
            if emotions.tension > self.config.fast_chord_change_threshold {
                1 // Changement rapide
            } else {
                2 // Changement normal
            };

        // ═══════════════════════════════════════════════════════════════════
        // MÉLODIE: Smoothness → Hurst factor
        // ═══════════════════════════════════════════════════════════════════
        params.melody_smoothness = emotions.smoothness;

        // Voicing: density + tension
        params.voicing_density = emotions.density;
        params.voicing_tension = emotions.tension;

        // Octave: arousal influence légèrement (plus aigu quand excité)
        params.melody_octave = 4 + (emotions.arousal * 0.5) as i32;

        // ═══════════════════════════════════════════════════════════════════
        // MIXER: Passthrough des gains
        // ═══════════════════════════════════════════════════════════════════
        params.gain_lead = emotions.gain_lead;
        params.gain_chords = emotions.gain_chords;
        params.gain_bass = emotions.gain_bass;
        params.gain_snare = emotions.gain_snare;
        params.gain_hat = emotions.gain_hat;
        params.vel_base_bass = emotions.vel_base_bass;
        params.vel_base_snare = emotions.vel_base_snare;

        // ═══════════════════════════════════════════════════════════════════
        // ROUTAGE & RECORDING: Passthrough
        // ═══════════════════════════════════════════════════════════════════
        params.channel_routing = emotions.channel_routing.clone();
        params.muted_channels = emotions.muted_channels.clone();
        params.record_wav = emotions.record_wav;
        params.record_midi = emotions.record_midi;
        params.record_musicxml = emotions.record_musicxml;

        // Mode Drum Kit (passthrough)
        params.fixed_kick = emotions.fixed_kick;

        params
    }

    /// Fonction statique pour usage rapide sans instancier le mapper
    #[must_use]
    pub fn quick_map(emotions: &EngineParams) -> MusicalParams {
        Self::new().map(emotions)
    }
}

/// Fonctions utilitaires pour le mapping
impl EmotionMapper {
    /// Calcule le BPM depuis l'arousal (extraction de la logique)
    #[must_use]
    pub fn arousal_to_bpm(&self, arousal: f32) -> f32 {
        arousal
            .clamp(0.0, 1.0)
            .mul_add(self.config.bpm_max - self.config.bpm_min, self.config.bpm_min)
    }

    /// Détermine la stratégie harmonique depuis la tension
    #[must_use]
    pub fn tension_to_strategy(&self, tension: f32) -> HarmonyStrategy {
        if tension > self.config.tension_threshold_high {
            HarmonyStrategy::NeoRiemannian
        } else if tension < self.config.tension_threshold_low {
            HarmonyStrategy::Steedman
        } else {
            HarmonyStrategy::Parsimonious
        }
    }

    /// Calcule les pulses depuis la density pour un nombre de steps donné
    #[must_use]
    pub fn density_to_pulses(density: f32, max_steps: usize) -> usize {
        let scale = (max_steps as f32 * 0.75).max(1.0);
        ((density * scale) as usize + 1).min(max_steps)
    }

    /// Calcule la rotation depuis la tension pour un nombre de steps donné
    #[must_use]
    pub fn tension_to_rotation(tension: f32, max_steps: usize) -> usize {
        let max_rotation = max_steps / 2;
        (tension * max_rotation as f32) as usize
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UNIFIED TENSION SYSTEM (TRQ Matrix)
// ═══════════════════════════════════════════════════════════════════════════

/// Unified Tension System - Couples harmonic and rhythmic tension via emotion
///
/// Implements the TRQ (Tension-Rhythmic-Quality) Matrix which maps emotional
/// inputs (arousal, valence, tension) to coherent combinations of:
/// - **Harmonic tension** via Lydian Chromatic Concept (LCC) levels 1-7
/// - **Rhythmic oddity** via antipodal distance (0-7+ oddity score)
///
/// ## TRQ Matrix States
/// ```text
///                 Low Oddity          High Oddity
///                 (4-5)               (6-7+)
///              ┌───────────────────┬──────────────────┐
/// Low LCC      │  Stability (A)    │ RhythmicDrive(C)│
/// (1-3)        │  Calm, consonant  │ Driving, groove │
///              ├───────────────────┼──────────────────┤
/// High LCC     │ HarmonicSuspense  │ PeakClimax (D)  │
/// (4-7)        │ (B) Tense harmony │ Maximum tension │
///              └───────────────────┴──────────────────┘
/// ```
///
/// ## Emotional Mapping
/// - **High arousal + Low valence** → State D (PeakClimax) — Angry, chaotic
/// - **High arousal + High valence** → State C (RhythmicDrive) — Excited, energetic
/// - **Low arousal + Low valence** → State B (HarmonicSuspense) — Sad, melancholic
/// - **Low arousal + High valence** → State A (Stability) — Relaxed, peaceful
#[derive(Clone, Debug)]
pub struct UnifiedTensionSystem {
    /// Current arousal level (0.0 to 1.0)
    pub arousal: f32,
    /// Current valence level (-1.0 to 1.0, negative = minor, positive = major)
    pub valence: f32,
    /// Current tension level (0.0 to 1.0)
    pub tension: f32,
    /// Current tension state (derived from arousal/valence/tension)
    pub tension_state: TensionState,
}

impl Default for UnifiedTensionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedTensionSystem {
    /// Create a new Unified Tension System with default values (Stability state)
    #[must_use]
    pub fn new() -> Self {
        Self { arousal: 0.0, valence: 0.0, tension: 0.0, tension_state: TensionState::Stability }
    }

    /// Update the system with new emotional inputs and recalculate tension state
    pub fn update(&mut self, arousal: f32, valence: f32, tension: f32) {
        self.arousal = arousal.clamp(0.0, 1.0);
        self.valence = valence.clamp(-1.0, 1.0);
        self.tension = tension.clamp(0.0, 1.0);
        self.tension_state = Self::select_tension_state(self.arousal, self.valence, self.tension);
    }

    /// Select tension state from emotional inputs using Russell's Circumplex Model
    ///
    /// Maps (arousal, valence, tension) to one of four TRQ Matrix states:
    /// - **High arousal + Low valence** → PeakClimax (Angry, chaotic)
    /// - **High arousal + High valence** → RhythmicDrive (Excited, energetic)
    /// - **Low arousal + Low valence** → HarmonicSuspense (Sad, melancholic)
    /// - **Low arousal + High valence** → Stability (Relaxed, peaceful)
    ///
    /// The tension parameter fine-tunes the selection within each quadrant.
    #[must_use]
    pub fn select_tension_state(arousal: f32, valence: f32, tension: f32) -> TensionState {
        // Normalize inputs
        let arousal = arousal.clamp(0.0, 1.0);
        let valence = valence.clamp(-1.0, 1.0);
        let tension = tension.clamp(0.0, 1.0);

        // Determine quadrant from arousal and valence (Russell's Circumplex)
        let high_arousal = arousal > 0.5;
        let high_valence = valence > 0.0;

        // Map to TRQ Matrix state
        // Tension biases towards higher-tension states within each quadrant
        match (high_arousal, high_valence) {
            (true, false) => {
                // High arousal, low valence (angry, stressed)
                // Bias: always PeakClimax (maximum tension)
                TensionState::PeakClimax
            }
            (true, true) => {
                // High arousal, high valence (excited, energetic)
                // High tension → PeakClimax, Low tension → RhythmicDrive
                if tension > 0.6 { TensionState::PeakClimax } else { TensionState::RhythmicDrive }
            }
            (false, false) => {
                // Low arousal, low valence (sad, melancholic)
                // High tension → HarmonicSuspense, Low tension → Stability
                if tension > 0.5 { TensionState::HarmonicSuspense } else { TensionState::Stability }
            }
            (false, true) => {
                // Low arousal, high valence (relaxed, peaceful)
                // Bias: always Stability (low tension)
                TensionState::Stability
            }
        }
    }

    /// Calculate Lydian Chromatic Concept (LCC) level from current tension state
    ///
    /// Returns LCC level (1-7) based on the TRQ Matrix state:
    /// - **Stability**: LCC 1-3 (Ingoing: I, IV, bVII)
    /// - **RhythmicDrive**: LCC 1-3 (Ingoing: consonant harmony)
    /// - **HarmonicSuspense**: LCC 4-7 (Outgoing: II, bII, VI, #IV)
    /// - **PeakClimax**: LCC 4-7 (Outgoing: maximum dissonance)
    ///
    /// Within each range, the exact level is determined by the tension parameter.
    #[must_use]
    pub fn calculate_lcc_level(&self) -> usize {
        match self.tension_state {
            TensionState::Stability | TensionState::RhythmicDrive => {
                // Ingoing scale degrees (1-3): I, IV, bVII
                // Map tension 0.0-1.0 to LCC 1-3
                (self.tension * 2.0) as usize + 1
            }
            TensionState::HarmonicSuspense | TensionState::PeakClimax => {
                // Outgoing scale degrees (4-7): II, bII, VI, #IV
                // Map tension 0.0-1.0 to LCC 4-7
                (self.tension * 3.0) as usize + 4
            }
        }
    }

    /// Calculate target rhythmic oddity from current tension state
    ///
    /// Returns oddity score (4-7+) based on the TRQ Matrix state:
    /// - **Stability**: Oddity 4-5 (low syncopation, stable patterns)
    /// - **HarmonicSuspense**: Oddity 4-5 (stable rhythm, tense harmony)
    /// - **RhythmicDrive**: Oddity 6-7 (high syncopation, driving groove)
    /// - **PeakClimax**: Oddity 6-7+ (maximum rhythmic complexity)
    ///
    /// Within each range, the exact oddity is determined by the tension parameter.
    #[must_use]
    pub fn calculate_target_oddity(&self) -> usize {
        match self.tension_state {
            TensionState::Stability | TensionState::HarmonicSuspense => {
                // Low oddity (4-5): stable, predictable rhythms
                // Map tension 0.0-1.0 to oddity 4-5
                (self.tension * 1.0) as usize + 4
            }
            TensionState::RhythmicDrive | TensionState::PeakClimax => {
                // High oddity (6-7+): syncopated, complex rhythms
                // Map tension 0.0-1.0 to oddity 6-7
                (self.tension * 1.0) as usize + 6
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arousal_to_bpm() {
        let mapper = EmotionMapper::new();

        // Arousal 0.0 = BPM minimum
        assert_eq!(mapper.arousal_to_bpm(0.0), 70.0);

        // Arousal 1.0 = BPM maximum
        assert_eq!(mapper.arousal_to_bpm(1.0), 180.0);

        // Arousal 0.5 = BPM moyen
        assert_eq!(mapper.arousal_to_bpm(0.5), 125.0);
    }

    #[test]
    fn test_tension_to_strategy() {
        let mapper = EmotionMapper::new();

        // Basse tension → Steedman
        assert_eq!(mapper.tension_to_strategy(0.3), HarmonyStrategy::Steedman);

        // Haute tension → NeoRiemannian
        assert_eq!(mapper.tension_to_strategy(0.8), HarmonyStrategy::NeoRiemannian);

        // Tension moyenne → Parsimonious
        assert_eq!(mapper.tension_to_strategy(0.6), HarmonyStrategy::Parsimonious);
    }

    #[test]
    fn test_full_mapping() {
        let emotions = EngineParams {
            arousal: 0.7, // → ~147 BPM
            valence: 0.5, // Major
            density: 0.4, // Medium density
            tension: 0.3, // Low tension → Steedman
            smoothness: 0.7,
            ..Default::default()
        };

        let params = EmotionMapper::quick_map(&emotions);

        // Vérifier le BPM
        assert!((params.bpm - 147.0).abs() < 1.0);

        // Vérifier la stratégie (basse tension)
        assert_eq!(params.harmony_strategy, HarmonyStrategy::Steedman);

        // Vérifier que les modules sont actifs par défaut
        assert!(params.enable_rhythm);
        assert!(params.enable_harmony);
        assert!(params.enable_melody);
    }

    #[test]
    fn test_density_to_pulses() {
        // 16 steps, density 0 → 1 pulse
        assert_eq!(EmotionMapper::density_to_pulses(0.0, 16), 1);

        // 16 steps, density 1 → 12 pulses (75% max)
        assert_eq!(EmotionMapper::density_to_pulses(1.0, 16), 13);

        // 48 steps, density 0.5 → ~18 pulses
        let pulses = EmotionMapper::density_to_pulses(0.5, 48);
        assert!(pulses > 10 && pulses < 30);
    }

    #[test]
    fn test_custom_config() {
        let config = MapperConfig { bpm_min: 60.0, bpm_max: 200.0, ..Default::default() };
        let mapper = EmotionMapper::with_config(config);

        assert_eq!(mapper.arousal_to_bpm(0.0), 60.0);
        assert_eq!(mapper.arousal_to_bpm(1.0), 200.0);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UNIFIED TENSION SYSTEM TESTS (Phase 5)
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_select_tension_state_stability() {
        // Low arousal + high valence → Stability
        let state = UnifiedTensionSystem::select_tension_state(0.2, 0.5, 0.3);
        assert_eq!(state, TensionState::Stability);

        // Also low arousal + low valence + low tension → Stability
        let state = UnifiedTensionSystem::select_tension_state(0.3, -0.3, 0.2);
        assert_eq!(state, TensionState::Stability);
    }

    #[test]
    fn test_select_tension_state_harmonic_suspense() {
        // Low arousal + low valence + high tension → HarmonicSuspense
        let state = UnifiedTensionSystem::select_tension_state(0.2, -0.7, 0.8);
        assert_eq!(state, TensionState::HarmonicSuspense);
    }

    #[test]
    fn test_select_tension_state_rhythmic_drive() {
        // High arousal + high valence + low tension → RhythmicDrive
        let state = UnifiedTensionSystem::select_tension_state(0.8, 0.7, 0.3);
        assert_eq!(state, TensionState::RhythmicDrive);
    }

    #[test]
    fn test_select_tension_state_peak_climax() {
        // High arousal + low valence → PeakClimax (always)
        let state = UnifiedTensionSystem::select_tension_state(0.9, -0.8, 0.5);
        assert_eq!(state, TensionState::PeakClimax);

        // High arousal + high valence + high tension → PeakClimax
        let state = UnifiedTensionSystem::select_tension_state(0.9, 0.8, 0.9);
        assert_eq!(state, TensionState::PeakClimax);
    }

    #[test]
    fn test_calculate_lcc_level_ingoing() {
        // Stability → Ingoing (LCC 1-3)
        let mut system = UnifiedTensionSystem::new();
        system.update(0.2, 0.5, 0.0);
        let lcc = system.calculate_lcc_level();
        assert!((1..=3).contains(&lcc), "Stability should map to LCC 1-3, got {}", lcc);

        // RhythmicDrive → Ingoing (LCC 1-3)
        system.update(0.8, 0.7, 0.3);
        let lcc = system.calculate_lcc_level();
        assert!((1..=3).contains(&lcc), "RhythmicDrive should map to LCC 1-3, got {}", lcc);
    }

    #[test]
    fn test_calculate_lcc_level_outgoing() {
        // HarmonicSuspense → Outgoing (LCC 4-7)
        let mut system = UnifiedTensionSystem::new();
        system.update(0.2, -0.7, 0.8);
        let lcc = system.calculate_lcc_level();
        assert!((4..=7).contains(&lcc), "HarmonicSuspense should map to LCC 4-7, got {}", lcc);

        // PeakClimax → Outgoing (LCC 4-7)
        system.update(0.9, -0.8, 0.9);
        let lcc = system.calculate_lcc_level();
        assert!((4..=7).contains(&lcc), "PeakClimax should map to LCC 4-7, got {}", lcc);
    }

    #[test]
    fn test_calculate_target_oddity_low() {
        // Stability → Low oddity (4-5)
        let mut system = UnifiedTensionSystem::new();
        system.update(0.2, 0.5, 0.0);
        let oddity = system.calculate_target_oddity();
        assert!((4..=5).contains(&oddity), "Stability should map to oddity 4-5, got {}", oddity);

        // HarmonicSuspense → Low oddity (4-5)
        system.update(0.2, -0.7, 0.8);
        let oddity = system.calculate_target_oddity();
        assert!(
            (4..=5).contains(&oddity),
            "HarmonicSuspense should map to oddity 4-5, got {}",
            oddity
        );
    }

    #[test]
    fn test_calculate_target_oddity_high() {
        // RhythmicDrive → High oddity (6-7)
        let mut system = UnifiedTensionSystem::new();
        system.update(0.8, 0.7, 0.3);
        let oddity = system.calculate_target_oddity();
        assert!(
            (6..=7).contains(&oddity),
            "RhythmicDrive should map to oddity 6-7, got {}",
            oddity
        );

        // PeakClimax → High oddity (6-7)
        system.update(0.9, -0.8, 0.9);
        let oddity = system.calculate_target_oddity();
        assert!((6..=7).contains(&oddity), "PeakClimax should map to oddity 6-7, got {}", oddity);
    }

    #[test]
    fn test_unified_tension_update() {
        let mut system = UnifiedTensionSystem::new();

        // Start in Stability
        assert_eq!(system.tension_state, TensionState::Stability);

        // Update to PeakClimax
        system.update(0.9, -0.8, 0.9);
        assert_eq!(system.tension_state, TensionState::PeakClimax);
        assert!((4..=7).contains(&system.calculate_lcc_level()));
        assert!((6..=7).contains(&system.calculate_target_oddity()));

        // Update to Stability
        system.update(0.2, 0.5, 0.1);
        assert_eq!(system.tension_state, TensionState::Stability);
        assert!((1..=3).contains(&system.calculate_lcc_level()));
        assert!((4..=5).contains(&system.calculate_target_oddity()));
    }
}

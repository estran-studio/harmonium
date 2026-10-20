//! Style tuning parameters — the ~83 "personality constants" that define how
//! the engine interprets real-time user controls (density, tension, valence).
//!
//! `TuningParams` holds sensible defaults matching every current hardcoded
//! constant in the engine. Loading a `StyleProfile` applies a partial
//! `TuningOverlay` on top of these defaults.

use serde::{Deserialize, Serialize};

use crate::{
    harmony::steedman_grammar::GrammarStyle,
    params::TimeSignature,
    trad::{cells::CellBank, feet::FeetTable, harmony::ChordDegree},
};

/// Version du schéma et des constantes de `TuningParams` (constitution I).
///
/// Incrémentée à chaque rupture assumée du contrat de snapshot (p. ex. D2 :
/// renommage `TrackId { Melody, Chords, Bass, Heel, Toe }` et banque de
/// cellules trad en phase 1).
pub const TUNING_VERSION: u32 = 4;

// ---------------------------------------------------------------------------
// Param sub-structs (Layer 2 — style constants)
// ---------------------------------------------------------------------------

/// Harmony strategy selection thresholds (driver.rs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HarmonyDriverParams {
    /// Lower hysteresis bound for Steedman strategy.
    pub steedman_lower: f32,
    /// Upper hysteresis bound for Steedman strategy.
    pub steedman_upper: f32,
    /// Lower hysteresis bound for Neo-Riemannian strategy.
    pub neo_lower: f32,
    /// Upper hysteresis bound for Neo-Riemannian strategy.
    pub neo_upper: f32,
    /// Bias added to previous strategy weight for stability.
    pub hysteresis_boost: f32,
    /// Tension above which a drop to below `dramatic_tension_drop_lower`
    /// triggers cadential resolution.
    pub dramatic_tension_drop_upper: f32,
    /// Lower bound for dramatic tension drop detection.
    pub dramatic_tension_drop_lower: f32,
    /// Probability of resolving directly to tonic (vs V-I).
    pub cadential_resolution_probability: f32,
    /// Max chord generation retries to avoid A-B-A loops.
    pub max_retries: usize,
}

impl Default for HarmonyDriverParams {
    fn default() -> Self {
        Self {
            steedman_lower: 0.45,
            steedman_upper: 0.55,
            neo_lower: 0.65,
            neo_upper: 0.75,
            hysteresis_boost: 0.1,
            dramatic_tension_drop_upper: 0.7,
            dramatic_tension_drop_lower: 0.5,
            cadential_resolution_probability: 0.6,
            max_retries: 3,
        }
    }
}

/// Grammar rule weight multipliers and style (steedman_grammar.rs).
///
/// Weight fields act as multipliers on the built-in per-`GrammarStyle` weights.
/// Default 1.0 means "use the style's weight as-is".
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GrammarParams {
    /// Base grammar style preset.
    pub grammar_style: GrammarStyle,
    /// Max recursion depth for back-cycling.
    pub max_recursion_depth: u8,
    /// Multiplier for ii-V preparation rules.
    pub preparation_weight: f32,
    /// Multiplier for back-cycle rules.
    pub backcycle_weight: f32,
    /// Multiplier for tritone substitution rules.
    pub tritone_sub_weight: f32,
    /// Multiplier for cadential resolution rules.
    pub cadential_weight: f32,
    /// Multiplier for deceptive cadence rules.
    pub deceptive_weight: f32,
    /// Multiplier for modal interchange rules.
    pub modal_interchange_weight: f32,
    /// Valence threshold for chord quality decisions (Major7 vs Major).
    pub chord_quality_valence_threshold: f32,
}

impl Default for GrammarParams {
    fn default() -> Self {
        Self {
            grammar_style: GrammarStyle::default(),
            max_recursion_depth: 2,
            preparation_weight: 1.0,
            backcycle_weight: 1.0,
            tritone_sub_weight: 1.0,
            cadential_weight: 1.0,
            deceptive_weight: 1.0,
            modal_interchange_weight: 1.0,
            chord_quality_valence_threshold: 0.3,
        }
    }
}

/// Neo-Riemannian P/L/R operation probabilities per valence zone.
///
/// Probability fields are cumulative thresholds (e.g. `positive_r_prob` = 0.5
/// means R chosen when `rand < 0.5`, P when `rand < positive_p_cumulative`,
/// else L).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeoRiemannianParams {
    /// Valence above which positive-zone probabilities apply.
    pub positive_valence_threshold: f32,
    /// Valence below which negative-zone probabilities apply.
    pub negative_valence_threshold: f32,
    /// Cumulative probability of R in positive valence zone.
    pub positive_r_prob: f32,
    /// Cumulative probability of P in positive valence zone.
    pub positive_p_cumulative: f32,
    /// Cumulative probability of L in negative valence zone.
    pub negative_l_prob: f32,
    /// Cumulative probability of P in negative valence zone.
    pub negative_p_cumulative: f32,
    /// Cumulative probability of P in neutral valence zone.
    pub neutral_p_prob: f32,
    /// Cumulative probability of L in neutral valence zone.
    pub neutral_l_cumulative: f32,
    /// Tension above which composite operations may fire.
    pub composite_tension_threshold: f32,
    /// Probability of using composite op when above threshold.
    pub composite_probability: f32,
}

impl Default for NeoRiemannianParams {
    fn default() -> Self {
        Self {
            positive_valence_threshold: 0.3,
            negative_valence_threshold: -0.3,
            positive_r_prob: 0.5,
            positive_p_cumulative: 0.8,
            negative_l_prob: 0.5,
            negative_p_cumulative: 0.8,
            neutral_p_prob: 0.4,
            neutral_l_cumulative: 0.7,
            composite_tension_threshold: 0.8,
            composite_probability: 0.5,
        }
    }
}

/// Voice-leading / parsimonious movement params (parsimonious.rs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceLeadingParams {
    /// Max semitone movement per voice in neighbor search.
    pub max_semitone_movement: u8,
    /// Allow chord cardinality changes (triad <-> seventh).
    pub allow_cardinality_morph: bool,
    /// Default TRQ threshold for neighbor filtering.
    pub trq_threshold: f32,
    /// Tension above which high-tension neighbor filtering applies.
    pub high_tension_threshold: f32,
    /// Tension below which low-tension neighbor filtering applies.
    pub low_tension_threshold: f32,
}

impl Default for VoiceLeadingParams {
    fn default() -> Self {
        Self {
            max_semitone_movement: 2,
            allow_cardinality_morph: true,
            trq_threshold: 0.5,
            high_tension_threshold: 0.6,
            low_tension_threshold: 0.4,
        }
    }
}

/// Melody generation params (melody.rs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MelodyParams {
    /// Pink noise octave depth for fractal contour.
    pub pink_noise_depth: usize,
    /// Hurst exponent controlling fractal drift smoothness.
    pub default_hurst_factor: f32,
    /// Semitone leap size that triggers Temperley gap-fill.
    pub gap_fill_threshold: i32,
    /// Probability of new material vs motif replay (0.0 = all replay, 1.0 = all new).
    pub motif_new_material_bias: f32,
    /// Weight multiplier for steps toward fractal target.
    pub fractal_boost: f32,
    /// Target amplitude in scale degrees for fractal drift.
    pub fractal_range: f32,
    /// Max consecutive same-direction steps before forced reversal.
    pub consecutive_direction_limit: i32,
    /// Weight for leading-tone resolution (+1 step) in step tables.
    pub leading_tone_resolution_weight: u32,
}

impl Default for MelodyParams {
    fn default() -> Self {
        Self {
            pink_noise_depth: 5,
            default_hurst_factor: 0.7,
            gap_fill_threshold: 5,
            motif_new_material_bias: 0.6,
            fractal_boost: 1.8,
            fractal_range: 22.0,
            consecutive_direction_limit: 6,
            leading_tone_resolution_weight: 50,
        }
    }
}

/// Classic groove rhythm params (sequencer.rs — ClassicGroove mode).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClassicGrooveParams {
    /// Density thresholds selecting kick patterns [half-time, secondary, straight, anticipation].
    pub kick_density_thresholds: [f32; 4],
    /// Velocity for kick downbeat (beat 1).
    pub kick_downbeat_velocity: f32,
    /// Velocity for secondary kick beats.
    pub kick_secondary_velocity: f32,
    /// Velocity for anticipation ("2-and") kick.
    pub kick_anticipation_velocity: f32,
    /// Tension threshold above which ghost notes appear.
    pub ghost_note_tension_threshold: f32,
    /// Velocity for ghost snare notes.
    pub ghost_note_velocity: f32,
    /// Density thresholds selecting hat patterns [sparse, standard, dense].
    pub hat_density_thresholds: [f32; 3],
    /// Hat velocity on strong eighth-note beats.
    pub hat_on_beat_velocity: f32,
    /// Hat velocity on weak eighth-note beats.
    pub hat_off_beat_velocity: f32,
    /// Hat velocity on strong beats in dense (16th) pattern.
    pub hat_dense_on_velocity: f32,
    /// Hat velocity on "and" beats in dense pattern.
    pub hat_dense_off_velocity: f32,
    /// Hat velocity on ghost 16th notes.
    pub hat_dense_ghost_velocity: f32,
    /// Hat velocity in sparse (off-beat only) pattern.
    pub hat_sparse_velocity: f32,
    /// Density below which hat is masked when kick/snare present.
    pub hat_masking_density_threshold: f32,
    /// Tension threshold for bass: below = follow kick, above = independent.
    pub bass_split_tension_threshold: f32,
    /// Snare backbeat velocity.
    pub snare_backbeat_velocity: f32,
}

impl Default for ClassicGrooveParams {
    fn default() -> Self {
        Self {
            kick_density_thresholds: [0.25, 0.4, 0.6, 0.8],
            kick_downbeat_velocity: 1.0,
            kick_secondary_velocity: 0.85,
            kick_anticipation_velocity: 0.7,
            ghost_note_tension_threshold: 0.3,
            ghost_note_velocity: 0.25,
            hat_density_thresholds: [0.3, 0.6, 0.85],
            hat_on_beat_velocity: 0.6,
            hat_off_beat_velocity: 0.4,
            hat_dense_on_velocity: 0.65,
            hat_dense_off_velocity: 0.45,
            hat_dense_ghost_velocity: 0.3,
            hat_sparse_velocity: 0.5,
            hat_masking_density_threshold: 0.75,
            bass_split_tension_threshold: 0.4,
            snare_backbeat_velocity: 1.0,
        }
    }
}

/// Perfect-balance polygon rhythm params (sequencer.rs — PerfectBalance mode).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PerfectBalanceParams {
    /// Density below which kick uses digon (2 vertices) instead of square.
    pub kick_polygon_low_threshold: f32,
    /// Kick velocity in low-density (digon) mode.
    pub kick_low_velocity: f32,
    /// Kick velocity in normal density mode.
    pub kick_normal_velocity: f32,
    /// Snare polygon velocity.
    pub snare_velocity: f32,
    /// Hat vertex counts per density band [very_low, mid, high, very_high].
    pub hat_vertex_counts: [usize; 4],
    /// Density thresholds for hat vertex band selection [low, mid, high].
    pub hat_density_thresholds: [f32; 3],
    /// Coefficient for hat velocity scaling (multiplied by density).
    pub hat_velocity_coefficient: f32,
    /// Tension above which hat offset/swing is applied.
    pub swing_tension_threshold: f32,
    /// Scaling factor for swing offset magnitude.
    pub swing_scaling_factor: f32,
    /// Bass polygon velocity.
    pub bass_polygon_velocity: f32,
    /// Density below which bass follows kick pattern.
    pub bass_low_density_threshold: f32,
    /// Lead polygon velocity.
    pub lead_polygon_velocity: f32,
    /// Density below which hat is masked when kick/snare present.
    pub hat_masking_density_threshold: f32,
    /// Per-vertex accent pattern for kick polygon (empty = uniform).
    #[serde(default)]
    pub kick_accent_pattern: crate::sequencer::AccentPattern,
    /// Per-vertex accent pattern for snare polygon (empty = uniform).
    #[serde(default)]
    pub snare_accent_pattern: crate::sequencer::AccentPattern,
    /// Per-vertex accent pattern for hat polygon (empty = uniform).
    #[serde(default)]
    pub hat_accent_pattern: crate::sequencer::AccentPattern,
    /// Per-vertex accent pattern for bass polygon (empty = uniform).
    #[serde(default)]
    pub bass_accent_pattern: crate::sequencer::AccentPattern,
    /// Per-vertex accent pattern for lead polygon (empty = uniform).
    #[serde(default)]
    pub lead_accent_pattern: crate::sequencer::AccentPattern,
    /// Extra polygons merged with the primary kick polygon.
    #[serde(default)]
    pub kick_extra_polygons: Vec<crate::sequencer::PolygonConfig>,
    /// Extra polygons merged with the primary snare polygon.
    #[serde(default)]
    pub snare_extra_polygons: Vec<crate::sequencer::PolygonConfig>,
    /// Extra polygons merged with the primary hat polygon.
    #[serde(default)]
    pub hat_extra_polygons: Vec<crate::sequencer::PolygonConfig>,
    /// Extra polygons merged with the primary bass polygon.
    #[serde(default)]
    pub bass_extra_polygons: Vec<crate::sequencer::PolygonConfig>,
    /// Extra polygons merged with the primary lead polygon.
    #[serde(default)]
    pub lead_extra_polygons: Vec<crate::sequencer::PolygonConfig>,
}

impl Default for PerfectBalanceParams {
    fn default() -> Self {
        Self {
            kick_polygon_low_threshold: 0.3,
            kick_low_velocity: 1.0,
            kick_normal_velocity: 1.0,
            snare_velocity: 0.9,
            hat_vertex_counts: [6, 8, 12, 16],
            hat_density_thresholds: [0.25, 0.6, 0.85],
            hat_velocity_coefficient: 0.6,
            swing_tension_threshold: 0.3,
            swing_scaling_factor: 0.5,
            bass_polygon_velocity: 0.8,
            bass_low_density_threshold: 0.4,
            lead_polygon_velocity: 0.7,
            hat_masking_density_threshold: 0.75,
            kick_accent_pattern: crate::sequencer::AccentPattern::empty(),
            snare_accent_pattern: crate::sequencer::AccentPattern::empty(),
            hat_accent_pattern: crate::sequencer::AccentPattern::empty(),
            bass_accent_pattern: crate::sequencer::AccentPattern::empty(),
            lead_accent_pattern: crate::sequencer::AccentPattern::empty(),
            kick_extra_polygons: Vec::new(),
            snare_extra_polygons: Vec::new(),
            hat_extra_polygons: Vec::new(),
            bass_extra_polygons: Vec::new(),
            lead_extra_polygons: Vec::new(),
        }
    }
}

/// Timeline arrangement and dynamics params (generator.rs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArrangementParams {
    /// Steps from bar end that constitute the fill zone (tom fills, lead muting).
    pub fill_zone_size: usize,
    /// MIDI velocity for crash cymbals.
    pub crash_velocity: u8,
    /// Multiplier for ghost-note velocity relative to normal.
    pub ghost_velocity_factor: f32,
    /// Multiplier boosting tom fill velocity.
    pub tom_velocity_boost: f32,
    /// Measures per chord when tension < first threshold (slow/ambient).
    pub progression_switch_interval_slow: usize,
    /// Measures per chord at moderate tension.
    pub progression_switch_interval_normal: usize,
    /// Measures per chord when tension >= second threshold (fast).
    pub progression_switch_interval_fast: usize,
    /// Tension thresholds [slow_to_normal, normal_to_fast] for harmonic rhythm.
    pub progression_tension_thresholds: [f32; 2],
    /// Hysteresis for valence/tension palette rescan.
    pub measures_per_chord_hysteresis: f32,
    /// Tension above which "high tension" context flag is set.
    pub energy_high_tension: f32,
    /// Density above which "high density" context flag is set.
    pub energy_high_density: f32,
    /// Arousal above which crash cymbals are activated.
    pub energy_high_arousal: f32,
}

impl Default for ArrangementParams {
    fn default() -> Self {
        Self {
            fill_zone_size: 4,
            crash_velocity: 110,
            ghost_velocity_factor: 0.65,
            tom_velocity_boost: 1.1,
            progression_switch_interval_slow: 3,
            progression_switch_interval_normal: 2,
            progression_switch_interval_fast: 1,
            progression_tension_thresholds: [0.25, 0.5],
            measures_per_chord_hysteresis: 0.4,
            energy_high_tension: 0.6,
            energy_high_density: 0.6,
            energy_high_arousal: 0.7,
        }
    }
}

/// Emotional quadrant thresholds for basic harmony selection (basic.rs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmotionalQuadrantParams {
    /// Valence above which "happy" progressions are selected.
    pub happy_valence_threshold: f32,
    /// Valence below which "sad" progressions are selected.
    pub sad_valence_threshold: f32,
    /// Tension above which "energetic" variants are used.
    pub energetic_tension_threshold: f32,
}

impl Default for EmotionalQuadrantParams {
    fn default() -> Self {
        Self {
            happy_valence_threshold: 0.3,
            sad_valence_threshold: -0.3,
            energetic_tension_threshold: 0.6,
        }
    }
}

/// Paramètres de planification, de style et de ligne de violon du reel traditionnel (D3, D4, D6, constitution V).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReelFiddleParams {
    /// Tempo nominal par défaut du style Reel à la noire (`230.0`, D3).
    #[serde(default = "default_reel_bpm")]
    pub default_bpm: f32,
    /// Ratio de swing par défaut du style Reel (`0.56`, D3).
    #[serde(default = "default_reel_swing_ratio")]
    pub default_swing_ratio: f32,
    /// Signature rythmique par défaut du style Reel (`2/2`, D3).
    #[serde(default = "default_reel_time_signature")]
    pub default_time_signature: TimeSignature,
    /// Poids de tirage pour Ré majeur (`D Ionian`, D4).
    pub weight_d_ionian: u32,
    /// Poids de tirage pour Sol majeur (`G Ionian`, D4).
    pub weight_g_ionian: u32,
    /// Poids de tirage pour La majeur (`A Ionian`, D4).
    pub weight_a_ionian: u32,
    /// Poids de tirage pour La mixolydien (`A Mixolydian`, D4).
    pub weight_a_mixolydian: u32,
    /// Poids de tirage pour Ré mixolydien (`D Mixolydian`, D4).
    pub weight_d_mixolydian: u32,
    /// Poids de tirage du rôle libre `CellRole::Run` (D6).
    pub weight_role_run: u32,
    /// Poids de tirage du rôle libre `CellRole::Arp` (D6).
    pub weight_role_arp: u32,
    /// Poids de tirage du rôle libre `CellRole::Turn` (D6).
    pub weight_role_turn: u32,
    /// Écart diatonique maximal (en pas) pour la continuité stricte entre mesures (2 = tierce, D6).
    pub max_continuity_step_distance: i8,
    /// Nombre maximal d'essais `N` de tirage pour satisfaire la continuité avant repli (D6).
    pub continuity_max_retries: usize,
    /// Hauteur moyenne visée de la mélodie en MIDI (`71` = si4, P1-T7).
    pub melody_center_midi: u8,
    /// Surélévation de la strain B par rapport à la strain A, en demi-tons (`2`, P1-T7).
    pub strain_b_lift_semitones: i8,
    /// Borne inférieure MIDI pour le choix de l'octave de la tonique (`66` = F#4, P1-T7).
    pub tonic_midi_min: u8,
    /// Borne supérieure MIDI pour le choix de l'octave de la tonique (`77` = F5, P1-T7).
    pub tonic_midi_max: u8,
    /// Borne inférieure de la tessiture du violon (`55` = G3, D6).
    pub tessitura_min: u8,
    /// Borne supérieure de la tessiture du violon (`88` = E6, P1-T7).
    pub tessitura_max: u8,
    /// Vélocité MIDI de base d'une croche de violon (`84`, D6).
    pub base_velocity: u8,
    /// Accent de vélocité sur les temps forts (croches 1 et 5 : `+12`, D6).
    pub strong_eighth_velocity_offset: i8,
    /// Atténuation de vélocité sur les croches paires (`-10`, D6).
    pub even_eighth_velocity_offset: i8,
}

const fn default_reel_bpm() -> f32 {
    230.0
}

const fn default_reel_swing_ratio() -> f32 {
    0.56
}

const fn default_reel_time_signature() -> TimeSignature {
    TimeSignature::new(2, 2)
}

impl Default for ReelFiddleParams {
    fn default() -> Self {
        Self {
            default_bpm: default_reel_bpm(),
            default_swing_ratio: default_reel_swing_ratio(),
            default_time_signature: default_reel_time_signature(),
            weight_d_ionian: 3,
            weight_g_ionian: 3,
            weight_a_ionian: 2,
            weight_a_mixolydian: 1,
            weight_d_mixolydian: 1,
            weight_role_run: 4,
            weight_role_arp: 3,
            weight_role_turn: 3,
            max_continuity_step_distance: 2,
            continuity_max_retries: 16,
            melody_center_midi: 71,
            strain_b_lift_semitones: 2,
            tonic_midi_min: 66,
            tonic_midi_max: 77,
            tessitura_min: 55,
            tessitura_max: 88,
            base_velocity: 84,
            strong_eighth_velocity_offset: 12,
            even_eighth_velocity_offset: -10,
        }
    }
}

/// Harmonisation et accompagnement du reel (D7, D8) : poids de vote, ordre de
/// priorité des égalités, rythme et tessiture de la basse et de la guitare.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TradHarmonyParams {
    /// Pas de la demi-mesure sur lesquels les notes comptent double (`[0, 8]`).
    pub strong_steps: Vec<usize>,
    /// Poids de vote d'une note placée sur un temps fort (défaut 2).
    pub strong_weight: u32,
    /// Poids de vote d'une note placée ailleurs (défaut 1).
    pub weak_weight: u32,
    /// Ordre de priorité en cas d'égalité de score (`[I, V, IV]`) ; les degrés
    /// absents passent après, dans l'ordre des accords candidats du mode.
    pub tie_priority: Vec<ChordDegree>,
    /// Pas de la basse dans la mesure (`[0, 8]` : fondamentale puis quinte).
    pub bass_steps: Vec<usize>,
    /// Durée d'une note de basse en pas (4 = noire en 2/2 à 16 pas).
    pub bass_duration_steps: usize,
    /// Vélocité de la basse.
    pub bass_velocity: u8,
    /// Note MIDI la plus grave de la basse (40 = E2).
    pub bass_low: u8,
    /// Note MIDI la plus aiguë de la basse (50 = D3).
    pub bass_high: u8,
    /// Pas des « chops » de guitare (`[4, 12]`).
    pub chop_steps: Vec<usize>,
    /// Durée d'un chop en pas (1).
    pub chop_duration_steps: usize,
    /// Vélocité accentuée d'un chop.
    pub chop_velocity: u8,
    /// Note MIDI autour de laquelle se placent les voicings (60 = C4).
    pub guitar_center: u8,
    /// Écart maximal, en demi-tons, entre le centre du voicing et
    /// `guitar_center` (borne la dérive du voice-leading).
    pub guitar_max_drift: u8,
}

impl Default for TradHarmonyParams {
    fn default() -> Self {
        Self {
            strong_steps: vec![0, 8],
            strong_weight: 2,
            weak_weight: 1,
            tie_priority: vec![ChordDegree::I, ChordDegree::V, ChordDegree::IV],
            bass_steps: vec![0, 8],
            bass_duration_steps: 4,
            bass_velocity: 90,
            bass_low: 40,
            bass_high: 50,
            chop_steps: vec![4, 12],
            chop_duration_steps: 1,
            chop_velocity: 100,
            guitar_center: 60,
            guitar_max_drift: 6,
        }
    }
}

// ---------------------------------------------------------------------------
// Root TuningParams
// ---------------------------------------------------------------------------

/// All style personality constants. Defaults match the current hardcoded values
/// so loading `TuningParams::default()` produces identical engine behaviour.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TuningParams {
    pub harmony_driver: HarmonyDriverParams,
    pub grammar: GrammarParams,
    pub neo_riemannian: NeoRiemannianParams,
    pub voice_leading: VoiceLeadingParams,
    pub melody: MelodyParams,
    pub classic_groove: ClassicGrooveParams,
    pub perfect_balance: PerfectBalanceParams,
    pub arrangement: ArrangementParams,
    pub emotional_quadrant: EmotionalQuadrantParams,
    /// Paramètres de planification et de ligne de violon du reel (D4, D6, constitution V).
    #[serde(default)]
    pub reel_fiddle: ReelFiddleParams,
    /// Banque de cellules mélodiques du reel traditionnel québécois (D5, constitution V).
    #[serde(default)]
    pub reel_cells: CellBank,
    /// Harmonisation et accompagnement du reel (D7, D8).
    #[serde(default)]
    pub trad_harmony: TradHarmonyParams,
    /// Table de podorythmie du reel traditionnel québécois (D9, constitution V).
    #[serde(default)]
    pub reel_feet: FeetTable,
}

impl Default for TuningParams {
    fn default() -> Self {
        Self {
            harmony_driver: HarmonyDriverParams::default(),
            grammar: GrammarParams::default(),
            neo_riemannian: NeoRiemannianParams::default(),
            voice_leading: VoiceLeadingParams::default(),
            melody: MelodyParams::default(),
            classic_groove: ClassicGrooveParams::default(),
            perfect_balance: PerfectBalanceParams::default(),
            arrangement: ArrangementParams::default(),
            emotional_quadrant: EmotionalQuadrantParams::default(),
            reel_fiddle: ReelFiddleParams::default(),
            reel_cells: CellBank::default(),
            trad_harmony: TradHarmonyParams::default(),
            reel_feet: FeetTable::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// A single validation failure.
#[derive(Clone, Debug)]
pub struct ValidationError {
    pub field: &'static str,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl TuningParams {
    /// Validate all fields are within documented ranges and invariants hold.
    /// Returns `Ok(())` when valid, or a list of every violation found.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // --- helpers ---
        let check_01 = |errors: &mut Vec<ValidationError>, field: &'static str, v: f32| {
            if v.is_nan() || v < 0.0 || v > 1.0 {
                errors.push(ValidationError {
                    field,
                    message: format!("expected 0.0..=1.0, got {v}"),
                });
            }
        };
        let check_range =
            |errors: &mut Vec<ValidationError>, field: &'static str, v: f32, lo: f32, hi: f32| {
                if v.is_nan() || v < lo || v > hi {
                    errors.push(ValidationError {
                        field,
                        message: format!("expected {lo}..={hi}, got {v}"),
                    });
                }
            };
        let check_monotonic =
            |errors: &mut Vec<ValidationError>, field: &'static str, arr: &[f32]| {
                for w in arr.windows(2) {
                    if w[0] > w[1] {
                        errors.push(ValidationError {
                            field,
                            message: format!("not monotonically increasing: {} > {}", w[0], w[1]),
                        });
                        break;
                    }
                }
            };

        // --- HarmonyDriverParams ---
        let hd = &self.harmony_driver;
        check_01(&mut errors, "harmony_driver.steedman_lower", hd.steedman_lower);
        check_01(&mut errors, "harmony_driver.steedman_upper", hd.steedman_upper);
        check_01(&mut errors, "harmony_driver.neo_lower", hd.neo_lower);
        check_01(&mut errors, "harmony_driver.neo_upper", hd.neo_upper);
        check_range(&mut errors, "harmony_driver.hysteresis_boost", hd.hysteresis_boost, 0.0, 0.5);
        check_01(
            &mut errors,
            "harmony_driver.dramatic_tension_drop_upper",
            hd.dramatic_tension_drop_upper,
        );
        check_01(
            &mut errors,
            "harmony_driver.dramatic_tension_drop_lower",
            hd.dramatic_tension_drop_lower,
        );
        check_01(
            &mut errors,
            "harmony_driver.cadential_resolution_probability",
            hd.cadential_resolution_probability,
        );
        if hd.steedman_lower >= hd.steedman_upper {
            errors.push(ValidationError {
                field: "harmony_driver.steedman_lower/upper",
                message: format!(
                    "steedman_lower ({}) must be < steedman_upper ({})",
                    hd.steedman_lower, hd.steedman_upper
                ),
            });
        }
        if hd.steedman_upper > hd.neo_lower {
            errors.push(ValidationError {
                field: "harmony_driver.steedman_upper/neo_lower",
                message: format!(
                    "steedman_upper ({}) must be <= neo_lower ({})",
                    hd.steedman_upper, hd.neo_lower
                ),
            });
        }
        if hd.neo_lower >= hd.neo_upper {
            errors.push(ValidationError {
                field: "harmony_driver.neo_lower/upper",
                message: format!(
                    "neo_lower ({}) must be < neo_upper ({})",
                    hd.neo_lower, hd.neo_upper
                ),
            });
        }

        // --- GrammarParams ---
        let gp = &self.grammar;
        if gp.max_recursion_depth > 5 {
            errors.push(ValidationError {
                field: "grammar.max_recursion_depth",
                message: format!("expected 0..=5, got {}", gp.max_recursion_depth),
            });
        }
        for (name, v) in [
            ("grammar.preparation_weight", gp.preparation_weight),
            ("grammar.backcycle_weight", gp.backcycle_weight),
            ("grammar.tritone_sub_weight", gp.tritone_sub_weight),
            ("grammar.cadential_weight", gp.cadential_weight),
            ("grammar.deceptive_weight", gp.deceptive_weight),
            ("grammar.modal_interchange_weight", gp.modal_interchange_weight),
        ] {
            check_range(&mut errors, name, v, 0.0, 3.0);
        }
        check_01(
            &mut errors,
            "grammar.chord_quality_valence_threshold",
            gp.chord_quality_valence_threshold,
        );

        // --- NeoRiemannianParams ---
        let nr = &self.neo_riemannian;
        check_01(
            &mut errors,
            "neo_riemannian.positive_valence_threshold",
            nr.positive_valence_threshold,
        );
        check_range(
            &mut errors,
            "neo_riemannian.negative_valence_threshold",
            nr.negative_valence_threshold,
            -1.0,
            0.0,
        );
        check_01(&mut errors, "neo_riemannian.positive_r_prob", nr.positive_r_prob);
        check_01(&mut errors, "neo_riemannian.positive_p_cumulative", nr.positive_p_cumulative);
        if nr.positive_r_prob > nr.positive_p_cumulative {
            errors.push(ValidationError {
                field: "neo_riemannian.positive_r/p",
                message: format!(
                    "positive_r_prob ({}) must be <= positive_p_cumulative ({})",
                    nr.positive_r_prob, nr.positive_p_cumulative
                ),
            });
        }
        check_01(&mut errors, "neo_riemannian.negative_l_prob", nr.negative_l_prob);
        check_01(&mut errors, "neo_riemannian.negative_p_cumulative", nr.negative_p_cumulative);
        if nr.negative_l_prob > nr.negative_p_cumulative {
            errors.push(ValidationError {
                field: "neo_riemannian.negative_l/p",
                message: format!(
                    "negative_l_prob ({}) must be <= negative_p_cumulative ({})",
                    nr.negative_l_prob, nr.negative_p_cumulative
                ),
            });
        }
        check_01(&mut errors, "neo_riemannian.neutral_p_prob", nr.neutral_p_prob);
        check_01(&mut errors, "neo_riemannian.neutral_l_cumulative", nr.neutral_l_cumulative);
        if nr.neutral_p_prob > nr.neutral_l_cumulative {
            errors.push(ValidationError {
                field: "neo_riemannian.neutral_p/l",
                message: format!(
                    "neutral_p_prob ({}) must be <= neutral_l_cumulative ({})",
                    nr.neutral_p_prob, nr.neutral_l_cumulative
                ),
            });
        }
        check_01(
            &mut errors,
            "neo_riemannian.composite_tension_threshold",
            nr.composite_tension_threshold,
        );
        check_01(&mut errors, "neo_riemannian.composite_probability", nr.composite_probability);

        // --- VoiceLeadingParams ---
        let vl = &self.voice_leading;
        if vl.max_semitone_movement == 0 || vl.max_semitone_movement > 4 {
            errors.push(ValidationError {
                field: "voice_leading.max_semitone_movement",
                message: format!("expected 1..=4, got {}", vl.max_semitone_movement),
            });
        }
        check_01(&mut errors, "voice_leading.trq_threshold", vl.trq_threshold);
        check_01(&mut errors, "voice_leading.high_tension_threshold", vl.high_tension_threshold);
        check_01(&mut errors, "voice_leading.low_tension_threshold", vl.low_tension_threshold);
        if vl.low_tension_threshold >= vl.high_tension_threshold {
            errors.push(ValidationError {
                field: "voice_leading.low/high_tension_threshold",
                message: format!(
                    "low ({}) must be < high ({})",
                    vl.low_tension_threshold, vl.high_tension_threshold
                ),
            });
        }

        // --- MelodyParams ---
        let ml = &self.melody;
        if ml.pink_noise_depth == 0 || ml.pink_noise_depth > 10 {
            errors.push(ValidationError {
                field: "melody.pink_noise_depth",
                message: format!("expected 1..=10, got {}", ml.pink_noise_depth),
            });
        }
        check_01(&mut errors, "melody.default_hurst_factor", ml.default_hurst_factor);
        if ml.gap_fill_threshold < 1 || ml.gap_fill_threshold > 12 {
            errors.push(ValidationError {
                field: "melody.gap_fill_threshold",
                message: format!("expected 1..=12, got {}", ml.gap_fill_threshold),
            });
        }
        check_01(&mut errors, "melody.motif_new_material_bias", ml.motif_new_material_bias);
        check_range(&mut errors, "melody.fractal_boost", ml.fractal_boost, 0.5, 5.0);
        check_range(&mut errors, "melody.fractal_range", ml.fractal_range, 5.0, 40.0);
        if ml.consecutive_direction_limit < 2 || ml.consecutive_direction_limit > 12 {
            errors.push(ValidationError {
                field: "melody.consecutive_direction_limit",
                message: format!("expected 2..=12, got {}", ml.consecutive_direction_limit),
            });
        }
        if ml.leading_tone_resolution_weight == 0 || ml.leading_tone_resolution_weight > 100 {
            errors.push(ValidationError {
                field: "melody.leading_tone_resolution_weight",
                message: format!("expected 1..=100, got {}", ml.leading_tone_resolution_weight),
            });
        }

        // --- ClassicGrooveParams ---
        let cg = &self.classic_groove;
        for v in &cg.kick_density_thresholds {
            check_01(&mut errors, "classic_groove.kick_density_thresholds[]", *v);
        }
        check_monotonic(
            &mut errors,
            "classic_groove.kick_density_thresholds",
            &cg.kick_density_thresholds,
        );
        check_01(&mut errors, "classic_groove.kick_downbeat_velocity", cg.kick_downbeat_velocity);
        check_01(&mut errors, "classic_groove.kick_secondary_velocity", cg.kick_secondary_velocity);
        check_01(
            &mut errors,
            "classic_groove.kick_anticipation_velocity",
            cg.kick_anticipation_velocity,
        );
        check_01(
            &mut errors,
            "classic_groove.ghost_note_tension_threshold",
            cg.ghost_note_tension_threshold,
        );
        check_01(&mut errors, "classic_groove.ghost_note_velocity", cg.ghost_note_velocity);
        for v in &cg.hat_density_thresholds {
            check_01(&mut errors, "classic_groove.hat_density_thresholds[]", *v);
        }
        check_monotonic(
            &mut errors,
            "classic_groove.hat_density_thresholds",
            &cg.hat_density_thresholds,
        );
        check_01(&mut errors, "classic_groove.hat_on_beat_velocity", cg.hat_on_beat_velocity);
        check_01(&mut errors, "classic_groove.hat_off_beat_velocity", cg.hat_off_beat_velocity);
        check_01(&mut errors, "classic_groove.hat_dense_on_velocity", cg.hat_dense_on_velocity);
        check_01(&mut errors, "classic_groove.hat_dense_off_velocity", cg.hat_dense_off_velocity);
        check_01(
            &mut errors,
            "classic_groove.hat_dense_ghost_velocity",
            cg.hat_dense_ghost_velocity,
        );
        check_01(&mut errors, "classic_groove.hat_sparse_velocity", cg.hat_sparse_velocity);
        check_01(
            &mut errors,
            "classic_groove.hat_masking_density_threshold",
            cg.hat_masking_density_threshold,
        );
        check_01(
            &mut errors,
            "classic_groove.bass_split_tension_threshold",
            cg.bass_split_tension_threshold,
        );
        check_01(&mut errors, "classic_groove.snare_backbeat_velocity", cg.snare_backbeat_velocity);

        // --- PerfectBalanceParams ---
        let pb = &self.perfect_balance;
        check_01(
            &mut errors,
            "perfect_balance.kick_polygon_low_threshold",
            pb.kick_polygon_low_threshold,
        );
        check_01(&mut errors, "perfect_balance.kick_low_velocity", pb.kick_low_velocity);
        check_01(&mut errors, "perfect_balance.kick_normal_velocity", pb.kick_normal_velocity);
        check_01(&mut errors, "perfect_balance.snare_velocity", pb.snare_velocity);
        for &v in &pb.hat_vertex_counts {
            if v == 0 || v > 32 {
                errors.push(ValidationError {
                    field: "perfect_balance.hat_vertex_counts[]",
                    message: format!("expected 1..=32, got {v}"),
                });
            }
        }
        for v in &pb.hat_density_thresholds {
            check_01(&mut errors, "perfect_balance.hat_density_thresholds[]", *v);
        }
        check_monotonic(
            &mut errors,
            "perfect_balance.hat_density_thresholds",
            &pb.hat_density_thresholds,
        );
        check_01(
            &mut errors,
            "perfect_balance.hat_velocity_coefficient",
            pb.hat_velocity_coefficient,
        );
        check_01(
            &mut errors,
            "perfect_balance.swing_tension_threshold",
            pb.swing_tension_threshold,
        );
        check_range(
            &mut errors,
            "perfect_balance.swing_scaling_factor",
            pb.swing_scaling_factor,
            0.0,
            2.0,
        );
        check_01(&mut errors, "perfect_balance.bass_polygon_velocity", pb.bass_polygon_velocity);
        check_01(
            &mut errors,
            "perfect_balance.bass_low_density_threshold",
            pb.bass_low_density_threshold,
        );
        check_01(&mut errors, "perfect_balance.lead_polygon_velocity", pb.lead_polygon_velocity);
        check_01(
            &mut errors,
            "perfect_balance.hat_masking_density_threshold",
            pb.hat_masking_density_threshold,
        );

        // --- ArrangementParams ---
        let ar = &self.arrangement;
        if ar.fill_zone_size == 0 || ar.fill_zone_size > 8 {
            errors.push(ValidationError {
                field: "arrangement.fill_zone_size",
                message: format!("expected 1..=8, got {}", ar.fill_zone_size),
            });
        }
        // crash_velocity: u8 is inherently 0..=255; MIDI caps at 127
        if ar.crash_velocity > 127 {
            errors.push(ValidationError {
                field: "arrangement.crash_velocity",
                message: format!("expected 0..=127, got {}", ar.crash_velocity),
            });
        }
        check_01(&mut errors, "arrangement.ghost_velocity_factor", ar.ghost_velocity_factor);
        check_range(&mut errors, "arrangement.tom_velocity_boost", ar.tom_velocity_boost, 0.5, 2.0);
        for interval in [
            ("arrangement.progression_switch_interval_slow", ar.progression_switch_interval_slow),
            (
                "arrangement.progression_switch_interval_normal",
                ar.progression_switch_interval_normal,
            ),
            ("arrangement.progression_switch_interval_fast", ar.progression_switch_interval_fast),
        ] {
            if interval.1 == 0 || interval.1 > 8 {
                errors.push(ValidationError {
                    field: interval.0,
                    message: format!("expected 1..=8, got {}", interval.1),
                });
            }
        }
        for v in &ar.progression_tension_thresholds {
            check_01(&mut errors, "arrangement.progression_tension_thresholds[]", *v);
        }
        check_monotonic(
            &mut errors,
            "arrangement.progression_tension_thresholds",
            &ar.progression_tension_thresholds,
        );
        check_01(
            &mut errors,
            "arrangement.measures_per_chord_hysteresis",
            ar.measures_per_chord_hysteresis,
        );
        check_01(&mut errors, "arrangement.energy_high_tension", ar.energy_high_tension);
        check_01(&mut errors, "arrangement.energy_high_density", ar.energy_high_density);
        check_01(&mut errors, "arrangement.energy_high_arousal", ar.energy_high_arousal);

        // --- EmotionalQuadrantParams ---
        let eq = &self.emotional_quadrant;
        check_01(
            &mut errors,
            "emotional_quadrant.happy_valence_threshold",
            eq.happy_valence_threshold,
        );
        check_range(
            &mut errors,
            "emotional_quadrant.sad_valence_threshold",
            eq.sad_valence_threshold,
            -1.0,
            0.0,
        );
        check_01(
            &mut errors,
            "emotional_quadrant.energetic_tension_threshold",
            eq.energetic_tension_threshold,
        );

        // --- ReelFiddleParams (D4, D6) ---
        let rf = &self.reel_fiddle;
        check_range(&mut errors, "reel_fiddle.default_bpm", rf.default_bpm, 40.0, 300.0);
        check_range(
            &mut errors,
            "reel_fiddle.default_swing_ratio",
            rf.default_swing_ratio,
            0.5,
            0.75,
        );
        if rf.default_time_signature.numerator == 0 || rf.default_time_signature.denominator == 0 {
            errors.push(ValidationError {
                field: "reel_fiddle.default_time_signature",
                message: format!(
                    "numerator ({}) and denominator ({}) must be > 0",
                    rf.default_time_signature.numerator, rf.default_time_signature.denominator
                ),
            });
        }
        let key_weight_sum = rf
            .weight_d_ionian
            .saturating_add(rf.weight_g_ionian)
            .saturating_add(rf.weight_a_ionian)
            .saturating_add(rf.weight_a_mixolydian)
            .saturating_add(rf.weight_d_mixolydian);
        if key_weight_sum == 0 {
            errors.push(ValidationError {
                field: "reel_fiddle.weight_*_ionian/mixolydian",
                message: "at least one key/mode weight must be > 0".to_string(),
            });
        }
        let role_weight_sum = rf
            .weight_role_run
            .saturating_add(rf.weight_role_arp)
            .saturating_add(rf.weight_role_turn);
        if role_weight_sum == 0 {
            errors.push(ValidationError {
                field: "reel_fiddle.weight_role_*",
                message: "at least one free role weight must be > 0".to_string(),
            });
        }
        if rf.continuity_max_retries == 0 {
            errors.push(ValidationError {
                field: "reel_fiddle.continuity_max_retries",
                message: "expected >= 1, got 0".to_string(),
            });
        }
        if rf.max_continuity_step_distance < 0 {
            errors.push(ValidationError {
                field: "reel_fiddle.max_continuity_step_distance",
                message: format!("expected >= 0, got {}", rf.max_continuity_step_distance),
            });
        }
        if rf.melody_center_midi < rf.tessitura_min || rf.melody_center_midi > rf.tessitura_max {
            errors.push(ValidationError {
                field: "reel_fiddle.melody_center_midi",
                message: format!(
                    "expected within tessitura {}..={}, got {}",
                    rf.tessitura_min, rf.tessitura_max, rf.melody_center_midi
                ),
            });
        }
        if !(-12..=12).contains(&rf.strain_b_lift_semitones) {
            errors.push(ValidationError {
                field: "reel_fiddle.strain_b_lift_semitones",
                message: format!("expected -12..=12, got {}", rf.strain_b_lift_semitones),
            });
        }
        if rf.tonic_midi_min > rf.tonic_midi_max || rf.tonic_midi_max > 127 {
            errors.push(ValidationError {
                field: "reel_fiddle.tonic_midi_min/max",
                message: format!(
                    "invalid tonic MIDI range {}..={}",
                    rf.tonic_midi_min, rf.tonic_midi_max
                ),
            });
        }
        if rf.tessitura_min > rf.tessitura_max
            || rf.tessitura_max.saturating_sub(rf.tessitura_min) < 12
            || rf.tessitura_max > 127
        {
            errors.push(ValidationError {
                field: "reel_fiddle.tessitura_min/max",
                message: format!(
                    "invalid tessitura range {}..={} (must span at least 12 semitones)",
                    rf.tessitura_min, rf.tessitura_max
                ),
            });
        }
        if rf.base_velocity == 0 || rf.base_velocity > 127 {
            errors.push(ValidationError {
                field: "reel_fiddle.base_velocity",
                message: format!("expected 1..=127, got {}", rf.base_velocity),
            });
        }

        // --- Reel CellBank (D5) ---
        if let Err(err) = self.reel_cells.validate() {
            errors.push(ValidationError { field: "reel_cells", message: err.to_string() });
        }
        // --- Reel FeetTable (D9) ---
        if let Err(err) = self.reel_feet.validate() {
            errors.push(ValidationError { field: "reel_feet", message: err.to_string() });
        }

        // --- TradHarmonyParams (D7, D8) ---
        let th = &self.trad_harmony;
        let mut check_th = |ok: bool, message: &str| {
            if !ok {
                errors.push(ValidationError { field: "trad_harmony", message: message.to_owned() });
            }
        };
        check_th(th.strong_weight > 0 && th.weak_weight > 0, "les poids de vote doivent être > 0");
        check_th(!th.tie_priority.is_empty(), "tie_priority ne doit pas être vide");
        check_th(
            th.bass_low <= th.bass_high && th.bass_high <= 127,
            "bass_low <= bass_high <= 127 requis",
        );
        check_th(th.bass_duration_steps > 0 && th.chop_duration_steps > 0, "durées en pas > 0");
        check_th(
            th.bass_steps.iter().chain(&th.chop_steps).all(|&s| s < 16),
            "pas d'accompagnement hors mesure (< 16)",
        );
        check_th(th.guitar_center <= 127, "guitar_center <= 127 requis");

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ---------------------------------------------------------------------------
// Overlay sub-structs (for partial profile merging)
// ---------------------------------------------------------------------------

/// Partial overlay for `HarmonyDriverParams`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HarmonyDriverOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steedman_lower: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steedman_upper: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neo_lower: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neo_upper: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hysteresis_boost: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dramatic_tension_drop_upper: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dramatic_tension_drop_lower: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadential_resolution_probability: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<usize>,
}

/// Partial overlay for `GrammarParams`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GrammarOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grammar_style: Option<GrammarStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_recursion_depth: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preparation_weight: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backcycle_weight: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tritone_sub_weight: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadential_weight: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deceptive_weight: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modal_interchange_weight: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chord_quality_valence_threshold: Option<f32>,
}

/// Partial overlay for `NeoRiemannianParams`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NeoRiemannianOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positive_valence_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_valence_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positive_r_prob: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positive_p_cumulative: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_l_prob: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_p_cumulative: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neutral_p_prob: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neutral_l_cumulative: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composite_tension_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composite_probability: Option<f32>,
}

/// Partial overlay for `VoiceLeadingParams`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VoiceLeadingOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_semitone_movement: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_cardinality_morph: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trq_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high_tension_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low_tension_threshold: Option<f32>,
}

/// Partial overlay for `MelodyParams`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MelodyOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pink_noise_depth: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_hurst_factor: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap_fill_threshold: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motif_new_material_bias: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fractal_boost: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fractal_range: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consecutive_direction_limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leading_tone_resolution_weight: Option<u32>,
}

/// Partial overlay for `ClassicGrooveParams`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ClassicGrooveOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kick_density_thresholds: Option<[f32; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kick_downbeat_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kick_secondary_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kick_anticipation_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghost_note_tension_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghost_note_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_density_thresholds: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_on_beat_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_off_beat_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_dense_on_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_dense_off_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_dense_ghost_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_sparse_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_masking_density_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bass_split_tension_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snare_backbeat_velocity: Option<f32>,
}

/// Partial overlay for `PerfectBalanceParams`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PerfectBalanceOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kick_polygon_low_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kick_low_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kick_normal_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snare_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_vertex_counts: Option<[usize; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_density_thresholds: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_velocity_coefficient: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swing_tension_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swing_scaling_factor: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bass_polygon_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bass_low_density_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_polygon_velocity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hat_masking_density_threshold: Option<f32>,
}

/// Partial overlay for `ArrangementParams`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ArrangementOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_zone_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crash_velocity: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghost_velocity_factor: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tom_velocity_boost: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progression_switch_interval_slow: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progression_switch_interval_normal: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progression_switch_interval_fast: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progression_tension_thresholds: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measures_per_chord_hysteresis: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy_high_tension: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy_high_density: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy_high_arousal: Option<f32>,
}

/// Partial overlay for `EmotionalQuadrantParams`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EmotionalQuadrantOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub happy_valence_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sad_valence_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energetic_tension_threshold: Option<f32>,
}

/// Partial overlay for `ReelFiddleParams`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReelFiddleOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_bpm: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_swing_ratio: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_time_signature: Option<TimeSignature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_d_ionian: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_g_ionian: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_a_ionian: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_a_mixolydian: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_d_mixolydian: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_role_run: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_role_arp: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_role_turn: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_continuity_step_distance: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuity_max_retries: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub melody_center_midi: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strain_b_lift_semitones: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tonic_midi_min: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tonic_midi_max: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tessitura_min: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tessitura_max: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_velocity: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strong_eighth_velocity_offset: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub even_eighth_velocity_offset: Option<i8>,
}

// ---------------------------------------------------------------------------
// Root TuningOverlay
// ---------------------------------------------------------------------------

/// Partial tuning overlay — `None` sub-structs mean "keep defaults for this group".
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TuningOverlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harmony_driver: Option<HarmonyDriverOverlay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grammar: Option<GrammarOverlay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neo_riemannian: Option<NeoRiemannianOverlay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_leading: Option<VoiceLeadingOverlay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub melody: Option<MelodyOverlay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classic_groove: Option<ClassicGrooveOverlay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perfect_balance: Option<PerfectBalanceOverlay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrangement: Option<ArrangementOverlay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emotional_quadrant: Option<EmotionalQuadrantOverlay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reel_fiddle: Option<ReelFiddleOverlay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reel_cells: Option<CellBank>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trad_harmony: Option<TradHarmonyParams>,
    pub reel_feet: Option<FeetTable>,
}

// ---------------------------------------------------------------------------
// Overlay merge
// ---------------------------------------------------------------------------

/// Helper: apply `Option<T>` overlay onto a base value.
macro_rules! apply {
    ($base:expr, $overlay:expr) => {
        if let Some(v) = $overlay {
            $base = v;
        }
    };
}

impl TuningParams {
    /// Return a copy with the overlay applied. `None` fields keep their base value.
    pub fn with_overlay(&self, overlay: &TuningOverlay) -> Self {
        let mut result = self.clone();

        if let Some(ref o) = overlay.harmony_driver {
            let b = &mut result.harmony_driver;
            apply!(b.steedman_lower, o.steedman_lower);
            apply!(b.steedman_upper, o.steedman_upper);
            apply!(b.neo_lower, o.neo_lower);
            apply!(b.neo_upper, o.neo_upper);
            apply!(b.hysteresis_boost, o.hysteresis_boost);
            apply!(b.dramatic_tension_drop_upper, o.dramatic_tension_drop_upper);
            apply!(b.dramatic_tension_drop_lower, o.dramatic_tension_drop_lower);
            apply!(b.cadential_resolution_probability, o.cadential_resolution_probability);
            apply!(b.max_retries, o.max_retries);
        }

        if let Some(ref o) = overlay.grammar {
            let b = &mut result.grammar;
            apply!(b.grammar_style, o.grammar_style);
            apply!(b.max_recursion_depth, o.max_recursion_depth);
            apply!(b.preparation_weight, o.preparation_weight);
            apply!(b.backcycle_weight, o.backcycle_weight);
            apply!(b.tritone_sub_weight, o.tritone_sub_weight);
            apply!(b.cadential_weight, o.cadential_weight);
            apply!(b.deceptive_weight, o.deceptive_weight);
            apply!(b.modal_interchange_weight, o.modal_interchange_weight);
            apply!(b.chord_quality_valence_threshold, o.chord_quality_valence_threshold);
        }

        if let Some(ref o) = overlay.neo_riemannian {
            let b = &mut result.neo_riemannian;
            apply!(b.positive_valence_threshold, o.positive_valence_threshold);
            apply!(b.negative_valence_threshold, o.negative_valence_threshold);
            apply!(b.positive_r_prob, o.positive_r_prob);
            apply!(b.positive_p_cumulative, o.positive_p_cumulative);
            apply!(b.negative_l_prob, o.negative_l_prob);
            apply!(b.negative_p_cumulative, o.negative_p_cumulative);
            apply!(b.neutral_p_prob, o.neutral_p_prob);
            apply!(b.neutral_l_cumulative, o.neutral_l_cumulative);
            apply!(b.composite_tension_threshold, o.composite_tension_threshold);
            apply!(b.composite_probability, o.composite_probability);
        }

        if let Some(ref o) = overlay.voice_leading {
            let b = &mut result.voice_leading;
            apply!(b.max_semitone_movement, o.max_semitone_movement);
            apply!(b.allow_cardinality_morph, o.allow_cardinality_morph);
            apply!(b.trq_threshold, o.trq_threshold);
            apply!(b.high_tension_threshold, o.high_tension_threshold);
            apply!(b.low_tension_threshold, o.low_tension_threshold);
        }

        if let Some(ref o) = overlay.melody {
            let b = &mut result.melody;
            apply!(b.pink_noise_depth, o.pink_noise_depth);
            apply!(b.default_hurst_factor, o.default_hurst_factor);
            apply!(b.gap_fill_threshold, o.gap_fill_threshold);
            apply!(b.motif_new_material_bias, o.motif_new_material_bias);
            apply!(b.fractal_boost, o.fractal_boost);
            apply!(b.fractal_range, o.fractal_range);
            apply!(b.consecutive_direction_limit, o.consecutive_direction_limit);
            apply!(b.leading_tone_resolution_weight, o.leading_tone_resolution_weight);
        }

        if let Some(ref o) = overlay.classic_groove {
            let b = &mut result.classic_groove;
            apply!(b.kick_density_thresholds, o.kick_density_thresholds);
            apply!(b.kick_downbeat_velocity, o.kick_downbeat_velocity);
            apply!(b.kick_secondary_velocity, o.kick_secondary_velocity);
            apply!(b.kick_anticipation_velocity, o.kick_anticipation_velocity);
            apply!(b.ghost_note_tension_threshold, o.ghost_note_tension_threshold);
            apply!(b.ghost_note_velocity, o.ghost_note_velocity);
            apply!(b.hat_density_thresholds, o.hat_density_thresholds);
            apply!(b.hat_on_beat_velocity, o.hat_on_beat_velocity);
            apply!(b.hat_off_beat_velocity, o.hat_off_beat_velocity);
            apply!(b.hat_dense_on_velocity, o.hat_dense_on_velocity);
            apply!(b.hat_dense_off_velocity, o.hat_dense_off_velocity);
            apply!(b.hat_dense_ghost_velocity, o.hat_dense_ghost_velocity);
            apply!(b.hat_sparse_velocity, o.hat_sparse_velocity);
            apply!(b.hat_masking_density_threshold, o.hat_masking_density_threshold);
            apply!(b.bass_split_tension_threshold, o.bass_split_tension_threshold);
            apply!(b.snare_backbeat_velocity, o.snare_backbeat_velocity);
        }

        if let Some(ref o) = overlay.perfect_balance {
            let b = &mut result.perfect_balance;
            apply!(b.kick_polygon_low_threshold, o.kick_polygon_low_threshold);
            apply!(b.kick_low_velocity, o.kick_low_velocity);
            apply!(b.kick_normal_velocity, o.kick_normal_velocity);
            apply!(b.snare_velocity, o.snare_velocity);
            apply!(b.hat_vertex_counts, o.hat_vertex_counts);
            apply!(b.hat_density_thresholds, o.hat_density_thresholds);
            apply!(b.hat_velocity_coefficient, o.hat_velocity_coefficient);
            apply!(b.swing_tension_threshold, o.swing_tension_threshold);
            apply!(b.swing_scaling_factor, o.swing_scaling_factor);
            apply!(b.bass_polygon_velocity, o.bass_polygon_velocity);
            apply!(b.bass_low_density_threshold, o.bass_low_density_threshold);
            apply!(b.lead_polygon_velocity, o.lead_polygon_velocity);
            apply!(b.hat_masking_density_threshold, o.hat_masking_density_threshold);
        }

        if let Some(ref o) = overlay.arrangement {
            let b = &mut result.arrangement;
            apply!(b.fill_zone_size, o.fill_zone_size);
            apply!(b.crash_velocity, o.crash_velocity);
            apply!(b.ghost_velocity_factor, o.ghost_velocity_factor);
            apply!(b.tom_velocity_boost, o.tom_velocity_boost);
            apply!(b.progression_switch_interval_slow, o.progression_switch_interval_slow);
            apply!(b.progression_switch_interval_normal, o.progression_switch_interval_normal);
            apply!(b.progression_switch_interval_fast, o.progression_switch_interval_fast);
            apply!(b.progression_tension_thresholds, o.progression_tension_thresholds);
            apply!(b.measures_per_chord_hysteresis, o.measures_per_chord_hysteresis);
            apply!(b.energy_high_tension, o.energy_high_tension);
            apply!(b.energy_high_density, o.energy_high_density);
            apply!(b.energy_high_arousal, o.energy_high_arousal);
        }

        if let Some(ref o) = overlay.emotional_quadrant {
            let b = &mut result.emotional_quadrant;
            apply!(b.happy_valence_threshold, o.happy_valence_threshold);
            apply!(b.sad_valence_threshold, o.sad_valence_threshold);
            apply!(b.energetic_tension_threshold, o.energetic_tension_threshold);
        }

        if let Some(o) = &overlay.reel_fiddle {
            let b = &mut result.reel_fiddle;
            apply!(b.default_bpm, o.default_bpm);
            apply!(b.default_swing_ratio, o.default_swing_ratio);
            apply!(b.default_time_signature, o.default_time_signature);
            apply!(b.weight_d_ionian, o.weight_d_ionian);
            apply!(b.weight_g_ionian, o.weight_g_ionian);
            apply!(b.weight_a_ionian, o.weight_a_ionian);
            apply!(b.weight_a_mixolydian, o.weight_a_mixolydian);
            apply!(b.weight_d_mixolydian, o.weight_d_mixolydian);
            apply!(b.weight_role_run, o.weight_role_run);
            apply!(b.weight_role_arp, o.weight_role_arp);
            apply!(b.weight_role_turn, o.weight_role_turn);
            apply!(b.max_continuity_step_distance, o.max_continuity_step_distance);
            apply!(b.continuity_max_retries, o.continuity_max_retries);
            apply!(b.melody_center_midi, o.melody_center_midi);
            apply!(b.strain_b_lift_semitones, o.strain_b_lift_semitones);
            apply!(b.tonic_midi_min, o.tonic_midi_min);
            apply!(b.tonic_midi_max, o.tonic_midi_max);
            apply!(b.tessitura_min, o.tessitura_min);
            apply!(b.tessitura_max, o.tessitura_max);
            apply!(b.base_velocity, o.base_velocity);
            apply!(b.strong_eighth_velocity_offset, o.strong_eighth_velocity_offset);
            apply!(b.even_eighth_velocity_offset, o.even_eighth_velocity_offset);
        }

        if let Some(reel_cells) = &overlay.reel_cells {
            result.reel_cells = reel_cells.clone();
        }
        if let Some(reel_feet) = &overlay.reel_feet {
            result.reel_feet = reel_feet.clone();
        }

        if let Some(trad_harmony) = &overlay.trad_harmony {
            result.trad_harmony = trad_harmony.clone();
        }

        result
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_validates() {
        let tp = TuningParams::default();
        tp.validate().expect("default TuningParams should be valid");
    }

    #[test]
    fn json_roundtrip() {
        let tp = TuningParams::default();
        let json = serde_json::to_string_pretty(&tp).expect("serialize");
        let tp2: TuningParams = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(tp, tp2);
    }

    #[test]
    fn toml_roundtrip() {
        let tp = TuningParams::default();
        let toml_str = toml::to_string_pretty(&tp).expect("serialize");
        let tp2: TuningParams = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(tp, tp2);
    }

    #[test]
    fn overlay_empty_is_noop() {
        let base = TuningParams::default();
        let overlay = TuningOverlay::default();
        let result = base.with_overlay(&overlay);
        assert_eq!(base, result);
    }

    #[test]
    fn overlay_applies_partial_changes() {
        let base = TuningParams::default();
        let overlay = TuningOverlay {
            harmony_driver: Some(HarmonyDriverOverlay {
                steedman_lower: Some(0.3),
                ..Default::default()
            }),
            melody: Some(MelodyOverlay {
                default_hurst_factor: Some(0.5),
                fractal_range: Some(30.0),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = base.with_overlay(&overlay);

        // Changed fields
        assert_eq!(result.harmony_driver.steedman_lower, 0.3);
        assert_eq!(result.melody.default_hurst_factor, 0.5);
        assert_eq!(result.melody.fractal_range, 30.0);

        // Unchanged fields within modified sub-structs
        assert_eq!(result.harmony_driver.steedman_upper, 0.55);
        assert_eq!(result.melody.pink_noise_depth, 5);

        // Untouched sub-structs
        assert_eq!(result.grammar, GrammarParams::default());
        assert_eq!(result.classic_groove, ClassicGrooveParams::default());
    }

    #[test]
    fn overlay_json_sparse_roundtrip() {
        let overlay = TuningOverlay {
            classic_groove: Some(ClassicGrooveOverlay {
                ghost_note_velocity: Some(0.15),
                ..Default::default()
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&overlay).expect("serialize");
        // None fields should be absent from JSON
        assert!(!json.contains("melody"));
        assert!(json.contains("ghost_note_velocity"));

        let overlay2: TuningOverlay = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(overlay, overlay2);
    }

    #[test]
    fn validation_catches_inverted_thresholds() {
        let mut tp = TuningParams::default();
        tp.harmony_driver.steedman_lower = 0.8;
        tp.harmony_driver.steedman_upper = 0.3;
        let errs = tp.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.field.contains("steedman")));
    }

    #[test]
    fn validation_catches_out_of_range() {
        let mut tp = TuningParams::default();
        tp.melody.fractal_boost = 100.0;
        tp.voice_leading.max_semitone_movement = 0;
        let errs = tp.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.field.contains("fractal_boost")));
        assert!(errs.iter().any(|e| e.field.contains("max_semitone_movement")));
    }

    #[test]
    fn validation_catches_non_monotonic_thresholds() {
        let mut tp = TuningParams::default();
        tp.classic_groove.kick_density_thresholds = [0.8, 0.4, 0.6, 0.2];
        let errs = tp.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.field.contains("kick_density")));
    }

    #[test]
    fn validation_catches_neo_riemannian_cumulative_inversion() {
        let mut tp = TuningParams::default();
        tp.neo_riemannian.positive_r_prob = 0.9;
        tp.neo_riemannian.positive_p_cumulative = 0.5;
        let errs = tp.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.field.contains("positive_r/p")));
    }
    #[test]
    fn validation_catches_invalid_reel_feet() {
        let mut tp = TuningParams::default();
        tp.reel_feet.patterns.clear();
        let errs = tp.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.field == "reel_feet"));
    }

    #[test]
    fn overlay_applies_reel_feet() {
        let base = TuningParams::default();
        let mut custom_feet = FeetTable::default();
        custom_feet.heel_velocity = 115;

        let overlay = TuningOverlay { reel_feet: Some(custom_feet.clone()), ..Default::default() };
        let result = base.with_overlay(&overlay);
        assert_eq!(result.reel_feet.heel_velocity, 115);
    }
}

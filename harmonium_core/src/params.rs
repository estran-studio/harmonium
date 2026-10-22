//! Paramètres Musicaux Techniques - Découplés des Émotions
//!
//! Ce module contient les paramètres techniques purs qui pilotent le moteur.
//! Le moteur n'a plus à "deviner" quoi faire - il applique ces paramètres directement.
//!
//! ## Architecture
//! ```text
//! [UI] → EngineParams → EmotionMapper → MusicalParams → moteur
//!                   ou
//! [Debug] → MusicalParams → moteur (bypass émotionnel)
//! ```

use arrayvec::ArrayString;
use rust_music_theory::note::PitchSymbol;
use serde::{Deserialize, Serialize};

use crate::{harmony::HarmonyMode, sequencer::RhythmMode};

/// Controls for musical variety and anti-repetition.
///
/// Every field is tunable — by presets, UI sliders, or a future RL agent (CORELIB-13).
/// Default values produce a balanced mix of structure and surprise.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VarietyParams {
    /// Phrase arc amplitude jitter ±range (0.0 = deterministic, 0.3 = very loose)
    pub phrase_arc_jitter: f32,
    /// Section arc offset jitter ±range (0.0 = fixed AABA, 0.15 = very varied)
    pub section_arc_jitter: f32,
    /// Random velocity jitter ±range in MIDI units (0 = deterministic, 15 = wild)
    pub velocity_jitter: f32,
    /// Bias toward new material vs motif replay (0.0 = all replay, 1.0 = all new)
    pub motif_new_material_bias: f32,
    /// Fractal GPS influence multiplier (1.0 = passive, 3.0 = dominant)
    pub fractal_boost: f32,
    /// Fractal target amplitude in scale degrees (10 = narrow, 30 = very wide)
    pub fractal_range: f32,
    /// Walking bass pitch variety (0.0 = root only, 1.0 = full palette)
    pub walking_bass_variety: f32,
    /// Rhythmic cell variety (0.0 = sustain only, 1.0 = max subdivision)
    pub rhythmic_cell_variety: f32,
    /// Rest probability scalar (0.0 = no rests, 2.0 = very sparse)
    pub rest_probability: f32,
}

impl Default for VarietyParams {
    fn default() -> Self {
        Self {
            phrase_arc_jitter: 0.20,
            section_arc_jitter: 0.08,
            velocity_jitter: 7.0,
            motif_new_material_bias: 0.6,
            fractal_boost: 1.8,
            fractal_range: 22.0,
            walking_bass_variety: 0.7,
            rhythmic_cell_variety: 0.5,
            rest_probability: 1.0,
        }
    }
}

/// Scale type for melody generation (CORELIB-22)
///
/// Controls the pitch-class vocabulary available to the melody generator.
/// Pentatonic (5 notes) is safe/consonant, Diatonic (7 notes) adds variety,
/// HarmonicMinor (7 notes) adds drama, Blues (6 notes) adds color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MelodyScaleType {
    /// 5-note scale — safe, consonant, ambient (PCE max ~2.3)
    Pentatonic,
    /// 7-note major/minor scale — standard for most music (PCE max ~2.8)
    #[default]
    Diatonic,
    /// 7-note harmonic minor — dramatic, cinematic
    HarmonicMinor,
    /// 6-note blues scale — jazzy color
    Blues,
}

impl MelodyScaleType {
    /// Convert to `rust_music_theory::ScaleType`
    #[must_use]
    pub fn to_rmt_scale_type(self, is_minor: bool) -> rust_music_theory::scale::ScaleType {
        use rust_music_theory::scale::ScaleType;
        match self {
            Self::Pentatonic => {
                if is_minor {
                    ScaleType::PentatonicMinor
                } else {
                    ScaleType::PentatonicMajor
                }
            }
            Self::Diatonic => ScaleType::Diatonic,
            Self::HarmonicMinor => ScaleType::HarmonicMinor,
            Self::Blues => ScaleType::Blues,
        }
    }
}

/// Convert a key_root (0=C, 1=C#, ..., 11=B) to `PitchSymbol`
#[must_use]
pub fn key_root_to_pitch_symbol(key_root: u8) -> PitchSymbol {
    match key_root % 12 {
        0 => PitchSymbol::C,
        1 => PitchSymbol::Cs,
        2 => PitchSymbol::D,
        3 => PitchSymbol::Eb,
        4 => PitchSymbol::E,
        5 => PitchSymbol::F,
        6 => PitchSymbol::Fs,
        7 => PitchSymbol::G,
        8 => PitchSymbol::Ab,
        9 => PitchSymbol::A,
        10 => PitchSymbol::Bb,
        11 => PitchSymbol::B,
        _ => PitchSymbol::C,
    }
}

/// Signature rythmique (numérateur/dénominateur)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSignature {
    pub numerator: usize,
    pub denominator: usize,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self { numerator: 4, denominator: 4 }
    }
}

impl TimeSignature {
    #[must_use]
    pub const fn new(numerator: usize, denominator: usize) -> Self {
        Self { numerator, denominator }
    }

    /// Calcule le nombre de steps par mesure selon une résolution (ticks par noire).
    /// Une résolution standard est de 4 ticks par noire (doubles croches).
    #[must_use]
    pub const fn steps_per_bar(&self, ticks_per_beat: usize) -> usize {
        // numerator * (ticks_per_beat * 4 / denominator)
        // Pour 4/4, 4 * (4 * 4 / 4) = 16 steps.
        // Pour 3/4, 3 * (4 * 4 / 4) = 12 steps.
        // Pour 7/8, 7 * (4 * 4 / 8) = 14 steps.
        (self.numerator * ticks_per_beat * 4) / self.denominator
    }
}

/// Specifies the dance pulse (the beat the player acts on) of a time
/// signature, derived from the notated meter rather than the quarter-note
/// tempo (constitution : le tempo reste à la noire, la pulse est dérivée).
///
/// The pulse is expressed as a number of grid steps and a number of pulses
/// per bar. For every supported meter (denominator 2, 4 or 8) the pulse tiles
/// the bar exactly: `steps_per_pulse * pulses_per_bar == steps_per_bar`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PulseSpec {
    /// Number of grid steps in one pulse.
    pub steps_per_pulse: u8,
    /// Number of pulses in one bar.
    pub pulses_per_bar: u8,
}

impl Default for PulseSpec {
    /// Default pulse: the quarter note of 4/4 (4 steps, 4 pulses).
    fn default() -> Self {
        Self { steps_per_pulse: 4, pulses_per_bar: 4 }
    }
}

impl PulseSpec {
    /// Derive the dance pulse from a time signature.
    ///
    /// - Compound meter — denominator 8 with a numerator that is a multiple
    ///   of 3 and strictly greater than 3 (6/8, 9/8, 12/8) — pulses on the
    ///   dotted quarter (3 eighth-note steps).
    /// - Any other meter pulses on the denominator note value: a quarter in
    ///   4/4 (4 steps), a half in 2/2 (8 steps), a quarter in 3/4 (4 steps), …
    ///
    /// The pulse tiles the bar exactly for the supported meters (denominator
    /// 2, 4 or 8). For an exotic meter where integer truncation in
    /// [`TimeSignature::steps_per_bar`] breaks that tiling, this falls back to
    /// pulse = one denominator step (documented, best effort).
    #[must_use]
    pub fn from_time_signature(ts: TimeSignature, ticks_per_beat: usize) -> Self {
        // Steps per denominator-note value (quarter for /4, eighth for /8, …).
        let steps_per_denominator = (ticks_per_beat * 4) / ts.denominator;

        let compound = ts.denominator == 8 && ts.numerator > 3 && ts.numerator.is_multiple_of(3);

        let (steps_per_pulse, pulses_per_bar) = if compound {
            (steps_per_denominator * 3, ts.numerator / 3)
        } else {
            (steps_per_denominator, ts.numerator)
        };

        // Invariant: the pulse must tile the bar exactly (see struct doc).
        let fits = steps_per_pulse * pulses_per_bar == ts.steps_per_bar(ticks_per_beat);

        let (steps_per_pulse, pulses_per_bar) = if fits {
            (steps_per_pulse, pulses_per_bar)
        } else {
            // Fallback: pulse = one denominator step (pulses = numerator).
            (steps_per_denominator, ts.numerator)
        };

        // Meter inputs are constrained in practice (ticks_per_beat = 4, small
        // denominators), so values always fit u8; saturate rather than panic.
        Self {
            steps_per_pulse: u8::try_from(steps_per_pulse).unwrap_or(u8::MAX),
            pulses_per_bar: u8::try_from(pulses_per_bar).unwrap_or(u8::MAX),
        }
    }
}

/// Per-track instrument configuration for note range and transposition.
///
/// Applies after all musical decisions, before storage in `TimelineNote`.
/// Default is transparent: full MIDI range, no transposition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstrumentConfig {
    /// Lowest allowed MIDI note (inclusive)
    pub min_note: u8,
    /// Highest allowed MIDI note (inclusive)
    pub max_note: u8,
    /// Transposition in semitones. Positive = written pitch higher than concert pitch.
    /// E.g., Bb instruments (tenor sax, trumpet) = +2.
    pub transposition_semitones: i16,
}

impl Default for InstrumentConfig {
    fn default() -> Self {
        Self { min_note: 0, max_note: 127, transposition_semitones: 0 }
    }
}

impl InstrumentConfig {
    /// Apply transposition and range restriction to a raw MIDI note.
    ///
    /// 1. Adds `transposition_semitones` to the raw note
    /// 2. Octave-folds the result into `[min_note, max_note]` (±12 until in range)
    /// 3. Falls back to hard clamp if range < 12 semitones
    #[must_use]
    pub fn apply(&self, raw_note: u8) -> u8 {
        let transposed = i16::from(raw_note) + self.transposition_semitones;
        let mut note = transposed.clamp(0, 127) as u8;

        let min = self.min_note;
        let max = self.max_note;

        if min > max || note >= min && note <= max {
            return note;
        }

        let range = max - min;
        if range >= 12 {
            // Octave-fold: shift by ±12 until in range
            while note < min {
                note += 12;
            }
            while note > max {
                note -= 12;
            }
            // Safety: if folding overshoots (shouldn't with range>=12), clamp
            note.clamp(min, max)
        } else {
            // Range too narrow for octave folding — hard clamp
            note.clamp(min, max)
        }
    }

    // === Factory constructors ===

    /// Tenor saxophone: Bb transposition (+2), range Ab3–F#6 (MIDI 56–90)
    #[must_use]
    pub const fn tenor_sax() -> Self {
        Self { min_note: 56, max_note: 90, transposition_semitones: 2 }
    }

    /// Alto saxophone: Eb transposition (-3), range Db3–A5 (MIDI 56–90 written)
    #[must_use]
    pub const fn alto_sax() -> Self {
        Self { min_note: 56, max_note: 90, transposition_semitones: -3 }
    }

    /// Soprano saxophone: Bb transposition (+2), range Ab3–F#6 (MIDI 56–90)
    #[must_use]
    pub const fn soprano_sax() -> Self {
        Self { min_note: 56, max_note: 90, transposition_semitones: 2 }
    }

    /// Baritone saxophone: Eb transposition (-3), range Db3–A5 (MIDI 56–90 written)
    #[must_use]
    pub const fn baritone_sax() -> Self {
        Self { min_note: 56, max_note: 90, transposition_semitones: -3 }
    }

    /// Trumpet: Bb transposition (+2), range G3–Bb5 (MIDI 55–82)
    #[must_use]
    pub const fn trumpet() -> Self {
        Self { min_note: 55, max_note: 82, transposition_semitones: 2 }
    }

    /// Concert pitch instrument with custom range, no transposition
    #[must_use]
    pub const fn concert_pitch(min_note: u8, max_note: u8) -> Self {
        Self { min_note, max_note, transposition_semitones: 0 }
    }

    // === MusicXML export helpers ===

    /// Returns a human-readable instrument name for MusicXML part naming.
    /// Returns `None` for default (transparent) config.
    #[must_use]
    pub fn instrument_name(&self) -> Option<&'static str> {
        match (self.transposition_semitones, self.min_note, self.max_note) {
            (2, 56, 90) => Some("Tenor Saxophone"),
            (-3, 56, 90) => Some("Alto Saxophone"),
            (2, 55, 82) => Some("Trumpet"),
            (0, 0, 127) => None, // default — transparent
            _ => Some("Transposing Instrument"),
        }
    }

    /// Returns MusicXML `<transpose>` values: `(chromatic, diatonic)`.
    ///
    /// In MusicXML, notes are written at **written pitch** and `<transpose>`
    /// tells the renderer how to convert to concert (sounding) pitch:
    ///   concert = written + chromatic
    ///
    /// Returns `None` when transposition is zero (concert pitch).
    #[must_use]
    pub fn musicxml_transpose(&self) -> Option<(i16, i16)> {
        if self.transposition_semitones == 0 {
            return None;
        }
        // chromatic = -transposition_semitones (our convention: positive = written higher)
        let chromatic = -self.transposition_semitones;
        // diatonic: map absolute semitones to diatonic steps, then apply sign
        let abs_semitones = chromatic.unsigned_abs() % 12;
        let abs_diatonic = match abs_semitones {
            0 => 0,
            1 => 1,  // minor/major 2nd
            2 => 1,  // major 2nd
            3 => 2,  // minor 3rd
            4 => 2,  // major 3rd
            5 => 3,  // perfect 4th
            6 => 3,  // tritone
            7 => 4,  // perfect 5th
            8 => 5,  // minor 6th
            9 => 5,  // major 6th
            10 => 6, // minor 7th
            11 => 6, // major 7th
            _ => 0,
        };
        let diatonic = if chromatic < 0 { -(abs_diatonic as i16) } else { abs_diatonic as i16 };
        Some((chromatic, diatonic))
    }
}

/// Tension state from TRQ (Tension-Rhythmic-Quality) Matrix
/// Maps emotional input (arousal, valence, tension) to coherent
/// harmonic and rhythmic tension combinations.
///
/// The four states represent different combinations of harmonic tension
/// (LCC level) and rhythmic oddity:
/// - **Stability**: Low LCC (1-3), Low Oddity (4-5) — Calm, consonant, predictable
/// - **HarmonicSuspense**: High LCC (4-7), Low Oddity (4-5) — Dissonant harmony, stable rhythm
/// - **RhythmicDrive**: Low LCC (1-3), High Oddity (6-7+) — Consonant harmony, syncopated rhythm
/// - **PeakClimax**: High LCC (4-7), High Oddity (6-7+) — Maximum tension on both dimensions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TensionState {
    /// Low harmonic tension, low rhythmic oddity (calm, stable)
    Stability,
    /// High harmonic tension, low rhythmic oddity (suspenseful, dissonant)
    HarmonicSuspense,
    /// Low harmonic tension, high rhythmic oddity (driving, syncopated)
    RhythmicDrive,
    /// High harmonic tension, high rhythmic oddity (peak climax, chaotic)
    PeakClimax,
}

impl Default for TensionState {
    fn default() -> Self {
        Self::Stability
    }
}

/// État du mode de contrôle (émotion vs direct)
/// Partagé entre VST et standalone builds
#[derive(Clone, Debug)]
pub struct ControlMode {
    /// true = mode émotion (`EngineParams` → `EmotionMapper` → `MusicalParams`)
    /// false = mode direct (`MusicalParams` directement)
    pub use_emotion_mode: bool,
    /// Paramètres musicaux directs (utilisés quand `use_emotion_mode` = false)
    pub direct_params: MusicalParams,

    // === GLOBAL ENABLE OVERRIDES ===
    // Ces flags s'appliquent dans TOUS les modes (émotion ET direct)
    /// Enable rhythm module (global override)
    pub enable_rhythm: bool,
    /// Enable harmony module (global override)
    pub enable_harmony: bool,
    /// Enable melody module (global override)
    pub enable_melody: bool,
    /// Enable voicing (harmonized chords) - global override
    pub enable_voicing: bool,
    /// Mode Drum Kit (kick fixe sur C1/36)
    pub fixed_kick: bool,

    // === WEBVIEW CONTROL FLAGS ===
    /// When true, the webview is the source of truth for emotional params
    /// and `sync_params_to_engine` should NOT overwrite `target_state`
    pub webview_controls_emotions: bool,
    /// When true, the webview controls the emotion/direct mode switch
    pub webview_controls_mode: bool,
    /// When true, the webview controls direct/technical params
    pub webview_controls_direct: bool,

    // === LIVE STATE FROM ENGINE ===
    // Updated by the engine during processing for UI visualization
    /// Current step in the sequencer (updated by engine)
    pub current_step: u32,
    /// Current measure number
    pub current_measure: u32,
    /// Current time signature
    pub time_signature: TimeSignature,
    /// Total number of bars in the current sequence/song
    pub total_bars: usize,
    /// Primary pattern (for visualization)
    pub primary_pattern: Vec<bool>,
    /// Secondary pattern (for visualization)
    pub secondary_pattern: Vec<bool>,
    /// Current chord name
    pub current_chord: String,
    /// Whether current chord is minor
    pub is_minor_chord: bool,
    /// Progression name
    pub progression_name: String,
    /// Session key (e.g., "C")
    pub session_key: String,
    /// Session scale (e.g., "major")
    pub session_scale: String,
}

impl Default for ControlMode {
    fn default() -> Self {
        Self {
            use_emotion_mode: true,
            direct_params: MusicalParams::default(),
            // All modules enabled by default
            enable_rhythm: true,
            enable_harmony: true,
            enable_melody: true,
            enable_voicing: false,
            fixed_kick: false,
            // Start with DAW params as source of truth
            webview_controls_emotions: false,
            webview_controls_mode: false,
            webview_controls_direct: false,
            // Live state
            current_step: 0,
            current_measure: 1,
            time_signature: TimeSignature::default(),
            total_bars: 4,
            primary_pattern: vec![],
            secondary_pattern: vec![],
            current_chord: "I".to_string(),
            is_minor_chord: false,
            progression_name: String::new(),
            session_key: "C".to_string(),
            session_scale: "major".to_string(),
        }
    }
}

/// Stratégie harmonique explicite (pour le Driver)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HarmonyStrategy {
    /// Grammaire de Steedman (harmonie fonctionnelle: ii-V-I, etc.)
    #[default]
    Steedman,
    /// Transformations Neo-Riemannian (P, L, R - triades uniquement)
    NeoRiemannian,
    /// Voice-leading parsimonieux (tous types d'accords)
    Parsimonious,
    /// Laisser le Driver décider automatiquement selon la tension
    Auto,
}

/// Choix de générateur : Legacy ou moteur piloté par un profil de données.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Style {
    /// Générateur procédural legacy (transitoire jusqu'à P1-T6).
    #[default]
    #[serde(alias = "Legacy")]
    Legacy,
    /// Générateur traditionnel piloté par `profile_id` et le profil du tuning.
    Profile,
}

/// Paramètres purement techniques. Le moteur obéit à ça directement.
/// Aucune interprétation émotionnelle - juste des valeurs musicales concrètes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MusicalParams {
    // ═══════════════════════════════════════════════════════════════════
    // GLOBAL
    // ═══════════════════════════════════════════════════════════════════
    /// BPM direct (pas calculé depuis arousal)
    pub bpm: f32,

    /// Time Signature (numerator/denominator)
    #[serde(default)]
    pub time_signature: TimeSignature,

    /// Swing ratio applied to pairs of eighth notes (unit = 4 steps, one
    /// quarter note). 0.5 = straight, ~0.6 ≈ reel. Bounded 0.5..=0.75 via
    /// [`MusicalParams::set_swing_ratio`]. Ignored in compound meter (a
    /// dotted-quarter pulse of 6 steps).
    #[serde(default = "default_swing_ratio")]
    pub swing_ratio: f32,

    /// Dance pulse override. `None` = derived from `time_signature` (see
    /// [`MusicalParams::effective_pulse`]).
    #[serde(default)]
    pub pulse: Option<PulseSpec>,

    /// Volume master (0.0 - 1.0)
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,

    /// Style de génération musicale (`Legacy` par défaut).
    #[serde(default)]
    pub style: Style,
    /// Identifiant du profil installé ; partagé sans allocation au clonage audio.
    #[serde(default = "default_profile_id")]
    pub profile_id: std::sync::Arc<str>,
    /// Instrumentarium résolu sur le thread principal pour le rendu et les exports.
    #[serde(default = "default_profile_instruments")]
    pub profile_instruments: crate::style_profile::ProfileInstruments,

    /// Densité de podorythmie (0..=2, défaut 1 ; bornée via [`MusicalParams::set_feet_density`]).
    #[serde(default = "default_feet_density")]
    pub feet_density: u8,

    /// Énergie du style (0..1), point neutre 0.5 : conserve le rendu historique.
    #[serde(default = "default_density")]
    pub intensity: f32,

    // ═══════════════════════════════════════════════════════════════════
    // MODULES ON/OFF (pour debug/tests)
    // ═══════════════════════════════════════════════════════════════════
    /// Activer le module rythmique
    #[serde(default = "default_true")]
    pub enable_rhythm: bool,

    /// Activer le module harmonique
    #[serde(default = "default_true")]
    pub enable_harmony: bool,

    /// Activer le module mélodique
    #[serde(default = "default_true")]
    pub enable_melody: bool,

    /// Activer le voicing (accords harmonisés sur la mélodie)
    /// Quand désactivé, seule la note mélodique joue
    #[serde(default)]
    pub enable_voicing: bool,

    /// Arrêter la génération après le dernier tour de l'air courant
    /// (utilisé pour l'outro en mode export avec `--bars`).
    #[serde(default)]
    pub end_after_tour: bool,

    // ═══════════════════════════════════════════════════════════════════
    // RYTHME
    // ═══════════════════════════════════════════════════════════════════
    /// Mode rythmique (Euclidean classique ou `PerfectBalance` polyrythmique)
    #[serde(default)]
    pub rhythm_mode: RhythmMode,

    /// Nombre de steps (16 pour Euclidean, 48/96/192 pour `PerfectBalance`)
    #[serde(default = "default_rhythm_steps")]
    pub rhythm_steps: usize,

    /// Nombre de pulses primaires (kicks/accents principaux)
    #[serde(default = "default_rhythm_pulses")]
    pub rhythm_pulses: usize,

    /// Rotation du pattern primaire (décalage de phase)
    #[serde(default)]
    pub rhythm_rotation: usize,

    /// Densité rythmique pour `PerfectBalance` (0.0-1.0)
    /// Contrôle la complexité du pattern de kicks
    #[serde(default = "default_density")]
    pub rhythm_density: f32,

    /// Tension rythmique (0.0-1.0)
    /// Contrôle les ghost notes et syncopation
    #[serde(default = "default_tension")]
    pub rhythm_tension: f32,

    /// Steps du séquenceur secondaire (polyrythme 4:3)
    #[serde(default = "default_secondary_steps")]
    pub rhythm_secondary_steps: usize,

    /// Pulses du séquenceur secondaire
    #[serde(default = "default_secondary_pulses")]
    pub rhythm_secondary_pulses: usize,

    /// Rotation du séquenceur secondaire
    #[serde(default)]
    pub rhythm_secondary_rotation: usize,

    /// Mode du Kick : true = Note fixe 36 (Drum Kit), false = Harmonisé (Synth Bass)
    /// - true: Idéal pour VST Drums, Samplers, Percussions
    /// - false: Idéal pour Synthwave, Basses mélodiques
    #[serde(default)]
    pub fixed_kick: bool,

    // ═══════════════════════════════════════════════════════════════════
    // HARMONIE
    // ═══════════════════════════════════════════════════════════════════
    /// Mode harmonique (Basic quadrants ou Driver avancé)
    #[serde(default)]
    pub harmony_mode: HarmonyMode,

    /// Stratégie harmonique explicite (pour Driver)
    #[serde(default)]
    pub harmony_strategy: HarmonyStrategy,

    /// Tension harmonique (0.0-1.0)
    /// Contrôle le niveau LCC (consonance → dissonance)
    /// et la sélection de stratégie si Auto
    #[serde(default = "default_tension")]
    pub harmony_tension: f32,

    /// Valence harmonique (-1.0 à 1.0)
    /// Contrôle la sélection majeur/mineur et les progressions
    #[serde(default)]
    pub harmony_valence: f32,

    /// Mesures par accord (1 = changement rapide, 2 = normal)
    #[serde(default = "default_measures_per_chord")]
    pub harmony_measures_per_chord: usize,

    /// Tonique globale (pitch class 0-11, 0 = C)
    #[serde(default)]
    pub key_root: u8,

    /// Fixed chord chart (chord names like "Cmaj7", "Dm7", "G7").
    /// When non-empty and `harmony_mode == Chart`, the generator cycles through
    /// these chords instead of generating procedurally. Auto-loops at chart end.
    #[serde(default)]
    pub chord_chart: Vec<ArrayString<16>>,

    // ═══════════════════════════════════════════════════════════════════
    // MÉLODIE / VOICING
    // ═══════════════════════════════════════════════════════════════════
    /// Lissage mélodique (facteur de Hurst, 0.0-1.0)
    /// 0.0 = erratique, 1.0 = très lisse
    #[serde(default = "default_smoothness")]
    pub melody_smoothness: f32,

    /// Scale type for melody generation (CORELIB-22)
    /// Default: Diatonic (7 notes, good pitch variety)
    #[serde(default)]
    pub melody_scale_type: MelodyScaleType,

    /// Variety/anti-repetition controls (CORELIB-13 tunable)
    #[serde(default)]
    pub variety: VarietyParams,

    /// Densité de voicing (0.0-1.0)
    /// Contrôle la probabilité de jouer des accords vs notes seules
    #[serde(default = "default_density")]
    pub voicing_density: f32,

    /// Tension de voicing (0.0-1.0)
    /// Affecte le nombre de voix et les extensions
    #[serde(default = "default_tension")]
    pub voicing_tension: f32,

    /// Octave de base pour la mélodie (3-6)
    #[serde(default = "default_octave")]
    pub melody_octave: i32,

    /// Instrument config for the lead/melody track
    #[serde(default)]
    pub instrument_lead: InstrumentConfig,

    /// Instrument config for the bass track
    #[serde(default)]
    pub instrument_bass: InstrumentConfig,

    // ═══════════════════════════════════════════════════════════════════
    // MIXER
    // ═══════════════════════════════════════════════════════════════════
    /// Gain du lead/mélodie (0.0-1.0)
    #[serde(default = "default_gain_lead")]
    pub gain_lead: f32,

    /// Gain de l'accompagnement accords/guitare (0.0-1.0)
    #[serde(default = "default_gain_chords")]
    pub gain_chords: f32,

    /// Gain de la basse/kick (0.0-1.0)
    #[serde(default = "default_gain_bass")]
    pub gain_bass: f32,

    /// Gain du snare (0.0-1.0)
    #[serde(default = "default_gain_snare")]
    pub gain_snare: f32,

    /// Gain du hi-hat (0.0-1.0)
    #[serde(default = "default_gain_hat")]
    pub gain_hat: f32,

    /// Vélocité de base pour la basse (0-127)
    #[serde(default = "default_vel_bass")]
    pub vel_base_bass: u8,

    /// Vélocité de base pour le snare (0-127)
    #[serde(default = "default_vel_snare")]
    pub vel_base_snare: u8,

    // ═══════════════════════════════════════════════════════════════════
    // ROUTAGE & MUTING
    // ═══════════════════════════════════════════════════════════════════
    /// Routage des canaux MIDI (-1 = banque 0 par défaut, >=0 = banque oxisynth)
    #[serde(default = "default_channel_routing")]
    pub channel_routing: Vec<i32>,

    /// Canaux mutés (true = silencieux)
    #[serde(default = "default_muted_channels")]
    pub muted_channels: Vec<bool>,

    // ═══════════════════════════════════════════════════════════════════
    // ENREGISTREMENT
    // ═══════════════════════════════════════════════════════════════════
    /// Enregistrer en WAV
    #[serde(default)]
    pub record_wav: bool,

    /// Enregistrer en MIDI
    #[serde(default)]
    pub record_midi: bool,

    /// Enregistrer en `MusicXML` (pour validation dans `MuseScore`)
    #[serde(default)]
    pub record_musicxml: bool,
}

// === Fonctions par défaut ===

const fn default_master_volume() -> f32 {
    1.0
}
const fn default_feet_density() -> u8 {
    1
}
/// Swing par défaut : droit (0.5). Partagé par `MusicalParams` et
/// `MeasureSnapshot` (serde `default`), d'où sa visibilité `pub`.
pub const fn default_swing_ratio() -> f32 {
    0.5
}
const fn default_true() -> bool {
    true
}
const fn default_rhythm_steps() -> usize {
    16
}
const fn default_rhythm_pulses() -> usize {
    4
}
const fn default_secondary_steps() -> usize {
    12
}
const fn default_secondary_pulses() -> usize {
    3
}
const fn default_density() -> f32 {
    0.5
}
const fn default_tension() -> f32 {
    0.3
}
const fn default_smoothness() -> f32 {
    0.7
}
const fn default_measures_per_chord() -> usize {
    2
}
const fn default_octave() -> i32 {
    4
}
const fn default_gain_lead() -> f32 {
    1.0
}
const fn default_gain_chords() -> f32 {
    0.6
}
const fn default_gain_bass() -> f32 {
    0.6
}
const fn default_gain_snare() -> f32 {
    0.5
}
const fn default_gain_hat() -> f32 {
    0.4
}
const fn default_vel_bass() -> u8 {
    85
}
const fn default_vel_snare() -> u8 {
    70
}
fn default_channel_routing() -> Vec<i32> {
    vec![-1; 16]
}
fn default_muted_channels() -> Vec<bool> {
    vec![false; 16]
}

fn default_profile_id() -> std::sync::Arc<str> {
    std::sync::Arc::from(crate::StyleProfile::builtin().meta.id.as_str())
}
fn default_profile_instruments() -> crate::style_profile::ProfileInstruments {
    crate::StyleProfile::builtin().instruments
}

impl Default for MusicalParams {
    fn default() -> Self {
        Self {
            // Global
            bpm: 120.0,
            time_signature: TimeSignature::default(),
            swing_ratio: default_swing_ratio(),
            pulse: None,
            master_volume: default_master_volume(),
            style: Style::default(),
            profile_id: default_profile_id(),
            profile_instruments: default_profile_instruments(),
            feet_density: default_feet_density(),
            intensity: default_density(),

            // Modules
            enable_rhythm: true,
            enable_harmony: true,
            enable_melody: true,
            enable_voicing: false,
            end_after_tour: false,

            // Rythme
            rhythm_mode: RhythmMode::Euclidean,
            rhythm_steps: default_rhythm_steps(),
            rhythm_pulses: default_rhythm_pulses(),
            rhythm_rotation: 0,
            rhythm_density: default_density(),
            rhythm_tension: default_tension(),
            rhythm_secondary_steps: default_secondary_steps(),
            rhythm_secondary_pulses: default_secondary_pulses(),
            rhythm_secondary_rotation: 0,
            fixed_kick: false,

            // Harmonie
            harmony_mode: HarmonyMode::Driver,
            harmony_strategy: HarmonyStrategy::Auto,
            harmony_tension: default_tension(),
            harmony_valence: 0.3,
            harmony_measures_per_chord: default_measures_per_chord(),
            key_root: 0, // C
            chord_chart: Vec::new(),

            // Mélodie / Voicing
            melody_smoothness: default_smoothness(),
            melody_scale_type: MelodyScaleType::default(),
            variety: VarietyParams::default(),
            voicing_density: default_density(),
            voicing_tension: default_tension(),
            melody_octave: default_octave(),
            instrument_lead: InstrumentConfig::default(),
            instrument_bass: InstrumentConfig::default(),

            // Mixer
            gain_lead: default_gain_lead(),
            gain_chords: default_gain_chords(),
            gain_bass: default_gain_bass(),
            gain_snare: default_gain_snare(),
            gain_hat: default_gain_hat(),
            vel_base_bass: default_vel_bass(),
            vel_base_snare: default_vel_snare(),

            // Routage
            channel_routing: default_channel_routing(),
            muted_channels: default_muted_channels(),

            // Recording
            record_wav: false,
            record_midi: false,
            record_musicxml: false,
        }
    }
}

impl MusicalParams {
    /// Créer des paramètres pour un test/debug avec rythme désactivé
    #[must_use]
    pub fn melody_only() -> Self {
        Self {
            enable_rhythm: false,
            enable_harmony: true,
            enable_melody: true,
            voicing_density: 1.0, // Force 100% de notes
            ..Default::default()
        }
    }

    /// Créer des paramètres pour un test/debug de rythme seul
    #[must_use]
    pub fn rhythm_only() -> Self {
        Self {
            enable_rhythm: true,
            enable_harmony: false,
            enable_melody: false,
            ..Default::default()
        }
    }

    /// Créer des paramètres pour un tempo spécifique
    #[must_use]
    pub fn with_bpm(bpm: f32) -> Self {
        Self { bpm, ..Default::default() }
    }

    /// Dance pulse effective for these params: the explicit override, or the
    /// one derived from `time_signature` when `pulse` is `None`.
    #[must_use]
    pub fn effective_pulse(&self, ticks_per_beat: usize) -> PulseSpec {
        self.pulse
            .unwrap_or_else(|| PulseSpec::from_time_signature(self.time_signature, ticks_per_beat))
    }

    /// Set the swing ratio, clamped to the valid range 0.5..=0.75.
    pub fn set_swing_ratio(&mut self, ratio: f32) {
        self.swing_ratio = ratio.clamp(0.5, 0.75);
    }

    /// Définit le style de génération (`Legacy` ou `Reel`).
    pub const fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    /// Définit la densité de podorythmie, bornée à `0..=2`.
    pub fn set_feet_density(&mut self, density: u8) {
        self.feet_density = density.min(2);
    }

    /// Définit l'énergie ; une entrée non finie revient au point neutre.
    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = if intensity.is_finite() { intensity.clamp(0.0, 1.0) } else { 0.5 };
    }

    /// Builder pattern: set BPM
    #[must_use]
    pub const fn bpm(mut self, bpm: f32) -> Self {
        self.bpm = bpm;
        self
    }

    /// Builder pattern: set harmony mode
    #[must_use]
    pub const fn harmony_mode(mut self, mode: HarmonyMode) -> Self {
        self.harmony_mode = mode;
        self
    }

    /// Builder pattern: set rhythm mode
    #[must_use]
    pub const fn rhythm_mode(mut self, mode: RhythmMode) -> Self {
        self.rhythm_mode = mode;
        self
    }

    /// Builder pattern: set chord chart
    #[must_use]
    pub fn chord_chart(mut self, chart: Vec<ArrayString<16>>) -> Self {
        self.chord_chart = chart;
        self
    }

    /// Builder pattern: set lead instrument config
    #[must_use]
    pub const fn instrument_lead(mut self, config: InstrumentConfig) -> Self {
        self.instrument_lead = config;
        self
    }

    /// Builder pattern: set bass instrument config
    #[must_use]
    pub const fn instrument_bass(mut self, config: InstrumentConfig) -> Self {
        self.instrument_bass = config;
        self
    }

    /// Builder pattern: set musical style
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Builder pattern: set feet density (clamped to `0..=2`)
    #[must_use]
    pub const fn feet_density(mut self, density: u8) -> Self {
        self.feet_density = if density > 2 { 2 } else { density };
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = MusicalParams::default();
        assert!((params.bpm - 120.0).abs() < f32::EPSILON);
        assert!(params.enable_rhythm);
        assert!(params.enable_harmony);
        assert!(params.enable_melody);
    }

    #[test]
    fn test_melody_only() {
        let params = MusicalParams::melody_only();
        assert!(!params.enable_rhythm);
        assert!(params.enable_melody);
        assert!((params.voicing_density - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_builder_pattern() {
        let params = MusicalParams::default().bpm(140.0).harmony_mode(HarmonyMode::Basic);

        assert!((params.bpm - 140.0).abs() < f32::EPSILON);
        assert_eq!(params.harmony_mode, HarmonyMode::Basic);
    }

    #[test]
    fn test_instrument_config_default_transparent() {
        let config = InstrumentConfig::default();
        // Default config should return the input unchanged
        for note in [0u8, 60, 127] {
            assert_eq!(config.apply(note), note);
        }
    }

    #[test]
    fn test_instrument_config_transposition() {
        let config = InstrumentConfig { transposition_semitones: 2, ..Default::default() };
        assert_eq!(config.apply(60), 62); // C4 → D4
        assert_eq!(config.apply(0), 2);

        let config_neg = InstrumentConfig { transposition_semitones: -3, ..Default::default() };
        assert_eq!(config_neg.apply(63), 60); // Eb4 → C4
    }

    #[test]
    fn test_instrument_config_octave_fold() {
        let config = InstrumentConfig { min_note: 56, max_note: 90, transposition_semitones: 0 };

        // Note below range should fold up by octaves
        assert_eq!(config.apply(44), 56); // 44 + 12 = 56
        assert_eq!(config.apply(32), 56); // 32 + 12 = 44, + 12 = 56

        // Note above range should fold down by octaves
        assert_eq!(config.apply(96), 84); // 96 - 12 = 84
        assert_eq!(config.apply(100), 88); // 100 - 12 = 88
    }

    #[test]
    fn test_instrument_config_transposition_and_fold() {
        let config = InstrumentConfig::tenor_sax(); // +2, 56-90
        // 54 + 2 = 56 → in range
        assert_eq!(config.apply(54), 56);
        // 90 + 2 = 92 → fold down: 92 - 12 = 80 → in range
        assert_eq!(config.apply(90), 80);
    }

    #[test]
    fn test_instrument_config_narrow_range_clamp() {
        // Range < 12 semitones → hard clamp
        let config = InstrumentConfig { min_note: 60, max_note: 67, transposition_semitones: 0 };
        assert_eq!(config.apply(50), 60);
        assert_eq!(config.apply(80), 67);
        assert_eq!(config.apply(64), 64);
    }

    #[test]
    fn test_factory_constructors_valid_ranges() {
        let configs = [
            InstrumentConfig::tenor_sax(),
            InstrumentConfig::alto_sax(),
            InstrumentConfig::soprano_sax(),
            InstrumentConfig::baritone_sax(),
            InstrumentConfig::trumpet(),
            InstrumentConfig::concert_pitch(48, 84),
        ];
        for config in &configs {
            assert!(config.min_note <= config.max_note);
            assert!(config.max_note <= 127);
        }
    }

    #[test]
    fn test_instrument_config_builder() {
        let params = MusicalParams::default()
            .instrument_lead(InstrumentConfig::tenor_sax())
            .instrument_bass(InstrumentConfig::concert_pitch(28, 55));
        assert_eq!(params.instrument_lead.transposition_semitones, 2);
        assert_eq!(params.instrument_bass.min_note, 28);
    }

    /// Table-driven check of `PulseSpec::from_time_signature`, including the
    /// tiling invariant `steps_per_pulse * pulses_per_bar == steps_per_bar`.
    #[test]
    fn test_pulse_spec_from_time_signature() {
        const TPB: usize = 4;
        // (numerator, denominator, expected steps_per_pulse, expected pulses_per_bar)
        let cases: [(usize, usize, u8, u8); 6] = [
            (4, 4, 4, 4),  // 4/4 → quarter
            (2, 2, 8, 2),  // 2/2 → half
            (3, 4, 4, 3),  // 3/4 → quarter
            (6, 8, 6, 2),  // 6/8 → dotted quarter
            (9, 8, 6, 3),  // 9/8 → dotted quarter
            (12, 8, 6, 4), // 12/8 → dotted quarter
        ];

        for (num, den, steps_per_pulse, pulses_per_bar) in cases {
            let ts = TimeSignature::new(num, den);
            let pulse = PulseSpec::from_time_signature(ts, TPB);

            assert_eq!(pulse.steps_per_pulse, steps_per_pulse, "{num}/{den}");
            assert_eq!(pulse.pulses_per_bar, pulses_per_bar, "{num}/{den}");

            // Tiling invariant must hold for every supported meter.
            assert_eq!(
                usize::from(pulse.steps_per_pulse) * usize::from(pulse.pulses_per_bar),
                ts.steps_per_bar(TPB),
                "pulse does not tile {num}/{den}",
            );
        }
    }

    /// The pulse override wins over the derived one; `None` derives from the
    /// signature.
    #[test]
    fn test_effective_pulse_override() {
        let params = MusicalParams::default();
        assert_eq!(params.effective_pulse(4), PulseSpec { steps_per_pulse: 4, pulses_per_bar: 4 });

        // 6/8 derives a dotted-quarter pulse (6 steps).
        let gigue =
            MusicalParams { time_signature: TimeSignature::new(6, 8), ..MusicalParams::default() };
        assert_eq!(gigue.effective_pulse(4).steps_per_pulse, 6);

        // An explicit override wins over the derived pulse.
        let overridden = MusicalParams {
            time_signature: TimeSignature::new(6, 8),
            pulse: Some(PulseSpec { steps_per_pulse: 3, pulses_per_bar: 4 }),
            ..MusicalParams::default()
        };
        assert_eq!(
            overridden.effective_pulse(4),
            PulseSpec { steps_per_pulse: 3, pulses_per_bar: 4 }
        );
    }

    #[test]
    fn test_swing_ratio_default_and_bounds() {
        assert!((MusicalParams::default().swing_ratio - 0.5).abs() < f32::EPSILON);

        let mut params = MusicalParams::default();
        params.set_swing_ratio(0.6);
        assert!((params.swing_ratio - 0.6).abs() < f32::EPSILON);

        // Below the lower bound → clamped to 0.5.
        params.set_swing_ratio(0.2);
        assert!((params.swing_ratio - 0.5).abs() < f32::EPSILON);

        // Above the upper bound → clamped to 0.75.
        params.set_swing_ratio(1.0);
        assert!((params.swing_ratio - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_style_and_feet_density_defaults_and_bounds() {
        let mut params = MusicalParams::default();
        assert_eq!(params.style, Style::Legacy);
        assert_eq!(params.feet_density, 1);

        params.set_style(Style::Profile);
        assert_eq!(params.style, Style::Profile);

        params.set_feet_density(0);
        assert_eq!(params.feet_density, 0);
        params.set_feet_density(2);
        assert_eq!(params.feet_density, 2);
        params.set_feet_density(99);
        assert_eq!(params.feet_density, 2);

        let built = MusicalParams::default().style(Style::Profile).feet_density(5);
        assert_eq!(built.style, Style::Profile);
        assert_eq!(built.feet_density, 2);
    }
}

// =========================================================================================
//  Structures moved from Engine (Data Contracts)
// =========================================================================================

#[derive(Clone, Debug)]
pub struct VisualizationEvent {
    pub note_midi: u8,
    pub instrument: u8, // 0=Melody, 1=Chords, 2=Bass, 3=Heel, 4=Toe
    pub step: usize,
    pub bar: usize,
    pub duration_samples: usize,
}

#[derive(Clone, Debug)]
pub struct HarmonyState {
    pub current_chord_index: usize,
    pub chord_root_offset: i32,
    pub chord_is_minor: bool,
    pub chord_name: ArrayString<64>,
    pub measure_number: usize,
    pub cycle_number: usize,
    pub current_step: usize,
    pub progression_name: ArrayString<64>,
    pub progression_length: usize,
    pub harmony_mode: HarmonyMode,
    pub primary_steps: usize,
    pub primary_pulses: usize,
    pub secondary_steps: usize,
    pub secondary_pulses: usize,
    pub primary_rotation: usize,
    pub secondary_rotation: usize,
    pub primary_pattern: [bool; 192],
    pub secondary_pattern: [bool; 192],
}

impl Default for HarmonyState {
    fn default() -> Self {
        Self {
            current_chord_index: 0,
            chord_root_offset: 0,
            chord_is_minor: false,
            chord_name: ArrayString::from("I").unwrap_or_default(),
            measure_number: 1,
            cycle_number: 1,
            current_step: 0,
            progression_name: ArrayString::from("Folk Peaceful (I-IV-I-V)").unwrap_or_default(),
            progression_length: 4,
            harmony_mode: HarmonyMode::Driver,
            primary_steps: 16,
            primary_pulses: 4,
            secondary_steps: 12,
            secondary_pulses: 3,
            primary_rotation: 0,
            secondary_rotation: 0,
            primary_pattern: [false; 192],
            secondary_pattern: [false; 192],
        }
    }
}

// Helpers for Serde defaults
const fn default_poly_steps() -> usize {
    48
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineParams {
    pub arousal: f32,
    pub valence: f32,
    pub density: f32,
    pub tension: f32,
    pub smoothness: f32,
    #[serde(default)]
    pub algorithm: RhythmMode,
    #[serde(default)]
    pub channel_routing: Vec<i32>,
    #[serde(default)]
    pub muted_channels: Vec<bool>,
    #[serde(default)]
    pub harmony_mode: HarmonyMode,

    // Recording Control
    #[serde(default)]
    pub record_wav: bool,
    #[serde(default)]
    pub record_midi: bool,
    #[serde(default)]
    pub record_musicxml: bool,

    // Synthesis Morphing Control
    #[serde(default = "default_true")]
    pub enable_synthesis_morphing: bool,

    // Mixer Gains (0.0 - 1.0)
    #[serde(default = "default_gain_lead")]
    pub gain_lead: f32,
    #[serde(default = "default_gain_chords")]
    pub gain_chords: f32,
    #[serde(default = "default_gain_bass")]
    pub gain_bass: f32,
    #[serde(default = "default_gain_snare")]
    pub gain_snare: f32,
    #[serde(default = "default_gain_hat")]
    pub gain_hat: f32,

    // Velocity Base (MIDI 0-127)
    #[serde(default = "default_vel_bass")]
    pub vel_base_bass: u8,
    #[serde(default = "default_vel_snare")]
    pub vel_base_snare: u8,

    // Polyrythm Steps (48, 96, 192...)
    #[serde(default = "default_poly_steps")]
    pub poly_steps: usize,

    // Mode Drum Kit
    #[serde(default)]
    pub fixed_kick: bool,
}

impl Default for EngineParams {
    fn default() -> Self {
        Self {
            arousal: 0.5,
            valence: 0.3,
            density: 0.2,
            tension: 0.4,
            smoothness: 0.7,
            algorithm: RhythmMode::Euclidean,
            channel_routing: vec![-1; 16],
            muted_channels: vec![false; 16],
            harmony_mode: HarmonyMode::Driver,
            record_wav: false,
            record_midi: false,
            record_musicxml: false,
            enable_synthesis_morphing: true,
            gain_lead: default_gain_lead(),
            gain_chords: default_gain_chords(),
            gain_bass: default_gain_bass(),
            gain_snare: default_gain_snare(),
            gain_hat: default_gain_hat(),
            vel_base_bass: default_vel_bass(),
            vel_base_snare: default_vel_snare(),
            poly_steps: default_poly_steps(),
            fixed_kick: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SessionConfig {
    pub bpm: f32,
    pub key: String,
    pub scale: String,
    pub pulses: usize,
    pub steps: usize,
}

#[derive(Clone, Debug, Default)]
pub struct CurrentState {
    pub bpm: f32,
    pub density: f32,
    pub tension: f32,
    pub smoothness: f32,
    pub valence: f32,
    pub arousal: f32,
}

/// Musical Conductor - Tracks the absolute musical time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Conductor {
    pub current_bar: usize,
    pub current_beat: usize,
    pub current_tick: usize,
    pub time_signature: TimeSignature,
    pub ticks_per_beat: usize,
}

impl Default for Conductor {
    fn default() -> Self {
        Self {
            current_bar: 1,
            current_beat: 1,
            current_tick: 0,
            time_signature: TimeSignature::default(),
            ticks_per_beat: 4, // 16th notes resolution
        }
    }
}

impl Conductor {
    /// Advance the conductor by one tick.
    /// Returns true if a barline was crossed.
    pub fn tick(&mut self) -> bool {
        let mut bar_crossed = false;
        self.current_tick += 1;

        if self.current_tick >= self.ticks_per_beat {
            self.current_tick = 0;
            self.current_beat += 1;

            if self.current_beat > self.time_signature.numerator {
                self.current_beat = 1;
                self.current_bar += 1;
                bar_crossed = true;
            }
        }
        bar_crossed
    }

    /// Reset the conductor to the beginning.
    pub fn reset(&mut self) {
        self.current_bar = 1;
        self.current_beat = 1;
        self.current_tick = 0;
    }
}

impl EngineParams {
    #[must_use]
    pub fn compute_bpm(&self) -> f32 {
        self.arousal.mul_add(110.0, 70.0)
    }
}

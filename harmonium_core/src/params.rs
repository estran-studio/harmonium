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

use serde::{Deserialize, Serialize};

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

/// Paramètres purement techniques. Le moteur obéit à ça directement.
/// Aucune interprétation émotionnelle - juste des valeurs musicales concrètes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MusicalParams {
    // ═══════════════════════════════════════════════════════════════════
    // GLOBAL
    // ═══════════════════════════════════════════════════════════════════
    /// BPM direct (pas calculé depuis arousal)
    pub bpm: f32,
    /// Commande directe du jeu : ni offset émotionnel ni rampe de set sur le tempo.
    #[serde(default)]
    pub bpm_explicit: bool,

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

    /// Identifiant du profil installé ; partagé sans allocation au clonage audio.
    #[serde(default = "default_profile_id")]
    pub profile_id: std::sync::Arc<str>,
    /// Instrumentarium résolu sur le thread principal pour le rendu et les exports.
    #[serde(default = "default_profile_instruments")]
    pub profile_instruments: crate::style_profile::ProfileInstruments,
    /// Contrôleurs et coefficients préparés sur le thread principal au taux de l'hôte.
    #[serde(default = "default_profile_mix")]
    pub profile_mix: crate::mix::PreparedMix,
    /// Pistes Odin résolues sur le thread principal (noms + règles, `Copy`).
    #[serde(default = "default_profile_odin")]
    pub profile_odin: [crate::odin::OdinTrackDesc; crate::timeline::TrackId::ALL.len()],

    /// Densité de podorythmie (0..=2, défaut 1 ; bornée via [`MusicalParams::set_feet_density`]).
    #[serde(default = "default_feet_density")]
    pub feet_density: u8,

    /// Énergie du style (0..1), point neutre 0.5 : conserve le rendu historique.
    #[serde(default = "default_density")]
    pub intensity: f32,
    /// Axe arousal brut ; distinct de l'intensité combinée avec la tension.
    #[serde(default = "default_density")]
    pub arousal: f32,

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

    /// Arrêter la génération après le dernier tour de l'air courant
    /// (utilisé pour l'outro en mode export avec `--bars`).
    #[serde(default)]
    pub end_after_tour: bool,

    // ═══════════════════════════════════════════════════════════════════
    // RYTHME
    // ═══════════════════════════════════════════════════════════════════
    /// Densité de texture (0..1), interprétée par les courbes du profil.
    #[serde(default = "default_density")]
    pub rhythm_density: f32,

    /// Tension rythmique (0.0-1.0)
    /// Contrôle les ghost notes et syncopation
    #[serde(default = "default_tension")]
    pub rhythm_tension: f32,

    /// Tension et valence transmises aux courbes du profil.
    #[serde(default = "default_tension")]
    pub harmony_tension: f32,
    #[serde(default)]
    pub harmony_valence: f32,
    #[serde(default = "default_smoothness")]
    pub melody_smoothness: f32,

    /// Instrument config for the lead/melody track
    #[serde(default)]
    pub instrument_lead: InstrumentConfig,

    /// Instrument config for the bass track
    #[serde(default)]
    pub instrument_bass: InstrumentConfig,

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
const fn default_density() -> f32 {
    0.5
}
const fn default_tension() -> f32 {
    0.3
}
const fn default_smoothness() -> f32 {
    0.7
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
fn default_profile_mix() -> crate::mix::PreparedMix {
    crate::StyleProfile::builtin().mix.prepare(44100.0)
}
fn default_profile_odin() -> [crate::odin::OdinTrackDesc; crate::timeline::TrackId::ALL.len()] {
    describe_odin_tracks(&std::collections::BTreeMap::new())
}

/// Vue `Copy` des pistes Odin d'un profil, résolue sur le thread principal.
#[must_use]
pub fn describe_odin_tracks(
    odin: &std::collections::BTreeMap<crate::timeline::TrackId, crate::odin::OdinTrackSpec>,
) -> [crate::odin::OdinTrackDesc; crate::timeline::TrackId::ALL.len()] {
    let mut descs = [crate::odin::OdinTrackDesc::default(); crate::timeline::TrackId::ALL.len()];
    for (index, track) in crate::timeline::TrackId::ALL.iter().enumerate() {
        if let Some(spec) = odin.get(track) {
            // Profil validé au chargement : la description réussit toujours.
            if let Ok(desc) = spec.describe(*track) {
                descs[index] = desc;
            }
        }
    }
    descs
}

/// Longueur maximale d'un identifiant de profil porté par [`PlaybackParams`].
pub const PROFILE_ID_CAP: usize = 64;

/// Vue `Copy` de [`MusicalParams`] pour le thread audio, les rapports et
/// l'enregistreur : tout ce que le callback lit, sans `Vec`, `Box` ni `Arc`.
///
/// Préparée sur le thread principal ([`MusicalParams::playback_params`]),
/// comme `PreparedMix` : la déplacer ou la remplacer dans le callback ne
/// libère rien. Un identifiant de profil plus long que [`PROFILE_ID_CAP`]
/// octets est tronqué à une frontière de caractère (les profils livrés
/// font moins de 32 octets).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaybackParams {
    pub bpm: f32,
    pub time_signature: TimeSignature,
    pub swing_ratio: f32,
    pub pulse: Option<PulseSpec>,
    pub profile_id: arrayvec::ArrayString<PROFILE_ID_CAP>,
    pub instruments: crate::style_profile::ProfileInstruments,
    pub mix: crate::mix::PreparedMix,
    /// Pistes Odin (`[odin.<Piste>]` du profil), vide = tout SoundFont.
    pub odin: [crate::odin::OdinTrackDesc; crate::timeline::TrackId::ALL.len()],
    pub feet_density: u8,
    pub enable_rhythm: bool,
    pub enable_harmony: bool,
    pub enable_melody: bool,
}

impl PlaybackParams {
    /// Dance pulse effective : le remplacement explicite, sinon celui de la
    /// signature (même règle que [`MusicalParams::effective_pulse`]).
    #[must_use]
    pub fn effective_pulse(&self, ticks_per_beat: usize) -> PulseSpec {
        self.pulse
            .unwrap_or_else(|| PulseSpec::from_time_signature(self.time_signature, ticks_per_beat))
    }
}

impl Default for PlaybackParams {
    fn default() -> Self {
        MusicalParams::default().playback_params()
    }
}

impl MusicalParams {
    /// Vue `Copy` pour le callback audio, les rapports et l'enregistreur.
    #[must_use]
    pub fn playback_params(&self) -> PlaybackParams {
        let mut profile_id = arrayvec::ArrayString::new();
        for ch in self.profile_id.chars() {
            if profile_id.try_push(ch).is_err() {
                break;
            }
        }
        PlaybackParams {
            bpm: self.bpm,
            time_signature: self.time_signature,
            swing_ratio: self.swing_ratio,
            pulse: self.pulse,
            profile_id,
            instruments: self.profile_instruments,
            mix: self.profile_mix,
            odin: self.profile_odin,
            feet_density: self.feet_density,
            enable_rhythm: self.enable_rhythm,
            enable_harmony: self.enable_harmony,
            enable_melody: self.enable_melody,
        }
    }
}

impl Default for MusicalParams {
    fn default() -> Self {
        let profile = crate::StyleProfile::builtin();
        Self {
            // Global
            bpm: profile.meter.default_bpm,
            bpm_explicit: false,
            time_signature: profile.meter.time_signature,
            swing_ratio: profile.meter.default_swing_ratio,
            pulse: Some(profile.meter.pulse_spec()),
            master_volume: default_master_volume(),
            profile_id: default_profile_id(),
            profile_instruments: default_profile_instruments(),
            profile_mix: default_profile_mix(),
            profile_odin: default_profile_odin(),
            feet_density: default_feet_density(),
            intensity: default_density(),
            arousal: default_density(),

            // Modules
            enable_rhythm: true,
            enable_harmony: true,
            enable_melody: true,
            end_after_tour: false,

            // Rythme
            rhythm_density: default_density(),
            rhythm_tension: default_tension(),
            harmony_tension: default_tension(),
            harmony_valence: 0.3,
            melody_smoothness: default_smoothness(),
            instrument_lead: InstrumentConfig::default(),
            instrument_bass: InstrumentConfig::default(),

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
    /// Défauts de tempo et de mètre du profil (tempo nominal, swing, signature, pulse) :
    /// seule source pour le composer et le plan partagé (`time_plan::score`).
    pub fn apply_profile_meter(&mut self, profile: &crate::StyleProfile) {
        let meter = &profile.meter;
        self.bpm = meter.default_bpm;
        self.bpm_explicit = false;
        self.set_swing_ratio(meter.default_swing_ratio);
        self.time_signature = meter.time_signature;
        self.pulse = Some(meter.pulse_spec());
    }

    /// Créer des paramètres pour un test/debug avec rythme désactivé
    #[must_use]
    pub fn melody_only() -> Self {
        Self {
            enable_rhythm: false,
            enable_harmony: true,
            enable_melody: true,
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
    fn test_melody_only() {
        let params = MusicalParams::melody_only();
        assert!(!params.enable_rhythm);
        assert!(params.enable_melody);
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
        let params = MusicalParams {
            pulse: None,
            time_signature: TimeSignature::new(4, 4),
            ..Default::default()
        };
        assert_eq!(params.effective_pulse(4), PulseSpec { steps_per_pulse: 4, pulses_per_bar: 4 });

        // 6/8 derives a dotted-quarter pulse (6 steps).
        let compound = MusicalParams {
            time_signature: TimeSignature::new(6, 8),
            pulse: None,
            ..MusicalParams::default()
        };
        assert_eq!(compound.effective_pulse(4).steps_per_pulse, 6);

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
    fn test_feet_density_bounds() {
        let mut params = MusicalParams::default();

        params.set_feet_density(0);
        assert_eq!(params.feet_density, 0);
        params.set_feet_density(2);
        assert_eq!(params.feet_density, 2);
        params.set_feet_density(99);
        assert_eq!(params.feet_density, 2);

        let built = MusicalParams::default().feet_density(5);
        assert_eq!(built.feet_density, 2);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineParams {
    pub arousal: f32,
    pub valence: f32,
    pub density: f32,
    pub tension: f32,
    pub smoothness: f32,
}

impl Default for EngineParams {
    fn default() -> Self {
        Self { arousal: 0.5, valence: 0.3, density: 0.2, tension: 0.4, smoothness: 0.7 }
    }
}

impl EngineParams {
    /// Axes `[arousal, valence, density, tension]` pour le morphing Odin.
    #[must_use]
    pub const fn emotions(&self) -> [f32; 4] {
        [self.arousal, self.valence, self.density, self.tension]
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

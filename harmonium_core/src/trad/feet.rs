//! Podorythmie traditionnelle québécoise et table de patterns pour le reel (D9).
//!
//! En musique traditionnelle québécoise, la podorythmie (battement des pieds)
//! fournit l'assise rythmique principale avec le violon.
//!
//! Ce module implémente :
//! - [`FootHit`] : un coup de pied unitaire `(step, piste Heel|Toe, delta_velocity)` ;
//! - [`FeetPattern`] : pattern de podorythmie pour une densité donnée ;
//! - [`FeetTable`] : table complète portée en données (constitution V) via `reel_feet.toml` ;
//! - [`feet_notes`] : fonction pure générant les [`TimelineNote`] de talon et de pointe.

use std::{collections::HashSet, fmt};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::plan::TunePlan;
use crate::{
    algo::RngCore,
    timeline::{Articulation, TimelineNote, TrackId},
};

/// Contenu brut de la table TOML embarquée de podorythmie pour le reel (D9).
pub const REEL_FEET_TOML: &str = include_str!("../../../profiles/reel_quebecois/feet.toml");

/// Un coup de pied individuel dans une mesure de podorythmie (D9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FootHit {
    /// Pas dans la mesure (`0..16`, soit `0..=15`).
    pub step: usize,
    /// Piste ciblée ([`TrackId::Heel`] ou [`TrackId::Toe`]).
    pub track: TrackId,
    /// Ajustement de vélocité relatif à la vélocité de base du membre.
    pub delta_velocity: i8,
    /// Note MIDI explicite (optionnelle, surpasse le défaut de piste/voix).
    pub note: Option<u8>,
    /// Voix percussive ciblée (optionnelle).
    pub voice: Option<String>,
}

impl FootHit {
    /// Crée un coup de pied avec pas, piste et delta de vélocité.
    #[must_use]
    pub const fn new(step: usize, track: TrackId, delta_velocity: i8) -> Self {
        Self { step, track, delta_velocity, note: None, voice: None }
    }

    /// Raccourci pour un coup de talon (delta de vélocité nul).
    #[must_use]
    pub const fn heel(step: usize) -> Self {
        Self { step, track: TrackId::Heel, delta_velocity: 0, note: None, voice: None }
    }

    /// Raccourci pour un coup de pointe (delta de vélocité nul).
    #[must_use]
    pub const fn toe(step: usize) -> Self {
        Self { step, track: TrackId::Toe, delta_velocity: 0, note: None, voice: None }
    }

    /// Raccourci pour un coup de pointe avec delta de vélocité explicite.
    #[must_use]
    pub const fn toe_with_delta(step: usize, delta_velocity: i8) -> Self {
        Self { step, track: TrackId::Toe, delta_velocity, note: None, voice: None }
    }

    /// Coup avec note explicite.
    #[must_use]
    pub const fn with_note(step: usize, track: TrackId, note: u8, delta_velocity: i8) -> Self {
        Self { step, track, delta_velocity, note: Some(note), voice: None }
    }

    /// Coup avec voix percussive nommée.
    #[must_use]
    pub fn with_voice(step: usize, voice: impl Into<String>, delta_velocity: i8) -> Self {
        Self { step, track: TrackId::Toe, delta_velocity, note: None, voice: Some(voice.into()) }
    }
}

impl From<(usize, TrackId)> for FootHit {
    fn from((step, track): (usize, TrackId)) -> Self {
        Self { step, track, delta_velocity: 0, note: None, voice: None }
    }
}

impl From<(usize, TrackId, i8)> for FootHit {
    fn from((step, track, delta_velocity): (usize, TrackId, i8)) -> Self {
        Self { step, track, delta_velocity, note: None, voice: None }
    }
}

impl From<FootHit> for (usize, TrackId, i8) {
    fn from(hit: FootHit) -> Self {
        (hit.step, hit.track, hit.delta_velocity)
    }
}

fn deserialize_foot_track<'de, D>(deserializer: D) -> Result<TrackId, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_ascii_lowercase().as_str() {
        "heel" => Ok(TrackId::Heel),
        "toe" => Ok(TrackId::Toe),
        _ => Err(serde::de::Error::custom(format!(
            "piste invalide pour la podorythmie : attendu 'Heel' ou 'Toe', reçu '{s}'"
        ))),
    }
}

fn deserialize_optional_foot_track<'de, D>(deserializer: D) -> Result<Option<TrackId>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    let Some(s) = opt else { return Ok(None) };
    match s.to_ascii_lowercase().as_str() {
        "heel" => Ok(Some(TrackId::Heel)),
        "toe" => Ok(Some(TrackId::Toe)),
        _ => Err(serde::de::Error::custom(format!(
            "piste invalide pour la podorythmie : attendu 'Heel' ou 'Toe', reçu '{s}'"
        ))),
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawFootHit {
    Struct {
        step: usize,
        #[serde(default, deserialize_with = "deserialize_optional_foot_track")]
        track: Option<TrackId>,
        #[serde(default)]
        voice: Option<String>,
        #[serde(default)]
        note: Option<u8>,
        #[serde(default)]
        delta_velocity: i8,
    },
    Tuple2(usize, #[serde(deserialize_with = "deserialize_foot_track")] TrackId),
    Tuple3(usize, #[serde(deserialize_with = "deserialize_foot_track")] TrackId, i8),
}

impl<'de> Deserialize<'de> for FootHit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawFootHit::deserialize(deserializer)?;
        match raw {
            RawFootHit::Struct { step, track, voice, note, delta_velocity } => {
                let track = track.unwrap_or(if voice.as_deref() == Some("bass_drum") {
                    TrackId::Heel
                } else {
                    TrackId::Toe
                });
                Ok(Self { step, track, delta_velocity, note, voice })
            }
            RawFootHit::Tuple2(step, track) => {
                Ok(Self { step, track, delta_velocity: 0, note: None, voice: None })
            }
            RawFootHit::Tuple3(step, track, delta_velocity) => {
                Ok(Self { step, track, delta_velocity, note: None, voice: None })
            }
        }
    }
}

#[derive(Serialize)]
struct RawFootHitSerialize<'a> {
    step: usize,
    track: &'a str,
    #[serde(skip_serializing_if = "is_zero_i8")]
    delta_velocity: i8,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    voice: Option<&'a str>,
}

const fn is_zero_i8(v: &i8) -> bool {
    *v == 0
}

impl Serialize for FootHit {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let track_str = match self.track {
            TrackId::Heel => "Heel",
            TrackId::Toe => "Toe",
            _ => "Unknown",
        };
        let raw = RawFootHitSerialize {
            step: self.step,
            track: track_str,
            delta_velocity: self.delta_velocity,
            note: self.note,
            voice: self.voice.as_deref(),
        };
        raw.serialize(serializer)
    }
}

/// Pattern de coups de pieds pour un niveau de densité donné (D9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeetPattern {
    /// Niveau de densité (`0`, `1` ou `2`).
    pub density: u8,
    /// Poids strictement positif lors du choix parmi les alternatives de même densité.
    pub weight: u32,
    /// Liste des coups composant la mesure.
    pub hits: Vec<FootHit>,
    euclid: Option<(EuclideanPattern, TrackId)>,
}

impl FeetPattern {
    /// Crée un nouveau [`FeetPattern`].
    #[must_use]
    pub const fn new(density: u8, hits: Vec<FootHit>) -> Self {
        Self { density, weight: 1, hits, euclid: None }
    }

    pub(crate) fn euclidean_steps(&self) -> Option<usize> {
        self.euclid.map(|(pattern, _)| pattern.steps)
    }
}

const fn default_pattern_weight() -> u32 {
    1
}

/// Motif euclidien en pas ; rotation positive vers la droite.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EuclideanPattern {
    pub pulses: usize,
    pub steps: usize,
    #[serde(default)]
    pub rotation: i32,
}

impl<'de> Deserialize<'de> for FeetPattern {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            density: u8,
            #[serde(default = "default_pattern_weight")]
            weight: u32,
            hits: Option<Vec<FootHit>>,
            euclid: Option<EuclideanPattern>,
            #[serde(default)]
            track: Option<String>,
        }
        let raw = Raw::deserialize(deserializer)?;
        let mut euclid = None;
        let hits = match (raw.hits, raw.euclid) {
            (Some(hits), None) if raw.track.is_none() => hits,
            (None, Some(e)) => {
                if !(1..=4096).contains(&e.steps) || e.pulses > e.steps {
                    return Err(serde::de::Error::custom(
                        "euclid : steps dans 1..4096 et pulses <= steps requis",
                    ));
                }
                let track = match raw.track.as_deref().unwrap_or("Heel") {
                    "Heel" | "heel" => TrackId::Heel,
                    "Toe" | "toe" => TrackId::Toe,
                    _ => return Err(serde::de::Error::custom("track : Heel ou Toe requis")),
                };
                euclid = Some((e, track));
                let mut pattern =
                    crate::algo::euclidean::generate_euclidean_bools(e.steps, e.pulses);
                pattern.rotate_right(e.rotation.rem_euclid(e.steps as i32) as usize);
                pattern
                    .into_iter()
                    .enumerate()
                    .filter_map(|(step, hit)| hit.then_some(FootHit::new(step, track, 0)))
                    .collect()
            }
            (None, None) if raw.track.is_none() => Vec::new(),
            _ => return Err(serde::de::Error::custom("choisir hits ou euclid, pas les deux")),
        };
        Ok(Self { density: raw.density, weight: raw.weight, hits, euclid })
    }
}

impl Serialize for FeetPattern {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer
            .serialize_struct("FeetPattern", if self.euclid.is_some() { 4 } else { 3 })?;
        state.serialize_field("density", &self.density)?;
        state.serialize_field("weight", &self.weight)?;
        if let Some((pattern, track)) = self.euclid {
            state.serialize_field("euclid", &pattern)?;
            state.serialize_field("track", if track == TrackId::Toe { "Toe" } else { "Heel" })?;
        } else {
            state.serialize_field("hits", &self.hits)?;
        }
        state.end()
    }
}

const fn default_heel_note() -> u8 {
    36
}

const fn default_toe_note() -> u8 {
    37
}

const fn default_velocity_humanize() -> u8 {
    12
}

const fn default_heel_velocity() -> u8 {
    55
}

const fn default_toe_velocity() -> u8 {
    45
}

const fn default_feet_gain() -> f32 {
    1.0
}

/// Voix percussive pour la batterie / percussion militaire ou podorythmie ornée.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PercussionVoice {
    pub name: String,
    pub note: u8,
    pub velocity: u8,
    #[serde(default)]
    pub track: Option<TrackId>,
}

#[derive(Deserialize)]
struct RawFeetTable {
    #[serde(default = "default_heel_note")]
    heel_note: u8,
    #[serde(default = "default_toe_note")]
    toe_note: u8,
    #[serde(default = "default_heel_velocity")]
    heel_velocity: u8,
    #[serde(default = "default_toe_velocity")]
    toe_velocity: u8,
    #[serde(default = "default_feet_gain")]
    feet_gain: f32,
    #[serde(default = "default_velocity_humanize")]
    velocity_humanize: u8,
    #[serde(default)]
    voices: Vec<PercussionVoice>,
    #[serde(default)]
    intro_tracks: Option<Vec<TrackId>>,
    #[serde(default)]
    patterns: Vec<FeetPattern>,
    #[serde(default)]
    densities: Vec<FeetPattern>,
    #[serde(default)]
    cadence_patterns: Vec<FeetPattern>,
}

/// Table de podorythmie portée en données (D9, constitution V).
///
/// Contient les notes GM et les vélocités de base du talon et de la pointe,
/// le gain global (`feet_gain`, `0..=1`) ainsi que les patterns associés à
/// chaque densité (`0..=2`).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FeetTable {
    /// Note GM du talon (canal 9 percussion, D10).
    #[serde(default = "default_heel_note")]
    pub heel_note: u8,
    /// Note GM de la pointe (canal 9 percussion, D10).
    #[serde(default = "default_toe_note")]
    pub toe_note: u8,
    /// Vélocité de base du talon (MIDI `0..=127`).
    #[serde(default = "default_heel_velocity")]
    pub heel_velocity: u8,
    /// Vélocité de base de la pointe (MIDI `0..=127`).
    #[serde(default = "default_toe_velocity")]
    pub toe_velocity: u8,
    /// Gain global appliqué aux vélocités effectives (`0..=1`).
    #[serde(default = "default_feet_gain")]
    pub feet_gain: f32,
    /// Amplitude d'humanisation des vélocités (`0..=40`, défaut 12) : chaque
    /// coup reçoit un décalage déterministe uniforme dans `±velocity_humanize`.
    #[serde(default = "default_velocity_humanize")]
    pub velocity_humanize: u8,
    /// Voix percussives nommées (pour kit de batterie / tambour militaire).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub voices: Vec<PercussionVoice>,
    /// Pistes actives lors du décompte d'intro percussion (défaut : [Heel]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intro_tracks: Option<Vec<TrackId>>,
    /// Patterns de coups par niveau de densité.
    #[serde(default)]
    pub patterns: Vec<FeetPattern>,
    /// Patterns de coups optionnels pour les cadences / fins de phrase.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cadence_patterns: Vec<FeetPattern>,
}

impl<'de> Deserialize<'de> for FeetTable {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawFeetTable::deserialize(deserializer)?;
        let patterns = if !raw.patterns.is_empty() { raw.patterns } else { raw.densities };
        Ok(Self {
            heel_note: raw.heel_note,
            toe_note: raw.toe_note,
            heel_velocity: raw.heel_velocity,
            toe_velocity: raw.toe_velocity,
            feet_gain: raw.feet_gain,
            velocity_humanize: raw.velocity_humanize,
            voices: raw.voices,
            intro_tracks: raw.intro_tracks,
            patterns,
            cadence_patterns: raw.cadence_patterns,
        })
    }
}

/// Erreur survenue lors du chargement ou de la validation d'une [`FeetTable`].
#[derive(Clone, Debug, PartialEq)]
pub enum FeetTableError {
    /// Erreur de syntaxe TOML.
    TomlParse(String),
    /// La table ne contient aucun pattern.
    EmptyTable,
    /// Une densité obligatoire (`0`, `1` ou `2`) est absente.
    MissingDensity(u8),
    /// Une densité apparaît plus d'une fois parmi les patterns de cadence.
    DuplicateCadenceDensity(u8),
    /// Le poids d'un pattern est nul.
    ZeroWeight { density: u8 },
    /// Piste invalide (seules `TrackId::Heel` et `TrackId::Toe` sont permises).
    InvalidTrack { density: u8, step: usize, track: TrackId },
    /// La vélocité effective (`base + delta`) dépasse la plage MIDI `0..=127`.
    VelocityOutOfRange { density: u8, step: usize, velocity: i16 },
    /// Une vélocité de base est hors de la plage MIDI `0..=127`.
    BaseVelocityOutOfRange { heel: u8, toe: u8 },
    /// Le gain global `feet_gain` est hors de la plage `0..=1`.
    GainOutOfRange { gain: f32 },
    /// L'humanisation `velocity_humanize` est hors de la plage `0..=40`.
    VelocityHumanizeOutOfRange { humanize: u8 },
    /// Voix percussive inconnue référencée par un coup.
    UnknownVoice { density: u8, step: usize, voice: String },
    /// Note MIDI hors limites (0..=127).
    NoteOutOfRange { density: u8, step: usize, note: u8 },
}

impl fmt::Display for FeetTableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TomlParse(msg) => {
                write!(f, "erreur de lecture TOML de la table de podorythmie : {msg}")
            }
            Self::EmptyTable => write!(f, "la table de podorythmie est vide"),
            Self::MissingDensity(density) => {
                write!(f, "densité requise manquante dans la table de podorythmie : {density}")
            }
            Self::DuplicateCadenceDensity(density) => {
                write!(f, "densité dupliquée dans les patterns de cadence : {density}")
            }
            Self::ZeroWeight { density } => {
                write!(f, "poids nul pour un pattern de percussion à la densité {density}")
            }
            Self::InvalidTrack { density, step, track } => {
                write!(
                    f,
                    "piste {track:?} invalide pour la podorythmie à la densité {density}, pas {step}"
                )
            }
            Self::VelocityOutOfRange { density, step, velocity } => {
                write!(
                    f,
                    "vélocité effective {velocity} hors limites MIDI (0..=127) à la densité {density}, pas {step}"
                )
            }
            Self::BaseVelocityOutOfRange { heel, toe } => {
                write!(
                    f,
                    "vélocité de base hors limites MIDI (0..=127) : talon={heel}, pointe={toe}"
                )
            }
            Self::GainOutOfRange { gain } => {
                write!(f, "gain global hors limites (0..=1) : feet_gain={gain}")
            }
            Self::VelocityHumanizeOutOfRange { humanize } => {
                write!(f, "humanisation hors limites (0..=40) : velocity_humanize={humanize}")
            }
            Self::UnknownVoice { density, step, voice } => {
                write!(f, "voix inconnue '{voice}' à la densité {density}, pas {step}")
            }
            Self::NoteOutOfRange { density, step, note } => {
                write!(
                    f,
                    "note MIDI {note} hors limites (0..=127) à la densité {density}, pas {step}"
                )
            }
        }
    }
}

impl std::error::Error for FeetTableError {}

impl Default for FeetTable {
    /// Charge la table embarquée `reel_feet.toml` sans jamais paniquer à
    /// l'exécution (en cas d'erreur de données, retourne une table par défaut
    /// vide ; la validité de `REEL_FEET_TOML` est garantie par les tests unitaires
    /// et [`crate::tuning::TuningParams::validate`]).
    fn default() -> Self {
        Self::from_toml(REEL_FEET_TOML).unwrap_or_else(|_| Self {
            heel_note: default_heel_note(),
            toe_note: default_toe_note(),
            heel_velocity: default_heel_velocity(),
            toe_velocity: default_toe_velocity(),
            feet_gain: default_feet_gain(),
            velocity_humanize: default_velocity_humanize(),
            voices: Vec::new(),
            intro_tracks: None,
            patterns: Vec::new(),
            cadence_patterns: Vec::new(),
        })
    }
}

impl FeetTable {
    /// Charge et valide une [`FeetTable`] depuis une chaîne TOML au format `reel_feet.toml`.
    pub fn from_toml(toml_str: &str) -> Result<Self, FeetTableError> {
        let table: Self =
            toml::from_str(toml_str).map_err(|e| FeetTableError::TomlParse(e.to_string()))?;
        table.validate()?;
        Ok(table)
    }

    /// Valide les contraintes d'intégrité de la table :
    /// - vélocités de base dans `0..=127` ;
    /// - gain global dans `0..=1` ;
    /// - densités `0`, `1` et `2` présentes, avec un seul pattern de cadence par densité ;
    /// - poids des patterns strictement positifs ;
    /// - chaque piste est `Heel` ou `Toe` ;
    /// - chaque vélocité calculée (`base + delta`) est dans `0..=127`.
    pub fn validate(&self) -> Result<(), FeetTableError> {
        if self.heel_velocity > 127 || self.toe_velocity > 127 {
            return Err(FeetTableError::BaseVelocityOutOfRange {
                heel: self.heel_velocity,
                toe: self.toe_velocity,
            });
        }
        if !(0.0..=1.0).contains(&self.feet_gain) {
            return Err(FeetTableError::GainOutOfRange { gain: self.feet_gain });
        }
        if self.velocity_humanize > 40 {
            return Err(FeetTableError::VelocityHumanizeOutOfRange {
                humanize: self.velocity_humanize,
            });
        }

        if self.patterns.is_empty() {
            return Err(FeetTableError::EmptyTable);
        }

        for voice in &self.voices {
            if voice.note > 127 || voice.velocity > 127 || voice.name.is_empty() {
                return Err(FeetTableError::BaseVelocityOutOfRange {
                    heel: self.heel_velocity,
                    toe: self.toe_velocity,
                });
            }
        }

        let validate_hit = |density: u8, hit: &FootHit| -> Result<(), FeetTableError> {
            let (base_velocity, pitch) = if let Some(voice_name) = &hit.voice {
                let Some(v) = self.voices.iter().find(|v| v.name == *voice_name) else {
                    return Err(FeetTableError::UnknownVoice {
                        density,
                        step: hit.step,
                        voice: voice_name.clone(),
                    });
                };
                (v.velocity, hit.note.unwrap_or(v.note))
            } else {
                let base = match hit.track {
                    TrackId::Heel => self.heel_velocity,
                    TrackId::Toe => self.toe_velocity,
                    other => {
                        return Err(FeetTableError::InvalidTrack {
                            density,
                            step: hit.step,
                            track: other,
                        });
                    }
                };
                let p = hit.note.unwrap_or(match hit.track {
                    TrackId::Heel => self.heel_note,
                    TrackId::Toe => self.toe_note,
                    _ => 0,
                });
                (base, p)
            };

            if pitch > 127 {
                return Err(FeetTableError::NoteOutOfRange {
                    density,
                    step: hit.step,
                    note: pitch,
                });
            }

            let effective_velocity = i16::from(base_velocity) + i16::from(hit.delta_velocity);
            if !(0..=127).contains(&effective_velocity) {
                return Err(FeetTableError::VelocityOutOfRange {
                    density,
                    step: hit.step,
                    velocity: effective_velocity,
                });
            }
            Ok(())
        };

        let mut seen_densities = HashSet::new();

        for pattern in &self.patterns {
            seen_densities.insert(pattern.density);
            if pattern.weight == 0 {
                return Err(FeetTableError::ZeroWeight { density: pattern.density });
            }

            for hit in &pattern.hits {
                validate_hit(pattern.density, hit)?;
            }
        }

        let mut seen_cadence = HashSet::new();
        for pattern in &self.cadence_patterns {
            if !seen_cadence.insert(pattern.density) {
                return Err(FeetTableError::DuplicateCadenceDensity(pattern.density));
            }
            if pattern.weight == 0 {
                return Err(FeetTableError::ZeroWeight { density: pattern.density });
            }
            for hit in &pattern.hits {
                validate_hit(pattern.density, hit)?;
            }
        }

        for required in 0..=2u8 {
            if !seen_densities.contains(&required) {
                return Err(FeetTableError::MissingDensity(required));
            }
        }

        Ok(())
    }

    /// Indique si au moins une densité possède plusieurs patterns ordinaires.
    #[must_use]
    pub fn has_alternatives(&self) -> bool {
        let mut seen = [false; 256];
        self.patterns.iter().any(|pattern| {
            let density = usize::from(pattern.density);
            let duplicate = seen[density];
            seen[density] = true;
            duplicate
        })
    }
}

/// Choix pondéré de percussion, stable pour chaque partie et densité d'un air.
///
/// Les reprises et les tours relisent le même choix. Les cadences, uniques par
/// densité, remplacent ce choix au rendu sans participer au tirage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeetPatternPlan {
    parts: Vec<[usize; 3]>,
}

impl FeetPatternPlan {
    /// Construit le plan depuis une table validée, sans tirage pour un candidat unique.
    #[must_use]
    pub fn new(plan: &TunePlan, table: &FeetTable, seed: u64) -> Self {
        let seed = crate::ornaments::mix(seed, 0x4645_4554);
        let parts = (0..plan.profile.form.parts.len())
            .map(|part| {
                std::array::from_fn(|density| {
                    let mut candidates = table
                        .patterns
                        .iter()
                        .enumerate()
                        .filter(|(_, pattern)| usize::from(pattern.density) == density);
                    let Some((first, pattern)) = candidates.next() else {
                        unreachable!("densité requise dans une table validée");
                    };
                    let Some((_, alternative)) = candidates.next() else {
                        return first;
                    };
                    let total = u64::from(pattern.weight)
                        + u64::from(alternative.weight)
                        + candidates.map(|(_, p)| u64::from(p.weight)).sum::<u64>();
                    let mut rng = ChaCha8Rng::seed_from_u64(crate::ornaments::mix(
                        crate::ornaments::mix(seed, part as u64),
                        density as u64,
                    ));
                    let mut draw = rng.next_range_u64(0..total);
                    for (index, candidate) in table
                        .patterns
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| usize::from(p.density) == density)
                    {
                        let weight = u64::from(candidate.weight);
                        if draw < weight {
                            return index;
                        }
                        draw -= weight;
                    }
                    unreachable!("poids cumulés cohérents dans une table validée")
                })
            })
            .collect();
        Self { parts }
    }

    /// Retourne le pattern choisi, ou son remplacement cadentiel s'il existe.
    #[must_use]
    pub fn pattern<'a>(
        &self,
        table: &'a FeetTable,
        part: usize,
        density: u8,
        is_cadence: bool,
    ) -> &'a FeetPattern {
        let density = density.min(2);
        if is_cadence
            && let Some(pattern) = table.cadence_patterns.iter().find(|p| p.density == density)
        {
            return pattern;
        }
        &table.patterns[self.parts[part][usize::from(density)]]
    }
}

/// Tire les décalages d'humanisation : une ligne par mesure distincte du
/// profil, une colonne par pas. Chaque
/// décalage est tiré uniformément dans `±humanize` via le RNG injecté
/// (déterministe : même graine ⇒ mêmes décalages). `humanize == 0` ne
/// consomme aucun tirage et produit des décalages tous nuls.
#[must_use]
pub fn draw_feet_humanize(
    humanize: u8,
    slots: usize,
    steps: usize,
    rng: &mut dyn RngCore,
) -> Vec<Vec<i8>> {
    let mut offsets = vec![vec![0i8; steps]; slots];
    let h = i32::from(humanize);
    if h == 0 {
        return offsets;
    }
    for slot in &mut offsets {
        for step in slot.iter_mut() {
            *step = (rng.next_range_u64(0..(2 * h + 1) as u64) as i32 - h) as i8;
        }
    }
    offsets
}

/// Génère les notes de podorythmie (talon, pointe) pour une mesure de reel (D9).
///
/// # Paramètres
/// - `table` : table de podorythmie ([`FeetTable`]) portant les vélocités de base.
/// - `pattern` : pattern choisi par [`FeetPatternPlan::pattern`], cadence comprise.
/// - `steps` : nombre de pas du profil ; `humanize` contient exactement ce nombre de valeurs.
///
/// # Retour
/// Tuple `(talon_notes, pointe_notes)` :
/// - premier élément : notes du talon ([`TrackId::Heel`]), `pitch = table.heel_note` ;
/// - second élément : notes de la pointe ([`TrackId::Toe`]), `pitch = table.toe_note`.
///
/// Toutes les notes produites ont `duration_steps = 0` et `articulation = Articulation::Trigger`.
/// La vélocité est
/// `((base_velocity + delta_velocity) * feet_gain).round() + humanize[step]`,
/// bornée à `1..=127`.
#[must_use]
pub fn feet_notes(
    table: &FeetTable,
    pattern: &FeetPattern,
    steps: usize,
    humanize: &[i8],
) -> (Vec<TimelineNote>, Vec<TimelineNote>) {
    debug_assert_eq!(humanize.len(), steps);

    let hits = &pattern.hits;

    let mut heel_notes = Vec::new();
    let mut toe_notes = Vec::new();

    for hit in hits {
        let (track, base_velocity, pitch) = if let Some(voice_name) = &hit.voice {
            if let Some(v) = table.voices.iter().find(|v| v.name == *voice_name) {
                (v.track.unwrap_or(hit.track), v.velocity, hit.note.unwrap_or(v.note))
            } else {
                (hit.track, table.toe_velocity, hit.note.unwrap_or(table.toe_note))
            }
        } else {
            let base = match hit.track {
                TrackId::Heel => table.heel_velocity,
                TrackId::Toe => table.toe_velocity,
                _ => continue,
            };
            let p = hit.note.unwrap_or(match hit.track {
                TrackId::Heel => table.heel_note,
                TrackId::Toe => table.toe_note,
                _ => continue,
            });
            (hit.track, base, p)
        };

        let target_vec = match track {
            TrackId::Heel => &mut heel_notes,
            TrackId::Toe => &mut toe_notes,
            _ => continue,
        };

        let effective = f32::from(base_velocity) + f32::from(hit.delta_velocity);
        let base = (effective * table.feet_gain).round() as i32;
        let velocity = (base + i32::from(humanize[hit.step])).clamp(1, 127) as u8;

        target_vec.push(TimelineNote {
            id: 0,
            pitch,
            start_step: hit.step,
            duration_steps: 0,
            velocity,
            articulation: Articulation::Trigger,
            performance: Default::default(),
        });
    }

    heel_notes.sort_by_key(|n| n.start_step);
    toe_notes.sort_by_key(|n| n.start_step);

    (heel_notes, toe_notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_density(
        table: &FeetTable,
        density: u8,
        humanize: &[i8],
    ) -> (Vec<TimelineNote>, Vec<TimelineNote>) {
        let plan = FeetPatternPlan::new(&TunePlan::default(), table, 0);
        feet_notes(table, plan.pattern(table, 0, density, false), humanize.len(), humanize)
    }

    #[test]
    fn test_embedded_table_validates() {
        let table = FeetTable::default();
        table.validate().expect("REEL_FEET_TOML doit être valide");
        assert_eq!(table.heel_note, 36);
        assert_eq!(table.toe_note, 37);
        assert_eq!(table.heel_velocity, 55);
        assert_eq!(table.toe_velocity, 45);
        assert_eq!(table.feet_gain, 0.9);
        assert_eq!(table.velocity_humanize, 12);
    }

    #[test]
    fn test_exact_d9_positions_density_0() {
        let table = FeetTable { feet_gain: 1.0, ..FeetTable::default() };
        let (heel, toe) = render_density(&table, 0, &[0; 16]);

        let heel_steps: Vec<usize> = heel.iter().map(|n| n.start_step).collect();
        assert_eq!(heel_steps, vec![0, 8]);
        assert!(toe.is_empty());

        for note in &heel {
            assert_eq!(note.pitch, 36);
            assert_eq!(note.velocity, 55);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }
    }

    #[test]
    fn test_exact_d9_positions_density_1() {
        let table = FeetTable { feet_gain: 1.0, ..FeetTable::default() };
        let (heel, toe) = render_density(&table, 1, &[0; 16]);

        let heel_steps: Vec<usize> = heel.iter().map(|n| n.start_step).collect();
        let toe_steps: Vec<usize> = toe.iter().map(|n| n.start_step).collect();

        assert_eq!(heel_steps, vec![0, 4, 8, 12]);
        assert_eq!(toe_steps, vec![2, 6, 10, 14]);

        for note in &heel {
            assert_eq!(note.pitch, 36);
            assert_eq!(note.velocity, 55);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }

        for note in &toe {
            assert_eq!(note.pitch, 37);
            assert_eq!(note.velocity, 45);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }
    }

    #[test]
    fn test_exact_d9_positions_density_2() {
        let table = FeetTable { feet_gain: 1.0, ..FeetTable::default() };
        let (heel, toe) = render_density(&table, 2, &[0; 16]);

        let heel_steps: Vec<usize> = heel.iter().map(|n| n.start_step).collect();
        let toe_steps: Vec<usize> = toe.iter().map(|n| n.start_step).collect();

        assert_eq!(heel_steps, vec![0, 4, 8, 12]);
        assert_eq!(toe_steps, vec![2, 3, 6, 7, 10, 11, 14, 15]);

        for note in &heel {
            assert_eq!(note.pitch, 36);
            assert_eq!(note.velocity, 55);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }

        for note in &toe {
            assert_eq!(note.pitch, 37);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
            if note.start_step % 2 == 1 {
                // Pas impairs : vélocité de base (45) - 20 = 25
                assert_eq!(note.velocity, 25, "pas impair {} attendu vel 25", note.start_step);
            } else {
                // Pas pairs : vélocité de base (45)
                assert_eq!(note.velocity, 45, "pas pair {} attendu vel 45", note.start_step);
            }
        }
    }

    #[test]
    fn test_density_clamping() {
        let table = FeetTable::default();
        let expected_2 = render_density(&table, 2, &[0; 16]);

        for d in [3, 4, 10, 99, 255] {
            let notes = render_density(&table, d, &[0; 16]);
            assert_eq!(notes.0.len(), expected_2.0.len(), "densité {d} talon");
            assert_eq!(notes.1.len(), expected_2.1.len(), "densité {d} pointe");
            assert_eq!(
                notes.0.iter().map(|n| n.start_step).collect::<Vec<_>>(),
                expected_2.0.iter().map(|n| n.start_step).collect::<Vec<_>>()
            );
            assert_eq!(
                notes.1.iter().map(|n| n.start_step).collect::<Vec<_>>(),
                expected_2.1.iter().map(|n| n.start_step).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_validation_missing_density() {
        let toml_str = r#"
heel_note = 35
toe_note = 37

[[patterns]]
density = 0
hits = [{ step = 0, track = "Heel" }]

[[patterns]]
density = 1
hits = [{ step = 0, track = "Heel" }]
"#;
        let err = FeetTable::from_toml(toml_str).unwrap_err();
        assert_eq!(err, FeetTableError::MissingDensity(2));
    }

    #[test]
    fn test_validation_empty_table() {
        let toml_str = r#"
heel_note = 35
toe_note = 37
"#;
        let err = FeetTable::from_toml(toml_str).unwrap_err();
        assert_eq!(err, FeetTableError::EmptyTable);
    }

    #[test]
    fn test_validation_base_velocity_out_of_range() {
        let mut table = FeetTable::default();
        table.heel_velocity = 128;
        assert_eq!(
            table.validate().unwrap_err(),
            FeetTableError::BaseVelocityOutOfRange { heel: 128, toe: 45 }
        );
    }

    #[test]
    fn test_validation_effective_velocity_out_of_range() {
        let toml_str = r#"
heel_note = 35
toe_note = 37
heel_velocity = 100
toe_velocity = 80

[[patterns]]
density = 0
hits = [{ step = 0, track = "Heel", delta_velocity = 50 }]

[[patterns]]
density = 1
hits = [{ step = 0, track = "Heel" }]

[[patterns]]
density = 2
hits = [{ step = 0, track = "Heel" }]
"#;
        let err = FeetTable::from_toml(toml_str).unwrap_err();
        assert_eq!(err, FeetTableError::VelocityOutOfRange { density: 0, step: 0, velocity: 150 });
    }

    #[test]
    fn test_custom_base_velocities() {
        let table = FeetTable {
            heel_velocity: 110,
            toe_velocity: 90,
            feet_gain: 1.0,
            ..FeetTable::default()
        };

        let (heel, toe) = render_density(&table, 2, &[0; 16]);
        assert_eq!(heel[0].velocity, 110);
        assert_eq!(toe[0].velocity, 90); // step 2
        assert_eq!(toe[1].velocity, 70); // step 3 (90 - 20)
    }

    #[test]
    fn test_feet_gain_scales_velocities() {
        let table = FeetTable {
            heel_velocity: 100,
            toe_velocity: 80,
            feet_gain: 0.5,
            ..FeetTable::default()
        };
        assert!(table.validate().is_ok(), "gain 0.5 valide");

        let (heel, toe) = render_density(&table, 2, &[0; 16]);
        assert!(heel.iter().all(|n| n.velocity == 50));
        assert_eq!(toe[0].velocity, 40); // step 2 : 80 * 0.5
        assert_eq!(toe[1].velocity, 30); // step 3 : (80 - 20) * 0.5
    }

    #[test]
    fn test_feet_gain_out_of_range_rejected() {
        for gain in [1.5, -0.5, f32::NAN] {
            let table = FeetTable { feet_gain: gain, ..FeetTable::default() };
            assert!(
                matches!(table.validate(), Err(FeetTableError::GainOutOfRange { .. })),
                "gain {gain} rejeté"
            );
        }
    }

    #[test]
    fn test_validation_velocity_humanize_bounds() {
        for humanize in [0, 1, 12, 40] {
            let table = FeetTable { velocity_humanize: humanize, ..FeetTable::default() };
            assert!(table.validate().is_ok(), "humanize {humanize} doit être valide");
        }
        for humanize in [41, 99, 255] {
            let table = FeetTable { velocity_humanize: humanize, ..FeetTable::default() };
            assert_eq!(
                table.validate(),
                Err(FeetTableError::VelocityHumanizeOutOfRange { humanize }),
                "humanize {humanize} doit être rejeté"
            );
        }
    }

    #[test]
    fn test_velocity_humanize_zero_is_constant() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let table = FeetTable { velocity_humanize: 0, feet_gain: 1.0, ..FeetTable::default() };
        let offsets = draw_feet_humanize(0, 16, 16, &mut rng);
        assert!(offsets.iter().flatten().all(|&o| o == 0));

        let (heel, toe) = render_density(&table, 1, &offsets[0]);
        assert!(heel.iter().all(|n| n.velocity == 55));
        assert!(toe.iter().all(|n| n.velocity == 45));
    }

    #[test]
    fn test_velocity_humanize_deterministic_and_bounded() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let table = FeetTable { velocity_humanize: 12, feet_gain: 1.0, ..FeetTable::default() };

        let mut rng_a = ChaCha8Rng::seed_from_u64(42);
        let mut rng_b = ChaCha8Rng::seed_from_u64(42);
        let offs_a = draw_feet_humanize(table.velocity_humanize, 16, 16, &mut rng_a);
        let offs_b = draw_feet_humanize(table.velocity_humanize, 16, 16, &mut rng_b);
        assert_eq!(offs_a, offs_b, "même graine ⇒ mêmes décalages");

        for slot in &offs_a {
            for &off in slot.iter() {
                assert!((-12..=12).contains(&off), "décalage {off} hors ±12");
            }
        }

        let (heel, toe) = render_density(&table, 1, &offs_a[0]);
        for n in heel.iter().chain(toe.iter()) {
            assert!((1..=127).contains(&n.velocity), "vélocité {} hors 1..=127", n.velocity);
        }
        assert!(heel.iter().all(|n| (43..=67).contains(&n.velocity)));
        assert!(toe.iter().all(|n| (33..=57).contains(&n.velocity)));
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut table = FeetTable::default();
        table.patterns[0].weight = 7;
        let toml_str = toml::to_string(&table).expect("serialize TOML");
        let restored: FeetTable = toml::from_str(&toml_str).expect("deserialize TOML");
        assert_eq!(table, restored);

        let json_str = serde_json::to_string(&table).expect("serialize JSON");
        let restored_json: FeetTable = serde_json::from_str(&json_str).expect("deserialize JSON");
        assert_eq!(table, restored_json);
    }

    #[test]
    fn test_tuple_syntax_in_toml() {
        let toml_str = r#"
heel_note = 35
toe_note = 37

[[patterns]]
density = 0
hits = [[0, "Heel"], [8, "heel", 0]]

[[patterns]]
density = 1
hits = [[0, "Heel"], [2, "Toe"]]

[[patterns]]
density = 2
hits = [[0, "Heel"], [3, "Toe", -20]]
"#;
        let table = FeetTable::from_toml(toml_str).expect("tuple syntax should parse");
        assert_eq!(table.patterns[0].hits[0], FootHit::heel(0));
        assert_eq!(table.patterns[0].hits[1], FootHit::heel(8));
        assert_eq!(table.patterns[2].hits[1], FootHit::toe_with_delta(3, -20));
    }

    #[test]
    fn weighted_alternatives_render_deterministically_by_part_and_density() {
        let table = FeetTable::from_toml(
            r#"
velocity_humanize = 0
patterns = [
    { density = 0, hits = [{step = 0, track = "Heel"}] },
    { density = 1, weight = 1, hits = [{step = 1, track = "Toe"}] },
    { density = 1, weight = 9, hits = [{step = 7, track = "Toe"}] },
    { density = 2, hits = [{step = 8, track = "Heel"}] },
]
"#,
        )
        .unwrap();
        assert!(table.has_alternatives());
        let tune = TunePlan::default();
        let mut heavy = 0;
        let mut distinct_parts = false;
        for seed in 0..256 {
            let plan = FeetPatternPlan::new(&tune, &table, seed);
            let repeated = FeetPatternPlan::new(&tune, &table, seed);
            let render = |plan: &FeetPatternPlan, part, density| {
                feet_notes(&table, plan.pattern(&table, part, density, false), 16, &[0; 16])
            };
            let first = render(&plan, 0, 1);
            let second = render(&plan, 1, 1);
            assert_eq!(first, render(&repeated, 0, 1));
            assert_eq!(second, render(&repeated, 1, 1));
            assert_eq!(render(&plan, 0, 0).0[0].start_step, 0);
            assert_eq!(render(&plan, 0, 2).0[0].start_step, 8);
            heavy += usize::from(first.1[0].start_step == 7);
            distinct_parts |= first != second;
        }
        assert!(distinct_parts);
        assert!((192..256).contains(&heavy), "poids 1:9, choix lourds : {heavy}");
    }

    #[test]
    fn large_weights_are_selected_without_u32_overflow() {
        let mut table = FeetTable::default();
        table.patterns.retain(|pattern| pattern.density != 1);
        for step in [1, 7] {
            let mut pattern = FeetPattern::new(1, vec![FootHit::toe(step)]);
            pattern.weight = u32::MAX;
            table.patterns.push(pattern);
        }
        table.validate().unwrap();
        let tune = TunePlan::default();
        let mut observed = [false; 2];
        for seed in 0..128 {
            let plan = FeetPatternPlan::new(&tune, &table, seed);
            let (_, notes) = feet_notes(&table, plan.pattern(&table, 0, 1, false), 16, &[0; 16]);
            observed[usize::from(notes[0].start_step == 7)] = true;
        }
        assert_eq!(observed, [true, true]);
    }

    #[test]
    fn zero_weights_and_duplicate_cadences_are_rejected() {
        let mut table = FeetTable::default();
        table.patterns[0].weight = 0;
        assert!(matches!(table.validate(), Err(FeetTableError::ZeroWeight { density: 0 })));
        table.patterns[0].weight = 1;
        let mut cadence = FeetPattern::new(1, vec![FootHit::toe(15)]);
        cadence.weight = 0;
        table.cadence_patterns.push(cadence);
        assert!(matches!(table.validate(), Err(FeetTableError::ZeroWeight { density: 1 })));
        table.cadence_patterns[0].weight = 1;
        table.cadence_patterns.push(table.cadence_patterns[0].clone());
        assert!(matches!(table.validate(), Err(FeetTableError::DuplicateCadenceDensity(1))));
    }

    #[test]
    fn cadences_override_selected_euclidean_alternatives_without_entering_variety() {
        let table = FeetTable::from_toml(
            r#"
patterns = [
    { density = 0, hits = [] },
    { density = 1, weight = 2, euclid = {pulses = 3, steps = 16, rotation = 1}, track = "Toe" },
    { density = 1, weight = 3, euclid = {pulses = 5, steps = 16, rotation = 2}, track = "Toe" },
    { density = 2, hits = [] },
]
cadence_patterns = [{density = 1, hits = [{step = 15, track = "Toe"}]}]
"#,
        )
        .unwrap();
        let restored = FeetTable::from_toml(&toml::to_string(&table).unwrap()).unwrap();
        let tune = TunePlan::default();
        for seed in 0..32 {
            let plan = FeetPatternPlan::new(&tune, &table, seed);
            let restored_plan = FeetPatternPlan::new(&tune, &restored, seed);
            let ordinary = feet_notes(&table, plan.pattern(&table, 0, 1, false), 16, &[0; 16]);
            assert!([3, 5].contains(&ordinary.1.len()));
            assert_eq!(
                ordinary,
                feet_notes(&restored, restored_plan.pattern(&restored, 0, 1, false), 16, &[0; 16],)
            );
            let cadence = feet_notes(&table, plan.pattern(&table, 0, 1, true), 16, &[0; 16]);
            assert_eq!(cadence.1.len(), 1);
            assert_eq!(cadence.1[0].start_step, 15);
            assert_eq!(
                feet_notes(&table, plan.pattern(&table, 0, 2, true), 16, &[0; 16]),
                feet_notes(&table, plan.pattern(&table, 0, 2, false), 16, &[0; 16]),
            );
        }
        let mut unique = FeetTable::default();
        unique.cadence_patterns = table.cadence_patterns;
        assert!(!unique.has_alternatives());
    }
}

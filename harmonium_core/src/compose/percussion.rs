//! Percussion à deux voix (grave, aiguë) et table de patterns portée en données (D9).
//!
//! La percussion fournit l'assise rythmique principale avec la mélodie ; ses sons
//! (notes, banque, vélocités) viennent du profil.
//!
//! Ce module implémente :
//! - [`PercussionHit`] : un coup de percussion unitaire `(step, piste PercussionLow|PercussionHigh, delta_velocity)` ;
//! - [`PercussionPattern`] : pattern de percussion pour une densité donnée ;
//! - [`PercussionTable`] : table complète portée en données (constitution V) via `percussion.toml` ;
//! - [`percussion_notes`] : fonction pure générant les [`TimelineNote`] de percussion grave et de percussion aiguë.

use std::{collections::HashSet, fmt};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{
    accomp_profile::require,
    arc::{TagWeight, tagged_weight},
    phrase::{
        Candidate, MAX_FILL_BARS, MAX_PHRASE_BARS, MAX_PHRASES, PhraseAt, Slot, Window, pick_fill,
        tile,
    },
    plan::TunePlan,
};
use crate::{
    algo::RngCore,
    style_profile::ProfileError,
    timeline::{Articulation, TimelineNote, TrackId},
};

/// Contenu brut de la table TOML embarquée de percussion (D9).
pub const DEFAULT_PERCUSSION_TOML: &str =
    include_str!("../../../profiles/reel_quebecois/percussion.toml");

/// Un coup de percussion individuel dans une mesure de percussion (D9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PercussionHit {
    /// Pas dans la mesure (`0..16`, soit `0..=15`).
    pub step: usize,
    /// Piste ciblée ([`TrackId::PercussionLow`] ou [`TrackId::PercussionHigh`]).
    pub track: TrackId,
    /// Ajustement de vélocité relatif à la vélocité de base du membre.
    pub delta_velocity: i8,
    /// Note MIDI explicite (optionnelle, surpasse le défaut de piste/voix).
    pub note: Option<u8>,
    /// Voix percussive ciblée (optionnelle).
    pub voice: Option<String>,
}

impl PercussionHit {
    /// Crée un coup de percussion avec pas, piste et delta de vélocité.
    #[must_use]
    pub const fn new(step: usize, track: TrackId, delta_velocity: i8) -> Self {
        Self { step, track, delta_velocity, note: None, voice: None }
    }

    /// Raccourci pour un coup de percussion grave (delta de vélocité nul).
    #[must_use]
    pub const fn low(step: usize) -> Self {
        Self { step, track: TrackId::PercussionLow, delta_velocity: 0, note: None, voice: None }
    }

    /// Raccourci pour un coup de percussion aiguë (delta de vélocité nul).
    #[must_use]
    pub const fn high(step: usize) -> Self {
        Self { step, track: TrackId::PercussionHigh, delta_velocity: 0, note: None, voice: None }
    }

    /// Raccourci pour un coup de percussion aiguë avec delta de vélocité explicite.
    #[must_use]
    pub const fn high_with_delta(step: usize, delta_velocity: i8) -> Self {
        Self { step, track: TrackId::PercussionHigh, delta_velocity, note: None, voice: None }
    }

    /// Coup avec note explicite.
    #[must_use]
    pub const fn with_note(step: usize, track: TrackId, note: u8, delta_velocity: i8) -> Self {
        Self { step, track, delta_velocity, note: Some(note), voice: None }
    }

    /// Coup avec voix percussive nommée.
    #[must_use]
    pub fn with_voice(step: usize, voice: impl Into<String>, delta_velocity: i8) -> Self {
        Self {
            step,
            track: TrackId::PercussionHigh,
            delta_velocity,
            note: None,
            voice: Some(voice.into()),
        }
    }
}

impl From<(usize, TrackId)> for PercussionHit {
    fn from((step, track): (usize, TrackId)) -> Self {
        Self { step, track, delta_velocity: 0, note: None, voice: None }
    }
}

impl From<(usize, TrackId, i8)> for PercussionHit {
    fn from((step, track, delta_velocity): (usize, TrackId, i8)) -> Self {
        Self { step, track, delta_velocity, note: None, voice: None }
    }
}

impl From<PercussionHit> for (usize, TrackId, i8) {
    fn from(hit: PercussionHit) -> Self {
        (hit.step, hit.track, hit.delta_velocity)
    }
}

fn deserialize_percussion_track<'de, D>(deserializer: D) -> Result<TrackId, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_ascii_lowercase().as_str() {
        "percussionlow" => Ok(TrackId::PercussionLow),
        "percussionhigh" => Ok(TrackId::PercussionHigh),
        _ => Err(serde::de::Error::custom(format!(
            "piste invalide pour la percussion : attendu 'PercussionLow' ou 'PercussionHigh', reçu '{s}'"
        ))),
    }
}

fn deserialize_optional_percussion_track<'de, D>(
    deserializer: D,
) -> Result<Option<TrackId>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    let Some(s) = opt else { return Ok(None) };
    match s.to_ascii_lowercase().as_str() {
        "percussionlow" => Ok(Some(TrackId::PercussionLow)),
        "percussionhigh" => Ok(Some(TrackId::PercussionHigh)),
        _ => Err(serde::de::Error::custom(format!(
            "piste invalide pour la percussion : attendu 'PercussionLow' ou 'PercussionHigh', reçu '{s}'"
        ))),
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawPercussionHit {
    Struct {
        step: usize,
        #[serde(default, deserialize_with = "deserialize_optional_percussion_track")]
        track: Option<TrackId>,
        #[serde(default)]
        voice: Option<String>,
        #[serde(default)]
        note: Option<u8>,
        #[serde(default)]
        delta_velocity: i8,
    },
    Tuple2(usize, #[serde(deserialize_with = "deserialize_percussion_track")] TrackId),
    Tuple3(usize, #[serde(deserialize_with = "deserialize_percussion_track")] TrackId, i8),
}

impl<'de> Deserialize<'de> for PercussionHit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawPercussionHit::deserialize(deserializer)?;
        match raw {
            RawPercussionHit::Struct { step, track, voice, note, delta_velocity } => {
                let track = track.unwrap_or(if voice.as_deref() == Some("bass_drum") {
                    TrackId::PercussionLow
                } else {
                    TrackId::PercussionHigh
                });
                Ok(Self { step, track, delta_velocity, note, voice })
            }
            RawPercussionHit::Tuple2(step, track) => {
                Ok(Self { step, track, delta_velocity: 0, note: None, voice: None })
            }
            RawPercussionHit::Tuple3(step, track, delta_velocity) => {
                Ok(Self { step, track, delta_velocity, note: None, voice: None })
            }
        }
    }
}

#[derive(Serialize)]
struct RawPercussionHitSerialize<'a> {
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

impl Serialize for PercussionHit {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let track_str = match self.track {
            TrackId::PercussionLow => "PercussionLow",
            TrackId::PercussionHigh => "PercussionHigh",
            _ => "Unknown",
        };
        let raw = RawPercussionHitSerialize {
            step: self.step,
            track: track_str,
            delta_velocity: self.delta_velocity,
            note: self.note,
            voice: self.voice.as_deref(),
        };
        raw.serialize(serializer)
    }
}

/// Pattern de coups de percussion pour un niveau de densité donné (D9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PercussionPattern {
    /// Niveau de densité (`0`, `1` ou `2`).
    pub density: u8,
    /// Poids strictement positif lors du choix parmi les alternatives de même densité.
    pub weight: u32,
    /// Portée optionnelle : le pattern ne concourt que pour cette partie (`form.parts[].name`).
    pub part: Option<String>,
    /// Portée optionnelle : le pattern ne concourt que pour ce tour de l'air (1-based).
    pub tour: Option<usize>,
    /// Étiquette que `[arc]` nomme pour choisir parmi les patterns (`arc.stages[].variants[].percussion`).
    pub tag: Option<String>,
    /// Liste des coups composant la mesure.
    pub hits: Vec<PercussionHit>,
    euclid: Option<(EuclideanPattern, TrackId)>,
}

impl PercussionPattern {
    /// Crée un nouveau [`PercussionPattern`].
    #[must_use]
    pub const fn new(density: u8, hits: Vec<PercussionHit>) -> Self {
        Self { density, weight: 1, part: None, tour: None, tag: None, hits, euclid: None }
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

impl<'de> Deserialize<'de> for PercussionPattern {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            density: u8,
            #[serde(default = "default_pattern_weight")]
            weight: u32,
            hits: Option<Vec<PercussionHit>>,
            euclid: Option<EuclideanPattern>,
            #[serde(default)]
            track: Option<String>,
            #[serde(default)]
            part: Option<String>,
            #[serde(default)]
            tour: Option<usize>,
            #[serde(default)]
            tag: Option<String>,
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
                let track = match raw.track.as_deref().unwrap_or("PercussionLow") {
                    "PercussionLow" | "percussionlow" => TrackId::PercussionLow,
                    "PercussionHigh" | "percussionhigh" => TrackId::PercussionHigh,
                    _ => {
                        return Err(serde::de::Error::custom(
                            "track : PercussionLow ou PercussionHigh requis",
                        ));
                    }
                };
                euclid = Some((e, track));
                let mut pattern =
                    crate::algo::euclidean::generate_euclidean_bools(e.steps, e.pulses);
                pattern.rotate_right(e.rotation.rem_euclid(e.steps as i32) as usize);
                pattern
                    .into_iter()
                    .enumerate()
                    .filter_map(|(step, hit)| hit.then_some(PercussionHit::new(step, track, 0)))
                    .collect()
            }
            (None, None) if raw.track.is_none() => Vec::new(),
            _ => return Err(serde::de::Error::custom("choisir hits ou euclid, pas les deux")),
        };
        Ok(Self {
            density: raw.density,
            weight: raw.weight,
            part: raw.part,
            tour: raw.tour,
            tag: raw.tag,
            hits,
            euclid,
        })
    }
}

impl Serialize for PercussionPattern {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct(
            "PercussionPattern",
            3 + usize::from(self.euclid.is_some())
                + usize::from(self.part.is_some())
                + usize::from(self.tour.is_some())
                + usize::from(self.tag.is_some()),
        )?;
        state.serialize_field("density", &self.density)?;
        state.serialize_field("weight", &self.weight)?;
        if let Some(part) = &self.part {
            state.serialize_field("part", part)?;
        }
        if let Some(tour) = self.tour {
            state.serialize_field("tour", &tour)?;
        }
        if let Some(tag) = &self.tag {
            state.serialize_field("tag", tag)?;
        }
        if let Some((pattern, track)) = self.euclid {
            state.serialize_field("euclid", &pattern)?;
            state.serialize_field(
                "track",
                if track == TrackId::PercussionHigh { "PercussionHigh" } else { "PercussionLow" },
            )?;
        } else {
            state.serialize_field("hits", &self.hits)?;
        }
        state.end()
    }
}

const fn is_any(at: &PhraseAt) -> bool {
    matches!(at, PhraseAt::Any)
}

/// Phrase de batterie : une suite de patterns de même densité jouée sur plusieurs mesures
/// (`[[phrases]]`). Le découpage de chaque passe de partie en phrases est tiré par poids, sans
/// état d'une mesure à l'autre (voir `phrase.rs`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PercussionPhrase {
    /// Niveau de densité (`0`, `1` ou `2`) : la phrase ne concourt qu'à cette densité.
    pub density: u8,
    /// Poids strictement positif du tirage.
    #[serde(default = "default_pattern_weight")]
    pub weight: u32,
    /// Portée optionnelle : la phrase ne concourt que pour cette partie (`form.parts[].name`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    /// Étiquette que `[arc]` nomme pour choisir parmi les phrases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Emplacement dans la passe de la partie.
    #[serde(default, skip_serializing_if = "is_any")]
    pub at: PhraseAt,
    /// Un pattern par mesure : indices dans `[[patterns]]`, de la densité de la phrase, sans
    /// portée `part` ni `tour`. 1 à 8 mesures.
    pub patterns: Vec<usize>,
}

/// Mesure d'une relance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FillBar {
    #[serde(default)]
    pub hits: Vec<PercussionHit>,
}

/// Relance (fill) de fin de phrase (`[[fills]]`) : de 1 à 4 mesures qui s'achèvent avec la
/// phrase. Dans la première mesure de la relance, le pattern ordinaire joue jusqu'au pas
/// `start_step` (p. ex. 12 : levée au temps 4), la relance reprend à partir de là ; les mesures
/// suivantes sont celles de la relance seule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PercussionFill {
    /// Niveau de densité (`0`, `1` ou `2`) : la relance ne concourt qu'à cette densité.
    pub density: u8,
    /// Poids strictement positif du tirage.
    #[serde(default = "default_pattern_weight")]
    pub weight: u32,
    /// Étiquette que `[arc]` nomme pour choisir parmi les relances.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Fins de phrase concernées : toutes (`any`), celles qui ferment la passe d'une partie
    /// (`part_end` : mise en place avant un changement de section) ou celle du chorus
    /// (`chorus_end`).
    #[serde(default, skip_serializing_if = "is_any")]
    pub at: PhraseAt,
    /// Pas de la première mesure où la relance prend la place du pattern ordinaire.
    #[serde(default)]
    pub start_step: usize,
    /// Les mesures de la relance, 1 à 4 ; les coups de la première sont à `start_step` ou après.
    pub bars: Vec<FillBar>,
}

const fn is_zero(value: &u32) -> bool {
    *value == 0
}

const fn default_low_note() -> u8 {
    36
}

const fn default_high_note() -> u8 {
    37
}

const fn default_velocity_humanize() -> u8 {
    12
}

const fn default_low_velocity() -> u8 {
    55
}

const fn default_high_velocity() -> u8 {
    45
}

const fn default_percussion_gain() -> f32 {
    1.0
}

/// Voix percussive pour la batterie / percussion militaire ou percussion ornée.
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
struct RawPercussionTable {
    #[serde(default = "default_low_note")]
    low_note: u8,
    #[serde(default = "default_high_note")]
    high_note: u8,
    #[serde(default = "default_low_velocity")]
    low_velocity: u8,
    #[serde(default = "default_high_velocity")]
    high_velocity: u8,
    #[serde(default = "default_percussion_gain")]
    percussion_gain: f32,
    #[serde(default = "default_velocity_humanize")]
    velocity_humanize: u8,
    #[serde(default)]
    voices: Vec<PercussionVoice>,
    #[serde(default)]
    intro_tracks: Option<Vec<TrackId>>,
    #[serde(default)]
    patterns: Vec<PercussionPattern>,
    #[serde(default)]
    densities: Vec<PercussionPattern>,
    #[serde(default)]
    cadence_patterns: Vec<PercussionPattern>,
    #[serde(default)]
    phrases: Vec<PercussionPhrase>,
    #[serde(default)]
    fills: Vec<PercussionFill>,
    #[serde(default)]
    fills_none_weight: u32,
}

/// Table de percussion portée en données (D9, constitution V).
///
/// Contient les notes GM et les vélocités de base de la percussion grave et de la percussion aiguë,
/// le gain global (`percussion_gain`, `0..=1`) ainsi que les patterns associés à
/// chaque densité (`0..=2`).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PercussionTable {
    /// Note GM de la percussion grave (canal 9 percussion, D10).
    #[serde(default = "default_low_note")]
    pub low_note: u8,
    /// Note GM de la percussion aiguë (canal 9 percussion, D10).
    #[serde(default = "default_high_note")]
    pub high_note: u8,
    /// Vélocité de base de la percussion grave (MIDI `0..=127`).
    #[serde(default = "default_low_velocity")]
    pub low_velocity: u8,
    /// Vélocité de base de la percussion aiguë (MIDI `0..=127`).
    #[serde(default = "default_high_velocity")]
    pub high_velocity: u8,
    /// Gain global appliqué aux vélocités effectives (`0..=1`).
    #[serde(default = "default_percussion_gain")]
    pub percussion_gain: f32,
    /// Amplitude d'humanisation des vélocités (`0..=40`, défaut 12) : chaque
    /// coup reçoit un décalage déterministe uniforme dans `±velocity_humanize`.
    #[serde(default = "default_velocity_humanize")]
    pub velocity_humanize: u8,
    /// Voix percussives nommées (pour kit de batterie / tambour militaire).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub voices: Vec<PercussionVoice>,
    /// Pistes actives lors du décompte d'intro percussion (défaut : [PercussionLow]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intro_tracks: Option<Vec<TrackId>>,
    /// Patterns de coups par niveau de densité.
    #[serde(default)]
    pub patterns: Vec<PercussionPattern>,
    /// Patterns de coups optionnels pour les cadences / fins de phrase.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cadence_patterns: Vec<PercussionPattern>,
    /// Phrases de plusieurs mesures (suites de patterns de même densité).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phrases: Vec<PercussionPhrase>,
    /// Banque pondérée de relances de fin de phrase ; remplace `cadence_patterns` aux densités
    /// qu'elle déclare.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fills: Vec<PercussionFill>,
    /// Poids de « pas de relance » dans le tirage d'une fin de phrase.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fills_none_weight: u32,
}

impl<'de> Deserialize<'de> for PercussionTable {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawPercussionTable::deserialize(deserializer)?;
        let patterns = if !raw.patterns.is_empty() { raw.patterns } else { raw.densities };
        Ok(Self {
            low_note: raw.low_note,
            high_note: raw.high_note,
            low_velocity: raw.low_velocity,
            high_velocity: raw.high_velocity,
            percussion_gain: raw.percussion_gain,
            velocity_humanize: raw.velocity_humanize,
            voices: raw.voices,
            intro_tracks: raw.intro_tracks,
            patterns,
            cadence_patterns: raw.cadence_patterns,
            phrases: raw.phrases,
            fills: raw.fills,
            fills_none_weight: raw.fills_none_weight,
        })
    }
}

/// Erreur survenue lors du chargement ou de la validation d'une [`PercussionTable`].
#[derive(Clone, Debug, PartialEq)]
pub enum PercussionTableError {
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
    /// La portée `tour` d'un pattern vaut 0 (les tours se comptent depuis 1).
    ZeroTour { density: u8 },
    /// Un pattern de cadence porte une portée (`part` ou `tour`), réservée aux patterns ordinaires.
    ScopedCadence { density: u8 },
    /// Une densité n'a aucun pattern sans portée : aucun repli pour les parties et tours non nommés.
    MissingUnscoped(u8),
    /// Piste invalide (seules `TrackId::PercussionLow` et `TrackId::PercussionHigh` sont permises).
    InvalidTrack { density: u8, step: usize, track: TrackId },
    /// La vélocité effective (`base + delta`) dépasse la plage MIDI `0..=127`.
    VelocityOutOfRange { density: u8, step: usize, velocity: i16 },
    /// Une vélocité de base est hors de la plage MIDI `0..=127`.
    BaseVelocityOutOfRange { percussion_low: u8, percussion_high: u8 },
    /// Le gain global `percussion_gain` est hors de la plage `0..=1`.
    GainOutOfRange { gain: f32 },
    /// L'humanisation `velocity_humanize` est hors de la plage `0..=40`.
    VelocityHumanizeOutOfRange { humanize: u8 },
    /// Voix percussive inconnue référencée par un coup.
    UnknownVoice { density: u8, step: usize, voice: String },
    /// Note MIDI hors limites (0..=127).
    NoteOutOfRange { density: u8, step: usize, note: u8 },
}

impl fmt::Display for PercussionTableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TomlParse(msg) => {
                write!(f, "erreur de lecture TOML de la table de percussion : {msg}")
            }
            Self::EmptyTable => write!(f, "la table de percussion est vide"),
            Self::MissingDensity(density) => {
                write!(f, "densité requise manquante dans la table de percussion : {density}")
            }
            Self::DuplicateCadenceDensity(density) => {
                write!(f, "densité dupliquée dans les patterns de cadence : {density}")
            }
            Self::ZeroWeight { density } => {
                write!(f, "poids nul pour un pattern de percussion à la densité {density}")
            }
            Self::ZeroTour { density } => {
                write!(
                    f,
                    "tour 0 pour un pattern de percussion à la densité {density} (tours 1-based)"
                )
            }
            Self::ScopedCadence { density } => {
                write!(
                    f,
                    "portée (part, tour) refusée pour le pattern de cadence de la densité {density}"
                )
            }
            Self::MissingUnscoped(density) => {
                write!(
                    f,
                    "aucun pattern sans portée (part, tour) à la densité {density} : un repli est requis"
                )
            }
            Self::InvalidTrack { density, step, track } => {
                write!(
                    f,
                    "piste {track:?} invalide pour la percussion à la densité {density}, pas {step}"
                )
            }
            Self::VelocityOutOfRange { density, step, velocity } => {
                write!(
                    f,
                    "vélocité effective {velocity} hors limites MIDI (0..=127) à la densité {density}, pas {step}"
                )
            }
            Self::BaseVelocityOutOfRange { percussion_low, percussion_high } => {
                write!(
                    f,
                    "vélocité de base hors limites MIDI (0..=127) : percussion grave={percussion_low}, percussion aiguë={percussion_high}"
                )
            }
            Self::GainOutOfRange { gain } => {
                write!(f, "gain global hors limites (0..=1) : percussion_gain={gain}")
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

impl std::error::Error for PercussionTableError {}

impl Default for PercussionTable {
    /// Charge la table embarquée `percussion.toml` sans jamais paniquer à
    /// l'exécution (en cas d'erreur de données, retourne une table par défaut
    /// vide ; la validité de `DEFAULT_PERCUSSION_TOML` est garantie par les tests unitaires
    /// et [`crate::tuning::TuningParams::validate`]).
    fn default() -> Self {
        Self::from_toml(DEFAULT_PERCUSSION_TOML).unwrap_or_else(|_| Self {
            low_note: default_low_note(),
            high_note: default_high_note(),
            low_velocity: default_low_velocity(),
            high_velocity: default_high_velocity(),
            percussion_gain: default_percussion_gain(),
            velocity_humanize: default_velocity_humanize(),
            voices: Vec::new(),
            intro_tracks: None,
            patterns: Vec::new(),
            cadence_patterns: Vec::new(),
            phrases: Vec::new(),
            fills: Vec::new(),
            fills_none_weight: 0,
        })
    }
}

impl PercussionTable {
    /// Charge et valide une [`PercussionTable`] depuis une chaîne TOML au format `percussion.toml`.
    pub fn from_toml(toml_str: &str) -> Result<Self, PercussionTableError> {
        let table: Self =
            toml::from_str(toml_str).map_err(|e| PercussionTableError::TomlParse(e.to_string()))?;
        table.validate()?;
        Ok(table)
    }

    /// Valide les contraintes d'intégrité de la table :
    /// - vélocités de base dans `0..=127` ;
    /// - gain global dans `0..=1` ;
    /// - densités `0`, `1` et `2` présentes, avec un seul pattern de cadence par densité ;
    /// - poids des patterns strictement positifs ;
    /// - chaque piste est `PercussionLow` ou `PercussionHigh` ;
    /// - chaque vélocité calculée (`base + delta`) est dans `0..=127`.
    pub fn validate(&self) -> Result<(), PercussionTableError> {
        if self.low_velocity > 127 || self.high_velocity > 127 {
            return Err(PercussionTableError::BaseVelocityOutOfRange {
                percussion_low: self.low_velocity,
                percussion_high: self.high_velocity,
            });
        }
        if !(0.0..=1.0).contains(&self.percussion_gain) {
            return Err(PercussionTableError::GainOutOfRange { gain: self.percussion_gain });
        }
        if self.velocity_humanize > 40 {
            return Err(PercussionTableError::VelocityHumanizeOutOfRange {
                humanize: self.velocity_humanize,
            });
        }

        if self.patterns.is_empty() {
            return Err(PercussionTableError::EmptyTable);
        }

        for voice in &self.voices {
            if voice.note > 127 || voice.velocity > 127 || voice.name.is_empty() {
                return Err(PercussionTableError::BaseVelocityOutOfRange {
                    percussion_low: self.low_velocity,
                    percussion_high: self.high_velocity,
                });
            }
        }

        let validate_hit = |density: u8, hit: &PercussionHit| -> Result<(), PercussionTableError> {
            let (base_velocity, pitch) = if let Some(voice_name) = &hit.voice {
                let Some(v) = self.voices.iter().find(|v| v.name == *voice_name) else {
                    return Err(PercussionTableError::UnknownVoice {
                        density,
                        step: hit.step,
                        voice: voice_name.clone(),
                    });
                };
                (v.velocity, hit.note.unwrap_or(v.note))
            } else {
                let base = match hit.track {
                    TrackId::PercussionLow => self.low_velocity,
                    TrackId::PercussionHigh => self.high_velocity,
                    other => {
                        return Err(PercussionTableError::InvalidTrack {
                            density,
                            step: hit.step,
                            track: other,
                        });
                    }
                };
                let p = hit.note.unwrap_or(match hit.track {
                    TrackId::PercussionLow => self.low_note,
                    TrackId::PercussionHigh => self.high_note,
                    _ => 0,
                });
                (base, p)
            };

            if pitch > 127 {
                return Err(PercussionTableError::NoteOutOfRange {
                    density,
                    step: hit.step,
                    note: pitch,
                });
            }

            let effective_velocity = i16::from(base_velocity) + i16::from(hit.delta_velocity);
            if !(0..=127).contains(&effective_velocity) {
                return Err(PercussionTableError::VelocityOutOfRange {
                    density,
                    step: hit.step,
                    velocity: effective_velocity,
                });
            }
            Ok(())
        };

        let mut seen_densities = HashSet::new();
        let mut seen_unscoped = HashSet::new();

        for pattern in &self.patterns {
            seen_densities.insert(pattern.density);
            if pattern.part.is_none() && pattern.tour.is_none() {
                seen_unscoped.insert(pattern.density);
            }
            if pattern.weight == 0 {
                return Err(PercussionTableError::ZeroWeight { density: pattern.density });
            }
            if pattern.tour == Some(0) {
                return Err(PercussionTableError::ZeroTour { density: pattern.density });
            }

            for hit in &pattern.hits {
                validate_hit(pattern.density, hit)?;
            }
        }

        let mut seen_cadence = HashSet::new();
        for pattern in &self.cadence_patterns {
            if !seen_cadence.insert(pattern.density) {
                return Err(PercussionTableError::DuplicateCadenceDensity(pattern.density));
            }
            if pattern.weight == 0 {
                return Err(PercussionTableError::ZeroWeight { density: pattern.density });
            }
            if pattern.part.is_some() || pattern.tour.is_some() {
                return Err(PercussionTableError::ScopedCadence { density: pattern.density });
            }
            for hit in &pattern.hits {
                validate_hit(pattern.density, hit)?;
            }
        }

        for fill in &self.fills {
            for hit in fill.bars.iter().flat_map(|bar| &bar.hits) {
                validate_hit(fill.density, hit)?;
            }
        }

        for required in 0..=2u8 {
            if !seen_densities.contains(&required) {
                return Err(PercussionTableError::MissingDensity(required));
            }
            if !seen_unscoped.contains(&required) {
                return Err(PercussionTableError::MissingUnscoped(required));
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

    /// Vrai si une mesure de relance remplace le pattern ordinaire à la densité `density`.
    pub(crate) fn has_cadence_pattern(&self, density: u8) -> bool {
        self.cadence_patterns.iter().any(|pattern| pattern.density == density.min(2))
    }

    /// Vrai si une banque de relances est déclarée à la densité `density` : elle remplace alors
    /// le pattern de cadence de cette densité.
    pub(crate) fn has_fills(&self, density: u8) -> bool {
        self.fills.iter().any(|fill| fill.density == density.min(2))
    }

    /// Vrai si la table a des phrases ou des relances : leurs tirages dérivent de la graine
    /// d'accompagnement de l'air, qui est donc tirée.
    pub(crate) fn is_phrased(&self) -> bool {
        !self.phrases.is_empty() || !self.fills.is_empty()
    }

    /// Découpage de la passe `window` de la partie `part` en phrases de batterie de la densité
    /// `density`, restreint aux étiquettes `tags` de l'arc (vide : toutes). Vide : aucune phrase
    /// déclarée à cette densité.
    pub(crate) fn phrase_tiling(
        &self,
        window: Window,
        part: &str,
        density: u8,
        tags: &[TagWeight],
    ) -> Vec<Option<Slot>> {
        let density = density.min(2);
        if !self.phrases.iter().any(|phrase| phrase.density == density) {
            return Vec::new();
        }
        tile(window, self.phrases.len(), |index| {
            let phrase = &self.phrases[index];
            (phrase.density == density && phrase.part.as_deref().is_none_or(|p| p == part)).then(
                || Candidate {
                    bars: phrase.patterns.len(),
                    weight: tagged_weight(phrase.weight, phrase.tag.as_deref(), tags),
                    at: phrase.at,
                },
            )
        })
    }

    /// Pattern de la mesure couverte par `slot` (phrase de la table).
    pub(crate) fn phrase_pattern(&self, slot: Slot) -> &PercussionPattern {
        &self.patterns[self.phrases[slot.index].patterns[slot.bar]]
    }

    /// Relance qui couvre la mesure `offset` de la passe `window`, avec le rang de la mesure
    /// dans la relance. Une relance se tire à la fin de chaque phrase : fin de la phrase de
    /// `tiling` qui couvre la mesure, sinon `form_phrase_len(mesure)` (longueur de la phrase de
    /// la forme qui s'achève à cette mesure, `None` : pas de fin de phrase). Une relance ne
    /// déborde jamais de la phrase qu'elle clôt. Seules les relances de la densité `density`
    /// et des étiquettes `tags` de l'arc (vide : toutes) concourent.
    pub(crate) fn fill_covering(
        &self,
        window: Window,
        offset: usize,
        density: u8,
        tags: &[TagWeight],
        tiling: &[Option<Slot>],
        form_phrase_len: impl Fn(usize) -> Option<usize>,
    ) -> Option<(&PercussionFill, usize)> {
        let density = density.min(2);
        for ahead in 0..MAX_FILL_BARS {
            let end = offset + ahead;
            if end >= window.bars {
                break;
            }
            let available = match tiling.get(end).copied().flatten() {
                Some(slot) => slot.is_last().then_some(slot.bars),
                None => form_phrase_len(end),
            };
            let Some(available) = available else { continue };
            let picked = pick_fill(
                window,
                end,
                available,
                self.fills.len(),
                self.fills_none_weight,
                |index| {
                    let fill = &self.fills[index];
                    (fill.density == density).then(|| Candidate {
                        bars: fill.bars.len(),
                        weight: tagged_weight(fill.weight, fill.tag.as_deref(), tags),
                        at: fill.at,
                    })
                },
            );
            if let Some(index) = picked {
                let fill = &self.fills[index];
                if fill.bars.len() > ahead {
                    return Some((fill, fill.bars.len() - 1 - ahead));
                }
            }
        }
        None
    }

    /// Valide `[[phrases]]` et `[[fills]]` contre le profil (`steps` : pas par mesure, `parts` :
    /// noms de `form.parts`) ; les erreurs portent le chemin du champ.
    pub(crate) fn validate_phrasing(
        &self,
        steps: usize,
        parts: &[&str],
    ) -> Result<(), ProfileError> {
        require(self.phrases.len() <= MAX_PHRASES, "percussion.phrases", "au plus 256 phrases")?;
        require(self.fills.len() <= MAX_PHRASES, "percussion.fills", "au plus 256 relances")?;
        for (i, phrase) in self.phrases.iter().enumerate() {
            let field = format!("percussion.phrases[{i}]");
            require(phrase.density <= 2, format!("{field}.density"), "densité 0, 1 ou 2 requise")?;
            require(phrase.weight > 0, format!("{field}.weight"), "poids positif requis")?;
            require(
                (1..=MAX_PHRASE_BARS).contains(&phrase.patterns.len()),
                format!("{field}.patterns"),
                "1 à 8 mesures requises",
            )?;
            require(
                phrase.part.as_deref().is_none_or(|name| parts.contains(&name)),
                format!("{field}.part"),
                "partie inconnue dans form.parts",
            )?;
            for (j, &index) in phrase.patterns.iter().enumerate() {
                let field = format!("{field}.patterns[{j}]");
                let Some(pattern) = self.patterns.get(index) else {
                    return Err(ProfileError::new(
                        field,
                        "pattern inconnu dans percussion.patterns",
                    ));
                };
                require(
                    pattern.density == phrase.density,
                    &field,
                    "le pattern doit avoir la densité de la phrase",
                )?;
                require(
                    pattern.part.is_none() && pattern.tour.is_none(),
                    &field,
                    "un pattern à portée (part, tour) est refusé dans une phrase",
                )?;
            }
        }
        for (i, fill) in self.fills.iter().enumerate() {
            let field = format!("percussion.fills[{i}]");
            require(fill.density <= 2, format!("{field}.density"), "densité 0, 1 ou 2 requise")?;
            require(
                !self.has_cadence_pattern(fill.density),
                format!("{field}.density"),
                "cadence_patterns déclare déjà cette densité : les relances la remplacent, \
                 en garder une seule des deux",
            )?;
            require(fill.weight > 0, format!("{field}.weight"), "poids positif requis")?;
            require(
                fill.at != PhraseAt::PartStart,
                format!("{field}.at"),
                "any, part_end ou chorus_end requis pour une relance",
            )?;
            require(
                (1..=MAX_FILL_BARS).contains(&fill.bars.len()),
                format!("{field}.bars"),
                "1 à 4 mesures requises",
            )?;
            require(
                fill.start_step < steps,
                format!("{field}.start_step"),
                "pas dans la mesure requis",
            )?;
            for (b, bar) in fill.bars.iter().enumerate() {
                for (h, hit) in bar.hits.iter().enumerate() {
                    let field = format!("{field}.bars[{b}].hits[{h}].step");
                    require(hit.step < steps, &field, "pas dans la mesure requis")?;
                    require(
                        b > 0 || hit.step >= fill.start_step,
                        &field,
                        "les coups de la première mesure commencent à start_step",
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Pattern de la densité `density` tiré parmi ceux qui portent un tag de `tags` (poids du
    /// pattern fois celui du tag) et dont la portée `part` convient, avec le flux `seed`.
    /// `None` : aucun pattern ne concourt, le choix ordinaire s'applique.
    pub(crate) fn tagged_pattern(
        &self,
        part: &str,
        density: u8,
        tags: &[super::arc::TagWeight],
        seed: u64,
    ) -> Option<&PercussionPattern> {
        let density = density.min(2);
        let weight = |pattern: &PercussionPattern| {
            if pattern.density == density && pattern.part.as_deref().is_none_or(|p| p == part) {
                u64::from(pattern.weight)
                    .saturating_mul(super::arc::tag_weight(tags, pattern.tag.as_deref()))
            } else {
                0
            }
        };
        let total = self.patterns.iter().map(weight).fold(0, u64::saturating_add);
        if total == 0 {
            return None;
        }
        let mut pick = ChaCha8Rng::seed_from_u64(seed).next_range_u64(0..total);
        self.patterns.iter().find(|pattern| {
            let weight = weight(pattern);
            let hit = pick < weight;
            pick = pick.saturating_sub(weight);
            hit
        })
    }
}

/// Choix pondéré de percussion, stable pour chaque partie et densité d'un air.
///
/// Les reprises et les tours relisent le même choix. Les cadences, uniques par
/// densité, remplacent ce choix au rendu sans participer au tirage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PercussionPatternPlan {
    /// `[partie][emplacement de tour][densité]` : indice dans `table.patterns`.
    parts: Vec<Vec<[usize; 3]>>,
    /// Plus grand tour nommé par un pattern : les tours `1..=max_tour` ont leur emplacement,
    /// les autres (intro comprise) partagent l'emplacement 0.
    max_tour: usize,
}

impl PercussionPatternPlan {
    /// Construit le plan depuis une table validée, sans tirage pour un candidat unique.
    ///
    /// Pour une partie, un tour et une densité, les candidats sont les patterns de la densité
    /// dont la portée convient (`part` égale à la partie, `tour` égal au tour) ; seuls les plus
    /// spécifiques (les deux portées, puis une seule, puis aucune) concourent, par poids. Sans
    /// aucune portée dans la table, le choix est celui de toujours : un tirage par partie et
    /// par densité.
    #[must_use]
    pub fn new(plan: &TunePlan, table: &PercussionTable, seed: u64) -> Self {
        let seed = crate::ornaments::mix(seed, 0x4645_4554);
        let max_tour = table.patterns.iter().filter_map(|pattern| pattern.tour).max().unwrap_or(0);
        let parts = plan
            .profile
            .form
            .parts
            .iter()
            .enumerate()
            .map(|(part, configured)| {
                (0..=max_tour)
                    .map(|slot| {
                        std::array::from_fn(|density| {
                            let eligible = |pattern: &PercussionPattern| {
                                usize::from(pattern.density) == density
                                    && pattern.part.as_deref().is_none_or(|p| p == configured.name)
                                    && pattern.tour.is_none_or(|tour| tour == slot)
                            };
                            let specificity = |pattern: &PercussionPattern| {
                                usize::from(pattern.part.is_some())
                                    + usize::from(pattern.tour.is_some())
                            };
                            let best = table
                                .patterns
                                .iter()
                                .filter(|pattern| eligible(pattern))
                                .map(specificity)
                                .max()
                                .unwrap_or_else(|| unreachable!("pattern de repli requis"));
                            let mut candidates =
                                table.patterns.iter().enumerate().filter(|(_, pattern)| {
                                    eligible(pattern) && specificity(pattern) == best
                                });
                            let Some((first, pattern)) = candidates.next() else {
                                unreachable!("densité requise dans une table validée");
                            };
                            let Some((_, alternative)) = candidates.next() else {
                                return first;
                            };
                            let total = u64::from(pattern.weight)
                                + u64::from(alternative.weight)
                                + candidates.map(|(_, p)| u64::from(p.weight)).sum::<u64>();
                            let stream = crate::ornaments::mix(
                                crate::ornaments::mix(seed, part as u64),
                                density as u64,
                            );
                            let mut rng = ChaCha8Rng::seed_from_u64(if slot == 0 {
                                stream
                            } else {
                                crate::ornaments::mix(stream, slot as u64)
                            });
                            let mut draw = rng.next_range_u64(0..total);
                            for (index, candidate) in table
                                .patterns
                                .iter()
                                .enumerate()
                                .filter(|(_, p)| eligible(p) && specificity(p) == best)
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
                    .collect()
            })
            .collect();
        Self { parts, max_tour }
    }

    /// Retourne le pattern choisi, ou son remplacement cadentiel s'il existe. `tour` : tour
    /// 1-based de l'air, 0 pour l'intro.
    #[must_use]
    pub fn pattern<'a>(
        &self,
        table: &'a PercussionTable,
        part: usize,
        tour: usize,
        density: u8,
        is_cadence: bool,
    ) -> &'a PercussionPattern {
        let density = density.min(2);
        if is_cadence
            && let Some(pattern) = table.cadence_patterns.iter().find(|p| p.density == density)
        {
            return pattern;
        }
        let slot = if (1..=self.max_tour).contains(&tour) { tour } else { 0 };
        &table.patterns[self.parts[part][slot][usize::from(density)]]
    }
}

/// Tire les décalages d'humanisation : une ligne par mesure distincte du
/// profil, une colonne par pas. Chaque
/// décalage est tiré uniformément dans `±humanize` via le RNG injecté
/// (déterministe : même graine ⇒ mêmes décalages). `humanize == 0` ne
/// consomme aucun tirage et produit des décalages tous nuls.
#[must_use]
pub fn draw_percussion_humanize(
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

/// Génère les notes de percussion (percussion grave, percussion aiguë) pour une mesure (D9).
///
/// # Paramètres
/// - `table` : table de percussion ([`PercussionTable`]) portant les vélocités de base.
/// - `pattern` : pattern choisi par [`PercussionPatternPlan::pattern`], cadence comprise.
/// - `steps` : nombre de pas du profil ; `humanize` contient exactement ce nombre de valeurs.
///
/// # Retour
/// Tuple `(grave_notes, aigu_notes)` :
/// - premier élément : notes de la percussion grave ([`TrackId::PercussionLow`]), `pitch = table.low_note` ;
/// - second élément : notes de la percussion aiguë ([`TrackId::PercussionHigh`]), `pitch = table.high_note`.
///
/// Toutes les notes produites ont `duration_steps = 0` et `articulation = Articulation::Trigger`.
/// La vélocité est
/// `((base_velocity + delta_velocity) * percussion_gain).round() + humanize[step]`,
/// bornée à `1..=127`.
#[must_use]
pub fn percussion_notes(
    table: &PercussionTable,
    pattern: &PercussionPattern,
    steps: usize,
    humanize: &[i8],
) -> (Vec<TimelineNote>, Vec<TimelineNote>) {
    debug_assert_eq!(humanize.len(), steps);

    let hits = &pattern.hits;

    let mut percussion_low_notes = Vec::new();
    let mut percussion_high_notes = Vec::new();

    for hit in hits {
        let (track, base_velocity, pitch) = if let Some(voice_name) = &hit.voice {
            if let Some(v) = table.voices.iter().find(|v| v.name == *voice_name) {
                (v.track.unwrap_or(hit.track), v.velocity, hit.note.unwrap_or(v.note))
            } else {
                (hit.track, table.high_velocity, hit.note.unwrap_or(table.high_note))
            }
        } else {
            let base = match hit.track {
                TrackId::PercussionLow => table.low_velocity,
                TrackId::PercussionHigh => table.high_velocity,
                _ => continue,
            };
            let p = hit.note.unwrap_or(match hit.track {
                TrackId::PercussionLow => table.low_note,
                TrackId::PercussionHigh => table.high_note,
                _ => continue,
            });
            (hit.track, base, p)
        };

        let target_vec = match track {
            TrackId::PercussionLow => &mut percussion_low_notes,
            TrackId::PercussionHigh => &mut percussion_high_notes,
            _ => continue,
        };

        let effective = f32::from(base_velocity) + f32::from(hit.delta_velocity);
        let base = (effective * table.percussion_gain).round() as i32;
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

    percussion_low_notes.sort_by_key(|n| n.start_step);
    percussion_high_notes.sort_by_key(|n| n.start_step);

    (percussion_low_notes, percussion_high_notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_density(
        table: &PercussionTable,
        density: u8,
        humanize: &[i8],
    ) -> (Vec<TimelineNote>, Vec<TimelineNote>) {
        let plan = PercussionPatternPlan::new(&TunePlan::default(), table, 0);
        percussion_notes(table, plan.pattern(table, 0, 1, density, false), humanize.len(), humanize)
    }

    #[test]
    fn test_embedded_table_validates() {
        let table = PercussionTable::default();
        table.validate().expect("DEFAULT_PERCUSSION_TOML doit être valide");
        assert_eq!(table.low_note, 36);
        assert_eq!(table.high_note, 37);
        assert_eq!(table.low_velocity, 55);
        assert_eq!(table.high_velocity, 45);
        assert_eq!(table.percussion_gain, 0.9);
        assert_eq!(table.velocity_humanize, 12);
    }

    #[test]
    fn test_exact_d9_positions_density_0() {
        let table = PercussionTable { percussion_gain: 1.0, ..PercussionTable::default() };
        let (percussion_low, percussion_high) = render_density(&table, 0, &[0; 16]);

        let percussion_low_steps: Vec<usize> =
            percussion_low.iter().map(|n| n.start_step).collect();
        assert_eq!(percussion_low_steps, vec![0, 8]);
        assert!(percussion_high.is_empty());

        for note in &percussion_low {
            assert_eq!(note.pitch, 36);
            assert_eq!(note.velocity, 55);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }
    }

    #[test]
    fn test_exact_d9_positions_density_1() {
        let table = PercussionTable { percussion_gain: 1.0, ..PercussionTable::default() };
        let (percussion_low, percussion_high) = render_density(&table, 1, &[0; 16]);

        let percussion_low_steps: Vec<usize> =
            percussion_low.iter().map(|n| n.start_step).collect();
        let percussion_high_steps: Vec<usize> =
            percussion_high.iter().map(|n| n.start_step).collect();

        assert_eq!(percussion_low_steps, vec![0, 4, 8, 12]);
        assert_eq!(percussion_high_steps, vec![2, 6, 10, 14]);

        for note in &percussion_low {
            assert_eq!(note.pitch, 36);
            assert_eq!(note.velocity, 55);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }

        for note in &percussion_high {
            assert_eq!(note.pitch, 37);
            assert_eq!(note.velocity, 45);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }
    }

    #[test]
    fn test_exact_d9_positions_density_2() {
        let table = PercussionTable { percussion_gain: 1.0, ..PercussionTable::default() };
        let (percussion_low, percussion_high) = render_density(&table, 2, &[0; 16]);

        let percussion_low_steps: Vec<usize> =
            percussion_low.iter().map(|n| n.start_step).collect();
        let percussion_high_steps: Vec<usize> =
            percussion_high.iter().map(|n| n.start_step).collect();

        assert_eq!(percussion_low_steps, vec![0, 4, 8, 12]);
        assert_eq!(percussion_high_steps, vec![2, 3, 6, 7, 10, 11, 14, 15]);

        for note in &percussion_low {
            assert_eq!(note.pitch, 36);
            assert_eq!(note.velocity, 55);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }

        for note in &percussion_high {
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
        let table = PercussionTable::default();
        let expected_2 = render_density(&table, 2, &[0; 16]);

        for d in [3, 4, 10, 99, 255] {
            let notes = render_density(&table, d, &[0; 16]);
            assert_eq!(notes.0.len(), expected_2.0.len(), "densité {d} percussion grave");
            assert_eq!(notes.1.len(), expected_2.1.len(), "densité {d} percussion aiguë");
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
low_note = 35
high_note = 37

[[patterns]]
density = 0
hits = [{ step = 0, track = "PercussionLow" }]

[[patterns]]
density = 1
hits = [{ step = 0, track = "PercussionLow" }]
"#;
        let err = PercussionTable::from_toml(toml_str).unwrap_err();
        assert_eq!(err, PercussionTableError::MissingDensity(2));
    }

    #[test]
    fn test_validation_empty_table() {
        let toml_str = r#"
low_note = 35
high_note = 37
"#;
        let err = PercussionTable::from_toml(toml_str).unwrap_err();
        assert_eq!(err, PercussionTableError::EmptyTable);
    }

    #[test]
    fn test_validation_base_velocity_out_of_range() {
        let mut table = PercussionTable::default();
        table.low_velocity = 128;
        assert_eq!(
            table.validate().unwrap_err(),
            PercussionTableError::BaseVelocityOutOfRange {
                percussion_low: 128,
                percussion_high: 45
            }
        );
    }

    #[test]
    fn test_validation_effective_velocity_out_of_range() {
        let toml_str = r#"
low_note = 35
high_note = 37
low_velocity = 100
high_velocity = 80

[[patterns]]
density = 0
hits = [{ step = 0, track = "PercussionLow", delta_velocity = 50 }]

[[patterns]]
density = 1
hits = [{ step = 0, track = "PercussionLow" }]

[[patterns]]
density = 2
hits = [{ step = 0, track = "PercussionLow" }]
"#;
        let err = PercussionTable::from_toml(toml_str).unwrap_err();
        assert_eq!(
            err,
            PercussionTableError::VelocityOutOfRange { density: 0, step: 0, velocity: 150 }
        );
    }

    #[test]
    fn test_custom_base_velocities() {
        let table = PercussionTable {
            low_velocity: 110,
            high_velocity: 90,
            percussion_gain: 1.0,
            ..PercussionTable::default()
        };

        let (percussion_low, percussion_high) = render_density(&table, 2, &[0; 16]);
        assert_eq!(percussion_low[0].velocity, 110);
        assert_eq!(percussion_high[0].velocity, 90); // step 2
        assert_eq!(percussion_high[1].velocity, 70); // step 3 (90 - 20)
    }

    #[test]
    fn test_percussion_gain_scales_velocities() {
        let table = PercussionTable {
            low_velocity: 100,
            high_velocity: 80,
            percussion_gain: 0.5,
            ..PercussionTable::default()
        };
        assert!(table.validate().is_ok(), "gain 0.5 valide");

        let (percussion_low, percussion_high) = render_density(&table, 2, &[0; 16]);
        assert!(percussion_low.iter().all(|n| n.velocity == 50));
        assert_eq!(percussion_high[0].velocity, 40); // step 2 : 80 * 0.5
        assert_eq!(percussion_high[1].velocity, 30); // step 3 : (80 - 20) * 0.5
    }

    #[test]
    fn test_percussion_gain_out_of_range_rejected() {
        for gain in [1.5, -0.5, f32::NAN] {
            let table = PercussionTable { percussion_gain: gain, ..PercussionTable::default() };
            assert!(
                matches!(table.validate(), Err(PercussionTableError::GainOutOfRange { .. })),
                "gain {gain} rejeté"
            );
        }
    }

    #[test]
    fn test_validation_velocity_humanize_bounds() {
        for humanize in [0, 1, 12, 40] {
            let table =
                PercussionTable { velocity_humanize: humanize, ..PercussionTable::default() };
            assert!(table.validate().is_ok(), "humanize {humanize} doit être valide");
        }
        for humanize in [41, 99, 255] {
            let table =
                PercussionTable { velocity_humanize: humanize, ..PercussionTable::default() };
            assert_eq!(
                table.validate(),
                Err(PercussionTableError::VelocityHumanizeOutOfRange { humanize }),
                "humanize {humanize} doit être rejeté"
            );
        }
    }

    #[test]
    fn test_velocity_humanize_zero_is_constant() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let table = PercussionTable {
            velocity_humanize: 0,
            percussion_gain: 1.0,
            ..PercussionTable::default()
        };
        let offsets = draw_percussion_humanize(0, 16, 16, &mut rng);
        assert!(offsets.iter().flatten().all(|&o| o == 0));

        let (percussion_low, percussion_high) = render_density(&table, 1, &offsets[0]);
        assert!(percussion_low.iter().all(|n| n.velocity == 55));
        assert!(percussion_high.iter().all(|n| n.velocity == 45));
    }

    #[test]
    fn test_velocity_humanize_deterministic_and_bounded() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let table = PercussionTable {
            velocity_humanize: 12,
            percussion_gain: 1.0,
            ..PercussionTable::default()
        };

        let mut rng_a = ChaCha8Rng::seed_from_u64(42);
        let mut rng_b = ChaCha8Rng::seed_from_u64(42);
        let offs_a = draw_percussion_humanize(table.velocity_humanize, 16, 16, &mut rng_a);
        let offs_b = draw_percussion_humanize(table.velocity_humanize, 16, 16, &mut rng_b);
        assert_eq!(offs_a, offs_b, "même graine ⇒ mêmes décalages");

        for slot in &offs_a {
            for &off in slot.iter() {
                assert!((-12..=12).contains(&off), "décalage {off} hors ±12");
            }
        }

        let (percussion_low, percussion_high) = render_density(&table, 1, &offs_a[0]);
        for n in percussion_low.iter().chain(percussion_high.iter()) {
            assert!((1..=127).contains(&n.velocity), "vélocité {} hors 1..=127", n.velocity);
        }
        assert!(percussion_low.iter().all(|n| (43..=67).contains(&n.velocity)));
        assert!(percussion_high.iter().all(|n| (33..=57).contains(&n.velocity)));
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut table = PercussionTable::default();
        table.patterns[0].weight = 7;
        let toml_str = toml::to_string(&table).expect("serialize TOML");
        let restored: PercussionTable = toml::from_str(&toml_str).expect("deserialize TOML");
        assert_eq!(table, restored);

        let json_str = serde_json::to_string(&table).expect("serialize JSON");
        let restored_json: PercussionTable =
            serde_json::from_str(&json_str).expect("deserialize JSON");
        assert_eq!(table, restored_json);
    }

    #[test]
    fn test_tuple_syntax_in_toml() {
        let toml_str = r#"
low_note = 35
high_note = 37

[[patterns]]
density = 0
hits = [[0, "PercussionLow"], [8, "percussionlow", 0]]

[[patterns]]
density = 1
hits = [[0, "PercussionLow"], [2, "PercussionHigh"]]

[[patterns]]
density = 2
hits = [[0, "PercussionLow"], [3, "PercussionHigh", -20]]
"#;
        let table = PercussionTable::from_toml(toml_str).expect("tuple syntax should parse");
        assert_eq!(table.patterns[0].hits[0], PercussionHit::low(0));
        assert_eq!(table.patterns[0].hits[1], PercussionHit::low(8));
        assert_eq!(table.patterns[2].hits[1], PercussionHit::high_with_delta(3, -20));
    }

    #[test]
    fn weighted_alternatives_render_deterministically_by_part_and_density() {
        let table = PercussionTable::from_toml(
            r#"
velocity_humanize = 0
patterns = [
    { density = 0, hits = [{step = 0, track = "PercussionLow"}] },
    { density = 1, weight = 1, hits = [{step = 1, track = "PercussionHigh"}] },
    { density = 1, weight = 9, hits = [{step = 7, track = "PercussionHigh"}] },
    { density = 2, hits = [{step = 8, track = "PercussionLow"}] },
]
"#,
        )
        .unwrap();
        assert!(table.has_alternatives());
        let tune = TunePlan::default();
        let mut heavy = 0;
        let mut distinct_parts = false;
        for seed in 0..256 {
            let plan = PercussionPatternPlan::new(&tune, &table, seed);
            let repeated = PercussionPatternPlan::new(&tune, &table, seed);
            let render = |plan: &PercussionPatternPlan, part, density| {
                percussion_notes(
                    &table,
                    plan.pattern(&table, part, 1, density, false),
                    16,
                    &[0; 16],
                )
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
        let mut table = PercussionTable::default();
        table.patterns.retain(|pattern| pattern.density != 1);
        for step in [1, 7] {
            let mut pattern = PercussionPattern::new(1, vec![PercussionHit::high(step)]);
            pattern.weight = u32::MAX;
            table.patterns.push(pattern);
        }
        table.validate().unwrap();
        let tune = TunePlan::default();
        let mut observed = [false; 2];
        for seed in 0..128 {
            let plan = PercussionPatternPlan::new(&tune, &table, seed);
            let (_, notes) =
                percussion_notes(&table, plan.pattern(&table, 0, 1, 1, false), 16, &[0; 16]);
            observed[usize::from(notes[0].start_step == 7)] = true;
        }
        assert_eq!(observed, [true, true]);
    }

    #[test]
    fn zero_weights_and_duplicate_cadences_are_rejected() {
        let mut table = PercussionTable::default();
        table.patterns[0].weight = 0;
        assert!(matches!(table.validate(), Err(PercussionTableError::ZeroWeight { density: 0 })));
        table.patterns[0].weight = 1;
        let mut cadence = PercussionPattern::new(1, vec![PercussionHit::high(15)]);
        cadence.weight = 0;
        table.cadence_patterns.push(cadence);
        assert!(matches!(table.validate(), Err(PercussionTableError::ZeroWeight { density: 1 })));
        table.cadence_patterns[0].weight = 1;
        table.cadence_patterns.push(table.cadence_patterns[0].clone());
        assert!(matches!(table.validate(), Err(PercussionTableError::DuplicateCadenceDensity(1))));
    }

    #[test]
    fn cadences_override_selected_euclidean_alternatives_without_entering_variety() {
        let table = PercussionTable::from_toml(
            r#"
patterns = [
    { density = 0, hits = [] },
    { density = 1, weight = 2, euclid = {pulses = 3, steps = 16, rotation = 1}, track = "PercussionHigh" },
    { density = 1, weight = 3, euclid = {pulses = 5, steps = 16, rotation = 2}, track = "PercussionHigh" },
    { density = 2, hits = [] },
]
cadence_patterns = [{density = 1, hits = [{step = 15, track = "PercussionHigh"}]}]
"#,
        )
        .unwrap();
        let restored = PercussionTable::from_toml(&toml::to_string(&table).unwrap()).unwrap();
        let tune = TunePlan::default();
        for seed in 0..32 {
            let plan = PercussionPatternPlan::new(&tune, &table, seed);
            let restored_plan = PercussionPatternPlan::new(&tune, &restored, seed);
            let ordinary =
                percussion_notes(&table, plan.pattern(&table, 0, 1, 1, false), 16, &[0; 16]);
            assert!([3, 5].contains(&ordinary.1.len()));
            assert_eq!(
                ordinary,
                percussion_notes(
                    &restored,
                    restored_plan.pattern(&restored, 0, 1, 1, false),
                    16,
                    &[0; 16],
                )
            );
            let cadence =
                percussion_notes(&table, plan.pattern(&table, 0, 1, 1, true), 16, &[0; 16]);
            assert_eq!(cadence.1.len(), 1);
            assert_eq!(cadence.1[0].start_step, 15);
            assert_eq!(
                percussion_notes(&table, plan.pattern(&table, 0, 1, 2, true), 16, &[0; 16]),
                percussion_notes(&table, plan.pattern(&table, 0, 1, 2, false), 16, &[0; 16]),
            );
        }
        let mut unique = PercussionTable::default();
        unique.cadence_patterns = table.cadence_patterns;
        assert!(!unique.has_alternatives());
    }
}

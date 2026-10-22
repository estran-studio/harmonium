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

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    harmony::RngCore,
    timeline::{Articulation, TimelineNote, TrackId},
};

/// Contenu brut de la table TOML embarquée de podorythmie pour le reel (D9).
pub const REEL_FEET_TOML: &str = include_str!("../../../profiles/reel_quebecois/feet.toml");

/// Un coup de pied individuel dans une mesure de podorythmie (D9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FootHit {
    /// Pas dans la mesure (`0..16`, soit `0..=15`).
    pub step: usize,
    /// Piste ciblée ([`TrackId::Heel`] ou [`TrackId::Toe`]).
    pub track: TrackId,
    /// Ajustement de vélocité relatif à la vélocité de base du membre.
    pub delta_velocity: i8,
}

impl FootHit {
    /// Crée un coup de pied avec pas, piste et delta de vélocité.
    #[must_use]
    pub const fn new(step: usize, track: TrackId, delta_velocity: i8) -> Self {
        Self { step, track, delta_velocity }
    }

    /// Raccourci pour un coup de talon (delta de vélocité nul).
    #[must_use]
    pub const fn heel(step: usize) -> Self {
        Self { step, track: TrackId::Heel, delta_velocity: 0 }
    }

    /// Raccourci pour un coup de pointe (delta de vélocité nul).
    #[must_use]
    pub const fn toe(step: usize) -> Self {
        Self { step, track: TrackId::Toe, delta_velocity: 0 }
    }

    /// Raccourci pour un coup de pointe avec delta de vélocité explicite.
    #[must_use]
    pub const fn toe_with_delta(step: usize, delta_velocity: i8) -> Self {
        Self { step, track: TrackId::Toe, delta_velocity }
    }
}

impl From<(usize, TrackId)> for FootHit {
    fn from((step, track): (usize, TrackId)) -> Self {
        Self { step, track, delta_velocity: 0 }
    }
}

impl From<(usize, TrackId, i8)> for FootHit {
    fn from((step, track, delta_velocity): (usize, TrackId, i8)) -> Self {
        Self { step, track, delta_velocity }
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

#[derive(Deserialize)]
#[serde(untagged)]
enum RawFootHit {
    Struct {
        step: usize,
        #[serde(deserialize_with = "deserialize_foot_track")]
        track: TrackId,
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
            RawFootHit::Struct { step, track, delta_velocity } => {
                Ok(Self { step, track, delta_velocity })
            }
            RawFootHit::Tuple2(step, track) => Ok(Self { step, track, delta_velocity: 0 }),
            RawFootHit::Tuple3(step, track, delta_velocity) => {
                Ok(Self { step, track, delta_velocity })
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
        };
        raw.serialize(serializer)
    }
}

/// Pattern de coups de pieds pour un niveau de densité donné (D9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeetPattern {
    /// Niveau de densité (`0`, `1` ou `2`).
    pub density: u8,
    /// Liste des coups composant la mesure.
    #[serde(default)]
    pub hits: Vec<FootHit>,
}

impl FeetPattern {
    /// Crée un nouveau [`FeetPattern`].
    #[must_use]
    pub const fn new(density: u8, hits: Vec<FootHit>) -> Self {
        Self { density, hits }
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
    patterns: Vec<FeetPattern>,
    #[serde(default)]
    densities: Vec<FeetPattern>,
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
    /// Patterns de coups par niveau de densité.
    #[serde(default)]
    pub patterns: Vec<FeetPattern>,
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
            patterns,
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
    /// Une densité apparaît plus d'une fois dans la table.
    DuplicateDensity(u8),
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
            Self::DuplicateDensity(density) => {
                write!(f, "densité dupliquée dans la table de podorythmie : {density}")
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
            patterns: Vec::new(),
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
    /// - densités `0`, `1` et `2` présentes sans doublon ;
    /// - chaque coup a un pas dans `0..16` (`0..=15`) ;
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

        let mut seen_densities = HashSet::new();

        for pattern in &self.patterns {
            if !seen_densities.insert(pattern.density) {
                return Err(FeetTableError::DuplicateDensity(pattern.density));
            }

            for hit in &pattern.hits {
                let base_velocity = match hit.track {
                    TrackId::Heel => self.heel_velocity,
                    TrackId::Toe => self.toe_velocity,
                    other => {
                        return Err(FeetTableError::InvalidTrack {
                            density: pattern.density,
                            step: hit.step,
                            track: other,
                        });
                    }
                };

                let effective_velocity = i16::from(base_velocity) + i16::from(hit.delta_velocity);
                if !(0..=127).contains(&effective_velocity) {
                    return Err(FeetTableError::VelocityOutOfRange {
                        density: pattern.density,
                        step: hit.step,
                        velocity: effective_velocity,
                    });
                }
            }
        }

        for required in 0..=2u8 {
            if !seen_densities.contains(&required) {
                return Err(FeetTableError::MissingDensity(required));
            }
        }

        Ok(())
    }

    /// Retourne le pattern pour une densité donnée, ou `None` s'il n'existe pas.
    #[must_use]
    pub fn pattern(&self, density: u8) -> Option<&FeetPattern> {
        self.patterns.iter().find(|p| p.density == density)
    }

    /// Retourne la tranche des coups pour une densité donnée bornée à `0..=2`.
    #[must_use]
    pub fn hits_for_density(&self, density: u8) -> &[FootHit] {
        let clamped = density.min(2);
        self.pattern(clamped).map_or(&[], |p| p.hits.as_slice())
    }

    /// Raccourci vers [`feet_notes`].
    #[must_use]
    pub fn feet_notes(
        &self,
        density: u8,
        steps: usize,
        humanize: &[i8],
    ) -> (Vec<TimelineNote>, Vec<TimelineNote>) {
        feet_notes(self, density, steps, humanize)
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
            *step = (rng.next_range_usize(0..(2 * h + 1) as usize) as i32 - h) as i8;
        }
    }
    offsets
}

/// Génère les notes de podorythmie (talon, pointe) pour une mesure de reel (D9).
///
/// # Paramètres
/// - `table` : table de podorythmie ([`FeetTable`]) portant les patterns et les vélocités de base.
/// - `density` : niveau de densité souhaité (`0..=2`). Les valeurs supérieures
///   à 2 sont bornées à 2 via `density.min(2)`.
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
    density: u8,
    steps: usize,
    humanize: &[i8],
) -> (Vec<TimelineNote>, Vec<TimelineNote>) {
    debug_assert_eq!(humanize.len(), steps);

    let clamped_density = density.min(2);
    let hits = table.hits_for_density(clamped_density);

    let mut heel_notes = Vec::new();
    let mut toe_notes = Vec::new();

    for hit in hits {
        let (pitch, base_velocity, target_vec): (u8, u8, &mut Vec<TimelineNote>) = match hit.track {
            TrackId::Heel => (table.heel_note, table.heel_velocity, &mut heel_notes),
            TrackId::Toe => (table.toe_note, table.toe_velocity, &mut toe_notes),
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
        });
    }

    heel_notes.sort_by_key(|n| n.start_step);
    toe_notes.sort_by_key(|n| n.start_step);

    (heel_notes, toe_notes)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let (heel, toe) = feet_notes(&table, 0, 16, &[0; 16]);

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
        let (heel, toe) = feet_notes(&table, 1, 16, &[0; 16]);

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
        let (heel, toe) = feet_notes(&table, 2, 16, &[0; 16]);

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
        let expected_2 = feet_notes(&table, 2, 16, &[0; 16]);

        for d in [3, 4, 10, 99, 255] {
            let notes = feet_notes(&table, d, 16, &[0; 16]);
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
    fn test_validation_duplicate_density() {
        let toml_str = r#"
heel_note = 35
toe_note = 37

[[patterns]]
density = 0
hits = [{ step = 0, track = "Heel" }]

[[patterns]]
density = 1
hits = [{ step = 0, track = "Heel" }]

[[patterns]]
density = 1
hits = [{ step = 4, track = "Heel" }]

[[patterns]]
density = 2
hits = [{ step = 0, track = "Heel" }]
"#;
        let err = FeetTable::from_toml(toml_str).unwrap_err();
        assert_eq!(err, FeetTableError::DuplicateDensity(1));
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

        let (heel, toe) = table.feet_notes(2, 16, &[0; 16]);
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

        let (heel, toe) = table.feet_notes(2, 16, &[0; 16]);
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

        let (heel, toe) = feet_notes(&table, 1, 16, &offsets[0]);
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

        let (heel, toe) = feet_notes(&table, 1, 16, &offs_a[0]);
        for n in heel.iter().chain(toe.iter()) {
            assert!((1..=127).contains(&n.velocity), "vélocité {} hors 1..=127", n.velocity);
        }
        assert!(heel.iter().all(|n| (43..=67).contains(&n.velocity)));
        assert!(toe.iter().all(|n| (33..=57).contains(&n.velocity)));
    }

    #[test]
    fn test_serde_roundtrip() {
        let table = FeetTable::default();
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
}

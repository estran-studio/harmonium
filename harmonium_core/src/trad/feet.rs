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

use crate::timeline::{Articulation, TimelineNote, TrackId};

/// Contenu brut de la table TOML embarquée de podorythmie pour le reel (D9).
pub const REEL_FEET_TOML: &str = include_str!("reel_feet.toml");

/// Nombre de pas par mesure dans un reel québécois en 2/2 (D9).
pub const REEL_STEPS: usize = 16;

/// Note MIDI provisoire attribuée au talon (`Heel`) en phase 1 (D10, canal 9 percussion).
pub const PITCH_HEEL: u8 = 36;

/// Note MIDI provisoire attribuée à la pointe (`Toe`) en phase 1 (D10, canal 9 percussion).
pub const PITCH_TOE: u8 = 37;

/// Vélocité MIDI par défaut pour les frappes de talon.
pub const DEFAULT_HEEL_VELOCITY: u8 = 100;

/// Vélocité MIDI par défaut pour les frappes de pointe.
pub const DEFAULT_TOE_VELOCITY: u8 = 80;

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

const fn default_heel_velocity() -> u8 {
    DEFAULT_HEEL_VELOCITY
}

const fn default_toe_velocity() -> u8 {
    DEFAULT_TOE_VELOCITY
}

#[derive(Deserialize)]
struct RawFeetTable {
    #[serde(default = "default_heel_velocity")]
    heel_base_velocity: u8,
    #[serde(default = "default_toe_velocity")]
    toe_base_velocity: u8,
    #[serde(default)]
    patterns: Vec<FeetPattern>,
    #[serde(default)]
    densities: Vec<FeetPattern>,
}

/// Table de podorythmie portée en données (D9, constitution V).
///
/// Contient les vélocités de base pour le talon et la pointe, ainsi que les
/// patterns associés à chaque densité (`0..=2`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FeetTable {
    /// Vélocité de base du talon (MIDI `0..=127`).
    #[serde(default = "default_heel_velocity")]
    pub heel_base_velocity: u8,
    /// Vélocité de base de la pointe (MIDI `0..=127`).
    #[serde(default = "default_toe_velocity")]
    pub toe_base_velocity: u8,
    /// Patterns de coups par niveau de densité.
    #[serde(default)]
    pub patterns: Vec<FeetPattern>,
}

impl<'de> Deserialize<'de> for FeetTable {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawFeetTable::deserialize(deserializer)?;
        let patterns = if !raw.patterns.is_empty() { raw.patterns } else { raw.densities };
        Ok(Self {
            heel_base_velocity: raw.heel_base_velocity,
            toe_base_velocity: raw.toe_base_velocity,
            patterns,
        })
    }
}

/// Erreur survenue lors du chargement ou de la validation d'une [`FeetTable`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeetTableError {
    /// Erreur de syntaxe TOML.
    TomlParse(String),
    /// La table ne contient aucun pattern.
    EmptyTable,
    /// Une densité obligatoire (`0`, `1` ou `2`) est absente.
    MissingDensity(u8),
    /// Une densité apparaît plus d'une fois dans la table.
    DuplicateDensity(u8),
    /// Un pas est en dehors de `0..16`.
    StepOutOfRange { density: u8, step: usize },
    /// Piste invalide (seules `TrackId::Heel` et `TrackId::Toe` sont permises).
    InvalidTrack { density: u8, step: usize, track: TrackId },
    /// La vélocité effective (`base + delta`) dépasse la plage MIDI `0..=127`.
    VelocityOutOfRange { density: u8, step: usize, velocity: i16 },
    /// Une vélocité de base est hors de la plage MIDI `0..=127`.
    BaseVelocityOutOfRange { heel: u8, toe: u8 },
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
            Self::StepOutOfRange { density, step } => {
                write!(f, "pas {step} hors limite (attendu 0..16) à la densité {density}")
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
            heel_base_velocity: DEFAULT_HEEL_VELOCITY,
            toe_base_velocity: DEFAULT_TOE_VELOCITY,
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
    /// - densités `0`, `1` et `2` présentes sans doublon ;
    /// - chaque coup a un pas dans `0..16` (`0..=15`) ;
    /// - chaque piste est `Heel` ou `Toe` ;
    /// - chaque vélocité calculée (`base + delta`) est dans `0..=127`.
    pub fn validate(&self) -> Result<(), FeetTableError> {
        if self.heel_base_velocity > 127 || self.toe_base_velocity > 127 {
            return Err(FeetTableError::BaseVelocityOutOfRange {
                heel: self.heel_base_velocity,
                toe: self.toe_base_velocity,
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
                if hit.step >= REEL_STEPS {
                    return Err(FeetTableError::StepOutOfRange {
                        density: pattern.density,
                        step: hit.step,
                    });
                }

                let base_velocity = match hit.track {
                    TrackId::Heel => self.heel_base_velocity,
                    TrackId::Toe => self.toe_base_velocity,
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
    pub fn feet_notes(&self, density: u8, steps: usize) -> (Vec<TimelineNote>, Vec<TimelineNote>) {
        feet_notes(self, density, steps)
    }
}

/// Génère les notes de podorythmie (talon, pointe) pour une mesure de reel (D9).
///
/// # Paramètres
/// - `table` : table de podorythmie ([`FeetTable`]) portant les patterns et les vélocités de base.
/// - `density` : niveau de densité souhaité (`0..=2`). Les valeurs supérieures
///   à 2 sont bornées à 2 via `density.min(2)`.
/// - `steps` : nombre de pas dans la mesure. En phase 1, seul le reel en 2/2 (16 pas) est
///   couvert. Si `steps != 16`, aucune note n'est produite et la fonction
///   retourne `(Vec::new(), Vec::new())`.
///
/// # Retour
/// Tuple `(talon_notes, pointe_notes)` :
/// - premier élément : notes du talon ([`TrackId::Heel`]), `pitch = 36` ;
/// - second élément : notes de la pointe ([`TrackId::Toe`]), `pitch = 37`.
///
/// Toutes les notes produites ont `duration_steps = 0` et `articulation = Articulation::Trigger`.
/// La vélocité est calculée comme `(base_velocity + delta_velocity).clamp(0, 127) as u8`.
#[must_use]
pub fn feet_notes(
    table: &FeetTable,
    density: u8,
    steps: usize,
) -> (Vec<TimelineNote>, Vec<TimelineNote>) {
    if steps != REEL_STEPS {
        return (Vec::new(), Vec::new());
    }

    let clamped_density = density.min(2);
    let hits = table.hits_for_density(clamped_density);

    let mut heel_notes = Vec::new();
    let mut toe_notes = Vec::new();

    for hit in hits {
        let (pitch, base_velocity, target_vec) = match hit.track {
            TrackId::Heel => (PITCH_HEEL, table.heel_base_velocity, &mut heel_notes),
            TrackId::Toe => (PITCH_TOE, table.toe_base_velocity, &mut toe_notes),
            _ => continue,
        };

        let effective = i16::from(base_velocity) + i16::from(hit.delta_velocity);
        let velocity = effective.clamp(0, 127) as u8;

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
        assert_eq!(table.heel_base_velocity, 100);
        assert_eq!(table.toe_base_velocity, 80);
    }

    #[test]
    fn test_exact_d9_positions_density_0() {
        let table = FeetTable::default();
        let (heel, toe) = feet_notes(&table, 0, 16);

        let heel_steps: Vec<usize> = heel.iter().map(|n| n.start_step).collect();
        assert_eq!(heel_steps, vec![0, 8]);
        assert!(toe.is_empty());

        for note in &heel {
            assert_eq!(note.pitch, PITCH_HEEL);
            assert_eq!(note.velocity, 100);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }
    }

    #[test]
    fn test_exact_d9_positions_density_1() {
        let table = FeetTable::default();
        let (heel, toe) = feet_notes(&table, 1, 16);

        let heel_steps: Vec<usize> = heel.iter().map(|n| n.start_step).collect();
        let toe_steps: Vec<usize> = toe.iter().map(|n| n.start_step).collect();

        assert_eq!(heel_steps, vec![0, 4, 8, 12]);
        assert_eq!(toe_steps, vec![2, 6, 10, 14]);

        for note in &heel {
            assert_eq!(note.pitch, PITCH_HEEL);
            assert_eq!(note.velocity, 100);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }

        for note in &toe {
            assert_eq!(note.pitch, PITCH_TOE);
            assert_eq!(note.velocity, 80);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }
    }

    #[test]
    fn test_exact_d9_positions_density_2() {
        let table = FeetTable::default();
        let (heel, toe) = feet_notes(&table, 2, 16);

        let heel_steps: Vec<usize> = heel.iter().map(|n| n.start_step).collect();
        let toe_steps: Vec<usize> = toe.iter().map(|n| n.start_step).collect();

        assert_eq!(heel_steps, vec![0, 4, 8, 12]);
        assert_eq!(toe_steps, vec![2, 3, 6, 7, 10, 11, 14, 15]);

        for note in &heel {
            assert_eq!(note.pitch, PITCH_HEEL);
            assert_eq!(note.velocity, 100);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
        }

        for note in &toe {
            assert_eq!(note.pitch, PITCH_TOE);
            assert_eq!(note.duration_steps, 0);
            assert_eq!(note.articulation, Articulation::Trigger);
            if note.start_step % 2 == 1 {
                // Pas impairs : vélocité de base (80) - 20 = 60
                assert_eq!(note.velocity, 60, "pas impair {} attendu vel 60", note.start_step);
            } else {
                // Pas pairs : vélocité de base (80)
                assert_eq!(note.velocity, 80, "pas pair {} attendu vel 80", note.start_step);
            }
        }
    }

    #[test]
    fn test_density_clamping() {
        let table = FeetTable::default();
        let expected_2 = feet_notes(&table, 2, 16);

        for d in [3, 4, 10, 99, 255] {
            let notes = feet_notes(&table, d, 16);
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
    fn test_steps_not_16_returns_empty() {
        let table = FeetTable::default();
        for invalid_steps in [0, 1, 8, 12, 15, 17, 32] {
            for density in [0, 1, 2, 5] {
                let (heel, toe) = feet_notes(&table, density, invalid_steps);
                assert!(
                    heel.is_empty(),
                    "talon non vide pour steps={invalid_steps}, density={density}"
                );
                assert!(
                    toe.is_empty(),
                    "pointe non vide pour steps={invalid_steps}, density={density}"
                );
            }
        }
    }

    #[test]
    fn test_validation_step_out_of_range() {
        let toml_str = r#"
heel_base_velocity = 100
toe_base_velocity = 80

[[patterns]]
density = 0
hits = [{ step = 0, track = "Heel" }, { step = 16, track = "Heel" }]

[[patterns]]
density = 1
hits = [{ step = 0, track = "Heel" }]

[[patterns]]
density = 2
hits = [{ step = 0, track = "Heel" }]
"#;
        let err = FeetTable::from_toml(toml_str).unwrap_err();
        assert_eq!(err, FeetTableError::StepOutOfRange { density: 0, step: 16 });
    }

    #[test]
    fn test_validation_missing_density() {
        let toml_str = r#"
heel_base_velocity = 100
toe_base_velocity = 80

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
heel_base_velocity = 100
toe_base_velocity = 80

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
heel_base_velocity = 100
toe_base_velocity = 80
"#;
        let err = FeetTable::from_toml(toml_str).unwrap_err();
        assert_eq!(err, FeetTableError::EmptyTable);
    }

    #[test]
    fn test_validation_base_velocity_out_of_range() {
        let mut table = FeetTable::default();
        table.heel_base_velocity = 128;
        assert_eq!(
            table.validate().unwrap_err(),
            FeetTableError::BaseVelocityOutOfRange { heel: 128, toe: 80 }
        );
    }

    #[test]
    fn test_validation_effective_velocity_out_of_range() {
        let toml_str = r#"
heel_base_velocity = 100
toe_base_velocity = 80

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
        let mut table = FeetTable::default();
        table.heel_base_velocity = 110;
        table.toe_base_velocity = 90;

        let (heel, toe) = table.feet_notes(2, 16);
        assert_eq!(heel[0].velocity, 110);
        assert_eq!(toe[0].velocity, 90); // step 2
        assert_eq!(toe[1].velocity, 70); // step 3 (90 - 20)
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
heel_base_velocity = 100
toe_base_velocity = 80

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

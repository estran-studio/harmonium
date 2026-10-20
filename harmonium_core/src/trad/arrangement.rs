//! Arrangement du reel : tours d'un air et entrées progressives des pistes (D11).
//!
//! Ce module porte deux choses :
//! - [`TunePosition`] : la position structurée `(tune_index, tour, part, pass,
//!   bar_in_strain)` d'une mesure, calculée en un seul endroit
//!   ([`TunePosition::at_bar`]) et exposée au jeu dans
//!   [`crate::report::MeasureSnapshot`] ;
//! - [`ReelArrangement`] : la table `reel_arrangement.toml` (constitution V) qui
//!   dit, pour chaque piste, la première position où elle joue.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use super::fiddle::BARS_PER_STRAIN;
use crate::timeline::TrackId;

/// Contenu brut de la table TOML embarquée d'arrangement du reel (D11).
pub const REEL_ARRANGEMENT_TOML: &str = include_str!("reel_arrangement.toml");

/// Nombre de mesures d'un tour AABB complet (A1 + A2 + B1 + B2).
pub const BARS_PER_TOUR: usize = 4 * BARS_PER_STRAIN;

/// Nombre maximal de tours par air et borne haute du `tour` d'une entrée.
pub const MAX_TOURS: usize = 16;

/// Partie d'un air AABB.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Part {
    /// Partie A (mesures 1–16 d'un tour : A1 puis sa reprise A2).
    A,
    /// Partie B (mesures 17–32 d'un tour : B1 puis sa reprise B2).
    B,
}

impl Part {
    const fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "A" => Some(Self::A),
            "B" => Some(Self::B),
            _ => None,
        }
    }
}

/// Rang d'une passe de strain dans un air : `0` = tour 1 / A / passe 1, croissant
/// par passe de 8 mesures. Seul endroit où `(tour, part, pass)` devient un ordre.
const fn pass_ordinal(tour: usize, part: Part, pass: usize) -> usize {
    ((tour.saturating_sub(1)) * 2 + part.index()) * 2 + pass.saturating_sub(1)
}

/// Position structurée d'une mesure dans la suite d'airs d'une session.
///
/// Conventions : `tune_index` est 0-based (comme `ActiveTune::tune_index`) ;
/// `tour`, `pass` et `bar_in_strain` sont 1-based, comme les numéros de mesure
/// du contrat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TunePosition {
    /// Indice 0-based de l'air dans la session.
    pub tune_index: usize,
    /// Tour de l'air (`1..=tours`).
    pub tour: usize,
    /// Nombre de tours de cet air (`tour == tours` : dernier tour).
    pub tours: usize,
    /// Partie courante (A ou B).
    pub part: Part,
    /// Passe de la partie (`1` ou `2`).
    pub pass: usize,
    /// Mesure dans la passe (`1..=8`).
    pub bar_in_strain: usize,
}

impl TunePosition {
    /// Position de la mesure 1-based `bar` quand chaque air dure `tours` tours.
    ///
    /// C'est l'unique conversion mesure → position du moteur trad.
    #[must_use]
    pub const fn at_bar(bar: usize, tours: usize) -> Self {
        let tours = if tours == 0 { 1 } else { tours };
        let zero = bar.saturating_sub(1);
        let in_tune = zero % (BARS_PER_TOUR * tours);
        let in_tour = in_tune % BARS_PER_TOUR;
        let in_part = in_tour % (2 * BARS_PER_STRAIN);
        Self {
            tune_index: zero / (BARS_PER_TOUR * tours),
            tour: in_tune / BARS_PER_TOUR + 1,
            tours,
            part: if in_tour < 2 * BARS_PER_STRAIN { Part::A } else { Part::B },
            pass: in_part / BARS_PER_STRAIN + 1,
            bar_in_strain: in_part % BARS_PER_STRAIN + 1,
        }
    }

    /// Indice 0-based de la mesure dans un tour AABB (`0..32`) : indexe la
    /// ligne de violon et l'harmonie, identiques à chaque tour.
    #[must_use]
    pub const fn bar_in_tour(&self) -> usize {
        pass_ordinal(1, self.part, self.pass) * BARS_PER_STRAIN + self.bar_in_strain - 1
    }

    /// Vrai au dernier tour de l'air.
    #[must_use]
    pub const fn is_last_tour(&self) -> bool {
        self.tour >= self.tours
    }

    const fn ordinal(&self) -> usize {
        pass_ordinal(self.tour, self.part, self.pass)
    }
}

/// Première position où une piste joue (`tour`, `part`, `pass`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPoint {
    /// Tour d'entrée (`1..=`[`MAX_TOURS`]) ; borné au dernier tour de l'air.
    pub tour: usize,
    /// Partie d'entrée.
    pub part: Part,
    /// Passe d'entrée (`1` ou `2`).
    pub pass: usize,
}

impl EntryPoint {
    /// Entrée dès le début de l'air.
    #[must_use]
    pub const fn from_start() -> Self {
        Self { tour: 1, part: Part::A, pass: 1 }
    }
}

/// Points d'entrée des cinq pistes (clés TOML = noms de [`TrackId`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TrackEntries {
    #[serde(rename = "Melody")]
    pub melody: EntryPoint,
    #[serde(rename = "Chords")]
    pub chords: EntryPoint,
    #[serde(rename = "Bass")]
    pub bass: EntryPoint,
    #[serde(rename = "Heel")]
    pub heel: EntryPoint,
    #[serde(rename = "Toe")]
    pub toe: EntryPoint,
}

impl TrackEntries {
    const fn get(&self, track: TrackId) -> EntryPoint {
        match track {
            TrackId::Melody => self.melody,
            TrackId::Chords => self.chords,
            TrackId::Bass => self.bass,
            TrackId::Heel => self.heel,
            TrackId::Toe => self.toe,
        }
    }

    fn all(&self) -> [(TrackId, EntryPoint); 5] {
        TrackId::ALL.map(|track| (track, self.get(track)))
    }
}

/// Erreur de chargement ou de validation d'une [`ReelArrangement`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrangementError {
    /// Erreur de syntaxe TOML.
    TomlParse(String),
    /// `tours_per_tune` hors de `1..=`[`MAX_TOURS`].
    ToursPerTuneOutOfRange(i64),
    /// Une clé de `[entries]` n'est pas une piste connue.
    UnknownTrack(String),
    /// Une piste n'a pas de point d'entrée.
    MissingTrack(TrackId),
    /// `tour` hors de `1..=`[`MAX_TOURS`].
    TourOutOfRange { track: TrackId, tour: i64 },
    /// `part` n'est ni `"A"` ni `"B"`.
    UnknownPart { track: TrackId, part: String },
    /// `pass` n'est ni `1` ni `2`.
    PassOutOfRange { track: TrackId, pass: i64 },
}

impl fmt::Display for ArrangementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TomlParse(msg) => {
                write!(f, "erreur de lecture TOML de la table d'arrangement : {msg}")
            }
            Self::ToursPerTuneOutOfRange(n) => {
                write!(f, "tours_per_tune hors limites (attendu 1..={MAX_TOURS}) : {n}")
            }
            Self::UnknownTrack(name) => write!(
                f,
                "piste inconnue dans l'arrangement : « {name} » (attendu Melody, Chords, Bass, Heel ou Toe)"
            ),
            Self::MissingTrack(track) => {
                write!(f, "point d'entrée manquant pour la piste {track:?}")
            }
            Self::TourOutOfRange { track, tour } => write!(
                f,
                "tour d'entrée hors limites pour {track:?} (attendu 1..={MAX_TOURS}) : {tour}"
            ),
            Self::UnknownPart { track, part } => {
                write!(f, "partie inconnue pour {track:?} : « {part} » (attendu \"A\" ou \"B\")")
            }
            Self::PassOutOfRange { track, pass } => {
                write!(f, "passe d'entrée hors limites pour {track:?} (attendu 1 ou 2) : {pass}")
            }
        }
    }
}

impl std::error::Error for ArrangementError {}

/// Table d'arrangement du reel, portée en données (D11, constitution V).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ReelArrangement {
    /// Nombre de tours AABB joués avant de tirer un nouvel air (`1..=`[`MAX_TOURS`]).
    pub tours_per_tune: usize,
    /// `false` : à partir du 2ᵉ air, toutes les pistes jouent dès le début.
    pub intro_each_tune: bool,
    /// Point d'entrée de chaque piste.
    pub entries: TrackEntries,
}

impl<'de> Deserialize<'de> for ReelArrangement {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawArrangement::deserialize(deserializer)?;
        Self::try_from(raw).map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
struct RawEntry {
    tour: i64,
    part: String,
    pass: i64,
}

const fn default_tours_per_tune() -> i64 {
    3
}

#[derive(Deserialize)]
struct RawArrangement {
    #[serde(default = "default_tours_per_tune")]
    tours_per_tune: i64,
    #[serde(default)]
    intro_each_tune: bool,
    entries: std::collections::BTreeMap<String, RawEntry>,
}

impl TryFrom<RawArrangement> for ReelArrangement {
    type Error = ArrangementError;

    fn try_from(mut raw: RawArrangement) -> Result<Self, Self::Error> {
        if let Some(unknown) =
            raw.entries.keys().find(|k| !TrackId::ALL.iter().any(|t| format!("{t:?}") == **k))
        {
            return Err(ArrangementError::UnknownTrack(unknown.clone()));
        }
        let tours_per_tune = usize::try_from(raw.tours_per_tune)
            .map_err(|_| ArrangementError::ToursPerTuneOutOfRange(raw.tours_per_tune))?;

        let mut take = |track: TrackId| -> Result<EntryPoint, ArrangementError> {
            let entry = raw
                .entries
                .remove(&format!("{track:?}"))
                .ok_or(ArrangementError::MissingTrack(track))?;
            let part = Part::parse(&entry.part)
                .ok_or(ArrangementError::UnknownPart { track, part: entry.part })?;
            let tour = usize::try_from(entry.tour)
                .map_err(|_| ArrangementError::TourOutOfRange { track, tour: entry.tour })?;
            let pass = usize::try_from(entry.pass)
                .map_err(|_| ArrangementError::PassOutOfRange { track, pass: entry.pass })?;
            Ok(EntryPoint { tour, part, pass })
        };
        let entries = TrackEntries {
            melody: take(TrackId::Melody)?,
            chords: take(TrackId::Chords)?,
            bass: take(TrackId::Bass)?,
            heel: take(TrackId::Heel)?,
            toe: take(TrackId::Toe)?,
        };
        let table = Self { tours_per_tune, intro_each_tune: raw.intro_each_tune, entries };
        table.validate()?;
        Ok(table)
    }
}

impl Default for ReelArrangement {
    /// Charge la table embarquée `reel_arrangement.toml` sans jamais paniquer
    /// (sa validité est garantie par les tests et par
    /// [`crate::tuning::TuningParams::validate`]) ; en cas d'erreur de données,
    /// repli sur trois tours avec toutes les pistes dès le début.
    fn default() -> Self {
        Self::from_toml(REEL_ARRANGEMENT_TOML).unwrap_or_else(|_| Self::from_start(3))
    }
}

impl ReelArrangement {
    /// Arrangement plat : `tours_per_tune` tours, toutes les pistes dès le début.
    #[must_use]
    pub const fn from_start(tours_per_tune: usize) -> Self {
        let start = EntryPoint::from_start();
        Self {
            tours_per_tune,
            intro_each_tune: false,
            entries: TrackEntries {
                melody: start,
                chords: start,
                bass: start,
                heel: start,
                toe: start,
            },
        }
    }

    /// Charge et valide une table depuis une chaîne TOML au format `reel_arrangement.toml`.
    pub fn from_toml(toml_str: &str) -> Result<Self, ArrangementError> {
        let raw: RawArrangement =
            toml::from_str(toml_str).map_err(|e| ArrangementError::TomlParse(e.to_string()))?;
        Self::try_from(raw)
    }

    /// Valide `tours_per_tune`, les `tour` (`1..=`[`MAX_TOURS`]) et les `pass` (`1` ou `2`).
    pub fn validate(&self) -> Result<(), ArrangementError> {
        if !(1..=MAX_TOURS).contains(&self.tours_per_tune) {
            return Err(ArrangementError::ToursPerTuneOutOfRange(
                i64::try_from(self.tours_per_tune).unwrap_or(i64::MAX),
            ));
        }
        for (track, entry) in self.entries.all() {
            if !(1..=MAX_TOURS).contains(&entry.tour) {
                return Err(ArrangementError::TourOutOfRange {
                    track,
                    tour: i64::try_from(entry.tour).unwrap_or(i64::MAX),
                });
            }
            if !(1..=2).contains(&entry.pass) {
                return Err(ArrangementError::PassOutOfRange {
                    track,
                    pass: i64::try_from(entry.pass).unwrap_or(i64::MAX),
                });
            }
        }
        Ok(())
    }

    /// Point d'entrée d'une piste.
    #[must_use]
    pub const fn entry(&self, track: TrackId) -> EntryPoint {
        self.entries.get(track)
    }

    /// Vrai si l'arrangement autorise `track` à jouer à `position`.
    ///
    /// Ne tient pas compte des drapeaux de jeu (`enable_*`), qui sont des
    /// plafonds appliqués par le générateur.
    #[must_use]
    pub fn allows(&self, track: TrackId, position: &TunePosition) -> bool {
        if position.tune_index > 0 && !self.intro_each_tune {
            return true;
        }
        let entry = self.entry(track);
        let last_tour = self.tours_per_tune.max(1);
        position.ordinal() >= pass_ordinal(entry.tour.min(last_tour), entry.part, entry.pass)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_table_validates_with_documented_defaults() -> Result<(), ArrangementError> {
        let table = ReelArrangement::from_toml(REEL_ARRANGEMENT_TOML)?;
        assert_eq!(table, ReelArrangement::default());
        assert_eq!(table.tours_per_tune, 3);
        assert!(!table.intro_each_tune);
        assert_eq!(table.entry(TrackId::Melody), EntryPoint::from_start());
        assert_eq!(table.entry(TrackId::Bass), EntryPoint { tour: 1, part: Part::A, pass: 2 });
        assert_eq!(table.entry(TrackId::Toe), EntryPoint { tour: 2, part: Part::A, pass: 1 });
        Ok(())
    }

    #[test]
    fn position_maps_bars_to_structure() {
        let p = TunePosition::at_bar(1, 3);
        assert_eq!(
            p,
            TunePosition {
                tune_index: 0,
                tour: 1,
                tours: 3,
                part: Part::A,
                pass: 1,
                bar_in_strain: 1
            }
        );
        let p = TunePosition::at_bar(9, 3);
        assert_eq!((p.part, p.pass, p.bar_in_strain), (Part::A, 2, 1));
        let p = TunePosition::at_bar(25, 3);
        assert_eq!((p.part, p.pass, p.bar_in_strain, p.tour), (Part::B, 2, 1, 1));
        assert_eq!(p.bar_in_tour(), 24);
        let p = TunePosition::at_bar(33, 3);
        assert_eq!(
            (p.tour, p.part, p.pass, p.bar_in_strain, p.bar_in_tour()),
            (2, Part::A, 1, 1, 0)
        );
        let p = TunePosition::at_bar(96, 3);
        assert_eq!((p.tune_index, p.tour, p.bar_in_tour()), (0, 3, 31));
        assert!(p.is_last_tour());
        let p = TunePosition::at_bar(97, 3);
        assert_eq!((p.tune_index, p.tour, p.bar_in_tour()), (1, 1, 0));
    }

    #[test]
    fn entries_beyond_last_tour_are_clamped() {
        let mut table = ReelArrangement::default();
        assert!(!table.allows(TrackId::Heel, &TunePosition::at_bar(1, 3)));
        table.tours_per_tune = 1;
        // Tour unique : l'entrée du tour 2 est ramenée au dernier tour (le 1er).
        assert!(table.allows(TrackId::Heel, &TunePosition::at_bar(1, 1)));
    }

    fn toml_with(replace: (&str, &str)) -> String {
        let ok = "tours_per_tune = 3\n[entries]\n\
            Melody = { tour = 1, part = \"A\", pass = 1 }\n\
            Chords = { tour = 1, part = \"A\", pass = 2 }\n\
            Bass = { tour = 1, part = \"A\", pass = 2 }\n\
            Heel = { tour = 2, part = \"A\", pass = 1 }\n\
            Toe = { tour = 2, part = \"A\", pass = 1 }\n";
        assert!(ok.contains(replace.0));
        ok.replace(replace.0, replace.1)
    }

    #[test]
    fn invalid_tables_yield_explicit_errors() {
        assert_eq!(
            ReelArrangement::from_toml(&toml_with(("Heel = { tour = 2", "Heel = { tour = 0"))),
            Err(ArrangementError::TourOutOfRange { track: TrackId::Heel, tour: 0 })
        );
        assert_eq!(
            ReelArrangement::from_toml(&toml_with((
                "Heel = { tour = 2, part = \"A\"",
                "Heel = { tour = 2, part = \"C\""
            ))),
            Err(ArrangementError::UnknownPart { track: TrackId::Heel, part: "C".to_owned() })
        );
        assert_eq!(
            ReelArrangement::from_toml(&toml_with(("Toe = {", "Cymbale = {"))),
            Err(ArrangementError::UnknownTrack("Cymbale".to_owned()))
        );
        assert_eq!(
            ReelArrangement::from_toml(&toml_with((
                "Toe = { tour = 2, part = \"A\", pass = 1 }\n",
                ""
            ))),
            Err(ArrangementError::MissingTrack(TrackId::Toe))
        );
        assert_eq!(
            ReelArrangement::from_toml(&toml_with((
                "Toe = { tour = 2, part = \"A\", pass = 1",
                "Toe = { tour = 2, part = \"A\", pass = 3"
            ))),
            Err(ArrangementError::PassOutOfRange { track: TrackId::Toe, pass: 3 })
        );
        assert_eq!(
            ReelArrangement::from_toml(&toml_with(("tours_per_tune = 3", "tours_per_tune = 0"))),
            Err(ArrangementError::ToursPerTuneOutOfRange(0))
        );
        let msg = ArrangementError::UnknownTrack("Cymbale".to_owned()).to_string();
        assert!(msg.contains("Cymbale"));
    }

    #[test]
    fn table_roundtrips_through_serde() -> Result<(), String> {
        let table = ReelArrangement::default();
        let text = toml::to_string(&table).map_err(|e| e.to_string())?;
        assert_eq!(ReelArrangement::from_toml(&text), Ok(table));
        Ok(())
    }
}

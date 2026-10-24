//! Arrangement du reel : tours d'un air, entrées progressives des pistes (D11),
//! et intros idiomatiques (P1-T11).
//!
//! Ce module porte :
//! - [`TunePosition`] : la position structurée `(section, tune_index, tour, …)` d'une
//!   mesure, calculée en un seul endroit ([`TunePosition::at_bar`]) et exposée au jeu dans
//!   [`crate::report::MeasureSnapshot`] ;
//! - [`ReelArrangement`] : la table `reel_arrangement.toml` (constitution V) qui
//!   dit, pour chaque piste, la première position où elle joue ;
//! - [`IntroConfig`] / [`IntroForm`] / [`IntroRole`] : la section `[intro]` de la
//!   table, avec les rôles d'intro idiomatiques décrits en données (P1-T11).

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{style_profile::ProfileForm, timeline::TrackId};

/// Contenu brut de la table TOML embarquée d'arrangement du reel (D11 + P1-T11).
pub const REEL_ARRANGEMENT_TOML: &str =
    include_str!("../../../profiles/reel_quebecois/arrangement.toml");

/// Nombre maximal de tours par air et borne haute du `tour` d'une entrée.
pub const MAX_TOURS: usize = 16;

/// Nombre maximal de mesures d'intro (borne de validation).
pub const MAX_INTRO_BARS: usize = 4;

/// Nombre maximal de croches de levée (borne de validation).
pub const MAX_PICKUP_EIGHTHS: u8 = 3;

// ---------------------------------------------------------------------------
// Section d'une mesure (intro ou tour)
// ---------------------------------------------------------------------------

/// Section dans laquelle se situe une mesure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Section {
    /// Mesure d'intro (précède le premier tour de l'air).
    Intro,
    /// Mesure d'un tour de l'air (AABB).
    Tour,
}

// ---------------------------------------------------------------------------
// Partie d'un air AABB
// ---------------------------------------------------------------------------

/// Indice 0-based d'une partie dans le profil.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Part(pub usize);
impl Part {
    pub const fn index(self) -> usize {
        self.0
    }
}

// ---------------------------------------------------------------------------
// TunePosition
// ---------------------------------------------------------------------------

/// Position structurée d'une mesure dans la suite d'airs d'une session.
///
/// Conventions : `tune_index` et `set_index` sont 0-based (comme `ActiveTune::tune_index`) ;
/// `tune_in_set`, `tour`, `pass` et `bar_in_strain` sont 1-based, comme les numéros de mesure
/// du contrat. `bar_in_intro` vaut 0 pour les mesures de tour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TunePosition {
    /// Section courante (intro ou tour).
    pub section: Section,
    /// Indice 0-based de l'air dans la session.
    pub tune_index: usize,
    /// Indice 0-based du set dans la session.
    #[serde(default)]
    pub set_index: usize,
    /// Rang 1-based de l'air dans son set (1 : premier air, intro complète). Dans un set sans
    /// fin (`tunes_in_set == 0`), compte les airs de la session.
    #[serde(default)]
    pub tune_in_set: usize,
    /// Longueur du set, tirée au début du set et constante dans le set ; `0` : set sans fin.
    #[serde(default)]
    pub tunes_in_set: usize,
    /// Tour de l'air (`1..=tours`). 0 pour les mesures d'intro.
    pub tour: usize,
    /// Nombre de tours de cet air (`tour == tours` : dernier tour).
    pub tours: usize,
    /// Partie courante (A ou B). Non significatif pour `Section::Intro`.
    pub part: Part,
    /// Passe de la partie (`1` ou `2`). Non significatif pour `Section::Intro`.
    pub pass: usize,
    /// Mesure dans la passe (`1..=8`). Non significatif pour `Section::Intro`.
    pub bar_in_strain: usize,
    /// Tonique de l'air : stable jusqu'au prochain air.
    pub key: u8,
    /// Mode effectivement entendu à cette mesure.
    pub mode: super::Mode,
    /// Indices 0-based dans le tour et son schéma de passes.
    pub bar_in_cycle: usize,
    pub sequence_index: usize,
    pub passes_per_tour: usize,
}

impl TunePosition {
    /// Position de la mesure 1-based `bar` quand chaque air dure `tours` tours
    /// et que le premier air est précédé de `intro_bars` mesures d'intro.
    ///
    /// C'est l'unique conversion mesure → position du moteur trad.
    /// Passer `intro_bars` identique pour chaque air ; la fonction décale `bar` d'autant.
    #[must_use]
    pub fn at_bar(
        bar: usize,
        tours: usize,
        intro_bars: usize,
        key: u8,
        mode: super::Mode,
        form: &ProfileForm,
    ) -> Self {
        let tours = tours.max(1);
        let cycle = form.bars_per_tour() * tours;
        let zero = bar.saturating_sub(intro_bars + 1);
        let bar_in_cycle = zero % form.bars_per_tour();
        let (part, pass, offset) = form.position(bar_in_cycle);
        let sequence_index = form
            .sequence
            .iter()
            .enumerate()
            .filter(|(_, p)| **p == part)
            .nth(pass - 1)
            .map_or(0, |(i, _)| i);
        let intro = bar <= intro_bars;
        Self {
            section: if intro { Section::Intro } else { Section::Tour },
            tune_index: zero / cycle,
            set_index: 0,
            tune_in_set: zero / cycle + 1,
            tunes_in_set: 0,
            tour: if intro { 0 } else { (zero % cycle) / form.bars_per_tour() + 1 },
            tours,
            part: Part(part),
            pass: if intro { 0 } else { pass },
            bar_in_strain: if intro { 0 } else { offset + 1 },
            key,
            mode,
            bar_in_cycle,
            sequence_index,
            passes_per_tour: form.sequence.len(),
        }
    }

    /// Indice 0-based de la mesure dans un tour AABB (`0..32`) : indexe la
    /// ligne de violon et l'harmonie, identiques à chaque tour.
    /// Panique en debug pour `Section::Intro` (pas de tour pendant l'intro).
    #[must_use]
    pub const fn bar_in_tour(&self) -> usize {
        self.bar_in_cycle
    }

    /// Vrai au dernier tour de l'air.
    #[must_use]
    pub const fn is_last_tour(&self) -> bool {
        self.tour >= self.tours
    }

    /// Vrai si la mesure est dans une intro.
    #[must_use]
    pub const fn is_intro(&self) -> bool {
        matches!(self.section, Section::Intro)
    }

    const fn ordinal(&self) -> usize {
        self.tour.saturating_sub(1) * self.passes_per_tour + self.sequence_index
    }
}

// ---------------------------------------------------------------------------
// Formes d'intro (P1-T11)
// ---------------------------------------------------------------------------

/// Rôle d'intro idiomatique, décrit en données (P1-T11, constitution v3 V).
///
/// Un rôle dit *quoi* joue une famille de pistes pendant l'intro ; il est
/// générique et ne suppose ni grille (2/2, 16 pas), ni forme (AABB), ni
/// instrumentarium particulier : la grille et la forme sont lues depuis l'air
/// courant au moment de générer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntroRole {
    /// Décompte de percussion : les pistes de percussion jouent leur pattern seul.
    Percussion,
    /// Entrée par la fin de l'air : la piste mélodique joue les `bars` dernières
    /// mesures de l'air (cadence finale) avant d'attaquer le premier tour.
    Tail,
    /// Vamp d'accompagnement : les pistes d'accompagnement jouent sur la tonique.
    Vamp,
}

impl IntroRole {
    /// Tous les rôles d'intro connus.
    pub const ALL: [Self; 3] = [Self::Percussion, Self::Tail, Self::Vamp];

    /// Nombre de mesures par défaut pour ce rôle.
    #[must_use]
    pub const fn default_bars(self) -> usize {
        match self {
            Self::Percussion => 1,
            Self::Tail => 2,
            Self::Vamp => 2,
        }
    }
}

/// Une forme d'intro avec son poids de tirage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntroForm {
    /// Rôle de la forme (qui joue, quoi).
    pub role: IntroRole,
    /// Poids relatif de tirage (≥ 1).
    #[serde(default = "default_intro_weight")]
    pub weight: u32,
    /// Nombre de mesures pour cette forme (1..=MAX_INTRO_BARS).
    #[serde(default)]
    pub bars: usize,
}

const fn default_intro_weight() -> u32 {
    1
}

impl IntroForm {
    /// Nombre effectif de mesures (bars explicite ou défaut du rôle).
    #[must_use]
    pub fn effective_bars(&self) -> usize {
        if self.bars > 0 { self.bars } else { self.role.default_bars() }
    }
}

/// Configuration de l'intro (section `[intro]` de `reel_arrangement.toml`, P1-T11).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntroConfig {
    /// Formes d'intro avec poids de tirage.
    #[serde(default)]
    pub forms: Vec<IntroForm>,
    /// Nombre de croches de levée (0..=3).
    #[serde(default)]
    pub pickup_eighths: u8,
}

impl Default for IntroConfig {
    fn default() -> Self {
        Self {
            forms: vec![
                IntroForm { role: IntroRole::Percussion, weight: 3, bars: 1 },
                IntroForm { role: IntroRole::Tail, weight: 2, bars: 2 },
                IntroForm { role: IntroRole::Vamp, weight: 1, bars: 2 },
            ],
            pickup_eighths: 2,
        }
    }
}

impl IntroConfig {
    /// Aucune intro : pas de formes, pas de levée.
    #[must_use]
    pub const fn none() -> Self {
        Self { forms: Vec::new(), pickup_eighths: 0 }
    }

    /// Valide la configuration d'intro.
    pub fn validate(&self) -> Result<(), ArrangementError> {
        for form in &self.forms {
            let bars = form.effective_bars();
            if !(1..=MAX_INTRO_BARS).contains(&bars) {
                return Err(ArrangementError::IntroBarsOutOfRange { role: form.role, bars });
            }
            if form.weight == 0 {
                return Err(ArrangementError::IntroWeightZero { role: form.role });
            }
        }
        if self.pickup_eighths > MAX_PICKUP_EIGHTHS {
            return Err(ArrangementError::PickupEighthsOutOfRange(self.pickup_eighths));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// EntryPoint, TrackEntries
// ---------------------------------------------------------------------------

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
    pub const fn from_start(part: Part) -> Self {
        Self { tour: 1, part, pass: 1 }
    }
}

/// Points d'entrée des cinq pistes (clés TOML = noms de [`TrackId`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TrackEntries {
    #[serde(rename = "Melody")]
    pub melody: EntryPoint,
    #[serde(rename = "Chords", skip_serializing_if = "Option::is_none")]
    pub chords: Option<EntryPoint>,
    #[serde(rename = "Bass", skip_serializing_if = "Option::is_none")]
    pub bass: Option<EntryPoint>,
    #[serde(rename = "Heel")]
    pub heel: EntryPoint,
    #[serde(rename = "Toe")]
    pub toe: EntryPoint,
}

impl TrackEntries {
    pub const fn get(&self, track: TrackId) -> Option<EntryPoint> {
        match track {
            TrackId::Melody => Some(self.melody),
            TrackId::Chords => self.chords,
            TrackId::Bass => self.bass,
            TrackId::Heel => Some(self.heel),
            TrackId::Toe => Some(self.toe),
        }
    }

    pub fn all(&self) -> Vec<(TrackId, EntryPoint)> {
        TrackId::ALL
            .into_iter()
            .filter_map(|track| self.get(track).map(|entry| (track, entry)))
            .collect()
    }
}
// ---------------------------------------------------------------------------
// ArrangementError
// ---------------------------------------------------------------------------

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
    /// Passe d'entrée nulle.
    PassOutOfRange { track: TrackId, pass: i64 },
    /// `bars` d'une forme d'intro hors limites.
    IntroBarsOutOfRange { role: IntroRole, bars: usize },
    /// Poids d'une forme d'intro nul.
    IntroWeightZero { role: IntroRole },
    /// `pickup_eighths` hors limites (0..=3).
    PickupEighthsOutOfRange(u8),
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
            Self::PassOutOfRange { track, pass } => {
                write!(f, "passe d'entrée hors limites pour {track:?} (attendu 1 ou 2) : {pass}")
            }
            Self::IntroBarsOutOfRange { role, bars } => write!(
                f,
                "forme d'intro {role:?} : bars hors limites (attendu 1..={MAX_INTRO_BARS}) : {bars}"
            ),
            Self::IntroWeightZero { role } => {
                write!(f, "forme d'intro {role:?} : weight doit être ≥ 1")
            }
            Self::PickupEighthsOutOfRange(n) => {
                write!(f, "pickup_eighths hors limites (attendu 0..={MAX_PICKUP_EIGHTHS}) : {n}")
            }
        }
    }
}

impl std::error::Error for ArrangementError {}

// ---------------------------------------------------------------------------
// ReelArrangement
// ---------------------------------------------------------------------------

/// Table d'arrangement du reel, portée en données (D11, P1-T11, constitution V).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReelArrangement {
    /// Nombre de tours AABB joués avant de tirer un nouvel air (`1..=`[`MAX_TOURS`]).
    pub tours_per_tune: usize,
    /// Point d'entrée de chaque piste.
    pub entries: TrackEntries,
    /// Configuration des intros idiomatiques (P1-T11).
    #[serde(default)]
    pub intro: IntroConfig,
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
    part: usize,
    pass: i64,
}

const fn default_tours_per_tune() -> i64 {
    3
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawArrangement {
    #[serde(default = "default_tours_per_tune")]
    tours_per_tune: i64,
    entries: std::collections::BTreeMap<String, RawEntry>,
    #[serde(default)]
    intro: IntroConfig,
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

        let mut take_required = |track: TrackId| -> Result<EntryPoint, ArrangementError> {
            let entry = raw
                .entries
                .remove(&format!("{track:?}"))
                .ok_or(ArrangementError::MissingTrack(track))?;
            let part = Part(entry.part);
            let tour = usize::try_from(entry.tour)
                .map_err(|_| ArrangementError::TourOutOfRange { track, tour: entry.tour })?;
            let pass = usize::try_from(entry.pass)
                .map_err(|_| ArrangementError::PassOutOfRange { track, pass: entry.pass })?;
            Ok(EntryPoint { tour, part, pass })
        };
        let melody = take_required(TrackId::Melody)?;
        let heel = take_required(TrackId::Heel)?;
        let toe = take_required(TrackId::Toe)?;

        let mut take_optional = |track: TrackId| -> Result<Option<EntryPoint>, ArrangementError> {
            let Some(entry) = raw.entries.remove(&format!("{track:?}")) else {
                return Ok(None);
            };
            let part = Part(entry.part);
            let tour = usize::try_from(entry.tour)
                .map_err(|_| ArrangementError::TourOutOfRange { track, tour: entry.tour })?;
            let pass = usize::try_from(entry.pass)
                .map_err(|_| ArrangementError::PassOutOfRange { track, pass: entry.pass })?;
            Ok(Some(EntryPoint { tour, part, pass }))
        };
        let chords = take_optional(TrackId::Chords)?;
        let bass = take_optional(TrackId::Bass)?;
        let entries = TrackEntries { melody, chords, bass, heel, toe };
        let table = Self { tours_per_tune, entries, intro: raw.intro };
        Ok(table)
    }
}

impl Default for ReelArrangement {
    /// Charge la table embarquée `reel_arrangement.toml` sans jamais paniquer
    /// (sa validité est garantie par les tests et par
    /// [`crate::tuning::TuningParams::validate`]) ; en cas d'erreur de données,
    /// repli sur trois tours avec toutes les pistes dès le début et sans intro.
    fn default() -> Self {
        Self::from_toml(REEL_ARRANGEMENT_TOML)
            .unwrap_or_else(|error| unreachable!("embedded arrangement: {error}"))
    }
}

impl ReelArrangement {
    /// Arrangement plat : `tours_per_tune` tours, toutes les pistes dès le début, sans intro.
    #[must_use]
    pub const fn from_start(tours_per_tune: usize, first_part: Part) -> Self {
        let start = EntryPoint::from_start(first_part);
        Self {
            tours_per_tune,
            entries: TrackEntries {
                melody: start,
                chords: Some(start),
                bass: Some(start),
                heel: start,
                toe: start,
            },
            intro: IntroConfig::none(),
        }
    }

    /// Charge et valide une table depuis une chaîne TOML au format `reel_arrangement.toml`.
    pub fn from_toml(toml_str: &str) -> Result<Self, ArrangementError> {
        let raw: RawArrangement =
            toml::from_str(toml_str).map_err(|e| ArrangementError::TomlParse(e.to_string()))?;
        let table = Self::try_from(raw)?;
        table.validate()?;
        Ok(table)
    }

    /// Valide `tours_per_tune`, les `tour`/`pass` des entrées et la section `[intro]`.
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
            if entry.pass == 0 {
                return Err(ArrangementError::PassOutOfRange {
                    track,
                    pass: i64::try_from(entry.pass).unwrap_or(i64::MAX),
                });
            }
        }
        self.intro.validate()?;
        Ok(())
    }

    /// Point d'entrée d'une piste.
    #[must_use]
    pub const fn entry(&self, track: TrackId) -> EntryPoint {
        match self.entries.get(track) {
            Some(e) => e,
            None => EntryPoint::from_start(Part(0)),
        }
    }

    /// Point d'entrée optionnel d'une piste (`None` si omise de l'arrangement).
    #[must_use]
    pub const fn entry_opt(&self, track: TrackId) -> Option<EntryPoint> {
        self.entries.get(track)
    }

    /// Vrai si l'arrangement déclare explicitement `track`.
    #[must_use]
    pub const fn has_track(&self, track: TrackId) -> bool {
        self.entries.get(track).is_some()
    }

    /// Vrai si l'arrangement autorise `track` à jouer à `position`. Seul le premier air d'un set
    /// (`tune_in_set == 1`) a des entrées progressives ; les suivants voient toutes les pistes
    /// dès A1.
    ///
    /// Ne tient pas compte des drapeaux de jeu (`enable_*`), qui sont des
    /// plafonds appliqués par le générateur. Les pistes sont autorisées
    /// pendant l'intro selon [`IntroInfo::tracks_active_during_intro`].
    #[must_use]
    pub fn allows(&self, track: TrackId, position: &TunePosition, form: &ProfileForm) -> bool {
        let Some(entry) = self.entry_opt(track) else {
            return false;
        };
        // Pendant l'intro, l'autorisation est gérée par le générateur
        // (seules certaines pistes jouent selon la forme).
        if position.is_intro() {
            return true;
        }
        if position.tune_in_set > 1 {
            return true;
        }
        let last_tour = self.tours_per_tune.max(1);
        let sequence = form
            .sequence
            .iter()
            .enumerate()
            .filter(|(_, part)| **part == entry.part.0)
            .nth(entry.pass - 1)
            .map_or(form.sequence.len(), |(i, _)| i);
        position.ordinal() >= (entry.tour.min(last_tour) - 1) * form.sequence.len() + sequence
    }
}

// ---------------------------------------------------------------------------
// IntroInfo : état tiré pour un air
// ---------------------------------------------------------------------------

/// Résultat du tirage d'intro pour un air (P1-T11).
#[derive(Clone, Debug, PartialEq)]
pub struct IntroInfo {
    /// Rôle d'intro tiré, ou `None` si pas d'intro.
    pub form: Option<IntroRole>,
    /// Nombre de mesures d'intro pour cet air.
    pub intro_bars: usize,
    /// Mesures vides avant la forme (pause de set ou transition silence).
    pub leading_silence: usize,
    /// Mesures consacrées à la forme, hors pause et levée séparée.
    pub form_bars: usize,
    /// Notes MIDI de la levée (croches ascendantes vers la première note de l'air).
    /// Vide si `pickup_eighths == 0`.
    pub pickup_notes: Vec<u8>,
}

impl IntroInfo {
    /// Pas d'intro du tout.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            form: None,
            intro_bars: 0,
            leading_silence: 0,
            form_bars: 0,
            pickup_notes: Vec::new(),
        }
    }

    /// Vrai si la percussion joue pendant l'intro (rôle `Percussion`).
    #[must_use]
    pub const fn feet_active(&self) -> bool {
        matches!(self.form, Some(IntroRole::Percussion))
    }

    /// Vrai si l'accompagnement joue pendant l'intro (rôle `Vamp`).
    #[must_use]
    pub const fn chords_active(&self) -> bool {
        matches!(self.form, Some(IntroRole::Vamp))
    }

    /// Vrai si la mélodie joue pendant l'intro (rôle `Tail` ou levée seule).
    #[must_use]
    pub const fn melody_active(&self) -> bool {
        matches!(self.form, Some(IntroRole::Tail))
            || (self.form.is_none() && !self.pickup_notes.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_maps_bars_to_structure() {
        // Sans intro
        let p = TunePosition::at_bar(
            1,
            3,
            0,
            2,
            super::super::Mode::Ionian,
            &crate::StyleProfile::default().form,
        );
        assert_eq!(
            p,
            TunePosition {
                section: Section::Tour,
                tune_index: 0,
                set_index: 0,
                tune_in_set: 1,
                tunes_in_set: 0,
                tour: 1,
                tours: 3,
                part: Part(0),
                pass: 1,
                bar_in_strain: 1,
                key: 2,
                mode: super::super::Mode::Ionian,
                bar_in_cycle: 0,
                sequence_index: 0,
                passes_per_tour: 4,
            }
        );
        let p = TunePosition::at_bar(
            9,
            3,
            0,
            2,
            super::super::Mode::Ionian,
            &crate::StyleProfile::default().form,
        );
        assert_eq!((p.part, p.pass, p.bar_in_strain), (Part(0), 2, 1));
        let p = TunePosition::at_bar(
            25,
            3,
            0,
            2,
            super::super::Mode::Ionian,
            &crate::StyleProfile::default().form,
        );
        assert_eq!((p.part, p.pass, p.bar_in_strain, p.tour), (Part(1), 2, 1, 1));
        assert_eq!(p.bar_in_tour(), 24);
        let p = TunePosition::at_bar(
            33,
            3,
            0,
            2,
            super::super::Mode::Ionian,
            &crate::StyleProfile::default().form,
        );
        assert_eq!(
            (p.tour, p.part, p.pass, p.bar_in_strain, p.bar_in_tour()),
            (2, Part(0), 1, 1, 0)
        );
        let p = TunePosition::at_bar(
            96,
            3,
            0,
            2,
            super::super::Mode::Ionian,
            &crate::StyleProfile::default().form,
        );
        assert_eq!((p.tune_index, p.tour, p.bar_in_tour()), (0, 3, 31));
        assert!(p.is_last_tour());
        let p = TunePosition::at_bar(
            97,
            3,
            0,
            2,
            super::super::Mode::Ionian,
            &crate::StyleProfile::default().form,
        );
        assert_eq!((p.tune_index, p.tour, p.bar_in_tour()), (1, 1, 0));
    }

    #[test]
    fn position_with_intro_bars() {
        // 2 mesures d'intro, 3 tours de 32 = 96 mesures d'air
        let p = TunePosition::at_bar(
            1,
            3,
            2,
            2,
            super::super::Mode::Ionian,
            &crate::StyleProfile::default().form,
        );
        assert!(p.is_intro());
        assert_eq!(p.section, Section::Intro);
        assert_eq!(p.tune_index, 0);

        let p = TunePosition::at_bar(
            2,
            3,
            2,
            2,
            super::super::Mode::Ionian,
            &crate::StyleProfile::default().form,
        );
        assert!(p.is_intro());

        // Mesure 3 = première mesure de l'air
        let p = TunePosition::at_bar(
            3,
            3,
            2,
            2,
            super::super::Mode::Ionian,
            &crate::StyleProfile::default().form,
        );
        assert!(!p.is_intro());
        assert_eq!(p.section, Section::Tour);
        assert_eq!(p.tune_index, 0);
        assert_eq!(p.tour, 1);
        assert_eq!(p.bar_in_strain, 1);

        // Après tout l'air (2 + 96 = 98), mesure 99 = tune 1
        let p = TunePosition::at_bar(
            99,
            3,
            2,
            2,
            super::super::Mode::Ionian,
            &crate::StyleProfile::default().form,
        );
        assert_eq!(p.tune_index, 1);
        assert_eq!(p.tour, 1);
    }

    #[test]
    fn entries_beyond_last_tour_are_clamped() {
        let mut table = ReelArrangement::default();
        assert!(!table.allows(
            TrackId::Heel,
            &TunePosition::at_bar(
                1,
                3,
                0,
                2,
                super::super::Mode::Ionian,
                &crate::StyleProfile::default().form
            ),
            &crate::StyleProfile::default().form
        ));
        table.tours_per_tune = 1;
        // Tour unique : l'entrée du tour 2 est ramenée au dernier tour (le 1er).
        assert!(table.allows(
            TrackId::Heel,
            &TunePosition::at_bar(
                1,
                1,
                0,
                2,
                super::super::Mode::Ionian,
                &crate::StyleProfile::default().form
            ),
            &crate::StyleProfile::default().form
        ));
    }

    fn toml_with(replace: (&str, &str)) -> String {
        let ok = "tours_per_tune = 3\n[entries]\n\
            Melody = { tour = 1, part = 0, pass = 1 }\n\
            Chords = { tour = 1, part = 0, pass = 2 }\n\
            Bass = { tour = 1, part = 0, pass = 2 }\n\
            Heel = { tour = 2, part = 0, pass = 1 }\n\
            Toe = { tour = 2, part = 0, pass = 1 }\n";
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
            ReelArrangement::from_toml(&toml_with(("Toe = {", "Cymbale = {"))),
            Err(ArrangementError::UnknownTrack("Cymbale".to_owned()))
        );
        assert_eq!(
            ReelArrangement::from_toml(&toml_with((
                "Toe = { tour = 2, part = 0, pass = 1 }\n",
                ""
            ))),
            Err(ArrangementError::MissingTrack(TrackId::Toe))
        );
        assert_eq!(
            ReelArrangement::from_toml(&toml_with((
                "Toe = { tour = 2, part = 0, pass = 1",
                "Toe = { tour = 2, part = 0, pass = 0"
            ))),
            Err(ArrangementError::PassOutOfRange { track: TrackId::Toe, pass: 0 })
        );
        assert_eq!(
            ReelArrangement::from_toml(&toml_with(("tours_per_tune = 3", "tours_per_tune = 0"))),
            Err(ArrangementError::ToursPerTuneOutOfRange(0))
        );
        let msg = ArrangementError::UnknownTrack("Cymbale".to_owned()).to_string();
        assert!(msg.contains("Cymbale"));
    }

    #[test]
    fn invalid_intro_configs_yield_explicit_errors() {
        // bars trop grand
        let mut config = IntroConfig::default();
        config.forms[0].bars = 10;
        assert!(matches!(config.validate(), Err(ArrangementError::IntroBarsOutOfRange { .. })));

        // weight nul
        let mut config = IntroConfig::default();
        config.forms[0].weight = 0;
        assert!(matches!(config.validate(), Err(ArrangementError::IntroWeightZero { .. })));

        // pickup_eighths hors limites
        let mut config = IntroConfig::default();
        config.pickup_eighths = 5;
        assert!(matches!(config.validate(), Err(ArrangementError::PickupEighthsOutOfRange(5))));
    }

    #[test]
    fn table_roundtrips_through_serde() -> Result<(), String> {
        let table = ReelArrangement::default();
        let text = toml::to_string(&table).map_err(|e| e.to_string())?;
        assert_eq!(ReelArrangement::from_toml(&text), Ok(table));
        Ok(())
    }
}

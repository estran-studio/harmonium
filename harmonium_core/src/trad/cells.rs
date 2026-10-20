//! Cellules mélodiques d'une mesure (8 croches) et banque TOML embarquée (D5).
//!
//! Une cellule représente une mesure de reel en 2/2 découpée en 8 croches
//! diatoniques (`CellNote::Degree(i8)` ou `CellNote::Hold`).
//! La banque par défaut est embarquée via `include_str!("reel_cells.toml")` et
//! portée par [`crate::tuning::TuningParams`] conformément au principe V de la
//! constitution.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::mode::{Mode, degree_to_diatonic_step};

/// Nombre exact de croches dans une cellule de reel (D5).
pub const NOTES_PER_CELL: usize = 8;

/// Ambitus maximal autorisé dans une cellule en pas diatoniques (7 pas = 1 octave).
pub const MAX_AMBITUS_STEPS: i8 = 7;

/// Ambitus maximal autorisé dans une cellule en demi-tons (12 demi-tons = 1 octave).
pub const MAX_AMBITUS_SEMITONES: i8 = 12;

/// Saut diatonique maximal considéré comme ≤ quinte (4 pas diatoniques = quinte).
pub const FIFTH_DIATONIC_STEPS: i8 = 4;

/// Saut en demi-tons maximal considéré comme ≤ quinte juste (7 demi-tons).
pub const FIFTH_SEMITONES: i8 = 7;

/// Nombre maximal de sauts > quinte autorisés dans une même cellule (D5).
pub const MAX_LARGE_LEAPS_PER_CELL: usize = 1;

/// Poids par défaut d'une cellule dont le TOML ne porte pas de champ `weight`.
pub const DEFAULT_CELL_WEIGHT: u32 = 1;

/// Contenu brut de la banque TOML embarquée du reel.
pub const REEL_CELLS_TOML: &str = include_str!("reel_cells.toml");

/// Rôle fonctionnel d'une cellule mélodique dans une strain de 8 mesures (D5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellRole {
    /// Début de phrase (mesure 1 de la strain).
    Open,
    /// Mouvement majoritairement conjoint.
    Run,
    /// Arpège d'accord.
    Arp,
    /// Broderie autour d'un degré pivot.
    Turn,
    /// Demi-cadence (mesure 4, finit sur un degré de dominante : 2, 5 ou 7).
    CadHalf,
    /// Cadence finale (mesure 8, finit sur la tonique).
    CadFull,
}

impl CellRole {
    /// Tous les rôles de cellules en ordre canonique.
    pub const ALL: [Self; 6] =
        [Self::Open, Self::Run, Self::Arp, Self::Turn, Self::CadHalf, Self::CadFull];
}

/// Une croche au sein d'une [`Cell`] de 8 croches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CellNote {
    /// Degré diatonique relatif à la tonique du mode :
    /// - `1..=7` : degrés 1 à 7 de l'octave principale (`1` = tonique) ;
    /// - `8..=14` : octave supérieure (`8` = tonique + 1 octave) ;
    /// - `-7..=-1` : octave inférieure notée `k,` (`k - 8` : `7,` = `-1`, `5,` = `-3`, `1,` = `-7`).
    Degree(i8),
    /// Prolongation (`-`) de la note précédente.
    Hold,
}

impl CellNote {
    /// Parse un jeton individuel de la notation D5 (`"1"`, `"8"`, `"5,"`, `"-"`).
    pub fn parse_token(token: &str, position: usize) -> Result<Self, CellError> {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            return Err(CellError::InvalidToken { token: token.to_string(), position });
        }
        if trimmed == "-" {
            return Ok(Self::Hold);
        }
        if let Some(stem) = trimmed.strip_suffix(',') {
            // Octave inférieure : seul un degré 1..=7 suivi d'une seule virgule est valide.
            let k = stem
                .parse::<i8>()
                .map_err(|_| CellError::InvalidToken { token: trimmed.to_string(), position })?;
            if !(1..=7).contains(&k) {
                return Err(CellError::InvalidToken { token: trimmed.to_string(), position });
            }
            return Ok(Self::Degree(k - 8));
        }

        let deg = trimmed
            .parse::<i8>()
            .map_err(|_| CellError::InvalidToken { token: trimmed.to_string(), position })?;
        if !(1..=14).contains(&deg) {
            return Err(CellError::InvalidToken { token: trimmed.to_string(), position });
        }
        Ok(Self::Degree(deg))
    }

    /// Formate la note dans la notation textuelle D5 (`"1"`, `"5,"`, `"-"`).
    #[must_use]
    pub fn to_notation(self) -> String {
        match self {
            Self::Hold => "-".to_string(),
            Self::Degree(d) if d < 0 => format!("{},", d + 8),
            Self::Degree(d) => d.to_string(),
        }
    }
}

/// Erreur de parsing ou de validation d'une [`Cell`] selon les règles de D5.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellError {
    /// Le nombre de croches diffère de 8.
    WrongNoteCount { expected: usize, actual: usize },
    /// Un jeton de note est syntaxiquement invalide (degré hors bornes, chromatisme, etc.).
    InvalidToken { token: String, position: usize },
    /// La première croche de la cellule est `Hold` (`"-"`).
    StartsWithHold,
    /// Un degré construit directement en Rust est hors de la plage valide (`-7..=-1` ou `1..=14`).
    DegreeOutOfRange { degree: i8, position: usize },
    /// L'ambitus total de la cellule dépasse une octave (7 pas diatoniques / 12 demi-tons).
    AmbitusExceedsOctave { min_degree: i8, max_degree: i8, semitones: i8 },
    /// La cellule comporte plus d'un saut mélodique strictement supérieur à la quinte.
    TooManyLargeLeaps { count: usize },
    /// Le poids de tirage de la cellule est nul (il doit être > 0).
    ZeroWeight,
}

impl fmt::Display for CellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongNoteCount { expected, actual } => {
                write!(f, "attendu {expected} croches par cellule, trouvé {actual}")
            }
            Self::InvalidToken { token, position } => {
                write!(f, "jeton de note invalide à la position {position} : \"{token}\"")
            }
            Self::StartsWithHold => {
                write!(f, "la première croche d'une cellule ne peut pas être Hold (\"-\")")
            }
            Self::DegreeOutOfRange { degree, position } => {
                write!(f, "degré {degree} invalide à la position {position}")
            }
            Self::AmbitusExceedsOctave { min_degree, max_degree, semitones } => {
                write!(
                    f,
                    "ambitus de {semitones} demi-tons (de {min_degree} à {max_degree}) > 1 octave"
                )
            }
            Self::TooManyLargeLeaps { count } => {
                write!(f, "{count} sauts > quinte dans la cellule (maximum autorisé : 1)")
            }
            Self::ZeroWeight => write!(f, "le poids (`weight`) de la cellule doit être > 0"),
        }
    }
}

impl std::error::Error for CellError {}

/// Erreur lors du chargement ou de la validation d'une [`CellBank`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellBankError {
    /// Erreur de syntaxe TOML.
    TomlParse(String),
    /// La banque est vide.
    EmptyBank,
    /// Une cellule à l'indice donné viole les règles de D5.
    InvalidCell { index: usize, error: CellError },
}

impl fmt::Display for CellBankError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TomlParse(msg) => write!(f, "erreur de lecture TOML de la banque : {msg}"),
            Self::EmptyBank => write!(f, "la banque de cellules est vide"),
            Self::InvalidCell { index, error } => {
                write!(f, "cellule #{index} invalide : {error}")
            }
        }
    }
}

impl std::error::Error for CellBankError {}

/// Cellule mélodique d'une mesure de reel (8 croches, D5).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Cell {
    /// Rôle fonctionnel de la cellule dans la strain.
    pub role: CellRole,
    /// Les 8 croches de la mesure.
    pub notes: [CellNote; NOTES_PER_CELL],
    /// Poids de tirage dans le rôle (fréquence au corpus pour les cellules minées, > 0).
    pub weight: u32,
}

impl Cell {
    /// Parse la chaîne de 8 croches (ex. `"1 3 5 3 1 3 5 3"`, `"5 4 3 2 1 - 5, -"`)
    /// sans valider les contraintes d'ambitus/sauts.
    pub fn parse_notes(notes_str: &str) -> Result<[CellNote; NOTES_PER_CELL], CellError> {
        let tokens: Vec<&str> = notes_str.split_whitespace().collect();
        if tokens.len() != NOTES_PER_CELL {
            return Err(CellError::WrongNoteCount {
                expected: NOTES_PER_CELL,
                actual: tokens.len(),
            });
        }

        let mut notes = [CellNote::Hold; NOTES_PER_CELL];
        for (i, tok) in tokens.iter().enumerate() {
            notes[i] = CellNote::parse_token(tok, i)?;
        }
        Ok(notes)
    }

    /// Construit et valide une [`Cell`] à partir de son rôle et de la notation textuelle D5.
    pub fn from_notation(role: CellRole, notes_str: &str) -> Result<Self, CellError> {
        Self::from_notation_weighted(role, notes_str, DEFAULT_CELL_WEIGHT)
    }

    /// Comme [`Cell::from_notation`] avec un poids de tirage explicite (> 0).
    pub fn from_notation_weighted(
        role: CellRole,
        notes_str: &str,
        weight: u32,
    ) -> Result<Self, CellError> {
        let notes = Self::parse_notes(notes_str)?;
        let cell = Self { role, notes, weight };
        cell.validate()?;
        Ok(cell)
    }

    /// Formate les 8 croches dans la notation textuelle D5 (ex. `"1 3 5 3 1 3 5 3"`).
    #[must_use]
    pub fn notes_to_notation(&self) -> String {
        let mut out = String::with_capacity(24);
        for (i, note) in self.notes.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            out.push_str(&note.to_notation());
        }
        out
    }

    /// Premier degré attaqué de la cellule (toujours présent sur une cellule valide).
    #[must_use]
    pub const fn first_degree(&self) -> Option<i8> {
        match self.notes[0] {
            CellNote::Degree(d) => Some(d),
            CellNote::Hold => None,
        }
    }

    /// Dernier degré attaqué de la cellule (en ignorant les `Hold` finaux).
    #[must_use]
    pub fn last_degree(&self) -> Option<i8> {
        for note in self.notes.iter().rev() {
            if let CellNote::Degree(d) = *note {
                return Some(d);
            }
        }
        None
    }

    /// Valide les règles D5 sur la cellule :
    /// 1. la première croche n'est pas `Hold` ;
    /// 2. tous les degrés sont dans `-7..=-1` ou `1..=14` ;
    /// 3. ambitus ≤ une octave (7 pas diatoniques / 12 demi-tons) ;
    /// 4. au plus un saut mélodique > quinte (> 4 pas diatoniques / > 7 demi-tons) ;
    /// 5. poids de tirage > 0.
    pub fn validate(&self) -> Result<(), CellError> {
        if self.weight == 0 {
            return Err(CellError::ZeroWeight);
        }
        if self.notes[0] == CellNote::Hold {
            return Err(CellError::StartsWithHold);
        }

        let mut min_step = i8::MAX;
        let mut max_step = i8::MIN;
        let mut min_deg = 0i8;
        let mut max_deg = 0i8;
        let mut prev_step: Option<i8> = None;
        let mut prev_semi: Option<i8> = None;
        let mut large_leaps = 0usize;

        for (i, &note) in self.notes.iter().enumerate() {
            if let CellNote::Degree(d) = note {
                if d == 0 || !(-7..=14).contains(&d) {
                    return Err(CellError::DegreeOutOfRange { degree: d, position: i });
                }
                let step = degree_to_diatonic_step(d);
                let semi = Mode::Ionian.degree_to_semitones(d);

                if step < min_step {
                    min_step = step;
                    min_deg = d;
                }
                if step > max_step {
                    max_step = step;
                    max_deg = d;
                }

                if let (Some(ps), Some(psemi)) = (prev_step, prev_semi) {
                    let step_diff = (i16::from(step) - i16::from(ps)).abs();
                    let semi_diff = (i16::from(semi) - i16::from(psemi)).abs();
                    if step_diff > i16::from(FIFTH_DIATONIC_STEPS)
                        || semi_diff > i16::from(FIFTH_SEMITONES)
                    {
                        large_leaps += 1;
                    }
                }
                prev_step = Some(step);
                prev_semi = Some(semi);
            }
        }

        let ambitus_steps = max_step.saturating_sub(min_step);
        let ambitus_semis = Mode::Ionian
            .degree_to_semitones(max_deg)
            .saturating_sub(Mode::Ionian.degree_to_semitones(min_deg));

        if ambitus_steps > MAX_AMBITUS_STEPS || ambitus_semis > MAX_AMBITUS_SEMITONES {
            return Err(CellError::AmbitusExceedsOctave {
                min_degree: min_deg,
                max_degree: max_deg,
                semitones: ambitus_semis,
            });
        }

        if large_leaps > MAX_LARGE_LEAPS_PER_CELL {
            return Err(CellError::TooManyLargeLeaps { count: large_leaps });
        }

        Ok(())
    }
}

/// Représentation sérialisée compacte d'une [`Cell`]
/// (`{ role = "arp", notes = "1 3 5 3 1 3 5 3", weight = 12 }`, `weight` optionnel).
#[derive(Serialize, Deserialize)]
struct RawCell {
    role: CellRole,
    notes: String,
    #[serde(default = "default_cell_weight")]
    weight: u32,
}

const fn default_cell_weight() -> u32 {
    DEFAULT_CELL_WEIGHT
}

impl Serialize for Cell {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let raw = RawCell { role: self.role, notes: self.notes_to_notation(), weight: self.weight };
        raw.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Cell {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawCell::deserialize(deserializer)?;
        Self::from_notation_weighted(raw.role, &raw.notes, raw.weight)
            .map_err(serde::de::Error::custom)
    }
}

/// Format racine du fichier `reel_cells.toml`.
#[derive(Deserialize)]
struct RawCellBank {
    cells: Vec<RawCell>,
}

/// Banque de cellules mélodiques d'un style traditionnel (D5, constitution V).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellBank {
    /// Liste ordonnée des cellules de la banque.
    pub cells: Vec<Cell>,
}

impl Default for CellBank {
    /// Charge la banque embarquée `reel_cells.toml` sans jamais paniquer à
    /// l'exécution (en cas d'erreur de données, retourne une banque vide ; la
    /// validité de `REEL_CELLS_TOML` est garantie par les tests unitaires et
    /// [`crate::tuning::TuningParams::validate`]).
    fn default() -> Self {
        Self::from_toml(REEL_CELLS_TOML).unwrap_or_else(|_| Self { cells: Vec::new() })
    }
}

impl CellBank {
    /// Charge et valide une [`CellBank`] depuis une chaîne TOML au format `reel_cells.toml`.
    pub fn from_toml(toml_str: &str) -> Result<Self, CellBankError> {
        let raw: RawCellBank =
            toml::from_str(toml_str).map_err(|e| CellBankError::TomlParse(e.to_string()))?;

        if raw.cells.is_empty() {
            return Err(CellBankError::EmptyBank);
        }

        let mut cells = Vec::with_capacity(raw.cells.len());
        for (index, item) in raw.cells.into_iter().enumerate() {
            let cell = Cell::from_notation_weighted(item.role, &item.notes, item.weight)
                .map_err(|error| CellBankError::InvalidCell { index, error })?;
            cells.push(cell);
        }

        Ok(Self { cells })
    }

    /// Valide toutes les cellules de la banque.
    pub fn validate(&self) -> Result<(), CellBankError> {
        if self.cells.is_empty() {
            return Err(CellBankError::EmptyBank);
        }
        for (index, cell) in self.cells.iter().enumerate() {
            cell.validate().map_err(|error| CellBankError::InvalidCell { index, error })?;
        }
        Ok(())
    }

    /// Itère sur les cellules ayant le [`CellRole`] demandé.
    pub fn by_role(&self, role: CellRole) -> impl Iterator<Item = &Cell> {
        self.cells.iter().filter(move |c| c.role == role)
    }

    /// Nombre de cellules pour un [`CellRole`] donné.
    #[must_use]
    pub fn count_by_role(&self, role: CellRole) -> usize {
        self.by_role(role).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_valid_seed_notations() -> Result<(), CellError> {
        let arp = Cell::from_notation(CellRole::Arp, "1 3 5 3 1 3 5 3")?;
        assert_eq!(arp.role, CellRole::Arp);
        assert_eq!(
            arp.notes,
            [
                CellNote::Degree(1),
                CellNote::Degree(3),
                CellNote::Degree(5),
                CellNote::Degree(3),
                CellNote::Degree(1),
                CellNote::Degree(3),
                CellNote::Degree(5),
                CellNote::Degree(3),
            ]
        );
        assert_eq!(arp.notes_to_notation(), "1 3 5 3 1 3 5 3");

        let lower = Cell::from_notation(CellRole::Open, "1 1 5, 1 3 1 5, 1")?;
        assert_eq!(lower.notes[2], CellNote::Degree(-3));
        assert_eq!(lower.notes_to_notation(), "1 1 5, 1 3 1 5, 1");

        let cad = Cell::from_notation(CellRole::CadFull, "5 4 3 2 1 - 5, -")?;
        assert_eq!(cad.notes[5], CellNote::Hold);
        assert_eq!(cad.notes[6], CellNote::Degree(-3));
        assert_eq!(cad.notes[7], CellNote::Hold);
        assert_eq!(cad.first_degree(), Some(5));
        assert_eq!(cad.last_degree(), Some(-3));
        Ok(())
    }

    #[test]
    fn parser_rejects_wrong_note_count() {
        assert_eq!(
            Cell::from_notation(CellRole::Run, "1 2 3 4"),
            Err(CellError::WrongNoteCount { expected: 8, actual: 4 })
        );
        assert_eq!(
            Cell::from_notation(CellRole::Run, "1 2 3 4 5 6 7 8 1"),
            Err(CellError::WrongNoteCount { expected: 8, actual: 9 })
        );
    }

    #[test]
    fn parser_rejects_invalid_tokens_and_chromatics() {
        assert_eq!(
            Cell::from_notation(CellRole::Run, "1 2 3# 4 5 6 7 8"),
            Err(CellError::InvalidToken { token: "3#".to_string(), position: 2 })
        );
        assert_eq!(
            Cell::from_notation(CellRole::Run, "0 2 3 4 5 6 7 8"),
            Err(CellError::InvalidToken { token: "0".to_string(), position: 0 })
        );
        assert_eq!(
            Cell::from_notation(CellRole::Run, "1 2 8, 4 5 6 7 8"),
            Err(CellError::InvalidToken { token: "8,".to_string(), position: 2 })
        );
    }

    #[test]
    fn validation_rejects_leading_hold() {
        assert_eq!(
            Cell::from_notation(CellRole::CadHalf, "- 3 2 1 2 - - -"),
            Err(CellError::StartsWithHold)
        );
    }

    #[test]
    fn validation_rejects_ambitus_over_octave() {
        // De 7, (-1) à 8 (+12) = 13 demi-tons > 1 octave
        let res = Cell::from_notation(CellRole::Run, "7, 1 2 3 4 5 6 8");
        assert!(matches!(res, Err(CellError::AmbitusExceedsOctave { .. })), "got {res:?}");
    }

    #[test]
    fn validation_allows_one_leap_over_fifth_and_rejects_two() {
        // Un seul saut > quinte (1 -> 6 = sixte) : valide
        assert!(Cell::from_notation(CellRole::Arp, "1 6 5 4 3 2 1 2").is_ok());

        // Deux sauts > quinte (1 -> 6 puis 6 -> 1) : refusé
        assert_eq!(
            Cell::from_notation(CellRole::Arp, "1 6 1 2 3 4 5 3"),
            Err(CellError::TooManyLargeLeaps { count: 2 })
        );
    }

    #[test]
    fn embedded_reel_cell_bank_loads_and_validates() -> Result<(), CellBankError> {
        let bank = CellBank::from_toml(REEL_CELLS_TOML)?;
        bank.validate()?;

        assert!(
            bank.cells.len() >= 30,
            "attendu ~30 cellules dans reel_cells.toml, trouvé {}",
            bank.cells.len()
        );

        for role in CellRole::ALL {
            let count = bank.count_by_role(role);
            assert!(count >= 3, "le rôle {role:?} doit avoir au moins 3 cellules, trouvé {count}");
        }
        Ok(())
    }
}

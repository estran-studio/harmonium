//! Forme longue : un set est une suite d'airs d'un même profil (même mètre, même forme).
//!
//! Ce module porte le schéma `[set]` du profil (`set.toml`) : longueur des sets et
//! facteurs de classe de transition de tonalité. Les algorithmes (classes, résolution
//! de la tonalité du prochain air) sont génériques ; aucun nom de style ici.
use serde::{Deserialize, Serialize};

use super::mode::Mode;
use crate::{style_profile::ProfileError, theory::PitchClass};

/// Longueur maximale d'un set, en airs.
pub const MAX_SET_TUNES: usize = 16;
/// Nombre maximal d'entrées de `set.length`.
pub const MAX_SET_LENGTH_ENTRIES: usize = 64;

/// Une longueur de set possible et son poids de tirage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetLength {
    /// Nombre d'airs du set, `1..=16`.
    pub tunes: usize,
    /// Poids relatif de tirage.
    pub weight: u32,
}

const fn one() -> f32 {
    1.0
}

/// Facteurs multipliant le poids de base de chaque tonalité candidate selon sa
/// classe de transition depuis l'air précédent. `0` interdit une classe, `1` la neutralise.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetKey {
    /// Même tonique, même mode.
    #[serde(default = "one")]
    pub same: f32,
    /// Même tonique, autre mode.
    #[serde(default = "one")]
    pub parallel: f32,
    /// Même collection diatonique, autre tonique.
    #[serde(default = "one")]
    pub relative: f32,
    /// Tonique +7 demi-tons.
    #[serde(default = "one")]
    pub fifth_up: f32,
    /// Tonique +5 demi-tons.
    #[serde(default = "one")]
    pub fourth_up: f32,
    /// Tonique +2 demi-tons.
    #[serde(default = "one")]
    pub step_up: f32,
    /// Tonique −2 demi-tons.
    #[serde(default = "one")]
    pub step_down: f32,
    /// Toute autre transition.
    #[serde(default = "one")]
    pub other: f32,
    /// Facteur des modes sombres (mineurs) au dernier air d'un set fini.
    #[serde(default = "one")]
    pub last_dark: f32,
}

impl Default for SetKey {
    fn default() -> Self {
        Self {
            same: 1.0,
            parallel: 1.0,
            relative: 1.0,
            fifth_up: 1.0,
            fourth_up: 1.0,
            step_up: 1.0,
            step_down: 1.0,
            other: 1.0,
            last_dark: 1.0,
        }
    }
}

/// Classe d'une transition de tonalité, dans l'ordre de priorité de la classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyClass {
    Same,
    Parallel,
    Relative,
    FifthUp,
    FourthUp,
    StepUp,
    StepDown,
    Other,
}

impl KeyClass {
    /// Tonique de la collection diatonique majeure qui contient `mode` sur `key`.
    const fn collection(key: PitchClass, mode: Mode) -> PitchClass {
        let offset = match mode {
            Mode::Ionian => 0,
            Mode::Dorian => 2,
            Mode::Mixolydian => 7,
            Mode::Aeolian => 9,
        };
        (key % 12 + 12 - offset) % 12
    }

    /// Classe de la transition `prev → next`.
    #[must_use]
    pub const fn of(prev: (PitchClass, Mode), next: (PitchClass, Mode)) -> Self {
        let (prev_key, prev_mode) = (prev.0 % 12, prev.1);
        let (next_key, next_mode) = (next.0 % 12, next.1);
        if prev_key == next_key {
            return if prev_mode as u8 == next_mode as u8 { Self::Same } else { Self::Parallel };
        }
        if Self::collection(prev_key, prev_mode) == Self::collection(next_key, next_mode) {
            return Self::Relative;
        }
        match (next_key + 12 - prev_key) % 12 {
            7 => Self::FifthUp,
            5 => Self::FourthUp,
            2 => Self::StepUp,
            10 => Self::StepDown,
            _ => Self::Other,
        }
    }
}

impl SetKey {
    /// Facteur de `class`.
    #[must_use]
    pub const fn factor(&self, class: KeyClass) -> f32 {
        match class {
            KeyClass::Same => self.same,
            KeyClass::Parallel => self.parallel,
            KeyClass::Relative => self.relative,
            KeyClass::FifthUp => self.fifth_up,
            KeyClass::FourthUp => self.fourth_up,
            KeyClass::StepUp => self.step_up,
            KeyClass::StepDown => self.step_down,
            KeyClass::Other => self.other,
        }
    }

    const fn class_factors(&self) -> [f32; 8] {
        [
            self.same,
            self.parallel,
            self.relative,
            self.fifth_up,
            self.fourth_up,
            self.step_up,
            self.step_down,
            self.other,
        ]
    }
}

/// Section `[set]` du profil (`set.toml`). Section absente : un set sans fin, tous
/// les facteurs à 1 ; chaque air se tire alors indépendamment du précédent.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileSet {
    /// Nombre d'airs d'un set : distribution pondérée. Vide : set sans fin.
    #[serde(default)]
    pub length: Vec<SetLength>,
    /// Facteurs de classe de transition de tonalité.
    #[serde(default)]
    pub key: SetKey,
}

impl ProfileSet {
    /// Vrai si chaque air est son propre set : l'air est annoncé en entier à chaque fois.
    #[must_use]
    pub fn is_one_tune_per_set(&self) -> bool {
        !self.length.is_empty()
            && self.length.iter().filter(|entry| entry.weight > 0).all(|entry| entry.tunes == 1)
    }

    /// Validation avec le chemin du champ fautif (`set.key.same`, `set.length[2].weight`…).
    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.length.len() > MAX_SET_LENGTH_ENTRIES {
            return Err(ProfileError::new(
                "set.length",
                format!("au plus {MAX_SET_LENGTH_ENTRIES} entrées, reçu {}", self.length.len()),
            ));
        }
        let mut total = 0u64;
        for (index, entry) in self.length.iter().enumerate() {
            if !(1..=MAX_SET_TUNES).contains(&entry.tunes) {
                return Err(ProfileError::new(
                    format!("set.length[{index}].tunes"),
                    format!("attendu 1..={MAX_SET_TUNES}, reçu {}", entry.tunes),
                ));
            }
            total += u64::from(entry.weight);
        }
        if !self.length.is_empty() && !(1..=u64::from(u32::MAX)).contains(&total) {
            return Err(ProfileError::new(
                "set.length",
                "somme des poids positive dans u32 requise",
            ));
        }
        for (name, value) in [
            ("same", self.key.same),
            ("parallel", self.key.parallel),
            ("relative", self.key.relative),
            ("fifth_up", self.key.fifth_up),
            ("fourth_up", self.key.fourth_up),
            ("step_up", self.key.step_up),
            ("step_down", self.key.step_down),
            ("other", self.key.other),
            ("last_dark", self.key.last_dark),
        ] {
            if !value.is_finite() || !(0.0..=1000.0).contains(&value) {
                return Err(ProfileError::new(
                    format!("set.key.{name}"),
                    format!("facteur fini attendu dans 0..=1000, reçu {value}"),
                ));
            }
        }
        if self.key.class_factors().iter().all(|&factor| factor == 0.0) {
            return Err(ProfileError::new(
                "set.key",
                "au moins un facteur de classe positif requis",
            ));
        }
        Ok(())
    }
}

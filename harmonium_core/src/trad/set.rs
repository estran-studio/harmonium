//! Forme longue : un set est une suite d'airs d'un même profil (même mètre, même forme).
//!
//! Ce module porte le schéma `[set]` du profil (`set.toml`) : longueur des sets et
//! facteurs de classe de transition de tonalité. Les algorithmes (classes, résolution
//! de la tonalité du prochain air) sont génériques ; aucun nom de style ici.
use serde::{Deserialize, Serialize};

use super::mode::Mode;
use crate::{
    algo::RngCore,
    style_profile::{ProfileError, StyleProfile},
    theory::PitchClass,
};

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

/// Mécanisme de raccord entre deux airs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionKind {
    #[default]
    Direct,
    Call,
    Held,
    Silence,
}

/// Transition pondérée ; `bars` est réservé à l'appel et au silence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetTransition {
    pub kind: TransitionKind,
    pub weight: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bars: Option<usize>,
}

/// Pause pondérée après un set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetGap {
    pub bars: usize,
    pub weight: u32,
}

/// Clôture d'un set fini.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetEnd {
    #[serde(default)]
    pub outro: bool,
    #[serde(default)]
    pub gap_bars: Vec<SetGap>,
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
    /// Raccords des airs 2..L ; vide : levée seule.
    #[serde(default)]
    pub transitions: Vec<SetTransition>,
    /// Cadence et pause de fin de set.
    #[serde(default)]
    pub end: SetEnd,
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
        let max_bars = super::arrangement::MAX_INTRO_BARS;
        validate_weights("set.transitions", self.transitions.iter().map(|t| t.weight))?;
        for (index, transition) in self.transitions.iter().enumerate() {
            let valid = match transition.kind {
                TransitionKind::Call | TransitionKind::Silence => {
                    transition.bars.is_some_and(|bars| (1..=max_bars).contains(&bars))
                }
                TransitionKind::Direct | TransitionKind::Held => transition.bars.is_none(),
            };
            if !valid {
                return Err(ProfileError::new(
                    format!("set.transitions[{index}].bars"),
                    "1..=MAX_INTRO_BARS pour call/silence ; absent pour direct/held",
                ));
            }
        }
        validate_weights("set.end.gap_bars", self.end.gap_bars.iter().map(|g| g.weight))?;
        for (index, gap) in self.end.gap_bars.iter().enumerate() {
            if gap.bars > max_bars {
                return Err(ProfileError::new(
                    format!("set.end.gap_bars[{index}].bars"),
                    "attendu 0..=MAX_INTRO_BARS",
                ));
            }
        }
        Ok(())
    }

    /// Résout le raccord sans tirer de nouvel aléa.
    #[must_use]
    pub fn transition(&self, u: u32) -> (TransitionKind, usize) {
        weighted_index(self.transitions.iter().map(|t| t.weight), u)
            .map_or((TransitionKind::Direct, 0), |index| {
            let t = self.transitions[index];
            (t.kind, t.bars.unwrap_or(0))
        })
    }

    /// Résout la pause de fin de set.
    #[must_use]
    pub fn gap_bars(&self, u: u32) -> usize {
        weighted_index(self.end.gap_bars.iter().map(|g| g.weight), u)
            .map_or(0, |index| self.end.gap_bars[index].bars)
    }
}

fn weighted_index(weights: impl Iterator<Item = u32> + Clone, u: u32) -> Option<usize> {
    let total: u64 = weights.clone().map(u64::from).sum();
    let mut remaining = u64::from(u) * total / KEY_DRAW_RANGE as u64;
    weights.enumerate().find_map(|(index, weight)| {
        let hit = remaining < u64::from(weight);
        remaining = remaining.saturating_sub(u64::from(weight));
        hit.then_some(index)
    })
}

fn validate_weights(field: &str, weights: impl Iterator<Item = u32>) -> Result<(), ProfileError> {
    let (count, total) = weights.fold((0usize, 0u64), |(n, sum), w| (n + 1, sum + u64::from(w)));
    if count > MAX_SET_LENGTH_ENTRIES || (count > 0 && !(1..=u64::from(u32::MAX)).contains(&total)) {
        return Err(ProfileError::new(field, "au plus 64 entrées, somme positive dans u32"));
    }
    Ok(())
}

/// Résolution d'un tirage uniforme `0..UNIT` en tonalité (même précision partout).
pub const KEY_DRAW_RANGE: usize = 1_000_000;

/// Tonalité candidate d'un air, avec son poids de base.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyCandidate {
    pub key: PitchClass,
    pub mode: Mode,
    pub weight: f64,
}

/// Candidats de tonalité à `valence` : `melody.key_modes` à `neutral_valence`, `[[color]]`
/// évalué à la valence ailleurs. Seule source des candidats : le jeu garde la couleur.
#[must_use]
pub fn candidates(profile: &StyleProfile, valence: f32) -> Vec<KeyCandidate> {
    let dynamics = &profile.dynamics;
    if valence == dynamics.neutral_valence {
        profile
            .melody
            .key_modes
            .iter()
            .map(|c| KeyCandidate { key: c.key, mode: c.mode, weight: f64::from(c.weight) })
            .collect()
    } else {
        let input = dynamics.valence_input(valence);
        dynamics
            .color
            .iter()
            .map(|c| KeyCandidate {
                key: c.key,
                mode: c.mode,
                weight: f64::from(c.weight.eval(input)).max(0.0),
            })
            .collect()
    }
}

/// Inverse de la répartition : indice du candidat que `u` (`0..KEY_DRAW_RANGE`) désigne,
/// jamais un poids nul. `None` si la somme des poids n'est pas positive.
fn pick(weights: &[f64], u: u32) -> Option<usize> {
    let total: f64 = weights.iter().sum();
    if total.is_nan() || total <= 0.0 {
        return None;
    }
    let mut remaining = f64::from(u) / KEY_DRAW_RANGE as f64 * total;
    for (index, &weight) in weights.iter().enumerate() {
        if remaining < weight {
            return Some(index);
        }
        remaining -= weight;
    }
    weights.iter().rposition(|&weight| weight > 0.0)
}

impl ProfileSet {
    /// Tonalité d'un air parmi `candidates` pour le tirage `u`.
    ///
    /// Sans `prev` (premier air d'un set) : poids de base. Sinon poids de base × facteur de
    /// la classe de `prev → candidat`, × `last_dark` pour un mode sombre si `last`. Si la
    /// somme s'annule, on retire d'abord les facteurs de classe, puis `last_dark` : le jeu
    /// ne se retrouve jamais sans tonalité.
    #[must_use]
    pub fn choose(
        &self,
        prev: Option<(PitchClass, Mode)>,
        candidates: &[KeyCandidate],
        last: bool,
        u: u32,
    ) -> (PitchClass, Mode) {
        let dark = |c: &KeyCandidate| {
            if last && c.mode.is_minor() { f64::from(self.key.last_dark) } else { 1.0 }
        };
        let class = |c: &KeyCandidate| {
            prev.map_or(1.0, |prev| f64::from(self.key.factor(KeyClass::of(prev, (c.key, c.mode)))))
        };
        let stages: [Vec<f64>; 3] = [
            candidates.iter().map(|c| c.weight * class(c) * dark(c)).collect(),
            candidates.iter().map(|c| c.weight * dark(c)).collect(),
            candidates.iter().map(|c| c.weight).collect(),
        ];
        let chosen = stages
            .iter()
            .find_map(|weights| pick(weights, u))
            .and_then(|index| candidates.get(index))
            .or_else(|| candidates.first())
            .unwrap_or_else(|| unreachable!("validated non-empty candidates"));
        (chosen.key, chosen.mode)
    }
}

/// Plan d'un set : sa longueur et ses tirages, pris **au début du set, avant l'air 1**.
///
/// Les tirages sont des entiers `0..KEY_DRAW_RANGE` convertis en tonalité à la construction
/// de chaque air, avec les poids de la valence d'alors (résolution tardive) : si le jeu change
/// de valence entre deux airs, la tonalité suit la valence, la structure du set ne change pas.
/// Un set sans fin (`tunes() == 0`) ne tire rien d'avance : chaque air tire à sa construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetPlan {
    first_tune: usize,
    tunes: usize,
    key_draws: Vec<u32>,
    transition_draws: Vec<u32>,
    gap_draw: u32,
}

impl SetPlan {
    /// Tire le bloc d'un set qui commence à l'air `first_tune` (0-based) : la longueur, puis un
    /// tirage de tonalité pour chacun des airs 2..=L.
    pub fn draw(first_tune: usize, set: &ProfileSet, rng: &mut dyn RngCore) -> Self {
        let total: u64 = set.length.iter().map(|entry| u64::from(entry.weight)).sum();
        if total == 0 {
            return Self {
                first_tune, tunes: 0, key_draws: Vec::new(),
                transition_draws: Vec::new(), gap_draw: 0,
            };
        }
        let mut remaining = rng.next_range_usize(0..total as usize) as u64;
        let tunes = set
            .length
            .iter()
            .find(|entry| {
                let hit = remaining < u64::from(entry.weight);
                remaining = remaining.saturating_sub(u64::from(entry.weight));
                hit
            })
            .map_or(1, |entry| entry.tunes);
        let key_draws =
            (1..tunes).map(|_| rng.next_range_usize(0..KEY_DRAW_RANGE) as u32).collect();
        let transition_draws =
            (1..tunes).map(|_| rng.next_range_usize(0..KEY_DRAW_RANGE) as u32).collect();
        let gap_draw = rng.next_range_usize(0..KEY_DRAW_RANGE) as u32;
        Self { first_tune, tunes, key_draws, transition_draws, gap_draw }
    }

    /// Indice 0-based du premier air du set dans la session.
    #[must_use]
    pub const fn first_tune(&self) -> usize {
        self.first_tune
    }

    /// Longueur du set en airs ; `0` pour un set sans fin.
    #[must_use]
    pub const fn tunes(&self) -> usize {
        self.tunes
    }

    /// Vrai si l'air `tune_index` (0-based dans la session) appartient à ce set.
    #[must_use]
    pub const fn contains(&self, tune_index: usize) -> bool {
        tune_index >= self.first_tune
            && (self.tunes == 0 || tune_index < self.first_tune + self.tunes)
    }

    /// Rang 1-based de `tune_index` dans le set.
    #[must_use]
    pub const fn tune_in_set(&self, tune_index: usize) -> usize {
        tune_index - self.first_tune + 1
    }

    /// Vrai si `tune_in_set` est le dernier air d'un set fini.
    #[must_use]
    pub const fn is_last(&self, tune_in_set: usize) -> bool {
        self.tunes != 0 && tune_in_set == self.tunes
    }

    /// Tirage de tonalité de l'air de rang `tune_in_set` (≥ 2). `None` : set sans fin.
    #[must_use]
    pub fn key_draw(&self, tune_in_set: usize) -> Option<u32> {
        self.key_draws.get(tune_in_set.checked_sub(2)?).copied()
    }

    /// Tirage de raccord de l'air de rang `tune_in_set` (≥ 2).
    #[must_use]
    pub fn transition_draw(&self, tune_in_set: usize) -> Option<u32> {
        self.transition_draws.get(tune_in_set.checked_sub(2)?).copied()
    }

    /// Pause choisie pour la fin de ce set.
    #[must_use]
    pub fn gap_bars(&self, set: &ProfileSet) -> usize {
        if self.tunes == 0 { 0 } else { set.gap_bars(self.gap_draw) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_follow_the_diatonic_collection_not_a_fixed_shift() {
        use Mode::{Aeolian, Dorian, Ionian, Mixolydian};
        let (d, g, a, b, e, fs) = (2, 7, 9, 11, 4, 6);
        assert_eq!(KeyClass::of((d, Ionian), (d, Ionian)), KeyClass::Same);
        assert_eq!(KeyClass::of((d, Ionian), (d, Mixolydian)), KeyClass::Parallel);
        assert_eq!(KeyClass::of((d, Ionian), (b, Aeolian)), KeyClass::Relative);
        // Ré mixolydien partage la collection de sol majeur : sa relative est la dorien, pas si dorien.
        assert_eq!(KeyClass::of((d, Mixolydian), (g, Ionian)), KeyClass::Relative);
        assert_eq!(KeyClass::of((d, Mixolydian), (a, Dorian)), KeyClass::Relative);
        assert_eq!(KeyClass::of((d, Mixolydian), (b, Dorian)), KeyClass::Other);
        assert_eq!(KeyClass::of((d, Ionian), (a, Ionian)), KeyClass::FifthUp);
        assert_eq!(KeyClass::of((d, Ionian), (g, Ionian)), KeyClass::FourthUp);
        assert_eq!(KeyClass::of((d, Ionian), (e, Ionian)), KeyClass::StepUp);
        assert_eq!(KeyClass::of((a, Ionian), (g, Ionian)), KeyClass::StepDown);
        assert_eq!(KeyClass::of((d, Ionian), (fs, Aeolian)), KeyClass::Other);
    }
}

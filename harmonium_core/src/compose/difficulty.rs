//! Filtre de difficulté de la mélodie (`[melody.difficulty]`, E10).
//!
//! Le filtre est en données : il borne le plus grand saut en demi-tons, le nombre d'attaques
//! par mesure, la plus petite durée et l'ambitus d'une cellule de la ligne écrite, et dit si les
//! ornements et l'ajustement aux accords peuvent encore la modifier. Il agit à deux endroits :
//! le tirage des cellules préfère celles qui le respectent (`pitch_excess`), puis
//! [`conform`] réécrit la ligne jouée d'un air, bout à bout et en boucle (d'une cellule, d'une
//! partie, d'un tour à l'autre), pour que les bornes tiennent sur chaque paire de notes
//! consécutives, quelle qu'ait été la structure de reprises qui les a amenées côte à côte.

use std::ops::Range;

use serde::{Deserialize, Serialize};

use super::{
    cells::{Cell, CellNote},
    melody::fold_midi_to_tessitura,
    mode::{Mode, degree_to_diatonic_step, diatonic_step_to_degree},
};
use crate::style_profile::ProfileError;

/// Plus petit pas diatonique écrit (`1,` = `-7`) et plus grand (`14` = `13`) d'une cellule.
const LOWEST_STEP: i8 = -7;
const HIGHEST_STEP: i8 = 13;

const fn default_true() -> bool {
    true
}
const fn is_true(value: &bool) -> bool {
    *value
}

/// Bornes de difficulté de la mélodie d'un profil. Chaque borne omise est sans effet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MelodyDifficulty {
    /// Plus grand saut, en demi-tons, entre deux attaques consécutives de la mélodie : dans une
    /// cellule, d'une cellule à la suivante, aux frontières de phrase et de partie, et de la
    /// dernière mesure de l'air à la première (retour au tour suivant).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_leap_semitones: Option<u8>,
    /// Plus grand nombre d'attaques dans une mesure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_attacks_per_bar: Option<usize>,
    /// Plus petite durée d'une note, en unités de cellule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_note_units: Option<usize>,
    /// Plus grand ambitus, en demi-tons, des attaques d'une même mesure. Le saut maximal prime :
    /// quand les deux ne se concilient pas, l'ambitus de la mesure est relâché.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cell_ambitus_semitones: Option<u8>,
    /// Faux : aucune note de la mélodie publiée ne sort de la gamme de l'air ; l'ajustement aux
    /// accords (`chord_fit`) ne retient alors que des notes de la gamme.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub out_of_scale: bool,
    /// Faux : aucun ornement sur la mélodie. Vrai : les ornements du profil s'ajoutent à la
    /// ligne écrite, hors des bornes.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub ornaments: bool,
    /// Faux : la mélodie publiée est la ligne écrite, sans rapprochement des notes des accords
    /// ni coupure aux changements d'accord. Vrai : l'ajustement du modèle d'harmonie du profil
    /// s'ajoute à la ligne écrite, hors des bornes.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub chord_fit: bool,
}

impl Default for MelodyDifficulty {
    fn default() -> Self {
        Self {
            max_leap_semitones: None,
            max_attacks_per_bar: None,
            min_note_units: None,
            max_cell_ambitus_semitones: None,
            out_of_scale: true,
            ornaments: true,
            chord_fit: true,
        }
    }
}

impl MelodyDifficulty {
    /// Valide les bornes contre la grille de la mélodie (`melody.cell_units` unités par mesure).
    pub fn validate(&self, cell_units: usize) -> Result<(), ProfileError> {
        let field = |name: &str| format!("melody.difficulty.{name}");
        let range = |value: Option<usize>, name: &str, hi: usize| match value {
            Some(v) if !(1..=hi).contains(&v) => {
                Err(ProfileError::new(field(name), format!("attendu 1..{hi}")))
            }
            _ => Ok(()),
        };
        range(self.max_leap_semitones.map(usize::from), "max_leap_semitones", 24)?;
        range(self.max_cell_ambitus_semitones.map(usize::from), "max_cell_ambitus_semitones", 36)?;
        range(self.max_attacks_per_bar, "max_attacks_per_bar", cell_units)?;
        range(self.min_note_units, "min_note_units", cell_units)
    }
}

/// Contexte de réalisation d'une ligne : tonalité rendue et tessiture.
pub(crate) struct Rendering<'a> {
    pub mode: Mode,
    pub tonic: u8,
    pub tessitura: (u8, u8),
    pub strong_units: &'a [usize],
}

impl Rendering<'_> {
    fn raw(&self, step: i8) -> i16 {
        i16::from(self.tonic)
            + i16::from(self.mode.degree_to_semitones(diatonic_step_to_degree(step)))
    }

    /// Hauteur rendue d'une note de cellule (repliée dans la tessiture, comme à la publication).
    pub(crate) fn pitch(&self, degree: i8) -> i16 {
        let raw = self.raw(degree_to_diatonic_step(degree));
        i16::from(fold_midi_to_tessitura(raw, self.tessitura.0, self.tessitura.1))
    }
}

/// Excès de hauteurs d'une cellule candidate sur les bornes (0 : elle les respecte), pour le
/// tirage : plus grand saut interne, saut d'entrée depuis `previous` et ambitus de la mesure.
pub(crate) fn pitch_excess(
    cell: &Cell,
    previous: Option<i16>,
    difficulty: &MelodyDifficulty,
    rendering: &Rendering<'_>,
) -> i16 {
    let pitches = cell.notes.iter().filter_map(|note| match note {
        CellNote::Degree(degree) => Some(rendering.pitch(*degree)),
        CellNote::Hold => None,
    });
    let mut excess = 0;
    let mut last = previous;
    let (mut lowest, mut highest) = (i16::MAX, i16::MIN);
    for pitch in pitches {
        if let (Some(leap), Some(last)) = (difficulty.max_leap_semitones, last) {
            excess += ((pitch - last).abs() - i16::from(leap)).max(0);
        }
        (lowest, highest) = (lowest.min(pitch), highest.max(pitch));
        last = Some(pitch);
    }
    if let Some(ambitus) = difficulty.max_cell_ambitus_semitones
        && highest >= lowest
    {
        excess += (highest - lowest - i16::from(ambitus)).max(0);
    }
    excess
}

/// Fusionne les attaques les plus courtes de `cell` jusqu'à respecter `max_attacks_per_bar` et
/// `min_note_units` : l'attaque retirée prolonge la précédente, les temps faibles partent avant
/// les temps forts, la fin de la mesure avant son début.
fn merge_rhythm(cell: &mut Cell, difficulty: &MelodyDifficulty, strong_units: &[usize]) {
    let max_attacks = difficulty.max_attacks_per_bar.unwrap_or(usize::MAX);
    let min_units = difficulty.min_note_units.unwrap_or(1);
    let units = cell.notes.len();
    loop {
        let attacks: Vec<usize> = cell
            .notes
            .iter()
            .enumerate()
            .filter(|(_, note)| matches!(note, CellNote::Degree(_)))
            .map(|(unit, _)| unit)
            .collect();
        if attacks.len() <= 1 {
            return;
        }
        let duration = |k: usize| attacks.get(k + 1).copied().unwrap_or(units) - attacks[k];
        let Some(shortest) = (0..attacks.len()).min_by_key(|&k| {
            (duration(k), strong_units.contains(&attacks[k]), std::cmp::Reverse(attacks[k]))
        }) else {
            return;
        };
        if attacks.len() <= max_attacks && duration(shortest) >= min_units {
            return;
        }
        cell.notes[attacks[shortest.max(1)]] = CellNote::Hold;
    }
}

/// Une attaque de la ligne : sa mesure, son unité, sa hauteur écrite et son poids.
struct Slot {
    bar: usize,
    unit: usize,
    written: i16,
    weight: i64,
}

/// Ce que la ligne sait d'une de ses mesures jouées.
#[derive(Clone, Copy)]
pub(crate) struct BarFacts {
    /// La mesure clôt une phrase : sa dernière note résiste aux déplacements.
    pub cadential: bool,
    /// La mesure est publiée : ni partie muette ni mesure muette de l'alternance. Les sauts se
    /// bornent d'une mesure publiée à la suivante, par-dessus les mesures muettes.
    pub published: bool,
}

/// Réécrit `cells` (les mesures jouées d'un air, décrites par `bars`) pour respecter
/// `difficulty`. La dernière mesure publiée se raccorde à la première : la ligne se rejoue en
/// boucle d'un tour à l'autre. `tour_bars` : mesures d'un tour ; la dernière mesure du premier
/// tour se raccorde aussi à la première, car l'intro d'entrée par la fin enchaîne l'une à l'autre.
pub(crate) fn conform(
    cells: &mut [Cell],
    bars: &[BarFacts],
    tour_bars: usize,
    difficulty: &MelodyDifficulty,
    rendering: &Rendering<'_>,
) {
    if difficulty.max_attacks_per_bar.is_some() || difficulty.min_note_units.is_some() {
        for cell in cells.iter_mut() {
            merge_rhythm(cell, difficulty, rendering.strong_units);
        }
    }
    if difficulty.max_leap_semitones.is_none() && difficulty.max_cell_ambitus_semitones.is_none() {
        return;
    }
    let scale: Vec<(i8, i16)> = (LOWEST_STEP..=HIGHEST_STEP)
        .map(|step| (step, rendering.raw(step)))
        .filter(|&(_, pitch)| {
            (i16::from(rendering.tessitura.0)..=i16::from(rendering.tessitura.1)).contains(&pitch)
        })
        .collect();
    if scale.is_empty() {
        return;
    }
    let mut slots = Vec::new();
    for (bar, cell) in cells.iter().enumerate() {
        if !bars[bar].published {
            continue;
        }
        let attacks: Vec<_> = cell
            .notes
            .iter()
            .enumerate()
            .filter_map(|(unit, note)| match note {
                CellNote::Degree(degree) => Some((unit, rendering.pitch(*degree))),
                CellNote::Hold => None,
            })
            .collect();
        for (index, &(unit, written)) in attacks.iter().enumerate() {
            let closing = index + 1 == attacks.len() && bars[bar].cadential;
            slots.push(Slot { bar, unit, written, weight: if closing { 4 } else { 1 } });
        }
    }
    if slots.is_empty() {
        return;
    }
    // Attaques qui doivent rejoindre la première : la dernière de la ligne et la dernière du
    // premier tour.
    let mut closing = vec![slots.len() - 1];
    if let Some(end) = slots.iter().rposition(|slot| slot.bar + 1 == tour_bars)
        && end + 1 != slots.len()
    {
        closing.push(end);
    }
    let leap = difficulty.max_leap_semitones.map(i16::from);
    let ambitus = difficulty.max_cell_ambitus_semitones.map(i16::from);
    let Some(chosen) = solve(&slots, &scale, ambitus, leap, &closing) else { return };
    for (slot, &index) in slots.iter().zip(&chosen) {
        cells[slot.bar].notes[slot.unit] =
            CellNote::Degree(diatonic_step_to_degree(scale[index].0));
    }
}

/// Fenêtres de hauteurs d'une mesure, en indices de `scale` : pour chaque note de départ, les
/// notes jusqu'à `ambitus` demi-tons au-dessus, sans les fenêtres contenues dans une autre.
/// Sans borne d'ambitus, la gamme entière.
fn windows(scale: &[(i8, i16)], ambitus: Option<i16>) -> Vec<Range<usize>> {
    let Some(ambitus) = ambitus else { return std::iter::once(0..scale.len()).collect() };
    let mut windows: Vec<Range<usize>> = Vec::new();
    for start in 0..scale.len() {
        let end = scale.partition_point(|&(_, pitch)| pitch <= scale[start].1 + ambitus);
        if windows.last().is_none_or(|last| end > last.end) {
            windows.push(start..end);
        }
    }
    windows
}

/// Meilleur chemin jusqu'à une attaque : son coût, la note de la mesure précédente d'où il vient
/// et les notes choisies pour les attaques de sa mesure.
#[derive(Clone)]
struct Step {
    cost: i64,
    entry: usize,
    path: Vec<usize>,
}

/// Choisit pour chaque attaque une note de `scale` (indice) en bornant le saut entre attaques
/// consécutives, dernière et première comprises, et l'ambitus de chaque mesure, au coût minimal
/// (écart pondéré aux hauteurs écrites). Une ligne constante convient toujours : la recherche
/// aboutit.
fn solve(
    slots: &[Slot],
    scale: &[(i8, i16)],
    ambitus: Option<i16>,
    leap: Option<i16>,
    closing: &[usize],
) -> Option<Vec<usize>> {
    let windows = windows(scale, ambitus);
    let search = Search { slots, scale, windows: &windows, leap, closing };
    let free = search.forward(None)?;
    if closing.iter().all(|&k| search.allowed(free[k], free[0])) {
        return Some(free);
    }
    // La boucle ne se ferme pas : on fixe la première note, d'abord parmi les trois moins chères,
    // puis parmi toutes.
    let mut starts: Vec<usize> = (0..scale.len()).collect();
    starts.sort_by_key(|&index| search.cost(&slots[0], index));
    for attempt in [&starts[..starts.len().min(3)], &starts[starts.len().min(3)..]] {
        let best = attempt
            .iter()
            .filter_map(|&start| search.forward(Some(start)))
            .min_by_key(|path| search.total(path));
        if best.is_some() {
            return best;
        }
    }
    None
}

struct Search<'a> {
    slots: &'a [Slot],
    scale: &'a [(i8, i16)],
    windows: &'a [Range<usize>],
    leap: Option<i16>,
    closing: &'a [usize],
}

impl Search<'_> {
    fn allowed(&self, from: usize, to: usize) -> bool {
        self.leap.is_none_or(|leap| (self.scale[from].1 - self.scale[to].1).abs() <= leap)
    }

    fn cost(&self, slot: &Slot, index: usize) -> i64 {
        slot.weight * i64::from((self.scale[index].1 - slot.written).abs())
    }

    fn total(&self, path: &[usize]) -> i64 {
        self.slots.iter().zip(path).map(|(slot, &index)| self.cost(slot, index)).sum()
    }

    /// Programmation dynamique mesure par mesure : le coût de chaque mesure est minimal sur
    /// toutes ses fenêtres d'ambitus, pour chaque dernière note. `start` fixe la première note
    /// et rend les attaques de `closing` tenues de la rejoindre.
    fn forward(&self, start: Option<usize>) -> Option<Vec<usize>> {
        const UNREACHABLE: i64 = i64::MAX;
        let width = self.scale.len();
        let mut runs: Vec<Range<usize>> = Vec::new();
        for (k, slot) in self.slots.iter().enumerate() {
            match runs.last_mut() {
                Some(run) if self.slots[run.start].bar == slot.bar => run.end = k + 1,
                _ => runs.push(k..k + 1),
            }
        }
        let mut carry = vec![0_i64; width];
        let mut trace: Vec<Vec<Option<Step>>> = Vec::with_capacity(runs.len());
        for (position, run) in runs.iter().enumerate() {
            let mut next: Vec<Option<Step>> = vec![None; width];
            for window in self.windows {
                // Première attaque de la mesure : meilleure note d'entrée compatible.
                let mut layer: Vec<(i64, usize)> = Vec::with_capacity(window.len());
                for to in window.clone() {
                    let (base, entry) = if position == 0 {
                        (if start.is_none_or(|s| s == to) { 0 } else { UNREACHABLE }, 0)
                    } else {
                        (0..width)
                            .filter(|&from| carry[from] != UNREACHABLE && self.allowed(from, to))
                            .map(|from| (carry[from], from))
                            .min_by_key(|&(cost, _)| cost)
                            .unwrap_or((UNREACHABLE, 0))
                    };
                    layer.push(
                        self.settle(run.start, to, base, start)
                            .map_or((UNREACHABLE, entry), |c| (c, entry)),
                    );
                }
                let first_layer = layer.clone();
                let mut back: Vec<Vec<usize>> = Vec::new();
                for k in run.start + 1..run.end {
                    let mut row = Vec::with_capacity(window.len());
                    let mut pointers = Vec::with_capacity(window.len());
                    for to in window.clone() {
                        let (base, from) = window
                            .clone()
                            .filter(|&from| {
                                layer[from - window.start].0 != UNREACHABLE
                                    && self.allowed(from, to)
                            })
                            .map(|from| (layer[from - window.start].0, from))
                            .min_by_key(|&(cost, _)| cost)
                            .unwrap_or((UNREACHABLE, window.start));
                        row.push((self.settle(k, to, base, start).unwrap_or(UNREACHABLE), 0));
                        pointers.push(from - window.start);
                    }
                    layer = row;
                    back.push(pointers);
                }
                for (offset, &(cost, _)) in layer.iter().enumerate() {
                    let to = window.start + offset;
                    if cost == UNREACHABLE
                        || next[to].as_ref().is_some_and(|best| best.cost <= cost)
                    {
                        continue;
                    }
                    // Remonte la mesure : notes choisies et note d'entrée.
                    let mut path = vec![to; run.len()];
                    let mut at = offset;
                    for (depth, pointers) in back.iter().enumerate().rev() {
                        at = pointers[at];
                        path[depth] = window.start + at;
                    }
                    next[to] = Some(Step { cost, entry: first_layer[at].1, path });
                }
            }
            carry = next.iter().map(|step| step.as_ref().map_or(UNREACHABLE, |s| s.cost)).collect();
            trace.push(next);
        }
        let mut end = (0..width).filter(|&j| carry[j] != UNREACHABLE).min_by_key(|&j| carry[j])?;
        let mut path = vec![0; self.slots.len()];
        for (run, steps) in runs.iter().zip(&trace).rev() {
            let step = steps[end].as_ref()?;
            path[run.clone()].copy_from_slice(&step.path);
            end = step.entry;
        }
        Some(path)
    }

    /// Coût de la note `index` pour l'attaque `k` après `base` ; `None` si une attaque de
    /// `closing` ne rejoint pas la première note fixée `start`.
    fn settle(&self, k: usize, index: usize, base: i64, start: Option<usize>) -> Option<i64> {
        if base == i64::MAX {
            return None;
        }
        if start.is_some_and(|start| self.closing.contains(&k) && !self.allowed(index, start)) {
            return None;
        }
        Some(base + self.cost(&self.slots[k], index))
    }
}

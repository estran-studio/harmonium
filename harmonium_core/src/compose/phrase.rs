//! Phrases de plusieurs mesures : découpage d'une passe de partie en phrases tirées d'une
//! banque (comping, batterie), sans état d'une mesure à l'autre.
//!
//! Algorithme générique : aucune valeur musicale ni nom de style ici. Une passe de partie de
//! `bars` mesures se découpe en phrases par un tirage pondéré dont les flux ChaCha8 ne dépendent
//! que de la graine de la passe (`Window::seed`, dérivée de la graine d'accompagnement et de
//! l'indice de la première mesure de la passe) : la mesure `n` retrouve sa phrase en
//! recalculant le découpage de sa passe, quelle que soit la mesure précédemment générée (le seek
//! rejoue la même musique). Le découpage est calculé **une fois par mesure générée** (`tile`),
//! pas une fois par mesure rejouée.
//!
//! Alignement : le découpage ne franchit jamais la frontière d'une partie, donc ni celle d'un
//! chorus (un chorus est un nombre entier de parties). Une phrase de `PartEnd` ou `ChorusEnd`
//! se pose en **fin** de passe, c'est la mise en place avant le changement de section ; le
//! reste de la passe se découpe depuis son début.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::{algo::RngCore, ornaments::mix};

/// Nombre maximal de mesures d'une phrase de comping ou de batterie.
pub const MAX_PHRASE_BARS: usize = 8;
/// Nombre maximal de phrases (et de relances) d'une banque.
pub const MAX_PHRASES: usize = 256;
/// Nombre maximal de mesures d'une relance.
pub const MAX_FILL_BARS: usize = 4;

const CLOSE_STREAM: u64 = 0x5048_434C;
const OPEN_STREAM: u64 = 0x5048_4F50;
const FILL_STREAM: u64 = 0x5048_464C;

/// Où une phrase (ou une relance) peut se poser dans la passe d'une partie.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhraseAt {
    /// N'importe où dans la passe.
    #[default]
    Any,
    /// Première phrase de la passe, ouverture après un changement de section.
    PartStart,
    /// Dernière phrase de chaque passe de partie : mise en place avant un changement de section.
    PartEnd,
    /// Dernière phrase de la dernière partie du chorus ; l'emporte sur `PartEnd` à cet endroit.
    ChorusEnd,
}

const CLAVE_STREAM: u64 = 0x434C_4156;

/// Côté de la clave où commence une phrase : la clave est un cycle de deux mesures, la mesure
/// « trois » (trois attaques) et la mesure « deux ».
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaveSide {
    Three,
    Two,
}

impl ClaveSide {
    /// Rang du côté dans le cycle de la clave (0 : mesure « trois » d'une 3-2).
    const fn phase(self) -> usize {
        match self {
            Self::Three => 0,
            Self::Two => 1,
        }
    }
}

/// Sens de la clave : 3-2 (la mesure « trois » d'abord), 2-3, ou tiré une fois par air dans la
/// graine d'accompagnement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaveDirection {
    #[default]
    ThreeTwo,
    TwoThree,
    Either,
}

/// Clave : cycle de deux mesures qui contraint le début des phrases de comping et de batterie
/// (`clave = "three"` ou `"two"` sur la phrase). Le cycle se poursuit d'un chorus à l'autre : la
/// mesure `n` de l'air (0-based, tours compris) est « trois » quand `n` est pair dans une 3-2,
/// impair dans une 2-3. Absente du profil, aucune phrase ne peut porter de côté.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clave {
    #[serde(default)]
    pub direction: ClaveDirection,
}

impl Clave {
    /// Rang dans le cycle (0 : « trois », 1 : « deux ») de la mesure `performed_bar` de l'air
    /// dont la graine d'accompagnement est `accompaniment_seed`.
    #[must_use]
    pub fn phase(&self, accompaniment_seed: u64, performed_bar: usize) -> usize {
        let shift = match self.direction {
            ClaveDirection::ThreeTwo => 0,
            ClaveDirection::TwoThree => 1,
            ClaveDirection::Either => (mix(accompaniment_seed, CLAVE_STREAM) & 1) as usize,
        };
        (performed_bar + shift) % 2
    }

    /// Côté de la mesure `performed_bar` de l'air.
    #[must_use]
    pub fn side(&self, accompaniment_seed: u64, performed_bar: usize) -> ClaveSide {
        if self.phase(accompaniment_seed, performed_bar) == 0 {
            ClaveSide::Three
        } else {
            ClaveSide::Two
        }
    }
}

/// Passe d'une partie, pour le découpage en phrases.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Window {
    /// Graine de la passe pour la couche (comping, batterie) : tous les tirages en dérivent.
    pub seed: u64,
    /// Mesures de la passe.
    pub bars: usize,
    /// Vrai si la passe est celle de la dernière partie du chorus.
    pub chorus_end: bool,
    /// Rang dans le cycle de la clave de la première mesure de la passe ; `None` : pas de clave.
    pub clave_phase: Option<usize>,
}

impl Window {
    /// Vrai si une phrase de côté `side` peut commencer à la mesure `at` de la passe.
    fn fits(&self, side: Option<ClaveSide>, at: usize) -> bool {
        match (side, self.clave_phase) {
            (Some(side), Some(phase)) => (phase + at) % 2 == side.phase(),
            _ => true,
        }
    }
}

/// Une phrase candidate au tirage : longueur, poids effectif (0 : exclue) et emplacement.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Candidate {
    pub bars: usize,
    pub weight: u64,
    pub at: PhraseAt,
    /// Côté de la clave où la phrase commence ; `None` : n'importe lequel.
    pub side: Option<ClaveSide>,
}

/// Phrase qui couvre une mesure : indice dans la banque, rang de la mesure dans la phrase et
/// longueur de la phrase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Slot {
    pub index: usize,
    pub bar: usize,
    pub bars: usize,
}

impl Slot {
    /// Dernière mesure de la phrase.
    pub(crate) const fn is_last(&self) -> bool {
        self.bar + 1 == self.bars
    }
}

/// Tirage pondéré parmi `candidates` avec le flux `seed` ; `None` si aucun ne concourt.
fn draw<'a>(
    seed: u64,
    candidates: impl Iterator<Item = &'a (usize, Candidate)> + Clone,
) -> Option<(usize, Candidate)> {
    let total: u64 = candidates.clone().map(|(_, c)| c.weight).fold(0, u64::saturating_add);
    if total == 0 {
        return None;
    }
    let mut pick = ChaCha8Rng::seed_from_u64(seed).next_range_u64(0..total);
    for &(index, candidate) in candidates {
        if pick < candidate.weight {
            return Some((index, candidate));
        }
        pick -= candidate.weight;
    }
    None
}

/// Découpe la passe `window` en phrases de la banque de `count` éléments ; `candidate(i)`
/// décrit l'élément `i` (`None` : exclu, p. ex. par une étiquette ou une densité).
///
/// Retourne, pour chaque mesure de la passe, la phrase qui la couvre ; `None` : aucune phrase
/// ne tient dans ce qui reste (la mesure se joue comme sans phrase).
pub(crate) fn tile(
    window: Window,
    count: usize,
    candidate: impl Fn(usize) -> Option<Candidate>,
) -> Vec<Option<Slot>> {
    let mut slots = vec![None; window.bars];
    let candidates: Vec<(usize, Candidate)> = (0..count)
        .filter_map(|index| candidate(index).map(|c| (index, c)))
        .filter(|(_, c)| c.weight > 0 && (1..=window.bars).contains(&c.bars))
        .collect();
    if candidates.is_empty() {
        return slots;
    }
    let place = |slots: &mut [Option<Slot>], start: usize, index: usize, bars: usize| {
        for bar in 0..bars {
            slots[start + bar] = Some(Slot { index, bar, bars });
        }
    };
    // Fin de passe : la classe la plus précise qui a un candidat l'emporte.
    let mut end = window.bars;
    let closing = |at: PhraseAt| {
        candidates
            .iter()
            .filter(move |(_, c)| c.at == at && window.fits(c.side, window.bars - c.bars))
    };
    let class = if window.chorus_end && closing(PhraseAt::ChorusEnd).next().is_some() {
        PhraseAt::ChorusEnd
    } else {
        PhraseAt::PartEnd
    };
    if let Some((index, c)) =
        draw(mix(mix(window.seed, CLOSE_STREAM), window.bars as u64), closing(class))
    {
        end = window.bars - c.bars;
        place(&mut slots, end, index, c.bars);
    }
    // Reste de la passe, depuis son début.
    let mut pos = 0;
    while pos < end {
        let rest = end - pos;
        let open = candidates.iter().filter(|(_, c)| {
            c.bars <= rest
                && (c.at == PhraseAt::Any || (c.at == PhraseAt::PartStart && pos == 0))
                && window.fits(c.side, pos)
        });
        let Some((index, c)) = draw(mix(mix(window.seed, OPEN_STREAM), pos as u64), open) else {
            break;
        };
        place(&mut slots, pos, index, c.bars);
        pos += c.bars;
    }
    slots
}

/// Relance tirée à la fin `end` (mesure 0-based de la passe) d'une phrase de `available`
/// mesures au plus : `candidate(i)` décrit la relance `i` (longueur, poids, emplacement) ;
/// `none_weight` : poids de « pas de relance ». `None` : pas de relance.
///
/// Comme pour la fin de passe, la classe la plus précise qui a un candidat l'emporte.
pub(crate) fn pick_fill(
    window: Window,
    end: usize,
    available: usize,
    count: usize,
    none_weight: u32,
    candidate: impl Fn(usize) -> Option<Candidate>,
) -> Option<usize> {
    let part_end = end + 1 == window.bars;
    let candidates: Vec<(usize, Candidate)> = (0..count)
        .filter_map(|index| candidate(index).map(|c| (index, c)))
        .filter(|(_, c)| c.weight > 0 && c.bars <= available)
        .filter(|(_, c)| match c.at {
            PhraseAt::Any => true,
            PhraseAt::PartEnd => part_end,
            PhraseAt::ChorusEnd => part_end && window.chorus_end,
            PhraseAt::PartStart => false,
        })
        .collect();
    let class = [PhraseAt::ChorusEnd, PhraseAt::PartEnd, PhraseAt::Any]
        .into_iter()
        .find(|at| candidates.iter().any(|(_, c)| c.at == *at))?;
    // « Pas de relance » concourt comme un candidat de plus (indice hors banque).
    let none =
        (count, Candidate { bars: 0, weight: u64::from(none_weight), at: class, side: None });
    let pool = candidates.iter().filter(|(_, c)| c.at == class).chain(std::iter::once(&none));
    let (index, _) = draw(mix(mix(window.seed, FILL_STREAM), end as u64), pool)?;
    (index < count).then_some(index)
}

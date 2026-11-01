//! Boucles d'accords générées par contraintes (`harmony.parts.<partie>.generated`).
//!
//! Une boucle est une suite de `count` accords dont chaque fondamentale suit le mouvement permis
//! (`motion`) et dont la qualité est tirée parmi les qualités permises qui respectent `key_fit`
//! (rapport des notes de l'accord, basse comprise, à la gamme du mode actif). Le tirage procède
//! sur une table de faisabilité calculée à rebours (quelles fondamentales laissent une suite
//! complète possible à chaque position) : aucune pioche rejetée, aucune boucle partielle, et le
//! flux du `ChaCha8Rng` injecté ne dépend que des données et du mode.
//!
//! Le même calcul sert à la validation : une contrainte qui n'a aucune solution dans un mode
//! atteignable est refusée au chargement, avec le champ en cause.

use arrayvec::ArrayVec;

use crate::{
    algo::RngCore,
    compose::Mode,
    style_profile::{GeneratedLoop, KeyFit, LoopChord, MotionDirection, RootMotion},
};

/// Pourquoi une contrainte n'a aucune solution dans un mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Unsatisfiable {
    /// Aucune qualité permise ne respecte `key_fit` sur la fondamentale de départ.
    Start,
    /// Aucune suite de `count` accords ne respecte le mouvement et `key_fit`.
    Motion,
}

/// Classes de hauteur (relatives à la tonique) de la gamme du mode, en masque de 12 bits.
fn scale_mask(mode: Mode) -> u16 {
    mode.intervals().iter().fold(0, |mask, &interval| mask | 1 << (interval as u8 % 12))
}

/// Masque des classes de hauteur d'un accord (fondamentale `root`, basse comprise).
fn chord_mask(spec: &GeneratedLoop, quality: usize, root: u8) -> u16 {
    let mut mask = spec.qualities[quality]
        .intervals()
        .into_iter()
        .fold(0u16, |mask, interval| mask | 1 << ((root + interval) % 12));
    if let Some(pedal) = spec.pedal {
        mask |= 1 << pedal;
    }
    mask
}

/// Qualités (indices dans `spec.qualities`) permises sur chaque fondamentale.
fn allowed(spec: &GeneratedLoop, mode: Mode) -> [Vec<usize>; 12] {
    let scale = scale_mask(mode);
    std::array::from_fn(|root| {
        (0..spec.qualities.len())
            .filter(|&quality| {
                let mask = chord_mask(spec, quality, root as u8);
                match spec.key_fit {
                    KeyFit::Free => true,
                    KeyFit::Within => mask & !scale == 0,
                    KeyFit::Outside => mask & !scale != 0,
                }
            })
            .collect()
    })
}

/// Fondamentales suivantes (classe, poids) depuis `root`.
fn successors(motion: &RootMotion, root: u8, mode: Mode) -> ArrayVec<(u8, u32), 24> {
    let mut found = ArrayVec::new();
    let mut push = |semitones: i16, weight: u32| {
        found.push((((i16::from(root) + semitones).rem_euclid(12)) as u8, weight));
    };
    let directed =
        |direction: MotionDirection, up: i16, down: i16, push: &mut dyn FnMut(i16, u32)| {
            if direction != MotionDirection::Down {
                push(up, 1);
            }
            if direction != MotionDirection::Up {
                push(down, 1);
            }
        };
    match motion {
        RootMotion::Chromatic { direction } => directed(*direction, 1, -1, &mut push),
        RootMotion::Fifths { direction } => directed(*direction, 7, -7, &mut push),
        RootMotion::ScaleSteps { direction } => {
            let scale = scale_mask(mode);
            let in_scale = |pc: i16| scale & (1 << pc.rem_euclid(12)) != 0;
            let root = i16::from(root);
            let up = (1..=12).find(|&i| in_scale(root + i)).unwrap_or(1);
            let down = (1..=12).find(|&i| in_scale(root - i)).unwrap_or(1);
            directed(*direction, up, -down, &mut push);
        }
        RootMotion::Intervals { intervals } => {
            for interval in intervals {
                push(i16::from(interval.semitones), interval.weight);
            }
        }
    }
    found
}

fn start_root(spec: &GeneratedLoop) -> u8 {
    spec.start.unwrap_or(0)
}

/// `rows[i]` : bit `r` si une suite valide existe depuis la fondamentale `r` en position `i`.
fn feasibility(spec: &GeneratedLoop, mode: Mode, allowed: &[Vec<usize>; 12]) -> Vec<u16> {
    let mut rows = vec![0u16; spec.count];
    for position in (0..spec.count).rev() {
        for root in 0..12u8 {
            if allowed[usize::from(root)].is_empty() {
                continue;
            }
            let last = position + 1 == spec.count;
            if last
                || successors(&spec.motion, root, mode)
                    .iter()
                    .any(|&(next, _)| rows[position + 1] & (1 << next) != 0)
            {
                rows[position] |= 1 << root;
            }
        }
    }
    rows
}

/// Vérifie que la contrainte a au moins une solution dans `mode`.
pub(crate) fn check(spec: &GeneratedLoop, mode: Mode) -> Result<(), Unsatisfiable> {
    let allowed = allowed(spec, mode);
    let start = start_root(spec);
    if allowed[usize::from(start)].is_empty() {
        return Err(Unsatisfiable::Start);
    }
    if feasibility(spec, mode, &allowed)[0] & (1 << start) == 0 {
        return Err(Unsatisfiable::Motion);
    }
    Ok(())
}

/// Tire une boucle de `slots` créneaux de `steps_per_chord` pas (contrainte validée : `check`
/// réussit pour `mode`). Les `slots % count` premiers accords reçoivent un créneau de plus.
pub(crate) fn generate(
    spec: &GeneratedLoop,
    slots: usize,
    steps_per_chord: usize,
    mode: Mode,
    rng: &mut dyn RngCore,
) -> Vec<LoopChord> {
    let allowed = allowed(spec, mode);
    let rows = feasibility(spec, mode, &allowed);
    let mut root = start_root(spec);
    let mut chords = Vec::with_capacity(spec.count);
    for position in 0..spec.count {
        let qualities = &allowed[usize::from(root)];
        let quality = qualities[rng.next_range_u64(0..qualities.len() as u64) as usize];
        let chord_slots = slots / spec.count + usize::from(position < slots % spec.count);
        chords.push(LoopChord {
            degree: None,
            semitones: Some(root),
            quality: spec.qualities[quality],
            bass: spec.pedal,
            steps: Some(chord_slots * steps_per_chord),
        });
        if position + 1 < spec.count {
            let options: ArrayVec<(u8, u32), 24> = successors(&spec.motion, root, mode)
                .into_iter()
                .filter(|&(next, _)| rows[position + 1] & (1 << next) != 0)
                .collect();
            let total: u64 = options.iter().map(|&(_, weight)| u64::from(weight)).sum();
            let mut pick = rng.next_range_u64(0..total);
            root = options
                .iter()
                .find(|&&(_, weight)| {
                    let hit = pick < u64::from(weight);
                    pick = pick.saturating_sub(u64::from(weight));
                    hit
                })
                .map_or_else(|| unreachable!("faisabilité calculée à rebours"), |&(next, _)| next);
        }
    }
    chords
}

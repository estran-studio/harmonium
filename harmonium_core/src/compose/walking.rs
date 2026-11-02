//! Marche de basse : une attaque par pulsation, référée à l'accord courant au pas près.
//!
//! Chaque changement d'accord (et le premier temps de la mesure) est une **arrivée** sur la
//! basse de l'accord (la fondamentale, ou la basse d'un accord slash), placée dans le registre
//! `low..=high`. La pulsation qui précède l'arrivée suivante est une **approche** (demi-ton
//! ou note de la gamme du mode, du côté de l'arrivée précédente) ; les pulsations entre les deux
//! sont des **notes de passage** (note de l'accord ou pas diatonique) dans la fenêtre de marche
//! `walk_low..=walk_high`, tirées de façon à pouvoir atteindre la cible.
//!
//! Le feel à deux temps ne garde que les pulsations qui tombent toutes les `attack_pulses`
//! (les temps 1 et 3 d'une mesure à quatre pulsations) et les arrivées : la même ligne, avec
//! une seule attaque par étendue d'accord en dehors de l'arrivée (l'approche), tenue jusqu'à
//! l'attaque suivante.
//!
//! Une mesure ne dépend que de ses accords, du premier accord de la suivante et de sa graine :
//! elle se rejoue à l'identique, sans état d'une mesure à l'autre.

use arrayvec::ArrayVec;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::{
    accomp::place_in_range,
    accomp_profile::{BassVoice, BassWalking},
    harmony::DegreeChord,
    mode::Mode,
};
use crate::{
    algo::RngCore,
    ornaments::mix,
    theory::PitchClass,
    timeline::{Articulation, NoteId, TimelineNote},
};

/// Candidats d'une note de passage : (hauteur, note de l'accord, pas diatonique).
type Pool = ArrayVec<(u8, bool, bool), 32>;

/// Grille et graine d'une mesure de marche.
pub(crate) struct WalkingBar<'a> {
    /// Un accord par créneau `steps_per_chord`.
    pub chords: &'a [DegreeChord],
    /// Premier accord de la mesure suivante ; `None` en fin de parcours (aucune cible).
    pub next_first: Option<DegreeChord>,
    pub key: PitchClass,
    pub mode: Mode,
    pub steps_per_chord: usize,
    pub steps_per_pulse: usize,
    /// Graine de la mesure : même graine, même ligne.
    pub seed: u64,
    /// Pulsations entre deux attaques : 1, une marche par pulsation.
    pub attack_pulses: usize,
}

/// Tirage pondéré ; `None` si tous les poids sont nuls.
fn draw(weights: &[u64], rng: &mut ChaCha8Rng) -> Option<usize> {
    let total: u64 = weights.iter().sum();
    if total == 0 {
        return None;
    }
    let mut pick = rng.next_range_u64(0..total);
    weights.iter().position(|&weight| {
        let hit = pick < weight;
        pick = pick.saturating_sub(weight);
        hit
    })
}

struct Line<'a> {
    bar: &'a WalkingBar<'a>,
    bass: &'a BassVoice,
    walking: &'a BassWalking,
    window: (u8, u8),
}

impl Line<'_> {
    fn target(&self, chord: DegreeChord) -> u8 {
        place_in_range(chord.bass_pc(self.bar.key, self.bar.mode), self.bass.low, self.bass.high)
    }

    fn in_window(&self, pitch: i16) -> Option<u8> {
        u8::try_from(pitch).ok().filter(|p| (self.window.0..=self.window.1).contains(p))
    }

    /// Note d'approche de `goal`, du côté de `from` (l'arrivée précédente), puis de l'autre côté
    /// si la fenêtre ne le permet pas. Chromatique : un demi-ton. Diatonique : la première
    /// note de la gamme du mode (à un, deux ou trois demi-tons).
    fn approach(&self, from: u8, goal: u8, chromatic: bool) -> Option<u8> {
        let from_above = from > goal;
        [from_above, !from_above].into_iter().find_map(|above| {
            (1..=if chromatic { 1 } else { 3 }).find_map(|distance| {
                let signed = if above { distance } else { -distance };
                let pitch = self.in_window(i16::from(goal) + signed)?;
                (chromatic || self.bar.mode.contains_midi(self.bar.key, pitch)).then_some(pitch)
            })
        })
    }

    /// Note de passage depuis `current` : un tirage pondéré entre notes de l'accord et pas
    /// diatoniques, parmi celles qui laissent la cible atteignable (`beats_left` bonds de
    /// `max_leap` au plus). `goal` : hauteur à atteindre et nombre de pulsations restantes ;
    /// `before` : la note qui précède `current` (la ligne n'y retourne pas tant qu'un autre
    /// choix existe).
    fn passing(
        &self,
        current: u8,
        before: Option<u8>,
        chord: DegreeChord,
        goal: Option<(u8, usize)>,
        rng: &mut ChaCha8Rng,
    ) -> u8 {
        let (key, mode) = (self.bar.key, self.bar.mode);
        let max_leap = self.walking.max_leap;
        let chord_pcs = chord.pitch_classes(key, mode);
        let up = (current.saturating_add(1)..=self.window.1).find(|&p| mode.contains_midi(key, p));
        let down = (self.window.0..current).rev().find(|&p| mode.contains_midi(key, p));
        let low = self.window.0.max(current.saturating_sub(max_leap));
        let high = self.window.1.min(current.saturating_add(max_leap));
        // (hauteur, note de l'accord, pas diatonique)
        let all: ArrayVec<(u8, bool, bool), 32> = (low..=high)
            .filter(|&p| p != current)
            .filter_map(|p| {
                let chord_tone = chord_pcs.contains(&(p % 12));
                let scale_step = Some(p) == up || Some(p) == down;
                (chord_tone || scale_step).then_some((p, chord_tone, scale_step))
            })
            .collect();
        let reaches = |p: u8| {
            goal.is_none_or(|(pitch, beats)| {
                usize::from(p.abs_diff(pitch)) <= beats * usize::from(max_leap)
            })
        };
        let reached: Pool = all.iter().copied().filter(|&(p, ..)| reaches(p)).collect();
        // Genre de note tiré selon les poids : d'abord parmi les notes qui laissent la cible
        // atteignable, sinon parmi toutes ; à défaut de genre pondéré, n'importe laquelle.
        let passing = &self.walking.passing;
        let mut pool = [&reached, &all]
            .into_iter()
            .find_map(|candidates| {
                let kinds = [
                    if candidates.iter().any(|c| c.1) {
                        u64::from(passing.chord_tone_weight)
                    } else {
                        0
                    },
                    if candidates.iter().any(|c| c.2) {
                        u64::from(passing.scale_step_weight)
                    } else {
                        0
                    },
                ];
                let kind = draw(&kinds, rng)?;
                Some(
                    candidates
                        .iter()
                        .copied()
                        .filter(|c| if kind == 0 { c.1 } else { c.2 })
                        .collect(),
                )
            })
            .unwrap_or_else(|| if reached.is_empty() { all.clone() } else { reached.clone() });
        // Pas de retour sur la note d'avant tant que le genre tiré offre un autre choix.
        if let Some(back) = before {
            let forward: ArrayVec<(u8, bool, bool), 32> =
                pool.iter().copied().filter(|&(p, ..)| p != back).collect();
            if !forward.is_empty() {
                pool = forward;
            }
        }
        if pool.is_empty() {
            // Ni note d'accord ni pas diatonique dans la fenêtre : le voisin de la gamme
            // le plus proche de la cible, sinon la note répétée.
            return match (up, down, goal) {
                (Some(u), Some(d), Some((g, _))) => {
                    if u.abs_diff(g) <= d.abs_diff(g) {
                        u
                    } else {
                        d
                    }
                }
                (Some(p), ..) | (None, Some(p), _) => p,
                (None, None, _) => current,
            };
        }
        let toward_bonus = u64::from(self.walking.toward_target_weight);
        let keep_bonus = u64::from(self.walking.keep_direction_weight);
        let weights: ArrayVec<u64, 32> = pool
            .iter()
            .map(|&(p, ..)| {
                let toward = goal.is_some_and(|(g, _)| p.abs_diff(g) < current.abs_diff(g));
                let keeps = before.is_some_and(|b| (p > current) == (current > b));
                1 + if toward { toward_bonus } else { 0 } + if keeps { keep_bonus } else { 0 }
            })
            .collect();
        draw(&weights, rng).map_or(pool[0].0, |i| pool[i].0)
    }
}

/// Une mesure de marche : une note par pulsation, ou une toutes les `attack_pulses` pulsations
/// et à chaque arrivée.
#[must_use]
pub(crate) fn walking_bar(
    bar: &WalkingBar<'_>,
    bass: &BassVoice,
    walking: &BassWalking,
    next_id: &mut impl FnMut() -> NoteId,
) -> Vec<TimelineNote> {
    let pulses = bar.chords.len() * bar.steps_per_chord / bar.steps_per_pulse;
    let chord_at = |pulse: usize| bar.chords[pulse * bar.steps_per_pulse / bar.steps_per_chord];
    let line = Line {
        bar,
        bass,
        walking,
        window: (bass.walk_low.unwrap_or(bass.low), bass.walk_high.unwrap_or(bass.high)),
    };
    // Pulsations qui sonnent : une sur `attack_pulses`, plus chaque arrivée. Les notes de la
    // ligne se comptent par attaque, pas par pulsation.
    let stride = bar.attack_pulses.max(1);
    let slots: Vec<usize> =
        (0..pulses).filter(|&p| p % stride == 0 || chord_at(p) != chord_at(p - 1)).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(mix(bar.seed, 0x5741_4C4B));
    let mut pitches = vec![0u8; slots.len()];
    let mut arrivals = vec![false; slots.len()];
    let mut start = 0;
    while start < slots.len() {
        let pulse = slots[start];
        let end_pulse =
            (pulse + 1..pulses).find(|&p| chord_at(p) != chord_at(p - 1)).unwrap_or(pulses);
        let end = slots.partition_point(|&p| p < end_pulse);
        arrivals[start] = true;
        let goal = if end_pulse < pulses {
            Some(line.target(chord_at(end_pulse)))
        } else {
            bar.next_first.map(|c| line.target(c))
        };
        pitches[start] = line.target(chord_at(pulse));
        let last = end - 1;
        // Attaque d'approche : la dernière de l'étendue, si elle n'est pas l'arrivée.
        let mut approach = None;
        if last > start
            && let Some(goal) = goal
        {
            let approach_weights = &walking.approach;
            let weights = [
                u64::from(approach_weights.none_weight),
                u64::from(approach_weights.chromatic_weight),
                u64::from(approach_weights.diatonic_weight),
            ];
            if let Some(kind @ 1..) = draw(&weights, &mut rng) {
                approach = line.approach(pitches[start], goal, kind == 1);
            }
        }
        let (stop, aim) = match approach {
            Some(pitch) => {
                pitches[last] = pitch;
                (last, Some((pitch, last)))
            }
            None => (end, goal.map(|g| (g, end))),
        };
        for slot in start + 1..stop {
            let beats_left = aim.map(|(pitch, beat)| (pitch, beat - slot));
            let before = slot.checked_sub(2).map(|i| pitches[i]);
            pitches[slot] = line.passing(
                pitches[slot - 1],
                before,
                chord_at(slots[slot]),
                beats_left,
                &mut rng,
            );
        }
        start = end;
    }
    slots
        .iter()
        .enumerate()
        .map(|(slot, &pulse)| {
            let next = slots.get(slot + 1).copied().unwrap_or(pulses);
            TimelineNote {
                id: next_id(),
                pitch: pitches[slot],
                start_step: pulse * bar.steps_per_pulse,
                // Tenue jusqu'à l'attaque suivante, moins le souffle d'une pulsation.
                duration_steps: (next - pulse - 1) * bar.steps_per_pulse + walking.duration,
                velocity: walking.velocity + if arrivals[slot] { walking.accent } else { 0 },
                articulation: Articulation::Normal,
                performance: Default::default(),
            }
        })
        .collect()
}

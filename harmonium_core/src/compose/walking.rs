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
//! Options de données (`accompaniment.bass.walking`), toutes absentes dans les profils existants :
//!
//! - `arrival` : l'arrivée joue la tierce, la quinte ou la septième de l'accord plutôt que sa
//!   basse. Un accord slash, une pédale ou un accord sans ce degré gardent la basse publiée
//!   (D4). Tirée avec la graine de la mesure et le rang de la pulsation, indépendamment du
//!   reste de la ligne : la mesure précédente retrouve la note d'arrivée de la suivante (graine
//!   de la mesure suivante, pulsation 0) pour y mener son approche ;
//! - `approach.side` : côté de l'approche tiré (côté de l'arrivée précédente, côté opposé,
//!   toujours au-dessus, toujours dessous) ;
//! - `passing.chromatic_weight`, `passing.repeat_weight` : chromatisme de passage (un demi-ton
//!   hors de la gamme et de l'accord) et note répétée, à côté des notes de l'accord et des pas
//!   de gamme ;
//! - `eighths` : après la note d'une pulsation dont la suivante tombe à la pulsation suivante, une
//!   croche de passage (approche de la note suivante) ou une note fantôme (même hauteur, légère) ;
//! - `two_feel` : à deux temps, la deuxième attaque d'une étendue d'accord est la quinte de
//!   l'accord (fondamentale-quinte) plutôt qu'une note de passage.
//!
//! Aucune option n'ajoute de tirage au flux de la ligne : elle ne change que ce qu'elle déclare.
//! Une mesure ne dépend que de ses accords, du premier accord de la suivante et de ses graines :
//! elle se rejoue à l'identique, sans état d'une mesure à l'autre.

use arrayvec::ArrayVec;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::{
    accomp::place_in_range,
    accomp_profile::{BassVoice, BassWalking},
    harmony::{ChordQuality, DegreeChord},
    mode::Mode,
};
use crate::{
    algo::RngCore,
    ornaments::mix,
    theory::PitchClass,
    timeline::{Articulation, NoteId, TimelineNote},
};

/// Candidat d'une note de passage.
#[derive(Clone, Copy)]
struct Candidate {
    pitch: u8,
    /// Note de l'accord en cours.
    chord_tone: bool,
    /// Pas diatonique voisin de la note précédente.
    scale_step: bool,
    /// Demi-ton voisin hors de la gamme et de l'accord.
    chromatic: bool,
    /// La note précédente rejouée.
    repeat: bool,
}

type Pool = ArrayVec<Candidate, 32>;

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
    /// Graine de la mesure suivante : l'arrivée sur son premier accord (`walking.arrival`).
    pub next_seed: u64,
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

/// Degrés d'arrivée autres que la basse (`walking.arrival`).
#[derive(Clone, Copy)]
enum Degree {
    Third,
    Fifth,
    Seventh,
}

/// Classe de hauteur du degré dans l'accord publié ; `None` s'il n'y en a pas (pédale, sus, triade
/// pour une septième).
fn degree_pc(
    chord: DegreeChord,
    key: PitchClass,
    mode: Mode,
    degree: Degree,
) -> Option<PitchClass> {
    if matches!(chord.quality, ChordQuality::Pedal | ChordQuality::PedalNeighbor(_)) {
        return None;
    }
    let root = chord.root_pc(key, mode);
    let tones = chord.pitch_classes(key, mode);
    let wanted: &[u8] = match degree {
        Degree::Third => &[4, 3],
        Degree::Fifth => &[7, 6, 8],
        Degree::Seventh => &[10, 11],
    };
    wanted
        .iter()
        .find(|&&interval| tones.iter().any(|&pc| (pc + 12 - root) % 12 == interval))
        .map(|&interval| (root + interval) % 12)
}

struct Line<'a> {
    bar: &'a WalkingBar<'a>,
    bass: &'a BassVoice,
    walking: &'a BassWalking,
    window: (u8, u8),
}

impl Line<'_> {
    /// Basse publiée de l'accord, placée dans le registre d'arrivée.
    fn target(&self, chord: DegreeChord) -> u8 {
        place_in_range(chord.bass_pc(self.bar.key, self.bar.mode), self.bass.low, self.bass.high)
    }

    /// Note d'arrivée sur `chord` à la pulsation `pulse` de la mesure de graine `seed` : sa basse
    /// publiée, ou, avec `walking.arrival`, la tierce, la quinte ou la septième tirées.
    fn arrival(&self, chord: DegreeChord, seed: u64, pulse: usize) -> u8 {
        let bass = self.target(chord);
        let Some(arrival) = &self.walking.arrival else {
            return bass;
        };
        let (key, mode) = (self.bar.key, self.bar.mode);
        if chord.is_slash(mode) {
            return bass;
        }
        let others = [
            (degree_pc(chord, key, mode, Degree::Third), arrival.third_weight),
            (degree_pc(chord, key, mode, Degree::Fifth), arrival.fifth_weight),
            (degree_pc(chord, key, mode, Degree::Seventh), arrival.seventh_weight),
        ];
        // Un degré absent de l'accord n'a pas de poids ; la basse (rang 0) en garde un.
        let weights = [
            u64::from(arrival.root_weight),
            others[0].0.map_or(0, |_| u64::from(others[0].1)),
            others[1].0.map_or(0, |_| u64::from(others[1].1)),
            others[2].0.map_or(0, |_| u64::from(others[2].1)),
        ];
        let mut rng = ChaCha8Rng::seed_from_u64(mix(seed, 0x4152_5256 ^ pulse as u64));
        match draw(&weights, &mut rng) {
            Some(rank @ 1..) => others[rank - 1]
                .0
                .map_or(bass, |pc| place_in_range(pc, self.bass.low, self.bass.high)),
            _ => bass,
        }
    }

    fn in_window(&self, pitch: i16) -> Option<u8> {
        u8::try_from(pitch).ok().filter(|p| (self.window.0..=self.window.1).contains(p))
    }

    /// Note d'approche de `goal`, du côté de `from` (l'arrivée précédente), puis de l'autre côté
    /// si la fenêtre ne le permet pas. Chromatique : un demi-ton. Diatonique : la première
    /// note de la gamme du mode (à un, deux ou trois demi-tons). Avec `approach.side`, le côté
    /// est tiré dans `rng`.
    fn approach(&self, from: u8, goal: u8, chromatic: bool, rng: &mut ChaCha8Rng) -> Option<u8> {
        let side_of_from = from > goal;
        let from_above = match &self.walking.approach.side {
            None => side_of_from,
            Some(side) => {
                let weights = [
                    u64::from(side.from_weight),
                    u64::from(side.other_weight),
                    u64::from(side.above_weight),
                    u64::from(side.below_weight),
                ];
                match draw(&weights, rng) {
                    Some(1) => !side_of_from,
                    Some(2) => true,
                    Some(3) => false,
                    _ => side_of_from,
                }
            }
        };
        [from_above, !from_above].into_iter().find_map(|above| {
            (1..=if chromatic { 1 } else { 3 }).find_map(|distance| {
                let signed = if above { distance } else { -distance };
                let pitch = self.in_window(i16::from(goal) + signed)?;
                (chromatic || self.bar.mode.contains_midi(self.bar.key, pitch)).then_some(pitch)
            })
        })
    }

    /// Quinte de `chord` la plus proche de `current` dans la fenêtre de marche.
    fn fifth(&self, chord: DegreeChord, current: u8) -> Option<u8> {
        let pc = degree_pc(chord, self.bar.key, self.bar.mode, Degree::Fifth)?;
        (self.window.0..=self.window.1)
            .filter(|&p| p % 12 == pc)
            .min_by_key(|&p| (p.abs_diff(current), p))
    }

    /// Note de passage depuis `current` : un tirage pondéré entre notes de l'accord, pas
    /// diatoniques, chromatisme et note répétée, parmi celles qui laissent la cible atteignable
    /// (`beats_left` bonds de `max_leap` au plus). `goal` : hauteur à atteindre et nombre de
    /// pulsations restantes ; `before` : la note qui précède `current` (la ligne n'y retourne pas
    /// tant qu'un autre choix existe).
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
        let passing = &self.walking.passing;
        let chord_pcs = chord.pitch_classes(key, mode);
        let up = (current.saturating_add(1)..=self.window.1).find(|&p| mode.contains_midi(key, p));
        let down = (self.window.0..current).rev().find(|&p| mode.contains_midi(key, p));
        let low = self.window.0.max(current.saturating_sub(max_leap));
        let high = self.window.1.min(current.saturating_add(max_leap));
        let all: Pool = (low..=high)
            .filter(|&p| p != current)
            .filter_map(|p| {
                let chord_tone = chord_pcs.contains(&(p % 12));
                let scale_step = Some(p) == up || Some(p) == down;
                (chord_tone || scale_step).then_some(Candidate {
                    pitch: p,
                    chord_tone,
                    scale_step,
                    chromatic: false,
                    repeat: false,
                })
            })
            .chain(
                [current.checked_sub(1), current.checked_add(1)]
                    .into_iter()
                    .flatten()
                    .filter(|&p| passing.chromatic_weight > 0 && (low..=high).contains(&p))
                    .filter(|&p| !chord_pcs.contains(&(p % 12)) && !mode.contains_midi(key, p))
                    .map(|pitch| Candidate {
                        pitch,
                        chord_tone: false,
                        scale_step: false,
                        chromatic: true,
                        repeat: false,
                    }),
            )
            .chain((passing.repeat_weight > 0).then_some(Candidate {
                pitch: current,
                chord_tone: false,
                scale_step: false,
                chromatic: false,
                repeat: true,
            }))
            .collect();
        let reaches = |p: u8| {
            goal.is_none_or(|(pitch, beats)| {
                usize::from(p.abs_diff(pitch)) <= beats * usize::from(max_leap)
            })
        };
        let reached: Pool = all.iter().copied().filter(|c| reaches(c.pitch)).collect();
        // Genre de note tiré selon les poids : d'abord parmi les notes qui laissent la cible
        // atteignable, sinon parmi toutes ; à défaut de genre pondéré, n'importe laquelle.
        let mut pool = [&reached, &all]
            .into_iter()
            .find_map(|candidates| {
                let weight =
                    |present: bool, weight: u32| if present { u64::from(weight) } else { 0 };
                let kinds = [
                    weight(candidates.iter().any(|c| c.chord_tone), passing.chord_tone_weight),
                    weight(candidates.iter().any(|c| c.scale_step), passing.scale_step_weight),
                    weight(candidates.iter().any(|c| c.chromatic), passing.chromatic_weight),
                    weight(candidates.iter().any(|c| c.repeat), passing.repeat_weight),
                ];
                let kind = draw(&kinds, rng)?;
                Some(
                    candidates
                        .iter()
                        .copied()
                        .filter(|c| match kind {
                            0 => c.chord_tone,
                            1 => c.scale_step,
                            2 => c.chromatic,
                            _ => c.repeat,
                        })
                        .collect(),
                )
            })
            .unwrap_or_else(|| if reached.is_empty() { all.clone() } else { reached.clone() });
        // Pas de retour sur la note d'avant tant que le genre tiré offre un autre choix.
        if let Some(back) = before {
            let forward: Pool = pool.iter().copied().filter(|c| c.pitch != back).collect();
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
            .map(|c| {
                let p = c.pitch;
                let toward = goal.is_some_and(|(g, _)| p.abs_diff(g) < current.abs_diff(g));
                let keeps = before.is_some_and(|b| (p > current) == (current > b));
                1 + if toward { toward_bonus } else { 0 } + if keeps { keep_bonus } else { 0 }
            })
            .collect();
        draw(&weights, rng).map_or(pool[0].pitch, |i| pool[i].pitch)
    }
}

/// Note ajoutée entre deux pulsations : hauteur et vélocité.
struct Extra {
    pitch: u8,
    velocity: u8,
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
    let mut approaches = vec![false; slots.len()];
    // Cible après la mesure : l'arrivée de la mesure suivante, avec sa propre graine.
    let next_goal = bar.next_first.map(|c| line.arrival(c, bar.next_seed, 0));
    let mut start = 0;
    while start < slots.len() {
        let pulse = slots[start];
        let end_pulse =
            (pulse + 1..pulses).find(|&p| chord_at(p) != chord_at(p - 1)).unwrap_or(pulses);
        let end = slots.partition_point(|&p| p < end_pulse);
        arrivals[start] = true;
        let goal = if end_pulse < pulses {
            Some(line.arrival(chord_at(end_pulse), bar.seed, end_pulse))
        } else {
            next_goal
        };
        pitches[start] = line.arrival(chord_at(pulse), bar.seed, pulse);
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
                approach = line.approach(pitches[start], goal, kind == 1, &mut rng);
            }
        }
        let (stop, aim) = match approach {
            Some(pitch) => {
                pitches[last] = pitch;
                approaches[last] = true;
                (last, Some((pitch, last)))
            }
            None => (end, goal.map(|g| (g, end))),
        };
        for slot in start + 1..stop {
            let beats_left = aim.map(|(pitch, beat)| (pitch, beat - slot));
            let before = slot.checked_sub(2).map(|i| pitches[i]);
            let chord = chord_at(slots[slot]);
            let fifth = walking
                .two_feel
                .as_ref()
                .filter(|_| stride > 1)
                .and_then(|two_feel| {
                    let weights =
                        [u64::from(two_feel.passing_weight), u64::from(two_feel.fifth_weight)];
                    (draw(&weights, &mut rng) == Some(1))
                        .then(|| line.fifth(chord, pitches[slot - 1]))
                })
                .flatten();
            pitches[slot] = fifth.unwrap_or_else(|| {
                line.passing(pitches[slot - 1], before, chord, beats_left, &mut rng)
            });
        }
        start = end;
    }
    // Croches entre deux pulsations : seulement quand la note suivante tombe à la pulsation
    // suivante (ou, après la dernière pulsation, sur l'arrivée de la mesure suivante).
    let extras: Vec<Option<Extra>> = match &walking.eighths {
        Some(eighths) if bar.steps_per_pulse >= 2 => {
            let mut eighth_rng = ChaCha8Rng::seed_from_u64(mix(bar.seed, 0x4549_4748));
            (0..slots.len())
                .map(|slot| {
                    let pulse = slots[slot];
                    let next = match slots.get(slot + 1) {
                        Some(&p) => (p == pulse + 1).then(|| pitches[slot + 1]),
                        None => next_goal.filter(|_| pulse + 1 == pulses),
                    }?;
                    if approaches[slot] {
                        return None;
                    }
                    let weights = [
                        u64::from(eighths.none_weight),
                        u64::from(eighths.chromatic_weight),
                        u64::from(eighths.diatonic_weight),
                        u64::from(eighths.ghost_weight),
                    ];
                    match draw(&weights, &mut eighth_rng)? {
                        kind @ (1 | 2) => line
                            .approach(pitches[slot], next, kind == 1, &mut eighth_rng)
                            .filter(|&pitch| pitch != pitches[slot])
                            .map(|pitch| Extra { pitch, velocity: eighths.velocity }),
                        3 => Some(Extra { pitch: pitches[slot], velocity: eighths.ghost_velocity }),
                        _ => None,
                    }
                })
                .collect()
        }
        _ => vec![],
    };
    let mut notes = Vec::with_capacity(slots.len() + extras.iter().flatten().count());
    for (slot, &pulse) in slots.iter().enumerate() {
        let next = slots.get(slot + 1).copied().unwrap_or(pulses);
        let extra = extras.get(slot).and_then(Option::as_ref);
        let half = bar.steps_per_pulse / 2;
        notes.push(TimelineNote {
            id: next_id(),
            pitch: pitches[slot],
            start_step: pulse * bar.steps_per_pulse,
            // Tenue jusqu'à l'attaque suivante, moins le souffle d'une pulsation ; la croche
            // qui suit coupe la note à la demi-pulsation.
            duration_steps: if extra.is_some() {
                half
            } else {
                (next - pulse - 1) * bar.steps_per_pulse + walking.duration
            },
            velocity: walking.velocity + if arrivals[slot] { walking.accent } else { 0 },
            articulation: Articulation::Normal,
            performance: Default::default(),
        });
        if let Some(extra) = extra {
            notes.push(TimelineNote {
                id: next_id(),
                pitch: extra.pitch,
                start_step: pulse * bar.steps_per_pulse + half,
                duration_steps: walking.duration.min(half),
                velocity: extra.velocity,
                articulation: Articulation::Normal,
                performance: Default::default(),
            });
        }
    }
    notes
}

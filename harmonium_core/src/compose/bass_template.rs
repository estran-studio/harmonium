//! Gabarits de basse : un rythme d'une mesure et une règle de hauteur, réalisés sur l'accord
//! courant au pas près (`accompaniment.bass.templates`).
//!
//! Aucun état d'une mesure à l'autre : la hauteur d'une attaque ne dépend que des accords de la
//! mesure, du premier accord de la suivante et du rang de la mesure dans le cycle du gabarit,
//! donc une mesure se rejoue à l'identique et aucun tirage n'a lieu à la réalisation (le gabarit
//! est tiré une fois par air par `BassMotifPlan`, avec les motifs écrits).
//!
//! - **Hauteur** : l'attaque `n` de la mesure `b` du cycle joue `cycle[(b·attaques + n) mod
//!   longueur]` : un degré de l'accord du pas (basse, quinte de la fondamentale, quinte si
//!   l'accord est tenu) et une octave relative, placés dans `low..=high`. Sans `cycle`, la basse
//!   de l'accord.
//! - **Montée** (`ramp`) : `stride × b` demi-tons ou pas de gamme ajoutés à la hauteur, `b` = rang
//!   de la mesure dans le cycle (`bars` mesures, puis la montée repart).
//! - **Relance** (`pickup`) : avant un changement d'accord, une note courte joue la basse de
//!   l'accord suivant `lead` pas plus tôt ; les attaques qu'elle recouvre sont retirées ou
//!   raccourcies.

use super::{
    accomp::place_in_range,
    accomp_profile::{
        BassDegree, BassTemplate, BassVoice, PickupWhen, RampMotion, TemplateNote, TemplatePickup,
        TemplateRamp,
    },
    harmony::DegreeChord,
    mode::Mode,
};
use crate::{
    theory::PitchClass,
    timeline::{Articulation, NoteId, TimelineNote},
};

/// Grille d'une mesure de gabarit.
pub(crate) struct TemplateBar<'a> {
    /// Un accord par créneau `steps_per_chord`.
    pub chords: &'a [DegreeChord],
    /// Premier accord de la mesure suivante ; `None` en fin de parcours (aucune relance).
    pub next_first: Option<DegreeChord>,
    pub key: PitchClass,
    pub mode: Mode,
    pub steps_per_chord: usize,
    pub steps_per_bar: usize,
    /// Rang de la mesure dans le cycle du gabarit (`0..template.bars`).
    pub bar_in_cycle: usize,
}

/// Hauteur d'un degré de `chord` placée dans la fenêtre de la basse, `octave` comprise.
/// `held` : l'accord du créneau précédent est le même (quinte conditionnelle).
fn degree_pitch(
    degree: BassDegree,
    octave: i8,
    chord: DegreeChord,
    held: bool,
    key: PitchClass,
    mode: Mode,
    bass: &BassVoice,
) -> i16 {
    let fifth = degree == BassDegree::Fifth || (degree == BassDegree::FifthIfHeld && held);
    let pc = if fifth { (chord.root_pc(key, mode) + 7) % 12 } else { chord.bass_pc(key, mode) };
    i16::from(place_in_range(pc, bass.low, bass.high)) + 12 * i16::from(octave)
}

/// Décale `pitch` de `bar_in_cycle` fois le pas de la montée.
fn ramped(pitch: i16, ramp: TemplateRamp, bar_in_cycle: usize, key: PitchClass, mode: Mode) -> i16 {
    let bars = i16::try_from(bar_in_cycle).unwrap_or(0);
    match ramp.motion {
        RampMotion::Semitone => pitch + bars * i16::from(ramp.stride),
        RampMotion::ScaleStep => {
            let direction = i16::from(ramp.stride.signum());
            let mut pitch = pitch;
            for _ in 0..bars * i16::from(ramp.stride.unsigned_abs()) {
                loop {
                    pitch += direction;
                    if !(0..=127).contains(&pitch) || mode.contains_midi(key, pitch as u8) {
                        break;
                    }
                }
            }
            pitch
        }
    }
}

/// Hauteur de l'attaque `n` (rang dans la mesure) du gabarit à la mesure `bar_in_cycle`.
fn hit_pitch(
    template: &BassTemplate,
    bar: &TemplateBar<'_>,
    bass: &BassVoice,
    n: usize,
    step: usize,
) -> u8 {
    let slot = step / bar.steps_per_chord;
    let note = if template.cycle.is_empty() {
        TemplateNote { degree: BassDegree::Root, octave: 0 }
    } else {
        template.cycle[(bar.bar_in_cycle * template.rhythm.len() + n) % template.cycle.len()]
    };
    let held = slot == 0 || bar.chords[slot] == bar.chords[slot - 1];
    let mut pitch =
        degree_pitch(note.degree, note.octave, bar.chords[slot], held, bar.key, bar.mode, bass);
    if let Some(ramp) = template.ramp {
        pitch = ramped(pitch, ramp, bar.bar_in_cycle, bar.key, bar.mode);
    }
    // La validation garantit 0..=127 pour toutes les classes de hauteur.
    pitch.clamp(0, 127) as u8
}

/// Relance posée avant la barre `boundary` (pas de la mesure) sur l'accord `next`.
struct Pickup {
    start: usize,
    boundary: usize,
    chord: DegreeChord,
}

/// Relances d'une mesure selon `pickup.when`.
fn pickups(pickup: TemplatePickup, bar: &TemplateBar<'_>) -> Vec<Pickup> {
    let mut found = Vec::new();
    if pickup.when == PickupWhen::ChordChange {
        for slot in 1..bar.chords.len() {
            if bar.chords[slot] != bar.chords[slot - 1] {
                let boundary = slot * bar.steps_per_chord;
                found.push(Pickup {
                    start: boundary - pickup.lead,
                    boundary,
                    chord: bar.chords[slot],
                });
            }
        }
    }
    if let Some(next) = bar.next_first {
        let last = bar.chords[bar.chords.len() - 1];
        if pickup.when == PickupWhen::BarEnd || next != last {
            found.push(Pickup {
                start: bar.steps_per_bar - pickup.lead,
                boundary: bar.steps_per_bar,
                chord: next,
            });
        }
    }
    found
}

/// Basse d'une mesure d'un gabarit.
#[must_use]
pub(crate) fn bass_template_bar(
    template: &BassTemplate,
    bar: &TemplateBar<'_>,
    bass: &BassVoice,
    next_id: &mut impl FnMut() -> NoteId,
) -> Vec<TimelineNote> {
    // (début, durée, hauteur, vélocité)
    let mut hits: Vec<(usize, usize, u8, u8)> = template
        .rhythm
        .iter()
        .enumerate()
        .map(|(n, hit)| {
            (hit.step, hit.duration, hit_pitch(template, bar, bass, n, hit.step), hit.velocity)
        })
        .collect();
    if let Some(pickup) = template.pickup {
        for relance in pickups(pickup, bar) {
            hits.retain_mut(|(start, duration, ..)| {
                if *start >= relance.start && *start < relance.boundary {
                    return false;
                }
                if *start < relance.start && *start + *duration > relance.start {
                    *duration = relance.start - *start;
                }
                true
            });
            let pitch =
                degree_pitch(pickup.degree, 0, relance.chord, false, bar.key, bar.mode, bass);
            hits.push((relance.start, pickup.duration, pitch.clamp(0, 127) as u8, pickup.velocity));
        }
        hits.sort_by_key(|hit| hit.0);
    }
    hits.into_iter()
        .map(|(start_step, duration_steps, pitch, velocity)| TimelineNote {
            id: next_id(),
            pitch,
            start_step,
            duration_steps,
            velocity,
            articulation: Articulation::Normal,
            performance: Default::default(),
        })
        .collect()
}

/// Première attaque (pas, hauteur) d'une mesure de gabarit, pour l'approche de la mesure
/// précédente (`bass.passing`). Les relances ne comptent pas : elles ne commencent jamais au
/// premier pas.
pub(crate) fn first_attack(
    bass: &BassVoice,
    template_index: usize,
    bar_in_cycle: usize,
    chords: &[DegreeChord],
    key: PitchClass,
    mode: Mode,
    steps_per_chord: usize,
) -> Option<(usize, u8)> {
    let template = &bass.templates[template_index];
    let first = template.rhythm.first()?;
    let bar = TemplateBar {
        chords,
        next_first: None,
        key,
        mode,
        steps_per_chord,
        steps_per_bar: 0,
        bar_in_cycle,
    };
    Some((first.step, hit_pitch(template, &bar, bass, 0, first.step)))
}

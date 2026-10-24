//! Approches des cibles de basse, préparées avant dynamique/interprétation.
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::{
    Mode,
    accomp_profile::{BassPassing, BassVoice, PassingMotion, PassingRule, PassingWhen},
};
use crate::{
    algo::RngCore,
    ornaments::mix,
    timeline::{Articulation, Measure, NoteId, TimelineNote, TrackId},
};

fn approach_pitch(
    rule: &PassingRule,
    index: usize,
    from: u8,
    target: u8,
    key: u8,
    mode: Mode,
) -> Option<u8> {
    let direction = if target > from { -1i16 } else { 1 };
    let count = rule.before.len() - index;
    let mut pitch = i16::from(target);
    for _ in 0..count {
        loop {
            pitch += direction;
            if !(0..=127).contains(&pitch) {
                return None;
            }
            if rule.motion == PassingMotion::Chromatic || mode.contains_midi(key, pitch as u8) {
                break;
            }
        }
    }
    let pitch = pitch as u8;
    // Pas de détour, de note répétée, ni de faux « passage » au-delà du départ.
    ((from < pitch && pitch < target) || (target < pitch && pitch < from)).then_some(pitch)
}

pub(crate) fn clashes(notes: &[TimelineNote], pitch: u8, onset: i64, avoid: &[u8]) -> bool {
    notes.iter().any(|n| {
        let start = n.start_step as i64 * 1000 + i64::from(n.performance.offset);
        start <= onset
            && onset < start + n.performance.duration(n.duration_steps)
            && avoid.contains(&((i16::from(n.pitch) - i16::from(pitch)).rem_euclid(12) as u8))
    })
}

pub(crate) struct PassingContext<'a> {
    pub key: u8,
    pub mode: Mode,
    pub seed: u64,
    pub bar: usize,
    pub phase: f32,
    pub phrase_bars: f32,
    pub density: f32,
    pub arousal: f32,
    pub strong_steps: &'a [usize],
    pub chords: &'a [super::TradChord],
    pub steps_per_chord: usize,
    /// Première basse de la mesure suivante, dans sa couleur harmonique active.
    pub next_target: Option<(usize, u8)>,
}

#[allow(clippy::too_many_arguments)]
fn eligible(
    rule: &PassingRule,
    from: &TimelineNote,
    target: (usize, u8),
    bass: &BassVoice,
    melody: &[TimelineNote],
    passing: &BassPassing,
    context: &PassingContext<'_>,
    steps: usize,
) -> bool {
    let (target_step, target_pitch) = target;
    if rule.weight == 0
        || from.pitch.abs_diff(target_pitch) < rule.min_leap
        || from.pitch == target_pitch
    {
        return false;
    }
    if rule.when == PassingWhen::ChordChangeInsideBar
        && (target_step >= steps
            || target_step % context.steps_per_chord != 0
            || target_step < context.steps_per_chord
            || context.chords[target_step / context.steps_per_chord]
                == context.chords[target_step / context.steps_per_chord - 1])
    {
        return false;
    }
    rule.before.iter().enumerate().all(|(i, &before)| {
        let Some(step) = target_step.checked_sub(before) else {
            return false;
        };
        let Some(pitch) =
            approach_pitch(rule, i, from.pitch, target_pitch, context.key, context.mode)
        else {
            return false;
        };
        step > from.start_step
            && step < steps
            && !bass.hits.iter().any(|h| h.step == step)
            && !context.strong_steps.contains(&step)
            && (bass.walk_low.unwrap_or(bass.low)..=bass.walk_high.unwrap_or(bass.high))
                .contains(&pitch)
            && !clashes(melody, pitch, step as i64 * 1000, &passing.avoid_melody_intervals)
    })
}

pub(crate) fn apply(
    measure: &mut Measure,
    bass: &BassVoice,
    context: &PassingContext<'_>,
    next_id: &mut impl FnMut() -> NoteId,
) {
    let Some(passing) = bass.passing.as_ref().filter(|p| !p.is_inert()) else {
        return;
    };
    let arousal = passing.arousal_low_multiplier
        + context.arousal.clamp(0.0, 1.0)
            * (passing.arousal_high_multiplier - passing.arousal_low_multiplier);
    let density = passing.density.eval(context.density) * arousal;
    if density == 0.0 {
        return;
    }
    let Some(slot) = measure.tracks.get_mut(&TrackId::Bass) else {
        return;
    };
    let mut notes = std::mem::take(slot);
    let melody = measure.notes_for_track(TrackId::Melody);
    let original = notes.len();
    let mut cursor = 0;
    while cursor < original {
        let index = cursor;
        let count = notes[index..original]
            .iter()
            .take_while(|n| n.start_step == notes[index].start_step)
            .count();
        cursor += count;
        let target = if cursor < original {
            Some((notes[cursor].start_step, notes[cursor].pitch))
        } else {
            context.next_target.map(|(step, pitch)| (measure.steps + step, pitch))
        };
        let Some(target) = target else {
            continue;
        };
        let from = &notes[index];
        let scale = density
            * passing.phrase.eval(
                context.phase + from.start_step as f32 / measure.steps as f32 / context.phrase_bars,
            );
        if scale == 0.0 {
            continue;
        }
        let valid = |r: &&PassingRule| {
            eligible(r, from, target, bass, melody, passing, context, measure.steps)
        };
        let weight: f32 = passing.rules.iter().filter(valid).map(|r| r.weight as f32).sum();
        if weight == 0.0 {
            continue;
        }
        let mut rng = ChaCha8Rng::seed_from_u64(mix(
            mix(context.seed, 0x4241_5353),
            mix(context.bar as u64, from.start_step as u64),
        ));
        let mut pick = rng.next_f32() * (passing.none_weight as f32 + weight * scale)
            - passing.none_weight as f32;
        if pick < 0.0 {
            continue;
        }
        let selected = passing.rules.iter().filter(valid).find(|r| {
            pick -= r.weight as f32 * scale;
            pick < 0.0
        });
        let Some(rule) = selected else {
            continue;
        };
        let velocity = (i16::from(from.velocity) + rule.velocity_delta).clamp(1, 127) as u8;
        let from_pitch = from.pitch;
        let first_step = target.0 - rule.before[0];
        for note in &mut notes[index..cursor] {
            note.duration_steps = note.duration_steps.min(first_step - note.start_step);
        }
        for (i, &before) in rule.before.iter().enumerate() {
            let Some(pitch) =
                approach_pitch(rule, i, from_pitch, target.1, context.key, context.mode)
            else {
                continue;
            };
            notes.push(TimelineNote {
                id: next_id(),
                pitch,
                start_step: target.0 - before,
                duration_steps: rule.duration,
                velocity,
                articulation: Articulation::Normal,
                performance: Default::default(),
            });
        }
    }
    notes.sort_unstable_by_key(|n| (n.start_step, n.id));
    if let Some(slot) = measure.tracks.get_mut(&TrackId::Bass) {
        *slot = notes;
    }
}

/// Les ornements peuvent ajouter une voix mélodique : leur veto est lu dans le flux final.
pub(crate) fn veto_final(measure: &mut Measure, bass: &BassVoice) {
    let Some(passing) = &bass.passing else {
        return;
    };
    let Some(slot) = measure.tracks.get_mut(&TrackId::Bass) else {
        return;
    };
    let mut notes = std::mem::take(slot);
    let melody = measure.notes_for_track(TrackId::Melody);
    notes.retain(|n| {
        bass.hits.iter().any(|h| h.step == n.start_step)
            || !clashes(
                melody,
                n.pitch,
                n.start_step as i64 * 1000 + i64::from(n.performance.offset),
                &passing.avoid_melody_intervals,
            )
    });
    if let Some(slot) = measure.tracks.get_mut(&TrackId::Bass) {
        *slot = notes;
    }
}

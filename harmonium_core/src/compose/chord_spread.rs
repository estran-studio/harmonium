//! Geste d'accord : tirages dérivés par attaque, composition après V2.
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::accomp_profile::{AttackDirection, ChordHit, ChordVoice};
use crate::{
    algo::RngCore,
    ornaments::mix,
    timeline::{Measure, TrackId},
};

pub(crate) fn selected(hit: &ChordHit, seed: u64) -> bool {
    if hit.probability >= 1.0 {
        return true;
    }
    if hit.probability <= 0.0 {
        return false;
    }
    let mut rng = ChaCha8Rng::seed_from_u64(mix(mix(seed, 0x4348_4F50), hit.step as u64));
    rng.next_f32() < hit.probability
}

/// Les vélocités écrites précèdent V2 ; un seul RNG de voix par attaque.
pub(crate) fn velocities(voice: &ChordVoice, hit: &ChordHit, count: usize, seed: u64) -> [u8; 6] {
    let mut values = [hit.velocity; 6];
    let Some(spread) = voice.spread.as_ref().filter(|_| hit.direction != AttackDirection::Block)
    else {
        return values;
    };
    let mut rng = (spread.velocity_jitter > 0)
        .then(|| ChaCha8Rng::seed_from_u64(mix(mix(seed, 0x5645_4C4F), hit.step as u64)));
    for (rank, value) in values[..count].iter_mut().enumerate() {
        let slope = f32::from(spread.velocity_slope) * (count - 1 - rank) as f32
            / (count - 1).max(1) as f32;
        let jitter = rng
            .as_mut()
            .map_or(0.0, |r| (r.next_f32() * 2.0 - 1.0) * f32::from(spread.velocity_jitter));
        *value = (f32::from(hit.velocity) + slope + jitter).round().clamp(1.0, 127.0) as u8;
    }
    values
}

pub(crate) fn apply(measure: &mut Measure, voice: &ChordVoice, hits: &[ChordHit], seed: u64) {
    let Some(spread) = &voice.spread else {
        return;
    };
    let step_ms = 60000.0 / f64::from(measure.tempo) / measure.steps_per_quarter();
    let tempo_scale = if spread.tempo_exponent == 0.0 {
        1.0
    } else {
        (f64::from(spread.ref_bpm) / f64::from(measure.tempo))
            .powf(f64::from(spread.tempo_exponent))
    };
    let Some(notes) = measure.tracks.get_mut(&TrackId::Chords) else {
        return;
    };
    let mut index = 0;
    while index < notes.len() {
        let step = notes[index].start_step;
        let count = notes[index..].iter().take_while(|n| n.start_step == step).count();
        let group = &mut notes[index..index + count];
        index += count;
        let Some(hit_index) = hits.iter().position(|h| h.step == step) else {
            continue;
        };
        let hit = &hits[hit_index];
        if hit.direction == AttackDirection::Block || count < 2 {
            continue;
        }
        let gap = f64::from(if hit.direction == AttackDirection::Up {
            spread.up_gap_ms
        } else {
            spread.down_gap_ms
        }) * tempo_scale;
        if gap == 0.0 {
            continue;
        }
        let mut rng = (spread.gap_jitter > 0.0)
            .then(|| ChaCha8Rng::seed_from_u64(mix(mix(seed, 0x5354_5255), step as u64)));
        let mut gaps = [0.0f64; 5];
        for value in &mut gaps[..count - 1] {
            let jitter = rng
                .as_mut()
                .map_or(0.0, |r| f64::from((r.next_f32() * 2.0 - 1.0) * spread.gap_jitter));
            *value = gap * (1.0 + jitter) / step_ms * 1000.0;
        }
        let total_gap: f64 = gaps[..count - 1].iter().sum();
        let next = hits.get(hit_index + 1).map_or(measure.steps + hits[0].step, |h| h.step);
        let base = f64::from(group[0].performance.offset);
        let anchor = f64::from(spread.anchor);
        let mut cap = (next - step) as f64 * 1000.0 * f64::from(spread.max_spread_ratio);
        // Réduire le geste entier, jamais clamper des voix séparément : l'ordre reste strict.
        if anchor > 0.0 {
            cap = cap.min((base + 490.0) / anchor);
        }
        if anchor < 1.0 {
            cap = cap.min((490.0 - base) / (1.0 - anchor));
        }
        let scale = (cap / total_gap).clamp(0.0, 1.0);
        let mut position = -anchor * total_gap * scale;
        for (rank, note) in group.iter_mut().enumerate() {
            note.performance.offset = (base + position).round().clamp(-490.0, 490.0) as i16;
            if rank + 1 < count {
                position += gaps[rank] * scale;
            }
        }
    }
}

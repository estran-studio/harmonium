//! Interprétation en millièmes de pas : indépendante du tempo, grille inchangée.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    algo::RngCore,
    style_profile::ProfileError,
    timeline::{Articulation, Measure, TrackId},
};

/// Intention publiée ; gate 1000 = durée écrite, sans chevauchement ajouté.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotePerformance {
    pub offset: i16,
    pub gate: u16,
    /// Raccourcissement supplémentaire de respiration, en millièmes de pas.
    pub breath: u32,
}
impl Default for NotePerformance {
    fn default() -> Self {
        Self { offset: 0, gate: 1000, breath: 0 }
    }
}
impl NotePerformance {
    pub fn is_neutral(&self) -> bool {
        *self == Self::default()
    }
    pub fn duration(&self, steps: usize) -> i64 {
        ((steps.max(1) as i64 * i64::from(self.gate)) - i64::from(self.breath)).max(1)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Performance {
    pub tracks: BTreeMap<String, TrackPerformance>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TrackPerformance {
    /// Une valeur par pas ; vide = zéro partout. Unité : fraction de pas.
    pub offsets: Vec<f32>,
    pub jitter: f32,
    pub rules: Vec<ArticulationRule>,
    /// Points [phase de phrase 0..1, delta de vélocité].
    pub phrase: Vec<[f32; 2]>,
    pub velocity_min: u8,
    pub velocity_max: u8,
    /// Fraction de la dernière durée, soustraite après le gate.
    pub breath: f32,
    pub emotions: PerformanceEmotions,
}
impl TrackPerformance {
    fn is_neutral(&self) -> bool {
        self.offsets.iter().all(|&x| x == 0.0)
            && self.jitter == 0.0
            && self.breath == 0.0
            && self.phrase.iter().all(|point| point[1] == 0.0)
            && self.velocity_min == 0
            && self.velocity_max == 0
            && self.emotions == PerformanceEmotions::default()
            && self.rules.iter().all(|rule| rule.shorten == 0.0 && rule.accent == 0.0)
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PerformanceEmotions {
    pub tension_advance: f32,
    pub tension_detach: f32,
    pub arousal_accent: f32,
    pub valence_velocity: f32,
    pub density_detach: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteContext {
    #[default]
    Any,
    Repeated,
    Ascending,
    Descending,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArticulationRule {
    pub articulation: Articulation,
    pub weight: f32,
    #[serde(default)]
    pub steps: Vec<usize>,
    #[serde(default)]
    pub min_duration: usize,
    #[serde(default)]
    pub max_duration: Option<usize>,
    #[serde(default)]
    pub context: NoteContext,
    /// Fraction retranchée : zéro conserve la durée mécanique.
    #[serde(default)]
    pub shorten: f32,
    #[serde(default)]
    pub accent: f32,
}
fn bounded(value: f32, lo: f32, hi: f32, field: &str) -> Result<(), ProfileError> {
    if value.is_finite() && (lo..=hi).contains(&value) {
        Ok(())
    } else {
        Err(ProfileError::new(field, format!("valeur finie attendue dans {lo}..{hi}")))
    }
}
impl Performance {
    /// Aucun réglage n'agit : le flux est celui d'une partition non interprétée.
    pub fn is_neutral(&self) -> bool {
        self.tracks.values().all(TrackPerformance::is_neutral)
    }

    pub fn validate(&self, steps: usize) -> Result<(), ProfileError> {
        for (track, p) in &self.tracks {
            let field = format!("performance.tracks.{track}");
            if !TrackId::ALL.iter().any(|id| format!("{id:?}") == *track) {
                return Err(ProfileError::new(&field, "piste inconnue"));
            }
            if !p.offsets.is_empty() && p.offsets.len() != steps {
                return Err(ProfileError::new(
                    format!("{field}.offsets"),
                    "une valeur par pas requise",
                ));
            }
            for offset in &p.offsets {
                bounded(*offset, -0.2, 0.2, &format!("{field}.offsets"))?;
            }
            bounded(p.jitter, 0.0, 0.1, &format!("{field}.jitter"))?;
            bounded(p.breath, 0.0, 0.5, &format!("{field}.breath"))?;
            if p.velocity_min > p.velocity_max || p.velocity_max > 127 {
                return Err(ProfileError::new(
                    format!("{field}.velocity_max"),
                    "bornes ordonnées dans 0..127 requises ; 0/0 = sans bornage ajouté",
                ));
            }
            let e = &p.emotions;
            for (name, value, max) in [
                ("tension_advance", e.tension_advance, 0.2),
                ("tension_detach", e.tension_detach, 0.4),
                ("arousal_accent", e.arousal_accent, 20.0),
                ("density_detach", e.density_detach, 0.3),
            ] {
                bounded(value, 0.0, max, &format!("{field}.emotions.{name}"))?;
            }
            bounded(
                e.valence_velocity,
                -10.0,
                10.0,
                &format!("{field}.emotions.valence_velocity"),
            )?;
            for (i, point) in p.phrase.iter().enumerate() {
                bounded(point[0], 0.0, 1.0, &format!("{field}.phrase[{i}].phase"))?;
                bounded(point[1], -20.0, 20.0, &format!("{field}.phrase[{i}].velocity"))?;
                if i > 0 && point[0] <= p.phrase[i - 1][0] {
                    return Err(ProfileError::new(
                        format!("{field}.phrase"),
                        "phases strictement croissantes requises",
                    ));
                }
            }
            for (i, rule) in p.rules.iter().enumerate() {
                let f = format!("{field}.rules[{i}]");
                bounded(rule.weight, 0.0, 1_000_000.0, &format!("{f}.weight"))?;
                bounded(rule.shorten, 0.0, 0.8, &format!("{f}.shorten"))?;
                bounded(rule.accent, 0.0, 20.0, &format!("{f}.accent"))?;
                if rule.steps.iter().any(|&s| s >= steps)
                    || rule.max_duration.is_some_and(|d| d < rule.min_duration)
                {
                    return Err(ProfileError::new(
                        &f,
                        "positions dans la grille et durées ordonnées requises",
                    ));
                }
                if rule.articulation == Articulation::Trigger
                    || rule.articulation == Articulation::Tied
                {
                    return Err(ProfileError::new(
                        format!("{f}.articulation"),
                        "Normal, Detached, Linked ou Accent requis (la liaison vient des coups)",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Toutes les décisions et tous les tirages sont faits avant publication.
    pub fn apply(
        &self,
        measure: &mut Measure,
        phase: f32,
        phrase_bars: f32,
        phrase_end: bool,
        emotions: [f32; 4],
        rng: &mut dyn RngCore,
    ) {
        for track in TrackId::ALL {
            let name = track.name();
            let Some(p) = self.tracks.get(name) else { continue };
            if p.is_neutral() {
                continue;
            }
            let Some(notes) = measure.tracks.get_mut(&track) else { continue };
            let last = notes.iter().map(|n| n.start_step).max();
            let mut previous = None;
            let mut index = 0;
            while index < notes.len() {
                let step = notes[index].start_step;
                let count = notes[index..].iter().take_while(|n| n.start_step == step).count();
                let attack = &mut notes[index..index + count];
                index += count;
                // Le contexte porte sur la voix grave de l'attaque précédente, pas sur ses cordes.
                let pitch = attack.iter().map(|n| n.pitch).min().unwrap_or(0);
                let context =
                    previous.map_or(NoteContext::Any, |previous| match pitch.cmp(&previous) {
                        std::cmp::Ordering::Equal => NoteContext::Repeated,
                        std::cmp::Ordering::Greater => NoteContext::Ascending,
                        std::cmp::Ordering::Less => NoteContext::Descending,
                    });
                previous = Some(pitch);
                let note = &mut attack[0];
                let matches = |r: &&ArticulationRule| {
                    (r.steps.is_empty() || r.steps.contains(&note.start_step))
                        && note.duration_steps >= r.min_duration
                        && r.max_duration.is_none_or(|d| note.duration_steps <= d)
                        && (r.context == NoteContext::Any || r.context == context)
                };
                let total: f32 = p.rules.iter().filter(matches).map(|r| r.weight).sum();
                let mut shorten = 0.0;
                let mut accent = 0.0;
                if total > 0.0 {
                    let mut pick = rng.next_f32() * total;
                    for rule in p.rules.iter().filter(matches) {
                        pick -= rule.weight;
                        if pick < 0.0 {
                            note.articulation = rule.articulation;
                            shorten = rule.shorten;
                            accent = rule.accent;
                            break;
                        }
                    }
                }
                let [arousal, valence, density, tension] = emotions;
                let jitter =
                    if p.jitter > 0.0 { (rng.next_f32() * 2.0 - 1.0) * p.jitter } else { 0.0 };
                let offset = p.offsets.get(note.start_step).copied().unwrap_or(0.0) + jitter
                    - tension * p.emotions.tension_advance;
                note.performance.offset = (offset.clamp(-0.49, 0.49) * 1000.0).round() as i16;
                let shorten = (shorten
                    + tension * p.emotions.tension_detach
                    + density * p.emotions.density_detach)
                    .clamp(0.0, 0.9);
                note.performance.gate = ((1.0 - shorten) * 1000.0).round() as u16;
                if phrase_end && Some(note.start_step) == last && note.duration_steps > 0 {
                    let requested = (note.duration_steps as f32 * p.breath * 1000.0).round() as u32;
                    let available = (note.duration_steps as u32 * u32::from(note.performance.gate))
                        .saturating_sub(1);
                    note.performance.breath = requested.min(available);
                }
                let phrase = curve(
                    &p.phrase,
                    phase + note.start_step as f32 / measure.steps as f32 / phrase_bars,
                );
                let accent = accent
                    + if note.articulation == Articulation::Accent {
                        arousal * p.emotions.arousal_accent
                    } else {
                        0.0
                    };
                let delta = phrase + accent + valence * p.emotions.valence_velocity;
                let velocity = f32::from(note.velocity) + delta;
                let (lo, hi) = if p.velocity_max == 0 {
                    (1, 127)
                } else {
                    (p.velocity_min.max(1), p.velocity_max)
                };
                note.velocity = velocity.round().clamp(f32::from(lo), f32::from(hi)) as u8;
                let performance = note.performance;
                let articulation = note.articulation;
                for other in &mut attack[1..] {
                    other.performance = performance;
                    other.articulation = articulation;
                    other.velocity = (f32::from(other.velocity) + delta)
                        .round()
                        .clamp(f32::from(lo), f32::from(hi))
                        as u8;
                }
            }
        }
    }
}
fn curve(points: &[[f32; 2]], phase: f32) -> f32 {
    let Some(first) = points.first() else { return 0.0 };
    if phase <= first[0] {
        return first[1];
    }
    for pair in points.windows(2) {
        if phase <= pair[1][0] {
            return pair[0][1]
                + (pair[1][1] - pair[0][1]) * (phase - pair[0][0]) / (pair[1][0] - pair[0][0]);
        }
    }
    points.last().map_or(0.0, |p| p[1])
}

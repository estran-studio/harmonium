//! Données et validation des deux voix d'accompagnement.
use serde::{Deserialize, Serialize};

use crate::{style_profile::ProfileError, trad::dynamics::Curve, tuning::TradHarmonyParams};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BassDegree {
    Root,
    Fifth,
    FifthIfHeld,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassHit {
    pub step: usize,
    pub degree: BassDegree,
    pub duration: usize,
    pub velocity: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassVoice {
    pub low: u8,
    pub high: u8,
    pub hits: Vec<BassHit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walk_low: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walk_high: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passing: Option<BassPassing>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChordHit {
    pub step: usize,
    pub duration: usize,
    pub velocity: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChordPattern {
    pub hits: Vec<ChordHit>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChordVoice {
    pub center: u8,
    pub max_drift: u8,
    pub default_pattern: usize,
    pub patterns: Vec<ChordPattern>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassingWhen {
    NextTargetDiffers,
    ChordChangeInsideBar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassingMotion {
    ScaleStep,
    ScaleWalk,
    Chromatic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassingRule {
    pub when: PassingWhen,
    pub before: Vec<usize>,
    pub motion: PassingMotion,
    pub weight: u32,
    pub duration: usize,
    pub velocity_delta: i16,
    pub min_leap: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassPassing {
    pub none_weight: u32,
    pub density: Curve,
    pub phrase: Curve,
    pub avoid_melody_intervals: Vec<u8>,
    #[serde(default = "unit_multiplier")]
    pub arousal_low_multiplier: f32,
    #[serde(default = "unit_multiplier")]
    pub arousal_high_multiplier: f32,
    pub rules: Vec<PassingRule>,
}

fn unit_multiplier() -> f32 {
    1.0
}

impl BassPassing {
    pub fn is_inert(&self) -> bool {
        self.rules.iter().all(|r| r.weight == 0)
            || self.density.0.iter().all(|p| p[1] == 0.0)
            || self.phrase.0.iter().all(|p| p[1] == 0.0)
            || (self.arousal_low_multiplier == 0.0 && self.arousal_high_multiplier == 0.0)
    }
    fn validate(&self, steps: usize) -> Result<(), ProfileError> {
        let field = "accompaniment.bass.passing";
        for (name, curve) in [("density", &self.density), ("phrase", &self.phrase)] {
            curve
                .validate(name, 0.0, 4.0)
                .map_err(|e| ProfileError::new(format!("{field}.{name}"), e))?;
        }
        for (name, value) in [
            ("arousal_low_multiplier", self.arousal_low_multiplier),
            ("arousal_high_multiplier", self.arousal_high_multiplier),
        ] {
            require(
                value.is_finite() && (0.0..=4.0).contains(&value),
                format!("{field}.{name}"),
                "multiplicateur fini dans 0..4 requis",
            )?;
        }
        require(
            self.avoid_melody_intervals.iter().all(|&i| i < 12),
            format!("{field}.avoid_melody_intervals"),
            "intervalles modulo 12 requis",
        )?;
        require(self.rules.len() <= 64, format!("{field}.rules"), "au plus 64 règles")?;
        let mut sum = self.none_weight;
        for (i, rule) in self.rules.iter().enumerate() {
            let f = format!("{field}.rules[{i}]");
            sum = sum.checked_add(rule.weight).ok_or_else(|| {
                ProfileError::new(format!("{f}.weight"), "somme des poids hors u32")
            })?;
            require(
                !rule.before.is_empty()
                    && rule.before.len() <= steps
                    && rule.before.iter().all(|&b| b > 0 && b < steps)
                    && rule.before.windows(2).all(|w| w[0] > w[1]),
                format!("{f}.before"),
                "distances positives décroissantes dans la mesure requises",
            )?;
            require(
                rule.motion == PassingMotion::ScaleWalk || rule.before.len() == 1,
                format!("{f}.before"),
                "scale_step et chromatic demandent une seule note",
            )?;
            require(
                rule.duration > 0
                    && rule.before.last().is_some_and(|&b| rule.duration <= b)
                    && rule.before.windows(2).all(|w| rule.duration <= w[0] - w[1]),
                format!("{f}.duration"),
                "durée positive sans chevauchement de la cible ni des passages requise",
            )?;
            require(
                (-127..=127).contains(&rule.velocity_delta),
                format!("{f}.velocity_delta"),
                "delta dans -127..127 requis",
            )?;
            require(rule.min_leap <= 127, format!("{f}.min_leap"), "intervalle MIDI requis")?;
        }
        require(sum > 0, format!("{field}.none_weight"), "somme des poids positive requise")
    }
}

pub(crate) fn require(
    ok: bool,
    field: impl Into<String>,
    message: &str,
) -> Result<(), ProfileError> {
    if ok { Ok(()) } else { Err(ProfileError::new(field, message)) }
}

fn validate_hits(
    hits: impl Iterator<Item = (usize, usize, u8)>,
    steps: usize,
    field: &str,
) -> Result<(), ProfileError> {
    let mut previous = None;
    for (i, (step, duration, velocity)) in hits.enumerate() {
        let f = format!("{field}[{i}]");
        require(
            step < steps && previous.is_none_or(|p| step > p),
            format!("{f}.step"),
            "pas croissants uniques dans la mesure requis",
        )?;
        require(
            duration > 0 && step.checked_add(duration).is_some_and(|end| end <= steps),
            format!("{f}.duration"),
            "durée positive dans la mesure requise",
        )?;
        require(
            (1..=127).contains(&velocity),
            format!("{f}.velocity"),
            "vélocité dans 1..127 requise",
        )?;
        previous = Some(step);
    }
    Ok(())
}

impl TradHarmonyParams {
    pub(crate) fn validate_accompaniment(
        &self,
        steps: usize,
        jazz: bool,
    ) -> Result<(), ProfileError> {
        require(
            self.strong_weight > 0 && self.weak_weight > 0,
            "accompaniment.strong_weight",
            "poids de vote positifs requis",
        )?;
        require(
            self.strong_steps.iter().all(|&s| s < steps),
            "accompaniment.strong_steps",
            "pas hors mesure",
        )?;
        let b = &self.bass;
        require(
            b.low <= b.high && b.high <= 127,
            "accompaniment.bass.low",
            "registre MIDI ordonné requis",
        )?;
        require(
            !jazz || b.high - b.low >= 11,
            "accompaniment.bass.high",
            "le registre jazz doit couvrir les douze classes de hauteur",
        )?;
        require(
            b.walk_low.unwrap_or(b.low) <= b.low,
            "accompaniment.bass.walk_low",
            "doit être inférieur ou égal à low",
        )?;
        require(
            b.walk_high.unwrap_or(b.high) >= b.high && b.walk_high.unwrap_or(b.high) <= 127,
            "accompaniment.bass.walk_high",
            "doit être supérieur ou égal à high, dans MIDI",
        )?;
        if let Some(passing) = &b.passing {
            require(
                !jazz || passing.is_inert(),
                "accompaniment.bass.passing",
                "les approches modales demandent l'harmonie trad_functional",
            )?;
            passing.validate(steps)?;
        }
        validate_hits(
            b.hits.iter().map(|h| (h.step, h.duration, h.velocity)),
            steps,
            "accompaniment.bass.hits",
        )?;
        let c = &self.chords;
        require(
            c.center <= 127 && c.max_drift <= 127,
            "accompaniment.chords.center",
            "registre MIDI requis",
        )?;
        require(
            c.default_pattern < c.patterns.len(),
            "accompaniment.chords.default_pattern",
            "indice de motif existant requis",
        )?;
        for (i, pattern) in c.patterns.iter().enumerate() {
            let field = format!("accompaniment.chords.patterns[{i}].hits");
            require(!pattern.hits.is_empty(), &field, "motif non vide requis")?;
            validate_hits(
                pattern.hits.iter().map(|h| (h.step, h.duration, h.velocity)),
                steps,
                &field,
            )?;
        }
        Ok(())
    }
}

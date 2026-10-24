//! Données et validation des deux voix d'accompagnement.
use serde::{Deserialize, Serialize};

use crate::{style_profile::ProfileError, tuning::TradHarmonyParams};

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

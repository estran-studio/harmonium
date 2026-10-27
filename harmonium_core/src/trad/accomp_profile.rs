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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BassDoubling {
    Octave,
    Tenth,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassHit {
    pub step: usize,
    pub degree: BassDegree,
    pub duration: usize,
    pub velocity: u8,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub doubling: Vec<BassDoubling>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BassReference {
    #[default]
    ChordRoot,
    Tonic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassMotifNote {
    pub step: usize,
    pub duration: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semitones: Option<i8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degree: Option<i8>,
    #[serde(default)]
    pub octave: i8,
    pub velocity: u8,
    #[serde(default)]
    pub tie: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub doubling: Vec<BassDoubling>,
}

const fn default_one_bar() -> usize {
    1
}

const fn default_motif_weight() -> u32 {
    1
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassMotif {
    #[serde(default = "default_one_bar")]
    pub bars: usize,
    #[serde(default = "default_motif_weight")]
    pub weight: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    #[serde(default)]
    pub reference: BassReference,
    pub notes: Vec<BassMotifNote>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassVoice {
    pub low: u8,
    pub high: u8,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hits: Vec<BassHit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub motifs: Vec<BassMotif>,
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
    #[serde(default)]
    pub stroke: Stroke,
    #[serde(default = "unit_multiplier")]
    pub probability: f32,
    /// Liaison : pas de nouvelle attaque si les hauteurs sonnent encore à
    /// l'arrivée (nappe). Sans coïncidence exacte, attaque normale.
    #[serde(default)]
    pub tie: bool,
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
    #[serde(default = "triad_strings")]
    pub strings: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strum: Option<Strum>,
    #[serde(default)]
    pub voicing: ChordVoicing,
    #[serde(default = "fixed_low")]
    pub low: u8,
    #[serde(default = "fixed_high")]
    pub high: u8,
}

fn triad_strings() -> u8 {
    3
}

fn fixed_low() -> u8 {
    52
}
fn fixed_high() -> u8 {
    64
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChordVoicing {
    #[default]
    CloseTriad,
    FixedTriad,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stroke {
    #[default]
    Block,
    Down,
    Up,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Strum {
    pub down_gap_ms: f32,
    pub up_gap_ms: f32,
    pub up_strings: u8,
    pub gap_jitter: f32,
    pub anchor: f32,
    pub tempo_exponent: f32,
    pub ref_bpm: f32,
    pub max_spread_ratio: f32,
    pub velocity_slope: u8,
    pub velocity_jitter: u8,
}

impl ChordVoice {
    pub(crate) fn pattern(&self, offset: f32) -> &ChordPattern {
        let index = if offset == 0.0 {
            self.default_pattern
        } else {
            (1.0 + offset).round().clamp(0.0, (self.patterns.len() - 1) as f32) as usize
        };
        &self.patterns[index]
    }
    pub(crate) fn needs_seed(&self) -> bool {
        self.patterns.iter().flat_map(|p| &p.hits).any(|h| {
            (h.probability > 0.0 && h.probability < 1.0)
                || (h.probability > 0.0
                    && h.stroke != Stroke::Block
                    && self
                        .strum
                        .as_ref()
                        .is_some_and(|s| s.gap_jitter > 0.0 || s.velocity_jitter > 0))
        })
    }
}

impl Strum {
    fn validate(&self, strings: u8) -> Result<(), ProfileError> {
        let field = "accompaniment.chords.strum";
        for (name, value, low, high) in [
            ("down_gap_ms", self.down_gap_ms, 0.0, 40.0),
            ("up_gap_ms", self.up_gap_ms, 0.0, 40.0),
            ("gap_jitter", self.gap_jitter, 0.0, 0.8),
            ("anchor", self.anchor, 0.0, 1.0),
            ("tempo_exponent", self.tempo_exponent, 0.0, 1.0),
            ("ref_bpm", self.ref_bpm, 1.0, 1000.0),
            ("max_spread_ratio", self.max_spread_ratio, 0.0, 0.4),
        ] {
            require(
                value.is_finite() && (low..=high).contains(&value),
                format!("{field}.{name}"),
                &format!("valeur finie dans {low}..{high} requise"),
            )?;
        }
        require(
            (2..=strings).contains(&self.up_strings),
            format!("{field}.up_strings"),
            "2..strings requis",
        )?;
        require(self.velocity_slope <= 20, format!("{field}.velocity_slope"), "0..20 requis")?;
        require(self.velocity_jitter <= 20, format!("{field}.velocity_jitter"), "0..20 requis")
    }
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

fn validate_doubling(
    doubling: &[BassDoubling],
    jazz: bool,
    low: u8,
    high: u8,
    max_offset: i16,
    field: &str,
) -> Result<(), ProfileError> {
    require(
        !jazz || doubling.is_empty(),
        field,
        "les doublures modales demandent l'harmonie trad_functional",
    )?;
    require(
        doubling.len() <= 2 && doubling.iter().enumerate().all(|(i, d)| !doubling[..i].contains(d)),
        field,
        "doublures uniques octave/tenth requises",
    )?;
    let extra = if doubling.contains(&BassDoubling::Tenth) {
        16i16
    } else if doubling.is_empty() {
        0
    } else {
        12
    };
    require(
        (0..12).all(|pc| {
            let base = i16::from(super::accomp::place_in_range(pc, low, high));
            base + max_offset + extra <= 127
        }),
        field,
        "registre trop aigu pour les doublures MIDI demandées",
    )
}

impl TradHarmonyParams {
    pub(crate) fn validate_accompaniment(
        &self,
        steps: usize,
        jazz: bool,
        parts: &[crate::style_profile::ProfilePart],
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
        for (i, hit) in b.hits.iter().enumerate() {
            let field = format!("accompaniment.bass.hits[{i}].doubling");
            validate_doubling(&hit.doubling, jazz, b.low, b.high, 0, &field)?;
        }
        if !b.motifs.is_empty() {
            require(
                b.hits.is_empty(),
                "accompaniment.bass.hits",
                "hits et motifs sont deux sources exclusives de la même voix",
            )?;
            require(
                !jazz,
                "accompaniment.bass.motifs",
                "les motifs de basse demandent l'harmonie trad_functional",
            )?;
            require(
                b.motifs.len() <= 256,
                "accompaniment.bass.motifs",
                "1..256 motifs de basse requis",
            )?;
            let mut total_weight = 0u32;
            for (i, motif) in b.motifs.iter().enumerate() {
                let mf = format!("accompaniment.bass.motifs[{i}]");
                require(
                    (1..=4).contains(&motif.bars),
                    format!("{mf}.bars"),
                    "1..4 mesures par motif requises",
                )?;
                total_weight = total_weight.checked_add(motif.weight).ok_or_else(|| {
                    ProfileError::new(format!("{mf}.weight"), "somme des poids hors u32")
                })?;
                if let Some(part_name) = &motif.part {
                    require(
                        parts.iter().any(|p| &p.name == part_name),
                        format!("{mf}.part"),
                        "partie inconnue dans form.parts",
                    )?;
                }
                let span = motif.bars * steps;
                let nf = format!("{mf}.notes");
                require(!motif.notes.is_empty(), &nf, "motif non vide requis")?;
                let mut previous = None;
                let mut previous_end = 0;
                for (j, note) in motif.notes.iter().enumerate() {
                    let f = format!("{nf}[{j}]");
                    require(
                        note.step < span && previous.is_none_or(|p| note.step > p),
                        format!("{f}.step"),
                        "pas croissants uniques dans le motif requis",
                    )?;
                    require(
                        note.step >= previous_end,
                        format!("{f}.step"),
                        "les notes principales du motif ne doivent pas se chevaucher",
                    )?;
                    require(
                        note.duration > 0
                            && note.step.checked_add(note.duration).is_some_and(|end| end <= span),
                        format!("{f}.duration"),
                        "durée positive dans le motif requise",
                    )?;
                    require(
                        (1..=127).contains(&note.velocity),
                        format!("{f}.velocity"),
                        "vélocité dans 1..127 requise",
                    )?;
                    require(
                        note.semitones.is_some() ^ note.degree.is_some(),
                        &f,
                        "exactement un champ parmi semitones ou degree requis",
                    )?;
                    if let Some(semitones) = note.semitones {
                        require(
                            (-24..=24).contains(&semitones),
                            format!("{f}.semitones"),
                            "intervalle dans -24..24 requis",
                        )?;
                    }
                    if let Some(degree) = note.degree {
                        require(
                            degree != 0 && (-14..=14).contains(&degree),
                            format!("{f}.degree"),
                            "degré non nul dans -14..14 requis",
                        )?;
                    }
                    require(
                        (-2..=2).contains(&note.octave),
                        format!("{f}.octave"),
                        "octave relative dans -2..2 requise",
                    )?;
                    // Un pas modal couvre 1..3 demi-tons (seconde augmentée comprise).
                    // Ces bornes couvrent aussi les références chromatiques hors du mode.
                    let (min_offset, max_offset) = if let Some(semitones) = note.semitones {
                        (i16::from(semitones), i16::from(semitones))
                    } else {
                        let degree = i16::from(note.degree.unwrap_or(1));
                        if degree > 0 {
                            (degree - 1, 3 * (degree - 1))
                        } else {
                            (3 * degree, degree)
                        }
                    };
                    let octave = 12 * i16::from(note.octave);
                    let (min_offset, max_offset) = (min_offset + octave, max_offset + octave);
                    require(
                        (0..12).all(|pc| {
                            let base = i16::from(super::accomp::place_in_range(pc, b.low, b.high));
                            base + min_offset >= 0 && base + max_offset <= 127
                        }),
                        &f,
                        "hauteur résultante hors du domaine MIDI 0..127",
                    )?;
                    validate_doubling(
                        &note.doubling,
                        jazz,
                        b.low,
                        b.high,
                        max_offset,
                        &format!("{f}.doubling"),
                    )?;
                    previous = Some(note.step);
                    previous_end = note.step + note.duration;
                }
            }
            require(
                total_weight > 0,
                "accompaniment.bass.motifs",
                "somme des poids positive requise",
            )?;
            for part in parts {
                require(
                    b.motifs.iter().any(|m| {
                        m.weight > 0
                            && (m.part.as_deref() == Some(part.name.as_str()) || m.part.is_none())
                    }),
                    "accompaniment.bass.motifs",
                    format!("aucun motif de poids positif pour la partie {}", part.name).as_str(),
                )?;
            }
        }
        let c = &self.chords;
        require(
            !jazz
                || (c.strum.is_none()
                    && c.strings == 3
                    && c.voicing == ChordVoicing::CloseTriad
                    && c.patterns
                        .iter()
                        .flat_map(|p| &p.hits)
                        .all(|h| h.probability == 1.0 && h.stroke == Stroke::Block)),
            "accompaniment.chords",
            "les gestes de cordes demandent l'harmonie trad_functional",
        )?;
        require(c.center <= 127, "accompaniment.chords.center", "registre MIDI requis")?;
        require(c.max_drift <= 127, "accompaniment.chords.max_drift", "registre MIDI requis")?;
        require(
            (3..=6).contains(&c.strings),
            "accompaniment.chords.strings",
            "3..6 cordes requises",
        )?;
        require(
            c.low <= c.high && c.high <= 127,
            "accompaniment.chords.low",
            "fenêtre MIDI ordonnée requise",
        )?;
        require(
            c.voicing != ChordVoicing::FixedTriad || c.high - c.low >= 11,
            "accompaniment.chords.high",
            "fixed_triad demande douze hauteurs consécutives",
        )?;
        if let Some(strum) = &c.strum {
            strum.validate(c.strings)?;
        }
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
            for (j, hit) in pattern.hits.iter().enumerate() {
                require(
                    hit.probability.is_finite() && (0.0..=1.0).contains(&hit.probability),
                    format!("{field}[{j}].probability"),
                    "probabilité finie dans 0..1 requise",
                )?;
            }
        }
        Ok(())
    }
}

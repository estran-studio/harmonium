//! Arc sur plusieurs chorus : l'intensité, la densité, le feel de basse, le comping, la
//! batterie, les breaks et la retombée finale varient d'un chorus à l'autre, en données
//! (section `[arc]` du profil, optionnelle).
//!
//! Un **chorus** est une passe complète de la forme (`TunePosition::tour`) ; il se compte dans
//! le set, sur plusieurs airs si le set en compte plusieurs (`TunePosition::chorus`). Le nombre
//! de chorus est connu quand le set a une longueur (`TunePosition::choruses`), sinon l'arc
//! n'a pas de fin : aucune retombée, et les étapes au-delà de la dernière se tiennent ou se
//! reprennent selon `repeat_from`.
//!
//! Algorithme générique : aucune valeur musicale ni nom de style ici. L'état d'une mesure ne
//! dépend que de sa position et de la graine de l'arc (flux ChaCha8 dérivé par chorus, entiers
//! `u64`), jamais d'une mesure précédente : le seek rejoue la même musique.

use std::collections::HashSet;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::{accomp_profile::require, arrangement::TunePosition};
use crate::{
    algo::RngCore,
    ornaments::mix,
    style_profile::{ProfileError, StyleProfile},
    timeline::{Measure, TrackId},
};

/// Nombre maximal d'étapes de `[[arc.stages]]`.
pub const MAX_STAGES: usize = 16;
/// Nombre maximal de variantes par étape (et de variantes de retombée).
pub const MAX_VARIANTS: usize = 16;
/// Nombre maximal de breaks par variante.
pub const MAX_BREAKS: usize = 8;
/// Nombre maximal de chorus d'une étape.
pub const MAX_STAGE_CHORUSES: u32 = 1024;
/// Nombre maximal de tags pondérés par variante et par banque.
pub const MAX_TAGS: usize = 8;

const VARIANT_STREAM: u64 = 0x4152_4356;
const FALL_STREAM: u64 = 0x4152_4346;
const BREAK_STREAM: u64 = 0x4152_4342;
const PERCUSSION_STREAM: u64 = 0x4152_4350;

/// Manière de jouer la basse sous une marche (`accompaniment.bass.walking`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BassFeel {
    /// Une attaque par pulsation.
    Walking,
    /// Une attaque toutes les deux pulsations (la première est l'arrivée, la seconde
    /// l'approche), plus une arrivée à chaque changement d'accord.
    TwoFeel,
}

impl BassFeel {
    /// Pulsations entre deux attaques de la ligne.
    #[must_use]
    pub const fn attack_pulses(self) -> usize {
        match self {
            Self::Walking => 1,
            Self::TwoFeel => 2,
        }
    }
}

/// Poids d'un tag : un motif de comping ou un pattern de batterie qui le porte concourt avec
/// son poids multiplié par celui du tag ; les autres ne concourent pas.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagWeight {
    pub tag: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
}

const fn default_weight() -> u32 {
    1
}

/// Poids effectif d'un candidat de tag `tag` sous `tags` : 0 s'il ne porte pas un tag nommé.
pub(crate) fn tag_weight(tags: &[TagWeight], tag: Option<&str>) -> u64 {
    tag.map_or(0, |tag| tags.iter().filter(|t| t.tag == tag).map(|t| u64::from(t.weight)).sum())
}

/// Poids d'un candidat sous les tags de l'arc : son poids seul si `tags` est vide, sinon son
/// poids fois celui de son tag (0 s'il n'en porte pas un nommé).
pub(crate) fn tagged_weight(weight: u32, tag: Option<&str>, tags: &[TagWeight]) -> u64 {
    if tags.is_empty() {
        u64::from(weight)
    } else {
        u64::from(weight).saturating_mul(tag_weight(tags, tag))
    }
}

/// Silence ou stop-time sur une fenêtre de mesures du chorus.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArcBreak {
    /// Première mesure de la fenêtre, 1-based dans le chorus ; négatif : compté depuis la fin
    /// (`-2` : l'avant-dernière mesure). Jamais 0.
    pub start: i32,
    /// Nombre de mesures de la fenêtre.
    #[serde(default = "default_break_bars")]
    pub bars: u32,
    /// Probabilité que le chorus porte ce break (tirage par chorus).
    #[serde(default = "default_chance")]
    pub chance: f32,
    /// Pistes touchées ; vide : toutes.
    #[serde(default)]
    pub tracks: Vec<TrackId>,
    /// Pas où les pistes touchées gardent leurs attaques ; vide : silence complet (break),
    /// `[0]` : stop-time sur le premier temps.
    #[serde(default)]
    pub keep_steps: Vec<usize>,
}

const fn default_break_bars() -> u32 {
    1
}

const fn default_chance() -> f32 {
    1.0
}

impl ArcBreak {
    /// Fenêtre `[début, fin)` en mesures 0-based dans un chorus de `bars` mesures.
    fn window(&self, bars: usize) -> (usize, usize) {
        let first = if self.start > 0 {
            self.start.unsigned_abs() as usize - 1
        } else {
            bars.saturating_sub(self.start.unsigned_abs() as usize)
        };
        (first, (first + self.bars as usize).min(bars))
    }
}

/// Réglages d'un chorus ; chaque champ omis laisse le profil tel quel.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArcVariant {
    /// Poids du tirage parmi les variantes de l'étape.
    #[serde(default = "default_weight")]
    pub weight: u32,
    /// Multiplicateur de l'intensité lue par les courbes de `dynamics` (vélocités, motif de
    /// comping par intensité, densité de batterie) ; le tempo ne change pas.
    #[serde(default = "default_multiplier")]
    pub intensity: f32,
    /// Multiplicateur de la densité rythmique lue par `density_percussion`, `density_chords` et
    /// la densité des notes de passage de la basse.
    #[serde(default = "default_multiplier")]
    pub density: f32,
    /// Feel de la basse ; exige `accompaniment.bass.walking`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bass_feel: Option<BassFeel>,
    /// Tags des motifs (`accompaniment.chords.comping[].tag`) et des phrases de comping
    /// (`accompaniment.chords.phrases[].tag`) autorisés, pondérés.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comping: Vec<TagWeight>,
    /// Tags des patterns (`percussion.patterns[].tag`), des phrases (`percussion.phrases[].tag`)
    /// et des relances (`percussion.fills[].tag`) de batterie autorisés, pondérés ; les mesures
    /// de relance de `cadence_patterns` ne changent pas.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub percussion: Vec<TagWeight>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breaks: Vec<ArcBreak>,
}

const fn default_multiplier() -> f32 {
    1.0
}

/// Étape de l'arc : `choruses` chorus consécutifs, chacun tirant sa variante.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArcStage {
    #[serde(default = "default_stage_choruses")]
    pub choruses: u32,
    pub variants: Vec<ArcVariant>,
}

const fn default_stage_choruses() -> u32 {
    1
}

/// Section `[arc]` du profil.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChorusArc {
    /// Étapes dans l'ordre des chorus. Passé la dernière : la dernière étape se tient, ou, avec
    /// `repeat_from`, les étapes depuis celle-là se répètent en boucle.
    pub stages: Vec<ArcStage>,
    /// Indice 0-based de l'étape où reprend l'arc après la dernière.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_from: Option<usize>,
    /// Retombée : variantes du dernier chorus quand le nombre de chorus est connu (au moins
    /// deux) ; elles remplacent celles de l'étape. Sans fin connue, jamais jouées.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fall: Vec<ArcVariant>,
}

/// État de l'arc à une mesure : variante du chorus et break éventuel.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ArcBar<'a> {
    variant: &'a ArcVariant,
    seed: u64,
    chorus: usize,
    brk: Option<&'a ArcBreak>,
}

impl<'a> ArcBar<'a> {
    /// Intensité de la mesure : celle du jeu fois le multiplicateur, bornée à 0..1.
    pub(crate) fn intensity(&self, base: f32) -> f32 {
        (base * self.variant.intensity).clamp(0.0, 1.0)
    }

    /// Densité rythmique de la mesure, bornée à 0..1.
    pub(crate) fn density(&self, base: f32) -> f32 {
        (base * self.variant.density).clamp(0.0, 1.0)
    }

    /// Pulsations entre deux attaques de la basse (1 : une marche par pulsation).
    pub(crate) fn attack_pulses(&self) -> usize {
        self.variant.bass_feel.map_or(1, BassFeel::attack_pulses)
    }

    pub(crate) fn comping_tags(&self) -> &'a [TagWeight] {
        &self.variant.comping
    }

    pub(crate) fn percussion_tags(&self) -> &'a [TagWeight] {
        &self.variant.percussion
    }

    /// Graine du tirage de batterie de la partie `part` à la densité `density` : constante dans
    /// le chorus, comme le choix de pattern par partie d'un air.
    pub(crate) fn percussion_seed(&self, part: usize, density: u8) -> u64 {
        mix(
            mix(mix(self.seed, PERCUSSION_STREAM), self.chorus as u64),
            (part as u64) << 8 | u64::from(density),
        )
    }

    /// Retire les attaques que le break de la mesure fait taire.
    pub(crate) fn apply_break(&self, measure: &mut Measure) {
        let Some(brk) = self.brk else { return };
        for (track, notes) in &mut measure.tracks {
            if brk.tracks.is_empty() || brk.tracks.contains(track) {
                notes.retain(|note| brk.keep_steps.contains(&note.start_step));
            }
        }
    }
}

/// Tirage pondéré d'une variante avec le flux `seed` ; `variants` est non vide (validé).
fn draw(variants: &[ArcVariant], seed: u64) -> &ArcVariant {
    let total: u64 = variants.iter().map(|v| u64::from(v.weight)).sum();
    let mut pick = ChaCha8Rng::seed_from_u64(seed).next_range_u64(0..total.max(1));
    for variant in variants {
        if pick < u64::from(variant.weight) {
            return variant;
        }
        pick -= u64::from(variant.weight);
    }
    &variants[variants.len() - 1]
}

impl ChorusArc {
    /// Indice de l'étape du chorus `chorus` (1-based).
    fn stage_of(&self, chorus: usize) -> usize {
        let mut remaining = chorus - 1;
        for (index, stage) in self.stages.iter().enumerate() {
            if remaining < stage.choruses as usize {
                return index;
            }
            remaining -= stage.choruses as usize;
        }
        let Some(from) = self.repeat_from else { return self.stages.len() - 1 };
        let cycle: usize = self.stages[from..].iter().map(|s| s.choruses as usize).sum();
        remaining %= cycle;
        for (index, stage) in self.stages.iter().enumerate().skip(from) {
            if remaining < stage.choruses as usize {
                return index;
            }
            remaining -= stage.choruses as usize;
        }
        self.stages.len() - 1
    }

    /// Variante du chorus `chorus` (1-based) sur un set de `choruses` chorus, si connu.
    fn variant(&self, seed: u64, chorus: usize, choruses: Option<usize>) -> &ArcVariant {
        if !self.fall.is_empty() && choruses.is_some_and(|last| last >= 2 && chorus == last) {
            return draw(&self.fall, mix(mix(seed, FALL_STREAM), chorus as u64));
        }
        let stage = &self.stages[self.stage_of(chorus)];
        draw(&stage.variants, mix(mix(seed, VARIANT_STREAM), chorus as u64))
    }

    /// État de l'arc à la mesure de `position` ; `None` pendant une intro. `bars` : mesures
    /// d'un chorus (`form.bars_per_tour()`), `seed` : graine de l'arc du set.
    pub(crate) fn at(&self, seed: u64, position: &TunePosition, bars: usize) -> Option<ArcBar<'_>> {
        if position.is_intro() {
            return None;
        }
        let chorus = position.chorus();
        let variant = self.variant(seed, chorus, position.choruses());
        let bar = position.bar_in_tour();
        let brk = variant.breaks.iter().enumerate().find_map(|(index, brk)| {
            let (first, end) = brk.window(bars);
            if !(first..end).contains(&bar) {
                return None;
            }
            let mut rng = ChaCha8Rng::seed_from_u64(mix(
                mix(mix(seed, BREAK_STREAM), chorus as u64),
                index as u64,
            ));
            let draw = rng.next_range_u64(0..1 << 32) as f64 / (1u64 << 32) as f64;
            (draw < f64::from(brk.chance)).then_some(brk)
        });
        Some(ArcBar { variant, seed, chorus, brk })
    }

    /// Valide la section contre le profil (`field` : chemin de la section, `arc`).
    pub(crate) fn validate(&self, profile: &StyleProfile) -> Result<(), ProfileError> {
        let steps = profile.meter.steps_per_bar;
        let bars = profile.form.bars_per_tour();
        require(
            (1..=MAX_STAGES).contains(&self.stages.len()),
            "arc.stages",
            "1..16 étapes requises",
        )?;
        if let Some(from) = self.repeat_from {
            require(from < self.stages.len(), "arc.repeat_from", "étape inconnue dans arc.stages")?;
        }
        for (s, stage) in self.stages.iter().enumerate() {
            let field = format!("arc.stages[{s}]");
            require(
                (1..=MAX_STAGE_CHORUSES).contains(&stage.choruses),
                format!("{field}.choruses"),
                "1..1024 chorus requis",
            )?;
            self.validate_variants(
                profile,
                &stage.variants,
                &format!("{field}.variants"),
                steps,
                bars,
            )?;
        }
        if !self.fall.is_empty() {
            self.validate_variants(profile, &self.fall, "arc.fall", steps, bars)?;
        }
        Ok(())
    }

    fn validate_variants(
        &self,
        profile: &StyleProfile,
        variants: &[ArcVariant],
        field: &str,
        steps: usize,
        bars: usize,
    ) -> Result<(), ProfileError> {
        require((1..=MAX_VARIANTS).contains(&variants.len()), field, "1..16 variantes requises")?;
        let comping_tags: HashSet<&str> = profile
            .accompaniment
            .iter()
            .flat_map(|a| {
                a.chords
                    .comping
                    .iter()
                    .filter_map(|motif| motif.tag.as_deref())
                    .chain(a.chords.phrases.iter().filter_map(|phrase| phrase.tag.as_deref()))
            })
            .collect();
        let percussion_tags: HashSet<&str> = profile
            .percussion
            .patterns
            .iter()
            .filter_map(|p| p.tag.as_deref())
            .chain(profile.percussion.phrases.iter().filter_map(|p| p.tag.as_deref()))
            .chain(profile.percussion.fills.iter().filter_map(|f| f.tag.as_deref()))
            .collect();
        let mut total = 0u64;
        for (v, variant) in variants.iter().enumerate() {
            let field = format!("{field}[{v}]");
            require(variant.weight > 0, format!("{field}.weight"), "poids positif requis")?;
            total += u64::from(variant.weight);
            for (name, value) in [("intensity", variant.intensity), ("density", variant.density)] {
                require(
                    value.is_finite() && (0.0..=2.0).contains(&value),
                    format!("{field}.{name}"),
                    "multiplicateur fini dans 0..2 requis",
                )?;
            }
            require(
                variant.bass_feel.is_none()
                    || profile.accompaniment.as_ref().is_some_and(|a| a.bass.walking.is_some()),
                format!("{field}.bass_feel"),
                "le feel de basse demande accompaniment.bass.walking",
            )?;
            for (name, tags, known, bank) in [
                (
                    "comping",
                    &variant.comping,
                    &comping_tags,
                    "accompaniment.chords (comping, phrases)",
                ),
                (
                    "percussion",
                    &variant.percussion,
                    &percussion_tags,
                    "percussion (patterns, phrases, fills)",
                ),
            ] {
                require(tags.len() <= MAX_TAGS, format!("{field}.{name}"), "au plus 8 tags")?;
                let mut seen = HashSet::new();
                for (t, tag) in tags.iter().enumerate() {
                    let field = format!("{field}.{name}[{t}]");
                    require(tag.weight > 0, format!("{field}.weight"), "poids positif requis")?;
                    require(
                        seen.insert(tag.tag.as_str()),
                        format!("{field}.tag"),
                        "tag unique requis",
                    )?;
                    require(
                        known.contains(tag.tag.as_str()),
                        format!("{field}.tag"),
                        &format!("aucun élément de {bank} ne porte ce tag"),
                    )?;
                }
            }
            require(
                variant.breaks.len() <= MAX_BREAKS,
                format!("{field}.breaks"),
                "au plus 8 breaks",
            )?;
            for (b, brk) in variant.breaks.iter().enumerate() {
                let field = format!("{field}.breaks[{b}]");
                let start = brk.start.unsigned_abs() as usize;
                require(
                    brk.start != 0 && start <= bars,
                    format!("{field}.start"),
                    "mesure du chorus requise (1-based, négatif depuis la fin, jamais 0)",
                )?;
                require(
                    brk.bars >= 1
                        && if brk.start > 0 {
                            start - 1 + brk.bars as usize <= bars
                        } else {
                            brk.bars as usize <= start
                        },
                    format!("{field}.bars"),
                    "fenêtre d'au moins une mesure, dans le chorus",
                )?;
                require(
                    brk.chance.is_finite() && (0.0..=1.0).contains(&brk.chance),
                    format!("{field}.chance"),
                    "probabilité finie dans 0..1 requise",
                )?;
                let mut tracks = HashSet::new();
                require(
                    brk.tracks.iter().all(|t| tracks.insert(*t)),
                    format!("{field}.tracks"),
                    "piste unique requise (doublon)",
                )?;
                let mut kept = HashSet::new();
                require(
                    brk.keep_steps.iter().all(|&step| step < steps && kept.insert(step)),
                    format!("{field}.keep_steps"),
                    "pas uniques dans la mesure requis",
                )?;
            }
        }
        require(
            total <= u64::from(u32::MAX),
            field,
            "somme des poids des variantes trop grande (u32)",
        )
    }
}

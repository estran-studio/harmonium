//! Ornements idiomatiques : un seul chemin du taux du profil au flux de notes.
//!
//! La section `[ornaments]` d'un profil décrit, par type, un taux pour 100
//! notes (global et par mesure de la partie), une durée porteuse minimale, des
//! contextes permis et un lien à l'arousal. La réalisation (`realize`) ne
//! connaît aucun nom de style : tout vient des données.
mod realize;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    style_profile::ProfileError,
    theory::PitchClass,
    trad::{Mode, dynamics::Curve},
};

/// Mélange de 64 bits (SplitMix64) pour dériver des graines sans état.
pub(crate) fn mix(a: u64, b: u64) -> u64 {
    let mut z =
        a.wrapping_add(0x9E37_79B9_7F4A_7C15).wrapping_add(b.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Graine des ornements d'un air, fonction de son matériel écrit (hauteur, pas, durée) :
/// un air ne dépend que de ses notes, jamais d'un tirage ni du reste de la session.
pub fn tune_seed(tune_index: usize, notes: impl IntoIterator<Item = (u8, usize, usize)>) -> u64 {
    let seed =
        notes.into_iter().fold(mix(0x4F52_4E41, tune_index as u64), |h, (pitch, start, len)| {
            mix(h, (u64::from(pitch) << 40) | ((start as u64) << 20) | len as u64)
        });
    seed.max(1)
}

/// Types d'ornements ; l'ordre de déclaration est l'ordre de tirage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrnamentKind {
    /// Note de passage de densité émotionnelle : scinde une tenue, sans tirage.
    Passing,
    GraceSingle,
    GraceMultiple,
    Roll,
    Turn,
    TripletRepeated,
    DoubleStop,
    Chord,
    Trill,
}
impl OrnamentKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Passing => "passing",
            Self::GraceSingle => "grace_single",
            Self::GraceMultiple => "grace_multiple",
            Self::Roll => "roll",
            Self::Turn => "turn",
            Self::TripletRepeated => "triplet_repeated",
            Self::DoubleStop => "double_stop",
            Self::Chord => "chord",
            Self::Trill => "trill",
        }
    }
    const fn is_grace(self) -> bool {
        matches!(self, Self::GraceSingle | Self::GraceMultiple)
    }
    const fn is_stop(self) -> bool {
        matches!(self, Self::DoubleStop | Self::Chord)
    }
}

/// Intervalle avec la note précédente de la mélodie dans la mesure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntervalClass {
    Repeated,
    /// Deux demi-tons au plus.
    Step,
    Leap,
}
/// Temps fort : début de pulse du profil ; temps faible : tout autre pas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeatClass {
    Strong,
    Weak,
}
/// Dernière mesure de la phrase (`parts.phrase_ends`, sinon rôles cadentiels).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhraseEnd {
    #[default]
    Any,
    Never,
    Only,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OrnamentRule {
    pub enabled: bool,
    /// Plancher écrit du corpus, pour 100 notes de la mélodie, avant `rate_scale`.
    pub written_per_100_notes: f32,
    /// Taux pour 100 notes par mesure de la partie (1 = première) ; au-delà ou
    /// vide : `written_per_100_notes`.
    pub per_100_notes_by_bar: Vec<f32>,
    /// Durée porteuse minimale, en croches ; durée totale remplacée pour les figures.
    pub min_carrier_eighths: f32,
    /// Multiplicateurs du taux à arousal 0 et 1 (interpolation linéaire).
    pub arousal_low_multiplier: f32,
    pub arousal_high_multiplier: f32,
    /// Intervalles permis avec la note précédente ; vide = tous. La première note de la
    /// mesure n'a pas de précédente connue : elle est exclue dès que la liste n'est pas vide.
    pub intervals: Vec<IntervalClass>,
    /// Temps permis ; vide = tous.
    pub beats: Vec<BeatClass>,
    pub phrase_end: PhraseEnd,
    /// Gain de vélocité des notes ajoutées (appogiatures, voisines, seconde corde).
    pub velocity_scale: f32,
    /// Appogiatures : durée totale avant la porteuse, en fraction de pas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<f32>,
    /// Appogiatures multiples : nombre de notes (2 par défaut).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u8>,
    /// Cordes à vide (hauteurs MIDI croissantes) de la double corde et de l'accord.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub strings: Vec<u8>,
    /// Intervalle (demi-tons) permis entre la porteuse et chaque note ajoutée.
    pub min_interval: u8,
    pub max_interval: u8,
    /// Note de passage : part de la mesure (0..1) couverte selon la densité.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density_curve: Option<Curve>,
}
impl Default for OrnamentRule {
    fn default() -> Self {
        Self {
            enabled: true,
            written_per_100_notes: 0.0,
            per_100_notes_by_bar: Vec::new(),
            min_carrier_eighths: 0.0,
            arousal_low_multiplier: 1.0,
            arousal_high_multiplier: 1.0,
            intervals: Vec::new(),
            beats: Vec::new(),
            phrase_end: PhraseEnd::Any,
            velocity_scale: 0.8,
            span: None,
            count: None,
            strings: Vec::new(),
            min_interval: 3,
            max_interval: 12,
            density_curve: None,
        }
    }
}
impl OrnamentRule {
    fn rate_at(&self, bar_in_part: usize) -> f32 {
        bar_in_part
            .checked_sub(1)
            .and_then(|i| self.per_100_notes_by_bar.get(i))
            .copied()
            .unwrap_or(self.written_per_100_notes)
    }
    fn arousal_multiplier(&self, arousal: f32) -> f32 {
        let a = if arousal.is_finite() { arousal.clamp(0.0, 1.0) } else { 0.5 };
        (self.arousal_high_multiplier - self.arousal_low_multiplier)
            .mul_add(a, self.arousal_low_multiplier)
    }
    fn max_rate(&self) -> f32 {
        self.per_100_notes_by_bar.iter().copied().fold(self.written_per_100_notes, f32::max)
    }
}

/// Section `[ornaments]` d'un profil.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Ornaments {
    /// Facteur global appliqué à tous les taux écrits (les ABC sous-notent).
    pub rate_scale: f32,
    /// Probabilité qu'une mesure rejouée (passe ou tour 2+) tire ses ornements
    /// indépendamment de sa source ; 0 : une copie porte les mêmes ornements.
    pub repeat_variation: f32,
    pub types: BTreeMap<OrnamentKind, OrnamentRule>,
}
impl Default for Ornaments {
    fn default() -> Self {
        Self { rate_scale: 1.0, repeat_variation: 0.0, types: BTreeMap::new() }
    }
}

/// Contexte d'une mesure de mélodie, préparé par le générateur.
#[derive(Clone, Debug)]
pub struct BarContext {
    pub key: PitchClass,
    pub mode: Mode,
    pub tessitura: (u8, u8),
    /// Mesure de la partie, 1-based.
    pub bar_in_part: usize,
    pub steps_per_pulse: usize,
    pub phrase_end_bar: bool,
    /// Mesure protégée par le profil (cadence) : aucun ornement.
    pub protected: bool,
    /// Passe ou tour 2+.
    pub repeat: bool,
    pub arousal: f32,
    pub density: f32,
    pub bar_index: usize,
    /// Graine de l'air (0 si aucun type ne tire).
    pub seed: u64,
    /// Porteuses dont les agréments tirent dans un domaine distinct de l'unisson.
    pub independent_steps: Vec<usize>,
}

fn bounded(value: f32, lo: f32, hi: f32, field: &str) -> Result<(), ProfileError> {
    if value.is_finite() && (lo..=hi).contains(&value) {
        Ok(())
    } else {
        Err(ProfileError::new(field, format!("valeur finie attendue dans {lo}..{hi}")))
    }
}

impl Ornaments {
    fn active(&self) -> impl Iterator<Item = (OrnamentKind, &OrnamentRule)> {
        self.types.iter().filter(|(_, rule)| rule.enabled).map(|(&kind, rule)| (kind, rule))
    }

    /// Aucun type n'agit : le flux est celui d'une partition sans ornement.
    pub fn is_inert(&self) -> bool {
        !self.active().any(|(kind, rule)| {
            if kind == OrnamentKind::Passing {
                rule.density_curve.as_ref().is_some_and(|c| c.0.iter().any(|p| p[1] > 0.0))
                    && rule.arousal_low_multiplier.max(rule.arousal_high_multiplier) > 0.0
            } else {
                self.draws(rule)
            }
        })
    }

    /// Au moins un type tire au hasard : une graine est tirée une fois par air.
    pub fn needs_seed(&self) -> bool {
        self.active().any(|(kind, rule)| kind != OrnamentKind::Passing && self.draws(rule))
    }

    fn draws(&self, rule: &OrnamentRule) -> bool {
        self.rate_scale > 0.0
            && rule.max_rate() > 0.0
            && rule.arousal_low_multiplier.max(rule.arousal_high_multiplier) > 0.0
    }

    pub fn validate(&self) -> Result<(), ProfileError> {
        bounded(self.rate_scale, 0.0, 10.0, "ornaments.rate_scale")?;
        bounded(self.repeat_variation, 0.0, 1.0, "ornaments.repeat_variation")?;
        for (&kind, rule) in &self.types {
            let name = kind.name();
            let f = |field: &str| format!("ornaments.types.{name}.{field}");
            bounded(rule.written_per_100_notes, 0.0, 100.0, &f("written_per_100_notes"))?;
            if rule.per_100_notes_by_bar.len() > 64 {
                return Err(ProfileError::new(
                    f("per_100_notes_by_bar"),
                    "au plus une valeur par mesure de la partie (64)",
                ));
            }
            for (i, rate) in rule.per_100_notes_by_bar.iter().enumerate() {
                bounded(*rate, 0.0, 100.0, &f(&format!("per_100_notes_by_bar[{i}]")))?;
            }
            bounded(rule.min_carrier_eighths, 0.0, 64.0, &f("min_carrier_eighths"))?;
            bounded(rule.arousal_low_multiplier, 0.0, 4.0, &f("arousal_low_multiplier"))?;
            bounded(rule.arousal_high_multiplier, 0.0, 4.0, &f("arousal_high_multiplier"))?;
            bounded(rule.velocity_scale, 0.1, 1.0, &f("velocity_scale"))?;
            let reject = |field: &str| {
                Err(ProfileError::new(f(field), format!("champ sans objet pour le type {name}")))
            };
            match rule.span {
                Some(span) if kind.is_grace() => {
                    bounded(span, 0.05, 0.49, &f("span"))?;
                }
                Some(_) => return reject("span"),
                None => {}
            }
            match rule.count {
                Some(count) if kind == OrnamentKind::GraceMultiple => {
                    if !(2..=4).contains(&count) {
                        return Err(ProfileError::new(f("count"), "attendu 2..4 notes"));
                    }
                }
                Some(_) => return reject("count"),
                None => {}
            }
            if kind.is_stop() {
                let s = &rule.strings;
                if s.len() < 2
                    || s.len() > 8
                    || s.iter().any(|&p| !(21..=108).contains(&p))
                    || s.windows(2).any(|w| w[0] >= w[1])
                {
                    return Err(ProfileError::new(
                        f("strings"),
                        "2..8 hauteurs MIDI (21..108) strictement croissantes requises",
                    ));
                }
                if rule.min_interval == 0
                    || rule.min_interval > rule.max_interval
                    || rule.max_interval > 36
                {
                    return Err(ProfileError::new(
                        f("max_interval"),
                        "1 <= min_interval <= max_interval <= 36 demi-tons requis",
                    ));
                }
            } else if !rule.strings.is_empty() {
                return reject("strings");
            }
            if kind == OrnamentKind::Passing {
                if rule.written_per_100_notes != 0.0 || !rule.per_100_notes_by_bar.is_empty() {
                    return Err(ProfileError::new(
                        f("written_per_100_notes"),
                        "la note de passage n'a pas de taux : elle suit density_curve",
                    ));
                }
                let Some(curve) = &rule.density_curve else {
                    return Err(ProfileError::new(f("density_curve"), "courbe de densité requise"));
                };
                curve.validate("density_curve", 0.0, 1.0).map_err(|m| {
                    ProfileError::new(f("density_curve"), m.trim_start_matches("density_curve: "))
                })?;
            } else if rule.density_curve.is_some() {
                return reject("density_curve");
            }
        }
        Ok(())
    }

    /// Réalise les ornements d'une mesure de mélodie déjà interprétée. `written` : durées
    /// écrites des notes de mélodie, avant la dynamique émotionnelle, dans l'ordre du
    /// flux ; elles décident de l'éligibilité, pour que les reprises gardent les mêmes
    /// ornements quand les émotions bougent. Retourne vrai si une note a changé.
    pub fn apply(
        &self,
        measure: &mut crate::timeline::Measure,
        track: crate::timeline::TrackId,
        ctx: &BarContext,
        written: &[usize],
        next_id: &mut dyn FnMut() -> crate::timeline::NoteId,
    ) -> bool {
        realize::apply(self, measure, track, ctx, written, next_id)
    }
}

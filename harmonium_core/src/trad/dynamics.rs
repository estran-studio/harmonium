//! Courbes de pilotage du style : aucune valeur musicale dans l'algorithme.

use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use super::{Mode, TunePlan};
use crate::{harmony::RngCore, timeline::TrackId};

pub const REEL_INTENSITY_TOML: &str = include_str!("reel_intensity.toml");

/// Courbe continue sur un axe normalisé, extrapolation constante aux extrémités.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Curve(pub Vec<[f32; 2]>);

impl Curve {
    #[must_use]
    pub fn eval(&self, input: f32) -> f32 {
        let Some(first) = self.0.first() else { return 0.0 };
        let x = if input.is_finite() { input.clamp(0.0, 1.0) } else { 0.5 };
        if x <= first[0] {
            return first[1];
        }
        for pair in self.0.windows(2) {
            let [a, b] = [pair[0], pair[1]];
            if x <= b[0] {
                return ((x - a[0]) / (b[0] - a[0])).mul_add(b[1] - a[1], a[1]);
            }
        }
        self.0.last().map_or(first[1], |p| p[1])
    }

    fn validate(&self, name: &str, min: f32, max: f32) -> Result<(), String> {
        if self.0.len() < 2
            || self.0.first().is_none_or(|p| p[0] != 0.0)
            || self.0.last().is_none_or(|p| p[0] != 1.0)
        {
            return Err(format!("{name}: au moins deux points couvrant 0..1 requis"));
        }
        if self.0.iter().any(|p| {
            !p[0].is_finite()
                || !p[1].is_finite()
                || !(0.0..=1.0).contains(&p[0])
                || !(min..=max).contains(&p[1])
        }) || self.0.windows(2).any(|p| p[0][0] >= p[1][0])
        {
            return Err(format!(
                "{name}: abscisses strictement croissantes dans 0..1 et valeurs finies dans {min}..{max} requises"
            ));
        }
        Ok(())
    }
}

/// Palette autorisée : le poids est une fonction de la valence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorChoice {
    pub key: u8,
    pub mode: Mode,
    pub weight: Curve,
}

/// Section générique du futur profil de style, chargée dans `TuningParams`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleDynamics {
    pub arousal_intensity: Curve,
    pub tension_intensity: Curve,
    pub tempo_offset: Curve,
    pub melody_velocity: Curve,
    pub guitar_velocity: Curve,
    pub bass_velocity: Curve,
    pub feet_velocity: Curve,
    pub neutral_valence: f32,
    /// Domaine de l'API émotionnelle, normalisé avant les courbes de couleur.
    pub valence_range: [f32; 2],
    pub feet_intensity: Curve,
    pub fiddle_duration: Curve,
    pub strong_accent: Curve,
    pub chop_intensity: Curve,
    pub run_weight: Curve,
    pub arp_weight: Curve,
    pub turn_weight: Curve,
    pub color: Vec<ColorChoice>,
    pub color_register: Curve,
    pub brightness_accent: Curve,
    pub density_feet: Curve,
    pub density_chop: Curve,
    pub density_passing: Curve,
    /// Seuils d'entrée des pistes selon la densité, après les plafonds du jeu.
    pub density_tracks: Vec<TrackThreshold>,
    pub chop_patterns: Vec<Vec<usize>>,
    pub tension_offbeat: Curve,
    pub tension_duration: Curve,
    pub tension_dominant: Curve,
    pub tension_chord_duration: Curve,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackThreshold {
    pub track: TrackId,
    pub minimum: f32,
}

impl Default for StyleDynamics {
    fn default() -> Self {
        static EMBEDDED: LazyLock<StyleDynamics> =
            LazyLock::new(|| match StyleDynamics::from_toml(REEL_INTENSITY_TOML) {
                Ok(data) => data,
                // Même contrat que les autres tables embarquées : un défaut
                // illisible reste INVALIDE, détecté par TuningParams::validate,
                // jamais remplacé par un deuxième jeu de valeurs musicales.
                Err(_) => {
                    let empty = || Curve(Vec::new());
                    StyleDynamics {
                        arousal_intensity: empty(),
                        tension_intensity: empty(),
                        tempo_offset: empty(),
                        melody_velocity: empty(),
                        guitar_velocity: empty(),
                        bass_velocity: empty(),
                        feet_velocity: empty(),
                        neutral_valence: 0.0,
                        valence_range: [0.0, 0.0],
                        feet_intensity: empty(),
                        fiddle_duration: empty(),
                        strong_accent: empty(),
                        chop_intensity: empty(),
                        run_weight: empty(),
                        arp_weight: empty(),
                        turn_weight: empty(),
                        color: Vec::new(),
                        color_register: empty(),
                        brightness_accent: empty(),
                        density_feet: empty(),
                        density_chop: empty(),
                        density_passing: empty(),
                        density_tracks: Vec::new(),
                        chop_patterns: Vec::new(),
                        tension_offbeat: empty(),
                        tension_duration: empty(),
                        tension_dominant: empty(),
                        tension_chord_duration: empty(),
                    }
                }
            });
        EMBEDDED.clone()
    }
}

impl StyleDynamics {
    pub fn from_toml(text: &str) -> Result<Self, String> {
        let data: Self = toml::from_str(text).map_err(|e| format!("reel_intensity: {e}"))?;
        data.validate()?;
        Ok(data)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.neutral_valence.is_finite()
            || !self.valence_range.iter().all(|v| v.is_finite())
            || self.valence_range[0] >= self.valence_range[1]
            || !(self.valence_range[0]..=self.valence_range[1]).contains(&self.neutral_valence)
        {
            return Err(
                "valence_range: bornes finies croissantes contenant neutral_valence requises"
                    .into(),
            );
        }
        for (name, curve, min, max) in [
            ("arousal_intensity", &self.arousal_intensity, 0.0, 1.0),
            ("tension_intensity", &self.tension_intensity, -1.0, 1.0),
            ("tempo_offset", &self.tempo_offset, -30.0, 30.0),
            ("melody_velocity", &self.melody_velocity, 0.1, 2.0),
            ("guitar_velocity", &self.guitar_velocity, 0.1, 2.0),
            ("bass_velocity", &self.bass_velocity, 0.1, 2.0),
            ("feet_velocity", &self.feet_velocity, 0.1, 2.0),
            ("feet_intensity", &self.feet_intensity, -2.0, 2.0),
            ("fiddle_duration", &self.fiddle_duration, 0.1, 2.0),
            ("strong_accent", &self.strong_accent, -32.0, 32.0),
            ("chop_intensity", &self.chop_intensity, -8.0, 8.0),
            ("run_weight", &self.run_weight, 0.01, 16.0),
            ("arp_weight", &self.arp_weight, 0.01, 16.0),
            ("turn_weight", &self.turn_weight, 0.01, 16.0),
            ("color_register", &self.color_register, -12.0, 12.0),
            ("brightness_accent", &self.brightness_accent, -32.0, 32.0),
            ("density_feet", &self.density_feet, -2.0, 2.0),
            ("density_chop", &self.density_chop, -8.0, 8.0),
            ("density_passing", &self.density_passing, 0.0, 1.0),
            ("tension_offbeat", &self.tension_offbeat, -32.0, 32.0),
            ("tension_duration", &self.tension_duration, 0.1, 2.0),
            ("tension_dominant", &self.tension_dominant, 0.0, 1.0),
            ("tension_chord_duration", &self.tension_chord_duration, 1.0, 8.0),
        ] {
            curve.validate(name, min, max)?;
        }
        if self.color.is_empty() {
            return Err("color: palette vide".into());
        }
        for choice in &self.color {
            if choice.key > 11 {
                return Err("color.key: attendu 0..11".into());
            }
            choice.weight.validate("color.weight", 0.0, 1000.0)?;
        }
        // Vérifier chaque cassure de la palette : une somme linéaire positive
        // à ses extrémités est positive sur l'intervalle entier.
        for x in self.color.iter().flat_map(|c| c.weight.0.iter().map(|p| p[0])) {
            if self.color.iter().map(|c| c.weight.eval(x)).sum::<f32>() <= 0.0 {
                return Err("color: poids total nul".into());
            }
        }
        if self.chop_patterns.is_empty()
            || self.chop_patterns.iter().any(|p| {
                p.is_empty() || p.iter().any(|&s| s >= 16) || p.windows(2).any(|s| s[0] >= s[1])
            })
        {
            return Err("chop_patterns: motifs non vides, pas croissants dans 0..15 requis".into());
        }
        for (index, threshold) in self.density_tracks.iter().enumerate() {
            if !threshold.minimum.is_finite()
                || !(0.0..=1.0).contains(&threshold.minimum)
                || self.density_tracks[..index].iter().any(|t| t.track == threshold.track)
            {
                return Err("density_tracks: seuil unique par piste dans 0..1 requis".into());
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn intensity_from_emotions(&self, arousal: f32, tension: f32) -> f32 {
        (self.arousal_intensity.eval(arousal) + self.tension_intensity.eval(tension))
            .clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn tempo_for(&self, base_bpm: f32, intensity: f32) -> f32 {
        (base_bpm + self.tempo_offset.eval(intensity)).clamp(40.0, 300.0)
    }

    #[must_use]
    pub fn allows(&self, track: TrackId, density: f32) -> bool {
        self.density_tracks.iter().find(|t| t.track == track).is_none_or(|t| density >= t.minimum)
    }

    #[must_use]
    pub fn velocity(&self, track: TrackId, intensity: f32) -> f32 {
        match track {
            TrackId::Melody => &self.melody_velocity,
            TrackId::Chords => &self.guitar_velocity,
            TrackId::Bass => &self.bass_velocity,
            TrackId::Heel | TrackId::Toe => &self.feet_velocity,
        }
        .eval(intensity)
    }

    #[must_use]
    pub fn valence_input(&self, valence: f32) -> f32 {
        ((valence - self.valence_range[0]) / (self.valence_range[1] - self.valence_range[0]))
            .clamp(0.0, 1.0)
    }

    pub fn draw_color(&self, valence: f32, rng: &mut dyn RngCore) -> TunePlan {
        let valence = self.valence_input(valence);
        let total: f32 = self.color.iter().map(|c| c.weight.eval(valence)).sum();
        let mut pick = rng.next_range_usize(0..1_000_000) as f32 / 1_000_000.0 * total;
        for choice in &self.color {
            let weight = choice.weight.eval(valence);
            if pick < weight {
                return TunePlan::reel(choice.key, choice.mode);
            }
            pick -= weight;
        }
        self.color.last().map_or_else(TunePlan::default, |c| TunePlan::reel(c.key, c.mode))
    }
}

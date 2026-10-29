//! Instrument Odin par piste, en données.
//!
//! Chaque piste garde son programme SoundFont (rendu sans `odin`) et peut
//! déclarer des zones d'émotion, chacune avec un preset Odin fourni par l'hôte
//! et ses réglages de morphing. Sans `odin` compilé ou sans le preset de la
//! zone active, la piste joue son programme SoundFont (avertissement au
//! chargement du profil) ; avec le preset présent, elle joue Odin.
//! Le thread principal connaît les socles des presets ([`OdinBank`]), calcule
//! le preset morphé de chaque piste à chaque changement d'émotion et envoie au
//! callback une charge `Copy` ([`OdinPatches`]) : le fil audio ne calcule
//! rien. Aucun nom de style ni canal fixe ici : tout est clé par
//! [`TrackId`](crate::timeline::TrackId).
//!
//! ```toml
//! [[odin.Bass.zones]]
//! preset = "basse_saturee"
//! morph = [
//!   { param = "fil1_freq", arousal = 3500.0, tension = 1500.0 },
//!   { param = "fil1_res", tension = 0.35 },
//! ]
//! ```

use serde::{Deserialize, Serialize};

use crate::timeline::TrackId;

/// Nom de preset maximal (octets SF2 exclus, nom court fourni par l'hôte).
pub const ODIN_PRESET_NAME_CAP: usize = 64;
/// Règles de morphing maximales par zone.
pub const ODIN_MAX_RULES: usize = 16;
/// Zones maximales par piste, dont le défaut final.
pub const ODIN_MAX_ZONES: usize = 4;
/// Marge d'hystérésis, en unités d'émotion, autour des bornes de zone.
pub const ODIN_ZONE_HYSTERESIS: f32 = 0.03;

// Noms et bornes du catalogue odin-rs 4718189. La table reste indépendante
// de la dépendance GPL et définit aussi l'ordre du tableau transmis au rendu.
macro_rules! morph_params {
    ($($variant:ident => ($name:literal, $min:expr, $max:expr)),+ $(,)?) => {
        /// Paramètre continu du catalogue Odin, dans son unité physique.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[repr(usize)]
        pub enum OdinMorphParam {
            $(#[serde(rename = $name)] $variant),+
        }

        impl OdinMorphParam {
            pub const ALL: [Self; Self::COUNT] = [$(Self::$variant),+];
            pub const COUNT: usize = [$(stringify!($variant)),+].len();

            #[must_use]
            pub const fn index(self) -> usize { self as usize }

            #[must_use]
            pub const fn name(self) -> &'static str {
                match self { $(Self::$variant => $name),+ }
            }

            #[must_use]
            pub const fn bounds(self) -> (f32, f32) {
                match self { $(Self::$variant => ($min, $max)),+ }
            }
        }
    };
}

morph_params! {
    Osc1Vol => ("osc1_vol", -60.0, 12.0),
    Osc2Vol => ("osc2_vol", -60.0, 12.0),
    Osc3Vol => ("osc3_vol", -60.0, 12.0),
    Osc1Detune => ("osc1_detune", 0.0, 1.0),
    Osc2Detune => ("osc2_detune", 0.0, 1.0),
    Osc3Detune => ("osc3_detune", 0.0, 1.0),
    Fil1Freq => ("fil1_freq", 20.0, 20_000.0),
    Fil1Res => ("fil1_res", 0.0, 1.0),
    Fil1Env => ("fil1_env", -1.0, 1.0),
    Fil1Saturation => ("fil1_saturation", 0.0, 1.0),
    Fil1Gain => ("fil1_gain", -60.0, 12.0),
    Fil2Freq => ("fil2_freq", 20.0, 20_000.0),
    Fil2Res => ("fil2_res", 0.0, 1.0),
    Fil2Env => ("fil2_env", -1.0, 1.0),
    Fil2Saturation => ("fil2_saturation", 0.0, 1.0),
    Fil2Gain => ("fil2_gain", -60.0, 12.0),
    Fil3Freq => ("fil3_freq", 20.0, 20_000.0),
    Fil3Res => ("fil3_res", 0.0, 1.0),
    Fil3Env => ("fil3_env", -1.0, 1.0),
    Fil3Saturation => ("fil3_saturation", 0.0, 1.0),
    Fil3Gain => ("fil3_gain", -60.0, 12.0),
    Env1Attack => ("env1_attack", 0.001, 10.0),
    Env1Decay => ("env1_decay", 0.001, 10.0),
    Env1Sustain => ("env1_sustain", 0.0, 1.0),
    Env1Release => ("env1_release", 0.001, 10.0),
    Env2Attack => ("env2_attack", 0.001, 10.0),
    Env2Decay => ("env2_decay", 0.001, 10.0),
    Env2Sustain => ("env2_sustain", 0.0, 1.0),
    Env2Release => ("env2_release", 0.001, 10.0),
    Env3Attack => ("env3_attack", 0.001, 10.0),
    Env3Decay => ("env3_decay", 0.001, 10.0),
    Env3Sustain => ("env3_sustain", 0.0, 1.0),
    Env3Release => ("env3_release", 0.001, 10.0),
    Env4Attack => ("env4_attack", 0.001, 10.0),
    Env4Decay => ("env4_decay", 0.001, 10.0),
    Env4Sustain => ("env4_sustain", 0.0, 1.0),
    Env4Release => ("env4_release", 0.001, 10.0),
    Master => ("master", -50.0, 12.0),
    DistBoost => ("dist_boost", 0.0, 1.0),
    DistDrywet => ("dist_drywet", 0.0, 1.0),
    DelayWet => ("delay_wet", 0.0, 1.0),
    PhaserDrywet => ("phaser_drywet", 0.0, 1.0),
    FlangerDrywet => ("flanger_drywet", 0.0, 1.0),
    ChorusDrywet => ("chorus_drywet", 0.0, 1.0),
    RevDrywet => ("rev_drywet", 0.0, 1.0),
}

/// Une règle de morphing : gains additifs par émotion (valence `-1..1`, autres `0..1`).
///
/// `valeur = base + arousal × gain + valence × gain + density × gain +
/// tension × gain`, bornée par paramètre (voir [`OdinVoiceParams::morphed`]).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OdinMorphRule {
    pub param: OdinMorphParam,
    #[serde(default)]
    pub arousal: f32,
    #[serde(default)]
    pub valence: f32,
    #[serde(default)]
    pub density: f32,
    #[serde(default)]
    pub tension: f32,
}

/// Instrument Odin d'une piste, en données du profil.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OdinTrackSpec {
    /// Zones prioritaires dans l'ordre ; la dernière est le défaut sans `when`.
    pub zones: Vec<OdinZone>,
}

/// Preset, niveau et morphing propres à une zone d'émotion.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OdinZone {
    #[serde(default)]
    pub when: Option<OdinZoneWhen>,
    /// Nom du preset, fourni par l'hôte (`<nom>.odin`) ; sans `/` ni `\`.
    pub preset: String,
    /// Niveau relatif au mix de piste, en dB (`-60..12`, défaut : 0).
    #[serde(default)]
    pub level_db: f32,
    #[serde(default)]
    pub morph: Vec<OdinMorphRule>,
}

/// Intersection de bornes inclusives ; au moins un axe doit être renseigné.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OdinZoneWhen {
    pub arousal: Option<[f32; 2]>,
    pub valence: Option<[f32; 2]>,
    pub density: Option<[f32; 2]>,
    pub tension: Option<[f32; 2]>,
}

impl OdinZoneWhen {
    fn contains(self, emotions: [f32; 4], margin: f32) -> bool {
        [self.arousal, self.valence, self.density, self.tension]
            .into_iter()
            .zip(emotions)
            .enumerate()
            .all(|(axis, (bounds, value))| {
                bounds.is_none_or(|[min, max]| {
                    let floor = if axis == 1 { -1.0 } else { 0.0 };
                    let midpoint = min + (max - min) * 0.5;
                    // Les extrêmes du domaine restent atteignables à l'entrée.
                    let lower = if min == floor { floor } else { (min - margin).min(midpoint) };
                    let upper = if max == 1.0 { 1.0 } else { (max + margin).max(midpoint) };
                    (lower..=upper).contains(&value)
                })
            })
    }
}

impl OdinTrackSpec {
    /// Première zone admissible, sinon défaut final. Le profil doit être validé.
    ///
    /// Sans historique, les bornes sont inclusives. Avec historique, l'entrée
    /// exige des bornes resserrées de 0,03 (au plus la demi-largeur de l'axe).
    /// Une zone bornée active reste dans ses bornes élargies de 0,03, sauf si
    /// une zone plus prioritaire satisfait ses bornes resserrées.
    #[must_use]
    pub fn select_zone(&self, emotions: [f32; 4], previous: Option<usize>) -> usize {
        let previous = previous.filter(|&index| index < self.zones.len());
        let margin = if previous.is_some() { -ODIN_ZONE_HYSTERESIS } else { 0.0 };
        let candidate = self
            .zones
            .iter()
            .position(|zone| zone.when.is_some_and(|when| when.contains(emotions, margin)));
        if let Some(index) = previous
            && candidate.is_none_or(|candidate| candidate >= index)
            && let Some(when) = self.zones[index].when
            && when.contains(emotions, ODIN_ZONE_HYSTERESIS)
        {
            return index;
        }
        candidate.unwrap_or_else(|| self.zones.len().saturating_sub(1))
    }
}

/// Paramètres continus d'une voix Odin : socle du preset (lu du `.odin` par
/// l'hôte, hors callback) comme valeurs après morphing.
///
/// Tableau fixe indexé par [`OdinMorphParam::index`], sans dépendance GPL.
/// Les valeurs sont celles du catalogue (Hz, secondes, dB ou rapports).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OdinVoiceParams {
    pub values: [f32; OdinMorphParam::COUNT],
}

impl OdinVoiceParams {
    /// Morphing déterministe, sans allocation : optique `f32` pure. `self` est
    /// le socle du preset ; le résultat est borné par [`Self::clamp`].
    #[must_use]
    pub fn morphed(&self, rules: &[OdinMorphRule], emotions: [f32; 4]) -> Self {
        let [arousal, valence, density, tension] = emotions;
        let mut out = *self;
        for rule in rules {
            let delta = arousal.mul_add(
                rule.arousal,
                valence
                    .mul_add(rule.valence, density.mul_add(rule.density, tension * rule.tension)),
            );
            out.values[rule.param.index()] += delta;
        }
        out.clamp()
    }

    /// Bornes physiques du catalogue, identiques pour socles et morphing.
    #[must_use]
    pub fn clamp(mut self) -> Self {
        for param in OdinMorphParam::ALL {
            let (min, max) = param.bounds();
            self.values[param.index()] = self.values[param.index()].clamp(min, max);
        }
        self
    }
}

/// Socle d'un preset fourni par l'hôte, analysé hors callback.
#[derive(Clone, Debug, PartialEq)]
pub struct OdinPresetBase {
    /// Nom court désigné dans `[[odin.<Piste>.zones]]`.
    pub name: String,
    pub base: OdinVoiceParams,
}

/// Charge `Copy` prête à appliquer pour une piste Odin : le preset morphé
/// par le thread principal. Le callback audio ne calcule rien, il copie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OdinTrackPatch {
    /// Rang de la zone dans le profil : deux zones peuvent partager un preset.
    pub zone: usize,
    /// Rang du preset dans la [`OdinBank`] de l'hôte.
    pub preset: usize,
    /// Gain linéaire propre à la zone, préparé hors callback.
    pub gain: f32,
    pub params: OdinVoiceParams,
}

/// Charges des 6 pistes, indexées par [`TrackId`] (`None` : piste SoundFont).
pub type OdinPatches = [Option<OdinTrackPatch>; TrackId::ALL.len()];

/// Presets fournis par l'hôte, connus du thread principal : socles à partir
/// desquels il prépare les charges du callback.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OdinBank {
    presets: Vec<OdinPresetBase>,
}

impl OdinBank {
    #[must_use]
    pub fn new(presets: Vec<OdinPresetBase>) -> Self {
        Self { presets }
    }

    /// Preset chargé sous ce nom.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.presets.iter().any(|preset| preset.name == name)
    }

    /// Charges initiales sans historique de zone. Les pistes dont le preset
    /// de la zone sélectionnée n'est pas chargé restent SoundFont (`None`).
    #[must_use]
    pub fn patches(
        &self,
        specs: &std::collections::BTreeMap<TrackId, OdinTrackSpec>,
        emotions: [f32; 4],
    ) -> OdinPatches {
        self.patches_for_zones(specs, emotions, [None; TrackId::ALL.len()])
    }

    /// Charges avec les zones explicitement choisies par le compositeur ;
    /// `None` sélectionne la zone initiale, sans historique.
    #[must_use]
    pub fn patches_for_zones(
        &self,
        specs: &std::collections::BTreeMap<TrackId, OdinTrackSpec>,
        emotions: [f32; 4],
        zones: [Option<usize>; TrackId::ALL.len()],
    ) -> OdinPatches {
        let mut patches = [None; TrackId::ALL.len()];
        for (track, spec) in specs {
            let channel = usize::from(track.channel());
            let zone_index = zones[channel].unwrap_or_else(|| spec.select_zone(emotions, None));
            let Some(zone) = spec.zones.get(zone_index) else {
                continue;
            };
            let found = self.presets.iter().enumerate().find(|(_, p)| p.name == zone.preset);
            if let Some((index, preset)) = found {
                patches[channel] = Some(OdinTrackPatch {
                    zone: zone_index,
                    preset: index,
                    gain: if zone.level_db == 0.0 {
                        1.0
                    } else {
                        10.0_f32.powf(zone.level_db / 20.0)
                    },
                    params: preset.base.morphed(&zone.morph, emotions),
                });
            }
        }
        patches
    }
}

//! Instrument Odin par piste, en données.
//!
//! Chaque piste garde son programme SoundFont (rendu sans `odin`) et peut
//! déclarer un preset Odin (nom fourni par l'hôte comme les octets SF2) avec
//! ses réglages de morphing par les 4 émotions. Sans `odin` compilé ou sans
//! preset fourni, la piste joue son programme SoundFont ; avec `odin` et le
//! preset présent, elle joue Odin. Aucun nom de style ni canal fixe ici :
//! tout est clé par [`TrackId`](crate::timeline::TrackId).
//!
//! ```toml
//! [odin.Bass]
//! preset = "basse_saturee"
//! morph = [
//!   { param = "filter_frequency", arousal = 3500.0, tension = 1500.0 },
//!   { param = "filter_resonance", tension = 0.35 },
//! ]
//! ```

use arrayvec::ArrayString;
use serde::{Deserialize, Serialize};

use crate::timeline::TrackId;

/// Nom de preset maximal (octets SF2 exclus, nom court fourni par l'hôte).
pub const ODIN_PRESET_NAME_CAP: usize = 64;
/// Règles de morphing maximales par piste (tableau fixe, `Copy` pour le callback).
pub const ODIN_MAX_RULES: usize = 8;

/// Paramètre continu du preset pilotable par les 4 émotions.
///
/// Sous-ensemble des champs de `PresetConfig` d'`odin2-core` (ceux que le
/// moteur expose : pas de drive ni d'effets dans la configuration
/// simplifiée). Les champs discrets (octaves, demi-tons) ne se morphent pas :
/// ils resteraient faux entre deux valeurs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OdinMorphParam {
    FilterFrequency,
    FilterResonance,
    FilterEnvAmount,
    AmpAttack,
    AmpDecay,
    AmpSustain,
    AmpRelease,
    FilterAttack,
    FilterDecay,
    FilterSustain,
    FilterRelease,
    OscDetune,
    Osc1Volume,
    Osc2Volume,
    Osc3Volume,
    MasterVolume,
}

/// Une règle de morphing : gains additifs par émotion (émotions `0..1`).
///
/// `valeur = base + arousal × gain + valence × gain + density × gain +
/// tension × gain`, bornée par paramètre (voir [`OdinVoiceParams::apply`]).
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
    /// Nom du preset, fourni par l'hôte (`<nom>.odin`) ; sans `/` ni `\`.
    pub preset: String,
    /// Règles de morphing (défaut : preset fixe, insensible aux émotions).
    #[serde(default)]
    pub morph: Vec<OdinMorphRule>,
}

/// Socle du preset : valeurs lues du `.odin` par l'hôte audio, hors callback.
///
/// Mêmes champs continus que `PresetConfig` d'`odin2-core`, en `f32` purs :
/// aucune dépendance GPL dans ce module.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OdinBaseParams {
    pub osc_volumes: [f32; 3],
    pub osc_detune: f32,
    pub filter_frequency: f32,
    pub filter_resonance: f32,
    pub filter_env_amount: f32,
    pub amp_attack: f32,
    pub amp_decay: f32,
    pub amp_sustain: f32,
    pub amp_release: f32,
    pub filter_attack: f32,
    pub filter_decay: f32,
    pub filter_sustain: f32,
    pub filter_release: f32,
    pub master_volume: f32,
}

/// Paramètres d'une voix après morphing (même disposition que le socle).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OdinVoiceParams {
    pub osc_volumes: [f32; 3],
    pub osc_detune: f32,
    pub filter_frequency: f32,
    pub filter_resonance: f32,
    pub filter_env_amount: f32,
    pub amp_attack: f32,
    pub amp_decay: f32,
    pub amp_sustain: f32,
    pub amp_release: f32,
    pub filter_attack: f32,
    pub filter_decay: f32,
    pub filter_sustain: f32,
    pub filter_release: f32,
    pub master_volume: f32,
}

impl OdinVoiceParams {
    /// Morphing déterministe, sans allocation : optique `f32` pure.
    #[must_use]
    pub fn morphed(base: &OdinBaseParams, rules: &[OdinMorphRule], emotions: [f32; 4]) -> Self {
        let [arousal, valence, density, tension] = emotions;
        let mut out = Self {
            osc_volumes: base.osc_volumes,
            osc_detune: base.osc_detune,
            filter_frequency: base.filter_frequency,
            filter_resonance: base.filter_resonance,
            filter_env_amount: base.filter_env_amount,
            amp_attack: base.amp_attack,
            amp_decay: base.amp_decay,
            amp_sustain: base.amp_sustain,
            amp_release: base.amp_release,
            filter_attack: base.filter_attack,
            filter_decay: base.filter_decay,
            filter_sustain: base.filter_sustain,
            filter_release: base.filter_release,
            master_volume: base.master_volume,
        };
        for rule in rules {
            let delta = arousal.mul_add(
                rule.arousal,
                valence.mul_add(rule.valence, density.mul_add(rule.density, tension * rule.tension)),
            );
            let slot: &mut f32 = match rule.param {
                OdinMorphParam::FilterFrequency => &mut out.filter_frequency,
                OdinMorphParam::FilterResonance => &mut out.filter_resonance,
                OdinMorphParam::FilterEnvAmount => &mut out.filter_env_amount,
                OdinMorphParam::AmpAttack => &mut out.amp_attack,
                OdinMorphParam::AmpDecay => &mut out.amp_decay,
                OdinMorphParam::AmpSustain => &mut out.amp_sustain,
                OdinMorphParam::AmpRelease => &mut out.amp_release,
                OdinMorphParam::FilterAttack => &mut out.filter_attack,
                OdinMorphParam::FilterDecay => &mut out.filter_decay,
                OdinMorphParam::FilterSustain => &mut out.filter_sustain,
                OdinMorphParam::FilterRelease => &mut out.filter_release,
                OdinMorphParam::OscDetune => &mut out.osc_detune,
                OdinMorphParam::Osc1Volume => &mut out.osc_volumes[0],
                OdinMorphParam::Osc2Volume => &mut out.osc_volumes[1],
                OdinMorphParam::Osc3Volume => &mut out.osc_volumes[2],
                OdinMorphParam::MasterVolume => &mut out.master_volume,
            };
            *slot += delta;
        }
        out.clamp()
    }

    /// Bornes du moteur : le filtre reste audible, les enveloppes valides.
    #[must_use]
    pub fn clamp(mut self) -> Self {
        self.filter_frequency = self.filter_frequency.clamp(20.0, 20_000.0);
        self.filter_resonance = self.filter_resonance.clamp(0.0, 1.0);
        self.filter_env_amount = self.filter_env_amount.clamp(-1.0, 1.0);
        self.amp_attack = self.amp_attack.clamp(0.001, 5.0);
        self.amp_decay = self.amp_decay.clamp(0.001, 5.0);
        self.amp_sustain = self.amp_sustain.clamp(0.0, 1.0);
        self.amp_release = self.amp_release.clamp(0.01, 10.0);
        self.filter_attack = self.filter_attack.clamp(0.001, 5.0);
        self.filter_decay = self.filter_decay.clamp(0.001, 5.0);
        self.filter_sustain = self.filter_sustain.clamp(0.0, 1.0);
        self.filter_release = self.filter_release.clamp(0.01, 10.0);
        self.osc_detune = self.osc_detune.clamp(0.0, 1.0);
        for volume in &mut self.osc_volumes {
            *volume = volume.clamp(0.0, 2.0);
        }
        self.master_volume = self.master_volume.clamp(0.0, 2.0);
        self
    }
}

/// Vue `Copy` d'une piste Odin pour le callback audio (nom + règles inline).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OdinTrackDesc {
    pub preset: ArrayString<ODIN_PRESET_NAME_CAP>,
    pub rules: [OdinMorphRule; ODIN_MAX_RULES],
    pub rule_count: u8,
}

impl Default for OdinTrackDesc {
    fn default() -> Self {
        Self {
            preset: ArrayString::new(),
            rules: [OdinMorphRule {
                param: OdinMorphParam::MasterVolume,
                arousal: 0.0,
                valence: 0.0,
                density: 0.0,
                tension: 0.0,
            }; ODIN_MAX_RULES],
            rule_count: 0,
        }
    }
}

impl OdinTrackDesc {
    /// Règles actives (jamais plus de [`ODIN_MAX_RULES`]).
    #[must_use]
    pub fn rules(&self) -> &[OdinMorphRule] {
        let count = (self.rule_count as usize).min(ODIN_MAX_RULES);
        &self.rules[..count]
    }

    /// Piste vide : aucun preset déclaré.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.preset.is_empty()
    }
}

/// Erreur de validation d'une section `[odin]` (champ fautif nommé).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OdinProfileError {
    pub field: String,
    pub message: String,
}

impl OdinTrackSpec {
    /// Convertit en vue `Copy` (échec si le nom déborde ou trop de règles).
    pub fn describe(&self, track: TrackId) -> Result<OdinTrackDesc, OdinProfileError> {
        let field = format!("odin.{}.preset", track.name());
        if self.preset.is_empty() || self.preset.len() > ODIN_PRESET_NAME_CAP {
            return Err(OdinProfileError {
                field,
                message: "nom de preset non vide de 64 caractères au plus".to_string(),
            });
        }
        if self.preset.contains('/') || self.preset.contains('\\') {
            return Err(OdinProfileError {
                field,
                message: "nom de preset sans chemin (`<nom>.odin` fourni par l'hôte)".to_string(),
            });
        }
        if self.morph.len() > ODIN_MAX_RULES {
            return Err(OdinProfileError {
                field: format!("odin.{}.morph", track.name()),
                message: format!("au plus {ODIN_MAX_RULES} règles par piste"),
            });
        }
        let mut desc = OdinTrackDesc::default();
        for ch in self.preset.chars() {
            if desc.preset.try_push(ch).is_err() {
                return Err(OdinProfileError {
                    field,
                    message: "nom de preset de 64 caractères au plus".to_string(),
                });
            }
        }
        for (slot, rule) in desc.rules.iter_mut().zip(self.morph.iter()) {
            for gain in [rule.arousal, rule.valence, rule.density, rule.tension] {
                if !gain.is_finite() {
                    return Err(OdinProfileError {
                        field: format!("odin.{}.morph", track.name()),
                        message: "gains de morphing finis".to_string(),
                    });
                }
            }
            *slot = *rule;
        }
        desc.rule_count = self.morph.len() as u8;
        Ok(desc)
    }
}

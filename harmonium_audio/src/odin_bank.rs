//! Analyse des presets Odin fournis par l'hôte, sur le thread principal.
//!
//! L'hôte passe des octets `.odin` ; le thread principal en tire les socles
//! ([`OdinBank`]) dont il prépare les presets morphés (`OdinBank::patches`),
//! hors du callback. Sans l'option `odin` (GPLv3 via `odin2-core`), rien
//! n'est analysé : la banque est vide et les profils qui déclarent un preset
//! reçoivent l'avertissement de chargement (`StyleProfile::missing_resources`).
//!
//! `read_preset` est l'unique lecture d'un `PresetConfig` ; son écriture
//! (`odin_voices::preset_config`) est l'unique côté audio. Un paramètre
//! ajouté à `odin2-core` passe par ces deux fonctions et par
//! `harmonium_core::odin::OdinVoiceParams`.

use harmonium_core::odin::OdinBank;

use crate::odin_source::OdinPresetSource;

/// Analyse les presets fournis (hors callback). Les octets illisibles ou les
/// noms invalides sont ignorés avec un avertissement : le profil qui les
/// désigne le signale au chargement.
#[must_use]
pub fn load_bank(presets: &[OdinPresetSource]) -> OdinBank {
    #[cfg(feature = "odin")]
    {
        parse::load(presets)
    }
    #[cfg(not(feature = "odin"))]
    {
        let _ = presets;
        OdinBank::default()
    }
}

#[cfg(feature = "odin")]
mod parse {
    use harmonium_core::odin::{
        ODIN_PRESET_NAME_CAP, OdinBank, OdinDiscreteParams, OdinPresetBase, OdinVoiceParams,
    };
    use odin2_core::{engine::PresetConfig, preset::OdinPreset};

    use crate::odin_source::OdinPresetSource;

    pub(super) fn load(presets: &[OdinPresetSource]) -> OdinBank {
        let mut bank = Vec::new();
        for source in presets {
            if source.name.is_empty() || source.name.len() > ODIN_PRESET_NAME_CAP {
                harmonium_core::log::warn(&format!(
                    "Preset Odin ignoré : nom vide ou trop long ({ODIN_PRESET_NAME_CAP} octets max)"
                ));
                continue;
            }
            match OdinPreset::from_bytes(source.bytes) {
                Ok(preset) => {
                    let (base, discrete) = read_preset(&PresetConfig::from_preset(&preset));
                    bank.push(OdinPresetBase { name: source.name.to_string(), base, discrete });
                    harmonium_core::log::info(&format!("Preset Odin chargé : {}", source.name));
                }
                Err(error) => {
                    harmonium_core::log::warn(&format!(
                        "Preset Odin `{}` illisible, piste en SoundFont : {error}",
                        source.name
                    ));
                }
            }
        }
        OdinBank::new(bank)
    }

    /// Unique lecture d'un `PresetConfig` : socle continu et champs discrets.
    fn read_preset(config: &PresetConfig) -> (OdinVoiceParams, OdinDiscreteParams) {
        (
            OdinVoiceParams {
                osc_volumes: config.osc_volumes,
                osc_detune: config.osc_detune,
                filter_frequency: config.filter_frequency,
                filter_resonance: config.filter_resonance,
                filter_env_amount: config.filter_env_amount,
                amp_attack: config.amp_attack,
                amp_decay: config.amp_decay,
                amp_sustain: config.amp_sustain,
                amp_release: config.amp_release,
                filter_attack: config.filter_attack,
                filter_decay: config.filter_decay,
                filter_sustain: config.filter_sustain,
                filter_release: config.filter_release,
                master_volume: config.master_volume,
            },
            OdinDiscreteParams { octaves: config.osc_octaves, semitones: config.osc_semitones },
        )
    }
}

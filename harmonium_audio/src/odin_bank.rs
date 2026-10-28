//! Presets complets et socles de morphing, préparés sur le thread principal.
//! Le callback ne charge jamais de preset : il reçoit une instance prête et
//! retourne l'ancienne instance à son propriétaire par SPSC.

use harmonium_core::odin::OdinBank;

use crate::odin_source::OdinPresetSource;

/// Banque publique sans GPL et configurations complètes réservées à l'hôte.
#[derive(Default)]
pub struct LoadedOdinBank {
    pub bank: OdinBank,
    #[cfg(feature = "odin")]
    pub configs: Vec<odin2_core::EngineConfig>,
}

/// Analyse le ValueTree et valide sa matrice hors callback. Un preset refusé
/// n'entre dans aucune des deux banques ; leurs rangs restent identiques.
#[must_use]
pub fn load_bank(presets: &[OdinPresetSource]) -> LoadedOdinBank {
    #[cfg(feature = "odin")]
    {
        parse::load(presets)
    }
    #[cfg(not(feature = "odin"))]
    {
        let _ = presets;
        LoadedOdinBank::default()
    }
}

#[cfg(feature = "odin")]
mod parse {
    use harmonium_core::odin::{
        ODIN_PRESET_NAME_CAP, OdinBank, OdinMorphParam, OdinPresetBase, OdinVoiceParams,
    };
    use odin2_core::{EngineConfig, ModMatrix, ParameterId, preset::OdinPreset};

    use super::LoadedOdinBank;
    use crate::odin_source::OdinPresetSource;

    pub(super) fn load(presets: &[OdinPresetSource]) -> LoadedOdinBank {
        let mut bases = Vec::with_capacity(presets.len());
        let mut configs = Vec::with_capacity(presets.len());
        for source in presets {
            if source.name.is_empty() || source.name.len() > ODIN_PRESET_NAME_CAP {
                harmonium_core::log::warn("Preset Odin ignoré : nom vide ou trop long");
                continue;
            }
            let config = match OdinPreset::from_bytes(source.bytes) {
                Ok(preset) => EngineConfig::from_preset(&preset),
                Err(error) => {
                    harmonium_core::log::warn(&format!(
                        "Preset Odin `{}` illisible : {error}",
                        source.name
                    ));
                    continue;
                }
            };
            if ParameterId::all().any(|id| !id.get(&config).is_finite()) {
                harmonium_core::log::warn(&format!(
                    "Preset Odin `{}` : paramètre non fini",
                    source.name
                ));
                continue;
            }
            if let Err(error) = ModMatrix::new().configure(&config.mod_matrix) {
                harmonium_core::log::warn(&format!(
                    "Preset Odin `{}` : matrice invalide : {error}",
                    source.name
                ));
                continue;
            }
            let values = OdinMorphParam::ALL.map(|param| {
                let id = ParameterId::from_name(param.name())
                    .expect("identifiant du catalogue Odin épinglé");
                id.get(&config)
            });
            bases.push(OdinPresetBase {
                name: source.name.to_owned(),
                base: OdinVoiceParams { values },
            });
            configs.push(config);
            harmonium_core::log::info(&format!("Preset Odin complet chargé : {}", source.name));
        }
        LoadedOdinBank { bank: OdinBank::new(bases), configs }
    }
}

/// Transaction propriétaire : paramètres Copy et instances de remplacement.
/// Après application, `engines` contient les anciennes instances, détruites
/// uniquement lors du retour de cette boîte sur le thread principal.
#[cfg(feature = "odin")]
pub struct OdinUpdate {
    pub patches: harmonium_core::odin::OdinPatches,
    pub engines:
        [Option<Box<odin2_core::OdinEngine>>; harmonium_core::timeline::TrackId::ALL.len()],
}

/// État d'expédition, exclusivement sur le thread principal. Le rang envoyé
/// n'avance qu'après insertion réussie de la transaction dans la file.
#[cfg(feature = "odin")]
pub struct OdinDispatch {
    configs: Vec<odin2_core::EngineConfig>,
    sent: [Option<usize>; harmonium_core::timeline::TrackId::ALL.len()],
    returned: rtrb::Consumer<Box<OdinUpdate>>,
    sample_rate: f32,
}

#[cfg(feature = "odin")]
impl OdinDispatch {
    #[must_use]
    pub fn new(
        configs: Vec<odin2_core::EngineConfig>,
        returned: rtrb::Consumer<Box<OdinUpdate>>,
        sample_rate: f32,
    ) -> Self {
        Self {
            configs,
            sent: [None; harmonium_core::timeline::TrackId::ALL.len()],
            returned,
            sample_rate,
        }
    }

    /// Libération des commandes et instances retirées, hors callback.
    pub fn reclaim(&mut self) {
        while let Ok(update) = self.returned.pop() {
            drop(update);
        }
    }

    /// Prépare une transaction sans modifier l'état acquitté de l'expéditeur.
    pub fn prepare(
        &mut self,
        patches: harmonium_core::odin::OdinPatches,
    ) -> Result<Box<OdinUpdate>, &'static str> {
        if patches
            .iter()
            .flatten()
            .any(|patch| patch.params.values.iter().any(|value| !value.is_finite()))
        {
            return Err("paramètre de morphing Odin non fini");
        }
        let engines = std::array::from_fn(|index| {
            let patch = patches[index]?;
            if self.sent[index] == Some(patch.preset) {
                return None;
            }
            let mut engine = Box::new(odin2_core::OdinEngine::new(self.sample_rate));
            engine
                .load_config(self.configs[patch.preset])
                .expect("matrice validée au chargement de la banque");
            Some(engine)
        });
        Ok(Box::new(OdinUpdate { patches, engines }))
    }

    pub fn sent(&mut self, patches: &harmonium_core::odin::OdinPatches) {
        self.sent = patches.map(|patch| patch.map(|patch| patch.preset));
    }
}

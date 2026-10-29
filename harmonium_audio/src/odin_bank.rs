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

/// Identité d'une zone : deux zones utilisant le même preset restent distinctes.
#[cfg(feature = "odin")]
pub type OdinKeys = [Option<(usize, usize)>; harmonium_core::timeline::TrackId::ALL.len()];

#[cfg(feature = "odin")]
pub fn patch_keys(patches: &harmonium_core::odin::OdinPatches) -> OdinKeys {
    patches.map(|patch| patch.map(|patch| (patch.zone, patch.preset)))
}

/// Acquittement minimal du lecteur, sans horloge : emplacement actif et fondu.
#[cfg(feature = "odin")]
pub struct OdinStatus {
    fading: std::sync::atomic::AtomicBool,
    zones: [std::sync::atomic::AtomicUsize; harmonium_core::timeline::TrackId::ALL.len()],
}

#[cfg(feature = "odin")]
impl OdinStatus {
    pub fn new() -> Self {
        Self {
            fading: std::sync::atomic::AtomicBool::new(false),
            zones: std::array::from_fn(|_| std::sync::atomic::AtomicUsize::new(usize::MAX)),
        }
    }

    pub fn zones(&self) -> [Option<usize>; harmonium_core::timeline::TrackId::ALL.len()] {
        self.zones.each_ref().map(|zone| {
            let zone = zone.load(std::sync::atomic::Ordering::Acquire);
            (zone != usize::MAX).then_some(zone)
        })
    }

    pub fn publish(&self, patches: &harmonium_core::odin::OdinPatches, fading: bool) {
        for (zone, patch) in self.zones.iter().zip(patches) {
            zone.store(
                patch.map_or(usize::MAX, |patch| patch.zone),
                std::sync::atomic::Ordering::Release,
            );
        }
        self.fading.store(fading, std::sync::atomic::Ordering::Release);
    }

    pub fn fading(&self) -> bool {
        self.fading.load(std::sync::atomic::Ordering::Acquire)
    }
}

#[cfg(feature = "odin")]
impl Default for OdinStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// La boîte possède le second emplacement avant la frontière, puis reçoit les
/// instances retirées après le fondu. Elle ne se détruit que sur le thread hôte.
#[cfg(feature = "odin")]
pub struct OdinUpdate {
    pub patches: harmonium_core::odin::OdinPatches,
    pub engines:
        [Option<Box<odin2_core::OdinEngine>>; harmonium_core::timeline::TrackId::ALL.len()],
    pub instance_ids: [u64; harmonium_core::timeline::TrackId::ALL.len()],
    pub accepted: bool,
    pub accepted_keys: OdinKeys,
    pub at_bar: usize,
    pub at_frame: u64,
}

/// Préparation et destruction exclusivement hors callback. Une transition en
/// cours retient le choix le plus récent chez le compositeur, pas dans une file.
#[cfg(feature = "odin")]
pub struct OdinDispatch {
    configs: Vec<odin2_core::EngineConfig>,
    sent: OdinKeys,
    returned: rtrb::Consumer<Box<OdinUpdate>>,
    sample_rate: f32,
    next_instance: u64,
    status: std::sync::Arc<OdinStatus>,
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
            next_instance: 1,
            status: std::sync::Arc::new(OdinStatus::new()),
        }
    }

    #[must_use]
    pub fn status(&self) -> std::sync::Arc<OdinStatus> {
        self.status.clone()
    }

    pub fn active_zones(&self) -> [Option<usize>; harmonium_core::timeline::TrackId::ALL.len()] {
        self.status.zones()
    }

    /// Libération des commandes et instances retirées, hors callback.
    pub fn reclaim(&mut self) {
        while let Ok(update) = self.returned.pop() {
            if !update.accepted {
                self.sent = update.accepted_keys;
            }
            drop(update);
        }
    }

    /// `None` : morphing seul, ou second emplacement encore occupé par le fondu.
    pub fn prepare(
        &mut self,
        patches: harmonium_core::odin::OdinPatches,
    ) -> Result<Option<Box<OdinUpdate>>, &'static str> {
        if patches.iter().flatten().any(|patch| {
            !patch.gain.is_finite() || patch.params.values.iter().any(|value| !value.is_finite())
        }) {
            return Err("paramètre de morphing Odin non fini");
        }
        let keys = patch_keys(&patches);
        if keys == self.sent || self.status.fading() {
            return Ok(None);
        }
        let mut instance_ids = [0; harmonium_core::timeline::TrackId::ALL.len()];
        let engines = std::array::from_fn(|index| {
            let patch = patches[index]?;
            if self.sent[index] == keys[index] {
                return None;
            }
            let mut engine = Box::new(odin2_core::OdinEngine::new(self.sample_rate));
            engine
                .load_config(self.configs[patch.preset])
                .expect("matrice validée au chargement de la banque");
            instance_ids[index] = self.next_instance;
            self.next_instance += 1;
            Some(engine)
        });
        Ok(Some(Box::new(OdinUpdate {
            patches,
            engines,
            instance_ids,
            accepted: true,
            accepted_keys: keys,
            at_bar: 0,
            at_frame: 0,
        })))
    }

    pub fn sent(&mut self, patches: &harmonium_core::odin::OdinPatches) {
        self.sent = patch_keys(patches);
    }
}

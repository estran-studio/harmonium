//! Ressources chargées par l'hôte, et ce qu'un profil en exige.
//!
//! L'hôte fournit des octets (banques SF2, presets Odin) ; un profil les
//! désigne en données (`instruments.percussion_bank`, `[[odin.<Piste>.zones]]`).
//! Quand le profil désigne une ressource que l'hôte n'a pas chargée, le rendu
//! n'a aucun repli implicite : pieds silencieux, ou piste jouée par le
//! programme SoundFont que le profil déclare pour elle. Le chargement du
//! profil le dit clairement au lieu de laisser un silence sans message.

use std::fmt;

use crate::{odin::OdinBank, style_profile::StyleProfile, timeline::TrackId};

/// Ressources effectivement chargées par l'hôte audio.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HostResources {
    /// Banques SF2 portant un kit de percussion jouable : `128` (kit GM de la
    /// banque principale) et/ou la banque de pieds chargée à part.
    pub percussion_banks: Vec<u16>,
    /// Presets Odin analysés (vide sans l'option `odin` ou sans preset fourni).
    pub odin: OdinBank,
}

/// Ressource désignée par un profil et absente de l'hôte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MissingResource {
    /// Banque de percussion non chargée : Heel/Toe silencieux.
    PercussionBank { bank: u16 },
    /// Preset Odin non chargé : la piste joue son programme SoundFont.
    OdinPreset { track: TrackId, preset: String },
}

impl fmt::Display for MissingResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PercussionBank { bank } => write!(
                f,
                "banque de percussion {bank} non chargée par l'hôte : les pieds (Heel/Toe) \
                 seront silencieux, aucun repli implicite (fournir la banque de pieds ou un kit \
                 GM en banque 128)"
            ),
            Self::OdinPreset { track, preset } => write!(
                f,
                "preset Odin « {preset} » de la piste {} non chargé par l'hôte (preset non \
                 fourni, illisible, ou build sans l'option `odin`) : la piste joue son \
                 programme SoundFont déclaré",
                track.name()
            ),
        }
    }
}

impl StyleProfile {
    /// Ressources que ce profil désigne et que l'hôte n'a pas chargées.
    #[must_use]
    pub fn missing_resources(&self, resources: &HostResources) -> Vec<MissingResource> {
        let mut missing = Vec::new();
        let bank = self.instruments.percussion_bank;
        if !resources.percussion_banks.contains(&bank) {
            missing.push(MissingResource::PercussionBank { bank });
        }
        for (track, spec) in &self.odin {
            for zone in &spec.zones {
                if !resources.odin.contains(&zone.preset) && !missing.iter().any(|resource| {
                    matches!(resource, MissingResource::OdinPreset { track: missing_track, preset }
                            if missing_track == track && preset == &zone.preset)
                }) {
                    missing.push(MissingResource::OdinPreset {
                        track: *track,
                        preset: zone.preset.clone(),
                    });
                }
            }
        }
        missing
    }
}

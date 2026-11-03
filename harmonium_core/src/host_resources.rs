//! Ressources chargées par l'hôte, et ce qu'un profil en exige.
//!
//! L'hôte fournit des octets (banques SF2, presets Odin) ; un profil les
//! désigne en données (`instruments.percussion_bank`, `[[odin.<Piste>.zones]]`).
//! Quand le profil désigne une ressource que l'hôte n'a pas chargée, le rendu
//! n'a aucun repli implicite : percussion silencieux, ou piste jouée par le
//! programme SoundFont que le profil déclare pour elle. Le chargement du
//! profil le dit clairement au lieu de laisser un silence sans message.

use std::fmt;

use crate::{odin::OdinBank, style_profile::StyleProfile, timeline::TrackId};

/// Ressources effectivement chargées par l'hôte audio.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HostResources {
    /// Banques SF2 portant un kit de percussion jouable : `128` (kit GM de la
    /// banque principale) et/ou la banque de percussion chargée à part.
    pub percussion_banks: Vec<u16>,
    /// Presets Odin analysés (vide sans l'option `odin` ou sans preset fourni).
    pub odin: OdinBank,
}

/// Ressource désignée par un profil et absente de l'hôte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MissingResource {
    /// Banque de percussion non chargée : PercussionLow/PercussionHigh silencieux.
    PercussionBank { bank: u16 },
    /// Preset Odin non chargé : la piste joue son programme SoundFont.
    OdinPreset { track: TrackId, preset: String },
}

impl fmt::Display for MissingResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PercussionBank { bank } => write!(
                f,
                "banque de percussion {bank} non chargée par l'hôte : la percussion (PercussionLow/PercussionHigh) \
                 seront silencieux, aucun repli implicite (fournir la banque de percussion ou un kit \
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
    /// Ressources que ce profil désigne et que l'hôte n'a pas chargées : celles de `[instruments]`
    /// et de `[odin]`, et celles que ses variantes de session ou ses chorus peuvent mettre à leur
    /// place (kit, presets Odin).
    #[must_use]
    pub fn missing_resources(&self, resources: &HostResources) -> Vec<MissingResource> {
        let mut missing = Vec::new();
        let arc_variants =
            self.arc.iter().chain(self.session.arcs.values()).flat_map(|arc| {
                arc.stages.iter().flat_map(|stage| &stage.variants).chain(&arc.fall)
            });
        let banks = std::iter::once(self.instruments.percussion_bank)
            .chain(
                self.session
                    .variants
                    .iter()
                    .filter_map(|variant| variant.instruments.and_then(|i| i.percussion_bank)),
            )
            .chain(arc_variants.filter_map(|v| v.instruments.and_then(|i| i.percussion_bank)));
        for bank in banks {
            if !resources.percussion_banks.contains(&bank)
                && !missing.contains(&MissingResource::PercussionBank { bank })
            {
                missing.push(MissingResource::PercussionBank { bank });
            }
        }
        let odin_specs = self.odin.iter().chain(self.session.variants.iter().flat_map(|v| &v.odin));
        for (track, spec) in odin_specs {
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

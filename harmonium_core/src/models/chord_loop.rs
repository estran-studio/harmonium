//! Boucles et pédales explicites par partie, résolues dans le mode actif.
use crate::{
    style_profile::{HarmonyPart, ProfileError, StyleProfile},
    trad::{ChordDegree, ChordQuality, Mode, TradChord, TunePlan},
};

#[derive(Clone, Debug, PartialEq)]
pub struct LoopHarmony {
    plan: TunePlan,
    chords: Vec<Vec<TradChord>>,
}
impl LoopHarmony {
    /// Le profil du plan doit avoir été validé avant la construction.
    pub fn new(plan: TunePlan) -> Self {
        let slots = plan.profile.meter.steps_per_bar / plan.profile.harmony.steps_per_chord;
        let chords = (0..plan.material_bars())
            .map(|material| {
                let (part, _, offset, _) = plan.position(plan.representative_bar(material));
                let name = &plan.profile.form.parts[part].name;
                let chord = match &plan.profile.harmony.parts[name] {
                    HarmonyPart::Pedal { degree, semitones, neighbor_semitones } => TradChord::new(
                        root(*degree, *semitones, plan.mode),
                        neighbor_semitones.map_or(ChordQuality::Pedal, ChordQuality::PedalNeighbor),
                    ),
                    HarmonyPart::ChordLoop { chords } => {
                        let default = plan.profile.harmony.default_bars_per_chord;
                        let length: usize =
                            chords.iter().map(|chord| chord.bars.unwrap_or(default)).sum();
                        let mut remaining = offset % length;
                        let chord = chords
                            .iter()
                            .find(|chord| {
                                let bars = chord.bars.unwrap_or(default);
                                if remaining < bars {
                                    true
                                } else {
                                    remaining -= bars;
                                    false
                                }
                            })
                            .unwrap_or_else(|| unreachable!("validated nonempty chord loop"));
                        TradChord::new(
                            root(chord.degree, chord.semitones, plan.mode),
                            chord.quality,
                        )
                    }
                };
                vec![chord; slots]
            })
            .collect();
        Self { plan, chords }
    }
    pub fn chords_for_bar(&self, bar: usize) -> &[TradChord] {
        &self.chords[self.plan.material_index(bar)]
    }
}
fn root(degree: Option<u8>, semitones: Option<u8>, mode: Mode) -> ChordDegree {
    ChordDegree::Semitones(degree.map_or_else(
        || semitones.unwrap_or_else(|| unreachable!("validated explicit root")),
        |degree| mode.intervals()[usize::from(degree - 1)] as u8,
    ))
}
fn validate_root(
    degree: Option<u8>,
    semitones: Option<u8>,
    path: &str,
) -> Result<(), ProfileError> {
    if degree.is_some() == semitones.is_some() {
        return Err(ProfileError::new(
            format!("{path}.degree"),
            "exactement une racine degree ou semitones requise",
        ));
    }
    if degree.is_some_and(|degree| !(1..=7).contains(&degree)) {
        return Err(ProfileError::new(format!("{path}.degree"), "degré modal attendu dans 1..7"));
    }
    if semitones.is_some_and(|semitones| semitones > 11) {
        return Err(ProfileError::new(format!("{path}.semitones"), "attendu 0..11"));
    }
    Ok(())
}
/// Validation appelée par le chargeur public de profils, avant toute génération.
pub(crate) fn validate(profile: &StyleProfile) -> Result<(), ProfileError> {
    let harmony = &profile.harmony;
    if harmony.model != "loop" {
        return if harmony.parts.is_empty() {
            Ok(())
        } else {
            Err(ProfileError::new("harmony.parts", "réservé au modèle loop"))
        };
    }
    if harmony.default_bars_per_chord == 0 {
        return Err(ProfileError::new(
            "harmony.default_bars_per_chord",
            "durée positive en mesures entières requise",
        ));
    }
    for name in harmony.parts.keys() {
        if !profile.form.parts.iter().any(|part| part.name == *name) {
            return Err(ProfileError::new(
                format!("harmony.parts.{name}"),
                "partie inconnue dans form.parts",
            ));
        }
    }
    for part in &profile.form.parts {
        let path = format!("harmony.parts.{}", part.name);
        let configured = harmony
            .parts
            .get(&part.name)
            .ok_or_else(|| ProfileError::new(&path, "harmonie requise pour chaque partie"))?;
        match configured {
            HarmonyPart::Pedal { degree, semitones, neighbor_semitones } => {
                validate_root(*degree, *semitones, &path)?;
                if neighbor_semitones.is_some_and(|neighbor| !(1..=11).contains(&neighbor)) {
                    return Err(ProfileError::new(
                        format!("{path}.neighbor_semitones"),
                        "attendu 1..11",
                    ));
                }
                if let Some(neighbor) = neighbor_semitones
                    && let Some(accompaniment) = &profile.accompaniment
                    && accompaniment.chords.voicing
                        == crate::trad::accomp_profile::ChordVoicing::FixedTriad
                    && accompaniment.chords.high.saturating_sub(accompaniment.chords.low)
                        < 11 + neighbor
                {
                    return Err(ProfileError::new(
                        "accompaniment.chords.high",
                        "la fenêtre fixe doit couvrir douze fondamentales et le voisin supérieur",
                    ));
                }
            }
            HarmonyPart::ChordLoop { chords } => {
                if chords.is_empty() {
                    return Err(ProfileError::new(
                        format!("{path}.chords"),
                        "boucle non vide requise",
                    ));
                }
                let mut length = 0usize;
                for (index, chord) in chords.iter().enumerate() {
                    let path = format!("{path}.chords[{index}]");
                    validate_root(chord.degree, chord.semitones, &path)?;
                    if !matches!(chord.quality, ChordQuality::Major | ChordQuality::Minor) {
                        return Err(ProfileError::new(
                            format!("{path}.quality"),
                            "Major ou Minor requis",
                        ));
                    }
                    let bars = chord.bars.unwrap_or(harmony.default_bars_per_chord);
                    if bars == 0 {
                        return Err(ProfileError::new(
                            format!("{path}.bars"),
                            "durée positive en mesures entières requise",
                        ));
                    }
                    length = length.checked_add(bars).ok_or_else(|| {
                        ProfileError::new(format!("{path}.bars"), "durée totale en débordement")
                    })?;
                }
                if !part.roles.len().is_multiple_of(length) {
                    return Err(ProfileError::new(
                        format!("{path}.chords"),
                        "la longueur de boucle doit diviser la longueur de la partie",
                    ));
                }
            }
        }
    }
    Ok(())
}

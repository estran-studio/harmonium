//! Boucles et pédales explicites par partie, résolues dans le mode actif.
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{
    algo::RngCore,
    compose::{ChordDegree, ChordQuality, DegreeChord, Mode, TunePlan},
    style_profile::{HarmonyPart, LoopChord, ProfileError, StyleProfile},
};

#[derive(Clone, Debug, PartialEq)]
pub struct LoopHarmony {
    plan: TunePlan,
    chords: Vec<Vec<DegreeChord>>,
}
impl LoopHarmony {
    /// Le profil du plan doit avoir été validé avant la construction.
    pub fn new(plan: TunePlan, seed: u64) -> Self {
        let steps_per_bar = plan.profile.meter.steps_per_bar;
        let period = plan.profile.harmony.steps_per_chord;
        let slots = steps_per_bar / period;
        let selected: Vec<_> = plan
            .profile
            .form
            .parts
            .iter()
            .enumerate()
            .map(|(part, configured)| match &plan.profile.harmony.parts[&configured.name] {
                HarmonyPart::Pedal { .. } => 0,
                HarmonyPart::ChordLoop { loops } => {
                    if loops.len() == 1 {
                        return 0;
                    }
                    let mut rng = ChaCha8Rng::seed_from_u64(crate::ornaments::mix(
                        crate::ornaments::mix(seed, 0x4841_524D),
                        part as u64,
                    ));
                    let total: u64 =
                        loops.iter().map(|alternative| u64::from(alternative.weight)).sum();
                    let mut pick = rng.next_range_u64(0..total);
                    loops
                        .iter()
                        .position(|alternative| {
                            let weight = u64::from(alternative.weight);
                            if pick < weight {
                                true
                            } else {
                                pick -= weight;
                                false
                            }
                        })
                        .unwrap_or_else(|| unreachable!("validated positive loop weights"))
                }
            })
            .collect();
        let chords = (0..plan.material_bars())
            .map(|material| {
                let (part, _, offset, _) = plan.position(plan.representative_bar(material));
                let name = &plan.profile.form.parts[part].name;
                match &plan.profile.harmony.parts[name] {
                    HarmonyPart::Pedal { degree, semitones, neighbor_semitones } => {
                        let chord = DegreeChord::new(
                            root(*degree, *semitones, plan.mode),
                            neighbor_semitones
                                .map_or(ChordQuality::Pedal, ChordQuality::PedalNeighbor),
                        );
                        vec![chord; slots]
                    }
                    HarmonyPart::ChordLoop { loops } => {
                        let chords = &loops[selected[part]].chords;
                        (0..slots)
                            .map(|slot| {
                                let mut remaining = offset * steps_per_bar + slot * period;
                                let chord = chords
                                    .iter()
                                    .find(|chord| {
                                        let steps = chord_steps(chord, period);
                                        if remaining < steps {
                                            true
                                        } else {
                                            remaining -= steps;
                                            false
                                        }
                                    })
                                    .unwrap_or_else(|| {
                                        unreachable!("validated nonempty chord loop")
                                    });
                                degree_chord(chord, plan.mode)
                            })
                            .collect()
                    }
                }
            })
            .collect();
        Self { plan, chords }
    }
    pub fn chords_for_bar(&self, bar: usize) -> &[DegreeChord] {
        &self.chords[self.plan.material_index(bar)]
    }
}
/// Durée en pas : explicite, sinon un créneau de la grille harmonique.
fn chord_steps(chord: &LoopChord, steps_per_chord: usize) -> usize {
    chord.steps.unwrap_or(steps_per_chord)
}
fn degree_chord(chord: &LoopChord, mode: Mode) -> DegreeChord {
    let resolved = DegreeChord::new(
        root(chord.degree, chord.semitones, mode),
        ChordQuality::from_quality(chord.quality),
    );
    chord.bass.map_or(resolved, |bass| resolved.with_bass(bass))
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
    for name in harmony.parts.keys() {
        if !profile.form.parts.iter().any(|part| part.name == *name) {
            return Err(ProfileError::new(
                format!("harmony.parts.{name}"),
                "partie inconnue dans form.parts",
            ));
        }
    }
    let fixed_triad = profile.accompaniment.as_ref().is_some_and(|accompaniment| {
        accompaniment.chords.voicing == crate::compose::accomp_profile::ChordVoicing::FixedTriad
    });
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
                        == crate::compose::accomp_profile::ChordVoicing::FixedTriad
                    && accompaniment.chords.high.saturating_sub(accompaniment.chords.low)
                        < 11 + neighbor
                {
                    return Err(ProfileError::new(
                        "accompaniment.chords.high",
                        "la fenêtre fixe doit couvrir douze fondamentales et le voisin supérieur",
                    ));
                }
            }
            HarmonyPart::ChordLoop { loops } => {
                if loops.is_empty() {
                    return Err(ProfileError::new(
                        format!("{path}.loops"),
                        "au moins une alternative harmonique requise",
                    ));
                }
                for (loop_index, alternative) in loops.iter().enumerate() {
                    let path = format!("{path}.loops[{loop_index}]");
                    if alternative.weight == 0 {
                        return Err(ProfileError::new(
                            format!("{path}.weight"),
                            "poids strictement positif requis",
                        ));
                    }
                    if alternative.chords.is_empty() {
                        return Err(ProfileError::new(
                            format!("{path}.chords"),
                            "boucle non vide requise",
                        ));
                    }
                    let mut length = 0usize;
                    for (index, chord) in alternative.chords.iter().enumerate() {
                        let path = format!("{path}.chords[{index}]");
                        validate_root(chord.degree, chord.semitones, &path)?;
                        if chord.bass.is_some_and(|bass| bass > 11) {
                            return Err(ProfileError::new(
                                format!("{path}.bass"),
                                "attendu 0..11 demi-tons au-dessus de la tonique",
                            ));
                        }
                        if fixed_triad
                            && (chord.bass.is_some()
                                || !matches!(
                                    ChordQuality::from_quality(chord.quality),
                                    ChordQuality::Major | ChordQuality::Minor
                                ))
                        {
                            return Err(ProfileError::new(
                                "accompaniment.chords.voicing",
                                "fixed_triad ne joue que des triades majeures et mineures \
                                 sans basse : qualités typées et accords slash demandent close_triad",
                            ));
                        }
                        let steps = chord_steps(chord, harmony.steps_per_chord);
                        if steps == 0 || !steps.is_multiple_of(harmony.steps_per_chord) {
                            return Err(ProfileError::new(
                                format!("{path}.steps"),
                                "durée positive en pas, multiple de harmony.steps_per_chord requise",
                            ));
                        }
                        length = length.checked_add(steps).ok_or_else(|| {
                            ProfileError::new(
                                format!("{path}.steps"),
                                "durée totale en débordement",
                            )
                        })?;
                    }
                    if length != part.roles.len() * profile.meter.steps_per_bar {
                        return Err(ProfileError::new(
                            format!("{path}.chords"),
                            "la somme des durées en pas de chaque alternative doit égaler la \
                             longueur de la partie en pas (mesures × meter.steps_per_bar)",
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

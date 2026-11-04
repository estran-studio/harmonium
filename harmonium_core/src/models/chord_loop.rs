//! Boucles et pédales explicites par partie, résolues dans le mode actif ; boucles générées par
//! contraintes (voir `loop_generation`).
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::loop_generation::{self, Unsatisfiable};
use crate::{
    algo::RngCore,
    compose::{ChordDegree, ChordQuality, DegreeChord, Mode, TunePlan},
    style_profile::{
        GeneratedLoop, HarmonyPart, KeyFit, LoopChord, ProfileError, RootMotion, StyleProfile,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct LoopHarmony {
    plan: TunePlan,
    /// Une table de mesures par tirage harmonique (`harmony.tour_variants`).
    chords: Vec<Vec<Vec<DegreeChord>>>,
}
impl LoopHarmony {
    /// Le profil du plan doit avoir été validé avant la construction.
    pub fn new(plan: TunePlan, seed: u64) -> Self {
        let variants = usize::from(plan.profile.harmony.tour_variants.max(1));
        let chords = (0..variants)
            .map(|variant| {
                // Le premier tirage garde la graine de l'air : un profil à un seul tirage ne change pas.
                let seed =
                    if variant == 0 { seed } else { crate::ornaments::mix(seed, variant as u64) };
                Self::draw(&plan, seed)
            })
            .collect();
        Self { plan, chords }
    }
    /// Mesures de matériau d'un tirage harmonique.
    fn draw(plan: &TunePlan, seed: u64) -> Vec<Vec<DegreeChord>> {
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
                HarmonyPart::Pedal { weight, alternatives, .. } => draw_alternative(
                    seed,
                    part,
                    std::iter::once(*weight).chain(alternatives.iter().map(|a| a.weight)),
                ),
                HarmonyPart::ChordLoop { loops, generated } => {
                    let modes: Vec<&[Mode]> = loops
                        .iter()
                        .map(|alternative| alternative.modes.as_slice())
                        .chain(generated.iter().map(|alternative| alternative.modes.as_slice()))
                        .collect();
                    // Alternatives du mode de l'air ; à défaut, toutes.
                    let fits = |modes: &[Mode]| modes.is_empty() || modes.contains(&plan.mode);
                    let any_fits = modes.iter().any(|m| fits(m));
                    draw_alternative(
                        seed,
                        part,
                        loops
                            .iter()
                            .map(|alternative| alternative.weight)
                            .chain(generated.iter().map(|alternative| alternative.weight))
                            .zip(modes)
                            .map(
                                |(weight, modes)| if !any_fits || fits(modes) { weight } else { 0 },
                            ),
                    )
                }
            })
            .collect();
        // Boucles générées : tirées une fois par partie (graine, partie, alternative), puis
        // répétées jusqu'à la fin de la partie ; la forme reste lisible d'une reprise à l'autre.
        let generated: Vec<Option<Vec<LoopChord>>> = plan
            .profile
            .form
            .parts
            .iter()
            .enumerate()
            .map(|(part, configured)| {
                let HarmonyPart::ChordLoop { loops, generated } =
                    &plan.profile.harmony.parts[&configured.name]
                else {
                    return None;
                };
                let index = selected[part].checked_sub(loops.len())?;
                let spec = &generated[index];
                let part_bars = configured.roles.len();
                let bars = spec.bars.unwrap_or(part_bars);
                let mut rng = ChaCha8Rng::seed_from_u64(crate::ornaments::mix(
                    crate::ornaments::mix(crate::ornaments::mix(seed, 0x4745_4E4C), part as u64),
                    index as u64,
                ));
                let one = loop_generation::generate(
                    spec,
                    bars * steps_per_bar / period,
                    period,
                    plan.mode,
                    &mut rng,
                );
                Some(one.iter().cycle().take(one.len() * (part_bars / bars)).cloned().collect())
            })
            .collect();
        (0..plan.material_bars())
            .map(|material| {
                let (part, _, offset, _) = plan.position(plan.representative_bar(material));
                let name = &plan.profile.form.parts[part].name;
                match &plan.profile.harmony.parts[name] {
                    HarmonyPart::Pedal {
                        degree,
                        semitones,
                        neighbor_semitones,
                        alternatives,
                        ..
                    } => {
                        let (degree, semitones, neighbor) = match selected[part].checked_sub(1) {
                            None => (*degree, *semitones, *neighbor_semitones),
                            Some(index) => {
                                let alternative = &alternatives[index];
                                (
                                    alternative.degree,
                                    alternative.semitones,
                                    alternative.neighbor_semitones,
                                )
                            }
                        };
                        let chord = DegreeChord::new(
                            root(degree, semitones, plan.mode),
                            neighbor.map_or(ChordQuality::Pedal, ChordQuality::PedalNeighbor),
                        );
                        vec![chord; slots]
                    }
                    HarmonyPart::ChordLoop { loops, .. } => {
                        let chords = generated[part]
                            .as_deref()
                            .unwrap_or_else(|| loops[selected[part]].chords.as_slice());
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
            .collect()
    }
    /// Accords de la mesure `bar` de l'air : le tirage du chorus qui la contient.
    pub fn chords_for_bar(&self, bar: usize) -> &[DegreeChord] {
        let variant = (bar / self.plan.total_bars()) % self.chords.len();
        &self.chords[variant][self.plan.material_index(bar)]
    }
}
/// Alternative tirée parmi `weights` avec le flux de la partie ; un candidat unique n'entraîne
/// aucun tirage.
fn draw_alternative(seed: u64, part: usize, weights: impl Iterator<Item = u32> + Clone) -> usize {
    if weights.clone().nth(1).is_none() {
        return 0;
    }
    let mut rng = ChaCha8Rng::seed_from_u64(crate::ornaments::mix(
        crate::ornaments::mix(seed, 0x4841_524D),
        part as u64,
    ));
    let total: u64 = weights.clone().map(u64::from).sum();
    let mut pick = rng.next_range_u64(0..total);
    weights
        .map(u64::from)
        .position(|weight| {
            if pick < weight {
                true
            } else {
                pick -= weight;
                false
            }
        })
        .unwrap_or_else(|| unreachable!("validated positive alternative weights"))
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
    if !(1..=16).contains(&harmony.tour_variants)
        || (harmony.model != "loop" && harmony.tour_variants != 1)
    {
        return Err(ProfileError::new(
            "harmony.tour_variants",
            "1..16 tirages requis (le modèle loop seul en accepte plus d'un)",
        ));
    }
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
            HarmonyPart::Pedal { degree, semitones, neighbor_semitones, weight, alternatives } => {
                validate_pedal(profile, *degree, *semitones, *neighbor_semitones, &path)?;
                if alternatives.is_empty() {
                    continue;
                }
                if *weight == 0 {
                    return Err(ProfileError::new(
                        format!("{path}.weight"),
                        "poids strictement positif requis",
                    ));
                }
                for (index, alternative) in alternatives.iter().enumerate() {
                    let path = format!("{path}.alternatives[{index}]");
                    if alternative.weight == 0 {
                        return Err(ProfileError::new(
                            format!("{path}.weight"),
                            "poids strictement positif requis",
                        ));
                    }
                    validate_pedal(
                        profile,
                        alternative.degree,
                        alternative.semitones,
                        alternative.neighbor_semitones,
                        &path,
                    )?;
                }
            }
            HarmonyPart::ChordLoop { loops, generated } => {
                if loops.is_empty() && generated.is_empty() {
                    return Err(ProfileError::new(
                        format!("{path}.loops"),
                        "au moins une alternative harmonique requise (loops ou generated)",
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
                            return Err(fixed_triad_error());
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
                for (index, spec) in generated.iter().enumerate() {
                    validate_generated(
                        profile,
                        spec,
                        part.roles.len(),
                        fixed_triad,
                        &format!("{path}.generated[{index}]"),
                    )?;
                }
            }
        }
    }
    Ok(())
}
fn fixed_triad_error() -> ProfileError {
    ProfileError::new(
        "accompaniment.chords.voicing",
        "fixed_triad ne joue que des triades majeures et mineures sans basse : qualités typées \
         et accords slash demandent close_triad",
    )
}
/// Une pédale (écrite ou alternative) : racine, voisin et fenêtre d'une triade fixe.
fn validate_pedal(
    profile: &StyleProfile,
    degree: Option<u8>,
    semitones: Option<u8>,
    neighbor_semitones: Option<u8>,
    path: &str,
) -> Result<(), ProfileError> {
    validate_root(degree, semitones, path)?;
    if neighbor_semitones.is_some_and(|neighbor| !(1..=11).contains(&neighbor)) {
        return Err(ProfileError::new(format!("{path}.neighbor_semitones"), "attendu 1..11"));
    }
    if let Some(neighbor) = neighbor_semitones
        && let Some(accompaniment) = &profile.accompaniment
        && accompaniment.chords.voicing == crate::compose::accomp_profile::ChordVoicing::FixedTriad
        && accompaniment.chords.high.saturating_sub(accompaniment.chords.low) < 11 + neighbor
    {
        return Err(ProfileError::new(
            "accompaniment.chords.high",
            "la fenêtre fixe doit couvrir douze fondamentales et le voisin supérieur",
        ));
    }
    Ok(())
}
/// Une boucle générée : bornes des champs, puis faisabilité dans chaque mode atteignable.
fn validate_generated(
    profile: &StyleProfile,
    spec: &GeneratedLoop,
    part_bars: usize,
    fixed_triad: bool,
    path: &str,
) -> Result<(), ProfileError> {
    let harmony = &profile.harmony;
    let field = |name: &str| format!("{path}.{name}");
    if spec.weight == 0 {
        return Err(ProfileError::new(field("weight"), "poids strictement positif requis"));
    }
    let bars = spec.bars.unwrap_or(part_bars);
    if bars == 0 || !part_bars.is_multiple_of(bars) {
        return Err(ProfileError::new(
            field("bars"),
            format!("diviseur de la longueur de la partie ({part_bars} mesures) requis"),
        ));
    }
    let slots = bars * profile.meter.steps_per_bar / harmony.steps_per_chord;
    if spec.count == 0 || spec.count > slots {
        return Err(ProfileError::new(
            field("count"),
            format!(
                "1..{slots} accords requis : un créneau de harmony.steps_per_chord par accord \
                 dans {bars} mesure(s)"
            ),
        ));
    }
    if spec.qualities.is_empty() || spec.qualities.len() > 64 {
        return Err(ProfileError::new(field("qualities"), "1..64 qualités requises"));
    }
    if spec.start.is_some_and(|start| start > 11) {
        return Err(ProfileError::new(field("start"), "attendu 0..11"));
    }
    if spec.pedal.is_some_and(|pedal| pedal > 11) {
        return Err(ProfileError::new(field("pedal"), "attendu 0..11"));
    }
    if let RootMotion::Intervals { intervals } = &spec.motion {
        let range = -11..=11;
        if intervals.is_empty()
            || intervals.iter().any(|i| i.weight == 0 || !range.contains(&i.semitones))
        {
            return Err(ProfileError::new(
                field("motion.intervals"),
                "au moins un intervalle, demi-tons dans -11..11, poids strictement positifs",
            ));
        }
    }
    if fixed_triad
        && (spec.pedal.is_some()
            || spec.qualities.iter().any(|&quality| {
                !matches!(
                    ChordQuality::from_quality(quality),
                    ChordQuality::Major | ChordQuality::Minor
                )
            }))
    {
        return Err(fixed_triad_error());
    }
    let depends_on_mode =
        spec.key_fit != KeyFit::Free || matches!(spec.motion, RootMotion::ScaleSteps { .. });
    let reachable = profile.reachable_modes();
    let modes: Vec<Mode> = if depends_on_mode {
        Mode::ALL.into_iter().filter(|mode| reachable.contains(mode)).collect()
    } else {
        vec![Mode::Ionian]
    };
    for mode in modes {
        match loop_generation::check(spec, mode) {
            Ok(()) => {}
            Err(Unsatisfiable::Start) => {
                return Err(ProfileError::new(
                    field("key_fit"),
                    format!(
                        "aucune qualité de qualities ne respecte key_fit sur la fondamentale de \
                         départ (start) en mode {mode:?}"
                    ),
                ));
            }
            Err(Unsatisfiable::Motion) => {
                return Err(ProfileError::new(
                    field("motion"),
                    format!(
                        "aucune suite de {} accords ne respecte motion, start et key_fit en mode \
                         {mode:?}",
                        spec.count
                    ),
                ));
            }
        }
    }
    Ok(())
}

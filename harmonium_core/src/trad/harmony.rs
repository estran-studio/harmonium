//! Harmonie fonctionnelle après sélection mélodique : vote pondéré des notes
//! parmi les candidats du profil, à son rythme harmonique. Les cadences
//! imposent la dominante ou la tonique, indépendamment de la métrique.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    mode::Mode,
    plan::{BarRole, ImposedRole, TunePlan},
};
use crate::{
    style_profile::StyleProfile, theory::PitchClass, timeline::TimelineNote,
    tuning::TradHarmonyParams,
};

/// Degré de la fondamentale d'un accord, relatif à la tonique du mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChordDegree {
    /// Tonique.
    I,
    /// Seconde.
    II,
    /// Quarte.
    IV,
    /// Quinte.
    V,
    /// Sixte du mode (9 demi-tons en ionien, 8 en éolien).
    VI,
    /// Septième mineure (10 demi-tons).
    BVII,
}

/// Qualité d'une triade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChordQuality {
    /// Tierce majeure (4 demi-tons).
    Major,
    /// Tierce mineure (3 demi-tons).
    Minor,
}

/// Triade du répertoire trad : degré du mode et qualité.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradChord {
    pub degree: ChordDegree,
    pub quality: ChordQuality,
}

impl TradChord {
    #[must_use]
    pub const fn new(degree: ChordDegree, quality: ChordQuality) -> Self {
        Self { degree, quality }
    }

    /// Demi-tons de la fondamentale au-dessus de la tonique.
    #[must_use]
    pub const fn root_semitones(self, mode: Mode) -> u8 {
        match self.degree {
            ChordDegree::I => 0,
            ChordDegree::II => 2,
            ChordDegree::IV => 5,
            ChordDegree::V => 7,
            ChordDegree::VI => mode.intervals()[5] as u8,
            ChordDegree::BVII => 10,
        }
    }

    /// Classe de hauteur de la fondamentale.
    #[must_use]
    pub const fn root_pc(self, key: PitchClass, mode: Mode) -> PitchClass {
        (key % 12 + self.root_semitones(mode)) % 12
    }

    /// Demi-tons de la tierce au-dessus de la fondamentale (4 ou 3).
    #[must_use]
    pub const fn third_semitones(self) -> u8 {
        match self.quality {
            ChordQuality::Major => 4,
            ChordQuality::Minor => 3,
        }
    }

    /// Classes de hauteur `[fondamentale, tierce, quinte]`.
    #[must_use]
    pub const fn pitch_classes(self, key: PitchClass, mode: Mode) -> [PitchClass; 3] {
        let root = self.root_pc(key, mode);
        [root, (root + self.third_semitones()) % 12, (root + 7) % 12]
    }

    /// `true` si la classe de hauteur de `midi` appartient à l'accord.
    #[must_use]
    pub fn contains_pitch(self, key: PitchClass, mode: Mode, midi: u8) -> bool {
        let pc = midi % 12;
        self.pitch_classes(key, mode).contains(&pc)
    }
}

#[cfg(test)]
const fn major(degree: ChordDegree) -> TradChord {
    TradChord::new(degree, ChordQuality::Major)
}

#[cfg(test)]
const fn minor(degree: ChordDegree) -> TradChord {
    TradChord::new(degree, ChordQuality::Minor)
}

/// Accords candidats décrits par le profil actif, dans l'ordre du corpus.
#[must_use]
pub fn candidate_chords(mode: Mode, profile: &StyleProfile) -> &[TradChord] {
    profile.harmony.candidates.iter().find(|c| c.mode == mode).map_or(&[], |c| c.chords.as_slice())
}

/// Dominante du mode (`V`, avec la qualité du candidat du mode).
#[must_use]
fn dominant(mode: Mode) -> TradChord {
    let quality =
        if matches!(mode, Mode::Ionian) { ChordQuality::Major } else { ChordQuality::Minor };
    TradChord::new(ChordDegree::V, quality)
}

/// Tonique du mode (`I`, avec la qualité du candidat du mode).
#[must_use]
fn tonic(mode: Mode) -> TradChord {
    let quality = if mode.is_minor() { ChordQuality::Minor } else { ChordQuality::Major };
    TradChord::new(ChordDegree::I, quality)
}

fn vote_weight(step: usize, params: &TradHarmonyParams) -> u32 {
    if params.strong_steps.contains(&step) { params.strong_weight } else { params.weak_weight }
}

fn tie_rank(chord_index: usize, chord: TradChord, params: &TradHarmonyParams) -> usize {
    params
        .tie_priority
        .iter()
        .position(|&d| d == chord.degree)
        .unwrap_or(params.tie_priority.len() + chord_index)
}

/// Accord de la demi-mesure `half` (0 ou 1) par vote des notes.
fn vote_half(
    melody: &[TimelineNote],
    key: PitchClass,
    mode: Mode,
    half: usize,
    params: &TradHarmonyParams,
    profile: &StyleProfile,
) -> TradChord {
    let width = profile.harmony.steps_per_chord;
    let steps = half * width..(half + 1) * width;
    let candidates = candidate_chords(mode, profile);
    let mut best = (tonic(mode), 0u32, usize::MAX);
    for (index, &chord) in candidates.iter().enumerate() {
        let score: u32 = melody
            .iter()
            .filter(|n| steps.contains(&n.start_step) && chord.contains_pitch(key, mode, n.pitch))
            .map(|n| vote_weight(n.start_step, params))
            .sum();
        let rank = tie_rank(index, chord, params);
        if score > best.1 || (score == best.1 && rank < best.2) {
            best = (chord, score, rank);
        }
    }
    best.0
}

/// Harmonise une mesure : un accord par demi-mesure (pas `0..8` puis `8..16`).
///
/// `CadHalf` impose `V` en seconde moitié ; `CadFull` impose `V` puis `I` ;
/// les autres demi-mesures sont votées d'après la mélodie.
#[must_use]
pub fn harmonize_bar(
    melody: &[TimelineNote],
    key: PitchClass,
    mode: Mode,
    bar_role: BarRole,
    params: &TradHarmonyParams,
    profile: &StyleProfile,
) -> Vec<TradChord> {
    let count = profile.meter.steps_per_bar / profile.harmony.steps_per_chord;
    let mut chords: Vec<_> =
        (0..count).map(|slot| vote_half(melody, key, mode, slot, params, profile)).collect();
    match bar_role.role {
        ImposedRole::CadFull | ImposedRole::CadTonic => {
            chords[0] = dominant(mode);
            chords[count - 1] = tonic(mode);
        }
        ImposedRole::CadHalf | ImposedRole::CadFifth => chords[count - 1] = dominant(mode),
        _ => {}
    }
    chords
}

/// Erreur de construction d'un [`TuneHarmony`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuneHarmonyError {
    /// Le nombre de mesures diffère du nombre de mesures distinctes du profil.
    WrongBarCount { expected: usize, found: usize },
}

impl fmt::Display for TuneHarmonyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongBarCount { expected, found } => {
                write!(f, "{expected} mesures mélodiques attendues, {found} reçues")
            }
        }
    }
}

impl std::error::Error for TuneHarmonyError {}

/// Harmonie des cellules distinctes, y compris les fins de reprise configurées.
#[derive(Clone, Debug, PartialEq)]
pub struct TuneHarmony {
    plan: TunePlan,
    bars: Vec<Vec<TradChord>>,
}

impl TuneHarmony {
    /// Harmonise le matériel de chaque partie, puis ses éventuelles secondes fins.
    pub fn new<M: AsRef<[TimelineNote]>>(
        plan: TunePlan,
        melody: &[M],
        params: &TradHarmonyParams,
    ) -> Result<Self, TuneHarmonyError> {
        let expected = plan.material_bars();
        if melody.len() != expected {
            return Err(TuneHarmonyError::WrongBarCount { expected, found: melody.len() });
        }
        let mut bars = Vec::with_capacity(expected);
        for (i, notes) in melody.iter().enumerate() {
            let role = plan.bar_role(plan.representative_bar(i));
            bars.push(harmonize_bar(
                notes.as_ref(),
                plan.key,
                plan.mode,
                role,
                params,
                &plan.profile,
            ));
        }
        Ok(Self { plan, bars })
    }

    /// Plan du tune harmonisé.
    #[must_use]
    pub fn plan(&self) -> TunePlan {
        self.plan.clone()
    }

    /// Accords de la mesure `bar_in_tune` (0-based, replié sur le tune).
    #[must_use]
    pub fn chords_for_bar(&self, bar_in_tune: usize) -> &[TradChord] {
        &self.bars[self.plan.material_index(bar_in_tune)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::Articulation;

    const D: PitchClass = 2;
    const TONIC_MIDI: u8 = 62;

    /// Mélodie de 8 croches (pas 0, 2, …, 14) à partir de degrés du mode.
    fn melody(degrees: &str, mode: Mode) -> Vec<TimelineNote> {
        degrees
            .split_whitespace()
            .enumerate()
            .map(|(i, d)| TimelineNote {
                id: i as u64,
                pitch: mode.degree_to_midi(TONIC_MIDI, d.parse().unwrap_or(1)),
                start_step: i * 2,
                duration_steps: 2,
                velocity: 90,
                articulation: Articulation::Normal,
            })
            .collect()
    }

    fn fixed_plan(mode: Mode) -> TunePlan {
        let mut profile = StyleProfile::default();
        for part in &mut profile.form.parts {
            part.second_ending = None;
            part.roles = [
                ImposedRole::Open,
                ImposedRole::Free,
                ImposedRole::Free,
                ImposedRole::CadHalf,
                ImposedRole::CopyOf(1),
                ImposedRole::CopyOf(2),
                ImposedRole::Free,
                ImposedRole::CadFull,
            ]
            .map(Into::into)
            .to_vec();
        }
        TunePlan::with_profile(D, mode, std::sync::Arc::new(profile))
    }

    fn role(bar_in_tune: usize) -> BarRole {
        fixed_plan(Mode::Ionian).bar_role(bar_in_tune)
    }

    fn free_role() -> BarRole {
        role(1)
    }

    fn harmonize(degrees: &str, mode: Mode, role: BarRole) -> Vec<TradChord> {
        harmonize_bar(
            &melody(degrees, mode),
            D,
            mode,
            role,
            &TradHarmonyParams::default(),
            &StyleProfile::default(),
        )
    }

    #[test]
    fn ionian_melodies_select_i_iv_v() {
        let cases = [
            ("1 3 5 3 1 3 5 3", ChordDegree::I),
            ("4 6 8 6 4 6 8 6", ChordDegree::IV),
            ("5 7 2 7 5 7 2 7", ChordDegree::V),
        ];
        for (degrees, expected) in cases {
            let chords = harmonize(degrees, Mode::Ionian, free_role());
            assert_eq!(
                chords.iter().map(|c| c.degree).collect::<Vec<_>>(),
                [expected, expected],
                "{degrees}"
            );
            assert_eq!(
                chords.iter().map(|c| c.quality).collect::<Vec<_>>(),
                [ChordQuality::Major; 2],
                "{degrees}"
            );
        }
    }

    #[test]
    fn halves_are_voted_independently() {
        let chords = harmonize("1 3 5 3 4 6 8 6", Mode::Ionian, free_role());
        assert_eq!(
            chords.iter().map(|c| c.degree).collect::<Vec<_>>(),
            [ChordDegree::I, ChordDegree::IV]
        );
    }

    #[test]
    fn cadential_roles_override_the_melody() {
        // Mélodie qui appelle clairement I partout.
        let tonic_melody = "1 3 5 3 1 3 5 3";
        let half = harmonize(tonic_melody, Mode::Ionian, role(3));
        assert_eq!(
            half.iter().map(|c| c.degree).collect::<Vec<_>>(),
            [ChordDegree::I, ChordDegree::V]
        );
        let full = harmonize(tonic_melody, Mode::Ionian, role(7));
        assert_eq!(
            full.iter().map(|c| c.degree).collect::<Vec<_>>(),
            [ChordDegree::V, ChordDegree::I]
        );
        // Et une mélodie sur V ne change pas la cadence parfaite.
        let full = harmonize("5 7 2 7 5 7 2 7", Mode::Ionian, role(7));
        assert_eq!(
            full.iter().map(|c| c.degree).collect::<Vec<_>>(),
            [ChordDegree::V, ChordDegree::I]
        );
    }

    #[test]
    fn cadence_uses_modal_qualities() {
        let full = harmonize("1 3 5 3 1 3 5 3", Mode::Dorian, role(7));
        assert_eq!(full, [minor(ChordDegree::V), minor(ChordDegree::I)]);
    }

    #[test]
    fn ties_prefer_i_then_v_then_iv() {
        // Aucune note : tous les scores sont nuls.
        let chords = harmonize_bar(
            &[],
            D,
            Mode::Ionian,
            free_role(),
            &TradHarmonyParams::default(),
            &StyleProfile::default(),
        );
        assert_eq!(chords.iter().map(|c| c.degree).collect::<Vec<_>>(), [ChordDegree::I; 2]);
        // « 5 » (A) appartient à I (D F# A) et à V (A C# E) : I l'emporte par défaut…
        let chords = harmonize("5 5 5 5 5 5 5 5", Mode::Ionian, free_role());
        assert_eq!(chords.iter().map(|c| c.degree).collect::<Vec<_>>(), [ChordDegree::I; 2]);
        // … et l'ordre de priorité vient des paramètres.
        let params = TradHarmonyParams {
            tie_priority: vec![ChordDegree::V, ChordDegree::I],
            ..TradHarmonyParams::default()
        };
        let notes = melody("5 5 5 5 5 5 5 5", Mode::Ionian);
        let chords =
            harmonize_bar(&notes, D, Mode::Ionian, free_role(), &params, &StyleProfile::default());
        assert_eq!(chords.iter().map(|c| c.degree).collect::<Vec<_>>(), [ChordDegree::V; 2]);
        // « 1 » appartient à I, IV et vi : I l'emporte à l'égalité.
        let chords = harmonize("1 1 1 1 1 1 1 1", Mode::Ionian, free_role());
        assert_eq!(chords.iter().map(|c| c.degree).collect::<Vec<_>>(), [ChordDegree::I; 2]);
    }

    #[test]
    fn mixolydian_melody_on_flat_seven_yields_bvii() {
        // Degrés mixolydiens 7 (C), 2 (E), 4 (G) : triade de C majeur = bVII.
        let chords = harmonize("7 2 4 2 7 2 4 2", Mode::Mixolydian, free_role());
        assert_eq!(chords, [major(ChordDegree::BVII); 2]);
        assert_eq!(chords[0].pitch_classes(D, Mode::Mixolydian), [0, 4, 7]);
    }

    #[test]
    fn candidate_qualities_per_mode() {
        use ChordDegree::{BVII, I, IV, V, VI};
        assert_eq!(
            candidate_chords(Mode::Ionian, &StyleProfile::default()),
            [major(I), major(IV), major(V), minor(VI)]
        );
        assert_eq!(
            candidate_chords(Mode::Mixolydian, &StyleProfile::default()),
            [major(I), major(IV), minor(V), major(BVII)]
        );
        assert_eq!(
            candidate_chords(Mode::Dorian, &StyleProfile::default()),
            [minor(I), major(IV), major(BVII), minor(V)]
        );
        assert_eq!(
            candidate_chords(Mode::Aeolian, &StyleProfile::default()),
            [minor(I), minor(IV), minor(V), major(BVII), major(VI)]
        );
    }

    #[test]
    fn chord_roots_follow_mode() {
        assert_eq!(major(ChordDegree::I).pitch_classes(D, Mode::Ionian), [2, 6, 9]);
        assert_eq!(minor(ChordDegree::VI).pitch_classes(D, Mode::Ionian), [11, 2, 6]);
        assert_eq!(major(ChordDegree::VI).pitch_classes(9, Mode::Aeolian), [5, 9, 0]);
        assert_eq!(major(ChordDegree::BVII).pitch_classes(9, Mode::Aeolian), [7, 11, 2]);
    }

    /// 16 mesures distinctes : strain A sur I, strain B sur IV (hors cadences).
    fn two_strain_melody() -> Vec<Vec<TimelineNote>> {
        (0..StyleProfile::default().form.unique_bars())
            .map(|i| {
                melody(if i < 8 { "1 3 5 3 1 3 5 3" } else { "4 6 8 6 4 6 8 6" }, Mode::Ionian)
            })
            .collect()
    }

    #[test]
    fn repeats_return_identical_chords() -> Result<(), TuneHarmonyError> {
        let plan = fixed_plan(Mode::Ionian);
        let harmony =
            TuneHarmony::new(plan.clone(), &two_strain_melody(), &TradHarmonyParams::default())?;
        for bar in 0..32 {
            let section = bar / 8;
            let first_pass = bar - if section % 2 == 1 { 8 } else { 0 };
            assert_eq!(
                harmony.chords_for_bar(bar),
                harmony.chords_for_bar(first_pass),
                "bar {bar}"
            );
        }
        // B diffère de A, cadences imposées dans chaque strain.
        assert_eq!(
            harmony.chords_for_bar(0).iter().map(|c| c.degree).collect::<Vec<_>>(),
            [ChordDegree::I; 2]
        );
        assert_eq!(
            harmony.chords_for_bar(16).iter().map(|c| c.degree).collect::<Vec<_>>(),
            [ChordDegree::IV; 2]
        );
        for strain_start in [0, 8, 16, 24] {
            assert_eq!(harmony.chords_for_bar(strain_start + 3)[1].degree, ChordDegree::V);
            assert_eq!(
                harmony
                    .chords_for_bar(strain_start + 7)
                    .iter()
                    .map(|c| c.degree)
                    .collect::<Vec<_>>(),
                [ChordDegree::V, ChordDegree::I]
            );
        }
        // Repli modulo 32.
        assert_eq!(harmony.chords_for_bar(32), harmony.chords_for_bar(0));
        Ok(())
    }

    #[test]
    fn tune_harmony_rejects_wrong_bar_count() {
        let plan = fixed_plan(Mode::Ionian);
        let few = vec![melody("1 3 5 3 1 3 5 3", Mode::Ionian); 15];
        assert_eq!(
            TuneHarmony::new(plan.clone(), &few, &TradHarmonyParams::default()),
            Err(TuneHarmonyError::WrongBarCount { expected: 16, found: 15 })
        );
    }
}

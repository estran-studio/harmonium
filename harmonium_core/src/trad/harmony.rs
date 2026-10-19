//! Harmonisation du reel (D7) : un accord par demi-mesure, choisi **après** la
//! mélodie par vote des notes.
//!
//! ## Accords candidats par mode
//!
//! Chaque accord est une triade (fondamentale, tierce, quinte juste) bâtie sur
//! un degré du mode ; la qualité est celle du répertoire trad, pas celle de
//! l'harmonisation diatonique stricte (le `V` mixolydien est mineur, le `bVII`
//! est toujours majeur).
//!
//! | Mode | Candidats |
//! |---|---|
//! | Ionien | I, IV, V (majeurs), vi (mineur) |
//! | Mixolydien | I, IV, bVII (majeurs), v (mineur) |
//! | Dorien | i, v (mineurs), IV, bVII (majeurs) |
//! | Éolien | i, iv, v (mineurs), bVII, bVI (majeurs) |
//!
//! `bVI` éolien se note [`ChordDegree::VI`] avec la qualité majeure ; le `vi`
//! ionien est [`ChordDegree::VI`] mineur.
//!
//! ## Vote
//!
//! Pour chaque demi-mesure (pas `0..8` et `8..16`), le score d'un candidat est
//! la somme des poids des notes dont la classe de hauteur appartient à
//! l'accord (poids fort sur les pas de `strong_steps`, faible ailleurs).
//! Égalité : ordre de `tie_priority` (I > V > IV), puis ordre de la table des
//! candidats. Les rôles `CadHalf` (seconde moitié = V) et `CadFull` (V puis I)
//! sont imposés quelle que soit la mélodie.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    mode::{Mode, PitchClass},
    plan::{BarRole, ImposedRole, TunePlan},
};
use crate::{timeline::TimelineNote, tuning::TradHarmonyParams};

/// Pas de grille par demi-mesure en reel (2/2, 16 pas par mesure).
pub const STEPS_PER_HALF_BAR: usize = 8;

/// Mesures d'une strain harmonisée (une strain = 8 mesures).
const BARS_PER_STRAIN: usize = 8;

/// Mesures harmonisées d'un tune : strain A puis strain B.
pub const TUNE_HARMONY_BARS: usize = 2 * BARS_PER_STRAIN;

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

const fn major(degree: ChordDegree) -> TradChord {
    TradChord::new(degree, ChordQuality::Major)
}

const fn minor(degree: ChordDegree) -> TradChord {
    TradChord::new(degree, ChordQuality::Minor)
}

const IONIAN_CHORDS: [TradChord; 4] =
    [major(ChordDegree::I), major(ChordDegree::IV), major(ChordDegree::V), minor(ChordDegree::VI)];
const MIXOLYDIAN_CHORDS: [TradChord; 4] = [
    major(ChordDegree::I),
    major(ChordDegree::IV),
    minor(ChordDegree::V),
    major(ChordDegree::BVII),
];
const DORIAN_CHORDS: [TradChord; 4] = [
    minor(ChordDegree::I),
    major(ChordDegree::IV),
    major(ChordDegree::BVII),
    minor(ChordDegree::V),
];
const AEOLIAN_CHORDS: [TradChord; 5] = [
    minor(ChordDegree::I),
    minor(ChordDegree::IV),
    minor(ChordDegree::V),
    major(ChordDegree::BVII),
    major(ChordDegree::VI),
];

/// Accords candidats du mode (voir la table du module).
#[must_use]
pub const fn candidate_chords(mode: Mode) -> &'static [TradChord] {
    match mode {
        Mode::Ionian => &IONIAN_CHORDS,
        Mode::Mixolydian => &MIXOLYDIAN_CHORDS,
        Mode::Dorian => &DORIAN_CHORDS,
        Mode::Aeolian => &AEOLIAN_CHORDS,
    }
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
) -> TradChord {
    let steps = half * STEPS_PER_HALF_BAR..(half + 1) * STEPS_PER_HALF_BAR;
    let candidates = candidate_chords(mode);
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
) -> [TradChord; 2] {
    match bar_role.role {
        ImposedRole::CadFull => [dominant(mode), tonic(mode)],
        ImposedRole::CadHalf => [vote_half(melody, key, mode, 0, params), dominant(mode)],
        ImposedRole::Open | ImposedRole::Free | ImposedRole::CopyOf(_) => {
            [vote_half(melody, key, mode, 0, params), vote_half(melody, key, mode, 1, params)]
        }
    }
}

/// Erreur de construction d'un [`TuneHarmony`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuneHarmonyError {
    /// Le nombre de mesures mélodiques fourni n'est pas [`TUNE_HARMONY_BARS`].
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

/// Harmonie d'un tune : les 16 mesures distinctes (strain A puis B) sont
/// harmonisées une fois ; les reprises relisent les mêmes accords.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TuneHarmony {
    plan: TunePlan,
    bars: [[TradChord; 2]; TUNE_HARMONY_BARS],
}

impl TuneHarmony {
    /// Harmonise `melody` : 16 mesures, les 8 de la strain A puis les 8 de B.
    pub fn new<M: AsRef<[TimelineNote]>>(
        plan: TunePlan,
        melody: &[M],
        params: &TradHarmonyParams,
    ) -> Result<Self, TuneHarmonyError> {
        if melody.len() != TUNE_HARMONY_BARS {
            return Err(TuneHarmonyError::WrongBarCount {
                expected: TUNE_HARMONY_BARS,
                found: melody.len(),
            });
        }
        let mut bars = [[tonic(plan.mode); 2]; TUNE_HARMONY_BARS];
        for (i, (slot, notes)) in bars.iter_mut().zip(melody).enumerate() {
            let role = plan.bar_role(first_pass_bar(i));
            *slot = harmonize_bar(notes.as_ref(), plan.key, plan.mode, role, params);
        }
        Ok(Self { plan, bars })
    }

    /// Plan du tune harmonisé.
    #[must_use]
    pub const fn plan(&self) -> TunePlan {
        self.plan
    }

    /// Accords de la mesure `bar_in_tune` (0-based, replié sur le tune).
    #[must_use]
    pub fn chords_for_bar(&self, bar_in_tune: usize) -> [TradChord; 2] {
        let role = self.plan.bar_role(bar_in_tune);
        let bar_index = role.strain.index() * BARS_PER_STRAIN + usize::from(role.bar_in_strain) - 1;
        self.bars[bar_index]
    }
}

/// Indice dans le tune de la première passe de la mesure distincte `i` (`0..16`) :
/// A = mesures `0..8`, B = mesures `16..24` (voir [`TunePlan::bar_role`]).
const fn first_pass_bar(i: usize) -> usize {
    (i / BARS_PER_STRAIN) * 2 * BARS_PER_STRAIN + i % BARS_PER_STRAIN
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
                pitch: mode.degree_to_midi(TONIC_MIDI, d.parse().expect("degré")),
                start_step: i * 2,
                duration_steps: 2,
                velocity: 90,
                articulation: Articulation::Normal,
            })
            .collect()
    }

    fn role(bar_in_tune: usize) -> BarRole {
        TunePlan::reel(D, Mode::Ionian).bar_role(bar_in_tune)
    }

    fn free_role() -> BarRole {
        role(1)
    }

    fn harmonize(degrees: &str, mode: Mode, role: BarRole) -> [TradChord; 2] {
        harmonize_bar(&melody(degrees, mode), D, mode, role, &TradHarmonyParams::default())
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
            assert_eq!(chords.map(|c| c.degree), [expected, expected], "{degrees}");
            assert_eq!(chords.map(|c| c.quality), [ChordQuality::Major; 2], "{degrees}");
        }
    }

    #[test]
    fn halves_are_voted_independently() {
        let chords = harmonize("1 3 5 3 4 6 8 6", Mode::Ionian, free_role());
        assert_eq!(chords.map(|c| c.degree), [ChordDegree::I, ChordDegree::IV]);
    }

    #[test]
    fn cadential_roles_override_the_melody() {
        // Mélodie qui appelle clairement I partout.
        let tonic_melody = "1 3 5 3 1 3 5 3";
        let half = harmonize(tonic_melody, Mode::Ionian, role(3));
        assert_eq!(half.map(|c| c.degree), [ChordDegree::I, ChordDegree::V]);
        let full = harmonize(tonic_melody, Mode::Ionian, role(7));
        assert_eq!(full.map(|c| c.degree), [ChordDegree::V, ChordDegree::I]);
        // Et une mélodie sur V ne change pas la cadence parfaite.
        let full = harmonize("5 7 2 7 5 7 2 7", Mode::Ionian, role(7));
        assert_eq!(full.map(|c| c.degree), [ChordDegree::V, ChordDegree::I]);
    }

    #[test]
    fn cadence_uses_modal_qualities() {
        let full = harmonize("1 3 5 3 1 3 5 3", Mode::Dorian, role(7));
        assert_eq!(full, [minor(ChordDegree::V), minor(ChordDegree::I)]);
    }

    #[test]
    fn ties_prefer_i_then_v_then_iv() {
        // Aucune note : tous les scores sont nuls.
        let chords =
            harmonize_bar(&[], D, Mode::Ionian, free_role(), &TradHarmonyParams::default());
        assert_eq!(chords.map(|c| c.degree), [ChordDegree::I; 2]);
        // « 5 » (A) appartient à I (D F# A) et à V (A C# E) : I l'emporte par défaut…
        let chords = harmonize("5 5 5 5 5 5 5 5", Mode::Ionian, free_role());
        assert_eq!(chords.map(|c| c.degree), [ChordDegree::I; 2]);
        // … et l'ordre de priorité vient des paramètres.
        let params = TradHarmonyParams {
            tie_priority: vec![ChordDegree::V, ChordDegree::I],
            ..TradHarmonyParams::default()
        };
        let notes = melody("5 5 5 5 5 5 5 5", Mode::Ionian);
        let chords = harmonize_bar(&notes, D, Mode::Ionian, free_role(), &params);
        assert_eq!(chords.map(|c| c.degree), [ChordDegree::V; 2]);
        // « 1 » appartient à I, IV et vi : I l'emporte à l'égalité.
        let chords = harmonize("1 1 1 1 1 1 1 1", Mode::Ionian, free_role());
        assert_eq!(chords.map(|c| c.degree), [ChordDegree::I; 2]);
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
        assert_eq!(candidate_chords(Mode::Ionian), [major(I), major(IV), major(V), minor(VI)]);
        assert_eq!(
            candidate_chords(Mode::Mixolydian),
            [major(I), major(IV), minor(V), major(BVII)]
        );
        assert_eq!(candidate_chords(Mode::Dorian), [minor(I), major(IV), major(BVII), minor(V)]);
        assert_eq!(
            candidate_chords(Mode::Aeolian),
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
        (0..TUNE_HARMONY_BARS)
            .map(|i| {
                melody(if i < 8 { "1 3 5 3 1 3 5 3" } else { "4 6 8 6 4 6 8 6" }, Mode::Ionian)
            })
            .collect()
    }

    #[test]
    fn repeats_return_identical_chords() {
        let plan = TunePlan::reel(D, Mode::Ionian);
        let harmony = TuneHarmony::new(plan, &two_strain_melody(), &TradHarmonyParams::default())
            .expect("16 mesures");
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
        assert_eq!(harmony.chords_for_bar(0).map(|c| c.degree), [ChordDegree::I; 2]);
        assert_eq!(harmony.chords_for_bar(16).map(|c| c.degree), [ChordDegree::IV; 2]);
        for strain_start in [0, 8, 16, 24] {
            assert_eq!(harmony.chords_for_bar(strain_start + 3)[1].degree, ChordDegree::V);
            assert_eq!(
                harmony.chords_for_bar(strain_start + 7).map(|c| c.degree),
                [ChordDegree::V, ChordDegree::I]
            );
        }
        // Repli modulo 32.
        assert_eq!(harmony.chords_for_bar(32), harmony.chords_for_bar(0));
    }

    #[test]
    fn tune_harmony_rejects_wrong_bar_count() {
        let plan = TunePlan::reel(D, Mode::Ionian);
        let few = vec![melody("1 3 5 3 1 3 5 3", Mode::Ionian); 15];
        assert_eq!(
            TuneHarmony::new(plan, &few, &TradHarmonyParams::default()),
            Err(TuneHarmonyError::WrongBarCount { expected: 16, found: 15 })
        );
    }
}

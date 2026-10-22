//! Basse et accords d'accompagnement : motifs en pas du profil, triades
//! en conduite de voix serrée et alternance fondamentale/quinte de la basse.

use super::{
    harmony::TradChord,
    mode::{Mode, PitchClass},
};
use crate::{
    timeline::{Articulation, NoteId, TimelineNote},
    tuning::TradHarmonyParams,
};

/// Place la classe de hauteur `pc` dans `low..=high` (voir le doc du module
/// pour la classe absente d'une fenêtre de 11 demi-tons).
fn place_in_range(pc: PitchClass, low: u8, high: u8) -> u8 {
    let low = i16::from(low);
    let high = i16::from(high);
    let pc = i16::from(pc);
    let above = low + (pc - low).rem_euclid(12);
    let chosen = if above <= high {
        above
    } else {
        let below = above - 12;
        if above - high < low - below { above } else { below }
    };
    // `chosen` reste dans 0..=127 pour toute tessiture valide (voir `validate`).
    u8::try_from(chosen.clamp(0, 127)).unwrap_or(low as u8)
}

/// Basse d'une mesure : une note par pas de `bass_steps` (voir le doc du module).
#[must_use]
pub fn bass_bar(
    chords: &[TradChord],
    key: PitchClass,
    mode: Mode,
    params: &TradHarmonyParams,
    steps_per_chord: usize,
    next_id: &mut impl FnMut() -> NoteId,
) -> Vec<TimelineNote> {
    params
        .bass_steps
        .iter()
        .enumerate()
        .map(|(index, &step)| {
            let slot = step / steps_per_chord;
            let root = chords[slot].root_pc(key, mode);
            let unchanged = slot == 0 || chords[slot] == chords[slot - 1];
            let pc = if index % 2 == 1 && unchanged { (root + 7) % 12 } else { root };
            TimelineNote {
                id: next_id(),
                pitch: place_in_range(pc, params.bass_low, params.bass_high),
                start_step: step,
                duration_steps: params.bass_duration_steps,
                velocity: params.bass_velocity,
                articulation: Articulation::Normal,
            }
        })
        .collect()
}

/// Voicings serrés d'une triade : 3 sons ascendants dans moins d'une octave.
fn close_voicings(chord: TradChord, key: PitchClass, mode: Mode) -> impl Iterator<Item = [u8; 3]> {
    let root = chord.root_pc(key, mode);
    let tones = [0, chord.third_semitones(), 7];
    (0..=10u8).flat_map(move |octave| {
        (0..3usize).filter_map(move |inversion| {
            let base = u16::from(root) + 12 * u16::from(octave);
            let note = |i: usize| {
                let wrapped = if i % 3 < inversion { 12 } else { 0 };
                u8::try_from(base + u16::from(tones[i % 3]) + wrapped).ok().filter(|&n| n <= 127)
            };
            Some([note(inversion)?, note(inversion + 1)?, note(inversion + 2)?])
        })
    })
}

/// Guitare d'accompagnement : enchaîne les voicings d'une mesure à l'autre.
///
/// Un `Guitarist` neuf démarre sans voicing précédent ; en créer un par tune.
#[derive(Clone, Debug, Default)]
pub struct Guitarist {
    previous: Option<[u8; 3]>,
}

impl Guitarist {
    #[must_use]
    pub const fn new() -> Self {
        Self { previous: None }
    }

    /// Voicing de `chord` : le plus proche du précédent (ou de `guitar_center`
    /// au premier accord), dans la limite de dérive autorisée.
    fn voice(
        &mut self,
        chord: TradChord,
        key: PitchClass,
        mode: Mode,
        params: &TradHarmonyParams,
    ) -> Option<[u8; 3]> {
        let center_sum = 3 * i32::from(params.guitar_center);
        let max_drift_sum = 3 * i32::from(params.guitar_max_drift);
        let best = close_voicings(chord, key, mode).min_by_key(|v| {
            let sum: i32 = v.iter().map(|&n| i32::from(n)).sum();
            let center_dist = (sum - center_sum).abs();
            let movement: i32 = self.previous.map_or(0, |p| {
                v.iter().zip(p).map(|(&a, b)| (i32::from(a) - i32::from(b)).abs()).sum()
            });
            (center_dist > max_drift_sum, movement, center_dist, v[0])
        })?;
        self.previous = Some(best);
        Some(best)
    }

    /// Chops d'une mesure : un accord de 3 sons par pas de `chop_steps`.
    pub fn chop_bar(
        &mut self,
        chords: &[TradChord],
        key: PitchClass,
        mode: Mode,
        params: &TradHarmonyParams,
        steps_per_chord: usize,
        next_id: &mut impl FnMut() -> NoteId,
    ) -> Vec<TimelineNote> {
        self.chop_bar_on_steps(
            chords,
            key,
            mode,
            params,
            &params.chop_steps,
            steps_per_chord,
            next_id,
        )
    }

    /// Variante pilotée par un motif emprunté au profil, sans copier le tuning.
    pub(crate) fn chop_bar_on_steps(
        &mut self,
        chords: &[TradChord],
        key: PitchClass,
        mode: Mode,
        params: &TradHarmonyParams,
        steps: &[usize],
        steps_per_chord: usize,
        next_id: &mut impl FnMut() -> NoteId,
    ) -> Vec<TimelineNote> {
        let mut notes = Vec::with_capacity(steps.len() * 3);
        for &step in steps {
            let Some(voicing) = self.voice(chords[step / steps_per_chord], key, mode, params)
            else {
                continue;
            };
            notes.extend(voicing.map(|pitch| TimelineNote {
                id: next_id(),
                pitch,
                start_step: step,
                duration_steps: params.chop_duration_steps,
                velocity: params.chop_velocity,
                articulation: Articulation::Normal,
            }));
        }
        notes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trad::harmony::{ChordDegree, ChordQuality, candidate_chords};

    fn profile() -> &'static crate::StyleProfile {
        static P: std::sync::LazyLock<crate::StyleProfile> =
            std::sync::LazyLock::new(crate::StyleProfile::default);
        &P
    }
    const D: PitchClass = 2;

    fn ids() -> impl FnMut() -> NoteId {
        let mut next = 0;
        move || {
            next += 1;
            next
        }
    }

    fn major(degree: ChordDegree) -> TradChord {
        TradChord::new(degree, ChordQuality::Major)
    }

    fn chop(guitarist: &mut Guitarist, chords: [TradChord; 2]) -> Vec<TimelineNote> {
        guitarist.chop_bar(&chords, D, Mode::Ionian, &TradHarmonyParams::default(), 8, &mut ids())
    }

    #[test]
    fn bass_plays_root_then_fifth_on_steps_0_and_8() {
        let one = [major(ChordDegree::I); 2];
        let notes = bass_bar(&one, D, Mode::Ionian, &TradHarmonyParams::default(), 8, &mut ids());
        let layout: Vec<_> =
            notes.iter().map(|n| (n.start_step, n.duration_steps, n.pitch)).collect();
        // Ré2 = 38 est sous la fenêtre : fondamentale Ré3 (50), quinte La2 (45).
        assert_eq!(layout, [(0, 4, 50), (8, 4, 45)]);
        let params = TradHarmonyParams::default();
        assert!(notes.iter().all(|n| n.velocity == params.bass_velocity));
    }

    #[test]
    fn bass_plays_root_when_chord_changes_at_half_bar() {
        let chords = [major(ChordDegree::I), major(ChordDegree::IV)];
        let notes =
            bass_bar(&chords, D, Mode::Ionian, &TradHarmonyParams::default(), 8, &mut ids());
        // Sol2 = 43.
        assert_eq!(
            notes.iter().map(|n| (n.start_step, n.pitch)).collect::<Vec<_>>(),
            [(0, 50), (8, 43)]
        );
    }

    #[test]
    fn bass_stays_in_e2_to_d3_for_every_key_mode_and_chord() {
        let params = TradHarmonyParams::default();
        for key in 0..12 {
            for mode in Mode::ALL {
                for &a in candidate_chords(mode, profile()) {
                    for &b in candidate_chords(mode, profile()) {
                        for n in bass_bar(&[a, b], key, mode, &params, 8, &mut ids()) {
                            let in_window = (40..=50).contains(&n.pitch);
                            // Seule la classe Ré♯/Mi♭ sort de la fenêtre (Mi♭2 = 39).
                            assert!(in_window || (n.pitch == 39 && n.pitch % 12 == 3), "{n:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn bass_flat_two_falls_one_semitone_below_the_window() {
        // Sol éolien : bVI = Mi♭, seule classe absente de E2–D3.
        let a = TradChord::new(ChordDegree::VI, ChordQuality::Major);
        let notes =
            bass_bar(&[a, a], 7, Mode::Aeolian, &TradHarmonyParams::default(), 8, &mut ids());
        assert_eq!(notes[0].pitch, 39);
    }

    #[test]
    fn guitar_chops_on_steps_4_and_12_with_three_chord_tones() {
        let chords = [major(ChordDegree::I), major(ChordDegree::IV)];
        let notes = chop(&mut Guitarist::new(), chords);
        assert_eq!(notes.len(), 6);
        for (i, step) in [4, 12].into_iter().enumerate() {
            let group: Vec<_> = notes.iter().filter(|n| n.start_step == step).collect();
            assert_eq!(group.len(), 3, "step {step}");
            let chop_velocity = TradHarmonyParams::default().chop_velocity;
            assert!(group.iter().all(|n| n.duration_steps == 1 && n.velocity == chop_velocity));
            let mut pitches: Vec<u8> = group.iter().map(|n| n.pitch).collect();
            pitches.sort_unstable();
            pitches.dedup();
            assert_eq!(pitches.len(), 3, "3 notes distinctes au pas {step}");
            let pcs = chords[i].pitch_classes(D, Mode::Ionian);
            let mut group_pcs: Vec<u8> = pitches.iter().map(|p| p % 12).collect();
            group_pcs.sort_unstable();
            let mut expected = pcs.to_vec();
            expected.sort_unstable();
            assert_eq!(group_pcs, expected, "step {step}");
            assert!(pitches[2] - pitches[0] < 12, "position serrée");
        }
    }

    #[test]
    fn first_voicing_is_closest_to_c4() {
        let notes = chop(&mut Guitarist::new(), [major(ChordDegree::I); 2]);
        let mut pitches: Vec<u8> = notes.iter().take(3).map(|n| n.pitch).collect();
        pitches.sort_unstable();
        // Ré majeur : La3 Ré4 Fa♯4, centre 61,7 (C4 = 60).
        assert_eq!(pitches, [57, 62, 66]);
    }

    #[test]
    fn next_voicing_is_the_closest_to_the_previous_one() {
        let mut guitarist = Guitarist::new();
        let _ = chop(&mut guitarist, [major(ChordDegree::I); 2]);
        let notes = chop(&mut guitarist, [major(ChordDegree::IV); 2]);
        let mut pitches: Vec<u8> = notes.iter().take(3).map(|n| n.pitch).collect();
        pitches.sort_unstable();
        // La3 Ré4 Fa♯4 → Si3 Ré4 Sol4 (mouvement total 1+0+1 = 2).
        assert_eq!(pitches, [59, 62, 67]);
    }

    #[test]
    fn voicings_never_drift_away_from_c4() {
        let params = TradHarmonyParams::default();
        let mut guitarist = Guitarist::new();
        let mut next_id = ids();
        for bar in 0..64 {
            let mode = Mode::ALL[bar % 4];
            let candidates = candidate_chords(mode, profile());
            let chords =
                [candidates[bar % candidates.len()], candidates[(bar * 3 + 1) % candidates.len()]];
            let notes = guitarist.chop_bar(&chords, D, mode, &params, 8, &mut next_id);
            for group in notes.chunks(3) {
                let sum: i32 = group.iter().map(|n| i32::from(n.pitch)).sum();
                assert!((sum - 3 * 60).abs() <= 3 * 6, "bar {bar}: {group:?}");
            }
        }
    }

    #[test]
    fn note_ids_are_unique_across_bass_and_guitar() {
        let mut next_id = ids();
        let params = TradHarmonyParams::default();
        let chords = [major(ChordDegree::I); 2];
        let mut all = bass_bar(&chords, D, Mode::Ionian, &params, 8, &mut next_id);
        all.extend(Guitarist::new().chop_bar(&chords, D, Mode::Ionian, &params, 8, &mut next_id));
        let mut found: Vec<_> = all.iter().map(|n| n.id).collect();
        found.sort_unstable();
        found.dedup();
        assert_eq!(found.len(), all.len());
    }
}

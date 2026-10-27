//! Modes du moteur : diatoniques traditionnels québécois, plus le mineur
//! harmonique (couleur sombre à sensible des boucles de jeu).
//!
//! ## Représentation des degrés (`degree: i8`)
//!
//! Les degrés sont relatifs à la tonique du mode (pas de chromatisme) :
//! - `1..=7` : degrés 1 à 7 de l'octave principale (`1` = tonique = `0` demi-ton,
//!   `5` = quinte = `7` demi-tons) ;
//! - `8..=14` : octave supérieure (`8` = tonique + 1 octave = `12` demi-tons,
//!   `9` = seconde + 1 octave, etc.) ;
//! - `-7..=-1` : octave inférieure, correspondant à la notation `k,` du plan
//!   via la formule `degree = k - 8` (`7,` = `-1`, `6,` = `-2`, `5,` = `-3`,
//!   `4,` = `-4`, `3,` = `-5`, `2,` = `-6`, `1,` = `-7` = `-12` demi-tons) ;
//! - `0` : traité comme `-1` (`7,`, un degré sous la tonique vers le grave)
//!   afin que tout `degree <= 0` descende sous la tonique sans ambiguïté.
//!
//! Le mineur harmonique déroge au diatonisme (seconde augmentée 6–7) : les
//! degrés restent une échelle à sept notes, donc tout le code générique
//! (cellules, ornements, marche de basse, levées) s'y applique sans branche.
//!
//! Parallèles clairs par la valence (données `parallel_modes`) : le phrygien
//! s'assombrit depuis le mixolydien (même septième mineure, degrés altérés
//! minimaux : 2, 3, 6), le mineur harmonique depuis l'ionien (même sensible,
//! degrés altérés minimaux : 3, 6).

use serde::{Deserialize, Serialize};

use crate::theory::PitchClass;

/// Mode utilisé par le répertoire traditionnel et les boucles de jeu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Majeur naturel (ex. ré majeur, sol majeur, la majeur).
    #[default]
    #[serde(alias = "Ionian")]
    Ionian,
    /// Majeur à septième mineure (ex. sol mixolydien, la mixolydien).
    #[serde(alias = "Mixolydian")]
    Mixolydian,
    /// Mineur à sixte majeure (ex. ré dorien, mi dorien, la dorien).
    #[serde(alias = "Dorian")]
    Dorian,
    /// Mineur naturel (ex. la éolien, mi éolien).
    #[serde(alias = "Aeolian")]
    Aeolian,
    /// Mineur à seconde mineure (couleur sombre des boucles de jeu : b2).
    Phrygian,
    /// Mineur à sensible et sixte mineure (V majeur, couleur tendue des
    /// boucles de jeu).
    HarmonicMinor,
}

impl Mode {
    /// Tous les modes supportés en ordre canonique.
    pub const ALL: [Self; 6] = [
        Self::Ionian,
        Self::Mixolydian,
        Self::Dorian,
        Self::Aeolian,
        Self::Phrygian,
        Self::HarmonicMinor,
    ];

    /// Intervalles en demi-tons depuis la tonique pour les degrés `1..=7`
    /// dans une octave (indices `0..7`).
    #[must_use]
    pub const fn intervals(self) -> [i8; 7] {
        match self {
            Self::Ionian => [0, 2, 4, 5, 7, 9, 11],
            Self::Mixolydian => [0, 2, 4, 5, 7, 9, 10],
            Self::Dorian => [0, 2, 3, 5, 7, 9, 10],
            Self::Aeolian => [0, 2, 3, 5, 7, 8, 10],
            Self::Phrygian => [0, 1, 3, 5, 7, 8, 10],
            Self::HarmonicMinor => [0, 2, 3, 5, 7, 8, 11],
        }
    }

    /// Indique si le mode possède une tierce mineure (Dorien, Éolien,
    /// Phrygien, mineur harmonique) : ces modes sont le côté sombre de la
    /// bascule de valence.
    #[must_use]
    pub const fn is_minor(self) -> bool {
        matches!(self, Self::Dorian | Self::Aeolian | Self::Phrygian | Self::HarmonicMinor)
    }

    /// Convertit un degré diatonique en décalage de demi-tons par rapport à la
    /// tonique (`1` → `0`, `8` → `12`, `7,` (`-1` ou `0`) → `-1`/`-2`,
    /// `5,` (`-3`) → `-5`, `1,` (`-7`) → `-12`).
    #[must_use]
    pub const fn degree_to_semitones(self, degree: i8) -> i8 {
        let steps = degree_to_diatonic_step(degree) as i32;
        let octave = steps.div_euclid(7);
        let idx = steps.rem_euclid(7) as usize;
        let semitones = octave * 12 + (self.intervals()[idx] as i32);
        if semitones > i8::MAX as i32 {
            i8::MAX
        } else if semitones < i8::MIN as i32 {
            i8::MIN
        } else {
            semitones as i8
        }
    }

    /// Convertit un degré diatonique en note MIDI à partir de la note MIDI de
    /// la tonique (ex. `tonic_midi = 62` pour D4).
    #[must_use]
    pub const fn degree_to_midi(self, tonic_midi: u8, degree: i8) -> u8 {
        let raw = (tonic_midi as i16) + (self.degree_to_semitones(degree) as i16);
        if raw < 0 {
            0
        } else if raw > 127 {
            127
        } else {
            raw as u8
        }
    }

    /// Retourne le degré d'échelle dans `1..=7` si la note MIDI appartient au
    /// mode pour la tonique `key` (`0..12`), ou `None` si la note est hors du mode.
    #[must_use]
    pub const fn scale_degree_of_midi(self, key: PitchClass, midi: u8) -> Option<u8> {
        let key_pc = (key % 12) as i8;
        let midi_pc = (midi % 12) as i8;
        let semitones = (midi_pc - key_pc).rem_euclid(12);
        let intervals = self.intervals();
        let mut i = 0usize;
        while i < 7 {
            if intervals[i] == semitones {
                return Some((i as u8) + 1);
            }
            i += 1;
        }
        None
    }

    /// Indique si la note MIDI appartient au mode pour la tonique `key`.
    #[must_use]
    pub const fn contains_midi(self, key: PitchClass, midi: u8) -> bool {
        self.scale_degree_of_midi(key, midi).is_some()
    }
}

/// Convertit un degré (`1` = tonique, `8` = octave, `5,` = `-3`, `7,` = `-1`)
/// en nombre linéaire de pas diatoniques depuis la tonique (`0` pour `1`,
/// `7` pour `8`, `-1` pour `7,` / `0`, `-3` pour `5,`, `-7` pour `1,`).
#[must_use]
pub const fn degree_to_diatonic_step(degree: i8) -> i8 {
    if degree >= 1 {
        degree - 1
    } else if degree == 0 {
        -1
    } else {
        degree
    }
}

/// Convertit un pas diatonique linéaire depuis la tonique (`0` = tonique) en
/// degré selon la convention du module (`0..` → `1..`, `< 0` → `-1..`).
#[must_use]
pub const fn diatonic_step_to_degree(step: i8) -> i8 {
    if step >= 0 { step.saturating_add(1) } else { step }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ionian_degrees_across_octaves() {
        let expected_main = [0, 2, 4, 5, 7, 9, 11, 12];
        for (i, &semi) in expected_main.iter().enumerate() {
            let deg = (i + 1) as i8;
            assert_eq!(Mode::Ionian.degree_to_semitones(deg), semi, "Ionian deg {deg}");
        }

        // Octave inférieure : 7, (-1) = -1, 6, (-2) = -3, 5, (-3) = -5, 1, (-7) = -12
        assert_eq!(Mode::Ionian.degree_to_semitones(0), -1);
        assert_eq!(Mode::Ionian.degree_to_semitones(-1), -1);
        assert_eq!(Mode::Ionian.degree_to_semitones(-2), -3);
        assert_eq!(Mode::Ionian.degree_to_semitones(-3), -5);
        assert_eq!(Mode::Ionian.degree_to_semitones(-4), -7);
        assert_eq!(Mode::Ionian.degree_to_semitones(-5), -8);
        assert_eq!(Mode::Ionian.degree_to_semitones(-6), -10);
        assert_eq!(Mode::Ionian.degree_to_semitones(-7), -12);

        // Octave supérieure : 9 = 14, 10 = 16
        assert_eq!(Mode::Ionian.degree_to_semitones(9), 14);
        assert_eq!(Mode::Ionian.degree_to_semitones(10), 16);
    }

    #[test]
    fn mixolydian_degrees() {
        let expected = [0, 2, 4, 5, 7, 9, 10, 12];
        for (i, &semi) in expected.iter().enumerate() {
            let deg = (i + 1) as i8;
            assert_eq!(Mode::Mixolydian.degree_to_semitones(deg), semi, "Mixolydian deg {deg}");
        }
        // 7, (-1) en mixolydien est un ton entier sous la tonique (-2 demi-tons)
        assert_eq!(Mode::Mixolydian.degree_to_semitones(-1), -2);
        assert_eq!(Mode::Mixolydian.degree_to_semitones(0), -2);
        // 5, (-3) reste à -5 demi-tons
        assert_eq!(Mode::Mixolydian.degree_to_semitones(-3), -5);
        assert_eq!(Mode::Mixolydian.degree_to_semitones(-7), -12);
    }

    #[test]
    fn dorian_degrees() {
        let expected = [0, 2, 3, 5, 7, 9, 10, 12];
        for (i, &semi) in expected.iter().enumerate() {
            let deg = (i + 1) as i8;
            assert_eq!(Mode::Dorian.degree_to_semitones(deg), semi, "Dorian deg {deg}");
        }
        assert_eq!(Mode::Dorian.degree_to_semitones(-1), -2);
        assert_eq!(Mode::Dorian.degree_to_semitones(-2), -3); // sixte majeure grave : 9 - 12 = -3
        assert_eq!(Mode::Dorian.degree_to_semitones(-3), -5);
        assert_eq!(Mode::Dorian.degree_to_semitones(-5), -9); // tierce mineure grave : 3 - 12 = -9
        assert_eq!(Mode::Dorian.degree_to_semitones(-7), -12);
    }

    #[test]
    fn aeolian_degrees() {
        let expected = [0, 2, 3, 5, 7, 8, 10, 12];
        for (i, &semi) in expected.iter().enumerate() {
            let deg = (i + 1) as i8;
            assert_eq!(Mode::Aeolian.degree_to_semitones(deg), semi, "Aeolian deg {deg}");
        }
        assert_eq!(Mode::Aeolian.degree_to_semitones(-1), -2);
        assert_eq!(Mode::Aeolian.degree_to_semitones(-2), -4); // sixte mineure grave : 8 - 12 = -4
        assert_eq!(Mode::Aeolian.degree_to_semitones(-3), -5);
        assert_eq!(Mode::Aeolian.degree_to_semitones(-7), -12);
    }

    #[test]
    fn phrygian_degrees() {
        // Seconde mineure (b2) : la couleur sombre des boucles de jeu.
        let expected = [0, 1, 3, 5, 7, 8, 10, 12];
        for (i, &semi) in expected.iter().enumerate() {
            let deg = (i + 1) as i8;
            assert_eq!(Mode::Phrygian.degree_to_semitones(deg), semi, "Phrygian deg {deg}");
        }
        assert_eq!(Mode::Phrygian.degree_to_semitones(-1), -2);
        assert_eq!(Mode::Phrygian.degree_to_semitones(-2), -4); // sixte mineure grave : 8 - 12 = -4
        assert_eq!(Mode::Phrygian.degree_to_semitones(-6), -11); // seconde mineure grave : 1 - 12
        assert_eq!(Mode::Phrygian.degree_to_semitones(-7), -12);
        assert!(Mode::Phrygian.is_minor());
    }

    #[test]
    fn harmonic_minor_degrees() {
        // Sensible (degré 7 à 11 demi-tons) et sixte mineure : le V est majeur.
        let expected = [0, 2, 3, 5, 7, 8, 11, 12];
        for (i, &semi) in expected.iter().enumerate() {
            let deg = (i + 1) as i8;
            assert_eq!(
                Mode::HarmonicMinor.degree_to_semitones(deg),
                semi,
                "HarmonicMinor deg {deg}"
            );
        }
        assert_eq!(Mode::HarmonicMinor.degree_to_semitones(-1), -1); // sensible grave : 11 - 12
        assert_eq!(Mode::HarmonicMinor.degree_to_semitones(-2), -4); // sixte mineure grave : 8 - 12
        assert_eq!(Mode::HarmonicMinor.degree_to_semitones(-7), -12);
        assert!(Mode::HarmonicMinor.is_minor());
    }

    #[test]
    fn new_modes_hold_seven_distinct_pitch_classes() {
        for mode in [Mode::Phrygian, Mode::HarmonicMinor] {
            let mut pcs = mode.intervals().to_vec();
            pcs.sort_unstable();
            pcs.dedup();
            assert_eq!(pcs.len(), 7, "{mode:?}");
            // Aucune tierce majeure : côté sombre de la bascule de valence.
            assert!(mode.is_minor(), "{mode:?}");
            assert!(Mode::ALL.contains(&mode), "{mode:?}");
        }
        assert!(!Mode::Ionian.is_minor());
        assert!(!Mode::Mixolydian.is_minor());
    }

    #[test]
    fn diatonic_step_roundtrip() {
        for deg in (-7..=-1).chain(1..=14) {
            let step = degree_to_diatonic_step(deg);
            assert_eq!(diatonic_step_to_degree(step), deg);
        }
    }
}

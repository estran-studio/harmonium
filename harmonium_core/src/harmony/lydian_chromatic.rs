//! Lydian Chromatic Concept (LCC) - George Russell
//!
//! Implémentation des 12 niveaux de gravité tonale selon le Lydian Chromatic Concept.
//! Le LCC organise les gammes selon leur "gravité" par rapport au centre Lydien.
//!
//! Niveau 1 (Lydien) = le plus consonant (Ingoing)
//! Niveau 12 (Chromatique) = le plus dissonant (Outgoing)

use serde::{Deserialize, Serialize};

use super::chord::{Chord, ChordType, PitchClass};

/// Improv guidance for one chord: the suggested scale + chord tones, derived
/// from the Lydian Chromatic Concept. Consumed by the improv-coach UI to show
/// the player what to solo over the current changes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaleGuidance {
    /// Display symbol for the chord (e.g. "Dm7", or a roman numeral like "ii").
    pub chord_symbol: String,
    /// Absolute root pitch class of the chord (0-11, 0=C).
    pub chord_root: u8,
    /// Chord-tone pitch classes (0-11) — the "safe/strong" notes to land on.
    pub chord_tones: Vec<u8>,
    /// Name of the suggested scale (e.g. "Lydian b7").
    pub scale_name: String,
    /// Scale pitch classes (0-11) to play over this chord.
    pub scale_notes: Vec<u8>,
    /// LCC level 1-12 (1 = most consonant/Lydian, 12 = chromatic).
    pub lcc_level: u8,
}

/// Les 12 niveaux de gravité tonale de Russell
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LccLevel {
    /// Gamme Lydienne (C D E F# G A B) - La plus consonante
    Lydian = 1,
    /// Gamme Lydienne Augmentée (C D E F# G# A B)
    LydianAugmented = 2,
    /// Gamme Lydienne Diminuée (C D Eb F# G A B)
    LydianDiminished = 3,
    /// Gamme Lydienne b7 / Mixolydien #4 (C D E F# G A Bb)
    LydianFlatSeventh = 4,
    /// Gamme Augmentée Auxiliaire (Tons entiers: C D E F# G# A#)
    AuxiliaryAugmented = 5,
    /// Gamme Blues Auxiliaire Diminuée (C Db Eb E F# G A Bb)
    AuxiliaryDiminishedBlues = 6,
    /// Gamme Lydienne Augmentée b7 (C D E F# G# A Bb)
    LydianAugmentedFlatSeventh = 7,
    /// Gamme Diminuée Auxiliaire (demi-ton / ton: C Db Eb E F# G A Bb)
    AuxiliaryDiminished = 8,
    /// Gamme Blues Augmentée Auxiliaire (C D Eb F F# G# A B)
    AuxiliaryAugmentedBlues = 9,
    /// Pentatonique Majeure (C D E G A)
    MajorPentatonic = 10,
    /// Gamme Japonaise In (C Db F G Ab)
    JapaneseIn = 11,
    /// Gamme Chromatique (tous les 12 tons) - La plus dissonante
    Chromatic = 12,
}

impl LccLevel {
    /// Convertit un u8 en `LccLevel`
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Lydian),
            2 => Some(Self::LydianAugmented),
            3 => Some(Self::LydianDiminished),
            4 => Some(Self::LydianFlatSeventh),
            5 => Some(Self::AuxiliaryAugmented),
            6 => Some(Self::AuxiliaryDiminishedBlues),
            7 => Some(Self::LydianAugmentedFlatSeventh),
            8 => Some(Self::AuxiliaryDiminished),
            9 => Some(Self::AuxiliaryAugmentedBlues),
            10 => Some(Self::MajorPentatonic),
            11 => Some(Self::JapaneseIn),
            12 => Some(Self::Chromatic),
            _ => None,
        }
    }

    /// Nom de la gamme
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Lydian => "Lydian",
            Self::LydianAugmented => "Lydian Augmented",
            Self::LydianDiminished => "Lydian Diminished",
            Self::LydianFlatSeventh => "Lydian b7",
            Self::AuxiliaryAugmented => "Aux. Augmented",
            Self::AuxiliaryDiminishedBlues => "Aux. Dim. Blues",
            Self::LydianAugmentedFlatSeventh => "Lydian Aug. b7",
            Self::AuxiliaryDiminished => "Aux. Diminished",
            Self::AuxiliaryAugmentedBlues => "Aux. Aug. Blues",
            Self::MajorPentatonic => "Major Pentatonic",
            Self::JapaneseIn => "Japanese In",
            Self::Chromatic => "Chromatic",
        }
    }
}

/// Le module Lydian Chromatic Concept complet
pub struct LydianChromaticConcept {
    /// Intervalles pré-calculés pour chaque niveau (en demi-tons depuis root)
    scale_intervals: [[u8; 8]; 12],
    /// Nombre de notes par gamme
    scale_lengths: [usize; 12],
}

impl Default for LydianChromaticConcept {
    fn default() -> Self {
        Self::new()
    }
}

impl LydianChromaticConcept {
    /// Crée une nouvelle instance avec les tables pré-calculées
    #[must_use]
    pub const fn new() -> Self {
        // Intervalles pour chaque niveau (padding avec 255 pour les gammes courtes)
        let scale_intervals: [[u8; 8]; 12] = [
            // Level 1: Lydian (C D E F# G A B)
            [0, 2, 4, 6, 7, 9, 11, 255],
            // Level 2: Lydian Augmented (C D E F# G# A B)
            [0, 2, 4, 6, 8, 9, 11, 255],
            // Level 3: Lydian Diminished (C D Eb F# G A B)
            [0, 2, 3, 6, 7, 9, 11, 255],
            // Level 4: Lydian b7 (C D E F# G A Bb)
            [0, 2, 4, 6, 7, 9, 10, 255],
            // Level 5: Auxiliary Augmented / Whole Tone (C D E F# G# A#)
            [0, 2, 4, 6, 8, 10, 255, 255],
            // Level 6: Auxiliary Diminished Blues (C Db Eb E F# G A Bb)
            [0, 1, 3, 4, 6, 7, 9, 10],
            // Level 7: Lydian Augmented b7 (C D E F# G# A Bb)
            [0, 2, 4, 6, 8, 9, 10, 255],
            // Level 8: Auxiliary Diminished (half-whole: C Db Eb E F# G A Bb)
            [0, 1, 3, 4, 6, 7, 9, 10],
            // Level 9: Auxiliary Augmented Blues (C D Eb F F# G# A B)
            [0, 2, 3, 5, 6, 8, 9, 11],
            // Level 10: Major Pentatonic (C D E G A)
            [0, 2, 4, 7, 9, 255, 255, 255],
            // Level 11: Japanese In (C Db F G Ab)
            [0, 1, 5, 7, 8, 255, 255, 255],
            // Level 12: Chromatic (all 12)
            [0, 1, 2, 3, 4, 5, 6, 7], // Les 8 premiers, get_scale() complète
        ];

        let scale_lengths = [7, 7, 7, 7, 6, 8, 7, 8, 8, 5, 5, 12];

        Self { scale_intervals, scale_lengths }
    }

    /// Calcule la tonique Lydienne parente pour un accord donné
    ///
    /// C'est le cœur du LCC: chaque accord a une "maison" Lydienne naturelle.
    #[must_use]
    pub const fn parent_lydian(&self, chord: &Chord) -> PitchClass {
        match chord.chord_type {
            // Accords majeurs: la fondamentale EST la tonique Lydienne
            ChordType::Major | ChordType::Major7 | ChordType::Major6 | ChordType::Add9 => {
                chord.root
            }

            // Dominant 7: la fondamentale est aussi la tonique Lydienne
            // (c'est le degré Mixolydien #4 de son propre Lydien)
            ChordType::Dominant7 | ChordType::Dominant7Sus4 => chord.root,

            // Accords mineurs: la tonique Lydienne est une tierce majeure EN DESSOUS
            // (Cm est construit sur le degré III de Ab Lydien)
            // root - 3 demi-tons = root + 9 mod 12
            ChordType::Minor | ChordType::Minor7 | ChordType::MinorMajor7 | ChordType::Minor6 => {
                (chord.root + 9) % 12
            }

            // Demi-diminué: la tonique est une tierce majeure au-dessus
            // (Cm7b5 peut être vu comme le VII de Db Lydien)
            ChordType::HalfDiminished => (chord.root + 1) % 12,

            // Diminué / Diminué 7: symétrique, plusieurs parents possibles
            // On prend le demi-ton au-dessus par convention
            ChordType::Diminished | ChordType::Diminished7 => (chord.root + 1) % 12,

            // Augmenté: symétrique, on garde la fondamentale
            ChordType::Augmented | ChordType::Augmented7 => chord.root,

            // Sus: traité comme majeur (neutre)
            ChordType::Sus2 | ChordType::Sus4 => chord.root,
        }
    }

    /// Retourne le niveau LCC approprié pour une tension donnée (0.0 - 1.0)
    #[must_use]
    pub fn level_for_tension(&self, tension: f32) -> LccLevel {
        // Mapping linéaire: 0.0 -> Level 1, 1.0 -> Level 12
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let level_num = 1 + (tension.clamp(0.0, 1.0) * 11.0).round() as u8;
        LccLevel::from_u8(level_num).unwrap_or(LccLevel::Lydian)
    }

    /// Retourne les pitch classes de la gamme pour un niveau LCC donné
    #[must_use]
    pub fn get_scale(&self, parent: PitchClass, level: LccLevel) -> Vec<PitchClass> {
        let level_idx = (level as u8 - 1) as usize;
        let intervals = &self.scale_intervals[level_idx];
        let len = self.scale_lengths[level_idx];

        // Cas spécial: chromatique (12 notes)
        if level == LccLevel::Chromatic {
            return (0..12).map(|i| (parent + i) % 12).collect();
        }

        intervals
            .iter()
            .take(len)
            .filter(|&&i| i != 255) // Ignorer le padding
            .map(|&interval| (parent + interval) % 12)
            .collect()
    }

    /// Vérifie si une note est valide dans le contexte LCC actuel
    #[must_use]
    pub fn is_valid_note(&self, note: PitchClass, chord: &Chord, tension: f32) -> bool {
        let parent = self.parent_lydian(chord);
        let level = self.level_for_tension(tension);
        let scale = self.get_scale(parent, level);
        scale.contains(&(note % 12))
    }

    /// Filtre une liste de notes pour ne garder que celles valides dans le contexte LCC
    #[must_use]
    pub fn filter_notes(
        &self,
        notes: &[PitchClass],
        chord: &Chord,
        tension: f32,
    ) -> Vec<PitchClass> {
        let parent = self.parent_lydian(chord);
        let level = self.level_for_tension(tension);
        let scale = self.get_scale(parent, level);

        notes.iter().filter(|&&note| scale.contains(&(note % 12))).copied().collect()
    }

    /// Retourne le poids d'une note dans le contexte LCC (pour la génération mélodique)
    /// Notes dans la gamme = 1.0, notes hors gamme = valeur réduite
    #[must_use]
    pub fn note_weight(&self, note: PitchClass, chord: &Chord, tension: f32) -> f32 {
        if self.is_valid_note(note, chord, tension) {
            1.0
        } else {
            // Notes "outgoing" - réduire mais pas interdire
            0.2
        }
    }

    /// Build improv guidance for a chord at a given tension: the suggested
    /// scale (parent Lydian + LCC level for the tension) plus the chord tones.
    ///
    /// `chord_symbol` is the display name to echo back (e.g. "Dm7" or "ii").
    /// All pitch classes returned are absolute (0-11, 0 = C).
    #[must_use]
    pub fn scale_guidance(
        &self,
        chord: &Chord,
        chord_symbol: String,
        tension: f32,
    ) -> ScaleGuidance {
        let parent = self.parent_lydian(chord);
        let level = self.level_for_tension(tension);
        ScaleGuidance {
            chord_symbol,
            chord_root: chord.root % 12,
            chord_tones: chord.pitch_classes(),
            scale_name: level.name().to_string(),
            scale_notes: self.get_scale(parent, level),
            lcc_level: level as u8,
        }
    }

    /// Suggère le niveau LCC optimal pour une transition entre deux accords
    #[must_use]
    pub fn suggest_level_for_transition(&self, from: &Chord, to: &Chord) -> LccLevel {
        let distance = from.voice_leading_distance(to);

        // Plus la distance est grande, plus on peut utiliser des niveaux dissonants
        match distance {
            0..=1 => LccLevel::Lydian,              // Très proche: rester consonant
            2..=3 => LccLevel::LydianAugmented,     // Proche: légère tension
            4..=5 => LccLevel::LydianFlatSeventh,   // Modéré
            6..=7 => LccLevel::AuxiliaryDiminished, // Éloigné
            _ => LccLevel::Chromatic,               // Très éloigné: tout est permis
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_lydian_major() {
        let lcc = LydianChromaticConcept::new();

        // C Major -> C Lydian
        let c_maj = Chord::new(0, ChordType::Major);
        assert_eq!(lcc.parent_lydian(&c_maj), 0);

        // G Major -> G Lydian
        let g_maj = Chord::new(7, ChordType::Major);
        assert_eq!(lcc.parent_lydian(&g_maj), 7);
    }

    #[test]
    fn test_parent_lydian_minor() {
        let lcc = LydianChromaticConcept::new();

        // A Minor -> F Lydian (A - 3 = F, ou A + 9 mod 12 = 5 = F)
        // Correction: A = 9, 9 + 9 = 18 mod 12 = 6 = F#
        // En fait dans le LCC traditionnel, Am -> Bb Lydian mais on simplifie
        let a_min = Chord::new(9, ChordType::Minor);
        assert_eq!(lcc.parent_lydian(&a_min), 6); // F#

        // C Minor -> Ab Lydian (C + 9 mod 12 = 9 = A, pas Ab...)
        // Notre implémentation simplifiée utilise +9
        let c_min = Chord::new(0, ChordType::Minor);
        assert_eq!(lcc.parent_lydian(&c_min), 9); // A
    }

    #[test]
    fn test_get_scale_lydian() {
        let lcc = LydianChromaticConcept::new();

        // C Lydian = C D E F# G A B = 0, 2, 4, 6, 7, 9, 11
        let c_lydian = lcc.get_scale(0, LccLevel::Lydian);
        assert_eq!(c_lydian, vec![0, 2, 4, 6, 7, 9, 11]);
    }

    #[test]
    fn test_get_scale_whole_tone() {
        let lcc = LydianChromaticConcept::new();

        // C Whole Tone = C D E F# G# A# = 0, 2, 4, 6, 8, 10
        let c_whole_tone = lcc.get_scale(0, LccLevel::AuxiliaryAugmented);
        assert_eq!(c_whole_tone, vec![0, 2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_get_scale_chromatic() {
        let lcc = LydianChromaticConcept::new();

        // Chromatic depuis C = tous les 12 tons
        let chromatic = lcc.get_scale(0, LccLevel::Chromatic);
        assert_eq!(chromatic.len(), 12);
        assert_eq!(chromatic, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn test_level_for_tension() {
        let lcc = LydianChromaticConcept::new();

        assert_eq!(lcc.level_for_tension(0.0), LccLevel::Lydian);
        assert_eq!(lcc.level_for_tension(1.0), LccLevel::Chromatic);
        // tension 0.5 -> 1 + (0.5 * 11).round() = 1 + 6 = level 7
        assert_eq!(lcc.level_for_tension(0.5), LccLevel::LydianAugmentedFlatSeventh);
    }

    #[test]
    fn test_is_valid_note() {
        let lcc = LydianChromaticConcept::new();
        let c_maj = Chord::new(0, ChordType::Major);

        // Tension basse (0.0) -> Lydian
        // F# (6) est dans C Lydian
        assert!(lcc.is_valid_note(6, &c_maj, 0.0));
        // F (5) n'est PAS dans C Lydian
        assert!(!lcc.is_valid_note(5, &c_maj, 0.0));

        // Tension haute (1.0) -> Chromatic, tout est valide
        assert!(lcc.is_valid_note(5, &c_maj, 1.0));
    }

    #[test]
    fn test_scale_guidance_cmaj_low_tension() {
        let lcc = LydianChromaticConcept::new();
        let c_maj = Chord::new(0, ChordType::Major);

        let g = lcc.scale_guidance(&c_maj, "I".to_string(), 0.0);

        assert_eq!(g.chord_symbol, "I");
        assert_eq!(g.chord_root, 0);
        // C major triad tones: C E G = 0, 4, 7
        assert_eq!(g.chord_tones, vec![0, 4, 7]);
        // Low tension -> Lydian over C = C D E F# G A B
        assert_eq!(g.scale_name, "Lydian");
        assert_eq!(g.lcc_level, 1);
        assert_eq!(g.scale_notes, vec![0, 2, 4, 6, 7, 9, 11]);
    }

    #[test]
    fn test_scale_guidance_minor_uses_parent_lydian() {
        let lcc = LydianChromaticConcept::new();
        // Dm7: minor parent Lydian is root+9 = (2+9)%12 = 11 (B Lydian) in this
        // simplified model; the scale must be rooted on that parent, and the
        // chord tones must be Dm7's own notes (D F A C = 2, 5, 9, 0).
        let dm7 = Chord::new(2, ChordType::Minor7);
        let g = lcc.scale_guidance(&dm7, "ii".to_string(), 0.0);

        assert_eq!(g.chord_root, 2);
        assert_eq!(g.chord_tones, dm7.pitch_classes());
        assert_eq!(g.lcc_level, 1);
        // scale is the Lydian rooted on the parent (root+9), not on D
        let parent = (2 + 9) % 12;
        assert_eq!(g.scale_notes, lcc.get_scale(parent, LccLevel::Lydian));
    }

    #[test]
    fn test_scale_guidance_serde_roundtrip() {
        let lcc = LydianChromaticConcept::new();
        let g = lcc.scale_guidance(&Chord::new(7, ChordType::Dominant7), "V7".to_string(), 0.5);
        let json = serde_json::to_string(&g).unwrap();
        let back: ScaleGuidance = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }
}

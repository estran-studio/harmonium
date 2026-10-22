//! Module de représentation des accords
//!
//! Fournit une représentation enrichie des accords avec:
//! - `PitchClass` (0-11)
//! - `ChordType` étendu (Major, Minor, Aug, Dim, Dom7, etc.)
//! - Extensions et basse séparée
//! - Niveau LCC (Lydian Chromatic Concept)

use super::PitchClass;

/// Noms des notes pour affichage
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoteName {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

impl NoteName {
    /// Convertit un `PitchClass` en `NoteName`
    #[must_use]
    pub fn from_pitch_class(pc: PitchClass) -> Self {
        match pc % 12 {
            0 => Self::C,
            1 => Self::Cs,
            2 => Self::D,
            3 => Self::Ds,
            4 => Self::E,
            5 => Self::F,
            6 => Self::Fs,
            7 => Self::G,
            8 => Self::Gs,
            9 => Self::A,
            10 => Self::As,
            11 => Self::B,
            _ => unreachable!(),
        }
    }

    /// Convertit en `PitchClass`
    #[must_use]
    pub const fn to_pitch_class(self) -> PitchClass {
        match self {
            Self::C => 0,
            Self::Cs => 1,
            Self::D => 2,
            Self::Ds => 3,
            Self::E => 4,
            Self::F => 5,
            Self::Fs => 6,
            Self::G => 7,
            Self::Gs => 8,
            Self::A => 9,
            Self::As => 10,
            Self::B => 11,
        }
    }

    /// Nom pour affichage
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::C => "C",
            Self::Cs => "C#",
            Self::D => "D",
            Self::Ds => "D#",
            Self::E => "E",
            Self::F => "F",
            Self::Fs => "F#",
            Self::G => "G",
            Self::Gs => "G#",
            Self::A => "A",
            Self::As => "A#",
            Self::B => "B",
        }
    }
}

impl std::fmt::Display for NoteName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Types d'accords étendus
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChordType {
    /// Accord majeur (1-3-5)
    Major,
    /// Accord mineur (1-b3-5)
    Minor,
    /// Accord augmenté (1-3-#5) - symétrique, utile pour pivots
    Augmented,
    /// Accord diminué (1-b3-b5)
    Diminished,
    /// Accord dominant 7 (1-3-5-b7)
    Dominant7,
    /// Accord majeur 7 (1-3-5-7)
    Major7,
    /// Accord mineur 7 (1-b3-5-b7)
    Minor7,
    /// Accord demi-diminué (1-b3-b5-b7)
    HalfDiminished,
    /// Accord diminué 7 (1-b3-b5-bb7) - symétrique, utile pour pivots
    Diminished7,
    /// Accord sus2 (1-2-5) - neutre, pas de tierce
    Sus2,
    /// Accord sus4 (1-4-5) - neutre, utile pour pivots
    Sus4,
    // === NOUVEAUX TYPES (V2) ===
    /// Accord mineur majeur 7 (1-b3-5-7) - ex: `CmMaj7`
    MinorMajor7,
    /// Accord augmenté 7 (1-3-#5-b7) - ex: C7#5
    Augmented7,
    /// Accord majeur 6 (1-3-5-6) - ex: C6
    Major6,
    /// Accord mineur 6 (1-b3-5-6) - ex: Cm6
    Minor6,
    /// Accord dominant 7 sus4 (1-4-5-b7) - ex: C7sus4
    Dominant7Sus4,
    /// Accord add9 (1-3-5-9) - ex: Cadd9
    Add9,
}

impl ChordType {
    /// Retourne les intervalles (en demi-tons) depuis la fondamentale
    #[must_use]
    pub const fn intervals(&self) -> &'static [u8] {
        match self {
            Self::Major => &[0, 4, 7],
            Self::Minor => &[0, 3, 7],
            Self::Augmented => &[0, 4, 8],
            Self::Diminished => &[0, 3, 6],
            Self::Dominant7 => &[0, 4, 7, 10],
            Self::Major7 => &[0, 4, 7, 11],
            Self::Minor7 => &[0, 3, 7, 10],
            Self::HalfDiminished => &[0, 3, 6, 10],
            Self::Diminished7 => &[0, 3, 6, 9],
            Self::Sus2 => &[0, 2, 7],
            Self::Sus4 => &[0, 5, 7],
            Self::MinorMajor7 => &[0, 3, 7, 11],
            Self::Augmented7 => &[0, 4, 8, 10],
            Self::Major6 => &[0, 4, 7, 9],
            Self::Minor6 => &[0, 3, 7, 9],
            Self::Dominant7Sus4 => &[0, 5, 7, 10],
            Self::Add9 => &[0, 2, 4, 7],
        }
    }

    /// Retourne le nombre de notes dans l'accord (cardinalité)
    #[must_use]
    pub fn cardinality(&self) -> usize {
        self.intervals().len()
    }

    /// Vérifie si c'est une triade (3 notes)
    #[must_use]
    pub fn is_triad(&self) -> bool {
        self.cardinality() == 3
    }

    /// Vérifie si c'est un tétracorde (4 notes)
    #[must_use]
    pub fn is_tetrad(&self) -> bool {
        self.cardinality() == 4
    }

    /// Nom court pour affichage
    #[must_use]
    pub const fn suffix(&self) -> &'static str {
        match self {
            Self::Major => "",
            Self::Minor => "m",
            Self::Augmented => "+",
            Self::Diminished => "dim",
            Self::Dominant7 => "7",
            Self::Major7 => "maj7",
            Self::Minor7 => "m7",
            Self::HalfDiminished => "m7b5",
            Self::Diminished7 => "dim7",
            Self::Sus2 => "sus2",
            Self::Sus4 => "sus4",
            // Nouveaux types V2
            Self::MinorMajor7 => "mMaj7",
            Self::Augmented7 => "7#5",
            Self::Major6 => "6",
            Self::Minor6 => "m6",
            Self::Dominant7Sus4 => "7sus4",
            Self::Add9 => "add9",
        }
    }

    /// Vérifie si c'est un accord majeur (inclut Maj7, Dom7, 6)
    #[must_use]
    pub const fn is_major(&self) -> bool {
        matches!(
            self,
            Self::Major
                | Self::Major7
                | Self::Dominant7
                | Self::Augmented
                | Self::Major6
                | Self::Augmented7
                | Self::Add9
        )
    }

    /// Vérifie si c'est un accord mineur (inclut Min7, `HalfDim`, mMaj7)
    #[must_use]
    pub const fn is_minor(&self) -> bool {
        matches!(
            self,
            Self::Minor
                | Self::Minor7
                | Self::HalfDiminished
                | Self::Diminished
                | Self::Diminished7
                | Self::MinorMajor7
                | Self::Minor6
        )
    }

    /// Vérifie si c'est un accord symétrique (utile pour pivots Neo-Riemannian)
    #[must_use]
    pub const fn is_symmetric(&self) -> bool {
        matches!(self, Self::Augmented | Self::Diminished7)
    }

    /// Vérifie si c'est un accord ambigu (sans tierce claire)
    #[must_use]
    pub const fn is_ambiguous(&self) -> bool {
        matches!(
            self,
            Self::Sus2 | Self::Sus4 | Self::Augmented | Self::Diminished7 | Self::Dominant7Sus4
        )
    }
}

// ── Chord name parsing ──

/// Error returned by `Chord::from_name()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChordParseError {
    /// Input string was empty or only whitespace
    EmptyInput,
    /// Could not parse a valid root note (e.g., "X7")
    InvalidRoot,
    /// Could not match a chord quality suffix (e.g., "Cmaj9")
    InvalidQuality(String),
    /// Could not parse the bass note in a slash chord (e.g., "C/X")
    InvalidBass(String),
}

impl std::fmt::Display for ChordParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "empty chord name"),
            Self::InvalidRoot => write!(f, "invalid root note"),
            Self::InvalidQuality(s) => write!(f, "unrecognized chord quality: \"{s}\""),
            Self::InvalidBass(s) => write!(f, "invalid bass note: \"{s}\""),
        }
    }
}

impl std::error::Error for ChordParseError {}

/// Parse a root note from the beginning of a string.
/// Returns `(PitchClass, remaining_str)` on success.
/// Handles sharps (`C#`, `F#`) and flats (`Db`, `Bb`, `Ab`, etc.).
fn parse_root(s: &str) -> Option<(PitchClass, &str)> {
    let mut chars = s.chars();
    let letter = chars.next()?;
    let base = match letter.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let rest = &s[letter.len_utf8()..];
    if let Some(acc) = rest.chars().next() {
        match acc {
            '#' => Some(((base + 1) % 12, &rest[1..])),
            // 'b' is always treated as a flat sign. This is unambiguous because
            // chord quality suffixes never start with 'b' — they start with 'm',
            // 'd', 's', 'a', '+', or a digit. So "Bb" = A#, "Bbm7" = A#m7, etc.
            'b' => Some(((base + 11) % 12, &rest[1..])),
            _ => Some((base, rest)),
        }
    } else {
        Some((base, rest))
    }
}

/// Parse a chord type suffix from the beginning of a string.
/// Returns `(ChordType, remaining_str)` on success.
/// Matches longest prefixes first to avoid ambiguity.
fn parse_chord_type(s: &str) -> Option<(ChordType, &str)> {
    // Ordered longest-first to prevent partial matches
    const SUFFIXES: &[(&str, ChordType)] = &[
        ("mMaj7", ChordType::MinorMajor7),
        ("m7b5", ChordType::HalfDiminished),
        ("7sus4", ChordType::Dominant7Sus4),
        ("maj7", ChordType::Major7),
        ("dim7", ChordType::Diminished7),
        ("add9", ChordType::Add9),
        ("sus2", ChordType::Sus2),
        ("sus4", ChordType::Sus4),
        ("7#5", ChordType::Augmented7),
        ("dim", ChordType::Diminished),
        ("m7", ChordType::Minor7),
        ("m6", ChordType::Minor6),
        ("7", ChordType::Dominant7),
        ("6", ChordType::Major6),
        ("m", ChordType::Minor),
        ("+", ChordType::Augmented),
    ];

    for &(suffix, chord_type) in SUFFIXES {
        if let Some(rest) = s.strip_prefix(suffix) {
            return Some((chord_type, rest));
        }
    }

    // Empty suffix = Major
    Some((ChordType::Major, s))
}

/// Un accord avec contexte harmonique complet
#[derive(Clone, Debug, PartialEq)]
pub struct Chord {
    /// Fondamentale (0-11)
    pub root: PitchClass,
    /// Type d'accord
    pub chord_type: ChordType,
    /// Basse séparée (pour slash chords comme C/G)
    pub bass: Option<PitchClass>,
    /// Extensions (9, 11, 13) en pitch classes relatives
    pub extensions: Vec<PitchClass>,
    /// Niveau LCC (1-12, voir `lydian_chromatic.rs`)
    pub lcc_level: u8,
}

impl Default for Chord {
    fn default() -> Self {
        Self {
            root: 0, // C
            chord_type: ChordType::Major,
            bass: None,
            extensions: Vec::new(),
            lcc_level: 1, // Lydian (le plus consonant)
        }
    }
}

impl Chord {
    /// Crée un nouvel accord simple
    #[must_use]
    pub const fn new(root: PitchClass, chord_type: ChordType) -> Self {
        Self { root: root % 12, chord_type, bass: None, extensions: Vec::new(), lcc_level: 1 }
    }

    /// Crée un accord avec basse différente (slash chord)
    #[must_use]
    pub const fn with_bass(mut self, bass: PitchClass) -> Self {
        self.bass = Some(bass % 12);
        self
    }

    /// Ajoute une extension (9, 11, 13 en demi-tons depuis root)
    #[must_use]
    pub fn with_extension(mut self, interval: u8) -> Self {
        self.extensions.push(interval % 12);
        self
    }

    /// Définit le niveau LCC
    #[must_use]
    pub fn with_lcc_level(mut self, level: u8) -> Self {
        self.lcc_level = level.clamp(1, 12);
        self
    }

    /// Retourne les pitch classes de l'accord (sans extensions)
    #[must_use]
    pub fn pitch_classes(&self) -> Vec<PitchClass> {
        self.chord_type.intervals().iter().map(|&interval| (self.root + interval) % 12).collect()
    }

    /// Retourne toutes les pitch classes (avec extensions)
    #[must_use]
    pub fn all_pitch_classes(&self) -> Vec<PitchClass> {
        let mut pcs = self.pitch_classes();
        for &ext in &self.extensions {
            let pc = (self.root + ext) % 12;
            if !pcs.contains(&pc) {
                pcs.push(pc);
            }
        }
        pcs
    }

    /// Vérifie si l'accord est ambigu (utile pour pivots)
    #[must_use]
    pub const fn is_ambiguous(&self) -> bool {
        self.chord_type.is_ambiguous()
    }

    /// Calcule la distance de voice-leading vers un autre accord
    /// (somme des mouvements minimaux en demi-tons)
    #[must_use]
    pub fn voice_leading_distance(&self, other: &Self) -> u32 {
        let self_pcs = self.pitch_classes();
        let other_pcs = other.pitch_classes();

        // Algorithme glouton simplifié: pour chaque note de self,
        // trouver la note la plus proche dans other
        let mut total_distance = 0u32;
        let mut used = vec![false; other_pcs.len()];

        for &self_pc in &self_pcs {
            let mut min_dist = 12u32;
            let mut min_idx = 0;

            for (i, &other_pc) in other_pcs.iter().enumerate() {
                if used[i] {
                    continue;
                }
                // Distance minimale sur le cercle des pitch classes
                let dist = {
                    let d1 = (i32::from(other_pc) - i32::from(self_pc)).unsigned_abs();
                    let d2 = 12 - d1;
                    d1.min(d2)
                };
                if dist < min_dist {
                    min_dist = dist;
                    min_idx = i;
                }
            }

            if min_dist < 12 && min_idx < used.len() {
                used[min_idx] = true;
                total_distance += min_dist;
            }
        }

        total_distance
    }

    /// Parse a chord from its name (e.g., "Cmaj7", "Dm7b5", "Bb7", "F#m/C#").
    ///
    /// This is the inverse of [`Chord::name()`]. Input is normalized to sharps
    /// (e.g., "Bb" → root 10 which displays as "A#").
    pub fn from_name(s: &str) -> Result<Self, ChordParseError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ChordParseError::EmptyInput);
        }

        // Split on '/' for slash chords — but only the LAST '/' counts as bass separator
        let (chord_part, bass_part) = if let Some(slash_pos) = s.rfind('/') {
            // Only treat as slash chord if there's content after the slash
            let after = &s[slash_pos + 1..];
            if after.is_empty() { (s, None) } else { (&s[..slash_pos], Some(after)) }
        } else {
            (s, None)
        };

        let (root, remainder) = parse_root(chord_part).ok_or(ChordParseError::InvalidRoot)?;

        let (chord_type, leftover) = parse_chord_type(remainder)
            .ok_or_else(|| ChordParseError::InvalidQuality(remainder.to_string()))?;

        if !leftover.is_empty() {
            return Err(ChordParseError::InvalidQuality(remainder.to_string()));
        }

        let mut chord = Self::new(root, chord_type);

        if let Some(bass_str) = bass_part {
            let (bass_pc, bass_leftover) = parse_root(bass_str)
                .ok_or_else(|| ChordParseError::InvalidBass(bass_str.to_string()))?;
            if !bass_leftover.is_empty() {
                return Err(ChordParseError::InvalidBass(bass_str.to_string()));
            }
            chord = chord.with_bass(bass_pc);
        }

        Ok(chord)
    }

    /// Nom complet de l'accord (ex: "C#m7")
    #[must_use]
    pub fn name(&self) -> String {
        let root_name = NoteName::from_pitch_class(self.root).as_str();
        let suffix = self.chord_type.suffix();
        let bass_str = self
            .bass
            .map_or(String::new(), |b| format!("/{}", NoteName::from_pitch_class(b).as_str()));
        format!("{root_name}{suffix}{bass_str}")
    }

    /// Identifie un accord à partir d'un ensemble de pitch classes.
    /// Retourne None si aucun type d'accord valide ne correspond.
    ///
    /// Cette méthode est utilisée par le `ParsimoniousDriver` pour valider
    /// les accords candidats générés par les mouvements de voix.
    #[must_use]
    pub fn identify(pitch_classes: &[PitchClass]) -> Option<Self> {
        if pitch_classes.is_empty() {
            return None;
        }

        // Normaliser et trier les pitch classes
        let mut pcs: Vec<PitchClass> = pitch_classes.iter().map(|&pc| pc % 12).collect();
        pcs.sort_unstable();
        pcs.dedup();

        // Essayer chaque note comme potentielle fondamentale
        for &potential_root in &pcs {
            // Calculer les intervalles depuis cette fondamentale
            let mut intervals: Vec<u8> =
                pcs.iter().map(|&pc| (pc + 12 - potential_root) % 12).collect();
            intervals.sort_unstable();

            // Essayer de matcher contre les types d'accords connus
            if let Some(chord_type) = Self::match_intervals(&intervals) {
                return Some(Self::new(potential_root, chord_type));
            }
        }

        None
    }

    /// Matche un ensemble d'intervalles à un `ChordType`
    fn match_intervals(intervals: &[u8]) -> Option<ChordType> {
        // Définir tous les patterns d'accords (ensembles d'intervalles)
        // Ordre important: plus spécifiques d'abord pour éviter les faux positifs
        let patterns: &[(&[u8], ChordType)] = &[
            // Triades
            (&[0, 4, 7], ChordType::Major),
            (&[0, 3, 7], ChordType::Minor),
            (&[0, 4, 8], ChordType::Augmented),
            (&[0, 3, 6], ChordType::Diminished),
            (&[0, 2, 7], ChordType::Sus2),
            (&[0, 5, 7], ChordType::Sus4),
            // Tétracordes (7èmes et 6èmes)
            (&[0, 4, 7, 10], ChordType::Dominant7),
            (&[0, 4, 7, 11], ChordType::Major7),
            (&[0, 3, 7, 10], ChordType::Minor7),
            (&[0, 3, 6, 10], ChordType::HalfDiminished),
            (&[0, 3, 6, 9], ChordType::Diminished7),
            (&[0, 3, 7, 11], ChordType::MinorMajor7),
            (&[0, 4, 8, 10], ChordType::Augmented7),
            (&[0, 4, 7, 9], ChordType::Major6),
            (&[0, 3, 7, 9], ChordType::Minor6),
            (&[0, 5, 7, 10], ChordType::Dominant7Sus4),
            (&[0, 2, 4, 7], ChordType::Add9),
        ];

        for (pattern, chord_type) in patterns {
            if intervals == *pattern {
                return Some(*chord_type);
            }
        }

        None
    }

    /// Return a new chord transposed by the given number of semitones.
    /// Root and bass (if present) are shifted; chord_type and extensions are preserved.
    #[must_use]
    pub fn transposed(&self, semitones: i32) -> Self {
        Self {
            root: ((i32::from(self.root) + semitones).rem_euclid(12)) as u8,
            chord_type: self.chord_type,
            bass: self.bass.map(|b| ((i32::from(b) + semitones).rem_euclid(12)) as u8),
            extensions: self.extensions.clone(),
            lcc_level: self.lcc_level,
        }
    }
}

/// Transpose a chord name string by the given semitones.
/// Returns the transposed name, or the original if parsing fails.
pub fn transpose_chord_name(name: &str, semitones: i32) -> String {
    match Chord::from_name(name) {
        Ok(chord) => chord.transposed(semitones).name(),
        Err(_) => name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_classes() {
        // C Major = C, E, G = 0, 4, 7
        let c_maj = Chord::new(0, ChordType::Major);
        assert_eq!(c_maj.pitch_classes(), vec![0, 4, 7]);

        // A Minor = A, C, E = 9, 0, 4
        let a_min = Chord::new(9, ChordType::Minor);
        assert_eq!(a_min.pitch_classes(), vec![9, 0, 4]);

        // G Dominant7 = G, B, D, F = 7, 11, 2, 5
        let g7 = Chord::new(7, ChordType::Dominant7);
        assert_eq!(g7.pitch_classes(), vec![7, 11, 2, 5]);
    }

    #[test]
    fn test_chord_name() {
        assert_eq!(Chord::new(0, ChordType::Major).name(), "C");
        assert_eq!(Chord::new(0, ChordType::Minor).name(), "Cm");
        assert_eq!(Chord::new(1, ChordType::Dominant7).name(), "C#7");
        assert_eq!(Chord::new(4, ChordType::Minor7).name(), "Em7");
        assert_eq!(Chord::new(0, ChordType::Major).with_bass(7).name(), "C/G");
    }

    #[test]
    fn test_voice_leading_distance() {
        // C Major -> A Minor (relatif): C,E,G -> A,C,E
        // C->C (0), E->E (0), G->A (2) = total 2
        let c_maj = Chord::new(0, ChordType::Major);
        let a_min = Chord::new(9, ChordType::Minor);
        assert_eq!(c_maj.voice_leading_distance(&a_min), 2);

        // C Major -> C Minor (parallèle): C,E,G -> C,Eb,G
        // C->C (0), E->Eb (1), G->G (0) = total 1
        let c_min = Chord::new(0, ChordType::Minor);
        assert_eq!(c_maj.voice_leading_distance(&c_min), 1);
    }

    #[test]
    fn test_symmetric_chords() {
        assert!(ChordType::Augmented.is_symmetric());
        assert!(ChordType::Diminished7.is_symmetric());
        assert!(!ChordType::Major.is_symmetric());
        assert!(!ChordType::Minor.is_symmetric());
    }

    #[test]
    fn test_ambiguous_chords() {
        assert!(ChordType::Sus4.is_ambiguous());
        assert!(ChordType::Augmented.is_ambiguous());
        assert!(ChordType::Diminished7.is_ambiguous());
        assert!(!ChordType::Major.is_ambiguous());
    }

    // ── from_name() tests ──

    #[test]
    fn test_from_name_round_trip_all_types() {
        // Round-trip: name() → from_name() should preserve root and chord_type
        let types = [
            ChordType::Major,
            ChordType::Minor,
            ChordType::Augmented,
            ChordType::Diminished,
            ChordType::Dominant7,
            ChordType::Major7,
            ChordType::Minor7,
            ChordType::HalfDiminished,
            ChordType::Diminished7,
            ChordType::Sus2,
            ChordType::Sus4,
            ChordType::MinorMajor7,
            ChordType::Augmented7,
            ChordType::Major6,
            ChordType::Minor6,
            ChordType::Dominant7Sus4,
            ChordType::Add9,
        ];
        for root in 0..12u8 {
            for &ct in &types {
                let original = Chord::new(root, ct);
                let name = original.name();
                let parsed = Chord::from_name(&name).unwrap_or_else(|e| {
                    panic!("Failed to parse \"{name}\" (root={root}, type={ct:?}): {e}")
                });
                assert_eq!(parsed.root, root, "root mismatch for \"{name}\"");
                assert_eq!(parsed.chord_type, ct, "type mismatch for \"{name}\"");
            }
        }
    }

    #[test]
    fn test_from_name_flat_notation() {
        // Flats should parse correctly (normalized to sharp internally)
        let cases = [
            ("Db", 1, ChordType::Major),
            ("Eb", 3, ChordType::Major),
            ("Gb", 6, ChordType::Major),
            ("Ab", 8, ChordType::Major),
            ("Bb", 10, ChordType::Major),
            ("Bbm7", 10, ChordType::Minor7),
            ("Ebmaj7", 3, ChordType::Major7),
            ("Abm7b5", 8, ChordType::HalfDiminished),
            ("Gb7", 6, ChordType::Dominant7),
        ];
        for (input, expected_root, expected_type) in cases {
            let parsed = Chord::from_name(input)
                .unwrap_or_else(|e| panic!("Failed to parse \"{input}\": {e}"));
            assert_eq!(parsed.root, expected_root, "root for \"{input}\"");
            assert_eq!(parsed.chord_type, expected_type, "type for \"{input}\"");
        }
    }

    #[test]
    fn test_from_name_slash_chords() {
        let parsed = Chord::from_name("Dm7/A").unwrap();
        assert_eq!(parsed.root, 2);
        assert_eq!(parsed.chord_type, ChordType::Minor7);
        assert_eq!(parsed.bass, Some(9)); // A = 9

        let parsed = Chord::from_name("C/G").unwrap();
        assert_eq!(parsed.root, 0);
        assert_eq!(parsed.chord_type, ChordType::Major);
        assert_eq!(parsed.bass, Some(7));

        let parsed = Chord::from_name("F#m7/C#").unwrap();
        assert_eq!(parsed.root, 6);
        assert_eq!(parsed.chord_type, ChordType::Minor7);
        assert_eq!(parsed.bass, Some(1));

        let parsed = Chord::from_name("Bbmaj7/D").unwrap();
        assert_eq!(parsed.root, 10);
        assert_eq!(parsed.chord_type, ChordType::Major7);
        assert_eq!(parsed.bass, Some(2));
    }

    #[test]
    fn test_from_name_errors() {
        assert_eq!(Chord::from_name(""), Err(ChordParseError::EmptyInput));
        assert_eq!(Chord::from_name("   "), Err(ChordParseError::EmptyInput));
        assert!(matches!(Chord::from_name("X7"), Err(ChordParseError::InvalidRoot)));
        assert!(matches!(Chord::from_name("Cblah"), Err(ChordParseError::InvalidQuality(_))));
        assert!(matches!(Chord::from_name("C/X"), Err(ChordParseError::InvalidBass(_))));
    }

    #[test]
    fn test_from_name_b_root_disambiguation() {
        // "B" alone = B Major (root 11)
        let parsed = Chord::from_name("B").unwrap();
        assert_eq!(parsed.root, 11);
        assert_eq!(parsed.chord_type, ChordType::Major);

        // "Bm" = B Minor
        let parsed = Chord::from_name("Bm").unwrap();
        assert_eq!(parsed.root, 11);
        assert_eq!(parsed.chord_type, ChordType::Minor);

        // "Bb" = Bb Major (root 10)
        let parsed = Chord::from_name("Bb").unwrap();
        assert_eq!(parsed.root, 10);
        assert_eq!(parsed.chord_type, ChordType::Major);

        // "Bbm" = Bb Minor
        let parsed = Chord::from_name("Bbm").unwrap();
        assert_eq!(parsed.root, 10);
        assert_eq!(parsed.chord_type, ChordType::Minor);

        // "Bdim" = B Diminished
        let parsed = Chord::from_name("Bdim").unwrap();
        assert_eq!(parsed.root, 11);
        assert_eq!(parsed.chord_type, ChordType::Diminished);

        // "Bb7" = Bb Dominant7
        let parsed = Chord::from_name("Bb7").unwrap();
        assert_eq!(parsed.root, 10);
        assert_eq!(parsed.chord_type, ChordType::Dominant7);

        // "Bmaj7" = B Major7
        let parsed = Chord::from_name("Bmaj7").unwrap();
        assert_eq!(parsed.root, 11);
        assert_eq!(parsed.chord_type, ChordType::Major7);

        // "Bbmaj7" = Bb Major7
        let parsed = Chord::from_name("Bbmaj7").unwrap();
        assert_eq!(parsed.root, 10);
        assert_eq!(parsed.chord_type, ChordType::Major7);
    }

    #[test]
    fn test_from_name_whitespace_trimmed() {
        let parsed = Chord::from_name("  Cm7  ").unwrap();
        assert_eq!(parsed.root, 0);
        assert_eq!(parsed.chord_type, ChordType::Minor7);
    }

    #[test]
    fn test_transposed_up_2_semitones() {
        let bb7 = Chord::from_name("Bb7").unwrap();
        let transposed = bb7.transposed(2);
        assert_eq!(transposed.root, 0); // Bb(10) + 2 = C(0)
        assert_eq!(transposed.chord_type, ChordType::Dominant7);
        assert_eq!(transposed.name(), "C7");
    }

    #[test]
    fn test_transposed_wraps_around() {
        let b7 = Chord::from_name("B7").unwrap();
        let transposed = b7.transposed(2);
        assert_eq!(transposed.root, 1); // B(11) + 2 = C#(1)
        assert_eq!(transposed.name(), "C#7");
    }

    #[test]
    fn test_transposed_negative() {
        let c7 = Chord::from_name("C7").unwrap();
        let transposed = c7.transposed(-2);
        assert_eq!(transposed.root, 10); // C(0) - 2 = Bb(10)
        assert_eq!(transposed.name(), "A#7"); // renders as sharp
    }

    #[test]
    fn test_transposed_zero_is_identity() {
        let dm7 = Chord::from_name("Dm7").unwrap();
        let transposed = dm7.transposed(0);
        assert_eq!(transposed.root, dm7.root);
        assert_eq!(transposed.chord_type, dm7.chord_type);
    }

    #[test]
    fn test_transposed_slash_chord() {
        let slash = Chord::from_name("C/G").unwrap();
        let transposed = slash.transposed(2);
        assert_eq!(transposed.root, 2); // C(0) + 2 = D
        assert_eq!(transposed.bass, Some(9)); // G(7) + 2 = A
    }

    #[test]
    fn test_transposed_round_trip_12() {
        let chord = Chord::from_name("Ebmaj7").unwrap();
        let round_trip = chord.transposed(12);
        assert_eq!(round_trip.root, chord.root);
        assert_eq!(round_trip.chord_type, chord.chord_type);
    }

    #[test]
    fn test_transpose_chord_name_valid() {
        assert_eq!(transpose_chord_name("Bb7", 2), "C7");
        assert_eq!(transpose_chord_name("Dm7", 5), "Gm7");
    }

    #[test]
    fn test_transpose_chord_name_invalid() {
        assert_eq!(transpose_chord_name("XYZ", 2), "XYZ");
    }
}

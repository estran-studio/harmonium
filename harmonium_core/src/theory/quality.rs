//! Qualité d'accord : un type de base et des tensions, avec **un seul** texte de suffixe.
//!
//! Le parseur de symboles (`Chord::from_name`), l'affichage (`Chord::name`) et les profils
//! `loop` (`quality = "m7"`) passent tous par ce module (principe IV). Le suffixe d'un type
//! nu vient de [`ChordType::suffix`] ; ce module ajoute les piles naturelles (`9`, `11`, `13`,
//! `maj9`, `m11`, `6/9`…) et les altérations (`b9`, `#9`, `#11`, `b13`, `b5`, `#5`, `alt`).
//!
//! Grammaire : `tête altération*`.
//!
//! - **Tête** : un suffixe de [`ChordType`] (`""`, `m`, `maj7`, `7sus4`…), ou une pile
//!   naturelle (`9`, `11`, `13`, `maj9`, `maj13`, `m9`, `m11`, `mMaj9`, `6/9`, `m6/9`,
//!   `m(add9)`, `9sus4`, `13sus4`), ou `7alt`, ou un synonyme (`maj`, `aug`, `add2`,
//!   `m(b5)`, `sus`). La plus longue tête gagne.
//! - **Altération** (sur une tête dominante `7`, `9`, `11`, `13`, `7b5`, `7#5` seulement) :
//!   `b9`, `#9`, `#11`, `b13`, dans n'importe quel ordre, sans doublon ni conflit avec la
//!   pile naturelle (`9b9`, `13b13` refusés) ni avec la quinte. `b5` et `#5` ne s'écrivent
//!   qu'après la tête `7` seule et la convertissent en `7b5` / `7#5`.
//!
//! L'affichage est canonique : tête, puis altérations par intervalle croissant. `parse` d'un
//! affichage redonne la même qualité.

use arrayvec::ArrayVec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::chord::ChordType;

/// Intervalle (demi-tons) → bit du masque de tensions.
const fn bit(interval: u8) -> u16 {
    1 << interval
}

const FLAT9: u16 = bit(1);
const NINE: u16 = bit(2);
const SHARP9: u16 = bit(3);
const ELEVEN: u16 = bit(5);
const SHARP11: u16 = bit(6);
const FLAT13: u16 = bit(8);
const THIRTEEN: u16 = bit(9);
/// Altérations d'un accord dominant, dans l'ordre d'affichage.
const ALTERATIONS: [(&str, u16); 4] =
    [("b9", FLAT9), ("#9", SHARP9), ("#11", SHARP11), ("b13", FLAT13)];
const ALTERED_BITS: u16 = FLAT9 | SHARP9 | SHARP11 | FLAT13;

/// Têtes à pile naturelle ou nommée : (texte, type, tensions). Le type nu reste dans
/// [`ChordType::suffix`].
const NAMED_HEADS: [(&str, ChordType, u16); 14] = [
    ("maj13", ChordType::Major7, NINE | THIRTEEN),
    ("maj9", ChordType::Major7, NINE),
    ("m11", ChordType::Minor7, NINE | ELEVEN),
    ("m9", ChordType::Minor7, NINE),
    ("mMaj9", ChordType::MinorMajor7, NINE),
    ("6/9", ChordType::Major6, NINE),
    ("m6/9", ChordType::Minor6, NINE),
    ("m(add9)", ChordType::Minor, NINE),
    ("13sus4", ChordType::Dominant7Sus4, NINE | THIRTEEN),
    ("9sus4", ChordType::Dominant7Sus4, NINE),
    ("13", ChordType::Dominant7, NINE | THIRTEEN),
    ("11", ChordType::Dominant7, NINE | ELEVEN),
    ("9", ChordType::Dominant7, NINE),
    ("7alt", ChordType::Altered, ALTERED_BITS),
];

/// Synonymes acceptés en lecture seulement.
const SYNONYMS: [(&str, ChordType); 5] = [
    ("maj", ChordType::Major),
    ("aug", ChordType::Augmented),
    ("add2", ChordType::Add9),
    ("m(b5)", ChordType::Diminished),
    ("sus", ChordType::Sus4),
];

/// Tensions imposées par le type lui-même (l'accord altéré porte toujours ses altérations).
const fn implied(chord_type: ChordType) -> u16 {
    match chord_type {
        ChordType::Altered => ALTERED_BITS,
        _ => 0,
    }
}

const fn takes_alterations(chord_type: ChordType) -> bool {
    matches!(chord_type, ChordType::Dominant7 | ChordType::Dominant7Flat5 | ChordType::Augmented7)
}

/// Symbole de qualité refusé : le texte fautif et la raison.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualityError {
    pub suffix: String,
    pub reason: &'static str,
}

impl std::fmt::Display for QualityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "qualité d'accord « {} » refusée : {}", self.suffix, self.reason)
    }
}

impl std::error::Error for QualityError {}

/// Type de base et tensions (piles naturelles et altérations) d'un accord.
///
/// `Copy` et sans allocation : c'est la forme que portent les accords du moteur.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Quality {
    chord_type: ChordType,
    /// Bit `i` : intervalle `i` demi-tons au-dessus de la fondamentale, ajouté au type.
    tensions: u16,
}

impl Quality {
    /// Le type seul, sans tension (sauf celles qu'il impose : `7alt`).
    #[must_use]
    pub const fn plain(chord_type: ChordType) -> Self {
        Self { chord_type, tensions: implied(chord_type) }
    }

    /// Type et tensions (intervalles en demi-tons) ; `None` hors du vocabulaire
    /// (voir la grammaire du module) ou si une tension double une note du type.
    #[must_use]
    pub fn from_parts(chord_type: ChordType, tensions: &[u8]) -> Option<Self> {
        let mut mask = implied(chord_type);
        for &interval in tensions {
            if !(1..12).contains(&interval) {
                return None;
            }
            mask |= bit(interval);
        }
        let quality = Self { chord_type, tensions: mask };
        (!quality.clashes() && quality.text().is_some()).then_some(quality)
    }

    #[must_use]
    pub const fn chord_type(self) -> ChordType {
        self.chord_type
    }

    /// Tensions ajoutées au type, par intervalle croissant (9 = 2, #11 = 6, 13 = 9…).
    #[must_use]
    pub fn extensions(self) -> ArrayVec<u8, 8> {
        (1..12u8).filter(|&i| self.tensions & bit(i) != 0).collect()
    }

    /// Classes de hauteur relatives de l'accord entier (type et tensions), croissantes.
    #[must_use]
    pub fn intervals(self) -> ArrayVec<u8, 12> {
        let owned = self.chord_type.intervals();
        (0..12u8).filter(|&i| owned.contains(&i) || self.tensions & bit(i) != 0).collect()
    }

    /// Hauteurs d'un voicing, empilées par tierces au-dessus de la fondamentale (les
    /// tensions à l'octave supérieure : 9 = 14, 13 = 21), au plus `limit.min(5)`.
    ///
    /// Au-delà, la quinte tombe d'abord (elle porte le moins), puis les tensions les plus
    /// hautes. Un type sans tension garde exactement ses intervalles.
    #[must_use]
    pub fn voicing_tones(self, limit: usize) -> ArrayVec<u8, 5> {
        let limit = limit.min(5);
        let mut tones: ArrayVec<u8, 16> = self.chord_type.intervals().iter().copied().collect();
        for interval in 1..12u8 {
            if self.tensions & bit(interval) != 0 {
                tones.push(interval + 12);
            }
        }
        while tones.len() > limit {
            match tones.iter().position(|t| (6..=8).contains(t)) {
                Some(fifth) => {
                    tones.remove(fifth);
                }
                None => {
                    tones.pop();
                }
            }
        }
        tones.into_iter().collect()
    }

    /// Une tension double une note du type : combinaison hors vocabulaire.
    fn clashes(self) -> bool {
        self.chord_type.intervals().iter().any(|&i| self.tensions & bit(i) != 0)
    }

    /// Suffixe canonique ; `None` si la combinaison n'appartient pas au vocabulaire.
    fn text(self) -> Option<String> {
        if let Some((text, ..)) =
            NAMED_HEADS.iter().find(|(_, t, m)| *t == self.chord_type && *m == self.tensions)
        {
            return Some((*text).to_string());
        }
        if self.tensions == implied(self.chord_type) {
            return Some(self.chord_type.suffix().to_string());
        }
        if !takes_alterations(self.chord_type) {
            return None;
        }
        let natural = self.tensions & !ALTERED_BITS;
        let head = if natural == 0 {
            self.chord_type.suffix()
        } else {
            NAMED_HEADS
                .iter()
                .find(|(_, t, m)| *t == ChordType::Dominant7 && *m == natural)
                .map(|(text, ..)| *text)
                .filter(|_| self.chord_type == ChordType::Dominant7)?
        };
        let mut text = head.to_string();
        for (token, mask) in ALTERATIONS {
            if self.tensions & mask != 0 {
                text.push_str(token);
            }
        }
        Some(text)
    }

    /// Lit un suffixe de qualité (`""`, `m7`, `7b9`, `13`, `7alt`, `m7b5`, `6/9`…).
    ///
    /// # Errors
    /// Suffixe hors de la grammaire du module, doublon ou conflit de tensions.
    pub fn parse(suffix: &str) -> Result<Self, QualityError> {
        let fail = |reason| QualityError { suffix: suffix.to_string(), reason };
        let (head_len, chord_type, mut tensions) = Self::head(suffix);
        let mut chord_type = chord_type;
        let mut rest = &suffix[head_len..];
        let natural_free = tensions == 0;
        let mut converted = false;
        while !rest.is_empty() {
            if !takes_alterations(chord_type) {
                return Err(fail("suffixe inconnu"));
            }
            if let Some(tail) = rest.strip_prefix("b5").or_else(|| rest.strip_prefix("#5")) {
                if chord_type != ChordType::Dominant7 || !natural_free || converted {
                    return Err(fail("b5 et #5 ne suivent que la tête 7 seule, une fois"));
                }
                chord_type = if rest.starts_with("b5") {
                    ChordType::Dominant7Flat5
                } else {
                    ChordType::Augmented7
                };
                converted = true;
                rest = tail;
                continue;
            }
            let Some((token, mask)) = ALTERATIONS.iter().find(|(t, _)| rest.starts_with(t)) else {
                return Err(fail("suffixe inconnu"));
            };
            if tensions & mask != 0 {
                return Err(fail("altération en double"));
            }
            tensions |= mask;
            rest = &rest[token.len()..];
        }
        // Une altération ne double ni la pile naturelle (9b9, 11#11, 13b13) ni la quinte.
        let natural = tensions & !ALTERED_BITS;
        let conflicting = (natural & NINE != 0 && tensions & (FLAT9 | SHARP9) != 0)
            || (natural & ELEVEN != 0 && tensions & SHARP11 != 0)
            || (natural & THIRTEEN != 0 && tensions & FLAT13 != 0);
        let quality = Self { chord_type, tensions };
        if conflicting || quality.clashes() {
            return Err(fail("altération en conflit avec la pile ou la quinte"));
        }
        Ok(quality)
    }

    /// Plus longue tête reconnue au début de `s` : (longueur, type, tensions).
    fn head(s: &str) -> (usize, ChordType, u16) {
        let types = ChordType::ALL.iter().map(|&t| (t.suffix(), t, implied(t)));
        let named = NAMED_HEADS.iter().copied();
        let synonyms = SYNONYMS.iter().map(|&(text, t)| (text, t, 0));
        types
            .chain(named)
            .chain(synonyms)
            .filter(|(text, ..)| s.starts_with(text))
            .max_by_key(|(text, ..)| text.len())
            .map_or((0, ChordType::Major, 0), |(text, t, mask)| (text.len(), t, mask))
    }
}

impl std::fmt::Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Toute qualité construite par `parse`, `plain` ou `from_parts` a un texte.
        f.write_str(&self.text().unwrap_or_else(|| self.chord_type.suffix().to_string()))
    }
}

impl std::str::FromStr for Quality {
    type Err = QualityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for Quality {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Quality {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vocabulaire iReal tel que le convertit `harmonium_practice::ireal::map_quality`,
    /// plus les suffixes propres du moteur : chacun doit être lu.
    const VOCABULARY: [&str; 48] = [
        "", "maj", "maj7", "maj9", "maj13", "6", "6/9", "add2", "add9", "m", "m7", "m9", "m11",
        "m6", "m6/9", "mMaj7", "mMaj9", "m(b5)", "m(add9)", "7", "9", "11", "13", "7b5", "7#5",
        "7b9", "7#9", "7#11", "7b13", "7b9b5", "7b9#5", "7#9b5", "7#9#5", "7alt", "m7b5", "dim",
        "dim7", "aug", "+", "sus", "sus4", "sus2", "7sus4", "7sus2", "9sus4", "13sus4", "9#11",
        "7#9#11",
    ];

    type TestResult = Result<(), QualityError>;

    fn parsed(text: &str) -> Result<(ChordType, Vec<u8>), QualityError> {
        let quality = Quality::parse(text)?;
        Ok((quality.chord_type(), quality.extensions().to_vec()))
    }

    #[test]
    fn tout_le_vocabulaire_est_lu() {
        for text in VOCABULARY {
            assert!(Quality::parse(text).is_ok(), "« {text} » devrait être lu");
        }
    }

    #[test]
    fn types_tensions_et_alterations() -> TestResult {
        use ChordType::*;
        let cases: [(&str, ChordType, &[u8]); 24] = [
            ("", Major, &[]),
            ("m7", Minor7, &[]),
            ("maj7", Major7, &[]),
            ("6", Major6, &[]),
            ("dim7", Diminished7, &[]),
            ("m7b5", HalfDiminished, &[]),
            ("sus", Sus4, &[]),
            ("aug", Augmented, &[]),
            ("add2", Add9, &[]),
            ("m(b5)", Diminished, &[]),
            ("9", Dominant7, &[2]),
            ("11", Dominant7, &[2, 5]),
            ("13", Dominant7, &[2, 9]),
            ("maj9", Major7, &[2]),
            ("6/9", Major6, &[2]),
            ("m11", Minor7, &[2, 5]),
            ("m(add9)", Minor, &[2]),
            ("7b9", Dominant7, &[1]),
            ("7#9#11", Dominant7, &[3, 6]),
            ("9#11", Dominant7, &[2, 6]),
            ("7b9b5", Dominant7Flat5, &[1]),
            ("7#9#5", Augmented7, &[3]),
            ("7alt", Altered, &[1, 3, 6, 8]),
            ("13sus4", Dominant7Sus4, &[2, 9]),
        ];
        for (text, chord_type, extensions) in cases {
            assert_eq!(parsed(text)?, (chord_type, extensions.to_vec()), "{text}");
        }
        Ok(())
    }

    #[test]
    fn l_affichage_est_canonique_et_relisible() -> TestResult {
        for text in VOCABULARY {
            let quality = Quality::parse(text)?;
            assert_eq!(Quality::parse(&quality.to_string()), Ok(quality), "{text}");
        }
        for &chord_type in &ChordType::ALL {
            let quality = Quality::plain(chord_type);
            assert_eq!(Quality::parse(&quality.to_string()), Ok(quality), "{chord_type:?}");
        }
        // L'ordre d'écriture des altérations ne compte pas ; l'affichage les trie.
        assert_eq!(Quality::parse("7b9b5"), Quality::parse("7b5b9"));
        assert_eq!(Quality::parse("7b9b5").map(|q| q.to_string()), Ok("7b5b9".to_string()));
        assert_eq!(Quality::parse("7#11b9").map(|q| q.to_string()), Ok("7b9#11".to_string()));
        // Synonymes lus, jamais écrits.
        assert_eq!(Quality::parse("maj").map(|q| q.to_string()), Ok(String::new()));
        assert_eq!(Quality::parse("aug").map(|q| q.to_string()), Ok("+".to_string()));
        Ok(())
    }

    #[test]
    fn les_symboles_invalides_sont_refuses() {
        for text in [
            "Major", "Minor", "x", "H7", "69", "7 b9", "7/9", "maj7b9", "m7b9", "m7#11", "7b9b9",
            "7#9#9", "9b9", "9#9", "11#11", "13b13", "7b5b5", "7b5#5", "7#5b5", "9b5", "7#5b13",
            "7#11b5", "7alt9", "7altb9", "sus9", "6/9b9", "add9b9", "b9", "#11", "7b", "7#",
            "7b10", "m11b9", "mMaj13",
        ] {
            assert!(Quality::parse(text).is_err(), "« {text} » devrait être refusé");
        }
        let err = Quality::parse("7b9b9").err();
        assert_eq!(err.map(|e| e.reason), Some("altération en double"));
    }

    #[test]
    fn voicings_empiles_sans_depasser_cinq_notes() -> TestResult {
        let tones = |text: &str, limit| {
            Quality::parse(text).map(|q| q.voicing_tones(limit).to_vec()).unwrap_or_default()
        };
        assert_eq!(tones("", 5), [0, 4, 7]);
        assert_eq!(tones("m7", 5), [0, 3, 7, 10]);
        assert_eq!(tones("9", 5), [0, 4, 7, 10, 14]);
        assert_eq!(tones("maj9", 5), [0, 4, 7, 11, 14]);
        assert_eq!(tones("7b9", 5), [0, 4, 7, 10, 13]);
        // La quinte tombe d'abord, puis les tensions les plus hautes.
        assert_eq!(tones("13", 5), [0, 4, 10, 14, 21]);
        assert_eq!(tones("m11", 5), [0, 3, 10, 14, 17]);
        assert_eq!(tones("13", 4), [0, 4, 10, 14]);
        assert_eq!(tones("7alt", 5), [0, 4, 10, 13, 15]);
        assert_eq!(tones("7alt", 4), [0, 4, 10, 13]);
        for text in VOCABULARY {
            for limit in [4, 5] {
                let quality = Quality::parse(text)?;
                let voiced = quality.voicing_tones(limit);
                assert!(voiced.len() <= limit.max(quality.chord_type().intervals().len()));
                assert!(voiced.windows(2).all(|w| w[0] < w[1]), "{text} : {voiced:?}");
            }
        }
        Ok(())
    }

    #[test]
    fn intervalles_de_l_accord_entier() {
        let all = |text: &str| Quality::parse(text).map(|q| q.intervals().to_vec());
        assert_eq!(all("7b9"), Ok(vec![0, 1, 4, 7, 10]));
        assert_eq!(all("7alt"), Ok(vec![0, 1, 3, 4, 6, 8, 10]));
        assert_eq!(all("13"), Ok(vec![0, 2, 4, 7, 9, 10]));
    }

    #[test]
    fn from_parts_refuse_les_combinaisons_hors_vocabulaire() {
        assert!(Quality::from_parts(ChordType::Dominant7, &[1]).is_some());
        assert!(Quality::from_parts(ChordType::Dominant7, &[4]).is_none(), "double la tierce");
        assert!(Quality::from_parts(ChordType::Dominant7, &[5]).is_none(), "11 sans 9");
        assert!(Quality::from_parts(ChordType::Minor7, &[1]).is_none());
        assert!(Quality::from_parts(ChordType::Major, &[12]).is_none());
        assert_eq!(
            Quality::from_parts(ChordType::Altered, &[]),
            Some(Quality::plain(ChordType::Altered))
        );
    }
}

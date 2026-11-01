//! Voicings d'accompagnement en notes guides, sans fondamentale et drop 2.
//!
//! Chaque voicing ne contient que des notes de l'accord publié (tensions comprises) :
//! la basse d'un accord slash n'y est jamais ajoutée, la piste de basse la joue. Une pédale
//! n'a pas de qualité typée : elle garde les voicings serrés (`None` ici).
//!
//! Les formes sont des listes croissantes d'intervalles relatifs à la fondamentale
//! (négatifs possibles pour le drop 2) ; `candidates` les place dans toutes les octaves
//! MIDI. Le choix entre candidats (fenêtre, conduite de voix) revient à `ChordPlayer`.

use arrayvec::ArrayVec;

use super::{
    accomp_profile::ChordVoicing,
    harmony::{ChordQuality, DegreeChord},
    mode::Mode,
};
use crate::theory::{PitchClass, chord::ChordType, quality::Quality};

/// Voicing d'un accord : au plus 5 hauteurs, sur la pile.
pub(crate) type Voicing = ArrayVec<u8, 5>;

/// Une forme : intervalles croissants au-dessus de la fondamentale.
type Shape = ArrayVec<i8, 5>;

/// Tension préférée d'abord : 9, 13, #11, b9, #9, b13, 11 (demi-tons au-dessus de la
/// fondamentale).
const TENSION_PRIORITY: [u8; 7] = [2, 9, 6, 1, 3, 8, 5];

fn quality_of(chord: DegreeChord) -> Option<Quality> {
    match chord.quality {
        ChordQuality::Major => Some(Quality::plain(ChordType::Major)),
        ChordQuality::Minor => Some(Quality::plain(ChordType::Minor)),
        ChordQuality::Typed(quality) => Some(quality),
        ChordQuality::Pedal | ChordQuality::PedalNeighbor(_) => None,
    }
}

/// Tierce (ou seconde, quarte d'une suspension) et septième (ou sixte, ou quinte d'une
/// triade) du type : les deux voix qui définissent la sonorité.
fn guide_intervals(chord_type: ChordType) -> (u8, u8) {
    let tones = chord_type.intervals();
    let third = tones
        .iter()
        .copied()
        .find(|t| matches!(t, 3 | 4))
        .or_else(|| tones.iter().copied().find(|t| matches!(t, 2 | 5)))
        .unwrap_or(tones[1]);
    (third, tones[tones.len() - 1])
}

/// Tension de la qualité qui colore le voicing, empilée au-dessus de l'octave.
fn color_tension(quality: Quality) -> Option<u8> {
    let tensions = quality.extensions();
    TENSION_PRIORITY.into_iter().find(|t| tensions.contains(t)).map(|t| t + 12)
}

/// Toutes les inversions d'une pile croissante : la voix la plus grave monte d'une octave
/// à chaque rang.
fn inversions(tones: &[u8]) -> impl Iterator<Item = Shape> + '_ {
    (0..tones.len()).map(move |rank| {
        let mut shape: Shape = (0..tones.len())
            .map(|i| {
                let k = rank + i;
                (tones[k % tones.len()] + if k >= tones.len() { 12 } else { 0 }) as i8
            })
            .collect();
        shape.sort_unstable();
        shape
    })
}

fn guide_shapes(quality: Quality) -> ArrayVec<Shape, 5> {
    let (third, seventh) = guide_intervals(quality.chord_type());
    let mut tones: ArrayVec<u8, 3> = [third, seventh].into_iter().collect();
    tones.extend(color_tension(quality));
    tones.sort_unstable();
    inversions(&tones).collect()
}

/// Pile sans fondamentale : tierce, quinte, septième, tension de couleur. Type A : quatre
/// sons depuis la tierce ; type B : quatre sons depuis la septième (septième, tension,
/// tierce, quinte). Sans tension, la pile se prolonge à l'octave supérieure.
fn rootless_shapes(quality: Quality, a: bool, b: bool) -> ArrayVec<Shape, 5> {
    let chord_type = quality.chord_type();
    let (_, seventh) = guide_intervals(chord_type);
    let core: ArrayVec<u8, 4> =
        chord_type.intervals().iter().copied().filter(|&t| t != 0).collect();
    let mut stack: ArrayVec<u8, 8> = core.iter().copied().collect();
    stack.extend(color_tension(quality));
    stack.sort_unstable();
    let mut extended: ArrayVec<u8, 16> = stack.iter().copied().collect();
    extended.extend(stack.iter().map(|t| t + 12));
    extended.sort_unstable();
    let window = |start: usize| -> Option<Shape> {
        extended.get(start..start + 4).map(|w| w.iter().map(|&t| t as i8).collect())
    };
    let mut shapes = ArrayVec::new();
    if a || !has_seventh(chord_type) {
        shapes.extend(window(0));
    }
    if b && has_seventh(chord_type) {
        shapes.extend(extended.iter().position(|&t| t == seventh).and_then(window));
    }
    shapes
}

/// Le type a trois sons au-dessus de la fondamentale : le type B a une septième à poser
/// en bas.
fn has_seventh(chord_type: ChordType) -> bool {
    chord_type.intervals().len() >= 4
}

/// Position serrée à quatre voix (la fondamentale double à l'octave une triade), la
/// deuxième voix depuis l'aigu descend d'une octave.
fn drop2_shapes(quality: Quality) -> ArrayVec<Shape, 5> {
    let mut tones: ArrayVec<u8, 5> = quality.voicing_tones(4).into_iter().collect();
    if tones.len() < 4 {
        tones.push(tones[0] + 12);
    }
    inversions(&tones)
        .map(|close| {
            let dropped = close[2] - 12;
            let mut shape: Shape = [dropped, close[0], close[1], close[3]].into_iter().collect();
            shape.sort_unstable();
            shape
        })
        .collect()
}

/// Voicings candidats de `chord` pour un voicing de comping, dans toutes les octaves MIDI
/// (ordre : octave croissante, puis forme). `None` pour une pédale et pour les voicings de
/// triade, qui gardent leurs voicings serrés.
pub(crate) fn candidates(
    chord: DegreeChord,
    key: PitchClass,
    mode: Mode,
    kind: ChordVoicing,
) -> Option<impl Iterator<Item = Voicing>> {
    let quality = quality_of(chord)?;
    let shapes = match kind {
        ChordVoicing::CloseTriad | ChordVoicing::FixedTriad => return None,
        ChordVoicing::GuideTones => guide_shapes(quality),
        ChordVoicing::Rootless => rootless_shapes(quality, true, true),
        ChordVoicing::RootlessA => rootless_shapes(quality, true, false),
        ChordVoicing::RootlessB => rootless_shapes(quality, false, true),
        ChordVoicing::Drop2 => drop2_shapes(quality),
    };
    let root = i16::from(chord.root_pc(key, mode));
    Some((0..=10i16).flat_map(move |octave| {
        shapes.clone().into_iter().filter_map(move |shape| {
            shape
                .iter()
                .map(|&interval| {
                    u8::try_from(root + 12 * octave + i16::from(interval))
                        .ok()
                        .filter(|&pitch| pitch <= 127)
                })
                .collect::<Option<Voicing>>()
        })
    }))
}

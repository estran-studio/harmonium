//! Voicings d'accompagnement : notes guides, sans fondamentale, drop 2, drop 3, quartes,
//! structures supérieures, *shell* à deux voix et *spread*.
//!
//! Chaque voicing ne contient que des notes de l'accord publié (tensions comprises) et, si le
//! profil le demande (`accompaniment.chords.color`), d'une tension de couleur que la qualité
//! permet sans la contredire (voir [`natural_allowed`]) : la basse d'un accord slash n'y est
//! jamais ajoutée, la piste de basse la joue. Une pédale n'a pas de qualité typée : elle garde
//! les voicings serrés (`None` ici).
//!
//! Les formes sont des listes croissantes d'intervalles relatifs à la fondamentale
//! (négatifs possibles pour les drops) ; `candidates` les place dans toutes les octaves
//! MIDI. Le choix entre candidats (fenêtre, conduite de voix) revient à `ChordPlayer`. Une
//! famille qui n'offre aucune forme pour une qualité (quartes sur une triade, structure
//! supérieure sans tension) retombe sur les voicings sans fondamentale.

use arrayvec::ArrayVec;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::{
    accomp_profile::{ChordVoicing, ColorTensions},
    harmony::{ChordQuality, DegreeChord},
    mode::Mode,
};
use crate::{
    algo::RngCore,
    theory::{PitchClass, chord::ChordType, quality::Quality},
};

/// Voicing d'un accord : au plus 5 hauteurs, sur la pile.
pub(crate) type Voicing = ArrayVec<u8, 5>;

/// Une forme : intervalles croissants au-dessus de la fondamentale.
type Shape = ArrayVec<i8, 5>;

/// Les formes d'une famille pour une qualité.
type Shapes = ArrayVec<Shape, 16>;

/// Tension préférée d'abord : 9, 13, #11, b9, #9, b13, 11 (demi-tons au-dessus de la
/// fondamentale).
const TENSION_PRIORITY: [u8; 7] = [2, 9, 6, 1, 3, 8, 5];

/// Tension de couleur d'un voicing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Color {
    /// Priorité historique parmi les tensions publiées de la qualité.
    Priority,
    /// Tension tirée (demi-tons au-dessus de la fondamentale, `< 12`) ; `None` : aucune.
    Drawn(Option<u8>),
}

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

/// Tension publiée de la qualité qui passe en premier selon la priorité historique.
fn published_tension(quality: Quality) -> Option<u8> {
    let tensions = quality.extensions();
    TENSION_PRIORITY.into_iter().find(|t| tensions.contains(t))
}

/// Tension de couleur de la qualité, empilée au-dessus de l'octave.
fn color_tension(quality: Quality, color: Color) -> Option<u8> {
    match color {
        Color::Priority => published_tension(quality),
        Color::Drawn(tension) => tension,
    }
    .map(|t| t + 12)
}

/// Tensions naturelles (9 = 2, 11 = 5, 13 = 9) que le type permet : le *tableau de métier* des
/// couleurs sans conflit. Les accords à quinte ou septième altérée, diminués et augmentés n'en
/// ont pas.
const fn permitted_naturals(chord_type: ChordType) -> &'static [u8] {
    match chord_type {
        ChordType::Major | ChordType::Major7 | ChordType::Dominant7 | ChordType::Dominant7Sus4 => {
            &[2, 9]
        }
        ChordType::Minor | ChordType::Minor7 => &[2, 5],
        ChordType::Major6 | ChordType::Minor6 | ChordType::MinorMajor7 => &[2],
        _ => &[],
    }
}

/// `true` si la tension naturelle `tension` (2, 5 ou 9) colore la qualité sans conflit avec la
/// grille : le type la permet, aucun son de l'accord (type et tensions publiées) ne la double ni
/// ne l'altère (b9, #9 pour la 9 ; #11 pour la 11 ; b13 pour la 13), et elle ne tombe pas un
/// demi-ton au-dessus d'un son de l'accord (note à éviter).
pub(crate) fn natural_allowed(quality: Quality, tension: u8) -> bool {
    if !permitted_naturals(quality.chord_type()).contains(&tension) {
        return false;
    }
    let degree: &[u8] = match tension {
        2 => &[1, 2, 3],
        5 => &[5, 6],
        _ => &[8, 9],
    };
    let owned = quality.intervals();
    !owned.iter().any(|&i| degree.contains(&i) || (tension + 12 - i) % 12 == 1)
}

/// Tension de couleur d'un accord pour la graine `seed` : un tirage pondéré entre aucune tension,
/// la tension publiée et les tensions naturelles permises. Sans poids éligible (une pédale, ou
/// aucune tension que la qualité autorise), la tension historique.
pub(crate) fn draw_color(chord: DegreeChord, color: &ColorTensions, seed: u64) -> Color {
    let Some(quality) = quality_of(chord) else {
        return Color::Priority;
    };
    let published = published_tension(quality);
    let natural = |tension: u8, weight: u32| {
        if natural_allowed(quality, tension) { u64::from(weight) } else { 0 }
    };
    let weights = [
        u64::from(color.none_weight),
        if published.is_some() { u64::from(color.published_weight) } else { 0 },
        natural(2, color.ninth_weight),
        natural(5, color.eleventh_weight),
        natural(9, color.thirteenth_weight),
    ];
    let total: u64 = weights.iter().sum();
    if total == 0 {
        return Color::Priority;
    }
    let mut pick = ChaCha8Rng::seed_from_u64(seed).next_range_u64(0..total);
    let rank = weights
        .iter()
        .position(|&weight| {
            let hit = pick < weight;
            pick = pick.saturating_sub(weight);
            hit
        })
        .unwrap_or(0);
    match rank {
        0 => Color::Drawn(None),
        1 => Color::Drawn(published),
        2 => Color::Drawn(Some(2)),
        3 => Color::Drawn(Some(5)),
        _ => Color::Drawn(Some(9)),
    }
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

fn guide_shapes(quality: Quality, color: Color) -> Shapes {
    let (third, seventh) = guide_intervals(quality.chord_type());
    let mut tones: ArrayVec<u8, 3> = [third, seventh].into_iter().collect();
    tones.extend(color_tension(quality, color));
    tones.sort_unstable();
    inversions(&tones).collect()
}

/// Pile sans fondamentale : tierce, quinte, septième, tension de couleur. Type A : quatre
/// sons depuis la tierce ; type B : quatre sons depuis la septième (septième, tension,
/// tierce, quinte). Sans tension, la pile se prolonge à l'octave supérieure.
fn rootless_shapes(quality: Quality, color: Color, a: bool, b: bool) -> Shapes {
    let chord_type = quality.chord_type();
    let (_, seventh) = guide_intervals(chord_type);
    let core: ArrayVec<u8, 4> =
        chord_type.intervals().iter().copied().filter(|&t| t != 0).collect();
    let mut stack: ArrayVec<u8, 8> = core.iter().copied().collect();
    stack.extend(color_tension(quality, color));
    stack.sort_unstable();
    let mut extended: ArrayVec<u8, 16> = stack.iter().copied().collect();
    extended.extend(stack.iter().map(|t| t + 12));
    extended.sort_unstable();
    let window = |start: usize| -> Option<Shape> {
        extended.get(start..start + 4).map(|w| w.iter().map(|&t| t as i8).collect())
    };
    let mut shapes = Shapes::new();
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

/// Pile de quatre sons de l'accord avec sa tension : les tensions publiées (historique) ou la
/// tension tirée ; au-delà de `limit`, la quinte tombe d'abord, puis les sons les plus hauts.
fn stack(quality: Quality, color: Color, limit: usize) -> ArrayVec<u8, 5> {
    match color {
        Color::Priority => quality.voicing_tones(limit),
        Color::Drawn(tension) => {
            let mut tones: ArrayVec<u8, 8> =
                quality.chord_type().intervals().iter().copied().collect();
            tones.extend(tension.map(|t| t + 12));
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
    }
}

/// Positions serrées à quatre voix dans tous les renversements (la fondamentale double à l'octave
/// une triade).
fn close_positions(quality: Quality, color: Color) -> ArrayVec<Shape, 5> {
    let mut tones = stack(quality, color, 4);
    if tones.len() < 4 {
        tones.push(tones[0] + 12);
    }
    inversions(&tones).collect()
}

/// Drop 2 : la deuxième voix depuis l'aigu descend d'une octave.
fn drop2_shapes(quality: Quality, color: Color) -> Shapes {
    close_positions(quality, color)
        .into_iter()
        .map(|close| {
            let dropped = close[2] - 12;
            let mut shape: Shape = [dropped, close[0], close[1], close[3]].into_iter().collect();
            shape.sort_unstable();
            shape
        })
        .collect()
}

/// Drop 3 : la troisième voix depuis l'aigu descend d'une octave.
fn drop3_shapes(quality: Quality, color: Color) -> Shapes {
    close_positions(quality, color)
        .into_iter()
        .map(|close| {
            let dropped = close[1] - 12;
            let mut shape: Shape = [dropped, close[0], close[2], close[3]].into_iter().collect();
            shape.sort_unstable();
            shape
        })
        .collect()
}

/// *Spread* : drop 2 et 4, les deuxième et quatrième voix depuis l'aigu descendent d'une octave.
fn open_shapes(quality: Quality, color: Color) -> Shapes {
    close_positions(quality, color)
        .into_iter()
        .map(|close| {
            let mut shape: Shape =
                [close[0] - 12, close[2] - 12, close[1], close[3]].into_iter().collect();
            shape.sort_unstable();
            shape
        })
        .collect()
}

/// *Shell* : la tierce et la septième, dans les deux dispositions.
fn shell_shapes(quality: Quality) -> Shapes {
    let (third, seventh) = guide_intervals(quality.chord_type());
    let (low, high) = (third.min(seventh), third.max(seventh));
    [[low as i8, high as i8], [high as i8, (low + 12) as i8]]
        .into_iter()
        .map(|shape| shape.into_iter().collect())
        .collect()
}

/// Sons permis pour une structure supérieure : ceux du type et la tension de couleur (la
/// tension publiée en priorité historique).
fn color_pool(quality: Quality, color: Color) -> ArrayVec<u8, 12> {
    let mut pool: ArrayVec<u8, 12> = quality.chord_type().intervals().iter().copied().collect();
    match color {
        Color::Priority => {
            for tension in quality.extensions() {
                if !pool.contains(&tension) {
                    pool.push(tension);
                }
            }
        }
        Color::Drawn(tension) => {
            pool.extend(tension.filter(|t| !pool.contains(t)));
        }
    }
    pool
}

/// Sons permis pour une pile de quartes : tous ceux du type, des tensions publiées et des
/// tensions naturelles que la qualité permet.
fn wide_pool(quality: Quality) -> ArrayVec<u8, 12> {
    let mut pool: ArrayVec<u8, 12> = quality.intervals().into_iter().collect();
    for &tension in permitted_naturals(quality.chord_type()) {
        if natural_allowed(quality, tension) && !pool.contains(&tension) {
            pool.push(tension);
        }
    }
    pool
}

/// `true` si `interval` (modulo 12) est la tierce ou la septième du type.
fn is_guide(quality: Quality, interval: u8) -> bool {
    let (third, seventh) = guide_intervals(quality.chord_type());
    interval % 12 == third % 12 || interval % 12 == seventh % 12
}

/// Piles de trois quartes justes : quatre sons dans les sons permis qui gardent la tierce ou la
/// septième. Celles qui portent la tension de couleur tirée sont seules gardées quand il y en a.
fn fourths_shapes(quality: Quality, color: Color) -> Shapes {
    let pool = wide_pool(quality);
    let mut shapes = Shapes::new();
    let mut colored = Shapes::new();
    for start in 0..12u8 {
        let tones = [start, start + 5, start + 10, start + 15];
        if tones.iter().all(|t| pool.contains(&(t % 12)))
            && tones.iter().any(|&t| is_guide(quality, t))
        {
            let shape: Shape = tones.iter().map(|&t| t as i8).collect();
            if let Color::Drawn(Some(tension)) = color
                && tones.iter().any(|&t| t % 12 == tension)
                && !colored.is_full()
            {
                colored.push(shape.clone());
            }
            if !shapes.is_full() {
                shapes.push(shape);
            }
        }
    }
    if colored.is_empty() { shapes } else { colored }
}

/// Structures supérieures : toutes les triades (majeures et mineures, tous renversements) bâties
/// sur un degré autre que la fondamentale dont les trois sons sont permis et qui gardent la
/// tierce ou la septième de l'accord.
fn upper_structure_shapes(quality: Quality, color: Color) -> Shapes {
    let pool = color_pool(quality, color);
    let mut shapes = Shapes::new();
    for start in 1..12u8 {
        for third in [4u8, 3] {
            let tones = [start, start + third, start + 7];
            if tones.iter().all(|t| pool.contains(&(t % 12)))
                && tones.iter().any(|&t| is_guide(quality, t))
            {
                for shape in inversions(&tones) {
                    if !shapes.is_full() {
                        shapes.push(shape);
                    }
                }
            }
        }
    }
    shapes
}

/// Voicings candidats de `chord` pour un voicing de comping, dans toutes les octaves MIDI
/// (ordre : octave croissante, puis forme). `None` pour une pédale et pour les voicings de
/// triade, qui gardent leurs voicings serrés.
pub(crate) fn candidates(
    chord: DegreeChord,
    key: PitchClass,
    mode: Mode,
    kind: ChordVoicing,
    color: Color,
) -> Option<impl Iterator<Item = Voicing>> {
    let quality = quality_of(chord)?;
    // Une dyade sans tierce ni septième n'a pas de voicing de comping : voicing serré.
    if quality.chord_type() == ChordType::Power {
        return None;
    }
    let mut shapes = match kind {
        ChordVoicing::CloseTriad | ChordVoicing::FixedTriad => return None,
        ChordVoicing::GuideTones => guide_shapes(quality, color),
        ChordVoicing::Rootless => rootless_shapes(quality, color, true, true),
        ChordVoicing::RootlessA => rootless_shapes(quality, color, true, false),
        ChordVoicing::RootlessB => rootless_shapes(quality, color, false, true),
        ChordVoicing::Drop2 => drop2_shapes(quality, color),
        ChordVoicing::Drop3 => drop3_shapes(quality, color),
        ChordVoicing::Fourths => fourths_shapes(quality, color),
        ChordVoicing::UpperStructure => upper_structure_shapes(quality, color),
        ChordVoicing::Shell => shell_shapes(quality),
        ChordVoicing::Open => open_shapes(quality, color),
    };
    if shapes.is_empty() {
        shapes = rootless_shapes(quality, color, true, true);
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theory::quality::Quality;

    #[allow(clippy::panic)]
    fn quality(suffix: &str) -> Quality {
        Quality::parse(suffix).unwrap_or_else(|e| panic!("qualité {suffix} : {e}"))
    }

    #[test]
    fn natural_tensions_never_clash_with_the_published_quality() {
        // 9 sur 7b9 ou 7#9, 13 sur 7b13, 11 sur un accord majeur : refusés ; 9 et 13 sur 7 : permis.
        assert!(!natural_allowed(quality("7b9"), 2));
        assert!(!natural_allowed(quality("7#9"), 2));
        assert!(!natural_allowed(quality("7b13"), 9));
        assert!(!natural_allowed(quality("maj7"), 5));
        assert!(!natural_allowed(quality("7"), 5));
        assert!(natural_allowed(quality("7"), 2));
        assert!(natural_allowed(quality("7"), 9));
        assert!(natural_allowed(quality("7b9"), 9));
        assert!(natural_allowed(quality("m7"), 5));
        // Déjà publiée : pas une tension de plus.
        assert!(!natural_allowed(quality("9"), 2));
        assert!(!natural_allowed(quality("m7b5"), 2));
        assert!(!natural_allowed(quality("dim7"), 9));
    }

    #[test]
    fn every_family_keeps_to_chord_and_permitted_tension_tones() {
        for suffix in ["", "m", "7", "maj7", "m7", "m7b5", "6", "m6", "7b9", "13", "m11", "7sus4"] {
            let quality = quality(suffix);
            let owned: ArrayVec<u8, 12> = quality.intervals().into_iter().collect();
            for color in [
                Color::Priority,
                Color::Drawn(None),
                Color::Drawn(Some(2)),
                Color::Drawn(Some(5)),
                Color::Drawn(Some(9)),
            ] {
                if let Color::Drawn(Some(tension)) = color
                    && !natural_allowed(quality, tension)
                {
                    continue;
                }
                let allowed = |interval: i16| {
                    let pc = interval.rem_euclid(12) as u8;
                    owned.contains(&pc)
                        || match color {
                            Color::Drawn(Some(t)) => t == pc,
                            _ => false,
                        }
                        || wide_pool(quality).contains(&pc)
                };
                for (name, shapes) in [
                    ("drop2", drop2_shapes(quality, color)),
                    ("drop3", drop3_shapes(quality, color)),
                    ("open", open_shapes(quality, color)),
                    ("fourths", fourths_shapes(quality, color)),
                    ("upper", upper_structure_shapes(quality, color)),
                    ("shell", shell_shapes(quality)),
                    ("rootless", rootless_shapes(quality, color, true, true)),
                    ("guide", guide_shapes(quality, color)),
                ] {
                    for shape in &shapes {
                        for &interval in shape {
                            assert!(
                                allowed(i16::from(interval)),
                                "{name} {suffix} {color:?}: {shape:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn drop_shapes_move_the_expected_voice() {
        let c7 = quality("7");
        // Position serrée 0 4 7 10 : drop 2 = 0 4 7 10 avec 7 en bas (−5) ; drop 3 avec 4 en bas
        // (−8) ; spread : −12 et −5.
        let close = close_positions(c7, Color::Priority);
        assert_eq!(close[0].as_slice(), &[0, 4, 7, 10]);
        assert!(drop2_shapes(c7, Color::Priority).iter().any(|s| s.as_slice() == [-5, 0, 4, 10]));
        assert!(drop3_shapes(c7, Color::Priority).iter().any(|s| s.as_slice() == [-8, 0, 7, 10]));
        assert!(open_shapes(c7, Color::Priority).iter().any(|s| s.as_slice() == [-12, -5, 4, 10]));
    }

    #[test]
    fn fourths_stack_perfect_fourths() {
        for suffix in ["m7", "7", "maj7", "m11", "13"] {
            let shapes = fourths_shapes(quality(suffix), Color::Priority);
            assert!(!shapes.is_empty(), "{suffix}");
            for shape in shapes {
                assert!(shape.windows(2).all(|w| w[1] - w[0] == 5), "{suffix}: {shape:?}");
            }
        }
    }
}

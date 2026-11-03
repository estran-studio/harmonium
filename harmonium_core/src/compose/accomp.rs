//! Basse et accords d'accompagnement : motifs en pas du profil, voicings serrés ou de comping
//! (notes guides, sans fondamentale, drop 2) enchaînés par la conduite de voix, attaques
//! anticipées et alternance fondamentale/quinte de la basse. La marche de basse est dans
//! `walking`.

use rand::SeedableRng;

use super::{
    accomp_profile::{
        BassCandidate, BassDegree, BassDoubling, BassMotifNote, BassReference, BassSource,
        BassVoice, ChordHit, ChordVoicing,
    },
    harmony::DegreeChord,
    mode::Mode,
    plan::{ImposedRole, TunePlan},
    voicing::{self, Voicing},
};
use crate::{
    algo::RngCore,
    theory::{PitchClass, voice_leading},
    timeline::{Articulation, NoteId, TimelineNote, TrackId},
    tuning::AccompanimentParams,
};

/// Place la classe de hauteur `pc` dans `low..=high` (voir le doc du module
/// pour la classe absente d'une fenêtre de 11 demi-tons).
pub(crate) fn place_in_range(pc: PitchClass, low: u8, high: u8) -> u8 {
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

pub(crate) fn bass_hit_pitch(
    chords: &[DegreeChord],
    key: PitchClass,
    mode: Mode,
    params: &AccompanimentParams,
    hit: &super::accomp_profile::BassHit,
    steps_per_chord: usize,
) -> u8 {
    let slot = hit.step / steps_per_chord;
    let held = slot == 0 || chords[slot] == chords[slot - 1];
    let fifth = hit.degree == BassDegree::Fifth || (hit.degree == BassDegree::FifthIfHeld && held);
    // La quinte reste celle de la fondamentale ; la fondamentale du coup est la basse
    // de l'accord (celle d'un accord slash, sinon la fondamentale).
    let pc = if fifth {
        (chords[slot].root_pc(key, mode) + 7) % 12
    } else {
        chords[slot].bass_pc(key, mode)
    };
    place_in_range(pc, params.bass.low, params.bass.high)
}

#[allow(clippy::too_many_arguments)]
fn push_with_doublings(
    notes: &mut Vec<TimelineNote>,
    primary: u8,
    doublings: &[BassDoubling],
    step: usize,
    duration: usize,
    velocity: u8,
    articulation: Articulation,
    key: PitchClass,
    mode: Mode,
    next_id: &mut impl FnMut() -> NoteId,
) {
    for doubling in std::iter::once(None).chain(doublings.iter().copied().map(Some)) {
        let pitch = match doubling {
            None => primary,
            Some(BassDoubling::Octave) => primary + 12,
            Some(BassDoubling::Tenth) => {
                let mut pitch = primary;
                for _ in 0..2 {
                    loop {
                        pitch += 1;
                        if mode.contains_midi(key, pitch) {
                            break;
                        }
                    }
                }
                pitch + 12
            }
        };
        notes.push(TimelineNote {
            id: next_id(),
            pitch,
            start_step: step,
            duration_steps: duration,
            velocity,
            articulation,
            performance: Default::default(),
        });
    }
}

/// Basse d'une mesure : un coup par entrée du motif du profil.
#[must_use]
pub fn bass_bar(
    chords: &[DegreeChord],
    key: PitchClass,
    mode: Mode,
    params: &AccompanimentParams,
    steps_per_chord: usize,
    next_id: &mut impl FnMut() -> NoteId,
) -> Vec<TimelineNote> {
    let capacity = params.bass.hits.iter().map(|h| 1 + h.doubling.len()).sum();
    let mut notes = Vec::with_capacity(capacity);
    for hit in &params.bass.hits {
        let primary = bass_hit_pitch(chords, key, mode, params, hit, steps_per_chord);
        push_with_doublings(
            &mut notes,
            primary,
            &hit.doubling,
            hit.step,
            hit.duration,
            hit.velocity,
            Articulation::Normal,
            key,
            mode,
            next_id,
        );
    }
    notes
}

/// Tranche d'une mesure issue d'un motif ou d'un gabarit de basse (potentiellement multi-mesures).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BassMotifSlice {
    source: BassSource,
    bar_in_motif: usize,
}

impl BassMotifSlice {
    /// Indice du gabarit réalisé par cette tranche, `None` pour un motif écrit.
    pub(crate) fn template_index(self) -> Option<usize> {
        match self.source {
            BassSource::Template(index) => Some(index),
            BassSource::Motif(_) => None,
        }
    }

    /// Mesure de la tranche dans son cycle.
    pub(crate) fn bar_in_cycle(self) -> usize {
        self.bar_in_motif
    }
}

/// Plan fixé une fois par air : les reprises et tours rejouent les mêmes tranches.
/// Vide lorsque ni `bass.motifs` ni `bass.templates` n'est renseigné.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct BassMotifPlan {
    parts: Vec<Vec<BassMotifSlice>>,
    second_endings: Vec<Option<BassMotifSlice>>,
}

fn draw_bass_motif(
    bass: &BassVoice,
    part_name: &str,
    remaining_bars: usize,
    rng: &mut dyn RngCore,
) -> Option<BassSource> {
    let eligible = |c: &BassCandidate<'_>, specific: bool| {
        c.weight > 0
            && c.bars <= remaining_bars
            && if specific { c.part == Some(part_name) } else { c.part.is_none() }
    };
    let specific_total: u32 = bass.sources().filter(|c| eligible(c, true)).map(|c| c.weight).sum();
    let use_specific = specific_total > 0;
    let total: u32 = if use_specific {
        specific_total
    } else {
        bass.sources().filter(|c| eligible(c, false)).map(|c| c.weight).sum()
    };
    if total == 0 {
        return None;
    }
    let mut pick = rng.next_range_u64(0..u64::from(total)) as u32;
    for candidate in bass.sources() {
        if !eligible(&candidate, use_specific) {
            continue;
        }
        if pick < candidate.weight {
            return Some(candidate.source);
        }
        pick -= candidate.weight;
    }
    None
}

impl BassMotifPlan {
    /// Construit une tranche par mesure. Les copies remplacent cette seule mesure,
    /// sans suspendre le motif en cours ; ses tranches suivantes gardent leur position.
    /// `SequenceOf` et `VariedEndOf` ne transforment que la mélodie : pour la basse,
    /// ces rôles poursuivent le motif ou en tirent un comme les autres rôles sans copie.
    /// Un motif qui ne tient pas dans la fin de partie est tronqué à cette frontière.
    /// Aucun tirage si `bass.motifs` est vide.
    #[must_use]
    pub(crate) fn new(plan: &TunePlan, bass: &BassVoice, rng: &mut dyn RngCore) -> Self {
        if !bass.uses_plan() {
            return Self::default();
        }
        let form = &plan.profile.form;
        let mut parts: Vec<Vec<BassMotifSlice>> = Vec::with_capacity(form.parts.len());
        for (part_idx, part) in form.parts.iter().enumerate() {
            // Partie qui tait la basse (`tracks_off`) : aucune tranche. Le
            // générateur ne lit jamais ces mesures (`track_active` coupe la
            // piste) ; `slice_for_bar` renvoie `None` par accès protégé.
            if part.tracks_off.contains(&TrackId::Bass) {
                parts.push(Vec::new());
                continue;
            }
            let roles = plan.roles_for_part(part_idx);
            let mut chosen: Vec<BassMotifSlice> = Vec::with_capacity(roles.len());
            let mut ongoing: Option<BassMotifSlice> = None;
            for (bar, role) in roles.iter().enumerate() {
                // Copie depuis une partie muette : aucune tranche source ; tirer
                // une tranche fraîche au lieu de copier du vide.
                let copied = match *role {
                    ImposedRole::CopyOf(source) => Some(chosen[usize::from(source) - 1]),
                    ImposedRole::CopyOfPart { part: src_part, bar: src_bar } => {
                        parts.get(src_part).and_then(|v| v.get(usize::from(src_bar) - 1)).copied()
                    }
                    _ => None,
                };
                let slice = copied.unwrap_or_else(|| match ongoing {
                    Some(slice) => slice,
                    None => {
                        // Préférer un motif entier ; faute de candidat assez court,
                        // tronquer un motif éligible plutôt que laisser une fin vide.
                        let Some(source) =
                            draw_bass_motif(bass, &part.name, roles.len() - bar, rng)
                                .or_else(|| draw_bass_motif(bass, &part.name, usize::MAX, rng))
                        else {
                            unreachable!("chaque partie validée dispose d'un motif de basse");
                        };
                        let slice = BassMotifSlice { source, bar_in_motif: 0 };
                        ongoing = Some(slice);
                        slice
                    }
                });
                chosen.push(slice);
                ongoing = ongoing.and_then(|mut next| {
                    next.bar_in_motif += 1;
                    (next.bar_in_motif < bass.source_bars(next.source)).then_some(next)
                });
            }
            parts.push(chosen);
        }
        let second_endings = (0..form.parts.len())
            .map(|part_idx| {
                if form.parts[part_idx].tracks_off.contains(&TrackId::Bass) {
                    return None;
                }
                let role = plan.second_ending_role(part_idx)?;
                let part = &form.parts[part_idx];
                let chosen = &parts[part_idx];
                match role {
                    ImposedRole::CopyOf(source) => Some(chosen[usize::from(source) - 1]),
                    ImposedRole::CopyOfPart { part: src_part, bar: src_bar } => {
                        parts.get(src_part).and_then(|v| v.get(usize::from(src_bar) - 1)).copied()
                    }
                    // Une seconde fin ne démarre jamais un motif multi-mesures :
                    // tirer une mesure entière ou rejouer la tranche finale d'origine.
                    _ => draw_bass_motif(bass, &part.name, 1, rng)
                        .map(|source| BassMotifSlice { source, bar_in_motif: 0 })
                        .or_else(|| chosen.last().copied()),
                }
            })
            .collect();
        Self { parts, second_endings }
    }

    /// Tranche de motif jouée à la mesure `bar_in_tour` du tour (`None` :
    /// plan vide ou partie qui tait la basse).
    #[must_use]
    pub(crate) fn slice_for_bar(
        &self,
        plan: &TunePlan,
        bar_in_tour: usize,
    ) -> Option<BassMotifSlice> {
        if self.parts.is_empty() {
            return None;
        }
        let (part, _, offset, second_ending) = plan.position(bar_in_tour);
        if second_ending && let Some(ending) = self.second_endings.get(part).and_then(|e| *e) {
            return Some(ending);
        }
        self.parts.get(part).and_then(|v| v.get(offset)).copied()
    }
}

/// Hauteur d'une note de motif : référence placée dans la fenêtre de la basse, puis
/// `semitones` ajoutés tels quels, ou `degree` en pas diatoniques du mode actif (1 = unisson)
/// à partir de la référence ; `octave` décale de 12 demi-tons.
#[allow(clippy::too_many_arguments)]
fn bass_motif_note_pitch(
    note: &BassMotifNote,
    local_step: usize,
    reference: BassReference,
    chords: &[DegreeChord],
    key: PitchClass,
    mode: Mode,
    bass: &BassVoice,
    steps_per_chord: usize,
) -> u8 {
    let ref_pc = match reference {
        BassReference::ChordRoot => chords[local_step / steps_per_chord].bass_pc(key, mode),
        BassReference::Tonic => key % 12,
    };
    let mut pitch =
        i16::from(place_in_range(ref_pc, bass.low, bass.high)) + 12 * i16::from(note.octave);
    if let Some(semitones) = note.semitones {
        pitch += i16::from(semitones);
    } else if let Some(degree) = note.degree {
        let degree = i16::from(degree);
        let direction = if degree > 0 { 1 } else { -1 };
        for _ in 0..(if degree > 0 { degree - 1 } else { -degree }) {
            loop {
                pitch += direction;
                if !(0..=127).contains(&pitch) || mode.contains_midi(key, pitch as u8) {
                    break;
                }
            }
        }
    }
    // La validation garantit 0..=127 pour toutes les références et tous les modes permis.
    pitch as u8
}

/// Basse d'une mesure tirée d'une tranche de motif (`slice`), découpée aux barres
/// de mesure avec `Articulation::Tied` sur les tenues franchissant la barre.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub(crate) fn bass_motif_bar(
    slice: BassMotifSlice,
    chords: &[DegreeChord],
    key: PitchClass,
    mode: Mode,
    params: &AccompanimentParams,
    steps_per_bar: usize,
    steps_per_chord: usize,
    next_id: &mut impl FnMut() -> NoteId,
) -> Vec<TimelineNote> {
    let BassSource::Motif(motif_index) = slice.source else {
        unreachable!("un gabarit est réalisé par bass_template_bar");
    };
    let motif = &params.bass.motifs[motif_index];
    let bar_start = slice.bar_in_motif * steps_per_bar;
    let bar_end = bar_start + steps_per_bar;
    let mut notes = Vec::new();
    for note in &motif.notes {
        let note_end = note.step + note.duration;
        if note_end <= bar_start || note.step >= bar_end {
            continue;
        }
        let local_start = note.step.saturating_sub(bar_start);
        let local_end = note_end.min(bar_end) - bar_start;
        let duration = local_end - local_start;
        let tied = note.step < bar_start || note.tie;
        let primary = bass_motif_note_pitch(
            note,
            local_start,
            motif.reference,
            chords,
            key,
            mode,
            &params.bass,
            steps_per_chord,
        );
        let articulation = if tied {
            Articulation::Tied
        } else if note.legato {
            Articulation::Legato
        } else {
            Articulation::Normal
        };
        push_with_doublings(
            &mut notes,
            primary,
            &note.doubling,
            local_start,
            duration,
            note.velocity,
            articulation,
            key,
            mode,
            next_id,
        );
    }
    notes
}

/// Première attaque (pas, hauteur) de la mesure pour l'approche de basse de la mesure précédente.
#[allow(clippy::too_many_arguments)]
pub(crate) fn first_bass_target(
    bass_plan: &BassMotifPlan,
    plan: &TunePlan,
    bar_in_tour: usize,
    chords: &[DegreeChord],
    params: &AccompanimentParams,
    steps_per_bar: usize,
    steps_per_chord: usize,
) -> Option<(usize, u8)> {
    if let Some(slice) = bass_plan.slice_for_bar(plan, bar_in_tour) {
        if let Some(index) = slice.template_index() {
            return super::bass_template::first_attack(
                &params.bass,
                index,
                slice.bar_in_motif,
                chords,
                plan.key,
                plan.mode,
                steps_per_chord,
            );
        }
        let BassSource::Motif(motif_index) = slice.source else {
            unreachable!("gabarit traité plus haut");
        };
        let motif = &params.bass.motifs[motif_index];
        let bar_start = slice.bar_in_motif * steps_per_bar;
        let bar_end = bar_start + steps_per_bar;
        let note =
            motif.notes.iter().find(|n| n.step + n.duration > bar_start && n.step < bar_end)?;
        let local_start = note.step.saturating_sub(bar_start);
        let pitch = bass_motif_note_pitch(
            note,
            local_start,
            motif.reference,
            chords,
            plan.key,
            plan.mode,
            &params.bass,
            steps_per_chord,
        );
        Some((local_start, pitch))
    } else {
        let hit = params.bass.hits.first()?;
        Some((hit.step, bass_hit_pitch(chords, plan.key, plan.mode, params, hit, steps_per_chord)))
    }
}

/// Les accords s'inversent ; une pédale garde son voisin au-dessus d'elle. Les tensions
/// (9, 11, 13) restent au-dessus de l'octave de la fondamentale avant renversement.
///
/// Un accord slash est joué sur sa basse : si la basse est une note du voicing, seules les
/// inversions qui la placent au grave sont proposées ; sinon elle s'ajoute sous le voicing
/// (alors limité à quatre notes pour tenir en cinq).
fn close_voicings(
    chord: DegreeChord,
    key: PitchClass,
    mode: Mode,
) -> impl Iterator<Item = Voicing> {
    let root = chord.root_pc(key, mode);
    let mut tones = chord.intervals();
    let bass = chord.is_slash(mode).then(|| chord.bass_pc(key, mode));
    let inside = bass.is_some_and(|bass| tones.iter().any(|t| (root + t) % 12 == bass));
    let foreign = bass.filter(|_| !inside);
    if foreign.is_some() {
        tones = chord.voicing_tones(4);
    }
    let inversions = if tones.len() < 3 { 1 } else { tones.len() };
    (0..=10u8).flat_map(move |octave| {
        let tones = tones.clone();
        (0..inversions).filter_map(move |inversion| {
            let base = u16::from(root) + 12 * u16::from(octave);
            let mut voicing: Voicing = (inversion..inversion + tones.len())
                .map(|i| {
                    let wrapped = if i % tones.len() < inversion { 12 } else { 0 };
                    u8::try_from(base + u16::from(tones[i % tones.len()]) + wrapped)
                        .ok()
                        .filter(|&n| n <= 127)
                })
                .collect::<Option<_>>()?;
            // Les tensions empilées au-delà de l'octave peuvent passer sous la note
            // renversée : l'ordre ascendant redevient celui du voicing.
            voicing.sort_unstable();
            if let Some(bass) = bass {
                if foreign.is_some() {
                    let below = voicing[0].checked_sub(1)?;
                    let drop = (below % 12 + 12 - bass) % 12;
                    voicing.insert(0, below.checked_sub(drop)?);
                } else if voicing[0] % 12 != bass {
                    return None;
                }
            }
            Some(voicing)
        })
    })
}

/// Accords d'accompagnement : enchaîne les voicings d'une mesure à l'autre.
///
/// Un `ChordPlayer` neuf démarre sans voicing précédent ; en créer un par tune.
#[derive(Clone, Debug, Default)]
pub struct ChordPlayer {
    previous: Option<Voicing>,
    /// Décalage du registre en demi-tons (`accompaniment.chords.register_drift`), nul au départ.
    drift: i16,
}

impl ChordPlayer {
    #[must_use]
    pub const fn new() -> Self {
        Self { previous: None, drift: 0 }
    }

    /// Déplace le centre du registre pour la mesure de graine `seed` : un pas vers le haut, vers
    /// le bas ou rien, sans jamais dépasser `max` demi-tons autour de `center`.
    fn drift_bar(&mut self, params: &AccompanimentParams, seed: u64) {
        let Some(drift) = params.chords.register_drift else {
            return;
        };
        let mut rng =
            rand_chacha::ChaCha8Rng::seed_from_u64(crate::ornaments::mix(seed, 0x4452_4654));
        let step = i16::from(drift.step);
        let moved = match rng.next_range_u64(0..3) {
            0 => -step,
            1 => 0,
            _ => step,
        };
        let max = i16::from(drift.max);
        self.drift = (self.drift + moved).clamp(-max, max);
    }

    /// Voicing le plus proche du précédent (ou du centre du profil au premier
    /// accord), dans la limite de dérive autorisée. Un voicing jazz reste dans la fenêtre
    /// `low..=high` quand un candidat y tient, sinon sort le moins possible. `color_seed` tire
    /// la tension de couleur (`accompaniment.chords.color`).
    fn voice(
        &mut self,
        chord: DegreeChord,
        key: PitchClass,
        mode: Mode,
        params: &AccompanimentParams,
        color_seed: u64,
    ) -> Option<Voicing> {
        let color = params
            .chords
            .color
            .as_ref()
            .map_or(voicing::Color::Priority, |c| voicing::draw_color(chord, c, color_seed));
        let candidates = match voicing::candidates(chord, key, mode, params.chords.voicing, color) {
            Some(comping) => Candidates::Comping(comping),
            None => Candidates::Close(close_voicings(chord, key, mode).filter(|v| {
                // Le doublage des voix supplémentaires ne concerne que les triades.
                v.len() != 3
                    || params.chords.note_count <= 3
                    || v[usize::from(params.chords.note_count - 4)] <= 115
            })),
        };
        let windowed = matches!(candidates, Candidates::Comping(_));
        let center = i32::from(params.chords.center) + i32::from(self.drift);
        let best = if params.chords.voicing == ChordVoicing::FixedTriad {
            candidates
                .filter(|v| v[0] >= params.chords.low && v[v.len() - 1] <= params.chords.high)
                .min_by_key(|v| v[0])
        } else {
            candidates.min_by_key(|v| {
                let count = v.len() as i32;
                let center_sum = count * center;
                let max_drift_sum = count * i32::from(params.chords.max_drift);
                let sum: i32 = v.iter().map(|&n| i32::from(n)).sum();
                let center_dist = (sum - center_sum).abs();
                let outside = if windowed {
                    i32::from(params.chords.low.saturating_sub(v[0]))
                        + i32::from(v[v.len() - 1].saturating_sub(params.chords.high))
                } else {
                    0
                };
                let movement = self.previous.as_ref().map_or(0, |p| {
                    i32::try_from(voice_leading::voicing_movement(v, p)).unwrap_or(i32::MAX)
                });
                (outside, center_dist > max_drift_sum, movement, center_dist, v[0])
            })
        }?;
        self.previous = Some(best.clone());
        Some(best)
    }

    /// Accords d'une mesure : un coup par entrée du motif par défaut.
    #[allow(clippy::too_many_arguments)]
    pub fn chord_bar(
        &mut self,
        chords: &[DegreeChord],
        key: PitchClass,
        mode: Mode,
        params: &AccompanimentParams,
        steps_per_chord: usize,
        seed: u64,
        next_id: &mut impl FnMut() -> NoteId,
    ) -> Vec<TimelineNote> {
        self.chord_bar_on_hits(
            chords,
            None,
            key,
            mode,
            params,
            &params.chords.patterns[params.chords.default_pattern].hits,
            steps_per_chord,
            seed,
            next_id,
        )
    }

    /// Variante pilotée par des attaques empruntées au profil, sans copier le tuning.
    /// `next_first` : premier accord de la mesure suivante, que jouent les attaques anticipées
    /// dont la fin tombe sur la barre.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn chord_bar_on_hits(
        &mut self,
        chords: &[DegreeChord],
        next_first: Option<DegreeChord>,
        key: PitchClass,
        mode: Mode,
        params: &AccompanimentParams,
        hits: &[ChordHit],
        steps_per_chord: usize,
        seed: u64,
        next_id: &mut impl FnMut() -> NoteId,
    ) -> Vec<TimelineNote> {
        let voice = &params.chords;
        self.drift_bar(params, seed);
        let mut notes = Vec::with_capacity(hits.len() * usize::from(voice.note_count));
        for hit in hits.iter().filter(|hit| super::chord_spread::selected(hit, seed)) {
            let step = hit.step;
            let Some(voicing) = self.voice(
                hit_chord(hit, chords, next_first, steps_per_chord),
                key,
                mode,
                params,
                color_seed(hit, seed, steps_per_chord),
            ) else {
                continue;
            };
            // Les voix supplémentaires doublent les triades seulement :
            // une pédale écrite reste une note, un voisin reste une dyade.
            let note_count = if voice.voicing.is_triad_kind() && voicing.len() == 3 {
                usize::from(voice.note_count)
            } else {
                voicing.len()
            };
            let mut pitches = [0u8; 6];
            pitches[..voicing.len()].copy_from_slice(&voicing);
            for i in voicing.len()..note_count {
                pitches[i] = pitches[i - voicing.len()] + 12;
            }
            let (first, count) = if hit.direction == super::accomp_profile::AttackDirection::Up {
                let count = voice
                    .spread
                    .as_ref()
                    .map_or(note_count, |s| usize::from(s.up_note_count).min(note_count));
                (note_count - count, count)
            } else {
                (0, note_count)
            };
            let pitches = &mut pitches[first..first + count];
            if hit.direction == super::accomp_profile::AttackDirection::Up {
                pitches.reverse();
            }
            let velocities = super::chord_spread::velocities(voice, hit, count, seed);
            notes.extend(pitches.iter().zip(velocities).map(|(&pitch, velocity)| TimelineNote {
                id: next_id(),
                pitch,
                start_step: step,
                duration_steps: hit.duration,
                velocity,
                articulation: if hit.tie { Articulation::Tied } else { Articulation::Normal },
                performance: Default::default(),
            }));
        }
        notes
    }

    /// Rejoue seulement la conduite de voix : aucune note ni allocation jetable.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn advance_bar(
        &mut self,
        chords: &[DegreeChord],
        next_first: Option<DegreeChord>,
        key: PitchClass,
        mode: Mode,
        params: &AccompanimentParams,
        hits: &[ChordHit],
        steps_per_chord: usize,
        seed: u64,
    ) {
        self.drift_bar(params, seed);
        for hit in hits.iter().filter(|h| super::chord_spread::selected(h, seed)) {
            let _ = self.voice(
                hit_chord(hit, chords, next_first, steps_per_chord),
                key,
                mode,
                params,
                color_seed(hit, seed, steps_per_chord),
            );
        }
    }
}

/// Graine de la tension de couleur d'une attaque : la mesure et le créneau d'accord qu'elle joue.
fn color_seed(hit: &ChordHit, seed: u64, steps_per_chord: usize) -> u64 {
    let at = if hit.anticipate { hit.step + hit.duration } else { hit.step };
    crate::ornaments::mix(seed, 0x434F_4C00 + (at / steps_per_chord) as u64)
}

/// Accord d'une attaque : celui du pas, ou, pour une attaque anticipée, celui en vigueur à la
/// fin de sa durée (le premier accord de la mesure suivante si elle y tombe, à défaut l'accord
/// du pas).
fn hit_chord(
    hit: &ChordHit,
    chords: &[DegreeChord],
    next_first: Option<DegreeChord>,
    steps_per_chord: usize,
) -> DegreeChord {
    let at = if hit.anticipate { hit.step + hit.duration } else { hit.step };
    chords
        .get(at / steps_per_chord)
        .copied()
        .or(next_first)
        .unwrap_or(chords[hit.step / steps_per_chord])
}

/// Candidats d'un voicing : serrés (triades, pédales) ou de comping.
enum Candidates<A, B> {
    Close(A),
    Comping(B),
}

impl<A: Iterator<Item = Voicing>, B: Iterator<Item = Voicing>> Iterator for Candidates<A, B> {
    type Item = Voicing;

    fn next(&mut self) -> Option<Voicing> {
        match self {
            Self::Close(iter) => iter.next(),
            Self::Comping(iter) => iter.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::harmony::{ChordDegree, ChordQuality, candidate_chords};

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

    fn major(degree: ChordDegree) -> DegreeChord {
        DegreeChord::new(degree, ChordQuality::Major)
    }

    fn mechanical() -> AccompanimentParams {
        let mut p = AccompanimentParams::default();
        p.chords.note_count = 3;
        p.chords.spread = None;
        p
    }

    fn chord(chord_player: &mut ChordPlayer, chords: [DegreeChord; 2]) -> Vec<TimelineNote> {
        chord_player.chord_bar(&chords, D, Mode::Ionian, &mechanical(), 8, 0, &mut ids())
    }

    #[test]
    fn bass_plays_root_then_fifth_on_steps_0_and_8() {
        let one = [major(ChordDegree::I); 2];
        let notes = bass_bar(&one, D, Mode::Ionian, &AccompanimentParams::default(), 8, &mut ids());
        let layout: Vec<_> =
            notes.iter().map(|n| (n.start_step, n.duration_steps, n.pitch)).collect();
        // Ré2 = 38 est sous la fenêtre : fondamentale Ré3 (50), quinte La2 (45).
        assert_eq!(layout, [(0, 4, 50), (8, 4, 45)]);
        let params = AccompanimentParams::default();
        assert!(notes.iter().all(|n| n.velocity == params.bass.hits[0].velocity));
    }

    #[test]
    fn bass_plays_root_when_chord_changes_at_half_bar() {
        let chords = [major(ChordDegree::I), major(ChordDegree::IV)];
        let notes =
            bass_bar(&chords, D, Mode::Ionian, &AccompanimentParams::default(), 8, &mut ids());
        // Sol2 = 43.
        assert_eq!(
            notes.iter().map(|n| (n.start_step, n.pitch)).collect::<Vec<_>>(),
            [(0, 50), (8, 43)]
        );
    }

    #[test]
    fn bass_stays_in_e2_to_d3_for_every_key_mode_and_chord() {
        let params = AccompanimentParams::default();
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
        let a = DegreeChord::new(ChordDegree::VI, ChordQuality::Major);
        let notes =
            bass_bar(&[a, a], 7, Mode::Aeolian, &AccompanimentParams::default(), 8, &mut ids());
        assert_eq!(notes[0].pitch, 39);
    }

    #[test]
    fn chord_hits_on_steps_4_and_12_with_three_chord_tones() {
        let chords = [major(ChordDegree::I), major(ChordDegree::IV)];
        let notes = chord(&mut ChordPlayer::new(), chords);
        assert_eq!(notes.len(), 6);
        for (i, step) in [4, 12].into_iter().enumerate() {
            let group: Vec<_> = notes.iter().filter(|n| n.start_step == step).collect();
            assert_eq!(group.len(), 3, "step {step}");
            let chord_velocity = AccompanimentParams::default().chords.patterns[1].hits[0].velocity;
            assert!(group.iter().all(|n| n.duration_steps == 1 && n.velocity == chord_velocity));
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
        let notes = chord(&mut ChordPlayer::new(), [major(ChordDegree::I); 2]);
        let mut pitches: Vec<u8> = notes.iter().take(3).map(|n| n.pitch).collect();
        pitches.sort_unstable();
        // Ré majeur : La3 Ré4 Fa♯4, centre 61,7 (C4 = 60).
        assert_eq!(pitches, [57, 62, 66]);
    }

    #[test]
    fn next_voicing_is_the_closest_to_the_previous_one() {
        let mut chord_player = ChordPlayer::new();
        let _ = chord(&mut chord_player, [major(ChordDegree::I); 2]);
        let notes = chord(&mut chord_player, [major(ChordDegree::IV); 2]);
        let mut pitches: Vec<u8> = notes.iter().take(3).map(|n| n.pitch).collect();
        pitches.sort_unstable();
        // La3 Ré4 Fa♯4 → Si3 Ré4 Sol4 (mouvement total 1+0+1 = 2).
        assert_eq!(pitches, [59, 62, 67]);
    }

    #[test]
    fn voicings_never_drift_away_from_c4() {
        let params = mechanical();
        let mut chord_player = ChordPlayer::new();
        let mut next_id = ids();
        for bar in 0..64 {
            let mode = Mode::ALL[bar % Mode::ALL.len()];
            let candidates = candidate_chords(mode, profile());
            let chords =
                [candidates[bar % candidates.len()], candidates[(bar * 3 + 1) % candidates.len()]];
            let notes = chord_player.chord_bar(&chords, D, mode, &params, 8, 0, &mut next_id);
            for group in notes.chunks(3) {
                let sum: i32 = group.iter().map(|n| i32::from(n.pitch)).sum();
                assert!((sum - 3 * 60).abs() <= 3 * 6, "bar {bar}: {group:?}");
            }
        }
    }

    #[test]
    fn note_ids_are_unique_across_bass_and_chords() {
        let mut next_id = ids();
        let params = AccompanimentParams::default();
        let chords = [major(ChordDegree::I); 2];
        let mut all = bass_bar(&chords, D, Mode::Ionian, &params, 8, &mut next_id);
        all.extend(ChordPlayer::new().chord_bar(
            &chords,
            D,
            Mode::Ionian,
            &params,
            8,
            0,
            &mut next_id,
        ));
        let mut found: Vec<_> = all.iter().map(|n| n.id).collect();
        found.sort_unstable();
        found.dedup();
        assert_eq!(found.len(), all.len());
    }
}

#[cfg(test)]
mod drift_tests {
    use super::*;
    use crate::compose::accomp_profile::RegisterDrift;

    #[test]
    fn register_drift_stays_within_its_bound_and_reaches_it() {
        let mut params = AccompanimentParams::default();
        params.chords.register_drift = Some(RegisterDrift { step: 3, max: 7 });
        let mut player = ChordPlayer::new();
        let mut seen = std::collections::BTreeSet::new();
        for seed in 0..4000u64 {
            let before = player.drift;
            player.drift_bar(&params, seed);
            assert!((-7..=7).contains(&player.drift), "dérive {}", player.drift);
            assert!((player.drift - before).abs() <= 3, "pas {before} -> {}", player.drift);
            seen.insert(player.drift);
        }
        // Haut, bas et immobile se tirent ; la dérive atteint les deux bornes.
        assert!(seen.contains(&7) && seen.contains(&-7), "{seen:?}");
        // Sans `register_drift`, la dérive reste nulle.
        let mut still = ChordPlayer::new();
        still.drift_bar(&AccompanimentParams::default(), 1);
        assert_eq!(still.drift, 0);
    }
}

//! Basse et accords d'accompagnement : motifs en pas du profil, triades
//! en conduite de voix serrée et alternance fondamentale/quinte de la basse.

use super::{
    accomp_profile::{
        BassDegree, BassDoubling, BassMotif, BassMotifNote, BassReference, BassVoice, ChordHit,
        ChordVoicing,
    },
    harmony::TradChord,
    mode::Mode,
    plan::{ImposedRole, TunePlan},
};
use crate::{
    algo::RngCore,
    theory::PitchClass,
    timeline::{Articulation, NoteId, TimelineNote, TrackId},
    tuning::TradHarmonyParams,
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
    chords: &[TradChord],
    key: PitchClass,
    mode: Mode,
    params: &TradHarmonyParams,
    hit: &super::accomp_profile::BassHit,
    steps_per_chord: usize,
) -> u8 {
    let slot = hit.step / steps_per_chord;
    let root = chords[slot].root_pc(key, mode);
    let held = slot == 0 || chords[slot] == chords[slot - 1];
    let fifth = hit.degree == BassDegree::Fifth || (hit.degree == BassDegree::FifthIfHeld && held);
    place_in_range(if fifth { (root + 7) % 12 } else { root }, params.bass.low, params.bass.high)
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
    chords: &[TradChord],
    key: PitchClass,
    mode: Mode,
    params: &TradHarmonyParams,
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

/// Tranche d'une mesure issue d'un motif de basse (potentiellement multi-mesures).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BassMotifSlice {
    motif_index: usize,
    bar_in_motif: usize,
}

/// Plan fixé une fois par air : les reprises et tours rejouent les mêmes tranches.
/// Vide lorsque `bass.motifs` est vide.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct BassMotifPlan {
    parts: Vec<Vec<BassMotifSlice>>,
    second_endings: Vec<Option<BassMotifSlice>>,
}

fn draw_bass_motif(
    motifs: &[BassMotif],
    part_name: &str,
    remaining_bars: usize,
    rng: &mut dyn RngCore,
) -> Option<usize> {
    let eligible = |m: &BassMotif, specific: bool| {
        m.weight > 0
            && m.bars <= remaining_bars
            && if specific { m.part.as_deref() == Some(part_name) } else { m.part.is_none() }
    };
    let specific_total: u32 = motifs.iter().filter(|m| eligible(m, true)).map(|m| m.weight).sum();
    let use_specific = specific_total > 0;
    let total: u32 = if use_specific {
        specific_total
    } else {
        motifs.iter().filter(|m| eligible(m, false)).map(|m| m.weight).sum()
    };
    if total == 0 {
        return None;
    }
    let mut pick = rng.next_range_u64(0..u64::from(total)) as u32;
    for (index, motif) in motifs.iter().enumerate() {
        if !eligible(motif, use_specific) {
            continue;
        }
        if pick < motif.weight {
            return Some(index);
        }
        pick -= motif.weight;
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
        if bass.motifs.is_empty() {
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
                        let Some(motif_index) =
                            draw_bass_motif(&bass.motifs, &part.name, roles.len() - bar, rng)
                                .or_else(|| {
                                    draw_bass_motif(&bass.motifs, &part.name, usize::MAX, rng)
                                })
                        else {
                            unreachable!("chaque partie validée dispose d'un motif de basse");
                        };
                        let slice = BassMotifSlice { motif_index, bar_in_motif: 0 };
                        ongoing = Some(slice);
                        slice
                    }
                });
                chosen.push(slice);
                ongoing = ongoing.and_then(|mut next| {
                    next.bar_in_motif += 1;
                    (next.bar_in_motif < bass.motifs[next.motif_index].bars).then_some(next)
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
                    _ => draw_bass_motif(&bass.motifs, &part.name, 1, rng)
                        .map(|motif_index| BassMotifSlice { motif_index, bar_in_motif: 0 })
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
    chords: &[TradChord],
    key: PitchClass,
    mode: Mode,
    bass: &BassVoice,
    steps_per_chord: usize,
) -> u8 {
    let ref_pc = match reference {
        BassReference::ChordRoot => chords[local_step / steps_per_chord].root_pc(key, mode),
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
    chords: &[TradChord],
    key: PitchClass,
    mode: Mode,
    params: &TradHarmonyParams,
    steps_per_bar: usize,
    steps_per_chord: usize,
    next_id: &mut impl FnMut() -> NoteId,
) -> Vec<TimelineNote> {
    let motif = &params.bass.motifs[slice.motif_index];
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
    chords: &[TradChord],
    params: &TradHarmonyParams,
    steps_per_bar: usize,
    steps_per_chord: usize,
) -> Option<(usize, u8)> {
    if let Some(slice) = bass_plan.slice_for_bar(plan, bar_in_tour) {
        let motif = &params.bass.motifs[slice.motif_index];
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

/// Les triades s'inversent ; une pédale garde son voisin au-dessus d'elle.
fn close_voicings(
    chord: TradChord,
    key: PitchClass,
    mode: Mode,
) -> impl Iterator<Item = arrayvec::ArrayVec<u8, 3>> {
    let root = chord.root_pc(key, mode);
    let tones = chord.intervals();
    let inversions = if tones.len() < 3 { 1 } else { tones.len() };
    (0..=10u8).flat_map(move |octave| {
        let tones = tones.clone();
        (0..inversions).filter_map(move |inversion| {
            let base = u16::from(root) + 12 * u16::from(octave);
            (inversion..inversion + tones.len())
                .map(|i| {
                    let wrapped = if i % tones.len() < inversion { 12 } else { 0 };
                    u8::try_from(base + u16::from(tones[i % tones.len()]) + wrapped)
                        .ok()
                        .filter(|&n| n <= 127)
                })
                .collect()
        })
    })
}

/// Guitare d'accompagnement : enchaîne les voicings d'une mesure à l'autre.
///
/// Un `Guitarist` neuf démarre sans voicing précédent ; en créer un par tune.
#[derive(Clone, Debug, Default)]
pub struct Guitarist {
    previous: Option<arrayvec::ArrayVec<u8, 3>>,
}

impl Guitarist {
    #[must_use]
    pub const fn new() -> Self {
        Self { previous: None }
    }

    /// Voicing le plus proche du précédent (ou du centre du profil au premier
    /// accord), dans la limite de dérive autorisée.
    fn voice(
        &mut self,
        chord: TradChord,
        key: PitchClass,
        mode: Mode,
        params: &TradHarmonyParams,
    ) -> Option<arrayvec::ArrayVec<u8, 3>> {
        let candidates = close_voicings(chord, key, mode).filter(|v| {
            v.len() < 3
                || params.chords.strings <= 3
                || v[usize::from(params.chords.strings - 4)] <= 115
        });
        let best = if params.chords.voicing == ChordVoicing::FixedTriad {
            candidates
                .filter(|v| v[0] >= params.chords.low && v[v.len() - 1] <= params.chords.high)
                .min_by_key(|v| v[0])
        } else {
            let count = chord.intervals().len() as i32;
            let center_sum = count * i32::from(params.chords.center);
            let max_drift_sum = count * i32::from(params.chords.max_drift);
            candidates.min_by_key(|v| {
                let sum: i32 = v.iter().map(|&n| i32::from(n)).sum();
                let center_dist = (sum - center_sum).abs();
                let movement: i32 = self.previous.as_ref().map_or(0, |p| {
                    v.iter().zip(p).map(|(&a, &b)| (i32::from(a) - i32::from(b)).abs()).sum()
                });
                (center_dist > max_drift_sum, movement, center_dist, v[0])
            })
        }?;
        self.previous = Some(best.clone());
        Some(best)
    }

    /// Accords d'une mesure : un coup par entrée du motif par défaut.
    #[allow(clippy::too_many_arguments)]
    pub fn chop_bar(
        &mut self,
        chords: &[TradChord],
        key: PitchClass,
        mode: Mode,
        params: &TradHarmonyParams,
        steps_per_chord: usize,
        seed: u64,
        next_id: &mut impl FnMut() -> NoteId,
    ) -> Vec<TimelineNote> {
        self.chop_bar_on_hits(
            chords,
            key,
            mode,
            params,
            &params.chords.patterns[params.chords.default_pattern].hits,
            steps_per_chord,
            seed,
            next_id,
        )
    }

    /// Variante pilotée par un motif emprunté au profil, sans copier le tuning.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn chop_bar_on_hits(
        &mut self,
        chords: &[TradChord],
        key: PitchClass,
        mode: Mode,
        params: &TradHarmonyParams,
        hits: &[ChordHit],
        steps_per_chord: usize,
        seed: u64,
        next_id: &mut impl FnMut() -> NoteId,
    ) -> Vec<TimelineNote> {
        let voice = &params.chords;
        let mut notes = Vec::with_capacity(hits.len() * usize::from(voice.strings));
        for hit in hits.iter().filter(|hit| super::strum::selected(hit, seed)) {
            let step = hit.step;
            let Some(voicing) = self.voice(chords[step / steps_per_chord], key, mode, params)
            else {
                continue;
            };
            // Les cordes supplémentaires doublent les triades seulement :
            // une pédale écrite reste une note, un voisin reste une dyade.
            let strings =
                if voicing.len() == 3 { usize::from(voice.strings) } else { voicing.len() };
            let mut pitches = [0u8; 6];
            pitches[..voicing.len()].copy_from_slice(&voicing);
            for i in voicing.len()..strings {
                pitches[i] = pitches[i - voicing.len()] + 12;
            }
            let (first, count) = if hit.stroke == super::accomp_profile::Stroke::Up {
                let count = voice
                    .strum
                    .as_ref()
                    .map_or(strings, |s| usize::from(s.up_strings).min(strings));
                (strings - count, count)
            } else {
                (0, strings)
            };
            let pitches = &mut pitches[first..first + count];
            if hit.stroke == super::accomp_profile::Stroke::Up {
                pitches.reverse();
            }
            let velocities = super::strum::velocities(voice, hit, count, seed);
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
        chords: &[TradChord],
        key: PitchClass,
        mode: Mode,
        params: &TradHarmonyParams,
        hits: &[ChordHit],
        steps_per_chord: usize,
        seed: u64,
    ) {
        for hit in hits.iter().filter(|h| super::strum::selected(h, seed)) {
            let _ = self.voice(chords[hit.step / steps_per_chord], key, mode, params);
        }
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

    fn mechanical() -> TradHarmonyParams {
        let mut p = TradHarmonyParams::default();
        p.chords.strings = 3;
        p.chords.strum = None;
        p
    }

    fn chop(guitarist: &mut Guitarist, chords: [TradChord; 2]) -> Vec<TimelineNote> {
        guitarist.chop_bar(&chords, D, Mode::Ionian, &mechanical(), 8, 0, &mut ids())
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
        assert!(notes.iter().all(|n| n.velocity == params.bass.hits[0].velocity));
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
            let chop_velocity = TradHarmonyParams::default().chords.patterns[1].hits[0].velocity;
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
        let params = mechanical();
        let mut guitarist = Guitarist::new();
        let mut next_id = ids();
        for bar in 0..64 {
            let mode = Mode::ALL[bar % Mode::ALL.len()];
            let candidates = candidate_chords(mode, profile());
            let chords =
                [candidates[bar % candidates.len()], candidates[(bar * 3 + 1) % candidates.len()]];
            let notes = guitarist.chop_bar(&chords, D, mode, &params, 8, 0, &mut next_id);
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
        all.extend(Guitarist::new().chop_bar(
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

//! Ligne mélodique : cellules pondérées, continuité diatonique et contours
//! de registre par partie ; reprises et tours selon les règles du profil.

use super::{
    arrangement::Part,
    cells::{Cell, CellBank, CellNote, CellRole},
    mode::{degree_to_diatonic_step, diatonic_step_to_degree},
    plan::{ImposedRole, TunePlan},
};
use crate::{
    algo::RngCore,
    theory::PitchClass,
    timeline::{Articulation, TimelineNote},
    tuning::{ReelFiddleParams, TuningParams},
};

/// État d'une ligne de violon pour un tune en cours (D4, D6).
#[derive(Clone, Debug, PartialEq)]
pub struct FiddleLine {
    plan: TunePlan,
    parts: Vec<Vec<Cell>>,
    second_endings: Vec<Option<Cell>>,
    performed: Vec<Cell>,
    params: ReelFiddleParams,
    tonic_midi: u8,
    render_mode: super::Mode,
}

impl FiddleLine {
    /// Choisit le matériel des parties et variantes configurées avec les
    /// paramètres mélodiques par défaut.
    #[must_use]
    pub fn new(plan: TunePlan, bank: &CellBank, rng: &mut dyn RngCore) -> Self {
        Self::with_params(plan, bank, &ReelFiddleParams::default(), rng)
    }

    /// Construit une [`FiddleLine`] à partir d'un [`TuningParams`] complet
    /// (utilise `tuning.reel_cells` et `tuning.reel_fiddle`).
    #[must_use]
    pub fn from_tuning(plan: TunePlan, tuning: &TuningParams, rng: &mut dyn RngCore) -> Self {
        Self::with_params(plan, &tuning.profile.melody.cells, &tuning.profile.melody.params, rng)
    }

    /// Construit une [`FiddleLine`] avec des paramètres explicites [`ReelFiddleParams`].
    #[must_use]
    pub fn with_params(
        mut plan: TunePlan,
        bank: &CellBank,
        params: &ReelFiddleParams,
        rng: &mut dyn RngCore,
    ) -> Self {
        plan.resolve_roles(rng);
        let fallback_bank;
        let active_bank = if bank.cells.is_empty() {
            fallback_bank = CellBank::default();
            &fallback_bank
        } else {
            bank
        };

        let tonic_midi = tonic_midi_for_key(plan.key, params.tonic_midi_min, params.tonic_midi_max);

        let mut prev_last_step: Option<i8> = None;
        let mut parts = Vec::with_capacity(plan.profile.form.parts.len());
        while parts.len() < plan.profile.form.parts.len() {
            parts.push(select_strain_cells(
                &plan,
                Part(parts.len()),
                tonic_midi,
                active_bank,
                params,
                &mut prev_last_step,
                &parts,
                rng,
            ));
        }
        let second_endings = (0..parts.len())
            .map(|part| {
                plan.second_ending_role(part).map(|role| {
                    let chosen = &parts[part];
                    let last = chosen.len() - 1;
                    select_bar_cell(
                        Part(part),
                        last,
                        role,
                        &plan,
                        tonic_midi,
                        active_bank,
                        params,
                        &chosen[..last],
                        &parts,
                        chosen[..last]
                            .last()
                            .and_then(|c| c.last_degree())
                            .map(degree_to_diatonic_step),
                        chosen.first().and_then(|c| c.first_degree()).map(degree_to_diatonic_step),
                        rng,
                    )
                })
            })
            .collect();
        let mut line = Self {
            performed: Vec::new(),
            render_mode: plan.mode,
            plan,
            parts,
            second_endings,
            params: params.clone(),
            tonic_midi,
        };
        if line.plan.profile.form.expanded_material() {
            line.build_performed(active_bank, rng);
        }
        line
    }

    /// Plan structurel du tune associé à cette ligne de violon.
    #[must_use]
    pub fn plan(&self) -> TunePlan {
        self.plan.clone()
    }
    /// Relit les cellules choisies dans un mode parallèle, sans nouveau tirage.
    pub(crate) fn set_mode(&mut self, mode: super::Mode) {
        self.render_mode = mode;
    }

    /// Note MIDI de la tonique dans l'octave de référence (`G4` = 67 à `D5` = 74 avec les défauts P1-T7).
    #[must_use]
    pub const fn tonic_midi(&self) -> u8 {
        self.tonic_midi
    }

    /// Cellules distinctes d'une partie du profil.
    #[must_use]
    pub fn cells_for_part(&self, part: Part) -> &[Cell] {
        &self.parts[part.0]
    }

    #[must_use]
    pub fn cell_for_bar(&self, bar_in_tune: usize) -> &Cell {
        if !self.performed.is_empty() {
            return &self.performed[bar_in_tune % self.performed.len()];
        }
        let (part, _, offset, second_ending) = self.plan.position(bar_in_tune);
        if second_ending {
            self.second_endings[part]
                .as_ref()
                .unwrap_or_else(|| unreachable!("resolved second ending"))
        } else {
            &self.parts[part][offset]
        }
    }

    /// Génère les [`TimelineNote`] d'une mesure du tune (`bar_in_tune` 0-based).
    ///
    /// - 8 croches : `start_step = 2 * i`, `duration_steps = 2` ;
    /// - un [`CellNote::Hold`] prolonge la note précédente de 2 pas ;
    /// - hauteur : `tonic_midi + mode.degree_to_semitones(degree)`, repliée par
    ///   octave dans la tessiture `tessitura_min..=tessitura_max` (55..=88) ;
    /// - vélocités : `base_velocity` (84), croches 1 et 5 `+12`, croches paires `-10`.
    #[must_use]
    pub fn notes_for_bar(&self, bar_in_tune: usize) -> Vec<TimelineNote> {
        let cell = self.cell_for_bar(bar_in_tune);
        let mut notes = render_cell_notes(cell, &self.plan, self.tonic_midi, &self.params);
        if self.render_mode != self.plan.mode {
            for note in &mut notes {
                if let Some(degree) = self.plan.mode.scale_degree_of_midi(self.plan.key, note.pitch)
                {
                    let index = usize::from(degree - 1);
                    let delta =
                        self.render_mode.intervals()[index] - self.plan.mode.intervals()[index];
                    // Conserver l'octave, sauf hors du domaine physique MIDI.
                    note.pitch =
                        fold_midi_to_tessitura(i16::from(note.pitch) + i16::from(delta), 0, 127);
                }
            }
        }
        notes
    }

    /// Rend les mesures du premier tour de la forme configurée.
    #[must_use]
    pub fn render_tune(&self) -> Vec<Vec<TimelineNote>> {
        let total = self.plan.total_bars();
        let mut bars = Vec::with_capacity(total);
        for bar in 0..total {
            bars.push(self.notes_for_bar(bar));
        }
        bars
    }
}

impl FiddleLine {
    fn build_performed(&mut self, bank: &CellBank, rng: &mut dyn RngCore) {
        let bars = self.plan.total_bars();
        let mut played = Vec::with_capacity(self.plan.material_bars());
        for start in (0..self.plan.material_bars()).filter(|&b| self.plan.position(b).2 == 0) {
            let (part, pass, _, _) = self.plan.position(start);
            let tour = start / bars;
            let len = self.parts[part].len();
            let mut cells = if self.plan.profile.form.resolve
                == crate::style_profile::RoleResolution::PerPass
                && (pass > 1 || tour > 0)
            {
                let selection = self.plan.selection_plan(start);
                let mut previous = if selection.profile.form.parts[part].link_previous {
                    played.last().and_then(|c: &Cell| c.last_degree()).map(degree_to_diatonic_step)
                } else {
                    None
                };
                select_strain_cells(
                    &selection,
                    Part(part),
                    self.tonic_midi,
                    bank,
                    &self.params,
                    &mut previous,
                    &self.parts[..part],
                    rng,
                )
            } else if tour > 0 {
                played[start % bars..start % bars + len].to_vec()
            } else {
                let mut cells = self.parts[part].clone();
                if self.plan.position(start + len - 1).3
                    && let Some(ending) = &self.second_endings[part]
                {
                    cells[len - 1] = ending.clone();
                }
                cells
            };
            for (offset, cell) in cells.iter_mut().enumerate() {
                let role = self.plan.bar_role(start + offset).role;
                for probability in [
                    if pass > 1 && tour == 0 {
                        self.plan.profile.form.repeat_variation
                    } else {
                        0.0
                    },
                    if tour > 0 { self.plan.profile.form.tour_variation } else { 0.0 },
                ] {
                    if probability > 0.0 && (probability == 1.0 || rng.next_f32() < probability) {
                        vary_cell(cell, role, bank, &self.plan, self.tonic_midi, &self.params, rng);
                    }
                }
            }
            played.extend(cells);
        }
        self.performed = played;
    }
}

fn vary_cell(
    cell: &mut Cell,
    role: ImposedRole,
    bank: &CellBank,
    plan: &TunePlan,
    tonic: u8,
    params: &ReelFiddleParams,
    rng: &mut dyn RngCore,
) {
    use crate::style_profile::VariationOperation;
    let operations = &plan.profile.form.variation_operations;
    let first = rng.next_range_usize(0..operations.len());
    let pitch = |note: &CellNote| match note {
        CellNote::Degree(degree) => Some(fold_midi_to_tessitura(
            i16::from(tonic) + i16::from(plan.mode.degree_to_semitones(*degree)),
            params.tessitura_min,
            params.tessitura_max,
        )),
        CellNote::Hold => None,
    };
    for index in 0..operations.len() {
        let candidate = match operations[(first + index) % operations.len()] {
            VariationOperation::VariedEnd => {
                if cell.notes.len() < 2 {
                    continue;
                }
                let split = plan.profile.form.varied_end_split(cell.notes.len());
                if !bank.cells.iter().any(|c| {
                    matches!(c.notes[split], CellNote::Degree(_))
                        && !same_degree_classes(&c.notes[split..], &cell.notes[split..])
                }) {
                    continue;
                }
                let mut candidate = varied_end_cell(cell, bank, plan, tonic, params, rng);
                // CadFull protège aussi sa tonique tenue avant une levée finale.
                if matches!(
                    role,
                    ImposedRole::CadHalf
                        | ImposedRole::CadFull
                        | ImposedRole::CadTonic
                        | ImposedRole::CadOpen
                        | ImposedRole::CadFifth
                        | ImposedRole::CadOther
                ) && let Some(last) =
                    cell.notes.iter().rposition(|n| matches!(n, CellNote::Degree(degree)
                        if role != ImposedRole::CadFull || degree_to_diatonic_step(*degree).rem_euclid(7) == 0))
                {
                    candidate.notes[last..].copy_from_slice(&cell.notes[last..]);
                }
                candidate
            }
            VariationOperation::RepeatedNotes => {
                let last =
                    cell.notes.iter().rposition(|n| matches!(n, CellNote::Degree(_))).unwrap_or(0);
                let Some(i) = (1..last).find(|&i| {
                    matches!(cell.notes[i], CellNote::Degree(_))
                        && cell.notes[i] == cell.notes[i - 1]
                }) else {
                    continue;
                };
                let CellNote::Degree(degree) = cell.notes[i] else {
                    continue;
                };
                let mut candidate = cell.clone();
                candidate.notes[i] =
                    CellNote::Degree(diatonic_step_to_degree(degree_to_diatonic_step(degree) + 1));
                candidate
            }
            VariationOperation::Octave => {
                let Some(octave) = [1, -1].into_iter().find(|&octave| {
                    cell.notes.iter().all(|note| match note {
                        CellNote::Degree(degree) => {
                            let shifted = diatonic_step_to_degree(
                                degree_to_diatonic_step(*degree).saturating_add(7 * octave),
                            );
                            (i16::from(params.tessitura_min)..=i16::from(params.tessitura_max))
                                .contains(
                                    &(i16::from(tonic)
                                        + i16::from(plan.mode.degree_to_semitones(shifted))),
                                )
                        }
                        CellNote::Hold => true,
                    })
                }) else {
                    continue;
                };
                transpose_cell_octaves(cell, octave)
            }
        };
        if candidate.notes.iter().map(&pitch).ne(cell.notes.iter().map(&pitch)) {
            *cell = candidate;
            return;
        }
    }
}

/// Choisit la note MIDI de la tonique dans la plage `[min_midi, max_midi]`
/// (par défaut `F#4` = 66 à `F5` = 77, soit `D5` = 74 pour ré, P1-T7).
#[must_use]
pub const fn tonic_midi_for_key(key: PitchClass, min_midi: u8, max_midi: u8) -> u8 {
    let key_pc = key % 12;
    let min_pc = min_midi % 12;
    let offset = (12 + key_pc - min_pc) % 12;
    let candidate = min_midi.saturating_add(offset);
    if candidate <= max_midi { candidate } else { min_midi }
}

/// Replie une hauteur MIDI brute par octaves (`±12` demi-tons) pour la maintenir
/// dans la tessiture `[min_midi, max_midi]` (par défaut `G3` = 55 à `E6` = 88).
#[must_use]
pub const fn fold_midi_to_tessitura(mut raw_midi: i16, min_midi: u8, max_midi: u8) -> u8 {
    let lo = min_midi as i16;
    let hi = max_midi as i16;
    if hi >= lo + 12 {
        while raw_midi < lo {
            raw_midi += 12;
        }
        while raw_midi > hi {
            raw_midi -= 12;
        }
    } else if raw_midi < lo {
        raw_midi = lo;
    } else if raw_midi > hi {
        raw_midi = hi;
    }
    if raw_midi < 0 {
        0
    } else if raw_midi > 127 {
        127
    } else {
        raw_midi as u8
    }
}

/// Calcule la vélocité MIDI d'une croche (`eighth_index` 0-based dans `0..8`) :
/// - croches 1 et 5 (indices 0 et 4) : `base_velocity + strong_eighth_velocity_offset` (`+12`) ;
/// - croches paires 2, 4, 6, 8 (indices impairs 1, 3, 5, 7) : `base_velocity + even_eighth_velocity_offset` (`-10`) ;
/// - croches 3 et 7 (indices 2 et 6) : `base_velocity`.
#[must_use]
pub fn unit_velocity(
    eighth_index: usize,
    params: &ReelFiddleParams,
    melody: &crate::style_profile::ProfileMelody,
) -> u8 {
    let base = params.base_velocity as i16;
    let delta = if melody.strong_units.contains(&eighth_index) {
        params.strong_eighth_velocity_offset as i16
    } else if melody.weak_units.contains(&eighth_index) {
        params.even_eighth_velocity_offset as i16
    } else {
        0
    };
    let v = base + delta;
    if v < 1 {
        1
    } else if v > 127 {
        127
    } else {
        v as u8
    }
}

/// Transpose tous les degrés d'une [`Cell`] de `octave_shift` octaves (`7 * octave_shift` pas diatoniques).
#[must_use]
pub fn transpose_cell_octaves(cell: &Cell, octave_shift: i8) -> Cell {
    if octave_shift == 0 {
        return cell.clone();
    }
    let step_offset = octave_shift.saturating_mul(7);
    let mut notes = cell.notes.clone();
    for note in &mut notes {
        if let CellNote::Degree(deg) = *note {
            let shifted_step = degree_to_diatonic_step(deg).saturating_add(step_offset);
            *note = CellNote::Degree(diatonic_step_to_degree(shifted_step));
        }
    }
    Cell { role: cell.role, notes, weight: cell.weight }
}

/// Convertit une [`Cell`] en vecteur de [`TimelineNote`] pour une mesure.
fn render_cell_notes(
    cell: &Cell,
    plan: &TunePlan,
    tonic_midi: u8,
    params: &ReelFiddleParams,
) -> Vec<TimelineNote> {
    let mut notes: Vec<TimelineNote> = Vec::with_capacity(cell.notes.len());

    for (i, &cell_note) in cell.notes.iter().enumerate() {
        match cell_note {
            CellNote::Degree(deg) => {
                let raw_midi =
                    i16::from(tonic_midi) + i16::from(plan.mode.degree_to_semitones(deg));
                let pitch =
                    fold_midi_to_tessitura(raw_midi, params.tessitura_min, params.tessitura_max);
                let velocity = unit_velocity(i, params, &plan.profile.melody);
                let id = notes.len() as u64;
                notes.push(TimelineNote {
                    id,
                    pitch,
                    start_step: plan.profile.meter.steps_per_unit * i,
                    duration_steps: plan.profile.meter.steps_per_unit,
                    velocity,
                    articulation: Articulation::Normal,
                    performance: Default::default(),
                });
            }
            CellNote::Hold => {
                if let Some(prev) = notes.last_mut() {
                    prev.duration_steps += plan.profile.meter.steps_per_unit;
                }
            }
        }
    }

    notes
}

/// Choisit une partie après les parties déjà construites (références validées).
#[allow(clippy::too_many_arguments)]
fn select_strain_cells(
    plan: &TunePlan,
    strain: Part,
    tonic_midi: u8,
    bank: &CellBank,
    params: &ReelFiddleParams,
    prev_last_step: &mut Option<i8>,
    previous_parts: &[Vec<Cell>],
    rng: &mut dyn RngCore,
) -> Vec<Cell> {
    let mut chosen: Vec<Cell> = Vec::with_capacity(plan.profile.form.parts[strain.0].roles.len());
    let mut bar1_first_step: Option<i8> = None;
    if !plan.profile.form.parts[strain.0].link_previous {
        *prev_last_step = None;
    }

    let roles = plan.roles_for_part(strain.0);
    for (bar_idx, &bar_role) in roles.iter().enumerate() {
        let cell = select_bar_cell(
            strain,
            bar_idx,
            bar_role,
            plan,
            tonic_midi,
            bank,
            params,
            &chosen,
            previous_parts,
            *prev_last_step,
            bar1_first_step,
            rng,
        );

        if bar_idx == 0 {
            bar1_first_step = cell.first_degree().map(degree_to_diatonic_step);
        }
        if let Some(last_deg) = cell.last_degree() {
            *prev_last_step = Some(degree_to_diatonic_step(last_deg));
        }
        chosen.push(cell);
    }

    chosen
}

#[allow(clippy::too_many_arguments)]
fn select_bar_cell(
    strain: Part,
    bar_idx: usize,
    bar_role: ImposedRole,
    plan: &TunePlan,
    tonic_midi: u8,
    bank: &CellBank,
    params: &ReelFiddleParams,
    chosen: &[Cell],
    previous_parts: &[Vec<Cell>],
    prev_last_step: Option<i8>,
    bar1_first_step: Option<i8>,
    rng: &mut dyn RngCore,
) -> Cell {
    let mut cell = match bar_role {
        ImposedRole::CopyOf(source) => chosen[usize::from(source) - 1].clone(),
        ImposedRole::CopyOfPart { part, bar } => previous_parts[part][usize::from(bar) - 1].clone(),
        ImposedRole::SequenceOf { bar, shift } => {
            sequence_cell(&chosen[usize::from(bar) - 1], shift, plan, tonic_midi, params)
        }
        ImposedRole::VariedEndOf(bar) => {
            varied_end_cell(&chosen[usize::from(bar) - 1], bank, plan, tonic_midi, params, rng)
        }
        imposed => {
            let excluded_cell = if plan.profile.form.avoid_adjacent
                && imposed == ImposedRole::Free
                && bar_idx > 0
            {
                Some(&chosen[bar_idx - 1])
            } else {
                None
            };
            pick_cell_for_bar(
                strain,
                bar_idx,
                imposed,
                plan,
                tonic_midi,
                bank,
                params,
                prev_last_step,
                bar1_first_step,
                excluded_cell,
                rng,
            )
        }
    };
    if matches!(
        bar_role,
        ImposedRole::Open
            | ImposedRole::Free
            | ImposedRole::CadOpen
            | ImposedRole::CadOther
            | ImposedRole::CadHalf
            | ImposedRole::CadFifth
    ) && let Some(rhythm) =
        plan.profile.form.parts[strain.0].rhythm.iter().find(|r| r.bar == bar_idx + 1)
    {
        apply_rhythm(&mut cell, rhythm, rng);
    }
    if matches!(
        bar_role,
        ImposedRole::CadTonic
            | ImposedRole::CadOpen
            | ImposedRole::CadFifth
            | ImposedRole::CadOther
    ) {
        shape_cadence(&mut cell, bar_role, &plan.profile.form);
    }
    cell
}

fn apply_rhythm(
    cell: &mut Cell,
    rhythm: &crate::style_profile::PositionRhythm,
    rng: &mut dyn RngCore,
) {
    let total: usize = rhythm.patterns.iter().map(|p| p.weight as usize).sum();
    let mut pick = rng.next_range_usize(0..total);
    let pattern = rhythm
        .patterns
        .iter()
        .find(|p| {
            if pick < p.weight as usize {
                true
            } else {
                pick -= p.weight as usize;
                false
            }
        })
        .unwrap_or_else(|| unreachable!("validated rhythm weights"));
    let source = cell.notes.clone();
    let last = cell.last_degree().unwrap_or(1);
    cell.notes.fill(CellNote::Hold);
    let mut offset = 0;
    for (index, &duration) in pattern.durations.iter().enumerate() {
        let degree = if index + 1 == pattern.durations.len() {
            last
        } else {
            source[..=offset]
                .iter()
                .rev()
                .find_map(|n| match n {
                    CellNote::Degree(d) => Some(*d),
                    CellNote::Hold => None,
                })
                .unwrap_or(last)
        };
        cell.notes[offset] = CellNote::Degree(degree);
        offset += duration;
    }
}

/// Déplacement diatonique uniforme ; une octave commune préserve le contour.
fn sequence_cell(
    source: &Cell,
    shift: i8,
    plan: &TunePlan,
    tonic: u8,
    params: &ReelFiddleParams,
) -> Cell {
    let octave = fitting_octave(source, shift, plan, tonic, params);
    let mut cell = source.clone();
    shift_notes(&mut cell.notes, shift + 7 * octave);
    cell
}

fn shift_notes(notes: &mut [CellNote], shift: i8) {
    for note in notes {
        if let CellNote::Degree(degree) = note {
            *degree = diatonic_step_to_degree(degree_to_diatonic_step(*degree) + shift);
        }
    }
}

fn fitting_octave(
    source: &Cell,
    shift: i8,
    plan: &TunePlan,
    tonic: u8,
    params: &ReelFiddleParams,
) -> i8 {
    (-4i8..=4)
        .min_by_key(|&octave| {
            source
                .notes
                .iter()
                .filter_map(|note| match note {
                    CellNote::Degree(degree) => {
                        let degree = diatonic_step_to_degree(
                            degree_to_diatonic_step(*degree) + shift + 7 * octave,
                        );
                        let pitch =
                            i16::from(tonic) + i16::from(plan.mode.degree_to_semitones(degree));
                        Some(
                            (i16::from(params.tessitura_min) - pitch).max(0)
                                + (pitch - i16::from(params.tessitura_max)).max(0),
                        )
                    }
                    CellNote::Hold => None,
                })
                .sum::<i16>()
                * 16
                + i16::from(octave).abs()
        })
        .unwrap_or(0)
}

fn varied_end_cell(
    source: &Cell,
    bank: &CellBank,
    plan: &TunePlan,
    tonic: u8,
    params: &ReelFiddleParams,
    rng: &mut dyn RngCore,
) -> Cell {
    let split = plan.profile.form.varied_end_split(source.notes.len());
    let eligible = |cell: &&Cell| {
        matches!(cell.notes[split], CellNote::Degree(_))
            && !same_degree_classes(&cell.notes[split..], &source.notes[split..])
    };
    let total: u64 = bank.cells.iter().filter(eligible).map(|cell| u64::from(cell.weight)).sum();
    let mut pick = rng.next_range_usize(0..total as usize) as u64;
    for candidate in bank.cells.iter().filter(eligible) {
        if pick < u64::from(candidate.weight) {
            let octave = fitting_octave(candidate, 0, plan, tonic, params);
            let mut cell = source.clone();
            cell.notes[split..].copy_from_slice(&candidate.notes[split..]);
            shift_notes(&mut cell.notes[split..], 7 * octave);
            return cell;
        }
        pick -= u64::from(candidate.weight);
    }
    unreachable!("validated distinct compatible endings")
}

pub(crate) fn same_degree_classes(left: &[CellNote], right: &[CellNote]) -> bool {
    left.iter().zip(right).all(|(a, b)| match (a, b) {
        (CellNote::Hold, CellNote::Hold) => true,
        (CellNote::Degree(a), CellNote::Degree(b)) => {
            degree_to_diatonic_step(*a).rem_euclid(7) == degree_to_diatonic_step(*b).rem_euclid(7)
        }
        _ => false,
    })
}

fn shape_cadence(cell: &mut Cell, role: ImposedRole, form: &crate::style_profile::ProfileForm) {
    if role == ImposedRole::CadTonic {
        let minimum = form.cadence_min_units.min(cell.notes.len());
        let search = (cell.notes.len() as f32 * form.cadence_search_from) as usize;
        let start = (search..=cell.notes.len().saturating_sub(minimum))
            .rev()
            .find(|&index| {
                matches!(cell.notes[index], CellNote::Degree(degree)
                if degree_to_diatonic_step(degree).rem_euclid(7) == 0)
                    && cell.notes[index + 1..index + minimum].iter().all(|n| *n == CellNote::Hold)
            })
            .unwrap_or_else(|| cell.notes.len().saturating_sub(minimum));
        let previous = cell.notes[..=start]
            .iter()
            .rev()
            .find_map(|note| match note {
                CellNote::Degree(d) => Some(degree_to_diatonic_step(*d)),
                CellNote::Hold => None,
            })
            .unwrap_or(0);
        let tonic = ((f32::from(previous) / 7.0).round() as i8) * 7;
        cell.notes[start] = CellNote::Degree(diatonic_step_to_degree(tonic));
        for note in &mut cell.notes[start + 1..] {
            *note = CellNote::Hold;
        }
    } else if let Some(CellNote::Degree(degree)) =
        cell.notes.iter_mut().rev().find(|note| matches!(note, CellNote::Degree(_)))
    {
        let step = degree_to_diatonic_step(*degree);
        let class = step.rem_euclid(7);
        if role == ImposedRole::CadFifth {
            let octave = ((f32::from(step - 4) / 7.0).round() as i8) * 7;
            *degree = diatonic_step_to_degree(octave + 4);
        } else if class == 0 || (role == ImposedRole::CadOther && class == 4) {
            *degree = diatonic_step_to_degree(step + 1);
        }
    }
}

/// Évaluation d'une cellule candidate transposée pour le tri de repli.
#[derive(Clone)]
struct EvaluatedCandidate {
    cell: Cell,
    /// `0` si toutes les notes brutes tombent dans `tessitura_min..=tessitura_max`, `1` sinon.
    tessitura_penalty: u8,
    /// Excès de saut au-delà d'une quinte (continuité d'entrée + rebouclage sur la mesure 1).
    fifth_excess: i16,
    /// `0` si le centre est dans le registre propre de la strain (A vs B), `1` sinon.
    register_penalty: u8,
    /// Excès au-delà du seuil strict de continuité (`max_continuity_step_distance` = tierce).
    continuity_excess: i16,
    /// Distance en pas diatoniques entre le premier degré et le dernier degré précédent.
    continuity_distance: i16,
    /// Distance absolue (en dixièmes de demi-ton) entre le centre rendu de la
    /// cellule et la cible de la strain (`melody_center_midi`, + lift en B).
    contour_distance_tenths: i32,
}

impl EvaluatedCandidate {
    const fn score_key(&self) -> (u8, i16, u8, i16, i16, i32) {
        (
            self.tessitura_penalty,
            self.fifth_excess,
            self.register_penalty,
            self.continuity_excess,
            self.continuity_distance,
            self.contour_distance_tenths,
        )
    }

    fn is_strict_match(&self, plan: &TunePlan) -> bool {
        self.tessitura_penalty == 0
            && self.fifth_excess == 0
            && self.register_penalty == 0
            && self.continuity_excess == 0
            && self.contour_distance_tenths
                <= (plan.profile.form.contour_window_semitones * 10.0).round() as i32
    }
}

#[allow(clippy::too_many_arguments)]
fn pick_cell_for_bar(
    strain: Part,
    bar_idx: usize,
    imposed: ImposedRole,
    plan: &TunePlan,
    tonic_midi: u8,
    bank: &CellBank,
    params: &ReelFiddleParams,
    prev_last_step: Option<i8>,
    bar1_first_step: Option<i8>,
    excluded_cell: Option<&Cell>,
    rng: &mut dyn RngCore,
) -> Cell {
    let target_midi = strain_target_midi(strain, plan, params);
    let split_midi = register_split_midi(plan, params);
    let max_retries = params.continuity_max_retries.max(1);

    let mut best: Option<EvaluatedCandidate> = None;

    for _ in 0..max_retries {
        let role = match imposed.forced_cell_role() {
            Some(forced) => forced,
            None => draw_free_cell_role(params, rng),
        };
        let Some(base_cell) = draw_cell_by_role(bank, role, excluded_cell, rng) else {
            continue;
        };

        for shift in contour_shifts_for_cell(base_cell, plan, tonic_midi, target_midi) {
            let cand = evaluate_candidate(
                base_cell,
                shift,
                strain,
                bar_idx,
                target_midi,
                split_midi,
                plan,
                tonic_midi,
                params,
                prev_last_step,
                bar1_first_step,
            );
            if cand.is_strict_match(plan) {
                return cand.cell;
            }
            if best.as_ref().is_none_or(|b| cand.score_key() < b.score_key()) {
                best = Some(cand);
            }
        }
    }

    // Repli déterministe sur la candidate la plus proche parmi les rôles permis (D6 : jamais
    // d'échec) : le rôle imposé, sinon les rôles libres de poids positif.
    let allowed = |role: CellRole| match imposed.forced_cell_role() {
        Some(forced) => role == forced,
        None => params.free_role_weights().iter().any(|&(r, w)| r == role && w > 0),
    };

    for base_cell in &bank.cells {
        if !allowed(base_cell.role) || !is_eligible_cell(base_cell, excluded_cell) {
            continue;
        }
        for shift in contour_shifts_for_cell(base_cell, plan, tonic_midi, target_midi) {
            let cand = evaluate_candidate(
                base_cell,
                shift,
                strain,
                bar_idx,
                target_midi,
                split_midi,
                plan,
                tonic_midi,
                params,
                prev_last_step,
                bar1_first_step,
            );
            if best.as_ref().is_none_or(|b| cand.score_key() < b.score_key()) {
                best = Some(cand);
            }
        }
    }

    if let Some(winner) = best {
        return winner.cell;
    }

    // Repli ultime si toutes les cellules permises étaient exclues (ex. banque à 1 cellule).
    for base_cell in &bank.cells {
        if !allowed(base_cell.role) || !base_cell.respects_cadence() {
            continue;
        }
        for shift in contour_shifts_for_cell(base_cell, plan, tonic_midi, target_midi) {
            let cand = evaluate_candidate(
                base_cell,
                shift,
                strain,
                bar_idx,
                target_midi,
                split_midi,
                plan,
                tonic_midi,
                params,
                prev_last_step,
                bar1_first_step,
            );
            if best.as_ref().is_none_or(|b| cand.score_key() < b.score_key()) {
                best = Some(cand);
            }
        }
    }

    if let Some(winner) = best {
        return winner.cell;
    }

    bank.cells[0].clone()
}

/// Tire un rôle libre parmi `Run`, `Arp`, `Turn` selon les poids de [`ReelFiddleParams`].
fn draw_free_cell_role(params: &ReelFiddleParams, rng: &mut dyn RngCore) -> CellRole {
    let candidates = params.free_role_weights();
    let total: u32 = candidates.iter().map(|&(_, w)| w).sum();
    if total == 0 {
        return CellRole::Run;
    }
    let mut pick = rng.next_range_usize(0..(total as usize)) as u32;
    for &(role, weight) in &candidates {
        if weight > 0 {
            if pick < weight {
                return role;
            }
            pick -= weight;
        }
    }
    CellRole::Run
}

/// Une cellule est tirable si elle respecte la cadence de son rôle (D4,
/// [`Cell::respects_cadence`]) et n'est pas la cellule exclue.
fn is_eligible_cell(candidate: &Cell, excluded_cell: Option<&Cell>) -> bool {
    candidate.respects_cadence() && !is_excluded_cell(candidate, excluded_cell)
}

/// Indique si `candidate` correspond à la cellule exclue (à une transposition d'octave près).
fn is_excluded_cell(candidate: &Cell, excluded_cell: Option<&Cell>) -> bool {
    excluded_cell.is_some_and(|excluded| same_cell_up_to_octave(candidate, excluded))
}

/// Vérifie si deux cellules ont exactement les mêmes degrés (à un décalage d'octave constant près)
/// et les mêmes tenues (`Hold`).
#[must_use]
pub fn same_cell_up_to_octave(a: &Cell, b: &Cell) -> bool {
    if a.notes.len() != b.notes.len() {
        return false;
    }
    let mut step_diff: Option<i8> = None;
    let mut i = 0;
    while i < a.notes.len() {
        match (a.notes[i], b.notes[i]) {
            (CellNote::Hold, CellNote::Hold) => {}
            (CellNote::Degree(da), CellNote::Degree(db)) => {
                let diff = degree_to_diatonic_step(da) - degree_to_diatonic_step(db);
                if diff % 7 != 0 {
                    return false;
                }
                if let Some(expected) = step_diff {
                    if diff != expected {
                        return false;
                    }
                } else {
                    step_diff = Some(diff);
                }
            }
            _ => return false,
        }
        i += 1;
    }
    true
}

/// Tire une cellule ayant le rôle `role` dans `bank` en excluant `excluded_cell`,
/// pondérée par [`Cell::weight`] (un seul tirage RNG). Repli sur toutes les
/// cellules si la banque n'a aucune cellule de ce rôle.
fn draw_cell_by_role<'a>(
    bank: &'a CellBank,
    role: CellRole,
    excluded_cell: Option<&Cell>,
    rng: &mut dyn RngCore,
) -> Option<&'a Cell> {
    let eligible = |c: &Cell| is_eligible_cell(c, excluded_cell);
    if let Some(cell) = draw_weighted(&bank.cells, |c| c.role == role && eligible(c), rng) {
        return Some(cell);
    }
    if bank.count_by_role(role) == 0 {
        return draw_weighted(&bank.cells, eligible, rng);
    }
    None
}

/// Tirage pondéré par [`Cell::weight`] parmi les cellules de `cells` retenues par
/// `keep` (deux passes, sans allocation, un seul tirage RNG).
fn draw_weighted<'a>(
    cells: &'a [Cell],
    keep: impl Fn(&Cell) -> bool,
    rng: &mut dyn RngCore,
) -> Option<&'a Cell> {
    let total: usize = cells.iter().filter(|c| keep(c)).map(|c| c.weight as usize).sum();
    if total == 0 {
        return None;
    }
    let mut pick = rng.next_range_usize(0..total);
    for cell in cells.iter().filter(|c| keep(c)) {
        let weight = cell.weight as usize;
        if pick < weight {
            return Some(cell);
        }
        pick -= weight;
    }
    None
}

/// Centre cible en hauteurs MIDI pour une strain (A : `melody_center_midi`,
/// B : `melody_center_midi + strain_b_lift_semitones`, P1-T7).
fn strain_target_midi(strain: Part, plan: &TunePlan, params: &ReelFiddleParams) -> f32 {
    f32::from(params.melody_center_midi)
        + f32::from(plan.profile.form.parts[strain.0].melody_center_offset)
}

/// Frontière médiane en hauteurs MIDI séparant le registre de la strain A et
/// celui de la strain B (`melody_center_midi + lift / 2`, P1-T7).
fn register_split_midi(plan: &TunePlan, params: &ReelFiddleParams) -> f32 {
    let offsets = plan.profile.form.parts.iter().map(|p| p.melody_center_offset);
    let lo = offsets.clone().min().unwrap_or(0);
    let hi = offsets.max().unwrap_or(0);
    f32::from(params.melody_center_midi) + 0.5 * (f32::from(lo) + f32::from(hi))
}

/// Décalages d'octave candidats pour viser la cible de strain en hauteurs
/// absolues, triés par proximité du centre rendu à `target_midi` : le premier
/// tirage strict trouvé est donc le plus proche de la cible (P1-T7).
fn contour_shifts_for_cell(
    cell: &Cell,
    plan: &TunePlan,
    tonic_midi: u8,
    target_midi: f32,
) -> [i8; 5] {
    let base = cell_mean_midi(cell, plan, tonic_midi);
    let mut shifts: [i8; 5] = [-2, -1, 0, 1, 2];
    shifts
        .sort_by_key(|&s| ((base + 12.0 * f32::from(s) - target_midi).abs() * 10.0).round() as i32);
    shifts
}

/// Calcule le centre moyen d'une cellule rendue en hauteurs MIDI
/// (`tonic_midi + mode.degree_to_semitones`, tenues exclues).
fn cell_mean_midi(cell: &Cell, plan: &TunePlan, tonic_midi: u8) -> f32 {
    let mut sum = 0i32;
    let mut count = 0i32;
    for &note in &cell.notes {
        if let CellNote::Degree(deg) = note {
            sum += i32::from(tonic_midi) + i32::from(plan.mode.degree_to_semitones(deg));
            count += 1;
        }
    }
    if count == 0 { f32::from(tonic_midi) } else { (sum as f32) / (count as f32) }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_candidate(
    base_cell: &Cell,
    octave_shift: i8,
    strain: Part,
    bar_idx: usize,
    target_midi: f32,
    split_midi: f32,
    plan: &TunePlan,
    tonic_midi: u8,
    params: &ReelFiddleParams,
    prev_last_step: Option<i8>,
    bar1_first_step: Option<i8>,
) -> EvaluatedCandidate {
    let cell = transpose_cell_octaves(base_cell, octave_shift);
    let mean_midi = cell_mean_midi(&cell, plan, tonic_midi);

    let mut all_in_tessitura = true;
    for &note in &cell.notes {
        if let CellNote::Degree(deg) = note {
            let raw = i16::from(tonic_midi) + i16::from(plan.mode.degree_to_semitones(deg));
            if raw < i16::from(params.tessitura_min) || raw > i16::from(params.tessitura_max) {
                all_in_tessitura = false;
                break;
            }
        }
    }
    let tessitura_penalty = u8::from(!all_in_tessitura);

    // Tolérance autour de la frontière de registre : rejette les erreurs
    // d'octave sans forcer de transposition (le contour vise la cible, P1-T7).
    let in_register = if target_midi <= split_midi {
        mean_midi <= split_midi + plan.profile.form.register_tolerance_semitones
    } else {
        mean_midi >= split_midi - plan.profile.form.register_tolerance_semitones
    };
    let register_penalty = u8::from(!in_register);

    let first_step = cell.first_degree().map_or(0, degree_to_diatonic_step);
    let last_step = cell.last_degree().map_or(first_step, degree_to_diatonic_step);

    let continuity_distance =
        prev_last_step.map_or(0, |prev| (i16::from(first_step) - i16::from(prev)).abs());
    let max_cont = i16::from(params.max_continuity_step_distance);
    let continuity_excess = (continuity_distance - max_cont).max(0);

    let fifth_steps = i16::from(plan.profile.form.max_leap_degrees);
    let entry_fifth_excess = (continuity_distance - fifth_steps).max(0);
    // Les mesures 4 (index 3, avant la copie de la mesure 1 en mesure 5) et 8
    // (index 7, avant la reprise de la mesure 1 en passe 2) doivent aussi
    // rester à ≤ une quinte du premier degré de la mesure 1.
    let roles = plan.roles_for_part(strain.0);
    let loop_fifth_excess = if ((plan.profile.form.close_part_loop && bar_idx + 1 == roles.len())
        || roles.get(bar_idx + 1) == Some(&ImposedRole::CopyOf(1)))
        && let Some(b1_first) = bar1_first_step
    {
        ((i16::from(last_step) - i16::from(b1_first)).abs() - fifth_steps).max(0)
    } else {
        0
    };
    let fifth_excess = entry_fifth_excess + loop_fifth_excess;

    let contour_distance_tenths = ((mean_midi - target_midi).abs() * 10.0).round() as i32;

    EvaluatedCandidate {
        cell,
        tessitura_penalty,
        fifth_excess,
        register_penalty,
        continuity_excess,
        continuity_distance,
        contour_distance_tenths,
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::trad::mode::Mode;

    #[test]
    fn weighted_draw_favors_heavier_cells_in_proportion() {
        let bank = CellBank::default();
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        for role in CellRole::ALL {
            let cells: Vec<&Cell> = bank.by_role(role).filter(|c| c.respects_cadence()).collect();
            let (Some(heavy), Some(light)) = (
                cells.iter().copied().max_by_key(|c| c.weight),
                cells.iter().copied().min_by_key(|c| c.weight),
            ) else {
                panic!("rôle {role:?} sans cellule");
            };
            assert!(heavy.weight > light.weight, "poids uniformes pour {role:?}");

            let (mut heavy_count, mut light_count) = (0u32, 0u32);
            for _ in 0..10_000 {
                let Some(drawn) = draw_cell_by_role(&bank, role, None, &mut rng) else {
                    panic!("aucun tirage pour {role:?}");
                };
                if drawn == heavy {
                    heavy_count += 1;
                } else if drawn == light {
                    light_count += 1;
                }
            }
            let expected = f64::from(heavy.weight) / f64::from(light.weight);
            let observed = f64::from(heavy_count) / f64::from(light_count);
            assert!(
                (observed / expected - 1.0).abs() <= 0.30,
                "{role:?} : rapport observé {observed:.2} vs poids {expected:.2}"
            );
        }
    }

    #[test]
    fn tonic_midi_falls_in_g3_to_fsharp4() {
        // D -> D4 = 62, G -> G3 = 55, A -> A3 = 57, C -> C4 = 60, F# -> F#4 = 66
        assert_eq!(tonic_midi_for_key(2, 55, 66), 62);
        assert_eq!(tonic_midi_for_key(7, 55, 66), 55);
        assert_eq!(tonic_midi_for_key(9, 55, 66), 57);
        assert_eq!(tonic_midi_for_key(0, 55, 66), 60);
        assert_eq!(tonic_midi_for_key(6, 55, 66), 66);

        for pc in 0..12 {
            let midi = tonic_midi_for_key(pc, 55, 66);
            assert!((55..=66).contains(&midi));
            assert_eq!(midi % 12, pc);
        }
    }

    #[test]
    fn default_tonic_octaves_sit_in_upper_register() {
        let params = ReelFiddleParams::default();
        // D -> D5 = 74, G -> G4 = 67, A -> A4 = 69 (P1-T7 : toniques à hauteur de chant).
        assert_eq!(tonic_midi_for_key(2, params.tonic_midi_min, params.tonic_midi_max), 74);
        assert_eq!(tonic_midi_for_key(7, params.tonic_midi_min, params.tonic_midi_max), 67);
        assert_eq!(tonic_midi_for_key(9, params.tonic_midi_min, params.tonic_midi_max), 69);
    }

    #[test]
    fn hold_extends_previous_note_duration_by_2_steps() {
        let parsed = Cell::from_notation(CellRole::CadFull, "5 4 3 2 1 - 5, -");
        assert!(parsed.is_ok());
        if let Ok(cell) = parsed {
            let plan = TunePlan::new(2, Mode::Ionian);
            let params = ReelFiddleParams::default();
            let notes = render_cell_notes(&cell, &plan, 62, &params);

            assert_eq!(notes.len(), 6);
            assert_eq!(notes[0].start_step, 0);
            assert_eq!(notes[0].duration_steps, 2);
            assert_eq!(notes[4].start_step, 8);
            assert_eq!(notes[4].duration_steps, 4);
            assert_eq!(notes[4].pitch, 62); // D4
            assert_eq!(notes[5].start_step, 12);
            assert_eq!(notes[5].duration_steps, 4);
            assert_eq!(notes[5].pitch, 57); // A3 (5,)
        }
    }

    #[test]
    fn empty_bank_falls_back_without_failure() {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let empty_bank = CellBank { cells: Vec::new() };
        let plan = TunePlan::with_profile(2, Mode::Ionian, std::sync::Arc::new(Default::default()));
        let line = FiddleLine::new(plan.clone(), &empty_bank, &mut rng);
        let tune = line.render_tune();
        assert_eq!(tune.len(), 32);
        assert!(tune.iter().all(|bar| !bar.is_empty()));
    }
}

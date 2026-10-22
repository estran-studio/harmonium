//! Ligne mélodique : sélection pondérée des cellules par rôles du profil,
//! continuité diatonique et contours de registre par partie. Les reprises
//! relisent les mêmes degrés, éventuellement dans un mode parallèle.

use super::{
    arrangement::Part,
    cells::{Cell, CellBank, CellNote, CellRole, FIFTH_DIATONIC_STEPS},
    mode::{PitchClass, degree_to_diatonic_step, diatonic_step_to_degree},
    plan::{ImposedRole, TunePlan},
};
use crate::{
    harmony::RngCore,
    timeline::{Articulation, TimelineNote},
    tuning::{ReelFiddleParams, TuningParams},
};

/// État d'une ligne de violon pour un tune en cours (D4, D6).
#[derive(Clone, Debug, PartialEq)]
pub struct FiddleLine {
    plan: TunePlan,
    parts: Vec<Vec<Cell>>,
    params: ReelFiddleParams,
    tonic_midi: u8,
    render_mode: super::Mode,
}

impl FiddleLine {
    /// Construit une nouvelle [`FiddleLine`] en choisissant immédiatement les
    /// 16 cellules des deux strains selon D6 avec [`ReelFiddleParams::default`].
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
        plan: TunePlan,
        bank: &CellBank,
        params: &ReelFiddleParams,
        rng: &mut dyn RngCore,
    ) -> Self {
        let fallback_bank;
        let active_bank = if bank.cells.is_empty() {
            fallback_bank = CellBank::default();
            &fallback_bank
        } else {
            bank
        };

        let tonic_midi = tonic_midi_for_key(plan.key, params.tonic_midi_min, params.tonic_midi_max);

        let mut prev_last_step: Option<i8> = None;
        let parts = (0..plan.profile.form.parts.len())
            .map(|part| {
                select_strain_cells(
                    Part(part),
                    &plan,
                    tonic_midi,
                    active_bank,
                    params,
                    &mut prev_last_step,
                    rng,
                )
            })
            .collect();
        Self { render_mode: plan.mode, plan, parts, params: params.clone(), tonic_midi }
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
        let role = self.plan.bar_role(bar_in_tune);
        &self.parts[role.strain.0][usize::from(role.bar_in_strain) - 1]
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

    /// Rend les 32 mesures complètes du tune (fonction de commodité pour les tests).
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

/// Choisit les 8 cellules d'une strain selon les règles D4 et D6.
fn select_strain_cells(
    strain: Part,
    plan: &TunePlan,
    tonic_midi: u8,
    bank: &CellBank,
    params: &ReelFiddleParams,
    prev_last_step: &mut Option<i8>,
    rng: &mut dyn RngCore,
) -> Vec<Cell> {
    let mut chosen: Vec<Cell> = Vec::with_capacity(plan.profile.form.parts[strain.0].roles.len());
    let mut bar1_first_step: Option<i8> = None;

    let roles = &plan.profile.form.parts[strain.0].roles;
    for (bar_idx, &bar_role) in roles.iter().enumerate() {
        let cell = match bar_role {
            ImposedRole::CopyOf(src_1based) => {
                let src_idx = usize::from(src_1based) - 1;
                chosen[src_idx].clone()
            }
            imposed => {
                let excluded_cell = if matches!(imposed, ImposedRole::Free) && bar_idx > 0 {
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
                    *prev_last_step,
                    bar1_first_step,
                    excluded_cell,
                    rng,
                )
            }
        };

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
    /// Fenêtre stricte du contour en dixièmes de demi-ton (`±4` demi-tons) :
    /// sans elle, une chaîne verrouillée bas par la continuité ne remonte
    /// jamais vers la cible de strain (P1-T7).
    const STRICT_CONTOUR_WINDOW_TENTHS: i32 = 40;

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

    const fn is_strict_match(&self) -> bool {
        self.tessitura_penalty == 0
            && self.fifth_excess == 0
            && self.register_penalty == 0
            && self.continuity_excess == 0
            && self.contour_distance_tenths <= Self::STRICT_CONTOUR_WINDOW_TENTHS
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
            if cand.is_strict_match() {
                return cand.cell;
            }
            if best.as_ref().is_none_or(|b| cand.score_key() < b.score_key()) {
                best = Some(cand);
            }
        }
    }

    // Repli déterministe sur la candidate la plus proche parmi les rôles permis (D6 : jamais d'échec).
    let allowed_roles: &[CellRole] = match imposed.forced_cell_role() {
        Some(CellRole::Open) => &[CellRole::Open],
        Some(CellRole::CadHalf) => &[CellRole::CadHalf],
        Some(CellRole::CadFull) => &[CellRole::CadFull],
        Some(other) => std::slice::from_ref(match other {
            CellRole::Run => &CellRole::Run,
            CellRole::Arp => &CellRole::Arp,
            CellRole::Turn => &CellRole::Turn,
            CellRole::Open => &CellRole::Open,
            CellRole::CadHalf => &CellRole::CadHalf,
            CellRole::CadFull => &CellRole::CadFull,
        }),
        None => &[CellRole::Run, CellRole::Arp, CellRole::Turn],
    };

    for base_cell in &bank.cells {
        if !allowed_roles.contains(&base_cell.role) || !is_eligible_cell(base_cell, excluded_cell) {
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
        if !allowed_roles.contains(&base_cell.role) || !base_cell.respects_cadence() {
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
    let candidates = [
        (CellRole::Run, params.weight_role_run),
        (CellRole::Arp, params.weight_role_arp),
        (CellRole::Turn, params.weight_role_turn),
    ];
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
    const REGISTER_TOLERANCE_MIDI: f32 = 6.0;
    let in_register = if target_midi <= split_midi {
        mean_midi <= split_midi + REGISTER_TOLERANCE_MIDI
    } else {
        mean_midi >= split_midi - REGISTER_TOLERANCE_MIDI
    };
    let register_penalty = u8::from(!in_register);

    let first_step = cell.first_degree().map_or(0, degree_to_diatonic_step);
    let last_step = cell.last_degree().map_or(first_step, degree_to_diatonic_step);

    let continuity_distance =
        prev_last_step.map_or(0, |prev| (i16::from(first_step) - i16::from(prev)).abs());
    let max_cont = i16::from(params.max_continuity_step_distance);
    let continuity_excess = (continuity_distance - max_cont).max(0);

    let fifth_steps = i16::from(FIFTH_DIATONIC_STEPS);
    let entry_fifth_excess = (continuity_distance - fifth_steps).max(0);
    // Les mesures 4 (index 3, avant la copie de la mesure 1 en mesure 5) et 8
    // (index 7, avant la reprise de la mesure 1 en passe 2) doivent aussi
    // rester à ≤ une quinte du premier degré de la mesure 1.
    let roles = &plan.profile.form.parts[strain.0].roles;
    let loop_fifth_excess = if (bar_idx + 1 == roles.len()
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
        let plan = TunePlan::draw(&mut rng, None, std::sync::Arc::new(Default::default()));
        let line = FiddleLine::new(plan.clone(), &empty_bank, &mut rng);
        let tune = line.render_tune();
        assert_eq!(tune.len(), 32);
        assert!(tune.iter().all(|bar| !bar.is_empty()));
    }

    #[test]
    fn single_cell_bank_falls_back_when_excluded() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let single_cell = Cell::from_notation(CellRole::Run, "5 4 3 2 1 2 3 4").unwrap_or(Cell {
            role: CellRole::Run,
            notes: vec![CellNote::Degree(1); 8],
            weight: 1,
        });
        let single_bank = CellBank { cells: vec![single_cell] };
        let plan = TunePlan::draw(&mut rng, None, std::sync::Arc::new(Default::default()));
        let line = FiddleLine::new(plan.clone(), &single_bank, &mut rng);
        assert_eq!(line.render_tune().len(), 32);
    }
}

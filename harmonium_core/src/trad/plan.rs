//! Plan formel piloté par le profil : aucune métrique ni forme implicite.
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{arrangement::Part, cells::CellRole, mode::Mode};
use crate::{algo::RngCore, style_profile::StyleProfile, theory::PitchClass};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImposedRole {
    Open,
    Free,
    CadHalf,
    CopyOf(u8),
    CadFull,
    CadOpen,
    CadTonic,
    CadFifth,
    CadOther,
    SequenceOf { bar: u8, shift: i8 },
    VariedEndOf(u8),
    CopyOfPart { part: usize, bar: u8 },
}
impl ImposedRole {
    pub const fn forced_cell_role(self) -> Option<CellRole> {
        match self {
            Self::Open => Some(CellRole::Open),
            Self::CadHalf | Self::CadFifth => Some(CellRole::CadHalf),
            Self::CadFull | Self::CadTonic => Some(CellRole::CadFull),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BarRole {
    pub strain: Part,
    pub bar_in_strain: u8,
    pub pass: u8,
    pub is_repeat: bool,
    pub role: ImposedRole,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TunePlan {
    pub key: PitchClass,
    pub mode: Mode,
    pub profile: Arc<StyleProfile>,
    roles: Vec<Vec<ImposedRole>>,
    second_endings: Vec<Option<ImposedRole>>,
    roles_resolved: bool,
}
impl Default for TunePlan {
    fn default() -> Self {
        Self::new(2, Mode::Ionian)
    }
}
impl TunePlan {
    pub fn new(key: PitchClass, mode: Mode) -> Self {
        Self::with_profile(key, mode, Arc::new(StyleProfile::default()))
    }
    pub fn with_profile(key: PitchClass, mode: Mode, profile: Arc<StyleProfile>) -> Self {
        let roles = profile
            .form
            .parts
            .iter()
            .map(|part| {
                part.roles.iter().map(|role| role.fixed().unwrap_or(ImposedRole::Free)).collect()
            })
            .collect();
        let second_endings = profile
            .form
            .parts
            .iter()
            .map(|part| part.second_ending.as_ref().and_then(|role| role.fixed()))
            .collect();
        Self { key: key % 12, mode, profile, roles, second_endings, roles_resolved: false }
    }
    pub fn draw(
        rng: &mut dyn RngCore,
        key_hint: Option<PitchClass>,
        profile: Arc<StyleProfile>,
    ) -> Self {
        let candidates = &profile.melody.key_modes;
        let key = key_hint.map(|k| k % 12);
        let total: u32 =
            candidates.iter().filter(|c| key.is_none_or(|k| c.key == k)).map(|c| c.weight).sum();
        if total == 0 {
            return Self::with_profile(
                key.unwrap_or(candidates[0].key),
                candidates[0].mode,
                profile,
            );
        }
        let mut pick = rng.next_range_usize(0..total as usize) as u32;
        for candidate in candidates {
            if key.is_none_or(|k| candidate.key == k) {
                if pick < candidate.weight {
                    return Self::with_profile(candidate.key, candidate.mode, profile.clone());
                }
                pick -= candidate.weight;
            }
        }
        unreachable!("validated positive weights")
    }
    /// Un seul tirage structurel par air ; les reprises ne consomment aucun aléa.
    pub fn resolve_roles(&mut self, rng: &mut dyn RngCore) {
        if self.roles_resolved {
            return;
        }
        for (part, configured) in self.profile.form.parts.iter().enumerate() {
            for (bar, role) in configured.roles.iter().enumerate() {
                self.roles[part][bar] = role.draw(&self.profile.form, rng);
            }
        }
        for (part, configured) in self.profile.form.parts.iter().enumerate() {
            self.second_endings[part] =
                configured.second_ending.as_ref().map(|role| role.draw(&self.profile.form, rng));
        }
        self.roles_resolved = true;
    }
    pub fn roles_for_part(&self, part: usize) -> &[ImposedRole] {
        &self.roles[part]
    }
    pub(crate) fn second_ending_role(&self, part: usize) -> Option<ImposedRole> {
        self.second_endings[part]
    }
    /// Partie, passe, décalage et variante, calculés en un seul parcours de forme.
    pub(crate) fn position(&self, bar: usize) -> (usize, usize, usize, bool) {
        let (part, pass, offset) = self.profile.form.position(bar);
        let configured = &self.profile.form.parts[part];
        let second_ending = configured.second_ending.is_some()
            && offset + 1 == configured.roles.len()
            && pass == self.profile.form.sequence.iter().filter(|&&p| p == part).count();
        (part, pass, offset, second_ending)
    }
    /// Cellules distinctes, y compris les fins de reprise configurées.
    pub fn material_bars(&self) -> usize {
        self.profile.form.unique_bars()
            + self.profile.form.parts.iter().filter(|part| part.second_ending.is_some()).count()
    }
    pub fn material_index(&self, bar: usize) -> usize {
        let (part, _, offset, second_ending) = self.position(bar);
        if second_ending {
            self.profile.form.unique_bars()
                + self.profile.form.parts[..part]
                    .iter()
                    .filter(|p| p.second_ending.is_some())
                    .count()
        } else {
            self.profile.form.unique_offset(part) + offset
        }
    }
    pub fn total_bars(&self) -> usize {
        self.profile.form.bars_per_tour()
    }
    pub fn bar_role(&self, bar: usize) -> BarRole {
        let (part, pass, offset, second_ending) = self.position(bar);
        BarRole {
            strain: Part(part),
            bar_in_strain: (offset + 1) as u8,
            pass: pass as u8,
            is_repeat: pass > 1,
            role: if second_ending {
                self.second_endings[part].unwrap_or(self.roles[part][offset])
            } else {
                self.roles[part][offset]
            },
        }
    }
    pub fn bar_role_1based(&self, bar: usize) -> BarRole {
        self.bar_role(bar.saturating_sub(1))
    }
    /// Mesure jouée représentant la cellule distincte `i` (fins comprises).
    pub fn representative_bar(&self, i: usize) -> usize {
        let form = &self.profile.form;
        let mut remaining = i;
        for (part, configured) in form.parts.iter().enumerate() {
            if remaining < configured.roles.len() {
                let sequence_index = form
                    .sequence
                    .iter()
                    .position(|&p| p == part)
                    .unwrap_or_else(|| unreachable!("validated referenced part"));
                return form.sequence[..sequence_index]
                    .iter()
                    .map(|&p| form.parts[p].roles.len())
                    .sum::<usize>()
                    + remaining;
            }
            remaining -= configured.roles.len();
        }
        for (part, configured) in form.parts.iter().enumerate() {
            if configured.second_ending.is_none() {
                continue;
            }
            if remaining == 0 {
                let sequence_index = form
                    .sequence
                    .iter()
                    .rposition(|&p| p == part)
                    .unwrap_or_else(|| unreachable!("validated referenced part"));
                return form.sequence[..=sequence_index]
                    .iter()
                    .map(|&p| form.parts[p].roles.len())
                    .sum::<usize>()
                    - 1;
            }
            remaining -= 1;
        }
        unreachable!("validated distinct material index")
    }
}

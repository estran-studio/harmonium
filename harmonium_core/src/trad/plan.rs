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
    SequenceOf { bar: u8, shift: i8 },
    VariedEndOf(u8),
    CopyOfPart { part: usize, bar: u8 },
}
impl ImposedRole {
    pub const fn forced_cell_role(self) -> Option<CellRole> {
        match self {
            Self::Open => Some(CellRole::Open),
            Self::CadHalf => Some(CellRole::CadHalf),
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
        Self { key: key % 12, mode, profile, roles, roles_resolved: false }
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
        self.roles_resolved = true;
    }
    pub fn roles_for_part(&self, part: usize) -> &[ImposedRole] {
        &self.roles[part]
    }
    pub fn total_bars(&self) -> usize {
        self.profile.form.bars_per_tour()
    }
    pub fn bar_role(&self, bar: usize) -> BarRole {
        let (part, pass, offset) = self.profile.form.position(bar);
        BarRole {
            strain: Part(part),
            bar_in_strain: (offset + 1) as u8,
            pass: pass as u8,
            is_repeat: pass > 1,
            role: self.roles[part][offset],
        }
    }
    pub fn bar_role_1based(&self, bar: usize) -> BarRole {
        self.bar_role(bar.saturating_sub(1))
    }
    /// Première passe de la mesure distincte `i`, dans l'ordre des parties du profil.
    pub fn first_pass_bar(&self, i: usize) -> usize {
        let mut part = 0;
        let mut offset = i;
        while offset >= self.profile.form.parts[part].roles.len() {
            offset -= self.profile.form.parts[part].roles.len();
            part += 1;
        }
        let mut bar = 0;
        for &p in &self.profile.form.sequence {
            if p == part {
                return bar + offset;
            }
            bar += self.profile.form.parts[p].roles.len();
        }
        unreachable!("validated referenced part")
    }
}

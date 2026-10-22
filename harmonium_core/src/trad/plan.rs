//! Plan formel piloté par le profil : aucune métrique ni forme implicite.
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{arrangement::Part, cells::CellRole, mode::Mode};
use crate::{harmony::RngCore, style_profile::StyleProfile, theory::PitchClass};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImposedRole {
    Open,
    Free,
    CadHalf,
    CopyOf(u8),
    CadFull,
}
impl ImposedRole {
    pub const fn forced_cell_role(self) -> Option<CellRole> {
        match self {
            Self::Open => Some(CellRole::Open),
            Self::CadHalf => Some(CellRole::CadHalf),
            Self::CadFull => Some(CellRole::CadFull),
            Self::Free | Self::CopyOf(_) => None,
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
        Self { key: key % 12, mode, profile }
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
            role: self.profile.form.parts[part].roles[offset],
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

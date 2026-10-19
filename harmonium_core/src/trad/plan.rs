//! Plan structurel d'un tune traditionnel (D3, D4).
//!
//! Définit la grille de mesure du reel (2/2, 16 pas, pulse à la blanche de 8
//! pas, swing nominal 0.56) et la structure formelle AABB (32 mesures : deux
//! strains A et B de 8 mesures chacune, jouées deux fois avec rôles imposés par
//! position dans la strain).

use serde::{Deserialize, Serialize};

use super::{
    cells::CellRole,
    mode::{Mode, PitchClass},
};
use crate::{
    harmony::RngCore,
    params::{PulseSpec, TimeSignature},
    tuning::ReelFiddleParams,
};

/// Type de pièce traditionnelle québécoise (D3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TuneKind {
    /// Reel en 2/2 (16 pas par mesure, pulse = blanche de 8 pas, croche = 2 pas).
    #[default]
    #[serde(alias = "Reel")]
    Reel,
}

impl TuneKind {
    /// Signature rythmique notée du type de tune (2/2 pour un reel).
    #[must_use]
    pub const fn time_signature(self) -> TimeSignature {
        match self {
            Self::Reel => TimeSignature::new(2, 2),
        }
    }

    /// Pulse dansée (`PulseSpec` : blanche = 8 pas, 2 pulses par mesure en reel).
    #[must_use]
    pub const fn pulse_spec(self) -> PulseSpec {
        match self {
            Self::Reel => PulseSpec { steps_per_pulse: 8, pulses_per_bar: 2 },
        }
    }

    /// Nombre de pas de grille par mesure (16 pas en reel).
    #[must_use]
    pub const fn steps_per_bar(self) -> usize {
        match self {
            Self::Reel => 16,
        }
    }

    /// Nombre de pas de grille par croche de mélodie (2 pas en reel).
    #[must_use]
    pub const fn steps_per_eighth(self) -> usize {
        match self {
            Self::Reel => 2,
        }
    }

    /// Ratio de swing par défaut du style (0.56 en reel).
    #[must_use]
    pub const fn default_swing_ratio(self) -> f32 {
        match self {
            Self::Reel => 0.56,
        }
    }

    /// Plage de tempo nominal à la noire `(min_bpm, max_bpm)` (220–240 BPM en reel).
    #[must_use]
    pub const fn nominal_bpm_range(self) -> (f32, f32) {
        match self {
            Self::Reel => (220.0, 240.0),
        }
    }

    /// Nombre de mesures par strain (8 mesures en reel).
    #[must_use]
    pub const fn bars_per_strain(self) -> usize {
        match self {
            Self::Reel => 8,
        }
    }
}

/// Partie (strain) d'un tune traditionnel (A = 1re partie, B = 2e partie).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Strain {
    /// Première partie : tessiture médiane centrée sur les degrés 3 à 5.
    #[default]
    A,
    /// Seconde partie : tessiture plus haute centrée sur les degrés 6 à 8.
    B,
}

impl Strain {
    /// Les deux strains en ordre canonique.
    pub const ALL: [Self; 2] = [Self::A, Self::B];

    /// Indice 0-based de la strain (`0` pour `A`, `1` pour `B`).
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }

    /// Plage de degrés cible `(min, max)` caractérisant le contour de la strain (D6).
    #[must_use]
    pub const fn target_degree_center(self) -> (i8, i8) {
        match self {
            Self::A => (3, 5),
            Self::B => (6, 8),
        }
    }
}

/// Schéma de répétition des strains d'un tune.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepeatScheme {
    /// Forme AABB : A (8 mes.) + A (8 mes.) + B (8 mes.) + B (8 mes.) = 32 mesures.
    #[default]
    #[serde(alias = "AABB", alias = "Aabb")]
    Aabb,
}

impl RepeatScheme {
    /// Nombre de mesures dans une strain unique (8).
    #[must_use]
    pub const fn bars_per_strain(self) -> usize {
        match self {
            Self::Aabb => 8,
        }
    }

    /// Nombre total de mesures d'un cycle complet du tune (32 en AABB).
    #[must_use]
    pub const fn total_bars(self) -> usize {
        match self {
            Self::Aabb => 32,
        }
    }
}

/// Rôle structurel imposé à une mesure au sein d'une strain de 8 mesures (D4, D6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImposedRole {
    /// Mesure 1 : cellule d'ouverture (`CellRole::Open`).
    Open,
    /// Mesures 2, 3, 7 : tirage libre parmi `Run`, `Arp`, `Turn`.
    Free,
    /// Mesure 4 : demi-cadence sur un degré de dominante (`CellRole::CadHalf`).
    CadHalf,
    /// Mesures 5–6 : copie à l'identique de la mesure `1` ou `2` de la même strain.
    CopyOf(u8),
    /// Mesure 8 : cadence parfaite sur la tonique (`CellRole::CadFull`).
    CadFull,
}

impl ImposedRole {
    /// Retourne le [`CellRole`] imposé lorsque la mesure exige un rôle dédié
    /// (`Open`, `CadHalf`, `CadFull`), ou `None` pour `Free` et `CopyOf`.
    #[must_use]
    pub const fn forced_cell_role(self) -> Option<CellRole> {
        match self {
            Self::Open => Some(CellRole::Open),
            Self::CadHalf => Some(CellRole::CadHalf),
            Self::CadFull => Some(CellRole::CadFull),
            Self::Free | Self::CopyOf(_) => None,
        }
    }
}

/// Rôle complet d'une mesure dans le plan du tune, retourné par
/// [`TunePlan::bar_role`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BarRole {
    /// Strain active (`Strain::A` ou `Strain::B`).
    pub strain: Strain,
    /// Numéro de mesure dans la strain (1-based : `1..=8`).
    pub bar_in_strain: u8,
    /// Numéro de passe de la strain (`1` = première passe, `2` = reprise).
    pub pass: u8,
    /// `true` lors de la seconde passe (reprise identique en phase 1).
    pub is_repeat: bool,
    /// Rôle structurel imposé à cette mesure dans la strain.
    pub role: ImposedRole,
}

/// Plan formel d'un tune de 32 mesures (D4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TunePlan {
    /// Tonique du tune (0 = C, 2 = D, 7 = G, 9 = A, etc.).
    pub key: PitchClass,
    /// Mode diatonique du tune.
    pub mode: Mode,
    /// Type de tune (`TuneKind::Reel`).
    pub kind: TuneKind,
    /// Schéma de répétition (`RepeatScheme::Aabb`).
    pub scheme: RepeatScheme,
}

impl Default for TunePlan {
    fn default() -> Self {
        // Ré majeur (D Ionian), tonalité reine du violon trad québécois.
        Self::reel(2, Mode::Ionian)
    }
}

impl TunePlan {
    /// Construit un nouveau plan de tune.
    #[must_use]
    pub const fn new(key: PitchClass, mode: Mode, kind: TuneKind, scheme: RepeatScheme) -> Self {
        Self { key: key % 12, mode, kind, scheme }
    }

    /// Construit un plan de reel en forme AABB pour une tonalité et un mode donnés.
    #[must_use]
    pub const fn reel(key: PitchClass, mode: Mode) -> Self {
        Self::new(key, mode, TuneKind::Reel, RepeatScheme::Aabb)
    }

    /// Tire un [`TunePlan`] de reel en forme AABB selon la distribution par
    /// défaut de [`ReelFiddleParams`] (D4, constitution V) :
    /// - `D Ionian` (poids 3), `G Ionian` (poids 3), `A Ionian` (poids 2),
    ///   `A Mixolydian` (poids 1), `D Mixolydian` (poids 1) ;
    /// - si `key_hint` est fourni, force la tonique `key_hint % 12`.
    #[must_use]
    pub fn draw(rng: &mut dyn RngCore, key_hint: Option<PitchClass>) -> Self {
        Self::draw_with_params(rng, key_hint, &ReelFiddleParams::default())
    }

    /// Tire un [`TunePlan`] de reel en forme AABB à partir des poids de
    /// [`ReelFiddleParams`] (portés par [`crate::tuning::TuningParams`]).
    #[must_use]
    pub fn draw_with_params(
        rng: &mut dyn RngCore,
        key_hint: Option<PitchClass>,
        params: &ReelFiddleParams,
    ) -> Self {
        let candidates: [(PitchClass, Mode, u32); 5] = [
            (2, Mode::Ionian, params.weight_d_ionian),
            (7, Mode::Ionian, params.weight_g_ionian),
            (9, Mode::Ionian, params.weight_a_ionian),
            (9, Mode::Mixolydian, params.weight_a_mixolydian),
            (2, Mode::Mixolydian, params.weight_d_mixolydian),
        ];

        if let Some(hint) = key_hint {
            let key = hint % 12;
            let total: u32 = candidates
                .iter()
                .filter(|&&(k, _, _)| k == key)
                .map(|&(_, _, w)| w)
                .sum();
            if total == 0 {
                return Self::reel(key, Mode::Ionian);
            }
            let mut pick = rng.next_range_usize(0..(total as usize)) as u32;
            for &(k, mode, weight) in &candidates {
                if k == key && weight > 0 {
                    if pick < weight {
                        return Self::reel(key, mode);
                    }
                    pick -= weight;
                }
            }
            return Self::reel(key, Mode::Ionian);
        }

        let total: u32 = candidates.iter().map(|&(_, _, w)| w).sum();
        if total == 0 {
            return Self::default();
        }
        let mut pick = rng.next_range_usize(0..(total as usize)) as u32;
        for &(key, mode, weight) in &candidates {
            if weight > 0 {
                if pick < weight {
                    return Self::reel(key, mode);
                }
                pick -= weight;
            }
        }
        Self::default()
    }

    /// Nombre total de mesures du tune (32 en forme AABB).
    #[must_use]
    pub const fn total_bars(&self) -> usize {
        self.scheme.total_bars()
    }

    /// Détermine le rôle d'une mesure dans le tune.
    ///
    /// `bar_in_tune` est l'indice 0-based de la mesure dans le tune (`0..32`,
    /// replié modulo 32) :
    /// - `0..8` : strain A, 1re passe (`bar_in_strain` 1..=8)
    /// - `8..16` : strain A, 2e passe (`bar_in_strain` 1..=8)
    /// - `16..24` : strain B, 1re passe (`bar_in_strain` 1..=8)
    /// - `24..32` : strain B, 2e passe (`bar_in_strain` 1..=8)
    #[must_use]
    pub const fn bar_role(&self, bar_in_tune: usize) -> BarRole {
        let idx = bar_in_tune % self.scheme.total_bars();
        let section = idx / 8;
        let strain = if section < 2 { Strain::A } else { Strain::B };
        let is_repeat = section % 2 == 1;
        let pass = if is_repeat { 2 } else { 1 };
        let bar_in_strain = (idx % 8) as u8 + 1;

        let role = match bar_in_strain {
            1 => ImposedRole::Open,
            4 => ImposedRole::CadHalf,
            5 => ImposedRole::CopyOf(1),
            6 => ImposedRole::CopyOf(2),
            8 => ImposedRole::CadFull,
            _ => ImposedRole::Free,
        };

        BarRole { strain, bar_in_strain, pass, is_repeat, role }
    }

    /// Variante 1-based (`bar_number` `1..=32`, conforme à `Measure::index`).
    #[must_use]
    pub const fn bar_role_1based(&self, bar_number: usize) -> BarRole {
        self.bar_role(bar_number.saturating_sub(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reel_grid_constants_match_d3() {
        let kind = TuneKind::Reel;
        assert_eq!(kind.time_signature(), TimeSignature::new(2, 2));
        assert_eq!(kind.pulse_spec(), PulseSpec { steps_per_pulse: 8, pulses_per_bar: 2 });
        assert_eq!(kind.steps_per_bar(), 16);
        assert_eq!(kind.steps_per_eighth(), 2);
        assert!((kind.default_swing_ratio() - 0.56).abs() < f32::EPSILON);
        assert_eq!(kind.nominal_bpm_range(), (220.0, 240.0));
    }

    #[test]
    fn bar_role_across_all_32_bars_of_aabb_tune() {
        let plan = TunePlan::reel(2, Mode::Ionian);
        assert_eq!(plan.total_bars(), 32);

        let expected_roles = [
            ImposedRole::Open,      // 1
            ImposedRole::Free,      // 2
            ImposedRole::Free,      // 3
            ImposedRole::CadHalf,   // 4
            ImposedRole::CopyOf(1), // 5
            ImposedRole::CopyOf(2), // 6
            ImposedRole::Free,      // 7
            ImposedRole::CadFull,   // 8
        ];

        for bar in 0..32 {
            let role = plan.bar_role(bar);
            let expected_strain = if bar < 16 { Strain::A } else { Strain::B };
            let expected_pass = if (bar / 8) % 2 == 0 { 1 } else { 2 };
            let expected_repeat = expected_pass == 2;
            let expected_bar_in_strain = (bar % 8) as u8 + 1;
            let expected_imposed = expected_roles[bar % 8];

            assert_eq!(role.strain, expected_strain, "bar {bar} strain");
            assert_eq!(role.pass, expected_pass, "bar {bar} pass");
            assert_eq!(role.is_repeat, expected_repeat, "bar {bar} is_repeat");
            assert_eq!(role.bar_in_strain, expected_bar_in_strain, "bar {bar} bar_in_strain");
            assert_eq!(role.role, expected_imposed, "bar {bar} role");
            assert_eq!(plan.bar_role_1based(bar + 1), role);
        }

        // Vérifie le repli modulo 32 au-delà d'un tune complet
        assert_eq!(plan.bar_role(32), plan.bar_role(0));
        assert_eq!(plan.bar_role(47), plan.bar_role(15));
    }

    #[test]
    fn draw_respects_weights_and_key_hint() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut counts = std::collections::HashMap::new();
        for _ in 0..1000 {
            let plan = TunePlan::draw(&mut rng, None);
            assert_eq!(plan.kind, TuneKind::Reel);
            assert_eq!(plan.scheme, RepeatScheme::Aabb);
            *counts.entry((plan.key, plan.mode)).or_insert(0usize) += 1;
        }

        // Les 5 combinaisons D/G/A Ionian + A/D Mixolydian doivent apparaître
        assert_eq!(counts.len(), 5);
        assert!(counts[&(2, Mode::Ionian)] > counts[&(2, Mode::Mixolydian)]);
        assert!(counts[&(7, Mode::Ionian)] > counts[&(9, Mode::Mixolydian)]);
        assert!(counts[&(9, Mode::Ionian)] > counts[&(9, Mode::Mixolydian)]);

        // Avec indice de tonalité
        let hinted_g = TunePlan::draw(&mut rng, Some(7));
        assert_eq!(hinted_g.key, 7);
        assert_eq!(hinted_g.mode, Mode::Ionian);

        let hinted_c = TunePlan::draw(&mut rng, Some(0));
        assert_eq!(hinted_c.key, 0);
        assert_eq!(hinted_c.mode, Mode::Ionian);
    }
}

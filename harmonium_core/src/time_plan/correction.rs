//! Correction de l'écart entre l'audio et le plan : prototype isolé, non branché.
//!
//! Le playhead mesure et publie son écart ; le corriger demande une décision du
//! propriétaire (seuils, et s'il est acceptable d'accélérer l'audio de quelques pour mille
//! ou de sauter à la mesure). Cette fonction pure décrit la politique proposée par l'étude
//! J1 (`specs/013-moteur-dans-le-jeu/horloge-etude.md`) pour qu'on puisse l'éprouver.

use super::Flicks;

/// Politique de correction : zone morte, micro-ajustement borné, saut au-delà.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CorrectionPolicy {
    /// Écart toléré sans rien faire (sous le seuil de perception d'un décalage).
    pub dead_zone: Flicks,
    /// Ajustement de tempo maximal, en pour mille (inaudible jusqu'à quelques ‰).
    pub max_nudge_per_mille: u32,
    /// Nombre de pulses sur lesquelles l'écart doit être rattrapé.
    pub window_pulses: u32,
}

impl Default for CorrectionPolicy {
    /// 10 ms de zone morte, 5 ‰ au plus, rattrapage sur 8 pulses.
    fn default() -> Self {
        Self { dead_zone: Flicks(7_056_000), max_nudge_per_mille: 5, window_pulses: 8 }
    }
}

/// Action proposée pour ramener l'audio sur le plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Correction {
    /// Écart dans la zone morte.
    None,
    /// Jouer l'audio plus vite (`per_mille > 0`) ou plus lentement pendant `pulses` pulses.
    Nudge { per_mille: i32, pulses: u32 },
    /// Écart trop grand pour un ajustement inaudible : se recaler à la prochaine mesure.
    JumpAtNextBar,
}

impl CorrectionPolicy {
    /// `gap` : instant du plan joué par l'audio moins instant du plan de la simulation, au
    /// même instant mur (positif : l'audio est en avance). `pulse` : longueur de la pulse.
    #[must_use]
    pub fn decide(&self, gap: i64, pulse: Flicks) -> Correction {
        let magnitude = gap.unsigned_abs();
        if magnitude <= self.dead_zone.0 {
            return Correction::None;
        }
        let window = u64::from(self.window_pulses.max(1)) * pulse.0.max(1);
        let needed = (u128::from(magnitude) * 1000).div_ceil(u128::from(window));
        if needed > u128::from(self.max_nudge_per_mille) {
            return Correction::JumpAtNextBar;
        }
        let per_mille = needed as i32;
        Correction::Nudge {
            per_mille: if gap > 0 { -per_mille } else { per_mille },
            pulses: self.window_pulses.max(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time_plan::FLICKS_PER_SECOND;

    #[test]
    fn small_gaps_are_ignored_medium_ones_nudged_large_ones_jump() {
        let policy = CorrectionPolicy::default();
        let pulse = Flicks(FLICKS_PER_SECOND / 2);
        let ms = |n: i64| n * (FLICKS_PER_SECOND as i64 / 1000);
        assert_eq!(policy.decide(ms(5), pulse), Correction::None);
        // 15 ms de retard sur 8 pulses de 500 ms : 3,75 ‰ → 4 ‰ plus vite.
        assert_eq!(policy.decide(-ms(15), pulse), Correction::Nudge { per_mille: 4, pulses: 8 });
        assert_eq!(policy.decide(ms(15), pulse), Correction::Nudge { per_mille: -4, pulses: 8 });
        assert_eq!(policy.decide(ms(40), pulse), Correction::JumpAtNextBar);
    }
}

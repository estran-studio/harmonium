//! Plan de temps musical partagé (phase 013, J1).
//!
//! La simulation du jeu est maîtresse du temps : chaque client calcule la même
//! carte « mesure/pas ↔ instant » à partir des en-têtes temporels des mesures
//! ([`BarTiming`]), sans lire l'audio. Tout le calcul est entier : instants en
//! [`Flicks`] (705 600 000 par seconde, divisible par les cadences de simulation
//! 24 à 240 Hz et par les fréquences 44,1, 48 et 96 kHz), tempo en millièmes de
//! BPM ([`Tempo`]), swing en dix-millièmes ([`Swing`]). Les seules conversions
//! depuis un flottant ([`Tempo::from_bpm`], [`Swing::from_ratio`]) sont une
//! multiplication IEEE arrondie, à la frontière avec le générateur.
//!
//! Le playback suit ce plan par construction : il gèle chaque pulse avec
//! [`BarTiming::step_offset`] au tempo de [`ramp_tempo`] et convertit les instants
//! cumulés en frames ([`Flicks::to_frames`]) ; aucune durée n'est arrondie
//! isolément, donc aucune dérive. [`score::SharedTimePlan`] construit le même plan
//! depuis la graine, le profil et des commandes datées, sans audio.
#![deny(clippy::float_arithmetic)]

pub mod correction;
pub mod score;

use serde::{Deserialize, Serialize};

use crate::{report::MeasureSnapshot, timeline::Measure};

/// Flicks par seconde : multiple commun de 24, 25, 30, 48, 50, 60, 90, 100, 120, 144 et
/// 240 Hz comme de 8 000 à 192 000 Hz. Une frame de simulation à 60 Hz vaut exactement
/// 11 760 000 flicks, une frame audio à 48 kHz 14 700.
pub const FLICKS_PER_SECOND: u64 = 705_600_000;
const FLICKS_PER_MINUTE: u128 = 60 * FLICKS_PER_SECOND as u128;

/// Division entière arrondie au plus proche (demi vers le haut).
const fn div_round(numerator: u128, denominator: u128) -> u128 {
    (numerator + denominator / 2) / denominator
}

/// Instant du plan, en flicks depuis le premier pas de la mesure 1.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Flicks(pub u64);

impl Flicks {
    pub const ZERO: Self = Self(0);

    /// Instant de la frame `frame` d'une horloge à `rate` frames par seconde (simulation ou
    /// audio). Exact quand `rate` divise [`FLICKS_PER_SECOND`], arrondi inférieur sinon.
    #[must_use]
    pub const fn from_frames(frame: u64, rate: u32) -> Self {
        let rate = if rate == 0 { 1 } else { rate };
        Self((frame as u128 * FLICKS_PER_SECOND as u128 / rate as u128) as u64)
    }

    /// Frame la plus proche de cet instant pour une horloge à `rate` frames par seconde.
    #[must_use]
    pub const fn to_frames(self, rate: u32) -> u64 {
        div_round(self.0 as u128 * rate as u128, FLICKS_PER_SECOND as u128) as u64
    }

    /// Frame qui contient cet instant (arrondi inférieur) : la frame de simulation courante.
    #[must_use]
    pub const fn frame_floor(self, rate: u32) -> u64 {
        (self.0 as u128 * rate as u128 / FLICKS_PER_SECOND as u128) as u64
    }
}

/// Tempo à la noire, en millièmes de BPM, borné à 40..=300 BPM comme `MusicalParams::bpm`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tempo(u32);

impl Tempo {
    pub const MIN: Self = Self(40_000);
    pub const MAX: Self = Self(300_000);

    /// Tempo exact en millièmes de BPM, borné.
    #[must_use]
    pub const fn from_milli_bpm(milli_bpm: u32) -> Self {
        if milli_bpm < Self::MIN.0 {
            Self::MIN
        } else if milli_bpm > Self::MAX.0 {
            Self::MAX
        } else {
            Self(milli_bpm)
        }
    }

    /// Seule entrée flottante du plan : le tempo de mesure du générateur, arrondi au
    /// millième de BPM. Une multiplication IEEE correctement arrondie et un arrondi : même
    /// résultat sur toute plateforme. Non fini : le minimum.
    #[must_use]
    #[allow(clippy::float_arithmetic)]
    pub fn from_bpm(bpm: f32) -> Self {
        if !bpm.is_finite() {
            return Self::MIN;
        }
        let milli = (f64::from(bpm) * 1000.0).round().clamp(0.0, f64::from(u32::MAX));
        Self::from_milli_bpm(milli as u32)
    }

    #[must_use]
    pub const fn milli_bpm(self) -> u32 {
        self.0
    }

    /// Valeur transmise au générateur (paramètres `f32`) : exacte au millième près.
    #[must_use]
    #[allow(clippy::float_arithmetic)]
    pub fn bpm(self) -> f32 {
        self.0 as f32 / 1000.0
    }

    /// Durée d'une noire, arrondie au flick.
    #[must_use]
    pub const fn quarter(self) -> Flicks {
        Flicks(div_round(FLICKS_PER_MINUTE * 1000, self.0 as u128) as u64)
    }
}

/// Part de la première croche d'une paire, en dix-millièmes : 5 000 = droit, borné à
/// 5 000..=7 500 comme `MusicalParams::swing_ratio`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Swing(u16);

impl Swing {
    pub const STRAIGHT: Self = Self(5_000);
    pub const MAX: Self = Self(7_500);
    const UNIT: u128 = 10_000;

    #[must_use]
    pub const fn from_ten_thousandths(value: u16) -> Self {
        if value < Self::STRAIGHT.0 {
            Self::STRAIGHT
        } else if value > Self::MAX.0 {
            Self::MAX
        } else {
            Self(value)
        }
    }

    /// Entrée flottante du générateur (`Measure::swing_ratio`), arrondie au dix-millième.
    #[must_use]
    #[allow(clippy::float_arithmetic)]
    pub fn from_ratio(ratio: f32) -> Self {
        if !ratio.is_finite() {
            return Self::STRAIGHT;
        }
        let value = (f64::from(ratio) * 10_000.0).round().clamp(0.0, f64::from(u16::MAX));
        Self::from_ten_thousandths(value as u16)
    }

    #[must_use]
    pub const fn ten_thousandths(self) -> u16 {
        self.0
    }
}

/// Tempo de la pulse `k` (1-based) d'une rampe linéaire de `n` pulses de `from` vers
/// `to` : interpolation entière, `to` exactement à la `n`-ième pulse et au-delà. Règle
/// unique du playback (rampe de mesure, `SetTempo`) et du plan.
#[must_use]
pub const fn ramp_tempo(from: Tempo, to: Tempo, k: u32, n: u32) -> Tempo {
    if n == 0 || k >= n {
        return to;
    }
    let delta = to.0 as i64 - from.0 as i64;
    Tempo((from.0 as i64 + delta * k as i64 / n as i64) as u32)
}

/// En-tête temporel d'une mesure : tout ce qui place ses pas dans le temps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarTiming {
    /// Tempo nominal de la mesure (atteint à sa dernière pulse si elle rampe).
    pub tempo: Tempo,
    pub swing: Swing,
    pub numerator: u16,
    pub denominator: u16,
    /// Pas de grille de la mesure.
    pub steps: u16,
    pub steps_per_pulse: u8,
    pub pulses_per_bar: u8,
    /// Un changement de tempo à cette mesure rampe sur ses pulses depuis le tempo de fin de
    /// la mesure précédente (mesures d'un air) ; sinon il saute.
    pub ramp_in: bool,
}

impl BarTiming {
    /// En-tête d'une mesure générée.
    #[must_use]
    pub fn of(measure: &Measure) -> Self {
        Self {
            tempo: Tempo::from_bpm(measure.tempo),
            swing: Swing::from_ratio(measure.swing_ratio),
            numerator: measure.time_signature.numerator as u16,
            denominator: measure.time_signature.denominator as u16,
            steps: measure.steps as u16,
            steps_per_pulse: measure.pulse.steps_per_pulse,
            pulses_per_bar: measure.pulse.pulses_per_bar,
            ramp_in: measure.position.is_some(),
        }
    }

    /// En-tête lu dans le flux de snapshots (contrat audio).
    #[must_use]
    pub fn of_snapshot(snapshot: &MeasureSnapshot) -> Self {
        Self {
            tempo: Tempo::from_bpm(snapshot.tempo),
            swing: Swing::from_ratio(snapshot.swing_ratio),
            numerator: snapshot.time_sig_numerator as u16,
            denominator: snapshot.time_sig_denominator as u16,
            steps: snapshot.steps as u16,
            steps_per_pulse: snapshot.pulse.steps_per_pulse,
            pulses_per_bar: snapshot.pulse.pulses_per_bar,
            ramp_in: snapshot.position.is_some(),
        }
    }

    #[must_use]
    pub const fn steps_per_pulse(&self) -> u32 {
        if self.steps_per_pulse == 0 { 1 } else { self.steps_per_pulse as u32 }
    }

    const fn steps(&self) -> u32 {
        if self.steps == 0 { 1 } else { self.steps as u32 }
    }

    /// Pulses jouées dans la mesure (la dernière peut être incomplète sur un mètre exotique).
    #[must_use]
    pub const fn pulse_count(&self) -> u32 {
        self.steps().div_ceil(self.steps_per_pulse())
    }

    /// Longueur de la rampe d'entrée, en pulses (celle de `PulseSpec::pulses_per_bar`).
    #[must_use]
    pub const fn ramp_pulses(&self) -> u32 {
        if self.pulses_per_bar == 0 { 1 } else { self.pulses_per_bar as u32 }
    }

    /// Pas joués dans la pulse `pulse` (0-based).
    #[must_use]
    pub const fn steps_in_pulse(&self, pulse: u32) -> u32 {
        let spp = self.steps_per_pulse();
        let left = self.steps().saturating_sub(pulse * spp);
        if left < spp { left } else { spp }
    }

    /// Longueur d'une pulse complète à `tempo` : noire × pas par pulse / pas par noire, où
    /// pas par noire = `steps × dénominateur / (numérateur × 4)`. Un seul arrondi.
    #[must_use]
    pub const fn pulse_length(&self, tempo: Tempo) -> u64 {
        let numerator = if self.numerator == 0 { 1 } else { self.numerator as u128 };
        let denominator = if self.denominator == 0 { 1 } else { self.denominator as u128 };
        div_round(
            FLICKS_PER_MINUTE * 1000 * self.steps_per_pulse() as u128 * numerator * 4,
            tempo.0 as u128 * self.steps() as u128 * denominator,
        ) as u64
    }

    /// Pas par noire quand le swing s'applique : grille entière et paire, pulse faite de
    /// noires entières, swing non droit. Mesure composée ou grille impaire : pas de swing.
    const fn swing_quarter_steps(&self) -> Option<u32> {
        let numerator = if self.numerator == 0 { 1 } else { self.numerator as u32 };
        let denominator = self.denominator as u32;
        let grid = self.steps() * denominator;
        if self.swing.0 <= Swing::STRAIGHT.0 || !grid.is_multiple_of(numerator * 4) {
            return None;
        }
        let quarter_steps = grid / (numerator * 4);
        if quarter_steps == 0
            || !quarter_steps.is_multiple_of(2)
            || !self.steps_per_pulse().is_multiple_of(quarter_steps)
        {
            return None;
        }
        Some(quarter_steps)
    }

    /// Début du pas `k` (`0..=steps_per_pulse`) relativement au début de sa pulse, à
    /// `tempo`. Droit : `k × pulse / pas`. Swing : chaque noire est partagée, la première
    /// croche durant `swing × noire` ; la noire et la pulse gardent leur longueur exacte.
    #[must_use]
    pub const fn step_offset(&self, tempo: Tempo, k: u32) -> u64 {
        let spp = self.steps_per_pulse();
        let k = if k > spp { spp } else { k };
        let pulse = self.pulse_length(tempo) as u128;
        let Some(quarter_steps) = self.swing_quarter_steps() else {
            return div_round(k as u128 * pulse, spp as u128) as u64;
        };
        let quarters = (spp / quarter_steps) as u128;
        let unit = (k / quarter_steps) as u128;
        let unit_start = div_round(unit * pulse, quarters);
        let within = k % quarter_steps;
        if within == 0 {
            return unit_start as u64;
        }
        let unit_length = div_round((unit + 1) * pulse, quarters) - unit_start;
        let half = (quarter_steps / 2) as u128;
        let long = self.swing.0 as u128;
        let within = within as u128;
        let share = if within <= half {
            within * long
        } else {
            half * long + (within - half) * (Swing::UNIT - long)
        };
        (unit_start + div_round(unit_length * share, Swing::UNIT * half)) as u64
    }
}

/// Mesure placée dans le plan : son début, sa longueur, l'origine de sa rampe éventuelle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PlannedBar {
    timing: BarTiming,
    start: u64,
    length: u64,
    /// Tempo de départ d'une rampe d'entrée ; `None` : tempo nominal dès la première pulse.
    ramp_from: Option<Tempo>,
}

impl PlannedBar {
    fn new(timing: BarTiming, start: u64, previous_end: Option<Tempo>) -> Self {
        let ramp_from = previous_end.filter(|&end| timing.ramp_in && end != timing.tempo);
        let mut bar = Self { timing, start, length: 0, ramp_from };
        bar.length = (0..timing.pulse_count()).map(|pulse| bar.pulse_length(pulse)).sum();
        bar
    }

    /// Tempo gelé de la pulse `pulse` (0-based).
    fn pulse_tempo(&self, pulse: u32) -> Tempo {
        self.ramp_from.map_or(self.timing.tempo, |from| {
            ramp_tempo(from, self.timing.tempo, pulse + 1, self.timing.ramp_pulses())
        })
    }

    /// Durée jouée de la pulse `pulse` (complète, ou ses premiers pas si elle est tronquée).
    fn pulse_length(&self, pulse: u32) -> u64 {
        self.timing.step_offset(self.pulse_tempo(pulse), self.timing.steps_in_pulse(pulse))
    }

    fn end_tempo(&self) -> Tempo {
        self.pulse_tempo(self.timing.pulse_count().saturating_sub(1))
    }

    /// Position dans la mesure de l'instant `offset` (relatif au début), `offset < length`.
    fn locate(&self, bar: usize, offset: u64) -> PlanPosition {
        let mut pulse_start = 0;
        let mut pulse = 0;
        loop {
            let length = self.pulse_length(pulse);
            if offset < pulse_start + length || pulse + 1 >= self.timing.pulse_count() {
                break;
            }
            pulse_start += length;
            pulse += 1;
        }
        let tempo = self.pulse_tempo(pulse);
        let steps = self.timing.steps_in_pulse(pulse);
        let within = offset - pulse_start;
        let mut k = 0;
        while k + 1 < steps && self.timing.step_offset(tempo, k + 1) <= within {
            k += 1;
        }
        let at = |relative: u64| Flicks(self.start + pulse_start + relative);
        PlanPosition {
            bar,
            step: (pulse * self.timing.steps_per_pulse() + k) as u16,
            pulse: (pulse + 1) as u8,
            pulses_per_bar: self.timing.pulses_per_bar,
            step_start: at(self.timing.step_offset(tempo, k)),
            next_step: at(self.timing.step_offset(tempo, k + 1)),
            pulse_start: at(0),
            next_pulse: at(self.timing.step_offset(tempo, steps)),
            bar_start: Flicks(self.start),
            next_bar: Flicks(self.start + self.length),
            tempo,
        }
    }
}

/// Position musicale de référence à un instant du plan : ce que la simulation lit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanPosition {
    /// Mesure 1-based.
    pub bar: usize,
    /// Pas 0-based dans la mesure.
    pub step: u16,
    /// Pulse dansée 1-based dans la mesure.
    pub pulse: u8,
    pub pulses_per_bar: u8,
    pub step_start: Flicks,
    pub next_step: Flicks,
    pub pulse_start: Flicks,
    pub next_pulse: Flicks,
    pub bar_start: Flicks,
    pub next_bar: Flicks,
    /// Tempo gelé de la pulse courante.
    pub tempo: Tempo,
}

impl PlanPosition {
    #[must_use]
    pub const fn is_downbeat(&self) -> bool {
        self.pulse == 1
    }
}

/// Carte mesures/pas ↔ instants, en arithmétique entière. La mesure 1 commence à
/// [`Flicks::ZERO`] ; chaque mesure suit la précédente sans trou.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TimePlan {
    bars: Vec<PlannedBar>,
}

impl TimePlan {
    #[must_use]
    pub const fn new() -> Self {
        Self { bars: Vec::new() }
    }

    /// Ajoute la mesure suivante.
    pub fn push(&mut self, timing: BarTiming) {
        let previous = self.bars.last();
        let start = previous.map_or(0, |bar| bar.start + bar.length);
        let bar = PlannedBar::new(timing, start, previous.map(PlannedBar::end_tempo));
        self.bars.push(bar);
    }

    /// Garde les mesures `1..=bars` (rollback : la suite est recalculée).
    pub fn truncate(&mut self, bars: usize) {
        self.bars.truncate(bars);
    }

    /// Nombre de mesures planifiées.
    #[must_use]
    pub fn bars(&self) -> usize {
        self.bars.len()
    }

    /// Fin de la dernière mesure planifiée (horizon du plan).
    #[must_use]
    pub fn end(&self) -> Flicks {
        Flicks(self.bars.last().map_or(0, |bar| bar.start + bar.length))
    }

    fn bar(&self, bar: usize) -> Option<&PlannedBar> {
        self.bars.get(bar.checked_sub(1)?)
    }

    /// En-tête de la mesure 1-based `bar`.
    #[must_use]
    pub fn timing(&self, bar: usize) -> Option<BarTiming> {
        self.bar(bar).map(|planned| planned.timing)
    }

    #[must_use]
    pub fn bar_start(&self, bar: usize) -> Option<Flicks> {
        self.bar(bar).map(|planned| Flicks(planned.start))
    }

    /// Tempo gelé de la pulse 1-based `pulse` de la mesure `bar`.
    #[must_use]
    pub fn pulse_tempo(&self, bar: usize, pulse: u8) -> Option<Tempo> {
        let planned = self.bar(bar)?;
        let index = u32::from(pulse).checked_sub(1)?;
        (index < planned.timing.pulse_count()).then(|| planned.pulse_tempo(index))
    }

    /// Instant du pas 0-based `step` de la mesure `bar`.
    #[must_use]
    pub fn step_instant(&self, bar: usize, step: u16) -> Option<Flicks> {
        let planned = self.bar(bar)?;
        let step = u32::from(step);
        if step >= planned.timing.steps() {
            return None;
        }
        let spp = planned.timing.steps_per_pulse();
        let pulse = step / spp;
        let before: u64 = (0..pulse).map(|p| planned.pulse_length(p)).sum();
        let within = planned.timing.step_offset(planned.pulse_tempo(pulse), step % spp);
        Some(Flicks(planned.start + before + within))
    }

    /// Position musicale à l'instant `at` ; `None` au-delà de l'horizon planifié.
    #[must_use]
    pub fn position_at(&self, at: Flicks) -> Option<PlanPosition> {
        let index = self.bars.partition_point(|bar| bar.start <= at.0).checked_sub(1)?;
        let planned = &self.bars[index];
        let offset = at.0 - planned.start;
        (offset < planned.length).then(|| planned.locate(index + 1, offset))
    }

    /// Premier début de pulse strictement après `at` (dans l'horizon).
    #[must_use]
    pub fn next_pulse_after(&self, at: Flicks) -> Option<Flicks> {
        self.position_at(at).map(|position| position.next_pulse)
    }

    /// Premier début de mesure strictement après `at` (dans l'horizon).
    #[must_use]
    pub fn next_bar_after(&self, at: Flicks) -> Option<Flicks> {
        self.position_at(at).map(|position| position.next_bar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reel(tempo: u32, swing: u16) -> BarTiming {
        BarTiming {
            tempo: Tempo::from_milli_bpm(tempo),
            swing: Swing::from_ten_thousandths(swing),
            numerator: 2,
            denominator: 2,
            steps: 16,
            steps_per_pulse: 8,
            pulses_per_bar: 2,
            ramp_in: true,
        }
    }

    #[test]
    fn quarter_and_frames_are_exact_for_common_rates() {
        // 120 BPM : une noire = 0,5 s = 22 050 frames à 44,1 kHz, 30 frames à 60 Hz.
        let quarter = Tempo::from_milli_bpm(120_000).quarter();
        assert_eq!(quarter, Flicks(FLICKS_PER_SECOND / 2));
        assert_eq!(quarter.to_frames(44_100), 22_050);
        assert_eq!(quarter.frame_floor(60), 30);
        assert_eq!(Flicks::from_frames(30, 60), quarter);
    }

    #[test]
    fn swing_splits_each_quarter_and_keeps_the_pulse_exact() {
        let bar = reel(120_000, 6_000);
        let tempo = bar.tempo;
        let quarter = tempo.quarter().0;
        // Pulse d'une blanche : deux noires ; pas 0-1 longs (0,6 × noire / 2).
        assert_eq!(bar.pulse_length(tempo), 2 * quarter);
        assert_eq!(Flicks(bar.step_offset(tempo, 1)).to_frames(44_100), 6_615);
        assert_eq!(Flicks(bar.step_offset(tempo, 2)).to_frames(44_100), 13_230);
        assert_eq!(bar.step_offset(tempo, 4), quarter);
        assert_eq!(bar.step_offset(tempo, 8), 2 * quarter);
    }

    #[test]
    fn a_tempo_change_ramps_over_the_bar_and_lands_on_the_target() {
        let mut plan = TimePlan::new();
        plan.push(reel(200_000, 5_000));
        plan.push(reel(240_000, 5_000));
        assert_eq!(plan.pulse_tempo(2, 1), Some(Tempo::from_milli_bpm(220_000)));
        assert_eq!(plan.pulse_tempo(2, 2), Some(Tempo::from_milli_bpm(240_000)));
        let mut jump = reel(240_000, 5_000);
        jump.ramp_in = false;
        plan.push(reel(200_000, 5_000));
        plan.push(jump);
        assert_eq!(plan.pulse_tempo(4, 1), Some(Tempo::from_milli_bpm(240_000)));
    }

    #[test]
    fn position_and_next_boundaries_follow_the_plan() {
        let mut plan = TimePlan::new();
        for _ in 0..3 {
            plan.push(reel(230_000, 5_600));
        }
        let bar2 = plan.bar_start(2).unwrap_or_default();
        let position = plan.position_at(bar2).unwrap_or_else(|| unreachable!());
        assert_eq!((position.bar, position.step, position.pulse), (2, 0, 1));
        assert!(position.is_downbeat());
        let step9 = plan.step_instant(2, 9).unwrap_or_default();
        let inside = plan.position_at(Flicks(step9.0 + 1)).unwrap_or_else(|| unreachable!());
        assert_eq!((inside.bar, inside.step, inside.pulse), (2, 9, 2));
        assert_eq!(inside.pulse_start, plan.step_instant(2, 8).unwrap_or_default());
        assert_eq!(plan.next_pulse_after(bar2), plan.step_instant(2, 8));
        assert_eq!(plan.next_bar_after(bar2), plan.bar_start(3));
        assert_eq!(plan.position_at(plan.end()), None);
    }
}

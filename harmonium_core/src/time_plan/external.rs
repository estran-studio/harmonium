//! Horloge externe (phase 013, J6) : le temps musical que suit harmonium vient de la
//! simulation du jeu, pas de l'audio.
//!
//! Le jeu donne la fréquence de sa simulation, la frame où commence la pulsation 1 de la mesure
//! 1 (l'ancrage) et la durée d'**une pulsation dansée** en frames de simulation, sous forme de
//! fraction (`3600/230` permis). Tout le calcul est entier, comme le reste du plan : un instant
//! est un nombre de [`Flicks`] depuis l'ancrage (signé : avant l'ancrage, il est négatif), et
//! le début de la pulsation `n` est arrondi une seule fois depuis `n × durée`, jamais cumulé.

use super::{BarTiming, FLICKS_PER_SECOND, Flicks, Tempo, div_round};

const F: i128 = FLICKS_PER_SECOND as i128;

/// Horloge de simulation du jeu : fréquence, ancrage et durée d'une pulsation dansée.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExternalClock {
    sim_hz: u32,
    anchor_frame: u64,
    pulse_num: u32,
    pulse_den: u32,
}

/// Une horloge externe refusée : un terme nul rend la période absurde.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalClockError {
    /// `sim_hz` nul.
    ZeroRate,
    /// Période de pulsation nulle (`num == 0` ou `den == 0`).
    ZeroPeriod,
}

impl std::fmt::Display for ExternalClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroRate => f.write_str("fréquence de simulation nulle"),
            Self::ZeroPeriod => f.write_str("période de pulsation nulle"),
        }
    }
}

impl std::error::Error for ExternalClockError {}

impl ExternalClock {
    /// `anchor_frame` : frame de simulation où commence la pulsation 1 de la mesure 1.
    /// `pulse_frames_num / pulse_frames_den` : durée d'une pulsation dansée en frames.
    ///
    /// # Errors
    /// Fréquence ou période nulle.
    pub const fn new(
        sim_hz: u32,
        anchor_frame: u64,
        pulse_frames_num: u32,
        pulse_frames_den: u32,
    ) -> Result<Self, ExternalClockError> {
        if sim_hz == 0 {
            return Err(ExternalClockError::ZeroRate);
        }
        if pulse_frames_num == 0 || pulse_frames_den == 0 {
            return Err(ExternalClockError::ZeroPeriod);
        }
        Ok(Self { sim_hz, anchor_frame, pulse_num: pulse_frames_num, pulse_den: pulse_frames_den })
    }

    #[must_use]
    pub const fn sim_hz(&self) -> u32 {
        self.sim_hz
    }

    #[must_use]
    pub const fn anchor_frame(&self) -> u64 {
        self.anchor_frame
    }

    /// Durée d'une pulsation, en frames de simulation : `(numérateur, dénominateur)`.
    #[must_use]
    pub const fn pulse_frames(&self) -> (u32, u32) {
        (self.pulse_num, self.pulse_den)
    }

    /// Durée d'une pulsation, arrondie au flick.
    #[must_use]
    pub const fn pulse_period(&self) -> Flicks {
        Flicks(div_round(
            self.pulse_num as u128 * FLICKS_PER_SECOND as u128,
            self.pulse_den as u128 * self.sim_hz as u128,
        ) as u64)
    }

    /// Instant de la frame de simulation `frame` (et de sa fraction, en millièmes de frame),
    /// en flicks depuis l'ancrage.
    #[must_use]
    pub const fn frame_time(&self, frame: u64, milli_frame: u32) -> i64 {
        let thousandths = (frame as i128 - self.anchor_frame as i128) * 1000 + milli_frame as i128;
        div_round_signed(thousandths * F, 1000 * self.sim_hz as i128) as i64
    }

    /// Début de la pulsation `pulse` (0 : la pulsation 1 de la mesure 1), en flicks depuis
    /// l'ancrage. Un seul arrondi : aucune dérive, même après des heures.
    #[must_use]
    pub const fn pulse_start(&self, pulse: i64) -> i64 {
        div_round_signed(
            pulse as i128 * self.pulse_num as i128 * F,
            self.pulse_den as i128 * self.sim_hz as i128,
        ) as i64
    }

    /// Pulsation (0-based) qui contient l'instant `at` (flicks depuis l'ancrage).
    #[must_use]
    pub const fn pulse_at(&self, at: i64) -> i64 {
        // Estimation par défaut ; l'arrondi du début de pulsation peut décaler d'une unité :
        // on corrige jusqu'à encadrer `at`.
        let mut pulse = (at as i128 * self.pulse_den as i128 * self.sim_hz as i128)
            .div_euclid(self.pulse_num as i128 * F) as i64;
        while self.pulse_start(pulse) > at {
            pulse -= 1;
        }
        while self.pulse_start(pulse + 1) <= at {
            pulse += 1;
        }
        pulse
    }

    /// Tempo à la noire (milli-BPM) dont la pulsation de `timing` dure la période de
    /// l'horloge : `60 000 × 4 × pas_par_pulsation × num / (pas × den) / période`, arrondi au
    /// milli-BPM. `None` hors de 40 à 300 BPM (bornes de `MusicalParams::bpm`).
    #[must_use]
    pub fn tempo_for(&self, timing: &BarTiming) -> Option<Tempo> {
        let numerator = u128::from(timing.numerator.max(1));
        let denominator = u128::from(timing.denominator.max(1));
        let quarter_steps_per_pulse = u128::from(timing.steps_per_pulse()) * numerator * 4;
        let milli = div_round(
            60_000 * quarter_steps_per_pulse * u128::from(self.pulse_den) * u128::from(self.sim_hz),
            u128::from(self.pulse_num) * u128::from(timing.steps.max(1)) * denominator,
        );
        let tempo = u32::try_from(milli).ok()?;
        (Tempo::MIN.milli_bpm()..=Tempo::MAX.milli_bpm())
            .contains(&tempo)
            .then(|| Tempo::from_milli_bpm(tempo))
    }

    /// Erreur de la pulsation que `tempo` fait rendre, en millionièmes de la période exacte :
    /// positif, la pulsation rendue est plus longue (l'audio prend du retard). Le suivi la
    /// rattrape ; elle est au plus de `0,5 / milli-BPM × 10⁶` (4 ppm à 120 BPM).
    #[must_use]
    pub fn rounding_error_ppm(&self, timing: &BarTiming, tempo: Tempo) -> i64 {
        let rendered =
            timing.pulse_length(tempo) as i128 * self.pulse_den as i128 * self.sim_hz as i128;
        let exact = self.pulse_num as i128 * F;
        div_round_signed((rendered - exact) * 1_000_000, exact) as i64
    }
}

/// Division entière arrondie au plus proche, demi vers l'infini positif ; dénominateur > 0.
const fn div_round_signed(numerator: i128, denominator: i128) -> i128 {
    (numerator + denominator / 2).div_euclid(denominator)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock(num: u32, den: u32) -> ExternalClock {
        let Ok(clock) = ExternalClock::new(60, 600, num, den) else {
            unreachable!("période non nulle")
        };
        clock
    }

    #[test]
    fn rejects_zero_terms() {
        assert_eq!(ExternalClock::new(0, 0, 30, 1), Err(ExternalClockError::ZeroRate));
        assert_eq!(ExternalClock::new(60, 0, 0, 1), Err(ExternalClockError::ZeroPeriod));
        assert_eq!(ExternalClock::new(60, 0, 30, 0), Err(ExternalClockError::ZeroPeriod));
    }

    #[test]
    fn frames_and_pulses_are_exact_on_the_anchor() {
        let c = clock(30, 1);
        assert_eq!(c.frame_time(600, 0), 0);
        assert_eq!(c.frame_time(660, 0), FLICKS_PER_SECOND as i64);
        assert_eq!(c.frame_time(0, 0), -10 * FLICKS_PER_SECOND as i64);
        assert_eq!(c.pulse_start(2), FLICKS_PER_SECOND as i64);
        assert_eq!(c.pulse_period(), Flicks(FLICKS_PER_SECOND / 2));
    }

    #[test]
    fn fractional_period_has_a_single_rounding() {
        // 3600/230 frames à 60 Hz : 230 pulsations durent exactement 60 s.
        let c = clock(3600, 230);
        assert_eq!(c.pulse_start(230), 60 * FLICKS_PER_SECOND as i64);
        assert_eq!(c.pulse_start(230 * 1000), 60_000 * FLICKS_PER_SECOND as i64);
    }

    #[test]
    fn pulse_at_brackets_the_instant_before_and_after_the_anchor() {
        let c = clock(3600, 230);
        for pulse in [-5_i64, -1, 0, 1, 7, 229, 100_000] {
            let start = c.pulse_start(pulse);
            assert_eq!(c.pulse_at(start), pulse);
            assert_eq!(c.pulse_at(start + 1), pulse);
            assert_eq!(c.pulse_at(c.pulse_start(pulse + 1) - 1), pulse);
        }
    }

    #[test]
    fn tempo_follows_the_meter_of_the_pulse() {
        // Période : 30 frames à 60 Hz = 0,5 s par pulsation.
        let c = clock(30, 1);
        let timing = |numerator, denominator, steps, steps_per_pulse, pulses_per_bar| BarTiming {
            tempo: Tempo::from_milli_bpm(120_000),
            swing: super::super::Swing::STRAIGHT,
            numerator,
            denominator,
            steps,
            steps_per_pulse,
            pulses_per_bar,
            ramp_in: false,
        };
        // 4/4, pulsation = noire : 120 BPM.
        assert_eq!(c.tempo_for(&timing(4, 4, 16, 4, 4)), Some(Tempo::from_milli_bpm(120_000)));
        // 2/2, pulsation = blanche (deux noires) : 240 BPM à la noire.
        assert_eq!(c.tempo_for(&timing(2, 2, 16, 8, 2)), Some(Tempo::from_milli_bpm(240_000)));
        // 6/8, pulsation = noire pointée (trois croches = 1,5 noire) : 180 BPM à la noire.
        assert_eq!(c.tempo_for(&timing(6, 8, 12, 6, 2)), Some(Tempo::from_milli_bpm(180_000)));
        // Hors bornes : 30 frames de pulsation = 120 BPM ; 1 frame = 3600 BPM.
        assert_eq!(clock(1, 1).tempo_for(&timing(4, 4, 16, 4, 4)), None);
    }

    #[test]
    fn rounding_error_is_bounded_by_half_a_milli_bpm() {
        let c = clock(3600, 230);
        let timing = BarTiming {
            tempo: Tempo::from_milli_bpm(120_000),
            swing: super::super::Swing::STRAIGHT,
            numerator: 4,
            denominator: 4,
            steps: 16,
            steps_per_pulse: 4,
            pulses_per_bar: 4,
            ramp_in: false,
        };
        let tempo = c.tempo_for(&timing).unwrap_or(Tempo::MIN);
        // 3600/230 frames à 60 Hz = 0,26087 s : 230 BPM.
        assert!((229_900..=230_100).contains(&tempo.milli_bpm()));
        let ppm = c.rounding_error_ppm(&timing, tempo);
        assert!(ppm.abs() <= 1_000_000 / (2 * i64::from(tempo.milli_bpm())) + 1, "{ppm} ppm");
    }

    #[test]
    fn rounding_error_stays_under_half_a_milli_bpm_over_the_whole_tempo_range() {
        let timing = BarTiming {
            tempo: Tempo::from_milli_bpm(120_000),
            swing: super::super::Swing::STRAIGHT,
            numerator: 6,
            denominator: 8,
            steps: 12,
            steps_per_pulse: 6,
            pulses_per_bar: 2,
            ramp_in: false,
        };
        for num in (20..=900).step_by(7) {
            for den in [1, 3, 7, 57, 230] {
                let Ok(c) = ExternalClock::new(60, 0, num, den) else { continue };
                let Some(tempo) = c.tempo_for(&timing) else { continue };
                let ppm = c.rounding_error_ppm(&timing, tempo) as f64;
                let bound = 0.5e6 / f64::from(tempo.milli_bpm());
                assert!(ppm.abs() <= bound + 1.0, "{num}/{den} : {ppm} ppm, borne {bound}");
            }
        }
    }
}

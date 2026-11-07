//! Contrat du plan de temps entier (phase 013, J1) : le calcul est entier et sans dérive.

use harmonium_core::time_plan::{BarTiming, FLICKS_PER_SECOND, Flicks, Swing, Tempo, TimePlan};

#[test]
fn integer_plan_has_no_drift_over_ten_thousand_bars() {
    let timing = BarTiming {
        tempo: Tempo::from_bpm(233.37),
        swing: Swing::from_ratio(0.56),
        numerator: 2,
        denominator: 2,
        steps: 16,
        steps_per_pulse: 8,
        pulses_per_bar: 2,
        ramp_in: true,
    };
    let mut plan = TimePlan::new();
    for _ in 0..10_001 {
        plan.push(timing);
    }
    let bar = plan.bar_start(2).unwrap_or_default().0;
    // Une noire arrondie au flick, deux par pulse, deux pulses : la mesure est exacte.
    assert_eq!(bar, 2 * timing.pulse_length(timing.tempo));
    assert_eq!(plan.bar_start(10_001), Some(Flicks(10_000 * bar)));
    for step in 0..16 {
        let first = plan.step_instant(1, step).unwrap_or_default().0;
        let last = plan.step_instant(10_001, step).unwrap_or_default().0;
        assert_eq!(last - first, 10_000 * bar, "pas {step}");
    }
    // 10 000 mesures de 4 noires à 233,37 BPM : 10 284,1 s, sans dérive d'arrondi.
    let seconds = plan.bar_start(10_001).unwrap_or_default().0 / FLICKS_PER_SECOND;
    assert_eq!(seconds, 10_284);
}

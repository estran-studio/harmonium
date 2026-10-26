//! Contrat du plan de temps partagé (phase 013, J1) : deux clients de la simulation, même
//! graine et mêmes commandes, calculent exactement le même temps musical, quel que soit
//! l'ordre d'arrivée des commandes ou les rollbacks ; le calcul est entier et sans dérive.

use harmonium_core::{
    StyleProfile,
    time_plan::{
        BarTiming, FLICKS_PER_SECOND, Flicks, Swing, Tempo, TimePlan,
        score::{CommandDate, DatedCommand, ScoreCommand, SharedTimePlan},
    },
    tuning::TuningParams,
};

const SIM_FPS: u32 = 60;
const BARS: usize = 72;

fn tuning(id: &str) -> TuningParams {
    let mut profile = StyleProfile::named(id).unwrap_or_else(|e| panic!("{e}"));
    profile.arrangement.tours_per_tune = 1;
    TuningParams { profile, ..TuningParams::default() }
}

fn at_bar(bar: usize, order: u64, command: ScoreCommand) -> DatedCommand {
    DatedCommand { order, command, date: CommandDate::Bar(bar) }
}

/// Commandes de jeu sur 72 mesures : colère, tempo direct, retour aux émotions, énergie
/// datée à un instant (frame de simulation), calme.
fn commands() -> Vec<DatedCommand> {
    vec![
        at_bar(
            3,
            10,
            ScoreCommand::Emotions { arousal: 900, valence: -400, density: 800, tension: 700 },
        ),
        at_bar(12, 20, ScoreCommand::Tempo(Tempo::from_milli_bpm(197_500))),
        // Deux commandes de la même mesure : l'ordre de jeu tranche, pas l'arrivée.
        at_bar(20, 31, ScoreCommand::Intensity(200)),
        at_bar(20, 30, ScoreCommand::ReleaseTempo),
        DatedCommand {
            order: 40,
            command: ScoreCommand::Intensity(950),
            date: CommandDate::At(Flicks::from_frames(60 * 40, SIM_FPS)),
        },
        at_bar(
            55,
            50,
            ScoreCommand::Emotions { arousal: 100, valence: 600, density: 300, tension: 100 },
        ),
    ]
}

/// Positions lues par la simulation à chaque frame à 60 Hz sur tout l'horizon.
fn sampled_positions(shared: &SharedTimePlan) -> Vec<(usize, u16, u8, Flicks, Flicks)> {
    let end = shared.plan().end();
    (0..)
        .map(|frame| Flicks::from_frames(frame, SIM_FPS))
        .take_while(|&at| at < end)
        .map(|at| {
            let p = shared.plan().position_at(at).unwrap_or_else(|| panic!("hors plan {at:?}"));
            (p.bar, p.step, p.pulse, p.next_pulse, p.next_bar)
        })
        .collect()
}

#[test]
fn two_clients_compute_the_same_time_whatever_the_arrival_order_and_rollbacks() {
    for id in ["reel_quebecois", "gigue_quebecoise", "valse_quebecoise"] {
        let all = commands();

        // Client A : tout connu d'avance.
        let mut a = SharedTimePlan::new(tuning(id), 7);
        for command in &all {
            a.insert(*command);
        }
        a.extend_to_bar(BARS);

        // Client B : les commandes arrivent en retard et à rebours, après que le plan a
        // déjà été calculé au-delà de leur mesure (rollback), plus une commande
        // mal prédite puis retirée.
        let mut b = SharedTimePlan::new(tuning(id), 7);
        let mispredicted = at_bar(8, 5, ScoreCommand::Tempo(Tempo::from_milli_bpm(120_000)));
        b.insert(mispredicted);
        b.extend_to_bar(30);
        for command in all.iter().rev() {
            b.insert(*command);
            b.extend_to_bar(60);
        }
        b.remove(&mispredicted);
        b.extend_to_bar(BARS);

        // Client C : la simulation lui donne sa liste (état rollbacké) par morceaux, dans
        // un autre ordre, entre deux lectures.
        let mut c = SharedTimePlan::new(tuning(id), 7);
        c.sync(&all[3..]);
        let _ = c.position_at(Flicks::from_frames(60 * 50, SIM_FPS));
        let mut shuffled = all.clone();
        shuffled.swap(0, 4);
        shuffled.swap(1, 3);
        c.sync(&shuffled);
        c.extend_to_bar(BARS);

        // Client D : commandes en retard dans l'ordre, chacune après un plan étendu à 60
        // mesures : chaque rollback repart d'un point de reprise plus tardif.
        let mut d = SharedTimePlan::new(tuning(id), 7);
        d.extend_to_bar(60);
        for command in &all {
            d.insert(*command);
            d.extend_to_bar(60);
        }
        d.extend_to_bar(BARS);

        assert_eq!(a.plan(), b.plan(), "{id} : plans A et B");
        assert_eq!(a.plan(), c.plan(), "{id} : plans A et C");
        assert_eq!(a.plan(), d.plan(), "{id} : plans A et D");
        let positions = sampled_positions(&a);
        assert!(positions.len() > 60 * 60, "{id} : horizon trop court");
        assert_eq!(positions, sampled_positions(&b), "{id} : positions A et B");
        assert_eq!(positions, sampled_positions(&c), "{id} : positions A et C");
        assert_eq!(positions, sampled_positions(&d), "{id} : positions A et D");

        // Les commandes ont bien agi : tempo direct exact à la mesure 12, puis retour.
        let plan = a.plan();
        let last_pulse = |bar: usize| {
            plan.timing(bar).map(|t| t.pulses_per_bar).and_then(|n| plan.pulse_tempo(bar, n))
        };
        assert_eq!(last_pulse(12), Some(Tempo::from_milli_bpm(197_500)), "{id}");
        assert_ne!(last_pulse(25), Some(Tempo::from_milli_bpm(197_500)), "{id}");
        assert_ne!(last_pulse(2), last_pulse(5), "{id} : la colère doit changer le tempo");
    }
}

#[test]
fn a_frame_dated_command_applies_at_the_first_bar_starting_at_or_after_it() {
    let mut shared = SharedTimePlan::new(tuning("reel_quebecois"), 3);
    shared.extend_to_bar(12);
    let bar6 = shared.plan().bar_start(6).unwrap_or_else(|| panic!("mesure 6"));
    let tempo = Tempo::from_milli_bpm(150_000);
    shared.insert(DatedCommand {
        order: 1,
        command: ScoreCommand::Tempo(tempo),
        date: CommandDate::At(Flicks(bar6.0 - 1)),
    });
    shared.extend_to_bar(12);
    let plan = shared.plan();
    assert_ne!(plan.timing(5).map(|t| t.tempo), Some(tempo));
    assert_eq!(plan.timing(6).map(|t| t.tempo), Some(tempo));
    assert_eq!(plan.bar_start(6), Some(bar6), "le passé du plan ne bouge pas");
}

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

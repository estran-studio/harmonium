//! Suivi d'une horloge externe (phase 013, J6) : la simulation du jeu est maîtresse du temps,
//! harmonium règle la vitesse et la phase de son playback dessus.
//!
//! Le [`ExternalFollower`] est une machine à états pure : elle lit les observations de la
//! simulation ([`SimObservation`] : une frame et l'`Instant` où elle vaut) et l'horloge publiée
//! par le callback ([`TransportClock`]), et répond par des [`Action`] (attente, micro-ajustement,
//! réalignement) que [`crate::LiveHost`] envoie au playback par les canaux existants. Rien ne
//! tourne dans le callback ; `BeatInfo` et `MusicClock` restent le playhead (principe II) :
//! le suivi ne touche jamais à la position publiée.
//!
//! # Mesure de l'écart
//!
//! La pulsation `n` (0 : pulsation 1 de la mesure 1) est entendue à l'instant où le callback
//! rend son premier échantillon (`pulse_start_frame` placé sur l'`Instant` audible du tampon, la
//! latence de sortie comprise) ; l'horloge externe la place à `ancrage + n × période`, convertie
//! en `Instant` par un décalage simulation → mur **filtré** (moyenne des dernières observations :
//! la frame de simulation tremble d'une frame). L'écart est la différence des deux instants, en
//! flicks, positif quand l'audio est en avance, comme dans [`CorrectionPolicy::decide`].

use std::{collections::VecDeque, time::Duration};

use harmonium_core::time_plan::{
    FLICKS_PER_SECOND,
    correction::{Correction, CorrectionPolicy},
    external::ExternalClock,
};
use web_time::Instant;

use crate::clock::TransportClock;

const F: f64 = FLICKS_PER_SECOND as f64;

/// Une observation de la simulation : la frame courante, sa fraction si le jeu la connaît, et
/// l'`Instant` où elle vaut (celui de la frame de présentation).
#[derive(Clone, Copy, Debug)]
pub struct SimObservation {
    pub sim_frame: u64,
    /// Fraction de frame écoulée, `0.0..1.0` (`0.0` si inconnue).
    pub frame_fraction: f32,
    pub at: Instant,
}

/// Réglages du suivi ; les défauts sont ceux du contrat J6 et se règlent à l'oreille.
#[derive(Clone, Copy, Debug)]
pub struct FollowConfig {
    /// Zone morte, micro-ajustement borné et fenêtre de rattrapage.
    pub policy: CorrectionPolicy,
    /// Marge entre « maintenant » et l'instant audible d'une mesure choisie pour démarrer ou
    /// sauter : couvre un tampon audio, la file de commandes et le rythme de l'hôte.
    pub start_guard: Duration,
    /// Observations moyennées pour le décalage simulation → mur.
    pub filter_observations: usize,
    /// Observations nécessaires avant de démarrer.
    pub min_observations: usize,
    /// Latence de sortie que la plateforme ne fournit pas (voir la doc de `harmonium_bevy`).
    pub extra_output_latency: Duration,
    /// Un saut de mesure d'au plus ce nombre de mesures en avant reste dans la fenêtre déjà
    /// composée ; au-delà (ou en arrière), l'hôte rejoue le composeur (`Seek`).
    pub soft_jump_bars: usize,
    /// Sans nouveau tampon pendant cette durée, l'audio est considéré arrêté.
    pub run_timeout: Duration,
    /// Saut du décalage simulation → mur au-delà duquel la simulation est jugée discontinue
    /// (trois observations de suite) : le filtre repart de zéro au lieu de glisser vers la
    /// nouvelle valeur, ce qui déclencherait des corrections sur un décalage à moitié filtré.
    pub step_threshold: Duration,
}

impl Default for FollowConfig {
    fn default() -> Self {
        Self {
            policy: CorrectionPolicy::default(),
            start_guard: Duration::from_millis(150),
            filter_observations: 60,
            min_observations: 30,
            extra_output_latency: Duration::ZERO,
            soft_jump_bars: 4,
            run_timeout: Duration::from_millis(250),
            step_threshold: Duration::from_millis(100),
        }
    }
}

/// Où en est le suivi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Le playback attend en silence : audio arrêté, ou pas assez d'observations.
    Waiting,
    /// Un saut est commandé ; il n'a pas encore sonné.
    Starting,
    /// L'audio joue sur l'horloge ; l'écart est mesuré et corrigé.
    Tracking,
}

/// Dernière décision du suivi, pour le diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    /// Démarrage ou reprise : la mesure `bar` à la frame de moteur `frame`.
    Start { bar: usize, frame: u64 },
    /// Saut à la fin de la mesure en cours vers `bar`, à la frame `frame` (nouvelle horloge ou
    /// écart au-delà du micro-ajustement).
    Jump { bar: usize, frame: u64 },
    /// Micro-ajustement (`per_mille > 0` : l'audio va plus vite) sur `pulses` pulsations.
    Nudge { per_mille: i32, pulses: u32 },
}

/// État publié du suivi, lisible à côté de `TransportClock` / `BeatInfo`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExternalStatus {
    pub phase: Phase,
    /// Écart filtré en flicks, positif quand l'audio est en avance sur l'horloge externe ;
    /// `None` tant qu'aucune pulsation n'a été mesurée depuis le dernier saut.
    pub gap: Option<i64>,
    /// Dernière correction décidée.
    pub last_correction: Option<Applied>,
    /// Micro-ajustements décidés depuis la pose de l'horloge.
    pub nudges: u32,
    /// Démarrages et sauts décidés depuis la pose de l'horloge.
    pub resyncs: u32,
}

impl ExternalStatus {
    /// Écart filtré en millisecondes.
    #[must_use]
    pub fn gap_ms(&self) -> Option<f64> {
        self.gap.map(|gap| gap as f64 * 1000.0 / F)
    }
}

/// Ce que l'hôte doit faire pour le suivi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    /// Silence jusqu'à nouvel ordre (audio arrêté ou relancé hors de notre contrôle).
    Hold,
    Nudge {
        ppm: i32,
        pulses: u32,
    },
    Resync {
        bar: usize,
        frame: u64,
        at_bar_end: bool,
        hard: bool,
    },
}

#[derive(Clone, Copy, Debug)]
struct Pending {
    bar: usize,
    /// Premier tampon publié après la commande : une mesure jouée avant n'est pas celle-là.
    issued_buffer_frame: u64,
    /// Instant mur (s depuis `base`) au-delà duquel on abandonne l'attente.
    deadline: f64,
}

/// Machine à états du suivi.
#[derive(Debug)]
pub struct ExternalFollower {
    clock: ExternalClock,
    config: FollowConfig,
    base: Option<Instant>,
    offsets: VecDeque<f64>,
    step_run: u32,
    last_buffer_frame: u64,
    last_advance: Option<f64>,
    phase: Phase,
    pending: Option<Pending>,
    realign: bool,
    gaps: VecDeque<i64>,
    last_sample_pulse: Option<i64>,
    ignore_until_pulse: i64,
    next_decision_pulse: i64,
    last_correction: Option<Applied>,
    nudges: u32,
    resyncs: u32,
}

impl ExternalFollower {
    #[must_use]
    pub fn new(clock: ExternalClock, config: FollowConfig) -> Self {
        Self {
            clock,
            config,
            base: None,
            offsets: VecDeque::new(),
            step_run: 0,
            last_buffer_frame: 0,
            last_advance: None,
            phase: Phase::Waiting,
            pending: None,
            realign: false,
            gaps: VecDeque::new(),
            last_sample_pulse: None,
            ignore_until_pulse: i64::MIN,
            next_decision_pulse: i64::MIN,
            last_correction: None,
            nudges: 0,
            resyncs: 0,
        }
    }

    #[must_use]
    pub const fn clock(&self) -> &ExternalClock {
        &self.clock
    }

    #[must_use]
    pub const fn config(&self) -> &FollowConfig {
        &self.config
    }

    pub const fn set_config(&mut self, config: FollowConfig) {
        self.config = config;
    }

    #[must_use]
    pub fn status(&self) -> ExternalStatus {
        let gap =
            (!self.gaps.is_empty()).then(|| self.gaps.iter().sum::<i64>() / self.gaps.len() as i64);
        ExternalStatus {
            phase: self.phase,
            gap,
            last_correction: self.last_correction,
            nudges: self.nudges,
            resyncs: self.resyncs,
        }
    }

    /// Nouvelle horloge (autre salle, autre tempo) : le décalage simulation → mur reste (sauf
    /// si la fréquence de simulation change), le réalignement se fait à la fin de la mesure en
    /// cours.
    pub fn replace_clock(&mut self, clock: ExternalClock) {
        if clock.sim_hz() != self.clock.sim_hz() {
            self.offsets.clear();
        }
        self.clock = clock;
        if self.phase == Phase::Starting {
            // Le saut commandé visait l'ancienne horloge : on en commande un autre.
            self.phase = Phase::Waiting;
            self.pending = None;
        }
        self.realign = self.phase == Phase::Tracking;
        self.reset_tracking();
    }

    /// L'audio est arrêté ou son playhead a été déplacé par ailleurs (pause, nouvelle graine) :
    /// attendre en silence, puis redémarrer sur l'horloge.
    pub fn restart(&mut self) {
        self.phase = Phase::Waiting;
        self.pending = None;
        self.realign = false;
        self.reset_tracking();
        self.last_advance = None;
    }

    fn reset_tracking(&mut self) {
        self.gaps.clear();
        self.last_sample_pulse = None;
        self.ignore_until_pulse = i64::MIN;
        self.next_decision_pulse = i64::MIN;
    }

    fn secs(&self, at: Instant) -> f64 {
        match self.base {
            Some(base) if at < base => -base.duration_since(at).as_secs_f64(),
            Some(base) => at.duration_since(base).as_secs_f64(),
            None => 0.0,
        }
    }

    fn mean_offset(&self) -> f64 {
        self.offsets.iter().sum::<f64>() / self.offsets.len().max(1) as f64
    }

    fn anchor_secs(&self) -> f64 {
        self.clock.anchor_frame() as f64 / f64::from(self.clock.sim_hz())
    }

    /// Instant mur (s) où la simulation atteint `flicks` depuis l'ancrage.
    fn ext_wall(&self, flicks: i64) -> f64 {
        self.anchor_secs() + flicks as f64 / F + self.mean_offset()
    }

    /// Flicks depuis l'ancrage de l'horloge externe à l'instant mur `wall` (s).
    fn wall_to_ext(&self, wall: f64) -> i64 {
        ((wall - self.mean_offset() - self.anchor_secs()) * F).round() as i64
    }

    /// Instant audible (s) de la frame de moteur `frame`.
    fn audible(&self, clock: &TransportClock, frame: u64) -> f64 {
        self.secs(clock.buffer_start_instant)
            + self.config.extra_output_latency.as_secs_f64()
            + (frame as f64 - clock.buffer_start_frame as f64) / f64::from(clock.sample_rate)
    }

    /// Première pulsation dont le début est à `at` ou après.
    fn first_pulse_from(&self, at: i64) -> i64 {
        let floor = self.clock.pulse_at(at);
        if self.clock.pulse_start(floor) >= at { floor } else { floor + 1 }
    }

    /// Première mesure (1-based) dont le début est à `at` (flicks depuis l'ancrage) ou après.
    fn first_bar_from(&self, at: i64, pulses_per_bar: i64) -> usize {
        let pulse = self.first_pulse_from(at).max(0) as u64;
        usize::try_from(pulse.div_ceil(pulses_per_bar as u64) + 1).unwrap_or(usize::MAX)
    }

    fn bar_frame(&self, bar: usize, pulses_per_bar: i64, clock: &TransportClock) -> u64 {
        let start = self.clock.pulse_start((bar as i64 - 1) * pulses_per_bar);
        let delta = self.ext_wall(start) - self.audible(clock, clock.buffer_start_frame);
        let frames = (delta * f64::from(clock.sample_rate)).round().max(0.0);
        clock.buffer_start_frame.saturating_add(frames as u64)
    }

    /// Une observation de la simulation avec l'horloge publiée : la décision éventuelle.
    /// `pulses_per_bar` : pulsations par mesure du profil.
    pub(crate) fn observe(
        &mut self,
        observation: SimObservation,
        clock: &TransportClock,
        pulses_per_bar: u32,
    ) -> Option<Action> {
        self.base.get_or_insert(observation.at);
        let now = self.secs(observation.at);
        let sim = (observation.sim_frame as f64 + f64::from(observation.frame_fraction))
            / f64::from(self.clock.sim_hz());
        let offset = now - sim;
        if !self.offsets.is_empty()
            && (offset - self.mean_offset()).abs() > self.config.step_threshold.as_secs_f64()
        {
            self.step_run += 1;
            if self.step_run >= 3 {
                self.offsets.clear();
                self.reset_tracking();
            }
        } else {
            self.step_run = 0;
        }
        if self.offsets.len() >= self.config.filter_observations.max(1) {
            self.offsets.pop_front();
        }
        self.offsets.push_back(offset);

        if clock.buffer_start_frame != self.last_buffer_frame {
            self.last_buffer_frame = clock.buffer_start_frame;
            self.last_advance = Some(now);
        }
        let running = self
            .last_advance
            .is_some_and(|advance| now - advance <= self.config.run_timeout.as_secs_f64());
        if !running {
            // Audio arrêté (pause, onglet gelé) : à la relance, le playback doit être déjà
            // muet. L'ordre est idempotent.
            let first = self.phase != Phase::Waiting;
            self.restart();
            return first.then_some(Action::Hold);
        }
        let pulses_per_bar = i64::from(pulses_per_bar.max(1));
        match self.phase {
            Phase::Waiting => self.start(now, clock, pulses_per_bar),
            Phase::Starting => self.await_start(now, clock, pulses_per_bar),
            Phase::Tracking => self.track(now, clock, pulses_per_bar),
        }
    }

    fn start(&mut self, now: f64, clock: &TransportClock, ppb: i64) -> Option<Action> {
        if self.offsets.len() < self.config.min_observations.max(1) {
            return None;
        }
        let audible_now = self.audible(clock, clock.buffer_start_frame).max(now);
        let earliest = audible_now + self.config.start_guard.as_secs_f64();
        let bar = self.first_bar_from(self.wall_to_ext(earliest), ppb);
        let frame = self.bar_frame(bar, ppb, clock);
        Some(self.issue(now, clock, bar, frame, false, true))
    }

    fn issue(
        &mut self,
        now: f64,
        clock: &TransportClock,
        bar: usize,
        frame: u64,
        at_bar_end: bool,
        hard: bool,
    ) -> Action {
        let wait =
            (frame.saturating_sub(clock.buffer_start_frame)) as f64 / f64::from(clock.sample_rate);
        self.pending = Some(Pending {
            bar,
            issued_buffer_frame: clock.buffer_start_frame,
            deadline: now + wait + 2.0,
        });
        self.last_correction = Some(if at_bar_end {
            Applied::Jump { bar, frame }
        } else {
            Applied::Start { bar, frame }
        });
        self.resyncs += 1;
        self.phase = Phase::Starting;
        self.realign = false;
        self.reset_tracking();
        Action::Resync { bar, frame, at_bar_end, hard }
    }

    fn pulse_index(clock: &TransportClock, ppb: i64) -> i64 {
        (clock.position.bar as i64 - 1) * ppb + i64::from(clock.pulse_in_bar)
    }

    fn valid(clock: &TransportClock) -> bool {
        !clock.holding && clock.next_pulse_frame != 0
    }

    fn await_start(&mut self, now: f64, clock: &TransportClock, ppb: i64) -> Option<Action> {
        let pending = self.pending?;
        if Self::valid(clock)
            && clock.position.bar == pending.bar
            && clock.step_start_frame >= pending.issued_buffer_frame
        {
            self.phase = Phase::Tracking;
            self.pending = None;
            let pulse = Self::pulse_index(clock, ppb);
            self.ignore_until_pulse = pulse;
            self.next_decision_pulse = pulse + 2;
            return None;
        }
        if now > pending.deadline {
            // La commande ne s'est pas réalisée (file pleine, mesure absente) : on repart.
            self.phase = Phase::Waiting;
            self.pending = None;
        }
        None
    }

    fn track(&mut self, now: f64, clock: &TransportClock, ppb: i64) -> Option<Action> {
        if !Self::valid(clock) || self.offsets.len() < self.config.min_observations.max(1) {
            return None;
        }
        if self.realign {
            return self.plan_jump(now, clock, ppb);
        }
        let pulse = Self::pulse_index(clock, ppb);
        if self.last_sample_pulse == Some(pulse) {
            return None;
        }
        self.last_sample_pulse = Some(pulse);
        if pulse <= self.ignore_until_pulse {
            return None;
        }
        let heard = self.audible(clock, clock.pulse_start_frame);
        let expected = self.ext_wall(self.clock.pulse_start(pulse));
        let gap = ((expected - heard) * F).round() as i64;
        if self.gaps.len() >= 3 {
            self.gaps.pop_front();
        }
        self.gaps.push_back(gap);
        if pulse < self.next_decision_pulse || self.gaps.len() < 2 {
            return None;
        }
        let filtered = self.gaps.iter().sum::<i64>() / self.gaps.len() as i64;
        match self.config.policy.decide(filtered, self.clock.pulse_period()) {
            Correction::None => None,
            Correction::Nudge { per_mille, pulses } => {
                self.last_correction = Some(Applied::Nudge { per_mille, pulses });
                self.nudges += 1;
                let span = i64::from(pulses);
                self.ignore_until_pulse = pulse + span;
                self.next_decision_pulse = pulse + span + 2;
                self.gaps.clear();
                Some(Action::Nudge { ppm: per_mille.saturating_mul(1000), pulses })
            }
            Correction::JumpAtNextBar => self.plan_jump(now, clock, ppb),
        }
    }

    /// Saut à la fin de la mesure en cours vers la première mesure de l'horloge qui commence
    /// (à la zone morte près) après cette fin. Attend, sans rien décider, quand la fin de la
    /// mesure est trop proche pour que la commande arrive à temps.
    fn plan_jump(&mut self, now: f64, clock: &TransportClock, ppb: i64) -> Option<Action> {
        let pulse_frames = clock.next_pulse_frame.checked_sub(clock.pulse_start_frame)?;
        let remaining = ppb - i64::from(clock.pulse_in_bar);
        let end_frame = clock.pulse_start_frame + pulse_frames * u64::try_from(remaining).ok()?;
        let end = self.audible(clock, end_frame);
        let audible_now = self.audible(clock, clock.buffer_start_frame).max(now);
        if end < audible_now + self.config.start_guard.as_secs_f64() {
            return None;
        }
        let threshold = self.wall_to_ext(end) - self.config.policy.dead_zone.0 as i64;
        let bar = self.first_bar_from(threshold, ppb);
        let next = clock.position.bar + 1;
        let hard = bar < next || bar - next > self.config.soft_jump_bars;
        let frame = self.bar_frame(bar, ppb, clock);
        Some(self.issue(now, clock, bar, frame, true, hard))
    }
}

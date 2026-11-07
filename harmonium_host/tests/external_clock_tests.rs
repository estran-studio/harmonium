//! Suivre l'horloge du jeu (phase 013, J6) : des clients simulés (hôte détaché, playback piloté
//! tampon par tampon, horloge audio qui dérive, gestes de démarrage décalés, simulation qui
//! tremble d'une frame) entendent la même pulsation au même instant de simulation.
//!
//! Chaque client reçoit à chaque frame de simulation (60 Hz) une observation `(frame, Instant)`
//! et l'horloge publiée par le callback ; l'hôte démarre le playback sur une mesure de
//! l'horloge, puis le corrige. La vérité de terrain est le modèle du périphérique du test :
//! la frame de moteur `f` est audible à `geste + latence + f / (fs × (1 + dérive))`.

use std::time::{Duration, Instant};

use harmonium::{
    ClockReader, LiveHost, TransportClock,
    external::{Applied, ExternalStatus, Phase, SimObservation},
    playback::PlaybackEngine,
};
use harmonium_core::{
    EngineCommand, StyleProfile,
    time_plan::{FLICKS_PER_SECOND, external::ExternalClock},
};

const SIM_HZ: u32 = 60;
const BUFFER: usize = 512;
/// Latence de sortie du périphérique simulé.
const LATENCY: f64 = 0.06;
const F: f64 = FLICKS_PER_SECOND as f64;

/// Une pulsation entendue : sa frame de moteur et son instant audible réel (modèle du test).
#[derive(Clone, Copy, Debug)]
struct Pulse {
    index: i64,
    bar: usize,
    audible: f64,
}

struct Config {
    profile: &'static str,
    sample_rate: u32,
    /// Écart de l'horloge audio avec la simulation : `+80` ppm, l'audio va plus vite.
    drift_ppm: f64,
    /// Instant (s de la salle) du geste qui ouvre le flux audio.
    gesture: f64,
    /// Gigue de la frame de simulation observée : `±1` frame si vrai.
    jitter: bool,
    /// Décalage entre l'`Instant` local du client et le temps de la salle.
    local_skew: f64,
    seed: u64,
}

struct Client {
    config: Config,
    host: LiveHost,
    playback: PlaybackEngine,
    reader: ClockReader,
    base: Instant,
    pulses_per_bar: i64,
    clock: ExternalClock,
    rendered: u64,
    started: bool,
    rng: u64,
    heard: Vec<Pulse>,
    /// Instants de salle, début et fin, des attentes (`holding`) vues après le démarrage.
    holds: Vec<(f64, Option<f64>)>,
    /// Décalage brusque du périphérique audio (s) : toutes les frames deviennent audibles plus
    /// tard de ce délai (écart forcé, par exemple un blocage du pilote).
    device_shift: f64,
}

impl Client {
    fn new(config: Config, clock: ExternalClock) -> Result<Self, String> {
        let (mut host, playback) =
            LiveHost::detached(None, None, &[], f64::from(config.sample_rate))?;
        let reader = host.take_clock_reader().ok_or("lecteur d'horloge")?;
        let profile = StyleProfile::named(config.profile).map_err(|e| e.to_string())?;
        let pulses_per_bar = profile.meter.pulses_per_bar as i64;
        host.apply(EngineCommand::SetWriteheadLookahead(harmonium::playback::MEASURE_WINDOW_CAP))?;
        host.apply(EngineCommand::SetStyleProfile(Box::new(profile)))?;
        host.set_external_clock(clock)?;
        host.apply(EngineCommand::SetSeed(config.seed))?;
        let seed = config.seed;
        Ok(Self {
            base: Instant::now() + Duration::from_secs_f64(config.local_skew + 1.0),
            config,
            host,
            playback,
            reader,
            pulses_per_bar,
            clock,
            rendered: 0,
            started: false,
            rng: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            heard: Vec::new(),
            holds: Vec::new(),
            device_shift: 0.0,
        })
    }

    fn device_rate(&self) -> f64 {
        f64::from(self.config.sample_rate) * (1.0 + self.config.drift_ppm * 1e-6)
    }

    /// Instant audible, en secondes de salle, de la frame de moteur `frame`.
    fn audible(&self, frame: u64) -> f64 {
        self.config.gesture + self.device_shift + LATENCY + frame as f64 / self.device_rate()
    }

    fn at(&self, room: f64) -> Instant {
        self.base + Duration::from_secs_f64(room.max(0.0))
    }

    fn next_jitter(&mut self) -> i64 {
        self.rng = self
            .rng
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.rng >> 33) % 3) as i64 - 1
    }

    /// Vérité de terrain : instant de salle où l'horloge externe place la pulsation `n`.
    fn ext_secs(&self, pulse: i64) -> f64 {
        self.clock.pulse_start(pulse) as f64 / F
            + self.clock.anchor_frame() as f64 / f64::from(SIM_HZ)
    }

    /// Fait avancer le client jusqu'au temps de salle `room` (une frame de simulation).
    fn step(&mut self, frame: u64) -> Result<(), String> {
        let room = frame as f64 / f64::from(SIM_HZ);
        if room >= self.config.gesture {
            self.started = true;
        }
        let mut buffer = [0.0f32; BUFFER * 2];
        while self.started
            && self.audible(self.rendered) - self.device_shift - LATENCY - 0.02 <= room
        {
            let instant = self.at(self.audible(self.rendered));
            self.playback.process_buffer(&mut buffer, 2, instant);
            self.rendered += BUFFER as u64;
        }
        let clock = self.reader.read();
        self.record(&clock, room);
        let jitter = if self.config.jitter { self.next_jitter() } else { 0 };
        let observed = (frame as f64 + jitter as f64).max(0.0);
        self.host.observe_external(
            SimObservation {
                sim_frame: observed.floor() as u64,
                frame_fraction: observed.fract() as f32,
                at: self.at(room),
            },
            &clock,
        )?;
        self.host.pump();
        Ok(())
    }

    fn record(&mut self, clock: &TransportClock, room: f64) {
        let valid = !clock.holding && clock.next_pulse_frame != 0;
        let holding = self.started && !valid;
        match self.holds.last_mut() {
            Some((_, end @ None)) => {
                if !holding {
                    *end = Some(room);
                }
            }
            _ => {
                if holding {
                    self.holds.push((room, None));
                }
            }
        }
        if !valid {
            return;
        }
        let index =
            (clock.position.bar as i64 - 1) * self.pulses_per_bar + i64::from(clock.pulse_in_bar);
        let audible = self.audible(clock.pulse_start_frame);
        if self.heard.last().is_none_or(|last| last.audible != audible) {
            self.heard.push(Pulse { index, bar: clock.position.bar, audible });
        }
    }

    fn status(&self) -> ExternalStatus {
        self.host.external_status().unwrap_or_else(|| unreachable!("pas d'horloge externe"))
    }

    /// Écart `horloge externe − audio` de chaque pulsation entendue depuis `from` (s de salle).
    fn gaps(&self, from: f64) -> Vec<(f64, f64)> {
        self.heard
            .iter()
            .filter(|pulse| pulse.audible >= from)
            .map(|pulse| (pulse.audible, self.ext_secs(pulse.index) - pulse.audible))
            .collect()
    }
}

fn run(clients: &mut [Client], from_frame: u64, seconds: f64) -> Result<u64, String> {
    let end = from_frame + (seconds * f64::from(SIM_HZ)) as u64;
    for frame in from_frame..end {
        for client in clients.iter_mut() {
            client.step(frame)?;
        }
    }
    Ok(end)
}

/// Client sans dérive ni gigue : le périphérique et la simulation vont exactement au même pas.
fn steady(seed: u64) -> Config {
    Config {
        profile: "reel_quebecois",
        sample_rate: 48_000,
        drift_ppm: 0.0,
        gesture: 0.3,
        jitter: false,
        local_skew: 0.0,
        seed,
    }
}

fn clock(anchor: u64, num: u32, den: u32) -> Result<ExternalClock, String> {
    ExternalClock::new(SIM_HZ, anchor, num, den).map_err(|e| e.to_string())
}

fn stats(gaps: &[(f64, f64)]) -> (f64, f64) {
    let max = gaps.iter().map(|(_, gap)| gap.abs()).fold(0.0, f64::max);
    let mean = gaps.iter().map(|(_, gap)| gap.abs()).sum::<f64>() / gaps.len().max(1) as f64;
    (max * 1000.0, mean * 1000.0)
}

/// Preuve 1 : deux clients (44,1 kHz à +80 ppm, 48 kHz à −80 ppm), gestes à 0,3 s et 2,1 s,
/// simulation à ± 1 frame : après convergence, chaque pulsation tombe sur l'horloge externe à la
/// zone morte près (10 ms, plus la résolution de la mesure), pendant plus de 10 minutes.
#[test]
fn two_drifting_clients_hear_the_same_pulse_at_the_same_simulation_instant() -> Result<(), String> {
    // 3600/57 frames à 60 Hz : 1,0526 s par blanche, 114 BPM à la noire.
    let room = clock(120, 3600, 57)?;
    let a = Config {
        profile: "reel_quebecois",
        sample_rate: 44_100,
        drift_ppm: 80.0,
        gesture: 0.3,
        jitter: true,
        local_skew: 0.0,
        seed: 7,
    };
    let b = Config {
        profile: "reel_quebecois",
        sample_rate: 48_000,
        drift_ppm: -80.0,
        gesture: 2.1,
        jitter: true,
        local_skew: 5_000.0,
        seed: 7,
    };
    let mut clients = [Client::new(a, room)?, Client::new(b, room)?];
    run(&mut clients, 0, 660.0)?;

    for (label, client) in ["A 44,1 kHz +80 ppm", "B 48 kHz −80 ppm"].iter().zip(&clients) {
        let gaps = client.gaps(60.0);
        assert!(gaps.len() > 500, "{label} : {} pulsations", gaps.len());
        let (max, mean) = stats(&gaps);
        let status = client.status();
        eprintln!(
            "{label} : écart max {max:.2} ms, moyen {mean:.2} ms sur {} pulsations ; {status:?}",
            gaps.len()
        );
        assert!(max <= 12.0, "{label} : écart max {max:.2} ms");
        assert_eq!(status.phase, Phase::Tracking, "{label}");
    }

    // Même mesure entendue au même instant de simulation : à chaque seconde, la mesure qui
    // sonne chez A est celle qui sonne chez B (hors des 25 ms autour d'un début de mesure).
    let heard_bar = |client: &Client, at: f64| {
        client.heard.iter().rev().find(|pulse| pulse.audible <= at).map(|pulse| pulse.bar)
    };
    let bar_secs = 2.0 * 3600.0 / 57.0 / f64::from(SIM_HZ);
    let mut compared = 0;
    let mut at = 120.0;
    while at < 650.0 {
        let phase = ((at - 2.0) / bar_secs).fract() * bar_secs;
        if phase > 0.025 && phase < bar_secs - 0.025 {
            assert_eq!(heard_bar(&clients[0], at), heard_bar(&clients[1], at), "à {at} s");
            compared += 1;
        }
        at += 1.0;
    }
    assert!(compared > 450);
    Ok(())
}

/// Preuve 2 : la première pulsation audible est un début de mesure de l'horloge, à ± 1 frame
/// audio de la frame que l'hôte avait prévue, et à moins de 10 ms de l'instant de la simulation.
#[test]
fn the_first_audible_pulse_is_a_bar_start_of_the_clock() -> Result<(), String> {
    let room = clock(300, 3600, 57)?;
    for (rate, gesture) in [(44_100, 0.3), (48_000, 2.1), (96_000, 7.77)] {
        let config = Config {
            profile: "reel_quebecois",
            sample_rate: rate,
            drift_ppm: 0.0,
            gesture,
            jitter: true,
            local_skew: 3.0,
            seed: 3,
        };
        let mut clients = [Client::new(config, room)?];
        run(&mut clients, 0, 20.0)?;
        let client = &clients[0];
        let first = client.heard.first().ok_or("rien n'a joué")?;
        assert_eq!(first.index % 2, 0, "{rate} Hz : pas un début de mesure");
        assert_eq!(first.bar as i64, first.index / 2 + 1);
        let Some(Applied::Start { bar, frame }) = client.status().last_correction else {
            return Err(format!("{rate} Hz : pas de démarrage décidé"));
        };
        assert_eq!(first.bar, bar, "{rate} Hz");
        let predicted = client.audible(frame);
        let heard_frame = (first.audible - client.audible(0)) * client.device_rate();
        assert!(
            (first.audible - predicted).abs() * client.device_rate() <= 1.0,
            "{rate} Hz : première pulsation à {heard_frame}, prévue {frame}"
        );
        let truth = client.ext_secs(first.index);
        assert!(
            (first.audible - truth).abs() < 0.010,
            "{rate} Hz : {} ms",
            (first.audible - truth) * 1e3
        );
        assert!(first.audible > gesture + LATENCY, "{rate} Hz : joue avant le geste");
        eprintln!(
            "{rate} Hz geste {gesture} s : mesure {bar}, frame {frame}, écart à l'instant de simulation {:.2} ms",
            (first.audible - truth) * 1e3
        );
    }
    Ok(())
}

/// Preuve 3 : la pulsation rendue dure la période de l'horloge, fractionnaire ou non, dans les
/// mètres 2/2, 6/8, 4/4 (et 3/4, 12/8, 4/4 ternaire) : moyenne sur 64 pulsations.
#[test]
fn the_rendered_pulse_lasts_the_clock_period() -> Result<(), String> {
    // Un tempo de 230 BPM à la noire, exprimé en pulsation du mètre.
    let cases = [
        ("reel_quebecois", 7200, 230),
        ("gigue_quebecoise", 5400, 230),
        ("gmd_pop", 3600, 230),
        ("valse_quebecoise", 3600, 230),
        ("nt_desert", 5400, 230),
        ("gmd_blues_shuffle", 3600, 230),
    ];
    for (profile, num, den) in cases {
        let room = clock(0, num, den)?;
        let config = Config {
            profile,
            sample_rate: 48_000,
            drift_ppm: 0.0,
            gesture: 0.0,
            jitter: false,
            local_skew: 0.0,
            seed: 5,
        };
        let mut clients = [Client::new(config, room)?];
        run(&mut clients, 0, 60.0)?;
        let client = &clients[0];
        assert_eq!(client.status().nudges, 0, "{profile}");
        let pulses: Vec<_> = client.heard.windows(2).collect();
        assert!(pulses.len() > 64 + 4, "{profile} : {} pulsations", pulses.len());
        let window = &client.heard[client.heard.len() - 65..];
        let mean = (window[64].audible - window[0].audible) / 64.0;
        let period = f64::from(num) / f64::from(den) / f64::from(SIM_HZ);
        let ppm = (mean / period - 1.0) * 1e6;
        // Borne : arrondi du tempo au milli-BPM, plus une frame audio répartie sur 64 pulsations.
        let tempo_ppm = 0.5 / (230.0 * 1000.0) * 1e6;
        let frames_ppm = 1.0 / (64.0 * period * 48_000.0) * 1e6;
        eprintln!(
            "{profile} {num}/{den} : pulsation moyenne {:.6} s, période {:.6} s, {ppm:+.2} ppm (borne {:.2})",
            mean,
            period,
            tempo_ppm + frames_ppm
        );
        assert!(ppm.abs() <= tempo_ppm + frames_ppm + 0.5, "{profile} : {ppm} ppm");
    }
    Ok(())
}

/// Preuve 4a : un écart forcé de 25 ms est ramené sous 10 ms par un micro-ajustement de 5 ‰ au
/// plus, en 8 pulsations.
#[test]
fn a_25_ms_gap_is_nudged_back_under_the_dead_zone_in_eight_pulses() -> Result<(), String> {
    // 40 frames : 0,667 s par blanche ; 25 ms sur 8 pulsations = 4,7 ‰.
    let room = clock(120, 40, 1)?;
    let mut clients = [Client::new(steady(9), room)?];
    let frame = run(&mut clients, 0, 30.0)?;
    let before = clients[0].status();
    assert_eq!(before.phase, Phase::Tracking);
    let (max_before, _) = stats(&clients[0].gaps(15.0));
    assert!(max_before < 3.0, "avant : {max_before} ms");
    // Le pilote audio se bloque 25 ms : tout ce qui n'est pas rendu sonne 25 ms plus tard.
    clients[0].device_shift = 0.025;
    let mut nudge_at = None;
    for frame in frame..frame + 60 * 20 {
        clients[0].step(frame)?;
        if nudge_at.is_none() && clients[0].status().nudges > before.nudges {
            nudge_at = Some(frame);
        }
    }
    let nudge_at = nudge_at.ok_or("aucun micro-ajustement décidé")?;
    let status = clients[0].status();
    assert_eq!(status.resyncs, before.resyncs, "pas de saut pour 25 ms : {status:?}");
    let Some(Applied::Nudge { per_mille, pulses }) = status.last_correction else {
        return Err(format!("dernière correction {:?}", status.last_correction));
    };
    assert!(per_mille.abs() <= 5 && pulses == 8, "{per_mille} ‰ sur {pulses}");
    // Huit pulsations après la décision, l'écart est sous la zone morte et y reste.
    let started = nudge_at as f64 / f64::from(SIM_HZ);
    let settled = started + 9.0 * 40.0 / f64::from(SIM_HZ);
    let (max_after, mean_after) = stats(&clients[0].gaps(settled));
    eprintln!(
        "écart forcé 25 ms : première décision {:?}, {} micro-ajustements ; écart max après {max_after:.2} ms, moyen {mean_after:.2} ms",
        status.last_correction, status.nudges
    );
    assert!(max_after < 10.0, "après : {max_after} ms");
    Ok(())
}

/// Preuve 4b : un écart de 300 ms ne se corrige pas par la vitesse : saut à la mesure suivante,
/// sans silence de plus d'une mesure.
#[test]
fn a_300_ms_gap_jumps_to_the_next_bar() -> Result<(), String> {
    let room = clock(120, 40, 1)?;
    let mut clients = [Client::new(steady(9), room)?];
    let frame = run(&mut clients, 0, 30.0)?;
    let before = clients[0].status();
    clients[0].device_shift = 0.3;
    let holds_before = clients[0].holds.len();
    let end = run(&mut clients, frame, 30.0)?;
    let status = clients[0].status();
    assert_eq!(status.nudges, before.nudges, "pas de micro-ajustement pour 300 ms : {status:?}");
    assert_eq!(status.resyncs, before.resyncs + 1, "un seul saut : {status:?}");
    assert!(matches!(status.last_correction, Some(Applied::Jump { .. })), "{status:?}");
    let bar_secs = 2.0 * 40.0 / f64::from(SIM_HZ);
    assert!(clients[0].holds.len() > holds_before, "le saut doit se voir dans l'horloge");
    for (start, stop) in &clients[0].holds[holds_before..] {
        let stop = stop.ok_or("attente jamais levée")?;
        assert!(stop - start <= bar_secs + 0.05, "silence de {:.2} s", stop - start);
    }
    let (max, _) = stats(&clients[0].gaps(end as f64 / f64::from(SIM_HZ) - 10.0));
    eprintln!("après le saut de 300 ms : écart max {max:.2} ms ; {status:?}");
    assert!(max < 12.0, "{max} ms");
    Ok(())
}

/// Preuve 5 : une nouvelle horloge (autre salle, autre tempo) remplace l'ancienne : l'audio finit
/// sa mesure puis se réaligne à la mesure suivante, sans silence de plus d'une mesure.
#[test]
fn a_new_clock_realigns_at_the_next_bar() -> Result<(), String> {
    let first = clock(120, 40, 1)?;
    let config = Config { drift_ppm: 20.0, jitter: true, ..steady(4) };
    let mut clients = [Client::new(config, first)?];
    let frame = run(&mut clients, 0, 30.0)?;
    // Autre salle, commencée 10 s plus tôt, autre tempo : 50 frames la blanche (0,833 s) au lieu
    // de 40 (0,667 s).
    let second = clock(frame - 600, 50, 1)?;
    clients[0].host.set_external_clock(second)?;
    clients[0].clock = second;
    let holds_before = clients[0].holds.len();
    let end = run(&mut clients, frame, 40.0)?;
    let client = &clients[0];
    let bar_secs = 2.0 * 50.0 / f64::from(SIM_HZ);
    assert!(client.holds.len() > holds_before, "le réalignement doit se voir dans l'horloge");
    for (start, stop) in &client.holds[holds_before..] {
        let stop = stop.ok_or("attente jamais levée")?;
        assert!(
            stop - start <= bar_secs + 0.05,
            "silence de {:.2} s ({:?}) {:?}",
            stop - start,
            client.holds,
            client.host.external_status()
        );
    }
    let status = client.status();
    assert!(matches!(status.last_correction, Some(Applied::Jump { .. })), "{status:?}");
    let gaps = client.gaps(end as f64 / f64::from(SIM_HZ) - 15.0);
    let (max, mean) = stats(&gaps);
    eprintln!("nouvelle horloge : écart max {max:.2} ms, moyen {mean:.2} ms ; {status:?}");
    assert!(max < 12.0, "{max} ms");
    // La pulsation rendue dure la nouvelle période.
    let window = &client.heard[client.heard.len() - 17..];
    let mean_pulse = (window[16].audible - window[0].audible) / 16.0;
    assert!((mean_pulse - 50.0 / 60.0).abs() < 0.002, "{mean_pulse}");
    Ok(())
}

//! Contrat jeu → LiveHost, sans périphérique audio ni attente murale.
use std::time::{Duration, Instant};

use bevy::{asset::AssetPlugin, ecs::message::Messages, prelude::*, time::TimeUpdateStrategy};
use harmonium::{LiveHost, playback::PlaybackEngine};
use harmonium_bevy::{
    Applied, Bar, Beat, ExternalClock, GameClock, HarmoniumConfig, HarmoniumControl,
    HarmoniumPlugin, HarmoniumRuntime, MusicClock, Phase,
};

const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES: usize = 512;
const BUFFER_TIME: Duration = Duration::from_nanos(11_609_977);

type TestResult = Result<(), String>;

struct Game {
    app: App,
    playback: PlaybackEngine,
    frames: u64,
}

impl Game {
    fn new(config: HarmoniumConfig, control: HarmoniumControl) -> Result<Self, String> {
        Self::with_bank(config, control, None)
    }

    fn with_bank(
        config: HarmoniumConfig,
        control: HarmoniumControl,
        bank: Option<&[u8]>,
    ) -> Result<Self, String> {
        let (host, playback) = LiveHost::detached(bank, None, &[], SAMPLE_RATE)?;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
            .add_plugins(HarmoniumPlugin::new(config))
            .insert_resource(control)
            .insert_non_send(HarmoniumRuntime::new(host)?);
        // La première update initialise les commandes et compose, mais ne rend rien.
        app.update();
        Ok(Self { app, playback, frames: 0 })
    }

    fn runtime(&self) -> &HarmoniumRuntime {
        self.app.world().non_send::<HarmoniumRuntime>()
    }

    fn control(&mut self) -> Mut<'_, HarmoniumControl> {
        self.app.world_mut().resource_mut::<HarmoniumControl>()
    }

    fn clock(&self) -> &MusicClock {
        self.app.world().resource::<MusicClock>()
    }

    fn update(&mut self, elapsed: Duration) {
        self.app.insert_resource(TimeUpdateStrategy::ManualDuration(elapsed));
        self.app.update();
    }

    fn render(&mut self, frames: usize) -> [f32; FRAMES * 2] {
        let mut output = [0.0; FRAMES * 2];
        self.playback.process_buffer(&mut output[..frames * 2], 2, Instant::now());
        self.frames += frames as u64;
        output
    }

    fn step(&mut self, elapsed: Duration) -> [f32; FRAMES * 2] {
        let output = self.render(FRAMES);
        self.update(elapsed);
        output
    }

    fn prime_clock(&mut self) -> TestResult {
        // L'hôte publie l'entrée d'un pas au tampon suivant ou à sa complétion,
        // pas nécessairement dans le premier petit tampon qui la rend.
        for _ in 0..8 {
            self.step(BUFFER_TIME);
            if self.clock().valid {
                return Ok(());
            }
        }
        Err("aucune horloge publiée après huit tampons".into())
    }

    fn messages(&mut self) -> (Vec<(usize, u8)>, Vec<usize>) {
        let beats = self
            .app
            .world_mut()
            .resource_mut::<Messages<Beat>>()
            .drain()
            .map(|message| (message.bar, message.pulse))
            .collect();
        let bars = self
            .app
            .world_mut()
            .resource_mut::<Messages<Bar>>()
            .drain()
            .map(|message| message.bar)
            .collect();
        (beats, bars)
    }
}

fn config() -> HarmoniumConfig {
    HarmoniumConfig { pump_interval: Duration::from_millis(10), ..Default::default() }
}

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0001, "{actual} attendu : {expected}");
}

#[test]
fn initial_game_targets_reach_the_host_before_the_first_audio_buffer() -> TestResult {
    let mut control = HarmoniumControl {
        emotions: [0.8, 0.2, 0.7, 0.3],
        volume: 0.6,
        seed: 731,
        ..Default::default()
    };
    control.set_profile_id("nt_desert")?;
    let game = Game::new(config(), control)?;
    let runtime = game.runtime();
    let host = runtime.host();
    assert_eq!(host.profile().meta.id, "nt_desert");
    assert_eq!(host.emotions(), [0.8, 0.2, 0.7, 0.3]);
    assert_eq!(host.session_seed(), 731);
    assert_close(runtime.volume(), 0.6);
    assert!(!runtime.paused());
    assert!(!game.clock().valid, "aucune horloge jouée avant le callback");
    Ok(())
}

#[test]
fn new_seed_and_volume_targets_are_applied_while_the_game_runs() -> TestResult {
    let mut game = Game::new(config(), HarmoniumControl::default())?;
    for _ in 0..4 {
        game.step(BUFFER_TIME);
    }
    {
        let mut control = game.control();
        control.seed = 98_765;
        control.volume = 0.35;
    }
    game.step(BUFFER_TIME);
    assert_eq!(game.runtime().host().session_seed(), 98_765);
    assert_close(game.runtime().volume(), 0.35);
    for _ in 0..4 {
        game.step(BUFFER_TIME);
    }
    assert_eq!(game.runtime().host().session_seed(), 98_765);
    Ok(())
}

#[test]
fn emotions_obey_real_time_cadence_and_accumulate_against_the_last_sent_target() -> TestResult {
    let initial = [0.4, 0.4, 0.4, 0.4];
    let mut game = Game::new(
        HarmoniumConfig {
            emotion_interval: Duration::from_millis(100),
            emotion_threshold: 0.02,
            ..config()
        },
        HarmoniumControl { emotions: initial, ..Default::default() },
    )?;
    let target = [0.8, 0.1, 0.9, 0.2];
    game.control().emotions = target;
    for _ in 0..2 {
        game.step(Duration::from_millis(40));
        assert_eq!(game.runtime().host().emotions(), initial, "pas avant 100 ms");
    }
    game.step(Duration::from_millis(20));
    assert_eq!(game.runtime().host().emotions(), target);

    for axis in 0..4 {
        let before = game.runtime().host().emotions();
        let mut small = before;
        small[axis] += 0.01;
        game.control().emotions = small;
        game.step(Duration::from_millis(100));
        assert_eq!(game.runtime().host().emotions(), before, "axe {axis} sous le seuil");
        small[axis] += 0.005;
        game.control().emotions = small;
        game.step(Duration::from_millis(100));
        assert_eq!(game.runtime().host().emotions(), before, "pas de dérive du point de référence");
        small[axis] += 0.016;
        game.control().emotions = small;
        game.step(Duration::from_millis(100));
        assert_eq!(game.runtime().host().emotions(), small, "axe {axis} au-delà du seuil");
    }
    Ok(())
}

#[test]
fn pause_freezes_the_played_clock_and_resume_does_not_repeat_its_messages() -> TestResult {
    let mut game = Game::new(config(), HarmoniumControl::default())?;
    game.prime_clock()?;
    assert!(game.clock().valid && game.clock().playing);
    game.messages();
    game.control().paused = true;
    game.update(BUFFER_TIME);
    assert!(game.runtime().paused());
    assert!(!game.clock().playing);
    let position = (game.clock().bar, game.clock().pulse, game.clock().beat);
    // Une vraie pause suspend les callbacks : ne pas en fabriquer dans le test.
    for _ in 0..4 {
        game.update(Duration::from_secs(1));
        assert_eq!((game.clock().bar, game.clock().pulse, game.clock().beat), position);
        assert!(!game.clock().playing);
        assert_eq!(game.messages(), (vec![], vec![]));
    }
    game.control().paused = false;
    game.update(Duration::ZERO);
    assert!(!game.runtime().paused());
    assert!(game.clock().playing);
    assert_eq!(game.messages(), (vec![], vec![]));
    game.step(BUFFER_TIME);
    assert!(game.clock().playing);
    assert_eq!(game.messages(), (vec![], vec![]), "même pulse après reprise");
    Ok(())
}

#[test]
fn initially_paused_control_waits_for_resume_and_a_real_buffer_to_validate_the_clock() -> TestResult
{
    let mut game = Game::new(config(), HarmoniumControl { paused: true, ..Default::default() })?;
    assert!(game.runtime().paused());
    assert!(!game.clock().valid && !game.clock().playing);
    game.update(Duration::from_secs(2));
    assert!(!game.clock().valid);
    assert_eq!(game.messages(), (vec![], vec![]));
    game.control().paused = false;
    game.update(Duration::ZERO);
    assert!(!game.clock().valid, "reprendre ne fabrique pas un tampon");
    game.prime_clock()?;
    assert!(game.clock().valid && game.clock().playing);
    Ok(())
}

#[test]
fn profile_changes_only_at_rendered_silence_then_fades_back_and_restarts_at_bar_one() -> TestResult
{
    let mut game = Game::with_bank(
        HarmoniumConfig { fade_duration: Duration::from_millis(100), ..config() },
        HarmoniumControl { volume: 0.8, ..Default::default() },
        Some(include_bytes!("../../assets/test.sf2")),
    )?;
    for _ in 0..1_000 {
        game.step(BUFFER_TIME);
        if game.clock().bar >= 2 {
            break;
        }
    }
    assert!(game.clock().bar >= 2, "le transport initial a réellement avancé");
    let old = game.runtime().host().profile().meta.id.clone();
    game.control().set_profile_id("nt_desert")?;
    game.update(Duration::ZERO);
    for _ in 0..3 {
        game.step(Duration::from_millis(25));
        assert_eq!(game.runtime().host().profile().meta.id, old);
        assert!(game.runtime().volume() > 0.0 && game.runtime().volume() < 0.8);
    }
    game.step(Duration::from_millis(25));
    assert_close(game.runtime().volume(), 0.0);
    assert_eq!(game.runtime().host().profile().meta.id, old, "zéro commandé, pas encore rendu");
    for _ in 0..3 {
        game.update(Duration::from_secs(1));
        assert_close(game.runtime().volume(), 0.0);
        assert_eq!(game.runtime().host().profile().meta.id, old, "attente du callback, pas du mur");
    }
    game.render(64);
    game.update(Duration::ZERO);
    assert_eq!(
        game.runtime().host().profile().meta.id,
        old,
        "64 frames ne terminent pas la rampe 10 ms"
    );
    for _ in 0..8 {
        let output = game.step(Duration::ZERO);
        if game.runtime().host().profile().meta.id == "nt_desert" {
            assert!(
                output.iter().all(|sample| *sample == 0.0),
                "profil remplacé seulement au silence rendu"
            );
            break;
        }
    }
    assert_eq!(game.runtime().host().profile().meta.id, "nt_desert");
    assert_close(game.runtime().volume(), 0.0);
    // La cadence de composition reste active : laisser passer une échéance,
    // puis rendre le tampon qui consomme la nouvelle mesure.
    game.update(Duration::from_millis(10));
    game.prime_clock()?;
    assert_eq!(game.clock().bar, 1, "le nouveau profil ne conserve pas l'ancienne mesure");
    game.step(Duration::from_millis(25));
    assert!(game.runtime().volume() > 0.0 && game.runtime().volume() < 0.8);
    for _ in 0..3 {
        game.step(Duration::from_millis(25));
    }
    assert_close(game.runtime().volume(), 0.8);
    assert_eq!(game.runtime().host().profile().meta.id, "nt_desert");
    Ok(())
}

#[test]
fn changing_emotions_during_a_seed_restart_does_not_starve_bar_one() -> TestResult {
    let mut game = Game::new(config(), HarmoniumControl::default())?;
    for _ in 0..1_000 {
        game.step(BUFFER_TIME);
        if game.clock().bar >= 2 {
            break;
        }
    }
    assert!(game.clock().bar >= 2);
    game.control().seed = 77;
    game.update(Duration::ZERO);
    // Une cible neuve avant que le callback ait consommé le redémarrage ne doit
    // pas faire repartir la génération depuis l'ancien playhead (mesure 2).
    let emotions = [0.9, 0.1, 0.8, 0.9];
    game.control().emotions = emotions;
    game.update(Duration::from_millis(200));
    game.prime_clock()?;
    assert_eq!(game.clock().bar, 1);
    game.step(Duration::from_millis(100));
    assert_eq!(game.runtime().host().emotions(), emotions);
    assert_eq!(game.runtime().host().session_seed(), 77);
    Ok(())
}

#[test]
fn a_new_profile_target_replaces_the_pending_one_even_while_paused_at_zero() -> TestResult {
    let mut game = Game::new(
        HarmoniumConfig { fade_duration: Duration::from_millis(100), ..config() },
        HarmoniumControl::default(),
    )?;
    game.step(BUFFER_TIME);
    let old = game.runtime().host().profile().meta.id.clone();
    game.control().set_profile_id("nt_desert")?;
    game.update(Duration::ZERO);
    game.step(Duration::from_millis(50));
    game.control().set_profile_id("isaac_basement")?;
    game.step(Duration::from_millis(50));
    assert_close(game.runtime().volume(), 0.0);
    game.control().paused = true;
    game.update(Duration::ZERO);
    for _ in 0..4 {
        game.update(Duration::from_secs(1));
        assert_eq!(game.runtime().host().profile().meta.id, old);
        assert_close(game.runtime().volume(), 0.0);
    }
    game.control().paused = false;
    game.update(Duration::ZERO);
    for _ in 0..8 {
        game.step(Duration::ZERO);
        let id = game.runtime().host().profile().meta.id.clone();
        assert_ne!(id, "nt_desert", "la cible remplacée ne doit jamais être appliquée");
        if id == "isaac_basement" {
            break;
        }
    }
    assert_eq!(game.runtime().host().profile().meta.id, "isaac_basement");
    for _ in 0..4 {
        game.step(Duration::from_millis(25));
    }
    assert_close(game.runtime().volume(), game.app.world().resource::<HarmoniumControl>().volume);
    Ok(())
}

#[test]
fn switching_profile_inside_bar_one_replaces_the_measure_actually_rendered() -> TestResult {
    let mut game = Game::new(
        HarmoniumConfig { fade_duration: Duration::ZERO, ..config() },
        HarmoniumControl::default(),
    )?;
    game.prime_clock()?;
    assert_eq!(game.clock().bar, 1);
    assert_eq!(game.clock().pulses_per_bar, 2, "2/2 effectivement rendu");
    game.control().set_profile_id("nt_desert")?;
    game.update(Duration::ZERO);
    for _ in 0..32 {
        game.step(BUFFER_TIME);
        if game.runtime().host().profile().meta.id == "nt_desert" && game.clock().valid {
            assert_eq!(game.clock().bar, 1);
            assert_eq!(
                game.clock().pulses_per_bar,
                4,
                "la mesure entendue est celle du nouveau profil"
            );
            return Ok(());
        }
    }
    Err("transition non acquittée".into())
}

#[test]
fn the_audio_clock_publishes_every_pulse_and_bar_once_with_a_quarter_note_bpm() -> TestResult {
    let mut control = HarmoniumControl::default();
    control.set_profile_id("nt_desert")?;
    let mut game = Game::new(config(), control)?;
    assert!(!game.clock().valid);
    assert_eq!(game.messages(), (vec![], vec![]));
    game.prime_clock()?;
    assert!(game.clock().valid && game.clock().playing);
    assert_eq!((game.clock().bar, game.clock().pulse), (1, 1));
    assert_eq!(game.messages(), (vec![(1, 1)], vec![1]));
    let meter = game.runtime().host().profile().meter.clone();
    let pulses = meter.pulses_per_bar as u8;
    let quarters_per_pulse = meter.time_signature.numerator as f64 * 4.0
        / meter.time_signature.denominator as f64
        / f64::from(pulses);
    assert_eq!(game.clock().pulses_per_bar, pulses);
    let mut previous = (1, 1);
    let mut previous_boundary = None;
    let mut observed_beats = 1;
    let mut observed_bars = vec![1];
    for _ in 0..2_000 {
        game.step(BUFFER_TIME);
        let current = (game.clock().bar, game.clock().pulse);
        assert!(game.clock().valid && game.clock().playing);
        assert!((0.0..1.0).contains(&game.clock().phase));
        assert!(game.clock().beat >= 1.0);
        assert!(game.clock().bpm.is_finite() && game.clock().bpm > 0.0);
        let (beats, bars) = game.messages();
        if current == previous {
            assert!(beats.is_empty() && bars.is_empty(), "aucun doublon par frame");
        } else {
            let next = if previous.1 == pulses {
                (previous.0 + 1, 1)
            } else {
                (previous.0, previous.1 + 1)
            };
            assert_eq!(current, next, "aucune frontière callback perdue");
            assert_eq!(beats, vec![current]);
            if current.1 == 1 {
                assert_eq!(bars, vec![current.0]);
                observed_bars.extend(bars);
            } else {
                assert!(bars.is_empty());
            }
            if let Some(frame) = previous_boundary {
                let measured_bpm =
                    60.0 * SAMPLE_RATE * quarters_per_pulse / (game.frames - frame) as f64;
                let relative_error = (f64::from(game.clock().bpm) / measured_bpm - 1.0).abs();
                assert!(
                    relative_error < 0.08,
                    "BPM noire {} contre {measured_bpm} mesuré par callback",
                    game.clock().bpm
                );
            }
            previous_boundary = Some(game.frames);
            observed_beats += 1;
            previous = current;
        }
        // Une update sans nouveau tampon ne doit rien réémettre, même avec du temps jeu.
        game.update(Duration::from_millis(20));
        assert_eq!((game.clock().bar, game.clock().pulse), current);
        assert_eq!(game.messages(), (vec![], vec![]));
        if current == (3, 1) {
            break;
        }
    }
    assert_eq!(previous, (3, 1));
    assert_eq!(observed_beats, usize::from(pulses) * 2 + 1);
    assert_eq!(observed_bars, vec![1, 2, 3]);
    Ok(())
}

fn rendered_signal(bank: &[u8], volume: f32) -> Result<Vec<f32>, String> {
    let mut game = Game::with_bank(
        config(),
        HarmoniumControl { volume, seed: 3, ..Default::default() },
        Some(bank),
    )?;
    let mut signal = Vec::with_capacity(400 * FRAMES * 2);
    for _ in 0..400 {
        signal.extend_from_slice(&game.step(BUFFER_TIME));
    }
    Ok(signal)
}

#[test]
fn game_volume_scales_the_actual_soundfont_signal_and_zero_is_silent() -> TestResult {
    let bank = include_bytes!("../../assets/test.sf2");
    let full = rendered_signal(bank, 1.0)?;
    let quarter = rendered_signal(bank, 0.25)?;
    let mute = rendered_signal(bank, 0.0)?;
    // Exclut largement la rampe anti-clic initiale (441 frames à 44,1 kHz).
    let after_ramp = 4 * FRAMES * 2;
    let full = &full[after_ramp..];
    let quarter = &quarter[after_ramp..];
    let energy: f64 = full.iter().map(|sample| f64::from(*sample).powi(2)).sum();
    assert!(energy > 0.01, "la banque tiny doit réellement produire un signal");
    let error: f64 = full
        .iter()
        .zip(quarter)
        .map(|(full, quarter)| f64::from(*quarter - *full * 0.25).powi(2))
        .sum();
    assert!(error / energy < 1e-8, "le volume jeu doit réduire chaque échantillon");
    assert!(
        mute[after_ramp..].iter().all(|sample| *sample == 0.0),
        "zéro : silence exact après rampe"
    );
    Ok(())
}

/// Le jeu joue à 60 Hz ; son périphérique audio rend un tampon dès qu'il est dû. Le temps de
/// Bevy (`Time<Real>`, avancé à la main) est le seul temps mur : le plugin y lit l'instant de
/// chaque frame, le périphérique simulé y date chaque tampon.
struct Room {
    game: Game,
    sim_frame: u64,
    rendered: u64,
    started: Option<Instant>,
}

const GAME_HZ: f64 = 60.0;
const LATENCY: Duration = Duration::from_millis(60);

impl Room {
    fn new(profile: &str) -> Result<Self, String> {
        let mut control = HarmoniumControl { seed: 11, ..Default::default() };
        control.set_profile_id(profile)?;
        let game = Game::new(config(), control)?;
        Ok(Self { game, sim_frame: 0, rendered: 0, started: None })
    }

    fn now(&self) -> Instant {
        self.game.app.world().resource::<Time<Real>>().last_update().unwrap_or_else(Instant::now)
    }

    fn set_clock(&mut self, clock: Option<ExternalClock>) {
        self.game.app.world_mut().resource_mut::<GameClock>().clock = clock;
    }

    /// Une frame de simulation : le jeu écrit la frame, Bevy avance de 1/60 s, puis le
    /// périphérique rend les tampons dus.
    fn frame(&mut self) {
        self.sim_frame += 1;
        self.game.app.world_mut().resource_mut::<GameClock>().sim_frame = self.sim_frame;
        self.game.update(Duration::from_secs_f64(1.0 / GAME_HZ));
        let now = self.now();
        let started = *self.started.get_or_insert(now);
        loop {
            let audible =
                started + LATENCY + Duration::from_secs_f64(self.rendered as f64 / SAMPLE_RATE);
            if audible > now + LATENCY + Duration::from_millis(20) {
                break;
            }
            let mut output = [0.0; FRAMES * 2];
            self.game.playback.process_buffer(&mut output, 2, audible);
            self.rendered += FRAMES as u64;
        }
    }

    fn run(&mut self, seconds: f64) {
        for _ in 0..(seconds * GAME_HZ) as u64 {
            self.frame();
        }
    }
}

#[test]
fn the_game_clock_sets_the_tempo_holds_the_start_and_is_followed() -> TestResult {
    let mut room = Room::new("reel_quebecois")?;
    // La musique n'a pas d'horloge à suivre : tempo du profil, rien ne change.
    assert!(room.game.runtime().host().external_status().is_none());
    assert_eq!(room.game.runtime().host().musical_params().bpm, 230.0);

    // La pulsation (la blanche du reel) dure 40 frames à 60 Hz : 180 BPM à la noire.
    let first = ExternalClock::new(60, 90, 40, 1).map_err(|e| e.to_string())?;
    room.set_clock(Some(first));
    room.frame();
    assert_eq!(room.game.runtime().host().external_clock(), Some(&first));
    assert_eq!(room.game.runtime().host().musical_params().bpm, 180.0);
    room.run(0.2);
    assert!(!room.game.clock().valid, "rien ne sonne avant la mesure de l'horloge");

    room.run(8.0);
    let clock = *room.game.clock();
    let follow = clock.follow.ok_or("le suivi n'est pas publié")?;
    assert!(clock.valid && clock.playing, "{clock:?}");
    assert_eq!(follow.phase, Phase::Tracking, "{follow:?}");
    assert!(matches!(follow.last_correction, Some(Applied::Start { .. })), "{follow:?}");
    // L'audio tombe sur l'horloge : la mesure entendue est celle que la simulation compte.
    let sim_secs = room.sim_frame as f64 / GAME_HZ - 90.0 / GAME_HZ;
    let expected_bar = (sim_secs / (2.0 * 40.0 / GAME_HZ)) as usize + 1;
    assert!(clock.bar.abs_diff(expected_bar) <= 1, "mesure {} attendue {expected_bar}", clock.bar);
    assert_close(clock.bpm, 180.0);
    assert!(follow.gap_ms().is_none_or(|gap| gap.abs() < 10.0), "{follow:?}");

    // Autre salle, autre tempo : 50 frames, 144 BPM ; l'audio se réaligne sans repartir de 1.
    let second = ExternalClock::new(60, room.sim_frame - 300, 50, 1).map_err(|e| e.to_string())?;
    room.set_clock(Some(second));
    room.run(14.0);
    assert_eq!(room.game.runtime().host().musical_params().bpm, 144.0);
    let follow = room.game.clock().follow.ok_or("le suivi n'est pas publié")?;
    assert_eq!(follow.phase, Phase::Tracking, "{follow:?}");
    assert!(matches!(follow.last_correction, Some(Applied::Jump { .. })), "{follow:?}");
    assert!(follow.gap_ms().is_none_or(|gap| gap.abs() < 10.0), "{follow:?}");
    assert_close(room.game.clock().bpm, 144.0);

    // Le jeu retire l'horloge : les émotions reprennent le tempo, le suivi disparaît.
    room.set_clock(None);
    room.frame();
    assert!(room.game.runtime().host().external_clock().is_none());
    assert!(room.game.clock().follow.is_none());
    assert_ne!(room.game.runtime().host().musical_params().bpm, 144.0);
    Ok(())
}

#[test]
fn an_unreachable_period_is_refused_and_keeps_the_previous_state() -> TestResult {
    let mut room = Room::new("reel_quebecois")?;
    // 1 frame par blanche : 3 600 BPM à la noire, hors de 40 à 300.
    let absurd = ExternalClock::new(60, 0, 1, 1).map_err(|e| e.to_string())?;
    room.set_clock(Some(absurd));
    room.frame();
    assert!(room.game.runtime().host().external_clock().is_none());
    assert_eq!(room.game.runtime().host().musical_params().bpm, 230.0);
    Ok(())
}

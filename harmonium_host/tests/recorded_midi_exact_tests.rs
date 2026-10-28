//! Le MIDI enregistré (`--record-midi`) coïncide avec la vérité du compositeur.
//!
//! Le moteur hors ligne est piloté comme `harmonium_cli::export` : profil,
//! graine, émotions constantes, enregistrement MIDI du premier au dernier
//! pas. Chaque attaque enregistrée est comparée à sa position écrite dans les
//! snapshots (pas + décalage V2 en millièmes de pas), passée par la grille
//! de swing audible quand la mesure en porte une. Le résidu est exprimé en
//! ticks (480 par noire) ; le contrat est ±1 tick, sans dérive.

use std::{collections::BTreeMap, time::Instant};

use harmonium::{audio::create_offline_engine, playback::PlaybackCommand};
use harmonium_core::{
    events::{AudioRecordFormat, RecordFormat},
    report::MeasureSnapshot,
};
use midly::{MidiMessage, Smf, TrackEventKind};

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;
const BUFFER_SIZE: usize = 256;
const TICKS_PER_STEP: f64 = 120.0;
const BARS: usize = 30;
const SEEDS: u64 = 20;
const PROFILES: [&str; 3] = ["reel_quebecois", "gigue_quebecoise", "valse_quebecoise"];
const TENSIONS: [f32; 2] = [0.4, 1.0];

struct Render {
    midi: Vec<u8>,
    measures: BTreeMap<usize, MeasureSnapshot>,
}

/// Réglages d'export qui déplacent les attaques dans le temps audible.
#[derive(Clone, Copy, Default)]
struct Options {
    swing: Option<f32>,
    /// `(mesure, bpm, pulses de rampe)` : `SetTempo` envoyé à l'entrée de la mesure.
    tempo: Option<(usize, f32, u8)>,
}

fn render(
    profile: &str,
    seed: u64,
    tension: f32,
    bars: usize,
    options: Options,
) -> Result<Render, String> {
    let (mut composer, mut playback, mut cmd_tx, mut report_rx, _clock, finished) =
        create_offline_engine(None, None, &[], SAMPLE_RATE)?;
    composer.set_seed(seed);
    composer.set_profile_id(profile)?;
    composer.use_emotion_mode();
    composer.set_emotions(0.5, 0.3, 0.2, tension);
    if let Some(ratio) = options.swing {
        composer.set_swing_ratio(ratio);
    }
    composer.invalidate_future();
    if let Some((bar, _)) = composer.take_pending_drop() {
        let _ = cmd_tx.push(PlaybackCommand::DropFrom { bar });
    }
    for (channel, program) in composer.take_pending_program_changes() {
        let _ = cmd_tx.push(PlaybackCommand::ProgramChange { channel, program });
    }
    let _ = cmd_tx
        .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()));
    composer.set_writehead_lookahead(bars + 16);
    composer.generate_bars(bars.max(32));
    let mut measures = BTreeMap::new();
    let mut collect = |composer: &mut harmonium::composer::MusicComposer| {
        for snapshot in composer.take_snapshots() {
            measures.insert(snapshot.index, snapshot);
        }
    };
    collect(&mut composer);
    let _ = cmd_tx.push(PlaybackCommand::StartRecording(AudioRecordFormat::Midi));
    let mut buffer = vec![0.0f32; BUFFER_SIZE * CHANNELS];
    let mut tempo_sent = false;
    while composer.playhead_bar() <= bars {
        if let Some((bar, bpm, ramp_pulses)) = options.tempo
            && !tempo_sent
            && composer.playhead_bar() >= bar
        {
            let _ = cmd_tx.push(PlaybackCommand::SetTempo { bpm, ramp_pulses });
            tempo_sent = true;
        }
        composer.generate_ahead();
        collect(&mut composer);
        buffer.fill(0.0);
        playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
        while report_rx.pop().is_ok() {}
    }
    let _ = cmd_tx.push(PlaybackCommand::StopRecording(AudioRecordFormat::Midi));
    buffer.fill(0.0);
    playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
    let queue = finished.lock().map_err(|e| e.to_string())?;
    let midi = queue
        .iter()
        .find(|(format, _)| *format == RecordFormat::Midi)
        .map(|(_, data)| data.clone())
        .ok_or("un enregistrement MIDI")?;
    Ok(Render { midi, measures })
}

/// Position en ticks d'une coordonnée continue `x` (en pas depuis la mesure
/// `bar`), grille de swing audible comprise : le quart de 4 pas est découpé
/// en `[r, r, 1-r, 1-r] / 2` quand la mesure swingue (même condition que
/// `PlaybackEngine::compute_pulse_steps`).
fn truth_ticks(measures: &BTreeMap<usize, MeasureSnapshot>, bar: usize, x: f64) -> f64 {
    let (mut bar, mut x) = (bar, x);
    while x < 0.0 {
        let Some(previous) = measures.get(&(bar - 1)) else { break };
        bar -= 1;
        x += previous.steps as f64;
    }
    while let Some(measure) = measures.get(&bar)
        && x >= measure.steps as f64
        && measures.contains_key(&(bar + 1))
    {
        x -= measure.steps as f64;
        bar += 1;
    }
    let start: usize = measures.range(..bar).map(|(_, m)| m.steps).sum();
    let Some(measure) = measures.get(&bar) else { return start as f64 * TICKS_PER_STEP };
    let steps_per_quarter = (measure.steps * measure.time_sig_denominator) as f64
        / (measure.time_sig_numerator * 4) as f64;
    let swung = measure.swing_ratio > 0.5
        && steps_per_quarter == 4.0
        && usize::from(measure.pulse.steps_per_pulse) % 4 == 0;
    let steps = if swung {
        let r = f64::from(measure.swing_ratio);
        let quarter = (x / 4.0).floor();
        let local = x - quarter * 4.0;
        let fraction =
            if local < 2.0 { local / 2.0 * r } else { r + (local - 2.0) / 2.0 * (1.0 - r) };
        (quarter + fraction) * 4.0
    } else {
        x
    };
    (start as f64 + steps) * TICKS_PER_STEP
}

/// Résidus `(enregistré − vérité)` en ticks pour les attaques des pistes
/// Melody/Chords/Bass, plus le nombre d'attaques enregistrées sans vérité.
fn residuals(render: &Render, bars: usize) -> Result<(Vec<f64>, usize), String> {
    let smf = Smf::parse(&render.midi).map_err(|e| e.to_string())?;
    let mut recorded: BTreeMap<(u8, u8), Vec<f64>> = BTreeMap::new();
    let mut tick = 0u32;
    for event in &smf.tracks[0] {
        tick += event.delta.as_int();
        if let TrackEventKind::Midi { channel, message: MidiMessage::NoteOn { key, vel } } =
            event.kind
            && vel.as_int() > 0
            && channel.as_int() <= 2
        {
            recorded.entry((channel.as_int(), key.as_int())).or_default().push(f64::from(tick));
        }
    }
    let mut truth: BTreeMap<(u8, u8), Vec<f64>> = BTreeMap::new();
    for (bar, measure) in render.measures.range(..=bars) {
        for note in measure.notes.iter().filter(|note| note.track <= 2) {
            let x = note.start_step as f64 + f64::from(note.performance.offset) / 1000.0;
            truth.entry((note.track, note.pitch)).or_default().push(truth_ticks(
                &render.measures,
                *bar,
                x,
            ));
        }
    }
    let mut out = Vec::new();
    let mut unmatched = 0;
    for (key, ticks) in recorded {
        let mut candidates = truth.remove(&key).unwrap_or_default();
        candidates.sort_by(f64::total_cmp);
        for tick in ticks {
            let best = candidates
                .iter()
                .enumerate()
                .filter(|(_, candidate)| (tick - **candidate).abs() < 60.0)
                .min_by(|a, b| (tick - a.1).abs().total_cmp(&(tick - b.1).abs()))
                .map(|(index, _)| index);
            match best {
                Some(index) => out.push(tick - candidates.remove(index)),
                None => unmatched += 1,
            }
        }
    }
    Ok((out, unmatched))
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * p) as usize]
}

/// Résidus (ticks) et attaques sans vérité, ou l'erreur de rendu.
type SeedResiduals = Result<(Vec<f64>, usize), String>;

/// Résidus d'un couple profil × tension sur toutes les graines.
fn residuals_over_seeds(profile: &str, tension: f32) -> SeedResiduals {
    let mut all = Vec::new();
    let mut unmatched = 0;
    for seed in 1..=SEEDS {
        let rendered = render(profile, seed, tension, BARS, Options::default())?;
        let (residual, missing) = residuals(&rendered, BARS)?;
        all.extend(residual);
        unmatched += missing;
    }
    Ok((all, unmatched))
}

/// 20 graines × 3 profils × tensions 0,4 et 1, une tâche par couple (le rendu
/// de débogage est lent). `cargo test --release` rejoue le tout en quelques secondes.
#[test]
fn recorded_midi_matches_written_positions_within_one_tick() -> Result<(), String> {
    let results: Vec<(&str, f32, SeedResiduals)> = std::thread::scope(|scope| {
        let tasks: Vec<_> = PROFILES
            .iter()
            .flat_map(|profile| TENSIONS.iter().map(move |tension| (*profile, *tension)))
            .map(|(profile, tension)| {
                (profile, tension, scope.spawn(move || residuals_over_seeds(profile, tension)))
            })
            .collect();
        tasks
            .into_iter()
            .map(|(profile, tension, task)| {
                let result = task.join().unwrap_or_else(|_| Err("tâche paniquée".to_string()));
                (profile, tension, result)
            })
            .collect()
    });
    let mut worst = 0.0f64;
    for (profile, tension, result) in results {
        let (mut all, unmatched) = result?;
        all.sort_by(f64::total_cmp);
        let max = all.iter().map(|r| r.abs()).fold(0.0, f64::max);
        println!(
            "{profile} tension {tension}: {} attaques, {unmatched} sans vérité, résidu ticks \
             médian {:+.1} [p10 {:+.1}; p90 {:+.1}] max |r| {max:.1}",
            all.len(),
            percentile(&all, 0.5),
            percentile(&all, 0.1),
            percentile(&all, 0.9),
        );
        assert!(all.len() > 1000, "{profile}: attaques appariées");
        worst = worst.max(max);
    }
    assert!(worst <= 1.0, "résidu maximal {worst:.2} ticks > 1");
    Ok(())
}

/// Swing audible et rampe de tempo : les croches décalées et les pulses
/// accélérées restent sur la grille écrite (le swing y est une position, la
/// rampe un simple changement de cadence).
#[test]
fn swing_and_tempo_ramp_stay_on_the_written_grid() -> Result<(), String> {
    let options = Options { swing: Some(0.62), tempo: Some((6, 150.0, 8)) };
    let mut checked = 0;
    let mut worst = 0.0f64;
    for profile in ["reel_quebecois", "valse_quebecoise"] {
        for seed in 1..=4 {
            let rendered = render(profile, seed, 0.4, 20, options)?;
            assert!(
                rendered.measures.values().any(|m| m.swing_ratio > 0.5),
                "{profile}: le swing doit être actif"
            );
            let (residual, _) = residuals(&rendered, 20)?;
            checked += residual.len();
            worst = residual.iter().fold(worst, |acc, r| acc.max(r.abs()));
        }
    }
    assert!(checked > 500, "attaques appariées : {checked}");
    assert!(worst <= 1.0, "résidu maximal {worst:.2} ticks > 1");
    Ok(())
}

//! REPL (Read-Eval-Print Loop) for Harmonium CLI
//!
//! Uses the decoupled MusicComposer + PlaybackEngine architecture.
//! Generation commands go to the composer directly; playback commands
//! go to the PlaybackEngine via a lock-free ring buffer.

use std::sync::{Arc, Mutex};

use anyhow::Result;
use colored::Colorize;
use harmonium::{composer::MusicComposer, playback::PlaybackCommand, BeatInfo, ClockReader};
use harmonium_core::{events::RecordFormat, EngineCommand, EngineReport};
use rustyline::{error::ReadlineError, Editor};

use crate::{
    completer::HarmoniumCompleter,
    help,
    parser::{self, CliCommand},
};

/// Emotion state for relative adjustments
struct EmotionState {
    arousal: f32,
    valence: f32,
    density: f32,
    tension: f32,
}

impl Default for EmotionState {
    fn default() -> Self {
        Self { arousal: 0.5, valence: 0.0, density: 0.5, tension: 0.5 }
    }
}

/// REPL state wrapping all engine handles
struct ReplState {
    composer: Arc<Mutex<MusicComposer>>,
    playback_cmd_tx: rtrb::Producer<PlaybackCommand>,
    report_rx: rtrb::Consumer<EngineReport>,
    clock_reader: ClockReader,
    cached_state: Option<EngineReport>,
    emotion_state: EmotionState,
    finished_recordings: Arc<Mutex<Vec<(RecordFormat, Vec<u8>)>>>,
}

impl ReplState {
    fn poll_reports(&mut self) {
        while let Ok(report) = self.report_rx.pop() {
            self.cached_state = Some(report);
        }
    }

    fn get_state(&self) -> Option<&EngineReport> {
        self.cached_state.as_ref()
    }

    /// After param changes: forward the pending eviction (if the timeline
    /// changed), generate 1 bar ahead for playback, and sync musical params
    /// to the playback engine for reporting.
    fn send_invalidate(&mut self) {
        if let Ok(mut c) = self.composer.lock() {
            if let Some((bar, _)) = c.take_pending_drop() {
                let _ = self.playback_cmd_tx.push(PlaybackCommand::DropFrom { bar });
            }
            for (channel, program) in c.take_pending_program_changes() {
                let _ =
                    self.playback_cmd_tx.push(PlaybackCommand::ProgramChange { channel, program });
            }
            c.generate_bars(1);
            let _ = self
                .playback_cmd_tx
                .push(PlaybackCommand::UpdateMusicalParams(Box::new(c.musical_params().clone())));
        }
    }
}

/// Run the interactive REPL
pub fn run(
    composer: Arc<Mutex<MusicComposer>>,
    playback_cmd_tx: rtrb::Producer<PlaybackCommand>,
    report_rx: rtrb::Consumer<EngineReport>,
    clock_reader: ClockReader,
    finished_recordings: Arc<Mutex<Vec<(RecordFormat, Vec<u8>)>>>,
) -> Result<()> {
    let mut state = ReplState {
        composer,
        playback_cmd_tx,
        report_rx,
        clock_reader,
        cached_state: None,
        emotion_state: EmotionState::default(),
        finished_recordings,
    };

    // Create readline editor with autocomplete
    let helper = HarmoniumCompleter::new();
    let mut rl = Editor::new()?;
    rl.set_helper(Some(helper));

    // Load history if it exists
    let history_path = dirs::home_dir().map(|mut p| {
        p.push(".harmonium_history");
        p
    });

    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    // Print welcome message
    print_welcome();

    // Main REPL loop
    loop {
        // Poll for reports from engine
        state.poll_reports();

        // Poll for finished recordings and save them
        if let Ok(mut queue) = state.finished_recordings.lock() {
            while let Some((format, data)) = queue.pop() {
                let filename = match format {
                    RecordFormat::Wav => "output.wav",
                    RecordFormat::Midi => "output.mid",
                    RecordFormat::MusicXml => "output.musicxml",
                };

                match std::fs::write(filename, &data) {
                    Ok(_) => {
                        println!(
                            "\n{} Saved {} ({} bytes)",
                            "[RECORDING]".cyan().bold(),
                            filename.green(),
                            data.len()
                        );
                    }
                    Err(e) => {
                        println!(
                            "\n{} Failed to write {}: {}",
                            "[ERROR]".red().bold(),
                            filename,
                            e
                        );
                    }
                }
            }
        }

        // Create prompt with current state
        let prompt = create_prompt(&state);

        // Read line
        match rl.readline(&prompt) {
            Ok(line) => {
                // Add to history
                let _ = rl.add_history_entry(line.trim());

                // Handle command
                match handle_command(&line, &mut state) {
                    Ok(should_continue) => {
                        if !should_continue {
                            break; // User requested quit
                        }
                    }
                    Err(e) => {
                        println!("{} {}", "[ERROR]".red().bold(), e);
                    }
                }
            }

            Err(ReadlineError::Interrupted) => {
                break;
            }

            Err(ReadlineError::Eof) => {
                println!("{}", "EOF".dimmed());
                break;
            }

            Err(err) => {
                println!("{} {:?}", "[ERROR]".red().bold(), err);
                break;
            }
        }
    }

    // Save history
    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }

    println!("{}", "Goodbye!".cyan().bold());
    Ok(())
}

/// Handle a single command
fn handle_command(line: &str, state: &mut ReplState) -> Result<bool> {
    let line = line.trim();

    if line.is_empty() {
        return Ok(true);
    }

    let tokens: Vec<&str> = line.split_whitespace().collect();

    match tokens[0] {
        "help" | "?" => {
            if tokens.len() > 1 {
                help::print_command_help(tokens[1]);
            } else {
                help::print_help();
            }
            return Ok(true);
        }

        "quit" | "exit" => {
            return Ok(false);
        }

        "state" | "show" | "status" => {
            print_state(state);
            return Ok(true);
        }

        "stop" => {
            if tokens.len() > 1 {
                let format = match tokens[1].to_lowercase().as_str() {
                    "wav" => RecordFormat::Wav,
                    "midi" | "mid" => RecordFormat::Midi,
                    "musicxml" | "xml" => RecordFormat::MusicXml,
                    _ => {
                        println!(
                            "{} Unknown format: {}. Use: wav, midi, musicxml",
                            "[ERROR]".red().bold(),
                            tokens[1]
                        );
                        return Ok(true);
                    }
                };
                let _ = state.playback_cmd_tx.push(PlaybackCommand::StopRecording(format));
                println!("{} Recording {:?} stopped", "[OK]".green().bold(), format);
            } else {
                let _ =
                    state.playback_cmd_tx.push(PlaybackCommand::StopRecording(RecordFormat::Wav));
                let _ =
                    state.playback_cmd_tx.push(PlaybackCommand::StopRecording(RecordFormat::Midi));
                let _ = state
                    .playback_cmd_tx
                    .push(PlaybackCommand::StopRecording(RecordFormat::MusicXml));
                println!("{} All recordings stopped", "[OK]".green().bold());
            }
            return Ok(true);
        }

        "seek" => {
            if tokens.len() > 1 {
                if let Ok(bar) = tokens[1].parse::<usize>() {
                    let target_bar = bar.max(1);
                    if let Ok(mut c) = state.composer.lock() {
                        c.seek_writehead(target_bar);
                    }
                    let _ = state.playback_cmd_tx.push(PlaybackCommand::Seek(target_bar));
                    // Pre-generate bars at the new position
                    if let Ok(mut c) = state.composer.lock() {
                        c.generate_bars(8);
                    }
                    println!("{} Seeking to bar {target_bar}", "[OK]".green().bold());
                } else {
                    println!("{} Usage: seek <bar_number>", "[ERROR]".red().bold());
                }
            } else {
                println!("{} Usage: seek <bar_number>", "[ERROR]".red().bold());
            }
            return Ok(true);
        }

        "loop" => {
            if tokens.len() > 2 {
                if let (Ok(start), Ok(end)) =
                    (tokens[1].parse::<usize>(), tokens[2].parse::<usize>())
                {
                    if !harmonium::playback::check_loop_len(start, end) {
                        println!(
                            "{} Loop too long: at most {} bars fit the playback window",
                            "[ERROR]".red().bold(),
                            harmonium::playback::MEASURE_WINDOW_CAP
                        );
                    } else {
                        let _ = state
                            .playback_cmd_tx
                            .push(PlaybackCommand::SetLoop { start_bar: start, end_bar: end });
                        println!("{} Loop set: bars {start}-{end}", "[OK]".green().bold());
                    }
                } else {
                    println!("{} Usage: loop <start_bar> <end_bar>", "[ERROR]".red().bold());
                }
            } else if tokens.len() > 1 && tokens[1] == "off" {
                let _ = state.playback_cmd_tx.push(PlaybackCommand::ClearLoop);
                println!("{} Loop cleared", "[OK]".green().bold());
            } else {
                println!("{} Usage: loop <start> <end> | loop off", "[ERROR]".red().bold());
            }
            return Ok(true);
        }

        // Handle relative emotion adjustments
        "emotion" if tokens.len() > 1 && has_relative_values(&tokens[1..]) => {
            return handle_relative_emotion(state, &tokens[1..]);
        }

        // Style profile commands
        "profile" => {
            return handle_profile_command(state, &tokens[1..]);
        }

        // Commandes CLI trad et horloge (hors EngineCommand).
        "swing" | "tempo" | "beat" | "style" | "feet" | "intensity" => {
            return handle_clock_command(state, &tokens);
        }
        _ => {}
    }

    // Parse command via existing parser (returns EngineCommand)
    match parser::parse_command(line) {
        Ok(cmd) => {
            dispatch_command(state, cmd)?;
        }
        Err(e) => {
            let msg = e.to_string();
            if msg != "help" && msg != "quit" && msg != "stop" {
                println!("{} {}", "[ERROR]".red().bold(), e);
            }
        }
    }

    Ok(true)
}

// ---------------------------------------------------------------------------
// Phase-0 clock commands: swing, tempo, beat
// ---------------------------------------------------------------------------

/// Handle `swing`, `tempo` and `beat` (parsed by `parser::parse_cli_command`).
fn handle_clock_command(state: &mut ReplState, tokens: &[&str]) -> Result<bool> {
    match parser::parse_cli_command(tokens) {
        Ok(CliCommand::Swing { ratio, clamped }) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_swing_ratio(ratio);
                c.invalidate_future();
            }
            state.send_invalidate();
            if clamped {
                println!(
                    "{} Swing hors plage (0.5–0.75), borné à {:.2}",
                    "[WARN]".yellow().bold(),
                    ratio
                );
            }
            print_success_msg(&format!(
                "Swing réglé à {ratio:.2} (prend effet à la prochaine mesure)"
            ));
        }
        Ok(CliCommand::Tempo { bpm, ramp_pulses }) => {
            let _ = state.playback_cmd_tx.push(PlaybackCommand::SetTempo { bpm, ramp_pulses });
            if ramp_pulses == 0 {
                print_success_msg(&format!(
                    "Tempo {bpm:.0} BPM : saut à la prochaine pulse (moteur borne à 40–300)"
                ));
            } else {
                print_success_msg(&format!(
                    "Tempo {bpm:.0} BPM : rampe sur {ramp_pulses} pulses (moteur borne à 40–300)"
                ));
            }
        }
        Ok(CliCommand::ClearTempo) => {
            let _ = state.playback_cmd_tx.push(PlaybackCommand::ClearTempo);
            print_success_msg("Tempo audible : retour au tempo des mesures");
        }
        Ok(CliCommand::Beat) => {
            return run_beat(state);
        }
        Ok(CliCommand::Style(style)) => {
            let (bpm, swing, ts) = if let Ok(mut c) = state.composer.lock() {
                c.set_style(style);
                c.invalidate_future();
                let mp = c.musical_params();
                (mp.bpm, mp.swing_ratio, mp.time_signature)
            } else {
                (120.0, 0.5, harmonium_core::params::TimeSignature::default())
            };
            state.send_invalidate();
            print_success_msg(&format!(
                "Style réglé à {:?} (BPM {:.0}, swing {:.2}, signature {}/{})",
                style, bpm, swing, ts.numerator, ts.denominator
            ));
        }
        Ok(CliCommand::Feet(density)) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_feet_density(density);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!(
                "Densité des pieds réglée à {density} (prend effet à la prochaine mesure)"
            ));
        }
        Ok(CliCommand::Intensity(value)) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_intensity(value);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Intensité {value:.2} : effet à la prochaine mesure"));
        }
        Err(e) => {
            println!("{} {}", "[ERROR]".red().bold(), e);
        }
    }
    Ok(true)
}

/// Live visual metronome driven only by `BeatInfo` (~60 reads/s, one rewritten
/// line). Exit with Enter, Esc, `q` or Ctrl+C; the REPL resumes afterwards.
///
/// Key reading uses crossterm in raw mode (enabled only for the loop, restored
/// by `RawGuard` on every exit path): a blocking `read_line` thread would race
/// rustyline for the next input line, and plain std has no key polling.
fn run_beat(state: &mut ReplState) -> Result<bool> {
    use std::{
        io::Write as _,
        time::{Duration, Instant},
    };

    use crossterm::{
        event::{self, Event, KeyCode, KeyModifiers},
        terminal,
    };

    struct RawGuard;
    impl Drop for RawGuard {
        fn drop(&mut self) {
            let _ = terminal::disable_raw_mode();
        }
    }

    println!("Métronome — Entrée, Échap, q ou Ctrl+C pour sortir :");
    terminal::enable_raw_mode()?;
    let _raw = RawGuard;

    loop {
        let clock = state.clock_reader.read();
        let now = Instant::now();

        if clock.next_pulse_frame == 0 {
            print!("\rEn attente de l'horloge… (Entrée pour sortir)   ");
        } else {
            let info = BeatInfo::from_clock(&clock, now);
            let width = 16usize;
            let filled = ((info.phase * width as f32) as usize).min(width);
            let bar_glyphs = "█".repeat(filled) + &"░".repeat(width - filled);
            let marker = if info.is_downbeat {
                "● TEMPS FORT"
            } else if info.is_backbeat {
                "◐"
            } else {
                "○"
            };
            let countdown = info.next_pulse_at.saturating_duration_since(now).as_millis();
            print!(
                "\rBar {:>3} | pulse {:>2}/{} | [{}] {:>3}% | {} | +{}ms  ",
                info.bar,
                info.pulse,
                info.pulses_per_bar,
                bar_glyphs,
                (info.phase * 100.0) as u32,
                marker,
                countdown
            );
        }
        std::io::stdout().flush().ok();

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                let ctrl_c =
                    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q' | 'Q'))
                    || ctrl_c
                {
                    break;
                }
            }
        }
    }

    drop(_raw);
    println!();
    print_success_msg("Métronome arrêté");
    Ok(true)
}

/// Route an EngineCommand to the appropriate target (composer or playback).
fn dispatch_command(state: &mut ReplState, cmd: EngineCommand) -> Result<()> {
    match cmd {
        // === Emotion handling ===
        EngineCommand::SetEmotionParams { arousal, valence, density, tension } => {
            state.emotion_state.arousal = arousal;
            state.emotion_state.valence = valence;
            state.emotion_state.density = density;
            state.emotion_state.tension = tension;

            if let Ok(mut c) = state.composer.lock() {
                c.use_emotion_mode();
                c.set_emotions(arousal, valence, density, tension);
                let bpm = c.musical_params().bpm;
                let density_mapped = c.musical_params().rhythm_density;
                c.invalidate_future();
                println!(
                    "{} A={:.2} V={:.2} D={:.2} T={:.2} → BPM={:.0} Density={:.2}",
                    "[OK]".green().bold(),
                    arousal,
                    valence,
                    density,
                    tension,
                    bpm,
                    density_mapped
                );
            }
            state.send_invalidate();
        }

        EngineCommand::UseEmotionMode => {
            if let Ok(mut c) = state.composer.lock() {
                c.use_emotion_mode();
            }
            print_success_msg("Switched to Emotion mode");
        }

        EngineCommand::UseDirectMode => {
            if let Ok(mut c) = state.composer.lock() {
                c.use_direct_mode();
            }
            print_success_msg("Switched to Direct mode");
        }

        // === Generation params → composer ===
        EngineCommand::SetBpm(bpm) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_bpm(bpm);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("BPM set to {}", bpm));
        }

        EngineCommand::SetTimeSignature { numerator, denominator } => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_time_signature(numerator, denominator);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Time signature set to {}/{}", numerator, denominator));
        }

        EngineCommand::SetRhythmMode(mode) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_rhythm_mode(mode);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Rhythm mode set to {:?}", mode));
        }

        EngineCommand::SetRhythmSteps(steps) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_rhythm_steps(steps);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Steps set to {}", steps));
        }

        EngineCommand::SetRhythmPulses(pulses) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_rhythm_pulses(pulses);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Pulses set to {}", pulses));
        }

        EngineCommand::SetRhythmRotation(rotation) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_rhythm_rotation(rotation);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Rotation set to {}", rotation));
        }

        EngineCommand::SetRhythmDensity(density) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_rhythm_density(density);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Density set to {:.2}", density));
        }

        EngineCommand::SetRhythmTension(tension) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_rhythm_tension(tension);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Rhythm tension set to {:.2}", tension));
        }

        EngineCommand::SetHarmonyMode(mode) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_harmony_mode(mode);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Harmony mode set to {:?}", mode));
        }

        EngineCommand::SetHarmonyTension(tension) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_harmony_tension(tension);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Harmony tension set to {:.2}", tension));
        }

        EngineCommand::SetHarmonyValence(valence) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_harmony_valence(valence);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Harmony valence set to {:.2}", valence));
        }

        EngineCommand::SetHarmonyStrategy(strategy) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_harmony_strategy(strategy);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Harmony strategy set to {:?}", strategy));
        }

        EngineCommand::SetMelodySmoothness(smoothness) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_melody_smoothness(smoothness);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Smoothness set to {:.2}", smoothness));
        }

        EngineCommand::SetMelodyOctave(octave) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_melody_octave(octave);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Octave set to {}", octave));
        }

        EngineCommand::SetVoicingDensity(density) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_voicing_density(density);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Voicing density set to {:.2}", density));
        }

        EngineCommand::SetVoicingTension(tension) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_voicing_tension(tension);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Voicing tension set to {:.2}", tension));
        }

        EngineCommand::EnableRhythm(enabled) => {
            if let Ok(mut c) = state.composer.lock() {
                c.enable_rhythm(enabled);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Rhythm {}", if enabled { "enabled" } else { "disabled" }));
        }

        EngineCommand::EnableHarmony(enabled) => {
            if let Ok(mut c) = state.composer.lock() {
                c.enable_harmony(enabled);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Harmony {}", if enabled { "enabled" } else { "disabled" }));
        }

        EngineCommand::EnableMelody(enabled) => {
            if let Ok(mut c) = state.composer.lock() {
                c.enable_melody(enabled);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Melody {}", if enabled { "enabled" } else { "disabled" }));
        }

        EngineCommand::EnableVoicing(enabled) => {
            if let Ok(mut c) = state.composer.lock() {
                c.enable_voicing(enabled);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Voicing {}", if enabled { "enabled" } else { "disabled" }));
        }

        EngineCommand::SetFixedKick(fixed) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_fixed_kick(fixed);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Fixed kick {}", if fixed { "ON" } else { "OFF" }));
        }

        EngineCommand::SetAllRhythmParams {
            mode,
            steps,
            pulses,
            rotation,
            density,
            tension,
            secondary_steps,
            secondary_pulses,
            secondary_rotation,
        } => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_all_rhythm_params(
                    mode,
                    steps,
                    pulses,
                    rotation,
                    density,
                    tension,
                    secondary_steps,
                    secondary_pulses,
                    secondary_rotation,
                );
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg("All rhythm params updated");
        }

        EngineCommand::SetRhythmSecondary { steps, pulses, rotation } => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_rhythm_secondary(steps, pulses, rotation);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Secondary rhythm: {steps}s/{pulses}p/r{rotation}"));
        }

        EngineCommand::SetKeyRoot(root) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_key_root(root);
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg(&format!("Key root set to {}", root));
        }

        EngineCommand::ResetBpm => {
            if let Ok(mut c) = state.composer.lock() {
                c.reset_bpm();
                c.invalidate_future();
            }
            state.send_invalidate();
            print_success_msg("BPM override cleared, returning to emotion-mapped BPM");
        }

        EngineCommand::Reset => {
            if let Ok(mut c) = state.composer.lock() {
                c.reset();
            }
            state.send_invalidate();
            print_success_msg("Engine reset to defaults");
        }

        EngineCommand::SetSeed(seed) => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_seed(seed);
            }
            let _ = state.playback_cmd_tx.push(PlaybackCommand::Seek(1));
            state.send_invalidate();
            print_success_msg(&format!("Seed réglé à {seed}"));
        }

        // === Playback commands → audio thread ===
        EngineCommand::SetChannelGain { channel, gain } => {
            let _ = state.playback_cmd_tx.push(PlaybackCommand::SetChannelGain { channel, gain });
            print_success_msg(&format!("Channel {} gain set to {:.2}", channel, gain));
        }

        EngineCommand::SetChannelMute { channel, muted } => {
            let _ = state.playback_cmd_tx.push(PlaybackCommand::SetChannelMute { channel, muted });
            print_success_msg(&format!(
                "Channel {} {}",
                channel,
                if muted { "muted" } else { "unmuted" }
            ));
        }

        EngineCommand::SetChannelRoute { channel, bank_id } => {
            let _ =
                state.playback_cmd_tx.push(PlaybackCommand::SetChannelRoute { channel, bank_id });
            print_success_msg(&format!("Channel {} routed to bank {}", channel, bank_id));
        }

        EngineCommand::SetVelocityBase { channel, velocity } => {
            let _ =
                state.playback_cmd_tx.push(PlaybackCommand::SetVelocityBase { channel, velocity });
            print_success_msg(&format!("Channel {} velocity base set to {}", channel, velocity));
        }

        EngineCommand::SetMasterVolume(volume) => {
            let _ = state.playback_cmd_tx.push(PlaybackCommand::SetMasterVolume(volume));
            print_success_msg(&format!("Master volume set to {:.2}", volume));
        }

        EngineCommand::StartRecording(format) => {
            let _ = state.playback_cmd_tx.push(PlaybackCommand::StartRecording(format));
            print_success_msg(&format!("Recording {:?} started", format));
        }

        EngineCommand::StopRecording(format) => {
            let _ = state.playback_cmd_tx.push(PlaybackCommand::StopRecording(format));
            print_success_msg(&format!("Recording {:?} stopped", format));
        }

        // Ignored / no-op commands
        EngineCommand::GetState => {
            print_state(state);
        }

        _ => {
            println!("{} Unhandled command: {:?}", "[WARN]".yellow().bold(), cmd);
        }
    }

    Ok(())
}

/// Check if tokens contain relative value syntax (+/-)
fn has_relative_values(tokens: &[&str]) -> bool {
    tokens.iter().any(|t| t.contains('+') || t.contains('-'))
}

/// Handle relative emotion adjustments (e.g., "emotion a+0.1 v-0.2")
fn handle_relative_emotion(state: &mut ReplState, tokens: &[&str]) -> Result<bool> {
    for token in tokens {
        if token.len() < 2 {
            continue;
        }

        let param = token.chars().next().unwrap().to_lowercase().to_string();
        let value_part = &token[1..];

        let delta = if value_part.starts_with('+') || value_part.starts_with('-') {
            value_part.parse::<f32>().unwrap_or(0.0)
        } else {
            continue;
        };

        match param.as_str() {
            "a" => {
                state.emotion_state.arousal = (state.emotion_state.arousal + delta).clamp(0.0, 1.0)
            }
            "v" => {
                state.emotion_state.valence = (state.emotion_state.valence + delta).clamp(-1.0, 1.0)
            }
            "d" => {
                state.emotion_state.density = (state.emotion_state.density + delta).clamp(0.0, 1.0)
            }
            "t" => {
                state.emotion_state.tension = (state.emotion_state.tension + delta).clamp(0.0, 1.0)
            }
            _ => {
                println!("{} Unknown emotion parameter: {}", "[WARN]".yellow().bold(), param);
            }
        }
    }

    // Apply updated emotions via the composer
    let a = state.emotion_state.arousal;
    let v = state.emotion_state.valence;
    let d = state.emotion_state.density;
    let t = state.emotion_state.tension;

    if let Ok(mut c) = state.composer.lock() {
        c.use_emotion_mode();
        c.set_emotions(a, v, d, t);
        let bpm = c.musical_params().bpm;
        let density_mapped = c.musical_params().rhythm_density;
        c.invalidate_future();
        println!(
            "{} A={:.2} V={:.2} D={:.2} T={:.2} → BPM={:.0} Density={:.2}",
            "[OK]".green().bold(),
            a,
            v,
            d,
            t,
            bpm,
            density_mapped
        );
    }
    state.send_invalidate();

    Ok(true)
}

fn print_success_msg(msg: &str) {
    println!("{} {}", "[OK]".green().bold(), msg);
}

/// Print welcome message
fn print_welcome() {
    println!();
    println!("{}", "╔════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║                                                        ║".cyan());
    println!("{}", "║     🎵  Harmonium CLI - Interactive Music Engine  🎵   ║".cyan().bold());
    println!("{}", "║                                                        ║".cyan());
    println!("{}", "╚════════════════════════════════════════════════════════╝".cyan());
    println!();
    println!("  Type {} for available commands", "help".green().bold());
    println!("  Type {} or {} to exit", "quit".green(), "exit".green());
    println!();
}

/// Create prompt string with current state
fn create_prompt(state: &ReplState) -> String {
    if let Some(report) = state.get_state() {
        let bpm = report.musical_params.bpm;
        let chord = &report.current_chord;
        let bar = report.current_bar + 1;

        format!(
            "{} {} {} {} ",
            "harmonium".cyan().bold(),
            format!("{}bpm", bpm as u32).yellow(),
            chord.to_string().magenta(),
            format!("[bar:{}]", bar).dimmed(),
        )
    } else {
        format!("{} ", "harmonium".cyan().bold())
    }
}

/// Formate la position dans la forme AABB d'un reel (ex. `"A2 m.5/8"`).
#[must_use]
pub fn format_reel_form_position(role: harmonium_core::trad::BarRole) -> String {
    format!("{:?}{} m.{}/8", role.strain, role.pass, role.bar_in_strain)
}

/// Formate la tonalité et le mode d'un [`harmonium_core::trad::TunePlan`] (ex. `"D Ionian"`).
#[must_use]
pub fn format_reel_tune_key_mode(plan: harmonium_core::trad::TunePlan) -> String {
    format!("{} {:?}", harmonium_core::trad::pitch_class_name(plan.key), plan.mode)
}

/// Print current engine state
fn print_state(state: &mut ReplState) {
    state.poll_reports();

    let composer_snapshot = state.composer.lock().ok().map(|c| {
        let bar = state.get_state().map_or_else(|| c.playhead_bar(), |r| r.current_bar.max(1));
        (
            c.musical_params().style,
            c.musical_params().feet_density,
            c.reel_bar_info(bar),
            c.config.key.clone(),
            c.config.scale.clone(),
        )
    });

    if let Some(report) = state.get_state() {
        let (style, feet_density, reel_info, comp_key, comp_scale) = composer_snapshot.unwrap_or((
            report.musical_params.style,
            report.musical_params.feet_density,
            None,
            report.session_key.to_string(),
            report.session_scale.to_string(),
        ));

        println!();
        println!("{}", "╔══════════════════════════════════════════════════╗".cyan());
        println!("{}", "║              ENGINE STATE                        ║".cyan().bold());
        println!("{}", "╚══════════════════════════════════════════════════╝".cyan());
        println!();

        // Style & Trad info
        println!("{}", "STYLE:".yellow().bold());
        println!("  Style: {}", format!("{style:?}").green());
        println!("  Feet Density: {}", format!("{feet_density}").green());
        if let Some((plan, role)) = reel_info {
            println!("  Tune Key/Mode: {}", format_reel_tune_key_mode(plan).green());
            println!("  Form Position: {}", format_reel_form_position(role).magenta().bold());
        }
        println!();

        // Timing
        println!("{}", "TIMING:".yellow().bold());
        println!("  BPM (audible): {}", format!("{:.1}", report.musical_params.bpm).green());
        println!(
            "  Swing: {}",
            format!("{:.2} (0.5 = droit)", report.musical_params.swing_ratio).green()
        );
        let pulse = report.musical_params.effective_pulse(4);
        println!(
            "  Pulse: {}",
            format!("{} pas × {} par mesure", pulse.steps_per_pulse, pulse.pulses_per_bar).green()
        );
        println!(
            "  Time Signature: {}",
            format!("{}/{}", report.time_signature.numerator, report.time_signature.denominator)
                .green()
        );
        println!("  Bar: {}", format!("{}", report.current_bar.max(1)).green());
        println!("  Beat: {}", format!("{}", report.current_beat.max(1)).green());
        println!("  Step: {}", format!("{}", report.current_step + 1).green());
        println!();

        // Harmony
        println!("{}", "HARMONY:".yellow().bold());
        println!("  Mode: {}", format!("{:?}", report.harmony_mode).green());
        println!("  Current Chord: {}", format!("{}", report.current_chord).magenta().bold());
        println!(
            "  Progression: {}",
            format!("{} ({} chords)", report.progression_name, report.progression_length).green()
        );
        let key_str = if let Some((plan, _)) = reel_info {
            format_reel_tune_key_mode(plan)
        } else if !report.session_key.is_empty() {
            format!("{} {}", report.session_key, report.session_scale)
        } else {
            format!("{comp_key} {comp_scale}")
        };
        println!("  Key: {}", key_str.green());
        println!();

        // Rhythm
        println!("{}", "RHYTHM:".yellow().bold());
        println!("  Mode: {}", format!("{:?}", report.rhythm_mode).green());
        println!(
            "  Primary: {} steps, {} pulses, rotation {}",
            format!("{}", report.primary_steps).green(),
            format!("{}", report.primary_pulses).green(),
            format!("{}", report.primary_rotation).green()
        );
        println!(
            "  Secondary: {} steps, {} pulses, rotation {}",
            format!("{}", report.secondary_steps).green(),
            format!("{}", report.secondary_pulses).green(),
            format!("{}", report.secondary_rotation).green()
        );

        // Print pattern (first 32 steps)
        print!("  Pattern: ");
        for (i, &trigger) in report.primary_pattern.iter().enumerate().take(32) {
            if i > 0 && i % 16 == 0 {
                print!("| ");
            }
            if trigger {
                print!("{}", "█".green());
            } else {
                print!("{}", "·".dimmed());
            }
        }
        println!();
        println!();

        // Modules
        println!("{}", "MODULES:".yellow().bold());
        println!(
            "  Rhythm:  {}",
            if report.musical_params.enable_rhythm { "ON".green() } else { "OFF".red() }
        );
        println!(
            "  Harmony: {}",
            if report.musical_params.enable_harmony { "ON".green() } else { "OFF".red() }
        );
        println!(
            "  Melody:  {}",
            if report.musical_params.enable_melody { "ON".green() } else { "OFF".red() }
        );
        println!(
            "  Voicing: {}",
            if report.musical_params.enable_voicing { "ON".green() } else { "OFF".red() }
        );
        println!();
    } else if let Some((style, feet_density, reel_info, comp_key, comp_scale)) = composer_snapshot {
        println!();
        println!("{}", "STYLE:".yellow().bold());
        println!("  Style: {}", format!("{style:?}").green());
        println!("  Feet Density: {}", format!("{feet_density}").green());
        if let Some((plan, role)) = reel_info {
            println!("  Tune Key/Mode: {}", format_reel_tune_key_mode(plan).green());
            println!("  Form Position: {}", format_reel_form_position(role).magenta().bold());
        } else {
            println!("  Key: {}", format!("{comp_key} {comp_scale}").green());
        }
        println!();
    } else {
        println!("{}", "No state available yet. Engine may still be initializing.".yellow());
    }
}

// ---------------------------------------------------------------------------
// Style Profile Commands
// ---------------------------------------------------------------------------

/// Default directory for style profiles.
fn profiles_dir() -> std::path::PathBuf {
    // Look for tune_output relative to the workspace root
    let candidates = [
        std::path::PathBuf::from("../tune_output"),
        std::path::PathBuf::from("./tune_output"),
        dirs::home_dir().unwrap_or_default().join(".harmonium").join("profiles"),
    ];
    for c in &candidates {
        if c.is_dir() {
            return c.clone();
        }
    }
    candidates[0].clone()
}

fn handle_profile_command(state: &mut ReplState, tokens: &[&str]) -> Result<bool> {
    let subcmd = tokens.first().copied().unwrap_or("help");

    match subcmd {
        "list" | "ls" => {
            let dir = profiles_dir();
            if !dir.is_dir() {
                println!(
                    "{} No profiles directory found at {}",
                    "[WARN]".yellow().bold(),
                    dir.display()
                );
                return Ok(true);
            }
            let mut profiles: Vec<String> = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            profiles.push(stem.to_string());
                        }
                    }
                }
            }
            profiles.sort();
            if profiles.is_empty() {
                println!("No .toml profiles found in {}", dir.display());
            } else {
                println!(
                    "{} Available profiles (in {}):",
                    "[PROFILES]".cyan().bold(),
                    dir.display()
                );
                for p in &profiles {
                    println!("  - {}", p.green());
                }
                println!();
                println!("Usage: {} <name>", "profile load".bold());
            }
        }

        "load" => {
            let name = tokens.get(1).copied().unwrap_or("");
            if name.is_empty() {
                println!("{} Usage: profile load <name>", "[ERROR]".red().bold());
                return Ok(true);
            }

            let dir = profiles_dir();
            let path = dir.join(format!("{}.toml", name));

            if !path.exists() {
                println!("{} Profile not found: {}", "[ERROR]".red().bold(), path.display());
                println!("Run {} to see available profiles", "profile list".bold());
                return Ok(true);
            }

            match load_and_apply_profile(state, &path) {
                Ok(info) => {
                    println!("{} Loaded profile: {}", "[OK]".green().bold(), name.cyan());
                    println!("  BPM: {:.0}, Density: {:.2}, Tension: {:.2}, Valence: {:.2}, Arousal: {:.2}",
                        info.bpm, info.density, info.tension, info.valence, info.arousal);
                }
                Err(e) => {
                    println!("{} Failed to load profile: {}", "[ERROR]".red().bold(), e);
                }
            }
        }

        "clear" => {
            if let Ok(mut c) = state.composer.lock() {
                c.set_tuning(harmonium_core::tuning::TuningParams::default());
                c.invalidate_future();
            }
            state.send_invalidate();
            println!("{} Style profile cleared (back to defaults)", "[OK]".green().bold());
        }

        _ => {
            println!("{}  profile list          — list available profiles", "Usage:".bold());
            println!("         profile load <name>   — load a style profile");
            println!("         profile clear         — revert to default tuning");
        }
    }

    Ok(true)
}

struct ProfileInfo {
    bpm: f32,
    density: f32,
    tension: f32,
    valence: f32,
    arousal: f32,
}

fn load_and_apply_profile(state: &mut ReplState, path: &std::path::Path) -> Result<ProfileInfo> {
    let toml_str = std::fs::read_to_string(path)?;

    // Parse the [render] section for emotion params
    let raw: toml::Value = toml::from_str(&toml_str)?;
    let render = raw.get("render");

    let bpm = render.and_then(|r| r.get("bpm")).and_then(|v| v.as_float()).unwrap_or(120.0) as f32;
    let density =
        render.and_then(|r| r.get("density")).and_then(|v| v.as_float()).unwrap_or(0.5) as f32;
    let tension =
        render.and_then(|r| r.get("tension")).and_then(|v| v.as_float()).unwrap_or(0.4) as f32;
    let valence =
        render.and_then(|r| r.get("valence")).and_then(|v| v.as_float()).unwrap_or(0.3) as f32;
    let arousal =
        render.and_then(|r| r.get("arousal")).and_then(|v| v.as_float()).unwrap_or(0.5) as f32;

    // Parse TuningParams (all sections except [render])
    // Re-parse with [render] stripped — or just parse the full thing and let serde ignore unknown
    let tuning: harmonium_core::tuning::TuningParams = {
        // Remove [render] section and parse as TuningParams
        let mut filtered = raw.clone();
        if let Some(table) = filtered.as_table_mut() {
            table.remove("render");
        }
        let filtered_str = toml::to_string(&filtered)?;
        toml::from_str(&filtered_str)?
    };

    // Apply tuning
    if let Ok(mut c) = state.composer.lock() {
        c.set_tuning(tuning);
        c.set_bpm(bpm);
        c.set_rhythm_mode(harmonium_core::sequencer::RhythmMode::ClassicGroove);
        c.set_emotions(arousal, valence, density, tension);
        c.sync_generator();
        c.invalidate_future();
    }
    state.send_invalidate();

    // Update cached emotion state
    state.emotion_state.arousal = arousal;
    state.emotion_state.valence = valence;
    state.emotion_state.density = density;
    state.emotion_state.tension = tension;

    Ok(ProfileInfo { bpm, density, tension, valence, arousal })
}

#[cfg(test)]
mod tests {
    use harmonium_core::trad::{Mode, TunePlan};

    use super::*;

    #[test]
    fn formats_reel_form_position_and_tune_key_mode() {
        let plan = TunePlan::reel(2, Mode::Ionian);
        assert_eq!(format_reel_tune_key_mode(plan), "D Ionian");
        assert_eq!(format_reel_form_position(plan.bar_role_1based(1)), "A1 m.1/8");
        assert_eq!(format_reel_form_position(plan.bar_role_1based(13)), "A2 m.5/8");
        assert_eq!(format_reel_form_position(plan.bar_role_1based(17)), "B1 m.1/8");
        assert_eq!(format_reel_form_position(plan.bar_role_1based(32)), "B2 m.8/8");
    }
}

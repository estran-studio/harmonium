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
    fn start_recording(&mut self, format: RecordFormat) -> Result<()> {
        self.composer
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .start_recording(format, &mut self.playback_cmd_tx)
            .map_err(anyhow::Error::msg)
    }

    fn stop_recording(&mut self, format: RecordFormat) -> Result<()> {
        self.composer
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .stop_recording(format, &mut self.playback_cmd_tx, &self.finished_recordings)
            .map_err(anyhow::Error::msg)
    }
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
                state.stop_recording(format)?;
                println!("{} Recording {:?} stopped", "[OK]".green().bold(), format);
            } else {
                for format in [RecordFormat::Wav, RecordFormat::Midi, RecordFormat::MusicXml] {
                    state.stop_recording(format)?;
                }
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
            let (label, bpm, swing, ts) = {
                let mut c = state.composer.lock().map_err(|e| anyhow::anyhow!(e.to_string()))?;
                c.set_profile(*style).map_err(|e| anyhow::anyhow!(e))?;
                c.invalidate_future();
                let mp = c.musical_params();
                let label = mp.profile_id.to_string();
                (label, mp.bpm, mp.swing_ratio, mp.time_signature)
            };
            state.send_invalidate();
            print_success_msg(&format!(
                "Style réglé à {} (BPM {:.0}, swing {:.2}, signature {}/{})",
                label, bpm, swing, ts.numerator, ts.denominator
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

        EngineCommand::SetStyleProfile(profile) => {
            let mut c = state.composer.lock().map_err(|e| anyhow::anyhow!(e.to_string()))?;
            c.set_profile(*profile).map_err(|e| anyhow::anyhow!(e))?;
            drop(c);
            state.send_invalidate();
        }
        EngineCommand::ClearStyleProfile => {
            let mut c = state.composer.lock().map_err(|e| anyhow::anyhow!(e.to_string()))?;
            c.set_profile(harmonium_core::StyleProfile::default())
                .map_err(|e| anyhow::anyhow!(e))?;
            drop(c);
            state.send_invalidate();
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
            state.start_recording(format)?;
            print_success_msg(&format!("Recording {:?} started", format));
        }

        EngineCommand::StopRecording(format) => {
            state.stop_recording(format)?;
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

/// Seules les formes préfixées a+/v-/d+/t- sont relatives ; une valence
/// numérique négative appartient à la commande absolue à quatre axes.
fn has_relative_values(tokens: &[&str]) -> bool {
    tokens.iter().any(|t| matches!(t.as_bytes(), [b'a' | b'v' | b'd' | b't', b'+' | b'-', ..]))
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

/// Position dans la forme décrite par le profil.
#[must_use]
pub fn format_profile_form_position(
    role: harmonium_core::trad::BarRole,
    plan: &harmonium_core::trad::TunePlan,
) -> String {
    let part = &plan.profile.form.parts[role.strain.0];
    format!("{}{} m.{}/{}", part.name, role.pass, role.bar_in_strain, part.roles.len())
}

/// Formate la tonalité et le mode d'un [`harmonium_core::trad::TunePlan`] (ex. `"D Ionian"`).
#[must_use]
pub fn format_profile_tune_key_mode(plan: &harmonium_core::trad::TunePlan) -> String {
    format!("{} {:?}", harmonium_core::trad::pitch_class_name(plan.key), plan.mode)
}

/// Print current engine state
fn print_state(state: &mut ReplState) {
    state.poll_reports();

    let composer_snapshot = state.composer.lock().ok().map(|c| {
        let bar = state.get_state().map_or_else(|| c.playhead_bar(), |r| r.current_bar.max(1));
        (
            c.musical_params().profile_id.clone(),
            c.musical_params().feet_density,
            c.profile_bar_info(bar),
            c.config.key.clone(),
            c.config.scale.clone(),
        )
    });

    if let Some(report) = state.get_state() {
        let (style, feet_density, reel_info, comp_key, comp_scale) = composer_snapshot.unwrap_or((
            report.musical_params.profile_id.clone(),
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
        if let Some((plan, role)) = &reel_info {
            println!("  Tune Key/Mode: {}", format_profile_tune_key_mode(plan).green());
            println!(
                "  Form Position: {}",
                format_profile_form_position(*role, plan).magenta().bold()
            );
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
        println!("  Current Chord: {}", format!("{}", report.current_chord).magenta().bold());
        println!(
            "  Progression: {}",
            format!("{} ({} chords)", report.progression_name, report.progression_length).green()
        );
        let key_str = if let Some((plan, _)) = reel_info {
            format_profile_tune_key_mode(&plan)
        } else if !report.session_key.is_empty() {
            format!("{} {}", report.session_key, report.session_scale)
        } else {
            format!("{comp_key} {comp_scale}")
        };
        println!("  Key: {}", key_str.green());
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
        println!();
    } else if let Some((style, feet_density, reel_info, comp_key, comp_scale)) = composer_snapshot {
        println!();
        println!("{}", "STYLE:".yellow().bold());
        println!("  Style: {}", format!("{style:?}").green());
        println!("  Feet Density: {}", format!("{feet_density}").green());
        if let Some((plan, role)) = reel_info {
            println!("  Tune Key/Mode: {}", format_profile_tune_key_mode(&plan).green());
            println!(
                "  Form Position: {}",
                format_profile_form_position(role, &plan).magenta().bold()
            );
        } else {
            println!("  Key: {}", format!("{comp_key} {comp_scale}").green());
        }
        println!();
    } else {
        println!("{}", "No state available yet. Engine may still be initializing.".yellow());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_absolute_valence_reaches_the_trad_mapper() -> Result<()> {
        let (composer, _playback, playback_cmd_tx, report_rx, clock_reader, finished_recordings) =
            harmonium::audio::create_offline_engine(None, None, 44100.0)
                .map_err(|e| anyhow::anyhow!(e))?;
        let mut state = ReplState {
            composer: Arc::new(Mutex::new(composer)),
            playback_cmd_tx,
            report_rx,
            clock_reader,
            finished_recordings,
            cached_state: None,
            emotion_state: EmotionState::default(),
        };
        handle_command("style reel", &mut state)?;
        handle_command("emotion 0.8 -0.5 0.9 0.7", &mut state)?;
        {
            let composer = state.composer.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
            let params = composer.musical_params();
            assert_eq!(params.harmony_valence, -0.5);
            assert_eq!(params.rhythm_density, 0.9);
            assert_eq!(params.harmony_tension, 0.7);
            assert!(params.intensity > 0.8);
        }
        handle_command("emotion v+0.1", &mut state)?;
        let composer = state.composer.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        assert!((composer.musical_params().harmony_valence + 0.4).abs() < 0.0001);
        Ok(())
    }
}

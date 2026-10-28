//! REPL (Read-Eval-Print Loop) for Harmonium CLI
//!
//! Text in, display out: every engine command is parsed into an
//! `EngineCommand` and executed by the live host (`LiveHost::apply`), the same
//! executor the OSC listener (`--osc`) and the studio use. The host composes
//! ahead through `LiveHost::pump`, called from a generation thread.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
    time::Duration,
};

use anyhow::{anyhow, Result};
use colored::Colorize;
use harmonium::{osc, BeatInfo, ClockReader, LiveHost};
use harmonium_core::{events::RecordFormat, EngineCommand};
use rustyline::{error::ReadlineError, Editor};

use crate::{
    completer::HarmoniumCompleter,
    help, parser,
    parser::{SWING_MAX, SWING_MIN},
};

/// Interval between two `LiveHost::pump` calls of the generation thread.
const PUMP_INTERVAL: Duration = Duration::from_millis(50);

/// REPL state: the shared host and the beat clock reader (taken from the host
/// so the `beat` view reads it without locking).
struct ReplState {
    host: Arc<Mutex<LiveHost>>,
    clock_reader: ClockReader,
}

impl ReplState {
    fn new(host: Arc<Mutex<LiveHost>>) -> Result<Self> {
        let clock_reader = lock(&host)?
            .take_clock_reader()
            .ok_or_else(|| anyhow!("horloge de pulsation déjà prise"))?;
        Ok(Self { host, clock_reader })
    }

    fn host(&self) -> Result<MutexGuard<'_, LiveHost>> {
        lock(&self.host)
    }
}

fn lock(host: &Mutex<LiveHost>) -> Result<MutexGuard<'_, LiveHost>> {
    host.lock().map_err(|e| anyhow!(e.to_string()))
}

/// Run the interactive REPL on `host`: starts the generation thread and, with
/// `osc`, the OSC listener (`/harmonium/params` on `osc::DEFAULT_ADDR`).
pub fn run(host: LiveHost, osc: bool) -> Result<()> {
    let host = Arc::new(Mutex::new(host));
    let mut state = ReplState::new(host.clone())?;

    // Generation thread: keeps the playback window fed.
    let pump_host = host.clone();
    let pump_stop = Arc::new(AtomicBool::new(false));
    let pump_flag = pump_stop.clone();
    let pump_thread = std::thread::spawn(move || {
        while !pump_flag.load(Ordering::Relaxed) {
            if let Ok(mut host) = pump_host.lock() {
                host.pump();
            }
            std::thread::sleep(PUMP_INTERVAL);
        }
    });

    // OSC: the same executor, driven from another thread.
    let _osc_listener = if osc {
        let osc_host = host.clone();
        let listener = osc::OscListener::spawn(osc::DEFAULT_ADDR, move |command| {
            if let Ok(mut host) = osc_host.lock() {
                let _ = host.apply(command);
            }
        })
        .map_err(|e| anyhow!(e))?;
        println!("OSC listening on {} ({})", listener.local_addr(), osc::PARAMS_ADDRESS);
        Some(listener)
    } else {
        None
    };

    let result = read_eval_loop(&mut state);

    pump_stop.store(true, Ordering::Relaxed);
    let _ = pump_thread.join();
    result
}

fn read_eval_loop(state: &mut ReplState) -> Result<()> {
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
        // Poll for finished recordings and save them
        let recordings = state.host()?.take_recordings();
        for (format, data) in recordings {
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
                    println!("\n{} Failed to write {}: {}", "[ERROR]".red().bold(), filename, e);
                }
            }
        }

        // Create prompt with current state
        let prompt = create_prompt(state)?;

        // Read line
        match rl.readline(&prompt) {
            Ok(line) => {
                // Add to history
                let _ = rl.add_history_entry(line.trim());

                // Handle command
                match handle_command(&line, state) {
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

    // REPL-only words: display and views, no engine command behind them.
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
            print_state(&mut *state.host()?);
            return Ok(true);
        }

        "beat" => {
            if tokens.len() != 1 {
                println!("{} Usage: beat", "[ERROR]".red().bold());
                return Ok(true);
            }
            return run_beat(state);
        }

        // Bare `stop` ends every recording; `stop <format>` is an engine command.
        "stop" if tokens.len() == 1 => {
            for format in [RecordFormat::Wav, RecordFormat::Midi, RecordFormat::MusicXml] {
                state
                    .host()?
                    .apply(EngineCommand::StopRecording(format))
                    .map_err(|e| anyhow!(e))?;
            }
            print_success_msg("All recordings stopped");
            return Ok(true);
        }

        // Relative emotion adjustments (e.g. `emotion a+0.1 v-0.2`)
        "emotion" if tokens.len() > 1 && has_relative_values(&tokens[1..]) => {
            return handle_relative_emotion(state, &tokens[1..]);
        }
        _ => {}
    }

    match parser::parse_command(line) {
        Ok(command) => execute(state, command)?,
        Err(e) => println!("{} {}", "[ERROR]".red().bold(), e),
    }

    Ok(true)
}

/// Execute `command` on the host, then tell the user what happened.
fn execute(state: &mut ReplState, command: EngineCommand) -> Result<()> {
    let mut host = state.host()?;
    host.apply(command.clone()).map_err(|e| anyhow!(e))?;
    report(&command, &host);
    Ok(())
}

/// Confirmation line of an executed command, read back from the host where
/// the host decided (bounds, derived tempo).
fn report(command: &EngineCommand, host: &LiveHost) {
    let params = host.musical_params();
    match command {
        EngineCommand::SetEmotionParams { arousal, valence, density, tension } => {
            print_emotions(*arousal, *valence, *density, *tension, host);
        }
        EngineCommand::UseEmotionMode => print_success_msg("Switched to Emotion mode"),
        EngineCommand::UseDirectMode => print_success_msg("Switched to Direct mode"),
        EngineCommand::SetBpm(bpm) => print_success_msg(&format!("BPM set to {bpm}")),
        EngineCommand::ResetBpm => {
            print_success_msg("BPM override cleared, returning to emotion-mapped BPM");
        }
        EngineCommand::SetTimeSignature { numerator, denominator } => {
            print_success_msg(&format!("Time signature set to {numerator}/{denominator}"));
        }
        EngineCommand::EnableRhythm(enabled) => {
            print_success_msg(&format!("Rhythm {}", if *enabled { "enabled" } else { "disabled" }))
        }
        EngineCommand::EnableHarmony(enabled) => {
            print_success_msg(&format!("Harmony {}", if *enabled { "enabled" } else { "disabled" }))
        }
        EngineCommand::EnableMelody(enabled) => {
            print_success_msg(&format!("Melody {}", if *enabled { "enabled" } else { "disabled" }))
        }
        EngineCommand::Reset => print_success_msg("Engine reset to defaults"),
        EngineCommand::SetSeed(seed) => print_success_msg(&format!("Seed réglé à {seed}")),
        EngineCommand::NewMelody => {
            print_success_msg(&format!("Nouvelle graine : {}", host.session_seed()));
        }
        EngineCommand::SetWriteheadLookahead(bars) => print_success_msg(&format!(
            "Anticipation réglée à {} mesures",
            (*bars).max(MIN_LOOKAHEAD_BARS)
        )),
        EngineCommand::SetStyleProfile(_) => {
            let ts = params.time_signature;
            print_success_msg(&format!(
                "Style réglé à {} (BPM {:.0}, swing {:.2}, signature {}/{})",
                params.profile_id, params.bpm, params.swing_ratio, ts.numerator, ts.denominator
            ));
            for missing in host.missing_resources() {
                println!("{} {missing}", "[WARN]".yellow().bold());
            }
        }
        EngineCommand::SetSwing(ratio) => {
            if !(SWING_MIN..=SWING_MAX).contains(ratio) {
                println!(
                    "{} Swing hors plage (0.5–0.75), borné à {:.2}",
                    "[WARN]".yellow().bold(),
                    params.swing_ratio
                );
            }
            print_success_msg(&format!(
                "Swing réglé à {:.2} (prend effet à la prochaine mesure)",
                params.swing_ratio
            ));
        }
        EngineCommand::SetFeetDensity(density) => print_success_msg(&format!(
            "Densité des pieds réglée à {density} (prend effet à la prochaine mesure)"
        )),
        EngineCommand::SetIntensity(value) => {
            print_success_msg(&format!("Intensité {value:.2} : effet à la prochaine mesure"));
        }
        EngineCommand::SetAudibleTempo { bpm, ramp_pulses: 0 } => print_success_msg(&format!(
            "Tempo {bpm:.0} BPM : saut à la prochaine pulse (moteur borne à 40–300)"
        )),
        EngineCommand::SetAudibleTempo { bpm, ramp_pulses } => print_success_msg(&format!(
            "Tempo {bpm:.0} BPM : rampe sur {ramp_pulses} pulses (moteur borne à 40–300)"
        )),
        EngineCommand::ClearAudibleTempo => {
            print_success_msg("Tempo audible : retour au tempo des mesures");
        }
        EngineCommand::SetMasterVolume(volume) => {
            print_success_msg(&format!("Master volume set to {:.2}", volume.clamp(0.0, 1.0)));
        }
        EngineCommand::SetChannelGain { channel, gain } => {
            print_success_msg(&format!("Channel {channel} gain set to {gain:.2}"));
        }
        EngineCommand::SetChannelMute { channel, muted } => print_success_msg(&format!(
            "Channel {channel} {}",
            if *muted { "muted" } else { "unmuted" }
        )),
        EngineCommand::SetChannelRoute { channel, bank_id } => {
            print_success_msg(&format!("Channel {channel} routed to bank {bank_id}"));
        }
        EngineCommand::SetOutputMute(muted) => {
            print_success_msg(&format!("Output {}", if *muted { "muted" } else { "unmuted" }))
        }
        EngineCommand::StartRecording(format) => {
            print_success_msg(&format!("Recording {format:?} started"));
        }
        EngineCommand::StopRecording(format) => {
            print_success_msg(&format!("Recording {format:?} stopped"));
        }
        EngineCommand::Seek(bar) => {
            print_success_msg(&format!("Seeking to bar {}", (*bar).max(1)));
        }
        EngineCommand::SeekPlayhead(bar) => print_success_msg(&format!(
            "Playhead moved to bar {} (nothing recomposed)",
            (*bar).max(1)
        )),
        EngineCommand::SetLoop { start_bar, end_bar } => {
            print_success_msg(&format!("Loop set: bars {start_bar}-{end_bar}"));
        }
        EngineCommand::ClearLoop => print_success_msg("Loop cleared"),
    }
}

/// Smallest writehead lookahead the composer accepts (it raises smaller values).
const MIN_LOOKAHEAD_BARS: usize = 4;

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

/// Seules les formes préfixées a+/v-/d+/t- sont relatives ; une valence
/// numérique négative appartient à la commande absolue à quatre axes.
fn has_relative_values(tokens: &[&str]) -> bool {
    tokens.iter().any(|t| matches!(t.as_bytes(), [b'a' | b'v' | b'd' | b't', b'+' | b'-', ..]))
}

/// Handle relative emotion adjustments (e.g., "emotion a+0.1 v-0.2"), from
/// the emotions now in effect on the host.
fn handle_relative_emotion(state: &mut ReplState, tokens: &[&str]) -> Result<bool> {
    let mut host = state.host()?;
    let [mut a, mut v, mut d, mut t] = host.emotions();

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
            "a" => a = (a + delta).clamp(0.0, 1.0),
            "v" => v = (v + delta).clamp(-1.0, 1.0),
            "d" => d = (d + delta).clamp(0.0, 1.0),
            "t" => t = (t + delta).clamp(0.0, 1.0),
            _ => {
                println!("{} Unknown emotion parameter: {}", "[WARN]".yellow().bold(), param);
            }
        }
    }

    host.apply(EngineCommand::SetEmotionParams { arousal: a, valence: v, density: d, tension: t })
        .map_err(|e| anyhow!(e))?;
    print_emotions(a, v, d, t, &host);

    Ok(true)
}

/// Confirmation of an emotion change, with the tempo and density it maps to.
fn print_emotions(a: f32, v: f32, d: f32, t: f32, host: &LiveHost) {
    let params = host.musical_params();
    println!(
        "{} A={:.2} V={:.2} D={:.2} T={:.2} → BPM={:.0} Density={:.2}",
        "[OK]".green().bold(),
        a,
        v,
        d,
        t,
        params.bpm,
        params.rhythm_density
    );
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
fn create_prompt(state: &ReplState) -> Result<String> {
    let mut host = state.host()?;
    Ok(if let Some(report) = host.poll_state() {
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
    })
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
fn print_state(host: &mut LiveHost) {
    let reported_bar = host.poll_state().map(|r| r.current_bar.max(1));
    let bar = reported_bar.unwrap_or_else(|| host.playhead_bar());
    let composer_snapshot = (
        host.musical_params().profile_id.clone(),
        host.musical_params().feet_density,
        host.profile_bar_info(bar),
        host.session_config().key.clone(),
        host.session_config().scale.clone(),
    );

    if let Some(report) = host.poll_state() {
        let (style, feet_density, reel_info, comp_key, comp_scale) = composer_snapshot;

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
    } else {
        let (style, feet_density, reel_info, comp_key, comp_scale) = composer_snapshot;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A REPL over a host without audio device (the engine is never run).
    fn repl() -> Result<ReplState> {
        let (host, _playback) =
            LiveHost::detached(None, None, &[], 44100.0).map_err(|e| anyhow!(e))?;
        ReplState::new(Arc::new(Mutex::new(host)))
    }

    #[test]
    fn negative_absolute_valence_reaches_the_trad_mapper() -> Result<()> {
        let mut state = repl()?;
        handle_command("style reel", &mut state)?;
        handle_command("emotion 0.8 -0.5 0.9 0.7", &mut state)?;
        {
            let host = state.host()?;
            let params = host.musical_params();
            assert_eq!(params.harmony_valence, -0.5);
            assert_eq!(params.rhythm_density, 0.9);
            assert_eq!(params.harmony_tension, 0.7);
            assert!(params.intensity > 0.8);
        }
        handle_command("emotion v+0.1", &mut state)?;
        let host = state.host()?;
        assert!((host.musical_params().harmony_valence + 0.4).abs() < 0.0001);
        Ok(())
    }

    #[test]
    fn relative_emotion_starts_from_the_emotions_in_effect() -> Result<()> {
        let mut state = repl()?;
        // An absolute change made elsewhere (OSC, studio) is the base.
        state
            .host()?
            .apply(EngineCommand::SetEmotionParams {
                arousal: 0.2,
                valence: -0.3,
                density: 0.6,
                tension: 0.1,
            })
            .map_err(|e| anyhow!(e))?;
        handle_command("emotion a+0.1", &mut state)?;
        let emotions = state.host()?.emotions();
        for (got, want) in emotions.iter().zip([0.3, -0.3, 0.6, 0.1]) {
            assert!((got - want).abs() < 0.0001, "{emotions:?}");
        }
        Ok(())
    }

    #[test]
    fn module_commands_toggle_the_module() -> Result<()> {
        let mut state = repl()?;
        handle_command("set rhythm off", &mut state)?;
        handle_command("set melody off", &mut state)?;
        {
            let host = state.host()?;
            assert!(!host.musical_params().enable_rhythm);
            assert!(!host.musical_params().enable_melody);
            assert!(host.musical_params().enable_harmony);
        }
        handle_command("set rhythm on", &mut state)?;
        assert!(state.host()?.musical_params().enable_rhythm);
        Ok(())
    }

    #[test]
    fn swing_feet_and_intensity_reach_the_composer() -> Result<()> {
        let mut state = repl()?;
        handle_command("swing 0.9", &mut state)?;
        handle_command("feet 2", &mut state)?;
        handle_command("intensity 0.8", &mut state)?;
        let host = state.host()?;
        assert_eq!(host.musical_params().swing_ratio, SWING_MAX);
        assert_eq!(host.musical_params().feet_density, 2);
        assert!((host.musical_params().intensity - 0.8).abs() < 0.0001);
        Ok(())
    }

    #[test]
    fn seed_commands_set_then_renew_the_session_seed() -> Result<()> {
        let mut state = repl()?;
        handle_command("seed 7", &mut state)?;
        assert_eq!(state.host()?.session_seed(), 7);
        handle_command("seed new", &mut state)?;
        assert_ne!(state.host()?.session_seed(), 7);
        Ok(())
    }

    #[test]
    fn a_loop_longer_than_the_window_is_refused() -> Result<()> {
        let mut state = repl()?;
        assert!(handle_command("loop 1 40", &mut state).is_err());
        assert!(handle_command("loop 4 8", &mut state).is_ok());
        Ok(())
    }
}

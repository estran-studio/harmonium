//! Harmonium CLI - Interactive command-line interface for the Harmonium music engine
//!
//! Uses the decoupled MusicComposer + PlaybackEngine architecture.
//! Use `--export` mode for headless batch export (no REPL, runs as fast as possible).

mod completer;
mod export;
mod help;
mod parser;
mod repl;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use anyhow::Result;
use clap::Parser;
use harmonium::audio;

#[derive(Parser)]
#[command(name = "harmonium-cli")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Audio backend to use (fundsp or odin2)
    #[arg(short, long, default_value = "fundsp")]
    backend: String,

    /// Path to SoundFont file (.sf2) for audio synthesis
    #[arg(short, long)]
    soundfont: Option<String>,

    /// Export mode: skip REPL, run engine offline for --duration seconds and save recordings
    #[arg(long)]
    export: bool,

    /// Duration in seconds for export mode (required with --export)
    #[arg(short, long)]
    duration: Option<u64>,

    /// Record WAV output to file
    #[arg(long, value_name = "PATH")]
    record_wav: Option<String>,

    /// Record MIDI output to file
    #[arg(long, value_name = "PATH")]
    record_midi: Option<String>,

    /// Record MusicXML output to file
    #[arg(long, value_name = "PATH")]
    record_musicxml: Option<String>,

    /// Record engine state snapshots to JSON file (ground truth for testing)
    #[arg(long, value_name = "PATH")]
    record_truth: Option<String>,
}

fn main() -> Result<()> {
    // Disable engine logs to avoid breaking REPL input
    std::env::set_var("HARMONIUM_CLI", "1");

    let args = Args::parse();

    // Validate export mode arguments
    if args.export && args.duration.is_none() {
        eprintln!("Error: --export requires --duration <seconds>");
        std::process::exit(1);
    }

    // Parse backend type
    let backend_type = match args.backend.as_str() {
        "fundsp" => audio::AudioBackendType::FundSP,
        #[cfg(feature = "odin2")]
        "odin2" => audio::AudioBackendType::Odin2,
        _ => {
            eprintln!("Unknown backend: {}. Using fundsp.", args.backend);
            audio::AudioBackendType::FundSP
        }
    };

    // Load SoundFont if provided
    let sf2_bytes = if let Some(sf2_path) = args.soundfont {
        println!("Loading SoundFont: {}", sf2_path);
        match std::fs::read(&sf2_path) {
            Ok(bytes) => {
                println!("Loaded SoundFont ({} bytes)", bytes.len());
                Some(bytes)
            }
            Err(e) => {
                eprintln!("Failed to load SoundFont: {}", e);
                return Err(e.into());
            }
        }
    } else {
        None
    };

    if args.export {
        // Export mode: offline rendering, no audio device
        println!("Initializing offline engine...");
        export::run(
            sf2_bytes.as_deref(),
            backend_type,
            args.duration.unwrap(),
            args.record_wav,
            args.record_midi,
            args.record_musicxml,
            args.record_truth,
        )?;
    } else {
        // Interactive REPL mode: needs audio device
        println!("Initializing Harmonium engine...");

        let (
            _stream,
            config,
            composer_mutex,
            playback_cmd_tx,
            _live_midi_tx,
            report_rx,
            _font_queue,
            finished_recordings,
        ) = audio::create_timeline_stream(sf2_bytes.as_deref(), backend_type)
            .map_err(|e| anyhow::anyhow!(e))?;

        // Wrap composer in Arc for sharing with the generation thread
        let composer = Arc::new(composer_mutex);

        // Spawn a generation thread to continuously feed the ring buffer
        let gen_composer = composer.clone();
        let gen_shutdown = Arc::new(AtomicBool::new(false));
        let gen_shutdown_flag = gen_shutdown.clone();
        std::thread::spawn(move || loop {
            if gen_shutdown_flag.load(Ordering::Relaxed) {
                break;
            }
            if let Ok(mut c) = gen_composer.lock() {
                c.generate_ahead();
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        });

        println!("Engine initialized");
        println!("  Sample rate: 44100 Hz");
        println!("  Key: {} {}", config.key, config.scale);
        println!("  BPM: {:.1}", config.bpm);
        println!();

        repl::run(composer, playback_cmd_tx, report_rx, finished_recordings)?;

        // Signal generation thread to stop
        gen_shutdown.store(true, Ordering::Relaxed);
    }

    Ok(())
}

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
    /// Path to a SoundFont file (.sf2). Defaults to `assets/GeneralUser-GS.sf2`
    /// when present, else `assets/test.sf2` (with warning), else runs silent.
    #[arg(short, long)]
    soundfont: Option<String>,

    /// Export mode: skip REPL, run engine offline and save recordings
    #[arg(long)]
    export: bool,

    /// Duration in seconds for export mode (required with --export unless --bars is set)
    #[arg(short, long)]
    duration: Option<u64>,

    /// Number of measures to render in export mode (starting from measure 1)
    #[arg(long)]
    bars: Option<usize>,

    /// Musical style to use (`legacy` or `reel`)
    #[arg(long)]
    style: Option<String>,

    /// Deterministic session seed
    #[arg(long)]
    seed: Option<u64>,

    /// Podorythmie density (`0`, `1` or `2`)
    #[arg(long)]
    feet: Option<u8>,
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

    let parsed_style = match args.style.as_deref() {
        Some(s) => Some(parser::parse_style_value(s)?),
        None => None,
    };
    if args.feet.is_some_and(|f| f > 2) {
        anyhow::bail!("--feet must be 0, 1 or 2");
    }

    // Validate export mode arguments
    if args.export && args.duration.is_none() && args.bars.is_none() {
        eprintln!("Error: --export requires --duration <seconds> or --bars <count>");
        std::process::exit(1);
    }

    let sf2_bytes = harmonium::load_soundfont(args.soundfont.as_deref())?;

    if args.export {
        // Export mode: offline rendering, no audio device
        println!("Initializing offline engine...");
        let record_wav = if args.record_wav.is_none()
            && args.record_midi.is_none()
            && args.record_musicxml.is_none()
            && args.record_truth.is_none()
        {
            Some("output.wav".to_string())
        } else {
            args.record_wav
        };
        export::run(
            sf2_bytes.as_deref(),
            args.duration,
            args.bars,
            parsed_style,
            args.seed,
            args.feet,
            record_wav,
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
            _transport_position,
            clock_reader,
            finished_recordings,
        ) = audio::create_timeline_stream(sf2_bytes.as_deref()).map_err(|e| anyhow::anyhow!(e))?;

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

        repl::run(composer, playback_cmd_tx, report_rx, clock_reader, finished_recordings)?;
        // Signal generation thread to stop
        gen_shutdown.store(true, Ordering::Relaxed);
    }

    Ok(())
}

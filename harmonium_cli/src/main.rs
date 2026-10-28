//! Harmonium CLI - Interactive command-line interface for the Harmonium music engine
//!
//! The REPL drives a `LiveHost` (decoupled MusicComposer + PlaybackEngine).
//! Use `--export` mode for headless batch export (no REPL, runs as fast as possible).

mod automation;
mod check_cells;
mod completer;
mod export;
mod help;
mod parser;
mod repl;

use anyhow::Result;
use clap::Parser;
use harmonium::LiveHost;
use harmonium_core::EngineCommand;

#[derive(Parser)]
#[command(name = "harmonium-cli")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to a SoundFont file (.sf2). Defaults to `assets/GeneralUser-GS.sf2`
    /// when present, else `assets/test.sf2` (with warning), else runs silent.
    #[arg(short, long)]
    soundfont: Option<String>,

    /// Validate a cell bank TOML file (`CellBank::from_toml`), print readable
    /// errors and exit 0 (valid) or 1 (invalid). Runs without starting the engine.
    #[arg(long, value_name = "FILE")]
    check_cells: Option<String>,
    /// Path to the feet SoundFont (.sf2). Defaults to `assets/harmonium_trad.sf2`
    /// when present (else the feet fall back to the GM kit on channel 9).
    #[arg(long)]
    feet_soundfont: Option<String>,
    /// Dossier de presets Odin (`.odin` fournis par le jeu) : chaque piste
    /// qui déclare le preset en données joue Odin, les autres SoundFont.
    /// Sans effet sans la compilation `--features odin`.
    #[cfg(feature = "odin")]
    #[arg(long, value_name = "DOSSIER")]
    odin_presets: Option<std::path::PathBuf>,

    /// Écoute OSC dans le REPL : `/harmonium/params` (arousal valence density
    /// tension, quatre flottants) sur UDP 127.0.0.1:8080 pilote les émotions.
    #[arg(long, conflicts_with = "export")]
    osc: bool,

    /// Export mode: skip REPL, run engine offline and save recordings
    #[arg(long)]
    export: bool,

    /// Duration in seconds for export mode (required with --export unless --bars is set)
    #[arg(short, long)]
    duration: Option<u64>,

    /// Number of measures to render in export mode (starting from measure 1)
    #[arg(long)]
    bars: Option<usize>,

    /// Render only this logical track, without changing generated notes (export only).
    #[arg(long, requires = "export", value_parser = clap::builder::PossibleValuesParser::new(harmonium_core::timeline::TrackId::ALL.map(|track| track.name())))]
    solo_track: Option<String>,

    /// Courbes TOML par mesure (export seulement).
    #[arg(long, value_name = "FILE", requires = "export")]
    automation: Option<String>,

    /// Identifiant d'un profil embarqué (défaut : `reel_quebecois`).
    #[arg(long)]
    style: Option<String>,

    /// Dossier d'un profil externe (contient profile.toml et ses banques).
    #[arg(long, value_name = "DOSSIER", conflicts_with = "style")]
    profile: Option<std::path::PathBuf>,

    /// Deterministic session seed
    #[arg(long)]
    seed: Option<u64>,

    /// Podorythmie (`0`, `1`, `2` ou `off` pour exporter sans pieds)
    #[arg(long)]
    feet: Option<String>,

    /// Swing des croches à l'export (0,5 = droit ; remplace le swing du style)
    #[arg(long)]
    swing: Option<f32>,

    /// Tours AABB joués par air avant d'en tirer un nouveau (défaut : `reel_arrangement.toml`)
    #[arg(long, value_name = "N")]
    tours: Option<usize>,
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

    if let Some(path) = args.check_cells.as_deref() {
        // Longueur attendue : celle du profil choisi (`--profile`/`--style`), sinon le
        // profil embarqué. Une banque se valide toujours contre un style précis.
        let profile = if let Some(dir) = &args.profile {
            harmonium_core::StyleProfile::load(dir)?
        } else if let Some(id) = args.style.as_deref() {
            harmonium_core::StyleProfile::named(id)?
        } else {
            harmonium_core::StyleProfile::builtin().clone()
        };
        std::process::exit(check_cells::run(path, profile.melody.cell_units));
    }

    let parsed_style = if let Some(path) = &args.profile {
        Some(Box::new(harmonium_core::StyleProfile::load(path)?))
    } else {
        args.style.as_deref().map(parser::parse_style_value).transpose()?
    };
    let parsed_feet = match args.feet.as_deref() {
        Some(s) => Some(parser::parse_feet_value(s).map_err(|e| anyhow::anyhow!(e))?),
        None => None,
    };
    if let Some(swing) = args.swing {
        if !swing.is_finite() || swing <= 0.0 {
            anyhow::bail!("--swing invalide: {swing} (attendu 0.5-0.75, 0.5 = droit)");
        }
        if !args.export {
            anyhow::bail!("--swing requiert --export");
        }
    }

    // Validate export mode arguments
    if args.export && args.duration.is_none() && args.bars.is_none() {
        eprintln!("Error: --export requires --duration <seconds> or --bars <count>");
        std::process::exit(1);
    }

    let sf2_bytes = harmonium::load_soundfont(args.soundfont.as_deref())?;
    let feet_sf2_bytes = harmonium::load_feet_soundfont(args.feet_soundfont.as_deref())?;
    #[cfg(feature = "odin")]
    let odin_owned = load_odin_presets(args.odin_presets.as_deref())?;
    #[cfg(feature = "odin")]
    let odin_presets: Vec<harmonium_audio::odin_source::OdinPresetSource> = odin_owned
        .iter()
        .map(|(name, bytes)| harmonium_audio::odin_source::OdinPresetSource { name, bytes })
        .collect();
    #[cfg(not(feature = "odin"))]
    let odin_presets: Vec<harmonium_audio::odin_source::OdinPresetSource> = Vec::new();

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
            feet_sf2_bytes.as_deref(),
            &odin_presets,
            args.duration,
            args.bars,
            args.solo_track,
            parsed_style,
            args.seed,
            parsed_feet,
            args.swing,
            args.tours,
            args.automation,
            record_wav,
            args.record_midi,
            args.record_musicxml,
            args.record_truth,
        )?;
    } else {
        // Interactive REPL mode: needs audio device
        println!("Initializing Harmonium engine...");

        let mut host =
            LiveHost::start(sf2_bytes.as_deref(), feet_sf2_bytes.as_deref(), &odin_presets, None)
                .map_err(|e| anyhow::anyhow!(e))?;
        if let Some(seed) = args.seed {
            host.apply(EngineCommand::SetSeed(seed)).map_err(|e| anyhow::anyhow!(e))?;
        }
        if let Some(style) = parsed_style {
            host.apply(EngineCommand::SetStyleProfile(style)).map_err(|e| anyhow::anyhow!(e))?;
        }

        let config = host.session_config();
        println!("Engine initialized");
        println!("  Sample rate: 44100 Hz");
        println!("  Key: {} {}", config.key, config.scale);
        println!("  BPM: {:.1}", config.bpm);
        println!();

        repl::run(host, args.osc)?;
    }

    Ok(())
}

/// Charge les presets `.odin` d'un dossier (nom = tige du fichier, trié) ;
/// `None` = aucun preset (tout SoundFont). Dossier hors dépôt, fourni par le jeu.
#[cfg(feature = "odin")]
fn load_odin_presets(dir: Option<&std::path::Path>) -> Result<Vec<(String, Vec<u8>)>> {
    let Some(dir) = dir else { return Ok(Vec::new()) };
    let mut presets = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("odin")) {
            let name = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default();
            let bytes = std::fs::read(&path)?;
            println!("Preset Odin : {} ({} octets)", name, bytes.len());
            presets.push((name, bytes));
        }
    }
    presets.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(presets)
}

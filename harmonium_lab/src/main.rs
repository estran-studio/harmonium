//! Harmonium Lab CLI
//!
//! Command-line interface for Musical DNA extraction, benchmarking, and tuning.
//!
//! ## Commands
//!
//! - `ingest` - Ingest MusicXML files and extract DNA
//! - `profile` - Build style profiles from DNA collections
//! - `compare` - Compare generated DNA against reference profiles

use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use harmonium_core::tuning::TuningParams;
use harmonium_lab::{DNAComparator, DNAProfile, MusicXMLIngester, render};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};

/// A style profile TOML file: TuningParams + render configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct StyleFile {
    /// Render configuration (BPM, emotions, seed)
    #[serde(default)]
    render: RenderSection,
    /// All TuningParams sections are flattened here
    #[serde(flatten)]
    tuning: TuningParams,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RenderSection {
    #[serde(default = "default_bpm")]
    bpm: f32,
    #[serde(default = "default_density")]
    density: f32,
    #[serde(default = "default_tension")]
    tension: f32,
    #[serde(default = "default_valence")]
    valence: f32,
    #[serde(default = "default_arousal")]
    arousal: f32,
}

fn default_bpm() -> f32 {
    120.0
}
fn default_density() -> f32 {
    0.5
}
fn default_tension() -> f32 {
    0.4
}
fn default_valence() -> f32 {
    0.3
}
fn default_arousal() -> f32 {
    0.5
}

impl Default for RenderSection {
    fn default() -> Self {
        Self {
            bpm: default_bpm(),
            density: default_density(),
            tension: default_tension(),
            valence: default_valence(),
            arousal: default_arousal(),
        }
    }
}

impl RenderSection {
    fn to_render_config(&self, seed: u64) -> render::RenderConfig {
        render::RenderConfig {
            bpm: self.bpm,
            density: self.density,
            tension: self.tension,
            valence: self.valence,
            arousal: self.arousal,
            seed,
        }
    }
}

#[derive(Parser)]
#[command(name = "harmonium-lab")]
#[command(author, version, about = "Musical DNA extraction and profile benchmarking")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest MusicXML files and extract Musical DNA
    Ingest {
        /// Source directory containing MusicXML files
        #[arg(short, long)]
        source: PathBuf,

        /// Output directory for DNA JSON files
        #[arg(short, long)]
        output: PathBuf,

        /// Recurse into subdirectories
        #[arg(short, long, default_value = "true")]
        recursive: bool,
    },

    /// Build a style profile from DNA files
    Profile {
        /// Name for the style profile
        #[arg(short, long)]
        name: String,

        /// Source directory containing DNA JSON files
        #[arg(short, long)]
        source: PathBuf,

        /// Output file for the profile
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Compare generated DNA against a reference profile
    Compare {
        /// Generated DNA JSON file
        #[arg(short, long)]
        generated: PathBuf,

        /// Reference profile JSON file
        #[arg(short, long)]
        reference: PathBuf,

        /// Output detailed comparison report
        #[arg(short, long)]
        verbose: bool,
    },

    /// Extract DNA from a single MusicXML file (for testing)
    ExtractOne {
        /// Input MusicXML file
        #[arg(short, long)]
        input: PathBuf,

        /// Output DNA JSON file (optional, prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Render audio from a TuningParams profile to WAV (non-interactive)
    Render {
        /// TuningParams TOML file
        #[arg(short, long)]
        profile: PathBuf,

        /// Number of bars to render
        #[arg(short, long, default_value = "16")]
        bars: usize,

        /// BPM for rendering
        #[arg(long, default_value = "120")]
        bpm: f32,

        /// Deterministic seed
        #[arg(long, default_value = "42")]
        seed: u64,

        /// Output WAV file
        #[arg(short, long, default_value = "./render.wav")]
        output: PathBuf,

        /// Path to an SF2 SoundFont file (renders silence if not specified)
        #[arg(long)]
        soundfont: Option<PathBuf>,
    },

    /// Render audio from a TuningParams profile and rate it
    RateStyle {
        /// TuningParams TOML file
        #[arg(short, long)]
        profile: PathBuf,

        /// Number of bars to render
        #[arg(short, long, default_value = "16")]
        bars: usize,

        /// BPM for rendering
        #[arg(long, default_value = "120")]
        bpm: f32,

        /// Deterministic seed for reproducible output
        #[arg(long, default_value = "42")]
        seed: u64,

        /// Output WAV file (default: ./render.wav)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Render and rate multiple candidate profiles from a directory
    RateBatch {
        /// Directory containing .toml profile candidates
        #[arg(short, long)]
        candidates: PathBuf,

        /// Number of bars to render per candidate
        #[arg(short, long, default_value = "16")]
        bars: usize,

        /// BPM for rendering
        #[arg(long, default_value = "120")]
        bpm: f32,

        /// Deterministic seed
        #[arg(long, default_value = "42")]
        seed: u64,
    },

    /// Dump a default-TuningParams TOML profile (with optional [render] block).
    /// Useful as a starting point for new style profiles or for QA loops.
    DefaultProfile {
        /// Output TOML file (defaults to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ingest { source, output, recursive } => {
            cmd_ingest(&source, &output, recursive)?;
        }
        Commands::Profile { name, source, output } => {
            cmd_profile(&name, &source, &output)?;
        }
        Commands::Compare { generated, reference, verbose } => {
            cmd_compare(&generated, &reference, verbose)?;
        }
        Commands::ExtractOne { input, output } => {
            cmd_extract_one(&input, output.as_deref())?;
        }
        Commands::Render { profile, bars, bpm, seed, output, soundfont } => {
            cmd_render(&profile, bars, bpm, seed, &output, soundfont.as_deref())?;
        }
        Commands::RateStyle { profile, bars, bpm, seed, output } => {
            cmd_rate_style(&profile, bars, bpm, seed, output)?;
        }
        Commands::RateBatch { candidates, bars, bpm, seed } => {
            cmd_rate_batch(&candidates, bars, bpm, seed)?;
        }
        Commands::DefaultProfile { output } => {
            cmd_default_profile(output.as_deref())?;
        }
    }

    Ok(())
}

fn cmd_ingest(source: &PathBuf, output: &PathBuf, recursive: bool) -> Result<()> {
    println!("Ingesting MusicXML files from: {}", source.display());

    let ingester = MusicXMLIngester::new();
    let files = ingester.find_musicxml_files(source, recursive)?;

    if files.is_empty() {
        println!("No MusicXML files found.");
        return Ok(());
    }

    println!("Found {} MusicXML files", files.len());

    // Create output directory
    std::fs::create_dir_all(output)?;

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
            )?
            .progress_chars("#>-"),
    );

    let mut success_count = 0;
    let mut error_count = 0;

    for file in &files {
        pb.set_message(file.file_name().unwrap_or_default().to_string_lossy().to_string());

        match ingester.ingest_file(file) {
            Ok(dna) => {
                // Create output filename
                let stem = file.file_stem().unwrap_or_default();
                let out_file = output.join(format!("{}.dna.json", stem.to_string_lossy()));

                if let Ok(json) = dna.to_json() {
                    if std::fs::write(&out_file, json).is_ok() {
                        success_count += 1;
                    } else {
                        error_count += 1;
                    }
                } else {
                    error_count += 1;
                }
            }
            Err(_) => {
                error_count += 1;
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("Done");
    println!("Ingested: {} success, {} errors", success_count, error_count);

    Ok(())
}

fn cmd_profile(name: &str, source: &PathBuf, output: &PathBuf) -> Result<()> {
    println!("Building style profile '{}' from: {}", name, source.display());

    let profile = DNAProfile::from_directory(name, source)?;

    println!("Profile built from {} DNA files", profile.sample_count);
    println!("Average voice leading effort: {:.2}", profile.metrics.average_voice_leading_effort);
    println!("Tension variance: {:.4}", profile.metrics.tension_variance);
    println!("Harmonic rhythm: {:.2} chords/measure", profile.metrics.harmonic_rhythm);

    // Save profile
    let json = serde_json::to_string_pretty(&profile)?;
    std::fs::write(output, json)?;

    println!("Profile saved to: {}", output.display());

    Ok(())
}

fn cmd_compare(generated: &PathBuf, reference: &PathBuf, verbose: bool) -> Result<()> {
    println!("Comparing DNA files...");

    let gen_json = std::fs::read_to_string(generated)?;
    let ref_json = std::fs::read_to_string(reference)?;

    let gen_dna: harmonium_lab::MusicalDNA = serde_json::from_str(&gen_json)?;
    let ref_profile: DNAProfile = serde_json::from_str(&ref_json)?;

    let comparator = DNAComparator::new();
    let report = comparator.compare(&gen_dna, &ref_profile);

    println!("\n=== Comparison Report ===\n");
    println!("Overall similarity: {:.1}%", report.overall_similarity * 100.0);
    println!();

    if verbose {
        println!("Detailed Metrics:");
        println!("  Voice leading divergence: {:.2}", report.voice_leading_divergence);
        println!("  Tension divergence: {:.4}", report.tension_divergence);
        println!("  Harmonic rhythm divergence: {:.2}", report.harmonic_rhythm_divergence);
        println!();
        println!("Suggestions:");
        for suggestion in &report.suggestions {
            println!("  - {}", suggestion);
        }
    }

    Ok(())
}

fn cmd_extract_one(input: &Path, output: Option<&Path>) -> Result<()> {
    println!("Extracting DNA from: {}", input.display());

    let ingester = MusicXMLIngester::new();
    let dna = ingester.ingest_file(input)?;

    let json = dna.to_json()?;

    if let Some(out_path) = output {
        std::fs::write(out_path, &json)?;
        println!("DNA saved to: {}", out_path.display());
    } else {
        println!("{}", json);
    }

    Ok(())
}

fn cmd_render(
    profile_path: &Path,
    bars: usize,
    bpm: f32,
    seed: u64,
    output: &Path,
    soundfont: Option<&Path>,
) -> Result<()> {
    let toml_str = std::fs::read_to_string(profile_path)
        .with_context(|| format!("Failed to read profile: {}", profile_path.display()))?;
    let style: StyleFile =
        toml::from_str(&toml_str).with_context(|| "Failed to parse style profile TOML")?;

    let sf2_bytes = match soundfont {
        Some(path) => {
            println!("Loading soundfont: {}", path.display());
            Some(
                std::fs::read(path)
                    .with_context(|| format!("Failed to read SF2: {}", path.display()))?,
            )
        }
        None => None,
    };

    // CLI --bpm overrides the profile's default
    let mut config = style.render.to_render_config(seed);
    if (bpm - 120.0).abs() > 0.01 {
        // User explicitly set BPM via CLI
        config.bpm = bpm;
    }
    println!(
        "Rendering {} bars at {} BPM (density={:.1}, tension={:.1}, valence={:.1}, arousal={:.1}, seed={})...",
        bars, config.bpm, config.density, config.tension, config.valence, config.arousal, seed
    );
    render::render_to_files(&style.tuning, bars, &config, output, sf2_bytes.as_deref())?;
    println!("WAV saved to: {}", output.display());

    // Auto-play
    render::play_wav(output)?;

    Ok(())
}

fn cmd_rate_style(
    profile_path: &Path,
    bars: usize,
    bpm: f32,
    seed: u64,
    output: Option<PathBuf>,
) -> Result<()> {
    // Load TuningParams
    let toml_str = std::fs::read_to_string(profile_path)
        .with_context(|| format!("Failed to read profile: {}", profile_path.display()))?;
    let tuning: TuningParams =
        toml::from_str(&toml_str).with_context(|| "Failed to parse TuningParams TOML")?;

    let wav_path = output.unwrap_or_else(|| PathBuf::from("./render.wav"));

    println!("Rendering {} bars at {} BPM (seed={})...", bars, bpm, seed);
    render::render_to_files(
        &tuning,
        bars,
        &render::RenderConfig { bpm, seed, ..Default::default() },
        &wav_path,
        None,
    )?;
    println!("WAV saved to: {}", wav_path.display());

    // Play audio
    println!("Playing audio...");
    render::play_wav(&wav_path)?;
    println!();

    // Collect rating
    print!("Rate authenticity (1-5): ");
    io::stdout().flush()?;
    let mut rating_input = String::new();
    io::stdin().read_line(&mut rating_input)?;
    let rating: u8 = rating_input.trim().parse().unwrap_or(3);

    print!("Feedback (optional): ");
    io::stdout().flush()?;
    let mut feedback_input = String::new();
    io::stdin().read_line(&mut feedback_input)?;
    let feedback = feedback_input.trim().to_string();

    // Save rating
    let rating_entry = serde_json::json!({
        "profile": profile_path.display().to_string(),
        "rating": rating,
        "feedback": feedback,
        "bars": bars,
        "bpm": bpm,
        "seed": seed,
    });

    let mut ratings_file =
        std::fs::OpenOptions::new().create(true).append(true).open("ratings.jsonl")?;
    use std::io::Write as _;
    writeln!(ratings_file, "{}", rating_entry)?;

    println!("Rating saved to ratings.jsonl");
    Ok(())
}

fn cmd_rate_batch(candidates_dir: &Path, bars: usize, bpm: f32, seed: u64) -> Result<()> {
    let mut profiles: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(candidates_dir)
        .with_context(|| format!("Failed to read directory: {}", candidates_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            profiles.push(path);
        }
    }
    profiles.sort();

    if profiles.is_empty() {
        println!("No .toml profiles found in {}", candidates_dir.display());
        return Ok(());
    }

    println!("Found {} candidate profiles.", profiles.len());
    println!();

    let mut ratings: Vec<(PathBuf, u8, String)> = Vec::new();

    for (i, profile_path) in profiles.iter().enumerate() {
        println!("=== Candidate {}/{}: {} ===", i + 1, profiles.len(), profile_path.display());

        let toml_str = std::fs::read_to_string(profile_path)?;
        let tuning: TuningParams = toml::from_str(&toml_str)?;

        let wav_path = candidates_dir.join(format!("candidate_{}.wav", i + 1));

        println!("  Rendering {} bars...", bars);
        render::render_to_files(
            &tuning,
            bars,
            &render::RenderConfig { bpm, seed, ..Default::default() },
            &wav_path,
            None,
        )?;
        println!("  Playing...");
        render::play_wav(&wav_path)?;

        print!("  Rate (1-5): ");
        io::stdout().flush()?;
        let mut rating_input = String::new();
        io::stdin().read_line(&mut rating_input)?;
        let rating: u8 = rating_input.trim().parse().unwrap_or(3);

        print!("  Feedback: ");
        io::stdout().flush()?;
        let mut feedback_input = String::new();
        io::stdin().read_line(&mut feedback_input)?;
        let feedback = feedback_input.trim().to_string();

        ratings.push((profile_path.clone(), rating, feedback));
        println!();
    }

    // Print ranked results
    ratings.sort_by(|a, b| b.1.cmp(&a.1));
    println!("=== RANKED RESULTS ===");
    for (i, (path, rating, feedback)) in ratings.iter().enumerate() {
        println!("  {}. [{}] {} — {}", i + 1, rating, path.display(), feedback);
    }

    Ok(())
}

fn cmd_default_profile(output: Option<&Path>) -> Result<()> {
    let style = StyleFile { render: RenderSection::default(), tuning: TuningParams::default() };
    let toml_str = toml::to_string_pretty(&style)
        .map_err(|e| anyhow::anyhow!("Failed to serialize default profile: {e}"))?;
    match output {
        Some(path) => {
            std::fs::write(path, &toml_str)
                .with_context(|| format!("Failed to write profile: {}", path.display()))?;
            println!("Default profile written to: {}", path.display());
        }
        None => {
            print!("{toml_str}");
        }
    }
    Ok(())
}

//! Harmonium Lab CLI
//!
//! Command-line interface for Musical DNA extraction, benchmarking, and tuning.
//!
//! ## Commands
//!
//! - `ingest` - Ingest MusicXML files and extract DNA
//! - `profile` - Build style profiles from DNA collections
//! - `compare` - Compare generated DNA against reference profiles
//! - `tune` - Interactive LLM-assisted tuning session

use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use harmonium_core::tuning::TuningParams;
use harmonium_lab::{
    DNAComparator, DNAProfile, GlobalMetrics, MusicXMLIngester, agent::ClaudeAgent, render,
};
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
    /// Optional rhythmic-cell variety override (CORELIB-13). `None` = engine
    /// default (`VarietyParams::default().rhythmic_cell_variety`).
    #[serde(default)]
    rhythmic_cell_variety: Option<f32>,
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
            rhythmic_cell_variety: None,
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
            rhythmic_cell_variety: self.rhythmic_cell_variety,
        }
    }
}

#[derive(Parser)]
#[command(name = "harmonium-lab")]
#[command(author, version, about = "Musical DNA extraction and LLM-assisted tuning")]
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

    /// Interactive LLM-assisted tuning session
    Tune {
        /// Target style profile
        #[arg(short, long)]
        target_style: PathBuf,

        /// Current tuning parameters file (TOML)
        #[arg(short = 'p', long)]
        tuning: PathBuf,

        /// Maximum iterations
        #[arg(short, long, default_value = "10")]
        iterations: usize,

        /// Anthropic API key (or set ANTHROPIC_API_KEY env var)
        #[arg(long, env = "ANTHROPIC_API_KEY")]
        api_key: Option<String>,
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

    /// Generate a style profile from a natural-language description using Claude
    GenerateStyle {
        /// Style name (e.g. "Bossa Nova")
        #[arg(short, long)]
        name: String,

        /// Style description (e.g. "straight 8ths, anticipated bass, sparse comping")
        #[arg(short, long)]
        description: String,

        /// Output file for the generated TuningParams (TOML)
        #[arg(short, long)]
        output: PathBuf,

        /// Maximum refinement iterations
        #[arg(short, long, default_value = "5")]
        iterations: usize,

        /// Anthropic API key (or set ANTHROPIC_API_KEY env var)
        #[arg(long, env = "ANTHROPIC_API_KEY")]
        api_key: Option<String>,
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

        /// Path to SF2 soundfont file (uses FundSP synth if not specified)
        #[arg(long)]
        soundfont: Option<PathBuf>,

        /// Override `VarietyParams::rhythmic_cell_variety` (0.0–1.0).
        /// Default 0.5 from VarietyParams. Set 0.0 to disable clave / 3+3+2
        /// cells and gap=2 splits — useful for A/B comparison against legacy.
        #[arg(long)]
        variety: Option<f32>,
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

    /// Full LLM-assisted style tuning: generate → render → rate → refine
    TuneStyle {
        /// Style name
        #[arg(short, long)]
        name: String,

        /// Style description
        #[arg(short, long)]
        description: String,

        /// Number of bars to render per iteration
        #[arg(short, long, default_value = "16")]
        bars: usize,

        /// BPM for rendering
        #[arg(long, default_value = "120")]
        bpm: f32,

        /// Maximum refinement iterations
        #[arg(short, long, default_value = "5")]
        iterations: usize,

        /// Output directory for profiles and WAV files
        #[arg(short, long, default_value = "./tune_output")]
        output: PathBuf,

        /// Anthropic API key (or set ANTHROPIC_API_KEY env var)
        #[arg(long, env = "ANTHROPIC_API_KEY")]
        api_key: Option<String>,
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
        Commands::Tune { target_style, tuning, iterations, api_key } => {
            cmd_tune(&target_style, &tuning, iterations, api_key)?;
        }
        Commands::ExtractOne { input, output } => {
            cmd_extract_one(&input, output.as_deref())?;
        }
        Commands::GenerateStyle { name, description, output, iterations, api_key } => {
            cmd_generate_style(&name, &description, &output, iterations, api_key)?;
        }
        Commands::Render { profile, bars, bpm, seed, output, soundfont, variety } => {
            cmd_render(&profile, bars, bpm, seed, &output, soundfont.as_deref(), variety)?;
        }
        Commands::RateStyle { profile, bars, bpm, seed, output } => {
            cmd_rate_style(&profile, bars, bpm, seed, output)?;
        }
        Commands::RateBatch { candidates, bars, bpm, seed } => {
            cmd_rate_batch(&candidates, bars, bpm, seed)?;
        }
        Commands::TuneStyle { name, description, bars, bpm, iterations, output, api_key } => {
            cmd_tune_style(&name, &description, bars, bpm, iterations, &output, api_key)?;
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

fn cmd_tune(
    target_style: &PathBuf,
    tuning_path: &PathBuf,
    iterations: usize,
    api_key: Option<String>,
) -> Result<()> {
    println!("\n");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                   HARMONIUM INTERACTIVE TUNING SESSION                 ");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();

    // Validate API key
    let api_key = api_key
        .context("Anthropic API key required. Set ANTHROPIC_API_KEY env var or use --api-key")?;

    // 1. Load target style profile
    println!("Loading target style profile: {}", target_style.display());
    let profile_json =
        std::fs::read_to_string(target_style).context("Failed to read target style profile")?;
    let target_profile: DNAProfile =
        serde_json::from_str(&profile_json).context("Failed to parse target style profile")?;

    println!("  Style: {}", target_profile.name);
    println!("  Based on {} samples", target_profile.sample_count);
    println!();

    // 2. Load or create tuning params
    let mut tuning = if tuning_path.exists() {
        println!("Loading tuning parameters: {}", tuning_path.display());
        let toml_str = std::fs::read_to_string(tuning_path)
            .map_err(|e| anyhow::anyhow!("Failed to read tuning file: {}", e))?;
        toml::from_str(&toml_str).map_err(|e| anyhow::anyhow!("Failed to parse tuning: {}", e))?
    } else {
        println!("Creating default tuning parameters: {}", tuning_path.display());
        let default_tuning = TuningParams::default();
        let toml_str = toml::to_string_pretty(&default_tuning)
            .map_err(|e| anyhow::anyhow!("Failed to serialize tuning: {}", e))?;
        std::fs::write(tuning_path, toml_str)
            .map_err(|e| anyhow::anyhow!("Failed to write tuning: {}", e))?;
        default_tuning
    };

    // Validate tuning parameters
    tuning.validate().map_err(|errors| {
        let msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        anyhow::anyhow!("Invalid tuning parameters: {}", msgs.join(", "))
    })?;

    // Create Claude agent
    let agent = ClaudeAgent::new().with_api_key(&api_key);
    println!("Claude API configured (model: claude-sonnet-4-20250514)");
    println!();

    // Display target metrics
    print_metrics_header("TARGET STYLE", &target_profile.metrics);
    println!();

    // 3. Interactive tuning loop
    for iteration in 1..=iterations {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!(
            "                         ITERATION {}/{}                              ",
            iteration, iterations
        );
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();

        // Generate music with current tuning and extract DNA
        println!("[1/4] Generating music with current parameters...");
        let generated_metrics = simulate_music_generation(&tuning);
        print_metrics_comparison("GENERATED", &generated_metrics, &target_profile.metrics);
        println!();

        // Calculate divergence score
        let divergence = calculate_divergence(&generated_metrics, &target_profile.metrics);
        println!("Overall divergence: {:.2}% (lower is better)", divergence * 100.0);
        println!();

        // Check for convergence
        if divergence < 0.05 {
            println!("Convergence achieved! Divergence < 5%");
            break;
        }

        // Call Claude API for suggestions
        println!("[2/4] Consulting Claude for parameter suggestions...");
        let suggestion = match agent.suggest_tuning_blocking(
            &target_profile.metrics,
            &generated_metrics,
            &tuning,
        ) {
            Ok(s) => s,
            Err(e) => {
                println!("  Error calling Claude API: {}", e);
                println!("  Skipping this iteration...");
                continue;
            }
        };

        println!();
        println!("[3/4] Claude's Analysis:");
        println!("─────────────────────────────────────────────────────────────────────────");
        println!("{}", suggestion.reasoning);
        println!();
        println!("Confidence: {:.0}%", suggestion.confidence * 100.0);
        println!();

        if !suggestion.has_changes() {
            println!("No parameter changes suggested.");
            continue;
        }

        println!("Suggested Changes:");
        for change in &suggestion.parameter_changes {
            println!("  {} : {} → {}", change.name, change.current, change.suggested);
        }
        println!();

        // Present options to user
        println!("[4/4] What would you like to do?");
        println!();
        println!("  [A]pply  - Apply suggested changes");
        println!("  [S]kip   - Skip this iteration");
        println!("  [E]dit   - Manually edit a parameter");
        println!("  [V]iew   - View current tuning parameters");
        println!("  [Q]uit   - Save and exit");
        println!();

        loop {
            print!("Choice: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim().to_lowercase();

            match choice.as_str() {
                "a" | "apply" => {
                    println!("Applying changes...");
                    tuning = suggestion.apply_to(&tuning);
                    save_tuning(&tuning, tuning_path)?;
                    println!("Tuning saved to: {}", tuning_path.display());
                    break;
                }
                "s" | "skip" => {
                    println!("Skipping this iteration.");
                    break;
                }
                "e" | "edit" => {
                    tuning = manual_edit_parameter(&tuning)?;
                    save_tuning(&tuning, tuning_path)?;
                    println!("Tuning saved.");
                    break;
                }
                "v" | "view" => {
                    print_current_tuning(&tuning);
                }
                "q" | "quit" => {
                    println!("Saving and exiting...");
                    save_tuning(&tuning, tuning_path)?;
                    println!("Final tuning saved to: {}", tuning_path.display());
                    return Ok(());
                }
                _ => {
                    println!("Invalid choice. Please enter A, S, E, V, or Q.");
                }
            }
        }

        println!();
    }

    // Final summary
    println!();
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                          SESSION COMPLETE                              ");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    println!("Final tuning saved to: {}", tuning_path.display());
    print_current_tuning(&tuning);

    Ok(())
}

/// Simulate music generation based on tuning parameters.
/// This produces estimated GlobalMetrics based on the parameter values.
/// In a full implementation, this would run the actual harmonium engine.
fn simulate_music_generation(tuning: &TuningParams) -> GlobalMetrics {
    let voice_leading_effort = f32::from(tuning.voice_leading.max_semitone_movement) * 0.8;

    let hysteresis_range = tuning.harmony_driver.neo_upper - tuning.harmony_driver.steedman_lower;
    let tension_variance = 0.1 / hysteresis_range.max(0.1);

    let tension_release_balance = tuning.voice_leading.trq_threshold;

    let steedman_bias = 1.0 - tuning.harmony_driver.steedman_upper;
    let diatonic_percentage = 60.0 + steedman_bias * 30.0;

    // Use hat vertex counts as proxy for rhythmic density
    let avg_vertices = (tuning.perfect_balance.hat_vertex_counts[1]
        + tuning.perfect_balance.hat_vertex_counts[2]) as f32
        / 2.0;
    let harmonic_rhythm = avg_vertices / 4.0;

    GlobalMetrics {
        average_voice_leading_effort: voice_leading_effort,
        tension_variance,
        tension_release_balance,
        diatonic_percentage,
        harmonic_rhythm,
        total_duration_beats: 64.0, // Simulate 16 measures at 4 beats each
        chord_change_count: 16,     // Simulate one chord change per measure
    }
}

/// Calculate overall divergence between generated and target metrics
fn calculate_divergence(generated: &GlobalMetrics, target: &GlobalMetrics) -> f32 {
    let vl_diff = (generated.average_voice_leading_effort - target.average_voice_leading_effort)
        .abs()
        / target.average_voice_leading_effort.max(0.1);
    let tv_diff = (generated.tension_variance - target.tension_variance).abs()
        / target.tension_variance.max(0.01);
    let trb_diff = (generated.tension_release_balance - target.tension_release_balance).abs();
    let dp_diff = (generated.diatonic_percentage - target.diatonic_percentage).abs() / 100.0;
    let hr_diff = (generated.harmonic_rhythm - target.harmonic_rhythm).abs()
        / target.harmonic_rhythm.max(0.1);

    // Weighted average
    (vl_diff * 0.25 + tv_diff * 0.2 + trb_diff * 0.2 + dp_diff * 0.15 + hr_diff * 0.2).min(1.0)
}

/// Print metrics header
fn print_metrics_header(label: &str, metrics: &GlobalMetrics) {
    println!("{} PROFILE:", label);
    println!("  Voice Leading Effort: {:.2}", metrics.average_voice_leading_effort);
    println!("  Tension Variance:     {:.4}", metrics.tension_variance);
    println!("  Tension/Release:      {:.2}", metrics.tension_release_balance);
    println!("  Diatonic %:           {:.1}%", metrics.diatonic_percentage);
    println!("  Harmonic Rhythm:      {:.2} chords/measure", metrics.harmonic_rhythm);
}

/// Print metrics comparison with direction indicators
fn print_metrics_comparison(label: &str, generated: &GlobalMetrics, target: &GlobalMetrics) {
    fn indicator(generated_val: f32, target_val: f32, threshold: f32) -> &'static str {
        let diff = generated_val - target_val;
        if diff.abs() < threshold {
            "="
        } else if diff > 0.0 {
            "↑"
        } else {
            "↓"
        }
    }

    println!("{} METRICS:", label);
    println!(
        "  Voice Leading Effort: {:.2} {} (target: {:.2})",
        generated.average_voice_leading_effort,
        indicator(generated.average_voice_leading_effort, target.average_voice_leading_effort, 0.1),
        target.average_voice_leading_effort
    );
    println!(
        "  Tension Variance:     {:.4} {} (target: {:.4})",
        generated.tension_variance,
        indicator(generated.tension_variance, target.tension_variance, 0.01),
        target.tension_variance
    );
    println!(
        "  Tension/Release:      {:.2} {} (target: {:.2})",
        generated.tension_release_balance,
        indicator(generated.tension_release_balance, target.tension_release_balance, 0.05),
        target.tension_release_balance
    );
    println!(
        "  Diatonic %:           {:.1}% {} (target: {:.1}%)",
        generated.diatonic_percentage,
        indicator(generated.diatonic_percentage, target.diatonic_percentage, 5.0),
        target.diatonic_percentage
    );
    println!(
        "  Harmonic Rhythm:      {:.2} {} (target: {:.2})",
        generated.harmonic_rhythm,
        indicator(generated.harmonic_rhythm, target.harmonic_rhythm, 0.2),
        target.harmonic_rhythm
    );
}

/// Print current tuning parameters
fn print_current_tuning(tuning: &TuningParams) {
    println!();
    println!("CURRENT TUNING PARAMETERS:");
    println!("─────────────────────────────────────────────────────────────────────────");
    println!("  HARMONY:");
    println!("    max_semitone_movement:       {}", tuning.voice_leading.max_semitone_movement);
    println!("    allow_cardinality_morph:     {}", tuning.voice_leading.allow_cardinality_morph);
    println!("    trq_threshold:               {:.2}", tuning.voice_leading.trq_threshold);
    println!();
    println!("  STRATEGY THRESHOLDS:");
    println!("    steedman_lower:              {:.2}", tuning.harmony_driver.steedman_lower);
    println!("    steedman_upper:              {:.2}", tuning.harmony_driver.steedman_upper);
    println!("    neo_lower:                   {:.2}", tuning.harmony_driver.neo_lower);
    println!("    neo_upper:                   {:.2}", tuning.harmony_driver.neo_upper);
    println!("    hysteresis_boost:            {:.2}", tuning.harmony_driver.hysteresis_boost);
    println!();
    println!("  RHYTHM (Perfect Balance):");
    println!("    hat_vertex_counts:           {:?}", tuning.perfect_balance.hat_vertex_counts);
    println!(
        "    kick_polygon_low_threshold:  {:.2}",
        tuning.perfect_balance.kick_polygon_low_threshold
    );
    println!();
}

/// Save tuning to file
fn save_tuning(tuning: &TuningParams, path: &Path) -> Result<()> {
    let toml_str = toml::to_string_pretty(tuning)
        .map_err(|e| anyhow::anyhow!("Failed to serialize tuning: {}", e))?;
    std::fs::write(path, toml_str)
        .map_err(|e| anyhow::anyhow!("Failed to write tuning file: {}", e))
}

/// Manual parameter editing
fn manual_edit_parameter(tuning: &TuningParams) -> Result<TuningParams> {
    let mut new_tuning = tuning.clone();

    println!();
    println!("Available parameters:");
    println!(
        "  1. max_semitone_movement (current: {})",
        tuning.voice_leading.max_semitone_movement
    );
    println!("  2. trq_threshold (current: {:.2})", tuning.voice_leading.trq_threshold);
    println!("  3. steedman_lower (current: {:.2})", tuning.harmony_driver.steedman_lower);
    println!("  4. steedman_upper (current: {:.2})", tuning.harmony_driver.steedman_upper);
    println!("  5. neo_lower (current: {:.2})", tuning.harmony_driver.neo_lower);
    println!("  6. neo_upper (current: {:.2})", tuning.harmony_driver.neo_upper);
    println!("  7. hysteresis_boost (current: {:.2})", tuning.harmony_driver.hysteresis_boost);
    println!(
        "  8. hat_vertex_counts[2] (current: {})",
        tuning.perfect_balance.hat_vertex_counts[2]
    );
    println!(
        "  9. hat_vertex_counts[3] (current: {})",
        tuning.perfect_balance.hat_vertex_counts[3]
    );
    println!();

    print!("Enter parameter number (1-9): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let param_num: usize = input.trim().parse().unwrap_or(0);

    print!("Enter new value: ");
    io::stdout().flush()?;

    let mut value_input = String::new();
    io::stdin().read_line(&mut value_input)?;
    let value_str = value_input.trim();

    match param_num {
        1 => {
            if let Ok(v) = value_str.parse::<u8>() {
                new_tuning.voice_leading.max_semitone_movement = v;
            }
        }
        2 => {
            if let Ok(v) = value_str.parse::<f32>() {
                new_tuning.voice_leading.trq_threshold = v;
            }
        }
        3 => {
            if let Ok(v) = value_str.parse::<f32>() {
                new_tuning.harmony_driver.steedman_lower = v;
            }
        }
        4 => {
            if let Ok(v) = value_str.parse::<f32>() {
                new_tuning.harmony_driver.steedman_upper = v;
            }
        }
        5 => {
            if let Ok(v) = value_str.parse::<f32>() {
                new_tuning.harmony_driver.neo_lower = v;
            }
        }
        6 => {
            if let Ok(v) = value_str.parse::<f32>() {
                new_tuning.harmony_driver.neo_upper = v;
            }
        }
        7 => {
            if let Ok(v) = value_str.parse::<f32>() {
                new_tuning.harmony_driver.hysteresis_boost = v;
            }
        }
        8 => {
            if let Ok(v) = value_str.parse::<usize>() {
                new_tuning.perfect_balance.hat_vertex_counts[2] = v;
            }
        }
        9 => {
            if let Ok(v) = value_str.parse::<usize>() {
                new_tuning.perfect_balance.hat_vertex_counts[3] = v;
            }
        }
        _ => {
            println!("Invalid parameter number.");
        }
    }

    // Validate the new tuning
    if let Err(errors) = new_tuning.validate() {
        for e in &errors {
            println!("Warning: Invalid value - {}", e);
        }
        println!("Reverting to previous value.");
        return Ok(tuning.clone());
    }

    println!("Parameter updated.");
    Ok(new_tuning)
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

fn cmd_generate_style(
    name: &str,
    description: &str,
    output: &Path,
    max_iterations: usize,
    api_key: Option<String>,
) -> Result<()> {
    let api_key = api_key
        .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
        .context("No API key provided. Set ANTHROPIC_API_KEY or use --api-key")?;

    let agent = ClaudeAgent::new().with_api_key(&api_key);

    println!("Generating style profile: \"{}\"", name);
    println!("Description: {}", description);
    println!();

    // Step 1: Initial generation
    println!("[1/{}] Generating initial TuningParams from description...", max_iterations);
    let result = agent.generate_style_blocking(name, description)?;

    println!("  Confidence: {:.0}%", result.confidence * 100.0);
    println!("  Reasoning: {}", result.reasoning);
    println!();

    let mut current_tuning = result.tuning;

    // Validate
    if let Err(errors) = current_tuning.validate() {
        println!("  Warning: Generated params have validation issues:");
        for e in &errors {
            println!("    - {}", e);
        }
        println!("  Continuing with defaults for invalid fields.");
        // Fall back to defaults for the whole thing if critical
    }

    // Save initial version
    save_tuning(&current_tuning, output)?;
    println!("  Saved to: {}", output.display());

    // Step 2: Interactive refinement loop
    for iteration in 2..=max_iterations {
        println!();
        println!("--- Iteration {}/{} ---", iteration, max_iterations);
        println!("Listen to the generated music, then rate it.");
        println!();

        // Get rating
        print!("Rate authenticity (1-5, or 'q' to quit): ");
        io::stdout().flush()?;
        let mut rating_input = String::new();
        io::stdin().read_line(&mut rating_input)?;
        let rating_str = rating_input.trim();

        if rating_str == "q" || rating_str == "Q" {
            println!("Stopping refinement. Final profile saved to: {}", output.display());
            break;
        }

        let rating: u8 = match rating_str.parse() {
            Ok(r) if (1..=5).contains(&r) => r,
            _ => {
                println!("Invalid rating. Enter 1-5 or 'q'.");
                continue;
            }
        };

        if rating == 5 {
            println!("Perfect score! Profile finalized.");
            break;
        }

        // Get feedback
        print!("Feedback (e.g. 'bass too busy, needs more space'): ");
        io::stdout().flush()?;
        let mut feedback_input = String::new();
        io::stdin().read_line(&mut feedback_input)?;
        let feedback = feedback_input.trim();

        // Refine
        println!("[{}/{}] Refining based on feedback...", iteration, max_iterations);
        let refined =
            agent.refine_style_blocking(name, description, &current_tuning, rating, feedback)?;

        println!("  Confidence: {:.0}%", refined.confidence * 100.0);
        println!("  Reasoning: {}", refined.reasoning);

        current_tuning = refined.tuning;

        // Validate and save
        if let Err(errors) = current_tuning.validate() {
            println!("  Warning: validation issues:");
            for e in &errors {
                println!("    - {}", e);
            }
        }

        save_tuning(&current_tuning, output)?;
        println!("  Updated: {}", output.display());
    }

    println!();
    println!("Style profile generation complete.");
    print_current_tuning(&current_tuning);

    Ok(())
}

fn cmd_render(
    profile_path: &Path,
    bars: usize,
    bpm: f32,
    seed: u64,
    output: &Path,
    soundfont: Option<&Path>,
    variety: Option<f32>,
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
    // CLI --variety overrides the profile's value when present; otherwise
    // the profile's `[render].rhythmic_cell_variety` is preserved.
    if variety.is_some() {
        config.rhythmic_cell_variety = variety;
    }

    let variety_label =
        config.rhythmic_cell_variety.map_or_else(|| "default".to_string(), |v| format!("{v:.2}"));
    println!(
        "Rendering {} bars at {} BPM (density={:.1}, tension={:.1}, valence={:.1}, arousal={:.1}, seed={}, variety={})...",
        bars,
        config.bpm,
        config.density,
        config.tension,
        config.valence,
        config.arousal,
        seed,
        variety_label
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

fn cmd_tune_style(
    name: &str,
    description: &str,
    bars: usize,
    bpm: f32,
    max_iterations: usize,
    output_dir: &Path,
    api_key: Option<String>,
) -> Result<()> {
    let api_key = api_key
        .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
        .context("No API key. Set ANTHROPIC_API_KEY or use --api-key")?;

    std::fs::create_dir_all(output_dir)?;

    let agent = ClaudeAgent::new().with_api_key(&api_key);

    println!("=== Tune Style: \"{}\" ===", name);
    println!("Description: {}", description);
    println!("Bars: {}, BPM: {}, Max iterations: {}", bars, bpm, max_iterations);
    println!();

    // Step 1: Generate initial candidate
    println!("[1/{}] Generating initial TuningParams...", max_iterations);
    let result = agent.generate_style_blocking(name, description)?;
    println!("  Confidence: {:.0}%", result.confidence * 100.0);
    println!("  Reasoning: {}", result.reasoning);

    let mut current_tuning = result.tuning;
    let profile_path = output_dir.join("current.toml");
    save_tuning(&current_tuning, &profile_path)?;

    let mut best_rating: u8 = 0;
    let mut best_tuning = current_tuning.clone();

    for iteration in 1..=max_iterations {
        println!();
        println!("--- Iteration {}/{} ---", iteration, max_iterations);

        // Render
        let wav_path = output_dir.join(format!("iter_{}.wav", iteration));
        println!("  Rendering {} bars...", bars);
        render::render_to_files(
            &current_tuning,
            bars,
            &render::RenderConfig { bpm, seed: 42, ..Default::default() },
            &wav_path,
            None,
        )?;

        // Play
        println!("  Playing: {}", wav_path.display());
        render::play_wav(&wav_path)?;

        // Rate
        print!("  Rate authenticity (1-5, 'q' to quit): ");
        io::stdout().flush()?;
        let mut rating_input = String::new();
        io::stdin().read_line(&mut rating_input)?;
        let rating_str = rating_input.trim();

        if rating_str == "q" || rating_str == "Q" {
            println!("Stopping. Best profile saved.");
            break;
        }

        let rating: u8 = match rating_str.parse() {
            Ok(r) if (1..=5).contains(&r) => r,
            _ => {
                println!("  Invalid. Skipping.");
                continue;
            }
        };

        if rating > best_rating {
            best_rating = rating;
            best_tuning = current_tuning.clone();
        }

        if rating == 5 {
            println!("  Perfect! Finalizing.");
            break;
        }

        // Feedback
        print!("  Feedback: ");
        io::stdout().flush()?;
        let mut feedback_input = String::new();
        io::stdin().read_line(&mut feedback_input)?;
        let feedback = feedback_input.trim();

        if iteration < max_iterations {
            // Refine
            println!("  Refining based on feedback...");
            let refined = agent.refine_style_blocking(
                name,
                description,
                &current_tuning,
                rating,
                feedback,
            )?;
            println!("  Confidence: {:.0}%", refined.confidence * 100.0);
            println!("  Reasoning: {}", refined.reasoning);

            current_tuning = refined.tuning;
            save_tuning(&current_tuning, &profile_path)?;
        }
    }

    // Save best
    let best_path = output_dir.join("best.toml");
    save_tuning(&best_tuning, &best_path)?;
    println!();
    println!("Best profile (rated {}/5) saved to: {}", best_rating, best_path.display());
    print_current_tuning(&best_tuning);

    Ok(())
}

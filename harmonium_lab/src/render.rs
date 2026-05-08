//! Headless audio rendering for style profile validation.
//!
//! Renders N bars of music from a `TuningParams` configuration to WAV audio
//! without requiring an audio device. Uses the same offline engine as the CLI
//! export command.

use std::path::Path;

use anyhow::{Context, Result};
use harmonium::{
    audio::{self, AudioBackendType},
    playback::PlaybackCommand,
};
use harmonium_core::{events::RecordFormat, tuning::TuningParams};

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;
const BUFFER_SIZE: usize = 1024;

/// Style-specific rendering parameters beyond TuningParams.
/// These control the emotional/dynamic state of the engine.
pub struct RenderConfig {
    /// BPM (70-180)
    pub bpm: f32,
    /// Density (0.0-1.0) — how many notes per beat
    pub density: f32,
    /// Tension (0.0-1.0) — harmonic complexity
    pub tension: f32,
    /// Valence (-1.0 to 1.0) — happy/sad
    pub valence: f32,
    /// Arousal (0.0-1.0) — energy level
    pub arousal: f32,
    /// Deterministic seed for reproducible output
    pub seed: u64,
    /// Rhythmic-cell variety override (0.0-1.0). `None` = use default (0.5).
    pub rhythmic_cell_variety: Option<f32>,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            density: 0.5,
            tension: 0.4,
            valence: 0.3,
            arousal: 0.5,
            seed: 42,
            rhythmic_cell_variety: None,
        }
    }
}

/// Output from headless rendering.
pub struct RenderOutput {
    /// WAV audio bytes (32-bit float PCM, stereo, 44100 Hz).
    pub wav: Vec<u8>,
    /// MIDI file bytes.
    pub midi: Option<Vec<u8>>,
    /// MusicXML score bytes.
    pub musicxml: Option<Vec<u8>>,
}

/// Render `bars` bars of music using the given `TuningParams` and `RenderConfig`.
pub fn render_to_wav(
    tuning: &TuningParams,
    bars: usize,
    config: &RenderConfig,
    sf2_bytes: Option<&[u8]>,
) -> Result<RenderOutput> {
    let (mut composer, mut playback, mut cmd_tx, mut report_rx, finished_recordings) =
        audio::create_offline_engine(sf2_bytes, AudioBackendType::FundSP, SAMPLE_RATE)
            .map_err(|e| anyhow::anyhow!("Failed to create offline engine: {}", e))?;

    // Apply tuning parameters (style personality)
    composer.set_tuning(tuning.clone());

    // Set deterministic seed
    composer.set_rng_seed(config.seed);

    // Set BPM
    composer.set_bpm(config.bpm);

    // Set rhythm mode to ClassicGroove so our groove params take effect
    composer.set_rhythm_mode(harmonium_core::sequencer::RhythmMode::ClassicGroove);

    // Set emotion params (density, tension, valence drive the generator)
    composer.set_emotions(config.arousal, config.valence, config.density, config.tension);

    // Optional variety override (CORELIB-13)
    if let Some(v) = config.rhythmic_cell_variety {
        composer.set_rhythmic_cell_variety(v);
    }

    // Sync all params to generator before generating
    composer.sync_generator();

    // Pre-generate bars
    composer.set_writehead_lookahead(bars.max(4) + 4);
    composer.generate_bars(bars);

    // Start WAV + MIDI + MusicXML recording
    let _ = cmd_tx.push(PlaybackCommand::StartRecording(RecordFormat::Wav));
    let _ = cmd_tx.push(PlaybackCommand::StartRecording(RecordFormat::Midi));
    let _ = cmd_tx.push(PlaybackCommand::StartRecording(RecordFormat::MusicXml));

    // Calculate total samples needed
    let seconds_per_bar = 4.0 * 60.0 / f64::from(config.bpm);
    let total_samples = (SAMPLE_RATE * seconds_per_bar * bars as f64) as usize;
    let mut rendered_samples = 0usize;
    let mut buffer = vec![0.0f32; BUFFER_SIZE * CHANNELS];

    while rendered_samples < total_samples {
        composer.generate_ahead();

        let remaining = total_samples - rendered_samples;
        let chunk_samples = remaining.min(BUFFER_SIZE);
        let chunk_len = chunk_samples * CHANNELS;

        for s in &mut buffer[..chunk_len] {
            *s = 0.0;
        }

        playback.process_buffer(&mut buffer[..chunk_len], CHANNELS);
        rendered_samples += chunk_samples;

        // Drain reports to avoid ring buffer backup
        while report_rx.pop().is_ok() {}
    }

    // Stop recordings
    let _ = cmd_tx.push(PlaybackCommand::StopRecording(RecordFormat::Wav));
    let _ = cmd_tx.push(PlaybackCommand::StopRecording(RecordFormat::Midi));
    let _ = cmd_tx.push(PlaybackCommand::StopRecording(RecordFormat::MusicXml));
    // Process stop commands
    let mut stop_buf = vec![0.0f32; BUFFER_SIZE * CHANNELS];
    playback.process_buffer(&mut stop_buf, CHANNELS);

    // Collect WAV + MIDI + MusicXML bytes
    let recordings =
        finished_recordings.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    let mut wav_data = None;
    let mut midi_data = None;
    let mut musicxml_data = None;
    for (format, data) in recordings.iter() {
        match format {
            RecordFormat::Wav => wav_data = Some(data.clone()),
            RecordFormat::Midi => midi_data = Some(data.clone()),
            RecordFormat::MusicXml => musicxml_data = Some(data.clone()),
        }
    }

    Ok(RenderOutput {
        wav: wav_data.ok_or_else(|| anyhow::anyhow!("No WAV data produced"))?,
        midi: midi_data,
        musicxml: musicxml_data,
    })
}

/// Render to WAV (+MIDI) and save to files.
/// MIDI is saved alongside WAV with `.mid` extension.
pub fn render_to_files(
    tuning: &TuningParams,
    bars: usize,
    config: &RenderConfig,
    wav_path: &Path,
    sf2_bytes: Option<&[u8]>,
) -> Result<()> {
    let output = render_to_wav(tuning, bars, config, sf2_bytes)?;
    std::fs::write(wav_path, &output.wav)
        .with_context(|| format!("Failed to write WAV to {}", wav_path.display()))?;

    // Save MIDI alongside WAV
    if let Some(midi) = &output.midi {
        let midi_path = wav_path.with_extension("mid");
        std::fs::write(&midi_path, midi)
            .with_context(|| format!("Failed to write MIDI to {}", midi_path.display()))?;
    }

    // Save MusicXML alongside WAV (used by lab Ingest → DNA pipeline)
    if let Some(xml) = &output.musicxml {
        let xml_path = wav_path.with_extension("musicxml");
        std::fs::write(&xml_path, xml)
            .with_context(|| format!("Failed to write MusicXML to {}", xml_path.display()))?;
    }

    Ok(())
}

/// Play a WAV file using the system's default audio player (non-blocking).
pub fn play_wav(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .context("Failed to open WAV with system player")?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .context("Failed to open WAV with system player")?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()
            .context("Failed to open WAV with system player")?;
    }
    Ok(())
}

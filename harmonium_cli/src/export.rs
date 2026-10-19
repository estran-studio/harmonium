//! Headless offline export mode for batch recording
//!
//! Drives the engine in a tight loop without any audio device, rendering
//! the requested duration as fast as the CPU allows.
//! Uses the decoupled MusicComposer + PlaybackEngine architecture.

use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use anyhow::Result;
use harmonium::{audio, playback::PlaybackCommand};
use harmonium_core::{events::RecordFormat, params::Style};

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;
const BUFFER_SIZE: usize = 256;

/// Run the engine offline for the given duration or bar count, saving recordings and snapshots.
#[allow(clippy::too_many_arguments)]
pub fn run(
    sf2_bytes: Option<&[u8]>,
    duration_secs: Option<u64>,
    target_bars: Option<usize>,
    style: Option<Style>,
    seed: Option<u64>,
    feet_density: Option<u8>,
    record_wav: Option<String>,
    record_midi: Option<String>,
    record_musicxml: Option<String>,
    record_truth: Option<String>,
) -> Result<()> {
    let (
        mut composer,
        mut playback,
        mut playback_cmd_tx,
        mut report_rx,
        _clock_reader,
        finished_recordings,
    ) = audio::create_offline_engine(sf2_bytes, SAMPLE_RATE).map_err(|e| anyhow::anyhow!(e))?;

    if let Some(s) = seed {
        composer.set_seed(s);
    }
    if let Some(st) = style {
        composer.set_style(st);
    }
    if let Some(fd) = feet_density {
        composer.set_feet_density(fd);
    }
    composer.invalidate_future();

    if let Some((bar, _)) = composer.take_pending_drop() {
        let _ = playback_cmd_tx.push(PlaybackCommand::DropFrom { bar });
    }
    for (channel, program) in composer.take_pending_program_changes() {
        let _ = playback_cmd_tx.push(PlaybackCommand::ProgramChange { channel, program });
    }
    let _ = playback_cmd_tx
        .push(PlaybackCommand::UpdateMusicalParams(Box::new(composer.musical_params().clone())));

    let initial_bars = target_bars.unwrap_or(32).max(32);
    composer.set_writehead_lookahead(initial_bars + 16);
    composer.generate_bars(initial_bars);

    if composer.musical_params().style == Style::Reel {
        let mp = composer.musical_params();
        println!(
            "Reel export: seed={}, feet={}, bpm={:.1}, swing={:.2}, sig={}/{}",
            composer.session_seed(),
            mp.feet_density,
            mp.bpm,
            mp.swing_ratio,
            mp.time_signature.numerator,
            mp.time_signature.denominator
        );
        for tune_start_bar in [1usize, 33] {
            if let Some((plan, _)) = composer.reel_bar_info(tune_start_bar) {
                println!(
                    "  Tune {} (bars {}..={}): {} {:?}",
                    (tune_start_bar - 1) / 32 + 1,
                    tune_start_bar,
                    tune_start_bar + 31,
                    harmonium_core::trad::pitch_class_name(plan.key),
                    plan.mode
                );
            }
        }
    } else {
        let mp = composer.musical_params();
        println!(
            "Legacy export: seed={}, key={} {}, bpm={:.1}, swing={:.2}, sig={}/{}",
            composer.session_seed(),
            composer.config.key,
            composer.config.scale,
            mp.bpm,
            mp.swing_ratio,
            mp.time_signature.numerator,
            mp.time_signature.denominator
        );
    }
    let has_recordings = record_wav.is_some() || record_midi.is_some() || record_musicxml.is_some();

    // Start recordings via playback command queue
    if record_wav.is_some() {
        let _ = playback_cmd_tx.push(PlaybackCommand::StartRecording(RecordFormat::Wav));
    }
    if record_midi.is_some() {
        let _ = playback_cmd_tx.push(PlaybackCommand::StartRecording(RecordFormat::Midi));
    }
    if record_musicxml.is_some() {
        let _ = playback_cmd_tx.push(PlaybackCommand::StartRecording(RecordFormat::MusicXml));
    }

    if has_recordings {
        println!("Recording started...");
    }

    let total_samples = duration_secs.map(|d| (SAMPLE_RATE * d as f64) as usize);
    let mut rendered_samples = 0usize;
    let mut buffer = vec![0.0f32; BUFFER_SIZE * CHANNELS];

    // Truth snapshot tracking
    let mut truth_snapshots: Vec<serde_json::Value> = Vec::new();
    let snapshot_interval_samples = (SAMPLE_RATE * 0.5) as usize; // every 500ms of audio
    let mut samples_since_snapshot = 0usize;
    let mut cached_report: Option<harmonium_core::EngineReport> = None;

    let wall_start = Instant::now();
    let mut last_progress_sec = 0u64;

    if let Some(bars) = target_bars {
        println!("Offline rendering {bars} measures of audio (from bar 1)...");
    } else if let Some(secs) = duration_secs {
        println!("Offline rendering {secs}s of audio...");
    }

    loop {
        if target_bars.is_some_and(|bars| composer.playhead_bar() > bars) {
            break;
        }
        if total_samples.is_some_and(|max_samples| rendered_samples >= max_samples) {
            break;
        }

        // Keep the ring buffer fed
        composer.generate_ahead();

        let chunk_samples =
            total_samples.map_or(BUFFER_SIZE, |max_s| (max_s - rendered_samples).min(BUFFER_SIZE));
        let chunk_len = chunk_samples * CHANNELS;

        // Zero the buffer slice we'll use
        for s in &mut buffer[..chunk_len] {
            *s = 0.0;
        }

        // Drive the playback engine
        playback.process_buffer(&mut buffer[..chunk_len], CHANNELS, std::time::Instant::now());
        rendered_samples += chunk_samples;

        // Poll reports
        while let Ok(report) = report_rx.pop() {
            cached_report = Some(report);
        }

        // Truth snapshots at regular audio-time intervals
        samples_since_snapshot += chunk_samples;
        if record_truth.is_some() && samples_since_snapshot >= snapshot_interval_samples {
            samples_since_snapshot = 0;
            if let Some(report) = &cached_report {
                let audio_time_ms = (rendered_samples as f64 / SAMPLE_RATE * 1000.0) as u64;
                let snapshot = serde_json::json!({
                    "timestamp_ms": audio_time_ms,
                    "bar": report.current_bar,
                    "beat": report.current_beat,
                    "step": report.current_step,
                    "chord": report.current_chord.as_str(),
                    "chord_root_offset": report.chord_root_offset,
                    "chord_is_minor": report.chord_is_minor,
                    "harmony_mode": format!("{:?}", report.harmony_mode),
                    "rhythm_mode": format!("{:?}", report.rhythm_mode),
                    "bpm": report.musical_params.bpm,
                    "time_signature": format!("{}/{}", report.time_signature.numerator, report.time_signature.denominator),
                    "primary_steps": report.primary_steps,
                    "primary_pulses": report.primary_pulses,
                    "session_key": report.session_key.as_str(),
                    "session_scale": report.session_scale.as_str(),
                });
                truth_snapshots.push(snapshot);
            }
        }

        // Progress every 5 simulated seconds
        let audio_sec = (rendered_samples as f64 / SAMPLE_RATE) as u64;
        if audio_sec >= last_progress_sec + 5 {
            last_progress_sec = audio_sec;
            let wall_elapsed = wall_start.elapsed().as_secs_f64();
            if let Some(secs) = duration_secs {
                let speedup = audio_sec as f64 / wall_elapsed;
                println!("  [{:>3}s / {}s] {:.1}x realtime", audio_sec, secs, speedup);
            }
        }
    }

    let wall_elapsed = wall_start.elapsed();
    let rendered_secs = rendered_samples as f64 / SAMPLE_RATE;
    println!(
        "Rendering complete: {:.2}s of audio in {:.2}s ({:.1}x realtime)",
        rendered_secs,
        wall_elapsed.as_secs_f64(),
        rendered_secs / wall_elapsed.as_secs_f64().max(1e-6)
    );
    // Stop recordings
    if record_wav.is_some() {
        let _ = playback_cmd_tx.push(PlaybackCommand::StopRecording(RecordFormat::Wav));
    }
    if record_midi.is_some() {
        let _ = playback_cmd_tx.push(PlaybackCommand::StopRecording(RecordFormat::Midi));
    }
    if record_musicxml.is_some() {
        let _ = playback_cmd_tx.push(PlaybackCommand::StopRecording(RecordFormat::MusicXml));
    }

    // The engine needs one more process_buffer call to handle the stop commands
    buffer.fill(0.0);
    playback.process_buffer(&mut buffer, CHANNELS, std::time::Instant::now());

    // Collect finished recordings
    save_recordings(&finished_recordings, &record_wav, &record_midi, &record_musicxml);

    // Save truth snapshots
    if let Some(truth_path) = record_truth {
        let truth_json = serde_json::to_string_pretty(&truth_snapshots)?;
        std::fs::write(&truth_path, &truth_json)?;
        println!(
            "Saved ground truth to {} ({} snapshots, {} bytes)",
            truth_path,
            truth_snapshots.len(),
            truth_json.len()
        );
    }

    println!("Export complete.");
    Ok(())
}

fn save_recordings(
    finished_recordings: &Arc<Mutex<Vec<(RecordFormat, Vec<u8>)>>>,
    record_wav: &Option<String>,
    record_midi: &Option<String>,
    record_musicxml: &Option<String>,
) {
    if let Ok(mut queue) = finished_recordings.lock() {
        while let Some((fmt, data)) = queue.pop() {
            let filename = match fmt {
                RecordFormat::Wav => record_wav.as_deref().unwrap_or("output.wav"),
                RecordFormat::Midi => record_midi.as_deref().unwrap_or("output.mid"),
                RecordFormat::MusicXml => record_musicxml.as_deref().unwrap_or("output.musicxml"),
            };

            match std::fs::write(filename, &data) {
                Ok(()) => {
                    println!("Saved {} ({} bytes)", filename, data.len());
                }
                Err(e) => {
                    eprintln!("Failed to write {}: {}", filename, e);
                }
            }
        }
    }
}

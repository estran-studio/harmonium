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
use harmonium_core::events::RecordFormat;

use crate::{
    automation::Automation,
    parser::{FeetOpt, SWING_MAX, SWING_MIN},
};

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;
const BUFFER_SIZE: usize = 256;

/// Run the engine offline for the given duration or bar count, saving recordings and snapshots.
#[allow(clippy::too_many_arguments)]
pub fn run(
    sf2_bytes: Option<&[u8]>,
    feet_sf2_bytes: Option<&[u8]>,
    odin_presets: &[harmonium_audio::odin_source::OdinPresetSource],
    duration_secs: Option<u64>,
    target_bars: Option<usize>,
    solo_track: Option<String>,
    style: Option<Box<harmonium_core::StyleProfile>>,
    seed: Option<u64>,
    feet: Option<FeetOpt>,
    swing: Option<f32>,
    tours: Option<usize>,
    automation_path: Option<String>,
    record_wav: Option<String>,
    record_midi: Option<String>,
    record_musicxml: Option<String>,
    record_truth: Option<String>,
) -> Result<()> {
    let automation = automation_path.as_deref().map(Automation::load).transpose()?;
    let (
        mut composer,
        mut playback,
        mut playback_cmd_tx,
        mut report_rx,
        mut clock_reader,
        finished_recordings,
    ) = audio::create_offline_engine(sf2_bytes, feet_sf2_bytes, odin_presets, SAMPLE_RATE)
        .map_err(|e| anyhow::anyhow!(e))?;

    if let Some(s) = seed {
        composer.set_seed(s);
    }
    if let Some(st) = style {
        composer.set_profile(*st).map_err(|e| anyhow::anyhow!(e))?;
        for missing in composer.missing_resources() {
            println!("[WARN] {missing}");
        }
    }
    // Le style réapplique ses défauts (dont le swing) : les réglages
    // d'export passent après, via les paramètres existants.
    if let Some(ratio) = swing {
        let clamped = ratio.clamp(SWING_MIN, SWING_MAX);
        if clamped != ratio {
            println!("[WARN] Swing hors plage (0.5–0.75), borné à {clamped:.2}");
        }
        composer.set_swing_ratio(clamped);
    }
    match feet {
        Some(FeetOpt::Density(fd)) => composer.set_feet_density(fd),
        Some(FeetOpt::Off) => composer.enable_rhythm(false),
        None => {}
    }
    if let Some(n) = tours {
        composer.set_tours_per_tune(n).map_err(|e| anyhow::anyhow!("--tours : {e}"))?;
    }
    // Une automation peut traverser plusieurs airs : ne pas terminer au
    // premier tour final. L'export fixe conserve son outro historique.
    if target_bars.is_some() && automation.is_none() {
        composer.set_end_after_tour(true);
    }
    let initial_emotions = [
        composer.musical_params().intensity,
        composer.musical_params().harmony_valence,
        composer.musical_params().rhythm_density,
        composer.musical_params().harmony_tension,
    ];
    if let Some(automation) = &automation {
        automation.apply(1, &mut composer, initial_emotions);
    }
    composer.invalidate_future();

    if let Some((bar, _)) = composer.take_pending_drop() {
        let _ = playback_cmd_tx.push(PlaybackCommand::DropFrom { bar });
    }
    for (channel, program) in composer.take_pending_program_changes() {
        let _ = playback_cmd_tx.push(PlaybackCommand::ProgramChange { channel, program });
    }
    let _ = playback_cmd_tx
        .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()));
    composer.send_odin(&mut playback_cmd_tx).map_err(anyhow::Error::msg)?;
    if let Some(track) = solo_track {
        for id in harmonium_core::timeline::TrackId::ALL {
            playback_cmd_tx
                .push(PlaybackCommand::SetChannelMute {
                    channel: id.channel(),
                    muted: id.name() != track,
                })
                .map_err(|_| anyhow::anyhow!("file de commandes audio pleine"))?;
        }
    }

    let initial_bars = if automation.is_some() { 1 } else { target_bars.unwrap_or(32).max(32) };
    composer.set_writehead_lookahead(initial_bars + 16);
    composer.generate_bars(initial_bars);
    let mut automated_measures = Vec::new();
    let initial_snapshots = composer.take_snapshots();
    for snapshot in &initial_snapshots {
        automated_measures.push(serde_json::json!({
            "bar": snapshot.index,
            "intensity": composer.musical_params().intensity,
            "measure": snapshot,
        }));
    }
    let mut prepared_after = 0usize;
    let mut bar_start_frames = std::collections::BTreeMap::new();
    bar_start_frames.insert(1usize, 0u64);

    {
        let mp = composer.musical_params();
        let feet = if mp.enable_rhythm { mp.feet_density.to_string() } else { "off".to_string() };
        println!(
            "Profil {} : seed={}, feet={}, tours/air={}, bpm={:.1}, swing={:.2}, sig={}/{}",
            mp.profile_id,
            composer.session_seed(),
            feet,
            composer.tours_per_tune(),
            mp.bpm,
            mp.swing_ratio,
            mp.time_signature.numerator,
            mp.time_signature.denominator
        );
        // La génération initiale est bornée par la timeline : ne lister que les
        // airs dont la première mesure existe, avec les positions publiées (intro comprise).
        let mut tunes: Vec<(harmonium_core::trad::TunePosition, usize, usize)> = Vec::new();
        for snapshot in &initial_snapshots {
            let Some(position) = snapshot.position else { continue };
            match tunes.last_mut() {
                Some((first, _, last_bar)) if first.tune_index == position.tune_index => {
                    *last_bar = snapshot.index;
                }
                _ => tunes.push((position, snapshot.index, snapshot.index)),
            }
        }
        if mp.end_after_tour {
            // L'export à durée fixe ne génère que l'air 1 (+ outro) : le reste est silence.
            tunes.truncate(1);
        }
        for (position, first_bar, last_bar) in tunes {
            let rank = if position.tunes_in_set == 0 {
                format!("{}", position.tune_in_set)
            } else {
                format!("{}/{}", position.tune_in_set, position.tunes_in_set)
            };
            println!(
                "  Tune {} (bars {}..={}): {} {:?} — set {}, air {}",
                position.tune_index + 1,
                first_bar,
                last_bar,
                harmonium_core::trad::pitch_class_name(position.key),
                position.mode,
                position.set_index + 1,
                rank
            );
        }
    }
    let has_recordings = record_wav.is_some() || record_midi.is_some() || record_musicxml.is_some();

    // Start recordings via playback command queue
    if record_wav.is_some() {
        let _ = playback_cmd_tx
            .push(PlaybackCommand::StartRecording(harmonium_core::events::AudioRecordFormat::Wav));
    }
    if record_midi.is_some() {
        let _ = playback_cmd_tx
            .push(PlaybackCommand::StartRecording(harmonium_core::events::AudioRecordFormat::Midi));
    }
    if record_musicxml.is_some() {
        composer
            .start_recording(RecordFormat::MusicXml, &mut playback_cmd_tx)
            .map_err(anyhow::Error::msg)?;
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

        if let Some(automation) = &automation {
            if rendered_samples > 0 {
                let current_bar = composer.playhead_bar();
                let has_next = target_bars.is_none_or(|last| current_bar < last);
                if has_next && current_bar != prepared_after {
                    automation.apply(current_bar + 1, &mut composer, initial_emotions);
                    composer.send_odin(&mut playback_cmd_tx).map_err(anyhow::Error::msg)?;
                    if let Some((bar, _)) = composer.take_pending_drop() {
                        playback_cmd_tx.push(PlaybackCommand::DropFrom { bar }).map_err(|_| {
                            anyhow::anyhow!("file de commandes pleine pendant l'automation")
                        })?;
                    }
                    prepared_after = current_bar;
                }
                // Exactement une mesure future ; jamais de pré-génération
                // de clés avec les paramètres de la mesure précédente.
                composer.generate_bars(if has_next { 2 } else { 1 });
                for snapshot in composer.take_snapshots() {
                    automated_measures.push(serde_json::json!({
                        "bar": snapshot.index,
                        "intensity": composer.musical_params().intensity,
                        "measure": snapshot,
                    }));
                }
            }
        } else {
            composer.generate_ahead();
            for snapshot in composer.take_snapshots() {
                automated_measures.push(serde_json::json!({
                    "bar": snapshot.index,
                    "intensity": composer.musical_params().intensity,
                    "measure": snapshot,
                }));
            }
        }

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
        let clock = clock_reader.read();
        if clock.pulse_in_bar == 0 {
            bar_start_frames.entry(clock.position.bar).or_insert(clock.pulse_start_frame);
        }

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
                    "bpm": report.musical_params.bpm,
                    "time_signature": format!("{}/{}", report.time_signature.numerator, report.time_signature.denominator),
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
        let _ = playback_cmd_tx
            .push(PlaybackCommand::StopRecording(harmonium_core::events::AudioRecordFormat::Wav));
    }
    if record_midi.is_some() {
        let _ = playback_cmd_tx
            .push(PlaybackCommand::StopRecording(harmonium_core::events::AudioRecordFormat::Midi));
    }
    if record_musicxml.is_some() {
        composer
            .stop_recording(RecordFormat::MusicXml, &mut playback_cmd_tx, &finished_recordings)
            .map_err(anyhow::Error::msg)?;
    }

    // The engine needs one more process_buffer call to handle the stop commands
    buffer.fill(0.0);
    playback.process_buffer(&mut buffer, CHANNELS, std::time::Instant::now());

    // Collect finished recordings
    save_recordings(&finished_recordings, &record_wav, &record_midi, &record_musicxml);

    // Save truth snapshots
    if let Some(truth_path) = record_truth {
        let snapshot_count = automated_measures.len();
        let truth = serde_json::json!({
            "sample_rate": SAMPLE_RATE,
            "rendered_frames": rendered_samples,
            "bar_start_frames": bar_start_frames,
            "measures": automated_measures,
            "reports": truth_snapshots,
        });
        let truth_json = serde_json::to_string_pretty(&truth)?;
        std::fs::write(&truth_path, &truth_json)?;
        println!(
            "Saved ground truth to {} ({} snapshots, {} bytes)",
            truth_path,
            snapshot_count,
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

use std::{
    env, fs,
    net::UdpSocket,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use harmonium::{audio, log, playback::PlaybackCommand};
use harmonium_core::events::RecordFormat;
use rand::Rng;
use rosc::{OscPacket, OscType};

fn perform_graceful_shutdown(
    playback_cmd_tx: &mut rtrb::Producer<PlaybackCommand>,
    finished_recordings: &harmonium::FinishedRecordings,
    composer: &std::sync::Mutex<harmonium::composer::MusicComposer>,
    record_wav: &Option<String>,
    record_midi: &Option<String>,
    record_musicxml: &Option<String>,
) -> bool {
    let expected_recordings = i32::from(record_wav.is_some())
        + i32::from(record_midi.is_some())
        + i32::from(record_musicxml.is_some());

    if expected_recordings == 0 {
        return true;
    }

    log::info(&format!("Stopping {expected_recordings} recording(s)..."));

    // Step 1: Mute all channels to stop new note generation
    for ch in 0..16u8 {
        let _ = playback_cmd_tx.push(PlaybackCommand::SetChannelMute { channel: ch, muted: true });
    }
    log::info("Muted all channels to stop new note generation");

    // Step 2: Wait for triple buffer propagation and AllNotesOff
    std::thread::sleep(Duration::from_millis(200));

    // Step 3: Wait for notes to finish playing
    log::info("Waiting for playing notes to finish...");
    std::thread::sleep(Duration::from_millis(3000));

    // Step 4: Stop recordings
    if record_wav.is_some() {
        let _ = playback_cmd_tx
            .push(PlaybackCommand::StopRecording(harmonium_core::events::AudioRecordFormat::Wav));
    }
    if record_midi.is_some() {
        let _ = playback_cmd_tx
            .push(PlaybackCommand::StopRecording(harmonium_core::events::AudioRecordFormat::Midi));
    }
    if record_musicxml.is_some() {
        if let Ok(mut composer) = composer.lock() {
            if let Err(error) = composer.stop_recording(
                RecordFormat::MusicXml,
                playback_cmd_tx,
                finished_recordings,
            ) {
                log::warn(&error);
                return false;
            }
        }
    }
    log::info("Recording stop signal sent to audio thread");

    // Step 5: Wait for backend to process stop events
    log::info("Waiting for recordings to finalize...");
    std::thread::sleep(Duration::from_millis(1500));

    // Step 6: Poll finished_recordings queue with timeout
    let mut saved_count = 0;
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 50;

    log::info("Polling for finalized recordings...");
    while iterations < MAX_ITERATIONS && saved_count < expected_recordings {
        if let Ok(mut queue) = finished_recordings.lock() {
            let queue_size = queue.len();
            if queue_size > 0 {
                log::info(&format!("Found {queue_size} recording(s) in queue"));
            }

            while let Some((fmt, data)) = queue.pop() {
                let filename = match fmt {
                    RecordFormat::Wav => record_wav.as_deref().unwrap_or("output.wav"),
                    RecordFormat::Midi => record_midi.as_deref().unwrap_or("output.mid"),
                    RecordFormat::MusicXml => {
                        record_musicxml.as_deref().unwrap_or("output.musicxml")
                    }
                };

                log::info(&format!(
                    "Saved {} to {} ({} bytes)",
                    match fmt {
                        RecordFormat::Wav => "WAV",
                        RecordFormat::Midi => "MIDI",
                        RecordFormat::MusicXml => "MusicXML",
                    },
                    filename,
                    data.len()
                ));

                if let Err(e) = fs::write(filename, &data) {
                    log::warn(&format!("Failed to write {filename}: {e}"));
                } else {
                    saved_count += 1;
                }
            }
        }

        if saved_count >= expected_recordings {
            log::info("All recordings saved successfully");
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
        iterations += 1;
    }

    if saved_count < expected_recordings {
        log::warn(&format!("Timeout: Only saved {saved_count}/{expected_recordings} recordings"));
    }

    saved_count >= expected_recordings
}

fn main() {
    log::info("Harmonium - Procedural Music Generator");

    // === 0. Parse Arguments ===
    let args: Vec<String> = env::args().collect();
    let mut sf2_path: Option<String> = None;
    let mut record_wav: Option<String> = None;
    let mut record_midi: Option<String> = None;
    let mut record_musicxml: Option<String> = None;
    let mut use_osc = false;
    let mut simulate = false;
    let mut duration_secs = 0; // 0 = infinite
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--soundfont" | "-s" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    sf2_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--record-wav" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    record_wav = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    record_wav = Some("output.wav".to_string());
                }
            }
            "--record-midi" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    record_midi = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    record_midi = Some("output.mid".to_string());
                }
            }
            "--record-musicxml" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    record_musicxml = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    record_musicxml = Some("output.musicxml".to_string());
                }
            }
            "--osc" => use_osc = true,
            "--simulate" => simulate = true,
            "--duration" => {
                if i + 1 < args.len()
                    && let Ok(d) = args[i + 1].parse::<u64>()
                {
                    duration_secs = d;
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("Usage: harmonium [OPTIONS] [SOUNDFONT.sf2]");
                println!();
                println!("Options:");
                println!(
                    "  --soundfont, -s <PATH>     SoundFont file (.sf2) (default: assets/GeneralUser-GS.sf2)"
                );
                println!("  --record-wav [PATH]        Record to WAV file (default: output.wav)");
                println!("  --record-midi [PATH]       Record to MIDI file (default: output.mid)");
                println!(
                    "  --record-musicxml [PATH]   Record to MusicXML file (default: output.musicxml)"
                );
                println!("  --osc                      Enable OSC control (UDP 8080)");
                println!("  --simulate                 Random emotion change every 5s");
                println!(
                    "  --duration <SECONDS>       Recording duration (0 = infinite, Ctrl+C to stop)"
                );
                println!("  --help, -h                 Show this help");
                println!();
                return;
            }
            arg => {
                if !arg.starts_with('-') && sf2_path.is_none() {
                    sf2_path = Some(arg.to_string());
                }
            }
        }
        i += 1;
    }

    let sf2_data = match harmonium::load_soundfont(sf2_path.as_deref()) {
        Ok(data) => data,
        Err(e) => {
            log::warn(&format!("Failed to read SoundFont: {e}"));
            None
        }
    };

    // === 0.5. Graceful Shutdown Handler ===
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_handler = shutdown_flag.clone();
    ctrlc::set_handler(move || {
        shutdown_flag_handler.store(true, Ordering::Relaxed);
    })
    .unwrap_or_else(|e| {
        #[allow(clippy::panic)]
        {
            panic!("Error setting Ctrl+C handler: {}", e);
        }
    });

    // === 1. Create Audio Stream (decoupled architecture) ===

    let (
        _stream,
        config,
        composer_mutex,
        mut playback_cmd_tx,
        _live_midi_tx,
        _report_rx,
        _font_queue,
        _transport_position,
        _clock_reader,
        finished_recordings,
    ) = audio::create_timeline_stream(sf2_data.as_deref(), None, &[]).unwrap_or_else(|e| {
        #[allow(clippy::panic)]
        {
            panic!("Failed to create audio stream: {}", e);
        }
    });

    // Wrap composer in Arc for sharing across threads
    let composer = Arc::new(composer_mutex);

    // === 2. Send Initial Configuration ===
    {
        let mut c = composer.lock().unwrap();
        c.use_emotion_mode();
        c.invalidate_future();
    }

    // Route all channels to Oxisynth when SoundFont is loaded
    if sf2_data.is_some() {
        for ch in 0..16u8 {
            let _ =
                playback_cmd_tx.push(PlaybackCommand::SetChannelRoute { channel: ch, bank_id: 0 });
        }
        log::info("Routing set to Oxisynth (Bank 0) for all channels");
    }

    // === 3. Start Recording if requested ===
    if record_wav.is_some() || record_midi.is_some() || record_musicxml.is_some() {
        if record_wav.is_some() {
            let _ = playback_cmd_tx.push(PlaybackCommand::StartRecording(
                harmonium_core::events::AudioRecordFormat::Wav,
            ));
        }
        if record_midi.is_some() {
            let _ = playback_cmd_tx.push(PlaybackCommand::StartRecording(
                harmonium_core::events::AudioRecordFormat::Midi,
            ));
        }
        if record_musicxml.is_some() {
            if let Ok(mut composer) = composer.lock() {
                if let Err(error) =
                    composer.start_recording(RecordFormat::MusicXml, &mut playback_cmd_tx)
                {
                    log::warn(&error);
                    return;
                }
            }
        }
        log::info("Recording started...");
    }

    // === 3.5. Spawn Generation Thread ===
    let gen_composer = composer.clone();
    let gen_shutdown = shutdown_flag.clone();
    thread::spawn(move || {
        loop {
            if gen_shutdown.load(Ordering::Relaxed) {
                break;
            }
            if let Ok(mut c) = gen_composer.lock() {
                c.generate_ahead();
            }
            thread::sleep(Duration::from_millis(50));
        }
    });

    // === 4. OSC Listener (opt-in) ===
    if use_osc {
        let osc_composer = composer.clone();
        let osc_shutdown = shutdown_flag.clone();

        thread::spawn(move || {
            let addr = "127.0.0.1:8080";
            let socket = match UdpSocket::bind(addr) {
                Ok(s) => {
                    log::info(&format!("OSC Listener bound to {addr}"));
                    s
                }
                Err(e) => {
                    log::error(&format!("Failed to bind OSC socket: {e}"));
                    return;
                }
            };

            let mut buf = [0u8; 4096];
            loop {
                if osc_shutdown.load(Ordering::Relaxed) {
                    break;
                }

                match socket.recv_from(&mut buf) {
                    Ok((size, _addr)) => {
                        if let Ok((_, OscPacket::Message(msg))) =
                            rosc::decoder::decode_udp(&buf[..size])
                        {
                            if msg.addr == "/harmonium/params" {
                                let args = msg.args;
                                if args.len() >= 4 {
                                    let get_float = |arg: &OscType| -> f32 {
                                        match arg {
                                            OscType::Float(f) => *f,
                                            OscType::Double(d) => *d as f32,
                                            _ => 0.0,
                                        }
                                    };

                                    let arousal = get_float(&args[0]);
                                    let valence = get_float(&args[1]);
                                    let density = get_float(&args[2]);
                                    let tension = get_float(&args[3]);

                                    if let Ok(mut c) = osc_composer.lock() {
                                        c.set_emotions(arousal, valence, density, tension);
                                        c.invalidate_future();
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error(&format!("Error receiving UDP packet: {e}"));
                    }
                }
            }
        });
    } else {
        log::info("OSC disabled. Use --osc to enable external control.");
    }

    // === 4.5. Emotion Simulator (opt-in) ===
    if simulate {
        // Simulator thread: random emotion changes every 5 seconds
        let simulator_composer = composer.clone();
        let simulator_shutdown = shutdown_flag.clone();

        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_secs(3));

            log::info("Emotion simulator started (changes every 5s)");

            loop {
                if simulator_shutdown.load(Ordering::Relaxed) {
                    log::info("Simulator thread stopping due to shutdown signal");
                    break;
                }

                // Sleep in small chunks to react to shutdown faster
                for _ in 0..50 {
                    if simulator_shutdown.load(Ordering::Relaxed) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }

                if simulator_shutdown.load(Ordering::Relaxed) {
                    log::info("Simulator thread stopping due to shutdown signal");
                    break;
                }

                let arousal = rng.gen_range(0.15..0.95);
                let valence = rng.gen_range(-0.8..0.8);
                let density = rng.gen_range(0.15..0.95);
                let tension = rng.gen_range(0.0..1.0);

                if let Ok(mut c) = simulator_composer.lock() {
                    if simulator_shutdown.load(Ordering::Relaxed) {
                        break;
                    }
                    c.set_emotions(arousal, valence, density, tension);
                    c.invalidate_future();
                    let bpm = c.musical_params().bpm;
                    log::info(&format!(
                        "EMOTION CHANGE: Arousal {arousal:.2} (-> {bpm:.0} BPM) | Valence {valence:.2} | Density {density:.2} | Tension {tension:.2}"
                    ));
                }
            }
        });
    }

    // === 5. Main Loop ===
    log::info(&format!(
        "Session: {} {} | BPM: {:.1} | Pulses: {}/{}",
        config.key, config.scale, config.bpm, config.pulses, config.steps
    ));
    log::info("Playing... Press Ctrl+C to stop.");

    let start_time = std::time::Instant::now();

    loop {
        std::thread::sleep(Duration::from_millis(100));

        // Check for Ctrl+C signal
        if shutdown_flag.load(Ordering::Relaxed) {
            log::info("Received interrupt signal, saving recordings...");
            let success = perform_graceful_shutdown(
                &mut playback_cmd_tx,
                &finished_recordings,
                &composer,
                &record_wav,
                &record_midi,
                &record_musicxml,
            );
            if !success {
                log::warn("Some recordings may not have been saved");
            }
            log::info("Exited gracefully.");
            break;
        }

        // Duration-based recording stop
        if duration_secs > 0 && start_time.elapsed().as_secs() >= duration_secs {
            log::info("Duration reached. Stopping recording...");
            let success = perform_graceful_shutdown(
                &mut playback_cmd_tx,
                &finished_recordings,
                &composer,
                &record_wav,
                &record_midi,
                &record_musicxml,
            );
            if !success {
                log::warn("Some recordings may not have been saved");
            }
            log::info("Exiting after recording.");
            break;
        }

        // Check for finished recordings
        if !shutdown_flag.load(Ordering::Relaxed)
            && let Ok(mut queue) = finished_recordings.lock()
        {
            while let Some((fmt, data)) = queue.pop() {
                let filename = match fmt {
                    RecordFormat::Wav => record_wav.as_deref().unwrap_or("output.wav"),
                    RecordFormat::Midi => record_midi.as_deref().unwrap_or("output.mid"),
                    RecordFormat::MusicXml => {
                        record_musicxml.as_deref().unwrap_or("output.musicxml")
                    }
                };
                log::info(&format!("Saving recording to {} ({} bytes)", filename, data.len()));
                if let Err(e) = fs::write(filename, &data) {
                    log::warn(&format!("Failed to write file: {e}"));
                }
            }
        }
    }
}

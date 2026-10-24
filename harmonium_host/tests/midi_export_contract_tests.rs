//! MIDI export contract (P1-T10): the recorded MIDI describes exactly what
//! the WAV plays.
//!
//! Drives the real offline engine in reel style and parses the produced
//! bytes with `midly`:
//! - every `NoteOn` pairs with a `NoteOff` on the same channel and pitch,
//!   in order (no dangling or orphan ends);
//! - the file tempo is the real tempo (230 BPM reel) with a 2/2 signature,
//!   and the MIDI duration matches the rendered audio within one beat;
//! - `--feet off` (`enable_rhythm(false)`) records no drum-channel notes;
//! - `--swing 0.5` yields equal eighth-note intervals.

use std::time::Instant;

use harmonium::{audio::create_offline_engine, playback::PlaybackCommand};
use harmonium_core::events::RecordFormat;
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;
const BUFFER_SIZE: usize = 256;
const TICKS_PER_QUARTER: u32 = 480;
const REEL_BPM: f32 = 230.0;

struct Export {
    midi: Vec<u8>,
    rendered_secs: f64,
}

/// Render `bars` reel bars offline (silent synth) with the given export
/// settings, mirroring `harmonium_cli::export::run`, and return the recorded
/// MIDI bytes plus the rendered audio duration.
fn render_reel(
    seed: u64,
    bars: usize,
    swing: Option<f32>,
    feet_off: bool,
    mechanical: bool,
) -> Result<Export, String> {
    let (mut composer, mut playback, mut cmd_tx, mut report_rx, _clock, finished) =
        create_offline_engine(None, None, SAMPLE_RATE)?;
    if mechanical {
        let mut profile = composer.profile().clone();
        profile.performance = Default::default();
        profile.ornaments = Default::default();
        composer.set_profile(profile)?;
    }

    composer.set_seed(seed);

    if let Some(ratio) = swing {
        composer.set_swing_ratio(ratio);
    }
    if feet_off {
        composer.enable_rhythm(false);
    }
    composer.invalidate_future();

    if let Some((bar, _)) = composer.take_pending_drop() {
        let _ = cmd_tx.push(PlaybackCommand::DropFrom { bar });
    }
    for (channel, program) in composer.take_pending_program_changes() {
        let _ = cmd_tx.push(PlaybackCommand::ProgramChange { channel, program });
    }
    let _ = cmd_tx
        .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()));

    composer.set_writehead_lookahead(bars + 16);
    composer.generate_bars(bars);

    let _ = cmd_tx
        .push(PlaybackCommand::StartRecording(harmonium_core::events::AudioRecordFormat::Midi));

    let mut rendered_samples = 0usize;
    let mut buffer = vec![0.0f32; BUFFER_SIZE * CHANNELS];
    loop {
        if composer.playhead_bar() > bars {
            break;
        }
        composer.generate_ahead();
        buffer.fill(0.0);
        playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
        rendered_samples += BUFFER_SIZE;
        while report_rx.pop().is_ok() {}
    }

    let _ = cmd_tx
        .push(PlaybackCommand::StopRecording(harmonium_core::events::AudioRecordFormat::Midi));
    buffer.fill(0.0);
    playback.process_buffer(&mut buffer, CHANNELS, Instant::now());

    let queue = finished.lock().map_err(|e| e.to_string())?;
    let midi = queue
        .iter()
        .find(|(fmt, _)| *fmt == RecordFormat::Midi)
        .map(|(_, data)| data.clone())
        .ok_or("one MIDI recording")?;
    Ok(Export { midi, rendered_secs: rendered_samples as f64 / SAMPLE_RATE })
}

/// Absolute-tick note events of a single-track recording: `(tick, on, channel, pitch)`.
fn note_events(smf: &Smf) -> Result<Vec<(u32, bool, u8, u8)>, String> {
    if smf.tracks.len() != 1 {
        return Err("single-track recording".to_string());
    }
    let mut out = Vec::new();
    let mut tick = 0u32;
    for e in &smf.tracks[0] {
        tick += e.delta.as_int();
        match e.kind {
            TrackEventKind::Midi { channel, message: MidiMessage::NoteOn { key, vel } }
                if vel.as_int() > 0 =>
            {
                out.push((tick, true, channel.as_int(), key.as_int()))
            }
            TrackEventKind::Midi { channel, message: MidiMessage::NoteOff { key, .. } } => {
                out.push((tick, false, channel.as_int(), key.as_int()))
            }
            _ => {}
        }
    }
    Ok(out)
}

fn tempos(smf: &Smf) -> Vec<u32> {
    smf.tracks[0]
        .iter()
        .filter_map(|e| match e.kind {
            TrackEventKind::Meta(MetaMessage::Tempo(t)) => Some(t.as_int()),
            _ => None,
        })
        .collect()
}

/// Every NoteOn closes with a NoteOff on the same channel and pitch, in
/// order: simulate the sounding set, which must drain exactly.
#[test]
fn midi_noteons_all_have_noteoffs() -> Result<(), String> {
    let export = render_reel(1, 8, None, false, false)?;
    let smf = Smf::parse(&export.midi).map_err(|e| e.to_string())?;
    let events = note_events(&smf)?;
    assert!(!events.is_empty(), "reel records notes");

    let mut sounding: Vec<(u8, u8)> = Vec::new();
    for (tick, on, ch, pitch) in &events {
        if *on {
            assert!(
                !sounding.contains(&(*ch, *pitch)),
                "double NoteOn ch {ch} pitch {pitch} at {tick}"
            );
            sounding.push((*ch, *pitch));
        } else if let Some(pos) = sounding.iter().position(|&(c, p)| c == *ch && p == *pitch) {
            sounding.swap_remove(pos);
        } else {
            return Err(format!("orphan NoteOff ch {ch} pitch {pitch} at {tick}"));
        }
    }
    assert!(sounding.is_empty(), "dangling NoteOns: {sounding:?}");

    let (ons, offs) = events
        .iter()
        .fold((0, 0), |(ons, offs), e| if e.1 { (ons + 1, offs) } else { (ons, offs + 1) });
    assert_eq!(ons, offs, "on/off counts differ");
    Ok(())
}

/// Tempo meta is the real tempo, the signature is 2/2, and the MIDI
/// duration matches the rendered WAV within one beat.
#[test]
fn midi_tempo_signature_and_duration_match_audio() -> Result<(), String> {
    let export = render_reel(1, 8, None, false, false)?;
    let smf = Smf::parse(&export.midi).map_err(|e| e.to_string())?;

    let expected_uspq = (60_000_000.0 / REEL_BPM) as u32;
    assert_eq!(tempos(&smf), vec![expected_uspq], "single real tempo meta");

    let sigs: Vec<(u8, u8)> = smf.tracks[0]
        .iter()
        .filter_map(|e| match e.kind {
            TrackEventKind::Meta(MetaMessage::TimeSignature(n, d, _, _)) => Some((n, d)),
            _ => None,
        })
        .collect();
    assert_eq!(sigs, vec![(2, 1)], "2/2 signature: {sigs:?}");

    let events = note_events(&smf)?;
    let last_tick = events.iter().map(|e| e.0).max().ok_or("notes")?;
    let midi_secs = last_tick as f64 / TICKS_PER_QUARTER as f64 * (60.0 / REEL_BPM as f64);
    let beat = 60.0 / REEL_BPM as f64;
    assert!(
        (midi_secs - export.rendered_secs).abs() <= beat,
        "MIDI {midi_secs:.3}s vs audio {:.3}s (±1 beat {beat:.3}s)",
        export.rendered_secs
    );
    Ok(())
}

/// Feet off: no notes on the GM drum channel (9).
#[test]
fn midi_feet_off_records_no_drums() -> Result<(), String> {
    let export = render_reel(1, 8, None, true, false)?;
    let smf = Smf::parse(&export.midi).map_err(|e| e.to_string())?;
    let events = note_events(&smf)?;
    let drum: Vec<_> = events.iter().filter(|e| e.2 == 9).collect();
    assert!(drum.is_empty(), "drum notes with feet off: {drum:?}");
    // The other tracks still play.
    assert!(!events.is_empty(), "melody/harmony must remain");
    Ok(())
}

/// Straight swing: consecutive melody NoteOn gaps are whole eighth-note
/// multiples; the default reel swing bends eighths off that grid.
#[test]
fn midi_straight_swing_gives_equal_eighths() -> Result<(), String> {
    let export = render_reel(1, 8, Some(0.5), false, true)?;
    let smf = Smf::parse(&export.midi).map_err(|e| e.to_string())?;
    let melody: Vec<u32> =
        note_events(&smf)?.into_iter().filter(|e| e.1 && e.2 == 0).map(|e| e.0).collect();
    assert!(melody.len() > 8, "enough melody notes");
    for pair in melody.windows(2) {
        let gap = pair[1] - pair[0];
        assert!(gap > 0 && gap % 240 == 0, "unequal eighth interval: {gap} in {melody:?}");
    }

    let export = render_reel(1, 8, None, false, false)?;
    let smf = Smf::parse(&export.midi).map_err(|e| e.to_string())?;
    let all: Vec<u32> = note_events(&smf)?.into_iter().filter(|e| e.1).map(|e| e.0).collect();
    let bent = all.windows(2).any(|pair| (pair[1] - pair[0]) % 240 != 0);
    assert!(bent, "default reel swing should bend eighth intervals");
    Ok(())
}

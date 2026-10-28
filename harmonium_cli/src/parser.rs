//! Command parser for Harmonium CLI
//!
//! Parses user input into `EngineCommand` variants: the REPL keeps the text,
//! the host (`LiveHost::apply`) executes the command.

use anyhow::{anyhow, Result};
use harmonium_core::{events::RecordFormat, EngineCommand};

/// Swing bounds (phase 0, D3): 0.5 = straight, 0.75 = hard swing.
pub const SWING_MIN: f32 = 0.5;
/// Swing bounds (phase 0, D3): 0.5 = straight, 0.75 = hard swing.
pub const SWING_MAX: f32 = 0.75;

/// Parse a command line into an EngineCommand.
///
/// Commands that are not engine commands (`help`, `quit`, `state`, `beat`,
/// bare `stop`) belong to the REPL and are answered before this parser.
pub fn parse_command(line: &str) -> Result<EngineCommand> {
    let line = line.trim();

    // Empty line
    if line.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    // Split into tokens
    let tokens: Vec<&str> = line.split_whitespace().collect();

    match tokens[0] {
        // === GLOBAL ===
        "set" => parse_set_command(&tokens[1..]),
        "reset" => {
            if tokens.get(1).copied() == Some("bpm") {
                Ok(EngineCommand::ResetBpm)
            } else {
                Ok(EngineCommand::Reset)
            }
        }

        // === CONTROL MODE ===
        "emotion" => parse_emotion_command(&tokens[1..]),
        "direct" => parse_direct_command(&tokens[1..]),

        // === RECORDING ===
        "record" => parse_record_command(&tokens[1..]),
        "stop" => {
            let format = tokens.get(1).ok_or_else(|| anyhow!("Usage: stop [wav|midi|musicxml]"))?;
            Ok(EngineCommand::StopRecording(parse_record_format(format)?))
        }

        // === SESSION ===
        "seed" => parse_seed_command(&tokens[1..]),

        // === TIMELINE ===
        "seek" => parse_seek_command(&tokens[1..]),
        "loop" => parse_loop_command(&tokens[1..]),

        // === TEMPO, GROOVE AND STYLE ===
        "swing" => parse_swing_command(&tokens[1..]),
        "tempo" => parse_tempo_command(&tokens[1..]),
        "style" => parse_style_command(&tokens[1..]),
        "feet" => parse_feet_command(&tokens[1..]),
        "intensity" => parse_intensity_command(&tokens[1..]),

        _ => Err(anyhow!("Unknown command: {}. Type 'help' for available commands.", tokens[0])),
    }
}

/// Parse "set <param> <value>" commands
fn parse_set_command(tokens: &[&str]) -> Result<EngineCommand> {
    if tokens.is_empty() {
        return Err(anyhow!("Usage: set <param> <value>"));
    }

    let param = tokens[0];
    let value = tokens.get(1).ok_or_else(|| anyhow!("Missing value for '{}'", param))?;

    match param {
        // === GLOBAL ===
        "bpm" => {
            let bpm = value.parse::<f32>().map_err(|_| anyhow!("Invalid BPM value: {}", value))?;
            Ok(EngineCommand::SetBpm(bpm))
        }

        "volume" | "master_volume" => {
            let volume =
                value.parse::<f32>().map_err(|_| anyhow!("Invalid volume value: {}", value))?;
            Ok(EngineCommand::SetMasterVolume(volume))
        }

        "time" | "time_signature" => {
            let parts: Vec<&str> = value.split('/').collect();
            if parts.len() != 2 {
                return Err(anyhow!("Invalid time signature format. Use: 4/4"));
            }
            let numerator = parts[0]
                .parse::<usize>()
                .map_err(|_| anyhow!("Invalid numerator: {}", parts[0]))?;
            let denominator = parts[1]
                .parse::<usize>()
                .map_err(|_| anyhow!("Invalid denominator: {}", parts[1]))?;
            Ok(EngineCommand::SetTimeSignature { numerator, denominator })
        }

        // === MODULES ===
        "rhythm" => Ok(EngineCommand::EnableRhythm(parse_on_off(value)?)),
        "harmony" => Ok(EngineCommand::EnableHarmony(parse_on_off(value)?)),
        "melody" => Ok(EngineCommand::EnableMelody(parse_on_off(value)?)),

        // === MIXER ===
        "gain" => {
            let channel = tokens.get(1).ok_or_else(|| anyhow!("Missing channel number"))?;
            let gain_value = tokens.get(2).ok_or_else(|| anyhow!("Missing gain value"))?;
            let ch = channel.parse::<u8>().map_err(|_| anyhow!("Invalid channel: {}", channel))?;
            let g =
                gain_value.parse::<f32>().map_err(|_| anyhow!("Invalid gain: {}", gain_value))?;
            Ok(EngineCommand::SetChannelGain { channel: ch, gain: g })
        }

        "mute" => {
            let channel = value.parse::<u8>().map_err(|_| anyhow!("Invalid channel: {}", value))?;
            Ok(EngineCommand::SetChannelMute { channel, muted: true })
        }

        "unmute" => {
            let channel = value.parse::<u8>().map_err(|_| anyhow!("Invalid channel: {}", value))?;
            Ok(EngineCommand::SetChannelMute { channel, muted: false })
        }

        "route" => {
            let bank = tokens.get(2).ok_or_else(|| anyhow!("Usage: set route <channel> <bank>"))?;
            let channel = value.parse::<u8>().map_err(|_| anyhow!("Invalid channel: {}", value))?;
            let bank_id = bank.parse::<i32>().map_err(|_| anyhow!("Invalid bank: {}", bank))?;
            Ok(EngineCommand::SetChannelRoute { channel, bank_id })
        }

        "output" => match value.to_lowercase().as_str() {
            "mute" => Ok(EngineCommand::SetOutputMute(true)),
            "unmute" => Ok(EngineCommand::SetOutputMute(false)),
            _ => Err(anyhow!("Usage: set output <mute|unmute>")),
        },

        // === SESSION ===
        "seed" => {
            let seed =
                value.parse::<u64>().map_err(|_| anyhow!("Invalid seed value: {}", value))?;
            Ok(EngineCommand::SetSeed(seed))
        }

        "lookahead" => {
            let bars =
                value.parse::<usize>().map_err(|_| anyhow!("Invalid lookahead: {}", value))?;
            Ok(EngineCommand::SetWriteheadLookahead(bars))
        }

        _ => {
            Err(anyhow!("Unknown parameter: {}. Type 'help set' for available parameters.", param))
        }
    }
}

/// Parse an `on|off` value (also `true|false`).
fn parse_on_off(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "on" | "true" => Ok(true),
        "off" | "false" => Ok(false),
        _ => Err(anyhow!("Invalid value: {value}. Use: on, off")),
    }
}

/// Parse "emotion <arousal> <valence> <density> <tension>" command
fn parse_emotion_command(tokens: &[&str]) -> Result<EngineCommand> {
    if tokens.is_empty() {
        // Switch to emotion mode
        return Ok(EngineCommand::UseEmotionMode);
    }

    // Set emotion parameters
    if tokens.len() < 4 {
        return Err(anyhow!("Usage: emotion <arousal> <valence> <density> <tension>"));
    }

    let arousal =
        tokens[0].parse::<f32>().map_err(|_| anyhow!("Invalid arousal: {}", tokens[0]))?;
    let valence =
        tokens[1].parse::<f32>().map_err(|_| anyhow!("Invalid valence: {}", tokens[1]))?;
    let density =
        tokens[2].parse::<f32>().map_err(|_| anyhow!("Invalid density: {}", tokens[2]))?;
    let tension =
        tokens[3].parse::<f32>().map_err(|_| anyhow!("Invalid tension: {}", tokens[3]))?;

    Ok(EngineCommand::SetEmotionParams { arousal, valence, density, tension })
}

/// Parse "direct" command (switch to direct mode)
fn parse_direct_command(_tokens: &[&str]) -> Result<EngineCommand> {
    Ok(EngineCommand::UseDirectMode)
}

/// Parse a recording format name (`wav`, `midi`, `musicxml`).
fn parse_record_format(raw: &str) -> Result<RecordFormat> {
    match raw.to_lowercase().as_str() {
        "wav" => Ok(RecordFormat::Wav),
        "midi" | "mid" => Ok(RecordFormat::Midi),
        "musicxml" | "xml" => Ok(RecordFormat::MusicXml),
        _ => Err(anyhow!("Unknown format: {raw}. Use: wav, midi, musicxml")),
    }
}

/// Parse "record <wav|midi|musicxml>" command
/// Note: filename is not supported yet in the command interface
fn parse_record_command(tokens: &[&str]) -> Result<EngineCommand> {
    if tokens.is_empty() {
        return Err(anyhow!("Usage: record <wav|midi|musicxml>"));
    }
    Ok(EngineCommand::StartRecording(parse_record_format(tokens[0])?))
}

/// Parse `seed <n>` (explicit seed) or `seed new` (fresh random seed).
fn parse_seed_command(tokens: &[&str]) -> Result<EngineCommand> {
    let val = tokens.first().ok_or_else(|| anyhow!("Usage: seed <u64> | seed new"))?;
    if val.eq_ignore_ascii_case("new") {
        return Ok(EngineCommand::NewMelody);
    }
    let seed = val.parse::<u64>().map_err(|_| anyhow!("Invalid seed value: {val}"))?;
    Ok(EngineCommand::SetSeed(seed))
}

/// Parse `seek <bar>` (deterministic) or `seek <bar> soft` (playhead only).
fn parse_seek_command(tokens: &[&str]) -> Result<EngineCommand> {
    let usage = || anyhow!("Usage: seek <bar_number> [soft]");
    let bar = tokens.first().and_then(|t| t.parse::<usize>().ok()).ok_or_else(usage)?;
    match tokens.get(1).copied() {
        None => Ok(EngineCommand::Seek(bar)),
        Some("soft") if tokens.len() == 2 => Ok(EngineCommand::SeekPlayhead(bar)),
        Some(_) => Err(usage()),
    }
}

/// Parse `loop <start> <end>` or `loop off`.
fn parse_loop_command(tokens: &[&str]) -> Result<EngineCommand> {
    if tokens.len() > 1 {
        if let (Ok(start_bar), Ok(end_bar)) =
            (tokens[0].parse::<usize>(), tokens[1].parse::<usize>())
        {
            return Ok(EngineCommand::SetLoop { start_bar, end_bar });
        }
        return Err(anyhow!("Usage: loop <start_bar> <end_bar>"));
    }
    if tokens.first().copied() == Some("off") {
        return Ok(EngineCommand::ClearLoop);
    }
    Err(anyhow!("Usage: loop <start> <end> | loop off"))
}

/// Sélecteur partagé par le démarrage CLI et la commande interactive `style`.
pub fn parse_style_value(raw: &str) -> Result<Box<harmonium_core::StyleProfile>> {
    let profile = if std::path::Path::new(raw).exists() {
        harmonium_core::StyleProfile::load(raw)?
    } else {
        harmonium_core::StyleProfile::named(raw)?
    };
    Ok(Box::new(profile))
}

fn parse_style_command(tokens: &[&str]) -> Result<EngineCommand> {
    if tokens.len() != 1 {
        return Err(anyhow!("Usage: style <identifiant|dossier>"));
    }
    Ok(EngineCommand::SetStyleProfile(parse_style_value(tokens[0])?))
}

/// Parse `feet <0|1|2>`.
fn parse_feet_command(tokens: &[&str]) -> Result<EngineCommand> {
    if tokens.len() != 1 {
        return Err(anyhow!("Usage: feet <0|1|2>"));
    }
    let density: u8 = tokens[0]
        .parse()
        .map_err(|_| anyhow!("Densité invalide: {}. Utiliser: feet 0|1|2", tokens[0]))?;
    if density > 2 {
        return Err(anyhow!("Densité hors plage ({density}). Utiliser: feet 0|1|2"));
    }
    Ok(EngineCommand::SetFeetDensity(density))
}

/// Parse `intensity <0..1>`.
fn parse_intensity_command(tokens: &[&str]) -> Result<EngineCommand> {
    if tokens.len() != 1 {
        return Err(anyhow!("Usage: intensity <0..1>"));
    }
    let value: f32 = tokens[0].parse().map_err(|_| anyhow!("Intensité invalide: {}", tokens[0]))?;
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(anyhow!("Intensité invalide: {} (attendu 0..1)", tokens[0]));
    }
    Ok(EngineCommand::SetIntensity(value))
}

/// Export `--feet` value: a podorythmie density (`0|1|2`) or `off`, which
/// drops the feet track (same as disabling the rhythm module for the export).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeetOpt {
    Density(u8),
    Off,
}

/// Parse an export `--feet` value (`0|1|2|off`, case-insensitive).
pub fn parse_feet_value(raw: &str) -> Result<FeetOpt> {
    if raw.eq_ignore_ascii_case("off") {
        return Ok(FeetOpt::Off);
    }
    let density: u8 =
        raw.parse().map_err(|_| anyhow!("Densité invalide: {raw}. Utiliser: --feet 0|1|2|off"))?;
    if density > 2 {
        return Err(anyhow!("Densité hors plage ({density}). Utiliser: --feet 0|1|2|off"));
    }
    Ok(FeetOpt::Density(density))
}

/// Parse `swing <ratio>`: a finite positive number. The host bounds it to
/// `SWING_MIN..=SWING_MAX`; the REPL tells the user when it did.
fn parse_swing_command(tokens: &[&str]) -> Result<EngineCommand> {
    if tokens.len() != 1 {
        return Err(anyhow!("Usage: swing <0.5-0.75>"));
    }
    let ratio: f32 =
        tokens[0].parse().map_err(|_| anyhow!("Invalid swing ratio: {}", tokens[0]))?;
    if !ratio.is_finite() || ratio <= 0.0 {
        return Err(anyhow!("Invalid swing ratio: {}", tokens[0]));
    }
    Ok(EngineCommand::SetSwing(ratio))
}

/// Parse `tempo <bpm> [pulses]` or `tempo off`. The engine clamps the BPM to
/// its 40–300 range; `pulses` is a `u8` ramp length (`0` = jump).
fn parse_tempo_command(tokens: &[&str]) -> Result<EngineCommand> {
    if tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("off") {
        return Ok(EngineCommand::ClearAudibleTempo);
    }
    if tokens.len() != 1 && tokens.len() != 2 {
        return Err(anyhow!("Usage: tempo <bpm> [pulses] | tempo off"));
    }
    let bpm: f32 = tokens[0].parse().map_err(|_| anyhow!("Invalid tempo: {}", tokens[0]))?;
    if !bpm.is_finite() || bpm <= 0.0 {
        return Err(anyhow!("Invalid tempo: {}", tokens[0]));
    }
    let ramp_pulses = if tokens.len() == 2 {
        tokens[1].parse().map_err(|_| anyhow!("Invalid pulse count: {}", tokens[1]))?
    } else {
        0u8
    };
    Ok(EngineCommand::SetAudibleTempo { bpm, ramp_pulses })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_set_bpm() {
        let cmd = parse_command("set bpm 140").unwrap();
        assert!(matches!(cmd, EngineCommand::SetBpm(140.0)));
    }

    #[test]
    fn test_parse_emotion() {
        let cmd = parse_command("emotion 0.9 0.5 0.7 0.6").unwrap();
        assert!(matches!(cmd, EngineCommand::SetEmotionParams { .. }));
    }

    #[test]
    fn test_parse_reset_bpm() {
        let cmd = parse_command("reset bpm").unwrap();
        assert!(matches!(cmd, EngineCommand::ResetBpm));
    }

    #[test]
    fn test_parse_reset_engine() {
        let cmd = parse_command("reset").unwrap();
        assert!(matches!(cmd, EngineCommand::Reset));
    }

    #[test]
    fn test_invalid_command() {
        let result = parse_command("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_swing() {
        assert_eq!(parse_command("swing 0.6").ok(), Some(EngineCommand::SetSwing(0.6)));
        assert_eq!(parse_command("swing 0.5").ok(), Some(EngineCommand::SetSwing(0.5)));
    }

    #[test]
    fn test_parse_swing_keeps_out_of_range_for_the_host_to_bound() {
        assert_eq!(parse_command("swing 0.9").ok(), Some(EngineCommand::SetSwing(0.9)));
        assert_eq!(parse_command("swing 0.1").ok(), Some(EngineCommand::SetSwing(0.1)));
    }

    #[test]
    fn test_parse_swing_invalid() {
        assert!(parse_command("swing").is_err());
        assert!(parse_command("swing 0.6 0.7").is_err());
        assert!(parse_command("swing lourd").is_err());
        assert!(parse_command("swing -1.0").is_err());
        assert!(parse_command("swing NaN").is_err());
    }

    #[test]
    fn test_parse_tempo() {
        assert_eq!(
            parse_command("tempo 200").ok(),
            Some(EngineCommand::SetAudibleTempo { bpm: 200.0, ramp_pulses: 0 })
        );
        assert_eq!(
            parse_command("tempo 200 8").ok(),
            Some(EngineCommand::SetAudibleTempo { bpm: 200.0, ramp_pulses: 8 })
        );
    }

    #[test]
    fn test_parse_tempo_off() {
        assert_eq!(parse_command("tempo off").ok(), Some(EngineCommand::ClearAudibleTempo));
        assert_eq!(parse_command("tempo OFF").ok(), Some(EngineCommand::ClearAudibleTempo));
    }

    #[test]
    fn test_parse_tempo_invalid() {
        assert!(parse_command("tempo").is_err());
        assert!(parse_command("tempo vite").is_err());
        assert!(parse_command("tempo -40").is_err());
        assert!(parse_command("tempo 200 huit").is_err());
        assert!(parse_command("tempo 200 300").is_err());
        assert!(parse_command("tempo 200 8 x").is_err());
    }

    #[test]
    fn test_parse_feet() {
        for d in 0..=2u8 {
            assert_eq!(
                parse_command(&format!("feet {d}")).ok(),
                Some(EngineCommand::SetFeetDensity(d))
            );
        }
        assert!(parse_command("feet").is_err());
        assert!(parse_command("feet 3").is_err());
        assert!(parse_command("feet dense").is_err());
    }

    #[test]
    fn test_parse_intensity() {
        assert_eq!(parse_command("intensity 0.7").ok(), Some(EngineCommand::SetIntensity(0.7)));
        assert!(parse_command("intensity").is_err());
        assert!(parse_command("intensity 1.5").is_err());
        assert!(parse_command("intensity NaN").is_err());
    }

    #[test]
    fn test_parse_seek_and_loop() {
        assert_eq!(parse_command("seek 5").ok(), Some(EngineCommand::Seek(5)));
        assert_eq!(parse_command("seek 5 soft").ok(), Some(EngineCommand::SeekPlayhead(5)));
        assert!(parse_command("seek").is_err());
        assert!(parse_command("seek x").is_err());
        assert!(parse_command("seek 5 hard").is_err());
        assert_eq!(
            parse_command("loop 4 8").ok(),
            Some(EngineCommand::SetLoop { start_bar: 4, end_bar: 8 })
        );
        assert_eq!(parse_command("loop off").ok(), Some(EngineCommand::ClearLoop));
        assert!(parse_command("loop").is_err());
        assert!(parse_command("loop 4 x").is_err());
    }

    #[test]
    fn test_parse_seed() {
        assert_eq!(parse_command("seed 42").ok(), Some(EngineCommand::SetSeed(42)));
        assert_eq!(parse_command("seed new").ok(), Some(EngineCommand::NewMelody));
        assert!(parse_command("seed").is_err());
        assert!(parse_command("seed x").is_err());
    }

    #[test]
    fn test_parse_recording() {
        assert_eq!(
            parse_command("record midi").ok(),
            Some(EngineCommand::StartRecording(RecordFormat::Midi))
        );
        assert_eq!(
            parse_command("stop wav").ok(),
            Some(EngineCommand::StopRecording(RecordFormat::Wav))
        );
        assert!(parse_command("record flac").is_err());
        assert!(parse_command("stop").is_err());
    }

    #[test]
    fn test_parse_modules_route_output_lookahead() {
        assert_eq!(parse_command("set rhythm off").ok(), Some(EngineCommand::EnableRhythm(false)));
        assert_eq!(parse_command("set harmony on").ok(), Some(EngineCommand::EnableHarmony(true)));
        assert_eq!(parse_command("set melody OFF").ok(), Some(EngineCommand::EnableMelody(false)));
        assert!(parse_command("set melody maybe").is_err());
        assert_eq!(
            parse_command("set route 2 -1").ok(),
            Some(EngineCommand::SetChannelRoute { channel: 2, bank_id: -1 })
        );
        assert!(parse_command("set route 2").is_err());
        assert_eq!(parse_command("set output mute").ok(), Some(EngineCommand::SetOutputMute(true)));
        assert_eq!(
            parse_command("set output unmute").ok(),
            Some(EngineCommand::SetOutputMute(false))
        );
        assert!(parse_command("set output half").is_err());
        assert_eq!(
            parse_command("set lookahead 8").ok(),
            Some(EngineCommand::SetWriteheadLookahead(8))
        );
        assert!(parse_command("set lookahead x").is_err());
    }

    #[test]
    fn test_repl_only_words_are_not_engine_commands() {
        for word in ["help", "quit", "state", "beat", "stop"] {
            assert!(parse_command(word).is_err(), "{word}");
        }
    }

    #[test]
    fn test_parse_feet_value() {
        assert_eq!(parse_feet_value("0").ok(), Some(FeetOpt::Density(0)));
        assert_eq!(parse_feet_value("1").ok(), Some(FeetOpt::Density(1)));
        assert_eq!(parse_feet_value("2").ok(), Some(FeetOpt::Density(2)));
        assert_eq!(parse_feet_value("off").ok(), Some(FeetOpt::Off));
        assert_eq!(parse_feet_value("OFF").ok(), Some(FeetOpt::Off));
        assert!(parse_feet_value("3").is_err());
        assert!(parse_feet_value("dense").is_err());
        assert!(parse_feet_value("").is_err());
    }
}

//! Command parser for Harmonium CLI
//!
//! Parses user input into EngineCommand variants

use anyhow::{anyhow, Result};
use harmonium_core::EngineCommand;

/// Parse a command line into an EngineCommand
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

        // === CONTROL MODE ===
        "emotion" => parse_emotion_command(&tokens[1..]),
        "direct" => parse_direct_command(&tokens[1..]),

        // === RECORDING ===
        "record" => parse_record_command(&tokens[1..]),

        // === UTILITY ===
        "state" | "show" | "status" => Ok(EngineCommand::GetState),
        "reset" => {
            if tokens.get(1).copied() == Some("bpm") {
                Ok(EngineCommand::ResetBpm)
            } else {
                Ok(EngineCommand::Reset)
            }
        }

        "seed" => {
            let val = tokens.get(1).ok_or_else(|| anyhow!("Usage: seed <u64>"))?;
            let seed = val.parse::<u64>().map_err(|_| anyhow!("Invalid seed value: {val}"))?;
            Ok(EngineCommand::SetSeed(seed))
        }
        // === HELP ===
        "help" | "?" => Err(anyhow!("help")), // Special case handled by REPL
        "quit" | "exit" => Err(anyhow!("quit")), // Special case handled by REPL
        "stop" => Err(anyhow!("stop")),       // Special case handled by REPL

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

        "seed" => {
            let seed =
                value.parse::<u64>().map_err(|_| anyhow!("Invalid seed value: {}", value))?;
            Ok(EngineCommand::SetSeed(seed))
        }

        _ => {
            Err(anyhow!("Unknown parameter: {}. Type 'help set' for available parameters.", param))
        }
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

/// Parse "record <wav|midi|musicxml>" command
/// Note: filename is not supported yet in the command interface
fn parse_record_command(tokens: &[&str]) -> Result<EngineCommand> {
    if tokens.is_empty() {
        return Err(anyhow!("Usage: record <wav|midi|musicxml>"));
    }

    let format = match tokens[0].to_lowercase().as_str() {
        "wav" => harmonium_core::events::RecordFormat::Wav,
        "midi" | "mid" => harmonium_core::events::RecordFormat::Midi,
        "musicxml" | "xml" => harmonium_core::events::RecordFormat::MusicXml,
        _ => return Err(anyhow!("Unknown format: {}. Use: wav, midi, musicxml", tokens[0])),
    };

    Ok(EngineCommand::StartRecording(format))
}

/// CLI-only commands, handled directly by the REPL (they need the playback
/// ring or the clock reader, so they do not go through `EngineCommand`).
#[derive(Clone, Debug, PartialEq)]
pub enum CliCommand {
    /// Set the swing ratio (already clamped to 0.5..=0.75; `clamped` tells
    /// whether the user's value was out of range).
    Swing { ratio: f32, clamped: bool },
    /// Audible tempo override, reached linearly over `ramp_pulses` pulse
    /// boundaries (`0` = jump at the next boundary).
    Tempo { bpm: f32, ramp_pulses: u8 },
    /// Back to the measures' tempo at the next pulse boundary.
    ClearTempo,
    /// Live visual metronome from `BeatInfo`.
    Beat,
    /// Sélection du générateur ou d'un profil validé.
    Style(Box<harmonium_core::StyleProfile>),
    /// Set reel podorythmie density (`0`, `1` or `2`).
    Feet(u8),
    /// Énergie du style, bornée explicitement à 0..1.
    Intensity(f32),
}

/// Swing bounds (phase 0, D3): 0.5 = straight, 0.75 = hard swing.
pub const SWING_MIN: f32 = 0.5;
/// Swing bounds (phase 0, D3): 0.5 = straight, 0.75 = hard swing.
pub const SWING_MAX: f32 = 0.75;

/// Parse the CLI-only commands (`swing`, `tempo`, `beat`). `tokens[0]` is the
/// command name; anything else is an error (this parser never shadows the
/// `EngineCommand` table above).
pub fn parse_cli_command(tokens: &[&str]) -> Result<CliCommand> {
    match tokens.first().copied() {
        Some("swing") => parse_swing_command(&tokens[1..]),
        Some("tempo") => parse_tempo_command(&tokens[1..]),
        Some("beat") => {
            if tokens.len() == 1 {
                Ok(CliCommand::Beat)
            } else {
                Err(anyhow!("Usage: beat"))
            }
        }
        Some("style") => parse_style_command(&tokens[1..]),
        Some("feet") => parse_feet_command(&tokens[1..]),
        Some("intensity") => {
            if tokens.len() != 2 {
                return Err(anyhow!("Usage: intensity <0..1>"));
            }
            let value: f32 =
                tokens[1].parse().map_err(|_| anyhow!("Intensité invalide: {}", tokens[1]))?;
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(anyhow!("Intensité invalide: {} (attendu 0..1)", tokens[1]));
            }
            Ok(CliCommand::Intensity(value))
        }
        Some(other) => Err(anyhow!("Not a CLI command: {other}")),
        None => Err(anyhow!("Empty command")),
    }
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

fn parse_style_command(tokens: &[&str]) -> Result<CliCommand> {
    if tokens.len() != 1 {
        return Err(anyhow!("Usage: style <identifiant|dossier>"));
    }
    Ok(CliCommand::Style(parse_style_value(tokens[0])?))
}

/// Parse `feet <0|1|2>`.
fn parse_feet_command(tokens: &[&str]) -> Result<CliCommand> {
    if tokens.len() != 1 {
        return Err(anyhow!("Usage: feet <0|1|2>"));
    }
    let density: u8 = tokens[0]
        .parse()
        .map_err(|_| anyhow!("Densité invalide: {}. Utiliser: feet 0|1|2", tokens[0]))?;
    if density > 2 {
        return Err(anyhow!("Densité hors plage ({density}). Utiliser: feet 0|1|2"));
    }
    Ok(CliCommand::Feet(density))
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

/// Parse `swing <ratio>`: a finite positive number, clamped to
/// 0.5..=0.75 (out-of-range values are clamped, not rejected).
fn parse_swing_command(tokens: &[&str]) -> Result<CliCommand> {
    if tokens.len() != 1 {
        return Err(anyhow!("Usage: swing <0.5-0.75>"));
    }
    let ratio: f32 =
        tokens[0].parse().map_err(|_| anyhow!("Invalid swing ratio: {}", tokens[0]))?;
    if !ratio.is_finite() || ratio <= 0.0 {
        return Err(anyhow!("Invalid swing ratio: {}", tokens[0]));
    }
    let clamped = ratio.clamp(SWING_MIN, SWING_MAX);
    Ok(CliCommand::Swing { ratio: clamped, clamped: clamped != ratio })
}

/// Parse `tempo <bpm> [pulses]` or `tempo off`. The engine clamps the BPM to
/// its 40–300 range; `pulses` is a `u8` ramp length (`0` = jump).
fn parse_tempo_command(tokens: &[&str]) -> Result<CliCommand> {
    if tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("off") {
        return Ok(CliCommand::ClearTempo);
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
    Ok(CliCommand::Tempo { bpm, ramp_pulses })
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
        assert!(matches!(
            parse_cli_command(&["swing", "0.6"]),
            Ok(CliCommand::Swing { ratio, clamped: false }) if ratio == 0.6
        ));
        assert!(matches!(
            parse_cli_command(&["swing", "0.5"]),
            Ok(CliCommand::Swing { ratio, clamped: false }) if ratio == 0.5
        ));
    }

    #[test]
    fn test_parse_swing_clamps_out_of_range() {
        assert!(matches!(
            parse_cli_command(&["swing", "0.9"]),
            Ok(CliCommand::Swing { ratio, clamped: true }) if ratio == SWING_MAX
        ));
        assert!(matches!(
            parse_cli_command(&["swing", "0.1"]),
            Ok(CliCommand::Swing { ratio, clamped: true }) if ratio == SWING_MIN
        ));
    }

    #[test]
    fn test_parse_swing_invalid() {
        assert!(parse_cli_command(&["swing"]).is_err());
        assert!(parse_cli_command(&["swing", "0.6", "0.7"]).is_err());
        assert!(parse_cli_command(&["swing", "lourd"]).is_err());
        assert!(parse_cli_command(&["swing", "-1.0"]).is_err());
        assert!(parse_cli_command(&["swing", "NaN"]).is_err());
    }

    #[test]
    fn test_parse_tempo() {
        assert!(matches!(
            parse_cli_command(&["tempo", "200"]),
            Ok(CliCommand::Tempo { bpm, ramp_pulses: 0 }) if bpm == 200.0
        ));
        assert!(matches!(
            parse_cli_command(&["tempo", "200", "8"]),
            Ok(CliCommand::Tempo { bpm, ramp_pulses: 8 }) if bpm == 200.0
        ));
    }

    #[test]
    fn test_parse_tempo_off() {
        assert!(matches!(parse_cli_command(&["tempo", "off"]), Ok(CliCommand::ClearTempo)));
        assert!(matches!(parse_cli_command(&["tempo", "OFF"]), Ok(CliCommand::ClearTempo)));
    }

    #[test]
    fn test_parse_tempo_invalid() {
        assert!(parse_cli_command(&["tempo"]).is_err());
        assert!(parse_cli_command(&["tempo", "vite"]).is_err());
        assert!(parse_cli_command(&["tempo", "-40"]).is_err());
        assert!(parse_cli_command(&["tempo", "200", "huit"]).is_err());
        assert!(parse_cli_command(&["tempo", "200", "300"]).is_err());
        assert!(parse_cli_command(&["tempo", "200", "8", "x"]).is_err());
    }

    #[test]
    fn test_parse_beat() {
        assert!(matches!(parse_cli_command(&["beat"]), Ok(CliCommand::Beat)));
        assert!(parse_cli_command(&["beat", "vite"]).is_err());
    }

    #[test]
    fn test_parse_cli_rejects_other_commands() {
        assert!(parse_cli_command(&["set", "bpm", "140"]).is_err());
        assert!(parse_cli_command(&[]).is_err());
    }

    #[test]
    fn test_parse_style_and_feet() {
        for d in 0..=2u8 {
            let s = d.to_string();
            assert_eq!(parse_cli_command(&["feet", &s]).ok(), Some(CliCommand::Feet(d)));
        }
        assert!(parse_cli_command(&["feet"]).is_err());
        assert!(parse_cli_command(&["feet", "3"]).is_err());
        assert!(parse_cli_command(&["feet", "dense"]).is_err());
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

//! Unified command interface for controlling the engine
//!
//! All control flows through EngineCommand (CLI, OSC).
//! This replaces the previous triple buffer + SPSC rings + mutex architecture
//! with a single lock-free command queue.

use serde::{Deserialize, Serialize};

use crate::events::RecordFormat;

/// Unified command interface for controlling the engine
/// All control flows through these commands (CLI, OSC)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EngineCommand {
    // === GLOBAL ===
    /// Set BPM (70-180). Sets a user override that persists through emotion changes.
    SetBpm(f32),

    /// Clear the user's BPM override, returning to emotion-mapped BPM
    ResetBpm,

    /// Set master volume (0.0-1.0)
    SetMasterVolume(f32),

    /// Set time signature
    SetTimeSignature { numerator: usize, denominator: usize },

    // === MODULE TOGGLES ===
    /// Enable/disable rhythm module
    EnableRhythm(bool),

    /// Enable/disable harmony module
    EnableHarmony(bool),

    /// Enable/disable melody module
    EnableMelody(bool),

    // === MIXER (per-channel) ===
    /// Set channel gain (channel 0-15, gain 0.0-1.0)
    SetChannelGain { channel: u8, gain: f32 },

    /// Set channel mute (channel 0-15)
    SetChannelMute { channel: u8, muted: bool },

    /// Set channel routing (channel 0-15, bank_id: -1 = default bank 0, >=0 = oxisynth bank)
    SetChannelRoute { channel: u8, bank_id: i32 },

    /// Set velocity base (channel 0-15, velocity 0-127)
    SetVelocityBase { channel: u8, velocity: u8 },

    // === RECORDING ===
    /// Start recording (Wav | Midi | MusicXml)
    StartRecording(RecordFormat),

    /// Stop recording (Wav | Midi | MusicXml)
    StopRecording(RecordFormat),

    // === CONTROL MODE (emotion mapping vs direct) ===
    /// Switch to emotion control mode
    UseEmotionMode,

    /// Switch to direct technical control mode
    UseDirectMode,

    /// Set emotional parameters (arousal, valence, density, tension), applied in emotion mode
    SetEmotionParams { arousal: f32, valence: f32, density: f32, tension: f32 },

    // === TIMELINE (Playhead control) ===
    /// Seek to a specific bar number (1-based)
    Seek(usize),

    /// Set loop region (start_bar..=end_bar, 1-based)
    SetLoop { start_bar: usize, end_bar: usize },

    /// Clear loop region
    ClearLoop,

    /// Generate a new melody with a fresh random seed.
    /// Resets the entire session to bar 1 with new random content.
    NewMelody,

    /// Set an explicit seed and regenerate from bar 1.
    /// Used for reproducible sessions (e.g. restoring a saved track).
    SetSeed(u64),

    /// Export timeline to MusicXML
    ExportTimeline(RecordFormat),

    // === PLAYHEAD (soft seek) ===
    /// Seek playhead to a bar WITHOUT resetting the writehead.
    /// Re-fills the ring buffer from the writehead's committed ScoreTimeline.
    SeekPlayhead(usize),

    /// Mute/unmute audio output (generation still runs, but output buffer is zeroed)
    SetOutputMute(bool),

    // === WRITEHEAD ===
    /// Set the writehead lookahead distance (minimum 4 bars)
    SetWriteheadLookahead(usize),

    // === STYLE PROFILE ===
    /// Installe un profil validé et sélectionne le générateur de profils.
    SetStyleProfile(Box<crate::style_profile::StyleProfile>),

    /// Rétablit le profil embarqué et sélectionne son générateur.
    ClearStyleProfile,

    // === UTILITY ===
    /// Request full state report
    GetState,

    /// Reset engine to defaults
    Reset,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serde_roundtrip() {
        let cmd = EngineCommand::SetBpm(140.0);
        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: EngineCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_emotion_params_command() {
        let cmd = EngineCommand::SetEmotionParams {
            arousal: 0.8,
            valence: 0.5,
            density: 0.7,
            tension: 0.6,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: EngineCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, deserialized);
    }
}

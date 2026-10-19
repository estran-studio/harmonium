//! Unified command interface for controlling the engine
//!
//! All control flows through EngineCommand (CLI, OSC).
//! This replaces the previous triple buffer + SPSC rings + mutex architecture
//! with a single lock-free command queue.

use serde::{Deserialize, Serialize};

use crate::{
    events::RecordFormat, harmony::HarmonyMode, params::HarmonyStrategy, sequencer::RhythmMode,
};

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

    /// Enable/disable voicing (harmonized chords)
    EnableVoicing(bool),

    // === RHYTHM ===
    /// Set rhythm mode (Euclidean | PerfectBalance | ClassicGroove)
    SetRhythmMode(RhythmMode),

    /// Set rhythm steps (16, 48, 96, 192)
    SetRhythmSteps(usize),

    /// Set rhythm pulses (1-steps)
    SetRhythmPulses(usize),

    /// Set rhythm rotation (0-steps)
    SetRhythmRotation(usize),

    /// Set rhythm density (0.0-1.0, for PerfectBalance mode)
    SetRhythmDensity(f32),

    /// Set rhythm tension (0.0-1.0)
    SetRhythmTension(f32),

    /// Set secondary rhythm parameters (for polyrhythm)
    SetRhythmSecondary { steps: usize, pulses: usize, rotation: usize },

    /// Set fixed kick mode (true = drum kit MIDI note 36, false = harmonized synth bass)
    SetFixedKick(bool),

    // === HARMONY ===
    /// Set harmony mode (Basic | Driver)
    SetHarmonyMode(HarmonyMode),

    /// Set harmony strategy (Steedman | NeoRiemannian | Parsimonious | Auto)
    SetHarmonyStrategy(HarmonyStrategy),

    /// Set harmony tension (0.0-1.0, affects LCC level)
    SetHarmonyTension(f32),

    /// Set harmony valence (-1.0 to 1.0, major/minor bias)
    SetHarmonyValence(f32),

    /// Set measures per chord (1 or 2)
    SetHarmonyMeasuresPerChord(usize),

    /// Set key root (0-11, where 0 is C)
    SetKeyRoot(u8),

    // === MELODY / VOICING ===
    /// Set melody smoothness (0.0-1.0, Hurst factor)
    SetMelodySmoothness(f32),

    /// Set melody octave (3-6)
    SetMelodyOctave(i32),

    /// Set voicing density (0.0-1.0)
    SetVoicingDensity(f32),

    /// Set voicing tension (0.0-1.0)
    SetVoicingTension(f32),

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

    // === BATCH OPERATIONS ===
    /// Set all rhythm parameters in one command
    SetAllRhythmParams {
        mode: RhythmMode,
        steps: usize,
        pulses: usize,
        rotation: usize,
        density: f32,
        tension: f32,
        secondary_steps: usize,
        secondary_pulses: usize,
        secondary_rotation: usize,
    },

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
    /// Load a style profile, applying its tuning overlay to engine parameters.
    SetStyleProfile(Box<crate::style_profile::StyleProfile>),

    /// Clear the active style profile, reverting to default tuning.
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
    fn test_complex_command_serde() {
        let cmd = EngineCommand::SetAllRhythmParams {
            mode: RhythmMode::PerfectBalance,
            steps: 48,
            pulses: 12,
            rotation: 4,
            density: 0.7,
            tension: 0.5,
            secondary_steps: 12,
            secondary_pulses: 3,
            secondary_rotation: 0,
        };
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

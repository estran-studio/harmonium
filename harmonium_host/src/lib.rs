use std::sync::{Arc, Mutex};

// Decoupled architecture: MusicComposer (main thread) + PlaybackEngine (audio thread)
pub mod composer;
pub mod playback;

// SoundFont resolution and fallback
pub mod soundfont;
pub use soundfont::{load_feet_soundfont, load_soundfont};

// Predictive transport clock (D1/D2): TransportClock, BeatInfo, ClockReader.
pub mod clock;
pub use clock::{BeatInfo, ClockReader, TransportClock};
// Re-exports from workspace crates
pub use harmonium_audio::{backend, realtime, voice_manager, voicing};
pub use harmonium_core::{events, fractal, harmony, log, mapper, params, sequencer};

// Real-time safety: Global allocator that panics on allocations in audio thread (debug builds only)
// Uses fully qualified path to avoid local mod ambiguity
#[cfg(debug_assertions)]
#[global_allocator]
static GLOBAL: harmonium_audio::realtime::rt_check::RTCheckAllocator =
    harmonium_audio::realtime::rt_check::RTCheckAllocator;

// Audio stream (cpal) and native handle require the `standalone` feature
#[cfg(feature = "standalone")]
pub mod audio;
#[cfg(feature = "standalone")]
pub mod native_handle;
pub use harmonium_core::{
    harmony::{HarmonyMode, basic as progression, melody as harmony_melody},
    params::{ControlMode, MusicalParams},
    sequencer::RhythmMode,
};
#[cfg(feature = "standalone")]
pub use native_handle::{LiveMidiSender, NativeHandle};

// Type aliases for complex types
pub type FontQueue = Arc<Mutex<Vec<(u32, Vec<u8>)>>>;
pub type FinishedRecordings = Arc<Mutex<Vec<(events::RecordFormat, Vec<u8>)>>>;

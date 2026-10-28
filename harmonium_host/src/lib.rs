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
pub use harmonium_audio::{backend, realtime, voice_manager};
pub use harmonium_core::{events, log, mapper, params};

// Real-time safety: Global allocator that panics on allocations in audio thread (debug builds only)
// Uses fully qualified path to avoid local mod ambiguity
#[cfg(debug_assertions)]
#[global_allocator]
static GLOBAL: harmonium_audio::realtime::rt_check::RTCheckAllocator =
    harmonium_audio::realtime::rt_check::RTCheckAllocator;

// Audio stream (cpal) and live host require the `standalone` feature; the OSC listener also needs a
// socket and a thread, so it does not exist on wasm.
#[cfg(feature = "standalone")]
pub mod audio;
#[cfg(feature = "standalone")]
pub mod live_host;
#[cfg(all(feature = "standalone", not(target_arch = "wasm32")))]
pub mod osc;
pub use harmonium_core::params::MusicalParams;
#[cfg(feature = "standalone")]
pub use live_host::LiveHost;

// Type aliases for complex types
pub type FinishedRecordings = Arc<Mutex<Vec<(events::RecordFormat, Vec<u8>)>>>;

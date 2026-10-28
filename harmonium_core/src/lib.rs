pub mod algo;
pub mod command;
pub mod events;
pub mod heterophony;
pub mod log;
pub mod mapper;
pub mod mix;
pub mod models;
pub mod odin;
pub mod ornaments;
pub mod params;
pub mod performance;
mod profile_source;
pub mod report;
pub mod style_profile;
pub mod theory;
pub mod time_plan;
pub mod timeline;
pub mod trad;
pub mod tuning;

// Re-export common types
pub use command::EngineCommand;
pub use events::AudioEvent;
pub use mapper::EmotionMapper;
pub use params::{EngineParams, InstrumentConfig, MusicalParams};
pub use report::{EngineReport, MeasureSnapshot, NoteEvent, NoteSnapshot};
pub use style_profile::StyleProfile;
pub use tuning::{ReelFiddleParams, TUNING_VERSION, TuningParams};

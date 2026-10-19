pub mod command;
pub mod events;
pub mod export;
pub mod fractal;
pub mod harmony;
pub mod log;
pub mod mapper;
pub mod params;
pub mod report;
pub mod sequencer;
pub mod style_profile;
pub mod timeline;
pub mod trad;
pub mod tuning;

// Re-export common types
pub use command::EngineCommand;
pub use events::AudioEvent;
pub use export::{
    ChordSymbol, GitVersion, to_musicxml, to_musicxml_with_chords, write_musicxml,
    write_musicxml_with_chords,
};
pub use mapper::{EmotionMapper, MapperConfig, UnifiedTensionSystem};
pub use params::{
    EngineParams, InstrumentConfig, MelodyScaleType, MusicalParams, Style, VarietyParams,
    key_root_to_pitch_symbol,
};
pub use report::{EngineReport, MeasureSnapshot, NoteEvent, NoteSnapshot};
pub use sequencer::Sequencer;
pub use style_profile::StyleProfile;
pub use tuning::{TUNING_VERSION, TuningOverlay, TuningParams};

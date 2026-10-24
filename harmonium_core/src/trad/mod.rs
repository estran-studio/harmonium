//! Génération traditionnelle : algorithmes génériques, données dans le profil actif.
pub mod accomp;
pub mod accomp_profile;
pub mod arrangement;
mod bass_passing;
pub mod cells;
pub mod dynamics;
pub mod feet;
pub mod fiddle;
pub mod generator;
pub mod harmony;
pub mod mode;
pub mod plan;
pub mod set;
mod strum;
pub use accomp::{Guitarist, bass_bar};
pub use arrangement::{
    ArrangementError, EntryPoint, IntroConfig, IntroForm, IntroInfo, IntroRole, MAX_TOURS, Part,
    REEL_ARRANGEMENT_TOML, ReelArrangement, Section, TrackEntries, TunePosition,
};
pub use cells::{
    Cell, CellBank, CellBankError, CellError, CellNote, CellRole, FIFTH_DIATONIC_STEPS,
    FIFTH_SEMITONES, REEL_CELLS_TOML,
};
pub use feet::{
    FeetPattern, FeetTable, FeetTableError, FootHit, PercussionVoice, REEL_FEET_TOML,
    draw_feet_humanize, feet_notes, feet_notes_with_cadence,
};
pub use fiddle::{
    FiddleLine, fold_midi_to_tessitura, same_cell_up_to_octave, tonic_midi_for_key,
    transpose_cell_octaves, unit_velocity,
};
pub use generator::{
    ActiveTune, TradGenerator, build_chord_context, pitch_class_name, trad_chord_name,
};
pub use harmony::{
    ChordDegree, ChordQuality, TradChord, TuneHarmony, TuneHarmonyError, candidate_chords,
    harmonize_bar,
};
pub use mode::{Mode, degree_to_diatonic_step, diatonic_step_to_degree};
pub use plan::{BarRole, ImposedRole, TunePlan};

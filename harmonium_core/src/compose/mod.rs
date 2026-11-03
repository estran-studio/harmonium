//! Composition musicale : algorithmes génériques, données dans le profil actif.
pub mod accomp;
pub mod accomp_profile;
pub mod arc;
pub mod arrangement;
mod bass_passing;
mod bass_template;
pub mod cells;
mod chord_spread;
pub mod difficulty;
pub mod dynamics;
pub mod generator;
pub mod harmony;
pub mod melody;
pub mod mode;
pub mod percussion;
mod phrase;
pub mod plan;
pub mod set;
mod voicing;
mod walking;
pub use accomp::{ChordPlayer, bass_bar};
pub use arc::{ArcBreak, ArcStage, ArcVariant, BassFeel, ChorusArc, TagWeight};
pub use arrangement::{
    Alternation, Arrangement, ArrangementError, DEFAULT_ARRANGEMENT_TOML, EntryPoint, IntroConfig,
    IntroForm, IntroInfo, IntroRole, MAX_ALTERNATION_PERIOD, MAX_TOURS, Part, Section,
    TrackEntries, TunePosition,
};
pub use cells::{
    Cell, CellBank, CellBankError, CellError, CellNote, CellRole, DEFAULT_CELLS_TOML,
    FIFTH_DIATONIC_STEPS, FIFTH_SEMITONES,
};
pub use difficulty::MelodyDifficulty;
pub use generator::{
    ActiveTune, TuneGenerator, build_chord_context, degree_chord_name, pitch_class_name,
};
pub use harmony::{
    ChordDegree, ChordQuality, DegreeChord, TuneHarmony, TuneHarmonyError, candidate_chords,
    harmonize_bar,
};
pub use melody::{
    MelodyLine, fold_midi_to_tessitura, same_cell_up_to_octave, tonic_midi_for_key,
    transpose_cell_octaves, unit_velocity,
};
pub use mode::{Mode, degree_to_diatonic_step, diatonic_step_to_degree};
pub use percussion::{
    DEFAULT_PERCUSSION_TOML, FillBar, PercussionFill, PercussionHit, PercussionPattern,
    PercussionPatternPlan, PercussionPhrase, PercussionTable, PercussionTableError,
    PercussionVoice, draw_percussion_humanize, percussion_notes,
};
pub use phrase::PhraseAt;
pub use plan::{BarRole, ImposedRole, TunePlan};

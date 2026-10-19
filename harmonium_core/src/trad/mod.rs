//! Fondations de génération de musique traditionnelle québécoise (phase 1 — Reel).
//!
//! Ce module regroupe les types purs et les données musicales du moteur trad :
//! - [`mode`] : modes diatoniques (`Ionian`, `Mixolydian`, `Dorian`, `Aeolian`)
//!   et conversion degrés → demi-tons ;
//! - [`cells`] : cellules mélodiques d'une mesure (8 croches), parseur de la
//!   notation D5, validation et [`CellBank`] chargée depuis `reel_cells.toml` ;
//! - [`plan`] : grille D3 (`TuneKind::Reel`), structure AABB (`Strain`,
//!   `RepeatScheme`, `TunePlan`) et rôles imposés par mesure (`BarRole`,
//!   `ImposedRole`) ;
//! - [`fiddle`] : ligne de violon d'un reel ([`FiddleLine`], sélection D6 des 16
//!   cellules, continuité, contour A/B et rendu en [`crate::timeline::TimelineNote`]) ;
//! - [`harmony`] : accords candidats par mode, harmonisation par demi-mesure
//!   ([`harmonize_bar`], [`TuneHarmony`]) ;
//! - [`accomp`] : basse et guitare d'accompagnement du reel (D8) ;
//! - [`feet`] : podorythmie du reel (D9), validation de table TOML et
//!   génération de notes de talon et pointe via [`feet_notes`] ;
//! - [`generator`] : [`TradGenerator`] assemblant plan, violon, harmonie,
//!   accompagnement et podorythmie en [`crate::timeline::Measure`] (D1).

pub mod accomp;
pub mod cells;
pub mod feet;
pub mod fiddle;
pub mod generator;
pub mod harmony;
pub mod mode;
pub mod plan;
pub use accomp::{Guitarist, bass_bar};
pub use cells::{
    Cell, CellBank, CellBankError, CellError, CellNote, CellRole, FIFTH_DIATONIC_STEPS,
    FIFTH_SEMITONES, NOTES_PER_CELL, REEL_CELLS_TOML,
};
pub use feet::{
    DEFAULT_HEEL_VELOCITY, DEFAULT_TOE_VELOCITY, FeetPattern, FeetTable, FeetTableError, FootHit,
    PITCH_HEEL, PITCH_TOE, REEL_FEET_TOML, REEL_STEPS, feet_notes,
};
pub use fiddle::{
    BARS_PER_STRAIN, CELLS_PER_TUNE, FiddleLine, eighth_velocity, fold_midi_to_tessitura,
    same_cell_up_to_octave, tonic_midi_for_key, transpose_cell_octaves,
};
pub use generator::{
    ActiveTune, BARS_PER_TUNE, TradGenerator, build_chord_context, draw_next_tune_plan,
    is_same_or_relative_plan, pitch_class_name, relative_tune_plan, trad_chord_name,
};
pub use harmony::{
    ChordDegree, ChordQuality, TradChord, TuneHarmony, TuneHarmonyError, candidate_chords,
    harmonize_bar,
};
pub use mode::{Mode, PitchClass, degree_to_diatonic_step, diatonic_step_to_degree};
pub use plan::{BarRole, ImposedRole, RepeatScheme, Strain, TuneKind, TunePlan};

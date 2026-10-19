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
//!   cellules, continuité, contour A/B et rendu en [`crate::timeline::TimelineNote`]).

pub mod cells;
pub mod fiddle;
pub mod mode;
pub mod plan;

pub use cells::{
    Cell, CellBank, CellBankError, CellError, CellNote, CellRole, NOTES_PER_CELL, REEL_CELLS_TOML,
};
pub use mode::{Mode, PitchClass, degree_to_diatonic_step, diatonic_step_to_degree};
pub use plan::{BarRole, ImposedRole, RepeatScheme, Strain, TuneKind, TunePlan};
pub use fiddle::{
    BARS_PER_STRAIN, CELLS_PER_TUNE, FiddleLine, eighth_velocity, fold_midi_to_tessitura,
    tonic_midi_for_key, transpose_cell_octaves,
};

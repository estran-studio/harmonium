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
//! - [`harmony`] : accords candidats par mode, harmonisation par demi-mesure
//!   ([`harmonize_bar`], [`TuneHarmony`]) ;
//! - [`accomp`] : basse et guitare d'accompagnement du reel (D8).
//! - [`feet`] : podorythmie du reel (D9), validation de table TOML et
//!   génération de notes de talon et pointe via [`feet_notes`].

pub mod accomp;
pub mod cells;
pub mod feet;
pub mod harmony;
pub mod mode;
pub mod plan;

pub use accomp::{Guitarist, bass_bar};
pub use cells::{
    Cell, CellBank, CellBankError, CellError, CellNote, CellRole, NOTES_PER_CELL, REEL_CELLS_TOML,
};
pub use feet::{
    DEFAULT_HEEL_VELOCITY, DEFAULT_TOE_VELOCITY, FeetPattern, FeetTable, FeetTableError, FootHit,
    PITCH_HEEL, PITCH_TOE, REEL_FEET_TOML, REEL_STEPS, feet_notes,
};
pub use harmony::{
    ChordDegree, ChordQuality, TradChord, TuneHarmony, TuneHarmonyError, candidate_chords,
    harmonize_bar,
};
pub use mode::{Mode, PitchClass, degree_to_diatonic_step, diatonic_step_to_degree};
pub use plan::{BarRole, ImposedRole, RepeatScheme, Strain, TuneKind, TunePlan};

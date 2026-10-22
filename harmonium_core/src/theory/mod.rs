//! Types musicaux et fonctions pures, indépendants des générateurs.
pub mod chord;
pub mod voice_leading;

/// Classe de hauteur chromatique (0 = C, 11 = B).
pub type PitchClass = u8;

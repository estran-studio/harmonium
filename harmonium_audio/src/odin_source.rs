//! Preset Odin fourni par l'hôte, comme les octets SF2.
//!
//! Type pur (aucune dépendance GPL) : la compilation sans `odin` ignore ces
//! presets et chaque piste joue son programme SoundFont. Avec `odin`, le
//! backend les analyse à la construction (hors callback) et les pistes qui
//! déclarent le preset en données (`[odin.<Piste>]` du profil) jouent Odin.

/// Un preset `.odin` fourni par l'hôte : nom court (sans extension ni chemin)
/// et octets du fichier, empruntés le temps de la construction.
#[derive(Clone, Copy, Debug)]
pub struct OdinPresetSource<'a> {
    pub name: &'a str,
    pub bytes: &'a [u8],
}

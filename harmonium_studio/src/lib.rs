//! Harmonium Studio : application egui, native et web (même code).
//!
//! - [`manifest`], [`packager`], [`loading`] : ressources (manifeste, générateur, logique du
//!   chargement). Pures : sans eframe, sans navigateur, sans disque pour le chargement.
//! - `platform` : tout ce qui diffère entre le bureau et le navigateur (sortie audio, fichiers et
//!   cache, minuteurs ; l'OSC, réservé au natif, y entrera avec l'écran Live).
//! - `app` : l'application, du chargement au premier son.
//!
//! Le studio ne lit que le contrat publié du moteur (`BeatInfo`, `MeasureSnapshot`) et ne le
//! pilote que par `LiveHost::apply(EngineCommand)` et `LiveHost::pump()`.

pub mod loading;
pub mod manifest;
pub mod packager;

#[cfg(feature = "app")]
mod app;
#[cfg(feature = "app")]
pub mod platform;

#[cfg(feature = "app")]
pub use platform::run;

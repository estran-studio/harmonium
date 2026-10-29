//! Harmonium Studio : application egui, native et web (même code).
//!
//! - [`manifest`], [`packager`], [`loading`] : ressources (manifeste, générateur, logique du
//!   chargement). Pures : sans eframe, sans navigateur, sans disque pour le chargement.
//! - [`live`] : l'écran Live sans fenêtre (session, intentions → `EngineCommand`, vues tirées de
//!   la sortie publiée).
//! - [`live_screen`] : l'écran Live dessiné par egui.
//! - [`platform`] : tout ce qui diffère entre le bureau et le navigateur (sortie audio, fichiers et
//!   cache, minuteurs, enregistrements, OSC).
//! - `app` : l'application, du chargement à l'écran Live.
//!
//! Le studio ne lit que le contrat publié du moteur (`BeatInfo`, `MeasureSnapshot`,
//! `EngineReport`) et l'API publique de l'hôte, et ne le pilote que par
//! `LiveHost::apply(EngineCommand)` et `LiveHost::pump()`.

pub mod loading;
pub mod manifest;
pub mod packager;

#[cfg(feature = "app")]
mod app;
#[cfg(feature = "app")]
pub mod live;
#[cfg(feature = "app")]
pub mod live_screen;
#[cfg(feature = "app")]
pub mod platform;

#[cfg(feature = "app")]
pub use platform::run;

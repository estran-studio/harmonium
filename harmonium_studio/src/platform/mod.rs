//! Ce qui diffère entre le bureau et le navigateur, et rien d'autre.
//!
//! | | natif (`native`) | web (`web`) |
//! |---|---|---|
//! | ressources | dossier sur disque, lu par un fil | `fetch` en flux, Cache Storage |
//! | hôte partagé | `Arc<Mutex<LiveHost>>` | `Rc<RefCell<LiveHost>>` (le moteur reste sur le fil de la page) |
//! | génération | un fil appelle `LiveHost::pump` | un minuteur de Worker appelle `LiveHost::pump` |
//! | sortie audio | cpal (carte son) | cpal WebAudio, créé en pause, repris au clic sur Start |
//! | réglages | arguments de la ligne de commande | paramètres d'URL |
//!
//! Le reste (écran, décisions de chargement, construction de l'hôte) est commun et vit hors de ce
//! module. Les deux côtés exposent les mêmes noms : `Settings`, `settings`, `start_loading`,
//! `HostHandle`, `Pump`, `start_pump`, `run`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use harmonium::{LiveHost, playback::MEASURE_WINDOW_CAP};
use harmonium_audio::odin_source::OdinPresetSource;
use harmonium_core::{EngineCommand, StyleProfile};

use crate::{
    loading::{LoadState, LoadView, Phase},
    manifest::LoadedResources,
};

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;

/// L'état du chargement, partagé entre le chargeur (qui écrit) et l'écran (qui lit).
pub type LoadHandle = Arc<Mutex<LoadState>>;

/// Verrou d'un état que seul un chargeur interrompu par une panique peut avoir empoisonné : l'état
/// reste lisible, et l'écran doit l'afficher plutôt que de tomber à son tour.
fn lock_state(state: &Mutex<LoadState>) -> MutexGuard<'_, LoadState> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Applique `change` à l'état puis demande une image : l'écran de chargement suit le chargeur.
fn update_state(
    state: &Mutex<LoadState>,
    ctx: &eframe::egui::Context,
    change: impl FnOnce(&mut LoadState),
) {
    change(&mut lock_state(state));
    ctx.request_repaint();
}

/// Où en est le chargement, vu par l'écran.
pub enum LoadOutcome {
    Pending,
    /// Tout est chargé : le contenu (rendu une seule fois) et les avis à garder à l'écran.
    Done {
        resources: LoadedResources,
        notices: Vec<String>,
    },
    Failed(String),
}

/// Prend le résultat du chargement s'il est fini.
pub fn poll_loading(state: &Mutex<LoadState>) -> LoadOutcome {
    let mut state = lock_state(state);
    match &state.phase {
        Phase::Failed(error) => LoadOutcome::Failed(error.clone()),
        Phase::Done => {
            let notices = state.notices();
            state.take_resources().map_or_else(
                || LoadOutcome::Failed("resources already taken".to_owned()),
                |resources| LoadOutcome::Done { resources, notices },
            )
        }
        Phase::Manifest | Phase::Resources => LoadOutcome::Pending,
    }
}

/// Ce que l'écran de chargement dessine maintenant.
pub fn load_view(state: &Mutex<LoadState>) -> LoadView {
    lock_state(state).view()
}

/// Mesures composées d'avance : le maximum que tiennent l'anneau de mesures et la fenêtre de
/// lecture (16).
///
/// Dimensionnement. Dans les profils distribués, une mesure dure de 0,94 s (`gigue_acadienne` à
/// son `max_bpm`) à 3,0 s (`hornpipe_britannique_coloniale` à son `min_bpm`) ; relevé sur le
/// moteur, voir le rapport S5. La mesure en cours compte dans la fenêtre : la génération peut donc
/// s'arrêter au moins 15 × 0,94 s ≈ 14 s avant que la lecture ne manque de mesures. Un onglet
/// masqué ralentit ses minuteurs à un réveil par seconde (limite des navigateurs pour un
/// minuteur de page) : c'est 14 réveils manqués au pire avant l'épuisement, contre 2 avec les
/// 4 mesures par défaut du moteur (3 × 0,94 s ≈ 2,8 s). Un arrêt complet du minuteur (onglet
/// gelé par le navigateur) n'est pas couvert : la musique s'arrête alors à la fin des mesures
/// composées, et reprend au réveil.
pub const LOOKAHEAD_BARS: usize = MEASURE_WINDOW_CAP;

/// Profil joué au démarrage.
pub const DEFAULT_PROFILE: &str = "reel_quebecois";

/// Construit l'hôte du moteur avec les ressources chargées, en pause : la sortie audio est ouverte
/// (et la banque analysée, ce qui bloque le fil appelant) mais ne joue qu'au `resume`. Le profil
/// par défaut est installé et les premières mesures sont composées.
pub fn open_host(
    resources: &LoadedResources,
    buffer_frames: Option<u32>,
) -> Result<HostHandle, String> {
    let presets: Vec<OdinPresetSource> = resources
        .odin_presets
        .iter()
        .map(|(name, bytes)| OdinPresetSource { name, bytes })
        .collect();
    let mut host = LiveHost::start_paused(
        Some(&resources.main_bank),
        resources.feet_bank.as_deref(),
        &presets,
        buffer_frames,
    )?;
    host.apply(EngineCommand::SetWriteheadLookahead(LOOKAHEAD_BARS))?;
    let profile = StyleProfile::named(DEFAULT_PROFILE).map_err(|e| e.to_string())?;
    host.apply(EngineCommand::SetStyleProfile(Box::new(profile)))?;
    // Les mesures sont prêtes quand Start reprend la sortie ; le studio n'a pas encore d'écran de
    // partition (S6) : les snapshots composés ne sont pas lus.
    host.pump();
    Ok(HostHandle::new(host))
}

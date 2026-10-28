//! Bureau : ressources sur disque, fil de génération, fenêtre eframe.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use eframe::egui;
use harmonium::LiveHost;

use super::{LoadHandle, update_state};
use crate::{
    app::StudioApp,
    loading::{ResourceStatus, Source, check_size, parse_buffer_frames},
    manifest::{LoadedResources, MANIFEST_FILE, Manifest},
};

/// Période de la génération : celle du REPL. Une mesure dure de 0,94 s à 3 s dans les profils
/// distribués (voir `LOOKAHEAD_BARS`) : 19 à 60 réveils par mesure, donc une commande est
/// recomposée sans attente audible.
const PUMP_PERIOD: Duration = Duration::from_millis(50);

/// Réglages du studio natif.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Taille du tampon audio en images (`--buffer`) ; `None` : celle de la carte son.
    pub buffer_frames: Option<u32>,
    /// Dossier des ressources (`--resources`) : `assets.json` et les fichiers qu'il cite, tels que
    /// les écrit `studio_assets`.
    pub resources_dir: PathBuf,
}

/// Lit la ligne de commande : `--buffer <images>`, `--resources <dossier>`.
pub fn settings() -> Result<Settings, String> {
    let mut settings = Settings {
        buffer_frames: None,
        resources_dir: Path::new(env!("CARGO_MANIFEST_DIR")).join("resources"),
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--buffer" => settings.buffer_frames = Some(parse_buffer_frames(&value()?)?),
            "--resources" => settings.resources_dir = PathBuf::from(value()?),
            _ => {
                return Err(format!(
                    "unknown argument `{flag}`\nusage: harmonium_studio [--buffer <frames>] [--resources <dir>]"
                ));
            }
        }
    }
    Ok(settings)
}

/// Lit les ressources sur disque, dans un fil : même écran que le web, sans cache (les fichiers
/// sont déjà sur l'appareil).
pub fn start_loading(ctx: egui::Context, settings: &Settings) -> LoadHandle {
    let state: LoadHandle = Arc::default();
    let task = state.clone();
    let dir = settings.resources_dir.clone();
    thread::spawn(move || {
        if let Err(error) = load(&dir, &task, &ctx) {
            update_state(&task, &ctx, |s| s.fail(error));
        }
    });
    state
}

fn load(
    dir: &Path,
    state: &Mutex<crate::loading::LoadState>,
    ctx: &egui::Context,
) -> Result<(), String> {
    let manifest_path = dir.join(MANIFEST_FILE);
    let text = fs::read_to_string(&manifest_path).map_err(|e| {
        format!(
            "{}: {e} (generate the resources with `make studio/assets`)",
            manifest_path.display()
        )
    })?;
    let manifest = Manifest::parse(&text)?;
    update_state(state, ctx, |s| s.manifest_loaded(&manifest));

    let mut contents = BTreeMap::new();
    for resource in &manifest.resources {
        let path = dir.join(&resource.file);
        let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        check_size(resource, bytes.len() as u64)?;
        contents.insert(resource.id.clone(), bytes);
        update_state(state, ctx, |s| {
            s.set_status(&resource.id, ResourceStatus::Loaded(Source::Device))
        });
    }
    let loaded = LoadedResources::assemble(&manifest, contents)?;
    update_state(state, ctx, |s| s.finish(loaded));
    Ok(())
}

/// Le moteur, partagé entre l'écran et le fil de génération.
#[derive(Clone)]
pub struct HostHandle(Arc<Mutex<LiveHost>>);

impl HostHandle {
    pub(super) fn new(host: LiveHost) -> Self {
        Self(Arc::new(Mutex::new(host)))
    }

    /// Accès exclusif à l'hôte le temps de `f`.
    pub fn with<R>(&self, f: impl FnOnce(&mut LiveHost) -> R) -> R {
        // Un fil de génération mort par panique n'invalide pas l'hôte.
        f(&mut self.0.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

/// Le fil de génération ; s'arrête quand on le lâche.
pub struct Pump {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

/// Lance le fil qui appelle `LiveHost::pump` toutes les [`PUMP_PERIOD`].
pub fn start_pump(host: &HostHandle) -> Result<Pump, String> {
    let stop = Arc::new(AtomicBool::new(false));
    let host = host.clone();
    let flag = stop.clone();
    let thread = thread::Builder::new()
        .name("studio-pump".into())
        .spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                host.with(|host| host.pump());
                thread::sleep(PUMP_PERIOD);
            }
        })
        .map_err(|e| format!("cannot start the generation thread: {e}"))?;
    Ok(Pump { stop, thread: Some(thread) })
}

impl Drop for Pump {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Ouvre la fenêtre et rend la main à la sortie de l'application.
pub fn run() {
    let settings = match settings() {
        Ok(settings) => settings,
        Err(message) => {
            eprintln!("harmonium_studio: {message}");
            std::process::exit(2);
        }
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Harmonium Studio")
            .with_inner_size([720.0, 480.0]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "Harmonium Studio",
        options,
        Box::new(move |cc| Ok(Box::new(StudioApp::new(cc, settings)))),
    );
    if let Err(error) = result {
        eprintln!("harmonium_studio: {error}");
        std::process::exit(1);
    }
}

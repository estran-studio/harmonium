//! Bureau : ressources sur disque, fil de génération, enregistrements en fichiers, OSC, fenêtre
//! eframe.

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
use harmonium::osc::OscListener;

use super::{LoadHandle, update_state};
use crate::{
    app::StudioApp,
    live::{Intent, Recording, Session},
    loading::{ResourceStatus, Source, check_size, parse_buffer_frames},
    manifest::{LoadedResources, MANIFEST_FILE, Manifest},
    workbench::{Draft, Origin, PROFILE_FILE, bank_names, is_safe_name},
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
    /// Dossier des enregistrements (`--recordings`).
    pub recordings_dir: PathBuf,
}

/// Dossier des enregistrements par défaut : `Harmonium Studio` dans le dossier de musique de
/// l'utilisateur (`~/Music` sur macOS), sinon dans son dossier personnel, sinon dans le dossier
/// courant.
fn default_recordings_dir() -> PathBuf {
    dirs::audio_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Harmonium Studio")
}

/// Lit la ligne de commande : `--buffer <images>`, `--resources <dossier>`,
/// `--recordings <dossier>`.
pub fn settings() -> Result<Settings, String> {
    let mut settings = Settings {
        buffer_frames: None,
        resources_dir: Path::new(env!("CARGO_MANIFEST_DIR")).join("resources"),
        recordings_dir: default_recordings_dir(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--buffer" => settings.buffer_frames = Some(parse_buffer_frames(&value()?)?),
            "--resources" => settings.resources_dir = PathBuf::from(value()?),
            "--recordings" => settings.recordings_dir = PathBuf::from(value()?),
            _ => {
                return Err(format!(
                    "unknown argument `{flag}`\nusage: harmonium_studio [--buffer <frames>] [--resources <dir>] [--recordings <dir>]"
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

/// La session du moteur, partagée entre l'écran, le fil de génération et l'écoute OSC.
#[derive(Clone)]
pub struct SessionHandle(Arc<Mutex<Session>>);

impl SessionHandle {
    #[must_use]
    pub fn new(session: Session) -> Self {
        Self(Arc::new(Mutex::new(session)))
    }

    /// Accès exclusif à la session le temps de `f`.
    pub fn with<R>(&self, f: impl FnOnce(&mut Session) -> R) -> R {
        // Un fil de génération mort par panique n'invalide pas la session.
        f(&mut self.0.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

/// Le fil de génération ; s'arrête quand on le lâche.
pub struct Pump {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

/// Lance le fil qui appelle `Session::pump` (donc `LiveHost::pump`) toutes les [`PUMP_PERIOD`].
pub fn start_pump(session: &SessionHandle) -> Result<Pump, String> {
    let stop = Arc::new(AtomicBool::new(false));
    let session = session.clone();
    let flag = stop.clone();
    let thread = thread::Builder::new()
        .name("studio-pump".into())
        .spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                session.with(Session::pump);
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

/// Écrit les enregistrements terminés dans un dossier.
pub struct RecordingSink {
    dir: PathBuf,
}

impl RecordingSink {
    #[must_use]
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Écrit `recording` sous son nom (suffixé `-2`, `-3`… si le fichier existe déjà) ; rend le
    /// texte à afficher.
    pub fn save(&self, recording: &Recording) -> Result<String, String> {
        fs::create_dir_all(&self.dir).map_err(|e| format!("{}: {e}", self.dir.display()))?;
        let name = Path::new(&recording.file_name);
        let stem = name.file_stem().and_then(|s| s.to_str()).unwrap_or("harmonium");
        let extension = name.extension().and_then(|s| s.to_str()).unwrap_or("bin");
        let mut path = self.dir.join(&recording.file_name);
        let mut copy = 1;
        while path.exists() {
            copy += 1;
            path = self.dir.join(format!("{stem}-{copy}.{extension}"));
        }
        fs::write(&path, &recording.bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(format!("Saved {}", path.display()))
    }
}

/// Les enregistrements vont dans `--recordings`.
#[must_use]
pub fn recording_sink(settings: &Settings) -> RecordingSink {
    RecordingSink::new(settings.recordings_dir.clone())
}

// === Atelier : dossiers de profils ===

/// Lit un profil pour l'atelier : `path` est son dossier ou son `profile.toml`. Les banques de son
/// `[files]` qui se lisent entrent dans le brouillon ; une banque absente est laissée au décodage,
/// qui la signale sur sa ligne de `[files]`.
pub fn read_profile_folder(path: &Path) -> Result<Draft, String> {
    let root = if path.is_dir() { path.join(PROFILE_FILE) } else { path.to_path_buf() };
    let dir = root.parent().unwrap_or(Path::new("")).to_path_buf();
    let text = fs::read_to_string(&root).map_err(|e| format!("{}: {e}", root.display()))?;
    let banks = bank_names(&text);
    let mut files = vec![(PROFILE_FILE.to_string(), text)];
    for name in banks.into_iter().filter(|name| is_safe_name(name)) {
        if let Ok(text) = fs::read_to_string(dir.join(&name)) {
            files.push((name, text));
        }
    }
    Draft::new(Origin::Folder(dir.display().to_string()), files)
}

/// Écrit chaque fichier du brouillon dans `dir` (créé au besoin) ; les autres fichiers du dossier
/// restent. Rend le nombre de fichiers écrits.
pub fn write_profile_folder(dir: &Path, draft: &Draft) -> Result<usize, String> {
    if let Some(file) = draft.files.iter().find(|file| !is_safe_name(&file.name)) {
        return Err(format!("unsafe file name: {}", file.name));
    }
    for file in &draft.files {
        let path = dir.join(&file.name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(&path, &file.text).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(draft.files.len())
}

/// L'écoute OSC du host (`/harmonium/params`) ; s'arrête quand on la lâche.
pub struct Osc(OscListener);

impl Osc {
    /// L'adresse UDP écoutée.
    #[must_use]
    pub fn address(&self) -> String {
        self.0.local_addr().to_string()
    }
}

/// Ouvre l'écoute OSC sur `addr` (`harmonium::osc::DEFAULT_ADDR`, celle du REPL) : chaque message
/// devient une commande exécutée par la session, comme un geste de l'écran.
pub fn start_osc(session: &SessionHandle, addr: &str) -> Result<Osc, String> {
    let session = session.clone();
    OscListener::spawn(addr, move |command| {
        // Un message refusé par l'hôte est ignoré, comme dans le REPL.
        let _ = session.with(|session| session.perform(Intent::Command(command)));
    })
    .map(Osc)
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
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1100.0, 700.0]),
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

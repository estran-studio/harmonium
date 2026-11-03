//! Bureau : ressources sur disque, fil de génération, enregistrements en fichiers, OSC, canal de
//! commande (liens d'ouverture), fenêtre eframe.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use eframe::egui;
use harmonium::osc::OscListener;
use harmonium_core::StyleProfile;

use super::{LoadHandle, update_state};
use crate::{
    app::StudioApp,
    control::{self, Control, Delivery},
    link::{OpenRequest, Opening, ProfileRef, ResolvedProfile},
    live::{Intent, Recording, Session},
    loading::{ResourceStatus, Source, check_size, parse_buffer_frames},
    manifest::{LoadedResources, MANIFEST_FILE, Manifest},
    reload::ProfileWatch,
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
    /// Lien à ouvrir au démarrage (`--open`), tel que reçu.
    pub open: Option<String>,
    /// Adresse du canal de commande local (`--control`).
    pub control_addr: String,
    /// Dossier des profils distribués sur disque (`--profiles`) : un lien qui nomme un
    /// identifiant y ouvre `<dossier>/<identifiant>` quand il existe, pour que l'édition se
    /// recharge sans recompiler ; sinon le profil compilé.
    pub profiles_dir: PathBuf,
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
/// `--recordings <dossier>`, `--open <lien>`, `--control <adresse>`, `--profiles <dossier>`.
pub fn settings() -> Result<Settings, String> {
    let mut settings = Settings {
        buffer_frames: None,
        resources_dir: Path::new(env!("CARGO_MANIFEST_DIR")).join("resources"),
        recordings_dir: default_recordings_dir(),
        open: None,
        control_addr: control::DEFAULT_ADDR.to_string(),
        profiles_dir: Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("profiles"),
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--buffer" => settings.buffer_frames = Some(parse_buffer_frames(&value()?)?),
            "--resources" => settings.resources_dir = PathBuf::from(value()?),
            "--recordings" => settings.recordings_dir = PathBuf::from(value()?),
            "--open" => settings.open = Some(value()?),
            "--control" => settings.control_addr = value()?,
            "--profiles" => settings.profiles_dir = PathBuf::from(value()?),
            _ => {
                return Err(format!(
                    "unknown argument `{flag}`\nusage: harmonium_studio [--buffer <frames>] [--resources <dir>] [--recordings <dir>] [--open <link>] [--control <addr>] [--profiles <dir>]\n       harmonium_studio open <link> | --profile <id|folder> [--seed <n>] …"
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

// === Liens d'ouverture ===

/// Lit le profil d'un dossier (ou de son `profile.toml`) avec le décodeur du moteur. Le chemin
/// devient absolu (c'est lui que le lien copié donne). La surveillance est prise avant la
/// lecture : une modification faite pendant la lecture est rechargée au lieu d'être perdue.
pub fn load_folder(path: &Path) -> Result<ResolvedProfile, String> {
    let path = fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let watch = ProfileWatch::new(&path);
    let profile = StyleProfile::load(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(ResolvedProfile { profile, watch: Some(watch) })
}

/// Lit le profil qu'une demande désigne. Un identifiant se résout vers le dossier
/// `<profiles_dir>/<identifiant>` quand il existe (l'édition se recharge sans recompiler), sinon
/// vers le profil compilé. Une erreur de profil porte son fichier et sa ligne.
pub fn resolve_open(request: OpenRequest, profiles_dir: &Path) -> Result<Opening, String> {
    let profile = match &request.profile {
        None => None,
        Some(ProfileRef::Folder(path)) => Some(load_folder(path)?),
        Some(ProfileRef::Id(id)) => {
            let dir = profiles_dir.join(id);
            if dir.join(PROFILE_FILE).is_file() {
                Some(load_folder(&dir)?)
            } else {
                let profile = StyleProfile::named(id).map_err(|e| e.to_string())?;
                Some(ResolvedProfile { profile, watch: None })
            }
        }
    };
    Ok(Opening { request, profile })
}

/// Les demandes d'ouverture reçues : celle de `--open`, puis celles du canal de commande.
pub struct OpenInbox {
    receiver: mpsc::Receiver<Result<Opening, String>>,
    /// Le canal vit avec la boîte ; `None` : il n'a pas pu s'ouvrir (dit dans la boîte).
    _control: Option<Control>,
}

/// Ouvre le canal de commande (`--control`) et lit `--open`. Chaque lien reçu est lu et validé
/// sur le fil du canal (le client sait donc tout de suite si le profil est refusé) ; l'écran le
/// prend par [`OpenInbox::take`]. Un canal qui ne s'ouvre pas (adresse prise, non locale) est
/// une erreur remise à l'écran, pas un arrêt.
pub fn open_inbox(ctx: &egui::Context, settings: &Settings) -> OpenInbox {
    let (sender, receiver) = mpsc::channel();
    if let Some(link) = &settings.open {
        let opening = OpenRequest::parse(link)
            .and_then(|request| resolve_open(request, &settings.profiles_dir));
        let _ = sender.send(opening);
    }
    let ctx = ctx.clone();
    let profiles_dir = settings.profiles_dir.clone();
    let channel = sender.clone();
    let control = Control::spawn(&settings.control_addr, move |request| {
        let opening = resolve_open(request, &profiles_dir)?;
        channel.send(Ok(opening)).map_err(|_| "the studio is closing".to_string())?;
        ctx.request_repaint();
        Ok(())
    });
    let control = match control {
        Ok(control) => Some(control),
        Err(message) => {
            let _ = sender.send(Err(format!("Links cannot be opened here: {message}")));
            None
        }
    };
    OpenInbox { receiver, _control: control }
}

impl OpenInbox {
    /// Les demandes arrivées depuis le dernier appel, dans l'ordre.
    pub fn take(&mut self) -> Vec<Result<Opening, String>> {
        self.receiver.try_iter().collect()
    }
}

/// L'heure locale, « hh:mm:ss ».
#[must_use]
pub fn clock_time() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

/// Le lien qui rouvre `request` : `harmonium://open?…`.
#[must_use]
pub fn share_link(request: &OpenRequest) -> String {
    request.to_link()
}

/// Ouvre la fenêtre et rend la main à la sortie de l'application.
pub fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("open") {
        std::process::exit(control::open_command(&args[1..]));
    }
    let settings = match settings() {
        Ok(settings) => settings,
        Err(message) => {
            eprintln!("harmonium_studio: {message}");
            std::process::exit(2);
        }
    };
    // Un studio tourne déjà : il prend le lien, celui-ci n'ouvre pas une deuxième fenêtre.
    if let Some(link) = &settings.open {
        match control::forward(&settings.control_addr, link) {
            Ok(Delivery::Delivered) => return,
            Ok(Delivery::NotRunning) => {}
            Err(message) => {
                eprintln!("harmonium_studio: {message}");
                std::process::exit(1);
            }
        }
    }
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

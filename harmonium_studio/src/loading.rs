//! Logique pure du chargement des ressources, commune au natif et au web : décisions de cache,
//! contrôle de taille, élagage, et l'état que l'écran de chargement dessine.
//!
//! Les chargeurs (`platform`) font les entrées-sorties (disque, `fetch`, Cache Storage) ; ils ne
//! décident de rien d'autre que ce que ce module leur dicte. Rien ici ne touche le réseau, le
//! disque ni le navigateur.

use crate::manifest::{LoadedResources, Manifest, Resource, Role};

/// Dossier des ressources dans le site servi (web), à côté de la page.
pub const RESOURCES_DIR: &str = "resources";
/// Nom du cache (Cache Storage) qui garde les ressources. Dédié : tout ce qu'il contient nous
/// appartient, donc tout ce que le manifeste ne cite plus peut être supprimé.
pub const CACHE_NAME: &str = "harmonium-studio-resources";

/// Adresse d'une ressource sur le site, relative à la page : c'est aussi sa clé dans le cache.
/// Le nom de fichier portant l'empreinte du contenu, la clé change avec le contenu.
pub fn cache_key(resource: &Resource) -> String {
    format!("{RESOURCES_DIR}/{}", resource.file)
}

/// Ce que le chargeur fait d'une ressource, d'après ce que le cache en garde.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheDecision {
    /// L'entrée du cache a la taille du manifeste : on la lit.
    UseCached,
    /// Pas d'entrée, ou une entrée de taille fausse (tronquée, ancienne) : on télécharge, puis on
    /// remplace l'entrée.
    Download,
}

/// `cached_len` : taille de l'entrée du cache (`None` : pas d'entrée).
pub fn decide(resource: &Resource, cached_len: Option<u64>) -> CacheDecision {
    if cached_len == Some(resource.bytes) {
        CacheDecision::UseCached
    } else {
        CacheDecision::Download
    }
}

/// Une ressource ne sort du réseau, et n'entre au cache, qu'à la taille du manifeste.
pub fn check_size(resource: &Resource, got: u64) -> Result<(), String> {
    if got == resource.bytes {
        Ok(())
    } else {
        Err(format!(
            "`{}`: got {got} bytes, the manifest says {} (download cut short or file changed)",
            resource.file, resource.bytes
        ))
    }
}

/// Les entrées du cache (adresses) que le manifeste ne cite pas : à supprimer après un
/// chargement réussi. Une entrée se reconnaît à son dernier segment, le nom de fichier à
/// empreinte, quel que soit le préfixe de l'adresse.
pub fn stale_entries<'a>(cached_urls: &'a [String], manifest: &Manifest) -> Vec<&'a str> {
    let files = manifest.files();
    cached_urls
        .iter()
        .map(String::as_str)
        .filter(|url| !files.contains(file_name_of(url).as_str()))
        .collect()
}

/// Le nom de fichier d'une adresse, décodé : le navigateur garde l'adresse encodée
/// (`Bass%20Crusher%20[RM].….odin`), le manifeste le nom tel quel (`Bass Crusher [RM].….odin`).
fn file_name_of(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    percent_decode(path.rsplit('/').next().unwrap_or(path))
}

/// Remplace chaque `%XX` par son octet ; une séquence invalide reste telle quelle.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let escaped = (bytes[i] == b'%')
            .then(|| bytes.get(i + 1..i + 3))
            .flatten()
            .and_then(|pair| std::str::from_utf8(pair).ok())
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match escaped {
            Some(byte) => {
                decoded.push(byte);
                i += 3;
            }
            None => {
                decoded.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

/// Taille de tampon (en images) acceptée : de quoi refuser une faute de frappe
/// (`buffer=20`, `buffer=2048000`) sans figer les choix musicaux.
pub const BUFFER_FRAMES_RANGE: std::ops::RangeInclusive<u32> = 64..=16384;

/// Lit une taille de tampon demandée (`?buffer=…` ou `--buffer …`).
pub fn parse_buffer_frames(text: &str) -> Result<u32, String> {
    let frames: u32 = text.trim().parse().map_err(|_| format!("`{text}` is not a frame count"))?;
    if BUFFER_FRAMES_RANGE.contains(&frames) {
        Ok(frames)
    } else {
        Err(format!(
            "buffer of {frames} frames is outside {}..={}",
            BUFFER_FRAMES_RANGE.start(),
            BUFFER_FRAMES_RANGE.end()
        ))
    }
}

/// D'où une ressource est venue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// Déjà sur l'appareil : cache du navigateur, ou fichier local en natif.
    Device,
    /// Téléchargée à cette visite.
    Network,
}

/// Où en est une ressource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceStatus {
    Waiting,
    /// Cherchée dans le cache.
    Checking,
    /// Octets reçus.
    Downloading(u64),
    Loaded(Source),
    Failed(String),
}

/// Une ligne de l'écran de chargement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceLine {
    pub id: String,
    pub label: String,
    pub bytes: u64,
    pub status: ResourceStatus,
}

impl ResourceLine {
    fn new(resource: &Resource) -> Self {
        Self {
            id: resource.id.clone(),
            label: label(resource),
            bytes: resource.bytes,
            status: ResourceStatus::Waiting,
        }
    }

    /// Le texte de l'état : « on this device » ou « downloading x / y MB ».
    pub fn status_text(&self) -> String {
        match &self.status {
            ResourceStatus::Waiting => "waiting".to_owned(),
            ResourceStatus::Checking => "looking on this device…".to_owned(),
            ResourceStatus::Downloading(got) => {
                format!("downloading {} / {} MB", megabytes(*got), megabytes(self.bytes))
            }
            ResourceStatus::Loaded(Source::Device) => "on this device".to_owned(),
            ResourceStatus::Loaded(Source::Network) => {
                format!("downloaded ({} MB)", megabytes(self.bytes))
            }
            ResourceStatus::Failed(error) => format!("failed: {error}"),
        }
    }

    /// Part chargée, `0.0..=1.0`.
    pub fn fraction(&self) -> f32 {
        match &self.status {
            ResourceStatus::Loaded(_) => 1.0,
            ResourceStatus::Downloading(got) => {
                (*got as f64 / self.bytes.max(1) as f64).min(1.0) as f32
            }
            _ => 0.0,
        }
    }
}

fn label(resource: &Resource) -> String {
    match resource.role() {
        Some(Role::MainBank) => "Instrument bank".to_owned(),
        Some(Role::FeetBank) => "Feet bank".to_owned(),
        Some(Role::OdinPreset(name)) => format!("Synth preset {name}"),
        None => resource.id.clone(),
    }
}

/// Mégaoctets décimaux (10⁶ octets), un chiffre après la virgule.
pub fn megabytes(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / 1_000_000.0)
}

/// Le cache de l'appareil.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheState {
    /// Pas encore sollicité, ou sans objet (natif : fichiers locaux).
    Untouched,
    /// Les ressources téléchargées y entrent.
    Working,
    /// Absent ou en échec (API absente, quota, navigation privée) : les octets restent en
    /// mémoire pour cette visite, et la prochaine visite télécharge de nouveau.
    Unavailable(String),
}

/// Étape du chargement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Lecture du manifeste.
    Manifest,
    /// Lecture ou téléchargement des ressources.
    Resources,
    /// Tout est chargé ; [`LoadState::take_resources`] rend le contenu.
    Done,
    Failed(String),
}

/// Ce que l'écran de chargement dessine à une image donnée.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadView {
    pub phase: Phase,
    pub lines: Vec<ResourceLine>,
    pub fraction: f32,
    pub notices: Vec<String>,
}

/// Ce que l'écran de chargement dessine, et ce que le chargeur met à jour. Un seul verrou le
/// protège (voir `LoadHandle`) : le chargeur écrit, l'interface lit.
#[derive(Debug)]
pub struct LoadState {
    pub phase: Phase,
    pub lines: Vec<ResourceLine>,
    pub cache: CacheState,
    resources: Option<LoadedResources>,
}

impl Default for LoadState {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadState {
    pub fn new() -> Self {
        Self {
            phase: Phase::Manifest,
            lines: Vec::new(),
            cache: CacheState::Untouched,
            resources: None,
        }
    }

    /// Le manifeste est lu : une ligne par ressource.
    pub fn manifest_loaded(&mut self, manifest: &Manifest) {
        self.lines = manifest.resources.iter().map(ResourceLine::new).collect();
        self.phase = Phase::Resources;
    }

    pub fn set_status(&mut self, id: &str, status: ResourceStatus) {
        if let Some(line) = self.lines.iter_mut().find(|line| line.id == id) {
            line.status = status;
        }
    }

    pub fn cache_working(&mut self) {
        if self.cache == CacheState::Untouched {
            self.cache = CacheState::Working;
        }
    }

    pub fn cache_unavailable(&mut self, reason: impl Into<String>) {
        self.cache = CacheState::Unavailable(reason.into());
    }

    pub fn finish(&mut self, resources: LoadedResources) {
        self.resources = Some(resources);
        self.phase = Phase::Done;
    }

    pub fn fail(&mut self, error: impl Into<String>) {
        self.phase = Phase::Failed(error.into());
    }

    /// Le contenu chargé, une seule fois.
    pub fn take_resources(&mut self) -> Option<LoadedResources> {
        self.resources.take()
    }

    /// Ce que l'écran dessine, copié sous le verrou (le contenu chargé, lui, n'est pas copié).
    pub fn view(&self) -> LoadView {
        LoadView {
            phase: self.phase.clone(),
            lines: self.lines.clone(),
            fraction: self.fraction(),
            notices: self.notices(),
        }
    }

    /// Part chargée de l'ensemble, `0.0..=1.0`, pondérée par les tailles.
    pub fn fraction(&self) -> f32 {
        let total: u64 = self.lines.iter().map(|line| line.bytes).sum();
        if total == 0 {
            return 0.0;
        }
        let done: f64 =
            self.lines.iter().map(|line| f64::from(line.fraction()) * line.bytes as f64).sum();
        (done / total as f64) as f32
    }

    fn downloaded_any(&self) -> bool {
        self.lines.iter().any(|line| {
            matches!(
                line.status,
                ResourceStatus::Downloading(_) | ResourceStatus::Loaded(Source::Network)
            )
        })
    }

    /// Les avis à afficher : la première visite le dit ; un cache refusé prévient que la prochaine
    /// visite télécharge de nouveau.
    pub fn notices(&self) -> Vec<String> {
        match &self.cache {
            CacheState::Unavailable(_) => vec![
                "This device cannot keep the files, so the next visit downloads them again."
                    .to_owned(),
            ],
            _ if self.downloaded_any() => {
                vec!["First visit: next visits load from this device.".to_owned()]
            }
            _ => Vec::new(),
        }
    }
}

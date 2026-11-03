//! Rechargement à chaud : surveille les fichiers d'un profil sur disque.
//!
//! [`ProfileWatch`] regarde `profile.toml` et les banques de son `[files]` (date de modification
//! et taille ; pas de crate de surveillance : quelques `stat` toutes les [`POLL_PERIOD`] suffisent
//! pour un profil de six fichiers). Un changement n'est lu qu'une fois les fichiers au repos
//! pendant [`DEBOUNCE`] : un éditeur qui écrit en plusieurs temps ou par renommage ne donne
//! qu'un rechargement. Le profil est relu par `StyleProfile::load`, le décodeur de `--profile` :
//! les erreurs portent le fichier et la ligne.
//!
//! Le module ne connaît ni egui ni le moteur : l'appelant lui donne l'heure (`poll(now)`) et
//! installe le profil rendu. Il se compile aussi pour le web, qui n'en crée jamais (pas de
//! disque).

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use harmonium_core::StyleProfile;
use web_time::Instant;

use crate::workbench::{PROFILE_FILE, bank_names, is_safe_name};

/// Intervalle minimal entre deux lectures des dates de modification.
pub const POLL_PERIOD: Duration = Duration::from_millis(150);

/// Durée sans changement avant de relire le profil.
pub const DEBOUNCE: Duration = Duration::from_millis(300);

/// Ce qu'on sait d'un fichier : date de modification et taille ; `None` : absent.
type Stamp = Option<(SystemTime, u64)>;

fn stamp(path: &Path) -> Stamp {
    let meta = fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

/// Le résultat d'un rechargement : le nouveau profil, ou l'erreur du décodeur
/// (`fichier:ligne: champ: message`).
pub type Reloaded = Result<StyleProfile, String>;

/// Surveille le dossier d'un profil.
#[derive(Clone, Debug)]
pub struct ProfileWatch {
    dir: PathBuf,
    /// Fichiers surveillés : `profile.toml`, puis les banques de son `[files]`.
    files: Vec<PathBuf>,
    /// État des fichiers à la dernière lecture des dates.
    seen: Vec<Stamp>,
    /// État des fichiers au dernier rechargement (ou à l'ouverture).
    loaded: Vec<Stamp>,
    /// Dernier instant où `seen` a changé.
    changed_at: Option<Instant>,
    polled_at: Option<Instant>,
}

impl ProfileWatch {
    /// Surveille `path` : un dossier de profil, ou son `profile.toml`. L'état présent est celui
    /// du profil que l'appelant vient de charger : rien n'est rechargé avant un changement.
    #[must_use]
    pub fn new(path: &Path) -> Self {
        let root = if path.is_dir() { path.join(PROFILE_FILE) } else { path.to_path_buf() };
        let dir = root.parent().unwrap_or(Path::new("")).to_path_buf();
        let files = watched_files(&dir, &root);
        let seen: Vec<Stamp> = files.iter().map(|file| stamp(file)).collect();
        Self { dir, files, loaded: seen.clone(), seen, changed_at: None, polled_at: None }
    }

    /// Le dossier surveillé.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Les fichiers surveillés.
    #[must_use]
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    /// À appeler souvent : rend le rechargement quand les fichiers ont changé puis sont restés
    /// au repos [`DEBOUNCE`] ; `None` sinon. Un profil invalide rend l'erreur, une fois : le
    /// rechargement suivant attend une nouvelle modification.
    pub fn poll(&mut self, now: Instant) -> Option<Reloaded> {
        if self.polled_at.is_some_and(|last| now.saturating_duration_since(last) < POLL_PERIOD) {
            return None;
        }
        self.polled_at = Some(now);
        // Le `[files]` de `profile.toml` peut avoir changé : la liste suit le fichier racine.
        if self.files.first().is_some_and(|root| stamp(root) != self.seen[0]) {
            let root = self.files[0].clone();
            self.files = watched_files(&self.dir, &root);
            self.seen.resize(self.files.len(), None);
        }
        let current: Vec<Stamp> = self.files.iter().map(|file| stamp(file)).collect();
        if current != self.seen {
            self.seen = current;
            self.changed_at = Some(now);
        }
        let settled = self
            .changed_at
            .is_some_and(|changed| now.saturating_duration_since(changed) >= DEBOUNCE);
        if !settled {
            return None;
        }
        self.changed_at = None;
        if self.seen == self.loaded {
            return None;
        }
        self.loaded = self.seen.clone();
        Some(StyleProfile::load(&self.dir).map_err(|error| error.to_string()))
    }
}

/// `profile.toml`, puis les banques de son `[files]` (celles dont le nom sort du dossier sont
/// ignorées, comme dans l'atelier).
fn watched_files(dir: &Path, root: &Path) -> Vec<PathBuf> {
    let banks = fs::read_to_string(root).map(|text| bank_names(&text)).unwrap_or_default();
    let mut files = vec![root.to_path_buf()];
    files.extend(banks.into_iter().filter(|name| is_safe_name(name)).map(|name| dir.join(name)));
    files
}

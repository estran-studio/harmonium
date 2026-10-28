//! Générateur du manifeste : de sources (banques, presets Odin) à un dossier de ressources servi
//! tel quel (web) ou lu sur disque (natif) : `assets.json` et les fichiers à empreinte.
//!
//! Les sources ne sont jamais modifiées ni commitées : le dossier de sortie reçoit des copies
//! nommées par leur empreinte. Ce qui change pour une autre source (banque SF3, fichiers d'une
//! release) est le chemin passé ici ; le manifeste et le chargeur ne bougent pas.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::manifest::{
    FEET_BANK_ID, MAIN_BANK_ID, MANIFEST_FILE, Manifest, ODIN_ID_PREFIX, Resource,
    hashed_file_name, sha256_hex,
};

/// D'où viennent les ressources.
#[derive(Clone, Debug)]
pub struct PackSpec {
    /// Banque principale (SF2 ou SF3).
    pub main_bank: PathBuf,
    /// Banque de pieds, si elle existe.
    pub feet_bank: Option<PathBuf>,
    /// Dossier de presets `*.odin` (nom du preset = tige du fichier) ; absent ou vide : aucun
    /// preset.
    pub odin_dir: Option<PathBuf>,
}

/// Écrit dans `out` le manifeste et une copie à empreinte de chaque source ; retire les fichiers
/// d'un manifeste précédent qui ne sont plus référencés. Rend le manifeste écrit.
pub fn pack(spec: &PackSpec, out: &Path) -> Result<Manifest, String> {
    let mut sources = vec![(MAIN_BANK_ID.to_owned(), spec.main_bank.clone())];
    if let Some(feet) = &spec.feet_bank {
        sources.push((FEET_BANK_ID.to_owned(), feet.clone()));
    }
    if let Some(dir) = &spec.odin_dir {
        sources.extend(odin_sources(dir)?);
    }

    let previous = previous_files(out);
    fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;

    let mut resources = Vec::with_capacity(sources.len());
    for (id, path) in sources {
        let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let original = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("{}: not a file name", path.display()))?;
        let sha256 = sha256_hex(&bytes);
        let file = hashed_file_name(original, &sha256);
        let target = out.join(&file);
        let up_to_date = fs::metadata(&target).is_ok_and(|m| m.len() == bytes.len() as u64);
        if !up_to_date {
            fs::write(&target, &bytes).map_err(|e| format!("{}: {e}", target.display()))?;
        }
        resources.push(Resource { id, file, bytes: bytes.len() as u64, sha256 });
    }

    let manifest = Manifest { resources };
    // Relit ce qu'on écrit : un manifeste que le chargeur refuserait ne sort jamais d'ici.
    let json = manifest.to_json()?;
    let manifest = Manifest::parse(&json)?;
    fs::write(out.join(MANIFEST_FILE), json).map_err(|e| format!("{MANIFEST_FILE}: {e}"))?;

    let kept = manifest.files();
    for stale in previous.iter().filter(|file| !kept.contains(file.as_str())) {
        // Un fichier déjà absent n'est pas une erreur.
        let _ = fs::remove_file(out.join(stale));
    }
    Ok(manifest)
}

/// Les `*.odin` d'un dossier : `(id, chemin)`, triés par nom. Dossier absent : aucun preset.
fn odin_sources(dir: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut presets = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        let is_odin = path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("odin"));
        if let (true, Some(stem)) = (is_odin, path.file_stem().and_then(|s| s.to_str())) {
            presets.push((format!("{ODIN_ID_PREFIX}{stem}"), path));
        }
    }
    presets.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(presets)
}

/// Fichiers listés par le manifeste déjà présent dans `out` (vide s'il n'y en a pas, ou s'il est
/// illisible : il ne sert qu'à nettoyer).
fn previous_files(out: &Path) -> HashSet<String> {
    fs::read_to_string(out.join(MANIFEST_FILE))
        .ok()
        .and_then(|json| serde_json::from_str::<Manifest>(&json).ok())
        .map(|manifest| {
            manifest
                .resources
                .into_iter()
                .map(|r| r.file)
                .filter(|file| {
                    !file.is_empty() && !file.contains(['/', '\\']) && !file.starts_with('.')
                })
                .collect()
        })
        .unwrap_or_default()
}

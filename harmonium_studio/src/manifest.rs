//! Manifeste des ressources du studio : `assets.json`.
//!
//! Le générateur (`studio_assets`, voir [`crate::packager`]) l'écrit au build ; le chargeur le lit à
//! chaque visite. Pour chaque ressource : `id` (son rôle), `file` (nom portant l'empreinte du
//! contenu : un nouveau contenu change le nom, donc la clé du cache), `bytes` (la taille, qui donne
//! la progression même sans `Content-Length` et sert de contrôle d'intégrité) et `sha256`.
//!
//! Le format de banque n'apparaît nulle part ici : le rôle `main-bank` désigne un SoundFont, SF2 ou
//! SF3 (oxisynth reconnaît le format au contenu). Changer de format, ou de source, ne touche ni ce
//! module ni le chargeur.

use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

/// Nom du manifeste dans le dossier des ressources.
pub const MANIFEST_FILE: &str = "assets.json";
/// Rôle de la banque principale (obligatoire).
pub const MAIN_BANK_ID: &str = "main-bank";
/// Rôle de la banque de pieds (facultative : sans elle, les pieds jouent le kit GM du canal 9).
pub const FEET_BANK_ID: &str = "feet-bank";
/// Préfixe du rôle d'un preset Odin : `odin:<nom court>`, le nom étant celui que les profils
/// désignent (`[odin.<piste>]`).
pub const ODIN_ID_PREFIX: &str = "odin:";
/// Nombre de caractères hexadécimaux de l'empreinte qui entrent dans le nom de fichier.
pub const HASH_IN_NAME: usize = 12;

/// Une ressource du manifeste.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub file: String,
    pub bytes: u64,
    pub sha256: String,
}

/// Ce qu'une ressource est pour le moteur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role<'a> {
    MainBank,
    FeetBank,
    /// Preset Odin, par son nom court.
    OdinPreset(&'a str),
}

impl Resource {
    /// Le rôle de la ressource, `None` si l'`id` n'est pas un rôle connu.
    pub fn role(&self) -> Option<Role<'_>> {
        match self.id.as_str() {
            MAIN_BANK_ID => Some(Role::MainBank),
            FEET_BANK_ID => Some(Role::FeetBank),
            id => id
                .strip_prefix(ODIN_ID_PREFIX)
                .filter(|name| !name.is_empty())
                .map(Role::OdinPreset),
        }
    }
}

/// Le manifeste : les ressources à charger.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub resources: Vec<Resource>,
}

impl Manifest {
    /// Lit et valide un manifeste : rôles connus et uniques, banque principale présente, noms de
    /// fichier sans chemin et portant l'empreinte, empreintes `sha256` bien formées, tailles non
    /// nulles.
    pub fn parse(json: &str) -> Result<Self, String> {
        let manifest: Self =
            serde_json::from_str(json).map_err(|e| format!("invalid {MANIFEST_FILE}: {e}"))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Sérialisation stable (lisible, ordre des ressources conservé).
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    fn validate(&self) -> Result<(), String> {
        let mut seen = HashSet::new();
        for resource in &self.resources {
            if resource.role().is_none() {
                return Err(format!("unknown resource id `{}`", resource.id));
            }
            if !seen.insert(resource.id.as_str()) {
                return Err(format!("duplicate resource id `{}`", resource.id));
            }
            validate_file_name(resource)?;
            if resource.bytes == 0 {
                return Err(format!("resource `{}` has no bytes", resource.id));
            }
        }
        if !seen.contains(MAIN_BANK_ID) {
            return Err(format!("{MANIFEST_FILE} has no `{MAIN_BANK_ID}`"));
        }
        Ok(())
    }

    /// Les noms de fichier du manifeste : tout autre fichier de cache est périmé.
    pub fn files(&self) -> HashSet<&str> {
        self.resources.iter().map(|r| r.file.as_str()).collect()
    }

    /// Taille totale à charger, en octets.
    pub fn total_bytes(&self) -> u64 {
        self.resources.iter().map(|r| r.bytes).sum()
    }
}

fn validate_file_name(resource: &Resource) -> Result<(), String> {
    let file = &resource.file;
    if file.is_empty() || file.starts_with('.') || file.contains(['/', '\\']) {
        return Err(format!("resource `{}`: `{file}` is not a plain file name", resource.id));
    }
    let hash = &resource.sha256;
    if hash.len() != 64 || !hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return Err(format!("resource `{}`: sha256 is not 64 lowercase hex digits", resource.id));
    }
    if !file.contains(&hash[..HASH_IN_NAME]) {
        return Err(format!(
            "resource `{}`: `{file}` does not carry the content hash {}",
            resource.id,
            &hash[..HASH_IN_NAME]
        ));
    }
    Ok(())
}

/// `sha256` d'un contenu, en hexadécimal minuscule.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Nom de fichier portant l'empreinte : `<tige>.<12 hex>.<extension>` à partir du nom d'origine.
pub fn hashed_file_name(original: &str, sha256: &str) -> String {
    let short = &sha256[..HASH_IN_NAME.min(sha256.len())];
    match original.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => format!("{stem}.{short}.{ext}"),
        _ => format!("{original}.{short}"),
    }
}

/// Les ressources lues, rangées par rôle, prêtes pour `LiveHost::start`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LoadedResources {
    pub main_bank: Vec<u8>,
    pub feet_bank: Option<Vec<u8>>,
    /// Presets Odin `(nom court, octets)`, triés par nom : zéro preset est un cas normal.
    pub odin_presets: Vec<(String, Vec<u8>)>,
}

impl LoadedResources {
    /// Range le contenu de chaque ressource (par `id`) selon son rôle.
    pub fn assemble(
        manifest: &Manifest,
        mut contents: BTreeMap<String, Vec<u8>>,
    ) -> Result<Self, String> {
        let mut loaded = Self::default();
        let mut main_bank = None;
        for resource in &manifest.resources {
            let bytes = contents
                .remove(&resource.id)
                .ok_or_else(|| format!("resource `{}` was not loaded", resource.id))?;
            match resource.role() {
                Some(Role::MainBank) => main_bank = Some(bytes),
                Some(Role::FeetBank) => loaded.feet_bank = Some(bytes),
                Some(Role::OdinPreset(name)) => loaded.odin_presets.push((name.to_owned(), bytes)),
                None => return Err(format!("unknown resource id `{}`", resource.id)),
            }
        }
        loaded.main_bank = main_bank.ok_or_else(|| format!("no `{MAIN_BANK_ID}` loaded"))?;
        loaded.odin_presets.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(loaded)
    }
}

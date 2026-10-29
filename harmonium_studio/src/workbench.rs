//! L'atelier de profils sans fenêtre.
//!
//! - [`Draft`] : le brouillon, c'est-à-dire les fichiers d'un profil tels qu'ils sont sur disque
//!   (`profile.toml` puis les banques de son `[files]`, commentaires et découpe compris), et leur
//!   origine. [`Draft::decode`] passe par le décodeur unique du moteur (`StyleProfile::from_files`) :
//!   mêmes erreurs, situées dans leur fichier et sur leur ligne, que `harmonium-cli --profile`.
//! - [`Draft::fork`] : copie modifiable d'un profil distribué, sous un nouvel identifiant.
//! - Archive (web) : [`Draft::to_zip`] et [`files_from_upload`] ; le dossier enregistré ou
//!   téléchargé se relit tel quel par `StyleProfile::load`.
//! - [`toml_spans`] : découpage d'un texte TOML pour sa coloration.
//!
//! Le module ne connaît ni egui ni la plate-forme.

use std::{
    io::{Cursor, Read, Write},
    ops::Range,
};

use harmonium_core::{StyleProfile, style_profile::ProfileError};
use serde::{Deserialize, Serialize};
use toml_edit::{DocumentMut, Item, Value};

/// Fichier racine d'un profil.
pub const PROFILE_FILE: &str = "profile.toml";

/// Extensions gardées d'un téléversement ou d'une archive : les fichiers du profil et sa notice.
const TEXT_EXTENSIONS: [&str; 2] = ["toml", "md"];

/// Suffixe de l'identifiant d'une copie (`reel_quebecois` → `reel_quebecois_fork`).
const FORK_ID_SUFFIX: &str = "_fork";

/// Un fichier du profil, par son nom relatif (celui qu'emploie `[files]`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileFile {
    pub name: String,
    pub text: String,
}

/// D'où vient le brouillon.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    /// Copie d'un profil distribué, par son identifiant.
    Fork(String),
    /// Dossier lu sur disque (natif), par son chemin.
    Folder(String),
    /// Fichiers ou archive téléversés (web).
    Upload,
}

/// Le profil en cours d'édition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Draft {
    pub origin: Origin,
    /// `profile.toml` d'abord, puis les banques dans l'ordre de `[files]`, puis le reste par nom.
    pub files: Vec<ProfileFile>,
}

impl Draft {
    /// Un brouillon de `files` (nom relatif, texte) ; il faut un `profile.toml`.
    pub fn new(origin: Origin, files: Vec<(String, String)>) -> Result<Self, String> {
        let root = files
            .iter()
            .find(|(name, _)| name == PROFILE_FILE)
            .ok_or_else(|| format!("no {PROFILE_FILE} among the files"))?;
        let banks = bank_names(&root.1);
        let rank = |name: &str| {
            if name == PROFILE_FILE {
                0
            } else {
                banks.iter().position(|bank| bank == name).map_or(usize::MAX, |i| i + 1)
            }
        };
        let mut files: Vec<ProfileFile> =
            files.into_iter().map(|(name, text)| ProfileFile { name, text }).collect();
        files.sort_by(|a, b| rank(&a.name).cmp(&rank(&b.name)).then_with(|| a.name.cmp(&b.name)));
        files.dedup_by(|a, b| a.name == b.name);
        Ok(Self { origin, files })
    }

    /// Copie modifiable du profil distribué `id` : mêmes fichiers, mêmes commentaires, sous
    /// l'identifiant `<id>_fork` et le nom « <nom> (fork) », pour ne pas se confondre avec
    /// l'original dans le studio.
    pub fn fork(id: &str) -> Result<Self, String> {
        let index = StyleProfile::distributed()
            .iter()
            .position(|profile| profile.meta.id == id)
            .ok_or_else(|| format!("no distributed profile `{id}`"))?;
        let original = &StyleProfile::distributed()[index];
        let files = StyleProfile::distributed_files()[index]
            .iter()
            .map(|(name, text)| {
                let text = if *name == PROFILE_FILE {
                    rename(
                        text,
                        &format!("{id}{FORK_ID_SUFFIX}"),
                        &format!("{} (fork)", original.meta.name),
                    )?
                } else {
                    (*text).to_string()
                };
                Ok(((*name).to_string(), text))
            })
            .collect::<Result<Vec<_>, String>>()?;
        Self::new(Origin::Fork(id.to_string()), files)
    }

    /// Décode et valide le brouillon par le décodeur du moteur.
    pub fn decode(&self) -> Result<StyleProfile, ProfileError> {
        StyleProfile::from_files(self.files.iter().map(|file| (&file.name, &file.text)))
    }

    #[must_use]
    pub fn file(&self, name: &str) -> Option<&ProfileFile> {
        self.files.iter().find(|file| file.name == name)
    }

    /// Nom du dossier du profil : son identifiant (`[meta] id`), sinon `profile`.
    #[must_use]
    pub fn folder_name(&self) -> String {
        self.file(PROFILE_FILE)
            .and_then(|root| root.text.parse::<DocumentMut>().ok())
            .and_then(|doc| doc.get("meta")?.get("id")?.as_str().map(str::to_string))
            .filter(|id| {
                !id.is_empty()
                    && id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
            })
            .unwrap_or_else(|| "profile".to_string())
    }

    /// Archive ZIP du dossier du profil : `<identifiant>/<fichier>` pour chaque fichier, sans
    /// compression (quelques centaines de Ko de texte au plus).
    pub fn to_zip(&self) -> Result<Vec<u8>, String> {
        let folder = self.folder_name();
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o644);
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for file in &self.files {
            if !is_safe_name(&file.name) {
                return Err(format!("unsafe file name: {}", file.name));
            }
            writer
                .start_file(format!("{folder}/{}", file.name), options)
                .map_err(|e| e.to_string())?;
            writer.write_all(file.text.as_bytes()).map_err(|e| e.to_string())?;
        }
        Ok(writer.finish().map_err(|e| e.to_string())?.into_inner())
    }
}

/// Les banques citées par `[files]` d'un `profile.toml`, dans leur ordre ; rien si le texte ne
/// se lit pas (l'erreur viendra du décodage).
#[must_use]
pub fn bank_names(profile_text: &str) -> Vec<String> {
    let Ok(doc) = profile_text.parse::<DocumentMut>() else {
        return Vec::new();
    };
    let Some(files) = doc.get("files").and_then(Item::as_table_like) else {
        return Vec::new();
    };
    files.iter().filter_map(|(_, item)| item.as_str().map(str::to_string)).collect()
}

/// Remplace `[meta] id` et `[meta] name` en gardant le reste du texte (commentaires, mise en
/// forme).
fn rename(profile_text: &str, id: &str, name: &str) -> Result<String, String> {
    let mut doc = profile_text.parse::<DocumentMut>().map_err(|e| e.to_string())?;
    let meta = doc
        .get_mut("meta")
        .and_then(Item::as_table_like_mut)
        .ok_or_else(|| format!("{PROFILE_FILE}: no [meta] table"))?;
    for (key, new) in [("id", id), ("name", name)] {
        let value = meta
            .get_mut(key)
            .and_then(Item::as_value_mut)
            .ok_or_else(|| format!("{PROFILE_FILE}: no meta.{key}"))?;
        let decor = value.decor().clone();
        *value = Value::from(new);
        *value.decor_mut() = decor;
    }
    Ok(doc.to_string())
}

/// Un nom relatif sans remontée ni racine : on peut l'écrire sous le dossier du profil.
#[must_use]
pub fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains('\\')
        && name.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

fn has_text_extension(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(_, extension)| {
        TEXT_EXTENSIONS.iter().any(|kept| extension.eq_ignore_ascii_case(kept))
    })
}

/// Les fichiers d'un profil téléversé : une archive ZIP (celle de [`Draft::to_zip`], ou un
/// dossier compressé), ou les fichiers du dossier choisis ensemble. Seuls les `.toml` et `.md`
/// sont gardés ; il faut un `profile.toml`.
pub fn files_from_upload(files: Vec<(String, Vec<u8>)>) -> Result<Vec<(String, String)>, String> {
    if let [(name, bytes)] = files.as_slice()
        && name.to_ascii_lowercase().ends_with(".zip")
    {
        return files_from_zip(bytes);
    }
    let mut kept = Vec::new();
    for (name, bytes) in files {
        // Un navigateur donne le nom seul ; on n'en garde que le dernier segment par prudence.
        let name = name.rsplit(['/', '\\']).next().unwrap_or_default().to_string();
        if !has_text_extension(&name) {
            continue;
        }
        let text = String::from_utf8(bytes).map_err(|_| format!("{name}: not UTF-8 text"))?;
        kept.push((name, text));
    }
    if kept.iter().all(|(name, _)| name != PROFILE_FILE) {
        return Err(format!("no {PROFILE_FILE} among the chosen files"));
    }
    Ok(kept)
}

/// Les fichiers d'une archive : le `profile.toml` le moins profond fixe le dossier du profil ;
/// les `.toml` et `.md` sous ce dossier en sont, nommés relativement à lui.
pub fn files_from_zip(bytes: &[u8]) -> Result<Vec<(String, String)>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let names: Vec<String> = archive
        .file_names()
        .filter_map(Result::ok)
        .map(|name| name.into_owned())
        .filter(|name| !name.starts_with("__MACOSX/") && !name.ends_with('/'))
        .collect();
    let root = names
        .iter()
        .filter(|name| *name == PROFILE_FILE || name.ends_with(&format!("/{PROFILE_FILE}")))
        .min_by_key(|name| name.matches('/').count())
        .ok_or_else(|| format!("no {PROFILE_FILE} in the archive"))?;
    let prefix = root[..root.len() - PROFILE_FILE.len()].to_string();
    let mut files = Vec::new();
    for name in &names {
        let Some(relative) = name.strip_prefix(&prefix) else { continue };
        if !has_text_extension(relative) || !is_safe_name(relative) {
            continue;
        }
        let mut entry = archive.by_name(name).map_err(|e| format!("{name}: {e}"))?;
        let mut text = String::new();
        entry.read_to_string(&mut text).map_err(|e| format!("{name}: {e}"))?;
        files.push((relative.to_string(), text));
    }
    Ok(files)
}

// === Coloration ===

/// Ce qu'est un morceau de texte TOML, pour sa couleur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TomlToken {
    Comment,
    /// En-tête de table, `[meter]` ou `[[form.parts]]`.
    Header,
    Key,
    String,
    /// Nombre, booléen ou date.
    Literal,
    /// Ponctuation, blancs et le reste.
    Plain,
}

/// Découpe `text` en morceaux contigus qui le couvrent entièrement, chacun avec son
/// [`TomlToken`]. Tolérant : un texte invalide se découpe quand même (le décodage dira l'erreur).
#[must_use]
pub fn toml_spans(text: &str) -> Vec<(Range<usize>, TomlToken)> {
    let bytes = text.as_bytes();
    let mut spans: Vec<(Range<usize>, TomlToken)> = Vec::new();
    let mut push = |range: Range<usize>, token: TomlToken| {
        if range.is_empty() {
            return;
        }
        match spans.last_mut() {
            Some((last, kind)) if *kind == token && last.end == range.start => {
                last.end = range.end;
            }
            _ => spans.push((range, token)),
        }
    };
    // Profondeur des tableaux et tables en ligne ouverts ; une clé est attendue en début de
    // ligne hors de toute valeur, et après `{` ou `,` dans une table en ligne.
    let mut nesting: Vec<u8> = Vec::new();
    let mut expect_key = true;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        let start = i;
        match c {
            b'#' => {
                i = line_end(bytes, i);
                push(start..i, TomlToken::Comment);
            }
            b'\n' => {
                i += 1;
                push(start..i, TomlToken::Plain);
                if nesting.is_empty() {
                    expect_key = true;
                }
            }
            b'"' | b'\'' => {
                i = string_end(bytes, i);
                push(start..i, if expect_key { TomlToken::Key } else { TomlToken::String });
            }
            b'[' if expect_key && nesting.is_empty() => {
                // En-tête : jusqu'au `]` fermant de la ligne (ou sa fin).
                let end = line_end(bytes, i);
                let close = text[i..end].rfind(']').map_or(end, |at| i + at + 1);
                i = close;
                push(start..i, TomlToken::Header);
                expect_key = false;
            }
            b'[' | b'{' => {
                i += 1;
                nesting.push(c);
                expect_key = c == b'{';
                push(start..i, TomlToken::Plain);
            }
            b']' | b'}' => {
                i += 1;
                nesting.pop();
                expect_key = false;
                push(start..i, TomlToken::Plain);
            }
            b',' => {
                i += 1;
                expect_key = nesting.last() == Some(&b'{');
                push(start..i, TomlToken::Plain);
            }
            b'=' => {
                i += 1;
                expect_key = false;
                push(start..i, TomlToken::Plain);
            }
            b' ' | b'\t' | b'\r' | b'.' => {
                i += 1;
                push(start..i, TomlToken::Plain);
            }
            _ => {
                // Un mot : clé nue, ou littéral (nombre, booléen, date).
                while i < bytes.len()
                    && !WORD_ENDS.contains(&bytes[i])
                    && !(expect_key && bytes[i] == b'.')
                {
                    i += 1;
                }
                // Au moins un caractère avance (un octet de continuation UTF-8 compris).
                if i == start {
                    i += 1;
                }
                while !text.is_char_boundary(i) {
                    i += 1;
                }
                push(start..i, if expect_key { TomlToken::Key } else { TomlToken::Literal });
            }
        }
    }
    spans
}

/// Octets qui terminent un mot nu (clé ou littéral).
const WORD_ENDS: &[u8] = b" \t\r\n#=,[]{}\"'";

fn line_end(bytes: &[u8], from: usize) -> usize {
    bytes[from..].iter().position(|&c| c == b'\n').map_or(bytes.len(), |at| from + at)
}

/// Fin d'une chaîne qui commence à `from` (simple, littérale ou sur plusieurs lignes) ; une
/// chaîne simple non fermée s'arrête à la fin de sa ligne.
fn string_end(bytes: &[u8], from: usize) -> usize {
    let quote = bytes[from];
    let triple = bytes.len() >= from + 3 && bytes[from + 1] == quote && bytes[from + 2] == quote;
    let mut i = from + if triple { 3 } else { 1 };
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\\' && quote == b'"' {
            i += 2;
            continue;
        }
        if !triple && c == b'\n' {
            return i;
        }
        if c == quote {
            if !triple {
                return i + 1;
            }
            if bytes.len() >= i + 3 && bytes[i + 1] == quote && bytes[i + 2] == quote {
                return i + 3;
            }
        }
        i += 1;
    }
    bytes.len()
}

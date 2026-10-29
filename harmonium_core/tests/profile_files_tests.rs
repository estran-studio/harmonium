//! Profils multi-fichiers (phase 015, S4) : les profils embarqués sont les fichiers d'origine
//! de `profiles/`, un seul décodeur les lit (disque ou mémoire), et chaque erreur désigne le
//! fichier source et la ligne de la valeur fautive.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use harmonium_core::StyleProfile;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn profiles_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../profiles")
}

/// Dossiers de `profiles/` qui contiennent un `profile.toml`, triés.
fn profile_dirs() -> TestResult<Vec<PathBuf>> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(profiles_dir())?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.join("profile.toml").is_file())
        .collect();
    dirs.sort();
    Ok(dirs)
}

/// Copie d'un profil distribué dans un dossier temporaire, effacé à la fin du test.
struct Copy {
    dir: PathBuf,
}

impl Copy {
    fn of(id: &str, test: &str) -> TestResult<Self> {
        let dir = std::env::temp_dir().join(format!("harmonium_s4_{}_{test}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;
        for entry in fs::read_dir(profiles_dir().join(id))? {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "toml") {
                if let Some(name) = path.file_name() {
                    fs::copy(&path, dir.join(name))?;
                }
            }
        }
        Ok(Self { dir })
    }
    fn read(&self, file: &str) -> TestResult<String> {
        Ok(fs::read_to_string(self.dir.join(file))?)
    }
    fn write(&self, file: &str, text: &str) -> TestResult {
        Ok(fs::write(self.dir.join(file), text)?)
    }
    /// Remplace `from` (présent une seule fois) par `to` dans `file` ; rend la ligne (base 1)
    /// où commence `to`.
    fn replace(&self, file: &str, from: &str, to: &str) -> TestResult<usize> {
        let text = self.read(file)?;
        assert_eq!(text.matches(from).count(), 1, "{from:?} doit être unique dans {file}");
        let at = text.find(from).ok_or("motif absent")?;
        self.write(file, &text.replacen(from, to, 1))?;
        Ok(text[..at].matches('\n').count() + 1)
    }
}

impl Drop for Copy {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn every_embedded_profile_is_its_directory_loaded_from_disk() -> TestResult {
    let dirs = profile_dirs()?;
    assert_eq!(StyleProfile::distributed().len(), dirs.len(), "un profil distribué par dossier");
    for dir in &dirs {
        let loaded = StyleProfile::load(dir)?;
        let embedded = StyleProfile::named(&loaded.meta.id)?;
        assert_eq!(loaded, embedded, "{}", dir.display());
    }
    Ok(())
}

#[test]
fn embedded_files_are_the_originals_listed_by_files() -> TestResult {
    let embedded = StyleProfile::distributed_files();
    assert_eq!(embedded.len(), StyleProfile::distributed().len());
    for (files, profile) in embedded.iter().zip(StyleProfile::distributed()) {
        assert_eq!(&StyleProfile::from_files(files.iter().copied())?, profile);
        let dir = profiles_dir().join(&profile.meta.id);
        let root: toml::Table = toml::from_str(files[0].1)?;
        let mut expected = vec!["profile.toml".to_string()];
        if let Some(listed) = root.get("files").and_then(toml::Value::as_table) {
            expected.extend(listed.values().filter_map(|v| v.as_str()).map(String::from));
        }
        let names: Vec<_> = files.iter().map(|(name, _)| (*name).to_string()).collect();
        assert_eq!(
            names, expected,
            "{} : profile.toml puis [files], rien d'autre",
            profile.meta.id
        );
        for (name, text) in *files {
            assert_eq!(*text, fs::read_to_string(dir.join(name))?, "{} / {name}", profile.meta.id);
        }
    }
    Ok(())
}

#[test]
fn invalid_bank_value_names_the_bank_file_and_line() -> TestResult {
    let copy = Copy::of("nt_desert", "bank_value")?;
    let line = copy.replace(
        "cells.toml",
        r#"notes = "5 - - - - - 6 - - 8 - -""#,
        r#"notes = "5 - - - - - oups - - 8 - -""#,
    )?;
    let error = StyleProfile::load(&copy.dir).err().ok_or("valeur invalide acceptée")?;
    assert_eq!(error.file.as_deref(), Some("cells.toml"), "{error}");
    assert_eq!(error.line, Some(line), "{error}");
    assert!(error.field.starts_with("melody.cells.cells["), "{error}");
    assert!(error.to_string().starts_with(&format!("cells.toml:{line}: melody.cells.cells[")));
    Ok(())
}

#[test]
fn validation_error_in_a_bank_names_the_bank_file_and_line() -> TestResult {
    let copy = Copy::of("nt_desert", "bank_validation")?;
    // Une cellule trop courte passe la lecture et échoue à `validate()`.
    let line =
        copy.replace("cells.toml", r#"notes = "5 - - - - - 6 - - 8 - -""#, r#"notes = "1 2 3""#)?;
    let error = StyleProfile::load(&copy.dir).err().ok_or("cellule courte acceptée")?;
    assert_eq!(error.file.as_deref(), Some("cells.toml"), "{error}");
    assert_eq!(error.line, Some(line), "{error}");
    assert!(error.field.ends_with(".notes"), "{error}");
    Ok(())
}

#[test]
fn validation_error_in_the_root_file_names_profile_toml_and_line() -> TestResult {
    let copy = Copy::of("nt_desert", "root_validation")?;
    let line = copy.replace("profile.toml", "master_db = 9.0", "master_db = 99.0")?;
    let error = StyleProfile::load(&copy.dir).err().ok_or("gain hors bornes accepté")?;
    assert_eq!(error.field, "mix.master_db");
    assert_eq!(error.file.as_deref(), Some("profile.toml"));
    assert_eq!(error.line, Some(line));
    assert!(error.to_string().starts_with(&format!("profile.toml:{line}: mix.master_db: ")));
    Ok(())
}

#[test]
fn odin_validation_error_names_profile_toml_and_line() -> TestResult {
    let copy = Copy::of("nt_desert", "odin_validation")?;
    let line =
        copy.replace("profile.toml", r#"preset = "Sausage Bassener""#, r#"preset = "dir/basse""#)?;
    let error = StyleProfile::load(&copy.dir).err().ok_or("preset avec chemin accepté")?;
    assert_eq!(error.field, "odin.Bass.zones[0].preset", "{error}");
    assert_eq!(error.file.as_deref(), Some("profile.toml"), "{error}");
    assert_eq!(error.line, Some(line), "{error}");
    Ok(())
}

#[test]
fn validation_error_in_the_arrangement_bank_names_that_file() -> TestResult {
    let copy = Copy::of("nt_desert", "arrangement_validation")?;
    let text = copy.read("arrangement.toml")?;
    let at = text.find("pickup_eighths").ok_or("pickup_eighths absent de la banque")?;
    let line = text[..at].matches('\n').count() + 1;
    let end = text[at..].find('\n').map_or(text.len(), |n| at + n);
    copy.write(
        "arrangement.toml",
        &format!("{}pickup_eighths = 200{}", &text[..at], &text[end..]),
    )?;
    let error = StyleProfile::load(&copy.dir).err().ok_or("levée hors mesure acceptée")?;
    assert_eq!(error.file.as_deref(), Some("arrangement.toml"), "{error}");
    assert_eq!(error.line, Some(line), "{error}");
    Ok(())
}

#[test]
fn syntax_error_names_file_line_and_column() -> TestResult {
    let copy = Copy::of("nt_desert", "syntax")?;
    let text = copy.read("feet.toml")?;
    let line = text.lines().count() + 2;
    copy.write("feet.toml", &format!("{text}\nbroken = [1, 2\n"))?;
    let error = StyleProfile::load(&copy.dir).err().ok_or("syntaxe invalide acceptée")?;
    assert_eq!(error.file.as_deref(), Some("feet.toml"), "{error}");
    assert!(error.line.is_some_and(|l| l >= line), "ligne {:?} attendue ≥ {line}", error.line);
    assert!(error.column.is_some(), "{error}");
    assert!(error.to_string().starts_with(&format!("feet.toml:{}: ", error.line.unwrap_or(0))));
    Ok(())
}

#[test]
fn missing_bank_points_at_its_files_entry() -> TestResult {
    let copy = Copy::of("nt_desert", "missing")?;
    fs::remove_file(copy.dir.join("dynamics.toml"))?;
    let text = copy.read("profile.toml")?;
    let line = text
        .lines()
        .position(|l| l.starts_with("dynamics = "))
        .ok_or("entrée files.dynamics absente")?
        + 1;
    let error = StyleProfile::load(&copy.dir).err().ok_or("banque absente acceptée")?;
    assert_eq!(error.field, "files.dynamics");
    assert_eq!(error.file.as_deref(), Some("profile.toml"));
    assert_eq!(error.line, Some(line));
    Ok(())
}

#[test]
fn in_memory_files_report_the_same_errors_as_the_disk() -> TestResult {
    let copy = Copy::of("nt_desert", "memory")?;
    copy.replace(
        "cells.toml",
        r#"notes = "5 - - - - - 6 - - 8 - -""#,
        r#"notes = "5 - - - - - oups - - 8 - -""#,
    )?;
    let files: BTreeMap<String, String> = [
        "profile.toml",
        "cells.toml",
        "feet.toml",
        "arrangement.toml",
        "dynamics.toml",
        "set.toml",
    ]
    .into_iter()
    .map(|name| Ok((name.to_string(), copy.read(name)?)))
    .collect::<TestResult<_>>()?;
    let from_memory = StyleProfile::from_files(&files).err().ok_or("valeur invalide acceptée")?;
    let from_disk = StyleProfile::load(&copy.dir).err().ok_or("valeur invalide acceptée")?;
    assert_eq!(from_memory, from_disk);

    let mut without_root = files.clone();
    without_root.remove("profile.toml");
    let error =
        StyleProfile::from_files(&without_root).err().ok_or("profil sans racine accepté")?;
    assert_eq!(error.file.as_deref(), Some("profile.toml"));

    let mut without_bank = files;
    without_bank.remove("set.toml");
    let error = StyleProfile::from_files(&without_bank).err().ok_or("banque absente acceptée")?;
    assert_eq!(error.field, "files.set");
    assert_eq!(error.message, "fichier absent");
    Ok(())
}

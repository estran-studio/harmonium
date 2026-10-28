//! Build script for `harmonium_core`
//!
//! Captures git version information at compile time to avoid runtime git dependencies,
//! and lists the original files of the distributed style profiles in
//! `$OUT_DIR/distributed_profiles.rs` (embedded verbatim, decoded at runtime).

use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Les fichiers d'un profil distribué : `profile.toml`, puis chaque banque de son `[files]`.
fn profile_files(dir: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let profile_path = dir.join("profile.toml");
    let root: toml::Table = toml::from_str(&std::fs::read_to_string(&profile_path)?)?;
    let mut names = vec!["profile.toml".to_string()];
    if let Some(files) = root.get("files") {
        let files = files
            .as_table()
            .ok_or_else(|| format!("files : table attendue dans {}", profile_path.display()))?;
        for (key, name) in files {
            let name = name
                .as_str()
                .ok_or_else(|| format!("files.{key} dans {}", profile_path.display()))?;
            names.push(name.to_string());
        }
    }
    Ok(names)
}

/// Génère `$OUT_DIR/distributed_profiles.rs` : pour chaque profil distribué, ses fichiers
/// d'origine inclus tels quels (`include_str!`). Aucune fusion : le décodeur du moteur lit
/// ces fichiers comme il lit un dossier.
fn bundle_profiles() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    let profiles_dir = Path::new("../profiles");
    println!("cargo:rerun-if-changed={}", profiles_dir.display());

    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(profiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("profile.toml").exists() {
                dirs.push(path);
            }
        }
    }
    // Place reel_quebecois first as the engine's default, followed by others sorted by path.
    dirs.sort_by(|a, b| {
        let a_default = a.file_name().and_then(|n| n.to_str()) == Some("reel_quebecois");
        let b_default = b.file_name().and_then(|n| n.to_str()) == Some("reel_quebecois");
        match (a_default, b_default) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        }
    });

    let mut source = String::from("&[\n");
    for dir in &dirs {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
        source.push_str("    &[\n");
        for name in profile_files(dir)? {
            let path = std::fs::canonicalize(dir.join(&name))?;
            source.push_str(&format!(
                "        ({name:?}, include_str!({:?})),\n",
                path.display().to_string()
            ));
        }
        source.push_str("    ],\n");
    }
    source.push_str("]\n");
    std::fs::write(Path::new(&out_dir).join("distributed_profiles.rs"), source)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell cargo to re-run this if git HEAD changes
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs/heads/");
    println!("cargo:rerun-if-changed=../.git/refs/tags/");

    bundle_profiles()?;

    // Get git describe (tag + commits since tag)
    let git_tag = Command::new("git")
        .args(["describe", "--tags", "--always"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() { String::from_utf8(output.stdout).ok() } else { None }
        })
        .map_or_else(|| env!("CARGO_PKG_VERSION").to_string(), |s| s.trim().to_string());

    // Get short SHA
    let git_sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() { String::from_utf8(output.stdout).ok() } else { None }
        })
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string());

    // Set environment variables for use with env!() macro
    println!("cargo:rustc-env=GIT_VERSION_TAG={git_tag}");
    println!("cargo:rustc-env=GIT_VERSION_SHA={git_sha}");
    Ok(())
}

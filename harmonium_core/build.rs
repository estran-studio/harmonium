//! Build script for `harmonium_core`
//!
//! Captures git version information at compile time to avoid runtime git dependencies,
//! and bundles distributed style profiles into `$OUT_DIR/distributed_profiles.toml`.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

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

    let mut profile_strings = Vec::new();
    for dir in &dirs {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
        let profile_path = dir.join("profile.toml");
        let text = std::fs::read_to_string(&profile_path)?;
        let mut root: toml::Value = toml::from_str(&text)?;

        if let Some(files) = root.as_table_mut().and_then(|t| t.remove("files")) {
            if let Some(files_tbl) = files.as_table() {
                for (key, filename) in files_tbl {
                    let fname = filename
                        .as_str()
                        .ok_or_else(|| format!("files.{key} dans {}", profile_path.display()))?;
                    let target = match key.as_str() {
                        "cells" => "melody.cells",
                        "feet" => "percussion",
                        "arrangement" => "arrangement",
                        "dynamics" => "dynamics",
                        _ => return Err(format!("banque {key}").into()),
                    };
                    let bank_path = dir.join(fname);
                    let bank_text = std::fs::read_to_string(&bank_path)?;
                    let bank_val: toml::Value = toml::from_str(&bank_text)?;
                    if target == "melody.cells" {
                        if let Some(melody) =
                            root.get_mut("melody").and_then(toml::Value::as_table_mut)
                        {
                            melody.insert("cells".into(), bank_val);
                        }
                    } else if let Some(tbl) = root.as_table_mut() {
                        tbl.insert(target.into(), bank_val);
                    }
                }
            }
        }

        if let Some(intro) = root.as_table_mut().and_then(|table| table.remove("intro")) {
            if let Some(arrangement) =
                root.get_mut("arrangement").and_then(toml::Value::as_table_mut)
            {
                arrangement.insert("intro".into(), intro);
            }
        }

        let inlined = toml::to_string(&root)?;
        profile_strings.push(inlined);
    }

    let mut bundle = toml::map::Map::new();
    bundle.insert(
        "profiles".to_string(),
        toml::Value::Array(profile_strings.into_iter().map(toml::Value::String).collect()),
    );
    let bundle_toml = toml::to_string(&toml::Value::Table(bundle))?;
    let dest = Path::new(&out_dir).join("distributed_profiles.toml");
    std::fs::write(&dest, bundle_toml)?;
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

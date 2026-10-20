//! Contrat de `harmonium-cli --check-cells <fichier.toml>` : code de sortie 0/1 et messages lisibles.

use std::{path::PathBuf, process::Command};

fn embedded_bank_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../harmonium_core/src/trad/reel_cells.toml")
}

fn run_check(path: &std::path::Path) -> std::io::Result<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_harmonium-cli")).arg("--check-cells").arg(path).output()
}

#[test]
fn embedded_bank_is_valid() -> std::io::Result<()> {
    let out = run_check(&embedded_bank_path())?;
    assert_eq!(out.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("180 cellules"));
    Ok(())
}

#[test]
fn invalid_bank_exits_1_with_readable_error() -> std::io::Result<()> {
    let dir = std::env::temp_dir().join(format!("harmonium_check_cells_{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let bad = dir.join("bad.toml");
    std::fs::write(&bad, "cells = [{ role = \"run\", notes = \"1 2 3\" }]\n")?;

    let out = run_check(&bad)?;
    std::fs::remove_dir_all(&dir)?;

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("croches"), "stderr: {stderr}");
    Ok(())
}

#[test]
fn missing_file_exits_1() -> std::io::Result<()> {
    let out = run_check(std::path::Path::new("/nonexistent/cells.toml"))?;
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("lecture impossible"));
    Ok(())
}

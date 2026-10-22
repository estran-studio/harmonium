//! Contrat de `harmonium-cli --check-cells <fichier.toml>` : code de sortie 0/1 et messages lisibles.

use std::{path::PathBuf, process::Command};

fn embedded_bank_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../profiles/reel_quebecois/cells.toml")
}

fn run_check(path: &std::path::Path) -> std::io::Result<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_harmonium-cli")).arg("--check-cells").arg(path).output()
}

#[test]
fn embedded_bank_is_valid() -> std::io::Result<()> {
    let out = run_check(&embedded_bank_path())?;
    assert_eq!(out.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    Ok(())
}

#[test]
fn missing_file_exits_1() -> std::io::Result<()> {
    let out = run_check(std::path::Path::new("/nonexistent/cells.toml"))?;
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("lecture impossible"));
    Ok(())
}

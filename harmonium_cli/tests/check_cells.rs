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

/// Une banque dont les cellules n'ont pas la longueur du profil est refusée (régression :
/// le contrôle de longueur avait disparu avec les profils v2).
#[test]
fn wrong_cell_length_exits_1_with_readable_error() -> std::io::Result<()> {
    let dir = std::env::temp_dir().join(format!("harmonium_check_cells_{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let bad = dir.join("bad.toml");
    std::fs::write(&bad, "cells = [{ role = \"run\", notes = \"1 2 3\" }]\n")?;

    let out = run_check(&bad)?;
    std::fs::remove_dir_all(&dir)?;

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("attendu 8 croches"), "stderr: {stderr}");
    Ok(())
}

#[test]
fn all_distributed_profile_banks_are_valid() -> std::io::Result<()> {
    for profile in harmonium_core::StyleProfile::distributed() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../profiles/{}", profile.meta.id));
        let cells_path = dir.join("cells.toml");
        if cells_path.exists() {
            let out = Command::new(env!("CARGO_BIN_EXE_harmonium-cli"))
                .arg("--check-cells")
                .arg(&cells_path)
                .arg("--style")
                .arg(&profile.meta.id)
                .output()?;
            assert_eq!(
                out.status.code(),
                Some(0),
                "profile {} stderr: {}",
                profile.meta.id,
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
    Ok(())
}

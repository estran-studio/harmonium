//! Contrat de `harmonium-cli --export --bars N` : l'export se termine en temps borné, quelle que
//! soit la longueur demandée (BUG1 : au-delà de ~100 mesures, la génération devançait la timeline
//! de 100 mesures, évinçait des mesures jamais jouées et le rendu attendait indéfiniment).

use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Largement au-dessus du temps de rendu (quelques dizaines de secondes en debug, tous profils en
/// parallèle) : un dépassement est un blocage.
const MAX_EXPORT: Duration = Duration::from_secs(120);
const BARS: usize = 200;

fn scratch_dir(name: &str) -> std::io::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("harmonium_{name}_{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn spawn_export(style: &str, automation: Option<&Path>, midi: &Path) -> std::io::Result<Child> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_harmonium-cli"));
    command
        .args(["--export", "--seed", "3", "--bars", &BARS.to_string(), "--style", style])
        .arg("--record-midi")
        .arg(midi)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(automation) = automation {
        command.arg("--automation").arg(automation);
    }
    command.spawn()
}

/// Exporte tous les profils distribués en parallèle ; échoue si l'un d'eux ne se
/// termine pas dans `MAX_EXPORT` (les survivants sont tués).
fn export_all_profiles(automation: Option<&Path>, name: &str) -> std::io::Result<()> {
    let dir = scratch_dir(name)?;
    let profiles = harmonium_core::StyleProfile::distributed();
    assert!(!profiles.is_empty(), "aucun profil distribué à exporter");

    let mut running = Vec::new();
    for profile in profiles {
        let midi = dir.join(format!("{}.mid", profile.meta.id));
        running.push((
            profile.meta.id.clone(),
            midi.clone(),
            spawn_export(&profile.meta.id, automation, &midi)?,
        ));
    }

    let start = Instant::now();
    let mut stuck = Vec::new();
    let mut failed = Vec::new();
    for (id, midi, child) in &mut running {
        loop {
            if let Some(status) = child.try_wait()? {
                if !status.success() {
                    failed.push(format!("{id} ({status})"));
                } else if std::fs::metadata(&midi)?.len() == 0 {
                    failed.push(format!("{id} (fichier MIDI vide)"));
                }
                break;
            }
            if start.elapsed() > MAX_EXPORT {
                child.kill()?;
                child.wait()?;
                stuck.push(id.clone());
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    std::fs::remove_dir_all(&dir)?;
    assert!(
        stuck.is_empty(),
        "export de {BARS} mesures non terminé après {MAX_EXPORT:?} : {stuck:?}"
    );
    assert!(failed.is_empty(), "export de {BARS} mesures en échec : {failed:?}");
    Ok(())
}

#[test]
fn export_without_automation_terminates_past_the_tune_change() -> std::io::Result<()> {
    export_all_profiles(None, "export_plain")
}

#[test]
fn export_with_constant_automation_terminates_past_the_tune_change() -> std::io::Result<()> {
    let dir = scratch_dir("export_auto_keys")?;
    let automation = dir.join("constant.toml");
    std::fs::write(&automation, "[[keys]]\nbar = 1\nintensity = 0.5\n")?;
    export_all_profiles(Some(&automation), "export_auto")?;
    std::fs::remove_dir_all(&dir)
}

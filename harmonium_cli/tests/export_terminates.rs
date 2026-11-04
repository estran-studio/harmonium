//! Contrat de `harmonium-cli --export --bars N` : l'export se termine en temps borné, quelle que
//! soit la longueur demandée (BUG1 : au-delà de ~100 mesures, la génération devançait la timeline
//! de 100 mesures, évinçait des mesures jamais jouées et le rendu attendait indéfiniment).

use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Largement au-dessus du temps de rendu d'un export (quelques dizaines de secondes en debug, au
/// plus un export par cœur, pendant le reste de la suite) : un dépassement est un blocage. Le délai
/// court depuis le lancement de chaque export, pour que le nombre de profils distribués ne change
/// pas ce que le test mesure.
const MAX_EXPORT: Duration = Duration::from_secs(300);
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

/// Exporte tous les profils distribués, au plus un par cœur à la fois ; échoue si l'un d'eux ne se
/// termine pas dans `MAX_EXPORT` après son lancement (il est alors tué).
fn export_all_profiles(automation: Option<&Path>, name: &str) -> std::io::Result<()> {
    let dir = scratch_dir(name)?;
    let profiles = harmonium_core::StyleProfile::distributed();
    assert!(!profiles.is_empty(), "aucun profil distribué à exporter");
    // Deux tests de ce fichier tournent en même temps : chacun prend la moitié des cœurs.
    let slots = std::thread::available_parallelism().map_or(2, |n| (n.get() / 2).max(1));

    let mut pending = profiles.iter().map(|p| p.meta.id.clone()).collect::<Vec<_>>();
    pending.reverse();
    let mut running: Vec<(String, PathBuf, Child, Instant)> = Vec::new();
    let mut stuck = Vec::new();
    let mut failed = Vec::new();
    while !pending.is_empty() || !running.is_empty() {
        while running.len() < slots
            && let Some(id) = pending.pop()
        {
            let midi = dir.join(format!("{id}.mid"));
            let child = spawn_export(&id, automation, &midi)?;
            running.push((id, midi, child, Instant::now()));
        }
        let mut index = 0;
        while index < running.len() {
            let (id, midi, child, started) = &mut running[index];
            if let Some(status) = child.try_wait()? {
                if !status.success() {
                    failed.push(format!("{id} ({status})"));
                } else if std::fs::metadata(&midi)?.len() == 0 {
                    failed.push(format!("{id} (fichier MIDI vide)"));
                }
                running.swap_remove(index);
            } else if started.elapsed() > MAX_EXPORT {
                child.kill()?;
                child.wait()?;
                stuck.push(id.clone());
                running.swap_remove(index);
            } else {
                index += 1;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
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

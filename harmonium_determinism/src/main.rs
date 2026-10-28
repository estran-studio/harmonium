//! Écrit les snapshots natifs : `determinism_dump <dossier>` → un fichier par cas, plus
//! un fichier par sonde. Le dossier miroir côté wasm vient de `scripts/dump_wasm.mjs`.

use std::{env, fs, path::PathBuf, process::ExitCode};

use harmonium_determinism::{PROBES, cases, run_case};

fn main() -> ExitCode {
    let Some(dir) = env::args().nth(1).map(PathBuf::from) else {
        eprintln!("usage : determinism_dump <dossier>");
        return ExitCode::FAILURE;
    };
    match dump(&dir) {
        Ok(count) => {
            println!("{count} fichiers écrits dans {}", dir.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("échec : {error}");
            ExitCode::FAILURE
        }
    }
}

fn dump(dir: &PathBuf) -> Result<usize, String> {
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let cases = cases();
    for case in &cases {
        let json = run_case(case).map_err(|error| format!("{}: {error}", case.file_name()))?;
        fs::write(dir.join(case.file_name()), json).map_err(|error| error.to_string())?;
    }
    for (name, probe) in PROBES {
        fs::write(dir.join(name), probe()?).map_err(|error| error.to_string())?;
    }
    Ok(cases.len() + PROBES.len())
}

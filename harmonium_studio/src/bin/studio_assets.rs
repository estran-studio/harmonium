//! `studio_assets` : prépare le dossier de ressources du studio (manifeste et fichiers à empreinte).
//!
//! ```text
//! studio_assets --main-bank <fichier> [--feet-bank <fichier>] [--odin-dir <dossier>] --out <dossier>
//! ```

use std::{path::PathBuf, process::ExitCode};

use harmonium_studio::packager::{PackSpec, pack};

const USAGE: &str = "usage: studio_assets --main-bank <file> [--feet-bank <file>] \
                     [--odin-dir <dir>] --out <dir>";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("studio_assets: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut main_bank = None;
    let mut feet_bank = None;
    let mut odin_dir = None;
    let mut out = None;
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value =
            || args.next().map(PathBuf::from).ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--main-bank" => main_bank = Some(value()?),
            "--feet-bank" => feet_bank = Some(value()?),
            "--odin-dir" => odin_dir = Some(value()?),
            "--out" => out = Some(value()?),
            _ => return Err(format!("unknown argument `{flag}`\n{USAGE}")),
        }
    }
    let main_bank = main_bank.ok_or_else(|| format!("--main-bank is required\n{USAGE}"))?;
    let out = out.ok_or_else(|| format!("--out is required\n{USAGE}"))?;

    let manifest = pack(&PackSpec { main_bank, feet_bank, odin_dir }, &out)?;
    for resource in &manifest.resources {
        println!("{:<16} {:>10} bytes  {}", resource.id, resource.bytes, resource.file);
    }
    println!("{} resource(s) in {}", manifest.resources.len(), out.display());
    Ok(())
}

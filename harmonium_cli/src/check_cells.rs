//! `--check-cells <fichier.toml>` : validation d'une banque de cellules sans lancer le moteur.

use harmonium_core::trad::cells::{CellBank, CellRole};

/// Charge `path` via [`CellBank::from_toml`] ; retourne le code de sortie (0 valide, 1 invalide).
///
/// Les erreurs (lecture du fichier, TOML, cellule invalide) sont écrites sur stderr ;
/// un résumé par rôle est écrit sur stdout en cas de succès.
pub fn run(path: &str) -> i32 {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("{path} : lecture impossible : {err}");
            return 1;
        }
    };
    match CellBank::from_toml(&content) {
        Ok(bank) => {
            println!("{path} : OK, {} cellules", bank.cells.len());
            for role in CellRole::ALL {
                println!("  {role:?} : {}", bank.count_by_role(role));
            }
            0
        }
        Err(err) => {
            eprintln!("{path} : {err}");
            1
        }
    }
}

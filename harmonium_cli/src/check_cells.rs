//! `--check-cells <fichier.toml>` : validation d'une banque de cellules sans lancer le moteur.

use harmonium_core::trad::cells::{CellBank, CellRole};

/// Charge `path` via [`CellBank::from_toml`] et vérifie que chaque cellule compte
/// `cell_units` unités (longueur du profil de style visé) ; retourne le code de sortie
/// (0 valide, 1 invalide).
///
/// Les erreurs (lecture du fichier, TOML, cellule invalide, mauvaise longueur) sont écrites
/// sur stderr ; un résumé par rôle est écrit sur stdout en cas de succès.
pub fn run(path: &str, cell_units: usize) -> i32 {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("{path} : lecture impossible : {err}");
            return 1;
        }
    };
    let bank = match CellBank::from_toml(&content) {
        Ok(bank) => bank,
        Err(err) => {
            eprintln!("{path} : {err}");
            return 1;
        }
    };
    let wrong: Vec<(usize, usize)> = bank
        .cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.notes.len() != cell_units)
        .map(|(index, cell)| (index, cell.notes.len()))
        .collect();
    if !wrong.is_empty() {
        for (index, len) in &wrong {
            eprintln!(
                "{path} : cellule {index} : attendu {cell_units} croches par cellule, trouvé {len}"
            );
        }
        return 1;
    }
    println!("{path} : OK, {} cellules de {cell_units} croches", bank.cells.len());
    for role in CellRole::ALL {
        println!("  {role:?} : {}", bank.count_by_role(role));
    }
    0
}

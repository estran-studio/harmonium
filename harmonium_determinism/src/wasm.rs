//! Exports pour Node (`wasm-bindgen --target nodejs`) : `scripts/dump_wasm.mjs` les appelle et
//! écrit les mêmes fichiers que `determinism_dump`.

// Le code généré par `#[wasm_bindgen]` contient du `unsafe`.
#![allow(unsafe_code)]

use wasm_bindgen::prelude::*;

/// Noms de fichiers des cas, dans l'ordre de [`run_case`](crate::run_case) par indice.
#[wasm_bindgen]
pub fn case_names() -> Vec<String> {
    crate::cases().iter().map(crate::Case::file_name).collect()
}

/// Snapshots JSON du cas `index`.
#[wasm_bindgen]
pub fn run_case(index: usize) -> Result<String, JsError> {
    let cases = crate::cases();
    let case = cases.get(index).ok_or_else(|| JsError::new("cas inconnu"))?;
    crate::run_case(case).map_err(|error| JsError::new(&error))
}

/// Noms de fichiers des sondes, dans l'ordre de [`run_probe`].
#[wasm_bindgen]
pub fn probe_names() -> Vec<String> {
    crate::PROBES.iter().map(|(name, _)| (*name).to_string()).collect()
}

/// Texte de la sonde `index`.
#[wasm_bindgen]
pub fn run_probe(index: usize) -> Result<String, JsError> {
    let (_, probe) = crate::PROBES.get(index).ok_or_else(|| JsError::new("sonde inconnue"))?;
    probe().map_err(|error| JsError::new(&error))
}

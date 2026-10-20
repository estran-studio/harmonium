use std::path::Path;

/// Chemin vers la banque GM complète par défaut (GeneralUser GS).
pub const GENERALUSER_SOUNDFONT: &str = "assets/GeneralUser-GS.sf2";

/// Chemin vers la banque minimale de test.
pub const TEST_SOUNDFONT: &str = "assets/test.sf2";

/// Chemin vers la banque de pieds par défaut (harmonium_lab `make trad-sf2`).
pub const FEET_SOUNDFONT: &str = "assets/harmonium_trad.sf2";

/// Résout et charge la SoundFont selon la hiérarchie de repli :
/// 1. `--soundfont <PATH>` explicite si fourni (erreur si le fichier est illisible).
/// 2. `assets/GeneralUser-GS.sf2` s'il existe (recommandé pour le développement).
/// 3. `assets/test.sf2` s'il existe, avec un avertissement explicite indiquant
///    que les instruments trad et les pieds ne sonneront pas.
/// 4. Aucune banque (`Ok(None)`), le moteur tourne en silence.
///
/// # Errors
/// Retourne une `std::io::Error` si un chemin explicite ou un fichier détecté
/// ne peut pas être lu depuis le disque.
fn resolve_soundfont_path(rel: &str) -> Option<std::path::PathBuf> {
    let p = Path::new(rel);
    if p.is_file() {
        return Some(p.to_path_buf());
    }
    let parent_p = Path::new("..").join(rel);
    if parent_p.is_file() {
        return Some(parent_p);
    }
    None
}

pub fn load_soundfont(explicit: Option<&str>) -> std::io::Result<Option<Vec<u8>>> {
    let (path_buf, is_fallback_test) = match explicit {
        Some(p) => {
            let resolved = resolve_soundfont_path(p).unwrap_or_else(|| Path::new(p).to_path_buf());
            (resolved, false)
        }
        None => {
            if let Some(p) = resolve_soundfont_path(GENERALUSER_SOUNDFONT) {
                (p, false)
            } else if let Some(p) = resolve_soundfont_path(TEST_SOUNDFONT) {
                (p, true)
            } else {
                eprintln!(
                    "Avertissement : aucune SoundFont trouvée ({GENERALUSER_SOUNDFONT} et \
                     {TEST_SOUNDFONT} absents) ; le moteur tourne sans banque (silence)."
                );
                harmonium_core::log::warn(
                    "Aucune SoundFont trouvée ; le moteur tourne sans banque (silence).",
                );
                return Ok(None);
            }
        }
    };

    if is_fallback_test {
        eprintln!(
            "Avertissement : {GENERALUSER_SOUNDFONT} non trouvé, repli sur {TEST_SOUNDFONT}. \
             Les instruments trad (violon, guitare, basse) et les pieds (podorythmie) ne sonneront pas. \
             Exécutez 'make soundfont' pour installer la banque GM complète."
        );
        harmonium_core::log::warn(
            "assets/GeneralUser-GS.sf2 non trouvé, repli sur assets/test.sf2 : \
             les instruments trad et les pieds ne sonneront pas.",
        );
    }

    let display_path = path_buf.display().to_string();
    println!("Loading SoundFont: {display_path}");
    let bytes = std::fs::read(&path_buf)?;
    println!("Loaded SoundFont ({} bytes)", bytes.len());
    Ok(Some(bytes))
}

/// Charge la banque de pieds (`harmonium_trad.sf2`, SK2) :
/// 1. `--feet-soundfont <PATH>` explicite si fourni (erreur si le fichier est illisible).
/// 2. `assets/harmonium_trad.sf2` à côté de la SoundFont principale s'il existe.
/// 3. Aucune banque : message clair indiquant de la reconstruire
///    (`make trad-sf2` dans `harmonium_lab`) ; le moteur conserve alors le kit GM
///    (canal 9) pour les pieds.
///
/// # Errors
/// Retourne une `std::io::Error` si un chemin explicite ou un fichier détecté
/// ne peut pas être lu depuis le disque.
pub fn load_feet_soundfont(explicit: Option<&str>) -> std::io::Result<Option<Vec<u8>>> {
    match explicit {
        Some(p) => {
            let resolved = resolve_soundfont_path(p).unwrap_or_else(|| Path::new(p).to_path_buf());
            let bytes = std::fs::read(&resolved)?;
            println!("Loading feet SoundFont: {}", resolved.display());
            Ok(Some(bytes))
        }
        None => {
            if let Some(p) = resolve_soundfont_path(FEET_SOUNDFONT) {
                let bytes = std::fs::read(&p)?;
                println!("Loading feet SoundFont: {}", p.display());
                Ok(Some(bytes))
            } else {
                let msg = "banque des pieds absente : `make trad-sf2` dans harmonium_lab";
                eprintln!("Avertissement : {msg}");
                harmonium_core::log::warn(msg);
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_explicit_missing_file_returns_err() {
        let res = load_soundfont(Some("assets/does_not_exist_xyz.sf2"));
        assert!(res.is_err());
    }

    #[test]
    fn test_load_explicit_test_sf2_returns_bytes() {
        let res = load_soundfont(Some("assets/test.sf2"));
        assert!(res.is_ok());
        if let Ok(Some(bytes)) = res {
            assert_eq!(bytes.len(), 451592);
        } else {
            assert!(false, "test.sf2 doit être chargé avec succès");
        }
    }

    #[test]
    fn test_load_default_hierarchy() {
        let res = load_soundfont(None);
        assert!(res.is_ok());
        if let Ok(bytes) = res {
            assert!(bytes.is_some());
        }
    }

    #[test]
    fn test_load_feet_soundfont_explicit_missing_returns_err() {
        let res = load_feet_soundfont(Some("assets/does_not_exist_feet.sf2"));
        assert!(res.is_err());
    }

    #[test]
    fn test_load_feet_soundfont_default_never_errors() {
        // La banque de pieds est ignorée par git : selon sa présence, `Ok(None)`
        // (message d'absence) ou `Ok(Some(bytes))` — mais jamais une erreur.
        let res = load_feet_soundfont(None);
        assert!(res.is_ok());
    }
}

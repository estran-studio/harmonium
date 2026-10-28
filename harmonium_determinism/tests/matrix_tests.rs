//! La matrice du harnais couvre tous les profils distribués, et un cas rejoué donne les mêmes octets.

use std::collections::HashSet;

use harmonium_core::StyleProfile;
use harmonium_determinism::{BARS, EMOTIONS, PROBES, SEEDS, cases, run_case};

#[test]
fn matrix_covers_every_distributed_profile_seed_and_emotion_setting() {
    let cases = cases();
    assert_eq!(cases.len(), StyleProfile::distributed().len() * SEEDS.len() * EMOTIONS.len());
    let names: HashSet<String> = cases.iter().map(harmonium_determinism::Case::file_name).collect();
    assert_eq!(names.len(), cases.len(), "un fichier par cas");
}

#[test]
fn a_case_replayed_gives_the_same_bytes() -> Result<(), String> {
    let cases = cases();
    let case = cases.first().ok_or("aucun profil distribué")?;
    let first = run_case(case)?;
    assert_eq!(first, run_case(case)?);
    let snapshots: Vec<serde_json::Value> =
        serde_json::from_str(&first).map_err(|error| error.to_string())?;
    assert_eq!(snapshots.len(), BARS);
    Ok(())
}

#[test]
fn every_probe_answers() -> Result<(), String> {
    for (name, probe) in PROBES {
        let text = probe().map_err(|error| format!("{name}: {error}"))?;
        assert!(!text.is_empty(), "{name}");
    }
    Ok(())
}

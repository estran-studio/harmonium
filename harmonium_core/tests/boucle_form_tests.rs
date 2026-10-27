//! Forme en boucle (G4, phase 014) : boucle courte sans cadence, banques de
//! cellules exigées pour les seuls rôles employés, mélodie entrant tard via
//! l'arrangement comme les autres pistes. Tout passe par `generate_measure`
use harmonium_core::{
    StyleProfile, TuningParams,
    ornaments::{OrnamentKind, PhraseEnd},
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    trad::{ImposedRole, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn load(id: &str) -> Result<StyleProfile, Box<dyn std::error::Error>> {
    let path = format!("{}/../tests/fixtures/profiles/{id}", env!("CARGO_MANIFEST_DIR"));
    let profile = StyleProfile::load(&path)?;
    profile.validate()?;
    Ok(profile)
}

fn stream(profile: &StyleProfile, seed: u64, bars: usize) -> Vec<MeasureSnapshot> {
    let mut params = MusicalParams::default();
    params.bpm = profile.meter.default_bpm;
    params.swing_ratio = profile.meter.default_swing_ratio;
    let mut generator =
        TradGenerator::new(params, TuningParams { profile: profile.clone(), ..Default::default() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect()
}

fn melody(snapshot: &MeasureSnapshot) -> Vec<&NoteSnapshot> {
    snapshot.notes.iter().filter(|note| note.track == 0).collect()
}

fn has_track(snapshot: &MeasureSnapshot, track: u8) -> bool {
    snapshot.notes.iter().any(|note| note.track == track)
}

fn json(snapshots: &[MeasureSnapshot], bar: usize) -> serde_json::Value {
    serde_json::to_value(&snapshots[bar].notes).unwrap()
}

#[test]
fn boucle_quatre_mesures_melodie_entre_au_tour_2() -> Result<(), Box<dyn std::error::Error>> {
    // Se charge : banques `open` + `run` seules, `arp`/`turn` de poids nul non exigés.
    let profile = load("boucle_test")?;
    for seed in [1, 2] {
        let snapshots = stream(&profile, seed, 64);
        assert_eq!(snapshots.len(), 64, "seed {seed}");
        // Tour 1 (mesures 1-8) : pas de mélodie, accompagnement et pieds présents.
        for snapshot in snapshots.iter().take(8) {
            assert!(melody(snapshot).is_empty(), "mélodie au tour 1, seed {seed}");
            assert!(has_track(snapshot, 1), "accords au tour 1, seed {seed}");
            assert!(has_track(snapshot, 2), "basse au tour 1, seed {seed}");
            assert!(has_track(snapshot, 3), "pieds au tour 1, seed {seed}");
        }
        // Tours 2-8 : la mélodie joue chaque mesure.
        for snapshot in snapshots.iter().skip(8) {
            assert!(!melody(snapshot).is_empty(), "mélodie absente, seed {seed}");
        }
        // L'accord voté se répète à chaque tour, tour 1 sans mélodie compris.
        for tour in 1..7 {
            for bar in 0..4 {
                assert_eq!(
                    snapshots[tour * 8 + bar].chord_name,
                    snapshots[bar].chord_name,
                    "accord tour {}, seed {seed}",
                    tour + 1
                );
            }
        }
        // Pas de tenue cadentielle imposée : aucune note de mélodie ne tient
        // la demi-mesure dans les fins de boucle (hors seconde fin).
        for bar in [3, 7, 11, 15, 19, 23, 27, 31, 35, 39, 43, 47, 51, 55, 59] {
            for note in melody(&snapshots[bar]) {
                assert!(
                    note.duration_steps < 8,
                    "tenue {} pas mesure {}, seed {seed}",
                    note.duration_steps,
                    bar + 1
                );
            }
        }
        // Déterminisme complet, seconde fin du tour 8 comprise.
        assert_eq!(
            serde_json::to_value(&snapshots)?,
            serde_json::to_value(stream(&profile, seed, 64))?,
            "seed {seed}"
        );
    }
    Ok(())
}

#[test]
fn boucle_quatre_mesures_repetition_sans_ornements() -> Result<(), Box<dyn std::error::Error>> {
    let mut profile = load("boucle_test")?;
    profile.ornaments.types.clear();
    profile.validate()?;
    for seed in [1, 2] {
        let snapshots = stream(&profile, seed, 64);
        // Passes et tours 2-7 à l'identique (la seconde fin ne touche que la mesure 64).
        for bar in 0..4 {
            for other in [bar + 4, bar + 8, bar + 16, bar + 24, bar + 32, bar + 40, bar + 48] {
                assert_eq!(json(&snapshots, 8 + bar), json(&snapshots, 8 + other), "seed {seed}");
            }
        }
        // Seconde fin : mesures 61-63 identiques à la boucle, mesure 64 jouée.
        for bar in 0..3 {
            assert_eq!(json(&snapshots, 60 + bar), json(&snapshots, 8 + bar), "seed {seed}");
        }
        assert!(!melody(&snapshots[63]).is_empty(), "seconde fin muette, seed {seed}");
    }
    Ok(())
}

#[test]
fn boucle_gigue_deux_mesures_melodie_entre_passe_2() -> Result<(), Box<dyn std::error::Error>> {
    // Autre style (principe VIII) : 6/8, banques `open` + `run` + `turn`.
    let profile = load("boucle_gigue_test")?;
    for seed in [1, 2] {
        let snapshots = stream(&profile, seed, 64);
        assert_eq!(snapshots.len(), 64, "seed {seed}");
        for snapshot in snapshots.iter().take(2) {
            assert!(melody(snapshot).is_empty(), "mélodie passe 1, seed {seed}");
        }
        for snapshot in snapshots.iter().skip(2) {
            assert!(!melody(snapshot).is_empty(), "mélodie absente, seed {seed}");
        }
        // L'accord se répète toutes les 2 mesures sur les 64.
        for bar in 2..64 {
            assert_eq!(
                snapshots[bar].chord_name,
                snapshots[bar % 2].chord_name,
                "mesure {}, seed {seed}",
                bar + 1
            );
        }
    }
    let mut plain = profile.clone();
    plain.ornaments.types.clear();
    plain.validate()?;
    for seed in [1, 2] {
        let snapshots = stream(&plain, seed, 64);
        for bar in (4..64).step_by(2) {
            assert_eq!(
                json(&snapshots, 2),
                json(&snapshots, bar),
                "mesure {}, seed {seed}",
                bar + 1
            );
            assert_eq!(
                json(&snapshots, 3),
                json(&snapshots, bar + 1),
                "mesure {}, seed {seed}",
                bar + 2
            );
        }
    }
    Ok(())
}

#[test]
fn banques_exigees_pour_les_seuls_roles_employes() -> Result<(), Box<dyn std::error::Error>> {
    let mut profile = load("boucle_test")?;
    // Un poids positif sur un rôle sans banque est refusé en nommant les cellules.
    profile.melody.params.weight_role_arp = 2;
    assert_eq!(profile.validate().unwrap_err().field, "melody.cells.Arp");
    // Zéro partout sans rôle libre employé : valide ; avec `Free` : refusé.
    profile.melody.params.weight_role_arp = 0;
    profile.melody.params.weight_role_run = 0;
    assert_eq!(profile.validate().unwrap_err().field, "melody.params.weight_role_run");
    profile.form.parts[0].roles = vec![ImposedRole::Open.into(); 4];
    profile.form.parts[0].second_ending = None;
    profile.validate()?;
    Ok(())
}

#[test]
fn melodie_omise_ne_joue_jamais() -> Result<(), Box<dyn std::error::Error>> {
    let mut profile = load("boucle_test")?;
    profile.arrangement.entries.melody = None;
    profile.validate()?;
    for seed in [1, 2] {
        let snapshots = stream(&profile, seed, 64);
        assert!(
            snapshots.iter().all(|snapshot| melody(snapshot).is_empty()),
            "mélodie jouée sans entrée, seed {seed}"
        );
        assert!(
            snapshots.iter().all(|snapshot| has_track(snapshot, 1) && has_track(snapshot, 2)),
            "accompagnement coupé, seed {seed}"
        );
    }
    Ok(())
}

/// Notes de mélodie ajoutées par les ornements dans `bars` (1-based), sur `seeds` airs.
fn extras(profile: &StyleProfile, bars: &[usize], seeds: u64) -> usize {
    let mut plain = profile.clone();
    plain.ornaments.types.clear();
    let (mut extra, mut notes) = (0, 0);
    for seed in 0..seeds {
        let a = stream(profile, seed, 64);
        let b = stream(&plain, seed, 64);
        for &bar in bars {
            let (m, n) = (melody(&a[bar - 1]).len(), melody(&b[bar - 1]).len());
            extra += m.saturating_sub(n);
            notes += n;
        }
    }
    assert!(notes > 0, "aucune note porteuse");
    extra
}

#[test]
fn ornements_sans_protection_par_defaut_en_boucle() -> Result<(), Box<dyn std::error::Error>> {
    // `ornament_protected_bars` omis + aucun rôle cadentiel : la fin de boucle
    // (mesures 4, 8, …) reçoit des ornements.
    let profile = load("boucle_test")?;
    assert!(profile.form.parts[0].ornament_protected_bars.is_none());
    let finals: Vec<usize> = (1..=16).map(|tour| tour * 4).collect();
    assert!(extras(&profile, &finals, 6) > 0, "aucun ornement en fin de boucle");
    // …et la fin de phrase tombe sur la fin de boucle, nulle part ailleurs :
    // une règle `only` ne tire que sur les mesures 4, 8, ….
    let mut only = profile.clone();
    only.ornaments.types.get_mut(&OrnamentKind::GraceSingle).unwrap().phrase_end = PhraseEnd::Only;
    only.validate()?;
    let middles: Vec<usize> = (1..=64).filter(|bar| bar % 4 != 0).collect();
    assert!(extras(&only, &finals, 6) > 0, "aucun ornement en fin de phrase");
    assert_eq!(extras(&only, &middles, 6), 0, "fin de phrase hors boucle");
    Ok(())
}

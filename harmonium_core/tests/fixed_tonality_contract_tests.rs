//! Tonalité imposée par le profil : contrat lu exclusivement dans les snapshots.
use harmonium_core::{
    StyleProfile, TuningParams,
    params::MusicalParams,
    report::MeasureSnapshot,
    trad::{Mode, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn profile() -> TestResult<StyleProfile> {
    let mut p = StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/nt_pedale_boucle",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    // Changer d'air toutes les 12 mesures exerce aussi le choix de tonalité du set.
    p.arrangement.tours_per_tune = 1;
    p.dynamics.color[0].key = 2;
    Ok(p)
}

fn locked(p: &StyleProfile, key: u8, mode: &str) -> TestResult<StyleProfile> {
    Ok(StyleProfile::from_toml(&format!(
        "{}\n[fixed_tonality]\nkey = {key}\nmode = '{mode}'\n",
        p.to_toml()?
    ))?)
}

fn snapshots(p: &StyleProfile, seed: u64, valence: impl Fn(usize) -> f32) -> Vec<MeasureSnapshot> {
    let mut params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: valence(1),
        ..Default::default()
    };
    let mut generator = TradGenerator::new(params.clone(), TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=72)
        .map(|bar| {
            params.harmony_valence = valence(bar);
            generator.invalidate_future(bar - 1);
            generator.update_params(params.clone());
            MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng))
        })
        .collect()
}

fn harmonic_contract(snapshots: &[MeasureSnapshot]) -> serde_json::Value {
    serde_json::Value::Array(
        snapshots.iter().map(|s| serde_json::json!({
            "tonality": s.position.map(|p| (p.key, p.mode)),
            "chord": (&s.chord_name, s.chord_root_offset, s.chord_is_minor),
            "changes": s.chord_changes,
            "pitches": s.notes.iter().filter(|n| n.track == 1).map(|n| n.pitch).collect::<Vec<_>>(),
        })).collect(),
    )
}

#[test]
fn verrou_conserve_tonique_mode_et_accords_sur_valence_et_graines() -> TestResult {
    let p = profile()?;
    let valences = [-1.0, -0.75, -0.25, 0.0, p.dynamics.neutral_valence, 0.5, 0.75, 1.0];
    for (key, name, mode) in [(5, "phrygian", Mode::Phrygian), (11, "mixolydian", Mode::Mixolydian)]
    {
        let fixed = locked(&p, key, name)?;
        for seed in [1, 2, 17] {
            let reference =
                harmonic_contract(&snapshots(&fixed, seed, |_| p.dynamics.neutral_valence));
            for valence in valences {
                let actual = snapshots(&fixed, seed, |_| valence);
                for s in &actual {
                    assert_eq!(s.position.map(|p| (p.key, p.mode)), Some((key, mode)));
                }
                assert_eq!(
                    harmonic_contract(&actual),
                    reference,
                    "graine {seed}, valence {valence}"
                );
            }
            let swept = snapshots(&fixed, seed, |bar| valences[((bar - 1) / 3) % valences.len()]);
            assert_eq!(
                harmonic_contract(&swept),
                reference,
                "balayage en cours de jeu, graine {seed}"
            );
        }
    }
    // Même profil, champ absent : couleurs initiales et bascules restent actives.
    for seed in [1, 2, 17] {
        let dark = snapshots(&p, seed, |_| -1.0);
        let bright = snapshots(&p, seed, |bar| if bar <= 3 { -1.0 } else { 1.0 });
        let neutral = snapshots(&p, seed, |_| p.dynamics.neutral_valence);
        assert_eq!(dark[0].position.map(|p| (p.key, p.mode)), Some((2, Mode::Phrygian)));
        assert_eq!(neutral[0].position.map(|p| (p.key, p.mode)), Some((8, Mode::Phrygian)));
        assert!(bright.iter().any(|s| s.position.is_some_and(|p| p.mode == Mode::Mixolydian)));
        assert_ne!(harmonic_contract(&dark), harmonic_contract(&bright));
        assert_ne!(harmonic_contract(&dark), harmonic_contract(&neutral));
    }
    Ok(())
}

#[test]
fn verrou_refuse_tonique_invalide_mode_absent_et_champ_inconnu() -> TestResult {
    let p = profile()?;
    let base = p.to_toml()?;
    for fields in [
        "key = -1\nmode = 'ionian'",
        "key = 12\nmode = 'ionian'",
        "key = 256\nmode = 'ionian'",
        "key = 0\nmode = 'inconnu'",
        "key = 0",
        "mode = 'ionian'",
        "key = 0\nmode = 'ionian'\nweight = 1",
    ] {
        assert!(
            StyleProfile::from_toml(&format!("{base}\n[fixed_tonality]\n{fields}\n")).is_err(),
            "verrou invalide accepté : {fields}"
        );
    }
    let mut fixed = locked(&p, 0, "ionian")?;
    fixed.fixed_tonality.as_mut().ok_or("verrou absent")?.key = 12;
    assert_eq!(
        fixed.validate().err().ok_or("tonique invalide acceptée")?.field,
        "fixed_tonality.key"
    );
    Ok(())
}

#[test]
fn harmonie_exige_le_mode_verrouille_sans_exiger_sa_bascule() -> TestResult {
    let mut p = locked(&StyleProfile::default(), 0, "phrygian")?;
    p.harmony.candidates.retain(|c| c.mode == Mode::Phrygian);
    p.dynamics.parallel_modes.retain(|c| c.dark != Mode::Phrygian);
    p.validate()?;
    for s in snapshots(&p, 1, |bar| if bar < 12 { -1.0 } else { 1.0 }) {
        assert_eq!(s.position.map(|p| (p.key, p.mode)), Some((0, Mode::Phrygian)));
    }
    p.harmony.candidates.clear();
    assert_eq!(
        p.validate().err().ok_or("candidats du mode fixé absents acceptés")?.field,
        "harmony.candidates"
    );
    Ok(())
}

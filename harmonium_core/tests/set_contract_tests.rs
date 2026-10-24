//! Contrat de la forme longue (sets) observé sur le profil et le flux de snapshots.
use harmonium_core::{
    StyleProfile,
    trad::set::{MAX_SET_TUNES, SetLength},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn invalid_field(mutate: impl FnOnce(&mut StyleProfile)) -> Option<String> {
    let mut profile = StyleProfile::default();
    mutate(&mut profile);
    profile.validate().err().map(|error| error.field)
}

#[test]
fn invalid_set_is_refused_with_the_field_path() {
    let cases: &[(&str, fn(&mut StyleProfile))] = &[
        ("set.length[0].tunes", |p| p.set.length[0].tunes = 0),
        ("set.length[1].tunes", |p| p.set.length[1].tunes = MAX_SET_TUNES + 1),
        ("set.length", |p| p.set.length.iter_mut().for_each(|entry| entry.weight = 0)),
        ("set.length", |p| p.set.length = vec![SetLength { tunes: 2, weight: 1 }; 65]),
        ("set.key.same", |p| p.set.key.same = -0.1),
        ("set.key.step_down", |p| p.set.key.step_down = f32::NAN),
        ("set.key.last_dark", |p| p.set.key.last_dark = f32::INFINITY),
        ("set.key", |p| {
            p.set.key = harmonium_core::trad::set::SetKey {
                same: 0.0,
                parallel: 0.0,
                relative: 0.0,
                fifth_up: 0.0,
                fourth_up: 0.0,
                step_up: 0.0,
                step_down: 0.0,
                other: 0.0,
                last_dark: 1.0,
            }
        }),
    ];
    for &(field, mutate) in cases {
        assert_eq!(invalid_field(mutate).as_deref(), Some(field), "{field}");
    }
}

#[test]
fn set_file_errors_name_the_field() -> TestResult {
    let mut value: toml::Value = toml::from_str(&StyleProfile::default().to_toml()?)?;
    let set = value.get_mut("set").and_then(toml::Value::as_table_mut).ok_or("set absent")?;
    let key = set.get_mut("key").and_then(toml::Value::as_table_mut).ok_or("set.key absent")?;
    key.insert("same".into(), toml::Value::String("beaucoup".into()));
    let error = StyleProfile::from_toml(&toml::to_string(&value)?).err().ok_or("profil accepté")?;
    assert_eq!(error.field, "set.key.same");

    let mut value: toml::Value = toml::from_str(&StyleProfile::default().to_toml()?)?;
    let set = value.get_mut("set").and_then(toml::Value::as_table_mut).ok_or("set absent")?;
    set.insert("transitions".into(), toml::Value::Array(Vec::new()));
    let error = StyleProfile::from_toml(&toml::to_string(&value)?).err().ok_or("profil accepté")?;
    assert!(error.message.contains("transitions"), "{error}");
    Ok(())
}

#[test]
fn removed_chaining_and_intro_flags_are_refused() -> TestResult {
    for (table, key) in [("arrangement", "intro_each_tune"), ("form", "next_same_weight")] {
        let mut value: toml::Value = toml::from_str(&StyleProfile::default().to_toml()?)?;
        value
            .get_mut(table)
            .and_then(toml::Value::as_table_mut)
            .ok_or("table absente")?
            .insert(key.into(), toml::Value::Boolean(true));
        assert!(StyleProfile::from_toml(&toml::to_string(&value)?).is_err(), "{table}.{key}");
    }
    let mut value: toml::Value = toml::from_str(&StyleProfile::default().to_toml()?)?;
    value
        .get_mut("arrangement")
        .and_then(|a| a.get_mut("intro"))
        .and_then(toml::Value::as_table_mut)
        .ok_or("arrangement.intro absent")?
        .insert("each_tune".into(), toml::Value::Boolean(true));
    assert!(StyleProfile::from_toml(&toml::to_string(&value)?).is_err(), "intro.each_tune");
    Ok(())
}

#[test]
fn distributed_profiles_load_their_set_file() {
    for profile in StyleProfile::distributed() {
        assert!(!profile.set.length.is_empty(), "{} : set.length vide", profile.meta.id);
    }
}

// ---------------------------------------------------------------------------
// Flux : tonalités publiées par air
// ---------------------------------------------------------------------------

use harmonium_core::{
    params::MusicalParams,
    report::MeasureSnapshot,
    trad::{Mode, TradGenerator, set::SetKey},
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const NEUTRAL: f32 = 0.3;

fn tuning(mutate: impl FnOnce(&mut StyleProfile)) -> TuningParams {
    let mut tuning = TuningParams::default();
    tuning.profile.arrangement.tours_per_tune = 1;
    mutate(&mut tuning.profile);
    tuning
}

fn session(seed: u64, tuning: &TuningParams, valence: f32, bars: usize) -> Vec<MeasureSnapshot> {
    let meter = &tuning.profile.meter;
    let mut params = MusicalParams::default();
    params.bpm = meter.default_bpm;
    params.set_swing_ratio(meter.default_swing_ratio);
    params.time_signature = meter.time_signature;
    params.harmony_valence = valence;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut generator = TradGenerator::new(params, tuning.clone());
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect()
}

/// `(key, mode)` de chaque air, lus à sa première mesure (l'intro, avant tout changement de mode).
fn tune_colors(snapshots: &[MeasureSnapshot]) -> Vec<(u8, Mode)> {
    let mut colors: Vec<(u8, Mode)> = Vec::new();
    let mut last_tune = None;
    for position in snapshots.iter().filter_map(|s| s.position) {
        if last_tune != Some(position.tune_index) {
            last_tune = Some(position.tune_index);
            colors.push((position.key, position.mode));
        }
    }
    colors
}

fn all_ones() -> SetKey {
    SetKey::default()
}

#[test]
fn published_colors_belong_to_the_candidates_of_the_valence() {
    let base = tuning(|p| p.set.length = vec![SetLength { tunes: 4, weight: 1 }]);
    let profile = &base.profile;
    for valence in [NEUTRAL, 0.6, -0.6] {
        let allowed: Vec<(u8, Mode)> = if valence == profile.dynamics.neutral_valence {
            profile.melody.key_modes.iter().map(|c| (c.key, c.mode)).collect()
        } else {
            let input = profile.dynamics.valence_input(valence);
            profile
                .dynamics
                .color
                .iter()
                .filter(|c| c.weight.eval(input) > 0.0)
                .map(|c| (c.key, c.mode))
                .collect()
        };
        let mut seen = 0;
        for seed in 1..=30 {
            for (index, color) in
                tune_colors(&session(seed, &base, valence, 8 * 33)).iter().enumerate()
            {
                assert!(
                    allowed.contains(color),
                    "seed {seed}, air {index}, valence {valence}: {color:?}"
                );
                seen += 1;
            }
        }
        assert!(seen >= 200, "valence {valence}: {seen} airs seulement");
    }
}

fn class_of(prev: (u8, Mode), next: (u8, Mode)) -> harmonium_core::trad::set::KeyClass {
    harmonium_core::trad::set::KeyClass::of(prev, next)
}

#[test]
fn class_factors_drive_the_chaining() {
    use harmonium_core::trad::set::KeyClass;
    let long_set = |key: SetKey| {
        tuning(|p| {
            p.set.length = vec![SetLength { tunes: MAX_SET_TUNES, weight: 1 }];
            p.set.key = key;
        })
    };
    let base = long_set(all_ones());
    let candidates: Vec<(u8, Mode)> =
        base.profile.melody.key_modes.iter().map(|c| (c.key, c.mode)).collect();

    // Tout à 0 sauf fifth_up : chaque transition est une quinte quand un candidat la permet.
    let fifth_only = long_set(SetKey {
        same: 0.0,
        parallel: 0.0,
        relative: 0.0,
        fifth_up: 1.0,
        fourth_up: 0.0,
        step_up: 0.0,
        step_down: 0.0,
        other: 0.0,
        last_dark: 1.0,
    });
    let (mut forced, mut free) = (0, 0);
    for seed in 1..=20 {
        let colors = tune_colors(&session(seed, &fifth_only, NEUTRAL, MAX_SET_TUNES * 33));
        for pair in colors.windows(2) {
            if candidates.iter().any(|&c| class_of(pair[0], c) == KeyClass::FifthUp) {
                assert_eq!(class_of(pair[0], pair[1]), KeyClass::FifthUp, "seed {seed}: {pair:?}");
                forced += 1;
            } else {
                assert!(candidates.contains(&pair[1]), "seed {seed}: {pair:?}");
                free += 1;
            }
        }
    }
    assert!(forced > 100 && free > 20, "forced {forced}, free {free}");

    // Tous à 1 : fréquences des airs 2 à 16 proches du poids de base des candidats.
    let total: f64 = base.profile.melody.key_modes.iter().map(|c| f64::from(c.weight)).sum();
    let mut counts = vec![0usize; candidates.len()];
    let mut airs = 0usize;
    for seed in 1..=40 {
        for color in tune_colors(&session(seed, &base, NEUTRAL, MAX_SET_TUNES * 33)).iter().skip(1)
        {
            if let Some(index) = candidates.iter().position(|c| c == color) {
                counts[index] += 1;
            }
            airs += 1;
        }
    }
    for (index, candidate) in base.profile.melody.key_modes.iter().enumerate() {
        let expected = f64::from(candidate.weight) / total;
        let observed = counts[index] as f64 / airs as f64;
        assert!(
            (observed - expected).abs() < 0.05,
            "{candidate:?} : {observed:.3} contre {expected:.3}"
        );
    }
}

#[test]
fn last_dark_keeps_the_last_tune_of_a_set_bright() {
    // Valence basse : les candidats sombres existent ; sets de 3 airs, dernier air = tune_index % 3 == 2.
    let dark_share = |last_dark: f32| {
        let tuning = tuning(|p| {
            p.set.length = vec![SetLength { tunes: 3, weight: 1 }];
            p.set.key.last_dark = last_dark;
        });
        let (mut last, mut dark) = (0, 0);
        for seed in 1..=40 {
            for (index, (_, mode)) in
                tune_colors(&session(seed, &tuning, -0.6, 9 * 33)).iter().enumerate()
            {
                if index % 3 == 2 {
                    last += 1;
                    dark += usize::from(mode.is_minor());
                }
            }
        }
        (last, dark)
    };
    let (last, dark) = dark_share(0.0);
    assert!(last >= 100);
    assert_eq!(dark, 0, "dernier air sombre malgré last_dark = 0");
    let (_, dark_neutral) = dark_share(1.0);
    assert!(
        dark_neutral > 10,
        "le contrôle (last_dark = 1) doit produire des derniers airs sombres"
    );
}

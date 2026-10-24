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
        ("set.signature.return_rate", |p| p.set.signature.return_rate = -0.1),
        ("set.signature.return_rate", |p| p.set.signature.return_rate = f32::NAN),
        ("set.signature.memory", |p| p.set.signature.memory = 17),
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
    set.insert("transitons".into(), toml::Value::Array(Vec::new()));
    let error = StyleProfile::from_toml(&toml::to_string(&value)?).err().ok_or("profil accepté")?;
    assert!(error.message.contains("transitons"), "{error}");
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
    tuning.profile.set.signature = Default::default();
    tuning.profile.set.ramp = None;
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

fn json(snapshots: &[MeasureSnapshot]) -> String {
    serde_json::to_string(snapshots).unwrap_or_default()
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

// ---------------------------------------------------------------------------
// Position publiée : set_index, tune_in_set, tunes_in_set
// ---------------------------------------------------------------------------

use harmonium_core::{timeline::TrackId, trad::TunePosition};

fn positions(snapshots: &[MeasureSnapshot]) -> Vec<TunePosition> {
    snapshots.iter().filter_map(|s| s.position).collect()
}

/// Un `(tune_index, set_index, tune_in_set, tunes_in_set)` par air, dans l'ordre.
fn tunes_of(snapshots: &[MeasureSnapshot]) -> Vec<(usize, usize, usize, usize)> {
    let mut tunes: Vec<(usize, usize, usize, usize)> = Vec::new();
    for p in positions(snapshots) {
        let tune = (p.tune_index, p.set_index, p.tune_in_set, p.tunes_in_set);
        if tunes.last().map(|t| t.0) != Some(p.tune_index) {
            tunes.push(tune);
        }
    }
    tunes
}

#[test]
fn set_position_walks_each_set_without_gap_or_return() {
    let tuning = tuning(|p| {
        p.set.length = vec![SetLength { tunes: 3, weight: 2 }, SetLength { tunes: 2, weight: 1 }];
    });
    for seed in 1..=8 {
        let all = positions(&session(seed, &tuning, NEUTRAL, 400));
        let first = all[0];
        assert_eq!(
            (first.tune_index, first.set_index, first.tune_in_set),
            (0, 0, 1),
            "seed {seed}"
        );
        for pair in all.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            assert!((1..=MAX_SET_TUNES).contains(&a.tunes_in_set), "seed {seed}: {a:?}");
            if a.tune_index == b.tune_index {
                assert_eq!(
                    (a.set_index, a.tune_in_set, a.tunes_in_set),
                    (b.set_index, b.tune_in_set, b.tunes_in_set),
                    "seed {seed}: changement dans un air"
                );
            } else if a.tune_in_set < a.tunes_in_set {
                assert_eq!(b.tune_index, a.tune_index + 1, "seed {seed}");
                assert_eq!(
                    (b.set_index, b.tune_in_set, b.tunes_in_set),
                    (a.set_index, a.tune_in_set + 1, a.tunes_in_set),
                    "seed {seed}: trou ou retour dans le set"
                );
            } else {
                assert_eq!(a.tune_in_set, a.tunes_in_set, "seed {seed}: set dépassé");
                assert_eq!(b.tune_index, a.tune_index + 1, "seed {seed}");
                assert_eq!(
                    (b.set_index, b.tune_in_set),
                    (a.set_index + 1, 1),
                    "seed {seed}: le set suivant doit recommencer à 1"
                );
            }
        }
        assert!(all.last().is_some_and(|p| p.set_index >= 1), "seed {seed}: moins de deux sets");
    }
}

#[test]
fn set_length_follows_the_configured_distribution() {
    let weights = [(2usize, 45u32), (3, 44), (4, 7), (5, 4)];
    let configured = tuning(|p| {
        p.set.length = weights.iter().map(|&(tunes, weight)| SetLength { tunes, weight }).collect();
    });
    let seeds = 500u64;
    let mut counts = [0usize; 6];
    for seed in 1..=seeds {
        let first = positions(&session(seed, &configured, NEUTRAL, 1))[0];
        counts[first.tunes_in_set] += 1;
    }
    let total: u32 = weights.iter().map(|&(_, weight)| weight).sum();
    for &(tunes, weight) in &weights {
        let expected = f64::from(weight) / f64::from(total);
        let observed = counts[tunes] as f64 / seeds as f64;
        // Tolérance : trois écarts-types d'un tirage binomial sur 500 graines (≈ 0,067 à p = 0,45).
        let tolerance = 3.0 * (expected * (1.0 - expected) / seeds as f64).sqrt();
        assert!(
            (observed - expected).abs() <= tolerance,
            "{tunes} airs : {observed:.3} contre {expected:.3} ± {tolerance:.3}"
        );
    }
    assert_eq!(counts[0] + counts[1], 0, "longueur hors distribution");

    let singles = tuning(|p| p.set.length = vec![SetLength { tunes: 1, weight: 1 }]);
    for seed in 1..=20 {
        let all = positions(&session(seed, &singles, NEUTRAL, 3 * 33));
        assert!(all.iter().all(|p| p.tunes_in_set == 1 && p.tune_in_set == 1), "seed {seed}");
        assert_eq!(all.last().map(|p| p.set_index), all.last().map(|p| p.tune_index));
    }
}

#[test]
fn same_seed_gives_the_same_sets_and_seek_replays_them() {
    let tuning = tuning(|p| p.set.length = vec![SetLength { tunes: 3, weight: 1 }]);
    let run = |seed: u64| session(seed, &tuning, NEUTRAL, 300);
    for seed in [1, 2, 3] {
        let (a, b) = (run(seed), run(seed));
        assert_eq!(json(&a), json(&b), "seed {seed}: flux différents");
        let shape = |s: &[MeasureSnapshot]| -> Vec<_> {
            positions(s).iter().map(|p| (p.set_index, p.tune_in_set, p.key, p.mode)).collect()
        };
        assert_eq!(shape(&a), shape(&b));
        assert_ne!(json(&run(seed + 100)), json(&a), "deux graines identiques ?");
    }

    // Un rejeu silencieux jusqu'à la mesure n redonne le flux continu.
    let meter = &tuning.profile.meter;
    let mut params = MusicalParams::default();
    params.bpm = meter.default_bpm;
    params.set_swing_ratio(meter.default_swing_ratio);
    params.time_signature = meter.time_signature;
    let continuous = run(5);
    for target in [40usize, 101, 140] {
        let mut rng = ChaCha8Rng::seed_from_u64(5);
        let mut generator = TradGenerator::new(params.clone(), tuning.clone());
        generator.silent_advance(target, &mut rng);
        let replay: Vec<_> = (target..=300)
            .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
            .collect();
        assert_eq!(json(&replay), json(&continuous[target - 1..]), "seek à la mesure {target}");
    }
}

#[test]
fn a_valence_change_keeps_the_set_plan_and_follows_the_new_candidates() {
    let tuning = tuning(|p| p.set.length = vec![SetLength { tunes: 5, weight: 1 }]);
    let meter = &tuning.profile.meter;
    let mut params = MusicalParams::default();
    params.bpm = meter.default_bpm;
    params.set_swing_ratio(meter.default_swing_ratio);
    params.time_signature = meter.time_signature;
    let color_input = tuning.profile.dynamics.valence_input(-0.6);
    let dark: Vec<(u8, Mode)> = tuning
        .profile
        .dynamics
        .color
        .iter()
        .filter(|c| c.weight.eval(color_input) > 0.0)
        .map(|c| (c.key, c.mode))
        .collect();

    let mut changed_airs = 0;
    for seed in 1..=12 {
        let control = tunes_of(&session(seed, &tuning, NEUTRAL, 400));

        // Le jeu prend la valence −0,6 pendant l'air 2 puis jette les airs encore futurs.
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut generator = TradGenerator::new(params.clone(), tuning.clone());
        let mut stream: Vec<MeasureSnapshot> = Vec::new();
        let mut bar = 1;
        let change_bar = loop {
            let snapshot =
                MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng));
            let tune = snapshot.position.map_or(0, |p| p.tune_index);
            stream.push(snapshot);
            if tune == 1
                && stream.iter().filter_map(|s| s.position).filter(|p| p.tune_index == 1).count()
                    == 20
            {
                break bar;
            }
            bar += 1;
        };
        let mut hot = params.clone();
        hot.harmony_valence = -0.6;
        generator.update_params(hot);
        generator.invalidate_future(change_bar);
        stream.extend(
            (change_bar + 1..=400)
                .map(|b| MeasureSnapshot::from_measure(&generator.generate_measure(b, &mut rng))),
        );
        let after = tunes_of(&stream);

        let common = control.len().min(after.len());
        assert!(common >= 8, "seed {seed}: {common} airs seulement");
        assert_eq!(control[..common], after[..common], "seed {seed}: plan du set modifié");
        let colors = tune_colors(&stream);
        let before = tune_colors(&session(seed, &tuning, NEUTRAL, 400));
        assert_eq!(colors[..2], before[..2], "seed {seed}: airs déjà construits modifiés");
        for (index, color) in colors.iter().enumerate().skip(2) {
            assert!(
                dark.contains(color),
                "seed {seed}, air {index}: {color:?} hors [[color]] à −0,6"
            );
            changed_airs += usize::from(
                !tuning.profile.melody.key_modes.iter().any(|c| (c.key, c.mode) == *color),
            );
        }
    }
    assert!(
        changed_airs > 0,
        "aucune tonalité propre à la valence −0,6 : le contrôle est sans portée"
    );
}

#[test]
fn a_profile_without_set_is_one_endless_set() {
    let endless = tuning(|p| p.set = Default::default());
    let snapshots = session(3, &endless, 0.6, 3 * 34);
    let all = positions(&snapshots);
    assert!(all.iter().all(|p| p.set_index == 0 && p.tunes_in_set == 0), "pas de set fini");
    assert!(all.iter().all(|p| p.tune_in_set == p.tune_index + 1), "tune_in_set compte les airs");
    // Intro complète et entrées progressives au seul premier air de la session.
    let intro_bars =
        |tune: usize| all.iter().filter(|p| p.tune_index == tune && p.is_intro()).count();
    assert!(intro_bars(0) >= 1);
    assert_eq!(intro_bars(1), 1, "le 2ᵉ air n'a que la levée");
    let second_tour = snapshots
        .iter()
        .find(|s| s.position.is_some_and(|p| p.tune_index == 1 && !p.is_intro()))
        .ok_or("2ᵉ air absent");
    let playing = |snapshot: &MeasureSnapshot, track: TrackId| {
        snapshot.notes.iter().any(|n| n.track == track.channel())
    };
    let first_bar = second_tour.map(|s| (playing(s, TrackId::Bass), playing(s, TrackId::Heel)));
    assert_eq!(first_bar, Ok((true, true)), "toutes les pistes dès A1 au 2ᵉ air");
}

#[test]
fn old_snapshots_without_set_fields_still_load() -> TestResult {
    let tuning = tuning(|p| p.set.length = vec![SetLength { tunes: 2, weight: 1 }]);
    let snapshots = session(2, &tuning, NEUTRAL, 40);
    let mut json = serde_json::to_value(&snapshots)?;
    for snapshot in json.as_array_mut().ok_or("flux attendu")? {
        let position = snapshot
            .get_mut("position")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or("position attendue")?;
        for field in ["set_index", "tune_in_set", "tunes_in_set"] {
            assert!(position.remove(field).is_some(), "{field} absent du JSON publié");
        }
    }
    let old: Vec<MeasureSnapshot> = serde_json::from_value(json)?;
    for (old, new) in old.iter().zip(&snapshots) {
        let (Some(old), Some(new)) = (old.position, new.position) else {
            return Err("position".into());
        };
        assert_eq!((old.set_index, old.tune_in_set, old.tunes_in_set), (0, 0, 0));
        assert_eq!(
            (old.tune_index, old.tour, old.key, old.mode),
            (new.tune_index, new.tour, new.key, new.mode)
        );
    }
    assert_eq!(harmonium_core::tuning::TUNING_VERSION, 19);
    Ok(())
}

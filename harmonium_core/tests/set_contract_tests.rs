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

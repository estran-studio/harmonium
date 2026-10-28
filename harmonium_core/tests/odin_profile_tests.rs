//! Validation de la section `[odin.<Piste>]` d'un profil : le programme
//! SoundFont reste requis (c'est le rendu sans `odin`), le preset est un nom
//! fourni par l'hôte, les règles de morphing sont bornées et uniques.
use harmonium_core::{
    StyleProfile,
    odin::{ODIN_MAX_RULES, OdinMorphParam, OdinMorphRule, OdinTrackSpec},
    timeline::TrackId,
};

fn rule(param: OdinMorphParam) -> OdinMorphRule {
    OdinMorphRule { param, arousal: 1.0, valence: 0.0, density: 0.0, tension: 0.0 }
}

fn with(track: TrackId, spec: OdinTrackSpec) -> StyleProfile {
    let mut profile = StyleProfile::builtin().clone();
    profile.odin.insert(track, spec);
    profile
}

fn error_field(profile: &StyleProfile) -> String {
    profile.validate().unwrap_err().field
}

#[test]
fn a_valid_odin_track_declared_in_toml_is_accepted() {
    let text = r#"
        [odin.Bass]
        preset = "basse_saturee"
        morph = [{ param = "filter_frequency", arousal = 3500.0, tension = 1500.0 }]
    "#;
    let spec: toml::Table = toml::from_str(text).unwrap();
    let bass: OdinTrackSpec = spec["odin"]["Bass"].clone().try_into().unwrap();
    let profile = with(TrackId::Bass, bass);
    profile.validate().unwrap();
    assert_eq!(profile.odin[&TrackId::Bass].morph[0].tension, 1500.0);
}

#[test]
fn the_soundfont_program_is_required_for_an_odin_track() {
    let mut profile =
        with(TrackId::Bass, OdinTrackSpec { preset: "basse".into(), morph: Vec::new() });
    profile.validate().unwrap();
    profile.instruments.bass_program = None;
    assert_eq!(error_field(&profile), "odin.Bass");
}

#[test]
fn preset_names_are_plain_and_bounded() {
    for name in ["", "dossier/nappe", "dossier\\nappe", &"x".repeat(65)] {
        let profile = with(TrackId::Bass, OdinTrackSpec { preset: name.into(), morph: vec![] });
        assert_eq!(error_field(&profile), "odin.Bass.preset", "{name:?}");
    }
}

#[test]
fn morph_rules_are_bounded_unique_and_finite() {
    let too_many = vec![rule(OdinMorphParam::FilterFrequency); ODIN_MAX_RULES + 1];
    let profile = with(TrackId::Bass, OdinTrackSpec { preset: "b".into(), morph: too_many });
    assert_eq!(error_field(&profile), "odin.Bass.morph");

    let duplicate = vec![rule(OdinMorphParam::FilterFrequency); 2];
    let profile = with(TrackId::Bass, OdinTrackSpec { preset: "b".into(), morph: duplicate });
    assert_eq!(error_field(&profile), "odin.Bass.morph");

    let mut nan = rule(OdinMorphParam::AmpRelease);
    nan.density = f32::NAN;
    let profile = with(TrackId::Bass, OdinTrackSpec { preset: "b".into(), morph: vec![nan] });
    assert_eq!(error_field(&profile), "odin.Bass.morph");
}

//! Contrat des zones Odin : défaut final, bornes d'émotion, niveau et morphing
//! par zone. Le programme SoundFont reste requis pour le rendu sans `odin`.
use harmonium_core::{
    StyleProfile,
    host_resources::{HostResources, MissingResource},
    odin::{ODIN_MAX_RULES, OdinMorphParam, OdinMorphRule, OdinTrackSpec, OdinZone, OdinZoneWhen},
    timeline::TrackId,
};

fn rule(param: OdinMorphParam) -> OdinMorphRule {
    OdinMorphRule { param, arousal: 1.0, valence: 0.0, density: 0.0, tension: 0.0 }
}

fn zone(preset: &str, morph: Vec<OdinMorphRule>) -> OdinZone {
    OdinZone { when: None, preset: preset.into(), level_db: 0.0, morph }
}

fn single(preset: &str, morph: Vec<OdinMorphRule>) -> OdinTrackSpec {
    OdinTrackSpec { zones: vec![zone(preset, morph)] }
}

fn bounded(when: OdinZoneWhen) -> OdinZone {
    OdinZone { when: Some(when), ..zone("b", vec![]) }
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
        [[odin.Bass.zones]]
        preset = "basse_saturee"
        morph = [{ param = "fil1_freq", arousal = 3500.0, tension = 1500.0 }]
    "#;
    let spec: toml::Table = toml::from_str(text).unwrap();
    let bass: OdinTrackSpec = spec["odin"]["Bass"].clone().try_into().unwrap();
    let profile = with(TrackId::Bass, bass);
    profile.validate().unwrap();
    assert_eq!(profile.odin[&TrackId::Bass].zones[0].morph[0].tension, 1500.0);
    assert_eq!(profile.odin[&TrackId::Bass].zones[0].level_db, 0.0);
}

#[test]
fn the_soundfont_program_is_required_for_an_odin_track() {
    let mut profile = with(TrackId::Bass, single("basse", Vec::new()));
    profile.validate().unwrap();
    profile.instruments.bass_program = None;
    assert_eq!(error_field(&profile), "odin.Bass");
}

#[test]
fn preset_names_are_plain_and_bounded() {
    for name in ["", "dossier/nappe", "dossier\\nappe", &"x".repeat(65)] {
        let profile = with(TrackId::Bass, single(name, vec![]));
        assert_eq!(error_field(&profile), "odin.Bass.zones[0].preset", "{name:?}");
    }
}

#[test]
fn morph_rules_are_bounded_unique_and_finite() {
    let too_many = vec![rule(OdinMorphParam::Fil1Freq); ODIN_MAX_RULES + 1];
    let profile = with(TrackId::Bass, single("b", too_many));
    assert_eq!(error_field(&profile), "odin.Bass.zones[0].morph");

    let duplicate = vec![rule(OdinMorphParam::Fil1Freq); 2];
    let profile = with(TrackId::Bass, single("b", duplicate));
    assert_eq!(error_field(&profile), "odin.Bass.zones[0].morph");

    let mut nan = rule(OdinMorphParam::Env1Release);
    nan.density = f32::NAN;
    let profile = with(TrackId::Bass, single("b", vec![nan]));
    assert_eq!(error_field(&profile), "odin.Bass.zones[0].morph");
}

#[test]
fn obsolete_parameter_names_are_rejected() {
    for param in [
        "filter_frequency",
        "filter_resonance",
        "filter_env_amount",
        "amp_attack",
        "amp_release",
        "filter_sustain",
        "osc_detune",
        "osc1_volume",
        "master_volume",
        "reverb_drywet",
        "dist_threshold",
    ] {
        let text = format!("param = {param:?}");
        assert!(toml::from_str::<OdinMorphRule>(&text).is_err(), "{param}");
    }
}

#[test]
fn sixteen_distinct_catalogue_rules_are_accepted() {
    let morph = OdinMorphParam::ALL.into_iter().take(ODIN_MAX_RULES).map(rule).collect();
    with(TrackId::Chords, single("nappe", morph)).validate().unwrap();
}

#[test]
fn direct_preset_and_unknown_zone_fields_are_rejected() {
    assert!(toml::from_str::<OdinTrackSpec>("preset = \"b\"").is_err());
    assert!(toml::from_str::<OdinTrackSpec>("zones = [{ preset = \"b\", gain = 1.0 }]").is_err());
    assert!(
        toml::from_str::<OdinTrackSpec>(
            "zones = [{ preset = \"b\", when = { excitement = [0.0, 1.0] } }]"
        )
        .is_err()
    );
}

#[test]
fn one_to_four_zones_and_only_a_final_default_are_required() {
    for count in [0, 5] {
        let spec = OdinTrackSpec { zones: vec![zone("b", vec![]); count] };
        assert_eq!(error_field(&with(TrackId::Bass, spec)), "odin.Bass.zones");
    }
    let condition = OdinZoneWhen { tension: Some([0.6, 1.0]), ..OdinZoneWhen::default() };
    let spec = OdinTrackSpec { zones: vec![bounded(condition)] };
    assert_eq!(error_field(&with(TrackId::Bass, spec)), "odin.Bass.zones[0].when");
    let spec = OdinTrackSpec { zones: vec![zone("b", vec![]), zone("b", vec![])] };
    assert_eq!(error_field(&with(TrackId::Bass, spec)), "odin.Bass.zones[0].when");
    let spec = OdinTrackSpec {
        zones: vec![bounded(condition), bounded(condition), bounded(condition), zone("b", vec![])],
    };
    with(TrackId::Bass, spec).validate().unwrap();
}

#[test]
fn zone_conditions_cannot_be_empty() {
    let spec = OdinTrackSpec { zones: vec![bounded(OdinZoneWhen::default()), zone("b", vec![])] };
    assert_eq!(error_field(&with(TrackId::Bass, spec)), "odin.Bass.zones[0].when");
}

#[test]
fn emotion_bounds_are_finite_ordered_and_in_their_domains() {
    for axis in ["arousal", "valence", "density", "tension"] {
        for bounds in [
            [f32::NAN, 1.0],
            [0.0, f32::INFINITY],
            [f32::NEG_INFINITY, 0.0],
            [0.8, 0.2],
            [-1.01, 0.5],
            [0.0, 1.01],
        ] {
            let mut when = OdinZoneWhen::default();
            match axis {
                "arousal" => when.arousal = Some(bounds),
                "valence" => when.valence = Some(bounds),
                "density" => when.density = Some(bounds),
                _ => when.tension = Some(bounds),
            }
            let spec = OdinTrackSpec { zones: vec![bounded(when), zone("b", vec![])] };
            assert_eq!(
                error_field(&with(TrackId::Bass, spec)),
                format!("odin.Bass.zones[0].when.{axis}"),
            );
        }
    }
    for when in [
        OdinZoneWhen { arousal: Some([-0.01, 1.0]), ..OdinZoneWhen::default() },
        OdinZoneWhen { density: Some([-0.01, 1.0]), ..OdinZoneWhen::default() },
        OdinZoneWhen { tension: Some([-0.01, 1.0]), ..OdinZoneWhen::default() },
    ] {
        let spec = OdinTrackSpec { zones: vec![bounded(when), zone("b", vec![])] };
        assert!(with(TrackId::Bass, spec).validate().is_err());
    }
    let when = OdinZoneWhen {
        arousal: Some([0.0, 1.0]),
        valence: Some([-1.0, 1.0]),
        density: Some([0.5, 0.5]),
        tension: Some([1.0, 1.0]),
    };
    with(TrackId::Bass, OdinTrackSpec { zones: vec![bounded(when), zone("b", vec![])] })
        .validate()
        .unwrap();
}

#[test]
fn zone_levels_are_finite_and_bounded_and_morph_rules_are_local() {
    for level_db in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -60.1, 12.1] {
        let mut spec = single("b", vec![]);
        spec.zones[0].level_db = level_db;
        assert_eq!(error_field(&with(TrackId::Bass, spec)), "odin.Bass.zones[0].level_db");
    }
    for level_db in [-60.0, 0.0, 12.0] {
        let mut spec = single("b", vec![]);
        spec.zones[0].level_db = level_db;
        with(TrackId::Bass, spec).validate().unwrap();
    }
    let mut active = bounded(OdinZoneWhen { tension: Some([0.6, 1.0]), ..OdinZoneWhen::default() });
    active.morph = vec![rule(OdinMorphParam::Fil1Freq)];
    let spec = OdinTrackSpec {
        zones: vec![active.clone(), zone("b", vec![rule(OdinMorphParam::Fil1Freq)])],
    };
    with(TrackId::Bass, spec).validate().unwrap();
    active.morph[0].tension = f32::INFINITY;
    let spec = OdinTrackSpec { zones: vec![active, zone("b", vec![])] };
    assert_eq!(error_field(&with(TrackId::Bass, spec)), "odin.Bass.zones[0].morph");
}

#[test]
fn every_zone_resource_is_reported_once_per_track_even_when_inactive() {
    let condition = OdinZoneWhen { tension: Some([0.6, 1.0]), ..OdinZoneWhen::default() };
    let spec = OdinTrackSpec {
        zones: vec![bounded(condition), bounded(condition), zone("defaut", vec![])],
    };
    let profile = with(TrackId::Bass, spec);
    let resources = HostResources {
        percussion_banks: vec![profile.instruments.percussion_bank],
        ..HostResources::default()
    };
    assert_eq!(
        profile.missing_resources(&resources),
        [
            MissingResource::OdinPreset { track: TrackId::Bass, preset: "b".into() },
            MissingResource::OdinPreset { track: TrackId::Bass, preset: "defaut".into() },
        ],
    );
}

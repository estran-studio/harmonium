//! Morphing Odin préparé sur le thread principal : valeurs exactes (gains
//! additifs par émotion, bornes du moteur) et charges par piste tirées de la
//! banque de l'hôte. Le callback audio ne fait que copier ces charges.
use std::collections::BTreeMap;

use harmonium_core::{
    odin::{
        ODIN_ZONE_HYSTERESIS, OdinBank, OdinMorphParam, OdinMorphRule, OdinPresetBase,
        OdinTrackSpec, OdinVoiceParams, OdinZone, OdinZoneWhen,
    },
    timeline::TrackId,
};

fn base() -> OdinVoiceParams {
    let mut base = OdinVoiceParams { values: [0.0; OdinMorphParam::COUNT] }.clamp();
    base.values[OdinMorphParam::Fil1Freq.index()] = 1000.0;
    base.values[OdinMorphParam::Fil1Res.index()] = 0.2;
    base.values[OdinMorphParam::Env1Attack.index()] = 0.01;
    base.values[OdinMorphParam::Master.index()] = -7.0;
    base
}

fn rule(param: OdinMorphParam, gains: [f32; 4]) -> OdinMorphRule {
    OdinMorphRule {
        param,
        arousal: gains[0],
        valence: gains[1],
        density: gains[2],
        tension: gains[3],
    }
}

fn bank() -> OdinBank {
    OdinBank::new(vec![
        OdinPresetBase { name: "autre".into(), base: base() },
        OdinPresetBase { name: "nappe".into(), base: base() },
    ])
}

fn specs(preset: &str, morph: Vec<OdinMorphRule>) -> BTreeMap<TrackId, OdinTrackSpec> {
    BTreeMap::from([(
        TrackId::Chords,
        OdinTrackSpec { zones: vec![zone(preset, None, 0.0, morph)] },
    )])
}

fn zone(
    preset: &str,
    when: Option<OdinZoneWhen>,
    level_db: f32,
    morph: Vec<OdinMorphRule>,
) -> OdinZone {
    OdinZone { when, preset: preset.into(), level_db, morph }
}

fn tension_spec() -> OdinTrackSpec {
    OdinTrackSpec {
        zones: vec![
            zone(
                "nappe",
                Some(OdinZoneWhen { tension: Some([0.6, 1.0]), ..OdinZoneWhen::default() }),
                -6.0,
                vec![rule(OdinMorphParam::Fil1Freq, [2000.0, 0.0, 0.0, 0.0])],
            ),
            zone("nappe", None, 0.0, vec![]),
        ],
    }
}

#[test]
fn gains_add_per_emotion_and_the_engine_bounds_apply() {
    let rules = [
        rule(OdinMorphParam::Fil1Freq, [3500.0, 0.0, 0.0, 1500.0]),
        rule(OdinMorphParam::Fil1Res, [0.0, 0.0, 0.0, 3.0]),
        rule(OdinMorphParam::Env1Attack, [0.0, -1.0, 0.0, 0.0]),
    ];
    let out = base().morphed(&rules, [0.5, 0.3, 0.2, 0.4]);
    assert!((out.values[OdinMorphParam::Fil1Freq.index()] - 3350.0).abs() < 1e-2);
    assert_eq!(out.values[OdinMorphParam::Fil1Res.index()], 1.0, "borné à 1");
    assert_eq!(out.values[OdinMorphParam::Env1Attack.index()], 0.001);
    assert_eq!(out.values[OdinMorphParam::Master.index()], -7.0, "socle en dB");
}

#[test]
fn a_preset_without_rules_is_insensitive_to_emotions() {
    assert_eq!(base().morphed(&[], [0.0; 4]), base().morphed(&[], [1.0; 4]));
}

#[test]
fn patches_are_prepared_only_for_tracks_whose_preset_the_host_loaded() {
    let morph = vec![rule(OdinMorphParam::Fil1Freq, [2000.0, 0.0, 0.0, 0.0])];
    let patches = bank().patches(&specs("nappe", morph.clone()), [1.0, 0.0, 0.0, 0.0]);
    for (index, patch) in patches.iter().enumerate() {
        assert_eq!(patch.is_some(), index == usize::from(TrackId::Chords.channel()));
    }
    let patch = patches[usize::from(TrackId::Chords.channel())].unwrap();
    assert_eq!(patch.preset, 1, "rang du preset dans la banque");
    assert_eq!(patch.zone, 0);
    assert_eq!(patch.gain.to_bits(), 1.0_f32.to_bits(), "0 dB garde le gain unitaire exact");
    assert_eq!(patch.params.values[OdinMorphParam::Fil1Freq.index()], 3000.0);

    // Preset absent de la banque : la piste reste SoundFont.
    let missing = bank().patches(&specs("introuvable", morph), [1.0; 4]);
    assert!(missing.iter().all(Option::is_none));
}

#[test]
fn new_emotions_give_new_patches_deterministically() {
    let morph = vec![rule(OdinMorphParam::Fil1Freq, [2000.0, 1000.0, 0.0, 0.0])];
    let specs = specs("nappe", morph);
    let low = bank().patches(&specs, [0.1, 0.1, 0.5, 0.5]);
    assert_eq!(low, bank().patches(&specs, [0.1, 0.1, 0.5, 0.5]));
    assert_ne!(low, bank().patches(&specs, [0.9, 0.1, 0.5, 0.5]));
}

#[test]
fn every_catalogue_parameter_has_a_unique_slot_and_clamps_in_physical_units() {
    let low = OdinVoiceParams { values: [f32::MIN; OdinMorphParam::COUNT] }.clamp();
    let high = OdinVoiceParams { values: [f32::MAX; OdinMorphParam::COUNT] }.clamp();
    for (index, param) in OdinMorphParam::ALL.into_iter().enumerate() {
        assert_eq!(param.index(), index);
        let (min, max) = param.bounds();
        assert_eq!(low.values[index], min, "{}", param.name());
        assert_eq!(high.values[index], max, "{}", param.name());
        let text = format!("param = {:?}", param.name());
        let decoded: OdinMorphRule = toml::from_str(&text).unwrap();
        assert_eq!(decoded.param, param);
        let encoded = toml::to_string(&decoded).unwrap();
        assert!(encoded.contains(&format!("param = {:?}", param.name())));
    }
}

#[test]
fn multiple_rules_accumulate_before_clamping() {
    let rules = [
        rule(OdinMorphParam::Master, [30.0, 0.0, 0.0, 0.0]),
        rule(OdinMorphParam::Master, [-25.0, 0.0, 0.0, 0.0]),
    ];
    let out = base().morphed(&rules, [1.0, 0.0, 0.0, 0.0]);
    assert_eq!(out.values[OdinMorphParam::Master.index()], -2.0);
}

#[test]
fn initial_zone_selection_is_inclusive_and_respects_priority() {
    let mut spec = tension_spec();
    assert_eq!(spec.select_zone([0.5, 0.0, 0.5, 0.6], None), 0);
    assert_eq!(spec.select_zone([0.5, 0.0, 0.5, 1.0], None), 0);
    assert_eq!(spec.select_zone([0.5, 0.0, 0.5, 0.599], None), 1);
    spec.zones.insert(
        0,
        zone(
            "autre",
            Some(OdinZoneWhen {
                valence: Some([-1.0, -0.4]),
                tension: Some([0.6, 1.0]),
                ..OdinZoneWhen::default()
            }),
            0.0,
            vec![],
        ),
    );
    assert_eq!(spec.select_zone([0.5, -1.0, 0.5, 0.6], None), 0);
    assert_eq!(spec.select_zone([0.5, -0.4, 0.5, 1.0], None), 0);
    assert_eq!(spec.select_zone([0.5, 0.0, 0.5, 0.6], None), 1);
    assert_eq!(spec.select_zone([0.5, -1.0, 0.5, 0.5], None), 2);
}

#[test]
fn hysteresis_prevents_oscillation_from_the_default_and_the_bounded_zone() {
    let spec = tension_spec();
    let mut selected = Some(1);
    for tension in [0.59, 0.60, 0.61, 0.62, 0.60] {
        selected = Some(spec.select_zone([0.5, 0.0, 0.5, tension], selected));
        assert_eq!(selected, Some(1), "pas d'entrée avant la marge");
    }
    let entry = 0.6 + ODIN_ZONE_HYSTERESIS;
    selected = Some(spec.select_zone([0.5, 0.0, 0.5, entry], selected));
    assert_eq!(selected, Some(0), "entrée à la borne resserrée inclusive");
    for tension in [1.0, 0.61, 0.60, 0.59, 0.6 - ODIN_ZONE_HYSTERESIS] {
        selected = Some(spec.select_zone([0.5, 0.0, 0.5, tension], selected));
        assert_eq!(selected, Some(0), "maintien dans les bornes élargies");
    }
    assert_eq!(spec.select_zone([0.5, 0.0, 0.5, 0.56], selected), 1);
    assert_eq!(spec.select_zone([0.5, 0.0, 0.5, 1.0], Some(1)), 0);
}

#[test]
fn hysteresis_applies_to_every_axis_without_trapping_the_default() {
    let spec = OdinTrackSpec {
        zones: vec![
            zone(
                "nappe",
                Some(OdinZoneWhen {
                    arousal: Some([0.2, 0.8]),
                    valence: Some([-1.0, -0.4]),
                    density: Some([0.3, 0.7]),
                    tension: Some([0.6, 1.0]),
                }),
                0.0,
                vec![],
            ),
            zone("autre", None, 0.0, vec![]),
        ],
    };
    assert_eq!(spec.select_zone([0.2, -0.4, 0.3, 0.6], None), 0);
    assert_eq!(spec.select_zone([0.2, -0.4, 0.3, 0.6], Some(1)), 1);
    assert_eq!(spec.select_zone([0.5, -1.0, 0.5, 1.0], Some(1)), 0);
    for emotions in [
        [0.84, -0.6, 0.5, 0.8],
        [0.5, -0.36, 0.5, 0.8],
        [0.5, -0.6, 0.74, 0.8],
        [0.5, -0.6, 0.5, 0.56],
    ] {
        assert_eq!(spec.select_zone(emotions, Some(0)), 1);
    }
}

#[test]
fn explicit_zones_keep_their_gain_and_morph_even_with_the_same_preset() {
    let specs = BTreeMap::from([(TrackId::Chords, tension_spec())]);
    let channel = usize::from(TrackId::Chords.channel());
    let emotions = [0.5, 0.0, 0.5, 0.9];
    let initial = bank().patches(&specs, emotions)[channel].unwrap();
    assert_eq!(initial.zone, 0);
    assert!((initial.gain - 10.0_f32.powf(-6.0 / 20.0)).abs() < f32::EPSILON);
    assert_eq!(initial.params.values[OdinMorphParam::Fil1Freq.index()], 2000.0);
    let mut zones = [None; TrackId::ALL.len()];
    zones[channel] = Some(1);
    let held = bank().patches_for_zones(&specs, emotions, zones)[channel].unwrap();
    assert_eq!(held.zone, 1);
    assert_eq!(held.preset, initial.preset);
    assert_eq!(held.gain, 1.0);
    assert_eq!(held.params, base());
    assert_ne!(held, initial);
}

#[test]
fn a_higher_priority_zone_can_interrupt_an_overlapping_active_zone() {
    let spec = OdinTrackSpec {
        zones: vec![
            zone(
                "autre",
                Some(OdinZoneWhen { tension: Some([0.6, 1.0]), ..OdinZoneWhen::default() }),
                0.0,
                vec![],
            ),
            zone(
                "nappe",
                Some(OdinZoneWhen { tension: Some([0.2, 1.0]), ..OdinZoneWhen::default() }),
                0.0,
                vec![],
            ),
            zone("nappe", None, 0.0, vec![]),
        ],
    };
    assert_eq!(spec.select_zone([0.5, 0.0, 0.5, 0.61], Some(1)), 1);
    assert_eq!(spec.select_zone([0.5, 0.0, 0.5, 0.64], Some(1)), 0);
    assert_eq!(spec.select_zone([0.5, 0.0, 0.5, 0.59], Some(0)), 0);
    assert_eq!(spec.select_zone([0.5, 0.0, 0.5, 0.56], Some(0)), 1);
}

#[test]
fn narrow_and_point_zones_remain_reachable_with_history() {
    for bounds in [[0.6, 0.62], [0.61, 0.61], [0.0, 0.0], [1.0, 1.0]] {
        let spec = OdinTrackSpec {
            zones: vec![
                zone(
                    "nappe",
                    Some(OdinZoneWhen { tension: Some(bounds), ..OdinZoneWhen::default() }),
                    0.0,
                    vec![],
                ),
                zone("autre", None, 0.0, vec![]),
            ],
        };
        let midpoint = bounds[0] + (bounds[1] - bounds[0]) * 0.5;
        assert_eq!(spec.select_zone([0.5, 0.0, 0.5, midpoint], Some(1)), 0);
    }
}

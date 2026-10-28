//! Morphing Odin préparé sur le thread principal : valeurs exactes (gains
//! additifs par émotion, bornes du moteur) et charges par piste tirées de la
//! banque de l'hôte. Le callback audio ne fait que copier ces charges.
use std::collections::BTreeMap;

use harmonium_core::{
    odin::{
        OdinBank, OdinMorphParam, OdinMorphRule, OdinPresetBase, OdinTrackSpec, OdinVoiceParams,
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
    BTreeMap::from([(TrackId::Chords, OdinTrackSpec { preset: preset.into(), morph })])
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

//! Morphing Odin préparé sur le thread principal : valeurs exactes (gains
//! additifs par émotion, bornes du moteur) et charges par piste tirées de la
//! banque de l'hôte. Le callback audio ne fait que copier ces charges.
use std::collections::BTreeMap;

use harmonium_core::{
    odin::{
        OdinBank, OdinDiscreteParams, OdinMorphParam, OdinMorphRule, OdinPresetBase, OdinTrackSpec,
        OdinVoiceParams,
    },
    timeline::TrackId,
};

fn base() -> OdinVoiceParams {
    OdinVoiceParams {
        osc_volumes: [1.0, 0.0, 0.0],
        osc_detune: 0.1,
        filter_frequency: 1000.0,
        filter_resonance: 0.2,
        filter_env_amount: 0.0,
        amp_attack: 0.01,
        amp_decay: 0.1,
        amp_sustain: 0.7,
        amp_release: 0.2,
        filter_attack: 0.01,
        filter_decay: 0.1,
        filter_sustain: 0.7,
        filter_release: 0.2,
        master_volume: 0.5,
    }
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
        OdinPresetBase {
            name: "autre".into(),
            base: base(),
            discrete: OdinDiscreteParams::default(),
        },
        OdinPresetBase {
            name: "nappe".into(),
            base: base(),
            discrete: OdinDiscreteParams { octaves: [0, 1, -1], semitones: [0, 7, 0] },
        },
    ])
}

fn specs(preset: &str, morph: Vec<OdinMorphRule>) -> BTreeMap<TrackId, OdinTrackSpec> {
    BTreeMap::from([(TrackId::Chords, OdinTrackSpec { preset: preset.into(), morph })])
}

#[test]
fn gains_add_per_emotion_and_the_engine_bounds_apply() {
    let rules = [
        rule(OdinMorphParam::FilterFrequency, [3500.0, 0.0, 0.0, 1500.0]),
        rule(OdinMorphParam::FilterResonance, [0.0, 0.0, 0.0, 3.0]),
        rule(OdinMorphParam::AmpAttack, [0.0, -1.0, 0.0, 0.0]),
    ];
    let out = base().morphed(&rules, [0.5, 0.3, 0.2, 0.4]);
    assert!((out.filter_frequency - (1000.0 + 1750.0 + 600.0)).abs() < 1e-2);
    assert_eq!(out.filter_resonance, 1.0, "borné à 1");
    assert_eq!(out.amp_attack, 0.001, "borné au minimum du moteur");
    assert_eq!(out.master_volume, base().master_volume, "hors règle : le socle");
}

#[test]
fn a_preset_without_rules_is_insensitive_to_emotions() {
    assert_eq!(base().morphed(&[], [0.0; 4]), base().morphed(&[], [1.0; 4]));
}

#[test]
fn patches_are_prepared_only_for_tracks_whose_preset_the_host_loaded() {
    let morph = vec![rule(OdinMorphParam::FilterFrequency, [2000.0, 0.0, 0.0, 0.0])];
    let patches = bank().patches(&specs("nappe", morph.clone()), [1.0, 0.0, 0.0, 0.0]);
    for (index, patch) in patches.iter().enumerate() {
        assert_eq!(patch.is_some(), index == usize::from(TrackId::Chords.channel()));
    }
    let patch = patches[usize::from(TrackId::Chords.channel())].unwrap();
    assert_eq!(patch.preset, 1, "rang du preset dans la banque");
    assert_eq!(patch.params.filter_frequency, 3000.0);
    assert_eq!(patch.discrete.semitones, [0, 7, 0], "champs discrets repris tels quels");

    // Preset absent de la banque : la piste reste SoundFont.
    let missing = bank().patches(&specs("introuvable", morph), [1.0; 4]);
    assert!(missing.iter().all(Option::is_none));
}

#[test]
fn new_emotions_give_new_patches_deterministically() {
    let morph = vec![rule(OdinMorphParam::FilterFrequency, [2000.0, 1000.0, 0.0, 0.0])];
    let specs = specs("nappe", morph);
    let low = bank().patches(&specs, [0.1, 0.1, 0.5, 0.5]);
    assert_eq!(low, bank().patches(&specs, [0.1, 0.1, 0.5, 0.5]));
    assert_ne!(low, bank().patches(&specs, [0.9, 0.1, 0.5, 0.5]));
}

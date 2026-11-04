//! R3 : le profil par défaut (`StyleProfile::default()`, `TuningParams::default()`) est `reel_quebecois`,
//! et il est **habillé** : sa graine tire tempo, tonalité, instruments et arc (`[session]`, A2). Un test qui
//! suppose un tempo, une tonalité ou des vélocités fixes sur le profil par défaut se trompe : il vide
//! `profile.session` (`Default::default()`) s'il porte sur autre chose que l'habillage, ou il compare à la valeur
//! tirée s'il porte sur une valeur publiée. Ce test garde cette intention explicite.
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams, compose::TuneGenerator, params::MusicalParams,
    report::MeasureSnapshot,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn first_bar(profile: &StyleProfile, seed: u64) -> MeasureSnapshot {
    let params = MusicalParams { bpm: profile.meter.default_bpm, ..Default::default() };
    let mut generator = TuneGenerator::new(params, TuningParams { profile: profile.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    MeasureSnapshot::from_measure(&generator.generate_measure(1, &mut rng))
}

#[test]
fn the_default_profile_is_dressed_by_its_seed() {
    let profile = StyleProfile::default();
    assert_eq!(profile.meta.id, "reel_quebecois", "le profil par défaut est le reel québécois");
    assert!(!profile.session.variants.is_empty(), "le profil par défaut porte une [session]");
    assert_eq!(TuningParams::default().profile, profile);

    let dressings: BTreeSet<_> = (1..=8u64)
        .map(|seed| {
            let bar = first_bar(&profile, seed);
            let position = bar.position.expect("position publiée");
            let programs = bar.instruments.map(|i| i.melody_program);
            (bar.tempo.to_bits(), position.key, format!("{:?}", position.mode), programs)
        })
        .collect();
    assert!(dressings.len() >= 4, "8 graines, {} habillages : {dressings:?}", dressings.len());

    let tempos: BTreeSet<_> =
        (1..=8u64).map(|seed| first_bar(&profile, seed).tempo.to_bits()).collect();
    assert!(tempos.len() >= 2, "le tempo vient de la graine, pas de `meter.default_bpm`");

    // Vidée, la session rend le tempo nominal du profil, quelle que soit la graine.
    let mut plain = profile.clone();
    plain.session = Default::default();
    for seed in 1..=8u64 {
        assert_eq!(first_bar(&plain, seed).tempo, plain.meter.default_bpm, "graine {seed}");
    }
}

//! Q0 : une variante de session ne décale plus les croches d'un profil droit pendant que les
//! doubles croches de sa batterie restent sur la grille. Vérification sur le flux publié.
use harmonium_core::{StyleProfile, TuningParams, compose::TuneGenerator, params::MusicalParams, report::MeasureSnapshot};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[test]
fn rock_et_hiphop_gardent_les_croches_droites_sur_toute_la_session() {
    for id in ["gmd_rock", "gmd_hiphop"] {
        let p = StyleProfile::named(id).unwrap();
        let params = MusicalParams {
            bpm: p.meter.default_bpm,
            time_signature: p.meter.time_signature,
            swing_ratio: p.meter.default_swing_ratio,
            harmony_valence: p.dynamics.neutral_valence,
            intensity: 0.5,
            percussion_density: 1,
            ..Default::default()
        };
        for seed in 1..=16 {
            let mut generator = TuneGenerator::new(params.clone(), TuningParams { profile: p.clone() });
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            for bar in 1..=32 {
                let snapshot = MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng));
                assert_eq!(snapshot.swing_ratio, 0.5, "{id} graine {seed} mesure {bar}");
            }
        }
    }
}

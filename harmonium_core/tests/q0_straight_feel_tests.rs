//! Q0 : une variante de session ne décale plus les croches d'un profil droit pendant que les
//! doubles croches de sa batterie restent sur la grille. Vérification sur le flux publié.
use harmonium_core::{
    StyleProfile, TuningParams, compose::TuneGenerator, params::MusicalParams,
    report::MeasureSnapshot,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Profils `gmd_*` à croches droites : le rapport de swing de chaque mesure reste 0,5 sur toute la session.
const STRAIGHT: [&str; 13] = [
    "gmd_rock",
    "gmd_pop",
    "gmd_hiphop",
    "gmd_soul",
    "gmd_reggae",
    "gmd_funk",
    "gmd_dance_disco",
    "gmd_afrobeat",
    "gmd_neworleans_funk_secondline",
    "gmd_latin_brazilian_bossa",
    "gmd_latin_brazilian_samba",
    "gmd_latin_brazilian_baiao",
    "gmd_afrocuban",
];

#[test]
fn les_profils_gmd_droits_gardent_les_croches_droites_sur_toute_la_session()
-> Result<(), Box<dyn std::error::Error>> {
    for id in STRAIGHT {
        let p = StyleProfile::named(id)?;
        assert_eq!(p.meter.default_swing_ratio, 0.5, "{id}");
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
            let mut generator =
                TuneGenerator::new(params.clone(), TuningParams { profile: p.clone() });
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            for bar in 1..=32 {
                let snapshot =
                    MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng));
                assert_eq!(snapshot.swing_ratio, 0.5, "{id} graine {seed} mesure {bar}");
            }
        }
    }
    Ok(())
}

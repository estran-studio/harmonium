//! Mesure V7b des sets générés : un `(set_index, tune_in_set, tunes_in_set, key, mode)` par air,
//! pour chaque profil distribué, chaque valence et chaque graine. Lu par
//! `specs/012-vie-realisme/measure_sets.py`, qui compare aux tableaux du §3 de l'étude.
//!
//!     cargo run --release -p harmonium_core --example set_metrics -- sortie.json [graines] [airs]
use harmonium_core::{
    StyleProfile, TuningParams, params::MusicalParams, report::MeasureSnapshot, trad::TradGenerator,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Écarts de valence au point neutre du profil : neutre, claire, sombre.
const VALENCES: [Option<f32>; 3] = [None, Some(0.6), Some(-0.6)];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("chemin de sortie requis")?;
    let seeds: u64 = args.next().map_or(Ok(500), |s| s.parse())?;
    let airs: usize = args.next().map_or(Ok(12), |s| s.parse())?;
    let mut rows = Vec::new();
    for profile in StyleProfile::distributed() {
        for valence in VALENCES {
            let valence = valence.unwrap_or(profile.dynamics.neutral_valence);
            for seed in 1..=seeds {
                let mut tuning = TuningParams { profile: profile.clone() };
                // Un tour par air : la logique de set ne dépend pas du nombre de tours.
                tuning.profile.arrangement.tours_per_tune = 1;
                let params = MusicalParams {
                    bpm: profile.meter.default_bpm,
                    harmony_valence: valence,
                    ..Default::default()
                };
                let mut generator = TradGenerator::new(params, tuning);
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                let mut tunes: Vec<serde_json::Value> = Vec::new();
                for bar in 1.. {
                    let measure = generator.generate_measure(bar, &mut rng);
                    let Some(position) = MeasureSnapshot::from_measure(&measure).position else {
                        continue;
                    };
                    if position.tune_index == airs {
                        break;
                    }
                    if position.tune_index == tunes.len() {
                        tunes.push(serde_json::json!([
                            position.set_index,
                            position.tune_in_set,
                            position.tunes_in_set,
                            position.key,
                            position.mode
                        ]));
                    }
                }
                rows.push(serde_json::json!({
                    "style": profile.meta.id, "valence": valence, "seed": seed, "airs": tunes,
                }));
            }
        }
    }
    std::fs::write(path, serde_json::to_vec(&rows)?)?;
    println!("{} sessions mesurées", rows.len());
    Ok(())
}

//! Flux publics pour la mesure V4, sans synthèse ni accès à l'état privé.
use std::{error::Error, io::Write};

use harmonium_core::{
    StyleProfile, TuningParams, params::MusicalParams, report::MeasureSnapshot, timeline::Measure,
    trad::TradGenerator,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
fn stream(profile: StyleProfile, seed: u64, arousal: f32) -> Vec<Measure> {
    let params = MusicalParams {
        bpm: profile.meter.default_bpm,
        arousal,
        rhythm_density: arousal,
        ..Default::default()
    };
    let mut generator = TradGenerator::new(params, TuningParams { profile });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=66).map(|bar| generator.generate_measure(bar, &mut rng)).collect()
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).ok_or("usage: heterophony_streams OUTPUT.jsonl [seeds]")?;
    let seeds = args.get(2).map_or(Ok(20), |s| s.parse::<u64>())?;
    let mut output = std::io::BufWriter::new(std::fs::File::create(path)?);
    for profile in StyleProfile::distributed() {
        for arousal in [0.0, 0.5, 1.0] {
            for seed in 0..seeds {
                let live = stream(profile.clone(), seed, arousal);
                let mut plain = profile.clone();
                plain.heterophony = None;
                plain.ornaments = Default::default();
                let plain = stream(plain, seed, arousal);
                serde_json::to_writer(
                    &mut output,
                    &serde_json::json!({"profile":profile.meta.id,"seed":seed,"arousal":arousal,"enabled":profile.heterophony.is_some(),"live":live.iter().map(MeasureSnapshot::from_measure).collect::<Vec<_>>(),"plain":plain.iter().map(MeasureSnapshot::from_measure).collect::<Vec<_>>()}),
                )?;
                writeln!(output)?;
            }
        }
    }
    Ok(())
}

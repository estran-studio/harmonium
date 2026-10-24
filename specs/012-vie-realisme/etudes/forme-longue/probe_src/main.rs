use harmonium_core::{
    MusicalParams, TuningParams,
    trad::{TradGenerator, pitch_class_name},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::BTreeMap;

fn run(valence: Option<f32>, seeds: u64, tunes: usize) -> Vec<Vec<(u8, String)>> {
    let mut out = Vec::new();
    for seed in 0..seeds {
        let tuning = TuningParams::default();
        let meter = tuning.profile.meter.clone();
        let mut params = MusicalParams::default();
        params.bpm = meter.default_bpm;
        params.time_signature = meter.time_signature;
        if let Some(v) = valence {
            params.harmony_valence = v;
        }
        let cycle = tuning.profile.form.bars_per_tour() * tuning.profile.arrangement.tours_per_tune;
        let mut g = TradGenerator::new(params, tuning);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let total = (cycle + 4) * tunes;
        let mut chain: Vec<(usize, u8, String)> = Vec::new();
        for bar in 1..=total {
            let _ = g.generate_measure(bar, &mut rng);
            let pos = g.position_of(bar);
            if chain.last().map_or(true, |c| c.0 != pos.tune_index) {
                if let Some(t) = g.tune_for_bar(bar) {
                    chain.push((pos.tune_index, t.plan.key, format!("{:?}", t.plan.mode)));
                }
            }
        }
        out.push(chain.into_iter().take(tunes).map(|(_, k, m)| (k, m)).collect());
    }
    out
}

fn main() {
    for (label, v) in [("valence neutre (0.3)", None), ("valence 0.6", Some(0.6)), ("valence -0.6", Some(-0.6))] {
        let chains = run(v, 400, 5);
        let mut trans: BTreeMap<String, usize> = BTreeMap::new();
        let mut keys: BTreeMap<String, usize> = BTreeMap::new();
        let mut n = 0usize;
        let mut same = 0usize;
        for c in &chains {
            for (k, m) in c {
                *keys.entry(format!("{} {}", pitch_class_name(*k), m)).or_default() += 1;
            }
            for w in c.windows(2) {
                n += 1;
                if w[0] == w[1] {
                    same += 1;
                }
                *trans
                    .entry(format!(
                        "{} {} -> {} {}",
                        pitch_class_name(w[0].0),
                        w[0].1,
                        pitch_class_name(w[1].0),
                        w[1].1
                    ))
                    .or_default() += 1;
            }
        }
        println!("== {label} : {} chaînes, {n} transitions, identiques {:.1} %", chains.len(), 100.0 * same as f64 / n as f64);
        println!("clés : {keys:?}");
        let mut t: Vec<_> = trans.into_iter().collect();
        t.sort_by(|a, b| b.1.cmp(&a.1));
        for (k, c) in t.iter().take(14) {
            println!("  {k} : {c}");
        }
    }
}

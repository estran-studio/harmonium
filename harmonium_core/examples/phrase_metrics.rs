use harmonium_core::{
    StyleProfile, TuningParams,
    params::MusicalParams,
    report::MeasureSnapshot,
    trad::{ImposedRole, ReelArrangement, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rows = Vec::new();
    for distributed in StyleProfile::distributed() {
        for variant in ["before_p3t1", "before_p3t3", "after"] {
            let mut p = distributed.clone();
            p.arrangement = ReelArrangement::from_start(2, Default::default());
            for part in &mut p.form.parts {
                if variant != "after" {
                    part.rhythm.clear();
                }
                if variant == "before_p3t1" {
                    part.second_ending = None;
                    part.roles = [
                        ImposedRole::Open,
                        ImposedRole::Free,
                        ImposedRole::Free,
                        ImposedRole::CadHalf,
                        ImposedRole::CopyOf(1),
                        ImposedRole::CopyOf(2),
                        ImposedRole::Free,
                        ImposedRole::CadFull,
                    ]
                    .map(Into::into)
                    .to_vec();
                }
            }
            for seed in 1..=200 {
                let mut g = TradGenerator::new(
                    MusicalParams { bpm: p.meter.default_bpm, ..Default::default() },
                    TuningParams { profile: p.clone() },
                );
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                let snapshots: Vec<_> = (1..=32)
                    .map(|b| MeasureSnapshot::from_measure(&g.generate_measure(b, &mut rng)))
                    .collect();
                let bars: Vec<_> = snapshots
                    .iter()
                    .filter(|s| s.position.is_some_and(|pos| pos.pass == 1))
                    .map(|s| {
                        let notes: Vec<_> = s
                            .notes
                            .iter()
                            .filter(|n| n.track == 0)
                            .map(|n| (n.pitch, n.start_step / p.meter.steps_per_unit))
                            .collect();
                        notes
                    })
                    .collect();
                rows.push(serde_json::json!({"style": p.meta.id, "variant": variant, "seed": seed, "units": p.melody.cell_units,
                    "meter": format!("{}/{}", p.meter.time_signature.numerator, p.meter.time_signature.denominator),
                    "key": snapshots[0].position.map(|pos| pos.key), "mode": snapshots[0].position.map(|pos| pos.mode), "bars": bars}));
            }
        }
    }
    let path = std::env::args().nth(1).ok_or("output path required")?;
    std::fs::write(path, serde_json::to_vec(&rows)?)?;
    println!("{} airs mesurés", rows.len());
    Ok(())
}

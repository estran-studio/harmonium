//! Mesures des sets complets, avant et après dynamique, depuis le générateur réel.
//! cargo run --release -p harmonium_core --example set_metrics -- sortie.json [graines] [airs]
use harmonium_core::{
    StyleProfile, TuningParams,
    params::MusicalParams,
    timeline::TrackId,
    trad::{ActiveTune, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

const VALENCES: [Option<f32>; 3] = [None, Some(0.6), Some(-0.6)];

#[derive(Serialize)]
struct TuneMetrics {
    first_bar: usize,
    last_bar: usize,
    body_bars: usize,
    intro_tracks: Vec<u8>,
    tempo_min: f32,
    tempo_max: f32,
    last_mode: harmonium_core::trad::Mode,
    tail_is_outro: bool,
    /// Par piste : nombre de notes et somme des vélocités, corps seulement.
    energy: [[u64; 2]; TrackId::ALL.len()],
}

fn material_hash(tune: &ActiveTune) -> String {
    // Empreinte déterministe du matériel écrit, pas de l'interprétation ni de sa graine.
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for bar in 0..tune.plan.total_bars() {
        hash = (hash ^ bar as u64).wrapping_mul(0x0000_0100_0000_01b3);
        for note in tune.fiddle.notes_for_bar(bar) {
            for value in [u64::from(note.pitch), note.start_step as u64, note.duration_steps as u64]
            {
                hash = (hash ^ value).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
    }
    format!("{hash:016x}")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("chemin de sortie requis")?;
    let seeds: u64 = args.next().map_or(Ok(500), |s| s.parse())?;
    let airs: usize = args.next().map_or(Ok(16), |s| s.parse())?;
    let mut rows = Vec::new();
    for profile in StyleProfile::distributed() {
        for valence in VALENCES {
            let valence = valence.unwrap_or(profile.dynamics.neutral_valence);
            for seed in 1..=seeds {
                let mut tuning = TuningParams { profile: profile.clone() };
                tuning.profile.arrangement.tours_per_tune = 1;
                let params = MusicalParams {
                    bpm: profile.meter.default_bpm,
                    harmony_valence: valence,
                    intensity: 0.5,
                    ..Default::default()
                };
                let mut generator = TradGenerator::new(params, tuning);
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                let mut tunes: Vec<serde_json::Value> = Vec::new();
                let mut materials = Vec::new();
                let mut details: Vec<TuneMetrics> = Vec::new();
                for bar in 1.. {
                    let measure = generator.generate_measure(bar, &mut rng);
                    let Some(position) = measure.position else { continue };
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
                        if let Some(tune) = generator.tune_for_bar(bar) {
                            materials.push(material_hash(tune));
                        }
                        details.push(TuneMetrics {
                            first_bar: bar,
                            last_bar: bar,
                            body_bars: 0,
                            intro_tracks: Vec::new(),
                            tempo_min: measure.tempo,
                            tempo_max: measure.tempo,
                            last_mode: position.mode,
                            tail_is_outro: false,
                            energy: [[0; 2]; TrackId::ALL.len()],
                        });
                    }
                    let Some(detail) = details.last_mut() else { continue };
                    detail.last_bar = bar;
                    detail.tempo_min = detail.tempo_min.min(measure.tempo);
                    detail.tempo_max = detail.tempo_max.max(measure.tempo);
                    detail.last_mode = position.mode;
                    if position.is_intro() {
                        let mask = TrackId::ALL.iter().fold(0u8, |mask, &track| {
                            if measure.notes_for_track(track).is_empty() {
                                mask
                            } else {
                                mask | (1 << track.channel())
                            }
                        });
                        detail.intro_tracks.push(mask);
                    } else {
                        detail.body_bars += 1;
                        for track in TrackId::ALL {
                            for note in measure.notes_for_track(track) {
                                let values = &mut detail.energy[usize::from(track.channel())];
                                values[0] += 1;
                                values[1] += u64::from(note.velocity);
                            }
                        }
                        let melody = measure.notes_for_track(TrackId::Melody);
                        detail.tail_is_outro = melody.len() == 1
                            && melody[0].pitch % 12 == position.key
                            && melody[0].start_step == 0;
                    }
                }
                rows.push(serde_json::json!({
                    "style": profile.meta.id, "valence": valence, "seed": seed,
                    "airs": tunes, "materials": materials, "details": details,
                }));
            }
        }
    }
    std::fs::write(path, serde_json::to_vec(&rows)?)?;
    println!("{} sessions mesurées", rows.len());
    Ok(())
}

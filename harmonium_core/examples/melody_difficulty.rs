//! Mesure de la difficulté de la mélodie : 3 graines × 64 mesures, quatre niveaux, profils
//! distribués qui ont une mélodie. Les niveaux reprennent les leviers de `harmonium_training`
//! (tessiture, saut en degrés, poids des rôles, ornements) ; `--filter` ajoute le filtre
//! `[melody.difficulty]` de référence de chaque niveau.
//!
//! `cargo run -p harmonium_core --example melody_difficulty --release [-- --filter]`
use harmonium_core::{
    StyleProfile, TuningParams,
    compose::{MelodyDifficulty, TuneGenerator},
    params::MusicalParams,
    report::MeasureSnapshot,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const SEEDS: [u64; 3] = [42, 137, 2026];
fn bars() -> usize {
    std::env::var("BARS").ok().and_then(|v| v.parse().ok()).unwrap_or(64)
}
const KEYBOARD: (u8, u8) = (48, 96);

/// Leviers d'application de `harmonium_training` (`apply_level`), niveau 0-based.
fn apply_training_levels(p: &mut StyleProfile, level: usize) {
    let width = [12, 15, 19, p.melody.params.tessitura_max - p.melody.params.tessitura_min][level]
        .min(KEYBOARD.1 - KEYBOARD.0);
    let low = KEYBOARD.0 + (KEYBOARD.1 - KEYBOARD.0 - width) / 2;
    let params = &mut p.melody.params;
    params.tessitura_min = low;
    params.tessitura_max = low + width;
    params.melody_center_midi = low + width / 2;
    params.tonic_midi_min = low;
    params.tonic_midi_max = low + width;
    if level < 3 {
        p.form.max_leap_degrees = [3, 4, 5][level];
    }
    let factors = [[0.0, 0.0, 0.0], [0.5, 0.5, 0.0], [1.0, 1.0, 0.5], [1.0, 1.0, 1.0]][level];
    let original = [params.weight_role_run, params.weight_role_arp, params.weight_role_turn];
    let mut weights =
        std::array::from_fn::<_, 3, _>(|n| (original[n] as f32 * factors[n]).round() as u32);
    if weights.iter().all(|w| *w == 0)
        && let Some(n) = original.iter().position(|w| *w > 0)
    {
        weights[n] = 1;
    }
    [params.weight_role_run, params.weight_role_arp, params.weight_role_turn] = weights;
    if level < 3 {
        simplify_rhythms(p, level);
    }
    p.ornaments.rate_scale *= [0.0, 0.0, 0.5, 1.0][level];
}

/// Plus petite durée (en unités) d'une note des cellules de la banque : la durée « native » du
/// profil, au-dessous de laquelle le filtre ne peut rien simplifier.
fn native_shortest(p: &StyleProfile) -> usize {
    use harmonium_core::compose::cells::CellNote;
    p.melody
        .cells
        .cells
        .iter()
        .flat_map(|cell| {
            let attacks: Vec<usize> = cell
                .notes
                .iter()
                .enumerate()
                .filter(|(_, n)| matches!(n, CellNote::Degree(_)))
                .map(|(i, _)| i)
                .collect();
            (0..attacks.len())
                .map(|k| attacks.get(k + 1).copied().unwrap_or(cell.notes.len()) - attacks[k])
                .collect::<Vec<_>>()
        })
        .min()
        .unwrap_or(1)
}

/// Filtre de référence d'un niveau (0-based) : moitié, trois quarts puis toute la mesure en
/// attaques ; saut d'une tierce, d'une quarte puis d'une quinte ; ambitus d'une quinte, d'une
/// sixte puis d'une octave ; plus petite durée double, une fois et demie puis égale à la durée
/// native du profil (au moins deux unités au niveau 1) ; ni ornement ni ajustement aux accords
/// aux niveaux 1 à 3. Le niveau 4 garde le profil.
fn reference_difficulty(level: usize, p: &StyleProfile) -> Option<MelodyDifficulty> {
    let units = p.melody.cell_units;
    let native = native_shortest(p);
    (level < 3).then(|| MelodyDifficulty {
        max_leap_semitones: Some([4, 5, 7][level]),
        max_attacks_per_bar: Some([units.div_ceil(2), (units * 3).div_ceil(4), units][level]),
        min_note_units: Some(match level {
            0 => (2 * native).max(2),
            1 => (3 * native).div_ceil(2),
            _ => native,
        }),
        max_cell_ambitus_semitones: Some([7, 9, 12][level]),
        out_of_scale: false,
        ornaments: false,
        chord_fit: false,
    })
}

fn simplify_rhythms(p: &mut StyleProfile, level: usize) {
    use harmonium_core::compose::cells::CellNote;
    let count: usize = p
        .melody
        .cells
        .cells
        .iter()
        .map(|cell| cell.notes.iter().filter(|n| matches!(n, CellNote::Degree(_))).count())
        .sum();
    let average = count as f32 / p.melody.cells.cells.len().max(1) as f32;
    let target = (average * [0.3, 0.5, 0.7][level]).floor().max(1.0) as usize;
    for part in &mut p.form.parts {
        for rhythm in &mut part.rhythm {
            for pattern in &mut rhythm.patterns {
                while pattern.durations.len() > target {
                    let index = pattern
                        .durations
                        .windows(2)
                        .enumerate()
                        .min_by_key(|(_, pair)| pair[0] + pair[1])
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    let merged = pattern.durations.remove(index + 1);
                    pattern.durations[index] += merged;
                }
            }
        }
    }
}

#[derive(Default, Clone, Copy)]
struct Metrics {
    ambitus: f64,
    leap: f64,
    density: f64,
    shortest: f64,
    worst_leap: u8,
    worst_attacks: usize,
    worst_shortest: usize,
}

fn measure(p: &StyleProfile, seed: u64) -> Metrics {
    let mut generator = TuneGenerator::new(
        MusicalParams { bpm: p.meter.default_bpm, ..Default::default() },
        TuningParams { profile: p.clone() },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let snapshots: Vec<_> = (1..=bars())
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect();
    let unit = p.meter.steps_per_unit;
    let mut pitches = Vec::new();
    let mut shortest = usize::MAX;
    let mut worst_attacks = 0;
    for snapshot in &snapshots {
        let notes: Vec<_> = snapshot.notes.iter().filter(|n| n.track == 0).collect();
        worst_attacks = worst_attacks.max(notes.len());
        for note in notes {
            pitches.push(note.pitch);
            shortest = shortest.min(note.duration_steps);
        }
    }
    let ambitus = pitches.iter().max().zip(pitches.iter().min()).map_or(0, |(a, b)| a - b);
    let leap = pitches.windows(2).map(|w| w[0].abs_diff(w[1])).max().unwrap_or(0);
    Metrics {
        ambitus: f64::from(ambitus),
        leap: f64::from(leap),
        density: pitches.len() as f64 / bars() as f64,
        shortest: if shortest == usize::MAX { 0.0 } else { (shortest / unit.max(1)) as f64 },
        worst_leap: leap,
        worst_attacks,
        worst_shortest: if shortest == usize::MAX { 0 } else { shortest / unit.max(1) },
    }
}

/// `--explain <profil> <niveau>` : où le flux dépasse les bornes du niveau, par graine.
fn explain(id: &str, level: usize) {
    let Some(distributed) = StyleProfile::distributed().iter().find(|p| p.meta.id == id) else {
        return;
    };
    let mut p = distributed.clone();
    apply_training_levels(&mut p, level);
    p.melody.difficulty = reference_difficulty(level, &p);
    let Some(limits) = p.melody.difficulty.clone() else { return };
    for seed in SEEDS {
        let mut generator = TuneGenerator::new(
            MusicalParams { bpm: p.meter.default_bpm, ..Default::default() },
            TuningParams { profile: p.clone() },
        );
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut last: Option<(usize, u8)> = None;
        for bar in 1..=bars() {
            let snapshot =
                MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng));
            let position = snapshot.position;
            let where_ = position.map_or(String::new(), |pos| {
                format!(
                    "{:?} air {} tour {} part {} passe {} mesure {}",
                    pos.section, pos.tune_index, pos.tour, pos.part.0, pos.pass, pos.bar_in_part
                )
            });
            let notes: Vec<_> = snapshot.notes.iter().filter(|n| n.track == 0).collect();
            let attacks = notes.len();
            for (i, note) in notes.iter().enumerate() {
                if let (Some(max), Some((previous_bar, previous))) =
                    (limits.max_leap_semitones, last)
                    && note.pitch.abs_diff(previous) > max
                {
                    println!(
                        "graine {seed} mesure {bar} ({where_}) note {i}/{attacks} : saut {} > {max} (note précédente mesure {previous_bar})",
                        note.pitch.abs_diff(previous)
                    );
                }
                if let Some(min) = limits.min_note_units
                    && note.duration_steps < min * p.meter.steps_per_unit
                {
                    println!(
                        "graine {seed} mesure {bar} ({where_}) note {i}/{attacks} : durée {} pas < {min} unités",
                        note.duration_steps
                    );
                }
                last = Some((bar, note.pitch));
            }
            if let Some(max) = limits.max_attacks_per_bar
                && attacks > max
            {
                println!("graine {seed} mesure {bar} ({where_}) : {attacks} attaques > {max}");
            }
        }
    }
}

/// `--show <profil> <niveau> <graine>` : les 16 premières mesures publiées (hauteur:unités).
fn show(id: &str, level: usize, seed: u64, filtered: bool) {
    let Some(distributed) = StyleProfile::distributed().iter().find(|p| p.meta.id == id) else {
        return;
    };
    let mut p = distributed.clone();
    apply_training_levels(&mut p, level);
    if filtered {
        p.melody.difficulty = reference_difficulty(level, &p);
    }
    let mut generator = TuneGenerator::new(
        MusicalParams { bpm: p.meter.default_bpm, ..Default::default() },
        TuningParams { profile: p.clone() },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    for bar in 1..=16 {
        let snapshot = MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng));
        let notes: Vec<String> = snapshot
            .notes
            .iter()
            .filter(|n| n.track == 0)
            .map(|n| format!("{}:{}", n.pitch, n.duration_steps / p.meter.steps_per_unit.max(1)))
            .collect();
        println!("{bar:>2} {}", notes.join(" "));
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if let Some(at) = arguments.iter().position(|a| a == "--show") {
        let filtered = arguments.iter().any(|a| a == "--filter");
        show(
            &arguments[at + 1],
            arguments[at + 2].parse::<usize>().unwrap_or(1) - 1,
            arguments[at + 3].parse().unwrap_or(42),
            filtered,
        );
        return;
    }
    if let Some(at) = arguments.iter().position(|a| a == "--explain") {
        explain(&arguments[at + 1], arguments[at + 2].parse::<usize>().unwrap_or(1) - 1);
        return;
    }
    let filtered = std::env::args().any(|a| a == "--filter");
    println!(
        "| Profil | Niv. | Ambitus | Plus grand saut (moy. / pire) | Attaques/mesure (moy. / pire mesure) | Plus petite durée (unités, pire) |"
    );
    println!("|---|---|---|---|---|---|");
    for distributed in StyleProfile::distributed() {
        if !distributed.arrangement.has_track(harmonium_core::timeline::TrackId::Melody) {
            continue;
        }
        for level in 0..4 {
            let mut p = distributed.clone();
            apply_training_levels(&mut p, level);
            if filtered {
                p.melody.difficulty = reference_difficulty(level, &p);
            }
            if p.validate().is_err() {
                println!("| {} | {} | invalide |||||", p.meta.id, level + 1);
                continue;
            }
            let runs: Vec<_> = SEEDS.iter().map(|&s| measure(&p, s)).collect();
            let mean = |f: fn(&Metrics) -> f64| runs.iter().map(f).sum::<f64>() / runs.len() as f64;
            println!(
                "| {} | {} | {:.1} | {:.1} / {} | {:.2} / {} | {:.1} / {} |",
                p.meta.id,
                level + 1,
                mean(|m| m.ambitus),
                mean(|m| m.leap),
                runs.iter().map(|m| m.worst_leap).max().unwrap_or(0),
                mean(|m| m.density),
                runs.iter().map(|m| m.worst_attacks).max().unwrap_or(0),
                mean(|m| m.shortest),
                runs.iter().map(|m| m.worst_shortest).min().unwrap_or(0),
            );
        }
    }
}

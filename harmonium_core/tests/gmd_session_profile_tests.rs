//! R2-styles : l'habillage par graine des neuf profils `gmd_*` (rock, pop, hip-hop, soul, reggae, disco, afrobeat, funk,
//! New Orleans), observé dans le flux de `MeasureSnapshot` (principe VI) : « New seed » doit changer le tempo, la tonalité,
//! les instruments, le kit et la famille de voicings, dans les bornes du style, et l'hôte garde la main.
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams, compose::TuneGenerator, params::MusicalParams,
    report::MeasureSnapshot, style_profile::FixedTonality,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const IDS: [&str; 9] = [
    "gmd_rock",
    "gmd_pop",
    "gmd_hiphop",
    "gmd_soul",
    "gmd_reggae",
    "gmd_dance_disco",
    "gmd_afrobeat",
    "gmd_funk",
    "gmd_neworleans_funk_secondline",
];
/// Rapport de swing de la source : droit seulement (Q0, phase 020, a retiré le swing léger du GMD).
const SWINGS: [f32; 1] = [0.5];

fn flow(p: &StyleProfile, seed: u64, bars: usize, bpm: Option<f32>) -> Vec<MeasureSnapshot> {
    let params = MusicalParams {
        bpm: bpm.unwrap_or(p.meter.default_bpm),
        bpm_explicit: bpm.is_some(),
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        intensity: 0.5,
        percussion_density: 1,
        ..Default::default()
    };
    let mut g = TuneGenerator::new(params, TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Dressing {
    tempo_cluster: usize,
    key: u8,
    mode: String,
    chords: Option<u8>,
    bass: Option<u8>,
    kit: Option<u8>,
    voicing: String,
    swing_permille: i32,
}

fn voicing_of(s: &MeasureSnapshot) -> String {
    s.session.iter().find(|v| v.group == "voicing").map(|v| v.name.clone()).unwrap_or_default()
}

/// Habillage d'une graine, lu dans sa première mesure ; les tempos se rangent ensuite en grappes de 16 BPM (± 8).
fn dressings(p: &StyleProfile) -> TestResult<Vec<(f32, Dressing)>> {
    let mut rows: Vec<(f32, Dressing)> = (1..=16u64)
        .map(|seed| -> TestResult<(f32, Dressing)> {
            let first = flow(p, seed, 1, None).remove(0);
            let position = first.position.ok_or("position publiée")?;
            let instruments = first.instruments.ok_or("programmes publiés sous une session")?;
            Ok((
                first.tempo,
                Dressing {
                    tempo_cluster: 0,
                    key: position.key,
                    mode: format!("{:?}", position.mode),
                    chords: instruments.chords_program,
                    bass: instruments.bass_program,
                    kit: Some(instruments.percussion_program),
                    voicing: voicing_of(&first),
                    swing_permille: (first.swing_ratio * 1000.0).round() as i32,
                },
            ))
        })
        .collect::<TestResult<_>>()?;
    let mut order: Vec<f32> = rows.iter().map(|(tempo, _)| *tempo).collect();
    order.sort_by(f32::total_cmp);
    let mut starts: Vec<f32> = Vec::new();
    for tempo in order {
        if starts.last().is_none_or(|start| tempo - start > 16.0) {
            starts.push(tempo);
        }
    }
    for (tempo, d) in &mut rows {
        d.tempo_cluster = starts.iter().rposition(|start| *tempo >= *start).unwrap_or(0);
    }
    Ok(rows)
}

#[test]
fn seize_graines_donnent_au_moins_huit_habillages_dans_les_bornes_du_style() -> TestResult {
    for id in IDS {
        let p = StyleProfile::named(id)?;
        let rows = dressings(&p)?;
        let distinct: BTreeSet<_> = rows.iter().map(|(_, d)| d.clone()).collect();
        let clusters: BTreeSet<_> = rows.iter().map(|(_, d)| d.tempo_cluster).collect();
        let keys: BTreeSet<_> = rows.iter().map(|(_, d)| (d.key, d.mode.clone())).collect();
        let chords: BTreeSet<_> = rows.iter().map(|(_, d)| d.chords).collect();
        let bass: BTreeSet<_> = rows.iter().map(|(_, d)| d.bass).collect();
        let kits: BTreeSet<_> = rows.iter().map(|(_, d)| d.kit).collect();
        let voicings: BTreeSet<_> = rows.iter().map(|(_, d)| d.voicing.clone()).collect();
        assert!(
            distinct.len() >= 8,
            "{id} : {} habillages distincts sur 16 graines",
            distinct.len()
        );
        assert!(clusters.len() >= 2, "{id} : tempos à ±8 BPM {rows:?}");
        assert!(keys.len() >= 3, "{id} : tonalités {keys:?}");
        assert!(
            chords.len() >= 3 && bass.len() >= 2 && kits.len() >= 3,
            "{id} : {chords:?} {bass:?} {kits:?}"
        );
        assert!(voicings.len() >= 2, "{id} : familles de voicings {voicings:?}");
        for (tempo, d) in &rows {
            assert!(
                (p.meter.min_bpm..=p.meter.max_bpm).contains(tempo),
                "{id} : {tempo} BPM hors de la plage de la source {}..{}",
                p.meter.min_bpm,
                p.meter.max_bpm
            );
            assert!(
                SWINGS
                    .iter()
                    .any(|s| (f32::from(d.swing_permille as i16) / 1000.0 - s).abs() < 2e-3),
                "{id} : swing {} hors de la source",
                d.swing_permille
            );
        }
    }
    Ok(())
}

#[test]
fn la_meme_graine_rejoue_le_meme_habillage_et_les_memes_notes() -> TestResult {
    for id in IDS {
        let p = StyleProfile::named(id)?;
        for seed in [3u64, 11] {
            let a = serde_json::to_value(flow(&p, seed, 40, None))?;
            let b = serde_json::to_value(flow(&p, seed, 40, None))?;
            assert_eq!(a, b, "{id} graine {seed}");
        }
    }
    Ok(())
}

#[test]
fn l_hote_garde_la_tonalite_et_le_tempo_et_l_habillage_reste_visible() -> TestResult {
    for id in IDS {
        let mut p = StyleProfile::named(id)?;
        let mode = p.melody.key_modes[0].mode;
        p.fixed_tonality = Some(FixedTonality { key: 5, mode });
        let mut kits = BTreeSet::new();
        for seed in 1..=16u64 {
            let snaps = flow(&p, seed, 8, Some(100.0));
            let first = &snaps[0];
            assert_eq!(
                first.position.map(|p| p.key),
                Some(5),
                "{id} graine {seed} : tonalité de l'hôte"
            );
            assert!(snaps.iter().all(|s| s.tempo == 100.0), "{id} graine {seed} : tempo de l'hôte");
            assert!(!first.session.is_empty(), "{id} : variante de session publiée");
            kits.insert(first.instruments.map(|i| i.percussion_program));
        }
        assert!(kits.len() >= 2, "{id} : le kit reste tiré par la graine sous la grille de l'hôte");
    }
    Ok(())
}

#[test]
fn les_notes_changent_avec_la_graine_a_grille_et_tonalite_egales() -> TestResult {
    // Sous la tonalité et le tempo de l'hôte, deux graines jouent des notes différentes (comping, basse, batterie).
    for id in IDS {
        let mut p = StyleProfile::named(id)?;
        let mode = p.melody.key_modes[0].mode;
        p.fixed_tonality = Some(FixedTonality { key: 0, mode });
        let runs = (1..=8u64)
            .map(|seed| {
                let notes: Vec<_> =
                    flow(&p, seed, 32, Some(100.0)).iter().map(|s| s.notes.clone()).collect();
                serde_json::to_string(&notes)
            })
            .collect::<Result<BTreeSet<String>, _>>()?;
        assert!(runs.len() >= 7, "{id} : {} suites de notes distinctes sur 8 graines", runs.len());
    }
    Ok(())
}

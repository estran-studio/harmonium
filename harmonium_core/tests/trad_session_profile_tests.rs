//! R2-trad : l'habillage par graine des dix profils trad distribués (reels, gigues, hornpipe, marche, valses),
//! observé dans le flux de `MeasureSnapshot` (principe VI) : « New seed » doit changer le tempo, la tonalité,
//! l'instrument de mélodie, le comping, la basse et le kit, dans les bornes de la tradition ; les airs ne changent pas
//! de nature ; l'hôte garde la main ; l'arc nommé se joue sur les tours d'un air.
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams, compose::TuneGenerator, params::MusicalParams,
    report::MeasureSnapshot, style_profile::FixedTonality,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const IDS: [&str; 10] = [
    "reel_quebecois",
    "reel_acadien",
    "reel_britannique_coloniale",
    "gigue_quebecoise",
    "gigue_acadienne",
    "gigue_britannique_coloniale",
    "hornpipe_britannique_coloniale",
    "marche_britannique_coloniale",
    "valse_quebecoise",
    "valse_britannique_coloniale",
];
const MELODY: u8 = 0;

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

fn group_of(s: &MeasureSnapshot, group: &str) -> String {
    s.session.iter().find(|v| v.group == group).map(|v| v.name.clone()).unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Dressing {
    tempo_cluster: usize,
    key: u8,
    mode: String,
    melody: u8,
    chords: Option<u8>,
    bass: Option<u8>,
    kit: u8,
    voicing: String,
    swing_permille: i32,
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
                    melody: instruments.melody_program,
                    chords: instruments.chords_program,
                    bass: instruments.bass_program,
                    kit: instruments.percussion_program,
                    voicing: group_of(&first, "voicing"),
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
fn seize_graines_donnent_au_moins_huit_habillages_dans_les_bornes_de_la_tradition() -> TestResult {
    for id in IDS {
        let p = StyleProfile::named(id)?;
        let rows = dressings(&p)?;
        let distinct: BTreeSet<_> = rows.iter().map(|(_, d)| d.clone()).collect();
        let clusters: BTreeSet<_> = rows.iter().map(|(_, d)| d.tempo_cluster).collect();
        let keys: BTreeSet<_> = rows.iter().map(|(_, d)| (d.key, d.mode.clone())).collect();
        let melody: BTreeSet<_> = rows.iter().map(|(_, d)| d.melody).collect();
        let chords: BTreeSet<_> = rows.iter().map(|(_, d)| d.chords).collect();
        let bass: BTreeSet<_> = rows.iter().map(|(_, d)| d.bass).collect();
        let kits: BTreeSet<_> = rows.iter().map(|(_, d)| d.kit).collect();
        assert!(
            distinct.len() >= 8,
            "{id} : {} habillages distincts sur 16 graines",
            distinct.len()
        );
        // La valse québécoise ne couvre que 10 BPM : une seule grappe à ± 8 BPM près.
        let narrow = p.meter.max_bpm - p.meter.min_bpm <= 16.0;
        assert!(narrow || clusters.len() >= 2, "{id} : tempos à ±8 BPM {rows:?}");
        assert!(keys.len() >= 3, "{id} : tonalités {keys:?}");
        assert!(melody.len() >= 2, "{id} : instruments de mélodie {melody:?}");
        if id == "marche_britannique_coloniale" {
            assert!(
                chords == BTreeSet::from([None]) && bass == BTreeSet::from([None]),
                "{id} : pas d'accompagnement"
            );
            assert!(kits.len() >= 2, "{id} : kits {kits:?}");
        } else {
            assert!(chords.len() >= 3 && bass.len() >= 2, "{id} : {chords:?} {bass:?}");
            let voicings: BTreeSet<_> = rows.iter().map(|(_, d)| d.voicing.clone()).collect();
            assert!(voicings.len() >= 2, "{id} : familles de voicings {voicings:?}");
        }
        for (tempo, _) in &rows {
            assert!(
                (p.meter.min_bpm..=p.meter.max_bpm).contains(tempo),
                "{id} : {tempo} BPM hors de la plage de la tradition {}..{}",
                p.meter.min_bpm,
                p.meter.max_bpm
            );
        }
    }
    Ok(())
}

#[test]
fn les_instruments_restent_dans_la_tradition() -> TestResult {
    // Mélodie : violon (40), fiddle (110), accordéon (21), harmonica (22), flûte (73) ; marche : fifre et cuivres.
    const MELODIES: [u8; 9] = [40, 110, 21, 22, 73, 72, 71, 56, 40];
    for id in IDS {
        let p = StyleProfile::named(id)?;
        for (_, d) in dressings(&p)? {
            assert!(MELODIES.contains(&d.melody), "{id} : mélodie {}", d.melody);
            if let Some(chords) = d.chords {
                assert!([0, 3, 24, 25].contains(&chords), "{id} : comping {chords}");
            }
            if let Some(bass) = d.bass {
                assert!([0, 32, 33].contains(&bass), "{id} : basse {bass}");
            }
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
        let mut melodies = BTreeSet::new();
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
            melodies.insert(first.instruments.map(|i| i.melody_program));
        }
        assert!(
            melodies.len() >= 2,
            "{id} : l'instrument reste tiré par la graine sous la grille de l'hôte"
        );
    }
    Ok(())
}

#[test]
fn l_arc_nomme_monte_sur_les_tours_d_un_air() -> TestResult {
    // Sous l'arc « montee », la vélocité moyenne de la mélodie croît du premier au dernier tour du premier air.
    for id in IDS {
        let p = StyleProfile::named(id)?;
        let bars = p.form.bars_per_tour() * 3;
        let seed = (1..=64u64)
            .find(|seed| {
                let first = flow(&p, *seed, 1, None).remove(0);
                group_of(&first, "arc") == "montee"
            })
            .ok_or("une graine à l'arc `montee` parmi 64")?;
        let snaps = flow(&p, seed, bars, None);
        let mean = |tour: usize| -> f32 {
            let v: Vec<f32> = snaps
                .iter()
                .filter(|s| s.position.is_some_and(|p| p.tour == tour))
                .flat_map(|s| {
                    s.notes.iter().filter(|n| n.track == MELODY).map(|n| f32::from(n.velocity))
                })
                .collect();
            v.iter().sum::<f32>() / v.len().max(1) as f32
        };
        let (first, last) = (mean(1), mean(3));
        assert!(first > 0.0 && last > first + 3.0, "{id} : vélocité tour 1 {first}, tour 3 {last}");
    }
    Ok(())
}

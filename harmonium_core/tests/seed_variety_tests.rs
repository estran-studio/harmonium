//! Variété et identité des arrangements observées dans le contrat de snapshots.
use std::collections::BTreeMap;

use harmonium_core::{
    MeasureSnapshot, StyleProfile, TuningParams, params::MusicalParams, trad::TradGenerator,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const ISAAC: [&str; 5] =
    ["isaac_basement", "isaac_cellar", "isaac_burning", "isaac_burning_psyche", "isaac_downpour"];
type Grid = Vec<(u8, u8, usize, usize)>;

fn params(profile: &StyleProfile, density: u8) -> MusicalParams {
    MusicalParams {
        bpm: profile.meter.default_bpm,
        time_signature: profile.meter.time_signature,
        swing_ratio: profile.meter.default_swing_ratio,
        harmony_valence: profile.dynamics.neutral_valence,
        feet_density: density,
        ..Default::default()
    }
}

fn stream(profile: &StyleProfile, seed: u64, density: u8, bars: usize) -> Vec<MeasureSnapshot> {
    let mut generator =
        TradGenerator::new(params(profile, density), TuningParams { profile: profile.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect()
}

fn grid(snapshot: &MeasureSnapshot, tracks: &[u8]) -> Grid {
    let mut notes: Grid = snapshot
        .notes
        .iter()
        .filter(|note| tracks.contains(&note.track))
        .map(|note| (note.track, note.pitch, note.start_step, note.duration_steps))
        .collect();
    notes.sort_unstable();
    notes
}

fn harmony(snapshots: &[MeasureSnapshot]) -> Vec<(i32, bool)> {
    snapshots.iter().map(|s| (s.chord_root_offset, s.chord_is_minor)).collect()
}

#[test]
fn graines_isaac_varient_accords_et_grilles_et_se_rejouent_exactement() -> TestResult {
    for id in ISAAC {
        let profile = StyleProfile::named(id)?;
        let bars = profile.form.bars_per_tour();
        for density in [1, 2] {
            let first = stream(&profile, 1, density, bars);
            let drums = |snapshots: &[MeasureSnapshot]| {
                snapshots.iter().map(|s| grid(s, &[3, 4])).collect::<Vec<_>>()
            };
            // Un tirage pondéré peut légitimement répéter le même arrangement.
            let distinct = (2..=16).find(|seed| {
                let other = stream(&profile, *seed, density, bars);
                harmony(&first) != harmony(&other) && drums(&first) != drums(&other)
            });
            assert!(
                distinct.is_some(),
                "{id}, aucune variété des deux couches à densité {density}"
            );
            assert_eq!(
                serde_json::to_value(&first)?,
                serde_json::to_value(stream(&profile, 1, density, bars))?,
                "{id}, déterminisme densité {density}"
            );
        }
    }
    Ok(())
}

#[test]
fn retours_de_partie_et_second_tour_conservent_les_choix() -> TestResult {
    for id in ISAAC {
        let mut profile = StyleProfile::named(id)?;
        // Isoler l'identité des patterns de la rampe d'intensité, qui peut changer
        // la densité effective entre les deux tours.
        profile.dynamics.feet_intensity.0 = vec![[0.0, 0.0], [1.0, 0.0]];
        profile.dynamics.density_feet.0 = vec![[0.0, 0.0], [1.0, 0.0]];
        // Downpour n'a qu'un tour en données ; exercer aussi son second tour.
        profile.arrangement.tours_per_tune = 2;
        let bars = profile.form.bars_per_tour();
        for seed in [1, 2] {
            let snapshots = stream(&profile, seed, 1, bars * 2);
            let mut written = BTreeMap::new();
            for snapshot in &snapshots {
                let position = snapshot.position.as_ref().ok_or("position absente")?;
                if position.is_intro() {
                    continue;
                }
                let key = (position.part.0, position.bar_in_strain);
                let score =
                    (snapshot.chord_root_offset, snapshot.chord_is_minor, grid(snapshot, &[3, 4]));
                if let Some(previous) = written.insert(key, score.clone()) {
                    assert_eq!(previous, score, "{id}, retour {key:?}, graine {seed}");
                }
            }
        }
    }
    Ok(())
}

#[test]
fn fixture_gigue_exerce_les_deux_couches_et_les_airs_renouvellent_leurs_choix() -> TestResult {
    let profile = StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/gigue_bourdon",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let bars = profile.form.bars_per_tour();
    let mut harmonies = std::collections::BTreeSet::new();
    let mut drums = std::collections::BTreeSet::new();
    for seed in 1..=12 {
        let snapshots = stream(&profile, seed, 1, bars);
        harmonies.insert(harmony(&snapshots));
        drums.insert(snapshots.iter().map(|s| grid(s, &[3, 4])).collect::<Vec<_>>());
    }
    assert!(harmonies.len() > 1, "boucles alternatives absentes hors Isaac");
    assert!(drums.len() > 1, "patterns alternatifs absents hors Isaac");

    let profile = StyleProfile::named("isaac_basement")?;
    let air_bars = profile.form.bars_per_tour() * profile.arrangement.tours_per_tune;
    let snapshots = stream(&profile, 1, 1, air_bars * 4);
    let choices: Vec<_> = snapshots.chunks_exact(air_bars).map(harmony).collect();
    assert!(
        choices.windows(2).any(|pair| pair[0] != pair[1]),
        "les accords ne sont jamais retirés aux airs suivants"
    );
    let choices: Vec<_> = snapshots
        .chunks_exact(air_bars)
        .map(|air| air.iter().map(|s| grid(s, &[3, 4])).collect::<Vec<_>>())
        .collect();
    assert!(
        choices.windows(2).any(|pair| pair[0] != pair[1]),
        "la batterie n'est jamais retirée aux airs suivants"
    );
    Ok(())
}

#[test]
fn changer_de_densite_ne_retire_pas_les_patterns_ni_les_accords() -> TestResult {
    let profile = StyleProfile::named("isaac_basement")?;
    let bars = profile.form.bars_per_tour() * profile.arrangement.tours_per_tune * 2;
    let reference = stream(&profile, 1, 1, bars);
    let mut generator =
        TradGenerator::new(params(&profile, 1), TuningParams { profile: profile.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    for (index, expected) in reference.iter().enumerate() {
        generator.musical_params.feet_density = if index % 2 == 0 { 2 } else { 1 };
        let actual =
            MeasureSnapshot::from_measure(&generator.generate_measure(index + 1, &mut rng));
        assert_eq!(
            (actual.chord_root_offset, actual.chord_is_minor),
            (expected.chord_root_offset, expected.chord_is_minor)
        );
        if index % 2 == 1 {
            assert_eq!(
                grid(&actual, &[3, 4]),
                grid(expected, &[3, 4]),
                "le retour à densité 1 change le pattern, mesure {}",
                index + 1
            );
        }
    }
    Ok(())
}

#[test]
fn melodie_realisee_sur_la_boucle_choisie_y_compris_hors_isaac() -> TestResult {
    let mut profiles = ISAAC.into_iter().map(StyleProfile::named).collect::<Result<Vec<_>, _>>()?;
    profiles.push(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/gigue_bourdon",
        env!("CARGO_MANIFEST_DIR")
    ))?);
    for profile in profiles {
        for seed in [1, 2, 3] {
            for snapshot in stream(&profile, seed, 1, profile.form.bars_per_tour()) {
                let chord: std::collections::BTreeSet<_> = snapshot
                    .notes
                    .iter()
                    .filter(|note| note.track == 1)
                    .map(|note| note.pitch % 12)
                    .collect();
                for note in snapshot.notes.iter().filter(|note| note.track == 0) {
                    assert!(
                        chord.contains(&(note.pitch % 12)),
                        "{} : note de chœur hors boucle choisie, mesure {} : {note:?}",
                        profile.meta.id,
                        snapshot.index
                    );
                }
            }
        }
    }
    Ok(())
}

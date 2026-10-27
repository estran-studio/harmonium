//! G1 (phase 014) : les modes phrygien et mineur harmonique portent la
//! grammaire d'un profil trad existant (copies du reel québécois).
//!
//! Contrat sur le flux de snapshots, plusieurs graines : chaque note de
//! mélodie appartient à la gamme du mode publié, et le V joué en mineur
//! harmonique est majeur.

use harmonium_core::{
    StyleProfile, TuningParams,
    params::MusicalParams,
    report::MeasureSnapshot,
    trad::{Mode, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn load(id: &str) -> Result<StyleProfile, Box<dyn std::error::Error>> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/{id}",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

fn stream(
    id: &str,
    seed: u64,
    bars: usize,
) -> Result<Vec<MeasureSnapshot>, Box<dyn std::error::Error>> {
    let p = load(id)?;
    let mut params = MusicalParams::default();
    params.bpm = p.meter.default_bpm;
    params.time_signature = p.meter.time_signature;
    params.set_swing_ratio(p.meter.default_swing_ratio);
    // À la valence neutre, la couleur vient de `key_modes` : un seul candidat.
    params.harmony_valence = p.dynamics.neutral_valence;
    let mut generator =
        TradGenerator::new(params, TuningParams { profile: p, ..Default::default() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng)))
        .collect())
}

fn check_melody_in_mode(id: &str, mode: Mode, key: u8) -> Result<(), Box<dyn std::error::Error>> {
    for seed in 1..=4 {
        let snapshots = stream(id, seed, 64)?;
        assert_eq!(snapshots.len(), 64, "{id} graine {seed}");
        let mut melody_notes = 0;
        for snapshot in &snapshots {
            let position = snapshot.position.ok_or("position absente")?;
            assert_eq!((position.key, position.mode), (key, mode), "{id} graine {seed}");
            for note in snapshot.notes.iter().filter(|n| n.track == 0) {
                assert!(
                    mode.contains_midi(key, note.pitch),
                    "{id} graine {seed} mesure {} : {:?} hors du mode",
                    snapshot.index,
                    note
                );
                melody_notes += 1;
            }
        }
        assert!(melody_notes > 100, "{id} graine {seed} : trop peu de notes");
    }
    Ok(())
}

#[test]
fn phrygian_fixture_keeps_reel_melodies_in_the_phrygian_scale()
-> Result<(), Box<dyn std::error::Error>> {
    check_melody_in_mode("reel_phrygien", Mode::Phrygian, 2)
}

#[test]
fn harmonic_minor_fixture_keeps_reel_melodies_in_scale_with_major_v()
-> Result<(), Box<dyn std::error::Error>> {
    check_melody_in_mode("reel_mineur_harmonique", Mode::HarmonicMinor, 2)?;
    // Le V joué par l'harmonie (premier accord publié, cadences imposées)
    // est majeur : sensible. En ré : A majeur.
    let mut dominants = 0;
    for seed in 1..=4 {
        for snapshot in stream("reel_mineur_harmonique", seed, 64)? {
            if snapshot.chord_root_offset == 7 {
                assert!(
                    !snapshot.chord_is_minor,
                    "graine {seed} mesure {} : {} mineur en mineur harmonique",
                    snapshot.index, snapshot.chord_name
                );
                assert_eq!(snapshot.chord_name, "A", "graine {seed} mesure {}", snapshot.index);
                dominants += 1;
            }
        }
    }
    assert!(dominants > 0, "aucun V joué en 4 × 64 mesures");
    Ok(())
}

#[test]
fn valence_switches_phrygian_with_mixolydian_and_harmonic_minor_with_ionian()
-> Result<(), Box<dyn std::error::Error>> {
    for (id, dark_mode, bright_mode) in [
        ("reel_phrygien", Mode::Phrygian, Mode::Mixolydian),
        ("reel_mineur_harmonique", Mode::HarmonicMinor, Mode::Ionian),
    ] {
        let p = load(id)?;
        let mut params = MusicalParams::default();
        params.bpm = p.meter.default_bpm;
        params.time_signature = p.meter.time_signature;
        params.set_swing_ratio(p.meter.default_swing_ratio);
        params.harmony_valence = p.dynamics.neutral_valence;
        let mut generator =
            TradGenerator::new(params.clone(), TuningParams { profile: p, ..Default::default() });
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let mut saw_dark = false;
        let mut saw_bright = false;
        let mut returned_dark = false;
        for bar in 1..=32 {
            if bar == 4 {
                params.harmony_valence = 0.9;
                generator.invalidate_future(3);
                generator.update_params(params.clone());
            } else if bar == 12 {
                params.harmony_valence = -0.9;
                generator.invalidate_future(11);
                generator.update_params(params.clone());
            }
            let s = MeasureSnapshot::from_measure(&generator.generate_measure(bar, &mut rng));
            let pos = s.position.ok_or("position absente")?;
            if pos.is_intro() {
                continue;
            }
            if pos.mode == dark_mode && !saw_bright {
                saw_dark = true;
            } else if pos.mode == bright_mode {
                saw_bright = true;
            } else if pos.mode == dark_mode && saw_bright {
                returned_dark = true;
            }
            for note in s.notes.iter().filter(|n| n.track == 0) {
                assert!(
                    pos.mode.contains_midi(pos.key, note.pitch),
                    "{id} mesure {bar} ({:?}) : {note:?}",
                    pos.mode
                );
            }
        }
        assert!(saw_dark && saw_bright && returned_dark, "{id} : bascule incomplète");
    }
    Ok(())
}

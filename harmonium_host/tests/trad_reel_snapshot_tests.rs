//! Tests d'acceptation sur le flux de snapshots du moteur en `Style::Reel`
//! via `create_offline_engine` (20 seeds, 64 mesures, critères 1–4 du plan + seek déterministe).

use harmonium::{audio::create_offline_engine, playback::PlaybackCommand};
use harmonium_core::{
    params::{PulseSpec, Style, TimeSignature},
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::TrackId,
    trad::{TunePlan, is_same_or_relative_plan},
};

const SAMPLE_RATE: f64 = 44100.0;
const NUM_SEEDS: u64 = 20;
const BARS_64: usize = 64;

/// Génère 64 mesures de `Style::Reel` via `create_offline_engine` pour un seed et une densité donnés.
fn generate_offline_reel_stream(
    seed: u64,
    feet_density: u8,
) -> Result<(harmonium::composer::MusicComposer, Vec<MeasureSnapshot>), String> {
    let (mut composer, _playback, _cmd_tx, _report_rx, _clock_reader, _recordings) =
        create_offline_engine(None, None, SAMPLE_RATE)?;

    composer.set_style(Style::Reel);
    composer.set_feet_density(feet_density);
    composer.set_seed(seed);
    composer.set_writehead_lookahead(BARS_64 + 4);
    composer.generate_bars(BARS_64);

    let snapshots = composer.take_snapshots();
    Ok((composer, snapshots))
}

fn notes_on_track(snapshot: &MeasureSnapshot, track: TrackId) -> Vec<&NoteSnapshot> {
    let ch = track.channel();
    snapshot.notes.iter().filter(|n| n.track == ch).collect()
}

fn note_tuples(snapshot: &MeasureSnapshot) -> Vec<(u8, u8, usize, usize, u8)> {
    snapshot
        .notes
        .iter()
        .map(|n| (n.track, n.pitch, n.start_step, n.duration_steps, n.velocity))
        .collect()
}

fn track_tuples(snapshot: &MeasureSnapshot, track: TrackId) -> Vec<(u8, usize, usize, u8)> {
    notes_on_track(snapshot, track)
        .into_iter()
        .map(|n| (n.pitch, n.start_step, n.duration_steps, n.velocity))
        .collect()
}

fn cadence_degree_bar8(plan: &TunePlan, melody: &[&NoteSnapshot]) -> Option<u8> {
    let last = melody.last()?;
    if plan.mode.scale_degree_of_midi(plan.key, last.pitch) == Some(1) {
        return Some(1);
    }
    if melody.len() >= 2 && last.start_step >= 12 {
        let penultimate = melody[melody.len() - 2];
        if penultimate.duration_steps >= 4 {
            return plan.mode.scale_degree_of_midi(plan.key, penultimate.pitch);
        }
    }
    plan.mode.scale_degree_of_midi(plan.key, last.pitch)
}

fn assert_snapshots_equal(a: &[MeasureSnapshot], b: &[MeasureSnapshot], context: &str) {
    assert_eq!(a.len(), b.len(), "{context}: nombre de snapshots différent");
    for (i, (ma, mb)) in a.iter().zip(b.iter()).enumerate() {
        let ja = serde_json::to_string(ma).unwrap_or_default();
        let jb = serde_json::to_string(mb).unwrap_or_default();
        assert!(!ja.is_empty() && !jb.is_empty());
        assert_eq!(ja, jb, "{context}: différence à la mesure {}", i + 1);
    }
}

/// Critères 1, 2, 3 du plan sur le flux de snapshots via `create_offline_engine` (20 seeds, 64 mesures).
#[test]
fn reel_snapshot_stream_criteria_1_to_3_across_20_seeds_64_bars() -> Result<(), String> {
    for seed in 0..NUM_SEEDS {
        let (composer, snapshots) = generate_offline_reel_stream(seed, 1)?;
        assert_eq!(snapshots.len(), BARS_64, "seed {seed}: 64 snapshots attendus");
        for (idx, snap) in snapshots.iter().enumerate() {
            assert_eq!(snap.index, idx + 1);
            assert_eq!(snap.time_sig_numerator, 2);
            assert_eq!(snap.time_sig_denominator, 2);
            assert_eq!(snap.steps, 16);
            assert_eq!(snap.pulse, PulseSpec { steps_per_pulse: 8, pulses_per_bar: 2 });
            assert!((snap.swing_ratio - 0.56).abs() < f32::EPSILON);
            assert!((snap.tempo - 230.0).abs() < f32::EPSILON);
            assert!(!snap.chord_name.is_empty());

            // Critère 3 : accompagnement D8 et podorythmie D9 (densité 1)
            let bass = notes_on_track(snap, TrackId::Bass);
            assert_eq!(
                bass.iter().map(|n| (n.start_step, n.duration_steps)).collect::<Vec<_>>(),
                vec![(0, 4), (8, 4)],
                "seed {seed} bar {}: basse D8",
                snap.index
            );

            let chords = notes_on_track(snap, TrackId::Chords);
            assert_eq!(chords.len(), 6, "seed {seed} bar {}: 2 chops de 3 notes", snap.index);
            assert_eq!(
                chords.iter().filter(|n| n.start_step == 4 && n.duration_steps == 1).count(),
                3
            );
            assert_eq!(
                chords.iter().filter(|n| n.start_step == 12 && n.duration_steps == 1).count(),
                3
            );

            let heel = notes_on_track(snap, TrackId::Heel);
            let toe = notes_on_track(snap, TrackId::Toe);
            assert_eq!(
                heel.iter().map(|n| (n.start_step, n.pitch, n.duration_steps)).collect::<Vec<_>>(),
                vec![(0, 36, 0), (4, 36, 0), (8, 36, 0), (12, 36, 0)]
            );
            assert_eq!(
                toe.iter().map(|n| (n.start_step, n.pitch, n.duration_steps)).collect::<Vec<_>>(),
                vec![(2, 37, 0), (6, 37, 0), (10, 37, 0), (14, 37, 0)]
            );
        }

        // Critères 1 et 2 sur les deux tunes (mesures 1..=32 et 33..=64)
        for tune_offset in [0usize, 32] {
            let (plan, _) = composer
                .reel_bar_info(tune_offset + 1)
                .ok_or_else(|| "plan du tune absent dans le composer".to_string())?;
            // Reprises A1 == A2 et B1 == B2 (notes et harmonie identiques)
            for b in 0..8 {
                let a1 = &snapshots[tune_offset + b];
                let a2 = &snapshots[tune_offset + 8 + b];
                assert_eq!(note_tuples(a1), note_tuples(a2), "seed {seed}: A1 != A2 à m.{}", b + 1);
                assert_eq!(a1.chord_name, a2.chord_name);
                assert_eq!(a1.chord_root_offset, a2.chord_root_offset);

                let b1 = &snapshots[tune_offset + 16 + b];
                let b2 = &snapshots[tune_offset + 24 + b];
                assert_eq!(note_tuples(b1), note_tuples(b2), "seed {seed}: B1 != B2 à m.{}", b + 1);
                assert_eq!(b1.chord_name, b2.chord_name);
                assert_eq!(b1.chord_root_offset, b2.chord_root_offset);
            }

            // Par passe de strain (A1, A2, B1, B2) : mesures 5-6 = 1-2 et cadences en 4 et 8
            for strain_start in [0usize, 8, 16, 24].map(|s| tune_offset + s) {
                assert_eq!(
                    track_tuples(&snapshots[strain_start + 4], TrackId::Melody),
                    track_tuples(&snapshots[strain_start], TrackId::Melody),
                    "seed {seed}: m.5 != m.1"
                );
                assert_eq!(
                    track_tuples(&snapshots[strain_start + 5], TrackId::Melody),
                    track_tuples(&snapshots[strain_start + 1], TrackId::Melody),
                    "seed {seed}: m.6 != m.2"
                );

                let m4 = notes_on_track(&snapshots[strain_start + 3], TrackId::Melody);
                let last4 = m4.last().ok_or_else(|| "m.4 vide".to_string())?;
                let deg4 = plan.mode.scale_degree_of_midi(plan.key, last4.pitch);
                assert!(
                    matches!(deg4, Some(2 | 5 | 7)),
                    "seed {seed}: m.4 finit sur {deg4:?} (attendu 2, 5 ou 7)"
                );

                let m8 = notes_on_track(&snapshots[strain_start + 7], TrackId::Melody);
                assert_eq!(
                    cadence_degree_bar8(&plan, &m8),
                    Some(1),
                    "seed {seed}: m.8 doit finir sur la tonique (degré 1)"
                );
                assert_eq!(
                    snapshots[strain_start + 7].chord_root_offset,
                    7,
                    "seed {seed}: m.8 1re moitié doit être sur V"
                );
            }
        }

        let (plan1, _) = composer.reel_bar_info(1).ok_or_else(|| "tune 1 absent".to_string())?;
        let (plan2, _) = composer.reel_bar_info(33).ok_or_else(|| "tune 2 absent".to_string())?;
        assert!(is_same_or_relative_plan(plan1, plan2));
    }
    Ok(())
}

/// Critère 4 : Déterminisme, diversité des seeds, `set_seed` et seek déterministe en `Style::Reel`.
#[test]
fn reel_determinism_set_seed_and_deterministic_seek() -> Result<(), String> {
    for seed in 0..NUM_SEEDS {
        let (_, run_a) = generate_offline_reel_stream(seed, 1)?;
        let (_, run_b) = generate_offline_reel_stream(seed, 1)?;
        assert_snapshots_equal(&run_a, &run_b, &format!("même seed {seed}"));
    }

    // Deux seeds différents produisent des flux différents
    for seed in 0..(NUM_SEEDS - 1) {
        let (_, a) = generate_offline_reel_stream(seed, 1)?;
        let (_, b) = generate_offline_reel_stream(seed + 1, 1)?;
        assert_ne!(
            serde_json::to_string(&a).unwrap_or_default(),
            serde_json::to_string(&b).unwrap_or_default(),
            "seeds {seed} et {} ont produit le même flux",
            seed + 1
        );
    }

    // Seek déterministe en Reel : sauter à la mesure 19 (strain B du tune 1) puis à la mesure 41 (strain A2 du tune 2)
    for &target_bar in &[1usize, 12, 19, 33, 41] {
        let seed = 7u64;
        let (_, linear) = generate_offline_reel_stream(seed, 1)?;

        let (mut composer, _playback, _cmd_tx, _report_rx, _clock_reader, _recordings) =
            create_offline_engine(None, None, SAMPLE_RATE)?;
        composer.set_style(Style::Reel);
        composer.set_seed(seed);
        composer.set_writehead_lookahead(BARS_64 + 4);
        composer.generate_bars(4);
        let _ = composer.take_snapshots();

        composer.deterministic_seek(target_bar);
        composer.generate_bars(BARS_64);
        let sought_snapshots = composer.take_snapshots();

        let expected: Vec<MeasureSnapshot> =
            linear.into_iter().filter(|m| m.index >= target_bar).collect();
        assert_snapshots_equal(
            &expected,
            &sought_snapshots,
            &format!("seek déterministe en Reel vers la mesure {target_bar}"),
        );
    }
    Ok(())
}

/// Changement de style : défauts issus de `TuningParams` et émission des programmes GM D10.
#[test]
fn set_style_applies_tuning_defaults_and_queues_gm_programs() -> Result<(), String> {
    let (mut composer, mut playback, mut cmd_tx, _report_rx, _clock_reader, _recordings) =
        create_offline_engine(None, None, SAMPLE_RATE)?;

    composer.set_style(Style::Reel);
    assert_eq!(composer.musical_params().style, Style::Reel);
    assert!((composer.musical_params().bpm - 230.0).abs() < f32::EPSILON);
    assert!((composer.musical_params().swing_ratio - 0.56).abs() < f32::EPSILON);
    assert_eq!(composer.musical_params().time_signature, TimeSignature::new(2, 2));

    let reel_progs = composer.take_pending_program_changes();
    assert_eq!(reel_progs, vec![(0, 40), (1, 24), (2, 32)]);
    for (channel, program) in reel_progs {
        let _ = cmd_tx.push(PlaybackCommand::ProgramChange { channel, program });
    }

    composer.set_style(Style::Legacy);
    assert_eq!(composer.musical_params().style, Style::Legacy);
    assert!((composer.musical_params().bpm - 120.0).abs() < f32::EPSILON);
    assert!((composer.musical_params().swing_ratio - 0.5).abs() < f32::EPSILON);
    assert_eq!(composer.musical_params().time_signature, TimeSignature::new(4, 4));

    let legacy_progs = composer.take_pending_program_changes();
    assert_eq!(legacy_progs, vec![(0, 0), (1, 24), (2, 32)]);
    for (channel, program) in legacy_progs {
        let _ = cmd_tx.push(PlaybackCommand::ProgramChange { channel, program });
    }

    // Le PlaybackEngine traite les ProgramChange sans allocation ni erreur
    let mut buf = [0.0f32; 512];
    playback.process_buffer(&mut buf, 2, std::time::Instant::now());
    Ok(())
}

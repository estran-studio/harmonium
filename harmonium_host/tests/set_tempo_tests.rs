//! La commande du jeu garde la priorité sur les paliers de set et les émotions.
use harmonium::audio::create_offline_engine;

#[test]
fn explicit_bpm_survives_emotions_and_reset_restores_the_set_ramp() -> Result<(), String> {
    let (mut composer, _playback, _commands, _reports, _clock, _recordings) =
        create_offline_engine(None, None, &[], 44100.0)?;
    composer.set_bpm(197.0);
    composer.use_emotion_mode();
    composer.set_emotions(0.8, 0.3, 0.7, 0.5);
    composer.sync_generator();
    composer.generate_bars(4);
    assert!(composer.take_snapshots().iter().all(|s| s.tempo == 197.0));
    composer.reset_bpm();
    composer.reset_timeline();
    composer.generate_bars(4);
    let snapshots = composer.take_snapshots();
    assert!(snapshots.iter().all(|s| s.tempo != 197.0));
    assert!(snapshots.iter().all(|s| s.tempo > 200.0));
    Ok(())
}

#[test]
fn game_stop_targets_the_playing_air_not_the_pregenerated_air() -> Result<(), String> {
    let (mut composer, _playback, _commands, _reports, _clock, _recordings) =
        create_offline_engine(None, None, &[], 44100.0)?;
    composer.set_seed(3);
    composer.generate_bars(100);
    let ahead = composer.take_snapshots();
    assert!(
        ahead.iter().any(|s| s.position.is_some_and(|p| p.tune_index > 0) && !s.notes.is_empty())
    );
    composer.set_end_after_tour(true);
    composer.invalidate_future();
    composer.generate_bars(100);
    let after = composer.take_snapshots();
    assert!(
        after
            .iter()
            .filter(|s| s.position.is_some_and(|p| p.tune_index > 0))
            .all(|s| s.notes.is_empty())
    );
    let ending = after
        .iter()
        .rev()
        .find(|s| s.position.is_some_and(|p| p.tune_index == 0))
        .ok_or("fin de l'air courant absente")?;
    let melody: Vec<_> = ending.notes.iter().filter(|n| n.track == 0).collect();
    assert_eq!(melody.len(), 1);
    assert_eq!(melody[0].pitch % 12, ending.position.ok_or("position absente")?.key);
    Ok(())
}

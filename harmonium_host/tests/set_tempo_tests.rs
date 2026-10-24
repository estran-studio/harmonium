//! La commande du jeu garde la priorité sur les paliers de set et les émotions.
use harmonium::{audio::create_offline_engine};

#[test]
fn explicit_bpm_survives_emotions_and_reset_restores_the_set_ramp() -> Result<(), String> {
    let (mut composer, _playback, _commands, _reports, _clock, _recordings) =
        create_offline_engine(None, None, 44100.0)?;
    composer.set_bpm(197.0);
    composer.use_emotion_mode();
    composer.set_emotions(0.8,0.3,0.7,0.5);
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

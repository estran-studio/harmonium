//! E11 côté hôte : l'alternance de mesures jouées et muettes se déclare dans l'arrangement du
//! profil ; `SetStyleProfile` la met en place (et la retire) sur les mesures encore à composer,
//! sans nouvelle commande.
use harmonium::audio::create_offline_engine;
use harmonium_core::{
    StyleProfile,
    compose::{Alternation, IntroConfig},
    report::MeasureSnapshot,
    timeline::TrackId,
};

fn profile(alternation: Option<Alternation>) -> Result<StyleProfile, String> {
    let p = StyleProfile::distributed().iter().find(|p| p.meta.id == "reel_britannique_coloniale");
    let mut p = p.cloned().ok_or("profil distribué")?;
    p.arrangement.intro = IntroConfig::none();
    if let Some(alternation) = alternation {
        p.arrangement.alternation.insert(TrackId::Melody, alternation);
    }
    p.validate().map_err(|e| e.to_string())?;
    Ok(p)
}

fn melody_bars(snapshots: &[MeasureSnapshot]) -> Vec<usize> {
    snapshots
        .iter()
        .filter(|s| s.notes.iter().any(|n| n.track == TrackId::Melody.channel()))
        .map(|s| s.index)
        .collect()
}

#[test]
fn set_style_profile_met_l_alternance_en_place_et_la_retire() -> Result<(), String> {
    let (mut composer, _playback, _commands, _reports, _clock, _recordings) =
        create_offline_engine(None, None, &[], 44100.0)?;
    composer.set_profile(profile(None)?)?;
    composer.set_seed(3);
    composer.generate_bars(8);
    assert_eq!(melody_bars(&composer.take_snapshots()), (1..=8).collect::<Vec<_>>());

    composer.set_profile(profile(Some(Alternation { period: 4, play: None, phase: 0 }))?)?;
    composer.generate_bars(8);
    assert_eq!(melody_bars(&composer.take_snapshots()), [1, 2, 5, 6]);

    composer.set_profile(profile(None)?)?;
    composer.generate_bars(8);
    assert_eq!(melody_bars(&composer.take_snapshots()), (1..=8).collect::<Vec<_>>());
    Ok(())
}

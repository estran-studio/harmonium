//! Les liens d'ouverture : ce qu'ils disent (lecture), ce qu'ils redonnent (écriture) et ce
//! qu'ils refusent. Pur : ni moteur, ni fenêtre.

use std::path::PathBuf;

use harmonium_core::timeline::TrackId;
use harmonium_studio::link::{OpenRequest, ProfileRef, percent_decode, percent_encode};

fn parse(link: &str) -> Result<OpenRequest, String> {
    OpenRequest::parse(link)
}

#[test]
fn the_documented_link_reads_every_field() -> Result<(), String> {
    let request = parse(
        "harmonium://open?profile=/tmp/wt/profiles/reel_quebecois&seed=3&bar=17&loop=17-24\
         &bpm=180&solo=Bass&mute=Melody,Chords&play=0",
    )?;
    assert_eq!(
        request,
        OpenRequest {
            profile: Some(ProfileRef::Folder(PathBuf::from("/tmp/wt/profiles/reel_quebecois"))),
            seed: Some(3),
            bar: Some(17),
            loop_range: Some((17, 24)),
            bpm: Some(180.0),
            swing: None,
            solo: Some(TrackId::Bass),
            mute: vec![TrackId::Melody, TrackId::Chords],
            play: false,
        }
    );
    Ok(())
}

#[test]
fn a_link_stays_paused_unless_it_asks_to_play() -> Result<(), String> {
    assert!(!parse("harmonium://open?profile=reel_quebecois")?.play);
    assert!(!parse("harmonium://open")?.play);
    assert!(parse("harmonium://open?play=1")?.play);
    assert!(!OpenRequest { play: false, ..Default::default() }.to_link().contains("play"));
    Ok(())
}

#[test]
fn writing_then_reading_gives_the_same_request() -> Result<(), String> {
    let request = OpenRequest {
        // Espaces, accents, `+`, `&` et `%` : tout ce qu'un chemin de worktree peut contenir.
        profile: Some(ProfileRef::Folder(PathBuf::from("/Users/élise/my work/a+b&c%d/profiles/x"))),
        seed: Some(u64::MAX),
        bar: Some(1),
        loop_range: Some((5, 5)),
        bpm: Some(172.5),
        swing: Some(0.62),
        solo: Some(TrackId::Countermelody),
        mute: vec![TrackId::PercussionLow, TrackId::Melody],
        play: true,
    };
    let link = request.to_link();
    assert!(!link.contains(' ') && !link.contains("é"), "lien opaque : {link}");
    assert_eq!(parse(&link)?, request);

    let by_id = OpenRequest {
        profile: Some(ProfileRef::Id("valse_quebecoise".into())),
        ..Default::default()
    };
    assert_eq!(by_id.to_link(), "harmonium://open?profile=valse_quebecoise");
    assert_eq!(parse(&by_id.to_link())?, by_id);
    Ok(())
}

#[test]
fn percent_coding_round_trips_any_text() -> Result<(), String> {
    for text in ["", "plain/path,list:1", "a b", "é€𝄞", "100%", "a+b&c=d?e#f"] {
        assert_eq!(percent_decode(&percent_encode(text))?, text);
    }
    assert_eq!(percent_decode("a+b")?, "a+b", "`+` n'est pas une espace");
    assert!(percent_decode("%zz").is_err());
    assert!(percent_decode("%+1").is_err());
    assert!(percent_decode("%4").is_err());
    assert!(percent_decode("%ff").is_err(), "UTF-8 invalide");
    Ok(())
}

#[test]
fn track_names_ignore_case_and_counter_is_the_countermelody() -> Result<(), String> {
    let request = parse("harmonium://open?solo=counter&mute=percussionhigh,BASS,bass")?;
    assert_eq!(request.solo, Some(TrackId::Countermelody));
    assert_eq!(request.mute, vec![TrackId::PercussionHigh, TrackId::Bass], "sans doublon");
    // `mute` se répète.
    let twice = parse("harmonium://open?mute=Melody&mute=Chords")?;
    assert_eq!(twice.mute, vec![TrackId::Melody, TrackId::Chords]);
    Ok(())
}

#[test]
fn bad_links_are_refused_with_the_reason() {
    let refused = |link: &str, reason: &str| {
        let error = parse(link).err().unwrap_or_default();
        assert!(error.contains(reason), "`{link}` → `{error}` ne dit pas `{reason}`");
    };
    refused("https://open?seed=1", "not a harmonium://");
    refused("harmonium://close?seed=1", "unknown action");
    refused("harmonium://open?sed=1", "sed: unknown parameter");
    refused("harmonium://open?seed=1&seed=2", "given twice");
    refused("harmonium://open?seed=-1", "invalid seed");
    refused("harmonium://open?bar=0", "invalid bar");
    refused("harmonium://open?loop=24-17", "invalid loop");
    refused("harmonium://open?loop=17", "invalid loop");
    refused("harmonium://open?bpm=500", "invalid value");
    refused("harmonium://open?bpm=nan", "invalid value");
    refused("harmonium://open?swing=0.3", "invalid value");
    refused("harmonium://open?solo=Banjo", "unknown track");
    refused("harmonium://open?play=maybe", "invalid value");
    refused("harmonium://open?profile=profiles/x", "absolute path");
    refused("harmonium://open?profile=Reel%20Quebecois", "expected a profile id");
    refused("harmonium://open?profile=a%2", "invalid %-escape");
}

#[test]
fn windows_paths_are_folders_too() -> Result<(), String> {
    let request = parse("harmonium://open?profile=C%3A%5CUsers%5Cme%5Cprofiles%5Cx")?;
    assert_eq!(request.profile, Some(ProfileRef::Folder(PathBuf::from(r"C:\Users\me\profiles\x"))));
    Ok(())
}

#[test]
fn a_page_query_reads_like_a_link_and_ignores_the_studio_settings() -> Result<(), String> {
    let request =
        OpenRequest::from_query("?buffer=1024&profile=reel_quebecois&seed=3", &["buffer"])?;
    assert_eq!(request.profile, Some(ProfileRef::Id("reel_quebecois".into())));
    assert_eq!(request.seed, Some(3));
    assert_eq!(OpenRequest::from_query("?buffer=1024", &["buffer"])?, OpenRequest::default());
    assert_eq!(OpenRequest::from_query("", &["buffer"])?, OpenRequest::default());
    assert!(
        OpenRequest::from_query("?buffer=1024", &[]).is_err(),
        "sans l'exception, `buffer` est inconnu"
    );
    // Ce que le web recopie dans la barre d'adresse se relit.
    let query = request.to_query();
    assert_eq!(query, "?profile=reel_quebecois&seed=3");
    assert_eq!(OpenRequest::from_query(&query, &[])?, request);
    Ok(())
}

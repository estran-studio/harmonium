//! Ressources désignées par un profil et absentes de l'hôte : avertissement
//! clair au chargement du profil (jamais de repli implicite), notes inchangées.
//!
//! Les avertissements sont lus dans le journal fichier du processus (le canal
//! réel de `harmonium_core::log`). Chaque test charge un profil sous un
//! identifiant unique pour ne lire que ses propres lignes.

use std::sync::Once;

use harmonium::audio::create_offline_engine;
use harmonium_core::{StyleProfile, host_resources::MissingResource, timeline::TrackId};

const SAMPLE_RATE: f64 = 44100.0;
/// Banque de test : tient lieu de banque de pieds chargée (banque 1).
const FEET_BANK: &[u8] = include_bytes!("../../assets/test.sf2");

fn log_path() -> std::path::PathBuf {
    static INIT: Once = Once::new();
    let dir = std::env::temp_dir().join(format!("harmonium_resources_{}", std::process::id()));
    INIT.call_once(|| {
        std::fs::create_dir_all(&dir).unwrap();
        harmonium_core::log::init_file_logging(&dir);
    });
    dir.join("harmonium.log")
}

/// Lignes du journal qui concernent le profil `id`.
fn warnings_for(id: &str) -> Vec<String> {
    let marker = format!("Profil « {id} »");
    std::fs::read_to_string(log_path())
        .unwrap_or_default()
        .lines()
        .filter(|line| line.contains("WARN") && line.contains(&marker))
        .map(str::to_owned)
        .collect()
}

/// Profil distribué ou fixture, rebaptisé `id`.
fn profile(path: &str, id: &str) -> StyleProfile {
    let mut profile =
        StyleProfile::load(format!("{}/../{path}", env!("CARGO_MANIFEST_DIR"))).unwrap();
    profile.meta.id = id.to_owned();
    profile
}

fn engine(feet: Option<&[u8]>) -> harmonium::composer::MusicComposer {
    log_path();
    let (composer, ..) = create_offline_engine(None, feet, &[], SAMPLE_RATE).unwrap();
    composer
}

#[test]
fn an_unloaded_percussion_bank_is_reported_when_the_profile_loads() {
    let mut composer = engine(None);
    composer.set_profile(profile("profiles/reel_quebecois", "avert_banque")).unwrap();

    let warnings = warnings_for("avert_banque");
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("banque de percussion 1 non chargée"), "{}", warnings[0]);
    assert_eq!(composer.missing_resources(), [MissingResource::PercussionBank { bank: 1 }]);
}

#[test]
fn a_loaded_feet_bank_raises_no_warning() {
    let mut composer = engine(Some(FEET_BANK));
    composer.set_profile(profile("profiles/reel_quebecois", "sans_avert")).unwrap();

    assert!(warnings_for("sans_avert").is_empty());
    assert!(composer.missing_resources().is_empty());
}

#[test]
fn an_unprovided_odin_preset_is_reported_for_another_style_too() {
    // Valse 3/4 (autre style que le reel) : même mécanisme, preset non fourni.
    let mut composer = engine(None);
    composer.set_profile(profile("tests/fixtures/profiles/valse_nappe", "avert_odin")).unwrap();

    let warnings = warnings_for("avert_odin");
    assert!(
        warnings.iter().any(|w| w.contains("preset Odin « nappe » de la piste Chords non chargé")),
        "{warnings:?}"
    );
    assert!(
        composer.missing_resources().contains(&MissingResource::OdinPreset {
            track: TrackId::Chords,
            preset: "nappe".into()
        })
    );
}

#[test]
fn the_same_absence_is_reported_once_not_at_every_tuning_change() {
    let mut composer = engine(None);
    let reel = profile("profiles/reel_quebecois", "avert_unique");
    composer.set_profile(reel.clone()).unwrap();
    composer.set_profile(reel).unwrap();

    assert_eq!(warnings_for("avert_unique").len(), 1);
}

#[test]
fn a_missing_resource_changes_no_note() {
    let notes = |feet: Option<&[u8]>| {
        let mut composer = engine(feet);
        composer.set_seed(1);
        composer.set_profile(profile("profiles/reel_quebecois", "notes_stables")).unwrap();
        composer.set_writehead_lookahead(12);
        composer.generate_bars(10);
        serde_json::to_string(&composer.take_snapshots()).unwrap()
    };
    let without = notes(None);
    assert!(without.contains("\"notes\""));
    assert_eq!(without, notes(Some(FEET_BANK)));
}

#[cfg(not(feature = "odin"))]
#[test]
fn emotion_zones_without_odin_keep_the_soundfont_audio_unchanged() {
    use harmonium::playback::PlaybackCommand;

    let render = |with_zones: bool| {
        let (mut composer, mut playback, mut commands, _, _, _) =
            create_offline_engine(Some(FEET_BANK), None, &[], SAMPLE_RATE).unwrap();
        let mut profile = StyleProfile::named("isaac_burning").unwrap();
        if !with_zones {
            profile.odin.clear();
        }
        composer.set_seed(1);
        composer.set_profile(profile).unwrap();
        composer.use_emotion_mode();
        composer.set_emotions(0.5, 0.3, 0.5, 0.85);
        if let Some((bar, _)) = composer.take_pending_drop() {
            commands.push(PlaybackCommand::DropFrom { bar }).unwrap();
        }
        commands
            .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()))
            .unwrap();
        composer.send_odin(&mut commands).unwrap();
        composer.generate_bars(4);
        let mut audio = vec![0.0; SAMPLE_RATE as usize * 2 * 3];
        playback.process_buffer(&mut audio, 2, std::time::Instant::now());
        audio
    };
    let zoned = render(true);
    assert!(zoned.iter().any(|sample| sample.abs() > 0.001), "le programme SF2 joue réellement");
    assert_eq!(zoned, render(false), "les zones ne changent aucun échantillon sans Odin");
}

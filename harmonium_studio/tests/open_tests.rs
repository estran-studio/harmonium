//! Ouvrir un lien et recharger un profil à chaud : ce que le studio entend après un lien (graine,
//! mesure, boucle, pistes), ce qu'il garde quand un fichier du profil change, et ce qu'il fait
//! d'un fichier invalide. Sans carte son ni fenêtre : hôte détaché, dossier temporaire, et
//! `egui_kittest` pour l'écran.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use eframe::egui::{self, Vec2};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use harmonium::{LiveHost, playback::PlaybackEngine};
use harmonium_core::{EngineCommand, StyleProfile, timeline::TrackId};
use harmonium_studio::{
    link::{OpenRequest, Opening},
    live::{Intent, Session},
    live_screen::{Credits, LiveScreen},
    platform::{RecordingSink, SessionHandle, resolve_open},
    reload::{ProfileWatch, Reloaded},
};
use tempfile::TempDir;
use web_time::Instant;

const SAMPLE_RATE: f64 = 44100.0;

fn repo_profiles() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("profiles")
}

/// Une copie du profil distribué `reel_quebecois` dans un dossier temporaire.
fn profile_copy() -> Result<(TempDir, PathBuf), String> {
    let temp = tempfile::tempdir().map_err(|e| e.to_string())?;
    let dir = fs::canonicalize(temp.path()).map_err(|e| e.to_string())?.join("my_profile");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let source = repo_profiles().join("reel_quebecois");
    for entry in fs::read_dir(&source).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_file() {
            let name = path.file_name().ok_or("nom")?;
            fs::copy(&path, dir.join(name)).map_err(|e| e.to_string())?;
        }
    }
    Ok((temp, dir))
}

fn edit(dir: &Path, file: &str, change: impl FnOnce(String) -> String) -> Result<(), String> {
    let path = dir.join(file);
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    fs::write(&path, change(text)).map_err(|e| e.to_string())
}

/// Change `[meta] name` (la première ligne `name = "…"` de `profile.toml`).
fn rename(dir: &Path, name: &str) -> Result<(), String> {
    edit(dir, "profile.toml", |text| {
        text.lines()
            .scan(false, |done, line| {
                if !*done && line.starts_with("name = \"") {
                    *done = true;
                    return Some(format!("name = \"{name}\""));
                }
                Some(line.to_string())
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    })
}

// === Surveillance des fichiers ===

/// Interroge la surveillance toutes les 200 ms (temps simulé) pendant 1,6 s.
fn settle(watch: &mut ProfileWatch, now: &mut Instant) -> Vec<Reloaded> {
    let mut reloads = Vec::new();
    for _ in 0..8 {
        *now += Duration::from_millis(200);
        reloads.extend(watch.poll(*now));
    }
    reloads
}

fn name_of(reloaded: &Reloaded) -> Result<String, String> {
    reloaded.as_ref().map(|profile| profile.meta.name.clone()).map_err(Clone::clone)
}

#[test]
fn a_changed_profile_file_reloads_once_and_only_after_the_files_settle() -> Result<(), String> {
    let (_temp, dir) = profile_copy()?;
    let mut watch = ProfileWatch::new(&dir);
    let mut now = Instant::now();
    assert_eq!(watch.files().len(), 7, "profile.toml et les six banques de [files]");
    assert!(settle(&mut watch, &mut now).is_empty(), "rien ne change : rien ne se recharge");

    rename(&dir, "Edited")?;
    // Vu à 200 ms, pas encore relu : un éditeur peut écrire en plusieurs temps.
    now += Duration::from_millis(200);
    assert!(watch.poll(now).is_none());
    // Une deuxième écriture dans la fenêtre repousse la lecture.
    rename(&dir, "Edited twice")?;
    now += Duration::from_millis(200);
    assert!(watch.poll(now).is_none());
    now += Duration::from_millis(200);
    assert!(watch.poll(now).is_none(), "seulement 200 ms de repos");

    let reloads = settle(&mut watch, &mut now);
    assert_eq!(reloads.len(), 1, "un seul rechargement pour deux écritures");
    assert_eq!(name_of(&reloads[0])?, "Edited twice");
    assert!(settle(&mut watch, &mut now).is_empty(), "et pas de répétition");
    Ok(())
}

#[test]
fn a_bank_of_the_files_table_is_watched_and_a_stranger_file_is_not() -> Result<(), String> {
    let (_temp, dir) = profile_copy()?;
    let mut watch = ProfileWatch::new(&dir);
    let mut now = Instant::now();

    fs::write(dir.join("notes.txt"), "pas du profil").map_err(|e| e.to_string())?;
    assert!(settle(&mut watch, &mut now).is_empty());

    // Un commentaire dans une banque : le profil se relit (même contenu, même nom).
    edit(&dir, "percussion.toml", |text| format!("# relu\n{text}"))?;
    let reloads = settle(&mut watch, &mut now);
    assert_eq!(reloads.len(), 1);
    assert_eq!(name_of(&reloads[0])?, "Reel québécois");
    Ok(())
}

#[test]
fn an_invalid_file_is_reported_once_with_its_file_and_line_then_a_fix_reloads() -> Result<(), String>
{
    let (_temp, dir) = profile_copy()?;
    let mut watch = ProfileWatch::new(&dir);
    let mut now = Instant::now();

    edit(&dir, "cells.toml", |text| format!("{text}\n[[broken\n"))?;
    let reloads = settle(&mut watch, &mut now);
    assert_eq!(reloads.len(), 1, "l'erreur est rendue une fois");
    let error = reloads[0].as_ref().err().ok_or("le fichier invalide doit être refusé")?;
    let (file, rest) = error.split_once(':').ok_or(error.clone())?;
    assert_eq!(file, "cells.toml", "{error}");
    assert!(rest.starts_with(|c: char| c.is_ascii_digit()), "la ligne suit le fichier : {error}");
    assert!(settle(&mut watch, &mut now).is_empty(), "pas de répétition tant que rien ne change");

    edit(&dir, "cells.toml", |text| text.replace("\n[[broken\n", ""))?;
    let reloads = settle(&mut watch, &mut now);
    assert_eq!(reloads.len(), 1);
    assert_eq!(name_of(&reloads[0])?, "Reel québécois");
    Ok(())
}

#[test]
fn an_id_opens_the_profile_folder_of_the_repository_when_there_is_one() -> Result<(), String> {
    let request = OpenRequest::parse("harmonium://open?profile=reel_quebecois&seed=3")?;
    let opening = resolve_open(request.clone(), &repo_profiles())?;
    let resolved = opening.profile.ok_or("profil")?;
    let watch = resolved.watch.ok_or("le dossier du dépôt est surveillé")?;
    assert!(watch.dir().ends_with("profiles/reel_quebecois"), "{}", watch.dir().display());

    // Sans dossier `profiles/<id>`, c'est le profil compilé, qu'on ne surveille pas.
    let empty = tempfile::tempdir().map_err(|e| e.to_string())?;
    let opening = resolve_open(request, empty.path())?;
    let resolved = opening.profile.ok_or("profil")?;
    assert!(resolved.watch.is_none());
    assert_eq!(resolved.profile.meta.id, "reel_quebecois");

    let error =
        resolve_open(OpenRequest::parse("harmonium://open?profile=no_such_style")?, empty.path())
            .err()
            .ok_or("un identifiant inconnu est refusé")?;
    assert!(error.contains("no_such_style"), "{error}");
    Ok(())
}

#[test]
fn a_folder_with_an_invalid_profile_is_refused_before_anything_changes() -> Result<(), String> {
    let (_temp, dir) = profile_copy()?;
    edit(&dir, "cells.toml", |text| format!("{text}\n[[broken\n"))?;
    let link = format!("harmonium://open?profile={}&seed=3", dir.display());
    let error = resolve_open(OpenRequest::parse(&link)?, &repo_profiles())
        .err()
        .ok_or("un profil invalide est refusé")?;
    assert!(error.contains("cells.toml:"), "{error}");
    Ok(())
}

// === Session : ouverture et rechargement ===

fn detached() -> Result<(Session, PlaybackEngine), String> {
    let (host, playback) = LiveHost::detached(None, None, &[], SAMPLE_RATE)?;
    let mut session = Session::new(host);
    session.perform(Intent::Command(EngineCommand::SetWriteheadLookahead(16)))?;
    Ok((session, playback))
}

fn play(session: &mut Session, playback: &mut PlaybackEngine, buffers: usize) {
    let mut buffer = [0.0f32; 1024];
    for _ in 0..buffers {
        session.pump();
        playback.process_buffer(&mut buffer, 2, Instant::now());
    }
}

fn open(session: &mut Session, link: &str) -> Result<(), String> {
    let Opening { request, profile } = resolve_open(OpenRequest::parse(link)?, &repo_profiles())?;
    session.open(&request, profile.map(|resolved| resolved.profile))
}

type Notes = Vec<(u8, u8, usize, usize, u8)>;

fn notes_of(session: &Session, bar: usize) -> Result<Notes, String> {
    let measure = session.score().bar(bar).ok_or(format!("mesure {bar} non composée"))?;
    Ok(measure
        .notes
        .iter()
        .map(|n| (n.track, n.pitch, n.start_step, n.duration_steps, n.velocity))
        .collect())
}

#[test]
fn a_link_lands_on_the_seed_and_bar_a_linear_session_reaches() -> Result<(), String> {
    // Référence : la graine 3 jouée depuis la mesure 1 jusqu'aux mesures 17 à 20.
    let (mut linear, mut linear_playback) = detached()?;
    let profile = StyleProfile::named("reel_quebecois").map_err(|e| e.to_string())?;
    linear.perform(Intent::Command(EngineCommand::SetStyleProfile(Box::new(profile))))?;
    linear.perform(Intent::Command(EngineCommand::SetSeed(3)))?;
    let mut expected = Vec::new();
    let mut guard = 0;
    while linear.score().last().is_none_or(|last| last < 21) {
        play(&mut linear, &mut linear_playback, 50);
        guard += 1;
        assert!(guard < 200, "la mesure 21 n'arrive pas");
    }
    for bar in 17..=20 {
        expected.push(notes_of(&linear, bar)?);
    }

    // Le lien : la même graine, directement à la mesure 17, bouclée de 17 à 20.
    let (mut session, mut playback) = detached()?;
    open(&mut session, "harmonium://open?profile=reel_quebecois&seed=3&bar=17&loop=17-20")?;
    play(&mut session, &mut playback, 300);
    assert_eq!(session.host().session_seed(), 3);
    let bar = session.host().transport_position().bar;
    assert!((17..=20).contains(&bar), "la tête de lecture reste dans la boucle : mesure {bar}");
    assert_eq!(session.desk().loop_range, Some((17, 20)));
    for (offset, notes) in expected.iter().enumerate() {
        assert_eq!(&notes_of(&session, 17 + offset)?, notes, "mesure {}", 17 + offset);
    }
    assert!(!session.desk().playing, "pas de `play=1` : le studio reste en pause");
    Ok(())
}

#[test]
fn a_link_sets_tracks_and_tempo_and_the_next_link_clears_them() -> Result<(), String> {
    let (mut session, _playback) = detached()?;
    open(
        &mut session,
        "harmonium://open?profile=valse_quebecoise&seed=5&bpm=150&swing=0.6&solo=Bass&mute=Melody&loop=3-4",
    )?;
    let desk = session.desk();
    assert_eq!(desk.solo, Some(TrackId::Bass));
    assert!(desk.muted[usize::from(TrackId::Melody.channel())]);
    assert!(!desk.audible(TrackId::Melody) && desk.audible(TrackId::Bass));
    assert!(!desk.audible(TrackId::Chords), "le solo coupe les autres pistes");
    assert_eq!((desk.imposed_bpm, desk.imposed_swing), (Some(150.0), Some(0.6)));
    let params = session.host().musical_params();
    assert_eq!((params.bpm, params.swing_ratio), (150.0, 0.6));
    assert_eq!(&*params.profile_id, "valse_quebecoise");

    // Un lien est un état entier : sans solo, sans sourdine, sans boucle, tout est rendu ; le
    // profil (non nommé) et le tempo imposé restent.
    open(&mut session, "harmonium://open?seed=6")?;
    let desk = session.desk();
    assert_eq!(desk.solo, None);
    assert!(TrackId::ALL.into_iter().all(|track| desk.audible(track)));
    assert_eq!(desk.loop_range, None);
    assert_eq!(session.host().session_seed(), 6);
    assert_eq!(&*session.host().musical_params().profile_id, "valse_quebecoise");
    assert_eq!(session.host().musical_params().bpm, 150.0);

    // Nommer le profil rend le tempo du profil (le moteur remet tempo et swing).
    open(&mut session, "harmonium://open?profile=valse_quebecoise")?;
    assert_eq!(session.desk().imposed_bpm, None);
    assert_ne!(session.host().musical_params().bpm, 150.0);
    Ok(())
}

#[test]
fn a_link_the_host_refuses_says_why_and_leaves_no_loop() -> Result<(), String> {
    let (mut session, _playback) = detached()?;
    open(&mut session, "harmonium://open?profile=reel_quebecois&seed=3")?;
    // Boucle plus longue que la fenêtre de lecture : refusée par l'hôte, après le profil et la graine.
    let error = open(&mut session, "harmonium://open?seed=9&loop=1-60").err().ok_or("refusé")?;
    assert!(error.contains("Loop too long"), "{error}");
    assert_eq!(session.desk().loop_range, None);
    Ok(())
}

#[test]
fn reloading_a_profile_keeps_what_the_engine_resets_with_a_profile() -> Result<(), String> {
    let (mut session, _playback) = detached()?;
    open(
        &mut session,
        "harmonium://open?profile=reel_quebecois&seed=5&bar=4&bpm=200&swing=0.6&solo=Chords&mute=Melody&loop=4-8",
    )?;
    session.perform(Intent::Command(EngineCommand::SetChannelGain {
        channel: TrackId::Bass.channel(),
        gain: 0.3,
    }))?;
    let bpm_of_profile =
        StyleProfile::named("reel_quebecois").map_err(|e| e.to_string())?.meter.default_bpm;

    let mut edited = StyleProfile::named("reel_quebecois").map_err(|e| e.to_string())?;
    edited.meta.name = "Edited".to_string();

    // Sans le rechargement dédié, installer le profil remet le tempo et le swing du profil.
    let mut control = StyleProfile::named("reel_quebecois").map_err(|e| e.to_string())?;
    control.meta.name = "Control".to_string();
    session.perform(Intent::Command(EngineCommand::SetStyleProfile(Box::new(control))))?;
    assert_eq!(session.host().musical_params().bpm, bpm_of_profile, "le moteur remet le tempo");

    open(
        &mut session,
        "harmonium://open?seed=5&bar=4&bpm=200&swing=0.6&solo=Chords&mute=Melody&loop=4-8",
    )?;
    session.reload_profile(edited)?;
    assert_eq!(session.host().profile().meta.name, "Edited");
    assert_eq!(session.host().session_seed(), 5);
    let params = session.host().musical_params();
    assert_eq!((params.bpm, params.swing_ratio), (200.0, 0.6));
    let desk = session.desk();
    assert_eq!(desk.loop_range, Some((4, 8)));
    assert_eq!(desk.solo, Some(TrackId::Chords));
    assert!(desk.muted[usize::from(TrackId::Melody.channel())]);
    assert_eq!((desk.imposed_bpm, desk.imposed_swing), (Some(200.0), Some(0.6)));
    Ok(())
}

// === Écran : rechargement à chaud, lien copié ===

struct Rig {
    harness: Harness<'static, LiveScreen>,
    session: SessionHandle,
    playback: PlaybackEngine,
    _recordings: TempDir,
}

impl Rig {
    fn new() -> Result<Self, String> {
        let (host, playback) = LiveHost::detached(None, None, &[], SAMPLE_RATE)?;
        let mut session = Session::new(host);
        session.perform(Intent::Command(EngineCommand::SetWriteheadLookahead(16)))?;
        let session = SessionHandle::new(session);
        let recordings = tempfile::tempdir().map_err(|e| e.to_string())?;
        let screen = LiveScreen::new(
            session.clone(),
            RecordingSink::new(recordings.path().to_path_buf()),
            Credits::default(),
        );
        let harness = Harness::builder()
            .with_size(Vec2::new(1440.0, 900.0))
            .build_ui_state(|ui: &mut egui::Ui, screen: &mut LiveScreen| screen.show(ui), screen);
        Ok(Self { harness, session, playback, _recordings: recordings })
    }

    fn open(&mut self, link: &str, profiles: &Path) -> Result<(), String> {
        let opening = resolve_open(OpenRequest::parse(link)?, profiles)?;
        self.harness.state_mut().open(opening);
        self.harness.step();
        Ok(())
    }

    /// Fait tourner l'écran et le moteur (temps réel : la surveillance lit l'horloge) jusqu'à ce
    /// qu'un libellé contenant `text` apparaisse.
    fn wait_for(&mut self, text: &str) -> Result<(), String> {
        let mut buffer = [0.0f32; 1024];
        for _ in 0..100 {
            self.session.with(Session::pump);
            self.playback.process_buffer(&mut buffer, 2, Instant::now());
            self.harness.step();
            if self.has_label(text) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Err(format!("`{text}` n'apparaît pas"))
    }

    fn has_label(&self, text: &str) -> bool {
        self.harness.query_all_by_label_contains(text).next().is_some()
    }

    fn profile_name(&self) -> String {
        self.session.with(|session| session.host().profile().meta.name.clone())
    }
}

#[test]
fn the_screen_reloads_an_edited_profile_and_keeps_the_seed_bar_and_tempo() -> Result<(), String> {
    let (_temp, dir) = profile_copy()?;
    let mut rig = Rig::new()?;
    rig.open(
        &format!("harmonium://open?profile={}&seed=9&bar=5&bpm=190&solo=Bass", dir.display()),
        &repo_profiles(),
    )?;
    rig.wait_for("Watching my_profile")?;
    assert_eq!(rig.profile_name(), "Reel québécois");

    rename(&dir, "Edited live")?;
    rig.wait_for("reloaded")?;
    assert_eq!(rig.profile_name(), "Edited live");
    rig.session.with(|session| {
        assert_eq!(session.host().session_seed(), 9);
        assert_eq!(session.host().transport_position().bar, 5);
        assert_eq!(session.host().musical_params().bpm, 190.0);
        assert_eq!(session.desk().solo, Some(TrackId::Bass));
    });
    Ok(())
}

#[test]
fn the_screen_keeps_the_previous_profile_when_the_new_one_is_invalid() -> Result<(), String> {
    let (_temp, dir) = profile_copy()?;
    let mut rig = Rig::new()?;
    rig.open(&format!("harmonium://open?profile={}&seed=9", dir.display()), &repo_profiles())?;
    rename(&dir, "Good edit")?;
    rig.wait_for("reloaded")?;
    assert_eq!(rig.profile_name(), "Good edit");

    edit(&dir, "cells.toml", |text| format!("{text}\n[[broken\n"))?;
    rig.wait_for("reload failed")?;
    assert!(rig.has_label("cells.toml:"), "fichier et ligne à l'écran");
    assert_eq!(rig.profile_name(), "Good edit", "le profil précédent joue toujours");
    assert_eq!(rig.session.with(|session| session.host().session_seed()), 9);

    edit(&dir, "cells.toml", |text| text.replace("\n[[broken\n", ""))?;
    rename(&dir, "Fixed")?;
    rig.wait_for("reloaded")?;
    assert_eq!(rig.profile_name(), "Fixed");
    assert!(!rig.has_label("reload failed"), "l'erreur est levée de la barre du haut");
    Ok(())
}

#[test]
fn copy_link_gives_the_link_that_reopens_what_is_heard() -> Result<(), String> {
    let (_temp, dir) = profile_copy()?;
    let mut rig = Rig::new()?;
    rig.open(
        &format!(
            "harmonium://open?profile={}&seed=9&bar=5&loop=5-8&solo=Bass&mute=Melody&bpm=190",
            dir.display()
        ),
        &repo_profiles(),
    )?;
    // La tête de lecture doit avoir rejoint la mesure 5 (le saut passe par le fil audio).
    let mut buffer = [0.0f32; 1024];
    for _ in 0..4 {
        rig.session.with(Session::pump);
        rig.playback.process_buffer(&mut buffer, 2, Instant::now());
    }
    // Un cadre de plus : la barre du haut a changé de mise en page avec l'étiquette du dossier.
    rig.harness.step();
    rig.harness.get_by_label("Copy link").click();
    rig.harness.step();
    let notice = rig
        .harness
        .get_by_label_contains("Copied: harmonium://open?")
        .accesskit_node()
        .value()
        .ok_or("avis sans texte")?;
    let link = notice.trim_start_matches("Copied: ");
    let copied = OpenRequest::parse(link)?;
    let expected = OpenRequest::parse(&format!(
        "harmonium://open?profile={}&seed=9&bar=5&loop=5-8&bpm=190&solo=Bass&mute=Melody",
        dir.display()
    ))?;
    assert_eq!(copied, expected, "{link}");
    Ok(())
}

#[test]
fn copy_link_of_a_distributed_style_names_it_by_id_and_refuses_a_draft() -> Result<(), String> {
    let mut rig = Rig::new()?;
    rig.open("harmonium://open?profile=valse_quebecoise&seed=2", Path::new("/nonexistent"))?;
    rig.harness.get_by_label("Copy link").click();
    rig.harness.step();
    let notice = rig
        .harness
        .get_by_label_contains("Copied: harmonium://open?")
        .accesskit_node()
        .value()
        .ok_or("avis sans texte")?;
    assert!(notice.contains("profile=valse_quebecoise&seed=2"), "{notice}");

    // Un brouillon (ni distribué, ni chargé d'un dossier) n'a pas de lien : le dire.
    let mut draft = StyleProfile::named("valse_quebecoise").map_err(|e| e.to_string())?;
    draft.meta.id = "my_draft".to_string();
    rig.session.with(|session| {
        session.perform(Intent::Command(EngineCommand::SetStyleProfile(Box::new(draft))))
    })?;
    rig.harness.step();
    rig.harness.get_by_label("Copy link").click();
    rig.harness.step();
    assert!(
        rig.harness
            .query_by_label_contains("neither a distributed one nor loaded from a folder")
            .is_some()
    );
    Ok(())
}

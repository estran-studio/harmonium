//! L'écran Live sans fenêtre (`egui_kittest`) : le test agit sur les vrais widgets, par l'arbre
//! d'accessibilité ou le pointeur, sur une session détachée (aucune carte son), et regarde l'effet
//! dans la sortie publiée du moteur, dans les fichiers écrits et à l'écran.

use eframe::egui::{
    self, Event, Modifiers, PointerButton, Pos2, Vec2,
    accesskit::{Action, ActionData, ActionRequest, Role},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use harmonium::{LiveHost, playback::PlaybackEngine};
use harmonium_core::{EngineCommand, StyleProfile, timeline::TrackId};
use harmonium_studio::{
    live::{Intent, Session, TRACKS},
    live_screen::{Credits, LiveScreen},
    platform::{RecordingSink, SessionHandle},
};
use tempfile::TempDir;
use web_time::Instant;

const SEED: u64 = 3;

struct Rig {
    harness: Harness<'static, LiveScreen>,
    session: SessionHandle,
    playback: PlaybackEngine,
    recordings: TempDir,
}

impl Rig {
    fn new() -> Result<Self, String> {
        let (host, playback) = LiveHost::detached(None, None, &[], 44100.0)?;
        let mut session = Session::new(host);
        let profile = StyleProfile::named("reel_quebecois").map_err(|e| e.to_string())?;
        for command in [
            EngineCommand::SetWriteheadLookahead(16),
            EngineCommand::SetStyleProfile(Box::new(profile)),
            EngineCommand::SetSeed(SEED),
        ] {
            session.perform(Intent::Command(command))?;
        }
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
        Ok(Self { harness, session, playback, recordings })
    }

    /// Compose et joue `buffers` tampons, puis dessine une image.
    fn play(&mut self, buffers: usize) {
        let mut buffer = [0.0f32; 1024];
        for _ in 0..buffers {
            self.session.with(Session::pump);
            self.playback.process_buffer(&mut buffer, 2, Instant::now());
        }
        self.harness.step();
    }

    fn transport_bar(&self) -> usize {
        self.session.with(|session| session.host().transport_position().bar)
    }

    fn press(&mut self, pos: Pos2, pressed: bool) {
        self.harness.event(Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        self.harness.step();
    }

    fn move_to(&mut self, pos: Pos2) {
        self.harness.event(Event::PointerMoved(pos));
        self.harness.step();
    }

    /// Glisse le pointeur de la mesure `from` à la mesure `to`.
    fn drag_bars(&mut self, from: usize, to: usize) {
        let start = self.harness.get_by_label(&format!("Bar {from}")).rect().center();
        let end = self.harness.get_by_label(&format!("Bar {to}")).rect().center();
        self.move_to(start);
        self.press(start, true);
        for step in 1..=8 {
            self.move_to(start + (end - start) * (step as f32 / 8.0));
        }
        self.press(end, false);
        // L'image suivante dessine l'effet du geste.
        self.harness.step();
    }

    fn first_bar(&self) -> usize {
        self.session.with(|session| session.view(Instant::now()).score.first_bar)
    }
}

#[test]
fn moving_the_arousal_slider_recomposes_the_bars_ahead() -> Result<(), String> {
    let mut rig = Rig::new()?;
    rig.session.with(|session| {
        session.perform(Intent::Command(EngineCommand::SetEmotionParams {
            arousal: 0.1,
            valence: -0.6,
            density: 0.2,
            tension: 0.1,
        }))
    })?;
    rig.play(200);
    let ahead = rig.transport_bar() + 3;
    let notes = |rig: &Rig| {
        rig.session.with(|s| {
            s.score().bar(ahead).map(|m| {
                m.notes
                    .iter()
                    .map(|n| (n.track, n.pitch, n.start_step, n.velocity))
                    .collect::<Vec<_>>()
            })
        })
    };
    let before = notes(&rig);

    let (target_node, target_tree) =
        rig.harness.get_by_role_and_label(Role::Slider, "Arousal").accesskit_node().locate();
    rig.harness.event(Event::AccessKitActionRequest(ActionRequest {
        target_node,
        target_tree,
        action: Action::SetValue,
        data: Some(ActionData::NumericValue(0.95)),
    }));
    rig.harness.step();

    let emotions = rig.session.with(|session| session.host().emotions());
    assert!((emotions[0] - 0.95).abs() < 1e-6, "arousal {emotions:?}");
    assert_eq!(emotions[1..], [-0.6, 0.2, 0.1], "les autres axes restent");

    // Publié au `pump` suivant : la mesure d'avance est recomposée.
    rig.play(1);
    let after = notes(&rig);
    assert!(before.is_some() && after.is_some() && before != after, "{before:?} → {after:?}");
    Ok(())
}

#[test]
fn clicking_a_bar_seeks_there() -> Result<(), String> {
    let mut rig = Rig::new()?;
    rig.play(100);
    let target = rig.first_bar() + 9;
    let center = rig.harness.get_by_label(&format!("Bar {target}")).rect().center();
    rig.move_to(center);
    rig.press(center, true);
    rig.press(center, false);
    rig.play(2);
    assert_eq!(rig.transport_bar(), target);
    assert!(rig.harness.query_by_label(&format!("Bar {target}")).is_some());
    Ok(())
}

#[test]
fn dragging_across_bars_sets_a_loop_and_a_long_drag_is_refused_with_a_message() -> Result<(), String>
{
    let mut rig = Rig::new()?;
    rig.play(20);
    let first = rig.first_bar();
    // De droite à gauche.
    rig.drag_bars(first + 5, first + 2);
    let range = rig.session.with(|session| session.desk().loop_range);
    assert_eq!(range, Some((first + 2, first + 5)));
    assert!(rig.harness.query_by_label_contains("Loop: bars").is_some());

    let first = rig.first_bar();
    rig.drag_bars(first, first + 19);
    assert!(rig.harness.query_by_label_contains("Loop too long").is_some(), "the refusal is shown");
    let unchanged = rig.session.with(|session| session.desk().loop_range);
    assert_eq!(unchanged, range);

    rig.harness.get_by_label("Clear loop").click();
    rig.harness.step();
    assert_eq!(rig.session.with(|session| session.desk().loop_range), None);
    Ok(())
}

#[test]
fn the_solo_button_mutes_every_other_track() -> Result<(), String> {
    let mut rig = Rig::new()?;
    rig.play(10);
    let bass = TRACKS.iter().position(|track| *track == TrackId::Bass).ok_or("Bass")?;
    rig.harness.get_all_by_label("Solo").nth(bass).ok_or("Solo button")?.click();
    rig.harness.step();
    rig.harness.step();
    let mixer = rig.session.with(|session| session.view(Instant::now()).mixer);
    for strip in &mixer {
        assert_eq!(strip.audible, strip.track == TrackId::Bass, "{:?}", strip.track);
    }
    rig.harness.get_all_by_label("Solo").nth(bass).ok_or("Solo button")?.click();
    rig.harness.step();
    rig.harness.step();
    let mixer = rig.session.with(|session| session.view(Instant::now()).mixer);
    assert!(mixer.iter().all(|strip| strip.audible));
    Ok(())
}

#[test]
fn the_record_toggle_writes_a_file_and_shows_where() -> Result<(), String> {
    let mut rig = Rig::new()?;
    rig.harness.get_by_label("WAV").click();
    rig.harness.step();
    rig.play(200);
    rig.harness.get_by_label("WAV").click();
    rig.harness.step();
    rig.play(2);
    rig.harness.step();

    let path = rig.recordings.path().join(format!("harmonium-reel_quebecois-seed{SEED}.wav"));
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    assert!(bytes.starts_with(b"RIFF"), "a WAV file");
    assert!(rig.harness.query_by_label_contains("Saved").is_some(), "the path is shown");
    Ok(())
}

#[test]
fn the_beat_clock_and_the_state_panel_follow_the_playhead() -> Result<(), String> {
    let mut rig = Rig::new()?;
    rig.play(300);
    let bar = rig.transport_bar();
    assert!(bar > 1);
    assert!(rig.harness.query_by_label_contains(" · pulse ").is_some(), "beat clock");
    assert!(rig.harness.query_by_label("Tune key/mode").is_some(), "state panel");
    assert!(rig.harness.query_by_label(&format!("Bar {bar}")).is_some(), "current bar shown");
    Ok(())
}

fn profile_id(rig: &Rig) -> String {
    rig.session.with(|session| session.host().musical_params().profile_id.to_string())
}

fn seed(rig: &Rig) -> u64 {
    rig.session.with(|session| session.host().session_seed())
}

#[test]
fn the_style_selector_the_profile_folder_and_the_seed_fields_drive_the_session()
-> Result<(), String> {
    let mut rig = Rig::new()?;
    rig.play(10);

    // Sélecteur de style : ouvrir, choisir.
    rig.harness.get_all_by_role(Role::ComboBox).next().ok_or("style selector")?.click();
    rig.harness.step();
    rig.harness.get_by_label("Valse québécoise").click();
    rig.harness.step();
    rig.harness.step();
    assert_eq!(profile_id(&rig), "valse_quebecoise");

    // Graine : saisie, puis nouvelle graine.
    let seed_field = rig.harness.get_all_by_role(Role::TextInput).next().ok_or("seed field")?;
    seed_field.click();
    rig.harness.step();
    rig.harness.get_all_by_role(Role::TextInput).next().ok_or("seed field")?.type_text("42");
    rig.harness.step();
    rig.harness.get_by_label("Set seed").click();
    rig.harness.step();
    assert_eq!(seed(&rig), 42);
    rig.harness.get_by_label("New seed").click();
    rig.harness.step();
    assert_ne!(seed(&rig), 42);

    // Natif : un dossier de profil sur disque, comme `style <dossier>`.
    rig.harness.get_by_label("Engine").click();
    rig.harness.step();
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/../profiles/gigue_acadienne");
    let field = rig.harness.get_all_by_role(Role::TextInput).nth(1).ok_or("folder field")?;
    field.click();
    rig.harness.step();
    rig.harness.get_all_by_role(Role::TextInput).nth(1).ok_or("folder field")?.type_text(folder);
    rig.harness.step();
    rig.harness.get_by_label("Load").click();
    rig.harness.step();
    assert_eq!(profile_id(&rig), "gigue_acadienne");
    Ok(())
}

#[test]
fn the_about_panel_lists_the_licences_and_credits() -> Result<(), String> {
    let mut rig = Rig::new()?;
    rig.play(2);
    assert!(rig.harness.query_by_label_contains("GPL-3.0").is_none(), "closed at first");
    rig.harness.get_by_label("About").click();
    rig.harness.step();
    rig.harness.step();
    for text in [
        "MIT OR Apache-2.0",
        "odin-rs",
        "GPL-3.0",
        "Complete source",
        "GeneralUser GS",
        "S. Christian Collins",
        "Odin presets: none loaded",
    ] {
        assert!(
            rig.harness.query_all_by_label_contains(text).next().is_some(),
            "{text} in the About panel"
        );
    }
    Ok(())
}

// === Atelier ===

/// Clique le bouton `label`, puis dessine deux images : l'atelier dessine sa colonne de gauche
/// avant le centre, donc un geste du centre (Fork) n'y paraît qu'à l'image suivante.
fn click(rig: &mut Rig, label: &str) {
    rig.harness.get_by_label(label).click();
    rig.harness.step();
    rig.harness.step();
}

/// Le texte `text` tapé dans le champ « Profile folder » de l'atelier, à la place du sien.
fn type_folder(rig: &mut Rig, text: &str) {
    rig.harness.get_by_label("Profile folder").click();
    rig.harness.step();
    rig.harness.key_press_modifiers(Modifiers::COMMAND, egui::Key::A);
    rig.harness.get_by_label("Profile folder").type_text(text);
    rig.harness.step();
}

/// Un texte de l'écran (une étiquette porte son texte en valeur) qui commence par `prefix`.
fn shown_text(rig: &Rig, prefix: &str) -> Option<String> {
    rig.harness
        .query_all_by(|node| node.value().is_some_and(|value| value.starts_with(prefix)))
        .next()
        .and_then(|node| node.accesskit_node().value())
}

fn bar_tempo(rig: &Rig, bar: usize) -> Option<f32> {
    rig.session.with(|session| session.score().bar(bar).map(|measure| measure.tempo))
}

#[test]
fn a_forked_profile_saved_edited_and_applied_plays_from_the_next_bar() -> Result<(), String> {
    let mut rig = Rig::new()?;
    rig.play(200);
    click(&mut rig, "Workbench");

    // Le premier profil distribué est montré, en lecture seule : Fork en fait le brouillon.
    click(&mut rig, "Fork");
    assert!(shown_text(&rig, "Valid profile").is_some());

    // Enregistré dans un dossier, modifié sur disque, rouvert.
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let folder = dir.path().join("reel_fork");
    type_folder(&mut rig, folder.to_str().ok_or("chemin")?);
    click(&mut rig, "Save");
    let root = folder.join("profile.toml");
    let text = std::fs::read_to_string(&root).map_err(|e| e.to_string())?;
    assert!(text.contains("id = \"reel_quebecois_fork\""));
    let edited = text.replace("default_bpm = 230.0", "default_bpm = 225.0");
    assert_ne!(edited, text);
    std::fs::write(&root, edited).map_err(|e| e.to_string())?;
    click(&mut rig, "Open");

    // Apply : la mesure qui sonne finit à 230, la suivante est recomposée à 225.
    let sounding = rig.transport_bar();
    click(&mut rig, "Apply");
    rig.session.with(Session::pump);
    assert_eq!(profile_id(&rig), "reel_quebecois_fork");
    assert_eq!(bar_tempo(&rig, sounding), Some(230.0));
    assert_eq!(bar_tempo(&rig, sounding + 1), Some(225.0));
    // L'écran nomme le profil installé, même hors des profils distribués.
    click(&mut rig, "Live");
    assert!(shown_text(&rig, "Reel québécois (fork)").is_some());

    // Le dossier enregistré se lit comme `harmonium-cli --profile <dossier>`.
    let loaded = StyleProfile::load(&folder).map_err(|e| e.to_string())?;
    assert_eq!(loaded, rig.session.with(|session| session.host().profile().clone()));
    Ok(())
}

#[test]
fn an_error_in_a_bank_is_placed_on_its_line_and_blocks_apply() -> Result<(), String> {
    let mut rig = Rig::new()?;
    rig.play(10);
    click(&mut rig, "Workbench");
    click(&mut rig, "Fork");
    click(&mut rig, "cells.toml");

    // Une ligne invalide, où que soit le curseur.
    rig.harness.get_by_role(Role::MultilineTextInput).focus();
    rig.harness.step();
    rig.harness.get_by_role(Role::MultilineTextInput).type_text("=oops");
    rig.harness.step();
    rig.harness.step();

    let text = rig.harness.get_by_role(Role::MultilineTextInput).value().ok_or("texte")?;
    let line = text.lines().position(|l| l.contains("=oops")).ok_or("ligne tapée")? + 1;
    let message = shown_text(&rig, "cells.toml:").ok_or("message d'erreur")?;
    assert!(message.starts_with(&format!("cells.toml:{line}: ")), "{message}");
    assert!(
        rig.harness.query_by_label("cells.toml (error)").is_some(),
        "l'onglet fautif est marqué"
    );
    assert!(shown_text(&rig, "Valid profile").is_none());
    assert!(rig.harness.get_by_label("Apply").accesskit_node().is_disabled());
    click(&mut rig, "Apply");
    assert_eq!(profile_id(&rig), "reel_quebecois", "rien d'installé");
    Ok(())
}

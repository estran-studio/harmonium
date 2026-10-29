//! L'atelier de profils dessiné par egui (onglet « Workbench »).
//!
//! À gauche : le brouillon (état du décodage, Apply, dossier ou archive) et les profils
//! distribués, lisibles. Au centre : un onglet par fichier du profil montré, l'éditeur TOML coloré
//! avec ses numéros de ligne, et l'erreur du décodage placée sur sa ligne.
//!
//! Le brouillon se décode à chaque modification (`Draft::decode`, le décodeur du moteur) ; Apply
//! l'installe par `EngineCommand::SetStyleProfile`, comme le sélecteur de style : il s'entend à la
//! mesure suivante. Un profil distribué ne se modifie pas : « Fork » en fait le brouillon.

use eframe::egui::{
    self, Align, Color32, FontId, Rect, RichText, TextFormat,
    cache::{ComputerMut, FrameCache},
    text::LayoutJob,
};
use harmonium_core::{EngineCommand, StyleProfile, style_profile::ProfileError};

use crate::{
    live::Intent,
    workbench::{Draft, Origin, TomlToken, toml_spans},
};

/// Largeur de la gouttière des numéros de ligne, en points (marge gauche de l'éditeur).
const GUTTER: i8 = 44;

/// Taille du texte de l'éditeur.
const CODE_SIZE: f32 = 13.0;

/// Ce que montre l'atelier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Shown {
    Draft,
    /// Un profil distribué, par son rang dans `StyleProfile::distributed()`.
    Distributed(usize),
}

/// Un avis pour la barre du bas : texte, erreur.
pub type WorkbenchNotice = (String, bool);

/// L'atelier.
pub struct WorkbenchScreen {
    draft: Option<Draft>,
    shown: Shown,
    /// Fichier montré (rang dans les fichiers du profil montré).
    tab: usize,
    /// Décodage du brouillon depuis sa dernière modification : le nom du profil, ou l'erreur.
    check: Option<Result<String, ProfileError>>,
    /// Le brouillon a changé depuis le dernier Apply.
    changed_since_apply: bool,
    /// Ligne à amener à l'écran à la prochaine image (l'erreur).
    scroll_to_line: Option<usize>,
    notices: Vec<WorkbenchNotice>,
    /// Dossier à ouvrir ou dans lequel enregistrer (natif ; champ de saisie).
    #[cfg(not(target_arch = "wasm32"))]
    folder: String,
    /// Choix de fichiers en cours (web).
    #[cfg(target_arch = "wasm32")]
    upload: Option<crate::platform::Upload>,
    /// Le navigateur a déjà refusé de garder le brouillon : l'avis n'est donné qu'une fois.
    #[cfg(target_arch = "wasm32")]
    store_failed: bool,
}

impl Default for WorkbenchScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkbenchScreen {
    /// L'atelier, avec le brouillon que le navigateur a gardé (web).
    #[must_use]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut screen = Self {
            draft: None,
            shown: Shown::Distributed(0),
            tab: 0,
            check: None,
            changed_since_apply: true,
            scroll_to_line: None,
            notices: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            folder: String::new(),
            #[cfg(target_arch = "wasm32")]
            upload: None,
            #[cfg(target_arch = "wasm32")]
            store_failed: false,
        };
        #[cfg(target_arch = "wasm32")]
        match crate::platform::load_draft() {
            Ok(Some(draft)) => {
                screen.draft = Some(draft);
                screen.shown = Shown::Draft;
                screen.recheck();
            }
            Ok(None) => {}
            Err(error) => {
                screen.notices.push((format!("Could not restore the saved draft: {error}"), true));
            }
        }
        screen
    }

    /// Le brouillon, s'il y en a un.
    #[must_use]
    pub fn draft(&self) -> Option<&Draft> {
        self.draft.as_ref()
    }

    /// Dessine l'atelier dans `ui` ; Apply pousse sa commande dans `intents`. Rend les avis.
    pub fn show(&mut self, ui: &mut egui::Ui, intents: &mut Vec<Intent>) -> Vec<WorkbenchNotice> {
        #[cfg(target_arch = "wasm32")]
        self.poll_upload();
        egui::Panel::left("workbench_profiles").resizable(false).exact_size(290.0).show_inside(
            ui,
            |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.sidebar(ui, intents));
            },
        );
        egui::CentralPanel::default().show_inside(ui, |ui| self.files(ui));
        std::mem::take(&mut self.notices)
    }

    fn notify(&mut self, text: impl Into<String>, error: bool) {
        self.notices.push((text.into(), error));
    }

    fn set_draft(&mut self, draft: Draft) {
        self.draft = Some(draft);
        self.shown = Shown::Draft;
        self.tab = 0;
        self.changed_since_apply = true;
        self.recheck();
        self.persist();
    }

    fn recheck(&mut self) {
        self.check = self.draft.as_ref().map(|draft| draft.decode().map(|p| p.meta.name));
    }

    /// Web : le navigateur garde le brouillon (`localStorage`). Natif : le brouillon vit dans les
    /// dossiers que l'on enregistre.
    fn persist(&mut self) {
        #[cfg(target_arch = "wasm32")]
        if let Err(error) = crate::platform::store_draft(self.draft.as_ref()) {
            if !self.store_failed {
                self.store_failed = true;
                self.notify(format!("This browser did not keep the draft: {error}"), true);
            }
        } else {
            self.store_failed = false;
        }
    }

    /// Montre le fichier et la ligne de l'erreur du brouillon.
    fn go_to_error(&mut self) {
        let (Some(draft), Some(Err(error))) = (&self.draft, &self.check) else { return };
        let Some(file) = &error.file else { return };
        if let Some(index) = draft.files.iter().position(|f| &f.name == file) {
            self.shown = Shown::Draft;
            self.tab = index;
            self.scroll_to_line = error.line;
        }
    }

    fn apply(&mut self, intents: &mut Vec<Intent>) {
        let Some(draft) = &self.draft else { return };
        match draft.decode() {
            Ok(profile) => {
                let name = profile.meta.name.clone();
                intents.push(Intent::Command(EngineCommand::SetStyleProfile(Box::new(profile))));
                self.changed_since_apply = false;
                self.notify(format!("Applied “{name}”: it plays from the next bar."), false);
            }
            Err(error) => {
                self.check = Some(Err(error));
                self.go_to_error();
            }
        }
    }

    // === Gauche : brouillon et profils distribués ===

    fn sidebar(&mut self, ui: &mut egui::Ui, intents: &mut Vec<Intent>) {
        ui.heading("Draft");
        if let Some(draft) = &self.draft {
            let title = match &self.check {
                Some(Ok(name)) => name.clone(),
                _ => draft.folder_name(),
            };
            if ui.add(egui::Button::selectable(self.shown == Shown::Draft, title)).clicked()
                && self.shown != Shown::Draft
            {
                self.shown = Shown::Draft;
                self.tab = 0;
            }
            ui.weak(match &draft.origin {
                Origin::Fork(id) => format!("Fork of {id}"),
                Origin::Folder(path) => format!("From {path}"),
                Origin::Upload => "Uploaded".to_string(),
            });
            match &self.check {
                Some(Ok(_)) => {
                    ui.colored_label(Color32::from_rgb(150, 200, 150), "Valid profile");
                }
                Some(Err(_)) => {
                    ui.colored_label(Color32::LIGHT_RED, "Error: see the message above the file");
                }
                None => {}
            }
            let valid = matches!(self.check, Some(Ok(_)));
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(valid, egui::Button::new("Apply"))
                    .on_hover_text("Install this profile: it plays from the next bar.")
                    .clicked()
                {
                    self.apply(intents);
                }
                if self.changed_since_apply {
                    ui.weak("not applied");
                } else {
                    ui.weak("applied");
                }
            });
        } else {
            ui.weak("No draft yet. Fork a distributed profile below, or open your own.");
        }
        ui.add_space(6.0);
        self.storage_controls(ui);

        ui.separator();
        ui.heading("Distributed profiles");
        ui.weak("Read-only. Fork one to edit it.");
        for (index, profile) in StyleProfile::distributed().iter().enumerate() {
            let shown = self.shown == Shown::Distributed(index);
            if ui.add(egui::Button::selectable(shown, profile.meta.name.as_str())).clicked()
                && !shown
            {
                self.shown = Shown::Distributed(index);
                self.tab = 0;
            }
        }
    }

    /// Natif : ouvrir et enregistrer un dossier.
    #[cfg(not(target_arch = "wasm32"))]
    fn storage_controls(&mut self, ui: &mut egui::Ui) {
        use std::path::Path;

        let label = ui.label("Profile folder");
        ui.add(
            egui::TextEdit::singleline(&mut self.folder)
                .desired_width(260.0)
                .hint_text("path to a profile folder"),
        )
        .labelled_by(label.id);
        ui.horizontal(|ui| {
            if ui.button("Open").on_hover_text("Edit the profile of this folder.").clicked() {
                match crate::platform::read_profile_folder(Path::new(self.folder.trim())) {
                    Ok(draft) => {
                        if let Origin::Folder(dir) = &draft.origin {
                            self.folder.clone_from(dir);
                        }
                        let text = format!("Opened {}", self.folder);
                        self.set_draft(draft);
                        self.notify(text, false);
                    }
                    Err(error) => self.notify(error, true),
                }
            }
            let save = ui
                .add_enabled(self.draft.is_some(), egui::Button::new("Save"))
                .on_hover_text("Write the draft's files into this folder (created if needed).");
            if save.clicked()
                && let Some(draft) = &self.draft
            {
                let dir = self.folder.trim();
                if dir.is_empty() {
                    self.notify("Choose a folder to save into.", true);
                } else {
                    match crate::platform::write_profile_folder(Path::new(dir), draft) {
                        Ok(count) => self.notify(format!("Saved {count} files to {dir}"), false),
                        Err(error) => self.notify(error, true),
                    }
                }
            }
        });
    }

    /// Web : téléverser un dossier ou une archive, télécharger le dossier en archive.
    #[cfg(target_arch = "wasm32")]
    fn storage_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .button("Upload…")
                .on_hover_text("A .zip of a profile folder, or all the files of the folder.")
                .clicked()
            {
                match crate::platform::pick_profile_files() {
                    Ok(upload) => self.upload = Some(upload),
                    Err(error) => self.notify(error, true),
                }
            }
            let download = ui
                .add_enabled(self.draft.is_some(), egui::Button::new("Download .zip"))
                .on_hover_text(
                    "The profile folder, to unzip and load with harmonium-cli --profile.",
                );
            if download.clicked()
                && let Some(draft) = &self.draft
            {
                let name = format!("{}.zip", draft.folder_name());
                let result = draft.to_zip().and_then(|bytes| {
                    crate::platform::offer_download(&name, "application/zip", &bytes)
                });
                match result {
                    Ok(()) => self.notify(format!("Downloaded {name}"), false),
                    Err(error) => self.notify(error, true),
                }
            }
        });
        ui.weak("The draft stays in this browser.");
    }

    #[cfg(target_arch = "wasm32")]
    fn poll_upload(&mut self) {
        let Some(picked) = self.upload.as_ref().and_then(crate::platform::Upload::take) else {
            return;
        };
        self.upload = None;
        let draft = picked
            .and_then(crate::workbench::files_from_upload)
            .and_then(|files| Draft::new(Origin::Upload, files));
        match draft {
            Ok(draft) => {
                let text = format!("Uploaded {}", draft.folder_name());
                self.set_draft(draft);
                self.notify(text, false);
            }
            Err(error) => self.notify(error, true),
        }
    }

    // === Centre : fichiers du profil montré ===

    fn files(&mut self, ui: &mut egui::Ui) {
        let names: Vec<String> = match self.shown {
            Shown::Draft => self
                .draft
                .as_ref()
                .map(|draft| draft.files.iter().map(|f| f.name.clone()).collect())
                .unwrap_or_default(),
            Shown::Distributed(index) => StyleProfile::distributed_files()
                .get(index)
                .map(|files| files.iter().map(|(name, _)| (*name).to_string()).collect())
                .unwrap_or_default(),
        };
        if names.is_empty() {
            ui.weak("Nothing to show.");
            return;
        }
        self.tab = self.tab.min(names.len() - 1);
        let error = match (&self.shown, &self.check) {
            (Shown::Draft, Some(Err(error))) => Some(error.clone()),
            _ => None,
        };

        // Une rangée d'en-tête, toujours là : l'éditeur ne saute pas quand une erreur paraît.
        match self.shown {
            Shown::Distributed(index) => self.distributed_header(ui, index),
            Shown::Draft => self.draft_header(ui, error.as_ref()),
        }
        ui.horizontal_wrapped(|ui| {
            for (index, name) in names.iter().enumerate() {
                let faulty = error.as_ref().and_then(|e| e.file.as_ref()) == Some(name);
                let text = if faulty {
                    RichText::new(format!("{name} (error)")).color(Color32::LIGHT_RED)
                } else {
                    RichText::new(name.as_str())
                };
                if ui.add(egui::Button::selectable(self.tab == index, text)).clicked() {
                    self.tab = index;
                }
            }
        });
        ui.separator();

        let file_name = &names[self.tab];
        let error_line =
            error.as_ref().filter(|e| e.file.as_ref() == Some(file_name)).and_then(|e| e.line);
        let scroll_to_line = self.scroll_to_line.take();
        let id = egui::Id::new(("workbench_editor", self.shown, file_name.as_str()));
        match self.shown {
            Shown::Draft => {
                let Some(draft) = &mut self.draft else { return };
                let changed = code_editor(
                    ui,
                    id,
                    &mut draft.files[self.tab].text,
                    error_line,
                    scroll_to_line,
                );
                if changed {
                    self.changed_since_apply = true;
                    self.recheck();
                    self.persist();
                }
            }
            Shown::Distributed(index) => {
                let Some((_, text)) =
                    StyleProfile::distributed_files().get(index).and_then(|f| f.get(self.tab))
                else {
                    return;
                };
                let mut text: &str = text;
                code_editor(ui, id, &mut text, None, None);
            }
        }
    }

    /// Nom du brouillon et, s'il ne se décode pas, l'erreur et le bouton qui mène à sa ligne.
    fn draft_header(&mut self, ui: &mut egui::Ui, error: Option<&ProfileError>) {
        let Some(draft) = &self.draft else { return };
        let title = match &self.check {
            Some(Ok(name)) => name.clone(),
            _ => draft.folder_name(),
        };
        ui.horizontal(|ui| {
            ui.heading(title);
            match error {
                Some(error) => {
                    ui.colored_label(Color32::LIGHT_RED, error.to_string());
                    if error.file.is_some() && ui.button("Go to line").clicked() {
                        self.go_to_error();
                    }
                }
                None => {
                    ui.weak("draft · editable");
                }
            }
        });
    }

    fn distributed_header(&mut self, ui: &mut egui::Ui, index: usize) {
        let Some(profile) = StyleProfile::distributed().get(index) else { return };
        ui.horizontal(|ui| {
            ui.heading(profile.meta.name.as_str());
            ui.weak(format!("{} · read-only", profile.meta.id));
            let hover = if self.draft.is_some() {
                "Copy this profile into the draft (it replaces the current draft)."
            } else {
                "Copy this profile into an editable draft."
            };
            if ui.button("Fork").on_hover_text(hover).clicked() {
                match Draft::fork(&profile.meta.id) {
                    Ok(draft) => {
                        let text = format!("Forked “{}” into the draft", profile.meta.name);
                        self.set_draft(draft);
                        self.notify(text, false);
                    }
                    Err(error) => self.notify(error, true),
                }
            }
        });
    }
}

// === Éditeur ===

/// L'éditeur d'un fichier TOML : coloré, sans retour à la ligne, numéros de ligne dans la marge,
/// ligne `error_line` (base 1) surlignée ; `scroll_to_line` est amenée au milieu de la vue. Rend
/// `true` si le texte a changé.
fn code_editor(
    ui: &mut egui::Ui,
    id: egui::Id,
    text: &mut dyn egui::TextBuffer,
    error_line: Option<usize>,
    scroll_to_line: Option<usize>,
) -> bool {
    let dark = ui.visuals().dark_mode;
    let mut layouter = |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, _wrap_width: f32| {
        let job = ui.ctx().memory_mut(|memory| {
            memory.caches.cache::<HighlightCache>().get((buffer.as_str(), error_line, dark)).clone()
        });
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    };
    let mut changed = false;
    egui::ScrollArea::both().id_salt(id).auto_shrink([false, false]).show(ui, |ui| {
        let output = egui::TextEdit::multiline(text)
            .id(id)
            .code_editor()
            .margin(egui::Margin { left: GUTTER, right: 4, top: 2, bottom: 2 })
            .desired_width(f32::INFINITY)
            .desired_rows(30)
            .lock_focus(true)
            .layouter(&mut layouter)
            .show(ui);
        changed = output.response.changed();

        // Numéros de ligne : une rangée par ligne (pas de retour à la ligne automatique).
        let painter = ui.painter();
        let visible = ui.clip_rect();
        let font = FontId::monospace(CODE_SIZE);
        let weak = ui.visuals().weak_text_color();
        for (index, row) in output.galley.rows.iter().enumerate() {
            let rect = row.rect().translate(output.galley_pos.to_vec2());
            if rect.bottom() < visible.top() || rect.top() > visible.bottom() {
                continue;
            }
            let line = index + 1;
            let color = if Some(line) == error_line { Color32::LIGHT_RED } else { weak };
            painter.text(
                egui::pos2(output.galley_pos.x - 8.0, rect.top()),
                egui::Align2::RIGHT_TOP,
                line.to_string(),
                font.clone(),
                color,
            );
        }
        if let Some(line) = scroll_to_line
            && let Some(row) = output.galley.rows.get(line.saturating_sub(1))
        {
            let rect: Rect = row.rect().translate(output.galley_pos.to_vec2());
            ui.scroll_to_rect(rect, Some(Align::Center));
        }
    });
    changed
}

/// Couleur d'un morceau de TOML.
const fn token_color(token: TomlToken, dark: bool) -> Color32 {
    match (token, dark) {
        (TomlToken::Comment, true) => Color32::from_rgb(125, 135, 145),
        (TomlToken::Comment, false) => Color32::from_rgb(115, 115, 115),
        (TomlToken::Header, true) => Color32::from_rgb(235, 195, 95),
        (TomlToken::Header, false) => Color32::from_rgb(150, 100, 0),
        (TomlToken::Key, true) => Color32::from_rgb(135, 185, 240),
        (TomlToken::Key, false) => Color32::from_rgb(20, 90, 170),
        (TomlToken::String, true) => Color32::from_rgb(155, 205, 135),
        (TomlToken::String, false) => Color32::from_rgb(40, 125, 40),
        (TomlToken::Literal, true) => Color32::from_rgb(235, 155, 105),
        (TomlToken::Literal, false) => Color32::from_rgb(175, 85, 20),
        (TomlToken::Plain, true) => Color32::from_rgb(215, 215, 215),
        (TomlToken::Plain, false) => Color32::from_rgb(40, 40, 40),
    }
}

/// Fond de la ligne fautive.
const ERROR_LINE_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(110, 25, 25, 160);

/// Le texte coloré, la ligne `error_line` (base 1) sur fond rouge.
fn highlight(text: &str, error_line: Option<usize>, dark: bool) -> LayoutJob {
    let font = FontId::monospace(CODE_SIZE);
    // Octets de la ligne fautive, saut de ligne compris.
    let error_range = error_line.and_then(|line| {
        let start = if line <= 1 {
            0
        } else {
            text.match_indices('\n').nth(line - 2).map(|(at, _)| at + 1)?
        };
        let end = text[start..].find('\n').map_or(text.len(), |at| start + at + 1);
        Some(start..end)
    });
    let mut job = LayoutJob::default();
    for (range, token) in toml_spans(text) {
        let color = token_color(token, dark);
        // Coupé aux bords de la ligne fautive, pour son fond.
        let mut cuts = vec![range.start, range.end];
        if let Some(error) = &error_range {
            cuts.extend([error.start, error.end].into_iter().filter(|c| range.contains(c)));
        }
        cuts.sort_unstable();
        cuts.dedup();
        for pair in cuts.windows(2) {
            let (start, end) = (pair[0], pair[1]);
            let in_error = error_range.as_ref().is_some_and(|e| e.start <= start && end <= e.end);
            let format = TextFormat {
                font_id: font.clone(),
                color,
                background: if in_error { ERROR_LINE_BACKGROUND } else { Color32::TRANSPARENT },
                ..TextFormat::default()
            };
            job.append(&text[start..end], 0.0, format);
        }
    }
    job
}

#[derive(Default)]
struct Highlighter;

impl ComputerMut<(&str, Option<usize>, bool), LayoutJob> for Highlighter {
    fn compute(&mut self, (text, error_line, dark): (&str, Option<usize>, bool)) -> LayoutJob {
        highlight(text, error_line, dark)
    }
}

type HighlightCache = FrameCache<LayoutJob, Highlighter>;

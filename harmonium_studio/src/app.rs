//! L'application, du chargement au premier son : ressources, préparation, Start, puis un écran
//! provisoire qui prouve que le moteur joue (profil, mesure et pulsation, sélecteur de style).
//! L'écran Live est l'affaire de S6.

use eframe::egui::{self, Color32, RichText};
use harmonium_core::{EngineCommand, StyleProfile};
use web_time::Instant;

use crate::{
    loading::{Phase, ResourceStatus},
    manifest::LoadedResources,
    platform::{
        DEFAULT_PROFILE, HostHandle, LoadHandle, LoadOutcome, Pump, Settings, load_view, open_host,
        poll_loading, start_loading, start_pump,
    },
};

/// Où en est l'application.
enum Stage {
    /// Les ressources se chargent.
    Loading(LoadHandle),
    /// Tout est chargé. `drawn` : le message « Preparing instruments… » a été peint ; l'analyse de
    /// la banque, qui bloque le fil, ne commence qu'à l'image suivante.
    Preparing {
        resources: Box<LoadedResources>,
        drawn: bool,
    },
    /// L'hôte existe, en pause : le bouton Start le démarre (geste de l'utilisateur).
    Ready(Session),
    Failed(String),
}

struct Session {
    host: HostHandle,
    /// Le minuteur de génération vit avec la session.
    _pump: Pump,
    playing: bool,
    /// Index dans `StyleProfile::distributed()`.
    profile: usize,
    error: Option<String>,
}

pub struct StudioApp {
    settings: Settings,
    stage: Stage,
    /// Avis du chargement qui restent affichés (cache refusé, première visite).
    notices: Vec<String>,
}

impl StudioApp {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: Settings) -> Self {
        // Le fond de la page de démarrage (HTML) est sombre : pas de flash à la relève.
        cc.egui_ctx.set_theme(egui::ThemePreference::Dark);
        let loading = start_loading(cc.egui_ctx.clone(), &settings);
        Self { settings, stage: Stage::Loading(loading), notices: Vec::new() }
    }

    /// Les changements d'étape, sans dessin : c'est ici que la banque est analysée.
    fn advance(&mut self) {
        match &mut self.stage {
            Stage::Loading(handle) => match poll_loading(handle) {
                LoadOutcome::Pending => {}
                LoadOutcome::Done { resources, notices } => {
                    self.notices = notices;
                    self.stage = Stage::Preparing { resources: Box::new(resources), drawn: false };
                }
                LoadOutcome::Failed(error) => self.stage = Stage::Failed(error),
            },
            Stage::Preparing { resources, drawn: true } => {
                self.stage = match open_session(resources, self.settings.buffer_frames) {
                    Ok(session) => Stage::Ready(session),
                    Err(error) => Stage::Failed(error),
                };
            }
            Stage::Preparing { drawn: false, .. } | Stage::Ready(_) | Stage::Failed(_) => {}
        }
    }
}

fn open_session(
    resources: &LoadedResources,
    buffer_frames: Option<u32>,
) -> Result<Session, String> {
    let host = open_host(resources, buffer_frames)?;
    let pump = start_pump(&host)?;
    let profile = StyleProfile::distributed()
        .iter()
        .position(|profile| profile.meta.id == DEFAULT_PROFILE)
        .unwrap_or(0);
    Ok(Session { host, _pump: pump, playing: false, profile, error: None })
}

impl eframe::App for StudioApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.advance();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add_space(12.0);
            ui.heading("Harmonium Studio");
            ui.add_space(12.0);
            match &mut self.stage {
                Stage::Loading(handle) => loading_screen(ui, handle),
                Stage::Preparing { drawn, .. } => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Preparing instruments…");
                    });
                    // Peint : l'image suivante peut bloquer sur l'analyse de la banque.
                    *drawn = true;
                    ui.ctx().request_repaint();
                }
                Stage::Ready(session) => session_screen(ui, session),
                Stage::Failed(error) => {
                    ui.colored_label(Color32::LIGHT_RED, "Something went wrong.");
                    ui.label(error.as_str());
                }
            }
            if !matches!(self.stage, Stage::Loading(_)) {
                notices(ui, &self.notices);
            }
        });
    }
}

fn notices(ui: &mut egui::Ui, notices: &[String]) {
    if !notices.is_empty() {
        ui.add_space(12.0);
        for notice in notices {
            ui.colored_label(Color32::from_rgb(230, 190, 90), notice);
        }
    }
}

fn loading_screen(ui: &mut egui::Ui, handle: &LoadHandle) {
    let view = load_view(handle);
    match &view.phase {
        Phase::Manifest => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Looking for the instrument files…");
            });
        }
        _ => {
            ui.label("Loading instruments");
            ui.add(egui::ProgressBar::new(view.fraction).show_percentage());
            ui.add_space(8.0);
            egui::Grid::new("resources").num_columns(2).spacing([24.0, 6.0]).show(ui, |ui| {
                for line in &view.lines {
                    ui.label(&line.label);
                    let text = RichText::new(line.status_text());
                    ui.label(if matches!(line.status, ResourceStatus::Failed(_)) {
                        text.color(Color32::LIGHT_RED)
                    } else {
                        text
                    });
                    ui.end_row();
                }
            });
        }
    }
    notices(ui, &view.notices);
}

fn session_screen(ui: &mut egui::Ui, session: &mut Session) {
    if !session.playing {
        ui.label("Instruments are ready.");
        ui.add_space(8.0);
        let start =
            egui::Button::new(RichText::new("Start").size(28.0)).min_size(egui::vec2(180.0, 56.0));
        if ui.add(start).clicked() {
            // Dans le geste de clic : le navigateur n'autorise le son qu'à cette condition.
            match session.host.with(|host| host.resume()) {
                Ok(()) => session.playing = true,
                Err(error) => session.error = Some(error),
            }
        }
    } else {
        playing_screen(ui, session);
        // La pulsation se redessine à chaque image.
        ui.ctx().request_repaint();
    }
    if let Some(error) = &session.error {
        ui.add_space(8.0);
        ui.colored_label(Color32::LIGHT_RED, error.as_str());
    }
}

fn playing_screen(ui: &mut egui::Ui, session: &mut Session) {
    let profiles = StyleProfile::distributed();
    let previous = session.profile;
    ui.horizontal(|ui| {
        ui.label("Style");
        egui::ComboBox::from_id_salt("profile")
            .selected_text(profiles[session.profile].meta.name.as_str())
            .show_ui(ui, |ui| {
                for (index, profile) in profiles.iter().enumerate() {
                    ui.selectable_value(&mut session.profile, index, profile.meta.name.as_str());
                }
            });
    });
    if session.profile != previous {
        let profile = Box::new(profiles[session.profile].clone());
        session.error =
            session.host.with(|host| host.apply(EngineCommand::SetStyleProfile(profile))).err();
    }

    ui.add_space(12.0);
    ui.label(RichText::new(profiles[session.profile].meta.name.as_str()).size(22.0));
    match session.host.with(|host| host.beat_info(Instant::now())) {
        Some(beat) => {
            ui.label(format!("Bar {} · pulse {} / {}", beat.bar, beat.pulse, beat.pulses_per_bar));
            pulse_dots(ui, beat.pulse, beat.pulses_per_bar, beat.phase);
        }
        None => {
            ui.label("Waiting for the first beat…");
        }
    }
}

/// Une pastille par pulsation de la mesure ; la pulsation courante est pleine, le temps fort en
/// couleur, et son anneau se remplit avec la phase.
fn pulse_dots(ui: &mut egui::Ui, pulse: u8, pulses_per_bar: u8, phase: f32) {
    let radius = 16.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(f32::from(pulses_per_bar) * (radius * 2.0 + 10.0), radius * 2.0 + 6.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    for index in 1..=pulses_per_bar {
        let center = egui::pos2(
            rect.left() + radius + f32::from(index - 1) * (radius * 2.0 + 10.0),
            rect.center().y,
        );
        let accent =
            if index == 1 { Color32::from_rgb(240, 170, 60) } else { Color32::from_gray(190) };
        if index == pulse {
            painter.circle_filled(center, radius * (0.55 + 0.45 * (1.0 - phase)), accent);
        }
        painter.circle_stroke(center, radius, egui::Stroke::new(1.5_f32, accent));
    }
}

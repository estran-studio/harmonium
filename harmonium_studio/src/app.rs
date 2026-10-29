//! L'application, du chargement à l'écran Live : ressources, préparation, Start, puis
//! [`LiveScreen`].

use eframe::egui::{self, Color32, RichText};

use crate::{
    live::Intent,
    live_screen::{Credits, LiveScreen},
    loading::{Phase, ResourceStatus},
    manifest::LoadedResources,
    platform::{
        LoadHandle, LoadOutcome, Pump, SessionHandle, Settings, load_view, open_session,
        poll_loading, recording_sink, start_loading, start_pump,
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
    /// La session existe, en pause : le bouton Start la démarre (geste de l'utilisateur).
    Ready(Ready),
    /// La musique joue : l'écran Live.
    Live(Live),
    Failed(String),
}

struct Ready {
    session: SessionHandle,
    pump: Pump,
    credits: Credits,
    error: Option<String>,
}

struct Live {
    screen: Box<LiveScreen>,
    /// Le minuteur de génération vit avec l'écran.
    _pump: Pump,
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
                self.stage = match prepare(resources, self.settings.buffer_frames) {
                    Ok(ready) => Stage::Ready(ready),
                    Err(error) => Stage::Failed(error),
                };
            }
            Stage::Preparing { drawn: false, .. }
            | Stage::Ready(_)
            | Stage::Live(_)
            | Stage::Failed(_) => {}
        }
    }
}

fn prepare(resources: &LoadedResources, buffer_frames: Option<u32>) -> Result<Ready, String> {
    let session = open_session(resources, buffer_frames)?;
    let pump = start_pump(&session)?;
    let credits = Credits {
        feet_bank: resources.feet_bank.is_some(),
        odin_presets: resources.odin_presets.iter().map(|(name, _)| name.to_string()).collect(),
    };
    Ok(Ready { session, pump, credits, error: None })
}

impl eframe::App for StudioApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.advance();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Stage::Live(live) = &mut self.stage {
            live.screen.show(ui);
            return;
        }
        let mut started = false;
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
                Stage::Ready(ready) => started = start_screen(ui, ready),
                Stage::Failed(error) => {
                    ui.colored_label(Color32::LIGHT_RED, "Something went wrong.");
                    ui.label(error.as_str());
                }
                Stage::Live(_) => {}
            }
            if !matches!(self.stage, Stage::Loading(_)) {
                notices(ui, &self.notices);
            }
        });
        if started
            && let Stage::Ready(ready) =
                std::mem::replace(&mut self.stage, Stage::Failed(String::new()))
        {
            let sink = recording_sink(&self.settings);
            let screen = LiveScreen::new(ready.session, sink, ready.credits);
            self.stage = Stage::Live(Live { screen: Box::new(screen), _pump: ready.pump });
        }
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

/// Le bouton Start ; `true` quand la musique a démarré.
fn start_screen(ui: &mut egui::Ui, ready: &mut Ready) -> bool {
    ui.label("Instruments are ready.");
    ui.add_space(8.0);
    let start =
        egui::Button::new(RichText::new("Start").size(28.0)).min_size(egui::vec2(180.0, 56.0));
    let mut started = false;
    if ui.add(start).clicked() {
        // Dans le geste de clic : le navigateur n'autorise le son qu'à cette condition.
        match ready.session.with(|session| session.perform(Intent::Play(true))) {
            Ok(()) => started = true,
            Err(error) => ready.error = Some(error),
        }
    }
    if let Some(error) = &ready.error {
        ui.add_space(8.0);
        ui.colored_label(Color32::LIGHT_RED, error.as_str());
    }
    started
}

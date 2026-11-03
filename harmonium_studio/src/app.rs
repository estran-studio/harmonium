//! L'application, du chargement à l'écran Live : ressources, préparation, Start, puis
//! [`LiveScreen`].

use std::collections::VecDeque;

use eframe::egui::{self, Color32, RichText};

use crate::{
    link::Opening,
    live::Intent,
    live_screen::{Credits, LiveScreen},
    loading::{Phase, ResourceStatus},
    manifest::LoadedResources,
    platform::{
        LoadHandle, LoadOutcome, OpenInbox, Pump, SessionHandle, Settings, load_view, open_inbox,
        open_session, poll_loading, recording_sink, start_loading, start_pump,
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
    /// D'où viennent les demandes d'ouverture (lien de `--open`, canal de commande, URL de la
    /// page).
    inbox: OpenInbox,
    /// Demandes reçues, exécutées dès que l'écran Live existe.
    pending: VecDeque<Result<Opening, String>>,
}

impl StudioApp {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: Settings) -> Self {
        // Le fond de la page de démarrage (HTML) est sombre : pas de flash à la relève.
        cc.egui_ctx.set_theme(egui::ThemePreference::Dark);
        let loading = start_loading(cc.egui_ctx.clone(), &settings);
        // Ouvert avant le chargement : un lien envoyé pendant les secondes de chargement attend.
        let inbox = open_inbox(&cc.egui_ctx, &settings);
        Self {
            settings,
            stage: Stage::Loading(loading),
            notices: Vec::new(),
            inbox,
            pending: VecDeque::new(),
        }
    }

    /// L'écran Live à partir d'une session prête ; les demandes d'ouverture en attente s'y
    /// exécutent.
    fn go_live(&mut self, ready: Ready) {
        let sink = recording_sink(&self.settings);
        let mut screen = LiveScreen::new(ready.session, sink, ready.credits);
        deliver(&mut screen, &mut self.pending);
        self.stage = Stage::Live(Live { screen: Box::new(screen), _pump: ready.pump });
    }

    /// Les changements d'étape, sans dessin : c'est ici que la banque est analysée.
    fn advance(&mut self) {
        // Natif : une demande d'ouverture dispense du bouton Start (le bouton n'existe que pour le
        // geste que le navigateur exige). L'écran arrive en pause : la demande dit si elle joue.
        if cfg!(not(target_arch = "wasm32"))
            && self.pending.iter().any(Result::is_ok)
            && matches!(self.stage, Stage::Ready(_))
            && let Stage::Ready(ready) =
                std::mem::replace(&mut self.stage, Stage::Failed(String::new()))
        {
            self.go_live(ready);
            return;
        }
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
        percussion_bank: resources.percussion_bank.is_some(),
        odin_presets: resources.odin_presets.iter().map(|(name, _)| name.to_string()).collect(),
    };
    Ok(Ready { session, pump, credits, error: None })
}

impl eframe::App for StudioApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.pending.extend(self.inbox.take());
        self.advance();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Stage::Live(live) = &mut self.stage {
            deliver(&mut live.screen, &mut self.pending);
            live.screen.show(ui);
            return;
        }
        let mut started = false;
        let has_open = self.pending.iter().any(Result::is_ok);
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
                Stage::Ready(ready) => started = start_screen(ui, ready, has_open),
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
            self.go_live(ready);
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
fn start_screen(ui: &mut egui::Ui, ready: &mut Ready, has_open: bool) -> bool {
    ui.label("Instruments are ready.");
    ui.add_space(8.0);
    let start =
        egui::Button::new(RichText::new("Start").size(28.0)).min_size(egui::vec2(180.0, 56.0));
    let mut started = false;
    if ui.add(start).clicked() {
        if has_open {
            // Un lien attend : c'est lui qui dit si la musique joue (`play=1`), pas Start.
            started = true;
        } else {
            // Dans le geste de clic : le navigateur n'autorise le son qu'à cette condition.
            match ready.session.with(|session| session.perform(Intent::Play(true))) {
                Ok(()) => started = true,
                Err(error) => ready.error = Some(error),
            }
        }
    }
    if let Some(error) = &ready.error {
        ui.add_space(8.0);
        ui.colored_label(Color32::LIGHT_RED, error.as_str());
    }
    started
}

/// Remet à l'écran les demandes d'ouverture reçues : il les exécute, ou montre l'erreur de celle
/// qui n'a pas pu être lue.
fn deliver(screen: &mut LiveScreen, pending: &mut VecDeque<Result<Opening, String>>) {
    for opening in pending.drain(..) {
        match opening {
            Ok(opening) => screen.open(opening),
            Err(message) => screen.notify(message, true),
        }
    }
}

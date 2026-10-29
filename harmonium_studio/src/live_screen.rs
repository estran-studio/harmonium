//! L'écran Live dessiné par egui : barre du haut, commandes à gauche, partition au centre
//! (forme, accords, piano-roll, tête de lecture), horloge, état et mixeur à droite, panneau
//! « About ».
//!
//! Chaque image lit une [`LiveView`] (sortie publiée du moteur) et rend des [`Intent`] que la
//! session exécute après le dessin : l'écran ne touche jamais le moteur autrement.

use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2,
    WidgetInfo, WidgetType,
};
use harmonium::playback::MEASURE_WINDOW_CAP;
use harmonium_core::{
    EngineCommand, StyleProfile,
    timeline::{Articulation, TrackId},
};
use web_time::Instant;

use crate::{
    live::{
        BarView, BeatView, Intent, LiveView, MixerStrip, NoteView, RECORD_FORMATS, ScoreView,
        Session, TRACKS, WINDOW_BARS, format_label, track_label,
    },
    platform::{RecordingSink, SessionHandle},
};

/// Adresse des sources publiques (licences : panneau « About »).
const SOURCE_URL: &str = "https://github.com/estran-studio/harmonium";
const ODIN_RS_URL: &str = "https://github.com/estran-studio/odin-rs";
const ODIN2_URL: &str = "https://github.com/TheWaveWarden/odin2";
const GENERALUSER_URL: &str = "https://github.com/mrbumpy409/GeneralUser-GS";

/// Avis gardés au bas de l'écran.
const NOTICES_KEPT: usize = 4;

/// Ce qui a été chargé, pour les crédits du panneau « About ».
#[derive(Clone, Debug, Default)]
pub struct Credits {
    /// Une banque de pieds est chargée.
    pub feet_bank: bool,
    /// Presets Odin chargés (noms courts).
    pub odin_presets: Vec<String>,
}

struct Notice {
    text: String,
    error: bool,
}

/// Un glisser sur la partition : la boucle en préparation.
#[derive(Clone, Copy)]
struct LoopDrag {
    from: usize,
    to: usize,
}

/// L'écran Live.
pub struct LiveScreen {
    session: SessionHandle,
    sink: RecordingSink,
    credits: Credits,
    about_open: bool,
    notices: Vec<Notice>,
    seed_text: String,
    /// Tempo audible à imposer et sa rampe (champ de saisie).
    audible_tempo: Option<(f32, u8)>,
    /// Signature à appliquer (champ de saisie).
    time_signature: Option<(usize, usize)>,
    /// Mesure du champ de saut.
    seek_bar: usize,
    loop_drag: Option<LoopDrag>,
    #[cfg(not(target_arch = "wasm32"))]
    osc: Option<crate::platform::Osc>,
    /// Dossier de profil à charger (natif ; champ de saisie).
    #[cfg(not(target_arch = "wasm32"))]
    profile_dir: String,
}

impl LiveScreen {
    #[must_use]
    pub fn new(session: SessionHandle, sink: RecordingSink, credits: Credits) -> Self {
        Self {
            session,
            sink,
            credits,
            about_open: false,
            notices: Vec::new(),
            seed_text: String::new(),
            audible_tempo: None,
            time_signature: None,
            seek_bar: 1,
            loop_drag: None,
            #[cfg(not(target_arch = "wasm32"))]
            osc: None,
            #[cfg(not(target_arch = "wasm32"))]
            profile_dir: String::new(),
        }
    }

    fn notify(&mut self, text: impl Into<String>, error: bool) {
        self.notices.push(Notice { text: text.into(), error });
        if self.notices.len() > NOTICES_KEPT {
            self.notices.remove(0);
        }
    }

    /// Dessine l'écran dans `ui`, puis exécute ce que l'utilisateur a demandé.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        for recording in self.session.with(Session::take_recordings) {
            match self.sink.save(&recording) {
                Ok(text) => self.notify(text, false),
                Err(error) => self.notify(format!("Could not save the recording: {error}"), true),
            }
        }
        let view = self.session.with(|session| session.view(Instant::now()));
        let mut intents = Vec::new();

        egui::Panel::top("live_top").show_inside(ui, |ui| self.top_bar(ui, &view, &mut intents));
        egui::Panel::bottom("live_notices").show_inside(ui, |ui| self.notices_bar(ui));
        egui::Panel::left("live_controls").resizable(false).exact_size(300.0).show_inside(
            ui,
            |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui, &view, &mut intents));
            },
        );
        egui::Panel::right("live_monitor").resizable(false).exact_size(320.0).show_inside(
            ui,
            |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    beat_clock(ui, &view);
                    ui.separator();
                    state_panel(ui, &view);
                    ui.separator();
                    mixer(ui, &view, &mut intents);
                });
            },
        );
        egui::CentralPanel::default().show_inside(ui, |ui| self.score(ui, &view, &mut intents));
        self.about(ui.ctx());

        for intent in intents {
            if let Err(error) = self.session.with(|session| session.perform(intent)) {
                self.notify(error, true);
            }
        }
        // L'horloge et la tête de lecture bougent à chaque image.
        ui.ctx().request_repaint();
    }

    // === Barre du haut ===

    fn top_bar(&mut self, ui: &mut egui::Ui, view: &LiveView, intents: &mut Vec<Intent>) {
        ui.horizontal_wrapped(|ui| {
            ui.heading("Harmonium Studio");
            ui.separator();
            ui.add(egui::Button::selectable(true, "Live"));
            ui.add_enabled(false, egui::Button::selectable(false, "Workbench"))
                .on_disabled_hover_text("The profile workbench is not available yet.");
            ui.separator();

            ui.label("Style");
            // Tous les profils distribués tiennent dans la liste, sans défilement.
            egui::ComboBox::from_id_salt("style")
                .height(400.0)
                .selected_text(view.profile_name.as_str())
                .show_ui(ui, |ui| {
                    for profile in StyleProfile::distributed() {
                        let selected = profile.meta.id == view.profile_id;
                        if ui.selectable_label(selected, profile.meta.name.as_str()).clicked()
                            && !selected
                        {
                            intents.push(Intent::Command(EngineCommand::SetStyleProfile(
                                Box::new(profile.clone()),
                            )));
                        }
                    }
                });
            ui.separator();

            let play = if view.desk.playing { "Pause" } else { "Play" };
            if ui.button(play).clicked() {
                intents.push(Intent::Play(!view.desk.playing));
            }
            let tempo = view
                .audible_bpm
                .map_or_else(|| "Tempo —".to_string(), |bpm| format!("Tempo {bpm:.0} BPM"));
            ui.label(RichText::new(tempo).strong())
                .on_hover_text("Quarter-note tempo heard now, from the beat clock.");
            if let Some((numerator, denominator)) = view.sounding_signature {
                ui.label(RichText::new(format!("{numerator}/{denominator}")).strong());
            }
            ui.separator();

            ui.label(format!("Seed {}", view.seed));
            ui.add(
                egui::TextEdit::singleline(&mut self.seed_text)
                    .desired_width(90.0)
                    .hint_text("seed"),
            );
            if ui.button("Set seed").clicked() {
                match self.seed_text.trim().parse::<u64>() {
                    Ok(seed) => intents.push(Intent::Command(EngineCommand::SetSeed(seed))),
                    Err(_) => self.notify(format!("Invalid seed: {}", self.seed_text.trim()), true),
                }
            }
            if ui.button("New seed").clicked() {
                intents.push(Intent::Command(EngineCommand::NewMelody));
            }
            ui.separator();

            ui.label("Record");
            for (index, format) in RECORD_FORMATS.into_iter().enumerate() {
                let on = view.desk.recording[index];
                let text = RichText::new(format_label(format));
                let text =
                    if on { text.color(Color32::from_rgb(240, 90, 80)).strong() } else { text };
                if ui.add(egui::Button::selectable(on, text)).clicked() {
                    intents.push(Intent::Command(if on {
                        EngineCommand::StopRecording(format)
                    } else {
                        EngineCommand::StartRecording(format)
                    }));
                }
            }
            ui.separator();
            if ui.button("About").clicked() {
                self.about_open = true;
            }
        });
    }

    fn notices_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if self.notices.is_empty() {
                ui.weak("Click a bar to seek · drag across bars to loop (up to 16) · right-click a bar for more.");
            }
            for notice in self.notices.iter().rev() {
                let color = if notice.error {
                    Color32::LIGHT_RED
                } else {
                    Color32::from_rgb(150, 200, 150)
                };
                ui.colored_label(color, notice.text.as_str());
                ui.separator();
            }
        });
    }

    // === Commandes (gauche) ===

    fn controls(&mut self, ui: &mut egui::Ui, view: &LiveView, intents: &mut Vec<Intent>) {
        let command = |intents: &mut Vec<Intent>, command| intents.push(Intent::Command(command));
        let [arousal, valence, density, tension] = view.emotions;

        ui.heading("Emotions");
        ui.horizontal(|ui| {
            ui.label("Control");
            if ui.add(egui::Button::selectable(view.emotion_mode, "Emotion")).clicked() {
                command(intents, EngineCommand::UseEmotionMode);
            }
            if ui.add(egui::Button::selectable(!view.emotion_mode, "Direct")).clicked() {
                command(intents, EngineCommand::UseDirectMode);
            }
        });
        if !view.emotion_mode {
            ui.weak("Direct mode: moving an emotion switches back to emotion mode.");
        }
        emotion_pad(ui, view.emotions, intents);
        let mut emotions = view.emotions;
        let mut changed = false;
        changed |= ui.add(egui::Slider::new(&mut emotions[0], 0.0..=1.0).text("Arousal")).changed();
        changed |=
            ui.add(egui::Slider::new(&mut emotions[1], -1.0..=1.0).text("Valence")).changed();
        changed |= ui.add(egui::Slider::new(&mut emotions[2], 0.0..=1.0).text("Density")).changed();
        changed |= ui.add(egui::Slider::new(&mut emotions[3], 0.0..=1.0).text("Tension")).changed();
        if changed && emotions != [arousal, valence, density, tension] {
            let [arousal, valence, density, tension] = emotions;
            command(
                intents,
                EngineCommand::SetEmotionParams { arousal, valence, density, tension },
            );
        }
        let mut intensity = view.intensity;
        if ui.add(egui::Slider::new(&mut intensity, 0.0..=1.0).text("Intensity")).changed() {
            command(intents, EngineCommand::SetIntensity(intensity));
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Feet");
            for density in 0..=2u8 {
                if ui
                    .add(egui::Button::selectable(view.feet == density, density.to_string()))
                    .clicked()
                    && view.feet != density
                {
                    command(intents, EngineCommand::SetFeetDensity(density));
                }
            }
        });

        ui.separator();
        ui.heading("Tempo");
        ui.horizontal(|ui| {
            ui.label("Nominal");
            let mut bpm = view.nominal_bpm;
            if ui
                .add(egui::DragValue::new(&mut bpm).range(20.0..=400.0).speed(0.5).suffix(" BPM"))
                .changed()
            {
                command(intents, EngineCommand::SetBpm(bpm));
            }
            if ui.button("Reset BPM").on_hover_text("Back to the emotion-mapped tempo.").clicked() {
                command(intents, EngineCommand::ResetBpm);
            }
        });
        let (mut audible, mut ramp) = *self
            .audible_tempo
            .get_or_insert((view.audible_bpm.unwrap_or(view.nominal_bpm).round(), 0));
        ui.horizontal(|ui| {
            ui.label("Audible");
            ui.add(
                egui::DragValue::new(&mut audible).range(40.0..=300.0).speed(0.5).suffix(" BPM"),
            );
            ui.add(egui::DragValue::new(&mut ramp).range(0..=64).suffix(" pulses"))
                .on_hover_text("Ramp length; 0 jumps at the next pulse.");
        });
        self.audible_tempo = Some((audible, ramp));
        ui.horizontal(|ui| {
            if ui.button("Apply tempo").clicked() {
                command(
                    intents,
                    EngineCommand::SetAudibleTempo { bpm: audible, ramp_pulses: ramp },
                );
            }
            if ui
                .add_enabled(view.desk.audible_tempo.is_some(), egui::Button::new("Clear tempo"))
                .clicked()
            {
                command(intents, EngineCommand::ClearAudibleTempo);
            }
        });
        ui.weak(match view.desk.audible_tempo {
            Some((bpm, 0)) => format!("Audible tempo held at {bpm:.0} BPM."),
            Some((bpm, ramp)) => format!("Audible tempo ramps to {bpm:.0} BPM over {ramp} pulses."),
            None => "Audible tempo follows the bars.".to_string(),
        });
        let mut swing = view.swing;
        if ui.add(egui::Slider::new(&mut swing, 0.5..=0.75).text("Swing")).changed() {
            command(intents, EngineCommand::SetSwing(swing));
        }
        let (mut numerator, mut denominator) =
            *self.time_signature.get_or_insert(view.time_signature);
        ui.horizontal(|ui| {
            ui.label("Time signature");
            ui.add(egui::DragValue::new(&mut numerator).range(1..=16));
            ui.label("/");
            egui::ComboBox::from_id_salt("denominator")
                .width(48.0)
                .selected_text(denominator.to_string())
                .show_ui(ui, |ui| {
                    for value in [2usize, 4, 8, 16] {
                        ui.selectable_value(&mut denominator, value, value.to_string());
                    }
                });
            if ui.button("Apply").clicked() {
                command(intents, EngineCommand::SetTimeSignature { numerator, denominator });
            }
        });
        self.time_signature = Some((numerator, denominator));
        ui.weak(format!(
            "Next bars: {:.0} BPM, {}/{}.",
            view.nominal_bpm, view.time_signature.0, view.time_signature.1
        ));

        ui.separator();
        if ui
            .button("Reset engine")
            .on_hover_text("Back to the style's defaults, no loop, restart at bar 1 (same seed).")
            .clicked()
        {
            command(intents, EngineCommand::Reset);
        }

        ui.separator();
        egui::CollapsingHeader::new("Engine").default_open(false).show(ui, |ui| {
            let names = ["Rhythm", "Harmony", "Melody"];
            ui.horizontal(|ui| {
                for (index, name) in names.into_iter().enumerate() {
                    let mut on = view.modules[index];
                    if ui.checkbox(&mut on, name).changed() {
                        command(
                            intents,
                            match index {
                                0 => EngineCommand::EnableRhythm(on),
                                1 => EngineCommand::EnableHarmony(on),
                                _ => EngineCommand::EnableMelody(on),
                            },
                        );
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Lookahead");
                let mut bars = view.desk.lookahead.unwrap_or(4);
                if ui
                    .add(
                        egui::DragValue::new(&mut bars)
                            .range(4..=MEASURE_WINDOW_CAP)
                            .suffix(" bars"),
                    )
                    .changed()
                {
                    command(intents, EngineCommand::SetWriteheadLookahead(bars));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Bar");
                ui.add(egui::DragValue::new(&mut self.seek_bar).range(1..=100_000));
                if ui.button("Seek").on_hover_text("Recompose from this bar (same seed).").clicked()
                {
                    command(intents, EngineCommand::Seek(self.seek_bar));
                }
                if ui
                    .button("Move playhead")
                    .on_hover_text("Jump to this bar without recomposing.")
                    .clicked()
                {
                    command(intents, EngineCommand::SeekPlayhead(self.seek_bar));
                }
            });
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.profile_folder_row(ui, intents);
                self.osc_row(ui);
            }
        });
    }

    /// Charge un profil depuis un dossier (fixtures, worktrees), comme `style <dossier>` du REPL.
    #[cfg(not(target_arch = "wasm32"))]
    fn profile_folder_row(&mut self, ui: &mut egui::Ui, intents: &mut Vec<Intent>) {
        ui.horizontal(|ui| {
            ui.label("Profile folder");
            ui.add(
                egui::TextEdit::singleline(&mut self.profile_dir)
                    .desired_width(130.0)
                    .hint_text("path"),
            );
            if ui.button("Load").clicked() {
                match StyleProfile::load(self.profile_dir.trim()) {
                    Ok(profile) => intents
                        .push(Intent::Command(EngineCommand::SetStyleProfile(Box::new(profile)))),
                    Err(error) => self.notify(error.to_string(), true),
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn osc_row(&mut self, ui: &mut egui::Ui) {
        use harmonium::osc;

        let mut on = self.osc.is_some();
        if ui.checkbox(&mut on, "OSC listener").changed() {
            if on {
                match crate::platform::start_osc(&self.session, osc::DEFAULT_ADDR) {
                    Ok(listener) => {
                        let text = format!(
                            "OSC listening on {} ({})",
                            listener.address(),
                            osc::PARAMS_ADDRESS
                        );
                        self.osc = Some(listener);
                        self.notify(text, false);
                    }
                    Err(error) => self.notify(error, true),
                }
            } else {
                self.osc = None;
            }
        }
        if let Some(listener) = &self.osc {
            ui.weak(format!(
                "{} {} <arousal> <valence> <density> <tension>",
                listener.address(),
                osc::PARAMS_ADDRESS
            ));
        }
    }

    // === Partition (centre) ===

    fn score(&mut self, ui: &mut egui::Ui, view: &LiveView, intents: &mut Vec<Intent>) {
        let score = &view.score;
        ui.horizontal(|ui| {
            match score.loop_range {
                Some((from, to)) => {
                    ui.label(format!("Loop: bars {from}–{to}"));
                    if ui.button("Clear loop").clicked() {
                        intents.push(Intent::Command(EngineCommand::ClearLoop));
                    }
                }
                None => {
                    ui.weak("No loop");
                }
            }
            ui.separator();
            ui.weak(format!(
                "Bar {} · composed through {}",
                score.current_bar,
                score.composed_through.map_or_else(|| "—".to_string(), |bar| bar.to_string())
            ));
        });

        let size = ui.available_size();
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let layout = ScoreLayout::new(rect);
        let pointer_bar = ui
            .input(|input| input.pointer.interact_pos())
            .and_then(|pos| layout.bar_at(pos, score.first_bar));

        // Une cible par mesure (accessible : « Bar 12 ») : clic = saut, clic droit = menu,
        // glisser = boucle.
        let mut dragging = false;
        for (offset, bar) in score.bars.iter().enumerate() {
            let response = ui.interact(
                layout.column(offset),
                ui.id().with(("bar", bar.index)),
                Sense::click_and_drag(),
            );
            response.widget_info(|| {
                WidgetInfo::labeled(WidgetType::Button, true, format!("Bar {}", bar.index))
            });
            if response.clicked() {
                intents.push(Intent::Command(EngineCommand::Seek(bar.index)));
            }
            if response.drag_started() {
                self.loop_drag = Some(LoopDrag { from: bar.index, to: bar.index });
            }
            dragging |= response.dragged();
            if response.drag_stopped()
                && let Some(drag) = self.loop_drag.take()
            {
                intents.push(Intent::Loop(drag.from, pointer_bar.unwrap_or(drag.to)));
            }
            response.context_menu(|ui| {
                ui.label(format!("Bar {}", bar.index));
                if ui
                    .button("Seek here")
                    .on_hover_text("Recompose from this bar (same seed).")
                    .clicked()
                {
                    intents.push(Intent::Command(EngineCommand::Seek(bar.index)));
                    ui.close();
                }
                if ui
                    .button("Move playhead here")
                    .on_hover_text("Jump without recomposing.")
                    .clicked()
                {
                    intents.push(Intent::Command(EngineCommand::SeekPlayhead(bar.index)));
                    ui.close();
                }
                if score.loop_range.is_some() && ui.button("Clear loop").clicked() {
                    intents.push(Intent::Command(EngineCommand::ClearLoop));
                    ui.close();
                }
            });
        }
        if dragging && let (Some(drag), Some(bar)) = (&mut self.loop_drag, pointer_bar) {
            drag.to = bar;
        }

        paint_score(ui, &layout, score, self.loop_drag);
    }

    // === About ===

    fn about(&mut self, ctx: &egui::Context) {
        let credits = &self.credits;
        egui::Window::new("About Harmonium Studio")
            .open(&mut self.about_open)
            .collapsible(false)
            .resizable(false)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.label(
                    "Harmonium is a procedural music engine driven by four emotions \
                     (arousal, valence, density, tension).",
                );
                ui.add_space(6.0);
                ui.strong("Licences");
                ui.label("Harmonium: MIT OR Apache-2.0.");
                ui.horizontal_wrapped(|ui| {
                    ui.label("Odin synthesizer (odin2-core,");
                    ui.hyperlink_to("odin-rs", ODIN_RS_URL);
                    ui.label("): GPL-3.0, derived from");
                    ui.hyperlink_to("Odin 2", ODIN2_URL);
                    ui.label("by TheWaveWarden.");
                });
                ui.label(
                    "This application is built with the Odin synthesizer, so the application \
                     as distributed is licensed under the GPL-3.0.",
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label("Complete source:");
                    ui.hyperlink(SOURCE_URL);
                });
                ui.add_space(6.0);
                ui.strong("Sounds");
                ui.horizontal_wrapped(|ui| {
                    ui.label("Instrument bank:");
                    ui.hyperlink_to("GeneralUser GS", GENERALUSER_URL);
                    ui.label("by S. Christian Collins.");
                });
                if credits.feet_bank {
                    ui.label("Foot percussion: CC0 samples from Freesound.");
                }
                if credits.odin_presets.is_empty() {
                    ui.label("Odin presets: none loaded.");
                } else {
                    ui.label(format!("Odin presets: {}.", credits.odin_presets.join(", ")));
                }
                ui.add_space(6.0);
                ui.weak("Interface: egui and eframe (MIT OR Apache-2.0).");
            });
    }
}

// === Pavé d'émotions ===

/// Pavé valence (horizontale) × arousal (verticale) ; densité et tension restent.
fn emotion_pad(ui: &mut egui::Ui, emotions: [f32; 4], intents: &mut Vec<Intent>) {
    let [arousal, valence, density, tension] = emotions;
    let width = ui.available_width().min(260.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 150.0), Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, "Emotion pad"));
    if (response.clicked() || response.dragged())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let new_valence = ((pos.x - rect.left()) / rect.width() * 2.0 - 1.0).clamp(-1.0, 1.0);
        let new_arousal = (1.0 - (pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
        if (new_valence - valence).abs() > 0.005 || (new_arousal - arousal).abs() > 0.005 {
            intents.push(Intent::Command(EngineCommand::SetEmotionParams {
                arousal: new_arousal,
                valence: new_valence,
                density,
                tension,
            }));
        }
    }
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, Color32::from_gray(28));
    let grid = Stroke::new(1.0_f32, Color32::from_gray(55));
    painter.line_segment([rect.center_top(), rect.center_bottom()], grid);
    painter.line_segment([rect.left_center(), rect.right_center()], grid);
    let font = FontId::proportional(10.0);
    let dim = Color32::from_gray(130);
    painter.text(
        rect.center_top() + Vec2::new(4.0, 2.0),
        Align2::LEFT_TOP,
        "arousal +",
        font.clone(),
        dim,
    );
    painter.text(
        rect.right_center() + Vec2::new(-4.0, 2.0),
        Align2::RIGHT_TOP,
        "valence +",
        font.clone(),
        dim,
    );
    painter.text(
        rect.left_center() + Vec2::new(4.0, 2.0),
        Align2::LEFT_TOP,
        "valence −",
        font,
        dim,
    );
    let point = Pos2::new(
        rect.left() + (valence + 1.0) / 2.0 * rect.width(),
        rect.top() + (1.0 - arousal) * rect.height(),
    );
    painter.circle_filled(point, 6.0, Color32::from_rgb(240, 170, 60));
}

// === Horloge, état, mixeur (droite) ===

fn beat_clock(ui: &mut egui::Ui, view: &LiveView) {
    ui.heading("Beat");
    match &view.beat {
        Some(beat) => beat_view(ui, beat),
        None => {
            ui.label("Waiting for the first beat…");
        }
    }
}

fn beat_view(ui: &mut egui::Ui, beat: &BeatView) {
    ui.label(
        RichText::new(format!("Bar {} · pulse {} / {}", beat.bar, beat.pulse, beat.pulses_per_bar))
            .size(18.0),
    );
    pulse_dots(ui, beat.pulse, beat.pulses_per_bar, beat.phase);
    ui.add(egui::ProgressBar::new(beat.phase).desired_width(240.0).text("phase"));
    ui.horizontal(|ui| {
        if beat.downbeat {
            ui.label(RichText::new("DOWNBEAT").color(Color32::from_rgb(240, 170, 60)).strong());
        }
        ui.weak(format!(
            "next pulse in {} ms · period {:.0} ms",
            beat.next_pulse_in.as_millis(),
            beat.pulse_period.as_secs_f64() * 1000.0
        ));
    });
}

/// Une pastille par pulsation de la mesure ; la pulsation courante est pleine, le temps fort en
/// couleur, et son disque se resserre avec la phase.
fn pulse_dots(ui: &mut egui::Ui, pulse: u8, pulses_per_bar: u8, phase: f32) {
    let radius = 14.0;
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(f32::from(pulses_per_bar) * (radius * 2.0 + 10.0), radius * 2.0 + 6.0),
        Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    for index in 1..=pulses_per_bar {
        let center = Pos2::new(
            rect.left() + radius + f32::from(index - 1) * (radius * 2.0 + 10.0),
            rect.center().y,
        );
        let accent =
            if index == 1 { Color32::from_rgb(240, 170, 60) } else { Color32::from_gray(190) };
        if index == pulse {
            painter.circle_filled(center, radius * (0.55 + 0.45 * (1.0 - phase)), accent);
        }
        painter.circle_stroke(center, radius, Stroke::new(1.5_f32, accent));
    }
}

fn state_panel(ui: &mut egui::Ui, view: &LiveView) {
    egui::CollapsingHeader::new("State").default_open(true).show(ui, |ui| {
        for group in &view.state {
            ui.strong(group.title);
            egui::Grid::new(("state", group.title)).num_columns(2).spacing([12.0, 2.0]).show(
                ui,
                |ui| {
                    for (label, value) in &group.rows {
                        ui.weak(*label);
                        ui.label(value.as_str());
                        ui.end_row();
                    }
                },
            );
        }
    });
}

fn mixer(ui: &mut egui::Ui, view: &LiveView, intents: &mut Vec<Intent>) {
    ui.heading("Mixer");
    egui::Grid::new("mixer").num_columns(4).spacing([6.0, 4.0]).show(ui, |ui| {
        for strip in &view.mixer {
            mixer_strip(ui, strip, intents);
            ui.end_row();
        }
    });
    let mut volume = view.desk.master_volume;
    if ui.add(egui::Slider::new(&mut volume, 0.0..=1.0).text("Master volume")).changed() {
        intents.push(Intent::Command(EngineCommand::SetMasterVolume(volume)));
    }
    let mut muted = view.desk.output_muted;
    if ui.checkbox(&mut muted, "Mute output").changed() {
        intents.push(Intent::Command(EngineCommand::SetOutputMute(muted)));
    }
    egui::CollapsingHeader::new("Routing").default_open(false).show(ui, |ui| {
        ui.weak("Bank per track; -1 is the default bank.");
        egui::Grid::new("routing").num_columns(2).show(ui, |ui| {
            for strip in &view.mixer {
                ui.label(track_label(strip.track));
                let mut bank = strip.route;
                if ui.add(egui::DragValue::new(&mut bank).range(-1..=15)).changed() {
                    intents.push(Intent::Command(EngineCommand::SetChannelRoute {
                        channel: strip.track.channel(),
                        bank_id: bank,
                    }));
                }
                ui.end_row();
            }
        });
    });
}

fn mixer_strip(ui: &mut egui::Ui, strip: &MixerStrip, intents: &mut Vec<Intent>) {
    let name = RichText::new(track_label(strip.track)).color(track_color(strip.track));
    ui.label(if strip.audible { name } else { name.weak() });
    let mut gain = strip.gain;
    if ui.add(egui::Slider::new(&mut gain, 0.0..=1.0).show_value(false)).changed() {
        intents.push(Intent::Command(EngineCommand::SetChannelGain {
            channel: strip.track.channel(),
            gain,
        }));
    }
    if ui.add(egui::Button::selectable(strip.muted, "Mute")).clicked() {
        intents.push(Intent::Mute(strip.track, !strip.muted));
    }
    if ui.add(egui::Button::selectable(strip.soloed, "Solo")).clicked() {
        intents.push(Intent::Solo(if strip.soloed { None } else { Some(strip.track) }));
    }
}

// === Dessin de la partition ===

const fn track_color(track: TrackId) -> Color32 {
    match track {
        TrackId::Melody => Color32::from_rgb(240, 170, 60),
        TrackId::Countermelody => Color32::from_rgb(230, 120, 160),
        TrackId::Chords => Color32::from_rgb(110, 170, 240),
        TrackId::Bass => Color32::from_rgb(120, 210, 140),
        TrackId::Heel => Color32::from_rgb(200, 200, 200),
        TrackId::Toe => Color32::from_rgb(160, 160, 175),
    }
}

const PART_COLORS: [Color32; 4] = [
    Color32::from_rgb(64, 96, 150),
    Color32::from_rgb(64, 130, 96),
    Color32::from_rgb(140, 96, 150),
    Color32::from_rgb(150, 120, 64),
];

/// Les rangées de la partition et la largeur d'une mesure.
struct ScoreLayout {
    rect: Rect,
    left: f32,
    bar_width: f32,
    ruler: Rect,
    tune: Rect,
    form: Rect,
    chords: Rect,
    lanes: [Rect; TRACKS.len()],
}

impl ScoreLayout {
    const LABEL_WIDTH: f32 = 70.0;
    const RULER: f32 = 18.0;
    const TUNE: f32 = 18.0;
    const FORM: f32 = 24.0;
    const CHORDS: f32 = 22.0;

    fn new(rect: Rect) -> Self {
        let left = rect.left() + Self::LABEL_WIDTH;
        let bar_width = ((rect.right() - left) / WINDOW_BARS as f32).max(1.0);
        let row = |top: f32, height: f32| {
            Rect::from_min_max(Pos2::new(left, top), Pos2::new(rect.right(), top + height))
        };
        let ruler = row(rect.top(), Self::RULER);
        let tune = row(ruler.bottom(), Self::TUNE);
        let form = row(tune.bottom(), Self::FORM);
        let chords = row(form.bottom(), Self::CHORDS);
        // Les pistes mélodiques ont plus de hauteur que les frappes de pieds.
        let lanes_top = chords.bottom() + 4.0;
        let unit = ((rect.bottom() - lanes_top) / LANE_WEIGHTS.iter().sum::<f32>()).max(4.0);
        let mut top = lanes_top;
        let lanes = LANE_WEIGHTS.map(|weight| {
            let lane = Rect::from_min_max(
                Pos2::new(left, top),
                Pos2::new(rect.right(), top + weight * unit - 2.0),
            );
            top += weight * unit;
            lane
        });
        Self { rect, left, bar_width, ruler, tune, form, chords, lanes }
    }

    fn bar_x(&self, offset: f32) -> f32 {
        self.left + offset * self.bar_width
    }

    /// La colonne de la mesure `offset` (depuis `first_bar`), sur toute la hauteur.
    fn column(&self, offset: usize) -> Rect {
        let x = self.bar_x(offset as f32);
        Rect::from_min_max(
            Pos2::new(x, self.rect.top()),
            Pos2::new(x + self.bar_width, self.rect.bottom()),
        )
    }

    /// La mesure sous `pos`, si elle est dans la grille.
    fn bar_at(&self, pos: Pos2, first_bar: usize) -> Option<usize> {
        if pos.x < self.left || pos.x > self.rect.right() || !self.rect.contains(pos) {
            return None;
        }
        let offset = ((pos.x - self.left) / self.bar_width).floor() as usize;
        (offset < WINDOW_BARS).then_some(first_bar + offset)
    }

    fn lane(&self, index: usize) -> Rect {
        self.lanes[index]
    }
}

/// Hauteur relative de chaque piste, dans l'ordre de [`TRACKS`].
const LANE_WEIGHTS: [f32; TRACKS.len()] = [3.0, 2.0, 3.0, 2.0, 1.0, 1.0];

/// Plage de hauteurs des notes montrées d'une piste (au moins une octave).
fn pitch_range(bars: &[BarView], track: TrackId) -> Option<(u8, u8)> {
    let pitches = bars
        .iter()
        .filter_map(|bar| bar.content.as_ref())
        .flat_map(|content| content.notes.iter())
        .filter(|note| note.track == track)
        .map(|note| note.pitch);
    let (low, high) = pitches.fold(None, |range: Option<(u8, u8)>, pitch| match range {
        None => Some((pitch, pitch)),
        Some((low, high)) => Some((low.min(pitch), high.max(pitch))),
    })?;
    let missing = 12u8.saturating_sub(high - low);
    Some((low.saturating_sub(missing / 2), high.saturating_add(missing - missing / 2)))
}

fn paint_score(ui: &egui::Ui, layout: &ScoreLayout, score: &ScoreView, drag: Option<LoopDrag>) {
    let painter = ui.painter_at(layout.rect);
    painter.rect_filled(layout.rect, 0.0, Color32::from_gray(20));
    let small = FontId::proportional(11.0);
    let text = Color32::from_gray(200);
    let dim = Color32::from_gray(120);

    let ranges: Vec<Option<(u8, u8)>> =
        TRACKS.iter().map(|track| pitch_range(&score.bars, *track)).collect();

    // Étiquettes des pistes ; une piste sans note dans la fenêtre est grisée.
    for (index, track) in TRACKS.into_iter().enumerate() {
        let lane = layout.lane(index);
        let color = if ranges[index].is_some() { track_color(track) } else { dim };
        painter.text(
            Pos2::new(layout.rect.left() + 6.0, lane.center().y),
            Align2::LEFT_CENTER,
            track_label(track),
            small.clone(),
            color,
        );
    }
    for (label, row) in [("Tune", layout.tune), ("Form", layout.form), ("Chords", layout.chords)] {
        painter.text(
            Pos2::new(layout.rect.left() + 6.0, row.center().y),
            Align2::LEFT_CENTER,
            label,
            small.clone(),
            dim,
        );
    }

    // Les notes (liaisons, legato) ne débordent pas sur les étiquettes de gauche.
    let grid = painter.with_clip_rect(Rect::from_min_max(
        Pos2::new(layout.left, layout.rect.top()),
        layout.rect.max,
    ));
    let mut chord_marks: Vec<(f32, &str)> = Vec::new();
    let mut previous_key = None;
    let mut tune_labels = Vec::new();
    for (offset, bar) in score.bars.iter().enumerate() {
        let x = layout.bar_x(offset as f32);
        let column = layout.column(offset);
        let fill = match (&bar.content, bar.index.cmp(&score.current_bar)) {
            (_, std::cmp::Ordering::Equal) => Color32::from_rgb(44, 52, 68),
            (Some(_), std::cmp::Ordering::Greater) => Color32::from_rgb(30, 38, 50),
            (Some(_), std::cmp::Ordering::Less) => Color32::from_gray(26),
            (None, _) => Color32::from_gray(16),
        };
        painter.rect_filled(column, 0.0, fill);
        painter.line_segment(
            [column.left_top(), column.left_bottom()],
            Stroke::new(1.0_f32, Color32::from_gray(60)),
        );
        painter.text(
            Pos2::new(x + 3.0, layout.ruler.top() + 2.0),
            Align2::LEFT_TOP,
            bar.index.to_string(),
            small.clone(),
            if bar.index == score.current_bar { Color32::WHITE } else { dim },
        );
        let Some(content) = &bar.content else { continue };

        // Forme : couleur de la partie, étiquette au début de chaque passe ; air (rang dans le
        // set), tonalité et mode à chaque changement.
        if let Some(form) = &content.form {
            let cell = Rect::from_min_max(
                Pos2::new(x + 1.0, layout.form.top() + 2.0),
                Pos2::new(x + layout.bar_width - 1.0, layout.form.bottom() - 2.0),
            );
            let color = if form.intro {
                Color32::from_gray(70)
            } else {
                PART_COLORS[form.part % PART_COLORS.len()]
            };
            painter.rect_filled(cell, 2.0, color);
            if form.intro || form.bar_in_strain == 1 || offset == 0 {
                painter.text(
                    cell.left_center() + Vec2::new(3.0, 0.0),
                    Align2::LEFT_CENTER,
                    form.label.as_str(),
                    small.clone(),
                    text,
                );
            }
            let key = (form.tune_index, form.key_mode.as_str());
            if previous_key != Some(key) {
                previous_key = Some(key);
                let set = if form.tunes_in_set == 0 {
                    format!("tune {}", form.tune_in_set)
                } else {
                    format!("tune {}/{}", form.tune_in_set, form.tunes_in_set)
                };
                painter.line_segment(
                    [Pos2::new(x, layout.tune.top()), Pos2::new(x, layout.tune.bottom())],
                    Stroke::new(2.0_f32, text),
                );
                tune_labels.push((x, format!("{set} · {}", form.key_mode)));
            }
        }

        // Accords à leur pas.
        let steps = content.steps as f32;
        for chord in &content.chords {
            let cx = x + chord.step as f32 / steps * layout.bar_width;
            painter.line_segment(
                [
                    Pos2::new(cx, layout.chords.top() + 2.0),
                    Pos2::new(cx, layout.chords.bottom() - 2.0),
                ],
                Stroke::new(1.0_f32, Color32::from_gray(90)),
            );
            chord_marks.push((cx, chord.name.as_str()));
        }

        // Notes, piste par piste.
        for (lane_index, track) in TRACKS.into_iter().enumerate() {
            let Some((low, high)) = ranges[lane_index] else { continue };
            let lane = layout.lane(lane_index);
            let span = f32::from(high - low) + 1.0;
            let note_height = (lane.height() / span).clamp(2.0, 6.0);
            let color = track_color(track);
            for note in content.notes.iter().filter(|note| note.track == track) {
                let start = x + note.start_step as f32 / steps * layout.bar_width;
                let length = note.duration_steps as f32 / steps * layout.bar_width;
                let y = lane.bottom() - (f32::from(note.pitch - low) + 0.5) / span * lane.height();
                paint_note(&grid, note, start, length, y, note_height, color);
            }
        }
    }
    // Accords : un nom à chaque changement (une pédale tenue sur plusieurs mesures ne se répète
    // pas), coupé là où le suivant commence.
    let mut previous_name = None;
    let mut named: Vec<(f32, &str)> = Vec::new();
    for (cx, name) in chord_marks {
        if previous_name != Some(name) {
            previous_name = Some(name);
            named.push((cx, name));
        }
    }
    for (index, (cx, name)) in named.iter().enumerate() {
        let end = named.get(index + 1).map_or(layout.rect.right(), |(next, _)| *next);
        let clip = Rect::from_min_max(
            Pos2::new(*cx, layout.chords.top()),
            Pos2::new(end, layout.chords.bottom()),
        );
        painter.with_clip_rect(clip).text(
            Pos2::new(cx + 3.0, layout.chords.center().y),
            Align2::LEFT_CENTER,
            *name,
            small.clone(),
            text,
        );
    }
    // Les étiquettes d'air débordent sur les mesures suivantes : peintes après leurs fonds.
    for (x, label) in tune_labels {
        painter.text(
            Pos2::new(x + 4.0, layout.tune.center().y),
            Align2::LEFT_CENTER,
            label,
            small.clone(),
            text,
        );
    }

    // Boucle en place, puis boucle en préparation.
    let span_rect = |from: usize, to: usize| {
        let (from, to) = (from.min(to), from.max(to));
        let left = layout.bar_x(from.saturating_sub(score.first_bar) as f32).max(layout.left);
        let right =
            layout.bar_x((to + 1).saturating_sub(score.first_bar) as f32).min(layout.rect.right());
        Rect::from_min_max(
            Pos2::new(left, layout.rect.top()),
            Pos2::new(right, layout.rect.bottom()),
        )
    };
    if let Some((from, to)) = score.loop_range
        && to + 1 > score.first_bar
    {
        let rect = span_rect(from, to);
        painter.rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(240, 170, 60, 28));
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.5_f32, Color32::from_rgb(240, 170, 60)),
            StrokeKind::Inside,
        );
    }
    if let Some(drag) = drag {
        let length = drag.from.abs_diff(drag.to) + 1;
        let too_long = length > MEASURE_WINDOW_CAP;
        let color =
            if too_long { Color32::from_rgb(240, 80, 80) } else { Color32::from_rgb(240, 200, 90) };
        let rect = span_rect(drag.from, drag.to);
        painter.rect_filled(
            rect,
            0.0,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 36),
        );
        painter.rect_stroke(rect, 0.0, Stroke::new(1.5_f32, color), StrokeKind::Inside);
        let label = if too_long {
            format!("{length} bars: too long (max {MEASURE_WINDOW_CAP})")
        } else {
            format!("loop {length} bars")
        };
        painter.text(
            rect.center_top() + Vec2::new(0.0, 20.0),
            Align2::CENTER_TOP,
            label,
            FontId::proportional(13.0),
            color,
        );
    }

    // Tête de lecture.
    let x = layout.bar_x(score.playhead);
    painter.line_segment(
        [Pos2::new(x, layout.rect.top()), Pos2::new(x, layout.rect.bottom())],
        Stroke::new(2.0_f32, Color32::WHITE),
    );
}

/// Une note du piano-roll, selon son articulation publiée. Chaque variante a son dessin ; le
/// `match` est exhaustif, donc une articulation nouvelle ne compile pas sans le sien.
///
/// | Articulation | Dessin |
/// |---|---|
/// | `Normal` | barre pleine |
/// | `Accent` | barre pleine, cerclée de blanc |
/// | `Detached` | barre raccourcie (gate court) |
/// | `Linked` | barre pleine, bouts carrés (gate plein, rien ajouté) |
/// | `Tied` | barre aux bouts carrés, raccrochée à la note précédente par un trait : pas d'attaque |
/// | `Legato` | barre avec une attaque, un arc depuis la note précédente et un prolongement clair au-delà de sa fin (le chevauchement) |
/// | `Trigger` | losange (percussion, sans durée) |
fn paint_note(
    painter: &egui::Painter,
    note: &NoteView,
    start: f32,
    length: f32,
    y: f32,
    height: f32,
    color: Color32,
) {
    let alpha = 120 + note.velocity / 2;
    let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
    let bar = |width: f32| {
        Rect::from_min_size(Pos2::new(start, y - height / 2.0), Vec2::new(width.max(2.0), height))
    };
    let stroke = Stroke::new(1.5_f32, color);
    match note.articulation {
        Articulation::Normal => {
            painter.rect_filled(bar(length), 1.5, fill);
        }
        Articulation::Accent => {
            let rect = bar(length);
            painter.rect_filled(rect, 1.5, fill);
            painter.rect_stroke(
                rect,
                1.5,
                Stroke::new(1.0_f32, Color32::WHITE),
                StrokeKind::Outside,
            );
        }
        Articulation::Detached => {
            painter.rect_filled(bar(length * 0.55), 1.5, fill);
        }
        Articulation::Linked => {
            painter.rect_filled(bar(length), 0.0, fill);
        }
        Articulation::Tied => {
            painter.rect_filled(bar(length), 0.0, fill);
            painter.line_segment([Pos2::new(start - TIE_REACH, y), Pos2::new(start, y)], stroke);
        }
        Articulation::Legato => {
            let rect = bar(length);
            painter.rect_filled(rect, 1.5, fill);
            // Le chevauchement : la note tenue du canal se prolonge un peu sous la suivante.
            let tail = Rect::from_min_size(
                Pos2::new(rect.right(), rect.top()),
                Vec2::new(LEGATO_OVERLAP, height),
            );
            painter.rect_filled(tail, 1.0, fill.gamma_multiply(0.45));
            // L'arc de liaison, de la fin de la note précédente à cette attaque.
            let arc: Vec<Pos2> = (0..=6)
                .map(|i| {
                    let t = i as f32 / 6.0;
                    Pos2::new(
                        start - TIE_REACH * (1.0 - t),
                        y - height / 2.0 - 1.0 - 4.0 * (std::f32::consts::PI * t).sin(),
                    )
                })
                .collect();
            painter.add(egui::Shape::line(arc, stroke));
        }
        Articulation::Trigger => {
            let c = Pos2::new(start + 2.0, y);
            let r = (height / 2.0).max(2.5);
            painter.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(c.x, c.y - r),
                    Pos2::new(c.x + r, c.y),
                    Pos2::new(c.x, c.y + r),
                    Pos2::new(c.x - r, c.y),
                ],
                fill,
                Stroke::NONE,
            ));
        }
    }
}

/// Longueur du trait d'une liaison ou d'un legato vers la note précédente, en points.
const TIE_REACH: f32 = 5.0;
/// Prolongement dessiné d'une note legato au-delà de sa fin, en points.
const LEGATO_OVERLAP: f32 = 3.0;

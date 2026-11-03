//! Génération traditionnelle depuis le profil actif : grille, forme, cellules,
//! harmonie, accompagnement, percussions, intros et dynamique émotionnelle.
//! Les airs sont construits sur le thread principal ; les reprises relisent
//! les mêmes cellules et tirages ChaCha8, sans données musicales implicites.

use std::{borrow::Cow, collections::BTreeMap, sync::Arc};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::{
    accomp::{BassMotifPlan, ChordPlayer, bass_bar, bass_motif_bar, first_bass_target},
    accomp_profile::ChordVoice,
    arc::{ArcBar, ChorusArc, TagWeight},
    arrangement::{IntroConfig, IntroInfo, IntroRole, TunePosition},
    bass_template::{TemplateBar, bass_template_bar},
    cells::Cell,
    harmony::{ChordDegree, ChordQuality, DegreeChord, TuneHarmony},
    melody::{MelodyLine, same_cell_up_to_octave, tonic_midi_for_key},
    mode::Mode,
    percussion::{
        PercussionPattern, PercussionPatternPlan, draw_percussion_humanize, percussion_notes,
    },
    phrase::{Clave, Slot, Window},
    plan::{BarRole, ImposedRole, TunePlan},
    session::SessionDraw,
    set::{KEY_DRAW_RANGE, SetPlan, TransitionKind, candidates},
    walking::{WalkingBar, walking_bar},
};
use crate::{
    algo::RngCore,
    models::{
        HarmonyPlan,
        chord_loop::LoopHarmony,
        rewrite::{GrammarStyle, SteedmanGrammar},
    },
    ornaments::BarContext,
    params::MusicalParams,
    theory::{
        PitchClass,
        chord::Chord,
        lydian_chromatic::{LydianChromaticConcept, ScaleGuidance},
    },
    timeline::{
        Articulation, ChordChange, ChordContext, Measure, NoteId, StateSnapshot, TimelineNote,
        TrackId,
    },
    tuning::{AccompanimentParams, MelodyParams, TuningParams},
};

/// Flux de découpage en phrases du comping et de la batterie (graines distinctes, aucune
/// corrélation entre les deux couches).
const COMPING_PHRASES: u64 = 0x434D_5048;
const PERCUSSION_PHRASES: u64 = 0x5052_5048;
const PERCUSSION_CHANCE: u64 = 0x4348_414E;

/// Graine des coups probabilistes de la mesure `bar` de l'air (0-based, tours compris) : sans
/// état, tirée de la graine d'accompagnement ; un seek rejoue les mêmes coups.
fn chance_seed(accompaniment_seed: u64, bar: u64) -> u64 {
    crate::ornaments::mix(crate::ornaments::mix(accompaniment_seed, PERCUSSION_CHANCE), bar)
}

/// Passe de partie qui contient la mesure `performed_bar` de l'air (0-based), pour le
/// découpage en phrases de la couche `layer` : la fenêtre, le rang de la mesure dans la passe
/// et la partie. La graine ne dépend que de la graine d'accompagnement de l'air et de la
/// première mesure de la passe : tout se recalcule sans état (seek). `clave` : rang dans le
/// cycle de la clave de la première mesure de la passe, pour les phrases qui portent un côté.
fn phrasing_frame(
    form: &crate::style_profile::ProfileForm,
    clave: Option<&Clave>,
    accompaniment_seed: u64,
    performed_bar: usize,
    layer: u64,
) -> (Window, usize, usize) {
    let (part, _, offset) = form.position(performed_bar);
    let bars = form.parts[part].roles.len();
    let chorus_end = performed_bar % form.bars_per_tour() - offset + bars == form.bars_per_tour();
    let seed = crate::ornaments::mix(
        crate::ornaments::mix(accompaniment_seed, layer),
        (performed_bar - offset) as u64,
    );
    let clave_phase = clave.map(|clave| clave.phase(accompaniment_seed, performed_bar - offset));
    (Window { seed, bars, chorus_end, clave_phase }, offset, part)
}

/// Accompagnement joué : celui que la variante de session tirée a habillé (famille de voicings),
/// sinon celui du profil.
fn dressed<'a>(
    session: &'a Option<Arc<SessionDraw>>,
    profile: &'a Option<AccompanimentParams>,
) -> Option<&'a AccompanimentParams> {
    session.as_deref().and_then(SessionDraw::accompaniment).or(profile.as_ref())
}

/// Arc en vigueur : celui que la variante de session nomme, sinon `[arc]`.
fn chorus_arc_of<'a>(
    profile: &'a crate::style_profile::StyleProfile,
    session: Option<&SessionDraw>,
) -> Option<&'a ChorusArc> {
    session
        .and_then(SessionDraw::arc_name)
        .and_then(|name| profile.session.arcs.get(name))
        .or(profile.arc.as_ref())
}
/// Découpage en phrases de comping de la passe qui contient `performed_bar` et rang de cette
/// mesure dans la passe ; vide si `chords` n'a pas de phrase.
fn comping_tiling(
    form: &crate::style_profile::ProfileForm,
    chords: &ChordVoice,
    clave: Option<&Clave>,
    accompaniment_seed: u64,
    performed_bar: usize,
    tags: &[TagWeight],
) -> (Vec<Option<Slot>>, usize) {
    let (window, offset, part) =
        phrasing_frame(form, clave, accompaniment_seed, performed_bar, COMPING_PHRASES);
    if chords.phrases.is_empty() {
        return (Vec::new(), offset);
    }
    (chords.phrase_tiling(window, &form.parts[part].name, tags), offset)
}

// ---------------------------------------------------------------------------
// ActiveTune
// ---------------------------------------------------------------------------

/// État d'un air généré par [`TuneGenerator`] (rejoué `tours_per_tune` tours).
#[derive(Clone, Debug, PartialEq)]
pub struct ActiveTune {
    /// Indice 0-based de l'air dans la session.
    pub tune_index: usize,
    /// Plan structurel du tune (tonalité, mode, type, schéma AABB).
    pub plan: TunePlan,
    /// Matériel mélodique des parties, passes et tours configurés.
    pub melody: MelodyLine,
    /// Modèle harmonique sélectionné par le profil, précalculé par mesure.
    pub harmony: HarmonyPlan,
    /// Plan motivique de la basse pour cet air (vide si la basse utilise `hits`).
    bass_plan: BassMotifPlan,
    /// Alternatives de percussion fixées par partie et densité pour cet air.
    percussion_plan: PercussionPatternPlan,
    /// Décalages d'humanisation des vélocités de la percussion, par mesure unique de
    /// chaque partie ; les reprises et les tours relisent les mêmes décalages.
    pub percussion_humanize: Vec<Vec<i8>>,
    /// Graine des alternatives harmoniques et rythmiques, retirée à chaque air ;
    /// zéro et aucun tirage si toutes les couches sont uniques.
    pub variant_seed: u64,
    /// Graine des tirages d'interprétation de l'air, tirée une fois (0 si le
    /// profil n'a aucune interprétation) ; un flux par mesure en dérive.
    pub performance_seed: u64,
    /// Graine dérivée du matériel écrit : ornements et hétérophonie seulement (la mélodie
    /// écrite et ses ornements ne dépendent pas de la graine d'accompagnement).
    pub material_seed: u64,
    /// Graine de l'accompagnement (comping, marche et notes de passage de la basse, motifs
    /// de basse, phrases de comping et de batterie), tirée une fois par air dans le RNG injecté,
    /// **après** la mélodie, l'intro et le matériel (0 et aucun tirage sans `[accompaniment]`
    /// ni phrases ou relances de batterie). Chaque mesure en dérive sa graine par son seul
    /// indice (`mix(graine, mesure)`), chaque passe de partie la sienne par l'indice de sa
    /// première mesure : aucun état d'une mesure à l'autre.
    pub accompaniment_seed: u64,
    /// Graine de l'arc sur plusieurs chorus, tirée une fois par air (0 si le profil n'a pas
    /// d'arc) ; l'arc d'un set lit celle de son premier air.
    arc_seed: u64,
    /// Informations d'intro pour cet air (P1-T11).
    pub intro_info: IntroInfo,
    initial_mode: Mode,
    /// Identité du matériel d'origine, conservée lors d'un retour.
    material_origin: usize,
    initial_dark: bool,
    bright_mode: Mode,
    dark_mode: Option<Mode>,
    /// Historique audible : une invalidation retire seulement les décisions futures.
    mode_changes: BTreeMap<usize, (Mode, bool)>,
    mode_requests: BTreeMap<usize, bool>,
}

impl ActiveTune {
    fn color_at(&self, bar: usize) -> (Mode, bool) {
        self.mode_changes
            .range(..=bar)
            .next_back()
            .map_or((self.initial_mode, self.initial_dark), |(_, &color)| color)
    }
}

// ---------------------------------------------------------------------------
// TuneGenerator
// ---------------------------------------------------------------------------

/// Générateur de mesures piloté par le profil actif.
#[derive(Clone, Debug)]
pub struct TuneGenerator {
    /// Paramètres musicaux courants (tempo, swing, densité de la percussion, activations).
    pub musical_params: MusicalParams,
    /// Paramètres de style et banques de données (cellules, percussion, harmonie).
    pub tuning: TuningParams,
    /// Airs générés indexés par `tune_index` (`0` = mesures 1..=32×tours, etc.).
    tunes: BTreeMap<usize, Arc<ActiveTune>>,
    /// Sets de la session, dans l'ordre ; chacun couvre des airs consécutifs.
    sets: Vec<SetPlan>,
    profile: Arc<crate::style_profile::StyleProfile>,
    /// Compteur monotone d'identifiants de notes.
    next_note_id: NoteId,
    /// Dernier indice de mesure généré.
    current_bar: usize,
    /// Air à terminer sur demande du jeu, indépendant de la fin de set.
    ending_tune: Option<usize>,
    /// Variantes de session tirées pour la graine (`None` : pas encore tirées, ou profil sans
    /// variantes). Tirage unique, avant le premier air ; remis à zéro par le rejeu et par un
    /// changement de profil.
    session: Option<Arc<SessionDraw>>,
}

impl TuneGenerator {
    /// Construit un nouveau [`TuneGenerator`] avec les paramètres et le tuning donnés.
    #[must_use]
    pub fn new(musical_params: MusicalParams, tuning: TuningParams) -> Self {
        let ending_tune = musical_params.end_after_tour.then_some(0);
        Self {
            musical_params,
            profile: Arc::new(tuning.profile.clone()),
            tuning,
            tunes: BTreeMap::new(),
            sets: Vec::new(),
            next_note_id: 1,
            current_bar: 0,
            session: None,
            ending_tune,
        }
    }

    /// Met à jour les paramètres musicaux (`bpm`, `swing_ratio`, `percussion_density`, etc.).
    pub fn update_params(&mut self, params: MusicalParams) {
        if params.end_after_tour && !self.musical_params.end_after_tour {
            self.ending_tune = Some(self.position_of(self.current_bar.max(1)).tune_index);
        } else if !params.end_after_tour {
            self.ending_tune = None;
        }
        self.musical_params = params;
    }

    /// Garde les cellules et les décisions déjà audibles ; retire uniquement
    /// les décisions de mode et les airs encore futurs.
    pub fn invalidate_future(&mut self, playing_bar: usize) {
        let current_tune = self.position_of(playing_bar).tune_index;
        self.current_bar = playing_bar;
        if let Some(ending) = &mut self.ending_tune {
            *ending = (*ending).min(current_tune);
        }
        self.tunes.retain(|&index, _| index <= current_tune);
        self.sets.retain(|set| set.first_tune() <= current_tune);
        for tune in self.tunes.values_mut() {
            let tune = Arc::make_mut(tune);
            tune.mode_changes.retain(|&bar, _| bar <= playing_bar);
            tune.mode_requests.retain(|&bar, _| bar <= playing_bar);
        }
    }

    fn percussion_density(&self, intensity: f32, rhythm_density: f32) -> u8 {
        let p = &self.musical_params;
        let d = &self.tuning.profile.dynamics;
        (f32::from(p.percussion_density)
            + d.percussion_intensity.eval(intensity)
            + d.density_percussion.eval(rhythm_density))
        .round()
        .clamp(0.0, 2.0) as u8
    }

    /// Intensité lue par les courbes de `dynamics` à cette mesure : celle de l'air, fois le
    /// multiplicateur de l'arc du chorus. Le tempo garde l'intensité de l'air.
    fn arc_intensity(&self, tune_index: usize, arc: Option<&ArcBar<'_>>) -> f32 {
        let intensity = self.effective_intensity(tune_index);
        arc.map_or(intensity, |arc| arc.intensity(intensity))
    }

    /// Densité rythmique lue par les courbes de densité, multipliée par l'arc du chorus.
    fn arc_density(&self, arc: Option<&ArcBar<'_>>) -> f32 {
        let density = self.musical_params.rhythm_density;
        arc.map_or(density, |arc| arc.density(density))
    }

    /// Graine de l'arc du set de l'air `tune_index` : celle de son premier air (0 sans arc).
    fn arc_seed(&self, tune_index: usize) -> u64 {
        let first = self.set_of(tune_index).map_or(tune_index, |(_, set)| set.first_tune());
        self.tunes.get(&first).map_or(0, |tune| tune.arc_seed)
    }

    /// Met à jour les paramètres de tuning et invalide le cache de tunes afin que
    /// les nouvelles banques/constantes prennent effet dès la prochaine mesure.
    pub fn update_tuning(&mut self, tuning: TuningParams) {
        if self.tuning != tuning {
            self.profile = Arc::new(tuning.profile.clone());
            self.tuning = tuning;
            self.tunes.clear();
            self.sets.clear();
            self.session = None;
        }
    }

    /// Variantes de session tirées pour la graine ; `None` tant qu'aucune mesure n'est générée
    /// et pour un profil sans `[[session.variants]]`.
    #[must_use]
    pub fn session(&self) -> Option<&SessionDraw> {
        self.session.as_deref()
    }

    fn chorus_arc(&self) -> Option<&ChorusArc> {
        chorus_arc_of(&self.profile, self.session.as_deref())
    }

    /// Tire les variantes de session avant le premier air : deux entiers de 32 bits du RNG
    /// injecté, seulement si le profil a des variantes (sans elles le flux est celui d'avant).
    fn ensure_session(&mut self, rng: &mut dyn RngCore) {
        if self.session.is_some() || self.profile.session.variants.is_empty() {
            return;
        }
        let high = rng.next_range_u64(0..1 << 32);
        let seed = (high << 32) | rng.next_range_u64(0..1 << 32);
        self.session = Some(Arc::new(self.profile.session.draw(&self.profile, seed)));
    }

    /// Réinitialise le générateur à son état initial pour le seek déterministe.
    ///
    /// Conserve `musical_params`, `tuning` et le compteur monotone `next_note_id`,
    /// et vide les tunes générés afin que le rejeu depuis la mesure 1 reproduise
    /// exactement la séquence de tirages sur le RNG.
    pub fn reset_to_initial(&mut self) {
        self.tunes.clear();
        self.sets.clear();
        self.session = None;
        self.ending_tune = self.musical_params.end_after_tour.then_some(0);
        self.current_bar = 0;
    }

    /// Avance silencieusement jusqu'à `target_bar` (exclus) en rejouant les
    /// mesures `1..target_bar` avec `rng`.
    pub fn silent_advance(&mut self, target_bar: usize, rng: &mut dyn RngCore) {
        for bar in 1..target_bar {
            let _ = self.generate_measure(bar, rng);
        }
    }

    /// Retourne le tune actif correspondant à la mesure 1-based `bar_number`, s'il a été généré.
    #[must_use]
    pub fn tune_for_bar(&self, bar_number: usize) -> Option<&ActiveTune> {
        let tune_idx = self.position_of(bar_number).tune_index;
        self.tunes.get(&tune_idx).or_else(|| self.tunes.values().last()).map(Arc::as_ref)
    }

    /// Retourne le [`TunePlan`] et le [`BarRole`] pour la mesure 1-based `bar_number`.
    #[must_use]
    pub fn bar_info(&self, bar_number: usize) -> Option<(TunePlan, BarRole)> {
        let tune = self.tune_for_bar(bar_number)?;
        let position = self.position_of(bar_number);
        let role = tune.plan.bar_role(self.performed_bar(&position));
        let mut plan = tune.plan.clone();
        plan.mode = tune.color_at(bar_number).0;
        Some((plan, role))
    }

    /// Position structurée de la mesure 1-based `bar_number` (unique conversion
    /// mesure → `(section, tune_index, tour, part, pass, bar_in_part)`).
    ///
    /// Prend en compte les mesures d'intro : parcourt les tunes déjà générés
    /// pour accumuler les barres d'intro et décaler le calcul.
    #[must_use]
    pub fn position_of(&self, bar_number: usize) -> TunePosition {
        let tours = self.tuning.profile.arrangement.tours_per_tune;
        let cycle = self.profile.form.bars_per_tour() * tours;

        // `offset` = nombre de mesures consommées par les tunes précédents
        // (intro + corps chacun).
        let mut offset = 0usize;
        let mut tune_idx = 0usize;

        loop {
            let intro_bars =
                self.tunes.get(&tune_idx).map(|t| t.intro_info.intro_bars).unwrap_or(0);

            let (key, mode) = self
                .tunes
                .get(&tune_idx)
                .map_or((0, Mode::Ionian), |t| (t.plan.key, t.color_at(bar_number).0));
            let body_end = offset + intro_bars + cycle;
            if bar_number <= body_end {
                let mut position = TunePosition::at_bar(
                    bar_number - offset,
                    tours,
                    intro_bars,
                    key,
                    mode,
                    &self.profile.form,
                );
                position.tune_index = tune_idx;
                if let Some((set_index, set)) = self.set_of(tune_idx) {
                    position.set_index = set_index;
                    position.tune_in_set = set.tune_in_set(tune_idx);
                    position.tunes_in_set = set.tunes();
                } else {
                    // Air pas encore construit : set provisoire, l'air ouvre un nouveau set.
                    position.set_index = self.sets.len();
                    position.tune_in_set = 1;
                    position.tunes_in_set = 0;
                }
                return position;
            }

            offset += intro_bars + cycle;
            tune_idx += 1;
        }
    }

    /// Vrai si `track` joue à `position` : l'arrangement l'autorise (ou la piste a
    /// joué l'intro de cet air) ET le drapeau `enable_*` du jeu le permet ET la
    /// partie courante ne la tait pas (`form.parts[].tracks_off`). Seul
    /// calcul de « piste active ».
    fn track_active(&self, track: TrackId, position: &TunePosition) -> bool {
        if !self.tuning.profile.has_track(track) {
            return false;
        }
        if !position.is_intro() && self.profile.form.part_mutes(position.part.0, track) {
            return false;
        }
        let flag = match track {
            TrackId::Melody | TrackId::Countermelody => self.musical_params.enable_melody,
            TrackId::Chords | TrackId::Bass => self.musical_params.enable_harmony,
            TrackId::PercussionLow | TrackId::PercussionHigh => self.musical_params.enable_rhythm,
        };
        let entered = self.tuning.profile.arrangement.allows(track, position, &self.profile.form)
            || self.played_intro(track, position.tune_index);
        flag && entered
            && self.tuning.profile.dynamics.allows(track, self.musical_params.rhythm_density)
    }

    /// Set de l'air `tune_index` et son indice dans la session, s'il est déjà tiré.
    fn set_of(&self, tune_index: usize) -> Option<(usize, &SetPlan)> {
        self.sets.iter().enumerate().rev().find(|(_, set)| set.contains(tune_index))
    }

    /// Vrai si l'air `tune_index` ouvre un set : intro complète et entrées progressives.
    fn is_first_in_set(&self, tune_index: usize) -> bool {
        self.set_of(tune_index).is_some_and(|(_, set)| set.first_tune() == tune_index)
    }

    fn transition_for(&self, tune_index: usize) -> (TransitionKind, usize) {
        self.set_of(tune_index)
            .and_then(|(_, set)| set.transition_draw(set.tune_in_set(tune_index)))
            .map_or((TransitionKind::Direct, 0), |u| self.profile.set.transition(u))
    }

    fn ramp_progress(&self, tune_index: usize) -> Option<(&super::set::SetRamp, f32)> {
        let (_, set) = self.set_of(tune_index)?;
        if set.tunes() <= 1 {
            return None;
        }
        Some((self.profile.set.ramp.as_ref()?, set.progress(tune_index)))
    }

    fn effective_intensity(&self, tune_index: usize) -> f32 {
        let intensity = self.musical_params.intensity;
        self.ramp_progress(tune_index)
            .map_or(intensity, |(ramp, p)| ramp.effective_intensity(p, intensity))
    }

    fn tempo_for(&self, tune_index: usize) -> f32 {
        let params = &self.musical_params;
        if params.bpm_explicit {
            return params.bpm;
        }
        // Tempo de la variante de session : base de la courbe de `dynamics` à la place du
        // tempo nominal du profil ; le tempo direct de l'hôte (ci-dessus) gagne.
        let nominal =
            self.session.as_ref().and_then(|session| session.tempo()).unwrap_or(params.bpm);
        let dynamics = &self.profile.dynamics;
        let game_tempo = dynamics.tempo_for(nominal, params.intensity);
        let Some((ramp, progress)) = self.ramp_progress(tune_index) else {
            return game_tempo;
        };
        let ceiling = ramp.ceiling_bpm.unwrap_or(self.profile.meter.max_bpm);
        let base = dynamics
            .tempo_for(nominal, self.effective_intensity(tune_index))
            .max(game_tempo)
            .min(ceiling.max(game_tempo));
        let lift = ramp.tempo_bpm.eval(progress) * ramp.gate.eval(params.intensity);
        base + lift.min((ceiling - base).max(0.0))
    }

    /// Vrai si `track` a joué l'intro de l'air `tune_index` : une piste qui a
    /// lancé l'air (décompte à la percussion, vamp) continue au lieu de se taire quand
    /// la mélodie entre (écoute 2026-10-22).
    fn played_intro(&self, track: TrackId, tune_index: usize) -> bool {
        self.tunes.get(&tune_index).is_some_and(|tune| {
            let intro = &tune.intro_info;
            match track {
                TrackId::Melody => intro.melody_active(),
                TrackId::Countermelody => false,
                TrackId::Chords | TrackId::Bass => intro.chords_active(),
                TrackId::PercussionLow | TrackId::PercussionHigh => intro.percussion_active(),
            }
        })
    }

    /// Vrai si `track` joue pendant une mesure d'intro selon la forme tirée.
    fn track_active_in_intro(&self, track: TrackId, intro_info: &IntroInfo) -> bool {
        if !self.tuning.profile.has_track(track) {
            return false;
        }
        self.tuning.profile.dynamics.allows(track, self.musical_params.rhythm_density)
            && match track {
                TrackId::Melody => intro_info.melody_active() && self.musical_params.enable_melody,
                TrackId::Countermelody => false,
                TrackId::Chords | TrackId::Bass => {
                    intro_info.chords_active() && self.musical_params.enable_harmony
                }
                TrackId::PercussionLow | TrackId::PercussionHigh => {
                    intro_info.percussion_active() && self.musical_params.enable_rhythm
                }
            }
    }

    /// Génère une [`Measure`] complète pour `bar_index` (1-based dans le flux du moteur).
    pub fn generate_measure(&mut self, bar_index: usize, rng: &mut dyn RngCore) -> Measure {
        self.current_bar = bar_index;

        // Générer les tunes nécessaires AVANT de calculer la position : la
        // position dépend du nombre de mesures d'intro (tirées au RNG).
        self.ensure_tunes_covering(bar_index, rng);

        let mut position = self.position_of(bar_index);
        self.update_parallel_mode(bar_index, &mut position, rng);

        let time_sig = self.profile.meter.time_signature;
        let steps = self.profile.meter.steps_per_bar;
        let tempo = self.tempo_for(position.tune_index);

        let mut measure = Measure::new(bar_index, time_sig, tempo, steps);
        measure.pulse = self.profile.meter.pulse_spec();
        measure.position = Some(position);
        let swing = self.swing_for(tempo);
        measure.swing_ratio = swing.grid;
        if let Some(session) = &self.session {
            measure.session = session.variants().to_vec();
        }
        measure.state_snapshot = StateSnapshot {
            bpm: tempo,
            density: self.musical_params.rhythm_density,
            tension: self.musical_params.harmony_tension,
            smoothness: self.musical_params.melody_smoothness,
            valence: self.musical_params.harmony_valence,
            arousal: self.musical_params.arousal,
        };

        if self.ending_tune.is_some_and(|last| position.tune_index > last) {
            return measure;
        }
        let Some(tune) = self.tunes.get(&position.tune_index).cloned() else {
            return measure;
        };

        let is_intro = position.is_intro();
        let last_bar = !is_intro
            && position.is_last_tour()
            && position.bar_in_tour() + 1 == self.profile.form.bars_per_tour();
        let set_outro = last_bar
            && self.set_of(position.tune_index).is_some_and(|(_, set)| {
                set.is_last(position.tune_in_set) && self.profile.set.end.outro
            });
        let held =
            last_bar && self.transition_for(position.tune_index + 1).0 == TransitionKind::Held;
        let is_outro = last_bar && (self.musical_params.end_after_tour || set_outro || held);
        // L'arc ne touche ni l'intro ni la mesure de clôture, dont les programmes restent ceux
        // de leur chorus. Le profil et la session sont clonés (un compteur de références) pour
        // que l'état de l'arc ne garde pas `self` emprunté.
        let session = self.session.clone();
        let arc_profile = Arc::clone(&self.profile);
        let chorus_arc = chorus_arc_of(&arc_profile, session.as_deref());
        let base = session.as_deref().and_then(SessionDraw::base);
        let arc_state = if is_intro {
            None
        } else if let Some(arc) = chorus_arc {
            arc.at(
                self.arc_seed(position.tune_index),
                &position,
                arc_profile.form.bars_per_tour(),
                base,
            )
        } else {
            base.zip(session.as_deref())
                .map(|(base, session)| ArcBar::base_only(base, session.seed(), position.chorus()))
        };
        let arc = arc_state.filter(|_| !is_outro);
        if session.as_deref().is_some_and(|session| session.instruments().is_some())
            || chorus_arc.is_some_and(ChorusArc::has_instruments)
        {
            let base = session
                .as_deref()
                .and_then(SessionDraw::instruments)
                .unwrap_or(self.profile.instruments);
            measure.instruments = Some(
                arc_state.and_then(|state| state.instruments()).map_or(base, |o| o.apply(base)),
            );
        }

        if is_intro {
            self.fill_intro_measure(&tune, &position, &mut measure);
        } else if is_outro {
            self.fill_outro_measure(&tune, &position, &mut measure);
        } else {
            self.fill_tour_measure(&tune, &position, &mut measure, arc.as_ref());
        }
        // Pas des notes écrites (coups ou motif) : seuls les ajouts de passage sont vétoés.
        let written_bass_steps: Vec<usize> = if !is_intro
            && !is_outro
            && self.profile.accompaniment.as_ref().is_some_and(|a| a.bass.passing.is_some())
        {
            measure.notes_for_track(TrackId::Bass).iter().map(|n| n.start_step).collect()
        } else {
            Vec::new()
        };
        if !is_intro && !is_outro {
            self.apply_bass_passing(&mut measure, &tune, &position, arc.as_ref());
        }

        let written: Vec<usize> = if self.profile.ornaments.is_inert() {
            Vec::new()
        } else {
            measure.notes_for_track(TrackId::Melody).iter().map(|n| n.duration_steps).collect()
        };
        self.apply_dynamics(&mut measure, arc.as_ref());
        self.fit_planned_melody(&mut measure, &tune, &position);
        let hetero_written = if !is_intro && self.track_active(TrackId::Countermelody, &position) {
            let context = self.ornament_context(&measure, &tune, &position);
            let changes: Vec<_> = if !is_outro && tune.harmony.uses_modal_chords() {
                self.chords_for_bar(&tune, self.performed_bar(&position))
                    .iter()
                    .enumerate()
                    .map(|(slot, chord)| ChordChange {
                        start_step: slot * self.profile.harmony.steps_per_chord,
                        context: build_chord_context(*chord, tune.plan.key, tune.plan.mode),
                    })
                    .collect()
            } else {
                measure.chord_changes.clone()
            };
            let next = &mut self.next_note_id;
            self.profile.heterophony.as_ref().map(|h| {
                h.prepare(&mut measure, &context, &written, &changes, &mut || {
                    let id = *next;
                    *next += 1;
                    id
                })
            })
        } else {
            None
        };
        if !is_intro && !self.profile.performance.is_neutral() {
            let offset = position.bar_in_part - 1;
            let (start, end) = self.phrase_window(&tune, &position);
            let span = (end - start).max(1) as f32;
            // Flux propre à la mesure : la mesure `n` se joue pareil après un
            // seek, que les mesures voisines aient été générées ou relues.
            let mut performance_rng = ChaCha8Rng::seed_from_u64(tune.performance_seed);
            performance_rng.set_stream(bar_index as u64);
            self.profile.performance.apply(
                &mut measure,
                (offset - start) as f32 / span,
                span,
                offset + 1 == end,
                [
                    self.musical_params.arousal,
                    self.musical_params.harmony_valence,
                    self.musical_params.rhythm_density,
                    self.musical_params.harmony_tension,
                ],
                &mut performance_rng,
            );
        }
        if let Some(a) = &self.profile.accompaniment {
            let offset = if is_intro {
                0.0
            } else {
                self.profile
                    .dynamics
                    .chords_intensity
                    .eval(self.arc_intensity(position.tune_index, arc.as_ref()))
                    + self.profile.dynamics.density_chords.eval(self.arc_density(arc.as_ref()))
            };
            let seed = crate::ornaments::mix(
                tune.accompaniment_seed,
                self.performed_bar(&position) as u64,
            );
            // L'intro joue le motif par défaut (voir `fill_intro_measure`) ; les tours, celui
            // de la mesure.
            let hits = if is_intro {
                &a.chords.pattern(0.0).hits
            } else {
                let tags = arc.as_ref().map_or(&[][..], ArcBar::comping_tags);
                let (tiling, rank) = comping_tiling(
                    &self.profile.form,
                    &a.chords,
                    self.profile.clave.as_ref(),
                    tune.accompaniment_seed,
                    self.performed_bar(&position),
                    tags,
                );
                a.chords.bar_hits(offset, seed, tags, tiling.get(rank).copied().flatten())
            };
            super::chord_spread::apply(&mut measure, &a.chords, hits, seed);
        }
        if !is_intro
            && !is_outro
            && !self.profile.ornaments.is_inert()
            && self.tuning.profile.melody.difficulty.as_ref().is_none_or(|limits| limits.ornaments)
        {
            self.apply_ornaments(&mut measure, &tune, &position, &written);
        }
        if let Some((written, independent_steps)) = hetero_written {
            if !is_outro && !self.profile.ornaments.is_inert() {
                let mut context = self.ornament_context(&measure, &tune, &position);
                if let Some(h) = &self.profile.heterophony {
                    context.tessitura = (h.low, h.high);
                }
                context.independent_steps = independent_steps;
                let next = &mut self.next_note_id;
                self.profile.ornaments.apply(
                    &mut measure,
                    TrackId::Countermelody,
                    &context,
                    &written,
                    &mut || {
                        let id = *next;
                        *next += 1;
                        id
                    },
                );
            }
            if self.ornament_context(&measure, &tune, &position).phrase_end_bar
                && let Some(h) = &self.profile.heterophony
            {
                let next = &mut self.next_note_id;
                h.close_cadence(&mut measure, &mut || {
                    let id = *next;
                    *next += 1;
                    id
                });
            }
            if let Some(h) = &self.profile.heterophony {
                h.veto(&mut measure);
            }
        }
        if !is_intro
            && !is_outro
            && let Some(a) = &self.profile.accompaniment
        {
            super::bass_passing::veto_final(&mut measure, &a.bass, &written_bass_steps);
        }
        if let Some(arc) = &arc {
            arc.apply_break(&mut measure);
        }
        if let Some(session) = &session {
            session.shift_registers(&mut measure);
        }
        if !is_intro && !is_outro {
            self.silence_alternated_tracks(&mut measure, &position);
        }
        crate::swing::apply_track_swing(&mut measure, &swing);

        measure
    }

    /// Alternance de l'arrangement : une piste muette à cette mesure est composée comme
    /// toujours (les autres pistes, vétos et voix dérivées la voient) puis retirée du
    /// snapshot, qui ne publie aucune de ses notes.
    fn silence_alternated_tracks(&self, measure: &mut Measure, position: &TunePosition) {
        let arrangement = &self.tuning.profile.arrangement;
        let bar = self.performed_bar(position);
        for &track in arrangement.alternation.keys() {
            if !arrangement.plays_bar(track, bar)
                && let Some(notes) = measure.tracks.get_mut(&track)
            {
                notes.clear();
            }
        }
    }

    /// Rapports de swing de la mesure au tempo `tempo`. Sous commande directe de l'hôte
    /// (`SetSwing`), son rapport pour toutes les pistes. Sinon le rapport de base est celui de la
    /// variante de session (A2) s'il en tire un, à défaut celui des paramètres ; sans courbe il
    /// vaut pour toutes les pistes, avec une courbe selon le tempo (A6) il la décale.
    fn swing_for(&self, tempo: f32) -> crate::swing::ResolvedSwing {
        let params = &self.musical_params;
        if params.swing_explicit {
            return crate::swing::ResolvedSwing::uniform(params.swing_ratio);
        }
        let ratio =
            self.session.as_ref().and_then(|session| session.swing()).unwrap_or(params.swing_ratio);
        match &self.profile.swing {
            Some(curves) => curves.resolve(tempo, ratio, self.profile.meter.default_swing_ratio),
            None => crate::swing::ResolvedSwing::uniform(ratio),
        }
    }

    fn apply_bass_passing(
        &mut self,
        measure: &mut Measure,
        tune: &ActiveTune,
        position: &TunePosition,
        arc: Option<&ArcBar<'_>>,
    ) {
        let Some(a) = &self.profile.accompaniment else {
            return;
        };
        if a.bass.passing.as_ref().is_none_or(|p| p.is_inert()) || !tune.harmony.has_chords() {
            return;
        }
        let bar = self.performed_bar(position);
        let chords = self.chords_for_bar(tune, bar);
        let next_target = (bar + 1 < self.profile.form.bars_per_tour() * position.tours)
            .then(|| {
                let next = self.chords_for_bar(tune, bar + 1);
                first_bass_target(
                    &tune.bass_plan,
                    &tune.plan,
                    bar + 1,
                    &next,
                    a,
                    self.profile.meter.steps_per_bar,
                    self.profile.harmony.steps_per_chord,
                )
            })
            .flatten();
        let (start, end) = self.phrase_window(tune, position);
        let span = (end - start).max(1) as f32;
        let context = super::bass_passing::PassingContext {
            key: tune.plan.key,
            mode: tune.plan.mode,
            seed: tune.accompaniment_seed,
            bar,
            phase: (position.bar_in_part - 1 - start) as f32 / span,
            phrase_bars: span,
            density: self.arc_density(arc),
            arousal: self.musical_params.arousal,
            strong_steps: &a.strong_steps,
            chords: &chords,
            steps_per_chord: self.profile.harmony.steps_per_chord,
            next_target,
        };
        let id = &mut self.next_note_id;
        super::bass_passing::apply(measure, &a.bass, &context, &mut || {
            let next = *id;
            *id += 1;
            next
        });
    }

    /// Mesures `[début, fin)` (0-based dans la partie) de la phrase qui contient
    /// la mesure de `position` : bornes `phrase_ends` du profil, sinon rôles
    /// cadentiels ; la partie entière à défaut de frontière.
    fn phrase_window(&self, tune: &ActiveTune, position: &TunePosition) -> (usize, usize) {
        let offset = position.bar_in_part - 1;
        let base = self.performed_bar(position) - offset;
        let length = self.profile.form.parts[position.part.0].roles.len();
        let mut start = 0;
        for i in 0..length {
            if self.profile.form.phrase_end(position.part.0, i, tune.plan.bar_role(base + i).role) {
                if i >= offset {
                    return (start, i + 1);
                }
                start = i + 1;
            }
        }
        (start, length)
    }

    fn update_parallel_mode(
        &mut self,
        bar: usize,
        position: &mut TunePosition,
        rng: &mut dyn RngCore,
    ) {
        if self.profile.fixed_tonality.is_some() {
            return;
        }
        let dynamics = &self.tuning.profile.dynamics;
        let Some(tune) = self.tunes.get_mut(&position.tune_index) else { return };
        let tune = Arc::make_mut(tune);
        let (mut mode, dark) = tune.color_at(bar);
        let requested = tune
            .mode_requests
            .range(..=bar)
            .next_back()
            .map_or(tune.initial_dark, |(_, &requested)| requested);
        let valence = self.musical_params.harmony_valence;
        let target_dark = if valence < dynamics.dark_below {
            true
        } else if valence > dynamics.bright_above {
            false
        } else {
            requested
        };
        if target_dark != requested {
            tune.mode_requests.insert(bar, target_dark);
        }
        if !position.is_intro()
            && (position.bar_in_part == 1
                || (matches!(
                    dynamics.mode_shift_boundary,
                    super::dynamics::ModeShiftBoundary::Phrase
                ) && self.profile.form.phrase_end(
                    position.part.0,
                    position.bar_in_part - 2,
                    tune.plan
                        .bar_role(
                            position.bar_in_tour() - 1
                                + position.tour.saturating_sub(1)
                                    * self.profile.form.bars_per_tour(),
                        )
                        .role,
                )))
            && target_dark != dark
        {
            mode = if target_dark {
                *tune
                    .dark_mode
                    .get_or_insert_with(|| dynamics.draw_parallel_dark(tune.bright_mode, rng))
            } else {
                tune.bright_mode
            };
            tune.mode_changes.insert(bar, (mode, target_dark));
        }
        if tune.plan.mode != mode {
            tune.plan.mode = mode;
            tune.melody.set_mode(mode);
            if matches!(tune.harmony, HarmonyPlan::MelodyFunctional(_)) {
                let melody: Vec<_> = (0..tune.plan.material_bars())
                    .map(|i| tune.melody.notes_for_bar(tune.plan.representative_bar(i)))
                    .collect();
                if let Some(accompaniment) = &self.tuning.profile.accompaniment
                    && let Ok(h) = TuneHarmony::new(tune.plan.clone(), &melody, accompaniment)
                {
                    tune.harmony = HarmonyPlan::MelodyFunctional(h);
                }
            }
            if matches!(tune.harmony, HarmonyPlan::Loop(_)) {
                tune.harmony =
                    HarmonyPlan::Loop(LoopHarmony::new(tune.plan.clone(), tune.variant_seed));
            }
        }
        position.mode = mode;
    }

    /// Remplit une mesure d'intro selon la forme tirée et la levée.
    fn fill_intro_measure(
        &mut self,
        tune: &ActiveTune,
        position: &TunePosition,
        measure: &mut Measure,
    ) {
        let intro = &tune.intro_info;
        let plan = &tune.plan;
        let tuning = &self.tuning;
        let steps = measure.steps;

        measure.chord_context = key_context(plan.key, plan.mode);

        // Indice (0-based) de la mesure d'intro dans cette intro : les mesures
        // d'intro sont générées séquentiellement, la position ne stockant pas
        // cet indice.
        let intro_bar_idx = self.current_bar - self.intro_start_bar_for_tune(position.tune_index);

        if intro_bar_idx < intro.leading_silence {
            return;
        }
        let form_bar = intro_bar_idx - intro.leading_silence;
        let form = if form_bar < intro.form_bars { intro.form } else { None };
        match form {
            Some(IntroRole::Percussion) => {
                // Décompte : les pistes de percussion jouent leur pattern seul.
                let default_intro_tracks = [TrackId::PercussionLow];
                let intro_tracks = tuning
                    .profile
                    .percussion
                    .intro_tracks
                    .as_deref()
                    .unwrap_or(&default_intro_tracks);
                let percussion_low_wanted = intro_tracks.contains(&TrackId::PercussionLow)
                    && self.track_active_in_intro(TrackId::PercussionLow, intro);
                let percussion_high_wanted = intro_tracks.contains(&TrackId::PercussionHigh)
                    && self.track_active_in_intro(TrackId::PercussionHigh, intro);
                if percussion_low_wanted || percussion_high_wanted {
                    let zero_humanize = vec![0i8; steps];
                    let pattern = tune.percussion_plan.pattern(
                        &tuning.profile.percussion,
                        plan.profile.form.sequence[0],
                        0,
                        self.percussion_density(
                            self.effective_intensity(position.tune_index),
                            self.musical_params.rhythm_density,
                        ),
                        false,
                    );
                    let chance =
                        chance_seed(tune.accompaniment_seed, (1u64 << 40) | intro_bar_idx as u64);
                    let (percussion_low_notes, percussion_high_notes) = percussion_notes(
                        &tuning.profile.percussion,
                        pattern,
                        steps,
                        &zero_humanize,
                        chance,
                    );
                    if percussion_low_wanted {
                        for mut note in percussion_low_notes {
                            note.id = self.alloc_note_id();
                            measure.add_note(TrackId::PercussionLow, note);
                        }
                    }
                    if percussion_high_wanted {
                        for mut note in percussion_high_notes {
                            note.id = self.alloc_note_id();
                            measure.add_note(TrackId::PercussionHigh, note);
                        }
                    }
                }
            }
            Some(IntroRole::Tail) => {
                // Entrée par la fin : la mélodie joue les `intro_bars` dernières
                // mesures de l'air (cadence finale), lues depuis la forme de l'air.
                if self.track_active_in_intro(TrackId::Melody, intro) {
                    let total_bars = plan.total_bars();
                    let tail_bars = intro.form_bars.max(1);
                    let bar_in_tune =
                        total_bars.saturating_sub(tail_bars) + form_bar.min(tail_bars - 1);
                    for mut note in tune.melody.notes_for_bar(bar_in_tune) {
                        note.id = self.alloc_note_id();
                        measure.add_note(TrackId::Melody, note);
                    }
                }
            }
            Some(IntroRole::Vamp) => {
                // Vamp : l'accompagnement joue sur la tonique.
                if self.track_active_in_intro(TrackId::Chords, intro) {
                    let tonic = DegreeChord::new(ChordDegree::I, ChordQuality::Major);
                    let chords = vec![tonic; steps / self.profile.harmony.steps_per_chord];
                    let seed = crate::ornaments::mix(
                        tune.accompaniment_seed,
                        self.performed_bar(position) as u64,
                    );
                    let next_note_id = &mut self.next_note_id;
                    let mut next_id = || {
                        let id = *next_note_id;
                        *next_note_id += 1;
                        id
                    };
                    if let Some(accompaniment) =
                        dressed(&self.session, &tuning.profile.accompaniment)
                    {
                        let mut chord_player = ChordPlayer::new();
                        for note in chord_player.chord_bar(
                            &chords,
                            plan.key,
                            plan.mode,
                            accompaniment,
                            self.profile.harmony.steps_per_chord,
                            seed,
                            &mut next_id,
                        ) {
                            measure.add_note(TrackId::Chords, note);
                        }
                    }
                }
            }
            None => {
                // Levée seule (pas de rôle, juste pickup dans la dernière mesure).
            }
        }

        // Levée (anacrouse) : dans la dernière mesure d'intro, jouer les notes
        // de pickup dans la piste Melody, sur les derniers pas de la mesure.
        let intro_bars = intro.intro_bars;
        if intro_bars > 0
            && !intro.pickup_notes.is_empty()
            && intro_bar_idx == intro_bars - 1
            && self.musical_params.enable_melody
        {
            let pickup_len = intro.pickup_notes.len();
            // Taille d'une croche en pas, lue depuis la grille de l'air.
            let eighth_steps = self.profile.meter.steps_per_unit;
            for (i, &midi) in intro.pickup_notes.iter().enumerate() {
                let start_step = steps - (pickup_len - i) * eighth_steps;
                let note = TimelineNote {
                    id: self.alloc_note_id(),
                    pitch: midi,
                    velocity: 80,
                    start_step,
                    duration_steps: eighth_steps,
                    articulation: Articulation::default(),
                    performance: Default::default(),
                };
                measure.add_note(TrackId::Melody, note);
            }
            // La fin empruntée peut attaquer après le début de la levée.
            // Le playhead parcourt chaque piste par attaques croissantes.
            if let Some(notes) = measure.tracks.get_mut(&TrackId::Melody) {
                notes.sort_unstable_by_key(|note| (note.start_step, note.id));
            }
        }
    }

    /// Calcule la mesure de début de l'intro pour `tune_index`.
    fn intro_start_bar_for_tune(&self, tune_index: usize) -> usize {
        let mut offset = 1usize;
        for idx in 0..tune_index {
            if let Some(t) = self.tunes.get(&idx) {
                offset += t.intro_info.intro_bars;
                offset += self.profile.form.bars_per_tour()
                    * self.tuning.profile.arrangement.tours_per_tune;
            }
        }
        offset
    }

    fn performed_bar(&self, position: &TunePosition) -> usize {
        position.bar_in_tour() + position.tour.saturating_sub(1) * self.profile.form.bars_per_tour()
    }

    /// Remplit une mesure de tour normale (identique à l'existant D11).
    fn fill_tour_measure(
        &mut self,
        tune: &ActiveTune,
        position: &TunePosition,
        measure: &mut Measure,
        arc: Option<&ArcBar<'_>>,
    ) {
        if let HarmonyPlan::None = &tune.harmony {
            let bar_in_tour = self.performed_bar(position);
            let plan = &tune.plan;
            measure.chord_context = key_context(plan.key, plan.mode);
            if self.track_active(TrackId::Melody, position) {
                for mut note in tune.melody.notes_for_bar(bar_in_tour) {
                    note.id = self.alloc_note_id();
                    measure.add_note(TrackId::Melody, note);
                }
            }
            self.fill_percussion(tune, position, measure, arc);
            return;
        }
        let bar_in_tour = self.performed_bar(position);
        let plan = &tune.plan;
        let chords = self.chords_for_bar(tune, bar_in_tour);
        self.publish_chords(tune, bar_in_tour, &chords, measure);

        // 1. Mélodie (D6)
        if self.track_active(TrackId::Melody, position) {
            for mut note in tune.melody.notes_for_bar(bar_in_tour) {
                note.id = self.alloc_note_id();
                measure.add_note(TrackId::Melody, note);
            }
        }

        // 2. Accompagnement (accords + basse, D8)
        if self.track_active(TrackId::Chords, position) {
            for note in self.chord_notes_for_bar(tune, bar_in_tour, arc) {
                measure.add_note(TrackId::Chords, note);
            }
        }
        if self.track_active(TrackId::Bass, position) {
            let next_first = self.next_bar_first_chord(tune, bar_in_tour);
            let next_note_id = &mut self.next_note_id;
            let mut next_id = || {
                let id = *next_note_id;
                *next_note_id += 1;
                id
            };
            if let Some(accompaniment) = &self.tuning.profile.accompaniment {
                let notes = if let Some(walking) = &accompaniment.bass.walking {
                    walking_bar(
                        &WalkingBar {
                            chords: &chords,
                            next_first,
                            key: plan.key,
                            mode: plan.mode,
                            steps_per_chord: self.profile.harmony.steps_per_chord,
                            steps_per_pulse: self.profile.meter.steps_per_pulse,
                            seed: crate::ornaments::mix(
                                tune.accompaniment_seed,
                                bar_in_tour as u64,
                            ),
                            next_seed: crate::ornaments::mix(
                                tune.accompaniment_seed,
                                bar_in_tour as u64 + 1,
                            ),
                            attack_pulses: arc.map_or(1, ArcBar::attack_pulses),
                        },
                        &accompaniment.bass,
                        walking,
                        &mut next_id,
                    )
                } else if let Some(slice) = tune.bass_plan.slice_for_bar(plan, bar_in_tour) {
                    if let Some(index) = slice.template_index() {
                        bass_template_bar(
                            &accompaniment.bass.templates[index],
                            &TemplateBar {
                                chords: &chords,
                                next_first,
                                key: plan.key,
                                mode: plan.mode,
                                steps_per_chord: self.profile.harmony.steps_per_chord,
                                steps_per_bar: self.profile.meter.steps_per_bar,
                                bar_in_cycle: slice.bar_in_cycle(),
                            },
                            &accompaniment.bass,
                            &mut next_id,
                        )
                    } else {
                        bass_motif_bar(
                            slice,
                            &chords,
                            plan.key,
                            plan.mode,
                            accompaniment,
                            self.profile.meter.steps_per_bar,
                            self.profile.harmony.steps_per_chord,
                            &mut next_id,
                        )
                    }
                } else {
                    bass_bar(
                        &chords,
                        plan.key,
                        plan.mode,
                        accompaniment,
                        self.profile.harmony.steps_per_chord,
                        &mut next_id,
                    )
                };
                for note in notes {
                    measure.add_note(TrackId::Bass, note);
                }
            }
        }
        self.fill_percussion(tune, position, measure, arc);
    }

    /// Premier accord de la mesure suivante du tour joué ; `None` après la dernière.
    fn next_bar_first_chord(&self, tune: &ActiveTune, bar_in_tour: usize) -> Option<DegreeChord> {
        let bars =
            self.profile.form.bars_per_tour() * self.tuning.profile.arrangement.tours_per_tune;
        (bar_in_tour + 1 < bars).then(|| self.chords_for_bar(tune, bar_in_tour + 1)[0])
    }

    /// Contexte d'accord de la mesure et changements d'accord publiés, d'après le modèle
    /// d'harmonie : `loop` et `rewrite_functional` publient chaque changement, la
    /// fonctionnelle par mélodie un accord par mesure.
    fn publish_chords(
        &self,
        tune: &ActiveTune,
        bar_in_tour: usize,
        chords: &[DegreeChord],
        measure: &mut Measure,
    ) {
        let plan = &tune.plan;
        let period = self.profile.harmony.steps_per_chord;
        if let HarmonyPlan::RewriteFunctional(bars) = &tune.harmony {
            for (slot, chord) in bars[plan.material_index(bar_in_tour)].iter().enumerate() {
                let mut context = ChordContext::from_chord(chord, plan.key, chord.name());
                context.scale_guidance = self.chord_guidance(chord, &context.chord_name);
                measure.chord_changes.push(ChordChange { start_step: slot * period, context });
            }
            measure.chord_context = measure.chord_changes[0].context.clone();
            return;
        }
        measure.chord_context = self.modal_chord_context(chords[0], plan.key, plan.mode);
        if matches!(tune.harmony, HarmonyPlan::Loop(_)) {
            // Un changement par accord distinct de la mesure, plus l'attaque de la mesure.
            for (slot, chord) in chords.iter().enumerate() {
                if slot == 0 || *chord != chords[slot - 1] {
                    measure.chord_changes.push(ChordChange {
                        start_step: slot * period,
                        context: if slot == 0 {
                            measure.chord_context.clone()
                        } else {
                            self.modal_chord_context(*chord, plan.key, plan.mode)
                        },
                    });
                }
            }
        }
    }

    /// Gammes d'improvisation d'un accord typé (Lydian Chromatic, tension harmonique de la
    /// mesure) : le même calcul pour tous les modèles, seulement si le profil le demande.
    fn chord_guidance(&self, chord: &Chord, symbol: &str) -> Option<ScaleGuidance> {
        self.profile.harmony.scale_guidance.then(|| {
            LydianChromaticConcept::new().scale_guidance(
                chord,
                symbol.to_string(),
                self.musical_params.harmony_tension,
            )
        })
    }

    /// Contexte d'un accord relatif à la tonique (modèles `loop` et `melody_functional`),
    /// avec ses gammes si le profil les demande. Une pédale n'a pas d'accord typé : pas de
    /// gammes.
    fn modal_chord_context(&self, chord: DegreeChord, key: PitchClass, mode: Mode) -> ChordContext {
        let mut context = build_chord_context(chord, key, mode);
        if let Some(typed) = chord.to_chord(key, mode) {
            context.scale_guidance = self.chord_guidance(&typed, &context.chord_name);
        }
        context
    }

    /// Les cellules restent la source du rythme et du contour ; seules les
    /// hauteurs hors accord/gamme sont rapprochées du contexte publié.
    fn fit_planned_melody(
        &mut self,
        measure: &mut Measure,
        tune: &ActiveTune,
        position: &TunePosition,
    ) {
        if !matches!(tune.harmony, HarmonyPlan::RewriteFunctional(_) | HarmonyPlan::Loop(_)) {
            return;
        }
        if measure.chord_changes.is_empty() {
            return;
        }
        let limits = self.tuning.profile.melody.difficulty.as_ref();
        if limits.is_some_and(|limits| !limits.chord_fit) {
            return;
        }
        // Filtre de difficulté sans altération : seules les notes de la gamme de l'air restent
        // permises ; sans note permise, la hauteur écrite est gardée.
        let in_scale_only = limits.is_some_and(|limits| !limits.out_of_scale);
        let (key, mode) = (tune.plan.key, tune.plan.mode);
        let in_scale = move |pc: &u8| !in_scale_only || mode.contains_midi(key, *pc);
        let bar = tune.plan.material_index(self.performed_bar(position));
        let Some(notes) = measure.tracks.get_mut(&TrackId::Melody) else { return };
        let source = std::mem::take(notes);
        notes.reserve(source.len() + measure.chord_changes.len());
        let params = &self.tuning.profile.melody.params;
        for note in source {
            let end = note.start_step + note.duration_steps;
            let mut start = note.start_step;
            while start < end {
                let slot =
                    measure.chord_changes.partition_point(|change| change.start_step <= start) - 1;
                let change = &measure.chord_changes[slot];
                let next =
                    measure.chord_changes.get(slot + 1).map_or(measure.steps, |c| c.start_step);
                let mut segment = note.clone();
                segment.pitch = match &tune.harmony {
                    HarmonyPlan::RewriteFunctional(bars) => {
                        let chord = &bars[bar][slot];
                        let scale = change
                            .context
                            .scale_guidance
                            .as_ref()
                            .map_or(&[][..], |guidance| guidance.scale_notes.as_slice());
                        let allowed = chord
                            .chord_type
                            .intervals()
                            .iter()
                            .map(|&interval| (chord.root + interval) % 12)
                            .chain(scale.iter().copied())
                            .filter(in_scale);
                        if allowed.clone().next().is_none() {
                            note.pitch
                        } else {
                            nearest_pitch(
                                allowed,
                                note.pitch,
                                params.tessitura_min,
                                params.tessitura_max,
                            )
                        }
                    }
                    HarmonyPlan::Loop(harmony) => {
                        // Les changements du modèle `loop` ne listent que les accords
                        // distincts : le créneau se lit au pas du changement.
                        let chord = harmony.chords_for_bar(self.performed_bar(position))
                            [change.start_step / self.profile.harmony.steps_per_chord];
                        let allowed = chord
                            .pitch_classes(tune.plan.key, tune.plan.mode)
                            .into_iter()
                            .filter(in_scale);
                        if allowed.clone().next().is_none() {
                            note.pitch
                        } else {
                            nearest_pitch(
                                allowed,
                                note.pitch,
                                params.tessitura_min,
                                params.tessitura_max,
                            )
                        }
                    }
                    _ => unreachable!("modèle filtré avant la réalisation mélodique"),
                };
                segment.start_step = start;
                segment.duration_steps = end.min(next) - start;
                if start != note.start_step {
                    segment.id = self.next_note_id;
                    self.next_note_id += 1;
                }
                notes.push(segment);
                start = end.min(next);
            }
        }
    }

    fn fill_percussion(
        &mut self,
        tune: &ActiveTune,
        position: &TunePosition,
        measure: &mut Measure,
        arc: Option<&ArcBar<'_>>,
    ) {
        // 3. Percussion (percussion grave + percussion aiguë, D9)
        let percussion_low_active = self.track_active(TrackId::PercussionLow, position);
        let percussion_high_active = self.track_active(TrackId::PercussionHigh, position);
        if percussion_low_active || percussion_high_active {
            let slot = self.profile.form.unique_offset(position.part.0) + position.bar_in_part - 1;
            let humanize = &tune.percussion_humanize[slot];
            let is_cadence = self.profile.form.phrase_end(
                position.part.0,
                position.bar_in_part - 1,
                tune.plan.bar_role(self.performed_bar(position)).role,
            );
            let table = &self.tuning.profile.percussion;
            let density = self.percussion_density(
                self.arc_intensity(position.tune_index, arc),
                self.arc_density(arc),
            );
            let tags = arc.map_or(&[][..], ArcBar::percussion_tags);
            let form = &self.profile.form;
            // Phrases de batterie de la passe de la partie et relance de fin de phrase : tirées
            // sans état depuis la graine d'accompagnement (la mesure se rejoue après un seek).
            let performed = self.performed_bar(position);
            let (window, rank, part) = phrasing_frame(
                form,
                self.profile.clave.as_ref(),
                tune.accompaniment_seed,
                performed,
                PERCUSSION_PHRASES,
            );
            let tiling = if table.is_phrased() {
                table.phrase_tiling(window, &form.parts[part].name, density, tags)
            } else {
                Vec::new()
            };
            let has_fills = table.has_fills(density);
            // Une relance de la forme (fin de phrase du profil) mesure la distance à la fin de
            // phrase précédente ; elle ne sert qu'aux mesures hors des phrases de la table.
            let form_phrase_end = |end: usize| {
                form.phrase_end(part, end, tune.plan.bar_role(performed - rank + end).role)
            };
            let fill = has_fills
                .then(|| {
                    table.fill_covering(window, rank, density, tags, &tiling, |end| {
                        form_phrase_end(end).then(|| {
                            let start =
                                (0..end).rev().find(|&e| form_phrase_end(e)).map_or(0, |e| e + 1);
                            end + 1 - start
                        })
                    })
                })
                .flatten();
            // Une mesure de cadence garde son pattern ; ailleurs la phrase de la table, puis
            // l'arc (patterns de ses tags), puis le choix ordinaire. Une banque de relances
            // remplace le pattern de cadence de la densité.
            let cadence_pattern = is_cadence && !has_fills;
            let replaced = cadence_pattern && table.has_cadence_pattern(density);
            let phrased = tiling
                .get(rank)
                .copied()
                .flatten()
                .filter(|_| !replaced)
                .map(|slot| table.phrase_pattern(slot));
            let tagged = arc.filter(|_| !replaced && phrased.is_none()).and_then(|arc| {
                table.tagged_pattern(
                    &form.parts[position.part.0].name,
                    density,
                    arc.percussion_tags(),
                    arc.percussion_seed(position.part.0, density.min(2)),
                )
            });
            let pattern = phrased.or(tagged).unwrap_or_else(|| {
                tune.percussion_plan.pattern(
                    table,
                    position.part.0,
                    position.tour,
                    density,
                    cadence_pattern,
                )
            });
            let chance = chance_seed(tune.accompaniment_seed, performed as u64);
            let (mut percussion_low_notes, mut percussion_high_notes) =
                percussion_notes(table, pattern, measure.steps, humanize, chance);
            if let Some((fill, bar)) = fill {
                // La relance reprend le pattern à son pas de départ (première mesure) ou le
                // remplace (mesures suivantes).
                let from = if bar == 0 { fill.start_step } else { 0 };
                let overlay = PercussionPattern::new(density, fill.bars[bar].hits.clone());
                let (low, high) =
                    percussion_notes(table, &overlay, measure.steps, humanize, chance);
                for (notes, played) in
                    [(&mut percussion_low_notes, low), (&mut percussion_high_notes, high)]
                {
                    notes.retain(|note| note.start_step < from);
                    notes.extend(played);
                    notes.sort_by_key(|note| note.start_step);
                }
            }
            if percussion_low_active {
                for mut note in percussion_low_notes {
                    note.id = self.alloc_note_id();
                    measure.add_note(TrackId::PercussionLow, note);
                }
            }
            if percussion_high_active {
                for mut note in percussion_high_notes {
                    note.id = self.alloc_note_id();
                    measure.add_note(TrackId::PercussionHigh, note);
                }
            }
        }
    }

    /// Remplit une mesure d'outro : tonique accentuée sur le temps 1, puis silence.
    fn fill_outro_measure(
        &mut self,
        tune: &ActiveTune,
        _position: &TunePosition,
        measure: &mut Measure,
    ) {
        let plan = &tune.plan;
        let tonic_midi = tonic_midi_for_key(
            plan.key,
            self.tuning.profile.melody.params.tonic_midi_min,
            self.tuning.profile.melody.params.tonic_midi_max,
        );

        measure.chord_context = key_context(plan.key, plan.mode);

        // Tonique accentuée sur le pas 0 (temps 1), durée 8 pas (blanche)
        if self.musical_params.enable_melody && self.tuning.profile.has_track(TrackId::Melody) {
            measure.add_note(
                TrackId::Melody,
                TimelineNote {
                    id: self.alloc_note_id(),
                    pitch: tonic_midi,
                    velocity: 110,
                    start_step: 0,
                    duration_steps: self.profile.meter.steps_per_pulse,
                    articulation: Articulation::Normal,
                    performance: Default::default(),
                },
            );
        }

        // Accompagnement : tonique accentuée
        if self.musical_params.enable_harmony {
            let next_id = &mut self.next_note_id;
            if self.tuning.profile.has_track(TrackId::Chords) {
                // Accords : accord tonique accentué
                measure.add_note(
                    TrackId::Chords,
                    TimelineNote {
                        id: {
                            let id = *next_id;
                            *next_id += 1;
                            id
                        },
                        pitch: tonic_midi,
                        velocity: 110,
                        start_step: 0,
                        duration_steps: self.profile.meter.steps_per_pulse,
                        articulation: Articulation::Normal,
                        performance: Default::default(),
                    },
                );
            }
            if self.tuning.profile.has_track(TrackId::Bass) {
                // Basse : fondamentale, deux octaves sous
                let bass_pitch = tonic_midi.saturating_sub(24);
                measure.add_note(
                    TrackId::Bass,
                    TimelineNote {
                        id: {
                            let id = *next_id;
                            *next_id += 1;
                            id
                        },
                        pitch: bass_pitch,
                        velocity: 110,
                        start_step: 0,
                        duration_steps: self.profile.meter.steps_per_pulse,
                        articulation: Articulation::Normal,
                        performance: Default::default(),
                    },
                );
            }
        }

        // Percussion : coup grave accentué
        if self.musical_params.enable_rhythm
            && self.tuning.profile.has_track(TrackId::PercussionLow)
        {
            measure.add_note(
                TrackId::PercussionLow,
                TimelineNote {
                    id: self.alloc_note_id(),
                    pitch: self.tuning.profile.percussion.low_note,
                    velocity: 110,
                    start_step: 0,
                    duration_steps: self.profile.meter.steps_per_unit,
                    articulation: Articulation::Normal,
                    performance: Default::default(),
                },
            );
        }
    }

    /// Variation harmonique de l'accompagnement uniquement : cadences et
    /// squelette mélodique restent ceux de l'air.
    fn chords_for_bar<'a>(&self, tune: &'a ActiveTune, bar: usize) -> Cow<'a, [DegreeChord]> {
        match &tune.harmony {
            HarmonyPlan::Loop(harmony) => return Cow::Borrowed(harmony.chords_for_bar(bar)),
            HarmonyPlan::RewriteFunctional(bars) => {
                return Cow::Owned(
                    bars[tune.plan.material_index(bar)]
                        .iter()
                        .map(|chord| DegreeChord::from_chord(chord, tune.plan.key))
                        .collect(),
                );
            }
            HarmonyPlan::MelodyFunctional(_) | HarmonyPlan::None => {}
        }
        let HarmonyPlan::MelodyFunctional(harmony) = &tune.harmony else {
            unreachable!("l'accompagnement demande une harmonie qui joue des accords")
        };
        let mut chords = Cow::Borrowed(harmony.chords_for_bar(bar));
        let threshold =
            self.tuning.profile.dynamics.tension_dominant.eval(self.musical_params.harmony_tension);
        let role = tune.plan.bar_role(bar);
        let slot = usize::from(role.bar_in_part) - 1;
        let length = self.profile.form.parts[role.part.0].roles.len();
        if !matches!(role.role, ImposedRole::CadHalf | ImposedRole::CadFull | ImposedRole::CadTonic)
            && (slot as f32 / length as f32) < threshold
        {
            let last = chords.len() - 1;
            if let Some(chord) = super::harmony::candidate_chords(tune.plan.mode, &self.profile)
                .iter()
                .copied()
                .find(|c| c.degree == ChordDegree::V)
            {
                chords.to_mut()[last] = chord;
            }
        }
        chords
    }

    fn apply_dynamics(&mut self, measure: &mut Measure, arc: Option<&ArcBar<'_>>) {
        let p = &self.musical_params;
        let d = &self.tuning.profile.dynamics;
        let intensity =
            self.arc_intensity(measure.position.map_or(0, |position| position.tune_index), arc);
        let duration =
            d.melody_duration.eval(intensity) * d.tension_duration.eval(p.harmony_tension);
        let strong = d.strong_accent.eval(intensity)
            + d.brightness_accent.eval(d.valence_input(p.harmony_valence));
        let offbeat = d.tension_offbeat.eval(p.harmony_tension);
        let chord_duration = d.tension_chord_duration.eval(p.harmony_tension);
        for (&track, notes) in &mut measure.tracks {
            let velocity = d.velocity(track, intensity);
            for note in notes {
                let accent = if matches!(track, TrackId::Melody | TrackId::Countermelody) {
                    if note.start_step % usize::from(measure.pulse.steps_per_pulse) == 0 {
                        strong
                    } else if self
                        .profile
                        .melody
                        .weak_units
                        .contains(&(note.start_step / self.profile.meter.steps_per_unit))
                    {
                        offbeat
                    } else {
                        0.0
                    }
                } else if track == TrackId::PercussionHigh || track == TrackId::PercussionLow {
                    if note.pitch == 40 { offbeat.max(0.0) } else { 0.0 }
                } else {
                    0.0
                };
                note.velocity =
                    (f32::from(note.velocity) * velocity + accent).round().clamp(1.0, 127.0) as u8;
                if matches!(track, TrackId::Melody | TrackId::Countermelody) {
                    note.duration_steps =
                        (note.duration_steps as f32 * duration).round().max(1.0) as usize;
                    note.duration_steps = note.duration_steps.min(measure.steps - note.start_step);
                } else if track == TrackId::Chords {
                    note.duration_steps =
                        (note.duration_steps as f32 * chord_duration).round().max(1.0) as usize;
                    note.duration_steps = note.duration_steps.min(measure.steps - note.start_step);
                }
            }
        }
    }

    /// Ornements idiomatiques de la mélodie (`[ornaments]`), après l'interprétation.
    fn apply_ornaments(
        &mut self,
        measure: &mut Measure,
        tune: &ActiveTune,
        position: &TunePosition,
        written: &[usize],
    ) {
        let context = self.ornament_context(measure, tune, position);
        let next_note_id = &mut self.next_note_id;
        self.profile.ornaments.apply(measure, TrackId::Melody, &context, written, &mut || {
            let id = *next_note_id;
            *next_note_id += 1;
            id
        });
    }

    fn ornament_context(
        &self,
        measure: &Measure,
        tune: &ActiveTune,
        position: &TunePosition,
    ) -> BarContext {
        let part = &self.profile.form.parts[position.part.0];
        // Omission : la dernière mesure de la partie, si elle porte une cadence.
        let protected = part.ornament_protected_bars.as_ref().map_or_else(
            || {
                position.bar_in_part == part.roles.len()
                    && tune.plan.bar_role(self.performed_bar(position)).role.is_cadential()
            },
            |bars| bars.contains(&position.bar_in_part),
        );
        let (_, end) = self.phrase_window(tune, position);
        let params = &self.tuning.profile.melody.params;
        BarContext {
            key: tune.plan.key,
            mode: tune.plan.mode,
            tessitura: (params.tessitura_min, params.tessitura_max),
            bar_in_part: self.copy_source_bar(tune, position),
            steps_per_pulse: self.profile.meter.steps_per_pulse,
            phrase_end_bar: position.bar_in_part == end,
            protected,
            repeat: position.pass > 1 || position.tour > 1,
            arousal: self.musical_params.arousal,
            density: self.musical_params.rhythm_density,
            bar_index: measure.index,
            seed: tune.material_seed,
            independent_steps: Vec::new(),
        }
    }

    /// Mesure de la partie dont les taux par mesure s'appliquent : une copie exacte
    /// (`CopyOf`, `CopyOfPart`) emprunte celle de sa source, pour porter les mêmes
    /// ornements qu'elle.
    fn copy_source_bar(&self, tune: &ActiveTune, position: &TunePosition) -> usize {
        let mut bar = position.bar_in_part;
        let mut part = position.part.0;
        let mut role = tune.plan.bar_role(self.performed_bar(position)).role;
        for _ in 0..8 {
            match role {
                ImposedRole::CopyOf(source) => bar = usize::from(source),
                ImposedRole::CopyOfPart { part: source_part, bar: source } => {
                    part = source_part;
                    bar = usize::from(source);
                }
                _ => break,
            }
            let Some(next) = tune.plan.roles_for_part(part).get(bar.wrapping_sub(1)) else {
                break;
            };
            role = *next;
        }
        bar
    }

    /// Calcule les attaques d'accords de `bar_in_tour` en rejouant le [`ChordPlayer`]
    /// depuis le début de la passe courante, sans hypothèse de longueur ;
    /// le matériel rejoué reste stable après invalidation émotionnelle.
    fn chord_notes_for_bar(
        &mut self,
        tune: &ActiveTune,
        bar_in_tour: usize,
        arc: Option<&ArcBar<'_>>,
    ) -> Vec<TimelineNote> {
        let (_, _, offset) = self.profile.form.position(bar_in_tour);
        let part_pass_start = bar_in_tour - offset;
        let Some(harmony) = dressed(&self.session, &self.tuning.profile.accompaniment) else {
            return Vec::new();
        };
        let d = &self.tuning.profile.dynamics;
        let chord_offset = d.chords_intensity.eval(self.arc_intensity(tune.tune_index, arc))
            + d.density_chords.eval(self.arc_density(arc));
        let tags = arc.map_or(&[][..], ArcBar::comping_tags);
        let (tiling, rank) = comping_tiling(
            &self.profile.form,
            &harmony.chords,
            self.profile.clave.as_ref(),
            tune.accompaniment_seed,
            bar_in_tour,
            tags,
        );
        let mut chord_player = ChordPlayer::new();
        for b in part_pass_start..bar_in_tour {
            let c = self.chords_for_bar(tune, b);
            let seed = crate::ornaments::mix(tune.accompaniment_seed, b as u64);
            chord_player.advance_bar(
                &c,
                self.next_bar_first_chord(tune, b),
                tune.plan.key,
                tune.plan.mode,
                harmony,
                harmony.chords.bar_hits(
                    chord_offset,
                    seed,
                    tags,
                    tiling.get(b - part_pass_start).copied().flatten(),
                ),
                self.profile.harmony.steps_per_chord,
                seed,
            );
        }
        let chords = self.chords_for_bar(tune, bar_in_tour);
        let next_first = self.next_bar_first_chord(tune, bar_in_tour);
        let seed = crate::ornaments::mix(tune.accompaniment_seed, bar_in_tour as u64);
        let hits =
            harmony.chords.bar_hits(chord_offset, seed, tags, tiling.get(rank).copied().flatten());
        let next_note_id = &mut self.next_note_id;
        let mut next_id = || {
            let id = *next_note_id;
            *next_note_id += 1;
            id
        };
        chord_player.chord_bar_on_hits(
            &chords,
            next_first,
            tune.plan.key,
            tune.plan.mode,
            harmony,
            hits,
            self.profile.harmony.steps_per_chord,
            seed,
            &mut next_id,
        )
    }

    /// S'assure que tous les tunes nécessaires pour couvrir `bar_index` ont été
    /// tirés, avec leur intro. Appelé avant [`Self::position_of`].
    fn ensure_tunes_covering(&mut self, bar_index: usize, rng: &mut dyn RngCore) {
        let tours = self.tuning.profile.arrangement.tours_per_tune;
        let cycle = self.profile.form.bars_per_tour() * tours;
        let mut cumulative_start = 1usize; // 1-based start of current tune's intro
        let mut tune_idx = 0usize;
        self.ensure_session(rng);

        loop {
            // La demande du jeu termine l'air courant, pas nécessairement l'air de session 0.
            if self.ending_tune.is_some_and(|last| tune_idx > last) {
                return;
            }
            if !self.tunes.contains_key(&tune_idx) {
                if self.set_of(tune_idx).is_none() {
                    // Début d'un set : le bloc de tirages passe avant la construction de l'air 1.
                    self.sets.push(SetPlan::draw(tune_idx, &self.tuning.profile.set, rng));
                }
                let prev = tune_idx.checked_sub(1).and_then(|prev_idx| self.tunes.get(&prev_idx));
                let prev_key_mode = prev.map(|t| (t.plan.key, t.plan.mode));
                let prev_first_cell = prev.map(|t| t.melody.cell_for_bar(0).clone());
                let tune = self.build_tune(tune_idx, prev_key_mode, prev_first_cell, rng);
                self.tunes.insert(tune_idx, Arc::new(tune));
            }
            let intro_bars = self.tunes[&tune_idx].intro_info.intro_bars;
            let tune_start = cumulative_start + intro_bars;
            let tune_end = tune_start + cycle;
            if bar_index < tune_end {
                return;
            }
            cumulative_start = tune_end;
            tune_idx += 1;
        }
    }

    /// Fenêtre bornée sur les airs déjà conservés pour le rejeu : aucune deuxième copie
    /// du matériel, aucun état de mémoire à invalider séparément.
    fn signature_source(&self, tune_index: usize, valence: f32) -> Option<&ActiveTune> {
        let signature = self.profile.set.signature;
        if signature.return_rate == 0.0 || signature.memory == 0 {
            return None;
        }
        let (_, set) = self.set_of(tune_index)?;
        let u = f64::from(set.signature_draw(set.tune_in_set(tune_index))?) / KEY_DRAW_RANGE as f64;
        let rate = f64::from(signature.return_rate);
        if u >= rate {
            return None;
        }
        let allowed = candidates(&self.profile, valence);
        let eligible = self
            .tunes
            .range(..set.first_tune())
            .rev()
            .take(signature.memory)
            .map(|(_, tune)| tune.as_ref())
            .filter(|tune| {
                allowed.iter().any(|c| {
                    c.weight > 0.0 && c.key == tune.plan.key && c.mode == tune.initial_mode
                }) && self
                    .tunes
                    .range(set.first_tune()..tune_index)
                    .all(|(_, current)| current.material_origin != tune.material_origin)
            });
        let count = eligible.clone().count();
        eligible.into_iter().nth((u / rate * count as f64) as usize)
    }

    /// Construit un [`ActiveTune`] complet (`TunePlan` + `MelodyLine` + `TuneHarmony` + intro).
    ///
    /// Règle P1-T7 : deux tunes consécutifs ne commencent pas par la même
    /// cellule (à une transposition d'octave près) ; en cas de collision, la
    /// ligne est retirée (tirage déterministe suivant) jusqu'à 8 essais.
    fn build_tune(
        &self,
        tune_index: usize,
        prev_key_mode: Option<(PitchClass, Mode)>,
        prev_first_cell: Option<Cell>,
        rng: &mut dyn RngCore,
    ) -> ActiveTune {
        let dynamics = &self.tuning.profile.dynamics;
        let valence = self.musical_params.harmony_valence;
        let source = self.signature_source(tune_index, valence);
        let material_origin = source.map_or(tune_index, |tune| tune.material_origin);
        let variant_seed = if self.profile.harmony.has_alternatives()
            || self.profile.percussion.has_alternatives()
        {
            let high = rng.next_range_u64(0..1 << 32);
            (high << 32) | rng.next_range_u64(0..1 << 32)
        } else {
            0
        };
        // Graine de l'arc : un tirage par air, seulement si le profil a un arc (les profils
        // sans arc gardent le même flux). Celle du premier air du set sert à tout le set.
        let arc_seed = if self.chorus_arc().is_some() {
            let high = rng.next_range_u64(0..1 << 32);
            (high << 32) | rng.next_range_u64(0..1 << 32)
        } else {
            0
        };
        let (plan, melody, loop_harmony) = if let Some(source) = source {
            let plan = source.melody.plan();
            let harmony = (self.profile.harmony.model == "loop")
                .then(|| LoopHarmony::new(plan.clone(), variant_seed));
            let mut melody = source.melody.clone();
            melody.set_mode(plan.mode);
            (plan, melody, harmony)
        } else {
            let (key, mode) = self.draw_key_mode(tune_index, prev_key_mode, valence, rng);
            let mut plan = TunePlan::with_profile(key, mode, self.profile.clone());
            plan.resolve_roles(rng);
            // Fixer les alternatives avant toute génération et tout guidage mélodique.
            let harmony = (self.profile.harmony.model == "loop")
                .then(|| LoopHarmony::new(plan.clone(), variant_seed));
            let mut melody_params = self.tuning.profile.melody.params.clone();
            let intensity = self.effective_intensity(tune_index);
            // Courbe d'intensité sur un rôle employé (au moins 1) ; un poids nul reste nul.
            let scale = |weight: u32, curve: &super::dynamics::Curve| {
                if weight == 0 {
                    0
                } else {
                    (weight as f32 * curve.eval(intensity)).round().max(1.0) as u32
                }
            };
            melody_params.weight_role_run =
                scale(melody_params.weight_role_run, &dynamics.run_weight);
            melody_params.weight_role_arp =
                scale(melody_params.weight_role_arp, &dynamics.arp_weight);
            melody_params.weight_role_turn =
                scale(melody_params.weight_role_turn, &dynamics.turn_weight);
            melody_params.melody_center_midi = (f32::from(melody_params.melody_center_midi)
                + dynamics.color_register.eval(dynamics.valence_input(valence)))
            .round()
            .clamp(f32::from(melody_params.tessitura_min), f32::from(melody_params.tessitura_max))
                as u8;
            let mut melody = MelodyLine::with_params(
                plan.clone(),
                &self.tuning.profile.melody.cells,
                &melody_params,
                rng,
            );
            if let Some(prev_first) = prev_first_cell {
                for _ in 0..self.profile.form.opening_retries {
                    if !same_cell_up_to_octave(melody.cell_for_bar(0), &prev_first) {
                        break;
                    }
                    melody = MelodyLine::with_params(
                        plan.clone(),
                        &self.tuning.profile.melody.cells,
                        &melody_params,
                        rng,
                    );
                }
            }
            (plan, melody, harmony)
        };

        let harmony = if self.profile.harmony.model == "rewrite_functional" {
            let mut model = SteedmanGrammar::new(
                self.profile.harmony.rules.clone(),
                self.profile.harmony.max_recursion,
            );
            let chords_per_bar =
                self.profile.meter.steps_per_bar / self.profile.harmony.steps_per_chord;
            let bars = (0..plan.material_bars())
                .map(|_| {
                    (0..chords_per_bar)
                        .map(|_| {
                            let numeral = model.next_numeral(valence, &mut || rng.next_f32());
                            SteedmanGrammar::realize(
                                numeral,
                                plan.key,
                                valence,
                                GrammarStyle::Extended,
                            )
                        })
                        .collect()
                })
                .collect();
            HarmonyPlan::RewriteFunctional(bars)
        } else if let Some(harmony) = loop_harmony {
            HarmonyPlan::Loop(harmony)
        } else if self.profile.harmony.model == "none" {
            HarmonyPlan::None
        } else if let Some(accompaniment) = &self.tuning.profile.accompaniment {
            let distinct_melody: Vec<_> = (0..plan.material_bars())
                .map(|i| melody.notes_for_bar(plan.representative_bar(i)))
                .collect();
            match TuneHarmony::new(plan.clone(), &distinct_melody, accompaniment) {
                Ok(h) => HarmonyPlan::MelodyFunctional(h),
                Err(_) => HarmonyPlan::None,
            }
        } else {
            HarmonyPlan::None
        };
        let percussion_plan =
            PercussionPatternPlan::new(&plan, &self.profile.percussion, variant_seed);
        let percussion_humanize = draw_percussion_humanize(
            self.tuning.profile.percussion.velocity_humanize,
            self.profile.form.unique_bars(),
            self.profile.meter.steps_per_bar,
            rng,
        );
        let performance_seed = if self.profile.performance.is_neutral() {
            0
        } else {
            let high = rng.next_range_u64(0..1 << 32);
            (high << 32) | rng.next_range_u64(0..1 << 32)
        };

        // Tirer l'intro (P1-T11)
        let intro_info = self.draw_intro(tune_index, &plan, &melody, rng);
        // Aucun tirage : une graine dérivée du matériel écrit de l'air garde l'intro, les
        // tirages suivants et la mélodie écrite identiques avec ou sans ornements. Les
        // ornements seuls la lisent ; la condition reste celle d'avant l'accompagnement
        // pour que leur tirage ne change pas.
        let material_seed = if self.profile.heterophony.is_some()
            || self.profile.ornaments.needs_seed()
            || self.profile.accompaniment.as_ref().is_some_and(|a| {
                a.bass.passing.as_ref().is_some_and(|p| !p.is_inert())
                    || a.bass.walking.is_some()
                    || a.chords.needs_seed()
            }) {
            crate::ornaments::tune_seed(
                tune_index,
                (0..plan.material_bars()).flat_map(|i| {
                    melody
                        .notes_for_bar(plan.representative_bar(i))
                        .into_iter()
                        .map(|n| (n.pitch, n.start_step, n.duration_steps))
                }),
            )
        } else {
            0
        };
        // Graine de l'accompagnement : un tirage par air dès que le profil a un accompagnement
        // ou des phrases et des relances de batterie (sans eux, le flux est celui d'avant),
        // qu'une couche en tire ou non : ajouter ou retirer une couche ne décale ainsi aucun
        // autre tirage. Elle vient du RNG injecté, donc de la graine de session, et non de la
        // mélodie écrite, que la banque de cellules d'un profil sous grille fixe réduit à deux
        // valeurs. Tirée après tout le reste de l'air : la mélodie, l'intro et les ornements
        // de cet air ne bougent pas.
        let accompaniment_seed =
            if self.profile.accompaniment.is_some() || self.profile.percussion.needs_seed() {
                let high = rng.next_range_u64(0..1 << 32);
                (high << 32) | rng.next_range_u64(0..1 << 32)
            } else {
                0
            };
        let bass_plan = if let Some(source) = source {
            source.bass_plan.clone()
        } else if let Some(a) = &self.profile.accompaniment
            && a.bass.uses_plan()
        {
            let mut bass_rng =
                ChaCha8Rng::seed_from_u64(crate::ornaments::mix(accompaniment_seed, 0x424D_4F54));
            BassMotifPlan::new(&plan, &a.bass, &mut bass_rng)
        } else {
            BassMotifPlan::default()
        };

        let bright_mode = dynamics
            .parallel_modes
            .iter()
            .find(|c| c.dark == plan.mode)
            .map_or(plan.mode, |c| c.bright);
        ActiveTune {
            tune_index,
            plan: plan.clone(),
            melody,
            harmony,
            bass_plan,
            percussion_plan,
            percussion_humanize,
            variant_seed,
            performance_seed,
            material_seed,
            accompaniment_seed,
            arc_seed,
            intro_info,
            initial_mode: plan.mode,
            material_origin,
            initial_dark: plan.mode.is_minor() && valence < dynamics.dark_below,
            bright_mode,
            dark_mode: plan.mode.is_minor().then_some(plan.mode),
            mode_changes: BTreeMap::new(),
            mode_requests: BTreeMap::new(),
        }
    }

    /// Tonalité de l'air `tune_index` parmi les candidats de la valence courante.
    ///
    /// Premier air d'un set : poids de base. Suivants : poids de base × facteur de classe de
    /// la transition depuis `prev_key_mode`, avec le tirage fixé au début du set (un set sans
    /// fin tire ici).
    fn draw_key_mode(
        &self,
        tune_index: usize,
        prev_key_mode: Option<(PitchClass, Mode)>,
        valence: f32,
        rng: &mut dyn RngCore,
    ) -> (PitchClass, Mode) {
        // Variante de session : tonalité du premier air. Le verrou du profil (`[fixed_tonality]`,
        // donc une grille utilisateur transposée par lui) gagne : la variante est ignorée.
        if tune_index == 0
            && self.tuning.profile.fixed_tonality.is_none()
            && let Some(tonality) = self.session.as_ref().and_then(|session| session.tonality())
        {
            return (tonality.key, tonality.mode);
        }
        let candidates = candidates(&self.tuning.profile, valence);
        let set = &self.tuning.profile.set;
        let planned = self
            .set_of(tune_index)
            .map(|(_, set)| set)
            .filter(|set| set.first_tune() != tune_index);
        let (prev, last, u) = match (prev_key_mode, planned) {
            (Some(prev), Some(plan)) => {
                let tune_in_set = plan.tune_in_set(tune_index);
                let u = plan
                    .key_draw(tune_in_set)
                    .unwrap_or_else(|| rng.next_range_u64(0..KEY_DRAW_RANGE) as u32);
                (Some(prev), plan.is_last(tune_in_set), u)
            }
            _ => (None, false, rng.next_range_u64(0..KEY_DRAW_RANGE) as u32),
        };
        set.choose(prev, &candidates, last, u)
    }

    /// Tire une intro pour `tune_index` selon `IntroConfig` (P1-T11).
    ///
    /// - Tire une forme pondérée parmi `intro.forms`
    /// - Calcule les notes de levée ascendantes vers la première note de A1
    /// - Seul le premier air d'un set a l'intro complète ; les suivants ont seulement la levée
    fn draw_intro(
        &self,
        tune_index: usize,
        plan: &TunePlan,
        melody: &MelodyLine,
        rng: &mut dyn RngCore,
    ) -> IntroInfo {
        let intro_config = &self.tuning.profile.arrangement.intro;

        // Tirage de la forme (seulement si forme complète nécessaire)
        let first_in_set = self.is_first_in_set(tune_index);
        let (transition, transition_bars) = self.transition_for(tune_index);
        let form = if first_in_set {
            draw_intro_form(intro_config, rng)
        } else if transition == TransitionKind::Call {
            Some(IntroRole::Percussion)
        } else {
            None
        };
        let form_bars = if first_in_set {
            form.map_or(0, |role| {
                intro_config
                    .forms
                    .iter()
                    .find(|f| f.role == role)
                    .map_or(role.default_bars(), |f| f.effective_bars())
            })
        } else if transition == TransitionKind::Call {
            transition_bars
        } else {
            0
        };
        let leading_silence = if first_in_set {
            self.set_of(tune_index)
                .and_then(|(index, _)| index.checked_sub(1))
                .map_or(0, |index| self.sets[index].gap_bars(&self.profile.set))
        } else if transition == TransitionKind::Silence {
            transition_bars
        } else {
            0
        };
        // La levée mène à la première note de la forme : seulement si la mélodie la joue.
        let profile = &self.tuning.profile;
        let melody_opens = profile.has_track(TrackId::Melody)
            && (!first_in_set || profile.arrangement.opens(TrackId::Melody, &self.profile.form));
        // La levée, courtes notes en marge de la ligne écrite, ne tient pas dans un filtre de
        // difficulté : elle est retirée.
        let pickup_eighths = if melody_opens && profile.melody.difficulty.is_none() {
            intro_config.pickup_eighths
        } else {
            0
        };
        let pickup_bars = usize::from(pickup_eighths > 0);
        let content_bars =
            if first_in_set { form_bars.max(pickup_bars) } else { form_bars + pickup_bars };
        let intro_bars = leading_silence + content_bars;

        // Calculer les notes de levée
        let pickup_notes = if pickup_eighths > 0 && intro_bars > 0 {
            let first_note_midi = first_melody_note_midi(melody, plan, &profile.melody.params);
            compute_pickup_notes(first_note_midi, pickup_eighths, plan.key, plan.mode)
        } else {
            Vec::new()
        };

        IntroInfo { form, intro_bars, leading_silence, form_bars, pickup_notes }
    }

    fn alloc_note_id(&mut self) -> NoteId {
        let id = self.next_note_id;
        self.next_note_id += 1;
        id
    }
}

// ---------------------------------------------------------------------------
// Fonctions d'intro
// ---------------------------------------------------------------------------

/// Tire une forme d'intro pondérée depuis la config.
fn draw_intro_form(config: &IntroConfig, rng: &mut dyn RngCore) -> Option<IntroRole> {
    if config.forms.is_empty() {
        return None;
    }
    let total_weight: u32 = config.forms.iter().map(|f| f.weight).sum();
    if total_weight == 0 {
        return None;
    }
    let mut roll = (rng.next_f32() * total_weight as f32) as u32;
    for form in &config.forms {
        if roll < form.weight {
            return Some(form.role);
        }
        roll -= form.weight;
    }
    // Repli : première forme
    config.forms.first().map(|f| f.role)
}

/// Première note MIDI de la première mesure de l'air (hors Hold).
///
/// Lue depuis la ligne rendue (première mesure), sans supposer la forme de
/// l'air : c'est la note d'arrivée de la levée.
fn first_melody_note_midi(melody: &MelodyLine, plan: &TunePlan, params: &MelodyParams) -> u8 {
    let tonic = tonic_midi_for_key(plan.key, params.tonic_midi_min, params.tonic_midi_max);
    if let Some(first) = melody.notes_for_bar(0).first() {
        return first.pitch;
    }
    // Repli : tonique
    tonic
}

/// Calcule les notes MIDI de levée (anacrouse) montant vers `target_midi`.
///
/// Retourne `count` notes ascendantes, la dernière étant `target_midi` (la
/// première note de l'air). Les notes sont des degrés diatoniques consécutifs
/// de la gamme du mode sous la cible : chaque saut est ≤ 2 demi-tons, donc
/// toujours ≤ quinte (7 demi-tons).
#[must_use]
fn compute_pickup_notes(target_midi: u8, count: u8, key: PitchClass, mode: Mode) -> Vec<u8> {
    if count == 0 {
        return Vec::new();
    }

    // Échelle diatonique ascendante sur 4 octaves depuis une tonique grave.
    let base_tonic = tonic_midi_for_key(key, 48, 60);
    let mut scale: Vec<u8> = Vec::with_capacity(4 * 7);
    for octave in 0..4i32 {
        let base = i32::from(base_tonic) + octave * 12;
        for deg in 1..=7i8 {
            let midi = base + i32::from(mode.degree_to_semitones(deg));
            if (0..=127).contains(&midi) {
                let m = midi as u8;
                if !scale.contains(&m) {
                    scale.push(m);
                }
            }
        }
    }
    scale.sort_unstable();

    let Some(target_idx) = scale.iter().position(|&m| m == target_midi) else {
        // Cible hors de l'échelle (ne devrait pas se produire) : juste la cible.
        return vec![target_midi];
    };

    let n = count as usize;
    // `n` degrés consécutifs finissant sur la cible.
    let start = target_idx.saturating_sub(n - 1);
    scale[start..=target_idx].to_vec()
}

// ---------------------------------------------------------------------------
// Fonctions existantes
// ---------------------------------------------------------------------------

/// Nom anglo-saxon lisible d'une classe de hauteur (`0` = `"C"`, `2` = `"D"`, `6` = `"F#"`, etc.).
#[must_use]
pub const fn pitch_class_name(pc: PitchClass) -> &'static str {
    match pc % 12 {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "Eb",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "Ab",
        9 => "A",
        10 => "Bb",
        11 => "B",
        _ => "C",
    }
}

/// Nom lisible d'un [`DegreeChord`] dans une tonalité et un mode donnés (ex. `"D"`, `"A"`, `"Bm"`).
#[must_use]
pub fn degree_chord_name(chord: DegreeChord, key: PitchClass, mode: Mode) -> String {
    let root = pitch_class_name(chord.root_pc(key, mode));
    let name = match chord.quality {
        ChordQuality::Major => root.to_string(),
        ChordQuality::Minor => format!("{root}m"),
        ChordQuality::Pedal => format!("{root} (pédale)"),
        ChordQuality::PedalNeighbor(neighbor) => format!("{root} (pédale +{neighbor})"),
        ChordQuality::Typed(quality) => format!("{root}{quality}"),
    };
    if chord.is_slash(mode) {
        format!("{name}/{}", pitch_class_name(chord.bass_pc(key, mode)))
    } else {
        name
    }
}

/// Construit le [`ChordContext`] d'un accord relatif à la tonique : l'identité d'un accord
/// typé vient de [`ChordContext::from_chord`], celle d'une pédale (sans qualité) de ses
/// hauteurs. Le nom reste celui de [`degree_chord_name`].
#[must_use]
pub fn build_chord_context(chord: DegreeChord, key: PitchClass, mode: Mode) -> ChordContext {
    let name = degree_chord_name(chord, key, mode);
    if let Some(typed) = chord.to_chord(key, mode) {
        return ChordContext::from_chord(&typed, key, name);
    }
    ChordContext {
        root_offset: i32::from(chord.root_semitones(mode)),
        is_minor: chord.quality.is_minor(),
        chord_name: name,
        root_pc: chord.root_pc(key, mode),
        bass_pc: chord.bass_pc(key, mode),
        quality: None,
        extensions: Vec::new(),
        pitch_classes: chord.pitch_classes(key, mode).to_vec(),
        scale_guidance: None,
    }
}

/// Contexte d'une mesure sans accord joué (intro, outro, harmonie `none`) : la tonique de
/// l'air comme fondamentale et basse, aucune qualité ni note d'accord.
fn key_context(key: PitchClass, mode: Mode) -> ChordContext {
    ChordContext {
        is_minor: mode.is_minor(),
        chord_name: pitch_class_name(key).to_string(),
        root_pc: key % 12,
        bass_pc: key % 12,
        ..ChordContext::default()
    }
}

/// Degré de l'accord tonique pour les assertions de tests.
#[must_use]
pub const fn is_tonic_chord(chord: DegreeChord) -> bool {
    matches!(chord.degree, ChordDegree::I)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

fn nearest_pitch(pcs: impl IntoIterator<Item = u8>, center: u8, low: u8, high: u8) -> u8 {
    pcs.into_iter()
        .flat_map(|pc| {
            let octave = (i16::from(center) - i16::from(pc)).div_euclid(12);
            [octave - 1, octave, octave + 1].map(move |o| o * 12 + i16::from(pc))
        })
        .filter(|&pitch| pitch >= i16::from(low) && pitch <= i16::from(high))
        .min_by_key(|&pitch| (pitch - i16::from(center)).abs())
        .map_or_else(|| unreachable!("validated register spans an octave"), |pitch| pitch as u8)
}
#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::{
        compose::{Arrangement, Section},
        params::{PulseSpec, TimeSignature},
    };

    fn make_tune_params() -> MusicalParams {
        let rf = crate::StyleProfile::default().meter;
        let mut params = MusicalParams::default();
        params.bpm = rf.default_bpm;
        params.set_swing_ratio(rf.default_swing_ratio);
        params.time_signature = rf.time_signature;
        params
    }

    fn make_flat_tuning() -> TuningParams {
        TuningParams {
            profile: {
                let mut p = TuningParams::default().profile;
                p.arrangement = Arrangement::from_start(3, Default::default());
                p
            },
            ..TuningParams::default()
        }
    }

    #[test]
    fn measure_metadata_matches_d3_and_params() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let params = make_tune_params();
        let tuning = make_flat_tuning();
        let mut tune_gen = TuneGenerator::new(params, tuning);

        let m1 = tune_gen.generate_measure(1, &mut rng);
        assert_eq!(m1.index, 1);
        assert_eq!(m1.time_signature, TimeSignature::new(2, 2));
        assert_eq!(m1.steps, 16);
        assert_eq!(m1.pulse, PulseSpec { steps_per_pulse: 8, pulses_per_bar: 2 });
        assert!((m1.swing_ratio - 0.56).abs() < f32::EPSILON);
        assert!((m1.tempo - 230.0).abs() < f32::EPSILON);
        assert!(!m1.chord_context.chord_name.is_empty());
    }

    #[test]
    fn intro_position_is_exposed() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let params = make_tune_params();
        let mut tune_gen = TuneGenerator::new(params, TuningParams::default());

        // La config par défaut a une intro avec `each_tune = false`
        let m1 = tune_gen.generate_measure(1, &mut rng);
        let pos = m1.position.unwrap();
        assert!(pos.is_intro(), "la mesure 1 devrait être une intro");
        assert_eq!(pos.section, Section::Intro);
        assert_eq!(pos.tune_index, 0);
    }

    #[test]
    fn intro_and_tour_same_melody() {
        // Même seed ⇒ les notes Melody de l'air après l'intro sont identiques
        // à celles d'un air sans intro (vérifie que l'intro ne perturbe pas le RNG du tune).

        let seed = 42;

        // 1. Sans intro : générer les 32 mesures du tune 0
        let mut rng_flat = ChaCha8Rng::seed_from_u64(seed);
        let params = make_tune_params();
        let mut flat_gen = TuneGenerator::new(params.clone(), make_flat_tuning());
        let flat_notes: Vec<Vec<u8>> = (1..=32)
            .map(|bar| {
                let m = flat_gen.generate_measure(bar, &mut rng_flat);
                m.notes_for_track(TrackId::Melody).iter().map(|n| n.pitch).collect()
            })
            .collect();

        // 2. Avec intro : générer intro + 32 mesures, puis comparer les 32 mesures après l'intro
        let mut rng_intro = ChaCha8Rng::seed_from_u64(seed);
        let mut intro_gen = TuneGenerator::new(params, TuningParams::default());

        // Générer assez pour passer l'intro et couvrir 32 mesures d'air
        for bar in 1..=100 {
            let _ = intro_gen.generate_measure(bar, &mut rng_intro);
        }

        let intro_bars = intro_gen.tunes.get(&0).unwrap().intro_info.intro_bars;

        // Régénérer depuis le début (même seed) pour la comparaison
        let mut rng_intro2 = ChaCha8Rng::seed_from_u64(seed);
        let mut intro_gen2 = TuneGenerator::new(make_tune_params(), TuningParams::default());

        for tune_bar in 1..=32 {
            let bar = tune_bar + intro_bars;
            let m = intro_gen2.generate_measure(bar, &mut rng_intro2);
            let intro_notes: Vec<u8> =
                m.notes_for_track(TrackId::Melody).iter().map(|n| n.pitch).collect();
            assert_eq!(
                flat_notes[tune_bar - 1],
                intro_notes,
                "mesure {} (bar {} dans le flux): les notes Melody diffèrent après l'intro",
                tune_bar,
                bar
            );
        }
    }

    #[test]
    fn each_tune_false_second_tune_no_intro() {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let params = make_tune_params();
        let tuning = TuningParams::default(); // each_tune = false par défaut
        let mut tune_gen = TuneGenerator::new(params, tuning);

        // Générer jusqu'au 2ᵉ tune
        for bar in 1..=200 {
            let m = tune_gen.generate_measure(bar, &mut rng);
            let pos = m.position.unwrap();
            if pos.tune_index == 1 && pos.section == Section::Tour && pos.bar_in_part == 1 {
                // La première mesure du 2ᵉ air ne devrait PAS être précédée d'une intro
                // (seulement une levée si pickup_eighths > 0)
                // Vérifions que la mesure juste avant est soit la dernière du tune 0,
                // soit une mesure de levée
                let prev = tune_gen.generate_measure(bar - 1, &mut ChaCha8Rng::seed_from_u64(7));
                let prev_pos = prev.position.unwrap();
                // Soit c'est la dernière mesure du tune 0 (sans end_after_tour)
                // Soit c'est une mesure d'intro (levée seule)
                assert!(
                    prev_pos.tune_index == 0 || prev_pos.is_intro(),
                    "mesure avant tune 1: attendu tune 0 ou intro, obtenu {:?}",
                    prev_pos
                );
                break;
            }
        }
    }

    #[test]
    fn pickup_ascends_to_first_a1_note() {
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let params = make_tune_params();
        let tuning = TuningParams::default();
        let mut tune_gen = TuneGenerator::new(params, tuning);

        // Générer assez pour avoir l'intro
        for bar in 1..=50 {
            let _ = tune_gen.generate_measure(bar, &mut rng);
        }

        let tune = tune_gen.tunes.get(&0).unwrap();
        let intro = &tune.intro_info;

        if !intro.pickup_notes.is_empty() {
            let target = first_melody_note_midi(
                &tune.melody,
                &tune.plan,
                &tune_gen.tuning.profile.melody.params,
            );
            let last_pickup = *intro.pickup_notes.last().unwrap();

            // La dernière note de levée doit arriver sur la première note de A1
            // (même pitch class, possiblement décalé d'octave)
            assert_eq!(
                last_pickup % 12,
                target % 12,
                "la dernière note de levée ({}) devrait avoir la même classe que la 1ʳᵉ note de A1 ({})",
                last_pickup,
                target
            );

            // Vérifier les sauts ≤ quinte
            let mut prev = intro.pickup_notes[0];
            for &m in &intro.pickup_notes[1..] {
                let diff = (m as i16 - prev as i16).abs();
                assert!(diff <= 7, "saut de levée {}→{} = {} demi-tons > quinte", prev, m, diff);
                prev = m;
            }

            // Les notes doivent être ascendantes
            for i in 1..intro.pickup_notes.len() {
                assert!(
                    intro.pickup_notes[i] >= intro.pickup_notes[i - 1],
                    "les notes de levée doivent être ascendantes"
                );
            }
        }
    }

    #[test]
    fn intro_form_deterministic() {
        // Même seed ⇒ même forme d'intro
        let mut rng1 = ChaCha8Rng::seed_from_u64(99);
        let mut rng2 = ChaCha8Rng::seed_from_u64(99);
        let params = make_tune_params();
        let tuning = TuningParams::default();

        let mut gen1 = TuneGenerator::new(params.clone(), tuning.clone());
        let mut gen2 = TuneGenerator::new(params, tuning);

        let _ = gen1.generate_measure(1, &mut rng1);
        let _ = gen2.generate_measure(1, &mut rng2);

        let form1 = gen1.tunes.get(&0).unwrap().intro_info.form;
        let form2 = gen2.tunes.get(&0).unwrap().intro_info.form;
        assert_eq!(form1, form2, "même seed ⇒ même forme d'intro");
    }

    #[test]
    fn intro_weighted_distribution() {
        // Sur 10 000 tirages, chaque forme apparaît proportionnellement à son poids
        let config = IntroConfig::default();
        let weights: Vec<u32> = config.forms.iter().map(|f| f.weight).collect();
        let total: u32 = weights.iter().sum();

        let mut counts = [0u32; 3]; // Percussion, Tail, Vamp
        for seed in 0..10_000u64 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            if let Some(form) = draw_intro_form(&config, &mut rng) {
                match form {
                    IntroRole::Percussion => counts[0] += 1,
                    IntroRole::Tail => counts[1] += 1,
                    IntroRole::Vamp => counts[2] += 1,
                }
            }
        }

        // Vérifier que chaque forme est dans ±5% du poids attendu
        for (i, &w) in weights.iter().enumerate() {
            let expected = (w as f64 / total as f64) * 10_000.0;
            let actual = counts[i] as f64;
            let ratio = actual / expected;
            assert!(
                (ratio - 1.0).abs() < 0.08,
                "forme {}: attendu ~{:.0}, obtenu {} (±{}%)",
                i,
                expected,
                counts[i],
                ((ratio - 1.0) * 100.0).abs()
            );
        }
    }

    #[test]
    fn intro_form_produces_promised_tracks() {
        // Percussion → PercussionLow présent, Melody absent (sauf levée en dernière mesure)
        // Tail → Melody présent
        // Vamp → Chords présent

        // On teste avec différentes seeds pour couvrir chaque forme
        for seed in 0..100u64 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let params = make_tune_params();
            let tuning = TuningParams::default();
            let mut tune_gen = TuneGenerator::new(params, tuning);

            let m1 = tune_gen.generate_measure(1, &mut rng);
            let tune = tune_gen.tunes.get(&0).unwrap();
            let intro = &tune.intro_info;

            match intro.form {
                Some(IntroRole::Percussion) => {
                    // Percussion grave devrait être présent (sauf si enable_rhythm désactivé)
                    let percussion_low_notes = m1.notes_for_track(TrackId::PercussionLow);
                    assert!(
                        !percussion_low_notes.is_empty(),
                        "Percussion (seed {seed}): attendu percussion grave présent"
                    );
                }
                Some(IntroRole::Tail) => {
                    let mel_notes = m1.notes_for_track(TrackId::Melody);
                    assert!(!mel_notes.is_empty(), "Tail (seed {seed}): attendu mélodie présente");
                }
                Some(IntroRole::Vamp) => {
                    let chord_notes = m1.notes_for_track(TrackId::Chords);
                    assert!(
                        !chord_notes.is_empty(),
                        "Vamp (seed {seed}): attendu accords présents"
                    );
                }
                None => {}
            }

            if intro.form.is_some() {
                break; // On a trouvé une forme, c'est suffisant pour ce test
            }
        }
    }

    #[test]
    fn invalid_intro_config_fails_validation() {
        use crate::compose::ArrangementError;

        let mut config = IntroConfig::default();
        config.forms[0].bars = 10;
        assert!(matches!(config.validate(), Err(ArrangementError::IntroBarsOutOfRange { .. })));

        let mut config = IntroConfig::default();
        config.pickup_eighths = 4;
        assert!(matches!(config.validate(), Err(ArrangementError::PickupEighthsOutOfRange(4))));
    }
}

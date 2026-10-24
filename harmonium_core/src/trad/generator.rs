//! Génération traditionnelle depuis le profil actif : grille, forme, cellules,
//! harmonie, accompagnement, percussions, intros et dynamique émotionnelle.
//! Les airs sont construits sur le thread principal ; les reprises relisent
//! les mêmes cellules et tirages ChaCha8, sans données musicales implicites.

use std::{borrow::Cow, collections::BTreeMap, sync::Arc};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::{
    accomp::{Guitarist, bass_bar},
    arrangement::{IntroConfig, IntroInfo, IntroRole, TunePosition},
    cells::Cell,
    feet::{draw_feet_humanize, feet_notes, feet_notes_with_cadence},
    fiddle::{FiddleLine, same_cell_up_to_octave, tonic_midi_for_key},
    harmony::{ChordDegree, ChordQuality, TradChord, TuneHarmony},
    mode::Mode,
    plan::{BarRole, ImposedRole, TunePlan},
};
use crate::{
    algo::RngCore,
    models::{
        HarmonyPlan,
        jazz::{GrammarStyle, SteedmanGrammar},
    },
    ornaments::BarContext,
    params::MusicalParams,
    theory::{PitchClass, chord::Chord, lydian_chromatic::LydianChromaticConcept},
    timeline::{
        Articulation, ChordChange, ChordContext, Measure, NoteId, StateSnapshot, TimelineNote,
        TrackId,
    },
    tuning::{ReelFiddleParams, TuningParams},
};

// ---------------------------------------------------------------------------
// ActiveTune
// ---------------------------------------------------------------------------

/// État d'un air généré par [`TradGenerator`] (rejoué `tours_per_tune` tours).
#[derive(Clone, Debug, PartialEq)]
pub struct ActiveTune {
    /// Indice 0-based de l'air dans la session.
    pub tune_index: usize,
    /// Plan structurel du tune (tonalité, mode, type, schéma AABB).
    pub plan: TunePlan,
    /// Matériel mélodique des parties, passes et tours configurés.
    pub fiddle: FiddleLine,
    /// Modèle harmonique sélectionné par le profil, précalculé par mesure.
    pub harmony: HarmonyPlan,
    /// Décalages d'humanisation des vélocités des pieds, par position de mesure
    /// de chaque partie et par pas. Tirés une fois par air, relus aux reprises.
    pub feet_humanize: Vec<Vec<i8>>,
    /// Graine des tirages d'interprétation de l'air, tirée une fois (0 si le
    /// profil n'a aucune interprétation) ; un flux par mesure en dérive.
    pub performance_seed: u64,
    /// Graine dérivée du matériel écrit, commune aux ornements et à l'accompagnement.
    pub material_seed: u64,
    /// Informations d'intro pour cet air (P1-T11).
    pub intro_info: IntroInfo,
    initial_mode: Mode,
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
// TradGenerator
// ---------------------------------------------------------------------------

/// Générateur de mesures piloté par le profil actif.
#[derive(Clone, Debug)]
pub struct TradGenerator {
    /// Paramètres musicaux courants (tempo, swing, densité des pieds, activations).
    pub musical_params: MusicalParams,
    /// Paramètres de style et banques de données (cellules, podorythmie, harmonie).
    pub tuning: TuningParams,
    /// Airs générés indexés par `tune_index` (`0` = mesures 1..=32×tours, etc.).
    tunes: BTreeMap<usize, Arc<ActiveTune>>,
    profile: Arc<crate::style_profile::StyleProfile>,
    /// Compteur monotone d'identifiants de notes.
    next_note_id: NoteId,
    /// Dernier indice de mesure généré.
    current_bar: usize,
}

impl TradGenerator {
    /// Construit un nouveau [`TradGenerator`] avec les paramètres et le tuning donnés.
    #[must_use]
    pub fn new(musical_params: MusicalParams, tuning: TuningParams) -> Self {
        Self {
            musical_params,
            profile: Arc::new(tuning.profile.clone()),
            tuning,
            tunes: BTreeMap::new(),
            next_note_id: 1,
            current_bar: 0,
        }
    }

    /// Met à jour les paramètres musicaux (`bpm`, `swing_ratio`, `feet_density`, etc.).
    pub fn update_params(&mut self, params: MusicalParams) {
        self.musical_params = params;
    }

    /// Garde les cellules et les décisions déjà audibles ; retire uniquement
    /// les décisions de mode et les airs encore futurs.
    pub fn invalidate_future(&mut self, playing_bar: usize) {
        let current_tune = self.position_of(playing_bar).tune_index;
        self.tunes.retain(|&index, _| index <= current_tune);
        for tune in self.tunes.values_mut() {
            let tune = Arc::make_mut(tune);
            tune.mode_changes.retain(|&bar, _| bar <= playing_bar);
            tune.mode_requests.retain(|&bar, _| bar <= playing_bar);
        }
    }

    fn feet_density(&self) -> u8 {
        let p = &self.musical_params;
        let d = &self.tuning.profile.dynamics;
        (f32::from(p.feet_density)
            + d.feet_intensity.eval(p.intensity)
            + d.density_feet.eval(p.rhythm_density))
        .round()
        .clamp(0.0, 2.0) as u8
    }

    /// Met à jour les paramètres de tuning et invalide le cache de tunes afin que
    /// les nouvelles banques/constantes prennent effet dès la prochaine mesure.
    pub fn update_tuning(&mut self, tuning: TuningParams) {
        if self.tuning != tuning {
            self.profile = Arc::new(tuning.profile.clone());
            self.tuning = tuning;
            self.tunes.clear();
        }
    }

    /// Réinitialise le générateur à son état initial pour le seek déterministe.
    ///
    /// Conserve `musical_params`, `tuning` et le compteur monotone `next_note_id`,
    /// et vide les tunes générés afin que le rejeu depuis la mesure 1 reproduise
    /// exactement la séquence de tirages sur le RNG.
    pub fn reset_to_initial(&mut self) {
        self.tunes.clear();
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
    /// mesure → `(section, tune_index, tour, part, pass, bar_in_strain)`).
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
                return position;
            }

            offset += intro_bars + cycle;
            tune_idx += 1;
        }
    }

    /// Vrai si `track` joue à `position` : l'arrangement l'autorise (ou la piste a
    /// joué l'intro de cet air) ET le drapeau `enable_*` du jeu le permet. Seul
    /// calcul de « piste active ».
    fn track_active(&self, track: TrackId, position: &TunePosition) -> bool {
        if !self.tuning.profile.has_track(track) {
            return false;
        }
        let flag = match track {
            TrackId::Melody => self.musical_params.enable_melody,
            TrackId::Chords | TrackId::Bass => self.musical_params.enable_harmony,
            TrackId::Heel | TrackId::Toe => self.musical_params.enable_rhythm,
        };
        let entered = self.tuning.profile.arrangement.allows(
            track,
            position,
            self.is_first_in_set(position.tune_index),
            &self.profile.form,
        ) || self.played_intro(track, position.tune_index);
        flag && entered
            && self.tuning.profile.dynamics.allows(track, self.musical_params.rhythm_density)
    }

    /// Vrai si l'air `tune_index` ouvre un set : intro complète et entrées progressives.
    fn is_first_in_set(&self, tune_index: usize) -> bool {
        tune_index == 0 || self.profile.set.is_one_tune_per_set()
    }

    /// Vrai si `track` a joué l'intro de l'air `tune_index` : une piste qui a
    /// lancé l'air (décompte aux pieds, vamp) continue au lieu de se taire quand
    /// le violon entre (écoute 2026-10-22).
    fn played_intro(&self, track: TrackId, tune_index: usize) -> bool {
        self.tunes.get(&tune_index).is_some_and(|tune| {
            let intro = &tune.intro_info;
            match track {
                TrackId::Melody => intro.melody_active(),
                TrackId::Chords | TrackId::Bass => intro.chords_active(),
                TrackId::Heel | TrackId::Toe => intro.feet_active(),
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
                TrackId::Chords | TrackId::Bass => {
                    intro_info.chords_active() && self.musical_params.enable_harmony
                }
                TrackId::Heel | TrackId::Toe => {
                    intro_info.feet_active() && self.musical_params.enable_rhythm
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
        let tempo = self
            .tuning
            .profile
            .dynamics
            .tempo_for(self.musical_params.bpm, self.musical_params.intensity);

        let mut measure = Measure::new(bar_index, time_sig, tempo, steps);
        measure.pulse = self.profile.meter.pulse_spec();
        measure.position = Some(position);
        measure.swing_ratio = self.musical_params.swing_ratio;
        measure.state_snapshot = StateSnapshot {
            bpm: tempo,
            density: self.musical_params.rhythm_density,
            tension: self.musical_params.harmony_tension,
            smoothness: self.musical_params.melody_smoothness,
            valence: self.musical_params.harmony_valence,
            arousal: self.musical_params.arousal,
        };

        let Some(tune) = self.tunes.get(&position.tune_index).cloned() else {
            return measure;
        };

        let is_intro = position.is_intro();
        let is_outro = self.musical_params.end_after_tour
            && !is_intro
            && position.is_last_tour()
            && position.bar_in_tour() + 1 == self.profile.form.bars_per_tour();

        if is_intro {
            self.fill_intro_measure(&tune, &position, &mut measure);
        } else if is_outro {
            self.fill_outro_measure(&tune, &position, &mut measure);
        } else {
            self.fill_tour_measure(&tune, &position, &mut measure);
        }
        if !is_intro && !is_outro {
            self.apply_bass_passing(&mut measure, &tune, &position);
        }

        let written: Vec<usize> = if self.profile.ornaments.is_inert() {
            Vec::new()
        } else {
            measure.notes_for_track(TrackId::Melody).iter().map(|n| n.duration_steps).collect()
        };
        self.apply_dynamics(&mut measure);
        self.fit_jazz_melody(&mut measure, &tune, &position);
        if !is_intro && !self.profile.performance.is_neutral() {
            let offset = position.bar_in_strain - 1;
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
                self.profile.dynamics.chop_intensity.eval(self.musical_params.intensity)
                    + self.profile.dynamics.density_chop.eval(self.musical_params.rhythm_density)
            };
            let seed =
                crate::ornaments::mix(tune.material_seed, self.performed_bar(&position) as u64);
            super::strum::apply(&mut measure, &a.chords, &a.chords.pattern(offset).hits, seed);
        }
        if !is_intro && !is_outro && !self.profile.ornaments.is_inert() {
            self.apply_ornaments(&mut measure, &tune, &position, &written);
        }
        if !is_intro
            && !is_outro
            && let Some(a) = &self.profile.accompaniment
        {
            super::bass_passing::veto_final(&mut measure, &a.bass);
        }

        measure
    }

    fn apply_bass_passing(
        &mut self,
        measure: &mut Measure,
        tune: &ActiveTune,
        position: &TunePosition,
    ) {
        let Some(a) = &self.profile.accompaniment else {
            return;
        };
        if a.bass.passing.as_ref().is_none_or(|p| p.is_inert())
            || !matches!(tune.harmony, HarmonyPlan::TradFunctional(_))
        {
            return;
        }
        let bar = self.performed_bar(position);
        let chords = self.chords_for_bar(tune, bar);
        let next_target = a
            .bass
            .hits
            .first()
            .filter(|_| bar + 1 < self.profile.form.bars_per_tour() * position.tours)
            .map(|hit| {
                let next = self.chords_for_bar(tune, bar + 1);
                (
                    hit.step,
                    super::accomp::bass_hit_pitch(
                        &next,
                        tune.plan.key,
                        tune.plan.mode,
                        a,
                        hit,
                        self.profile.harmony.steps_per_chord,
                    ),
                )
            });
        let (start, end) = self.phrase_window(tune, position);
        let span = (end - start).max(1) as f32;
        let context = super::bass_passing::PassingContext {
            key: tune.plan.key,
            mode: tune.plan.mode,
            seed: tune.material_seed,
            bar,
            phase: (position.bar_in_strain - 1 - start) as f32 / span,
            phrase_bars: span,
            density: self.musical_params.rhythm_density,
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
        let offset = position.bar_in_strain - 1;
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
            && (position.bar_in_strain == 1
                || (matches!(
                    dynamics.mode_shift_boundary,
                    super::dynamics::ModeShiftBoundary::Phrase
                ) && self.profile.form.phrase_end(
                    position.part.0,
                    position.bar_in_strain - 2,
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
            tune.fiddle.set_mode(mode);
            if matches!(tune.harmony, HarmonyPlan::TradFunctional(_)) {
                let melody: Vec<_> = (0..tune.plan.material_bars())
                    .map(|i| tune.fiddle.notes_for_bar(tune.plan.representative_bar(i)))
                    .collect();
                if let Some(accompaniment) = &self.tuning.profile.accompaniment
                    && let Ok(h) = TuneHarmony::new(tune.plan.clone(), &melody, accompaniment)
                {
                    tune.harmony = HarmonyPlan::TradFunctional(h);
                }
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

        measure.chord_context = ChordContext {
            root_offset: 0,
            is_minor: plan.mode.is_minor(),
            chord_name: pitch_class_name(plan.key).to_string(),
            scale_guidance: None,
        };

        // Indice (0-based) de la mesure d'intro dans cette intro : les mesures
        // d'intro sont générées séquentiellement, la position ne stockant pas
        // cet indice.
        let intro_bar_idx = self.current_bar - self.intro_start_bar_for_tune(position.tune_index);

        match intro.form {
            Some(IntroRole::Percussion) => {
                // Décompte : les pistes de percussion jouent leur pattern seul.
                let default_intro_tracks = [TrackId::Heel];
                let intro_tracks = tuning
                    .profile
                    .percussion
                    .intro_tracks
                    .as_deref()
                    .unwrap_or(&default_intro_tracks);
                let heel_wanted = intro_tracks.contains(&TrackId::Heel)
                    && self.track_active_in_intro(TrackId::Heel, intro);
                let toe_wanted = intro_tracks.contains(&TrackId::Toe)
                    && self.track_active_in_intro(TrackId::Toe, intro);
                if heel_wanted || toe_wanted {
                    let zero_humanize = vec![0i8; steps];
                    let (heel_notes, toe_notes) = feet_notes(
                        &tuning.profile.percussion,
                        self.feet_density(),
                        steps,
                        &zero_humanize,
                    );
                    if heel_wanted {
                        for mut note in heel_notes {
                            note.id = self.alloc_note_id();
                            measure.add_note(TrackId::Heel, note);
                        }
                    }
                    if toe_wanted {
                        for mut note in toe_notes {
                            note.id = self.alloc_note_id();
                            measure.add_note(TrackId::Toe, note);
                        }
                    }
                }
            }
            Some(IntroRole::Tail) => {
                // Entrée par la fin : la mélodie joue les `intro_bars` dernières
                // mesures de l'air (cadence finale), lues depuis la forme de l'air.
                if self.track_active_in_intro(TrackId::Melody, intro) {
                    let total_bars = plan.total_bars();
                    let tail_bars = intro.intro_bars.max(1);
                    let bar_in_tune =
                        total_bars.saturating_sub(tail_bars) + intro_bar_idx.min(tail_bars - 1);
                    for mut note in tune.fiddle.notes_for_bar(bar_in_tune) {
                        note.id = self.alloc_note_id();
                        measure.add_note(TrackId::Melody, note);
                    }
                }
            }
            Some(IntroRole::Vamp) => {
                // Vamp : l'accompagnement joue sur la tonique.
                if self.track_active_in_intro(TrackId::Chords, intro) {
                    let tonic = TradChord { degree: ChordDegree::I, quality: ChordQuality::Major };
                    let chords = vec![tonic; steps / self.profile.harmony.steps_per_chord];
                    let seed = crate::ornaments::mix(
                        tune.material_seed,
                        self.performed_bar(position) as u64,
                    );
                    let next_note_id = &mut self.next_note_id;
                    let mut next_id = || {
                        let id = *next_note_id;
                        *next_note_id += 1;
                        id
                    };
                    if let Some(accompaniment) = &tuning.profile.accompaniment {
                        let mut guitarist = Guitarist::new();
                        for note in guitarist.chop_bar(
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
    ) {
        if let HarmonyPlan::JazzFunctional(bars) = &tune.harmony {
            let slot = tune.plan.material_index(self.performed_bar(position));
            self.fill_jazz_measure(tune, position, &bars[slot], measure);
            self.fill_percussion(tune, position, measure);
            return;
        }
        if let HarmonyPlan::None = &tune.harmony {
            let bar_in_tour = self.performed_bar(position);
            let plan = &tune.plan;
            measure.chord_context = ChordContext {
                root_offset: 0,
                is_minor: plan.mode.is_minor(),
                chord_name: pitch_class_name(plan.key).to_string(),
                scale_guidance: None,
            };
            if self.track_active(TrackId::Melody, position) {
                for mut note in tune.fiddle.notes_for_bar(bar_in_tour) {
                    note.id = self.alloc_note_id();
                    measure.add_note(TrackId::Melody, note);
                }
            }
            self.fill_percussion(tune, position, measure);
            return;
        }
        let bar_in_tour = self.performed_bar(position);
        let plan = &tune.plan;
        let chords = self.chords_for_bar(tune, bar_in_tour);
        measure.chord_context = build_chord_context(chords[0], plan.key, plan.mode);

        // 1. Mélodie (violon, D6)
        if self.track_active(TrackId::Melody, position) {
            for mut note in tune.fiddle.notes_for_bar(bar_in_tour) {
                note.id = self.alloc_note_id();
                measure.add_note(TrackId::Melody, note);
            }
        }

        // 2. Accompagnement (guitare chop + basse, D8)
        if self.track_active(TrackId::Chords, position) {
            for note in self.chop_notes_for_bar(tune, bar_in_tour) {
                measure.add_note(TrackId::Chords, note);
            }
        }
        if self.track_active(TrackId::Bass, position) {
            let next_note_id = &mut self.next_note_id;
            let mut next_id = || {
                let id = *next_note_id;
                *next_note_id += 1;
                id
            };
            if let Some(accompaniment) = &self.tuning.profile.accompaniment {
                for note in bass_bar(
                    &chords,
                    plan.key,
                    plan.mode,
                    accompaniment,
                    self.profile.harmony.steps_per_chord,
                    &mut next_id,
                ) {
                    measure.add_note(TrackId::Bass, note);
                }
            }
        }
        self.fill_percussion(tune, position, measure);
    }

    fn fill_jazz_measure(
        &mut self,
        tune: &ActiveTune,
        position: &TunePosition,
        chords: &[Chord],
        measure: &mut Measure,
    ) {
        let period = self.profile.harmony.steps_per_chord;
        for (slot, chord) in chords.iter().enumerate() {
            let name = chord.name();
            let guidance = self.profile.harmony.scale_guidance.then(|| {
                LydianChromaticConcept::new().scale_guidance(
                    chord,
                    name.clone(),
                    self.musical_params.harmony_tension,
                )
            });
            measure.chord_changes.push(ChordChange {
                start_step: slot * period,
                context: ChordContext {
                    root_offset: (i32::from(chord.root) - i32::from(tune.plan.key)).rem_euclid(12),
                    is_minor: chord.chord_type.is_minor(),
                    chord_name: name,
                    scale_guidance: guidance,
                },
            });
        }
        measure.chord_context = measure.chord_changes[0].context.clone();
        if self.track_active(TrackId::Melody, position) {
            for mut note in tune.fiddle.notes_for_bar(self.performed_bar(position)) {
                note.id = self.alloc_note_id();
                measure.add_note(TrackId::Melody, note);
            }
        }
        let Some(accompaniment) = &self.tuning.profile.accompaniment else { return };
        for track in [TrackId::Chords, TrackId::Bass] {
            if !self.track_active(track, position) {
                continue;
            }
            let bass_hits = &accompaniment.bass.hits;
            let chord_hits =
                &accompaniment.chords.patterns[accompaniment.chords.default_pattern].hits;
            let count = if track == TrackId::Chords { chord_hits.len() } else { bass_hits.len() };
            for index in 0..count {
                let (step, duration, velocity) = if track == TrackId::Chords {
                    let h = &chord_hits[index];
                    (h.step, h.duration, h.velocity)
                } else {
                    let h = &bass_hits[index];
                    (h.step, h.duration, h.velocity)
                };
                let chord = &chords[step / period];
                let intervals =
                    if track == TrackId::Chords { chord.chord_type.intervals() } else { &[0] };
                for &interval in intervals {
                    let pc = (chord.root + interval) % 12;
                    let (center, low, high) = if track == TrackId::Chords {
                        (accompaniment.chords.center, 0, 127)
                    } else {
                        (accompaniment.bass.low, accompaniment.bass.low, accompaniment.bass.high)
                    };
                    let pitch = nearest_pitch([pc], center, low, high);
                    let note = TimelineNote {
                        id: self.next_note_id,
                        pitch,
                        velocity,
                        start_step: step,
                        duration_steps: duration.min(period - step % period),
                        articulation: Articulation::Normal,
                        performance: Default::default(),
                    };
                    self.next_note_id += 1;
                    measure.add_note(track, note);
                }
            }
        }
    }

    /// Les cellules restent la source du rythme et du contour ; seules les
    /// hauteurs hors accord/gamme sont rapprochées du contexte publié.
    fn fit_jazz_melody(
        &mut self,
        measure: &mut Measure,
        tune: &ActiveTune,
        position: &TunePosition,
    ) {
        let HarmonyPlan::JazzFunctional(bars) = &tune.harmony else { return };
        if measure.chord_changes.is_empty() {
            return;
        }
        let bar = tune.plan.material_index(self.performed_bar(position));
        let chords = &bars[bar];
        let Some(notes) = measure.tracks.get_mut(&TrackId::Melody) else { return };
        let source = std::mem::take(notes);
        notes.reserve(source.len() + measure.chord_changes.len());
        let params = &self.tuning.profile.melody.params;
        for note in source {
            let end = note.start_step + note.duration_steps;
            let mut start = note.start_step;
            while start < end {
                let slot = start / self.profile.harmony.steps_per_chord;
                let change = &measure.chord_changes[slot];
                let next =
                    measure.chord_changes.get(slot + 1).map_or(measure.steps, |c| c.start_step);
                let chord = &chords[slot];
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
                    .chain(scale.iter().copied());
                let mut segment = note.clone();
                segment.pitch =
                    nearest_pitch(allowed, note.pitch, params.tessitura_min, params.tessitura_max);
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
    ) {
        // 3. Podorythmie (talon + pointe, D9)
        let heel_active = self.track_active(TrackId::Heel, position);
        let toe_active = self.track_active(TrackId::Toe, position);
        if heel_active || toe_active {
            let slot =
                self.profile.form.unique_offset(position.part.0) + position.bar_in_strain - 1;
            let humanize = &tune.feet_humanize[slot];
            let is_cadence = self.profile.form.phrase_end(
                position.part.0,
                position.bar_in_strain - 1,
                tune.plan.bar_role(self.performed_bar(position)).role,
            );
            let (heel_notes, toe_notes) = feet_notes_with_cadence(
                &self.tuning.profile.percussion,
                self.feet_density(),
                measure.steps,
                humanize,
                is_cadence,
            );
            if heel_active {
                for mut note in heel_notes {
                    note.id = self.alloc_note_id();
                    measure.add_note(TrackId::Heel, note);
                }
            }
            if toe_active {
                for mut note in toe_notes {
                    note.id = self.alloc_note_id();
                    measure.add_note(TrackId::Toe, note);
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

        measure.chord_context = ChordContext {
            root_offset: 0,
            is_minor: plan.mode.is_minor(),
            chord_name: pitch_class_name(plan.key).to_string(),
            scale_guidance: None,
        };

        // Tonique accentuée sur le pas 0 (temps 1), durée 8 pas (blanche)
        if self.musical_params.enable_melody {
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
                // Guitare : accord tonique accentué
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

        // Pieds : talon accentué
        if self.musical_params.enable_rhythm && self.tuning.profile.has_track(TrackId::Heel) {
            measure.add_note(
                TrackId::Heel,
                TimelineNote {
                    id: self.alloc_note_id(),
                    pitch: self.tuning.profile.percussion.heel_note,
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
    fn chords_for_bar<'a>(&self, tune: &'a ActiveTune, bar: usize) -> Cow<'a, [TradChord]> {
        let HarmonyPlan::TradFunctional(harmony) = &tune.harmony else {
            unreachable!("traditional voicing requires traditional harmony")
        };
        let mut chords = Cow::Borrowed(harmony.chords_for_bar(bar));
        let threshold =
            self.tuning.profile.dynamics.tension_dominant.eval(self.musical_params.harmony_tension);
        let role = tune.plan.bar_role(bar);
        let slot = usize::from(role.bar_in_strain) - 1;
        let length = self.profile.form.parts[role.strain.0].roles.len();
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

    fn apply_dynamics(&mut self, measure: &mut Measure) {
        let p = &self.musical_params;
        let d = &self.tuning.profile.dynamics;
        let duration =
            d.fiddle_duration.eval(p.intensity) * d.tension_duration.eval(p.harmony_tension);
        let strong = d.strong_accent.eval(p.intensity)
            + d.brightness_accent.eval(d.valence_input(p.harmony_valence));
        let offbeat = d.tension_offbeat.eval(p.harmony_tension);
        let chord_duration = d.tension_chord_duration.eval(p.harmony_tension);
        for (&track, notes) in &mut measure.tracks {
            let velocity = d.velocity(track, p.intensity);
            for note in notes {
                let accent = if track == TrackId::Melody {
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
                } else if track == TrackId::Toe || track == TrackId::Heel {
                    if note.pitch == 40 { offbeat.max(0.0) } else { 0.0 }
                } else {
                    0.0
                };
                note.velocity =
                    (f32::from(note.velocity) * velocity + accent).round().clamp(1.0, 127.0) as u8;
                if track == TrackId::Melody {
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
        let part = &self.profile.form.parts[position.part.0];
        let protected = part
            .ornament_protected_bars
            .as_ref()
            .map_or(position.bar_in_strain == part.roles.len(), |bars| {
                bars.contains(&position.bar_in_strain)
            });
        let (_, end) = self.phrase_window(tune, position);
        let params = &self.tuning.profile.melody.params;
        let context = BarContext {
            key: tune.plan.key,
            mode: tune.plan.mode,
            tessitura: (params.tessitura_min, params.tessitura_max),
            bar_in_part: self.copy_source_bar(tune, position),
            steps_per_pulse: self.profile.meter.steps_per_pulse,
            phrase_end_bar: position.bar_in_strain == end,
            protected,
            repeat: position.pass > 1 || position.tour > 1,
            arousal: self.musical_params.arousal,
            density: self.musical_params.rhythm_density,
            bar_index: measure.index,
            seed: tune.material_seed,
        };
        let next_note_id = &mut self.next_note_id;
        self.profile.ornaments.apply(measure, &context, written, &mut || {
            let id = *next_note_id;
            *next_note_id += 1;
            id
        });
    }

    /// Mesure de la partie dont les taux par mesure s'appliquent : une copie exacte
    /// (`CopyOf`, `CopyOfPart`) emprunte celle de sa source, pour porter les mêmes
    /// ornements qu'elle.
    fn copy_source_bar(&self, tune: &ActiveTune, position: &TunePosition) -> usize {
        let mut bar = position.bar_in_strain;
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

    /// Calcule les chops de guitare de `bar_in_tour` en rejouant le [`Guitarist`]
    /// depuis le début de la passe courante, sans hypothèse de longueur ;
    /// le matériel rejoué reste stable après invalidation émotionnelle.
    fn chop_notes_for_bar(&mut self, tune: &ActiveTune, bar_in_tour: usize) -> Vec<TimelineNote> {
        let (_, _, offset) = self.profile.form.position(bar_in_tour);
        let strain_pass_start = bar_in_tour - offset;
        let Some(harmony) = self.tuning.profile.accompaniment.as_ref() else {
            return Vec::new();
        };
        let d = &self.tuning.profile.dynamics;
        let chop_offset = d.chop_intensity.eval(self.musical_params.intensity)
            + d.density_chop.eval(self.musical_params.rhythm_density);
        let hits = &harmony.chords.pattern(chop_offset).hits;
        let mut guitarist = Guitarist::new();
        for b in strain_pass_start..bar_in_tour {
            let c = self.chords_for_bar(tune, b);
            guitarist.advance_bar(
                &c,
                tune.plan.key,
                tune.plan.mode,
                harmony,
                hits,
                self.profile.harmony.steps_per_chord,
                crate::ornaments::mix(tune.material_seed, b as u64),
            );
        }
        let chords = self.chords_for_bar(tune, bar_in_tour);
        let next_note_id = &mut self.next_note_id;
        let mut next_id = || {
            let id = *next_note_id;
            *next_note_id += 1;
            id
        };
        guitarist.chop_bar_on_hits(
            &chords,
            tune.plan.key,
            tune.plan.mode,
            harmony,
            hits,
            self.profile.harmony.steps_per_chord,
            crate::ornaments::mix(tune.material_seed, bar_in_tour as u64),
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

        loop {
            // `end_after_tour` : seul l'air 0 est généré ; les mesures suivantes
            // sont vides (le compositeur arrête la génération au-delà).
            if self.musical_params.end_after_tour && tune_idx > 0 {
                return;
            }
            if !self.tunes.contains_key(&tune_idx) {
                let prev = tune_idx.checked_sub(1).and_then(|prev_idx| self.tunes.get(&prev_idx));
                let prev_plan = prev.map(|t| t.plan.clone());
                let prev_first_cell = prev.map(|t| t.fiddle.cell_for_bar(0).clone());
                let tune = self.build_tune(tune_idx, prev_plan, prev_first_cell, rng);
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

    /// Construit un [`ActiveTune`] complet (`TunePlan` + `FiddleLine` + `TuneHarmony` + intro).
    ///
    /// Règle P1-T7 : deux tunes consécutifs ne commencent pas par la même
    /// cellule (à une transposition d'octave près) ; en cas de collision, la
    /// ligne est retirée (tirage déterministe suivant) jusqu'à 8 essais.
    fn build_tune(
        &self,
        tune_index: usize,
        prev_plan: Option<TunePlan>,
        prev_first_cell: Option<Cell>,
        rng: &mut dyn RngCore,
    ) -> ActiveTune {
        let dynamics = &self.tuning.profile.dynamics;
        let valence = self.musical_params.harmony_valence;
        let mut plan = if valence == dynamics.neutral_valence {
            match prev_plan {
                None => TunePlan::draw(rng, None, self.profile.clone()),
                Some(prev) => draw_next_tune_plan(prev, &self.tuning.profile.melody.params, rng),
            }
        } else {
            dynamics.draw_color(valence, rng, self.profile.clone())
        };
        plan.resolve_roles(rng);
        let mut fiddle_params = self.tuning.profile.melody.params.clone();
        let intensity = self.musical_params.intensity;
        fiddle_params.weight_role_run = (fiddle_params.weight_role_run as f32
            * dynamics.run_weight.eval(intensity))
        .round()
        .max(1.0) as u32;
        fiddle_params.weight_role_arp = (fiddle_params.weight_role_arp as f32
            * dynamics.arp_weight.eval(intensity))
        .round()
        .max(1.0) as u32;
        fiddle_params.weight_role_turn = (fiddle_params.weight_role_turn as f32
            * dynamics.turn_weight.eval(intensity))
        .round()
        .max(1.0) as u32;
        fiddle_params.melody_center_midi = (f32::from(fiddle_params.melody_center_midi)
            + dynamics.color_register.eval(dynamics.valence_input(valence)))
        .round()
        .clamp(f32::from(fiddle_params.tessitura_min), f32::from(fiddle_params.tessitura_max))
            as u8;
        let mut fiddle = FiddleLine::with_params(
            plan.clone(),
            &self.tuning.profile.melody.cells,
            &fiddle_params,
            rng,
        );
        if let Some(prev_first) = prev_first_cell {
            for _ in 0..self.profile.form.opening_retries {
                if !same_cell_up_to_octave(fiddle.cell_for_bar(0), &prev_first) {
                    break;
                }
                fiddle = FiddleLine::with_params(
                    plan.clone(),
                    &self.tuning.profile.melody.cells,
                    &fiddle_params,
                    rng,
                );
            }
        }

        let harmony = if self.profile.harmony.model == "jazz_functional" {
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
                            SteedmanGrammar::realize(numeral, plan.key, valence, GrammarStyle::Jazz)
                        })
                        .collect()
                })
                .collect();
            HarmonyPlan::JazzFunctional(bars)
        } else if self.profile.harmony.model == "none" {
            HarmonyPlan::None
        } else if let Some(accompaniment) = &self.tuning.profile.accompaniment {
            let distinct_melody: Vec<_> = (0..plan.material_bars())
                .map(|i| fiddle.notes_for_bar(plan.representative_bar(i)))
                .collect();
            match TuneHarmony::new(plan.clone(), &distinct_melody, accompaniment) {
                Ok(h) => HarmonyPlan::TradFunctional(h),
                Err(_) => HarmonyPlan::None,
            }
        } else {
            HarmonyPlan::None
        };
        let feet_humanize = draw_feet_humanize(
            self.tuning.profile.percussion.velocity_humanize,
            self.profile.form.unique_bars(),
            self.profile.meter.steps_per_bar,
            rng,
        );
        let performance_seed = if self.profile.performance.is_neutral() {
            0
        } else {
            let high = rng.next_range_usize(0..1 << 32) as u64;
            (high << 32) | rng.next_range_usize(0..1 << 32) as u64
        };

        // Tirer l'intro (P1-T11)
        let intro_info = self.draw_intro(tune_index, &plan, &fiddle, rng);
        // Aucun tirage : une graine dérivée du matériel écrit de l'air garde l'intro, les
        // tirages suivants et la mélodie écrite identiques avec ou sans ornements.
        let material_seed = if self.profile.ornaments.needs_seed()
            || self.profile.accompaniment.as_ref().is_some_and(|a| {
                a.bass.passing.as_ref().is_some_and(|p| !p.is_inert()) || a.chords.needs_seed()
            }) {
            crate::ornaments::tune_seed(
                tune_index,
                (0..plan.material_bars()).flat_map(|i| {
                    fiddle
                        .notes_for_bar(plan.representative_bar(i))
                        .into_iter()
                        .map(|n| (n.pitch, n.start_step, n.duration_steps))
                }),
            )
        } else {
            0
        };

        let bright_mode = dynamics
            .parallel_modes
            .iter()
            .find(|c| c.dark == plan.mode)
            .map_or(plan.mode, |c| c.bright);
        ActiveTune {
            tune_index,
            plan: plan.clone(),
            fiddle,
            harmony,
            feet_humanize,
            performance_seed,
            material_seed,
            intro_info,
            initial_mode: plan.mode,
            initial_dark: plan.mode.is_minor() && valence < dynamics.dark_below,
            bright_mode,
            dark_mode: plan.mode.is_minor().then_some(plan.mode),
            mode_changes: BTreeMap::new(),
            mode_requests: BTreeMap::new(),
        }
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
        fiddle: &FiddleLine,
        rng: &mut dyn RngCore,
    ) -> IntroInfo {
        let intro_config = &self.tuning.profile.arrangement.intro;

        // Tirage de la forme (seulement si forme complète nécessaire)
        let first_in_set = self.is_first_in_set(tune_index);
        let form = if first_in_set { draw_intro_form(intro_config, rng) } else { None };

        let form_bars = form
            .map(|k| {
                intro_config
                    .forms
                    .iter()
                    .find(|f| f.role == k)
                    .map(|f| f.effective_bars())
                    .unwrap_or(k.default_bars())
            })
            .unwrap_or(0);

        let intro_bars = intro_config.bars_for_tune(first_in_set, form_bars);

        // Calculer les notes de levée
        let pickup_notes = if intro_config.pickup_eighths > 0 && intro_bars > 0 {
            let first_note_midi =
                first_melody_note_midi(fiddle, plan, &self.tuning.profile.melody.params);
            compute_pickup_notes(first_note_midi, intro_config.pickup_eighths, plan.key, plan.mode)
        } else {
            Vec::new()
        };

        IntroInfo { form, intro_bars, pickup_notes }
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
fn first_melody_note_midi(fiddle: &FiddleLine, plan: &TunePlan, params: &ReelFiddleParams) -> u8 {
    let tonic = tonic_midi_for_key(plan.key, params.tonic_midi_min, params.tonic_midi_max);
    if let Some(first) = fiddle.notes_for_bar(0).first() {
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

/// Tire le [`TunePlan`] du tune suivant à partir du précédent :
/// même tonalité/mode ou relatif, selon les poids du profil.
#[must_use]
pub fn draw_next_tune_plan(
    prev_plan: TunePlan,
    _params: &ReelFiddleParams,
    rng: &mut dyn RngCore,
) -> TunePlan {
    if rng.next_range_usize(0..2) < 1 {
        TunePlan::with_profile(prev_plan.key, prev_plan.mode, prev_plan.profile)
    } else {
        relative_tune_plan(prev_plan)
    }
}

/// Calcule le [`TunePlan`] relatif (majeur ↔ mineur relatif à ±3 demi-tons) d'un plan donné.
#[must_use]
pub fn relative_tune_plan(plan: TunePlan) -> TunePlan {
    let (key, mode) = relative_key_mode(plan.key, plan.mode);
    TunePlan::with_profile(key, mode, plan.profile)
}

fn relative_key_mode(key: PitchClass, mode: Mode) -> (PitchClass, Mode) {
    match mode {
        Mode::Ionian => ((key + 9) % 12, Mode::Aeolian),
        Mode::Mixolydian => ((key + 9) % 12, Mode::Dorian),
        Mode::Aeolian => ((key + 3) % 12, Mode::Ionian),
        Mode::Dorian => ((key + 3) % 12, Mode::Mixolydian),
    }
}

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

/// Nom lisible d'un [`TradChord`] dans une tonalité et un mode donnés (ex. `"D"`, `"A"`, `"Bm"`).
#[must_use]
pub fn trad_chord_name(chord: TradChord, key: PitchClass, mode: Mode) -> String {
    let root = pitch_class_name(chord.root_pc(key, mode));
    match chord.quality {
        ChordQuality::Major => root.to_string(),
        ChordQuality::Minor => format!("{root}m"),
    }
}

/// Construit le [`ChordContext`] d'une mesure à partir de l'accord de sa première demi-mesure.
#[must_use]
pub fn build_chord_context(chord: TradChord, key: PitchClass, mode: Mode) -> ChordContext {
    ChordContext {
        root_offset: i32::from(chord.root_semitones(mode)),
        is_minor: matches!(chord.quality, ChordQuality::Minor),
        chord_name: trad_chord_name(chord, key, mode),
        scale_guidance: None,
    }
}

/// Vérifie si `next` est soit identique à `prev`, soit sa relative.
#[must_use]
pub fn is_same_or_relative_plan(prev: TunePlan, next: TunePlan) -> bool {
    next.profile == prev.profile
        && ((next.key, next.mode) == (prev.key, prev.mode)
            || (next.key, next.mode) == relative_key_mode(prev.key, prev.mode))
}

/// Degré de l'accord tonique pour les assertions de tests.
#[must_use]
pub const fn is_tonic_chord(chord: TradChord) -> bool {
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
        params::{PulseSpec, TimeSignature},
        trad::{ReelArrangement, Section},
    };

    fn make_reel_params() -> MusicalParams {
        let rf = crate::StyleProfile::default().meter;
        let mut params = MusicalParams::default();
        params.bpm = rf.default_bpm;
        params.set_swing_ratio(rf.default_swing_ratio);
        params.time_signature = rf.time_signature;
        params
    }

    fn make_reel_tuning_flat() -> TuningParams {
        TuningParams {
            profile: {
                let mut p = TuningParams::default().profile;
                p.arrangement = ReelArrangement::from_start(3, Default::default());
                p
            },
            ..TuningParams::default()
        }
    }

    #[test]
    fn measure_metadata_matches_d3_and_params() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let params = make_reel_params();
        let tuning = make_reel_tuning_flat();
        let mut trad_gen = TradGenerator::new(params, tuning);

        let m1 = trad_gen.generate_measure(1, &mut rng);
        assert_eq!(m1.index, 1);
        assert_eq!(m1.time_signature, TimeSignature::new(2, 2));
        assert_eq!(m1.steps, 16);
        assert_eq!(m1.pulse, PulseSpec { steps_per_pulse: 8, pulses_per_bar: 2 });
        assert!((m1.swing_ratio - 0.56).abs() < f32::EPSILON);
        assert!((m1.tempo - 230.0).abs() < f32::EPSILON);
        assert!(!m1.chord_context.chord_name.is_empty());
    }

    #[test]
    fn next_tune_is_same_or_relative_across_seeds() {
        for seed in 0..20 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut trad_gen = TradGenerator::new(make_reel_params(), make_reel_tuning_flat());
            for bar in 1..=97 {
                let _ = trad_gen.generate_measure(bar, &mut rng);
            }
            let tune1 = trad_gen.tune_for_bar(1).map(|t| t.plan.clone());
            let tune2 = trad_gen.tune_for_bar(97).map(|t| t.plan.clone());
            assert!(tune1.is_some() && tune2.is_some());
            if let (Some(p1), Some(p2)) = (tune1, tune2) {
                assert!(
                    is_same_or_relative_plan(p1.clone(), p2.clone()),
                    "seed {seed}: tune 2 ({p2:?}) n'est ni identique ni relatif à tune 1 ({p1:?})"
                );
            }
        }
    }

    #[test]
    fn intro_position_is_exposed() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let params = make_reel_params();
        let mut trad_gen = TradGenerator::new(params, TuningParams::default());

        // La config par défaut a une intro avec `each_tune = false`
        let m1 = trad_gen.generate_measure(1, &mut rng);
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
        let params = make_reel_params();
        let mut flat_gen = TradGenerator::new(params.clone(), make_reel_tuning_flat());
        let flat_notes: Vec<Vec<u8>> = (1..=32)
            .map(|bar| {
                let m = flat_gen.generate_measure(bar, &mut rng_flat);
                m.notes_for_track(TrackId::Melody).iter().map(|n| n.pitch).collect()
            })
            .collect();

        // 2. Avec intro : générer intro + 32 mesures, puis comparer les 32 mesures après l'intro
        let mut rng_intro = ChaCha8Rng::seed_from_u64(seed);
        let mut intro_gen = TradGenerator::new(params, TuningParams::default());

        // Générer assez pour passer l'intro et couvrir 32 mesures d'air
        for bar in 1..=100 {
            let _ = intro_gen.generate_measure(bar, &mut rng_intro);
        }

        let intro_bars = intro_gen.tunes.get(&0).unwrap().intro_info.intro_bars;

        // Régénérer depuis le début (même seed) pour la comparaison
        let mut rng_intro2 = ChaCha8Rng::seed_from_u64(seed);
        let mut intro_gen2 = TradGenerator::new(make_reel_params(), TuningParams::default());

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
        let params = make_reel_params();
        let tuning = TuningParams::default(); // each_tune = false par défaut
        let mut trad_gen = TradGenerator::new(params, tuning);

        // Générer jusqu'au 2ᵉ tune
        for bar in 1..=200 {
            let m = trad_gen.generate_measure(bar, &mut rng);
            let pos = m.position.unwrap();
            if pos.tune_index == 1 && pos.section == Section::Tour && pos.bar_in_strain == 1 {
                // La première mesure du 2ᵉ air ne devrait PAS être précédée d'une intro
                // (seulement une levée si pickup_eighths > 0)
                // Vérifions que la mesure juste avant est soit la dernière du tune 0,
                // soit une mesure de levée
                let prev = trad_gen.generate_measure(bar - 1, &mut ChaCha8Rng::seed_from_u64(7));
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
        let params = make_reel_params();
        let tuning = TuningParams::default();
        let mut trad_gen = TradGenerator::new(params, tuning);

        // Générer assez pour avoir l'intro
        for bar in 1..=50 {
            let _ = trad_gen.generate_measure(bar, &mut rng);
        }

        let tune = trad_gen.tunes.get(&0).unwrap();
        let intro = &tune.intro_info;

        if !intro.pickup_notes.is_empty() {
            let target = first_melody_note_midi(
                &tune.fiddle,
                &tune.plan,
                &trad_gen.tuning.profile.melody.params,
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
        let params = make_reel_params();
        let tuning = TuningParams::default();

        let mut gen1 = TradGenerator::new(params.clone(), tuning.clone());
        let mut gen2 = TradGenerator::new(params, tuning);

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
        // Percussion → Heel présent, Melody absent (sauf levée en dernière mesure)
        // Tail → Melody présent
        // Vamp → Chords présent

        // On teste avec différentes seeds pour couvrir chaque forme
        for seed in 0..100u64 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let params = make_reel_params();
            let tuning = TuningParams::default();
            let mut trad_gen = TradGenerator::new(params, tuning);

            let m1 = trad_gen.generate_measure(1, &mut rng);
            let tune = trad_gen.tunes.get(&0).unwrap();
            let intro = &tune.intro_info;

            match intro.form {
                Some(IntroRole::Percussion) => {
                    // Talon devrait être présent (sauf si enable_rhythm désactivé)
                    let heel_notes = m1.notes_for_track(TrackId::Heel);
                    assert!(
                        !heel_notes.is_empty(),
                        "Percussion (seed {seed}): attendu talon présent"
                    );
                }
                Some(IntroRole::Tail) => {
                    let mel_notes = m1.notes_for_track(TrackId::Melody);
                    assert!(!mel_notes.is_empty(), "Tail (seed {seed}): attendu violon présent");
                }
                Some(IntroRole::Vamp) => {
                    let chord_notes = m1.notes_for_track(TrackId::Chords);
                    assert!(
                        !chord_notes.is_empty(),
                        "Vamp (seed {seed}): attendu guitare présente"
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
        use crate::trad::ArrangementError;

        let mut config = IntroConfig::default();
        config.forms[0].bars = 10;
        assert!(matches!(config.validate(), Err(ArrangementError::IntroBarsOutOfRange { .. })));

        let mut config = IntroConfig::default();
        config.pickup_eighths = 4;
        assert!(matches!(config.validate(), Err(ArrangementError::PickupEighthsOutOfRange(4))));
    }
}

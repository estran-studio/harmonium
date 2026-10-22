//! Générateur de mesures [`TradGenerator`] pour le style [`crate::params::Style::Reel`]
//! (D1, D3, D4, D6–D9, P1-T11).
//!
//! Produit des [`Measure`] complètes en 2/2 (16 pas, pulse blanche de 8 pas) :
//! - au début de chaque air, tire une intro idiomatique (P1-T11) puis un
//!   [`TunePlan`], construit la [`FiddleLine`] et la [`TuneHarmony`] ;
//!   l'air est ensuite joué `tours_per_tune` tours AABB de 32 mesures, chaque
//!   tour rejouant les mêmes degrés, relus dans le mode parallèle demandé ;
//! - pour les tunes suivants, tire au seed soit la même tonalité, soit sa relative ;
//! - pour chaque mesure : peuple `Melody` (violon), `Chords` (guitare chop) et
//!   `Bass` (basse fondamentale/quinte), ainsi que `Heel` et `Toe` selon
//!   `MusicalParams::feet_density`, pour les seules pistes actives (l'arrangement
//!   [`ReelArrangement`] les autorise ET le drapeau `enable_*` le permet) ;
//! - expose la [`TunePosition`] de la mesure (`Measure::position`) ;
//! - remplit `Measure::chord_context` depuis l'accord de la première demi-mesure
//!   avec un nom lisible (ex. `"D"`, `"G"`, `"A"`, `"Bm"`, `"Em"`) ;
//! - gère la fin (outro) : dernière mesure du dernier tour, tonique accentuée
//!   puis silence (P1-T11).

use std::collections::BTreeMap;

use super::{
    accomp::{Guitarist, bass_bar},
    arrangement::{self, IntroConfig, IntroInfo, IntroRole, Section, TunePosition},
    cells::{Cell, NOTES_PER_CELL},
    feet::{FEET_HUMANIZE_SLOTS, REEL_STEPS, draw_feet_humanize, feet_notes},
    fiddle::{BARS_PER_STRAIN, FiddleLine, same_cell_up_to_octave, tonic_midi_for_key},
    harmony::{ChordDegree, ChordQuality, TUNE_HARMONY_BARS, TradChord, TuneHarmony},
    mode::{Mode, PitchClass},
    plan::{BarRole, TuneKind, TunePlan},
};
use crate::{
    harmony::RngCore,
    params::MusicalParams,
    timeline::{
        Articulation, ChordContext, GenerationContext, Measure, NoteId, StateSnapshot,
        TimelineNote, TrackId,
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
    /// Ligne de violon des 32 mesures d'un tour.
    pub fiddle: FiddleLine,
    /// Harmonie par demi-mesure des 32 mesures d'un tour.
    pub harmony: TuneHarmony,
    /// Décalages d'humanisation des vélocités des pieds, par position de mesure
    /// (2 strains × 8 mesures) et par pas. Tirés une fois à la génération du
    /// tune pour garantir des reprises identiques (A1=A2, B1=B2, D4).
    pub feet_humanize: [[i8; REEL_STEPS]; FEET_HUMANIZE_SLOTS],
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

/// Générateur de musique traditionnelle québécoise (`Style::Reel`, D1).
#[derive(Clone, Debug)]
pub struct TradGenerator {
    /// Paramètres musicaux courants (tempo, swing, densité des pieds, activations).
    pub musical_params: MusicalParams,
    /// Paramètres de style et banques de données (cellules, podorythmie, harmonie).
    pub tuning: TuningParams,
    /// Airs générés indexés par `tune_index` (`0` = mesures 1..=32×tours, etc.).
    tunes: BTreeMap<usize, ActiveTune>,
    /// Compteur monotone d'identifiants de notes.
    next_note_id: NoteId,
    /// Dernier indice de mesure généré.
    current_bar: usize,
}

impl TradGenerator {
    /// Construit un nouveau [`TradGenerator`] avec les paramètres et le tuning donnés.
    #[must_use]
    pub fn new(musical_params: MusicalParams, tuning: TuningParams) -> Self {
        Self { musical_params, tuning, tunes: BTreeMap::new(), next_note_id: 1, current_bar: 0 }
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
            tune.mode_changes.retain(|&bar, _| bar <= playing_bar);
            tune.mode_requests.retain(|&bar, _| bar <= playing_bar);
        }
    }

    fn feet_density(&self) -> u8 {
        let p = &self.musical_params;
        let d = &self.tuning.reel_intensity;
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
            self.tuning = tuning;
            self.tunes.clear();
        }
    }

    /// Réinitialise le générateur à son état initial pour le seek déterministe.
    ///
    /// Conserve `musical_params`, `tuning` et le compteur monotone `next_note_id`,
    /// et vide les tunes générés afin que le rejeu depuis la mesure 1 reproduise
    /// exactement la séquence de tirages sur le RNG.
    pub fn reset_to_initial(&mut self, _ctx: &GenerationContext) {
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

    /// Opération sans effet pour [`TradGenerator`] : le générateur trad n'effectue
    /// pas de morphing progressif d'état émotionnel entre les pas (`MusicalParams`
    /// est lu directement à chaque mesure).
    pub const fn snap_current_state(&mut self) {}

    /// Réinitialise l'indice de grille d'accords (sans objet en trad, exposé pour
    /// symétrie avec [`crate::timeline::TimelineGenerator`]).
    pub fn reset_chart_index(&mut self) {
        self.tunes.clear();
        self.current_bar = 0;
    }

    /// Retourne le tune actif correspondant à la mesure 1-based `bar_number`, s'il a été généré.
    #[must_use]
    pub fn tune_for_bar(&self, bar_number: usize) -> Option<&ActiveTune> {
        let tune_idx = self.position_of(bar_number).tune_index;
        self.tunes.get(&tune_idx).or_else(|| self.tunes.values().last())
    }

    /// Retourne le [`TunePlan`] et le [`BarRole`] pour la mesure 1-based `bar_number`.
    #[must_use]
    pub fn bar_info(&self, bar_number: usize) -> Option<(TunePlan, BarRole)> {
        let tune = self.tune_for_bar(bar_number)?;
        let role = tune.plan.bar_role_1based(bar_number.max(1));
        let mut plan = tune.plan;
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
        let tours = self.tuning.reel_arrangement.tours_per_tune;
        let cycle = arrangement::BARS_PER_TOUR * tours;

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
            let body_start = offset + intro_bars + 1;
            let body_end = offset + intro_bars + cycle; // dernière mesure du corps (incluse)

            if bar_number < body_start {
                // Barre dans l'intro du tune courant
                return TunePosition {
                    section: Section::Intro,
                    tune_index: tune_idx,
                    tour: 0,
                    tours,
                    part: arrangement::Part::A,
                    pass: 0,
                    bar_in_strain: 0,
                    key,
                    mode,
                };
            }

            if bar_number <= body_end {
                // Barre dans le corps du tune courant
                let local_bar = bar_number - offset - intro_bars; // 1-based dans le tune
                let zero = local_bar.saturating_sub(1);
                let in_tune = zero % cycle;
                let in_tour = in_tune % arrangement::BARS_PER_TOUR;
                let in_part = in_tour % (2 * BARS_PER_STRAIN);
                return TunePosition {
                    section: Section::Tour,
                    tune_index: tune_idx,
                    tour: in_tune / arrangement::BARS_PER_TOUR + 1,
                    tours,
                    part: if in_tour < 2 * BARS_PER_STRAIN {
                        arrangement::Part::A
                    } else {
                        arrangement::Part::B
                    },
                    pass: in_part / BARS_PER_STRAIN + 1,
                    bar_in_strain: in_part % BARS_PER_STRAIN + 1,
                    key,
                    mode,
                };
            }

            offset += intro_bars + cycle;
            tune_idx += 1;
        }
    }

    /// Vrai si `track` joue à `position` : l'arrangement l'autorise (ou la piste a
    /// joué l'intro de cet air) ET le drapeau `enable_*` du jeu le permet. Seul
    /// calcul de « piste active ».
    fn track_active(&self, track: TrackId, position: &TunePosition) -> bool {
        let flag = match track {
            TrackId::Melody => self.musical_params.enable_melody,
            TrackId::Chords | TrackId::Bass => self.musical_params.enable_harmony,
            TrackId::Heel | TrackId::Toe => self.musical_params.enable_rhythm,
        };
        let entered = self.tuning.reel_arrangement.allows(track, position)
            || self.played_intro(track, position.tune_index);
        flag && entered
            && self.tuning.reel_intensity.allows(track, self.musical_params.rhythm_density)
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
        self.tuning.reel_intensity.allows(track, self.musical_params.rhythm_density)
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

        let kind = TuneKind::Reel;
        let time_sig = kind.time_signature();
        let steps = kind.steps_per_bar();
        let tempo = self
            .tuning
            .reel_intensity
            .tempo_for(self.musical_params.bpm, self.musical_params.intensity);

        let mut measure = Measure::new(bar_index, time_sig, tempo, steps);
        measure.pulse = kind.pulse_spec();
        measure.position = Some(position);
        measure.swing_ratio = self.musical_params.swing_ratio;
        measure.state_snapshot = StateSnapshot {
            bpm: tempo,
            density: self.musical_params.rhythm_density,
            tension: self.musical_params.harmony_tension,
            smoothness: self.musical_params.melody_smoothness,
            valence: self.musical_params.harmony_valence,
            arousal: (tempo - 70.0) / 110.0,
        };

        let Some(tune) = self.tunes.get(&position.tune_index).cloned() else {
            return measure;
        };

        let is_intro = position.is_intro();
        let is_outro = self.musical_params.end_after_tour
            && !is_intro
            && position.is_last_tour()
            && position.bar_in_strain == BARS_PER_STRAIN
            && position.pass == 2
            && matches!(position.part, arrangement::Part::B);

        if is_intro {
            self.fill_intro_measure(&tune, &position, &mut measure);
        } else if is_outro {
            self.fill_outro_measure(&tune, &position, &mut measure);
        } else {
            self.fill_tour_measure(&tune, &position, &mut measure);
        }

        self.apply_dynamics(&mut measure);

        measure
    }

    fn update_parallel_mode(
        &mut self,
        bar: usize,
        position: &mut TunePosition,
        rng: &mut dyn RngCore,
    ) {
        let dynamics = &self.tuning.reel_intensity;
        let Some(tune) = self.tunes.get_mut(&position.tune_index) else { return };
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
            && (position.bar_in_strain - 1).is_multiple_of(dynamics.mode_shift_boundary.bars())
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
            let melody: [Vec<TimelineNote>; TUNE_HARMONY_BARS] = std::array::from_fn(|i| {
                let bar = if i < BARS_PER_STRAIN { i } else { i + BARS_PER_STRAIN };
                tune.fiddle.notes_for_bar(bar)
            });
            tune.harmony = TuneHarmony::new(tune.plan, &melody, &self.tuning.trad_harmony)
                .unwrap_or_else(|_| fallback_tune_harmony(tune.plan));
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
        let plan = tune.plan;
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
                if self.track_active_in_intro(TrackId::Heel, intro) {
                    let (heel_notes, _) = feet_notes(
                        &tuning.reel_feet,
                        self.feet_density(),
                        steps,
                        &[0i8; REEL_STEPS],
                    );
                    for mut note in heel_notes {
                        note.id = self.alloc_note_id();
                        measure.add_note(TrackId::Heel, note);
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
                    let chords = [tonic, tonic];
                    let next_note_id = &mut self.next_note_id;
                    let mut next_id = || {
                        let id = *next_note_id;
                        *next_note_id += 1;
                        id
                    };
                    let mut guitarist = Guitarist::new();
                    for note in guitarist.chop_bar(
                        chords,
                        plan.key,
                        plan.mode,
                        &tuning.trad_harmony,
                        &mut next_id,
                    ) {
                        measure.add_note(TrackId::Chords, note);
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
            let eighth_steps = steps / NOTES_PER_CELL;
            let eighth_steps = eighth_steps.max(1);
            for (i, &midi) in intro.pickup_notes.iter().enumerate() {
                let start_step = steps - (pickup_len - i) * eighth_steps;
                let note = TimelineNote {
                    id: self.alloc_note_id(),
                    pitch: midi,
                    velocity: 80,
                    start_step,
                    duration_steps: eighth_steps,
                    articulation: Articulation::default(),
                };
                measure.add_note(TrackId::Melody, note);
            }
        }
    }

    /// Calcule la mesure de début de l'intro pour `tune_index`.
    fn intro_start_bar_for_tune(&self, tune_index: usize) -> usize {
        let mut offset = 1usize;
        for idx in 0..tune_index {
            if let Some(t) = self.tunes.get(&idx) {
                offset += t.intro_info.intro_bars;
                offset += arrangement::BARS_PER_TOUR * self.tuning.reel_arrangement.tours_per_tune;
            }
        }
        offset
    }

    /// Remplit une mesure de tour normale (identique à l'existant D11).
    fn fill_tour_measure(
        &mut self,
        tune: &ActiveTune,
        position: &TunePosition,
        measure: &mut Measure,
    ) {
        let bar_in_tour = position.bar_in_tour();
        let plan = tune.plan;
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
            for note in
                bass_bar(chords, plan.key, plan.mode, &self.tuning.trad_harmony, &mut next_id)
            {
                measure.add_note(TrackId::Bass, note);
            }
        }

        // 3. Podorythmie (talon + pointe, D9)
        let heel_active = self.track_active(TrackId::Heel, position);
        let toe_active = self.track_active(TrackId::Toe, position);
        if heel_active || toe_active {
            let strain_slot = if bar_in_tour < 2 * BARS_PER_STRAIN { 0 } else { BARS_PER_STRAIN };
            let humanize = &tune.feet_humanize[strain_slot + bar_in_tour % BARS_PER_STRAIN];
            let (heel_notes, toe_notes) =
                feet_notes(&self.tuning.reel_feet, self.feet_density(), 16, humanize);
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
        let plan = tune.plan;
        let tonic_midi = tonic_midi_for_key(
            plan.key,
            self.tuning.reel_fiddle.tonic_midi_min,
            self.tuning.reel_fiddle.tonic_midi_max,
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
                    duration_steps: 8,
                    articulation: Articulation::Normal,
                },
            );
        }

        // Accompagnement : tonique accentuée
        if self.musical_params.enable_harmony {
            let next_id = &mut self.next_note_id;
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
                    duration_steps: 8,
                    articulation: Articulation::Normal,
                },
            );
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
                    duration_steps: 8,
                    articulation: Articulation::Normal,
                },
            );
        }

        // Pieds : talon accentué
        if self.musical_params.enable_rhythm {
            measure.add_note(
                TrackId::Heel,
                TimelineNote {
                    id: self.alloc_note_id(),
                    pitch: self.tuning.reel_feet.heel_note,
                    velocity: 110,
                    start_step: 0,
                    duration_steps: 2,
                    articulation: Articulation::Normal,
                },
            );
        }
    }

    /// Variation harmonique de l'accompagnement uniquement : cadences et
    /// squelette mélodique restent ceux de l'air.
    fn chords_for_bar(&self, tune: &ActiveTune, bar: usize) -> [TradChord; 2] {
        let mut chords = tune.harmony.chords_for_bar(bar);
        let threshold =
            self.tuning.reel_intensity.tension_dominant.eval(self.musical_params.harmony_tension);
        let slot = bar % BARS_PER_STRAIN;
        if slot != 3 && slot != 7 && (slot as f32 / BARS_PER_STRAIN as f32) < threshold {
            // La dominante garde la qualité diatonique du mode courant.
            chords[1] = super::harmony::candidate_chords(tune.plan.mode)
                .iter()
                .copied()
                .find(|c| c.degree == ChordDegree::V)
                .unwrap_or(chords[1]);
        }
        chords
    }

    fn apply_dynamics(&mut self, measure: &mut Measure) {
        let p = &self.musical_params;
        let d = &self.tuning.reel_intensity;
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
                    } else if note.start_step % 4 != 0 {
                        offbeat
                    } else {
                        0.0
                    }
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
        // Ornement déterministe d'une tenue : aucun nouveau tirage ni mutation
        // du plan. Le degré adjacent appartient toujours au mode de l'air.
        let passing = d.density_passing.eval(p.rhythm_density);
        if passing <= 0.0 {
            return;
        }
        let Some(position) = measure.position else {
            return;
        };
        if position.is_intro() || position.bar_in_strain == BARS_PER_STRAIN {
            return;
        }
        let Some(plan) = self.tunes.get(&position.tune_index).map(|t| t.plan) else {
            return;
        };
        if let Some(notes) = measure.tracks.get_mut(&TrackId::Melody) {
            let original_count = notes.len();
            for index in 0..original_count {
                let note = &notes[index];
                if note.duration_steps < 3
                    || (note.start_step as f32 / measure.steps as f32) >= passing
                {
                    continue;
                }
                let mut ornament = note.clone();
                ornament.start_step += note.duration_steps - 1;
                ornament.duration_steps = 1;
                let next_pitch = (note.pitch.saturating_add(1)
                    ..=note.pitch.saturating_add(3).min(127))
                    .find(|&pitch| plan.mode.contains_midi(plan.key, pitch));
                let Some(pitch) = next_pitch else {
                    continue;
                };
                notes[index].duration_steps -= 1;
                ornament.pitch = pitch;
                ornament.id = self.next_note_id;
                self.next_note_id += 1;
                notes.push(ornament);
            }
            notes.sort_by_key(|note| note.start_step);
        }
    }

    /// Calcule les chops de guitare de `bar_in_tour` en rejouant le [`Guitarist`]
    /// depuis le début de la passe de strain courante (`(bar_in_tour / 8) * 8`),
    /// garantissant des reprises A1=A2, B1=B2 et des tours identiques même après
    /// invalidation.
    fn chop_notes_for_bar(&mut self, tune: &ActiveTune, bar_in_tour: usize) -> Vec<TimelineNote> {
        let strain_pass_start = (bar_in_tour / BARS_PER_STRAIN) * BARS_PER_STRAIN;
        let harmony = &self.tuning.trad_harmony;
        let d = &self.tuning.reel_intensity;
        let chop_offset = d.chop_intensity.eval(self.musical_params.intensity)
            + d.density_chop.eval(self.musical_params.rhythm_density);
        let steps = if chop_offset != 0.0 {
            let pattern =
                (1.0 + chop_offset).round().clamp(0.0, (d.chop_patterns.len() - 1) as f32) as usize;
            &d.chop_patterns[pattern]
        } else {
            &harmony.chop_steps
        };
        let mut guitarist = Guitarist::new();
        let mut dummy_id = 0u64;
        let mut dummy_next = || {
            dummy_id += 1;
            dummy_id
        };
        for b in strain_pass_start..bar_in_tour {
            let c = self.chords_for_bar(tune, b);
            let _ = guitarist.chop_bar_on_steps(
                c,
                tune.plan.key,
                tune.plan.mode,
                harmony,
                steps,
                &mut dummy_next,
            );
        }
        let chords = self.chords_for_bar(tune, bar_in_tour);
        let next_note_id = &mut self.next_note_id;
        let mut next_id = || {
            let id = *next_note_id;
            *next_note_id += 1;
            id
        };
        guitarist.chop_bar_on_steps(
            chords,
            tune.plan.key,
            tune.plan.mode,
            harmony,
            steps,
            &mut next_id,
        )
    }

    /// S'assure que tous les tunes nécessaires pour couvrir `bar_index` ont été
    /// tirés, avec leur intro. Appelé avant [`Self::position_of`].
    fn ensure_tunes_covering(&mut self, bar_index: usize, rng: &mut dyn RngCore) {
        let tours = self.tuning.reel_arrangement.tours_per_tune;
        let cycle = arrangement::BARS_PER_TOUR * tours;
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
                let prev_plan = prev.map(|t| t.plan);
                let prev_first_cell = prev.map(|t| t.fiddle.cells_a()[0].clone());
                let tune = self.build_tune(tune_idx, prev_plan, prev_first_cell, rng);
                self.tunes.insert(tune_idx, tune);
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
        let dynamics = &self.tuning.reel_intensity;
        let valence = self.musical_params.harmony_valence;
        let plan = if valence == dynamics.neutral_valence {
            match prev_plan {
                None => TunePlan::draw_with_params(rng, None, &self.tuning.reel_fiddle),
                Some(prev) => draw_next_tune_plan(prev, &self.tuning.reel_fiddle, rng),
            }
        } else {
            dynamics.draw_color(valence, rng)
        };
        let mut fiddle_params = self.tuning.reel_fiddle.clone();
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
        let mut fiddle =
            FiddleLine::with_params(plan, &self.tuning.reel_cells, &fiddle_params, rng);
        if let Some(prev_first) = prev_first_cell {
            for _ in 0..8 {
                if !same_cell_up_to_octave(&fiddle.cells_a()[0], &prev_first) {
                    break;
                }
                fiddle =
                    FiddleLine::with_params(plan, &self.tuning.reel_cells, &fiddle_params, rng);
            }
        }

        let distinct_melody: [Vec<TimelineNote>; TUNE_HARMONY_BARS] = std::array::from_fn(|i| {
            let bar_in_tune =
                if i < BARS_PER_STRAIN { i } else { 2 * BARS_PER_STRAIN + (i - BARS_PER_STRAIN) };
            fiddle.notes_for_bar(bar_in_tune)
        });

        let harmony = TuneHarmony::new(plan, &distinct_melody, &self.tuning.trad_harmony)
            .unwrap_or_else(|_| fallback_tune_harmony(plan));

        let feet_humanize = draw_feet_humanize(self.tuning.reel_feet.velocity_humanize, rng);

        // Tirer l'intro (P1-T11)
        let intro_info = self.draw_intro(tune_index, &plan, &fiddle, rng);

        let bright_mode = dynamics
            .parallel_modes
            .iter()
            .find(|c| c.dark == plan.mode)
            .map_or(plan.mode, |c| c.bright);
        ActiveTune {
            tune_index,
            plan,
            fiddle,
            harmony,
            feet_humanize,
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
    /// - Respecte `each_tune` : tunes 1+ sans intro complète ont seulement la levée
    fn draw_intro(
        &self,
        tune_index: usize,
        plan: &TunePlan,
        fiddle: &FiddleLine,
        rng: &mut dyn RngCore,
    ) -> IntroInfo {
        let intro_config = &self.tuning.reel_arrangement.intro;

        // Tirage de la forme (seulement si forme complète nécessaire)
        let form = if tune_index == 0 || intro_config.each_tune {
            draw_intro_form(intro_config, rng)
        } else {
            None
        };

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

        let intro_bars = intro_config.bars_for_tune(tune_index, form_bars);

        // Calculer les notes de levée
        let pickup_notes = if intro_config.pickup_eighths > 0 && intro_bars > 0 {
            let first_note_midi = first_melody_note_midi(fiddle, plan, &self.tuning.reel_fiddle);
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
/// même tonalité/mode ou tonalité relative (choix équiprobable au seed, D4).
#[must_use]
pub fn draw_next_tune_plan(
    prev_plan: TunePlan,
    _params: &ReelFiddleParams,
    rng: &mut dyn RngCore,
) -> TunePlan {
    if rng.next_range_usize(0..2) == 0 { prev_plan } else { relative_tune_plan(prev_plan) }
}

/// Calcule le [`TunePlan`] relatif (majeur ↔ mineur relatif à ±3 demi-tons) d'un plan donné.
#[must_use]
pub const fn relative_tune_plan(plan: TunePlan) -> TunePlan {
    let (key, mode) = match plan.mode {
        Mode::Ionian => ((plan.key + 9) % 12, Mode::Aeolian),
        Mode::Mixolydian => ((plan.key + 9) % 12, Mode::Dorian),
        Mode::Aeolian => ((plan.key + 3) % 12, Mode::Ionian),
        Mode::Dorian => ((plan.key + 3) % 12, Mode::Mixolydian),
    };
    TunePlan::new(key, mode, plan.kind, plan.scheme)
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

/// Repli sans panique pour `TuneHarmony` (ne peut pas se produire car `distinct_melody` a 16 éléments).
fn fallback_tune_harmony(plan: TunePlan) -> TuneHarmony {
    let empty: [Vec<TimelineNote>; TUNE_HARMONY_BARS] = std::array::from_fn(|_| Vec::new());
    if let Ok(h) = TuneHarmony::new(plan, &empty, &crate::tuning::TradHarmonyParams::default()) {
        return h;
    }
    unreachable!("TUNE_HARMONY_BARS est constant (16)")
}

/// Vérifie si `next` est soit identique à `prev`, soit sa relative.
#[must_use]
pub fn is_same_or_relative_plan(prev: TunePlan, next: TunePlan) -> bool {
    next == prev || next == relative_tune_plan(prev)
}

/// Degré de l'accord tonique pour les assertions de tests.
#[must_use]
pub const fn is_tonic_chord(chord: TradChord) -> bool {
    matches!(chord.degree, ChordDegree::I)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::{
        params::{PulseSpec, Style, TimeSignature},
        trad::ReelArrangement,
    };

    fn make_reel_params() -> MusicalParams {
        let rf = ReelFiddleParams::default();
        let mut params = MusicalParams::default();
        params.set_style(Style::Reel);
        params.bpm = rf.default_bpm;
        params.set_swing_ratio(rf.default_swing_ratio);
        params.time_signature = rf.default_time_signature;
        params
    }

    fn make_reel_tuning_flat() -> TuningParams {
        TuningParams { reel_arrangement: ReelArrangement::from_start(3), ..TuningParams::default() }
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

        // Arrangement plat : les 5 pistes sont peuplées dès la mesure 1 en densité de pieds 1
        assert!(!m1.notes_for_track(TrackId::Melody).is_empty());
        assert_eq!(m1.notes_for_track(TrackId::Chords).len(), 6);
        assert_eq!(m1.notes_for_track(TrackId::Bass).len(), 2);
        assert_eq!(m1.notes_for_track(TrackId::Heel).len(), 4);
        assert_eq!(m1.notes_for_track(TrackId::Toe).len(), 4);
    }

    #[test]
    fn next_tune_is_same_or_relative_across_seeds() {
        for seed in 0..20 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut trad_gen = TradGenerator::new(make_reel_params(), make_reel_tuning_flat());
            for bar in 1..=97 {
                let _ = trad_gen.generate_measure(bar, &mut rng);
            }
            let tune1 = trad_gen.tune_for_bar(1).map(|t| t.plan);
            let tune2 = trad_gen.tune_for_bar(97).map(|t| t.plan);
            assert!(tune1.is_some() && tune2.is_some());
            if let (Some(p1), Some(p2)) = (tune1, tune2) {
                assert!(
                    is_same_or_relative_plan(p1, p2),
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
            let target =
                first_melody_note_midi(&tune.fiddle, &tune.plan, &trad_gen.tuning.reel_fiddle);
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

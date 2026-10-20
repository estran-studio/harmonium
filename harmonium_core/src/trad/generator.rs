//! Générateur de mesures [`TradGenerator`] pour le style [`crate::params::Style::Reel`] (D1, D3, D4, D6–D9).
//!
//! Produit des [`Measure`] complètes en 2/2 (16 pas, pulse blanche de 8 pas) :
//! - au début de chaque tune de 32 mesures (forme AABB), tire un [`TunePlan`],
//!   construit la [`FiddleLine`] et la [`TuneHarmony`] ;
//! - pour les tunes suivants, tire au seed soit la même tonalité, soit sa relative ;
//! - pour chaque mesure : peuple `Melody` (violon), `Chords` (guitare chop) et
//!   `Bass` (basse fondamentale/quinte), ainsi que `Heel` et `Toe` selon
//!   `MusicalParams::feet_density` ;
//! - remplit `Measure::chord_context` depuis l'accord de la première demi-mesure
//!   avec un nom lisible (ex. `"D"`, `"G"`, `"A"`, `"Bm"`, `"Em"`).

use std::collections::BTreeMap;

use super::{
    accomp::{Guitarist, bass_bar},
    cells::Cell,
    feet::feet_notes,
    fiddle::{BARS_PER_STRAIN, FiddleLine, same_cell_up_to_octave},
    harmony::{ChordDegree, ChordQuality, TUNE_HARMONY_BARS, TradChord, TuneHarmony},
    mode::{Mode, PitchClass},
    plan::{BarRole, TuneKind, TunePlan},
};
use crate::{
    harmony::RngCore,
    params::MusicalParams,
    timeline::{
        ChordContext, GenerationContext, Measure, NoteId, StateSnapshot, TimelineNote, TrackId,
    },
    tuning::{ReelFiddleParams, TuningParams},
};

/// Nombre de mesures par tune AABB complet (32 mesures : A1 + A2 + B1 + B2).
pub const BARS_PER_TUNE: usize = 32;

/// État d'un tune de 32 mesures généré par [`TradGenerator`].
#[derive(Clone, Debug, PartialEq)]
pub struct ActiveTune {
    /// Indice 0-based du tune (`(bar_1based - 1) / 32`).
    pub tune_index: usize,
    /// Plan structurel du tune (tonalité, mode, type, schéma AABB).
    pub plan: TunePlan,
    /// Ligne de violon des 32 mesures du tune.
    pub fiddle: FiddleLine,
    /// Harmonie par demi-mesure des 32 mesures du tune.
    pub harmony: TuneHarmony,
}

/// Générateur de musique traditionnelle québécoise (`Style::Reel`, D1).
#[derive(Clone, Debug)]
pub struct TradGenerator {
    /// Paramètres musicaux courants (tempo, swing, densité des pieds, activations).
    pub musical_params: MusicalParams,
    /// Paramètres de style et banques de données (cellules, podorythmie, harmonie).
    pub tuning: TuningParams,
    /// Tunes générés indexés par `tune_index` (`0` = mesures 1..=32, `1` = mesures 33..=64, etc.).
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
        let tune_idx = bar_number.saturating_sub(1) / BARS_PER_TUNE;
        self.tunes.get(&tune_idx).or_else(|| self.tunes.values().last())
    }

    /// Retourne le [`TunePlan`] et le [`BarRole`] pour la mesure 1-based `bar_number`.
    #[must_use]
    pub fn bar_info(&self, bar_number: usize) -> Option<(TunePlan, BarRole)> {
        let tune = self.tune_for_bar(bar_number)?;
        let role = tune.plan.bar_role_1based(bar_number.max(1));
        Some((tune.plan, role))
    }

    /// Génère une [`Measure`] complète pour `bar_index` (1-based dans le flux du moteur).
    pub fn generate_measure(&mut self, bar_index: usize, rng: &mut dyn RngCore) -> Measure {
        self.current_bar = bar_index;
        let bar_1based = bar_index.max(1);
        let zero_based = bar_1based - 1;
        let tune_index = zero_based / BARS_PER_TUNE;
        let bar_in_tune = zero_based % BARS_PER_TUNE;

        self.ensure_tune_generated(tune_index, rng);

        let kind = TuneKind::Reel;
        let time_sig = kind.time_signature();
        let steps = kind.steps_per_bar();
        let tempo = self.musical_params.bpm;

        let mut measure = Measure::new(bar_index, time_sig, tempo, steps);
        measure.pulse = kind.pulse_spec();
        measure.swing_ratio = self.musical_params.swing_ratio;
        measure.state_snapshot = StateSnapshot {
            bpm: tempo,
            density: self.musical_params.rhythm_density,
            tension: self.musical_params.harmony_tension,
            smoothness: self.musical_params.melody_smoothness,
            valence: self.musical_params.harmony_valence,
            arousal: (tempo - 70.0) / 110.0,
        };

        let Some(tune) = self.tunes.get(&tune_index).cloned() else {
            return measure;
        };

        let plan = tune.plan;
        let chords = tune.harmony.chords_for_bar(bar_in_tune);
        measure.chord_context = build_chord_context(chords[0], plan.key, plan.mode);

        // 1. Mélodie (violon, D6)
        let melody_notes = tune.fiddle.notes_for_bar(bar_in_tune);
        if self.musical_params.enable_melody {
            for mut note in melody_notes {
                note.id = self.alloc_note_id();
                measure.add_note(TrackId::Melody, note);
            }
        }

        // 2. Accompagnement (guitare chop + basse, D8)
        let chop_notes = self.chop_notes_for_bar(&tune, bar_in_tune);
        let next_note_id = &mut self.next_note_id;
        let mut next_id = || {
            let id = *next_note_id;
            *next_note_id += 1;
            id
        };
        let bass_notes =
            bass_bar(chords, plan.key, plan.mode, &self.tuning.trad_harmony, &mut next_id);
        if self.musical_params.enable_harmony {
            for note in chop_notes {
                measure.add_note(TrackId::Chords, note);
            }
            for note in bass_notes {
                measure.add_note(TrackId::Bass, note);
            }
        }

        // 3. Podorythmie (talon + pointe, D9)
        let (heel_notes, toe_notes) =
            feet_notes(&self.tuning.reel_feet, self.musical_params.feet_density, steps);
        if self.musical_params.enable_rhythm {
            for mut note in heel_notes {
                note.id = self.alloc_note_id();
                measure.add_note(TrackId::Heel, note);
            }
            for mut note in toe_notes {
                note.id = self.alloc_note_id();
                measure.add_note(TrackId::Toe, note);
            }
        }

        measure
    }

    /// Calcule les chops de guitare de `bar_in_tune` en rejouant le [`Guitarist`]
    /// depuis le début de la passe de strain courante (`(bar_in_tune / 8) * 8`),
    /// garantissant des reprises A1=A2 et B1=B2 identiques même après invalidation.
    fn chop_notes_for_bar(&mut self, tune: &ActiveTune, bar_in_tune: usize) -> Vec<TimelineNote> {
        let strain_pass_start = (bar_in_tune / BARS_PER_STRAIN) * BARS_PER_STRAIN;
        let mut guitarist = Guitarist::new();
        let mut dummy_id = 0u64;
        let mut dummy_next = || {
            dummy_id += 1;
            dummy_id
        };
        for b in strain_pass_start..bar_in_tune {
            let c = tune.harmony.chords_for_bar(b);
            let _ = guitarist.chop_bar(
                c,
                tune.plan.key,
                tune.plan.mode,
                &self.tuning.trad_harmony,
                &mut dummy_next,
            );
        }
        let chords = tune.harmony.chords_for_bar(bar_in_tune);
        let next_note_id = &mut self.next_note_id;
        let mut next_id = || {
            let id = *next_note_id;
            *next_note_id += 1;
            id
        };
        guitarist.chop_bar(
            chords,
            tune.plan.key,
            tune.plan.mode,
            &self.tuning.trad_harmony,
            &mut next_id,
        )
    }

    /// S'assure que tous les tunes jusqu'à `target_tune_index` inclus ont été tirés.
    fn ensure_tune_generated(&mut self, target_tune_index: usize, rng: &mut dyn RngCore) {
        for idx in 0..=target_tune_index {
            if self.tunes.contains_key(&idx) {
                continue;
            }
            let prev = idx.checked_sub(1).and_then(|prev_idx| self.tunes.get(&prev_idx));
            let prev_plan = prev.map(|t| t.plan);
            let prev_first_cell = prev.map(|t| t.fiddle.cells_a()[0].clone());
            let tune = self.build_tune(idx, prev_plan, prev_first_cell, rng);
            self.tunes.insert(idx, tune);
        }
    }

    /// Construit un [`ActiveTune`] complet (`TunePlan` + `FiddleLine` + `TuneHarmony`).
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
        let plan = match prev_plan {
            None => TunePlan::draw_with_params(rng, None, &self.tuning.reel_fiddle),
            Some(prev) => draw_next_tune_plan(prev, &self.tuning.reel_fiddle, rng),
        };
        let mut fiddle = FiddleLine::from_tuning(plan, &self.tuning, rng);
        if let Some(prev_first) = prev_first_cell {
            for _ in 0..8 {
                if !same_cell_up_to_octave(&fiddle.cells_a()[0], &prev_first) {
                    break;
                }
                fiddle = FiddleLine::from_tuning(plan, &self.tuning, rng);
            }
        }

        let distinct_melody: [Vec<TimelineNote>; TUNE_HARMONY_BARS] = std::array::from_fn(|i| {
            let bar_in_tune =
                if i < BARS_PER_STRAIN { i } else { 2 * BARS_PER_STRAIN + (i - BARS_PER_STRAIN) };
            fiddle.notes_for_bar(bar_in_tune)
        });

        let harmony = TuneHarmony::new(plan, &distinct_melody, &self.tuning.trad_harmony)
            .unwrap_or_else(|_| fallback_tune_harmony(plan));

        ActiveTune { tune_index, plan, fiddle, harmony }
    }

    fn alloc_note_id(&mut self) -> NoteId {
        let id = self.next_note_id;
        self.next_note_id += 1;
        id
    }
}

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

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::params::{PulseSpec, Style, TimeSignature};

    fn make_reel_params() -> MusicalParams {
        let rf = ReelFiddleParams::default();
        let mut params = MusicalParams::default();
        params.set_style(Style::Reel);
        params.bpm = rf.default_bpm;
        params.set_swing_ratio(rf.default_swing_ratio);
        params.time_signature = rf.default_time_signature;
        params
    }

    #[test]
    fn measure_metadata_matches_d3_and_params() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let params = make_reel_params();
        let mut trad_gen = TradGenerator::new(params, TuningParams::default());

        let m1 = trad_gen.generate_measure(1, &mut rng);
        assert_eq!(m1.index, 1);
        assert_eq!(m1.time_signature, TimeSignature::new(2, 2));
        assert_eq!(m1.steps, 16);
        assert_eq!(m1.pulse, PulseSpec { steps_per_pulse: 8, pulses_per_bar: 2 });
        assert!((m1.swing_ratio - 0.56).abs() < f32::EPSILON);
        assert!((m1.tempo - 230.0).abs() < f32::EPSILON);
        assert!(!m1.chord_context.chord_name.is_empty());

        // Les 5 pistes doivent être peuplées en densité de pieds 1
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
            let mut trad_gen = TradGenerator::new(make_reel_params(), TuningParams::default());
            for bar in 1..=64 {
                let _ = trad_gen.generate_measure(bar, &mut rng);
            }
            let tune1 = trad_gen.tune_for_bar(1).map(|t| t.plan);
            let tune2 = trad_gen.tune_for_bar(33).map(|t| t.plan);
            assert!(tune1.is_some() && tune2.is_some());
            if let (Some(p1), Some(p2)) = (tune1, tune2) {
                assert!(
                    is_same_or_relative_plan(p1, p2),
                    "seed {seed}: tune 2 ({p2:?}) n'est ni identique ni relatif à tune 1 ({p1:?})"
                );
            }
        }
    }
}

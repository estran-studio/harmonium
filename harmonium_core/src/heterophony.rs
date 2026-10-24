//! Une seconde lecture du même matériel, sans tirage sur le RNG séquentiel.
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::{
    algo::RngCore,
    ornaments::{BarContext, BeatClass, PhraseEnd, mix},
    style_profile::ProfileError,
    timeline::{Measure, TimelineNote, TrackId},
};

/// Opération de variation de la seconde voix pour une porteuse de la source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Même note ; les agréments V3b de la mélodie sont rejoués avec la même graine.
    Unison,
    /// Même note, agréments V3b tirés avec une graine dérivée distincte.
    Ornament,
    /// Même note, sans agrément.
    Plain,
    /// Tenue jusqu'à la prochaine pulse ou au prochain accord, à la place des traits.
    Simplify,
    /// Une octave au-dessus (repli dans la tessiture).
    Octave,
    /// Tierce diatonique au-dessus, sur un son de l'accord, en fin de phrase seulement.
    Third,
    /// Sixte diatonique au-dessus, sur un son de l'accord, en fin de phrase seulement.
    Sixth,
    /// Silence d'une attaque faible.
    Rest,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variation {
    pub operation: Operation,
    pub weight: u32,
    #[serde(default)]
    pub beats: Vec<BeatClass>,
    #[serde(default)]
    pub phrase_end: PhraseEnd,
    /// Positions 1-based dans la partie ; vide : toutes.
    #[serde(default)]
    pub bars: Vec<usize>,
}

/// Section `[heterophony]` : une seconde lecture de `source`, décrite en données.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Heterophony {
    /// Piste suivie ; `Melody` seulement.
    pub source: TrackId,
    /// Décalage de base, en octaves (−2..2).
    pub octave: i8,
    /// Tessiture de l'instrument (MIDI), au moins une octave.
    pub low: u8,
    pub high: u8,
    /// Facteurs appliqués aux poids des opérations autres qu'unisson, interpolés
    /// linéairement sur l'arousal et la densité bruts (0 → `_low`, 1 → `_high`).
    pub arousal_low: f32,
    pub arousal_high: f32,
    pub density_low: f32,
    pub density_high: f32,
    pub operations: Vec<Variation>,
}
impl Heterophony {
    pub fn validate(&self) -> Result<(), ProfileError> {
        let check = |ok, field, message| {
            if ok {
                Ok(())
            } else {
                Err(ProfileError::new(format!("heterophony.{field}"), message))
            }
        };
        check(self.source == TrackId::Melody, "source", "Melody requis")?;
        check((-2..=2).contains(&self.octave), "octave", "octave dans -2..2")?;
        check(
            self.low <= self.high && self.high <= 127 && self.high - self.low >= 12,
            "low",
            "tessiture MIDI ordonnée couvrant une octave",
        )?;
        for (field, value) in [
            ("arousal_low", self.arousal_low),
            ("arousal_high", self.arousal_high),
            ("density_low", self.density_low),
            ("density_high", self.density_high),
        ] {
            check(
                value.is_finite() && (0.0..=4.0).contains(&value),
                field,
                "facteur fini dans 0..4",
            )?;
        }
        check(
            self.arousal_high >= self.arousal_low && self.density_high >= self.density_low,
            "arousal_high",
            "facteurs non décroissants",
        )?;
        check(
            !self.operations.is_empty() && self.operations.len() <= 64,
            "operations",
            "1..64 opérations",
        )?;
        let total: u64 = self.operations.iter().map(|r| u64::from(r.weight)).sum();
        check(
            total > 0 && total <= u64::from(u32::MAX),
            "operations",
            "somme des poids positive dans u32",
        )?;
        check(
            self.operations.iter().any(|r| {
                r.operation == Operation::Unison
                    && r.weight > 0
                    && r.beats.is_empty()
                    && r.bars.is_empty()
                    && r.phrase_end == PhraseEnd::Any
            }),
            "operations",
            "unisson inconditionnel de poids positif requis",
        )?;
        for rule in &self.operations {
            check(
                rule.bars.len() <= 64 && rule.bars.iter().all(|b| (1..=64).contains(b)),
                "operations.bars",
                "positions 1..64",
            )?;
        }
        Ok(())
    }

    fn register(&self, pitch: i16) -> u8 {
        let mut p = pitch;
        while p < i16::from(self.low) {
            p += 12;
        }
        while p > i16::from(self.high) {
            p -= 12;
        }
        p as u8
    }

    /// Retourne les durées éligibles aux ornements V3b ; zéro = porteuse protégée.
    pub fn prepare(
        &self,
        measure: &mut Measure,
        ctx: &BarContext,
        source_written: &[usize],
        changes: &[crate::timeline::ChordChange],
        next_id: &mut dyn FnMut() -> u64,
    ) -> (Vec<usize>, Vec<usize>) {
        let source = measure.notes_for_track(self.source);
        let mut notes: Vec<TimelineNote> = Vec::with_capacity(source.len());
        let mut written = Vec::with_capacity(source.len());
        let mut independent_steps = Vec::new();
        let factor = (self.arousal_high - self.arousal_low)
            .mul_add(ctx.arousal.clamp(0.0, 1.0), self.arousal_low)
            * (self.density_high - self.density_low)
                .mul_add(ctx.density.clamp(0.0, 1.0), self.density_low);
        let mut skip_until = 0;
        for (i, original) in source.iter().enumerate() {
            if original.start_step < skip_until {
                continue;
            }
            let strong = original.start_step.is_multiple_of(ctx.steps_per_pulse);
            let beat = if strong { BeatClass::Strong } else { BeatClass::Weak };
            let anchor = ctx.phrase_end_bar && i + 1 == source.len();
            let allowed = |r: &&Variation| {
                (r.beats.is_empty() || r.beats.contains(&beat))
                    && (r.bars.is_empty() || r.bars.contains(&ctx.bar_in_part))
                    && match r.phrase_end {
                        PhraseEnd::Any => true,
                        PhraseEnd::Never => !ctx.phrase_end_bar,
                        PhraseEnd::Only => ctx.phrase_end_bar,
                    }
            };
            let weight = |r: &Variation| {
                f64::from(r.weight)
                    * if r.operation == Operation::Unison { 1.0 } else { f64::from(factor) }
            };
            let sum: f64 = self.operations.iter().filter(allowed).map(weight).sum();
            let mut rng =
                ChaCha8Rng::seed_from_u64(mix(mix(ctx.seed, 0x4845_5445), measure.index as u64));
            rng.set_stream(original.start_step as u64);
            let mut draw = f64::from(rng.next_f32()) * sum;
            let mut operation = Operation::Unison;
            for rule in self.operations.iter().filter(allowed) {
                let w = weight(rule);
                if draw < w {
                    operation = rule.operation;
                    break;
                }
                draw -= w;
            }
            if anchor {
                operation = Operation::Unison;
            }
            if operation == Operation::Rest && !strong {
                continue;
            }
            let mut note = original.clone();
            note.id = next_id();
            note.pitch = self.register(i16::from(note.pitch) + 12 * i16::from(self.octave));
            match operation {
                Operation::Octave => note.pitch = self.register(i16::from(note.pitch) + 12),
                Operation::Third | Operation::Sixth if ctx.phrase_end_bar => {
                    let count = if operation == Operation::Third { 2 } else { 5 };
                    let mut pitch = note.pitch;
                    for _ in 0..count {
                        if let Some(next) = (pitch.saturating_add(1)..=self.high)
                            .find(|p| ctx.mode.contains_midi(ctx.key, *p))
                        {
                            pitch = next;
                        }
                    }
                    let chord = changes
                        .iter()
                        .rev()
                        .find(|c| c.start_step <= note.start_step)
                        .map_or(&measure.chord_context, |c| &c.context);
                    let root = (i32::from(ctx.key) + chord.root_offset).rem_euclid(12);
                    let interval = (i32::from(pitch) - root).rem_euclid(12);
                    if [0, if chord.is_minor { 3 } else { 4 }, 7].contains(&interval) {
                        note.pitch = pitch;
                    }
                }
                Operation::Simplify if !ctx.phrase_end_bar => {
                    let boundary =
                        ((note.start_step / ctx.steps_per_pulse) + 1) * ctx.steps_per_pulse;
                    let boundary = changes
                        .iter()
                        .find(|c| c.start_step > note.start_step)
                        .map_or(boundary, |c| boundary.min(c.start_step));
                    if source.get(i + 1).is_some_and(|n| n.start_step < boundary) {
                        note.duration_steps = boundary.min(measure.steps) - note.start_step;
                        skip_until = boundary;
                    }
                }
                _ => {}
            }
            written.push(
                if !anchor && matches!(operation, Operation::Unison | Operation::Ornament) {
                    source_written.get(i).copied().unwrap_or(original.duration_steps)
                } else {
                    0
                },
            );
            if operation == Operation::Ornament {
                independent_steps.push(note.start_step);
            }
            notes.push(note);
        }
        measure.tracks.insert(TrackId::Countermelody, notes);
        (written, independent_steps)
    }

    /// La clôture suit la dernière porteuse réellement entendue, y compris le
    /// passage de densité V3b qui peut scinder la dernière tenue écrite.
    pub fn close_cadence(&self, measure: &mut Measure, next_id: &mut dyn FnMut() -> u64) {
        let [Some(melody), Some(notes)] =
            measure.tracks.get_disjoint_mut([&self.source, &TrackId::Countermelody])
        else {
            return;
        };
        let Some(lead) = melody.iter().max_by_key(|n| (n.start_step, std::cmp::Reverse(n.id)))
        else {
            return;
        };
        let Some(mut closing) = notes.last().cloned() else { return };
        let step = lead.start_step;
        let pitch = self.register(i16::from(lead.pitch) + 12 * i16::from(self.octave));
        if closing.start_step == step && closing.pitch == pitch {
            return;
        }
        notes.retain(|n| n.start_step < step);
        for note in notes.iter_mut() {
            note.duration_steps = note.duration_steps.min(step - note.start_step);
            note.performance.breath = note.performance.breath.min(
                (note.duration_steps as u32 * u32::from(note.performance.gate)).saturating_sub(1),
            );
        }
        closing.id = next_id();
        closing.pitch = pitch;
        closing.start_step = step;
        closing.duration_steps = lead.duration_steps;
        closing.performance.breath = closing.performance.breath.min(
            (closing.duration_steps as u32 * u32::from(closing.performance.gate)).saturating_sub(1),
        );
        notes.push(closing);
    }

    /// Veto sur la musique réalisée, y compris les secondes créées par V3b et les tenues :
    /// à chaque temps fort, aucune note de la seconde voix ne sonne en seconde (mineure ou
    /// majeure, composée comprise) avec une note de la mélodie qui sonne au même instant.
    /// On ne change ni la mélodie ni l'harmonie : une note de la seconde voix qui déborde sur
    /// l'attaque suivante de la mélodie est raccourcie jusqu'à cette attaque ; une note qui
    /// commence sous une note tenue de la mélodie s'abstient.
    pub fn veto(&self, measure: &mut Measure) {
        let pulse = usize::from(measure.pulse.steps_per_pulse).max(1);
        let steps = measure.steps;
        let [Some(melody), Some(notes)] =
            measure.tracks.get_disjoint_mut([&self.source, &TrackId::Countermelody])
        else {
            return;
        };
        let span = |n: &TimelineNote| {
            let on = n.start_step as i64 * 1000 + i64::from(n.performance.offset);
            on..on + n.performance.duration(n.duration_steps)
        };
        // Attaque de la note de la mélodie avec laquelle `note` sonne en seconde à un temps fort.
        let clash = |note: &TimelineNote| {
            let own = span(note);
            (0..steps).step_by(pulse).find_map(|step| {
                let time = step as i64 * 1000;
                if !own.contains(&time) {
                    return None;
                }
                melody.iter().find_map(|lead| {
                    let sounding = span(lead);
                    (sounding.contains(&time)
                        && matches!(note.pitch.abs_diff(lead.pitch) % 12, 1 | 2))
                    .then_some(sounding.start)
                })
            })
        };
        notes.retain_mut(|note| {
            while let Some(lead_start) = clash(note) {
                let on = span(note).start;
                if lead_start <= on {
                    return false;
                }
                let full = note.duration_steps.max(1) as i64 * i64::from(note.performance.gate);
                let Ok(breath) = u32::try_from(full - (lead_start - on)) else { return false };
                note.performance.breath = breath;
            }
            true
        });
    }
}

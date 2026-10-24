//! Réalisation des ornements sur une mesure de mélodie déjà interprétée.
//!
//! Toute la mesure est décidée en un passage : une note porte au plus un
//! ornement, la note de passage de densité (sans tirage) passe avant les types
//! tirés, et les temps se calculent en millièmes de pas **composés** avec le
//! décalage, le gate et la respiration de l'interprétation : chaque note
//! publiée garde `start_step` sur la grille et un décalage borné.
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::{
    BarContext, BeatClass, IntervalClass, OrnamentKind, OrnamentRule, Ornaments, PhraseEnd, mix,
};
use crate::{
    algo::RngCore,
    timeline::{Articulation, Measure, NoteId, TimelineNote, TrackId},
};

/// Attaque la plus en avance permise : la borne de l'interprétation (0,49 pas).
const EARLIEST: i64 = -490;
/// Durée minimale (millièmes de pas) que garde une note écourtée par une appogiature.
const MIN_KEPT: i64 = 250;
/// Durée minimale d'une appogiature (millièmes de pas).
const MIN_GRACE: i64 = 40;
/// Durée d'appogiature (pas) quand le profil n'en fixe pas.
const DEFAULT_SPAN: f32 = 0.3;

fn onset(note: &TimelineNote) -> i64 {
    note.start_step as i64 * 1000 + i64::from(note.performance.offset)
}
fn length(note: &TimelineNote) -> i64 {
    note.performance.duration(note.duration_steps)
}
fn end(note: &TimelineNote) -> i64 {
    onset(note) + length(note)
}

/// Place `note` à `on` (millièmes de pas) pour `len` : grille la plus proche,
/// reste en décalage, durée écrite en pas entiers et gate pour le reste.
fn place(note: &mut TimelineNote, on: i64, len: i64, steps: usize) {
    let step = (on + 500).div_euclid(1000).clamp(0, steps as i64 - 1);
    note.start_step = step as usize;
    note.performance.offset =
        (on - step * 1000).clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16;
    let len = len.max(1);
    let whole = ((len + 999) / 1000).max(1);
    note.duration_steps = whole as usize;
    note.performance.gate = ((len + whole / 2) / whole).clamp(1, 1000) as u16;
    note.performance.breath = 0;
}

fn scaled(velocity: u8, scale: f32) -> u8 {
    (f32::from(velocity) * scale).round().clamp(1.0, 127.0) as u8
}

fn interval_class(prev: Option<u8>, pitch: u8) -> Option<IntervalClass> {
    prev.map(|p| match p.abs_diff(pitch) {
        0 => IntervalClass::Repeated,
        1 | 2 => IntervalClass::Step,
        _ => IntervalClass::Leap,
    })
}

struct Candidate<'a> {
    kind: OrnamentKind,
    rule: &'a OrnamentRule,
    min_steps: usize,
    multiplier: f32,
}
impl Candidate<'_> {
    /// `duration` : durée écrite pour les types tirés (stable sous les émotions), durée
    /// jouée pour la note de passage, qui scinde la tenue réellement publiée.
    fn permits(
        &self,
        note: &TimelineNote,
        duration: usize,
        prev: Option<u8>,
        ctx: &BarContext,
    ) -> bool {
        if duration < self.min_steps || ctx.protected {
            return false;
        }
        let rule = self.rule;
        if (rule.phrase_end == PhraseEnd::Never && ctx.phrase_end_bar)
            || (rule.phrase_end == PhraseEnd::Only && !ctx.phrase_end_bar)
        {
            return false;
        }
        let beat = if note.start_step.is_multiple_of(ctx.steps_per_pulse.max(1)) {
            BeatClass::Strong
        } else {
            BeatClass::Weak
        };
        (rule.beats.is_empty() || rule.beats.contains(&beat))
            && (rule.intervals.is_empty()
                || interval_class(prev, note.pitch).is_some_and(|c| rule.intervals.contains(&c)))
    }
}

pub(super) fn apply(
    rules: &Ornaments,
    measure: &mut Measure,
    ctx: &BarContext,
    written: &[usize],
    next_id: &mut dyn FnMut() -> NoteId,
) -> bool {
    let steps = measure.steps;
    let steps_per_eighth = measure.steps_per_quarter() as f32 / 2.0;
    let Some(slot) = measure.tracks.get_mut(&TrackId::Melody) else { return false };
    if slot.is_empty() || steps == 0 {
        return false;
    }
    let original = std::mem::take(slot);
    let total = original.len() as f32;
    let candidates: Vec<Candidate> = rules
        .active()
        .filter(|(kind, rule)| {
            if *kind == OrnamentKind::Passing {
                rule.density_curve.is_some()
            } else {
                rules.draws(rule)
            }
        })
        .map(|(kind, rule)| Candidate {
            kind,
            rule,
            min_steps: ((rule.min_carrier_eighths * steps_per_eighth - 1e-3).ceil().max(1.0)
                as usize)
                .max(if kind == OrnamentKind::Passing { 2 } else { 1 }),
            multiplier: rule.arousal_multiplier(ctx.arousal),
        })
        .collect();
    let prevs: Vec<Option<u8>> =
        (0..original.len()).map(|i| i.checked_sub(1).map(|j| original[j].pitch)).collect();

    // Probabilité par porteuse : l'espérance de la mesure (taux pour 100 notes
    // × notes de la mesure) répartie sur les porteuses permises du type.
    let probabilities: Vec<f32> = candidates
        .iter()
        .map(|c| {
            if c.kind == OrnamentKind::Passing {
                return 0.0;
            }
            let eligible = original
                .iter()
                .enumerate()
                .filter(|(i, note)| {
                    c.permits(
                        note,
                        written.get(*i).copied().unwrap_or(note.duration_steps),
                        prevs[*i],
                        ctx,
                    )
                })
                .count();
            if eligible == 0 {
                return 0.0;
            }
            let expected =
                c.rule.rate_at(ctx.bar_in_part) * rules.rate_scale * c.multiplier / 100.0 * total;
            (expected / eligible as f32).clamp(0.0, 1.0)
        })
        .collect();
    let coverage = candidates
        .iter()
        .find(|c| c.kind == OrnamentKind::Passing)
        .and_then(|c| {
            c.rule
                .density_curve
                .as_ref()
                .map(|curve| (curve.eval(ctx.density) * c.multiplier).clamp(0.0, 1.0))
        })
        .unwrap_or(0.0);

    // Graine de la mesure : contenu écrit (hauteurs, positions), donc une copie
    // exacte porte les mêmes ornements ; `repeat_variation` la dissocie.
    let content = original.iter().fold(0xCBF2_9CE4_8422_2325u64, |h, n| {
        mix(h, ((n.start_step as u64) << 8) | u64::from(n.pitch))
    });
    let mut salt = 0;
    if ctx.repeat && rules.repeat_variation > 0.0 && ctx.seed != 0 {
        let mut bar = ChaCha8Rng::seed_from_u64(mix(ctx.seed, 0xA11C_E5ED));
        bar.set_stream(ctx.bar_index as u64);
        if bar.next_f32() < rules.repeat_variation {
            salt = ctx.bar_index as u64 + 1;
        }
    }
    let bar_seed = mix(mix(ctx.seed, content), salt);

    let mut work = Work {
        ctx,
        steps,
        next_id,
        out: Vec::with_capacity(original.len() + 8),
        tails: Vec::new(),
    };
    let mut changed = false;
    for (i, note) in original.into_iter().enumerate() {
        let prev = prevs[i];
        let mut rng = None;
        let mut chosen = None;
        if let Some(c) = candidates.iter().find(|c| c.kind == OrnamentKind::Passing)
            && coverage > 0.0
            && (note.start_step as f32 / steps as f32) < coverage
            && c.permits(&note, note.duration_steps, prev, ctx)
        {
            chosen = Some(c);
        } else {
            let applicable: Vec<(&Candidate, f32)> = candidates
                .iter()
                .zip(&probabilities)
                .filter(|(c, p)| {
                    **p > 0.0
                        && c.permits(
                            &note,
                            written.get(i).copied().unwrap_or(note.duration_steps),
                            prev,
                            ctx,
                        )
                })
                .map(|(c, p)| (c, *p))
                .collect();
            if !applicable.is_empty() {
                let mut r = ChaCha8Rng::seed_from_u64(bar_seed);
                r.set_stream(note.start_step as u64);
                let u = r.next_f32();
                let mut cumulative = 0.0;
                for (c, p) in applicable {
                    cumulative += p;
                    if u < cumulative {
                        chosen = Some(c);
                        break;
                    }
                }
                rng = Some(r);
            }
        }
        let note = match chosen {
            Some(c) => match work.realize(c, note, prev, rng.as_mut()) {
                Ok(()) => {
                    changed = true;
                    continue;
                }
                Err(note) => note,
            },
            None => note,
        };
        work.tails = vec![work.out.len()];
        work.out.push(note);
    }
    let mut out = work.out;
    if changed {
        // À temps égal, la note écrite précède la note ajoutée (identifiants croissants).
        out.sort_by_key(|n| (onset(n), n.id));
    }
    if let Some(slot) = measure.tracks.get_mut(&TrackId::Melody) {
        *slot = out;
    }
    changed
}

struct Work<'a> {
    ctx: &'a BarContext,
    steps: usize,
    next_id: &'a mut dyn FnMut() -> NoteId,
    out: Vec<TimelineNote>,
    /// Notes publiées qui portent la fin de la dernière porteuse.
    tails: Vec<usize>,
}

/// Élément d'une figure : hauteur (`None` = la porteuse) et poids de durée.
type Figure = Vec<(Option<u8>, f32)>;

impl Work<'_> {
    fn push(&mut self, note: TimelineNote) -> usize {
        self.out.push(note);
        self.out.len() - 1
    }

    fn added(&mut self, template: &TimelineNote, pitch: u8, velocity: u8) -> TimelineNote {
        TimelineNote {
            id: (self.next_id)(),
            pitch,
            velocity,
            articulation: Articulation::Normal,
            ..template.clone()
        }
    }

    /// Voisin diatonique à `count` degrés de la porteuse, dans la tessiture.
    fn scale_step(&self, pitch: u8, direction: i16, count: u8) -> Option<u8> {
        let (lo, hi) = self.ctx.tessitura;
        let mut current = i16::from(pitch);
        for _ in 0..count {
            current = (1..=3).map(|d| current + direction * d).find(|&p| {
                (i16::from(lo)..=i16::from(hi)).contains(&p)
                    && self.ctx.mode.contains_midi(self.ctx.key, p as u8)
            })?;
        }
        Some(current as u8)
    }

    fn realize(
        &mut self,
        c: &Candidate,
        note: TimelineNote,
        prev: Option<u8>,
        rng: Option<&mut ChaCha8Rng>,
    ) -> Result<(), TimelineNote> {
        match c.kind {
            OrnamentKind::Passing => self.passing(note),
            OrnamentKind::GraceSingle | OrnamentKind::GraceMultiple => {
                self.grace(c, note, prev, rng)
            }
            OrnamentKind::DoubleStop | OrnamentKind::Chord => self.stop(c, note, rng),
            _ => self.figure(c, note),
        }
    }

    /// Scinde la fin de la tenue : la porteuse perd un pas, la note voisine le prend.
    fn passing(&mut self, mut note: TimelineNote) -> Result<(), TimelineNote> {
        let Some(pitch) = self.scale_step(note.pitch, 1, 1) else { return Err(note) };
        let start = note.start_step + note.duration_steps - 1;
        let passing = TimelineNote {
            id: (self.next_id)(),
            pitch,
            start_step: start,
            duration_steps: 1,
            articulation: Articulation::Normal,
            performance: Default::default(),
            ..note.clone()
        };
        note.duration_steps -= 1;
        let room =
            (note.duration_steps as u32 * u32::from(note.performance.gate)).saturating_sub(1);
        note.performance.breath = note.performance.breath.min(room);
        self.push(note);
        let tail = self.push(passing);
        self.tails = vec![tail];
        Ok(())
    }

    /// Appogiature(s) avant la porteuse, sur le temps de la note précédente : la
    /// porteuse reste sur la grille, la note précédente est écourtée d'autant
    /// (respiration) quand elle dépasse l'attaque de l'appogiature.
    fn grace(
        &mut self,
        c: &Candidate,
        note: TimelineNote,
        prev: Option<u8>,
        rng: Option<&mut ChaCha8Rng>,
    ) -> Result<(), TimelineNote> {
        let count =
            if c.kind == OrnamentKind::GraceMultiple { c.rule.count.unwrap_or(2) } else { 1 };
        let on = onset(&note);
        let span =
            ((c.rule.span.unwrap_or(DEFAULT_SPAN) * 1000.0).round() as i64).min(on - EARLIEST);
        if span < MIN_GRACE * i64::from(count) {
            return Err(note);
        }
        let pitches: Vec<u8> = if count == 1 {
            let open = |p: Option<u8>| p.filter(|&p| Some(p) != prev);
            let up = open(self.scale_step(note.pitch, 1, 1));
            let down = open(self.scale_step(note.pitch, -1, 1));
            let far = open(self.scale_step(note.pitch, 1, 2));
            let draw = rng.map_or(0.0, |r| r.next_f32());
            match (up, down, far) {
                (Some(u), d, _) if draw < 0.7 || d.is_none() => vec![u],
                (_, Some(d), _) => vec![d],
                (Some(u), None, _) => vec![u],
                (None, None, Some(f)) => vec![f],
                _ => return Err(note),
            }
        } else {
            let run: Option<Vec<u8>> =
                (1..=count).rev().map(|k| self.scale_step(note.pitch, 1, k)).collect();
            match run {
                Some(run) => run,
                None => return Err(note),
            }
        };
        let from = on - span;
        let cuts: Vec<usize> =
            self.tails.iter().copied().filter(|&i| end(&self.out[i]) > from).collect();
        if cuts.iter().any(|&i| from - onset(&self.out[i]) < MIN_KEPT) {
            return Err(note);
        }
        for &i in &cuts {
            let cut = (end(&self.out[i]) - from) as u32;
            self.out[i].performance.breath += cut;
        }
        let each = span / i64::from(count);
        let velocity = scaled(note.velocity, c.rule.velocity_scale);
        for (k, pitch) in pitches.into_iter().enumerate() {
            let mut grace = self.added(&note, pitch, velocity);
            grace.performance = Default::default();
            let start = from + each * k as i64;
            let len = if k + 1 == usize::from(count) { on - start } else { each };
            place(&mut grace, start, len, self.steps);
            self.push(grace);
        }
        let tail = self.push(note);
        self.tails = vec![tail];
        Ok(())
    }

    /// Seconde (ou troisième) note simultanée sur cordes voisines, même timing.
    fn stop(
        &mut self,
        c: &Candidate,
        note: TimelineNote,
        rng: Option<&mut ChaCha8Rng>,
    ) -> Result<(), TimelineNote> {
        let (lo, hi) = self.ctx.tessitura;
        let strings = &c.rule.strings;
        let Some(s) = strings.iter().rposition(|&open| open <= note.pitch) else {
            return Err(note);
        };
        let playable = |p: u8| {
            let interval = p.abs_diff(note.pitch);
            (lo..=hi).contains(&p)
                && p != note.pitch
                && (c.rule.min_interval..=c.rule.max_interval).contains(&interval)
                && self.ctx.mode.contains_midi(self.ctx.key, p)
        };
        let string = |i: isize| {
            usize::try_from(i).ok().and_then(|i| strings.get(i)).copied().filter(|&p| playable(p))
        };
        let s = s as isize;
        let options: Vec<Vec<u8>> = if c.kind == OrnamentKind::DoubleStop {
            let mut options: Vec<Vec<u8>> =
                [s - 1, s + 1].into_iter().filter_map(string).map(|p| vec![p]).collect();
            // Tierce diatonique sous la porteuse, au doigt, sur la corde voisine.
            if let Some(third) =
                self.scale_step(note.pitch, -1, 2).filter(|&p| playable(p) && p >= strings[0])
            {
                options.push(vec![third]);
            }
            options
        } else {
            [(s - 1, s - 2), (s - 1, s + 1), (s + 1, s + 2)]
                .into_iter()
                .filter_map(|(a, b)| Some(vec![string(a)?, string(b)?]))
                .collect()
        };
        if options.is_empty() {
            return Err(note);
        }
        let pick = rng.map_or(0, |r| r.next_range_usize(0..options.len()));
        let velocity = scaled(note.velocity, c.rule.velocity_scale);
        let extra: Vec<TimelineNote> =
            options[pick].iter().map(|&p| self.added(&note, p, velocity)).collect();
        let mut tails = vec![self.push(note)];
        for added in extra {
            tails.push(self.push(added));
        }
        self.tails = tails;
        Ok(())
    }

    /// Figure diatonique dans la durée jouée de la porteuse.
    fn figure(&mut self, c: &Candidate, note: TimelineNote) -> Result<(), TimelineNote> {
        let up = self.scale_step(note.pitch, 1, 1);
        let down = self.scale_step(note.pitch, -1, 1);
        let total = length(&note);
        let figure: Figure = match c.kind {
            OrnamentKind::TripletRepeated => vec![(None, 1.0); 3],
            OrnamentKind::Roll => match (up, down) {
                (Some(u), Some(d)) => {
                    vec![(None, 0.36), (Some(u), 0.16), (None, 0.16), (Some(d), 0.16), (None, 0.16)]
                }
                _ => return Err(note),
            },
            OrnamentKind::Turn => match (up, down) {
                (Some(u), Some(d)) => {
                    vec![(Some(u), 0.14), (None, 0.18), (Some(d), 0.14), (None, 0.54)]
                }
                _ => return Err(note),
            },
            OrnamentKind::Trill => {
                let Some(u) = up else { return Err(note) };
                let n = ((total / 200).clamp(3, 31) as usize) | 1;
                (0..n).map(|i| (if i % 2 == 0 { None } else { Some(u) }, 1.0)).collect()
            }
            _ => return Err(note),
        };
        let sum: f32 = figure.iter().map(|p| p.1).sum();
        let on = onset(&note);
        let gap = if c.kind == OrnamentKind::TripletRepeated { 0.9 } else { 1.0 };
        let template = note.clone();
        let mut carrier = Some(note);
        let velocity = scaled(template.velocity, c.rule.velocity_scale);
        let mut cumulative = 0.0;
        let mut last = 0;
        for &(pitch, weight) in &figure {
            let start = on + (total as f32 * cumulative / sum).round() as i64;
            cumulative += weight;
            let stop = on + (total as f32 * cumulative / sum).round() as i64;
            let len = (((stop - start) as f32) * gap).round() as i64;
            let mut piece = match pitch {
                Some(p) => self.added(&template, p, velocity),
                None => match carrier.take() {
                    Some(own) => own,
                    None => self.added(&template, template.pitch, velocity),
                },
            };
            place(&mut piece, start, len, self.steps);
            last = self.push(piece);
        }
        self.tails = vec![last];
        Ok(())
    }
}

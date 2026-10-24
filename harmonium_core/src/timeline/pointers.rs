//! Writehead principal et Playhead audio : événements préparés, grille indépendante.
use super::{Measure, MusicalPosition, ScoreTimeline, TempoMap, TrackId};
use crate::{events::AudioEvent, params::TimeSignature};

pub struct Writehead {
    pub current_bar: usize,
    pub lookahead: usize,
    pub timeline: ScoreTimeline,
    pub tempo_map: TempoMap,
}
impl Writehead {
    #[must_use]
    pub fn new(sample_rate: f64, ticks_per_beat: usize) -> Self {
        Self {
            current_bar: 1,
            lookahead: 4,
            timeline: ScoreTimeline::with_default_capacity(),
            tempo_map: TempoMap::new(sample_rate, ticks_per_beat),
        }
    }
    /// Vrai si une mesure reste à générer devant `playhead_bar`.
    ///
    /// La fenêtre est bornée par la capacité de la timeline : générer plus loin
    /// évincerait des mesures pas encore jouées (jamais envoyées au moteur de
    /// lecture, qui attendrait alors indéfiniment la mesure évincée).
    #[must_use]
    pub fn needs_generation(&self, playhead_bar: usize) -> bool {
        self.current_bar < playhead_bar + self.lookahead.min(self.timeline.capacity())
    }
    pub fn commit_measure(&mut self, measure: Measure) {
        self.current_bar = measure.index + 1;
        self.timeline.push_measure(measure);
    }
    pub fn reset(&mut self) {
        self.current_bar = 1;
        self.timeline.clear();
    }
}

/// Événement musical trié sur le thread principal ; temps en millièmes de pas.
#[derive(Clone, Copy, Debug)]
pub struct PreparedEvent {
    pub time: i64,
    pub id: u64,
    pub channel: u8,
    pub pitch: u8,
    pub velocity: u8,
    pub on: bool,
}
impl Measure {
    /// Appeler après toute modification de notes, avant le handoff audio.
    pub fn prepare_playback(&mut self) {
        self.playback_events.clear();
        for track in TrackId::ALL {
            if let Some(notes) = self.tracks.get(&track) {
                for note in notes {
                    let time = note.start_step as i64 * 1000 + i64::from(note.performance.offset);
                    let event = PreparedEvent {
                        time,
                        id: note.id,
                        channel: track.channel(),
                        pitch: note.pitch,
                        velocity: note.velocity,
                        on: true,
                    };
                    self.playback_events.push(event);
                    self.playback_events.push(PreparedEvent {
                        time: time + note.performance.duration(note.duration_steps),
                        on: false,
                        ..event
                    });
                }
            }
        }
        self.playback_events.sort_by_key(|e| (e.time, e.on, e.channel, e.id, e.pitch));
    }
}

/// Temps à l'intérieur du pas ; la conversion en frames suit sa durée gelée.
#[derive(Clone, Debug)]
pub struct StepEvent {
    pub offset: u16,
    pub event: AudioEvent,
}
#[derive(Clone, Copy)]
struct Spill {
    bar: usize,
    event: PreparedEvent,
}

/// Ne déplace que des pointeurs ; aucun tri allouant, aucune libération de mesure.
pub struct Playhead {
    pub position: MusicalPosition,
    current_measure: Option<Box<Measure>>,
    finished_measure: Option<Box<Measure>>,
    ticks_per_beat: usize,
    time_signature: TimeSignature,
    candidates: Vec<PreparedEvent>,
    events: Vec<StepEvent>,
    pending: Vec<Spill>,
    active: [[Option<u64>; 128]; TrackId::ALL.len()],
    anticipated_bar: Option<usize>,
    clamp_opening: bool,
}
// Les profils bornent la grille à 4096 pas ; une piste ne peut émettre plus
// de 128 hauteurs simultanées. Capacité fixe généreuse : au-delà, l'événement
// est abandonné (jamais de croissance ni de panique dans le callback).
const EVENT_CAP: usize = 8192;

fn push_capped<T>(items: &mut Vec<T>, item: T) {
    debug_assert!(items.len() < EVENT_CAP, "capacité d'événements dépassée");
    if items.len() < EVENT_CAP {
        items.push(item);
    }
}
impl Playhead {
    #[must_use]
    pub fn new(_sample_rate: f64, ticks_per_beat: usize) -> Self {
        Self {
            position: MusicalPosition::new(1, 1, 0),
            current_measure: None,
            finished_measure: None,
            ticks_per_beat,
            time_signature: TimeSignature::default(),
            candidates: Vec::with_capacity(EVENT_CAP),
            events: Vec::with_capacity(EVENT_CAP),
            pending: Vec::with_capacity(EVENT_CAP),
            active: [[None; 128]; TrackId::ALL.len()],
            anticipated_bar: None,
            clamp_opening: true,
        }
    }
    pub fn load_measure(&mut self, measure: Box<Measure>) {
        debug_assert!(self.current_measure.is_none());
        self.time_signature = measure.time_signature;
        self.clamp_opening = self.anticipated_bar != Some(measure.index);
        let length = measure.steps.max(1) as i64 * 1000;
        for event in &measure.playback_events {
            if !event.on && event.time >= length {
                push_capped(
                    &mut self.pending,
                    Spill {
                        bar: measure.index + (event.time / length) as usize,
                        event: PreparedEvent { time: event.time % length, ..*event },
                    },
                );
            }
        }
        self.current_measure = Some(measure);
    }
    pub fn unload_current(&mut self) -> Option<Box<Measure>> {
        self.current_measure.take()
    }
    pub fn take_finished(&mut self) -> Option<Box<Measure>> {
        self.finished_measure.take()
    }
    pub fn event(&self, index: usize) -> Option<&StepEvent> {
        self.events.get(index)
    }

    /// Avance la grille d'un pas et prépare ses événements fractionnaires.
    /// Le pas final emprunte les attaques négatives de la mesure préchargée.
    pub fn tick(&mut self, next: Option<&Measure>) {
        self.events.clear();
        self.candidates.clear();
        let bar = self.position.bar;
        let step = self.position.step_in_bar(self.ticks_per_beat);
        let start = step as i64 * 1000;
        let end = start + 1000;
        if let Some(measure) = &self.current_measure {
            let first = measure.playback_events.partition_point(|e| e.time < start);
            let last = measure.playback_events.partition_point(|e| e.time < end);
            for event in &measure.playback_events[first..last] {
                push_capped(
                    &mut self.candidates,
                    PreparedEvent { time: event.time - start, ..*event },
                );
            }
            if step == 0 && self.clamp_opening {
                for event in measure.playback_events.iter().take_while(|e| e.time < 0) {
                    // Pas de pré-roll : l'attaque part à la frame 0 et sa fin, si
                    // elle est aussi avant la barre, suit d'un millième de pas.
                    let time = i64::from(!event.on);
                    push_capped(&mut self.candidates, PreparedEvent { time, ..*event });
                }
            }
            if step + 1 == measure.steps
                && let Some(next) = next
            {
                for event in next.playback_events.iter().take_while(|e| e.time < 0) {
                    push_capped(
                        &mut self.candidates,
                        PreparedEvent { time: 1000 + event.time, ..*event },
                    );
                }
                self.anticipated_bar = Some(next.index);
            }
        }
        let mut i = 0;
        while i < self.pending.len() {
            let p = self.pending[i];
            if p.bar < bar || (p.bar == bar && p.event.time < end) {
                push_capped(
                    &mut self.candidates,
                    PreparedEvent { time: (p.event.time - start).max(0), ..p.event },
                );
                self.pending.swap_remove(i);
            } else {
                i += 1;
            }
        }
        self.candidates.sort_unstable_by_key(|e| (e.time, e.on, e.channel, e.id, e.pitch));
        for candidate in &self.candidates {
            let active =
                &mut self.active[usize::from(candidate.channel)][usize::from(candidate.pitch)];
            let offset = candidate.time as u16;
            if candidate.on {
                if active.is_some() {
                    push_capped(
                        &mut self.events,
                        StepEvent {
                            offset,
                            event: AudioEvent::NoteOff {
                                note: candidate.pitch,
                                channel: candidate.channel,
                            },
                        },
                    );
                }
                *active = Some(candidate.id);
                push_capped(
                    &mut self.events,
                    StepEvent {
                        offset,
                        event: AudioEvent::NoteOn {
                            note: candidate.pitch,
                            velocity: candidate.velocity,
                            channel: candidate.channel,
                        },
                    },
                );
            } else if *active == Some(candidate.id) {
                *active = None;
                push_capped(
                    &mut self.events,
                    StepEvent {
                        offset,
                        event: AudioEvent::NoteOff {
                            note: candidate.pitch,
                            channel: candidate.channel,
                        },
                    },
                );
            }
        }
        let steps = self
            .current_measure
            .as_ref()
            .map_or_else(|| self.time_signature.steps_per_bar(self.ticks_per_beat), |m| m.steps)
            .max(1);
        if step + 1 >= steps {
            self.position = MusicalPosition::new(bar + 1, 1, 0);
            debug_assert!(self.finished_measure.is_none());
            self.finished_measure = self.current_measure.take();
        } else {
            self.position = MusicalPosition::new(
                bar,
                (step + 1) / self.ticks_per_beat + 1,
                (step + 1) % self.ticks_per_beat,
            );
        }
    }
    #[must_use]
    pub fn needs_measure(&self) -> bool {
        self.current_measure.is_none()
    }
    #[must_use]
    pub fn current_bar(&self) -> usize {
        self.position.bar
    }
    pub fn seek_to_bar(&mut self, bar: usize) {
        self.position = MusicalPosition::new(bar, 1, 0);
        self.pending.clear();
        self.events.clear();
        self.active = [[None; 128]; TrackId::ALL.len()];
        self.anticipated_bar = None;
        self.clamp_opening = true;
    }
    pub fn reset(&mut self) {
        self.seek_to_bar(1);
    }
}

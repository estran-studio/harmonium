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

/// Rang de tri d'un événement préparé : l'attaque liée ou legato précède la
/// fin coïncidente pour pouvoir fusionner (liaison) ou chevaucher (legato),
/// puis fins, puis attaques. Sans lien, l'ordre historique (fins avant
/// attaques) est inchangé.
fn tied_rank(event: &PreparedEvent) -> u8 {
    if event.on { u8::from(!(event.tied || event.legato)) * 2 } else { 1 }
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
    /// Attaque liée : fusionne avec la fin coïncidente de la note active.
    pub tied: bool,
    /// Attaque legato : chevauche la note tenue du canal (voir [`Playhead`]).
    pub legato: bool,
    /// Fin forcée (chevauchement legato) : émise même si la note active ne
    /// correspond plus ; ne touche jamais à l'état des voix.
    pub force_off: bool,
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
                        tied: note.articulation == crate::timeline::Articulation::Tied,
                        legato: note.articulation == crate::timeline::Articulation::Legato,
                        force_off: false,
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
        self.playback_events.sort_by_key(|e| (e.time, tied_rank(e), e.channel, e.id, e.pitch));
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

/// Chevauchement du legato monophonique, en millièmes de pas : la fin de la
/// note précédente est repoussée d'autant au-delà de l'attaque pour que la
/// note nouvelle ne soit jamais détachée (≈ 0,2 pas ; ≈ 15–50 ms selon le
/// tempo). Borné sous un pas : l'attaque et son chevauchement restent voisins.
const LEGATO_OVERLAP_MILLI_STEPS: i64 = 200;

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
        // Longueur de la mesure en millièmes de pas, pour reporter un
        // chevauchement legato au-delà du pas courant (même barre ou suivante).
        let length = self
            .current_measure
            .as_ref()
            .map_or_else(|| self.time_signature.steps_per_bar(self.ticks_per_beat), |m| m.steps)
            .max(1) as i64
            * 1000;
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
        self.candidates.sort_unstable_by_key(|e| (e.time, tied_rank(e), e.channel, e.id, e.pitch));
        let mut j = 0;
        while j < self.candidates.len() {
            let candidate = self.candidates[j];
            let channel = usize::from(candidate.channel);
            let pitch = usize::from(candidate.pitch);
            let offset = candidate.time as u16;
            if candidate.on {
                if candidate.tied && self.active[channel][pitch].is_some() {
                    // Attaque liée : la note active se prolonge sans réattaque
                    // si sa fin coïncide ici (même instant, canal, hauteur).
                    // Sinon, repli sur l'attaque normale : jamais de note bloquée.
                    let mut k = j + 1;
                    let mut merge = false;
                    while k < self.candidates.len() && self.candidates[k].time == candidate.time {
                        let later = self.candidates[k];
                        if !later.on
                            && later.channel == candidate.channel
                            && later.pitch == candidate.pitch
                            && Some(later.id) == self.active[channel][pitch]
                        {
                            merge = true;
                            break;
                        }
                        k += 1;
                    }
                    if merge {
                        self.active[channel][pitch] = Some(candidate.id);
                        j += 1;
                        continue;
                    }
                }
                if candidate.legato && self.legato_attack(candidate, start, end, length) {
                    j += 1;
                    continue;
                }
                if self.active[channel][pitch].is_some() {
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
                self.active[channel][pitch] = Some(candidate.id);
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
            } else if candidate.force_off {
                // Fin forcée du chevauchement legato : la note précédente est
                // déjà remplacée, on l'émet ici sans toucher à l'état des voix.
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
            } else if self.active[channel][pitch] == Some(candidate.id) {
                self.active[channel][pitch] = None;
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
            j += 1;
        }
        // L'ordre des événements suit l'offset croissant (le playback s'arrête au
        // premier événement futur). À offset égal, NoteOff avant NoteOn (l'ordre
        // détaché) ; le legato, lui, écarte sa fin de l'attaque (offset +200),
        // donc la paire ne partage jamais le même offset. `sort_unstable_by_key`
        // n'alloue pas (contrairement au tri stable), exigé dans le callback.
        self.events
            .sort_unstable_by_key(|e| (e.offset, matches!(e.event, AudioEvent::NoteOn { .. })));
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
    /// Legato monophonique : la note nouvelle (autre hauteur) attaque pendant
    /// que la note tenue du canal se prolonge encore [`LEGATO_OVERLAP_MILLI_STEPS`]
    /// millièmes de pas, au lieu de se détacher. `start`/`end`/`length` sont en
    /// millièmes de pas absolus dans la mesure. Retourne `true` si le candidat
    /// a été consommé ; sinon attaque normale.
    fn legato_attack(
        &mut self,
        candidate: PreparedEvent,
        start: i64,
        end: i64,
        length: i64,
    ) -> bool {
        let channel = usize::from(candidate.channel);
        let held = self.active[channel]
            .iter()
            .enumerate()
            .find(|(p, slot)| *p != usize::from(candidate.pitch) && slot.is_some())
            .and_then(|(p, slot)| slot.map(|id| (p, id)));
        let Some((old_pitch, old_id)) = held else { return false };

        // Libère l'ancienne note, enregistre la nouvelle, émet l'attaque.
        self.active[channel][old_pitch] = None;
        self.active[channel][usize::from(candidate.pitch)] = Some(candidate.id);
        push_capped(
            &mut self.events,
            StepEvent {
                offset: candidate.time as u16,
                event: AudioEvent::NoteOn {
                    note: candidate.pitch,
                    velocity: candidate.velocity,
                    channel: candidate.channel,
                },
            },
        );

        // La fin de l'ancienne note chevauche l'attaque (léger overlap). Dans le
        // pas courant elle part tout de suite ; au-delà elle est reportée (même
        // barre, ou barre suivante s'il franchit la mesure).
        let off_abs = start + candidate.time + LEGATO_OVERLAP_MILLI_STEPS;
        let off = |time: i64| PreparedEvent {
            time,
            id: old_id,
            channel: candidate.channel,
            pitch: old_pitch as u8,
            velocity: 0,
            on: false,
            tied: false,
            legato: false,
            force_off: true,
        };
        if off_abs < end {
            push_capped(
                &mut self.events,
                StepEvent {
                    offset: (off_abs - start) as u16,
                    event: AudioEvent::NoteOff {
                        note: old_pitch as u8,
                        channel: candidate.channel,
                    },
                },
            );
        } else if off_abs < length {
            push_capped(&mut self.pending, Spill { bar: self.position.bar, event: off(off_abs) });
        } else {
            push_capped(
                &mut self.pending,
                Spill { bar: self.position.bar + 1, event: off(off_abs - length) },
            );
        }
        true
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

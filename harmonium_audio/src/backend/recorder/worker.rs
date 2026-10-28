//! Travailleur d'enregistrement : encode le WAV et le MIDI hors du callback.
//!
//! Il consomme la file SPSC dans l'ordre, possède toute la croissance mémoire
//! (tampon WAV, piste MIDI, notes actives, SMF) et publie les fichiers finis
//! dans `finished_recordings`. Les positions MIDI arrivent en frames ; il les
//! convertit en ticks **cumulés** avec la cadence courante, et arrondit la
//! position absolue, jamais un delta isolé.
//!
//! Deux façons de le faire tourner, avec le même encodeur : [`spawn`] lui donne un
//! thread (natif) ; [`RecorderWorker::pump`] le fait avancer depuis l'appelant, sans
//! thread (wasm, où `std::thread::spawn` n'existe pas).

use std::{
    io::{Cursor, Seek, SeekFrom, Write},
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use harmonium_core::{events::RecordFormat, log};
use hound::{SampleFormat, WavSpec, WavWriter};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

use super::protocol::{MidiEvent, PcmBlock, RecordMsg};
use crate::backend::recorder::FinishedRecordings;

/// Cadence initiale (ticks/frame) avant tout `MidiClock` : 120 BPM sans swing à 44,1 kHz
/// avec le pas par défaut de 11025 frames.
pub(super) const DEFAULT_TICKS_PER_FRAME: f64 = 120.0 / 11025.0;

/// Sommeil du thread quand la file est vide.
const IDLE_SLEEP: Duration = Duration::from_millis(1);

pub(super) struct WorkerPorts {
    pub rx: rtrb::Consumer<RecordMsg>,
    pub free_tx: rtrb::Producer<PcmBlock>,
    pub finished: FinishedRecordings,
    pub sample_rate: u32,
    /// Nombre de messages consommés, lu par `wait_idle`.
    pub progress: Arc<AtomicU64>,
    /// Passe à faux quand le thread s'arrête (y compris sur panique).
    pub alive: Arc<AtomicBool>,
}

struct AliveGuard(Arc<AtomicBool>);

impl Drop for AliveGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

pub(super) fn spawn(ports: WorkerPorts) -> std::io::Result<std::thread::JoinHandle<()>> {
    std::thread::Builder::new().name("harmonium-recorder".into()).spawn(move || {
        let _alive = AliveGuard(ports.alive.clone());
        RecorderWorker::new(ports).run();
    })
}

/// Tampon WAV partagé entre l'écrivain `hound` et la session (jamais en concurrence : le verrou
/// ne sert qu'à garder le travailleur `Send`, il est libre à chaque accès).
#[derive(Clone)]
struct SharedBuf(Arc<Mutex<Cursor<Vec<u8>>>>);

impl Write for SharedBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).flush()
    }
}

impl Seek for SharedBuf {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).seek(pos)
    }
}

struct WavSession {
    writer: WavWriter<SharedBuf>,
    buf: SharedBuf,
    written: u64,
    /// Échantillons remplacés par du silence (blocs perdus au débordement).
    padded: u64,
}

struct MidiSession {
    track: Vec<TrackEvent<'static>>,
    /// Dernière position absolue émise, en ticks.
    last_tick: u64,
    /// Point d'ancrage de la cadence courante : frame et ticks (fractionnaires).
    clock_frame: u64,
    clock_ticks: f64,
    ticks_per_frame: f64,
    /// Notes en train de sonner `(canal, hauteur)` : chaque `NoteOn` reçoit son
    /// `NoteOff`, même quand l'export coupe une résonance.
    active: Vec<(u8, u8)>,
}

impl MidiSession {
    fn new(frame: u64) -> Self {
        Self {
            track: Vec::with_capacity(4096),
            last_tick: 0,
            clock_frame: frame,
            clock_ticks: 0.0,
            ticks_per_frame: DEFAULT_TICKS_PER_FRAME,
            active: Vec::new(),
        }
    }

    /// Ticks fractionnaires à `frame` avec la cadence courante.
    fn ticks_at(&self, frame: u64) -> f64 {
        self.clock_ticks + frame.saturating_sub(self.clock_frame) as f64 * self.ticks_per_frame
    }

    /// Ajoute un événement à `frame` : position absolue arrondie, delta déduit
    /// des positions absolues (le reste fractionnaire ne se perd pas).
    fn push(&mut self, frame: u64, kind: TrackEventKind<'static>) {
        let tick = (self.ticks_at(frame).round() as u64).max(self.last_tick);
        let delta = (tick - self.last_tick) as u32;
        self.last_tick = tick;
        self.track.push(TrackEvent { delta: delta.into(), kind });
    }

    fn push_midi(&mut self, frame: u64, channel: u8, message: MidiMessage) {
        self.push(frame, TrackEventKind::Midi { channel: channel.into(), message });
    }

    fn apply(&mut self, frame: u64, event: MidiEvent) {
        match event {
            MidiEvent::NoteOn { channel, key, vel } => {
                self.push_midi(
                    frame,
                    channel,
                    MidiMessage::NoteOn { key: key.into(), vel: vel.into() },
                );
                if !self.active.contains(&(channel, key)) {
                    self.active.push((channel, key));
                }
            }
            MidiEvent::NoteOff { channel, key } => {
                self.push_midi(
                    frame,
                    channel,
                    MidiMessage::NoteOff { key: key.into(), vel: 0.into() },
                );
                if let Some(pos) = self.active.iter().position(|&(ch, p)| ch == channel && p == key)
                {
                    self.active.swap_remove(pos);
                }
            }
            MidiEvent::ProgramChange { channel, program } => {
                self.push_midi(
                    frame,
                    channel,
                    MidiMessage::ProgramChange { program: program.into() },
                );
            }
            MidiEvent::Controller { channel, controller, value } => self.push_midi(
                frame,
                channel,
                MidiMessage::Controller { controller: controller.into(), value: value.into() },
            ),
            MidiEvent::AllNotesOff { channel } => {
                let mut i = 0;
                while i < self.active.len() {
                    if self.active[i].0 == channel {
                        let (_, key) = self.active.swap_remove(i);
                        self.push_midi(
                            frame,
                            channel,
                            MidiMessage::NoteOff { key: key.into(), vel: 0.into() },
                        );
                    } else {
                        i += 1;
                    }
                }
            }
            MidiEvent::Tempo { micros_per_beat } => {
                self.push(frame, TrackEventKind::Meta(MetaMessage::Tempo(micros_per_beat.into())));
            }
            MidiEvent::TimeSignature { numerator, denominator_pow } => self.push(
                frame,
                TrackEventKind::Meta(MetaMessage::TimeSignature(numerator, denominator_pow, 24, 8)),
            ),
            MidiEvent::Clock { ticks_per_frame } => {
                // Ancre au point exact (non arrondi) : la cadence change, la position non.
                self.clock_ticks = self.ticks_at(frame);
                self.clock_frame = frame;
                self.ticks_per_frame = ticks_per_frame;
            }
        }
    }

    /// Ferme les notes qui sonnent encore, ajoute `EndOfTrack` et sérialise.
    fn finish(mut self, frame: u64) -> Option<Vec<u8>> {
        for (channel, key) in std::mem::take(&mut self.active) {
            self.push_midi(frame, channel, MidiMessage::NoteOff { key: key.into(), vel: 0.into() });
        }
        self.track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });
        let mut smf = Smf::new(Header::new(Format::SingleTrack, Timing::Metrical(480.into())));
        smf.tracks.push(self.track);
        let mut bytes = Vec::new();
        smf.write(&mut bytes).ok()?;
        Some(bytes)
    }
}

/// Encodeur d'enregistrement : consomme les messages du callback. Natif, un thread le fait
/// tourner ; sans thread (wasm), l'hôte l'appelle par [`pump`](Self::pump).
pub struct RecorderWorker {
    ports: WorkerPorts,
    processed: u64,
    wav: Option<WavSession>,
    midi: Option<MidiSession>,
    /// Dernière frame vue : borne de fermeture quand la file se ferme sans `MidiStop`.
    last_frame: u64,
}

impl RecorderWorker {
    pub(super) fn new(ports: WorkerPorts) -> Self {
        Self { ports, processed: 0, wav: None, midi: None, last_frame: 0 }
    }

    /// Consomme tous les messages en attente et publie les fichiers terminés dans
    /// `finished_recordings`. Si le callback a disparu, ferme la prise en cours. Hors callback
    /// uniquement : l'encodage alloue.
    pub fn pump(&mut self) {
        self.drain();
        if self.closed() {
            self.close();
        }
    }

    /// Vide la file ; rend vrai si au moins un message a été traité.
    fn drain(&mut self) -> bool {
        let mut busy = false;
        while let Ok(msg) = self.ports.rx.pop() {
            busy = true;
            self.handle(msg);
            self.processed += 1;
            self.ports.progress.store(self.processed, Ordering::Release);
        }
        busy
    }

    /// Le callback a disparu et la file est vide.
    fn closed(&self) -> bool {
        self.ports.rx.is_abandoned() && self.ports.rx.is_empty()
    }

    /// Le callback a disparu : ne pas perdre une prise en cours.
    fn close(&mut self) {
        if self.wav.is_some() {
            self.stop_wav(None);
        }
        if self.midi.is_some() {
            self.stop_midi(self.last_frame);
        }
    }

    /// Boucle du thread d'enregistrement.
    fn run(&mut self) {
        loop {
            if !self.drain() {
                if self.closed() {
                    break;
                }
                std::thread::sleep(IDLE_SLEEP);
            }
        }
        self.close();
    }

    fn handle(&mut self, msg: RecordMsg) {
        match msg {
            RecordMsg::WavStart => self.start_wav(),
            RecordMsg::WavStop { total_samples } => self.stop_wav(Some(total_samples)),
            RecordMsg::Pcm(block) => self.write_pcm(block),
            RecordMsg::MidiStart { frame } => {
                self.last_frame = frame;
                self.midi = Some(MidiSession::new(frame));
            }
            RecordMsg::Midi { frame, event } => {
                self.last_frame = frame;
                if let Some(session) = &mut self.midi {
                    session.apply(frame, event);
                }
            }
            RecordMsg::MidiStop { frame } => self.stop_midi(frame),
        }
    }

    fn start_wav(&mut self) {
        let spec = WavSpec {
            channels: 2,
            sample_rate: self.ports.sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };
        let buf = SharedBuf(Arc::new(Mutex::new(Cursor::new(Vec::with_capacity(1024 * 1024)))));
        self.wav = WavWriter::new(buf.clone(), spec).ok().map(|writer| WavSession {
            writer,
            buf,
            written: 0,
            padded: 0,
        });
    }

    fn write_pcm(&mut self, mut block: PcmBlock) {
        if let Some(session) = &mut self.wav {
            // Blocs refusés au débordement : le silence garde la durée exacte.
            while session.written < block.start_sample {
                session.writer.write_sample(0.0f32).ok();
                session.written += 1;
                session.padded += 1;
            }
            for sample in &block.data[..block.len] {
                session.writer.write_sample(*sample).ok();
            }
            session.written += block.len as u64;
        }
        block.len = 0;
        // L'anneau a la capacité du lot : le retour ne peut échouer.
        let _ = self.ports.free_tx.push(block);
    }

    fn stop_wav(&mut self, total_samples: Option<u64>) {
        let Some(mut session) = self.wav.take() else { return };
        if let Some(total) = total_samples {
            while session.written < total {
                session.writer.write_sample(0.0f32).ok();
                session.written += 1;
                session.padded += 1;
            }
        }
        if session.padded > 0 {
            log::warn(&format!(
                "enregistrement WAV : {} échantillons perdus au débordement, remplacés par du silence",
                session.padded
            ));
        }
        let WavSession { writer, buf, .. } = session;
        if writer.finalize().is_err() {
            return;
        }
        if let Ok(cell) = Arc::try_unwrap(buf.0) {
            let cursor = cell.into_inner().unwrap_or_else(PoisonError::into_inner);
            self.publish(RecordFormat::Wav, cursor.into_inner());
        }
    }

    fn stop_midi(&mut self, frame: u64) {
        if let Some(session) = self.midi.take()
            && let Some(bytes) = session.finish(frame)
        {
            self.publish(RecordFormat::Midi, bytes);
        }
    }

    fn publish(&self, format: RecordFormat, bytes: Vec<u8>) {
        let mut queue = match self.ports.finished.lock() {
            Ok(queue) => queue,
            Err(poisoned) => poisoned.into_inner(),
        };
        queue.push((format, bytes));
    }
}

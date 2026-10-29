//! Enregistreur WAV/MIDI : le callback ne fait que copier et horodater.
//!
//! # Contrat temps réel
//!
//! Le callback (`handle_event`, `process_buffer`) n'alloue pas, ne libère
//! rien et ne prend aucun verrou. Il écrit dans une file SPSC `rtrb` bornée,
//! de taille fixe :
//!
//! - des blocs PCM préalloués (recyclés par une seconde file) ;
//! - des événements MIDI `Copy`, horodatés en **frames** ;
//! - les commandes de début/fin (places réservées : une fin entre toujours).
//!
//! Un travailleur d'enregistrement encode le WAV et le MIDI, possède toute la
//! croissance mémoire et publie le résultat dans `finished_recordings`
//! (`worker`). Natif, c'est un thread créé à la construction ; sans thread
//! (wasm), [`RecorderBackend::pumped`] rend le travailleur, que l'hôte fait
//! avancer depuis son propre fil ([`RecorderWorker::pump`]) : même encodeur.
//!
//! # Débordement
//!
//! [`OverflowPolicy::DropAndCount`] (temps réel) refuse la donnée qui ne
//! tient pas, la compte dans [`RecorderStats`] et le WAV la remplace par du
//! silence : la durée reste exacte, la perte est publiée. [`OverflowPolicy::Block`]
//! (rendu hors ligne, sans échéance) attend le thread d'enregistrement : rien
//! n'est perdu et la fin d'une prise rend la main une fois le fichier publié.
//!
//! # MIDI exact
//!
//! Le moteur annonce sa cadence (`AudioEvent::MidiClock`, ticks par frame) à
//! chaque pulse ; le thread d'enregistrement convertit la frame de chaque
//! événement en ticks **cumulés** et arrondit la position absolue : aucun
//! reste fractionnaire n'est jeté, le swing et les rampes de tempo restent
//! sur la grille écrite.

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use harmonium_core::{
    events::{AudioEvent, AudioRecordFormat, RecordFormat},
    log,
    mix::PreparedMix,
    params::{PlaybackParams, TimeSignature},
    style_profile::ProfileInstruments,
};

use crate::backend::AudioRenderer;

mod protocol;
mod worker;

pub use protocol::RecorderStats;
use protocol::{
    CONTROL_RESERVE, MidiEvent, PCM_BLOCK_SAMPLES, PCM_POOL_BLOCKS, PcmBlock, QUEUE_CAPACITY,
    RecordMsg,
};
pub use worker::RecorderWorker;
use worker::{DEFAULT_TICKS_PER_FRAME, WorkerPorts};

pub type FinishedRecordings = Arc<Mutex<Vec<(RecordFormat, Vec<u8>)>>>;

/// Que faire quand la file d'enregistrement est pleine ?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverflowPolicy {
    /// Temps réel : refuser la donnée, la compter, ne jamais attendre.
    DropAndCount,
    /// Hors ligne : attendre le thread d'enregistrement ; rien n'est perdu.
    Block,
}

/// Map a logical channel to its MIDI channel: Heel (3) and Toe (4) share
/// the GM percussion channel 9.
fn map_midi_channel(channel: u8) -> u8 {
    if channel == 3 || channel == 4 { crate::voice_manager::GM_DRUM_CHANNEL } else { channel }
}

fn micros_per_beat(bpm: f32) -> u32 {
    let bpm = if bpm > 0.0 { bpm } else { 120.0 };
    (60_000_000.0 / bpm) as u32
}

fn time_signature_event(signature: TimeSignature) -> MidiEvent {
    MidiEvent::TimeSignature {
        numerator: signature.numerator as u8,
        denominator_pow: signature.denominator.trailing_zeros() as u8,
    }
}

pub struct RecorderBackend {
    inner: Box<dyn AudioRenderer>,
    policy: OverflowPolicy,

    // Files vers le thread d'enregistrement.
    tx: rtrb::Producer<RecordMsg>,
    free_rx: rtrb::Consumer<PcmBlock>,
    stats: Arc<RecorderStats>,
    /// Messages consommés par le thread ; `sent` en compte les envois réussis.
    progress: Arc<AtomicU64>,
    sent: u64,
    alive: Arc<AtomicBool>,

    /// Frames rendues depuis la création : l'horodatage des événements MIDI.
    frame: u64,

    // WAV : bloc en cours de remplissage et échantillons vus depuis le début de la prise.
    wav_active: bool,
    wav_samples: u64,
    pcm: Option<PcmBlock>,

    // MIDI
    midi_active: bool,
    ticks_per_frame: f64,
    bpm: f32,
    time_signature: TimeSignature,
    instruments: ProfileInstruments,
    channel_programs: [u8; 16],
    mix: PreparedMix,
}

impl RecorderBackend {
    /// Enregistreur temps réel ([`OverflowPolicy::DropAndCount`]).
    pub fn new(
        inner: Box<dyn AudioRenderer>,
        finished_recordings: FinishedRecordings,
        sample_rate: u32,
    ) -> Self {
        Self::with_policy(inner, finished_recordings, sample_rate, OverflowPolicy::DropAndCount)
    }

    /// Enregistreur avec une politique de débordement explicite. Crée le
    /// thread d'enregistrement et préalloue les blocs PCM : à construire hors du callback.
    pub fn with_policy(
        inner: Box<dyn AudioRenderer>,
        finished_recordings: FinishedRecordings,
        sample_rate: u32,
        policy: OverflowPolicy,
    ) -> Self {
        let (backend, ports) = Self::assemble(inner, finished_recordings, sample_rate, policy);
        if let Err(error) = worker::spawn(ports) {
            log::error(&format!("thread d'enregistrement impossible : {error}"));
            backend.alive.store(false, Ordering::Release);
        }
        backend
    }

    /// Enregistreur sans thread, [`OverflowPolicy::DropAndCount`] : rend le
    /// travailleur, que l'appelant fait avancer par [`RecorderWorker::pump`]
    /// (l'hôte wasm, à chaque tour de sa boucle). Le callback ne bloque jamais ; ce
    /// que le travailleur n'a pas vidé à temps est perdu et compté
    /// ([`RecorderStats`]). [`wait_idle`](Self::wait_idle) n'y a pas de sens.
    pub fn pumped(
        inner: Box<dyn AudioRenderer>,
        finished_recordings: FinishedRecordings,
        sample_rate: u32,
    ) -> (Self, RecorderWorker) {
        let (backend, ports) =
            Self::assemble(inner, finished_recordings, sample_rate, OverflowPolicy::DropAndCount);
        (backend, RecorderWorker::new(ports))
    }

    /// Construit l'enregistreur et les ports du thread sans le lancer.
    fn assemble(
        inner: Box<dyn AudioRenderer>,
        finished: FinishedRecordings,
        sample_rate: u32,
        policy: OverflowPolicy,
    ) -> (Self, WorkerPorts) {
        let (tx, rx) = rtrb::RingBuffer::<RecordMsg>::new(QUEUE_CAPACITY);
        let (mut free_tx, free_rx) = rtrb::RingBuffer::<PcmBlock>::new(PCM_POOL_BLOCKS);
        for _ in 0..PCM_POOL_BLOCKS {
            let _ = free_tx.push(PcmBlock::new());
        }
        let progress = Arc::new(AtomicU64::new(0));
        let alive = Arc::new(AtomicBool::new(true));
        let defaults = PlaybackParams::default();
        let mut channel_programs = [0u8; 16];
        for (channel, program) in
            crate::voice_manager::gm_programs_for_profile(&defaults.instruments)
        {
            channel_programs[channel as usize] = program;
        }
        let backend = Self {
            inner,
            policy,
            tx,
            free_rx,
            stats: Arc::new(RecorderStats::default()),
            progress: progress.clone(),
            sent: 0,
            alive: alive.clone(),
            frame: 0,
            wav_active: false,
            wav_samples: 0,
            pcm: None,
            midi_active: false,
            ticks_per_frame: DEFAULT_TICKS_PER_FRAME,
            bpm: defaults.bpm,
            time_signature: defaults.time_signature,
            instruments: defaults.instruments,
            channel_programs,
            mix: defaults.mix,
        };
        let ports = WorkerPorts { rx, free_tx, finished, sample_rate, progress, alive };
        (backend, ports)
    }

    /// Compteurs de perte, lisibles depuis n'importe quel thread.
    #[must_use]
    pub fn stats(&self) -> Arc<RecorderStats> {
        self.stats.clone()
    }

    /// Bloque jusqu'à ce que le thread d'enregistrement ait consommé tout ce
    /// que le callback a envoyé. Hors callback uniquement.
    pub fn wait_idle(&self) {
        while self.progress.load(Ordering::Acquire) < self.sent
            && self.alive.load(Ordering::Acquire)
        {
            std::thread::sleep(Duration::from_micros(100));
        }
    }

    /// Access the inner backend for downcasting
    pub fn inner_mut(&mut self) -> &mut dyn AudioRenderer {
        self.inner.as_mut()
    }

    /// Envoie `msg` si la file garde plus de `reserve` places libres. En
    /// `Block`, attend le thread d'enregistrement tant qu'il vit ; sinon rend
    /// le message à l'appelant, qui le compte.
    fn push(&mut self, mut msg: RecordMsg, reserve: usize) -> Result<(), RecordMsg> {
        loop {
            if self.tx.slots() > reserve {
                match self.tx.push(msg) {
                    Ok(()) => {
                        self.sent += 1;
                        return Ok(());
                    }
                    Err(rtrb::PushError::Full(back)) => msg = back,
                }
            }
            if self.policy == OverflowPolicy::DropAndCount || !self.alive.load(Ordering::Acquire) {
                return Err(msg);
            }
            std::thread::yield_now();
        }
    }

    /// Commande de début/fin : jamais refusée tant que la réserve tient.
    fn control(&mut self, msg: RecordMsg) -> bool {
        let sent = self.push(msg, 0).is_ok();
        if !sent {
            self.stats.add_command();
        }
        sent
    }

    fn midi(&mut self, event: MidiEvent) {
        if !self.midi_active {
            return;
        }
        let msg = RecordMsg::Midi { frame: self.frame, event };
        if self.push(msg, CONTROL_RESERVE).is_err() {
            self.stats.add_midi();
        }
    }

    fn acquire_block(&mut self) -> Option<PcmBlock> {
        loop {
            if let Ok(block) = self.free_rx.pop() {
                return Some(block);
            }
            if self.policy == OverflowPolicy::DropAndCount || !self.alive.load(Ordering::Acquire) {
                return None;
            }
            std::thread::yield_now();
        }
    }

    /// Envoie le bloc en cours ; refusé, il est recyclé et ses échantillons comptés.
    fn flush_pcm(&mut self) {
        let Some(block) = self.pcm.take() else { return };
        if block.len == 0 {
            self.pcm = Some(block);
            return;
        }
        let len = block.len;
        if let Err(RecordMsg::Pcm(mut block)) = self.push(RecordMsg::Pcm(block), CONTROL_RESERVE) {
            self.stats.add_pcm(len);
            block.len = 0;
            self.pcm = Some(block);
        }
    }

    fn capture_pcm(&mut self, mut src: &[f32]) {
        while !src.is_empty() {
            if self.pcm.is_none() {
                self.pcm = self.acquire_block();
            }
            let Some(block) = self.pcm.as_mut() else {
                // Aucun bloc libre : perte comptée, le WAV comblera par du silence.
                self.stats.add_pcm(src.len());
                self.wav_samples += src.len() as u64;
                return;
            };
            if block.len == 0 {
                block.start_sample = self.wav_samples;
            }
            let n = (PCM_BLOCK_SAMPLES - block.len).min(src.len());
            block.data[block.len..block.len + n].copy_from_slice(&src[..n]);
            block.len += n;
            self.wav_samples += n as u64;
            src = &src[n..];
            if block.len == PCM_BLOCK_SAMPLES {
                self.flush_pcm();
            }
        }
    }

    fn start_wav(&mut self) {
        if self.wav_active {
            return;
        }
        self.wav_samples = 0;
        self.wav_active = self.control(RecordMsg::WavStart);
    }

    fn stop_wav(&mut self) {
        if !self.wav_active {
            return;
        }
        self.wav_active = false;
        self.flush_pcm();
        let total_samples = self.wav_samples;
        self.control(RecordMsg::WavStop { total_samples });
        self.settle();
    }

    fn start_midi(&mut self) {
        if self.midi_active {
            return;
        }
        self.midi_active = self.control(RecordMsg::MidiStart { frame: self.frame });
        // Cadence, tempo réel et signature d'abord, puis programmes GM et mix.
        self.midi(MidiEvent::Clock { ticks_per_frame: self.ticks_per_frame });
        self.midi(MidiEvent::Tempo { micros_per_beat: micros_per_beat(self.bpm) });
        self.midi(time_signature_event(self.time_signature));
        for (channel, _) in crate::voice_manager::gm_programs_for_profile(&self.instruments) {
            self.midi(MidiEvent::ProgramChange {
                channel,
                program: self.channel_programs[channel as usize],
            });
        }
        self.record_mix();
    }

    fn stop_midi(&mut self) {
        if !self.midi_active {
            return;
        }
        self.midi_active = false;
        self.control(RecordMsg::MidiStop { frame: self.frame });
        self.settle();
    }

    /// Hors ligne, rend la main une fois le fichier publié.
    fn settle(&self) {
        if self.policy == OverflowPolicy::Block {
            self.wait_idle();
        }
    }

    fn record_mix(&mut self) {
        // Canonical logical order, like SynthBackend : Toe wins on shared GM ch9.
        for (channel, track) in self.mix.tracks.into_iter().enumerate() {
            for (controller, value) in track.controllers() {
                self.midi(MidiEvent::Controller {
                    channel: map_midi_channel(channel as u8),
                    controller,
                    value,
                });
            }
        }
    }

    fn update_timing(&mut self, bpm: f32, signature: TimeSignature) {
        if self.midi_active {
            if (bpm - self.bpm).abs() > 0.01 {
                self.midi(MidiEvent::Tempo { micros_per_beat: micros_per_beat(bpm) });
            }
            if signature != self.time_signature {
                self.midi(time_signature_event(signature));
            }
        }
        self.bpm = bpm;
        self.time_signature = signature;
    }
}

impl AudioRenderer for RecorderBackend {
    fn handle_event(&mut self, event: AudioEvent) {
        match &event {
            AudioEvent::StartRecording { format } => match format {
                AudioRecordFormat::Wav => self.start_wav(),
                AudioRecordFormat::Midi => self.start_midi(),
            },
            AudioEvent::StopRecording { format } => match format {
                AudioRecordFormat::Wav => self.stop_wav(),
                AudioRecordFormat::Midi => self.stop_midi(),
            },
            AudioEvent::SetMix { mix } => {
                self.mix = *mix;
                self.record_mix();
            }
            AudioEvent::MidiClock { ticks_per_frame } => {
                self.ticks_per_frame = *ticks_per_frame;
                self.midi(MidiEvent::Clock { ticks_per_frame: *ticks_per_frame });
            }
            AudioEvent::UpdateMusicalParams { params } => {
                self.update_timing(params.bpm, params.time_signature);
                for (channel, program) in
                    crate::voice_manager::gm_programs_for_profile(&params.instruments)
                {
                    self.channel_programs[channel as usize] = program;
                }
                self.instruments = params.instruments;
            }
            AudioEvent::MeasureTiming { bpm, time_signature, .. } => {
                self.update_timing(*bpm, *time_signature);
            }
            AudioEvent::ProgramChange { channel, program } => {
                if (*channel as usize) < 16 {
                    self.channel_programs[*channel as usize] = *program;
                }
                self.midi(MidiEvent::ProgramChange {
                    channel: map_midi_channel(*channel),
                    program: *program,
                });
            }
            AudioEvent::NoteOn { note, velocity, channel } => {
                self.midi(MidiEvent::NoteOn {
                    channel: map_midi_channel(*channel),
                    key: *note,
                    vel: *velocity,
                });
            }
            AudioEvent::NoteOff { note, channel } => {
                self.midi(MidiEvent::NoteOff { channel: map_midi_channel(*channel), key: *note });
            }
            AudioEvent::AllNotesOff { channel } => {
                self.midi(MidiEvent::AllNotesOff { channel: map_midi_channel(*channel) });
            }
            _ => {}
        }
        self.inner.handle_event(event);
    }

    fn process_buffer(&mut self, output: &mut [f32], channels: usize) {
        self.inner.process_buffer(output, channels);

        if self.wav_active {
            self.capture_pcm(output);
        }
        if let Some(frames) = output.len().checked_div(channels) {
            self.frame += frames as u64;
        }
    }

    #[cfg(feature = "odin")]
    fn apply_odin(&mut self, update: &mut crate::odin_bank::OdinUpdate) {
        self.inner.apply_odin(update);
    }

    #[cfg(feature = "odin")]
    fn morph_odin(&mut self, patches: &[harmonium_core::odin::OdinPatches; 2]) {
        self.inner.morph_odin(patches);
    }

    #[cfg(feature = "odin")]
    fn finish_odin(&mut self, update: &mut crate::odin_bank::OdinUpdate) -> bool {
        self.inner.finish_odin(update)
    }

    #[cfg(feature = "odin")]
    fn odin_diagnostics(&self) -> Option<crate::odin_voices::OdinDiagnostics> {
        self.inner.odin_diagnostics()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use harmonium_core::params::TimeSignature;
    use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};

    use super::*;

    struct NullRenderer;

    impl AudioRenderer for NullRenderer {
        fn handle_event(&mut self, _event: AudioEvent) {}
        fn process_buffer(&mut self, _output: &mut [f32], _channels: usize) {}
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    fn params_at(bpm: f32, numerator: usize, denominator: usize) -> PlaybackParams {
        PlaybackParams {
            bpm,
            time_signature: TimeSignature { numerator, denominator },
            ..PlaybackParams::default()
        }
    }

    fn lossless(finished: &FinishedRecordings) -> RecorderBackend {
        RecorderBackend::with_policy(
            Box::new(NullRenderer),
            finished.clone(),
            44100,
            OverflowPolicy::Block,
        )
    }

    /// `steps` sixteenth-steps elapse at 1000 frames per step (0.12 tick/frame).
    fn advance(rec: &mut RecorderBackend, steps: f64) {
        rec.handle_event(AudioEvent::MidiClock { ticks_per_frame: 120.0 / 1000.0 });
        let frames = (steps * 1000.0) as usize;
        rec.process_buffer(&mut vec![0.0f32; frames * 2], 2);
    }

    fn finished_midi(finished: &FinishedRecordings) -> Result<Vec<u8>, String> {
        let queue = finished.lock().map_err(|e| e.to_string())?;
        queue
            .iter()
            .find(|(format, _)| *format == RecordFormat::Midi)
            .map(|(_, bytes)| bytes.clone())
            .ok_or_else(|| "un enregistrement MIDI".to_string())
    }

    /// Absolute tick of every `NoteOn` of channel 0, in order.
    fn note_on_ticks(bytes: &[u8]) -> Result<Vec<u32>, String> {
        let smf = Smf::parse(bytes).map_err(|e| e.to_string())?;
        let mut tick = 0u32;
        let mut out = Vec::new();
        for event in &smf.tracks[0] {
            tick += event.delta.as_int();
            if let TrackEventKind::Midi { message: MidiMessage::NoteOn { vel, .. }, .. } =
                event.kind
                && vel.as_int() > 0
            {
                out.push(tick);
            }
        }
        Ok(out)
    }

    /// The recorded MIDI mirrors the played stream: real tempo and time
    /// signature metas, one `NoteOff` per `NoteOn` (same channel and pitch),
    /// `AllNotesOff` translated, truncated rings closed at stop.
    #[test]
    fn midi_mirrors_tempo_signature_and_note_lifetimes() -> Result<(), String> {
        let finished: FinishedRecordings = Arc::new(Mutex::new(Vec::new()));
        let mut rec = lossless(&finished);

        rec.handle_event(AudioEvent::UpdateMusicalParams { params: params_at(230.0, 2, 2) });
        rec.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Midi });
        rec.handle_event(AudioEvent::NoteOn { note: 60, velocity: 90, channel: 0 });
        advance(&mut rec, 2.0);
        rec.handle_event(AudioEvent::NoteOff { note: 60, channel: 0 });
        rec.handle_event(AudioEvent::NoteOn { note: 62, velocity: 90, channel: 0 });
        advance(&mut rec, 1.0);
        // Legato cut: the synth drops 62 here, so the file must too.
        rec.handle_event(AudioEvent::AllNotesOff { channel: 0 });
        // Truncated ring: no NoteOff before stop.
        rec.handle_event(AudioEvent::NoteOn { note: 40, velocity: 90, channel: 3 });
        rec.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Midi });

        let queue = finished.lock().map_err(|e| e.to_string())?;
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].0, RecordFormat::Midi);
        let smf = Smf::parse(&queue[0].1).map_err(|e| e.to_string())?;
        assert_eq!(smf.tracks.len(), 1);
        let track = &smf.tracks[0];

        let tempos: Vec<u32> = track
            .iter()
            .filter_map(|e| match e.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(t)) => Some(t.as_int()),
                _ => None,
            })
            .collect();
        assert_eq!(tempos, vec![(60_000_000.0 / 230.0) as u32], "tempo meta: {tempos:?}");
        let sigs: Vec<(u8, u8)> = track
            .iter()
            .filter_map(|e| match e.kind {
                TrackEventKind::Meta(MetaMessage::TimeSignature(n, d, _, _)) => Some((n, d)),
                _ => None,
            })
            .collect();
        assert_eq!(sigs, vec![(2, 1)], "time signature 2/2: {sigs:?}");

        let mut ons: Vec<(u8, u8)> = Vec::new();
        let mut offs: Vec<(u8, u8)> = Vec::new();
        for e in track {
            match e.kind {
                TrackEventKind::Midi { channel, message: MidiMessage::NoteOn { key, vel } }
                    if vel.as_int() > 0 =>
                {
                    ons.push((channel.as_int(), key.as_int()))
                }
                TrackEventKind::Midi { channel, message: MidiMessage::NoteOff { key, .. } } => {
                    offs.push((channel.as_int(), key.as_int()))
                }
                _ => {}
            }
        }
        ons.sort_unstable();
        offs.sort_unstable();
        // Ch 3 (heel) is mapped to GM drums (9); the truncated 40 still closes.
        assert_eq!(ons, vec![(0, 60), (0, 62), (9, 40)], "ons: {ons:?}");
        assert_eq!(offs, vec![(0, 60), (0, 62), (9, 40)], "offs: {offs:?}");
        Ok(())
    }

    /// A tempo or signature change mid-recording lands in the file.
    #[test]
    fn midi_follows_mid_recording_tempo_change() -> Result<(), String> {
        let finished: FinishedRecordings = Arc::new(Mutex::new(Vec::new()));
        let mut rec = lossless(&finished);

        rec.handle_event(AudioEvent::UpdateMusicalParams { params: params_at(200.0, 2, 2) });
        rec.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Midi });
        advance(&mut rec, 4.0);
        rec.handle_event(AudioEvent::UpdateMusicalParams { params: params_at(240.0, 2, 2) });
        rec.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Midi });

        let bytes = finished_midi(&finished)?;
        let smf = Smf::parse(&bytes).map_err(|e| e.to_string())?;
        let tempos: Vec<u32> = smf.tracks[0]
            .iter()
            .filter_map(|e| match e.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(t)) => Some(t.as_int()),
                _ => None,
            })
            .collect();
        assert_eq!(
            tempos,
            vec![(60_000_000.0 / 200.0) as u32, (60_000_000.0 / 240.0) as u32],
            "tempos: {tempos:?}"
        );
        Ok(())
    }

    /// A delta rounded on its own loses its remainder at every event; the
    /// absolute position must not: 0.7 tick per event stays within half a tick
    /// of `0.7 × n` after thousands of events.
    #[test]
    fn midi_ticks_are_cumulated_without_drift() -> Result<(), String> {
        let finished: FinishedRecordings = Arc::new(Mutex::new(Vec::new()));
        let mut rec = lossless(&finished);
        rec.handle_event(AudioEvent::MidiClock { ticks_per_frame: 0.1 });
        rec.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Midi });
        for _ in 0..3000 {
            rec.handle_event(AudioEvent::NoteOn { note: 60, velocity: 90, channel: 0 });
            rec.handle_event(AudioEvent::NoteOff { note: 60, channel: 0 });
            rec.process_buffer(&mut [0.0; 14], 2); // 7 frames = 0.7 tick
        }
        rec.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Midi });

        let ticks = note_on_ticks(&finished_midi(&finished)?)?;
        assert_eq!(ticks.len(), 3000);
        for (n, tick) in ticks.iter().enumerate() {
            let exact = 0.7 * n as f64;
            assert!(
                (f64::from(*tick) - exact).abs() <= 0.5 + 1e-9,
                "attaque {n}: tick {tick}, exact {exact}"
            );
        }
        Ok(())
    }

    /// A cadence change keeps the position continuous: swing/ramp pulses are
    /// converted with their own ticks-per-frame from the exact change frame.
    #[test]
    fn midi_clock_change_keeps_positions_exact() -> Result<(), String> {
        let finished: FinishedRecordings = Arc::new(Mutex::new(Vec::new()));
        let mut rec = lossless(&finished);
        rec.handle_event(AudioEvent::MidiClock { ticks_per_frame: 0.2 });
        rec.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Midi });
        rec.handle_event(AudioEvent::NoteOn { note: 60, velocity: 90, channel: 0 });
        rec.process_buffer(&mut [0.0; 1000], 2); // 500 frames × 0.2 = 100 ticks
        rec.handle_event(AudioEvent::MidiClock { ticks_per_frame: 0.05 });
        rec.handle_event(AudioEvent::NoteOn { note: 62, velocity: 90, channel: 0 });
        rec.process_buffer(&mut [0.0; 2002], 2); // 1001 frames × 0.05 = 50.05 ticks
        rec.handle_event(AudioEvent::NoteOn { note: 64, velocity: 90, channel: 0 });
        rec.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Midi });

        let ticks = note_on_ticks(&finished_midi(&finished)?)?;
        assert_eq!(ticks, vec![0, 100, 150]);
        Ok(())
    }

    fn expected_wav(samples: &[f32]) -> Result<Vec<u8>, String> {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            let mut writer = hound::WavWriter::new(&mut cursor, spec).map_err(|e| e.to_string())?;
            for sample in samples {
                writer.write_sample(*sample).map_err(|e| e.to_string())?;
            }
            writer.finalize().map_err(|e| e.to_string())?;
        }
        Ok(cursor.into_inner())
    }

    fn finished_wav(finished: &FinishedRecordings) -> Result<Vec<u8>, String> {
        let queue = finished.lock().map_err(|e| e.to_string())?;
        queue
            .iter()
            .find(|(format, _)| *format == RecordFormat::Wav)
            .map(|(_, bytes)| bytes.clone())
            .ok_or_else(|| "un enregistrement WAV".to_string())
    }

    /// `frames` stereo frames of reproducible noise in `-1.0..1.0`.
    fn noise(seed: &mut u32, frames: usize) -> Vec<f32> {
        (0..frames * 2)
            .map(|_| {
                *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (*seed >> 8) as f32 / (1u32 << 23) as f32 - 1.0
            })
            .collect()
    }

    const CHUNKS: [usize; 8] = [1, 3, 4096, 4095, 4097, 20_000, 2, 9000];

    /// Chunks of any size (1 frame, block boundary, several blocks) produce
    /// the exact bytes of a direct `hound` encoding.
    #[test]
    fn wav_is_byte_identical_to_direct_encoding() -> Result<(), String> {
        let finished: FinishedRecordings = Arc::new(Mutex::new(Vec::new()));
        let mut rec = lossless(&finished);
        rec.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Wav });
        let mut all = Vec::new();
        let mut seed = 0x2545_F491u32;
        for chunk_frames in CHUNKS {
            let mut buffer = noise(&mut seed, chunk_frames);
            all.extend_from_slice(&buffer);
            rec.process_buffer(&mut buffer, 2);
        }
        rec.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Wav });
        // `NullRenderer` leaves the buffer untouched: what is recorded is what we fed.
        assert_eq!(finished_wav(&finished)?, expected_wav(&all)?);
        assert!(rec.stats().is_lossless());
        Ok(())
    }

    /// Without a thread, the caller's `pump` is the only thing that encodes:
    /// nothing is published before it, and the file is the exact bytes of the
    /// threaded recorder (same encoder).
    #[test]
    fn pumped_worker_encodes_without_a_thread() -> Result<(), String> {
        let finished: FinishedRecordings = Arc::new(Mutex::new(Vec::new()));
        let (mut rec, mut worker) =
            RecorderBackend::pumped(Box::new(NullRenderer), finished.clone(), 44100);
        rec.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Wav });
        let mut all = Vec::new();
        let mut seed = 0x2545_F491u32;
        for chunk_frames in CHUNKS {
            let mut buffer = noise(&mut seed, chunk_frames);
            all.extend_from_slice(&buffer);
            rec.process_buffer(&mut buffer, 2);
            worker.pump();
        }
        rec.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Wav });
        assert!(finished.lock().map_err(|e| e.to_string())?.is_empty(), "rien avant le pump");
        worker.pump();
        assert_eq!(finished_wav(&finished)?, expected_wav(&all)?);
        assert!(rec.stats().is_lossless());
        Ok(())
    }

    /// The callback disappears mid-take: the next `pump` closes and publishes
    /// the take, as the thread does when its queue closes.
    #[test]
    fn pumped_worker_publishes_the_take_when_the_callback_disappears() -> Result<(), String> {
        let finished: FinishedRecordings = Arc::new(Mutex::new(Vec::new()));
        let (mut rec, mut worker) =
            RecorderBackend::pumped(Box::new(NullRenderer), finished.clone(), 44100);
        rec.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Wav });
        let mut buffer = noise(&mut 7, 1000);
        let fed = buffer.clone();
        rec.process_buffer(&mut buffer, 2);
        rec.flush_pcm();
        drop(rec);
        worker.pump();
        assert_eq!(finished_wav(&finished)?, expected_wav(&fed)?);
        Ok(())
    }

    /// Without a consumer the pool runs dry: the overrun is counted, the WAV
    /// keeps its exact length with silence in place of the lost blocks, and the
    /// stop command still gets through.
    #[test]
    fn wav_overflow_is_counted_and_padded_with_silence() -> Result<(), String> {
        let finished: FinishedRecordings = Arc::new(Mutex::new(Vec::new()));
        let (mut rec, ports) = RecorderBackend::assemble(
            Box::new(NullRenderer),
            finished.clone(),
            44100,
            OverflowPolicy::DropAndCount,
        );
        rec.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Wav });
        let blocks = PCM_POOL_BLOCKS + 2;
        for _ in 0..blocks {
            rec.process_buffer(&mut vec![0.5f32; PCM_BLOCK_SAMPLES], 2);
        }
        rec.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Wav });

        let stats = rec.stats();
        assert_eq!(stats.dropped_pcm_samples(), 2 * PCM_BLOCK_SAMPLES as u64);
        assert_eq!(stats.lost_commands(), 0, "la fin passe toujours");
        assert!(!stats.is_lossless());

        worker::spawn(ports).map_err(|e| e.to_string())?;
        rec.wait_idle();
        let wav = finished_wav(&finished)?;
        let reader = hound::WavReader::new(std::io::Cursor::new(wav)).map_err(|e| e.to_string())?;
        let samples: Vec<f32> =
            reader.into_samples::<f32>().collect::<Result<_, _>>().map_err(|e| e.to_string())?;
        assert_eq!(samples.len(), blocks * PCM_BLOCK_SAMPLES, "durée exacte");
        let kept = PCM_POOL_BLOCKS * PCM_BLOCK_SAMPLES;
        assert!(samples[..kept].iter().all(|s| *s == 0.5));
        assert!(samples[kept..].iter().all(|s| *s == 0.0), "perte = silence");
        Ok(())
    }

    /// A full queue refuses MIDI data (counted) but never the stop command,
    /// and the file closes the notes that lost their `NoteOff`.
    #[test]
    fn midi_overflow_is_counted_and_stop_still_delivered() -> Result<(), String> {
        let finished: FinishedRecordings = Arc::new(Mutex::new(Vec::new()));
        let (mut rec, ports) = RecorderBackend::assemble(
            Box::new(NullRenderer),
            finished.clone(),
            44100,
            OverflowPolicy::DropAndCount,
        );
        rec.handle_event(AudioEvent::StartRecording { format: AudioRecordFormat::Midi });
        rec.handle_event(AudioEvent::NoteOn { note: 60, velocity: 90, channel: 0 });
        let attempts = QUEUE_CAPACITY * 2;
        for _ in 0..attempts {
            rec.handle_event(AudioEvent::NoteOff { note: 61, channel: 0 });
        }
        rec.handle_event(AudioEvent::StopRecording { format: AudioRecordFormat::Midi });

        let stats = rec.stats();
        assert!(stats.dropped_midi_events() >= (attempts - QUEUE_CAPACITY) as u64);
        assert_eq!(stats.lost_commands(), 0);

        worker::spawn(ports).map_err(|e| e.to_string())?;
        rec.wait_idle();
        let bytes = finished_midi(&finished)?;
        let smf = Smf::parse(&bytes).map_err(|e| e.to_string())?;
        let closing = smf.tracks[0]
            .iter()
            .filter(|e| {
                matches!(
                    e.kind,
                    TrackEventKind::Midi { message: MidiMessage::NoteOff { key, .. }, .. }
                        if key.as_int() == 60
                )
            })
            .count();
        assert_eq!(closing, 1, "la note 60 est fermée à la fin");
        Ok(())
    }
}

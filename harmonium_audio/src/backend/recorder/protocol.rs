//! Messages de taille fixe entre le callback audio et le thread d'enregistrement.
//!
//! Tout ce qui traverse la file SPSC est `Copy` ou un bloc PCM préalloué dont
//! le callback ne détient jamais la destruction : les blocs reviennent au
//! thread d'enregistrement par l'anneau de recyclage.

use std::sync::atomic::{AtomicU64, Ordering};

/// Échantillons (frames × canaux) par bloc PCM.
pub(super) const PCM_BLOCK_SAMPLES: usize = 8192;
/// Blocs PCM préalloués : ≈ 3 s de stéréo à 44,1 kHz d'avance au thread d'enregistrement.
pub(super) const PCM_POOL_BLOCKS: usize = 48;
/// Capacité de la file de messages (PCM, MIDI et commandes confondus).
pub(super) const QUEUE_CAPACITY: usize = 4096;
/// Places gardées pour les commandes de début/fin : les données (PCM, MIDI)
/// ne peuvent jamais prendre les dernières, un `Stop` entre toujours.
pub(super) const CONTROL_RESERVE: usize = 8;

/// Bloc d'échantillons entrelacés, alloué une fois par le thread d'enregistrement.
pub(super) struct PcmBlock {
    /// Rang (depuis le début de l'enregistrement WAV) du premier échantillon.
    pub start_sample: u64,
    pub len: usize,
    pub data: Box<[f32]>,
}

impl PcmBlock {
    pub(super) fn new() -> Self {
        Self { start_sample: 0, len: 0, data: vec![0.0; PCM_BLOCK_SAMPLES].into_boxed_slice() }
    }
}

/// Événement MIDI de taille fixe ; la position vient de l'horodatage en frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum MidiEvent {
    NoteOn {
        channel: u8,
        key: u8,
        vel: u8,
    },
    NoteOff {
        channel: u8,
        key: u8,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    Controller {
        channel: u8,
        controller: u8,
        value: u8,
    },
    /// Ferme les notes suivies sur ce canal (réponse à un `AllNotesOff`).
    AllNotesOff {
        channel: u8,
    },
    Tempo {
        micros_per_beat: u32,
    },
    TimeSignature {
        numerator: u8,
        denominator_pow: u8,
    },
    /// Nouvelle cadence ticks/frame, effective à l'horodatage du message.
    Clock {
        ticks_per_frame: f64,
    },
}

pub(super) enum RecordMsg {
    WavStart,
    /// Fin du WAV ; `total_samples` complète d'un silence les éventuels
    /// échantillons perdus en fin de prise.
    WavStop {
        total_samples: u64,
    },
    Pcm(PcmBlock),
    MidiStart {
        frame: u64,
    },
    Midi {
        frame: u64,
        event: MidiEvent,
    },
    MidiStop {
        frame: u64,
    },
}

/// Compteurs publiés par l'enregistreur : toute perte y est comptée, jamais silencieuse.
#[derive(Debug, Default)]
pub struct RecorderStats {
    dropped_pcm_samples: AtomicU64,
    dropped_midi_events: AtomicU64,
    lost_commands: AtomicU64,
}

impl RecorderStats {
    /// Échantillons PCM (frames × canaux) refusés faute de bloc libre ou de place :
    /// le WAV les remplace par du silence, la durée reste exacte.
    #[must_use]
    pub fn dropped_pcm_samples(&self) -> u64 {
        self.dropped_pcm_samples.load(Ordering::Relaxed)
    }

    /// Événements MIDI refusés faute de place dans la file.
    #[must_use]
    pub fn dropped_midi_events(&self) -> u64 {
        self.dropped_midi_events.load(Ordering::Relaxed)
    }

    /// Débuts/fins d'enregistrement qui n'ont pas pu être transmis (file saturée
    /// ou thread d'enregistrement absent).
    #[must_use]
    pub fn lost_commands(&self) -> u64 {
        self.lost_commands.load(Ordering::Relaxed)
    }

    /// Vrai si aucune donnée ni commande n'a été perdue.
    #[must_use]
    pub fn is_lossless(&self) -> bool {
        self.dropped_pcm_samples() == 0
            && self.dropped_midi_events() == 0
            && self.lost_commands() == 0
    }

    pub(super) fn add_pcm(&self, samples: usize) {
        self.dropped_pcm_samples.fetch_add(samples as u64, Ordering::Relaxed);
    }

    pub(super) fn add_midi(&self) {
        self.dropped_midi_events.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn add_command(&self) {
        self.lost_commands.fetch_add(1, Ordering::Relaxed);
    }
}

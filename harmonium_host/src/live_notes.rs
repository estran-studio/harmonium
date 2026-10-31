//! Entrée des notes jouées, indépendante du compositeur et de son emprunt mutable.

use harmonium_audio::played_notes::PlayedNoteEvent;

/// Nombre maximal d'événements en attente par producteur.
pub const LIVE_NOTE_CAPACITY: usize = 256;

/// Refus immédiat : aucun événement n'a été publié.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveNoteError {
    /// Le callback n'a pas encore libéré de place ; réessayer après son passage.
    Full,
    /// Canal hors `0..16`, ou donnée MIDI hors `0..128`.
    InvalidMidiData,
}

impl std::fmt::Display for LiveNoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Full => "file de notes jouées pleine",
            Self::InvalidMidiData => "donnée MIDI invalide",
        })
    }
}

impl std::error::Error for LiveNoteError {}

/// Producteur unique `Send`, sans verrou ni allocation à l'envoi.
///
/// Obtenu par `LiveHost::take_note_sender`. Non `Clone` : chaque file
/// SPSC conserve exactement un producteur. Le thread MIDI possède cette poignée
/// pendant que le thread principal compose ; il n'emprunte jamais `LiveHost`.
/// Un refus `Full` ne perd rien silencieusement : l'appelant doit notamment
/// réessayer les relâchements ou `all_notes_off` après le prochain callback.
pub struct LiveNoteSender {
    producer: rtrb::Producer<PlayedNoteEvent>,
}

impl LiveNoteSender {
    pub(crate) fn channel() -> (Self, rtrb::Consumer<PlayedNoteEvent>) {
        let (producer, consumer) = rtrb::RingBuffer::new(LIVE_NOTE_CAPACITY);
        (Self { producer }, consumer)
    }

    fn send(&mut self, event: PlayedNoteEvent) -> Result<(), LiveNoteError> {
        self.producer.push(event).map_err(|_| LiveNoteError::Full)
    }

    /// Attaque au prochain callback ; vélocité zéro équivaut à un relâchement.
    pub fn note_on(&mut self, channel: u8, note: u8, velocity: u8) -> Result<(), LiveNoteError> {
        if channel >= 16 || note >= 128 || velocity >= 128 {
            return Err(LiveNoteError::InvalidMidiData);
        }
        if velocity == 0 {
            self.note_off(channel, note)
        } else {
            self.send(PlayedNoteEvent::NoteOn { channel, note, velocity })
        }
    }

    /// Relâche la hauteur sur le canal joué, jamais sur une piste générée.
    pub fn note_off(&mut self, channel: u8, note: u8) -> Result<(), LiveNoteError> {
        if channel >= 16 || note >= 128 {
            return Err(LiveNoteError::InvalidMidiData);
        }
        self.send(PlayedNoteEvent::NoteOff { channel, note })
    }

    /// Coupe toutes les voix jouées, sans toucher aux pistes générées.
    pub fn all_notes_off(&mut self) -> Result<(), LiveNoteError> {
        self.send(PlayedNoteEvent::AllNotesOff)
    }

    /// Programme SoundFont `0..128`, commun aux seize canaux joués.
    pub fn set_program(&mut self, program: u8) -> Result<(), LiveNoteError> {
        if program >= 128 {
            return Err(LiveNoteError::InvalidMidiData);
        }
        self.send(PlayedNoteEvent::SetProgram(program))
    }
}

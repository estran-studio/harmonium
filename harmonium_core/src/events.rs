use crate::params::MusicalParams;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AudioEvent {
    NoteOn {
        note: u8,
        velocity: u8,
        channel: u8,
    },
    NoteOff {
        note: u8,
        channel: u8,
    },
    ControlChange {
        ctrl: u8,
        value: u8,
        channel: u8,
    },
    /// Coupe toutes les notes sur un channel (CC 123)
    AllNotesOff {
        channel: u8,
    },
    LoadFont {
        id: u32,
        bytes: Vec<u8>,
    },
    /// Route a logical channel to a SoundFont bank (-1 = default bank 0, >=0 = oxisynth bank)
    SetChannelRoute {
        channel: u8,
        bank: i32,
    },
    /// Send a GM Program Change to a logical channel
    ProgramChange {
        channel: u8,
        program: u8,
    },
    TimingUpdate {
        samples_per_step: usize,
    },
    /// Send musical parameters (key, time signature, etc.) to recorder
    UpdateMusicalParams {
        params: Box<MusicalParams>,
    },
    /// Métadonnées de la pulse audible, sans allocation sur le chemin audio.
    MeasureTiming {
        bpm: f32,
        time_signature: crate::params::TimeSignature,
        swing_ratio: f32,
    },
    StartRecording {
        format: AudioRecordFormat,
    },
    StopRecording {
        format: AudioRecordFormat,
    },
    /// Salle, bus et contrôleurs précalculés ; seul état de mixage rendu.
    SetMix {
        mix: crate::mix::PreparedMix,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecordFormat {
    Wav,
    Midi,
    MusicXml,
}

/// Formats capturés par le rendu ; MusicXML vient des mesures du compositeur.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioRecordFormat {
    Wav,
    Midi,
}

impl From<AudioRecordFormat> for RecordFormat {
    fn from(format: AudioRecordFormat) -> Self {
        match format {
            AudioRecordFormat::Wav => Self::Wav,
            AudioRecordFormat::Midi => Self::Midi,
        }
    }
}

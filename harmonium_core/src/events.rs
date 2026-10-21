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
        format: RecordFormat,
    },
    StopRecording {
        format: RecordFormat,
    },
    /// Set mixer gains for each instrument (0.0-1.0)
    SetMixerGains {
        lead: f32,
        chords: f32,
        bass: f32,
        snare: f32,
        hat: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecordFormat {
    Wav,
    Midi,
    MusicXml,
}

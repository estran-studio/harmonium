//! Voie SoundFont des notes jouées, indépendante des pistes générées.
//!
//! Le constructeur charge la banque hors callback. Les événements et le rendu
//! n'allouent pas ; le programme s'applique aux seize canaux mélodiques, y compris 9.

use std::io::Cursor;

use oxisynth::{MidiEvent, SoundFont, Synth, SynthDescriptor};

// Taille d'un bloc interne oxisynth ; sert aussi à purger son audio déjà calculé.
const CHUNK_FRAMES: usize = 64;

/// Événement borné transportable sans allocation jusqu'au callback audio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayedNoteEvent {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8 },
    AllNotesOff,
    SetProgram(u8),
}

/// Synthé propre aux notes jouées : ni routage, ni mixage des pistes générées.
///
/// Sans banque exploitable, la voie reste silencieuse. L'option `odin` ne change
/// pas cette voie : aucun preset Odin n'est fourni par cette API.
pub struct PlayedNotes {
    synth: Synth,
    scratch: [f32; CHUNK_FRAMES * 2],
}

impl PlayedNotes {
    /// Charge la banque et réserve les 256 voix avant le démarrage audio.
    #[must_use]
    pub fn new(sf2_bytes: Option<&[u8]>, sample_rate: f32) -> Self {
        let descriptor = SynthDescriptor {
            reverb_active: false,
            chorus_active: false,
            drums_channel_active: false,
            min_note_length: 0,
            ..SynthDescriptor::default()
        };
        // Le descripteur ne contient que des constantes valides. Comme les autres
        // voies audio, le taux de l'hôte est installé après cette construction.
        #[expect(clippy::expect_used, reason = "le descripteur constant est valide")]
        let mut synth = Synth::new(descriptor).expect("configuration SoundFont fixe valide");
        synth.set_sample_rate(sample_rate);
        if let Some(bytes) = sf2_bytes {
            // Les décodeurs SF2/SF3 peuvent paniquer sur une banque corrompue ;
            // ce chargement et la destruction de l'erreur restent hors callback.
            let font = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                SoundFont::load(&mut Cursor::new(bytes))
            }));
            if let Ok(Ok(font)) = font {
                synth.add_font(font, true);
            }
        }
        let mut played = Self { synth, scratch: [0.0; CHUNK_FRAMES * 2] };
        played.handle_event(PlayedNoteEvent::SetProgram(0));
        played
    }

    /// Applique un événement sans allocation ; les données MIDI invalides sont ignorées.
    /// Une vélocité nulle équivaut à `NoteOff`.
    pub fn handle_event(&mut self, event: PlayedNoteEvent) {
        match event {
            PlayedNoteEvent::NoteOn { channel, note, velocity }
                if channel < 16 && note < 128 && velocity < 128 =>
            {
                let _ =
                    self.synth.send_event(MidiEvent::NoteOn { channel, key: note, vel: velocity });
            }
            PlayedNoteEvent::NoteOff { channel, note } if channel < 16 && note < 128 => {
                let _ = self.synth.send_event(MidiEvent::NoteOff { channel, key: note });
            }
            PlayedNoteEvent::AllNotesOff => {
                for channel in 0..16 {
                    let _ = self.synth.send_event(MidiEvent::AllSoundOff { channel });
                }
                // AllSoundOff coupe les voix, mais le bloc interne peut encore
                // contenir jusqu'à 63 frames audibles. Une lecture jetée les purge.
                self.synth.write(self.scratch.as_mut_slice());
            }
            PlayedNoteEvent::SetProgram(program_id) if program_id < 128 => {
                for channel in 0..16 {
                    let _ = self.synth.send_event(MidiEvent::ProgramChange { channel, program_id });
                }
            }
            _ => {}
        }
    }

    /// Ajoute la voie à la sortie existante, par blocs de 64 frames au plus.
    ///
    /// En mono, les côtés sont moyennés ; à partir de deux canaux, seuls les deux
    /// premiers reçoivent le signal stéréo. Les frames incomplètes et les sorties
    /// à zéro canal sont laissées intactes.
    pub fn render_add(&mut self, output: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        let frames = output.len() / channels;
        let mut rendered = 0;
        while rendered < frames {
            let count = (frames - rendered).min(CHUNK_FRAMES);
            let scratch = &mut self.scratch[..count * 2];
            self.synth.write(&mut *scratch);
            for (index, frame) in scratch.chunks_exact(2).enumerate() {
                let start = (rendered + index) * channels;
                if channels == 1 {
                    output[start] += (frame[0] + frame[1]) * 0.5;
                } else {
                    output[start] += frame[0];
                    output[start + 1] += frame[1];
                }
            }
            rendered += count;
        }
    }
}

use harmonium_core::events::AudioEvent;

pub mod midi_backend;
pub mod recorder;
pub mod synth_backend;
pub mod wav_backend;

pub trait AudioRenderer: Send + Sync {
    /// Appelé à chaque tick logique (ex: changement de step)
    fn handle_event(&mut self, event: AudioEvent);

    /// Appelé pour générer l'audio par bloc
    /// output est un buffer entrelacé [L, R, L, R, ...]
    fn process_buffer(&mut self, output: &mut [f32], channels: usize);

    /// Allow downcasting to concrete types
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

use harmonium_core::events::AudioEvent;

pub mod midi_backend;
pub mod recorder;
pub mod synth_backend;
pub mod wav_backend;

pub trait AudioRenderer: Send {
    /// Appelé à chaque tick logique (ex: changement de step)
    fn handle_event(&mut self, event: AudioEvent);

    /// Appelé pour générer l'audio par bloc
    /// output est un buffer entrelacé [L, R, L, R, ...]
    fn process_buffer(&mut self, output: &mut [f32], channels: usize);

    /// Applique une transaction Odin sans en prendre la propriété. Les instances
    /// retirées restent dans la transaction, retournée au thread principal.
    #[cfg(feature = "odin")]
    fn apply_odin(&mut self, update: &mut crate::odin_bank::OdinUpdate) {
        assert!(
            update.patches.iter().all(Option::is_none),
            "ce renderer ne prend pas en charge Odin"
        );
    }

    /// Allow downcasting to concrete types
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

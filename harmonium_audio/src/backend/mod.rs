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

    /// Morphing sans ressources propriétaires, y compris pendant un fondu.
    #[cfg(feature = "odin")]
    fn morph_odin(&mut self, _patches: &[harmonium_core::odin::OdinPatches; 2]) {}

    /// Retire les instances seulement une fois le fondu terminé.
    #[cfg(feature = "odin")]
    fn finish_odin(&mut self, _update: &mut crate::odin_bank::OdinUpdate) -> bool {
        true
    }

    /// Snapshot fixe pour les contrats de rendu hors ligne.
    #[cfg(feature = "odin")]
    fn odin_diagnostics(&self) -> Option<crate::odin_voices::OdinDiagnostics> {
        None
    }

    /// Allow downcasting to concrete types
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

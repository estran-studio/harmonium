//! Mapping des émotions vers les courbes `[dynamics]` du profil actif.
use crate::params::{EngineParams, MusicalParams};

#[derive(Default)]
pub struct EmotionMapper;

impl EmotionMapper {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Conserve les réglages de transport et de rendu ; applique les quatre axes.
    pub fn apply_dynamics(
        &self,
        emotions: &EngineParams,
        dynamics: &crate::trad::dynamics::StyleDynamics,
        nominal_bpm: f32,
        params: &mut MusicalParams,
    ) {
        params.bpm = nominal_bpm;
        params.arousal = emotions.arousal;
        params.intensity = dynamics.intensity_from_emotions(emotions.arousal, emotions.tension);
        params.rhythm_density = emotions.density;
        params.rhythm_tension = emotions.tension;
        params.harmony_valence = emotions.valence;
        params.harmony_tension = emotions.tension;
        params.melody_smoothness = emotions.smoothness;
    }
}

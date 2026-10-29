//! Mapping des émotions vers les courbes `[dynamics]` du profil actif.
use crate::{
    StyleProfile,
    params::{EngineParams, MusicalParams},
};

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

/// Commandes du jeu sur le tempo et les émotions : état et sémantique uniques, partagés par
/// le composer (commandes immédiates) et le plan de temps partagé (commandes datées).
///
/// Le tempo direct du jeu (`set_bpm`) gagne sur les émotions et la rampe de set jusqu'à
/// `reset_bpm`, qui rend le tempo issu des émotions.
#[derive(Clone, Debug)]
pub struct GameControls {
    emotions: EngineParams,
    emotion_mode: bool,
    bpm_override: Option<f32>,
    emotion_mapped_bpm: f32,
}

impl GameControls {
    #[must_use]
    pub fn new(nominal_bpm: f32) -> Self {
        Self {
            emotions: EngineParams::default(),
            emotion_mode: false,
            bpm_override: None,
            emotion_mapped_bpm: nominal_bpm,
        }
    }

    /// Nouveau profil : le tempo revient au nominal du profil, sans tempo direct.
    pub fn reset_tempo(&mut self, nominal_bpm: f32) {
        self.bpm_override = None;
        self.emotion_mapped_bpm = nominal_bpm;
    }

    /// Passe en mode émotion (`true`) ou direct. En venant du mode direct, les quatre axes partent
    /// de ceux qui jouent ([`Self::axes`]) : changer ensuite un seul axe ne fait pas sauter les
    /// trois autres à des valeurs par défaut.
    pub fn set_emotion_mode(&mut self, on: bool, params: &MusicalParams) {
        if on && !self.emotion_mode {
            [
                self.emotions.arousal,
                self.emotions.valence,
                self.emotions.density,
                self.emotions.tension,
            ] = self.axes(params);
        }
        self.emotion_mode = on;
    }

    #[must_use]
    pub const fn is_emotion_mode(&self) -> bool {
        self.emotion_mode
    }

    /// Applique les quatre axes aux paramètres (ignoré hors mode émotion) ; le tempo direct,
    /// s'il est actif, reste en place.
    pub fn set_emotions(
        &mut self,
        [arousal, valence, density, tension]: [f32; 4],
        profile: &StyleProfile,
        params: &mut MusicalParams,
    ) {
        if !self.emotion_mode {
            return;
        }
        self.emotions.arousal = arousal;
        self.emotions.valence = valence;
        self.emotions.density = density;
        self.emotions.tension = tension;
        EmotionMapper::new().apply_dynamics(
            &self.emotions,
            &profile.dynamics,
            profile.meter.default_bpm,
            params,
        );
        self.emotion_mapped_bpm = params.bpm;
        if let Some(bpm) = self.bpm_override {
            params.bpm = bpm;
        }
    }

    /// Tempo direct du jeu : désactive l'offset émotionnel et la rampe de set.
    pub fn set_bpm(&mut self, bpm: f32, params: &mut MusicalParams) {
        let clamped = bpm.clamp(40.0, 300.0);
        self.bpm_override = Some(clamped);
        params.bpm = clamped;
        params.bpm_explicit = true;
    }

    /// Rend le tempo aux émotions et à la rampe de set.
    pub fn reset_bpm(&mut self, params: &mut MusicalParams) {
        self.bpm_override = None;
        params.bpm = self.emotion_mapped_bpm;
        params.bpm_explicit = false;
    }

    #[must_use]
    pub const fn bpm_override(&self) -> Option<f32> {
        self.bpm_override
    }

    #[must_use]
    pub const fn emotion_mapped_bpm(&self) -> f32 {
        self.emotion_mapped_bpm
    }

    /// Émotions courantes `[arousal, valence, density, tension]` (même hors
    /// mode émotion : valeurs par défaut) pour le morphing Odin audio.
    #[must_use]
    pub const fn emotions(&self) -> [f32; 4] {
        [self.emotions.arousal, self.emotions.valence, self.emotions.density, self.emotions.tension]
    }

    /// Les quatre axes `[arousal, valence, density, tension]` qui jouent : ceux commandés en mode
    /// émotion, sinon ceux des paramètres directs.
    #[must_use]
    pub const fn axes(&self, params: &MusicalParams) -> [f32; 4] {
        if self.emotion_mode {
            self.emotions()
        } else {
            [params.arousal, params.harmony_valence, params.rhythm_density, params.rhythm_tension]
        }
    }
}

//! Intégration Bevy de l'hôte vivant : ressources, commandes et horloge de jeu.
//! Aucun rendu Bevy Audio : la sortie reste celle de `LiveHost` (cpal).

use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashSet,
    rc::Rc,
    time::Duration,
};

use bevy::{
    asset::{AssetLoader, LoadContext, LoadState, io::Reader},
    prelude::*,
};
use harmonium::{
    LiveHost,
    clock::{BeatInfo, ClockReader, TransportClock},
};
use harmonium_audio::odin_source::OdinPresetSource;
use harmonium_core::EngineCommand;
pub use harmonium_core::StyleProfile;
use web_time::Instant;

#[cfg(target_arch = "wasm32")]
mod web;

/// Ressources et rythmes de l'intégration. Chemins relatifs à l'`AssetServer` du jeu.
#[derive(Resource, Clone)]
pub struct HarmoniumConfig {
    pub main_bank: Option<String>,
    pub percussion_bank: Option<String>,
    pub odin_presets: Vec<String>,
    pub pump_interval: Duration,
    pub emotion_interval: Duration,
    pub emotion_threshold: f32,
    /// Durée de chaque demi-fondu ; l'attente du callback au silence s'y ajoute.
    pub fade_duration: Duration,
    pub buffer_frames: Option<u32>,
    pub start_paused: bool,
}

impl Default for HarmoniumConfig {
    fn default() -> Self {
        Self {
            main_bank: None,
            percussion_bank: None,
            odin_presets: Vec::new(),
            pump_interval: Duration::from_millis(if cfg!(target_arch = "wasm32") {
                100
            } else {
                50
            }),
            emotion_interval: Duration::from_millis(100),
            emotion_threshold: 0.02,
            fade_duration: Duration::from_millis(500),
            buffer_frames: None,
            start_paused: false,
        }
    }
}

/// Cibles écrites par le jeu ; le plugin exécute seulement les changements nécessaires.
#[derive(Resource, Clone)]
pub struct HarmoniumControl {
    pub profile: StyleProfile,
    /// `[arousal, valence, density, tension]`, axes normalisés dans `0..=1`.
    pub emotions: [f32; 4],
    pub volume: f32,
    pub paused: bool,
    pub seed: u64,
}

impl Default for HarmoniumControl {
    fn default() -> Self {
        Self {
            profile: StyleProfile::default(),
            emotions: [0.5; 4],
            volume: 0.7,
            paused: false,
            seed: 1,
        }
    }
}

impl HarmoniumControl {
    /// Choisit un profil distribué sans modifier la cible si l'identifiant est inconnu.
    pub fn set_profile_id(&mut self, id: &str) -> Result<(), String> {
        self.profile = StyleProfile::named(id).map_err(|error| error.to_string())?;
        Ok(())
    }
}

/// Octets conservés par l'`AssetServer` ; le nom court d'un preset est son nom de fichier.
#[derive(Asset, TypePath)]
pub struct HarmoniumAsset {
    pub bytes: Vec<u8>,
    pub name: String,
}

#[derive(Default, TypePath)]
struct HarmoniumLoader;

impl AssetLoader for HarmoniumLoader {
    type Asset = HarmoniumAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let name = context
            .path()
            .path()
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "nom de ressource non UTF-8")
            })?
            .to_owned();
        Ok(HarmoniumAsset { bytes, name })
    }

    fn extensions(&self) -> &[&str] {
        &["sf2", "sf3", "odin"]
    }
}

/// État stable, lisible par un écran de chargement. Un échec n'est pas relancé à chaque frame.
#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub enum HarmoniumStatus {
    #[default]
    Loading,
    Ready,
    Failed(String),
}

#[derive(Resource, Default)]
struct ResourceHandles {
    main: Option<Handle<HarmoniumAsset>>,
    percussion: Option<Handle<HarmoniumAsset>>,
    presets: Vec<Handle<HarmoniumAsset>>,
}

/// Horloge publiée par le callback, jamais extrapolée en mesures depuis le temps de Bevy.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct MusicClock {
    pub valid: bool,
    pub playing: bool,
    pub bar: usize,
    /// Temps à la noire (1-based), sur la grille du transport.
    pub beat: f32,
    /// Pulsation dansée (1-based), distincte de la noire en 2/2 ou 6/8.
    pub pulse: u8,
    pub pulses_per_bar: u8,
    pub phase: f32,
    /// Tempo à la noire de la mesure entendue.
    pub bpm: f32,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Beat {
    pub bar: usize,
    pub pulse: u8,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bar {
    pub bar: usize,
}

/// À ordonner après les systèmes du jeu qui écrivent les cibles (dans `PostUpdate`).
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HarmoniumSet;

pub struct HarmoniumPlugin {
    config: HarmoniumConfig,
}

impl HarmoniumPlugin {
    #[must_use]
    pub fn new(config: HarmoniumConfig) -> Self {
        Self { config }
    }
}

impl Default for HarmoniumPlugin {
    fn default() -> Self {
        Self::new(HarmoniumConfig::default())
    }
}

impl Plugin for HarmoniumPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .init_resource::<HarmoniumControl>()
            .init_resource::<HarmoniumStatus>()
            .init_resource::<MusicClock>()
            .init_asset::<HarmoniumAsset>()
            .init_asset_loader::<HarmoniumLoader>()
            .add_message::<Beat>()
            .add_message::<Bar>()
            .add_systems(Startup, load_resources)
            .add_systems(PostUpdate, (start_host, drive_host).chain().in_set(HarmoniumSet));
        if self.config.start_paused {
            app.world_mut().resource_mut::<HarmoniumControl>().paused = true;
        }
    }
}

fn load_resources(mut commands: Commands, server: Res<AssetServer>, config: Res<HarmoniumConfig>) {
    commands.insert_resource(ResourceHandles {
        main: config.main_bank.as_ref().map(|path| server.load(path.clone())),
        percussion: config.percussion_bank.as_ref().map(|path| server.load(path.clone())),
        presets: config.odin_presets.iter().map(|path| server.load(path.clone())).collect(),
    });
}

fn start_host(world: &mut World) {
    if *world.resource::<HarmoniumStatus>() != HarmoniumStatus::Loading {
        return;
    }
    if world.contains_non_send::<HarmoniumRuntime>() {
        *world.resource_mut::<HarmoniumStatus>() = HarmoniumStatus::Ready;
        return;
    }
    let handles = world.resource::<ResourceHandles>();
    let assets = world.resource::<Assets<HarmoniumAsset>>();
    let server = world.resource::<AssetServer>();
    let config = world.resource::<HarmoniumConfig>();
    for handle in handles.main.iter().chain(handles.percussion.iter()).chain(handles.presets.iter())
    {
        if let Some(LoadState::Failed(error)) = server.get_load_state(handle.id()) {
            let message = format!(
                "ressource Harmonium {} : {error}",
                handle.path().map_or_else(|| "inconnue".to_owned(), ToString::to_string)
            );
            error!("{message}");
            *world.resource_mut::<HarmoniumStatus>() = HarmoniumStatus::Failed(message);
            return;
        }
        if !assets.contains(handle.id()) {
            return;
        }
    }
    let main = handles.main.as_ref().and_then(|handle| assets.get(handle));
    let percussion = handles.percussion.as_ref().and_then(|handle| assets.get(handle));
    let presets: Vec<_> = handles
        .presets
        .iter()
        .filter_map(|handle| assets.get(handle))
        .map(|asset| OdinPresetSource { name: &asset.name, bytes: &asset.bytes })
        .collect();
    info!(
        "Harmonium : ressources chargées (principale={}, percussion={}, presets={})",
        main.is_some(),
        percussion.is_some(),
        presets.len()
    );
    // Toujours ouvrir en pause : aucune mesure par défaut ne fuit avant l'initialisation.
    let result = LiveHost::start_paused(
        main.map(|asset| asset.bytes.as_slice()),
        percussion.map(|asset| asset.bytes.as_slice()),
        &presets,
        config.buffer_frames,
    )
    .and_then(HarmoniumRuntime::new);
    match result {
        Ok(runtime) => {
            world.insert_non_send(runtime);
            *world.resource_mut::<HarmoniumStatus>() = HarmoniumStatus::Ready;
            info!("Harmonium : hôte démarré en pause, initialisation des cibles");
        }
        Err(message) => {
            error!("Harmonium : {message}");
            *world.resource_mut::<HarmoniumStatus>() = HarmoniumStatus::Failed(message);
        }
    }
}

enum Fade {
    Idle,
    Out { elapsed: Duration, from: f32 },
    Silence { revision: u64, frame: Option<u64> },
    In { elapsed: Duration },
}

/// Hôte non-Send : accès sur le fil principal uniquement, y compris dans les callbacks web.
/// Le partage interne permet au minuteur web de pomper hors de `requestAnimationFrame`.
pub struct HarmoniumRuntime {
    host: Rc<RefCell<LiveHost>>,
    reader: ClockReader,
    initialized: bool,
    profile: Option<StyleProfile>,
    emotions: [f32; 4],
    emotion_elapsed: Duration,
    pump_elapsed: Duration,
    volume: f32,
    paused: bool,
    seed: u64,
    fade: Fade,
    last_beat: Option<(usize, u8)>,
    /// Ne pas publier l'ancienne mesure entre une commande Seek et son callback.
    restart_frame: Option<u64>,
    notices: HashSet<String>,
    /// Mètres réellement composés, pour calculer le tempo entendu, même après changement.
    meters: std::collections::BTreeMap<usize, usize>,
    #[cfg(target_arch = "wasm32")]
    web: Option<web::WebDriver>,
}

impl HarmoniumRuntime {
    /// Injecte notamment un `LiveHost::detached` pour simuler le callback sans carte son.
    pub fn new(mut host: LiveHost) -> Result<Self, String> {
        let reader = host.take_clock_reader().ok_or("lecteur d'horloge déjà pris")?;
        Ok(Self {
            host: Rc::new(RefCell::new(host)),
            reader,
            initialized: false,
            profile: None,
            emotions: [0.0; 4],
            emotion_elapsed: Duration::ZERO,
            pump_elapsed: Duration::ZERO,
            volume: 1.0,
            paused: true,
            seed: 0,
            fade: Fade::Idle,
            last_beat: None,
            notices: HashSet::new(),
            meters: std::collections::BTreeMap::new(),
            restart_frame: None,
            #[cfg(target_arch = "wasm32")]
            web: None,
        })
    }

    pub fn host(&self) -> Ref<'_, LiveHost> {
        self.host.borrow()
    }
    pub fn host_mut(&self) -> RefMut<'_, LiveHost> {
        self.host.borrow_mut()
    }
    pub fn volume(&self) -> f32 {
        self.volume
    }
    pub fn paused(&self) -> bool {
        self.paused
    }

    fn notice(&mut self, message: String) {
        if self.notices.insert(message.clone()) {
            warn!("Harmonium : {message}");
        }
    }

    fn apply(&mut self, command: EngineCommand) -> bool {
        let result = self.host.borrow_mut().apply(command);
        match result {
            Ok(()) => true,
            Err(error) => {
                self.notice(error);
                false
            }
        }
    }

    fn set_volume(&mut self, volume: f32) {
        if volume != self.volume && self.apply(EngineCommand::SetMasterVolume(volume)) {
            self.volume = volume;
        }
    }

    fn initialize(&mut self, target: &HarmoniumControl) -> Result<(), String> {
        let mut host = self.host.borrow_mut();
        host.apply(EngineCommand::SetWriteheadLookahead(harmonium::playback::MEASURE_WINDOW_CAP))?;
        host.apply(EngineCommand::SetStyleProfile(Box::new(target.profile.clone())))?;
        let [arousal, valence, density, tension] = target.emotions;
        host.apply(EngineCommand::SetEmotionParams { arousal, valence, density, tension })?;
        // Le redémarrage doit être la dernière invalidation : les commandes de
        // paramètres lisent encore l'ancien playhead jusqu'au prochain callback.
        host.apply(EngineCommand::SetSeed(target.seed))?;
        host.apply(EngineCommand::SetMasterVolume(target.volume))?;
        drop(host);
        self.profile = Some(target.profile.clone());
        self.emotions = target.emotions;
        self.seed = target.seed;
        self.volume = target.volume;
        self.initialized = true;
        Ok(())
    }

    fn pump(&mut self) {
        let snapshots = self.host.borrow_mut().pump();
        if let (Some(first), Some(last)) = (snapshots.first(), snapshots.last()) {
            info!(
                "Harmonium : composition mesures {}–{} ({} mesures)",
                first.index,
                last.index,
                snapshots.len()
            );
        }
        for snapshot in snapshots {
            self.meters.insert(snapshot.index, usize::from(snapshot.pulse.steps_per_pulse));
        }
    }

    fn transition(
        &mut self,
        target: &HarmoniumControl,
        config: &HarmoniumConfig,
        delta: Duration,
        clock: &TransportClock,
    ) {
        if matches!(self.fade, Fade::Idle) && self.profile.as_ref() != Some(&target.profile) {
            // Valider avant de descendre : un profil refusé ne coupe pas la musique.
            if let Err(error) = target.profile.validate() {
                self.notice(error.to_string());
                return;
            }
            self.fade = Fade::Out { elapsed: Duration::ZERO, from: self.volume };
        }
        let fade = std::mem::replace(&mut self.fade, Fade::Idle);
        self.fade = match fade {
            Fade::Idle => {
                self.set_volume(target.volume);
                Fade::Idle
            }
            Fade::Out { elapsed, from } => {
                let elapsed = elapsed.saturating_add(delta);
                let ratio = fraction(elapsed, config.fade_duration);
                self.set_volume(from * (1.0 - ratio));
                if ratio >= 1.0 && self.volume == 0.0 {
                    Fade::Silence { revision: self.host.borrow().volume_revision(), frame: None }
                } else {
                    Fade::Out { elapsed, from }
                }
            }
            Fade::Silence { revision, mut frame } => {
                // Ne pas confondre un callback qui avance avec une commande
                // consommée : une transaction Odin peut retenir la file.
                let settled = clock.volume_revision >= revision && clock.output_gain == 0.0;
                // Le tampon qui termine la rampe peut encore contenir des notes :
                // attendre aussi le suivant, entièrement à gain zéro.
                let silent_buffer =
                    settled && frame.is_some_and(|old| clock.buffer_start_frame > old);
                if !settled {
                    frame = None;
                } else if frame.is_none() {
                    frame = Some(clock.buffer_start_frame);
                }
                if silent_buffer
                    && self.apply(EngineCommand::SetStyleProfile(Box::new(target.profile.clone())))
                {
                    self.profile = Some(target.profile.clone());
                    // Réinstaller les émotions avant le redémarrage : une invalidation
                    // ultérieure repartirait de l'ancien playhead publié. SetSeed vide
                    // aussi les mesures passées, que Seek(1) aurait rejouées inchangées.
                    let [arousal, valence, density, tension] = target.emotions;
                    self.apply(EngineCommand::SetEmotionParams {
                        arousal,
                        valence,
                        density,
                        tension,
                    });
                    self.apply(EngineCommand::SetSeed(target.seed));
                    self.seed = target.seed;
                    self.emotions = target.emotions;
                    self.emotion_elapsed = Duration::ZERO;
                    self.restart_frame = Some(clock.buffer_start_frame);
                    self.meters.clear();
                    self.last_beat = None;
                    Fade::In { elapsed: Duration::ZERO }
                } else {
                    Fade::Silence { revision, frame }
                }
            }
            Fade::In { elapsed } => {
                // Une nouvelle demande pendant la montée redescend depuis le gain courant.
                if self.profile.as_ref() != Some(&target.profile) {
                    Fade::Out { elapsed: Duration::ZERO, from: self.volume }
                } else {
                    let elapsed = elapsed.saturating_add(delta);
                    let ratio = fraction(elapsed, config.fade_duration);
                    self.set_volume(target.volume * ratio);
                    if ratio >= 1.0 { Fade::Idle } else { Fade::In { elapsed } }
                }
            }
        };
    }
}

fn fraction(elapsed: Duration, duration: Duration) -> f32 {
    if duration.is_zero() { 1.0 } else { (elapsed.as_secs_f32() / duration.as_secs_f32()).min(1.0) }
}

#[allow(clippy::too_many_arguments)]
fn drive_host(
    runtime: Option<NonSendMut<HarmoniumRuntime>>,
    target: Res<HarmoniumControl>,
    config: Res<HarmoniumConfig>,
    time: Res<Time<Real>>,
    mut status: ResMut<HarmoniumStatus>,
    mut clock: ResMut<MusicClock>,
    mut beats: MessageWriter<Beat>,
    mut bars: MessageWriter<Bar>,
) {
    let Some(mut runtime) = runtime else {
        return;
    };
    if matches!(*status, HarmoniumStatus::Failed(_)) {
        return;
    }
    if !runtime.initialized {
        if let Err(error) = runtime.initialize(&target) {
            runtime.notice(error.clone());
            *status = HarmoniumStatus::Failed(error);
            return;
        }
        runtime.pump();
        #[cfg(target_arch = "wasm32")]
        match web::WebDriver::new(runtime.host.clone(), config.pump_interval) {
            Ok(driver) => runtime.web = Some(driver),
            Err(error) => {
                runtime.notice(error.clone());
                *status = HarmoniumStatus::Failed(error);
                return;
            }
        }
    }
    let delta = time.delta();
    let published = runtime.reader.read();
    if target.is_changed() || !matches!(runtime.fade, Fade::Idle) {
        runtime.transition(&target, &config, delta, &published);
    }
    runtime.emotion_elapsed = runtime.emotion_elapsed.saturating_add(delta);
    let significant = runtime.emotions.iter().zip(target.emotions).any(|(old, new)| {
        !old.is_finite() || (*old - new).abs() >= config.emotion_threshold.max(f32::EPSILON)
    });
    if significant
        && runtime.restart_frame.is_none()
        && runtime.emotion_elapsed >= config.emotion_interval
    {
        let [arousal, valence, density, tension] = target.emotions;
        if runtime.apply(EngineCommand::SetEmotionParams { arousal, valence, density, tension }) {
            runtime.emotions = target.emotions;
            runtime.emotion_elapsed = Duration::ZERO;
        }
    }
    if target.seed != runtime.seed && runtime.apply(EngineCommand::SetSeed(target.seed)) {
        runtime.seed = target.seed;
        runtime.last_beat = None;
        runtime.restart_frame = Some(published.buffer_start_frame);
        runtime.meters.clear();
    }
    let paused = target.paused;
    #[cfg(target_arch = "wasm32")]
    let paused = {
        if let Some(web) = &runtime.web {
            web.set_paused_target(target.paused);
            paused || !web.activated()
        } else {
            true
        }
    };
    if paused != runtime.paused {
        let result =
            if paused { runtime.host.borrow().pause() } else { runtime.host.borrow().resume() };
        match result {
            Ok(()) => runtime.paused = paused,
            Err(error) => runtime.notice(error),
        }
    }
    runtime.pump_elapsed = runtime.pump_elapsed.saturating_add(delta);
    #[cfg(not(target_arch = "wasm32"))]
    if runtime.pump_elapsed >= config.pump_interval {
        runtime.pump();
        runtime.pump_elapsed = Duration::ZERO;
    }
    // Le Worker web fournit aussi les mètres composés sans dépendre des frames UI.
    #[cfg(target_arch = "wasm32")]
    if let Some(web) = &runtime.web {
        let meters = web.take_meters();
        runtime.meters.extend(meters);
    }
    publish_clock(&mut runtime, published, &mut clock, &mut beats, &mut bars);
}

fn publish_clock(
    runtime: &mut HarmoniumRuntime,
    published: TransportClock,
    clock: &mut MusicClock,
    beats: &mut MessageWriter<Beat>,
    bars: &mut MessageWriter<Bar>,
) {
    clock.playing = !runtime.paused;
    if published.next_pulse_frame == 0 {
        clock.valid = false;
        return;
    }
    if runtime.paused {
        return;
    }
    if let Some(frame) = runtime.restart_frame {
        if published.buffer_start_frame == frame || published.position.bar != 1 {
            clock.valid = false;
            return;
        }
        runtime.restart_frame = None;
    }
    let info = BeatInfo::from_clock(&published, Instant::now());
    let steps_per_pulse = runtime.meters.get(&info.bar).copied();
    // Pas de tempo inventé depuis le nouveau profil lorsque l'ancienne mesure sonne.
    let bpm = steps_per_pulse
        .map_or(0.0, |steps| 60.0 * steps as f32 / 4.0 / info.pulse_period.as_secs_f32());
    *clock = MusicClock {
        valid: true,
        playing: true,
        bar: info.bar,
        beat: published.position.beat,
        pulse: info.pulse,
        pulses_per_bar: info.pulses_per_bar,
        phase: info.phase,
        bpm,
    };
    let current = (info.bar, info.pulse);
    if runtime.last_beat != Some(current) {
        // Rattraper les frames lentes à mètre constant. Après un saut/reprofilage,
        // seul le point réellement observé est publié, pas des temps imaginaires.
        if let Some((bar, pulse)) = runtime.last_beat.filter(|&(bar, _)| bar <= info.bar) {
            let mut cursor = (bar, pulse);
            while cursor < current {
                cursor = if cursor.1 < info.pulses_per_bar {
                    (cursor.0, cursor.1 + 1)
                } else {
                    (cursor.0 + 1, 1)
                };
                if cursor > current {
                    break;
                }
                beats.write(Beat { bar: cursor.0, pulse: cursor.1 });
                if cursor.1 == 1 {
                    bars.write(Bar { bar: cursor.0 });
                }
            }
        } else {
            beats.write(Beat { bar: info.bar, pulse: info.pulse });
            bars.write(Bar { bar: info.bar });
        }
        runtime.last_beat = Some(current);
    }
    // Mémoire bornée : seule la réserve de lecture et la mesure courante sont utiles.
    runtime.meters.retain(|&bar, _| bar >= info.bar);
}

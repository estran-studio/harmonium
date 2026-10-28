//! Harnais de déterminisme natif / wasm (constitution, principe II).
//!
//! Le même code produit, pour chaque profil distribué, chaque graine et chaque réglage
//! d'émotions, les `MeasureSnapshot` de 64 mesures en JSON. `determinism_dump` l'écrit en natif ;
//! compilé pour `wasm32-unknown-unknown` et chargé par Node, le même code produit les mêmes
//! fichiers (`wasm`). `run.sh` compare les deux dossiers octet par octet.
//!
//! Chaque cas passe par l'hôte vivant (`LiveHost::detached` : `apply` des commandes, `pump`), puis
//! rend un tampon (`process_buffer`) et lit l'horloge (`beat_info`) : sur wasm cela exerce
//! `web_time::Instant`, qui y remplace `std::time::Instant` (celui-ci y panique).

use std::{
    fmt::Write as _,
    sync::{Arc, atomic::AtomicU64},
};

use harmonium::{
    LiveHost,
    composer::MusicComposer,
    playback::{TaggedMeasure, pack},
};
use harmonium_core::{EngineCommand, MeasureSnapshot, StyleProfile, events::RecordFormat};
use web_time::Instant;

#[cfg(target_arch = "wasm32")]
mod wasm;

/// Mesures par cas.
pub const BARS: usize = 64;
/// Graines essayées.
pub const SEEDS: [u64; 3] = [1, 2, 3];
/// Émotions fixes `[arousal, valence, density, tension]` : un réglage moyen, un réglage extrême.
pub const EMOTIONS: [(&str, [f32; 4]); 2] =
    [("mid", [0.6, 0.5, 0.5, 0.4]), ("hi", [0.9, 0.2, 0.85, 0.9])];

/// Sondes en texte, un fichier chacune : `(nom du fichier, sonde)`. Natif et wasm les écrivent,
/// `run.sh` les compare comme les cas.
pub const PROBES: [(&str, Probe); 2] =
    [("session.txt", session_probe), ("recording.txt", recording_probe)];

/// Une sonde : le texte du fichier qu'elle produit.
pub type Probe = fn() -> Result<String, String>;

const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: usize = 2;
const BUFFER_FRAMES: usize = 512;
/// Banque de test du dépôt : la plus petite, de quoi rendre de vrais échantillons.
const TEST_SF2: &[u8] = include_bytes!("../../assets/test.sf2");

/// Un cas : un profil distribué, une graine, un réglage d'émotions.
#[derive(Clone, Copy)]
pub struct Case {
    profile: &'static StyleProfile,
    seed: u64,
    emotions_name: &'static str,
    emotions: [f32; 4],
}

impl Case {
    /// `<profil>_s<graine>_<émotions>.json`.
    #[must_use]
    pub fn file_name(&self) -> String {
        format!("{}_s{}_{}.json", self.profile.meta.id, self.seed, self.emotions_name)
    }
}

/// Tous les cas, dans l'ordre de `StyleProfile::distributed()`.
#[must_use]
pub fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for profile in StyleProfile::distributed() {
        for seed in SEEDS {
            for (emotions_name, emotions) in EMOTIONS {
                cases.push(Case { profile, seed, emotions_name, emotions });
            }
        }
    }
    cases
}

/// Exécute un cas : les `MeasureSnapshot` composés, en JSON.
///
/// # Errors
/// Hôte non créé, commande refusée, mesures manquantes, horloge muette ou sérialisation impossible.
pub fn run_case(case: &Case) -> Result<String, String> {
    let (mut host, mut playback) = LiveHost::detached(None, None, &[], SAMPLE_RATE)?;
    let [arousal, valence, density, tension] = case.emotions;
    host.apply(EngineCommand::SetStyleProfile(Box::new(case.profile.clone())))?;
    host.apply(EngineCommand::SetSeed(case.seed))?;
    host.apply(EngineCommand::SetEmotionParams { arousal, valence, density, tension })?;
    host.apply(EngineCommand::SetWriteheadLookahead(BARS))?;
    let snapshots = host.pump();
    check_bars(&snapshots)?;

    // Le tampon et l'horloge exercent `Instant` : sur wasm, `std::time::Instant::now()` panique.
    let mut buffer = vec![0.0f32; BUFFER_FRAMES * CHANNELS];
    playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
    host.pump();
    host.beat_info(Instant::now()).ok_or("horloge de pulsation muette après un tampon")?;

    serde_json::to_string(&snapshots).map_err(|error| error.to_string())
}

/// Les snapshots couvrent exactement les mesures `1..=BARS`, une fois chacune.
fn check_bars(snapshots: &[MeasureSnapshot]) -> Result<(), String> {
    let bars: Vec<usize> = snapshots.iter().map(|snapshot| snapshot.index).collect();
    let expected: Vec<usize> = (1..=BARS).collect();
    if bars == expected {
        Ok(())
    } else {
        Err(format!("mesures composées {bars:?}, attendu 1..={BARS}"))
    }
}

/// Tonalité et gamme tirées à l'ouverture d'une session par le composer, pour 32 graines : le
/// tirage d'affichage du composer est un tirage d'entier comme les autres (sur `u64`, pas `usize`).
///
/// # Errors
/// Jamais en pratique ; la signature est celle des sondes.
pub fn session_probe() -> Result<String, String> {
    let mut text = String::new();
    for seed in 1..=32u64 {
        let (measure_tx, _measure_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(1);
        let (_return_tx, return_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(1);
        let position = Arc::new(AtomicU64::new(pack(1, 0)));
        let composer =
            MusicComposer::new_with_seed(SAMPLE_RATE, measure_tx, return_rx, position, seed);
        writeln!(text, "seed={seed} key={} scale={}", composer.config.key, composer.config.scale)
            .map_err(|error| error.to_string())?;
    }
    Ok(text)
}

/// Sondage d'enregistrement : un WAV enregistré par l'hôte, puis relu. Ligne de texte, identique
/// en natif (thread d'encodage) et en wasm (travailleur pompé par `LiveHost::pump`) : taille du
/// fichier et nombre d'échantillons, qui ne dépendent que de la durée rendue.
///
/// # Errors
/// Hôte non créé, commande refusée, aucun fichier publié ou fichier illisible.
pub fn recording_probe() -> Result<String, String> {
    const BUFFERS: usize = 40;
    let (mut host, mut playback) = LiveHost::detached(Some(TEST_SF2), None, &[], SAMPLE_RATE)?;
    host.apply(EngineCommand::SetSeed(1))?;
    host.pump();
    host.apply(EngineCommand::StartRecording(RecordFormat::Wav))?;
    let mut buffer = vec![0.0f32; BUFFER_FRAMES * CHANNELS];
    // Le début passe par le callback : un tampon le consomme, puis `BUFFERS` sont enregistrés.
    playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
    for _ in 0..BUFFERS {
        playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
        host.pump();
    }
    host.apply(EngineCommand::StopRecording(RecordFormat::Wav))?;
    playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
    host.pump();

    let wav = host
        .take_recordings()
        .into_iter()
        .find_map(|(format, bytes)| (format == RecordFormat::Wav).then_some(bytes))
        .ok_or("aucun WAV publié")?;
    let reader = hound::WavReader::new(std::io::Cursor::new(&wav)).map_err(|e| e.to_string())?;
    Ok(format!("wav_bytes={} wav_samples={}\n", wav.len(), reader.len()))
}

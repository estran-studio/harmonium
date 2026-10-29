//! L'écran Live sans fenêtre : ce que l'interface commande au moteur, et ce qu'elle en montre.
//!
//! - [`Session`] : l'hôte (`LiveHost`), la partition rendue par `pump` ([`Score`]) et ce que le
//!   studio a commandé sans que le moteur le publie ([`Desk`] : mixeur, boucle, enregistrements…).
//! - [`Intent`] : une intention de l'interface. [`Session::perform`] la traduit en `EngineCommand`
//!   et l'exécute par `LiveHost::apply`, le seul chemin vers le moteur (celui du REPL et de l'OSC).
//! - [`LiveView`] : ce que l'écran dessine, construit à partir de la sortie publiée
//!   (`MeasureSnapshot`, `BeatInfo`, `EngineReport`) et de l'API publique de l'hôte, jamais d'un
//!   état interne du générateur.
//!
//! Le module ne connaît ni egui ni la plate-forme : les tests le pilotent sur un hôte détaché.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

use harmonium::{
    BeatInfo, LiveHost,
    playback::{MEASURE_WINDOW_CAP, TICKS_PER_BEAT, TransportPosition},
};
use harmonium_core::{
    EngineCommand, EngineReport, MeasureSnapshot, StyleProfile,
    events::RecordFormat,
    timeline::{Articulation, TrackId},
    trad::{Section, TunePosition, pitch_class_name},
};
use web_time::Instant;

/// Les pistes, dans l'ordre de l'écran (piano-roll et mixeur).
pub const TRACKS: [TrackId; 6] = [
    TrackId::Melody,
    TrackId::Countermelody,
    TrackId::Chords,
    TrackId::Bass,
    TrackId::Heel,
    TrackId::Toe,
];

/// Nom court d'une piste à l'écran.
#[must_use]
pub const fn track_label(track: TrackId) -> &'static str {
    match track {
        TrackId::Countermelody => "Counter",
        other => other.name(),
    }
}

/// Mesures déjà jouées montrées à gauche de la tête de lecture.
pub const BARS_BEHIND: usize = 4;

/// Mesures montrées : les dernières jouées, puis toute la fenêtre de lecture (la mesure en cours et
/// celles composées d'avance, 16 au plus).
pub const WINDOW_BARS: usize = BARS_BEHIND + MEASURE_WINDOW_CAP;

/// Mesures gardées derrière la tête de lecture (pour un retour par saut doux ou une boucle).
const HISTORY_BARS: usize = 64;

/// Les formats d'enregistrement, dans l'ordre de l'écran.
pub const RECORD_FORMATS: [RecordFormat; 3] =
    [RecordFormat::Wav, RecordFormat::Midi, RecordFormat::MusicXml];

#[must_use]
pub const fn format_label(format: RecordFormat) -> &'static str {
    match format {
        RecordFormat::Wav => "WAV",
        RecordFormat::Midi => "MIDI",
        RecordFormat::MusicXml => "MusicXML",
    }
}

#[must_use]
pub const fn format_extension(format: RecordFormat) -> &'static str {
    match format {
        RecordFormat::Wav => "wav",
        RecordFormat::Midi => "mid",
        RecordFormat::MusicXml => "musicxml",
    }
}

const fn format_index(format: RecordFormat) -> usize {
    match format {
        RecordFormat::Wav => 0,
        RecordFormat::Midi => 1,
        RecordFormat::MusicXml => 2,
    }
}

fn track_of_channel(channel: u8) -> Option<TrackId> {
    TrackId::ALL.into_iter().find(|track| track.channel() == channel)
}

// === Partition reçue ===

/// Les mesures composées, telles que `LiveHost::pump` les publie, rangées par numéro.
#[derive(Default)]
pub struct Score {
    bars: BTreeMap<usize, MeasureSnapshot>,
}

impl Score {
    /// Range un lot rendu par `pump`. Un lot est une suite de mesures composées d'un trait : les
    /// mesures déjà reçues à partir de son début et qu'il ne contient pas ont été invalidées
    /// (recomposition après une commande, nouvelle graine, saut) et sont oubliées.
    pub fn absorb(&mut self, batch: Vec<MeasureSnapshot>) {
        let Some(first) = batch.iter().map(|measure| measure.index).min() else { return };
        let fresh: BTreeSet<usize> = batch.iter().map(|measure| measure.index).collect();
        self.bars.retain(|index, _| *index < first || fresh.contains(index));
        for measure in batch {
            self.bars.insert(measure.index, measure);
        }
    }

    /// Oublie les mesures trop anciennes pour être revues.
    pub fn forget_before(&mut self, bar: usize) {
        let keep_from = bar.saturating_sub(HISTORY_BARS);
        self.bars.retain(|index, _| *index >= keep_from);
    }

    #[must_use]
    pub fn bar(&self, index: usize) -> Option<&MeasureSnapshot> {
        self.bars.get(&index)
    }

    /// Dernière mesure composée.
    #[must_use]
    pub fn last(&self) -> Option<usize> {
        self.bars.keys().next_back().copied()
    }
}

// === Ce que le studio a commandé ===

/// Ce que le studio a commandé et que le moteur ne publie pas : il le garde pour l'afficher.
/// Mis à jour seulement quand `LiveHost::apply` a accepté la commande.
#[derive(Clone, Debug)]
pub struct Desk {
    /// Gain choisi par piste (indice = canal) ; `None` : le niveau du profil.
    pub gains: [Option<f32>; 6],
    /// Pistes coupées à la main (indice = canal).
    pub muted: [bool; 6],
    /// Piste en solo : les autres sont coupées.
    pub solo: Option<TrackId>,
    /// Sourdine envoyée au moteur (indice = canal) : `muted`, plus le solo.
    sent_mutes: [bool; 6],
    pub master_volume: f32,
    pub output_muted: bool,
    /// Banque par piste (indice = canal) ; -1 : banque par défaut.
    pub routes: [i32; 6],
    /// Mesures composées d'avance ; `None` : la valeur par défaut du moteur.
    pub lookahead: Option<usize>,
    pub loop_range: Option<(usize, usize)>,
    /// Tempo audible imposé (BPM, rampe en pulsations) ; `None` : le tempo des mesures.
    pub audible_tempo: Option<(f32, u8)>,
    /// Enregistrements en cours, dans l'ordre de [`RECORD_FORMATS`].
    pub recording: [bool; 3],
    pub playing: bool,
    /// Profil dont les gains choisis valent : le moteur rend à chaque piste le niveau du profil
    /// quand le profil change.
    mix_profile: Arc<str>,
}

impl Desk {
    fn new(profile: Arc<str>) -> Self {
        Self {
            gains: [None; 6],
            muted: [false; 6],
            solo: None,
            sent_mutes: [false; 6],
            master_volume: 1.0,
            output_muted: false,
            routes: [-1; 6],
            lookahead: None,
            loop_range: None,
            audible_tempo: None,
            recording: [false; 3],
            playing: false,
            mix_profile: profile,
        }
    }

    /// Prend acte d'une commande acceptée par l'hôte.
    fn record(&mut self, command: &EngineCommand) {
        match command {
            EngineCommand::SetChannelGain { channel, gain } => {
                if let Some(slot) = self.gains.get_mut(usize::from(*channel)) {
                    *slot = Some(gain.clamp(0.0, 1.0));
                }
            }
            EngineCommand::SetChannelMute { channel, muted } => {
                if let Some(slot) = self.sent_mutes.get_mut(usize::from(*channel)) {
                    *slot = *muted;
                }
            }
            EngineCommand::SetChannelRoute { channel, bank_id } => {
                if let Some(slot) = self.routes.get_mut(usize::from(*channel)) {
                    *slot = *bank_id;
                }
            }
            EngineCommand::SetMasterVolume(volume) if volume.is_finite() => {
                self.master_volume = volume.clamp(0.0, 1.0);
            }
            EngineCommand::SetOutputMute(muted) => self.output_muted = *muted,
            EngineCommand::SetWriteheadLookahead(bars) => self.lookahead = Some(*bars),
            EngineCommand::SetLoop { start_bar, end_bar } => {
                self.loop_range = Some((*start_bar, *end_bar));
            }
            EngineCommand::ClearLoop => self.loop_range = None,
            EngineCommand::SetAudibleTempo { bpm, ramp_pulses } => {
                self.audible_tempo = Some((*bpm, *ramp_pulses));
            }
            EngineCommand::ClearAudibleTempo => self.audible_tempo = None,
            EngineCommand::Reset => {
                self.loop_range = None;
                self.audible_tempo = None;
            }
            EngineCommand::StartRecording(format) => self.recording[format_index(*format)] = true,
            EngineCommand::StopRecording(format) => self.recording[format_index(*format)] = false,
            _ => {}
        }
    }

    /// Les sourdines à envoyer pour que le moteur coupe ce que demandent `muted` et `solo`.
    fn mute_commands(&self) -> Vec<EngineCommand> {
        TrackId::ALL
            .into_iter()
            .filter_map(|track| {
                let channel = track.channel();
                let index = usize::from(channel);
                let muted = self.muted[index] || self.solo.is_some_and(|solo| solo != track);
                (muted != self.sent_mutes[index])
                    .then_some(EngineCommand::SetChannelMute { channel, muted })
            })
            .collect()
    }

    /// La piste joue : ni coupée à la main, ni écartée par un solo.
    #[must_use]
    pub fn audible(&self, track: TrackId) -> bool {
        !self.sent_mutes[usize::from(track.channel())]
    }
}

// === Intentions de l'interface ===

/// Ce que l'interface demande.
#[derive(Clone, Debug, PartialEq)]
pub enum Intent {
    /// Une commande du moteur, telle quelle.
    Command(EngineCommand),
    /// Couper ou rendre une piste.
    Mute(TrackId, bool),
    /// Mettre une piste en solo (les autres se taisent) ; `None` : fin du solo.
    Solo(Option<TrackId>),
    /// Boucler entre deux mesures, dans n'importe quel ordre (glisser de droite à gauche).
    Loop(usize, usize),
    /// Lecture (`true`) ou pause de la sortie audio.
    Play(bool),
}

// === Session ===

/// Un enregistrement terminé, prêt à écrire (natif) ou à télécharger (web).
pub struct Recording {
    pub format: RecordFormat,
    pub bytes: Vec<u8>,
    /// `harmonium-<profil>-seed<graine>.<extension>`.
    pub file_name: String,
}

/// Le moteur tel que l'écran Live le voit.
pub struct Session {
    host: LiveHost,
    score: Score,
    desk: Desk,
}

impl Session {
    #[must_use]
    pub fn new(host: LiveHost) -> Self {
        let profile = host.musical_params().profile_id.clone();
        Self { host, score: Score::default(), desk: Desk::new(profile) }
    }

    /// L'hôte, pour ses lectures publiques (`emotions`, `musical_params`, `poll_state`…).
    #[must_use]
    pub fn host(&self) -> &LiveHost {
        &self.host
    }

    #[must_use]
    pub fn desk(&self) -> &Desk {
        &self.desk
    }

    #[must_use]
    pub fn score(&self) -> &Score {
        &self.score
    }

    /// Composition d'avance (`LiveHost::pump`) ; les mesures rendues entrent dans la partition.
    pub fn pump(&mut self) {
        let batch = self.host.pump();
        self.score.absorb(batch);
        self.score.forget_before(self.host.transport_position().bar);
    }

    /// Exécute une intention par `LiveHost::apply`.
    pub fn perform(&mut self, intent: Intent) -> Result<(), String> {
        match intent {
            Intent::Command(command) => self.apply(command),
            Intent::Mute(track, muted) => {
                self.desk.muted[usize::from(track.channel())] = muted;
                self.sync_mutes()
            }
            Intent::Solo(track) => {
                self.desk.solo = track;
                self.sync_mutes()
            }
            Intent::Loop(a, b) => {
                self.apply(EngineCommand::SetLoop { start_bar: a.min(b), end_bar: a.max(b) })
            }
            Intent::Play(true) => {
                self.host.resume()?;
                self.desk.playing = true;
                Ok(())
            }
            Intent::Play(false) => {
                self.host.pause()?;
                self.desk.playing = false;
                Ok(())
            }
        }
    }

    fn apply(&mut self, command: EngineCommand) -> Result<(), String> {
        self.host.apply(command.clone())?;
        self.desk.record(&command);
        // Un nouveau profil rend à chaque piste son niveau : les gains choisis ne valent plus.
        let profile = &self.host.musical_params().profile_id;
        if *profile != self.desk.mix_profile {
            self.desk.mix_profile = profile.clone();
            self.desk.gains = [None; 6];
        }
        Ok(())
    }

    fn sync_mutes(&mut self) -> Result<(), String> {
        for command in self.desk.mute_commands() {
            self.apply(command)?;
        }
        Ok(())
    }

    /// Les enregistrements terminés depuis le dernier appel, nommés.
    pub fn take_recordings(&mut self) -> Vec<Recording> {
        let profile = self.host.musical_params().profile_id.clone();
        let seed = self.host.session_seed();
        self.host
            .take_recordings()
            .into_iter()
            .map(|(format, bytes)| Recording {
                format,
                bytes,
                file_name: format!("harmonium-{profile}-seed{seed}.{}", format_extension(format)),
            })
            .collect()
    }

    /// Ce que l'écran dessine à l'instant `now`.
    pub fn view(&mut self, now: Instant) -> LiveView {
        let position = self.host.transport_position();
        let current = position.bar.max(1);
        let profile = self.host.profile_bar_info(current).map(|(plan, _)| plan.profile);
        let part_names: Vec<String> = profile
            .as_ref()
            .map(|profile| profile.form.parts.iter().map(|part| part.name.clone()).collect())
            .unwrap_or_default();
        let strain_lengths: Vec<usize> = profile
            .as_ref()
            .map(|profile| profile.form.parts.iter().map(|part| part.roles.len()).collect())
            .unwrap_or_default();
        let sounding = self.score.bar(current);
        let beat = self.host.beat_info(now).and_then(|beat| BeatView::new(&beat, now));
        let audible_bpm = match (&beat, sounding) {
            (Some(beat), Some(measure)) => {
                audible_bpm(beat.pulse_period, usize::from(measure.pulse.steps_per_pulse))
            }
            _ => None,
        };
        let score =
            score_view(&self.score, position, &part_names, self.desk.loop_range, self.score.last());
        let params = self.host.musical_params();
        let profile_id = params.profile_id.to_string();
        let profile_name = StyleProfile::distributed()
            .iter()
            .find(|profile| profile.meta.id == profile_id)
            .map_or_else(|| profile_id.clone(), |profile| profile.meta.name.clone());
        let dynamics = &self.host.profile().dynamics;
        let mixer = TRACKS
            .into_iter()
            .map(|track| {
                let index = usize::from(track.channel());
                let level = params.profile_mix.tracks[index].volume;
                MixerStrip {
                    track,
                    gain: self.desk.gains[index].unwrap_or_else(|| gain_of_volume(level)),
                    muted: self.desk.muted[index],
                    soloed: self.desk.solo == Some(track),
                    audible: self.desk.audible(track),
                    route: self.desk.routes[index],
                    density_gate: dynamics
                        .density_tracks
                        .iter()
                        .find(|gate| gate.track == track && params.rhythm_density < gate.minimum)
                        .map(|gate| gate.minimum),
                }
            })
            .collect();
        let ts = params.time_signature;
        let mut view = LiveView {
            profile_id,
            profile_name,
            seed: self.host.session_seed(),
            emotion_mode: self.host.is_emotion_mode(),
            emotions: self.host.emotions(),
            intensity: params.intensity,
            feet: params.feet_density,
            swing: params.swing_ratio,
            nominal_bpm: params.bpm,
            time_signature: (ts.numerator, ts.denominator),
            modules: [params.enable_rhythm, params.enable_harmony, params.enable_melody],
            audible_bpm,
            sounding_signature: sounding
                .map(|measure| (measure.time_sig_numerator, measure.time_sig_denominator)),
            beat,
            score,
            state: Vec::new(),
            mixer,
            desk: self.desk.clone(),
        };
        let position = sounding.and_then(|measure| measure.position);
        view.state =
            state_groups(&view, self.host.poll_state(), position, &part_names, &strain_lengths);
        view
    }
}

/// Gain linéaire d'un volume MIDI (CC7) : l'inverse de ce que fait le moteur
/// (`volume = 127 × √gain`).
fn gain_of_volume(volume: u8) -> f32 {
    let level = f32::from(volume) / 127.0;
    level * level
}

/// Tempo à la noire entendu, tiré de la durée de la pulsation publiée par `BeatInfo` et du nombre
/// de pas par pulsation de la mesure qui sonne (4 pas à la noire).
#[must_use]
pub fn audible_bpm(pulse_period: Duration, steps_per_pulse: usize) -> Option<f32> {
    let seconds = pulse_period.as_secs_f32();
    (seconds > 0.0 && steps_per_pulse > 0)
        .then(|| 60.0 / seconds * steps_per_pulse as f32 / TICKS_PER_BEAT as f32)
}

// === Vues ===

/// Tout ce que l'écran Live dessine.
#[derive(Clone, Debug)]
pub struct LiveView {
    pub profile_id: String,
    pub profile_name: String,
    pub seed: u64,
    pub emotion_mode: bool,
    /// `[arousal, valence, density, tension]`.
    pub emotions: [f32; 4],
    pub intensity: f32,
    pub feet: u8,
    pub swing: f32,
    /// Tempo nominal commandé (celui des prochaines mesures).
    pub nominal_bpm: f32,
    /// Signature commandée.
    pub time_signature: (usize, usize),
    /// Modules rythme, harmonie, mélodie.
    pub modules: [bool; 3],
    /// Tempo à la noire entendu ; `None` avant la première pulsation.
    pub audible_bpm: Option<f32>,
    /// Signature de la mesure qui sonne.
    pub sounding_signature: Option<(usize, usize)>,
    pub beat: Option<BeatView>,
    pub score: ScoreView,
    /// Le panneau d'état (l'équivalent de `state` dans le REPL).
    pub state: Vec<StateGroup>,
    pub mixer: Vec<MixerStrip>,
    pub desk: Desk,
}

/// L'horloge de pulsation, lue dans `BeatInfo`.
#[derive(Clone, Debug, PartialEq)]
pub struct BeatView {
    pub bar: usize,
    pub pulse: u8,
    pub pulses_per_bar: u8,
    pub phase: f32,
    pub downbeat: bool,
    pub pulse_period: Duration,
    pub next_pulse_in: Duration,
}

impl BeatView {
    /// `None` tant que la sortie n'a pas joué de tampon (pas encore de pulsation prédite).
    #[must_use]
    pub fn new(beat: &BeatInfo, now: Instant) -> Option<Self> {
        (beat.next_pulse_frame > 0).then(|| Self {
            bar: beat.bar,
            pulse: beat.pulse,
            pulses_per_bar: beat.pulses_per_bar,
            phase: beat.phase,
            downbeat: beat.is_downbeat,
            pulse_period: beat.pulse_period,
            next_pulse_in: beat.next_pulse_at.saturating_duration_since(now),
        })
    }
}

/// Les mesures montrées : quelques-unes déjà jouées, la mesure en cours et celles composées
/// d'avance.
#[derive(Clone, Debug)]
pub struct ScoreView {
    pub first_bar: usize,
    /// [`WINDOW_BARS`] mesures à partir de `first_bar`.
    pub bars: Vec<BarView>,
    /// Mesure sous la tête de lecture.
    pub current_bar: usize,
    /// Tête de lecture, en mesures depuis le bord gauche de `first_bar`.
    pub playhead: f32,
    /// Dernière mesure composée.
    pub composed_through: Option<usize>,
    pub loop_range: Option<(usize, usize)>,
}

#[derive(Clone, Debug)]
pub struct BarView {
    pub index: usize,
    /// `None` : pas encore composée.
    pub content: Option<BarContent>,
}

#[derive(Clone, Debug)]
pub struct BarContent {
    pub steps: usize,
    pub chords: Vec<ChordMark>,
    pub notes: Vec<NoteView>,
    pub form: Option<FormMark>,
}

/// Un accord à son pas (`chord_changes`).
#[derive(Clone, Debug, PartialEq)]
pub struct ChordMark {
    pub step: usize,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoteView {
    pub track: TrackId,
    pub pitch: u8,
    pub start_step: usize,
    /// 0 : frappe sans durée (percussion).
    pub duration_steps: usize,
    pub velocity: u8,
    /// Telle que publiée : le piano-roll dessine chaque variante (liée, legato, détachée…).
    pub articulation: Articulation,
}

/// La place d'une mesure dans la forme (`TunePosition`).
#[derive(Clone, Debug, PartialEq)]
pub struct FormMark {
    pub intro: bool,
    /// Indice de la partie (couleur).
    pub part: usize,
    /// « A1 » : partie et passe ; « Intro ».
    pub label: String,
    pub bar_in_strain: usize,
    /// Air dans le set (1-based) et longueur du set (0 : sans fin).
    pub tune_in_set: usize,
    pub tunes_in_set: usize,
    pub tune_index: usize,
    /// Tonalité et mode entendus (« D Dorian »).
    pub key_mode: String,
}

fn part_name(part_names: &[String], part: usize) -> String {
    part_names.get(part).cloned().unwrap_or_else(|| {
        char::from_u32(u32::from(b'A') + part as u32).map_or_else(|| part.to_string(), String::from)
    })
}

fn key_mode(position: &TunePosition) -> String {
    format!("{} {:?}", pitch_class_name(position.key), position.mode)
}

fn form_mark(position: &TunePosition, part_names: &[String]) -> FormMark {
    let intro = position.section == Section::Intro;
    FormMark {
        intro,
        part: position.part.index(),
        label: if intro {
            "Intro".to_string()
        } else {
            format!("{}·{}", part_name(part_names, position.part.index()), position.pass)
        },
        bar_in_strain: position.bar_in_strain,
        tune_in_set: position.tune_in_set,
        tunes_in_set: position.tunes_in_set,
        tune_index: position.tune_index,
        key_mode: key_mode(position),
    }
}

fn bar_content(measure: &MeasureSnapshot, part_names: &[String]) -> BarContent {
    let chords = if measure.chord_changes.is_empty() {
        vec![ChordMark { step: 0, name: measure.chord_name.clone() }]
    } else {
        measure
            .chord_changes
            .iter()
            .map(|change| ChordMark {
                step: change.start_step,
                name: change.context.chord_name.clone(),
            })
            .collect()
    };
    let notes = measure
        .notes
        .iter()
        .filter_map(|note| {
            Some(NoteView {
                track: track_of_channel(note.track)?,
                pitch: note.pitch,
                start_step: note.start_step,
                duration_steps: note.duration_steps,
                velocity: note.velocity,
                articulation: note.articulation,
            })
        })
        .collect();
    BarContent {
        steps: measure.steps.max(1),
        chords,
        notes,
        form: measure.position.as_ref().map(|position| form_mark(position, part_names)),
    }
}

/// La fenêtre de mesures autour de la tête de lecture.
#[must_use]
pub fn score_view(
    score: &Score,
    position: TransportPosition,
    part_names: &[String],
    loop_range: Option<(usize, usize)>,
    composed_through: Option<usize>,
) -> ScoreView {
    let current = position.bar.max(1);
    let first_bar = current.saturating_sub(BARS_BEHIND).max(1);
    let bars = (first_bar..first_bar + WINDOW_BARS)
        .map(|index| BarView {
            index,
            content: score.bar(index).map(|measure| bar_content(measure, part_names)),
        })
        .collect();
    let steps = score.bar(current).map_or(TICKS_PER_BEAT * 4, |measure| measure.steps.max(1));
    let within = (position.step as f32 / steps as f32).clamp(0.0, 1.0);
    ScoreView {
        first_bar,
        bars,
        current_bar: current,
        playhead: (current - first_bar) as f32 + within,
        composed_through,
        loop_range,
    }
}

/// Un groupe du panneau d'état.
#[derive(Clone, Debug, PartialEq)]
pub struct StateGroup {
    pub title: &'static str,
    pub rows: Vec<(&'static str, String)>,
}

fn on_off(value: bool) -> String {
    if value { "on" } else { "off" }.to_string()
}

/// Le panneau d'état : ce que montre `state` dans le REPL. Tonalité, mode et forme viennent de la
/// position publiée de la mesure qui sonne ; le tempo, de la pulsation entendue (`BeatInfo`).
fn state_groups(
    view: &LiveView,
    report: Option<&EngineReport>,
    position: Option<TunePosition>,
    part_names: &[String],
    strain_lengths: &[usize],
) -> Vec<StateGroup> {
    let mut style = vec![
        ("Style", format!("{} ({})", view.profile_name, view.profile_id)),
        ("Feet density", view.feet.to_string()),
        ("Mode", if view.emotion_mode { "emotion" } else { "direct" }.to_string()),
        ("Seed", view.seed.to_string()),
    ];
    if let Some(position) = position {
        style.push(("Tune key/mode", key_mode(&position)));
        let form = if position.section == Section::Intro {
            "intro".to_string()
        } else {
            let part = position.part.index();
            let length = strain_lengths.get(part).map_or_else(String::new, |len| format!("/{len}"));
            format!(
                "{} pass {} · bar {}{length}",
                part_name(part_names, part),
                position.pass,
                position.bar_in_strain
            )
        };
        style.push(("Form position", form));
        let set_length = if position.tunes_in_set == 0 {
            String::new()
        } else {
            format!(" of {}", position.tunes_in_set)
        };
        style.push((
            "Tune",
            format!(
                "{}{set_length} · round {} of {}",
                position.tune_in_set, position.tour, position.tours
            ),
        ));
    }
    let mut groups = vec![StateGroup { title: "Style", rows: style }];

    let mut timing = vec![(
        "Tempo (audible)",
        view.audible_bpm.map_or_else(|| "—".to_string(), |bpm| format!("{bpm:.1} BPM")),
    )];
    if let Some(report) = report {
        let params = &report.musical_params;
        let pulse = params.effective_pulse(TICKS_PER_BEAT);
        timing.extend([
            ("Swing", format!("{:.2} (0.5 = straight)", params.swing_ratio)),
            (
                "Pulse",
                format!("{} steps × {} per bar", pulse.steps_per_pulse, pulse.pulses_per_bar),
            ),
            (
                "Time signature",
                format!(
                    "{}/{}",
                    report.time_signature.numerator, report.time_signature.denominator
                ),
            ),
            ("Bar", report.current_bar.max(1).to_string()),
            ("Beat", report.current_beat.max(1).to_string()),
            ("Step", (report.current_step + 1).to_string()),
        ]);
        groups.push(StateGroup { title: "Timing", rows: timing });
        // `progression_name` / `progression_length` (comme `session_key` / `session_scale`) ne
        // sont jamais remplis par l'hôte : vestiges, non montrés.
        groups.push(StateGroup {
            title: "Harmony",
            rows: vec![("Current chord", report.current_chord.to_string())],
        });
        groups.push(StateGroup {
            title: "Modules",
            rows: vec![
                ("Rhythm", on_off(params.enable_rhythm)),
                ("Harmony", on_off(params.enable_harmony)),
                ("Melody", on_off(params.enable_melody)),
            ],
        });
    } else {
        groups.push(StateGroup { title: "Timing", rows: timing });
    }
    groups
}

/// Une tranche du mixeur.
#[derive(Clone, Debug, PartialEq)]
pub struct MixerStrip {
    pub track: TrackId,
    /// Gain linéaire `0..=1` : celui choisi, sinon le niveau du profil.
    pub gain: f32,
    /// Coupée à la main.
    pub muted: bool,
    pub soloed: bool,
    /// Ni coupée, ni écartée par un solo.
    pub audible: bool,
    pub route: i32,
    /// Seuil de densité du profil (`density_tracks`) quand la densité courante est en dessous :
    /// la piste ne joue pas dans les prochaines mesures.
    pub density_gate: Option<f32>,
}

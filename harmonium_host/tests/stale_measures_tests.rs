//! Aucune mesure périmée n'est jouée après une commande (E13).
//!
//! Scénario de training porté dans le moteur : un hôte hors ligne joue (ou
//! prépare) une séance A, reçoit une commande, puis joue. Le MIDI enregistré
//! (attaques des pistes 0 à 2, position en ticks comprise) doit égaler les
//! snapshots de la mesure que la commande a produite, dès la première mesure
//! jouée après elle. Trois états de départ :
//!
//! - `Fresh` : A est préparée et publiée (file pleine), aucun callback n'a tourné ;
//!   la commande est suivie d'un `pump` avant le premier callback ;
//! - `Parked` : A joue jusqu'au milieu d'une mesure, le flux s'arrête, la commande
//!   et quelques `pump` arrivent avant que le callback reprenne ;
//! - `Live` : A joue, la commande arrive sans interruption du callback.

// Les préparations fallibles sont des assertions de test, jamais du code temps réel.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{collections::BTreeMap, time::Instant};

use harmonium::{
    LiveHost,
    audio::create_offline_engine,
    playback::{PlaybackCommand, PlaybackEngine},
};
use harmonium_core::{
    EngineCommand, MeasureSnapshot, StyleProfile,
    events::{AudioRecordFormat, RecordFormat},
};
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};

const SAMPLE_RATE: f64 = 8000.0;
const TICKS_PER_STEP: f64 = 120.0;
/// Mesures vérifiées après la commande.
const CHECKED_BARS: usize = 3;
/// Pas courant à l'instant de la commande, pour `Parked` et `Live`.
const COMMAND_BAR: usize = 4;
const COMMAND_STEP: usize = 5;

fn profile(id: &str) -> StyleProfile {
    let mut profile = StyleProfile::named(id).expect("profil intégré");
    // Les ornements et la performance ajoutent des attaques : isoler l'identité des mesures.
    profile.performance = Default::default();
    profile.ornaments = Default::default();
    profile
}

fn configure(host: &mut LiveHost, style: &str, seed: u64, bpm: f32) {
    host.apply(EngineCommand::SetStyleProfile(Box::new(profile(style)))).unwrap();
    host.apply(EngineCommand::SetBpm(bpm)).unwrap();
    host.apply(EngineCommand::SetSwing(0.5)).unwrap();
    host.apply(EngineCommand::SetSeed(seed)).unwrap();
    host.apply(EngineCommand::Seek(1)).unwrap();
}

struct Rig {
    host: LiveHost,
    audio: PlaybackEngine,
    /// Dernier snapshot publié par mesure : la vérité du compositeur.
    snapshots: BTreeMap<usize, MeasureSnapshot>,
}

impl Rig {
    fn new() -> Self {
        let (mut host, audio) =
            LiveHost::detached(None, None, &[], SAMPLE_RATE).expect("hôte sans périphérique");
        host.apply(EngineCommand::SetWriteheadLookahead(16)).unwrap();
        Self { host, audio, snapshots: BTreeMap::new() }
    }

    fn pump(&mut self) {
        for snapshot in self.host.pump() {
            self.snapshots.insert(snapshot.index, snapshot);
        }
    }

    fn render(&mut self, frames: usize) {
        let mut buffer = vec![0.0; frames * 2];
        self.audio.process_buffer(&mut buffer, 2, Instant::now());
    }

    /// Joue image par image jusqu'à ce que le transport annonce `(bar, step)` :
    /// le prochain image entre dans ce pas.
    fn play_until(&mut self, bar: usize, step: usize) {
        let reached = (0..400_000usize).any(|frame| {
            if frame % 64 == 0 {
                self.pump();
            }
            self.render(1);
            let position = self.host.transport_position();
            (position.bar, position.step) == (bar, step)
        });
        assert!(reached, "position {bar}:{step} jamais atteinte");
    }
}

#[derive(Clone, Copy, Debug)]
enum State {
    Fresh,
    Parked,
    Live,
}

#[derive(Clone, Copy, Debug)]
enum Command {
    /// Training : profil, tempo, swing, graine, seek 1.
    Session,
    NewMelody,
    SetSeed,
    /// Deux commandes sans callback entre elles : la seconde voit un transport en retard.
    SeedThenBpm,
    SetStyleProfile,
    SetBpm,
    SetSwing,
    SeekPlayhead,
    Seek,
}

impl Command {
    /// La commande renvoie la lecture à une mesure donnée (sinon la mesure en
    /// cours se termine avec ses anciens réglages).
    fn restart_bar(self) -> Option<usize> {
        match self {
            Self::Session | Self::NewMelody | Self::SetSeed | Self::SeedThenBpm => Some(1),
            Self::SeekPlayhead | Self::Seek => Some(6),
            Self::SetStyleProfile | Self::SetBpm | Self::SetSwing => None,
        }
    }

    fn apply(self, host: &mut LiveHost) {
        match self {
            Self::Session => configure(host, "valse_quebecoise", 2, 180.0),
            Self::NewMelody => host.apply(EngineCommand::NewMelody).unwrap(),
            Self::SetSeed => host.apply(EngineCommand::SetSeed(9)).unwrap(),
            Self::SeedThenBpm => {
                host.apply(EngineCommand::SetSeed(9)).unwrap();
                host.apply(EngineCommand::SetBpm(150.0)).unwrap();
            }
            Self::SetStyleProfile => host
                .apply(EngineCommand::SetStyleProfile(Box::new(profile("valse_quebecoise"))))
                .unwrap(),
            Self::SetBpm => host.apply(EngineCommand::SetBpm(150.0)).unwrap(),
            Self::SetSwing => host.apply(EngineCommand::SetSwing(0.66)).unwrap(),
            Self::SeekPlayhead => host.apply(EngineCommand::SeekPlayhead(6)).unwrap(),
            Self::Seek => host.apply(EngineCommand::Seek(6)).unwrap(),
        }
    }

    /// Ce que la première mesure jouée après la commande doit porter.
    fn assert_applied(self, first: &MeasureSnapshot, label: &str) {
        match self {
            Self::Session => {
                assert_eq!((first.time_sig_numerator, first.tempo), (3, 180.0), "{label}");
            }
            Self::SetStyleProfile => assert_eq!(first.time_sig_numerator, 3, "{label}"),
            Self::SetBpm | Self::SeedThenBpm => assert_eq!(first.tempo, 150.0, "{label}"),
            Self::SetSwing => assert!((first.swing_ratio - 0.66).abs() < 1e-6, "{label}"),
            Self::NewMelody | Self::SetSeed | Self::SeekPlayhead | Self::Seek => {}
        }
    }
}

/// Position en ticks, depuis le début de la mesure `bar` de départ, de la
/// coordonnée `x` (en pas depuis le début de la mesure `bar`), grille de swing
/// audible comprise (même condition que `PlaybackEngine::compute_pulse_steps`).
fn swung_steps(measure: &MeasureSnapshot, x: f64) -> f64 {
    let steps_per_quarter = (measure.steps * measure.time_sig_denominator) as f64
        / (measure.time_sig_numerator * 4) as f64;
    let swung = measure.swing_ratio > 0.5
        && steps_per_quarter == 4.0
        && usize::from(measure.pulse.steps_per_pulse) % 4 == 0;
    if !swung {
        return x;
    }
    let r = f64::from(measure.swing_ratio);
    let quarter = (x / 4.0).floor();
    let local = x - quarter * 4.0;
    let fraction = if local < 2.0 { local / 2.0 * r } else { r + (local - 2.0) / 2.0 * (1.0 - r) };
    (quarter + fraction) * 4.0
}

struct Recording {
    /// `(piste, hauteur, tick)` des attaques des pistes 0 à 2.
    attacks: Vec<(u8, u8, f64)>,
    /// `(tick, bpm)` des méta-événements de tempo.
    tempos: Vec<(u32, f64)>,
}

fn parse(midi: &[u8]) -> Recording {
    let smf = Smf::parse(midi).expect("MIDI valide");
    let mut tick = 0u32;
    let mut recording = Recording { attacks: Vec::new(), tempos: Vec::new() };
    for event in &smf.tracks[0] {
        tick += event.delta.as_int();
        match event.kind {
            TrackEventKind::Midi { channel, message: MidiMessage::NoteOn { key, vel } }
                if vel.as_int() > 0 && channel.as_int() <= 2 =>
            {
                recording.attacks.push((channel.as_int(), key.as_int(), f64::from(tick)));
            }
            TrackEventKind::Meta(MetaMessage::Tempo(micros)) => {
                recording.tempos.push((tick, 60_000_000.0 / f64::from(micros.as_int())));
            }
            _ => {}
        }
    }
    recording
}

fn scenario(state: State, command: Command) {
    let mut rig = Rig::new();
    configure(&mut rig.host, "reel_quebecois", 1, 120.0);
    rig.pump();

    // Mesure et tick où la lecture reprend après la commande : `(mesure, décalage en ticks)`.
    // Une mesure déjà chargée finit avec ses anciens réglages ; `Fresh` n'en a chargé aucune.
    let (first_bar, first_tick) = match (state, command.restart_bar()) {
        (_, Some(bar)) => (bar, 0.0),
        (State::Fresh, None) => (1, 0.0),
        (State::Parked | State::Live, None) => {
            rig.play_until(COMMAND_BAR, COMMAND_STEP);
            let steps = rig.snapshots[&COMMAND_BAR].steps;
            (COMMAND_BAR + 1, (steps - COMMAND_STEP) as f64 * TICKS_PER_STEP)
        }
    };
    if matches!((state, command.restart_bar()), (State::Parked | State::Live, Some(_))) {
        rig.play_until(COMMAND_BAR, COMMAND_STEP);
    }

    command.apply(&mut rig.host);
    rig.host.apply(EngineCommand::StartRecording(RecordFormat::Midi)).unwrap();
    // Le pump de l'application tourne pendant que le flux est arrêté.
    let pumps = match state {
        State::Fresh => 1,
        State::Parked => 3,
        State::Live => 0,
    };
    for _ in 0..pumps {
        rig.pump();
    }
    let mut rendered = 0usize;
    loop {
        rig.render(64);
        rig.pump();
        rendered += 1;
        if rig.host.transport_position().bar > first_bar + CHECKED_BARS || rendered > 100_000 {
            break;
        }
    }
    assert!(rendered <= 100_000, "{state:?} {command:?} : les mesures suivantes n'arrivent pas");
    rig.host.apply(EngineCommand::StopRecording(RecordFormat::Midi)).unwrap();
    rig.render(64);
    let midi = rig
        .host
        .take_recordings()
        .into_iter()
        .find(|(format, _)| *format == RecordFormat::Midi)
        .expect("enregistrement MIDI")
        .1;
    let recording = parse(&midi);

    // Vérité : attaques des mesures `first_bar..` (la suivante apporte les attaques
    // anticipées de la dernière), placées sur la grille audible.
    let last_bar = first_bar + CHECKED_BARS;
    let mut start = first_tick;
    let mut starts = BTreeMap::new();
    for bar in first_bar..=last_bar {
        starts.insert(bar, start);
        start += rig.snapshots[&bar].steps as f64 * TICKS_PER_STEP;
    }
    let window_end = starts[&last_bar];
    let mut expected: Vec<(u8, u8, f64)> = Vec::new();
    for bar in first_bar..=last_bar {
        let measure = &rig.snapshots[&bar];
        for note in measure.notes.iter().filter(|note| note.track <= 2) {
            let x = note.start_step as f64 + f64::from(note.performance.offset) / 1000.0;
            let tick = starts[&bar] + swung_steps(measure, x) * TICKS_PER_STEP;
            // La mesure de reprise n'est pas rejouée avant son début.
            if tick >= first_tick && tick < window_end {
                expected.push((note.track, note.pitch, tick));
            }
        }
    }
    let mut heard: Vec<(u8, u8, f64)> = recording
        .attacks
        .iter()
        .copied()
        .filter(|(_, _, tick)| *tick >= first_tick && *tick < window_end)
        .collect();
    let by_key = |a: &(u8, u8, f64), b: &(u8, u8, f64)| {
        (a.0, a.1).cmp(&(b.0, b.1)).then(a.2.total_cmp(&b.2))
    };
    expected.sort_by(by_key);
    heard.sort_by(by_key);
    let label = format!("{state:?} {command:?}");
    let pitches = |notes: &[(u8, u8, f64)]| -> Vec<(u8, u8)> {
        notes.iter().map(|(track, pitch, _)| (*track, *pitch)).collect()
    };
    assert_eq!(pitches(&heard), pitches(&expected), "{label} : notes jouées ≠ snapshots");
    for (heard, expected) in heard.iter().zip(&expected) {
        assert!(
            (heard.2 - expected.2).abs() <= 2.0,
            "{label} : attaque {:?} à {} ticks, attendue à {}",
            (heard.0, heard.1),
            heard.2,
            expected.2
        );
    }

    // La première mesure jouée porte la commande (ce n'est pas seulement une mesure cohérente).
    command.assert_applied(&rig.snapshots[&first_bar], &label);

    // Tempo audible de chaque mesure vérifiée, au dernier pas : une rampe de tempo
    // entre mesures est finie à la dernière pulse.
    for bar in first_bar..last_bar {
        let probe = starts[&bar] + (rig.snapshots[&bar].steps - 1) as f64 * TICKS_PER_STEP;
        let sounding = recording
            .tempos
            .iter()
            .rev()
            .find(|(tick, _)| f64::from(*tick) <= probe)
            .map(|(_, bpm)| *bpm);
        let wanted = f64::from(rig.snapshots[&bar].tempo);
        assert!(
            sounding.is_some_and(|bpm| (bpm - wanted).abs() < 0.5),
            "{label} : mesure {bar} jouée à {sounding:?} bpm, snapshot à {wanted}"
        );
    }
}

macro_rules! cases {
    ($($name:ident: $state:ident, $command:ident;)*) => {
        $(
            #[test]
            fn $name() {
                scenario(State::$state, Command::$command);
            }
        )*
    };
}

cases! {
    fresh_session: Fresh, Session;
    fresh_new_melody: Fresh, NewMelody;
    fresh_set_seed: Fresh, SetSeed;
    fresh_seed_then_bpm: Fresh, SeedThenBpm;
    fresh_set_style_profile: Fresh, SetStyleProfile;
    fresh_set_bpm: Fresh, SetBpm;
    fresh_set_swing: Fresh, SetSwing;
    fresh_seek_playhead: Fresh, SeekPlayhead;
    fresh_seek: Fresh, Seek;
    parked_session: Parked, Session;
    parked_new_melody: Parked, NewMelody;
    parked_set_seed: Parked, SetSeed;
    parked_seed_then_bpm: Parked, SeedThenBpm;
    parked_set_style_profile: Parked, SetStyleProfile;
    parked_set_bpm: Parked, SetBpm;
    parked_set_swing: Parked, SetSwing;
    parked_seek_playhead: Parked, SeekPlayhead;
    parked_seek: Parked, Seek;
    live_session: Live, Session;
    live_new_melody: Live, NewMelody;
    live_set_seed: Live, SetSeed;
    live_seed_then_bpm: Live, SeedThenBpm;
    live_set_style_profile: Live, SetStyleProfile;
    live_set_bpm: Live, SetBpm;
    live_set_swing: Live, SetSwing;
    live_seek_playhead: Live, SeekPlayhead;
    live_seek: Live, Seek;
}

/// Boîtes d'un epoch remplacé encore dans la file : même sans remplaçante (file
/// pleine côté compositeur), l'annonce de l'epoch suffit à ce qu'aucune ne joue.
#[test]
fn boxes_queued_before_a_new_epoch_never_play() {
    let (mut composer, mut audio, mut commands, _reports, _clock, finished) =
        create_offline_engine(None, None, &[], SAMPLE_RATE).expect("moteur hors ligne");
    composer.set_seed(1);
    composer.set_writehead_lookahead(8);
    composer.generate_ahead();
    // Séance suivante : la file garde les mesures de la graine 1, rien ne les remplace.
    composer.set_seed(2);
    let (bar, epoch) = composer.take_pending_drop().expect("éviction demandée");
    commands.push(PlaybackCommand::DropFrom { bar, epoch }).unwrap();
    commands.push(PlaybackCommand::Seek(1)).unwrap();
    commands.push(PlaybackCommand::StartRecording(AudioRecordFormat::Midi)).unwrap();
    let mut buffer = [0.0; 128];
    for _ in 0..2000 {
        audio.process_buffer(&mut buffer, 2, Instant::now());
    }
    commands.push(PlaybackCommand::StopRecording(AudioRecordFormat::Midi)).unwrap();
    audio.process_buffer(&mut buffer, 2, Instant::now());
    let midi = finished
        .lock()
        .unwrap()
        .iter()
        .find(|(format, _)| *format == RecordFormat::Midi)
        .expect("enregistrement MIDI")
        .1
        .clone();
    assert!(parse(&midi).attacks.is_empty(), "une mesure de l'ancien epoch a joué");
}

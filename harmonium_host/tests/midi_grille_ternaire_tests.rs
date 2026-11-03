//! A6 : le MIDI enregistré (`--record-midi`) d'une grille de 12 pas garde 480 ticks par noire, et le
//! swing de chaque piste y suit sa courbe selon le tempo.
//!
//! Le moteur hors ligne est piloté comme `harmonium_cli::export` : profil externe (fixtures
//! `grille_shuffle_ternaire` en 4/4 sur 12 pas et `grille_valse_jazz` en 3/4), tempo imposé par
//! `set_bpm`, enregistrement MIDI. Le rapport de swing d'une piste est la fraction de la noire où
//! tombent ses attaques de contretemps (même définition que `references/jtd-swing-tempo`), lue
//! dans les ticks du fichier ; la courbe attendue est celle du profil (JTD, batterie, basse, piano).

use std::time::Instant;

use harmonium::{audio::create_offline_engine, playback::PlaybackCommand};
use harmonium_core::{
    StyleProfile,
    events::{AudioRecordFormat, RecordFormat},
};
use midly::{MidiMessage, Smf, TrackEventKind};

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;
const BUFFER_SIZE: usize = 256;
const BARS: usize = 36;
const TICKS_PER_QUARTER: f64 = 480.0;
const RIDE: u8 = 51;
const CHORDS: u8 = 1;
const BASS: u8 = 2;
const DRUMS: u8 = 9;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
/// Attaque du fichier : `(tick, canal, note)`.
type Attack = (u32, u8, u8);

/// Profil de la fixture, sans micro-timing de voix (le rapport mesuré est celui du swing seul).
fn profile(name: &str) -> TestResult<StyleProfile> {
    let mut loaded = StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    loaded.performance = Default::default();
    Ok(loaded)
}

/// MIDI enregistré : `(ticks par noire, attaques)`.
fn record(profile: StyleProfile, bpm: f32, swing: Option<f32>) -> TestResult<(u16, Vec<Attack>)> {
    let (mut composer, mut playback, mut cmd_tx, mut report_rx, _clock, finished) =
        create_offline_engine(None, None, &[], SAMPLE_RATE)?;
    composer.set_seed(4);
    composer.set_profile(profile)?;
    composer.set_bpm(bpm);
    if let Some(ratio) = swing {
        composer.set_swing_ratio(ratio);
    }
    composer.invalidate_future();
    if let Some((bar, epoch)) = composer.take_pending_drop() {
        let _ = cmd_tx.push(PlaybackCommand::DropFrom { bar, epoch });
    }
    for (channel, program) in composer.take_pending_program_changes() {
        let _ = cmd_tx.push(PlaybackCommand::ProgramChange { channel, program });
    }
    let _ = cmd_tx
        .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()));
    composer.set_writehead_lookahead(BARS + 16);
    composer.generate_bars(BARS.max(32));
    let _ = cmd_tx.push(PlaybackCommand::StartRecording(AudioRecordFormat::Midi));
    let mut buffer = vec![0.0f32; BUFFER_SIZE * CHANNELS];
    while composer.playhead_bar() <= BARS {
        composer.generate_ahead();
        buffer.fill(0.0);
        playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
        while report_rx.pop().is_ok() {}
        let _ = composer.take_snapshots();
    }
    let _ = cmd_tx.push(PlaybackCommand::StopRecording(AudioRecordFormat::Midi));
    buffer.fill(0.0);
    playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
    let queue = finished.lock().map_err(|e| e.to_string())?;
    let midi = queue
        .iter()
        .find(|(format, _)| *format == RecordFormat::Midi)
        .map(|(_, data)| data.clone())
        .ok_or("un enregistrement MIDI")?;
    let smf = Smf::parse(&midi)?;
    let ppq = match smf.header.timing {
        midly::Timing::Metrical(ticks) => ticks.as_int(),
        midly::Timing::Timecode(..) => return Err("timing métrique attendu".into()),
    };
    let mut attacks = Vec::new();
    for track in &smf.tracks {
        let mut tick = 0u32;
        for event in track {
            tick += event.delta.as_int();
            if let TrackEventKind::Midi { channel, message: MidiMessage::NoteOn { key, vel } } =
                event.kind
                && vel.as_int() > 0
            {
                attacks.push((tick, channel.as_int(), key.as_int()));
            }
        }
    }
    Ok((ppq, attacks))
}

/// Médiane des fractions de noire des attaques de contretemps (entre 0,45 et 0,85, comme JTD).
fn swing(attacks: &[Attack], channel: u8, key: Option<u8>) -> TestResult<(f64, usize)> {
    let mut ratios: Vec<f64> = attacks
        .iter()
        .filter(|(_, c, k)| *c == channel && key.is_none_or(|key| *k == key))
        .map(|(tick, ..)| f64::from(*tick % 480) / TICKS_PER_QUARTER)
        .filter(|fraction| (0.45..0.85).contains(fraction))
        .collect();
    ratios.sort_by(f64::total_cmp);
    let n = ratios.len();
    Ok((*ratios.get(n / 2).ok_or("aucun contretemps")?, n))
}

const TEMPOS: [(f32, f64, f64, f64); 4] = [
    // (tempo, batterie, basse, piano) : JTD `a6_table.curve`.
    (110.0, 0.7213, 0.7148, 0.7021),
    (150.0, 0.712, 0.706, 0.702),
    (230.0, 0.633, 0.671, 0.644),
    (270.0, 0.595, 0.653, 0.615),
];

#[test]
fn le_midi_d_une_grille_de_12_pas_suit_la_courbe_de_swing_de_chaque_piste() -> TestResult {
    for (bpm, drums, bass, piano) in TEMPOS {
        let (ppq, attacks) = record(profile("grille_shuffle_ternaire")?, bpm, None)?;
        assert_eq!(ppq, 480);
        // Une mesure de 4/4 fait 1920 ticks quelle que soit la grille (120 par pas sur 16 pas,
        // 160 sur 12 pas) : la dernière attaque tombe avant la fin de la 36e mesure.
        let last = attacks.iter().map(|(tick, ..)| *tick).max().ok_or("attaques")?;
        assert!(last < (BARS as u32) * 1920 + 1920, "{bpm} BPM : dernière attaque au tick {last}");
        assert!(last > (BARS as u32 - 3) * 1920, "{bpm} BPM : dernière attaque au tick {last}");
        for (name, channel, key, expected) in [
            ("batterie", DRUMS, Some(RIDE), drums),
            ("basse", BASS, None, bass),
            ("comping", CHORDS, None, piano),
        ] {
            let (measured, n) = swing(&attacks, channel, key)?;
            assert!(n >= 20, "{name} à {bpm} BPM : {n} contretemps");
            assert!(
                (measured - expected).abs() < 0.006,
                "{name} à {bpm} BPM : {measured:.4} (JTD {expected:.4}, {n} attaques)"
            );
        }
    }
    Ok(())
}

#[test]
fn le_midi_du_jazz_waltz_compte_1440_ticks_par_mesure() -> TestResult {
    let (bpm, drums, _, piano) = TEMPOS[1];
    let (ppq, attacks) = record(profile("grille_valse_jazz")?, bpm, None)?;
    assert_eq!(ppq, 480);
    let last = attacks.iter().map(|(tick, ..)| *tick).max().ok_or("attaques")?;
    assert!(last < (BARS as u32) * 1440 + 1440 && last > (BARS as u32 - 3) * 1440, "tick {last}");
    // Ride : sur les temps (multiples de 480) ou à la croche swinguée ; jamais ailleurs.
    let ride: Vec<u32> = attacks
        .iter()
        .filter(|(_, c, k)| *c == DRUMS && *k == RIDE)
        .map(|(t, ..)| t % 480)
        .collect();
    let swung = (drums * 480.0).round() as i64;
    assert!(
        ride.iter().all(|&t| t <= 2 || (i64::from(t) - swung).abs() <= 2 || t >= 478),
        "attaques du ride hors grille : {ride:?}"
    );
    let (ride_swing, _) = swing(&attacks, DRUMS, Some(RIDE))?;
    let (comping_swing, _) = swing(&attacks, CHORDS, None)?;
    assert!((ride_swing - drums).abs() < 0.006, "ride {ride_swing:.4}");
    assert!((comping_swing - piano).abs() < 0.006, "comping {comping_swing:.4}");
    Ok(())
}

#[test]
fn une_commande_directe_de_swing_donne_le_meme_rapport_a_toutes_les_pistes() -> TestResult {
    let (_, attacks) = record(profile("grille_shuffle_ternaire")?, 230.0, Some(0.6667))?;
    for (channel, key) in [(DRUMS, Some(RIDE)), (BASS, None), (CHORDS, None)] {
        let (measured, n) = swing(&attacks, channel, key)?;
        assert!(n >= 20);
        assert!((measured - 2.0 / 3.0).abs() < 0.006, "canal {channel} : {measured:.4}");
    }
    Ok(())
}

#[test]
fn set_swing_ratio_marque_la_commande_directe_et_le_profil_la_retire() -> TestResult {
    let (mut composer, ..) = create_offline_engine(None, None, &[], SAMPLE_RATE)?;
    composer.set_profile(profile("grille_shuffle_ternaire")?)?;
    assert!(!composer.musical_params().swing_explicit);
    composer.set_swing_ratio(0.6);
    assert!(composer.musical_params().swing_explicit);
    composer.set_profile(profile("grille_valse_jazz")?)?;
    assert!(!composer.musical_params().swing_explicit, "un nouveau profil rend le swing au profil");
    composer.set_swing_ratio(0.6);
    composer.reset();
    assert!(!composer.musical_params().swing_explicit, "reset rend le swing au profil");
    Ok(())
}

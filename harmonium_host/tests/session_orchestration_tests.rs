//! A2 : l'habillage par graine et l'orchestration par chorus côté hôte.
//!
//! La fixture `grille_habillage` (blues de douze mesures, quatre chorus) tire une variante par
//! groupe ; un de ses arcs change les programmes à chaque frontière de chorus. Le moteur hors ligne
//! est piloté comme `harmonium_cli::export` : le MIDI enregistré porte les changements de programme
//! des snapshots, à la barre du chorus ; la commande de l'hôte (tempo, swing) l'emporte ; le seek
//! rejoue la même variante et les mêmes programmes.

use std::{
    sync::{Arc, atomic::AtomicU64},
    time::Instant,
};

use harmonium::{
    audio::create_offline_engine,
    composer::MusicComposer,
    playback::{MEASURE_RING_CAP, PlaybackCommand, RETURN_RING_CAP, TaggedMeasure},
};
use harmonium_core::{
    StyleProfile, TuningParams,
    compose::TuneGenerator,
    events::{AudioRecordFormat, RecordFormat},
    params::MusicalParams,
    report::MeasureSnapshot,
    session::session_rng,
};
use midly::{MidiMessage, Smf, TrackEventKind};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SAMPLE_RATE: f64 = 44100.0;
const CHANNELS: usize = 2;
const BUFFER_SIZE: usize = 256;
const BLUES: usize = 12;
const BARS: usize = 48;
const TICKS_PER_BAR: u64 = 16 * 120;

fn fixture() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/grille_habillage",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

/// Graine de session (même RNG que le compositeur) dont l'arc tiré est « orchestre », avec un
/// comping qui n'est pas déjà à l'orgue et un kit aux baguettes : chaque changement de programme
/// de l'arc s'entend.
fn orchestrated_seed(profile: &StyleProfile) -> TestResult<u64> {
    for seed in 1..=256 {
        let params = MusicalParams {
            bpm: profile.meter.default_bpm,
            time_signature: profile.meter.time_signature,
            harmony_valence: profile.dynamics.neutral_valence,
            ..Default::default()
        };
        let mut generator = TuneGenerator::new(params, TuningParams { profile: profile.clone() });
        let mut rng = session_rng(seed);
        let first = MeasureSnapshot::from_measure(&generator.generate_measure(1, &mut rng));
        let named =
            |group: &str| first.session.iter().find(|v| v.group == group).map(|v| v.name.as_str());
        if named("arc") == Some("orchestre")
            && named("comping") != Some("orgue")
            && named("kit") == Some("baguettes")
        {
            return Ok(seed);
        }
    }
    Err("aucune graine orchestrée parmi 256".into())
}

fn composer_with(seed: u64, profile: StyleProfile) -> TestResult<MusicComposer> {
    let (measure_tx, _measure_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(MEASURE_RING_CAP);
    let (_return_tx, return_rx) = rtrb::RingBuffer::<TaggedMeasure>::new(RETURN_RING_CAP);
    let transport = Arc::new(AtomicU64::new(harmonium::playback::pack(1, 0)));
    let mut composer =
        MusicComposer::new_with_seed(SAMPLE_RATE, measure_tx, return_rx, transport, seed);
    composer.set_profile(profile)?;
    Ok(composer)
}

fn composer(seed: u64) -> TestResult<MusicComposer> {
    composer_with(seed, fixture()?)
}

fn json(snapshots: &[MeasureSnapshot]) -> Vec<String> {
    snapshots.iter().map(|s| serde_json::to_string(s).unwrap_or_default()).collect()
}

#[test]
fn le_seek_rejoue_la_meme_variante_et_les_memes_programmes() -> TestResult {
    let seed = orchestrated_seed(&fixture()?)?;
    let mut linear = composer(seed)?;
    linear.set_writehead_lookahead(BARS + 16);
    linear.generate_bars(BARS);
    let linear = linear.take_snapshots();
    assert_eq!(linear.len(), BARS);
    assert!(linear.iter().all(|s| s.instruments.is_some() && s.session.len() == 9));

    let mut seeked = composer(seed)?;
    seeked.set_writehead_lookahead(BARS + 16);
    seeked.generate_bars(4);
    let _ = seeked.take_snapshots();
    seeked.deterministic_seek(2 * BLUES + 1);
    seeked.generate_bars(BARS);
    let replay = seeked.take_snapshots();
    assert_eq!(
        json(&linear[2 * BLUES..]),
        json(&replay[..BARS - 2 * BLUES]),
        "du chorus 3 (l'orgue) au dernier : mêmes snapshots, programmes compris"
    );
    // Rejeu depuis la mesure 1 avec la même graine : même variante.
    let mut again = composer(seed)?;
    again.set_writehead_lookahead(BARS + 16);
    again.generate_bars(BARS);
    assert_eq!(json(&linear), json(&again.take_snapshots()));
    assert_eq!(
        again.session_variants().iter().map(|v| v.name.clone()).collect::<Vec<_>>(),
        linear[0].session.iter().map(|v| v.name.clone()).collect::<Vec<_>>(),
        "la variante tirée est lisible sur le compositeur"
    );
    Ok(())
}

#[test]
fn le_tempo_et_le_swing_de_l_hote_gagnent_et_la_variante_reste_visible() -> TestResult {
    let seed = orchestrated_seed(&fixture()?)?;
    let mut free = composer(seed)?;
    free.set_writehead_lookahead(BARS);
    free.generate_bars(BLUES);
    let free = free.take_snapshots();
    let drawn_bpm = free[0].tempo;
    assert!((100.0..=196.0).contains(&drawn_bpm) && drawn_bpm != 133.0);
    assert_eq!(free[0].composition_bpm, drawn_bpm, "tempo de composition : celui de la variante");

    let mut host = composer(seed)?;
    host.set_bpm(133.0);
    host.set_swing_ratio(0.55);
    host.invalidate_future();
    host.set_writehead_lookahead(BARS);
    host.generate_bars(BLUES);
    let imposed = host.take_snapshots();
    assert!(imposed.iter().all(|s| s.tempo == 133.0), "SetBpm gagne");
    assert!(imposed.iter().all(|s| (s.swing_ratio - 0.55).abs() < 1e-6), "SetSwing gagne");
    assert_eq!(imposed[0].composition_bpm, drawn_bpm, "la variante n'en reste pas moins tirée");
    assert_eq!(imposed[0].session, free[0].session);
    assert_eq!(
        imposed[0].instruments, free[0].instruments,
        "les programmes de la variante tiennent"
    );
    Ok(())
}

/// Changements de programme du MIDI : `(tick, canal, programme)`, triés.
fn program_changes(midi: &[u8]) -> TestResult<Vec<(u64, u8, u8)>> {
    let smf = Smf::parse(midi)?;
    let mut changes = Vec::new();
    for track in &smf.tracks {
        let mut tick = 0u64;
        for event in track {
            tick += u64::from(u32::from(event.delta));
            if let TrackEventKind::Midi {
                channel,
                message: MidiMessage::ProgramChange { program },
            } = event.kind
            {
                changes.push((tick, u8::from(channel), u8::from(program)));
            }
        }
    }
    changes.sort_unstable();
    Ok(changes)
}

/// Rend `bars` mesures du moteur hors ligne en enregistrant le MIDI ; `from` : mesure où la tête
/// de lecture démarre (seek avant l'enregistrement).
fn render(seed: u64, from: usize, bars: usize) -> TestResult<(Vec<u8>, Vec<MeasureSnapshot>)> {
    let (mut composer, mut playback, mut cmd_tx, mut report_rx, _clock, finished) =
        create_offline_engine(None, None, &[], SAMPLE_RATE)?;
    composer.set_seed(seed);
    composer.set_profile(fixture()?)?;
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
    if from > 1 {
        composer.seek(from);
        if let Some((bar, epoch)) = composer.take_pending_drop() {
            let _ = cmd_tx.push(PlaybackCommand::DropFrom { bar, epoch });
        }
        let _ = cmd_tx.push(PlaybackCommand::Seek(from));
    }
    composer.generate_bars(BARS);
    let mut snapshots = composer.take_snapshots();
    let _ = cmd_tx.push(PlaybackCommand::StartRecording(AudioRecordFormat::Midi));
    let mut buffer = vec![0.0f32; BUFFER_SIZE * CHANNELS];
    while composer.playhead_bar() < from + bars {
        composer.generate_ahead();
        snapshots.extend(composer.take_snapshots());
        buffer.fill(0.0);
        playback.process_buffer(&mut buffer, CHANNELS, Instant::now());
        while report_rx.pop().is_ok() {}
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
    Ok((midi, snapshots))
}

/// Tick du début de la mesure `bar` d'un MIDI enregistré dont la première mesure est `from`.
fn bar_tick(bar: usize, from: usize) -> u64 {
    (bar - from) as u64 * TICKS_PER_BAR
}

fn near(a: u64, b: u64) -> bool {
    a.abs_diff(b) <= 2
}

#[test]
fn le_midi_enregistre_porte_les_changements_de_programme_a_la_barre_du_chorus() -> TestResult {
    let seed = orchestrated_seed(&fixture()?)?;
    let (midi, snapshots) = render(seed, 1, BARS)?;
    let changes = program_changes(&midi)?;
    let by_bar = |bar: usize| snapshots.iter().find(|s| s.index == bar).and_then(|s| s.instruments);
    let session_kit =
        by_bar(BLUES + 1).map(|i| i.percussion_program).ok_or("programmes du chorus 2")?;
    assert_eq!(session_kit, 32, "baguettes");
    // Kit : balais au chorus 1 (tête de piste), baguettes au chorus 2, balais à la retombée.
    let kit: Vec<(u64, u8)> =
        changes.iter().filter(|(_, ch, _)| *ch == 9).map(|(t, _, p)| (*t, *p)).collect();
    assert!(near(kit[0].0, 0) && kit[0].1 == 40, "chorus 1 aux balais : {kit:?}");
    assert!(
        kit.iter().any(|&(t, p)| near(t, bar_tick(BLUES + 1, 1)) && p == 32),
        "baguettes à la mesure 13 : {kit:?}"
    );
    assert!(
        kit.iter().any(|&(t, p)| near(t, bar_tick(3 * BLUES + 1, 1)) && p == 40),
        "balais à la mesure 37 : {kit:?}"
    );
    assert_eq!(kit.len(), 3, "trois changements de kit sur quatre chorus : {kit:?}");
    // Comping : l'orgue entre au chorus 3 (mesure 25) et en sort au chorus 4.
    let chords: Vec<(u64, u8)> =
        changes.iter().filter(|(_, ch, _)| *ch == 1).map(|(t, _, p)| (*t, *p)).collect();
    assert!(
        chords.iter().any(|&(t, p)| near(t, bar_tick(2 * BLUES + 1, 1)) && p == 16),
        "l'orgue entre à la mesure 25 : {chords:?}"
    );
    let back = by_bar(3 * BLUES + 1).and_then(|i| i.chords_program).ok_or("comping du chorus 4")?;
    assert!(
        chords.iter().any(|&(t, p)| near(t, bar_tick(3 * BLUES + 1, 1)) && p == back),
        "le comping de la session revient à la mesure 37 : {chords:?}"
    );
    Ok(())
}

#[test]
fn apres_un_seek_le_playback_annonce_les_programmes_de_la_mesure() -> TestResult {
    let seed = orchestrated_seed(&fixture()?)?;
    // Tête de lecture au chorus 3 : le MIDI enregistré commence par l'orgue, sans avoir joué
    // les chorus précédents ; puis au chorus 4 : balais et comping de la session.
    let (midi, _) = render(seed, 2 * BLUES + 1, 4)?;
    let changes = program_changes(&midi)?;
    let organ = changes.iter().rfind(|(t, ch, _)| *ch == 1 && near(*t, 0)).map(|(_, _, p)| *p);
    assert_eq!(organ, Some(16), "l'orgue sonne dès la mesure 25 : {changes:?}");
    let (midi, snapshots) = render(seed, 3 * BLUES + 1, 4)?;
    let changes = program_changes(&midi)?;
    let instruments = snapshots
        .iter()
        .find(|s| s.index == 3 * BLUES + 1)
        .and_then(|s| s.instruments)
        .ok_or("programmes de la mesure 37")?;
    let last_at_start = |channel: u8| {
        changes.iter().rfind(|(t, ch, _)| *ch == channel && near(*t, 0)).map(|(_, _, p)| *p)
    };
    assert_eq!(instruments.percussion_program, 40);
    assert_eq!(last_at_start(9), Some(40), "balais dès la mesure 37 : {changes:?}");
    assert_eq!(last_at_start(1), instruments.chords_program, "{changes:?}");
    assert_ne!(last_at_start(1), Some(16), "l'orgue du chorus 3 n'est pas rejoué : {changes:?}");
    Ok(())
}

/// Profil de la fixture avec un groupe `synth` : deux variantes qui remplacent les zones Odin de la
/// basse par un preset ou par l'autre (`a` et `b`).
fn with_odin_variants() -> TestResult<StyleProfile> {
    use harmonium_core::{
        compose::SessionVariant,
        odin::{OdinTrackSpec, OdinZone},
        timeline::TrackId,
    };
    let mut profile = fixture()?;
    for (name, preset) in [("a", "synth_a"), ("b", "synth_b")] {
        let spec = OdinTrackSpec {
            zones: vec![OdinZone {
                when: None,
                preset: preset.to_owned(),
                level_db: 0.0,
                glide_ms: 80.0,
                transpose: 0,
                morph: Vec::new(),
            }],
        };
        profile.session.variants.push(SessionVariant {
            group: "synth".to_owned(),
            name: name.to_owned(),
            odin: [(TrackId::Bass, spec)].into(),
            ..Default::default()
        });
    }
    Ok(profile)
}

#[test]
fn le_preset_odin_de_la_variante_remplace_celui_du_profil() -> TestResult {
    use harmonium_core::{
        host_resources::{HostResources, MissingResource},
        odin::{OdinBank, OdinMorphParam, OdinPresetBase, OdinVoiceParams},
        timeline::TrackId,
    };
    let preset = |name: &str| OdinPresetBase {
        name: name.to_owned(),
        base: OdinVoiceParams { values: [0.0; OdinMorphParam::COUNT] },
    };
    let bass = usize::from(TrackId::Bass.channel());
    let mut chosen = std::collections::BTreeSet::new();
    for seed in 1..=16 {
        let mut composer = composer_with(seed, with_odin_variants()?)?;
        // Le preset `synth_b` n'est pas fourni par l'hôte.
        composer.set_host_resources(HostResources {
            percussion_banks: vec![128],
            odin: OdinBank::new(vec![preset("synth_a")]),
        });
        assert_eq!(
            composer.missing_resources(),
            [MissingResource::OdinPreset { track: TrackId::Bass, preset: "synth_b".to_owned() }],
            "le preset d'une variante absente de l'hôte est signalé dès le chargement"
        );
        assert!(composer.odin_patches()[bass].is_none(), "avant le tirage : basse SoundFont");
        composer.set_writehead_lookahead(4);
        composer.generate_bars(1);
        let name = composer
            .session_variants()
            .iter()
            .find(|v| v.group == "synth")
            .map(|v| v.name.clone())
            .unwrap_or_default();
        let patch = composer.odin_patches()[bass];
        match name.as_str() {
            "a" => assert!(patch.is_some(), "graine {seed} : preset `synth_a` joué"),
            "b" => assert!(patch.is_none(), "graine {seed} : `synth_b` absent, basse SoundFont"),
            other => return Err(format!("graine {seed} : variante {other:?}").into()),
        }
        chosen.insert(name);
    }
    assert_eq!(chosen.len(), 2, "les deux variantes se tirent sur 16 graines");
    Ok(())
}

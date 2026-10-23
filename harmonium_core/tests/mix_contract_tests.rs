//! Les erreurs de chargement nomment chaque paramètre borné du mixage.
use harmonium_core::{StyleProfile, mix::ProfileMix};

fn rejects(field: &str, values: &[f32], change: impl Fn(&mut ProfileMix, f32)) {
    for &value in values {
        let mut profile = StyleProfile::builtin().clone();
        change(&mut profile.mix, value);
        let text = profile.to_toml().expect("profil sérialisable");
        let error = StyleProfile::from_toml(&text).expect_err("mix invalide refusé au chargement");
        assert_eq!(error.field, field);
        assert!(error.message.contains("attendue"), "{error}");
    }
}
macro_rules! bounded {
    ($test:ident, $path:literal, $($member:ident).+, [$($value:expr),+]) => {
        #[test]
        fn $test() {
            rejects($path, &[$($value),+, f32::NAN, f32::INFINITY], |mix, value| mix.$($member).+ = value);
        }
    };
}
bounded!(master_db, "mix.master_db", master_db, [-60.1, 48.1]);
bounded!(room_size, "mix.room.room_size", room.room_size, [-0.01, 1.01]);
bounded!(damping, "mix.room.damping", room.damping, [-0.01, 1.01]);
bounded!(width, "mix.room.width", room.width, [-0.01, 1.01]);
bounded!(room_level, "mix.room.level", room.level, [-0.01, 1.01]);
bounded!(track_level_db, "mix.tracks.Melody.level_db", tracks.melody.level_db, [0.1]);
bounded!(track_pan, "mix.tracks.Melody.pan", tracks.melody.pan, [-1.01, 1.01]);
bounded!(
    track_reverb_send,
    "mix.tracks.Melody.reverb_send",
    tracks.melody.reverb_send,
    [-0.01, 1.01]
);
bounded!(highpass_hz, "mix.eq.highpass_hz", eq.highpass_hz, [-0.01, 19.0, 1001.0]);
bounded!(presence_hz, "mix.eq.presence_hz", eq.presence_hz, [19.0, 20001.0]);
bounded!(presence_db, "mix.eq.presence_db", eq.presence_db, [-12.1, 12.1]);
bounded!(presence_q, "mix.eq.presence_q", eq.presence_q, [0.09, 10.1]);

#[test]
fn every_logical_track_is_validated() {
    for name in ["Melody", "Chords", "Bass", "Heel", "Toe"] {
        let profile = StyleProfile::builtin();
        let mut value: toml::Value = toml::from_str(&profile.to_toml().unwrap()).unwrap();
        for (field, invalid) in [("level_db", 1.0), ("pan", -2.0), ("reverb_send", 2.0)] {
            let original = value["mix"]["tracks"][name][field].clone();
            value["mix"]["tracks"][name][field] = toml::Value::Float(invalid);
            let error = StyleProfile::from_toml(&toml::to_string(&value).unwrap()).unwrap_err();
            assert_eq!(error.field, format!("mix.tracks.{name}.{field}"));
            value["mix"]["tracks"][name][field] = original;
        }
    }
}

#[test]
fn omitted_mix_is_dry_centered_unity_without_eq() {
    let mut value: toml::Value =
        toml::from_str(&StyleProfile::builtin().to_toml().unwrap()).unwrap();
    value.as_table_mut().unwrap().remove("mix");
    let profile = StyleProfile::from_toml(&toml::to_string(&value).unwrap()).unwrap();
    let mix = profile.mix.prepare(44100.0);
    assert_eq!(mix.master_gain, 1.0);
    assert_eq!(mix.room.level, 0.0);
    assert_eq!(mix.eq, [[1.0, 0.0, 0.0, 0.0, 0.0]; 2]);
    for track in mix.tracks {
        assert_eq!(track.controllers(), [(7, 127), (10, 64), (91, 0)]);
    }
}

#[test]
fn timeline_midi_writes_profile_controllers_before_instrument_notes() {
    use harmonium_core::{
        mix::TrackMix,
        params::TimeSignature,
        timeline::{Articulation, Measure, ScoreTimeline, TimelineNote, TrackId, timeline_to_midi},
    };
    use midly::{MidiMessage, Smf, TrackEventKind};
    let track = TrackMix { level_db: -12.0, pan: -1.0, reverb_send: 1.0 };
    let mut mix = ProfileMix::default();
    mix.tracks.melody = track;
    mix.tracks.chords = track;
    mix.tracks.bass = track;
    mix.tracks.heel = TrackMix { level_db: 0.0, pan: 0.0, reverb_send: 0.0 };
    mix.tracks.toe = track;
    let mut score = ScoreTimeline::new(1);
    let mut measure = Measure::new(1, TimeSignature::new(4, 4), 120.0, 16);
    measure.add_note(
        TrackId::Melody,
        TimelineNote {
            id: 1,
            pitch: 60,
            start_step: 0,
            duration_steps: 4,
            velocity: 80,
            articulation: Articulation::Normal,
        },
    );
    score.push_measure(measure);
    let bytes = timeline_to_midi(&score, &mix).unwrap();
    let smf = Smf::parse(&bytes).unwrap();
    for (instrument, expected_channel) in smf.tracks[1..].iter().zip([0, 1, 2, 9]) {
        let events: Vec<_> = instrument
            .iter()
            .filter_map(|event| match event.kind {
                TrackEventKind::Midi { channel, message } => {
                    Some((event.delta.as_int(), channel.as_int(), message))
                }
                _ => None,
            })
            .collect();
        for ((tick, channel, message), (cc, value)) in
            events.iter().zip([(7, 64), (10, 0), (91, 127)])
        {
            assert_eq!((*tick, *channel), (0, expected_channel));
            assert_eq!(
                *message,
                MidiMessage::Controller { controller: cc.into(), value: value.into() }
            );
        }
        assert!(events.len() >= 3, "les trois CC précèdent les notes, Toe gagne sur Heel en GM");
    }
}

#[test]
fn arbitrarily_quiet_finite_track_levels_load_as_midi_silence() {
    let mut profile = StyleProfile::builtin().clone();
    profile.mix.tracks.melody.level_db = -200.0;
    let decoded = StyleProfile::from_toml(&profile.to_toml().unwrap()).unwrap();
    assert_eq!(decoded.mix.prepare(44100.0).tracks[0].volume, 0);
}

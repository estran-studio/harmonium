//! Mixage de profil et préparation hors callback des contrôleurs et biquads.
use serde::{Deserialize, Serialize};

use crate::style_profile::ProfileError;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MixRoom {
    pub room_size: f32,
    pub damping: f32,
    pub width: f32,
    pub level: f32,
}
impl Default for MixRoom {
    fn default() -> Self {
        Self { room_size: 0.2, damping: 0.2, width: 0.5, level: 0.0 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackMix {
    pub level_db: f32,
    pub pan: f32,
    pub reverb_send: f32,
}
impl TrackMix {
    /// SF2 : atténuation CC7 = -40 log10(CC7/127) dB (modulateur concave 960 cB).
    pub fn midi(self) -> MidiTrackMix {
        MidiTrackMix {
            volume: (127.0 * 10.0_f32.powf(self.level_db / 40.0)).round() as u8,
            // Centre exactement 64 ; les deux extrêmes restent 0 et 127.
            pan: if self.pan <= 0.0 {
                (64.0 * (self.pan + 1.0)).round() as u8
            } else {
                (64.0 + 63.0 * self.pan).round() as u8
            },
            reverb: (127.0 * self.reverb_send).round() as u8,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MixTracks {
    #[serde(rename = "Melody")]
    pub melody: TrackMix,
    #[serde(rename = "Chords")]
    pub chords: TrackMix,
    #[serde(rename = "Bass")]
    pub bass: TrackMix,
    #[serde(rename = "Heel")]
    pub heel: TrackMix,
    #[serde(rename = "Toe")]
    pub toe: TrackMix,
    #[serde(rename = "Countermelody", default)]
    pub countermelody: TrackMix,
}
impl MixTracks {
    pub fn all(self) -> [TrackMix; crate::timeline::TrackId::ALL.len()] {
        crate::timeline::TrackId::ALL.map(|track| self.get(track))
    }

    pub const fn get(self, track: crate::timeline::TrackId) -> TrackMix {
        use crate::timeline::TrackId;
        match track {
            TrackId::Melody => self.melody,
            TrackId::Chords => self.chords,
            TrackId::Bass => self.bass,
            TrackId::Heel => self.heel,
            TrackId::Toe => self.toe,
            TrackId::Countermelody => self.countermelody,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MixEq {
    /// 0 : bypass ; sinon 20..1000 Hz, Butterworth du deuxième ordre.
    pub highpass_hz: f32,
    pub presence_hz: f32,
    pub presence_db: f32,
    pub presence_q: f32,
}
impl Default for MixEq {
    fn default() -> Self {
        Self { highpass_hz: 0.0, presence_hz: 3000.0, presence_db: 0.0, presence_q: 0.707 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileMix {
    pub master_db: f32,
    pub room: MixRoom,
    pub tracks: MixTracks,
    pub eq: MixEq,
}

/// Charge utile Copy : aucune trigonométrie ni allocation dans le callback.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreparedMix {
    pub room: MixRoom,
    pub tracks: [MidiTrackMix; crate::timeline::TrackId::ALL.len()],
    /// [b0, b1, b2, a1, a2], normalisés par a0.
    pub eq: [[f32; 5]; 2],
    pub master_gain: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiTrackMix {
    pub volume: u8,
    pub pan: u8,
    pub reverb: u8,
}
impl Default for MidiTrackMix {
    fn default() -> Self {
        Self { volume: 127, pan: 64, reverb: 0 }
    }
}
impl MidiTrackMix {
    pub const fn controllers(self) -> [(u8, u8); 3] {
        [(7, self.volume), (10, self.pan), (91, self.reverb)]
    }
}

impl ProfileMix {
    pub fn validate(self) -> Result<(), ProfileError> {
        fn range(field: &str, value: f32, min: f32, max: f32) -> Result<(), ProfileError> {
            if value.is_finite() && (min..=max).contains(&value) {
                Ok(())
            } else {
                Err(ProfileError::new(
                    field,
                    format!("valeur finie attendue dans {min}..={max}, reçu {value}"),
                ))
            }
        }
        range("mix.master_db", self.master_db, -60.0, 48.0)?;
        for (name, value) in [
            ("room_size", self.room.room_size),
            ("damping", self.room.damping),
            ("width", self.room.width),
            ("level", self.room.level),
        ] {
            range(&format!("mix.room.{name}"), value, 0.0, 1.0)?;
        }
        for (id, track) in crate::timeline::TrackId::ALL.into_iter().zip(self.tracks.all()) {
            let name = id.name();
            if !track.level_db.is_finite() || track.level_db > 0.0 {
                return Err(ProfileError::new(
                    format!("mix.tracks.{name}.level_db"),
                    format!("valeur finie attendue ≤ 0 dB, reçu {}", track.level_db),
                ));
            }
            range(&format!("mix.tracks.{name}.pan"), track.pan, -1.0, 1.0)?;
            range(&format!("mix.tracks.{name}.reverb_send"), track.reverb_send, 0.0, 1.0)?;
        }
        if self.eq.highpass_hz != 0.0 {
            range("mix.eq.highpass_hz", self.eq.highpass_hz, 20.0, 1000.0)?;
        }
        range("mix.eq.presence_hz", self.eq.presence_hz, 20.0, 20000.0)?;
        range("mix.eq.presence_db", self.eq.presence_db, -12.0, 12.0)?;
        range("mix.eq.presence_q", self.eq.presence_q, 0.1, 10.0)
    }

    pub fn prepare(self, sample_rate: f64) -> PreparedMix {
        // RBJ Audio EQ Cookbook ; fréquences bornées sous Nyquist pour les hôtes à bas taux.
        let identity = [1.0, 0.0, 0.0, 0.0, 0.0];
        let trig = |hz: f32| {
            let w = std::f64::consts::TAU * f64::from(hz).min(sample_rate * 0.45) / sample_rate;
            (w.sin(), w.cos())
        };
        let normalize = |b: [f64; 3], a: [f64; 3]| {
            [
                (b[0] / a[0]) as f32,
                (b[1] / a[0]) as f32,
                (b[2] / a[0]) as f32,
                (a[1] / a[0]) as f32,
                (a[2] / a[0]) as f32,
            ]
        };
        let highpass = if self.eq.highpass_hz == 0.0 {
            identity
        } else {
            let (s, c) = trig(self.eq.highpass_hz);
            let alpha = s / std::f64::consts::SQRT_2;
            normalize(
                [(1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0],
                [1.0 + alpha, -2.0 * c, 1.0 - alpha],
            )
        };
        let presence = if self.eq.presence_db == 0.0 {
            identity
        } else {
            let (s, c) = trig(self.eq.presence_hz);
            let alpha = s / (2.0 * f64::from(self.eq.presence_q));
            let a = 10.0_f64.powf(f64::from(self.eq.presence_db) / 40.0);
            normalize(
                [1.0 + alpha * a, -2.0 * c, 1.0 - alpha * a],
                [1.0 + alpha / a, -2.0 * c, 1.0 - alpha / a],
            )
        };
        PreparedMix {
            room: self.room,
            tracks: self.tracks.all().map(TrackMix::midi),
            eq: [highpass, presence],
            master_gain: 10.0_f32.powf(self.master_db / 20.0),
        }
    }
}

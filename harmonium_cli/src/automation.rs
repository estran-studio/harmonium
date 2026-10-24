//! Automation hors ligne : interpolation par axe, appliquée par l'API du jeu.
use anyhow::{bail, Context, Result};
use harmonium::composer::MusicComposer;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Automation {
    pub keys: Vec<Key>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub bar: usize,
    pub intensity: Option<f32>,
    pub arousal: Option<f32>,
    pub valence: Option<f32>,
    pub density: Option<f32>,
    pub tension: Option<f32>,
    pub bpm: Option<f32>,
    pub feet: Option<f32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Values {
    pub intensity: Option<f32>,
    pub arousal: Option<f32>,
    pub valence: Option<f32>,
    pub density: Option<f32>,
    pub tension: Option<f32>,
    pub bpm: Option<f32>,
    pub feet: Option<f32>,
}

impl Automation {
    pub fn load(path: &str) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("--automation {path}: lecture impossible"))?;
        Self::from_toml(&text).with_context(|| format!("--automation {path}"))
    }

    pub fn from_toml(text: &str) -> Result<Self> {
        let data: Self = toml::from_str(text).context("TOML d'automation invalide")?;
        if data.keys.is_empty() {
            bail!("automation: au moins une [[keys]] requise");
        }
        let mut previous = 0;
        for key in &data.keys {
            if key.bar <= previous {
                bail!(
                    "automation: bar={} doit être positive et strictement supérieure à {previous}",
                    key.bar
                );
            }
            previous = key.bar;
            let fields = [
                ("intensity", key.intensity, 0.0, 1.0),
                ("arousal", key.arousal, 0.0, 1.0),
                ("valence", key.valence, -1.0, 1.0),
                ("density", key.density, 0.0, 1.0),
                ("tension", key.tension, 0.0, 1.0),
                ("bpm", key.bpm, 40.0, 300.0),
                ("feet", key.feet, 0.0, 2.0),
            ];
            if fields.iter().all(|(_, v, _, _)| v.is_none()) {
                bail!("automation: bar={} ne contient aucun paramètre", key.bar);
            }
            for (name, value, min, max) in fields {
                if let Some(value) = value {
                    if !value.is_finite() || !(min..=max).contains(&value) {
                        bail!("automation: bar={} {name}={value}, attendu un nombre fini dans {min}..{max}",key.bar);
                    }
                }
            }
        }
        Ok(data)
    }

    fn interpolate(&self, bar: usize, field: impl Fn(&Key) -> Option<f32>) -> Option<f32> {
        let mut previous = None;
        for key in &self.keys {
            let Some(value) = field(key) else {
                continue;
            };
            if key.bar >= bar {
                return match previous {
                    None => Some(value),
                    Some((start, initial)) => Some(
                        ((bar - start) as f32 / (key.bar - start) as f32)
                            .mul_add(value - initial, initial),
                    ),
                };
            }
            previous = Some((key.bar, value));
        }
        previous.map(|(_, value)| value)
    }

    pub fn at_bar(&self, bar: usize) -> Values {
        Values {
            intensity: self.interpolate(bar, |k| k.intensity),
            arousal: self.interpolate(bar, |k| k.arousal),
            valence: self.interpolate(bar, |k| k.valence),
            density: self.interpolate(bar, |k| k.density),
            tension: self.interpolate(bar, |k| k.tension),
            bpm: self.interpolate(bar, |k| k.bpm),
            feet: self.interpolate(bar, |k| k.feet),
        }
    }

    /// Les axes absents conservent les valeurs initiales ; les commandes
    /// directes gagnent sur les émotions quand les deux courbes sont présentes.
    pub fn apply(&self, bar: usize, composer: &mut MusicComposer, initial_emotions: [f32; 4]) {
        let values = self.at_bar(bar);
        if values.arousal.is_some()
            || values.valence.is_some()
            || values.density.is_some()
            || values.tension.is_some()
        {
            composer.use_emotion_mode();
            composer.set_emotions(
                values.arousal.unwrap_or(initial_emotions[0]),
                values.valence.unwrap_or(initial_emotions[1]),
                values.density.unwrap_or(initial_emotions[2]),
                values.tension.unwrap_or(initial_emotions[3]),
            );
        }
        if let Some(value) = values.intensity {
            composer.set_intensity(value);
        }
        if let Some(value) = values.bpm {
            composer.set_bpm(value);
        }
        if let Some(value) = values.feet {
            composer.set_feet_density(value.round() as u8);
        }
        composer.invalidate_future();
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use harmonium::playback::pack;
    use harmonium_core::{report::MeasureSnapshot, trad::ReelArrangement, tuning::TuningParams};

    use super::*;

    #[test]
    fn invalid_files_explain_the_field_and_bar() {
        for (text, expected) in [
            ("keys=[]", "au moins une"),
            ("[[keys]]\nbar=0\nintensity=0.5", "bar=0"),
            ("[[keys]]\nbar=1\nintensity=nan", "intensity"),
            ("[[keys]]\nbar=1\nfeet=3", "feet"),
            ("[[keys]]\nbar=1\nintensite=0.2", "unknown field"),
            ("[[keys]]\nbar=1\nintensity=0.2\n[[keys]]\nbar=1\nintensity=0.9", "strictement"),
            ("[[keys]]\nbar=1", "aucun paramètre"),
        ] {
            let error = Automation::from_toml(text).unwrap_err();
            assert!(format!("{error:#}").contains(expected), "{error:#}");
        }
    }

    #[test]
    fn sparse_fields_interpolate_independently_and_hold_endpoints() {
        let a=Automation::from_toml("[[keys]]\nbar=1\nintensity=0.0\nfeet=0\n[[keys]]\nbar=5\nvalence=-1.0\n[[keys]]\nbar=9\nintensity=1.0\nfeet=2\nvalence=1.0").unwrap();
        assert_eq!(a.at_bar(5).intensity, Some(0.5));
        assert_eq!(a.at_bar(5).feet, Some(1.0));
        assert_eq!(a.at_bar(7).valence, Some(0.0));
        assert_eq!(a.at_bar(1).valence, Some(-1.0));
        assert_eq!(a.at_bar(100).intensity, Some(1.0));
    }

    fn stream(text: &str, seed: u64, ornaments: bool) -> Result<Vec<MeasureSnapshot>> {
        let automation = Automation::from_toml(text)?;
        // Le transport factice ne remplace pas le générateur : il avance la
        // même position atomique que le playback pour tester son contrat pur.
        let transport = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(pack(1, 0)));
        let (measure_tx, _measure_rx) = rtrb::RingBuffer::new(256);
        let (_return_tx, return_rx) = rtrb::RingBuffer::new(256);
        let font_queue = Default::default();
        let mut composer = MusicComposer::new_with_seed(
            44100.0,
            measure_tx,
            return_rx,
            transport.clone(),
            font_queue,
            seed,
        );

        composer.set_tuning(TuningParams {
            profile: {
                let mut p = TuningParams::default().profile;
                p.arrangement = ReelArrangement::from_start(3, Default::default());
                if !ornaments {
                    // Seule la note de passage de densité existait avant V3b.
                    p.ornaments.types.retain(|kind, _| {
                        *kind == harmonium_core::ornaments::OrnamentKind::Passing
                    });
                }
                p
            },
            ..Default::default()
        });
        let mut snapshots = Vec::new();
        for bar in 1usize..=64 {
            transport
                .store(pack(bar.saturating_sub(1).max(1), 0), std::sync::atomic::Ordering::Relaxed);
            automation.apply(bar, &mut composer, [0.5, 0.3, 0.5, 0.3]);
            composer.generate_bars(if bar == 1 { 1 } else { 2 });
            snapshots.extend(composer.take_snapshots());
        }
        Ok(snapshots)
    }

    #[test]
    fn same_seed_and_file_produce_identical_snapshots_and_keep_repeats() -> Result<()> {
        let text = include_str!("../../demos/automation_monte.toml");
        // Les ornements tirés sont déterministes ; ils suivent la durée jouée, qui varie
        // avec les émotions : l'identité exacte des reprises se vérifie sans eux.
        let ornamented = stream(text, 1, true)?;
        assert_eq!(
            serde_json::to_string(&ornamented)?,
            serde_json::to_string(&stream(text, 1, true)?)?
        );
        let first = stream(text, 1, false)?;
        let second = stream(text, 1, false)?;
        assert_eq!(serde_json::to_string(&first)?, serde_json::to_string(&second)?);
        let pitches = |bar: usize| {
            first
                .iter()
                .find(|s| s.index == bar)
                .ok_or_else(|| anyhow!("mesure {bar} absente"))
                .map(|s| {
                    s.notes
                        .iter()
                        .filter(|n| n.track == 0)
                        .map(|n| (n.start_step, n.pitch))
                        .collect::<Vec<_>>()
                })
        };
        for start in [1, 17, 33, 49] {
            for offset in 0..7 {
                assert_eq!(pitches(start + offset)?, pitches(start + offset + 8)?);
            }
        }
        // Une volta diffère entre passes, mais reste la même au tour suivant.
        for bar in [8usize, 16, 24, 32] {
            assert_eq!(pitches(bar)?, pitches(bar + 32)?);
        }
        Ok(())
    }

    /// Contrat courant et preuve D2 : les rôles fixes conservent l'ancien flux.
    #[test]
    fn profile_d2_main_note_reference() -> Result<()> {
        profile_note_reference(false)?;
        profile_note_reference(true)
    }

    fn profile_note_reference(fixed_roles: bool) -> Result<()> {
        let automation = Automation::from_toml(include_str!("../../demos/automation_combat.toml"))?;
        let mut runs = Vec::new();
        let mut count = 0;
        for automated in [false, true] {
            for seed in 1..=5 {
                let transport = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(pack(1, 0)));
                let (measure_tx, _measure_rx) = rtrb::RingBuffer::new(256);
                let (_return_tx, return_rx) = rtrb::RingBuffer::new(256);
                let mut composer = MusicComposer::new_with_seed(
                    44100.0,
                    measure_tx,
                    return_rx,
                    transport.clone(),
                    Default::default(),
                    seed,
                );

                if fixed_roles {
                    use harmonium_core::trad::ImposedRole::{CadFull, CadHalf, CopyOf, Free, Open};
                    let mut tuning = harmonium_core::TuningParams::default();
                    tuning.profile.performance = Default::default();
                    // Seule la note de passage de densité existait sur main.
                    tuning.profile.ornaments.types.retain(|kind, _| {
                        *kind == harmonium_core::ornaments::OrnamentKind::Passing
                    });
                    for part in &mut tuning.profile.form.parts {
                        part.second_ending = None;
                        part.rhythm.clear();
                        part.roles =
                            [Open, Free, Free, CadHalf, CopyOf(1), CopyOf(2), Free, CadFull]
                                .map(Into::into)
                                .to_vec();
                    }
                    composer.set_tuning(tuning);
                }
                composer.use_emotion_mode();
                composer.set_emotions(0.5, 0.0, 0.5, 0.0);
                let mut bars = Vec::new();
                for bar in 1usize..=100 {
                    transport.store(
                        pack(bar.saturating_sub(1).max(1), 0),
                        std::sync::atomic::Ordering::Relaxed,
                    );
                    if automated {
                        automation.apply(bar, &mut composer, [0.5, 0.0, 0.5, 0.0]);
                    }
                    composer.generate_bars(if bar == 1 { 1 } else { 2 });
                    for snapshot in composer.take_snapshots() {
                        let notes: Vec<_> = snapshot
                            .notes
                            .iter()
                            .map(|n| (n.track, n.pitch, n.start_step, n.duration_steps, n.velocity))
                            .collect();
                        count += notes.len();
                        bars.push((snapshot.index, notes));
                    }
                }
                assert_eq!(bars.len(), 100);
                runs.push(serde_json::json!({
                    "seed": seed, "combat": automated, "bars": bars
                }));
            }
        }
        let actual = serde_json::json!({
            "source": if fixed_roles { "main:64266ff5b27284a6cd37dfb82c500325776f1cb0".to_string() } else { format!("TUNING_VERSION:{}", harmonium_core::tuning::TUNING_VERSION) },
            "runs": runs
        });
        let filename = if fixed_roles { "profile_d2_fixed.json" } else { "profile_d2_main.json" };
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(filename);
        if !fixed_roles && std::env::var_os("HARMONIUM_UPDATE_GOLDENS").is_some() {
            std::fs::write(&path, serde_json::to_string(&actual)?)?;
        }
        let reference: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        assert_eq!(actual, reference, "D2 : notes différentes de main");
        println!("D2 (rôles fixes = {fixed_roles}) : {count} notes comparées, 1000 mesures, 0 différence");
        Ok(())
    }
}

//! Le playback suit le plan de temps (phase 013, J1).
//!
//! Pour chacun des profils distribués, avec et sans automation (émotions, tempo direct,
//! retour aux émotions, énergie), le moteur offline joue 48 mesures — changements de tempo,
//! rampe de set et frontières d'air comprises — et chaque pas réellement rendu tombe à
//! ± 1 frame de l'instant que le plan construit depuis les en-têtes des snapshots
//! ([`BarTiming::of_snapshot`]) calcule seul. Le playhead publie un écart nul avec le plan ;
//! un sous-débit (mesure absente) apparaît dans l'écart publié au lieu d'être corrigé en
//! silence.

use std::{collections::BTreeMap, time::Instant};

use harmonium::{audio::create_offline_engine, composer::MusicComposer, playback::PlaybackCommand};
use harmonium_core::{
    StyleProfile,
    time_plan::{BarTiming, Flicks, Tempo, TimePlan},
};

const SAMPLE_RATE: u32 = 44_100;
const BUFFER: usize = 512;
const BARS: usize = 48;
const SEED: u64 = 11;

/// Commande de jeu appliquée juste avant de générer la mesure à laquelle elle est datée.
#[derive(Clone, Copy)]
enum Command {
    Emotions { arousal: f32, valence: f32, density: f32, tension: f32 },
    Intensity(f32),
    Tempo(Tempo),
    ReleaseTempo,
}

type Dated = (usize, Command);

impl Command {
    fn apply(self, composer: &mut MusicComposer) {
        match self {
            Self::Emotions { arousal, valence, density, tension } => {
                composer.use_emotion_mode();
                composer.set_emotions(arousal, valence, density, tension);
            }
            Self::Intensity(value) => composer.set_intensity(value),
            Self::Tempo(tempo) => composer.set_bpm(tempo.bpm()),
            Self::ReleaseTempo => composer.reset_bpm(),
        }
    }
}

/// Automation de jeu datée à la mesure, relative au tempo nominal du profil.
fn automation(default_bpm: f32) -> Vec<Dated> {
    let direct = Tempo::from_milli_bpm((default_bpm * 1000.0) as u32 + 7_250);
    vec![
        (4, Command::Emotions { arousal: 0.95, valence: -0.3, density: 0.8, tension: 0.8 }),
        (11, Command::Tempo(direct)),
        (17, Command::ReleaseTempo),
        (17, Command::Intensity(0.15)),
        (30, Command::Emotions { arousal: 0.2, valence: 0.5, density: 0.4, tension: 0.1 }),
    ]
}

/// Ce que le playhead publie pour chaque pas rendu.
struct RenderedStep {
    start_frame: u64,
    plan_instant: Flicks,
    plan_gap: i64,
}

fn forward_drop(composer: &mut MusicComposer, commands: &mut rtrb::Producer<PlaybackCommand>) {
    if let Some((bar, epoch)) = composer.take_pending_drop() {
        assert!(commands.push(PlaybackCommand::DropFrom { bar, epoch }).is_ok());
    }
}

/// Ce que le moteur a joué : les pas rendus, le nombre d'airs traversés et le plan que les
/// en-têtes des snapshots (le flux du contrat audio) construisent pour les mêmes mesures.
struct Played {
    steps: BTreeMap<(usize, usize), RenderedStep>,
    tunes: usize,
    plan: TimePlan,
}

/// Joue `BARS` mesures du profil en appliquant chaque commande juste avant de générer sa
/// mesure.
fn render(profile: &StyleProfile, commands: &[Dated]) -> Result<Played, String> {
    let (mut composer, mut playback, mut queue, mut reports, mut clock, _recordings) =
        create_offline_engine(None, None, &[], f64::from(SAMPLE_RATE))?;
    composer.set_profile(profile.clone())?;
    composer.set_seed(SEED);
    forward_drop(&mut composer, &mut queue);
    assert!(
        queue
            .push(PlaybackCommand::UpdateMusicalParams(composer.musical_params().playback_params()))
            .is_ok()
    );

    let mut steps = BTreeMap::new();
    let mut tunes = std::collections::BTreeSet::new();
    let mut timings = BTreeMap::new();
    let mut buffer = [0.0f32; BUFFER * 2];
    loop {
        let playhead = composer.playhead_bar();
        while composer.writehead_position() <= BARS + 1
            && composer.writehead_position() < playhead + 2
        {
            let bar = composer.writehead_position();
            for (_, command) in commands.iter().filter(|(at, _)| *at == bar) {
                command.apply(&mut composer);
            }
            composer.sync_generator();
            composer.generate_bars(bar + 1 - playhead);
            for snapshot in composer.take_snapshots() {
                timings.insert(snapshot.index, BarTiming::of_snapshot(&snapshot));
                if let Some(position) = snapshot.position {
                    tunes.insert(position.tune_index);
                }
            }
        }
        playback.process_buffer(&mut buffer, 2, Instant::now());
        while reports.pop().is_ok() {}
        let published = clock.read();
        if published.next_pulse_frame > 0 {
            steps.entry((published.position.bar, published.position.step)).or_insert(
                RenderedStep {
                    start_frame: published.step_start_frame,
                    plan_instant: published.plan_instant,
                    plan_gap: published.plan_gap_frames,
                },
            );
        }
        if published.position.bar > BARS {
            let mut plan = TimePlan::new();
            for bar in 1..=timings.len() {
                plan.push(*timings.get(&bar).ok_or_else(|| format!("snapshot {bar} absent"))?);
            }
            return Ok(Played { steps, tunes: tunes.len(), plan });
        }
    }
}

#[test]
fn rendered_steps_land_on_the_snapshot_plan_for_every_profile() -> Result<(), String> {
    let profiles = StyleProfile::distributed();
    assert!(!profiles.is_empty());
    let mut set_ramp_runs = 0;
    for base in profiles {
        let mut profile = base.clone();
        profile.arrangement.tours_per_tune = 1;
        for automated in [false, true] {
            let commands =
                if automated { automation(profile.meter.default_bpm) } else { Vec::new() };
            let label = format!("{} (automation {automated})", profile.meta.id);

            let Played { steps, tunes, plan } = render(&profile, &commands)?;
            assert!(plan.bars() > BARS, "{label} : {} mesures composées", plan.bars());
            let steps_per_bar = usize::from(plan.timing(1).map_or(0, |t| t.steps));
            assert!(steps.len() >= BARS * steps_per_bar, "{label} : {} pas vus", steps.len());
            for (&(bar, step), rendered) in &steps {
                if bar > BARS {
                    continue;
                }
                let planned = plan
                    .step_instant(bar, step as u16)
                    .ok_or_else(|| format!("{label} : pas {bar}:{step} hors plan"))?;
                assert_eq!(rendered.plan_instant, planned, "{label} : instant {bar}:{step}");
                let expected = planned.to_frames(SAMPLE_RATE);
                assert!(
                    rendered.start_frame.abs_diff(expected) <= 1,
                    "{label} : pas {bar}:{step} rendu à {}, plan {expected}",
                    rendered.start_frame
                );
                assert_eq!(rendered.plan_gap, 0, "{label} : écart publié {bar}:{step}");
            }

            // Le parcours a bien traversé ce que le contrat promet.
            let tempos: std::collections::BTreeSet<_> =
                (1..=BARS).filter_map(|bar| plan.timing(bar).map(|t| t.tempo)).collect();
            let ramps = (2..=BARS).any(|bar| {
                plan.pulse_tempo(bar, 1) != plan.timing(bar).map(|t| t.tempo)
                    && plan.timing(bar).is_some_and(|t| t.ramp_in)
            });
            assert!(tunes >= 2, "{label} : un seul air traversé");
            if automated {
                assert!(tempos.len() >= 3, "{label} : tempos {tempos:?}");
                assert!(ramps, "{label} : aucune rampe de mesure");
            } else if tempos.len() >= 2 && ramps {
                // Sans automation, seul l'arc de tempo du set change le tempo.
                set_ramp_runs += 1;
            }
        }
    }
    assert!(set_ramp_runs >= 3, "rampe de set traversée par {set_ramp_runs} profils");
    Ok(())
}

#[test]
fn an_underflow_is_published_as_a_plan_gap_not_hidden() -> Result<(), String> {
    let (mut composer, mut playback, mut queue, _reports, mut clock, _recordings) =
        create_offline_engine(None, None, &[], f64::from(SAMPLE_RATE))?;
    composer.set_profile(StyleProfile::named("reel_quebecois").map_err(|e| e.to_string())?)?;
    composer.set_seed(SEED);
    forward_drop(&mut composer, &mut queue);
    composer.generate_bars(2);
    let mut buffer = [0.0f32; BUFFER * 2];
    let mut gap_before = None;
    for buffers in 0..2000 {
        // Mesure 3 générée seulement après ~6 s : sous-débit, puis reprise.
        if buffers == 700 {
            composer.generate_bars(3);
        }
        playback.process_buffer(&mut buffer, 2, Instant::now());
        let published = clock.read();
        if published.position.bar == 2 && gap_before.is_none() {
            gap_before = Some(published.plan_gap_frames);
        }
        if published.position.bar >= 3 {
            assert!(buffers >= 700);
            assert_eq!(gap_before, Some(0));
            assert!(published.plan_gap_frames > 0, "écart non publié après sous-débit");
            return Ok(());
        }
    }
    Err("la mesure 3 n'a jamais été jouée".into())
}

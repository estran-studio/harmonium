//! Démonstration native : deux sources AssetServer nommées lisent directement les
//! banques du checkout principal et les presets plats, sans copie dans le studio.

use std::{env, error::Error, io, path::PathBuf, time::Duration};

use bevy::{
    app::AppExit,
    asset::io::{AssetSourceBuilder, file::FileAssetReader},
    prelude::*,
};
use harmonium_bevy::{Beat, HarmoniumConfig, HarmoniumControl, HarmoniumPlugin, MusicClock};

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct Pulse;

#[derive(Resource, Default)]
struct PulseLevel(f32);

#[derive(Resource)]
struct Smoke(Option<Duration>);

/// Dossier lu dans une variable d'environnement : `~` développé (une variable passée à
/// `make VAR=~/…` arrive non développée sous zsh), chemin absolu, erreur qui nomme le dossier.
fn folder(var: &str, default: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    let raw = env::var_os(var).map_or(default, PathBuf::from);
    let home = PathBuf::from(env::var_os("HOME").unwrap_or_default());
    let path = match raw.strip_prefix("~") {
        Ok(rest) => home.join(rest),
        Err(_) => raw,
    };
    path.canonicalize().map_err(|error| {
        format!("{var} : dossier introuvable « {} » ({error})", path.display()).into()
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let home = PathBuf::from(env::var_os("HOME").unwrap_or_default());
    let banks = folder("HARMONIUM_BANKS", PathBuf::from("assets"))?;
    let presets = folder("HARMONIUM_ODIN", home.join("Project/bascanada/odin-presets"))?;
    let mut odin_presets = Vec::new();
    for entry in std::fs::read_dir(&presets)? {
        let entry = entry?;
        if entry.file_type()?.is_file() && entry.path().extension().is_some_and(|ext| ext == "odin")
        {
            let name = entry.file_name().into_string().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Nom de preset non UTF-8")
            })?;
            odin_presets.push(format!("odin://{name}"));
        }
    }
    odin_presets.sort();
    let mut seconds = env::var("HARMONIUM_DEMO_SECONDS").ok();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--smoke-seconds" {
            seconds = Some(args.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--smoke-seconds attend une durée")
            })?);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Argument inconnu : {arg}"),
            )
            .into());
        }
    }
    let smoke = seconds
        .map(|s| -> Result<Duration, Box<dyn Error>> {
            let value: f64 = s.parse()?;
            if !value.is_finite() || value <= 0.0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "La durée doit être finie et positive",
                )
                .into());
            }
            Ok(Duration::try_from_secs_f64(value)?)
        })
        .transpose()?;
    let mut control = HarmoniumControl::default();
    control.set_profile_id("nt_desert")?;
    // Vérification d'agent : sortie muette, sans empêcher le callback ni la composition.
    if smoke.is_some() {
        control.volume = 0.0;
    }
    eprintln!(
        "Démarrage démo Bevy — banques : {} ; presets : {} ({} fichiers) ; arrêt : {smoke:?}",
        banks.display(),
        presets.display(),
        odin_presets.len()
    );
    let mut app = App::new();
    // Enregistrer les sources avant AssetPlugin : chemins absolus, natifs, sans duplication.
    app.register_asset_source(
        "banks",
        AssetSourceBuilder::new(move || Box::new(FileAssetReader::new(banks.clone()))),
    );
    app.register_asset_source(
        "odin",
        AssetSourceBuilder::new(move || Box::new(FileAssetReader::new(presets.clone()))),
    );
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Harmonium — laboratoire Bevy".into(),
            resolution: (920, 620).into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(control)
    .insert_resource(Smoke(smoke))
    .init_resource::<PulseLevel>()
    .add_plugins(HarmoniumPlugin::new(HarmoniumConfig {
        main_bank: Some("banks://GeneralUser-GS.sf2".into()),
        feet_bank: Some("banks://harmonium_trad.sf2".into()),
        odin_presets,
        ..default()
    }))
    .add_systems(Startup, setup)
    .add_systems(Update, (keyboard, smoke_exit))
    .add_systems(Last, (pulse, hud))
    .run();
    Ok(())
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text::new("Chargement des banques…"),
        TextFont { font_size: FontSize::Px(22.0), ..default() },
        Node { position_type: PositionType::Absolute, left: px(24), top: px(24), ..default() },
        Hud,
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: px(40),
            bottom: px(40),
            width: px(100),
            height: px(100),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.35, 0.45)),
        Pulse,
    ));
}

fn keyboard(keys: Res<ButtonInput<KeyCode>>, mut control: ResMut<HarmoniumControl>) {
    for (key, id) in [
        (KeyCode::Digit1, "nt_desert"),
        (KeyCode::Digit2, "isaac_basement"),
        (KeyCode::Digit3, "isaac_cellar"),
        (KeyCode::Digit4, "isaac_burning"),
        (KeyCode::Digit5, "isaac_downpour"),
    ] {
        if keys.just_pressed(key)
            && let Err(error) = control.set_profile_id(id)
        {
            eprintln!("Profil {id} : {error}");
        }
    }
    for (up, down, index) in [
        (KeyCode::KeyA, KeyCode::KeyZ, 0),
        (KeyCode::ArrowRight, KeyCode::ArrowLeft, 1),
        (KeyCode::KeyD, KeyCode::KeyF, 2),
        (KeyCode::ArrowUp, KeyCode::ArrowDown, 3),
    ] {
        let delta = f32::from(u8::from(keys.just_pressed(up)))
            - f32::from(u8::from(keys.just_pressed(down)));
        if delta != 0.0 {
            control.emotions[index] = (control.emotions[index] + delta * 0.05).clamp(0.0, 1.0);
        }
    }
    if keys.just_pressed(KeyCode::Space) {
        control.paused = !control.paused;
    }
    if keys.just_pressed(KeyCode::KeyN) {
        control.seed = control.seed.wrapping_add(1);
    }
}

fn pulse(
    mut beats: MessageReader<Beat>,
    time: Res<Time<Real>>,
    mut level: ResMut<PulseLevel>,
    mut square: Query<(&mut Node, &mut BackgroundColor), With<Pulse>>,
) {
    level.0 = (level.0 - time.delta_secs() * 3.5).max(0.0);
    if beats.read().count() > 0 {
        level.0 = 1.0;
    }
    for (mut node, mut color) in &mut square {
        let size = 100.0 + level.0 * 45.0;
        node.width = px(size);
        node.height = px(size);
        color.0 = Color::srgb(0.15 + level.0 * 0.65, 0.35 + level.0 * 0.55, 0.45 + level.0 * 0.45);
    }
}

fn hud(
    control: Res<HarmoniumControl>,
    clock: Res<MusicClock>,
    mut text: Query<&mut Text, With<Hud>>,
) {
    for mut text in &mut text {
        **text = format!(
            "HARMONIUM / BEVY\n\nProfil cible : {}\nÉveil (arousal) : {:.2}    Valence : {:.2}\nDensité : {:.2}    Tension : {:.2}\nGraine : {}    {}\n\nHorloge : {}\nMesure : {}    Temps : {:.2}\nPulse : {}/{}    Phase : {:.2}    Tempo : {:.1}\n\n1 désert · 2 sous-sol · 3 cave · 4 incendie · 5 pluie\n↑/↓ tension · ←/→ valence\nA/Z éveil · D/F densité (±0,05)\nEspace : pause / reprise · N : nouvelle graine\n\nLe carré suit les messages Beat du transport audio.",
            control.profile.meta.id,
            control.emotions[0],
            control.emotions[1],
            control.emotions[2],
            control.emotions[3],
            control.seed,
            if control.paused { "pause demandée" } else { "lecture demandée" },
            if !clock.valid {
                "en attente du premier tampon"
            } else if clock.playing {
                "lecture"
            } else {
                "arrêtée"
            },
            clock.bar,
            clock.beat,
            clock.pulse,
            clock.pulses_per_bar,
            clock.phase,
            clock.bpm,
        );
    }
}

fn smoke_exit(time: Res<Time<Real>>, smoke: Res<Smoke>, mut exit: MessageWriter<AppExit>) {
    if smoke.0.is_some_and(|duration| time.elapsed() >= duration) {
        eprintln!("Fin automatique de la démo après {:.2} s", time.elapsed_secs());
        exit.write(AppExit::Success);
    }
}

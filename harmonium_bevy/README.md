# harmonium_bevy

Plugin mince **Bevy 0.19.1** sur `harmonium::LiveHost`. Pas de logique musicale, de synthèse ni de sortie `bevy_audio` dans le plugin : cpal reste propriétaire de la musique. Le jeu peut conserver `bevy_audio` pour ses bruitages.

Le crate est membre du workspace, hors `default-members`. Par défaut : SoundFont, licence `MIT OR Apache-2.0`. **Activer `odin` lie du code GPLv3 : le binaire du jeu doit être distribué sous GPL.** Ne pas activer cette option pour un jeu fermé.

## Ajouter le plugin

```toml
[dependencies]
harmonium_bevy = { path = "../harmonium/harmonium_bevy" }
# Jeu GPL seulement : features = ["odin"]
```

```rust,no_run
use bevy::prelude::*;
use harmonium_bevy::{HarmoniumConfig, HarmoniumControl, HarmoniumPlugin};

fn main() -> Result<(), String> {
    let mut control = HarmoniumControl::default();
    control.set_profile_id("nt_desert")?;
    control.seed = 42;
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(control)
        .add_plugins(HarmoniumPlugin::new(HarmoniumConfig {
            main_bank: Some("audio/GeneralUser-GS.sf3".into()),
            percussion_bank: Some("audio/harmonium_trad.sf2".into()),
            // Avec la feature odin : liste explicite des presets du jeu.
            odin_presets: vec![],
            ..default()
        }))
        .run();
    Ok(())
}
```

`AssetPlugin` et `TimePlugin` doivent être présents (`DefaultPlugins` les fournit). Aucun moteur de rendu, fenêtre ou clavier n'est requis par la bibliothèque. En simulation : `MinimalPlugins` + `AssetPlugin`.

### Ressources déclarées

Les chemins passent tous par l'`AssetServer` : fichiers relatifs au dossier d'assets du jeu, URL relatives sur le web, ou sources Bevy nommées. Le chargeur accepte `.sf2`, `.sf3`, `.odin`. Un preset porte exactement le nom de son fichier sans extension ; le profil doit employer ce nom. Les presets d'un même jeu doivent avoir des noms courts distincts.

L'hôte n'est ouvert qu'après chargement de **toutes** les ressources déclarées. Un fichier déclaré introuvable produit `HarmoniumStatus::Failed(String)` et un journal, sans nouvelle tentative par frame. `HarmoniumStatus::{Loading, Ready, Failed}` est lisible par l'écran du jeu ; `Ready` signifie hôte initialisé, pas forcément lecture active. Les ressources que le profil nomme mais qui n'ont pas été déclarées restent accessibles par `runtime.host().missing_resources()` ; l'hôte les journalise à l'installation du profil, puis seulement si l'absence change, jamais à chaque frame. Le plugin ne duplique pas ces avis. Une banque principale omise est autorisée par l'hôte, mais ne fournit pas de son SoundFont.

Les octets résident dans `Assets<HarmoniumAsset>` pendant le chargement ; les handles sont conservés pour la vie du plugin. Le décodage des instruments par `LiveHost::start_paused` bloque brièvement le fil principal, après le chargement asynchrone des octets. Pas de rechargement à chaud d'instruments dans l'hôte vivant.

## Traduire les signaux du jeu

Le jeu écrit `HarmoniumControl` dans `Update` ; le plugin consomme ses cibles dans `PostUpdate`, ensemble `HarmoniumSet`. Pour écrire dans `PostUpdate`, ordonner le système avant cet ensemble.

```rust
use bevy::prelude::*;
use harmonium_bevy::HarmoniumControl;

fn combat(mut music: ResMut<HarmoniumControl>) {
    // Exemple de traduction choisie par le jeu, pas une règle du moteur.
    music.emotions = [0.8, 0.25, 0.7, 0.9]; // arousal, valence, density, tension
    music.volume = 0.6;
    music.paused = false;
}
```

Axes et volume : valeurs finies dans `0..=1`. Les émotions ne sont envoyées que si au moins un axe s'écarte de `emotion_threshold` depuis la **dernière cible envoyée**, et si `emotion_interval` est écoulé. Défauts : seuil `0.02`, intervalle `100 ms`, temps réel Bevy (`Time<Real>`, indépendant du temps virtuel du jeu). Les petits écarts s'accumulent ; pas de rafale de rattrapage après une frame lente.

- Profil distribué : `control.set_profile_id("isaac_burning")?`.
- Profil personnalisé validé : `control.profile = mon_style_profile`.
- Graine : `control.seed = 123`; une nouvelle graine repart à la mesure 1.
- Pause : `control.paused = true`; reprise : `false`. `start_paused` en configuration fixe la cible initiale en pause.
- `HarmoniumRuntime`, stocké **non-Send**, offre `host()` / `host_mut()` sous emprunts `Ref` / `RefMut`. Ne pas garder un emprunt entre systèmes/callbacks. Préférer les cibles publiques pour conserver la cohérence des fondus et de la cadence.

### Transition de profil

`fade_duration` est la durée de **chaque demi-fondu** (défaut 500 ms). Le plugin descend le volume courant jusqu'à zéro, attend dans `TransportClock` l'acquittement de sa commande (`volume_revision`) **et** le gain réellement atteint (`output_gain == 0`), installe `SetStyleProfile`, réinstalle les émotions du jeu, repart par `SetSeed` avec la graine cible conservée, puis remonte vers le volume cible. Aucune substitution de profil avant le silence rendu, même si une transaction Odin retarde la file de commandes. Le plugin ne suppose pas la durée de la rampe anti-clic. `SetSeed` vide aussi les mesures passées : `Seek(1)` seul aurait rejoué la mesure 1 de l'ancien profil. Le redémarrage est la dernière invalidation ; les émotions suivantes attendent son acquittement par l'horloge audio pour ne pas générer depuis l'ancien playhead.

Le tampon qui atteint zéro peut encore contenir le début de la rampe : le plugin attend également la frontière du tampon suivant, entièrement à gain zéro, avant d'installer le profil.

Une demande pendant la descente remplace la cible en attente ; pendant la remontée, elle déclenche une nouvelle descente depuis le volume atteint. Un profil invalide est refusé avant le début du fondu. En pause, le passage au nouveau profil attend la reprise du callback : le temps UI seul ne prouve pas le silence audio. Même une durée nulle conserve cette barrière audio. Une transition redémarre la forme ; elle ne fait pas de raccord harmonique (J3 reste hors périmètre).

## Lire l'horloge et les messages

```rust
use bevy::prelude::*;
use harmonium_bevy::{Bar, Beat, MusicClock};

fn pulse(clock: Res<MusicClock>, mut beats: MessageReader<Beat>, mut bars: MessageReader<Bar>) {
    if clock.valid && clock.playing {
        // clock.bar : mesure 1-based ; clock.beat : noire sur la grille 1-based.
        // clock.pulse : pulsation dansée 1-based ; clock.phase : [0, 1).
        // clock.bpm : tempo entendu à la noire, pas le tempo nominal du profil.
    }
    for beat in beats.read() { let _ = (beat.bar, beat.pulse); }
    for bar in bars.read() { let _ = bar.bar; }
}
```

L'horloge vient du `ClockReader` public pris à `LiveHost` et de `BeatInfo::from_clock`, avec la position cohérente du callback. Le tempo convertit la durée de la pulse par le mètre de la mesure réellement composée. Avant le premier tampon : `valid == false`. En pause : position conservée et `playing == false`, aucun nouveau message. Une reprise ne répète pas la pulse déjà observée. Un redémarrage invalide temporairement l'horloge jusqu'au retour du callback à la mesure 1.

La première pulse observée émet `Beat` et la mesure observée émet `Bar`. Ensuite : un message par nouvelle frontière ; les frames lentes rattrapent les pulses sautées à mètre constant. Un saut de session ne simule pas les temps de la session abandonnée. Les messages suivent les règles de rétention Bevy : les consommer régulièrement. Pour une réaction dans la même frame, placer le lecteur après `HarmoniumSet` ou dans `Last`.

## Suivre l'horloge du jeu

Quand la simulation du jeu est la source du temps (jeu en ligne à rollback, par exemple), la musique **suit son horloge** : chaque client génère et joue sa musique, et tous entendent la même pulsation au même instant de simulation. Sans horloge, rien ne change (tempo des émotions, mesure 1 au démarrage).

Le jeu écrit la ressource `GameClock` :

| Champ | Unité | Rôle |
|---|---|---|
| `clock` | `Option<ExternalClock>` | `None` : pas de suivi. Posée à l'entrée dans une salle, remplacée si la salle ou le tempo change |
| `ExternalClock::new(sim_hz, anchor_frame, num, den)` | `sim_hz` en frames par seconde (60 pour `alacod`) ; `anchor_frame` en frame de simulation ; `num/den` en frames | l'ancrage est la frame où commence la **pulsation 1 de la mesure 1** ; la période est la durée d'**une pulsation dansée** du profil, en frames, fraction permise (`3600/230`) |
| `sim_frame` | frame de simulation | la frame courante, écrite à **chaque frame de présentation** |
| `frame_fraction` | `0.0..1.0` | fraction de la frame écoulée, si le jeu la connaît ; `0.0` sinon |

La pulsation dansée est celle du profil (`meter`, `steps_per_pulse`) : la **blanche** au reel (2/2), la **noire pointée** en 6/8, la **noire** en 4/4. Le tempo à la noire s'en déduit par le mètre : une période de 40 frames à 60 Hz fait 180 BPM à la noire au reel (blanche de 0,667 s), 90 BPM en 4/4. Il doit tomber entre 40 et 300 BPM pour le profil actif, sinon la horloge est refusée (journal, état précédent conservé).

```rust
use bevy::prelude::*;
use harmonium_bevy::{ExternalClock, GameClock, HarmoniumSet, MusicClock};

/// Dans `Update` : la frame de simulation qu'on présente.
fn feed(mut game_clock: ResMut<GameClock>, sim: Res<MySimulation>) {
    game_clock.sim_frame = sim.frame;
}

/// À l'entrée dans la salle (et si son tempo ou son ancrage changent).
fn enter_room(mut game_clock: ResMut<GameClock>, room: Res<MyRoom>) -> Result<(), String> {
    // Mesure 1 commencée à `room.start_frame` ; la pulsation dure `3600/230` frames à 60 Hz.
    game_clock.clock =
        Some(ExternalClock::new(60, room.start_frame, 3600, 230).map_err(|e| e.to_string())?);
    Ok(())
}

/// Diagnostic : écart filtré et dernière correction.
fn debug(clock: Res<MusicClock>) {
    if let Some(follow) = clock.follow {
        let _ = (follow.phase, follow.gap_ms(), follow.last_correction);
    }
}
```

Le plugin joint à `sim_frame` l'instant de la frame (`Time<Real>::last_update`, soit `Instant::now()` au début de la frame de présentation, dans `PostUpdate`) et transmet l'observation à l'hôte (`LiveHost::observe_external`). Le jeu n'a donc qu'à écrire la frame **avant** `HarmoniumSet`, et à écrire la même frame que celle qu'il présente. Avec un pas fixe, écrire la dernière frame simulée et sa fraction (`Time<Fixed>::overstep_fraction()`) donne le meilleur résultat ; une frame qui tremble d'une frame (16,7 ms) est filtrée.

Ce que fait la musique :

- **Tempo.** La pulsation du profil dure exactement la période. Le tempo d'émotion, la rampe de set et la variante de tempo de session ne s'appliquent plus (même priorité que `EngineCommand::SetBpm`) ; les émotions agissent toujours sur le reste (densité, mode, dynamique). Le tempo est une valeur entière en milli-BPM : il arrondit la période de 0,5 / (milli-BPM) au plus, soit 4 ppm à 120 BPM, 8 ppm à 60 BPM ; le suivi le rattrape.
- **Démarrage et reprise** (geste web, `paused`, retour d'onglet). Dès que `clock` est posée, le playback reste **silencieux** : il ne part pas de sa mesure 1 au premier tampon. Quand l'audio tourne et que 0,5 s d'observations sont disponibles, l'hôte saute (seek déterministe, même graine, même mesure sur tous les clients) à la première mesure de l'horloge dont l'instant audible est encore à venir (150 ms de marge), puis joue exactement à cet instant. La première mesure entendue est donc celle de la simulation, pas la 1.
- **Suivi.** À chaque pulsation, l'écart est la position audible moins la position de l'horloge au même instant, filtrée. Sous 10 ms (zone morte) : rien. Sinon, un micro-ajustement de la conversion instants → frames du playback, de 5 ‰ au plus, sur 8 pulsations ; ni rééchantillonnage, ni nouvel instant du plan, ni note modifiée. Au-delà de 5 ‰ (environ 27 ms sur une pulsation de 0,67 s), l'audio finit sa mesure puis saute à la mesure suivante de l'horloge, avec au plus une mesure de silence. Les seuils se règlent par `HarmoniumConfig::follow` (`FollowConfig`).
- **Changement d'horloge.** Poser une autre `ExternalClock` remplace la précédente : l'audio finit sa mesure et se réaligne sur la mesure suivante de la nouvelle, avec au plus une mesure de silence. Retirer l'horloge (`None`) rend le tempo aux émotions.
- **Publication.** `MusicClock` et `Beat`/`Bar` restent le playhead (ce que le joueur **entend**), jamais interpolés ni corrigés après coup. `MusicClock::follow: Option<ExternalStatus>` publie la phase (`Waiting`, `Starting`, `Tracking`), l'écart filtré (flicks, `gap_ms()` ; positif quand l'audio est en avance), la dernière correction et les comptes de micro-ajustements et de sauts. `TransportClock` publie aussi `nudge_ppm` et `holding`. Le gameplay ne lit jamais l'audio : il lit son horloge.

Ce que le jeu doit fournir : la fréquence de simulation, l'ancrage et la période **en frames de simulation** ; la frame à chaque frame de présentation ; un `GameClock::clock` qui change seulement quand la salle ou le tempo change. Une nouvelle graine ou un nouveau profil (`HarmoniumControl`) redémarre la musique : elle attend alors de nouveau son instant dans l'horloge.

Limites :

- **Latence de sortie.** Natif : cpal 0.15 publie `playback − callback`, l'instant où le tampon est audible selon le pilote (ALSA : délai `snd_pcm_delay` du tampon ; WASAPI : estimation qui, de l'aveu de cpal, ignore une petite latence). Web : cpal 0.15 ne publie que le délai de programmation du tampon dans `AudioContext` (`playback` = `currentTime` + 25 ms puis tampons successifs) ; il **omet** `outputLatency` et `baseLatency`, que le navigateur ne donne pas toujours (Chrome : quelques ms à 100 ms ; Bluetooth : 150 à 300 ms, qu'aucune API fiable n'indique). Le suivi aligne donc l'instant où le tampon est **programmé**. Un décalage connu se compense par `HarmoniumConfig::follow.extra_output_latency`.
- **Onglet gelé** : plus de callback ni de frame ; au retour, l'audio est vu à l'arrêt (250 ms sans tampon), mis en attente, puis saute à la mesure courante de l'horloge.
- **Gigue de la simulation entre pairs** (GGRS) : l'horloge est celle de la simulation locale. Un client qui rattrape ou ralentit sa simulation décale d'autant sa musique, à la vitesse du rattrapage ; un saut brutal (plus de 100 ms de décalage entre la simulation et l'heure murale, trois frames de suite) vide le filtre et provoque un saut de mesure.
- **Mètre** : les pulsations par mesure sont celles du profil (`SetTimeSignature` en cours de route n'est pas suivi), et une pulsation incomplète en fin de mesure n'est pas prise en compte.

## Pompage et navigateur

Réserve : **16 mesures**, maximum de la fenêtre de lecture du moteur. Natif : `pump()` depuis le système, au plus une fois toutes les **50 ms** par défaut ; pas de boucle de rattrapage. Le pompage et les commandes restent sur le fil principal. Un jeu qui bloque ce fil au-delà de la réserve peut manquer de mesures.

Web : hôte toujours ouvert en pause. Le plugin installe des callbacks DOM `pointerdown`, `keydown`, `touchend` : au premier geste **après chargement**, si la cible n'est pas en pause, il appelle `resume()` synchronement dans le geste. Ne pas remplacer cela par une reprise différée dans `Update` : certains navigateurs refusent le geste différé. Si `start_paused` est vrai, mettre `control.paused = false` avant le geste de démarrage. Le geste fait pendant un écran encore en chargement n'est pas mémorisé : afficher un bouton de démarrage quand `HarmoniumStatus::Ready` est atteint.

Un Worker poste une échéance de pompage toutes les **100 ms** ; le callback sur le fil de la page appelle l'hôte partagé par `Rc<RefCell<_>>`, indépendamment de `requestAnimationFrame`. Prévoir dans la CSP les Workers Blob (`worker-src blob:`) et les requêtes d'assets. Le Worker et les écouteurs sont retirés quand le runtime est détruit. Un onglet entièrement gelé n'est pas couvert : comme le studio, le plugin dispose seulement des 16 mesures préparées. L'horloge et les messages gameplay restent actualisés par les frames Bevy.

## Démo locale et vérifications

```sh
make bevy/demo
make bevy/demo HARMONIUM_ODIN="$HOME/Project/bascanada/odin-presets"
make bevy/demo ARGS="--smoke-seconds 8"
make bevy/test
make bevy/check
```

La démo requiert `demo,odin` : binaire GPL. `HARMONIUM_BANKS` reprend `STUDIO_BANKS` (dossier `assets/` du checkout principal, aussi en worktree). Fichiers : `GeneralUser-GS.sf2`, `harmonium_trad.sf2`. `HARMONIUM_ODIN` désigne un dossier **plat** de presets `.odin`, défaut `~/Project/bascanada/odin-presets`. Les sources AssetServer nommées `banks://` et `odin://` lisent directement ces dossiers, sans copie ni modification du studio.

`--smoke-seconds N` (ou `HARMONIUM_DEMO_SECONDS=N`) quitte automatiquement et fixe le volume à zéro : vérification visuelle et de composition **sans écoute ni son sur la carte**. Sans cette option, la démo joue normalement.

| Touches | Action |
|---|---|
| 1–5 | `nt_desert`, `isaac_basement`, `isaac_cellar`, `isaac_burning`, `isaac_downpour` |
| ↑ / ↓ | tension ±0,05 |
| → / ← | valence ±0,05 |
| A / Z | arousal ±0,05 |
| D / F | densité ±0,05 |
| Espace | pause / reprise |
| N | graine suivante |

HUD : profil cible, émotions, graine, mesure/noire/pulse/phase/tempo ; carré animé sur les messages `Beat`. L'écoute appartient au propriétaire. Une session d'agent contrôle le chargement, la composition et le rendu hors ligne, sans revendiquer d'écoute.

# Harmonium

Moteur Rust de musique procédurale tous styles — traditions, musiques de jeu,
genres fusionnés — piloté en temps réel par l'état d'un jeu. Premier client :
« 1837 » (roguelike Rust/Bevy) ; le beat est exposé au gameplay, quatre
émotions pilotent la musique.

Direction et principes : [`.specify/memory/constitution.md`](.specify/memory/constitution.md).
Phase en cours : [`specs/009-phase1-reel/plan.md`](specs/008-phase0-clock-playback/plan.md).

## Workspace

| Crate | Rôle |
|---|---|
| `harmonium_core` | Types purs et génération : grille de pas, `Measure`, `TimelineGenerator`, `Writehead`/`Playhead`, harmonie, `TuningParams`/`StyleProfile`, mapping émotion → paramètres. Aucune I/O. |
| `harmonium_audio` | Rendu SoundFont (oxisynth), enregistrement WAV / MIDI / MusicXML, garde d'allocation temps réel. |
| `harmonium_host` | `MusicComposer` (thread principal) + `PlaybackEngine` (callback cpal), transport lock-free, `LiveHost` (hôte vivant : exécuteur d'`EngineCommand`, `pump`), module OSC, `create_offline_engine`. |
| `harmonium_cli` | REPL d'écoute et de test (`harmonium-cli`), contrôle OSC par `--osc`. |
| `harmonium_studio` | Studio egui (natif et web) : jouer le moteur, le voir et le régler — émotions, piano-roll, mixeur, enregistrement. Toujours compilé avec `odin`, hors des membres par défaut. |
| `harmonium_lab` | Ingestion de corpus MusicXML, métriques de distance, tuning assisté. |
| `harmonium_determinism` | Harnais : les mêmes snapshots en natif et en `wasm32-unknown-unknown`, comparés octet par octet. |

## Démarrer

```bash
make soundfont           # télécharge la banque GM complète GeneralUser GS (~32 Mo)
make run                 # REPL, charge assets/GeneralUser-GS.sf2 par défaut (ou repli test.sf2)
make run ARGS="--soundfont path/to/bank.sf2"
make run ARGS="--osc"   # REPL + OSC : /harmonium/params (arousal valence density tension) sur UDP 127.0.0.1:8080
make test                # tests sans Odin (hors studio), puis le studio, puis le contrat Odin du host
make quality             # fmt + clippy + tests
make wasm/check          # le moteur compile pour wasm32-unknown-unknown, avec et sans Odin
make test/determinism    # snapshots natif = wasm (Node, CLI wasm-bindgen de la version de Cargo.lock)
```

## Studio : écouter et valider un son

Le studio est l'outil pour essayer le moteur à l'oreille et à l'œil : choisir un style, bouger les
émotions, voir les notes publiées et entendre le résultat à la mesure suivante.

```bash
make studio   # natif ; recompile, donc embarque les profils de profiles/ tels quels
make studio/web/serve   # web sur http://127.0.0.1:8787 (trunk requis)
```

Version publique : <https://harmonium-website.pages.dev/> (page d'accueil) et
<https://harmonium-website.pages.dev/studio/> (le studio web) ; `make studio/site/deploy` la met à jour.

- **Banques** : `assets/GeneralUser-GS.sf2` (`make soundfont`) et `assets/harmonium_trad.sf2` (percussion) ;
  `STUDIO_BANKS` change le dossier. **Presets Odin** : `STUDIO_ODIN` désigne un dossier de `*.odin` à plat
  (nom du preset = nom du fichier) ; par défaut `assets/odin/` quand il existe (`STUDIO_ODIN=` pour s'en
  passer). Sans presets, les pistes Odin des profils (Isaac, `nt_desert`) jouent leur programme SoundFont,
  avec un avertissement au démarrage. `assets/odin/` est local et non versionné : des liens vers les
  presets d'usine d'Odin 2 (`Soundbanks/Factory Presets/`).
- **Écran Live** :
  - en haut : style, graine, lecture, enregistrement ;
  - à gauche : émotions (pavé valence × arousal, densité, tension), mode direct, percussion, tempo, swing,
    signature, « Reset engine » ;
  - au centre : forme, accords, piano-roll par piste ;
  - à droite : horloge, état (équivalent de `state` du REPL), mixeur (gain, mute, solo).
  - Un clic sur une mesure y saute ; glisser fait une boucle (16 mesures au plus).
  - Une piste écartée par la densité du profil (`density_tracks`) est grisée : « off: density < x ».
- **Valider un réglage de profil** : onglet **Workbench**, sans recompiler. « Fork » copie un profil distribué
  dans un brouillon ; chaque fichier TOML s'édite dans son onglet, une erreur s'affiche sur sa ligne ; « Apply »
  installe le profil, qui s'entend à la mesure suivante. Natif : « Profile folder » ouvre et enregistre un dossier
  (relu tel quel par `harmonium-cli --profile`). Web : téléverser un dossier ou un `.zip`, télécharger le dossier en
  `.zip` ; le brouillon reste dans le navigateur. Pour un profil distribué : modifier `profiles/<id>/`, relancer
  `make studio`. En natif, le repli « Engine » de l'écran Live charge aussi un dossier de profil hors du dépôt.
- **Garder une trace** : REC (WAV, MIDI, MusicXML) écrit dans `~/Music/Harmonium Studio/`, nommé par profil et
  graine ; la graine affichée rejoue la même musique.
- **Liens d'ouverture** : le studio s'ouvre sur un état précis par un lien, comme avec `xdg-open`.
  `harmonium://open?profile=<id ou dossier absolu>&seed=3&bar=17&loop=17-24&bpm=180&swing=0.6&solo=Bass&mute=Melody,Chords&play=0`
  (tout est facultatif ; les noms sont ceux du REPL et du CLI). Un `id` de profil distribué ouvre le dossier
  `profiles/<id>` du dépôt quand il existe (l'édition se recharge sans recompiler), sinon le profil compilé ; un
  chemin absolu ouvre n'importe quel dossier de profil (worktree, fixture). **En pause par défaut** : aucun son
  avant `play=1` ou le bouton lecture. Un lien est un état entier : sans `solo`, `mute` ni `loop`, les pistes sont
  rendues et la boucle levée ; sans `bar`, la lecture part de la mesure 1 (ou du début de la boucle).
  ```bash
  make studio/open PROFILE=~/orca/workspaces/harmonium/a2-x/profiles/reel_quebecois SEED=3 BAR=17   # le studio qui tourne le prend, sinon il se lance
  make studio/open URL='harmonium://open?profile=reel_quebecois&seed=3&bar=17&loop=17-24'
  target/release/harmonium_studio open --profile profiles/reel_quebecois --seed 3 --bar 17
  make studio/install-handler   # les liens harmonium://… deviennent cliquables (macOS, Linux) ; studio/uninstall-handler les retire
  ```
  Le canal est un port TCP local (`127.0.0.1:47820`, `--control <adresse>` pour un autre), sans autre accès que
  l'ouverture d'un lien. Web : les mêmes paramètres en requête d'URL pour les profils compilés
  (`…/studio/?profile=valse_quebecoise&seed=3&bar=9`). Détails : [`harmonium_studio/README.md`](harmonium_studio/README.md).
- **Rechargement à chaud** (natif) : le studio surveille `profile.toml` et les banques de son `[files]` du dossier
  chargé (lien, ou « Profile folder » → « Load »). Un changement se recharge après 300 ms de repos ; la graine, la
  mesure, la boucle, les solos et mutes, le tempo et le swing imposés sont gardés. Un profil invalide ne remplace
  rien : l'erreur (`fichier:ligne: champ: message`) s'affiche en rouge, le précédent continue de jouer. L'heure du
  dernier rechargement est dans la barre du haut (« reloaded hh:mm:ss »).
- **« Copy link »** (barre du haut) copie le lien de l'état présent (profil, graine, mesure, boucle, tempo et swing
  imposés, solo, mutes) : celui qui l'ouvre entend la même chose. Un brouillon de l'atelier n'a pas de lien tant
  qu'il n'est pas enregistré dans un dossier et chargé.
- **Session d'agent** : le studio et le REPL sortent du son. Seuls l'orchestrateur et le propriétaire ouvrent des
  liens, en pause par défaut ; les agents de tâche ne lancent jamais le studio (ni `make studio/open`) et vérifient
  hors ligne, en silence : rendu hors ligne, par exemple
  `cargo run -p harmonium-cli --features odin -- --export --bars 8 --style isaac_basement -s assets/GeneralUser-GS.sf2 --odin-presets assets/odin --record-wav /tmp/essai.wav`,
  ou `LiveHost::detached` dans un test.

Détails (manifeste, cache web, parité avec le REPL) : [`harmonium_studio/README.md`](harmonium_studio/README.md).

## Architecture d'exécution

```mermaid
flowchart LR
  G[Jeu / REPL / OSC] -->|EngineCommand| H[LiveHost]
  H --> C[MusicComposer\nthread principal]
  C -->|Measure| P[PlaybackEngine\ncallback cpal]
  P -->|AudioEvent| S[oxisynth SF2]
  P -->|AtomicU64 bar,step| G
  C -->|MeasureSnapshot| G
```

Le composer génère des mesures d'avance ; le callback audio les joue pas à
pas et publie la position sans verrou. Les paramètres structurels prennent
effet à la mesure suivante ; gains, mute, programmes GM et seek sont
immédiats.

## Intégration dans un jeu

`harmonium::audio::create_offline_engine(sf2, percussion_sf2, sample_rate)` rend un
`MusicComposer` et un `PlaybackEngine` sans ouvrir de périphérique : le jeu
appelle `playback.process_buffer()` depuis son propre callback audio et
`composer.generate_ahead()` depuis sa boucle principale.

## Licence

Harmonium est distribué sous `MIT OR Apache-2.0`, au choix (`LICENSE-MIT`, `LICENSE-APACHE`).
L'option de compilation `odin` (synthé `odin2-core`, dérivé d'Odin 2) est sous GPLv3 : un binaire
compilé avec elle est GPL. Sans `odin`, le moteur rend en SoundFont et ne contient aucun code GPL.

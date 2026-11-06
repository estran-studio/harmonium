# harmonium_bevy dans l'engine : prêt pour 1837 ?

**Date** : 2026-11-06 | **Demande** : chef de cabinet du studio, accord de William (6 nov) | **Échéance** : 13 nov
**Engine** : `alacod` `022c7c7c`, Bevy `0.19.1`, lu et compilé dans des worktrees jetables (`/tmp`), **rien n'est commité dans `alacod`**
**Moteur** : `harmonium` `origin/main` = `a0b9a60` (ce que tire la dépendance git d'`alacod`), revérifié en natif sur `main` local `906956c` (phase 019, non poussée)

## En bref

- **Ça compile et ça joue**, en natif (macOS) et en wasm32 dans Chrome, dans le graphe de dépendances de l'engine. **Rien n'est à corriger côté harmonium** : `harmonium_bevy` est déjà en Bevy `=0.19.1` et n'entre en conflit avec aucune version de l'engine.
- **Ce qui bloque est côté `alacod`** :
  - le code derrière la feature `harmonium` appelle une API supprimée depuis longtemps (7 erreurs de compilation) ;
  - le `Cargo.lock` garde une ancienne révision de `harmonium_bevy`, et avec elle tout un arbre Bevy 0.17.3.
  
  Le correctif minimal tient dans 4 fichiers (§ 3). Il revient au coordinateur `alacod`, qui réactivera la feature.
- **Le multijoueur n'est pas prêt** : chaque client joue la musique sur sa propre carte son, à partir de son propre clic. Pour que deux joueurs en ligne entendent la même mesure au même moment, calée sur le temps de leur salle, il manque cinq chantiers (§ 5). Deux d'entre eux dépendent d'une décision de William : le principe II, et le choix de la source du tempo.

## 1. Ce qui a été vérifié

| Vérification | Résultat |
|---|---|
| `cargo check -p game --features harmonium` (dépendance remplacée par un chemin vers `harmonium` `a0b9a60`) | les dépendances se résolvent, `harmonium_bevy` compile ; **7 erreurs, toutes dans le code d'`alacod`** (§ 2) |
| `Cargo.lock` après résolution | seuls 3 paquets s'ajoutent (`harmonium`, `rosc`, `serde_path_to_error`) ; 207 paquets de l'ancien arbre Bevy 0.17 disparaissent ; **aucune version partagée avec l'engine ne change** |
| Natif : `zombies --features native,game/harmonium`, correctif minimal, profil `reel_quebecois`, volume 0 | banques chargées et hôte prêt 0,45 s après le démarrage ; « composition mesures 1–16 » ; `MusicClock` valide, `playing`, une mesure `Bar` par mesure à 230,9 BPM pendant toute la partie |
| Natif avec `main` local (019, 32 profils) | compile, l'hôte démarre et compose |
| wasm32 : `zombies` `--target wasm32-unknown-unknown --no-default-features --features render,game/harmonium`, `RUSTFLAGS=--cfg=web_sys_unstable_apis`, `wasm-bindgen` 0.2.129 (version de `make dep_web`) | compile ; page servie en local, **Google Chrome** (vrai Chrome, profil jetable) |
| Chrome : chargement et lecture | `GeneralUser-GS.sf3` (7,6 Mo, release `studio-resources-2`) et `harmonium_trad.sf2` (0,15 Mo) chargés par l'`AssetServer` du jeu ; aucune erreur. Après un clic dans le canvas, les mesures 1 → 30 en 30 s à 230,9 BPM, `playing true valid true`, avec les effets `bevy_kira_audio` actifs à côté |
| `bevy_kira_audio` (bruitages) et cpal (musique) dans le même processus | coexistent, en natif comme dans Chrome |

Aucune écoute : le volume de la musique était à 0, conformément à la règle des agents. C'est l'horloge, publiée par le callback audio, qui prouve que la sortie tourne.

## 2. L'API que le jeu appelle

**Aujourd'hui**, le code derrière `cfg(feature = "harmonium")` appelle une API supprimée du moteur :

| Fichier `alacod` | Appel | État |
|---|---|---|
| `crates/game/src/audio/mod.rs` | `HarmoniumPlugin` (struct unité), `components::{HarmoniumSource, GenerativeConfig, OdinConfig}`, `harmonium_core::sequencer::RhythmMode::Euclidean` | supprimés |
| `crates/game/src/audio/adaptive.rs` | `HarmoniumSource.config` (tempo, densité, tension), `RhythmMode` | supprimés |
| `crates/game/src/character/enemy/create.rs` | `components::HarmoniumTag` | supprimé |
| `crates/game/src/character/input_devices.rs` | `components::AiDriver` | supprimé |
| `crates/game/Cargo.toml` | `git = "https://github.com/bascanada/harmonium", branch = "main"` | redirige vers `estran-studio/harmonium` ; branche flottante |

**L'API actuelle** est celle du README de `harmonium_bevy`. Le jeu s'en sert seulement en présentation :

- **Installation** : `HarmoniumPlugin::new(HarmoniumConfig { main_bank, percussion_bank, odin_presets, start_paused, .. })`. Les chemins passent par l'`AssetServer`.
- **Cibles, à écrire dans `Update`** : `HarmoniumControl { profile, emotions: [arousal, valence, density, tension], volume, paused, seed }` et `set_profile_id("…")`. Le plugin les applique dans `PostUpdate` (`HarmoniumSet`), avec un fondu en cas de changement de profil.
- **Lecture** :
  - `HarmoniumStatus::{Loading, Ready, Failed}` ;
  - `MusicClock { valid, playing, bar, beat, pulse, pulses_per_bar, phase, bpm }` ;
  - les messages `Beat { bar, pulse }` et `Bar { bar }`.
  
  Tout cela vient du playhead audio de la machine locale : **ce n'est pas un temps de simulation**.
- **Accès direct** : `HarmoniumRuntime::host()` et `host_mut()`, non-Send.
- **Web** : l'hôte s'ouvre en pause. Le premier `pointerdown`, `keydown` ou `touchend` reprend l'`AudioContext` dans le geste. Un Worker Blob relance le pompage toutes les 100 ms, d'où le besoin de `worker-src blob:` dans la CSP.

## 3. Correctif minimal côté `alacod` (proposé, non appliqué)

C'est ce qui a tourné dans les deux essais, sans les deux lignes propres à la fumée : le volume à 0 et le journal à chaque mesure.

1. Dans `crates/game/Cargo.toml` :
   - `harmonium_bevy = { git = "https://github.com/estran-studio/harmonium", rev = "<commit>", optional = true }` : pin sur une révision plutôt que `branch = "main"`, et URL directe ;
   - retirer les deux commentaires périmés (« encore sur bevy 0.17 », « Désactivée tant que… »).
2. `cargo update -p harmonium_bevy` : le lock perd l'arbre Bevy 0.17.3 (§ 1).
3. Dans `crates/game/src/audio/mod.rs` :
   - supprimer `adaptive.rs` et `setup_harmonium_music` ;
   - `add_harmonium` devient :

   ```rust
   let mut control = HarmoniumControl::default();
   control.set_profile_id("reel_quebecois").expect("profil distribué"); // choix du jeu
   app.insert_resource(control).add_plugins(HarmoniumPlugin::new(HarmoniumConfig {
       main_bank: Some(if cfg!(target_arch = "wasm32") { "harmonium/GeneralUser-GS.sf3" } else { "harmonium/GeneralUser-GS.sf2" }.into()),
       percussion_bank: Some("harmonium/harmonium_trad.sf2".into()),
       ..default()
   }));
   ```
4. Retirer les blocs `HarmoniumTag` (`enemy/create.rs`) et `AiDriver` (`input_devices.rs`) : rien ne les lit plus.
5. Assets :
   - `GeneralUser-GS.sf3` et sa licence (`GeneralUser-GS-LICENSE.txt`) viennent de la release `studio-resources-2` d'`estran-studio/harmonium` ; `harmonium_trad.sf2` est dans `harmonium/assets/`. Ce sont les mêmes fichiers que le studio web.
   - Le jeu web les sert sous `/builds/<version>/<jeu>/assets/harmonium/`.
   - Le build web doit ajouter `game/harmonium` aux features.

Sans la feature `odin` (le défaut), aucun code GPL n'entre dans le jeu.

## 4. Points relevés en route

- **Le chargeur web du site** (`website/src/lib/components/GameLoader.svelte`) ouvre le jeu dans une iframe `blob:`. Dans cette iframe, Chrome refuse **tous** les `fetch` d'assets du jeu (« Failed to parse URL from /builds/… »), y compris les sprites et `.ron` de `zombies`, pas seulement ceux d'harmonium. L'essai a donc servi la page sans iframe. Ce point ne relève pas d'harmonium : je le signale au coordinateur, sans l'avoir vérifié sur le site déployé.
- **Fil principal bloqué** : en natif, la génération du niveau de `zombies` a gelé les frames Bevy 8,4 s. La réserve de 16 mesures du plugin a couvert le trou (16,7 s à 230 BPM, 32 s à 120 BPM).
  - Sur le web, cpal s'exécute sur le fil de la page [INFERENCE] : un tel gel peut couper le son. Non mesuré, faute d'écoute.
- **Taille du binaire** : le wasm de débogage de `zombies` (jeu complet) fait 254 Mo. Le surcoût d'harmonium dans le wasm de production n'a pas été mesuré.
- **GitHub** : `main` (`a0b9a60`) porte déjà le plugin Bevy 0.19.1, mais pas la phase 019. Les 32 profils et l'habillage par graine attendent le push que William doit décider.

## 5. Ce qui manque pour deux joueurs en ligne, calés sur le tempo de leur salle

**Objectif** : la simulation (GGRS, 60 Hz, `SIM_FPS`) décide du temps musical ; chaque client génère et joue sa musique ; les deux entendent la même mesure au même instant de partie, à quelques millisecondes près. C'est la décision A de la phase 013, étudiée dans `horloge-etude.md`.

**Aujourd'hui** : chaque client démarre au premier clic de son joueur. Les deux musiques sont donc décalées d'un temps arbitraire, souvent plusieurs secondes, puis dérivent l'une par rapport à l'autre (de 0,6 à 6 ms par minute selon les quartz [INFERENCE], § 6 de l'étude). Un changement d'émotion s'applique à « la mesure suivante » du playhead de chaque client : à des mesures différentes selon la machine.

### La source du temps (§ 8 de l'étude) : à choisir

| | Temps de la salle | Ce que la musique perd | Ce qui existe |
|---|---|---|---|
| **a. Horloge de l'engine** : `Clock` à événement répété des conventions `alacod` § 23 (120 BPM = 30 frames) | la simulation fixe le tempo ; la musique le suit | le tempo d'émotion et l'arc de set ; et le tempo ne peut valoir que 3 600 / *n* BPM (225 et 240 existent, pas 230) | horloge d'engine ; `SetBpm` du moteur (non exposé par le plugin) |
| **b. Plan partagé d'harmonium** : `SharedTimePlan` dans la simulation (option 1 de l'étude) | graine, profil et commandes datées donnent le même plan sur chaque client | rien | prototype `harmonium_core::time_plan`, testé sur tous les profils distribués ; le générateur tourne sans rendu dans la simulation |
| b'. Le même avec un squelette de forme séparé (option 2, J1b) | idem, en entiers de bout en bout | rien | reporté (`j1b-note.md`) ; change le flux, bump de `TUNING_VERSION` |

Pour l'alpha, où les deux joueurs sont dans Chrome, donc deux clients wasm32, le défaut de portabilité 32/64 bits du RNG (constat 4 de l'étude) ne joue pas. Il jouerait entre un client natif et un client web.

### Les chantiers manquants

| # | Manque | Où | Ce qu'il faut |
|---|---|---|---|
| M1 | **Le temps de la salle n'arrive pas au plugin** | `harmonium_bevy` | une entrée « instant de partie » : frame de simulation, frame d'ancrage F0, et en option b le plan (ou ses commandes datées). En option a : `bpm` dans `HarmoniumControl` et tempo d'émotion coupé |
| M2 | **Ancrage audio ↔ simulation** : chaque client démarre à son clic | `harmonium_host` + plugin | au geste, jouer l'instant de plan qui correspond à la frame courante : saut à la mesure suivante du plan, attente de son instant mur, latence de sortie comprise (horodatage cpal ; sur le web, `outputLatency`). Une reprise après un onglet en arrière-plan fait de même |
| M3 | **Correction de dérive non branchée** | `harmonium_host` | brancher `time_plan::correction` (zone morte 10 ms, ±1 à 5 ‰ sur 8 pulses, sinon saut à la mesure ; § 6 de l'étude) par une `PlaybackCommand::Nudge`. **Les seuils se règlent à l'oreille** : écoute du propriétaire |
| M4 | **Commandes appliquées « maintenant »** | composer (J2) | émotions et tempo datés par la simulation (« mesure courante + 1 » au plus tôt), appliqués à la génération de leur mesure, avec points de reprise pour une commande en retard. Pour l'alpha, une alternative : profil et émotions figés pendant une salle ; les changements ne se font qu'entre deux salles, datés sur une frontière de mesure |
| M5 | **Contrat de lecture côté jeu** | `alacod` + principe II | la simulation lit `PlanPosition` (option b) ou sa propre `Clock` (option a) pour juger « sur le temps » ; `MusicClock` reste la vue de ce que le joueur entend, pour le rendu |

**Principe II** : l'option b fait lire au jeu un temps qui ne vient pas du playhead. Le texte actuel du principe dit que « la position lue par le jeu vient du playhead ». L'amendement proposé au § 10 de `horloge-etude.md` attend la décision de William. Ce rapport **n'amende rien**.

**Ordre suggéré pour le 20 déc** : réactiver la feature (§ 3, une salle et un profil) → M1 et M2 → M3 → M4 en version alpha (rien ne change pendant une salle). Le détail est dans les propositions d'`alacod` (`propositions.md`, entrées datées du 2026-11-06), sur la branche locale `harmonium/pret-pour-1837` d'`alacod` (commit `c85b1dc0`, depuis `022c7c7c`, non poussée).

## Reproduire

- Worktrees jetables : `alacod` `022c7c7c` → `/tmp/harmonium-1837/alacod` ; `harmonium` `a0b9a60` → `/tmp/harmonium-1837/harmonium`.
- Natif : `CARGO_TARGET_DIR=/tmp/harmonium-1837/target APP_VERSION=dev cargo build -p zombies --features native,game/harmonium`, puis `ALACOD_MUTE=1 zombies --local-port 7011 --players localhost` depuis `games/zombies`.
- Web : même build avec `--target wasm32-unknown-unknown --no-default-features --features render,game/harmonium`, `RUSTFLAGS=--cfg=web_sys_unstable_apis`, `wasm-bindgen --target web --out-name wasm`. Assets copiés par `scripts/web-assets.mjs` sous `builds/dev/zombies/assets`, page servie sans iframe.

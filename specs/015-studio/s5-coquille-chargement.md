# S5 — Coquille, chargement, cache

Branche `wquintal/s5-coquille-chargement` (depuis `c2c16b7` : S2, S3 et S4 fusionnées). Plan : `specs/015-studio/plan.md`,
§ « Chargement et cache ». Crate : `harmonium_studio/` (README : usage).

## Ce qui a changé

1. **Crate `harmonium_studio`** (membre du workspace ; eframe 0.34, `glow` ; toujours compilé avec `odin`, donc GPL).
   Une lib, deux binaires : `harmonium_studio` (l'application) et `studio_assets` (le générateur du manifeste). La feature
   `app` (par défaut) porte eframe et le moteur ; `--no-default-features` ne compile que le manifeste et le générateur, que
   `make studio/assets` lance sans compiler eframe.
   - `manifest.rs` : `assets.json` (`id`, `file` à empreinte, `bytes`, `sha256`), validation (rôles connus et uniques,
     banque principale présente, nom de fichier sans chemin portant les 12 premiers hex du `sha256`), rangement des
     contenus par rôle (`LoadedResources` : banque, pieds, presets Odin triés ; zéro preset est un cas normal).
   - `packager.rs` / `studio_assets` : sources → copies `<tige>.<12 hex>.<ext>` + `assets.json` ; retire les fichiers d'un
     manifeste précédent qui ne sont plus cités ; relit ce qu'il écrit avec le validateur du chargeur.
   - `loading.rs` : toute la logique pure du chargement (décision de cache, contrôle de taille, élagage, textes et avis de
     l'écran, progression pondérée, validation du tampon).
   - `platform/{native,web}.rs` : seules différences bureau / navigateur (chargeur, hôte partagé, génération, réglages,
     point d'entrée). `app.rs` : l'écran.
2. **Deux temps de chargement** : page HTML avec l'initialiseur Trunk (`initializer.js`, barre indéterminée quand `total`
   vaut 0, texte d'échec) ; écran egui (une ligne par ressource, barre globale, avis), « Preparing instruments… » peint
   une image avant l'analyse bloquante (`logic` construit l'hôte à l'image suivante), bouton Start, puis écran provisoire
   (profil, mesure et pulsation de `BeatInfo`, sélecteur sur `StyleProfile::distributed()` appliqué par
   `SetStyleProfile`).
3. **Chargeur web** : manifeste revalidé à chaque visite (`no-cache`), `fetch` en flux avec compteur d'octets (taille du
   manifeste, pas de `Content-Length` requis ; `no-store` : la banque n'est pas en double dans le cache HTTP), Cache
   Storage `harmonium-studio-resources` clé `resources/<fichier à empreinte>`, entrée mise au cache seulement à la taille du
   manifeste, élagage des entrées hors manifeste après un chargement réussi, pas de `persist()`. Cache absent ou en échec
   (lecture, `put`) : octets gardés en mémoire, avis « This device cannot keep the files, so the next visit downloads them
   again. » **Chargeur natif** : mêmes ressources lues sur disque dans un fil, même écran, pas de cache.
4. **Hôte et génération** : `open_host` (commun) construit `LiveHost::start_paused` avec banque, pieds et presets Odin,
   `SetWriteheadLookahead(16)`, `SetStyleProfile(reel_quebecois)` et un premier `pump` ; Start appelle `resume` dans le
   geste de clic (eframe exécute la logique dans le gestionnaire `pointerup`, lu dans `eframe-0.34.3/src/web/events.rs`).
   Natif : fil `studio-pump` toutes les 50 ms. Web : Worker (script en `Blob`) qui poste un message toutes les 100 ms ; le
   message appelle `LiveHost::pump` sur le fil de la page, qui avance aussi l'enregistreur (S3). Aucun repli sur
   `requestAnimationFrame` : si le Worker ne démarre pas, Start échoue avec un message.
5. **Changement d'API du host** : `LiveHost::start` / `start_paused` et `audio::create_timeline_stream` reçoivent
   `buffer_frames: Option<u32>` (taille de tampon fixe, `None` = défaut de la plate-forme). Seul appelant migré : le REPL
   (`None`). README du host mis à jour. Le studio web passe 2048 (recommandation S1) ; `?buffer=` / `--buffer` le changent
   (64 à 16 384).
6. **Hors périmètre touché** : deux avertissements clippy de `harmonium_audio/src/odin_voices.rs` (`derive(Default)`,
   `is_multiple_of`) corrigés : le studio active `odin` pour tout le workspace, ce qui les exposait à `cargo clippy
   --workspace`.
7. **Makefile** : `studio/assets`, `studio` (natif, release), `studio/web/build`, `studio/web/serve` (Trunk, port 8787).
   Sources des banques : `STUDIO_BANKS` (défaut : `assets/` du checkout principal, résolu par `git rev-parse
   --git-common-dir`, donc valable depuis un worktree), `STUDIO_ODIN`. `.gitignore` : `harmonium_studio/resources/` et
   `dist/`. **CI** : seuls les jobs `quality` et `test` changent (paquets système d'eframe : `libxcb-render0-dev
   libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev`) ; **non vérifié** (la CI n'a pas
   tourné ; liste tirée de la pratique d'eframe, `libwayland-dev` et `libgl1-mesa-dev` par précaution). Le job `wasm` ne
   construit pas le studio (S9).

## Dimensionnement de l'anticipation

Mesuré sur le moteur (hôte détaché, `test.sf2`, graine par défaut, 40 tampons de 2048 images, `pulse_period ×
pulses_per_bar` de `beat_info`, `SetBpm` aux bornes `min_bpm` / `default_bpm` / `max_bpm` du profil ; test jetable non
conservé) :

| | durée d'une mesure |
|---|---|
| la plus courte | 0,938 s (`gigue_acadienne`, `max_bpm` 192) |
| la plus longue | 3,000 s (`hornpipe_britannique_coloniale`, `min_bpm` 80) |
| à `default_bpm` | 1,000 s (`gigue_acadienne`) à 2,609 s (`hornpipe_britannique_coloniale`) |

- **16 mesures d'avance** (`LOOKAHEAD_BARS` = `MEASURE_WINDOW_CAP`, le maximum que tiennent l'anneau de mesures et la
  fenêtre de lecture ; la valeur par défaut du moteur est 4). La mesure en cours compte dans la fenêtre : la génération
  peut s'arrêter au moins 15 × 0,938 s ≈ **14 s** avant que la lecture ne manque de mesures, contre 3 × 0,938 s ≈ 2,8 s
  avec 4. Un onglet masqué ralentit les minuteurs d'une page à un réveil par seconde (comportement connu des navigateurs,
  non mesuré ici) : 14 réveils manqués au pire avant épuisement. Un gel complet (onglet suspendu) n'est pas couvert : la
  musique s'arrête à la fin des mesures composées et reprend au réveil.
- **Tick** : 100 ms (web), 50 ms (natif, celui du REPL) : 9 à 30 réveils par mesure côté web ; une commande est recomposée
  en 0,1 s au plus. L'enregistreur pompé veut un `pump` au moins toutes les ≈ 3 s : le tick en donne trente fois son besoin,
  trois fois même ralenti à 1 s.
- **Tampon web** : 2048 images (≈ 46 ms à 44,1 kHz), non mesuré par S5 ; `?buffer=` pour 1024 ou 4096 (zéro retard mesuré
  par S1 à ces deux tailles).
- Coût d'une recomposition de 16 mesures sur le fil de la page : **non mesuré** (S1 : `generate_ahead()` ≤ 0,5 ms pour la
  valeur par défaut de 4 mesures).

## Vérification

Machine : macOS arm64, `rustc 1.101.0-nightly`, `+stable` 1.99.1, Chrome for Testing 150.0.7871.24, Trunk 0.22.1, `wasm-bindgen`
0.2.108 (= `Cargo.lock`), `wasm-opt` 124 (`.tools` ignoré par git).

### Commandes

- `cargo test --workspace` : **84 suites, 617 réussis, 0 échec** ; goldens inchangés (`HARMONIUM_UPDATE_GOLDENS` jamais
  posée, aucun fichier de golden modifié). Nouveaux : 24 tests dans `harmonium_studio/tests/` (12 `manifest_tests`,
  12 `loading_tests`).
- `make wasm/check` : vert (avec et sans `odin`). `cargo check` / `clippy --target wasm32-unknown-unknown -p harmonium_studio` :
  vert, aucun avertissement du studio.
- `WASM_BINDGEN=$PWD/.tools/bin/wasm-bindgen make test/determinism` : **68 / 68 fichiers identiques**.
- `cargo clippy --workspace --lib --bins --message-format=short` : **149 lignes d'avertissement ici et sur `main`
  (`c2c16b7`, archive git)**, mêmes lignes distinctes une fois les numéros retirés (comparaison `uniq -c` identique) ;
  aucun avertissement dans `harmonium_studio`. Avant la correction de `odin_voices.rs` : 151 lignes (2 de plus).
- `cargo +nightly fmt --all -- --check` : propre.
- `cargo test -p harmonium_audio --features odin` : vert.
- `trunk build --release` (depuis `harmonium_studio/`, `PATH` avec `.tools/bin`) : vert. Wasm : **6 259 932 o** après
  `wasm-opt` de Trunk, 2 437 472 o en `gzip -9` (calculé). S1 annonçait 16,3 Mo pour sa sonde ; l'écart n'a pas été analysé.

### Natif

- `harmonium_studio --release` : la fenêtre s'ouvre, l'hôte se construit (banque 32 319 396 o et pieds 154 186 o chargés,
  « Preparing instruments… » puis Start). Un clic synthétique (`orca computer click`) sur Start affiche l'écran de jeu :
  « Reel québécois · Bar 1 · pulse 1 / 2 », sélecteur de style, pastilles de pulsation (capture lue).
- Un programme jetable (supprimé) a chargé les ressources par `start_loading`, ouvert l'hôte, lancé le fil de génération,
  `resume`, puis lu `beat_info` chaque seconde : mesures 1 à 4 de `reel_quebecois` (pulses 2/2), puis, après
  `SetStyleProfile(valse_quebecoise)`, mesures 5 à 9 avec 3 pulsations par mesure : **le changement de profil marche et le
  fil de génération alimente la lecture**. Ces deux essais ont joué sur la carte son ; l'écoute (qualité, absence de
  craquements) est **à vérifier par le propriétaire**.
- Limite de l'environnement : toute fenêtre eframe, même un exemple vide de 10 lignes, se ferme d'elle-même après 3 à 5 s
  (demande de fermeture reçue sans action) ; le sélecteur de style n'a donc pas pu être manipulé à la souris. Le clic sur
  Start n'a été obtenu qu'en cadençant la commande sur la fin du chargement.

### Web (Chromium 150, serveur local jetable avec journal des requêtes, débit et `Content-Length` réglables)

- **Premier chargement** : l'écran egui montre « Loading instruments », 25 % et « Instrument bank — downloading 9.0 / 32.3 MB »
  (débit limité à 2 Mo/s, **réponse sans `Content-Length`**), « Feet bank — waiting », « First visit: next visits load from this
  device. » ; puis « Instruments are ready. » et Start. Premier chargement à 4 Mo/s : banque en 8,7 s.
- **Start** : `AudioContext` créé en pause à la préparation (« suspended », 44 100 Hz), « running » après le clic, `currentTime`
  2,98 s après ≈ 3 s ; écran « Reel québécois · Bar 3 · pulse 2 / 2 ».
- **Rechargement** : le journal du serveur ne contient que la page, le JS, le wasm et `assets.json` ; l'API Performance ne
  liste aucune requête de banque ; Cache Storage garde les deux entrées. **Aucune requête réseau pour la banque.**
- **Nouvelle empreinte** (banque de pieds remplacée par `nt_desert.sf2`, 3 351 554 o, dans `dist/resources`) : un seul
  téléchargement (`nt_desert.3ee78dc2ec83.sf2`) ; Cache Storage ne contient plus que la banque principale et la nouvelle ;
  l'ancienne entrée est supprimée.
- **Cache Storage absent** (`window.caches` rendu `undefined` par un script d'initialisation) : tout se télécharge, l'avis
  « This device cannot keep the files, so the next visit downloads them again. » s'affiche avant et après Start, et la musique
  joue (`AudioContext` « running », `currentTime` 2,98 s).

### Non vérifié ou bloqué

- **Onglet masqué, génération par Worker : bloqué.** La fenêtre pilotée repassait en `visibilityState: hidden` (fenêtre
  recouverte) dès que l'outil rendait la main ; une page masquée ne peint pas, donc n'arrive pas jusqu'à Start, et les clics
  synthétiques n'y passent pas. Essayé : navigateur géré (rAF jamais déclenché), Chrome for Testing lancé à part avec profil
  dédié, `osascript … activate`, second onglet au premier plan (reste « visible »), `--autoplay-policy`. Les scripts
  d'initialisation qui enveloppent `AudioBufferSourceNode.start` ou `AudioNode.connect` (pour mesurer le niveau de sortie)
  ont été suivis de la mort de la page (« detached frame ») ; abandonnés, pas de boucle. Le principe (minuteur dans un
  Worker, 14 s de réserve) repose donc sur le raisonnement ci-dessus et sur S1, **pas sur une mesure**.
- **Échec de `put` (quota)** : code écrit (même avis, octets gardés), non exercé ; seul le cas « API absente » l'a été.
- Écran de la page HTML (progression du wasm, barre indéterminée, échec) : initialiseur généré par Trunk (`total` = taille
  du fichier connue de Trunk), **non observé** à l'écran (le wasm local se charge en quelques ms).
- Firefox, Safari : non essayés. Safari exige l'appel à `resume()` dans le gestionnaire de clic : c'est le cas
  (`pointerup` exécute la logique d'eframe dans le geste), non vérifié.
- Le chargeur ne recalcule pas le `sha256` (contrôle de taille seulement, comme le plan ; le `sha256` sert aux S8/S9 et à la
  forme du nom de fichier).
- Temps jusqu'à Start et analyse de la banque en wasm : non chronométrés (les seuils de S1 restent à vérifier en S8).

## Reste ouvert / pour S6, S8, S9

- S6 remplace l'écran provisoire ; les `MeasureSnapshot` rendus par `pump` sont aujourd'hui ignorés.
- S8 : changer de banque = autre fichier source passé à `studio_assets` (même rôle `main-bank`), rien dans le chargeur ;
  presets Odin : `STUDIO_ODIN`.
- S9 : ajouter le studio (natif et wasm) à la CI ; valider la liste de paquets Linux ; `trunk build` demande `resources/`
  déjà généré (`make studio/assets`).

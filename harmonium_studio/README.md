# harmonium_studio

Application egui du moteur Harmonium, native et web (même code). Phase 015 : la coquille (S5 — chargement des ressources, cache, bouton Start) et l'écran Live (S6 — lecture, visualisation, parité avec le REPL). L'atelier de profils est la tâche S7.

Le studio ne lit que la sortie publiée du moteur (`MeasureSnapshot`, `BeatInfo`, `EngineReport`) et l'API publique de l'hôte, et ne le pilote que par `LiveHost::apply(EngineCommand)` et `LiveHost::pump()`. Il est toujours compilé avec `odin` : **un binaire ou un bundle du studio est GPL** (constitution 4.2.0).

## Hors des membres par défaut

Le studio est toujours compilé avec `odin`, et les options de Cargo s'unifient entre les paquets compilés ensemble : le mettre dans un `--workspace` ferait compiler le REPL, le labo et le banc avec Odin (donc GPL), et la configuration sans Odin de 1837 ne serait plus testée. Il est donc absent de `default-members` : `cargo build` et `cargo test` nus, et `--workspace --exclude harmonium_studio`, restent sans Odin ; le studio se construit et se teste par son nom (`-p harmonium_studio`, `make studio*`). `make test` et `make lint` enchaînent les deux configurations.

## Lancer

```sh
make studio                 # natif : prépare les ressources, puis ouvre la fenêtre (release)
make studio ARGS="--buffer 1024"
make studio/web/serve       # web : Trunk, http://127.0.0.1:8787
make studio/web/build       # web : build dans harmonium_studio/dist
```

Le web demande `trunk` et `rustup target add wasm32-unknown-unknown`, et un CLI `wasm-bindgen` de la version de `Cargo.lock` (Trunk télécharge le sien s'il ne trouve pas la bonne version dans le `PATH`) :
`cargo install --locked --root .tools trunk wasm-bindgen-cli --version <version de Cargo.lock>`, puis `PATH=$PWD/.tools/bin:$PATH`.

Réglages :

| | natif | web |
|---|---|---|
| tampon audio (images) | `--buffer <n>` (défaut : celui de la carte son) | `?buffer=<n>` (défaut : 2048) |
| dossier des ressources | `--resources <dossier>` (défaut : `harmonium_studio/resources`) | `resources/` à côté de la page |
| enregistrements | `--recordings <dossier>` (défaut : `~/Music/Harmonium Studio` sur macOS, sinon le dossier personnel) | téléchargements du navigateur |

Un tampon doit être compris entre 64 et 16 384 images.

## Ressources : le manifeste

`make studio/assets` (appelé par les cibles ci-dessus) lance `studio_assets`, qui copie les sources en fichiers **nommés par leur empreinte** dans `harmonium_studio/resources/` (ignoré par git) et écrit `assets.json` :

```json
{ "resources": [
  { "id": "main-bank", "file": "GeneralUser-GS.9575028c7a1f.sf2", "bytes": 32319396, "sha256": "…" },
  { "id": "feet-bank", "file": "harmonium_trad.4dc8159837b4.sf2", "bytes": 154186, "sha256": "…" }
] }
```

Rôles (`id`) : `main-bank` (obligatoire), `feet-bank`, `odin:<nom court>` (un par preset `.odin`, zéro est normal). Le format de la banque n'est écrit nulle part : SF2 ou SF3, c'est le même rôle.

Sources, par variables du Makefile :

- `STUDIO_BANKS` : dossier de `GeneralUser-GS.sf2` et `harmonium_trad.sf2`. Défaut : `assets/` du checkout principal (aussi depuis un worktree).
- `STUDIO_ODIN` : dossier de presets `*.odin` ; vide par défaut (aucun preset n'existe encore dans le dépôt).

Autres sources (banque SF3, fichiers d'une release) : appeler directement
`cargo run -p harmonium_studio --no-default-features --bin studio_assets -- --main-bank <fichier> [--feet-bank <fichier>] [--odin-dir <dossier>] --out <dossier>` ; le manifeste et le chargeur ne changent pas.

## Chargement

1. **Page HTML** (Trunk, `initializer.js`) : progression du téléchargement du wasm, barre indéterminée sans taille connue, texte d'échec.
2. **Écran egui** : une ligne par ressource (« on this device » ou « downloading x / y MB »), barre globale, avis de première visite.
3. **« Preparing instruments… »** : l'analyse des banques bloque le fil ; le message est peint une image avant.
4. **Start** : le clic démarre la sortie audio (geste exigé par le navigateur) ; puis l'écran Live.

Web : le manifeste se revalide à chaque visite (`cache: no-cache`) ; chaque ressource est lue dans Cache Storage (cache `harmonium-studio-resources`, clé = `resources/<fichier à empreinte>`) si l'entrée a la taille du manifeste, sinon téléchargée en flux (`cache: no-store`, progression comptée en octets reçus, taille attendue du manifeste) et mise au cache **après** contrôle de sa taille ; après un chargement réussi, les entrées que le manifeste ne cite pas sont supprimées. Pas de `navigator.storage.persist()`. Cache absent ou refusé : les octets restent en mémoire, la musique joue, un avis le dit.

Natif : mêmes ressources lues sur disque par un fil, même écran, pas de cache.

## Génération

`Session::pump` (donc `LiveHost::pump()`) est appelé par un fil (natif, toutes les 50 ms) ou par un minuteur de Worker (web, toutes les 100 ms, indépendant de `requestAnimationFrame`), avec 16 mesures d'avance ; les mesures rendues entrent dans la partition de la session, que l'écran dessine. Le pompage fait aussi avancer l'enregistreur sans fil du web (besoin : au moins toutes les ~3 s). Détail et chiffres : `specs/015-studio/s5-coquille-chargement.md`.

## Écran Live

| Zone | Contenu |
|---|---|
| Barre du haut | onglets Live / Workbench (désactivé : S7), style (`StyleProfile::distributed()`), lecture / pause, tempo entendu (à la noire, tiré de la pulsation de `BeatInfo`) et signature de la mesure qui sonne, graine (affichée, saisie, « New seed »), enregistrement WAV / MIDI / MusicXML, « About » |
| Gauche | contrôle émotion / direct ; pavé valence × arousal ; curseurs arousal, valence, densité, tension, intensité (valeurs affichées) ; pieds 0/1/2 ; tempo nominal et « Reset BPM » ; tempo audible et rampe (« Apply tempo », « Clear tempo ») ; swing 0,5–0,75 ; signature ; « Reset engine » ; repli « Engine » : modules rythme / harmonie / mélodie, anticipation, saut dur ou doux à une mesure, et en natif dossier de profil et écoute OSC |
| Centre | 20 mesures : 4 jouées, la mesure en cours, celles composées d'avance (fond plus clair ; pas encore composées : fond sombre). Rangées : numéros, air (rang dans le set, tonalité et mode, de `TunePosition`), forme (partie et passe, « A·1 » : partie A, passe 1 ; couleur par partie), accords (`chord_changes` ; un nom à chaque changement, une pédale ne se répète pas), piano-roll par piste (Melody, Counter, Chords, Bass, Heel, Toe ; hauteurs relatives 3·2·3·2·1·1 ; piste sans note dans la fenêtre : nom grisé), tête de lecture. Une note se dessine selon son `Articulation` publiée : `Normal` barre pleine, `Accent` cerclée de blanc, `Detached` barre raccourcie, `Linked` bouts carrés, `Tied` bouts carrés raccrochée à la précédente par un trait (pas d'attaque), `Legato` attaque, arc depuis la note précédente et prolongement clair (le chevauchement), `Trigger` losange. Clic sur une mesure : saut (`Seek`) ; clic droit : saut, déplacement doux de la tête (`SeekPlayhead`), fin de boucle ; glisser : boucle (16 mesures au plus ; au-delà, refus affiché) |
| Droite | horloge (mesure, pulsation, phase, temps fort, prochaine pulsation) ; état (l'équivalent de `state` du REPL : style, pieds, mode, graine, tonalité et forme de la mesure qui sonne, tempo entendu, swing, pulsation, signature, mesure / temps / pas, accord, modules) ; mixeur : gain, Mute, Solo par piste (solo = couper les autres ; une sourdine à la main survit à la fin du solo), volume général, sourdine de la sortie, routage par piste |
| Bas | avis : enregistrement écrit (chemin) ou téléchargé, commande refusée par l'hôte |

Les gains choisis valent pour le profil en cours : un changement de profil rend à chaque piste le niveau du profil (c'est ce que fait le moteur), et l'écran le montre.

## Organisation

- `src/manifest.rs`, `src/packager.rs`, `src/loading.rs` : logique pure du chargement (testée dans `tests/`), sans eframe.
- `src/live.rs` : l'écran Live sans fenêtre. `Session` (l'hôte, la partition reçue de `pump`, ce que le studio a commandé sans que le moteur le publie : mixeur, boucle, enregistrements…), `Intent` (geste de l'écran → `EngineCommand`, exécuté par `LiveHost::apply`), `LiveView` (ce que l'écran dessine, tiré de la sortie publiée).
- `src/live_screen.rs` : l'écran Live dessiné par egui ; il lit une `LiveView` par image et rend des `Intent`.
- `src/platform/` : `native.rs` et `web.rs`, les seules différences entre bureau et navigateur (dont `SessionHandle`, `RecordingSink`, et en natif `start_osc`).
- `src/app.rs` : chargement, Start, puis l'écran Live.
- La feature `app` (par défaut) porte eframe et le moteur ; `--no-default-features` ne compile que le manifeste et `studio_assets`.

## Tests

Sans carte son ni fenêtre : `tests/live_tests.rs` pilote une `Session` sur un hôte détaché (`LiveHost::detached`) et regarde la sortie publiée, le signal rendu et les fichiers (dont : tempo, tonalité et mode viennent de ce qui sonne, jamais des restes de session ; chaque `Articulation`, le legato de la basse et les accords liés d'`isaac_basement` arrivent au piano-roll) ; `tests/live_screen_tests.rs` agit sur les vrais widgets avec `egui_kittest` (sélecteur de style, graine, curseur, clic et glisser sur les mesures, Solo, enregistrement, panneau About). Le rendu en pixels n'est pas testé : voir `specs/015-studio/s6-studio-live.md` pour la façon dont il a été relu.

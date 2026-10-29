# harmonium_studio

Application egui du moteur Harmonium, native et web (même code). Phase 015 : la coquille (S5 — chargement des ressources, cache, bouton Start), l'écran Live (S6 — lecture, visualisation, parité avec le REPL) et l'atelier de profils (S7 — onglet Workbench).

Le studio ne lit que la sortie publiée du moteur (`MeasureSnapshot`, `BeatInfo`, `EngineReport`) et l'API publique de l'hôte, et ne le pilote que par `LiveHost::apply(EngineCommand)` et `LiveHost::pump()`. Il est toujours compilé avec `odin` : **un binaire ou un bundle du studio est GPL** (constitution 4.2.0).

## Hors des membres par défaut

Le studio est toujours compilé avec `odin`, et les options de Cargo s'unifient entre les paquets compilés ensemble : le mettre dans un `--workspace` ferait compiler le REPL, le labo et le banc avec Odin (donc GPL), et la configuration sans Odin de 1837 ne serait plus testée. Il est donc absent de `default-members` : `cargo build` et `cargo test` nus, et `--workspace --exclude harmonium_studio`, restent sans Odin ; le studio se construit et se teste par son nom (`-p harmonium_studio`, `make studio*`). `make test` et `make lint` enchaînent les deux configurations.

## Lancer

```sh
make studio                 # natif : prépare les ressources, puis ouvre la fenêtre (release)
make studio ARGS="--buffer 1024"
make studio/web/serve       # web : Trunk, http://127.0.0.1:8787
make studio/web/build       # web : build dans harmonium_studio/dist
make studio/site            # site public dans target/site : page d'accueil, studio sous /studio/
make studio/site/deploy     # et déploiement sur Cloudflare Pages
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
- `STUDIO_ODIN` : dossier de presets `*.odin` ; vide par défaut (aucun preset n'est versionné).

Autres sources : appeler directement
`cargo run -p harmonium_studio --no-default-features --bin studio_assets -- --main-bank <fichier> [--feet-bank <fichier>] [--odin-dir <dossier>] --out <dossier>` ; le manifeste et le chargeur ne changent pas.

## Ressources de la vitrine : une release fixée

Le site public ne prend pas les banques locales : `make studio/web/resources` (appelé par `make studio/site`) lit `web-resources.json`, qui fixe une release GitHub du dépôt (`studio-resources-1`) et, pour chaque fichier, son rôle, son nom, son nom d'asset, sa taille et son SHA-256 :

| Rôle | Fichier | Source, licence |
|---|---|---|
| `main-bank` | `GeneralUser-GS.sf3` (7,6 Mo) | GeneralUser GS v2.0.3 (S. Christian Collins, licence fournie dans la release), convertie de SF2 par `scripts/sf2_to_sf3.py` (oggenc qualité 4 ; même entrée, même fichier) |
| `feet-bank` | `harmonium_trad.sf2` | banque de pieds du labo, échantillons CC0 (Freesound) |
| `odin` | les presets nommés par `[[odin.<piste>.zones]] preset` dans `profiles/` | presets d'usine d'Odin 2 (`Soundbanks/Factory Presets/`, GPL-3.0) |
| `license` | `GeneralUser-GS-LICENSE.txt` | licence de la banque (non servie par le site) |

`scripts/web_resources.py fetch` télécharge chaque asset, contrôle taille et SHA-256 (un écart fait échouer la commande, donc la CI), et range les fichiers sous leur nom d'origine (le nom d'un preset Odin est son nom de fichier) ; `check-profiles` échoue si un profil nomme un preset absent du verrou. Changer une ressource : `scripts/web_resources.py stage --tag studio-resources-<n> …` copie les fichiers sous leur nom d'asset et réécrit le verrou, puis `gh release create studio-resources-<n> <dossier>/*`.

## Site public

`make studio/site` assemble `target/site` : `site/index.html` (page d'accueil, statique) à la racine et le studio web sous `/studio/` (`trunk build --public-url /studio/`). Il est servi par **Cloudflare Pages**, projet `harmonium-website` (<https://harmonium-website.pages.dev/>, studio : <https://harmonium-website.pages.dev/studio/>) :

- à la main : `make studio/site/deploy` (`wrangler login`, ou `CLOUDFLARE_API_TOKEN` et `CLOUDFLARE_ACCOUNT_ID`) ;
- par la CI (`.github/workflows/site.yml`) : chaque PR construit le site avec les ressources vérifiées ; chaque push sur `main` le déploie, si le dépôt a le secret `CLOUDFLARE_API_TOKEN` (jeton « Cloudflare Pages : Edit ») et la variable `CLOUDFLARE_ACCOUNT_ID` ; sans eux, le job le signale par un avertissement et ne déploie pas.

Cloudflare Pages limite un fichier à 25 Mio : le wasm (≈ 19 Mo) et la banque SF3 passent, pas la banque SF2 (32 Mo).

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
| Barre du haut | onglets Live / Workbench, style (`StyleProfile::distributed()` ; le nom affiché est celui du profil installé, un brouillon de l'atelier compris), lecture / pause, tempo entendu (à la noire, tiré de la pulsation de `BeatInfo`) et signature de la mesure qui sonne, graine (affichée, saisie, « New seed »), enregistrement WAV / MIDI / MusicXML, « About » |
| Gauche | contrôle émotion / direct ; pavé valence × arousal ; curseurs arousal, valence, densité, tension, intensité (valeurs affichées) ; pieds 0/1/2 ; tempo nominal et « Reset BPM » ; tempo audible et rampe (« Apply tempo », « Clear tempo ») ; swing 0,5–0,75 ; signature ; « Reset engine » ; repli « Engine » : modules rythme / harmonie / mélodie, anticipation, saut dur ou doux à une mesure, et en natif dossier de profil et écoute OSC |
| Centre | 20 mesures : 4 jouées, la mesure en cours, celles composées d'avance (fond plus clair ; pas encore composées : fond sombre). Rangées : numéros, air (rang dans le set, tonalité et mode, de `TunePosition`), forme (partie et passe, « A·1 » : partie A, passe 1 ; couleur par partie), accords (`chord_changes` ; un nom à chaque changement, une pédale ne se répète pas), piano-roll par piste (Melody, Counter, Chords, Bass, Heel, Toe ; hauteurs relatives 3·2·3·2·1·1 ; piste sans note dans la fenêtre : nom grisé), tête de lecture. Une note se dessine selon son `Articulation` publiée : `Normal` barre pleine, `Accent` cerclée de blanc, `Detached` barre raccourcie, `Linked` bouts carrés, `Tied` bouts carrés raccrochée à la précédente par un trait (pas d'attaque), `Legato` attaque, arc depuis la note précédente et prolongement clair (le chevauchement), `Trigger` losange. Clic sur une mesure : saut (`Seek`) ; clic droit : saut, déplacement doux de la tête (`SeekPlayhead`), fin de boucle ; glisser : boucle (16 mesures au plus ; au-delà, refus affiché) |
| Droite | horloge (mesure, pulsation, phase, temps fort, prochaine pulsation) ; état (l'équivalent de `state` du REPL : style, pieds, mode, graine, tonalité et forme de la mesure qui sonne, tempo entendu, swing, pulsation, signature, mesure / temps / pas, accord, modules) ; mixeur : gain, Mute, Solo par piste (solo = couper les autres ; une sourdine à la main survit à la fin du solo ; une piste écartée par la densité du profil, `density_tracks`, est grisée, avec « off: density < x » au survol et au bout de sa rangée du piano-roll), volume général, sourdine de la sortie, routage par piste |
| Bas | avis : enregistrement écrit (chemin) ou téléchargé, commande refusée par l'hôte, actions de l'atelier |

Les gains choisis valent pour le profil en cours : un changement de profil rend à chaque piste le niveau du profil (c'est ce que fait le moteur), et l'écran le montre.

## Atelier (Workbench)

Un **brouillon** : les fichiers d'un profil tels qu'ils sont sur disque (`profile.toml`, puis les banques de son `[files]` dans leur ordre ; commentaires et découpe gardés). Il se décode à chaque frappe par le décodeur du moteur (`StyleProfile::from_files`, celui de `load` et de `harmonium-cli --profile` : 0,5 à 2,2 ms par décodage en natif release pour les profils distribués).

| Zone | Contenu |
|---|---|
| Gauche | le brouillon (nom, origine, « Valid profile » ou erreur, « Apply », appliqué ou non) ; natif : « Profile folder », « Open », « Save » ; web : « Upload… », « Download .zip » ; les profils distribués, lisibles |
| Centre | en-tête (profil distribué : nom, « read-only », « Fork » ; brouillon : nom, ou l'erreur et « Go to line ») ; un onglet par fichier (le fichier fautif est marqué « (error) ») ; l'éditeur TOML : coloration (en-têtes, clés, chaînes, nombres, commentaires), numéros de ligne, ligne de l'erreur sur fond rouge |

- **Fork** : copie d'un profil distribué sous l'identifiant `<id>_fork` et le nom « <nom> (fork) » (seules ces deux lignes de `[meta]` changent), pour ne pas se confondre avec l'original. Il remplace le brouillon en cours.
- **Apply** : `EngineCommand::SetStyleProfile`, comme le sélecteur de style ; la mesure qui sonne finit, la suivante est composée avec le profil. Refusé tant que le brouillon ne se décode pas.
- **Erreur** : `fichier:ligne: champ: message` du décodeur ; « Go to line » (et un Apply refusé) ouvre l'onglet du fichier et amène la ligne au milieu de la vue.
- **Natif** : « Open » lit un dossier (ou son `profile.toml`) et les banques de son `[files]` ; « Save » écrit les fichiers du brouillon dans le dossier du champ (créé au besoin ; les autres fichiers du dossier restent ; un nom qui sortirait du dossier est refusé).
- **Web** : « Upload… » accepte une archive `.zip` (le `profile.toml` le moins profond fixe le dossier ; `__MACOSX/` ignoré ; Deflate lu) ou les fichiers d'un dossier choisis ensemble ; seuls les `.toml` et `.md` sont gardés. « Download .zip » donne `<id>.zip`, qui se décompresse en `<id>/`. Le brouillon est gardé dans `localStorage` (clé `harmonium-studio.workbench.draft`) à chaque modification et revient à la visite suivante ; un refus du navigateur est signalé une fois.

## Organisation

- `src/manifest.rs`, `src/packager.rs`, `src/loading.rs` : logique pure du chargement (testée dans `tests/`), sans eframe.
- `src/live.rs` : l'écran Live sans fenêtre. `Session` (l'hôte, la partition reçue de `pump`, ce que le studio a commandé sans que le moteur le publie : mixeur, boucle, enregistrements…), `Intent` (geste de l'écran → `EngineCommand`, exécuté par `LiveHost::apply`), `LiveView` (ce que l'écran dessine, tiré de la sortie publiée).
- `src/live_screen.rs` : l'écran après Start (barre du haut, avis, onglet Live dessiné par egui) ; il lit une `LiveView` par image et rend des `Intent`.
- `src/workbench.rs` : l'atelier sans fenêtre (`Draft`, `Draft::fork`, `Draft::decode`, archive, `toml_spans` pour la coloration). `src/workbench_screen.rs` : l'onglet Workbench dessiné par egui.
- `src/platform/` : `native.rs` et `web.rs`, les seules différences entre bureau et navigateur (dont `SessionHandle`, `RecordingSink` ; en natif `start_osc`, `read_profile_folder`, `write_profile_folder` ; en web `offer_download`, `pick_profile_files`, `load_draft`, `store_draft`).
- `src/app.rs` : chargement, Start, puis l'écran Live.
- La feature `app` (par défaut) porte eframe et le moteur ; `--no-default-features` ne compile que le manifeste et `studio_assets`.
- `site/` : la page d'accueil du site public ; `web-resources.json` : le verrou des ressources de la vitrine ; `scripts/` : `web_resources.py` (télécharger et vérifier, préparer une release) et `sf2_to_sf3.py` (conversion de la banque).

## Tests

Sans carte son ni fenêtre : `tests/live_tests.rs` pilote une `Session` sur un hôte détaché (`LiveHost::detached`) et regarde la sortie publiée, le signal rendu et les fichiers (dont : tempo, tonalité et mode viennent de ce qui sonne, jamais des restes de session ; chaque `Articulation`, le legato de la basse et les accords liés d'`isaac_basement` arrivent au piano-roll) ; `tests/live_screen_tests.rs` agit sur les vrais widgets avec `egui_kittest` (sélecteur de style, graine, curseur, clic et glisser sur les mesures, Solo, enregistrement, panneau About ; atelier : Fork, Save, Open, Apply entendu à la mesure suivante, erreur placée sur sa ligne qui bloque Apply) ; `tests/workbench_tests.rs` couvre le brouillon (copie, dossier relu par `StyleProfile::load`, archive décompressée puis téléversée, téléversement filtré, coloration). Le rendu en pixels n'est pas testé : voir `specs/015-studio/s6-studio-live.md` et `s7-atelier.md` pour la façon dont il a été relu.

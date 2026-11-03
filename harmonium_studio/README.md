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
| lien à ouvrir | `--open <lien>` ; sous-commande `open` (voir « Liens d'ouverture ») | requête de l'URL (`?profile=…&seed=…`) |
| canal de commande | `--control <adresse>` (défaut `127.0.0.1:47820`, local seulement) | absent |
| profils du dépôt | `--profiles <dossier>` (défaut : `profiles/` du dépôt qui a compilé le studio) | absent |

Un tampon doit être compris entre 64 et 16 384 images.

## Ressources : le manifeste

`make studio/assets` (appelé par les cibles ci-dessus) lance `studio_assets`, qui copie les sources en fichiers **nommés par leur empreinte** dans `harmonium_studio/resources/` (ignoré par git) et écrit `assets.json` :

```json
{ "resources": [
  { "id": "main-bank", "file": "GeneralUser-GS.9575028c7a1f.sf2", "bytes": 32319396, "sha256": "…" },
  { "id": "percussion-bank", "file": "harmonium_trad.4dc8159837b4.sf2", "bytes": 154186, "sha256": "…" }
] }
```

Rôles (`id`) : `main-bank` (obligatoire), `percussion-bank`, `odin:<nom court>` (un par preset `.odin`, zéro est normal). Le format de la banque n'est écrit nulle part : SF2 ou SF3, c'est le même rôle.

Sources, par variables du Makefile :

- `STUDIO_BANKS` : dossier de `GeneralUser-GS.sf2` et `harmonium_trad.sf2`. Défaut : `assets/` du checkout principal (aussi depuis un worktree).
- `STUDIO_ODIN` : dossier de presets `*.odin` ; vide par défaut (aucun preset n'est versionné).

Autres sources : appeler directement
`cargo run -p harmonium_studio --no-default-features --bin studio_assets -- --main-bank <fichier> [--percussion-bank <fichier>] [--odin-dir <dossier>] --out <dossier>` ; le manifeste et le chargeur ne changent pas.

## Ressources de la vitrine : une release fixée

Le site public ne prend pas les banques locales : `make studio/web/resources` (appelé par `make studio/site`) lit `web-resources.json`, qui fixe une release GitHub du dépôt (`studio-resources-1`) et, pour chaque fichier, son rôle, son nom, son nom d'asset, sa taille et son SHA-256 :

| Rôle | Fichier | Source, licence |
|---|---|---|
| `main-bank` | `GeneralUser-GS.sf3` (7,6 Mo) | GeneralUser GS v2.0.3 (S. Christian Collins, licence fournie dans la release), convertie de SF2 par `scripts/sf2_to_sf3.py` (oggenc qualité 4 ; même entrée, même fichier) |
| `percussion-bank` | `harmonium_trad.sf2` | banque de percussion du labo, échantillons CC0 (Freesound) |
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
| Gauche | contrôle émotion / direct ; pavé valence × arousal ; curseurs arousal, valence, densité, tension, intensité (valeurs affichées) ; percussion 0/1/2 ; tempo nominal et « Reset BPM » ; tempo audible et rampe (« Apply tempo », « Clear tempo ») ; swing 0,5–0,75 ; signature ; « Reset engine » ; repli « Engine » : modules rythme / harmonie / mélodie, anticipation, saut dur ou doux à une mesure, et en natif dossier de profil et écoute OSC |
| Centre | 20 mesures : 4 jouées, la mesure en cours, celles composées d'avance (fond plus clair ; pas encore composées : fond sombre). Rangées : numéros, air (rang dans le set, tonalité et mode, de `TunePosition`), forme (partie et passe, « A·1 » : partie A, passe 1 ; couleur par partie), accords (`chord_changes` ; un nom à chaque changement, une pédale ne se répète pas), piano-roll par piste (Melody, Counter, Chords, Bass, PercussionLow, PercussionHigh ; hauteurs relatives 3·2·3·2·1·1 ; piste sans note dans la fenêtre : nom grisé), tête de lecture. Une note se dessine selon son `Articulation` publiée : `Normal` barre pleine, `Accent` cerclée de blanc, `Detached` barre raccourcie, `Linked` bouts carrés, `Tied` bouts carrés raccrochée à la précédente par un trait (pas d'attaque), `Legato` attaque, arc depuis la note précédente et prolongement clair (le chevauchement), `Trigger` losange. Clic sur une mesure : saut (`Seek`) ; clic droit : saut, déplacement doux de la tête (`SeekPlayhead`), fin de boucle ; glisser : boucle (16 mesures au plus ; au-delà, refus affiché) |
| Droite | horloge (mesure, pulsation, phase, temps fort, prochaine pulsation) ; état (l'équivalent de `state` du REPL : style, percussion, mode, graine, tonalité et forme de la mesure qui sonne, tempo entendu, swing, pulsation, signature, mesure / temps / pas, accord, modules) ; mixeur : gain, Mute, Solo par piste (solo = couper les autres ; une sourdine à la main survit à la fin du solo ; une piste écartée par la densité du profil, `density_tracks`, est grisée, avec « off: density < x » au survol et au bout de sa rangée du piano-roll), volume général, sourdine de la sortie, routage par piste |
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

## Liens d'ouverture et rechargement à chaud

Les revues et les tests passent par le studio : un lien l'ouvre sur un profil, une graine, une mesure… et le studio recharge le profil quand ses fichiers changent. Seuls l'orchestrateur et le propriétaire ouvrent des liens ; **en pause par défaut** (aucun son avant `play=1` ou le bouton lecture) ; les agents de tâche ne lancent jamais le studio.

### Le lien

`harmonium://open?profile=<id ou dossier absolu>&seed=3&bar=17&loop=17-24&bpm=180&swing=0.6&solo=Bass&mute=Melody,Chords&play=0`

| Paramètre | Sens | Comme |
|---|---|---|
| `profile` | identifiant d'un profil distribué (`reel_quebecois`) ou **chemin absolu** d'un dossier de profil (`profile.toml` et ses banques : worktree, fixture). Un identifiant ouvre le dossier `profiles/<id>` du dépôt quand il existe (natif ; `--profiles <dossier>` en change la racine, par défaut celle du dépôt qui a compilé le studio), sinon le profil compilé. Absent : le profil qui joue reste | `--profile`, `--style` |
| `seed` | graine de session (`SetSeed`, repart de la mesure 1) | `--seed`, `set seed` |
| `bar` | mesure d'arrivée, recomposée depuis cette mesure avec la même graine (`Seek`, déterministe). Défaut : le début de la boucle, sinon 1 | `seek` |
| `loop` | `début-fin`, mesures 1-based, 16 au plus (l'hôte refuse plus long) | `loop` |
| `bpm`, `swing` | tempo (40 à 300) et swing (0,5 à 0,75) imposés. Absents : ceux du profil | `bpm`, `swing`, `--swing` |
| `solo` | une piste en solo (`Melody`, `Countermelody` ou `Counter`, `Chords`, `Bass`, `PercussionLow`, `PercussionHigh` ; la casse ne compte pas) | `--solo-track` |
| `mute` | pistes coupées, séparées par des virgules (le paramètre peut se répéter) | `set mute` |
| `play` | `1` : lancer la lecture. Défaut `0` : pause | |

Un lien est un **état entier** : sans `solo`, `mute` ni `loop`, les pistes sont rendues et la boucle levée ; sans `bar`, la lecture part de la mesure 1. Une clé inconnue, une valeur hors bornes ou deux fois la même clé (sauf `mute`) refuse le lien, avec la raison ; rien n'a changé. Les valeurs sont encodées en pourcentage (UTF-8) : un chemin avec des espaces s'écrit `%20`. Un chemin relatif est refusé (le studio ne tourne pas dans votre dossier) ; la commande `open` ci-dessous complète les chemins relatifs.

Web : les mêmes paramètres en requête de l'URL de la page, pour les profils compilés seulement (`https://…/studio/?profile=valse_quebecoise&seed=3&bar=9&solo=Bass`). Un dossier ne s'ouvre pas sur le web (pas de disque : onglet Workbench, « Upload… »). La page est en pause : « Start » ouvre l'écran sans lancer la lecture quand un lien attend, `play=1` lance après le clic. « Copy link » y copie l'adresse de la page, requête comprise. `?buffer=<n>` reste un réglage du studio, pas un paramètre de lien.

### Ouvrir un lien (natif)

```sh
harmonium_studio open 'harmonium://open?profile=/chemin/vers/profiles/reel_quebecois&seed=3&bar=17'
harmonium_studio open --profile profiles/reel_quebecois --seed 3 --bar 17 --loop 17-24 --solo Bass   # mêmes paramètres en options
make studio/open PROFILE=/chemin/vers/worktree/profiles/reel_quebecois SEED=3 BAR=17
make studio/open URL='harmonium://open?profile=reel_quebecois&seed=3'
```

Les options : `--profile <id|dossier>`, `--seed`, `--bar`, `--loop <a-b>`, `--bpm`, `--swing`, `--solo`, `--mute <pistes>`, `--play` ; un dossier qui existe (relatif ou `~/…`) devient son chemin absolu. Un lien ou des options, pas les deux. `make studio/open` prend `URL=` ou `PROFILE= SEED= BAR= LOOP= BPM= SWING= SOLO= MUTE= PLAY=1`.

La commande cherche un studio sur le canal de commande : s'il répond, elle lui transmet le lien (`opened in the running studio`), sinon elle **lance le studio** détaché avec `--open <lien>` (`started the studio`). Le studio lancé de cette façon saute le bouton Start (il n'existe que pour le geste qu'exige le navigateur) et arrive en pause, sur l'état du lien. Un studio lancé à la main avec `--open` à côté d'un studio qui tourne lui passe le lien et s'arrête. Codes de sortie : 0 ouvert, 1 refusé (la raison est imprimée : lien, profil invalide avec fichier et ligne, studio qui répond autre chose), 2 mal appelé.

### Le canal de commande

TCP sur **127.0.0.1:47820** (`--control <adresse>` change l'adresse, pour le studio et pour `open`). Le client envoie **une ligne**, un lien `harmonium://open?…`, et lit **une ligne** : `ok` (le profil est lu et validé, la demande est passée à l'écran) ou `error: <raison>` (rien n'a changé). Il n'y a pas d'autre message : ni lecture, ni état, ni commande du moteur. L'écoute est **locale seulement** : une adresse qui n'est pas de boucle locale est refusée à l'ouverture, et une connexion qui ne vient pas d'une adresse locale est ignorée ; la première ligne doit être un lien, donc une requête HTTP qu'une page web enverrait à ce port est refusée. Le module OSC de l'hôte (UDP, émotions seulement, sans réponse) ne dit pas si un studio écoute, d'où ce canal à part. Le canal s'ouvre au démarrage, avant le chargement des ressources : un lien envoyé pendant ces secondes attend l'écran Live. Port pris (un autre studio, un autre programme) : le studio démarre quand même et le dit dans un avis.

Quelques lignes suffisent à un client : `printf 'harmonium://open?seed=3\n' | nc 127.0.0.1 47820`.

### Rendre les liens cliquables

```sh
make studio/install-handler      # compile en release, puis enregistre le schéma pour CE binaire (target/release/harmonium_studio)
make studio/uninstall-handler
```

- **macOS** : une petite appli, `~/Applications/Harmonium Link Handler.app` (AppleScript `on open location`, compilée par `osacompile`, sans icône de Dock), déclare le schéma `harmonium` dans son `Info.plist` ; macOS lui remet l'URL par Apple Event (pas par `argv`), et elle appelle `harmonium_studio open '<lien>'`. Signature ad hoc refaite, enregistrement par `lsregister`. Vérifié : l'appli reçoit `harmonium://open?…` et appelle la commande avec le lien tel quel.
- **Linux** : `~/.local/share/applications/harmonium-studio-link.desktop` (`Exec="<binaire>" open %u`, `MimeType=x-scheme-handler/harmonium;`), puis `xdg-mime default harmonium-studio-link.desktop x-scheme-handler/harmonium`. Non exécuté sur cette machine (macOS) : le fichier se vérifie avec `xdg-mime query default x-scheme-handler/harmonium` et `xdg-open 'harmonium://open?seed=3'`.
- **Windows** : non fait. À la main, dans le registre : `HKCU\Software\Classes\harmonium` (valeur `URL Protocol` vide), `…\shell\open\command` = `"C:\chemin\harmonium_studio.exe" open "%1"`. Le canal TCP et la commande `open` sont portables (lancement détaché par `DETACHED_PROCESS`), mais n'ont pas été essayés sous Windows.
- Pour changer de binaire (un autre worktree, une autre compilation), réinstaller : `make studio/install-handler STUDIO_BIN=/chemin/vers/harmonium_studio`. Le script (`scripts/install_link_handler.sh`) accepte `--dest <dossier>` et `--no-register` pour contrôler le résultat sans toucher aux réglages du système.

### Rechargement à chaud

Quand le profil vient d'un dossier (lien avec un chemin ou un identifiant dont le dossier `profiles/<id>` existe, ou « Profile folder » → « Load » dans le repli « Engine »), le studio surveille `profile.toml` et les banques de son `[files]` (date et taille, toutes les 150 ms ; une liste qui change avec `[files]` est suivie). Un changement est relu après **300 ms de repos** : un éditeur qui écrit en plusieurs temps ou par renommage ne donne qu'un rechargement. Le profil est relu par `StyleProfile::load` (celui de `harmonium-cli --profile`).

- **Gardé** : graine, mesure (la tête de lecture ne bouge pas), boucle, solos et mutes, **tempo et swing imposés** (`SetStyleProfile` les remet au profil : le studio les réimpose, `Session::reload_profile`), niveaux choisis au mixeur. Ce que le profil installe reste celui du profil : signature, percussion, intensité, émotions.
- **Invalide** : le profil précédent continue de jouer ; la barre du haut montre en rouge `reload failed hh:mm:ss: <fichier>:<ligne>: <champ>: <message>` (même texte que `harmonium-cli --profile`) et un avis le répète. L'erreur n'est pas répétée tant que les fichiers ne changent pas ; la correction recharge.
- **Visible** : « Watching <dossier> » et « reloaded hh:mm:ss » (heure locale) dans la barre du haut.
- Choisir un style dans la liste, ou « Apply » dans l'atelier, remplace le profil : la surveillance s'arrête. Un profil compilé (identifiant sans dossier, web) n'est pas surveillé.

### « Copy link »

Le bouton de la barre du haut copie le lien de l'état présent : profil (le dossier absolu s'il vient d'un dossier, sinon l'identifiant distribué), graine, mesure sous la tête de lecture, boucle, tempo et swing imposés, solo, mutes ; pas de `play`. Un brouillon de l'atelier qui n'est ni distribué ni chargé d'un dossier n'a pas de lien : l'avis le dit (enregistrer le dossier, puis « Load »). Le lien copié rouvre la même musique : les mesures sont déterministes à profil et graine donnés.

## Organisation

- `src/manifest.rs`, `src/packager.rs`, `src/loading.rs` : logique pure du chargement (testée dans `tests/`), sans eframe.
- `src/live.rs` : l'écran Live sans fenêtre. `Session` (l'hôte, la partition reçue de `pump`, ce que le studio a commandé sans que le moteur le publie : mixeur, boucle, enregistrements…), `Intent` (geste de l'écran → `EngineCommand`, exécuté par `LiveHost::apply`), `LiveView` (ce que l'écran dessine, tiré de la sortie publiée).
- `src/live_screen.rs` : l'écran après Start (barre du haut, avis, onglet Live dessiné par egui) ; il lit une `LiveView` par image et rend des `Intent`.
- `src/workbench.rs` : l'atelier sans fenêtre (`Draft`, `Draft::fork`, `Draft::decode`, archive, `toml_spans` pour la coloration). `src/workbench_screen.rs` : l'onglet Workbench dessiné par egui.
- `src/platform/` : `native.rs` et `web.rs`, les seules différences entre bureau et navigateur (dont `SessionHandle`, `RecordingSink` ; en natif `start_osc`, `read_profile_folder`, `write_profile_folder` ; en web `offer_download`, `pick_profile_files`, `load_draft`, `store_draft`).
- `src/app.rs` : chargement, Start, puis l'écran Live.
- `src/link.rs` : les liens d'ouverture (`OpenRequest`, `ProfileRef`, `Opening` ; lecture, écriture, traduction en `Intent`), sans egui. `src/reload.rs` : `ProfileWatch`, la surveillance des fichiers d'un profil (dates et tailles, anti-rebond), sans egui ni moteur. `src/control.rs` (natif) : le canal de commande TCP local, `forward`, `launch` et la commande `open`. `scripts/install_link_handler.sh` : le gestionnaire du schéma `harmonium://` (macOS, Linux).
- La feature `app` (par défaut) porte eframe et le moteur ; `--no-default-features` ne compile que le manifeste et `studio_assets`.
- `site/` : la page d'accueil du site public ; `web-resources.json` : le verrou des ressources de la vitrine ; `scripts/` : `web_resources.py` (télécharger et vérifier, préparer une release) et `sf2_to_sf3.py` (conversion de la banque).

## Tests

Sans carte son ni fenêtre : `tests/live_tests.rs` pilote une `Session` sur un hôte détaché (`LiveHost::detached`) et regarde la sortie publiée, le signal rendu et les fichiers (dont : tempo, tonalité et mode viennent de ce qui sonne, jamais des restes de session ; chaque `Articulation`, le legato de la basse et les accords liés d'`isaac_basement` arrivent au piano-roll) ; `tests/live_screen_tests.rs` agit sur les vrais widgets avec `egui_kittest` (sélecteur de style, graine, curseur, clic et glisser sur les mesures, Solo, enregistrement, panneau About ; atelier : Fork, Save, Open, Apply entendu à la mesure suivante, erreur placée sur sa ligne qui bloque Apply) ; `tests/workbench_tests.rs` couvre le brouillon (copie, dossier relu par `StyleProfile::load`, archive décompressée puis téléversée, téléversement filtré, coloration). Le rendu en pixels n'est pas testé : voir `specs/015-studio/s6-studio-live.md` et `s7-atelier.md` pour la façon dont il a été relu.

`tests/link_tests.rs` (lecture, écriture et refus des liens, requête d'URL du web), `tests/control_tests.rs` (canal de commande avec un faux studio : lien transmis et acquitté, refus avec sa raison, requête HTTP et ligne trop longue refusées, adresse non locale refusée, personne à l'écoute, options de `open` ; le lancement d'un studio n'y est jamais exercé) et `tests/open_tests.rs` (surveillance sur un dossier temporaire : un rechargement après repos, un seul pour deux écritures, erreur avec fichier et ligne rendue une fois ; un lien arrive à la graine et à la mesure qu'atteint une session linéaire, note pour note ; un rechargement garde graine, boucle, solo, mutes, tempo et swing ; à l'écran, un fichier modifié se recharge, un fichier invalide laisse jouer le profil précédent, « Copy link » donne le lien de ce qu'on entend).

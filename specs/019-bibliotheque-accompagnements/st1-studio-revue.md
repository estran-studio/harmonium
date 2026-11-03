# ST1 — Le studio comme outil de revue : liens d'ouverture et rechargement à chaud

Branche `wquintal/st1-studio-revue`. Décision du propriétaire (2026-11-03) : les revues et les tests
passent entièrement par le studio (plus de pages d'écoute MP3) ; l'orchestrateur ouvre un lien qui
charge un profil quelconque avec une graine et une position, et le studio recharge à chaud les
fichiers du profil. **Aucun changement du moteur** (`git diff` : seuls `harmonium_studio/`, `Makefile`,
`README.md`, `CLAUDE.md`, `Cargo.lock` et ce compte rendu) : dumps `harmonium_determinism` et rendus
identiques à `main` par construction.

## Livré

| Brief | Où | État |
|---|---|---|
| 1. Format de lien | `harmonium_studio/src/link.rs` ; `README.md`, `harmonium_studio/README.md` (« Liens d'ouverture ») | fait |
| 2a. Commande `open` | `harmonium_studio open '<lien>'` ou `--profile … --seed … --bar …` ; `make studio/open URL=… \| PROFILE= SEED= BAR= LOOP= BPM= SWING= SOLO= MUTE= PLAY=1` ; `src/control.rs` | fait |
| 2b. Canal local | TCP `127.0.0.1:47820`, une ligne en entrée (le lien), une ligne en sortie (`ok` ou `error: …`) ; `src/control.rs` | fait |
| 2c. Schéma `harmonium://` | `scripts/install_link_handler.sh`, `make studio/install-handler` / `studio/uninstall-handler` | macOS et Linux faits ; Windows documenté seulement |
| 3. Rechargement à chaud | `src/reload.rs` (`ProfileWatch`), `Session::reload_profile`, barre du haut | fait |
| 4. « Copy link » | barre du haut, `Session::link_state` | fait |
| 5. Web | requête d'URL `?profile=<id>&seed=…` pour les profils compilés | fait |
| 6. Docs | `README.md`, `harmonium_studio/README.md`, `CLAUDE.md` (règle des agents), ce compte rendu | fait |

### Le lien

`harmonium://open?profile=<id ou dossier absolu>&seed=3&bar=17&loop=17-24&bpm=180&swing=0.6&solo=Bass&mute=Melody,Chords&play=0`

Noms de `--seed`, `--swing`, `--solo-track` (CLI) et de `seek`, `loop`, `bpm` (REPL). Décisions :

- **Un lien est un état entier.** Sans `solo`/`mute`/`loop` : pistes rendues, boucle levée ; sans `bar` :
  début de la boucle, sinon mesure 1. Sans cela, deux ouvertures successives se contamineraient et
  « le même lien » ne redonnerait pas la même chose. `profile` absent garde le profil qui joue (changer
  seulement la graine) ; `bpm`/`swing` absents : ceux du profil.
- **Pause par défaut** (`play=1` pour lancer). `Session::open` finit par `Intent::Play(play)` : un lien
  sans `play` met aussi en pause un studio qui jouait.
- **Strict** : clé inconnue, valeur hors bornes, clé répétée (sauf `mute`), chemin relatif, `loop` à
  l'envers : refusé avec la raison, rien n'a changé. Une faute de frappe (`sed=3`) ne passe pas en silence.
- `profile` : un chemin absolu ouvre ce dossier ; un identifiant ouvre `profiles/<id>` du dépôt (racine
  réglable par `--profiles`, par défaut celle du dépôt qui a compilé le studio) quand `profile.toml`
  y est, sinon le profil compilé. Le dossier est donc rechargé à chaud, comme exigé.
- Valeurs en pourcentage UTF-8 (`+` reste `+`) ; chemins Windows (`C:\…`) reconnus comme dossiers.

### Le canal de commande

OSC (`harmonium::osc`) n'a pas été réutilisé : UDP, un seul message (émotions), pas de réponse ; la
commande `open` doit savoir si un studio écoute pour décider de lancer le sien, et dire si le profil
est refusé. Protocole minimal TCP, documenté dans `control.rs` et le README du studio.

- **Local seulement** : adresse de boucle locale exigée à l'ouverture (`0.0.0.0`, une IP de LAN :
  refusées) ; une connexion dont l'origine n'est pas locale est ignorée ; la première ligne doit être
  un lien (une requête HTTP qu'une page web enverrait au port est refusée et ne parvient jamais à
  l'écran) ; ligne de 8 Kio au plus ; délai de 2 s par client.
- Le profil est **lu et validé sur le fil du canal** avant la réponse : `open` imprime donc tout de suite
  `cells.toml:213: …` si le profil du lien est invalide, sans que le studio ait rien changé.
- Le canal s'ouvre avant le chargement des ressources (quelques secondes) : un lien reçu alors attend
  l'écran Live. Port pris : le studio démarre quand même et le dit dans un avis.
- `open` sans studio **lance** le même exécutable avec `--open <lien>`, détaché (groupe de processus
  propre : il survit au terminal) ; le studio lancé saute le bouton Start (réservé au geste que le
  navigateur exige) et arrive **en pause**. Un studio lancé avec `--open` à côté d'un studio qui tourne
  lui passe le lien et s'arrête (deux lancements rapprochés ne donnent pas deux fenêtres, sauf si le
  second précède de quelques dixièmes de seconde l'ouverture du canal du premier).
- Codes de sortie de `open` : 0 ouvert, 1 refusé (raison imprimée), 2 mal appelé.

### Gestionnaire du schéma

- **macOS** : `~/Applications/Harmonium Link Handler.app`, compilée par `osacompile` (`on open location`),
  `CFBundleURLTypes` ajouté par PlistBuddy, `LSUIElement` (pas de Dock), signature ad hoc refaite,
  `lsregister -f`. Vérifié ici, sans enregistrer le schéma ni lancer le studio : l'appli construite dans un
  dossier temporaire (chemin avec espace) contient le schéma, `codesign -v` passe, et `open -a <appli>
  'harmonium://open?profile=/tmp/a%20b&seed=3&bar=17'` appelle le binaire factice avec exactement
  `open` puis le lien tel quel (Apple Event → commande).
- **Linux** : `harmonium-studio-link.desktop` (`Exec="<binaire>" open %u`) et `xdg-mime default …
  x-scheme-handler/harmonium`. **Non exécuté** (machine macOS) : la syntaxe du script est contrôlée
  (`sh -n`), pas le résultat.
- **Windows** : clé de registre documentée, non faite ; le canal et `open` sont portables, non essayés.
- Le script refuse un chemin de binaire avec `"`, `\`, `` ` ``, `$` ou `%` (rien à échapper).

### Rechargement à chaud

`ProfileWatch` : date de modification et taille de `profile.toml` et des banques de son `[files]`
(liste relue quand `profile.toml` change), toutes les 150 ms ; une modification est relue après 300 ms
de repos (un seul rechargement pour un éditeur qui écrit en deux temps ou par renommage), par
`StyleProfile::load`. Pas de crate de surveillance (aucune n'est dans l'arbre ; six `stat` par 150 ms).
La surveillance est prise **avant** la lecture : une modification pendant le chargement est rechargée.

Gardé par `Session::reload_profile` : graine, mesure (la tête de lecture ne bouge pas), boucle, solos
et mutes (le moteur ne les touche pas avec un profil) ; **tempo et swing imposés** que
`SetStyleProfile` remet (`Desk::imposed_bpm`/`imposed_swing`, effacés par un nouveau profil, `ResetBpm`,
`Reset`) et niveaux choisis au mixeur (le moteur rend leur niveau aux pistes quand le mixage change).
Invalide : le précédent joue, erreur `fichier:ligne: champ: message` en rouge dans la barre du haut et en
avis, rendue une fois ; « reloaded hh:mm:ss » (heure locale, `chrono` natif, `Date` côté web) sinon. Un
style choisi dans la liste ou un « Apply » de l'atelier arrête la surveillance.

## Preuves

Sans carte son ni fenêtre (hôte détaché, dossier temporaire, `egui_kittest`). `cargo test -p
harmonium_studio` : **vert, 8 + 8 + 13 tests nouveaux** (`link_tests`, `control_tests`, `open_tests`) et
les 56 existants.

| Exigence du brief | Test |
|---|---|
| Lien : lecture de tous les champs, aller-retour (chemin avec espaces, accents, `+`, `&`, `%`), refus motivés, pause par défaut, requête web | `link_tests.rs` |
| Canal : lien transmis et acquitté ; refus avec sa raison ; HTTP, ligne trop longue, ligne mal formée refusés sans atteindre l'écran, le canal sert le client suivant ; adresse non locale refusée (ouverture et client) ; personne à l'écoute ≠ échec ; un autre service sur le port n'est pas pris pour un studio ; `open` : codes 0 / 1 / 2, options = lien | `control_tests.rs` |
| **Un lien arrive à la graine et à la mesure voulues** : `seed=3&bar=17&loop=17-20` donne, note pour note (piste, hauteur, pas, durée, vélocité), les mesures 17 à 20 d'une session linéaire de graine 3 ; reste en pause ; tête de lecture dans la boucle | `a_link_lands_on_the_seed_and_bar_a_linear_session_reaches` |
| Solo, sourdines, tempo, swing imposés ; le lien suivant les rend ; nommer le profil rend le tempo du profil | `a_link_sets_tracks_and_tempo_and_the_next_link_clears_them` |
| Un identifiant ouvre `profiles/<id>` quand il existe (surveillé), sinon le compilé ; identifiant inconnu et profil invalide refusés avant tout changement | `an_id_opens_…`, `a_folder_with_an_invalid_profile_is_refused_…` |
| Rechargement : le changement d'un fichier recharge, une seule fois pour deux écritures dans la fenêtre, une banque de `[files]` est surveillée et un fichier étranger non, un fichier invalide donne `cells.toml:<ligne>` une fois puis la correction recharge | `open_tests.rs` (temps simulé) |
| **Rechargement garde graine et mesure** : à l'écran, `seed=9&bar=5&bpm=190&solo=Bass`, modification de `profile.toml`, « reloaded » apparaît, le nom change, graine 9, mesure 5, tempo 190, solo Bass | `the_screen_reloads_an_edited_profile_and_keeps_the_seed_bar_and_tempo` |
| **Fichier invalide garde l'ancien profil** : « reload failed » et `cells.toml:` à l'écran, profil et graine inchangés, puis correction : « reloaded », l'erreur disparaît | `the_screen_keeps_the_previous_profile_when_the_new_one_is_invalid` |
| `reload_profile` garde ce que le moteur remet : sans lui, le tempo retombe à celui du profil (témoin dans le test) | `reloading_a_profile_keeps_what_the_engine_resets_with_a_profile` |
| « Copy link » donne le lien de ce qu'on entend (dossier absolu, graine, mesure, boucle, tempo, solo, mutes) ; id distribué par son identifiant ; un brouillon est refusé avec l'explication | `copy_link_*` |

Smoke de la vraie commande, sans studio ni son : le binaire `harmonium_studio open --control
127.0.0.1:<port> --profile ../profiles/reel_quebecois --seed 3 --bar 17 --loop 17-24 --solo bass`
contre un faux studio reçoit `harmonium://open?profile=<chemin absolu>&seed=3&bar=17&loop=17-24&solo=Bass`
et sort 0 ; lien invalide : 2 ; sans argument : 2. Le chemin « lancer le studio » n'est exercé nulle part
(il ouvrirait une fenêtre avec une sortie audio) ; il n'a été que relu.

Acceptation : `cargo check -p harmonium_studio` natif **et** `--target wasm32-unknown-unknown` verts,
`--no-default-features` vert, `cargo clippy -p harmonium_studio --all-targets` (natif et wasm) sans
avertissement sur le studio, `cargo fmt --all -- --check` vert, `cargo check --locked` vert.

## Limites et manques

- **Le studio décode avec le moteur de son binaire.** Un profil de worktree qui emploie un mécanisme
  absent de ce moteur (champ `[arc]`, `[[fills]]`… non fusionné) est refusé par un studio compilé ailleurs :
  `open` imprime l'erreur du décodeur. Pour ces profils, compiler le studio **dans le worktree**
  (`make studio/open` depuis le worktree, qui prend ses propres `profiles/` et son moteur) ; si un studio
  plus ancien tourne, le fermer d'abord (un seul studio écoute le port).
- Le studio lancé par un lien arrive après le chargement des ressources (banque de 32 Mo, quelques
  secondes) ; le lien attend.
- Un profil compilé ou ouvert par le sélecteur n'est pas surveillé ; le web ne surveille rien (pas de disque).
- Un brouillon de l'atelier n'a pas de lien tant qu'il n'est pas enregistré dans un dossier et chargé.
- Non réimposés par un rechargement : signature, percussion, intensité, émotions (le profil les redéfinit
  ou elles ne dépendent pas de lui ; seul le tempo/swing était demandé). `SetStyleProfile` remet aussi
  la signature de temps : si cela gêne, un `Desk::imposed_signature` suit le même schéma.
- Le port 47820 est fixe ; `--control` en change, pour le studio comme pour `open`.
- Linux et Windows non essayés (voir plus haut).
- Dépendance ajoutée : `chrono` (natif, `clock` seul), pour l'heure locale du bandeau ; `Cargo.lock` :
  `chrono`, `iana-time-zone`, `iana-time-zone-haiku`. Le web n'en a pas besoin.
- Un test existant (`the_style_selector_the_profile_folder_and_the_seed_fields_drive_the_session`)
  cliquait « Engine » sur l'arbre d'accessibilité du cadre précédent : avec « Copy link », la barre du haut
  passe sur deux rangées avec une graine de vingt chiffres. Un cadre ajouté avant le clic ; rien d'autre.
- Disque de la machine presque plein pendant ce travail (3 à 11 Go libres) : `cargo test --workspace`
  et le script `harmonium_determinism` n'ont pas été relancés ; aucun crate moteur n'a changé.

## Commandes pour l'orchestrateur

```sh
# Ouvrir le profil d'un worktree à une graine et une mesure (en pause ; PLAY=1 pour lancer)
cd ~/Project/bascanada/harmonium
make studio/open PROFILE=$HOME/orca/workspaces/harmonium/<worktree>/profiles/<id> SEED=3 BAR=17 LOOP=17-24
# ou, par lien
make studio/open URL='harmonium://open?profile=/…/profiles/<id>&seed=3&bar=17'

# Installer le gestionnaire de liens (clic sur un harmonium://… dans un terminal, un message, une page)
make studio/install-handler
```

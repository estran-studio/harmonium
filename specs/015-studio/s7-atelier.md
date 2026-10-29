# S7 — Atelier de profils (onglet Workbench)

Branche `wquintal/s7-atelier` (depuis `daf090b`). Plan : `specs/015-studio/plan.md`, tâche S7 et acceptation 5.
Crate : `harmonium_studio/` (README : section « Atelier (Workbench) »).

Rien n'a été écouté. Le web a été conduit dans un Chromium sans tête lancé avec `--mute-audio` (aucun son ne
sort) ; le natif, par `egui_kittest` sur un hôte détaché.

## Ce qui a changé

1. **Couche pure `workbench.rs`** (sans egui ni plate-forme).
   - `Draft` : les fichiers d'un profil tels qu'ils sont sur disque (`profile.toml`, puis les banques dans l'ordre
     de son `[files]`, puis le reste par nom ; commentaires et découpe gardés) et leur origine (`Fork`,
     `Folder`, `Upload`). `Draft::decode` passe par `StyleProfile::from_files` : le décodeur de `load`, donc de
     `harmonium-cli --profile`, avec ses erreurs `fichier:ligne: champ: message` (S4).
   - `Draft::fork(id)` : copie d'un profil distribué sous `<id>_fork` et « <nom> (fork) ». Seules ces deux
     valeurs de `[meta]` changent (`toml_edit`, décor gardé) : le profil copié joue la même musique que l'original.
   - Archive : `Draft::to_zip` (`<id>/<fichier>`, sans compression) et `files_from_upload` (une archive : le
     `profile.toml` le moins profond fixe le dossier, `__MACOSX/` ignoré, Deflate lu ; ou des fichiers choisis
     ensemble ; seuls `.toml` et `.md` gardés ; un nom qui sort du dossier est ignoré).
   - `toml_spans` : découpage tolérant d'un texte TOML pour la coloration (en-têtes, clés, y compris dans une
     table en ligne, chaînes, littéraux, commentaires ; un tableau en début de ligne dans une valeur n'est pas un
     en-tête).
2. **Écran `workbench_screen.rs`** : à gauche le brouillon (état, Apply) et les profils distribués ; au centre
   l'en-tête (profil distribué : « read-only » et « Fork » ; brouillon : son nom, ou l'erreur et « Go to line »),
   un onglet par fichier (le fautif marqué « (error) »), l'éditeur : coloration, numéros de ligne dans la marge,
   ligne de l'erreur sur fond rouge, sans retour à la ligne automatique.
   - Le brouillon se décode **à chaque modification** : l'erreur suit la frappe. Coût mesuré (natif, release,
     profils distribués) : 0,5 à 2,2 ms par décodage, 30 à 110 µs de découpage pour la coloration (mise en cache
     par egui d'une image à l'autre).
   - **Apply** pousse `EngineCommand::SetStyleProfile`, comme le sélecteur de style ; refusé tant que le
     brouillon ne se décode pas.
   - L'en-tête est toujours là : l'éditeur ne saute pas d'une rangée quand une erreur paraît ou disparaît.
3. **Plate-forme**. Natif : `read_profile_folder` (dossier ou `profile.toml`, banques de `[files]` qui se lisent ;
   une banque absente est laissée au décodage, qui pointe sa ligne de `[files]`), `write_profile_folder` (dossier
   créé au besoin, autres fichiers laissés, nom sortant refusé avant toute écriture). Web : `offer_download`
   (tiré de l'ancien `RecordingSink::save`, qui l'appelle), `pick_profile_files` (champ `<input type=file
   multiple>` attaché le temps du choix), `load_draft` / `store_draft` (`localStorage`, clé
   `harmonium-studio.workbench.draft`, à chaque modification ; refus signalé une fois).
4. **Écran Live** : l'onglet Workbench est actif ; le nom du style affiché est celui du profil installé
   (`LiveHost::profile().meta.name`), plus une recherche parmi les profils distribués : un brouillon appliqué
   s'appelle « Reel québécois (fork) », pas `reel_quebecois_fork`.
5. **Dépendances** du studio : `toml_edit` 0.22.27 (celle du moteur) et `zip` 9.0.5 (celle du labo) sans ses
   options par défaut, avec `deflate-flate2-zlib-rs` (Rust pur, compile en wasm). `Cargo.lock` : +2 lignes.

## Acceptation

| Critère | Vérifié |
|---|---|
| Un profil modifié s'entend à la mesure suivante | `live_screen_tests::a_forked_profile_saved_edited_and_applied_plays_from_the_next_bar` (kittest : Workbench, Fork, Save dans un dossier, `default_bpm` 230 → 225 sur disque, Open, Apply ; la mesure qui sonne reste à 230, la suivante est composée à 225 ; l'écran Live nomme « Reel québécois (fork) »). Web : Fork, édition, Apply, le sélecteur de style affiche « Reel québécois (fork) » |
| Une erreur pointe le champ et la ligne | `an_error_in_a_bank_is_placed_on_its_line_and_blocks_apply` (kittest : `=oops` tapé dans `cells.toml` ; message `cells.toml:<ligne tapée>: …`, onglet marqué, Apply désactivé, profil inchangé). Web : `max_bpm` sous `default_bpm` donne `profile.toml:23: meter.default_bpm: tempo hors plage`, ligne 23 en rouge ; relu en image (rendu `egui_kittest` jetable, voir plus bas) |
| Le dossier enregistré se charge par `harmonium-cli --profile` | `workbench_tests::a_saved_folder_loads_like_harmonium_cli_profile_and_reopens_identically` (`StyleProfile::load`, le chargeur de `--profile`) ; `the_downloaded_archive_unzips_into_a_loadable_folder_and_uploads_back`. Réel : l'archive téléchargée du web, décompressée, puis `harmonium-cli --export --bars 8 --seed 1 --profile /tmp/…/reel_quebecois_fork --record-midi …` → « Profil reel_quebecois_fork … Export complete », MIDI de 1 143 octets |
| Fork d'un profil distribué | `a_fork_keeps_every_file_and_comment_under_a_new_identity` (mêmes fichiers ; deux lignes de `profile.toml` changées ; profil décodé égal à l'original hors `[meta]`) |
| Natif : ouvrir et enregistrer un dossier | test kittest ci-dessus ; `a_saved_folder_refuses_names_that_leave_it` |
| Web : téléverser, télécharger, modifications gardées | Chromium 150 sans tête, son coupé : « Download .zip » → `reel_quebecois_fork.zip` (6 fichiers sous `reel_quebecois_fork/`) ; nouveau lancement du navigateur, même profil : le brouillon revient (avec la modification) ; « Upload… » d'une archive faite par `zip -r` (Deflate) avec `id` et `name` changés → brouillon « Reel téléversé », origine « Uploaded », valide. Tests : `an_upload_keeps_the_profile_files_only`, `an_archive_of_a_nested_folder_drops_what_is_outside_the_profile` |

## Relecture du rendu

Comme en S6, un test jetable (non commité, `egui_kittest` avec `wgpu` et `snapshot`) a rendu l'atelier : profil
distribué, brouillon copié, erreur au bout de `cells.toml`, « Go to line ». Corrigé après lecture : le marqueur
« ● » de l'onglet fautif n'existe pas dans les polices d'egui (carré vide), remplacé par « (error) ». Le saut
à la ligne est animé par egui : la ligne arrive au milieu de la vue en quelques images.

## Vérification

| Commande | Résultat |
|---|---|
| `make test` | voir plus bas |
| `make lint` | voir plus bas |
| `cargo +nightly fmt --all -- --check` | sortie 0 |
| `cargo check --target wasm32-unknown-unknown -p harmonium_studio` | sortie 0 |
| `trunk serve --release --port 8788` (le 8787 était pris par un serveur du checkout principal) | build et service ; essais web ci-dessus |

## À vérifier par le propriétaire

1. Natif (`make studio`) : Workbench → Fork de « Reel québécois » ; dans `profile.toml`, `default_bpm = 225.0` ;
   Apply : le tempo de la barre du haut passe à 225 à la mesure suivante, et s'entend.
2. « Profile folder » : un chemin, Save ; `harmonium-cli --profile <chemin>` le joue.
3. Web (`make studio/web/serve`) : mêmes gestes ; « Download .zip », puis « Upload… » du `.zip` ; recharger la
   page : le brouillon est toujours là.
4. Firefox et Safari : le choix de fichiers part d'une image egui qui suit le clic (activation transitoire du
   geste), pas du gestionnaire du clic lui-même ; mesuré seulement dans Chromium.

## Limites

- Un seul brouillon ; Fork, Open ou Upload le remplacent (l'infobulle de Fork le dit). Le web garde ce brouillon,
  pas un historique.
- Natif : le dossier se saisit au clavier (comme « Profile folder » de l'écran Live), sans boîte de dialogue :
  `rfd` ajouterait GTK ou un portail XDG à la CI Linux (S9).
- Le natif ne garde pas le brouillon entre deux lancements : il vit dans le dossier enregistré.

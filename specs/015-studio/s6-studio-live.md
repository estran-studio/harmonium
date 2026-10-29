# S6 — Écran Live et parité REPL

Branche `wquintal/s6-studio-live` (depuis `63557cb`, main après la phase 014 : `Articulation::Legato`, essai
`isaac_basement`). Plan : `specs/015-studio/plan.md`, § « Écran Live » et « Parité REPL ↔ studio ». Crate :
`harmonium_studio/` (README : usage et tableau des zones de l'écran).

Rien de ce qui est décrit ici n'a été écouté : toute la vérification s'est faite sans carte son (hôte détaché,
rendu hors ligne, tests egui sans fenêtre). Ce qui demande des oreilles ou une vraie fenêtre est sous
« À vérifier par le propriétaire ».

## Ce qui a changé

1. **Couche pure `live.rs`** (sans egui, sans plate-forme), testée sur un hôte détaché.
   - `Session` : l'hôte (`LiveHost`), la partition reçue de `pump` (`Score`, mesures rangées par numéro ; un lot
     rendu par `pump` remplace les mesures qu'il invalide) et le `Desk` (ce que le studio a commandé sans que le
     moteur le publie : gains choisis, sourdines, solo, routage, boucle, tempo audible imposé, enregistrements
     en cours). Le `Desk` ne change que si `LiveHost::apply` a accepté la commande.
   - `Intent` → `EngineCommand` : `Session::perform` est le seul chemin de l'écran, de l'OSC et des tests vers
     `LiveHost::apply`. `Mute`, `Solo`, `Loop` (ordre libre) et `Play` sont les seules intentions qui ne sont pas
     une commande telle quelle ; le solo se réduit à des `SetChannelMute` (les autres pistes sont coupées, une
     sourdine à la main survit à la fin du solo).
   - `LiveView` : ce que l'écran dessine, tiré de la sortie publiée. Horloge (`BeatView`, de `BeatInfo`), fenêtre de
     20 mesures (`ScoreView` : 4 jouées, la mesure en cours, 15 composées d'avance), accords (`chord_changes`),
     forme et tonalité (`TunePosition`), panneau d'état (équivalent de `state`), mixeur.
   - **Tempo et tonalité viennent de ce qui sonne.** Le tempo entendu est tiré de la durée de la pulsation de
     `BeatInfo` et des pas par pulsation de la mesure qui sonne (reel québécois : 230 BPM) ; la tonalité et le mode,
     de `MeasureSnapshot.position` (`D Ionian`). `EngineReport.session_key` / `session_scale` ne sont lus nulle part
     (voir « À nettoyer »).
2. **Écran `live_screen.rs`** : barre du haut, commandes à gauche, forme / accords / piano-roll au centre, horloge,
   état et mixeur à droite, panneau About. Chaque image lit une `LiveView` et rend des `Intent` exécutés après le
   dessin.
   - Piano-roll : une note se dessine selon son `Articulation` publiée, avec un `match` exhaustif
     (`paint_note`) : une articulation nouvelle ne compile pas sans son dessin. `Normal` barre pleine ; `Accent`
     cerclée de blanc ; `Detached` barre raccourcie ; `Linked` bouts carrés ; `Tied` bouts carrés raccrochée à la
     précédente par un trait (pas d'attaque) ; **`Legato`** (phase 014) attaque, arc depuis la note précédente et
     prolongement clair au-delà de sa fin (le chevauchement joué) ; `Trigger` losange (pieds). Les pistes ont des
     hauteurs relatives (3·2·3·2·1·1, les frappes de pieds n'ont pas besoin d'une rangée entière) et une piste sans
     note dans la fenêtre est grisée.
   - Accords : un nom à chaque changement, coupé là où le suivant commence (la pédale d'`isaac_basement` tient
     8 mesures sur « D (pédale) » : le nom n'est plus répété ni tronqué mesure par mesure).
   - Forme : « A·1 » = partie A, passe 1 (l'ancienne étiquette « A11 » de `isaac_basement`, partie « A1 » passe 1,
     était ambiguë) ; l'état dit « A pass 1 · bar 5/8 ».
   - Le sélecteur de style montre les onze profils sans défilement : à 200 px par défaut, la liste de egui défilait
     et le dernier profil (« Valse québécoise ») sortait de la zone cliquable.
   - Mode direct : un avertissement dit que bouger une émotion repasse en mode émotion (c'est ce que fait
     `SetEmotionParams`).
3. **Plate-forme** (`platform/`) : `SessionHandle` (natif : `Arc<Mutex<Session>>` ; web : `Rc<RefCell<Session>>`), fil
   ou Worker qui appelle `Session::pump`, `RecordingSink` (natif : fichiers dans `--recordings`, défaut
   `~/Music/Harmonium Studio`, suffixe `-2`, `-3` pour ne jamais écraser ; web : téléchargement par un lien
   `download` sur un `Blob`, URL libérée après 60 s), `start_osc` (natif, `harmonium::osc`, même adresse que le REPL). Web : le Worker de S5 appelle `Session::pump`
   (donc `LiveHost::pump`, qui avance l'enregistreur sans fil de S3) toutes les 100 ms, soit bien plus souvent que
   les ~3 s exigées ; par construction, non mesuré dans un navigateur.
4. **Host** (additif, goldens inchangés) : `LiveHost::is_emotion_mode()`. Le studio lit le mode de contrôle par
   l'API publique, pas dans le composer.
5. **Coquille** : fenêtre native 1440 × 900 (minimum 1100 × 700) ; `app.rs` passe de l'écran provisoire de S5 à
   `LiveScreen` après Start.

## Parité REPL ↔ studio

Même commande, même exécuteur. Colonne « Vérifié » : ce qui a été fait sans son. `lt` = `tests/live_tests.rs`,
`ls` = `tests/live_screen_tests.rs` ; `kittest` = le test agit sur le vrai widget.

| REPL | Contrôle du studio | Vérifié |
|---|---|---|
| `style <id>` | sélecteur de style | `ls` `the_style_selector_…` (kittest : ouvrir la liste, choisir « Valse québécoise » ; l'hôte joue `valse_quebecoise`) |
| `style <dossier>` | natif : « Profile folder » + Load (repli « Engine ») | `ls` même test (kittest : `profiles/gigue_acadienne`) ; web : n'existe pas (atelier S7) |
| `reset` | « Reset engine » | `lt` `reset_returns_to_the_active_profile_defaults_at_bar_one` (profil actif gardé, mesure 1, boucle et tempo audible retirés) ; host `reset_restarts_the_active_profile_from_bar_one` (mêmes notes qu’au premier passage) |
| `feet 0\|1\|2` | pieds 0 / 1 / 2 | `lt` `the_view_follows_the_commands_of_the_parity_table` (commande, vue) |
| `intensity`, `direct` | curseur Intensity ; boutons Emotion / Direct | idem (commande, vue) |
| `emotion …` | pavé, quatre curseurs, bouton Emotion | `ls` `moving_the_arousal_slider_…` (kittest : curseur Arousal ; l'hôte publie la valeur et recompose la mesure d'avance après `pump`) ; `lt` `an_emotion_change_recomposes_the_bars_ahead` |
| `seed <n>`, `seed new` | champ + Set seed ; New seed | `ls` `the_style_selector_…` (kittest) ; `lt` `seed_commands_restart_the_score` |
| `set bpm`, `reset bpm` | Nominal + Reset BPM | `lt` parité (commande, vue) |
| `swing <x>` | curseur 0,5–0,75 | `lt` parité |
| `tempo <bpm> [N]`, `tempo off` | Audible + pulses, Apply tempo, Clear tempo | `lt` parité (état du bureau) ; le tempo qui change à l'oreille : à vérifier |
| `set time n/d` | signature + Apply | `lt` parité |
| `beat` | horloge, toujours visible | `lt` `the_score_view_shows_…` (pulsation proche du transport) ; `ls` `the_beat_clock_and_the_state_panel_…` (kittest : l'horloge est dessinée) |
| `seek <n>` | clic sur une mesure ; champ « Seek » | `ls` `clicking_a_bar_seeks_there` (kittest : clic ; le transport est à la mesure) ; `lt` `seek_moves_the_playhead_hard_and_soft` |
| `seek <n> soft` | clic droit « Move playhead here » ; « Move playhead » | `lt` même test (commande) ; le menu : à vérifier |
| `loop a b`, `loop off` | glisser sur les mesures ; Clear loop | `ls` `dragging_across_bars_…` (kittest : glisser de droite à gauche, boucle posée ; glisser de 20 mesures : « Loop too long » affiché, boucle inchangée ; Clear loop) ; `lt` `a_loop_holds_the_playhead_…` |
| `set volume` | Master volume | `lt` parité (vue) ; le volume entendu (rampe de 10 ms) : à vérifier |
| `set gain`, `set mute/unmute` | gain, Mute par piste | `lt` parité (gain, vue) ; `lt` `solo_silences_the_other_tracks_…` (sourdine à la main, signal rendu) |
| solo (nouveau, sans commande propre) | Solo par piste | `ls` `the_solo_button_mutes_every_other_track` (kittest) ; `lt` `solo_silences_the_other_tracks_…` (signal rendu : solo sur une piste sans note = silence sous −80 dB, retour du son à la fin du solo, sourdine à la main conservée) |
| `set route` | Routing → banque par piste | `lt` parité (commande, vue) ; l'effet à l'oreille : à vérifier |
| `set output mute` | Mute output | `lt` parité (commande, vue) |
| `set rhythm\|harmony\|melody` | cases dans « Engine » | `lt` parité (commande, vue : `modules`, état) |
| `set lookahead` | « Lookahead » dans « Engine » | `lt` parité (commande, vue) |
| `record wav\|midi\|musicxml`, `stop <format>` | WAV / MIDI / MusicXML (bascule) | natif : `ls` `the_record_toggle_writes_a_file_and_shows_where` (kittest : un fichier `RIFF` dans le dossier, chemin affiché) ; `lt` `recordings_are_named_and_saved_without_overwriting` (MIDI `MThd`, nom `harmonium-<profil>-seed<graine>`, jamais écrasé) ; web : le téléchargement : à vérifier |
| `state` | panneau « State » | `lt` `the_screen_shows_what_plays_not_the_session_leftovers` ; `ls` (kittest : « Tune key/mode » présent) |
| `--osc` | natif : « OSC listener » (127.0.0.1:8080) | `lt` `the_osc_listener_drives_the_emotions_of_the_session` (message UDP réel sur un port libre) |
| REPL : touches a/v/d/t | pavé et curseurs | idem `emotion` |

Aucune commande du parseur du REPL n'a manqué au tableau : `set rhythm|harmony|melody`, `set route`,
`set output`, `set lookahead`, `seek … soft` y sont déjà (S2). Le solo n'existe pas au REPL (seulement
`set mute`) : le studio le réduit à des `SetChannelMute`.

## Tests ajoutés ou réparés

- `ls` : le sélecteur de style n'était pas cliquable par kittest (liste défilante, voir plus haut) ; il l'est.
  Nouveau : `the_about_panel_lists_the_licences_and_credits`.
- `lt` : `the_screen_shows_what_plays_not_the_session_leftovers` (tempo entendu = tempo de la mesure qui sonne,
  ≠ 120 ; tonalité et mode = `TunePosition` ; aucune valeur du panneau ne parle de `PentatonicMinor` ni de
  « Session ») ; `every_articulation_reaches_the_piano_roll` (une note de chacune des sept `Articulation` dans une
  mesure publiée : toutes arrivent à la vue, dans l'ordre) ; `legato_bass_and_tied_chords_of_isaac_are_in_the_view`
  (`isaac_basement` joué : des notes `Legato` à la basse et `Tied` aux accords dans la fenêtre) ;
  `reset_returns_the_engine_to_the_profile_defaults_…`.
- Le contrôle de chaque articulation est un `match` exhaustif à la compilation. Le dessin lui-même (pixels) n'est
  pas testé : relu en image (voir plus bas).

## Relecture du rendu

Pour juger la lisibilité sans fenêtre ni carte son, un test jetable (non commité) rendait l'écran avec
`egui_kittest` (features `wgpu` et `snapshot`, rendu GPU hors écran, sans audio) après 700 tampons joués sur un hôte
détaché, pour `reel_quebecois` et `isaac_basement`. Il a montré des défauts, corrigés : notes liées débordant
sous les étiquettes de piste, noms d'accord tronqués et répétés, rangées de pieds aussi hautes que la mélodie,
étiquettes de forme ambiguës. Ces captures ne sont pas des tests : la dépendance `wgpu` n'entre pas dans la CI
Linux du studio (S9).

## Vérification

Toutes dans le worktree, sans son :

| Commande | Résultat |
|---|---|
| `make test` | sortie 0 : 90 suites, 658 tests verts, 2 ignorés (les mêmes qu'avant) ; goldens inchangés (`HARMONIUM_UPDATE_GOLDENS` jamais posé) |
| `make lint` | sortie 0 ; aucun avertissement dans `harmonium_studio` ; les avertissements restants viennent des crates du moteur, que cette tâche ne touche pas (seul ajout : `LiveHost::is_emotion_mode`) |
| `make wasm/check` | sortie 0 |
| `cargo check --target wasm32-unknown-unknown -p harmonium_studio` | sortie 0 (le code web du studio compile) |
| `WASM_BINDGEN=$PWD/.tools/bin/wasm-bindgen make test/determinism` | natif = wasm : 74 / 74 fichiers identiques |
| `cargo +nightly fmt --all -- --check` | sortie 0 |
| `PATH=$PWD/.tools/bin:$PATH make studio/web/build` | `trunk build --release` : succès (construit, jamais servi ni ouvert) |

## À vérifier par le propriétaire

Rien de ce qui suit n'a été fait : il faut des oreilles ou une vraie fenêtre.

**Natif** (`make studio`) :

1. Start, puis écouter `reel_quebecois` : le piano-roll doit suivre l'audio (la tête de lecture traverse une mesure
   en même temps que la pulsation de l'horloge ; le tempo de la barre du haut dit 230).
2. Bouger le pavé ou le curseur Arousal : le changement s'entend à la mesure suivante (les 15 mesures d'avance sont
   recomposées, la mesure en cours finit).
3. Style → « Basement de The Binding of Isaac (essai) » : la basse legato se voit (arcs et prolongements) et
   s'entend sans coupure entre les notes ; les accords liés tiennent la pédale.
4. Cliquer une mesure (saut dur), clic droit → « Move playhead here » (saut doux), glisser sur 5 mesures (boucle),
   glisser sur plus de 16 (message rouge, boucle inchangée).
5. Mixeur : Solo sur Bass (seule la basse sonne), Mute sur Melody, gain, volume général ; « Mute output ».
6. Tempo audible : Apply tempo à 180 avec 8 pulses (rampe audible), Clear tempo.
7. REC : cliquer WAV, laisser jouer 10 s, recliquer : « Saved … » avec le chemin (`~/Music/Harmonium Studio/`) ;
   ouvrir le fichier (c'est le seul moment où un son sort, de votre fait) ; idem MIDI et MusicXML.
8. « Engine » → « OSC listener », puis envoyer un message sans son depuis un terminal ; les curseurs doivent bouger :
   ```sh
   python3 - <<'EOF'
   import socket, struct
   def pad(text): b = text.encode() + b"\0"; return b + b"\0" * (-len(b) % 4)
   packet = pad("/harmonium/params") + pad(",ffff") + struct.pack(">ffff", 0.9, -0.2, 0.4, 0.6)
   socket.socket(socket.AF_INET, socket.SOCK_DGRAM).sendto(packet, ("127.0.0.1", 8080))
   EOF
   ```
9. « About » : licences, liens, crédits.
10. Repérer ce que ce rapport n'a pas pu juger : lisibilité des couleurs, taille des caractères, hauteur de la
    liste des styles sur votre écran.

**Web** (`PATH=$PWD/.tools/bin:$PATH make studio/web/serve`, http://127.0.0.1:8787) :

1. Mêmes points 1 à 6 et 9.
2. REC : le bouton WAV déclenche un téléchargement du navigateur (`harmonium-reel_quebecois-seed<graine>.wav`) à l'arrêt ;
   un onglet masqué à l'arrêt télécharge au retour au premier plan (la lecture de l'enregistreur sans fil est
   pompée toutes les 100 ms par le Worker, mais le téléchargement part dans une image de l'interface).
3. Pas de « Profile folder » ni d'« OSC listener » : normal (natifs).

## Corrections après l'essai du propriétaire

- **`reset`** gardait mal le style : il rechargeait le profil par défaut, laissait la tête de lecture où elle
  était et ne rejouait pas la graine. Il rend maintenant les défauts du profil actif, retire boucle et tempo
  audible imposé, et recommence à la mesure 1 avec la même graine (`90bf109`).
- **Pistes d'Isaac qui disparaissaient** au premier geste sur le pavé : le passage en mode émotion envoyait la
  densité par défaut du moteur (0,2), sous les seuils `density_tracks` d'Isaac (Bass 0,25 … Melody 0,4), d'où
  la nappe seule. Le mode émotion part désormais des axes qui jouent (`GameControls::set_emotion_mode`,
  `LiveHost::emotions`, aussi pour `emotion a+0.1` du REPL) ; le mixeur et le piano-roll disent quelle piste
  la densité écarte (« off: density < x ») ; « switched to Emotion mode » ne s'imprime plus qu'au vrai
  changement de mode.

## À nettoyer (hors périmètre S6)

- `EngineReport.session_key` / `session_scale` (`harmonium_core/src/report.rs:201-204`) ne décrivent pas ce qui
  joue : `MusicComposer - Session: A PentatonicMinor | BPM: 120.0` (`harmonium_host/src/composer.rs:149`) est
  imprimé au démarrage alors que `reel_quebecois` joue à 230 en ré. Le studio ne les lit pas. Les retirer demande
  de toucher l'export JSON du CLI (`harmonium_cli/src/export.rs:304`) et `state` du REPL
  (`harmonium_cli/src/repl.rs:630`) : pas trivial, donc laissé. `progression_name` / `progression_length` du
  rapport ne sont jamais remplis non plus.
- Les captures du rendu (ci-dessus) ne sont pas dans la CI : un test d'instantané demanderait une dépendance GPU
  et des images de référence versionnées ; à décider avec S9.

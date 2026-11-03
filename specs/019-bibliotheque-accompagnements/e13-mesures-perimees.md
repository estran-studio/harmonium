# E13 — Aucune mesure périmée jouée après un changement de séance

Date : 2026-11-03. Branche `wquintal/e13-mesures-perimees`, partie de `main` à `4d7bf50`. Demande
d'origine : `harmonium_training/specs/010-lecture-impro/demandes-moteur.md`, § E13. Aucune note
générée ne change : dumps `harmonium_determinism` identiques à `main` (98 fichiers), pas de bump de
`TUNING_VERSION`.

## Diagnostic vérifié

Le diagnostic de training (lignes de `main`) est exact, mais incomplet : quatre défauts, dont deux
qu'il ne nomme pas.

| # | Défaut (sur `main`) | Où | Effet |
|---|---|---|---|
| 1 | `DropFrom { bar }` n'annonce pas l'epoch. Le playback ne le connaît que par la première boîte qu'il dépile ; la file `rtrb` étant FIFO, ce sont les boîtes de l'ancienne séance qui arrivent en tête | `playback.rs:1347-1365` (`DropFrom`), `1022-1062` (`drain_staged`) | Les boîtes périmées de la file sont adoptées à l'epoch courant, entrent dans la fenêtre, `try_load` (`1116`) charge l'ancienne mesure du même index |
| 2 | La file pleine bloque le re-staging : `stage_bar` s'arrête dès que le ring est plein, et seul le consommateur peut vider un ring SPSC | `composer.rs:251-264` | Après un changement de séance avec la file pleine d'anciennes boîtes, la mesure 1 de la nouvelle séance ne peut pas être poussée avant le premier callback, qui adopte alors les anciennes (cas de training : A préparée seize mesures, B configurée sans callback) |
| 3 | La mesure sous la tête est gardée par `invalidate_future` même quand le playback ne l'a pas chargée (démarrage, après seek) | `composer.rs:358-364` | `SetBpm`, `SetSwing`, `SetStyleProfile` à l'arrêt : la mesure 1 joue avec les anciens réglages |
| 4 | Le compositeur lit la tête dans l'atomique de transport, que seul le callback écrit. Après `SetSeed`/`NewMelody`/`Seek` il reste en retard (indéfiniment si le flux est arrêté) | `composer.rs:359` | `SetSeed` puis `SetBpm` sans callback entre eux : `current_bar` repart de l'ancienne tête, les mesures 1…N−1 ne sont jamais composées, silence permanent (reproduit : « les mesures suivantes n'arrivent pas ») |

Aggravant, sans être un défaut de mesure périmée : après `Seek`/`SetSeed`, la mesure cible n'est
re-stagée qu'au `pump` suivant ; le callback qui précède sous-passe (un pas « figé » de l'ancienne
durée avant de réessayer, `playback.rs` `enter_step`) : le départ est en retard d'un pas.

Le symptôme du propriétaire est reproduit à l'identique par le test : sur `main`, séance A
(`reel_quebecois`, graine 1) puis B, notes de la mesure 1 jouées = `69, 72, 76, 78, 81…` au lieu des
snapshots de B.

## Correction

Générique, sans allocation ni verrou dans le callback.

**Playback (`playback.rs`)**
- `PlaybackCommand::DropFrom { bar, epoch }` : le playback adopte l'epoch du compositeur dès la
  commande. `take_pending_drop` renvoyait déjà `(bar, epoch)` ; `LiveHost::publish` le transmet.
  Les boîtes de la fenêtre sous la coupure sont inchangées par contrat : elles passent au nouvel
  epoch au lieu d'être balayées puis re-stagées.
- `drain_staged` regarde la tête de file (`peek`) et rend au compositeur toute boîte d'un epoch
  antérieur **même fenêtre pleine** : les boîtes périmées n'occupent plus jamais la file. Rejet à
  l'adoption, déplacement de pointeurs uniquement, sous `RtGuard`.
- Bit « non chargé » dans l'atomique de transport (bit 31 du pas ; `pack`/`unpack` le masquent,
  `pack_unloaded`/`is_unloaded` le lisent). Posé au démarrage, aux sauts (`Seek`, `SeekPlayhead`,
  repli de boucle) et sur sous-passe ; levé par `try_load`. Une écriture atomique de plus, aucune
  allocation. Sémantique de `transport_position()` inchangée (training s'en sert comme accusé de
  réception du callback).

**Compositeur (`composer.rs`, `live_host.rs`)**
- **Tête vue par le compositeur** (`playhead()`) : un `seek` commandé (`MusicComposer::seek`, utilisé
  par `LiveHost` ; `set_seed`, `new_melody`, `reset` le posent aussi) compte comme effectué jusqu'à
  ce que le playback publie une position à la cible. `deterministic_seek` reste pur.
- **Mesure sous une tête non chargée** : `invalidate_future` la recompose avec les autres, rejouée
  depuis la graine comme un seek (`deterministic_seek`), si elle était composée. La première mesure
  jouée porte donc le nouveau tempo, swing ou profil, et reste celle que la graine donne. Une tête
  garée au-delà de tout ce qui est composé ne déclenche aucun rejeu (pas de `silent_advance` de
  quatre milliards de mesures). Une mesure chargée, donc audible, est toujours gardée.
- **Réserve de file** : les 8 dernières places du ring (`URGENT_SLOTS`) ne servent qu'aux mesures
  `< tête + 8`. La mesure courante trouve de la place même si d'anciennes boîtes remplissent la file.
  Limite : plus de huit changements d'epoch sans callback entre eux épuisent la réserve ; le `pump`
  suivant le callback reprend.
- **Re-staging immédiat** (`stage_playhead_bar`) : `Seek`, `SetSeed`, `NewMelody`, `Reset` composent
  et stagent tout de suite la mesure que la tête attend ; les commandes de paramètre le font
  seulement si un seek est en vol. Le premier callback la trouve. Pas de re-staging systématique par
  commande : cinq commandes de configuration consommeraient cinq places pour des boîtes aussitôt
  périmées.
- Les snapshots en attente d'une mesure évincée sont retirés à l'éviction (`bump_epoch`,
  `clear_staged`) : plus de doublon de la mesure 1 dans `pump()`, et `pump()` ne rend plus que la
  séance courante.

Choix écartés : purge côté compositeur (impossible, ring SPSC) ; marquage d'epoch seul (ne résout ni 2
ni 3 ni 4) ; compteur partagé d'epoch (la commande `DropFrom` suffit et reste ordonnée avec les
autres).

## Tests

`harmonium_host/tests/stale_measures_tests.rs` : hôte hors ligne, MIDI enregistré (pistes 0 à 2,
hauteurs **et ticks**, grille de swing audible comprise, ±2 ticks), tempo audible de chaque mesure,
et ce que la première mesure jouée porte (tempo 150, swing 0,66, 3/4, séance B). 27 cas = 9
commandes (`Session` de training, `NewMelody`, `SetSeed`, `SetSeed` puis `SetBpm`, `SetStyleProfile`,
`SetBpm`, `SetSwing`, `SeekPlayhead`, `Seek`) × 3 états : `Fresh` (A préparée, file pleine, aucun
callback), `Parked` (A jouée à mi-mesure, flux arrêté, `pump` seuls), `Live`. Plus
`boxes_queued_before_a_new_epoch_never_play` : boîtes d'un epoch remplacé encore dans la file, sans
remplaçante, rien ne doit sonner (échoue si l'on retire l'annonce d'epoch de `DropFrom`).

| | échecs | causes |
|---|---:|---|
| `main` | **13 / 27** | notes de l'ancienne séance (`Fresh` Session, NewMelody, SetSeed, SeedThenBpm) ; tempo/profil/swing non appliqués (`Fresh` SetBpm, SetSwing, SetStyleProfile) ; départ en retard d'un pas (`Live` Session, NewMelody, Seek, SetSeed) ; silence permanent (`Live`/`Parked` SeedThenBpm) |
| branche | 0 / 28 (les 27 cas + le test de file) | |

Mutation : sans `self.epoch = self.epoch.max(epoch)` dans `DropFrom`, `boxes_queued…` échoue.

Tests existants modifiés :
- `DropFrom { bar, epoch }` : tous les appelants (`harmonium_cli/src/export.rs`, tests de
  `harmonium_host`) passent l'epoch de `take_pending_drop`.
- `playback::transport_position_tests::pack_unpack_at_bounds` : le pas est borné à 31 bits
  (le bit 31 est le drapeau) ; test du drapeau ajouté.
- `played_notes_tests::mutes_gains_et_seek_generes_ne_coupent_pas_une_note_jouee` : le seek stage
  maintenant la mesure cible, que le moteur généré joue coupée à gain nul (résidu ≈ 1,7e-8) ; la
  comparaison bit à bit devient un écart < 1e-6. La note jouée reste intacte.

## Temps réel

Chemins nouveaux : `peek`/`pop`/`stage_return` (pointeurs), retag de la fenêtre dans `DropFrom`
(boucle sur `ArrayVec`, sous `RtGuard`), `fetch_and`/`store` sur un atomique. Aucun `Vec`, aucun
verrou. `RTCheckAllocator` est l'allocateur global de la crate `harmonium` en debug
(`harmonium_host/src/lib.rs`) et `process_buffer` entre en contexte audio : les 28 tests de scénario
rendent des milliers de callbacks avec rejets de boîtes périmées, `DropFrom`, `Seek` et
enregistrement, sans panique. `handoff_tests` et `realtime_callback_tests` passent.

## Latence et seek

Mesuré en debug, hôte hors ligne à 8 kHz, `Seek(64)` en lecture :

| | `apply(Seek 64)` | buffers de 128 images jusqu'au pas 1 de la mesure 64 |
|---|---:|---:|
| `main` | 26,1–26,8 ms | 19 |
| branche | 26,4–26,8 ms | 10 |

Le re-staging immédiat coûte ≈ 0,3 ms (une mesure), et supprime la sous-passe : environ un pas de
moins avant le premier son (≈ 125 ms à 120 BPM). `deterministic_seek(64)` seul (le test
`seek_to_bar_64_under_50ms`, seuil inchangé) : 25,8–29,0 ms sur `main`, 25,8–28,3 ms sur la branche
(inchangé : `deterministic_seek` n'est pas modifié). Un changement de séance à l'arrêt n'attend plus
de callback pour que la mesure 1 soit juste ; si le premier callback précède tout `pump` après une
commande de paramètre (hôte jamais pompé), il sous-passe un pas en silence, jamais de note périmée.

## Ce que training peut retirer

Dans `harmonium_practice/src/engine.rs` :

1. **`drain_playback_window`** en entier : `SetOutputMute(true)`, `ClearLoop`, les
   `MEASURE_RING_CAP.div_ceil(MEASURE_WINDOW_CAP) + 1` `SeekPlayhead` de stationnement
   (`u32::MAX - drains + offset`), leurs attentes de callback, et le `SeekPlayhead(return_bar)`
   final qui sert de barrière ; le `SetOutputMute(false)` qui suit.
2. **Dans `prepare_playback`** : le cycle `resume` → attente de `transport_position().bar` (délai de
   2 s) → `pause`, la ré-application de `SetLoop` après `ClearLoop` (la boucle n'est plus touchée),
   et le drapeau `seek_bar` qui en dépendait. Il reste le positionnement de la tête
   (`SeekPlayhead(return_bar)` ou `Seek(return_bar)` selon que la séance change).
3. **`rebuild_current_bar`** (`SetSeed` + `Seek(bar)`) pour `SetBpm`/`SetSwing` : inutile quand la
   tête est à une barre de mesure non chargée (départ de piste, après `Seek`/`SeekPlayhead`) ;
   le moteur recompose cette mesure depuis la graine. **À garder** si training veut qu'un
   changement de tempo à l'arrêt **au milieu d'une mesure** s'applique au reste de cette mesure : la
   mesure chargée est audible, elle est toujours gardée (comportement en lecture identique).
4. **`session_start` / `new_piece`** : plus besoin de purger avant de rejouer ; `SetSeed` +
   `Seek(1)` suffisent. **À garder** : `install_profile` qui renvoie `SetBpm`, `SetSwing`, mutes et
   gains après `SetStyleProfile` (celui-ci réinitialise tempo et swing, documenté).
5. Ne plus compter sur `drain_playback_window` pour qu'« aucune composition ne tourne » : le
   stationnement n'est plus nécessaire. Un `SeekPlayhead` très loin reste à éviter pendant un
   `pump` (le compositeur composerait jusqu'à la tête).

`harmonium_practice/tests/startup_window.rs` passe tel quel côté moteur, à l'exception de
l'appel à `drain_playback_window` à supprimer ; son équivalent est `stale_measures_tests.rs`.

## Wasm

Le chemin du playback (`PlaybackEngine`, `LiveHost::detached`, composer) est compilé pour wasm :
`cargo check --target wasm32-unknown-unknown -p harmonium_core -p harmonium_audio -p harmonium`, avec
et sans `odin`. Les tests de scénario sont natifs (`create_offline_engine` n'existe pas en wasm, pas
de lanceur de tests wasm pour `harmonium_host`) ; `harmonium_determinism/scripts/run.sh` compare la
génération natif/wasm (98/98 identiques) et ne passe pas par le playback.

## Vérifications

Voir le message de commit final de la branche pour la liste exécutée.

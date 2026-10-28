# S2 — Hôte unique, commandes partagées, OSC

Branche `wquintal/s2-hote-unique` (depuis `ecb50a0`, G6b incluse). Plan : `specs/015-studio/plan.md`.

## Ce qui a changé

1. **Un seul hôte vivant : `LiveHost`** (`harmonium_host/src/live_host.rs`, ex-`NativeHandle`). Il prend la
   banque principale, la banque de pieds et les presets Odin ; le REPL ne construit plus son propre moteur.
2. **Un seul exécuteur : `LiveHost::apply(EngineCommand)`.** `EngineCommand` absorbe `CliCommand` ; le REPL
   garde son analyseur de texte (texte → `EngineCommand`) et son affichage, et perd `dispatch_command`,
   `handle_clock_command` et `CliCommand`.
3. **Génération pilotée par l'appelant : `LiveHost::pump()`** (composer en avance, retour des boîtes jouées,
   vidage des rapports, snapshots composés rendus). Le REPL l'appelle depuis son fil de génération (50 ms) ;
   le studio l'appellera depuis sa boucle d'interface. `apply` ne compose jamais : les mesures de
   remplacement viennent du `pump` suivant.
4. **OSC dans le host** (`harmonium_host/src/osc.rs`, `rosc`, derrière `standalone`) : même adresse
   (`/harmonium/params`), mêmes quatre flottants, UDP `127.0.0.1:8080` ; le message devient un
   `EngineCommand::SetEmotionParams` remis à un rappel de l'appelant. `harmonium-cli --osc` l'ouvre.
5. **Supprimé** : binaire `harmonium` (`harmonium_host/src/main.rs`, `[[bin]]`), `ctrlc`, `--simulate`,
   cible `make play` et leurs sections de README ; chemin MIDI vivant de bout en bout ; `font_queue`.

## API de l'hôte

| Méthode | Rôle |
|---|---|
| `LiveHost::start(sf2, feet_sf2, odin_presets)` / `start_paused(…)` | ouvre la sortie par défaut ; les presets de démarrage (paramètres, programmes GM du style) sont publiés à la construction |
| `LiveHost::detached(…, sample_rate)` | hôte sans carte son ; rend le `PlaybackEngine` à faire tourner par l'appelant (tests, rendu) |
| `apply(EngineCommand) -> Result<(), String>` | l'exécuteur |
| `pump() -> Vec<MeasureSnapshot>` | génération en avance + rapports |
| `poll_state()`, `take_recordings()` | rapport publié, enregistrements terminés |
| `transport_position()` | position sans verrou |
| `beat_info(now)`, `take_clock_reader()`, `restore_clock_reader()` | horloge prédictive (le REPL prend le lecteur pour `beat`) |
| `musical_params()`, `emotions()`, `session_seed()`, `session_config()`, `profile_bar_info(bar)`, `playhead_bar()` | lectures pour l'affichage |
| `resume()`, `pause()` | transport de la sortie |

`apply` et `pump` prennent `&mut self` : partagé entre fils, l'hôte est derrière un `Mutex` (REPL).
`audio::create_timeline_stream` rend maintenant `(cpal::Stream, EngineControls)` ; `create_offline_engine`
garde sa signature (même `build_engine` en dessous).

## Commandes (`EngineCommand`, état après S2)

Ajoutées (ex-`CliCommand`) : `SetSwing(f32)` (borné par l'hôte à 0,5–0,75), `SetAudibleTempo { bpm,
ramp_pulses }`, `ClearAudibleTempo`, `SetFeetDensity(u8)`, `SetIntensity(f32)` ; le style passe par
`SetStyleProfile`. `beat` n'est pas une commande : c'est une vue du REPL.

Variantes sans commande REPL avant S2 :

| Variante | Décision |
|---|---|
| `EnableRhythm` / `EnableHarmony` / `EnableMelody` | commande REPL `set rhythm\|harmony\|melody on\|off` |
| `SetChannelRoute` | commande REPL `set route <canal> <banque>` |
| `SetOutputMute` | commande REPL `set output mute\|unmute` |
| `SetWriteheadLookahead` | commande REPL `set lookahead <mesures>` |
| `SeekPlayhead` | commande REPL `seek <n> soft` (`seek <n>` reste le saut déterministe) |
| `NewMelody` (absente aussi du REPL) | commande REPL `seed new` (affiche la graine tirée) |
| `SetVelocityBase` | **supprimée** (no-op côté lecture, « géré à la génération » : faux) |
| `ExportTimeline` | **supprimée**, avec `MusicComposer::export_timeline` (écrivait dans le répertoire courant ; `record musicxml` couvre l'export) |
| `ClearStyleProfile` | **supprimée** (doublon de `SetStyleProfile` avec le profil par défaut) |
| `GetState` | **supprimée** (aucun effet ; l'état se lit par `poll_state`) |

Commandes du REPL : `set bpm|volume|time|gain|mute|unmute|route|output|rhythm|harmony|melody|lookahead|seed`,
`emotion` (absolu, relatif, seul), `direct`, `reset [bpm]`, `seed <n>|new`, `style`, `feet`, `intensity`,
`swing`, `tempo <bpm> [N]|off`, `seek <n> [soft]`, `loop <a> <b>|off`, `record`, `stop [format]`, plus les
vues `state`, `beat`, `help`, `quit`.

## Ce qui a été supprimé

- `NativeHandle`/`native_handle.rs` (renommé et réécrit) ; `LiveMidiSender`, `LiveMidiEvent`,
  `MONITOR_CHANNEL`, l'anneau MIDI vivant, `PlaybackEngine::process_live_midi` et son argument `live_rx` ;
  `transport_position_handle`.
- `FontQueue`, `MusicComposer.font_queue` (et le paramètre des constructeurs), `add_soundfont`,
  `flush_fonts`, `PlaybackCommand::LoadFont`.
- `MusicComposer::seek_writehead` (« legacy, non déterministe », seul appelant : le `seek` du REPL).
- Du handle : `measures_buffer` et ses accesseurs, `generate_bars`, `reset_composer`,
  `invalidate_and_regenerate`, `regenerate_with_current_params`, `apply_params_preserving_preview`,
  `set_tuning`, `set_instrument_*`, `set_channel_program` et les autres setters sans consommateur (tout passe
  par `apply`).
- `harmonium_host/src/main.rs`, `[[bin]] harmonium`, `ctrlc`, `--simulate`, `make play`, sections de README.

Les garanties temps réel du playback sont inchangées : seule une lecture d'anneau de moins par tampon.

## Changements de comportement (voulus)

- `seek <n>` rejoue le générateur de façon déterministe (`deterministic_seek`, ce que faisait
  `NativeHandle::seek`) au lieu de l'ancien `seek_writehead`. Le saut garde le même effet audible.
- `emotion a+0.1` part des émotions réellement en vigueur sur l'hôte (donc aussi celles posées par l'OSC),
  et non d'un état local du REPL initialisé à `0.5 / 0 / 0.5 / 0.5` (le moteur démarre à
  `0.5 / 0.3 / 0.2 / 0.4`).
- L'hôte publie à la construction les paramètres et les changements de programme GM du profil (le REPL le
  faisait à la main ; `NativeHandle::start` l'omettait).
- `swing` : l'analyseur ne borne plus, l'hôte le fait ; le REPL affiche toujours l'avertissement.
- OSC : le message passe par `SetEmotionParams`, qui entre en mode émotion (le binaire le faisait une fois au
  démarrage).
- `--osc` est refusé avec `--export`.

À noter, hors périmètre : `SetMasterVolume` est un no-op côté lecture (« appliqué au rendu », mais rien ne
l'applique) ; la commande `set volume` n'a donc aucun effet audible, comme avant.

## Vérification

Machine : macOS, carte son disponible.

- `cargo +nightly fmt --all -- --check` : propre.
- `cargo test --workspace` : 570 réussis, 0 échec, 2 ignorés (73 binaires de test) ; goldens inchangés
  (`HARMONIUM_UPDATE_GOLDENS` jamais posée).
- `cargo test -p harmonium --features odin --test odin_contract_tests` : 5 réussis.
- `cargo clippy --workspace --lib --bins` : 157 avertissements distincts sur `main` (`ecb50a0`, mesuré sur
  une copie `git archive`) contre 149 ici ; aucun nouveau (la différence ne contient que des suppressions :
  le binaire `harmonium` et deux `type_complexity` du REPL).
- Nouveaux tests : `harmonium_host/tests/live_host_tests.rs` (émotions, paramètres de génération jusqu'au
  rapport publié, graine, `Seek` / `SeekPlayhead` sur le transport, boucle refusée, tempo audible puis
  retour, enregistrement MIDI), `harmonium_host/tests/osc_tests.rs` et `osc.rs` (décodage ; client UDP →
  rappel ; client UDP → émotions de l'hôte), tests du REPL et de l'analyseur pour chaque commande ajoutée.
- **Fumée OSC** : `harmonium-cli --osc` lancé, paquet `/harmonium/params 0.9 -0.2 0.4 0.6` envoyé par un
  client UDP Python vers `127.0.0.1:8080`, puis `emotion a+0` dans le REPL : `A=0.90 V=-0.20 D=0.40 T=0.60`
  (avant : défauts du moteur). Rejoué avec `0.1 0.7 0.9 0.2` à deux reprises : `A=0.10 V=0.70 D=0.90 T=0.20`.
  Une première tentative, partie avant que l'écoute ne soit prête, n'avait rien changé ; non reproduite.
- Session REPL scriptée (carte son réelle) : toutes les commandes ajoutées ou migrées répondent
  (`seed new`, `seek 3`, `seek 2 soft`, `set rhythm|output|route|lookahead`, `tempo`, `swing 0.9` avec
  avertissement, `feet`, `intensity`, `loop 1 40` refusé, `record`/`stop`, `style`, `emotion`, `state`).

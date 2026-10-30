# E4 — Notes jouées en direct

Branche `wquintal/e4-notes-jouees`, décision F2 de la phase 018.
Aucune écoute revendiquée : rendu et assertions PCM hors ligne, sans ouvrir de périphérique.

## Décisions et périmètre

- `LiveHost::take_note_sender() -> Option<LiveNoteSender>` transfère une poignée
  `Send`, non `Clone`. Cloner un producteur `rtrb` violerait SPSC ; ajouter un
  verrou au callback violerait la constitution. Deux producteurs fixes suffisent :
  `apply` garde le sien, le thread MIDI prend l'autre. Deux files de 256 événements
  `Copy`, même transport, même code de synthèse et mêmes voix jouées.
- Début de callback : commandes de lecture ordinaires, puis événements joués
  (`apply` avant MIDI, FIFO par file). Les tailles des deux files sont capturées
  avant leur drain : 512 événements au maximum, même si le MIDI continue d'écrire.
  Pas d'ordre global entre producteurs : un NoteOn MIDI peut suivre une coupure
  envoyée par `apply` au même callback. Un appel accepté n'est pas un acquittement
  audio ; sortie suspendue, les événements attendent et les files peuvent saturer.
- `LiveNoteError::{Full, InvalidMidiData}` refuse sans attendre et sans publier.
  `apply` retourne la même erreur convertie en texte. Un NoteOff refusé doit être
  conservé et réessayé, ou remplacé par AllNotesOff après le callback suivant ;
  aucune suppression silencieuse ni priorité cachée des relâchements.
- Voie SoundFont dédiée, propre synthé et réserve de 256 voix préallouées,
  programme global de banque 0 (défaut 0), canaux 0–15 tous mélodiques, dont 9.
  Les paramètres MIDI sont 0-based ; vélocité zéro = NoteOff. Plusieurs zones
  d'une note peuvent consommer plusieurs voix ; saturation : vol de voix limité
  au synthé joué, jamais aux pistes générées.
- `NoteOff` respecte l'enveloppe SF2 ; `AllNotesOff` coupe toutes les voix jouées
  et purge le bloc PCM interne déjà calculé. Réverbération et chorus désactivés.
  Les programmes des attaques suivantes se changent sans couper les notes tenues.
- Ni mute/gain/routage/programme de piste, ni seek/reset de composition ne commande
  les voix jouées. Le volume et le mute de sortie restent globaux. Aucun `pump`
  requis pour jouer : la voie s'entend même avant la première mesure.
- Rendu ajouté après l'enregistreur, avant volume/mute de sortie. Les notes jouées
  ne sont ni composées, ni stockées dans la timeline, ni publiées dans les
  `MeasureSnapshot`, ni incluses dans les exports WAV/MIDI/MusicXML. Il s'agit du
  retour instrumental direct, pas d'un nouvel enregistreur d'improvisation.
- La banque principale est chargée une seconde fois, au démarrage seulement :
  coût mémoire assumé pour isoler complètement les voix et contrôleurs. Sans
  banque exploitable, silence. Avec `odin`, cette voie reste SoundFont ; aucun
  choix de preset Odin ni morphing n'est exposé pour les notes jouées.

### Pourquoi ce n'est pas le retour du chemin supprimé en S2

S2 a supprimé le chemin MIDI vivant et les setters parallèles en établissant
`LiveHost::apply` comme exécuteur partagé (`s2-hote-unique.md`, sections 1 et
« Ce qui a été supprimé »). Recréer un moniteur privé avec son instrument et ses
règles propres au seul client MIDI ferait à nouveau deux façons de jouer.
Ici les variantes ajoutées **en fin** d'`EngineCommand` et la poignée utilisent
le même événement `PlayedNoteEvent`, la même discipline SPSC et le même synthé ;
la seconde file ne sert qu'à conserver deux producteurs sans verrou audio.
`ClockReader` est lui aussi `Send` et non clonable : les deux poignées transfèrent
un rôle unique, pas l'hôte.

### Allocation tierce retirée, pas masquée

L'ancien `VoiceManager::process_event` désactivait `RTCheckAllocator` autour de
chaque NoteOn : oxisynth 0.1.0 construisait un `Vec` pour initialiser un tableau
fixe de 64 modulateurs. Son stockage de voix pouvait aussi croître aux premières
attaques. Cela ne satisfaisait pas l'acceptation E4.

La copie locale `vendor/oxisynth` remplace le temporaire par `[None; 64]` et
réserve la capacité de voix à la construction/configuration. Aucun algorithme
DSP ni politique de vol de voix ne change. L'exemption et son API inutilisée sont
supprimées : premier NoteOn, allocations/libérations et saturation passent sous
la garde complète du callback. Aucun verrou ajouté ; SPSC et mémoire propre au
callback uniquement sur la nouvelle voie. L'allocateur détecte les allocations,
pas les verrous : l'absence de verrou relève aussi de cette structure de code.
Un journaliseur installé par l'appelant doit rester temps réel sur les messages
éventuels du synthé tiers.

Provenance, correctifs et licence **LGPL-2.1**, déjà celle de la dépendance amont :
[`vendor/oxisynth/README.md`](../../vendor/oxisynth/README.md). Ce code tiers n'est
pas relicencié MIT/Apache ; ses obligations de distribution restent applicables.

## Preuves comportementales observées

`cargo test -p harmonium --test played_notes_tests --test realtime_callback_tests`
passe : 13 tests E4 et le test complet callback/enregistreur existant. Après ajout
de la rafale de saturation, les 13 tests E4 passent également.

- Vraie banque `assets/test.sf2`, aucune exemption de l'allocateur host.
- Poignée déplacée dans un thread, hôte détruit : attaque puis relâchement possibles.
- Les deux entrées attaquent dès le premier buffer, sans composer ; relâchement
  vers silence en moins de cinq secondes audio, coupure générale au callback suivant.
- Programmes réellement différents, les seize canaux mélodiques, vélocité zéro,
  données invalides refusées ; changement d'instrument dans une même session.
- Les deux files pleines refusent les arrêts sans blocage, puis récupèrent de la
  place au callback ; ordre inter-producteurs exercé.
- Snapshots sérialisés strictement égaux, avec les mêmes commandes de composition,
  avec ou sans les événements joués des deux producteurs.
- Mutes, gains et seek générés ne modifient aucun échantillon de la note jouée.
  NoteOff et AllNotesOff joués ne coupent pas le couple canal/hauteur généré.
  Une rafale de 512 attaques dépasse la réserve jouée ; après sa coupure, le PCM
  généré reste exactement égal au témoin, donc aucun vol de voix générées.
- Mute et rampe de volume de sortie coupent aussi le retour direct, sans le relâcher.

Un exécutable temporaire a lancé le vrai `LiveHost::detached` et son
`PlaybackEngine`, sans périphérique, en debug puis release. Thread MIDI,
programme 15, canal 0, note 60, vélocité 100 ; buffers stéréo de 256 frames à
44 100 Hz. Énergie `sum(sample²)` au premier buffer : **0,472529829** par poignée,
puis **0,416292995** pour la note 64 par commande. Après NoteOff et cinq secondes
rendues : **0** ; après AllNotesOff : buffer exactement nul. L'exécutable a été
supprimé après mesure, les tests consommateurs restent versionnés.

Le transport applique les événements présents au début du callback dans ce
callback : **aucune mesure ni composition à attendre**. Oxisynth synthétise par
blocs de 64 frames et peut conserver jusqu'à 63 frames de PCM antérieur, soit
1,429 ms à 44 100 Hz ; c'est une limite de synthèse distincte de la file. La preuve
énergétique porte sur des buffers de 256 et 512 frames ; elle n'affirme pas que
chaque instrument possède une attaque audible dès un buffer d'une seule frame.
La latence du périphérique et du système MIDI n'est pas mesurée.

## Mesure de `pump()` et motivation F2

Mesures locales macOS arm64, `Instant`, sans `odin`, sans banque et sans sortie
pendant la mesure de composition. Pour chaque profil et chaque graine 1–16 :
création d'un hôte détaché, installation du profil, SetSeed, lookahead 16 ;
chronométrage du premier `pump` (16 snapshots), rendu d'un buffer, `pump` sans
composition, changement d'émotions `(0.9, -0.5, 0.9, 0.8)` puis `pump` après
invalidation. La création de l'hôte et les commandes sont hors intervalle ;
le temps mesuré inclut le retour des snapshots. Médiane = moyenne des deux
observations centrales sur 16. Ce sont des observations locales, pas une borne
maximale temps réel ni une mesure sous charge du studio.

| Profil | Cas | Debug médiane (ms) | Debug max (ms) | Release médiane (ms) | Release max (ms) |
|---|---|---:|---:|---:|---:|
| reel_quebecois | initial, 16 mesures | 4,152750 | 6,032542 | 0,185750 | 0,260833 |
| reel_quebecois | sans composition | 0,004063 | 0,004708 | 0,000396 | 0,000666 |
| reel_quebecois | invalidation | 4,606229 | 8,515959 | 0,137896 | 0,189416 |
| nt_desert | initial, 16 mesures | 3,267854 | 3,792500 | 0,189271 | 0,210167 |
| nt_desert | sans composition | 0,003938 | 0,006667 | 0,000333 | 0,000500 |
| nt_desert | invalidation | 1,331937 | 1,661666 | 0,057062 | 0,059917 |
| isaac_basement | initial, 16 mesures | 1,790604 | 1,915500 | 0,111459 | 0,150042 |
| isaac_basement | sans composition | 0,003834 | 0,004167 | 0,000333 | 0,000583 |
| isaac_basement | invalidation | 1,071896 | 1,104666 | 0,055854 | 0,083458 |

Un buffer de 256 frames à 44 100 Hz dure 5,805 ms. `pump` tenant `&mut self`,
un client MIDI passant par le verrou de l'hôte attendrait sa fin : **8,516 ms
observées en debug**, plus le prochain callback. Même le maximum release mesuré,
**0,261 ms**, n'est pas une garantie sous charge. La poignée indépendante retire
cette dépendance à la composition plutôt que d'essayer d'accélérer le verrou.

## Utilisation prévue

Le [README du host](../../harmonium_host/README.md#notes-jouées-en-direct-midi-studio-jeux)
montre acquisition, thread MIDI et commandes équivalentes.

- **Training** : conserver la poignée dans son entrée MIDI ; `set_program`,
  `note_on`, `note_off`, `all_notes_off`. La grille reste un profil de composition,
  les notes d'impro ne la modifient jamais.
- **Studio** : poignée au clavier MIDI ; `EngineCommand` pour le clavier visuel et
  le choix d'instrument ; traiter explicitement la contrepression. Pas de voie
  privée dans le studio. Aucun fichier du studio modifié par E4.
- **Bevy** : déplacer la poignée dans une tâche d'entrée MIDI, commandes depuis
  les systèmes ordinaires. Ni tâche ni périphérique MIDI imposés par le moteur.
  Aucun fichier Bevy modifié par E4.
- **Web** : même API dans un rappel Web MIDI sur le thread de la page ; aucun
  thread nécessaire. La poignée n'autorise pas à déplacer les objets JS de l'hôte.

Le nom de paquet Cargo du répertoire `harmonium_host/` est **`harmonium`** : les
commandes de vérification utilisent donc `-p harmonium`, et non `-p harmonium_host`.
Goldens, `TUNING_VERSION` et `harmonium_core/src/trad/` ne sont pas modifiés.

## Vérifications de compilation et qualité

Commandes réussies :

```sh
cargo check --target wasm32-unknown-unknown -p harmonium_core -p harmonium_audio -p harmonium
cargo check --target wasm32-unknown-unknown -p harmonium_audio -p harmonium --features odin
cargo check -p harmonium_studio -p harmonium_bevy
cargo check -p harmonium-cli
cargo +nightly fmt --all -- --check
cargo +stable clippy -p harmonium_core -p harmonium_audio -p harmonium -p harmonium-cli \
  --all-targets --features harmonium/odin,harmonium_audio/odin \
  -- -A clippy::all -W clippy::unwrap_used -W clippy::panic \
  -W clippy::todo -W clippy::expect_used
```

Clippy suit les options du `Makefile` ; il publie encore les avertissements
`unwrap`/`expect` déjà présents dans le moteur et ses tests. Aucun diagnostic sur
`live_notes.rs`, `played_notes.rs` ou `played_notes_tests.rs`, ni nouvel appel
`unwrap`/`expect` non justifié sur les lignes ajoutées. L'unique `expect` de
construction du synthé porte sur un descripteur constant valide, avec justification
locale. Les échecs de préparation des tests consommateurs sont des assertions.
Les avertissements Cargo de compatibilité future concernent `glutin`,
`glutin-winit`, `nix` et `winit`, pas la voie jouée.

Le format tiers est conservé par `vendor/oxisynth/rustfmt.toml` ; le contrôle
`fmt --all` passe sans reformater l'ensemble des sources amont.

### Suite finale workspace

`cargo test --workspace -q --no-fail-fast` est vert : **728 tests réussis,
2 ignorés, 95 suites**. Aucun golden régénéré ; `TUNING_VERSION` reste à 21.
Le test `seek_to_bar_64_under_50ms` n'a pas nécessité de relance isolée.

### Suites Odin explicites

- `cargo test -p harmonium --features odin` : **118 tests réussis, 25 suites**.
- `cargo test -p harmonium_audio --features odin` : **20 tests réussis,
  1 ignoré, 4 suites**.

Le premier lancement chaîné avait expiré à 300 secondes, après des suites vertes
dont les 13 scénarios E4 ; la relance avec une échéance de 3 600 secondes a terminé
les deux commandes en 352 secondes. Aucun échec de test masqué ou désactivé.

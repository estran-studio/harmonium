# A6 — Swing selon le tempo, ternaire, 3/4, 6/8, feels latins

Date : 2026-11-03. Branche `wquintal/a6-swing-metriques`, partie de `main` à `0fe8b52` (`TUNING_VERSION`
26 **inchangé**). Tout est optionnel en données : un profil sans `[swing]`, sans `[clave]`, sans `voices`
ni `probability` garde ses dumps, ses WAV et son MIDI (preuves plus bas). Aucun profil distribué n'est
touché (B1, B3, B5, B6 déclareront leurs courbes).

## Ce qui est livré

| Brief | Livré |
|---|---|
| 1. Swing selon le tempo | `[swing]` (ou `swing.toml`) : une courbe tempo → rapport **par piste** (`harmonium_core/src/swing.rs`), mesurée dans JTD. La piste `reference` (défaut `PercussionHigh`) fixe le rapport de la grille (`Measure::swing_ratio`) ; les autres pistes dont le rapport diffère sont replacées par un décalage de leurs attaques (`performance.offset`) qui inverse la géométrie du swing : leur croche de contretemps tombe à leur rapport, temps et longueur de mesure intacts. Bornes `0.5..0.75` **inchangées** : les mesures vont de 0,566 à 0,721. |
| `SetSwing` prioritaire | `MusicalParams::swing_explicit`, posé par `MusicComposer::set_swing_ratio` (REPL `swing`, `--swing`, `LiveHost`) ; rapport imposé à **toutes** les pistes, courbe ignorée ; retiré par `reset` et par le chargement d'un profil. Sans commande directe, l'écart entre `swing_ratio` et `meter.default_swing_ratio` décale toutes les courbes (point d'entrée de l'habillage d'A2). **Point d'entrée unique** : `SwingByTempo::resolve`, appelé par `TuneGenerator::swing_for`. |
| 2. Ternaire, 12 pas, grille impaire ou composée | `BarTiming::step_offset` généralisé : le swing partage la noire au pas `⌈pas par noire / 2⌉`, donc aussi sur **3 pas par noire** (12 pas en 4/4, triolets) ; mêmes entiers qu'avant sur 4 pas (preuve : dumps et rendus identiques). Grille composée (6/8 sur 12 pas, 12/8 sur 24) : **jamais de swing**, par construction (aucune mesure de swing composé dans JTD, le ternaire est la grille). `BarTiming::swing_unit_steps` dit si une grille swingue. |
| Export MIDI sur grille ternaire | **Défaut trouvé et corrigé** : le MIDI enregistré et `timeline_to_midi` comptaient 120 ticks par pas : une mesure de 4/4 sur 12 pas durait 1440 ticks au lieu de 1920 (un tempo 25 % trop rapide dans un DAW). `events::midi_ticks_per_step` : 480 ticks par noire quelle que soit la grille (120 par pas sur 4 pas par noire : **octets inchangés** pour tous les profils distribués). |
| 3/4 et 6/8 sous grille | Fixtures `grille_valse_jazz` (3/4, 12 pas, profil `loop`, arc, phrases de comping et de batterie, relances) et `grille_six_huit` (6/8, 12 pas, mêmes mécanismes) ; tests de contrat (arc, phrases, instants du plan). |
| 3. Micro-timing par voix | `performance.tracks.<piste>.voices = [{ note, on_beat, off_beat?, jitter? }]` : la voix est la **note MIDI** (ride 51, pédale 44, caisse claire 38, grosse caisse 36) ; une piste à voix traite chaque note seule, donc le ride, la pédale et la caisse claire d'un même pas ne partagent plus un décalage. |
| Coup probabiliste | `probability` (0 exclu, 1 inclus) sur un coup de pattern ou de relance ; tirage **sans état** (`percussion::hit_plays`) fonction de la graine d'accompagnement, de la mesure, du pas, de la note et de la piste : stable par graine, identique après un seek. |
| 4. Clave | `[clave] direction = "three_two" \| "two_three" \| "either"` (tiré par air) et `clave = "three" \| "two"` sur les phrases de comping et de batterie : une phrase ne peut **commencer** que sur une mesure de son côté du cycle de deux mesures (y compris la phrase de fin de passe). Montuno et bossa : comping anticipé (`anticipate`) et basse anticipée (gabarit, `pickup.when = "bar_end"`) déjà exprimables, vérifiés par la fixture. |
| Rôles latins (point 7) | Congas, bongos, timbales, cloche, claves, shaker : `[[voices]]` de `percussion.toml` sur les deux pistes existantes, par touche GM. Touches 56, 60-66, 69, 70, 75 et 82 **présentes** dans les kits 0, 32 et 40 de `GeneralUser-GS.sf2` (`a6-roles-latins-sf2.py`, 36 lignes sur 36 : instruments « Accessory Drums 1 » et « Accessory Drums 2 »). |
| 5. Fixtures sans Odin | `tests/fixtures/profiles/` : `grille_shuffle_ternaire`, `grille_valse_jazz`, `grille_six_huit`, `grille_clave`. |
| Sonde natif = wasm | `harmonium_determinism` : sonde `swing_fixtures.txt` (les quatre fixtures lues en mémoire, 2 tempos, 24 mesures, empreinte de toutes les attaques et rapport de grille). |

Documentation d'usage : `profiles/README.md`, « Swing, métriques et feels latins (phase 019, A6) ».

## Preuves chiffrées

### Rapport de swing mesuré sur le MIDI exporté, par tempo et par piste, contre JTD

`a6-mesure-swing.py` : CLI `--export --record-midi`, tempo imposé par automation, 3 graines × 96
mesures, médiane des attaques de contretemps (0,45 à 0,85 de la noire, définition de
`references/jtd-swing-tempo`). Colonne « JTD » : `a6_table.curve` (batterie, basse, piano) au tempo.
Fixture `grille_shuffle_ternaire` **sans** micro-timing de voix (le rapport est celui du swing seul) :

| Tempo | Batterie (ride) mesuré / JTD | Basse mesuré / JTD | Comping mesuré / JTD |
|---|---|---|---|
| 110 | 0,721 / 0,721 | 0,715 / 0,715 | 0,702 / 0,702 |
| 150 | 0,713 / 0,712 | 0,706 / 0,706 | 0,702 / 0,702 |
| 190 | 0,673 / 0,673 | 0,688 / 0,688 | 0,673 / 0,673 |
| 230 | 0,633 / 0,633 | 0,671 / 0,671 | 0,644 / 0,644 |
| 270 | 0,594 / 0,595 | 0,654 / 0,653 | 0,615 / 0,615 |

Écart maximal **0,001** (1 152 attaques par cellule pour la batterie et la basse, 380 pour le comping).
`grille_valse_jazz` (3/4) : ride 0,721 / 0,713 / 0,673 / 0,633 / 0,594 (n = 140), comping 0,702 / 0,702 /
0,673 / 0,644 / 0,615 (n = 472), mêmes écarts. Avec le micro-timing de voix de `grille_shuffle_ternaire`
(ride +0,045 pas sur le temps, +0,07 hors temps) le ride mesure 0,742 à 110 BPM : +0,021, l'effet du
décalage de voix, attendu. Témoin : `wjazzd_swing`, **sans courbe**, mesure 0,710 à tous les tempos
(0,115 au-dessus de JTD à 270 BPM : ce que B1 corrigera en déclarant la courbe) ; inchangé par A6.

### Placement sur la grille ternaire

Tests de contrat sur le plan partagé (`TimePlan`) et sur le MIDI enregistré :

- 4/4 sur 12 pas : 4 noires par mesure à la flick près, temps aux pas 0, 3, 6, 9 ; à `swing = 0,6667`
  chaque piste (ride, basse, comping) mesure 0,6667 ± 0,001 ; ride sur les seuls pas {0, 2, 3, 5, 6, 8, 9, 11}.
- Le pas 2 des trois suit le rapport demandé (0,5 → 2/3, 0,6, 0,72) à 10⁻⁴ de noire, les temps ne bougent pas.
- MIDI : mesure de 1920 ticks en 4/4 sur 12 pas (dernière attaque de 36 mesures rendues entre les ticks
  33 × 1920 et 37 × 1920), 1440 en 3/4 ; ride du 3/4 sur les temps ou à `rapport × 480` ± 2 ticks,
  jamais ailleurs.
- 6/8 : `swing_unit_steps = None`, pas 0 à 11 à ± 1 flick de la grille droite **même sous `swing = 0,7`** ;
  pulse de six pas = demi-mesure exacte.
- 3/4 : 3 noires par mesure, `swing_unit_steps = 4`, ride sur {0, 4, 6, 8}, 0,7018 à 160 BPM (JTD : 0,7018).

### Alignement sur la clave

`grille_clave`, 16 graines × 48 mesures (4 chorus) : **les 768 mesures** portent la clave son sur le côté
attendu, « trois » et « deux » alternent sans faute d'un chorus à l'autre, le comping est sur le même côté
que les claves **mesure par mesure**, et les 16 graines donnent des claves 3-2 **et** 2-3. Témoin : sans la
contrainte (clé `clave` retirée des phrases), plus de 100 mesures sur 768 où comping et claves divergent.
Basse : à chaque changement d'accord, la relance du pas 14 joue la fondamentale de l'accord suivant ;
comping : l'attaque anticipée du pas 14 joue les classes de hauteur de l'accord que la mesure suivante attaque.

### Coups probabilistes

`grille_shuffle_ternaire`, densité 2, 10 graines × 48 mesures : note fantôme à 20 % au pas 5 → fréquence
dans 0,15..0,25 ; à 10 % au pas 11 → dans 0,06..0,14 ; 10 suites distinctes (≥ 9) ; même graine, mêmes
coups ; densité 1 : aucune. Rejeu après un seek à 7, 19 et 33 : mesures identiques (toutes pistes).

### Micro-timing par voix

Ride +45 / +70 millièmes de pas (temps / hors temps), pédale −70, caisse claire −20, grosse caisse −25 ;
un temps où ride, pédale et caisse claire jouent ensemble : trois décalages distincts (45, −70, −20) ;
sans `performance`, aucun décalage de batterie.

### Non-régression

| Contrôle | Résultat |
|---|---|
| Dumps `harmonium_determinism` (98 fichiers) contre `main` | **98 / 98 identiques** |
| Natif = wasm (`run.sh`) | **99 / 99** (98 + la sonde `swing_fixtures.txt`) |
| `--export --record-wav` et `--record-midi`, 16 profils × 2 graines × 24 mesures, contre le binaire de `main` | **64 fichiers (32 WAV, 32 MIDI) identiques** octet pour octet |
| `TUNING_VERSION`, goldens | inchangés |
| Clippy | 867 avertissements, comme `main` (comptes par fichier identiques) |
| `cargo fmt --check` | propre |
| `seek_to_bar_64_under_50ms` | minimum sur 20 graines : 23,2–23,3 ms sur la branche, 23,2–24,5 ms sur `main` (la médiane bouge avec la charge de la machine) |

Mutations vérifiées (chacune fait échouer au moins un test de `swing_metriques_tests`) : remappage par piste
désactivé (2 échecs), priorité de l'hôte ignorée (2), probabilité ignorée (1), alignement de clave ignoré (1),
micro-timing par voix ignoré (1), swing impair désactivé (5).

## Tests

- `harmonium_core/tests/swing_metriques_tests.rs` (20 tests) : courbe par piste et par tempo sur 12 pas et
  en 3/4 (5 tempos, contre JTD à 0,004 près), monotonie et bornes de la courbe, priorité de la commande
  directe, dosage, profil sans courbe, triolets, déplacement du pas 2, jazz waltz, 6/8 droit, arc de chorus
  et phrases sous 3/4 et 6/8, micro-timing par voix, coups probabilistes (fréquence, graine, seek), clave
  (alignement, témoin, anticipations), erreurs de schéma avec chemin du champ.
- `harmonium_host/tests/midi_grille_ternaire_tests.rs` (4 tests) : MIDI enregistré sur 12 pas et en 3/4
  (ticks par mesure, courbe de chaque piste, commande directe), `set_swing_ratio` / `reset` / profil.
- Unitaires : `time_plan` (grille impaire, pulse composée), `swing.rs` (courbe, résolution, remappage, schéma).

## Choix documentés (sans question au propriétaire)

- Un profil de swing par **piste** et non par rôle de batterie : les deux pistes de percussion portent la
  même courbe dans les fixtures.
- Le micro-timing par voix est en **fractions de pas** comme tout `performance` (indépendant du tempo) : les
  médianes en ms du GMD (ride +9 / +14 ms) se convertissent au tempo de référence du style (B3, B6, B4).
- `clave` contraint le **début des phrases**, pas des patterns isolés : un pattern ordinaire est tiré une
  fois par partie, sans mesure de référence.
- Pas de swing composé : aucune donnée, et la grille de 12 pas en 4/4 couvre le shuffle 12/8.

## Limites

- Le tempo lu est celui de la mesure à sa génération : un `SetTempo` audible qui ne régénère pas les mesures
  ne déplace pas le swing déjà publié (le tempo imposé par `set_bpm` régénère).
- JTD : trios de piano, 102 à 299 BPM, swing seulement. Les courbes des fixtures sont celles de JTD ; **B6**
  (shuffle) devra citer sa propre source (GMD `blues/shuffle`) et B3 (latin) la sienne : aucune mesure de
  swing ni de micro-timing latin ici.
- Valeurs de batterie des fixtures (ride, pédale, notes fantômes, tumbao) : valeurs d'exercice, aucune
  mesure de corpus ; non distribuées.
- Écart maximal à la piste de référence : 0,12 (décalage sous le demi-pas que borne `performance`).

## Manques moteur relevés

1. **MusicXML** (`timeline/export.rs`, `divisions = 4`) et champ `beat` du transport
   (`TICKS_PER_BEAT = 4`) supposent encore quatre pas par noire : faux sur 12 pas en 4/4.
2. Swing qui suit un `SetTempo` audible sans régénération (le plan partagé connaît le tempo audible, le
   générateur non).
3. La basse en marche (`walking`) n'a aucune attaque de contretemps : sa courbe de swing ne s'entend que sous
   des coups ou des motifs (A4, croches de passage).
4. Un pattern de batterie isolé ne peut pas porter de côté de clave (seulement les phrases) ; une clave de
   trois mesures ou plus (rumba 3-2 de quatre mesures) demande un cycle paramétrable.
5. Micro-timing en millisecondes (le piano de JTD est en retard d'environ 17 à 21 ms, constant en ms) :
   exprimé en fraction de pas, il dérive avec le tempo.

## Reproduire

```sh
cd ~/Project/bascanada/harmonium
cargo build --release -p harmonium-cli
~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/a6-mesure-swing.py \
    --cli target/release/harmonium-cli --out /tmp/harmonium-019/a6/mesures.json
python3 specs/019-bibliotheque-accompagnements/a6-roles-latins-sf2.py assets/GeneralUser-GS.sf2
cargo test -p harmonium_core --test swing_metriques_tests
cargo test -p harmonium --test midi_grille_ternaire_tests
```

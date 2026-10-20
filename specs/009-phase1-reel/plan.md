# Phase 1 — Un reel identifiable

**Statut** : point d’arrêt — P1-T1..T5 mergées, P1-T6 en attente du verdict | **Date** : 2026-10-19 | **Dépend de** : phase 0 (horloge, swing, rampe)

## But

Générer en continu un **reel** québécois que des oreilles québécoises
reconnaissent comme tel, à partir d'un seed. Ensemble minimal : violon,
guitare (chop), basse, pieds (talon, pointe). Synthèse SoundFont GM
provisoire. Pas encore de pilotage par jauges (phase 3).

Ce qui rend un reel identifiable, et que cette phase doit produire :
**forme AABB stricte**, **cellules d'une mesure** en croches, **cadences
imposées**, **accompagnement oom-chop**, **podorythmie**, **swing léger**.
La mélodie n'a pas besoin d'être géniale ; elle doit se répéter.

## Décisions

### D1 — Un générateur trad à côté, puis on coupe

Nouveau module `harmonium_core::trad`. `TradGenerator` produit des `Measure`
avec la même signature que `TimelineGenerator::generate_measure(bar, rng)` ;
le `Writehead`, le handoff et le playback ne changent pas. Le composer
choisit le générateur via `MusicalParams.style: Style { Legacy, Reel }`.

C'est un chemin parallèle **transitoire et daté** : la suppression de
`Legacy` (TimelineGenerator, séquenceur euclidien/PerfectBalance/ClassicGroove,
Steedman, Tonnetz, LCC, mélodie Markov) est la tâche P1-T6, déclenchée par la
validation à l'oreille de P1-T5. Pas avant : on garde de quoi comparer.

### D2 — Pistes

`TrackId` devient `{ Melody, Chords, Bass, Heel, Toe }` (canaux 0..4).
Le générateur Legacy écrit lead→`Melody`, bass→`Bass`, snare→`Heel`,
hat→`Toe` jusqu'à sa suppression. Rupture assumée du snapshot : bump de
`TuningParams`, golden régénéré **dans le même commit**, une seule fois.

### D3 — Mesure et grille

Reel noté 2/2, 16 pas, pulse = blanche (8 pas, `PulseSpec` de la phase 0).
Croche de tune = 2 pas. Tempo nominal 220–240 à la noire (110–120 à la
blanche) ; swing 0.56 par défaut du style.

### D4 — Plan de tune

`TunePlan { key: PitchClass, mode: Mode, strains: [Strain; 2], scheme: AABB }`,
tiré au seed en début de tune ; un tune = 32 mesures, puis un nouveau plan
(même tonalité ou relative, choix au seed).

Règles par strain de 8 mesures (tirées des structures réelles du répertoire) :
- mesures 5-6 = mesures 1-2, **à l'identique** (question/réponse) ;
- mesure 4 : dernière note longue sur un degré de dominante (2, 5 ou 7) ;
- mesure 8 : finit sur la tonique (degré 1), noire tenue ou croche + pickup
  vers la reprise ;
- strain B : tessiture plus haute (centre vers le degré 8) que A ;
- la reprise rejoue la strain **à l'identique** en phase 1 (la variation
  d'ornementation arrive en phase 2).

### D5 — Cellules (`trad::cells`), en données

Une cellule = 8 croches exprimées en degrés de la gamme du mode, octave
relative (`1` = tonique, `8` = tonique +1, `5,` = quinte grave), `-` =
prolongation de la note précédente. Banque en TOML embarquée
(`harmonium_core/src/trad/reel_cells.toml`, `include_str!`), chargée dans
`TuningParams` (constitution V). Chaque cellule porte un `role` :
`open` (début de phrase), `run` (mouvement conjoint), `arp` (arpège),
`turn` (broderie), `cad_half` (mesure 4), `cad_full` (mesure 8).

Graines de la banque (ré majeur pour l'oreille : 1 = D) :

```toml
cells = [
  { role = "arp", notes = "1 3 5 3 1 3 5 3" },  # D F# A F# — l'arpège de reel
  { role = "arp", notes = "8 5 3 5 8 5 3 5" },
  { role = "run", notes = "5 4 3 2 1 2 3 4" },
  { role = "run", notes = "1 2 3 4 5 6 7 8" },
  { role = "run", notes = "8 7 6 5 4 3 2 1" },
  { role = "turn", notes = "5 6 5 4 3 4 3 2" },
  { role = "turn", notes = "3 2 1 2 3 5 6 5" },
  { role = "open", notes = "5 5 3 5 6 5 3 5" },  # répétition de note, typique au violon
  { role = "open", notes = "1 1 5, 1 3 1 5, 1" },
  { role = "cad_half", notes = "4 3 2 1 2 - - -" },
  { role = "cad_half", notes = "3 4 5 4 5 - - -" },
  { role = "cad_full", notes = "3 2 1 7, 1 - - -" },
  { role = "cad_full", notes = "5 4 3 2 1 - 5, -" },  # tonique + pickup
]
```

Étendre à ~30 cellules en respectant : intervalles majoritairement conjoints
ou en arpège d'accord ; au plus un saut > quinte par cellule ; ambitus de
cellule ≤ une octave ; pas de chromatisme.

### D6 — Ligne de violon

Par mesure : rôle imposé par la position (1→`open`, 4→`cad_half`,
8→`cad_full`, 5-6 copiées), sinon tirage pondéré entre `run`/`arp`/`turn`
avec deux contraintes : continuité (premier degré de la cellule à ≤ une
tierce du dernier de la précédente, transposition d'octave autorisée) et
contour de strain (A centré sur 3-5, B sur 6-8). Tessiture G3–B5 ; ce qui
sort est transposé d'une octave. Vélocités : temps forts (croches 1, 5)
+12, croches paires −10.

### D7 — Harmonie

Un accord par demi-mesure, choisi **après** la mélodie, parmi
`{I, IV, V}` (+ `vi` en majeur, `bVII` en mixolydien) par vote des notes de
la demi-mesure pondérées (temps fort ×2). Égalité : I > V > IV. Imposé :
mesure 4 seconde moitié → V ; mesure 8 → V puis I. Harmonie identique aux
reprises.

### D8 — Accompagnement reel

- Basse : fondamentale sur le pas 0, quinte (ou fondamentale) sur le pas 8,
  noires.
- Guitare (`Chords`) : « chop » sur les pas 4 et 12, accord de 3 sons serrés
  autour de C4, durée 1 pas, vélocité accentuée.

### D9 — Podorythmie, en données

Trois densités (`feet_density` 0/1/2 dans les params, défaut 1) :

| Densité | Talon (`Heel`) | Pointe (`Toe`) |
|---|---|---|
| 0 | 0, 8 | — |
| 1 | 0, 4, 8, 12 | 2, 6, 10, 14 |
| 2 | 0, 4, 8, 12 | 2, 3, 6, 7, 10, 11, 14, 15 (pas impairs −20 vélocité) |

`[INFERENCE]` Ces patterns sont un point de départ : ils doivent être
validés à l'oreille contre de vrais enregistrements de podorythmie et
corrigés dans la table, pas dans le code.

### D10 — Sons GM provisoires

Melody → 41 (violon, programme GM 40), Chords → 25 (guitare acier, 24),
Bass → 33 (basse acoustique, 32), Heel → percussion 36, Toe → percussion 37
(canal 9). Les pieds seront remplacés par une SF2 dédiée avec de vrais pieds.

Violon retenu à l'oreille (comparatif SK3, 2026-10-20, contre FluidR3,
MuseScore General et Sonatina) : **GeneralUser GS, programme 40**. On le
garde. Défaut connu, commun aux quatre : attaque de 250 à 360 ms pour une
croche de 130 ms à 230 la noire. Le coup d'archet se travaille en phase 2
dans le moteur (durée de note, accents), pas en changeant de banque.

### D11 — Arrangement : tours et entrées progressives

Demande du propriétaire, à l'écoute de *Reel Béatrice* : le violon part seul,
puis les autres instruments entrent. Un air est joué plusieurs tours AABB
(3 par défaut). Le point d'entrée de chaque piste (tour, partie, passe) est
dans `reel_arrangement.toml`. Les drapeaux `enable_*` du jeu restent un
plafond. La position dans l'air est exposée dans `MeasureSnapshot`. Tâche
P1-T8, à lancer après P1-T7.

*Reel Béatrice* est la cible d'oreille du propriétaire. Traits à viser en
phase 2 : mode mineur (la mineur), triolets de notes répétées (`(3eee`),
trois parties (AABBCC), ambitus large (la2 à do6).

## Découpage

| Tâche | Contenu | Dépend de | Modèle |
|---|---|---|---|
| P1-T1 | Types `trad` (Mode, TunePlan, Cell, banque TOML ~30 cellules), `Style`, renommage `TrackId` + golden | — | gemini-4-argon |
| P1-T2 | `FiddleLine` : plan de tune + ligne de violon (D4, D6) | T1 | gemini-4-argon |
| P1-T3 | Podorythmie (D9) | T1 | gemini-3.8-flash |
| P1-T4 | `TradHarmonizer` + accompagnement (D7, D8), sur une ligne quelconque | T1 | claude-sonnet-5-5 (antigravity) |
| P1-T5 | `TradGenerator` câblé au composer, canaux + GM (D10), CLI `style reel`, WAV de démo | T2, T3, T4 | gemini-4-argon |
| P1-T6 | Suppression du chemin Legacy | validation à l'oreille | — |

## Acceptation

1. **Forme** (tests sur le snapshot) : pour 20 seeds, mesures 5-6 = 1-2 dans
   chaque strain ; chaque strain jouée deux fois à l'identique ; mesure 8 de
   chaque strain finit sur la tonique ; mesure 4 sur 2, 5 ou 7.
2. **Harmonie** : accords des reprises identiques ; cadences V/V-I aux
   positions imposées.
3. **Accompagnement et pieds** : positions exactes des tables D8/D9.
4. **Déterminisme** : même seed ⇒ même snapshot ; deux seeds ⇒ tunes
   différents.
5. **Démo** : `harmonium-cli`, `style reel`, 64 mesures enregistrées en WAV
   et MIDI dans `demos/` (non versionnés).
6. **À l'oreille (humain)** : test en aveugle, 10 générés / 10 réels ;
   ≥ 70 % des générés reconnus « reel ». Déclenche P1-T6.

## Risques

- **Générique plutôt que trad** : mitigé par la répétition stricte et les
  cadences imposées ; si l'écoute échoue, on corrige la banque (données)
  avant l'algorithme.
- **Violon GM** : laid mais suffisant pour juger la forme et le rythme.
- **Pieds mal transcrits** : tables en données, correctibles sans code.

## État au point d'arrêt (2026-10-19)

| Tâche | État |
|---|---|
| P1-T1 Fondations (`TrackId` 5 pistes, `Style`, `trad::{mode, cells, plan}`, 31 cellules) | mergée |
| P1-T2 Plan de tune + ligne de violon | mergée |
| P1-T3 Podorythmie (table en données) | mergée |
| P1-T4 Harmonisation + accompagnement | mergée |
| P1-T5 `TradGenerator` câblé, CLI `style`/`feet`, programmes GM | mergée |
| SF1 GeneralUser GS par défaut (`make soundfont`), détection GM réelle | mergée |
| P1-T6 Suppression du chemin Legacy | **en attente** |

Vérifié par le coordinateur, sur le flux MIDI des démos (seeds 1 et 2) :
A1 = A2, B1 = B2, mesures 5-6 = 1-2, B ≠ A ; pieds audibles et différents
entre `feet 1` et `feet 2`. 434 tests verts.

**Écoute (humain)** : « pas si pire, ça ressemble un peu à ce qu'on veut ».
Direction validée, identifiabilité pas encore atteinte → le critère 6 n'est
pas rempli ; P1-T6 reste en attente.

## P1-T7 — Corrections d'écoute : pieds, registre, débuts identiques

Retour d'écoute (10 reels, seeds 1–10) : talon assommant (GM 36 à 90 sur
chaque noire), 6 seeds sur 10 ouvrant sur `5 5 3 5 6 5 3 5`, mélodie centrée
sur ré4 contre si4 dans le corpus (`harmonium_lab trad-compare`, 555 reels).

### Décisions

- **Pieds en données (V)** : `reel_feet.toml` porte `heel_note` (35),
  `toe_note` (37), `heel_velocity` (55), `toe_velocity` (45) et `feet_gain`
  global (1.0, `0..=1`) ; constantes Rust supprimées. Le propriétaire règle
  à l'oreille en TOML.
- **Registre** : `melody_center_midi` (71 = si4), `strain_b_lift_semitones`
  (+2) ; le contour vise ces cibles en hauteurs absolues (transposition
  d'octave par cellule, essais les plus proches d'abord, match strict dans
  ±4 demi-tons), avec garde-fou de registre lâche (±6, anti-erreurs
  d'octave). Toniques G4/D5/A4 (66–77), tessiture G3–E6 (55–88).
  `TUNING_VERSION` 2 → 3 (rupture snapshot assumée, chemin Legacy inchangé).
- **Débuts** : cause — la grille diatonique A (≤ 4,5 pas) disqualifiait les 2
  `open` aiguës et le plancher de tessiture disqualifiait la grave en
  sol/la, ne laissant que 2–3 cellules éligibles à la mesure 1 ; le premier
  tirage strict gagnait. Correctif : contour absolu (toutes les cellules
  éligibles par transposition) + 12 `open` (17 au total, attaques sur tous
  les degrés) + 2 cadences de pont par le grave (`5, 1 2 3 4 3 2 5`,
  `4, 1 2 3 4 3 2 1`) + règle « deux tunes consécutifs, deux ouvertures ».
- **Tests** (`trad_p1t7_listening_tests`, flux snapshots 20×32) : ≥ 16
  mesures 1 distinctes ; moyenne 68–75 par seed ; B−A moyen 0–4 ; pieds
  ≤ 60 par défaut. Critères 1–4 du plan et continuité ≥ 95 % toujours verts ;
  critère 5 réécrit en contraste moyen (l'ancien « B > A par seed »
  figeait l'écart +6 remplacé).
Reproduire les démos :

```bash
make soundfont
cargo run -p harmonium-cli --release -- --export --bars 64 --style reel --seed 1 --feet 1 \
  --record-wav demos/reel_seed1.wav --record-midi demos/reel_seed1.mid
```

Dette connue, à traiter au passage :
- `assets/test.sf2` ne contient ni violon, ni guitare acier, ni basse
  acoustique, ni kit : inutilisable pour juger le reel (les tests s'en
  servent seulement pour le flux).
- Warning clippy `large_enum_variant` sur l'enum `Generator` du composer :
  disparaît avec P1-T6.
- `harmonium_host/src/soundfont.rs` : tests qui figent la taille de
  `test.sf2` et dépendent de la présence de GeneralUser ; à réécrire.
- Logs du générateur Legacy imprimés en plein REPL (`→ Chord: …`).

## Suite proposée (prochaine session)

1. **Écoute ciblée, 30 min** : noter par élément ce qui sonne « pas trad »
   (cellules, pieds, swing, tempo, cadences, accompagnement). Correction en
   données d'abord.
2. **Phase 2 — l'idiome** (ce qui manque le plus à l'oreille) :
   ornements rendus en temps-échantillon (cut, roll, triolet d'archet,
   double-corde), variation d'ornementation aux reprises, accordéon en
   doublure, humanisation des pieds, puis gigue 6/8 et valse.
3. **P1-T6** dès que l'écoute dit « c'est un reel ».
4. **Phase 3 — le jeu** : plugin Bevy sur `create_offline_engine`, couche
   `GameMood` (sacre, faim, intensité) → paramètres trad.

Orchestration : agent `omp` dans des worktrees Orca, un worktree par tâche,
merge `--no-ff` par le coordinateur après tests + clippy + fmt + écoute.
Routage des modèles : Antigravity (gemini-4-argon difficile,
claude-sonnet-5-5 moyen, gemini-3.8-flash mécanique), repli Muse, puis
DeepInfra (DeepSeek-V4-Pro / V4-Flash) quand un quota tombe.

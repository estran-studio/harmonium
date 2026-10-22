# Phase 1 — Un reel identifiable

**Statut** : point d’arrêt — P1-T1..T5 mergées, P1-T6 en attente du verdict | **Date** : 2026-10-19 | **Dépend de** : phase 0 (horloge, swing, rampe)

## But

Générer en continu un **reel** québécois que des oreilles québécoises
reconnaissent comme tel, à partir d'un seed. Ensemble minimal : violon,
guitare (chop), basse, pieds (talon, pointe). Synthèse SoundFont GM
provisoire. Pilotage par les quatre émotions du jeu : P1-T12 ci-dessous.

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

Pieds validés à l'oreille (2026-10-20, « ça fait l'affaire pour le moment ») :
`harmonium_trad.sf2`, construit par `harmonium_lab` (`make trad-sf2`) à partir
de 8 frappes CC0 Freesound (talon touche 36, pointe 37, banque 0 programme 0),
pieds 9 à 12 dB sous le violon, vélocités humanisées ±12. Elle sera remplacée
par les enregistrements du propriétaire (`samples/feet/own/`). Branchement
moteur : tâche SK2.

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
### D12 — Cellules minées du corpus (P1-T9)

- Banque `reel_cells.toml` : 180 cellules minées du corpus harmonium_lab (507 airs de reels québécois, extraction 2026-10-19), top 30 par rôle (`open`, `run`, `arp`, `turn`, `cad_half`, `cad_full`), `weight` = fréquence au corpus. Fichier unique de vérité : le `.weights.json` compagnon n'est pas versionné dans le moteur.
- Les 13 cellules inventées du plan D5 et les 11 `open` de P1-T7 sont retirées ; les cellules d'origine présentes au corpus restent.
- Degrés au-delà de 8 (`10`, `12`, `13`) et graves (`5,`, `7,`) : la validation `CellNote` (degrés 1..=14, −7..=−1, ambitus ≤ octave) les accepte déjà — cohérent avec l'ambitus du violon (tessitura G3–E6, `melody_center_midi` 71).
- Tirage pondéré par rôle (même RNG, un seul tirage) ; `Cell::weight` > 0.
- Cadences D4 : les cellules minées `cad_half`/`cad_full` ne finissent pas toutes comme le plan l'impose ; la sélection filtre via `Cell::respects_cadence` (m.4 sur 2/5/7 ; m.8 sur tonique, ou tonique tenue + croche de reprise). Corrigé dans la sélection, pas en réinventant des cellules.
- Point d'entrée de validation Rust : `harmonium-cli --check-cells <fichier.toml>` (charge via `CellBank::from_toml`, sort 0/1 avec erreurs lisibles). `harmonium_lab` remplace son script Python par cet appel.
- Contrat : banque embarquée 180 cellules (30 par rôle), poids > 0 ; tirage pondéré sur 10 000 tirages dans ±30 % du rapport des poids ; P1-T7 : ≥ 16/20 premières mesures distinctes (seeds 1..20 : 16), moyenne mélodique dans 68..75, B−A dans 0..4, deux tunes consécutifs ne commencent pas par la même cellule.

## P1-T8 — Tours d'un air et entrées progressives

Implémenté sur `main` après P1-T7 (branche `wquintal/phase1-t8-arrangement`).

### Décisions

- **Position structurée** : `TunePosition { tune_index, tour, tours, part, pass,
  bar_in_strain }` (`trad::arrangement`), calculée par `TunePosition::at_bar` —
  unique conversion mesure → position. `BARS_PER_TUNE` est supprimé au profit de
  `BARS_PER_TOUR` (32) × `tours_per_tune`.
- **Tours** : un air dure `tours_per_tune` tours AABB (défaut 3) ; chaque tour
  rejoue la même `FiddleLine` et la même `TuneHarmony` (indexation par
  `bar_in_tour`), les tours 2..N ne consomment aucun tirage RNG.
- **Entrées en données** : `harmonium_core/src/trad/reel_arrangement.toml`,
  chargé dans `TuningParams::reel_arrangement` et validé (`check_range` pour les
  tours, part `A`/`B`, passe 1/2, piste connue) — erreurs explicites nommant la
  piste fautive. Défauts : Melody tour 1 / A1, Chords et Bass tour 1 / A2,
  Heel et Toe tour 2 / A1 ; `intro_each_tune = false` (dès le 2ᵉ air, tous en
  A1). Une entrée de tour dépassant le dernier tour est ramenée à celui-ci.
- **Un seul calcul « piste active »** : `TradGenerator::track_active` =
  arrangement (`allows`) ET drapeau `enable_*`/`feet_density` (plafond du jeu).
- **Contrat** : `Measure::position` et `MeasureSnapshot::position`
  (`Option<TunePosition>`, `serde(default)`, `None` hors trad). Rupture de
  snapshot : `TUNING_VERSION` 3 → 4, golden Legacy régénéré dans le même commit
  (seul ajout `"position": null`).
- **CLI** : `--tours N` (`MusicComposer::set_tours_per_tune`, validé) ;
  l'export imprime les bornes des airs selon la table.

### Mesure sur `demos/reel_arrangement_seed1.wav` (96 mesures, seed 1, GeneralUser GS)

Première note de chaque piste (MIDI) : Melody mesure 1 pas 0 ; Bass mesure 9
pas 0 ; Chords mesure 9 pas 4 ; Heel et Toe mesure 33 pas 0.

RMS par tranche de 8 mesures (dBFS, mix stéréo moyenné) : −42.1 (1–8),
−32.4 (9–16), −32.9 (17–24), −32.9 (25–32), −32.2 (33–40), puis −32.2 à −32.7
jusqu'à 96. L'entrée de l'accompagnement monte de ~9,7 dB ; l'entrée des pieds
de ~0,7 dB (table P1-T7 volontairement discrète, crête inchangée).

### Tests

`harmonium_core/tests/trad_arrangement_tests.rs` (8 tests, flux de snapshots,
5 seeds) : violon seul 1–8, accompagnement 9–32, pieds 33–96, tout le monde dès
97 ; tours identiques ; tours 2..N sans tirage RNG (seek `silent_advance(97)`) ;
tables invalides ; plafonds `enable_*` et `feet_density` ; position exposée.
Les tests de forme/accompagnement (P1-T1..T7) sont isolés de D11 par un
arrangement plat `ReelArrangement::from_start(1)`.

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
| P1-T8 Tours d'un air et entrées progressives (D11) | mergée à valider |

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

## Objectif de la première démo du jeu (priorité, 2026-10-21)

Un reel sur lequel le personnage danse la gigue, et qui évolue selon les
ennemis, la vie et la barre damnation/sacré. Ces concepts restent dans le
jeu : le jeu les traduit en 4 émotions (`arousal`, `valence`, `density`,
`tension`) et le moteur ne connaît que ces 4. Exigences moteur (P1-T12) :
chaque émotion a un effet audible et distinct en trad, un changement
s'entend à la mesure suivante, la pulse `BeatInfo` ne saute jamais (tempo
par rampe), automation d'export pour écouter un scénario de combat.

Validé à l'oreille (2026-10-22, `demos/combat_seed1.wav`, après correction
du mixage `2e4089a`) : « on sent une progression plus rapide et intense
pendant que la musique joue ». Mixage retenu : le violon mène à toutes les
intensités ; guitare 5 à 12 dB sous le violon, basse 1 à 4 dB sous.
P1-T13 permet désormais le passage au mode parallèle pendant l'air, à la
frontière de phrase suivante (voir ci-dessous).

## Suite proposée (prochaine session)

1. **Écoute ciblée, 30 min** : noter par élément ce qui sonne « pas trad »
   (cellules, pieds, swing, tempo, cadences, accompagnement). Correction en
   données d'abord.
2. **Phase 2 — moteur multi-styles par profils** (constitution v3, principe V) :
   remplacer `TuneKind` et les constantes du reel (2/2, 16 pas, parties de 8,
   AABB) par un `StyleProfile` en TOML : grille et signature, grammaire de
   forme (AABB, AABBCC, couplet-refrain, appel-réponse, strophique), banques
   de cellules, modèle d'harmonie (fonctionnelle, modale, bourdon, aucune),
   instruments, percussions, arrangement. Le reel devient un profil parmi
   d'autres, sans changement audible (goldens identiques). Preuve : gigue
   6/8 et valse 3/4 québécoises ajoutées **sans toucher au code**, à partir
   du corpus du labo (88 gigues, 67 valses).
3. **Phase 3 — l'idiome du reel** : ornements en temps-échantillon (cut,
   roll, triolet d'archet, double-corde), variation aux reprises, accordéon,
   mode mineur et 3 parties (*Reel Béatrice*).
4. **Phase 4 — autres traditions**, une par profil + corpus au labo :
   acadienne (proche, même instrumentarium), britannique coloniale (fifre et
   tambour, country dances, hymnes), autochtones (principe VII : consultation
   avant entrée dans le jeu, pas de chants sacrés).
5. **P1-T6** dès que l'écoute dit « c'est un reel » ; Legacy n'est pas un
   profil de style, il est supprimé.
6. **Phase 5 — le jeu** : plugin Bevy sur `create_offline_engine`, couche
   `GameMood` (sacre, faim, intensité) → paramètres du profil actif.

Orchestration : agent `omp` dans des worktrees Orca, un worktree par tâche,
merge `--no-ff` par le coordinateur après tests + clippy + fmt + écoute.
Routage des modèles : `~/.harmonium-orch/models.md`, `omp usage` vérifié avant
chaque lancement ; depuis 2026-10-20, OpenRouter (`deepseek-v4-pro` difficile,
`deepseek-v4.1-flash` moyen et mécanique) tant que les abonnements sont épuisés.

## P1-T12 — Intensité, quatre émotions et automation d'export

### Contrat et données

- `MusicalParams.intensity` est bornée à `0..1`, défaut `0.5` (rendu neutre).
  `TuningParams.reel_intensity` charge `trad/reel_intensity.toml` dans
  `StyleDynamics` ; la même section est remplaçable via `TuningOverlay`.
  Courbes par points, interpolation linéaire, endpoints maintenus ; validation
  des domaines, points ordonnés et finis, palette non vide et motifs de chop.
- Le mapper reçoit les données du générateur, jamais un nom de style : Legacy
  conserve son mapping 70–180 ; trad conserve une base nominale dans
  `MusicalParams.bpm`, puis applique le décalage d'intensité à `Measure.tempo`.
  Avec les données livrées : 230 − 10 à 230 + 15 BPM.
- Les concepts ennemis/vie/damnation/sacré restent dans le jeu. Il ne transmet
  que `set_emotions(arousal, valence, density, tension)` en mode émotion.

| Axe | Effet immédiat à la mesure suivante | Effet au prochain air |
|---|---|---|
| Arousal → intensity | Énergie/vélocités, tempo borné, coups de pieds, chop, notes plus courtes, accents forts | Poids run/arp/turn |
| Valence (−1..1) | Brillance des accents ; mode parallèle à la prochaine phrase (P1-T13) | Tonalité pondérée et centre de registre |
| Density (0..1) | Instruments actifs sous les plafonds du jeu et de l'arrangement, pieds, chop, notes de passage sur les tenues | Aucun changement du plan |
| Tension (0..1) | Accents de contretemps, articulation sèche, dominantes plus fréquentes et tenues | Aucun changement du plan |

- Les cellules de l'air déjà audible sont conservées ; seules les mesures futures et
  les plans d'airs non encore entendus sont invalidés. AABB conserve les degrés et
  positions du squelette mélodique sous automation d'intensité/valence ;
  vélocités/durées et ornements de densité restent des paramètres de rendu.
- Un changement de tempo d'une mesure trad utilise la rampe existante de
  `SetTempo`, sur le nombre de pulses d'une mesure (deux en reel). Un override
  `SetTempo` explicite garde priorité. Pas de seek, pulse en cours inchangée.
  Les métadonnées MIDI suivent le tempo audible de chaque pulse via un
  `AudioEvent::MeasureTiming` sans Box/clone de paramètres dans le callback.
- `TUNING_VERSION` reste 5 : le point neutre ne change pas les snapshots.

### Utilisation

REPL : `style reel`, `intensity 0.8`, ou `emotion 0.8 -0.5 0.9 0.7`.
Les mutations passent par les setters du composer + `invalidate_future`,
puis transmission de `take_pending_drop` et régénération des mesures futures,
comme pour `create_offline_engine` côté jeu.

```bash
cargo run -p harmonium-cli -- --export --style reel --bars 100 --seed 1 \
  --automation demos/automation_monte.toml \
  --record-wav demos/intensity_seed1.wav --record-midi demos/intensity_seed1.mid \
  --record-truth demos/intensity_seed1.json

cargo run -p harmonium-cli -- --export --style reel --bars 100 --seed 1 \
  --automation demos/automation_combat.toml \
  --record-wav demos/combat_seed1.wav --record-midi demos/combat_seed1.mid
```

Automation : `[[keys]]`, `bar` positif strictement croissant ; champs optionnels
`intensity`, `arousal`, `valence`, `density`, `tension`, `bpm`, `feet`.
Chaque champ est interpolé indépendamment ; les premières/dernières valeurs
sont maintenues hors de leur intervalle. `feet` est interpolé puis arrondi à
0/1/2. `bpm` désigne la base avant le décalage d'intensité. Les commandes
directes (`intensity`, `bpm`, `feet`) gagnent sur le mapping émotionnel.
Un champ émotionnel absent conserve sa valeur initiale. Un export automatisé
peut traverser plusieurs airs ; il ne force pas l'outro du premier air.

Avec automation, `--record-truth` écrit un objet : `measures` (snapshot et
intensité réellement appliquée), `bar_start_frames` (horloge audio),
`sample_rate`, `rendered_frames`, et `reports` (rapports périodiques habituels).
Sans automation, le tableau de rapports historique reste inchangé.

### Mesures du rendu P1-T12 (seed 1, 100 mesures)

Fichiers locaux : `demos/intensity_seed1.{wav,mid,json}` et
`demos/combat_seed1.{wav,mid,json}` ; seuls les TOML des scénarios sont versionnés.
GeneralUser GS (32 319 396 octets), 44 100 Hz stéréo float32. La banque dédiée
`harmonium_trad.sf2` est absente de ce worktree : les pieds utilisent le kit GM,
pas les échantillons CC0 du propriétaire. Cette limite affecte le timbre et le
RMS, pas les événements générés.

| Mesures | Intensité moy. | Vél. violon | Vél. pieds | Durée violon (ms) | Frappes pieds/mesure | RMS (dBFS) |
|---|---:|---:|---:|---:|---:|---:|
| 1–8 | 0.145 | 65.85 | 34.00 | 177.5 | 0.25 | -44.87 |
| 9–16 | 0.248 | 71.25 | — | 189.0 | 0.00 | -37.13 |
| 17–24 | 0.352 | 76.26 | — | 159.6 | 0.00 | -35.76 |
| 25–32 | 0.455 | 80.73 | — | 152.4 | 0.00 | -34.88 |
| 33–40 | 0.542 | 84.66 | 48.54 | 166.1 | 7.00 | -34.16 |
| 41–48 | 0.617 | 87.85 | 52.81 | 163.3 | 8.00 | -33.87 |
| 49–56 | 0.692 | 90.00 | 55.45 | 145.0 | 8.00 | -33.44 |
| 57–64 | 0.767 | 92.18 | 51.43 | 93.6 | 11.00 | -32.68 |
| 65–72 | 0.828 | 94.72 | 54.34 | 92.9 | 12.00 | -32.19 |
| 73–80 | 0.878 | 97.10 | 56.66 | 95.3 | 12.00 | -31.78 |
| 81–88 | 0.928 | 98.19 | 57.99 | 77.6 | 12.00 | -31.51 |
| 89–96 | 0.978 | 99.40 | 60.57 | 72.5 | 12.00 | -31.42 |
| 97–100 | 1.000 | 99.64 | 59.42 | 73.0 | 9.00 | -32.77 |

Moyennes de vélocité/durée pondérées par note ; pieds = talon + pointe.
Durée écrite jusqu'au NoteOff (hors release SF2), calculée sur les pas gelés
des pulses avec rampe et swing. Les durées de mesures reconstruites concordent
exactement avec tous les `bar_start_frames` publiés. RMS calculé sur les
échantillons du WAV, énergie des deux canaux moyennée, aux frontières exactes
des mesures ; la dernière tranche inclut la fin du dernier buffer de rendu.
L'intro et les entrées D11 sont conservées : l'absence de pieds en 9–32 est
attendue, leur entrée principale est en 33–40. L'ajout de pointes faibles
explique qu'une vélocité moyenne de pieds puisse baisser malgré la montée.

### Preuves observées

- Latence : chacun des quatre axes, appelé au milieu de la mesure 1, modifie
  la mesure 2, soit **une mesure** ; la pulse en cours garde sa borne de fin,
  chaque pulse suivante commence exactement sur la précédente, sans saut de
  mesure. L'arousal 1 donne 245 BPM ; la rampe de la mesure 2 atteint cette
  cible sur sa deuxième pulse.
- Reprises : hauteurs/positions A1=A2 et B1=B2 vérifiées aussi sur le vrai
  export automatisé, y compris le décalage de l'intro.
- Deux exports indépendants de 100 mesures donnent WAV, MIDI et JSON
  **identiques bit à bit**. SHA-256 :
  WAV `f7279da47779a6d20b08d140ddeba4f2658dc78c9a700fd8e58ea7460ef0af7b`,
  MIDI `76fbfa50d1272dddfe558351fe05d905ed645bc51059d088745ed6b280e74388`,
  JSON `003764959aea753c5e14fc45c8d51f35c2f026f5cf447135e04d602e87bfc196`.
- Snapshots neutres : probe temporaire supprimée, **768 snapshots**
  (seeds 1, 2, 3 × 256 mesures) comparés à une archive de `main` :
  mêmes octets, SHA-256
  `71e88162199eb2ddc41cfc513586fbfe8478cda2d0ca9b1cfd1d0cb2c5cac0de`.
- Export invalide réel : `bar=1 intensity=1.5` sort 1 avec le chemin, la mesure,
  le champ et le domaine `0..1`. REPL réel : `intensity 0.9` accepté,
  `intensity 1.2` refusé ; `emotion 0.8 -0.5 0.9 0.7` transmet bien les quatre
  axes. Ce smoke a révélé/corrigé le routage préexistant des valences absolues
  négatives vers la syntaxe relative ; test régressif conservé.
- Clippy production : `cargo clippy --workspace --lib --bins --message-format=json`
  exécuté sur la branche et l'archive `main`, même `target/debug` :
  **187 diagnostics de chaque côté, zéro nouveau**, comparaison code/message/
  chemin sans les numéros de ligne. `-D warnings` global reste bloqué par la
  dette préexistante (notamment `unwrap_used`/`expect_used` des exports Legacy) ;
  elle n'est ni masquée ni étendue dans cette tâche.

- Vérification finale : `cargo test --workspace` **507 réussis, 33 ignorés** ;
  `cargo fmt --all -- --check` passe. Baseline `main` de la comparaison :
  `b1e4184066d90cdc5f8f2de33236229849267de6`. Un seul profil `target/debug`,
  archive de comparaison et probes supprimées après mesure.


## P1-T13 — Mode parallèle pendant l'air

- `reel_intensity.toml` : `mode_shift_boundary = "phrase"` (4 mesures) ou
  `"part"` (8), `dark_below = -0.3`, `bright_above = 0.1`. Seuils stricts ;
  entre les seuils, la dernière demande reste mémorisée jusqu'à la frontière.
- Table pondérée `parallel_modes` : ionien → éolien (3) / dorien (1),
  mixolydien → dorien (1). Tirage par le RNG injecté au premier passage sombre,
  mémorisé pour l'air ; retour au mode clair d'origine, sur la même tonique.
  Un air né mineur utilise le mode clair de la première ligne correspondante.
- Les mêmes cellules sont relues en conservant degré, octave et rythme de chaque
  note déjà choisie (repli d'octave uniquement hors du domaine physique MIDI
  0..127). L'harmonie est revotée seulement au changement de mode ; basse et
  chop utilisent ce mode, y compris les dominantes sous tension.
- `invalidate_future` conserve les décisions audibles et retire les décisions
  spéculatives futures ; aucune mutation de la mesure en cours ni de `BeatInfo`.
- `MeasureSnapshot.position` publie toujours `key` et `mode` en trad, intro et
  outro comprises. Décision du coordinateur : publication inconditionnelle,
  rupture de snapshot assumée, `TUNING_VERSION` **6 → 7**. L'invariant neutre
  porte sur toutes les notes (piste, hauteur, début, durée, vélocité), pas sur
  les nouveaux octets des métadonnées. Aucun tirage ajouté au chemin neutre.
- Régression reproduite avant correction : le degré 3 de la mesure 5 restait
  majeur après valence −0.9 à la mesure 3. Les sept tests de contrat
  `trad_mode_shift_tests` passent après correction (frontières, degrés/octaves,
  hystérésis, accompagnement/ornements, limite MIDI, invalidation et déterminisme).

### Rendu combat observé (100 mesures, seed 1)

Commande : `cargo run -p harmonium-cli -- --export --style reel --bars 100
--seed 1 --automation demos/automation_combat.toml --record-wav
demos/combat_t13_seed1.wav --record-midi demos/combat_t13_seed1.mid
--record-truth demos/combat_t13_seed1.json`.

GeneralUser GS installé via `make soundfont` (32 319 396 octets), 44 100 Hz,
102,50 secondes. Banque dédiée des pieds absente : kit GM, sans effet sur
le mode ni l'horloge. Le scénario de combat existant n'a pas besoin de changer.
L'intro prend deux mesures : les phrases du premier air commencent en 3, 7, etc.

| Frontière (mesure) | Tonique | Mode actif | Valence appliquée |
|---:|---|---|---:|
| 3 | la | ionien | 0.4000 |
| 7 | la | ionien | 0.4000 |
| 11 | la | ionien | 0.4000 |
| 15 | la | ionien | 0.4000 |
| 19 | la | ionien | 0.2875 |
| 23 | la | ionien | 0.1375 |
| 27 | la | ionien | -0.0125 |
| 31 | la | ionien | -0.1625 |
| 35 | la | ionien | -0.2875 |
| 39 | la | éolien | -0.4042 |
| 43 | la | éolien | -0.5208 |
| 47 | la | éolien | -0.6375 |
| 51 | la | éolien | -0.7542 |
| 55 | la | éolien | -0.8708 |
| 59 | la | éolien | -0.9000 |
| 63 | la | éolien | -0.9000 |
| 67 | la | éolien | -0.5531 |
| 71 | la | éolien | -0.0906 |
| 75 | la | ionien | 0.3719 |
| 79 | la | ionien | 0.8344 |
| 83 | la | ionien | 0.9500 |
| 87 | la | ionien | 0.9500 |
| 91 | la | ionien | 0.9500 |
| 95 | la | ionien | 0.9500 |
| 100 | sol | ionien | 0.9500 |

- Seuil sombre franchi à la mesure **36** (−0.3167), entendu à **39** :
  **3 mesures**. Seuil clair franchi à **73** (+0.1406), entendu à **75** :
  **2 mesures**. Valences interpolées par le TOML, échantillonnées à la mesure
  comme le fait l'export. La tonique ne change qu'au nouvel air (mesure 100).
- Deux rendus indépendants sont identiques bit à bit (SHA-256) :
  WAV `bfbe1a14d038bee52fbba24dd27e8d4fa6ca4421e08492b4dab1c5693fc3c0a5`,
  MIDI `7c8e35dd851a79134f8d2dbcda916b6ee9767b9d8b2a30e3e13e5147bb2a292a`,
  JSON `9247f78d8a9802090eb2fef9e17a602d2189255a2f30c35f9d7bdcaa5577a772`.
- Neutralité contre l'archive de `main` **aacc9aa** : **768 snapshots**
  (seeds 1, 2, 3 × 256 mesures), toutes les notes identiques (piste, hauteur,
  début, durée, vélocité). SHA-256 du JSON canonique des notes :
  `f720deeffc3bb0d97ad747f54e423fe440c7fcd681b54f10a2e1f1b42a581991`.
  Probes temporaires supprimées. Golden Legacy régénéré : octets inchangés
  (`position = null`, donc aucune métadonnée trad à enrichir).
- Contrat playback : appels pendant les mesures 3 et 7, passages à 5 et 9,
  soit **2 mesures** chacun ; chaque pulse commence exactement à la fin de
  la précédente, sans modifier la pulse en cours ni la mesure audible.

### Vérification

- `cargo test --workspace` : **515 réussis, 33 ignorés** (dont sept contrats
  cœur P1-T13 et un contrat playback/horloge).
- `cargo fmt --all`, puis `cargo fmt --all -- --check` : succès.
- Clippy production : `cargo clippy --workspace --lib --bins
  --message-format=json`, exécuté sur la branche et l'archive de `main`,
  **187 diagnostics dans les deux cas, zéro nouveau** (comparaison par
  chemin/code/message, sans numéros de ligne). La dette préexistante empêche
  toujours `-D warnings` global ; aucun lint n'a été masqué.
- Les **1 691 notes** de mélodie/accords/basse du vrai rendu combat
  appartiennent toutes au mode publié à leur mesure.


# Phase 1 — Un reel identifiable

**Statut** : en cours | **Date** : 2026-10-19 | **Dépend de** : phase 0 (horloge, swing, rampe)

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
(canal 9). Remplacés en phase 2 par une SF2 dédiée avec de vrais pieds.

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

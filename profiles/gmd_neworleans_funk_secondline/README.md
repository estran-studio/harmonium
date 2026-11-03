# Profil `gmd_neworleans_funk_secondline` (phase 019, B7)

Accompagnement New Orleans, second line et funk de la Nouvelle-Orléans, sous une grille d'accords :
basse (tuba), comping (piano) et batterie ne dépendent que de l'accord publié (profil `loop`,
`[fixed_tonality]`, pas de piste `Melody`). Fait pour training : l'utilisateur fournit sa grille, le
profil joue dessous ; sans grille, cinq boucles de douze mesures tirées par la graine.

Le nom est celui des sources qui définissent le style (principe VII) : les styles `neworleans/funk` et
`neworleans/secondline` du Groove MIDI Dataset. Il ne dit rien d'une tradition entière : un kit
électronique, quatre batteurs, trois fichiers de second line.

## Provenance et droits

| Couche | Source | Licence | Mention (aussi dans `[meta] corpus`) |
|---|---|---|---|
| Batterie : groupes de mesures, patterns, phrases, relances, micro-timing, feel, tempos | Groove MIDI Dataset v1.0.0 : `neworleans/funk` (8 fichiers `beat`, 4 batteurs, 396 mesures, « assez » ; 40 fichiers `fill`, 2 batteurs, 59 mesures, « juste ») et `neworleans/secondline` (3 fichiers `beat`, 2 batteurs, 212 mesures, « juste »), soit 51 des 53 fichiers `neworleans` ; `neworleans/chacha` et `neworleans/shuffle` (un fichier chacun, « trop peu ») ne sont pas lus. SHA256 de l'archive vérifié avant toute lecture : `651cbc52…1dcc1e` | CC BY 4.0 | Gillick, Roberts, Engel, Eck, Bamman, « Learning to Groove with Inverse Sequence Transformations », ICML 2019 ; Google LLC |
| Basse (tuba), comping (piano), grilles par défaut, arc, mixage | Règles de métier écrites pour ce profil | — | Aucune source ouverte : WJazzD n'a pas de sous-ensemble New Orleans, le GMD n'a que la batterie |

Aucune mesure d'un enregistrement n'est copiée : seuls des agrégats (probabilités par pas, comptes,
parts, centiles) servent. Les fichiers MIDI du GMD ne sont pas dans le dépôt. Mesures du labo :
`harmonium_lab/references/gmd-neworleans/`, `gmd-neworleans-funk/`, `gmd-neworleans-secondline/`,
`references/l2-ingestion.md` ; la batterie est régénérée par
`specs/019-bibliotheque-accompagnements/b47_batterie.py gmd_neworleans_funk_secondline` (labo en lecture
seule).

## Mesure → valeur

| Paramètre | Valeur | Mesure source |
|---|---|---|
| `meter.steps_per_bar`, `default_swing_ratio` | 16 pas, 0,5 | GMD neworleans : 11 fichiers `beat` sur 11 « straight » ; rapport de swing médian 0,50 (funk), 0,51 (second line, p90 0,53) |
| `default_bpm`, `min_bpm`, `max_bpm` | 93, 82, 124 | 11 fichiers `beat` : médiane 93, minimum 82, maximum 124 (funk 82 à 102, médiane 90 ; second line 94 à 124, médiane 99) |
| Longueur du chorus | 12 mesures | blues de la Nouvelle-Orléans (forme commune) ; aucune mesure |
| Batterie : groupes de mesures | 8 groupes (k-moyennes) dont 4 retenus : 20 mesures, 3 fichiers, 2 batteurs au moins, aucun fichier au-delà de 60 % ; une grosse caisse d'au moins 2 coups et un roulement de caisse claire (≥ 8 coups) ou un timekeeper (≥ 4 coups) | GMD neworleans `beat` (608 mesures) |
| Batterie : seuils par densité | densité 0 : timekeeper ≥ 0,5, grosse caisse et caisse claire ≥ 0,75 ; densité 1 : tout ≥ 0,5 ; densité 2 : grosse caisse, caisse claire, cross-stick ≥ 0,3, le reste ≥ 0,5 | probabilités par pas du groupe |
| Batterie : phrases | 9 suites de 2 et 4 mesures (≥ 3 occurrences sur ≥ 2 batteurs), 3 densités, `part_end` pour 3 d'entre elles | GMD neworleans `beat`, fenêtres alignées sur la première mesure |
| Batterie : relances | 4 groupes de fichiers `fill` (1 mesure du pas 0 : 5 et 16 fichiers ; 2 mesures du pas 15 : 4 ; 2 mesures du pas 0 : 11), densités 1 et 2 | GMD `neworleans/funk` `fill` : 40 fichiers, 2 batteurs |
| `fills_none_weight` | 29 | valeur de service (44 % de fins de phrase sans relance) |
| `performance.tracks.Percussion*` | sur le temps −0,08 à −0,09 pas, contretemps +0,03 à +0,075, sans `jitter` | GMD neworleans/funk, médianes de l'écart à la grille (grosse caisse −0,08 sur le temps, +0,075 hors temps ; caisse claire −0,08 et +0,05 ; charleston −0,08 et ≈ 0 ; pédale −0,12 et ≈ 0) |
| Niveau de chaque voix de batterie | gain par rôle, voir le script | LUFS en solo mesuré sur le kit 128/0 de GeneralUser GS (même mesure que `gmd_funk`) |
| `mix.tracks` | accords −19 dB, basse −11 dB, percussion −19 dB, maître +24,5 dB | LUFS intégrés en solo, graine 1, 36 mesures : accords −26,4, basse −24,2, batterie haute −25,0, grosse caisse −30,3, mix −20,3 |

Valeurs de métier, aucune mesure : les cinq grilles, les riffs de tuba, les rythmes de comping et leurs
phrases, l'approche de la basse (`none_weight` 60, chromatique 28, degré de gamme 12), l'arc, le mixage.

## Ce que le profil joue

- **Grille** : cinq boucles de douze mesures en si bémol mixolydien (`fixed_tonality` si bémol,
  `mixolydian`), relatives à la tonique : blues de la Nouvelle-Orléans `I7 IV7 I7 I7 | IV7 IV7 I7 I7 | V7
  IV7 I7 | I7 V7` (poids 4) ; blues à changement rapide (3) ; vamp `I7` huit mesures puis `IV7` quatre (3) ;
  vamp `I9` douze mesures (2) ; forme de fanfare `I6`, `V7`, `I6`, `IV` `IVm`, `I6`, `V7`, `I6` (2). La
  graine en tire une par air ; training la remplace par la sienne.
- **Basse** (tuba, GeneralUser GS 58) : sept riffs, `reference = "chord_root"` : deux temps (fondamentale
  sur le un, quinte sur le trois), deux temps syncopé, syncope sur les pas 0, 3, 6, 8, 11, 14 (les
  positions de la grosse caisse du second line du GMD : kick le plus fréquent `[0, 3, 6, 8, 11, 14]` dans
  `neworleans/secondline`), marche de quatre noires, croches, deux mesures, deux blanches. Les riffs
  suivent l'accord publié et arrivent sur sa basse à chaque changement ; fondamentale, quinte, septième
  mineure et octave, sans tierce.
- **Comping** (piano 0, accords à quatre sons : `drop2`) : seize rythmes d'une mesure en trois étiquettes
  (`sparse` : tenues ; `medium` : le 3-3-2 des pas 0, 3, 6, 8, 11, 14 et les contretemps ; `dense` : croches
  et doubles croches) et vingt-cinq phrases de 2 et 4 mesures.
- **Batterie** (kit standard 128/0) : 12 patterns (4 groupes × 3 densités) dont des roulements de caisse
  claire de seize doubles croches (le groupe le mieux fourni : 180 mesures, 9 fichiers, 4 batteurs), 36
  phrases (9 suites × 3 densités, dont 3 aussi en `part_end`), 8 relances.
- **Arc** : celui de `gmd_funk` (cinq chorus : exposition, deux de corps, sommet, retombée), les breaks du
  corps tombant à la mesure 7 et à la dernière.

## Contrat et preuves

`harmonium_core/tests/gmd_funk_neworleans_profile_tests.rs` : les mêmes contrats que `gmd_funk`, joués
sur les deux profils (grille, basse sur les changements, comping dans l'accord publié, kit, feel droit,
tempo 82..124, arc, variété sur 16 graines). Variété et distance au corpus :
`specs/019-bibliotheque-accompagnements/b47-funk-neworleans.md`.

## Limites

- **Trois fichiers de second line.** Le style qui donne son nom est « juste » (3 fichiers, 2 batteurs) ; la
  batterie vient pour l'essentiel de `neworleans/funk` (8 fichiers). Les roulements de caisse claire de
  seize doubles croches sont des composites de probabilités, pas des rudiments : le schéma n'a pas de coup
  probabiliste (L2 § 6 point 5), pas de roulé.
- **Pas de timekeeper dans 41 % des mesures** de `neworleans/funk` (GMD) : une partie des patterns n'a ni
  charleston ni ride ; ils tiennent par la caisse claire et la pédale.
- **Basse et comping : aucune source.** WJazzD n'a aucun sous-ensemble New Orleans ; le tuba et le piano
  sont des règles de métier, jugées par des preuves chiffrées de contrat et de variété, pas à l'oreille (D5).
  Le plan de motifs de basse est fixé par air (la basse ne change pas d'un chorus à l'autre ; A4 l'étendra).
  Les riffs ignorent la qualité de l'accord (pas de tierce). L'approche suit la même règle que `gmd_funk` :
  tout changement de hauteur du riff, pas seulement le changement d'accord.
- **Pas de ternaire.** Le second line se joue souvent en feel ouvert (shuffle) ; le GMD le range en
  « straight » (11 fichiers sur 11, `neworleans/shuffle` : un fichier) : le profil est droit, 0,5, tous
  tempos.
- **Micro-timing** : décalages médians seulement (charleston et caisse claire partagent `PercussionHigh`), sans `jitter` (même raison que `gmd_funk`).
- **Phrases du GMD** : fenêtres de 2 et 4 mesures alignées sur la première mesure ; pas de phrases de 8
  mesures. Relances : 4 groupes, 2 batteurs, `fills_none_weight` est un réglage.
- **Habillage par graine** (A2, vague 4) : n'existe pas encore ; tempo, tonalité, instruments, kit sont fixes
  par profil. Le tuba ne se remplace pas par un sousaphone ou une basse électrique, le piano par un
  orgue ou des cuivres.
- **Mixage** : réglé au LUFS en solo (graine 1), aucune écoute.

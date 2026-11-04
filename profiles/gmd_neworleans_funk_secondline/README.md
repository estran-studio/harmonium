# Profil `gmd_neworleans_funk_secondline` (phase 019, B7)

Accompagnement New Orleans, second line et funk de la Nouvelle-Orléans, sous une grille d'accords :
basse (tuba), comping (piano) et batterie ne dépendent que de l'accord publié (profil `loop`,
pas de `[fixed_tonality]` (tonique tirée par `session.toml`), pas de piste `Melody`). Fait pour training : l'utilisateur fournit sa grille, le
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
| `performance.tracks.Percussion*.voices` | une entrée par note GM jouée, `on_beat` et `off_beat` en fraction de pas, sans `jitter` | GMD, médianes en ms de l'écart à la grille par rôle (groupe 4/4 droit, pondérées par l'effectif), converties au tempo par défaut ; table dans « Habillage par graine » |
| Niveau de chaque voix de batterie | gain par rôle, voir le script | LUFS en solo mesuré sur le kit 128/0 de GeneralUser GS (même mesure que `gmd_funk`) |
| `mix.tracks` | accords −19 dB, basse −11 dB, percussion −19 dB, maître +24,5 dB | LUFS intégrés en solo, graine 1, 36 mesures : accords −26,4, basse −24,2, batterie haute −25,0, grosse caisse −30,3, mix −20,3 |

Valeurs de métier, aucune mesure : les cinq grilles, les riffs de tuba, les rythmes de comping et leurs
phrases, l'approche de la basse (`none_weight` 60, chromatique 28, degré de gamme 12), l'arc, le mixage.

## Ce que le profil joue

- **Grille** : cinq boucles de douze mesures en si bémol mixolydien (si♭ mixolydien par défaut ; la session tire la tonique, voir « Habillage par graine »), relatives à la tonique : blues de la Nouvelle-Orléans `I7 IV7 I7 I7 | IV7 IV7 I7 I7 | V7
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

## Habillage par graine (`session.toml`, A2 ; R2-styles)

Une graine tire **un habillage** : un tirage indépendant par groupe, tous les groupes se combinent (9 groupes,
31 variantes, **62208 habillages possibles**). Le tirage est stable par graine, rejoué après un seek, et **l'hôte gagne** : `SetBpm` impose le tempo, `SetSwing`
le swing, `[fixed_tonality]` du profil de l'hôte (la grille de training, transposée par lui) la tonalité (la variante est
alors ignorée) ; programmes, voicings, registre, énergie et arc n'ont pas de commande (un hôte qui n'en veut pas installe
un profil sans `[session]`). Ce profil n'a donc **plus de `[fixed_tonality]`** : la tonalité par défaut (si♭ mixolydien) n'est
que la plus pondérée.

| Groupe | Variantes (poids) | Ce que règle le groupe | D'où vient la valeur |
|---|---|---|---|
| `tempo` | lent (2), medium (3), second_line (2), vif (1) | tempo (plage de BPM, un entier tiré) | GMD : centiles des BPM des fichiers `beat` |
| `tonalite` | si_bemol_mixolydian (3), fa_mixolydian (2), mi_bemol_mixolydian (2), do_mixolydian (1) | tonique et mode du premier air (grille relative transposée) | métier : tonalités courantes du style |
| `comping` | piano (3), honky_tonk (2), accordeon (1), orgue (1) | programme des accords | métier |
| `basse` | tuba (4), contrebasse (2), doigts (1) | programme de la basse | métier |
| `kit` | standard (3), room (1), jazz (2) | kit de la banque 128 | métier ; le GMD est joué sur un kit électronique Roland TD-11 |
| `voicing` | drop_2 (3), ouvert (1), shell (1) | famille de voicings du comping | métier |
| `registre` | normal (4), haut (1), grave (1) | octave du comping | métier |
| `energie` | doux (1), normal (2), fort (1) | multiplicateurs d'intensité et de densité de base | service |
| `arc` | defaut (3), orchestre (2), direct (1), montee (1) | arc : `[arc]` du profil ou arc nommé de `session.toml` | service (aucun corpus ne donne l'arc d'un morceau) |

Les arcs nommés (`orchestre` : le comping et le kit changent de chorus en chorus et reviennent aux programmes de la session ;
`direct` : pas d'exposition ; `montee` : un cran par chorus) sont écrits dans `session.toml`, mêmes valeurs de service que
`arc.toml`. Tous les instruments sont des programmes General MIDI de GeneralUser GS (banque 0) et des kits de la banque 128
(0, 8, 16, 24, 25), vérifiés dans la banque ; aucune zone Odin (D1).

**Micro-timing par voix** (`performance.tracks.Percussion*.voices`, A6) : médianes de l'écart à la grille mesurées dans le
GMD (groupe 4/4 droit de `neworleans/funk`, `neworleans/secondline`, pondérées par l'effectif), en **fraction de pas au tempo par
défaut (93 BPM)** : un pas = 15 000 / BPM ms ; les décalages mesurés en ms ne suivent donc pas exactement le tempo tiré
(à 130 BPM le même retard en ms vaut 24 % de pas de plus). `jitter` : amplitude d'un tirage uniforme dont l'écart-type est celui
mesuré, plafonnée à 0,1 pas (borne du schéma ; l'écart-type mesuré est de 0,11 à 0,20 pas). Aucun `jitter` : la basse à notes de passage partage son tirage avec la percussion (contrat `passing_keeps_neighbour_tracks…`).

| Voix | Note GM | Sur le temps (pas) | Hors temps (pas) | Jitter (pas) |
|---|---:|---:|---:|---:|
| kick | 36 | -0.082 | +0.035 | — |
| sidestick | 37 | -0.062 | -0.034 | — |
| snare | 38 | -0.104 | +0.016 | — |
| hat_closed | 42 | -0.084 | +0.004 | — |
| tom_low | 43 | -0.082 | -0.082 | — |
| hat_pedal | 44 | -0.153 | -0.047 | — |
| hat_open | 46 | -0.123 | -0.168 | — |
| tom_mid | 47 | -0.068 | -0.068 | — |
| crash | 49 | -0.179 | -0.179 | — |
| tom_high | 50 | -0.089 | -0.012 | — |
| ride | 51 | -0.096 | -0.051 | — |

**Coups probabilistes** (`probability`, A6) : 44 coups de `percussion.toml` jouent avec la probabilité par pas mesurée dans le
groupe (grosse caisse, caisse claire, cross-stick entre 0,08 et le seuil de la densité ; notes fantômes à la densité 2),
au lieu d'un coup certain dès 0,25 (0,3 pour funk et New Orleans) ou absent : le tirage est sans état (graine, mesure, pas, note) donc stable par graine.

**Voicings** : `chords.color` (tension 9, 11 ou 13 que la qualité permet, jamais en conflit avec la grille, D4) et
`chords.register_drift` (dérive bornée du centre) sont déclarés ; ils n'agissent que sur les familles de voicings qui les
lisent (pas `close_triad`).

## Contrat et preuves

`harmonium_core/tests/gmd_funk_neworleans_profile_tests.rs` : les mêmes contrats que `gmd_funk`, joués
sur les deux profils (grille, basse sur les changements, comping dans l'accord publié, kit, feel droit,
tempo 82..124, arc, variété sur 16 graines). Variété et distance au corpus :
`specs/019-bibliotheque-accompagnements/b47-funk-neworleans.md`.

## Limites

- **Trois fichiers de second line.** Le style qui donne son nom est « juste » (3 fichiers, 2 batteurs) ; la
  batterie vient pour l'essentiel de `neworleans/funk` (8 fichiers). Les roulements de caisse claire de
  seize doubles croches sont des composites de probabilités, pas des rudiments : le schéma n'a pas de coup
  fixe ; les notes fantômes sont probabilistes (R2), pas de roulé.
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
- **Micro-timing** : médianes par voix (A6, R2) ; la dispersion mesurée (0,11 à 0,20 pas) n'est pas reproduite : un `jitter` de batterie partage son tirage avec la basse à notes de passage et la déplacerait (contrat `passing_keeps_neighbour_tracks_and_strong_bass_unchanged`). Le décalage est mesuré en ms et converti au tempo par défaut : il dérive de la mesure aux autres tempos tirés.
- **Phrases du GMD** : fenêtres de 2 et 4 mesures alignées sur la première mesure ; pas de phrases de 8
  mesures. Relances : 4 groupes, 2 batteurs, `fills_none_weight` est un réglage.
- **Habillage par graine** (R2) : groupes indépendants, aucun lien entre kit, tempo et instruments (un kit « power » peut tomber avec le tempo le plus lent) ; tonalités, instruments, kits, voicings, registre, énergie et arcs sont de métier, jugés par des preuves chiffrées seulement (D5). Les options de marche d'A4 (notes fantômes, croches, notes répétées, arrivée) sont exclusives de `motifs` et de `passing` : la basse garde ses riffs.
- **Mixage** : réglé au LUFS en solo (graine 1), aucune écoute.

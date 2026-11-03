# B4 + B7 — Profils `gmd_funk` et `gmd_neworleans_funk_secondline`

Date : 2026-11-03. Branche `wquintal/b47-funk-neworleans`, partie de `main` à `0fe8b52`
(`TUNING_VERSION` 26, **inchangé**). Données seulement : aucune ligne de moteur, aucun mécanisme ajouté ;
deux dossiers de profil neufs, un test de contrat, cinq scripts de reproduction. Les 16 profils existants
ne sont pas touchés (preuves plus bas).

## Ce qui est livré

| Profil | Source | Chorus | Tempo | Instruments | Contenu |
|---|---|---|---|---|---|
| `gmd_funk` (B4) | GMD `funk` : 36 fichiers `beat` (« assez »), 41 `fill` (« juste ») ; basse : WJazzD `FUNK` (« juste ») | 16 mesures, 5 chorus par air | 96 (80 à 130), droit 0,5 | guitare claire 27 (notes guides), basse aux doigts 33, kit standard 128/0 | 5 boucles de grille, 8 riffs de basse, 12 rythmes de comping et 17 phrases, 24 patterns, 102 phrases et 8 relances de batterie, arc |
| `gmd_neworleans_funk_secondline` (B7) | GMD `neworleans/funk` (8 `beat`, 40 `fill`) et `neworleans/secondline` (3 `beat`) : 51 des 53 fichiers `neworleans` | 12 mesures, 5 chorus par air | 93 (82 à 124), droit 0,5 | piano 0 (drop 2), tuba 58, kit standard 128/0 | 5 boucles de grille, 7 riffs de basse, 16 rythmes de comping et 25 phrases, 12 patterns, 36 phrases et 8 relances de batterie, arc |

Chaque dossier : `profile.toml`, `percussion.toml` (généré), `arc.toml`, `dynamics.toml`, `set.toml`,
`arrangement.toml`, `cells.toml` (banque minimale exigée par le chargeur), `README.md` (provenance, mesure →
valeur, limites). Mentions de licence (GMD CC BY 4.0, WJazzD ODbL §4.3) dans `[meta] corpus` et dans chaque
README. Aucune banque ni preset neuf : GeneralUser GS, kit GM. Les profils entrent dans
`harmonium_determinism` et dans le studio sans rien d'autre (`StyleProfile::distributed()`, compilés par
`build.rs`).

Scripts (labo en lecture seule, archive GMD vérifiée par SHA256) :

- `b47_batterie.py` : régénère `percussion.toml` (voir « Batterie » ci-dessous) ;
- `b47_calibrage_kit.py` : niveau de chaque voix de batterie sur le kit de GeneralUser GS (LUFS en solo) ;
- `b47_distance_gmd.py` : corpus de référence pour `make style-distance` (hors dépôt) ;
- `b47_seuils_v2.py`, `b47_distance_resume.py` : tableaux de ce compte rendu.

Documentation : `profiles/README.md` (deux lignes de tableau, un paragraphe), `plan.md` (ligne d'état).

## Choix de conception

- **Noms par source précise** (principe VII) : `gmd_funk` (style `funk` du GMD) ;
  `gmd_neworleans_funk_secondline` (styles `neworleans/funk` et `neworleans/secondline`). Ni « funk » ni
  « New Orleans » génériques. Pas d'alias.
- **Une partie, un chorus** : `[form] sequence = [0]`, une partie de 16 (funk : WJazzD, chorus de 16 mesures,
  le plus fréquent) ou 12 mesures (New Orleans : la longueur du blues) ; cinq chorus par air, nombre
  connu, donc la retombée de l'arc joue. Training remplace la boucle par la grille de l'utilisateur.
- **Cinq boucles de grille par défaut** par profil (formes communes écrites, vamps à un ou deux accords,
  blues de la Nouvelle-Orléans, forme de fanfare), tirées par la graine, relatives à la tonique
  (`fixed_tonality` mi dorien ; si bémol mixolydien). Aucune réharmonisation (D4). Rythme harmonique du
  funk d'après WJazzD : 82 % de mesures à un accord, changement au temps 1 (74 %) ou 3 (25 %).
- **Feel** : 16 pas droits, swing 0,5, tempos de la source (voir les README).
- **Basse en riffs** (`bass.motifs`, `reference = "chord_root"`) : ils suivent l'accord publié et arrivent sur
  sa basse à chaque changement. Pas de marche : une basse funk n'est pas une marche, et la marche de WJazzD
  n'a pas de croches. Fondamentale, quinte, septième mineure, octave, sans tierce (le riff ignore la
  qualité). Une « tenue » est écrite en deux blanches : une ronde ne suit pas un changement d'accord
  au temps 3 (test de contrat qui l'a révélé).
- **Calibrage WJazzD (funk, quand c'est mesuré)** : registre (30..41 pour la fondamentale, étendue 28..53 :
  hauteur de basse p1 31, p99 51), approches (`passing` : aucune 68, chromatique 22, degré 10), rythme
  harmonique, tempos, longueur de chorus. Le reste de la basse et le comping sont des règles de métier
  (aucun corpus : WJazzD n'a pas de croches ; aucun corpus de comping, décision F3 de 018). New Orleans :
  aucun sous-ensemble WJazzD, basse et comping entièrement de métier ; les positions des riffs de tuba
  reprennent celles de la grosse caisse mesurée du second line (`[0, 3, 6, 8, 11, 14]`).
- **Comping** : guitare claire en notes guides (funk) et piano à quatre sons (New Orleans), trois
  étiquettes `sparse` / `medium` / `dense` (tenues, croches décalées, doubles croches avec notes fantômes
  à probabilité réduite), phrases de 2 et 4 mesures, une mise en place `part_end` par étiquette.
- **Arc** (cinq chorus, mêmes valeurs de métier pour les deux profils) : exposition légère, deux chorus de
  corps (un break de la guitare ou du piano à mi-chorus, ou de l'accompagnement avant le chorus suivant),
  sommet avec stop-time une fois sur deux, retombée avec stop-time ; `repeat_from` pour plus de cinq chorus.
  Les vélocités par piste de `dynamics.toml` sont élargies (0,6 à 1,3) pour que l'arc s'entende.
- **Mixage** : réglé sur le LUFS en solo de chaque piste (graine 1), pas à l'oreille : voir « Mixage ».

### Batterie : construction (`b47_batterie.py`)

Le GMD donne des probabilités par pas, pas des phrases (L2 § 6 points 5 et 6) ; la construction ne copie aucune
mesure d'un enregistrement.

1. **Groupes de mesures** : les mesures 4/4 droites (`gmd_styles.analyse_archive`) sont décrites par leur
   grosse caisse, leur caisse claire (cross-stick compris) et leur timekeeper (charleston, ride, cloche,
   charleston ouvert) ; k-moyennes à `random_state` fixe (12 groupes pour le funk, 8 pour New Orleans).
   Un groupe est retenu s'il compte ≥ 20 mesures, ≥ 3 fichiers, ≥ 2 batteurs, aucun fichier au-delà de 60 %
   (règles d'E7 et de L2), et s'il est un groove : funk, un timekeeper d'au moins 4 coups et une caisse
   claire au temps 2 ou 4 ; New Orleans, une grosse caisse d'au moins 2 coups et un roulement de caisse
   claire ou un timekeeper. Funk : 8 groupes retenus sur 12 (1 842 mesures droites lues, 1 473 retenues) ; New
   Orleans : 4 sur 8 (608 lues, 436 retenues).
2. **Patterns** : par groupe retenu et par densité, un coup par (rôle, pas) dont la probabilité par pas atteint
   le seuil de la densité (0 : ossature, timekeeper 0,5, grosse caisse et caisse claire 0,75 ; 1 : tout 0,5 ;
   2 : grosse caisse, caisse claire, cross-stick 0,3, le reste 0,5) ; poids = effectif de mesures ;
   étiquette = tiers d'activité.
3. **Phrases** (A1) : par fichier, suite des groupes de ses mesures, fenêtres de 2 et 4 mesures alignées sur la
   première ; suite gardée si ≥ 3 occurrences sur ≥ 2 batteurs ; poids = effectif ; une suite dont la dernière
   mesure change de groupe est aussi proposée en `part_end` (la mesure sur laquelle A1 pose une mise en
   place). Funk : 21 suites ; New Orleans : 9.
4. **Relances** : les fichiers `fill` groupés par longueur (1 ou 2 mesures) et par départ (mesure entière ou
   levée à partir du pas 8), scindés par k-moyennes quand le groupe compte ≥ 6 fichiers ; coup de probabilité
   ≥ 0,3 ; `start_step` = plus petit pas porteur ; poids = effectif de fichiers ; densités 1 et 2. Quatre
   groupes dans chaque profil. `fills_none_weight` = 0,8 × le poids total (44 % de fins de phrase sans
   relance) : valeur de service, le GMD isole les relances.
5. **Niveaux** : le GMD est joué sur un Roland TD-11, le kit GM de GeneralUser GS ne sonne pas pareil. Un gain
   par rôle est calé sur le LUFS en solo mesuré à vélocité 50 et 100 (droite en log2, 10 à 16 LU par octave)
   contre un niveau visé par rôle ; la forme des accents (rapport à la médiane du rôle) est conservée.

## Preuves

### Variété entre graines et entre chorus (`seed-variety --chorus 16`, 16 graines, V0)

`harmonium-lab seed-variety --profile <dossier> --seeds 16 --chorus 16`, binaire de la branche en `--release`, lancé
depuis `~/Project/bascanada/harmonium`. `wjazzd_swing` (V0, grille par défaut de 44 mesures) est remesuré avec le
même binaire (profil non touché). Une ligne par seuil V2 proposé par V0 (`b47_seuils_v2.py` lit les JSON) :

| # | Seuil V2 | `wjazzd_swing` | `gmd_funk` | `gmd_neworleans_funk_secondline` |
|---|---|---|---|---|
| 1 | paires de chorus identiques, accompagnement : 0 % ; 16 / 16 chorus distincts | 0 % ; 16,0 / 16 ✓ | 0 % ; 16,0 / 16 ✓ | 0 % ; 16,0 / 16 ✓ |
| 2 | paires identiques par couche (comping rythme / voicings / basse / batterie) ≤ 5 % | 0 / 0 / 0 / **100** % ✗ | 0 / 0 / **25** / 2 % ✗ | 0 / 0 / 0 / 4 % ✓ |
| 3 | accords changés d'un chorus au suivant ≤ 2 % | 0 % ✓ | 0 % ✓ | 0 % ✓ |
| 4 | comping rythme changé (stable) ≥ 75 % | 80 % ✓ | 93 % ✓ | 78 % ✓ |
| 5 | comping voicings changés (stable) ≥ 65 % | 74 % ✓ | 90 % ✓ | 73 % ✓ |
| 6 | basse changée (stable) ≥ 80 % | 93 % ✓ | **10 %** ✗ | **45 %** ✗ |
| 7 | batterie changée (stable) ≥ 30 % ; distance graduée ≥ 10 % | 0 % ; 0 % ✗ | 74 % ; 54 % ✓ | 74 % ; 34 % ✓ |
| 8 | séquences distinctes sur 16 graines (comping rythme / basse / batterie) ≥ 12 | 16 / 16 / 11 ✗ | 16 / 16 / 16 ✓ | 16 / 16 / 16 ✓ |
| 9 | orchestration (A2) | 1 programme par piste | 1 programme par piste | 1 programme par piste |
| 10 | arc, densité (notes / mesure) : cv ≥ 10 %, étendue ≥ 25 % | 2 % ; 3 % ✗ | 18 % ; 55 % ✓ | 23 % ; 66 % ✓ |
| 11 | arc, vélocité moyenne : cv ≥ 3 %, étendue ≥ 8 % | 0 % ; 0 % ✗ | 4 % ; 14 % ✓ | 5 % ; 19 % ✓ |
| 12 | arc, `intensity` du moteur : étendue ≥ 20 % | 0 % ✗ | 0 % ✗ | 0 % ✗ |

Lecture :

- **Tenus** : 1, 3, 4, 5, 7, 8, 10, 11 par les deux profils. La batterie, que `wjazzd_swing` rejoue à l'identique
  d'un chorus à l'autre (100 % de paires identiques, 11 séquences de batterie distinctes sur 16 graines), est
  ici de 2 à 4 % de paires identiques et 16 séquences distinctes sur 16 graines (batterie : 14,6 chorus distincts
  par graine pour le funk, 12,5 pour New Orleans). L'arc donne 55 et 66 % d'étendue de densité et 14 et 19 %
  de vélocité (V0 : 3 % et 0 %).
- **Non tenus, limite du moteur** : 2 (basse du funk) et 6 (basse des deux profils). Le plan de motifs de basse est
  fixé une fois par air et rejoué à chaque chorus (`profiles/README.md`, « Motifs de basse ») ; la seule variation
  d'un chorus à l'autre est celle des notes d'approche (10 % des mesures changées pour le funk, 45 % pour New
  Orleans dont les riffs ont plus de notes à approcher). `wjazzd_swing` passe 6 parce que sa marche se retire
  mesure par mesure. A4 (basse, vague 3) est la tâche qui peut changer cela.
- **12** : l'arc multiplie l'intensité lue par les courbes de `dynamics` ; il ne change pas le champ `intensity`
  publié par le moteur que mesure V0. Même constat pour la fixture de l'arc ; ce n'est pas une propriété de ces
  profils.
- **9** : A2 (habillage par graine) n'existe pas encore ; le brief demande de ne pas le simuler (vague 4).
- La basse est la seule couche où `gmd_funk` varie moins que `wjazzd_swing` : une basse funk est un riff, et le
  riff qui se répète est le style.

Variété entre graines à chorus égal (distance graduée moyenne, `inter`) : funk 62 % (accompagnement), 45 %
(comping rythme), 47 % (batterie), 84 % (basse) ; New Orleans 58, 44, 26, 91 % ; `wjazzd_swing` 38, 68, 19, 59 %.
Accords : 5 grilles distinctes sur 16 graines (funk), 4 (New Orleans), 1 (`wjazzd_swing`).

### Distance au corpus GMD (`make style-distance`) contre une copie non calibrée

`make style-distance` compare un rendu du moteur à des pistes de référence (audio + transcriptions). Le GMD est du
MIDI de batterie : `b47_distance_gmd.py` fabrique, hors dépôt, un corpus par profil (7 fichiers `beat` du funk, 9 de
New Orleans, 2 à 3 par batteur ; MIDI rendu par `fluidsynth` avec GeneralUser GS, coups kick / snare / charleston
transcrits en doubles croches). **Copie non calibrée** : le même profil, `percussion.toml` remplacé par un rock en
croches (grosse caisse 1 et 3, caisse claire 2 et 4, charleston en croches ; les trois étiquettes de l'arc portent le
même pattern ; aucune phrase ni relance) et tempo 120 (60 à 180) ; tout le reste égal. 10 graines × 48 mesures.
Scores 0 à 100 (100 = identique), moyenne sur les pistes de référence (`b47_distance_resume.py`) :

| Mesure (symbolique) | `gmd_funk` | copie non calibrée | `gmd_neworleans_funk_secondline` | copie non calibrée |
|---|---:|---:|---:|---:|
| un coup quelconque par pas | 89,8 | 69,2 | 91,7 | 70,1 |
| grosse caisse | 55,5 | 35,2 | 59,5 | 44,4 |
| caisse claire | 77,3 | 33,1 | 74,4 | 23,8 |
| charleston | 74,3 | 78,1 | 64,5 | 71,7 |
| **groupe batterie** | **74,3** | 53,9 | **72,5** | 52,5 |

- La calibration rapproche la batterie du corpus de **20 points** pour les deux profils ; la caisse claire gagne le plus
  (+ 44 et + 51 points : les backbeats, ghost notes et roulements du GMD contre deux coups fixes). Le charleston perd
  4 à 7 points : le rock en croches y colle mieux à la fréquence par pas que les composites de doubles croches.
- **Non interprétables ici** : `loop_drums` (34,9 et 53,1 contre 97,8 pour la copie) mesure si la voix se répète à
  la longueur d'une boucle de référence ; j'ai déclaré 4 mesures par défaut, un batteur du GMD ne joue pas une boucle,
  et le but de ces profils est justement de ne pas se répéter. L'audio global (11,6 contre 11,6 ; 11,8 contre 14,7)
  compare un mixage complet à une batterie rendue seule et ne sépare pas les deux copies. Basse et accords : référence
  vide (le GMD n'en a pas).

### Mixage (LUFS intégrés en solo, graine 1, `ffmpeg ebur128`)

| Piste | `gmd_funk` (40 mesures) | `gmd_neworleans_funk_secondline` (36 mesures) |
|---|---:|---:|
| Chords | −25,6 | −26,4 |
| Bass | −25,0 | −24,2 |
| PercussionHigh | −25,7 | −25,0 |
| PercussionLow (grosse caisse) | −31,6 | −30,3 |
| mix complet | −21,2 | −20,3 |

Cibles de métier : accords et basse vers −25 LUFS, batterie haute vers −25, mix vers −21 (`wjazzd_swing` : −22).
`PercussionLow` et `PercussionHigh` partagent le canal 9 du kit GM : le niveau de `PercussionHigh` gagne ; la
grosse caisse se règle par la vélocité de sa voix (gain par rôle), pas par `level_db`.

### Contrat (`harmonium_core/tests/gmd_funk_neworleans_profile_tests.rs`)

Huit tests sur le flux de `MeasureSnapshot`, joués sur les deux profils (graines 1, 2, 3, 17 et 1 à 16) :

| Contrat | Test |
|---|---|
| la grille par défaut est une des 5 boucles écrites, jouée à l'identique à chaque chorus, ≥ 3 grilles sur 16 graines | `la_grille_par_defaut_est_une_des_boucles_ecrites_…` |
| la basse arrive sur la basse publiée à chaque changement d'accord (hors stop-time), dans la fenêtre du profil | `la_basse_arrive_sur_la_basse_publiee_…` |
| la basse ne joue que fondamentale, quinte, septième, octave ou une approche à 1-2 demi-tons de la note suivante | `la_basse_ne_joue_que_la_fondamentale_…` |
| le comping ne joue que des notes de l'accord publié (celui de la mesure suivante pour une attaque anticipée), sans fondamentale pour les notes guides, dans la tessiture | `le_comping_ne_joue_que_des_notes_de_l_accord_publie` |
| la batterie reste dans le kit, grosse caisse dans ≥ 70 % des mesures, backbeat (funk) | `la_batterie_reste_dans_le_kit_…` |
| feel droit en 16 pas, tempo dans 80..130 / 82..124 à toutes les intensités, doubles croches à la densité 2 | `le_feel_est_droit_en_seize_pas_…` |
| exposition et retombée plus légères que le sommet, stop-time de la retombée | `l_arc_creuse_l_ecart_…` |
| même graine, même sortie ; ≥ 14 comping, ≥ 14 batteries, ≥ 12 basses distincts sur 16 graines | `la_graine_change_le_comping_…` |

Mutations vérifiées (chaque mutation fait échouer au moins un test) : swing à 0,62 ; stop-time de la retombée
retiré ; référence de riff `tonic` au lieu de `chord_root`. Trois contrats génériques de la suite ont aussi
attrapé des défauts avant livraison : `profile_files_tests` (embarqué = disque), `passing_keeps_neighbour_tracks…`
(un `jitter` de batterie partage son tirage avec la basse à notes de passage : retiré), le test de ce profil lui-même
(une ronde de basse ne suit pas un changement d'accord à mi-mesure : deux blanches).

### Vérifications d'acceptation

VERIFICATION

## Ce qui ne s'exprime pas dans le schéma (L2 § 6)

Rien n'est contourné ; chaque point est listé dans les README des profils.

1. **Coup probabiliste** (point 5) : les ghost notes de la caisse claire sont des coups fixes à faible vélocité dans un
   pattern construit sur un seuil de probabilité ; aucune probabilité par coup côté batterie (le comping a
   `probability`).
2. **Micro-timing par rôle** (point 3) : `PercussionHigh` mêle charleston, caisse claire, ride et toms ; un seul décalage
   par pas, médianes seulement, **pas de dispersion** (un `jitter` de batterie casse le contrat de la basse à notes de
   passage, voir plus haut). Mesuré : dispersion 0,13 à 0,20 pas.
3. **Phrases de 8 mesures** (point 6) : aucune suite de 8 groupes vue 3 fois sur 2 batteurs ; phrases de 2 et 4 mesures
   seulement, sur les groupes de mesures du profil, pas sur des motifs mesurés par rôle.
4. **Relances** (point 4) : 4 groupes par profil, 2 batteurs ; la part de fins de phrase sans relance n'est pas
   mesurable (`fills_none_weight` de service) ; une relance de deux mesures coupe tout le charleston.
5. **Basse** (point 8) : croches, notes fantômes, silences de basse non mesurables dans WJazzD ; la part d'arrivée sur la
   basse de l'accord au changement (45 % [34–63]) n'est pas reproduite (un riff arrive toujours sur la basse publiée) ;
   l'approche mesurée (68 / 22 / 10) vaut pour le temps 4 avant un changement à la barre, le moteur l'applique à
   toute note de riff qui change de hauteur (`next_target_differs`), donc plus souvent.
6. **Feel ternaire** (point 1) : sans objet ici (GMD funk et neworleans : droit) ; `funk/purdieshuffle` et
   `neworleans/shuffle` ne sont pas lus (B6).

## Manques moteur relevés (rien n'a été changé)

- **Plan de motifs de basse fixé par air** : pas de variation de riff d'un chorus à l'autre ; les seuils V2 6 (et 2 pour
  le funk) ne se tiennent pas. À traiter avec A4 : un tirage de motif par chorus ou par passe, sous étiquette d'arc
  (`arc.stages[].variants[].bass = [{ tag }]` comme pour le comping et la batterie).
- **`passing` ne se restreint pas à un changement d'accord à la barre** : `chord_change_inside_bar` couvre le
  changement intérieur à la mesure, pas celui de la barre ; les parts mesurées ne s'appliquent donc pas telles quelles.
- **`intensity` publiée** : l'arc n'apparaît pas dans le champ `intensity` du snapshot (seuil V2 12).
- **Contrainte non documentée** : un `jitter` de percussion déplace tous les tirages qui suivent la basse quand la basse a des notes de
  passage (`passing_keeps_neighbour_tracks_and_strong_bass_unchanged`) ; l'ordre de tirage des pistes n'est écrit nulle part.
- **Disque** : la machine a atteint 100 % en cours de tâche (le dossier `target/` de chaque worktree de la vague fait de
  2 à 9 Go) ; les rendus de V0 et de `style-distance` échouent alors en `No space left on device` (rejoués).

## Limites

- Tout est jugé par des chiffres (D5), rien à l'oreille : les patterns de batterie sont des composites de probabilités,
  rigides ; quatre batteurs, un kit électronique. New Orleans : trois fichiers de second line, basse et comping sans source.
- **Habillage par graine** (A2) : absent, tempo, tonalité, instruments, voicings et kit sont fixes par profil (vague 4).
- Tempo constant sur un air ; pas de swing selon le tempo (droit).
- Le plan de basse ne varie pas d'un chorus à l'autre (manque moteur ci-dessus).

## Reproduire

```sh
# batterie (labo en lecture seule)
cd ~/Project/bascanada/harmonium_lab
.venv/bin/python ~/Project/bascanada/harmonium/specs/019-bibliotheque-accompagnements/b47_batterie.py gmd_funk > percussion.toml
# variété (CLI release de la branche)
.venv/bin/harmonium-lab seed-variety --profile <dossier du profil> --seeds 16 --chorus 16 \
  --engine-dir ~/Project/bascanada/harmonium --engine-bin <harmonium-cli> --label B47 --work-dir /tmp/b47/truth -o v.json
python specs/019-bibliotheque-accompagnements/b47_seuils_v2.py wjazzd_swing.json gmd_funk.json gmd_neworleans_funk_secondline.json
# distance au GMD
.venv/bin/python specs/019-bibliotheque-accompagnements/b47_distance_gmd.py gmd_funk /tmp/b47/corpus_gmd_funk
make style-distance PROFILE=<dossier> REF=gmd_funk TRACKS=<ids> STYLE_CORPUS=/tmp/b47/corpus_gmd_funk STEPS_PER_BAR=16 \
  ENGINE_DIR=<moteur> BANKS="generaluser=<GeneralUser-GS.sf2>" DIST_OUT=d.json DIST_SEEDS=10 DIST_BARS=48
python specs/019-bibliotheque-accompagnements/b47_distance_resume.py d.json d_uncal.json
```

# Profil `gmd_funk` (phase 019, B4)

Accompagnement funk, doubles croches droites, sous une grille d'accords : basse, comping et batterie ne
dépendent que de l'accord publié (profil `loop`, `[fixed_tonality]`, pas de piste `Melody`). Fait pour
training : l'utilisateur fournit sa grille, le profil joue dessous ; sans grille, cinq boucles de seize
mesures tirées par la graine.

Le nom est celui de la source qui définit le style (principe VII) : le style `funk` du Groove MIDI
Dataset. Il ne dit rien d'un « funk » général : un kit électronique, quatre batteurs, une basse
calibrée sur vingt solos de fusion/funk.

## Provenance et droits

| Couche | Source | Licence | Mention (aussi dans `[meta] corpus`) |
|---|---|---|---|
| Batterie : groupes de mesures, patterns, phrases, relances, micro-timing, tempos | Groove MIDI Dataset v1.0.0, style `funk` : 36 fichiers `beat` (4 batteurs, 1 904 mesures, « assez »), 41 fichiers `fill` (2 batteurs, 52 mesures, « juste »). SHA256 de l'archive vérifié avant toute lecture : `651cbc52…1dcc1e` | CC BY 4.0 | Gillick, Roberts, Engel, Eck, Bamman, « Learning to Groove with Inverse Sequence Transformations », ICML 2019 ; Google LLC |
| Basse : registre, parts d'approche, rythme harmonique, tempos, longueurs de chorus | Weimar Jazz Database v2.1, `rhythmfeel = FUNK` : 20 solos, 16 avec mesures valides, 928 mesures, 31,3 min, « juste » | ODbL 1.0 (contenus DbCL 1.0) | « Contains information from the Weimar Jazz Database (Jazzomat Research Project, HfM Weimar), which is made available here under the Open Database License (ODbL) … » (ODbL §4.3) |
| Basse (riffs), comping, grilles par défaut, arc, mixage | Règles de métier écrites pour ce profil | — | Aucun corpus ouvert ne mesure les croches d'une basse funk (WJazzD : une hauteur par temps) ni le comping |

Aucune note de solo, aucune ligne de basse, aucune mesure d'un enregistrement n'est copiée : seuls des
agrégats (probabilités par pas, comptes, parts, centiles) servent. Les fichiers MIDI du GMD ne sont pas
dans le dépôt. Mesures du labo : `harmonium_lab/references/gmd-funk/`, `references/wjazzd-funk/`,
`references/l2-ingestion.md` ; la batterie est régénérée par
`specs/019-bibliotheque-accompagnements/b47_batterie.py gmd_funk` (labo en lecture seule).

## Mesure → valeur

| Paramètre | Valeur | Mesure source |
|---|---|---|
| `meter.steps_per_bar`, `default_swing_ratio` | 16 pas, 0,5 | GMD funk : 34 fichiers sur 36 « straight » (grille 16 pas, doubles croches), rapport de swing médian 0,500 (p10 0,480, p90 0,500) |
| `default_bpm` | 96 | GMD funk `beat` 4/4 droit : médiane 96 (moyenne 97,6) |
| `min_bpm` | 80 | GMD p10 80 ; WJazzD FUNK minimum 84 |
| `max_bpm` | 130 | GMD p90 112 ; WJazzD FUNK médiane 117,8, p75 144,9 ; 130 : au-dessus des deux médianes, sous le p75 de WJazzD (choix, pas une mesure) |
| Longueur du chorus | 16 mesures | WJazzD FUNK : 24 chorus complets, 16 mesures (7), 12 (5), autres (12) |
| Grilles par défaut | 5 boucles de 16 mesures, un accord par mesure sauf un changement au temps 3 | WJazzD FUNK : 82 % de mesures à un accord (IC 61–97 %), changement sur le temps 1 dans 74 % des cas et sur le temps 3 dans 25 %, accord de 2 temps (49 %) ou 4 temps (46 %) |
| `bass.low`, `high`, `walk_low`, `walk_high` | 30, 41, 28, 53 | WJazzD FUNK : hauteur de basse p1 31, p99 51, étendue 28 à 53 ; la fondamentale se place dans 30..41, l'octave monte à 53 |
| `bass.passing` | `none_weight` 68, chromatique 22, degré de gamme 10 | WJazzD FUNK : approche du temps 4 avant un changement à la barre : aucune 68 % [62–73], chromatique 22 % [17–28], diatonique 10 % [6–16] (494 événements, 12 solos) |
| Batterie : groupes de mesures | 12 groupes (k-moyennes) dont 8 retenus : 20 mesures, 3 fichiers, 2 batteurs au moins, aucun fichier au-delà de 60 % ; un timekeeper d'au moins 4 coups et une caisse claire au temps 2 ou 4 | GMD funk `beat` (voir « Batterie ») |
| Batterie : seuils par densité | densité 0 : timekeeper ≥ 0,5, grosse caisse et caisse claire ≥ 0,75 ; densité 1 : tout ≥ 0,5 ; densité 2 : grosse caisse, caisse claire, cross-stick ≥ 0,3, le reste ≥ 0,5 | probabilités par pas du groupe |
| Batterie : phrases | 21 suites de 2 et 4 mesures (≥ 3 occurrences sur ≥ 2 batteurs), poids = occurrences, 3 densités, `part_end` pour les suites dont la dernière mesure change de groupe | GMD funk `beat`, fenêtres alignées sur la première mesure |
| Batterie : relances | 4 groupes de fichiers `fill` (1 mesure du pas 0 : 21 et 5 fichiers ; 2 mesures du pas 14 : 6 ; 2 mesures du pas 0 : 4), densités 1 et 2 | GMD funk `fill` : départ au temps 1 dans 27 fichiers sur 40, au temps 4 dans 7 ; 1 mesure dans 28 fichiers, 2 dans 11 |
| `fills_none_weight` | 29 (44 % de fins de phrase sans relance) | valeur de service : le GMD isole les relances, la part de fins de phrase sans relance n'y est pas mesurable |
| `performance.tracks.Percussion*` | décalage −0,04 à −0,07 pas, sans `jitter` | GMD funk, médianes de l'écart à la grille : charleston −0,04, grosse caisse −0,04 (hors temps) à −0,07 (sur le temps), caisse claire −0,02 à −0,07 |
| Niveau de chaque voix de batterie | gain par rôle, voir le script | LUFS en solo mesuré sur le kit 128/0 de GeneralUser GS à vélocité 50 et 100, droite en log2 de la vélocité, niveau visé par rôle |
| `mix.tracks` | accords −17 dB, basse −18,5 dB, percussion −19 dB, maître +25 dB | LUFS intégrés en solo, graine 1, 40 mesures : accords −26,2, basse −24,9, batterie haute −25,0, grosse caisse −31,8, mix −21,0 |

Valeurs de métier, aucune mesure (listées dans les commentaires du `profile.toml`) : les riffs de basse
(croches et doubles croches syncopées, fondamentale-octave, quinte et septième mineure, sans tierce
parce que le riff ignore la qualité de l'accord), les rythmes de comping (banque d'une mesure : tenues,
croches décalées, doubles croches avec notes fantômes à probabilité réduite) et leurs phrases, l'arc
sur cinq chorus, le gratté de la guitare, le mixage.

## Ce que le profil joue

- **Grille** : cinq boucles de seize mesures en mi dorien (`fixed_tonality` mi, `dorian`), relatives à la
  tonique : `I9` pendant seize mesures (poids 3) ; `I9` 4, `IV9` 4, `I9` 4, `IV9` 2, `I9` 2 (3) ;
  `Im7` 4, `IV7` 4, `Im7` 4, `IV7` 2, `bVII7` 2 (3) ; `Im7` et `bVII7` deux mesures chacun (2) ; huit
  mesures `Im7` 2, `IVm7` 2, `Im7` 2, `bVII7`, `V7` à la blanche puis `bVII7`, deux fois (2). La graine en
  tire une par air ; training la remplace par la sienne.
- **Basse** (basse aux doigts, GeneralUser GS 33) : huit riffs de une ou deux mesures, `reference =
  "chord_root"` : ils suivent l'accord publié et arrivent sur sa basse à chaque changement. Le plan de
  motifs est fixé par air ; une approche chromatique ou diatonique (parts mesurées) précède une note de
  riff différente.
- **Comping** (guitare claire 27, notes guides : tierce, septième, tension de la qualité ; gratté de
  quelques millisecondes) : douze rythmes d'une mesure en trois étiquettes (`sparse` : tenues ; `medium` :
  croches décalées ; `dense` : doubles croches à notes fantômes) et dix-sept phrases de 2 et 4 mesures
  (suites de ces rythmes, une mise en place `part_end` par étiquette).
- **Batterie** (kit standard 128/0) : 24 patterns (8 groupes de mesures × 3 densités), 102 phrases
  (21 suites × 3 densités, plus 13 suites aussi en `part_end`), 8 relances (4 × 2 densités).
- **Arc** (cinq chorus par air, nombre connu) : exposition (comping `sparse` ou `medium`, batterie claire,
  doux), deux chorus de corps (`medium` ou `dense`, un break de guitare à mi-chorus ou un break guitare +
  basse avant le chorus suivant), sommet (`dense`, plus fort, stop-time une fois sur deux), retombée
  (`sparse`, stop-time sur le un des deux dernières mesures). Au-delà de cinq chorus : sommet puis corps
  et sommet en boucle.

## Contrat et preuves

`harmonium_core/tests/gmd_funk_neworleans_profile_tests.rs` (flux de `MeasureSnapshot`, graines 1, 2, 3, 17
et 1 à 16) : grille par défaut = une des cinq boucles écrites (≥ 3 tirées sur 16 graines) ; basse sur la
basse publiée à chaque changement, dans la fenêtre du profil ; basse limitée à fondamentale, quinte,
septième, octave et approches ; comping dans l'accord publié, sans sa fondamentale, dans la tessiture ;
batterie dans le kit, grosse caisse et backbeat ; feel droit en 16 pas, tempo dans 80..130 ; arc :
exposition et retombée plus légères que le sommet, stop-time de la retombée ; mêmes graine, même sortie, et
≥ 14 comping, ≥ 14 batteries, ≥ 12 basses distincts sur 16 graines. Variété et distance au corpus :
`specs/019-bibliotheque-accompagnements/b47-funk-neworleans.md`.

## Limites

- **Batterie : des composites, pas des mesures.** Chaque pattern est un agrégat de probabilités par pas d'un
  groupe de mesures (k-moyennes) : rigide (pas de coup probabiliste dans le schéma : L2 § 6 point 5), les
  notes fantômes de la caisse claire y sont des coups à vélocité faible. Quatre batteurs, un kit
  électronique : un idiome de batteur, pas du funk en général. Les patterns ne sont pas validés à l'écoute
  (D5) mais mesurés (preuve chiffrée).
- **Phrases du GMD** : fenêtres de 2 et 4 mesures alignées sur la première mesure de fichiers de 30 à 90
  mesures, sur les groupes de mesures du profil ; pas de phrases de 8 mesures (aucune suite de 8 groupes
  vue 3 fois). Le comping n'a aucune source : ses phrases sont écrites.
- **Relances** : 4 groupes seulement, 2 batteurs ; une relance de deux mesures dure toute la fin de phrase
  (pas de charleston) ; `fills_none_weight` est un réglage.
- **Basse** : WJazzD ne dit rien des croches, des notes fantômes ni des silences (L2 § 6 point 8) : les riffs
  sont écrits. La part d'arrivée sur la basse de l'accord au changement (45 % [34–63]) n'est pas
  reproduite : un riff arrive toujours sur la basse publiée. L'approche mesurée (aucune 68 %, chromatique
  22 %, diatonique 10 %) vaut pour le temps 4 avant un changement à la barre ; le moteur l'applique à tout
  changement de hauteur du riff (`next_target_differs`), donc plus souvent qu'à la mesure. Pas de passage
  sur un coup du riff : la part jouée est inférieure à la part mesurée quand le riff attaque le pas 12.
  Le plan de motifs de basse est fixé par air : la basse ne change pas d'un chorus à l'autre
  (A4 de la vague 3 l'étendra). 16 solos et 12 pour les parts « au changement » : intervalles de confiance
  larges, 8 solos sur 20 sur un vamp (55 % des mesures).
- **Micro-timing** : décalages médians seulement (rôle par pas de pulsation ; charleston et caisse claire partagent une piste, `PercussionHigh`) ; la dispersion mesurée (0,13 à 0,20 pas) n'est pas reproduite : un `jitter` de batterie partage son tirage avec la basse à notes de passage et la déplacerait (contrat `passing_keeps_neighbour_tracks_and_strong_bass_unchanged`).
- **Swing** : droit, 0,5, tous tempos (GMD funk) ; la courbe de swing selon le tempo (A6) ne concerne pas ce
  feel. Rien de ternaire : `funk/purdieshuffle` (B6) n'est pas lu.
- **Tempo** : WJazzD FUNK n'a rien au-dessus de 160 BPM ; le GMD funk n'a qu'un fichier rapide (125). Le
  tempo ne change pas d'un chorus à l'autre.
- **Habillage par graine** (A2, vague 4) : n'existe pas encore ; tempo, tonalité, instruments, voicings,
  kit sont fixes par profil. La seule variété de graine est celle des grilles, du plan de basse, des
  phrases, des relances et de l'arc.
- **Instruments** : un seul par piste (guitare claire, basse aux doigts, kit standard). Le comping n'a
  qu'un voicing (notes guides) ; Rhodes, orgue, basse slap ne sont pas tirables.
- **Mixage** : réglé au LUFS en solo (graine 1), aucune écoute ; la grosse caisse et le charleston partagent
  le canal 9 du kit GM, le niveau de `PercussionHigh` gagne, la grosse caisse se règle par sa voix.

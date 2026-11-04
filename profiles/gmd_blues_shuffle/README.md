# Profil `gmd_blues_shuffle` (phase 019, B6)

Accompagnement de **blues shuffle**, feel ternaire (12/8 écrit en 4/4 sur douze pas : un pas = un tiers de temps), sous une grille
d'accords : basse, comping et batterie ne dépendent que de l'accord publié (profil `loop`, pas de `[fixed_tonality]` (tonique tirée par
`session.toml`), pas de piste `Melody`). Fait pour training : l'utilisateur fournit sa grille, le profil joue dessous ; sans grille, un
blues de douze mesures tiré par la graine parmi cinq formes.

Le nom est celui de la source qui définit le style (principe VII) : les **shuffles** du Groove MIDI Dataset. Ce n'est pas « le blues » :
cinq fichiers de batterie, quatre batteurs, kit électronique ; une basse et un comping écrits par règles de métier.

## Provenance et droits

| Couche | Source | Licence | Mention (aussi dans `[meta] corpus`) |
|---|---|---|---|
| Batterie : groupes de mesures, patterns, phrases, relances, micro-timing, rapport de swing, plage de tempo | Groove MIDI Dataset v1.0.0 : fichiers `beat` 4/4 de feel **ternaire** de `blues/shuffle` (2 sur 4), `rock/shuffle` (2 sur 4), `funk/purdieshuffle` (1) ; relances : fichiers `fill` ternaires de `funk/purdieshuffle` (26) et de `rock` (3). Au total 5 fichiers `beat` (4 batteurs, 340 mesures) et 29 `fill`. SHA256 de l'archive vérifié avant toute lecture : `651cbc52…1dcc1e` | CC BY 4.0 | Gillick, Roberts, Engel, Eck, Bamman, « Learning to Groove with Inverse Sequence Transformations », ICML 2019 ; Groove MIDI Dataset v1.0.0, Google LLC, CC BY 4.0 (<https://creativecommons.org/licenses/by/4.0/>) |
| Basse : registre, longueur de chorus, rythme harmonique, proportions des approches | Weimar Jazz Database v2.1, groupe `blues` : 87 solos, 5 725 mesures valides (`references/wjazzd-swing/measures-bass.json`) | ODbL 1.0 (contenus DbCL 1.0) | « Contains information from the Weimar Jazz Database (Jazzomat Research Project, HfM Weimar), which is made available here under the Open Database License (ODbL) … » (ODbL §4.3) |
| Basse (riffs « boogie »), comping, grilles par défaut, arc, mixage | Règles de métier écrites pour ce profil | — | Aucun corpus ouvert ne mesure les croches d'une basse de blues ni le comping ; les grilles sont des formes communes (aucune œuvre reproduite ni nommée, même texte que `wjazzd_swing` : Compendium du US Copyright Office §802.5(A)) |

Aucune note de solo, aucune ligne de basse, aucune mesure d'un enregistrement n'est copiée : seuls des agrégats (probabilités par pas,
comptes, parts, centiles) servent. Les fichiers MIDI du GMD ne sont pas dans le dépôt. Mesures du labo :
`harmonium_lab/references/gmd-blues-shuffle/`, `gmd-rock-shuffle/`, `gmd-funk-purdieshuffle/`, `references/wjazzd-swing/` (groupe blues),
`references/l2-ingestion.md` ; la batterie se régénère par `specs/019-bibliotheque-accompagnements/b6/batterie.py` (labo en lecture
seule), le reste par `b6/emit_profile.py` et `b6/emit_session.py`.

## Pourquoi cinq fichiers de trois styles

`blues/shuffle` n'a que **quatre** fichiers `beat` (90 mesures, 2 batteurs) dont **deux seulement** sont de feel ternaire ; L2 le classe
« juste » pour le style entier mais « trop peu » une fois le feel séparé. Plutôt que de jouer deux fichiers, le profil lit aussi les autres
shuffles ternaires du GMD : `rock/shuffle` (2 fichiers ternaires, 152 mesures) et `funk/purdieshuffle` (1 fichier, 104 mesures, un seul batteur, style `purdieshuffle` du GMD ; 26 relances ternaires, de loin la plus grosse réserve de relances ternaires). Le « rock
12/8 » (`rock` : 19 fichiers ternaires) n'est **pas** lu : c'est le feel de `gmd_rock`, pas un shuffle de blues. Les fichiers droits et à
swing léger de `blues/shuffle` et de `rock/shuffle` (autre grille) sont écartés ; `neworleans/shuffle` (1 fichier droit) aussi.

## Mesure → valeur

| Paramètre | Valeur | Mesure source |
|---|---|---|
| `meter.steps_per_bar`, `steps_per_pulse` | 12 pas, 3 par temps | GMD : feel `ternary` = triolets, 12 pas en 4/4 (L2) ; le moteur place la croche de contretemps au pas 2 de chaque temps (A6) |
| `default_swing_ratio` | 0,664 | rapport de swing des 5 fichiers ternaires : 0,627 (112 BPM), 0,650 (130), 0,664 (134), 0,666 (72), 0,696 (125) ; médiane 0,664, aucune pente avec le tempo |
| `[swing]` (courbe selon le tempo, A6) | **absent** | la courbe d'A6 est celle de JTD (trios de piano de jazz, 102–299 BPM) ; les 5 fichiers du GMD ne montrent aucune pente (72 à 134 BPM) : un rapport unique |
| `default_bpm`, `min_bpm`, `max_bpm` | 125, 72, 134 | BPM des 5 fichiers : 72, 112, 125, 130, 134 (médiane 125, extrêmes de la source) |
| Longueur du chorus | 12 mesures | WJazzD blues : 378 chorus sur 420 (90 %) font douze mesures |
| Grilles par défaut | 5 boucles de 12 mesures, un accord par mesure | WJazzD blues : 92 % de mesures à un accord, changement au temps 1 dans 89 % des cas, au temps 3 dans 11 % |
| `bass.low`, `high`, `walk_low`, `walk_high` | 30, 41, 28, 53 | WJazzD blues : hauteur de basse p5 34, médiane 42, p95 50, étendue 28 à 53, fenêtre d'arrivée 34–46 (80 %) ; la fondamentale se place dans 30..41, la quinte, la sixte et l'octave montent à 53 |
| `bass.passing` | `none_weight` 60, chromatique 28, degré de gamme 12 | WJazzD blues, passage au temps 4 (`passage_engine`) : chromatique 41 %, degré de gamme 19 %, note d'accord 40 % : rapport chromatique / degré de gamme 2,2 conservé (28 / 12) ; `none_weight` est **de métier** (un riff de boogie ne marche pas vers chaque accord) |
| Batterie : groupes de mesures | 6 groupes (timekeeper et figure) retenus : `ride:spang` (45 mesures), `ride:eighths` (28), `ride:quarters` (21), `hat:sixteenths` (18), `ride:sixteenths` (18), `hat:eighths` (8) | GMD, 340 mesures ternaires (voir « Batterie ») |
| Batterie : seuils par densité | densité 0 : timekeeper ≥ 0,5 ; densité 1 : + pédale, charleston ouverte, grosse caisse, caisse claire, cross-stick ≥ 0,5 ; densité 2 : + crash, grosse caisse et caisse claire dès 0,08 | probabilités par pas du groupe |
| Batterie : phrases | 24 (8 suites × 3 densités ; dont 6 « groove tenu quatre mesures » de métier) | GMD, suites de groupes alignées sur la première mesure du fichier, vues ≥ 2 fois |
| Batterie : relances | 3 groupes de fichiers `fill` × 2 densités × 2 étiquettes (`hat`, `ride`) : 1 mesure du pas 0 (15 fichiers), 1 mesure du pas 3 (5), 2 mesures du pas 9 (6, `part_end`) | GMD `fill` ternaires : premier coup au pas 0 dans 15 fichiers, au pas 2 dans 5 (ramené au pas 3, comme B8 ramène le pas 3 de seize au pas 4), aux pas 10 et 11 dans 6 (deux mesures, début au pas 9) ; 2 autres groupes de moins de 3 fichiers écartés |
| `fills_none_weight` | 21 (45 % de fins de phrase sans relance) | valeur de service : le GMD isole les relances |
| `performance.tracks.Percussion*.voices` | une entrée par note GM jouée, `on_beat` et `off_beat` en fraction de pas, sans `jitter` | GMD, médianes en ms de l'écart à la grille par rôle (groupe `4-4:ternary` des trois styles, pondérées par l'effectif), converties au tempo par défaut : un pas vaut ici `20 000 / BPM` ms |
| Niveau de chaque voix de batterie | niveaux de service des profils `gmd_*` (`RENDER_LEVEL`) | forme des accents du GMD gardée, niveaux absolus de service |
| `mix.tracks` | accords −16,2 dB, basse −14,8 dB, percussion −16,5 dB, maître +20 dB | LUFS intégrés en solo, graines 1 et 2, 48 mesures : accords −26,1, basse −25,0, batterie −25,7 (grosse caisse −31,3, reste −27,1), mix −21,5 (cibles de `b8/calibrer_mix.py`) |

Valeurs de métier, aucune mesure : les riffs de basse (boogie fondamentale-quinte-sixte en croches ternaires), les rythmes de comping et
leurs phrases, les cinq grilles, l'arc sur cinq chorus, le gratté de la guitare, la part de « aucune » du passage de la basse, tout
l'habillage sauf les bornes de tempo.

## Ce que le profil joue

- **Grille** : cinq boucles de douze mesures en mixolydien (tonique tirée par la session), relatives à la tonique, un accord par mesure :
  forme commune `I7` 4, `IV7` 2, `I7` 2, `V7`, `IV7`, `I7`, `V7` (poids 5) ; changement rapide `I7`, `IV7`, `I7` 2, `IV7` 2, `I7` 2, `V7`,
  `IV7`, `I7`, `V7` (5) ; retournement I-VI-ii-V `…`, `VI7`, `iim7`, `V7`, `I7`, `V7` (3) ; blues mineur `Im7` 4, `IVm7` 2, `Im7` 2, `bVI7`,
  `V7`, `Im7`, `V7` (2) ; forme qui se pose, mesures 11 et 12 sur `I7` (2). La graine en tire une par air ; training remplace par la sienne.
- **Basse** (basse électrique aux doigts 33 par défaut ; la session tire contrebasse 32, médiator 34 ou fretless 35) : neuf riffs de une ou
  deux mesures, `reference = "chord_root"` : ils suivent l'accord publié et arrivent sur sa basse à chaque changement. Croches ternaires,
  une paire longue-brève par temps (2 pas puis 1 pas) : boogie fondamentale-quinte-sixte-quinte (poids 5), montée vers la septième mineure,
  pompe à l'octave, marche en noires, fondamentale-quinte, deux mesures qui vont vers l'accord suivant, triolets de fondamentale (blues lent),
  tenue. Le plan de motifs est fixé par air ; une approche chromatique ou diatonique (rapport mesuré) précède une note de riff différente
  au temps 4.
- **Comping** (piano 0 par défaut ; la session tire guitare claire 27, guitare saturée 29, orgue 16, orgue rock 18, piano honky-tonk 3 ;
  notes guides et tensions de la qualité publiée) : douze rythmes d'une mesure en trois étiquettes (`sparse` : tenues et silence ; `medium` :
  une frappe par temps, contretemps, backbeat ; `dense` : paires longue-brève, backbeat accentué, **triolets de piano**) et dix-sept phrases
  de 2 et 4 mesures (suites de ces rythmes, une mise en place `part_end` par étiquette).
- **Batterie** (kit standard 128/0 par défaut ; la session tire room, jazz, power ou balais) : 18 patterns (6 groupes × 3 densités), 24
  phrases, 12 relances (3 × 2 densités × 2 étiquettes) ; ride en shuffle (`spang`) ou en croches, charleston en croches ou triolets ;
  grosse caisse, caisse claire (backbeat aux temps 2 et 4, pas 3 et 9) et cross-stick à la densité 1, notes fantômes probabilistes à la
  densité 2.
- **Arc** (cinq chorus par air, nombre connu) : exposition (comping `sparse` ou `medium`, batterie à la charleston, doux), deux chorus de
  corps (`medium` ou `dense`, ride ; un break du comping à la mesure 8 ou un break comping + basse à la dernière mesure), sommet (`dense`,
  plus fort, stop-time sur les deux dernières mesures une fois sur deux), retombée (`sparse`, charleston seule, stop-time sur le un des
  deux dernières mesures). Au-delà de cinq chorus : sommet puis corps et sommet en boucle.

## Batterie

Construite par `b8/gmd_batterie.py` (même méthode que les profils `gmd_*` de B8, étendue à la grille ternaire : `feel = "ternary"`, 12 pas,
départs de relance aux temps 1 à 4 de douze pas) : chaque coup est un (rôle, pas) dont la probabilité par pas dans un groupe de mesures
atteint le seuil de la densité (`LEVELS`, `PROB_FLOORS`) ; un coup entre le plancher et le seuil joue avec sa fréquence mesurée
(`probability`, 52 coups). Groupes : même geste de timekeeper exact, ou figure nette (noires, croches, doubles croches, « spang »).

Écarts aux seuils de B8, et pourquoi : **un seul fichier suffit** à un groupe (`min_files = 1`) et il compte au moins 8 mesures, sans plafond
de part d'un fichier. Les seuils de L2 (3 fichiers, 20 mesures, 60 % par fichier) ne retiennent **aucun** groupe sur ces cinq fichiers (essayé aussi à 2
fichiers, 20 mesures et 70 % par fichier : aucun groupe ne passe ; les figures `sparse` et `other`, les plus fréquentes, n'ont pas d'ossature). Chaque
groupe décrit donc surtout **un batteur** : `ride:spang` et `ride:quarters` sont le seul fichier `rock/shuffle` à 125 BPM, `hat:sixteenths` et
`ride:sixteenths` le seul fichier `blues/shuffle` à 72 BPM, `ride:eighths` et `hat:eighths` `funk/purdieshuffle` (plus 5 mesures de `rock/shuffle`). Deux
groupes de charleston du fichier `rock/shuffle` à 112 BPM (`0,2,4,5,7` : 21 mesures ; `2,4,5,7` : 16) sont **écartés** : ce sont des
figures de charleston en doubles croches ramenées au pas de triolet le plus proche, pas un shuffle. Les phrases sont mesurées sur des
fichiers de 64 à 104 mesures ; un fichier de 4 mesures (`blues/shuffle` à 134 BPM) ne donne rien.

Micro-timing (A6, `voices`) : médianes de l'écart à la grille en fraction de pas **au tempo par défaut (125 BPM : un pas = 160 ms)**, voix à
moins de 40 attaques écartées (cross-stick, cloche de ride, crash, tom médium). Les décalages mesurés en ms ne suivent donc pas exactement
le tempo tiré (à 72 BPM le même retard vaut 42 % de pas de moins ; à 134 BPM 7 % de plus).

| Voix | Note GM | Sur le temps (pas) | Hors temps (pas) |
|---|---:|---:|---:|
| kick | 36 | -0.072 | -0.086 |
| snare | 38 | -0.047 | -0.054 |
| hat_closed | 42 | -0.006 | -0.053 |
| hat_pedal | 44 | -0.113 | -0.139 |
| ride | 51 | -0.039 | +0.032 |
| tom_low | 43 | -0.061 | -0.137 |
| tom_high | 50 | -0.001 | — |

## Habillage par graine (`session.toml`, A2)

Une graine tire **un habillage** : un tirage indépendant par groupe, tous les groupes se combinent (9 groupes, 414 720 habillages possibles).
Le tirage est stable par graine, rejoué après un seek, et **l'hôte gagne** : `SetBpm` impose le tempo, `SetSwing` le swing,
`[fixed_tonality]` du profil de l'hôte la tonalité (la variante est alors ignorée) ; programmes, voicings, registre, énergie et arc n'ont
pas de commande. Pas de variante de swing : les cinq fichiers ternaires n'en montrent aucune pente avec le tempo.

| Groupe | Variantes (poids) | Ce que règle le groupe | D'où vient la valeur |
|---|---|---|---|
| `tempo` | lent 72–92 (2), medium 96–112 (3), vif 114–126 (3), rapide 128–134 (2) | tempo (un BPM entier tiré dans la plage) | GMD : extrêmes 72 et 134, médiane 125 ; **rien dans la source entre 72 et 112 que le fichier de blues lent** : les bandes `lent` et `medium` interpolent |
| `tonalite` | mi (3), la (3), sol (2), do (2), si♭ (2), ré (1), toutes mixolydiennes | tonique et mode du premier air (grille relative transposée) | métier : tonalités courantes du blues |
| `comping` | piano 0 (3), guitare claire 27 (2), guitare saturée 29 (2), orgue 16 (2), orgue rock 18 (1), piano honky-tonk 3 (1) | programme des accords | métier |
| `basse` | électrique aux doigts 33 (3), contrebasse 32 (3), médiator 34 (1), fretless 35 (1) | programme de la basse | métier |
| `kit` | standard 0 (3), room 8 (2), jazz 32 (2), power 16 (1), balais 40 (1) | kit de la banque 128 | métier ; le GMD est joué sur un kit électronique Roland TD-11 |
| `voicing` | notes_guides (3), shell (2), drop_2 (1), rootless (1) | famille de voicings du comping | métier |
| `registre` | normal (4), haut (1), grave (1) | octave du comping | métier |
| `energie` | doux (1), normal (2), fort (1) | multiplicateurs d'intensité et de densité de base | service |
| `arc` | defaut (3), orchestre (2), direct (1), montee (1) | `[arc]` du profil ou arc nommé de `session.toml` | service (aucun corpus ne donne l'arc d'un morceau) |

Les arcs nommés (`orchestre` : le comping et la basse changent de programme de chorus en chorus ; `direct` : pas d'exposition ; `montee` :
un cran par chorus) sont écrits dans `session.toml`, mêmes valeurs de service que `arc.toml`. Tous les instruments sont des programmes
General MIDI de GeneralUser GS (banque 0) et des kits de la banque 128, vérifiés dans la banque ; aucune zone Odin (D1).

## Contrat et preuves

`harmonium_core/tests/gmd_blues_shuffle_profile_tests.rs` (flux de `MeasureSnapshot`, 11 tests) : grille par défaut = une des cinq boucles
écrites (≥ 3 tirées sur 24 graines), tierce et septième de chaque accord, reprise au chorus suivant ; douze pas, rapport de swing 0,664,
tempo dans 72..134, changements d'accord sur un temps ; basse sur l'accord publié (fondamentale, quinte, sixte, septième mineure ou
approche de l'accord suivant), dans la fenêtre 28..53, arrivée sur la basse publiée, paires longue-brève ; comping dans l'accord publié et
dans la tessiture ; batterie : timekeeper seul à la densité 0, coups croissants avec la densité, caisse claire dans la plupart des mesures ;
relances aux toms seulement hors densité 0 ; arc de cinq chorus (exposition et retombée plus légères que le corps, ride du corps, stop-time) ;
mêmes graine, même sortie ; ≥ 14 comping, ≥ 12 basses, ≥ 12 batteries distincts sur 16 graines ; ≥ 8 habillages sur 16 graines dans les
bornes (tempo, tonalités mixolydiennes écrites) ; l'hôte garde la tonalité et le tempo. Variété, distance au corpus :
`specs/019-bibliotheque-accompagnements/b6-blues-shuffle.md`.

## Limites

- **Batterie : cinq fichiers, quatre batteurs, des composites.** Chaque groupe est surtout un batteur (voir « Batterie »). Les patterns sont
  un agrégat de probabilités par pas, pas une mesure ; ils ne sont pas validés à l'écoute (D5) mais mesurés. Pas de 12/8 lent écrit en
  vraie grille de 24 pas (voir manques) : le blues lent est un shuffle de 12 pas à 72–92 BPM (un seul fichier de la source, 72 BPM).
- **Relances** : 29 fichiers dont 26 d'un seul batteur (`funk/purdieshuffle`) ; les relances sont des remplissages de caisse claire et de toms de ce batteur.
  `fills_none_weight` est un réglage.
- **Basse** : WJazzD ne dit rien des croches ni des silences : les riffs sont écrits, sans tierce (la tierce dépend de la qualité). La part
  d'approche « aucune » est de métier ; les rapports chromatique / degré viennent d'une **marche** de jazz, pas d'un boogie. Le plan de
  motifs est fixé par air : la basse change peu d'un chorus à l'autre (6 % de mesures, voir la preuve).
- **Comping** : aucune source ; le « piano triolets » et le backbeat sont des idiomes connus, pas des mesures.
- **Swing** : un rapport unique 0,664, tous tempos ; l'écart entre fichiers (0,627 à 0,696) n'est pas reproduit. Le tempo tiré ne change pas le
  rapport.
- **Micro-timing** : médianes par voix, sans dispersion (un `jitter` de batterie partage son tirage avec la basse à notes de passage).
- **Mixage** : réglé au LUFS en solo (graines 1 et 2), aucune écoute.
- **Habillage** : groupes indépendants, aucun lien entre kit, tempo et instruments (des balais à 134 BPM peuvent tomber) ; tout est de
  métier sauf les bornes de tempo.

## Licence

Groove MIDI Dataset v1.0.0 : © Google LLC, CC BY 4.0 ; Weimar Jazz Database : ODbL 1.0 / DbCL 1.0 (mentions complètes dans `[meta] corpus`).
Les données dérivées sont des agrégats ; le reste du profil est sous la licence du dépôt.

# B3 — Profil `gmd_latin_brazilian_bossa` (bossa nova)

Date : 2026-11-03. Branche `wquintal/b3-latin`, partie de `main` à `TUNING_VERSION` 26, **inchangé** : aucune ligne de moteur, aucun mécanisme ajouté ; un
dossier de profil neuf, un test de contrat, cinq scripts de reproduction (plus le script partagé `mesures_latin.py`, non modifié). Les autres profils ne sont pas touchés. Dumps `harmonium_determinism`, `profiles/README.md` et
`plan.md` : à la charge de l'orchestrateur (hors du périmètre de cette tâche).

> **Limite de la source.** Style GMD `latin/brazilian-bossa` : **2 fichiers `beat`, 1 batteur, 631 mesures, 127 BPM**, fiabilité **« trop peu »** (L2). Le profil décrit un
> batteur, pas la bossa nova ; le GMD n'a aucune percussion latine ; WJazzD `LATIN` ne distingue pas la bossa et n'observe aucune croche de basse.

## Ce qui est livré

| Profil | Source | Chorus | Tempo | Instruments (défaut, puis tirés par la graine) | Contenu |
|---|---|---|---|---|---|
| `gmd_latin_brazilian_bossa` | batterie : GMD `latin/brazilian-bossa` (« trop peu ») ; basse : WJazzD `LATIN` (« assez ») | 16 mesures, 5 chorus par air | 127 (104 à 180), droit 0,5 | guitare nylon 24, contrebasse 32, kit jazz 128/32 ; session : 5 comping, 3 basses, 4 kits, 5 voicings | 5 boucles de grille, 3 gabarits de basse anticipée, 12 rythmes de comping et 16 phrases, 9 patterns, 30 phrases et 8 relances de batterie, arc, 9 groupes de session |

Fichiers : `profiles/gmd_latin_brazilian_bossa/` (`profile.toml`, `percussion.toml` généré, `arc.toml`, `dynamics.toml`, `set.toml`, `arrangement.toml`, `cells.toml`,
`session.toml`, `README.md`), `harmonium_core/tests/gmd_latin_brazilian_bossa_profile_tests.rs` (12 tests), et `specs/019-bibliotheque-accompagnements/b3/` :

- `gmd_latin_brazilian_bossa_generer.py` : régénère `percussion.toml` (et `gmd_latin_brazilian_bossa_mesures.json`, effectifs et micro-timing) ;
- `gmd_latin_brazilian_bossa_mix.py` : LUFS intégrés en solo par piste ; `gmd_latin_brazilian_bossa_distance.py` : distance au GMD ; `gmd_latin_brazilian_bossa_variete.py` : tableau de
  variété ; `gmd_latin_brazilian_bossa_parite.py` : le GMD montre-t-il un cycle de deux mesures ? ;
- `mesures_latin.py` (script partagé de B3, non modifié) : habillages et basse contre WJazzD.

## Choix de conception

- **Nom par source précise** (principe VII), pas d'alias. Une partie de seize mesures, cinq chorus par air (nombre connu : la retombée joue). Training remplace la boucle.
- **Cinq boucles de grille** relatives à la tonique, tirées par la graine (formes communes écrites, D4 aucune réharmonisation, aucune œuvre) : tourne I–vi–ii–V, cycle ii–V–I, tourne à
  la blanche, huit mesures jouées deux fois, vamp ii–V–I–IV. Rythme harmonique de WJazzD LATIN : 87 % de mesures à un accord, changement au temps 1 (85 %) ou 3 (15 %) ; rendu : 86 % de
  mesures à un accord, 13 % de changements au temps 3.
- **Pas de `[fixed_tonality]`** : la tonalité se tire par la session (do, ré, fa, sol, si♭, mi♭, ionien). Le mode tiré est une étiquette : la boucle écrite ne dépend pas de la tonalité tirée.
  `dynamics.toml` garde la bascule ionien/éolien que le chargeur exige, sans effet (valence neutre 0,5 entre `dark_below` 0,1 et `bright_above` 0,9).
- **Basse** (A4, gabarits) : fondamentale au un, quinte au « et » du deux, fondamentale au trois, et au pas 14 une note courte de la basse de l'accord **suivant** (`pickup`
  `when = "bar_end"`). Trois gabarits (poids 5, 3, 2). Le deuxième joue la quinte en `fifth_if_held` : une quinte au temps 3 d'un accord qui change à la blanche ne tombait pas sur la
  basse publiée (le test de contrat l'a révélé). `passing` : parts de WJazzD LATIN (aucune 59,9 %, chromatique 21,6 %, diatonique 18,5 %), `before = [2]` pour que l'approche tombe sur le
  temps 4 devant l'anticipation du pas 14. **Ce qui est mesuré** : registre, parts d'approche, rythme harmonique, longueurs de chorus, tempos. **Ce qui est de métier** : gabarits, anticipation, croches.
- **Comping** (A1, A5) : guitare nylon, notes guides, `color` et `register_drift`, gratté de quelques millisecondes. Douze rythmes d'une mesure (`sparse`, `medium`, `dense`) dont une « clave » de
  la bossa de deux mesures (« trois » aux pas 0, 6, 12, « deux » aux pas 4, 8, puis l'anticipation du pas 14), seize phrases de 2 et 4 mesures. **Ce cycle est de métier, pas une clave
  mesurée ni contrainte** : pas de `[clave]` ; le GMD n'a aucun cycle de deux mesures commun à ses deux fichiers (`gmd_latin_brazilian_bossa_parite.py`).
- **Batterie** (A1, A6) : voir « Construction de la batterie ». Cross-stick, caisse claire fantôme et grosse caisse fantôme en coups `probability`, micro-timing par voix.
- **Arc** (A3) : exposition au charleston en croches, corps au ride, sommet avec stop-time une fois sur deux, retombée avec stop-time ; arc nommé « orchestre » (A2) qui change aussi de
  programmes. Valeurs de métier.
- **Habillage par graine** (A2, D6) : neuf groupes, 33 variantes, 64 800 habillages possibles. Tempos : **les trois plages sont celles de la source** (lent 104–112 = p10 de WJazzD ; médium 122–132
  autour des 127 du GMD ; vif 152–180 = de la médiane de WJazzD à la borne de la classe « juste »), séparées de 10 et 20 BPM ; aucune n'étend la mesure. Le GMD n'a qu'un tempo : **faible étendue
  documentée**. Deux groupes ne règlent jamais le même champ.
- **Mixage** : réglé sur le LUFS en solo de chaque piste (pas à l'oreille).

### Construction de la batterie (`gmd_latin_brazilian_bossa_generer.py`)

Le GMD donne des probabilités par pas, pas des phrases (L2 § 6) ; rien d'un enregistrement n'est copié.

1. **Groupes de mesures** : timekeeper et figure par mesure (`gmd_styles.timekeeper_of`, `figure_of`). Trois groupes cohérents : ride en croches (244 mesures), ride en noires plus (46), charleston
   fermé en croches (21). Les seuils de L2 (3 fichiers, 2 batteurs) ne laissent rien : ceux d'ici sont 20 mesures et 2 fichiers lus (un seul batteur). Les figures `other` et `sparse` du ride (182 mesures
   hétérogènes) sont écartées comme en B8.
2. **Patterns** (3 groupes × 3 densités = 9) : densité 0 : timekeeper ≥ 0,5, grosse caisse et pédale ≥ 0,75 ; densité 1 : tout ≥ 0,5 plus grosse caisse fantôme (pas 1 et 9, 29–37 %) et cross-stick
   probabilistes (≥ 15 %, à leur probabilité mesurée dans le groupe) ; densité 2 : plus caisse claire fantôme, tom médium (probabilistes) et **shaker GM de métier**. Poids = effectif de mesures ;
   étiquette d'arc = `ride` ou `hat`.
3. **Phrases** : suites de groupes sur 2 et 4 mesures (fenêtres alignées sur la première mesure du fichier) vues ≥ 2 fois si elles changent de groupe, ≥ 3 sinon : 7 suites mesurées × 3 densités ;
   plus 9 phrases « groove tenu » 2 et 4 mesures (règle de métier) pour les groupes et densités qu'aucune suite n'atteint.
4. **Relances** : le GMD n'a **aucun fichier `fill`** bossa ; 4 relances de métier par étiquette (cross-stick et caisse claire / toms du pas 8 ou 12), densités 1 et 2 ; `fills_none_weight` 8.
5. **Niveaux** : le TD-11 ne se transpose pas au kit GM ; chaque voix a un niveau de service, la forme des accents (vélocité relative du groupe, bornée à 0,55–1,4) est gardée. Calés sur le LUFS (voir « Mixage »).
6. **Voix latines exclues** : le shaker GM (82) est retiré de toute comparaison au GMD (distance) ; il n'a aucun équivalent mesuré.

## Preuves

Toutes depuis `~/Project/bascanada/harmonium`, binaire du worktree (`<cli>` = `target/release/harmonium-cli`), hors ligne, sans lire d'audio.

### Habillages sur 16 graines

`python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py habillages --cli <cli> --profile <worktree>/profiles/gmd_latin_brazilian_bossa --seeds 16 --bars 48`

| Mesure sur 16 graines | Valeur | Possibles |
|---|---:|---:|
| habillages distincts (tempo, tonalité, programmes, voicing, registre, énergie, arc) | **16** | 64 800 |
| tempos : plages tirées / grappes à ± 8 BPM | 3 plages (104–112, 122–132, 152–180) / 5 | 3 / — |
| tonalités | 6 (do, ré, fa, sol, si♭, mi♭) | 6 |
| triplets de programmes (comping, basse, kit) | 15 | 60 |
| comping / basse / kit | {4, 24, 25, 26} / {32, 33, 35} / {0, 8, 32, 40} | idem |
| variantes distinctes par groupe | tempo 3, tonalité 6, comping 4, basse 3, kit 4, voicing 5, registre 2, énergie 3, arc 2 | 3, 6, 5, 3, 4, 5, 2, 3, 2 |

Test de contrat : ≥ 8 habillages distincts, 3 tempos, ≥ 4 tonalités, ≥ 3 comping, ≥ 2 basses, ≥ 2 kits, ≥ 3 voicings (même lecture dans `MeasureSnapshot.session` et `instruments`).

### Variété (`harmonium-lab seed-variety --chorus 4`, 16 graines)

`harmonium-lab seed-variety --profile <dossier> --seeds 16 --chorus 4 --engine-dir ~/Project/bascanada/harmonium --engine-bin <cli> --label B3 --work-dir /tmp/harmonium-019/b3/bossa/var/w_<nom> -o <nom>.json`
sur le profil livré, sur une copie **sans `session.toml`** (« avant ») et sur `wjazzd_swing` (V0, grille par défaut de 44 mesures) **remesuré avec le même binaire** ; résumé par
`gmd_latin_brazilian_bossa_variete.py livre.json avant.json v0.json`. Cellule : mesures changées d'un chorus au suivant · suites distinctes sur 16 graines · distance graduée entre graines.

| Couche | Livré (avec session) | Sans habillage (avant) | V0 `wjazzd_swing` |
|---|---:|---:|---:|
| accords | 0 % · 5 · 72 % | 0 % · 5 · 73 % | 0 % · 1 · 0 % |
| comping : rythme | 84 % · 16 · 38 % | 90 % · 16 · 40 % | 80 % · 16 · 68 % |
| comping : voicings | 96 % · 16 · **98 %** | 100 % · 16 · 90 % | 74 % · 16 · 46 % |
| basse | 76 % · 16 · 92 % | 78 % · 16 · 87 % | 92 % · 16 · 59 % |
| batterie | 100 % · 16 · 27 % | 100 % · 16 · 26 % | 0 % · 11 · 19 % |
| **orchestration** (programmes) | 0 % · **11** · **44 %** | 0 % · 1 · 0 % | 0 % · 1 · 0 % |
| accompagnement (toutes couches) | 100 % · 16 · **65 %** | 100 % · 16 · 63 % | 99 % · 16 · 38 % |

Lecture : la session fait passer l'orchestration de 1 à **11 suites de programmes distinctes sur 16** et de 0 à 44 % de distance entre graines (V0 : 1, 0 %) ; l'accompagnement entier gagne 2 points
de distance (63 → 65 %) et la distance des voicings 8 points. Les autres couches ne bougent pas : le tempo, la tonalité et le registre ne se lisent pas dans ces signatures (accords relatifs à la
tonique, sans le tempo). La batterie d'un chorus à l'autre change dans 100 % des mesures contre 0 % pour V0. Le comping rythme est moins distant entre graines que V0 (38 contre 68 %) : douze rythmes
de trois étiquettes et des phrases de 2 et 4 mesures contre la banque libre de V0. Les 8 points de distance des voicings (90 → 98 %) et les 5 de la basse (87 → 92 %) viennent vraisemblablement de la tonalité tirée
(hauteurs) et, pour les voicings, de la famille et du registre [INFERENCE : non isolé groupe par groupe].

### Basse contre WJazzD `LATIN`

`python3 b3/mesures_latin.py basse --cli <cli> --profile <dossier> --seeds 16 --bars 48 --reference ~/Project/bascanada/harmonium_lab/references/wjazzd-latin/measures-bass.json` : seul ce que la source mesure,
échantillonné au temps (hauteur de basse aux temps 1 à 4), 764 mesures, 16 graines.

| Mesure | Profil | WJazzD `LATIN` (effectif · fiabilité) |
|---|---:|---|
| registre de basse, p1 / p50 / p99 | 34 / 40 / 45 | 31 / 40 / 50 (6 556 temps, 23 solos · assez) |
| étendue de basse | 34–45 (aux temps) | 28–53 |
| arrivée sur la basse de l'accord au changement | 100 % (798 changements) | 57 % [46–68 %] (1 420 changements, 22 solos) : **non reproduite**, un gabarit arrive toujours sur la basse publiée |
| approche à un demi-ton avant un changement à la barre | 22 % (691 approches comparées) | chromatique 21,6 % [17–26 %] (1 205 événements) |
| approche à deux demi-tons | 43 % | diatonique 18,5 % [11–29 %] : « deux demi-tons » n'est pas un degré diatonique, aucun score agrégé |
| changement d'accord au temps 3 | 13,3 % | 15,1 % |
| mesures à un seul accord | 86,0 % | 86,9 % [79–93 %] |
| tempo min–max rendu | 104–176 BPM | p10 104, p50 152,7, p90 227,5 |

Registre : la fondamentale et la quinte se placent dans une fenêtre de douze demi-tons (34 à 45) : le p1 et le p99 de WJazzD (31, 50) ne sont pas atteints. Les croches de basse ne sont pas comparables.

### Distance au GMD (batterie)

`make style-distance` mesure contre de l'audio et des transcriptions : le GMD est du MIDI de batterie sans audio. Modèle de `b8/distance_gmd.py` : Bray-Curtis des fréquences d'attaque par pas (16
pas) contre **toutes** les mesures `beat` 4/4 droites du style (631), 16 graines × 48 mesures, densité neutre. **Voix latines exclues : le shaker GM (82) est retiré du rendu avant la mesure** (il n'a
aucun équivalent dans le GMD ; les toms restent dans « un coup quelconque »). Copie non calibrée : le même profil sans arc ni session, `percussion.toml` de `wjazzd_swing`.
Commande (venv du labo) : `.venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa_distance.py --out d.json`. Distance (0 = identique) :

| Métrique | Profil livré | Sans arc ni session | Copie non calibrée |
|---|---:|---:|---:|
| un coup quelconque par pas | 0,105 | 0,105 | 0,312 |
| grosse caisse | 0,076 | 0,077 | 1,000 |
| caisse claire (cross-stick compris) | **0,471** | 0,428 | 1,000 |
| charleston (fermé, pédale, ouvert) | 0,162 | 0,167 | 0,566 |
| ride | 0,072 | 0,110 | 0,267 |
| **moyenne des quatre premières** | **0,203** | 0,194 | **0,720** |

La calibration rapproche la batterie du GMD de **0,52** (moyenne 0,720 → 0,203) ; l'arc coûte 0,9 point de moyenne (exposition et retombée au charleston, sans ride). **Le point faible est la caisse
claire (0,47)** : le batteur du GMD joue des coups fantômes sur presque tous les pas (15 à 41 %), le profil ne les rend qu'en densité 2 et à la moitié des pas ; la métrique ne voit que la présence par pas,
pas la vélocité (40 sur le TD-11). Un seul batteur : cette distance se mesure à lui seul.

### Mixage (LUFS intégrés en solo, `--solo-track`, `ffmpeg ebur128`, 48 mesures)

`python3 b3/gmd_latin_brazilian_bossa_mix.py --profile <dossier> --cli <cli> --seeds 1 2 --bars 48`. Cibles de service : accords −26, basse −25, batterie −25,5, mix ≈ −21 à −22 ; crête vraie ≤ −1 dBFS.
Niveaux réglés par mesure : accords −16,3 dB, basse −18,7 dB, batterie −10,0 dB (le niveau de `PercussionHigh` gagne sur le canal 9 ; la grosse caisse se règle par sa vélocité).

| Piste | Graines 1 et 2 (avec session) | Graines 3 à 8 | Sans session, graines 1 et 2 |
|---|---:|---:|---:|
| Chords | −25,5 | −25,9 (−30,7 à −22,8) | −26,9 |
| Bass | −24,8 | −25,1 (−26,6 à −24,0) | −25,9 |
| Batterie (somme en puissance) | −26,0 | −25,5 | −25,9 |
| dont grosse caisse (`PercussionLow`) / reste du kit (`PercussionHigh`) | −30,6 / −27,9 | −30,4 / −27,2 | −30,7 / −27,6 |
| Mix | −20,9 | −20,9 (−21,9 à −19,8) | −21,9 |
| Crête vraie du mix (dBFS) | −2,7 | −1,3 | −2,1 |

L'écart d'une graine à l'autre est celui des timbres tirés (guitare nylon, acier, jazz, piano, Tine ; kit) : les niveaux sont calés sur la moyenne des graines 1 à 8, pas sur un timbre. Calage : sur les instruments
par défaut (sans session), accords −27,6 LUFS au niveau −17,0 dB → −15,4 dB (−26,2), basse −25,6 à −18,5 dB → −17,9 dB (−25,1), batterie −30,4 LUFS à −19,0 dB → voix plus fortes et −9,6 dB (kit −25,4, mix −21,2) ;
puis −0,9 / −0,8 / −0,4 dB sur la moyenne des graines 1 à 8 avec session pour que le mix ne dépasse pas −1 dBFS de crête vraie.

## Contrat (`harmonium_core/tests/gmd_latin_brazilian_bossa_profile_tests.rs`)

Douze tests sur le flux de `MeasureSnapshot` (graines 1, 2, 3, 17 et 1 à 16 ; 40 graines pour l'arc « orchestre »).

| Contrat | Test |
|---|---|
| grille par défaut = une des 5 boucles écrites, à la tonique tirée (`position.key`, mode ionien), reprise à l'identique à chaque chorus ; ≥ 3 grilles et ≥ 4 tonalités sur 16 graines | `la_grille_par_defaut_est_une_des_boucles_ecrites_a_la_tonique_tiree_…` |
| la basse arrive sur la basse publiée à chaque changement (hors break), dans la fenêtre 34..45, étendue 28..53 | `la_basse_arrive_sur_la_basse_publiee_a_chaque_changement_…` |
| la basse ne joue que fondamentale, quinte, anticipation du pas 14 (basse de l'accord suivant, toutes les mesures hors break et dernière) ou approche à 1–2 demi-tons ; > 100 quintes | `la_basse_joue_fondamentale_quinte_anticipation_…` |
| le comping ne joue que des notes de l'accord publié (celui de la mesure suivante pour une attaque finissant sur la barre) plus les tensions naturelles de la couleur ; sans fondamentale sauf drop 2 et structure supérieure ; tessiture (registre « grave » : −12) ; ≥ 20 attaques anticipées | `le_comping_ne_joue_que_des_notes_de_l_accord_publie_…` |
| cycle de deux mesures : « trois » (pas 0, 6, 12) sur les mesures impaires du chorus, « deux » (4, 8, 14) sur les paires, « trois » suivi de « deux » ≥ 70 % | `la_bossa_joue_son_cycle_de_deux_mesures_trois_puis_deux` |
| batterie dans le kit (8 notes, densité 1 et 2), grosse caisse [0, 6, 8, 14] et pédale 4, 12 dans ≥ 70 % des mesures, cross-stick, shaker GM sur `PercussionHigh` (densité 2), grosse caisse sur `PercussionLow` | `la_batterie_reste_dans_le_kit_pose_la_grosse_caisse_…` |
| charleston (étiquette `hat`) à l'exposition et à la retombée sans ride ; ride au sommet sans charleston | `le_charleston_porte_l_exposition_et_la_retombee_…` |
| feel droit en 16 pas, swing 0,5, tempo = BPM entier dans la plage de la variante tirée (3 plages, toutes les intensités), plage du profil 104–180 | `le_feel_est_droit_en_seize_pas_et_le_tempo_reste_dans_la_plage_…` |
| exposition et retombée plus légères que le sommet, stop-time de la retombée (accords et basse au pas 0 seulement, batterie continue) | `l_arc_creuse_l_ecart_entre_l_exposition_le_sommet_et_la_retombee` |
| arc « orchestre » : balais à l'exposition, piano électrique au sommet, nylon et balais à la retombée, basse inchangée | `l_arc_orchestre_change_les_instruments_a_chaque_frontiere_de_chorus` |
| ≥ 8 habillages distincts sur 16 graines (tempo, tonalité, programmes, voicing, registre, énergie, arc, lus dans `session` et `instruments`) | `des_graines_differentes_donnent_des_habillages_differents` |
| même graine, même sortie ; ≥ 14 comping, ≥ 14 batteries, ≥ 12 basses distincts sur 16 graines | `la_graine_change_le_comping_la_batterie_et_la_basse_…` |

Mutations vérifiées (chacune fait échouer au moins un test, puis annulée) :

| # | Mutation | Test qui échoue |
|---|---|---|
| M1 | `default_swing_ratio` 0,62 | feel et tempo |
| M2 | percussion de l'exposition `ride` au lieu de `hat` | charleston / ride |
| M3 | sans `pickup` des gabarits de basse | basse (anticipation) |
| M4 | sans `anticipate` | comping (attaques anticipées) |
| M5 | tempo médium 122–140 | feel et tempo |
| M6 | retombée sans stop-time | arc |
| M7 | phrases `[4, 5]` inversées en `[5, 4]` | cycle de deux mesures |
| M8 | `fifth` au lieu de `fifth_if_held` (le défaut trouvé avant livraison) | basse sur les changements |
| M9 | session sans groupe de tonalité | habillages, grille |
| M10 | grosse caisse routée sur `PercussionHigh` | batterie |
| M11 | `ii` remplacé par `iii` dans la boucle ii–V–I | grille |
| M12 | shaker sur la note 83 (hors du kit) | batterie (densité 2) |
| M13 | basse 30..41 au lieu de 34..45 | basse sur les changements |
| M14 | sans `session.toml` | feel et tempo, habillages, grille, arc « orchestre » |
| M16 | retombée dense, ride, intensité 1,3 | charleston, arc |
| M17 | exposition dense et forte | arc |
| M18 | sans cross-stick | batterie |

Dix mutations de départ (M1 à M8, M12, M13) ne s'appliquaient pas (`sed -i` BSD sans suffixe) : elles ont été rejouées avec un remplaçant Python qui exige un remplacement effectif. M12 passait avant que le test
joue la densité 2 (le shaker n'y est que là) : le test a été complété. M15 (retombée plus forte sans changer les étiquettes) ne faisait rien échouer : remplacée par M16.
`cargo +nightly fmt -p harmonium_core -- --check` propre sur ce fichier ; `cargo clippy -p harmonium_core --test gmd_latin_brazilian_bossa_profile_tests` : aucun avertissement.

## Ce qui ne s'exprime pas dans le schéma ou le moteur (A7, rien n'est contourné)

1. **Anticipation liée** : le gabarit de basse réattaque la note du pas 14 au temps 1 ; pas de `tie` entre `pickup` et arrivée.
2. **Approche limitée à un changement à la barre** : `passing` n'a que `next_target_differs` et `chord_change_inside_bar` ; WJazzD mesure l'approche avant un changement à la barre (21,6 % chromatique) et
   le moteur l'applique à toute note qui change de hauteur.
3. **Micro-timing** : l'écart par voix est borné à ±0,2 pas (la pédale hors temps, +0,225 dans le GMD, ne passe pas) et la dispersion d'une voix demande un `jitter` de piste (qui déplace la basse à notes de passage).
4. **Groupes de session indépendants** : balais à 180 BPM, piano électrique et contrebasse à 104 BPM ne s'excluent pas ; aucun groupe n'en conditionne un autre.
5. **Grille et mode tirés sans lien** : la boucle de grille ne dépend pas de la tonalité tirée ; le mode n'est qu'une étiquette (ionien).
6. **Programmes à la barre** (A2) : les attaques anticipées du pas 14 de la dernière mesure d'un chorus sonnent encore avec l'instrument précédent.

## Limites

- **Un batteur, deux fichiers, kit électronique** : un idiome de batteur, pas la bossa nova ; aucun fichier `fill`, aucune percussion latine ; la caisse claire du GMD est un nuage de coups fantômes
  que le profil rend à moitié.
- **Basse** : WJazzD `LATIN` est du hard bop et du post-bop sur rythme latin, sans croche : les gabarits, l'anticipation et l'arrivée à 100 % sont de métier. 22 solos pour les parts « au changement » : intervalles de confiance larges.
- **Tempos** : un seul tempo dans le GMD (127) ; plage lente = p10 de WJazzD, classe « trop peu » (un solo) ; rien au-delà de 180.
- **Aucun mode mineur** ; la tonalité tirée ne change que la hauteur de la grille.
- **Tout est jugé par des chiffres (D5)** : aucune écoute ; le propriétaire écoute dans le studio.

## Liens d'écoute (à ouvrir par l'orchestrateur)

Deux graines dont les habillages diffèrent sur les **neuf** groupes (même lecture que le tableau de `mesures_latin.py habillages`, graines 1 et 2) :

- `harmonium://open?profile=gmd_latin_brazilian_bossa&seed=1` : médium 125 BPM, ré, guitare nylon, contrebasse, balais, voicing sans fondamentale, comping une octave plus bas, énergie douce, **arc « orchestre »** (exposition
  aux balais, sommet au piano électrique, retombée balais et nylon).
- `harmonium://open?profile=gmd_latin_brazilian_bossa&seed=2` : lent 110 BPM, do, piano électrique « Tine », basse fretless, kit room, notes guides, registre normal, énergie pleine, arc `arc.toml`.

## Reproduire

```sh
cd ~/Project/bascanada/harmonium_lab
.venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa_generer.py --out <worktree>/profiles/gmd_latin_brazilian_bossa
.venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa_parite.py
.venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa_distance.py --out d.json
cd ~/Project/bascanada/harmonium    # CLI : <worktree>/target/release/harmonium-cli
python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py habillages --cli <cli> --profile <worktree>/profiles/gmd_latin_brazilian_bossa --seeds 16 --bars 48
python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py basse --cli <cli> --profile <worktree>/profiles/gmd_latin_brazilian_bossa --seeds 16 --bars 48 \
  --reference ~/Project/bascanada/harmonium_lab/references/wjazzd-latin/measures-bass.json
~/Project/bascanada/harmonium_lab/.venv/bin/harmonium-lab seed-variety --profile <dossier> --seeds 16 --chorus 4 --engine-dir ~/Project/bascanada/harmonium --engine-bin <cli> --label B3 --work-dir <tmp> -o <nom>.json
python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa_variete.py livre.json avant.json v0.json
python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa_mix.py --profile <dossier> --cli <cli> --seeds 1 2 --bars 48
cargo test -p harmonium_core --test gmd_latin_brazilian_bossa_profile_tests
```

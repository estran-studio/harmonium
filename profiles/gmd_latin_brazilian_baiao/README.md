# `gmd_latin_brazilian_baiao` — baião brésilien sous une grille d'accords

Accompagnement de baião (Nordeste du Brésil) sous une grille d'accords, sans Odin : tout sonne avec GeneralUser GS
(accordéon, guitare nylon ou piano ; contrebasse ou basse aux doigts ; kit GM). Phase 019, tâche B3, profil neuf
(aucun bump de `TUNING_VERSION`, aucun code moteur). La batterie du kit, le tempo et le feel viennent du Groove MIDI
Dataset ; la basse est calibrée sur WJazzD `LATIN` pour ce qui s'y mesure ; **tout le reste est une règle de métier
(idiome traditionnel), dite telle ci-dessous** : aucun corpus ouvert ne mesure la zabumba écrite, le triangle,
l'agogô, la basse syncopée, le comping, les vamps modaux, l'arc ni l'habillage.

**Nom (principe VII).** Le profil nomme la source qui définit son style : le style `latin/brazilian-baiao` du GMD, soit
**3 fichiers `beat`, 200 mesures, 2 batteurs, fiabilité « juste »** (pas « assez » : le seuil « assez » demande 8 fichiers
et 3 batteurs) et 29 fichiers `fill` d'un seul batteur. Ce n'est pas « le baião » : deux batteurs, un kit électronique
Roland TD-11, des fichiers à 95, 95 et 110 BPM. **Limite écrite : livrable parce que la source est « juste », pas
parce qu'elle représente le genre.**

## Mentions de licence

- **Groove MIDI Dataset v1.0.0** (Google LLC) — CC BY 4.0, <https://creativecommons.org/licenses/by/4.0/> : Gillick,
  Roberts, Engel, Eck, Bamman, *Learning to Groove with Inverse Sequence Transformations*, ICML 2019 ; Groove MIDI
  Dataset v1.0.0, Google LLC, CC BY 4.0. Probabilités par pas agrégées, groupes de mesures, phrases et relances ;
  aucune mesure d'enregistrement. SHA256 de l'archive (vérifié par le labo avant toute lecture) :
  `651cbc524ffb891be1a3e46d89dc82a1cecb09a57c748c7b45b844c4841dcc1e`.
- **Weimar Jazz Database** (WJazzD 2.1) : « Contains information from the Weimar Jazz Database (Jazzomat Research Project,
  HfM Weimar), which is made available here under the Open Database License (ODbL); individual contents under the
  Database Contents License (DbCL 1.0). » Statistiques agrégées du sous-ensemble `rhythmfeel = LATIN` seulement
  (registre, approches avant un changement d'accord, rythme harmonique) ; aucune note de solo, aucune suite de notes.
- Aucune autre source ; aucune source NC ni SA. Les grilles par défaut sont des formes communes écrites pour ce profil
  (aucune œuvre nommée, aucune réharmonisation : D4). Provenance chiffrée :
  `harmonium_lab/references/l2-ingestion.md`, `references/gmd-latin-brazilian-baiao/`, `references/wjazzd-latin/`.

## Ce que le profil joue

| Couche | Contenu | Origine |
|---|---|---|
| Grille | 4 vamps modaux de 16 mesures (I7, bVII, IV), relatifs à la tonique, mixolydiens | **métier** |
| Batterie | 9 patterns (3 groupes x 3 densités), 18 phrases, 1 relance (x 3 étiquettes x 2 densités) ; zabumba (grosse caisse), charleston à la pédale, ride ou charleston fermée ou caisse claire en doubles croches (`caixa`) | **GMD** (mesuré) |
| Triangle (80 muet, 81 ouvert), agogô (67, 68) | écrits sur la piste haute par-dessus les patterns | **métier** |
| Basse | 6 motifs d'une mesure : fondamentale sur le un et le trois, quinte en contretemps ; notes de passage `passing` | motifs : **métier** ; registre et approches : **WJazzD `LATIN`** |
| Comping | accordéon 21, guitare nylon 24 ou piano 0 ; 9 rythmes (3 étiquettes), 9 phrases de 2 et 4 mesures | **métier** |
| Arc | 5 chorus de 16 mesures : exposition, corps (2), sommet, retombée ; stop-time | **métier** |
| Habillage | tempo, tonalité, comping, basse, kit, voicings, registre, énergie, arc : 28 variantes en 9 groupes | **métier** (tempo : bornes de la source) |
| Feel | droit en 16 pas, `default_swing_ratio = 0.5`, pas de `[swing]` | **GMD** |

Pas de `[clave]` : le baião n'en a pas. Pas de piste `Melody` (comme `gmd_funk`).

## Mesure → valeur

GMD `latin/brazilian-baiao` (`references/gmd-latin-brazilian-baiao/measures-drums.json`, groupe `4-4:straight`) :
3 fichiers `beat`, 200 mesures, 7,9 minutes, 2 batteurs, **« juste »** ; 29 fichiers `fill`, 1 batteur, 29 mesures.

| Mesure | Effectif | Valeur dans le profil |
|---|---|---|
| Tempo des fichiers : 95, 95, 110 BPM (p10 95, p50 95, p90 107) | 3 fichiers | `default_bpm` 95, `min_bpm` 95, `max_bpm` 110 ; trois plages d'habillage 95..98, 101..104, 107..110 |
| Feel : rapport de swing 0,50 | 3 fichiers sur 3 droits | `default_swing_ratio = 0.5`, pas de `[swing]` |
| Grosse caisse (zabumba) : probabilité par pas 1,00 (pas 0), 0,99 (3), 0,97 (8), 0,94 (11) ; 0,27 (6) et 0,29 (14) | 200 mesures ; figure [0, 3, 8, 11] dans 104 mesures (52 %), [0, 3, 6, 8, 11, 14] dans 50 (25 %) | zabumba aux pas 0, 3, 8, 11 dès la densité 0 ; pas 6 et 14 à la densité 2 du groupe `ride`, pas 14 à celle de la `caixa` (seuil 0,25) |
| Charleston à la pédale : 0,75 (pas 2), 0,84 (6), 0,79 (10), 0,83 (14) | 200 mesures ; figure [2, 6, 10, 14] dans 141 mesures (70,5 %) | pédale aux quatre contretemps dès la densité 1 |
| Timekeeper : ride 101 mesures (50,5 %), aucun 67 (33,5 %), charleston fermée 30 (15 %), ouverte 2 | 200 mesures | trois groupes (ci-dessous) |
| Groupe `ride` (ride, figure `quarters_plus`) | 57 mesures, 2 fichiers, 2 batteurs | 3 patterns (densités 0 à 2), étiquette `ride` |
| Groupe `hat` (charleston fermée, `quarters_plus`) | 21 mesures, 2 fichiers, 1 batteur | 3 patterns, étiquette `hat` |
| Groupe `caixa` (aucun timekeeper : caisse claire en doubles croches, grosse caisse [0, 3, 8, 11], pédale [2, 6, 10, 14]) ; probabilité de la caisse claire par pas 0,69 à 0,96 | 67 mesures, 2 fichiers, 2 batteurs, **mais 55 mesures (82 %) d'un seul fichier** | 3 patterns, étiquette `caixa` ; coups de caisse claire avec `probability` = fréquence mesurée |
| Mesures sans pattern : ride `sparse` 24 et `other` 20, charleston fermée `sparse` 5 et `other` 4, charleston ouverte 2 | 55 mesures | non reproduites |
| Phrases : suites de groupes vues au moins 2 fois (seuil relâché, 1 fichier suffit) | `ride` et `hat` : 2 et 4 mesures du même groupe ; `caixa` : 29 fenêtres de 2 et 11 de 4 mesures | 18 phrases ; **aucune suite qui change de groupe n'a 2 occurrences** : toutes les phrases tiennent un groove |
| Relances : 29 fichiers `fill`, 27 sur une mesure, 2 sur deux ; premier coup au pas 0 (29 sur 29), dernier coup médian au pas 14, 20 coups médians | 1 batteur | 1 relance d'une mesure (seuil 0,25), poids 29 ; `fills_none_weight = 46` (valeur de service) |
| Micro-timing par voix : médianes en pas (grosse caisse -0,09 sur le temps, caisse claire -0,12, charleston -0,07, pédale -0,16 sur le temps et -0,2 hors temps (borne), ride -0,07…) | 4 976 coups | `performance.tracks.*.voices`, bornées à ±0,2 pas (limite du schéma), sans dispersion |

WJazzD `LATIN` (`references/wjazzd-latin/measures-bass.json`) : 27 solos dont **23 avec des mesures valides, 1 639 mesures,
« assez »**. C'est du hard bop sur un rythme latin : il ne distingue ni la bossa, ni la samba, ni le baião, et ses
tempos (médiane 152,7 BPM) ne valent pas pour un baião à 95..110. Croches et silences de basse : non observables.

| Mesure | Effectif | Valeur dans le profil |
|---|---|---|
| Hauteur de basse p1 31, p5 33, p25 37, p50 40, p75 43, p95 48, p99 50 ; étendue 28..53 | 6 556 temps, 23 solos | fenêtre de la fondamentale `low..high` 33..44, `walk_low`/`walk_high` 28..53 |
| Approche du 4ᵉ temps avant un changement à la barre : aucune 60 %, chromatique 22 % [17-26], diatonique 19 % [11-29] | 1 205 événements, 22 solos | `passing` : `none_weight` 60, `chromatic` 22, `scale_step` 19, `before = [4]` |
| Mesures à un seul accord 87 %, changement au temps 1 85 % | 1 639 mesures | grilles à un accord par mesure, changements à la barre |
| Chorus : 8 mesures (31), 16 (9), 24 (9), 32 (9)… | 69 chorus complets | chorus de 16 mesures |

## Règles de métier (non mesurées, dites telles)

- **Grilles.** Vamps modaux du Nordeste, relatifs à la tonique : mixolydien (I7, bVII, IV), un ou deux accords, seize
  mesures : `I7` x 16 ; `I7`(4) `bVII`(4) x 2 ; `I7`(4) `IV`(4) x 2 ; `I7 bVII I7 IV` (2 mesures chacun) x 2. Poids 2, 3, 3, 2.
  Une boucle de grille ne dépend pas de la tonalité tirée : le mode n'est qu'une étiquette (manque, plus bas).
- **Zabumba.** La grosse caisse mesurée (0, 3, 8, 11) joue dès la densité 0, à toutes les densités : choix d'ossature.
  La zabumba réelle a deux peaux (« bacalhau » à la baguette, « leve » à la main) ; le kit GM n'a qu'une grosse caisse.
- **Triangle.** Huit coups par mesure (croches) : GM 80 (triangle muet) sur les temps, 81 (ouvert) sur les contretemps.
  **Limite du kit** : le triangle muet de GeneralUser GS reste sous -49 LUFS en solo à vélocité 100 (voir « Mixage ») :
  il est joué à 127 (maximum MIDI, non mesuré à ce niveau) ; le triangle ouvert porte le rythme.
- **Agogô.** 67 (aigu) et 68 (grave) aux pas 0, 3, 8, 11, à la densité 2 seulement.
- **Basse.** Six motifs d'une mesure : fondamentale sur le un et le trois (pas 0, 8), quinte en contretemps (pas 6, 14),
  doublures de la fondamentale aux pas 3 et 11 de la zabumba pour deux motifs. Les motifs suivent l'accord publié
  (`reference = "chord_root"`) : arrivée sur la basse publiée à chaque changement.
- **Comping.** Rythmes d'une mesure en trois étiquettes (`sparse`, `medium`, `dense`) : accords tenus, contretemps
  aux pas 6 et 14 (le pas 14 anticipé sur l'accord de la mesure suivante), croches fantômes avec `probability`
  (0,35 à 0,4) pour `dense`. Phrases de 2 et 4 mesures, une mise en place `part_end` par étiquette.
- **Arc.** Exposition (charleston, comping clairsemé), deux chorus de corps (ride ; le second, une fois sur deux,
  aussi à la `caixa`), sommet (ride ou `caixa`, comping dense, stop-time une fois sur deux sur les deux dernières
  mesures), retombée (charleston, stop-time). `repeat_from = 1` si l'hôte joue plus de cinq chorus.

## Grilles par défaut

Quatre boucles de seize mesures, relatives à la tonique (G, D, C, F, A ou Si bémol, mixolydien, selon l'habillage). Un
chorus = une boucle ; les cinq chorus de l'air rejouent la même boucle (le tirage est par air). Sous une grille fournie
par l'utilisateur, basse, comping et batterie ne dépendent que de l'accord publié et de la graine.

## Habillage par graine (`session.toml`, A2)

Neuf groupes indépendants, chacun règle des champs distincts, tirés une fois par graine : **28 variantes, 13 824
habillages possibles**. Programmes General MIDI de GeneralUser GS (0-based, vérifiés dans la banque).

| Groupe | Variantes | Champ |
|---|---|---|
| `tempo` | pose 95..98, intermédiaire 101..104, allant 107..110 | `bpm` (plage, BPM entier tiré) |
| `tonalite` | sol (7), ré (2), do (0), fa (5), la (9), si bémol (10), tous mixolydiens | `tonality` |
| `comping` | accordéon 21, guitare nylon 24, piano 0 | `chords_program` |
| `basse` | contrebasse 32, basse aux doigts 33 | `bass_program` |
| `kit` | standard 0, salle 8, jazz 32, balais 40 (banque 128) | `percussion_bank`, `percussion_program` |
| `voicing` | serré `close_triad`, `drop2`, coquille `shell`, ouvert `open` | `voicing` |
| `registre` | normal, grave (comping une octave plus bas) | `register_octaves` |
| `energie` | souple (0,9), franc (1,1) | `intensity`, `density` |
| `arc` | continu (`arc.toml`), orchestre (balais à l'exposition, accordéon au sommet) | `arc` |

**Tempo : faible étendue de la source.** Le GMD n'a que trois fichiers, à 95, 95 et 110 BPM : étendue de 15 BPM. Les
trois plages restent dans 95..110 et **ne sont pas trois régimes mesurés** ; seules la première et la dernière sont
séparées de plus de 8 BPM (deux grappes de tempo à ± 8 BPM au plus). Aucun BPM hors de la source : une extension serait
une règle de métier que je n'ai pas prise.

## Mixage

Niveaux de piste réglés par mesure, pas à l'oreille : LUFS intégrés en solo (`ffmpeg ebur128`, 48 mesures, graines 1 et 2),
cibles de service communes aux profils `gmd_*` (accords -26, basse -25, batterie -25,5, mix -21 à -22 ; aucun corpus).

| Piste | Graine 1 | Graine 2 | Moyenne | Cible |
|---|---:|---:|---:|---:|
| Chords | -26,4 | -25,5 | -25,9 | -26 |
| Bass | -25,2 | -25,0 | -25,1 | -25 |
| PercussionLow (zabumba) | -28,1 | -25,6 | -26,9 | |
| PercussionHigh | -32,0 | -29,5 | -30,8 | |
| batterie (somme en puissance) | -26,6 | -24,1 | -25,4 | -25,5 |
| mix complet | -21,6 | -20,3 | -21,0 | -21 à -22 |
| crête vraie (dBFS) | -4,6 | -3,6 | -4,1 | <= -1 |

`level_db` : accords -12,4, basse -12,4, batterie -9,5, `master_db` 20. Les voix de batterie reprennent les niveaux de B47
pour les rôles communs (grosse caisse 86, charleston 65, pédale 69, ride 66, cross-stick 68, toms 48 et 60) ; la caisse claire est à 48
(sa vélocité médiane GMD de 42 est celle de coups fantômes joués à presque chaque pas). Voix latines calées sur le kit
standard 128/0 (`gmd_latin_brazilian_baiao_calibrage_kit.py`, 4 coups par mesure, vélocité 50 puis 100) :

| Voix | LUFS à 50 | LUFS à 100 | Vélocité retenue |
|---|---:|---:|---:|
| triangle muet (80) | -59,9 | -49,9 | 127 (maximum MIDI ; la cible demandait plus) |
| triangle ouvert (81) | -46,0 | -36,0 | 72 |
| agogô aigu (67) | -52,8 | -39,2 | 112 |
| agogô grave (68) | -51,0 | -36,8 | 105 |

Les kits de l'habillage (0, 8, 32, 40) n'ont pas le même niveau : mesuré sur le kit standard seulement ; l'écart de la
batterie entre les graines 1 (balais) et 2 (jazz) est de 2,5 dB.

## Preuves

Toutes depuis `~/Project/bascanada/harmonium` avec le binaire du worktree (`<wt>/target/release/harmonium-cli`), sans
son, sans studio ; commandes et tableaux complets dans `specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao.md`.

| Preuve | Commande | Résultat |
|---|---|---|
| Habillages distincts | `mesures_latin.py habillages --seeds 16 --bars 48` | **16 / 16** graines ; 2 grappes de tempo ; 6 tonalités ; 4 kits |
| Variété entre chorus (`seed-variety --chorus 4`, 16 graines) | `harmonium-lab seed-variety … --label B3` | batterie : 81 % des mesures changées d'un chorus au suivant, 5 % de paires de chorus identiques (sans habillage 88 % et 17 % ; V0 `wjazzd_swing` 0 % et 100 %) ; comping : 84 % ; basse : **43 %** (plan de motifs fixé par air) |
| Basse contre WJazzD `LATIN` | `mesures_latin.py basse`, `gmd_latin_brazilian_baiao_approches.py` | hauteur p1 / p50 / p99 33 / 39 / 44 contre 31 / 40 / 50 ; 35 % des changements à la barre précédés d'une approche contre 40 % (chromatique 64 % des approches contre 54 %) ; arrivée sur la basse publiée 100 % contre 57 % |
| Distance de la batterie au GMD (voix latines exclues) | `gmd_latin_brazilian_baiao_distance.py` | moyenne de Bray-Curtis 0,157 (0,074 sans arc) contre 0,844 pour la copie non calibrée ; caisse claire 0,34 (le point faible) |
| Mixage | `gmd_latin_brazilian_baiao_mix.py` | accords -25,9, basse -25,1, batterie -25,4 LUFS, mix -21,0 ; crête vraie -3,6 dBFS au plus |
| Contrat | `cargo test -p harmonium_core --test gmd_latin_brazilian_baiao_profile_tests` | 11 tests |

## Limites

- **Deux batteurs.** « Juste » : un fichier à 110 BPM d'un batteur donne 104 mesures sur 200 et 55 des 67 mesures de la
  `caixa` ; la caisse claire en doubles croches est le groove d'un seul fichier, gardée hors seuil (82 % > 60 %).
- **Aucune percussion latine dans le GMD** (kit Roland TD-11) : triangle, agogô et zabumba à deux peaux sont du métier.
- **La figure du ride** (`quarters_plus`, trois coups par temps aux pas 0, 1 et 2) est lue telle quelle ; 17 % des coups du
  corpus sont à plus d'un quart de pas de la grille (`grid_fit`), des triolets rapportés au 16 pas en sont une cause possible,
  non vérifiée.
- **Phrases : aucune suite ne change de groupe** (seuils relâchés) ; les relances sont une mesure moyenne de 29 fichiers d'un
  batteur (une mesure de caisse claire en doubles croches avec la zabumba).
- **Basse.** Arrivée sur la basse publiée à chaque changement : 100 % ici (un motif arrive toujours sur la basse publiée), contre
  57 % dans WJazzD ; croches, silences, notes fantômes non mesurables. Le moteur applique `passing` à toute note de motif qui
  change de hauteur au suivant, plus souvent que la part mesurée.
- **Tempo.** 95..110 BPM seulement, aucune extension.
- **La grille ne dépend pas de la tonalité tirée.**
- Tout est jugé par des chiffres (D5), rien à l'oreille.

## Reproduire

```sh
# profil (labo en lecture seule) ; percussion.toml est GÉNÉRÉ
~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_generer.py --out profiles/gmd_latin_brazilian_baiao
cargo test -p harmonium_core --test gmd_latin_brazilian_baiao_profile_tests
```

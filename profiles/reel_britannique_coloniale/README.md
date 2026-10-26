# Reel britannique colonial (violon, guitare tenue, sans pieds)

## Tradition, corpus et choix du style (Constitution V et VII)

- **Tradition** : Musique britannique coloniale, bals de garnison et danses de campagne d'Amérique du Nord britannique, c. 1760–1840.
- **Corpus de référence** : `harmonium_lab`, `corpus/britannique_coloniale/tunes/reel/`, version 2026-10-22+1025.
  **493 airs** (Nottingham Music Database 479, Cahusac 8, AmRev (Chambers) 6) ; 493 airs mesurés, 51266 notes écrites, 416 airs aux parties de huit mesures positionnables. Provenance et droits : `corpus/sources.md`.
- Aucune musique autochtone.

### Pourquoi ce style

Le corpus britannique colonial compte 493 reels (479 Nottingham Music Database, 8 Cahusac 1809, 6 AmRev de Chambers) : c'est le plus
gros type non québécois du corpus. Il se distingue du reel québécois (574) et du reel acadien (169) **à l'oreille** par l'absence de podorythmie
et de chops (accords tenus sur les deux pulsations de la blanche), un tempo plus posé de danse de bal, des ornements écrits presque absents
(aucun rouleau ; 5 appogiatures simples et 5 triolets répétés sur 51 266 notes) et une harmonie à un accord par mesure (7 973 mesures sur 11 992 chiffrées).
Les tonalités sont ré (193 airs) et sol (168) ionien, avec peu de modal. Le libellé « colonial » est celui du lot du labo : la source
dominante est un recueil anglais (NMD), pas des relevés nord-américains ; même réserve pour la gigue et le hornpipe britanniques.

### Métrique, swing et tempo

2/2 en 16 pas, pulse à la blanche (2 pulses de 8 pas) comme les autres reels, **croches égales** (swing 0,5 : le 4/4 du corpus ne
documente aucun swing). Tempo **proposé** 208 à la noire (104 à la blanche), plage 192–224 : danse de bal plus lente que les reels
de veillée (230) — aucune mesure (corpus sans `Q:`), à régler à l'oreille.

### Formes observées

Parties notées (A à F) : A 399, B 398, C 117, D 61, E 4, F 3 (982 au total). Structures de paires de mesures (part des parties) : « a b c d » 40.0% (216 airs) ; « a b a c » 20.4% (149 airs) ; « a b a' c » 10.9% (91 airs) ; « a b a b' » 8.0% (65 airs).

## Phrases probabilistes — DS-T9 / P3-T1

Source : `harmonium_lab/references/structure/britannique_coloniale-reel.json`, champ `real` (formules de [profiles/README.md](../README.md#source-des-réglages-distribués-p3-t1-contrat-11) ;
reproduites par `specs/013-moteur-dans-le-jeu/r1_profils.py selftest` sur quatre profils distribués).

| Partie | Copie m.5←1 | Copie m.6←2 | Cadence tonique m.8 | Copie B:m.7←A:m.7 |
|---|---:|---:|---:|---:|
| A | 64.16% | 49.62% | 15.79% | 0.00% |
| B | 48.24% | 37.44% | 18.09% | 33.42% |

Poids arrondis sur 10 000 ; les mesures se tirent indépendamment ; les taux conjoints m.5–6 et les structures de paires restent des
mesures de sortie, pas des garanties du profil. `melody_center_offset` de B = 1 demi-tons (moyenne `register.b_minus_a_mean_pitch` arrondie).

Rythmes de cadence m.4/m.8 : 62 motifs de mesure pleine à durées contrastées, 5157 mesures (`real.rhythm.cells`, effectifs sans arrondi, même distribution aux deux positions).

## Secondes fins — P3-T2

Effectifs de dernière note **écrite** des fins finales de parties de huit mesures (tonique / quinte / autre), commande [`measure_endings.py`](../../specs/011-phase3-phrase/measure_endings.py) :

- A : 250 / 21 / 42 (313 occasions)
- B : 129 / 17 / 21 (167 occasions)

## Cellules mélodiques

`cells.toml` : `harmonium-lab mine-cells --type reel --corpus corpus/britannique_coloniale` (437 airs, top 30 par rôle) :
open 30, run 30, arp 30, turn 30, cad_half 30, cad_full 30 cellules, poids = effectifs. Validées par le chargeur du moteur (`--check-cells`).

Tonalités et modes (`key_modes`, cinq plus fréquents du corpus, poids de somme 20 par plus grand reste) : Ré ionian (193 airs), Sol ionian (168 airs), La ionian (62 airs), Do ionian (22 airs), La aeolian (11 airs) sur 493 airs analysés.

## Harmonie et accompagnement

Accords tenus (durée de 6 pas) sur les pas 0 et 8, motif à quatre accords à haute densité ; basse sur 0 et 8 (durée 6) ; notes de passage
à poids de reel (`none_weight` 4). Guitare nylon et basse acoustique comme les profils britanniques ; **aucune** podorythmie
(table de pieds vide, entrées d'arrangement conservées pour garder la même forme de fichier). Accompagnement par convention
(étude V5a §2.1 : aucune source sur les reels de bal coloniaux). Un accord par mesure (`steps_per_chord = 16`) : le rythme harmonique
dominant du corpus (7 973 mesures à un accord, 3 879 à deux).

## Espace et mixage

Salle de danse courte des profils britanniques (taille 0,40, amorti 0,55, niveau 0,28), guitare à droite, violon légèrement à gauche,
bus 13,5 dB (cinq graines : RMS moyen −19,7 dBFS en neutre, pic le pire aux émotions maximales −2,9 dBFS).

Choix d'espace proposés, pas des mesures historiques ; à valider à l'oreille. Schéma : [Mixage](../README.md#mixage).

## Interprétation V2

Source : `harmonium_lab/references/ornaments/britannique_coloniale-reel.json` (DS-T10b). 493 airs, 51266 attaques porteuses écrites.

| Marque écrite | Occurrences | Par 100 notes |
|---|---:|---:|
| Liaison `(…)` | 55 | 0.107284 |
| Tenue liée `-` | 253 | 0.493504 |
| Détaché `.` | 4 | 0.007802 |
| Accent explicite | 0 | 0.000000 |
| Tenuto `L` | 0 | 0.000000 |

32 règles de conditionnelles **conjointes** position × durée × contexte (cellules d'au moins 30 porteuses), poids `100 × marques / (porteuses − toutes marques)` face à `Normal = 100`, comme les sept profils distribués ; chaque règle porte son effectif en commentaire. Micro-timing, jitter, courbe de phrase, respiration et émotions : **valeurs de proposition des profils voisins** (même mètre), à valider à l'oreille ; le corpus ne les mesure pas. Schéma : [Interprétation](../README.md#interprétation).

## Ornements V3b

Source : `harmonium_lab/references/ornaments/britannique_coloniale-reel.json` (DS-T10, `engine_proposal`) ; `measure_ornaments.py corpus` contrôle que les taux du profil sont le recalcul du corpus. Taux écrits = **planchers** (les transcriptions sous-notent), `rate_scale = 4` pour tous les types, multiplicateurs d'arousal neutres.

| Type | Occurrences | /100 notes (plancher) | Taux par mesure | Durée porteuse min. (croches) |
|---|---:|---:|---|---:|
| `grace_single` | 5 | 0.010 | non | 1 |
| `grace_multiple` | 0 | 0.000 | — | aucune occurrence : désactivé |
| `roll` | 0 | 0.000 | — | aucune occurrence : désactivé |
| `turn` | 0 | 0.000 | — | aucune occurrence : désactivé |
| `triplet_repeated` | 5 | 0.010 | non | 2 |
| `double_stop` | 151 | 0.295 | oui | 2 |
| `chord` | 2 | 0.004 | non | 2 |
| `trill` | 1 | 0.002 | non | 2 |

`passing` : note de passage de densité émotionnelle, courbe `[[0,0],[0.5,0],[1,1]]`, tenue ≥ 1,5 croche, sans tirage. Durées porteuses : minimum observé sous 30 occurrences ; sinon plus petite durée (classes de ≥ 5 occurrences) dont le taux par note atteint 10 % du meilleur ; triolet = durée totale remplacée. Schéma : [Ornements](../README.md#ornements).

## Forme longue V7b

`set.toml` : sets de 2 à 5 airs (45·44·7·4, The Session), intro complète et entrées de pistes au premier air de chaque set. Facteurs de tonalité = rapports observé / indépendance **reel → reel** de The Session, repris de reel_quebecois : `same` 1.28 · `parallel` 1.00 · `relative` 1.02 · `fifth_up` 0.97 · `fourth_up` 1.01 · `step_up` 0.91 · `step_down` 0.73 · `other` 0.75 · `last_dark` 0.80. Aucune mesure par tradition ni de pratique de set acadienne/britannique coloniale : valeurs de répertoire, à régler à l'écoute. Rampe : +5 bpm / +0.06 d'intensité, plafond 224 (proposition, valeur du type voisin ; borne proxy cap-bretonnaise, **pas une mesure**). Raccords : direct 90, held 10. Schéma : [Forme longue](../README.md#forme-longue-set).

## Hétérophonie V4

**Aucune seconde voix** : pas de source d'ensemble pour ce répertoire, comme pour la gigue et le hornpipe britanniques.

## Limites

L'essentiel du lot vient d'un recueil (NMD, notation sans ornements ni articulations : 73,6 % des airs sans aucune marque). Les taux
d'ornements et d'articulations sont donc des planchers très bas, et rien dans l'ABC ne distingue un jeu colonial d'un jeu anglais.
Seules les deux premières parties sont modélisées : le profil joue AABB comme tous les autres, alors que 130 airs sur 437 sont notés
AABB, 115 AAB, 47 ABCD (parties toutes différentes) et 47 en croches continues sans parties.

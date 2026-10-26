# Gigue acadienne (violon, piano de veillée et podorythmie)

## Tradition, corpus et choix du style (Constitution V et VII)

- **Tradition** : Acadie / Maritimes (Nouveau-Brunswick, Île-du-Prince-Édouard, Cap-Breton / Chéticamp, Îles-de-la-Madeleine), répertoire traditionnel de violon, de piano et de podorythmie.
- **Corpus de référence** : `harmonium_lab`, `corpus/acadienne/tunes/jig/`, version 2026-10-22+270.
  **55 airs** (The Session, ODbL ; airs enregistrés par des artistes acadiens) ; 55 airs mesurés, 6100 notes écrites, 50 airs aux parties de huit mesures positionnables. Provenance et droits : `corpus/sources.md`.
- Aucune musique autochtone.

### Pourquoi ce style

Le corpus acadien compte 55 gigues (The Session, ODbL, airs enregistrés par des artistes acadiens ; même source que le reel acadien,
dont l'échantillon est dominé par le Cap-Breton). Elle se distingue de la gigue québécoise (88 airs) par des **données** — taux
d'ornements écrits de 2 à 8 fois plus élevés (appogiatures 0,39 contre 0,21 pour 100 notes, multiples 0,26 contre 0,03, rouleaux 0,41
contre 0,21, doubles cordes 0,39 contre 0,08), la majeur bien plus présent (11 airs sur 55, 20 %, contre 7 sur 88 au Québec), paires de
mesures « a b c d » à 32,5 % contre 24,5 % — et par **l'accompagnement**, qui est le point audible : le piano de
veillée en recette « main gauche » (basse sur 1 et 4, accords sur 2-3 et 5-6) remplace la guitare. Le piano est la pratique
documentée du Cap-Breton voisin (étude V5a §2.1 : MacDonald, Fraser, MacLean) ; rien de comparable n'est mesuré pour l'Acadie
elle-même : extrapolation **[V]**, à valider à l'oreille.

### Métrique, swing et tempo

6/8 en 12 pas, pulse à la noire pointée (2 pulses de 6 pas), croches égales (swing 0,5). Tempo **proposé** : 180 à la noire,
soit 120 par noire pointée, plage 168–192 — **aucune mesure** : le corpus ne porte aucun `Q:` exploitable (`mine-tempo` : tempos manquants).
Comme la gigue québécoise (172,5), la valeur reste dans l'ordre de grandeur cap-breton ; elle sera réglée à l'écoute.

### Formes observées

Parties notées (A à F) : A 50, B 49, C 14, D 4, E 2, F 1 (120 au total). Structures de paires de mesures (part des parties) : « a b c d » 32.5% (30 airs) ; « a b a c » 19.2% (19 airs) ; « a b a' c » 19.2% (20 airs) ; « a a' a' b » 11.7% (7 airs).

## Phrases probabilistes — DS-T9 / P3-T1

Source : `harmonium_lab/references/structure/acadienne-jig.json`, champ `real` (formules de [profiles/README.md](../README.md#source-des-réglages-distribués-p3-t1-contrat-11) ;
reproduites par `specs/013-moteur-dans-le-jeu/r1_profils.py selftest` sur quatre profils distribués).

| Partie | Copie m.5←1 | Copie m.6←2 | Cadence tonique m.8 | Copie B:m.7←A:m.7 |
|---|---:|---:|---:|---:|
| A | 72.00% | 36.00% | 16.00% | 0.00% |
| B | 51.02% | 34.69% | 10.20% | 51.02% |

Poids arrondis sur 10 000 ; les mesures se tirent indépendamment ; les taux conjoints m.5–6 et les structures de paires restent des
mesures de sortie, pas des garanties du profil. `melody_center_offset` de B = 2 demi-tons (moyenne `register.b_minus_a_mean_pitch` arrondie).

Rythmes de cadence m.4/m.8 : 12 motifs de mesure pleine à durées contrastées, 491 mesures (`real.rhythm.cells`, effectifs sans arrondi, même distribution aux deux positions).

## Secondes fins — P3-T2

Effectifs de dernière note **écrite** des fins finales de parties de huit mesures (tonique / quinte / autre), commande [`measure_endings.py`](../../specs/011-phase3-phrase/measure_endings.py) :

- A : 22 / 3 / 23 (48 occasions)
- B : 24 / 2 / 10 (36 occasions)

## Cellules mélodiques

`cells.toml` : `harmonium-lab mine-cells --type jig --corpus corpus/acadienne` (55 airs, top 30 par rôle) :
open 30, run 30, arp 30, turn 24, cad_half 30, cad_full 30 cellules, poids = effectifs. Validées par le chargeur du moteur (`--check-cells`).

Tonalités et modes (`key_modes`, cinq plus fréquents du corpus, poids de somme 20 par plus grand reste) : Ré ionian (13 airs), Sol ionian (11 airs), La ionian (11 airs), Do ionian (4 airs), Mi dorian (3 airs) sur 55 airs analysés.

## Harmonie et accompagnement

**Piano** (programmes 0/0) selon la recette « piano main gauche » de [profiles/README.md](../README.md#recettes-piano-et-accordéon),
adaptée à la grille de gigue : basse (doublée à l'octave) sur les pas 0 et 6, accords en blocs sur 2/4/8/10 (motif par défaut) ou 4/10
(motif clairsemé à faible densité), marches de basse et conduite de voix de la recette ; **aucun grattage**, le piano n'en a pas.
Pieds : même table que la gigue québécoise (talon sur 1 et 4, pointes sur 2-3 et 5-6, pratique décrite pour le Cap-Breton, Fraser 2012).
L'ABC ne fournit presque aucun accord (56 mesures chiffrées sur 55 airs) : harmonie I/IV/V/vi par mode comme la gigue québécoise.
Mesure : mélodie − piano 7,4 dB, mélodie − basse 10,3 dB (critère ≥ 4 dB) ; la basse du piano GM penche à gauche
(G−D +3,8 dB avec `pan = 0`) : c'est le placement des touches de la banque, laissé tel quel.

## Espace et mixage

Salle communautaire courte comme les gigues et reels à pieds (taille 0,38, amorti 0,58, niveau 0,28). Piano et basse à −5 / −8 dB,
pieds −6 dB, bus **+15 dB** : le piano GM est ~1,5 dB moins énergique que la guitare nylon à niveau égal (RMS moyen de cinq graines en
neutre −21,1 dBFS à 13,5 dB de bus, −19,6 à 15 dB), le bus est relevé pour retrouver le niveau des autres gigues ; pic le pire des cinq
graines aux émotions maximales −2,3 dBFS.

Choix d'espace proposés, pas des mesures historiques ; à valider à l'oreille. Schéma : [Mixage](../README.md#mixage).

## Interprétation V2

Source : `harmonium_lab/references/ornaments/acadienne-jig.json` (DS-T10b). 55 airs, 6100 attaques porteuses écrites.

| Marque écrite | Occurrences | Par 100 notes |
|---|---:|---:|
| Liaison `(…)` | 1 | 0.016393 |
| Tenue liée `-` | 10 | 0.163934 |
| Détaché `.` | 0 | 0.000000 |
| Accent explicite | 0 | 0.000000 |
| Tenuto `L` | 0 | 0.000000 |

2 règles de conditionnelles **conjointes** position × durée × contexte (cellules d'au moins 30 porteuses), poids `100 × marques / (porteuses − toutes marques)` face à `Normal = 100`, comme les sept profils distribués ; chaque règle porte son effectif en commentaire. Micro-timing, jitter, courbe de phrase, respiration et émotions : **valeurs de proposition des profils voisins** (même mètre), à valider à l'oreille ; le corpus ne les mesure pas. Schéma : [Interprétation](../README.md#interprétation).

## Ornements V3b

Source : `harmonium_lab/references/ornaments/acadienne-jig.json` (DS-T10, `engine_proposal`) ; `measure_ornaments.py corpus` contrôle que les taux du profil sont le recalcul du corpus. Taux écrits = **planchers** (les transcriptions sous-notent), `rate_scale = 4` pour tous les types, multiplicateurs d'arousal neutres.

| Type | Occurrences | /100 notes (plancher) | Taux par mesure | Durée porteuse min. (croches) |
|---|---:|---:|---|---:|
| `grace_single` | 24 | 0.393 | non | 1 |
| `grace_multiple` | 16 | 0.262 | non | 0.5 |
| `roll` | 25 | 0.410 | non | 2 |
| `turn` | 0 | 0.000 | — | aucune occurrence : désactivé |
| `triplet_repeated` | 0 | 0.000 | — | aucune occurrence : désactivé |
| `double_stop` | 24 | 0.393 | non | 1 |
| `chord` | 0 | 0.000 | — | aucune occurrence : désactivé |
| `trill` | 0 | 0.000 | — | aucune occurrence : désactivé |

`passing` : note de passage de densité émotionnelle, courbe `[[0,0],[0.5,0],[1,1]]`, tenue ≥ 1,5 croche, sans tirage. Durées porteuses : minimum observé sous 30 occurrences ; sinon plus petite durée (classes de ≥ 5 occurrences) dont le taux par note atteint 10 % du meilleur ; triolet = durée totale remplacée. Schéma : [Ornements](../README.md#ornements).

## Forme longue V7b

`set.toml` : sets de 2 à 5 airs (45·44·7·4, The Session), intro complète et entrées de pistes au premier air de chaque set. Facteurs de tonalité = rapports observé / indépendance **jig → jig** de The Session, repris de gigue_quebecoise : `same` 1.21 · `parallel` 0.90 · `relative` 0.98 · `fifth_up` 0.97 · `fourth_up` 1.06 · `step_up` 0.79 · `step_down` 0.74 · `other` 0.95 · `last_dark` 1.00. Aucune mesure par tradition ni de pratique de set acadienne/britannique coloniale : valeurs de répertoire, à régler à l'écoute. Rampe : +6 bpm / +0.08 d'intensité, plafond 192 (proposition, valeur du type voisin ; borne proxy cap-bretonnaise, **pas une mesure**). Raccords : direct 70, call 20, held 10. Schéma : [Forme longue](../README.md#forme-longue-set).

## Hétérophonie V4

Second violon, entrée au tour 2, unisson dominant : **mêmes poids que le reel acadien** (76 / 10 / 8 / 6 / 2 / 2 / 1 / 2). Proposition
d'arrangement à écouter, pas un taux d'hétérophonie mesuré dans le corpus monodique ABC.

## Limites

55 airs d'une seule source (The Session) : positions de mesure, taux par mesure et secondes fins reposent sur de petits effectifs
(B : 36 secondes fins, A : 48). Aucun ornement `turn`, `chord` ni trille mesuré dans ce type : désactivés. Les tailles de l'échantillon
interdisent de conclure sur la tradition acadienne dans son ensemble.

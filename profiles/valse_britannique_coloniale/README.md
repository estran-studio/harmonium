# Valse britannique coloniale (violon, guitare, sans pieds)

## Tradition, corpus et choix du style (Constitution V et VII)

- **Tradition** : Musique britannique coloniale, bals de garnison et salons d'Amérique du Nord britannique, c. 1760–1840.
- **Corpus de référence** : `harmonium_lab`, `corpus/britannique_coloniale/tunes/waltz/`, version 2026-10-22+1025.
  **60 airs** (Nottingham Music Database 58, Cahusac 2) ; 60 airs mesurés, 5126 notes écrites, 52 airs aux parties de huit mesures positionnables. Provenance et droits : `corpus/sources.md`.
- Aucune musique autochtone.

### Pourquoi ce style

60 valses (58 Nottingham Music Database, 2 Cahusac) ; la valse québécoise en compte 67 (42 Chambers, 25 The Session). Le profil se distingue
**dans les données** — forme plus souvent en quatre parties différentes (« a b c d » à 66,7 % des paires contre 45,5 % au Québec),
doubles cordes beaucoup plus fréquentes (2,54 pour 100 notes contre 1,28), ré et sol majeurs dominants, finales longues quasi systématiques
aux m.4/m.8 (76–82 %) — et **à l'oreille** par l'absence de pieds (la valse québécoise en a), un tempo plus posé et une guitare identique
mais sans podorythmie. Époque du lot : c. 1760–1840 selon le manifeste du labo.

### Métrique, swing et tempo

3/4 en 12 pas, pulse à la noire (3 pulses de 4 pas), swing 0,5. Tempo **proposé** 138 à la noire (plage 126–150), un peu en dessous
de la valse québécoise (145) : aucune mesure (corpus sans `Q:`).

### Formes observées

Parties notées (A à F) : A 50, B 51, C 45, D 25, E 3, F 2, G 1 (177 au total). Structures de paires de mesures (part des parties) : « a b c d » 66.7% (46 airs) ; « a b a' c » 11.3% (13 airs) ; « a a' b c » 5.6% (9 airs) ; « a a b c » 4.0% (4 airs).

## Phrases probabilistes — DS-T9 / P3-T1

Source : `harmonium_lab/references/structure/britannique_coloniale-waltz.json`, champ `real` (formules de [profiles/README.md](../README.md#source-des-réglages-distribués-p3-t1-contrat-11) ;
reproduites par `specs/013-moteur-dans-le-jeu/r1_profils.py selftest` sur quatre profils distribués).

| Partie | Copie m.5←1 | Copie m.6←2 | Cadence tonique m.8 | Copie B:m.7←A:m.7 |
|---|---:|---:|---:|---:|
| A | 20.00% | 14.00% | 2.00% | 0.00% |
| B | 11.76% | 5.88% | 11.76% | 6.12% |

Poids arrondis sur 10 000 ; les mesures se tirent indépendamment ; les taux conjoints m.5–6 et les structures de paires restent des
mesures de sortie, pas des garanties du profil. `melody_center_offset` de B = 2 demi-tons (moyenne `register.b_minus_a_mean_pitch` arrondie).

Rythmes de cadence m.4/m.8 : 15 motifs de mesure pleine à durées contrastées, 1022 mesures (`real.rhythm.cells`, effectifs sans arrondi, même distribution aux deux positions).

## Secondes fins — P3-T2

Effectifs de dernière note **écrite** des fins finales de parties de huit mesures (tonique / quinte / autre), commande [`measure_endings.py`](../../specs/011-phase3-phrase/measure_endings.py) :

- A : 7 / 4 / 6 (17 occasions)
- B : 5 / 1 / 1 (7 occasions)

## Cellules mélodiques

`cells.toml` : `harmonium-lab mine-cells --type waltz --corpus corpus/britannique_coloniale` (58 airs, top 30 par rôle) :
open 7, run 30, arp 30, turn 2, cad_half 7, cad_full 8 cellules, poids = effectifs. Validées par le chargeur du moteur (`--check-cells`).

Tonalités et modes (`key_modes`, cinq plus fréquents du corpus, poids de somme 20 par plus grand reste) : Sol ionian (24 airs), Ré ionian (21 airs), Do ionian (5 airs), La aeolian (3 airs), Fa ionian (2 airs) sur 60 airs analysés.

## Harmonie et accompagnement

Basse sur le premier temps, accords sur les temps 2 et 3 (« boum-tchic-tchic »), guitare et basse de la valse québécoise, marches de basse
limitées au dernier quart de phrase ; **pas de pieds** (table vide). Un accord par mesure (1 710 mesures sur 1 823 chiffrées).

## Espace et mixage

Salle de danse courte des profils britanniques (taille 0,40, amorti 0,55, niveau 0,28), bus 12 dB comme la valse québécoise
(cinq graines : RMS moyen −19,5 dBFS en neutre, pic le pire aux émotions maximales −4,5 dBFS).

Choix d'espace proposés, pas des mesures historiques ; à valider à l'oreille. Schéma : [Mixage](../README.md#mixage).

## Interprétation V2

Source : `harmonium_lab/references/ornaments/britannique_coloniale-waltz.json` (DS-T10b). 60 airs, 5126 attaques porteuses écrites.

| Marque écrite | Occurrences | Par 100 notes |
|---|---:|---:|
| Liaison `(…)` | 13 | 0.253609 |
| Tenue liée `-` | 87 | 1.697230 |
| Détaché `.` | 3 | 0.058525 |
| Accent explicite | 0 | 0.000000 |
| Tenuto `L` | 0 | 0.000000 |

9 règles de conditionnelles **conjointes** position × durée × contexte (cellules d'au moins 30 porteuses), poids `100 × marques / (porteuses − toutes marques)` face à `Normal = 100`, comme les sept profils distribués ; chaque règle porte son effectif en commentaire. Micro-timing, jitter, courbe de phrase, respiration et émotions : **valeurs de proposition des profils voisins** (même mètre), à valider à l'oreille ; le corpus ne les mesure pas. Schéma : [Interprétation](../README.md#interprétation).

## Ornements V3b

Source : `harmonium_lab/references/ornaments/britannique_coloniale-waltz.json` (DS-T10, `engine_proposal`) ; `measure_ornaments.py corpus` contrôle que les taux du profil sont le recalcul du corpus. Taux écrits = **planchers** (les transcriptions sous-notent), `rate_scale = 4` pour tous les types, multiplicateurs d'arousal neutres.

| Type | Occurrences | /100 notes (plancher) | Taux par mesure | Durée porteuse min. (croches) |
|---|---:|---:|---|---:|
| `grace_single` | 3 | 0.059 | non | 0.5 |
| `grace_multiple` | 0 | 0.000 | — | aucune occurrence : désactivé |
| `roll` | 0 | 0.000 | — | aucune occurrence : désactivé |
| `turn` | 0 | 0.000 | — | aucune occurrence : désactivé |
| `triplet_repeated` | 0 | 0.000 | — | aucune occurrence : désactivé |
| `double_stop` | 130 | 2.536 | oui | 2 |
| `chord` | 1 | 0.020 | non | 6 |
| `trill` | 0 | 0.000 | — | aucune occurrence : désactivé |

`passing` : note de passage de densité émotionnelle, courbe `[[0,0],[0.5,0],[1,1]]`, tenue ≥ 1,5 croche, sans tirage. Durées porteuses : minimum observé sous 30 occurrences ; sinon plus petite durée (classes de ≥ 5 occurrences) dont le taux par note atteint 10 % du meilleur ; triolet = durée totale remplacée. Schéma : [Ornements](../README.md#ornements).

## Forme longue V7b

`set.toml` : sets de 2 à 5 airs (45·44·7·4, The Session), intro complète et entrées de pistes au premier air de chaque set. Facteurs de tonalité = rapports observé / indépendance **waltz → waltz** de The Session, repris de valse_quebecoise : `same` 1.27 · `parallel` 1.18 · `relative` 0.86 · `fifth_up` 0.97 · `fourth_up` 0.97 · `step_up` 0.84 · `step_down` 0.81 · `other` 0.62 · `last_dark` 1.00. Aucune mesure par tradition ni de pratique de set acadienne/britannique coloniale : valeurs de répertoire, à régler à l'écoute. Rampe : +2 bpm / +0.05 d'intensité, plafond 150 (proposition, valeur du type voisin ; borne proxy cap-bretonnaise, **pas une mesure**). Raccords : direct 100. Schéma : [Forme longue](../README.md#forme-longue-set).

## Hétérophonie V4

**Aucune seconde voix** : pas de source d'ensemble pour ce répertoire ; l'accordéon de la valse québécoise est un anachronisme déclaré
pour 1837 et n'est pas repris.

## Limites

Seulement 60 airs, 58 du même recueil ; 52 positionnables, 27 paires de fins à huit mesures (B : 7 secondes fins). Les poids de
`second_ending` de B et les taux par position sont des petits effectifs. Aucune articulation écrite sauf quelques liaisons et ties
(13 liaisons, 87 ties sur 5 126 notes).

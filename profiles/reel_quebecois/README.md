# Reel québécois

## Phrases probabilistes — DS-T9 / P3-T1

Source : `harmonium_lab/references/structure/quebecoise-reel.json`, champ `real`
(statistiques DS-T9, corpus et droits dans le manifeste du labo).
Les taux par mesure de `transformations` pilotent copies, séquences et fins variées ;
les déplacements utilisent les effectifs `sequence_shifts` de la même position.
Les repos toniques viennent de `phrases.rest_by_position` ; les autres longues
fins servent de proxy de demi-cadence. Voir [le schéma et les limites](../README.md#phrases--rôles-pondérés-motifs-et-cadences).

| Partie | Copie m.5←1 | Copie m.6←2 | Cadence tonique m.8 | Copie B:m.7←A:m.7 |
|---|---:|---:|---:|---:|
| A | 71.69 % | 66.27 % | 21.99 % | 0.00 % |
| B | 67.54 % | 58.19 % | 17.84 % | 20.63 % |

Poids arrondis sur 10 000 ; les mesures se tirent indépendamment.
Les taux conjoints m.5–6 et les structures de paires restent des mesures de
sortie, pas des garanties du profil. Banques de cellules inchangées.

## Secondes fins — P3-T2 (contrat 12)

`second_ending` utilise les effectifs de dernière note **écrite**, avant toute
complétion par une levée : tonique (`CadTonic`), quinte (`CadFifth`), autre
(`CadOther`). Source : `harmonium_lab/corpus/trad/tunes/reel/*.abc`,
analysé par `trad.form.analyze_form` avec la commande
[`measure_endings.py`](../../specs/011-phase3-phrase/measure_endings.py).
Les poids du profil sont les effectifs des paires de passes de huit mesures,
séparément pour A (**287** occasions) et B (**278** occasions), sans arrondi.

La variante s'applique à A2/B2 et reste identique à chaque tour ; les sept
autres mesures ne changent pas entre passes. Les premières fins conservent
les réglages DS-T9 ci-dessus. Les chiffres réels et les limites de mesure
sont dans [le rapport de phase](../../specs/011-phase3-phrase/plan.md#p3-t2--fins-de-reprise).

## Espace et mixage V1

Cuisine de veillée : petite salle amortie (taille 0,35, niveau 0,28), violon presque centré, guitare à gauche ; basse au centre, pieds proches.
Guitare −5 dB, basse −8 dB, pieds −6 dB de canal et bus +15,5 dB : priorité au violon sans effacer les entrées progressives ; choix de rendu, non mesure historique.

## Interprétation V2

Source : `harmonium_lab/references/ornaments/quebecoise-reel.json`, DS-T10b
(labo main `f8e0cbc`, corpus et droits dans ses manifestes).
574 airs, 72627 attaques porteuses écrites ; reprises non multipliées.

| Marque écrite | Occurrences | Par 100 notes |
|---|---:|---:|
| Liaison `(…)` | 181 | 0.249219 |
| Tenue liée `-` | 224 | 0.308425 |
| Détaché `.` | 65 | 0.089498 |
| Accent explicite | 0 | 0.000000 |
| Fermata `H` | 5 | 0.006884 |

Les règles sont des conditionnelles **conjointes** position × durée ×
intervalle précédent, pas un produit de marginales. Reproduction :
`measure_performance.py --corpus-only` réutilise uniquement
`harmonium_lab.trad.ornaments.extract_ornaments`, donc le parseur ABC commun.
Positions/durées exactement représentables seulement, sans arrondir les
triolets ni les changements de métrique ; notes de 1–2 pas ou 3–grille.
Au pas 0, contexte regroupé (le moteur recommence le contexte par mesure) ;
ailleurs, répétition/montée/descente. Cellules de moins de 30 porteuses
écartées par prudence, sans prétendre à une indépendance statistique.
Chaque commentaire de règle donne son effectif et son dénominateur.
Avec `Normal = 100`, poids `100 × marques / (porteuses − toutes marques)` ;
liaison/tie/tenuto → `Linked` plein, point → `Detached`, gate 0,80 avant
émotion. Une liaison compte sa porteuse initiale, pas toutes ses notes :
ces taux sont des planchers écrits et des **poids**, pas une garantie de
fréquence jouée ni une simulation des groupes de coups d’archet.

Aucun accent, tenuto `L` ni marcato écrit : le petit poids d’accent métrique
4 face à Normal 100 est une **proposition**, non une mesure du corpus.
`L` désigne tenuto dans ABC ; `H` est mesuré mais ne sert pas à déduire
une durée de respiration ou à ralentir les fermatas.

| Réglage proposé | Valeur / intention |
|---|---|
| Timing par pas | −0.02 aux pulses ; +0.01 aux autres pas pairs ; 0 aux impairs |
| Dispersion | ±0.02 pas, soit ±1.30 ms au tempo nominal avant swing |
| Courbe de phrase | −2 → +2 → −3 unités MIDI ; bornes 30..127 |
| Respiration | 8 % de la dernière durée avant chaque fin de phrase |
| Tension 0 → 1 | jusqu’à −0,04 pas et −0,10 de gate |
| Arousal 0 → 1 | accent de base +2, puis jusqu’à +4 unités supplémentaires |
| Valence / densité | coefficients 0 : pas de corrélation déduite des ABC |

Micro-timing, gates, accents, courbe et respiration : **à valider à l’oreille**.
Ces petites valeurs ne sont pas des mesures de violoneux/fifre de cette tradition.
Repères généraux : [discrimination temporelle, Friberg & Sundberg 1995](https://pubs.aip.org/asa/jasa/article/98/5/2524/798047/Time-discrimination-in-a-monotonic-isochronous)
et [phrasé/articulation, système KTH 2006](https://doi.org/10.2478/v10053-008-0052-x).
Sans `[performance]`, ou avec tous ses effets à zéro, le flux antérieur est
conservé sans tirage supplémentaire ; basse, accords et pieds non retouchés.
Schéma et unités : [Interprétation](../README.md#interprétation).

## Ornements V3b

Source : `harmonium_lab/references/ornaments/quebecoise-reel.json` (DS-T10, `engine_proposal`), recalculé par
`measure_ornaments.py corpus`. Taux écrits = **planchers** (les transcriptions sous-notent) ; `rate_scale = 4` les relève
identiquement pour tous les types, valeur d'écoute à valider. Multiplicateurs d'arousal neutres : le corpus n'annote pas l'arousal.
Le taux par mesure (forme du corpus positionné, niveau du taux global) n'est repris que pour les types à ≥ 30 occurrences dans des parties de huit mesures reconnues.

| Type | Occurrences (positionnées) | /100 notes (plancher) | /100 notes × échelle | Taux par mesure | Durée porteuse min. (croches) |
|---|---:|---:|---:|---|---:|
| `grace_single` | 134 (57) | 0.185 | 0.738 | oui | 0.5 |
| `grace_multiple` | 34 (9) | 0.047 | 0.187 | non | 1 |
| `roll` | 501 (321) | 0.690 | 2.759 | oui | 2 (observé 1/2) |
| `turn` | 3 (2) | 0.004 | 0.017 | non | 1 |
| `triplet_repeated` | 132 (74) | 0.182 | 0.727 | oui | 1 (3 × 1/3) |
| `double_stop` | 250 (81) | 0.344 | 1.377 | oui | 2 (observé 1) |
| `chord` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |
| `trill` | 7 (7) | 0.010 | 0.039 | non | 1 |

`passing` : note de passage de densité émotionnelle (ancien `dynamics.density_passing`), inchangée : courbe `[[0,0],[0.5,0],[1,1]]`, tenue ≥ 1,5 croche, sans tirage.

Effet de source manifeste dans le corpus : Chambers note 166 rouleaux pour 42 632 notes, The Session 335 pour 29 995 ; ne pas lire l'écart comme un effet de tradition.

Le triolet est donné en durée totale remplacée (le labo donne la première note après subdivision). `min_carrier_eighths` est le minimum observé quand l'effectif est faible ; au-delà de 30 occurrences, la plus petite durée dont le taux par note atteint 10 % du meilleur (les rouleaux du corpus sont sur des noires et des noires pointées, pas sur des croches). Schéma : [Ornements](../README.md#ornements).

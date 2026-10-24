# Gigue québécoise

## Phrases probabilistes — DS-T9 / P3-T1

Source : `harmonium_lab/references/structure/quebecoise-jig.json`, champ `real`
(statistiques DS-T9, corpus et droits dans le manifeste du labo).
Les taux par mesure de `transformations` pilotent copies, séquences et fins variées ;
les déplacements utilisent les effectifs `sequence_shifts` de la même position.
Les repos toniques viennent de `phrases.rest_by_position` ; les autres longues
fins servent de proxy de demi-cadence. Voir [le schéma et les limites](../README.md#phrases--rôles-pondérés-motifs-et-cadences).

| Partie | Copie m.5←1 | Copie m.6←2 | Cadence tonique m.8 | Copie B:m.7←A:m.7 |
|---|---:|---:|---:|---:|
| A | 68.49 % | 61.64 % | 16.44 % | 0.00 % |
| B | 62.50 % | 50.00 % | 22.22 % | 30.56 % |

Poids arrondis sur 10 000 ; les mesures se tirent indépendamment.
Les taux conjoints m.5–6 et les structures de paires restent des mesures de
sortie, pas des garanties du profil. Banques de cellules inchangées.

## Secondes fins — P3-T2 (contrat 12)

`second_ending` utilise les effectifs de dernière note **écrite**, avant toute
complétion par une levée : tonique (`CadTonic`), quinte (`CadFifth`), autre
(`CadOther`). Source : `harmonium_lab/corpus/trad/tunes/jig/*.abc`,
analysé par `trad.form.analyze_form` avec la commande
[`measure_endings.py`](../../specs/011-phase3-phrase/measure_endings.py).
Les poids du profil sont les effectifs des paires de passes de huit mesures,
séparément pour A (**67** occasions) et B (**60** occasions), sans arrondi.

La variante s'applique à A2/B2 et reste identique à chaque tour ; les sept
autres mesures ne changent pas entre passes. Les premières fins conservent
les réglages DS-T9 ci-dessus. Les chiffres réels et les limites de mesure
sont dans [le rapport de phase](../../specs/011-phase3-phrase/plan.md#p3-t2--fins-de-reprise).

## Espace et mixage V1

Veillée en petite salle : taille 0,38, amorti 0,58, niveau 0,28 ; violon presque au centre, guitare à gauche, basse et pieds proches du centre.
Guitare −5 dB, basse −8 dB, pieds −6 dB de canal et bus +13,5 dB préservent la danse et la priorité mélodique ; hypothèse sonore, non mesure historique.

## Interprétation V2

Source : `harmonium_lab/references/ornaments/quebecoise-jig.json`, DS-T10b
(labo main `f8e0cbc`, corpus et droits dans ses manifestes).
88 airs, 9194 attaques porteuses écrites ; reprises non multipliées.

| Marque écrite | Occurrences | Par 100 notes |
|---|---:|---:|
| Liaison `(…)` | 24 | 0.261040 |
| Tenue liée `-` | 42 | 0.456820 |
| Détaché `.` | 0 | 0.000000 |
| Accent explicite | 0 | 0.000000 |
| Fermata `H` | 1 | 0.010877 |

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
| Timing par pas | −0.015 aux pulses ; +0.0075 aux autres pas pairs ; 0 aux impairs |
| Dispersion | ±0.015 pas, soit ±1.30 ms au tempo nominal avant swing |
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

Source : `harmonium_lab/references/ornaments/quebecoise-jig.json` (DS-T10, `engine_proposal`), recalculé par
`measure_ornaments.py corpus`. Taux écrits = **planchers** (les transcriptions sous-notent) ; `rate_scale = 4` les relève
identiquement pour tous les types, valeur d'écoute à valider. Multiplicateurs d'arousal neutres : le corpus n'annote pas l'arousal.
Le taux par mesure (forme du corpus positionné, niveau du taux global) n'est repris que pour les types à ≥ 30 occurrences dans des parties de huit mesures reconnues.

| Type | Occurrences (positionnées) | /100 notes (plancher) | /100 notes × échelle | Taux par mesure | Durée porteuse min. (croches) |
|---|---:|---:|---:|---|---:|
| `grace_single` | 19 (7) | 0.207 | 0.827 | non | 1 |
| `grace_multiple` | 3 (0) | 0.033 | 0.131 | non | 1 |
| `roll` | 19 (17) | 0.207 | 0.827 | non | 3 |
| `turn` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |
| `triplet_repeated` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |
| `double_stop` | 7 (5) | 0.076 | 0.305 | non | 1 |
| `chord` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |
| `trill` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |

`passing` : note de passage de densité émotionnelle (ancien `dynamics.density_passing`), inchangée : courbe `[[0,0],[0.5,0],[1,1]]`, tenue ≥ 1,5 croche, sans tirage.

Effectifs faibles (≤ 19 occurrences par type) : taux global seulement, pas de taux par mesure.

Le triolet est donné en durée totale remplacée (le labo donne la première note après subdivision). `min_carrier_eighths` est le minimum observé quand l'effectif est faible ; au-delà de 30 occurrences, la plus petite durée dont le taux par note atteint 10 % du meilleur (les rouleaux du corpus sont sur des noires et des noires pointées, pas sur des croches). Schéma : [Ornements](../README.md#ornements).

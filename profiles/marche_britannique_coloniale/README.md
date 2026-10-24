# Profil de style : Marche britannique coloniale (fifre et tambour)

Style de garnison militaire britannique (vers 1760–1840) joué au **fifre et tambour**, sans guitare ni accords. Utile au jeu « 1837 » (soldats britanniques, garnison de Québec).

## Tradition et sources (Constitution V et VII)

- **Tradition** : Musique militaire britannique, régiments et garnisons d'Amérique du Nord, c. 1760–1840.
- **Corpus de référence** : 25 airs de marche du corpus `bascanada/harmonium_lab` issus de :
  - *Nottingham Music Database* (NMD)
  - *John Chambers AmRev* (AmRev, trillian)
  - *24 Country Dances for the Year 1809* (Cahusac, Londres)
- **Métrique retenue** : 2/2 (Alla Breve / Quick March), 8 croches par mesure (16 pas sur la grille de doubles-croches), pulse dansée à la blanche (2 pas cadencés par mesure : gauche-droite).
- **Tempo** : 108 BPM à la noire (valeur médiane observée), plage 90–126 BPM. Pas de swing (0.50, droit et militaire).

## Instrumentation

- **Fifre** : General MIDI 72 (Piccolo), tessiture 72–102 (D5 à F#7), centre mélodique à 84.
- **Tambour militaire** (General MIDI kit canal 9) :
  - Grosse caisse : note 36 sur piste Heel (temps forts, gauche-droite)
  - Caisse claire : note 38 sur piste Toe (battements d'ordonnance et roulements en 16ᵉ)
  - Caisse claire flam / coup accentué : note 40 sur piste Toe (conclusions cadentielles aux fins de phrase)
- **Harmonie** : Modèle `none` (aucune piste d'accords ni de basse). La mélodie reste tonale ; les cadences en m.4 et m.8 sont désormais pondérées plutôt qu'imposées.

## Banque de cellules mélodiques (`cells.toml`)

- 111 cellules au total, extraites des 16 airs en 2/2 et 4/4 :
  - `open` : 16 cellules
  - `run` : 30 cellules (top 30 parmi 43)
  - `arp` : 30 cellules (top 30 parmi 45)
  - `turn` : 4 cellules (seul rôle à moins de 15 cellules, ce qui reflète fidèlement la tradition militaire où les ornements en gruppetto sont très rares au profit des fanfares, arpèges et traits directs)
  - `cad_half` : 16 cellules
  - `cad_full` : 15 cellules
- Chaque entrée porte son poids d'occurrence observé (`weight`).

## Réaction aux 4 émotions (`dynamics.toml`)

- **Arousal** : accélère le tempo de marche (décalage jusqu'à +15 BPM) et renforce la dynamique du tambour (`feet_velocity`).
- **Density** : active le palier de roulements en doubles croches (16ᵉ) à la caisse claire (`density_feet`).
- **Tension** : renforce l'accentuation des frappes et des flams cadentiels (`strong_accent`, `tension_offbeat`).
- **Valence** : pilote la brillance et fait basculer le mode entre majeur (Ionien) et mineur militaire (Éolien) au prochain air (`color`, `parallel_modes`).

## Phrases probabilistes — DS-T9 / P3-T1

Source : `harmonium_lab/references/structure/britannique_coloniale-march.json`, champ `real`
(statistiques DS-T9, corpus et droits dans le manifeste du labo).
Les taux par mesure de `transformations` pilotent copies, séquences et fins variées ;
les déplacements utilisent les effectifs `sequence_shifts` de la même position.
Les repos toniques viennent de `phrases.rest_by_position` ; les autres longues
fins servent de proxy de demi-cadence. Voir [le schéma et les limites](../README.md#phrases--rôles-pondérés-motifs-et-cadences).

| Partie | Copie m.5←1 | Copie m.6←2 | Cadence tonique m.8 | Copie B:m.7←A:m.7 |
|---|---:|---:|---:|---:|
| A | 52.94 % | 29.41 % | 41.18 % | 0.00 % |
| B | 33.33 % | 16.67 % | 50.00 % | 47.06 % |

Poids arrondis sur 10 000 ; les mesures se tirent indépendamment.
Les taux conjoints m.5–6 et les structures de paires restent des mesures de
sortie, pas des garanties du profil. Banques de cellules inchangées.

## Secondes fins — P3-T2 (contrat 12)

`second_ending` utilise les effectifs de dernière note **écrite**, avant toute
complétion par une levée : tonique (`CadTonic`), quinte (`CadFifth`), autre
(`CadOther`). Source : `harmonium_lab/corpus/britannique_coloniale/tunes/march/*.abc`,
analysé par `trad.form.analyze_form` avec la commande
[`measure_endings.py`](../../specs/011-phase3-phrase/measure_endings.py).
Les poids du profil sont les effectifs des paires de passes de huit mesures,
séparément pour A (**14** occasions) et B (**10** occasions), sans arrondi.

La variante s'applique à A2/B2 et reste identique à chaque tour ; les sept
autres mesures ne changent pas entre passes. Les premières fins conservent
les réglages DS-T9 ci-dessus. Les chiffres réels et les limites de mesure
sont dans [le rapport de phase](../../specs/011-phase3-phrase/plan.md#p3-t2--fins-de-reprise).

Effectifs faibles : ces poids décrivent ce corpus, pas une certitude sur
l’ensemble de la tradition.

## Espace et mixage V1

Plein air fifre-et-tambour : taille 0,12, amorti 0,80, niveau 0,10 et envoi 0,04 évitent une salle artificielle ; fifre et tambour près du centre.
Tambour −6 dB de canal, bus +15 dB et présence +0,5 dB gardent le fifre lisible ; Chords/Bass/Toe restent absents, proposition de rendu et non mesure historique.

## Interprétation V2

Source : `harmonium_lab/references/ornaments/britannique_coloniale-march.json`, DS-T10b
(labo main `f8e0cbc`, corpus et droits dans ses manifestes).
25 airs, 2026 attaques porteuses écrites ; reprises non multipliées.

| Marque écrite | Occurrences | Par 100 notes |
|---|---:|---:|
| Liaison `(…)` | 24 | 1.184600 |
| Tenue liée `-` | 25 | 1.233959 |
| Détaché `.` | 0 | 0.000000 |
| Accent explicite | 0 | 0.000000 |
| Fermata `H` | 1 | 0.049358 |

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
| Timing par pas | −0.01 aux pulses ; +0.005 aux autres pas pairs ; 0 aux impairs |
| Dispersion | ±0.01 pas, soit ±1.39 ms au tempo nominal avant swing |
| Courbe de phrase | −2 → +2 → −3 unités MIDI ; bornes 30..127 |
| Respiration | 12 % de la dernière durée avant chaque fin de phrase |
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

Source : `harmonium_lab/references/ornaments/britannique_coloniale-march.json` (DS-T10, `engine_proposal`), recalculé par
`measure_ornaments.py corpus`. Taux écrits = **planchers** (les transcriptions sous-notent) ; `rate_scale = 4` les relève
identiquement pour tous les types, valeur d'écoute à valider. Multiplicateurs d'arousal neutres : le corpus n'annote pas l'arousal.
Le taux par mesure (forme du corpus positionné, niveau du taux global) n'est repris que pour les types à ≥ 30 occurrences dans des parties de huit mesures reconnues.

| Type | Occurrences (positionnées) | /100 notes (plancher) | /100 notes × échelle | Taux par mesure | Durée porteuse min. (croches) |
|---|---:|---:|---:|---|---:|
| `grace_single` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |
| `grace_multiple` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |
| `roll` | 5 (5) | 0.247 | 0.987 | non | 2 |
| `turn` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |
| `triplet_repeated` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |
| `double_stop` | 2 (2) | 0.099 | — | — | fifre monophonique : désactivé |
| `chord` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |
| `trill` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |

Pas de note de passage de densité : l'ancienne courbe `density_passing` de ce profil était nulle.

Fifre : monophonique, donc `double_stop` (2 occurrences) désactivé malgré le corpus ; 5 rouleaux seulement.

Le triolet est donné en durée totale remplacée (le labo donne la première note après subdivision). `min_carrier_eighths` est le minimum observé quand l'effectif est faible ; au-delà de 30 occurrences, la plus petite durée dont le taux par note atteint 10 % du meilleur (les rouleaux du corpus sont sur des noires et des noires pointées, pas sur des croches). Schéma : [Ornements](../README.md#ornements).

## Forme longue V7b

`set.toml` : `length = [{ tunes = 1, weight = 1 }]`. Chaque air est son propre set : il est annoncé en entier par le
décompte de tambour de `[intro]` (rôle `percussion`, 1 mesure), comme avant V7b (`intro_each_tune`). Aucun facteur de
tonalité (pas de logique d'enchaînement mesurable : aucun set britannique n'existe en accès libre). Schéma :
[profiles/README.md](../README.md#forme-longue-set).

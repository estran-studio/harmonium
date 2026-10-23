# Profil de style : Gigue britannique coloniale (violon et accompagnement léger)

Élargissement de la tradition britannique coloniale (vers 1760–1840, garnisons et bals d'officiers d'Amérique du Nord) avec la danse ternaire la plus représentée du corpus : la **jig** (6/8, 365 airs).

Conformément au principe V de la constitution, ce profil est défini **exclusivement en données** (aucun fichier `.rs` modifié).

---

## 1. Tradition et sources (Constitution V et VII)

- **Tradition** : Musique britannique coloniale, régiments, garnisons (Québec, Halifax, Montréal) et bals d'officiers en Amérique du Nord britannique, c. 1760–1840.
- **Corpus de référence** : 365 airs de jig du corpus `bascanada/harmonium_lab` (`corpus/britannique_coloniale/tunes/jig/`, identifiant `2026-10-22+1025`) issus de :
  - *Nottingham Music Database* (NMD)
  - *John Chambers AmRev* (AmRev, trillian)
  - *24 Country Dances for the Year 1809* (Cahusac, Londres)
- **Métrique retenue** : 6/8 composé, 12 pas sur la grille (6 unités de croche par mesure), pulse dansée à la noire pointée (2 temps par mesure : pas 0 et 6).
- **Tempo** : 156.0 BPM à la noire (soit 104 BPM à la pulse de noire pointée dansée, médiane observée dans le corpus), plage 144–168 BPM. Tempo mesuré et régulier de contredanse de salon.

---

## 2. Tableau comparatif : Britannique vs Québécois

Le type « gigue » existant dans les deux traditions, le tableau ci-dessous détaille les choix de données étayés par le corpus :

| Dimension / Champ | Gigue québécoise (`gigue_quebecoise`) | Gigue britannique coloniale (`gigue_britannique_coloniale`) | Justification et source des données |
|---|---|---|---|
| **Tradition et contexte** | Veillées villageoises, violoneux du terroir québécois (XIXᵉ–XXᵉ s.) | Bals d'officiers, garnisons militaires et salons coloniaux (c. 1760–1840) | Contextes historiques distincts : énergie rurale et percussive au Québec vs tenue et élégance de salon dans les garnisons britanniques. |
| **Corpus de référence** | 88 gigues québécoises (`2026-10-19+764`) | 365 gigues britanniques (`2026-10-22+1025`) | Corpus de référence documenté dans le labo (Constitution VII). |
| **Tempo dansé** (`default_bpm`) | 172.5 BPM à la noire (115 BPM à la noire pointée, plage 160–185) | **156.0 BPM à la noire** (104 BPM à la noire pointée, plage 144–168) | Médiane observée à 104 BPM dans `tempo.json`. La gigue britannique de contredanse se joue à un tempo plus mesuré que la gigue québécoise très vive. |
| **Accompagnement harmonique** (`accompaniment`) | **Chop québécois** : coups percussifs secs (durée 1 pas) sur les contretemps 4 et 10 | **Accompagnement classique léger** : accords tenus (durée 3 pas) posés sur les temps 0 et 6 avec la basse | Les garnisons et salons n'utilisent pas le chop percussif québécois : basse continue légère et accords d'harmonie posés sur les temps forts (0 et 6). |
| **Vélocités d'accompagnement** | Basse 78, chop guitare 72 | **Basse 70, accords 60** | Accompagnement plus discret et aéré, laissant le violon en avant-plan sans saturation percussive. |
| **Podorythmie** (`percussion`) | **Talon et pointe actifs** : 3 densités de podorythmie québécoise virtuose | **Aucune podorythmie** : table vide, pistes Heel/Toe désactivées dans l'arrangement | La podorythmie est une spécificité canadienne-française/acadienne absente de la danse d'officiers britannique. |
| **Tonalités reines** (`key_modes`) | Ré (D) 33 %, Sol (G) 33 %, La (A) 22 %, modes mixolydiens présents | **Ré (D) 32 %, Sol (G) 30 %**, La (A) 13 %, Do (C) 7 %, La mineur (Am) 5 % | Le corpus britannique compte une présence notable du ton de Do majeur (C) et du mineur éolien (Am, Em) absents de la gigue québécoise. |
| **Décalage registre Partie B** (`melody_center_offset`) | `+2` demi-tons | **`+1` demi-ton** | `register__b_minus_a_mean_pitch` au corpus : moyenne de 1.33 demi-ton (médiane 0.60) en britannique, contre 2.0 en québécois. La strain B reste proche du registre de la strain A. |
| **Poids des rôles mélodiques** (`melody.params`) | `run = 4`, `arp = 3`, `turn = 3` | **`run = 4`, `arp = 4`, `turn = 1`** | Minage des cellules : 2 384 arpèges et 2 363 traits de run (ratio 1:1), gruppettos (`turn`) très minoritaires (201 occurrences, ratio ~1:12). |
| **Levée d'intro** (`pickup_eighths`) | 2 croches | **1 croche** | `pickups_eighths` du corpus : 234 airs sur 352 (66.5 %) débutent par une levée d'une seule croche (anacrouse brève). |
| **Forme** | AABB (32 mesures, A=8, B=8) | **AABB** (32 mesures, A=8, B=8) | Identique : structure binaire classique confirmée par 122 airs en AABB et 166 airs en 8/8/8/8. |

---

## 3. Banque de cellules mélodiques (`cells.toml`)

- **180 cellules au total** (top 30 par rôle), extraites des 352 airs en 6/8 avec validation stricte du moteur :
  - `open` : 30 cellules (poids 10 à 4)
  - `run` : 30 cellules (poids 11 à 4)
  - `arp` : 30 cellules (poids 10 à 4)
  - `turn` : 30 cellules (poids 5 à 2)
  - `cad_half` : 30 cellules (poids 11 à 4, 24 respectant la terminaison cadentielle 2/5/7)
  - `cad_full` : 30 cellules (poids 10 à 5, 26 respectant la résolution sur la tonique)
- Chaque cellule porte son poids d'occurrence réel (`weight`) issu du minage du corpus.

---

## 4. Instrumentation

- **Violon** (Melody, canal 0) : Programme GM 40 (Violin), tessiture 55–88 (Sol grave G3 à Mi aigu E6), centre mélodique à 71.
- **Accompagnement** (Chords, canal 1) : Programme GM 24 (Guitare acoustique nylon), accords légers posés sur les temps 0 et 6.
- **Basse acoustique** (Bass, canal 2) : Programme GM 32 (Acoustic Bass), fondamentale/quinte sur les pulsations 0 et 6.
- **Percussion** : Pistes Heel et Toe désactivées dans l'arrangement (pas de podorythmie).

## Phrases probabilistes — DS-T9 / P3-T1

Source : `harmonium_lab/references/structure/britannique_coloniale-jig.json`, champ `real`
(statistiques DS-T9, corpus et droits dans le manifeste du labo).
Les taux par mesure de `transformations` pilotent copies, séquences et fins variées ;
les déplacements utilisent les effectifs `sequence_shifts` de la même position.
Les repos toniques viennent de `phrases.rest_by_position` ; les autres longues
fins servent de proxy de demi-cadence. Voir [le schéma et les limites](../README.md#phrases--rôles-pondérés-motifs-et-cadences).

| Partie | Copie m.5←1 | Copie m.6←2 | Cadence tonique m.8 | Copie B:m.7←A:m.7 |
|---|---:|---:|---:|---:|
| A | 73.95 % | 53.38 % | 12.86 % | 0.00 % |
| B | 48.66 % | 32.89 % | 17.79 % | 35.03 % |

Poids arrondis sur 10 000 ; les mesures se tirent indépendamment.
Les taux conjoints m.5–6 et les structures de paires restent des mesures de
sortie, pas des garanties du profil. Banques de cellules inchangées.

## Secondes fins — P3-T2 (contrat 12)

`second_ending` utilise les effectifs de dernière note **écrite**, avant toute
complétion par une levée : tonique (`CadTonic`), quinte (`CadFifth`), autre
(`CadOther`). Source : `harmonium_lab/corpus/britannique_coloniale/tunes/jig/*.abc`,
analysé par `trad.form.analyze_form` avec la commande
[`measure_endings.py`](../../specs/011-phase3-phrase/measure_endings.py).
Les poids du profil sont les effectifs des paires de passes de huit mesures,
séparément pour A (**260** occasions) et B (**136** occasions), sans arrondi.

La variante s'applique à A2/B2 et reste identique à chaque tour ; les sept
autres mesures ne changent pas entre passes. Les premières fins conservent
les réglages DS-T9 ci-dessus. Les chiffres réels et les limites de mesure
sont dans [le rapport de phase](../../specs/011-phase3-phrase/plan.md#p3-t2--fins-de-reprise).

## Valeurs DS-T9b (parseur corrigé)

Les champs `roles`, `second_ending` et `rhythm` des parties A et B sont les extraits
proposés par `harmonium_lab/specs/001-trad-dataset/ds-t9b-profils-proposes.md`,
appliqués tels quels. Ils dérivent des références régénérées
(`references/structure/britannique_coloniale-jig.json`) après correction du parseur ABC
et des secondes fins mesurées par
[`measure_endings.py`](../../specs/011-phase3-phrase/measure_endings.py), relancé sans
modification. La formule est celle des valeurs précédentes (rejouée sur les anciennes
références, elle les reproduit à l'identique). Les banques `cells.toml` ne sont pas
régénérées.

## Espace et mixage V1

Salle de danse modeste : taille 0,48, amorti 0,50, largeur 0,80 et niveau 0,30 ; violon légèrement à gauche, guitare à droite, basse centrée.
Guitare −5 dB, basse −8 dB et bus +16 dB gardent le violon devant ; pieds non arrangés (l'accent final peut en jouer), espace proposé plutôt que vérité historique.

## Interprétation V2

Source : `harmonium_lab/references/ornaments/britannique_coloniale-jig.json`, DS-T10b
(labo main `f8e0cbc`, corpus et droits dans ses manifestes).
365 airs, 37036 attaques porteuses écrites ; reprises non multipliées.

| Marque écrite | Occurrences | Par 100 notes |
|---|---:|---:|
| Liaison `(…)` | 34 | 0.091803 |
| Tenue liée `-` | 69 | 0.186305 |
| Détaché `.` | 11 | 0.029701 |
| Accent explicite | 0 | 0.000000 |
| Fermata `H` | 0 | 0.000000 |

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
| Dispersion | ±0.015 pas, soit ±1.44 ms au tempo nominal avant swing |
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

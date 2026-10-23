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
| A | 74.33 % | 52.67 % | 13.33 % | 0.00 % |
| B | 49.65 % | 33.22 % | 18.18 % | 34.52 % |

Poids arrondis sur 10 000 ; les mesures se tirent indépendamment.
Les taux conjoints m.5–6 et les structures de paires restent des mesures de
sortie, pas des garanties du profil. Banques de cellules inchangées.

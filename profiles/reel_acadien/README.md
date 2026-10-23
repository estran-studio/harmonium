# Profil de style : Reel acadien (violon et podorythmie)

Deuxième tradition francophone du moteur Harmonium, le reel **acadien** représente la tradition musicale vivante des Maritimes : Nouveau-Brunswick, Île-du-Prince-Édouard, Cap-Breton / Chéticamp et Îles-de-la-Madeleine.

Conformément au principe V de la constitution, ce profil est défini **exclusivement en données** (aucun fichier `.rs` modifié).

---

## 1. Tradition et sources (Constitution V et VII)

- **Tradition** : Acadie et provinces maritimes (XIXᵉ–XXᵉ siècle), violon traditionnel et podorythmie.
- **Corpus de référence** : 169 reels acadiens du corpus `harmonium_lab` (`corpus/acadienne/`, identifiant `2026-10-22+270`), recensés dans `corpus/sources.md` :
  - **The Session (Acadie)** : 167 airs (licence ODbL) qualifiés par enregistrement d'artistes majeurs de la tradition acadienne :
    - *Barachois* (Albert Arsenault, Louise Arsenault, Chuck Arsenault, Hélène Bergeron — Î.-P.-É.) ;
    - *Vishtèn* (Pastelle LeBlanc, Emmanuelle LeBlanc, Pascal Miousse — Î.-P.-É. / Îles-de-la-Madeleine) ;
    - *Joe Cormier* (Chéticamp, Cap-Breton — violoneux acadien émérite, National Heritage Fellow) ;
    - *The Robichaud Brothers* (Gerry et Armand Robichaud — Saint-Paul, Nouveau-Brunswick) ;
    - *Eddy Arsenault* (Baie-Egmont, Î.-P.-É.) ;
    - *Donny LeBlanc* (Margaree / Chéticamp) ;
    - *Suroît* (Îles-de-la-Madeleine) ;
    - *Marc Boudreau* (Cap-Breton) ;
    - *Mario Landry* (Nouveau-Brunswick).
  - **John Chambers (Collection Canada)** : 2 airs (*Avila LeBlanc #1*, *Avila LeBlanc #2*), violoneux acadien de Bouctouche (Nouveau-Brunswick).
- **Métrique retenue** : 2/2 (Alla Breve), 16 pas de double-croche par mesure (8 unités de croche), pulse dansée à la blanche (2 temps par mesure : pas 0 et 8).

---

## 2. Tableau comparatif : Acadien vs Québécois

Le profil acadien reprend du québécois ce que les données statistiques ne contredisent pas (forme AABB 32 mesures, 16 pas par mesure, accompagnement basse/chops, podorythmie talon/pointe) et adapte ce que les données distinguent nettement :

| Dimension / Champ | Reel québécois (`reel_quebecois`) | Reel acadien (`reel_acadien`) | Justification et source des données |
|---|---|---|---|
| **Tonalité reine** | Ré (D) à 40.1 % (230/574) | **La (A)** à 33.7 % (57/169) | Corpus acadien : La est la tonique dominante (A maj 33, A dor 13, A mix 9), liée au jeu de violon sur chanterelle et cordes à vide La/Mi. Ré vient en second (38 airs). |
| **Part des modes doriens** | 6.4 % (37/574 airs) | **14.2 %** (24/169 airs) | Le dorien est plus de deux fois plus fréquent en Acadie (influence maritime/écossaise du Cap-Breton et des Maritimes). |
| **Décalage registre Partie B** (`melody_center_offset`) | `+2` demi-tons | **`+3` demi-tons** | `register__b_minus_a_mean_pitch` au corpus : moyenne de 2.93 demi-tons (médiane 2.66) en acadien, contre 1.52 demi-tons (médiane 1.27) en québécois. La strain B acadienne monte nettement plus haut. |
| **Poids des rôles mélodiques** (`melody.params`) | `run = 4`, `arp = 3`, `turn = 3` | **`arp = 5`, `run = 3`, `turn = 3`** | Minage des cellules : `arp` compte 781 occurrences valides contre 444 pour `run` et 418 pour `turn` (ratio arp/run de 1.76:1). Le violon acadien privilégie les grands traits arpégés et sauts de cordes. |
| **Ambitus moyen** (`register__ambitus`) | 18.87 demi-tons (std 3.79) | **20.01 demi-tons** (std 4.21) | Légèrement plus large en Acadie (q90 à 26 demi-tons), reflétant l'amplitude entre cordes graves et positions aiguës. |
| **Sous-tonique bVII majeure** (`harmony.candidates`) | Mixolydien, dorien et éolien seulement | **Présente aussi en ionien** ; `tie_priority` inclut `BVII` avant `IV` | Fréquence des accords : VII représente 12.1 % en Acadie (4x plus qu'au Québec à 3.0 %). L'alternance I - bVII est emblématique du jeu modal maritime. |
| **Modes parallèles de dynamique** (`parallel_modes`) | Ionian -> Aeolian (poids 3), Dorian (poids 1) | **Ionian -> Dorian (poids 3)**, Aeolian (poids 2) | En réaction à une valence sombre, bascule préférentielle vers le dorien (mode mineur acadien par excellence). |
| **Tempo** (`default_bpm`) | 230.0 BPM (plage 220–240) | **228.0 BPM** (plage 215–238) | Vitesse vive de danse carrée acadienne, légèrement plus posée pour l'articulation des arpèges. |
| **Continuité des croches** (`durations`) | 87.0 % de croches (durée 1) | **91.0 % de croches** (durée 1) | Débit de croches encore plus soutenu et continu dans le violon acadien (flow ininterrompu). |
| **Forme** | AABB (32 mesures, A=8, B=8) | **AABB** (32 mesures, A=8, B=8) | Identique : schéma majoritaire confirmé par 70 airs en AABB et 56 airs en 8/8/8/8. |
| **Podorythmie** | Talon pas 0, 8 ; pointe pas pairs | **Talon pas 0, 8 ; pointe pas pairs** | Identique : pratique commune violon et pieds assis (D9). |

---

## 3. Banque de cellules mélodiques (`cells.toml`)

- **180 cellules au total** (top 30 par rôle), extraites des 169 airs acadiens avec validation stricte du moteur (invariants D5) :
  - `open` : 30 cellules (poids 4 à 2)
  - `run` : 30 cellules (poids 8 à 4)
  - `arp` : 30 cellules (poids 10 à 4)
  - `turn` : 30 cellules (poids 6 à 3)
  - `cad_half` : 30 cellules (poids 6 à 2, 15 respectant la terminaison cadentielle 2/5/7)
  - `cad_full` : 30 cellules (poids 11 à 2, 13 respectant la résolution sur la tonique)
- Chaque cellule porte son poids d'occurrence réel (`weight = count`) issu du minage (`cells.weights.json`).

---

## 4. Instrumentation et podorythmie

- **Violon** (Melody, canal 0) : Programme GM 40 (Violin), tessiture 55–88 (Sol grave G3 à Mi aigu E6), centre mélodique à 71.
- **Accompagnement** (Chords, canal 1) : Programme GM 24 (Guitare acoustique nylon), chops sur les contretemps (pas 4 et 12).
- **Basse acoustique** (Bass, canal 2) : Programme GM 32 (Acoustic Bass), fondamentale/quinte sur les temps de blanche (pas 0 et 8).
- **Podorythmie** (Heel / Toe, canal 9 kit GM ou banque de pieds `harmonium_trad.sf2`) :
  - Talon (note 36) : pas 0 et 8 (densité 0), pas 0, 4, 8, 12 (densité 1 et 2).
  - Pointe (note 37) : contretemps 2, 6, 10, 14 (densité 1), doublée en doubles-croches (densité 2).

## Phrases probabilistes — DS-T9 / P3-T1

Source : `harmonium_lab/references/structure/acadienne-reel.json`, champ `real`
(statistiques DS-T9, corpus et droits dans le manifeste du labo).
Les taux par mesure de `transformations` pilotent copies, séquences et fins variées ;
les déplacements utilisent les effectifs `sequence_shifts` de la même position.
Les repos toniques viennent de `phrases.rest_by_position` ; les autres longues
fins servent de proxy de demi-cadence. Voir [le schéma et les limites](../README.md#phrases--rôles-pondérés-motifs-et-cadences).

| Partie | Copie m.5←1 | Copie m.6←2 | Cadence tonique m.8 | Copie B:m.7←A:m.7 |
|---|---:|---:|---:|---:|
| A | 77.17 % | 64.13 % | 23.91 % | 0.00 % |
| B | 61.90 % | 44.76 % | 24.76 % | 35.23 % |

Poids arrondis sur 10 000 ; les mesures se tirent indépendamment.
Les taux conjoints m.5–6 et les structures de paires restent des mesures de
sortie, pas des garanties du profil. Banques de cellules inchangées.

## Secondes fins — P3-T2 (contrat 12)

`second_ending` utilise les effectifs de dernière note **écrite**, avant toute
complétion par une levée : tonique (`CadTonic`), quinte (`CadFifth`), autre
(`CadOther`). Source : `harmonium_lab/corpus/acadienne/tunes/reel/*.abc`,
analysé par `trad.form.analyze_form` avec la commande
[`measure_endings.py`](../../specs/011-phase3-phrase/measure_endings.py).
Les poids du profil sont les effectifs des paires de passes de huit mesures,
séparément pour A (**72** occasions) et B (**62** occasions), sans arrondi.

La variante s'applique à A2/B2 et reste identique à chaque tour ; les sept
autres mesures ne changent pas entre passes. Les premières fins conservent
les réglages DS-T9 ci-dessus. Les chiffres réels et les limites de mesure
sont dans [le rapport de phase](../../specs/011-phase3-phrase/plan.md#p3-t2--fins-de-reprise).

## Espace et mixage V1

Petite salle communautaire : taille 0,42, amorti 0,55, niveau 0,30 ; violon légèrement à gauche, guitare à droite, basse centrale et pieds resserrés.
Guitare −5 dB, basse −8 dB, pieds −6 dB de canal et bus +15,5 dB gardent la ligne de violon devant ; espace proposé pour l'écoute, pas une estimation du corpus.

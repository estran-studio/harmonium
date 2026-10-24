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

## Interprétation V2

Source : `harmonium_lab/references/ornaments/acadienne-reel.json`, DS-T10b
(labo main `f8e0cbc`, corpus et droits dans ses manifestes).
169 airs, 21640 attaques porteuses écrites ; reprises non multipliées.

| Marque écrite | Occurrences | Par 100 notes |
|---|---:|---:|
| Liaison `(…)` | 100 | 0.462107 |
| Tenue liée `-` | 11 | 0.050832 |
| Détaché `.` | 22 | 0.101664 |
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
| Timing par pas | −0.025 aux pulses ; +0.0125 aux autres pas pairs ; 0 aux impairs |
| Dispersion | ±0.025 pas, soit ±1.64 ms au tempo nominal avant swing |
| Courbe de phrase | −2 → +2 → −3 unités MIDI ; bornes 30..127 |
| Respiration | 10 % de la dernière durée avant chaque fin de phrase |
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

Source : `harmonium_lab/references/ornaments/acadienne-reel.json` (DS-T10, `engine_proposal`), recalculé par
`measure_ornaments.py corpus`. Taux écrits = **planchers** (les transcriptions sous-notent) ; `rate_scale = 4` les relève
identiquement pour tous les types, valeur d'écoute à valider. Multiplicateurs d'arousal neutres : le corpus n'annote pas l'arousal.
Le taux par mesure (forme du corpus positionné, niveau du taux global) n'est repris que pour les types à ≥ 30 occurrences dans des parties de huit mesures reconnues.

| Type | Occurrences (positionnées) | /100 notes (plancher) | /100 notes × échelle | Taux par mesure | Durée porteuse min. (croches) |
|---|---:|---:|---:|---|---:|
| `grace_single` | 59 (37) | 0.273 | 1.091 | oui | 1 |
| `grace_multiple` | 22 (17) | 0.102 | 0.407 | non | 1 |
| `roll` | 190 (104) | 0.878 | 3.512 | oui | 2 (observé 1/3) |
| `turn` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |
| `triplet_repeated` | 124 (55) | 0.573 | 2.292 | oui | 1 (3 × 1/3) |
| `double_stop` | 27 (1) | 0.125 | 0.499 | non | 1 |
| `chord` | 0 (0) | 0.000 | — | — | aucune occurrence : désactivé |
| `trill` | 6 (0) | 0.028 | 0.111 | non | 2 |

`passing` : note de passage de densité émotionnelle (ancien `dynamics.density_passing`), inchangée : courbe `[[0,0],[0.5,0],[1,1]]`, tenue ≥ 1,5 croche, sans tirage.

Le triolet de notes répétées (124) est plus fréquent que dans les autres reels.

Le triolet est donné en durée totale remplacée (le labo donne la première note après subdivision). `min_carrier_eighths` est le minimum observé quand l'effectif est faible ; au-delà de 30 occurrences, la plus petite durée dont le taux par note atteint 10 % du meilleur (les rouleaux du corpus sont sur des noires et des noires pointées, pas sur des croches). Schéma : [Ornements](../README.md#ornements).

## Forme longue V7b

`set.toml` : sets de 2 à 5 airs (poids 45·44·7·4, The Session), intro complète et entrées de pistes au premier air
de chaque set seulement. Racine carrée des rapports mesurés chez les artistes acadiens enregistrés (205 transitions, dominées par le Cap-Breton : même tonalité ×3,45 mesuré, ×1,86 retenu ; la dorien et mi dorien restent candidats).

`same` 1,86 · `parallel` 1,42 · `relative` 0,60 · `fifth_up` 0,58 · `fourth_up` 0,66 · `step_up` 0,76 · `step_down` 0,66 · `other` 0,74 · `last_dark` 0,80.

`last_dark` 0,80 : le dernier air d'un set finit plus souvent en majeur (28,9 % de modes mineurs contre 36,2 % aux deux premiers airs de 19 771 sets de 3 reels). Valeurs de répertoire, pas de la pratique du style : à régler à l'écoute. Schéma et effet à l'oreille :
[profiles/README.md](../README.md#forme-longue-set).

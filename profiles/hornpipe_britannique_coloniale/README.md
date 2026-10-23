# Profil de style : Hornpipe britannique colonial (violon et rythme pointé)

Danse emblématique du monde maritime et des garnisons britanniques (vers 1760–1840), le **hornpipe** se distingue par sa mesure à 4 temps (4/4), son **rythme pointé caractéristique** (« croches pointées » avec fort swing), son tempo modéré et sa **cadence finale typique en trois noires accentuées**.

Conformément au principe V de la constitution, ce profil est défini **exclusivement en données** (aucun fichier `.rs` modifié).

---

## 1. Tradition et sources (Constitution V et VII)

- **Tradition** : Musique britannique coloniale, matelots, régiments côtiers et bals de garnison en Amérique du Nord britannique (Québec, Halifax, Boston/AmRev), c. 1760–1840.
- **Corpus de référence** : 78 airs de hornpipe du corpus `bascanada/harmonium_lab` (`corpus/britannique_coloniale/tunes/hornpipe/`, identifiant `2026-10-22+1025`) issus de :
  - *Nottingham Music Database* (NMD)
  - *John Chambers AmRev* (AmRev, trillian)
  - *24 Country Dances for the Year 1809* (Cahusac, Londres)
- **Métrique retenue** : 4/4 (73 airs sur 77 dans le corpus), 16 pas sur la grille de double-croches (8 unités de croche par mesure), pulse à la noire (4 temps par mesure : pas 0, 4, 8, 12).
- **Tempo** : 92.0 BPM à la noire (plage 80–104 BPM). Nettement plus lent qu'un reel (230 BPM à la noire, soit 115 à la blanche), permettant l'articulation soignée des croches pointées et des sauts d'archet.

---

## 2. Rythme pointé (Swing) et justification musicale

- **Swing ratio** : `default_swing_ratio = 0.65` (dans la plage 0.62–0.67 documentée).
- **Justification** :
  - Dans la tradition britannique et irlandaise, le hornpipe n'est pas joué en croches égales (« straight », swing 0.50), mais en **croches pointées** (notes inégales).
  - Un ratio de **0.65** établit un rapport de durée d'environ **1.86:1** entre la croche impaire (sur le temps) et la croche paire (en l'air), se situant idéalement entre le swing ternaire jazz / shuffle (2:1, soit ~0.667) et la notation formelle croche pointée / double-croche (3:1, soit 0.75).
  - Ce swing prononcé confère au hornpipe son rebond caractéristique (« lilt » / « bounce »), bien plus marqué que le swing fluide du reel acadien (0.56) ou la rigueur métronomique de la marche (0.50).

---

## 3. Cadence finale typique : les trois noires accentuées

Le corpus de référence confirme sans ambiguïté la formule de conclusion traditionnelle du hornpipe :
- Sur la 8ᵉ mesure (cadence complète `CadFull`), les airs de hornpipe terminent typiquement par **trois noires accentuées** sur les temps 1, 2 et 3 (unités 0, 2 et 4 de la mesure), suivies d'une tenue ou d'une levée vers la reprise.
- Exemples observés directement dans le corpus :
  - `8 - 10 - 8 - 5 -` (octave, tierce supérieure, octave, dominante)
  - `1 - 1 - 1 - 5, -` (trois toniques martelées puis levée sur la quinte grave)
  - `1 - 3 - 1 - 5 -` (tonique, tierce, tonique)
  - `3 - 1 - 1 - 5 -` (tierce, tonique, tonique)
  - `8 - 10 - 8 - 7 8` (trois noires puis résolution conjointe)
  - `8 - 10 - 8 - - -` (trois noires puis suspension du 4ᵉ temps)
- La banque `cells.toml` retient en priorité ces cellules cadentielles idiomatiques validées par le moteur.

---

## 4. Tableau comparatif : Hornpipe britannique vs Reel (traditionnel)

| Dimension / Champ | Reel québécois (`reel_quebecois`) | Hornpipe britannique (`hornpipe_britannique_coloniale`) | Justification et source des données |
|---|---|---|---|
| **Métrique et pulse** | 2/2 (Alla Breve), pulse à la blanche (2 temps de 8 pas) | **4/4**, pulse à la noire (**4 temps de 4 pas**) | 73 airs sur 77 sont notés en 4/4 dans le corpus de hornpipes britanniques. |
| **Swing (rythme pointé)** | Swing léger (0.56, quasi-égal) | **Fort swing pointé : 0.65** (~1.86:1) | Caractéristique fondamentale du hornpipe britannique (« dotted hornpipe »). |
| **Tempo à la noire** | 230.0 BPM (vif et continu) | **92.0 BPM** (plage 80–104) | Tempo de danse beaucoup plus lent, indispensable pour articuler le rythme pointé. |
| **Cadence finale (m.8)** | Type pondéré depuis DS-T9 | Type pondéré ; les cellules cadentielles conservent leurs tenues quand une cadence tonique est tirée | La formule en trois noires de la banque n'est plus imposée à chaque partie. |
| **Levée d'intro** (`pickup_eighths`) | 2 croches (1 temps de blanche/demi-mesure en 2/2) | **2 croches = 1 noire** (71 airs sur 77 ont une anacrouse d'une noire) | `pickups_eighths` du labo : 92.2 % des hornpipes démarrent sur le 4ᵉ temps. |
| **Accompagnement** | Chop guitare percussif (durée 1) sur contretemps 4 et 12 | **Accords simples soutenus** (durée 4) sur temps 1 et 3 (0 et 8) | Accompagnement mesuré sans contretemps syncopé. |
| **Podorythmie** | Podorythmie québécoise omniprésente | **Aucune podorythmie** | Pratique absente des danses militaires et maritimes d'officiers britanniques. |
| **Tonalités reines** | Ré majeur (40 %), Sol majeur (33 %) | **Sol majeur (44 %)**, Ré majeur (36 %), La majeur (13 %) | Sol majeur (G) domine nettement le répertoire de hornpipe pour violon. |

---

## 5. Instrumentation

- **Violon** (Melody, canal 0) : Programme GM 40 (Violin), tessiture 55–88 (Sol grave G3 à Mi aigu E6), centre mélodique à 72.
- **Accompagnement** (Chords, canal 1) : Programme GM 24 (Guitare acoustique nylon), accords tenus de noire (durée 4 pas) sur les temps 0 et 8.
- **Basse acoustique** (Bass, canal 2) : Programme GM 32 (Acoustic Bass), fondamentale/quinte sur les temps 0 et 8.
- **Percussion** : Pistes Heel et Toe désactivées dans l'arrangement (pas de podorythmie).

## Phrases probabilistes — DS-T9 / P3-T1

Source : `harmonium_lab/references/structure/britannique_coloniale-hornpipe.json`, champ `real`
(statistiques DS-T9, corpus et droits dans le manifeste du labo).
Les taux par mesure de `transformations` pilotent copies, séquences et fins variées ;
les déplacements utilisent les effectifs `sequence_shifts` de la même position.
Les repos toniques viennent de `phrases.rest_by_position` ; les autres longues
fins servent de proxy de demi-cadence. Voir [le schéma et les limites](../README.md#phrases--rôles-pondérés-motifs-et-cadences).

| Partie | Copie m.5←1 | Copie m.6←2 | Cadence tonique m.8 | Copie B:m.7←A:m.7 |
|---|---:|---:|---:|---:|
| A | 84.75 % | 74.58 % | 6.78 % | 0.00 % |
| B | 44.07 % | 28.81 % | 8.47 % | 42.11 % |

Poids arrondis sur 10 000 ; les mesures se tirent indépendamment.
Les taux conjoints m.5–6 et les structures de paires restent des mesures de
sortie, pas des garanties du profil. Banques de cellules inchangées.

## Secondes fins — P3-T2 (contrat 12)

`second_ending` utilise les effectifs de dernière note **écrite**, avant toute
complétion par une levée : tonique (`CadTonic`), quinte (`CadFifth`), autre
(`CadOther`). Source : `harmonium_lab/corpus/britannique_coloniale/tunes/hornpipe/*.abc`,
analysé par `trad.form.analyze_form` avec la commande
[`measure_endings.py`](../../specs/011-phase3-phrase/measure_endings.py).
Les poids du profil sont les effectifs des paires de passes de huit mesures,
séparément pour A (**51** occasions) et B (**12** occasions), sans arrondi.

La variante s'applique à A2/B2 et reste identique à chaque tour ; les sept
autres mesures ne changent pas entre passes. Les premières fins conservent
les réglages DS-T9 ci-dessus. Les chiffres réels et les limites de mesure
sont dans [le rapport de phase](../../specs/011-phase3-phrase/plan.md#p3-t2--fins-de-reprise).

Effectifs faibles : ces poids décrivent ce corpus, pas une certitude sur
l’ensemble de la tradition.

## Espace et mixage V1

Salle de danse assez courte : taille 0,40, amorti 0,55 et niveau 0,28 pour garder les attaques pointées nettes ; violon légèrement à gauche, guitare à droite, basse au centre.
Guitare −5 dB, basse −8 dB et bus +13,5 dB donnent la priorité à la ligne ; pieds non arrangés (hors accent final possible), choix d'espace à valider à l'oreille sans prétention historique.

## Interprétation V2

Source : `harmonium_lab/references/ornaments/britannique_coloniale-hornpipe.json`, DS-T10b
(labo main `f8e0cbc`, corpus et droits dans ses manifestes).
78 airs, 9162 attaques porteuses écrites ; reprises non multipliées.

| Marque écrite | Occurrences | Par 100 notes |
|---|---:|---:|
| Liaison `(…)` | 0 | 0.000000 |
| Tenue liée `-` | 0 | 0.000000 |
| Détaché `.` | 0 | 0.000000 |
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

Aucune articulation écrite dans ce sous-corpus : aucune règle liée ou
détachée n’est inventée. L’interprétation vient du timing, du phrasé et
du raccourcissement émotionnel, indépendamment du swing déjà configuré.

Aucun accent, tenuto `L` ni marcato écrit : le petit poids d’accent métrique
4 face à Normal 100 est une **proposition**, non une mesure du corpus.
`L` désigne tenuto dans ABC ; `H` est mesuré mais ne sert pas à déduire
une durée de respiration ou à ralentir les fermatas.

| Réglage proposé | Valeur / intention |
|---|---|
| Timing par pas | −0.015 aux pulses ; +0.0075 aux autres pas pairs ; 0 aux impairs |
| Dispersion | ±0.015 pas, soit ±2.45 ms au tempo nominal avant swing |
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

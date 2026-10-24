# Accompagnement humain — étude et conception (V5a)

Phase 012, point 5. Étude et conception : **aucun code moteur modifié**. Mesures, maquette
et WAV ont été produits dans `harmonium_lab/target/accomp_etude/` (hors dépôt, effacé
depuis) ; les scripts jetables sont archivés dans
[`etudes/accompagnement/`](etudes/accompagnement/) pour pouvoir refaire les mesures (voir
[Reproduire](#reproduire)).

Légende des affirmations : **[S]** appuyée par une source citée en fin de document ;
**[M]** mesurée dans le corpus du labo ; **[C]** convention ou raisonnement, non
mesuré ; **[V]** à valider à l'oreille (propriétaire). Une valeur sans étiquette
est un choix de conception, justifié dans le texte.

## 0. Recommandation

1. **Notes de passage** (basse) en premier : c'est la piste la mieux fondée par les
   mesures (62 à 83 % des changements d'accord laissent deux degrés diatoniques entre
   les fondamentales, 63 à 95 % tombent au début d'une mesure, avec un temps libre
   juste avant).
2. **Grattage** ensuite : écarts de 4 à 10 ms entre cordes (3 à 5 cordes), centré sur
   le temps, coup montant doux optionnel ; ordres de grandeur issus d'une seule étude
   de guitare classique, donc à régler à l'oreille.
3. **Piano main gauche et accordéon basse-accord** ne demandent **aucun algorithme de
   plus** : mêmes voix basse/accord, autre motif, autre programme SoundFont,
   doublures d'octave/dixième, voicing fixe pour l'accordéon. Tout est donnée.
4. Schéma `[accompaniment.bass]` / `[accompaniment.chords]` en **remplacement** des
   champs plats actuels (principe IV) ; migration neutre des sept profils d'abord
   (snapshots identiques), puis bump `TUNING_VERSION` quand les profils adoptent des
   règles non nulles.
5. **Deux points de coordination avec V2** à régler avant V5b : le décalage de V2 est
   *affecté* (pas ajouté) et son jitter/ses tirages d'articulation sont *par note* ;
   voir [§5.4](#54-interaction-avec-performance-v2).

## 1. État actuel (main `ad7ad58`, vérifié dans le code)

`harmonium_core/src/trad/accomp.rs` :

- `bass_bar` : une note par pas de `bass_steps` ; fondamentale placée dans
  `bass_low..=bass_high` (40..50 pour le reel) ; **la
  note d'indice impair devient la quinte quand l'accord de son créneau est
  inchangé** (`slot == 0 || chords[slot] == chords[slot-1]`). Durée
  `bass_duration_steps`, vélocité `bass_velocity` : **identiques à chaque mesure**.
- `Guitarist::chop_bar_on_steps` : triade de 3 sons (voicing serré, enchaînement par
  plus petit mouvement, centre `guitar_center`, dérive `guitar_max_drift`), tous
  les sons **au même `start_step`**, durée `chop_duration_steps`, vélocité
  `chop_velocity` fixes.
- `generator.rs::chop_notes_for_bar` choisit le motif de chops dans
  `dynamics.chop_patterns` selon `chop_intensity + density_chop` (4 niveaux de
  densité, ex. reel `[[4], [4,12], [2,4,10,12], [2,4,6,10,12,14]]`), sinon
  `chop_steps`. Il **rejoue** le `Guitarist` depuis le début de la passe pour que
  le matériel reste stable après invalidation émotionnelle : aucun tirage dans ce
  chemin aujourd'hui.
- La basse n'a **aucune courbe de densité** (seulement `dynamics.bass_velocity`
  et le seuil d'entrée d'arrangement) ; l'harmonie est un vote par créneau
  `harmony.steps_per_chord` (8 pas = demi-mesure en reel, 12/16 = mesure
  entière en gigue/valse/hornpipe).

Réglages distribués (un pas de grille = une double-croche) :

| Profil | BPM (noire) | pas (ms) | créneau d'accord | basse (pas) | chops (pas) | durée chop | programmes accords/basse |
|---|---:|---:|---|---|---|---:|---|
| `reel_quebecois` | 230 | 65,2 | 8 pas (demi-mesure) | 0, 8 | 4, 12 | 1 pas | 24 / 32 |
| `reel_acadien` | 228 | 65,8 | 8 pas | 0, 8 | 4, 12 | 1 pas | 24 / 32 |
| `gigue_quebecoise` | 172,5 | 87,0 | 12 pas (mesure) | 0, 6 | 4, 10 | 1 pas | 24 / 32 |
| `gigue_britannique_coloniale` | 156 | 96,2 | 12 pas | 0, 6 | 0, 6 | 3 pas | 24 / 32 |
| `valse_quebecoise` | 145 | 103,4 | 12 pas | 0 | 4, 8 | 2 pas | 24 / 32 |
| `hornpipe_britannique_coloniale` | 92 | 163,0 | 16 pas | 0, 8 | 0, 8 | 4 pas | 24 / 32 |
| `marche_britannique_coloniale` | 108 | 138,9 | 16 pas | — | — | — | aucune section `[accompaniment]` |

La marche n'a ni basse ni accords : **cohérent** avec l'ensemble fifre-caisse-
grosse caisse **[S: Wikipedia, Fife and drum corps]** (pas d'instrument harmonique).
Aucun changement proposé pour elle.

## 2. Pratique réelle

### 2.1 Ce que les sources établissent, et ce qu'elles n'établissent pas

- **Québec, piano puis guitare.** Le piano apparaît dans les paroisses à la fin du
  xixe siècle et devient un instrument d'accompagnement courant ; plus tard,
  guitare, basse et batterie le complètent ou le remplacent en partie **[S:
  Ornstein 1982]**. Le « clogging » (pieds) a souvent été le seul accompagnement
  disponible du violon **[S: Wikipedia, Quebec fiddle]** ; la podorythmie accompagne
  traditionnellement les airs **[S: RPCQ, podorythmie, extrait de recherche]**. L'accordéon diatonique à 10 boutons s'installe à la fin du
  xixe siècle et « accompagne souvent le violon » **[S: RPCQ, jeu d'accordéon
  diatonique]**.
- **Basse « qui marche » dans le trad québécois ancien.** Des enregistrements de Jos
  Bouchard (fin des années 1930 à fin des années 1950) sont accompagnés d'un
  guitariste à « touche jazz », « walking bass », motifs **[S: La Presse 2012,
  M. Maziade]**. C'est la seule source québécoise obtenue sur des marches de basse.
- **Cape Breton (piano).** Trois sources concordantes : la main gauche porte
  un mouvement « root–five » (fondamentale–quinte) traditionnel (G. MacLean) ; des
  basses chromatiques « avec syncope subtile » (M. J. MacDonald, qui les aurait
  apprises en regardant la main gauche de pianistes de jazz) ; « walking bass lines,
  broken chords and melodic fragments » (M. Chisholm) ; « bass runs in the left hand »
  comme trait distinctif, avec la mise en garde de ne pas en faire trop en grand groupe
  **[S: Cranford/MacDonald ; Kimberley Fraser 2012 ; The Session, fil 18187]**.
  L'harmonie des débuts « ne s'aventurait pas loin de I–IV–V » **[S: Fraser]**.
  Les accompagnateurs adaptent leur jeu au violoneux ; beaucoup préfèrent des
  accords seuls **[S: Cranford, livret 7040]**.
- **Rythme de gigue (6/8).** Guitare irlandaise : motif « bas-haut-bas, bas-haut-bas » plutôt
  qu'un va-et-vient alterné **[S: Peghead Nation, Flynn Cohen]**. Cape Breton :
  chaque pulsation s'entend ; le pied d'appui donne 1 et 4, talon et pointe
  2, 3 et 5, 6 **[S: Fraser]**.
- **Stride.** « Basse, octave, septième ou dixième aux temps 1 et 3, accord aux
  temps 2 et 4 » **[S: Wikipedia, Stride piano]**. Le « stride de veillée » du
  propriétaire est lu ici comme ce même schéma basse-accord à main gauche large
  **[V]**.
- **Accordéon basse-accord.** Main gauche : paires bouton de basse (la
  fondamentale) / bouton d'accord, tonique poussée et dominante tirée sur les
  mêmes boutons ; le mélodéon québécois a **2 boutons** à la main gauche
  (« deux boutons et une valve ») ; les tierces sont souvent supprimées
  **[S: Wikipedia FR, accordéon diatonique ; RPCQ ; Rohart 2017]**. Suites de basses
  « sans en abuser » **[S: commentaire sous Rohart 2017]**.
- **Ce qui manque** : aucune source obtenue ne décrit en détail le motif de guitare
  du reel québécois ou acadien, ni l'accompagnement de la gigue/hornpipe
  britanniques coloniales du xixe siècle. Le recueil « Quebec Tunes 4 » prévient
  que ses accords « ne sont dans la plupart des cas pas représentatifs du
  jeu des accompagnateurs québécois contemporains » **[S, extrait de recherche]** : les
  accords chiffrés du corpus ABC sont à lire avec la même prudence.
  Pour ces profils, le motif boum-tchic actuel reste **[C]** (le labo l'écrit lui-même :
  « convention métrique ; motifs proposés, non mesurés dans l'ABC »,
  `mine_harmony`). Les valeurs ci-dessous pour ces profils sont des
  extrapolations des pratiques Cape Breton/irlandaises, **[V]**.

### 2.2 Motifs par type d'air

| Air | Guitare | Piano main gauche | Accordéon basse-accord | Statut |
|---|---|---|---|---|
| Reel (2/2, 4 noires) | basse aux noires 1 et 3, accord aux noires 2 et 4 (motif actuel) ; basse alternée fondamentale/quinte quand l'accord tient | idem, octave ou dixième sous la basse, accord en bloc ; marche de basse sur la noire 4 avant un changement | basse 1/3, accord 2/4, alternance basse tonique/dominante | motif **[C]** ; marches de basse **[S: Cape Breton]** ; reste **[V]** |
| Gigue (6/8) | « bas-haut-bas » : basse sur 1 et 4, accord sur les croches de contretemps (motif actuel `chop 4, 10`) | chaque pulsation audible : basse 1 et 4, accords 2-3 et 5-6 (Cape Breton) | basse 1 et 4, accord 2-3, 5-6 | **[S]** guitare et Cape Breton ; accordéon **[C]** |
| Valse (3/4) | basse 1, accord 2 et 3 (motif actuel) | idem, basse dans le grave | idem | **[C]** (labo : `basse [1], accord [2,3]`) |
| Hornpipe (4/4, lent) | basse et accord ensemble aux temps 1 et 3 (motif actuel) | idem | idem | **[C]** |
| Marche | aucun accompagnement harmonique | — | — | **[S]** (fifre-caisse) |

### 2.3 Densité selon l'énergie

- **[S]** Les accompagnateurs adaptent leur jeu au violoneux et évitent la
  surcharge en groupe (Cranford ; The Session). **[C]** Énergie basse : basse seule
  ou un accord par mesure ; énergie haute : chops à la croche et marches de basse
  aux changements. Le moteur sait déjà faire varier les chops (4 niveaux,
  `chop_patterns`). **Aucune donnée mesurée** pour la basse ou le grattage :
  l'arousal n'est pas mesurable dans l'ABC (V3a, V2). Le plan de V5b prévoit une
  courbe de densité pour les notes de passage, **[V]**.

### 2.4 Entre A et B, aux cadences, aux reprises

Seules les **fondamentales d'accord** se mesurent dans le corpus (le reste est
**[C]**). Airs AABB à 32 mesures pleines et accords chiffrés ; part du total des
symboles (%), changements d'accord par mesure :

| Profil (airs) | Partie | I | IV | V | vi | ii | ♭VII | chg/mesure | fin m.4 = V | fin m.8 = I |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `reel_quebecois` (78) | A | 47 | 11 | 25 | 4 | 7 | 4 | 1,01 | 49 | 86 / 83 |
| | B | 34 | 10 | 25 | 6 | 9 | 7 | 1,10 | 38 | 72 / 78 |
| `gigue_quebecoise` (21) | A | 44 | 13 | 24 | 4 | 10 | 1 | 1,00 | 48 | 71 / 95 |
| | B | 38 | 17 | 22 | 5 | 12 | 2 | 1,05 | 43 | 76 / 86 |
| `gigue_britannique_coloniale` (101) | A | 47 | 12 | 26 | 3 | 7 | 2 | 1,12 | 76 | 82 / 84 |
| | B | 36 | 13 | 27 | 8 | 9 | 2 | 1,16 | 54 | 73 / 78 |
| `hornpipe_britannique_coloniale` (9) | A | 50 | 17 | 26 | 1 | 6 | 0 | 1,42 | 89 | 100 / 100 |
| | B | 41 | 17 | 28 | 5 | 9 | 0 | 1,42 | 78 | 100 / 100 |
| valse québ. (6), reel acadien (5), marche (7) | — | effectifs trop faibles : non rapportés | | | | | | | | |

« fin m.8 = I » : première passe / seconde passe de la partie (les fins
alternatives, 1re/2e fin, n'ont pas le même accord final). **[M]** Lecture : B est
un peu plus actif que A (I −6 à −13 points, plus de vi, ii, ♭VII ; 0 à +0,09
changement par mesure) ; la fin de partie finit plus souvent sur V à la première
passe (gigue québ. A : 29 % contre 0 % à la reprise). **[C]** pour l'accompagnement :
marches de basse plus probables en B et vers la m.8, et fin de reprise sur
fondamentale tenue.

## 3. Mesures dans le corpus

Lecture seule via le parseur commun du labo (`harmonium_lab.trad.form.analyze_form`
et `features.parse_chord_symbol`). Script : `measure_chords.py`. **Population** :
airs du type du profil à la métrique du profil (reel : 8 croches/mesure ; gigue :
6/8 ; valse : 3/4 ; hornpipe, marche : 8 croches) ; **annoté** = première mesure
pleine chiffrée et au moins 0,4 symbole par mesure pleine. Accord = fondamentale +
qualité (majeur, mineur, diminué) ; septièmes ignorées ; accords tenus
(mesures sans symbole) reportés d'une mesure à la suivante. Un changement est un symbole dont l'identité
diffère de l'accord actif. Pourcentages arrondis au dixième.

| Profil | airs du type (métrique du profil) | airs avec accords chiffrés | symboles | `/` basse écrite (%) | septièmes (%) |
|---|---:|---:|---:|---:|---:|
| `reel_quebecois` | 507 | 161 | 5570 | 1,9 | 21,1 |
| `gigue_quebecoise` | 80 | 33 | 1193 | 2,7 | 21,5 |
| `valse_quebecoise` | 66 | 24 | 947 | 1,8 | 17,0 |
| `reel_acadien` | 169 | 8 | 313 | 0,0 | 12,5 |
| `gigue_britannique_coloniale` | 352 | 337 | 14065 | 1,7 | 20,0 |
| `hornpipe_britannique_coloniale` | 77 | 74 | 2685 | 1,0 | 22,9 |
| `marche_britannique_coloniale` | 16 | 15 | 408 | 0,0 | 31,1 |

**Limites [M]** : reel acadien = **8 airs annotés sur 169** : le corpus acadien vient
presque entièrement de The Session (168 reels cités par le profil), qui chiffre rarement
les accords ; ses lignes servent de repère seulement et le reel québécois sert de
référence (161 airs annotés sur 507). Marche : 15 airs annotés, hornpipe de l'A/B : 9 airs
AABB, valse : 6 : indicatifs. Les accords sont ceux des transcripteurs (voir
§2.1) ; les basses écrites (`"D/f#"`) sont rares (0 à 2,7 %), donc le corpus ne
contient pas de marches de basse écrites, seulement des fondamentales. Les
septièmes (12 à 31 % des symboles) sont ignorées.

### 3.1 Rythme harmonique

**Changements d'accord effectifs par mesure (%, mesures pleines)**

| Profil | n | 0 (tenu) | 1 | 2 | 3+ |
|---|---:|---:|---:|---:|---:|
| `reel_quebecois` | 4641 | 24,5 | 49,2 | 24,2 | 2,1 |
| `gigue_quebecoise` | 1031 | 21,0 | 56,4 | 22,6 | 0,1 |
| `valse_quebecoise` | 1105 | 30,1 | 67,0 | 2,9 | 0,0 |
| `reel_acadien` | 240 | 22,5 | 38,3 | 37,5 | 1,7 |
| `gigue_britannique_coloniale` | 10648 | 18,0 | 51,2 | 30,5 | 0,3 |
| `hornpipe_britannique_coloniale` | 1829 | 13,2 | 46,6 | 38,1 | 2,1 |
| `marche_britannique_coloniale` | 420 | 29,8 | 55,5 | 14,8 | 0,0 |

**Position des changements (% des changements, hors premier accord)**

| Profil | n | début de mesure | demi-mesure | autre temps | contretemps |
|---|---:|---:|---:|---:|---:|
| `reel_quebecois` | 4671 | 63,8 | 32,9 | 1,6 | 1,7 |
| `gigue_quebecoise` | 1017 | 70,2 | 29,6 | 0,0 | 0,2 |
| `valse_quebecoise` | 780 | 95,0 | 0,0 | 4,6 | 0,4 |
| `reel_acadien` | 276 | 58,7 | 39,9 | 1,4 | 0,0 |
| `gigue_britannique_coloniale` | 11725 | 67,5 | 32,3 | 0,0 | 0,2 |
| `hornpipe_britannique_coloniale` | 2291 | 62,7 | 35,0 | 2,1 | 0,2 |
| `marche_britannique_coloniale` | 342 | 73,4 | 21,6 | 0,3 | 4,7 |

**[M]** Lecture : **0,7 à 1,3 changement par mesure** (reel 1,04 ; gigue 1,02 québécoise,
1,13 britannique ; hornpipe 1,29 ; valse 0,73 ; marche 0,85). La valse est quasi à un
accord par mesure (95 % des changements au début de mesure, 2,9 % de mesures à deux
changements). Les changements au demi-temps pèsent **30 à 40 %** des changements au
reel, à la gigue, au hornpipe (21,6 % à la marche) ; aucun à la valse (4,6 % au
temps 3). Le créneau d'accord actuel du moteur (reel : demi-mesure ; gigue, valse,
hornpipe : mesure) est cohérent avec ces ordres de grandeur.

### 3.2 Accords par position de phrase

Mesures 1 à 8 des parties de 8 mesures pleines (changement à l'entrée = l'accord
du début de mesure diffère du précédent ; accord final = accord actif à la fin de
la mesure). Profils avec effectifs suffisants :

**Par position de phrase — `reel_quebecois` (mesures 1..8 des parties de 8 mesures complètes)**

| Mesure | n | entrée : changement (%) | changement intra-mesure (%) | accord final : I | V | IV | vi | autre |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 358 | 50 | 16 | 70 | 9 | 8 | 5 | 8 |
| 2 | 358 | 56 | 36 | 48 | 16 | 11 | 6 | 19 |
| 3 | 358 | 80 | 28 | 27 | 33 | 8 | 10 | 22 |
| 4 | 358 | 71 | 43 | 32 | 42 | 3 | 4 | 19 |
| 5 | 358 | 69 | 16 | 58 | 15 | 9 | 6 | 12 |
| 6 | 358 | 61 | 35 | 39 | 21 | 12 | 6 | 22 |
| 7 | 358 | 83 | 55 | 20 | 47 | 8 | 2 | 23 |
| 8 | 358 | 85 | 70 | 81 | 9 | 0 | 3 | 7 |

**Par position de phrase — `gigue_quebecoise` (mesures 1..8 des parties de 8 mesures complètes)**

| Mesure | n | entrée : changement (%) | changement intra-mesure (%) | accord final : I | V | IV | vi | autre |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 90 | 58 | 24 | 51 | 9 | 31 | 4 | 5 |
| 2 | 90 | 58 | 22 | 62 | 16 | 11 | 7 | 4 |
| 3 | 90 | 78 | 29 | 31 | 24 | 18 | 11 | 16 |
| 4 | 90 | 82 | 31 | 36 | 44 | 0 | 2 | 18 |
| 5 | 90 | 78 | 18 | 36 | 13 | 31 | 9 | 11 |
| 6 | 90 | 73 | 29 | 42 | 27 | 13 | 4 | 14 |
| 7 | 90 | 71 | 40 | 13 | 61 | 9 | 2 | 15 |
| 8 | 90 | 87 | 44 | 81 | 14 | 0 | 2 | 3 |

**Par position de phrase — `valse_quebecoise` (mesures 1..8 des parties de 8 mesures complètes)**

| Mesure | n | entrée : changement (%) | changement intra-mesure (%) | accord final : I | V | IV | vi | autre |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 43 | 70 | 0 | 65 | 5 | 26 | 5 | 0 |
| 2 | 43 | 58 | 0 | 49 | 23 | 19 | 5 | 4 |
| 3 | 43 | 86 | 0 | 33 | 19 | 12 | 9 | 27 |
| 4 | 43 | 86 | 0 | 44 | 23 | 0 | 5 | 28 |
| 5 | 43 | 81 | 0 | 56 | 5 | 21 | 9 | 9 |
| 6 | 43 | 77 | 0 | 33 | 19 | 28 | 5 | 15 |
| 7 | 43 | 100 | 19 | 0 | 49 | 9 | 5 | 37 |
| 8 | 43 | 93 | 21 | 79 | 16 | 0 | 0 | 5 |

**Par position de phrase — `gigue_britannique_coloniale` (mesures 1..8 des parties de 8 mesures complètes)**

| Mesure | n | entrée : changement (%) | changement intra-mesure (%) | accord final : I | V | IV | vi | autre |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 871 | 52 | 21 | 63 | 17 | 9 | 4 | 7 |
| 2 | 871 | 61 | 33 | 46 | 24 | 16 | 5 | 9 |
| 3 | 871 | 83 | 39 | 39 | 21 | 11 | 7 | 22 |
| 4 | 871 | 86 | 54 | 13 | 66 | 5 | 2 | 14 |
| 5 | 871 | 91 | 27 | 54 | 19 | 11 | 4 | 12 |
| 6 | 871 | 70 | 40 | 40 | 21 | 17 | 9 | 13 |
| 7 | 871 | 89 | 70 | 18 | 57 | 6 | 3 | 16 |
| 8 | 871 | 95 | 32 | 79 | 13 | 1 | 4 | 3 |

**Par position de phrase — `hornpipe_britannique_coloniale` (mesures 1..8 des parties de 8 mesures complètes)**

| Mesure | n | entrée : changement (%) | changement intra-mesure (%) | accord final : I | V | IV | vi | autre |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 174 | 48 | 24 | 63 | 24 | 7 | 6 | 0 |
| 2 | 174 | 71 | 43 | 41 | 24 | 24 | 5 | 6 |
| 3 | 174 | 93 | 49 | 36 | 29 | 9 | 10 | 16 |
| 4 | 174 | 94 | 72 | 8 | 85 | 2 | 0 | 5 |
| 5 | 174 | 95 | 28 | 60 | 20 | 14 | 2 | 4 |
| 6 | 174 | 76 | 47 | 39 | 22 | 25 | 5 | 9 |
| 7 | 174 | 94 | 79 | 10 | 76 | 3 | 2 | 9 |
| 8 | 174 | 98 | 18 | 90 | 7 | 0 | 3 | 0 |

**[M]** Lecture : m.4 finit sur V (42 à 85 % hors valse, 23 % en valse) et m.8 sur I
(79 à 90 %) ; m.7 est une mesure de dominante (47 à 76 %) ; la m.8 change
d'accord à l'intérieur de la mesure dans 70 % des cas au reel, 44 % à la gigue
québécoise, 18 % au hornpipe (lecture probable : cadence V → I en deux
demi-mesures, **[C]**). C'est la position où une marche de basse vers I est la plus
attendue **[C]**.

### 3.3 Mouvements de fondamentale et base des notes de passage

Écart de fondamentale entre accords successifs. `d` = degrés diatoniques montants
modulo 7 dans le mode de l'air.

**Distance diatonique entre fondamentales successives (% des changements ; d = degrés montants mod 7)**

| Profil | n | d=1 seconde ↑ | d=6 seconde ↓ | d=2 tierce ↑ | d=5 tierce ↓ | d=3 quarte ↑ | d=4 quinte ↑ | d=0 même racine | chromatique |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `reel_quebecois` | 4671 | 13,6 | 4,6 | 1,7 | 5,1 | 47,9 | 21,9 | 0,6 | 4,5 |
| `gigue_quebecoise` | 1017 | 14,2 | 8,3 | 2,7 | 4,8 | 45,7 | 22,9 | 0,5 | 1,0 |
| `valse_quebecoise` | 780 | 14,6 | 8,8 | 1,8 | 10,9 | 40,4 | 20,3 | 0,6 | 2,6 |
| `reel_acadien` | 276 | 16,3 | 9,8 | 2,2 | 7,2 | 46,4 | 18,1 | 0,0 | 0,0 |
| `gigue_britannique_coloniale` | 11725 | 12,7 | 5,0 | 2,0 | 4,3 | 47,2 | 26,4 | 0,5 | 1,8 |
| `hornpipe_britannique_coloniale` | 2291 | 12,1 | 4,2 | 0,9 | 4,5 | 51,3 | 26,3 | 0,2 | 0,4 |
| `marche_britannique_coloniale` | 342 | 11,7 | 2,6 | 0,6 | 2,0 | 49,7 | 33,3 | 0,0 | 0,0 |

**Notes de passage diatoniques possibles dans le plus court sens (% des changements)**

| Profil | n | 0 (seconde) | 1 (tierce) | 2 (quarte/quinte) |
|---|---:|---:|---:|---:|
| `reel_quebecois` | 4462 | 19,7 | 7,2 | 73,1 |
| `gigue_quebecoise` | 1007 | 23,1 | 7,5 | 69,3 |
| `valse_quebecoise` | 760 | 24,7 | 13,0 | 62,2 |
| `reel_acadien` | 276 | 26,1 | 9,4 | 64,5 |
| `gigue_britannique_coloniale` | 11515 | 18,6 | 6,4 | 75,0 |
| `hornpipe_britannique_coloniale` | 2281 | 16,6 | 5,4 | 77,9 |
| `marche_britannique_coloniale` | 342 | 14,3 | 2,6 | 83,0 |

**Passage : position × disponibilité (% des changements diatoniques)**

| Profil | frontière de mesure, ≥1 passage | intra-mesure, ≥1 passage | frontière, 0 passage | intra, 0 passage |
|---|---:|---:|---:|---:|
| `reel_quebecois` | 49,6 | 30,7 | 14,0 | 5,8 |
| `gigue_quebecoise` | 52,5 | 24,3 | 17,7 | 5,5 |
| `valse_quebecoise` | 71,7 | 3,6 | 23,2 | 1,6 |
| `reel_acadien` | 41,3 | 32,6 | 17,4 | 8,7 |
| `gigue_britannique_coloniale` | 54,5 | 26,9 | 12,8 | 5,7 |
| `hornpipe_britannique_coloniale` | 51,8 | 31,6 | 10,9 | 5,7 |
| `marche_britannique_coloniale` | 61,1 | 24,6 | 12,3 | 2,0 |

**[M]** Lecture : **la quarte montante (= quinte descendante) pèse 40 à 51 %** des
changements et la quinte montante 18 à 33 % ; secondes 14 à 26 % ; tierces 3 à
13 %. Dans le plus court sens, **62 à 83 % des changements laissent deux degrés
diatoniques entre les fondamentales** (matière à une marche de deux notes), 3 à 13 %
un seul, 14 à 26 % aucun (seconde : l'approche ne peut être que chromatique).
63 à 95 % des changements tombent au début d'une mesure (§3.1) ; 41 à 72 % cumulent
frontière de mesure et au moins un degré de passage : il existe alors un temps libre
dans la mesure précédente pour l'approche.

Écarts en demi-tons (montants) pour référence :

**Fondamentales : décalage en demi-tons montants (% des changements)**

| Profil | 5 | 7 | 2 | 10 | 9 | 3 | 8 | 4 | 1 | 0 | 6 | 11 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `reel_quebecois` | 48,1 | 22,0 | 14,5 | 5,7 | 4,8 | 1,3 | 1,2 | 0,6 | 1,2 | 0,6 | 0,1 | 0,0 |
| `gigue_quebecoise` | 46,1 | 23,1 | 13,9 | 8,0 | 3,4 | 0,6 | 1,6 | 2,1 | 0,4 | 0,5 | 0,0 | 0,4 |
| `valse_quebecoise` | 40,6 | 20,3 | 14,0 | 8,8 | 8,2 | 0,9 | 2,9 | 1,4 | 1,4 | 0,6 | 0,3 | 0,5 |
| `reel_acadien` | 46,4 | 18,1 | 15,6 | 9,8 | 5,8 | 0,0 | 1,4 | 2,2 | 0,7 | 0,0 | 0,0 | 0,0 |
| `gigue_britannique_coloniale` | 47,6 | 26,7 | 12,7 | 5,3 | 3,8 | 1,4 | 0,7 | 0,6 | 0,4 | 0,5 | 0,0 | 0,2 |
| `hornpipe_britannique_coloniale` | 51,3 | 26,3 | 12,2 | 4,1 | 4,2 | 0,9 | 0,3 | 0,2 | 0,1 | 0,2 | 0,0 | 0,2 |
| `marche_britannique_coloniale` | 49,7 | 33,3 | 11,1 | 2,6 | 1,8 | 0,6 | 0,3 | 0,0 | 0,6 | 0,0 | 0,0 | 0,0 |

### 3.4 Mélodie contre accord aux temps forts

Notes de la mélodie dont l'attaque tombe sur un temps, comparées à la triade de l'accord
chiffré actif :

**Mélodie sur le temps : % de notes hors triade de l'accord écrit**

| Profil | n notes sur le temps | temps 1 | demi-mesure (ou temps 2 du 6/8) | autres temps |
|---|---:|---:|---:|---:|
| `reel_quebecois` | 11406 | 14,7 | 22,3 | 28,3 |
| `gigue_quebecoise` | 1999 | 8,6 | 14,4 | — |
| `valse_quebecoise` | 2683 | 16,7 | — | 29,5 |
| `reel_acadien` | 936 | 7,2 | 18,8 | 26,3 |
| `gigue_britannique_coloniale` | 21217 | 9,3 | 15,5 | — |
| `hornpipe_britannique_coloniale` | 6996 | 5,8 | 17,0 | 26,6 |
| `marche_britannique_coloniale` | 1031 | 8,1 | 20,8 | 41,1 |

**[M]** Même chez les transcripteurs, **5,8 à 16,7 % des notes de mélodie au temps 1
sont hors triade** (14 à 22 % à la demi-mesure du reel et de la gigue) : le contrat
« aucune fausse note contre la mélodie aux temps forts » du plan s'entend donc
**sur la basse jouée à ces temps** (fondamentale ou quinte de l'accord choisi),
jamais sur une note de passage, et un test relatif au corpus plutôt qu'une
interdiction absolue (voir §5.8, test 8).

## 4. Grattage : ordres de grandeur

| Source | Donnée | Contexte |
|---|---|---|
| Freire et al. 2018, tableau 6 | accord plaqué, 4 sons, très rapide : écart entre cordes **0–6 ms**, étalement global 5–18 ms ; accord plaqué 3 sons, très lent : 0–5,5 ms / 5–11 ms | guitare nylon à micros hexaphoniques, 5 musiciens |
| idem | **accord gratté 6 cordes, modéré : 5–12,5 ms**, global 26–62 ms ; adagio : 8–11 ms, global 37–46 ms | idem |
| idem, tableau 5 | étalement global moyen : 3 sons 16–23 ms ; 4 sons 23–42 ms ; 5 sons 36–60 ms ; 6 sons 26–62 ms | introduction de *Wonderwall*, q = 86, trois exécutions |
| idem, tableau 3 | accord de 3 sons gratté (guarania) : 11, 13, 23 ms entre sons ; arpèges 16–40 ms | q = 90 |
| idem, §2 | résolution temporelle perceptive **1–18 ms** ; en dessous, accord perçu comme plaqué | psychoacoustique citée par l'étude |
| idem, §4.3 | les accords prescrits **montants** ont été joués avec moins de cordes par deux musiciens sur trois | une seule pièce |

**Limites.** Étude exploratoire : guitare classique à cordes nylon, étudiants,
pièces pop et guarania, tempos 64–90 à la noire, moyennes seulement, pas de coup de
médiator trad à 230 BPM. **Aucune mesure n'a été trouvée** pour : (a) la part de coups
montants en accompagnement trad, (b) l'écart des coups montants, (c) l'évolution de
l'écart avec le tempo (l'étude ne distingue pas : 8–11 ms à l'adagio, 5–12,5 ms au
modéré). Ce qui suit est donc **[V]**.

Part de coups montants : conventions pédagogiques, pas mesures. Le coup descendant
est le coup principal et porte la pulsation ; la gigue se joue « bas-haut-bas » (un
coup montant sur trois) **[S: Peghead Nation ; National Guitar Academy, extrait de
recherche non relu en entier]** ; au reel, les chops trad sont surtout descendants,
**[C]**.

**Valeurs de départ proposées (**[V]**, à régler à l'oreille).** Écart entre cordes en
ms fixes (le geste physique ne suit pas le tempo : exposant 0 par défaut), 4 sons
par coup descendant, 3 sons par coup montant :

| Profil | pas (ms) | écart descente (ms) | étalement global, 4 sons (ms) = fraction de pas | écart montée (ms) | coups montants |
|---|---:|---:|---:|---:|---|
| reel québécois / acadien | 65 | 6 | 18 = 0,28 pas | 4 | 25–40 % des croches suivant les chops 2 et 4 |
| gigue québécoise | 87 | 7 | 21 = 0,24 | 5 | « bas-haut-bas » : 1 coup sur 3 |
| gigue britannique coloniale | 96 | 8 | 24 = 0,25 | 5 | idem |
| valse québécoise | 103 | 9 | 27 = 0,26 | 6 | rares (≤ 10 %) |
| hornpipe britannique coloniale | 163 | 10 | 30 = 0,18 | 7 | rares (≤ 10 %) |

Plafond imposé par le moteur : l'étalement total doit tenir dans les ±0,49 pas du
décalage V2 et rester sous 40 % de la distance entre deux coups du motif (52 ms pour
deux chops à la croche, 130 ms d'écart, au reel). La maquette (§6) utilise 7 ms
±30 % en descente, 4 sons : 15–26 ms d'étalement mesurés dans les MIDI produits.

## 5. Conception

### 5.1 Principes

1. **Aucun nom de style dans le code** (V) : le code lit des champs, jamais un
   identifiant de motif. Les noms `guitar_strum`, `piano_left_hand`,
   `accordion_bass_chord` ne sont que des **étiquettes de recettes** dans ce document
   (et éventuellement dans un README de profil) ; le chargeur n'en connaît aucune.
2. **Voix basse et voix accords séparées**, chacune un motif de coups (pas, degré ou
   geste, durée, vélocité) et pas de table par position de phrase. Les décisions
   contextuelles (note de passage, coup montant) sont des **règles pondérées**
   conditionnées par le contexte musical (accord suivant différent, phase de phrase,
   densité émotionnelle), jamais par un numéro de mesure.
3. **Chemin unique** (IV) : les champs plats `bass_steps`, `bass_duration_steps`,
   `bass_velocity`, `bass_low`, `bass_high`, `chop_steps`, `chop_duration_steps`,
   `chop_velocity`, `guitar_center`, `guitar_max_drift` et `dynamics.chop_patterns`
   sont **remplacés**, pas doublés ; les sept profils sont migrés dans le commit qui
   change le schéma.
4. **Neutralité** : profil sans règle de passage ni grattage ⇒ **aucun tirage de
   plus** et flux de snapshots identique à main (même exigence que V2).
5. **Tout est préparé hors callback** : l'accompagnement produit des `TimelineNote`
   (notes supplémentaires) et des `performance.offset` ; le callback ne lit que
   des événements déjà prêts (§5.6).

### 5.2 Schéma proposé

`[accompaniment]` conserve le vote d'harmonie (`strong_steps`, `strong_weight`,
`weak_weight`, `tie_priority`) ; deux sous-tables la complètent. Valeurs
d'exemple, pas des réglages distribués.

```toml
[accompaniment]                         # vote d'harmonie, inchangé
strong_steps = [0, 8]
strong_weight = 2
weak_weight = 1
tie_priority = ["I", "V", "IV"]

# ── Voix basse ─────────────────────────────────────────────────────────────
[accompaniment.bass]
low = 40                                # fenêtre des coups (fondamentale/quinte)
high = 50
walk_low = 38                           # fenêtre élargie des notes de passage (défaut : low/high)
walk_high = 52

[[accompaniment.bass.hits]]             # remplace bass_steps/_duration/_velocity
step = 0
degree = "root"                         # root | fifth | fifth_if_held (quinte si l'accord est tenu depuis le créneau précédent)
duration = 4
velocity = 78
doubling = []                           # "octave" | "tenth" : sons ajoutés au-dessus

[[accompaniment.bass.hits]]
step = 8
degree = "fifth_if_held"                # = le comportement actuel pour l'indice impair
duration = 4
velocity = 78
doubling = []

# Règles de notes de passage : absent = aucune note, aucun tirage.
[accompaniment.bass.passing]
none_weight = 4                         # poids de « ne rien ajouter » à chaque transition
density = [[0.0, 0.0], [0.5, 0.6], [1.0, 1.0]]   # multiplicateur selon rhythm_density (même axe que chop_patterns)
phrase  = [[0.0, 0.7], [0.75, 1.0], [1.0, 1.3]]  # multiplicateur selon la phase de phrase (convention V2)
avoid_melody_intervals = [1, 6, 11]     # demi-tons modulo 12 interdits contre la mélodie qui sonne à l'attaque

[[accompaniment.bass.passing.rules]]
when = "next_target_differs"            # next_target_differs | chord_change_inside_bar
before = [4]                            # pas avant le coup cible suivant ; une note par entrée
motion = "scale_step"                   # scale_step : voisin diatonique de la cible, côté de départ
weight = 3                              # chromatic : demi-ton ; scale_walk : voisins successifs
duration = 2
velocity_delta = -12
min_leap = 0                            # demi-tons minimum entre la note précédente et la cible

[[accompaniment.bass.passing.rules]]
when = "next_target_differs"
before = [4, 2]                         # deux notes : marche de deux degrés
motion = "scale_walk"
weight = 2
duration = 1
velocity_delta = -12
min_leap = 5                            # seulement pour un saut de quarte ou plus

[[accompaniment.bass.passing.rules]]
when = "next_target_differs"
before = [2]
motion = "chromatic"
weight = 1
duration = 2
velocity_delta = -12
min_leap = 0

# ── Voix accords ───────────────────────────────────────────────────────────
[accompaniment.chords]
voicing = "close_triad"                 # close_triad (conduite de voix, actuel) | fixed_triad (boutons d'accordéon : sans conduite de voix)
center = 60                             # close_triad : comme guitar_center / guitar_max_drift
max_drift = 6
low = 52                                # fixed_triad : fenêtre du voicing
high = 64
strings = 4                             # sons par coup descendant ; la triade est complétée par doublures d'octave en montant depuis la plus grave
default_pattern = 1                     # motif utilisé quand l'offset d'émotion est nul (= l'ancien chop_steps)

[[accompaniment.chords.patterns]]       # indexés par densité, remplacent dynamics.chop_patterns
hits = [{ step = 4, stroke = "down", duration = 1, velocity = 72 }]

[[accompaniment.chords.patterns]]
hits = [
  { step = 4,  stroke = "down", duration = 1, velocity = 72 },
  { step = 12, stroke = "down", duration = 1, velocity = 72 },
]

[[accompaniment.chords.patterns]]
hits = [
  { step = 2,  stroke = "down", duration = 1, velocity = 66 },
  { step = 4,  stroke = "down", duration = 1, velocity = 72 },
  { step = 10, stroke = "down", duration = 1, velocity = 66 },
  { step = 12, stroke = "down", duration = 1, velocity = 72 },
]

[[accompaniment.chords.patterns]]
hits = [
  { step = 2,  stroke = "down", duration = 1, velocity = 66 },
  { step = 4,  stroke = "down", duration = 1, velocity = 72 },
  { step = 6,  stroke = "up",   duration = 1, velocity = 54, probability = 0.4 },
  { step = 10, stroke = "down", duration = 1, velocity = 66 },
  { step = 12, stroke = "down", duration = 1, velocity = 72 },
  { step = 14, stroke = "up",   duration = 1, velocity = 54, probability = 0.4 },
]

# Grattage : absent = tous les coups sont plaqués (stroke "block" implicite), aucun tirage.
[accompaniment.chords.strum]
down_gap_ms = 6.0                       # écart entre cordes adjacentes, coup descendant (graves → aigus)
up_gap_ms = 4.0                         # coup montant (aigus → graves)
up_strings = 3                          # sons d'un coup montant (les plus aigus)
gap_jitter = 0.3                        # fraction, tirage par écart, ChaCha8Rng dérivé (voir §5.5)
anchor = 0.5                            # 0 : première corde sur le temps ; 0,5 : centre ; 1 : dernière corde
tempo_exponent = 0.0                    # 0 : ms fixes ; 1 : écart ∝ durée du pas (réf. ref_bpm)
ref_bpm = 230.0                         # lu seulement si tempo_exponent > 0
max_spread_ratio = 0.4                  # étalement ≤ ratio × distance au coup suivant du même motif
velocity_slope = 4                      # surplus de la première corde, en vélocité
velocity_jitter = 5                     # ± vélocité par corde
```

Un `stroke = "block"` est un accord plaqué (comportement actuel) ; `probability`
vaut 1 par défaut ; un coup à `probability < 1` consomme un tirage, un coup à 1 ne
consomme rien. Validation au chargement, erreur lisible avec chemin de champ :
`step` dans la grille et sans collision à l'intérieur d'une voix ; `degree`,
`motion`, `when`, `stroke`, `doubling` dans l'ensemble fermé ; `before` décroissant
et ≥ 1 ; `walk_low ≤ low`, `high ≤ walk_high` ; la fenêtre de 12 hauteurs de la basse
jazz reste exigée pour le modèle jazz ; poids entiers, somme dans `u32` ; `down_gap_ms`/`up_gap_ms` dans
0..40 ; `gap_jitter` dans 0..0,8 ; `anchor` dans 0..1 ; `strings` dans 3..6 ;
`up_strings` dans 2..`strings` ; et **validation croisée avec `[performance]`** :
`performance.tracks.Chords.jitter × pas_ms(min_bpm) ≤ 0,5 × down_gap_ms ×
(1 − gap_jitter)` tant que V2 tire un jitter par note (§5.4).

**Correspondance avec les champs actuels** (migration neutre) :

| Actuel | Nouveau |
|---|---|
| `bass_steps = [0, 8]` | deux `[[bass.hits]]` aux pas 0 et 8 ; `degree = "root"` au premier, `"fifth_if_held"` au second (indice impair actuel) |
| `bass_duration_steps`, `bass_velocity` | `duration`, `velocity` de chaque coup |
| `bass_low`, `bass_high` | `bass.low`, `bass.high` |
| `chop_steps = [4, 12]` | `chords.patterns[default_pattern].hits` avec `stroke = "block"` |
| `chop_duration_steps`, `chop_velocity` | `duration`, `velocity` de chaque coup |
| `guitar_center`, `guitar_max_drift` | `chords.center`, `chords.max_drift`, `voicing = "close_triad"` |
| `dynamics.chop_patterns` | `chords.patterns` (même indexation par densité) |

### 5.3 Recettes (exemples, pas des styles du code)

**Guitare grattée** : schéma complet ci-dessus, `[instruments] chords_program = 24`
(nylon ; 25 pour acier à voir à l'oreille).

**Piano main gauche (basse-accord, « stride » de veillée)** :

```toml
[instruments]
chords_program = 0                      # piano à queue : accords et basse sur le même instrument
bass_program = 0

[accompaniment.bass]
low = 36
high = 47
walk_low = 34
walk_high = 50
[[accompaniment.bass.hits]]
step = 0
degree = "root"
duration = 3
velocity = 82
doubling = ["octave"]                   # "tenth" pour la basse-dixième du stride
[[accompaniment.bass.hits]]
step = 8
degree = "fifth_if_held"
duration = 3
velocity = 82
doubling = ["octave"]
[accompaniment.bass.passing]
none_weight = 3                         # plus de marches qu'à la guitare : « bass runs » Cape Breton [S]
density = [[0.0, 0.0], [0.5, 0.8], [1.0, 1.2]]
phrase  = [[0.0, 0.7], [0.75, 1.0], [1.0, 1.3]]
avoid_melody_intervals = [1, 6, 11]
# + les trois règles ci-dessus, poids plus élevés pour scale_walk
[accompaniment.chords]
voicing = "close_triad"
center = 58
max_drift = 5
strings = 3                             # pas de grattage : aucune table [strum]
default_pattern = 0
[[accompaniment.chords.patterns]]
hits = [
  { step = 4,  stroke = "block", duration = 2, velocity = 62 },
  { step = 12, stroke = "block", duration = 2, velocity = 58 },
]
```

**Accordéon basse-accord (mélodéon : deux boutons)** :

```toml
[instruments]
chords_program = 21                     # accordéon GM
bass_program = 21

[accompaniment.bass]
low = 36
high = 47
[[accompaniment.bass.hits]]
step = 0
degree = "root"
duration = 3
velocity = 70
[[accompaniment.bass.hits]]
step = 8
degree = "fifth_if_held"                # basse tonique ↔ dominante : les deux boutons
duration = 3
velocity = 70
# aucune [accompaniment.bass.passing] : aucune marche sur un mélodéon à 2 basses [S]
[accompaniment.chords]
voicing = "fixed_triad"                 # bouton d'accord : voicing fixe, sans conduite de voix
low = 52
high = 64
strings = 3
default_pattern = 0
[[accompaniment.chords.patterns]]
hits = [
  { step = 4,  stroke = "block", duration = 2, velocity = 56 },
  { step = 12, stroke = "block", duration = 2, velocity = 56 },
]
```

Pour un accordéon à 8–24 basses, ajouter une règle `scale_step` de poids faible
(« suites de basses, sans en abuser »). Pour la gigue de Cape Breton : basse aux pas
0 et 6, accords aux pas 2, 4, 8, 10 (chaque pulsation audible).

### 5.4 Interaction avec `[performance]` (V2)

État lu : branche `wquintal/vie-v2-performance` en cours (non commitée,
`harmonium_core/src/performance.rs` non suivi) ; noms susceptibles de bouger.
V2 publie `NotePerformance { offset (millièmes de pas, i16), gate (millièmes de
durée), breath }`, ses décalages vivent en **fraction de pas** (le geste suit le
tempo), clampés à ±0,49 pas, et les événements sont précalculés à `stage_bar`.

Partage des rôles :

| Quoi | Propriétaire | Règle |
|---|---|---|
| Décalage de l'attaque de l'accord (avance, retard, `tension_advance`) | V2 `performance.tracks.Chords.offsets` | s'applique à toutes les cordes d'un coup, par `start_step` |
| **Décalage entre cordes** | V5 `chords.strum` | **ajouté** à l'offset V2 (`offset_corde += k·écart − ancre`), puis clampé |
| Gate, articulations, respiration | V2 | s'appliquent par corde sur la durée écrite ; la respiration touche « toutes les notes de la dernière attaque » (déjà dans V2) : tout le coup |
| Courbe de phrase, bornes de vélocité, accents d'arousal | V2 | sur la vélocité de chaque corde ; la pente de vélocité du grattage (`velocity_slope`) est écrite dans la note avant V2 |
| Notes de passage (hauteur, pas, durée) | V5 `bass.passing` | notes **écrites** (présentes dans le snapshot comme toute note de basse) ; V2 leur applique ses réglages de piste Bass |

**Deux points à régler avec le propriétaire de V2 avant V5b** (lus dans le code en cours,
à confirmer sur la version fusionnée) :

1. `Performance::apply` **affecte** `note.performance.offset` (= tirage + décalage de
   pas − tension). Si le grattage est calculé avant V2, il est écrasé ; s'il est calculé
   après, il ajoute. **Demande** : V2 additionne, ou V5b passe après V2 et additionne.
   Choix recommandé : V5b après V2 (`strum::apply(measure, …)` juste après
   `Performance::apply`, avec la durée du pas de la mesure), sans toucher à V2.
2. Le **jitter** (`rng.next_f32` par note) et le tirage de règle d'articulation sont
   **par note**, donc par corde : un jitter de plus de la moitié de l'écart entre cordes
   **inverse l'ordre des cordes**, et `NoteContext::Ascending/Descending` est évalué de
   note à note, ce qui n'a pas de sens à l'intérieur d'un accord. **Demande** :
   grouper par (piste, `start_step`) : un tirage de jitter, un tirage d'articulation
   et un contexte par attaque. À défaut, la validation croisée du §5.2 borne le jitter
   V2 de la piste Chords : avec `down_gap_ms = 6` et `gap_jitter = 0,3` au reel,
   `jitter ≤ 0,03` pas (≈ 2 ms).

Contrat d'ordre : dans un groupe (piste, `start_step`), **l'ordre du `Vec` est l'ordre
d'attaque** ; un coup descendant est croissant en hauteur, un coup montant décroissant.
Le décalage de la première corde peut être négatif (ancre centrale) : V2 sait déjà
emprunter l'attaque du pas 0 dans le dernier pas de la mesure précédente.

### 5.5 Contrat de snapshot et déterminisme

- **Notes d'un accord gratté** : N `TimelineNote` sur la piste `Chords`, même
  `start_step`, `id` croissants dans l'ordre d'attaque, hauteurs du voicing complétées
  des doublures d'octave, et **attaques décalées dans le flux** par `performance.offset`
  (millièmes de pas, additif V2), comme les décalages de V2. L'ms du profil est converti
  en millièmes de pas à la mesure, avec le tempo de la mesure, arrondi à 0,001 pas ;
  le snapshot lui-même ne porte pas de ms. Un coup `block` ou une table `strum`
  absente : aucun décalage de V5, accords identiques à main.
- **Notes de passage** : notes `Bass` ordinaires (`start_step`, `duration_steps`,
  `velocity`) à un pas qui n'est pas un coup `hits` de la basse ; jamais sur un pas
  de `hits`, ni sur un coup de basse ; hauteur dans `[walk_low, walk_high]`.
- **Tirages** (passage, `probability` des coups, `gap_jitter`) : un `ChaCha8Rng`
  **dérivé** de (graine de l'air, indice de mesure du tour, voix), pas le flux
  séquentiel du générateur. Raisons : `chop_notes_for_bar` **rejoue** les mesures
  d'une passe après invalidation émotionnelle sans rien tirer ; un tirage pris dans le
  flux partagé déplacerait toutes les décisions de mélodie et d'arrangement pour la même
  graine, ce qui ferait changer les pistes Melody/Heel/Toe. Un profil sans
  règle active ne tire rien. **À trancher à l'ouverture de V5b-1** en lisant
  `ActiveTune` : si l'air ne porte pas de graine propre, un unique tirage à la création
  de l'air (rupture avouée, bump) est le repli ; [INFERENCE] sur ce point, non vérifié
  dans le code.
- **Neutralité** : migration des sept profils sans règle ⇒ `Measure` identiques à
  main sur 100 mesures (goldens inchangés, `profile_d2_*.json` inclus) ; adoption de
  règles non nulles ⇒ bump `TUNING_VERSION` et goldens régénérés dans le même commit.

### 5.6 Coût temps réel

- Rien ne change dans le callback : notes, offsets, gate et événements sont préparés à
  `stage_bar` (V2) ; le callback ne lit que des `PreparedEvent` triés. Aucun RNG, aucune
  allocation, aucun verrou n'est ajouté au thread audio.
- Volume : motif de densité maximale (6 coups × 4 cordes) + 3 notes de basse + coups
  montants ≈ 30 notes par mesure, soit ≈ 60 événements ; la capacité fixe `EVENT_CAP =
  8192` de V2 (`pointers.rs`) est largement suffisante. `active[piste][hauteur]` admet une voix par hauteur : les
  doublures d'octave sont des hauteurs distinctes ; un retrigger de même hauteur (passage
  répétant une note tenue) est géré par la logique V2 existante (test avec retrigger
  traversant la barre).
- Test debug sous `RTCheckAllocator` : étendre `performance_playback_tests` avec une
  mesure à accord gratté (offsets négatifs au pas 0, offsets positifs à la fin du pas,
  marche de basse à cheval sur la barre).
- La dette RT préexistante (enregistreur, `process_commands`, rapport) est hors de ce
  périmètre, comme pour V1/V2.

### 5.7 Interaction avec `[mix]` (V1)

Aucun champ de `[mix]` ne change. Points d'attention (mesurables avec
`measure_mix.py`) : un 4ᵉ son par coup et les coups montants ajoutent de l'énergie à
`Chords` (maquette : +0,3 dB de RMS sur le mélange complet pour le grattage, seeds 6 et 8) ; les niveaux `Chords` et
`Bass` par profil restent calés à la guitare nylon/basse acoustique : **piano et
accordéon changent de programme** et ont leur propre dynamique (dans la maquette, seed 6 : RMS du mélange complet de −0,8 dB
pour le piano et de −3,1 dB pour l'accordéon, par rapport à la référence, avant égalisation) ; il faudra
recaler `level_db` de `Chords`/`Bass` dans `[mix]` quand un profil change
d'instrument. Critères de V1 conservés : mélodie ≥ 4 dB au-dessus de chaque piste
d'accompagnement (min mesuré V1 : 8,99 dB), crête mélangée ≤ −1,5 dBFS au pire des
rendus à émotions maximales (règle révisée de V1).

### 5.8 Tests de contrat à écrire (flux de snapshots, principe VI)

1. **Migration neutre** : les sept profils migrés produisent les mêmes `Measure` que
   main sur 100 mesures (y compris Melody/Heel/Toe) et les goldens passent sans
   régénération.
2. **Validation** : chaque champ hors borne / collision de pas / `degree` ou `motion`
   inconnu / `strings` hors 3..6 / validation croisée jitter-écart → `ProfileError` avec
   le chemin du champ.
3. **Ordre des cordes** : sur 200 mesures, tout groupe (Chords, `start_step`) à coup
   descendant a un `performance.offset` strictement croissant dans l'ordre du `Vec` et
   des hauteurs croissantes ; coup montant : décroissants.
4. **Écart entre cordes** : écart mesuré en ms (avec le tempo de la mesure) dans
   `down_gap_ms × [1−jitter, 1+jitter]` ± 1 millième de pas d'arrondi ; étalement ≤
   `max_spread_ratio × distance au coup suivant` ; offsets dans ±0,49 pas.
5. **Ancrage** : moyenne des offsets d'un groupe = `(0,5 − anchor) × étalement`
   (± arrondi).
6. **Atomicité du coup** : toutes les cordes d'un groupe partagent articulation,
   jitter V2 et gate (conséquence de la demande V2 n° 2).
7. **Notes de passage** : jamais sur un pas de `hits` ; jamais deux fois la même
   hauteur consécutive ; hauteur dans `[walk_low, walk_high]` ; dernière approche à
   ≤ 2 demi-tons de la cible (`scale_step`, `chromatic`) ou diatonique (`scale_walk`
   dans le mode publié) ; part de notes de passage par profil dans les bornes de
   conception ; jamais d'intervalle de `avoid_melody_intervals` contre la mélodie
   qui sonne à l'attaque.
8. **Aux temps forts** : la basse aux pas de `strong_steps` est la fondamentale ou la
   quinte de l'accord choisi (invariant) ; le taux de basse en seconde mineure /
   septième majeure / triton contre la mélodie à ces pas n'excède pas le taux du
   corpus pour les notes de mélodie hors triade (§3.4) + 3 points.
9. **Déterminisme et rejeu** : même graine ⇒ même flux ; notes d'une mesure
   identiques avant et après une invalidation émotionnelle de la passe ; aucun tirage
   quand aucune règle n'a de poids/probabilité positifs (fenêtre de 100 mesures
   comparée à un profil sans la table).
10. **Pistes voisines inchangées** : Melody, Heel et Toe identiques à main pour une même
    graine quand seules les tables d'accompagnement changent.
11. **Export MIDI** : les offsets de grattage sont dans le MIDI enregistré (tick 480
    PPQ : 6 ms = 11 ticks à 230 BPM) ; l'ordre des attaques d'un accord est celui du
    flux.
12. **Temps réel** : test debug `RTCheckAllocator` sur un enchaînement mesure gratté +
    marche de basse traversant la barre (frames d'attaque négatives comprises).

## 6. Maquette hors moteur

Aucun fichier du dépôt n'est touché. Script jetable `mockup.py` (copié dans
`harmonium_lab/target/accomp_etude/`) : reel exporté par le CLI de main
(`harmonium-cli --export --bars 34 --style reel --seed S --feet off --record-midi …
--record-truth …`, GeneralUser GS), canaux accords et basse **remplacés**, mélodie
(canal 0) inchangée, rendu par `fluidsynth` avec
`/Users/william.quintal/Project/bascanada/harmonium/assets/GeneralUser-GS.sf2` (réverbération
par défaut de fluidsynth ; pas de pieds ; pas de `[mix]`). Seeds **6** et **8**
(chops dès la mesure 1, basse dès la mesure 3 ; les seeds 1, 2, 3, 4, 9, 10 ne les
font entrer qu'autour de la mesure 10). L'accord vient de la vérité (une valeur par demi-mesure) ;
la gamme des notes de passage = la gamme majeure qui couvre le mieux la
mélodie.

| Fichier | Contenu |
|---|---|
| `reel_sN_baseline.wav` | moteur de main, repassé par le même pipeline (référence) |
| `reel_sN_strum.wav` | chops en coups descendants à 4 cordes, 7 ms ±30 % centrés sur le temps, coup montant doux (3 cordes, 5 ms) deux pas après 40 % des chops des pas 4 et 12 (pas 6 et 14) |
| `reel_sN_passing.wav` | basse avec approche diatonique/chromatique/marche de deux notes avant un changement (16–20 % des notes de basse) ; accords d'origine |
| `reel_sN_strum_passing.wav` | les deux ensemble (cible probable) |
| `reel_sN_piano.wav` | programme 0 : basse avec octave, accords en blocs avec conduite de voix, marches de basse |
| `reel_sN_accordion.wav` | programme 21 : basse tonique/dominante, accords en blocs à voicing fixe, aucune marche |

Chemin : `/Users/william.quintal/Project/bascanada/harmonium_lab/target/accomp_etude/`
(`.wav` et `.mid` pour N ∈ {6, 8}, plus `levels.json`). **Même volume moyen** : chaque
WAV est mis à gain constant pour que le **RMS sur tous les échantillons** soit identique entre
les six variantes d'un même seed, au plus fort niveau où la crête de la variante la plus
pointue reste ≤ −1 dBFS (sortie 16 bits) ; RMS final −17,4 dBFS (seed 6), −17,9 (seed 8).

| Fichier | RMS avant (dBFS) | crête avant | gain (dB) | RMS après | crête après |
|---|---:|---:|---:|---:|---:|
| `reel_s6_baseline.wav` | −36,6 | −21,5 | +19,2 | −17,4 | −2,2 |
| `reel_s6_strum.wav` | −36,3 | −20,4 | +18,9 | −17,4 | −1,5 |
| `reel_s6_passing.wav` | −36,5 | −20,5 | +19,1 | −17,4 | −1,4 |
| `reel_s6_strum_passing.wav` | −36,2 | −19,8 | +18,8 | −17,4 | −1,0 |
| `reel_s6_piano.wav` | −37,4 | −21,2 | +20,0 | −17,4 | −1,2 |
| `reel_s6_accordion.wav` | −39,7 | −24,7 | +22,3 | −17,4 | −2,4 |
| `reel_s8_baseline.wav` | −36,8 | −21,9 | +18,9 | −17,9 | −3,0 |
| `reel_s8_strum.wav` | −36,6 | −21,3 | +18,6 | −17,9 | −2,7 |
| `reel_s8_passing.wav` | −36,7 | −21,4 | +18,8 | −17,9 | −2,6 |
| `reel_s8_strum_passing.wav` | −36,4 | −20,4 | +18,5 | −17,9 | −2,0 |
| `reel_s8_piano.wav` | −37,8 | −20,9 | +19,9 | −17,9 | −1,0 |
| `reel_s8_accordion.wav` | −40,2 | −24,5 | +22,3 | −17,9 | −2,2 |

Vérifications **objectives** faites sur les MIDI de la maquette (**pas d'écoute**) :

- grattage : 89 (seed 6) et 91 (seed 8) groupes d'accords ; étalement global moyen à
  4 cordes **21,3 ms** (min 15,2, max 26,1) au seed 6 et **21,0 ms** (15,8–26,6) au seed 8 ;
  24 % et 25 % des groupes sont des coups montants ;
- notes de passage : 16 % à 20 % des notes de basse (hors pas 0 et 8), contre 0 à 6 %
  pour la référence (les 4 notes de la référence du seed 8 hors pas 0/8 ne sont pas
  expliquées ici) ; **1 sur 16 (seed 6) et 3 sur 12 (seed 8)** passages sont en seconde mineure / septième
  majeure / triton contre la mélodie qui sonne : sans veto, ce taux est trop élevé
  pour être laissé tel quel → `avoid_melody_intervals` du schéma ;
- aucune note répétée dans l'approche (correction faite en cours de maquette).

Constat honnête : la maquette n'est pas le moteur. Vélocités et durées sont
reprises du moteur (basse 78, accords 72, durée 1 pas), mais les voicings de piano et d'accordéon
sont ad hoc, la basse n'est pas dans la fenêtre 40–50 pour le piano (36–47), et la
tonalité vient d'une estimation sur la mélodie. **Écoute du propriétaire à faire**
sur : (1) grattage perceptible ou trop lâche à 230 BPM ; (2) marches de basse :
trop fréquentes, trop mélodiques ? (3) piano et accordéon : timbre GM acceptable
ou banque dédiée à chercher ?

## 7. Plan V5b

Dépend de V2 fusionné. Dépendance de fichiers : tout passe par `style_profile.rs`,
`tuning.rs`, `trad/accomp.rs` et `trad/generator.rs` : **un seul agent** sur 1 → 3,
puis 4 et 5 en parallèle.

| Tâche | Contenu | Acceptation |
|---|---|---|
| **V5b-0** Coordination V2 | Faire trancher les deux demandes du §5.4 avec le propriétaire de V2 (offset additif ou passe après V2 ; jitter et articulation par attaque) ; trancher la graine dérivée (§5.5) | décision écrite dans `plan.md` |
| **V5b-1** Schéma + migration | `[accompaniment.bass]`/`[accompaniment.chords]`, validation (§5.2), migration neutre des sept profils (champs plats et `dynamics.chop_patterns` supprimés), `profiles/README.md` | tests 1 et 2 ; goldens inchangés, `TUNING_VERSION` inchangé |
| **V5b-2** Notes de passage | règles pondérées (`passing`), fenêtre de marche, veto mélodie, RNG dérivé, courbes `density`/`phrase`, tests 7 à 10 | tests 7–10 ; part de passage mesurée par profil |
| **V5b-3** Grattage | `strum` (cordes, doublures, ordre, ms → millièmes de pas, ancre, coup montant), passe après V2, export MIDI, tests 3 à 6, 11, 12 | tests 3–6, 11, 12 |
| **V5b-4** Piano / accordéon et mix | recettes en données (programmes 0/21, `fixed_triad`, doublures) dans un profil de démonstration, recalage de `level_db` ; relance de `measure_mix.py` | écart mélodie − accompagnement ≥ 4 dB conservé ; crête max-émotions ≤ −1,5 dBFS |
| **V5b-5** Labo | promouvoir `measure_chords.py` en commande `mine-accomp` (corpus, tests) ; ajouter les métriques côté généré (part de passage, étalement, ordre) pour comparer au corpus ; enrichir le corpus annoté acadien si possible | mesures versionnées dans `references/` ; compare généré/corpus |
| **V5b-6** Écoute et ADR | kit de 3 profils, avant/après, réglages valeurs de départ (§4, §5.2) à l'oreille ; ADR dans le plan de phase | validation propriétaire |

Valeurs de départ des règles de passage (**[V]**, hypothèses issues des mesures) :
probabilité d'au moins un passage par transition de mesure visée à 25–35 % au reel
et à la gigue (guitare), 40–50 % au piano, 0 à la valse sauf phase de cadence (basse
« oom » seule : la valse a 95 % de ses changements au début de la mesure, 5 % en cours de
mesure), pondération par la phase de phrase (×1,3 vers la m.8, ×0,7 aux m.1–2).

## 8. Questions ouvertes

1. V2 : offset additif et tirages par attaque (§5.4) ; état de V2 lu en cours, à
   relire après fusion.
2. Graine de l'air pour le RNG dérivé (§5.5) : non vérifiée dans le code.
3. ms contre pas : choisi **ms** pour le grattage (le geste physique ne suit pas le
   tempo ; trois tempos de profil de 65 à 163 ms/pas) ; l'ordre de grandeur étudié
   ne permet pas de trancher l'exposant de tempo (0 par défaut).
4. Pratique de guitare québécoise/acadienne : aucune source détaillée trouvée ; à
   valider avec l'oreille du propriétaire avant de figer des valeurs (principe V :
   « mesure, puis écoute »).
5. Banques : programmes GM 0 et 21 de GeneralUser GS utilisés dans la maquette ;
   banque dédiée (piano d'ambiance, accordéon à anches) à évaluer avec V6.

## Reproduire

```sh
# exports sources (depuis le worktree du dépôt moteur, main) ; seeds 6 et 8
mkdir -p /tmp/accomp_etude/src
for s in 6 8; do target/release/harmonium-cli --export --bars 34 --style reel --seed $s --feet off \
  -s /Users/william.quintal/Project/bascanada/harmonium/assets/GeneralUser-GS.sf2 \
  --record-midi /tmp/accomp_etude/src/reel_s$s.mid --record-truth /tmp/accomp_etude/src/reel_s$s.json; done
cd /Users/william.quintal/Project/bascanada/harmonium_lab
S=/Users/william.quintal/Project/bascanada/harmonium/specs/012-vie-realisme/etudes/accompagnement
cp $S/*.py /tmp/accomp_etude/ && mkdir -p target/accomp_etude
.venv/bin/python /tmp/accomp_etude/measure_chords.py target/accomp_etude/chords.json
.venv/bin/python /tmp/accomp_etude/measure_parts.py target/accomp_etude/parts.json
.venv/bin/python /tmp/accomp_etude/tables.py target/accomp_etude/chords.json
# maquette : lit /tmp/accomp_etude/src/reel_sN.{mid,json} exportés avec harmonium-cli
.venv/bin/python /tmp/accomp_etude/mockup.py target/accomp_etude 6 8
```

`measure_parts.py` importe `measure_chords.py` depuis `/tmp/accomp_etude` : d'où la copie
des scripts à cet endroit. Ils lisent le moteur de `main` `ad7ad58` ; sur un moteur plus
récent, les chiffres de la maquette peuvent différer.

## Sources

- Freire, Armondes, Viana, Silva, « Strumming on an acoustic nylon guitar: microtiming,
  beat control and rhythmic expression in three different accompaniment patterns »,
  SMC 2018 — <https://musica.ufmg.br/lapis/wp-content/uploads/sites/9/2019/04/Freire-et-al-smc2018.pdf>
- Ornstein, « Instrumental Folk Music of Quebec: An Introduction », *Canadian Journal for
  Traditional Music*, 1982 — <https://cjtm.icaap.org/content/10/v10art2.html>
- « MAZ : trad modal pour temps modernes », *La Presse*, 14 avril 2012 —
  <https://www.lapresse.ca/arts/musique/201204/14/01-4515371-maz-trad-modal-pour-temps-modernes.php>
- Répertoire du patrimoine culturel du Québec, « Jeu d'accordéon diatonique » —
  <https://www.patrimoine-culturel.gouv.qc.ca/rpcq/detail.do?id=121&methode=consulter&type=imma>
  et « Podorythmie » — <https://www.patrimoine-culturel.gouv.qc.ca/detail.do?id=83&methode=consulter&type=imma>
- Wikipédia, « Accordéon diatonique » — <https://fr.wikipedia.org/wiki/Accord%C3%A9on_diatonique>
- Rohart, « Accordéon diatonique — la main gauche », 2017 (et commentaires) —
  <https://www.roh-art.com/2017/02/03/accord%C3%A9on-diatonique-la-main-gauche>
- MacDonald, « The Cape Breton Piano Style », *Silver Apple News* (Cranford Publications), 1999 —
  <https://www.cranfordpub.com/articles/cape_breton_piano.htm> ; livret *Traditional
  Fiddle Music of Cape Breton* 7040 — <https://cranfordpub.com/recordings/7040.pdf>
- Fraser, « The unique style of Cape Breton piano accompaniment », 2012 —
  <http://sessions.kimberleyfraser.com/the-unique-style-of-cape-breton-piano-accompaniment>
- The Session, « Cape Breton Piano Accompaniment » —
  <https://thesession.org/discussions/18187>
- Peghead Nation, « The Jig Rhythm, Part 1 » (Flynn Cohen) —
  <https://pegheadnation.com/string-school/courses/irish-backup-guitar/jig-rhythm-part-1/>
- National Guitar Academy, « How to strum a guitar » —
  <https://nationalguitaracademy.com/guides/beginners/how-to-strum-a-guitar> (extrait de recherche seulement)
- Wikipedia, « Stride piano » — <https://en.wikipedia.org/wiki/Stride_piano> ;
  « Quebec fiddle » — <https://en.wikipedia.org/wiki/Quebec_fiddle> ;
  « Fife and drum corps » — <https://en.wikipedia.org/wiki/Fife_and_drum_corps> (extrait de recherche seulement)
- « Quebec Tunes 4 » (recueil ABC avec accords, mise en garde sur la valeur des accords) —
  extrait de recherche seulement,
  <https://s6c2dcc185d7fe6ad.jimcontent.com/download/version/1735335540/module/12495485060/name/Quebec%20Tunes%204.pdf>
- Corpus du labo : manifeste `corpus/{trad,acadienne,britannique_coloniale}/manifest.jsonl`
  (provenance et droits : `corpus/sources.md`, principe VII).
- Dépôt : `harmonium_core/src/trad/accomp.rs`, `trad/generator.rs`,
  `profiles/README.md`, plan 012 (points 1, 2, 5), constitution (I, II, III, IV, V, VII).

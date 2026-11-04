# E12 — Une vraie mélodie jazz pour `wjazzd_swing`

Date : 2026-11-04. Branches : moteur `wquintal/e12-melodie-jazz` (après la fusion de B1, `b4ba224`,
`TUNING_VERSION` 30 → **32**, réservé), labo `wquintal/e12-melodie-jazz` (de `main`, `d01d684` ; commits `b275fec`,
`7700882`). Demande de `harmonium_training` (`demandes-training-010.md`, § E12) : étapes « lire le solo » et
« mesures à trous », lecture à vue en jazz. Données et labo seulement : **aucun fichier de `src/`** du moteur
n'a changé (hors la ligne de version).

## Livré

| Où | Quoi |
|---|---|
| Labo `references/wjazzd-melodie/` | `measures-melody.json` (statistiques agrégées de 348 solos swing, 26 086 mesures, 124 348 attaques), `cells.toml` (213 cellules), `compare-melody.{json,md}`, `README.md` (provenance, licence, N, validation) |
| Labo `backing/melody_{measures,cells,compare}.py`, `Makefile` | `make melody-measures`, `melody-cells`, `melody-check`, `melody-compare` ; `tests/test_backing_melody.py` (30 tests) |
| Moteur `profiles/wjazzd_swing/` | `cells.toml` remplacé par la banque ; `arrangement.toml` : `Melody` entre au tour 1 ; `profile.toml` : rôles de phrase, poids des rôles libres, registre, variation de chorus, saxophone ténor (programme 66), mixage de la voix, mention ODbL §4.3 dans `[meta] corpus` ; `README.md` |
| Moteur | `TUNING_VERSION` 32 (`tuning.rs`), référence D2 (`profile_d2_main.json`, seule la source change), `melody_difficulty_tests` (16 profils avec mélodie au lieu de 15), nouveau test de contrat `la_melodie_swing_joue_dans_l_accord_et_la_tessiture_du_profil_et_se_coupe` |

Le moteur est inchangé : une banque, un arrangement et des réglages de profil suffisent. `EnableMelody(false)` coupe
la mélodie (test : plus une note de la piste 0, la basse joue).

## Le corpus et la licence

WJazzD v2.1 (ODbL 1.0 + DbCL 1.0), sous-ensemble `rhythmfeel = SWING`, usage A (décision L0/F3) : **statistiques
agrégées seulement**. `measures-melody.json` ne porte que des comptes et des parts (rythme par créneau, pas
diatoniques, cibles par rapport à l'accord, longueurs de phrase) ; aucune suite de notes n'est écrite, l'index de
suites ne vit qu'en mémoire. Chaque cellule de la banque est **tirée d'un modèle** et **refusée** si l'une de ses
suites de notes figure dans un solo. `[meta] corpus` du profil porte la mention ODbL/DbCL et dit que la mélodie
est un *Produced Work* (§4.3). Aucun fichier sous droits ni base brute dans les deux diffs.

**N (suite de notes refusée).** Deux critères, tous deux sur des pas diatoniques (grain d'une cellule, plus
sévère qu'en demi-tons) : **6 notes** aux mêmes pas **et** mêmes durées, ou **7 notes** aux mêmes pas. Raison,
mesurée (`ngram_sharing`) : à 6 notes avec rythme, 85 % des suites distinctes du corpus n'apparaissent que dans
un seul solo (15 % sont partagées par au moins deux solistes : idiomes communs) ; à 7 notes sans rythme, 80 %.
Au-dessus, on ne peut pas refuser davantage : une cellule a huit croches. 51 tirages ont été refusés sur 53 175 ;
`make melody-check` et le test `test_aucune_cellule_de_la_banque_n_est_une_suite_de_n_notes_d_un_solo` donnent
0 cellule présente.

## Ce qui rend la banque jouable : la boucle fermée

Sous une grille, le modèle `loop` **ramène chaque note à la note d'accord la plus proche** (`fit_planned_melody` ;
README des profils : « les gammes d'un accord de boucle ne modifient pas les notes »). Les cellules d'un
modèle en degrés se ramassent alors en notes répétées : le bouche-trou en jouait 39 %, les solos 7 %. Le labo
simule ce rapprochement (`CHORD_BATTERY` : les accords des grilles du profil, pondérés) et refuse toute cellule
dont la part de répétitions rendues dépasse deux fois celle du corpus. Contrôle : simulé 0,27, mesuré 0,29 sur la
vérité terrain pour la banque sans filtre ; avec le filtre, 0,085 mesuré (corpus 0,070).

## Validation (8 graines, 176 mesures par graine ; `references/wjazzd-melodie/compare-melody.md`)

Distance à WJazzD, 0 = identique : mesures de `melody_measures.measure` (corpus comme généré), distances de
`bass_compare`. `bouche_trou` = l'ancienne banque, même profil, mélodie ajoutée à l'arrangement.

| Famille | bouche-trou | banque swing | niveau 1 | niveau 2 | niveau 3 |
|---|---|---|---|---|---|
| intervalles | 0,359 | **0,179** | 0,269 | 0,228 | 0,228 |
| rythme | 0,383 | **0,200** | 0,590 | 0,586 | 0,224 |
| accord (notes sur les temps) | 0,493 | 0,487 | 0,181 | 0,104 | 0,103 |
| registre | 0,424 | **0,241** | 0,433 | 0,402 | 0,296 |
| phrasé | 0,599 | 0,599 | 0,599 | 0,599 | 0,599 |
| **agrégée** | 0,435 | **0,323** | 0,371 | 0,333 | 0,241 |

| Mesure | WJazzD | bouche-trou | banque swing |
|---|---|---|---|
| Attaques par mesure | 4,77 | 7,96 | 4,89 |
| Notes répétées (intervalle 0) | 7,0 % | 39,0 % | 8,5 % |
| Intervalles de ±1 demi-ton | 22,1 % | 0,2 % | 1,9 % |
| Registre, centiles 5 / 50 / 95 | 53 / 65 / 77 | 63 / 71 / 79 | 58 / 65 / 73 |
| **Part des notes sur les temps forts dans l'accord courant** | **0,530** | 1,000 | 1,000 |
| Mesures avec attaque au créneau 7 (anticipation) | 0,579 | 0,994 | 0,371 |

**Notes sur les temps forts dans l'accord courant : 1,000, contre 0,530 au corpus.** C'est le moteur, pas la banque :
le rapprochement du modèle `loop` ramène toutes les notes (temps forts et croches) à l'accord. Hors rapprochement
(`chord_fit = false`, comme aux niveaux 1 à 3 de training), la part est de 0,51 à 0,56, proche du corpus. Je garde
le rapprochement : sans lui, les notes sont celles de la tonalité et non de la grille. Voir « Manques de schéma ».

**Compatibilité avec le filtre de difficulté (E10), quatre niveaux.** Niveaux 1 à 3 : filtre de référence du compte
rendu E10 (saut 4 / 5 / 7, attaques 4 / 6 / 8, durée 2 / 2 / 1, ambitus 7 / 9 / 12, `chord_fit = false`), 8 graines ×
176 mesures : **0 mesure hors bornes** par niveau ; pires valeurs saut / attaques / durée / ambitus 4 / 4 / 2 / 7,
5 / 4 / 2 / 9, 7 / 8 / 1 / 12 (la mesure de clôture de l'air, non filtrée, est écartée). La banque garde au moins
4 cellules par rôle à chacun de ces niveaux : la réécriture a de quoi choisir. Niveau 4 = le profil, sans filtre. Les
niveaux 1 et 2 s'écartent du corpus pour le rythme par construction (≤ 4 attaques, durée ≥ 2).

**Accompagnement.** La mélodie tire dans le RNG de session avant la graine d'accompagnement (S0) : les notes de
l'accompagnement de `wjazzd_swing` changent donc à graine égale (1 645 mesures sur 1 648 comparées à l'état d'avant),
comme B1 l'avait fait. Distance de la basse au corpus (`bass_compare`, 16 graines, premiers 103 mesures du fichier) :
**0,1469 avant, 0,1384 après** (blues : 0,1824 → 0,1755) : pas de perte. Habillages (tempo, tonalité, programmes,
voicings, kit) : ceux de B1, le test `seize_graines_donnent_au_moins_douze_habillages_distincts…` passe.

**Variété par graine** (`harmonium-lab seed-variety --chorus 4`, 16 graines, état d'avant `base` et état d'après,
`profile.toml` final) : accords, basse, batterie, comping : lignes égales à ±2 points d'un état à l'autre (basse :
16 séquences distinctes sur 16 graines, distance graduée inter 92 % avant comme après ; batterie 48 → 49 %).
Nouvelle ligne **mélodie** : 100 % des mesures actives, 44 % des mesures changées d'un chorus au suivant
(distance graduée 28 %), 4,0 chorus distincts sur 4, **16 séquences distinctes sur 16 graines** (distance graduée
inter 96 %), 7 tonalités sur 16 graines. La variation de reprise et de chorus (`repeat_variation` 0,25,
`tour_variation` 0,35, opérations `varied_end` et `repeated_notes`) est une **règle de métier**, sans mesure ; sans
elle, les quatre chorus rejouaient les mêmes notes (0 % de mesures changées). Pas d'opération `octave` : elle
ajoutait des sauts de plus d'une octave que les solos n'ont pas.

## Choix du profil (règles de métier, pas des mesures)

- **Phrases** : `Open`, trois mesures libres dont `CadHalf` à la mesure 4, reprise, fin sur la tonique (`CadFull`) en
  A et au blues, `CadHalf` en B. `phrase_ends` reste celui de la batterie (une phrase par partie) : B1 en dépend.
- **Poids des rôles libres** : conjoint 21, arpège 6, broderie 16 = parts mesurées des mesures ni ouvrantes ni
  cadentielles (0,208 / 0,059 / 0,157).
- **Registre** : centre 65, tessiture 53..77 (centiles 5, 50, 95 des solos). Avant ce réglage la médiane générée
  était 72 (distance de registre 0,55) ; 65 maintenant.
- **Timbre** : saxophone ténor (GM 66, banque 0 de GeneralUser GS : programme présent) ; mixage `-10 dB`, pan 0,2
  (au-dessus du comping −14 dB et de la basse −16 dB). **Non écouté** : à régler à l'écoute du propriétaire.
- **Cadences** : les cibles de `cad_half` (5, 2, 7 ; 0,45 / 0,30 / 0,25) sont la contrainte du moteur
  (`Cell::respects_cadence`), pas une mesure ; les parts de cadence du corpus servent de fréquence des rôles.

## Manques de schéma (liste pour A7, aucun contournement)

1. **Rapprochement du modèle `loop` aux notes d'accord seules.** Toutes les notes de la mélodie sont ramenées à
   l'accord ; pas de note de passage ni d'approche chromatique (21 % des temps forts du corpus sont chromatiques,
   26 % de la gamme hors accord). Demande : pour `loop`, ne ramener que les unités de `strong_units` aux notes
   d'accord et laisser les unités de `weak_units` sur la gamme publiée (`scale_guidance`, déjà calculée et publiée),
   par un champ du profil (`melody.fit = "chord_strong_scale_weak"`). La banque n'aurait qu'à sortir de la boucle
   fermée (`CHORD_BATTERY`).
2. **Pas de silence dans une cellule.** Le corpus a 20 % de temps sans note et des phrases de 5,5 temps (médiane) ;
   le généré joue sans pause (phrases de 701 temps). Demande : un jeton de silence dans la notation des cellules
   (« . »), ou des silences d'alternance par mesure (E11 existe au grain de la mesure).
3. **Pas de liaison avec la mesure précédente.** 33 % des mesures du corpus ne commencent pas par une attaque
   (anticipation liée, silence) ; une cellule commence toujours par une attaque.
4. **Croches seulement.** 20 % des attaques du corpus (seizièmes, ornements rapides) tombent sur un créneau pris
   et sont écartées du modèle ; `steps_per_unit = 2` n'a pas de seizième de cellule.

Ces quatre manques expliquent l'essentiel de la distance restante (famille `chord` 0,487 et `phrasing` 0,599, qui
ne bougent pas d'un bouche-trou à la banque). Ils ne sont pas contournables sans toucher au moteur.

## Vérification

Labo : `make test` (`pytest tests/`) **542 réussis, 32 ignorés** (+30 du fichier ajouté ; la vraie base est lue par un
test, sans quoi il serait ignoré).

Moteur (`ovh-inner`, machine partagée, toolchain `nightly-2026-10-03`) :

- `harmonium_core` : tous les fichiers de test passent (dont `wjazzd_swing_profile_tests`, 14 tests avec le nouveau, et
  `melody_difficulty_tests`, où `wjazzd_swing` entre dans la boucle des profils à mélodie : 16).
- `cargo test -p harmonium` avec et sans `--features odin` : tout passe **sauf `seek_to_bar_64_under_50ms`** :
  124, 140 et 154 ms mesurés en debug sur la machine partagée (minimum **124 ms**, limite 50 ms). Le test joue le profil par
  défaut (`reel_quebecois`), non touché par E12 : écart de machine, à remesurer à vide.
- `cargo test -p harmonium_bevy` : 11 réussis. `cargo check -p harmonium_studio` : sans erreur.
- `harmonium-cli` : `export_terminates` (200 mesures de chacun des 31 profils sous 300 s) **échoue à chaque essai**
  (deux lancements), tous les profils étant signalés non terminés, pas seulement `wjazzd_swing` : [INFERENCE]
  lenteur du build debug sous charge, pas un profil (un export release de `wjazzd_swing` de 88 mesures prend
  quelques secondes). Non confirmé sur l'état d'avant ; à rejouer à vide. La référence D2
  (`profile_d2_main.json`) est régénérée (`HARMONIUM_UPDATE_GOLDENS=1`, une ligne : seule la source change).
- Déterminisme : dumps `determinism_dump` avant / après : **183 / 189 identiques**, les 6 qui changent sont ceux de
  `wjazzd_swing` ; **natif = wasm : 189 / 189**.
- `cargo +nightly fmt --all --check` propre ; `cargo clippy --workspace` : aucune erreur, un avertissement existant
  (`finished_recordings`, hors de ce diff).
- Preuves hors ligne seulement : aucune écoute ni lecture audio (D5/D7).

## Liens d'écoute (à ouvrir par l'orchestrateur sur le Mac)

- `harmonium://open?profile=wjazzd_swing&seed=1` : si bémol, 219 BPM (`up_rapide`), piano et contrebasse, voicings
  `drop_3` ; ténor sur le blues puis l'AABA, à tempo rapide.
- `harmonium://open?profile=wjazzd_swing&seed=2` : mi bémol, 136 BPM (`medium_orchestre`), orgue et basse
  électrique, voicings sans fondamentale ; autres phrases, cadences et tempo moyen.

Ce qui change d'une graine à l'autre côté mélodie : le ténor (notes, phrases, cadences, tonalité) ; l'habillage est
celui de B1.

À juger à l'oreille : le saxophone ténor et son mixage (−10 dB), la fluidité d'une ligne sans silence, les notes
répétées résiduelles, l'effet du rapprochement tout-accord (ligne d'arpèges).

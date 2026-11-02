# R1 — Intégration des propositions P1/P2 aux profils de jeu

Date : 2026-11-02. Branche : `wquintal/r1-integration-p1-p2`, partie de `main` (`ff50b96`,
`TUNING_VERSION` 24). **`TUNING_VERSION` 25** : rupture assumée (principe I), les notes de quatre
profils changent. Données seulement : le seul changement sous `harmonium_core/src` est la
constante et sa ligne de doc (`tuning.rs`). Décision du propriétaire du 2026-11-02 (il a écouté
les rendus de 018) : intégration selon les recommandations de
[`p1-procedural-regles.md`](../018-grille-et-moteur-neutre/p1-procedural-regles.md) (§6.1, §6.4) et
de `harmonium_lab/references/p2-variete.md` (§3, §5).

## Résumé

| Question | Réponse |
|---|---|
| Qu'est-ce qui change ? | `nt_desert`, `isaac_basement`, `isaac_burning`, `isaac_downpour` (profils de données). `isaac_cellar` et les onze autres profils : octet pour octet identiques à `main`. |
| Preuve d'identité | `determinism_dump` : 98 fichiers ; **74 identiques**, **24 différents** = exactement les quatre profils × 3 graines × 2 réglages d'émotion. `--export --bars 64 --seed 5 --record-wav --record-midi` des 12 profils non listés : 12 WAV et 12 MIDI identiques (`cmp`) ; les quatre autres : WAV différents. |
| Variété par graine | Monte sur chaque couche intégrée (§2), sauf la basse d'`isaac_downpour` en signatures distinctes par partie (15,33 → 15,00 ; par mesure 2,12 → 3,12). |
| Distance au corpus | Dans la tolérance (1 point) sur chaque couche symbolique et sur l'audio global, **sauf l'audio d'`isaac_downpour` : −2,1**, imputé à son groove de batterie de densité 1 (ablation, §4). Gardé sur décision du propriétaire, retrait en deux lignes. |
| `isaac_burning`, P1 + P2 combinés | Respecte la règle : variété en hausse sur les trois couches, distance de couche symbolique +0,7 à +9,1, audio −0,2. Meilleur que chaque source seule. Aucune variante à écarter. |
| Mixage | Niveaux de piste mesurés avant et après : écarts de −0,8 à +0,8 LU (§5), aucun rééquilibrage. |
| Seek | `seek_to_bar_64_under_50ms` : 25,2 ms avant, 25,1 et 25,4 ms après (médiane de 15, débogage, machine au repos) ; limite 50 ms inchangée. |
| Studio web | Aucun nouveau preset ni nouvelle banque nommés : **pas de release `studio-resources-N`** nécessaire. |

## 1. Table d'intégration, profil × couche

| Profil | Accords | Basse | Batterie |
|---|---|---|---|
| `nt_desert` | **Intégré** (P1) : A2 boucle générée (4 accords mineurs chromatiques sur pédale de ré, poids 1), B boucle générée (4 accords majeurs chromatiques depuis la sensible, poids 1), A1 pédale G♯ (3) ou quinte (1). **Écarté** : rien de P2 (3 fenêtres non assignées) | **Rien**, décision du propriétaire. Écartés : rampe A1 de P1 (−6,3 de distance de couche), riff racine/quinte A2 de P1 (−1,1), 3 groupes de P2 sans soutien | **Intégré** (P2) : le groove de densité 1 de *Fläshyn* (`0K 2S 8K 10S 14S 20K 22S`), poids 1 à côté de l'écrit. **Écarté** : densité 0 (`0S 2S 10S 14S 20K`, sans grosse caisse, confiance 0,408) ; portée de tour 2 de P1 (sans intérêt de variété) |
| `isaac_basement` | **Intégré** (P2) : 2 boucles d'A2 (poids 1 chacune). **Écarté** : 3 fenêtres non assignées | **Intégré** (P2) : 1 motif d'A1 (poids 3). **Écarté** : 9 groupes non assignés, 6 sans soutien | **Rien** : les 4 patterns de P2 (−17,9 contre *Periculum*, deux sans grosse caisse au temps 1) |
| `isaac_burning` | **Intégré** P1 **et** P2 : C = 2 écrites + boucle de P2 (poids 2) + boucle générée (poids 2, `within`, intervalles pondérés) ; C2 = 2 écrites + boucle générée (poids 2). **Écarté** : pédale voisine b2 sur R (P1, hors du tableau du propriétaire) ; l'écart de P2 sur R (pédale) | **Intégré** P1 **et** P2 : R, motif P2 (poids 3) ; C, motifs P2 (2 et 1) ; C et C2, gabarits P1 (poids 3 chacun : 8 croches sur la fondamentale, octave 1, relance de 2 pas) | **Intégré** (P2) : groove de densité 1 (grosse caisse 0, 2, 6), poids 3. **Écarté** : densité 0 de P2 ; grooves par portée de partie C2 de P1 (couche −2,4) |
| `isaac_downpour` | **Intégré** (P2) : boucle de C (10 accords, poids 2). **Écarté** : 2 fenêtres non assignées | **Intégré** (P2) : 2 motifs d'A (poids 2 et 1) | **Intégré** (P2) : groove de densité 1 (`0K 4H 8S 12H`), poids 3. **Écarté** : densité 2 (`0H 2S 4K 8S 10K 14S`) |
| `isaac_cellar` | rien | rien | rien (un pattern de sept caisses claires, artefact) |

Poids : ceux des essais de P1 (§6.1, poids total des alternatives ajoutées = poids total des
écrites) et ceux de P2 **rééquilibrés** par `p2-enrich` (total des ajouts d'une partie ou d'une
densité = total des écrites, minimum 1). `isaac_burning` cumule donc P1 et P2 à leur poids propre :
C a trois alternatives de poids 2 contre deux écrites de poids 1 (une écrite sur trois). Les
propositions « présentes » de P2 (déjà équivalentes à une alternative écrite) ne sont pas
rajoutées, et aucun ajout n'apparaît dans les deux sources : pas de doublon.

Provenance : chaque ajout porte en commentaire TOML sa source (piste, mesures, minutage `m:ss`
dans la piste, confiance, soutien pour P2 ; section pour P1). Une ligne par profil touché dans
`profiles/README.md`, les README de `nt_desert` et `isaac_basement` (qui sert aux trois profils
Isaac : `isaac_burning` et `isaac_downpour` n'ont pas de README propre) et l'en-tête de leur
`profile.toml`.

### Ce qui ne s'exprime pas, ou n'est pas pris

| Élément | Cause |
|---|---|
| Fills pondérés (manque 3 de P2) et forme variable par graine (manque 4) | Toujours sans mécanisme moteur ; non traités par P1 (voir son §4). |
| Pédale `R` d'`isaac_burning` (écart de P2, manque 1) | Le mécanisme existe depuis P1 (`alternatives`), mais la fenêtre transcrite n'est pas une alternative définie ; le tableau du propriétaire ne la demande pas. Non intégrée. |
| Grooves de batterie par partie ou par tour (manque 2) | Exprimable depuis P1 (`part`, `tour`) ; écarté par décision du propriétaire (couche −2,4 sur `isaac_burning`, hors tolérance). |
| Rampe de basse d'A1 et riff d'A2 de `nt_desert` | Exprimables (gabarits de P1) ; écartés par décision du propriétaire et par la distance. |

Aucune proposition retenue ne demande un changement du schéma actuel.

## 2. Variété par graine, avant / après

16 graines × 64 mesures, premier air, `harmonium-lab seed-variety` (binaire de la branche, lancé
depuis `~/Project/bascanada/harmonium`). « Avant » est re-mesuré avec le même binaire : il
retrouve à l'identique les valeurs V1 de P2 (§2). `mesures` = part des mesures qui varient d'une
graine à l'autre, `parties` = part des parties, `dist./partie` = signatures distinctes par partie,
`dist./mesure` = par mesure.

| Profil | Couche | mesures | parties | dist./partie | dist./mesure |
|---|---|---:|---:|---:|---:|
| `nt_desert` | accords | 0 → **66 %** | 0 → **86 %** | 1,00 → **4,14** | 1,00 → 2,00 |
| | basse | 25 → 30 % | 29 → 43 % | 3,29 → 4,29 | 1,25 → 1,31 |
| | batterie | 0 → **55 %** | 0 → **71 %** | 1,00 → **1,71** | 1,00 → 1,55 |
| | les trois couches | 0 → 20 % | 0 → 43 % | | |
| `isaac_basement` | accords | 38 → 44 % | 75 → 75 % | 1,75 → 2,08 | 1,38 → 1,56 |
| | basse | 41 → **59 %** | 100 → 100 % | 4,92 → **6,25** | 1,47 → 1,83 |
| | batterie (inchangée) | 70 → 70 % | 92 → 92 % | 1,92 → 1,92 | 1,70 → 1,70 |
| | les trois couches | 0 → 6 % | 67 → 67 % | | |
| `isaac_burning` | accords | 23 → **36 %** | 38 → 38 % | 1,38 → **3,62** | 1,23 → 2,00 |
| | basse | 100 → 100 % | 100 → 100 % | 11,62 → **14,75** | 2,44 → **5,22** |
| | batterie | 88 → 88 % | 100 → 100 % | 2,00 → **3,00** | 1,88 → 2,75 |
| | les trois couches | 22 → **31 %** | 38 → 38 % | | |
| `isaac_downpour` | accords | 48 → 52 % | 100 → 100 % | 2,00 → 2,33 | 1,48 → 1,60 |
| | basse | 73 → 79 % | 100 → 100 % | 15,33 → **15,00** | 2,12 → **3,12** |
| | batterie | 88 → 88 % | 100 → 100 % | 2,00 → **3,00** | 1,88 → 2,75 |
| | les trois couches | 42 → 44 % | 100 → 100 % | | |

Lecture. Le critère du plan (les trois couches ensemble) ne bouge que par les mesures, comme
P1 et P2 l'annonçaient : la part de parties est déjà saturée sur trois profils. Le gain est dans
la profondeur (accords de `nt_desert` 1,0 → 4,1 signatures par partie ; basse d'`isaac_burning`
2,4 → 5,2 par mesure). La basse d'`isaac_downpour` perd 0,33 signature **par partie** (le motif de
P2 à une seule note tenue remplace des variantes plus finement différenciées) mais en gagne une
par mesure : mentionné dans le compte rendu de P2 (15,33 → 15,00), repris tel quel.

## 3. Distance au corpus, avant / après

`make style-distance` du labo, mêmes paramètres que P1 et P2 : graines 1–10, 48 mesures,
`type=zone`, `GeneralUser-GS.sf2` (et `nt_desert.sf2` pour `nt_desert`), mêmes pistes de référence,
cache audio du corpus recopié du labo. Chaque variante a son dossier de travail (les rendus du
labo sont mis en cache sous le nom du profil : les réutiliser aurait mélangé les moteurs). « Avant »
est re-mesuré : retrouve P2 (V1) à l'identique. Score 0–100, 100 = identique ; `Δ` contre l'avant ;
tolérance 1 point (bruit mesuré par P2 : 0,0 à 0,6). `boucles` mesure la répétition à la longueur
de boucle de la référence : elle baisse dès qu'on ajoute une alternative, par construction, et
**n'entre pas dans la règle** (P1 §6.3, P2 §4.2).

| Profil | Audio | Symbolique | basse | batterie | accords | boucles |
|---|---|---|---|---|---|---|
| `nt_desert` | `nt_desert.sf2` 57,5 → 57,0 (−0,5) ; GeneralUser 58,0 → 57,8 (−0,2) | 48,4 → 47,4 (−1,0) | 43,1 → 43,2 (+0,1) | 30,4 → 31,2 (+0,8) | 77,0 → 78,0 (+1,0) | 43,0 → 37,2 (−5,8) |
| `isaac_basement` | 40,4 → 40,0 (−0,4) | 65,9 → 66,0 (+0,1) | 69,2 → 69,0 (−0,2) | 70,3 → 70,3 (0) | 71,1 → 71,2 (+0,1) | 53,2 → 53,4 (+0,2) |
| `isaac_burning` | 39,6 → 39,4 (−0,2) | 60,6 → 62,3 (+1,7) | 59,1 → 68,2 (+9,1) | 76,7 → 79,9 (+3,2) | 61,2 → 61,9 (+0,7) | 45,3 → 39,0 (−6,3) |
| `isaac_downpour` | 61,4 → 59,3 (**−2,1**) | 58,8 → 61,8 (+3,0) | 77,7 → 79,2 (+1,5) | 44,6 → 58,1 (+13,5) | 86,7 → 86,5 (−0,2) | 26,2 → 23,4 (−2,8) |

`nt_desert` : le symbolique global (−1,0) est à la limite, tiré par `boucles` (hors règle) ;
les trois couches se tiennent chacune à +0,1 à +1,0, et l'audio à −0,5 / −0,2. Les deux sources
s'additionnent : P1 accords −0,2, P2 batterie −0,8 (P2 §4.2) donnent −1,0.

Biais d'optimisme (P1 §6.3, P2 §4.2) : les hausses symboliques de la basse et de la batterie
d'`isaac_burning` et d'`isaac_downpour` viennent des mêmes transcriptions que la référence ; les
baisses ne sont pas circulaires.

## 4. `isaac_burning` : P1 + P2 contre chaque source seule

La règle de P2 (variété en hausse, distance de couche et audio global à −1 point au plus) décide
de la variante. Trois variantes mesurées sur le même profil (copies jetables, 16 graines et
10 graines) :

| Variante | accords dist./partie | basse dist./partie, /mesure | batterie dist./partie | audio | basse | batterie | accords | symbolique |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| Avant | 1,38 | 11,62 ; 2,44 | 2,00 | 39,6 | 59,1 | 76,7 | 61,2 | 60,6 |
| P1 seul (accords + gabarits) | 4,12 | 11,62 ; 4,47 | 2,00 | 40,4 | 61,2 | 76,7 | 61,1 | 60,8 |
| P2 seul (boucle, 3 motifs, densité 1) | 1,62 | 14,62 ; 3,69 | 3,00 | 38,5 (−1,1) | 64,5 | 79,9 | 61,4 | 61,7 |
| **P1 + P2** | 3,62 | 14,75 ; 5,22 | 3,00 | 39,4 (−0,2) | 68,2 | 79,9 | 61,9 | 62,3 |

Aucune couche ne dégrade au-delà de la règle ; P2 seul passait l'audio de 0,1 point au-delà de
la tolérance (−1,1), la combinaison la ramène à −0,2. La variante complète est donc gardée.

### Ablation `isaac_downpour` (audio −2,1)

| Variante | audio | symbolique | basse | batterie | accords |
|---|---:|---:|---:|---:|---:|
| Avant | 61,4 | 58,8 | 77,7 | 44,6 | 86,7 |
| Accords + basse seuls | **61,6** (+0,2) | 59,5 | 79,2 | 44,6 | 86,5 |
| Batterie de densité 1 seule | **59,0** (−2,4) | 61,1 | 77,7 | 58,1 | 86,7 |
| Complet (intégré) | 59,3 (−2,1) | 61,8 | 79,2 | 58,1 | 86,5 |

Le coût audio vient **tout entier** du groove de batterie de densité 1, pas de la densité 2
(écartée) comme P2 le supposait. Le propriétaire a demandé la densité 1 ; je la garde et le
signale. Retrait : supprimer le `[[patterns]]` « R1 » de `isaac_downpour/percussion.toml`
(accords et basse restent dans la règle) ; la distance symbolique de la batterie retombe à 44,6.

## 5. Mixage, rendus de contrôle et écoute

Un rendu de 64 mesures, graine 5, par profil touché et par version (avant / après), via
`harmonium-cli --export --bars 64 --seed 5`, banque `GeneralUser-GS.sf2` (`nt_desert.sf2` pour
`nt_desert`), lancé depuis `~/Project/bascanada/harmonium`. Les WAV restent dans
`/tmp/harmonium-019/r1/listen/` ; les MP3 320 k, avec un gain fixe par profil (identique avant et
après) qui ramène l'avant à −21 LUFS (crêtes −5,0 à −6,2 dBFS), sont dans
`harmonium_lab/corpus/ecoute_019/r1/`. Mixage mesuré (`--solo-track`, `ffmpeg ebur128`,
intégré, LUFS, avant → après, graine 5, 64 mesures) :

| Profil | Mix | Chords | Bass | PercussionLow | PercussionHigh | Melody |
|---|---|---|---|---|---|---|
| `nt_desert` | −17,5 → −17,5 | −21,7 → −21,8 | −18,3 → −18,3 | −28,8 → −28,7 | −32,0 → −31,6 | −28,0 → −28,2 |
| `isaac_basement` | −22,9 → −22,9 | −30,2 → −30,2 | −24,3 → −24,4 | −35,8 → −35,8 | −33,1 → −33,1 | −41,1 → −41,1 |
| `isaac_burning` | −21,8 → −22,1 | −25,0 → −24,9 | −24,8 → **−25,5** | −35,0 → −34,9 | −32,8 → −32,9 | −40,5 → −40,4 |
| `isaac_downpour` | −28,1 → −28,1 | −31,6 → −31,6 | −32,2 → −32,1 | −37,6 → **−36,8** | −35,2 → −35,3 | −40,0 → −40,0 |

Aucun écart au-delà de 0,8 LU ; rien à corriger au mixage. Les niveaux absolus (mélodie à −40 LUFS
en solo sur les profils Isaac, mix d'`isaac_downpour` à −28 LUFS) sont ceux de `main`, non touchés.

Section d'écoute : `http://192.168.8.171:8080/#phase-019-r1` (fichier
`harmonium_lab/corpus/index.html`, non suivi). Moments à écouter (minutes:secondes ; mesures dont
la couche ne joue dans aucune des 16 graines de l'avant à la même mesure) :

| Profil, graine | Couche | Moments |
|---|---|---|
| `nt_desert`, 5 | accords | 0:00–0:13,7 (A1, autre pédale) · 0:15,4 · 0:18,9 · 0:22,3 · 0:25,7 (A2, boucle chromatique) · 0:58,3–1:01,7 · 1:05,1–1:36,0 (B) · 1:37,7 · 1:41,1 · 1:44,6 · 1:48,0 |
| | basse (suit les accords) | 0:58,3–1:00,0 · 1:05,1–1:06,9 |
| | batterie | 0:54,9–1:06,9 · 1:08,6–1:20,6 · 1:22,3–1:34,3 |
| `isaac_basement`, 5 | basse | 0:00–0:03,4 · 0:10,3–0:13,7 · 0:54,9–0:58,3 · 1:05,1–1:12,0 · 1:18,9–1:22,3 (accords et batterie identiques à cette graine) |
| `isaac_basement`, 10 | accords | 0:13,7–0:18,9 · 0:27,4–0:32,6 · 1:22,3–1:27,4 · 1:36,0–1:41,1 |
| | basse | 0:00–0:10,3 · 0:54,9–1:05,1 · 1:08,6–1:18,9 |
| `isaac_burning`, 5 | accords | 0:13,7–0:15,4 · 0:17,1–0:22,3 · 0:25,7–0:27,4 · 1:22,3–1:24,0 · 1:25,7–1:30,9 · 1:34,3–1:36,0 |
| | basse | 0:03,4–0:10,3 · 0:13,7–0:15,4 · 0:17,1–0:24,0 · 0:25,7–0:27,4 · 0:30,9–0:37,7 · 0:46,3–0:48,0 · 0:49,7–0:53,1 · 0:58,3–1:05,1 · 1:12,0–1:18,9 · 1:22,3–1:24,0 · 1:25,7–1:32,6 · 1:34,3–1:36,0 · 1:39,4–1:46,3 |
| | batterie | 0:00–0:12 · 0:13,7–0:25,7 · 0:27,4–0:39,4 · 0:54,9–1:06,9 · 1:08,6–1:20,6 · 1:22,3–1:34,3 · 1:36,0–1:48,0 |
| `isaac_downpour`, 5 | basse | 0:00–0:03,3 · 0:06,7–0:10,0 · 0:13,3–0:23,3 |
| | batterie | 0:00–0:11,7 · 0:13,3–0:25,0 · 0:26,7–0:38,3 · 0:40,0–0:51,7 |
| `isaac_downpour`, 1 | accords | 0:53,3–0:56,7 · 1:06,7–1:13,3 |
| | basse | 0:03,3–0:26,7 · 1:06,7–1:10,0 |
| | batterie | 0:00–0:11,7 · 0:13,3–0:25,0 · 0:26,7–0:38,3 · 0:40,0–0:51,7 · 0:53,3–1:05,0 · 1:06,7–1:18,3 |

À juger : (1) la boucle chromatique de `nt_desert` sur pédale de ré (déjà entendue en 018) ;
(2) la relance de basse d'`isaac_burning` avant chaque changement d'accord : naturelle ou
mécanique ? (3) la batterie de densité 1 d'`isaac_downpour`, qui coûte −2,4 en distance audio.

## 6. Contrat et suites

| Vérification | Résultat |
|---|---|
| `cargo test --workspace -q --no-fail-fast` | 787 réussis, 0 échec, 2 ignorés (101 suites) |
| `cargo test -p harmonium --features odin` ; `cargo test -p harmonium_bevy` | 118 et 11 réussis |
| `cargo check -p harmonium_studio` | passe |
| `harmonium_determinism/scripts/run.sh` (wasm-bindgen 0.2.108, installé hors dépôt) | **natif = wasm : 98 / 98** (les parties `generated`, la pédale à alternatives et les gabarits de basse sont dans la matrice par `nt_desert` et `isaac_burning`) |
| `determinism_dump` branche contre `main` (`git archive ff50b96`) | 74 identiques, 24 différents : `nt_desert`, `isaac_basement`, `isaac_burning`, `isaac_downpour` × graines 1–3 × `hi`/`mid`, rien d'autre |
| `cargo clippy --workspace --all-targets`, décomptes d'avertissements par message comparés à `main` | aucun avertissement nouveau : mêmes listes (89 lignes de décompte) ; l'unique écart est un `unwrap()` de plus du côté `main`, dans ma sonde de durée jetable copiée là |
| `cargo +nightly fmt --all -- --check` | propre |
| `seek_to_bar_64_under_50ms` (profil par défaut, graine 42, `deterministic_seek(64)`, build de `cargo test` (débogage), médiane de 15 passes, machine au repos) | 25,2 ms (`main`) → 25,1 et 25,4 ms (deux séries). Profils touchés (même mesure) : `nt_desert` 8,4 → 8,7 et 8,8 ; `isaac_basement` 4,9 → 4,9 et 4,8 ; `isaac_burning` 5,2 → 5,4 et 5,3 ; `isaac_downpour` 15,3 → 15,1 et 15,1 ms. Seuil inchangé (50 ms), test vert. |

Attentes de tests qui dépendaient des notes de ces profils :

| Fichier | Changement |
|---|---|
| `harmonium_core/src/tuning.rs` | `TUNING_VERSION` 24 → 25 et sa ligne de doc |
| `harmonium_core/tests/set_contract_tests.rs` | `assert_eq!(TUNING_VERSION, 25)` |
| `harmonium_cli/tests/fixtures/profile_d2_main.json` | régénéré (`HARMONIUM_UPDATE_GOLDENS=1`) : seul le champ `source` (`TUNING_VERSION:25`) change, les 1000 mesures de `reel_quebecois` sont identiques ; `profile_d2_fixed.json` (source figée à 20) inchangé |
| `harmonium_core/tests/nt_desert_profile_tests.rs` | deux tests supposaient une pédale G♯ fixe en A1 et un Dm tenu huit mesures en A2 : ils vérifient maintenant l'identité d'accord publiée (A1 : pédale G♯ ou sa quinte, égale à la racine publiée ; A2 : basse de ré, ouverture en Dm, triade plus basse d'un accord slash, nappe ré-attaquée seulement au changement d'accord, trois ou quatre notes) |
| Goldens `harmonium_determinism` | aucun fichier attendu n'est versionné : le harnais compare natif et wasm (`run.sh`) ; dumps régénérés dans `target/determinism/` |

Les autres profils, fixtures et goldens de `harmonium_core`, `harmonium_host`, `harmonium_studio` et
`harmonium_bevy` n'ont pas bougé.

## 7. Commandes

```sh
# binaire de la branche, rendus depuis le checkout principal (assets/)
cargo build --release -p harmonium-cli
cd ~/Project/bascanada/harmonium
CLI=~/orca/workspaces/harmonium/r1-integration-p1-p2/target/release/harmonium-cli
A=$PWD/assets ; P=~/orca/workspaces/harmonium/r1-integration-p1-p2/profiles
$CLI --export --bars 64 --seed 5 --profile $P/nt_desert --soundfont $A/nt_desert.sf2 \
  --percussion-soundfont $A/nt_desert.sf2 --record-wav /tmp/harmonium-019/r1/listen/nt_desert_apres_s5.wav

# variété par graine (copies avant : `git archive ff50b96 profiles/`)
cd ~/Project/bascanada/harmonium_lab
.venv/bin/harmonium-lab seed-variety --profile <dossier> ... --seeds 16 --bars 64 \
  --engine-dir ~/Project/bascanada/harmonium --engine-bin $CLI --label R1 \
  --work-dir /tmp/harmonium-019/r1/variety -o /tmp/harmonium-019/r1/variety/seed-variety-R1.json --markdown

# distance (un dossier de travail par variante, cache du corpus partagé par lien)
make style-distance ENGINE_DIR=~/orca/workspaces/harmonium/r1-integration-p1-p2 PROFILE=<dossier> \
  REF=isaac-basement TRACKS=36_fundamentum STYLE_CORPUS=$CORPUS/isaac CONTEXT="type=zone" \
  BANKS="generaluser=$A/GeneralUser-GS.sf2" STYLE_DIST_DIR=/tmp/harmonium-019/r1/style/<variante> \
  DIST_OUT=/tmp/harmonium-019/r1/dist/<variante>.json
# pistes : isaac_basement 02_diptera_sonata,21_periculum ; isaac_downpour 47_river_of_despair ;
# nt_desert REF=nt-desert TRACKS=02_flashyn,19_a_way STYLE_CORPUS=$CORPUS/nuclear_throne
#   BANKS="nt_desert=$A/nt_desert.sf2 generaluser=$A/GeneralUser-GS.sf2"

# mixage : un rendu par piste (--solo-track Melody|Chords|Bass|PercussionLow|PercussionHigh), puis
ffmpeg -nostats -i piste.wav -af ebur128=peak=true -f null -

# preuves de non-régression
cargo run --release -p harmonium_determinism --bin determinism_dump -- /tmp/harmonium-019/r1/dump-branch
WASM_BINDGEN=/tmp/harmonium-019/r1/wb/bin/wasm-bindgen harmonium_determinism/scripts/run.sh
```

Sorties lourdes : `/tmp/harmonium-019/r1/` (`variety/`, `dist/`, `listen/`, `solo/`, `dump-main`,
`dump-branch`, `unchanged/`, `lufs.json`, `mp3.json`).

## 8. Limites

- Une seule écoute par profil et par graine (5, plus 10 et 1 pour les accords d'`isaac_basement`
  et d'`isaac_downpour` qui ne changent pas à la graine 5) ; aucune écoute humaine revendiquée.
- Les gabarits de basse d'`isaac_burning` ont des vélocités (96 / 80, relance 90) et une durée de
  croche choisies ici : P1 §6.1 donne « huit croches sur la fondamentale, octave 1, relance de
  2 pas » sans vélocités ; elles reprennent celles du motif écrit de C.
- Les 16 graines mesurent le premier air seulement ; `--chorus` (V0) concerne les grilles
  d'accords des profils d'accompagnement, pas ces profils de jeu en tours.
- Biais d'optimisme de la distance symbolique : voir §3.

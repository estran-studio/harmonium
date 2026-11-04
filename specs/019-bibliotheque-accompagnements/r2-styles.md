# R2-styles — Habillage par graine des neuf profils GMD (phase 019, vague 4)

Date : 2026-11-03. Branche `wquintal/r2-styles`, partie de `main` à `eecbd3f`. **`TUNING_VERSION` 29** (rupture assumée :
principe I). Aucune ligne de moteur : un champ de `tuning.rs` (la version et sa ligne de doc), des **données** (neuf
profils), des scripts, trois fichiers de test, ce compte rendu. Les 98 fichiers de `harmonium_determinism` des autres profils
sont identiques à `main` (preuves plus bas).

## Ce qui est livré

Pour `gmd_rock`, `gmd_pop`, `gmd_hiphop`, `gmd_soul`, `gmd_reggae`, `gmd_dance_disco`, `gmd_afrobeat` (B8), `gmd_funk` et
`gmd_neworleans_funk_secondline` (B4 + B7) :

1. **`session.toml`** (`[files] session`) : 9 à 10 groupes de variantes tirés indépendamment par graine : `tempo` (sous-plages
   des centiles de BPM de la source), `feel` (swing léger, hip-hop et rock seulement), `tonalite`, `comping` (programme des
   accords), `basse` (programme), `kit` (banque 128), `voicing` (famille), `registre` (comping ±1 octave), `energie`
   (multiplicateurs d'intensité et de densité), `arc` (`[arc]` du profil ou arc nommé). Trois arcs nommés par profil :
   `orchestre` (le comping et le kit changent de chorus en chorus et reviennent à ceux de la session : A2, orchestration par
   chorus), `direct` (sans exposition), `montee` (un cran par chorus). **De 25 920 à 311 040 habillages possibles** par profil.
2. **Plus de `[fixed_tonality]`** dans ces profils : sans cela la variante de tonalité est ignorée (`generator.rs`,
   `draw_key_mode`). `melody.key_modes`, `dynamics.color` et `dynamics.parallel_modes` portent les tonalités tirables ;
   `dark_below` / `bright_above` (0,1 / 0,9) écartent la bascule de mode de la valence neutre (0,3), comme la fixture de l'A2.
   Training installe son propre `[fixed_tonality]` : l'hôte gagne, la variante de tonalité est alors ignorée (test de contrat).
3. **A6, micro-timing par voix** : `performance.tracks.Percussion*.voices`, médianes en ms par rôle de batterie du GMD (groupe
   4/4 droit, pondérées par l'effectif) converties en fraction de pas au tempo par défaut du profil ; `jitter` uniforme
   d'écart-type mesuré, plafonné à 0,1 (borne du schéma), **sauf `gmd_funk` et `gmd_neworleans_funk_secondline`** (la basse à notes
   de passage partage le tirage : contrat `passing_keeps_neighbour_tracks…`). Remplace les `offsets` par pas de B4 + B7.
4. **A6, coups probabilistes** : en dessous du seuil certain (0,5), une grosse caisse, une caisse claire, un cross-stick (dès
   0,25 à la densité 1 et 0,08 à la densité 2 ; funk et New Orleans : 0,08 à la densité 2 seulement) et une charleston ouverte (dès
   0,20, densité 2) jouent avec leur fréquence mesurée dans le groupe (`probability`) au lieu d'un coup certain dès 0,25.
5. **A5, voicings** : `chords.color` (tension 9, 11 ou 13) et `chords.register_drift` déclarés ; les familles tirées vont de
   `close_triad` à `drop2`, `shell`, `guide_tones`, `upper_structure` et `open`.
6. **Swing léger** là où la source en a : hip-hop 0,59 (10 fichiers `fill` « swing », médiane 0,5896) et rock 0,60 (7 fichiers `beat`
   « swing », médiane 0,6125), à poids 1 sur 5 ou 6. Les patterns de ces deux profils sont mesurés sur les fichiers droits : le
   swing léger est un choix de feel de la source, pas une mesure des grooves joués.
7. README des neuf profils (section « Habillage par graine » : groupes, poids, source de chaque valeur, table de micro-timing),
   `profiles/README.md`, `plan.md`.

Scripts et données (`specs/019-bibliotheque-accompagnements/r2-styles/`) : `habillage.py` (toutes les valeurs, écriture de
`session.toml`, micro-timing mesuré dans le labo, section de README) ; `appliquer_b47.py` (funk et New Orleans, écrits à la main
par B4 + B7) ; `habillage_mesure.py` (habillages distincts, lus dans la vérité terrain du CLI) ; `comparer_variete.py`
(tableau avant / après). Les générateurs de B8 (`b8/generer.py`, `emit.py`, `gmd_batterie.py`, `distance_gmd.py`) et
`b47_batterie.py` écrivent maintenant le profil habillé.

### Programmes vérifiés dans GeneralUser GS

`phdr` de `assets/GeneralUser-GS.sf2` lu : banque 0, programmes 0, 3, 4, 5, 7, 11, 16, 18, 21, 24, 25, 27, 28, 29, 30, 32, 33, 34, 35,
36, 38, 39, 58, 61 ; banque 128, kits 0, 8, 16, 24, 25, 32 (« Standard 1 », « Room », « Power », « Electronic », « 808/909 »,
« Jazz »). `habillage.py` refuse tout programme absent de cette table.

## Preuve : habillages distincts sur 16 graines (V2, D6)

`habillage_mesure.py`, binaire de la branche, vérité terrain des 16 premières mesures de chaque graine ; un habillage = (grappe
de tempo à ± 8 BPM, tonique, mode, programmes des accords, de la basse, du kit, famille de voicings, swing). Seuil du brief : ≥ 8.
Avant R2 : **1 habillage** par profil (tempo, tonalité, programmes, kit et voicings fixes).

| Profil | Habillages distincts / 16 | Grappes de tempo (plage tirée) | Tonalités | Comping | Basse | Kit | Voicings |
|---|---:|---|---:|---:|---:|---:|---:|
| `gmd_rock` | **16** | 4 (90 à 129 BPM) | 4 | 3 | 2 | 3 | 2 |
| `gmd_pop` | **16** | 2 (83 à 142 BPM) | 3 | 4 | 4 | 4 | 3 |
| `gmd_hiphop` | **16** | 3 (70 à 99 BPM) | 6 | 3 | 3 | 4 | 3 |
| `gmd_soul` | **15** | 2 (80 à 105 BPM) | 4 | 4 | 3 | 3 | 4 |
| `gmd_reggae` | **16** | 3 (83 à 121 BPM) | 5 | 4 | 3 | 3 | 2 |
| `gmd_dance_disco` | **16** | 2 (120 à 136 BPM) | 5 | 3 | 4 | 4 | 3 |
| `gmd_afrobeat` | **15** | 3 (86 à 109 BPM) | 4 | 3 | 3 | 3 | 3 |
| `gmd_funk` | **16** | 4 (80 à 126 BPM) | 4 | 4 | 4 | 4 | 3 |
| `gmd_neworleans_funk_secondline` | **15** | 3 (82 à 118 BPM) | 4 | 3 | 3 | 3 | 3 |

Le comping lu est celui de la première mesure : un arc `orchestre` y met le programme de son exposition (la variété réelle de
programmes est plus grande sur un air complet). Test de contrat : `gmd_session_profile_tests.rs`.

## Preuve : `seed-variety --chorus 4` avant / après (16 graines × 4 chorus)

`harmonium-lab seed-variety --profile profiles/<id> --seeds 16 --chorus 4`, binaire de `main`/de la branche (`/tmp`, cli-avant et
cli-apres), lancé depuis `~/Project/bascanada/harmonium`. « Suites » : suites distinctes sur 16 graines ; « Distance » : distance
graduée moyenne entre graines (`inter.graded`) ; « Chorus » : part de mesures changées d'un chorus au suivant (`intra.adjacent.exact`).
`comparer_variete.py` écrit le tableau complet (54 lignes) ; ci-dessous les couches qui changent le plus et les deux qui peuvent
régresser.

| Profil | Orchestration : suites | Orchestration : distance | Comping voicings : distance | Accompagnement : distance | Batterie : distance | Batterie : chorus |
|---|---|---|---|---|---|---|
| `gmd_rock` | 1 → 10 | 0% → 32% | 65% → 89% | 57% → 65% | 47% → 58% | 83% → 91% |
| `gmd_pop` | 1 → 14 | 0% → 49% | 56% → 97% | 52% → 63% | 33% → 36% | 84% → 88% |
| `gmd_hiphop` | 1 → 12 | 0% → 38% | 63% → 97% | 60% → 66% | 48% → 50% | 82% → 85% |
| `gmd_soul` | 1 → 13 | 0% → 41% | 71% → 98% | 55% → 62% | 20% → 28% | 61% → 73% |
| `gmd_reggae` | 1 → 12 | 0% → 40% | 58% → 95% | 49% → 57% | 18% → 28% | 63% → 91% |
| `gmd_dance_disco` | 1 → 12 | 0% → 39% | 73% → 98% | 52% → 58% | 20% → 29% | 67% → 71% |
| `gmd_afrobeat` | 1 → 12 | 0% → 38% | 58% → 96% | 50% → 61% | 28% → 41% | 79% → 100% |
| `gmd_funk` | 1 → 13 | 0% → 42% | 68% → 95% | 64% → 69% | 54% → 44% | 90% → 78% |
| `gmd_neworleans_funk_secondline` | 1 → 9 | 0% → 27% | 68% → 98% | 60% → 66% | 31% → 33% | 87% → 77% |

Lecture :

- **Orchestration** (A2, `program_change` du MIDI) : de 1 suite distincte sur 16 graines (0 %) à 9–14 ; c'est ce que « New seed » fait
  entendre en plus. Elle ne monte pas à 16 / 16 : les 16 graines tirent des triplets de programmes qui se recoupent.
- **Voicings du comping** : la distance entre graines passe de 56–73 % à 95–98 % (familles tirées, `color`, `register_drift`) ; pour
  `gmd_rock` (voicings `close_triad` le plus souvent) de 65 à 89 %.
- **Batterie** : en hausse pour sept profils sur neuf (coups probabilistes, arc nommé) ; **en baisse pour `gmd_funk`** (distance
  entre graines 54 → 44 %, chorus changés 90 → 78 %) et pour `gmd_neworleans_funk_secondline` (chorus changés 87 → 77 %) : les
  notes fantômes de densité 2 jouent à 8–50 % au lieu de 100 % dès 0,3, donc un chorus à la densité 1 ressemble davantage à un autre.
  Les 16 séquences sont toujours distinctes (16 / 16), la batterie de ces deux profils reste très au-dessus de `wjazzd_swing` (V0 : 11 / 16).
- **Basse** : 2–6 % de mesures changées d'un chorus au suivant (inchangé : le plan de motifs est fixé par air, manque moteur déjà
  relevé par B4 + B7 et B8). La distance entre graines monte de 1 à 7 points (instrument et registre).
- Comping, rythme : distance entre graines à ± 4 points (poids des rythmes inchangés), mesures changées d'un chorus au suivant 2 à 6 points
  plus bas (83 → 77 % pour le rock) : la part de mesures où l'arc nommé impose les mêmes étiquettes est un peu plus grande.

## Preuve : distance au GMD (règle de P2 : pas plus de 1 point de perte)

### B8 : `b8/distance_gmd.py` (Bray-Curtis des fréquences d'attaque par pas, 0 = identique, 16 graines × 47 mesures)

Avant : binaire de `main`, profils de B8. Après : binaire de la branche, **profil habillé** (le tempo, l'instrument, le kit ne se
voient pas dans les attaques par pas ; coups probabilistes et arcs nommés oui). Moyenne des quatre métriques (tout, grosse caisse,
caisse claire, charleston) ; **plus petit = plus proche du GMD**.

| Profil | Moyenne avant | Moyenne après | Grosse caisse | Caisse claire | Charleston |
|---|---:|---:|---|---|---|
| `gmd_rock` | 0.230 | **0.184** | 0.218 → 0.153 | 0.243 → 0.221 | 0.373 → 0.302 |
| `gmd_pop` | 0.212 | **0.171** | 0.301 → 0.220 | 0.211 → 0.180 | 0.253 → 0.212 |
| `gmd_hiphop` | 0.149 | **0.147** | 0.181 → 0.202 | 0.212 → 0.169 | 0.125 → 0.124 |
| `gmd_soul` | 0.285 | **0.252** | 0.346 → 0.314 | 0.309 → 0.302 | 0.397 → 0.304 |
| `gmd_reggae` | 0.251 | **0.179** | 0.278 → 0.161 | 0.478 → 0.326 | 0.078 → 0.079 |
| `gmd_dance_disco` | 0.164 | **0.163** | 0.210 → 0.215 | 0.151 → 0.159 | 0.206 → 0.189 |
| `gmd_afrobeat` | 0.211 | **0.152** | 0.339 → 0.239 | 0.329 → 0.228 | 0.116 → 0.109 |

Aucune perte : sept profils sur sept ne baissent pas (rock −0,046, pop −0,041, soul −0,033, reggae −0,072, afrobeat −0,059, disco et
hip-hop à ±0,003). La charleston de `gmd_pop` (0,253 → 0,212), de `gmd_soul` (0,397 → 0,304) et de `gmd_dance_disco` (0,206 → 0,189) s'améliore ;
aucune charleston ne perd plus d'un point. La copie « sans arc » du script retire maintenant aussi
`session.toml` (qui pourrait tirer un arc nommé).

### B4 + B7 : `make style-distance` (corpus GMD de B4 + B7, 10 graines × 48 mesures, scores 0 à 100, **plus grand = plus proche**)

| Mesure | `gmd_funk` avant → après | `gmd_neworleans_funk_secondline` avant → après |
|---|---|---|
| un coup quelconque par pas | 89,8 → 91,0 | 91,7 → 92,1 |
| grosse caisse | 55,5 → 60,5 | 59,5 → 57,8 |
| caisse claire | 77,3 → 77,2 | 74,4 → 74,6 |
| charleston | 74,3 → 77,6 | 64,5 → 66,4 |
| **groupe batterie** | **74,3 → 76,6** | **72,5 → 72,7** |

La grosse caisse de New Orleans perd 1,7 point (mesure d'un sous-groupe, pas le critère) ; le groupe batterie, mesure de P2, ne perd rien.
`loop_drums` (la batterie rejoue-t-elle la boucle de 4 mesures de la référence ?) monte de 34,9 à 52,2 et de 53,1 à 63,6 : non
interprétable ici (voir B4 + B7), cité pour mémoire.

## Contrats et suites

| Vérification | Résultat |
|---|---|
| `gmd_session_profile_tests.rs` (4 tests, neuf profils) : ≥ 8 habillages sur 16 graines et ≥ 2 grappes de tempo, ≥ 3 tonalités, ≥ 3 comping, ≥ 3 kits, ≥ 2 voicings, tempo dans la plage de la source, swing de la source ; même graine, même flux ; `[fixed_tonality]` et `SetBpm` de l'hôte gagnent, kit toujours tiré ; ≥ 7 suites de notes sur 8 graines sous tonalité et tempo de l'hôte | passent |
| `gmd_styles_profile_tests.rs` (B8) et `gmd_funk_neworleans_profile_tests.rs` | adaptés : tonique lue dans le snapshot (`position.key`), comping dans l'accord publié **et ses tensions naturelles** sans conflit (D4 ; convention de `bass_voicings_tests`), tessiture et registre de la session, un test d'instruments qui ne faisait que répéter la configuration supprimé, breaks de l'arc nommé (basse muette) hors de la lecture des arrivées. Tests de base sans `session` ni arc |
| Mutations (chacune fait échouer ≥ 1 test, puis annulée) | `session` retirée du profil `gmd_rock` (2 échecs), `[fixed_tonality]` remis (1 échec) |
| `cargo test --workspace -q --no-fail-fast` | 118 binaires réussis ; 1 échec : `profile_d2_main_note_reference` (étiquette `TUNING_VERSION:26`) → référence régénérée : **seule la ligne `source` change, les notes sont identiques** |
| `cargo test -p harmonium --features odin`, `-p harmonium_bevy`, `cargo check -p harmonium_studio` | `--features odin` : tous les binaires réussis (seek compris, mesuré hors charge) ; Bevy : 11 réussis ; studio : compile |
| `harmonium_determinism/scripts/run.sh` (wasm-bindgen 0.2.108) | **natif = wasm : 153 / 153** |
| Dumps `harmonium_determinism` contre `main` | les **98 fichiers des profils non touchés sont identiques octet pour octet** ; la référence en cache n'a ni les 54 fichiers des neuf profils (6 chacun, présents seulement côté branche) ni `swing_fixtures.txt` |
| Goldens `style_profile_tests`, `snapshot_stream_tests` | inchangés (profils non touchés) |
| `seek_to_bar_64_under_50ms` (profil par défaut, build de test, 25 passages, machine chargée par d'autres agents) | minimum **26,3 ms** (B8 : 26,4 ; référence 24–26 ms) ; code de génération inchangé |
| `cargo +nightly fmt --all --check` ; clippy `harmonium_core --all-targets` | propres pour les fichiers touchés |

## Manques de schéma relevés (tâche A7 à venir ; rien n'est contourné)

1. **Options de basse d'A4 inutilisables avec des riffs** : `walking.{eighths, passing.repeat_weight, arrival, two_feel}` sont
   exclusives de `motifs`, `templates` et `passing`. Ces neuf profils jouent des riffs (notes répétées, croches et notes fantômes
   y sont écrites à la main) : A4 n'a pas pu être activé « où c'est utile » sans remplacer les riffs par une marche, ce qui aurait
   changé la nature de la basse funk, rock et reggae. Il faudrait ces options **sur les motifs et les gabarits** (notes fantômes
   probabilistes, arrivée sur une autre note de l'accord, côté d'approche).
2. **Basse sans étiquette d'arc ni tirage par chorus** (déjà relevé) : 2–6 % de mesures de basse changent d'un chorus au suivant ; la
   session ne peut pas non plus tirer un plan de motifs (`tags` de comping et de batterie seulement).
3. **Groupes de session indépendants** : aucune dépendance entre kit, instruments et tempo (un kit « power » avec le tempo le plus
   lent, un clavinet avec un shell). Un groupe `conditions` ou des variantes jointes (« formations ») éviterait les mariages douteux
   sans multiplier les variantes ; ici les alternatives de chaque groupe ont été choisies pour que tous les mariages restent plausibles.
4. **Micro-timing en fraction de pas, pas en ms** : un retard mesuré en ms (le GMD) dérive avec le tempo tiré ; on le convertit au tempo
   par défaut (à 130 BPM le même retard en ms vaut 24 % de pas de plus ; à 83 BPM, 20 % de moins). Déjà noté par A6 (manque 5).
5. **`jitter` de batterie et basse à notes de passage** : le tirage partagé interdit tout `jitter` de batterie sous `bass.passing`
   (funk, New Orleans) ; un flux par piste le réglerait.
6. **Tonalité de la session : premier air seulement** (A2) ; sans importance ici (un air par set). **`[fixed_tonality]` ou variante** :
   pas de moyen d'imposer « la tonalité de la variante *et* le mode du profil » ; les neuf profils ont dû retirer le verrou.
7. **Accords de puissance** (« 5 ») toujours absents : le rock joue des triades doublées.
8. **Pas de swing selon le tempo** pour ces profils : le swing léger (hip-hop, rock) est un rapport unique ; aucune courbe JTD n'a de sens
   pour un feel que le GMD ne mesure pas au-delà de 7 et 10 fichiers.

## Limites

- Les groupes de métier (tonalités, instruments, kits, voicings, registre, énergie, arcs) ne sont pas mesurés : preuves chiffrées de
  variété seulement (D5), aucune écoute ; les contrats vérifient la cohérence avec la grille publiée, pas une ressemblance à un corpus.
- Le GMD est un kit électronique Roland TD-11 (dix batteurs) : le micro-timing et les fréquences par pas sont ceux de ces batteurs ; reggae
  et disco n'ont que deux batteurs, afrobeat deux.
- `gmd_reggae` a trois bandes de tempo (83–94, 97–108, 110–121) mais **deux fichiers** (90 et 114 BPM) : la bande du milieu interpole.
- Aucune zone Odin, aucune banque ni preset neuf : rien à publier côté `web-resources.json`.
- Les niveaux de mixage (LUFS) de B8 et de B4 + B7 sont calés pour le kit et les programmes **par défaut** ; les autres programmes tirés
  (organ, slap, kit power…) ne sont pas recalés (une même vélocité ne donne pas le même niveau sur un autre programme).

## Liens d'écoute (deux graines contrastées par profil ; à ouvrir par l'orchestrateur, en pause)

- `gmd_rock` : graine **7** `harmonium://open?profile=gmd_rock&seed=7` — 124 BPM, tonique 9, comping 29, basse 33, kit 16, voicing ouvert, arc montee, swing 0.5 ; graine **8** `harmonium://open?profile=gmd_rock&seed=8` — 92 BPM, tonique 4, comping 30, basse 34, kit 8, voicing triades_serrees, arc direct, swing 0.6.
- `gmd_pop` : graine **2** `harmonium://open?profile=gmd_pop&seed=2` — 88 BPM, tonique 7, comping 0, basse 38, kit 25, voicing triades_serrees, arc orchestre, swing 0.5 ; graine **4** `harmonium://open?profile=gmd_pop&seed=4` — 141 BPM, tonique 5, comping 4, basse 35, kit 0, voicing ouvert, arc defaut, swing 0.5.
- `gmd_hiphop` : graine **6** `harmonium://open?profile=gmd_hiphop&seed=6` — 98 BPM, tonique 4, comping 11, basse 33, kit 0, voicing drop_2, arc orchestre, swing 0.5 ; graine **8** `harmonium://open?profile=gmd_hiphop&seed=8` — 72 BPM, tonique 0, comping 0, basse 39, kit 25, voicing ouvert, arc direct, swing 0.59.
- `gmd_soul` : graine **5** `harmonium://open?profile=gmd_soul&seed=5` — 104 BPM, tonique 0, comping 16, basse 35, kit 0, voicing drop_2, arc defaut, swing 0.5 ; graine **9** `harmonium://open?profile=gmd_soul&seed=9` — 80 BPM, tonique 3, comping 4, basse 33, kit 0, voicing ouvert, arc orchestre, swing 0.5.
- `gmd_reggae` : graine **4** `harmonium://open?profile=gmd_reggae&seed=4` — 90 BPM, tonique 7, comping 0, basse 34, kit 0, voicing triades_serrees, arc defaut, swing 0.5 ; graine **7** `harmonium://open?profile=gmd_reggae&seed=7` — 117 BPM, tonique 9, comping 28, basse 33, kit 24, voicing ouvert, arc orchestre, swing 0.5.
- `gmd_dance_disco` : graine **1** `harmonium://open?profile=gmd_dance_disco&seed=1` — 131 BPM, tonique 9, comping 27, basse 33, kit 8, voicing ouvert, arc montee, swing 0.5 ; graine **3** `harmonium://open?profile=gmd_dance_disco&seed=3` — 123 BPM, tonique 7, comping 4, basse 36, kit 0, voicing triades_serrees, arc defaut, swing 0.5.
- `gmd_afrobeat` : graine **3** `harmonium://open?profile=gmd_afrobeat&seed=3` — 102 BPM, tonique 4, comping 4, basse 35, kit 0, voicing notes_guides, arc defaut, swing 0.5 ; graine **9** `harmonium://open?profile=gmd_afrobeat&seed=9` — 86 BPM, tonique 2, comping 27, basse 33, kit 0, voicing shell, arc orchestre, swing 0.5.
- `gmd_funk` : graine **5** `harmonium://open?profile=gmd_funk&seed=5` — 89 BPM, tonique 7, comping 28, basse 38, kit 0, voicing notes_guides, arc defaut, swing 0.5 ; graine **7** `harmonium://open?profile=gmd_funk&seed=7` — 126 BPM, tonique 9, comping 27, basse 33, kit 24, voicing drop_2, arc orchestre, swing 0.5.
- `gmd_neworleans_funk_secondline` : graine **6** `harmonium://open?profile=gmd_neworleans_funk_secondline&seed=6` — 84 BPM, tonique 10, comping 16, basse 58, kit 32, voicing drop_2, arc montee, swing 0.5 ; graine **7** `harmonium://open?profile=gmd_neworleans_funk_secondline&seed=7` — 118 BPM, tonique 5, comping 0, basse 58, kit 0, voicing shell, arc orchestre, swing 0.5.

Les liens chargent les profils embarqués ; pour voir le dossier du worktree, `harmonium://open?profile=<chemin du worktree>/profiles/<id>&seed=<n>`
(le comping lu est celui de la première mesure : un arc `orchestre` le change aux chorus suivants).

## Reproduire

```sh
cd ~/orca/workspaces/harmonium/r2-styles
LAB=~/Project/bascanada/harmonium_lab
$LAB/.venv/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py                    # sept profils B8
$LAB/.venv/bin/python specs/019-bibliotheque-accompagnements/r2-styles/appliquer_b47.py        # funk et New Orleans
cargo build --release -p harmonium-cli
python3 specs/019-bibliotheque-accompagnements/r2-styles/habillage_mesure.py target/release/harmonium-cli --out /tmp/h.json
python3 specs/019-bibliotheque-accompagnements/r2-styles/comparer_variete.py <avant> <après>   # dossiers de seed-variety --chorus 4
cargo test -p harmonium_core --test gmd_session_profile_tests
```

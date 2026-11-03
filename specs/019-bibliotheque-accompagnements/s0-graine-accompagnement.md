# S0 — L'accompagnement suit la graine de session

Date : 2026-11-02. Branche `wquintal/s0-graine-accomp`, partie de `main` à `7349278` (après la
fusion d'A3). **`TUNING_VERSION` 24 → 26** (25 est réservée à R1). Rupture assumée pour tout profil
distribué qui a un `[accompaniment]` (15 sur 16) ; un profil sans accompagnement garde ses notes à
l'octet (preuves plus bas).

## Cause (vérifiée)

Comping (et gratté), marche de basse, notes de passage de basse et vamp d'intro tiraient leur graine
de `ActiveTune::material_seed`, que `ornaments::tune_seed` dérive de l'indice de l'air et des **notes
de la mélodie écrite** (`build_tune`), sans aucun tirage de session. Le plan de motifs de basse
(`BassMotifPlan`) était amorcé par la même dérivation.

Mesure sur 16 graines de session (`TuneGenerator`, `tune_for_bar(1).material_seed`, tonalité fixée
par `[fixed_tonality]`) :

| Profil | `material_seed` distincts / 16 | Tonalités distinctes |
|---|---:|---:|
| `wjazzd_swing` (grille par défaut) | **2** | 1 |
| `grille_arc_chorus` (blues fixe) | **2** | 1 |

Sous une grille d'accords fixe, la banque de cellules minimale réduit la mélodie à deux versions : la
graine de matériel n'a que deux valeurs, et l'accompagnement (une graine par mesure =
`mix(material_seed, mesure)`) aussi. Constat initial (V0, orchestrateur) : 8 graines × 24 mesures →
piano 2 séquences, basse 4 ; ligne de base V0 sur 16 graines × 4 chorus : piano 2, basse 2 ; la
batterie, qui lit `variant_seed` (tirée), varie.

## Correction

**Une graine d'accompagnement par air**, nommée par son rôle : `ActiveTune::accompaniment_seed`.

- **Tirée dans le RNG injecté**, une fois par air (deux entiers de 32 bits, comme `arc_seed` et
  `performance_seed`), en **dernier** dans `build_tune`, après la mélodie, l'intro et le matériel : la
  mélodie, l'intro et les ornements du premier air ne bougent pas. Aucun profil sans
  `[accompaniment]` ne tire (flux inchangé).
- Consommateurs, tous mesure par mesure par `mix(accompaniment_seed, indice de mesure)`, **sans état
  d'une mesure à l'autre** : motif et gratté du comping (`generate_measure`, `chord_notes_for_bar`, qui
  rejoue `ChordPlayer` depuis le début de la passe pour la conduite de voix, comme avant), marche de
  basse (`WalkingBar::seed`), notes de passage (`PassingContext::seed`), vamp d'intro, et
  `BassMotifPlan` (amorce `mix(accompaniment_seed, 0x424D4F54)` au lieu du hachage de la mélodie).
- **Gardent `material_seed`** : ornements (`ornament_context`) et hétérophonie. Sa condition de calcul
  est inchangée (elle garde les clauses d'accompagnement d'avant) pour que le tirage d'ornements,
  qui lit `ctx.seed != 0` pour `repeat_variation`, ne change pas.

**Pourquoi un tirage.** Aucune valeur de session déjà disponible n'est présente dans tous les
profils : `variant_seed`, `performance_seed`, `arc_seed` et l'humanisation de percussion ne sont tirés
que si le profil en a l'usage. Dériver de l'une d'elles ferait dépendre la variété de l'accompagnement
d'un champ sans rapport (p. ex. `velocity_humanize > 0`) et le contrat « graines de session
différentes → accompagnements différents » serait faux pour un profil de style futur sans l'un d'eux.
Le tirage est donc nécessaire et explicite.

**Quels profils tirent.** Tous ceux qui ont `[accompaniment]`, que leurs couches tirent ou non (j'ai
d'abord conditionné le tirage aux couches qui en ont besoin ; le test existant
`passing_keeps_neighbour_tracks_and_strong_bass_unchanged` l'a refusé : retirer la couche de passage
supprimait le tirage et décalait le flux du reste de l'air suivant). Ajouter ou retirer une couche ne
décale donc aucun autre tirage.

**Coût du tirage.** Les airs **suivants** d'un même set (et tout nouveau set) reçoivent un RNG décalé
de deux tirages par air précédent : ils tirent d'autres cellules. Visible dans les dumps de 64
mesures seulement pour `isaac_downpour` (nouveau set dès la mesure 48 : accords, grosse caisse et
percussion aiguë changent à partir de là). La mélodie et les ornements du premier air, eux, sont
identiques (test de contrat 5, piste 0 et contre-mélodie, pour `reel_quebecois`).

## Profils distribués touchés

Comparaison des 98 dumps de `harmonium_determinism` (binaire de `main` à `7349278`, `git archive`,
cible séparée) et de la branche : notes par piste (piste, hauteur, début, durée, vélocité). Colonnes :
mesures changées sur les 6 dumps de chaque profil (3 graines × 2 jeux d'émotions, 64 mesures chacun),
première mesure (0-based) changée.

| Profil | Accords | Basse | Autres pistes | Cause |
|---|---|---|---|---|
| `wjazzd_swing` | 326 (m. 0) | 372 (m. 0) | — | comping, marche, passage |
| `reel_quebecois` | 338 (m. 0) | 224 (m. 3) | — | gratté et coups d'accords probabilistes ; passage |
| `reel_acadien` | 329 (m. 9) | 231 (m. 9) | — | idem (premiers dumps : intro) |
| `reel_britannique_coloniale` | 344 (m. 0) | 249 (m. 2) | — | idem |
| `gigue_quebecoise` | 354 (m. 0) | 276 (m. 2) | — | idem |
| `gigue_acadienne` | — | 259 (m. 4) | — | passage (accords sans tirage) |
| `gigue_britannique_coloniale` | 334 (m. 0) | 254 (m. 2) | — | idem |
| `hornpipe_britannique_coloniale` | 354 (m. 0) | 195 (m. 2) | — | idem |
| `valse_quebecoise` | 330 (m. 9) | 1 (m. 40) | — | idem |
| `valse_britannique_coloniale` | 334 (m. 0) | 2 (m. 17) | — | idem |
| `isaac_basement` | — | 176 (m. 0) | — | plan de motifs de basse |
| `isaac_burning` | — | 190 (m. 0) | — | idem |
| `isaac_cellar` | — | 234 (m. 0) | — | idem |
| `isaac_downpour` | 10 (m. 54) | 170 (m. 0) | grosse caisse 30, aiguë 16 (m. 48) | plan de basse ; nouveau set à la mesure 48 (RNG décalé) |
| `nt_desert` | — | 24 (m. 10) | — | plan de motifs de basse (tranches tirées ; 5 dumps sur 6, le sixième est identique) |
| `marche_britannique_coloniale` | — | — | — | **sans `[accompaniment]` : 6 dumps identiques à `main`** |

La piste 0 (mélodie) et la contre-mélodie ne changent dans aucun des 98 dumps.

**Invariance** (profil sans accompagnement) : `marche_britannique_coloniale`, seul profil distribué sans
`[accompaniment]` : 6 dumps identiques à `main` ; WAV `--export --seed {1,2} --bars 44 --style
marche_britannique_coloniale` (CLI de `main` et de la branche, lancés depuis
`~/Project/bascanada/harmonium`) : `cmp` identiques.

**Références régénérées dans le même commit** (principe I) : `harmonium_cli/tests/fixtures/profile_d2_main.json`
(`source` `TUNING_VERSION:26`), `profile_d2_fixed.json` (rôles fixes, accompagnement de passage retiré :
la dérivation de la graine ne change que le flux des airs suivants, source conservée à
`TUNING_VERSION:20:fixed_roles`), `harmonium_host/tests/golden_measures/snapshot_stream_seed42_8bars.json`.
Test supprimé : `profile_without_heterophony_keeps_main_v20_notes`, qui comparait le hachage FNV-1a
des notes de `main` V20 : la preuve d'identité d'un changement ancien (V20) n'est plus reproductible
par construction (tout profil avec accompagnement change) et la re-figer en un nouveau hachage
aurait épinglé l'implémentation. Les goldens ci-dessus restent la référence.

## Variété avant / après

`harmonium-lab seed-variety --chorus 4`, 16 graines × 4 chorus, **binaire compilé depuis le worktree**
(`main` à `7349278` avant, la branche après), lancé depuis `~/Project/bascanada/harmonium`
(`--engine-dir`, `assets/`). Profils : `wjazzd_swing` (grille par défaut : 44 mesures par chorus),
`wjazzd_swing_grille8` (copie sous **grille utilisateur** de 8 mesures, `scripts/v0_user_grid_profile.py`,
le cas de training), fixture `grille_arc_chorus` (avec arc, 12 mesures). Inter-graines : à chorus et
à place égale.

| Profil | Couche | Places qui varient | Distance graduée inter-graines | Séquences distinctes sur 16 graines |
|---|---|---|---|---|
| `wjazzd_swing` | comping : rythme | 83 % → 100 % | 37 % → 68 % | **2 → 16** |
| `wjazzd_swing` | comping : voicings | 73 % → 100 % | 24 % → 46 % | **2 → 16** |
| `wjazzd_swing` | basse | 93 % → 100 % | 31 % → 59 % | **2 → 16** |
| `wjazzd_swing` | accompagnement (toutes couches) | 100 % → 100 % | 22 % → 38 % | 14 → 16 |
| `wjazzd_swing` | batterie | 100 % → 100 % | 19 % → 19 % | 11 → 11 |
| `wjazzd_swing_grille8` | comping : rythme | 77 % → 100 % | 35 % → 68 % | **2 → 16** |
| `wjazzd_swing_grille8` | comping : voicings | 68 % → 100 % | 16 % → 36 % | **2 → 16** |
| `wjazzd_swing_grille8` | basse | 96 % → 100 % | 37 % → 67 % | **2 → 16** |
| `wjazzd_swing_grille8` | accompagnement (toutes couches) | 100 % → 100 % | 22 % → 39 % | 5 → 16 |
| `wjazzd_swing_grille8` | batterie | 100 % → 100 % | 21 % → 21 % | 3 → 3 |
| `grille_arc_chorus` | comping : rythme | 81 % → 100 % | 41 % → 59 % | 10 → 16 |
| `grille_arc_chorus` | comping : voicings | 77 % → 100 % | 19 % → 31 % | 10 → 16 |
| `grille_arc_chorus` | basse | 74 % → 96 % | 27 % → 58 % | **2 → 16** |
| `grille_arc_chorus` | accompagnement (toutes couches) | 87 % → 98 % | 18 % → 31 % | 15 → 16 |
| `grille_arc_chorus` | batterie | 50 % → 50 % | 8 % → 8 % | 7 → 7 |

Accords (symboles) : 1 séquence avant et après (grille fixe, non touchée). Variété **intra-graine**
(d'un chorus au suivant) : inchangée dans ses ordres de grandeur (p. ex. `wjazzd_swing` comping
rythme, mesures changées d'un chorus au suivant 78 % → 80 %). JSON : `/tmp/harmonium-019/s0/{avant,apres}-*.json`.

**Limite** : la batterie ne dépend pas de la graine d'accompagnement (V0 et orchestrateur) : 11, 3 et
7 séquences sur 16, et 100 % de chorus identiques sous `wjazzd_swing` (batterie fixe d'un chorus au
suivant). À traiter avec A1 ou l'arc de données ; S0 ne la touche pas.

## Seek

`seek_to_bar_64_under_50ms` (`deterministic_seek(64)`, graine 42, profil par défaut, build de test,
local, 3 passages de 25 seeks, minimum et médiane ; le test ne change pas, son seuil non plus). Deux
tours alternés `main` / branche :

| | `main` (`7349278`) | branche |
|---|---|---|
| tour 1, minimum (médiane) | 25,08 / 25,17 / 25,20 ms (25,6 ms) | 25,80 / 25,83 / 25,80 ms (25,8 à 26,5 ms) |
| tour 2, minimum (médiane) | 25,50 / 25,52 / 25,49 ms (25,7 ms) | 25,55 / 25,49 / 25,47 ms (25,7 ms) |

Rejeu de 65 mesures du générateur seul (`silent_advance(65)`, 30 passages, minimum) :

| profil | `main` | branche |
|---|---|---|
| `reel_quebecois` | 25,09 et 25,02 ms | 24,57 et 24,71 ms |
| `wjazzd_swing` | 21,31 et 21,06 ms | 18,41 et 18,43 ms |

Un tirage de deux entiers par air n'a pas de coût mesurable ; la durée de `wjazzd_swing` baisse de
2,7 ms parce que le plan de la marche change de contenu (la marche tire d'autres approches), pas par
une optimisation. Le seek est inchangé à ± 0,7 ms, bien sous le seuil et sous la médiane de 018.

## Tests

`harmonium_core/tests/accompaniment_seed_tests.rs`, 5 tests sur le flux de `MeasureSnapshot`
(principe VI), sur trois profils : `wjazzd_swing` (grille par défaut), `grille_arc_chorus` sans `[arc]`
(grille fixe : le cas de training) et `grille_arc_chorus` avec arc :

| Contrat | Test |
|---|---|
| 16 graines × 24 mesures : **≥ 12 séquences distinctes** de comping et de basse (avant S0 : 2) | `des_graines_de_session_differentes_donnent_des_comping_et_des_basses_differents` |
| à **mélodie égale** (graines qui jouent la même mélodie), l'accompagnement diffère presque toujours (au plus une répétition) | `a_melodie_egale_la_graine_de_session_change_l_accompagnement` |
| même graine → même sortie (graines 1, 2, 17) | `meme_graine_meme_sortie` |
| rejeu après seek (`silent_advance` aux mesures 13, 25, 31, sortie identique jusqu'à la mesure 36) | `une_mesure_se_rejoue_a_l_identique_apres_seek` |
| mélodie et contre-mélodie (ornements, hétérophonie) du premier air du `reel_quebecois` identiques que l'accompagnement tire ou non (la variante n'a plus de passage de basse, plus de probabilités de coups, plus de jitter de gratté ; l'accompagnement diffère bien) | `la_melodie_et_les_ornements_ne_dependent_pas_de_la_graine_d_accompagnement` |

Mutations vérifiées (chacune fait échouer au moins un test) : `accompaniment_seed := material_seed`
(échec des deux premiers tests), tirage placé avant la mélodie (échec du cinquième). Modifié :
`set_contract_tests.rs` (`TUNING_VERSION` 26). Supprimé : voir plus haut.

## Preuves

| Vérification | Résultat |
|---|---|
| `harmonium_determinism`, `main` contre branche | 15 profils sur 16 diffèrent (table plus haut) ; `marche_britannique_coloniale` 6 / 6 identiques ; piste 0 et contre-mélodie identiques dans les 98 |
| WAV `marche_britannique_coloniale`, graines 1 et 2, 44 mesures | `cmp` identiques |
| `cargo test --workspace -q --no-fail-fast` | 810 réussis, 0 échec, 2 ignorés (806 d'A3, moins 1 supprimé, plus 5) |
| `cargo test -p harmonium --features odin` ; `cargo test -p harmonium_bevy` | 118 réussis ; 11 réussis |
| `cargo check -p harmonium_studio` ; `cargo check --target wasm32-unknown-unknown` (`harmonium_core`, `harmonium_audio`, `harmonium`, avec et sans `odin` ; `harmonium_bevy --no-default-features`, avec et sans `odin`) | passent |
| `harmonium_determinism/scripts/run.sh` (wasm-bindgen 0.2.108) | natif = wasm : 98 / 98 |
| `cargo clippy -p harmonium_core -p harmonium -p harmonium-cli --all-targets` | par fichier touché (`generator.rs`, `accomp_profile.rs`, `tuning.rs`, `automation.rs`), mêmes avertissements que `main` (12 dans `generator.rs`, 3 dans `automation.rs`, 0 ailleurs) ; aucun dans `accompaniment_seed_tests.rs` |
| `cargo +nightly fmt --all --check` | propre |

## Écoute (propriétaire)

Rendus `wjazzd_swing`, 24 mesures (38,4 s, 150 BPM, 1,6 s par mesure), CLI de la branche et de `main`
lancés depuis `~/Project/bascanada/harmonium` (`assets/`) ; WAV dans `/tmp/harmonium-019/s0/ecoute/`,
MP3 320 k dans `~/Project/bascanada/harmonium_lab/corpus/ecoute_019/s0/`, page
`http://192.168.8.171:8080/#phase-019-s0` :

```sh
cd ~/Project/bascanada/harmonium
$CLI --export --seed 1 --bars 24 --style wjazzd_swing --record-wav /tmp/harmonium-019/s0/ecoute/wjazzd_swing_s1_apres.wav   # CLI de la branche ; idem graine 2
$CLI --export --seed 1 --bars 24 --style wjazzd_swing --record-wav /tmp/harmonium-019/s0/ecoute/wjazzd_swing_s1_main.wav    # CLI de main ; idem graine 4
```

Fichiers : `wjazzd_swing_s1.mp3`, `wjazzd_swing_s2.mp3` (après) ; `avant_wjazzd_swing_s1.mp3`,
`avant_wjazzd_swing_s4.mp3` (avant : piano et basse **identiques note pour note** entre ces deux
graines, seule la batterie diffère : c'est le défaut corrigé). Niveau intégré de chaque rendu :
−20,7 LUFS (inchangé : le mix du profil ne change pas).

| Moment | Mesures | À écouter (graines 1 et 2, après) |
|---|---|---|
| 0:06,4 à 0:09,6 | 5 et 6 (Si♭7) | graine 1 : le piano se tait, la basse monte (Si♭ do ré do) ; graine 2 : le piano joue (2 puis 3 attaques), la basse suit une autre ligne (Si♭ fa sol la) |
| 0:09,6 à 0:12,8 | 7 et 8 (Fa7) | graine 1 : piano à la mesure 7 (3 attaques), presque rien à la 8 ; graine 2 : l'inverse |
| 0:19,2 à 0:38,4 | 13 à 24 | début de l'AABA, deux accords par mesure : mêmes accords, autre comping et autre basse |

À juger : (1) l'accompagnement change-t-il d'une graine à l'autre sans devenir brouillon (les
rythmes de comping sont ceux du profil, tirés par poids ; rien n'a été recalibré) ; (2) la basse :
les graines donnent d'autres lignes de marche, mais le feel et le registre sont ceux du profil.

## Limites

- **Airs suivants d'un set** : le tirage décale le RNG d'un air à l'autre ; les airs 2 et suivants des
  profils à plusieurs airs par set (reels, gigues, valses, hornpipe) ne sont plus les mêmes qu'avant S0
  (dans les dumps de 64 mesures, seul `isaac_downpour` en change, à la mesure 48 ; les autres n'ont pas
  d'air suivant dans 64 mesures). Conséquence assumée de la rupture, couverte par les goldens.
- **Batterie** : variée par `variant_seed` seulement quand les patterns ont des alternatives ; hors
  périmètre S0 (11, 3 et 7 séquences sur 16 dans les mesures ci-dessus).
- **Mélodie et accompagnement découplés** : le comping ne dérive plus des notes de la mélodie écrite.
  Aucun contrat ne reposait sur ce lien (la graine n'était qu'un hachage des notes) ; un profil qui
  voudrait un comping lié à la mélodie le demanderait en données.
- **Une graine par air** : un set de plusieurs airs donne à chaque air son comping (`tune_index` ne
  suffit plus, la graine est tirée) ; l'arc, lui, garde la graine du premier air du set (A3).
- **R1 et goldens** : `profile_d2_main.json`, `profile_d2_fixed.json` et le golden de flux de
  `harmonium_host` sont à réconcilier à la fusion avec R1 (`TUNING_VERSION` 25 réservée, 26 ici).

## Fichiers

Moteur : `harmonium_core/src/compose/generator.rs` (`accompaniment_seed`), `tuning.rs`
(`TUNING_VERSION`). Tests : `harmonium_core/tests/accompaniment_seed_tests.rs` (nouveau),
`set_contract_tests.rs`, `harmonium_cli/src/automation.rs` (test V20 supprimé). Références :
`harmonium_cli/tests/fixtures/profile_d2_{main,fixed}.json`,
`harmonium_host/tests/golden_measures/snapshot_stream_seed42_8bars.json`. Docs :
`profiles/README.md` (« Graine d'accompagnement »), `harmonium_core/README.md`, ce compte rendu,
`plan.md`. Labo (non suivi) : `corpus/index.html`, `corpus/ecoute_019/s0/`.

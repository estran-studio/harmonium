# A1 — Phrases sur plusieurs mesures

Date : 2026-11-02. Branche `wquintal/a1-phrases`, partie de `main` à `4d7bf50` (A3, R1 et S0
fusionnés, `TUNING_VERSION` 26 **inchangé**). Mécanisme optionnel en données : un profil sans
`phrases` ni `fills` garde ses dumps, ses WAV et ses goldens (preuves plus bas). Aucun profil distribué
n'est touché (calibration : tâches B).

## Ce qui est livré

- **Phrases de comping** : `accompaniment.chords.phrases`, suites de motifs de `comping` (1 à 8 mesures).
- **Phrases de batterie** : `[[phrases]]` de `percussion.toml`, suites de patterns de même densité.
- **Relances (fills)** : `[[fills]]` de `percussion.toml`, banque pondérée de 1 à 4 mesures, avec
  position de départ dans la première mesure (`start_step`, p. ex. 12 : levée au temps 4). Elle
  **remplace** `[[cadence_patterns]]` à la densité qu'elle déclare ; l'ancien reste valide seul.
- **Place dans la forme** (`at`) : `any`, `part_start`, `part_end` (mise en place avant un changement de
  section), `chorus_end`.
- **Étiquettes** (`tag`) sur phrases et relances : l'arc de chorus (A3) les restreint comme les motifs.
- **Tirage sans état** : la phrase d'une mesure se recalcule depuis sa position et ses graines.
- Fixtures sans Odin `grille_phrases` (blues, 12 mesures) et `grille_phrases_aaba` ; 16 tests de contrat.

Basse : **rien ajouté**. Les motifs de basse de plusieurs mesures existent déjà
(`accompaniment.bass.motifs[].bars`, `BassMotifPlan`, `profiles/README.md`) ; une phrase de basse
générique relève d'A4.

## Schéma

Détail et exemples : `profiles/README.md`, « Phrases d'accompagnement sur plusieurs mesures ».

```toml
[[accompaniment.chords.phrases]]       # comping
weight = 3; tag = "medium"; at = "part_end"; part = "B"     # tous optionnels
motifs = [0, 3, 0, 3]                  # 1..=8 indices dans accompaniment.chords.comping

# percussion.toml
fills_none_weight = 3                  # poids de « pas de relance »
[[phrases]]
density = 1; weight = 3; tag = "ride"; at = "any"; part = "B"
patterns = [3, 4, 3, 5]                # 1..=8 indices de [[patterns]], même densité, sans portée
[[fills]]
density = 2; weight = 3; tag = "ride"; at = "part_end"
start_step = 12                        # pas de la 1re mesure où la relance prend la main
bars = [{ hits = [{ step = 12, voice = "snare" }] }]   # 1..=4 mesures
```

Bornes : 256 phrases et 256 relances par banque ; poids ≥ 1. Erreurs avec chemin du champ
(`accompaniment.chords.phrases[0].motifs[1]`, `percussion.phrases[0].patterns[1]`,
`percussion.fills[0].bars[0].hits[0].step`…). Champ inconnu refusé. Un profil sans ces champs ne les
sérialise pas.

## Algorithme (`compose/phrase.rs`, générique : aucun style, aucune valeur musicale)

1. **Passe** : une entrée de `form.sequence` est une passe de partie de `bars` mesures. Graine de passe =
   `mix(mix(accompaniment_seed, couche), indice de la première mesure de la passe)` (couche : comping ou
   batterie, flux distincts). Le découpage **ne franchit jamais la frontière d'une partie**, donc ni
   celle d'un chorus.
2. **Fin de passe** (mise en place) : la classe la plus précise qui a un candidat (`chorus_end` si la
   passe est celle de la dernière partie du chorus, sinon `part_end`) tire une phrase posée en **fin**
   de passe. Elle joue toujours quand un candidat concourt.
3. **Reste** : tirages successifs depuis le début parmi les phrases `any` (et `part_start` à la mesure 0)
   qui tiennent dans ce qui reste ; ce qui ne tient plus se joue comme sans phrase.
4. **Tags de l'arc** : poids de la phrase × poids du tag, phrases sans tag nommé exclues ; aucune phrase
   ne concourt → tirage mesure par mesure (lui-même restreint par l'arc). Relances : mêmes tags ;
   aucune relance concernée → pas de relance.
5. **Relance** : tirée à chaque fin de phrase (fin de la phrase de batterie qui couvre la mesure ; hors
   phrases de la table, fin de phrase de la forme `phrase_ends` / rôles cadentiels). Une relance de `n`
   mesures couvre les `n` dernières de la phrase, jamais plus ; première mesure : pattern jusqu'à
   `start_step`, relance ensuite ; suivantes : relance seule. Parmi la classe de place la plus précise
   qui a un candidat, avec « pas de relance » (`fills_none_weight`).
6. **Sans état** : le découpage ne dépend que de la graine de passe et de la position ; il est calculé
   **une fois par mesure générée** (pas une fois par mesure rejouée de la conduite de voix de
   `chord_notes_for_bar`). Génération côté compositeur, aucune allocation dans le callback.
7. **Graine** : `accompaniment_seed` est aussi tirée quand le profil n'a que des phrases ou relances de
   batterie, sans `[accompaniment]` (le flux des profils sans phrases ne change pas).

Choix documentés (pas de question au propriétaire) :
- « Suite pondérée de motifs » : le poids porte sur le choix de la **phrase** ; une phrase est une suite
  fixe d'indices de motifs existants (pas de tirage par case).
- Longueurs 1 à 8 (le brief cite 2, 4, 8 ; 1 sert une mise en place d'une mesure, 3 ou 6 une forme impaire).
- Une relance de la forme (sans phrase de batterie à la densité) s'ancre sur `phrase_ends` de la forme ;
  avec phrases, sur leurs fins.
- `fills` et `cadence_patterns` à la même densité : refusé (chemin `percussion.fills[i].density`).

## Fixtures

`tests/fixtures/profiles/grille_phrases/` (copie de `grille_arc_chorus`, `arc.toml` inchangé) : 12
phrases de comping (trois étiquettes `sparse` / `medium` / `dense`, phrases de 2 et 4 mesures, une
mise en place `part_end` par étiquette), 12 phrases de batterie `ride` (densités 0 à 2), 9 relances
(levée au pas 12, relance de deux mesures `part_end`, relances `hat`). `grille_phrases_aaba/` ajoute
`part_start` et `chorus_end` (comping et batterie). `grille_arc_chorus*` ne change pas (tests A3 et S0
intacts). Valeurs d'exercice, aucune mesure de corpus.

**Phrases du GMD (point 6 de L2 § 6) — mesure à ajouter au labo.** Les `beat` du GMD durent 30 à 40
mesures : pour chaque style, découper chaque fichier en fenêtres de 2, 4 et 8 mesures alignées sur la
première mesure et compter les **suites de groupes de mesures** (les groupes de même figure de
ride de `measures-drums.json`) : poids d'une phrase = effectif de la suite ; ne garder que les suites
d'au moins 3 occurrences sur 2 batteurs. Pour les relances : à partir de `fill_position` (L2 § 6, point 4),
publier pour chaque `fill` la longueur (1 à 4 mesures), le pas de départ dans sa première mesure et les
probabilités par pas et par voix de chaque mesure, au format de `[[fills]].bars[].hits` (seuil de
probabilité 0,5 comme `engine_values`), plus la part de fins de phrase **sans** relance
(`fills_none_weight`). Je ne l'ai pas faite.

## Tests

`harmonium_core/tests/phrases_tests.rs`, 16 tests sur le flux de `MeasureSnapshot` (graines 1, 2, 3, 17 et
1 à 16). Les six motifs de comping se reconnaissent à leurs pas d'attaque : une phrase se lit comme une
suite d'indices.

| Contrat | Test |
|---|---|
| passes du blues et de l'AABA découpées en phrases de la banque, mise en place `part_end`, fin de chorus `chorus_end` | `les_phrases_de_comping_s_alignent_sur_les_frontieres_…` |
| sans phrase, les passes ne se découpent plus en phrases (témoin) | `les_phrases_remplacent_le_tirage_d_un_motif_par_mesure` |
| restriction par tag de l'arc : seules les phrases nommées jouent, aucune autre étiquette ne tuile | `l_arc_restreint_les_phrases_de_comping_par_tag` |
| phrases de batterie : suite de patterns, alignement, mise en place ; autre densité sans effet | `les_phrases_de_batterie_…` (2 tests) |
| relance d'une mesure au pas donné (pattern avant, relance après), à chaque fin de phrase | `la_relance_d_une_mesure_part_au_pas_donne_…` |
| relance de trois mesures qui s'achève avec la phrase ; jamais plus longue que la phrase | `la_relance_de_plusieurs_mesures_…`, `une_relance_ne_deborde_pas_…` |
| `part_end` seulement en fin de partie ; `chorus_end` l'emporte | `la_relance_de_fin_de_partie_…`, `la_relance_de_fin_de_chorus_…` |
| restriction des relances et phrases de batterie par tag | `l_arc_restreint_les_relances_et_les_phrases_de_batterie_par_tag` |
| rejeu après seek identique (mesures 10 à 55, milieu de phrase et de passe, JSON complet) | `une_phrase_se_rejoue_a_l_identique_apres_un_seek` |
| même graine → même sortie ; 16 graines → ≥ 14 suites distinctes de comping et de batterie ; chorus d'une graine différents | `meme_graine_meme_sortie_graines_differentes_…` |
| portée `part` | `une_phrase_de_partie_ne_joue_que_dans_cette_partie` |
| 20 erreurs de schéma avec chemin ; étiquette de phrase seule connue de l'arc | `les_erreurs_de_schema_nomment_le_champ_…` |
| aucun champ sérialisé sans phrases ; aller-retour TOML | `un_profil_sans_phrases_ne_les_serialise_pas` |

Mutations vérifiées (chacune fait échouer au moins un test) : fin de passe désactivée (4 échecs), tags
ignorés (2), relance qui déborde de sa phrase (1), `start_step` ignoré (2), graine de passe dépendante de
la mesure (3).

## Variance avant / après

`harmonium-lab seed-variety --chorus`, 16 graines, binaire de la branche lancé depuis
`~/Project/bascanada/harmonium` (`assets/`). **Avant** = fixture de l'arc `grille_arc_chorus*`
(A3 + S0, tirage mesure par mesure), **après** = `grille_phrases*` (même arc, mêmes motifs et patterns,
plus les phrases et relances). JSON et tableaux : `/tmp/harmonium-019/a1/var/`.

Blues (4 chorus) :

| Couche | Intra : mesures changées (stable) | Intra : paires de chorus identiques | Chorus distincts / graine | Inter : distance graduée | Inter : séquences distinctes |
|---|---|---|---|---|---|
| comping rythme | 90 % → 87 % (100 → 100) | 0 → 0 % | 4,0 → 3,9 | 59 → 52 % | 16 → 16 |
| comping voicings | 83 → 85 % | 0 → 0 % | 3,9 → 3,9 | 31 → 28 % | 16 → 16 |
| batterie | 85 → 81 % (100 → 100) | **24 → 17 %** | **2,6 → 3,0** | 8 → 11 % | **7 → 16** |
| accompagnement | 100 → 100 % | 0 → 0 % | 4,0 → 4,0 | 31 → 30 % | 16 → 16 |

AABA (3 chorus) :

| Couche | Intra : mesures changées | Intra : paires identiques | Chorus distincts / graine | Inter : distance graduée | Inter : séquences distinctes |
|---|---|---|---|---|---|
| comping rythme | 95 → 96 % | 0 → 0 % | 3,0 → 3,0 | 58 → 47 % | 16 → 16 |
| comping voicings | 95 → 96 % | 0 → 0 % | 3,0 → 3,0 | 52 → 42 % | 16 → 16 |
| batterie | 100 → 100 % | **33 → 2 %** | **2,0 → 2,9** | 6 → 8 % | **5 → 16** |
| accompagnement | 100 → 100 % | 0 → 0 % | 3,0 → 3,0 | 28 → 24 % | 16 → 16 |

Lecture contre V0 (`references/v0-variance/README.md`, seuils V2 proposés) :

- **Seuil 8** (≥ 12 / 16 séquences distinctes, comping et batterie) : batterie 7 → 16 (blues) et 5 → 16
  (AABA) ; comping 16 → 16.
- **Seuil 2** (batterie, paires de chorus identiques ≤ 5 %) : AABA 2 % ; blues 17 % : le corps de l'arc
  a deux chorus sur le même pattern de batterie, avec deux variantes de phrases : à régler en données (B*).
- **Seuil 4** (comping rythme changé, stable ≥ 75 %) : 87 % et 96 % ; le tirage par phrase garde un
  grand nombre de phrases par position (douze, pondérées).
- **Seuil 7** (batterie changée, stable ≥ 30 %) : 81 % et 100 % (déjà acquis par la batterie de l'arc et les
  relances).
- **Coût** : la distance inter-graines du comping baisse de 6 à 11 points (suites de motifs répétés dans une
  phrase, donc moins d'entropie par mesure) ; la structure gagne ce que la distance perd. À calibrer avec
  des phrases mesurées dans le GMD.

## Seek

Rejeu de 65 mesures du générateur seul (`silent_advance(65)`, graine 42, build de test, 30 passages,
minimum, deux tours alternés `main` / branche ; mesure temporaire non commitée) :

| profil | `main` | branche |
|---|---|---|
| `reel_quebecois` | 25,18 · 24,93 ms | 24,82 · 24,91 ms |
| `wjazzd_swing` | 18,57 · 18,73 ms | 18,63 · 18,51 ms |
| `grille_arc_chorus` | 21,28 · 21,15 ms | 21,15 · 20,90 ms |
| `grille_arc_chorus_aaba` | 18,03 · 18,10 ms | 18,25 · 18,03 ms |
| `grille_phrases` (phrases + relances) | — | 23,67 · 23,73 ms |
| `grille_phrases_aaba` | — | 18,95 · 18,94 ms |

Profils sans phrases : inchangé (± 0,3 ms). Avec phrases : + 2,5 ms (+ 12 %) pour le blues, + 0,9 ms pour
l'AABA, soit ≈ 38 µs par mesure (trois découpages par mesure générée : comping dans `generate_measure` et
dans `chord_notes_for_bar`, batterie). `seek_to_bar_64_under_50ms` (profil par défaut, sans phrases) est du
niveau de `reel_quebecois` (≈ 25 ms) : inchangé. Je n'ai pas optimisé (un seul découpage de comping par
mesure serait possible en le passant de `generate_measure` à `fill_tour_measure`) ; à reprendre si le
propriétaire veut ces 2,5 ms.

## Preuves

| Vérification | Résultat |
|---|---|
| `harmonium_determinism`, binaire de `main` (`git archive` de `4d7bf50`, cible séparée) contre la branche | 98 fichiers identiques octet pour octet (aucun profil distribué n'a de phrases) |
| WAV `--export --bars 24 --seed {1,3}` des 16 profils distribués (CLI lancé depuis `~/Project/bascanada/harmonium`), binaires `main` et branche | 32 / 32 `cmp` identiques |
| `TUNING_VERSION`, goldens, `profile_d2_main.json` | inchangés (aucune rupture) |
| `cargo test --workspace -q --no-fail-fast` | 826 réussis, 0 échec, 2 ignorés (810 de S0 + 16 de `phrases_tests`) |
| `cargo test -p harmonium --features odin` ; `cargo test -p harmonium_bevy` | 118 réussis ; 11 réussis |
| `cargo check -p harmonium_studio` ; `cargo check --target wasm32-unknown-unknown` (`harmonium_core`, `harmonium_audio`, `harmonium`, avec et sans `odin` ; `harmonium_bevy --no-default-features`, avec et sans `odin`) | passent |
| `harmonium_determinism/scripts/run.sh` (wasm-bindgen 0.2.108) | natif = wasm : 98 / 98 |
| `cargo clippy -p harmonium_core -p harmonium -p harmonium-cli --all-targets` | aucun avertissement dans `phrase.rs` ni `phrases_tests.rs` ; ceux de `percussion.rs` et `generator.rs` sont tous dans leurs modules de test existants (`unwrap`, `Default::default()`), inchangés |
| `cargo +nightly fmt --all --check` | propre |

## Écoute (propriétaire)

Rendus de la branche (`--export --record-wav`, CLI lancé depuis `~/Project/bascanada/harmonium` pour
`assets/`), WAV dans `/tmp/harmonium-019/a1/ecoute/`, MP3 320 k dans
`~/Project/bascanada/harmonium_lab/corpus/ecoute_019/a1/`, page `http://192.168.8.171:8080/#phase-019-a1` :

```sh
cd ~/Project/bascanada/harmonium && CLI=~/orca/workspaces/harmonium/a1-phrases/target/release/harmonium-cli \
  FX=~/orca/workspaces/harmonium/a1-phrases/tests/fixtures/profiles
$CLI --export --seed 1 --bars 48 --profile $FX/grille_phrases --record-wav /tmp/harmonium-019/a1/ecoute/blues_s1.wav
$CLI --export --seed 4 --bars 48 --profile $FX/grille_phrases --record-wav /tmp/harmonium-019/a1/ecoute/blues_s4.wav
$CLI --export --seed 1 --bars 48 --percussion 2 --profile $FX/grille_phrases --record-wav .../blues_s1_dense.wav
$CLI --export --seed 1 --bars 96 --profile $FX/grille_phrases_aaba --record-wav .../aaba_s1.wav   # idem graine 4 ; --percussion 2
$CLI --export --seed 1 --bars 48 --solo-track Chords --profile $FX/grille_phrases --record-wav .../blues_s1_Chords.wav   # Bass, PercussionHigh
```

Une mesure = 1,6 s (150 BPM). Moments (relevés dans la vérité terrain `--record-truth`) :

| Rendu | Moment | À écouter |
|---|---|---|
| blues graine 1 | 0:00–0:19,2 | chorus 1 : phrase médiane de 4 mesures, phrase clairsemée (0:06,4), 2 mesures (0:12,8), mise en place 0:16–0:19,2 (silence puis accord tenu) |
| blues graine 1 | 0:19,2–0:38,4 | chorus 2 : phrases denses ; relances de batterie **0:27,2** et **0:33,6** ; mise en place 0:35,2 |
| blues graine 1 | 0:38,4–0:57,6 | chorus 3 : relances **0:43,2**, **0:46,4**, **0:52,8** ; le comping passe de dense à médian à 0:44,8 |
| blues graine 1 | 0:57,6–1:15,2 | chorus 4 (retombée) clairsemé ; stop-time 1:12,0 |
| blues graine 4 | 0:17,6 · 0:36,8 · 0:49,6 · 0:52,8 | relance en levée ; break d'une mesure ; relances |
| AABA graine 1 | 0:09,6 · 0:22,4 · 0:35,2 · 0:44,8–0:51,2 | mises en place de fin de partie ; phrase de 4 mesures de fin de chorus |
| AABA graine 1 | 0:24,0 · 1:12,0 · 1:31,2 | relances ; 1:42,4 chorus 3 (retombée), stop-time 2:30,4 |
| AABA graine 4 | 0:36,8 · 0:56,0 · 0:59,2 · 1:05,6 · 1:21,6 · 1:24,8 · 1:26,4 · 1:28,0 | relances |

Mixage (LUFS intégrés, mesures 1 à 47 sans la clôture) : mix blues −22,2 (graine 1), −21,7 (graine 4),
−22,1 (graine 1, densité 2) ; AABA −21,9 / −21,6 / −21,8 ; blues graine 1 en solo : accords −25,2, basse
−24,7, PercussionHigh −31,7, PercussionLow silence (la grosse caisse ne joue qu'à la densité 2).

**À juger** : (1) les phrases de comping sonnent-elles comme des phrases qui se ferment (mise en place
en fin de partie) ou comme des mesures tirées ; (2) la relance en levée au temps 4 et la relance de deux
mesures ; (3) la fin de chorus de quatre mesures (AABA) ; (4) l'abaissement de la variété mesure par mesure
au profit de la structure.

## Limites

- **Valeurs d'exercice** : phrases et relances de la fixture écrites à la main ; aucune mesure du GMD
  (voir plus haut). `wjazzd_swing` n'a pas de phrases (B1).
- **Une phrase est une suite fixe** de motifs ; pas de choix pondéré par case.
- **Fin de passe jamais optionnelle** quand un candidat concourt (une phrase de fin neutre l'adoucit).
- **Phrases de batterie** : une par densité ; la densité qui change au milieu d'une passe (jeu) donne un
  découpage par densité (cohérent, mais deux découpages se croisent).
- **Relances d'une phrase de la forme** : mesurées à la fin de phrase précédente de la forme.
- **Basse** : phrases de basse non ajoutées (motifs `bars` existants ; A4).
- **Coût** : voir Seek.
- **Harnais natif/wasm** : ne couvre que les profils distribués ; le chemin des phrases n'a pas de code
  spécifique à wasm (flux ChaCha8, entiers `u64`).

## Fichiers

Moteur : `harmonium_core/src/compose/phrase.rs` (nouveau), `percussion.rs`, `accomp_profile.rs`,
`generator.rs`, `arc.rs`, `mod.rs`, `style_profile.rs`. Tests : `harmonium_core/tests/phrases_tests.rs`.
Données : `tests/fixtures/profiles/grille_phrases/`, `grille_phrases_aaba/`. Docs : `profiles/README.md`,
ce compte rendu, `plan.md`. Labo (non suivi) : `corpus/index.html`, `corpus/ecoute_019/a1/`.

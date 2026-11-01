# E5 + E6 — Accompagnement jazz en données et marche de basse

Date : 2026-11-01. Branche : `wquintal/e5-e6-accompagnement-jazz`, partie de `main` à `150b04f`
(N1, E1, E3 fusionnées, `TUNING_VERSION` 23). **Les notes des profils distribués ne changent
pas, aucun golden n'est régénéré** (preuves plus bas). Les notes de `rewrite_functional`
(fixture `jazz_test`) changent pour un même seed : rupture de contrat (principe I),
l'orchestrateur a porté `TUNING_VERSION` de 23 à 24 et le tampon de la référence D2 du CLI.
Aucun nom de
genre dans le code ni le schéma : tout est nommé par le rôle (`rootless`, `guide_tones`, `drop2`,
`walking`, `approach`, `comping`, `anticipate`).

## Ce qui est ajouté

### Données (toutes optionnelles : un profil qui n'active rien garde ses notes)

| Champ | Contenu |
|---|---|
| `accompaniment.chords.voicing` | trois nouvelles valeurs de la liste existante (`close_triad`, `fixed_triad`) : `guide_tones`, `rootless` (A et B), `rootless_a`, `rootless_b`, `drop2` |
| `accompaniment.chords.comping` | banque de rythmes d'**une mesure** tirés par poids (`weight`, `hits` : vide = silence), remplace le choix du motif d'intensité |
| `accompaniment.chords.patterns[].hits[].anticipate` et `comping[].hits[].anticipate` | attaque anticipée : joue l'accord en vigueur à la fin de sa durée |
| `accompaniment.bass.walking` | marche de basse : `velocity`, `accent`, `duration`, `max_leap`, `toward_target_weight`, `keep_direction_weight`, `approach = {none_weight, chromatic_weight, diatonic_weight}`, `passing = {chord_tone_weight, scale_step_weight}` |

`bass.walk_low` / `walk_high` (existants) deviennent la fenêtre de la marche. `walking` est
exclusive de `hits`, `motifs` et `passing`. Exemples et sémantique : `profiles/README.md`
(« Marche de basse », « Voicings et comping »).

### E5 — voicings en données, enchaînés par `theory::voice_leading`

Nouveau module `compose/voicing.rs`. Chaque voicing est une forme (intervalles croissants
au-dessus de la fondamentale) placée dans toutes les octaves MIDI ; `ChordPlayer::voice` choisit
parmi les candidats, dans l'ordre : **sortie de la fenêtre** `low..high` (la moins grande),
**dérive** de la hauteur moyenne hors de `center ± max_drift`, **mouvement** depuis le voicing
précédent, distance à `center`, hauteur la plus grave. Le mouvement est la première utilisation
de `theory::voice_leading` : `voicing_movement` (somme des écarts, voix appariées par rang depuis
le grave). La même fonction sert les voicings serrés existants, avec le même résultat
(vérifié : dumps et rendus identiques).

| Voicing | Construction | Décisions |
|---|---|---|
| `guide_tones` | tierce + septième, plus une tension de la qualité | tierce = 3 ou 4 demi-tons (sinon 2 ou 5 pour une suspension) ; « septième » = la plus haute note du type (sixte, quinte d'une triade). Tension : 9, 13, #11, b9, #9, b13, 11 par priorité, parmi celles de la qualité seulement. Sans tension : deux sons |
| `rootless` | quatre sons sans fondamentale : type A (tierce, quinte, septième, tension) ou B (septième, tension, tierce, quinte) | sans tension dans la qualité, la pile se prolonge à l'octave (tierce, quinte, septième, tierce) ; une triade n'a que le type A |
| `drop2` | position serrée à quatre voix, deuxième voix depuis l'aigu une octave plus bas | une triade double sa fondamentale |

Aucun voicing n'ajoute une note absente de l'accord publié (`pitch_classes` d'E3) : c'est
pourquoi `7` et `m7` sans tension ont une tierce doublée (3-5-7-3) plutôt qu'une neuvième
inventée. La basse d'un accord slash n'est pas jouée par ces voicings (la piste de basse la
joue). Une pédale garde ses voicings serrés.

**Comping.** `ChordVoice::bar_hits` tire le rythme de la mesure par poids avec la graine de
matériel et le numéro de mesure (`mix`), donc une mesure se rejoue à l'identique ; le replay de
la conduite de voix (`advance_bar`) relit la même mesure. `anticipate` : l'attaque joue
`chords[(step + duration) / steps_per_chord]`, ou le premier accord de la mesure suivante si la fin
tombe sur la barre ; en fin de parcours, l'accord du pas. `patterns` reste requis (intro, et
profils sans comping).

### E6 — marche de basse au pas près

Nouveau module `compose/walking.rs`, une mesure par appel, sans état d'une mesure à l'autre.

- **Une note par pulsation** (`meter.steps_per_pulse` : quatre noires en 4/4). Le chargeur exige
  `harmony.steps_per_chord` multiple de `steps_per_pulse` : un changement tombe sur une pulsation.
- **Arrivée** : premier temps de la mesure et chaque pulsation où l'accord change → la **basse**
  de l'accord (`bass_pc` : basse d'un accord slash, sinon fondamentale), placée dans `low..high`.
- **Approche** de la pulsation qui précède une arrivée (si elle n'en est pas une) : tirage pondéré
  `none` / `chromatic` (± 1 demi-ton) / `diatonic` (première note de la gamme du mode à 1, 2 ou 3
  demi-tons), du côté de l'arrivée précédente de la ligne. La cible de la dernière pulsation est
  l'arrivée de la mesure suivante.
- **Passage** : tirage `chord_tone` / `scale_step` parmi les notes à `max_leap` de la précédente
  qui laissent la cible atteignable ; poids ajoutés pour les notes qui se rapprochent de la cible
  et qui poursuivent le sens ; pas de retour sur l'avant-dernière note tant qu'un autre choix
  existe. Tout reste dans `walk_low..walk_high`.

### E6 — `rewrite_functional` sans second chemin (principe IV)

`fill_rewrite_measure` calculait lui-même les notes de basse (`bass.hits` sur la fondamentale)
et d'accords (`nearest_pitch` autour de `center`). Il ne publie plus que le contexte (accords
absolus, gammes) : la basse et les accords passent par `bass_bar` / `bass_motif_bar` /
`walking_bar` et `ChordPlayer`, comme `loop` et `melody_functional`.

- Conversion unique `DegreeChord::from_chord(chord, key)` : fondamentale en demi-tons chromatiques
  (`ChordDegree::Semitones`), qualité typée, basse. `chords_for_bar` renvoie ces accords pour le
  modèle ; le contexte publié reste celui d'avant (noms `Chord::name`, gammes inchangées).
- Les refus propres à ce modèle disparaissent : motifs de basse, `passing`, doublures, `spread`,
  `note_count`, voicings, probabilités. `bass.high - bass.low ≥ 11` ne reste exigé que pour
  `walking`.
- **Aucun profil distribué n'utilise `rewrite_functional`** (seul `jazz_test`, fixture) : pas de
  bump de `TUNING_VERSION`. Les notes de `jazz_test` changent (les accords sont des voicings serrés
  conduits et non plus des notes individuelles autour de `center`, le motif d'accords suit
  l'intensité, la durée d'un coup de basse n'est plus coupée au créneau) ; ses tests de contrat
  (`rewrite_model_tests`, `rewrite_profile_tests`, `chord_identity_tests`) passent sans
  modification.

## Fixture et écoute

`tests/fixtures/profiles/grille_accompagnee/` : un blues de douze mesures en fa (ii-V dans la
mesure 9), un ii-V-I de quatre mesures (mesure 15 : Gm7 et C13 à la noire puis Fmaj9 à la blanche ;
mesure 16 : D7, G7, C7alt) ; comping `rootless` à quatre rythmes (deux avec attaque anticipée, un silence) et
marche. Pas de profil distribué : `wjazzd_swing` est E7.

Rendus (CLI de la branche, lancé depuis `~/Project/bascanada/harmonium`, graine 1, 32 mesures,
swing 0,6667, 132 BPM, 58 s) :

```sh
cd ~/Project/bascanada/harmonium && \
  ~/orca/workspaces/harmonium/e5-e6-accompagnement-jazz/target/release/harmonium-cli --export \
  --seed 1 --bars 32 --swing 0.6667 \
  --profile ~/orca/workspaces/harmonium/e5-e6-accompagnement-jazz/tests/fixtures/profiles/grille_accompagnee \
  --record-wav /tmp/grille_accompagnee.wav
```

Déjà rendus dans `/tmp/e5e6/ecoute/` : `grille_accompagnee_complet.wav`, `…_chords.wav`
(`--solo-track Chords`), `…_bass.wav` (`--solo-track Bass`), et les `.mid`. Une mesure dure
1,82 s. À écouter :

| Moment | Mesure | Ce qu'on doit entendre |
|---|---|---|
| 0:00 à 0:07 | 1 à 4 | F7 sur quatre mesures : marche qui revient sur fa au premier temps, approche chromatique au 4e temps ; comping qui change d'attaque d'une mesure à l'autre |
| 0:14,5 à 0:18 | 9 et 10 | ii-V dans une mesure : Gm7 puis C7 au 3e temps, basse sur sol puis do ; comping à voix serrées |
| 0:20 à 0:22 | 12 | F7 puis C7 à mi-mesure |
| 0:21,8 à 0:25 | 13 et 14 | Gm9 C7b9 puis Fmaj7 : la neuvième et la b9 dans le comping |
| 0:25,5 à 0:27,3 | 15 | Gm7, C13 à la noire puis Fmaj9 : trois arrivées de basse aux pulsations 1, 2 et 3 |
| 0:27,3 à 0:29,1 | 16 | D7, G7, C7alt : basse qui enchaîne les trois |
| 0:29,1 et suivants | 17 à 32 | deuxième tour : mêmes accords, autres rythmes et lignes de basse (graine de mesure) |
| silences de comping | p. ex. mesures 6, 14 | une mesure sans accord, la basse continue |

À juger : naturel de la ligne de basse (repasse-t-elle trop sur les mêmes notes ?), lisibilité des
voicings sans fondamentale avec la basse seule en dessous, poids des anticipations.

## Preuves

### Tests de contrat

`harmonium_core/tests/accompaniment_jazz_tests.rs` (14 tests, flux de `MeasureSnapshot`, graines
1, 2, 17, deux tours de 16 mesures, tonalités fa, do, sol ou la selon le test). Les attendus de la
grille sont écrits en clair, relatifs à la tonique.

| Test | Contenu |
|---|---|
| `le_comping_sans_fondamentale_ne_joue_que_des_notes_de_l_accord_publie_avec_ses_tensions` | quatre hauteurs distinctes, classes ⊂ `pitch_classes` (tensions comprises) de l'accord joué, fondamentale absente, tierce et septième présentes, une tension de la qualité présente, type A ou B en bas, tessiture |
| `les_notes_guides_donnent_tierce_septieme_et_une_tension_si_la_qualite_en_a_une` | 2 sons sans tension, 3 avec |
| `le_drop_2_descend_d_une_octave_la_deuxieme_voix_depuis_l_aigu` | `n[2] < n[0] + 12 < n[3]` et voix hautes dans une position serrée |
| `la_conduite_de_voix_bouge_moins_que_la_position_fondamentale_naive` | `rootless`, `guide_tones`, `drop2` : déplacement moyen par voix entre voicings successifs ≤ **1,0 demi-ton** (seuil) et inférieur à celui de la position fondamentale naïve de l'accord publié |
| `le_comping_joue_des_attaques_anticipees_sur_l_accord_suivant_et_des_silences` | au pas 14, des attaques sur le premier accord de la mesure suivante ; des mesures silencieuses ; pas d'attaque hors des rythmes du profil |
| `la_marche_joue_quatre_noires_par_mesure_dans_la_fenetre` | pas 0, 4, 8, 12, durée 4, hauteurs dans 33..55 (fa, do, sol) |
| `la_marche_arrive_sur_la_fondamentale_a_chaque_changement_d_accord_au_pas_pres` | arrivée = `bass_pc` publié à chaque changement ; mesure 9 (ii-V à la blanche) et mesure 15 (trois accords, un par noire) ciblées au pas |
| `l_approche_chromatique_arrive_a_un_demi_ton_de_la_cible` | `chromatic_weight` seul : toute approche à ±1 demi-ton |
| `l_approche_diatonique_arrive_sur_la_note_de_la_gamme_voisine_de_la_cible` | `diatonic_weight` seul : 1 à 3 demi-tons, dans la gamme, aucune note de la gamme entre l'approche et la cible, un ton entier au moins une fois |
| `les_notes_de_passage_sans_approche_suivent_l_accord_du_pas` | `chord_tone_weight` seul : chaque note de la ligne dans l'accord du pas |
| `la_marche_arrive_sur_la_basse_d_un_accord_slash` | `grille_riche` (`C/E`, `D/C` dont la basse sort de l'accord) |
| `la_marche_et_le_comping_se_rejouent_a_l_identique_et_varient_selon_la_graine` | principe II |
| `rewrite_functional_joue_la_marche_et_le_comping_du_profil` | `jazz_test` : quatre noires, arrivée sur `bass_pc` aux pas 0 et 8, comping sans fondamentale (aucun second chemin) |
| `les_options_incoherentes_sont_refusees_au_chargement` | marche avec `hits`, `steps_per_chord` hors pulsation, durée, `max_leap`, poids nuls, fenêtre de voicing trop étroite, poids de comping nuls, probabilité |

Plus `theory::voice_leading::tests::voicing_movement_sums_rank_matched_voices_and_ignores_extra_ones`.

Mutations vérifiées : lire l'accord de la mesure (`chords[0]`) au lieu de celui de la pulsation dans
`walking_bar` fait échouer cinq tests (arrivée, approches, passage, rewrite). **Limite connue** :
retirer le terme de mouvement de la clé de choix de `ChordPlayer::voice` **ne fait pas échouer**
`la_conduite_de_voix…`. Sur cette fixture, la fenêtre et `center` imposent déjà un registre serré :
déplacement moyen par voix, avec puis sans le terme de mouvement, `rootless` 0,54–0,63 puis
0,54–0,64, `guide_tones` 0,61–0,63 dans les deux cas, `drop2` 0,44–0,47 puis 0,57–0,61, contre
1,53–1,78 pour la position fondamentale naïve. Le test garantit le seuil documenté (1,0) et l'écart
à la position fondamentale, pas que le terme de mouvement tienne le seuil à lui seul.

### Les notes des profils distribués ne changent pas

| Vérification | Résultat |
|---|---|
| `harmonium_determinism` natif, 92 fichiers, binaire de `main` (`git archive`, cible séparée) contre la branche | `diff -r` vide : identiques octet pour octet |
| `harmonium-cli --export --seed 1 --bars 32 --record-wav --record-midi` de `reel_quebecois`, `isaac_cellar`, `nt_desert` (binaires `main` et branche, lancés depuis `~/Project/bascanada/harmonium`, `--profile` vers les profils du worktree) | 3 WAV + 3 MIDI identiques (`cmp`) |
| `TUNING_VERSION`, goldens | inchangés (aucun diff) ; aucune liste de profils touchés : aucun profil distribué n'emploie `rewrite_functional` |
| `harmonium_determinism/scripts/run.sh` (wasm-bindgen 0.2.108, `/tmp/n1/wb`) | natif = wasm : 92 / 92 |
| `cargo test --workspace -q --no-fail-fast` | 767 réussis, 0 échec, 2 ignorés (752 avant : +14 tests de contrat, +1 de `voice_leading`) |
| `cargo test -p harmonium --features odin`, `cargo test -p harmonium_bevy` | 118 réussis (odin), 11 réussis (Bevy) |
| `cargo check -p harmonium_studio` ; `cargo check --target wasm32-unknown-unknown` (`harmonium_core`, `harmonium_audio`, `harmonium` avec et sans `odin`, `harmonium_bevy --no-default-features`) | passent |
| `cargo clippy -p harmonium_core -p harmonium -p harmonium-cli -p harmonium_lab --all-targets`, comparé à `main` | aucun avertissement nouveau |
| `cargo +nightly fmt --all -- --check` | propre |

Temps réel : la génération reste côté compositeur. Les voicers emploient `ArrayVec` sur la pile
(voicings de cinq hauteurs, candidats de passage) ; `walking_bar` alloue deux `Vec` par mesure
comme `bass_bar`. Rien n'est ajouté au chemin audio.

## Limites

- **Arrivées à octave fixe** : la racine est placée par `place_in_range` dans `low..high`. Deux
  arrivées successives peuvent être à onze demi-tons l'une de l'autre ; la ligne ne cherche pas la
  continuité d'octave d'une arrivée à la suivante.
- Une note par pulsation : pas de croches de passage, de notes répétées ni fantômes, pas de « two-feel »
  (blanches). Le swing vient du swing du profil, non de la marche.
- La marche ne joue que les mesures de tour : l'intro « vamp » joue des accords sans basse, l'outro
  joue la tonique.
- Les changements d'accord doivent tomber sur une pulsation (validation) : `steps_per_chord` non
  multiple de `steps_per_pulse` est refusé avec la marche.
- Les tensions viennent de la qualité écrite : sans elles, un voicing sans fondamentale répète la
  tierce (3-5-7-3). Le choix de la tension de couleur (9, 13, #11, b9, #9, b13, 11) est une priorité
  codée sans corpus ; sur un `7alt`, elle donne la #11 plutôt que la b9.
- Le comping tire une mesure à la fois : pas de motif par partie ni de phrase de plusieurs mesures
  (la banque de basse a `part` et `bars`, pas celle d'accords).
- `anticipate` ne regarde que l'accord à la fin de la durée : une anticipation de deux accords n'existe pas.
- La conduite de voix compare des voicings par rang depuis le grave ; entre un voicing à deux sons
  (`guide_tones`) et un à trois (avec tension), la voix en surplus est ignorée.
- Le harnais natif/wasm ne couvre que les profils distribués : `grille_accompagnee` n'y est pas
  (le chemin n'emploie que des entiers et ChaCha8). E7 y fera entrer le profil jazz distribué.
- Les notes de `jazz_test` ont changé (voir plus haut) : fixture, aucune écoute associée.

## Ce qu'E7 devra calibrer (WJazzD, GMD, JTD)

Décision F3 : pas de source distribuable de voicings ; les voicings restent des règles codées,
validées à l'écoute. Côté données, E7 (profil `wjazzd_swing`) devra fixer, avec les mesures du labo :

- **Basse (WJazzD, statistiques par temps, ODbL)** : fenêtre `low..high` / `walk_low..walk_high` ;
  poids `approach` (aucune, chromatique, diatonique) avant chaque changement d'accord **et** avant le
  premier temps, par temps ; poids `passing` (note d'accord ou pas diatonique) par temps ; distribution
  des intervalles entre notes consécutives (`max_leap`, `toward_target_weight`,
  `keep_direction_weight`) ; vélocité et `accent` ; `duration` (legato ou détaché) selon le tempo ;
  fréquence d'une fondamentale hors arrivée au premier temps (non représentable aujourd'hui).
- **Accords (JTD ou autre corpus du labo, jamais distribué)** : banque `comping` (positions des attaques
  dans la mesure, durées, part des anticipations, part des silences, par rôle de mesure) ; choix
  entre `rootless`, `guide_tones` et `drop2` ; fenêtre `low..high`, `center`, `max_drift` ; vélocités.
- **Percussion (GMD si utile)** : hors de cette tâche (le ride/charleston existent déjà).
- **Rythme harmonique** de la grille d'entrée : `steps_per_chord` de 4 (la noire) suffit pour la
  marche ; une grille à la croche demanderait de lever la contrainte d'alignement sur la pulsation.
- Un profil de bout en bout dans `harmonium_determinism` (natif = wasm).

## Fichiers

`compose/voicing.rs` et `compose/walking.rs` (nouveaux), `compose/accomp.rs` (`ChordPlayer`,
`hit_chord`, candidats), `compose/accomp_profile.rs` (données, validation),
`compose/generator.rs` (`fill_tour_measure`, `publish_chords`, `next_bar_first_chord`,
`chords_for_bar`, `chord_notes_for_bar` ; `fill_rewrite_measure` supprimé), `compose/harmony.rs`
(`DegreeChord::from_chord`), `models/mod.rs` (`has_chords`), `style_profile.rs`,
`theory/voice_leading.rs` (`voicing_movement`), `tests/accompaniment_jazz_tests.rs`, fixture
`tests/fixtures/profiles/grille_accompagnee/`, `profiles/README.md`, `harmonium_core/README.md`.

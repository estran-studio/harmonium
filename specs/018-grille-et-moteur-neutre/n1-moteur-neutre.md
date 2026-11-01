# N1 — Moteur neutre : compte rendu

Phase 018, vague 2. Branche `wquintal/n1-moteur-neutre`, partie de `097106f` (`main` avec E0, E2, E4, E9).
Principe V (constitution 4.5.0) : le générateur neutre se nomme `compose` ; aucun vocabulaire de
tradition ne nomme ses modules, types, pistes ou champs ni n'apparaît dans le schéma public. Principe IV :
aucun alias, aucune lecture de l'ancien schéma.

**Aucun changement musical.** Renommage et déplacement seulement : mêmes notes, mêmes octets (preuves plus bas).

## Étapes (commits)

| Commit | Contenu |
|---|---|
| `f6e848c` | étape 1 : module `trad/` → `compose/` (premier agent, interrompu après) |
| `6f23f28` | étape 2 : types, modules et fonctions Rust (`feet.rs` → `percussion.rs`, `fiddle.rs` → `melody.rs`, `strum.rs` → `chord_spread.rs`, `models/jazz.rs` → `models/rewrite.rs`, `TradGenerator`…) ; compile |
| `9a6d307` | étape 3 : schéma public, pistes, valeurs de chaîne, profils et fixtures, studio, Bevy, CLI, labo, déterminisme, tests |
| `963ed47` | docs : READMEs et `profiles/README.md` |
| dernier | nettoyage de commentaires et de noms de tests restants, ce compte rendu, état dans `plan.md` |

Aucune modification du comportement : la matrice de dumps de `harmonium_determinism` (profils distribués,
graines 1–3, émotions `mid` et `hi`, 64 mesures, 92 fichiers) existait déjà sur `main` ; aucun commit
séparé n'a été nécessaire.

## Décisions de nommage

- **Rôle, pas instrument ni tradition.** Violon → mélodie (`melody`), guitare → accords (`chords`), pieds →
  percussion (`percussion`), `strain` → `part` (les parties de la forme existaient déjà : `form.parts`, `Part`).
- **Pistes `Heel` / `Toe`** → `PercussionLow` / `PercussionHigh` : voix grave et voix aiguë de la percussion.
  Le rôle reste vrai pour les trois usages (talon/pointe, grosse caisse/caisse claire dans Isaac, kit GM).
  `Kick`/`Snare` aurait été faux pour la podorythmie. Canaux MIDI 3 et 4 inchangés.
  Dans la table de percussion, les champs deviennent `low_*` / `high_*` (le contexte porte déjà « percussion »).
- **`PercussionHit::low/high/high_with_delta`** remplacent `FootHit::heel/toe/toe_with_delta`.
- **`trad_functional` → `melody_functional`** : l'harmonie est choisie parmi des accords candidats par mode, ajustée
  à la mélodie et aux cadences. **`jazz_functional` → `rewrite_functional`** : harmonie par grammaire de réécriture.
  Le genre ne nomme plus le modèle ; `GrammarStyle::{Jazz, Pop, Classical, Contemporary}` (enum publique sérialisable)
  devient `{Extended, Triadic, Cadential, Modal}`, d'après les poids de catégories qu'il porte.
- **`strum`, `stroke`, `strings`** (vocabulaire de guitare dans `accompaniment.chords`) : `spread`, `direction`,
  `note_count`. La liste du brief ne les citait pas ; le brief nommait `strum.rs` et demandait l'inventaire complet.
  `strings` des ornements (`double_stop`, cordes à vide du violon) est **conservé** : voir limites.
- **`REEL_*_TOML`** → `DEFAULT_*_TOML` : ce sont les données du profil par défaut embarquées, plus un « reel ».
- **Noms de profils et `[meta]`** (`reel_quebecois`, `isaac_cellar`, `tradition`, `tags`, `corpus`, `name`, `aliases`) :
  inchangés, ce sont des données. Les commentaires et README par profil gardent leur prose de tradition ;
  seuls les noms de champs, de fichiers et de pistes y sont migrés.
- **Lecture des pistes en TOML** : `percussionlow` / `percussionhigh` en minuscules restent acceptés là où
  `heel` / `toe` l'étaient (comparaison insensible à la casse, comme avant) ; ce n'est pas un alias de l'ancien
  schéma, ces noms-là sont rejetés.
- **Snapshot** : `bar_in_strain` → `bar_in_part` change une clé JSON du flux de snapshots. Les goldens
  contiennent cette clé : ils changent **de ce seul nom** (23 lignes, aucune valeur). Rupture de contrat
  (principe I) : l'orchestrateur a porté `TUNING_VERSION` de 21 à 22 dans la même branche, goldens régénérés.

## Table ancien → nouveau

Entrées : **70** lignes (une ligne peut regrouper des noms apparentés).

### Module et fichiers du moteur

| Ancien | Nouveau | Note |
|---|---|---|
| `harmonium_core/src/trad/` | `harmonium_core/src/compose/` | nom fixé par la constitution 4.5.0 (étape 1, `f6e848c`) |
| `compose/feet.rs` | `compose/percussion.rs` |  |
| `compose/fiddle.rs` | `compose/melody.rs` |  |
| `compose/strum.rs` | `compose/chord_spread.rs` | étalement des notes d'un accord |
| `models/jazz.rs` | `models/rewrite.rs` | grammaire de réécriture (règles Steedman) |
| `profiles/*/feet.toml`, `tests/fixtures/profiles/*/feet.toml` (19 fichiers) | `percussion.toml` | git mv |
| `[files] feet = "feet.toml"` (profil) | `[files] percussion = "percussion.toml"` | clé de `[files]` |

### Types, fonctions et constantes Rust

| Ancien | Nouveau | Note |
|---|---|---|
| `TradGenerator` | `TuneGenerator` | génère des airs (`tune`) ; `trad_gen` → `tune_gen` |
| `TradChord` | `DegreeChord` | accord = degré + qualité ; `trad_chord_name` → `degree_chord_name` |
| `TradHarmonyParams` | `AccompanimentParams` | basse, accords, passages |
| `ReelFiddleParams` | `MelodyParams` |  |
| `ReelArrangement` | `Arrangement` |  |
| `FiddleLine` | `MelodyLine` |  |
| `Guitarist` | `ChordPlayer` | `chop_bar` → `chord_bar`, `chop_bar_on_hits` → `chord_bar_on_hits`, `chop_notes_for_bar` → `chord_notes_for_bar` |
| `FeetTable`, `FeetTableError` | `PercussionTable`, `PercussionTableError` |  |
| `FeetPattern`, `FeetPatternPlan` | `PercussionPattern`, `PercussionPatternPlan` |  |
| `FootHit`, `RawFootHit`, `RawFootHitSerialize` | `PercussionHit`, `RawPercussionHit`, `RawPercussionHitSerialize` | constructeurs `FootHit::heel/toe/toe_with_delta` → `PercussionHit::low/high/high_with_delta` |
| `RawFeetTable` | `RawPercussionTable` |  |
| `feet_notes`, `draw_feet_humanize` | `percussion_notes`, `draw_percussion_humanize` |  |
| `deserialize_foot_track`, `deserialize_optional_foot_track` | `deserialize_percussion_track`, `deserialize_optional_percussion_track` |  |
| `Strum` | `Spread` | `ChordVoice.strum` → `spread` |
| `Stroke` | `AttackDirection` | variantes `Block`/`Down`/`Up` conservées |
| `HarmonyPlan::TradFunctional` | `HarmonyPlan::MelodyFunctional` |  |
| `HarmonyPlan::JazzFunctional` | `HarmonyPlan::RewriteFunctional` |  |
| `GrammarStyle::{Jazz, Pop, Classical, Contemporary}` | `GrammarStyle::{Extended, Triadic, Cadential, Modal}` | nommés selon les poids de catégories de règles ; comportement identique |
| `REEL_ARRANGEMENT_TOML`, `REEL_CELLS_TOML`, `REEL_FEET_TOML`, `REEL_INTENSITY_TOML` | `DEFAULT_ARRANGEMENT_TOML`, `DEFAULT_CELLS_TOML`, `DEFAULT_PERCUSSION_TOML`, `DEFAULT_DYNAMICS_TOML` | toujours embarquées depuis `profiles/reel_quebecois/` (donnée) |
| `select_strain_cells`, `strain_target_midi` | `select_part_cells`, `part_target_midi` | `strain: Part` → `part: Part` |
| `load_feet_soundfont`, `FEET_SOUNDFONT` | `load_percussion_soundfont`, `PERCUSSION_SOUNDFONT` | valeur `assets/harmonium_trad.sf2` inchangée (voir limites) |
| `FEET_BANK_OFFSET` (audio), `FEET_BANK_ID` (studio) | `PERCUSSION_BANK_OFFSET`, `PERCUSSION_BANK_ID` |  |
| `feet_sf2_bytes` (audio, host, CLI) | `percussion_sf2_bytes` |  |
| `FeetOpt`, `parse_feet_value`, `parse_feet_command` | `PercussionOpt`, `parse_percussion_value`, `parse_percussion_command` | CLI |
| `EngineCommand::SetFeetDensity` | `EngineCommand::SetPercussionDensity` | `MusicalParams::set_feet_density` → `set_percussion_density` |
| `HarmoniumConfig.feet_bank` (Bevy) | `percussion_bank` | studio : `LoadedResources.feet_bank`, `PackSpec.feet_bank` → `percussion_bank` |

### Champs du schéma public (profils TOML, `StyleProfile`, `TuningParams`, snapshot, commandes)

| Ancien | Nouveau | Note |
|---|---|---|
| `dynamics.guitar_velocity` | `chords_velocity` | miroir de `melody_velocity`, `bass_velocity` |
| `dynamics.feet_velocity` | `percussion_velocity` |  |
| `dynamics.feet_intensity` | `percussion_intensity` |  |
| `dynamics.fiddle_duration` | `melody_duration` |  |
| `dynamics.chop_intensity` | `chords_intensity` |  |
| `dynamics.density_feet` | `density_percussion` |  |
| `dynamics.density_chop` | `density_chords` |  |
| `percussion.heel_note` / `toe_note` | `low_note` / `high_note` | table de percussion |
| `percussion.heel_velocity` / `toe_velocity` | `low_velocity` / `high_velocity` |  |
| `percussion.feet_gain` | `percussion_gain` |  |
| `[accompaniment.chords.strum]` | `[accompaniment.chords.spread]` | mêmes sous-champs |
| `accompaniment.chords.strings` | `note_count` | polyphonie écrite, 3 à 6 ; `up_strings` → `up_note_count` |
| `accompaniment.chords.patterns[].hits[].stroke` | `direction` |  |
| `MusicalParams.feet_density` | `percussion_density` | aussi dans le snapshot d'état et le builder |
| `TunePosition.bar_in_strain`, `BarRole.strain`, `BarRole.bar_in_strain`, `FormMark.bar_in_strain` (studio) | `bar_in_part`, `part`, `bar_in_part`, `bar_in_part` | `bar_in_part` est un champ du snapshot de flux (JSON) ; `strain_lengths` → `part_lengths` |
| CLI `--feet`, `--feet-soundfont`, `--feet-bank` (studio) | `--percussion`, `--percussion-soundfont`, `--percussion-bank` |  |
| REPL `feet <0|1|2>` | `percussion <0|1|2>` | aide, complétion, état |
| automation `[[keys]] feet = …` | `percussion = …` |  |
| studio : manifeste `feet-bank` (rôle, id) | `percussion-bank` | `web-resources.json`, `studio_assets` |

### Pistes (`TrackId`) — canaux MIDI 3 et 4 inchangés

| Ancien | Nouveau | Note |
|---|---|---|
| `TrackId::Heel` (canal 3) | `TrackId::PercussionLow` | voix grave : talon, grosse caisse (Isaac) |
| `TrackId::Toe` (canal 4) | `TrackId::PercussionHigh` | voix aiguë : pointe, caisse claire (Isaac) |
| chaînes de piste `"Heel"`, `"Toe"` (TOML : `track`, `[mix.tracks.*]`, `[performance.tracks.*]`, `[arrangement.entries]`, `density_tracks`, `tracks_off`, `intro_tracks`, `solo_track`) | `"PercussionLow"`, `"PercussionHigh"` | la lecture accepte aussi les minuscules (`percussionlow`), comme `heel` avant |
| `MixTracks.heel` / `toe` (`[mix.tracks.Heel]`) | `percussion_low` / `percussion_high` (`[mix.tracks.PercussionLow]`) | renommage serde `"Heel"` → `"PercussionLow"` |
| `TrackEntries.heel` / `toe` | `percussion_low` / `percussion_high` |  |

### Valeurs de chaîne du schéma

| Ancien | Nouveau | Note |
|---|---|---|
| `harmony.model = "trad_functional"` | `"melody_functional"` | harmonie tirée des accords candidats selon la mélodie et les cadences |
| `harmony.model = "jazz_functional"` | `"rewrite_functional"` | harmonie par grammaire de réécriture |

### Noms de tests et de fichiers de tests

| Ancien | Nouveau | Note |
|---|---|---|
| `harmonium_core/tests/trad_arrangement_tests.rs` | `arrangement_tests.rs` |  |
| `trad_dynamics_tests.rs` | `dynamics_tests.rs` |  |
| `trad_fiddle_tests.rs` | `melody_line_tests.rs` | `trad_fiddle_demo.mid` → `melody_line_demo.mid` |
| `trad_intro_tests.rs` | `intro_tests.rs` |  |
| `trad_mode_shift_tests.rs` (core, host) | `mode_shift_tests.rs` |  |
| `trad_p1t7_listening_tests.rs` | `p1t7_listening_tests.rs` |  |
| `trad_reel_tests.rs` | `tune_stream_tests.rs` |  |
| `jazz_model_tests.rs`, `jazz_profile_tests.rs` | `rewrite_model_tests.rs`, `rewrite_profile_tests.rs` |  |
| `harmonium_host/tests/trad_emotion_tests.rs` | `emotion_tests.rs` |  |
| `harmonium_host/tests/trad_reel_snapshot_tests.rs` | `tune_snapshot_tests.rs` |  |
| `golden_measures/snapshot_stream_reel_seed42_8bars.json` | `snapshot_stream_seed42_8bars.json` | seule différence de contenu : `bar_in_strain` → `bar_in_part` |

### Noms de tests renommés par la même table

Fonctions de test dont le nom portait un mot renommé, remplacé par le nouveau mot : `feet` → `percussion`,
`heel` → `percussion_low`, `strain` → `part`, `strum` → `spread`, `guitar`/`chop` → `chords`/`chord`,
`jazz` → `rewrite`. Exemples : `test_feet_gain_scales_velocities` → `test_percussion_gain_scales_velocities`,
`midi_feet_off_records_no_drums` → `midi_percussion_off_records_no_drums`,
`guitar_chops_on_steps_4_and_12_with_three_chord_tones` → `chord_hits_on_steps_4_and_12_with_three_chord_tones`,
`reel_form_aabb_and_cadences_20_seeds_64_bars` → `form_aabb_and_cadences_20_seeds_64_bars`,
`render_reel` → `render_default_style`, `REEL_BPM` → `DEFAULT_BPM`.

## Preuves

Références prises sur `097106f` (binaires de `main` dans `/tmp/n1-baseline`), comparées à `963ed47`+.
CLI lancé depuis le checkout principal (pour `assets/`), binaire du worktree.

| Contrôle | Résultat |
|---|---|
| Dumps `harmonium_determinism` natifs, 92 fichiers, avant / après | identiques octet pour octet **après substitution** `bar_in_strain` → `bar_in_part` (seule différence : cette clé, 5 760 occurrences) ; somme de la concaténation triée : avant brut `dc27ca9742dc1d22`, avant substitué `434dac3e6c6e569f`, après natif `434dac3e6c6e569f`, après wasm `434dac3e6c6e569f` |
| `harmonium_determinism/scripts/run.sh` (natif = wasm) | 92 / 92 identiques (wasm-bindgen-cli 0.2.108 installé dans `/tmp`, hors `~/.cargo`) |
| Rendus `--export --seed 1 --bars 32 --record-wav --record-midi` de `reel_quebecois`, `isaac_cellar`, `nt_desert`, sans `odin` | 6 fichiers (3 WAV + 3 MIDI) identiques à `main` |
| Mêmes rendus avec `--features odin --odin-presets ~/Project/bascanada/odin-presets` | 6 fichiers identiques à `main` |
| `TUNING_VERSION` | 21 → 22 (clé de snapshot renommée, principe I) |
| Goldens `harmonium_host/tests/golden_measures` | seul le nom de la clé `bar_in_strain` → `bar_in_part` ; fichier renommé `snapshot_stream_seed42_8bars.json` |
| `cargo test --workspace -q --no-fail-fast` | 96 suites vertes, 0 échec |
| `cargo test -p harmonium --features odin` | 118 tests verts |
| `cargo test -p harmonium_bevy` | 11 tests verts |
| `cargo check -p harmonium_studio` | passe |
| `cargo check --target wasm32-unknown-unknown` : `harmonium_core`, `harmonium_audio`, `harmonium` (avec et sans `odin`), `harmonium_bevy --no-default-features` (avec et sans `odin`) | passent |
| `cargo clippy --workspace --all-targets` | 806 avertissements, identiques à `097106f` (même multiset de messages, ligne par ligne hors noms de suites de tests) : aucun nouveau |
| `cargo fmt --all` | propre |

Sommes des WAV (SHA-256 tronqué, avant = après) : `reel_quebecois` `0142c4cdb6234ee1` (identique avec et sans `odin`),
`isaac_cellar` `786a6c1ab114623c` / `odin` `79eb00ac24474f9a`, `nt_desert` `25a814713d31b00b` / `odin` `b4a9b514f4859d62`.
MIDI : `reel_quebecois` `5b2e53afa4355b1f`, `isaac_cellar` `94369a6f333088d8`, `nt_desert` `ec1c8ec101819e0d`.

## Limites et occurrences conservées

`grep -rniE 'trad|reel|fiddle|guitar|chop|feet|heel|toe|strain|podo'` sur le code et le schéma, hors `specs/0xx`,
hors noms et `meta` de profils : chaque reste est justifié.

| Reste | Raison |
|---|---|
| `reel_quebecois`, `reel_acadien`, `reel_britannique_coloniale`, `reel_phrygien`, `reel_mineur_harmonique`, alias `reel`, `jazz_test`, `Reel québécois (fork)` | noms de profils, de fixtures et `meta.name` : des données |
| `tradition` (`[meta]`, `ProfileMeta.tradition`), « traditionnel », « traduire/traduction » | champ de métadonnée du profil (la tradition d'origine est une donnée) ; mots français courants |
| `harmonium_trad.sf2`, `make trad-sf2` | nom d'un fichier binaire fourni par le labo (`assets/` hors dépôt) et cible Makefile du dépôt frère. Le renommer demande de renommer l'asset dans le labo, `web-resources.json` (release publiée), le manifeste du studio déployé et les checkouts des utilisateurs. **À faire avec la migration du labo** (`harmonium_percussion.sf2`) : les constantes `PERCUSSION_SOUNDFONT` et `web-resources.json` sont les deux seuls points de code concernés |
| `GM_PROGRAM_VIOLIN`, `GM_PROGRAM_STEEL_GUITAR`, « violon / guitare acier (programme N) » dans `voice_manager.rs` | noms de programmes General MIDI du standard, utilisés pour dire quel programme manque dans une SoundFont |
| `strings` de `[ornaments.types.double_stop|chord]` et `ornaments/realize.rs` | cordes à vide en hauteurs MIDI pour réaliser des doubles cordes : vocabulaire d'ornementation (`DoubleStop`, `Trill`, `Roll`, `GraceSingle`…), pas dans la liste du brief. À reprendre avec les ornements si on veut les nommer par rôle (`open_pitches`) |
| `profiles/*/README.md`, fixtures `README.md`, commentaires de `profile.toml`/`percussion.toml`/`arrangement.toml`/`dynamics.toml`/`cells.toml`/`set.toml`, `tags` | documentation et commentaires de données de profil : décrivent la tradition modélisée |
| `harmonium_lab/src/benchmark/profiles.rs` (`DNAProfile::jazz`, `baroque`…) | profils de distance au corpus du labo : des noms de styles de corpus (données de mesure), pas le moteur |
| `harmonium_studio/site/index.html` | texte public de la vitrine qui décrit la musique des profils distribués |
| `specs/0xx-*`, `README.md` (lien vers `specs/009-phase1-reel`) | spécifications historiques, non réécrites |
| `tetrad`, `constrained`, `trademark`, `Chopin`, `vetoes`, `réelles`, `pointeur`, `instrument` | faux positifs de la recherche (sous-chaînes) |
| `.specify/memory/constitution.md` | exemples de styles dans le texte de la constitution |

Hors périmètre de ce travail, à noter pour la suite :

- Le dépôt frère `harmonium_lab` (Python : `types.py`, `styledist/engine.py`, docs de notation) lit les anciens noms
  (`Heel`, `feet_*`, `trad_functional`, `bar_in_strain`…) : il se migre avec cette table.
- Les specs de la phase 018 (E5, E6, E7) parlent de `jazz_functional` : lire `rewrite_functional`.
- Studio : seuls les renommages ont été appliqués (`live.rs`, `live_screen.rs`, `manifest.rs`, `packager.rs`,
  `studio_assets.rs`, `loading.rs`, `app.rs`, tests) ; la fusion avec `s6-studio-live` ne devrait produire de conflit que sur ces lignes.
- Les profils non commités ailleurs (autres branches) doivent passer la même table avant de se charger : plus de lecture de l'ancien schéma.

## Rejouer

Les commandes de la section « Preuves » se rejouent depuis la racine du worktree. Les rendus se lancent depuis
`~/Project/bascanada/harmonium` : `harmonium-cli --export --style <profil> --seed 1 --bars 32 --record-wav <f.wav> --record-midi <f.mid>`.

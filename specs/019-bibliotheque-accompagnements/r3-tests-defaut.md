# R3 — Aligner les tests sur le profil par défaut habillé (phase 019, vague 5)

`TUNING_VERSION` inchangé (32) : aucune note de profil ne change, seuls des tests et la référence D2 bougent. Le seul code non-test
touché est le module `#[cfg(test)]` de `generator.rs`.

## Cause

Depuis R2-trad, le profil par défaut (`StyleProfile::default()` = `reel_quebecois`) porte une `[session]` : la graine tire tempo, tonalité,
instruments, arc nommé (vélocités qui montent d'un tour à l'autre). Les tests qui lisent le tempo nominal, une tonalité de `melody.key_modes`
ou qui comparent deux tours à l'identique voyaient donc des valeurs tirées.

## Décisions (a : session figée, b : compare à la valeur tirée, c : test faux sur le fond)

Aucun test en (b) ni (c) : chaque test échoué porte sur autre chose que l'habillage (le tempo, la tonalité et l'arc de session sont déjà
couverts par `trad_session_profile_tests`). Figer = `profile.session = Default::default()` dans le profil que le test utilise.

| Test | Déc. | Justification |
|---|---|---|
| `generator::tests::measure_metadata_matches_d3_and_params` (+ `intro_and_tour_same_melody`, qui compare un profil figé à un défaut : les deux figés) | a | Métadonnées D3 (tempo 230) ; l'intro ne doit pas perturber la mélodie. |
| `dynamics_tests::intensity_changes_energy_and_articulation_not_the_current_tune` | a | Intensité ; tempo 220 contre 218 tiré. |
| `arrangement_tests::tours_replay_the_same_melody…` | a | Rejeu des tours ; l'arc `montee` change les vélocités (± 1). |
| `second_ending_snapshot_tests::distributed_final_endings…` | a | Idem sur chaque profil distribué ; l'arc de session change les vélocités d'un tour à l'autre. |
| `mode_shift_tests` ×3 | a | Changements de mode ; tonalité et mode tirés par la session. |
| `set_ramp_tests` ×2 | a | Rampe du set ; tempo de session. |
| `set_signature_tests::signature_repeats…` | a | Signature de set ; vélocités d'arc. |
| `set_contract_tests::published_colors…` | a | Couleurs ⊂ candidates du profil ; la session tire hors des `key_modes`. |
| `rewrite_model_tests` ×2 | a | Accords écrits en sol ; tonalité tirée. |
| `phrase_snapshot_tests` ×3 | a | Fixture en ré dorien (`key_modes`) ; la session déplaçait la tonalité. |
| `phrase_parameter_tests::cadence_chord_qualities…` | a | Qualités d'accord à la cadence ; fixture sans session. |
| `ornaments_contract_tests::graces_end_on_the_carrier…` | a | Ornements ; registre/tonalité tirés. |
| `accompaniment_recipe_tests::accordion_recipe…` | a | Recette d'accompagnement du README (registre d'accordéon) ; la session change les voicings. |
| `r1_styles_tests::meter_pulse_and_tempo_are_published` | a | Contrat de mètre et tempo nominal (`gigue_acadienne` tirait son tempo). |
| `british_colonial_profiles_tests::hornpipe_meter_swing_and_tempo_contract` | a | Idem pour le hornpipe (92 BPM nominal). |
| `style_profile_tests::fixed_roles_gigue_valse_notes_match_the_reference` | a | Référence `main_golden_*` des notes sans habillage : le golden n'est pas régénéré. |
| `harmonium_host` `tune_snapshot_tests` ×2 | a | `flat_tuning()` fige la session. |
| `harmonium_host` `live_host_tests::audible_tempo_is_taken_then_cleared…` | a | Échec réel (2 fois sur 3 en isolé, pas seulement sous charge) : un tempo tiré n'a pas une période entière en trames (22329 ≠ 22330). Profil installé sans session par `SetStyleProfile`. |
| `harmonium_cli` `automation::tests::profile_d2_main_note_reference` | régénérée | Référence du profil par défaut (rôles tirés) : `HARMONIUM_UPDATE_GOLDENS=1`. Seul `profile_d2_main.json` change, source `TUNING_VERSION:32` identique. |
| même test, variante `fixed_roles` | a | Référence historique « rôles fixes » : session figée, `profile_d2_fixed.json` **inchangé** (octet pour octet). |
| `harmonium_studio` `workbench_tests::a_fork_keeps_every_file…` | a (attendu) | Le profil distribué a une banque de plus, `session.toml` : 7 fichiers, pas 6. Pas un défaut du studio : le fork copie bien tout `[files]`. |
| `harmonium_studio` `open_tests::a_changed_profile_file_reloads_once…` | a (attendu) | `ProfileWatch` surveille bien `profile.toml` + les six banques (7), le test comptait six. Rechargement correct. |
| `harmonium_studio` `live_screen_tests::a_forked_profile_saved_edited_and_applied…` | a | Le test édite `default_bpm` 230 → 225 ; la session du fork tire le tempo (224) et l'emporte sur `default_bpm`, comme voulu (D6). L'édition retire aussi `session = "session.toml"` de `[files]`. Pas de changement du studio. |

## Garde-fou

`harmonium_core/tests/default_profile_dressed_tests.rs` : le profil par défaut est `reel_quebecois`, porte une `[session]`, donne ≥ 4
habillages (tempo, tonalité, mode, instrument de mélodie) et ≥ 2 tempos sur 8 graines ; session vidée, le tempo vaut `meter.default_bpm`.
Son en-tête dit la règle : vider `profile.session`, ou comparer à la valeur tirée.

## Résultats (machine partagée, charge 9 à 11)

- `cargo test --workspace` : tous verts, sauf `seek_to_bar_64_under_50ms` (voir plus bas). Lancé par crate ; `export_terminates` seul : 666 s, vert.
- `cargo test -p harmonium --features odin` : tous verts sauf le même seek.
- `cargo +nightly fmt --all --check` : propre.
- `seek_to_bar_64_under_50ms` : **minimum 79 ms** sur 12 lancements (build de test, machine chargée ; 76 ms en R2-trad). Le test ne dépend pas de ce
  changement ; limite tenue seulement sur le Mac (26 ms).
- Dumps `harmonium_determinism` : non relancés ; aucun code hors `#[cfg(test)]` n'a changé, donc identiques à `main` par construction.

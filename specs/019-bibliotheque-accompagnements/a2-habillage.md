# A2 — Habillage par graine et orchestration par chorus

Date : 2026-11-03. Branche `wquintal/a2-habillage`, partie de `main` à `0fe8b52` (`TUNING_VERSION` 26,
**inchangé**). Mécanisme optionnel en données : sans `[session]`, ni les dumps, ni les WAV, ni les MIDI,
ni les goldens ne changent (preuves plus bas). Ne modifie aucun profil distribué (ce sera R2).

## Constat (D6)

« New seed » sur `wjazzd_swing` ou `isaac_cellar` donnait « quasiment la même chose » : une graine ne
changeait que le rythme du comping par mesure et les notes de passage. Tempo, tonalité, instruments,
famille de voicings, kit, registre et arc étaient fixes par profil.

## Ce qui est livré

- **`[session]`** (ou `session.toml`, `[files] session = "session.toml"`), optionnelle, validée au
  chargement avec le chemin du champ (fichier et ligne quand le profil vient de fichiers) :
  `[[session.variants]]` pondérées en **groupes** (un tirage par groupe, les groupes se combinent) et
  `[session.arcs.<nom>]` (arcs nommés, même schéma que `[arc]`).
- **Surcharges en liste blanche**, tirées **une fois par graine** : tempo (valeur ou plage dont la graine
  tire un BPM entier), tonalité et mode (atteignables par le profil), programmes par piste et kit
  (banque et programme), zones Odin par piste (jeux), famille de voicings (`ChordVoicing`, extensible :
  aucune énumération en dur), rapport de swing, registre par piste (octaves, −1 à 1), tags de comping et
  de batterie (motifs, phrases et relances de l'A1), arc nommé, multiplicateurs d'intensité et de densité
  de base. Deux groupes ne règlent jamais le même champ (erreur de chargement).
- **Orchestration par chorus** : `instruments = { … }` dans une variante d'arc (étape ou `fall`) change
  les programmes à la première mesure du chorus ; un chorus qui n'en déclare pas **rend** ceux de la
  session. La clôture garde les programmes de son chorus, l'intro ceux de la session.
- **Visible** : `MeasureSnapshot.instruments` (programmes qui sonnent, pistes mélodiques et kit) et
  `MeasureSnapshot.session` (une entrée par groupe : `group`, `index`, `name`), publiés seulement quand
  le profil orchestre ou a des variantes (sérialisation inchangée sinon, `skip_serializing_if`) ;
  `TuneGenerator::session()`, `MusicComposer::session_variants()`, `composition_bpm` des snapshots =
  tempo nominal de la variante. Donc dans la vérité terrain de `--export --record-truth`.
- **Programmes jusqu'au rendu** : `Measure.instruments` → le playback copie ce `Copy` au chargement de la
  mesure et annonce `AudioEvent::SetInstruments` au premier pas dont les programmes diffèrent de ceux
  déjà annoncés (aucune allocation dans le callback, `RtGuard` inchangé). `VoiceManager` applique
  programmes, banque et canal du kit ; le **MIDI enregistré** écrit un `program_change` par piste qui
  change, à la barre (kit sur le canal GM 9) ; **`timeline_to_midi`** fait de même depuis les mesures.
  Un `UpdateMusicalParams` ou un `ProgramChange` de l'hôte efface l'annonce : la mesure sous la tête la
  redonne au pas suivant (sinon le profil de base écraserait la variante après n'importe quel réglage).
- **Commande de l'hôte** (tableau complet dans `profiles/README.md`) : `SetBpm` gagne tel quel sur le
  tempo de la variante ; `[fixed_tonality]` du profil de l'hôte (donc la grille de training, transposée
  par lui) gagne sur la tonalité (variante ignorée, même inatteignable) ; `SetSwing` gagne
  (`MusicalParams::swing_explicit`, nouveau, remis à faux par un profil ou `Reset`) ; programmes, voicings,
  registre, tags, arc, énergie, Odin : aucune commande, un hôte qui n'en veut pas installe un profil sans
  `[session]`.
- **Fixture** sans Odin `tests/fixtures/profiles/grille_habillage/` : `grille_phrases` (blues de douze
  mesures, quatre chorus, arc de l'A3, phrases et relances de l'A1) sans `[fixed_tonality]`, plus
  `session.toml` : 23 variantes en neuf groupes (3 tempos et swing, 4 tonalités dont un éolien,
  piano / guitare / orgue, contrebasse / basse électrique, baguettes / balais, `rootless` /
  `guide_tones` / `drop2`, registre du comping, énergie, arc de l'A3 ou arc « orchestre » qui change de
  kit et de comping à chaque chorus) : 3 456 habillages possibles.
- Trois refactorisations génériques, sans changement de comportement : `ChorusArc::validate` prend le
  chemin de la section (`arc` ou `session.arcs.<nom>`), les tags se valident par `validate_tags`
  (arc et session), la validation d'une spécification Odin devient `StyleProfile::validate_odin_spec`.

## Algorithme

Module `compose/session.rs`, générique : aucun `match` sur un style, aucune valeur musicale.

1. **Tirage** (`TuneGenerator::ensure_session`, avant le premier air) : deux entiers de 32 bits du RNG
   injecté, **seulement si le profil a des variantes** (sinon le flux est celui d'avant). `ProfileSession::
   draw` en tire un flux ChaCha8 par groupe (`mix(mix(graine, GROUP), rang du groupe)`, entiers `u64`) et
   un flux par plage de BPM. Le rejeu (`reset_to_initial`, `deterministic_seek`) et un changement de profil
   retirent la variante, la même.
2. **Application** (par mesure, état dérivé de la seule position) :
   - tempo : base de la courbe `tempo` de `dynamics` à la place du tempo nominal, sauf `bpm_explicit` ;
   - tonalité : `draw_key_mode` du premier air, sauf `[fixed_tonality]` ;
   - swing : `Measure.swing_ratio`, sauf `swing_explicit` ;
   - voicings : `dressed()` lit l'`AccompanimentParams` habillé (cloné une fois par session) aux deux
     endroits qui voicent le comping ; le reste de l'accompagnement est celui du profil ;
   - arc, intensité, densité, tags : une **base de session** (`SessionBase`) que `ArcBar` lit sous le
     chorus (multiplicateurs composés, tags de repli quand le chorus n'en déclare pas) ; sans arc, un
     `ArcBar` neutre la porte ; l'arc nommé remplace `[arc]` (`chorus_arc_of`) ;
   - programmes : `Measure.instruments` = programmes de la session sous ceux du chorus ;
   - registre : décalage d'octaves entières des notes de la piste, en dernier (une note qui sortirait de
     `0..=127` garde sa hauteur).
3. **Odin** : `MusicComposer::odin_specs` remplace piste par piste `[odin.<Piste>]` par la variante
   tirée ; `missing_resources` signale les presets et les banques de kit que les variantes ou les arcs
   peuvent demander ; `web_resources.py` (`check-profiles`, `stage`) lit les presets des variantes.

## Tests

`harmonium_core/tests/session_variants_tests.rs` (18) et `harmonium_host/tests/session_orchestration_tests.rs`
(5), sur le flux de `MeasureSnapshot`, le MIDI et le moteur hors ligne :

| Contrat | Test |
|---|---|
| tirage stable par graine, une variante par groupe, la même sur toute la session ; seek (rejeu depuis la mesure 31) identique | `le_tirage_est_stable_par_graine_et_rejoue_apres_un_seek`, `le_seek_rejoue_la_meme_variante_et_les_memes_programmes` |
| graines différentes → habillages différents (16 graines : tempos à ±8 BPM, tonalités, programmes, voicing, kit) | `des_graines_differentes_donnent_des_habillages_differents` |
| tempo dans la plage de la variante, courbe de `dynamics` ajoutée ; tonalité transposée ; swing de la variante | `le_tempo_tire_…`, `la_tonalite_tiree_…` |
| l'hôte gagne : `SetBpm`, `SetSwing`, `[fixed_tonality]` (variante ignorée) ; la variante reste visible | `la_commande_de_l_hote_gagne_sur_la_variante`, `la_tonalite_d_une_variante_est_ignoree_sous_un_verrou_…`, `le_tempo_et_le_swing_de_l_hote_gagnent_…` |
| changement de programme au chorus dans les snapshots, dans `timeline_to_midi` et dans le MIDI enregistré par le moteur (barre du chorus ±2 ticks), après un seek | `le_programme_change_a_la_frontiere_du_chorus_dans_…` (3), `apres_un_seek_le_playback_annonce_…` |
| surcharges une à une : voicings, registre (±12 exactement), tags de comping et de batterie, arc nommé, énergie sans toucher au tempo, programmes | `la_famille_de_voicings_…`, `le_registre_tire_…`, `les_tags_de_session_…`, `l_arc_nomme_…`, `l_energie_de_base_…`, `les_programmes_de_session_…` |
| Odin : preset de la variante, signalé absent, basse SoundFont sinon | `le_preset_odin_de_la_variante_remplace_celui_du_profil` |
| sans orchestration ni variante : ni `instruments` ni `session` dans le JSON ; arc nommé seul : flux identique | `un_profil_sans_orchestration_ne_publie_…` |
| erreurs de schéma avec le chemin du champ (23 cas, conflit de deux groupes, champ inconnu), fichier et ligne dans `session.toml`, aller-retour TOML | `les_erreurs_de_schema_…`, `une_erreur_de_la_session_est_situee_…`, `la_section_session_survit_…` |

Mutations vérifiées (chacune fait échouer au moins un test, puis annulée) : `swing_explicit` ignoré,
`[fixed_tonality]` sans effet sur la variante, `SetBpm` sans effet, graine de session constante, playback
qui n'annonce rien, enregistreur qui n'écrit pas les `program_change`, `timeline_to_midi` sans changements.

## Preuves chiffrées

### Contrat inchangé pour les profils existants

| Vérification | Résultat |
|---|---|
| `harmonium_determinism`, binaire de `main` (`git archive` de `0fe8b52`, cible séparée) contre la branche | 98 fichiers identiques octet pour octet ; natif = wasm 98 / 98 (`run.sh`, wasm-bindgen 0.2.108) |
| WAV `--export --seed 1 --bars 44` des 16 profils distribués, CLI de `main` et de la branche (lancés depuis `~/Project/bascanada/harmonium`) | 16 / 16 `cmp` identiques ; WAV et MIDI de `wjazzd_swing`, `nt_desert`, `isaac_cellar`, `reel_quebecois` aux graines 2 et 5 (36 mesures) : 8 / 8 identiques |
| `TUNING_VERSION`, goldens, `profile_d2_main.json` | inchangés |
| `cargo test --workspace -q --no-fail-fast` | 878 réussis, 0 échec, 2 ignorés |
| `cargo test -p harmonium --features odin` ; `cargo test -p harmonium_bevy` | 152 réussis ; 11 réussis |
| `cargo check -p harmonium_studio` ; `cargo check --target wasm32-unknown-unknown` (`harmonium_core`, `harmonium_audio`, `harmonium` sans `standalone`, avec et sans `odin` ; `harmonium_bevy --no-default-features`, avec et sans `odin`) | passent |
| clippy (`harmonium_core`, `harmonium`, `harmonium_audio`, `harmonium-cli`, `--all-targets`) | décompte d'avertissements par fichier identique à celui de `main` ; `cargo +nightly fmt --all --check` propre |
| Seek `deterministic_seek(64)`, graine 42, build de test, minimum de 25 seeks, 3 passages | `reel_quebecois` 25,9 à 26,0 ms (main 26,0 à 26,1) ; `wjazzd_swing` 18,5 ms (main 18,6) ; fixture à variantes 22,6 ms. Rien n'est ajouté pour un profil sans variante |

### Variété qui s'entend, 16 graines de la fixture

Comptes du test `des_graines_differentes_donnent_des_habillages_differents` (flux du générateur, graines
1 à 16) et, indépendamment, des vérités terrain du CLI (`--export --record-midi`, `session_rng`, graines
1 à 16 × 16 chorus, `harmonium-lab seed-variety --chorus 16`) :

| Mesure sur 16 graines | Générateur | CLI (vérité terrain) | Possibles |
|---|---:|---:|---:|
| habillages distincts (tonalité, mode, 9 variantes) | **16** | **16** | 3 456 |
| tempos distincts à ±8 BPM (grappes de 16 BPM) | 3 (100 à 193 BPM) | 3 (100 à 196) | 3 |
| tonalités (classe de hauteur et mode) | 4 | 4 | 4 |
| triplets de programmes (comping, basse, kit) | 9 | 9 | 12 |
| familles de voicings | 3 | 3 | 3 |
| comping / basse / kit | {0, 16, 26} / {32, 33} / {32, 40} | idem | idem |

### `seed-variety --chorus 16` contre la ligne de base

Fixture sans `[session]` (`grille_phrases`, même blues, même arc et mêmes phrases) contre la fixture
habillée, 16 graines × 16 chorus (`harmonium_lab/references/v0-variance/README.md` pour les définitions ;
mesures V0 de `wjazzd_swing` : orchestration 1 séquence distincte sur 16, 0 %) :

| Couche | `grille_phrases` (sans session) | `grille_habillage` | V0 `wjazzd_swing` |
|---|---:|---:|---:|
| **orchestration** : séquences de 16 chorus distinctes | 1 / 16 | **11 / 16** | 1 / 16 |
| orchestration : places où les graines diffèrent | 0 % | **100 %** | 0 % |
| orchestration : distance graduée inter-graines | 0 % | **38 %** | 0 % |
| orchestration : chorus distincts par graine | 1,0 | 1,5 | 1,0 |
| orchestration : mesures changées d'un chorus au suivant | 0 % | 6 % (les graines à arc « orchestre ») | 0 % |
| comping, voicings : distance inter-graines | 15 % | **85 %** | 24 % |
| comping, voicings : places qui varient | 79 % | 100 % | 73 % |
| basse : distance inter-graines | 67 % | 76 % | 31 % |
| accompagnement (toutes couches) : distance inter-graines | 29 % | **46 %** | 22 % |
| accompagnement : séquences distinctes | 16 / 16 | 16 / 16 | 14 / 16 |
| accords (symboles, relatifs à la tonique) | 1 / 16 | 1 / 16 | 1 / 16 |

La signature « accords » de V0 est relative à la tonique : la transposition (4 tonalités) et le tempo
(3 classes) ne s'y voient pas, d'où les comptes de la section précédente. L'orchestration ne tombe pas à
16 / 16 parce que les 16 graines tirent 9 triplets de programmes sur 12 possibles.

**Le labo n'a pas eu besoin d'extension** : sa couche orchestration lit déjà les `program_change` du MIDI
(`--record-midi`) et ils portent maintenant les changements de session et de chorus (4 à 9
`program_change` par fichier selon la graine). Aucune branche du labo.

## Limites et manques moteur relevés

- **Tirage à la première mesure composée.** `session()` est `None` avant ; les zones Odin envoyées avant
  sont celles du profil et le `send_odin` suivant les remplace (le CLI et l'hôte vivant le rappellent à
  chaque tour). Les programmes, le tempo et la tonalité, eux, sont dans les mesures et n'ont pas ce délai.
- **Tonalité : premier air seulement.** Les airs suivants d'un set se tirent comme avant (candidats du
  profil et transitions). Pour une grille de training (un air, plusieurs chorus) c'est la session entière.
- **Aucune commande pour imposer une variante** (« les balais, quelle que soit la graine ») : l'hôte
  impose tempo, swing et tonalité ; le reste s'impose en installant un profil sans `[session]`, ou avec
  une seule variante par groupe. À décider avec training.
- **Registre** : décalage d'octaves entières appliqué après la génération, sans retoucher les fenêtres
  de la conduite de voix ni les tessitures d'instrument ; borné à une octave.
- **Programmes à la barre.** Le changement part au premier pas de la mesure ; les attaques anticipées
  de cette mesure (avant la barre) sonnent encore avec l'instrument précédent. Le MIDI enregistré ne
  transporte pas la banque du kit (seul `percussion_program` change dans le fichier ; les profils de la
  fixture restent en banque 128).
- **Un groupe = un tirage indépendant** : aucune dépendance entre groupes (le swing suit le tempo parce
  qu'ils sont dans la même variante). Un tempo tiré hors de la plage utile d'un feel (shuffle rapide) est
  à la charge des données.
- **BPM entier** pour une plage ; le tempo reste borné par `dynamics.tempo_for` (40 à 300).
- **Odin** : les variantes d'Odin sont vérifiées au niveau du compositeur (zones, patches, ressources
  manquantes) et par la suite `--features odin` existante ; aucune écoute (D5), aucun rendu Odin des
  variantes.

## Fichiers

Moteur : `harmonium_core/src/compose/session.rs` (nouveau), `arc.rs`, `generator.rs`, `mod.rs`,
`params.rs`, `style_profile.rs`, `host_resources.rs`, `report.rs`, `events.rs`, `timeline/mod.rs`,
`timeline/midi_export.rs` ; `harmonium_audio/src/voice_manager.rs`, `backend/recorder.rs` ;
`harmonium_host/src/composer.rs`, `playback.rs`. Outil : `harmonium_studio/scripts/web_resources.py`.
Tests : `harmonium_core/tests/session_variants_tests.rs`,
`harmonium_host/tests/session_orchestration_tests.rs`. Données : `tests/fixtures/profiles/grille_habillage/`.
Docs : `profiles/README.md` (« Habillage par graine et orchestration par chorus »), ce compte rendu, `plan.md`.

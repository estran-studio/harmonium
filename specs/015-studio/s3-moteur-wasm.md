# S3 — Moteur portable wasm32

Branche `wquintal/s3-moteur-wasm` (depuis `259c47a`, S2 et S4 incluses). Plan : `specs/015-studio/plan.md`.

## Ce qui a changé

1. **Tirages à largeur fixe.** `RngCore::next_range_usize` devient `next_range_u64` ; plus aucun tirage
   d'entier sur `usize` dans `harmonium_core`, `harmonium_audio`, `harmonium_host` (ni `harmonium_cli`,
   `harmonium_lab`). rand 0.8 échantillonne un `usize` avec l'entier de la largeur du pointeur : en wasm32 le
   flux ChaCha8 était consommé autrement qu'en natif 64 bits (constat et correctif de S1). Pour une plage
   `u64` sur un hôte 64 bits, l'algorithme de rand est celui de l'ancien `usize` : **la sortie native ne
   change pas** (mesuré, voir plus bas).
2. **Dépendances par cible.**
   - `getrandom` 0.2 feature `js` : `[target.'cfg(target_arch = "wasm32")'.dependencies]` de `harmonium_core`
     (`rand`, `rand_chacha`) et de `harmonium_host` (`thread_rng` du composer).
   - `cpal` feature `wasm-bindgen` : même table, dans `harmonium_host` (le crate reste optionnel :
     `standalone = ["dep:cpal", "dep:rosc"]`).
   - `rosc` passe dans la table `cfg(not(target_arch = "wasm32"))` ; `pub mod osc` n'existe pas en wasm
     (`cfg(all(feature = "standalone", not(target_arch = "wasm32")))`).
   - `web-time` 1 dans `harmonium_host`.
3. **`web_time::Instant`** à la place de `std::time::Instant` dans `clock.rs` (`BeatInfo::next_pulse_at`,
   `TransportClock`), `playback.rs` (`process_buffer(…, Instant)`), `audio.rs` (callback cpal) et
   `live_host.rs` (`beat_info`). En natif, `web_time::Instant` **est** `std::time::Instant` (réexport) : le
   REPL et l'export de `harmonium_cli` passent toujours leurs `std::time::Instant`, sans changement.
4. **Travailleur d'enregistrement pompable.** `worker::Worker` devient `RecorderWorker` (public) : `drain` (vider
   la file), `closed`/`close` (le callback a disparu), `run` (la boucle du thread, inchangée dans son effet) et
   `pump()` (vider, puis fermer si le callback a disparu). Même encodeur (`hound`, `midly`). Natif :
   `RecorderBackend::with_policy` crée toujours son thread. Sans thread : `RecorderBackend::pumped(inner,
   finished, sample_rate) -> (RecorderBackend, RecorderWorker)`, toujours en `DropAndCount` (rien ne pourrait
   attendre). `SharedBuf` passe de `Rc<RefCell<…>>` à `Arc<Mutex<…>>` pour que le travailleur soit `Send` ;
   le verrou est toujours libre (un seul fil, jamais le callback).
   - Hôte : `audio::build_engine` choisit par cible (`build_recorder`) ; en wasm, `EngineControls::recorder`
     porte le travailleur à `LiveHost`, dont `pump()` l'avance. `create_offline_engine` (qui rend un
     `Block` sur un thread) n'existe que hors wasm ; en wasm, `LiveHost::detached` tient ce rôle.
5. **Flux de sortie par cible.** `SendStream` devient `OutputStream` : les `unsafe impl Send/Sync` (justifiés
   par les poignées OS du bureau) sont compilés hors wasm seulement ; en wasm le flux enveloppe des objets JS
   d'un seul fil, n'est pas `Send`, et `LiveHost` non plus (il vit sur le fil de la page). Aucun `unsafe` en
   wasm.
6. **Banc permanent** `harmonium_determinism` (membre du workspace) : voir plus bas.
7. **CI** (`.github/workflows/quality.yml`, job `wasm`), **Makefile** (`wasm/check`, `test/determinism`),
   **constitution 4.3.0** (contraintes techniques : tirages en largeur fixe, moteur compilé et identique en
   wasm, vérifié en CI), READMEs (`README.md`, `harmonium_audio`, `harmonium_host`).

## Audit des tirages

Recherche dans `src/` de `harmonium_core`, `harmonium_audio`, `harmonium_host`, `harmonium_cli`, `harmonium_lab`
(`grep` de `gen_range`, `gen::<`, `r#gen`, `gen_bool`, `gen_ratio`, `choose`, `shuffle`, `sample`, `Uniform`,
`WeightedIndex`, `next_range_`, `thread_rng`, `from_entropy`, `seed_from_u64`).

| Site | Avant | Après |
|---|---|---|
| `algo/mod.rs` : `RngCore` | `next_range_usize(Range<usize>)` | `next_range_u64(Range<u64>)` |
| `style_profile.rs` `draw_weight` | `0..total as usize` | `0..u64::from(total)` |
| `trad/fiddle.rs` : première opération de variation | `0..operations.len()` | `0..len as u64` |
| `trad/fiddle.rs` `apply_rhythm` | `total: usize` | `total: u64` |
| `trad/fiddle.rs` : cellule d'une banque (`pick … as u64`) | `0..total as usize` | `0..total` (`u64`) |
| `trad/fiddle.rs` : rôle libre | `0..(total as usize)` | `0..u64::from(total)` |
| `trad/fiddle.rs` `draw_weighted` | `total: usize` | `total: u64` |
| `trad/accomp.rs` `draw_bass_motif` | `0..total as usize` | `0..u64::from(total)` |
| `trad/feet.rs` `draw_feet_humanize` | `0..(2 * h + 1) as usize` | `… as u64` |
| `trad/dynamics.rs` `draw_parallel_dark` | `0..total as usize` | `0..u64::from(total)` |
| `trad/set.rs` `SetPlan::draw` (longueur, 3 × tirages de tonalité / raccord / signature, pause) | `usize` | `u64` ; `KEY_DRAW_RANGE: usize` → `u64` |
| `trad/generator.rs` `performance_seed` (2 tirages 32 bits) | `0..1 << 32` sur `usize` (`1 << 32` ne tient pas en 32 bits) | `u64` |
| `trad/generator.rs` tirages de tonalité (2) | `0..KEY_DRAW_RANGE` | `u64` |
| `ornaments/realize.rs` : choix d'une option | `0..options.len()` | `0..len as u64` |
| `time_plan/score.rs` `session_rng` | `gen_range(0..7usize)`, `(0..2usize)` | `session_start` : `0..SESSION_KEYS` / `0..SESSION_SCALES` en `u64` |
| `harmonium_host/src/composer.rs` (**site vu par S1, hors du générateur**) | `gen_range(0..keys.len())`, `(0..scales.len())` | appelle `session_start` : un seul code pour le composer et le plan, plus de constantes `7` et `2` dupliquées |

Sans changement : `next_f32` (`gen::<f32>()` : un `u32`, fixe), `thread_rng().gen::<u64>()` (graines de
session), `seed_from_u64`. Aucun `choose`, `shuffle`, `gen_bool`, `gen_ratio`, `WeightedIndex`, `Uniform` ni
`gen::<usize>()` dans le code des crates (les tirages pondérés sont écrits à la main sur `u64`/`u32`). Les
tests qui tirent (`rng.gen_range(0..1_000_000)` dans `trad_fiddle_tests.rs`) n'ont pas de type `usize` et ne
tournent qu'en natif.

`getrandom` dans le graphe wasm (`cargo tree --target wasm32-unknown-unknown -e normal,build`, avec et sans
`odin`) : **0.2.17 seulement** (via `rand_core`). `getrandom` 0.3.4 est dans `Cargo.lock` mais hors du graphe
wasm (aucune arête normale ou de construction) : ni `wasm_js` ni `--cfg getrandom_backend` n'est nécessaire.
`odin2-core` et `oxisynth` ne tirent pas de hasard.

## Banc de déterminisme

`harmonium_determinism/` : un petit membre du workspace (`publish = false`, `cdylib` + `rlib`).

- `src/lib.rs` — le code partagé. `cases()` : 11 profils de `StyleProfile::distributed()` × graines 1–3 × 2
  réglages d'émotions (`mid` = 0,6 / 0,5 / 0,5 / 0,4 ; `hi` = 0,9 / 0,2 / 0,85 / 0,9). `run_case` : par
  `LiveHost::detached` (donc le chemin du studio), `apply` de `SetStyleProfile`, `SetSeed`, `SetEmotionParams`,
  `SetWriteheadLookahead(64)`, puis `pump()` ; il refuse un résultat qui n'est pas exactement les mesures
  `1..=64` ; il rend **un tampon** (`PlaybackEngine::process_buffer(…, web_time::Instant::now())`), pompe de
  nouveau et lit `beat_info(Instant::now())` (muette = erreur) : en wasm, ces appels paniqueraient avec
  `std::time::Instant`. Résultat : les `MeasureSnapshot` en JSON.
- Deux **sondes** texte (`PROBES`), comparées comme les cas : `session.txt` (tonalité et gamme tirées par
  `MusicComposer::new_with_seed` pour 32 graines : le tirage du composer) et `recording.txt` (un WAV enregistré
  par l'hôte sur `assets/test.sf2`, relu avec `hound` : taille du fichier et nombre d'échantillons ; en wasm
  c'est le travailleur pompé qui l'encode).
- `src/main.rs` (`determinism_dump <dossier>`) écrit les 68 fichiers en natif ; `src/wasm.rs` (wasm seulement)
  exporte `case_names`, `run_case`, `probe_names`, `run_probe` ; `scripts/dump_wasm.cjs` les appelle sous Node et
  écrit les mêmes fichiers ; `scripts/run.sh` enchaîne : contrôle de la version du CLI `wasm-bindgen` contre le crate
  de `Cargo.lock`, dump natif (`--release`), `cargo build --release --target wasm32-unknown-unknown`,
  `wasm-bindgen --target nodejs`, dump wasm, `cmp` fichier par fichier ; code de sortie non nul à la moindre
  différence ou si les ensembles de fichiers diffèrent. `run.sh --locked-version` imprime la version à installer.
- `tests/matrix_tests.rs` (natif) : la matrice couvre tous les profils distribués, un cas rejoué rend les mêmes
  octets et 64 mesures, chaque sonde répond.
- Où : un membre plutôt qu'un exemple ou un test, parce que la cible wasm exige un `cdylib` avec exports
  `wasm-bindgen` ; le manifeste de `harmonium_host` n'a pas à porter cette dépendance.

### Le banc voit le défaut (contrôle négatif)

Avec `next_range_u64` réécrit pour tirer sur `usize` les plages tenant en 32 bits (le comportement d'avant) : **1 / 67
fichiers identiques** (seul `recording.txt` ; `session.txt` n'existait pas encore), 66 cas différents, exit 1 :
c'est le 0 / 66 de S1. Avec
seulement `session_start` en `usize` : 23 / 68 identiques, `session.txt` et les graines 2 et 3 de tous les profils
différents, exit 1. Les deux mutations ont été annulées avant les mesures ci-dessous.

## Vérification

Machine : macOS arm64, `cargo 1.101.0-nightly (2026-09-29)` (fmt, build), `rustc 1.99.1` stable (local), Node 25.8.0, `wasm-bindgen` CLI 0.2.108 (= crate verrouillé),
installé sous `.tools/` (ignoré par git).

- `make wasm/check` :
  `cargo check --target wasm32-unknown-unknown -p harmonium_core -p harmonium_audio -p harmonium` puis
  `cargo check --target wasm32-unknown-unknown -p harmonium_audio -p harmonium --features odin` (`harmonium_core`
  n'a pas de feature `odin` ; il est dans le graphe des deux commandes) : verts.
- `WASM_BINDGEN=$PWD/.tools/bin/wasm-bindgen make test/determinism` : **68 / 68 fichiers identiques**
  (66 cas + `session.txt` + `recording.txt`), 0 différent, 68 côté wasm.
  `recording.txt` : `wav_bytes=168004 wav_samples=41984` des deux côtés.
- **Sortie native inchangée** : le même banc compilé contre `259c47a` (archive git, `main`) écrit en natif 68
  fichiers ; `diff -rq` avec ceux de cette branche est vide.
- `cargo +nightly fmt --all -- --check` : propre.
- `cargo test --workspace` : 588 réussis, 0 échec, 2 ignorés (78 suites) ; goldens inchangés
  (`HARMONIUM_UPDATE_GOLDENS` jamais posée, aucun fichier de golden modifié). Nouveaux tests : 2 dans
  `recorder.rs` (WAV d'un travailleur pompé = octets d'un encodage direct, rien publié avant le `pump`, prise
  fermée par `pump` quand le callback disparaît) et 3 dans `harmonium_determinism/tests/matrix_tests.rs`.
- `cargo test -p harmonium --features odin --test odin_contract_tests` : 5 réussis.
- `cargo clippy --workspace --lib --bins --message-format=short` : 156 lignes d'avertissement, dont 149
  localisées (`fichier:ligne`), 24 distinctes sans les numéros de ligne, **sur `main` (`259c47a`, archive) comme
  ici** ; le `diff` des listes (numéros de ligne retirés) est vide. (Un premier passage avait relevé un
  `type_complexity` dans le banc : corrigé par l'alias `Probe`.) `cargo clippy --target wasm32-unknown-unknown`
  sur `harmonium_determinism`, `harmonium_audio`, `harmonium` : rien dans le code ajouté.
- Preuve de l'enregistrement sans thread : le test natif ci-dessus (travailleur pompé, octets identiques à un
  encodage direct) et, en wasm sous Node, `recording.txt` identique à celui du natif.

## Reste ouvert

- **La CI n'a pas tourné** (aucun push demandé) : le job `wasm` est écrit et ses commandes (`make wasm/check`,
  `make test/determinism`) ont été exécutées en local ; reste à voir l'installation de `wasm-bindgen-cli` (cache
  `~/.cargo/bin/wasm-bindgen` ; 30 s de compilation mesurées en local) et `setup-node` sur le runner. Le crate `odin2-core` est un
  dépôt Git (`bascanada/odin-rs`), lisible sans compte (`git ls-remote`).
- **Enregistrement en wasm** : testé sous Node (le travailleur pompé publie un WAV de la bonne durée) mais pas
  dans un navigateur (le flux WebAudio de cpal tourne sur le fil principal : S1/S5).
  Le `pump` doit venir au moins toutes les ≈ 3 s de stéréo à 44,1 kHz (réserve de blocs PCM), sinon la perte est
  comptée (`RecorderStats`) et le WAV garde sa durée avec du silence.
- Le banc compare des snapshots (flottants du générateur compris) ; il ne compare pas l'audio rendu, dont les
  bits bas peuvent différer entre la libm native et celle de wasm.
- Mesures en profil `release` seulement (le banc l'impose) : les assertions de débordement du profil `dev` ne
  sont pas exercées en wasm 32 bits.

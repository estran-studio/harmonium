# S4 — Profils embarqués bruts et décodeur unique

**Statut** : réalisée sur `wquintal/s4-profils-bruts` (base `6b6dbef`). Phase 015, tâche S4 ; prépare S7
(atelier TOML) : l'atelier lit les fichiers d'origine, avec leurs commentaires et leur découpe.

## Ce qui a changé

- **`build.rs`** n'aplatit plus rien. Pour chaque dossier de `profiles/` qui contient un `profile.toml`, il
  lit seulement la table `[files]` pour connaître les banques, puis génère `$OUT_DIR/distributed_profiles.rs` :
  une liste de listes `(nom relatif, include_str!(chemin absolu))`, `profile.toml` d'abord puis les fichiers de
  `[files]`, et rien d'autre. L'ordre par défaut (`reel_quebecois` en tête, puis chemin) et le suivi
  `rerun-if-changed` sont conservés, ainsi que `GIT_VERSION_TAG` et `GIT_VERSION_SHA`. Plus de fusion ni de
  `toml::to_string`.
- **Un seul décodeur**, `StyleProfile::decode` (`harmonium_core/src/style_profile.rs`) : il prend le texte du
  `profile.toml` et une fonction qui livre chaque fichier référencé par son nom relatif. Trois entrées :
  `load` (disque), `from_files` (mémoire), `from_toml` (profil autonome, toute référence externe est refusée).
  La fusion des banques par clé de `[files]` (`cells` → `melody.cells`, `feet` → `percussion`, `arrangement`,
  `dynamics`, `set`) n'existe plus qu'à cet endroit.
- **`StyleProfile::distributed()`** décode les fichiers embarqués par `from_files`, comme `load` décode un
  dossier. Un profil embarqué invalide fait maintenant échouer l'initialisation avec le message d'erreur
  (fichier, ligne) ; avant, il était écarté en silence. `builtin()` et `named()` sont inchangés.
- **Table `intro` de haut niveau** : `decode` la déplaçait déjà dans `arrangement.intro`, comme `build.rs`
  (avec une erreur en plus si `arrangement.intro` existe déjà). Aucun profil distribué n'en a : leur `[intro]`
  est dans `arrangement.toml`. Le comportement est conservé, donc `load()` et `named()` donnent des profils
  identiques.
- **Nouveau module** `harmonium_core/src/profile_source.rs` (privé) : monte chaque fichier lu à son endroit de
  l'arbre du profil et situe une erreur de champ dans le fichier qui le porte.
- **Dépendance** : `toml_edit = "0.22"` dans `harmonium_core`. `toml::Spanned` ne convient pas : toml 0.8
  refuse de le désérialiser sur les tables implicites (`[harmony.parts.A1]` sans `[harmony.parts]`), et
  `toml_edit` 0.20 (celui de `toml` 0.8.2) efface les positions à l'analyse. `toml_edit` 0.22 les conserve
  (`ImDocument`, `Key::span`). `Cargo.lock` : ajout de `toml_edit 0.22.27` et `toml_write`, et
  `toml_datetime` passe de 0.6.3 à 0.6.11.
- **Nom de champ corrigé** : `validate()` signalait `melody.cells[i].notes` alors que la donnée est à
  `melody.cells.cells[i].notes` (le fichier `cells.toml` a une clé `cells`). Le champ suit maintenant le
  chemin réel, ce qui permet de le situer ; `style_profile_tests.rs` est mis à jour.
- `mix.rs` : deux `ProfileError { .. }` littéraux passent par `ProfileError::new`.

## API

```rust
StyleProfile::load(dossier_ou_profile_toml)            // disque
StyleProfile::from_files([("profile.toml", texte), ("cells.toml", texte), …])  // mémoire
StyleProfile::from_toml(texte)                         // autonome
StyleProfile::distributed_files()                      // &[&[(nom, contenu)]], même ordre que distributed()
```

`from_files` accepte tout `IntoIterator<Item = (K, V)>` avec `K, V: AsRef<str>` : une `&BTreeMap<String,
String>` (atelier web) ou un tableau de `(&str, &str)` (embarqué). Il faut `profile.toml` ; chaque autre
nom est celui qu'emploie `[files]`. Les noms de fichiers absents donnent `files.<clé>: fichier absent`.

## Format des erreurs

`ProfileError { field, message, file, line, column }` ; `Display` : `fichier:ligne: champ: message`, sans les
parties inconnues.

| Cas | Exemple |
|---|---|
| Valeur de banque invalide | `cells.toml:9: melody.cells.cells[3]: jeton de note invalide à la position 2 : "oups"` |
| Validation hors banque | `profile.toml:215: mix.master_db: valeur finie attendue dans -60..=48, reçu 99` |
| Syntaxe TOML | `cells.toml:6: invalid string, expected `"`, `'`` (`line` = 6, `column` = 64) |
| Banque absente | `profile.toml:<ligne de files.dynamics>: files.dynamics: No such file or directory` (en mémoire : message `fichier absent`) |

Le fichier est celui de `[files]` pour une banque, le fichier racine sinon. La ligne est celle de la clé du
champ ; si le chemin ne correspond à rien dans le fichier (champ calculé), la ligne est celle du plus proche
ancêtre existant, ou absente quand l'erreur porte sur toute la banque (`melody.cells` vide, par exemple).
Les erreurs de `validate()` sur un `StyleProfile` construit en mémoire, ou lu par `from_toml` sans
fichier, n'ont ni fichier ni ligne.

## Tests

`harmonium_core/tests/profile_files_tests.rs` (dossiers temporaires, déterministes) :

- chaque profil embarqué est égal au même dossier lu par `load` (11 profils) ;
- les fichiers embarqués sont, octet pour octet, ceux de `profiles/<id>/`, dans l'ordre `profile.toml` puis
  `[files]`, et `from_files` les relit en le profil de `distributed()` ;
- valeur invalide dans `cells.toml` : fichier et ligne exacts ; cellule trop courte (erreur de `validate()`)
  : idem ; gain hors bornes dans `profile.toml` ; levée hors mesure dans `arrangement.toml` ;
- erreur de syntaxe : fichier, ligne, colonne ; banque absente : pointe l'entrée de `[files]` ;
- les mêmes erreurs depuis la mémoire (`from_files` sur une `BTreeMap`) et depuis le disque sont égales.

## Vérification

## Résultats

Commandes exécutées dans le worktree :

- `cargo test --workspace` : sortie 0, 554 tests réussis, 0 échec, 2 ignorés (71 suites), dont les 9 de
  `profile_files_tests`. Goldens (`harmonium_core/tests/fixtures/main_golden_*.json`) non modifiés,
  `HARMONIUM_UPDATE_GOLDENS` jamais positionné, `TUNING_VERSION` inchangé.
- `cargo clippy --workspace --lib --bins` : 165 lignes `warning` avant (base `6b6dbef`) et 165 après ;
  aucun avertissement dans `profile_source.rs`, `style_profile.rs` ni `mix.rs`.
- `cargo +nightly fmt --all -- --check` : passe.

`build.rs` ne contient plus de logique de fusion (il lit `[files]` pour lister les fichiers) ; le seul
décodeur est `StyleProfile::decode`.

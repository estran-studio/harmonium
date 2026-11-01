# E3 — Identité d'accord complète dans le snapshot

Date : 2026-11-01. Branche : `wquintal/e3-identite-accord`, partie de `main` à `efab271`
(E0, E1, E2, E4, E9 et N1 fusionnées). Rupture de contrat assumée (principe I) :
`TUNING_VERSION` **22 → 23**, goldens régénérés dans le même commit. **Les notes ne changent
pas.**

## Schéma JSON ajouté

Cinq clés dans `MeasureSnapshot` (premier accord de la mesure) et les mêmes cinq, sans le
préfixe `chord_`, dans chaque `chord_changes[].context`. Classes de hauteur **absolues**
`0..=11`, `0` = do, tonalité verrouillée comprise.

| `MeasureSnapshot` | `chord_changes[].context` | Type | Contenu |
|---|---|---|---|
| `chord_root_pc` | `root_pc` | `u8` | fondamentale |
| `chord_bass_pc` | `bass_pc` | `u8` | basse ; la fondamentale hors accord slash |
| `chord_quality` | `quality` | texte ou `null` | qualité typée, suffixe de `theory::quality` (`""`, `m7`, `7b9`, `7alt`…) |
| `chord_extensions` | `extensions` | `[u8]` | intervalles des tensions, croissants (`G7b9` → `[1]`) |
| `chord_pitch_classes` | `pitch_classes` | `[u8]` | notes de l'accord, tensions comprises, fondamentale en tête |

Les champs relus d'anciens JSON prennent leurs défauts (`#[serde(default)]`). `chord_name`,
`chord_root_offset`, `chord_is_minor`, `root_offset`, `is_minor`, `chord_name` et
`scale_guidance` sont inchangés.

### Exemple réel

`grille_riche` en do, graine 1, mesure 13 (`C/E` au pas 0, `G7b9` au pas 8), notes et
position omises :

```json
{
  "chord_name": "C/E", "chord_root_offset": 0, "chord_is_minor": false,
  "chord_root_pc": 0, "chord_bass_pc": 4, "chord_quality": "",
  "chord_extensions": [], "chord_pitch_classes": [0, 4, 7],
  "scale_guidance": null,
  "chord_changes": [
    { "start_step": 0, "context": { "root_offset": 0, "is_minor": false, "chord_name": "C/E",
        "root_pc": 0, "bass_pc": 4, "quality": "", "extensions": [],
        "pitch_classes": [0, 4, 7], "scale_guidance": null } },
    { "start_step": 8, "context": { "root_offset": 7, "is_minor": false, "chord_name": "G7b9",
        "root_pc": 7, "bass_pc": 7, "quality": "7b9", "extensions": [1],
        "pitch_classes": [7, 8, 11, 2, 5], "scale_guidance": null } }
  ]
}
```

## Décisions

| Question | Décision | Raison |
|---|---|---|
| Forme de la qualité | `Option<Quality>` sérialisée en **texte de suffixe** (celui de `Quality::parse` / `Display`) | un seul analyseur et un seul affichage (principe IV) ; `ChordType` seul perdrait les tensions (`7alt`, `maj9`). `""` = triade majeure, `"m"` = mineure. |
| `pitch_classes` ou `all_pitch_classes` | **Avec les tensions** (`all_pitch_classes`) | c'est la sonorité écrite : `G7b9` contient le b9 ; c'est aussi ce que la mélodie du modèle `loop` considère dans l'accord (`DegreeChord::pitch_classes`). Le noyau sans tension se retrouve en retirant `extensions`. |
| Ordre des notes | fondamentale en tête, puis intervalles croissants | stable et indépendant de l'ordre interne de `Chord` (type puis extensions : `7alt` donnerait `0 4 10 1 3 6 8`). |
| Basse hors accord (`D/C`) | publiée dans `bass_pc` seulement, pas dans `pitch_classes` | `pitch_classes` reste l'accord ; la basse a son champ. |
| Pédale | `quality = null`, `pitch_classes` = la hauteur tenue (plus le voisin) | aucune qualité typée à inventer ; pas de gammes (`ScaleGuidance` demande un `Chord`). |
| Intro, outro, harmonie `none` | `quality = null`, `root_pc = bass_pc` = tonique, `pitch_classes` vide, `chord_name` inchangé (nom de la tonique) | aucun accord n'y est joué (l'outro joue la tonique seule) ; ne pas inventer de triade. |
| Gammes `loop` | `harmony.scale_guidance = true` → `ScaleGuidance` par accord, **même calcul** que `rewrite_functional` (`chord_guidance`, `LydianChromaticConcept`, tension harmonique de la mesure) | le drapeau est celui du profil, pas du modèle : `melody_functional` + `true` publie aussi des gammes (aucun profil distribué ne le demande). Ne touche pas les notes : la branche `loop` de `fit_planned_melody` ne lit pas les gammes. |
| `ScaleGuidance.chord_tones` | inchangé : le **type seul**, sans tensions | le changer modifierait `rewrite_functional` hors « nouvelles clés ». |
| Conversion `DegreeChord` → `Chord` | `DegreeChord::to_chord(key, mode)` (`compose/harmony.rs`), unique ; `ChordContext::from_chord` (`timeline/mod.rs`) unique constructeur de l'identité d'un `Chord` | principe IV. `build_chord_context` (loop, melody_functional) et `fill_rewrite_measure` passent tous deux par `from_chord`. |
| Champs historiques | `root_offset`, `is_minor`, `chord_name` gardés | consommés par le studio, l'hôte et le labo. |

### Champs devenus redondants (à trancher plus tard, rien retiré)

- `root_offset` = `(root_pc − tonique) mod 12`.
- `is_minor` se déduit de `quality` quand elle existe (`m`, `m7`, `m7b5`, `dim7`, `mMaj7`…) ; il
  reste la seule indication de mode pour les mesures sans accord.
- `chord_name` = nom de `root_pc` + `quality` + `/` + nom de `bass_pc`. L'orthographe diffère
  selon le modèle (`loop` et `melody_functional` : `pitch_class_name`, bémols ; `rewrite` :
  `Chord::name`) : un client lit les classes de hauteur, pas le nom.

## Ce que training doit lire

- Notation de l'impro : `chord_root_pc` / `root_pc` (fondamentale), `chord_quality` / `quality`
  (symbole), `chord_pitch_classes` / `pitch_classes` (notes de l'accord, tensions comprises),
  `chord_extensions` pour séparer noyau et tensions, `chord_bass_pc` pour la basse.
- Accords en cours de mesure : `chord_changes[].context` à chaque `start_step` (le premier est
  toujours au pas 0 ; `melody_functional` n'a pas de `chord_changes`, la mesure porte l'accord).
- Gammes : `scale_guidance` (mesure) ou `chord_changes[].context.scale_guidance`, présentes
  seulement avec `harmony.scale_guidance = true` dans le profil.
- `quality = null` : ne pas chercher d'accord ; `root_pc` reste la tonique ou la hauteur tenue.

## Preuves

`harmonium_core/tests/chord_identity_tests.rs` (7 tests), sur le flux de `MeasureSnapshot`.
Les attendus de la grille riche sont écrits en clair, relatifs à la tonique, sans appeler
l'API de théorie testée :

| Test | Contenu |
|---|---|
| `le_blues_publie_qualite_basse_fondamentale_et_notes_de_chaque_accord` | `grille_riche`, 12 mesures, tonique 5, 0 et 9, graines 1, 2, 17 : `F7`, `Bb7`, `Gm7` + `C7` (ii-V dans la mesure 9, `start_step` 0 et 8), `F7` + `C7` ; fondamentale, basse, qualité, extensions, notes exactes ; champs de mesure = premier accord |
| `la_cadence_publie_slash_alterations_et_tensions_aux_bons_pas` | `C/E`, `G7b9`, `Dm7b5`, `G7alt`, `Cmaj9`, `D/C` (basse hors accord) en do et en sol, 3 graines |
| `la_json_publie_la_qualite_en_texte_et_se_relit` | clés JSON (`"7b9"`, `[1]`, `root_pc`), relecture, relecture d'un snapshot sans les cinq champs |
| `les_gammes_de_loop_suivent_le_drapeau_sans_toucher_aux_notes` | sans drapeau : aucune gamme ; avec : une par accord (symbole = `chord_name`, racine = `root_pc`, notes d'accord incluses dans `pitch_classes`, `G7b9` → `[2, 5, 7, 11]`), notes **identiques** avec et sans drapeau |
| `melody_functional_publie_une_identite_coherente_avec_chord_name` | `reel_quebecois`, `valse_quebecoise`, `gigue_acadienne`, 64 mesures × 3 graines : le nom relu par `Chord::from_name` redonne racine, basse, qualité, notes ; triades `""`/`m` de trois notes |
| `rewrite_functional_publie_une_identite_coherente_avec_chord_name` | `jazz_test`, 32 mesures × 3 graines : même cohérence ; gammes présentes et cohérentes |
| `pedale_et_harmonie_nulle_n_inventent_pas_de_qualite` | `nt_pedale_boucle` (pédale `null`, accords typés ailleurs), `marche_britannique_coloniale` (`none`) |

Mutations vérifiées : lire `pitch_classes()` au lieu de `all_pitch_classes()`, ou retirer le
tri par intervalle, fait échouer `la_cadence_publie_…`.

### Les notes n'ont pas changé

| Vérification | Résultat |
|---|---|
| `harmonium_determinism` natif (92 fichiers) contre `main` (`git archive main`, cible séparée) | 2 `.txt` identiques octet pour octet ; 90 JSON égaux une fois les nouvelles clés retirées (script ci-dessous) |
| `harmonium-cli --export --seed 1 --bars 32 --record-wav --record-midi` de `reel_quebecois`, `isaac_cellar`, `nt_desert` (binaires `main` et branche, lancés depuis `~/Project/bascanada/harmonium`, `--profile` vers les profils du worktree) | 3 WAV + 3 MIDI identiques (`cmp`) |
| `harmonium_host/tests/golden_measures/snapshot_stream_seed42_8bars.json` (ancien contre nouveau) | égaux une fois les cinq clés de mesure et les cinq clés de `chord_changes[].context` retirées |
| `harmonium_cli/tests/fixtures/profile_d2_main.json` | seul `source` change (`TUNING_VERSION:22` → `:23`) ; `runs` identique |
| `harmonium_core/tests/fixtures/main_golden_*.json`, `profile_d2_fixed.json` | inchangés (aucun diff) |

Script de comparaison des dumps (les deux dossiers viennent de `determinism_dump`) :

```python
# strip.py — retire les clés d'identité d'E3 d'un JSON de snapshots
import json, sys
FLAT = {"chord_root_pc", "chord_bass_pc", "chord_quality", "chord_extensions", "chord_pitch_classes"}
CTX = {"root_pc", "bass_pc", "quality", "extensions", "pitch_classes"}
def strip(v):
    if isinstance(v, list):
        return [strip(x) for x in v]
    if isinstance(v, dict):
        drop = FLAT if "chord_name" in v else set()
        if "chord_name" in v and "root_offset" in v:  # ChordContext
            drop = CTX
        return {k: strip(x) for k, x in v.items() if k not in drop}
    return v
print(json.dumps(strip(json.load(open(sys.argv[1]))), sort_keys=True))
```

```sh
for f in main-dumps/*.json; do
  cmp -s <(python3 strip.py "$f") <(python3 strip.py "branche-dumps/$(basename "$f")") || echo "DIFFÈRE : $f"
done
```

| Autre vérification | Résultat |
|---|---|
| `harmonium_determinism/scripts/run.sh` (wasm-bindgen 0.2.108, `/tmp/n1/wb`) | natif = wasm : 92 / 92 |
| `cargo test --workspace -q --no-fail-fast` (avant régénération, avec `TUNING_VERSION` 23) | seuls `snapshot_stream_unchanged` et `profile_d2_main_note_reference` échouent, comme attendu ; verts après régénération |
| `cargo test -p harmonium --features odin`, `cargo test -p harmonium_bevy` | verts |
| `cargo check -p harmonium_studio` ; `cargo check --target wasm32-unknown-unknown` (`harmonium_core`, `harmonium_audio`, `harmonium` avec et sans `odin`, `harmonium_bevy --no-default-features`) | passent |
| `cargo clippy -p harmonium_core -p harmonium -p harmonium-cli -p harmonium_lab --all-targets`, comparé à `main` | mêmes avertissements, aucun nouveau |
| `cargo +nightly fmt --all -- --check` | propre |

## Limites

- `chord_tones` de `ScaleGuidance` est le type seul tandis que `pitch_classes` porte les
  tensions : deux listes proches, volontairement différentes (voir Décisions).
- Pédale : pas de gammes. Mesures sans accord : pas de gammes.
- `melody_functional` + `scale_guidance = true` publie des gammes ; non demandé par le brief,
  conséquence du drapeau de profil unique. Aucun profil distribué ne l'active.
- `chord_name` des mesures sans accord reste le nom de la tonique, sans `m` en mode mineur
  (inchangé : ne pas toucher aux valeurs existantes) ; `is_minor` suit le mode.
- Le harnais natif/wasm ne couvre pas `grille_riche` ni `jazz_test` (profils non distribués) ;
  le chemin ajouté n'emploie que des entiers et du texte de suffixe déjà couverts.
- Les noms d'accord n'ont pas la même orthographe selon le modèle (bémols ou dièses).

## Fichiers

`timeline/mod.rs` (`ChordContext`, `from_chord`), `compose/harmony.rs` (`DegreeChord::to_chord`),
`compose/generator.rs` (`build_chord_context`, `key_context`, `chord_guidance`,
`modal_chord_context`, `fill_rewrite_measure`), `report.rs` (`MeasureSnapshot`), `tuning.rs`
(version), `tests/chord_identity_tests.rs`, `tests/set_contract_tests.rs` (version), goldens
`snapshot_stream_seed42_8bars.json` et `profile_d2_main.json`, `profiles/README.md`,
`harmonium_core/README.md`.

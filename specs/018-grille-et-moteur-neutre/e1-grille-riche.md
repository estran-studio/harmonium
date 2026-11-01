# E1 — Grille riche dans le modèle `loop`

Date : 2026-11-01. Branche : `wquintal/e1-grille-riche`, partie de `main` à `8650cab`
(`TUNING_VERSION` 22). Décision F1 du plan 018, imposée.

## Forme des données

Un accord de boucle est **relatif à la tonique** ; aucun symbole absolu.

```toml
{ semitones = 2, quality = "m7",  steps = 8 }
{ semitones = 7, quality = "7b9", bass = 11, steps = 8 }
{ degree = 1,    quality = "",    steps = 32 }
```

| Champ | Forme | Décision |
|---|---|---|
| Racine | `degree` (modal `1..7`) **ou** `semitones` (`0..11`), exactement un | `degree` est conservé : il porte le recalcul modal à la bascule de valence (test `valence_attend_frontiere…`). Ce n'est pas l'ancienne écriture des qualités. |
| `quality` | texte de suffixe (`""`, `m`, `m7`, `7b9`, `7alt`…), requis | Lu par `theory::quality::Quality::parse`, le seul analyseur (principe IV). `Major` / `Minor` disparaissent sans alias. |
| `bass` | optionnel, demi-tons `0..11` au-dessus de la tonique | Accord slash. |
| `steps` | durée **en pas**, multiple de `harmony.steps_per_chord`, omission = un créneau | Voir ci-dessous. |

`harmony.default_bars_per_chord` et `bars` sont retirés (`deny_unknown_fields` les refuse :
test `validation_refuse_durees_basses_et_qualites_invalides`).

### Durées : en pas, pas en fraction de mesure

- Le moteur adresse tout en pas (coups de basse, motifs, grille de la mesure) : une durée
  en pas est un entier exact. Une fraction de mesure demanderait un rationnel, sans gain
  (`1/3` n'est pas représentable en flottant, et les mesures à 12 ou 24 pas ne se
  découpent pas en `1/2`, `1/4` partout).
- **Résolution = `harmony.steps_per_chord`** : la grille harmonique existe déjà
  (un créneau par `steps_per_chord`, consommée par la basse, le voicer, la mélodie et la
  passe de basse). Une durée est un nombre entier de créneaux. Pour un ii–V dans une mesure
  de 16 pas : `steps_per_chord = 8` (ou 4) et deux accords de `steps = 8`.
- Validation (`models/chord_loop.rs`) : `steps > 0`, multiple de `steps_per_chord`, somme de
  **chaque alternative** = `rôles de la partie × meter.steps_per_bar`.
- Migration des profils distribués : `bars = n` → `steps = n × steps_per_bar`, `quality
  "Major"` → `""`, `"Minor"` → `"m"`. Les sept profils `loop` (cinq distribués, deux
  fixtures) gardent leurs notes octet pour octet.

## Qualités prises en charge

`theory::ChordType` passe de 17 à 20 variantes (`Dominant7Flat5` `7b5`, `Dominant7Sus2`
`7sus2`, `Altered` `7alt`) et expose `ChordType::ALL`, source unique du suffixe d'un type nu.
`theory/quality.rs` ajoute `Quality { chord_type, tensions }` (`Copy`, sans allocation,
sérialisé en texte) et la grammaire `tête altération*` :

| Famille | Suffixes |
|---|---|
| Triades | `""`, `m`, `dim`, `+`, `sus2`, `sus4`, `add9` |
| Sixtes | `6`, `m6`, `6/9`, `m6/9` |
| Septièmes | `7`, `maj7`, `m7`, `m7b5`, `dim7`, `mMaj7`, `7#5`, `7b5`, `7sus4`, `7sus2` |
| Piles naturelles | `9`, `11`, `13`, `maj9`, `maj13`, `m9`, `m11`, `mMaj9`, `m(add9)`, `9sus4`, `13sus4` |
| Altérations d'une tête dominante | `b9`, `#9`, `#11`, `b13` (`7b9`, `7#9#11`, `9#11`, `13#11`…) puis `b5` / `#5` après `7` seul (`7b9b5`, `7#9#5`) |
| Altéré | `7alt` (fondamentale, tierce, septième + b9, #9, #11, b13) |
| Synonymes lus, jamais écrits | `maj`, `aug`, `add2`, `m(b5)`, `sus` |

Le vocabulaire est celui de `harmonium_practice::ireal::map_quality` (training). Refus
explicites : altération en double ou qui double la pile ou la quinte (`7b9b9`, `9b9`,
`13b13`, `7b5#5`, `7#5b13`), altération sur une tête non dominante (`m7b9`, `maj7#11`),
`b5`/`#5` sur une pile (`9b5`), texte inconnu (`Major`, `69`, `7 b9`).

`Chord::from_name` passe par ce parseur : il remplit `extensions` (intervalles relatifs
croissants : `G7b9` → `[1]`, `C7alt` → `[1, 3, 6, 8]`, `C13` → `[2, 9]`) et la basse (`C/E`).
`C6/9` n'est pas un accord slash (la barre suit un `6` et précède un `9`). `Chord::name`
écrit le même texte ; `from_name(name())` est stable sur 5 fondamentales × 24 qualités × 3
basses. `Chord::identify` (labo) est **inchangé** : il ne renvoie pas les trois nouveaux types.

## Voicer

- `ArrayVec<u8, 5>` sur la pile (au lieu de 3) ; aucune allocation ajoutée.
- Les triades gardent exactement leurs voicings : même gabarit d'inversions, tri ascendant
  sans effet, doublure `chords.note_count` réservée aux voicings de 3 notes.
- Septièmes et tensions : tensions empilées au-dessus de l'octave (9 = 14, 13 = 21), quatre
  ou cinq hauteurs ; au-delà de cinq, la quinte tombe, puis les tensions les plus hautes.
  `7alt` joue fondamentale, tierce, septième, b9, #9.
- **Basse respectée, décision** : `Bass` joue la basse de l'accord (`bass_pc`) sur un coup
  `root` ; un coup `fifth` reste la quinte de la **fondamentale** ; une référence de motif
  `chord_root` prend la basse. `Chords` joue un accord slash **sur sa basse** : basse dans
  le voicing → seules les inversions qui la mettent au grave ; basse hors voicing → ajoutée
  sous un voicing de quatre notes. Les accords non slash gardent le renversement libre.
- `voicing = "fixed_triad"` refuse qualités typées et accords slash (validation, champ
  `accompaniment.chords.voicing`) : sa fenêtre fixe ne couvre pas 5 hauteurs.

## Publication (sans changement de contrat)

- `chord_changes` : un changement par accord distinct de la mesure (créneau 0 toujours, puis
  chaque créneau dont l'accord diffère du précédent). Avec `steps_per_chord` = mesure
  (profils distribués), un seul changement au pas 0, comme avant.
- `chord_name` : racine (bémols, `pitch_class_name`) + suffixe + `/basse`. `is_minor` suit
  `ChordType::is_minor` pour les qualités typées.
- Correction : `fit_planned_melody` indexait les accords du modèle `loop` par rang de changement ;
  il lit maintenant le créneau `start_step / steps_per_chord` (sans effet tant qu'une mesure
  n'avait qu'un changement ; nécessaire dès qu'une mesure en a deux sur une grille plus fine).

## Preuves

Fixture `tests/fixtures/profiles/grille_riche` (blues de 12 mesures, cadence de 4,
`steps_per_chord = 4`) et `harmonium_core/tests/chord_grid_tests.rs`, sur le flux de snapshots,
graines 1, 2, 17 :

| Test | Contenu |
|---|---|
| `blues_en_fa_en_accords_de_septieme_dont_un_ii_v_dans_une_mesure` | tonique 5 (fa), 0 et 9 : `chord_changes` aux bons `start_step` (Gm7 pas 0 + C7 pas 8 dans la mesure 9, F7 + C7 en mesure 12) ; piste Chords = notes de l'accord ; basse et quinte sur Bass ; mélodie dans l'accord du pas |
| `accords_slash_alteres_et_tensions_aux_bons_pas` | `C/E`, `G7b9`, `Dm7b5`, `G7alt`, `Cmaj9`, `D/C` (basse hors accord) en do et en sol |
| `duree_omise_vaut_un_creneau_…` | `maj9` sur 16 pas = quatre `maj9` sans durée : flux identique |
| `validation_refuse_durees_basses_et_qualites_invalides` | durées 0, 5, 62, 60, 72, `bass = 12`, `bars`, `default_bars_per_chord`, 18 qualités invalides, `fixed_triad` + typé |
| `theory::quality::tests` (7), `theory::chord::tests` (+3) | vocabulaire complet lu, types et tensions attendus, affichage canonique relisible, 34 refus, voicings, `from_parts` |

Un test est vérifié par mutation : lire `[slot]` au lieu de `start_step / steps_per_chord`
dans `fit_planned_melody` fait échouer les deux tests de grille (mélodie hors de l'accord).

| Vérification | Résultat |
|---|---|
| `cargo test --workspace -q --no-fail-fast` | 745 réussis, 0 échec |
| `cargo test -p harmonium --features odin`, `cargo test -p harmonium_bevy` | verts |
| `cargo check -p harmonium_studio` ; `cargo check --target wasm32-unknown-unknown` (`harmonium_core`, `harmonium_audio`, `harmonium` avec et sans `odin`, `harmonium_bevy --no-default-features`) | passent |
| `harmonium_determinism/scripts/run.sh` (wasm-bindgen 0.2.108, `/tmp/n1/wb`) | natif = wasm : 92 / 92 |
| Dumps natifs (92 fichiers) contre `main` | `diff -r` vide : identiques octet pour octet |
| `harmonium-cli --export --seed 1 --bars 32 --record-wav --record-midi` de `nt_desert` et `isaac_cellar` (binaire `main` contre binaire de la branche, lancés depuis `~/Project/bascanada/harmonium`, `--profile` sur le dossier du profil) | 2 WAV + 2 MIDI identiques (`cmp`) |
| Export de `grille_riche` (16 mesures, WAV + MIDI) | termine sans erreur |
| `TUNING_VERSION`, goldens | inchangés (aucun diff) |
| `cargo clippy -p harmonium_core -p harmonium -p harmonium-cli -p harmonium_lab --all-targets`, comparé à `main` | aucun avertissement nouveau |
| `cargo +nightly fmt --all -- --check` | propre |

## Limites

- Les changements d'accord tombent sur la grille `steps_per_chord` : pas de changement entre
  deux créneaux, pas de syncope d'accord. Une grille fine (4 pas) rend aussi la basse et le
  voicer plus sensibles au changement ; les profils distribués gardent `steps_per_chord` =
  mesure.
- Dans une mesure, deux accords égaux consécutifs ne publient qu'un changement ; un accord
  identique d'une mesure à l'autre republie le pas 0 (comportement inchangé).
- `ScaleGuidance` reste `None` pour `loop` (E3). `ChordType::is_triad` répond `true` pour
  `Altered` (trois intervalles propres) : sans usage dans le moteur.
- `lydian_chromatic::parent_lydian` classe les trois nouveaux types comme `Dominant7`
  (fondamentale) : approximation, à reprendre avec E3.
- Le harnais natif/wasm ne couvre que les profils distribués : la fixture `grille_riche`
  n'y est pas. Le chemin ajouté (parseur, voicer, `loop`) n'emploie que des entiers ;
  E7 fera entrer un profil jazz distribué au harnais.
- Pas de `maj7#11`, `m7b9`, `9b5`, `+7`, `69`, `sus9` : refusés plutôt qu'interprétés.
- `7alt` choisit fondamentale, tierce, septième, b9, #9 ; le choix des deux altérations
  jouées est une décision de voicing, non issue d'un corpus (E5 reprendra le voicing jazz).

## Ce qu'E3 devra publier

Aujourd'hui seul `chord_name` (texte) porte l'identité d'un accord de boucle. E3 dispose de :

- `DegreeChord { degree, quality: ChordQuality::Typed(Quality) | Major | Minor, bass }` ;
  `Quality::chord_type()` et `Quality::extensions()` (intervalles relatifs) donnent la
  qualité typée et les extensions ; `DegreeChord::bass_pc` / `root_pc` donnent la basse et
  la fondamentale absolues (classes de hauteur).
- À écrire : la conversion `DegreeChord` → `theory::Chord` (`Chord::quality()` existe dans
  l'autre sens) et le calcul de `ScaleGuidance` pour `loop` quand `scale_guidance = true`.
  `Chord::pitch_classes()` n'inclut pas les extensions (`all_pitch_classes()` oui) : à
  trancher pour `chord_tones`.
- `ChordContext.is_minor` est vrai pour tout type mineur ou diminué (`m7`, `m7b5`, `dim7`,
  `mMaj7`…) ; `Altered` et `Dominant7Flat5` sont majeurs.
- Le contrat de snapshot de cette branche n'a pas changé : l'ajout de champs d'identité
  demande le bump de `TUNING_VERSION` et la régénération des goldens (principe I).

## Fichiers

`theory/quality.rs` (nouveau), `theory/chord.rs`, `theory/lydian_chromatic.rs`,
`compose/harmony.rs`, `compose/accomp.rs`, `compose/generator.rs`, `models/chord_loop.rs`,
`style_profile.rs` ; profils `nt_desert`, `isaac_*` (4), fixtures `nt_pedale_boucle`,
`gigue_bourdon` migrés ; fixture `grille_riche` ; `profiles/README.md` (section boucle).

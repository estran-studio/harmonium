# G1 — Modes phrygien et mineur harmonique

Tâche moteur de la phase 014 : `trad::Mode` ne connaissait qu'ionien,
mixolydien, dorien, éolien. Les boucles sombres de NT/Isaac demandent le
phrygien (b2) et le mineur harmonique (sensible, V majeur).

## Ce qui a changé

- `Mode` gagne `Phrygian` (`phrygian`) et `HarmonicMinor` (`harmonic_minor`) :
  intervalles `[0,1,3,5,7,8,10]` et `[0,2,3,5,7,8,11]`, tous deux `is_minor`
  (côté sombre de la bascule de valence), `Mode::ALL` passe à 6.
- Degré II modal (`TradChord::root_semitones`) : suit la seconde du mode
  (bII phrygien à 1 demi-ton), inchangé pour les quatre anciens modes.
- Classification relative des enchaînements (`KeyClass::collection`) :
  phrygien → décalage 4 (relatif majeur diatonique) ; mineur harmonique →
  décalage 9 comme l'éolien (relatif en bIII, convention documentée, le mode
  n'étant pas diatonique).
- Couverture par atteignabilité (`StyleProfile::validate`, `StyleDynamics::validate`) :
  on ferme l'ensemble des modes de `melody.key_modes` et `dynamics.color` par
  `dynamics.parallel_modes` dans les deux sens ; seuls ces modes atteignables
  exigent des accords dans `harmony.candidates` et une bascule claire ↔ sombre.
  Un profil qui n'emploie pas le phrygien ni le mineur harmonique n'a pas à les
  mettre dans sa table `parallel_modes`, ce qui laisse les tirages sombres des
  dix profils distribués strictement identiques.
- Données des 9 profils trad avec harmonie : `profile.toml` gagne les candidats
  phrygien (`i, iv, V, bVII, VI`, V majeur) et mineur harmonique (`i, iv, V, VI`,
  V majeur) — lus par recherche du mode actif, zéro effet tant que le mode n'est
  pas joué. Les `dynamics.toml` distribués restent inchangés octet pour octet.

## Choix et pourquoi

- **V majeur en phrygien.** Le v diatonique phrygien est diminué
  (5–b7–b2 : quinte diminuée), inexprimable en triades majeur/mineur ; le
  v mineur aurait une quinte E naturel hors mode sans fonction cadentielle.
  Le V majeur (sensible empruntée, couleur flamenco/métal) ne gagne que sur
  mélodie de dominante ou aux cadences imposées : le vote libre reste modal.
- **Parallèles clairs : phrygien ↔ mixolydien, mineur harmonique ↔ ionien.**
  Règle : le majeur le plus proche (degrés altérés minimaux), qui partage la
  septième — b7 commune avec le mixolydien (degrés 2, 3, 6 altérés), sensible
  commune avec l'ionien (degrés 3, 6 altérés). Porté en données dans les
  fixtures (`parallel_modes`).
- **Atteignabilité plutôt que 6 modes obligatoires partout.** Ajouter deux
  lignes sombres aux `dynamics.toml` des profils trad changerait la somme des
  poids de `draw_parallel_dark(Ionian, …)` (de 4 à 5) dès que la valence
  descend sous `dark_below` (ex. `demos/automation_combat.toml`), donc les
  notes des profils distribués. Vérifier la couverture sur les seuls modes
  atteignables garde les 10 profils, `TUNING_VERSION` (21) et tous les goldens
  intacts.

## Fichiers

Code : `harmonium_core/src/trad/mode.rs`, `harmony.rs`, `dynamics.rs`, `set.rs`,
`style_profile.rs`, `accomp.rs`, `harmonium_core/tests/g1_modes_tests.rs`.
Données : `profiles/*/profile.toml` (9 profils trad).
Fixtures : `tests/fixtures/profiles/reel_phrygien/`,
`tests/fixtures/profiles/reel_mineur_harmonique/` (copies du reel québécois,
`key_modes` et `color` en ré phrygien / ré mineur harmonique, `dark_below = 0.35`,
`parallel_modes` complet — principe VIII).
Docs : `profiles/README.md`, `specs/014-styles-de-jeu/g1-modes.md`.
## Preuves de test

- `g1_modes_tests` (3 tests) : mélodies dans la gamme sur 4 graines × 64
  mesures pour les deux fixtures ; V majeur (`A`, `root_offset == 7`,
  `!chord_is_minor`) en mineur harmonique ; bascule aller-retour par la
  valence phrygien ↔ mixolydien et mineur harmonique ↔ ionien, notes dans le
  mode actif à chaque mesure.
- Non-régression : `profile_d2_main_note_reference`,
  `profile_without_heterophony_keeps_main_v20_notes`,
  `fixed_roles_gigue_valse_notes_match_the_reference` passent sans régénérer.
- Smoke CLI (`--export --bars 32 --seed 1 --record-midi`) : `D Phrygian` (230
  notes mélodie, les 7 classes `[0,1,3,5,7,8,10]`, 0 hors mode, 5983 o) et
  `D HarmonicMinor` (230 notes, `[0,2,3,5,7,8,11]`, 0 hors mode, 10 mesures
  sur `A` majeur / 0 sur `Am`, 5897 o). `make quality` vert.

## Limites

- Triades majeur/mineur seulement : pas de v° phrygien ni de III+ / vii°
  harmonique ; les notes hors mode du V phrygien (tierce, quinte) ne sonnent
  qu'à l'accompagnement des cadences imposées.
- Les 10 profils trad distribués n'activent pas les deux nouveaux modes dans
  leur `parallel_modes` afin de préserver leurs notes sous valence négative ;
  les profils NT / Isaac (G7, G9) et les fixtures les déclarent en données.

## Ce qu'il faut écouter

- `/tmp/g1-phrygien.mid` et `/tmp/g1-harmonique.mid` (32 mesures, graine 1) :
  sur la grammaire de reel québécois, le phrygien expose la b2 (Mi♭ en ré)
  dans les traits conjoints ; le mineur harmonique fait entendre la sensible
  (Do♯) et la cadence V majeur → i (La majeur → Ré mineur) aux fins de phrase.

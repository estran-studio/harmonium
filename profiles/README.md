# Écrire un profil de style

Un dossier contient `profile.toml` et ses banques TOML. Le moteur embarque
`reel_quebecois` : aucun fichier installé n'est nécessaire. Pour un profil externe,
utiliser `StyleProfile::load(dossier)` (ou le chemin du `profile.toml`).
`from_toml` lit un profil autonome avec toutes les banques inline.

| Section | Champs / sens |
|---|---|
| `[meta]` | `id` ASCII minuscule, `name`, `tradition` précise (région, époque), `tags`, `corpus` (référence au manifeste du labo et aux droits). |
| `[files]` | `cells`, `feet`, `arrangement`, `dynamics` : chemins relatifs au dossier, sans `..`. Omettre pour écrire les sections inline. |
| `[meter]` | `time_signature = {numerator, denominator}`, `steps_per_bar`, `steps_per_unit` (unité des cellules), `steps_per_pulse`, `pulses_per_bar`, `default_swing_ratio`, `default_bpm`, `min_bpm`, `max_bpm`. Tempo à la noire ; pulse dansée indépendante. |
| `[form]` | `sequence` : indices 0-based des parties jouées (AABB = `[0,0,1,1]`). Chaque `[[form.parts]]` donne `name`, `melody_center_offset` en demi-tons et `roles`, un rôle par mesure : `Open`, `Free`, `CadHalf`, `CadFull`, `{CopyOf = n}` (mesure précédente 1-based). |
| `[melody]` | `cell_units` unités par cellule (= une mesure), `strong_units`, `weak_units` (accents 0-based), `key_modes = [{key, mode, weight}]` (classe MIDI 0..11). `params` règle tessiture, registre, continuité, vélocités et poids de rôles. `cells` contient une liste `[[cells]]` avec `role`, `notes`, `weight`. Notes : degrés `1..14`, octave grave `1,`..`7,`, `-` pour prolonger. |
| `[harmony]` | `model = "trad_functional"` (seul modèle implémenté), `steps_per_chord` diviseur de la mesure, `[[harmony.candidates]]` : `mode` et liste `chords = [{degree, quality}]`. Quatre modes disponibles : ionian, mixolydian, dorian, aeolian. |
| `[accompaniment]` | `strong_steps`, poids de vote et `tie_priority`, `bass_steps`, `chop_steps` en pas ; durées, vélocités et tessitures de basse/guitare. |
| `[percussion]` | Notes MIDI talon/pointe, vélocités, gain, humanisation et `[[patterns]]` par densité 0, 1, 2 : coups `{step, track="Heel"|"Toe", delta_velocity}`. Durée zéro = déclenchement de sample. |
| `[instruments]` | Programmes SoundFont 0-based : `melody_program`, `chords_program`, `bass_program`, `percussion_program` et `percussion_bank`. |
| `[arrangement]` | `tours_per_tune`, `intro_each_tune`, `entries` par piste (`tour`, `part`, `pass`). Une piste entrée continue à jouer. |
| `[arrangement.intro]` / `[intro]` du fichier d'arrangement | Formes pondérées d'intro : rôle `percussion`, `tail` ou `vamp`, longueur en mesures, levée en unités, fin accentuée. Consulter `reel_quebecois/arrangement.toml`. |
| `[dynamics]` | Courbes par axe émotionnel : intensité, tempo, vélocités, densité, longueur de note et substitutions harmoniques ; seuils et frontière de changement de mode. Chaque courbe est une liste de points `[entrée, sortie]`, ordonnée ; extrémités constantes. Voir `dynamics.toml` pour les noms complets. |

Le chargeur vérifie les références, bornes, copies antérieures, métrique et banques
avant l'installation. Les erreurs nomment le chemin du champ fautif. Un modèle
d'harmonie non implémenté est refusé : aucune substitution silencieuse.

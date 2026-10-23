# Écrire un profil de style

Un dossier contient `profile.toml` et ses banques TOML. Le moteur embarque
`reel_quebecois` : aucun fichier installé n'est nécessaire. Pour un profil externe,
utiliser `StyleProfile::load(dossier)` (ou le chemin du `profile.toml`).
`from_toml` lit un profil autonome avec toutes les banques inline.

| Section | Champs / sens |
|---|---|
| `[meta]` | `id` ASCII minuscule, `name`, `tradition` précise (région, époque), `tags`, `corpus` (référence au manifeste du labo et aux droits), `aliases` optionnels : noms courts pour `--style` (ex. `["gigue", "jig"]`), uniques parmi les profils distribués. Les tags ne servent pas à choisir un style. |
| `[files]` | `cells`, `feet`, `arrangement`, `dynamics` : chemins relatifs au dossier, sans `..`. Omettre pour écrire les sections inline. |
| `[meter]` | `time_signature = {numerator, denominator}`, `steps_per_bar`, `steps_per_unit` (unité des cellules), `steps_per_pulse`, `pulses_per_bar`, `default_swing_ratio`, `default_bpm`, `min_bpm`, `max_bpm`. Tempo à la noire ; pulse dansée indépendante. |
| `[form]` | `sequence` : indices 0-based des parties jouées (AABB = `[0,0,1,1]`). Chaque `[[form.parts]]` donne `name`, `melody_center_offset` en demi-tons et `roles`, un rôle fixe ou pondéré par mesure (voir « Phrases »). |
| `[melody]` | `cell_units` unités par cellule (= une mesure), `strong_units`, `weak_units` (accents 0-based), `key_modes = [{key, mode, weight}]` (classe MIDI 0..11). `params` règle tessiture, registre, continuité, vélocités et poids de rôles. `cells` contient une liste `[[cells]]` avec `role`, `notes`, `weight`. Notes : degrés `1..14`, octave grave `1,`..`7,`, `-` pour prolonger. |
| `[harmony]` | `model = "trad_functional"` ou `"jazz_functional"`, `steps_per_chord` diviseur de la mesure. Trad : `[[harmony.candidates]]` par mode, `chords = [{degree, quality}]` ; modes ionian, mixolydian, dorian, aeolian. Jazz : `[[harmony.rules]]`, `max_recursion` (0..16, défaut 0), `scale_guidance` (défaut false). |
| `[accompaniment]` | `strong_steps`, poids de vote et `tie_priority`, `bass_steps`, `chop_steps` en pas ; durées, vélocités et tessitures de basse/guitare. Le registre de basse jazz doit couvrir au moins 12 hauteurs consécutives. Pas d'option de voicing parcimonieux à ce stade (voir l'inventaire Legacy). |
| `[percussion]` | Notes MIDI, vélocités, gain, humanisation et `[[patterns]]` par densité 0, 1, 2 : `hits = [{step, track="Heel"\|"Toe", delta_velocity}]` **ou** `euclid = {pulses, steps, rotation}` avec `track = "Heel"` ou `"Toe"` (défaut Heel). Rotation entière, positive vers la droite ; `steps` égale la grille, `0 <= pulses <= steps`. Durée zéro = déclenchement de sample. |
| `[instruments]` | Programmes SoundFont 0-based : `melody_program`, `chords_program`, `bass_program`, `percussion_program` et `percussion_bank`. |
| `[arrangement]` | `tours_per_tune`, `intro_each_tune`, `entries` par piste (`tour`, `part`, `pass`). Une piste entrée continue à jouer. |
| `[arrangement.intro]` / `[intro]` du fichier d'arrangement | Formes pondérées d'intro : rôle `percussion`, `tail` ou `vamp`, longueur en mesures, levée en unités, fin accentuée. Consulter `reel_quebecois/arrangement.toml`. |
| `[dynamics]` | Courbes par axe émotionnel : intensité, tempo, vélocités, densité, longueur de note et substitutions harmoniques ; seuils et frontière de changement de mode. Chaque courbe est une liste de points `[entrée, sortie]`, ordonnée ; extrémités constantes. Voir `dynamics.toml` pour les noms complets. |

Le chargeur vérifie les références, bornes, copies antérieures, métrique et banques
avant l'installation. Les erreurs nomment le chemin du champ fautif. Un modèle
d'harmonie non implémenté est refusé : aucune substitution silencieuse.

## Phrases : rôles pondérés, motifs et cadences

Un rôle fixe reste valide sans consommer de nouveau tirage. Une alternative
pondérée se tire une seule fois par mesure distincte de partie et par air,
sur le `ChaCha8Rng` du générateur. Les reprises et les tours relisent ce choix ;
une invalidation émotionnelle ne le retire pas.

```toml
roles = [
  "Open",
  "Free",
  { SequenceOf = { bar = 1, shift = [{ degrees = -1, weight = 2 }, { degrees = 2, weight = 1 }] } },
  { VariedEndOf = 2 },
  { choose = [{ role = { CopyOf = 1 }, weight = 53 }, { role = "Free", weight = 47 }] },
  { CopyOf = 2 },
  "Free",
  { choose = [{ role = "CadTonic", weight = 22 }, { role = "CadHalf", weight = 8 }, { role = "CadOpen", weight = 70 }] },
]
```

- `CopyOf = k` : copie exacte de la mesure antérieure `k` (1-based).
- `SequenceOf` : mêmes attaques et durées, déplacement uniforme en degrés
  diatoniques, dans le mode publié. `shift` est une distribution de décalages
  `degrees` dans `-14..14`, sans degré zéro dans la notation des notes.
  Une octave commune garde le contour dans la tessiture quand il y tient ;
  le rendu replie par octave les notes qui dépassent, comme les autres cellules.
- `VariedEndOf = k` : première moitié de `k`, deuxième moitié d'une autre
  cellule de la banque, avec attaque au raccord et fin distincte même après
  réduction des octaves. La banque doit offrir deux fins compatibles distinctes.
- `{ CopyOfPart = { part = "A", bar = 7 } }` : copie exacte entre parties.
  La partie référencée doit précéder la partie courante dans la déclaration
  **et** lors de sa première occurrence dans `sequence`.
- `Open`, `Free`, `CadHalf`, `CadFull` : rôles fixes historiques ;
  `CadFull` conserve les cellules d'origine, levées de reprise comprises.
- `CadTonic` : cellule cadentielle, dernier repos tonique tenu au moins deux
  unités de cellule ; harmonie dominante–tonique. Une tenue tonique existante
  est conservée, une levée éventuelle est absorbée par cette tenue.
- `CadOpen` : cellule libre, fin non tonique dans le mode, sans accord imposé.
  `CadHalf` conserve sa fin sur 2, 5 ou 7 et sa dominante harmonique.
  Tous ces rôles cadentiels marquent une frontière de phrase pour le changement
  de mode émotionnel, même si la cadence est ouverte.

Poids entiers non négatifs, somme positive dans `u32`, au plus 64 choix ou
décalages. Pas de choix imbriqué, de référence en avant ni d'auto-référence.
Toutes les branches, même de poids zéro, sont validées avant génération.

### Source des réglages distribués (P3-T1, contrat 11)

Les README de profil citent les fichiers DS-T9 du labo et leurs champs `real`.
Aux m.2/3/5, la référence locale est m.1 ; à m.6, m.2 ; à m.7, m.3.
Les poids `exact`, `sequence`, `varied_ending` viennent de
`real.transformations.<position>.rates`, arrondis sur 10 000 ; `Free` est le
complément. Les déplacements reprennent les effectifs de `real.sequence_shifts`
à la même position, sans inventer une distribution lorsqu'elle est absente.
À B:m7, la copie de A:m7 utilise son taux `exact` mesuré ; les opérations locales
se partagent le complément proportionnellement.

Aux m.4/8, `CadTonic` utilise `tonic_cadence_rate` dans
`real.phrases.rest_by_position`. Faute de classe harmonique finale dans DS-T9,
`CadHalf` emploie **comme proxy**, et non comme mesure de demi-cadence,
`max(0, long_final_rate - tonic_cadence_rate)` ; `CadOpen` reçoit le complément.
Les tirages par position sont indépendants : les taux conjoints m.5–6 et les
structures de paires ne sont pas imposés et doivent être remesurés au labo.
Les banques rythmiques restent inchangées ; le rapport avant/après indique
aussi leur entropie, qui n'est pas garantie meilleure par ce seul changement.

### Reproduire le contrat historique à rôles fixes

Remplacer `roles` dans chaque partie du reel par :

```toml
roles = ["Open", "Free", "Free", "CadHalf", { CopyOf = 1 }, { CopyOf = 2 }, "Free", "CadFull"]
```

Aucun drapeau de compatibilité ni second générateur : ces rôles ne consomment
aucun tirage structurel. Le test `profile_d2_main_note_reference` compare ce
profil à `profile_d2_fixed.json` (ancien main), et le profil distribué au nouveau
`profile_d2_main.json`. Régénération explicite de la référence courante :
`HARMONIUM_UPDATE_GOLDENS=1 cargo test -p harmonium-cli profile_d2_main_note_reference`.

## Modèle fonctionnel jazz

Les règles se lisent dans le profil, sans état partagé ni RNG caché :

```toml
[harmony]
model = "jazz_functional"
steps_per_chord = 8
max_recursion = 2
scale_guidance = true

[[harmony.rules]]
lhs = "V"
rhs = ["II", "V"]
weight = 0.85
min_valence = -1.0
max_valence = 1.0
category = "Preparation"
max_recursion = 0
```

Chaque règle remplace son symbole gauche par la séquence droite ; les symboles
en attente sont consommés aux pas harmoniques suivants. Poids finis positifs
ou nuls (au moins un positif), 1..64 symboles droits, valences ordonnées dans
`-1..1`. `max_recursion > 0` soumet une règle à la limite globale de
back-cycling. Catégories Steedman et multiplicateurs jazz : `Cadential` 0,8,
`Preparation` 1,5, `BackCycle` 1,3, `TritoneSubstitution` 1,2, `Deceptive` et
`ModalInterchange` 1,0. Sans règle applicable : mouvement fonctionnel par
quintes porté de Steedman.

Symboles : `I`, `II`, `III`, `IV`, `V`, `VI`, `VII`, `VofV`, `VofII`,
`VofIV`, `VofVI`, `VofIII`, `FlatII`, `FlatVI`, `FlatVII`. La réalisation
produit des accords absolus à partir de la tonique publiée. Chaque accord est
publié dans `MeasureSnapshot.chord_changes` avec son `start_step` et son
contexte ; les champs historiques de mesure décrivent le premier accord.
`scale_guidance = true` publie aussi les gammes Lydian Chromatic ; les
cellules restent la source du rythme et du contour mélodique, puis les notes
sont rapprochées des accords/gammes publiés et les tenues sont coupées aux
changements d'accord.

Le dossier `../tests/fixtures/profiles/jazz_test/` illustre un profil complet
**de test**, en 4/4 swing et AABA 32 mesures. Il n'est pas embarqué avec les
profils distribués ; un style jazz destiné à l'écoute demandera un corpus
et une phase de validation musicale.

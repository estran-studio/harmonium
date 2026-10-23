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
| `[form]` | `sequence` : indices 0-based des parties jouées (AABB = `[0,0,1,1]`). Chaque `[[form.parts]]` donne `name`, `melody_center_offset` en demi-tons et `roles`, un rôle fixe ou pondéré par mesure (voir « Règles de phrase »). |
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

## Règles de phrase

Le profil décide des répétitions ; le moteur ne suppose ni huit mesures, ni
deux parties, ni un regroupement 4+4. `roles.len()` donne la longueur de chaque
partie ; les références de motifs et les frontières sont 1-based. La première
mesure accepte les mêmes choix pondérés que toutes les autres (sans référence
antérieure locale, puisqu'elle n'en a pas).

Les valeurs ci-dessous sont des **exemples de réglage**, pas des estimations
historiques. Les valeurs omises conservent les reprises/tours identiques,
la résolution `per_tune` et les contraintes de registre antérieures.

| Paramètre | À l'oreille / unité | Mécanique | Libre |
|---|---|---|---|
| `form.sequence` | Ordre et nombre de passes ; indices des parties déclarées | `[0,0,1,1]` | `[0,1]`, ou quatre parties `[0,1,2,3]` |
| `parts.roles` | Une décision par mesure ; première mesure comprise | `"Open"` puis `CopyOf` | `{ choose = [{role="Open",weight=1},{role="Free",weight=3}] }` dès m.1 |
| `form.resolve` | `per_tune` garde le matériel ; `per_pass` retire rôles **et cellules** à chaque passe jouée, tours compris | `"per_tune"` (défaut) | `"per_pass"` |
| `form.repeat_variation` | Probabilité 0..1 de tenter une variation à chaque mesure des passes 2+ | `0.0` (défaut) | `0.35` |
| `form.tour_variation` | Même probabilité aux tours 2+, à partir du premier tour déjà arrangé | `0.0` (défaut) | `0.20` |
| `form.variation_operations` | Menu des opérations autorisées, départ équiprobable ; essaie les autres si l'opération ne convient pas | `["octave"]` | `["varied_end","repeated_notes","octave"]` (défaut) |
| `form.varied_end_keep` | Fraction conservée avant la fin variée, strictement entre 0 et 1, tronquée à l'unité de cellule ; vaut aussi pour `VariedEndOf` | `0.75` | `0.25` ; défaut `0.5` |
| `form.cadence_min_units` | Tenue tonique minimale de `CadTonic`, 1..`cell_units` unités | `4` | `2` (défaut) |
| `form.cadence_search_from` | Fraction 0..1 à partir de laquelle rechercher une tonique déjà tenue ; sinon pose la tenue minimale en fin de mesure | `0.0` | `0.75` ; défaut `0.5` |
| `parts.phrase_ends` | Frontières explicites pour percussion cadentielle et changements de mode émotionnels ; ne remplace pas les rôles harmoniques | `[4,8]` | `[3,8]` (3+5) ; vide/omis : rôles cadentiels |
| `parts.link_previous` | Continuité de hauteur avec la partie précédente ; les `CopyOfPart` restent explicites | `true` (défaut) | `false` : départ indépendant |
| `parts.melody_center_offset` | Centre de la partie en demi-tons relativement au centre mélodique | `0` partout | `0` en A, `2` en B, `-3` en C |
| `form.avoid_adjacent` | Évite de retirer la même cellule libre immédiatement, octave commune ignorée | `false` | `true` (défaut) |
| `form.opening_retries` | Nombre maximal de nouveaux tirages si deux airs commencent pareil, 0..10000 | `0` | `8` (défaut) |
| `form.close_part_loop` | Pénalise le saut de retour de fin de partie vers son début | `true` (défaut) | `false` |
| `form.max_leap_degrees` | Seuil de saut préféré à l'entrée et au retour ; pas diatoniques, 0..28, contrainte souple | `4` (quinte, défaut) | `7` |
| `form.contour_window_semitones` | Écart préféré au centre de partie, 0..127 ; la meilleure candidate sert de repli | `0.0` | `8.0` ; défaut `4.0` |
| `form.register_tolerance_semitones` | Tolérance autour de la frontière médiane des registres, 0..127 | `0.0` | `12.0` ; défaut `6.0` |
| `form.next_same_weight`, `next_relative_weight` | Poids du prochain air : même couleur ou relatif diatonique ; somme positive dans u32 | `1`, `0` | `1`, `1` (défaut) |
| `parts.rhythm` | Distribution de durées aux positions choisies, voir ci-dessous | Un motif unique | Plusieurs motifs aux poids de corpus |
| `parts.second_ending` | Variante de la dernière mesure à la dernière passe de la partie | Omettre, ou `"CadTonic"` | Distribution tonique/quinte/autre |
| `parts.ornament_protected_bars` | Mesures où l'ornement de densité émotionnelle est interdit ; omission : dernière mesure ; `[]` : aucune | `[4,8]` | `[]` |
| `form.ornament_min_steps` | Durée minimale pour scinder une tenue en note + passage d'un pas, 2..256 pas de grille | `256` désactive sur les petites grilles | `3` (défaut) |
| `melody.params.max_continuity_step_distance`, `continuity_max_retries` | Distance diatonique préférée entre cellules et nombre d'essais ; contraintes souples, tessiture prioritaire | `2`, `16` | `7`, `4` |

Une variation conserve le mode et la tessiture. `repeated_notes` remplace une
répétition intérieure par son voisin diatonique ; `octave` déplace la cellule
entière seulement si elle tient dans la tessiture ; `varied_end` emprunte une
fin compatible de la banque. Les cibles et tenues terminales des cadences sont
protégées. Une opération impossible est sautée, sans fabriquer de notes hors
banque ; une mesure sans aucune opération applicable reste identique.
`repeat_variation` se tire au premier tour : si `tour_variation = 0`, ce tour
varié reste identique aux suivants. `per_pass` retire indépendamment chaque
passe, même si les rôles sont fixes ; une copie entre parties vise toujours
le matériel de première passe de la partie source.

### Rythme par position

```toml
# Dans une [[form.parts]], durées en unités de cellule, somme = melody.cell_units.
rhythm = [
  { bar = 4, patterns = [
    { durations = [2,1,1,1,1,2], weight = 3 },
    { durations = [1,1,1,1,4], weight = 2 },
  ] },
  { bar = 8, patterns = [
    { durations = [2,2,4], weight = 1 },
    { durations = [3,1,2,2], weight = 1 },
  ] },
]
```

Le rythme agit sur `Open`, `Free`, `CadOpen`, `CadOther`, `CadHalf` et `CadFifth`.
Les attaques sont repositionnées, la dernière hauteur est conservée puis la
cible cadentielle est appliquée. `CadFull`/`CadTonic` gardent leurs tenues
cadentielles ; les copies, séquences et fins variées conservent le rythme du
motif : les préférences de la position ne cassent jamais une copie.
Chaque distribution contient 1..256 motifs, durées entières positives, poids
non négatifs à somme positive dans u32, une seule entrée par position.

Les sept profils utilisent aux m.4/8 les effectifs `real.rhythm.cells` de leurs
JSON `references/structure` du labo. Préférence cadentielle explicite :
conditionner sur les motifs couvrant exactement la mesure, représentables
sur la grille actuelle, à durées **contrastées** (au moins deux durées
distinctes). Les croches continues, pulsations isochrones et tenues uniques
restent disponibles dans les cellules et cadences fermées ; on ne les ajoute
pas une seconde fois aux fins libres. Aucun arrondi des effectifs, aucune
nouvelle cellule ni retouche des poids de rôles. Le labo ne fournit pas encore
de distribution de motifs de durée *par position* : la même distribution
conditionnelle est donc configurée séparément pour A/B:m.4/8, pas présentée
comme une mesure positionnelle directe. Les fractions et silences du corpus
non représentables par les cellules entières ne sont pas quantifiés en secret.

### Trois recettes

À adapter dans un profil complet ; les snippets remplacent les champs
correspondants, pas les banques et instruments.

**Comptine très répétitive** : `resolve = "per_tune"`, `repeat_variation = 0.0`,
`tour_variation = 0.0`, `avoid_adjacent = false`, `opening_retries = 0`,
`next_same_weight = 1`, `next_relative_weight = 0`. Pour chaque partie :

```toml
roles = ["Open", {CopyOf=1}, {CopyOf=1}, "CadHalf",
         {CopyOf=1}, {CopyOf=2}, {CopyOf=3}, "CadTonic"]
phrase_ends = [4,8]
# Omettre second_ending et rhythm pour garder exactement le matériel.
```

**Marche libre « a b c d »** : `resolve = "per_tune"`, aucune copie imposée,
`repeat_variation = 0.0`, parties de huit mesures (ou toute autre longueur) :

```toml
roles = ["Free", "Free", "Free", "CadOpen", "Free", "Free", "Free", "CadTonic"]
link_previous = false
phrase_ends = [4,8]
# Conserver les préférences rhythm du profil de marche.
```

La recette retire les liens obligatoires ; « a b c d » n'est pas garanti à
chaque tirage, puisque des cellules peuvent fortuitement coïncider.

**Reprise variée de violoneux** : préserver les motifs pondérés du profil,
`resolve = "per_tune"`, `repeat_variation = 0.35`, `tour_variation = 0.15`,
`variation_operations = ["varied_end","repeated_notes","octave"]`,
`varied_end_keep = 0.5`. Conserver `second_ending` pour la vraie conclusion
de A2/B2 ; la variation ne change pas sa cible cadentielle.

### Opérations de motif et cadences

Un rôle fixe reste valide sans consommer de tirage structurel. Une alternative
pondérée se tire selon `resolve`, sur le `ChaCha8Rng` du générateur. Le matériel
de toutes les passes/tours nécessaires est construit au début de l'air ;
une invalidation émotionnelle ne retire aucune de ces décisions.

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
second_ending = { choose = [{ role = "CadTonic", weight = 70 }, { role = "CadFifth", weight = 10 }, { role = "CadOther", weight = 20 }] }
```

- `CopyOf = k` : copie exacte de la mesure antérieure `k` (1-based).
- `SequenceOf` : mêmes attaques et durées, déplacement uniforme en degrés
  diatoniques, dans le mode publié. `shift` est une distribution de décalages
  `degrees` dans `-14..14`, sans degré zéro dans la notation des notes.
  Une octave commune garde le contour dans la tessiture quand il y tient ;
  le rendu replie par octave les notes qui dépassent, comme les autres cellules.
- `VariedEndOf = k` : préfixe de `k` selon `form.varied_end_keep`, puis fin
  d'une autre cellule de la banque, avec attaque au raccord et fin distincte
  même après réduction des octaves. Deux fins compatibles sont requises.
- `{ CopyOfPart = { part = "A", bar = 7 } }` : copie exacte entre parties.
  La partie référencée doit précéder la partie courante dans la déclaration
  **et** lors de sa première occurrence dans `sequence`.
- `Open`, `Free`, `CadHalf`, `CadFull` : rôles fixes historiques ;
  `CadFull` conserve les cellules d'origine, levées de reprise comprises.
- `CadTonic` : cellule cadentielle, dernier repos tonique tenu au moins
  `cadence_min_units` unités (défaut deux) ; harmonie dominante–tonique.
  Une tenue éligible après `cadence_search_from` est conservée, levée absorbée.
- `CadOpen` : cellule libre, fin non tonique dans le mode, sans accord imposé.
  `CadHalf` conserve sa fin sur 2, 5 ou 7 et sa dominante harmonique.
  Tous ces rôles cadentiels marquent une frontière de phrase pour le changement
  de mode émotionnel, même si la cadence est ouverte.
  Les qualités de I/V proviennent de `harmony.candidates`, y compris aux
  cadences ; elles ne sont pas remplacées par une qualité codée selon le mode.
- `CadFifth` : cellule de demi-cadence, dernière attaque ajustée à la quinte ;
  rythme de la cellule ou de la préférence positionnelle, dernier accord dominant.
- `CadOther` : cellule libre, dernière attaque hors tonique/quinte, toujours
  dans le mode ; aucun accord imposé. `CadFifth` et `CadOther` marquent aussi
  une frontière de phrase.

`[[form.parts]] second_ending` est optionnel et accepte exactement les mêmes
rôles fixes, opérations et distributions que `roles`. Il remplace **uniquement
la dernière mesure à la dernière passe de cette partie dans `sequence`** :
A2/B2 en AABB ; A3 si la partie A joue trois fois. Les autres passes utilisent
`roles` quand cette variante n'est pas sélectionnée. Avec les paramètres par
défaut, toutes les autres mesures restent identiques. La partie doit jouer au
moins deux fois. Les références restent strictement antérieures à la mesure
finale. Mélodie et harmonie sont précalculées ; `resolve` et les variations
peuvent diversifier les autres passes. L'outro `end_after_tour` reste prioritaire
et conclut sur la tonique, quel que soit `second_ending`.

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
P3-T3 ajoute les préférences rythmiques par position décrites plus haut ;
les banques et les poids structurels restent inchangés.

### Secondes fins distribuées (P3-T2, contrat 12)

Les poids de `second_ending` sont les **effectifs**, sans arrondi, de la
dernière note écrite des fins finales A/B des parties de huit mesures :
tonique → `CadTonic`, quinte → `CadFifth`, autre → `CadOther`.
La commande de mesure et les quatorze relevés tradition × type se trouvent
dans [le rapport de phase](../specs/011-phase3-phrase/plan.md#p3-t2--fins-de-reprise).
Le parseur du labo complète les mesures partielles par la levée de reprise :
cette note ajoutée est exclue de la fin écrite mesurée. Les premières fins
conservent leurs réglages DS-T9 de **repos tonique**, qui ne sont pas une mesure
de dernière note écrite ; les deux indicateurs ne sont pas interchangeables.
Les taux réels finaux ne justifient pas une tonique obligatoire dans tous les
styles, et les petits effectifs sont signalés dans le rapport.

### Reproduire le contrat historique à rôles fixes

Omettre `second_ending`, `rhythm` et les variations, garder les autres valeurs
par défaut, puis remplacer `roles` dans chaque partie du reel par :

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

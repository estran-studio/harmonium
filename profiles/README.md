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
| `[mix]` | Gain de bus, salle, niveaux/placement/envois des cinq pistes et EQ stéréo ; schéma et bornes ci-dessous. |

Le chargeur vérifie les références, bornes, copies antérieures, métrique et banques
avant l'installation. Les erreurs nomment le chemin du champ fautif. Un modèle
d'harmonie non implémenté est refusé : aucune substitution silencieuse.

## Mixage

Le mixage ne change aucune note ni aucun tirage. Les sept profils déclarent
tous leurs paramètres ; **section absente** : salle sèche (`level = 0`),
pistes à 0 dB / centre / sans envoi, bus 0 dB, coupe-bas et présence neutres.
Les paramètres d'une section explicite sont requis, pas remplis en secret.

```toml
[mix]
master_db = 16.0
[mix.room]
room_size = 0.35
damping = 0.6
width = 0.7
level = 0.28
[mix.tracks]
Melody = { level_db = 0.0, pan = 0.04, reverb_send = 0.16 }
Chords = { level_db = -5.0, pan = -0.35, reverb_send = 0.22 }
Bass = { level_db = -8.0, pan = 0.0, reverb_send = 0.08 }
Heel = { level_db = -6.0, pan = -0.06, reverb_send = 0.10 }
Toe = { level_db = -6.0, pan = 0.06, reverb_send = 0.10 }
[mix.eq]
highpass_hz = 40.0
presence_hz = 2800.0
presence_db = 1.5
presence_q = 0.8
```

| Paramètre | Bornes (fini) | À l'oreille : sec / ample |
|---|---|---|
| `master_db` | −60..48 dB ; gain amplitude `10^(dB/20)` | 0 : unité ; augmenter rapproche la crête de 0 dBFS, sans limiteur ni saturation volontaire. Les valeurs distribuées sont calibrées sur les banques de référence, pas une garantie toutes banques/graines. |
| `room.room_size` | 0..1 (`ReverbParams.roomsize`) | 0 : décroissance courte ; 0,6..1 : longue salle. |
| `room.damping` | 0..1 (`damp`) | 0 : réverbération brillante ; 1 : absorption maximale des aigus. |
| `room.width` | 0..1 | 0 : retour mono ; 1 : retour stéréo large. |
| `room.level` | 0..1 | 0 : sec ; 0,3..0,9 : retour de salle croissant. |
| `tracks.<piste>.level_db` | ≤0 dB, relativement à CC 7 = 127 | 0 : niveau maximal du canal ; négatif : accompagnement en retrait, jusqu'à la quantification à CC 7 = 0. |
| `tracks.<piste>.pan` | −1..1 → CC 10 | −1 gauche, 0 centre (64), +1 droite ; au centre, l'EQ ne déplace pas la piste. |
| `tracks.<piste>.reverb_send` | 0..1 → `round(127 × envoi)`, CC 91 | 0 : aucun envoi ajouté ; 1 : envoi maximal. Les envois intrinsèques SF2 peuvent s'y ajouter. |
| `eq.highpass_hz` | 0 (bypass), ou 20..1000 Hz | Coupe-bas Butterworth 12 dB/octave, Q fixe `1/√2` ; enlève le sous-grave, ne rend pas le signal plus large. |
| `eq.presence_hz` | 20..20000 Hz | Centre d'une bande en cloche (RBJ), p. ex. 2,8 kHz pour la lisibilité de l'archet. |
| `eq.presence_db` | −12..12 dB | 0 : neutre ; positif : présence, négatif : adoucir. |
| `eq.presence_q` | 0,1..10 | Petit Q : large bande ; grand Q : correction étroite. |

Conversion SF2 du niveau : l'atténuation du modulateur CC 7 concave est
`−40 log10(CC7 / 127)` dB ; donc **`CC7 = round(127 × 10^(level_db/40))`**,
pas une conversion linéaire `127 × 10^(dB/20)`. Quantification à 7 bits
(0 = silence, atteint aux très bas niveaux) et aux centibels du synthé.
Le pan est arrondi avec 64 pas à gauche et 63 à droite : centre exactement
64, extrêmes 0/127. Les coefficients des deux biquads et le gain master sont
préparés sur le thread principal au taux de l'hôte ; fréquences limitées à
0,45 × ce taux si Nyquist est inférieur au réglage demandé.

Un seul état par piste dans la lecture : le profil initialise niveau/pan/envoi,
`set gain` modifie le niveau (amplitude 0..1), `mute` filtre les attaques.
Un nouvel air restaure les niveaux du profil sans lever les mutes, donc
`--solo-track Melody` reste une sélection de rendu et ne change pas la génération.
En GM, Heel et Toe partagent le canal 9 : **les réglages de Toe gagnent**,
également dans le MIDI. La banque dédiée les sépare sur 3/4. Les profils ont
les mêmes niveaux/envois aux deux pieds, mais leur léger écart de pan est
réduit à celui de Toe dans le repli GM.

Choix d'espace proposés (pas des mesures historiques) : cuisine amortie pour
le reel québécois, petite salle pour sa gigue, salle communautaire pour le reel
acadien, salle paroissiale plus large pour la valse ; salles de danse courtes
pour gigue/hornpipe britanniques, plein air quasi sec pour fifre-et-tambour.
La guitare est écartée, la basse centrale, les pieds resserrés ; la mélodie
reste près du centre et au-dessus de chaque accompagnement mesuré.
Les README individuels donnent les intentions ; les mesures avant/après et
leurs limites sont dans [Mesures V1](../specs/012-vie-realisme/plan.md#mesures-v1).

## Interprétation

`[performance.tracks.<piste>]` fonctionne pour `Melody`, `Chords`, `Bass`,
`Heel` et `Toe`. Les profils distribués n'ajoutent d'interprétation qu'à la
mélodie ; les durées écrites de basse/guitare et les samples de pieds restent
ceux des banques. **Section absente ou tous les réglages à zéro : aucun
tirage supplémentaire**, mêmes hauteurs, vélocités et durées qu'avant V2.
Les champs omis sont neutres ; `velocity_min = velocity_max = 0` désactive
le bornage supplémentaire.

L'unité de timing est la **fraction de pas**, pas la milliseconde : elle suit
le tempo, le swing et les rampes sans réécrire la grille. À 230 BPM et quatre
pas/noire, `0.02` vaut environ 1,3 ms avant swing. Les décisions sont publiées
dans chaque note : `performance.offset` en millièmes de pas signés,
`performance.gate` en millièmes de durée (1000 = plein),
`performance.breath` en millièmes de pas soustraits **après** le gate.
La résolution de décision est donc 0,001 pas ; la lecture arrondit ensuite
chaque événement à la frame la plus proche dans le pas effectivement gelé.
`start_step`, `BeatInfo` et le transport restent sur la grille.

```toml
[performance.tracks.Melody]
offsets = [-0.02, 0.0, 0.01, 0.0, -0.02, 0.0, 0.01, 0.0,
           -0.02, 0.0, 0.01, 0.0, -0.02, 0.0, 0.01, 0.0]
jitter = 0.02
phrase = [[0.0, -2.0], [0.5, 2.0], [1.0, -3.0]]
velocity_min = 30
velocity_max = 120
breath = 0.08
[[performance.tracks.Melody.rules]]
articulation = "Normal"
weight = 90.0
[[performance.tracks.Melody.rules]]
articulation = "Detached"
weight = 10.0
steps = [0, 4, 8, 12]
min_duration = 1
max_duration = 4
context = "repeated"
shorten = 0.2
[performance.tracks.Melody.emotions]
tension_advance = 0.04
tension_detach = 0.10
arousal_accent = 4.0
valence_velocity = 0.0
density_detach = 0.0
```

Exemple de syntaxe, **pas une estimation de taux traditionnel**. Les README
individuels citent les comptages utilisés et les choix non mesurables.

| Paramètre | Bornes / effet audible | Mécanique | Vivant, point de départ |
|---|---|---|---|
| `offsets` | Vide ou exactement un nombre fini par pas, chacun −0,2..0,2 ; négatif = en avant, positif = en arrière. | `[]` ou zéros | ±0,01..0,025 pas |
| `jitter` | 0..0,1 pas ; uniforme symétrique bornée. Tirée par un `ChaCha8Rng` dérivé d'une graine tirée **une fois par air** sur le ChaCha8 du générateur (uniquement si l'interprétation n'est pas neutre), un flux par mesure : la mesure `n` se joue pareil après un seek, sans horloge. | `0` | 0,01..0,025 |
| `rules[].articulation` | `Normal`, `Detached`, `Linked`, `Accent` ; l'étiquette est publiée, jamais interprétée secrètement dans l'audio. | Aucune règle positive | Détaché : gate court ; lié : gate plein, aucun chevauchement ajouté ; accent : vélocité |
| `rules[].weight` | Poids fini 0..1 000 000 ; tirage parmi **toutes** les règles applicables, addition des poids, pas de priorité implicite. Aucun poids positif : note inchangée. | `0` | Poids de marques écrites conditionnés par position/durée/contexte |
| `rules[].steps` | Positions 0-based dans la mesure ; vide = toutes. | `[]` | Positions de temps/demi-temps du corpus, ramenées à la grille du profil |
| `rules[].min_duration`, `max_duration` | Durée écrite en pas, bornes inclusives ; minimum omis = 0, maximum omis = sans plafond. | `0`, omis | Réserver une règle liée aux tenues, détachée aux notes courtes |
| `rules[].context` | `any`, `repeated`, `ascending`, `descending` ; intervalle avec l'attaque précédente de la même piste **dans la mesure**, première attaque = `any`. | `any` | Poids conditionnels sur répétitions ou contour |
| `rules[].shorten` | Fraction 0..0,8 retranchée à la durée ; pas de gate caché dans l'articulation. | `0` | Détaché : 0,15..0,25 ; lié : 0 |
| `rules[].accent` | Ajout de vélocité 0..20 unités MIDI, après la dynamique existante. | `0` | 2..4 pour `Accent` |
| `phrase` | Points `[phase 0..1, delta −20..20]`, phases strictement croissantes ; interpolation linéaire, extrémités constantes. Phase calculée à l'attaque sur la phrase configurée, pas sur tout l'air. | `[]` ou deltas nuls | −2 → +2 → −3 |
| `velocity_min`, `velocity_max` | Bornes MIDI ordonnées 0..127, appliquées après phrase/accent ; 0/0 = pas de bornage ajouté, vélocité toujours 1..127. | `0`, `0` | 30, 120 |
| `breath` | Fraction 0..0,5 de la durée de la dernière attaque avant chaque frontière `phrase_ends` ; en leur absence, frontières cadentielles du plan. Sans étirer la mesure ni déplacer les suivantes. | `0` | 0,08..0,12 |
| `emotions.tension_advance` | 0..0,2 pas soustraits proportionnellement à la tension ; borne finale de l'attaque ±0,49 pas. | `0` | 0,04 |
| `emotions.tension_detach` | 0..0,4 ajoutés au raccourcissement ; gate final ≥0,1 avant respiration. | `0` | 0,10 |
| `emotions.arousal_accent` | 0..20 unités MIDI ajoutées aux seules notes `Accent` à arousal 1 ; axe brut, distinct de l'intensité combinée. | `0` | 4 |
| `emotions.valence_velocity` | −10..10 unités multipliées par la valence signée. | `0` | 0 si aucune justification d'écoute |
| `emotions.density_detach` | 0..0,3 ajoutés au raccourcissement à densité 1. | `0` | 0 si aucune justification d'écoute |

La durée jouée vaut `max(0.001 pas, durée × gate / 1000 − breath / 1000)`.
Les triggers de durée écrite zéro ont une durée MIDI de référence d'un pas.
Les attaques négatives à la barre sont jouées dans le pas précédent depuis la
mesure préchargée ; au début de lecture/seek ou après sous-alimentation, elles
sont bornées au premier échantillon disponible, pas perdues. Une répétition de
la même hauteur ferme d'abord la note active ; son ancienne fin ne coupe pas
la nouvelle note. Les notes ne sont jamais déplacées dans la partition.

Les ABC donnent des **marques écrites**, pas des gates mesurés ni des timings
de violoneux : absence de marque ≠ absence de jeu expressif. Micro-timing,
gates, courbe de phrase et émotions restent **à valider à l'oreille**. Repères
généraux, non preuves historiques des sept traditions :
[Friberg & Sundberg, 1995, discrimination temporelle](https://pubs.aip.org/asa/jasa/article/98/5/2524/798047/Time-discrimination-in-a-monotonic-isochronous),
[Friberg, Bresin & Sundberg, 2006, règles KTH de phrasé/articulation](https://doi.org/10.2478/v10053-008-0052-x).
Mesures reproductibles : `specs/012-vie-realisme/measure_performance.py`,
MIDI + vérité pour 20 seeds/profil, tension et arousal à 0/1.

## Ornements

`[ornaments]` décrit les agréments idiomatiques ; un seul chemin va du taux du profil
aux notes publiées, la note de passage de densité émotionnelle en fait partie
(`[ornaments.types.passing]`, qui remplace `dynamics.density_passing` et
`form.ornament_min_steps`). **Section absente, types absents ou `enabled = false`,
`rate_scale = 0` : aucun tirage, flux identique à celui d'un profil sans ornements.**
Les ornements sont réalisés **après** l'interprétation ci-dessus : ils composent avec son
décalage, son gate et sa respiration, sans les remplacer.

```toml
[ornaments]
rate_scale = 4.0          # facteur global sur tous les taux écrits
repeat_variation = 0.0    # 0 : une mesure copiée/rejouée porte les mêmes ornements

[ornaments.types.roll]
written_per_100_notes = 0.689826
per_100_notes_by_bar = [1.023166, 0.583255, 1.167532, 0.410754, 1.060606, 0.797922, 0.689299, 0.2083]
min_carrier_eighths = 2.0
arousal_low_multiplier = 0.5   # à arousal 0 ; 1,0 : neutre
arousal_high_multiplier = 1.5  # à arousal 1

[ornaments.types.double_stop]
written_per_100_notes = 0.344225
min_carrier_eighths = 2.0
strings = [55, 62, 69, 76]     # cordes à vide (MIDI) du violon

[ornaments.types.passing]      # densité : aucun tirage
min_carrier_eighths = 1.5
density_curve = [[0.0, 0.0], [0.5, 0.0], [1.0, 1.0]]
```

Exemple de syntaxe : les valeurs distribuées sont celles du labo (DS-T10), voir les README
de profil. Types : `grace_single`, `grace_multiple`, `roll`, `turn`, `triplet_repeated`,
`double_stop`, `chord`, `trill`, `passing`.

| Paramètre | Bornes / effet audible | Mécanique | Vivant, point de départ |
|---|---|---|---|
| `rate_scale` | 0..10, multiplie tous les taux. Les transcriptions sous-notent : taux écrit = plancher | `1` | `4` aux sept profils, à valider à l'oreille |
| `repeat_variation` | 0..1 : probabilité qu'une mesure d'une passe ou d'un tour 2+ tire ses ornements indépendamment de sa source. À 0, les copies (`CopyOf`, `CopyOfPart`) et les reprises ont les mêmes ornements tant que la mélodie écrite est la même | `0` | `0` ; 0,3 pour un jeu qui ne se répète pas |
| `written_per_100_notes` | 0..100 pour 100 notes de la mélodie de la mesure | `0` | Corpus ABC (plancher écrit) |
| `per_100_notes_by_bar` | Jusqu'à 64 taux, mesure 1 de la partie en premier ; au-delà ou vide : `written_per_100_notes`. Une copie exacte emprunte la mesure de sa source | `[]` | Forme du corpus positionné (≥ 30 occurrences dans des parties de huit mesures), niveau ramené au taux global ; sinon taux global seul |
| `min_carrier_eighths` | 0..64 croches, durée **écrite** minimale de la porteuse (durée totale remplacée pour les figures) | `0` (un pas) | Minimum observé si peu d'occurrences ; sinon plus petite durée dont le taux par note atteint 10 % du meilleur |
| `arousal_low_multiplier`, `arousal_high_multiplier` | 0..4, taux × interpolation linéaire à l'arousal brut (0 → 1) | `1`, `1` | Neutres : le corpus n'annote pas l'arousal |
| `intervals` | `repeated`, `step` (≤ 2 demi-tons), `leap` : intervalle avec la note précédente de la mesure ; vide = tous ; la première note de la mesure est exclue dès que la liste n'est pas vide | `[]` | — |
| `beats` | `strong` (début de pulse du profil), `weak` | `[]` | — |
| `phrase_end` | `any`, `never`, `only` : dernière mesure de la phrase (`parts.phrase_ends`) | `any` | — |
| `velocity_scale` | 0,1..1, vélocité des notes ajoutées (appogiatures, voisines, seconde corde) | `0.8` | 0,7..0,9 |
| `span` | Appogiatures : durée totale avant la porteuse, 0,05..0,49 pas | `0.3` | 0,4 (simple), 0,49 (multiple) |
| `count` | `grace_multiple` : 2..4 notes | `2` | `2` |
| `strings` | `double_stop`, `chord` : 2..8 cordes à vide MIDI croissantes | requis | `[55,62,69,76]` (violon) ; type désactivé pour un fifre |
| `min_interval`, `max_interval` | `double_stop`, `chord` : demi-tons entre la porteuse et chaque note ajoutée | `3`, `12` | `chord` : `max_interval = 24` |
| `density_curve` | `passing` seulement : part de la mesure (0..1) qui reçoit la note de passage selon la densité | requis | `[[0,0],[0.5,0],[1,1]]` |

Réalisation, dans la durée jouée de la porteuse (millièmes de pas, ± 0,49 pas de la grille,
donc dans les bornes et le chemin du lecteur de l'interprétation) :

- **Appogiature(s)** : avant la porteuse, qui reste **sur la grille** ; elles prennent leur
  temps sur la note précédente de la mesure, écourtée d'autant (respiration composable), ou
  sur un silence ; si la note précédente n'a pas ≥ 0,25 pas à rendre, ou si elle est dans une
  autre mesure, l'ornement n'est pas posé (une mesure déjà publiée ne change pas). Hauteur :
  voisine diatonique du mode actif, dans la tessiture, différente de la note précédente.
- **Rouleau** : porteuse, voisine haute, porteuse, voisine basse, porteuse (36 / 16 / 16 / 16 /
  16 %). **Turn** : voisine haute, porteuse, voisine basse, porteuse. **Trille** : porteuse et
  voisine haute en alternance, nombre impair de notes (≥ 5, une note ≈ 0,2 pas).
- **Triolet de notes répétées** : trois notes égales de la porteuse, séparées par 10 % de silence.
- **Double corde / accord** : seconde (et troisième) note simultanée, même piste, même durée et
  même timing que la porteuse : corde à vide voisine de la corde de la porteuse, dans le mode,
  l'intervalle demandé et la tessiture ; la double corde essaie aussi une tierce diatonique
  sous la porteuse. Sans note jouable, la porteuse reste seule.
- **Note de passage** : inchangée (dernier pas de la tenue, voisine diatonique haute, sans tirage).
- Une note porte **au plus un** ornement ; la note de passage passe avant les types tirés ; une
  figure impossible (voisine hors mode ou tessiture) laisse la porteuse intacte. Les mesures
  d'intro et d'outro et les mesures protégées n'en reçoivent pas.

**Probabilité.** Pour chaque mesure, l'espérance du type vaut
`taux (par mesure) × rate_scale × multiplicateur d'arousal / 100 × notes de la mesure`, répartie
également sur les porteuses permises (durée écrite, contextes) ; une porteuse tire au plus une fois.
Le taux reste donc « pour 100 notes de la mélodie » quelle que soit la part de notes éligibles,
comme dans le corpus. Les tirages dérivent d'une graine propre à l'air, calculée **sans tirage**
depuis son matériel écrit : ni l'intro, ni les tirages suivants, ni la mélodie écrite ne changent
quand on active les ornements, et un même air porte les mêmes ornements quelle que soit la mesure
déjà générée (seek). Les tirages d'une note ne dépendent que de sa mesure écrite (hauteurs,
positions), pas de la durée jouée : à émotions constantes une reprise a les mêmes ornements ; si
les émotions changent entre deux passes, la même figure s'étire avec la durée jouée.

Mesures reproductibles : `specs/012-vie-realisme/measure_ornaments.py` (`corpus`, `rates`,
`wav`, `peaks`, `midi`).

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
| `parts.ornament_protected_bars` | Mesures sans aucun ornement (cadences), tous types de `[ornaments]` ; omission : dernière mesure ; `[]` : aucune | `[4,8]` | `[]` |
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

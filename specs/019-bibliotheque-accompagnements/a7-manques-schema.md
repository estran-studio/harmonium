# A7 — Manques de schéma relevés par les styles

Date : 2026-11-04. Branche `wquintal/a7-manques-schema`, partie de `main` à `3fb01da`. `TUNING_VERSION` 32 → **33**
(profils touchés : `gmd_rock`, `gmd_jazz_waltz`, et `tour_variants` dans huit autres `gmd_*`). Tout est optionnel en données :
un profil qui n'en déclare rien garde son flux de snapshots (les profils non listés sont identiques à `main`).
Documentation d'usage : `profiles/README.md`, « Manques de schéma comblés (phase 019, A7) ». Contrats :
`harmonium_core/tests/schema_gaps_tests.rs` (flux de `MeasureSnapshot`).

## Demande de l'orchestrateur (V2 + T1), traitée en premier

| Constat | Résultat |
|---|---|
| (1) Un `breaks.start` hors de la grille de l'utilisateur est refusé au chargement | **Corrigé.** `start` et `bars` ne sont plus comparés à la longueur de la forme à la validation (seuls `start ≠ 0` et `bars ≥ 1` restent exigés) ; `ArcBreak::window` borne déjà la fenêtre au chorus joué : un break qui commence au-delà de la dernière mesure est ignoré, un break trop long est ramené à la fin du chorus. Test `un_break_hors_du_chorus_n_est_pas_refuse`. Training n'a plus à retirer les breaks (9 mesures pour les latins/funk/afro-cubain, 24 pour valse et ballade). |
| (3) Imposer une variante de session ; `SetBpm` n'impose pas tempo + kit + arc du groupe `rythme` de `wjazzd_swing` | **Fait.** `EngineCommand::ForceSessionVariant { group, index }` et `ForceSessionVariantNamed { group, name }` (p. ex. `{ "rythme", "up_rapide" }`), `ClearSessionVariants` ; REPL `variant <groupe> <indice\|nom>`, `variant clear`. Le groupe garde la variante imposée quelle que soit la graine, **y compris tempo, kit et arc qu'elle réunit** ; les autres groupes gardent leur tirage (flux dérivés par groupe, vérifié sur 16 graines) ; rejeu depuis la mesure 1 ; indice ou nom inconnu, ou variante d'un autre groupe : refusé avec la liste des noms du groupe. Un `SetStyleProfile` retire les variantes imposées. Un `SetBpm` ensuite règle le tempo comme avant (la variante n'est plus tirée par la graine, donc kit et arc restent ceux du groupe). |
| (2) Profils de jeu refusés sous grille utilisateur | **Message clair seulement** : `accompaniment.bass.motifs[0].part: partie « Desert » absente de form.parts (une grille d'utilisateur remplace la forme : un motif ou gabarit rattaché à une partie nommée ne s'y applique pas ; retirer `part` pour qu'il joue sous toute grille)`. Repli générique **non fait** (voir plus bas). |

## Inventaire dédoublonné et priorité

Priorité : 1 = variété entre graines ou d'un chorus à l'autre (D6) ; 2 = fidélité au style ; 3 = le reste. « Coût » : S (un champ), M
(un mécanisme), L (refonte d'une couche).

| # | Manque | Cité par | Effet mesuré / audible | Prio | Coût | État |
|---|---|---|---|:-:|:-:|---|
| 1 | Motifs de basse sans étiquette ni arc ; plan fixé par air | B8 (2), B47, R2-styles (2), B3 (1), B6 (2), R2-jeux, V2 | 2–9 % de mesures de basse changées d'un chorus au suivant (21–53 % de paires identiques) | 1 | M | **fait** |
| 2 | Grille par défaut fixe par air, indifférente au mode tiré | B8 (4), B6 (4), B3 (3), R2-jeux | 0 % d'accords changés d'un chorus au suivant ; 4 à 11 suites d'accords distinctes sur 16 graines | 1 | M | **fait** (`tour_variants`, `modes`) |
| 3 | Commande de l'hôte pour imposer une variante | A2, B1 (1), T1, V2 | `SetBpm` n'impose pas le groupe `rythme` | 1 | M | **fait** |
| 4 | Break hors de la grille refusé | V2, T1 | profils de grille refusés sous 9 ou 24 mesures | 1 | S | **fait** |
| 5 | Niveau de densité de batterie non réglable par variante d'arc | B8 (6), R2-jeux | recherche à tâtons des multiplicateurs | 1 | S | **fait** (`percussion_density`) |
| 6 | Relance à une seule étiquette (recopie par étiquette : 24 entrées pour le rock) | B8 (5) | | 2 | S | **fait** (`tags` des relances et phrases) |
| 7 | Accords de puissance (qualité « 5 ») | B8 (1), R2-styles (7) | la guitare du rock joue des triades | 2 | M | **fait** |
| 8 | Blanche pointée en 3/4 (`bass_feel`) | B25 (1) | le deux temps joue blanche + noire | 2 | S | **fait** (`three_feel`, `bar_feel`) |
| 9 | `passing` non restreint au changement d'accord à la barre | B47, B3 (2) | plus d'approches que WJazzD n'en mesure | 2 | S | **fait** (`chord_change_at_bar`) |
| 10 | `tempo_offset` borné à ±30 BPM | B8 (8), B3 (14) | la plage de pop (83–142) n'est pas parcourable | 2 | S | **fait** (±120) |
| 11 | Coût de l'automation sur un export | vague 4 | 94 s → 169 s | 3 | — | **mesuré : aucun travail redondant** (voir plus bas) |
| 12 | Message d'erreur des profils de jeu sous grille | V2, T1 | refus opaque | 3 | S | **fait** (message) |
| 13 | Options de marche d'A4 (notes fantômes, arrivée, côté d'approche) exclusives des riffs et gabarits | R2-styles (1), B8 (3) | basse funk/rock/reggae sans notes fantômes probabilistes | 1 | L | non fait |
| 14 | Étiquette de batterie de l'arc devant celle de la session (le kit ne suit pas) ; kit par étiquette | B25 (2), B3 (6), R2-jeux | ballade : balais avec patterns de baguettes possibles | 1 | M | non fait |
| 15 | Groupes de session indépendants (balais à 180 BPM, kit « power » au tempo lent) ; couplage déclaré | R2-styles (3), B25 (4), B1 (3), B3 (12), R2-jeux, A2 | mariages douteux | 1 | M | non fait |
| 16 | Tonalité de la variante limitée au premier air ; `[fixed_tonality]` ou variante, pas les deux | A2, R2-styles (6) | sans objet pour un air par set | 3 | M | non fait |
| 17 | Micro-timing en fraction de pas, pas en ms | R2-styles (4), B6 (3), A6 (5), B3 (10) | dérive avec le tempo tiré (+24 % / −20 %) | 2 | M | non fait |
| 18 | `intensity` publiée sans l'arc | B47, B3 (11), V2 | un arc n'est pas vérifiable de l'extérieur | 3 | S | non fait |
| 19 | `jitter` de batterie et notes de passage partagent le tirage | B47, R2-styles (5) | `jitter` de batterie interdit sous `bass.passing` | 3 | M | non fait |
| 20 | Latin : relance sans côté de clave, `keep_steps` sans l'anticipation du pas 14, gabarit sans changement au temps 3, pas de `tie` sur le pas 14, relance remplaçant la mesure, clave de 3 mesures | B3 (5, 7, 8, 9, 11), A6 (4) | fidélité latine | 2 | L | non fait |
| 21 | Grille composée swinguée (12/8 sur 24 pas) | B6 (1) | shuffle lent en 12 pas seulement | 2 | L | non fait |
| 22 | Un seul instrument d'accords ; pas de piste ajoutée par une variante ; pas de `melody_program` par chorus lié à une contre-mélodie ; tag de pattern de pieds | B8 (9), R2-trad | orgue « bubble » + guitare, duo accordéon/violon | 2 | M | non fait |
| 23 | Mélodie jazz : rapprochement `chord_strong_scale_weak`, silences dans une cellule, liaison, doubles croches | E12 (1–4) | l'essentiel de la distance restante (0,323) | 2 | L | non fait (voie E) |
| 24 | Mélodie E10/E11 : jonction d'airs, filtre variable en cours d'air, ornements sous bornes, alternance par phrase | E10-E11 | | 3 | L | non fait (voie E) |
| 25 | `[mix]` non surchargeable par variante ; zones Odin d'une variante sans `level_db` | B1 (4), R2-jeux | niveaux non recalés par timbre | 3 | M | non fait |
| 26 | MusicXML `divisions = 4` et champ `beat` supposent 4 pas par noire ; swing qui ne suit pas un `SetTempo` audible | A6 (1, 2) | faux sur 12 pas en 4/4 | 3 | M | non fait |
| 27 | Seek : +16 ms sur un profil à phrases, arc et variantes | B1 (5) | mesuré plus bas | 3 | — | mesuré |
| 28 | Tirage de session à la première mesure, programmes à la barre, BPM entier, registre borné à une octave, courbe de tempo plate refusée, `tempo_offset` non écrêté aux bornes du profil | A2, B3 (14) | | 3 | S–M | non fait |

### Pourquoi ces choix, et pourquoi pas le reste

Les neuf premiers sont des champs ou des mécanismes génériques de quelques dizaines de lignes, chacun testé sur le flux de
snapshots. Les non faits les plus utiles à D6 sont **13** (une couche de basse probabiliste sur les riffs : demande de
rendre `walking.eighths`, `arrival` et `approach.side` applicables aux motifs et gabarits, donc de réécrire la réalisation de
`bass_motif_bar`/`bass_template_bar` autour de la même structure de notes), **14** (une étiquette de batterie à plusieurs
dimensions : les patterns n'ont qu'un `tag` et une désérialisation personnalisée ; la voie la plus courte est `tags` sur les
patterns puis une règle « l'arc restreint la session » en intersection, comme les relances le permettent maintenant) et **15**
(un `conditions` par variante de session ; touche le tirage de groupe, donc tous les profils habillés). Ils sont à reprendre
dans cet ordre. Le repli générique pour les profils de jeu sous grille (constat 2) demande en plus que les motifs de basse
tolèrent une partie absente : la garde « partie inconnue » est utile à l'écriture des profils, il faudrait la distinguer de
l'installation d'une grille d'utilisateur.

## Ce qui est livré

| Mécanisme | Où | Contrat |
|---|---|---|
| `bass.motifs[].tag`, `templates[].tag` ; `bass = [{ tag, weight }]` dans l'arc et la session ; plan de basse tiré **à chaque chorus** sous des tags | `accomp.rs` (`draw_bass_motif_tagged`), `generator.rs` (`bass_plan_for`), `arc.rs`, `session.rs` | chorus 1 : seuls les motifs `sparse` ; chorus suivants : `dense` ; chorus de mêmes tags : tirages distincts (≥ 6 graines sur 8) ; sans tags : plan de l'air rejoué ; seek identique ; tag inconnu refusé avec le chemin |
| `harmony.tour_variants` (1..16) ; `loops[].modes`, `generated[].modes` | `chord_loop.rs`, `style_profile.rs` | un tirage : accords identiques à chaque chorus ; deux : chorus 1 = tirage de l'air, 3 = 1, 2 = 4 ; boucle dorienne jamais jouée en ionien |
| Qualité `5` (`ChordType::Power`) | `theory/chord.rs`, `quality.rs`, `accomp.rs`, `voicing.rs` | accord publié `5`, deux classes de hauteur, comping dans ces classes ; jamais déduit d'un ensemble de notes (`Chord::from_notes` inchangé) ; `fixed_triad` la refuse |
| `bass_feel` `three_feel`, `bar_feel` | `arc.rs` | au plus une attaque par mesure (+ arrivées), moins qu'à la marche |
| `percussion_density` (arc, session) | `arc.rs`, `session.rs`, `generator.rs` | le niveau de l'arc remplace celui de l'hôte (0 et 2 donnent des batteries différentes, hôte à 0 ou 2 identique) ; > 2 refusé |
| `fills[].tags`, `phrases[].tags` | `percussion.rs`, `arc.rs` | `tag = x` ≡ `tags = [x]` (batterie identique sur 3 graines) ; tag connu par `tags` seul accepté |
| `passing.rules[].when = "chord_change_at_bar"` | `bass_passing.rs` | approche au pas 14 seulement si l'accord de la mesure suivante diffère |
| `tempo_offset` ±120 | `dynamics.rs` | écart de 120 BPM entre les extrémités |
| Variante de session imposée (indice ou nom) | `session.rs` (`draw_forced`), `generator.rs`, `composer.rs`, `live_host.rs`, CLI | voir plus haut |
| Break hors chorus | `arc.rs` | voir plus haut |

## Activation dans les profils (`TUNING_VERSION` 33)

| Profil | Mécanismes |
|---|---|
| `gmd_rock` | `tour_variants = 4` ; huit boucles de plus en accords de puissance (mêmes progressions) ; motifs et gabarit de basse étiquetés `sparse` / `medium` / `dense` et `bass` dans l'arc ; `percussion_density` 1 (exposition) et 0 (retombée) à la place des multiplicateurs trouvés par essais |
| `gmd_jazz_waltz` | `bass_feel = "three_feel"` à l'exposition et à la retombée (arc et arc nommé de la session) |
| `gmd_pop`, `gmd_hiphop`, `gmd_soul`, `gmd_funk`, `gmd_dance_disco`, `gmd_afrobeat`, `gmd_reggae`, `gmd_neworleans_funk_secondline` | `tour_variants = 4` |

Non activé, avec raison : les profils de jazz (`wjazzd_*`, `gmd_jazz_waltz`, blues, latins) rejouent les mêmes changes à chaque chorus,
c'est le genre ; les profils de jeu ont une contrainte de distance au corpus que je n'ai pas le temps de revérifier.

## Tableau de variété (`seed-variety --chorus 4`, 16 graines, avant → après)

Même binaire (release), profil de `main` lu depuis une archive git contre le profil de la branche. Colonnes : mesures changées
d'un chorus au suivant (chorus ≥ 3), pour les accords, la basse et la batterie ; suites d'accords distinctes sur 16 graines ;
chorus distincts par graine (accompagnement entier).

| Profil | Accords changés | Basse changée | Batterie changée | Suites d'accords distinctes | Chorus distincts / 4 |
|---|---:|---:|---:|---:|---:|
| `gmd_rock` | 0 → 81 % | 6 → 72 % | 100 → 100 % | 9 → 16 | 4,0 → 4,0 |
| `gmd_pop` | 0 → 54 % | 7 → 58 % | 100 → 100 % | 11 → 16 | 4,0 → 4,0 |
| `gmd_hiphop` | 0 → 65 % | 5 → 66 % | 100 → 100 % | 10 → 16 | 4,0 → 4,0 |
| `gmd_soul` | 0 → 58 % | 7 → 59 % | 100 → 100 % | 8 → 16 | 4,0 → 4,0 |
| `gmd_funk` | 0 → 70 % | 9 → 43 % | 97 → 97 % | 5 → 16 | 4,0 → 4,0 |
| `gmd_dance_disco` | 0 → 59 % | 7 → 61 % | 100 → 100 % | 8 → 16 | 4,0 → 4,0 |
| `gmd_afrobeat` | 0 → 50 % | 7 → 48 % | 100 → 100 % | 8 → 16 | 4,0 → 4,0 |
| `gmd_reggae` | 0 → 55 % | 7 → 57 % | 100 → 100 % | 8 → 16 | 4,0 → 4,0 |
| `gmd_neworleans_funk_secondline` | 0 → 40 % | 42 → 61 % | 99 → 99 % | 4 → 15 | 4,0 → 4,0 |
| `gmd_jazz_waltz` (`three_feel`) | 0 → 0 % | 100 → 100 % | 100 → 100 % | 6 → 6 | 4,0 → 4,0 |

Lecture honnête : la basse des huit profils à riff suit la fondamentale des accords, son gain vient donc surtout de la grille
par chorus ; l'effet propre des motifs étiquetés n'est isolé que dans `gmd_rock`, où la basse change de 72 % (grille et tags ensemble, non séparés : [INFERENCE] la
basse du rock suit les accords, donc l'essentiel vient de la grille). Les habillages distincts
sur 16 graines (tempo, tonalité, programmes, voicings, kit) ne bougent pas : A7 ne touche à aucun `session.toml` de ces profils
(15 à 16 habillages distincts, R2-styles). Distance au corpus : les couches accords, basse, comping de ces profils n'ont pas de corpus
(règles de métier) ; la batterie, seule mesurée au GMD, n'est pas modifiée sauf le niveau d'exposition et de retombée du rock
(`percussion_density` remplace les multiplicateurs : la distance n'a **pas** été remesurée, à faire par V2).

## Coût de l'automation sur un export

Mesure (debug, `gmd_rock`, graine 3, 200 mesures, automation constante `intensity = 0.5`) : 105 s sans automation, 179 s avec.
Phases instrumentées temporairement (130 mesures) : génération 1,6 s, rendu audio 148 s (avec) contre 84 s (sans), 129 appels
d'`apply`, **aucun seek déterministe** (journal). Le MIDI explique l'écart : l'export sans automation appelle
`set_end_after_tour(true)` (« l'export fixe conserve son outro historique »), l'air de 64 mesures finit et les 136 mesures
restantes sont silencieuses (dernier son à la mesure 64, 2 500 notes) ; l'export avec automation joue 200 mesures (7 600 notes).
Le coût supplémentaire est du rendu SoundFont de musique réellement jouée, pas du travail redondant : rien à corriger dans le
moteur ni dans le CLI. La génération pèse 1 % du temps d'un export. (Le test `export_terminates` compare donc des durées de
musique différentes ; son plafond de 900 s reste valide.)

## Durée du seek

`deterministic_seek(64)`, minimum de dix essais, debug, `ovh-inner` chargé (trois agents) : profil par défaut 83 ms ; `gmd_rock`
**avant 132 ms, après 122 ms** (le plan de basse par chorus ne se mesure pas dans le bruit) ; `gmd_jazz_waltz` 154 ms. Le seuil de
50 ms de `seek_to_bar_64_under_50ms` échoue ici comme avant A7 (88 ms constatés par la vague 4) et passe sur le Mac (décision du
propriétaire : seuil inchangé).

## Vérifications

Voir le dernier message pour les résultats des suites (workspace, Odin, Bevy, studio, déterminisme natif = wasm, fmt, clippy).

## Ce qui reste (phase suivante)

Les lignes « non fait » du tableau, dans l'ordre : 13, 14, 15, 17, 20, 22, puis le reste. Ajouter aussi : revérifier la distance
de la batterie du rock (`percussion_density`), et décider avec training si les profils de jeu doivent perdre leurs `part` de basse
pour jouer sous grille.

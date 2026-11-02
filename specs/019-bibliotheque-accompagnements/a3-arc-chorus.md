# A3 — Arc sur plusieurs chorus

Date : 2026-11-02. Branche `wquintal/a3-arc-chorus`, partie de `main` à `63e26da` (`TUNING_VERSION` 24,
**inchangé**). Mécanisme optionnel en données : sans `[arc]`, rien ne change (dumps, WAV et goldens
identiques, preuves plus bas).

## Ce qui est livré

- **Numéro de chorus** lu dans `TunePosition` (déjà dans `MeasureSnapshot.position`), sans nouveau
  champ sérialisé : `chorus()` et `choruses()`.
- **Section `[arc]`** (ou `arc.toml`, `[files] arc = "arc.toml"`), optionnelle, validée au chargement
  avec le chemin du champ (et fichier et ligne quand le profil vient de fichiers) : étapes de
  chorus, variantes tirées par graine, retombée finale, breaks et stop-time.
- Deux ajouts génériques au schéma existant, nécessaires à l'arc : le feel de basse **deux temps**
  (walking) et les **tags** de motifs de comping et de patterns de batterie.
- Deux fixtures sans Odin (`grille_arc_chorus`, `grille_arc_chorus_aaba`), 19 tests de contrat.

## Numéro de chorus (principe VI, pas de seconde convention)

Un chorus est une passe complète de la forme : `TunePosition::tour` (1-based, `tours` par air), déjà
publié. Un set compte plusieurs airs (`tune_in_set`, `tunes_in_set`, déjà publiés aussi) ; le chorus
du set est donc dérivé, sans champ de plus :

```text
chorus()   = (tune_in_set - 1) × tours + tour        (0 en intro)
choruses() = tunes_in_set × tours                    (None si tunes_in_set == 0 : set sans fin)
```

- **Nombre de chorus connu (fourni)** : `set.length = [{ tunes = N }]` et
  `arrangement.tours_per_tune` (1..=16) ; training règle les deux. Un set d'un air de 8 chorus,
  c'est `choruses() == Some(8)`.
- **Infini** : `set.length` omis (set sans fin). `chorus()` continue de compter les airs de la
  session, `choruses()` est `None` : jamais de retombée ; les étapes se tiennent ou boucle
  (`repeat_from`).
- Un profil `loop` sous grille utilisateur (une partie de N mesures) est le cas simple : chorus = tour
  = N mesures. Une forme à plusieurs parties (AABA) donne un chorus de 32 mesures
  (`grille_arc_chorus_aaba`).

Je n'ai **pas** ajouté de champ `chorus` à `MeasureSnapshot` : ce serait une rupture de sérialisation
(goldens, dumps, JSON de référence) pour une valeur qui se lit déjà dans quatre champs publiés.

## Schéma

Détail et exemple dans `profiles/README.md`, « Arc sur plusieurs chorus ».

| Champ | Bornes | Effet |
|---|---|---|
| `[[stages]]` | 1..=16 | étapes dans l'ordre des chorus ; `choruses` (1..=1024, défaut 1) chorus consécutifs chacune |
| `stages[].variants` | 1..=16 | chaque chorus de l'étape tire **sa** variante par poids |
| `repeat_from` | indice 0-based d'étape | après la dernière étape, reprise en boucle ; absent : la dernière étape se tient |
| `[[fall]]` | 0..=16 variantes | variantes du dernier chorus du set, si le nombre de chorus est connu et ≥ 2 |
| variante : `weight` | ≥ 1 | poids du tirage |
| `intensity` | 0..=2, défaut 1 | multiplicateur de l'intensité lue par les courbes de `dynamics` (vélocités par piste, motif de comping par intensité, niveau de densité de la batterie), borné à 0..1 ; pas le tempo |
| `density` | 0..=2, défaut 1 | multiplicateur de la densité rythmique lue par `density_percussion`, `density_chords` et les notes de passage de la basse |
| `bass_feel` | `walking` / `two_feel` | exige `accompaniment.bass.walking` |
| `comping`, `percussion` | 0..=8 `{ tag, weight }` | restreignent le tirage aux motifs ou patterns qui portent un tag nommé |
| `breaks` | 0..=8 | `{ start, bars, chance, tracks, keep_steps }` : fenêtre de mesures du chorus (`start` 1-based, négatif depuis la fin), probabilité par chorus, pistes touchées (vide : toutes), pas gardés (vide : silence, `[0]` : stop-time) |

Erreurs : chemin du champ (`arc.stages[1].variants[0].breaks[0].start`), contre le profil (tag absent
de la banque, feel sans marche, fenêtre hors du chorus, pas hors mesure, `repeat_from` inconnu…).
Champ inconnu : refusé (`deny_unknown_fields`).

Données ajoutées hors de `[arc]` (optionnelles, sans effet seules, sérialisation inchangée si absentes) :
`accompaniment.chords.comping[].tag` et `percussion.patterns[].tag`.

## Algorithme

Module `compose/arc.rs`, générique : aucun `match` sur un style, aucune valeur musicale.

1. **Étape du chorus** `c` : on parcourt les étapes en retirant `choruses` ; passé la dernière, la
   dernière étape se tient, ou, avec `repeat_from`, `reste mod (somme des chorus de l'étape de
   reprise à la fin)` reparcourt les étapes depuis la reprise. Forme fermée, sans état.
2. **Variante** : si le nombre de chorus est connu, au moins 2 et `c` en est le dernier, tirage
   pondéré dans `fall` ; sinon dans les variantes de l'étape. Flux ChaCha8 amorcé par
   `mix(mix(graine, VARIANT|FALL), c)`, un entier `u64` par tirage.
3. **Graine de l'arc** : deux tirages de 32 bits dans le RNG injecté à la construction de chaque air,
   **seulement si le profil a un arc** (sinon le flux est celui d'avant). Celle du **premier air du
   set** sert à tout le set, donc un arc qui traverse plusieurs airs reste cohérent.
4. **Break** : pour chaque break de la variante dont la fenêtre contient la mesure, un flux dérivé
   `mix(mix(mix(graine, BREAK), c), indice)` tire `chance` ; les attaques des pistes touchées qui ne
   démarrent pas à un pas de `keep_steps` sont retirées en fin de génération de la mesure. Les
   accords publiés ne changent pas.
5. **Application par mesure** (`generate_measure`) : l'arc ne touche ni l'intro ni la mesure de clôture.
   - `intensity` et `density` remplacent l'intensité et la densité lues par `apply_dynamics`,
     la densité de batterie, le décalage de comping, les notes de passage. Le tempo garde l'intensité
     de l'air.
   - `bass_feel` règle `WalkingBar::attack_pulses` (1 : marche, 2 : deux temps).
   - `comping` passe les tags à `ChordVoice::bar_hits` : le tirage garde la graine de la mesure, avec
     pour poids (poids du motif × poids du tag), les motifs sans tag nommé exclus ; aucun motif ne
     concourt : tirage du profil.
   - `percussion` : `PercussionTable::tagged_pattern` tire parmi les patterns de la densité de la mesure
     (et de sa partie) qui portent un tag nommé, avec un flux par chorus, partie et densité ; aucun :
     choix ordinaire ; une mesure de relance (`[[cadence_patterns]]`) garde son pattern.

**Deux temps** (`walking_bar`). La ligne de marche est calculée sur les **attaques** et non plus sur les
pulsations : une attaque toutes les `attack_pulses` pulsations, plus une à chaque changement d'accord.
L'arrivée est celle de la marche (basse publiée, registre `low..high`, accent). L'attaque suivante dans
l'étendue d'un accord est l'**approche** de la cible suivante (tirage `approach` du profil, sinon une
note de passage qui la rend atteignable). Chaque note est tenue jusqu'à l'attaque suivante (blanche aux
temps 1 et 3). Avec une attaque par pulsation le calcul est celui d'avant, tirage par tirage (dumps
identiques). Les enrichissements (notes répétées, croches de passage, chromatisme, autres arrivées) sont
pour A4 : rien n'en est fait.

## Seek

L'état d'une mesure ne dépend que de son indice, de sa position et de la graine de l'arc, comme
`walking.rs` : variante, break, motif de comping et pattern de batterie se tirent par flux dérivés,
jamais d'une mesure précédente. Test : un générateur remis à zéro puis avancé en silence jusqu'à la
mesure 31 rejoue les mesures 31 à 48 à l'identique du flux linéaire.

Durée de `seek_to_bar_64_under_50ms` (`deterministic_seek(64)`, graine 42, `reel_quebecois`, build de
test, local, 3 passages de 25 seeks, minimum et médiane ; le test ne change pas, son seuil non plus) :

| | main (`63e26da`) | branche |
|---|---|---|
| minimum | 25,9 à 26,2 ms | 26,0 à 26,7 ms |
| médiane du meilleur passage | 26,6 ms | 26,9 ms |

Les médianes des passages bruités montent à 30–60 ms sur les deux arbres quand la machine est occupée
(compilations en parallèle) ; les minimums sont stables. Rejeu de 64 mesures du générateur seul
(`silent_advance(65)`, 30 passages, minimum) :

| profil | main | branche |
|---|---|---|
| `reel_quebecois` | 25,3 ms | 25,0 ms |
| `wjazzd_swing` | 21,4 ms | 21,2–21,3 ms |
| `grille_arc_chorus` (avec arc) | — | 21,7–22,1 ms |

Un arc coûte ≈ 0,5 ms sur 64 mesures (un flux ChaCha8 par mesure pour la variante, un par break de la
fenêtre). Rien n'est ajouté pour les profils sans arc. Je n'ai pas accéléré le chemin de seek : la
question de la CI (55–57 ms) reste celle de la reprise de 018, ce travail ne l'alourdit pas.

## Fixtures

`tests/fixtures/profiles/grille_arc_chorus/` : copie de `wjazzd_swing` réduite au blues de douze mesures
(quatre chorus, un air par set : nombre de chorus connu), `percussion.toml` de `wjazzd_swing` avec ses
patterns étiquetés `ride` et trois patterns `hat` ajoutés (charleston seul, densités 0 à 2), comping
étiqueté `sparse` / `medium` / `dense`, vélocités de `dynamics` élargies et `density_percussion` à
`-1..+1` pour que l'arc s'entende. `grille_arc_chorus_aaba/` : même chose sur la forme AABA (32
mesures, trois chorus). `profiles/wjazzd_swing` n'est pas touché (sa calibration viendra en B1).
Arc d'exercice (`arc.toml`, valeurs de service, aucune mesure de corpus) :

| Chorus | Basse | Comping | Batterie | Intensité, densité | Autre |
|---|---|---|---|---|---|
| 1 | deux temps | `sparse` (3) ou `medium` (1) | charleston | 0,6 · 0,5 ou 0,7 · 0,6 | |
| 2–3 (corps) | marche | `medium` / `dense` | ride | 1 · 1, ou 1,2 · 1,2 (1 chance sur 3) | la variante forte a un break d'une mesure à 50 % (accords et basse) |
| 4 (retombée, ou dernier) | deux temps | `sparse` | charleston | 0,5 · 0,5 | stop-time sur les deux dernières mesures (accords et basse, premier temps) |
| au-delà (set sans fin ou plus de chorus) | sommet (marche, dense, ride, 1,4 · 1,4, stop-time à 50 %), puis corps et sommet en boucle | | | | |

## Tests

`harmonium_core/tests/arc_chorus_tests.rs`, 19 tests sur le flux de `MeasureSnapshot` (graines 1, 2, 3, 17
et 1 à 16) :

| Contrat | Test |
|---|---|
| numéro de chorus, nombre connu, set sans fin, set de deux airs | `le_numero_de_chorus_se_lit_dans_la_position_du_flux` |
| étapes appliquées, mesurées par chorus : feel de basse (attaques, durées, arrivée sur la basse publiée à chaque changement) | `le_feel_de_basse_bascule_…`, `le_deux_temps_arrive_sur_la_basse_…` |
| comping clairsemé → dense ; batterie charleston → ride et densité ; intensité (vélocités) sans toucher au tempo | `le_comping_passe_du_clairseme_au_dense_…`, `la_batterie_change_de_pattern_…`, `l_intensite_de_l_arc_…` |
| breaks aux positions données (début et depuis la fin), stop-time, break tiré par chorus et par graine | `un_break_se_joue_aux_mesures_donnees_…`, `un_stop_time_ne_garde_que_…`, `un_break_tire_au_sort_…` |
| retombée au dernier chorus du set (d'un air, de deux airs), jamais sans fin connue ni avec un seul chorus | `la_retombee_joue_au_dernier_chorus_…`, `sans_fin_connue_ou_avec_un_seul_chorus_…` |
| étapes de plusieurs chorus, étape qui se tient, `repeat_from` | `les_etapes_se_tiennent_ou_reprennent_…` |
| même graine, même sortie ; graines différentes, arcs différents (≥ 4 arcs sur 16 graines) ; chorus d'une graine différents ; rejeu après seek | `meme_graine_meme_sortie_…`, `une_mesure_se_rejoue_a_l_identique_…` |
| forme à plusieurs parties : chorus de 32 mesures, parties lues dans la position | `une_forme_a_plusieurs_parties_…` |
| erreurs de schéma avec le chemin du champ (19 cas), feel sans marche, erreur située dans `arc.toml` avec sa ligne, aller-retour TOML | `les_erreurs_de_schema_…`, `le_feel_de_basse_exige_…`, `une_erreur_de_l_arc_est_situee_…`, `l_arc_survit_…` |

Mutations vérifiées (chacune fait échouer au moins un test) : retombée décalée d'un chorus, fenêtre de
break depuis la fin décalée d'une mesure, `repeat_from` ignoré, graine de l'arc constante.

## Preuves

| Vérification | Résultat |
|---|---|
| `harmonium_determinism`, binaire de `main` (`git archive` de `63e26da`, cible séparée) contre la branche | 98 fichiers identiques octet pour octet (aucun profil distribué n'a d'arc ; `wjazzd_swing` passe par le `walking_bar` généralisé) |
| WAV `--export --seed 1 --bars 44` (CLI lancé depuis `~/Project/bascanada/harmonium`) de `reel_quebecois`, `valse_quebecoise`, `gigue_quebecoise`, `isaac_cellar`, `nt_desert`, `wjazzd_swing`, binaires `main` et branche | `cmp` identiques |
| `TUNING_VERSION`, goldens, `profile_d2_main.json` | inchangés (aucune rupture) |
| `cargo test --workspace -q --no-fail-fast` | 806 réussis, 0 échec, 2 ignorés (dont les 19 de `arc_chorus_tests`) |
| `cargo test -p harmonium --features odin`, `cargo test -p harmonium_bevy` | 118 réussis ; 11 réussis |
| `cargo check -p harmonium_studio` ; `cargo check --target wasm32-unknown-unknown` (`harmonium_core`, `harmonium_audio`, `harmonium`, avec et sans `odin` ; `harmonium_bevy --no-default-features`, avec et sans `odin`) | passent |
| `harmonium_determinism/scripts/run.sh` (wasm-bindgen 0.2.108) | natif = wasm : 98 / 98 |
| `cargo clippy -p harmonium_core -p harmonium -p harmonium-cli --all-targets` | aucun avertissement dans `arc.rs`, `walking.rs` ni `arc_chorus_tests.rs` ; les avertissements `unwrap` / `panic` des modules de test existants sont inchangés |
| `cargo +nightly fmt --all` | propre |

## Écoute (propriétaire)

Rendus (graines 1 et 4, `--export --record-wav`, CLI de la branche lancé depuis
`~/Project/bascanada/harmonium` pour `assets/`), WAV dans `/tmp/harmonium-019/a3/ecoute/`, MP3 320 k dans
`~/Project/bascanada/harmonium_lab/corpus/ecoute_019/a3/` :

```sh
cd ~/Project/bascanada/harmonium && CLI=~/orca/workspaces/harmonium/a3-arc-chorus/target/release/harmonium-cli \
  FX=~/orca/workspaces/harmonium/a3-arc-chorus/tests/fixtures/profiles
$CLI --export --seed 1 --bars 48 --profile $FX/grille_arc_chorus      --record-wav /tmp/harmonium-019/a3/ecoute/blues_s1.wav
$CLI --export --seed 4 --bars 48 --profile $FX/grille_arc_chorus      --record-wav /tmp/harmonium-019/a3/ecoute/blues_s4.wav
$CLI --export --seed 1 --bars 96 --profile $FX/grille_arc_chorus_aaba --record-wav /tmp/harmonium-019/a3/ecoute/aaba_s1.wav
$CLI --export --seed 4 --bars 96 --profile $FX/grille_arc_chorus_aaba --record-wav /tmp/harmonium-019/a3/ecoute/aaba_s4.wav
```

Fichiers MP3 : `blues_s1.mp3`, `blues_s4.mp3` (4 chorus), `aaba_s1.mp3`, `aaba_s4.mp3` (3 chorus), et les
pistes de `blues_s1` en solo (`_Bass`, `_Chords`, `_PercussionHigh`). 150 noires/min : une mesure dure
1,6 s. L'export ferme la dernière mesure sur une tonique (son historique, hors arc) : le stop-time de la
retombée s'entend sur l'avant-dernière mesure, la dernière est la clôture.

**Blues, 4 chorus (12 mesures = 19,2 s chacun)** :

| Moment | Mesures | À écouter |
|---|---|---|
| 0:00 à 0:19,2 | 1 à 12 | chorus 1, exposition : basse en blanches (temps 1 et 3), comping clairsemé (une ou deux attaques), charleston seul, doux |
| 0:19,2 à 0:38,4 | 13 à 24 | chorus 2 : la marche arrive (une note par temps), ride et pédale, comping à trois attaques, plus fort |
| 0:38,4 à 0:57,6 | 25 à 36 | chorus 3 : même étape, autre variante (graine 1 : ride simple, vélocité plus basse que le chorus 2 ; graine 4 : le plus dense, 8 coups de batterie par mesure) |
| 0:57,6 à 1:13,6 | 37 à 46 | chorus 4, retombée : retour aux blanches, comping d'une attaque, charleston, doux |
| 1:13,6 à 1:15,2 | 47 | stop-time : accords et basse sur le premier temps seulement, charleston continue |
| 1:15,2 à 1:16,8 | 48 | clôture de l'export (tonique) |
| graine 4 : 0:36,8 à 0:38,4 | 24 | break : accords et basse se taisent, la batterie continue, juste avant le chorus 3 |

**AABA, 3 chorus (32 mesures = 51,2 s chacun ; A 12,8 s, B 12,8 s)** :

| Moment | Mesures | À écouter |
|---|---|---|
| 0:00 à 0:51,2 | 1 à 32 | chorus 1 : deux accords par mesure, une arrivée de basse sur chacun, blanches, charleston ; B (0:25,6 à 0:38,4) : un accord par deux mesures |
| 0:51,2 à 1:42,4 | 33 à 64 | chorus 2 : marche, ride, comping plus dense ; graine 4 : break à la mesure 64 (1:40,8 à 1:42,4) |
| 1:42,4 à 2:30,4 | 65 à 94 | chorus 3, retombée : blanches, comping d'une attaque, charleston |
| 2:30,4 à 2:32 | 95 | stop-time (premier temps seulement) ; 2:32 à 2:33,6 : clôture |

Mixage (LUFS intégrés en solo, graine 1, mesures 1 à 47 du blues et 1 à 95 de l'AABA, sans la mesure de
clôture, dont la tonique à vélocité 110 fausse l'intégré) :

| Piste | Blues | AABA |
|---|---|---|
| Bass | −24,5 | −24,8 |
| Chords | −25,2 | −23,5 |
| PercussionHigh (charleston, ride, pédale) | −31,3 | −32,6 |
| PercussionLow (grosse caisse) | silence | silence |
| mix complet | −22,0 | −21,6 |

La grosse caisse ne joue qu'à la densité 2, que ces arcs n'atteignent pas. Le mix complet par chorus
(sans la clôture) montre l'arc : blues graine 1 −24,9 / −20,2 / −20,8 / −25,7 LUFS, graine 4 −23,4 /
−20,5 / −20,1 / −25,7 ; AABA graine 1 −23,3 / −19,3 / −25,1, graine 4 −22,7 / −19,4 / −25,1 (amplitude
d'environ 5 LU entre l'exposition ou la retombée et le corps).

**À juger** : (1) si le deux temps tient (la seconde blanche est une approche ou une note de passage,
pas la quinte d'un deux-temps classique : voir limites) ; (2) le contraste exposition / corps /
retombée (valeurs de service, jamais mesurées) ; (3) le break et le stop-time (la queue des accords
tenus se coupe net : les notes gardées sont celles du premier temps) ; (4) la lisibilité du charleston
seul au chorus 1 (un seul son de batterie, sans pédale).

## Limites

- **Valeurs d'exercice** : l'arc des fixtures n'est calibré sur aucun corpus ni écoute ; `wjazzd_swing`
  n'a pas d'arc (B1). Deux variantes par étape : peu de variété entre graines (c'est de la donnée, pas
  du moteur).
- **Comping et graine de session (constat V0, non corrigé ici)** : le comping et la marche de basse se
  tirent avec la graine de matériel de l'air (`material_seed`, dérivée de la mélodie écrite), qui varie
  peu d'une graine de session à l'autre quand la banque de cellules est minimale et la grille fixe.
  L'arc, lui, a sa propre graine tirée dans le RNG injecté : ses variantes, ses breaks et ses patterns
  de batterie suivent la graine de session. Mesuré sur la fixture, 16 graines × 48 mesures :
  vélocité des arrivées par mesure (intensité tirée par l'arc) 8 séquences distinctes, comping 12,
  batterie 7, hauteurs de basse 6. Le test « graines différentes → arcs différents » ne lit que
  l'arc (feel, charleston ou ride, vélocité d'arrivée, break), pas le comping ni la basse. Le comping
  tiré sous un tag reste, à tag égal, aussi peu varié d'une graine à l'autre que celui du profil : la
  variété du comping d'un arc est limitée par cette dépendance, à traiter dans la tâche séparée.
- **Deux temps** : la seconde attaque d'une étendue est une approche ou une note de passage (poids du
  profil), jamais la quinte ; notes répétées, croches de passage, chromatisme, arrivées autres que la
  basse publiée : A4. « Une attaque toutes les deux pulsations » n'est pas un deux-temps en 3/4 ou en
  composé (A6).
- **Densité** : le multiplicateur n'ouvre ni ne ferme une piste (`density_tracks` lit la densité du
  jeu) ; l'intensité ne change pas le tempo.
- **Retombée** : jouée au dernier chorus du set quand le nombre de chorus est connu et ≥ 2. La demande
  de fin du jeu (`end_after_tour`) n'est pas une fin connue : sans `set.length`, pas de retombée. Le
  nombre de chorus d'un air est limité à 16 (`tours_per_tune`) ; un arc plus long passe par plusieurs
  airs d'un set, dont les airs suivants le premier rejouent un matériel neuf (intro réduite à la levée).
- **Tags** : le pattern de batterie étiqueté ignore sa portée `tour` (il respecte `part`) ; un pattern
  sans densité correspondante laisse le choix ordinaire. Les motifs de comping sont tirés par mesure,
  les patterns de batterie par chorus et par partie (pas de phrase sur plusieurs mesures : A1).
- **Break** : retire des attaques après coup (la queue d'une note d'accord déjà générée est coupée avec
  elle) ; les identifiants de notes ont des trous. L'harmonie et les accords publiés ne changent pas.
- **Graine** : un nouveau set (après la fin du set courant) tire un nouvel arc. Le harnais natif/wasm ne
  couvre que les profils distribués : les fixtures n'y sont pas ; le chemin de l'arc n'a pas de code
  spécifique à wasm (flux ChaCha8 et entiers `u64`).
- **Temps réel** : le calcul de l'arc est dans le compositeur, pas dans le callback ; aucune
  allocation ajoutée au chemin audio. `walking_bar` alloue deux `Vec` par mesure comme avant (hors
  callback).

## Fichiers

Moteur : `harmonium_core/src/compose/arc.rs` (nouveau), `walking.rs`, `generator.rs`,
`accomp_profile.rs`, `percussion.rs`, `arrangement.rs`, `mod.rs`, `style_profile.rs`. Tests :
`harmonium_core/tests/arc_chorus_tests.rs`. Données : `tests/fixtures/profiles/grille_arc_chorus/`,
`grille_arc_chorus_aaba/`. Docs : `profiles/README.md`, ce compte rendu, `plan.md`.

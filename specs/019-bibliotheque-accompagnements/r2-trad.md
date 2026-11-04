# R2-trad — Variété qui s'entend dans les dix profils trad (phase 019, vague 4)

Date : 2026-11-04. Branche `wquintal/r2-trad`, partie de `main` à `3b90e67`. **`TUNING_VERSION` 31** (rupture assumée :
principe I ; la graine tire maintenant une variante de session, donc le flux du RNG et les notes changent pour ces dix profils).
Compilateur : `rustc 1.101.0-nightly (0abfedbc7 2026-10-02)` (override d'`ovh-inner`) ; tout a été compilé et mesuré avec lui.
Aucune ligne de moteur : données (dix `session.toml`), un champ de `tuning.rs`, un test de contrat, quatre tests d'hôte ajustés,
un golden, le générateur et les scripts de mesure.

## Ce qui est livré

Pour `reel_quebecois`, `reel_acadien`, `reel_britannique_coloniale`, `gigue_quebecoise`, `gigue_acadienne`,
`gigue_britannique_coloniale`, `hornpipe_britannique_coloniale`, `marche_britannique_coloniale`, `valse_quebecoise`,
`valse_britannique_coloniale` : un `session.toml` (`[files] session`), **GÉNÉRÉ** par `r2-trad/habillage.py`, de 10 à 11 groupes tirés
par graine :

- `tempo` : trois sous-plages de `meter.min_bpm..max_bpm` (le moteur refuse un tempo hors de cette plage) ;
- `tonalite` : celles de `melody.key_modes` (poids du corpus) plus quelques tonalités courantes des airs de violon (do, fa, mi et la dorien) ;
  modes atteignables par le profil, aucun `[fixed_tonality]` n'existait ;
- `melodie` : violon (40), « fiddle » (110), accordéon (21), flûte (73) pour les britanniques, harmonica (22) pour la valse québécoise ; la marche :
  piccolo (72), flûte (73), clarinette (71), trompette (56) ; la contre-mélodie suit quand le profil en a une ;
- `comping` (guitare nylon 24 ou acier 25, piano 0, piano bastringue 3), `basse` (32, 33 ; piano 0 pour la gigue acadienne), `voicing`
  (`close_triad`, `fixed_triad` : triades seulement), `registre` (accords ±1 octave) ;
- `kit` (banque 128 : standard, room, brush ; marche : standard, room, power) pour les profils britanniques ; les québécois et acadiens gardent
  leurs pieds (`percussion_bank = 1`) ;
- `swing` (reels québécois et acadien 0,54–0,58 ; hornpipe 0,62–0,68) ;
- `energie` (intensité 0,85 / 1 / 1,12) et `pieds` (densité 0,8 / 1 / 1,2 : densité des pieds ou de la percussion) ;
- `arc` : `defaut` (plat), `montee` (0,8 → 0,95 → 1,1), `sommet` (deux tours posés, un sommet), `orchestre` (comping et basse qui changent
  à l'exposition et au sommet ; absent pour la marche) ; chaque arc reprend à chaque air d'un set (un chorus = un tour, trois tours par air),
  et le dernier tour du set se détache (`fall`, intensité 1,25 à 1,3).

Les airs (cellules, rôles, formes, ornements) ne changent pas. La marche n'a pas d'accompagnement : mélodie, kit, tempo, tonalité,
énergie et arc seulement. Section « Habillage par graine » ajoutée à chaque README de profil.

Programmes vérifiés dans le `phdr` de `assets/GeneralUser-GS.sf2` par le générateur (refus sinon) : banque 0 : 0, 3, 21, 22, 24, 25, 32, 33,
40, 56, 71, 72, 73, 110 ; banque 128 : 0, 8, 16, 40.

## Preuve : habillages distincts sur 16 graines (V2, D6)

`r2-trad/habillage_mesure.py`, binaire de la branche, vérité terrain de la première mesure ; un habillage = (grappe de tempo à ± 8 BPM, tonique,
mode, programmes de mélodie, d'accords, de basse, du kit, famille de voicings, swing). Seuil du brief : ≥ 8 ; avant : **1** par profil.

| Profil | Habillages distincts / 16 | Grappes de tempo (plage tirée) | Tonalités | Mélodie | Comping | Basse | Kit |
|---|---:|---|---:|---:|---:|---:|---:|
| `reel_quebecois` | **15** | 2 (220 à 239 BPM) | 6 | 3 | 4 | 2 | 1 |
| `reel_acadien` | **15** | 3 (215 à 237 BPM) | 5 | 3 | 4 | 2 | 1 |
| `reel_britannique_coloniale` | **16** | 4 (192 à 223 BPM) | 5 | 4 | 3 | 2 | 3 |
| `gigue_quebecoise` | **16** | 3 (160 à 184 BPM) | 4 | 3 | 4 | 2 | 1 |
| `gigue_acadienne` | **16** | 3 (168 à 191 BPM) | 5 | 3 | 4 | 3 | 1 |
| `gigue_britannique_coloniale` | **16** | 3 (144 à 167 BPM) | 6 | 4 | 3 | 2 | 3 |
| `hornpipe_britannique_coloniale` | **16** | 3 (80 à 103 BPM) | 6 | 4 | 3 | 2 | 3 |
| `marche_britannique_coloniale` | **16** | 4 (90 à 125 BPM) | 5 | 4 | 0 | 0 | 3 |
| `valse_quebecoise` | **16** | 1 (140 à 150 BPM) | 7 | 4 | 4 | 2 | 1 |
| `valse_britannique_coloniale` | **16** | 3 (126 à 149 BPM) | 5 | 4 | 3 | 2 | 3 |

La valse québécoise ne couvre que 10 BPM (140 à 150) : une seule grappe de tempo à ± 8 BPM, bornes de la tradition du profil.

## Preuve : `seed-variety --chorus 4` avant / après (16 graines × 4 chorus)

Binaires `/tmp` de `main` et de la branche, profils de `main` pour « avant » (`git archive`), même compilateur. Distance graduée entre graines
(`inter.graded`), 0 % = identique.

| Profil | Orchestration : suites distinctes | Orchestration : distance | Accompagnement : distance | Mélodie : distance | Batterie : distance |
|---|---|---|---|---|---|
| `reel_quebecois` | 1 → 9 | 0% → 31% | 47% → 53% | 95% → 96% | 12% → 16% |
| `reel_acadien` | 1 → 6 | 0% → 26% | 48% → 55% | 94% → 95% | 12% → 15% |
| `reel_britannique_coloniale` | 1 → 12 | 0% → 53% | 61% → 63% | 96% → 95% | 0% → 0% |
| `gigue_quebecoise` | 1 → 8 | 0% → 24% | 47% → 52% | 95% → 96% | 13% → 16% |
| `gigue_acadienne` | 1 → 12 | 0% → 34% | 51% → 54% | 94% → 96% | 13% → 16% |
| `gigue_britannique_coloniale` | 1 → 12 | 0% → 53% | 59% → 61% | 94% → 94% | 0% → 0% |
| `hornpipe_britannique_coloniale` | 1 → 13 | 0% → 50% | 59% → 63% | 96% → 97% | 0% → 0% |
| `marche_britannique_coloniale` | 1 → 6 | 0% → 51% | 16% → 19% | 96% → 95% | 0% → 5% |
| `valse_quebecoise` | 1 → 9 | 0% → 35% | 49% → 50% | 96% → 96% | 13% → 15% |
| `valse_britannique_coloniale` | 1 → 13 | 0% → 52% | 50% → 49% | 92% → 90% | 0% → 0% |

L'orchestration (programmes lus dans le MIDI) passe de 1 suite distincte à 6–13 sur 16 graines. Mélodie : inchangée à 1–2 points (les airs ne
changent pas). Batterie : la distance entre graines des profils québécois monte de 12 à 16 % (densité des pieds, énergie) ; elle reste à 0 % pour
les valses, dont les patterns de pieds n'ont qu'une densité utile. Tableau complet : `/tmp/h019-r2-trad/variete.json` (régénérable par
`r2-trad/comparer_variete.py`).

## Preuve : distance au corpus (`harmonium-lab trad-compare`, comme les phases 009–012)

16 graines × 32 mesures exportées en MIDI, comparées au `profile.json` du type dans le labo ; écart moyen en valeur absolue des z des écarts
signalés (plus petit = plus proche). Variation d'un tirage à l'autre de l'ordre de ± 0,07 ; aucun profil ne perd plus de 0,07. Les notes de la mélodie viennent des mêmes banques ; ce qui bouge, c'est la tonalité, le registre et le flux du RNG.

| Profil | Référence du labo | Écart moyen \|z\| avant | après |
|---|---|---:|---:|
| `reel_quebecois` | trad-reel | 0.419 | 0.430 |
| `reel_acadien` | trad-acadienne-reel | 0.606 | 0.672 |
| `reel_britannique_coloniale` | trad-britannique_coloniale-reel | 0.939 | 0.845 |
| `gigue_quebecoise` | trad-jig | 0.682 | 0.587 |
| `gigue_acadienne` | trad-acadienne-jig | 0.759 | 0.700 |
| `gigue_britannique_coloniale` | trad-britannique_coloniale-jig | 0.732 | 0.650 |
| `hornpipe_britannique_coloniale` | (aucun air de hornpipe dans le corpus) | — | — |
| `marche_britannique_coloniale` | trad-britannique_coloniale-march | 1.082 | 1.012 |
| `valse_quebecoise` | trad-waltz | 0.576 | 0.632 |
| `valse_britannique_coloniale` | trad-britannique_coloniale-waltz | 0.772 | 0.721 |


## Tests

- Nouveau `harmonium_core/tests/trad_session_profile_tests.rs` (5) : ≥ 8 habillages distincts sur 16 graines, tempo dans la plage de la tradition,
  instruments dans la liste de la tradition, même graine = mêmes notes, l'hôte garde la tonalité et le tempo, la vélocité de la mélodie monte du
  tour 1 au tour 3 sous l'arc `montee`.
- Le profil par défaut est `reel_quebecois` : quatre tests d'hôte qui supposaient son tempo nominal (230) vident sa `session`
  (`emotion_tests` ×2, `midi_export_contract_tests`, `style_profile_tests`), un cinquième compare au tempo tiré (`deterministic_seek_tests`).
  Golden `snapshot_stream_seed42_8bars.json` régénéré (profil par défaut).
- `harmonium_determinism/scripts/run.sh` : natif = wasm, **153 / 153** fichiers identiques (cible privée du worktree : une cible partagée entre worktrees remplace le binaire `determinism_dump` par celui d'un autre).
- `cargo test -p harmonium --features odin` 20 ; `harmonium_bevy` 11 ; `cargo check -p harmonium_studio` propre ; fmt propre.

## Mesures de la machine (partagée)

- `seek_to_bar_64_under_50ms` échoue ici avec ou sans habillage : minimum de 25 seeks, build de test, graine 42 : **76,4 ms avec session,
  70,7 ms sans** (charge de 5 à 46 pendant les mesures). L'habillage ajoute 8 % ; la limite de 50 ms n'est tenue que sur le Mac (26 ms).
- `export_terminates` (200 mesures, tous les profils distribués en parallèle, build de test) dépasse 300 s sous une charge de 30 à 46 :
  quinze profils bloqués, dont neuf `gmd_*` et des jeux que cette tâche ne touche pas ; les dix trad s'exportent en 4 à 9 s chacun avec le CLI
  de la branche (`--release`).
- `audible_tempo_is_taken_then_cleared_at_pulse_boundaries` a échoué une fois sous charge, réussi seul.

## Manques de schéma (A7)

- Un tag de percussion sur les patterns de pieds (`percussion.patterns[].tag`) permettrait des variantes de pieds (talon seul, pointes
  doublées) tirées par graine ; aujourd'hui seule la densité (`density`) les fait varier.
- Pas de `melody_program` par chorus avec contre-mélodie liée : un `instruments` d'arc peut changer `melody_program` mais pas le relier à un
  duo accordéon / violon.
- Une variante de session ne peut pas ajouter une piste (guitare *et* piano) : un seul programme d'accords par session.

## Liens d'écoute (à ouvrir par l'orchestrateur)

- `harmonium://open?profile=reel_quebecois&seed=11` et `…&seed=13` : 222 BPM la mixolydien, fiddle, guitare nylon, montée / 239 BPM ré ionien, violon, guitare acier, arc plat
- `harmonium://open?profile=reel_acadien&seed=7` et `…&seed=8` : 236 BPM ré ionien, violon, piano / 218 BPM la dorien, fiddle, piano bastringue
- `harmonium://open?profile=reel_britannique_coloniale&seed=5` et `…&seed=9` : 223 BPM do ionien, violon, piano, orchestre / 192 BPM ré ionien, accordéon, guitare acier
- `harmonium://open?profile=gigue_quebecoise&seed=11` et `…&seed=13` : 164 BPM la ionien, fiddle / 184 BPM ré mixolydien, violon, montée
- `harmonium://open?profile=gigue_acadienne&seed=11` et `…&seed=13` : 172 BPM la ionien, fiddle, piano bastringue / 191 BPM mi dorien, violon, piano, montée
- `harmonium://open?profile=gigue_britannique_coloniale&seed=7` et `…&seed=9` : 164 BPM mi éolien, kit brush, sommet / 144 BPM ré ionien, accordéon, guitare acier
- `harmonium://open?profile=hornpipe_britannique_coloniale&seed=2` et `…&seed=13` : 86 BPM mi éolien, swing 0,62, accordéon, piano / 103 BPM sol ionien, swing 0,68, violon, guitare acier
- `harmonium://open?profile=marche_britannique_coloniale&seed=7` et `…&seed=8` : 123 BPM fa ionien, flûte, montée / 94 BPM ré éolien, clarinette, sommet
- `harmonium://open?profile=valse_quebecoise&seed=13` et `…&seed=15` : 150 BPM do ionien, violon, montée / 140 BPM ré ionien, harmonica, guitare acier, orchestre
- `harmonium://open?profile=valse_britannique_coloniale&seed=7` et `…&seed=9` : 146 BPM la ionien, violon, brush, sommet / 126 BPM sol ionien, accordéon, guitare acier

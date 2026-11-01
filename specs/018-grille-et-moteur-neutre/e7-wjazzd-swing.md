# E7 — Profil distribué `wjazzd_swing`

Date : 2026-11-01. Branches : moteur `wquintal/e7-wjazzd-swing` (partie de `main` à `f9de703`,
`TUNING_VERSION` 24), labo `wquintal/e7-wjazzd-swing` (partie de `main`, `fb1d47b`). Décision F3 du
plan 018 : combinaison 1, mentions ODbL §4.3 / CC BY 4.0 / MIT, NC et SA au labo seulement, nom
`wjazzd_swing`, grilles du domaine public vérifiées aux États-Unis **et** au Canada.

**Le moteur n'a pas changé** : aucun fichier de `src/` ; le profil est fait de données. Les notes des
profils existants et `TUNING_VERSION` sont donc inchangés (preuves plus bas). Un seul test existant a
été corrigé (`nappe_contract_tests`, voir « Preuves »).

## Sources et licences utilisées

| Source | Version, SHA256 vérifié avant lecture | Licence | Usage | Mention portée par `[meta].corpus` |
|---|---|---|---|---|
| Weimar Jazz Database | v2.1 (base 2.2), `af6a0d9debf042c3581565bd75baad591d32e166cd2ec7298519883de614bf12` (= `sources-jazz.md`) | ODbL 1.0 + DbCL 1.0 | sous-ensemble `rhythmfeel = SWING` : basse par temps, rythme harmonique, tempos, formes. Statistiques agrégées seulement (*Produced Work*) | ODbL §4.3 |
| Groove MIDI Dataset | v1.0.0 midi-only, `651cbc524ffb891be1a3e46d89dc82a1cecb09a57c748c7b45b844c4841dcc1e` (= `sources-jazz.md`) | CC BY 4.0 | `jazz`, `jazz/swing` (cœur), `blues/shuffle`, 4/4 : probabilités par pas de la batterie, rapport de swing du ride | Gillick et al., ICML 2019 |
| Jazz Trio Database | v0.2, `jazz-trio-database-v02.zip` (release GitHub `v02-zenodo`), `42c3df0700e0cd6e73231b9ed111283600750305e2e9cf451f80664d1ec90025`, MD5 `43f543fb286c6222ae1f52bcf7561f37` (celui de `mirdata`) | MIT | annotations CSV/JSON : rapport de swing, placement basse/piano/batterie. Jamais `piano_midi.mid` | Cheston et al., TISMIR 2024 |
| Grilles | écrites pour le profil | forme commune / domaine public US + Canada | blues de douze mesures, AABA *rhythm changes* | README du profil |

Le SHA256 de WJazzD et de GMD est celui de `sources-jazz.md` (vérifié par le code avant toute lecture, test
d'une empreinte fausse). JTD : la note L0 donnait l'URL du dépôt GitHub, dont les CSV d'annotations ne sont pas
dans l'arbre (le dépôt fait 1,1 Go de documentation) ; ils sont dans la release `jazz-trio-database-v02.zip`
(24 Mo) que `mirdata` télécharge, dont le MD5 correspond. Sources NC/SA : aucune (iRb, PiJAMA, JHT, JAAH ne sont pas lues).
Aucun fichier sous droits ni base brute dans les deux diffs (`git ls-files` : manifestes sans note, `VERSION`, JSON agrégés).

## Labo (module générique `harmonium_lab/backing/`, nommé par rôle)

| Module | Rôle |
|---|---|
| `chords.py`, `wjazzd.py` | analyseur des 419 symboles d'accord de la base (0 % non analysé) ; lecteur SQLite, manifeste, `VERSION` |
| `bass_measures.py` | arrivée, approche, passage, intervalles, registre, rythme harmonique, tempos, formes |
| `bass_compare.py`, `validate.py` | mêmes mesures sur la vérité terrain du moteur (`--record-truth`), distance par critère et agrégée, plusieurs profils par rapport |
| `drums.py` | GMD → grille 16 pas swinguée, probabilités par pas, patterns proposés |
| `timing.py` | JTD → rapport de swing par tempo, placement relatif, dispersion |
| `engine_values.py` | un seul chemin des mesures aux valeurs du profil : `percussion.toml` généré, poids de basse |
| Makefile (bloc séparé) | `backing-fetch`, `backing-measures`, `backing-values`, `backing-validate` |

`references/wjazzd-swing/README.md` : provenance, commandes, mesures, limites. `make test` du labo : **408 réussis,
31 ignorés** avant mon ajout (`BassMeasures`, `DrumTimingMeasures`), **414** après (+6 tests d'`engine_values`). `backing-measures` relancé donne des fichiers identiques.

Commits du labo : `ba777e1`, `6a59501`, `a108804`, `c081ae0` (basse), `323114f` (batterie), `08bef6d` (timing),
`397f77e` (valeurs de profil, validation, README, Makefile).

## Mesures → valeurs du profil

| Paramètre du profil | Valeur | Mesure (n) | Source |
|---|---|---|---|
| `meter.default_swing_ratio` | 0,71 | rapport de swing de la batterie : 125–150 BPM 0,718 (160 pistes), 150–175 0,700 (149) ; GMD cœur 0,646 (18 fichiers, kit électronique, non retenu) | JTD |
| `default_bpm` / `min_bpm` / `max_bpm` | 150 / 90 / 285 | tempo moyen par solo : médiane 177,5, blues 147,4 ; centiles 10, 90 : 93, 284 (361 solos) | WJazzD |
| `bass.walk_low` / `walk_high` | 33 / 50 | hauteur de basse, centiles 1 et 99 : 33, 52 (92 896 temps) | WJazzD |
| `bass.low` / `high` | 36 / 47 | arrivées, centiles 10–90 : 36..46 ; meilleure fenêtre de 12 demi-tons 36..47 | WJazzD |
| `walking.max_leap` | 11 | intervalles, centile 95 : 11 (50 / 75 / 90 / 99 : 2 / 5 / 8 / 14 ; n = 61 380) | WJazzD |
| `walking.approach` | none 60, chromatic 8, diatonic 8 | parts de notes d'approche : aucune 59 %, chromatique 21 %, diatonique 19 % (n = 27 931) | WJazzD |
| `walking.passing` | chord_tone 35, scale_step 65 | passage : accord 48 %, gamme 23 %, chromatique 29 % (n = 52 041) | WJazzD |
| `toward_target_weight` / `keep_direction_weight` | 2 / 1 | se rapprochent de la cible 44 %, poursuivent le sens 42 % | WJazzD |
| Blues : un accord par mesure, deux dans la dernière | — | 92 % des mesures à un accord (87 solos), changement au temps 1 dans 89 % | WJazzD |
| `[[patterns]]` de batterie | 11 patterns pondérés | probabilités par pas du ride (spang-a-lang 0,4,6,8,12,14), pédale pas 4 et 12 (45 %, 39 %), caisse claire fantôme, grosse caisse (1 229 mesures `beat`) | GMD |
| `[[cadence_patterns]]` densité 2 | relance | 22 mesures `fill` | GMD |
| `performance.Chords.offsets` | 0,2 pas | piano moins batterie +0,226 pas, 976 pistes | JTD |
| `performance.Bass` | 0 pas, jitter 0,1 | basse moins batterie 0,000 pas ; écart-type 0,22 borné à 0,1 | JTD |
| Voicings, rythmes de comping | `rootless` ; 6 rythmes dont 3 anticipés et 1 silence | **aucune mesure distribuable** : règles de métier (F3). JTD ne donne que le piano du soliste (7 attaques par mesure) | — |
| Courbes d'émotion, mixage | — | réglages de service | — |

**Pourquoi les poids de basse ne valent pas les parts mesurées.** Les parts décrivent la note jouée ; les poids sont un
tirage. Avec 21 / 19 / 59 % comme poids, le moteur joue 34 % d'approches chromatiques et 33 % de diatoniques (heldout,
16 graines) : une note de passage tirée « sans approche » tombe souvent à un demi-ton ou un ton de la cible. Les poids
sont donc ajustés (descente par coordonnée sur les graines 1 à 6 ; critères approche, passage et intervalles de
`bass_compare`) pour que les parts de notes **générées** retrouvent les parts mesurées. Score moyen sur ces critères, 16
graines non vues (11 à 26) : poids égaux aux parts (avec les `max_leap` 5, `toward` 3 et `keep` 2 de la fixture) 0,234 ;
poids retenus 0,129 (0,128 sur les graines 1 à 6 de la calibration : pas de surajustement). `max_leap` reste à 11 (centile 95)
et non au 12 que la descente préférait (0,114 sur graines non vues) : un saut d'octave de passage n'a pas de justification
dans les mesures au-delà du centile 95.

## Le profil

`profiles/wjazzd_swing/` : `profile.toml`, `percussion.toml` (généré), `dynamics.toml`, `arrangement.toml`, `set.toml`,
`cells.toml` (banque minimale, jamais jouée), `README.md`. Modèle `loop`, `fixed_tonality` fa ionien, 4/4 à 16 pas,
`steps_per_chord` 4. Un tour de 44 mesures (`Blues`, `A`, `A`, `B`, `A`), deux tours par air. Pas de piste `Melody`
(absente de l'arrangement). Instruments GeneralUser GS : piano 0, basse acoustique 32, kit jazz banque 128 programme 32.
Odin : non utilisé (optionnel, non requis).

Grilles (provenance et droits : README du profil) :

- blues en fa, `F7 | Bb7 | F7 F7 | Bb7 Bb7 | F7 F7 | Gm7 | C7 | F7 | F7 C7` ;
- AABA *rhythm changes*, écrit en fa pour garder une seule tonique (tonalité verrouillée) : A `F6 D7 | Gm7 C7 | Am7 D7 |
  Gm7 C7 | Cm7 F7 | Bb6 Bbm6 | F6/C D7 | Gm7 C7` ; B `A7 A7 D7 D7 G7 G7 C7 C7`. *I Got Rhythm* (George Gershwin, 1930)
  est dans le domaine public américain depuis le 2026-01-01 (Duke CSPD, qui la cite comme source des *rhythm changes*) ;
  Gershwin est mort en 1937 : libre au Canada [inférence depuis la règle vie + 70 ans non rétroactive, CARL]. Les paroles
  d'Ira Gershwin ne sont pas utilisées ; la grille est écrite comme forme commune (Compendium USCO §802.5(A)), pas
  recopiée d'une partition. *Body and Soul* et *Georgia on My Mind* sont exclues (libres aux États-Unis seulement).

## Validation

### Distance à WJazzD (labo, `validation-bass.md`)

Basse générée (8 graines × 88 mesures) contre le sous-ensemble swing, 0 = identique. Ligne de base : la fixture
`grille_accompagnee` (marche non calibrée), même moteur.

| Famille | `wjazzd_swing` | `grille_accompagnee` |
|---|---|---|
| approche | **0,169** | 0,435 |
| intervalles | **0,144** | 0,186 |
| registre | **0,117** | 0,185 |
| passage | 0,137 | **0,082** |
| rythme harmonique (grille) | 0,302 | 0,188 |
| arrivée (fixe) | 0,678 | 0,678 |
| **moyenne des critères** | **0,250** | 0,326 |

Sur les familles réglables (approche, intervalles, registre, passage) : 0,142 contre 0,222. Seize autres graines
(`validation-bass-heldout.md`) : 0,250 contre 0,327. Le passage est le prix de la calibration. Critères E6 du moteur
vérifiés sur le généré contre WJazzD (`validation-bass.md`, groupe `all`) :

| Critère | WJazzD | Généré |
|---|---|---|
| approche : aucune / chromatique / diatonique | 59 / 21 / 19 % | 49 / 20 / 30 % |
| passage : accord / chromatique / gamme | 48 / 29 / 23 % | 58 / 14 / 29 % |
| intervalles, centiles 50 / 75 / 90 / 95 / 99 | 2 / 5 / 8 / 11 / 14 | 2 / 5 / 7 / 9 / 10 |
| poursuite du sens ; vers la cible | 42 % ; 44 % | 53 % ; 43 % |
| répétitions | 20 % | 3 % |
| arrivée sur la basse de l'accord au changement | 41 % | 100 % |
| fenêtre des arrivées | 36..46 | 36..46 |

### Tests de contrat du moteur

`harmonium_core/tests/wjazzd_swing_profile_tests.rs` (7 tests, flux de `MeasureSnapshot`, graines 1, 2, 17, deux tours) :
la grille par défaut (fondamentales et pas de chaque mesure, écrits en clair relatifs à la tonique) ; quatre noires de basse,
durée 4, fenêtre 33..50, arrivée sur la basse publiée à chaque changement, arrivées dans 36..47 ; l'accord slash joué sur sa
basse ; comping dans l'accord publié, sans sa fondamentale, en anticipant l'accord suivant seulement au pas 14 ; ride à
chaque temps et pédale aux temps 2 et 4 aux trois densités, relance de fin de phrase sans ride à la densité 2 ; tempo dans la
plage à cinq intensités et swing du profil ; mêmes graines = mêmes mesures, graines différentes = lignes de basse différentes.
Mutations vérifiées : `voicing = close_triad` fait échouer le test du comping ; `walk_high = 55` celui de la marche.

`nappe_contract_tests::seuls_les_profils_a_nappes_lient` supposait que tout profil en boucle lie ses nappes (cas des seuls
profils de jeu) ; il exclut maintenant les profils qui jouent une banque de rythmes de comping. C'est la seule modification
d'un test existant.

## Preuves

| Vérification | Résultat |
|---|---|
| `harmonium_determinism`, binaire de `main` (`git archive` de `f9de703`, cible séparée) contre la branche | 92 fichiers identiques octet pour octet ; 6 fichiers `wjazzd_swing_s{1,2,3}_{mid,hi}.json` ajoutés |
| WAV et MIDI (`--seed 1 --bars 32`, CLI lancé depuis `~/Project/bascanada/harmonium`) de `reel_quebecois`, `valse_quebecoise`, `isaac_cellar`, `nt_desert`, binaires `main` et branche | `cmp` identiques |
| `harmonium_determinism/scripts/run.sh` (wasm-bindgen 0.2.108, `/tmp/n1/wb/bin`) | natif = wasm : 98 / 98 (92 + 6) |
| `TUNING_VERSION`, goldens | inchangés (aucun fichier de `src/` modifié) |
| `cargo test --workspace -q --no-fail-fast` | 774 réussis, 0 échec, 2 ignorés (767 avant : +7 tests de contrat) |
| `cargo test -p harmonium --features odin`, `cargo test -p harmonium_bevy` | 118 réussis, 11 réussis |
| `cargo check -p harmonium_studio` ; `cargo check --target wasm32-unknown-unknown` (`harmonium_core`, `harmonium_audio`, `harmonium` avec et sans `odin`, `harmonium_bevy --no-default-features`) | passent ; le studio liste le profil (`StyleProfile::distributed()`), sans changement de sa logique |
| `cargo clippy -p harmonium_core -p harmonium -p harmonium-cli --all-targets` | aucun avertissement dans les fichiers ajoutés ou modifiés (les avertissements `unwrap`/`panic` existants des tests sont inchangés) |
| `cargo +nightly fmt --all -- --check` | propre |
| Labo `make test` | 414 réussis, 31 ignorés |

## Écoute (propriétaire)

Les WAV sont dans `/tmp/e7/ecoute/`. Une mesure dure 1,6 s (150 noires/min) ; un tour fait 70,4 s (44 mesures). Commande (depuis
`~/Project/bascanada/harmonium`, binaire de la branche, graine 1) :

```sh
cd ~/Project/bascanada/harmonium && \
  ~/orca/workspaces/harmonium/e7-wjazzd-swing/target/release/harmonium-cli --export --seed 1 --bars 44 \
  --profile ~/orca/workspaces/harmonium/e7-wjazzd-swing/profiles/wjazzd_swing --record-wav /tmp/e7/ecoute/wjazzd_swing_s1.wav
```

Fichiers : `wjazzd_swing_s1.wav` (complet, densité 1), `wjazzd_swing_s1_densite2.wav` (`--percussion 2` : grosse caisse,
caisse claire fantôme, relances), `wjazzd_swing_s1_Bass.wav`, `_Chords.wav`, `_PercussionHigh.wav` (`--solo-track`),
`wjazzd_swing_s2.wav` (graine 2, mêmes grilles, autres lignes et rythmes), `.mid`.

| Moment | Mesures | Forme | À écouter |
|---|---|---|---|
| 0:00 à 0:19 | 1 à 12 | blues en fa | marche en noires, retour sur fa et si♭ au premier temps, approche au 4ᵉ temps ; ride en noires avec croche swinguée aux temps 2 et 4 ; charleston aux temps 2 et 4 |
| 0:12,8 à 0:16 | 9 et 10 | ii-V du blues | Gm7 puis C7 une mesure chacun : basse sol puis do |
| 0:17,6 à 0:19,2 | 12 | retour | F7 puis C7 à mi-mesure : arrivée de la basse au 3ᵉ temps |
| 0:19,2 à 0:32 | 13 à 20 | A (1ʳᵉ fois) | deux accords par mesure : une arrivée de basse à chaque changement, approche chromatique ou diatonique avant ; comping en voicings sans fondamentale qui bougent peu |
| 0:27,2 à 0:30,4 | 18 et 19 | fin d'A | Bb6 puis Bbm6 ; puis F6 sur do (basse do au premier temps), D7 au 3ᵉ temps |
| 0:32 à 0:44,8 | 21 à 28 | A (2ᵉ fois) | mêmes accords, autres lignes et rythmes (tirages par mesure) |
| 0:44,8 à 0:57,6 | 29 à 36 | B | A7, D7, G7, C7 deux mesures chacun : la marche tient un accord sur deux mesures |
| 0:57,6 à 1:10,4 | 37 à 44 | A (dernière) | retour |
| silences de comping | p. ex. mesures 15, 23, 34, 36, 40, 41 | | une mesure sans accord, la basse continue |
| densité 2 (fichier `_densite2`) | 12, 20, 28, 36, 44 | fins de phrase | relance sans ride (caisse claire, toms, grosse caisse) ; ghost notes de caisse claire |

À juger : (1) le naturel de la ligne de basse (notes d'accord trop fréquentes ? trop de sauts d'octave ?) ; (2) le swing de
0,71 ; (3) la lisibilité des voicings sans fondamentale au-dessus de la basse ; (4) le poids des anticipations ; (5) l'équilibre
du kit de rendu (le ride est le plus discret, calé par `ROLE_GAIN` pour que la densité 2 n'écrête pas).

## Limites

- **Arrivée** : WJazzD n'arrive sur la basse publiée que dans 41 % des cas (63 % pour les solos dont la basse tombe sur
  l'accord) ; le moteur y arrive toujours. Aucun paramètre ne lève cet écart (fondamentale hors arrivée et arrivée sur une note
  d'accord non représentables). Ni décalage d'un temps ni erreur d'octave ne l'explique (intervalles sans pic à 12).
  La qualité de `bass_pitch` n'est pas documentée par Jazzomat.
- **Pas de répétition de note** (20 % mesurés, 3 % générés) ; côté d'approche non réglable (85 % du côté de l'arrivée précédente,
  66 % mesurés) ; part de notes d'accord au passage trop haute (58 % contre 48 %) ; poursuite du sens trop fréquente (53 % contre 42 %).
- **Un seul rapport de swing** (0,71) : JTD le mesure de 0,72 (100–125 BPM) à 0,58 (275–330). `SetSwing` ou `--swing` le change.
- **Comping** : aucune source distribuable ; rythmes et voicings sont des règles de métier, jugés à l'écoute. Le MIDI piano de JTD
  joue le soliste.
- **Batterie** : le GMD est joué sur un kit électronique et seuls 22 fichiers `jazz` (11 `jazz/swing`) sont étiquetés ; 52 % de ses
  mesures n'ont pas de ride. Les niveaux du kit de rendu sont recalés par rôle (`ROLE_GAIN`), sans corpus.
- **Rythme harmonique** de l'AABA : deux accords dans 24 de ses 32 mesures, contre 22 % des mesures de WJazzD.
- **Corpus historique** : le sous-ensemble swing de WJazzD va de 1937 à 2007 avec une majorité de hard bop et de post-bop ;
  le nom ne promet aucune époque.
- Les valeurs de dynamique émotionnelle (`dynamics.toml`), de mixage et de vélocité sont des réglages de service.

## Fichiers

Moteur : `profiles/wjazzd_swing/{profile,percussion,dynamics,arrangement,set,cells}.toml`, `profiles/wjazzd_swing/README.md`,
`profiles/README.md` (tableau et renvois), `harmonium_core/tests/wjazzd_swing_profile_tests.rs`,
`harmonium_core/tests/nappe_contract_tests.rs`, ce compte rendu, `plan.md`.
Labo : voir le tableau des modules ci-dessus.

# R2-jeux — Variété qui s'entend dans les profils de jeu

Date : 2026-11-03. Branche `wquintal/r2-jeux`, partie de `main` (`eecbd3f`, `TUNING_VERSION` 26).
**`TUNING_VERSION` 27** : rupture assumée (principe I), les notes de cinq profils changent. Données seulement :
sous `src/`, seule la constante et sa ligne de doc (`tuning.rs`) changent. Aucune lecture audio, aucun studio
(D5, D7) ; les rendus hors ligne servent aux seules mesures.

## Résumé

| Question | Réponse |
|---|---|
| Profils touchés | `isaac_basement`, `isaac_burning`, `isaac_cellar`, `isaac_downpour`, `nt_desert`. |
| Preuve d'identité des autres | Les 20 autres profils distribués : `--export --bars 64` graines 1 et 2, MIDI identique octet pour octet à celui du binaire de `main` ; les cinq profils touchés diffèrent. Natif = wasm : **153 / 153**. |
| Habillages distincts sur 16 graines (tempo ±8 BPM, tonalité, programmes, voicing) | 1 → **12** (`isaac_basement`), **14** (`isaac_burning`), **14** (`isaac_cellar`), **11** (`isaac_downpour`), **10** (`nt_desert`) ; cible ≥ 8. |
| Variété par graine (`seed-variety --chorus 4`) | Batterie : 16 suites distinctes sur 16 graines (7–10 avant), chaque chorus d'une graine différent (1,0 → 4,0 chorus distincts) ; orchestration 1 → 4 à 14 suites ; accompagnement entre chorus 0 % → 40 à 69 % des mesures changées. Accords et basse ne varient pas d'un chorus à l'autre (voir manques). |
| Distance au corpus | Couches symboliques (basse, batterie, accords) : dans le point sauf **`nt_desert` accords −1,6** (bruit de tirage : −0,5 sur 40 graines). **L'audio global baisse de 1,1 à 4,0 points** (tempo et timbres qui varient : voulu) : voir § 4, à trancher par le propriétaire. |
| Seek `seek_to_bar_64_under_50ms` | Test inchangé vert (profil par défaut 25,9 ms). Profils touchés (build de test, 15 passes) : `isaac_downpour` 29,8 ms (R1 : 15,1), `isaac_cellar` 11,5, `nt_desert` 15,2, `isaac_basement` 9,0, `isaac_burning` 7,3 ms. |

## 1. Ce qui change, profil par profil

Chaque profil reçoit `session.toml` (groupes tirés une fois par graine), `arc.toml` (étapes d'un chorus,
`repeat_from`, **aucune retombée** : le jeu n'annonce pas de fin, un set = un air = deux tours mais l'arc reste
valable à 4 ou 16 tours), des phrases et des relances de batterie (`[[phrases]]`, `[[fills]]` remplaçant les
`[[cadence_patterns]]` à contenu repris), des tags. Les contrats du jeu ne bougent pas : `dynamics.toml`,
`density_tracks`, conditions et ordre des zones Odin. Les variantes Odin **permutent des presets déjà présents**
en emportant leur `level_db` (aucun rendu Odin, aucune écoute ; les noms existent dans `odin-presets`).
Programmes SoundFont de GeneralUser GS (`nt_desert` : seulement 38, 81, 88, 89 et le kit 0 de `nt_desert.sf2`).
Détail et provenance de chaque variante : en-têtes de `session.toml`, `arc.toml` et `percussion.toml`.

| Profil | Groupes de session | Particularités |
|---|---|---|
| `isaac_cellar` | tempo (128–134, **138–144 ×6**, 148–154), basse (38 / 33 / 34 + zones Odin permutées), nappe (91 / 89 / 19), chœur (53 / 52 / 10), kit (0 ×5, 16, 8, 24), groove (stomp / demi-temps), voicing (serré / drop2 / open), registre de la nappe, énergie (`density` seule), arc (`descente` ; `orchestre` ; `calme`) | Le plus nourri, à la demande du propriétaire. **Harmonie** : pédale d'A à alternative (D1, ou D1 + voisine D#, poids 4/2), boucle générée de deux accords dans la gamme (`key_fit = "within"`, pédale D) en A', descente chromatique générée au pont. **Batterie** : 5 patterns écrits étiquetés `stomp`, ossature de *River of Despair* (`sparse`), demi-temps de *Fundamentum* (`rock`), stomp dense ; 13 phrases ; 9 relances dont deux de fin de chorus sur deux mesures. |
| `isaac_basement` | tempo (120–127 ×1, 136–144 ×8, 153–160 ×1), basse, nappe, chœur, kit, voicing, énergie, patterns, arc | Comping de nappe (tenue / souffle) en phrases ; relances `part_end` et `chorus_end`. |
| `isaac_burning` | tempo (126–129 ×1, 138–142 ×6, 151–154 ×1), basse, nappe, chœur, kit, voicing, énergie, batterie, arc | Patterns `drive` (règle de métier : snare sur 8, hats 4/12, levée de grosse caisse). Les P1 + P2 de R1 sont conservés. |
| `isaac_downpour` | tempo (124–130, 140–148 ×6, 158–164), basse, nappe, chœur, kit (0 / 8), voicing, registre, énergie, patterns, arc (`blocs`, `brume`) | Groove de densité 1 de R1 gardé à son poids d'origine (un premier essai le faisait passer de 50 % à 17 % des tirages et coûtait 2,5 points de batterie). 13 phrases, 18 relances. |
| `nt_desert` | tempo (204–216 ×4, 180–190, 226–236), basse (81 / 38 + Odin), nappe (89 / 88), chœur, batterie, énergie, arc (`montee`, `relais`) | Comping de nappe, 3 phrases de pont. Test de contrat `nt_desert_profile_tests.rs` migré vers les accords publiés. |

**Tonalité : une seule par profil, décision documentée.** La règle du brief est « attestée par les références du
niveau » : `isaac_basement`, `isaac_cellar`, `isaac_downpour` n'en ont qu'une (D mineur harmonique ; l'inflexion
phrygienne D# de Periculum est locale, ce n'est pas un mode global) ; `isaac_burning` F# phrygien (l'éolien exigeait
une bascule `parallel_modes`, donc le contrat émotion, et la référence donne G = b2 en confiance haute) ;
`nt_desert` atteste aussi A phrygien (*A Way*) mais une graine en A fait chuter la basse de Fläshyn de 86 à 18
(−6,8 points sur 10 graines) : le bloc est écrit dans la note de l'agent, non activé. Rien n'est emprunté à un autre
niveau ni à une autre tonique de la famille (C# mineur de Caves, etc.).

## 2. Habillages distincts, 16 graines (binaire de la branche, `--style`, 96 mesures)

| Profil | Avant | Après | Tempos joués (base) | Combinaisons de programmes |
|---|---:|---:|---|---:|
| `isaac_basement` | 1 | **12** | 136–144, 154–156 | |
| `isaac_burning` | 1 | **14** | 126–127, 138–142, 153 | 13 |
| `isaac_cellar` | 1 | **14** | 130–133, 138–144, 152 | 14 |
| `isaac_downpour` | 1 | **11** | 126–129, 140–148, 162 | |
| `nt_desert` | 1 | **10** | 204–216, 229–230 | |

## 3. `seed-variety --chorus 4`, avant (binaire de `main`) → après, 16 graines

Mesures changées d'un chorus au suivant ; chorus distincts par graine (sur 4) ; suites de 4 chorus distinctes
sur 16 graines.

| Profil | Couche | Mesures changées | Chorus distincts | Suites distinctes |
|---|---|---|---|---|
| `isaac_basement` | accompagnement | 0 % → **46 %** | 1,0 → **4,0** | 16 → 16 |
| | comping : voicings | 0 % → 4 % | 1,0 → 2,4 | 9 → **16** |
| | batterie | 0 % → **55 %** | 1,0 → **4,0** | 7 → **16** |
| | orchestration | 0 % → 6 % | 1,0 → 1,1 | 1 → **7** |
| `isaac_burning` | accompagnement | 0 % → **40 %** | 1,0 → **4,0** | 16 → 16 |
| | batterie | 0 % → **40 %** | 1,0 → **4,0** | 10 → **16** |
| | orchestration | 0 % → **31 %** | 1,0 → 1,6 | 1 → **12** |
| `isaac_cellar` | accompagnement | 0 % → **51 %** | 1,0 → **4,0** | 16 → 16 |
| | accords | 0 % → 0 % | 1,0 → 1,0 | 4 → **10** |
| | comping : voicings | 0 % → 0 % | 1,0 → 1,0 | 4 → **13** |
| | batterie | 0 % → **51 %** | 1,0 → **4,0** | 7 → **16** |
| | orchestration | 0 % → 15 % | 1,0 → 1,3 | 1 → **7** |
| `isaac_downpour` | accompagnement | 0 % → **69 %** | 1,0 → **4,0** | 16 → 16 |
| | batterie | 0 % → **69 %** | 1,0 → **4,0** | 10 → **16** |
| | orchestration | 0 % → **73 %** | 1,0 → 2,1 | 1 → **14** |
| `nt_desert` | accompagnement | 6 % → **41 %** | 2,0 → **4,0** | 16 → 16 |
| | comping : voicings | 0 % → 21 % | 1,0 → 2,6 | 15 → **16** |
| | batterie | 8 % → **47 %** | 2,0 → **4,0** | 8 → **16** |
| | orchestration | 0 % → 19 % | 1,0 → 1,2 | 1 → 4 |

Table complète par couche : sorties du labo dans `/tmp/harmonium-019/r2-jeux/{avant,final}/` (hors dépôt). Les
séquences d'accords distinctes baissent de 9 à 7 (`isaac_downpour`) et de 15 à 12 (`nt_desert`) : les tirages
de boucles se décalent avec les tirages de session (même loi, autre échantillon de graines), rien de voulu.

## 4. Distance au corpus (`make style-distance`, graines 1–10, 48 mesures, `type=zone`, GeneralUser GS ; `nt_desert.sf2` pour `nt_desert`)

Score 0–100, avant (`main`) → après. `boucles` n'entre pas dans la règle (R1 § 3) : la batterie qui varie d'un
chorus à l'autre ne se répète plus à la longueur de boucle de la référence.

| Profil, référence | Audio | basse | batterie | accords | boucles |
|---|---|---|---|---|---|
| `isaac_cellar`, *Periculum* | 54,9 → 50,9 (**−4,0**) | 78,2 → 78,4 | 83,5 → 84,7 | 67,8 → 67,3 | 35,4 → 36,1 |
| `isaac_basement`, *Diptera Sonata* | 42,0 → 40,4 (−1,6) | 74,5 → 74,9 | 56,8 → 58,6 | 77,8 → 78,4 | 49,9 → 37,1 |
| `isaac_basement`, *Periculum* | 38,7 → 37,6 (−1,1) | 63,7 → 64,2 | 83,9 → 85,1 | 64,5 → 65,0 | 57,0 → 43,9 |
| `isaac_burning`, *Fundamentum* | 39,4 → 37,4 (−2,0) | 68,7 → 67,8 | 79,9 → 82,8 | 61,9 → 61,1 | 38,5 → 38,5 |
| `isaac_downpour`, *River of Despair* | 59,6 → 56,7 (**−2,9**) | 78,5 → 78,9 | 58,1 → 62,6 | 86,5 → 86,4 | 23,1 → 15,3 |
| `nt_desert`, *Fläshyn* (`nt_desert.sf2`) | 61,0 → 58,5 (−2,5) | 86,1 → 86,1 | 62,4 → 63,0 | 83,8 → 82,2 (**−1,6**) | 53,9 → 52,7 |
| `nt_desert`, *A Way* (`nt_desert.sf2`) | 52,9 → 52,9 | 0,0 → 0,0 | 0,0 → 0,0 | 72,3 → 72,0 | 21,9 → 21,9 |
| `nt_desert`, *Fläshyn* (GeneralUser) | 60,3 → 59,8 | | | | |
| `nt_desert`, *A Way* (GeneralUser) | 55,3 → 59,0 (+3,7) | | | | |

Lecture. Les trois couches symboliques tiennent la règle, sauf les accords de `nt_desert` sur Fläshyn (−1,6 sur 10
graines ; l'agent mesure −0,5 sur 40 graines : bruit de tirage). **L'audio global ne tient pas le point** : il
perd 1,1 à 4,0 points. Cause mesurée sur `isaac_cellar` : le tempo (sous-score 100 → 90,3 avec des plages de ±3 à
±6 BPM autour de 140 ; 63,5 avec des plages de ±8), le kit et le timbre de la basse et de la nappe (le rapport
percussif 73,6 → 48,9). Ce sont les variations demandées (tempo, instruments) : la plage nominale pèse 67 à 80 %
et le kit standard le plus, sans quoi le coût double. Le levier est dans les poids de `session.toml` ; la baisse
d'`isaac_downpour` (−2,9) dépasse les −2,1 déjà acceptés par le propriétaire en R1 : **à trancher**.

Pour `isaac_cellar`, la batterie symbolique perdait 6,7 points en première écriture : les multiplicateurs
`intensity` **et** `density` ensemble font passer la batterie à la densité voisine (l'un sans l'autre : aucune
perte) ; l'arc ne règle donc que `intensity` et l'énergie de session que `density`. Les accords perdaient 2,2 :
la boucle générée d'A' à quatre accords est passée à deux accords (−0,6).

## 5. Liens d'écoute (profils compilés ; studio ouvert par l'orchestrateur, pause par défaut)

| Lien | Ce qui change |
|---|---|
| `harmonium://open?profile=isaac_cellar&seed=7` | 152 BPM, orgue de chœur 91, boîte à musique 10 au chœur, kit room, demi-temps, voicing ouvert, arc `orchestre` |
| `harmonium://open?profile=isaac_cellar&seed=11` | 130 BPM, basse au doigt 33, nappe chaude 89, kit standard, stomp, voicing drop 2, énergie retenue, arc `calme` |
| `harmonium://open?profile=isaac_cellar&seed=5` | 140 BPM, orgue 19, kit power, basse synthé 38, arc `calme` |
| `harmonium://open?profile=isaac_cellar&seed=1` | 139 BPM, basse piquée 34, kit électronique, chœur aahs, arc `orchestre` |
| `harmonium://open?profile=isaac_basement&seed=5` / `&seed=7` | 138 BPM, basse synthétique, kit standard, voicing serré / 156 BPM, basse au doigt, kit power, voicing ouvert, chœur aah, batterie en réponses |
| `harmonium://open?profile=isaac_burning&seed=15` / `&seed=7` | 126 BPM, basse médiator, pad sombre, batterie mixte, arc `montée` / 153 BPM, basse au doigt, orgue à tirettes, batterie `drive`, arc `orchestre` (kit et chœur changent par chorus), voicing drop 2 |
| `harmonium://open?profile=isaac_downpour&seed=14` / `&seed=7` | 140 BPM, nappe vocale, kit sec, arc `brume` / 162 BPM, nappe chaude, kit pièce, voicing ouvert, arc `blocs` |
| `harmonium://open?profile=nt_desert&seed=9` / `&seed=7` | 204 BPM, basse growly, nappe ouverte, batterie pleine, arc `montée` / 230 BPM, basse creuse 38, nappe sombre, batterie mixte, énergie tendue, arc `relais` |

Chorus 2 : ajouter `&bar=49` (`isaac_cellar`, `nt_desert` : 48 mesures par tour) ou `&bar=41` (`isaac_basement`, `isaac_burning` : 40 mesures) ; `isaac_downpour` n'a qu'un tour par air (`tours_per_tune = 1`), son arc ne joue que l'étape 1 du jeu et les suivantes sous `--tours`/set plus long :
l'arc, les relances de fin de chorus et la batterie y changent.

## 6. Manques de schéma et limites (tâche A7 à venir)

- **Arc sans choix de motifs ou gabarits de basse ni de boucles** : seuls `bass_feel` (marche), comping et batterie
  se choisissent par tag ; accords et basse restent identiques d'un chorus à l'autre (intra 0 %) dans les cinq profils.
- **Orchestration par chorus de la basse** : un `bass_program` d'arc scinde la piste MIDI de basse et fausse la
  distance symbolique (−37 points sur 3 graines pour `nt_desert`) ; l'arc `relais` ne change donc pas la basse.
- **Aucune dépendance entre groupes** (a2) : un tempo vif avec un kit lourd se tire indépendamment ; ni la
  tonalité de session ni l'arc ne sont conditionnés par le tempo.
- **Les zones Odin d'une variante remplacent toute la piste** : pas de `level_db` mesuré pour une permutation (aucun
  rendu Odin).
- **`intensity` + `density` de session et d'arc se composent avant l'arrondi de densité** de la batterie ; un
  profil de jeu qui veut tenir la distance au corpus ne règle que l'un par niveau (voir § 4).
- **Variantes non écoutées** (D5) ; les tonalités restent uniques ; le comping de nappe n'a qu'un rythme dans
  `isaac_cellar` (pas de banque : la nappe liée est son identité).

## 7. Vérifications

| Vérification | Résultat |
|---|---|
| `cargo test --workspace -q --no-fail-fast` | 0 échec (trois tests migrés : `seed_variety_tests` — batterie qui change d'un chorus à l'autre, accords du retour de partie inchangés —, `nappe_contract_tests` — nappe liée aussi par une banque de comping —, `profile_files_tests` — fichiers `arc.toml` et `session.toml`) ; `nt_desert_profile_tests` migré vers les accords publiés ; assertion `TUNING_VERSION == 26` supprimée de `set_contract_tests` (valeur incidente) |
| `cargo test -p harmonium --features odin`, `-p harmonium_bevy`, `cargo check -p harmonium_studio` | verts |
| `harmonium_determinism/scripts/run.sh` | natif = wasm : 153 / 153 |
| `cargo +nightly fmt --all -- --check` | propre ; clippy : listes d'avertissements des fichiers touchés identiques à `main` |
| Goldens | `harmonium_cli/tests/fixtures/profile_d2_main.json` : seul le champ `source` (`TUNING_VERSION:27`) change |

Outils jetables (hors dépôt, `/tmp/harmonium-019/r2-jeux/`) : `habillage.py` (habillages distincts), `symdist.py`
(distance symbolique rapide, identique à `style-distance` : `78,2 / 83,5 / 67,8` retrouvés à l'identique).

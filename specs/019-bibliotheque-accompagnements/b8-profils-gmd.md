# B8 — Profils rock, pop, hip-hop, soul, reggae, disco et afrobeat (Groove MIDI Dataset)

Date : 2026-11-03. Branche `wquintal/b8-styles-gmd`, partie de `main` à `0fe8b52` (`TUNING_VERSION` 26,
**inchangé**, aucun fichier de `src/` d'un crate moteur modifié). Sept profils distribués sous une grille d'accords,
des **données** seulement : le moteur de 019 (arc A3, phrases et relances A1, graine d'accompagnement S0) suffit.
L'habillage par graine d'A2 n'existe pas encore : il n'est pas simulé (tempo, instruments, kit et tonalité sont fixes
par profil), il sera ajouté en vague 4.

## Ce qui est livré

| Profil | Source (L2 § 4.2) | Fiabilité `beat` | Fichiers retenus (4/4 droits) | Batteurs | Tempo (centile 10 · médiane · centile 90) | Tonalité |
|---|---|---|---:|---:|---|---|
| `gmd_rock` | `rock` | assez | 154 | 9 | 90 · 105 · 130 | mi mixolydien |
| `gmd_pop` | famille `pop` (pop, pop/soft, pop/groove7) | juste (chacun) | 14 | 4 | 83 · 135 · 142 | sol ionien |
| `gmd_hiphop` | famille `hiphop` (hiphop, hiphop/groove6) | assez ; juste | 34 | 5 | 70 · 91 · 100 | do éolien |
| `gmd_soul` | famille `soul` (soul, motown, groove3/4/9/10) | juste (chacun) | 28 | 4 | 80 · 102 · 105 | mi♭ ionien |
| `gmd_reggae` | `reggae` | juste (3 fichiers) | **2** | **2** | 83 · 102 · 121 | la ionien |
| `gmd_dance_disco` | `dance/disco` | juste | 5 | 2 | 120 · 120 · 137 | la éolien |
| `gmd_afrobeat` | `afrobeat` | juste | 13 | 2 | 86 · 98 · 113 | ré dorien |

Pour chaque profil (dossier `profiles/<id>/`) : `profile.toml`, `percussion.toml` (généré), `arc.toml`, `dynamics.toml`,
`arrangement.toml`, `set.toml`, `cells.toml` (banque minimale jamais jouée), `README.md` (mentions de licence, grilles,
mesures → paramètres, groupes de batterie retenus, mixage, variété, distance, limites).

Générateur et preuves : `specs/019-bibliotheque-accompagnements/b8/` — `generer.py` (écrit tout, README compris),
`gmd_batterie.py` (batterie depuis le GMD), `styles.py` (données de basse, comping, grilles, instruments), `emit.py`,
`calibrer_mix.py`, `resumer_variete.py`, `distance_gmd.py`, `mix.json`, `preuves/*.json`. Tests :
`harmonium_core/tests/gmd_styles_profile_tests.rs` (10 tests). Dumps `harmonium_determinism` : 42 fichiers ajoutés
(7 profils × 3 graines × 2 réglages d'émotions), aucun changement de code. Le studio liste les profils sans modification
(`StyleProfile::distributed()`).

### Styles écartés

| Style GMD | Fiabilité `beat` (L2) | Décision |
|---|---|---|
| `punk` | juste (6 fichiers) | **coupé** : un seul batteur (« un style à un batteur décrit un batteur », L2 § 4.1) ; pas un style d'iReal Pro |
| `country` | trop peu (2 fichiers) | non fait ; relances « juste » (27 fichiers) mais pas de `beat` suffisant |
| `gospel`, `highlife`, `dance/breakbeat`, `middleeastern`, `reggae/slow`, `rock/*` sauf groupes `rock` | trop peu (1 à 2 fichiers) | non regroupés avec leur parent : la source ne le justifie pas (un fichier = un batteur) |
| `rock/shuffle`, `blues/shuffle`, `funk/*`, `neworleans/*`, `jazz/*`, `latin/*`, `afrocuban/*` | — | autres tâches (B1–B7) |

Reggae, disco et afrobeat sont « juste » par le critère de L2 (fichiers et mesures) mais **à deux batteurs ou moins** : ils
sont livrés avec cette limite écrite dans leur README (la variété de batterie qu'ils portent est celle de deux batteurs).

## Construction

### Batterie (GMD, CC BY 4.0) — mesurée

`gmd_batterie.py` relit le GMD en cache vérifié du labo (`corpus/gmd/raw/`, SHA256 contrôlé) avec les fonctions de L2
(`analyse_archive`, `settle`) ; il n'écrit rien dans le labo. Même construction que `gmd_styles.proposed_patterns` :
chaque coup est un (rôle, pas) dont la probabilité par pas dans un **groupe de mesures** atteint le seuil de la densité
(0 : timekeeper ≥ 0,5 ; 1 : + pédale, ouverte, grosse caisse, caisse claire, cross-stick ≥ 0,5 ; 2 : grosse caisse, caisse
claire, cross-stick dès 0,25, charleston ouverte dès 0,35, crash ≥ 0,5). Aucune mesure d'enregistrement n'est copiée.

Écarts à L2, et pourquoi :

- **Groupes** : mêmes pas exacts du timekeeper (`pas:…`) **ou** figure nette (croches, doubles croches, noires,
  noires plus). Les figures `sparse` et `other` (hétérogènes : la moyenne seuillée n'y a pas d'ossature, essayé : un
  groupe `hat_closed:sparse` du rock donnait grosse caisse au pas 0, charleston aux pas 2, 3, 6, 10) sont écartées.
  Les groupes de mêmes mesures sont fusionnés ; 6 à 8 groupes au plus par profil.
- **Seuils d'éligibilité par profil** (fichiers, mesures, part maximale d'un fichier), écrits dans chaque README :
  ceux de L2 (3 fichiers, 20 mesures, 60 %) ne laissent aucun groupe à pop, reggae, disco, afrobeat.
- **Niveaux de rendu** : les vélocités du GMD sont celles des pads d'un Roland TD-11 (grosse caisse médiane 29 à 78,
  caisse claire 91 à 122) ; elles ne se transposent pas. Chaque voix a un niveau de service (`RENDER_LEVEL`), seule la
  **forme des accents** (vélocité relative du groupe) est gardée (`delta_velocity`).
- **Phrases (A1)** : suites de groupes de 2 et 4 mesures (alignées sur le début du fichier) vues ≥ 3 fois dans ≥ 2
  fichiers (≥ 2 occurrences si la suite change de groupe). C'est la mesure que `a1-phrases.md` demandait au labo, faite ici.
  Résultat : l'essentiel des suites sont des **grooves tenus** (un batteur ne change pas de groove sur 4 mesures) ; les
  groupes sans aucune suite mesurée reçoivent une phrase « groove tenu 4 mesures » (règle de métier, dite dans le
  `percussion.toml` et le README) sans quoi ils ne seraient jamais tirés.
- **Relances (A1)** : fichiers `fill` de 1 ou 2 mesures groupés par longueur et pas de départ (0, 4, 8, 12), moyenne
  seuillée (au moins 2 fichiers sur 3 d'accord dans un petit groupe, 0,25 au-delà), `start_step` = le pas de départ du
  groupe, `part_end` pour les relances de deux mesures. `fills_none_weight` : valeur de service (la part de fins de phrase
  sans relance n'est pas mesurée). Disco et afrobeat n'ont **aucun fichier `fill`** : pas de relance.
- Relances et phrases sont étiquetées comme les patterns (`hat`, `ride`, `open`, le timekeeper du groupe) ; **une relance
  est recopiée par étiquette que l'arc nomme** (voir manques).

### Basse, comping, grilles — règles de métier, dites comme telles

Aucune source mesurée de basse ni de voicings pour ces styles (WJazzD ne mesure que le jazz ; décision F3 pour les
voicings) : motifs de basse, rythmes de comping et boucles de grille sont écrits à la main dans `styles.py`, jugés
seulement par des contrats de cohérence avec l'accord publié. Chaque README le dit.

- **Grilles par défaut** (D4 : aucune réharmonisation) : deux parties de 8 mesures (A et B), 3 ou 4 boucles pondérées par
  partie, progressions d'accords de forme commune écrites relatives à la tonique (I–V–vi–IV et ses rotations, I–IV–V,
  cycles ii–V, vamps mineurs à septièmes, vamps à un ou deux accords pour l'afrobeat). Droits : même texte que
  `wjazzd_swing` (Compendium du US Copyright Office §802.5(A), « chord symbols based on standard chord progressions ») ;
  aucune œuvre n'est reproduite ni nommée.
- **Basse** : 5 ou 6 motifs d'une ou deux mesures par profil (`reference = "chord_root"`, tirés par poids, un plan par air),
  plus un gabarit à relance vers la fondamentale suivante (rock, pop, soul). Instruments GM de GeneralUser GS :
  basse au médiator 34 (rock), au doigt 33 (pop, soul, reggae, disco, afrobeat), synthétique 38 (hip-hop).
- **Comping** : 8 rythmes d'une mesure par profil, étiquetés `sparse` / `medium` / `dense` (pour l'arc), 12 phrases de 2 à 4
  mesures dont 4 de mise en place `part_end`. Instruments : guitare saturée 29 (rock, accords grattés vers le bas), piano 0
  (pop), piano électrique « Tine » 4 (hip-hop, soul, voicings `drop2`), guitare claire 27 (reggae : skank vers le haut sur
  trois cordes ; disco : contretemps et doubles croches ; afrobeat : `guide_tones`, doubles croches). Kits : Room 8 (rock),
  808/909 25 (hip-hop), standard 0 (les autres).
- **Arc (A3)** : quatre chorus par air (un air par set : fin connue, donc retombée) ; exposition (comping clairsemé,
  ossature à la charleston fermée, doux), corps (deux chorus : ride ou charleston ouverte selon les groupes, variante
  forte avec break d'une mesure), sommet (au-delà de quatre chorus, stop-time une fois sur deux), retombée (comping clairsemé,
  charleston seule, stop-time sur les deux dernières mesures). Valeurs de service.
- **Mixage** : `calibrer_mix.py` règle les niveaux de piste hors ligne (LUFS intégré en solo) sur −26 (accords), −25 (basse),
  −25,5 (batterie) ; mêmes cibles que `wjazzd_swing` pour les accords et la basse.

## Preuves chiffrées

### Variété entre graines et entre chorus (D6)

`harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `--engine-bin` de la branche lancé depuis
`~/Project/bascanada/harmonium`), résumé par `resumer_variete.py` (`preuves/variete.json`). Chaque cellule : **mesures changées
d'un chorus au suivant / suites distinctes sur 16 graines** (chorus égal, place égale).

| Profil | comping rythme | comping voicings | basse | batterie | accompagnement | accords |
|---|---|---|---|---|---|---|
| `gmd_rock` | 83 % / 16 | 84 % / 16 | 3 % / 16 | 83 % / 16 | 94 % / 16 | 0 % / 10 |
| `gmd_pop` | 83 % / 16 | 82 % / 16 | 3 % / 16 | 84 % / 16 | 95 % / 16 | 0 % / 12 |
| `gmd_hiphop` | 83 % / 16 | 81 % / 16 | 3 % / 16 | 82 % / 16 | 95 % / 16 | 0 % / 10 |
| `gmd_soul` | 88 % / 16 | 84 % / 16 | 3 % / 16 | 61 % / 16 | 90 % / 16 | 0 % / 11 |
| `gmd_reggae` | 87 % / 16 | 86 % / 16 | 3 % / 16 | 63 % / 16 | 93 % / 16 | 0 % / 9 |
| `gmd_dance_disco` | 83 % / 16 | 83 % / 16 | 3 % / 16 | 67 % / 16 | 88 % / 16 | 0 % / 9 |
| `gmd_afrobeat` | 87 % / 16 | 87 % / 16 | 3 % / 16 | 79 % / 16 | 95 % / 16 | 0 % / 9 |

Repère V0 (`wjazzd_swing`, avant A3 et S0, grille par défaut) : comping 2 suites distinctes sur 16 graines, basse 2,
batterie 11, accompagnement 14, et **100 % de chorus identiques pour la batterie**. Ici toutes les couches actives
atteignent 16 / 16 graines distinctes, et la batterie change de 61 à 84 % de ses mesures d'un chorus au suivant (arc, phrases,
relances), 0 % de paires de chorus identiques (reggae et soul 1 %, disco 17 % : deux chorus de même exposition/retombée).
Seuils V2 proposés par V0 (`references/v0-variance/README.md`) : séquences distinctes ≥ 12 / 16 comping et batterie
(**acquis partout**), paires de chorus de batterie identiques ≤ 5 % (acquis sauf disco, 17 % : un seul groupe domine,
deux timekeepers seulement).

**Ce qui ne varie pas** (voir manques) : la **basse** (3 % de mesures changées d'un chorus au suivant : un plan de motifs
par air, celui des motifs `bars`/`templates` d'S0, pas de tirage par chorus) et les **accords** (une boucle par partie et
par air, tenue à chaque chorus : 0 % ; d'une graine à l'autre 9 à 12 suites distinctes sur 16, il n'y a que 3 ou 4 boucles par
partie). L'orchestration ne varie pas (1 / 16) : A2 n'existe pas.

### Distance de la batterie au GMD, contre une copie non calibrée

`make style-distance` mesure un profil contre des **pistes audio** de référence et leurs transcriptions : le GMD est du MIDI de
batterie, sans audio ni transcription de basse, l'outil ne s'y applique pas. `distance_gmd.py` reprend ses métriques de
batterie (`drums_any`, `drums_kick`, `drums_snare`, `drums_hat` de `styledist/symbolic.py` : Bray-Curtis des fréquences
d'attaque par pas, 0 = identique) contre **toutes** les mesures `beat` 4/4 droites du style (pas seulement les groupes retenus), sur
16 graines × 47 mesures rendues par le profil tel que livré. **Copie non calibrée** : le même profil (grille, basse, comping) avec le
`percussion.toml` de `wjazzd_swing` (batterie jazz calibrée pour un autre style) et sans arc. « Sans arc » : le profil sans
`[arc]`, pour isoler les patterns.

| Profil | tout | grosse caisse | caisse claire | charleston | **moyenne** | moyenne sans arc | copie non calibrée |
|---|---|---|---|---|---|---|---|
| `gmd_rock` | 0,085 | 0,218 | 0,243 | 0,373 | **0,230** | 0,245 | 0,763 |
| `gmd_pop` | 0,084 | 0,301 | 0,211 | 0,253 | **0,212** | 0,208 | 0,750 |
| `gmd_hiphop` | 0,080 | 0,181 | 0,212 | 0,125 | **0,149** | 0,166 | 0,799 |
| `gmd_soul` | 0,087 | 0,346 | 0,309 | 0,397 | **0,285** | 0,214 | 0,784 |
| `gmd_reggae` | 0,169 | 0,278 | 0,478 | 0,078 | **0,251** | 0,239 | 0,780 |
| `gmd_dance_disco` | 0,089 | 0,210 | 0,151 | 0,206 | **0,164** | 0,113 | 0,824 |
| `gmd_afrobeat` | 0,060 | 0,339 | 0,329 | 0,116 | **0,211** | 0,158 | 0,799 |

La copie jazz est à 0,75–0,82 (pas de grosse caisse ni de caisse claire à la densité 1, ride à la place de la charleston).
L'arc éloigne un peu la batterie du corpus pour le soul (charleston 0,40 contre 0,12 : l'exposition et la retombée ne
jouent que la charleston) et la rapproche pour le rock : choix de service. Le ride (non compté dans la moyenne : absent de la
plupart des styles) vaut 1,0 pour hip-hop, reggae et disco, dont les groupes n'ont pas de ride.

### Mixage (LUFS intégré en solo, graines 1 et 2, 48 mesures)

| Profil | accords | basse | batterie | mix |
|---|---|---|---|---|
| `gmd_rock` | −26,0 | −25,0 | −24,9 | −21,4 |
| `gmd_pop` | −26,1 | −25,2 | −23,9 | −20,9 |
| `gmd_hiphop` | −25,9 | −25,0 | −24,8 | −21,1 |
| `gmd_soul` | −26,2 | −25,0 | −25,4 | −21,4 |
| `gmd_reggae` | −26,1 | −25,1 | −26,2 | −22,0 |
| `gmd_dance_disco` | −26,2 | −25,2 | −25,4 | −21,0 |
| `gmd_afrobeat` | −26,1 | −25,2 | −26,7 | −21,9 |

`wjazzd_swing` : accords −25,8, basse −25,4, mix −22,2. Crête vraie du mix (graine 3, 48 mesures, densités 1 et 2) :
au plus −1,1 dBFS (pop, densité 2) ; les autres de −2,9 à −6,8.

### Contrats (`gmd_styles_profile_tests.rs`, flux de `MeasureSnapshot`)

| Contrat | Test |
|---|---|
| instruments GM (programmes, banque 128), aucune zone Odin, arc présent | `les_instruments_sont_ceux_de_generaluser_gs_sans_odin` |
| grille par défaut : chaque graine joue une des boucles écrites (relatives à la tonique) de A puis de B ; un accord par mesure, reprise identique au chorus suivant ; ≥ 2 boucles distinctes sur 16 graines | `la_grille_par_defaut_est_une_boucle_ecrite_…` |
| tempo : plage de la source ; feel droit (0,5) ; tempo des snapshots dans la plage à toutes les intensités | `le_tempo_et_le_feel_sont_ceux_de_la_source` |
| basse : intervalle à la basse publiée dans l'ensemble écrit du style, fenêtre, arrivée du premier temps sur la basse publiée | `la_basse_joue_l_accord_publie_…` |
| comping : hauteurs ⊂ notes de l'accord publié (accord suivant pour une attaque qui finit sur la barre), tessiture, ≥ 150 attaques | `le_comping_ne_joue_que_des_notes_de_l_accord_publie` |
| batterie : voix sur les pistes grave et aiguë, densité 0 = timekeeper seul, une caisse claire ou un cross-stick dans ≥ 60 % des mesures dès la densité 1, coups croissants avec la densité | `la_batterie_monte_de_la_charleston_…` |
| relances : des toms à la densité 2 (styles à fichiers `fill`), aucune à la densité 0, aucune du tout sans fichier `fill` | `les_relances_de_fin_de_phrase_…` |
| arc : retombée sans grosse caisse, comping clairsemé, stop-time sur les deux dernières mesures ; exposition avec grosse caisse et comping ≤ 5 attaques ; corps plus fort que l'exposition et la retombée (vélocité de basse) ; ride au corps | `l_arc_joue_exposition_corps_et_retombee_…` |
| même graine, même flux | `le_profil_rejoue_la_meme_graine_a_l_identique` |
| 16 graines : suites distinctes de comping, de basse, de batterie (seuils par style), ≥ 8 formes (boucles et niveau des chorus) | `seize_graines_donnent_des_accompagnements_distincts` |

Mutations vérifiées (chacune fait échouer exactement le test attendu, les neuf autres passent) : une fondamentale de la
boucle A de `gmd_rock` décalée (grille), une quinte de basse de `gmd_pop` en triton (basse), la retombée de `gmd_rock` sans
stop-time sur l'avant-dernière mesure (arc), le swing de `gmd_dance_disco` à 0,55 (feel), le corps de l'arc de `gmd_rock`
à la charleston au lieu du ride (arc). Le contrat de comping n'a pas de mutation simple (les voicings `drop2`, `close_triad` et
`guide_tones` sont tous des notes de l'accord publié) : il est exercé par plus de 150 attaques par profil (trois graines, un air chacune).

### Suites

| Vérification | Résultat |
|---|---|
| `harmonium_determinism`, `main` (`git archive` de `0fe8b52`, cible séparée) contre la branche | **98 fichiers identiques octet pour octet** ; 42 fichiers ajoutés (140 au total) |
| `harmonium_determinism/scripts/run.sh` (wasm-bindgen du harnais, `/tmp/harmonium-019/wb/bin`) | **natif = wasm : 140 / 140** |
| `TUNING_VERSION`, goldens, `profile_d2_main.json` | inchangés (aucun profil existant touché) |
| `cargo test --workspace -q --no-fail-fast` | voir « Suites complètes » en fin de document |
| `cargo test -p harmonium --features odin` ; `cargo test -p harmonium_bevy` | verts (11 tests Bevy) |
| `cargo check -p harmonium_studio` ; `web_resources.py check-profiles` | passent ; 17 presets Odin, aucun nouveau |
| `cargo check --target wasm32-unknown-unknown` (`harmonium_core`, `harmonium_audio`, `harmonium`, avec et sans `odin` ; `harmonium_bevy --no-default-features`) | passent |
| seek `seek_to_bar_64_under_50ms` (profil par défaut `reel_quebecois`, aucun code de génération touché) | 26,4 ms au meilleur de 3 passages (machine chargée par d'autres agents : 172 et 243 ms aux deux premiers) ; référence de la phase 24–26 ms |

## Manques du schéma et du moteur rencontrés (non contournés)

1. **Accords de puissance** : aucune qualité « 5 » (fondamentale et quinte). La guitare saturée du rock joue des triades
   majeures et mineures doublées à l'octave ; le punk (coupé) en aurait eu besoin.
2. **Basse sans étiquette ni arc** : `bass_feel` exige la marche (`walking`) ; les motifs de basse n'ont ni `tag` ni restriction
   par l'arc, et le plan de motifs est fixé **par air** : 3 % seulement des mesures de basse changent d'un chorus au suivant
   (tableau de variété). Il faudrait des motifs étiquetés (clairsemé → dense) que l'arc restreint par chorus, et un tirage par chorus.
3. **Notes fantômes, croches de passage, relances de basse au-delà du `pickup`** : seul le gabarit à `pickup` approche la
   fondamentale suivante ; reste d'A4. Aucune source mesurée de basse pour ces styles.
4. **Boucle d'accords fixe par air** : une boucle est tirée par partie et par air, puis tenue à chaque chorus (0 % d'accords
   changés d'un chorus au suivant). Une alternance de boucles par chorus (`tour`) n'est pas exprimable.
5. **Relance à étiquettes multiples** : l'arc ne laisse jouer, sous un tag nommé, que les relances qui portent ce tag ; il faut
   **recopier chaque relance par étiquette** (rock : 6 groupes × 2 étiquettes × 2 densités = 24 entrées).
   Une relance `tags = [...]` ou « toutes étiquettes » éviterait la copie. Une phrase de batterie ne porte de même qu'une
   étiquette (celle de la majorité de ses groupes).
6. **Niveau de densité de batterie par variante d'arc** : le niveau (0, 1, 2) est l'arrondi de trois contributions (densité du jeu +
   `percussion_intensity` + `density_percussion`) ; régler l'exposition au niveau 1 a demandé de chercher par essais les
   multiplicateurs de l'arc (`intensity = 0,65`, `density = 0,9`). Un champ `percussion_density` de la variante serait direct.
7. **Micro-timing par voix de batterie** (L2 § 6, point 3) : `[performance.tracks.*]` n'existe que pour les pistes ; le
   retard de la caisse claire et de la charleston du GMD (−7 à +14 ms) et le retard du reggae ne s'expriment pas.
8. **Décalage de tempo borné à ±30 BPM** (`tempo_offset`) : la plage de pop (83–142) n'est pas parcourable par l'arousal ; le
   tempo par défaut (135) est fixe (A2 le tirera dans la plage).
9. **Un seul instrument d'accords** : pas d'orgue « bubble » avec le skank du reggae, ni de cordes avec la guitare du disco.
10. **Pas de probabilité par pas en batterie** (L2 § 6, point 5) : les notes fantômes sont déterministes (seuil de
    probabilité), pas jouées avec la probabilité mesurée.
11. **Mixage et vélocités** : aucune mesure de corpus ; la grosse caisse est 7 à 8 LU sous le reste du kit dans soul et pop
    (LUFS pondéré K : la basse fréquence compte peu ; niveau de voix non touché).
12. **Hors périmètre** : `harmonium_studio/tests/live_screen_tests.rs` — le test du sélecteur de style cliquait « Valse
    québécoise » sans défilement ; la liste compte désormais 23 profils (400 px de hauteur de liste) et l'entrée sort de la
    zone cliquable. Correction minimale : `scroll_to_me()` suivi de huit `step()` avant le clic (une `step()` ne suffit pas,
    le défilement est animé). B4 + B7 ajouteront d'autres profils : la même correction vaudra.

## Limites

- **Basse, comping, grilles : règles de métier**, jugées sans écoute (D5) : leurs contrats vérifient la cohérence avec
  l'accord publié, pas une ressemblance à un corpus. Les rythmes de comping et motifs de basse sont des figures
  classiques du style ; leur poids dans le tirage est de service.
- **Reggae** : le groupe 4/4 droit n'a que deux fichiers et deux batteurs, dont la grosse caisse tombe sur les quatre temps
  (« steppers ») plutôt qu'en « one drop » ; la basse et la guitare, elles, suivent le one drop (temps 1 vide, skank).
- **Disco, afrobeat** : deux batteurs, pas de relance (aucun fichier `fill`) ; un groupe domine le disco (114 mesures sur 3 fichiers).
- **Pop** : deux régimes de tempo (83 pour `pop/soft`, 132–142 pour `pop` et `groove7`).
- **Hip-hop** : un seul timekeeper (charleston fermée) ; les dix fichiers à swing léger sont écartés (feel droit seulement).
- **GMD** : kit électronique Roland TD-11, dix batteurs en tout (L2 § 7) ; relances de petits groupes (1 ou 2 fichiers par
  position pour certaines) : seuil de 2 fichiers sur 3.
- Rien de **training** dans ce dépôt (constitution 4.5.0) ; la table style iReal → profil est T1.

## Reproduire

```sh
cd ~/orca/workspaces/harmonium/b8-styles-gmd
LAB=~/Project/bascanada/harmonium_lab
$LAB/.venv/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py          # réécrit les sept profils (lit le GMD en cache vérifié)
python3 specs/019-bibliotheque-accompagnements/b8/calibrer_mix.py [--apply] [--write-lufs]   # niveaux de piste hors ligne
$LAB/.venv/bin/harmonium-lab seed-variety --profile profiles/gmd_rock --seeds 16 --chorus 4 \
    --engine-dir ~/Project/bascanada/harmonium --engine-bin target/release/harmonium-cli --label B8 -o /tmp/var/gmd_rock.json
python3 specs/019-bibliotheque-accompagnements/b8/resumer_variete.py /tmp/var specs/019-bibliotheque-accompagnements/b8/preuves/variete.json
(cd $LAB && .venv/bin/python <dépôt>/specs/019-bibliotheque-accompagnements/b8/distance_gmd.py --out <dépôt>/specs/019-bibliotheque-accompagnements/b8/preuves/distance-gmd.json)
cargo test -p harmonium_core --test gmd_styles_profile_tests
```

Le CLI se lance depuis `~/Project/bascanada/harmonium` (`assets/`) ; reconstruire après un changement de profil (profils
embarqués). Aucun rendu audio n'est écouté ni publié : tout est vérifié hors ligne, en silence.

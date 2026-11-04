# `gmd_hiphop` — accompagnement hip-hop sous une grille

Profil d'accompagnement **sous une grille d'accords** : basse synthétique tenue (programme 38), piano électrique en voicings à quatre sons, batterie hip-hop du GMD (kit 808/909). Sans grille fournie : deux parties de huit mesures de boucles mineures à septièmes. Aucune mélodie.

**Nom (principe VII).** Le profil nomme la source qui définit son style : la famille `hiphop` du Groove MIDI Dataset v1.0.0 (`hiphop` : 30 fichiers, 796 mesures, « assez » ; `hiphop/groove6` : « juste » ; fills « juste », L2 § 4.2). 90 % des fichiers sont droits (80 sur 91 ; 10 swing, 1 ternaire), seuls les droits sont retenus. Ce n'est pas « le hip-hop » :
il décrit la batterie de 34 fichiers (cinq batteurs) ; la basse sub, le piano électrique et les boucles sont des règles de métier. Aucun échantillonnage, scratch ni voix.

## Mentions de licence

- **Groove MIDI Dataset v1.0.0** (Google LLC) — CC BY 4.0, <https://creativecommons.org/licenses/by/4.0/> : J. Gillick,
  A. Roberts, J. Engel, D. Eck, D. Bamman, « Learning to Groove with Inverse Sequence Transformations », ICML 2019.
  Les patterns, phrases et relances sont construits sur des probabilités par pas agrégées ; aucune mesure
  d'enregistrement. SHA256 de l'archive (vérifié par le labo avant toute lecture) :
  `651cbc524ffb891be1a3e46d89dc82a1cecb09a57c748c7b45b844c4841dcc1e`.
- Aucune autre source ; aucune source NC ni SA n'entre dans ce profil. Pas d'ODbL : WJazzD n'est pas lu.

Provenance chiffrée : `harmonium_lab/references/l2-ingestion.md` (§ 4.2), `references/gmd-*/`, `corpus/gmd/manifest.jsonl`.

## Grilles par défaut : provenance et droits

Quand l'utilisateur ne fournit pas de grille, le profil joue la partie A puis la partie B (8 mesures chacune ; une
boucle par partie est tirée par air selon la graine et les poids, puis tenue : un chorus fait 16 mesures). Les
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**do éolien** par défaut ; la variante
de session en tire une autre, voir « Habillage par graine »), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
2021-01-28) range les « chord symbols based on standard chord progressions » parmi le matériau non protégeable
(*common property*) ; aucune œuvre n'est reproduite, aucune n'est nommée.

| Partie | Poids | Progression (mesure par mesure) |
|---|---|---|
| A (8 mesures) | 3 | i7 bVI bIII bVII7 (deux fois) |
| A (8 mesures) | 3 | i7 i7 iv7 iv7 (deux fois) |
| A (8 mesures) | 2 | i7 bVII7 bVI V7 (deux fois) |
| A (8 mesures) | 1 | i7 bIII bVII7 iv7 i7 bIII bVII7 V7 |
| B (8 mesures) | 3 | bVI bVII7 i7 i7 (deux fois) |
| B (8 mesures) | 2 | iv7 i7 bVI V7 (deux fois) |
| B (8 mesures) | 2 | bIII bVII7 i7 i7 bIII bVII7 iv7 V7 |
| B (8 mesures) | 1 | i7 (quatre mesures) bVI bVI bVII7 bVII7 |



## Écriture

4/4, 16 pas par mesure, une pulsation de 4 pas, feel droit (rapport de swing 0,5 : le GMD mesure 0,50 (médiane de 34 fichiers droits ; 10 fichiers à swing léger écartés)). Tempo
91 noires/min par défaut, plage 70 à 100 : tempo des 34 fichiers `beat` du GMD (BPM du nom de fichier) : centile 10 = 70, médiane = 91, centile 90 = 100. Un chorus = 16 mesures ; 4 chorus par air
(un air par set) : le nombre de chorus est connu, la retombée de l'arc joue au dernier.

| Couche | Réglage |
|---|---|
| Basse | basse synthétique (programme 38), cinq motifs d'une ou deux mesures tirés par poids (sub tenu, sub syncopé, octave en contretemps, note tenue, deux mesures avec quinte) ; fenêtre 24..35 (do grave) |
| Accords | piano électrique « Tine » (programme 4), voicings `drop2` à quatre sons dans la fenêtre 52..76 ; huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense`, douze phrases de 2 à 4 mesures, quatre de mise en place `part_end` |
| Batterie | kit GeneralUser GS banque 128, programme 25 (kit 808/909) ; 20 patterns, 29 phrases distinctes, 4 relances ; densité 0 timekeeper, 1 ossature, 2 notes fantômes |
| Arc | exposition, corps, sommet, retombée : `arc.toml` (valeurs de service) |
| Timing | basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service) |
| Mixage | master 20.0 dB, accords -13.4 dB, basse -11.5 dB, batterie -14.6 dB : réglages de service calés hors ligne (section Mixage) |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS (D1).

## Mesures → paramètres

| Paramètre | Valeur | Mesure | Source |
|---|---|---|---|
| `meter.default_bpm` / `min_bpm` / `max_bpm` | 91 / 70 / 100 | tempo des 34 fichiers `beat` (4/4 droit) : médiane et centiles 10 et 90 ; BPM du nom de fichier | GMD |
| `meter.default_swing_ratio` | 0,5 | rapport médian des fichiers retenus 0,50 (médiane de 34 fichiers droits ; 10 fichiers à swing léger écartés) | GMD |
| Patterns de batterie | 20 | groupes ci-dessous, seuils d'éligibilité du profil, seuils de probabilité de L2 | GMD |
| Phrases de batterie | 29 entrées (3 densités) | mesurées : suites de groupes vues ≥ 3 fois (≥ 2 si la suite change de groupe) dans ≥ 2 fichiers ; 11 phrases de groove tenu 4 mesures ajoutées pour les groupes sans suite mesurée (règle de métier) | GMD |
| Relances | 4 groupes × 1 étiquettes × 2 densités | fichiers `fill` (48) | GMD |
| Basse, comping, grilles | — | **aucune mesure** : règles de métier (sub tenu et syncopé, accords de touches clairsemés) | règles |
| Plages de tempo de la session, swing léger | voir « Habillage par graine » | centiles des BPM du GMD ; rapport médian des fichiers `swing` | GMD |
| Micro-timing par voix, coups probabilistes | voir « Habillage par graine » et `percussion.toml` | médianes en ms par rôle ; probabilité par pas du groupe | GMD |
| Tonalités, instruments, kits, voicings, registre, énergie, arcs nommés, mixage, vélocités | — | réglages de service (règles de métier) | — |

### Batterie : groupes retenus

Fichiers `beat` : 34, mesures 860, batteurs 5. Seuils d'éligibilité de ce profil :
au moins 2 fichiers, 15 mesures, aucun fichier au-delà de 80 % du groupe (34 fichiers mais cinq batteurs, groupes de gestes de charleston très partagés : seuils de L2 relâchés à deux fichiers).

| Groupe (timekeeper:figure ou pas exacts) | Étiquette | Mesures | Fichiers | Batteurs |
|---|---|---|---|---|
| `hat_closed:quarters_plus` | hat | 131 | 20 | 4 |
| `hat_closed:eighths` | hat | 116 | 9 | 4 |
| `hat_closed:sixteenths` | hat | 108 | 10 | 3 |
| `hat_closed:pas:0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15` | hat | 38 | 7 | 2 |
| `hat_closed:pas:0,1,4,5,8,9,12,13` | hat | 36 | 3 | 1 |
| `hat_closed:pas:0,1,4,5,9,12,13` | hat | 22 | 2 | 1 |
| `hat_closed:pas:4,5,6,7,8,9,10,11,12,13,14,15` | hat | 22 | 5 | 3 |


Groupes écartés (extraits) :

| Groupe | Mesures | Fichiers | Raison |
|---|---|---|---|
| `hat_closed:pas:0,2,4,6,8,10,14` | 16 | 3 | au-delà des 7 groupes les plus fournis |

Relances retenues :

| Longueur | Pas de départ | Fichiers | Batteurs | Seuil de probabilité |
|---|---|---|---|---|
| 1 mesure(s) | 0 | 26 | 1 | 0.25 |
| 1 mesure(s) | 4 | 4 | 1 | 0.5 |
| 1 mesure(s) | 8 | 7 | 1 | 0.29 |
| 2 mesure(s) | 0 | 5 | 1 | 0.4 |

## Mixage

Aucune mesure de corpus ne donne un mixage : les niveaux de piste sont calés hors ligne, en silence
(`calibrer_mix.py` : rendus WAV de 48 mesures, graines 1 et 2, LUFS intégré en solo par piste) sur les mêmes cibles
pour tous les profils `gmd_*` : accords −26 LUFS, basse −25, batterie (grosse caisse + reste du kit) −25,5, soit un mix
d'environ −21,5 LUFS (`wjazzd_swing` : −22,2).

| Piste en solo | LUFS intégré |
|---|---:|
| accords | -25.9 |
| basse | -25.0 |
| grosse caisse | -26.0 |
| reste du kit | -30.9 |
| batterie (somme en puissance) | -24.8 |
| mix complet | -21.1 |


## Habillage par graine (`session.toml`, A2 ; R2-styles)

Une graine tire **un habillage** : un tirage indépendant par groupe, tous les groupes se combinent (10 groupes,
37 variantes, **311040 habillages possibles**). Le tirage est stable par graine, rejoué après un seek, et **l'hôte gagne** : `SetBpm` impose le tempo, `SetSwing`
le swing, `[fixed_tonality]` du profil de l'hôte (la grille de training, transposée par lui) la tonalité (la variante est
alors ignorée) ; programmes, voicings, registre, énergie et arc n'ont pas de commande (un hôte qui n'en veut pas installe
un profil sans `[session]`). Ce profil n'a donc **plus de `[fixed_tonality]`** : la tonalité par défaut (do éolien) n'est
que la plus pondérée.

| Groupe | Variantes (poids) | Ce que règle le groupe | D'où vient la valeur |
|---|---|---|---|
| `tempo` | lent (2), medium (3), vif (1) | tempo (plage de BPM, un entier tiré) | GMD : centiles des BPM des fichiers `beat` |
| `feel` | droit (4), swing_leger (1) | rapport de swing | GMD : rapport médian des fichiers `swing` du style (patterns mesurés droits : voir limites) |
| `tonalite` | do_aeolian (3), la_aeolian (2), sol_aeolian (2), fa_aeolian (2), re_aeolian (1), mi_aeolian (1) | tonique et mode du premier air (grille relative transposée) | métier : tonalités courantes du style |
| `comping` | piano_electrique (4), piano (2), fm (1), vibraphone (1), nylon (1) | programme des accords | métier |
| `basse` | synth_1 (3), synth_2 (2), doigts (1) | programme de la basse | métier |
| `kit` | 808 (3), electronique (2), standard (1), room (1) | kit de la banque 128 | métier ; le GMD est joué sur un kit électronique Roland TD-11 |
| `voicing` | drop_2 (3), shell (2), structure_superieure (1), ouvert (1) | famille de voicings du comping | métier |
| `registre` | normal (4), haut (1), grave (1) | octave du comping | métier |
| `energie` | doux (1), normal (2), fort (1) | multiplicateurs d'intensité et de densité de base | service |
| `arc` | defaut (3), orchestre (2), direct (1), montee (1) | arc : `[arc]` du profil ou arc nommé de `session.toml` | service (aucun corpus ne donne l'arc d'un morceau) |

Les arcs nommés (`orchestre` : le comping et le kit changent de chorus en chorus et reviennent aux programmes de la session ;
`direct` : pas d'exposition ; `montee` : un cran par chorus) sont écrits dans `session.toml`, mêmes valeurs de service que
`arc.toml`. Tous les instruments sont des programmes General MIDI de GeneralUser GS (banque 0) et des kits de la banque 128
(0, 8, 16, 24, 25), vérifiés dans la banque ; aucune zone Odin (D1).

**Micro-timing par voix** (`performance.tracks.Percussion*.voices`, A6) : médianes de l'écart à la grille mesurées dans le
GMD (groupe 4/4 droit de `hiphop`, `hiphop/groove6`, pondérées par l'effectif), en **fraction de pas au tempo par
défaut (91 BPM)** : un pas = 15 000 / BPM ms ; les décalages mesurés en ms ne suivent donc pas exactement le tempo tiré
(à 130 BPM le même retard en ms vaut 24 % de pas de plus). `jitter` : amplitude d'un tirage uniforme dont l'écart-type est celui
mesuré, plafonnée à 0,1 pas (borne du schéma ; l'écart-type mesuré est de 0,11 à 0,20 pas).

| Voix | Note GM | Sur le temps (pas) | Hors temps (pas) | Jitter (pas) |
|---|---:|---:|---:|---:|
| kick | 36 | -0.096 | -0.070 | 0.10 |
| snare | 38 | -0.069 | +0.005 | 0.10 |
| hat_closed | 42 | -0.065 | -0.076 | 0.10 |
| tom_low | 43 | -0.125 | -0.125 | 0.10 |
| hat_pedal | 44 | -0.152 | -0.153 | 0.10 |
| hat_open | 46 | -0.025 | -0.054 | 0.10 |
| tom_mid | 47 | -0.065 | -0.065 | 0.10 |
| crash | 49 | -0.011 | +0.048 | 0.10 |
| tom_high | 50 | -0.121 | -0.121 | 0.10 |
| ride | 51 | -0.035 | -0.071 | 0.10 |

**Coups probabilistes** (`probability`, A6) : 68 coups de `percussion.toml` jouent avec la probabilité par pas mesurée dans le
groupe (grosse caisse, caisse claire, cross-stick entre 0,08 et le seuil de la densité ; notes fantômes à la densité 2),
au lieu d'un coup certain dès 0,25 (0,3 pour funk et New Orleans) ou absent : le tirage est sans état (graine, mesure, pas, note) donc stable par graine.

**Voicings** : `chords.color` (tension 9, 11 ou 13 que la qualité permet, jamais en conflit avec la grille, D4) et
`chords.register_drift` (dérive bornée du centre) sont déclarés ; ils n'agissent que sur les familles de voicings qui les
lisent (pas `close_triad`).


## Variété entre graines et entre chorus (D6)

Entre graines : une des quatre boucles de chaque partie, le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie (charleston en croches, doubles croches, noires plus) et leurs phrases, les relances. Entre chorus : l'arc.

Mesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) :

| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |
|---|---:|---:|---:|---:|---:|
| accompagnement | 99% | 0% | 4.0 | 16 / 16 | 66% |
| accords | 0% | 100% | 1.0 | 10 / 16 | 52% |
| basse | 2% | 53% | 1.81 | 16 / 16 | 92% |
| batterie | 85% | 0% | 4.0 | 16 / 16 | 50% |
| comping : rythme | 80% | 1% | 3.94 | 16 / 16 | 42% |
| comping : voicings | 92% | 1% | 3.94 | 16 / 16 | 97% |




## Distance de la batterie au GMD

`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), 16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la batterie de `wjazzd_swing` et sans arc.

| Voix | Profil | Profil sans arc | Copie non calibrée |
|---|---:|---:|---:|
| any | 0.091 | 0.094 | 0.368 |
| hat | 0.124 | 0.125 | 0.827 |
| kick | 0.202 | 0.211 | 1.000 |
| moyenne | 0.147 | 0.156 | 0.799 |
| ride | 1.000 | 1.000 | 0.877 |
| snare | 0.169 | 0.195 | 1.000 |

## Limites

- **Un seul timekeeper** : le hip-hop du GMD est joué à la charleston fermée ; les étiquettes d'arc n'ont donc que `hat` (pas de ride) et le corps de l'arc ne change pas de timekeeper.
- **Swing** : dix fichiers à swing léger écartés (feel droit seulement).
- **Basse et comping : règles de métier**, jugées à l'écoute seulement (D5 : aucune écoute exigée ; des preuves chiffrées,
  compte rendu `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`). WJazzD ne mesure que la basse du jazz ; aucun corpus distribuable de
  voicings ni de rythmes de comping (décision F3).
- **Batterie** : kit électronique Roland TD-11, dix batteurs en tout ; les vélocités et le micro-timing sont ceux du
  kit (niveaux recalés au kit de rendu). Un groupe de peu de fichiers décrit un batteur, pas un style (L2 § 4.1).
- **Manques du schéma** (comptes rendus B8 et R2-styles) : accords de puissance (« 5 »), options de basse d'A4
  (`walking.*` : notes fantômes, croches, notes répétées, arrivée) exclusives de `motifs` et de `templates` donc
  inutilisables avec des riffs, feel de basse et étiquettes d'arc pour les motifs de basse (`bass_feel` exige la marche),
  groupes de session indépendants (un kit « power » avec un tempo lent n'est pas exclu), boucle de grille fixe par air.

## Grille par chorus (A7, `harmony.tour_variants = 4`)

La grille tirée une fois par air et tenue à chaque chorus (0 % de mesures d'accord changées d'un chorus au suivant) est
remplacée par un tirage par chorus : le chorus `n` joue le tirage `n mod 4` (le premier est celui de l'air). Mesuré par
`harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus ; avant → après) : accords changés d'un chorus au suivant
0 % → 65 %, paires de chorus identiques 100 % → 2 %, suites d'accords distinctes sur 16 graines 10 / 16 → 16 / 16 ;
basse changée (elle suit la fondamentale des accords) 5 % → 66 %. Poids et progressions de métier, aucun corpus de grilles.

## Régénérer

`percussion.toml` est généré : `{venv du labo}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py gmd_hiphop`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).

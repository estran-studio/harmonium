# `gmd_rock` — accompagnement rock sous une grille

Profil d'accompagnement **sous une grille d'accords** pour le client externe `harmonium_training` : basse (motifs), guitare saturée (rythmes de comping, phrases) et batterie rock du GMD ne dépendent que de l'accord publié et de la graine. Sans grille fournie, il joue deux parties de huit mesures de forme commune. Aucune mélodie.

**Nom (principe VII).** Le profil nomme la source qui définit son style : le style `rock` du Groove MIDI Dataset v1.0.0 (fiabilité « assez » en `beat`, L2 § 4.2 : 193 fichiers, 9 batteurs, 5 367 mesures, dont 154 fichiers 4/4 droits retenus ici). Ce n'est pas « le rock » :
il décrit la batterie de ce style telle que les dix batteurs du GMD la jouent sur un kit électronique ; la guitare et la basse sont des règles de métier.

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
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**mi mixolydien** par défaut ; la variante
de session en tire une autre, voir « Habillage par graine »), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
2021-01-28) range les « chord symbols based on standard chord progressions » parmi le matériau non protégeable
(*common property*) ; aucune œuvre n'est reproduite, aucune n'est nommée.

| Partie | Poids | Progression (mesure par mesure) |
|---|---|---|
| A (8 mesures) | 3 | I I IV IV I I V IV |
| A (8 mesures) | 3 | I V vi IV (deux fois) |
| A (8 mesures) | 2 | I bVII IV I | I bVII IV V (mode mixolydien) |
| A (8 mesures) | 2 | vi IV I V (deux fois) |
| B (8 mesures) | 3 | IV V I I IV V vi V |
| B (8 mesures) | 2 | IV I V vi IV I V V |
| B (8 mesures) | 2 | I IV V IV (deux fois) |
| B (8 mesures) | 1 | bVI bVII I I (deux fois) |



## Écriture

4/4, 16 pas par mesure, une pulsation de 4 pas, feel droit (rapport de swing 0,5 : le GMD mesure 0,50 (médiane des 161 fichiers droits)) ; la variante de session `swing_leger` a été retirée en phase 020 (Q0 : croches en contretemps retardées de ~50 ms pendant que les doubles croches de la batterie et de la guitare restaient droites). Tempo
105 noires/min par défaut, plage 90 à 130 : tempo des 154 fichiers `beat` du GMD (BPM du nom de fichier) : centile 10 = 90, médiane = 105, centile 90 = 130. Un chorus = 16 mesures ; 4 chorus par air
(un air par set) : le nombre de chorus est connu, la retombée de l'arc joue au dernier.

| Couche | Réglage |
|---|---|
| Basse | basse au médiator (programme 34), six motifs d'une ou deux mesures tirés par poids (croches à la fondamentale, noires, octave en contretemps, syncope, deux mesures avec la quinte au temps 4, note tenue) et un gabarit de croches avec relance vers la fondamentale de l'accord suivant ; fenêtre des fondamentales 28..39 (mi grave) |
| Accords | guitare saturée (programme 29), triade doublée à 4 sons conduite par les voix communes, grattée vers le bas (écart 7 ms) ; huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense`, douze phrases de 2 à 4 mesures dont quatre de mise en place `part_end` |
| Batterie | kit GeneralUser GS banque 128, programme 8 (kit « Room ») ; 22 patterns, 38 phrases distinctes, 6 relances ; densité 0 timekeeper, 1 ossature, 2 notes fantômes |
| Arc | exposition, corps, sommet, retombée : `arc.toml` (valeurs de service) |
| Timing | basse sur la grosse caisse, accords sur la caisse claire et le charleston (décalages par voix, Q0), jitter 0,02 pas ; batterie 0,07 pas |
| Mixage | master 20.0 dB, accords -19.8 dB, basse -10.6 dB, batterie -16.2 dB : réglages de service calés hors ligne (section Mixage) |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS (D1).

## Mesures → paramètres

| Paramètre | Valeur | Mesure | Source |
|---|---|---|---|
| `meter.default_bpm` / `min_bpm` / `max_bpm` | 105 / 90 / 130 | tempo des 154 fichiers `beat` (4/4 droit) : médiane et centiles 10 et 90 ; BPM du nom de fichier | GMD |
| `meter.default_swing_ratio` | 0,5 | rapport médian des fichiers retenus 0,50 (médiane des 161 fichiers droits) | GMD |
| Patterns de batterie | 22 | groupes ci-dessous, seuils d'éligibilité du profil, seuils de probabilité de L2 | GMD |
| Phrases de batterie | 42 entrées (3 densités) | mesurées : suites de groupes vues ≥ 3 fois (≥ 2 si la suite change de groupe) dans ≥ 2 fichiers ; 9 phrases de groove tenu 4 mesures ajoutées pour les groupes sans suite mesurée (règle de métier) | GMD |
| Relances | 6 groupes × 2 étiquettes × 2 densités | fichiers `fill` (82) | GMD |
| Basse, comping, grilles | — | **aucune mesure** : règles de métier (motifs de basse de rock, rythmes de guitare en noires, croches et syncopes) | règles |
| Plages de tempo de la session | voir « Habillage par graine » | centiles des BPM du GMD | GMD |
| Micro-timing par voix, coups probabilistes | voir « Habillage par graine » et `percussion.toml` | médianes en ms par rôle ; probabilité par pas du groupe | GMD |
| Tonalités, instruments, kits, voicings, registre, énergie, arcs nommés, mixage, vélocités | — | réglages de service (règles de métier) | — |

### Batterie : groupes retenus

Fichiers `beat` : 154, mesures 4440, batteurs 9. Seuils d'éligibilité de ce profil :
au moins 3 fichiers, 20 mesures, aucun fichier au-delà de 50 % du groupe (style le plus fourni du GMD : seuils de L2, part maximale d'un fichier resserrée).

| Groupe (timekeeper:figure ou pas exacts) | Étiquette | Mesures | Fichiers | Batteurs |
|---|---|---|---|---|
| `ride:quarters_plus` | ride | 403 | 33 | 8 |
| `ride:eighths` | ride | 218 | 34 | 7 |
| `hat_closed:quarters_plus` | hat | 144 | 34 | 6 |
| `hat_closed:eighths` | hat | 141 | 30 | 7 |
| `hat_closed:sixteenths` | hat | 102 | 11 | 4 |
| `hat_closed:quarters` | hat | 50 | 8 | 3 |
| `hat_open:quarters` | open | 42 | 8 | 5 |
| `ride:pas:0` | ride | 41 | 19 | 6 |


Groupes écartés (extraits) :

| Groupe | Mesures | Fichiers | Raison |
|---|---|---|---|
| `hat_closed:pas:0,2,4,6,8,10,12` | 38 | 18 | au-delà des 8 groupes les plus fournis |
| `hat_closed:pas:2,4,6,8,10,12,14` | 34 | 17 | au-delà des 8 groupes les plus fournis |
| `ride:pas:0,2,4,6` | 34 | 18 | au-delà des 8 groupes les plus fournis |
| `hat_closed:pas:0,2,6,8,10,14` | 29 | 4 | au-delà des 8 groupes les plus fournis |
| `ride:pas:2,4,6,8,10,12,14` | 26 | 16 | au-delà des 8 groupes les plus fournis |
| `hat_closed:pas:4,6,8,10,12,14` | 24 | 14 | au-delà des 8 groupes les plus fournis |
| `hat_open:pas:0` | 23 | 11 | au-delà des 8 groupes les plus fournis |
| `hat_closed:pas:0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15` | 22 | 8 | au-delà des 8 groupes les plus fournis |
| `hat_closed:pas:0,2,4,6` | 20 | 14 | au-delà des 8 groupes les plus fournis |

Relances retenues :

| Longueur | Pas de départ | Fichiers | Batteurs | Seuil de probabilité |
|---|---|---|---|---|
| 1 mesure(s) | 0 | 35 | 3 | 0.25 |
| 1 mesure(s) | 4 | 7 | 1 | 0.29 |
| 1 mesure(s) | 8 | 13 | 2 | 0.25 |
| 1 mesure(s) | 12 | 4 | 1 | 0.5 |
| 2 mesure(s) | 0 | 8 | 2 | 0.25 |
| 2 mesure(s) | 8 | 3 | 1 | 0.5 |

## Mixage

Aucune mesure de corpus ne donne un mixage : les niveaux de piste sont calés hors ligne, en silence
(`calibrer_mix.py` : rendus WAV de 48 mesures, graines 1 et 2, LUFS intégré en solo par piste) sur les mêmes cibles
pour tous les profils `gmd_*` : accords −26 LUFS, basse −25, batterie (grosse caisse + reste du kit) −25,5, soit un mix
d'environ −21,5 LUFS (`wjazzd_swing` : −22,2).

| Piste en solo | LUFS intégré |
|---|---:|
| accords | -26.0 |
| basse | -25.0 |
| grosse caisse | -29.2 |
| reste du kit | -26.9 |
| batterie (somme en puissance) | -24.9 |
| mix complet | -21.4 |


## Habillage par graine (`session.toml`, A2 ; R2-styles)

Une graine tire **un habillage** : un tirage indépendant par groupe, tous les groupes se combinent (9 groupes,
27 variantes, **13824 habillages possibles**). Le tirage est stable par graine, rejoué après un seek, et **l'hôte gagne** : `SetBpm` impose le tempo, `SetSwing`
le swing, `[fixed_tonality]` du profil de l'hôte (la grille de training, transposée par lui) la tonalité (la variante est
alors ignorée) ; programmes, voicings, registre, énergie et arc n'ont pas de commande (un hôte qui n'en veut pas installe
un profil sans `[session]`). Ce profil n'a donc **plus de `[fixed_tonality]`** : la tonalité par défaut (mi mixolydien) n'est
que la plus pondérée.

| Groupe | Variantes (poids) | Ce que règle le groupe | D'où vient la valeur |
|---|---|---|---|
| `tempo` | lent (2), medium (3), vif (2) | tempo (plage de BPM, un entier tiré) | GMD : centiles des BPM des fichiers `beat` |
| `tonalite` | mi_mixolydian (3), la_mixolydian (3), re_mixolydian (2), sol_mixolydian (2) | tonique et mode du premier air (grille relative transposée) | métier : tonalités courantes du style |
| `comping` | overdrive (4), saturee (2), claire (2), orgue_rock (1) | programme des accords | métier |
| `basse` | mediator (3), doigts (2) | programme de la basse | métier |
| `kit` | room (3), standard (2), power (2) | kit de la banque 128 | métier ; le GMD est joué sur un kit électronique Roland TD-11 |
| `voicing` | triades_serrees (3), ouvert (1) | famille de voicings du comping | métier |
| `registre` | normal (4), haut (1) | octave du comping | métier |
| `energie` | doux (1), normal (2), fort (1) | multiplicateurs d'intensité et de densité de base | service |
| `arc` | defaut (3), orchestre (2), direct (1), montee (1) | arc : `[arc]` du profil ou arc nommé de `session.toml` | service (aucun corpus ne donne l'arc d'un morceau) |

Les arcs nommés (`orchestre` : le comping et le kit changent de chorus en chorus et reviennent aux programmes de la session ;
`direct` : pas d'exposition ; `montee` : un cran par chorus) sont écrits dans `session.toml`, mêmes valeurs de service que
`arc.toml`. Tous les instruments sont des programmes General MIDI de GeneralUser GS (banque 0) et des kits de la banque 128
(0, 8, 16, 24, 25), vérifiés dans la banque ; aucune zone Odin (D1).

**Micro-timing par voix** (`performance.tracks.Percussion*.voices`, A6) : médianes de l'écart à la grille mesurées dans le
GMD (groupe 4/4 droit de `rock`, pondérées par l'effectif), en **fraction de pas au tempo par
défaut (105 BPM)** : un pas = 15 000 / BPM ms ; les décalages mesurés en ms ne suivent donc pas exactement le tempo tiré
(à 130 BPM le même retard en ms vaut 24 % de pas de plus). `jitter` : amplitude d'un tirage uniforme dont l'écart-type est celui
mesuré, plafonnée à 0,1 pas (borne du schéma ; l'écart-type mesuré est de 0,11 à 0,20 pas).

| Voix | Note GM | Sur le temps (pas) | Hors temps (pas) | Jitter (pas) |
|---|---:|---:|---:|---:|
| kick | 36 | -0.024 | -0.039 | 0.10 |
| sidestick | 37 | +0.017 | -0.038 | 0.10 |
| snare | 38 | -0.029 | -0.015 | 0.10 |
| hat_closed | 42 | -0.050 | -0.055 | 0.10 |
| tom_low | 43 | -0.037 | -0.080 | 0.10 |
| hat_pedal | 44 | -0.097 | +0.069 | 0.10 |
| hat_open | 46 | -0.048 | -0.017 | 0.10 |
| tom_mid | 47 | -0.081 | -0.066 | 0.10 |
| crash | 49 | +0.024 | +0.111 | 0.10 |
| tom_high | 50 | +0.180 | +0.056 | 0.10 |
| ride | 51 | +0.064 | +0.097 | 0.10 |
| ride_bell | 53 | -0.038 | -0.077 | 0.10 |

**Coups probabilistes** (`probability`, A6) : 152 coups de `percussion.toml` jouent avec la probabilité par pas mesurée dans le
groupe (grosse caisse, caisse claire, cross-stick entre 0,08 et le seuil de la densité ; notes fantômes à la densité 2),
au lieu d'un coup certain dès 0,25 (0,3 pour funk et New Orleans) ou absent : le tirage est sans état (graine, mesure, pas, note) donc stable par graine.

**Voicings** : `chords.color` (tension 9, 11 ou 13 que la qualité permet, jamais en conflit avec la grille, D4) et
`chords.register_drift` (dérive bornée du centre) sont déclarés ; ils n'agissent que sur les familles de voicings qui les
lisent (pas `close_triad`).


## Variété entre graines et entre chorus (D6)

Entre graines : une des quatre boucles de chaque partie (A et B), un des motifs de basse d'un plan de motifs par air, les rythmes et phrases de comping, les groupes de batterie et leurs phrases, les relances. Entre chorus d'une graine : l'arc (exposition, corps, sommet, retombée) change le comping, la batterie (charleston puis ride) et l'intensité ; les breaks et stop-time sont tirés par chorus.

Mesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) ; accords et basse **après A7** (`tour_variants = 4` : un tirage de grille par chorus, accords pleins ou de puissance ; motifs de basse étiquetés `sparse` / `medium` / `dense` que l'arc restreint, plan de basse tiré à chaque chorus ; avant A7 : accords 0 % changés d'un chorus au suivant, basse 6 %, 9 suites d'accords distinctes sur 16 graines) :

| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |
|---|---:|---:|---:|---:|---:|
| accompagnement | 100% | 0% | 4.0 | 16 / 16 | 70% |
| accords | 81% | 2% | 3.9 | 16 / 16 | 79% |
| basse | 72% | 3% | 3.8 | 16 / 16 | 89% |
| batterie | 91% | 0% | 4.0 | 16 / 16 | 58% |
| comping : rythme | 77% | 0% | 3.94 | 16 / 16 | 33% |
| comping : voicings | 89% | 0% | 4.0 | 16 / 16 | 89% |




## Distance de la batterie au GMD

`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), 16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la batterie de `wjazzd_swing` et sans arc.

| Voix | Profil | Profil sans arc | Copie non calibrée |
|---|---:|---:|---:|
| any | 0.060 | 0.050 | 0.339 |
| hat | 0.302 | 0.400 | 0.714 |
| kick | 0.153 | 0.202 | 1.000 |
| moyenne | 0.184 | 0.236 | 0.763 |
| ride | 0.168 | 0.160 | 0.579 |
| snare | 0.221 | 0.293 | 1.000 |

## Limites

- **Accords de puissance** (A7) : la moitié des grilles tirables est écrite en qualité « 5 » (fondamentale, quinte, octave) ; l'autre moitié garde les triades. Poids de métier, sans corpus.
- Le rock du GMD est joué majoritairement au ride et à la charleston fermée en croches ; les groupes de toms seuls (sans timekeeper) sont écartés.
- **Basse et comping : règles de métier**, jugées à l'écoute seulement (D5 : aucune écoute exigée ; des preuves chiffrées,
  compte rendu `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`). WJazzD ne mesure que la basse du jazz ; aucun corpus distribuable de
  voicings ni de rythmes de comping (décision F3).
- **Batterie** : kit électronique Roland TD-11, dix batteurs en tout ; les vélocités et le micro-timing sont ceux du
  kit (niveaux recalés au kit de rendu). Un groupe de peu de fichiers décrit un batteur, pas un style (L2 § 4.1).
- **Manques du schéma restants** (A7, `specs/019-bibliotheque-accompagnements/a7-manques-schema.md`) : options de basse d'A4
  (`walking.*` : notes fantômes, croches, notes répétées, arrivée) toujours exclusives de `motifs` et de `templates` donc
  inutilisables avec des riffs ; groupes de session indépendants (un kit « power » avec un tempo lent n'est pas exclu).

## Régénérer

`percussion.toml` est généré : `{venv du labo}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py gmd_rock`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).

<!-- q0:début -->
## Corrections Q0 (phase 020, `TUNING_VERSION` 34)

Compte rendu, mesures avant / après et limites : `specs/020-qualite-des-styles/q0-corrections-rapides.md`. Scripts : `specs/020-qualite-des-styles/q0/` (`appliquer.py` : accents et mise en place, `batterie.py` : groove de batterie).

- **Accents du comping** (règle de métier : temps 1, backbeat et temps 3 appuyés, croches de contretemps et doubles croches fantômes, anticipation poussée). Chaque coup garde la vélocité de 019 et reçoit un écart selon sa place, forme `rock_comp`, amplitude 24, décalage de niveau -11 (le niveau moyen est conservé : la calibration du mixage reste valable) ; appuyés : temps 1 (+0,9), temps 3 (+0,5), contretemps 2 et 4 (+0,7), attaque anticipée (+0,8) ; légers : croche de contretemps (-0,4), 1re double croche (-0,9), 3e double croche (-0,9) ; coups probabilistes : -0,9 de plus. Plage de vélocité des accords et de la basse : 32 à 118 (`velocity_min`, `velocity_max` d'A6).
- **Accents de la basse** (temps 1 appuyé, croches de contretemps et doubles croches plus légères) : forme `rock_bass`, amplitude 15, décalage -3 ; appuyés : temps 1 (+1), temps 3 (+0,5), attaque anticipée (+0,5) ; légers : croche de contretemps (-0,4), 1re double croche (-0,9), 3e double croche (-0,9) ; coups probabilistes : -0,9 de plus.
- **Groove de batterie** : phrases et patterns de 019 inchangés ; probabilité des coups secondaires 1, 1 note(s) fantôme(s) à 0,05, exposant des probabilités 2, phrases tenues ×2, part de fins de phrase sans relance ×4.
- **Mise en place** : basse sur les décalages de la grosse caisse (A6, par pas), accords sur la moyenne caisse claire / timekeeper ; `jitter` basse 0,02 et accords 0,02 pas ; `jitter` de la batterie 0,07 pas (écart-type intra-fichier du GMD) ; égrenage ancré sur la première corde (`anchor = 0`).

<!-- q0:fin -->

# `gmd_soul` — accompagnement soul sous une grille

Profil d'accompagnement **sous une grille d'accords** : basse au doigt, piano électrique « Tine » en voicings à quatre sons et batterie soul du GMD. Sans grille fournie : deux parties de huit mesures à septièmes. Aucune mélodie.

**Nom (principe VII).** Le profil nomme la source qui définit son style : la famille `soul` du Groove MIDI Dataset v1.0.0 (`soul` : 8 fichiers, 221 mesures ; `soul/motown` et les quatre `soul/groove*` : « juste » chacun ; fills « juste », L2 § 4.2). Les sous-styles sont réunis parce que L2 leur donne chacun « juste » et qu'ils sont sous la même famille ; le soul tient lieu de R&B (iReal « RnB ») faute de style `rnb` dans le GMD. Ce n'est pas « le soul » :
il décrit la batterie de 28 fichiers joués par quatre batteurs (dont les quatre séries « groove » d'un même protocole) ; piano électrique et basse sont des règles de métier.

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
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**mi bémol ionien** par défaut ; la variante
de session en tire une autre, voir « Habillage par graine »), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
2021-01-28) range les « chord symbols based on standard chord progressions » parmi le matériau non protégeable
(*common property*) ; aucune œuvre n'est reproduite, aucune n'est nommée.

| Partie | Poids | Progression (mesure par mesure) |
|---|---|---|
| A (8 mesures) | 3 | I vi ii V (deux fois) |
| A (8 mesures) | 2 | I I IV IV I vi ii V |
| A (8 mesures) | 2 | I7 IV7 I7 I7 IV7 IV7 I7 V7 (soul blues) |
| A (8 mesures) | 1 | ii V I vi ii V I I |
| B (8 mesures) | 3 | IV iii ii V (deux fois, la seconde sur V7sus4) |
| B (8 mesures) | 2 | IV V iii vi IV V I I |
| B (8 mesures) | 2 | vi ii V I (deux fois) |



## Écriture

4/4, 16 pas par mesure, une pulsation de 4 pas, feel droit (rapport de swing 0,5 : le GMD mesure 0,48 (médiane de 28 fichiers)). Tempo
102 noires/min par défaut, plage 80 à 105 : tempo des 28 fichiers `beat` du GMD (BPM du nom de fichier) : centile 10 = 80, médiane = 102, centile 90 = 105. Un chorus = 16 mesures ; 4 chorus par air
(un air par set) : le nombre de chorus est connu, la retombée de l'arc joue au dernier.

| Couche | Réglage |
|---|---|
| Basse | basse au doigt (programme 33), cinq motifs d'une ou deux mesures (racine-octave-racine-quinte, syncopé avec octave, noires, blanches, deux mesures) et un gabarit en cycle racine-octave-quinte avec relance ; fenêtre 28..39 |
| Accords | piano électrique « Tine » (programme 4), voicings `drop2` à quatre sons dans la fenêtre 52..76 ; huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense` (dont un « chank » en doubles croches), douze phrases de 2 à 4 mesures dont quatre `part_end` |
| Batterie | kit GeneralUser GS banque 128, programme 0 (kit standard) ; 16 patterns, 22 phrases distinctes, 1 relances ; densité 0 timekeeper, 1 ossature, 2 notes fantômes |
| Arc | exposition, corps, sommet, retombée : `arc.toml` (valeurs de service) |
| Timing | basse sur la grosse caisse, accords sur la caisse claire et le charleston (décalages par voix, Q0), jitter 0,02 pas ; batterie 0,045 pas |
| Mixage | master 20.0 dB, accords -14.4 dB, basse -14.6 dB, batterie -15.3 dB : réglages de service calés hors ligne (section Mixage) |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS (D1).

## Mesures → paramètres

| Paramètre | Valeur | Mesure | Source |
|---|---|---|---|
| `meter.default_bpm` / `min_bpm` / `max_bpm` | 102 / 80 / 105 | tempo des 28 fichiers `beat` (4/4 droit) : médiane et centiles 10 et 90 ; BPM du nom de fichier | GMD |
| `meter.default_swing_ratio` | 0,5 | rapport médian des fichiers retenus 0,48 (médiane de 28 fichiers) | GMD |
| Patterns de batterie | 16 | groupes ci-dessous, seuils d'éligibilité du profil, seuils de probabilité de L2 | GMD |
| Phrases de batterie | 23 entrées (3 densités) | mesurées : suites de groupes vues ≥ 3 fois (≥ 2 si la suite change de groupe) dans ≥ 2 fichiers ; 5 phrases de groove tenu 4 mesures ajoutées pour les groupes sans suite mesurée (règle de métier) | GMD |
| Relances | 1 groupes × 2 étiquettes × 2 densités | fichiers `fill` (33) | GMD |
| Basse, comping, grilles | — | **aucune mesure** : règles de métier (figures de basse soul (octave, quinte), « chank » du clavier électrique) | règles |
| Plages de tempo de la session, swing léger | voir « Habillage par graine » | centiles des BPM du GMD ; rapport médian des fichiers `swing` | GMD |
| Micro-timing par voix, coups probabilistes | voir « Habillage par graine » et `percussion.toml` | médianes en ms par rôle ; probabilité par pas du groupe | GMD |
| Tonalités, instruments, kits, voicings, registre, énergie, arcs nommés, mixage, vélocités | — | réglages de service (règles de métier) | — |

### Batterie : groupes retenus

Fichiers `beat` : 28, mesures 613, batteurs 4. Seuils d'éligibilité de ce profil :
au moins 2 fichiers, 10 mesures, aucun fichier au-delà de 60 % du groupe (quatre batteurs, groupes de charleston en croches et doubles croches ; seuils de L2 relâchés à deux fichiers et dix mesures).

| Groupe (timekeeper:figure ou pas exacts) | Étiquette | Mesures | Fichiers | Batteurs |
|---|---|---|---|---|
| `hat_closed:eighths` | hat | 134 | 11 | 4 |
| `hat_closed:sixteenths` | hat | 62 | 4 | 4 |
| `hat_closed:pas:0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15` | hat | 54 | 4 | 4 |
| `hat_closed:quarters_plus` | hat | 25 | 7 | 3 |
| `ride:quarters_plus` | ride | 24 | 4 | 1 |
| `hat_closed:pas:2,4,6,8,10,12,14` | hat | 10 | 5 | 2 |


Relances retenues :

| Longueur | Pas de départ | Fichiers | Batteurs | Seuil de probabilité |
|---|---|---|---|---|
| 1 mesure(s) | 0 | 22 | 1 | 0.25 |

## Mixage

Aucune mesure de corpus ne donne un mixage : les niveaux de piste sont calés hors ligne, en silence
(`calibrer_mix.py` : rendus WAV de 48 mesures, graines 1 et 2, LUFS intégré en solo par piste) sur les mêmes cibles
pour tous les profils `gmd_*` : accords −26 LUFS, basse −25, batterie (grosse caisse + reste du kit) −25,5, soit un mix
d'environ −21,5 LUFS (`wjazzd_swing` : −22,2).

| Piste en solo | LUFS intégré |
|---|---:|
| accords | -26.2 |
| basse | -25.0 |
| grosse caisse | -34.3 |
| reste du kit | -26.0 |
| batterie (somme en puissance) | -25.4 |
| mix complet | -21.4 |


## Habillage par graine (`session.toml`, A2 ; R2-styles)

Une graine tire **un habillage** : un tirage indépendant par groupe, tous les groupes se combinent (9 groupes,
31 variantes, **43200 habillages possibles**). Le tirage est stable par graine, rejoué après un seek, et **l'hôte gagne** : `SetBpm` impose le tempo, `SetSwing`
le swing, `[fixed_tonality]` du profil de l'hôte (la grille de training, transposée par lui) la tonalité (la variante est
alors ignorée) ; programmes, voicings, registre, énergie et arc n'ont pas de commande (un hôte qui n'en veut pas installe
un profil sans `[session]`). Ce profil n'a donc **plus de `[fixed_tonality]`** : la tonalité par défaut (mi bémol ionien) n'est
que la plus pondérée.

| Groupe | Variantes (poids) | Ce que règle le groupe | D'où vient la valeur |
|---|---|---|---|
| `tempo` | lent (1), medium (3) | tempo (plage de BPM, un entier tiré) | GMD : centiles des BPM des fichiers `beat` |
| `tonalite` | mi_bemol_ionian (3), si_bemol_ionian (2), la_bemol_ionian (2), fa_ionian (1), do_ionian (2) | tonique et mode du premier air (grille relative transposée) | métier : tonalités courantes du style |
| `comping` | piano_electrique (4), orgue (2), piano (2), clavinet (1), guitare_claire (1) | programme des accords | métier |
| `basse` | doigts (4), fretless (2), mediator (1) | programme de la basse | métier |
| `kit` | standard (4), room (2), balais (1) | kit de la banque 128 | métier ; le GMD est joué sur un kit électronique Roland TD-11 |
| `voicing` | drop_2 (3), shell (1), notes_guides (1), ouvert (1) | famille de voicings du comping | métier |
| `registre` | normal (4), haut (1) | octave du comping | métier |
| `energie` | doux (1), normal (2), fort (1) | multiplicateurs d'intensité et de densité de base | service |
| `arc` | defaut (3), orchestre (2), direct (1), montee (1) | arc : `[arc]` du profil ou arc nommé de `session.toml` | service (aucun corpus ne donne l'arc d'un morceau) |

Les arcs nommés (`orchestre` : le comping et le kit changent de chorus en chorus et reviennent aux programmes de la session ;
`direct` : pas d'exposition ; `montee` : un cran par chorus) sont écrits dans `session.toml`, mêmes valeurs de service que
`arc.toml`. Tous les instruments sont des programmes General MIDI de GeneralUser GS (banque 0) et des kits de la banque 128
(0, 8, 16, 24, 25), vérifiés dans la banque ; aucune zone Odin (D1).

**Micro-timing par voix** (`performance.tracks.Percussion*.voices`, A6) : médianes de l'écart à la grille mesurées dans le
GMD (groupe 4/4 droit de `soul`, `soul/groove3`, `soul/groove4`, `soul/groove9`, `soul/groove10`, `soul/motown`, pondérées par l'effectif), en **fraction de pas au tempo par
défaut (102 BPM)** : un pas = 15 000 / BPM ms ; les décalages mesurés en ms ne suivent donc pas exactement le tempo tiré
(à 130 BPM le même retard en ms vaut 24 % de pas de plus). `jitter` : amplitude d'un tirage uniforme dont l'écart-type est celui
mesuré, plafonnée à 0,1 pas (borne du schéma ; l'écart-type mesuré est de 0,11 à 0,20 pas).

| Voix | Note GM | Sur le temps (pas) | Hors temps (pas) | Jitter (pas) |
|---|---:|---:|---:|---:|
| kick | 36 | -0.146 | -0.095 | 0.10 |
| sidestick | 37 | -0.175 | -0.175 | 0.10 |
| snare | 38 | -0.097 | -0.076 | 0.10 |
| hat_closed | 42 | -0.068 | -0.071 | 0.10 |
| tom_low | 43 | -0.151 | -0.151 | 0.10 |
| hat_pedal | 44 | -0.200 | +0.196 | 0.10 |
| hat_open | 46 | -0.114 | -0.174 | 0.10 |
| crash | 49 | -0.181 | -0.181 | 0.10 |
| tom_high | 50 | -0.148 | -0.148 | 0.10 |
| ride | 51 | -0.134 | -0.157 | 0.10 |
| ride_bell | 53 | -0.083 | -0.111 | 0.10 |

**Coups probabilistes** (`probability`, A6) : 89 coups de `percussion.toml` jouent avec la probabilité par pas mesurée dans le
groupe (grosse caisse, caisse claire, cross-stick entre 0,08 et le seuil de la densité ; notes fantômes à la densité 2),
au lieu d'un coup certain dès 0,25 (0,3 pour funk et New Orleans) ou absent : le tirage est sans état (graine, mesure, pas, note) donc stable par graine.

**Voicings** : `chords.color` (tension 9, 11 ou 13 que la qualité permet, jamais en conflit avec la grille, D4) et
`chords.register_drift` (dérive bornée du centre) sont déclarés ; ils n'agissent que sur les familles de voicings qui les
lisent (pas `close_triad`).


## Variété entre graines et entre chorus (D6)

Entre graines : une des quatre boucles de la partie A et trois de la partie B, le plan de motifs de basse et le gabarit, les rythmes et phrases de comping, les groupes de batterie et leurs phrases, les relances. Entre chorus : l'arc.

Mesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) :

| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |
|---|---:|---:|---:|---:|---:|
| accompagnement | 99% | 0% | 4.0 | 16 / 16 | 62% |
| accords | 0% | 100% | 1.0 | 8 / 16 | 60% |
| basse | 2% | 46% | 2.0 | 16 / 16 | 90% |
| batterie | 73% | 0% | 4.0 | 16 / 16 | 28% |
| comping : rythme | 84% | 0% | 4.0 | 16 / 16 | 33% |
| comping : voicings | 98% | 0% | 4.0 | 16 / 16 | 98% |




## Distance de la batterie au GMD

`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), 16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la batterie de `wjazzd_swing` et sans arc.

| Voix | Profil | Profil sans arc | Copie non calibrée |
|---|---:|---:|---:|
| any | 0.086 | 0.117 | 0.326 |
| hat | 0.304 | 0.148 | 0.810 |
| kick | 0.314 | 0.430 | 1.000 |
| moyenne | 0.252 | 0.247 | 0.784 |
| ride | 0.438 | 0.903 | 0.742 |
| snare | 0.302 | 0.294 | 1.000 |

## Limites

- **Peu de batteurs** : quatre, et quatre séries « groove » de 64 mesures chacune d'un même protocole.
- **Pas de cuivres ni de cordes** : un seul instrument d'accords par profil.
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
0 % → 58 %, paires de chorus identiques 100 % → 11 %, suites d'accords distinctes sur 16 graines 8 / 16 → 16 / 16 ;
basse changée (elle suit la fondamentale des accords) 7 % → 59 %. Poids et progressions de métier, aucun corpus de grilles.

## Régénérer

`percussion.toml` est généré : `{venv du labo}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py gmd_soul`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).

<!-- q0:début -->
## Corrections Q0 (phase 020, `TUNING_VERSION` 34)

Compte rendu, mesures avant / après et limites : `specs/020-qualite-des-styles/q0-corrections-rapides.md`. Scripts : `specs/020-qualite-des-styles/q0/` (`appliquer.py` : accents et mise en place, `batterie.py` : groove de batterie).

- **Accents du comping** (règle de métier : temps 1 et backbeat appuyés, 3e double croche légèrement appuyée, scratchs fantômes). Chaque coup garde la vélocité de 019 et reçoit un écart selon sa place, forme `soul_comp`, amplitude 26, décalage de niveau -8 (le niveau moyen est conservé : la calibration du mixage reste valable) ; appuyés : temps 1 (+0,8), temps 3 (+0,3), contretemps 2 et 4 (+0,5), attaque anticipée (+0,7) ; légers : 1re double croche (-0,7) ; coups probabilistes : -0,9 de plus. Plage de vélocité des accords et de la basse : 32 à 118 (`velocity_min`, `velocity_max` d'A6).
- **Accents de la basse** (temps 1 appuyé, notes mortes aux 1res doubles croches) : forme `soul_bass`, amplitude 14, décalage -3 ; appuyés : temps 1 (+1), temps 3 (+0,3), 3e double croche (+0,3), attaque anticipée (+0,5) ; légers : croche de contretemps (-0,4), 1re double croche (-0,9) ; coups probabilistes : -0,9 de plus.
- **Groove de batterie** : phrases et patterns de 019 inchangés ; probabilité des coups secondaires 0,95, 0 note(s) fantôme(s) à 0,45, exposant des probabilités 3, phrases tenues ×16, part de fins de phrase sans relance ×8.
- **Mise en place** : basse sur les décalages de la grosse caisse (A6, par pas), accords sur la moyenne caisse claire / timekeeper ; `jitter` basse 0,02 et accords 0,02 pas ; `jitter` de la batterie 0,045 pas (écart-type intra-fichier du GMD) ; égrenage ancré sur la première corde (`anchor = 0`).

<!-- q0:fin -->

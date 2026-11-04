# `gmd_dance_disco` — accompagnement disco sous une grille

Profil d'accompagnement **sous une grille d'accords** : basse en octaves, guitare claire en contretemps et batterie disco du GMD. Sans grille fournie : deux parties de huit mesures de boucles mineures à septièmes. Aucune mélodie.

**Nom (principe VII).** Le profil nomme la source qui définit son style : le style `dance/disco` du Groove MIDI Dataset v1.0.0 (5 fichiers `beat`, 431 mesures, 4/4 droit, « juste », L2 § 4.2 ; aucun fichier `fill`). Ce n'est pas « le disco » :
il décrit la batterie de CINQ fichiers joués par deux batteurs sur un kit électronique ; la basse en octaves et la guitare sont des règles de métier.

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
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**la éolien** par défaut ; la variante
de session en tire une autre, voir « Habillage par graine »), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
2021-01-28) range les « chord symbols based on standard chord progressions » parmi le matériau non protégeable
(*common property*) ; aucune œuvre n'est reproduite, aucune n'est nommée.

| Partie | Poids | Progression (mesure par mesure) |
|---|---|---|
| A (8 mesures) | 3 | i7 i7 iv7 iv7 (deux fois) |
| A (8 mesures) | 3 | i7 iv7 bVII7 bIII (deux fois) |
| A (8 mesures) | 2 | i7 bVII7 bVI V7 (deux fois) |
| A (8 mesures) | 1 | i7 (quatre mesures) iv7 iv7 V7 V7 |
| B (8 mesures) | 3 | bVI bVII7 i7 i7 bVI bVII7 i7 V7 |
| B (8 mesures) | 2 | bIII bVI bVII7 i7 (deux fois) |
| B (8 mesures) | 2 | iv7 iv7 i7 i7 iv7 iv7 V7 V7 |



## Écriture

4/4, 16 pas par mesure, une pulsation de 4 pas, feel droit (rapport de swing 0,5 : le GMD mesure 0,49 (médiane de 5 fichiers)). Tempo
120 noires/min par défaut, plage 120 à 137 : tempo des 5 fichiers `beat` du GMD (BPM du nom de fichier) : centile 10 = 120, médiane = 120, centile 90 = 137. Un chorus = 16 mesures ; 4 chorus par air
(un air par set) : le nombre de chorus est connu, la retombée de l'arc joue au dernier.

| Couche | Réglage |
|---|---|
| Basse | basse au doigt (programme 33), cinq motifs d'une ou deux mesures en octaves (croches, croches avec quinte, syncopés, fondamentale longue, deux mesures) ; fenêtre 28..39 |
| Accords | guitare claire (programme 27), triades, contretemps et « chicken-scratch » en doubles croches (attaques vers le haut sur trois cordes) : huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense`, douze phrases de 2 à 4 mesures dont quatre `part_end` |
| Batterie | kit GeneralUser GS banque 128, programme 0 (kit standard) ; 17 patterns, 17 phrases distinctes, 0 relances ; densité 0 timekeeper, 1 ossature, 2 notes fantômes |
| Arc | exposition, corps, sommet, retombée : `arc.toml` (valeurs de service) |
| Timing | basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service) |
| Mixage | master 20.0 dB, accords -14.4 dB, basse -15.4 dB, batterie -14.2 dB : réglages de service calés hors ligne (section Mixage) |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS (D1).

## Mesures → paramètres

| Paramètre | Valeur | Mesure | Source |
|---|---|---|---|
| `meter.default_bpm` / `min_bpm` / `max_bpm` | 120 / 120 / 137 | tempo des 5 fichiers `beat` (4/4 droit) : médiane et centiles 10 et 90 ; BPM du nom de fichier | GMD |
| `meter.default_swing_ratio` | 0,5 | rapport médian des fichiers retenus 0,49 (médiane de 5 fichiers) | GMD |
| Patterns de batterie | 17 | groupes ci-dessous, seuils d'éligibilité du profil, seuils de probabilité de L2 | GMD |
| Phrases de batterie | 18 entrées (3 densités) | mesurées : suites de groupes vues ≥ 3 fois (≥ 2 si la suite change de groupe) dans ≥ 2 fichiers ; 12 phrases de groove tenu 4 mesures ajoutées pour les groupes sans suite mesurée (règle de métier) | GMD |
| Relances | 0 groupes × 1 étiquettes × 2 densités | fichiers `fill` (0) | GMD |
| Basse, comping, grilles | — | **aucune mesure** : règles de métier (octaves de basse, guitare en contretemps) | règles |
| Plages de tempo de la session, swing léger | voir « Habillage par graine » | centiles des BPM du GMD ; rapport médian des fichiers `swing` | GMD |
| Micro-timing par voix, coups probabilistes | voir « Habillage par graine » et `percussion.toml` | médianes en ms par rôle ; probabilité par pas du groupe | GMD |
| Tonalités, instruments, kits, voicings, registre, énergie, arcs nommés, mixage, vélocités | — | réglages de service (règles de métier) | — |

### Batterie : groupes retenus

Fichiers `beat` : 5, mesures 431, batteurs 2. Seuils d'éligibilité de ce profil :
au moins 2 fichiers, 6 mesures, aucun fichier au-delà de 90 % du groupe (cinq fichiers et deux batteurs : seuils réduits à deux fichiers et six mesures (aucun groupe ne tient les seuils de L2)).

| Groupe (timekeeper:figure ou pas exacts) | Étiquette | Mesures | Fichiers | Batteurs |
|---|---|---|---|---|
| `hat_closed:pas:0,1,2,3,5,6,7,8,9,10,11,13,14,15` | hat | 114 | 3 | 2 |
| `hat_closed:quarters_plus` | hat | 28 | 3 | 1 |
| `hat_open:pas:2,3,6,7,10,11,14,15` | open | 26 | 2 | 1 |
| `hat_closed:pas:0` | hat | 9 | 3 | 2 |
| `hat_closed:pas:0,2,4,6,8,12,14` | hat | 7 | 2 | 1 |
| `hat_closed:sixteenths` | hat | 7 | 2 | 1 |


Relances retenues :

| Longueur | Pas de départ | Fichiers | Batteurs | Seuil de probabilité |
|---|---|---|---|---|
| — | | | | |

## Mixage

Aucune mesure de corpus ne donne un mixage : les niveaux de piste sont calés hors ligne, en silence
(`calibrer_mix.py` : rendus WAV de 48 mesures, graines 1 et 2, LUFS intégré en solo par piste) sur les mêmes cibles
pour tous les profils `gmd_*` : accords −26 LUFS, basse −25, batterie (grosse caisse + reste du kit) −25,5, soit un mix
d'environ −21,5 LUFS (`wjazzd_swing` : −22,2).

| Piste en solo | LUFS intégré |
|---|---:|
| accords | -26.2 |
| basse | -25.2 |
| grosse caisse | -28.1 |
| reste du kit | -28.6 |
| batterie (somme en puissance) | -25.4 |
| mix complet | -21.0 |


## Habillage par graine (`session.toml`, A2 ; R2-styles)

Une graine tire **un habillage** : un tirage indépendant par groupe, tous les groupes se combinent (9 groupes,
31 variantes, **46080 habillages possibles**). Le tirage est stable par graine, rejoué après un seek, et **l'hôte gagne** : `SetBpm` impose le tempo, `SetSwing`
le swing, `[fixed_tonality]` du profil de l'hôte (la grille de training, transposée par lui) la tonalité (la variante est
alors ignorée) ; programmes, voicings, registre, énergie et arc n'ont pas de commande (un hôte qui n'en veut pas installe
un profil sans `[session]`). Ce profil n'a donc **plus de `[fixed_tonality]`** : la tonalité par défaut (la éolien) n'est
que la plus pondérée.

| Groupe | Variantes (poids) | Ce que règle le groupe | D'où vient la valeur |
|---|---|---|---|
| `tempo` | direct (3), rapide (2) | tempo (plage de BPM, un entier tiré) | GMD : centiles des BPM des fichiers `beat` |
| `tonalite` | la_aeolian (3), re_aeolian (2), mi_aeolian (2), sol_aeolian (1), si_aeolian (1) | tonique et mode du premier air (grille relative transposée) | métier : tonalités courantes du style |
| `comping` | guitare_claire (3), piano_electrique (2), clavinet (1), piano (1) | programme des accords | métier |
| `basse` | doigts (3), slap (2), mediator (1), synthetique (1) | programme de la basse | métier |
| `kit` | standard (4), electronique (2), room (1), 808 (1) | kit de la banque 128 | métier ; le GMD est joué sur un kit électronique Roland TD-11 |
| `voicing` | triades_serrees (3), drop_2 (2), ouvert (1) | famille de voicings du comping | métier |
| `registre` | normal (4), haut (1) | octave du comping | métier |
| `energie` | doux (1), normal (2), fort (1) | multiplicateurs d'intensité et de densité de base | service |
| `arc` | defaut (3), orchestre (2), direct (1), montee (1) | arc : `[arc]` du profil ou arc nommé de `session.toml` | service (aucun corpus ne donne l'arc d'un morceau) |

Les arcs nommés (`orchestre` : le comping et le kit changent de chorus en chorus et reviennent aux programmes de la session ;
`direct` : pas d'exposition ; `montee` : un cran par chorus) sont écrits dans `session.toml`, mêmes valeurs de service que
`arc.toml`. Tous les instruments sont des programmes General MIDI de GeneralUser GS (banque 0) et des kits de la banque 128
(0, 8, 16, 24, 25), vérifiés dans la banque ; aucune zone Odin (D1).

**Micro-timing par voix** (`performance.tracks.Percussion*.voices`, A6) : médianes de l'écart à la grille mesurées dans le
GMD (groupe 4/4 droit de `dance/disco`, pondérées par l'effectif), en **fraction de pas au tempo par
défaut (120 BPM)** : un pas = 15 000 / BPM ms ; les décalages mesurés en ms ne suivent donc pas exactement le tempo tiré
(à 130 BPM le même retard en ms vaut 24 % de pas de plus). `jitter` : amplitude d'un tirage uniforme dont l'écart-type est celui
mesuré, plafonnée à 0,1 pas (borne du schéma ; l'écart-type mesuré est de 0,11 à 0,20 pas).

| Voix | Note GM | Sur le temps (pas) | Hors temps (pas) | Jitter (pas) |
|---|---:|---:|---:|---:|
| kick | 36 | -0.083 | -0.029 | 0.10 |
| sidestick | 37 | -0.167 | -0.167 | 0.10 |
| snare | 38 | -0.080 | -0.075 | 0.10 |
| hat_closed | 42 | -0.044 | -0.051 | 0.10 |
| hat_pedal | 44 | -0.200 | +0.200 | 0.10 |
| hat_open | 46 | -0.109 | -0.109 | 0.10 |
| tom_high | 50 | -0.167 | -0.167 | 0.10 |
| ride | 51 | -0.142 | -0.175 | 0.10 |

**Coups probabilistes** (`probability`, A6) : 51 coups de `percussion.toml` jouent avec la probabilité par pas mesurée dans le
groupe (grosse caisse, caisse claire, cross-stick entre 0,08 et le seuil de la densité ; notes fantômes à la densité 2),
au lieu d'un coup certain dès 0,25 (0,3 pour funk et New Orleans) ou absent : le tirage est sans état (graine, mesure, pas, note) donc stable par graine.

**Voicings** : `chords.color` (tension 9, 11 ou 13 que la qualité permet, jamais en conflit avec la grille, D4) et
`chords.register_drift` (dérive bornée du centre) sont déclarés ; ils n'agissent que sur les familles de voicings qui les
lisent (pas `close_triad`).


## Variété entre graines et entre chorus (D6)

Entre graines : une des quatre boucles de la partie A et trois de la partie B, le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie et leurs phrases. Entre chorus : l'arc. Pas de relance de batterie (voir limites).

Mesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) :

| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |
|---|---:|---:|---:|---:|---:|
| accompagnement | 95% | 0% | 4.0 | 16 / 16 | 58% |
| accords | 0% | 100% | 1.0 | 8 / 16 | 52% |
| basse | 2% | 46% | 2.0 | 16 / 16 | 75% |
| batterie | 71% | 15% | 3.12 | 16 / 16 | 29% |
| comping : rythme | 79% | 0% | 4.0 | 16 / 16 | 35% |
| comping : voicings | 91% | 0% | 4.0 | 16 / 16 | 98% |




## Distance de la batterie au GMD

`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), 16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la batterie de `wjazzd_swing` et sans arc.

| Voix | Profil | Profil sans arc | Copie non calibrée |
|---|---:|---:|---:|
| any | 0.087 | 0.036 | 0.435 |
| hat | 0.189 | 0.118 | 0.862 |
| kick | 0.215 | 0.062 | 1.000 |
| moyenne | 0.163 | 0.111 | 0.824 |
| ride | 1.000 | 1.000 | 0.870 |
| snare | 0.159 | 0.227 | 1.000 |

## Limites

- **Fiabilité** : « juste » (5 fichiers) mais deux batteurs seulement ; un groupe domine (114 mesures sur 3 fichiers).
- **Aucun fichier `fill`** dans le style : pas de relance de batterie ; la variation de fin de phrase vient des phrases, de l'arc (breaks, stop-time) et de la basse.
- **Pas de cordes ni de cuivres** : un seul instrument d'accords par profil.
- **Basse et comping : règles de métier**, jugées à l'écoute seulement (D5 : aucune écoute exigée ; des preuves chiffrées,
  compte rendu `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`). WJazzD ne mesure que la basse du jazz ; aucun corpus distribuable de
  voicings ni de rythmes de comping (décision F3).
- **Batterie** : kit électronique Roland TD-11, dix batteurs en tout ; les vélocités et le micro-timing sont ceux du
  kit (niveaux recalés au kit de rendu). Un groupe de peu de fichiers décrit un batteur, pas un style (L2 § 4.1).
- **Manques du schéma** (comptes rendus B8 et R2-styles) : accords de puissance (« 5 »), options de basse d'A4
  (`walking.*` : notes fantômes, croches, notes répétées, arrivée) exclusives de `motifs` et de `templates` donc
  inutilisables avec des riffs, feel de basse et étiquettes d'arc pour les motifs de basse (`bass_feel` exige la marche),
  groupes de session indépendants (un kit « power » avec un tempo lent n'est pas exclu), boucle de grille fixe par air.

## Régénérer

`percussion.toml` est généré : `{venv du labo}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py gmd_dance_disco`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).

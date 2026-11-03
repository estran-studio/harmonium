# `gmd_pop` — accompagnement pop sous une grille

Profil d'accompagnement **sous une grille d'accords** : basse au doigt (motifs, gabarit à relance), piano en triades (rythmes de comping, phrases) et batterie pop du GMD. Sans grille fournie : deux parties de huit mesures de forme commune. Aucune mélodie.

**Nom (principe VII).** Le profil nomme la source qui définit son style : la famille `pop` du Groove MIDI Dataset v1.0.0 (`pop` : 8 fichiers, 174 mesures, « juste » ; `pop/soft` et `pop/groove7` : « juste » chacun ; L2 § 4.2). Les trois sous-styles du GMD sont réunis parce que L2 leur donne chacun « juste » et que le GMD les range sous la même famille. Ce n'est pas « le pop » :
il décrit la batterie de 14 fichiers joués par quatre batteurs ; le piano et la basse sont des règles de métier. Les fichiers `pop` sont à 132-142 BPM, `pop/soft` à 83.

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
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**sol ionien**,
`fixed_tonality`), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
2021-01-28) range les « chord symbols based on standard chord progressions » parmi le matériau non protégeable
(*common property*) ; aucune œuvre n'est reproduite, aucune n'est nommée.

| Partie | Poids | Progression (mesure par mesure) |
|---|---|---|
| A (8 mesures) | 3 | I V vi IV (deux fois) |
| A (8 mesures) | 2 | vi IV I V (deux fois) |
| A (8 mesures) | 2 | I vi IV V (deux fois) |
| A (8 mesures) | 1 | I IV vi V (deux fois) |
| B (8 mesures) | 3 | IV I V vi (deux fois) |
| B (8 mesures) | 2 | IV V iii vi IV V I I |
| B (8 mesures) | 2 | vi V IV I (deux fois) |
| B (8 mesures) | 1 | I V IV IV I V vi V |



## Écriture

4/4, 16 pas par mesure, une pulsation de 4 pas, feel droit (rapport de swing 0,5 : le GMD mesure 0,49 (médiane de 14 fichiers)). Tempo
135 noires/min par défaut, plage 83 à 142 : tempo des 14 fichiers `beat` du GMD (BPM du nom de fichier) : centile 10 = 83, médiane = 135, centile 90 = 142. Un chorus = 16 mesures ; 4 chorus par air
(un air par set) : le nombre de chorus est connu, la retombée de l'arc joue au dernier.

| Couche | Réglage |
|---|---|
| Basse | basse au doigt (programme 33), six motifs d'une ou deux mesures tirés par poids (noires, syncopé, croches, blanches, octaves, deux mesures) et un gabarit en noires avec relance vers la fondamentale ; fenêtre 28..39 |
| Accords | piano à queue (programme 0), triades conduites par les voix communes (centre 62) ; huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense`, douze phrases de 2 à 4 mesures, quatre de mise en place `part_end` |
| Batterie | kit GeneralUser GS banque 128, programme 0 (kit standard) ; 20 patterns, 31 phrases distinctes, 2 relances ; densité 0 timekeeper, 1 ossature, 2 notes fantômes |
| Arc | exposition, corps, sommet, retombée : `arc.toml` (valeurs de service) |
| Timing | basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service) |
| Mixage | master 20.0 dB, accords -13.6 dB, basse -14.4 dB, batterie -12.6 dB : réglages de service calés hors ligne (section Mixage) |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS (D1).

## Mesures → paramètres

| Paramètre | Valeur | Mesure | Source |
|---|---|---|---|
| `meter.default_bpm` / `min_bpm` / `max_bpm` | 135 / 83 / 142 | tempo des 14 fichiers `beat` (4/4 droit) : médiane et centiles 10 et 90 ; BPM du nom de fichier | GMD |
| `meter.default_swing_ratio` | 0,5 | rapport médian des fichiers retenus 0,49 (médiane de 14 fichiers) | GMD |
| Patterns de batterie | 20 | groupes ci-dessous, seuils d'éligibilité du profil, seuils de probabilité de L2 | GMD |
| Phrases de batterie | 36 entrées (3 densités) | mesurées : suites de groupes vues ≥ 3 fois (≥ 2 si la suite change de groupe) dans ≥ 2 fichiers ; 15 phrases de groove tenu 4 mesures ajoutées pour les groupes sans suite mesurée (règle de métier) | GMD |
| Relances | 2 groupes × 2 étiquettes × 2 densités | fichiers `fill` (10) | GMD |
| Basse, comping, grilles | — | **aucune mesure** : règles de métier (basse de pop (noires, syncope, octaves), piano en noires, croches et contretemps) | règles |
| Tonalité, instruments, arc, mixage, vélocités | — | réglages de service | — |

### Batterie : groupes retenus

Fichiers `beat` : 14, mesures 294, batteurs 4. Seuils d'éligibilité de ce profil :
au moins 2 fichiers, 6 mesures, aucun fichier au-delà de 90 % du groupe (14 fichiers seulement, quatre batteurs : les seuils de L2 (3 fichiers, 20 mesures, 60 %) ne laissent qu'un groupe ; relâchés, ils en gardent plusieurs, qui décrivent surtout un ou deux batteurs).

| Groupe (timekeeper:figure ou pas exacts) | Étiquette | Mesures | Fichiers | Batteurs |
|---|---|---|---|---|
| `hat_closed:eighths` | hat | 71 | 9 | 4 |
| `hat_closed:quarters_plus` | hat | 62 | 10 | 4 |
| `ride:quarters_plus` | ride | 9 | 3 | 1 |
| `ride:eighths` | ride | 7 | 2 | 1 |
| `ride_bell:quarters` | ride | 7 | 2 | 1 |
| `hat_closed:pas:0,2,4,6,8,10,12` | hat | 6 | 5 | 3 |
| `hat_closed:pas:0,2,4,8,10,12,14` | hat | 6 | 5 | 1 |


Relances retenues :

| Longueur | Pas de départ | Fichiers | Batteurs | Seuil de probabilité |
|---|---|---|---|---|
| 1 mesure(s) | 0 | 5 | 1 | 0.4 |
| 2 mesure(s) | 12 | 3 | 1 | 0.5 |

## Mixage

Aucune mesure de corpus ne donne un mixage : les niveaux de piste sont calés hors ligne, en silence
(`calibrer_mix.py` : rendus WAV de 48 mesures, graines 1 et 2, LUFS intégré en solo par piste) sur les mêmes cibles
pour tous les profils `gmd_*` : accords −26 LUFS, basse −25, batterie (grosse caisse + reste du kit) −25,5, soit un mix
d'environ −21,5 LUFS (`wjazzd_swing` : −22,2).

| Piste en solo | LUFS intégré |
|---|---:|
| accords | -26.1 |
| basse | -25.2 |
| grosse caisse | -31.6 |
| reste du kit | -24.8 |
| batterie (somme en puissance) | -23.9 |
| mix complet | -20.9 |


## Variété entre graines et entre chorus (D6)

Entre graines : une des quatre boucles de chaque partie, le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie et leurs phrases, les relances. Entre chorus : l'arc (comping clairsemé puis dense, charleston puis ride, breaks, stop-time).

Mesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) :

| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |
|---|---:|---:|---:|---:|---:|
| accompagnement | 95% | 0% | 4.0 | 16 / 16 | 52% |
| accords | 0% | 100% | 1.0 | 12 / 16 | 52% |
| basse | 3% | 42% | 2.0 | 16 / 16 | 88% |
| batterie | 84% | 0% | 4.0 | 16 / 16 | 33% |
| comping : rythme | 83% | 0% | 4.0 | 16 / 16 | 29% |
| comping : voicings | 82% | 0% | 4.0 | 16 / 16 | 56% |


L'habillage par graine (A2 : tempo dans la plage, instruments, kit, tonalité, famille de voicings) **n'existe pas
encore** : il est ajouté aux profils B en vague 4. Ce profil ne le simule pas.


## Distance de la batterie au GMD

`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), 16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la batterie de `wjazzd_swing` et sans arc.

| Voix | Profil | Profil sans arc | Copie non calibrée |
|---|---:|---:|---:|
| any | 0.084 | 0.059 | 0.236 |
| hat | 0.253 | 0.152 | 0.762 |
| kick | 0.301 | 0.361 | 1.000 |
| moyenne | 0.212 | 0.208 | 0.750 |
| ride | 0.446 | 0.152 | 0.704 |
| snare | 0.211 | 0.260 | 1.000 |

## Limites

- **Peu de batteurs** : quatre ; les groupes de batterie retenus décrivent un ou deux batteurs chacun (voir « groupes retenus »).
- Les fichiers `pop/soft` (83 BPM) et `pop` (132-142 BPM) forment deux régimes de tempo : la plage de tempo est la plus large du lot (centiles 10 et 90).
- **Basse et comping : règles de métier**, jugées à l'écoute seulement (D5 : aucune écoute exigée ; des preuves chiffrées,
  compte rendu `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`). WJazzD ne mesure que la basse du jazz ; aucun corpus distribuable de
  voicings ni de rythmes de comping (décision F3).
- **Batterie** : kit électronique Roland TD-11, dix batteurs en tout ; les vélocités et le micro-timing sont ceux du
  kit (niveaux recalés au kit de rendu). Un groupe de peu de fichiers décrit un batteur, pas un style (L2 § 4.1).
- **Manques du schéma** (voir le compte rendu B8) : accords de puissance (« 5 »), notes fantômes et croches de passage de la
  basse (A4), swing selon le tempo (A6), feel de basse selon le chorus pour les motifs (`bass_feel` exige la marche),
  micro-timing par voix de batterie, étiquettes sur les motifs de basse.

## Régénérer

`percussion.toml` est généré : `{venv du labo}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py gmd_pop`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).

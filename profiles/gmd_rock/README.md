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
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**mi mixolydien**,
`fixed_tonality`), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
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

4/4, 16 pas par mesure, une pulsation de 4 pas, feel droit (rapport de swing 0,5 : le GMD mesure 0,50 (médiane des 161 fichiers droits)). Tempo
105 noires/min par défaut, plage 90 à 130 : tempo des 154 fichiers `beat` du GMD (BPM du nom de fichier) : centile 10 = 90, médiane = 105, centile 90 = 130. Un chorus = 16 mesures ; 4 chorus par air
(un air par set) : le nombre de chorus est connu, la retombée de l'arc joue au dernier.

| Couche | Réglage |
|---|---|
| Basse | basse au médiator (programme 34), six motifs d'une ou deux mesures tirés par poids (croches à la fondamentale, noires, octave en contretemps, syncope, deux mesures avec la quinte au temps 4, note tenue) et un gabarit de croches avec relance vers la fondamentale de l'accord suivant ; fenêtre des fondamentales 28..39 (mi grave) |
| Accords | guitare saturée (programme 29), triade doublée à 4 sons conduite par les voix communes, grattée vers le bas (écart 7 ms) ; huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense`, douze phrases de 2 à 4 mesures dont quatre de mise en place `part_end` |
| Batterie | kit GeneralUser GS banque 128, programme 8 (kit « Room ») ; 22 patterns, 38 phrases distinctes, 6 relances ; densité 0 timekeeper, 1 ossature, 2 notes fantômes |
| Arc | exposition, corps, sommet, retombée : `arc.toml` (valeurs de service) |
| Timing | basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service : aucun corpus de placement rock) |
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
| Tonalité, instruments, arc, mixage, vélocités | — | réglages de service | — |

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


## Variété entre graines et entre chorus (D6)

Entre graines : une des quatre boucles de chaque partie (A et B), un des motifs de basse d'un plan de motifs par air, les rythmes et phrases de comping, les groupes de batterie et leurs phrases, les relances. Entre chorus d'une graine : l'arc (exposition, corps, sommet, retombée) change le comping, la batterie (charleston puis ride) et l'intensité ; les breaks et stop-time sont tirés par chorus.

Mesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) :

| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |
|---|---:|---:|---:|---:|---:|
| accompagnement | 94% | 0% | 4.0 | 16 / 16 | 57% |
| accords | 0% | 100% | 1.0 | 10 / 16 | 60% |
| basse | 3% | 42% | 2.0 | 16 / 16 | 85% |
| batterie | 83% | 0% | 4.0 | 16 / 16 | 47% |
| comping : rythme | 83% | 0% | 3.94 | 16 / 16 | 29% |
| comping : voicings | 84% | 0% | 3.94 | 16 / 16 | 65% |


L'habillage par graine (A2 : tempo dans la plage, instruments, kit, tonalité, famille de voicings) **n'existe pas
encore** : il est ajouté aux profils B en vague 4. Ce profil ne le simule pas.


## Distance de la batterie au GMD

`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), 16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la batterie de `wjazzd_swing` et sans arc.

| Voix | Profil | Profil sans arc | Copie non calibrée |
|---|---:|---:|---:|
| any | 0.085 | 0.088 | 0.339 |
| hat | 0.373 | 0.406 | 0.714 |
| kick | 0.218 | 0.234 | 1.000 |
| moyenne | 0.230 | 0.245 | 0.763 |
| ride | 0.157 | 0.147 | 0.579 |
| snare | 0.243 | 0.251 | 1.000 |

## Limites

- **Accords de puissance** : le schéma n'a pas de qualité « 5 » (fondamentale et quinte) ; la guitare joue des triades majeures et mineures doublées à l'octave.
- Le rock du GMD est joué majoritairement au ride et à la charleston fermée en croches ; les groupes de toms seuls (sans timekeeper) sont écartés.
- **Basse et comping : règles de métier**, jugées à l'écoute seulement (D5 : aucune écoute exigée ; des preuves chiffrées,
  compte rendu `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`). WJazzD ne mesure que la basse du jazz ; aucun corpus distribuable de
  voicings ni de rythmes de comping (décision F3).
- **Batterie** : kit électronique Roland TD-11, dix batteurs en tout ; les vélocités et le micro-timing sont ceux du
  kit (niveaux recalés au kit de rendu). Un groupe de peu de fichiers décrit un batteur, pas un style (L2 § 4.1).
- **Manques du schéma** (voir le compte rendu B8) : accords de puissance (« 5 »), notes fantômes et croches de passage de la
  basse (A4), swing selon le tempo (A6), feel de basse selon le chorus pour les motifs (`bass_feel` exige la marche),
  micro-timing par voix de batterie, étiquettes sur les motifs de basse.

## Régénérer

`percussion.toml` est généré : `{venv du labo}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py gmd_rock`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).

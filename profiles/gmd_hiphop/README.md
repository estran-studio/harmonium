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
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**do éolien**,
`fixed_tonality`), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
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
| Batterie | kit GeneralUser GS banque 128, programme 25 (kit 808/909) ; 19 patterns, 28 phrases distinctes, 4 relances ; densité 0 timekeeper, 1 ossature, 2 notes fantômes |
| Arc | exposition, corps, sommet, retombée : `arc.toml` (valeurs de service) |
| Timing | basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service) |
| Mixage | master 20.0 dB, accords -13.4 dB, basse -11.5 dB, batterie -14.6 dB : réglages de service calés hors ligne (section Mixage) |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS (D1).

## Mesures → paramètres

| Paramètre | Valeur | Mesure | Source |
|---|---|---|---|
| `meter.default_bpm` / `min_bpm` / `max_bpm` | 91 / 70 / 100 | tempo des 34 fichiers `beat` (4/4 droit) : médiane et centiles 10 et 90 ; BPM du nom de fichier | GMD |
| `meter.default_swing_ratio` | 0,5 | rapport médian des fichiers retenus 0,50 (médiane de 34 fichiers droits ; 10 fichiers à swing léger écartés) | GMD |
| Patterns de batterie | 19 | groupes ci-dessous, seuils d'éligibilité du profil, seuils de probabilité de L2 | GMD |
| Phrases de batterie | 28 entrées (3 densités) | mesurées : suites de groupes vues ≥ 3 fois (≥ 2 si la suite change de groupe) dans ≥ 2 fichiers ; 10 phrases de groove tenu 4 mesures ajoutées pour les groupes sans suite mesurée (règle de métier) | GMD |
| Relances | 4 groupes × 1 étiquettes × 2 densités | fichiers `fill` (48) | GMD |
| Basse, comping, grilles | — | **aucune mesure** : règles de métier (sub tenu et syncopé, accords de touches clairsemés) | règles |
| Tonalité, instruments, arc, mixage, vélocités | — | réglages de service | — |

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


## Variété entre graines et entre chorus (D6)

Entre graines : une des quatre boucles de chaque partie, le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie (charleston en croches, doubles croches, noires plus) et leurs phrases, les relances. Entre chorus : l'arc.

Mesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) :

| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |
|---|---:|---:|---:|---:|---:|
| accompagnement | 95% | 0% | 4.0 | 16 / 16 | 60% |
| accords | 0% | 100% | 1.0 | 10 / 16 | 56% |
| basse | 3% | 45% | 1.94 | 16 / 16 | 90% |
| batterie | 82% | 0% | 4.0 | 16 / 16 | 48% |
| comping : rythme | 83% | 0% | 4.0 | 16 / 16 | 42% |
| comping : voicings | 81% | 0% | 4.0 | 16 / 16 | 63% |


L'habillage par graine (A2 : tempo dans la plage, instruments, kit, tonalité, famille de voicings) **n'existe pas
encore** : il est ajouté aux profils B en vague 4. Ce profil ne le simule pas.


## Distance de la batterie au GMD

`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), 16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la batterie de `wjazzd_swing` et sans arc.

| Voix | Profil | Profil sans arc | Copie non calibrée |
|---|---:|---:|---:|
| any | 0.080 | 0.091 | 0.368 |
| hat | 0.125 | 0.146 | 0.827 |
| kick | 0.181 | 0.197 | 1.000 |
| moyenne | 0.149 | 0.166 | 0.799 |
| ride | 1.000 | 1.000 | 0.877 |
| snare | 0.212 | 0.231 | 1.000 |

## Limites

- **Un seul timekeeper** : le hip-hop du GMD est joué à la charleston fermée ; les étiquettes d'arc n'ont donc que `hat` (pas de ride) et le corps de l'arc ne change pas de timekeeper.
- **Swing** : dix fichiers à swing léger écartés (feel droit seulement).
- **Basse et comping : règles de métier**, jugées à l'écoute seulement (D5 : aucune écoute exigée ; des preuves chiffrées,
  compte rendu `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`). WJazzD ne mesure que la basse du jazz ; aucun corpus distribuable de
  voicings ni de rythmes de comping (décision F3).
- **Batterie** : kit électronique Roland TD-11, dix batteurs en tout ; les vélocités et le micro-timing sont ceux du
  kit (niveaux recalés au kit de rendu). Un groupe de peu de fichiers décrit un batteur, pas un style (L2 § 4.1).
- **Manques du schéma** (voir le compte rendu B8) : accords de puissance (« 5 »), notes fantômes et croches de passage de la
  basse (A4), swing selon le tempo (A6), feel de basse selon le chorus pour les motifs (`bass_feel` exige la marche),
  micro-timing par voix de batterie, étiquettes sur les motifs de basse.

## Régénérer

`percussion.toml` est généré : `{venv du labo}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py gmd_hiphop`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).

# `gmd_afrobeat` — accompagnement afrobeat sous une grille

Profil d'accompagnement **sous une grille d'accords** : basse en ostinato syncopé, guitare claire en « chicken-scratch » et batterie afrobeat du GMD. Sans grille fournie : deux parties de huit mesures de vamps à un ou deux accords. Aucune mélodie.

**Nom (principe VII).** Le profil nomme la source qui définit son style : le style `afrobeat` du Groove MIDI Dataset v1.0.0 (13 fichiers `beat`, 982 mesures, 4/4 droit, « juste », L2 § 4.2 ; aucun fichier `fill`). Ce n'est pas « le afrobeat » :
il décrit la batterie de 13 fichiers joués par deux batteurs sur un kit électronique, sans les percussions ni les cuivres du genre ; basse et guitare sont des règles de métier.

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
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**ré dorien**,
`fixed_tonality`), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
2021-01-28) range les « chord symbols based on standard chord progressions » parmi le matériau non protégeable
(*common property*) ; aucune œuvre n'est reproduite, aucune n'est nommée.

| Partie | Poids | Progression (mesure par mesure) |
|---|---|---|
| A (8 mesures) | 3 | i7 (quatre mesures) IV9 (quatre mesures) |
| A (8 mesures) | 3 | i7 i7 IV9 IV9 (deux fois) |
| A (8 mesures) | 2 | i9 (huit mesures) |
| A (8 mesures) | 1 | i7 bVII9 (alternés) IV9 v7 |
| B (8 mesures) | 3 | IV9 IV9 i7 i7 (deux fois) |
| B (8 mesures) | 2 | bVII9 bVII9 IV9 IV9 i7 (quatre mesures) |
| B (8 mesures) | 2 | i7 IV9 (alternés) |



## Écriture

4/4, 16 pas par mesure, une pulsation de 4 pas, feel droit (rapport de swing 0,5 : le GMD mesure 0,50 (médiane de 13 fichiers)). Tempo
98 noires/min par défaut, plage 86 à 113 : tempo des 13 fichiers `beat` du GMD (BPM du nom de fichier) : centile 10 = 86, médiane = 98, centile 90 = 113. Un chorus = 16 mesures ; 4 chorus par air
(un air par set) : le nombre de chorus est connu, la retombée de l'arc joue au dernier.

| Couche | Réglage |
|---|---|
| Basse | basse au doigt (programme 33), cinq motifs d'une ou deux mesures (ostinatos syncopés, version posée, croches avec quinte et septième, note tenue) ; fenêtre 28..39 |
| Accords | guitare claire (programme 27), voicings `guide_tones` (tierce et septième, plus une tension), huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense` (chicka syncopé, doubles croches), douze phrases de 2 à 4 mesures dont quatre `part_end` |
| Batterie | kit GeneralUser GS banque 128, programme 0 (kit standard) ; 21 patterns, 30 phrases distinctes, 0 relances ; densité 0 timekeeper, 1 ossature, 2 notes fantômes |
| Arc | exposition, corps, sommet, retombée : `arc.toml` (valeurs de service) |
| Timing | basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service) |
| Mixage | master 20.0 dB, accords -12.4 dB, basse -14.4 dB, batterie -12.5 dB : réglages de service calés hors ligne (section Mixage) |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS (D1).

## Mesures → paramètres

| Paramètre | Valeur | Mesure | Source |
|---|---|---|---|
| `meter.default_bpm` / `min_bpm` / `max_bpm` | 98 / 86 / 113 | tempo des 13 fichiers `beat` (4/4 droit) : médiane et centiles 10 et 90 ; BPM du nom de fichier | GMD |
| `meter.default_swing_ratio` | 0,5 | rapport médian des fichiers retenus 0,50 (médiane de 13 fichiers) | GMD |
| Patterns de batterie | 21 | groupes ci-dessous, seuils d'éligibilité du profil, seuils de probabilité de L2 | GMD |
| Phrases de batterie | 30 entrées (3 densités) | mesurées : suites de groupes vues ≥ 3 fois (≥ 2 si la suite change de groupe) dans ≥ 2 fichiers ; 12 phrases de groove tenu 4 mesures ajoutées pour les groupes sans suite mesurée (règle de métier) | GMD |
| Relances | 0 groupes × 2 étiquettes × 2 densités | fichiers `fill` (0) | GMD |
| Basse, comping, grilles | — | **aucune mesure** : règles de métier (ostinato de basse, guitare en doubles croches) | règles |
| Tonalité, instruments, arc, mixage, vélocités | — | réglages de service | — |

### Batterie : groupes retenus

Fichiers `beat` : 13, mesures 982, batteurs 2. Seuils d'éligibilité de ce profil :
au moins 2 fichiers, 10 mesures, aucun fichier au-delà de 90 % du groupe (deux batteurs : seuils réduits à deux fichiers et dix mesures).

| Groupe (timekeeper:figure ou pas exacts) | Étiquette | Mesures | Fichiers | Batteurs |
|---|---|---|---|---|
| `ride:quarters_plus` | ride | 92 | 7 | 2 |
| `ride:eighths` | ride | 68 | 5 | 1 |
| `hat_closed:quarters_plus` | hat | 65 | 4 | 2 |
| `hat_closed:eighths` | hat | 32 | 2 | 1 |
| `hat_closed:pas:0,2,6,8,10` | hat | 25 | 3 | 1 |
| `hat_closed:pas:0,6,7,10,13` | hat | 25 | 2 | 1 |
| `hat_closed:pas:0,4,6,7,10,13` | hat | 22 | 2 | 1 |


Groupes écartés (extraits) :

| Groupe | Mesures | Fichiers | Raison |
|---|---|---|---|
| `hat_closed:pas:0,2,6,8,10,14` | 17 | 4 | au-delà des 7 groupes les plus fournis |
| `hat_closed:pas:2,6,8,10,12,14` | 12 | 3 | au-delà des 7 groupes les plus fournis |
| `ride:pas:0,2,4,6,8,10,12` | 10 | 4 | au-delà des 7 groupes les plus fournis |

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
| accords | -26.1 |
| basse | -25.2 |
| grosse caisse | -28.2 |
| reste du kit | -31.8 |
| batterie (somme en puissance) | -26.7 |
| mix complet | -21.9 |


## Variété entre graines et entre chorus (D6)

Entre graines : une des quatre boucles de la partie A et trois de la partie B (de huit mesures d'un accord aux alternances), le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie (charleston et ride) et leurs phrases. Entre chorus : l'arc. Pas de relance de batterie.

Mesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) :

| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |
|---|---:|---:|---:|---:|---:|
| accompagnement | 95% | 0% | 4.0 | 16 / 16 | 50% |
| accords | 0% | 100% | 1.0 | 9 / 16 | 49% |
| basse | 3% | 42% | 2.0 | 16 / 16 | 83% |
| batterie | 79% | 0% | 4.0 | 16 / 16 | 28% |
| comping : rythme | 87% | 0% | 4.0 | 16 / 16 | 35% |
| comping : voicings | 87% | 0% | 4.0 | 16 / 16 | 58% |


L'habillage par graine (A2 : tempo dans la plage, instruments, kit, tonalité, famille de voicings) **n'existe pas
encore** : il est ajouté aux profils B en vague 4. Ce profil ne le simule pas.


## Distance de la batterie au GMD

`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), 16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la batterie de `wjazzd_swing` et sans arc.

| Voix | Profil | Profil sans arc | Copie non calibrée |
|---|---:|---:|---:|
| any | 0.060 | 0.031 | 0.397 |
| hat | 0.116 | 0.150 | 0.800 |
| kick | 0.339 | 0.224 | 1.000 |
| moyenne | 0.211 | 0.158 | 0.799 |
| ride | 0.445 | 0.143 | 0.602 |
| snare | 0.329 | 0.225 | 1.000 |

## Limites

- **Fiabilité** : « juste » (13 fichiers) mais deux batteurs seulement.
- **Aucun fichier `fill`** dans le style : pas de relance de batterie.
- **Pas de percussions, de cuivres ni de seconde guitare** : un seul instrument d'accords par profil, et le GMD n'a que des pads de kit (L2 § 3).
- **Basse et comping : règles de métier**, jugées à l'écoute seulement (D5 : aucune écoute exigée ; des preuves chiffrées,
  compte rendu `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`). WJazzD ne mesure que la basse du jazz ; aucun corpus distribuable de
  voicings ni de rythmes de comping (décision F3).
- **Batterie** : kit électronique Roland TD-11, dix batteurs en tout ; les vélocités et le micro-timing sont ceux du
  kit (niveaux recalés au kit de rendu). Un groupe de peu de fichiers décrit un batteur, pas un style (L2 § 4.1).
- **Manques du schéma** (voir le compte rendu B8) : accords de puissance (« 5 »), notes fantômes et croches de passage de la
  basse (A4), swing selon le tempo (A6), feel de basse selon le chorus pour les motifs (`bass_feel` exige la marche),
  micro-timing par voix de batterie, étiquettes sur les motifs de basse.

## Régénérer

`percussion.toml` est généré : `{venv du labo}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py gmd_afrobeat`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).

# `gmd_reggae` — accompagnement reggae sous une grille

Profil d'accompagnement **sous une grille d'accords** : basse au doigt, guitare claire en skank (attaques vers le haut, trois cordes aiguës) et batterie reggae du GMD. Sans grille fournie : deux parties de huit mesures à deux ou quatre accords. Aucune mélodie.

**Nom (principe VII).** Le profil nomme la source qui définit son style : le style `reggae` du Groove MIDI Dataset v1.0.0 (3 fichiers `beat`, 253 mesures, « juste » au sens de L2 ; le groupe 4/4 droit retenu n'en compte que 2, 132 mesures, deux batteurs, 16 fichiers `fill`). `reggae/slow` (1 fichier, 29 mesures, « trop peu ») et l'unique fichier ternaire ne sont pas lus. Ce n'est pas « le reggae » :
il décrit la batterie de DEUX fichiers joués par deux batteurs : les grooves y ont la grosse caisse sur les quatre temps (la moitié des mesures) plutôt que le « one drop » classique ; la basse et la guitare, elles, suivent le métier (temps 1 laissé vide, skank sur les contretemps).

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
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**la ionien**,
`fixed_tonality`), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
2021-01-28) range les « chord symbols based on standard chord progressions » parmi le matériau non protégeable
(*common property*) ; aucune œuvre n'est reproduite, aucune n'est nommée.

| Partie | Poids | Progression (mesure par mesure) |
|---|---|---|
| A (8 mesures) | 3 | I I IV IV (deux fois) |
| A (8 mesures) | 3 | I IV I V (deux fois) |
| A (8 mesures) | 2 | I vi IV V (deux fois) |
| A (8 mesures) | 1 | vi vi IV IV vi vi iii iii |
| B (8 mesures) | 3 | IV IV I I IV IV V V |
| B (8 mesures) | 2 | I V vi IV (deux fois) |
| B (8 mesures) | 2 | IV I IV I IV V I I |



## Écriture

4/4, 16 pas par mesure, une pulsation de 4 pas, feel droit (rapport de swing 0,5 : le GMD mesure 0,49 (médiane de 2 fichiers droits)). Tempo
102 noires/min par défaut, plage 83 à 121 : tempo des 2 fichiers `beat` du GMD (BPM du nom de fichier) : centile 10 = 83, médiane = 102, centile 90 = 121. Un chorus = 16 mesures ; 4 chorus par air
(un air par set) : le nombre de chorus est connu, la retombée de l'arc joue au dernier.

| Couche | Réglage |
|---|---|
| Basse | basse au doigt (programme 33), cinq motifs d'une ou deux mesures (« one drop » qui laisse le temps 1 vide, fondamentale longue, noires, contretemps, deux mesures) ; fenêtre 28..39 |
| Accords | guitare claire (programme 27), triades jouées vers le haut sur les trois cordes aiguës (`up_note_count = 3`, écart 4 ms) : huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense` (skank sur les contretemps, doublé en doubles croches), douze phrases de 2 à 4 mesures dont quatre `part_end` |
| Batterie | kit GeneralUser GS banque 128, programme 0 (kit standard) ; 8 patterns, 8 phrases distinctes, 1 relances ; densité 0 timekeeper, 1 ossature, 2 notes fantômes |
| Arc | exposition, corps, sommet, retombée : `arc.toml` (valeurs de service) |
| Timing | basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service ; le reggae joue en retard sur le temps, rien n'est mesuré ici) |
| Mixage | master 20.0 dB, accords -14.1 dB, basse -13.9 dB, batterie -14.1 dB : réglages de service calés hors ligne (section Mixage) |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS (D1).

## Mesures → paramètres

| Paramètre | Valeur | Mesure | Source |
|---|---|---|---|
| `meter.default_bpm` / `min_bpm` / `max_bpm` | 102 / 83 / 121 | tempo des 2 fichiers `beat` (4/4 droit) : médiane et centiles 10 et 90 ; BPM du nom de fichier | GMD |
| `meter.default_swing_ratio` | 0,5 | rapport médian des fichiers retenus 0,49 (médiane de 2 fichiers droits) | GMD |
| Patterns de batterie | 8 | groupes ci-dessous, seuils d'éligibilité du profil, seuils de probabilité de L2 | GMD |
| Phrases de batterie | 9 entrées (3 densités) | mesurées : suites de groupes vues ≥ 3 fois (≥ 2 si la suite change de groupe) dans ≥ 2 fichiers ; 6 phrases de groove tenu 4 mesures ajoutées pour les groupes sans suite mesurée (règle de métier) | GMD |
| Relances | 1 groupes × 1 étiquettes × 2 densités | fichiers `fill` (16) | GMD |
| Basse, comping, grilles | — | **aucune mesure** : règles de métier (one drop, skank sur les contretemps) | règles |
| Tonalité, instruments, arc, mixage, vélocités | — | réglages de service | — |

### Batterie : groupes retenus

Fichiers `beat` : 2, mesures 132, batteurs 2. Seuils d'éligibilité de ce profil :
au moins 2 fichiers, 6 mesures, aucun fichier au-delà de 90 % du groupe (deux fichiers 4/4 droits et deux batteurs seulement : seuils réduits à deux fichiers et six mesures (les seuils de L2 ne retiennent aucun groupe)).

| Groupe (timekeeper:figure ou pas exacts) | Étiquette | Mesures | Fichiers | Batteurs |
|---|---|---|---|---|
| `hat_closed:quarters_plus` | hat | 19 | 2 | 2 |
| `hat_closed:eighths` | hat | 16 | 2 | 2 |
| `hat_closed:pas:0,2,4,6,8,10,12` | hat | 6 | 2 | 2 |


Relances retenues :

| Longueur | Pas de départ | Fichiers | Batteurs | Seuil de probabilité |
|---|---|---|---|---|
| 1 mesure(s) | 0 | 15 | 1 | 0.25 |

## Mixage

Aucune mesure de corpus ne donne un mixage : les niveaux de piste sont calés hors ligne, en silence
(`calibrer_mix.py` : rendus WAV de 48 mesures, graines 1 et 2, LUFS intégré en solo par piste) sur les mêmes cibles
pour tous les profils `gmd_*` : accords −26 LUFS, basse −25, batterie (grosse caisse + reste du kit) −25,5, soit un mix
d'environ −21,5 LUFS (`wjazzd_swing` : −22,2).

| Piste en solo | LUFS intégré |
|---|---:|
| accords | -26.1 |
| basse | -25.1 |
| grosse caisse | -28.3 |
| reste du kit | -30.3 |
| batterie (somme en puissance) | -26.2 |
| mix complet | -22.0 |


## Variété entre graines et entre chorus (D6)

Entre graines : une des quatre boucles de la partie A et trois de la partie B, le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie (groupes de charleston fermée) et leurs phrases, les relances. Entre chorus : l'arc (comping clairsemé puis dense, breaks, stop-time) ; la batterie n'a qu'un timekeeper (`hat`).

Mesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) :

| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |
|---|---:|---:|---:|---:|---:|
| accompagnement | 93% | 0% | 4.0 | 16 / 16 | 49% |
| accords | 0% | 100% | 1.0 | 9 / 16 | 57% |
| basse | 3% | 42% | 2.0 | 16 / 16 | 87% |
| batterie | 63% | 1% | 3.94 | 16 / 16 | 18% |
| comping : rythme | 87% | 0% | 4.0 | 16 / 16 | 24% |
| comping : voicings | 86% | 0% | 4.0 | 16 / 16 | 58% |


L'habillage par graine (A2 : tempo dans la plage, instruments, kit, tonalité, famille de voicings) **n'existe pas
encore** : il est ajouté aux profils B en vague 4. Ce profil ne le simule pas.


## Distance de la batterie au GMD

`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), 16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la batterie de `wjazzd_swing` et sans arc.

| Voix | Profil | Profil sans arc | Copie non calibrée |
|---|---:|---:|---:|
| any | 0.169 | 0.167 | 0.319 |
| hat | 0.078 | 0.083 | 0.801 |
| kick | 0.278 | 0.265 | 1.000 |
| moyenne | 0.251 | 0.239 | 0.780 |
| ride | 1.000 | 1.000 | 1.000 |
| snare | 0.478 | 0.442 | 1.000 |

## Limites

- **Fiabilité** : « juste » pour le style (3 fichiers), mais **le groupe retenu ne compte que deux fichiers et deux batteurs** : la variété de batterie d'une graine à l'autre est faible par construction (trois groupes, donc trois patterns par densité au plus). Le profil ne dit rien du reggae joué par d'autres batteurs.
- **Pas d'orgue « bubble »** : un seul instrument d'accords par profil (la guitare tient le skank).
- **Pas de retard sur le temps** (le micro-timing reggae n'est pas mesuré ici ; décalage par piste de basse et d'accords à 0).
- **Basse et comping : règles de métier**, jugées à l'écoute seulement (D5 : aucune écoute exigée ; des preuves chiffrées,
  compte rendu `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`). WJazzD ne mesure que la basse du jazz ; aucun corpus distribuable de
  voicings ni de rythmes de comping (décision F3).
- **Batterie** : kit électronique Roland TD-11, dix batteurs en tout ; les vélocités et le micro-timing sont ceux du
  kit (niveaux recalés au kit de rendu). Un groupe de peu de fichiers décrit un batteur, pas un style (L2 § 4.1).
- **Manques du schéma** (voir le compte rendu B8) : accords de puissance (« 5 »), notes fantômes et croches de passage de la
  basse (A4), swing selon le tempo (A6), feel de basse selon le chorus pour les motifs (`bass_feel` exige la marche),
  micro-timing par voix de batterie, étiquettes sur les motifs de basse.

## Régénérer

`percussion.toml` est généré : `{venv du labo}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py gmd_reggae`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).

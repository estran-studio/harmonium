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
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**la éolien**,
`fixed_tonality`), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
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
| Tonalité, instruments, arc, mixage, vélocités | — | réglages de service | — |

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


## Variété entre graines et entre chorus (D6)

Entre graines : une des quatre boucles de la partie A et trois de la partie B, le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie et leurs phrases. Entre chorus : l'arc. Pas de relance de batterie (voir limites).

Mesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) :

| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |
|---|---:|---:|---:|---:|---:|
| accompagnement | 88% | 0% | 4.0 | 16 / 16 | 52% |
| accords | 0% | 100% | 1.0 | 9 / 16 | 56% |
| basse | 3% | 42% | 2.0 | 16 / 16 | 74% |
| batterie | 67% | 17% | 3.0 | 16 / 16 | 20% |
| comping : rythme | 83% | 0% | 4.0 | 16 / 16 | 36% |
| comping : voicings | 83% | 0% | 4.0 | 16 / 16 | 73% |


L'habillage par graine (A2 : tempo dans la plage, instruments, kit, tonalité, famille de voicings) **n'existe pas
encore** : il est ajouté aux profils B en vague 4. Ce profil ne le simule pas.


## Distance de la batterie au GMD

`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), 16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la batterie de `wjazzd_swing` et sans arc.

| Voix | Profil | Profil sans arc | Copie non calibrée |
|---|---:|---:|---:|
| any | 0.089 | 0.045 | 0.435 |
| hat | 0.206 | 0.117 | 0.862 |
| kick | 0.210 | 0.074 | 1.000 |
| moyenne | 0.164 | 0.113 | 0.824 |
| ride | 1.000 | 1.000 | 0.870 |
| snare | 0.151 | 0.215 | 1.000 |

## Limites

- **Fiabilité** : « juste » (5 fichiers) mais deux batteurs seulement ; un groupe domine (114 mesures sur 3 fichiers).
- **Aucun fichier `fill`** dans le style : pas de relance de batterie ; la variation de fin de phrase vient des phrases, de l'arc (breaks, stop-time) et de la basse.
- **Pas de cordes ni de cuivres** : un seul instrument d'accords par profil.
- **Basse et comping : règles de métier**, jugées à l'écoute seulement (D5 : aucune écoute exigée ; des preuves chiffrées,
  compte rendu `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`). WJazzD ne mesure que la basse du jazz ; aucun corpus distribuable de
  voicings ni de rythmes de comping (décision F3).
- **Batterie** : kit électronique Roland TD-11, dix batteurs en tout ; les vélocités et le micro-timing sont ceux du
  kit (niveaux recalés au kit de rendu). Un groupe de peu de fichiers décrit un batteur, pas un style (L2 § 4.1).
- **Manques du schéma** (voir le compte rendu B8) : accords de puissance (« 5 »), notes fantômes et croches de passage de la
  basse (A4), swing selon le tempo (A6), feel de basse selon le chorus pour les motifs (`bass_feel` exige la marche),
  micro-timing par voix de batterie, étiquettes sur les motifs de basse.

## Régénérer

`percussion.toml` est généré : `{venv du labo}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py gmd_dance_disco`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).

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
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**mi bémol ionien**,
`fixed_tonality`), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
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
| Batterie | kit GeneralUser GS banque 128, programme 0 (kit standard) ; 15 patterns, 21 phrases distinctes, 1 relances ; densité 0 timekeeper, 1 ossature, 2 notes fantômes |
| Arc | exposition, corps, sommet, retombée : `arc.toml` (valeurs de service) |
| Timing | basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service) |
| Mixage | master 20.0 dB, accords -14.4 dB, basse -14.6 dB, batterie -15.3 dB : réglages de service calés hors ligne (section Mixage) |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS (D1).

## Mesures → paramètres

| Paramètre | Valeur | Mesure | Source |
|---|---|---|---|
| `meter.default_bpm` / `min_bpm` / `max_bpm` | 102 / 80 / 105 | tempo des 28 fichiers `beat` (4/4 droit) : médiane et centiles 10 et 90 ; BPM du nom de fichier | GMD |
| `meter.default_swing_ratio` | 0,5 | rapport médian des fichiers retenus 0,48 (médiane de 28 fichiers) | GMD |
| Patterns de batterie | 15 | groupes ci-dessous, seuils d'éligibilité du profil, seuils de probabilité de L2 | GMD |
| Phrases de batterie | 22 entrées (3 densités) | mesurées : suites de groupes vues ≥ 3 fois (≥ 2 si la suite change de groupe) dans ≥ 2 fichiers ; 4 phrases de groove tenu 4 mesures ajoutées pour les groupes sans suite mesurée (règle de métier) | GMD |
| Relances | 1 groupes × 2 étiquettes × 2 densités | fichiers `fill` (33) | GMD |
| Basse, comping, grilles | — | **aucune mesure** : règles de métier (figures de basse soul (octave, quinte), « chank » du clavier électrique) | règles |
| Tonalité, instruments, arc, mixage, vélocités | — | réglages de service | — |

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


## Variété entre graines et entre chorus (D6)

Entre graines : une des quatre boucles de la partie A et trois de la partie B, le plan de motifs de basse et le gabarit, les rythmes et phrases de comping, les groupes de batterie et leurs phrases, les relances. Entre chorus : l'arc.

Mesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) :

| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |
|---|---:|---:|---:|---:|---:|
| accompagnement | 90% | 0% | 4.0 | 16 / 16 | 55% |
| accords | 0% | 100% | 1.0 | 11 / 16 | 62% |
| basse | 3% | 42% | 2.0 | 16 / 16 | 88% |
| batterie | 61% | 1% | 3.94 | 16 / 16 | 20% |
| comping : rythme | 88% | 0% | 3.94 | 16 / 16 | 33% |
| comping : voicings | 84% | 0% | 3.94 | 16 / 16 | 71% |


L'habillage par graine (A2 : tempo dans la plage, instruments, kit, tonalité, famille de voicings) **n'existe pas
encore** : il est ajouté aux profils B en vague 4. Ce profil ne le simule pas.


## Distance de la batterie au GMD

`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), 16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la batterie de `wjazzd_swing` et sans arc.

| Voix | Profil | Profil sans arc | Copie non calibrée |
|---|---:|---:|---:|
| any | 0.087 | 0.074 | 0.326 |
| hat | 0.397 | 0.117 | 0.810 |
| kick | 0.346 | 0.372 | 1.000 |
| moyenne | 0.285 | 0.214 | 0.784 |
| ride | 0.466 | 0.851 | 0.742 |
| snare | 0.309 | 0.292 | 1.000 |

## Limites

- **Peu de batteurs** : quatre, et quatre séries « groove » de 64 mesures chacune d'un même protocole.
- **Pas de cuivres ni de cordes** : un seul instrument d'accords par profil.
- **Basse et comping : règles de métier**, jugées à l'écoute seulement (D5 : aucune écoute exigée ; des preuves chiffrées,
  compte rendu `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`). WJazzD ne mesure que la basse du jazz ; aucun corpus distribuable de
  voicings ni de rythmes de comping (décision F3).
- **Batterie** : kit électronique Roland TD-11, dix batteurs en tout ; les vélocités et le micro-timing sont ceux du
  kit (niveaux recalés au kit de rendu). Un groupe de peu de fichiers décrit un batteur, pas un style (L2 § 4.1).
- **Manques du schéma** (voir le compte rendu B8) : accords de puissance (« 5 »), notes fantômes et croches de passage de la
  basse (A4), swing selon le tempo (A6), feel de basse selon le chorus pour les motifs (`bass_feel` exige la marche),
  micro-timing par voix de batterie, étiquettes sur les motifs de basse.

## Régénérer

`percussion.toml` est généré : `{venv du labo}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py gmd_soul`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).

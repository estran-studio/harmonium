# `wjazzd_swing` — accompagnement swing sous une grille

Premier profil distribué d'accompagnement **sous une grille d'accords** : la basse (marche), les
accords (voicings sans fondamentale, rythmes de comping) et la batterie ne dépendent que de l'accord
publié. Il est fait pour le client externe `harmonium_training`, qui fournit sa grille ; sans grille,
il joue deux formes communes écrites pour lui (blues de douze mesures, forme AABA de trente-deux).
Aucune mélodie : la piste `Melody` n'est pas dans l'arrangement.

**Nom (principe VII).** Le profil nomme la source qui définit son style : le **sous-ensemble
`rhythmfeel = SWING` de la Weimar Jazz Database v2.1** (361 solos, parutions de 1937 à 2007, styles
SWING, BEBOP, COOL, HARDBOP, POSTBOP, FREE). Ce n'est ni « le jazz » ni une époque. La batterie vient
du Groove MIDI Dataset (jazz, jazz/swing), le swing et le placement des instruments de la Jazz Trio
Database ; ce sont des couches d'appoint, pas l'identité.

## Mentions de licence (décision F3 du propriétaire)

- **WJazzD** — *Contains information from the Weimar Jazz Database (Jazzomat Research Project, HfM
  Weimar), which is made available here under the Open Database License (ODbL) 1.0,
  <https://opendatacommons.org/licenses/odbl/1-0/> ; individual contents under the Database Contents
  License (DbCL) 1.0.* Le profil est un *Produced Work* (ODbL §1.0, §4.3) : il ne contient que des
  statistiques agrégées (parts, centiles, poids), aucune note de solo ni de ligne de basse, aucun
  extrait substantiel de la base (pas de *Derivative Database*).
- **Groove MIDI Dataset v1.0.0** (Google LLC) — CC BY 4.0 : J. Gillick, A. Roberts, J. Engel, D. Eck,
  D. Bamman, « Learning to Groove with Inverse Sequence Transformations », ICML 2019. Les patterns
  sont construits sur des probabilités par pas agrégées ; aucune mesure d'enregistrement.
- **Jazz Trio Database v0.2** — MIT License, Copyright (c) 2024 H. Cheston, J. L. Schlichting,
  I. Cross, P. M. C. Harrison (TISMIR 2024, DOI 10.5334/tismir.186). Seuls des rapports de swing et des
  écarts de placement agrégés ; le MIDI de piano n'est jamais lu.
- Aucune source NC ni SA n'entre dans ce profil.

Provenance chiffrée (empreintes SHA256, manifestes, commandes) :
`harmonium_lab/references/wjazzd-swing/README.md` et `corpus/{wjazzd,gmd,jtd}/`.

## Grilles par défaut : provenance et droits

Les deux grilles sont écrites relativement à la tonique (fa, `fixed_tonality`), pas copiées d'une partition.

| Grille | Écriture | Droits |
|---|---|---|
| **Blues de douze mesures** | `I7 IV7 I7 I7 \| IV7 IV7 I7 I7 \| ii7 V7 I7 \| I7 V7` (la dernière mesure à la blanche) | Forme commune, sans œuvre reproduite. Le Compendium du US Copyright Office (§802.5(A), éd. du 2021-01-28) range les « chord symbols based on standard chord progressions » parmi le matériau non protégeable (*common property*) ; le blues de douze mesures est publié depuis *Memphis Blues* (1912) et *St. Louis Blues* (1914) de W. C. Handy (1873–1958), libres aux États-Unis et, l'auteur étant mort en 1958, au Canada. |
| **AABA, *rhythm changes*** | A : `I6 VI7 \| ii7 V7 \| iii7 VI7 \| ii7 V7 \| v7 I7 \| IV6 IVm6 \| I6/V VI7 \| ii7 V7` ; B : `III7 \| III7 \| VI7 \| VI7 \| II7 \| II7 \| V7 \| V7` | La progression vient de *I Got Rhythm* (George Gershwin, publiée en 1930) : **libre aux États-Unis depuis le 2026-01-01** (Duke CSPD, *Public Domain Day 2026*, qui la cite comme « the source of the foundational jazz chord progression known as the “rhythm changes” »). **Libre au Canada** : la musique est de George Gershwin (1898–1937) ; depuis le 2022-12-30 le terme est de 70 ans après la mort de l'auteur, sans effet rétroactif (CARL), donc libre pour tout auteur mort en 1971 ou avant [inférence tirée de cette règle]. Les paroles d'Ira Gershwin (mort en 1983) ne sont pas utilisées : la grille n'en contient aucune ; la qualification de l'œuvre en collaboration au Canada n'est pas vérifiée, c'est pourquoi la grille est écrite ici comme forme commune et non recopiée de la partition de 1930. |

Sources lues le 2026-10-30 / 2026-11-01 : <https://cspd.law.duke.edu/publicdomainday/2026/>,
<https://www.copyright.gov/comp3/chap800/ch800-performing-arts.pdf>,
<https://www.carl-abrc.ca/influencing-policy/copyright/faq-term-extension-for-authored-works> ;
dates de naissance et de mort : Wikipédia (résumés REST).
Exclues pour le Canada (donc absentes) : *Body and Soul* (Johnny Green, mort en 1989) et *Georgia on My
Mind* (Hoagy Carmichael, mort en 1981).

## Écriture

4/4, 16 pas par mesure, une pulsation de 4 pas ; 150 noires/min par défaut (plage 90 à 285), swing
0,71. Un tour de 44 mesures : le blues (mesures 1 à 12) puis A A B A (13 à 44) ; deux tours par air.
Le tempo, la vélocité et la densité de la batterie suivent l'arousal (`dynamics.toml`) ; la
tonalité est verrouillée en fa et la grille d'entrée s'écrit relative à la tonique.

| Couche | Réglage |
|---|---|
| Basse | marche, quatre noires par mesure, arrivée sur la basse de l'accord à chaque changement, programme 32 (basse acoustique) |
| Accords | voicings `rootless` de quatre sons conduits par les voix communes, fenêtre 52..76, programme 0 (piano) ; six rythmes d'une mesure tirés par poids, dont trois avec une attaque anticipée sur la mesure suivante et un silence |
| Batterie | kit jazz de GeneralUser GS (banque 128, programme 32) : ride 51, pédale de charleston 44, caisse claire 38, toms 43, grosse caisse 36 ; densité 0 ride seul, 1 ride et pédale aux temps 2 et 4, 2 caisse claire fantôme et grosse caisse ; relance de fin de phrase à la densité 2 |
| Timing | basse sur la batterie (0 pas), accords en retard de 0,2 pas, jitter 0,1 pas |
| Mixage | master +23 dB, batterie à 0 dB, basse −16 dB, accords −14 dB (RMS de chaque piste mesuré hors ligne) |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS.

## Mesures → paramètres

Toutes les mesures sont celles de `harmonium_lab/references/wjazzd-swing/` (`measures-*.json`).

| Paramètre | Valeur | Mesure | Source |
|---|---|---|---|
| `meter.default_swing_ratio` | 0,71 | rapport de swing de la batterie, 125–150 BPM : 0,718 (160 pistes), 150–175 : 0,700 (149) ; le GMD donne 0,646 (médiane de 18 fichiers, kit électronique) | JTD |
| `meter.default_bpm` / `min_bpm` / `max_bpm` | 150 / 90 / 285 | tempo moyen par solo : médiane 177,5 (blues 147,4), centiles 10 et 90 : 93 et 284 | WJazzD |
| `bass.walk_low` / `walk_high` | 33 / 50 | hauteur de basse, centiles 1 et 99 : 33 et 52 (plage 28–53) | WJazzD |
| `bass.low` / `high` (arrivées) | 36 / 47 | arrivées, centiles 10–90 : 36..46 ; meilleure fenêtre de 12 demi-tons : 36..47 | WJazzD |
| `walking.max_leap` | 11 | intervalles entre notes consécutives, centile 95 : 11 (50 / 75 / 90 / 99 : 2 / 5 / 8 / 14) | WJazzD |
| `walking.approach` | none 60, chromatic 8, diatonic 8 | part de notes d'approche chromatique 21 %, diatonique 19 %, aucune 59 % ; les poids sont ajustés pour que les parts de notes **générées** retrouvent ces parts (voir plus bas) | WJazzD |
| `walking.passing` | chord_tone 35, scale_step 65 | note de passage : 48 % d'accord, 23 % de gamme, 29 % chromatique (non représentable) | WJazzD |
| `walking.toward_target_weight` / `keep_direction_weight` | 2 / 1 | 44 % des pas se rapprochent de la cible, 42 % poursuivent le sens ; poids ajustés de même | WJazzD |
| `walking.duration` | 4 (noire pleine) | non mesurable : la base ne donne que la hauteur par temps | — |
| Blues : un accord par mesure, deux à la dernière | — | 92 % des mesures des solos swing sur blues ont un accord, 8 % deux ; changement au temps 1 dans 89 % des cas | WJazzD |
| Percussion `[[patterns]]` | 11 patterns pondérés (3, 4 et 4 par densité 0, 1, 2) | probabilités par pas, groupes de mesures de même figure de ride : spang-a-lang `0,4,6,8,12,14` (poids 300 / 154), noires `0,4,8,12` (73), pédale aux pas 4 et 12 (45 et 39 %), caisse claire fantôme aux pas 2, 6, 14 | GMD |
| Percussion `[[cadence_patterns]]` | densité 2 | 22 mesures `fill` : caisse claire, toms, grosse caisse | GMD |
| `performance.Chords.offsets` | 0,2 pas | piano moins batterie : +0,226 pas (+20 ms) ; médiane de 976 pistes | JTD |
| `performance.Bass` | 0 pas, jitter 0,1 | basse moins batterie : 0,000 pas (médiane) ; écart-type 0,22 pas, borné par le moteur à 0,1 | JTD |
| Voicings et rythmes de comping | `rootless` ; six rythmes | **aucune mesure distribuable** : les voicings sont des règles de métier (décision F3). Le MIDI de piano de JTD joue le soliste, pas l'accompagnement : ses probabilités par pas (7 attaques par mesure) ne donnent pas de banque de comping | règles |
| Courbes d'émotion, mixage, vélocités | — | réglages de service, aucune mesure | — |

### Pourquoi les poids de basse ne sont pas les parts mesurées

Les parts mesurées décrivent la **note jouée** (« la note du temps avant l'arrivée est à un demi-ton de la
cible »). Les poids du moteur sont ceux d'un **tirage** : une note de passage tirée « sans approche » tombe
souvent à un demi-ton ou un ton de la cible. Avec les parts mesurées comme poids, le moteur joue 34 %
d'approches chromatiques (mesuré : 21 %) et 33 % de diatoniques (19 %). `none` 60, `chromatic` 8,
`diatonic` 8 donnent 20 % et 30 % sur 8 graines. La descente par coordonnée a porté sur six graines
(approche, passage et intervalles de `bass_compare`) ; seize autres graines (11 à 26) donnent la même
distance : `references/wjazzd-swing/validation-bass-heldout.md`.

## Distance à WJazzD

Basse générée contre les 361 solos swing (0 = identique), 8 graines de 88 mesures :

| Famille | `wjazzd_swing` | ligne de base `grille_accompagnee` |
|---|---|---|
| approche | 0,169 | 0,435 |
| intervalles | 0,144 | 0,186 |
| registre | 0,117 | 0,185 |
| passage | 0,137 | 0,082 |
| rythme harmonique (grille, non réglé) | 0,302 | 0,188 |
| arrivée (fixe) | 0,678 | 0,678 |
| **moyenne** | **0,250** | **0,326** |

## Limites

- **Arrivée.** Le moteur arrive à 100 % sur la basse publiée ; WJazzD n'y arrive que dans 41 % des cas
  (63 % dans le groupe de solos dont la basse tombe sur l'accord), 23 % pour la fondamentale au premier
  temps d'une mesure sans changement. La distance de la famille `arrival` (0,68) est identique à celle de
  la ligne de base et ne se règle pas : fondamentale hors arrivée et arrivée sur une note d'accord ne sont
  pas représentables aujourd'hui.
- **Pas de répétition de note** (20 % mesurés contre 3 % générés), **pas de côté d'approche** réglable
  (même côté que l'arrivée précédente dans 85 % des cas, 66 % mesurés) ; part de notes d'accord au passage
  trop haute (58 % contre 48 %) et poursuite du sens trop fréquente (53 % contre 42 %), pas de notes en plus de la noire
  (17 % des attaques de basse de JTD tombent hors pulsation), pas de two-feel.
- **Un seul swing** (0,71) : JTD le mesure de 0,72 (100–125 BPM) à 0,58 (275–330) ; au-delà de 200 BPM le
  réglage est trop swingué. `SetSwing` ou `--swing` le change à la volée.
- **Rythme harmonique de la grille AABA** plus dense que le corpus (deux accords dans 24 de ses 32 mesures
  contre 22 % des mesures de WJazzD).
- Le comping et la batterie sont jugés **à l'écoute** : le comping n'a aucune source distribuable,
  la batterie du GMD vient d'un kit électronique (niveaux recalés au kit de rendu, voir
  `harmonium_lab/backing/engine_values.py`, `ROLE_GAIN`).
- Les solos transcrits restent sous droits : rien ne les copie.

## Régénérer

`percussion.toml` est généré : `make backing-values OUT=… HEADER=references/wjazzd-swing/percussion-header.txt`
dans `harmonium_lab`. Les autres valeurs sont portées à la main dans `profile.toml`, avec la mesure en
commentaire.

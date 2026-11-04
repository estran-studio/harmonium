# `wjazzd_swing` — accompagnement swing sous une grille

Premier profil distribué d'accompagnement **sous une grille d'accords** : la basse (marche), les
accords (voicings, rythmes et phrases de comping) et la batterie (patterns, phrases, relances) ne
dépendent que de l'accord publié. Il est fait pour le client externe `harmonium_training`, qui fournit sa
grille ; sans grille, il joue des formes communes écrites pour lui (quatre blues de douze mesures, forme
AABA de trente-deux en deux versions). Depuis E12, une mélodie jazz (saxophone ténor) tirée de statistiques
de WJazzD joue par-dessus, coupable par `EnableMelody(false)` (voir « Mélodie »).

Depuis B1 (phase 019, `TUNING_VERSION` 28) **une graine change tout ce qui s'entend** : tempo, tonalité,
instruments, voicings, kit, arc sur quatre chorus (`session.toml`, A2 et A3). Voir « Habillage par graine ».

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

4/4, 16 pas par mesure, une pulsation de 4 pas ; tempo de repli 150 (plage 90 à 285). Un chorus de 44
mesures : un blues (mesures 1 à 12, quatre versions) puis A A B A (13 à 44, deux versions de A, deux de
B) ; quatre chorus par air (176 mesures). Sous une grille d'hôte (une partie de N mesures), le chorus est
une passe de la grille et l'arc joue sur ces chorus. Le tempo, la vélocité et la densité de la batterie
suivent l'arousal (`dynamics.toml`). Pas de `[fixed_tonality]` : la tonalité vient de la graine ; la
grille s'écrit relative à la tonique, et celle d'un hôte gagne.

| Couche | Réglage |
|---|---|
| Basse | marche (quatre noires) ou deux temps selon l'arc ; arrivée sur la basse de l'accord, la tierce, la quinte ou la septième (A4) ; notes répétées, chromatisme de passage, côté d'approche tiré ; croches de passage et notes fantômes (règle de métier) ; contrebasse 32 (basse aux doigts 33, rare) |
| Accords | six familles de voicings tirées par graine (`rootless`, `guide_tones`, `drop2`, `drop3`, `fourths`, `shell`), tension de couleur (9, 11, 13) et dérive de registre ; quinze rythmes d'une mesure (`sparse`, `medium`, `dense`, certains anticipés) joués en 24 phrases de 2 à 8 mesures, mises en place en fin de partie et de chorus ; piano 0, guitare de jazz 26 ou orgue 17 |
| Batterie | kit jazz de GeneralUser GS (128/32) ou balais (128/40) : ride 51, pédale 44, caisse claire 38, toms 43, grosse caisse 36, balayage 40 ; densité 0 timekeeper, 1 + pédale aux temps 2 et 4, 2 + caisse claire fantôme et grosse caisse ; patterns `spang` / `quarters` (GMD) et `brush` (règle de métier), phrases de 2 et 4 mesures, relances d'une mesure en fin de phrase |
| Swing | rapport par piste selon le tempo (`swing.toml`, JTD) : batterie 0,721 jusqu'à 140 BPM puis −0,098 par 100 BPM ; basse 0,715 puis −0,044 ; comping 0,702 puis −0,073 |
| Timing | basse sur la batterie (0 pas), accords en retard de 0,2 pas **sur les temps** (le rapport du piano est mesuré sur ses contretemps, déjà en retard), jitter 0,1 pas |
| Mixage | master +25 dB, batterie −9 dB, basse −16 dB, accords −14 dB : en solo, ride environ 8 LU sous la basse et les accords (la batterie à 0 dB couvrait tout à l'écoute du propriétaire). Même niveau pour le piano, la guitare et l'orgue : non mesuré par timbre |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS.

## Habillage par graine (`session.toml`, B1)

Une graine tire une variante par groupe (A2) : **sept groupes, 19 656 habillages possibles**.

| Groupe | Variantes (poids) | Source |
|---|---|---|
| `rythme` : tempo, kit, arc ensemble | lent 100-118 (5), medium 122-142 (7), medium-up 148-186 (7), up 192-225 (5), très rapide 232-265 (3) ; balais en majorité au tempo lent (3 sur 5), 1 sur 7 au medium, jamais au-delà de 142 | tempos : WJazzD (centiles 10 / 25 / 50 / 75 / 90 = 93 / 132 / 177,5 / 242,6 / 284,5 BPM) ; ballade (< 100) = B2 ; poids aplatis |
| `tonalite` | fa, si♭, mi♭, do (3), sol (2), la♭, ré (1) | métier (tonalités courantes du répertoire) |
| `comping` | piano 4, guitare 3, orgue 2 | métier |
| `basse` | contrebasse 14, basse aux doigts 1 | métier |
| `voicing` | `rootless` 4, `guide_tones` 2, `drop2` 2, `drop3`, `fourths`, `shell` 1 | A5 |
| `registre` | normal 4, comping une octave plus bas 1 | métier |
| `energie` | doux 0,85, neutre, fort 1,12 (intensité et densité de base) | métier |

Six arcs nommés (quatre chorus ; au-delà, corps et sommet en boucle) : `exposition_baguettes` et
`exposition_balais` (chorus 1 à deux temps, comping clairsemé), `directe_baguettes` et `directe_balais` (la
marche dès le chorus 1), `orchestre` (balais à l'exposition et à la retombée, baguettes au corps, guitare
ou orgue au corps ou au sommet), `rapide` (≥ 190 BPM : comping d'une ou deux attaques). Chaque arc a un sommet
(plus fort, comping dense, stop-time de deux mesures une fois sur deux) et une retombée au dernier chorus
(deux temps ou marche selon le tempo, stop-time).

## Swing rapide : le choix pour T1 (B1)

**Décision : pas de profil frère.** `wjazzd_swing` couvre les styles iReal « Medium Swing », « Medium Up
Swing » et « Up Tempo Swing » ; le tempo vient de l'hôte (`SetBpm`, il gagne sur la variante) et le rapport
de swing suit alors la courbe JTD de chaque piste (0,72 à 100-140 BPM, 0,64 à 220, 0,59 à 270 : mesuré sur
le MIDI, voir le compte rendu). Preuves :

- distance de la basse à WJazzD par tranche de tempo (8 graines × 88 mesures, `make backing-validate`) : de
  0,141 (< 130 BPM) à 0,182 (≥ 235 BPM), contre 0,217 à 0,274 avant B1 : un seul jeu de poids, calibré sur
  toutes les tranches, tient chacune à moins de 0,05 de la moyenne (0,158). Un profil frère calibré sur le
  rapide gagnerait au plus l'écart entre 0,182 et 0,141 (non mesuré : aucun profil frère n'a été calibré) ;
- les réglages qui dépendent du tempo (kit, arc, densité du comping) se tirent dans le même groupe
  `rythme` que le tempo, donc une graine rapide est cohérente.

**Limite, et usage pour T1.** L'hôte qui impose un tempo n'impose pas la variante : une graine tirée à 120 BPM
(balais, exposition à deux temps) jouée à 230 BPM reste aux balais. Training lit `MeasureSnapshot.session`
(groupe `rythme`) : **pour « Up Tempo Swing », choisir une graine dont `rythme` est `up_rapide` ou
`tres_rapide`** ; pour « Medium Swing », `medium_*` ; pour « Medium Up Swing », `medium_up_*` ; hors du
tempo de la variante, rien ne casse mais l'habillage n'est plus à l'échelle du tempo. Ne pas imposer `SetSwing` :
la commande désactive les courbes par piste. Manque de schéma (A7) : une commande qui impose une variante
(« balais, quelle que soit la graine »), et des poids de basse ou de comping qui dépendent du tempo.

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

Basse générée contre les 361 solos swing (0 = identique), 8 graines de 88 mesures, **sans arc ni croches**
(le validateur du labo ne lit que des mesures à une note par temps ; copie du profil sans `session.toml`
ni `eighths`, mêmes poids de basse) :

| Famille | B1 | avant B1 (E7 + S0) |
|---|---|---|
| approche | 0,076 | 0,167 |
| intervalles | 0,082 | 0,137 |
| registre | 0,118 | 0,105 |
| passage | 0,044 | 0,129 |
| arrivée | 0,333 | 0,678 |
| rythme harmonique (grille, non réglé : les blues de jazz ont plus de ii-V) | 0,372 | 0,302 |
| **moyenne** | **0,158** | **0,246** |

Par tranche de tempo de WJazzD (moyenne) : < 130 BPM 0,141 ; 130-175 0,153 ; 175-235 0,160 ; ≥ 235 0,182
(avant : 0,217 / 0,234 / 0,248 / 0,274). Contrôle de la calibration d'A4 sur ce profil : aucune perte.

## Mélodie (E12, `TUNING_VERSION` 32)

La piste `Melody` joue dès le premier chorus (saxophone ténor, programme 66 de GeneralUser GS) ;
`EnableMelody(false)` la coupe, pour l'improvisation de training. `cells.toml` (213 cellules, six rôles :
`open`, `run`, `arp`, `turn`, `cad_half`, `cad_full`) est **synthétisé** par le labo depuis des statistiques
agrégées des solos swing de WJazzD (rythmes en croches, pas diatoniques, cibles sur les temps, cadences,
longueur des phrases) ; aucune phrase de solo n'est copiée, et toute cellule qui contiendrait une suite de
6 notes (pas et durées) ou de 7 notes (pas) d'un solo est refusée à la génération (ODbL 1.0 §4.3, usage A ;
`harmonium_lab/references/wjazzd-melodie/README.md`).

| Paramètre | Valeur | Source |
|---|---|---|
| Phrases de quatre mesures | `Open`, trois libres dont `CadHalf` (m.4), reprise, `CadFull` (fin de A et du blues), `CadHalf` (fin de B) | règle de métier |
| Poids des rôles libres | conjoint 21, arpège 6, broderie 16 | parts des mesures ni ouvrantes ni cadentielles (26 086 mesures) |
| Registre | centre 65, tessiture 53..77 | centiles 50, 5 et 95 des hauteurs des solos |
| Niveaux E10 | au moins 4 cellules par rôle aux niveaux 1, 2 et 3 | les 3 niveaux de référence respectent leurs bornes sur 8 graines |

Sous une grille le moteur (modèle `loop`) ramène chaque note à la note d'accord la plus proche : la ligne
n'a ni note de passage ni approche chromatique, et la banque est choisie pour que l'effondrement ne la
réduise pas à des notes répétées (8 % de répétitions, 7 % au corpus, 39 % pour l'ancien bouche-trou).
Distance au corpus (intervalles, rythme, accord, registre, phrasé) 0,435 → 0,321 ; limites (phrasé sans
silence, tout-accord) dans `specs/019-bibliotheque-accompagnements/e12-melodie-jazz.md`.

## Limites

- **Arrivée « autre ».** WJazzD arrive sur la basse de l'accord dans 41 % des cas (63 % dans le groupe
  `reliable`) ; le moteur en fait 65 % (tierce, quinte et septième à 12 / 12 / 13 %, D4 : jamais une note
  hors de l'accord). Les 27 % d'arrivées « autres » ne se reproduisent pas.
- **Croches de la basse** (notes de passage, fantômes) : aucune source (WJazzD ne note qu'une hauteur par
  temps), règle de métier ; elles ne dépendent pas du tempo.
- **Rythme harmonique des grilles par défaut** plus dense que le corpus (le blues de jazz et l'AABA ont
  deux accords par mesure plus souvent que 22 % des mesures).
- **Comping** : les rythmes, phrases et voicings sont des règles de métier (aucune source distribuable) ;
  même niveau de mixage pour le piano, la guitare et l'orgue. La batterie du GMD vient d'un kit électronique
  (niveaux recalés au kit de rendu, `ROLE_GAIN`) ; la source jazz n'a que 5 batteurs, 22 relances d'un seul
  batteur ; les balais sont une règle de métier ; la part de fins de phrase sans relance n'est pas mesurée.
- **Arc** : valeurs de service, aucune mesure de la dynamique d'un chorus. Un chorus de 44 mesures (blues
  puis AABA) est celui des grilles par défaut seulement.
- **Tonalité de la graine : premier air seulement** ; un hôte qui impose sa tonalité la fait gagner.
- Les solos transcrits restent sous droits : rien ne les copie.

## Régénérer

`percussion.toml` est généré : `specs/019-bibliotheque-accompagnements/b1/batterie_jazz.py` (venv du labo,
GMD en cache). Les autres valeurs sont portées à la main (`profile.toml`, `session.toml`, `swing.toml`), avec
la mesure ou la règle de métier en commentaire. Preuves : `b1/habillages.py` (habillages distincts),
`b1/mesure_swing.py` (swing sur le MIDI contre JTD), `make seed-variety` et `make backing-validate` du labo.

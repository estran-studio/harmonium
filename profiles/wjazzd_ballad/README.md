# `wjazzd_ballad` — ballade jazz sous une grille d'accords

Profil d'accompagnement de ballade pour le client externe `harmonium_training` : la basse (deux temps, puis marche
lente), les accords (voicings tenus, tension 9, 11 ou 13 tirée par accord) et la batterie (balais dominants) ne
dépendent que de l'accord publié. Sans grille fournie, il joue une forme AABA de trente-deux mesures (trois A et deux
ponts pondérés). Aucune mélodie, aucun Odin : tout sonne avec GeneralUser GS.

**Nom (principe VII).** Le **sous-ensemble `rhythmfeel = BALLAD` de la Weimar Jazz Database v2.1** : 10 solos, 9 avec
des mesures valides, 294 mesures, 48,8 à 67,7 BPM, enregistrés de 1945 à 1994 (post-bop 6, cool 2, bebop 1, swing 1 :
pas un style homogène). Fiabilité « **trop peu** » au sens de L2 : les valeurs de basse sont des indications. Le
sous-ensemble `TWOBEAT` (années vingt) n'est **pas** utilisé : ce n'est pas un modèle moderne.

## Mentions de licence

- **WJazzD** — *Contains information from the Weimar Jazz Database (Jazzomat Research Project, HfM Weimar), which is
  made available here under the Open Database License (ODbL) 1.0, <https://opendatacommons.org/licenses/odbl/1-0/> ;
  individual contents under the Database Contents License (DbCL) 1.0.* Statistiques agrégées seulement (*Produced
  Work*) ; aucune note de solo ni de ligne de basse.
- **Groove MIDI Dataset v1.0.0** (Google LLC), CC BY 4.0 : J. Gillick, A. Roberts, J. Engel, D. Eck, D. Bamman,
  « Learning to Groove with Inverse Sequence Transformations », ICML 2019. Probabilités par pas agrégées, aucune
  mesure d'enregistrement.
- **Jazz Trio Database v0.2**, MIT License, Copyright (c) Huw Cheston et al. : H. Cheston, J. L. Schlichting,
  I. Cross, P. M. C. Harrison, TISMIR 2024, DOI 10.5334/tismir.186. Courbes de swing agrégées.
- Aucune source NC ni SA.

Grilles : formes communes écrites pour le profil, relatives à la tonique (`I vi ii V`, `ii V`, cycle des dominantes,
cycle de deux-cinq) ; le Compendium du US Copyright Office (§802.5(A)) range les suites d'accords standard parmi le
matériau non protégeable. Aucune œuvre n'est reproduite ni nommée. Aucune réharmonisation (D4).

## Ce qui est mesuré, ce qui ne l'est pas

| Couche | Source | Statut |
|---|---|---|
| Tempo par défaut 58, plage 44–84 | WJazzD `BALLAD` : médiane 57,9, quartiles 53 et 64 | mesuré ; la session tire de 48 à 78 (au-dessus du maximum de la source, 67,7, pour la ballade « medium ») |
| Basse : fenêtre 34–45 (arrivées), 31–51 (marche) | WJazzD `BALLAD`, centiles 10–90 des arrivées, 1–99 de la hauteur | mesuré |
| Basse : parts d'approche, de passage, de notes répétées, d'arrivées | WJazzD `BALLAD` | mesuré, poids calés hors ligne sur le validateur du labo (ci-dessous) |
| Batterie : patterns `ride`, phrases, relances | GMD `jazz`, 4/4, 92 à 112 BPM, ride swingué : 5 fichiers, 206 mesures, 3 batteurs ; 22 fichiers `fill` d'un batteur | mesuré (agrégats par groupe de mesures) |
| Batterie : patterns `brush` et `hat` | — | **règle de métier** : le GMD est joué aux baguettes sur kit électronique, aucun balai n'y est mesuré |
| Swing | JTD, batterie < 120 BPM : 0,720 (plateau) | **extrapolé** : la JTD n'a aucun tempo sous 102 BPM ; la courbe de `swing.toml` reste sur son plateau |
| Comping (rythmes, phrases, voicings), grilles, arc, habillage, mixage | — | **règles de métier**, aucun corpus ouvert (décision F3) ; jugées à l'écoute par le propriétaire, pas par ce profil |

Les croches de basse ne sont pas observables (WJazzD ne note qu'une hauteur par temps) : la ballade n'en joue aucune.

## Écriture

4/4, 16 pas par mesure, 4 pas par pulsation, `default_bpm` 58, swing 0,72. Un chorus = une passe AABA de 32 mesures,
trois chorus par air (96 mesures, 6 min 37 s à 58 BPM).

| Couche | Réglage |
|---|---|
| Basse | marche aux noires tenues (`duration` = la pulsation), approches `none` 90 / chromatique 5 / diatonique 3 (côté de l'arrivée précédente 60, opposé 10, au-dessus 30), passage note d'accord 60 / gamme 40 / chromatique 18 / répétition 45, arrivée sur la basse 66 / tierce 13 / quinte 10 / septième 11 (les 18 % d'arrivées « autres » de la source ne sont pas représentables sous D4, parts normalisées), `two_feel` quinte 3 contre passage 1 |
| Accords | voicings `drop2` par défaut, fenêtre 52–78, `color` (aucune 2, publiée 2, 9 : 3, 11 : 1, 13 : 3), `register_drift` ±5 ; onze rythmes d'une mesure (5 `sparse`, 3 `medium`, 3 `dense`) et quinze phrases de 2 et 4 mesures |
| Batterie | kit balais 128/40 par défaut : `tap` (note 38) et `swirl` (note 40, « Brushed Snares » dans le kit 40 : deux touches, pas la preuve de deux gestes distincts), pédale 44, grosse caisse 36 ; kit jazz 128/32 : ride 51 |
| Arc (`arc.toml`) | exposition à deux temps (comping clairsemé, étiquette `hat` : balayage et pédale), corps en marche lente (un break de la basse et du comping à la mesure 24 une fois sur deux), retombée à deux temps avec stop-time sur les deux dernières mesures |
| Timing | basse et accords sur la batterie (0 pas, Q0), jitter 0,02 pas |
| Mixage | niveaux de `wjazzd_swing`, batterie à −10 dB ; **non mesuré au LUFS** pour ce profil |

## Habillage par graine (`session.toml`)

Neuf groupes tirés une fois par graine : tempo et swing (3 plages : 48–54, 58–66, 72–78 BPM, séparées de plus de
8 BPM), tonalité (fa, si bémol, mi bémol, la bémol, do, sol, toujours majeure), comping (piano 0, guitare de jazz 26,
orgue 16), basse (contrebasse 32, fretless 35, aux doigts 33), kit (balais 4 / baguettes douces 1, avec l'étiquette
de batterie du kit), voicings (`drop2`, `open` = *spread*, `rootless`, `upper_structure`), registre, énergie, arc
(classique, `marche_tot`, `deux_temps_tenu`, `orgue_au_corps` : l'orgue prend le comping au corps,
`baguettes_au_corps` : le kit passe aux baguettes au corps). 3 × 6 × 3 × 3 × 2 × 4 × 2 × 3 × 5 = **38 880**
habillages possibles. Programmes vérifiés dans `assets/GeneralUser-GS.sf2` (banque 0 : 0, 16, 26, 32, 33, 35 ;
banque 128 : kits 32 « Jazz » et 40 « Brush »).

Sous la grille de training, `[fixed_tonality]` de l'hôte gagne sur la tonalité de la variante, `SetBpm` et `SetSwing`
gagnent aussi ; un hôte qui veut les balais seuls installe un profil sans `[session]`.

## Distance à WJazzD `BALLAD`

`python -m harmonium_lab.backing.validate`, 16 graines de 128 mesures, référence `references/wjazzd-ballad/` (groupe
`all`) ; témoin : `wjazzd_swing`, même procédure. 0 = identique.

| Famille | `wjazzd_ballad` | `wjazzd_swing` (témoin) |
|---|---|---|
| approche | 0,118 | 0,213 |
| arrivée | 0,285 | 0,536 |
| rythme harmonique | 0,124 | 0,282 |
| intervalles | 0,078 | 0,164 |
| passage | 0,034 | 0,118 |
| registre | 0,116 | 0,138 |
| **agrégée** | **0,130** | **0,244** |

Le premier jet (approche `none` 70, 7, 4) donnait 0,137 ; trois autres jeux de poids essayés donnent 0,130 à 0,134 :
le calage s'arrête là (la source est « trop peu »). La batterie rendue est comparée au GMD par `b25_distance.py`
(`specs/019-bibliotheque-accompagnements/b25-ballade-valse.md`).

## Limites

- **Source « trop peu »** : 9 solos, 7 solistes (Murray ×3, Pepper ×2 font la moitié), une seule classe de tempo.
- **Arrivée** : le moteur n'arrive jamais hors de l'accord publié (D4) ; la part « autre » de la source (18 %) et les
  mesures sans changement dont le premier temps n'est pas la basse (distance 0,43) ne se reproduisent pas.
- **Balais** : règle de métier, ni mesure ni écoute ; le ride joué au kit 40 reste une cymbale.
- **Swing hors plage mesurée** (48–78 BPM contre 102–299 pour la JTD).
- Une seule batterie par graine dans la source : 3 batteurs, dont un porte 88 % d'un groupe.

## Régénérer

`percussion.toml` est généré :
`cd ~/Project/bascanada/harmonium_lab && .venv/bin/python <moteur>/specs/019-bibliotheque-accompagnements/b25_batterie.py wjazzd_ballad > <moteur>/profiles/wjazzd_ballad/percussion.toml`.
Les autres fichiers sont écrits à la main, avec la mesure ou la règle en commentaire.

<!-- q0:début -->
## Corrections Q0 (phase 020, `TUNING_VERSION` 34)

Compte rendu, mesures avant / après et limites : `specs/020-qualite-des-styles/q0-corrections-rapides.md`. Scripts : `specs/020-qualite-des-styles/q0/` (`appliquer.py` : accents et mise en place, `batterie.py` : groove de batterie).

- **Accents du comping** (règle de métier : accords tenus : accent sur le « et » du 2 et l'anticipation, temps légers). Chaque coup garde la vélocité de 019 et reçoit un écart selon sa place, forme `ballade_comp`, amplitude 24, décalage de niveau -1 (le niveau moyen est conservé : la calibration du mixage reste valable) ; appuyés : croche de contretemps (+0,6), attaque anticipée (+1), pas 6 (+1) ; légers : temps 3 (-0,5), contretemps 2 et 4 (-0,5), 1re double croche (-0,8), 3e double croche (-0,8) ; coups probabilistes : -0,6 de plus. Plage de vélocité des accords et de la basse : 32 à 118 (`velocity_min`, `velocity_max` d'A6).
- **Accents de la basse** (deux-temps et marche : courbe de phrase ±18 (A6), arrivées +20) : vélocité de la marche 61, arrivées +20, courbe de phrase ±18.
- **Groove de batterie** : phrases et patterns de 019 inchangés ; probabilité des coups secondaires 0,7, 4 note(s) fantôme(s) à 0,2, exposant des probabilités 1, phrases tenues ×6, part de fins de phrase sans relance ×2.
- **Mise en place** : basse et accords sans décalage systématique (la batterie n'a aucun décalage par voix ; le retard de +0,2 pas du piano est retiré : aucun instrument ne dérive de la grille plus que la batterie) ; `jitter` 0,02 pas.

<!-- q0:fin -->

# `gmd_jazz_waltz` — valse jazz (3/4) sous une grille d'accords

Profil d'accompagnement de valse jazz pour le client externe `harmonium_training` : basse (deux temps ou marche aux
noires), comping « à la valse » (temps 2 et 3, anticipations), batterie jazz 3/4. Sans grille fournie, il joue une
forme AABA de trente-deux mesures de 3/4 (trois A et deux ponts pondérés). Aucune mélodie, aucun Odin.

**Nom (principe VII).** La seule source qui définit le style est le **style `jazz` en 3/4 du Groove MIDI Dataset
v1.0.0** : **deux fichiers**, un batteur, 120 BPM (86 mesures) et 200 BPM (131 mesures). C'est « trop peu » au sens de
L2 (« un style à un ou deux fichiers décrit un batteur, pas un style ») : le profil complète par des règles de métier
dites comme telles. WJazzD n'a aucune valse exploitable (E7 ne lit que le 4/4), la JTD ne contient que du 4/4.

## Mentions de licence

- **Groove MIDI Dataset v1.0.0** (Google LLC), CC BY 4.0 : J. Gillick, A. Roberts, J. Engel, D. Eck, D. Bamman,
  « Learning to Groove with Inverse Sequence Transformations », ICML 2019. Probabilités par pas agrégées ; aucune
  mesure d'enregistrement.
- **Jazz Trio Database v0.2**, MIT License, Copyright (c) Huw Cheston et al. : H. Cheston, J. L. Schlichting,
  I. Cross, P. M. C. Harrison, TISMIR 2024, DOI 10.5334/tismir.186. Courbes de swing agrégées.
- Aucune source NC ni SA. Grilles : formes communes écrites pour le profil (suites d'accords standard : Compendium
  du US Copyright Office §802.5(A)), aucune œuvre reproduite ni nommée. Aucune réharmonisation (D4).

## Ce qui est mesuré, ce qui ne l'est pas

| Couche | Source | Statut |
|---|---|---|
| Batterie : patterns `ride` (ride sur 1, 2, la croche de 2, 3 ; pédale ; grosse caisse ; caisse claire), phrases de 2 et 4 mesures | GMD `jazz` 3/4 : 217 mesures, 1 batteur, 3 groupes de mesures (68, 62 et 87 mesures) | mesuré, **un seul batteur**, groupes de mêmes fichiers (part max 0,82) |
| Batterie : patterns `brush` et `hat` ; relances | — | **règles de métier** (aucun balai dans le GMD, aucun fichier `fill` en 3/4) : caisse claire sur le trois, caisse claire et tom sur le deux et le trois, grosse caisse sur le un |
| Swing | JTD (976 pistes, 4/4) : courbe tempo → rapport par piste | **porté du 4/4 au 3/4** ; les deux fichiers GMD 3/4 mesurent 0,656 (120 BPM) et 0,598 (200 BPM), soit 0,06 sous la courbe, comme le GMD 4/4 (0,646 contre 0,718) : la JTD est retenue |
| Tempo 160 (100–230) | GMD 120 et 200 BPM ; iReal : valse jazz medium à vive | choix de métier entre les deux fichiers |
| Basse (fenêtre, approches, notes de passage, arrivées) | `wjazzd_swing` (4/4) | **aucune mesure de basse en 3/4** : poids repris du swing, non mesurés sur une valse |
| Comping, grilles, arc, habillage, mixage | — | **règles de métier**, aucun corpus ouvert |

## Écriture

3/4, 12 pas par mesure, 4 pas par pulsation, `default_bpm` 160, swing 0,70. Un chorus = une passe AABA de
32 mesures, quatre chorus par air (128 mesures, 2 min 24 s à 160 BPM).

| Couche | Réglage |
|---|---|
| Basse | marche aux noires (`walking`) ; **deux temps = une blanche sur le un puis une noire sur le trois** (`two_feel` en 3/4 : attaques aux pulsations 0 et 2) ; arrivée sur l'accord à chaque changement ; croches fantômes rares |
| Accords | voicings `rootless` par défaut, `color` 9/11/13, `register_drift` ±5 ; dix rythmes d'une mesure (4 `sparse`, 3 `medium`, 3 `dense`) sur les temps 2 et 3 avec anticipations, douze phrases |
| Batterie | kit jazz 128/32 par défaut (ride 51, pédale 44, grosse caisse 36, caisse claire 38, tom 43) ; kit balais 128/40 : `tap` 38 et `swirl` 40 |
| Arc | exposition à deux temps ou à la blanche pointée (`three_feel`, A7), deux chorus de marche (ride, un break à la mesure 24 ou un chorus plus fort), retombée à la blanche pointée avec stop-time ; au-delà de quatre chorus : sommet puis corps en boucle |
| Timing | basse sur la batterie, accords en retard de 0,2 pas (valeurs de `wjazzd_swing`, non mesurées en 3/4) |
| Mixage | niveaux de `wjazzd_swing` ; **non mesuré au LUFS** pour ce profil |

## Habillage par graine (`session.toml`)

Tempo et swing (118–134, 150–170, 186–206 BPM ; swing 0,72, 0,69, 0,64 : plages séparées de plus de 8 BPM),
tonalité majeure (fa, si bémol, mi bémol, sol, do, ré), comping (piano 0, piano électrique « Tine » 4, guitare de
jazz 26, orgue 16), basse (contrebasse 32, aux doigts 33, fretless 35), kit (baguettes 3 / balais 1, avec l'étiquette
de batterie du kit), voicings (`rootless`, `drop2`, `open`, `guide_tones`), registre, énergie, arc (classique,
`deux_temps_long`, `sans_sommet_orgue` : l'orgue prend le comping aux deux chorus du corps,
`balais_puis_baguettes` : balais à l'exposition et à la retombée, ride du kit jazz au corps). 3 × 6 × 4 × 3 × 2 × 4 ×
2 × 3 × 4 = **41 472** habillages. Programmes vérifiés dans `assets/GeneralUser-GS.sf2`.

## Limites et manques de schéma

- **Blanche pointée** (A7) : `bass_feel = "three_feel"` joue une attaque par mesure de 3/4 (une arrivée de plus à chaque
  changement d'accord) ; l'exposition et la retombée la tirent (poids de métier, sans corpus), le deux temps garde blanche + noire.
- **Basse sans mesure en 3/4**, comping sans corpus ; les poids de basse viennent du swing 4/4.
- Un seul batteur, deux fichiers : la variété de batterie mesurée est celle d'un batteur ; les relances sont de métier.
- Swing de la JTD (4/4) porté au 3/4, 0,06 au-dessus du GMD.

## Régénérer

`cd ~/Project/bascanada/harmonium_lab && .venv/bin/python <moteur>/specs/019-bibliotheque-accompagnements/b25_batterie.py gmd_jazz_waltz > <moteur>/profiles/gmd_jazz_waltz/percussion.toml`

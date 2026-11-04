# B2 + B5 — Ballade jazz (`wjazzd_ballad`) et valse jazz (`gmd_jazz_waltz`)

Date : 2026-11-03. Branche `wquintal/b25-ballade-valse`, partie de `main` à `eecbd3f` (`TUNING_VERSION` 26,
**inchangé** : profils neufs, aucun profil existant modifié ; seuls des fichiers s'ajoutent aux dumps).
Aucun fichier de `src/` touché. Aucun audio lancé (D5, D7) : vérification hors ligne.

## Ce qui est livré

| Profil | Source qui nomme le style | Mètre, tempo | Habillage (`[session]`) |
|---|---|---|---|
| `wjazzd_ballad` (B2) | WJazzD `rhythmfeel = BALLAD` : 10 solos, 9 exploitables, 294 mesures, 48,8–67,7 BPM, « trop peu » | 4/4 (16 pas), 58 (44–84), swing 0,72 | 31 variantes en 9 groupes : **38 880** habillages |
| `gmd_jazz_waltz` (B5) | GMD `jazz` 3/4 : 2 fichiers (120 et 200 BPM), un batteur, 217 mesures | 3/4 (12 pas), 160 (100–230), swing 0,70 | 31 variantes en 9 groupes : **41 472** habillages |

Chaque dossier : `profile.toml`, `percussion.toml` (généré), `arc.toml`, `session.toml`, `swing.toml`, `dynamics.toml`,
`arrangement.toml`, `set.toml`, `cells.toml`, `README.md` (sources, mentions de licence, mesure → valeur, limites).
Les deux sont sous grille (`loop`, AABA de 32 mesures : 3 A × 2 ponts pondérés par graine), sans mélodie, sans Odin.

- **Ballade** : balais dominants (kit 128/40, `tap` 38 et `swirl` 40), deux temps à l'exposition et à la retombée,
  marche lente au corps (3 chorus), comping clairsemé tenu, voicings riches (`drop2`, `open` = *spread*, `rootless`,
  `upper_structure`) avec `color` 9/11/13 et `register_drift`, basse A4 en notes tenues avec arrivées
  (`arrival`, `approach.side`, notes répétées, `two_feel`).
- **Valse** : basse aux noires ou, au deux temps, blanche sur le un puis noire sur le trois ; comping sur les temps 2 et 3
  avec anticipations ; batterie GMD 3/4 (ride 1, 2, croche de 2, 3), balais ou baguettes ; arc sur 4 chorus.
- Générateurs et mesures : `b25_batterie.py` (batterie des deux profils), `b25_distance.py` (distance de la batterie
  au GMD) ; test de contrat `harmonium_core/tests/ballad_waltz_profile_tests.rs`.

## Construction de la batterie (`b25_batterie.py`)

Mesures `beat` du GMD ramenées à la grille swinguée de E7 (pas 2 de chaque pulsation retardé au rapport mesuré du
fichier ; E7 ne lit que le 4/4, la lecture est étendue ici à 12 pas), k-moyennes sur grosse caisse, caisse claire, ride
et pédale, un pattern par groupe retenu et par densité (seuils de probabilité par pas, identiques à B4), phrases de 2 et
4 mesures vues dans les fichiers, relances groupées.

| | `wjazzd_ballad` | `gmd_jazz_waltz` |
|---|---|---|
| Fichiers retenus | 5 sur 8 candidats (`jazz` et `jazz/swing`, 4/4, ≤ 112 BPM ; rapport de swing mesuré ≥ 0,54 : les deux fichiers `jazz/swing` du batteur 10 sont droits, 0,49 et 0,52, écartés ; un fichier de 2 mesures écarté) | 2 sur 2 |
| Mesures, batteurs | 206, 3 (d1 : 110, d3 : 32, d7 : 64) | 217, 1 |
| Groupes retenus | 3 sur 4 (50 / 40 / 84 mesures ; part max d'un fichier 0,88 / 0,35 / 0,35) ; le groupe du batteur 3 (32 mesures, un fichier) écarté | 3 sur 3 (68 / 62 / 87 mesures ; part max 0,82) |
| Seuils d'éligibilité | 10 mesures, 2 fichiers, 1 batteur, part max 0,9 (ceux de L2 écartent presque tout) | 12 mesures, 1 fichier, 1 batteur |
| Patterns, phrases, relances | 21 patterns (ride, brush, hat × 3 densités), 14 suites, 1 relance mesurée (22 fichiers `fill` d'un batteur) | 21 patterns, 16 suites, 3 relances **de métier** |

Étiquettes : `ride` = patterns **mesurés** ; `brush` = la figure du ride du groupe jouée à la frappe de balai
(règle de métier, aucun balai dans le GMD) plus le balayage ; `hat` = balayage et pédale (exposition et retombée,
valables pour les deux kits). Vélocités : forme des accents du groupe, niveau de la voix ; bornes de service par rôle
(grosse caisse relevée de 19 à 30 : valeurs de métier, **ni calées au LUFS ni écoutées**).

## Preuves

### Habillages distincts sur 16 graines (graines 1 à 16, première mesure de la vérité terrain `--record-truth`)

| | `wjazzd_ballad` | `gmd_jazz_waltz` | Possibles |
|---|---:|---:|---|
| **habillages distincts** (tempo, tonalité, programmes, voicings, kit) | **16 / 16** | **15 / 16** | 38 880 / 41 472 |
| classes de tempo à ± 8 BPM (48–54, 58–66, 72–78 ; 118–134, 150–170, 186–206) | 3 (48 à 77 BPM) | 3 (118 à 194 BPM) | 3 |
| tonalités | 6 | 5 | 6 |
| programmes de comping | 3 (0, 16, 26) | 4 (0, 4, 16, 26) | 3 / 4 |
| programmes de basse | 3 | 3 | 3 |
| kits | 2 (balais 40 : 14 graines, jazz 32 : 2) | 2 (jazz 32 : 13, balais 40 : 3) | 2 |
| familles de voicings | 4 | 4 | 4 |
| arcs | 4 sur 5 | 4 sur 4 | 5 / 4 |

La vérité terrain porte les variantes (`MeasureSnapshot.session`) et les programmes (`instruments`).

### `seed-variety --chorus 4` (16 graines × 4 chorus), contre `wjazzd_swing` (état d'avant : profil swing de la phase 018)

| Couche | `wjazzd_swing` | `wjazzd_ballad` | `gmd_jazz_waltz` |
|---|---:|---:|---:|
| accompagnement : distance inter-graines | 38 % | **60 %** | **63 %** |
| accompagnement : séquences distinctes | 16 / 16 | 16 / 16 | 16 / 16 |
| comping rythme : inter-graines | 68 % | 41 % | 45 % |
| comping voicings : inter-graines | 46 % | **98 %** | **94 %** |
| basse : inter-graines | 59 % | **87 %** | **87 %** |
| batterie : inter-graines | 19 % | **30 %** | **39 %** |
| batterie : paires de chorus identiques | 100 % | **29 %** | **21 %** |
| batterie : chorus distincts par graine | 1,0 / 4 | **2,6 / 4** | **2,9 / 4** |
| orchestration : séquences distinctes | 1 / 16 | **8 / 16** | **10 / 16** |
| orchestration : places qui varient | 0 % | **100 %** | **100 %** |
| intra-graine : mesures changées d'un chorus au suivant (accompagnement) | 99 % | 99 % | 100 % |

Le rythme du comping inter-graines baisse (68 → 41 et 45 %) : les phrases (A1) remplacent le tirage mesure par mesure
(structure au lieu du hasard, même effet que B4 et B8). Les accords (symboles) ne varient que par la boucle de grille
tirée (6 séquences distinctes sur 16 : 3 A × 2 ponts) ; la transposition et le tempo n'y figurent pas (signature
relative à la tonique).

### Distance au corpus

**Basse de la ballade contre WJazzD `BALLAD`** (`python -m harmonium_lab.backing.validate`, groupe `all`, 16 graines
de 128 mesures ; témoin = `wjazzd_swing`, même procédure ; 0 = identique) :

| Famille | `wjazzd_swing` | `wjazzd_ballad` |
|---|---:|---:|
| approche | 0,213 | 0,118 |
| arrivée | 0,536 | 0,285 |
| rythme harmonique | 0,282 | 0,124 |
| intervalles | 0,164 | 0,078 |
| passage | 0,118 | 0,034 |
| registre | 0,138 | 0,116 |
| **agrégée** | **0,244** | **0,130** |

Gain de 0,114 (règle de P2 : pas plus de 1 point de perte : respectée, ici gain). Calage : trois itérations sur les poids
d'approche (0,137 → 0,130, `/tmp/harmonium-019/b25/tune`) ; les parts mesurées restent éloignées (approche aucune 61 %
contre 72 %, diatonique 24 % contre 13 %) : la source est « trop peu » (9 solos), le calage s'arrête sans sur-ajuster.
La part d'arrivée sur la basse (64,5 %) suit celle de la source (55 %) renormalisée de la part « autre » (18 %,
non représentable sous D4). La valse n'a **aucun corpus de basse** (aucune mesure 3/4 dans WJazzD, E7 ne lit que le 4/4) :
aucune distance à rapporter.

**Batterie rendue contre le GMD** (`b25_distance.py`, Bray-Curtis des fréquences d'attaque par pas, 16 graines ; témoin :
même profil sans arc ni session, `percussion.toml` de `wjazzd_swing` (4/4) ou de la fixture `grille_valse_jazz` (3/4) ;
moyenne des quatre métriques `any`, `kick`, `snare`, `hat`, comme B8) :

| Profil | `any` | `kick` | `snare` | `hat` | **moyenne** | témoin |
|---|---:|---:|---:|---:|---:|---:|
| `wjazzd_ballad` | 0,338 | 0,806 | 0,589 | 0,309 | **0,510** | 0,651 |
| `gmd_jazz_waltz` | 0,260 | 0,547 | 0,364 | 0,349 | **0,380** | 0,542 |

`ride` : 0,829 (ballade) contre 0,194 (témoin), parce que la ballade joue **aux balais** par défaut (frappe sur la note
38, comptée comme caisse claire) et que l'exposition et la retombée n'ont pas de ride : voulu, non mesuré à l'oreille.
La grosse caisse de la ballade (0,806) est très au-dessus du GMD parce que les fichiers lents de d1 la jouent
beaucoup plus souvent que la grosse caisse plumée des patterns retenus (seuil 0,75 à la densité 0).

## Contrats testés (`ballad_waltz_profile_tests.rs`, les deux profils, 16 graines, un chorus par chorus d'air)

| Contrat | Test |
|---|---|
| métrique (4/4 sur 16 pas, 3/4 sur 12), tempo dans la plage du profil, swing dans 0,5–0,75, aucune note de mélodie, changements d'accord sur une pulsation, même grille d'un chorus au suivant | `la_metrique_le_tempo_et_la_forme_aaba_restent_coherents` |
| basse : une attaque au pas de chaque changement d'accord publié, dans l'accord, dans 31–51 (hors break et stop-time) | `la_basse_arrive_dans_l_accord_a_chaque_changement` |
| comping : chaque note dans l'accord publié ou ses tensions naturelles sans altération (ou l'accord de la mesure suivante pour une attaque anticipée) | `le_comping_ne_contredit_pas_l_accord_publie_ni_ses_alterations` |
| session : même variante sur toute la session, ≥ 8 habillages distincts sur 16 graines, exposition à deux temps (basse aux pas 0 et 8 en 4/4, 0 et 8 en 3/4), marche (une attaque par pulsation) au corps | `les_sessions_varient_et_la_basse_change_de_feel_sur_l_arc` |

Mutations vérifiées : fenêtre de basse resserrée (31 → 40) fait échouer le contrat de basse ; deux temps attendu aux
pas 0 et 4 fait échouer le contrat de session.

## Limites et manques de schéma (pour A7)

1. **Blanche pointée (une attaque par mesure de 3/4)** : `bass_feel` n'a que `walking` et `two_feel` ; `two_feel` en 3/4
   joue blanche (un) + noire (trois). Manque listé, pas contourné : un feel à trois pulsations (`attack_pulses` 3, ou un
   nombre libre) l'exprimerait.
2. **Étiquettes de batterie de session et d'arc** : l'arc d'un chorus qui déclare des étiquettes de batterie passe devant
   celles de la session ; la ballade utilise `hat` (valable pour les deux kits) à l'exposition et à la retombée, mais
   un tirage « balais ou baguettes » ne peut pas lier le kit à l'étiquette de l'arc : manque d'une référence de
   l'arc à l'étiquette de la session (ou d'un kit par étiquette).
3. **Swing de la ballade** hors de la plage mesurée de la JTD (aucun tempo sous 102 BPM) ; **swing de la valse** porté
   du 4/4. Les courbes de A6 n'ont pas de provenance par métrique.
4. **Groupes de session indépendants** : le tempo et le swing sont dans la même variante, mais comping, basse, kit et
   voicings se tirent sans lien (pas de règle « orgue ⇒ voicings plus tenus »).
5. **Balais** : `tap` (38) et `swirl` (40) sont deux touches du même instrument « Brushed Snares » du kit 40 ; le
   balayage n'est pas une note longue modulée. Aucune écoute (D5).
6. **Mixage** : niveaux de `wjazzd_swing`, non mesurés au LUFS pour ces deux profils.
7. **Sources** : ballade « trop peu » (9 solos, 5 fichiers GMD lents, 3 batteurs) ; valse : deux fichiers, un batteur.

## Reproduire

```sh
cd ~/Project/bascanada/harmonium_lab
.venv/bin/python <moteur>/specs/019-bibliotheque-accompagnements/b25_batterie.py wjazzd_ballad > <moteur>/profiles/wjazzd_ballad/percussion.toml
.venv/bin/python <moteur>/specs/019-bibliotheque-accompagnements/b25_batterie.py gmd_jazz_waltz > <moteur>/profiles/gmd_jazz_waltz/percussion.toml
.venv/bin/python <moteur>/specs/019-bibliotheque-accompagnements/b25_distance.py
.venv/bin/harmonium-lab seed-variety --profile <moteur>/profiles/wjazzd_ballad --seeds 16 --chorus 4 --engine-bin <moteur>/target/release/harmonium-cli
.venv/bin/python -m harmonium_lab.backing.validate --profile ballade=<moteur>/profiles/wjazzd_ballad --reference references/wjazzd-ballad/measures-bass.json --seeds 16 --bars 128
cargo test -p harmonium_core --test ballad_waltz_profile_tests
```

## Écoute (facultative, D5) : liens pour l'orchestrateur

Deux graines contrastées par profil, ouvertes dans le studio en pause ; `&bar=33` démarre au deuxième chorus.

| Profil | Lien | Ce qui change |
|---|---|---|
| `wjazzd_ballad` | `harmonium://open?profile=wjazzd_ballad&seed=11` | 50 BPM, la bémol, guitare de jazz, contrebasse, balais, voicings `drop2`, deux temps tenu |
| `wjazzd_ballad` | `harmonium://open?profile=wjazzd_ballad&seed=7&bar=33` | 76 BPM, fa, piano, contrebasse, baguettes douces (ride), voicings sans fondamentale, deux temps tenu |
| `gmd_jazz_waltz` | `harmonium://open?profile=gmd_jazz_waltz&seed=7` | 194 BPM (swing 0,61), ré, piano électrique, contrebasse, kit jazz, voicings `open`, deux temps long |
| `gmd_jazz_waltz` | `harmonium://open?profile=gmd_jazz_waltz&seed=2&bar=33` | 129 BPM (swing 0,74), fa, orgue, basse fretless, balais puis baguettes, voicings sans fondamentale |

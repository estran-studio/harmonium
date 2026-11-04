# Profil `gmd_latin_brazilian_bossa` (phase 019, B3)

> **Limite de la source, en tête.** La batterie vient du style `latin/brazilian-bossa` du Groove MIDI Dataset : **deux
> fichiers `beat`, un seul batteur** (631 mesures, 19,9 minutes, 127 BPM dans les deux fichiers, kit électronique Roland
> TD-11), fiabilité **« trop peu »** (L2). Ce profil décrit **un batteur, pas la bossa nova**. Le GMD ne contient **aucune
> percussion latine**. La basse est calibrée sur WJazzD `LATIN` (hard bop et post-bop sur rythme latin) qui **ne distingue ni
> bossa, ni samba, ni afro-cubain** et **n'observe aucune croche de basse**. Le comping, la basse en croches, le shaker, les
> relances, l'arc et les grilles sont des **règles de métier** (idiome traditionnel), dites telles ci-dessous.

Bossa nova sous une grille d'accords : guitare nylon au comping syncopé, basse fondamentale-quinte anticipée, ride ou
charleston en croches, pédale aux temps 2 et 4, grosse caisse sur 1, « et » du 2, 3 et « et » du 4, cross-stick. Fait pour
training : l'utilisateur fournit sa grille, le profil joue dessous ; sans grille, cinq boucles de seize mesures. Le nom est
celui de la source qui définit le style (principe VII). Feel droit (16 pas, swing 0,5), aucune piste `Melody`, aucun Odin :
GeneralUser GS seulement (programmes GM, kit banque 128).

## Provenance et droits

| Couche | Source | Licence | Mention (aussi dans `[meta] corpus`) |
|---|---|---|---|
| Batterie du kit : groupes de mesures, probabilités par pas, micro-timing, phrases, tempo | Groove MIDI Dataset v1.0.0, `latin/brazilian-bossa` : 2 fichiers `beat` (1 batteur, 631 mesures, « trop peu »). SHA256 de l'archive vérifié par le labo avant toute lecture : `651cbc52…1dcc1e` | CC BY 4.0 | Gillick, Roberts, Engel, Eck, Bamman, Learning to Groove with Inverse Sequence Transformations, ICML 2019 ; Groove MIDI Dataset v1.0.0, Google LLC, CC BY 4.0 |
| Basse : registre, approches avant un changement, rythme harmonique, longueurs de chorus, tempos | Weimar Jazz Database v2.1, `rhythmfeel = LATIN` : 27 solos dont 23 avec mesures valides, 1 639 mesures, « assez » | ODbL 1.0 (contenus DbCL 1.0) | « Contains information from the Weimar Jazz Database (Jazzomat Research Project, HfM Weimar), which is made available here under the Open Database License (ODbL); individual contents under the Database Contents License (DbCL 1.0). » (§4.3) |
| Comping, basse en croches, shaker, relances, arc, grilles par défaut, habillage, mixage | **Règles de métier** écrites pour ce profil (idiome traditionnel de la bossa) | — | Aucun corpus ouvert : voir « Règles de métier » |

Aucune mesure d'un enregistrement, aucune note de solo, aucune ligne de basse n'est copiée : seuls des agrégats (probabilités
par pas, parts, centiles) servent. Les fichiers MIDI du GMD ne sont pas dans le dépôt. Grilles par défaut : formes communes
écrites pour ce profil, relatives à la tonique, aucune œuvre reproduite ni nommée, **aucune réharmonisation (D4)**.
Mesures du labo : `harmonium_lab/references/gmd-latin-brazilian-bossa/`, `references/wjazzd-latin/`, `references/l2-ingestion.md`.

## Mesure → valeur

Tout nombre ci-dessous vient d'une lecture de `measures-drums.json` (GMD) ou `measures-bass.json` (WJazzD), ou d'une mesure du
profil rendu (commandes dans « Reproduire »). « Fiabilité » : celle de L2 (`trop peu` sous 3 fichiers et 60 mesures pour le
GMD ; `assez` pour WJazzD tous solos, `juste` dans la classe de tempo 120–180, `trop peu` ailleurs).

| Paramètre | Valeur | Mesure source (effectif · fiabilité) |
|---|---|---|
| `meter.steps_per_bar`, `default_swing_ratio` | 16 pas, 0,5 | GMD : 2 fichiers « straight », rapport de swing médian 0,4896 (p10 0,4829, p90 0,4962) · trop peu |
| `default_bpm` | 127 | GMD : 127 dans les deux fichiers (p10 = p50 = p90) · trop peu |
| `min_bpm`, `max_bpm` | 104, 180 | WJazzD LATIN, tempo de solo p10 103,96, médiane 152,7, p90 227,5 (27 solos) ; classe 120–180 « juste » (15 solos), < 120 « trop peu » (un solo avec mesures) ; 104 = p10, 180 = borne de la classe « juste » |
| Tempos de session | lent 104–112, médium 122–132, vif 152–180 | mêmes bornes : lent = p10 de WJazzD, médium autour des 127 du GMD, vif = de la médiane de WJazzD à 180 ; aucune plage n'étend la mesure |
| Longueur du chorus | 16 mesures (boucles de 8 jouées deux fois ou de 16) | WJazzD LATIN : 69 chorus complets, 8 mesures (31), 16 (9), 24 (9), 32 (9), autres (11) · assez |
| Rythme harmonique | 1 accord par mesure ; changement à la blanche (temps 3) dans 5 mesures sur 16 de la boucle « tourne à la blanche » et 2 sur 16 des huit mesures ; 86 % de mesures à un accord dans le rendu (16 graines × 48 mesures) | WJazzD : 86,9 % de mesures à un accord (IC 79–93 %), changement au temps 1 pour 84,9 %, au temps 3 pour 15,1 % ; accord de 4 temps 58,7 %, 2 temps 29,8 %, 8 temps 8,4 % · assez |
| `bass.low`, `high`, `walk_low`, `walk_high` | 34, 45, 28, 53 | WJazzD : hauteur de basse p1 31, p50 40, p99 50, étendue 28–53 ; arrivées 35–45 (fenêtre de 12 demi-tons 36–47 : 85 %) ; la fondamentale se place dans 34..45 · assez |
| `bass.passing` | aucune 599, chromatique 216, degré de gamme 185 (millièmes) | WJazzD : approche du temps 4 avant un changement à la barre : aucune 59,9 %, chromatique 21,6 % (IC 17–26 %), diatonique 18,5 % (IC 11–29 %) · 1 205 événements, 22 solos · assez |
| Batterie : groupes de mesures | `ride_croches` 244 mesures, `ride_noires` 46, `charleston_croches` 21 | GMD : timekeeper ride 75 % (473 mesures), charleston fermé 9,5 % (60), aucun 14,4 % (91) ; figure en croches 42 % · trop peu |
| Grosse caisse | pas 0, 6, 8, 14 (90, 86, 86, 84 %), fantôme aux pas 1 et 9 (37, 31 %) | GMD : [0, 6, 8, 14] est le motif le plus fréquent (31 % des mesures) · trop peu |
| Pédale du charleston | pas 4 et 12 (84 %), pas 0 et 8 (42 %) | GMD · trop peu |
| Cross-stick, caisse claire | cross-stick aux pas 4, 10, 2 (25, 26, 18 %) ; caisse claire fantôme sur presque tous les pas (15–41 %) | GMD : coups probabilistes (A6) à leur probabilité mesurée · trop peu |
| Micro-timing par voix | ride −0,04 / −0,06 pas (sur le temps / hors), charleston −0,09 / −0,11, pédale −0,2, cross-stick −0,06 / −0,07, caisse claire −0,08 / −0,11, grosse caisse +0,03 / −0,17, tom −0,14 / −0,15 | GMD : médianes, 631 mesures ; la pédale hors temps (+0,225) dépasse la borne ±0,2 du schéma : non représentée |
| Phrases de batterie | 7 suites mesurées de 2 et 4 mesures × 3 densités, 9 phrases « groove tenu » | GMD : suites de groupes vues ≥ 2 fois (≥ 3 pour un groupe tenu), 1 batteur · trop peu |
| `performance.tracks.Percussion*` | aucun `jitter` | un `jitter` de batterie partage son flux avec la basse à notes de passage |
| `mix.tracks` | accords −16,3 dB, basse −18,7 dB, batterie −10,0 dB, maître +25 dB | LUFS intégrés en solo (voir « Mixage ») |

## Règles de métier (aucune mesure, dites comme telles)

- **Comping de la guitare nylon** (GeneralUser GS 24, notes guides : tierce, septième, tension) : douze rythmes d'une mesure en trois
  étiquettes (`sparse` : tenues ; `medium` : la « clave » de la bossa ; `dense` : croches syncopées et notes fantômes à `probability`
  réduite), seize phrases de 2 et 4 mesures. **Cycle de deux mesures** : « trois » aux pas 0, 6, 12 puis « deux » aux pas 4, 8 et
  l'attaque anticipée du pas 14 (`anticipate` : elle joue l'accord de la mesure suivante). Ce cycle vient de l'idiome traditionnel ;
  il n'est **ni une clave contrainte (`[clave]` absent), ni une mesure** : le GMD ne montre pas de cycle de deux mesures commun à ses deux
  fichiers (`b3/gmd_latin_brazilian_bossa_parite.py` : fichier 1, écart maximal de 0,02 à 0,09 entre mesures paires et impaires sur la grosse caisse, la
  pédale et le ride ; fichier 2, jusqu'à 0,27).
- **Basse fondamentale-quinte anticipée** (contrebasse 32) : trois gabarits (poids 5, 3, 2) : fondamentale au un, quinte au « et » du
  deux, fondamentale au trois (puis, selon le gabarit, une quatrième attaque) ; au pas 14 une note courte de la basse de l'accord **suivant**
  (`pickup` `when = "bar_end"`). La position des attaques suit la grosse caisse mesurée du GMD (0, 6, 8, 14), mais **rien dans WJazzD ne
  mesure les croches, l'anticipation ni les silences de la basse**.
- **Shaker GM** (note 82, très bas, densité 2 seulement), **relances** (le GMD n'a aucun fichier `fill` bossa) et leur poids de « pas de
  relance » (`fills_none_weight`), **niveau de service de chaque voix** de batterie (le TD-11 ne se transpose pas au kit GM).
- **Arc** sur cinq chorus, **grilles par défaut**, **couleur** et dérive de registre des voicings, **habillage par graine** (tonalités,
  timbres, kits, voicings, registre, énergie, arc « orchestre »), **mixage**.

## Ce que le profil joue

- **Grille** : cinq boucles de seize mesures, tonique tirée par la graine (do par défaut ; ré, fa, sol, si♭, mi♭ par la session), mode ionien.
  Tourne I–vi–ii–V ×4 (poids 3) ; cycle ii–V–I de quatre mesures ×4 (3) ; tourne à la blanche I, vi, ii–V ×5 puis I (3) ; huit mesures
  I (2), IV, ii, V, I, vi, ii–V jouées deux fois (2) ; vamp ii–V–I–IV ×4 (2). Mesures à un accord : 86 % dans le rendu (WJazzD 86,9 %).
- **Basse** : voir ci-dessus ; arrivée sur la basse publiée à chaque changement, approche chromatique ou diatonique avec les parts de WJazzD
  (le moteur l'applique à toute note qui change de hauteur, pas seulement avant un changement d'accord à la barre : plus souvent que la mesure).
- **Comping** : voir ci-dessus. Voicings : notes guides par défaut ; la session tire aussi drop 2, shell, structure supérieure, sans fondamentale.
  `color` tire une tension naturelle (9, 11, 13) jamais en conflit avec la grille.
- **Batterie** (kit jazz 128/32 par défaut) : 9 patterns (3 groupes × 3 densités), 30 phrases, 8 relances. Densité 0 : timekeeper, grosse caisse et
  pédale ; 1 : plus grosse caisse fantôme et cross-stick probabiliste ; 2 : plus caisse claire fantôme, tom médium, shaker.
- **Arc** (cinq chorus par air) : exposition (comping `sparse` ou `medium`, **charleston en croches**, doux), deux chorus de corps (comping
  `medium` ou `dense`, **ride**, un break de la guitare à mi-chorus ou un break guitare + basse avant le chorus suivant), sommet (`dense`, plus
  fort, stop-time sur les deux dernières mesures une fois sur deux), retombée (`sparse`, charleston, stop-time). Au-delà de cinq chorus : sommet puis
  corps et sommet en boucle.

## Habillage par graine (`session.toml`, D6)

Neuf groupes tirés indépendamment, 33 variantes, **64 800 habillages possibles** ; aucun groupe ne règle le champ d'un autre.

| Groupe | Variantes | Source |
|---|---|---|
| tempo | lent 104–112, médium 122–132 (poids 2), vif 152–180 | WJazzD LATIN et GMD (voir plus haut) |
| tonalité | do, ré, fa, sol, si♭, mi♭ (ionien) | métier |
| comping | guitare nylon 24 (3), acier 25, jazz 26, piano 0, piano électrique « Tine » 4 | métier |
| basse | contrebasse 32 (3), basse aux doigts 33 (2), fretless 35 | métier |
| kit | standard 0, room 8, jazz 32 (2), balais 40 (2) | métier |
| voicing | notes guides, drop 2, shell, structure supérieure, sans fondamentale | métier |
| registre | normal (2), comping une octave plus bas | métier |
| énergie | doux (0,85 / 0,8), moyen, plein (1,1 / 1,1) | métier |
| arc | `arc.toml` (2), « orchestre » : exposition aux balais, sommet au piano électrique, retombée balais et guitare nylon | métier |

Seize graines montrent **16 habillages distincts** (6 tonalités, 15 triplets de programmes comping/basse/kit, 4 kits, 5 voicings, 3 tempos, 5 grappes
de tempo à ± 8 BPM parce que la plage vive couvre 28 BPM). Le programme de chaque piste est publié dans `MeasureSnapshot.instruments` et la
variante de chaque groupe dans `MeasureSnapshot.session`.

## Mixage

Réglé sur le LUFS intégré en solo de chaque piste (`--solo-track`, `ffmpeg ebur128`), 48 mesures, hors ligne, en silence
(`b3/gmd_latin_brazilian_bossa_mix.py`). Cibles de service : accords −26, basse −25, batterie −25,5, mix ≈ −21 à −22 LUFS ; crête vraie ≤ −1 dBFS.

| Piste | Livré, graines 1 et 2 (avec session) | Livré, graines 3 à 8 | Instruments par défaut (sans session), graines 1 et 2 |
|---|---:|---:|---:|
| Chords | −25,5 | −25,9 (−30,7 à −22,8) | −26,9 |
| Bass | −24,8 | −25,1 (−26,6 à −24,0) | −25,9 |
| Batterie (grosse caisse + reste du kit, somme en puissance) | −26,0 | −25,5 | −25,9 |
| dont `PercussionLow` (grosse caisse) / `PercussionHigh` | −30,6 / −27,9 | −30,4 / −27,2 | −30,7 / −27,6 |
| Mix | −20,9 | −20,9 (−21,9 à −19,8) | −21,9 |
| Crête vraie du mix (dBFS) | −2,7 | −1,3 | −2,1 |

Le niveau dépend des programmes tirés (guitare, piano, kit) : l'écart d'une graine à l'autre est celui des timbres, pas du réglage.
`PercussionLow` et `PercussionHigh` partagent le canal 9 du kit GM : le niveau de `PercussionHigh` gagne ; la grosse caisse se règle par
la vélocité de sa voix (47 + écarts mesurés), plus douce que le reste du kit parce que « Jazz Bass Drum » sonne plus fort à vélocité égale.

## Contrat et preuves

`harmonium_core/tests/gmd_latin_brazilian_bossa_profile_tests.rs` : 12 tests sur le flux de `MeasureSnapshot` (graines 1, 2, 3, 17 et 1 à 16) : grille par
défaut = une des cinq boucles à la tonique tirée, reprise à l'identique à chaque chorus (≥ 3 grilles, ≥ 4 tonalités sur 16 graines) ; basse sur la
basse publiée à chaque changement, dans la fenêtre 34..45 ; basse limitée à fondamentale, quinte, anticipation du pas 14 et approches ; comping dans
l'accord publié (celui de la mesure suivante pour une attaque anticipée), tessiture, sans fondamentale sauf drop 2 et structure supérieure ; cycle de
deux mesures « trois » puis « deux » ; batterie dans le kit, grosse caisse [0, 6, 8, 14], pédale 4 et 12, shaker GM sur `PercussionHigh`, à la densité 2 aussi ;
charleston à l'exposition et à la retombée, ride au corps et au sommet ; feel droit, tempo dans la plage de la variante tirée ; arc : exposition et
retombée plus légères que le sommet, stop-time de la retombée ; arc « orchestre » : instruments par chorus ; ≥ 8 habillages distincts sur 16 graines ;
même graine, même sortie, ≥ 14 comping, ≥ 14 batteries, ≥ 12 basses distincts. Preuves chiffrées (variété, basse contre WJazzD, distance au GMD,
mutations) : `specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa.md`.

## Limites

- **Un batteur, deux fichiers.** Les patterns de batterie sont des composites de probabilités par pas d'un seul batteur sur un kit électronique : un
  idiome de batteur, pas la bossa nova. La caisse claire du GMD est un nuage de coups fantômes sur presque tous les pas (15 à 41 %), que le profil ne rend
  qu'en densité 2 : sa distance au GMD est la plus grande des cinq métriques.
- **Pas de percussion latine mesurée.** Le GMD n'en a aucune : le shaker est de métier, exclu de toute comparaison au GMD. Aucune clave, aucun tamborim,
  aucun agogô : une bossa de kit.
- **Basse.** WJazzD `LATIN` est du hard bop et du post-bop sur rythme latin (14 solos post-bop, 8 cool, 4 hard bop, 1 bebop) : il ne distingue pas la
  bossa et n'a ni croche, ni silence, ni anticipation. L'arrivée sur la basse de l'accord (57 % [46–68 %] dans WJazzD) n'est pas reproduite : un gabarit arrive
  toujours sur la basse publiée (100 %). L'anticipation du pas 14 est de métier ; le moteur réattaque ensuite cette note au temps 1 (une bossa liée ne
  la rejouerait pas). L'approche est appliquée à toute note qui change de hauteur : plus souvent que la mesure (WJazzD : avant un changement à la barre).
  Registre plus étroit que WJazzD (p1 34, p99 45 contre 31 et 50) : la fondamentale et la quinte se placent dans une fenêtre de douze demi-tons.
- **Tempos.** Le GMD n'a qu'un tempo (127). La plage lente (104–112) est le p10 de WJazzD, classe « trop peu » (un solo) ; la plage vive (152–180) est la moitié
  haute de la classe « juste ». Rien n'est étendu au-delà de la mesure ; l'étendue des tempos de bossa est une limite de la source.
- **La grille ne dépend pas de la tonalité tirée.** La tonalité (do, ré, fa, sol, si♭, mi♭) transpose la grille relative ; le mode est une étiquette (ionien) : les
  boucles sont écrites en majeur, aucune n'est mineure.
- **Programmes à la barre.** Un changement de programme au chorus part au premier pas de la mesure ; les attaques anticipées du pas 14 de la mesure d'avant sonnent
  encore avec l'ancien instrument (A2).
- **Micro-timing** : médianes seulement, sans dispersion (voir le tableau) ; la pédale hors temps n'est pas représentée.
- **Aucune écoute (D5).** Tout est jugé par des chiffres et des contrats ; le propriétaire écoute dans le studio.

## Manques du schéma (rien n'est contourné)

1. **Anticipation liée** : le gabarit de basse réattaque la note du pas 14 au temps 1 ; il n'y a pas de `tie` entre l'anticipation (`pickup`) et l'arrivée.
2. **Approche limitée à un changement d'accord à la barre** : `passing` n'a que `next_target_differs` et `chord_change_inside_bar` ; aucune règle « avant la barre
   seulement », donc plus d'approches que WJazzD n'en mesure.
3. **Micro-timing de batterie** : `performance.tracks.<piste>.voices` borne l'écart à ±0,2 pas (la pédale hors temps, +0,225, ne passe pas) et n'a pas de dispersion
   propre à une voix sans `jitter` (le `jitter` de batterie déplace la basse à notes de passage).
4. **Groupes de session indépendants** : tempo, kit et timbres se tirent sans lien (balais à 180 BPM, piano électrique et contrebasse à 104 BPM) ; aucun groupe n'en conditionne un autre.
5. **Grille indépendante de la tonalité tirée** : la tonalité (do, ré, fa, sol, si♭, mi♭) transpose la grille relative ; le mode est une étiquette (ionien). Une boucle
   mineure demande des boucles écrites à part et un tirage de mode lié à la boucle (aucun lien entre la boucle tirée et le mode tiré).

Non utilisé, pas un manque : `[clave]` aligne des phrases de comping et de batterie sur un cycle de deux mesures continu d'un chorus à l'autre ; ici les chorus ont seize
mesures (nombre pair) et les phrases de 2 et 4 mesures se découpent depuis le début de la passe, le cycle « trois puis deux » reste donc aligné sans lui. Un chorus de longueur
impaire fournie par l'utilisateur le désalignerait (le profil ne le teste pas).

## Reproduire

```sh
# batterie (labo en lecture seule ; archive GMD vérifiée par SHA256)
cd ~/Project/bascanada/harmonium_lab
.venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa_generer.py --out <worktree>/profiles/gmd_latin_brazilian_bossa
# habillages, basse contre WJazzD (depuis ~/Project/bascanada/harmonium, CLI du worktree)
python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py habillages --cli <cli> --profile <dossier> --seeds 16 --bars 48
python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py basse --cli <cli> --profile <dossier> --seeds 16 --bars 48 \
  --reference ~/Project/bascanada/harmonium_lab/references/wjazzd-latin/measures-bass.json
# variété, mixage, distance au GMD : voir le compte rendu
cargo test -p harmonium_core --test gmd_latin_brazilian_bossa_profile_tests
```

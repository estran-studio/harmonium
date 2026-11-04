# B3 — Profil `gmd_afrocuban` (afro-cubain)

Date : 2026-11-03. Branche `wquintal/b3-latin`, partie de `main` à `eecbd3f` (`TUNING_VERSION` 26, **inchangé** : aucun fichier
sous `src/`, aucun mécanisme ajouté). Un dossier de profil neuf, un test de contrat, quatre scripts de reproduction. Les profils
existants ne sont pas touchés.

## Ce qui est livré

| Fichier | Contenu |
|---|---|
| `profiles/gmd_afrocuban/` | `profile.toml`, `percussion.toml` (généré), `arc.toml`, `dynamics.toml`, `set.toml`, `arrangement.toml`, `cells.toml` (banque minimale, jamais jouée), `session.toml`, `README.md` |
| `harmonium_core/tests/gmd_afrocuban_profile_tests.rs` | 16 tests de contrat sur le flux de `MeasureSnapshot` |
| `specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_generer.py` | génère tous les fichiers du profil sauf `README.md` (lecture seule de `references/gmd-afrocuban/measures-drums.json`) |
| `…/gmd_afrocuban_mix.py`, `gmd_afrocuban_mix.json` | calage du mixage au LUFS en solo, niveaux retenus |
| `…/gmd_afrocuban_distance.py`, `gmd_afrocuban_variete.py` | distance au GMD de la batterie du kit ; résumé de `seed-variety --chorus 4` |
| `…/preuves/gmd_afrocuban-*.json` | sorties des mesures de ce compte rendu |

| Profil | Sources | Chorus | Tempo | Instruments | Contenu |
|---|---|---|---|---|---|
| `gmd_afrocuban` | GMD `afrocuban` droit (1 fichier, 98 mesures, « trop peu ») et `afrocuban/rhumba` droit (1 fichier, 103 mesures ; 18 fills, « juste ») ; basse : WJazzD `LATIN` (23 solos, « assez ») ; clave, montuno, tumbao, percussions latines : **métier** | 16 mesures, 5 chorus par air | 105 (105 à 128), droit 0,5 | piano 0 (électrique 4, guitare nylon 24), contrebasse 32 (33, 35), kit 128/0 (8, 16, 32 ; 40 aux balais en arc « orchestre ») | 5 boucles de grille, 12 motifs de montuno et 42 phrases, 4 gabarits de tumbao, 36 patterns, 72 phrases et 4 relances de batterie, arc, `[session]` : 9 groupes, 29 variantes |

## Choix de conception

- **Ce que le GMD donne, ce qu'il ne donne pas.** Le style `afrocuban` compte 3 fichiers `beat` (2 batteurs, 276 mesures, « juste ») :
  1 fichier droit (98 mesures) et 2 ternaires (178 mesures). Retenus : le groupe droit et `afrocuban/rhumba` droit (la grille du profil est
  de 16 pas) pour le **kit**, les 18 fills du rhumba pour les **relances**. Écartés, dit : les mesures ternaires (une grille de triolets est
  un autre profil), `afrocuban/bembe` (ternaire), `afrocuban/calypso`. Le GMD ne contient **aucune percussion latine** (Roland TD-11) :
  clave, montuno, tumbao, congas, cloche, bongos, timbales, shaker, maracas, güiro sont des **règles de métier** (`l2-ingestion.md` § 5),
  dites telles dans `profile.toml`, `percussion.toml` et le README. Statut juridique d'un idiome rythmique non vérifié : propriétaire.
  WJazzD `LATIN` est du hard bop sur rythme latin, pas du salsa : seuls registre et approches en sont tirés ; le tumbao n'est pas « mesuré »
  (croches non observables).
- **Clave** (`[clave] direction = "either"`, fixture `grille_clave`) : son, deux mesures (« trois » pas 0, 6, 12 ; « deux » pas 4, 8), voix
  `claves` (GM 75) dans chaque pattern de batterie ; phrases de batterie et de comping alignées (`clave = "three" | "two"`), 2 et 4 mesures.
  Les relances ne reprennent la main qu'au pas 13 : elles ne touchent jamais la clave (voir « Manques »).
- **Montuno** : piano, guajeo syncopé d'une mesure par côté de clave et par étiquette (`sparse`, `medium`, `dense`), deux variantes
  chacune, attaques du pas 14 anticipées, notes fantômes probabilistes ; `color` et `register_drift` (A5) pour varier les voicings.
- **Tumbao** : quatre gabarits (`templates`), pas 6 et 12, le un joué, anticipation du changement au pas 14 (`pickup`, `bar_end`) ; le
  quatrième, au temps 4 libre, laisse la note de passage (WJazzD LATIN : aucune 60, chromatique 22, diatonique 19) approcher le changement.
- **Section latine** : congas 62 / 63 / 64 et timbale grave 66 sur `PercussionLow` ; claves 75, cloche (53 au corps, cowbell 56 au sommet),
  bongos 60 / 61, timbale aiguë 65, shaker 82, maracas 70, güiro 73 / 74 sur `PercussionHigh`. Chaque motif en deux versions, une par côté.
- **Arc** : exposition `sparse`, corps `medium` (cloche de ride), sommet `dense` (cowbell, bongos, güiro, timbales, stop-time une fois sur
  deux), retombée `sparse` avec stop-time. Arc nommé « orchestre » (A2) : exposition aux balais, sommet aux cuivres (programme 61).
- **Tempo** : les tempos mesurés dans la source (105, 110, 128) n'ont pas l'étendue pour trois plages séparées de plus de 8 BPM ; le profil
  garde les trois valeurs et **n'élargit pas** (deux grappes à ± 8 BPM).
- **Habillage** (D6) : tempo, tonalité (six, **sans `[fixed_tonality]`**), comping (piano, électrique, guitare nylon), basse (32, 33, 35), kit
  (0, 8, 16, 32), voicings (`close_triad`, `drop2`, `rootless`, `open`), registre, énergie, arc.
- **Mixage** : réglé au LUFS en solo, pas à l'oreille (tableau plus bas).

## Preuves

Toutes depuis `~/Project/bascanada/harmonium`, binaire `/Users/william.quintal/orca/workspaces/harmonium/b3-latin/target/release/harmonium-cli`
(`W` = le worktree). Aucun son, aucune lecture audio : exports hors ligne (`--export --record-truth`, `--record-wav` pour le LUFS).

### Habillages distincts (16 graines, 48 mesures)

```sh
python3 $W/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py habillages --cli <cli> --profile $W/profiles/gmd_afrocuban \
    --seeds 16 --bars 48 --out preuves/gmd_afrocuban-habillages.json
```

**16 habillages distincts sur 16 graines** ; 2 grappes de tempo à ± 8 BPM (105 à 110, puis 128) ; 6 tonalités ; 11 triplets (comping, basse,
kit) ; variantes distinctes par groupe : tempo 3, tonalité 6, comping 3, basse 3, kit 4, voicing 4, registre 2, énergie 2, arc 2 ; swing 0,5
partout. Le kit lu au premier chorus comprend le 40 (balais) des graines à arc « orchestre » (6, 7, 9, 10, 11, 14, 15). 20 736 habillages
possibles (3 × 6 × 3 × 3 × 4 × 4 × 2 × 2 × 2).

### Variété (`seed-variety --chorus 4`, 16 graines)

```sh
harmonium-lab seed-variety --profile <dossier> --seeds 16 --chorus 4 --engine-dir ~/Project/bascanada/harmonium \
    --engine-bin <cli> --label B3 --work-dir /tmp/harmonium-019/b3/afrocubain/var/work_<n> -o var/<n>.json
python3 $W/specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_variete.py var preuves/gmd_afrocuban-variete.json
```

« Avant » = copie de `profiles/gmd_afrocuban` sans `session.toml` (et sans sa ligne `session =`). V0 : `wjazzd_swing` remesuré avec le même binaire
(profil non touché). Chaque cellule : mesures changées d'un chorus au suivant / paires de chorus identiques / chorus distincts par graine /
suites distinctes sur 16 graines / distance graduée entre graines.

| Couche | `gmd_afrocuban` (livré) | `gmd_afrocuban` sans `session.toml` | `wjazzd_swing` (V0 remesuré) |
|---|---|---|---|
| accords | 0 % / 100 % / 1,0 / 5 / 76 % | 0 % / 100 % / 1,0 / 5 / 76 % | 0 % / 100 % / 1,0 / 1 / 0 % |
| comping : rythme | 88 % / 0 % / 4,0 / 16 / 62 % | 84 % / 0 % / 4,0 / 16 / 61 % | 80 % / 0 % / 4,0 / 16 / 68 % |
| comping : voicings | 87 % / 0 % / 4,0 / 16 / **99 %** | 82 % / 0 % / 4,0 / 16 / 95 % | 74 % / 0 % / 4,0 / 16 / 46 % |
| basse | 49 % / 0 % / 4,0 / 16 / 91 % | 46 % / 0 % / 4,0 / 16 / 90 % | 92 % / 0 % / 4,0 / 16 / 59 % |
| batterie | 100 % / 0 % / 4,0 / 16 / 45 % | 100 % / 0 % / 4,0 / 16 / 44 % | 0 % / 100 % / 1,0 / 11 / 19 % |
| accompagnement | 100 % / 0 % / 4,0 / 16 / **75 %** | 100 % / 0 % / 4,0 / 16 / 74 % | 99 % / 0 % / 4,0 / 16 / 38 % |

Arc (notes par mesure, chorus 1 à 4) : livré 31,4 / 50,9 / 51,2 / 29,8 (étendue 52,5 %) ; sans session 31,8 / 51,3 / 51,0 / 30,0 ; V0 19,5 / 19,6 / 19,3 /
19,6 (1,6 %). Programmes par piste sur 16 graines : livré comping {0, 4, 24}, basse {32, 33, 35}, kit {0, 8, 16, 32, 40} ; avant et V0 : un seul chacun.

Lecture : la distance entre graines de l'accompagnement passe de 38 % (V0) à 75 % (livré) ; la session n'ajoute presque rien aux couches de notes
(+ 1 point) mais change les **timbres, le tempo, la tonalité, le registre, les voicings** (comping : voicings 95 → 99 %), que `seed-variety` ne
compte qu'en partie (hauteurs relatives à la tonique, vélocités et programmes ignorés) : l'habillage se mesure dans le tableau précédent. La
basse varie de 49 % d'un chorus au suivant (notes de passage seulement : le plan de gabarits est fixé par air, limite du moteur déjà notée par B4)
contre 92 % pour la marche de V0. Les accords ne changent pas d'un chorus à l'autre (boucle de la grille, 5 grilles distinctes sur 16 graines).

### Basse contre WJazzD `LATIN` (16 graines, 48 mesures)

```sh
python3 $W/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py basse --cli <cli> --profile $W/profiles/gmd_afrocuban --seeds 16 --bars 48 \
    --reference ~/Project/bascanada/harmonium_lab/references/wjazzd-latin/measures-bass.json --out preuves/gmd_afrocuban-basse.json
```

(762 mesures ; seul ce que la source mesure est comparé ; les croches n'y sont pas observables.)

| Mesure | `gmd_afrocuban` | WJazzD `LATIN` | Lecture |
|---|---|---|---|
| Registre p1 / p50 / p99 (hauteur à la pulsation) | 36 / 41 / 46 | 31 / 40 / 50 | médiane tenue ; étendue plus étroite : fenêtre de douze demi-tons (`low = 35`, `high = 46`), pas d'octave |
| Étendue | 35..46 | 28..53 | idem |
| Arrivée sur la basse de l'accord au changement | 100 % (285 changements) | 57 % [46–68] | le moteur n'arrive que sur la basse publiée (D4) ; l'annotation de WJazzD n'est pas une marche idéale |
| Approche à un demi-ton de l'arrivée, à la barre | 15,5 % | chromatique 21,6 % | |
| Approche à deux demi-tons | 25,0 % | diatonique 18,5 % | « deux demi-tons » n'est pas « diatonique » (limite du script) |
| Pas d'approche | 59,5 % | 59,9 % | |
| Changement au temps 3 | 0 % | 15,1 % | grilles écrites à la barre |
| Mesures à un accord | 100 % | 86,9 % | idem |
| Tempo min / max | 105 / 128 | p10 / p50 / p90 104 / 153 / 228 | WJazzD est du jazz : autre ordre de grandeur |

### Distance au GMD de la batterie du kit (copie non calibrée)

```sh
~/Project/bascanada/harmonium_lab/.venv/bin/python $W/specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_distance.py \
    --out preuves/gmd_afrocuban-distance-gmd.json
```

Bray-Curtis des fréquences d'attaque par pas (0 = identique), 16 graines × 48 mesures, **kit seulement** : les voix latines (claves, congas,
bongos, timbales, cloche de ride 53 et cowbell 56 de métier, shaker, maracas, güiro) sont **exclues** du rendu et de la référence (`ride_bell`),
le GMD n'en ayant aucune. Référence : toutes les mesures `beat` 4/4 droites de `afrocuban` (98) et `afrocuban/rhumba` (103). Copie non calibrée :
le même profil dont `percussion.toml` est celui de `wjazzd_swing`, sans arc ni session.

| Référence | Métrique | Profil | Sans arc | Non calibré |
|---|---|---|---|---|
| `afrocuban` + `rhumba` (201 mesures) | tout / grosse caisse / caisse claire / charleston / ride | 0,303 / 0,212 / 0,403 / 0,288 / 0,520 | 0,296 / 0,221 / 0,410 / 0,273 / 0,508 | 0,418 / 1,000 / 1,000 / 0,727 / 0,699 |
| | **moyenne** (tout, grosse caisse, caisse claire, charleston) | **0,301** | 0,300 | **0,786** |
| `afrocuban` seul (98) | moyenne | 0,368 | 0,360 | 0,808 |
| `rhumba` seul (103) | moyenne | 0,324 | 0,327 | 0,761 |

L'arc coûte 0,001 point (règle de P2 : pas plus de 1 point de perte) ; le profil est à 0,30 du GMD contre 0,79 pour la batterie d'un autre style.
Le kit est un composite de deux batteurs, ni une copie ni le style entier.

### Mixage (LUFS intégrés en solo, graines 1 et 2, 48 mesures)

```sh
python3 $W/specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_mix.py --out $W/specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_mix.json \
    --measure-only
```

| Piste | LUFS | Cible |
|---|---|---|
| Accords | −26,1 | −26 |
| Basse | −25,4 | −25 |
| Grosse caisse et congas graves (`PercussionLow`) | −29,4 | |
| Reste du kit (`PercussionHigh`) | −27,8 | |
| Batterie (somme en puissance) | −25,5 | −25,5 |
| Mix | −21,3 | −21 à −22 |

Crête vraie du mix : −4,3 dBFS (≤ −1). Niveaux : accords −16,9 dB, basse −19,1 dB, percussion −10,4 dB (les deux pistes ; le niveau de
`PercussionLow` est sans effet, canal 9 partagé), maître +25 dB. Aucune écoute (D5).

## Contrat et mutations

`harmonium_core/tests/gmd_afrocuban_profile_tests.rs`, 16 tests, ~12 s ; `cargo test -p harmonium_core --test gmd_afrocuban_profile_tests`.
Le profil est lu depuis son dossier ; un test vérifie que la copie distribuée dans le binaire est la même.

| Contrat | Test |
|---|---|
| copie distribuée = dossier | `le_profil_distribue_est_celui_du_dossier` |
| grille par défaut = une des cinq boucles écrites (relatives à la tonique lue dans `position.key`), reprise identique à chaque chorus, ≥ 4 grilles sur 16 graines | `la_grille_par_defaut_est_une_des_boucles_ecrites_…` |
| **clave respectée mesure par mesure** : `claves` aux pas 0, 6, 12 ou 4, 8 exactement, alternance continue sur cinq chorus, **3-2 et 2-3 vus** sur 16 graines ; aux trois densités et trois intensités | `la_clave_alterne_trois_deux_mesure_par_mesure_dans_les_deux_sens` |
| comping dans le motif du côté de la clave de la mesure (et pas dans celui de l'autre) ; cloche de la batterie du même côté (pas 6 « trois », 14 « deux ») ; les deux familles de motifs ne se confondent pas | `le_comping_et_la_batterie_jouent_du_meme_cote_que_la_clave`, `les_montunos_des_deux_cotes_ne_se_confondent_pas` |
| tempo = 105 / 110 / 128 selon la variante, constant sous toute intensité ; bornes 105..128 | `le_tempo_reste_dans_la_plage_mesuree_…` |
| basse sur la basse publiée à chaque changement, dans 35..46 (étendue 28..53) | `la_basse_arrive_sur_la_basse_publiee_…` |
| tumbao : attaque au pas 6 (≥ 95 % des mesures), au pas 12 (≥ 70 %), anticipation au pas 14 sur la basse de la mesure suivante (≥ 55 %), notes de passage à un ou deux demi-tons de la cible | `la_basse_joue_le_tumbao_…` |
| comping dans l'accord publié (celui de la mesure suivante pour une attaque anticipée), tessiture 48..84 (36..72 au registre « grave ») | `le_comping_ne_joue_que_des_notes_de_l_accord_publie_…` |
| batterie dans le kit, voix latines sur leurs pistes, grosse caisse ≥ 90 % des mesures, congas partout ; cloche : rien à l'exposition, cloche de ride au corps, cowbell au sommet, rien à la retombée | `la_batterie_reste_dans_le_kit_…` |
| relances : au pas 13, uniquement en fin de chorus, jamais sur la clave | `la_relance_ne_touche_pas_la_clave_…` |
| feel droit, 16 pas, aucune mélodie | `le_feel_est_droit_en_seize_pas` |
| arc : exposition et retombée plus légères que le sommet, stop-time des deux dernières mesures (accords et basse sur le un, batterie qui continue) | `l_arc_creuse_l_ecart_…` |
| même graine = même sortie ; 16 comping, 16 batteries, 16 basses distincts sur 16 graines | `la_graine_change_…` |
| habillage : ≥ 8 distincts sur 16 graines, ≥ 2 grappes de tempo, ≥ 4 tonalités, ≥ 3 comping, ≥ 2 basses, ≥ 2 kits, ≥ 3 voicings (programmes du deuxième chorus, que l'arc « orchestre » ne masque pas) | `l_habillage_de_la_graine_change_…` |
| arc « orchestre » : balais à l'exposition et à la retombée, cuivres au sommet, programmes de la session au corps ; arc classique : programmes constants | `l_arc_orchestre_change_…` |

**Mutations vérifiées** (29, sur une copie du profil lue par une copie du test, jamais dans `profiles/` ; chacune échoue, puis est annulée) :

| # | Mutation | Test qui échoue |
|---|---|---|
| M1 | `[clave]` toujours 3-2 | clave alterne dans les deux sens |
| M2 | un coup de clave du pas 6 au pas 7 (pattern de densité 0) | clave alterne (densités) |
| M3 | phrase de comping `clave = "two"` au lieu de `"three"` | comping et batterie du même côté |
| M4 | phrase de batterie `clave = "two"` au lieu de `"three"` | clave alterne ; comping et batterie du même côté |
| M5 | variante de tempo 128 → 120 | tempo |
| M6 | plus d'anticipation (`pickup`) aux trois premiers gabarits | tumbao |
| M7 | plus d'attaque au pas 0 (gabarit 1) | basse arrive sur la basse publiée ; arc (stop-time) |
| M8 | fenêtre de basse 35..46 → 32..43 | basse arrive sur la basse publiée |
| M9 | plus de notes de passage | tumbao |
| M10 | variante « grave » sans décalage d'octave | comping dans l'accord et la tessiture |
| M11 | tension naturelle (neuvième) hors de l'accord publié | comping dans l'accord |
| M12 | cowbell 56 → 54 (tambourin) | batterie dans le kit ; comping et batterie du même côté |
| M13 | conga grave sur `PercussionHigh` | batterie dans le kit (pistes) |
| M14 | swing 0,5 → 0,6 | feel droit |
| M15 | le sommet joue `medium` (plus de cowbell) | batterie, cloche selon l'arc |
| M16 | retombée sans stop-time | arc |
| M17 | relances `at = "any"` au lieu de `part_end` | relance en fin de chorus |
| M18 | une boucle écrite change d'accord (IV → II) | grille par défaut |
| M19 | un seul comping, une seule basse | habillage |
| M20 | tonalités réduites à deux | habillage |
| M21 | arc « orchestre » : sommet sans cuivres | arc « orchestre » |
| M22 | arc « orchestre » : exposition sans balais | arc « orchestre » |
| M23 | motif de montuno « trois » joué sur les pas du côté « deux » | comping et batterie du même côté |
| M24 | `min_bpm` 105 → 100 | tempo |
| M25 | la piste `Melody` entre à l'arrangement | feel droit (aucune mélodie) |
| M26 | relances jamais tirées (`fills_none_weight` = 100 000) | relance |
| M27 | deux familles de voicings seulement | habillage |
| M28 | un seul kit | habillage |
| M29 | l'exposition joue `dense` et fort | arc |

Deux mutations n'avaient d'abord rien fait échouer et ont durci les tests : M2 (la clave n'était lue qu'à la densité 1 : le test balaie maintenant
les trois densités et intensités) et M28 (les kits comptés au premier chorus incluaient les balais de l'arc « orchestre » : ils se lisent
maintenant au deuxième chorus). « Même graine, même sortie » compare deux exécutions : aucune mutation de données ne le rend faux, le contrat
ne peut échouer que si le moteur devient non déterministe (couvert par `harmonium_determinism`).

## Manques du schéma et du moteur (rien n'est contourné ; tâche A7)

1. **`[[fills]]` sans côté de clave** : une relance prend la main à `start_step` sans savoir si la mesure est « trois » ou « deux » ; elle ne peut pas
   commencer avant le pas 13 sans risquer le coup du pas 12. D'où des relances réduites à la queue (pas 13 à 15) des fills du rhumba. Un champ `clave`
   sur `[[fills]]` autoriserait des relances de mesure entière (les 18 fills ont un premier coup médian au pas 5).
2. **Variante de clave « rumba » non tirable** (troisième coup au pas 11 au lieu de 12) : les étiquettes de l'arc, qui nomment les patterns de chaque
   chorus, l'emportent sur celles d'une variante de session ; une variante de clave par graine entrerait en conflit de tags. Il faudrait des tags de
   session qui composent avec ceux de l'arc (ou un champ de session désignant la famille de clave).
3. **Le mode tiré par la session n'est qu'une étiquette** : une boucle de grille ne dépend pas de la tonalité tirée (mineure ou majeure) ; toutes les
   tonalités sont ioniennes.
4. **Gabarit de basse et changement au temps 3** : un gabarit dit les attaques de la mesure sans savoir où les accords changent ; deux gabarits sur
   quatre font sonner la quinte du pas 6 par-dessus le pas 8, donc les grilles ne changent d'accord qu'à la barre (WJazzD LATIN : 15 % au temps 3).
5. **Stop-time** : `keep_steps` garde les attaques de la mesure listées, pas l'anticipation du pas 14 de la mesure précédente ; un tumbao strict
   (qui tait le un) n'a donc pas de stop-time utilisable ; les gabarits jouent le un.
6. **Note de passage** : `next_target_differs` approche toute cible de basse différente, pas seulement un changement d'accord à la barre (mesuré par
   WJazzD) ; la part jouée est plus haute que la part mesurée.
7. **`PercussionLow` sans niveau propre** (canal 9 du kit GM partagé) : l'équilibre grosse caisse et congas / reste du kit se règle par la vélocité
   de chaque voix, pas par piste.
8. **Plage de tempo de session** : `bpm = [bas, haut]` suppose une étendue mesurée ; la source n'en a que trois points (105, 110, 128).

## Limites

- **Un idiome de batteur.** Kit de deux fichiers à un batteur chacun, « trop peu » ; vélocités de pads TD-11 ramenées à un niveau de service du kit GM
  (seule la forme des accents est gardée). Le style entier (276 mesures, 2 batteurs) est « juste » ; sa moitié ternaire n'est pas représentée.
- **Tout ce qui est latin est de métier** : clave, montuno, tumbao, congas, cloche, bongos, timbales, shaker, maracas, güiro. Aucune écoute (D5) ;
  des rythmes écrits de mémoire peuvent se tromper. Statut juridique à vérifier par le propriétaire.
- **Tumbao avec le un joué** (le tumbao strict le tait) ; arrivée sur la basse de l'accord à 100 % (57 % annoté dans WJazzD) ; registre de basse
  plus étroit que le mesuré (douze demi-tons contre dix-neuf) ; rythme harmonique plus lent que le mesuré (100 % de mesures à un accord, 87 %
  mesurés).
- **Tempo** : 105 à 128 seulement (source) ; un salsa rapide (160 et plus) n'est pas joué.
- **Habillage** : le « même mode » de toutes les tonalités ; les kits 0, 8, 16, 32 partagent les mêmes touches latines (le timbre change, pas la
  section rythmique) ; arc « orchestre » : cuivres au comping 61 sur des voicings de piano, jamais écoutés.
- **Mixage** réglé au LUFS en solo, aucune écoute.
- **Déterminisme** : le profil entre dans `StyleProfile::distributed()` (compilé par `build.rs`) : les dumps de `harmonium_determinism` et le test
  `profile_files_tests` le comptent à l'intégration ; non régénérés ici (hors périmètre).

## Écoute (liens ouverts par l'orchestrateur, pas par moi)

| Lien | Ce qui change |
|---|---|
| `harmonium://open?profile=gmd_afrocuban&seed=5` | 3-2, 105 BPM, si bémol, grille I–IV–V7–I ; piano électrique Tine, basse fretless, kit room, voicings sans fondamentale, doux ; arc classique |
| `harmonium://open?profile=gmd_afrocuban&seed=6` | 2-3, 128 BPM, do, vamp Im7–IV7 ; guitare nylon, contrebasse, kit balais à l'exposition puis room, voicings serrés, soutenu ; arc « orchestre » (balais, puis cuivres au sommet : `&bar=49`) |

## Reproduire

```sh
python3 specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_generer.py --out profiles/gmd_afrocuban \
    --mix specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_mix.json
cargo test -p harmonium_core --test gmd_afrocuban_profile_tests
```

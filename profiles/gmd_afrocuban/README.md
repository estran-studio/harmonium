# Profil `gmd_afrocuban` (phase 019, B3)

Accompagnement afro-cubain en 16 pas droits sous une grille d'accords : clave son 3-2 / 2-3 tirée par air, montuno
(comping), tumbao (basse), section rythmique latine (congas, cloche, bongos, timbales, shaker, maracas, güiro) et kit.
Fait pour training : l'utilisateur fournit sa grille, le profil joue dessous ; sans grille, cinq boucles de seize
mesures tirées par la graine. Aucune piste `Melody`. Tout sonne avec GeneralUser GS (programmes GM, kits de la banque
128), sans Odin. `session.toml` habille chaque graine (tempo, tonalité, instruments, voicings, kit, registre, énergie,
arc).

Le nom est celui de la source qui définit le style (principe VII) : le style `afrocuban` du Groove MIDI Dataset. Cette
source ne contient **aucune percussion latine** (kit électronique Roland TD-11, dix batteurs au total, un ou deux pour
ces fichiers) : ce que le profil dit de la **clave, du montuno, du tumbao, des congas, de la cloche** n'est mesuré dans
aucune source ouverte ; c'est un **idiome traditionnel écrit en règles de métier**, dit tel dans `profile.toml`, dans
`percussion.toml` et ici. WJazzD `rhythmfeel = LATIN` n'est **pas du salsa** (hard bop et cool sur rythme latin ; la
base ne distingue ni bossa, ni samba, ni afro-cubain) : le tumbao n'y est pas « mesuré » (les croches de basse n'y sont
pas observables).

**Statut juridique des idiomes.** La clave, le montuno et le tumbao sont des formules rythmiques traditionnelles, écrites
ici de mémoire de métier, sans partition ni enregistrement. Aucune source ouverte ne les fournit (`harmonium_lab
references/l2-ingestion.md` § 5, recherches du 2026-11-02) et le statut juridique d'un idiome rythmique n'a pas été
vérifié : il relève du propriétaire.

## Provenance et droits

| Couche | Source | Licence | Mention (aussi dans `[meta] corpus`) |
|---|---|---|---|
| Kit : probabilités par pas, vélocités relatives, micro-timing | Groove MIDI Dataset v1.0.0 : `afrocuban` 4-4 droit (1 fichier `beat`, 98 mesures, 1 batteur, « trop peu ») et `afrocuban/rhumba` 4-4 droit (1 fichier, 103 mesures, 1 batteur, « trop peu »). SHA256 de l'archive vérifié par le labo avant toute lecture : `651cbc52…1dcc1e` | CC BY 4.0 | Gillick, Roberts, Engel, Eck, Bamman, « Learning to Groove with Inverse Sequence Transformations », ICML 2019 ; Groove MIDI Dataset v1.0.0, Google LLC, CC BY 4.0 |
| Relances : queue des fills | GMD `afrocuban/rhumba` : 18 fichiers `fill`, 1 batteur (« juste ») | CC BY 4.0 | idem |
| Tempos | GMD `afrocuban` (3 fichiers : 105 droit, 105 et 128 ternaires) et `afrocuban/rhumba` (110) | CC BY 4.0 | idem |
| Basse : registre, approche avant un changement | Weimar Jazz Database v2.1, `rhythmfeel = LATIN` : 27 solos, 23 avec mesures valides, 1 639 mesures, 40,0 min (« assez ») | ODbL 1.0 (contenus DbCL 1.0) | « Contains information from the Weimar Jazz Database (Jazzomat Research Project, HfM Weimar), which is made available here under the Open Database License (ODbL); individual contents under the Database Contents License (DbCL 1.0). » (ODbL §4.3) |
| Clave, montuno, tumbao, congas, cloche, bongos, timbales, shaker, maracas, güiro, phrases alignées, grilles, arc, habillage, mixage | Règles de métier écrites pour ce profil | — | Idiomes traditionnels, aucune source ouverte ; statut juridique : propriétaire |

Aucune note de solo, aucune ligne de basse, aucune mesure d'un enregistrement n'est copiée : seuls des agrégats
(probabilités par pas, comptes, centiles) servent. Les fichiers MIDI du GMD ne sont pas dans le dépôt. Mesures du labo :
`harmonium_lab/references/gmd-afrocuban/`, `references/wjazzd-latin/`, `references/l2-ingestion.md`.

**Ce qui est retenu du GMD, et pourquoi.** `afrocuban` droit (98 mesures) et `afrocuban/rhumba` droit (103 mesures,
18 fills) : seuls fichiers en 16 pas droits, la grille du profil. `afrocuban` ternaire (2 fichiers, 178 mesures) et
`afrocuban/bembe` (ternaire, 104 mesures et 29 fills, dont 28 ternaires) ne sont **pas rabattus** sur 16 pas : une grille
de triolets est une autre grille (`steps_per_bar = 12`, `[swing]`), donc un autre profil. `afrocuban/calypso` n'est pas un
style cubain demandé. Ni le ternaire ni le bembé ne sont donc représentés (voir « Limites »).

## Mesure → valeur

| Paramètre | Valeur | Mesure source (commande) | Effectif, fiabilité |
|---|---|---|---|
| `steps_per_bar`, `default_swing_ratio` | 16 pas, 0,5 | rapport de swing 0,5 (p10 à p90) ; `grid_fit.within_quarter_step` 0,779, écart moyen à la grille 0,1727 pas. `jq '.styles.afrocuban.groups["4-4:straight"].beat \| {swing_ratio, grid_fit}' references/gmd-afrocuban/measures-drums.json` | 1 fichier, 98 mesures, 1 batteur, « trop peu » |
| `default_bpm` | 105 | médiane 105 du groupe droit. `jq '.styles.afrocuban.groups["4-4:straight"].beat.bpm.p50'` | idem |
| `min_bpm`, `max_bpm` | 105, 128 | `afrocuban` ternaire : moyenne 116,5, p10 107,3, p90 125,7 sur deux fichiers, soit 105 et 128 (`jq '.styles.afrocuban.groups["4-4:ternary"].beat.bpm'`) ; rhumba 110 | 3 fichiers `afrocuban`, 1 fichier rhumba |
| Variantes de tempo | 105, 110, 128 | les trois tempos mesurés des fichiers du style (voir « Habillage ») | idem |
| Kit : grosse caisse | pas 3 (0,93) et 11 (0,84) ; 6 (0,26), 14 (0,35) | `jq '.styles.afrocuban.groups["4-4:straight"].beat.per_step.kick'` ; rhumba : 3 (0,65), 6 (0,57), 11 (0,43), 0 (0,29) | « trop peu » |
| Kit : charleston pédale | pas 0, 4, 8, 12 (0,68 à 0,70) et 2, 6, 10, 14 (0,41 à 0,45) | `…per_step.hat_pedal` | « trop peu » |
| Kit : ride, caisse claire, cross-stick, toms | ride pas 0 (0,52), 8 (0,44) …, caisse claire pas 9 (0,50), 10 (0,44), 7 (0,41) …, cross-stick du rhumba pas 0, 3, 7, 10, 12 (0,37 à 0,42), tom aigu pas 14 (0,31) | `…per_step.<rôle>` ; la médiane de vélocité de la caisse claire est 29 (fantôme) | « trop peu » |
| Seuils par densité | densité 0 : pédale ≥ 0,6, grosse caisse ≥ 0,6 ; 1 : pédale ≥ 0,4, grosse caisse ≥ 0,4, ride ≥ 0,4, caisse claire ≥ 0,4 ; 2 : plus de coups (grosse caisse ≥ 0,25, ride ≥ 0,3, charleston fermé et cross-stick ≥ 0,25, tom ≥ 0,3) | seuils de service dans `gmd_afrocuban_generer.py` (`SEUILS`) ; la probabilité mesurée d'un pas est conservée comme `probability` du coup quand elle est < 0,9 | |
| `performance.tracks.Percussion*` | grosse caisse hors temps −0,175 ; pédale −0,175 ; ride −0,108 / −0,175 ; caisse claire −0,175 / −0,167 ; tom −0,042 / −0,133 ; cross-stick −0,108 / −0,158 pas | médiane du décalage à la grille par rôle, sur le temps et hors du temps ; neutre au-delà de ±0,2 (borne de l'A6 : grosse caisse sur le temps −0,204, pédale hors temps −0,242). `jq '.styles.afrocuban.groups["4-4:straight"].beat.micro_timing'` | « trop peu » |
| Relances | 4 (densités 1 et 2 × `medium` et `dense`) : pas 13 (grosse caisse 0,17, tom aigu 0,44), pas 14 (caisse claire 0,28, tom grave 0,17) | queue des 18 fills rhumba : dernier coup au pas 13 (p10) à 14 (p90), premier coup médian au pas 5, une mesure touchée dans les 18. `jq '.styles["afrocuban/rhumba"].groups["4-4:straight"] \| {fill_position, fill: .fill.per_step}'` ; seuls les pas ≥ 13 de probabilité ≥ 0,15 sont gardés (la clave est jouée jusqu'au pas 12) | 18 fichiers, 1 batteur, « juste » |
| Basse : registre | fondamentale et quinte entre 35 et 46 (`low`, `high`), étendue des passages 28..53 (`walk_low`, `walk_high`) | WJazzD LATIN : hauteur de basse p1 31, p50 40, p99 50, étendue 28..53 (`jq '.groups.all.register' references/wjazzd-latin/measures-bass.json`) ; rendu mesuré : p1 36, p50 41, p99 46, étendue 35..46 (`mesures_latin.py basse`, voir « Basse contre WJazzD LATIN ») | 1 639 mesures, 23 solos, « assez » |
| Basse : approche | `passing` : aucune 60, chromatique 22, degré de gamme 19 | WJazzD LATIN, temps 4 avant un changement à la barre : aucune 60 % [51–66], chromatique 22 % [17–26], diatonique 19 % [11–29] | 1 205 événements, 22 solos |
| Chorus | 16 mesures, 5 chorus par air | WJazzD LATIN : chorus conservés de 8 mesures (31), 16 (9), 24 (9), 32 (9), autres (11) ; 16 mesures est la longueur des boucles écrites (deux demi-boucles de 8 pour deux d'entre elles) | 69 chorus |
| Rythme harmonique | un changement par mesure au plus, sur la barre | WJazzD LATIN : 87 % [79–93] des mesures à un accord, changement au temps 3 dans 15 % des cas ; les boucles du profil ne changent qu'à la barre (voir « Limites ») | 1 639 mesures |
| `mix.tracks` | accords −16,9 dB, basse −19,1 dB, percussion −10,4 dB (les deux pistes), maître +25 dB | LUFS intégrés en solo, graines 1 et 2, 48 mesures (voir « Mixage ») | |

Valeurs de **métier**, aucune mesure (listées dans les commentaires de `profile.toml` et de `percussion.toml`) : la clave,
les motifs de montuno et leurs phrases, les gabarits de tumbao, toutes les voix latines et leurs motifs, les grilles par
défaut, l'arc, l'habillage, le mixage.

## Ce que le profil joue

- **Clave** : son, cycle de deux mesures, `[clave] direction = "either"` : 3-2 ou 2-3 tiré une fois par air (16 graines :
  les deux sens sortent). Mesure « trois » : voix `claves` (GM 75) aux pas 0, 6, 12 ; mesure « deux » : pas 4, 8. Chaque pattern de
  batterie porte la clave ; les phrases de batterie (`clave = "three" | "two"`) et de comping sont alignées : une phrase ne
  commence que sur une mesure de son côté, d'un chorus à l'autre (seize mesures, cycle de deux).
- **Grilles** (relatives à la tonique, aucune œuvre nommée, D4 aucune réharmonisation) : `I7` seize mesures (poids 2) ;
  `I`, `IV`, `V7`, `I` quatre mesures chacun (3) ; `Im7` et `IV7` deux mesures chacun, quatre fois (3) ; `iim7` 2, `V7` 2,
  `Imaj7` 4, deux fois (2) ; `Im7`, `bVII7`, `bVImaj7`, `V7` deux mesures chacun, deux fois (2). Le mode tiré par la session
  est une étiquette : les accords ne dépendent pas de lui.
- **Comping : montuno** (piano 0 ; piano électrique 4, guitare nylon 24 en variante ; voicing `drop2`, famille tirable) : douze
  motifs d'une mesure, quatre par étiquette `sparse` / `medium` / `dense`, deux par côté de clave ; guajeo syncopé autour des
  pas 0, 6, 12 côté « trois » et 4, 8 côté « deux » ; les attaques du pas 14 sont anticipées (`anticipate`) ; les notes fantômes
  de la version `dense` sont des coups probabilistes. 42 phrases de 2 et 4 mesures alignées sur la clave (dont une
  mise en place `part_end` par étiquette et par côté). Couleur (`color`) et dérive de registre (`register_drift`) varient les
  voicings d'une graine et d'un chorus à l'autre.
- **Basse : tumbao** (basse acoustique 32 ; aux doigts 33, fretless 35 en variante) : quatre gabarits, `templates` : attaques aux
  pas 6 et 12, le un joué, anticipation du changement d'accord au pas 14 (`pickup`, `when = "bar_end"`, la basse de l'accord de la
  mesure suivante) ; un gabarit sans anticipation, au temps 4 libre, où une note de passage (60 / 22 / 19, WJazzD LATIN) peut
  approcher le changement. La basse arrive sur la basse publiée à chaque changement.
- **Section rythmique latine** (kit GM de GeneralUser GS, banque 128, programmes 0, 8, 16, 32 et 40 : les touches 56, 60 à 66, 70, 73 à 75 et 82 y
  sont toutes, lues dans `GeneralUser-GS.sf2` avec les fonctions de `a6-roles-latins-sf2.py`) : `PercussionLow` : congas sourde 62 (pas 4, 12), ouverte 63 (6, 14), grave 64 (0, 8), timbale grave 66
  (probabiliste, dense) ; `PercussionHigh` : claves 75, cloche (cloche de ride 53 dans le corps, cowbell 56 au sommet),
  bongos 60 (aigu) et 61 (grave), timbale aiguë 65, shaker 82 (croches), maracas 70, güiro 73 / 74 (dense). Chaque motif existe en
  deux versions, une par côté de clave (la cloche accentue le pas 6 côté « trois », le pas 14 côté « deux »).
- **Kit** (grosse caisse, charleston pédale, ride, caisse claire, cross-stick, toms ; MESURÉ) : 36 patterns = 3 densités × 3
  étiquettes × 2 kits (« afrocuban » et « rhumba ») × 2 côtés de clave ; 72 phrases de batterie de deux mesures ; 4 relances
  (densités 1 et 2 × étiquettes `medium` et `dense`, `part_end`, `start_step = 13`).
- **Arc** (cinq chorus par air, nombre connu) : exposition (comping et batterie `sparse` : clave, congas, doux), deux chorus de
  corps (`medium`, parfois `dense` pour le comping : cloche de ride, shaker, un break du comping à mi-chorus ou des accords et
  de la basse avant le chorus suivant), sommet (`dense` : cowbell, bongos, güiro, timbales, plus fort, stop-time sur les deux
  dernières mesures une fois sur deux), retombée (`sparse`, doux, stop-time sur le un des deux dernières mesures). Au-delà de cinq
  chorus : sommet puis corps et sommet en boucle.

## Habillage par graine (`session.toml`, A2, D6)

Un tirage indépendant par groupe, neuf groupes, 29 variantes, 20 736 habillages possibles ; deux groupes ne règlent jamais le
même champ.

| Groupe | Variantes | Source |
|---|---|---|
| `tempo` | 105 (droit), 110 (rhumba), 128 | tempos mesurés des fichiers du style : pas de plage élargie |
| `tonalite` | do, ré, fa, sol, la, si bémol (ionien) | métier ; **sans `[fixed_tonality]`** : la variante prime (`melody.key_modes` les contient) |
| `comping` | piano 0, piano électrique 4, guitare nylon 24 | métier |
| `basse` | contrebasse 32, aux doigts 33, fretless 35 | métier |
| `kit` | standard 0, room 8, power 16, jazz 32 | métier (kits de GeneralUser GS, mêmes touches latines) |
| `voicing` | `close_triad`, `drop2`, `rootless`, `open` | métier |
| `registre` | normal (poids 2), grave (comping une octave plus bas) | métier |
| `energie` | doux (×0,85), soutenu (×1,15 / ×1,1) | métier |
| `arc` | classique (`arc.toml`) ou « orchestre » (arc nommé : exposition aux balais, kit 40 ; sommet aux cuivres, comping 61 ; retombée aux balais) | métier |

**Étendue de tempo.** Les tempos mesurés dans la source n'ont pas l'étendue pour trois plages séparées de plus de 8 BPM : 105
(droit) et 110 (rhumba) sont à 5 BPM, 128 est à 18 BPM du plus proche. Le profil garde les trois valeurs mesurées (deux
grappes de tempo à ± 8 BPM sur 16 graines : 105 à 110 puis 128) et **n'élargit pas** : un salsa se joue souvent bien plus vite
(160 et plus) ; aucune source ouverte ne le mesure ici, WJazzD LATIN (médiane 152,7 BPM) est du jazz.

**Limite connue.** Une boucle de grille ne dépend pas de la tonalité tirée : le mode (ionien) n'est qu'une étiquette, aucune
boucle n'est mineure par le mode.

## Mixage mesuré

Réglé sur le LUFS intégré en solo de chaque piste (graines 1 et 2, 48 mesures, arc du profil), pas à l'oreille :
`gmd_afrocuban_mix.py`. Cibles de service (celles de B8) : accords −26, basse −25, batterie −25,5, mix ≈ −21 à −22.

| Piste | LUFS en solo (graines 1 et 2) | Cible |
|---|---|---|
| Accords | −26,1 | −26 |
| Basse | −25,4 | −25 |
| Grosse caisse et congas graves (`PercussionLow`) | −29,4 | |
| Reste du kit (`PercussionHigh`) | −27,8 | |
| Batterie (somme en puissance) | −25,5 | −25,5 |
| Mix | −21,3 LUFS | −21 à −22 |

Crête vraie du mix : −4,3 dBFS (≤ −1). Les deux pistes de percussion partagent le canal 9 du kit GM : le niveau de
`PercussionHigh` s'applique aux deux (celui de `PercussionLow` est écrit égal et reste sans effet) ; l'équilibre se règle par
la vélocité de chaque voix.

## Contrat et preuves

`harmonium_core/tests/gmd_afrocuban_profile_tests.rs` (flux de `MeasureSnapshot`, graines 1 à 16 ou 1, 2, 3, 17) : le profil
distribué est celui du dossier ; grille par défaut = une des cinq boucles écrites, reprise identique à chaque chorus (≥ 4
tirées sur 16 graines) ; **clave respectée mesure par mesure** (0, 6, 12 | 4, 8, alternance continue sur cinq chorus, 3-2 et
2-3 vus sur 16 graines) ; **comping et batterie du même côté que la clave** ; tempo dans 105..128 et celui de la variante
(105, 110 ou 128) ; basse sur la basse publiée à chaque changement, dans 35..46 ; tumbao (pas 6 et 12, anticipation au pas 14,
notes de passage à un ou deux demi-tons de la cible) ; comping dans l'accord publié (celui de la mesure suivante pour une
attaque anticipée) et dans la tessiture de la variante ; batterie dans le kit, voix latines sur leurs pistes, cloche de ride
au corps et cowbell au sommet ; relances de fin de chorus sans toucher la clave ; feel droit ; arc (exposition et retombée plus
légères que le sommet, stop-time) ; mêmes graine, même sortie, 16 comping, 16 batteries, 16 basses distincts sur 16 graines ;
≥ 8 habillages distincts ; arc « orchestre ». Mutations vérifiées, variété, distance au GMD, basse, habillages :
`specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban.md`.

## Limites

- **Un idiome de batteur, pas un style.** Le kit vient de deux fichiers à un batteur chacun (« trop peu ») : composites de
  probabilités par pas, vélocités de pads TD-11 transposées au niveau de service du kit GM. Le style entier (3 fichiers, 276
  mesures, 2 batteurs) est « juste » ; sa moitié ternaire n'est pas représentée.
- **Section rythmique latine non mesurée.** Clave, montuno, tumbao, congas, cloche, bongos, timbales, shaker, maracas, güiro :
  règles de métier. Aucune écoute (D5) : le propriétaire écoute dans le studio ; un rythme écrit de mémoire peut être faux.
- **Tumbao avec le un joué.** Le tumbao strict tait le un (la basse joue sur le « et » du 2 et du 4, puis l'anticipation) ; les
  gabarits jouent aussi le pas 0 pour que la basse arrive sur la basse publiée à chaque changement, y compris sous stop-time.
- **Basse.** WJazzD LATIN ne mesure ni croche ni silence : l'arrivée sur la basse de l'accord est de 100 % ici (57 % [46–68] dans
  l'annotation WJazzD, qui n'est pas une marche idéale). La fenêtre de douze demi-tons donne p1 36, p50 41, p99 46 contre 31, 40,
  50 : l'étendue mesurée (dix-neuf demi-tons) n'est pas atteignable sans octave. Le moteur applique la note de passage à toute
  cible de basse différente (`next_target_differs`), pas seulement à un changement à la barre. Le plan de gabarits est fixé par air
  (49 % des mesures de basse changent d'un chorus au suivant, par les notes de passage seulement).
- **Rythme harmonique.** Toutes les grilles changent à la barre (100 % des mesures à un accord contre 87 % mesurés, changement au
  temps 3 : 0 % contre 15 %) : deux des quatre gabarits (1 et 2) font sonner leur quinte du pas 6 sur quatre pas, par-dessus le pas 8 : un changement
  au temps 3 ne serait pas suivi d'une attaque de basse.
- **Relances.** Le GMD n'a pas de relance afro-cubaine entière (fills isolés d'un batteur) ; seule la queue (pas 13 à 15) en est
  jouée, pour ne pas toucher la clave ; `fills_none_weight` (18) est un réglage.
- **Variante de clave rumba** (troisième coup au pas 11) : non tirable, voir « Manques du schéma ».
- **Mixage** : réglé au LUFS en solo, aucune écoute.

## Manques du schéma et du moteur (rien n'est contourné)

- **`[[fills]]` n'a pas de côté de clave** : une relance prend la main à `start_step` sans savoir si la mesure est « trois » ou
  « deux », donc ne peut pas commencer avant le pas 13 sans risquer le coup du pas 12. Un champ `clave` sur les relances
  autoriserait des relances de mesure entière.
- **Pas de tirage de variante de clave par graine** : les étiquettes de l'arc, qui nomment les patterns de chaque chorus,
  l'emportent sur celles d'une variante de session ; une clave rumba (autre pattern de `claves`) ne peut donc pas être tirée
  par la graine sans conflit de tags. Un tag de session qui compose avec ceux de l'arc, ou un champ `clave` pattern, le
  permettrait.
- **Le mode tiré n'est pas lié à la grille** : une boucle écrite ne sait pas si la tonalité de session est majeure ou mineure.
- **Gabarits de basse et changement au temps 3** : un gabarit ne peut pas dire « attaque au pas du changement si le changement
  tombe dans la mesure » ; il faut un gabarit à attaque au pas 8 (`motifs` + `reference = "chord_root"` le font par motif).
- **Stop-time** : `keep_steps` garde les attaques des pas listés de la mesure, pas l'anticipation de la mesure précédente (pas 14).
- **`PercussionLow` sans niveau propre** (canal 9 partagé) : le niveau d'une voix se règle par sa vélocité.

## Reproduire

```sh
# fichiers du profil (sauf README), labo en lecture seule
python3 specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_generer.py --out profiles/gmd_afrocuban \
    --mix specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_mix.json
# mixage
python3 specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_mix.py --out specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_mix.json
# preuves (depuis ~/Project/bascanada/harmonium, binaire du worktree) : voir specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban.md
cargo test -p harmonium_core --test gmd_afrocuban_profile_tests
```

<!-- q0:début -->
## Corrections Q0 (phase 020, `TUNING_VERSION` 34)

Compte rendu, mesures avant / après et limites : `specs/020-qualite-des-styles/q0-corrections-rapides.md`. Scripts : `specs/020-qualite-des-styles/q0/` (`appliquer.py` : accents et mise en place, `batterie.py` : groove de batterie).

- **Accents du comping** (règle de métier : clave : pas 6, 14 et anticipation accentués, temps plats, doubles croches fantômes). Chaque coup garde la vélocité de 019 et reçoit un écart selon sa place, forme `afrocubain_comp`, amplitude 28, décalage de niveau -8 (le niveau moyen est conservé : la calibration du mixage reste valable) ; appuyés : croche de contretemps (+0,5), attaque anticipée (+0,9), pas 6 (+0,9), pas 14 (+1), pas 2 (+0,3) ; légers : 1re double croche (-0,6), 3e double croche (-0,4) ; coups probabilistes : -0,7 de plus. Plage de vélocité des accords et de la basse : 32 à 118 (`velocity_min`, `velocity_max` d'A6).
- **Accents de la basse** (tumbao : anticipation et pas 6 et 12 accentués) : forme `afrocubain_bass`, amplitude 14, décalage -6 ; appuyés : croche de contretemps (+0,4), attaque anticipée (+1), pas 6 (+0,7), pas 12 (+0,7) ; légers : contretemps 2 et 4 (-0,3), 1re double croche (-0,6), 3e double croche (-0,6) ; coups probabilistes : -0,8 de plus.
- **Groove de batterie** : phrases et patterns de 019 inchangés ; probabilité des coups secondaires 0,5, 5 note(s) fantôme(s) à 0,1, exposant des probabilités 0,5, phrases tenues ×16, part de fins de phrase sans relance ×0,25.
- **Mise en place** : basse sur les décalages de la grosse caisse (A6, par pas), accords sur la moyenne caisse claire / timekeeper ; `jitter` basse 0,02 et accords 0,02 pas (celui de la batterie est déjà mesuré) ; égrenage ancré sur la première corde quand le profil en a un (`anchor = 0`).

<!-- q0:fin -->

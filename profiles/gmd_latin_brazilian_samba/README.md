# Profil `gmd_latin_brazilian_samba` (phase 019, B3)

Accompagnement de samba, feel droit, sous une grille d'accords : basse, comping et batterie ne dépendent que de
l'accord publié (profil `loop`, pas de piste `Melody`, aucune clave). Fait pour training : l'utilisateur fournit sa
grille, le profil joue dessous ; sans grille, quatre boucles de seize mesures tirées par la graine, dans une
tonalité elle aussi tirée par la graine. Tout sonne avec GeneralUser GS (programmes GM, kits de la banque 128) :
aucun Odin.

Le nom est celui de la source qui définit le style (principe VII) : le style `latin/brazilian-samba` du Groove MIDI
Dataset. Il ne dit rien de la samba en général : cinq fichiers d'**un seul batteur** sur kit électronique ; tout ce
qui est latin (surdo, tamborim, agogô, chocalho, pandeiro, basse, comping) est une **règle de métier**, dite telle
dans ce document et dans les commentaires de `profile.toml` et de `percussion.toml`.

**Un 4/4 de 16 pas est deux mesures de 2/4** (la samba est en 2/4) : chaque moitié de la mesure (pas 0 à 7, pas 8 à
15) est une mesure de 2/4 de huit doubles croches. Surdo étouffé au temps 1 (pas 0 et 8), ouvert au temps 2 (pas 4 et
12). Le tempo est celui des fichiers du GMD, noté à la noire de leur mesure 4/4.

## Provenance et droits

| Couche | Source | Licence | Mention (aussi dans `[meta] corpus`) |
|---|---|---|---|
| Batterie du kit : probabilités par pas, deux gestes de caisse claire, suites, micro-timing, tempos | Groove MIDI Dataset v1.0.0, style `latin/brazilian-samba` : 5 fichiers `beat` 4/4 droits, 1 batteur (`drummer5`), 322 mesures, 14,0 min, « juste ». SHA256 de l'archive vérifié avant toute lecture : `651cbc52…1dcc1e` | CC BY 4.0 | Gillick, Roberts, Engel, Eck, Bamman, Learning to Groove with Inverse Sequence Transformations, ICML 2019 ; Groove MIDI Dataset v1.0.0, Google LLC, CC BY 4.0 |
| Relances | GMD `latin/samba`, 17 fichiers `fill` (« juste »), 12 retenus (voir « Batterie ») ; ses 2 fichiers `beat` (« trop peu », 116 BPM) ne servent qu'au tempo | CC BY 4.0 | idem |
| Basse : registre, approches avant un changement, rythme harmonique, longueur de chorus | Weimar Jazz Database v2.1, `rhythmfeel = LATIN` : 27 solos, 23 avec mesures valides, 1 639 mesures, 40,0 min, « assez » | ODbL 1.0 (contenus DbCL 1.0) | Contains information from the Weimar Jazz Database (Jazzomat Research Project, HfM Weimar), which is made available here under the Open Database License (ODbL); individual contents under the Database Contents License (DbCL 1.0). |
| Surdo, tamborim, agogô, chocalho, pandeiro, basse en deux temps, comping, grilles par défaut, arc, habillage, mixage | Règles de métier écrites pour ce profil (idiome traditionnel) | — | Aucun corpus ouvert ne mesure la section rythmique d'une samba |

Aucune note de solo, aucune ligne de basse, aucune mesure d'un enregistrement n'est copiée : seuls des agrégats
(probabilités par pas, comptes, parts, centiles) servent. Les fichiers MIDI du GMD ne sont pas dans le dépôt.
Mesures du labo : `harmonium_lab/references/gmd-latin-brazilian-samba/`, `gmd-latin-samba/`, `wjazzd-latin/`,
`l2-ingestion.md` (§ 4.2, § 5, § 6). **BRID** (*Brazilian Rhythmic Instruments Dataset*, audio, CC BY 4.0, que L2 § 5
désigne pour la samba) **n'est pas ingéré** : il demande une détection d'attaques par instrument (une chaîne de plus
et une fiabilité à mesurer) ; c'est la source qui mesurerait surdo, tamborim et pandeiro.

## Mesure → valeur

Effectifs : GMD `latin/brazilian-samba`, 322 mesures, 5 fichiers, 1 batteur, « juste » (seuil : 3 fichiers et 60
mesures ; « assez » demande 8 fichiers et 3 batteurs). WJazzD `LATIN`, « assez » (≥ 20 solos et ≥ 1 500 mesures),
hors classes de tempo (seule `120–180` est « juste »).

| Paramètre | Valeur | Mesure source (effectif, fiabilité) |
|---|---|---|
| `meter.steps_per_bar`, `default_swing_ratio` | 16 pas, 0,5 | GMD samba : 5 fichiers sur 5 « straight » |
| `default_bpm` | 89 | GMD samba : tempos des 5 fichiers 89, 89, 89, 93, 110 ; p10 89, p50 89, p90 103,2 |
| `min_bpm`, `max_bpm` | 85, 120 | Tempos mesurés 89 (GMD) à 116 (`latin/samba` `beat`, « trop peu ») ± 4 BPM de l'arousal (`tempo_offset`) |
| Plages de tempo de l'habillage | 89–92, 101–104, 113–116 | Toutes dans l'étendue mesurée 89–116. Séparées de plus de 8 BPM (au pire 9). **La plage du milieu n'a aucun fichier** (le GMD saute de 93 à 110) ; la plage haute s'appuie sur un fichier à 110 et sur `latin/samba` (116, « trop peu ») ; la mesure n'a pas l'étendue pour plus de trois plages étroites |
| Longueur du chorus | 16 mesures | WJazzD LATIN : chorus de 8 mesures (31 sur 69), 16 (9), 24 (9), 32 (9) ; 16 est le choix du brief, pas le plus fréquent |
| Rythme harmonique des grilles | un accord par mesure, 10 % des mesures à deux accords (au poids des quatre boucles ; 6 à 25 % selon la boucle) | WJazzD LATIN : 86,9 % des mesures à un accord (1 639 mesures, 23 solos) ; changement au temps 1 dans 84,9 % des cas, au temps 3 dans 15,1 % ; accord de 4 temps (58,7 %), 2 temps (29,8 %), 8 temps (8,4 %) |
| `bass.low`, `high`, `walk_low`, `walk_high` | 35, 46, 33, 50 | WJazzD LATIN, 6 556 temps : hauteur de basse p1 31, p50 40, p99 50, étendue 28–53 ; arrivées sur un changement dans 35..45 (1 015 événements), fenêtre de douze demi-tons la plus dense 36..47 (84,7 %) ; fenêtre de marche 33..50 |
| `bass.passing` | `none_weight` 60, chromatique 22, degré de gamme 19 | WJazzD LATIN : approche du temps 4 avant un changement à la barre (1 205 événements, 22 solos) : aucune 59,9 % [51–66], chromatique 21,6 % [17–26], diatonique 18,5 % [11–29] |
| Surdo ouvert | GM 43 (tom), pas 4 et 12 (temps 2 de chaque 2/4) | GMD samba : `tom_low` (GM 43 dans la table Roland → GM du labo) au pas 4 dans 63 % des mesures, au pas 12 dans 61 %, 17 % au plus ailleurs (pas 14). Vélocité médiane 87 contre 50 pour la caisse claire |
| Surdo étouffé | GM 36 (grosse caisse), pas 0 et 8, vélocité réduite de 26 à 30 | GMD samba : grosse caisse au pas 0 dans 99 % des mesures, au pas 8 dans 92 %. Pas de surdo étouffé dans le kit GM : la grosse caisse douce le remplace |
| Grosse caisse du kit | pas 0, 3, 4, 7, 8, 11, 12, 15 | GMD samba : 99, 92, 92, 83, 92, 82, 86, 74 % ; moins de 8 % ailleurs. Vélocité médiane 31 (TD-11) |
| Charleston au pied | GM 44, pas 2, 6, 10, 14 | GMD samba : 84, 92, 84, 85 % ; moins de 5 % ailleurs. Vélocité médiane 55 |
| Caisse claire (la caixa) | doubles croches continues, sauf aux pas 4 et 12 (où parle le surdo) | GMD samba : 59 à 87 % des mesures à chaque pas, 30 % aux pas 4 et 12. Deux gestes : ≥ 12 coups par mesure (196 mesures, 4 fichiers) ou jeu plus aéré (126 mesures, 5 fichiers) |
| Cross-stick (GM 37) | pas 7, 11, 15 à la densité 2 | GMD samba : 23, 22, 30 % ; au plus 12 % ailleurs |
| Phrases de batterie | 10 suites de 2 et 4 mesures des deux gestes, poids = occurrences (5 à 63) | GMD samba : suites vues ≥ 3 fois sur ≥ 2 fichiers (ou ≥ 6 fois) ; la plus fréquente `[1,1]` 63 fois sur 4 fichiers |
| Relances | 3 : `court` (9 fichiers, < 18 coups), densités 1 et 2 ; `chargé` (3 fichiers), densité 2 | GMD `latin/samba` `fill` : 17 fichiers à 116 BPM, 12 d'une mesure droits retenus ; exclus : 4 fragments de moins de trois temps (pas 0 à 8-10) et 1 fichier swing |
| `fills_none_weight` | 10 (≈ 45 % de fins de phrase sans relance) | valeur de service : le GMD isole les relances, la part de fins de phrase sans relance n'y est pas mesurable |
| Micro-timing de la batterie | grosse caisse −0,008 sur le temps, −0,046 hors temps ; surdo (tom) −0,021 et −0,040 ; caisse claire −0,035 et −0,048 ; charleston au pied −0,027 et −0,025 (pas) | GMD samba, médianes de l'écart à la grille (n = 1 192 / 1 136, 406 / 137, 734 / 2 941, 24 / 1 145). Aucun `jitter` (contrainte de B4) |
| Niveaux de rendu de chaque voix | vélocités de base de `percussion.toml`, formes d'accents du GMD | réglages de service : les vélocités du TD-11 (grosse caisse 31, caisse claire 50) ne se transposent pas au kit GM ; seule la vélocité **relative** du groupe est gardée |
| `mix.tracks` | accords −14,5 dB, basse −17,8 dB, percussion −14,2 dB (les deux pistes), maître +24,3 dB | voir « Mixage » |

Valeurs de métier, aucune mesure : tout ce qui est latin dans la batterie (surdo comme voix, tamborim, agogô,
chocalho, pandeiro), l'orchestration par étiquette, la basse (deux appuis, figure de la grosse caisse, anticipation),
le comping et ses phrases, les grilles par défaut, l'arc, les variantes de l'habillage, le gratté, le mixage.

## Choix de voix (kit GM de GeneralUser GS)

Touches vérifiées dans la banque (`a6-roles-latins-sf2.py`, kits 128/0, 8, 32, 40 : tous les kits tirables ont ces
touches) : surdo **43**, grosse caisse **36**, caixa 38, cross-stick 37, charleston au pied 44, tamborim **76**,
agogô aigu / grave **67 / 68**, chocalho **82**, pandeiro **54**.

- **Surdo = tom 43, pas la grosse caisse seule ni le tom 41.** Le GMD joue le temps 2 sur le tom grave (rôle `tom_low`,
  GM 43 dans la table du labo) et garde la grosse caisse pour le pied (pas 0 et 8, puis les anticipations). La voix
  du surdo est donc sur la piste grave avec la grosse caisse, au pas où le batteur la joue. GM 41 serait plus grave
  mais n'est pas ce que le GMD mesure : non utilisé.
- **Tamborim = wood block aigu 76.** Le kit GM n'a pas de tamborim (membrane aiguë à baguette) ; le wood block est le
  stand-in le plus court et le plus sec. Figures 3-3-2 (pas 0, 3, 6 puis 8, 11, 14) ou 3-2-3 (0, 3, 5 puis 8, 11,
  13) : règle de métier.
- **Agogô 67 / 68** (aigu aux pas 0 et 8, grave aux pas 6 et 14), **chocalho 82** (croches à la densité 0, doubles
  croches ensuite, accents sur les temps), **pandeiro 54** (tambourin, contretemps 2, 6, 10, 14) : règles de métier.

## Ce que le profil joue

- **Grille** : quatre boucles de seize mesures relatives à la tonique (majeure, tonique tirée par la graine : do, ré,
  fa, sol, la, si bémol). `I–vi–ii–V` quatre fois (`Imaj7`, `vim7`, `iim7`, `V7` ; poids 3 ; mesure 15 : `iim7` et
  `V7` à la blanche, `V7` tient la 16) ; `I–IV–V` (`I6`, `IVmaj7`, `V7`, poids 2 ; deux mesures par accord, la
  seconde moitié resserrée, `IV` puis `V7` à la blanche à la mesure 13) ; cycle de quintes diatonique
  `Imaj7, IVmaj7, viim7b5, III7b9, vim7, iim7, V7, Imaj7` deux fois (3 ; `iim7`, `V7` à la blanche mesure 14) ;
  `vi–ii–V–I` (2 ; `vim7`, `iim7` et `V7` à la blanche, `Imaj7` deux mesures, quatre fois). Formes communes écrites
  pour ce profil, aucune œuvre nommée, **aucune réharmonisation** (D4). Training remplace la boucle par la sienne.
- **Basse** (contrebasse 32 par défaut ; l'habillage tire aussi la basse aux doigts 33 et la fretless 35) : quatre
  gabarits à deux appuis et deux motifs. *Marcação* (fondamentale puis quinte à chaque temps, notes courtes),
  figure de la grosse caisse (0, 3, 4, 7, 8, 11, 12, 15 avec notes fantômes), deux tenues, marcação variée
  (fondamentale, fondamentale, quinte, fondamentale). Les gabarits **anticipent le changement d'accord** (`pickup` :
  la basse de l'accord suivant, un ou deux pas avant la barre ou la blanche). Deux motifs sans le quatrième temps
  laissent la place aux approches de `passing`. La quinte est `fifth_if_held` : la quinte tant que l'accord tient,
  la fondamentale à un changement, donc toujours l'arrivée sur la basse publiée. Aucun swing.
- **Comping** (guitare nylon 24 par défaut ; l'habillage tire aussi l'acier 25, le piano 0, le piano Tine 4 ; notes
  guides, `drop_2`, sans fondamentale ou *shell* ; dérive de registre de 2 demi-tons au plus, 5 au maximum) : douze
  rythmes d'une mesure en trois étiquettes (`sparse` : tenues, temps 2, silence ; `medium` : contretemps, trois-trois-
  deux, anticipations ; `dense` : doubles croches de cavaquinho à notes fantômes à probabilité réduite) et dix-sept
  phrases de 2 et 4 mesures, une mise en place `part_end` par étiquette. Les attaques qui finissent sur la barre ou la
  blanche jouent l'accord suivant (`anticipate`).
- **Batterie** (kit tiré : standard 0, room 8, jazz 32 ou balais 40) : 18 patterns (3 densités × 3 étiquettes × 2
  gestes), 90 phrases (10 suites × 3 densités × 3 étiquettes), 3 relances (étiquette `batucada`). Étiquettes de
  l'arc : **`surdo`** (surdo, grosse caisse douce, chocalho ; à la densité 2, les anticipations de la grosse caisse),
  **`ensemble`** (+ tamborim, agogô à partir de la densité 1, charleston au pied et grosse caisse du kit),
  **`batucada`** (+ caisse claire, cross-stick, pandeiro, relances). La densité (0, 1, 2) est celle de l'arc :
  exposition et retombée niveau 0, corps niveau 1, sommet niveau 2.
- **Arc** (cinq chorus par air, nombre connu) : 1 exposition (`surdo`, comping `sparse` ou `medium`, doux) ; 2 l'ensemble
  entre ; 3 corps (la batucada une fois sur deux, un break de la guitare à mi-chorus ou un break guitare + basse
  avant le chorus suivant) ; 4 sommet (batucada pleine, `dense`, plus fort, stop-time sur les deux dernières mesures
  une fois sur deux) ; 5 retombée (`surdo`, `sparse`, doux, stop-time sur le un des deux dernières mesures). Au-delà de
  cinq chorus : l'ensemble, le corps et le sommet en boucle.
- **Habillage par graine** (`session.toml`, neuf groupes indépendants, 31 variantes, 41 472 habillages) : tempo (trois
  plages), tonalité (6), instrument du comping (4), de la basse (3), kit (4), famille de voicings (4), registre (2),
  énergie (3), arc (2 : `arc.toml` ou l'arc nommé « orchestre » : chorus 1 guitare nylon et balais, chorus 3 piano,
  sommet guitare acier et kit de puissance 16, retombée guitare nylon et balais).

## Mixage

Niveaux de piste calés sur le LUFS intégré en solo (`--solo-track`, graines 1 et 2, 48 mesures,
`ffmpeg ebur128=peak=true`) avec un maître à +25 dB, cibles de service accords −26, basse −25, batterie −25,5
(somme en puissance des deux pistes de percussion, un seul niveau : elles partagent le canal 9). Les vélocités de la
grosse caisse et du surdo ont été ramenées de 78 et 92 à 64 et 78 : la crête vraie du kit « room » (graine 8, +0,3
dBFS) venait de la piste grave. Maître ensuite ramené à +24,3 dB : crête vraie ≤ −1 dBFS sur 16 graines.

| Piste | LUFS en solo, maître +24,3 dB (graines 1 et 2) |
|---|---:|
| Accords | −26,6 |
| Basse | −25,6 |
| Percussion grave (grosse caisse, surdo) | −28,8 |
| Percussion aiguë | −30,1 |
| Batterie (somme en puissance) | −26,4 |
| Mix complet, graines 1 et 2 | −22,0 |
| Mix complet, 16 graines (min – max) | −22,9 – −20,0 |
| Crête vraie, 16 graines (max) | −1,3 dBFS |

Les kits ne sonnent pas pareil (le kit « room » est le plus fort, les balais le plus doux) : l'écart de 3 LU entre
graines est celui du kit tiré. Aucun mixage par écoute (D5).

## Contrat et preuves

`harmonium_core/tests/gmd_latin_brazilian_samba_profile_tests.rs` (flux de `MeasureSnapshot`, 11 tests) : grille
par défaut = une des quatre boucles écrites, reprise à l'identique à chaque chorus, tonique de la variante tirée ;
ni clave, ni mélodie, ni swing, seize pas droits ; tempo dans la plage de la variante tirée et de la source à toutes
les intensités ; basse avec attaques aux pas 0 et 8, arrivée sur la basse publiée à chaque changement (hors
stop-time) dans 35..46 ; basse limitée à fondamentale, quinte, anticipation et approche ; comping dans l'accord
publié (celui de la mesure suivante pour une attaque qui finit sur la barre), sans fondamentale hors `drop_2`, dans
la tessiture du registre tiré ; batterie dans le kit, voix latines sur la bonne piste ; surdo aux pas 4 et 12
seulement, orchestration par chorus (exposition : surdo, grosse caisse, chocalho ; ensemble : tamborim, charleston
au pied ; sommet : caisse claire, pandeiro, agogô, relances) ; arc (exposition et retombée plus légères que le sommet,
stop-time) ; même graine même sortie ; habillage (≥ 8 habillages distincts sur 16 graines, 3 grappes de tempo,
≥ 4 tonalités, ≥ 3 comping, ≥ 2 basses, ≥ 2 kits, ≥ 3 voicings, programmes publiés = variantes tirées, orchestration
par chorus). Mutations vérifiées, variété, distance et basse : `specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_samba.md`.

## Limites

- **Un batteur, un kit électronique.** Le GMD samba est le jeu d'un batteur sur un Roland TD-11 : une samba de batteur,
  pas celle d'une bateria. Les patterns de la batterie du kit sont des composites de probabilités par pas, avec coups
  probabilistes (A6) : jamais une mesure d'un enregistrement. Rien de ce qui est latin n'est mesuré (BRID non ingéré).
- **Le tamborim est un wood block, le surdo un tom** : le kit GM n'a ni l'un ni l'autre ; l'étouffé du surdo est une
  grosse caisse douce.
- **Tempo** : l'étendue mesurée (89 à 116 BPM, un fichier à 110 et deux à 116 « trop peu ») ne laisse place qu'à trois
  plages étroites ; celle du milieu n'est mesurée par aucun fichier. Le tempo ne change pas d'un chorus à l'autre ;
  l'arousal décale de ± 4 BPM.
- **Basse** : WJazzD ne dit rien des croches, des anticipations ni des silences : les gabarits sont écrits. `LATIN` est du
  hard bop sur rythme latin (14 solos post-bop, 8 cool, 4 hard bop, 1 bebop, 1951–1997), il ne distingue ni samba ni
  bossa. L'arrivée sur la basse publiée vaut 100 % ici contre 57,1 % [46–68] dans la source (annotation peu cohérente
  avec la grille, pas une règle de jeu). Le registre joué va de 30 à 46 (p1 31, p50 40, p99 46) contre 28–53 (p99 50) :
  aucun gabarit n'utilise l'octave. L'approche du moteur est une croche ou une noire avant une cible, pas l'approche du
  quatrième temps de la source.
- **Un seul plan de basse par air** : les gabarits tirés sont rejoués à chaque chorus ; seules les approches varient
  (20 % de mesures changées d'un chorus au suivant contre 92 % pour la marche de `wjazzd_swing`).
- **La relance coupe l'ensemble** : une relance remplace tout le pattern à partir de son premier pas (`start_step` 0 :
  la mesure entière) ; surdo, tamborim, agogô et chocalho se taisent pendant la caisse claire.
- **Micro-timing** : médianes seulement, sans dispersion (0,13 à 0,20 pas, mesurée ailleurs) ; voix latines sans décalage.
- **Grilles** : une boucle de grille ne dépend pas de la tonalité tirée au-delà de la tonique (le mode n'est qu'une
  étiquette : toutes les boucles sont majeures et les tonalités tirées sont toutes `ionian`). Pas de grille mineure
  (samba-canção en mineur, partido alto) : non écrite, aucune source ne la mesure.
- **Comping** : règles de métier (aucun corpus ouvert) ; les phrases ne sont pas mesurées. Un gratté de guitare est
  appliqué aussi au piano (valeurs de départ, aucune mesure).
- **Habillage** : le tempo moyen n'est pas lié au kit ni au comping (les groupes sont indépendants) ; l'arc « orchestre »
  répète les variantes d'`arc.toml` (une variante de session ne peut pas hériter de l'arc par défaut et ne surcharger que
  les programmes). Le mapping graine → habillage du CLI (`--seed`) n'est pas celui des tests Rust (`seed_from_u64`).
- **Mixage** : réglé au LUFS en solo (graines 1 et 2), aucune écoute ; le kit tiré change le niveau de 3 LU.

## Manques du schéma (rien n'est contourné ; pour A7)

1. **`two_feel` n'existe que sous `walking`** (exclusif de `motifs`, `templates`, `passing`) : la basse à deux appuis est
   écrite en gabarits, et l'arc ne peut pas passer d'un feel à l'autre.
2. **`passing.when`** : `next_target_differs` ou `chord_change_inside_bar` seulement. Aucune règle « avant un changement
   d'accord à la barre » : l'approche mesurée (WJazzD) s'applique à toute cible différente, et le moteur la pose
   seulement sur un pas sans coup de basse (d'où les motifs sans quatrième temps).
3. **Pas d'étiquette d'arc pour la basse** : le plan de motifs de basse est fixé par air (voir Limites).
4. **Pas de voix de surdo ni de tamborim** dans le kit GM : stand-ins (tom 43, wood block 76).
5. **Relance par-dessus un pattern** : une relance remplace la mesure ; il n'y a pas de relance qui laisse jouer les
   voix d'ensemble (surdo, tamborim) sous la caixa.
6. **Mesure de 2/4** : une mesure de 4/4 sur 16 pas porte deux mesures de 2/4 ; les phrases, les breaks et les relances
   comptent en mesures de 4/4 (un chorus de 16 mesures est un chorus de 32 mesures de 2/4).
7. **Arc nommé de session** : pas d'héritage de `[arc]` (l'arc « orchestre » le recopie).
8. **Micro-timing** : aucune dispersion possible sur la batterie sans déplacer les tirages de la basse à notes de passage.

## Reproduire

```sh
# batterie (labo en lecture seule, archive GMD vérifiée par SHA256)
~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_samba_batterie.py \
  --out profiles/gmd_latin_brazilian_samba/percussion.toml --stats /tmp/harmonium-019/b3/samba/mesures-generation.json
# depuis ~/Project/bascanada/harmonium, binaire du worktree :
CLI=<worktree>/target/release/harmonium-cli ; P=<worktree>/profiles/gmd_latin_brazilian_samba
python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py habillages --cli $CLI --profile $P --seeds 16 --bars 48
python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py basse --cli $CLI --profile $P --seeds 16 --bars 48 \
  --reference ~/Project/bascanada/harmonium_lab/references/wjazzd-latin/measures-bass.json
python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_samba_mixage.py --profile $P
~/Project/bascanada/harmonium_lab/.venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_samba_distance.py --profile $P --out distance.json
# variété : harmonium-lab seed-variety --profile $P --seeds 16 --chorus 4 --engine-dir ~/Project/bascanada/harmonium --engine-bin $CLI --label B3 --work-dir … -o …
cargo test -p harmonium_core --test gmd_latin_brazilian_samba_profile_tests
```

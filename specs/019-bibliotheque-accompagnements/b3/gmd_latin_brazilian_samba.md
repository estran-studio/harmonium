# B3 — Profil `gmd_latin_brazilian_samba` (samba)

Date : 2026-11-03. Worktree `b3-latin`, branche `wquintal/b3-latin`, `TUNING_VERSION` 26, **inchangé** : aucune ligne de
moteur, aucun mécanisme ajouté. Un dossier de profil neuf, un fichier de test de contrat, cinq scripts de reproduction,
des preuves JSON. Aucun autre profil n'est touché.

## Ce qui est livré

| Profil | Source | Chorus | Tempo | Instruments | Contenu |
|---|---|---|---|---|---|
| `gmd_latin_brazilian_samba` | GMD `latin/brazilian-samba` : 5 fichiers `beat`, 1 batteur, 322 mesures (« juste ») ; relances `latin/samba` `fill` (« juste ») ; basse : WJazzD `LATIN` (« assez ») | 16 mesures, 5 chorus par air | 89 (85 à 120), droit 0,5 ; habillage 89–92, 101–104, 113–116 | guitare nylon 24, contrebasse 32, kit 128/0 ; tirés par l'habillage : comping 24, 25, 0, 4 ; basse 32, 33, 35 ; kit 0, 8, 32, 40 | 4 boucles de grille, 4 gabarits et 2 motifs de basse, 12 rythmes de comping et 17 phrases, 18 patterns, 90 phrases et 3 relances de batterie, arc, `session.toml` (9 groupes, 31 variantes) |

Fichiers : `profile.toml`, `percussion.toml` (généré), `arc.toml`, `dynamics.toml`, `set.toml`, `arrangement.toml`,
`cells.toml` (banque minimale exigée par le chargeur, jamais jouée), `session.toml`, `README.md` (provenance et droits,
tableau mesure → valeur, grilles, règles de métier, habillage, mixage, limites, manques). Mentions de licence (GMD CC
BY 4.0, WJazzD ODbL §4.3) dans `[meta] corpus` et dans le README. Aucune banque ni preset neuf : GeneralUser GS, kit GM.
Le profil entre dans `harmonium_determinism` et dans le studio sans rien d'autre (`StyleProfile::distributed()`).

Scripts (labo en lecture seule, archive GMD vérifiée par SHA256) :

- `gmd_latin_brazilian_samba_batterie.py` : régénère `percussion.toml` (reproductible : sortie identique octet pour
  octet, vérifiée) et `mesures-generation.json` ;
- `gmd_latin_brazilian_samba_mixage.py` : LUFS en solo par piste, mix complet, crête vraie, `--apply` ;
- `gmd_latin_brazilian_samba_distance.py` : distance de la batterie rendue au GMD ;
- `gmd_latin_brazilian_samba_variete.py` : tableau de `seed-variety --chorus 4` ;
- `mesures_latin.py` (script partagé de B3, non modifié) : habillages et basse.

Preuves JSON : `gmd_latin_brazilian_samba_preuves/` (`habillages.json`, `basse.json`, `variete.json`, `distance.json`,
`mixage.json`, `mesures-generation.json`).

## Choix de conception

- **Nom par source précise** (principe VII) : `gmd_latin_brazilian_samba`. Un seul batteur : le README le dit en tête.
- **Un 4/4 de 16 pas = deux mesures de 2/4** : surdo ouvert aux pas 4 et 12, étouffé aux pas 0 et 8. Aucune clave. Feel
  droit, aucun `[swing]`.
- **Surdo = tom 43 sur la piste grave**, avec la grosse caisse douce pour l'étouffé. Choix justifié par le GMD : le
  batteur joue le temps 2 sur `tom_low` (GM 43) à 63 et 61 % des mesures (pas 4, 12) et la grosse caisse à 99 et 92 %
  (pas 0, 8). GM 41 n'est pas ce que le GMD mesure : non retenu.
- **Tamborim = wood block 76** (le kit GM n'a pas de tamborim), agogô 67 / 68, chocalho 82, pandeiro 54 : règles de
  métier, dites telles. Touches vérifiées dans les kits 0, 8, 16, 32 et 40.
- **Orchestration par chorus par les étiquettes de batterie de l'arc** : `surdo` (surdo, grosse caisse douce,
  chocalho) → `ensemble` (+ tamborim, agogô, charleston au pied, grosse caisse du kit) → `batucada` (+ caisse claire,
  cross-stick, pandeiro, relances). Le niveau de densité suit l'intensité et la densité de l'arc (`dynamics.toml` : 0 à
  l'exposition et à la retombée, 1 au corps, 2 au sommet). Les relances ne concourent que sous `batucada`.
- **Batterie du kit mesurée, avec coups probabilistes** (A6) : un coup est un (rôle, pas) dont la probabilité par pas
  atteint le seuil de la densité ; au-dessus de 90 % le coup est joué toujours, sinon il joue avec sa probabilité
  mesurée. Deux gestes de caisse claire (≥ 12 coups : 196 mesures, 4 fichiers ; moins : 126 mesures, 5 fichiers), dix
  suites vues ≥ 3 fois sur ≥ 2 fichiers (A1).
- **Basse à deux appuis par gabarits** (`two_feel` n'existe que sous `walking`, exclusif de `templates` et de `passing`) :
  marcação, figure de la grosse caisse, tenues, marcação variée, avec `pickup` pour l'anticipation du changement
  d'accord ; deux motifs sans quatrième temps pour les approches. `fifth_if_held` (la quinte tant que l'accord tient,
  la fondamentale à un changement) : un `fifth` simple jouait la quinte à un changement au temps 3 (test de contrat,
  graine 17). La quinte n'a pas d'octave : placée dans la fenêtre 35..46 comme la fondamentale.
- **Registre de basse WJazzD LATIN** : fondamentale et quinte dans 35..46 (arrivées 35..45, fenêtre dense 36..47), approches
  dans 33..50, p1 31 p50 40 p99 50 mesurés.
- **Comping** : guitare nylon (tirable : acier, piano, piano Tine), notes guides / `drop_2` / sans fondamentale / *shell*
  par l'habillage, `register_drift`, pas de `color` (une tension naturelle non publiée sortirait de l'accord publié : le
  contrat « comping dans l'accord publié » ne tiendrait plus).
- **Habillage** (D6) : neuf groupes indépendants (tempo, tonalité, comping, basse, kit, voicings, registre, énergie, arc),
  aucun champ réglé deux fois, pas de `[fixed_tonality]`. Tempo : trois plages dans l'étendue mesurée (89–116), jamais
  hors d'elle ; la plage du milieu n'a aucun fichier (dit dans le README).
- **Mixage** : calé sur le LUFS en solo, vélocités de grosse caisse et de surdo ramenées à 64 et 78, maître +24,3 dB.

## Preuves

Toutes depuis `~/Project/bascanada/harmonium`, binaire du worktree
(`<worktree>/target/release/harmonium-cli`, `CLI` ci-dessous), profil `P = <worktree>/profiles/gmd_latin_brazilian_samba`.

### Habillages distincts (16 graines CLI × 48 mesures)

`python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py habillages --cli $CLI --profile $P --seeds 16 --bars 48`

| Mesure | Valeur |
|---|---|
| **Habillages distincts** (tempo à ± 8 BPM, tonalité, mode, programmes comping / basse / kit, voicings) | **14 sur 16 graines** |
| Grappes de tempo (± 8 BPM) | 3 (tempos 89 à 116) |
| Tonalités | 6 (do, ré, fa, sol, la, si bémol) |
| Triplets de programmes (comping, basse, kit) | 10 |
| Kits tirés (banque 128) | programmes 0, 8, 40 (le jazz 32 est tirable, non tiré par ces 16 graines) |
| Variantes distinctes par groupe | tempo 3, tonalité 6, comping 4, basse 3, kit 3, voicings 4, registre 2, énergie 3, arc 2 |
| Graines à orchestration par chorus (arc « orchestre ») | 2 (graines 1 et 6) |
| Swing | 0,5 partout |

Le test Rust (`seed_from_u64`, mapping différent de `--seed`) lit 16 habillages distincts sur 16 graines et 7 graines à
orchestration par chorus.

### Variété entre graines et entre chorus (`seed-variety --chorus 4`, 16 graines)

```sh
cd ~/Project/bascanada/harmonium_lab && for spec in "livre:$WT/profiles/gmd_latin_brazilian_samba" \
    "avant:/tmp/harmonium-019/b3/samba/avant/gmd_latin_brazilian_samba_sans_session" "v0:$WT/profiles/wjazzd_swing"; do
  .venv/bin/harmonium-lab seed-variety --profile ${spec#*:} --seeds 16 --chorus 4 --engine-dir ~/Project/bascanada/harmonium \
    --engine-bin $CLI --label B3 --work-dir /tmp/harmonium-019/b3/samba/var/work -o /tmp/harmonium-019/b3/samba/var/${spec%%:*}.json; done
python3 specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_samba_variete.py /tmp/harmonium-019/b3/samba/var
```

« Avant » : la même copie sans `session.toml` (ni ligne `[files] session`). V0 : `wjazzd_swing` remesuré avec le même
binaire (profil non touché). Chaque cellule : mesures changées d'un chorus au suivant · paires de chorus identiques ·
chorus distincts par graine (sur 4) · suites distinctes sur 16 graines · distance graduée entre graines.

| Couche | `gmd_latin_brazilian_samba` (livré) | même profil sans `session.toml` (avant) | `wjazzd_swing` (V0, même binaire) |
|---|---|---|---|
| accords | 0 % · 100 % · 1,0 · 4/16 · 51 % | 0 % · 100 % · 1,0 · 4/16 · 64 % | 0 % · 100 % · 1,0 · 1/16 · 0 % |
| comping : rythme | 87 % · 0 % · 4,0 · 16/16 · 50 % | 86 % · 0 % · 4,0 · 16/16 · 45 % | 80 % · 0 % · 4,0 · 16/16 · 68 % |
| comping : voicings | 92 % · 0 % · 4,0 · 16/16 · 95 % | 90 % · 0 % · 4,0 · 16/16 · 88 % | 74 % · 0 % · 4,0 · 16/16 · 46 % |
| basse | 20 % · 6 % · 3,62 · 16/16 · 89 % | 19 % · 4 % · 3,81 · 16/16 · 91 % | 92 % · 0 % · 4,0 · 16/16 · 59 % |
| batterie | 100 % · 17 % · 3,0 · 16/16 · 13 % | 100 % · 17 % · 3,0 · 16/16 · 13 % | 0 % · 100 % · 1,0 · 11/16 · 19 % |
| accompagnement | 100 % · 0 % · 4,0 · 16/16 · 60 % | 100 % · 0 % · 4,0 · 16/16 · 60 % | 99 % · 0 % · 4,0 · 16/16 · 38 % |

Orchestration (programmes distincts sur 16 graines × 4 chorus) : livré comping {0, 4, 24, 25}, basse {32, 33, 35}, kit
{128/0, 128/8, 128/40} ; avant : un programme par piste (24, 32, 128/0) ; V0 : un par piste.
Arc (notes par mesure, trajectoire sur 4 chorus, `numeric.all`) : étendue de la densité 97 % (avant 100 %, V0 1,6 %), cv
intra 44 % ; vélocité moyenne : étendue 18 % (avant 19 %, V0 0,3 %), cv 7 %. Batterie : densité 106 %, vélocité 34 %.

Lecture :

- **Tenus** (seuils V2 de V0) : accords inchangés d'un chorus au suivant (0 %), comping rythme (87 % ≥ 75), voicings (92 % ≥
  65), batterie changée d'un chorus à l'autre (100 % ≥ 30, distance graduée 13 % ≥ 10), 16 séquences distinctes sur 16
  graines pour le comping, la basse et la batterie, arc (densité et vélocité), orchestration (4 / 3 / 3 programmes
  contre 1).
- **Ce que l'habillage change** (livré contre « avant ») : un seul programme par piste devient 4 / 3 / 3 ; la distance
  graduée entre graines de l'accompagnement reste à 60 %, celle des voicings du comping passe de 88 à 95 %, celle des
  accords de 64 à 51 % ; la distance du comping rythme de 45 à 50 %. Les mesures changées d'un chorus au suivant ne
  bougent pas (l'habillage est tiré par graine, pas par chorus, sauf l'arc « orchestre »).
- **Non tenus, limites** : paires de chorus identiques de la batterie 17 % (≥ 5 %) : [INFERENCE, non vérifiée] sur quatre
  chorus, l'exposition et la retombée jouent la même étiquette (`surdo`) à la même densité et peuvent tirer le même pattern ;
  basse changée d'un chorus au suivant 20 % (≥ 80 %) : le plan de gabarits est fixé par air (même limite que B4, A4 ne le
  change pas) ; `intensity` publiée par le moteur : 0 % d'étendue (l'arc multiplie l'intensité lue par `dynamics`, il ne
  change pas le champ publié : même constat que B4).

### Basse contre WJazzD `LATIN` (16 graines × 48 mesures, 767 mesures lues)

`python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py basse --cli $CLI --profile $P --seeds 16 --bars 48 --reference ~/Project/bascanada/harmonium_lab/references/wjazzd-latin/measures-bass.json`

Seul ce que la source mesure, échantillonné au temps (la source ne voit ni croche ni silence de basse).

| Mesure | Profil | WJazzD `LATIN` (23 solos, 1 639 mesures, « assez ») |
|---|---|---|
| Registre de basse p1 / p50 / p99 | 31 / 40 / 46 | 31 / 40 / 50 |
| Étendue | 30–46 | 28–53 |
| Arrivée sur la basse de l'accord à un changement | 100 % (656 changements) | 57,1 % [46–68] : annotation peu cohérente avec la grille |
| Pas de basse active à un changement | 0 | — |
| Distance de la note du temps 4 à l'arrivée, demi-ton | 19,7 % (482 changements à la barre) | approche chromatique 21,6 % [17–26] |
| idem, deux demi-tons | 41,7 % | approche diatonique 18,5 % [11–29] (comparaison non équivalente : une quinte à deux demi-tons de l'arrivée en cycle de quintes y compte) |
| Changement d'accord au temps 3 | 8,7 % | 15,1 % |
| Mesures à un seul accord | 92,6 % | 86,9 % [79–93] |
| Tempo (nominal) | 89 à 116 | `avgtempo` p10 104, p50 152,7, p90 227,5 (hard bop sur rythme latin : non comparable) |

La part de croches, les silences et la figure des anticipations ne sont pas comparables (non mesurés par la source).

### Distance au GMD de la batterie du kit (`gmd_latin_brazilian_samba_distance.py`)

`~/Project/bascanada/harmonium_lab/.venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_samba_distance.py --profile $P --seeds 16 --bars 48 --out distance.json`

Distance de Bray-Curtis des fréquences d'attaque par pas (0 = identique, 1 = disjoint), même méthode que B8 et B4 + B7,
référence = les 322 mesures `beat` 4/4 droites du GMD samba. **Voix latines exclues** du rendu (tamborim 76, agogô 67 /
68, chocalho 82, pandeiro 54 : le GMD n'en a aucune) ; ne comptent que 36, 37, 38, 42, 43, 44, 46, 47, 48, 50, 51, 53.
16 graines × 48 mesures, 752 mesures rendues.

| Mesure | Livré (avec arc et habillage) | Livré sans `arc.toml` ni `session.toml` | Copie non calibrée (batterie de `wjazzd_swing`, sans arc) | **Sommet seul** (chorus 4, batucada pleine, 256 mesures) |
|---|---:|---:|---:|---:|
| un coup quelconque par pas | 0,272 | 0,284 | 0,472 | **0,077** |
| grosse caisse | 0,189 | 0,262 | 1,000 | **0,106** |
| caisse claire (et cross-stick) | 0,668 | 0,554 | 1,000 | **0,094** |
| charleston (au pied) | 0,288 | 0,369 | 0,980 | **0,154** |
| tom grave (surdo) | 0,275 | 0,204 | 1,000 | **0,187** |
| **moyenne (quatre premières)** | **0,354** | 0,367 | 0,863 | **0,108** |

En score 0 à 100 (100 − 100 × distance) : livré 64,6 ; sans arc 63,3 ; non calibré 13,7 ; sommet 89,2. Le profil livré est
**plus proche du corpus que sa version sans arc** (pas de perte, règle de P2) : le gain sur la grosse caisse et le
charleston compense la caisse claire (+ 11 points de distance), absente par construction de trois chorus sur cinq
(`surdo`, `ensemble`) ; les voix du kit jouent à plein au sommet (0,108). La mesure ne dit rien des voix latines, ni de la
basse et des accords (le GMD n'en a pas).

### Mixage (LUFS intégrés, `ffmpeg ebur128=peak=true`)

`python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_samba_mixage.py --profile $P --peaks 16`

Solo par piste : graines 1 et 2, 48 mesures ; mix complet et crête vraie : 16 graines. Niveaux réglés par mesure (`--apply`
à maître +25 dB, cibles accords −26, basse −25, batterie −25,5 en somme de puissance), puis maître à +24,3 dB.

| Piste | LUFS en solo (maître +24,3 dB) | Cible de service |
|---|---:|---:|
| Accords | −26,6 | −26 |
| Basse | −25,6 | −25 |
| Percussion grave (grosse caisse, surdo) | −28,8 | — |
| Percussion aiguë | −30,1 | — |
| Batterie (somme en puissance) | −26,4 | −25,5 |
| Mix complet, graines 1 et 2 | −22,0 | ≈ −21 / −22 |
| Mix complet, 16 graines | −22,9 à −20,0 | |
| Crête vraie, 16 graines (max) | **−1,3 dBFS** | ≤ −1 |

Au maître +25 dB, avec les vélocités d'origine de la grosse caisse et du surdo (78 et 92), le kit « room » de la graine 8
crêtait à +0,3 dBFS ; abaissées à 64 et 78 puis niveaux recalés (solos −25,9 / −24,9 / −25,7, mix −21,2 pour les graines 1
et 2), la crête vraie maximale était de −0,6 dBFS ; le maître est ramené de 0,7 dB. Solos de 0,6 à 0,9 LU sous leur cible.

### Contrat (`harmonium_core/tests/gmd_latin_brazilian_samba_profile_tests.rs`)

`cargo test -p harmonium_core --test gmd_latin_brazilian_samba_profile_tests` : **11 tests, 11 réussis** (flux de
`MeasureSnapshot` ; graines 1, 2, 3, 17 et 1 à 16 ; profil distribué `StyleProfile::named`).

| Contrat | Test |
|---|---|
| grille par défaut = une des 4 boucles écrites (≥ 3 tirées sur 16 graines), reprise identique d'un chorus à l'autre, tonique = variante tirée, ≥ 4 tonalités | `la_grille_par_defaut_est_une_des_boucles_ecrites_reprise_a_l_identique_a_chaque_chorus` |
| ni clave, ni mélodie, ni swing (0,5), 16 pas, aucune attaque hors grille, bornes de tempo 85–89–120 | `le_profil_n_a_ni_clave_ni_melodie_ni_swing_et_reste_en_seize_pas_droits` |
| tempo nominal dans la plage de la variante, tempo à ± 4 BPM de celle-ci à 5 intensités, 3 plages tirées | `le_tempo_reste_dans_la_plage_de_la_variante_tiree_et_de_la_source` |
| basse : attaques aux pas 0 et 8, arrivée sur la basse publiée à chaque changement (hors stop-time), dans 35..=46, hauteurs dans 28..=53 | `la_basse_joue_deux_appuis_et_arrive_sur_la_basse_publiee_a_chaque_changement` |
| basse : fondamentale, quinte, anticipation (basse de l'accord qui commence à la fin de la note) ou approche à 1–2 demi-tons | `la_basse_ne_joue_que_fondamentale_quinte_anticipation_ou_approche` |
| comping dans l'accord publié (celui de la mesure suivante pour une attaque qui finit sur la barre), sans fondamentale hors `drop_2`, *shell* à 2 sons, tessiture du registre tiré | `le_comping_ne_joue_que_des_notes_de_l_accord_publie_dans_la_famille_et_le_registre_tires` |
| batterie dans le kit, grosse caisse et surdo sur la piste grave, toutes les voix latines sur la piste aiguë | `la_batterie_reste_dans_le_kit_et_les_voix_latines_sont_sur_les_bonnes_pistes` |
| surdo aux pas 4 et 12 seulement (chorus 1, 2, 5), exposition et retombée à surdo / grosse caisse / chocalho, ensemble sans caisse claire, sommet avec caisse claire, pandeiro, agogô et relances | `le_surdo_est_ouvert_aux_temps_deux_et_la_batterie_s_etoffe_chorus_apres_chorus` |
| arc : exposition et retombée plus légères que le sommet (exposition ≤ 70 %), stop-time de la retombée, tempo constant | `l_arc_creuse_l_ecart_entre_l_exposition_le_sommet_et_la_retombee_avec_stop_time` |
| même graine, même sortie ; ≥ 14 comping, ≥ 14 batteries, ≥ 12 basses distincts sur 16 graines | `la_meme_graine_rejoue_pareil_et_les_graines_changent_comping_batterie_et_basse` |
| **≥ 8 habillages distincts sur 16 graines** (lus dans `MeasureSnapshot.session` et `instruments`), 3 grappes de tempo, ≥ 4 tonalités, ≥ 3 comping, ≥ 2 basses, ≥ 2 kits, ≥ 3 voicings, programmes publiés = variantes tirées, orchestration par chorus | `des_graines_differentes_donnent_au_moins_huit_habillages_distincts_sur_seize` |

**Mutations vérifiées** (copie du test lisant le profil d'un dossier de `/tmp`, profil muté ; 18 mutations, chacune fait
échouer au moins un test, puis annulées) :

| Mutation | Test qui échoue |
|---|---|
| `default_swing_ratio` 0,62 | feel droit |
| retombée sans `breaks` (stop-time) | arc |
| `fifth_if_held` → `fifth` | arrivée de la basse |
| surdo au pas 11 au lieu de 12 | surdo et orchestration |
| plage de tempo « vif » 117–120 | tempo |
| `[clave]` ajoutée | ni clave ni mélodie |
| un accord de la boucle 1 (`vim7` → degré 8) | grille |
| `session.toml` retiré | tempo, grille, habillage, comping |
| caisse claire dans un pattern `surdo` | surdo et orchestration |
| deux voicings au lieu de quatre | habillage |
| trois tonalités au lieu de six | habillage, grille |
| comping trop grave (`low` 36, `center` 44) | comping |
| fenêtre de basse 30–41 | arrivée de la basse |
| attaque de comping anticipée à la blanche | comping |
| chorus 3 de « orchestre » : piano Tine au lieu du piano | habillage |
| surdo sur la piste aiguë | batterie / pistes |
| exposition en `batucada` | surdo et orchestration |

### Liens d'écoute (à ouvrir par l'orchestrateur ; non ouverts ici)

Graines de `mesures_latin.py habillages` dont les habillages diffèrent le plus (8 groupes sur 9, seule la basse est
commune) :

- `harmonium://open?profile=gmd_latin_brazilian_samba&seed=7` : 91 BPM, sol, guitare nylon, contrebasse, kit aux balais,
  voicings *shell*, comping une octave plus bas, énergie douce, arc `arc.toml`.
- `harmonium://open?profile=gmd_latin_brazilian_samba&seed=6` : 114 BPM, do, piano Tine, contrebasse, kit standard,
  notes guides, registre normal, énergie neutre, arc « orchestre » (chorus 1 guitare nylon et balais, chorus 3 piano,
  chorus 4 guitare acier et kit de puissance).

Ce qui change de l'une à l'autre : tempo (91 → 114), tonalité, instrument du comping, kit, famille de voicings, registre,
énergie, arc et orchestration par chorus. Dans les deux : surdo et chocalho seuls au chorus 1, l'ensemble au chorus 2,
la caisse claire et les relances à partir du 3, stop-time à la retombée.

## Manques du schéma / moteur (rien n'a été contourné)

1. **`two_feel` n'existe que sous `walking`** : exclusif de `motifs`, `templates` et `passing`. La basse à deux appuis est
   écrite en gabarits et motifs ; l'arc ne peut pas piloter le feel de basse.
2. **`passing.when`** : seulement `next_target_differs` et `chord_change_inside_bar`. Pas de règle « avant un changement
   d'accord à la barre » : l'approche mesurée s'applique à toute cible différente et ne se pose que sur un pas sans
   coup de basse (motifs sans quatrième temps).
3. **Pas d'étiquette d'arc pour la basse** : le plan de gabarits est fixé par air (seuil V2 6 non tenu : 20 % de mesures
   changées d'un chorus au suivant).
4. **Pas de surdo ni de tamborim dans le kit GM** : stand-ins (tom 43, wood block 76) ; pas de surdo étouffé.
5. **Relance par-dessus un pattern** : une relance remplace la mesure entière (`start_step` 0) ; l'ensemble se tait sous
   la caisse claire.
6. **Mesure de 2/4** : 16 pas portent deux mesures de 2/4 ; phrases, breaks et relances comptent en mesures de 4/4.
7. **Arc nommé de session** : pas d'héritage de `[arc]` ; l'arc « orchestre » recopie les variantes d'`arc.toml` pour y
   ajouter des programmes.
8. **Micro-timing** : aucune dispersion possible sur la batterie sans déplacer les tirages de la basse à notes de passage
   (contrainte de B4, toujours non documentée par le moteur).
9. **Une boucle de grille ne dépend pas de la tonalité tirée** : le mode n'est qu'une étiquette (toutes les tonalités
   tirées sont majeures, les boucles aussi).
10. **`intensity` publiée** : l'arc n'apparaît pas dans le champ `intensity` du snapshot (seuil V2 12 : 0 %).

## Limites

- Un batteur, un kit électronique, 5 fichiers (« juste ») : une samba de batteur. Rien de latin n'est mesuré : BRID (audio,
  CC BY 4.0) n'est pas ingéré. Les 2 fichiers `beat` de `latin/samba` (« trop peu ») ne servent qu'au tempo.
- Tempo : 89 à 116 BPM mesurés (un fichier à 110, deux à 116 « trop peu ») ; la plage du milieu (101–104) n'a aucun fichier.
- Basse : WJazzD `LATIN` est du hard bop sur rythme latin (ne distingue ni samba ni bossa) et n'a aucune croche ; arrivée
  sur la basse publiée 100 % contre 57,1 % ; registre joué 30–46 contre 28–53 (aucun gabarit n'utilise l'octave).
- Comping, grilles par défaut, arc, habillage, mixage : règles de métier, non validés à l'écoute (D5).
- Distance au GMD : mesure symbolique de la batterie du kit seule (voix latines exclues) ; `loop_drums` et l'audio global
  de `make style-distance` ne s'y appliquent pas (le GMD n'a pas d'audio).
- `fills_none_weight` (10) est un réglage de service : la part de fins de phrase sans relance n'est pas mesurable.
- Mixage : réglé au LUFS en solo (graines 1 et 2) ; le kit tiré change le niveau du mix de 3 LU ; aucune écoute.
- Mapping graine → habillage : celui du CLI (`--seed`) diffère de celui des tests (`seed_from_u64`) ; les liens
  d'écoute sont ceux du CLI.

## Vérifications laissées à l'agent principal

`cargo test -p harmonium_core --test gmd_latin_brazilian_samba_profile_tests` passe (11 / 11) et `rustfmt +nightly --edition 2024
--check` sur ce fichier est propre. Non lancés ici : les contrats génériques qui parcourent `StyleProfile::distributed()`
(`profile_files_tests`, `style_profile_tests`, ornements, hétérophonie), les dumps de `harmonium_determinism` (profil neuf :
graines × émotions à ajouter), clippy, `cargo test --workspace`.

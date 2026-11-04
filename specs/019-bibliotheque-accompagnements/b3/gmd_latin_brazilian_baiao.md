# B3 — Profil `gmd_latin_brazilian_baiao` (baião brésilien)

Date : 2026-11-03. Branche `wquintal/b3-latin`, `TUNING_VERSION` 26 **inchangé**. Données seulement : aucune ligne de moteur,
aucun mécanisme ajouté ; un dossier de profil neuf, un test de contrat, six scripts. Autres profils non touchés.

## Ce qui est livré

| Élément | Contenu |
|---|---|
| `profiles/gmd_latin_brazilian_baiao/` | `profile.toml`, `percussion.toml` (GÉNÉRÉ), `arc.toml`, `dynamics.toml`, `set.toml`, `arrangement.toml`, `cells.toml` (banque minimale jamais jouée), `session.toml` (`[files] session`), `README.md` |
| `harmonium_core/tests/gmd_latin_brazilian_baiao_profile_tests.rs` | 11 tests de contrat sur le flux de `MeasureSnapshot` |
| `specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_generer.py` | écrit le profil (réutilise `b8/gmd_batterie.py` et `b8/emit.py`) |
| `…_mix.py`, `…_calibrage_kit.py`, `…_distance.py`, `…_variete.py`, `…_approches.py` | mixage (LUFS), niveau des voix latines, distance au GMD, résumé de `seed-variety`, approches de basse |

Source : GMD `latin/brazilian-baiao` (3 fichiers `beat`, 200 mesures, 2 batteurs, **« juste »** ; 29 fichiers `fill`, 1 batteur) ;
WJazzD `LATIN` (23 solos valides, 1 639 mesures, « assez ») pour le registre et les approches de basse seulement.
**Limite écrite : la source est « juste » à deux batteurs ; le profil est livré à ce titre, il ne représente pas le genre.**

| Profil | Chorus | Tempo | Instruments (habillage) | Contenu |
|---|---|---|---|---|
| `gmd_latin_brazilian_baiao` | 16 mesures, 5 chorus par air | 95 (95 à 110, la source), droit 0,5 | comping accordéon 21 / guitare nylon 24 / piano 0 ; basse 32 / 33 ; kit 128/0, 8, 32, 40 | 4 grilles, 6 motifs de basse, 9 rythmes de comping et 9 phrases, 9 patterns, 18 phrases, 1 relance, arc, 28 variantes de session |

## Choix de conception

- **Nom par source précise** (principe VII), pas d'alias. Mentions de licence dans `[meta] corpus` et le README (GMD CC BY 4.0,
  WJazzD ODbL §4.3).
- **Mesuré / métier.** Mesuré (GMD) : batterie du kit (grosse caisse [0, 3, 8, 11], pédale [2, 6, 10, 14], ride, caisse claire),
  tempo, feel, relances, micro-timing. Mesuré (WJazzD `LATIN`) : registre de basse (p1 31, p50 40, p99 50 ; 28..53) et poids des
  approches (60 / 22 / 19). **Métier, dit tel dans `profile.toml`, `percussion.toml` et le README** : zabumba écrite à toutes
  les densités, triangle, agogô, motifs de basse, comping, vamps, arc, habillage. Le GMD n'a aucune percussion latine.
- **Trois groupes de batterie** : `ride` (57 mesures, 2 batteurs), `hat` (21, 1 batteur) et `caixa` (67 mesures sans timekeeper,
  la caisse claire en doubles croches, 2 batteurs mais **82 % d'un seul fichier**, au-delà du seuil de 60 % de B8 : gardée hors seuil
  parce que c'est le tiers du corpus ; dit dans `percussion.toml` et le README). Un premier essai sans `caixa` laissait la caisse
  claire à une distance de 0,54 du GMD (arc compris, 0,45 sans arc) ; avec, 0,34 (arc compris), 0,09 sans arc.
- **Seuils de B8 relâchés** (3 fichiers) : groupes à 2 fichiers (au lieu de 3), phrases à 2 occurrences sur 1 fichier. Conséquence
  mesurée : aucune suite de groupes ne change de groupe, les 18 phrases tiennent un groove.
- **Grilles** : quatre vamps de seize mesures (I7 ; I7/bVII ; I7/IV ; I7 bVII I7 IV), relatifs à la tonique, aucune œuvre,
  aucune réharmonisation (D4).
- **Basse** : six motifs `chord_root` (fondamentale pas 0 et 8, quinte pas 6 et 14), arrivée sur la basse publiée ; `passing`
  60 / 22 / 19. Fenêtre de la fondamentale 33..44 : meilleure des trois fenêtres essayées (31..42, 33..44, 35..46) pour la
  médiane des temps (34, 39, 43 contre 40).
- **Habillage (D6)** : neuf groupes indépendants, 28 variantes, 13 824 habillages. Tempo : trois plages 95..98, 101..104, 107..110
  **dans l'étendue réellement mesurée (95..110)**, sans extension ; elles ne sont pas trois régimes mesurés. Pas de `[fixed_tonality]`.
- **Arc** : exposition à la charleston, deux chorus de corps (le second, une fois sur deux, à la `caixa`), sommet au ride ou à
  la `caixa` avec stop-time une fois sur deux, retombée à la charleston avec stop-time. Arc nommé « orchestre » : balais à
  l'exposition, accordéon au sommet.

## Preuves

Toutes depuis `~/Project/bascanada/harmonium`, binaire du worktree (`<wt>/target/release/harmonium-cli`), sans son, sans studio.

### Habillages distincts (16 graines)

```sh
python3 <wt>/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py habillages --cli <cli> --profile <wt>/profiles/gmd_latin_brazilian_baiao --seeds 16 --bars 48
```

| Mesure | Valeur |
|---|---|
| habillages distincts (tempo à ± 8 BPM, tonalité, programmes, voicing) | **16 / 16** |
| grappes de tempo (± 8 BPM) | 2 (étendue de la source : 15 BPM) |
| tonalités / triplets de programmes / kits | 6 / 11 / 4 (128/0, 8, 32, 40) |
| variantes distinctes par groupe | tempo 3, tonalité 6, comping 3, basse 2, kit 4, voicing 4, registre 2, énergie 2, arc 2 |
| tempos tirés (graines 1 à 16) | 104, 104, 103, 97, 98, 108, 109, 98, 95, 104, 96, 107, 110, 101, 95, 103 |
| swing | 0,5 pour toutes |

Le test de contrat lit la même chose dans `MeasureSnapshot.session` et `instruments` (≥ 8 distincts, ≥ 4 tonalités, 3
programmes de comping, 2 basses, ≥ 2 kits, ≥ 3 familles de voicings).

### Variété (`seed-variety --chorus 4`, 16 graines)

```sh
harmonium-lab seed-variety --profile <dossier> --seeds 16 --chorus 4 --engine-dir ~/Project/bascanada/harmonium \
  --engine-bin <cli> --label B3 --work-dir /tmp/harmonium-019/b3/baiao/var/<nom> -o <nom>.json
python3 <wt>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_variete.py <livré>.json <sans_session>.json <wjazzd_swing>.json
```

« Livré » : le profil ; « sans habillage » : la même copie sans `session.toml` ; V0 : `wjazzd_swing` remesuré avec le même binaire.

| Couche | Mesures changées d'un chorus au suivant (livré / sans / V0) | Paires de chorus identiques | Chorus distincts / graine (sur 4) | Suites distinctes sur 16 graines | Distance graduée entre graines |
|---|---|---|---|---|---|
| accords | 0 % / 0 % / 0 % | 100 % / 100 % / 100 % | 1,0 / 1,0 / 1,0 | 4 / 4 / 1 | 30 % / 42 % / 0 % |
| comping : rythme | 84 % / 82 % / 80 % | 0 / 0 / 0 % | 4,0 / 4,0 / 4,0 | 16 / 16 / 16 | 15 % / 15 % / 68 % |
| comping : voicings | 82 % / 91 % / 74 % | 0 / 0 / 0 % | 4,0 / 4,0 / 4,0 | 16 / 16 / 16 | **98 %** / 88 % / 46 % |
| basse | 43 % / 44 % / 92 % | 0 / 0 / 0 % | 4,0 / 4,0 / 4,0 | 16 / 16 / 16 | 86 % / 89 % / 59 % |
| batterie | 81 % / 88 % / 0 % | 5 % / 17 % / 100 % | 3,69 / 3,0 / 1,0 | 16 / 16 / 11 | 26 % / 22 % / 19 % |
| accompagnement | 96 % / 99 % / 99 % | 0 / 0 / 0 % | 4,0 / 4,0 / 4,0 | 16 / 16 / 16 | 51 % / 51 % / 38 % |

| Arc et orchestration | Livré | Sans habillage | V0 |
|---|---|---|---|
| densité (notes / mesure) par chorus 1 à 4 ; cv intra | 36,9 / 49,8 / 49,7 / 35,0 ; 16 % | 36,6 / 52,0 / 51,3 / 35,6 ; 18 % | 19,5 / 19,6 / 19,3 / 19,6 ; 2 % |
| vélocité moyenne par chorus ; cv intra | 72,1 / 76,2 / 75,5 / 67,0 ; 5 % | 69,6 / 74,9 / 75,1 / 66,4 ; 5 % | 69,9 / 69,9 / 70,1 / 69,9 ; 0 % |
| programmes vus par la sonde (comping / basse / kit) | [21, 24] / [33] / 4 kits | [21] / [33] / [128/0] | [0] / [32] / [128/32] |

Lecture : l'habillage ajoute 3 kits, un comping de plus et un tempo aux « programmes vus » ; la distance graduée des voicings
entre graines passe de 88 à 98 % et les paires de chorus de batterie identiques de 17 à 5 %. **Non tenu, limite du moteur** :
la basse change de 43 % seulement d'un chorus à l'autre (plan de motifs fixé par air, voir « Manques »). La sonde de V0 ne liste
pas le piano (programme 0) parmi les comping vus [INFERENCE : elle ignore le programme 0] alors que le tirage le donne à 8
graines sur 16 (tableau des habillages) : l'habillage se lit dans `session`, pas dans cette colonne.

### Basse contre WJazzD `LATIN`

```sh
python3 <wt>/specs/019-bibliotheque-accompagnements/b3/mesures_latin.py basse --cli <cli> --profile <wt>/profiles/gmd_latin_brazilian_baiao \
  --seeds 16 --bars 48 --reference ~/Project/bascanada/harmonium_lab/references/wjazzd-latin/measures-bass.json
python3 <wt>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_approches.py --cli <cli> --profile <wt>/profiles/gmd_latin_brazilian_baiao --seeds 16 --bars 80
```

Seul ce que la source **mesure** est comparé (échantillonné au temps ; croches et silences non observables) :

| Mesure | Profil | WJazzD `LATIN` |
|---|---|---|
| mesures lues, temps de basse actifs | 765, 1 878 | 1 639 mesures, 6 556 temps |
| hauteur de basse p1 / p50 / p99 | 33 / 39 / 44 | 31 / 40 / 50 |
| étendue | 33..44 (aux temps), 33..51 avec les quintes et les notes de passage (16 graines, 80 mesures) | 28..53 |
| arrivée sur la basse publiée à un changement | 100 % (166 changements) | 57 % |
| changement au temps 3 ; mesures à un seul accord | 0 % ; 100 % | 15 % ; 87 % |
| changements à la barre précédés d'une approche | 35 % (240 changements) | 40 % (22 chromatique + 19 diatonique ; 1 205 événements) |
| notes d'approche : chromatique / de degré | 64 % / 36 % (104 notes) | 22 / 19, soit 54 % / 46 % |
| tempo | 95 à 110 | médiane 152,7 (p10 104, p90 227,5) : hard bop, non comparable |

Lecture : le registre est trop étroit par le haut (les temps échantillonnés ne comptent pas la quinte en contretemps) ; l'arrivée
à 100 % est un choix du motif (il arrive toujours sur la basse publiée), la source donne 57 % ; la grille n'a ni changement au
temps 3 ni deux accords par mesure, d'où 0 % et 100 % ; les approches sont du même ordre (35 % contre 40 %). Ces écarts sont
dans le README du profil.

### Distance au GMD (batterie du kit ; voix latines exclues)

```sh
~/Project/bascanada/harmonium_lab/.venv/bin/python <wt>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_distance.py
```

Modèle de `b8/distance_gmd.py` : fréquence d'attaque par pas (16 pas) de la grosse caisse, de la caisse claire (et du cross-stick),
du charleston, du ride et d'un coup quelconque, distance de Bray-Curtis (0 = identique), référence = les 200 mesures `beat` droites
du style ; 16 graines x 48 mesures, densité 1. **Triangle (80, 81) et agogô (67, 68) exclus** : le GMD n'en a pas. Copie non
calibrée : même profil avec le `percussion.toml` de `wjazzd_swing`, sans arc ni habillage.

| Rôle | Profil livré | Sans arc ni habillage | Copie non calibrée |
|---|---:|---:|---:|
| un coup quelconque | 0,035 | 0,016 | 0,465 |
| grosse caisse | 0,058 | 0,044 | 1,000 |
| caisse claire | 0,337 | 0,094 | 1,000 |
| charleston | 0,199 | 0,142 | 0,910 |
| ride (hors moyenne) | 0,101 | 0,171 | 0,535 |
| **moyenne des quatre premiers** | **0,157** | 0,074 | 0,844 |

Lecture : la calibration rapproche la batterie du GMD de 0,69 (0,844 à 0,157). L'arc, de métier, l'éloigne de la mixture du GMD
(0,157 contre 0,074 sans arc) : il tire `hat` à l'exposition et à la retombée, `caixa` seulement aux chorus 2 à 4. La caisse
claire reste le point faible (0,34) : elle est rendue à 0,08 à 0,53 par pas (le plus souvent 0,28) contre 0,38 à 0,81 mesuré, parce que les groupes `ride`
et `hat`, joués à l'exposition, au corps et à la retombée, n'ont que 0 à 4 coups de caisse claire.

### Mixage (LUFS intégrés en solo, `ffmpeg ebur128`, graines 1 et 2, 48 mesures)

```sh
python3 <wt>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_mix.py --profile <wt>/profiles/gmd_latin_brazilian_baiao
python3 <wt>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_calibrage_kit.py
```

| Piste | Graine 1 | Graine 2 | Moyenne | Cible |
|---|---:|---:|---:|---:|
| Chords | -26,4 | -25,5 | -25,9 | -26 |
| Bass | -25,2 | -25,0 | -25,1 | -25 |
| PercussionLow | -28,1 | -25,6 | -26,9 | |
| PercussionHigh | -32,0 | -29,5 | -30,8 | |
| batterie (somme en puissance) | -26,6 | -24,1 | -25,4 | -25,5 |
| mix complet | -21,6 | -20,3 | -21,0 | -21 à -22 |
| crête vraie (dBFS) | -4,6 | -3,6 | -4,1 | ≤ -1 |

Niveaux réglés par mesure : accords -12,4 dB, basse -12,4, batterie -9,5, `master_db` 20. Triangle muet (GM 80) : -59,9 LUFS à
vélocité 50, -49,9 à 100 (kit standard, 4 coups par mesure) : trop faible pour être réglé, joué à 127.

### Contrat

`harmonium_core/tests/gmd_latin_brazilian_baiao_profile_tests.rs`, onze tests (graines 1, 2, 3, 17 et 1 à 16) :

| Contrat | Test |
|---|---|
| la grille par défaut est un des 4 vamps écrits, tonique et mode (mixolydien) de l'habillage, reprise identique d'un chorus à l'autre, ≥ 3 grilles et ≥ 4 tonalités sur 16 graines | `la_grille_par_defaut_est_un_vamp_modal_…` |
| la basse arrive sur la basse publiée à chaque changement (hors stop-time), fondamentale dans 33..44 | `la_basse_arrive_sur_la_basse_publiee_…` |
| fondamentale sur le un et le trois, quinte au pas 6 (≥ 90 % des mesures), sinon une approche à 1-2 demi-tons de la note suivante | `la_basse_joue_la_fondamentale_…` |
| le comping ne joue que des notes de l'accord publié (celui de la mesure suivante pour une attaque anticipée), dans la tessiture du registre tiré (50..80 ou 38..68) | `le_comping_ne_joue_que_…` |
| kit : grosse caisse seule sur `PercussionLow` aux pas {0, 3, 6, 8, 11, 14}, zabumba complète dans ≥ 95 % des mesures, triangle muet sur les temps et ouvert sur les contretemps, agogô sur la piste haute, toute mesure sans triangle est une relance | `la_batterie_joue_la_zabumba_…` |
| feel droit en 16 pas ; tempo dans 95..110 et dans la plage de la variante tirée, à cinq intensités ; trois plages vues | `le_feel_est_droit_en_seize_pas_…` |
| exposition et retombée plus légères que le sommet ; stop-time de la retombée | `l_arc_creuse_l_ecart_…` |
| la `caixa` ne joue qu'aux chorus 2 à 4 | `la_caixa_ne_joue_que_…` |
| même graine, même sortie ; ≥ 14 comping, ≥ 14 batteries, ≥ 12 basses distincts sur 16 graines | `la_graine_change_le_comping_…` |
| habillage : 9 groupes dans l'ordre, ≥ 8 habillages distincts sur 16 graines, programmes publiés = ceux des variantes | `l_habillage_par_graine_donne_…` |
| arc « orchestre » : balais à l'exposition, accordéon au sommet, programmes de la session rendus aux autres chorus | `l_arc_orchestre_change_…` |

Mutations vérifiées (profil ou test modifié, `cargo test -p harmonium_core --test gmd_latin_brazilian_baiao_profile_tests`, puis annulée) :

| # | Mutation | Test qui échoue |
|---|---|---|
| M1 | `default_swing_ratio = 0.62` | `le_feel_est_droit_en_seize_pas_…` |
| M2 | stop-time de la retombée retiré (`breaks` de `[[fall]]`) | `l_arc_creuse_l_ecart_…` |
| M3 | quinte du pas 6 remplacée par une quarte (`semitones = 5`) | `la_basse_joue_la_fondamentale_…` |
| M4 | grosse caisse du pas 3 déplacée au pas 4 | `la_batterie_joue_la_zabumba_…` |
| M5 | variante de tempo « allant » à 120..125 (`max_bpm` 130) | `le_feel_est_droit_en_seize_pas_…` (tempo hors de la source) |
| M6 | groupe `kit` retiré de `session.toml` | `l_habillage_par_graine_donne_…` et `l_arc_orchestre_change_…` |
| M7 | triangle muet déclaré sur `PercussionLow` | `la_batterie_joue_la_zabumba_…` |
| M8 | étiquette `caixa` retirée de `arc.toml` et de l'arc « orchestre » | `la_caixa_ne_joue_que_…` |
| M9 | fenêtre de la fondamentale de basse 31..42 au lieu de 33..44 | `la_basse_arrive_sur_la_basse_publiee_…` |
| M10 | arc « orchestre » : sommet à la guitare nylon (24) au lieu de l'accordéon (21) | `l_arc_orchestre_change_…` et `l_habillage_par_graine_donne_…` |
| M11 | comping : centre 76, fenêtre 50..92 | `le_comping_ne_joue_que_…` (tessiture) |
| M12 | test : attaque anticipée refusée (l'accord suivant n'est plus accepté) | `le_comping_ne_joue_que_…` (accord publié) |

Une première mutation de M8 (étiquette retirée de `arc.toml` seul) ne faisait rien échouer : la moitié des graines tire l'arc « orchestre »,
qui nomme encore la `caixa` ; le test compte sur 16 graines, la mutation complète les retire des deux. Le contrat « même graine, même
sortie » n'a pas de mutation de profil possible (seul un moteur non déterministe le ferait échouer) ; il compare deux rejeux sérialisés.

## Ce qui ne s'exprime pas dans le schéma ou le moteur (rien n'est contourné)

1. **Plan de motifs de basse fixé par air** : la basse ne change d'un chorus à l'autre que par les notes de passage (43 % des
   mesures, seuil V2 ≥ 80 %) ; l'arc n'a pas d'étiquette de basse (`bass_feel` exige `walking`). Même constat que B4 et B7.
2. **`passing` ne se restreint pas au 4ᵉ temps avant un changement à la barre** : les poids WJazzD (60 / 22 / 19) valent pour ce
   temps ; le moteur les applique à toute note de motif qui change de hauteur (35 % des changements à la barre précédés d'une
   approche, plus d'approches dans la mesure).
3. **Une boucle de grille ne dépend pas de la tonalité tirée** : la variante `tonality` change la tonique et l'étiquette du mode,
   pas les accords (un vamp dorien i7 / IV ne peut pas être tiré au lieu du mixolydien I7 / bVII).
4. **Tempo de l'habillage et émotions** : la plage d'une variante est un tempo de base ; `tempo_offset` (arousal) s'y ajoute. La courbe est
   limitée à ± 4 BPM, non écrêtée (le contrat générique `time_plan_playback_tests` exige qu'une commande d'émotion déplace le tempo) : un arousal extrême sort donc de 4 BPM au plus de la plage de la variante.
5. **Niveau par kit** : un seul `level_db` par piste ; les quatre kits de l'habillage (0, 8, 32, 40) n'ont pas le même niveau
   (2,5 dB mesurés entre deux graines), réglé sur le kit standard seulement.
6. **Micro-timing** : médianes par voix bornées à ±0,2 pas, sans dispersion (un `jitter` de batterie déplace les tirages de
   la basse à notes de passage, constat de B4).
7. **Triangle** : le triangle muet du kit GM (80) est presque inaudible dans GeneralUser GS (-49,9 LUFS à 100) ; aucun moyen
   de le relever au-delà de la vélocité 127 (pas de gain par voix hors vélocité).
8. **Zabumba à deux peaux** : le kit n'a qu'une grosse caisse ; la « leve » à la main n'est pas écrite.
9. **Sonde `seed-variety`** : la colonne « programmes vus » ne montre pas le piano (programme 0) [INFERENCE] ; l'habillage se
   vérifie dans `MeasureSnapshot.session` et `instruments`.

## Limites

- **Deux batteurs** (« juste ») : un fichier de 104 mesures à 110 BPM d'un seul batteur, 55 des 67 mesures de la `caixa` ; la
  figure du ride (`quarters_plus` : trois coups par temps aux pas 0, 1, 2) est lue telle quelle, 17 % des coups du corpus sont à plus
  d'un quart de pas de la grille (triolets rapportés au 16 pas possibles, non vérifié).
- **Phrases** : aucune suite de groupes ne change de groupe (seuils relâchés) ; relance : une mesure moyenne de 29 fichiers d'un batteur.
- **Tempo** : 95..110 BPM seulement ; aucune extension arbitraire.
- **Basse** : arrivée sur la basse publiée à 100 % (57 % mesuré), croches et silences non mesurables, pas de changement au temps 3.
- **Mixage** : réglé sur le kit standard, graines 1 et 2 ; tout est jugé par des chiffres (D5), rien à l'oreille.

## Liens d'écoute (à ouvrir par l'orchestrateur ; non ouverts ici)

| Lien | Habillage tiré | Ce qui change |
|---|---|---|
| `harmonium://open?profile=gmd_latin_brazilian_baiao&seed=11` | tempo 96, la mixolydien, piano, basse aux doigts, kit salle (8), voicing ouvert, registre grave, énergie souple, arc orchestre | tempo bas (96), piano en voicings ouverts une octave plus bas, basse aux doigts, kit de salle ; l'arc « orchestre » met les balais à l'exposition (chorus 1) et l'accordéon à la place du piano au sommet (chorus 4), puis rend piano et salle |
| `harmonium://open?profile=gmd_latin_brazilian_baiao&seed=13` | tempo 110, si bémol mixolydien, accordéon, contrebasse, kit jazz (32), voicing coquille, registre normal, énergie franche, arc continu | tempo le plus haut (la borne de la source), accordéon en coquilles (tierce et septième) au registre normal, contrebasse, kit jazz aux baguettes, énergie 1,1, mêmes programmes sur les cinq chorus |

Les neuf groupes diffèrent entre les deux graines (maximum des paires de 16 graines, dont une orchestre visiblement). Aucune des deux n'a été ouverte.

## Reproduire

```sh
~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_generer.py --out profiles/gmd_latin_brazilian_baiao
cargo test -p harmonium_core --test gmd_latin_brazilian_baiao_profile_tests
# preuves : voir « Preuves » ci-dessus (mesures_latin.py habillages / basse, …_approches.py, …_distance.py, …_variete.py, …_mix.py)
```

# A4 + A5 — Basse plus riche et voicings plus variés (phase 019, vague 3)

**Statut** : livrée sur la branche `wquintal/a45-basse-voicings` (base `main` à `0fe8b52`),
`TUNING_VERSION` 26 inchangé. Mécanismes **optionnels en données** : aucun des 16 profils distribués ne
les active, leurs dumps et leurs rendus sont ceux de `main`.

Contrats : fixture `tests/fixtures/profiles/grille_basse_voicings/` (copie de `grille_arc_chorus`),
tests `harmonium_core/tests/bass_voicings_tests.rs` (19 tests), tests unitaires de `voicing.rs` et de
la dérive de `accomp.rs`, documentation `profiles/README.md` (« Marche de basse », « Voicings et
comping »).

## A4 — Basse

Six options dans `[accompaniment.bass.walking]`, chacune absente par défaut ; une option absente
n'ajoute **aucun tirage** au flux de la ligne (les poids nuls des genres de passage n'en changent pas
le total, l'arrivée et les croches tirent dans des flux dérivés de la graine de la mesure) :

| Option | Mécanisme (`walking.rs`) |
|---|---|
| `approach.side` | `from_weight` (côté de l'arrivée précédente : règle historique), `other_weight`, `above_weight`, `below_weight` ; la fenêtre qui n'offre pas le côté tiré laisse l'autre |
| `passing.chromatic_weight` | demi-ton de la note précédente, hors gamme du mode et hors accord (genre de note tiré avec les notes d'accord et les pas de gamme) |
| `passing.repeat_weight` | note précédente rejouée |
| `arrival` | tierce, quinte ou septième de l'accord à l'arrivée ; basse publiée pour un accord slash, une pédale ou un accord sans le degré (**D4**). Tirage indépendant (graine de la mesure + rang de la pulsation) : la mesure précédente retrouve l'arrivée de la suivante avec `next_seed` (= graine d'accompagnement mêlée à `bar_in_tour + 1`) et mène son approche à cette note |
| `eighths` | après une pulsation dont la note suivante tombe à la pulsation suivante : croche de passage (approche chromatique ou diatonique de la note suivante, vélocité `velocity`) ou note fantôme (même hauteur, `ghost_velocity`) ; la note de la pulsation est coupée à la demi-pulsation ; aucune note de pulsation ne change (test) ; pas au deux temps ni après une approche |
| `two_feel` | à deux temps, la deuxième attaque d'une étendue d'accord qui n'est pas une approche est la quinte de l'accord la plus proche (`fifth_weight`) ou une note de passage (`passing_weight`) |

Gabarits non swing (fondamentale-octave, fondamentale-quinte en croches, motifs syncopés) :
**`bass_template.rs` les exprime déjà** (cycle de degrés et octaves, rythme à pas libres) ; rien n'est
ajouté au moteur, un test de contrat les exerce sur la fixture (`les_gabarits_expriment_…`).

### Calibrage sur WJazzD (`make backing-validate`)

Commande équivalente (labo non modifié) : `python -m harmonium_lab.backing.validate` avec le CLI de la
branche, `avant` = `grille_arc_chorus`, `apres` = `grille_basse_voicings`, **sans arc ni croches**
(le validateur du labo ne lit que des mesures à une note par temps : les croches en retirent les
mesures du comparatif ; elles n'ont de toute façon aucune mesure de corpus), 88 mesures par graine.
Calibration sur les graines 1 à 6, **contrôle sur les graines 11 à 26 jamais vues** ; poids obtenus
par quelques itérations par groupe (côté d'approche, chromatisme, répétitions, approche `none`).

| Critère (WJazzD swing, `all`) | Référence | Avant | Après (graines 11–26) |
|---|---|---|---|
| Distance agrégée | 0 | 0,224 | **0,133** (blues 0,217 → 0,133) |
| Passage : accord / gamme / chromatique | 0,48 / 0,25 / 0,27 | 0,48 / 0,38 / 0,14 | 0,48 / 0,24 / 0,28 |
| Part de notes répétées | 0,196 | 0,050 | 0,201 |
| Arrivée sur la basse au changement | 0,41 (`reliable` 0,63) | 1,00 | 0,64 |
| Arrivée : tierce / quinte / septième | 0,10 / 0,11 / 0,10 | 0 / 0 / 0 | 0,12 / 0,11 / 0,13 |
| Approche : none / chromatique / diatonique | 0,59 / 0,21 / 0,19 | 0,55 / 0,22 / 0,23 | 0,58 / 0,18 / 0,24 |
| Part d'approches au-dessus de la cible | 0,526 | 0,365 | 0,456 |
| Part d'approches du côté de l'arrivée précédente | 0,661 | 0,769 | 0,610 |

Par famille (graines 11–26) : approche 0,125 → 0,060 ; arrivée 0,678 → 0,314 ; passage 0,148 → 0,013 ;
intervalles 0,147 → 0,099 ; rythme harmonique 0,251 (propriété de la grille) ; **registre 0,056 → 0,095
(coût mesuré** : le chromatisme et les notes répétées resserrent le registre de marche). Sur les graines de
calibration (1 à 6) : 0,225 → 0,144. Sur la fixture complète (arc, croches, deux temps ; 8 graines ×
88 mesures) : 0,229 → 0,160 ; les mesures du deux temps et celles qui portent une croche sortent du
comparatif (le validateur exige une note par temps).

**Limite de l'arrivée.** Le moteur ne joue que des notes de l'accord à l'arrivée (D4) : les 27 % d'arrivées
« autres » de WJazzD (`bass_pitch` qui ne colle pas à la grille, voir le README de `wjazzd-swing`) ne se
reproduisent pas ; la part de la basse (0,64) suit le groupe `reliable` (0,63), pas `all` (0,41).

### Croches : règles de métier, sans source (L2 § 6 point 8)

WJazzD ne note qu'une hauteur par temps : **aucune mesure de croche de basse**. Poids de la fixture
(84 aucune, 3 chromatique, 3 diatonique, 10 fantôme par pulsation éligible), vélocités 58 (passage) et 34
(fantôme, contre 78 à la marche). Mesuré sur la fixture complète (8 graines) : 97 notes hors pulsation
sur 1 225 notes de basse (7,9 %), 72 fantômes et 25 croches de passage. Les profils B2, B4, B8 les
règlent à leur tradition.

## A5 — Voicings

Cinq familles ajoutées à `chords.voicing` (extensibles : un `match` dans `voicing::candidates`, une
forme par qualité ; **A2 tire la famille par graine sans autre changement**, la tension de couleur se tire
à l'intérieur de chaque famille). Une famille sans forme pour une qualité retombe sur `rootless`.

| `voicing` | Forme |
|---|---|
| `drop3` | position serrée à quatre voix, troisième voix depuis l'aigu une octave plus bas |
| `fourths` | piles de trois quartes justes dans les sons de l'accord, des tensions publiées et des tensions naturelles permises, qui gardent la tierce ou la septième |
| `upper_structure` | triade majeure ou mineure bâtie sur un degré autre que la fondamentale, tous renversements, dans les sons de l'accord et de la tension de couleur, qui garde la tierce ou la septième |
| `shell` | tierce et septième, deux dispositions |
| `open` (*spread*) | drop 2 et 4 ; nommée `open` pour ne pas la confondre avec `chords.spread` (étalement d'une attaque) |

**Tension de couleur** (`chords.color`) : tirage pondéré par accord joué (graine de la mesure + créneau
d'accord) entre aucune tension, la tension publiée (priorité historique) et les tensions naturelles 9,
11, 13 que le type permet. Règle écrite dans `voicing.rs` (`natural_allowed`, tableau de métier par
type) : permise par le type ; jamais doublée ni altérée par la qualité publiée (b9 ou #9 pour 9,
#11 pour 11, b13 pour 13) ; jamais **un demi-ton au-dessus d'un son de l'accord** (note à éviter).
Sans poids éligible, tension historique. **Dérive de registre** (`chords.register_drift`) : marche
aléatoire d'un pas par mesure (`step`), bornée à `±max` autour de `center`, mêmes conduite de voix
(`theory::voice_leading`) et fenêtre `low..high` ; l'état vit dans `ChordPlayer`, rejoué depuis le début
de la passe de partie comme la conduite de voix (rejeu après seek).

### Preuve chiffrée (16 graines, 12 mesures, deux attaques par mesure)

Notes d'accord = classes de hauteur de l'accord publié (tensions publiées comprises) ; tensions = les autres
notes du voicing (toutes permises par la règle ci-dessus : test de contrat). « Distincts » = voicings
différents par (mesure, attaque) sur 16 graines. Grille « blues » = F7 B♭7 Gm7 C7 ; grille « qualités » =
7b9, 7#9, 7b13, 7#11, m7b5, maj7, m7, 13. « Couleur » : `color` et `register_drift` de la fixture.

| Famille | Grille | Couleur | Notes d'accord | Tensions | Distincts / accord | Voix | Ambitus |
|---|---|---|---|---|---|---|---|
| rootless (existant) | blues | non | 100,0 % | 0,0 % | 1,00 | 4,00 | 12,0 |
| rootless | blues | oui | 88,7 % | 11,3 % | 2,58 | 4,00 | 11,1 |
| guide_tones (existant) | blues | oui | 73,6 % | 26,4 % | 4,38 | 2,72 | 8,3 |
| drop2 (existant) | blues | oui | 82,0 % | 18,0 % | 4,88 | 4,00 | 16,1 |
| **drop3** | blues | non / oui | 100,0 / 82,0 % | 0,0 / 18,0 % | 1,00 / 4,33 | 4,00 | 18,2 / 18,6 |
| **fourths** | blues | non / oui | 52,1 / 52,1 % | 47,9 % | 1,00 / 1,00 | 4,00 | 15,0 |
| **upper_structure** | blues | non / oui | 100,0 / 78,6 % | 0,0 / 21,4 % | 1,00 / 6,67 | 3,92 / 3,23 | 11,6 / 8,9 |
| **shell** | blues | non / oui | 100,0 / 100,0 % | 0,0 % | 1,00 / 2,00 | 2,00 | 6,1 / 6,0 |
| **open** | blues | non / oui | 100,0 / 82,0 % | 0,0 / 18,0 % | 1,00 / 3,00 | 4,00 | 21,5 / 21,4 |
| rootless | qualités | non / oui | 100,0 / 93,4 % | 0,0 / 6,6 % | 1,00 / 3,71 | 4,00 | 10,4 / 11,0 |
| drop3 | qualités | non / oui | 100,0 / 90,2 % | 0,0 / 9,8 % | 1,00 / 4,38 | 4,00 | 18,7 / 18,8 |
| fourths | qualités | non / oui | 85,4 / 82,0 % | 14,6 / 18,0 % | 1,00 / 2,21 | 4,00 | 11,9 / 12,6 |
| upper_structure | qualités | non / oui | 100,0 / 90,7 % | 0,0 / 9,3 % | 1,00 / 4,83 | 3,50 / 3,35 | 9,3 |
| open | qualités | non / oui | 100,0 / 90,2 % | 0,0 / 9,8 % | 1,00 / 3,46 | 4,00 | 21,9 / 21,2 |

Sans `color` ni `register_drift`, un accord joue **un seul voicing sur 16 graines** (comping fixe) ; avec
eux, 2 à 7 selon la famille et la grille (rootless 1 → 2,6 / 3,7, drop 3 1 → 4,3 / 4,4, structure
supérieure 1 → 6,7 / 4,8). La famille `fourths` reste à 1,00 sur le blues : une septième de dominante ou
mineure n'a qu'une pile de quartes permise, la dérive ne la déplace pas d'une octave (voir Limites).

## Vérifications

- Dumps `harmonium_determinism` : **98 / 98 identiques à `main`** (binaire de `main` sur 0fe8b52 contre la
  branche) ; rendus `harmonium-cli --export --bars 16 --record-wav` (lancé depuis le checkout principal,
  16 profils × graines 1 et 7) : **32 / 32 identiques** ; natif = wasm 98 / 98
  (`harmonium_determinism/scripts/run.sh`).
- Suites : `cargo test --workspace --no-fail-fast` 878 réussis, 1 échec : `seek_to_bar_64_under_50ms`
  sous charge de la machine (charge moyenne 75 : 155 ms sur une exécution), **réussi isolément** ;
  `cargo test -p harmonium --features odin` 147, `-p harmonium_bevy` 11, `cargo check -p harmonium_studio`,
  `cargo check --target wasm32-unknown-unknown` (moteur, avec et sans `odin` ; Bevy sans défaut), `cargo fmt
  --check`, clippy : aucun avertissement nouveau dans les fichiers touchés.
- **Seek** : 15 exécutions de `seek_to_bar_64_under_50ms`, minimum **26,1 ms** (médiane 29,7 ms, machine
  chargée ; `main` : 25,9–26,2 ms au compte rendu d'A3). Aucun des profils de ce seek n'active une option :
  le chemin ne change pas. Avec les options de la fixture, une mesure se rejoue depuis son début de passe
  de partie, comme le comping existant (la dérive ajoute un tirage de graine par mesure rejouée).
- Sans option, `grille_basse_voicings` redonne le flux de `grille_arc_chorus` octet pour octet
  (`sans_option_le_flux_est_celui_de_la_fixture_de_l_arc`, 6 graines).
- Test de contrat de chaque option : retirer une option change la ligne
  (`chaque_option_de_basse_change_la_ligne_et_elle_seule`) ; arrivée sur chaque changement d'accord et au
  premier temps dans l'accord publié, dans `low..high` ; l'approche tombe à un demi-ton de l'arrivée
  tirée de la mesure suivante (mutation vérifiée : une graine suivante fausse fait échouer le test) ;
  accord slash : basse publiée ; croches : pulsations inchangées, hors pulsation à la demi-pulsation,
  fantômes à la hauteur de la note, vélocités de la fixture ; deux temps : fondamentale puis quinte ;
  gabarits ; voicings de chaque famille dans les notes de l'accord publié et les tensions permises ;
  aucune 9 sur b9 ou #9, 13 sur b13, 11 sur #11 ; rejeu identique après un seek aux mesures 2, 14, 31 et
  40 ; erreurs de schéma nommant le champ.
- `chord_player` : la dérive reste dans `±max`, par pas d'au plus `step`, atteint les deux bornes (test
  unitaire sur 4 000 graines).

## Limites et manques moteur

- **Croches sans source** : poids de métier, jugés sur leur effet statistique seulement (D5). Pas de
  croche au deux temps (la note suivante ne tombe pas à la pulsation suivante) ; elles demandent un
  nombre pair de pas par pulsation (refus au chargement sinon) ; 3/4 et 6/8 sont du ressort d'A6.
- **`fourths`** : une seule pile de quartes par qualité dominante ou mineure, donc peu de variété par
  graine (1,00 sur le blues) ; la pile à cinq sons (trois quartes plus une tierce majeure) n'est pas
  proposée (les tensions 13 d'un mineur ne sont pas permises).
- **m7b5, dim7, augmenté, sus2, altéré** : aucune tension naturelle ajoutée (la neuvième et la onzième d'un
  demi-diminué entrent en conflit avec sa quinte bémol et sa tierce) ; seules les tensions publiées
  colorent.
- **Registre de la marche** : 0,056 → 0,095 (chromatisme et répétitions resserrent la ligne) ; non corrigé,
  on aurait réduit le gain sur le passage.
- **Dérive** : repart de zéro à chaque passe de partie (conséquence du rejeu depuis le début de la passe).
- **Arrivée « autre »** de WJazzD (27 %) non reproductible sous D4.
- `chords.color` et `chords.register_drift` n'ont d'effet que sur les voicings de comping (pas
  `close_triad` ni `fixed_triad`) : acceptés sans erreur pour ne pas bloquer le tirage de famille d'A2.

## Changements hors du fichier de la tâche (génériques, minimaux)

- `generator.rs` : `WalkingBar.next_seed` (graine de la mesure suivante) pour l'arrivée tirée.
- `accomp.rs` : `ChordPlayer` porte la dérive de registre et tire la couleur ; `voicing::candidates` prend
  la couleur (`Color::Priority` = comportement historique, identique à `main`).
- `accomp_profile.rs` : champs optionnels, validation (`accompaniment.bass.walking.*`,
  `accompaniment.chords.color|register_drift`), variantes de `ChordVoicing`.

## Pour les profils B

B2 (balais, deux temps) : `two_feel`, `arrival`, croches fantômes ; B4 / B8 (funk, rock) : `templates`
(déjà là) plus `eighths` pour les fantômes ; tous : `color` et `register_drift` dans leur banque de voicings,
`shell`, `upper_structure`, `fourths` pour les styles modaux.

Valeurs de départ tirées des mesures de L2 (`references/wjazzd-{swing,twobeat,ballad,latin,funk}/measures-bass.json`,
groupe `all`, mesures valides : 23 224, 752, 294, 1 639, 928 ; peu de solos hors swing : fiabilité faible) :

| Style | Mesures | Notes répétées | Arrivée : basse / tierce / quinte / septième | Passage : accord / gamme / chromatique | Approche : none / chromatique / diatonique | Approche du côté de l'arrivée précédente |
|---|---|---|---|---|---|---|
| swing | 23 224 | 0,20 | 0,41 / 0,10 / 0,11 / 0,10 | 0,48 / 0,23 / 0,29 | 0,59 / 0,21 / 0,19 | 0,66 |
| twobeat | 752 | 0,37 | 0,25 / 0,15 / 0,09 / 0,12 | 0,40 / 0,19 / 0,41 | 0,69 / 0,17 / 0,13 | 0,79 |
| ballad | 294 | 0,24 | 0,55 / 0,11 / 0,08 / 0,09 | 0,64 / 0,19 / 0,17 | 0,72 / 0,14 / 0,13 | 0,75 |
| latin | 1 639 | 0,38 | 0,57 / 0,07 / 0,06 / 0,10 | 0,63 / 0,16 / 0,21 | 0,58 / 0,25 / 0,17 | 0,75 |
| funk | 928 | 0,28 | 0,45 / 0,03 / 0,11 / 0,13 | 0,39 / 0,22 / 0,39 | 0,72 / 0,14 / 0,14 | 0,66 |

Les poids de la fixture (swing) donnent la méthode : approche et passage se règlent par poids, la part
des arrivées sur la basse suit le groupe `reliable` (la source ne colle pas à la grille).

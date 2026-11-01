# P1 — Procédural par règles, en option

Date : 2026-11-01. Branche : `wquintal/p1-procedural-regles`, partie de `main` à `f9de703`
(E1, E3, E5, E6 fusionnées, `TUNING_VERSION` 24). **`TUNING_VERSION` inchangé, goldens inchangés,
notes des profils distribués identiques** (preuves plus bas). Aucun profil distribué n'active
ces options : l'intégration à un profil attend l'écoute du propriétaire (plan 018). Aucun nom de
genre ni de jeu dans le code ou le schéma, aucun `match` sur un nom de style : tout est nommé par
le rôle musical.

## Résumé

| Question | Réponse |
|---|---|
| Qu'est-ce qui est ajouté ? | Quatre options de données : **boucles d'accords générées par contraintes** (`harmony.parts.<partie>.generated`), **pédale à alternatives** (manque moteur 1 de P2), **basse par gabarits** (`accompaniment.bass.templates`), **portée de partie et de tour d'un pattern de batterie** (manque moteur 2 de P2). |
| Les profils actuels changent-ils ? | Non. Dumps `harmonium_determinism` (92) identiques à `main`, rendus `--export --record-wav` de `reel_quebecois`, `isaac_cellar`, `nt_desert` identiques (WAV et MIDI). |
| Variété par graine | `nt_desert` : accords 0 → 66 % des mesures, basse 25 → 67 % ; `isaac_burning` : signatures d'accords distinctes par partie 1,38 → 4,12, basse par mesure 2,44 → 4,47 ; batterie inchangée. |
| Distance au corpus (tolérance 1 point) | Accords : neutre ou meilleure partout. Basse par gabarits : neutre sur `isaac_burning`, **nettement pire** sur `nt_desert` (−6,3 à −1,1). Batterie par portée : neutre sur `nt_desert`, **pire** sur `isaac_burning` (−1,1 symbolique, −2,4 sur la couche). |
| Recommandation | `nt_desert` : **intégrer les accords** ; ni la basse ni la batterie. `isaac_burning` : **intégrer accords + basse** ; pas la batterie. Détail et réserves : section 6. |

## 1. Données ajoutées (toutes optionnelles)

Schéma complet, exemples et sémantique : `profiles/README.md`, section « Procédural par règles ».

| Champ | Contenu |
|---|---|
| `harmony.parts.<p>.generated[]` | `weight`, `bars`, `count`, `qualities` (texte F1), `motion` (`chromatic`, `fifths`, `scale_steps` avec `direction`, ou `intervals` pondérés), `start`, `pedal`, `key_fit` (`free`, `within`, `outside`). Dans une partie `chord_loop`, à côté de `loops` (l'une des deux au moins) |
| `harmony.parts.<p>` `kind = "pedal"` | `weight`, `alternatives[]` (`weight`, `degree` ou `semitones`, `neighbor_semitones`) |
| `accompaniment.bass.templates[]` | `weight`, `part`, `bars` (cycle, 1..16), `rhythm[]` (`step`, `duration`, `velocity`), `cycle[]` (`degree` root / fifth / fifth_if_held, `octave`), `ramp` (`stride`, `motion` semitone / scale_step), `pickup` (`lead`, `duration`, `velocity`, `degree`, `when` chord_change / bar_end). Exclusif de `hits` et de `walking` |
| `percussion.patterns[]` | `part` (nom de `form.parts`) et `tour` (1-based) optionnels |

Changements d'API Rust (pas de contrat de snapshot) : `HarmonyPart::Pedal` gagne `weight` et
`alternatives`, `HarmonyPart::ChordLoop` gagne `generated` ; `PercussionPattern` gagne `part` et
`tour` ; `PercussionPatternPlan::pattern` gagne le paramètre `tour` ; `BassVoice` gagne
`templates` ; `StyleProfile::reachable_modes` (extrait de `validate`) est publique ;
`PercussionTableError` gagne `ZeroTour`, `ScopedCadence`, `MissingUnscoped`.

## 2. Algorithmes

### Boucle générée (`models/loop_generation.rs`)

1. **Tirage de l'alternative.** Le pool de la partie est `loops` puis `generated`, un seul tirage
   pondéré avec le flux historique (`mix(mix(seed, 0x4841_524D), partie)`), sans tirage pour une
   alternative unique. Sans `generated`, le flux est celui d'avant octet pour octet.
2. **Génération** (une fois par partie et par air, flux séparé `mix(mix(mix(seed, "GENL"), partie),
   alternative)`). Table de faisabilité à rebours : `rows[i]` = ensemble des fondamentales d'où
   une suite de `count − i` accords existe (fondamentale à qualité permise par `key_fit`, et un
   successeur faisable selon `motion`). Tirage en avant sans rejet : qualité uniforme parmi les
   permises de la fondamentale, puis fondamentale suivante pondérée parmi les successeurs
   faisables. Aucune boucle partielle, aucun essai refusé.
3. **Répétition.** La boucle de `bars` mesures est répétée jusqu'à la longueur de la partie
   (`bars` divise la partie) : la forme reste lisible, la boucle se rejoue à l'identique aux
   reprises et aux tours. Durée : `créneaux / count` par accord, les premiers accords reçoivent le
   reste.
4. **Pédale.** `pedal = n` pose la basse `n` sur tous les accords : l'accord slash apparaît
   quand la fondamentale s'en écarte. `quality = "m"` sur `C#` au-dessus d'une pédale `C` est
   publié `C#m/C`.
5. **Mode.** `scale_steps` et `key_fit` dépendent du mode actif ; la validation vérifie la
   faisabilité dans **chaque mode atteignable** (`reachable_modes`, avec `fixed_tonality` un seul).
   Une bascule de couleur régénère la boucle avec la même graine dans le nouveau mode, comme
   les degrés d'une boucle écrite se recalculent.

### Pédale à alternatives (`models/chord_loop.rs`)

La pédale écrite directement dans la partie est la première alternative (poids `weight`, défaut 1).
Même tirage que les boucles (`draw_alternative`), une fois par partie et par air ; sans
`alternatives`, aucun tirage.

### Gabarits de basse (`compose/bass_template.rs`)

Les gabarits entrent dans le **même pool pondéré** que les motifs (`BassVoice::sources` : motifs
puis gabarits ; `BassMotifPlan` tire par partie avec les règles existantes : portée `part`
prioritaire, mesures restantes, copies, secondes fins). Sans gabarit, le pool est la liste des
motifs : même flux. La réalisation ne tire rien : l'attaque `n` de la mesure `b` du cycle prend
`cycle[(b × attaques + n) mod longueur]`, placée dans `low..=high` sur l'accord du **créneau** de
son pas (`harmony.steps_per_chord`), plus `b × stride` (demi-tons ou pas de gamme).
La relance joue la basse de l'accord suivant `lead` pas avant le changement, retire les attaques
qu'elle recouvre et raccourcit celle qui la déborde ; en fin de parcours (aucun accord suivant)
il n'y en a pas. `first_attack` donne la première attaque à `bass.passing` (approche de la
mesure précédente), comme pour un motif.

### Portée de la batterie (`compose/percussion.rs`)

`PercussionPatternPlan` choisit par partie, **tour** et densité : candidats de la densité dont la
portée convient, puis **seuls les plus spécifiques** (deux portées > une > aucune) concourent par
poids. Les tours `1..=max_tour` (plus grand `tour` nommé) ont leur emplacement, les autres (intro,
tours au-delà) utilisent l'emplacement 0, où les patterns à `tour` ne concourent jamais. Sans
portée dans la table : même tirage qu'avant (graine `mix(mix(seed, partie), densité)` inchangée pour
l'emplacement 0). Chaque densité requiert un pattern sans portée (repli) ; les cadences n'ont pas
de portée.

## 3. Décisions (sans question au propriétaire)

- **Qualité indépendante de la fondamentale.** `qualities` est une liste tirée uniformément ; seul
  `key_fit` lie qualité et racine (qualités permises sur une racine = celles dont les notes
  tiennent dans la gamme). Une grille « chaque degré a sa qualité » reste une boucle écrite.
- **`loops` devient facultatif** quand `generated` est non vide : le message
  `harmony.parts.<p>.loops` est conservé quand les deux sont vides.
- **Poids des ajouts.** Pour les essais (section 5), le poids total des alternatives ajoutées est
  égal au poids total des alternatives écrites (règle de P2).
- **`fifth` = quinte de la fondamentale**, pas de la pédale (cohérent avec `bass.hits` d'E1).
- **Gabarit : un cycle d'une longueur de 1 à 16 mesures** (les motifs écrits vont jusqu'à 4) ; la
  montée repart au début du cycle, sans repli de registre (la validation exige que toutes les
  hauteurs possibles restent dans 0..127 pour les douze classes de départ).
- **`when = chord_change`** : la relance suit les changements, pas les mesures ; `bar_end` couvre
  la basse sur accord tenu.
- **Public** : nouvelles erreurs de validation nommant le champ ; aucun alias, aucune migration de
  profil nécessaire (les champs ajoutés ont une valeur par défaut).

## 4. Manques moteur de P2 (§3 de `p2-variete.md`)

| Manque | Décision |
|---|---|
| 1. Parties `pedal` sans alternatives | **Traité** : `alternatives`, générique, même tirage que les boucles. |
| 2. Aucune portée par partie ou par tour pour la batterie | **Traité** : `part` et `tour` sur un pattern. Garde l'apport de *Diptera Sonata* sans le coût sur *Périculum* quand les patterns de P2 sont liés à leur section. |
| 3. Remplissages non pondérés (`cadence_patterns` unique par densité) | **Laissé** : demande des poids et une portée sur les fills, donc un nouveau tirage au bord de phrase ; le transcripteur ne distingue que kick / snare / hat, aucune donnée pour justifier le mécanisme. |
| 4. Forme fixe par graine | **Laissé** : séquence et longueur des parties fixes dans toute la chaîne (plan de masse, mélodie, intro, arrangement) ; ce n'est pas un petit changement générique. |

## 5. Preuves

### 5.1 Tests de contrat (principe VI)

Fixture `tests/fixtures/profiles/regles_procedurales/` (do ionien verrouillé, deux tours de
vingt mesures : `Chroma`, `Quintes`, `Pedale`) et `harmonium_core/tests/procedural_rules_tests.rs`
(13 tests, flux de `MeasureSnapshot`, graines 1 à 16, attendus écrits relativement à l'accord
publié E3) :

| Test | Contenu |
|---|---|
| `la_boucle_generee_respecte_qualites_mouvement_pedale_et_longueur` | 16 graines : départ sur la tonique, qualité `m`, basse = pédale, mouvement chromatique (±1) entre accords, boucle de 4 mesures **répétée** dans la partie de 8 et **identique au tour suivant**, même graine même flux, ≥ 4 suites de fondamentales distinctes entre graines |
| `une_boucle_generee_partage_le_tirage_pondere_avec_les_boucles_ecrites` | poids 3 contre 1 : les deux sortes tirées, la générée domine |
| `les_quintes_montantes_dans_la_gamme_gardent_leurs_racines_et_varient_en_qualite` | `key_fit = within` : racines 0, 7, 2, 9 ; toutes les notes dans la gamme ; qualités permises ; deux accords par mesure ; 4 répétitions identiques ; qualités variables entre graines |
| `la_pedale_tire_une_alternative_par_air_et_la_garde` | trois pédales tirées sur 16 graines, gardées sur la partie et au tour suivant, notes du voisin |
| `la_quinte_fondamentale_alterne_la_basse_publiee_et_la_quinte_de_la_fondamentale` | `bass_pc` publié aux pas 0 et 8, quinte de `root_pc` aux pas 4 et 12, registre |
| `les_octaves_alternent_et_la_relance_anticipe_la_basse_de_l_accord_suivant` | attaques `(0,3) (4,2) (6,2) (8,3) (12,2) (14,2)`, octave, relance = basse publiée de l'accord suivant (changement interne et passage de mesure) |
| `la_relance_suit_les_changements_d_accord_ou_chaque_barre_selon_when` | aucune relance sur accord tenu, une avant le changement de partie, `bar_end` à chaque barre sans relance interne, aucune relance à la dernière mesure |
| `la_montee_ajoute_un_demi_ton_par_mesure_puis_repart_au_cycle_suivant` | +0, +1, +2, +3 demi-tons sur la basse publiée, repart au tour suivant |
| `la_descente_par_pas_de_gamme_suit_la_gamme_du_mode` | `stride = -2`, `scale_step` : deux notes de gamme par mesure, hauteurs calculées indépendamment |
| `le_pattern_le_plus_specifique_gagne_selon_la_partie_et_le_tour` | repli, `part`, `tour`, deux portées ; égalité partie / tour tirée par poids puis fixée pour la partie |
| `la_batterie_sans_portee_ne_change_pas_de_tour_en_tour` | sans portée, même grille à toutes les mesures |
| `la_validation_refuse_les_contraintes_insatisfaisables_en_nommant_le_champ` | `count`, `bars`, `qualities`, `motion` (quintes descendantes dans la gamme : message avec le mode), `key_fit` (départ non diatonique), `motion.intervals`, `pedal`, poids d'une pédale alternative |
| `la_validation_refuse_les_gabarits_et_les_portees_incoherents` | `ramp.stride`, `pickup.lead`, `part`, `bars`, `rhythm`, exclusivité de `walking`, `percussion.patterns[i].part`, densité sans repli |

Mutations vérifiées (le test échoue, source restaurée) : montée +1 demi-ton, mouvement
chromatique 1 → 2, relance à chaque barre (ce dernier a motivé le test `…selon_when`).

### 5.2 Profils distribués inchangés

| Vérification | Résultat |
|---|---|
| `determinism_dump` (92 fichiers), binaire de la branche contre `main` (`git archive` à `f9de703`) | `diff -r` vide |
| `harmonium-cli --export --seed 1 --bars 32 --record-wav --record-midi` de `reel_quebecois`, `isaac_cellar`, `nt_desert`, lancé depuis `~/Project/bascanada/harmonium`, `--profile` vers les profils de chaque arbre | 3 WAV et 3 MIDI identiques (`cmp`) |
| `harmonium_determinism/scripts/run.sh` (wasm-bindgen 0.2.108, `/tmp/n1/wb`) | natif = wasm : 92 / 92 |
| Le même harnais **avec la fixture ajoutée** (copie jetable hors dépôt : le harnais du dépôt ne couvre que les profils distribués) | natif = wasm : 98 / 98 (6 cas de `regles_procedurales`) : les chemins ajoutés n'emploient que des entiers et des tirages `u64` |
| `TUNING_VERSION` | 24, aucun golden touché |

### 5.3 Suites

| Vérification | Résultat |
|---|---|
| `cargo test --workspace -q --no-fail-fast` | 780 réussis, 0 échec (767 avant, 13 ajoutés) |
| `cargo test -p harmonium --features odin`, `cargo test -p harmonium_bevy` | 118 et 11 réussis |
| `cargo check -p harmonium_studio` ; `cargo check --target wasm32-unknown-unknown` (`harmonium_core`, `harmonium_audio`, `harmonium` avec et sans `odin`, `harmonium_bevy --no-default-features`) | passent |
| `cargo clippy -p harmonium_core --all-targets` comparé à `main` | aucun avertissement nouveau (listes par fichier identiques) |
| `cargo +nightly fmt --all -- --check` | propre |

## 6. Preuve de style : variété par graine et distance au corpus

Profils d'essai **copiés hors dépôt** (`/tmp/p1/profiles/<profil>_p1_<variante>`, profils du dépôt
intacts), options activées par un script jetable ; 16 graines × 64 mesures (`make seed-variety`
du labo, binaire de la branche) ; distance : `make style-distance`, mêmes paramètres que P2
(graines 1–10, 48 mesures, `type=zone`, `GeneralUser-GS.sf2` et `nt_desert.sf2`, mêmes pistes de
référence, cache audio du corpus de P2), moteur compilé depuis un instantané de la branche.
Le profil écrit est **re-mesuré** par le même moteur : variété et distances de V1 retrouvées à
l'identique de `seed-variety-V1.json` et de P2 (`nt_desert` audio 57,5 / 58,0, symbolique 48,4 ;
`isaac_burning` audio 39,6, symbolique 60,6).

### 6.1 Options activées (les mêmes données dans les deux profils, au poids total des écrites)

`nt_desert` (12/8, créneau d'une mesure) :

```toml
[harmony.parts.A1]                      # pédale : tonique 3, quinte 1
kind = "pedal"
degree = 1
weight = 3
alternatives = [{ weight = 1, semitones = 7 }]
[harmony.parts.A2]                      # + loops écrite (Ré mineur sur 8 mesures)
generated = [{ weight = 1, bars = 4, count = 4, qualities = ["m"], motion = { kind = "chromatic" }, start = 6, pedal = 6 }]
[harmony.parts.B]
generated = [{ weight = 1, bars = 8, count = 4, qualities = [""], motion = { kind = "chromatic" }, start = 11 }]
# basse : A1 = tenue de 22 pas, ramp = { stride = 1 } sur 4 mesures (poids 1) ;
#         A2 = 4 attaques, cycle racine/racine/quinte/racine à l'octave 1, relance de 3 pas (poids 4)
# batterie : tour 2, le groove de densité 2 écrit joue à la densité 1
```

`isaac_burning` (4/4, créneau d'une mesure) :

```toml
[harmony.parts.R]                       # + voisin b2 sur la tonique (invisible à la signature d'accords)
kind = "pedal"
degree = 1
weight = 3
alternatives = [{ weight = 1, semitones = 0, neighbor_semitones = 1 }]
[harmony.parts.C]                       # idem C2 ; poids 2 = total des deux boucles écrites
generated = [{ weight = 2, count = 8, qualities = ["", "m"], key_fit = "within",
  motion = { kind = "intervals", intervals = [{ semitones = 3, weight = 2 }, { semitones = 5, weight = 3 },
    { semitones = -3, weight = 2 }, { semitones = -5, weight = 2 }, { semitones = 0, weight = 1 }] } }]
# basse : C et C2 = huit croches sur la fondamentale à l'octave 1, relance de 2 pas (poids 3)
# batterie : part = "C2", les deux grooves de densité 2 écrits jouent à la densité 1
```

### 6.2 Variété par graine

16 graines × 64 mesures, premier air. `distinctes / partie` = signatures distinctes par partie.

| Profil | Couche | V1 : mesures qui varient | P1 | V1 : parties | P1 | V1 : distinctes / partie | P1 |
|---|---|---:|---:|---:|---:|---:|---:|
| `nt_desert` | accords | 0 % | **66 %** | 0 % | **86 %** | 1,00 | **4,14** |
| | basse | 25 % | **67 %** | 29 % | **86 %** | 3,29 | **7,86** |
| | batterie | 0 % | 0 % | 0 % | 0 % | 1,00 | 1,00 |
| | une couche au moins / les trois | 25 / 0 % | 72 / 0 % | 29 / 0 % | 86 / 0 % | | |
| `isaac_burning` | accords | 23 % | **36 %** | 38 % | 38 % | 1,38 | **4,12** |
| | basse | 100 % | 100 % | 100 % | 100 % | 11,62 | 11,62 |
| | batterie | 88 % | 88 % | 100 % | 100 % | 2,00 | 2,00 |
| | les trois couches | 22 % | **31 %** | 38 % | 38 % | | |

Lecture : la part de parties qui varient sature (`isaac_burning` : 100 % déjà sur deux couches) et
**le critère du plan (les trois couches) ne bouge que par les mesures** sur `isaac_burning`
(22 → 31 %), pas sur `nt_desert` dont la batterie n'a aucune source de variété (la portée de tour
est déterministe, et une portée d'une seule alternative ne tire rien). Le gain est dans la
profondeur des accords (1,0 → 4,1 signatures par partie). Le voisin de la pédale de
`isaac_burning` change les notes d'accord, pas la fondamentale : la signature d'accords de la
métrique ne le voit pas. La basse de `nt_desert` suit les accords générés (motifs sur `chord_root`) :
la variante accords seule fait déjà passer la basse de 25 à 30 % des mesures.

### 6.3 Distance au corpus

Score 0–100 (100 = identique) ; `Δ` contre le profil écrit ; tolérance **1 point** (bruit mesuré par
P2 : 0,0 à 0,6). Audio : moyenne des pistes de référence par banque. Symbolique : moyenne, puis
groupes basse / batterie / accords / boucles.

| Profil | Variante | Audio | Symbolique | basse / batterie / accords / boucles |
|---|---|---|---|---|
| `nt_desert` | écrit | nt_desert 57,5 / generaluser 58,0 | 48,4 | 43,1 / 30,4 / 77,0 / 43,0 |
| | tout | 58,5 (+1,0) / 58,7 (+0,7) | 45,3 (**−3,1**) | 35,5 (**−7,6**) / 30,4 / 78,0 (+1,0) / 37,2 (−5,8) |
| | accords seuls | 57,5 (0,0) / 58,2 (+0,2) | 48,2 (−0,2) | 43,2 (+0,1) / 30,4 / 78,0 (+1,0) / 41,1 (−1,9) |
| | basse seule (rampe A1 + riff A2) | 58,5 (+1,0) / 58,8 (+0,8) | 45,5 (**−2,9**) | 35,8 (**−7,3**) / 30,4 / 77,0 / 38,5 (−4,5) |
| | · rampe A1 seule | 58,4 (+0,9) / 58,4 (+0,4) | 46,2 (**−2,2**) | 36,8 (**−6,3**) / 30,4 / 77,0 / 40,7 |
| | · riff A2 seul | 57,6 (+0,1) / 58,2 (+0,2) | 47,7 (−0,7) | 42,0 (**−1,1**) / 30,4 / 77,0 / 41,5 |
| | batterie seule (tour 2) | 57,6 (+0,1) / 57,9 (−0,1) | 48,3 (−0,1) | 43,2 / 30,4 (0,0) / 77,0 / 42,5 (−0,5) |
| `isaac_burning` | écrit | generaluser 39,6 | 60,6 | 59,1 / 76,7 / 61,2 / 45,3 |
| | tout | 40,5 (+0,9) | 59,7 (−0,9) | 61,2 (+2,1) / 74,3 (**−2,4**) / 61,1 (−0,1) / 42,2 (−3,1) |
| | accords seuls | 39,8 (+0,2) | 60,8 (+0,2) | 61,2 (+2,1) / 76,7 / 61,1 (−0,1) / 44,3 (−1,0) |
| | basse seule | 40,0 (+0,4) | 60,5 (−0,1) | 59,1 / 76,7 / 61,2 / 45,0 (−0,3) |
| | batterie seule (C2) | 39,7 (+0,1) | 59,5 (**−1,1**) | 59,1 / 74,3 (**−2,4**) / 61,2 / 43,5 (−1,8) |

`boucles` baisse dès qu'une alternative est ajoutée à une couche (répétition à la longueur de
boucle de la référence, par construction : −0,3 à −5,8, même constat que P2) ; elle n'entre pas dans
la règle. Biais d'optimisme : la distance symbolique vient des mêmes transcriptions que les
propositions de P2, mais ici **aucune donnée du corpus** n'entre dans les options : les baisses et
les hausses sont des effets de la règle, pas de la source.

### 6.4 Recommandation par profil

Règle de P2 : une couche est intégrée si sa variété monte et si ni sa distance symbolique de
couche ni la distance audio globale ne perdent plus de 1 point. Toute intégration reste soumise
à l'écoute du propriétaire ; **rien n'est intégré aux profils distribués**.

| Profil | Accords | Basse | Batterie | Décision |
|---|---|---|---|---|
| `nt_desert` | boucles générées A2 (4 accords mineurs chromatiques sur pédale de ré) et B, pédale A1 alternative : variété 0 → 66 %, distance symbolique −0,2, audio 0,0 / +0,2 | rampe A1 : couche −6,3 ; riff A2 : couche −1,1 (hors règle de peu), audio +0,1 / +0,2 | tour 2 plus dense : neutre (−0,1), aucune variété | **Intégrer les accords.** Abandonner la rampe. Riff A2 : seulement si l'écoute le justifie (−1,1). Batterie : sans intérêt de variété, utile quand P2 apportera des patterns liés à une section |
| `isaac_burning` | boucles générées C et C2 (`within`, intervalles pondérés) : signatures par partie 1,38 → 4,12, distance +0,2, audio +0,2 | gabarits C et C2 avec relance : variété par mesure 2,44 → 2,95, distance −0,1, audio +0,4 | grooves denses liés à C2 : couche −2,4, symbolique −1,1 | **Intégrer accords + basse** (relance à écouter). **Ne pas intégrer** la batterie par portée de partie avec ces grooves |

## 7. Écoute (pour le propriétaire)

Rendus 64 mesures, 109,7 s, crête −1,5 à −6,8 dBFS, dans `/tmp/p1/listen/`. Chaque paire : profil
écrit (`_v1_`) puis essai, **même graine**. Profils d'essai : `/tmp/p1/profiles/` (copies, voir 6.1).

```sh
cd ~/Project/bascanada/harmonium
CLI=~/orca/workspaces/harmonium/p1-procedural-regles/target/release/harmonium-cli
A=~/Project/bascanada/harmonium/assets
# nt_desert, configuration recommandée (accords), graine 5 : nt_desert.sf2
$CLI --export --bars 64 --seed 5 --profile ~/orca/workspaces/harmonium/p1-procedural-regles/profiles/nt_desert \
  --soundfont $A/nt_desert.sf2 --percussion-soundfont $A/nt_desert.sf2 --record-wav /tmp/p1/listen/nt_desert_v1_s5.wav
$CLI --export --bars 64 --seed 5 --profile /tmp/p1/profiles/nt_desert_p1_chords \
  --soundfont $A/nt_desert.sf2 --percussion-soundfont $A/nt_desert.sf2 --record-wav /tmp/p1/listen/nt_desert_chords_s5.wav
afplay /tmp/p1/listen/nt_desert_v1_s5.wav ; afplay /tmp/p1/listen/nt_desert_chords_s5.wav
```

Autres paires (même commande, `--profile` et `--seed` à changer) :
`nt_desert_v1_s6` / `nt_desert_p1_s6` (`nt_desert_p1_all`, graine 6, toutes les options),
`isaac_burning_v1_s10` / `isaac_burning_chords_bass_s10` (`isaac_burning_p1_cb`, configuration
recommandée) / `isaac_burning_p1_s10` (`isaac_burning_p1_all`, avec la batterie de C2) ;
`GeneralUser-GS.sf2` pour les deux fichiers `isaac_burning`.

Moments où l'essai joue autre chose que le profil écrit (mesures dont la signature de la couche
diffère à la même position, minutes:secondes ; une mesure dure 1,7 s) :

| Paire | Couche | À écouter |
|---|---|---|
| `nt_desert_chords_s5` | accords | 0:00–0:13,7 (pédale A1 alternative), 0:15,4–0:17,1, 0:18,9–0:20,6, 0:22,3–0:24,0 et 0:25,7–0:27,4 (A2 : mesures paires de la boucle chromatique), 0:58,3–1:01,7, 1:05,1–1:36,0 (B généré, tour 2) |
| | basse | 0:58,3–1:00,0 et 1:05,1–1:06,9 (la basse suit la racine de l'accord) |
| `nt_desert_p1_s6` (toutes options) | accords | 0:00–0:13,7, 0:15,4–0:20,6, 0:22,3–0:27,4, 1:08,6–1:36,0 |
| | basse (rampe A1, riff A2) | 0:00–0:27,4, 1:08,6–1:49,7 |
| | batterie (tour 2) | 1:22,3–1:34,3 et 1:36,0–1:48,0 |
| `isaac_burning_chords_bass_s10` | accords et basse | 0:13,7–0:27,4 (C, mesures 9–16), 0:42,9–0:54,9 (C2 : accords ; basse dès 0:41,1), 1:22,3–1:36,0 (C, tour 2) |
| `isaac_burning_p1_s10` | batterie (grooves denses de C2) | 0:41,1–0:53,1 |

À juger : (1) la boucle chromatique de `nt_desert` sur pédale de ré : tension utile ou désagrément ?
(2) la relance de basse d'`isaac_burning` avant chaque changement : naturelle ou mécanique ?
(3) l'alternance racine / quinte (`nt_desert_p1_s6`, riff A2) et la rampe sont dans l'essai « tout » de
`nt_desert` ; l'alternance d'octaves n'est que dans la fixture (`tests/fixtures/profiles/regles_procedurales`,
à rendre avec `--profile` sur le dossier).

## 8. Limites

- **Pas de lien qualité ↔ racine** hors `key_fit` : une grille « I majeur, ii mineur, V7 » reste
  écrite ; un gabarit de qualités par degré demanderait un schéma de règles de substitution (E8).
- **Durées égales** (le reste aux premiers accords) : pas de boucle générée du type 2 + 1 + 1 créneaux.
- **Bascule de mode** : une boucle `scale_steps` ou `within` est régénérée dans le nouveau mode (même
  graine, autre boucle possible).
- **Rampe sans repli de registre** : la hauteur sort de la fenêtre `low..high` ; la validation ne
  refuse que le débordement de MIDI.
- **Métrique de variété** : la pédale à voisin n'y apparaît pas ; la part de parties est saturée ;
  une seule graine par essai d'écoute ; aucune écoute humaine revendiquée.
- **Harnais natif / wasm** : couvre seulement les profils distribués ; preuve de la fixture par une
  copie jetable (section 5.2).
- **Labo** : `p2/enrich.py` lit `loops` d'une partie : il n'est pas prévu pour les parties
  `generated` seules (aucun profil distribué n'en a).

## 9. Fichiers

Code : `style_profile.rs`, `models/chord_loop.rs`, `models/loop_generation.rs` (nouveau),
`compose/accomp_profile.rs`, `compose/accomp.rs`, `compose/bass_template.rs` (nouveau),
`compose/percussion.rs`, `compose/generator.rs`, `compose/mod.rs`, `models/mod.rs`.
Tests : `harmonium_core/tests/procedural_rules_tests.rs`, fixture
`tests/fixtures/profiles/regles_procedurales/`. Doc : `profiles/README.md` (section
« Procédural par règles »), ce compte rendu, état de P1 dans `plan.md`.

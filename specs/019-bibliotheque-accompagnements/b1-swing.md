# B1 — `wjazzd_swing` : un swing qui change vraiment d'une graine à l'autre

Date : 2026-11-03. Branche `wquintal/b1-swing`, partie de `main` à `eecbd3f` (`TUNING_VERSION` 26 →
**28**, numéro réservé). Le moteur ne change pas (seul `tuning.rs` : version et sa ligne de doc). Profil
touché : `wjazzd_swing` ; les 24 autres profils sont identiques à `main` (preuves plus bas).

## Constat (D6)

« New seed » donnait « quasiment la même chose » : tempo 150, fa, piano `rootless`, contrebasse, kit jazz,
une boucle par partie, ni arc ni phrases. Mesure V0 (16 graines × 16 chorus) : orchestration 1 séquence
distincte sur 16, batterie identique d'un chorus à l'autre (100 % de paires de chorus identiques).

## Ce qui est livré

Aucun mécanisme nouveau : B1 active en données ceux d'A1 à A6 sur `profiles/wjazzd_swing/`.

| Fichier | Contenu |
|---|---|
| `session.toml` (nouveau) | **sept groupes**, 19 656 habillages : `rythme` (tempo, kit et arc ensemble : lent 100-118, medium 122-142, medium-up 148-186, up 192-225, très rapide 232-265 ; balais en majorité au tempo lent, jamais au-delà de 142), `tonalite` (fa, si♭, mi♭, do, sol, la♭, ré), `comping` (piano 0, guitare de jazz 26, orgue 17), `basse` (contrebasse 32 ; basse aux doigts 33 à 1 sur 15), `voicing` (`rootless`, `guide_tones`, `drop2`, `drop3`, `fourths`, `shell`), `registre`, `energie` ; **six arcs nommés** (`exposition_baguettes`, `exposition_balais`, `directe_baguettes`, `directe_balais`, `orchestre`, `rapide`) : exposition à deux temps puis marche, sommet, retombée, breaks et stop-time |
| `profile.toml` | plus de `[fixed_tonality]` (la graine tire la tonalité, l'hôte la verrouille par la sienne) ; **4 blues** (simple, changement rapide, de jazz, sous-dominante mineure), **2 A** et **2 B** par défaut (une boucle par partie et par graine) ; basse d'A4 (arrivées tierce / quinte / septième, notes répétées, chromatisme, côté d'approche, croches, deux temps) ; `color` et `register_drift` d'A5 ; **15 motifs** de comping étiquetés `sparse` / `medium` / `dense` et **24 phrases** (2 à 8 mesures, `part_start`, `part_end`, `chorus_end`) ; décalage du comping sur les temps seulement |
| `percussion.toml` (généré) | patterns du GMD (étiquettes `spang`, `quarters`), **10 suites de figures** mesurées sur 2 et 4 mesures, **relances** du GMD (une mesure) et en levée, patterns et phrases `brush` (règle de métier) ; `specs/019-bibliotheque-accompagnements/b1/batterie_jazz.py` |
| `swing.toml` (nouveau) | courbes JTD par piste (A6) : batterie 0,721 jusqu'à 140 BPM puis −0,098 par 100 BPM, basse 0,715 puis −0,044, comping 0,702 puis −0,073 |
| `dynamics.toml`, `arrangement.toml` | courbes de vélocité et densité de la batterie de l'arc (comme la fixture A3), seuils de bascule de mode écartés de la valence neutre (la tonalité ne change plus de mode seule) ; quatre chorus par air |

Provenance dite dans l'en-tête de chaque fichier : **mesuré** (tempos, patterns, suites et relances du GMD,
courbes de swing, poids de basse) et **règle de métier** (tonalités, instruments, voicings, rythmes et phrases
de comping, balais, croches de basse, arcs, énergie, part de fins de phrase sans relance, relances en
levée). Aucun corpus ne donne la dynamique d'un chorus.

## Preuves

### Habillages distincts sur 16 graines (`b1/habillages.py`, vérité terrain du CLI)

| Graines 1 à 16 | Avant B1 | Après B1 |
|---|---:|---:|
| habillages distincts (tempo à ± 8 BPM, tonalité, programmes comping / basse / kit, voicings) | 1 | **16** |
| grappes de tempo à ± 8 BPM | 1 (150) | 8 (106 à 259 BPM) |
| tonalités | 1 | 7 |
| comping / basse / kit | 0 / 32 / 32 | {0, 17, 26} / {32, 33} / {32, 40} |
| familles de voicings | 1 | 6 |

Sur 160 graines : 13 variantes de `rythme` tirées, dans les proportions des poids (up 19 %, très rapide 14 %,
medium-up 26 %, medium 24 %, lent 16 %). Sous une grille d'hôte de huit mesures : mêmes habillages
(16 / 16, 7 tonalités, 8 grappes) : la variante ne dépend pas de la grille.

| Graine | `rythme` | BPM | Tonalité | Comping | Basse | Kit | Voicings |
|---:|---|---:|---|---|---|---|---|
| 68 | `lent_balais_exposition` | 115 | ré | piano | contrebasse | balais | `rootless` |
| 4 | `medium_baguettes_exposition` | 140 | mi♭ | guitare | basse aux doigts | baguettes | `fourths` |
| 1 | `up_rapide` | 219 | si♭ | piano | contrebasse | baguettes | `drop3` |
| 5 | `lent_orchestre` | 116 | la♭ | orgue | contrebasse | balais → baguettes → balais | `rootless` |

### `seed-variety --chorus 4`, 16 graines (labo, binaire de la branche ; `/tmp/harmonium-019/b1/sv_*.md`)

Avant = `profiles/wjazzd_swing` de `main` (mêmes mesures que V0 : orchestration 1 / 16), après = B1. Grille par
défaut (44 mesures) et grille d'hôte de 8 mesures (`v0_user_grid_profile.py`).

| Couche | Grille 44 : avant | après | Grille 8 : avant | après |
|---|---:|---:|---:|---:|
| accompagnement : distance inter-graines | 35 % | **63 %** | 39 % | **59 %** |
| accompagnement : séquences distinctes | 16 / 16 | 16 / 16 | 16 / 16 | 16 / 16 |
| comping rythme : distance inter-graines | 68 % | 56 % | 68 % | 55 % |
| comping voicings : distance inter-graines | 46 % | **95 %** | 36 % | **95 %** |
| comping rythme : mesures changées d'un chorus au suivant | 80 % | 84 % | 81 % | 85 % |
| comping voicings : mesures changées d'un chorus au suivant | 74 % | **94 %** | 73 % | **93 %** |
| basse : distance inter-graines | 59 % | **92 %** | 67 % | **92 %** |
| basse : mesures changées d'un chorus au suivant | 92 % | 100 % | 95 % | 100 % |
| **batterie** : séquences distinctes | 11 / 16 | **16 / 16** | 3 / 16 | **16 / 16** |
| batterie : distance inter-graines | 19 % | **55 %** | 21 % | **55 %** |
| batterie : mesures changées d'un chorus au suivant | **0 %** | **83 %** | 0 % | 84 % |
| batterie : paires de chorus identiques | **100 %** | **0 %** | 100 % | 3 % |
| **orchestration** : séquences distinctes | 1 / 16 | **9 / 16** | 1 / 16 | **9 / 16** |
| orchestration : places où les graines diffèrent | 0 % | 100 % | 0 % | 100 % |
| accords : séquences distinctes (boucles par défaut) | 1 / 16 | 9 / 16 | 1 / 16 | 1 / 16 (grille d'hôte) |

Le comping rythme perd 12 points de distance inter-graines : tirage par phrases (suites de motifs répétés), le
même constat qu'A1 (la structure gagne ce que la distance perd) ; les rythmes restent distincts sur 16 / 16 graines.
L'orchestration ne tombe pas à 16 / 16 parce que seuls les arcs `orchestre` (14 graines sur 160 :
9 %) et les kits de session distinguent les séquences. Trajectoire (chorus 1, 2, 3, dernier) de la densité de
notes de comping : 5,1 ; 8,4 ; 9,9 ; 2,5 par mesure ; de la batterie : 6,9 ; 9,5 ; 10,3 ; 4,7 (arc sur quatre chorus).

### Basse contre WJazzD (`make backing-validate` : 8 graines × 88 mesures, **sans perte**)

Même copie de profil sans arc ni croches que A4 (le validateur ne lit que des mesures à une note par temps).

| Famille | B1 | Avant (E7 + S0) |
|---|---:|---:|
| approche | 0,076 | 0,167 |
| intervalles | 0,082 | 0,137 |
| registre | 0,118 | 0,105 |
| passage | 0,044 | 0,129 |
| arrivée | 0,333 | 0,678 |
| rythme harmonique (propriété des grilles par défaut : les blues de jazz ont plus de ii-V) | 0,372 | 0,302 |
| **moyenne** | **0,158** | **0,246** |

Groupes de WJazzD : tous 0,158 (0,246) ; blues 0,194 (0,280) ; `reliable` 0,147 (0,219) ; < 130 BPM 0,141
(0,217) ; 130-175 0,153 (0,234) ; 175-235 0,160 (0,248) ; ≥ 235 0,182 (0,274). Poids repris d'A4 sans
recalibrage : la distance baisse sur toutes les tranches de tempo.

### Swing mesuré sur le MIDI exporté contre JTD (`b1/mesure_swing.py`, 6 graines × 96 mesures, tempo imposé)

Médiane des attaques de contretemps (0,45 à 0,85 de la noire), mesuré / JTD :

| Tempo | Batterie (ride) | Basse | Comping |
|---:|---|---|---|
| 110 | 0,721 / 0,721 | 0,710 / 0,715 | 0,702 / 0,702 |
| 150 | 0,713 / 0,712 | 0,702 / 0,706 | 0,702 / 0,702 |
| 190 | 0,673 / 0,673 | 0,683 / 0,688 | 0,673 / 0,673 |
| 230 | 0,633 / 0,633 | 0,667 / 0,671 | 0,644 / 0,644 |
| 270 | 0,594 / 0,595 | 0,650 / 0,653 | 0,615 / 0,615 |

Écart maximal 0,005 (basse : 237 attaques de croches seulement). Avant B1 : 0,710 à tous les tempos, soit 0,115
au-dessus de JTD à 270 BPM. **Défaut trouvé** : avec le retard de 0,2 pas du piano appliqué à toutes les attaques,
le comping mesurait 0,03 trop haut (0,731 à 110 BPM) : la courbe du piano est déjà mesurée sur ses contretemps. Le
retard ne vaut plus que sur les temps (pas 0, 4, 8, 12).

### Non-régression et suites

| Vérification | Résultat |
|---|---|
| dumps `harmonium_determinism`, binaire de `main` (`git archive` de `eecbd3f`, cible séparée) contre la branche | 153 fichiers : **147 identiques**, 6 changés (`wjazzd_swing_s{1,2,3}_{mid,hi}.json`) |
| natif = wasm (`WASM_BINDGEN=/tmp/harmonium-019/wb/bin/wasm-bindgen …/run.sh`) | 153 / 153 |
| MIDI `--export --style <id> --seed {1,7} --bars 24` des 24 autres profils, CLI de `main` et de la branche | 48 / 48 `cmp` identiques |
| `TUNING_VERSION` | 28 ; `profile_d2_main.json` : seule la chaîne `source` change (`runs` identiques, vérifié) ; `set_contract_tests` aligné |
| `cargo test --workspace -q --no-fail-fast` | **1 000 réussis, 0 échec, 2 ignorés** |
| `cargo test -p harmonium --features odin` ; `-p harmonium_bevy` ; `cargo check -p harmonium_studio` ; `cargo +nightly fmt --all --check` | passent (odin : 152+ réussis ; Bevy : 11) |
| clippy `harmonium_core --all-targets` | aucun avertissement dans le fichier de tests ajouté |
| **Seek** `deterministic_seek(64)`, graine 42, build de test, minimum de 25 | `reel_quebecois` 25,6-25,9 ms (inchangé) ; **`wjazzd_swing` 18,5 → 34,4 ms** (+16 ms : phrases, arc et session rejoués par mesure) ; `seek_to_bar_64_under_50ms` (profil par défaut) passe. Sous 50 ms mais en hausse : **à signaler** (D5) |

## Tests

`harmonium_core/tests/wjazzd_swing_profile_tests.rs` réécrit (13 tests, flux de `MeasureSnapshot`) : les
sept contrats d'E7 qui supposaient un profil fixe sont remplacés ou généralisés. Conservés : accord slash joué
sur sa basse, rejeu à l'identique, graines différentes → lignes différentes. Remplacés : grille par défaut
(4 blues × 2 A × 2 B, les reprises de A gardent leur version), arrivées dans l'accord publié et fenêtre de la
basse (la basse n'arrive plus toujours sur la fondamentale, A4), comping dans l'accord publié (jusqu'à 48 %
de tensions en quartes), kit et patterns ensemble (balais : balayage, jamais de ride ; baguettes au sommet de
l'arc, jamais de balayage). Nouveaux : ≥ 12 habillages distincts sur 16 graines dans les plages de tempo ;
balais plus probables aux tempos lents, absents au-delà de 142 BPM (200 graines) ; rapport de swing = courbe JTD
de la batterie à 0,002 près, décroissant avec le tempo ; arc (deux temps → marche → sommet plus dense →
retombée, stop-time à l'avant-dernière mesure) ; arc `orchestre` (kits des quatre chorus) ; **sous une grille
d'hôte de huit mesures** : la grille ne bouge jamais, la graine habille (tonalités, tempos), l'arc contraste, et
la tonalité de l'hôte (`[fixed_tonality]`) gagne en laissant le reste de la variante. `set_contract_tests` :
version 28.

## Swing rapide : décision pour T1

**Pas de profil frère `wjazzd_swing_up`.** `wjazzd_swing` couvre « Medium Swing », « Medium Up Swing » et « Up Tempo
Swing ». Raisons : (1) le swing suit le tempo, mesuré (tableau ci-dessus : 0,72 à 110 BPM, 0,64 à 220) ; (2) la basse
calibrée sur toutes les tranches de WJazzD reste à moins de 0,05 de sa moyenne sur chacune (0,141 à 0,182) ; (3)
tempo, kit et arc sont tirés ensemble (groupe `rythme`), donc une graine est cohérente avec son propre tempo.
**Limite** : un hôte qui impose son BPM (`SetBpm`) n'impose pas la variante ; une graine lente jouée à 230 BPM
garde ses balais et son exposition à deux temps. **Usage pour T1** : lire `MeasureSnapshot.session` (groupe
`rythme`) et choisir une graine dont la variante est à l'échelle du tempo voulu (`medium_*`, `medium_up_*`,
`up_rapide` ou `tres_rapide`) ; ne pas imposer `SetSwing` (désactive les courbes par piste). Détail :
`profiles/wjazzd_swing/README.md`, « Swing rapide ».

## Manques de schéma pour A7

1. **Imposer une variante** (« les balais, quelle que soit la graine ») : aucune commande, A2 le notait déjà ; T1 le contourne par le choix de la graine.
2. **Réglages dépendants du tempo** : poids de basse, croches, comping, ne se surchargent pas par variante (liste blanche d'A2) ni selon le tempo imposé par l'hôte ; le kit et l'arc sont fixés par la graine, pas par le BPM de l'hôte.
3. **Dépendance entre groupes** : tempo, kit et arc sont réunis dans un seul groupe de 13 variantes pour garder
   le couplage ; un couplage déclaré (« balais seulement sous 142 BPM ») éviterait ce produit.
4. **Mixage par timbre** : `[mix]` n'est pas surchargeable par variante (piano, guitare et orgue au même niveau, non mesuré).
5. **Seek** : le rejeu d'une session à phrases, arc et variantes coûte +16 ms par `deterministic_seek(64)` sur ce profil.

## Limites

- Valeurs de service sans corpus : arcs, énergie, tonalités, instruments, rythmes et phrases de comping, balais
  (`brush`), part de fins de phrase sans relance, croches de la basse.
- GMD jazz : 5 batteurs, 22 relances d'un seul batteur, kit électronique ; les relances en levée (pas 8 et 12)
  sont la relance mesurée tronquée.
- Écoute : aucune (D5/D7) ; les liens ci-dessous sont à ouvrir par l'orchestrateur.
- Un chorus de 44 mesures (blues puis AABA) n'est le chorus de l'arc que sous les grilles par défaut.
- Mixage : même niveau pour le piano, la guitare et l'orgue ; balais jamais mesurés au LUFS.

## Fichiers

Données : `profiles/wjazzd_swing/{profile,session,swing,percussion,dynamics,arrangement}.toml`, `README.md`.
Outils : `specs/019-bibliotheque-accompagnements/b1/{batterie_jazz,habillages,mesure_swing}.py`. Tests :
`harmonium_core/tests/{wjazzd_swing_profile_tests,set_contract_tests}.rs`. Version : `harmonium_core/src/tuning.rs`,
`harmonium_cli/tests/fixtures/profile_d2_main.json`. Docs : `profiles/README.md`, `plan.md`, ce compte rendu.

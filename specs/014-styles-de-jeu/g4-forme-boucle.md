# G4 — Forme en boucle

## Ce qui a changé

1. **Banques de cellules exigées pour les seuls rôles employés.**
   `StyleProfile::validate` ne réclame plus `Run`, `Arp` et `Turn` en bloc : chaque `CellRole` n'est exigé que si la forme (`parts.roles` ou `second_ending`) peut le tirer — rôle imposé (`Open`, `CadHalf`/`CadFifth`, `CadFull`/`CadTonic`) ou rôle libre (`Free`, `CadOpen`, `CadOther`) affecté d'un poids strictement positif dans `ReelFiddleParams::free_role_weights`. Un poids nul retire le rôle du tirage et des replis déterministes de `pick_cell_for_bar` ; la courbe d'intensité laisse nul un poids nul au lieu de le relever à `1`. La somme des poids libres peut valoir zéro si la forme n'emploie aucun rôle libre.

2. **Mélodie gouvernée par l'arrangement (et la densité) comme les autres pistes.**
   `TrackEntries.melody` devient `Option<EntryPoint>` ; seuls `Heel` et `Toe` restent obligatoires. `StyleProfile::has_track(TrackId::Melody)` lit `self.arrangement.has_track(TrackId::Melody)` au lieu de renvoyer `true` en dur. La mélodie peut entrer à n'importe quels `tour`/`part`/`pass` ou être omise ; une section `[heterophony]` exige explicitement `arrangement.entries.Melody`. La levée d'intro (`pickup_eighths`) et l'outro tonique ne posent de notes sur `Melody` que si la piste ouvre la forme (ou existe).

3. **Boucle courte sans cadence : fins de phrase et protection d'ornements.**
   `ImposedRole::is_cadential` factorise le test des six rôles cadentiels (`CadHalf`, `CadFull`, `CadTonic`, `CadOpen`, `CadFifth`, `CadOther`). Lorsque `ornament_protected_bars` est omis, la dernière mesure de la partie n'est protégée des ornements que si son rôle résolu est cadentiel : une boucle `Open`/`Free` reçoit donc des ornements jusque sur sa dernière mesure, tandis que les dix profils distribués (dont la dernière mesure est toujours cadentielle) gardent le même comportement. À défaut de `phrase_ends` et de rôle cadentiel, `phrase_window` couvre la partie entière : la fin de boucle sert de frontière de phrase unique.

## Choix et pourquoi

- **Un seul chemin d'arrangement (`Option<EntryPoint>`).** Aucun drapeau dédié « boucle » ni mode sans mélodie séparé (principe IV) : `Melody` passe par `has_track`, `arrangement.allows` et `dynamics.allows` comme `Chords`, `Bass` et `Countermelody`. Le plan mélodique interne continue d'être résolu pour que le vote harmonique et la conduite d'accords restent déterministes même pendant les tours où la mélodie se tait.
- **Poids libre nul = rôle absent.** Mettre `weight_role_arp = 0` ou `weight_role_turn = 0` suffit à déclarer une boucle qui n'utilise que `open` et `run`, sans clé supplémentaire dans le schéma TOML.
- **Protection implicite liée au caractère cadentiel.** Protéger la dernière mesure inconditionnellement supposait que toute partie se conclut par une cadence ; tester `role.is_cadential()` lève cette hypothèse sans toucher aux profils existants.

## Fichiers touchés

- `harmonium_core/src/trad/plan.rs` — `ImposedRole::draws_free_cell`, `ImposedRole::is_cadential`.
- `harmonium_core/src/tuning.rs` — `ReelFiddleParams::free_role_weights`.
- `harmonium_core/src/style_profile.rs` — `ProfileRole::needs_cell_role` / `draws_free_cell`, `has_track(Melody)`, validation conditionnelle des banques et de `heterophony`.
- `harmonium_core/src/trad/arrangement.rs` — `TrackEntries.melody: Option<EntryPoint>`, `ReelArrangement::opens` / `entry_ordinal`.
- `harmonium_core/src/trad/fiddle.rs` — replis et tirage libre filtrés par `free_role_weights`.
- `harmonium_core/src/trad/generator.rs` — échelle des poids libres préservant `0`, protection d'ornements sur dernière mesure cadentielle, levée et outro conditionnées à `Melody`.
- `harmonium_core/tests/style_profile_tests.rs` — adaptation au type `Option<EntryPoint>`.
- `harmonium_core/tests/boucle_form_tests.rs` — tests de contrat sur snapshots.
- `tests/fixtures/profiles/boucle_test/` — fixture boucle 4 mesures en 2/2 (`Open`, `Free`, `Free`, `Free`, `second_ending = "Free"`, mélodie au tour 2, banques `open` + `run`).
- `tests/fixtures/profiles/boucle_gigue_test/` — fixture boucle 2 mesures en 6/8 (`Free`, `Open`, mélodie à la passe 2, banques `open` + `run` + `turn`, principe VIII).
- `profiles/README.md` — documentation du schéma et recette « Boucle courte sans cadence ».

## Preuves de test

- **Non-régression des 10 profils distribués (`TUNING_VERSION = 21` inchangé)** : export `--bars 64 --seed 7 --record-truth` comparé avant/après sur `reel_quebecois`, `gigue_quebecoise`, `valse_quebecoise`, `reel_acadien`, `marche_britannique_coloniale`, `gigue_britannique_coloniale`, `hornpipe_britannique_coloniale`, `gigue_acadienne`, `reel_britannique_coloniale`, `valse_britannique_coloniale` — 143 mesures chacun, **0 mesure divergente** (notes, vélocités, interprétation, accords).
- **Tests de contrat (`cargo test -p harmonium_core --test boucle_form_tests`)** :
  - `boucle_quatre_mesures_melodie_entre_au_tour_2` : 64 mesures, mélodie absente aux mesures 1–8 pendant qu'accords/basse/pieds jouent, présente aux mesures 9–64, accords répétés à chaque tour, aucune tenue cadentielle forcée.
  - `boucle_quatre_mesures_repetition_sans_ornements` : répétition exacte de la boucle de 4 mesures à travers passes et tours, seconde fin à la mesure 64.
  - `boucle_gigue_deux_mesures_melodie_entre_passe_2` : second style en 6/8 (2 mesures × 4 passes), entrée à la passe 2 (mesure 3), répétition exacte sur 64 mesures.
  - `banques_exigees_pour_les_seuls_roles_employes` : rejet `melody.cells.Arp` si `weight_role_arp > 0` sans banque ; acceptation de poids libres tous nuls quand la forme n'emploie que `Open`.
  - `melody_omise_ne_joue_jamais` : `Melody` absente de `[arrangement.entries]` coupe la mélodie sur 64 mesures sans affecter l'accompagnement.
  - `ornements_sans_protection_par_defaut_en_boucle` : la mesure 4 d'une boucle non cadentielle reçoit des ornements par défaut, et constitue la seule frontière `phrase_end = "only"`.
- **Smoke CLI** :
  - `cargo run -q -p harmonium-cli --release -- --export --bars 32 --profile tests/fixtures/profiles/boucle_test --seed 1 --record-midi /tmp/g4-forme-boucle.mid` (5 032 octets, mesures 1–8 sans mélodie, entrée mesure 9 au tour 2, boucle de 4 mesures répétée).
  - `cargo run -q -p harmonium-cli --release -- --export --bars 32 --profile tests/fixtures/profiles/boucle_gigue_test --seed 1 --record-midi /tmp/g4-forme-boucle-gigue.mid` (4 972 octets, mesures 1–2 sans mélodie, entrée mesure 3 à la passe 2, boucle de 2 mesures en 6/8 répétée).

## Limites

- Les entrées d'arrangement (`[arrangement.entries]`) ne s'appliquent qu'au premier air d'un set (`tune_in_set == 1`) : dans un set multi-airs, les airs suivants démarrent toutes leurs pistes dès la première mesure, comportement existant conservé.
- Même lorsque `Melody` est omise de l'arrangement, une cellule d'`Open` ou de rôle libre reste requise pour construire le plan interne qui sert au vote harmonique `trad_functional` ; les modèles d'harmonie en boucle/pédale et la basse motivique relèvent respectivement de G2 et G3.

## Ce qu'il faut écouter

- `/tmp/g4-forme-boucle.mid` : deux passes de 4 mesures basse + accords + percussion seuls (mesures 1–8), puis entrée de la mélodie à la mesure 9 sur le motif de 4 mesures répété sans arrêt cadentiel aux mesures 4, 8, 12, 16…
- `/tmp/g4-forme-boucle-gigue.mid` : même mécanique en 6/8 sur une boucle de 2 mesures, accompagnement seul sur la première passe (mesures 1–2) puis entrée mélodique dès la mesure 3.

# E10 + E11 — Difficulté de la mélodie et question-réponse

Date : 2026-11-03. Branche `wquintal/e1011-melodie-qr`, partie de `main` à `0fe8b52`
(`TUNING_VERSION` 26 **inchangé**). Deux mécanismes optionnels en données, demandés par
`harmonium_training` (`specs/010-lecture-impro/demandes-moteur.md`, § E10 et § E11) : un profil qui
ne les déclare pas garde ses dumps, ses WAV et ses goldens (preuves plus bas). Aucun profil distribué
n'est touché. Schéma et exemples : `profiles/README.md`, « Difficulté de la mélodie » et
« Alternance de mesures (question-réponse) ».

## Ce que training appelle ou écrit

### E10 — `profile.melody.difficulty`

En données seulement, sur la copie du profil que training installe avant `SetSeed`/`NewMelody`
(comme tessiture et poids de rôles) :

```rust
use harmonium_core::compose::MelodyDifficulty;
profile.melody.difficulty = Some(MelodyDifficulty {
    max_leap_semitones: Some(4),            // plus grand saut, y compris entre cellules et phrases
    max_attacks_per_bar: Some(4),           // densité
    min_note_units: Some(2),                // plus petite durée, en unités de cellule
    max_cell_ambitus_semitones: Some(7),    // ambitus d'une mesure
    out_of_scale: false,                    // aucune note hors de la gamme de l'air
    ornaments: false,                       // aucun ornement sur la mélodie
    chord_fit: false,                       // pas de rapprochement des accords (modèles loop, rewrite)
});
```

ou `[melody.difficulty]` en TOML. Chaque borne est optionnelle. **Pourquoi pas une commande** : le
filtre s'applique au tirage de la ligne de tout un air (cellules, reprises, tours) ; le changer en
cours d'air demande de retirer cette ligne, ce que `SetStyleProfile` fait déjà pour les airs suivants
(l'air courant garde ses cellules). Une commande doublerait ce chemin (principe IV) sans rien
permettre de plus à training, qui règle un niveau au démarrage d'une séance. Pour les jeux
(« mélodie plus calme à faible intensité »), le filtre se pose de même dans le profil du jeu ; une
courbe d'intensité qui change le filtre en cours d'air n'est pas livrée (voir Manques).

Ce que le filtre remplace côté training : `simplify_rhythms` (fusion de durées dans les `RhythmPattern`
→ `max_attacks_per_bar`, `min_note_units`), le levier `form.max_leap_degrees` (degrés → demi-tons
sur toute la ligne), `ornaments.rate_scale = 0` (→ `ornaments = false`, ou `true` et `rate_scale`
réduit : les ornements s'ajoutent alors hors des bornes). Tessiture, tempo, poids de rôles et
intensité restent ses leviers.

### E11 — `[arrangement.alternation]`

```toml
[alternation]            # fichier d'arrangement, à côté de [entries]
Melody = { period = 4 }  # joue 2 mesures, se tait 2 ; play = 1..period-1, phase = 0..period-1
```

À l'écriture : `profile.arrangement.alternation.insert(TrackId::Melody, Alternation { period: 4,
play: None, phase: 0 })`. **À la volée** : `SetStyleProfile` avec l'alternance renseignée (ou
vidée) ; elle est lue à chaque mesure composée, donc elle agit sur les mesures à venir sans toucher
aux cellules (test `alternation_live_tests`). `SetStyleProfile` réinitialise tempo et swing, que
l'application renvoie. Pas de nouvelle commande, pour la même raison.

## Livré

- `compose/difficulty.rs` : `MelodyDifficulty` (`[melody.difficulty]`, validé avec chemin du champ),
  préférence au tirage (`pitch_excess`, en tête de la clé de score de `pick_cell_for_bar`, donc sans
  effet quand le profil n'a pas de filtre), réécriture de la ligne jouée de l'air.
- Réécriture : fusion des attaques courtes (la note retirée prolonge la précédente ; temps faibles et
  fin de mesure d'abord), puis hauteurs reprises par **programmation dynamique** sur toute la ligne
  (notes de la gamme dans la tessiture, écart total minimal aux notes écrites, saut et ambitus de
  chaque mesure en contraintes, dernière note d'une cadence plus coûteuse à déplacer). Une ligne
  constante convient toujours : la recherche aboutit, les bornes sont garanties, pas seulement
  préférées. Coût mesuré : 0,03 à 0,6 ms par air (96 mesures).
- Raccords : dernière mesure → première (tour suivant), et dernière mesure du premier tour →
  première (intro d'entrée par la fin). Les mesures d'une partie qui tait la mélodie
  (`tracks_off`) ou muettes par l'alternance ne comptent pas : le saut se borne d'une mesure
  publiée à la suivante. Mesure trouvée en cours de route : `isaac_basement` sautait de 10
  demi-tons par-dessus 20 mesures de silence.
- Un profil à filtre compose sa ligne sur tous les tours (`StyleProfile::expanded_material`, déjà
  employé par les profils à reprises variées) et **n'a plus de levée d'intro** (ses croches ne
  tiennent pas dans les bornes ; la levée recouvrait aussi la fin empruntée de l'intro `tail`).
- `ornaments = false` : `apply_ornaments` saute la piste `Melody` ; `chord_fit = false` : sortie
  immédiate de `fit_planned_melody` ; `out_of_scale = false` : l'ajustement ne retient que des notes
  de la gamme de l'air, la hauteur écrite sinon.
- E11 : `Alternation { period, play, phase }` dans `Arrangement` (`BTreeMap<TrackId, Alternation>`,
  même désérialisation que `entries`, erreurs `arrangement.alternation.<Piste>[.period|.play|.phase]`),
  `silence_alternated_tracks` en fin de `generate_measure`. Une piste muette est **composée puis
  retirée du snapshot** : les autres pistes ne changent pas, y compris les vétos de la basse de
  passage (retirer la piste avant faisait apparaître des notes de passage de basse aux mesures
  muettes, vu au premier essai). Intro et mesure de clôture non touchées. `Countermelody` suit la
  mélodie : pour la taire aussi, lui déclarer la même alternance.
- Exemple de mesure `harmonium_core/examples/melody_difficulty.rs` (`--filter`, `--explain`,
  `--show`).

## Preuves

### E10 — 3 graines (42, 137, 2026) × 64 mesures, 15 profils distribués avec mélodie

Les leviers de training (`apply_level` : tessiture 12/15/19/profil centrée sur 48–96, saut en degrés
3/4/5, poids de rôles, fusion de `RhythmPattern`, `ornaments.rate_scale` ×0/×0/×0,5/×1) sont
reproduits par l'exemple ; le filtre de référence s'y ajoute aux niveaux 1 à 3, le niveau 4 reste
le profil :

| Niveau | saut max | attaques / mesure | plus petite durée | ambitus d'une mesure |
|---|---|---|---|---|
| 1 | 4 demi-tons | moitié de `cell_units` | 2 × la durée native du profil (≥ 2) | 7 |
| 2 | 5 | trois quarts | 1,5 × la durée native | 9 |
| 3 | 7 | `cell_units` | durée native | 12 |
| 4 | profil | profil | profil | profil |

(durée native = plus petite durée d'une note des cellules : 1 unité pour les reels, 2 pour
`isaac_*`, 3 pour `nt_desert` ; `ornaments`, `chord_fit` et `out_of_scale` à `false` aux niveaux 1 à 3.)
Moyenne sur les graines de l'ambitus total, du plus grand saut (et pire des trois graines) en
demi-tons, des attaques par mesure (et pire mesure), de la plus petite durée en unités (et pire) :

| Profil | Niv. | Ambitus | Plus grand saut (moy. / pire) | Attaques/mesure (moy. / pire mesure) | Plus petite durée (unités, moy. / pire) |
|---|---|---|---|---|---|
| reel_quebecois | 1 | 9.3 | 4.0 / 4 | 3.41 / 4 | 2.0 / 2 |
| reel_quebecois | 2 | 12.7 | 5.0 / 5 | 3.56 / 4 | 2.0 / 2 |
| reel_quebecois | 3 | 15.0 | 7.0 / 7 | 6.47 / 8 | 1.0 / 1 |
| reel_quebecois | 4 | 19.3 | 11.0 / 19 | 7.52 / 11 | 0.3 / 0 |
| gigue_acadienne | 1 | 9.3 | 4.0 / 4 | 1.78 / 2 | 2.0 / 2 |
| gigue_acadienne | 2 | 11.0 | 5.0 / 5 | 1.99 / 2 | 2.0 / 2 |
| gigue_acadienne | 3 | 13.3 | 7.0 / 7 | 4.76 / 6 | 1.0 / 1 |
| gigue_acadienne | 4 | 16.7 | 11.0 / 12 | 5.43 / 11 | 0.0 / 0 |
| gigue_britannique_coloniale | 1 | 8.7 | 4.0 / 4 | 1.79 / 2 | 2.0 / 2 |
| gigue_britannique_coloniale | 2 | 11.7 | 5.0 / 5 | 1.98 / 2 | 2.0 / 2 |
| gigue_britannique_coloniale | 3 | 16.3 | 7.0 / 7 | 4.78 / 6 | 1.0 / 1 |
| gigue_britannique_coloniale | 4 | 20.0 | 11.3 / 15 | 5.09 / 7 | 0.7 / 0 |
| gigue_quebecoise | 1 | 9.7 | 4.0 / 4 | 1.79 / 2 | 2.0 / 2 |
| gigue_quebecoise | 2 | 12.0 | 5.0 / 5 | 1.98 / 2 | 2.0 / 2 |
| gigue_quebecoise | 3 | 16.3 | 7.0 / 7 | 4.66 / 6 | 1.0 / 1 |
| gigue_quebecoise | 4 | 14.7 | 10.3 / 12 | 5.06 / 7 | 0.3 / 0 |
| hornpipe_britannique_coloniale | 1 | 10.7 | 4.0 / 4 | 3.35 / 4 | 2.0 / 2 |
| hornpipe_britannique_coloniale | 2 | 10.3 | 5.0 / 5 | 3.63 / 4 | 2.0 / 2 |
| hornpipe_britannique_coloniale | 3 | 17.0 | 7.0 / 7 | 5.92 / 8 | 1.0 / 1 |
| hornpipe_britannique_coloniale | 4 | 20.0 | 12.0 / 12 | 6.37 / 9 | 1.0 / 1 |
| isaac_basement | 1 | 5.7 | 4.0 / 4 | 0.65 / 2 | 4.0 / 4 |
| isaac_basement | 2 | 9.0 | 5.0 / 5 | 0.62 / 2 | 4.0 / 4 |
| isaac_basement | 3 | 10.7 | 7.0 / 7 | 0.77 / 3 | 2.0 / 2 |
| isaac_basement | 4 | 11.3 | 9.0 / 12 | 0.77 / 3 | 2.0 / 2 |
| isaac_burning | 1 | 3.0 | 3.0 / 3 | 0.66 / 2 | 4.0 / 4 |
| isaac_burning | 2 | 4.7 | 3.3 / 5 | 0.66 / 2 | 4.0 / 4 |
| isaac_burning | 3 | 4.3 | 3.3 / 4 | 1.02 / 3 | 2.0 / 2 |
| isaac_burning | 4 | 6.3 | 5.7 / 7 | 0.76 / 3 | 2.0 / 2 |
| isaac_cellar | 1 | 5.7 | 4.0 / 4 | 1.12 / 2 | 4.0 / 4 |
| isaac_cellar | 2 | 9.0 | 5.0 / 5 | 1.11 / 2 | 4.0 / 4 |
| isaac_cellar | 3 | 10.0 | 6.3 / 7 | 1.33 / 3 | 2.0 / 2 |
| isaac_cellar | 4 | 12.0 | 9.3 / 11 | 1.33 / 3 | 2.0 / 2 |
| isaac_downpour | 1 | 5.7 | 4.0 / 4 | 0.73 / 2 | 4.0 / 4 |
| isaac_downpour | 2 | 5.7 | 4.7 / 5 | 0.73 / 2 | 4.0 / 4 |
| isaac_downpour | 3 | 5.7 | 4.0 / 4 | 0.92 / 2 | 2.0 / 2 |
| isaac_downpour | 4 | 8.3 | 5.3 / 6 | 0.92 / 2 | 2.0 / 2 |
| marche_britannique_coloniale | 1 | 11.7 | 4.0 / 4 | 2.69 / 4 | 2.0 / 2 |
| marche_britannique_coloniale | 2 | 13.7 | 5.0 / 5 | 2.97 / 4 | 2.0 / 2 |
| marche_britannique_coloniale | 3 | 14.3 | 7.0 / 7 | 3.42 / 6 | 1.0 / 1 |
| marche_britannique_coloniale | 4 | 13.3 | 10.3 / 14 | 3.90 / 6 | 1.0 / 1 |
| nt_desert | 1 | 6.3 | 4.0 / 4 | 1.07 / 2 | 6.0 / 6 |
| nt_desert | 2 | 8.0 | 5.0 / 5 | 1.07 / 2 | 6.0 / 6 |
| nt_desert | 3 | 13.0 | 7.0 / 7 | 1.71 / 3 | 3.0 / 3 |
| nt_desert | 4 | 19.0 | 9.3 / 12 | 1.82 / 3 | 3.0 / 3 |
| reel_acadien | 1 | 8.7 | 4.0 / 4 | 3.42 / 4 | 2.0 / 2 |
| reel_acadien | 2 | 11.3 | 5.0 / 5 | 3.69 / 4 | 2.0 / 2 |
| reel_acadien | 3 | 17.7 | 7.0 / 7 | 6.22 / 8 | 1.0 / 1 |
| reel_acadien | 4 | 22.3 | 12.7 / 13 | 8.88 / 13 | 0.0 / 0 |
| reel_britannique_coloniale | 1 | 7.7 | 4.0 / 4 | 2.75 / 4 | 2.0 / 2 |
| reel_britannique_coloniale | 2 | 10.7 | 5.0 / 5 | 3.16 / 4 | 2.0 / 2 |
| reel_britannique_coloniale | 3 | 16.0 | 7.0 / 7 | 4.83 / 8 | 1.0 / 1 |
| reel_britannique_coloniale | 4 | 17.0 | 11.0 / 12 | 5.81 / 8 | 1.0 / 1 |
| valse_britannique_coloniale | 1 | 9.0 | 4.0 / 4 | 2.03 / 3 | 2.0 / 2 |
| valse_britannique_coloniale | 2 | 12.3 | 5.0 / 5 | 2.25 / 3 | 2.0 / 2 |
| valse_britannique_coloniale | 3 | 13.3 | 7.0 / 7 | 3.38 / 6 | 1.0 / 1 |
| valse_britannique_coloniale | 4 | 19.3 | 15.7 / 21 | 3.72 / 6 | 1.0 / 1 |
| valse_quebecoise | 1 | 7.0 | 4.0 / 4 | 2.04 / 3 | 2.0 / 2 |
| valse_quebecoise | 2 | 9.7 | 5.0 / 5 | 2.21 / 3 | 2.0 / 2 |
| valse_quebecoise | 3 | 15.3 | 7.0 / 7 | 3.98 / 6 | 1.0 / 1 |
| valse_quebecoise | 4 | 16.3 | 11.3 / 16 | 4.60 / 7 | 1.0 / 1 |

Les niveaux 1 à 3 respectent le filtre **sur chaque graine** (pire cas ≤ borne : sauts 4 / 5 / 7,
attaques ≤ moitié / trois quarts de la mesure, durées ≥ minimum), pour les 15 profils. Le niveau 1 est
nettement plus facile que le niveau 4 : saut maximal 4 contre 5 à 16 (moyenne), durée minimale 2 à 6
unités contre 0 à 3, 0,65 à 3,4 attaques par mesure contre 0,77 à 8,9. Sans filtre, avec les mêmes
leviers de training, le niveau 1 sautait de 5 à 12 demi-tons et `reel_quebecois` jouait 5,5 attaques
par mesure (3,4 maintenant).

Critère de training (niveau 1 < niveau 4 strictement, marches à −5 % au plus) sur ambitus, saut et
densité : 40 lignes sur 45 le satisfont ; les 5 autres ne dépendent pas du filtre :
`gigue_quebecoise`, `marche_britannique_coloniale` et `isaac_burning` (ambitus) et `isaac_burning`
(densité) suivent la tessiture et les poids de rôles de training (mêmes marches sans filtre dans le
tableau « Avant » de T204) ; `isaac_downpour` (saut 4,0 → 4,7 → 4,0 → 5,3) varie avec les tirages.
Les neuf exclusions de training se réduisent à ces profils-là.

Contrats (`harmonium_core/tests/melody_difficulty_tests.rs`, 8 tests) : bornes tenues sur les 15
profils × 3 graines × 64 mesures (saut, attaques, durée, ambitus de chaque mesure) ; niveau facile
plus calme que le profil ; raccord de tour et de l'intro `tail` sur 98 mesures ; levée retirée ;
rejeu à l'identique après un seek (`silent_advance`) et graine stable ; `ornaments`, `chord_fit` et
`out_of_scale` ; filtre vide et bornes invalides refusées avec leur chemin.

### E11 — 64 mesures, `reel_britannique_coloniale`

`Melody` présente aux mesures 1–2, 5–6, 9–10… (32 mesures sur 64), absente aux mesures 3–4, 7–8… ;
notes des mesures jouées et **toutes les notes des autres pistes identiques** au flux sans alternance
(3 graines). `harmonium_core/tests/alternation_tests.rs` (6 tests) : phase et mesures jouées, compte
continu d'un tour à l'autre (cycle de 6 sur une forme de 32), rejeu après seek à 3, 4, 21, 36 et 50,
graine stable, table vide sans effet ni sérialisation, plafond `enable_melody` toujours actif,
alternances invalides refusées. `harmonium_host/tests/alternation_live_tests.rs` : `SetStyleProfile`
met l'alternance en place puis la retire.

### Profils sans le mécanisme

- Dumps `harmonium_determinism` : 98 / 98 identiques à `main` (`diff -r`).
- `harmonium-cli --export --record-wav`, lancé depuis le checkout principal : 32 WAV (16 profils × 2
  graines, 40 mesures) identiques octet à octet à ceux du binaire de `main`.
- `TUNING_VERSION` 26 inchangé ; aucun golden régénéré.

### Suites et mesures

`cargo test --workspace --no-fail-fast` : 108 binaires de test, tous verts ; `cargo test -p harmonium
--features odin` vert ; `cargo test -p harmonium_bevy` : 11 verts ; `cargo check -p harmonium_studio` ;
`make wasm/check` (avec et sans Odin) ; `harmonium_determinism/scripts/run.sh` : natif = wasm, 98 / 98 ;
`cargo fmt --all` ; clippy : aucun avertissement dans les fichiers ajoutés ou touchés au-delà de ceux
de `main`. Seek à la mesure 64 (`MusicComposer::deterministic_seek`, profil par défaut, sans filtre) :
26,0–26,3 ms en debug, **identique à `main`** (test `seek_to_bar_64_under_50ms`, seuil inchangé) ;
avec un filtre complet : 1,3–2,1 ms en release (0,8–1,2 sans), 31 à 139 ms en debug sur une machine
chargée par six agents (la réécriture de la ligne est une recherche, lente sans optimisation).

## Limites

- **Jonction entre deux airs d'un même set non bornée** : la ligne d'un air se raccorde à elle-même,
  pas au dernier son de l'air précédent (mesuré sur 400 mesures : saut de 11 au niveau 1 de
  `reel_quebecois` à la jonction). Les 64 premières mesures ne la franchissent pas pour les profils
  dont l'air dépasse 64 mesures (reels, gigues, valses, marche : 96 mesures ; `isaac_basement`,
  `isaac_burning`, `isaac_cellar`, `nt_desert` : 80 à 96) ; `isaac_downpour` (48 mesures) la
  franchit sans dépasser, sans garantie.
- Les bornes valent pour la ligne écrite : `ornaments` ou `chord_fit` à `true` les dépassent
  (ornements dont `Passing`, graces, doubles cordes ; notes déplacées sur les accords).
- Une bascule de mode parallèle (valence qui bouge) déplace des notes d'au plus 1 demi-ton après le
  filtre (un saut varie de 2).
- Le filtre ne peut que simplifier : `isaac_*` et `nt_desert` sont déjà clairsemés (0,8 à 1,8 attaques
  par mesure) ; leur densité ne baisse que si `min_note_units` dépasse la durée native (d'où la
  durée de référence relative). Aucune densification (training l'obtient par ses poids de rôles).
- Le niveau 3 de référence taît les ornements ; avec `ornaments = true` et `rate_scale` ×0,5 comme
  dans la table de training, les bornes ne couvrent plus les notes ajoutées.
- La levée d'intro (`pickup_eighths`) est retirée sous un filtre.
- Ambitus : il borne les attaques d'une mesure, pas l'ambitus d'une phrase.

## Manques moteur relevés

- Jonction d'airs bornée : demande de connaître la dernière hauteur de l'air précédent et la forme
  d'intro tirée (`tail`, `vamp`, pickup) avant de réécrire la ligne.
- Filtre variable en cours d'air (courbe d'intensité, ou commande) : demanderait une ligne
  recomposée mesure par mesure, que l'architecture « ligne tirée une fois par air » n'a pas.
- Ornements et ajustement aux accords sous les mêmes bornes (note ajoutée vérifiée contre le saut et
  la durée) : aujourd'hui tout ou rien.
- Alternance sur une grille de plusieurs mesures par jeu (compte par phrase, non par mesure) : les
  frontières de phrase de `phrase_ends` ne sont pas lues.

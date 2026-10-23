# Phase 3 — Phrases, fins de reprise et règles paramétrables (P3-T1 / P3-T2 / P3-T3)

## Problème et décisions

Les copies et cadences fixes rendaient les parties trop régulières. Le profil
décrit maintenant des choix pondérés par position, des séquences diatoniques,
des fins variées et des copies entre parties. Aucun identifiant de style dans
les algorithmes. Les choix sont résolus une fois par air ; reprises, tours et
invalidation émotionnelle conservent ces décisions. Un nouvel air retire ses
rôles même quand il garde la tonalité et le mode du précédent.

`CadTonic`, `CadHalf` et `CadOpen` permettent une fin tonique tenue, une
demi-cadence ou une fin ouverte. Toutes sont dans le mode publié et toutes
marquent la frontière de phrase pour une demande émotionnelle de changement
de mode. `CadFull` conserve exactement son comportement historique.

Les profils utilisent les taux par position de DS-T9, pas des probabilités
inventées à partir des indicateurs agrégés. Les poids et leur dérivation
(y compris le proxy de demi-cadence faute de classes harmoniques dans DS-T9)
sont documentés dans [profiles/README.md](../../profiles/README.md) et dans les
sept README de profils. Les mesures par position sont indépendantes : les
corrélations entre m.5 et m.6 du corpus ne sont pas modélisées.

## Fichiers touchés et contrat

- `harmonium_core/src/style_profile.rs` : schéma fixe/pondéré, validation de
  toutes les branches, sommes de poids, références antérieures et banques.
- `harmonium_core/src/trad/{plan,fiddle,generator,harmony}.rs` : résolution
  déterministe, opérations de motif, cadences et frontières émotionnelles.
- `profiles/*/profile.toml` : sept profils distribués, dont les deux profils
  britanniques ajoutés sur main pendant P3-T1 (main intégré à `885b546`).
- `harmonium_core/tests/phrase_snapshot_tests.rs` : fréquences sur 200 airs,
  contour/mode/tessiture dans les quatre modes et douze tonalités, reprises,
  tours, déterminisme, validation et nouveau tirage au prochain air.
- P3-T1 : `TUNING_VERSION = 11`, goldens régénérés dans le même commit. D2 alors :
  **22 840 notes / 1 000 mesures** ; D2 aux rôles fixes d’origine :
  **22 558 notes / 1 000 mesures, aucune différence avec l’ancien main**
  (`profile_d2_fixed.json`, source `64266ff5b27284a6cd37dfb82c500325776f1cb0`).
  Les anciens tests de forme explicite utilisent désormais ces rôles fixes ;
  les nouveaux tests couvrent le flux des profils probabilistes.

## Mesures avant/après — 20 exports par profil

Seeds 1..20, `--bars 34 --tours 2`, MIDI et vérité JSON, première passe A et B
seulement après réalignement des intros par `trad-structure`. Avant : contrat 10
à `d4037ce` pour les cinq profils initiaux, main `885b546` pour les deux nouveaux ;
le changement intermédiaire d’alias ne modifie pas les notes des cinq premiers.
Après : contrat 11 de P3-T1. Les corpus québécois sont dans `corpus/trad`,
les autres dans `corpus/acadienne` et `corpus/britannique_coloniale`.

Chaque cellule : **réel / avant → après**, en pourcentage sauf l’entropie.

| Profil | m.5–6 = m.1–2 | Paires « a b a c » | Repos tonique A:m.8 |
|---|---:|---:|---:|
| `reel_quebecois` | 53.2 / 100.0 → 42.5 | 22.5 / 95.0 → 22.5 | 22.0 / 85.0 → 25.0 |
| `gigue_quebecoise` | 52.8 / 100.0 → 40.0 | 28.9 / 92.5 → 27.5 | 16.4 / 50.0 → 15.0 |
| `valse_quebecoise` | 21.8 / 95.0 → 10.0 | 11.9 / 87.5 → 10.0 | 13.3 / 80.0 → 20.0 |
| `reel_acadien` | 43.5 / 100.0 → 45.0 | 25.2 / 90.0 → 27.5 | 23.9 / 45.0 → 15.0 |
| `marche_britannique_coloniale` | 20.5 / 100.0 → 10.0 | 5.1 / 95.0 → 10.0 | 41.2 / 75.0 → 35.0 |
| `gigue_britannique_coloniale` | 37.3 / 100.0 → 32.5 | 22.8 / 92.5 → 22.5 | 13.3 / 45.0 → 20.0 |
| `hornpipe_britannique_coloniale` | 47.2 / 100.0 → 47.5 | 32.8 / 97.5 → 27.5 | 6.8 / 25.0 → 10.0 |

| Profil | Paires « a b c d » | B:m.7 = A:m.7 exact | Entropie rythmique (bits) |
|---|---:|---:|---:|
| `reel_quebecois` | 24.2 / 0.0 → 20.0 | 20.6 / 5.0 → 10.0 | 4.601 / 3.099 → 2.780 |
| `gigue_quebecoise` | 24.5 / 0.0 → 22.5 | 30.6 / 5.0 → 10.0 | 3.275 / 2.335 → 1.857 |
| `valse_quebecoise` | 45.5 / 0.0 → 50.0 | 11.1 / 0.0 → 15.0 | 4.048 / 3.256 → 2.971 |
| `reel_acadien` | 21.0 / 0.0 → 20.0 | 35.2 / 0.0 → 40.0 | 3.661 / 2.989 → 2.842 |
| `marche_britannique_coloniale` | 48.7 / 0.0 → 32.5 | 47.1 / 5.0 → 40.0 | 5.192 / 3.634 → 3.687 |
| `gigue_britannique_coloniale` | 37.6 / 0.0 → 32.5 | 34.5 / 0.0 → 45.0 | 3.116 / 2.272 → 2.102 |
| `hornpipe_britannique_coloniale` | 27.2 / 0.0 → 27.5 | 42.1 / 5.0 → 35.0 | 4.633 / 3.176 → 2.793 |

La copie acadienne B:m.7 utilise bien `transformations["B:m7/A:m7"].rates.exact`,
pas `phrases.b_reuses_a_ending_rate` (qui porte sur la paire m.7–8).

**Limites observées, non masquées :** les structures et cadences se rapprochent
nettement du corpus, mais l’entropie rythmique baisse dans six profils.
Les fins ouvertes font moins appel aux cellules cadentielles longues ; les
banques et leur modèle de rythme ne sont pas modifiés dans cette tâche.
Les copies conjointes m.5–6 restent sous la cible pour certaines traditions,
car les tirages indépendants reproduisent les marges mesurées, pas leur
corrélation. Vingt exports donnent des pas de 5 points pour A:m.8 et B:m.7,
de 2,5 points pour les indicateurs sur les 40 parties ; aucune conclusion
de qualité musicale ou validation à l’oreille n’est déduite de ces chiffres.

### Reproduction du relevé

```sh
cargo build -p harmonium-cli
# Pour chaque profil et seed 1..20 (exemple) :
target/debug/harmonium-cli --export --style reel_quebecois --seed 1 \
  --bars 34 --tours 2 --record-midi /tmp/phrase/reel_quebecois/seed-01.mid \
  --record-truth /tmp/phrase/reel_quebecois/seed-01.truth.json
cd /Users/william.quintal/Project/bascanada/harmonium_lab
.venv/bin/harmonium-lab trad-structure --corpus corpus/trad --type reel \
  --generated /tmp/phrase/reel_quebecois --output /tmp/phrase/reports \
  --references /tmp/phrase/references
```

Les 280 exports MIDI/vérité et les rapports complets de cette exécution sont
dans `/tmp/harmonium-phrase-metrics/{before,after}` ; ils ne sont pas ajoutés
au dépôt. Les tableaux ci-dessus sont le relevé durable du résultat.

## Vérification P3-T1

- Les six tests phrase passent, dont 200 airs par profil pour les marges copie
  m.5 et cadence tonique m.8, chacune dans ±10 points des poids configurés.
- Export CLI supplémentaire avec automation neutre (`--bars 130 --tours 2`) :
  **130 snapshots, deux airs en ré ionien**, 127 mesures de corps vérifiées
  identiques à leurs reprises disponibles. A:m.5 ne copie pas m.1 dans le premier
  air, mais la copie dans le second : les choix se renouvellent malgré la même
  tonalité. Rendu offline terminé (136,01 s audio en 4,94 s), sans panne RT.
- `cargo test --workspace` : **362 tests passent, 37 suites, 2 ignorés** ;
  `cargo fmt --all --check` passe.
- La banque GM complète et les samples de pieds ne sont pas installés dans ce
  worktree : l'export charge `assets/test.sf2` et signale les programmes absents.
  Les MIDI et snapshots sont mesurés ; aucune validation d'écoute instrumentale
  n'est revendiquée.
- `cargo clippy --workspace --lib --bins --message-format=json` : code retour 0,
  **165 avertissements sur main et 165 après, aucun nouveau** (comparaison
  fichier/code/message hors tests). `-D warnings` ne passe pas sur le socle
  existant, notamment `log.rs` et les exports MusicXML/MIDI ; ces fichiers ne
  sont pas modifiés par P3-T1.
- Aucun ajout d’`unwrap()`/`expect()` hors tests, aucun changement dans le
  callback audio ; les opérations de phrase s’exécutent sur le générateur
  du thread principal. Ni push ni intégration dans main.

## P3-T2 — Fins de reprise

### Défaut et décision

P3-T1 imposait une même dernière mesure à toutes les passes. DS-T9 ne mesurait
que les premières passes : une fin ouverte destinée à reprendre A ou B devenait
aussi la conclusion de B2. `[[form.parts]] second_ending` décrit désormais une
variante de la seule dernière mesure, tirée une fois par partie et par air.
La dernière passe de chaque partie l'utilise (A2/B2 en AABB, A3 si trois passes),
les tours la relisent. Mélodie et harmonie ont un matériel distinct pour cette
variante ; les autres cellules, accords et percussions ne sont pas retirés.
L'outro `end_after_tour` garde sa priorité et résout sur la tonique.

Les secondes fins utilisent les rôles génériques `CadTonic`, `CadFifth` et
`CadOther`, aux effectifs mesurés ci-dessous. `CadFifth` garde le rythme d'une
cellule de demi-cadence et termine sur 5 ; `CadOther` garde le rythme d'une
cellule libre et évite 1/5. Les choix acceptent aussi les opérations de motif
existantes. Une seconde fin sans reprise, une auto-référence ou une référence
en avant est rejetée, ainsi que toute branche ou banque invalide.

### Mesure réelle : note écrite, pas levée ajoutée

Commande exécutée dans l'environnement Python du labo, sans modifier ce dépôt
ni les transcriptions :

```sh
/Users/william.quintal/Project/bascanada/harmonium_lab/.venv/bin/python \
  specs/011-phase3-phrase/measure_endings.py \
  --lab-root /Users/william.quintal/Project/bascanada/harmonium_lab \
  --output /tmp/harmonium-real-endings-final.json
```

Le script réutilise `trad.form.analyze_form`. Ce parseur recolle une levée aux
mesures finales partielles : prendre simplement la dernière note de `plan.bars`
compterait cette levée comme une conclusion. Un hook Python temporaire, restauré
après chaque appel, capture à son retour la correspondance de sources déjà
calculée (`segments`, `passes`, `kept`) ; il lit la dernière note du segment
écrit d'origine, **avant** cette complétion. Il échoue explicitement si cette
correspondance privée change ; aucun second parseur de reprises n'est ajouté.
Voix principale et note aiguë des doubles-cordes suivent les conventions du labo.
Classification par classe de hauteur : tonique = 0, quinte = +7 demi-tons,
autre = tout le reste, sans rabattement chromatique.
La tonique est celle du `K:` initial fournie par le parseur ; les modulations
et changements de métrique internes ne sont pas réinterprétés par ce script.

Une occasion = première et dernière occurrence de la même lettre du plan de
forme, avec ou sans voltas explicites ; les reprises écrites reconnues par le
labo sont comprises. Les formes croches/non positionnables sont exclues ; les
parties jouées une seule fois restent comptées séparément dans le JSON, pas
mélangées à cette comparaison appariée. Aucun corpus de hornpipes québécois
n'est présent : les **14** combinaisons présentes sont toutes analysées.

Les colonnes de fins donnent **tonique / quinte / autre**, en effectifs :

| Tradition | Type | Parties appariées | Première fin | Fin finale | Airs irréguliers exclus |
|---|---|---:|---:|---:|---:|
| quebecoise | reel | 851 | 430 / 108 / 313 | 529 / 63 / 259 | 120 |
| quebecoise | jig | 144 | 74 / 23 / 47 | 93 / 16 / 35 | 11 |
| quebecoise | waltz | 77 | 36 / 10 / 31 | 48 / 7 / 22 | 19 |
| quebecoise | march | 21 | 8 / 3 / 10 | 9 / 3 / 9 | 5 |
| acadienne | reel | 221 | 115 / 20 / 86 | 127 / 13 / 81 | 25 |
| acadienne | jig | 97 | 37 / 8 / 52 | 50 / 8 / 39 | 4 |
| acadienne | waltz | 5 | 3 / 1 / 1 | 3 / 1 / 1 | 2 |
| acadienne | march | 12 | 11 / 0 / 1 | 11 / 0 / 1 | 4 |
| acadienne | hornpipe | 38 | 19 / 6 / 13 | 23 / 3 / 12 | 9 |
| britannique_coloniale | reel | 564 | 414 / 48 / 102 | 437 / 41 / 86 | 63 |
| britannique_coloniale | jig | 438 | 333 / 31 / 74 | 346 / 33 / 59 | 59 |
| britannique_coloniale | waltz | 28 | 8 / 11 / 9 | 15 / 7 / 6 | 8 |
| britannique_coloniale | march | 27 | 23 / 2 / 2 | 23 / 2 / 2 | 6 |
| britannique_coloniale | hornpipe | 73 | 61 / 3 / 9 | 62 / 4 / 7 | 14 |

Ces chiffres **ne confirment pas une tonique finale proche de 100 % dans tous
les styles** : le reel, la gigue et la valse québécois se situent autour de
62–65 % toutes parties confondues, alors que plusieurs types britanniques
atteignent 79–85 %. Les secondes fins ne sont donc pas forcées artificiellement
à 100 %. Le très petit nombre d'occasions de certains types (par exemple cinq
parties de valse acadienne) interdit une généralisation à toute la tradition.

### Poids des sept profils

Pour coller à leurs parties de huit mesures, les profils utilisent uniquement
les paires dont première et dernière passes ont huit mesures, séparément pour
A et B. Les poids sont exactement les effectifs `eight_A:second_written` et
`eight_B:second_written`, sans arrondi. Les premières fins restent celles de
P3-T1, calibrées sur le **repos tonique** DS-T9, pas sur cette nouvelle mesure
de dernière note écrite ; ces deux indicateurs ne doivent pas être confondus.

| Profil | A2 : tonique / quinte / autre | B2 : tonique / quinte / autre | Cible tonique A2 | Cible tonique B2 |
|---|---:|---:|---:|---:|
| `reel_quebecois` | 169 / 19 / 99 | 182 / 18 / 78 | 58.89 % | 65.47 % |
| `gigue_quebecoise` | 41 / 7 / 19 | 44 / 5 / 11 | 61.19 % | 73.33 % |
| `valse_quebecoise` | 9 / 2 / 7 | 16 / 2 / 1 | 50.00 % | 84.21 % |
| `reel_acadien` | 45 / 5 / 22 | 40 / 2 / 20 | 62.50 % | 64.52 % |
| `marche_britannique_coloniale` | 11 / 2 / 1 | 9 / 0 / 1 | 78.57 % | 90.00 % |
| `gigue_britannique_coloniale` | 205 / 15 / 30 | 100 / 12 / 21 | 82.00 % | 75.19 % |
| `hornpipe_britannique_coloniale` | 45 / 3 / 3 | 11 / 0 / 1 | 88.24 % | 91.67 % |

### Moteur corrigé : 12 exports par profil

Seeds 1..12, tour 1, `--bars 34 --tours 2`, automation neutre constante
(arousal 0,5 ; valence 0 ; density 0,5 ; tension 0), MIDI et vérité JSON.
L'automation conserve les snapshots complets et laisse l'air continuer : B2
mesure la seconde fin musicale, **pas** un outro terminal imposé.
Tonalité lue à chaque snapshot, dernière attaque de mélodie de m.8, mêmes
classes tonique/quinte/autre que le relevé réel.

| Profil | Partie / passe | Tonique | Quinte | Autre |
|---|---|---:|---:|---:|
| `reel_quebecois` | A1 | 4 | 1 | 7 |
| `reel_quebecois` | A2 | 9 | 1 | 2 |
| `reel_quebecois` | B1 | 3 | 1 | 8 |
| `reel_quebecois` | B2 | 7 | 0 | 5 |
| `gigue_quebecoise` | A1 | 2 | 1 | 9 |
| `gigue_quebecoise` | A2 | 9 | 2 | 1 |
| `gigue_quebecoise` | B1 | 4 | 1 | 7 |
| `gigue_quebecoise` | B2 | 7 | 0 | 5 |
| `valse_quebecoise` | A1 | 3 | 3 | 6 |
| `valse_quebecoise` | A2 | 5 | 1 | 6 |
| `valse_quebecoise` | B1 | 4 | 2 | 6 |
| `valse_quebecoise` | B2 | 11 | 0 | 1 |

Sur ces exports, les **504 paires de mesures non finales** (12 × 3 × 2 × 7)
ont des hauteurs, attaques, durées et vélocités de mélodie identiques entre
passes. Les fins diffèrent selon le tirage. Douze airs donnent des pas de
8,33 points : ce tableau illustre le rendu, il ne remplace pas la vérification
de fréquence sur 200 airs. Les 36 sorties et journaux sont conservés dans
`/tmp/harmonium-second-ending-exports`, dont `summary.json`.
La banque GM/pieds complète n'étant pas installée, le CLI emploie `assets/test.sf2`
et avertit des programmes absents ; aucune validation instrumentale à l'oreille
n'est revendiquée.

### Baisse d'entropie : attribution, pas perte de durées

Analyse des mêmes 20 exports avant/après P3-T1 avec `engine_material` et
`rhythm_cell` du labo, sur premières passes, gate/swing neutralisés comme DS-T9.
On sépare m.4/m.8 (80 mesures par lot) des six autres positions (240 mesures),
puis on remplace **uniquement** la distribution rythmique m.4/m.8 après P3-T1
par celle d'avant pour calculer une entropie contrefactuelle descriptive.
Ce n'est ni un nouveau rendu ni une estimation causale indépendante des airs.

| Profil | Croches continues m.4/m.8 avant → après | Croches continues autres positions avant → après | Entropie après avec anciennes m.4/m.8 (bits) |
|---|---:|---:|---:|
| `reel_quebecois` | 25.00 → 56.25 % | 44.58 → 45.83 % | 3.035 |
| `gigue_quebecoise` | 13.75 → 63.75 % | 55.42 → 57.08 % | 2.338 |
| `valse_quebecoise` | 6.25 → 18.75 % | 36.25 → 35.42 % | 3.209 |
| `reel_acadien` | 26.25 → 50.00 % | 37.08 → 32.08 % | 3.003 |
| `marche_britannique_coloniale` | 0.00 → 2.50 % | 4.17 → 2.50 % | 3.646 |
| `gigue_britannique_coloniale` | 35.00 → 61.25 % | 58.33 → 53.33 % | 2.390 |
| `hornpipe_britannique_coloniale` | 7.50 → 41.25 % | 51.25 → 55.00 % | 3.070 |

En gigue québécoise, cette seule substitution remonte l'entropie de 1,857 à
2,338 bits, contre 2,335 avant P3-T1 : les autres positions n'expliquent donc
pas la baisse observée. Le même mécanisme domine dans les autres profils :
remplacer les rôles `CadHalf`/`CadFull` obligatoires de m.4/m.8 par beaucoup de
`CadOpen` sélectionne la banque libre `Run`/`Arp`/`Turn`, plus riche en croches
continues et moins riche en longues cellules cadentielles.

`CopyOf` et `SequenceOf` recopient intentionnellement les rythmes (contrat de
motif), mais ne suppriment ni n'écourtent les tenues. Ce n'est pas un défaut de
copie/transposition à corriger en altérant les rythmes des motifs. La véritable
erreur de couche phrase était de reconduire cette même fin ouverte à la reprise
finale : P3-T2 la corrige avec un matériel cadentiel distinct. Les premières
passes restent identiques au modèle P3-T1 et leur entropie n'est pas présentée
comme corrigée ; aucun changement opportuniste des banques ou des poids de
rythme n'est ajouté pour faire monter une métrique.

### Contrat et vérification P3-T2

- `TUNING_VERSION = 12`, goldens régénérés dans le même commit. D2 courant :
  **23 212 notes / 1 000 mesures** ; les **22 558** notes historiques restent
  identiques en omettant `second_ending` et en gardant les rôles fixes d'origine.
- `profile_d2_main.json` est écrit avec `serde_json::to_string`, **344 264 octets,
  une ligne**, au lieu de la version pretty de P3-T1. Le format compact ne change
  pas les assertions de contenu JSON.
- Six tests de contrat supplémentaires : catégories finales dans ±10 points
  des poids sur **200 airs par profil**, identité des sept autres mesures sur
  toutes les pistes et de leurs accords, stabilité des tours, déterminisme,
  forme entrelacée à trois passes, copie de motif en seconde fin, références
  invalides, round-trip du profil et priorité de l'outro.
- Les tests P3-T1 restent verts. Les anciennes assertions d'automation excluent
  uniquement la volta de la comparaison entre passes ; les fins sont encore
  comparées entre tours, et les tests de dynamique comparent aussi leur contour
  sous intensité basse/haute.
- `cargo test --workspace` : **368 tests passent, 38 suites, 2 ignorés** ;
  `cargo fmt --all --check` passe. `main` est intégré, sans nouveau commit à
  absorber lors de la vérification finale.
- Clippy hors tests : **165 avertissements préexistants, aucun nouveau** ;
  `cargo clippy --workspace --lib --bins --message-format=json` retourne 0.
- Aucun `unwrap()`/`expect()` ajouté hors tests, aucun changement du callback
  audio ni du chemin RT ; tout le matériel de seconde fin est construit sur
  le thread principal. Aucun push ni fusion dans main.

## Écoute du propriétaire (2026-10-23)

Paires `demos/phrase_avant_apres/reel_seed{3,7}_{AVANT,APRES}.wav` : « ça sonne plus vrai comme
air, et je ne trouve pas qu'on perde vraiment rien ». Couche phrase retenue.

Exigence : **toute règle de structure doit être paramétrable** dans le profil (reprise de motif,
variation aux reprises, etc.). Suite : P3-T3 (audit de ce qui reste codé en dur, rythme par
position, guide des règles).

## P3-T3 — Toute règle de structure dans le profil

### Audit règle → donnée

Localisations **avant modification, commit `9323ef1`** : elles restent stables
pour retrouver les décisions retirées, même si les ajouts déplacent les lignes.
Le schéma et ses bornes sont décrits dans « Règles de phrase » du guide des profils.

| Règle / source avant P3-T3 | Décision après audit |
|---|---|
| Reprises et tours relisent une seule cellule ; `trad/plan.rs:136-150`, `trad/fiddle.rs:134-142` | `form.repeat_variation = 0..1`, `tour_variation = 0..1`, menu `variation_operations = ["varied_end","repeated_notes","octave"]`. Défauts zéro, aucun tirage nouveau si désactivés. |
| Choix une fois par air ; `trad/plan.rs:104-118` | `form.resolve = "per_tune"` (défaut) ou `"per_pass"`. Cette dernière portée retire rôles et cellules pour chaque passe jouée, tours compris ; tout est précalculé au début de l'air. |
| Première mesure toujours Open ; `profiles/reel_quebecois/profile.toml:32,46` | **Déjà une donnée**, pas un verrou du générateur : elle accepte un choix Open/Free/cadence pondéré. Les sept profils gardent Open ; un contrat vérifie le choix dès m.1. |
| Longueur de partie et ordre AABB ; `style_profile.rs:234-269` | Déjà `roles.len()` et `form.sequence` ; aucun 8 obligatoire. Contrat supplémentaire : partie de six mesures, groupement 3+3. |
| Cadences de percussion à m.4 et fin de partie ; `trad/generator.rs:755-756` | `parts.phrase_ends`, explicite `[4,8]` dans les sept profils. Vide/omis : positions portant un rôle cadentiel. Aucun test `bar == 4` restant. |
| Frontière de changement de mode après rôle cadentiel ; `trad/generator.rs:352-364` | Même `phrase_ends`, avec `dynamics.mode_shift_boundary` déjà configurable ; le début d'une nouvelle partie reste une frontière. Les accords restent gouvernés par les rôles, pas par ce regroupement. |
| Continuité entre parties ; `trad/fiddle.rs:63-74,305-343` | `parts.link_previous = true` par défaut, `false` pour repartir indépendamment. `CopyOfPart` reste un lien explicite vers la première passe de la source. |
| Exclusion de la cellule libre précédente ; `trad/fiddle.rs:373-376` | `form.avoid_adjacent = true` par défaut ; `false` autorise la répétition immédiate. |
| Maximum huit nouveaux tirages d'ouverture entre airs ; `trad/generator.rs:1124-1135` | `form.opening_retries = 8`, réglable 0..10000 ; zéro permet de garder le même incipit. |
| Même couleur / relatif à 50/50 ; `trad/generator.rs:1342` | `form.next_same_weight`, `next_relative_weight`, défauts 1/1. Le déplacement relatif ±3 demi-tons reste la définition diatonique de l'opération, pas un choix de style. |
| Première moitié conservée par VariedEndOf ; `trad/fiddle.rs:468` | `form.varied_end_keep = 0.5`, strictement 0..1, raccord borné à l'intérieur de la mesure. S'applique aussi aux variations de reprise. |
| CadTonic : recherche en seconde moitié et tenue minimale de deux unités ; `trad/fiddle.rs:500-507` | `form.cadence_search_from = 0.5`, `cadence_min_units = 2`. La cible tonique reste la définition du rôle, sa durée n'est plus imposée par l'algorithme. |
| Fenêtre stricte de contour ±4 demi-tons ; `trad/fiddle.rs:555-576` | `form.contour_window_semitones = 4.0`. |
| Tolérance de registre six demi-tons ; `trad/fiddle.rs:892-896` | `form.register_tolerance_semitones = 6.0`. Les centres par partie restent `melody_center_offset`. |
| Sauts de quinte aux entrées et aux retours ; `trad/fiddle.rs:908-921` | `form.max_leap_degrees = 4` pas diatoniques ; `close_part_loop = true`. La proximité d'un CopyOf(1) est évaluée depuis les rôles réellement résolus, pas depuis une position m.4 codée. |
| Distance de continuité et nombre d'essais ; `trad/fiddle.rs:596,600,905-906` | Déjà `melody.params.max_continuity_step_distance`, `continuity_max_retries` ; pas de deuxième seuil parallèle. |
| Dernière mesure protégée des ornements de densité ; `trad/generator.rs:949-951` | `parts.ornament_protected_bars` : omission protège la fin, liste explicite pour d'autres positions, `[]` pour aucune. |
| Tenue d'au moins trois pas avant ornement de densité ; `trad/generator.rs:961` | `form.ornament_min_steps = 3`. La note de passage dure un pas, unité minimale de la grille ; la note source conserve au moins un pas. La quantité reste la courbe existante `dynamics.density_passing`. |
| Rythme général aux positions cadentielles libres ; `trad/fiddle.rs:601-645` | `parts.rhythm = [{bar, patterns=[{durations,weight}]}]`, durées en unités de cellule ; tirage distinct du rôle. Copie/séquence/fin variée ont priorité, CadFull/CadTonic gardent leurs tenues. |
| Qualité I/V codée selon le mode même si le profil dit autre chose ; `trad/harmony.rs:118-128` | Réutilise désormais **les qualités de `harmony.candidates`** ; I/V sont déjà exigés par la validation. Les sept profils déclarent les mêmes qualités qu'avant. |
| V–I ou V imposé par un rôle ; `trad/harmony.rs:183-193` | Invariant **sémantique des rôles choisis** CadTonic/CadFull et CadHalf/CadFifth. Pour ne pas imposer cette harmonie, choisir CadOpen/Free ; nombre/positions d'accords, candidats, poids de vote et priorités sont déjà en données. |
| Seconde fin à la dernière passe ; `trad/plan.rs:130-133` | Déjà `parts.second_ending` optionnel et `sequence` : « seconde fin » signifie ici dernière passe de la partie. Aucun présupposé de deux passes ; la forme entrelacée à trois passes reste couverte. |
| Invalidation émotionnelle ne retire pas les choix ; `trad/generator.rs:379-388` | Invariant de déterminisme/relecture : elle relit le matériel préconstruit. La portée per_pass n'est pas un nouveau hasard au moment du callback ou de l'invalidation. |
| Humanisation de pieds réutilisée par position ; `trad/generator.rs:752-754,1169-1174` | Choix expressif, pas nouvelle forme : amplitude déjà `percussion.velocity_humanize`, motifs en données. Les variations de phrase portent sur la mélodie ; aucune déstabilisation opportuniste des pieds. |
| Intro/tours/entrées/outro ; `trad/generator.rs:405-514,1047-1055,1205-1244` | Longueurs, nombre de tours, levée et formes d'intro sont déjà `arrangement`, `intro.forms[].bars`, `pickup_eighths`, `each_tune`. `end_after_tour` est une commande explicite de terminaison qui conserve sa priorité tonique. |
| Réinitialisation des voicings à chaque passe ; `trad/generator.rs:992-1026` | Déjà calculée depuis `form.position`, pas par division par huit ; matériel harmonique indexé sur les nouvelles passes/tours quand nécessaire. |

Les autres nombres des quatre modules ne sont pas des règles de style cachées :

- **7 degrés, 12 demi-tons, MIDI 0..127**, ±octaves et absence du degré zéro :
  invariants de représentation musicale ; validation et repli de tessiture.
- Recherche d'octave `-4..4` pour une séquence, `-2..2` pour le registre, poids
  de priorité `16` devant la distance d'octave (`fiddle.rs:434-455,839-843`) :
  enveloppes et ordre du solveur de tessiture, pas probabilité/phrase. Une
  pénalité de dépassement domine toute distance d'octave candidate (au plus 4).
  Le seuil stylistique de contour et la tolérance de registre sont, eux, sortis.
- Milieu des centres de registre (`fiddle.rs:824-828`) : définition de la
  frontière géométrique ; centres et tolérance sont configurables.
- Chiffres 0/1 de boucles, indices 1-based, compteurs de passes et limites de
  capacité validées (parties, choix, mesures, tours) : invariants d'indexation
  et de représentation bornée, pas décisions de répétition.
- Une levée ascendante emploie les degrés voisins du mode ; son nombre de notes
  est une donnée. L'échelle temporaire de quatre octaves (`generator.rs:1299-1313`)
  est une enveloppe de calcul pour les toniques MIDI, pas quatre phrases.
- Vélocités d'intro/outro, numéros de timbre/percussion et durées d'articulation
  ne décident pas de la structure : non modifiés dans cette tâche. Aucun nom
  de style n'est introduit dans les algorithmes.

### Rythme : source et décision de réglage

Le labo publie `real.rhythm.cells`, mais pas de motifs de durée par position.
Les préférences de m.4/8 sont donc une **distribution conditionnelle de corpus**,
pas une fausse mesure positionnelle : mesure entière, durées entières positives
représentables par les cellules actuelles, au moins deux durées distinctes.
Les poids sont les effectifs exacts conservés ; pas de poids inventé pour
atteindre l'entropie. Les rythmes isochrones et tenues uniques restent dans
les banques et cadences fermées. Le premier essai non conditionné conservait
trop de croches continues ; le conditionnement « tenue » seul gardait trop de
pulsations identiques en valse. Le choix final privilégie donc les durées
contrastées dans les positions qui avaient perdu leur variété. Aucune
préférence n'est ajoutée aux six autres positions ; banques et poids de rôles
restent inchangés. Les tirages supplémentaires peuvent déplacer les choix
de cellules suivants dans le flux RNG, d'où la version de contrat 13.

Sources : `/Users/william.quintal/Project/bascanada/harmonium_lab/references/structure/`
`quebecoise-{reel,jig,waltz}.json`, `acadienne-reel.json`,
`britannique_coloniale-{march,jig,hornpipe}.json`. Les fractions, triolets et
silences non représentables ne sont pas arrondis ni supprimés du corpus ;
ils sont exclus de cette projection explicitement limitée à la grille actuelle.

### Mesure appariée : 200 airs par profil

Seeds **1..200**, un premier tour AABB, premières passes A et B uniquement,
arrangement plat pour ne pas confondre le résultat avec les intros. Trois
configurations du même générateur : rôles fixes pré-P3-T1 sans seconde fin et
sans préférence rythmique ; profil P3-T2 sans préférence ; profil P3-T3 courant.
La preuve D2 fixe établit séparément l'identité du chemin pré-P3-T1 historique.

Les `MeasureSnapshot` fournissent hauteurs/attaques et mode/tonique publiés.
Les durées vont jusqu'à l'attaque suivante ou la barre, comme `engine_material`
du labo : gate et swing exclus. Les fonctions inchangées `Material` et
`analyze_materials` de DS-T9 calculent entropie, motifs et cadences. **4 200 airs,
67 200 mesures distinctes** sur l'ensemble des trois configurations ; chaque
cellule d'entropie porte sur 3 200 mesures. Ce relevé apparié à 200 seeds ne
remplace pas les anciennes valeurs sur 20 exports par une comparaison de lots
différents.

Entropie Shannon, bits par mesure :

| Profil | Avant P3-T1 | Avant P3-T3 | Après P3-T3 |
|---|---:|---:|---:|
| `reel_quebecois` | 2.895 | 2.706 | 3.316 |
| `gigue_quebecoise` | 2.343 | 1.931 | 2.440 |
| `valse_quebecoise` | 3.225 | 2.956 | 3.265 |
| `reel_acadien` | 2.970 | 2.697 | 3.065 |
| `marche_britannique_coloniale` | 3.769 | 3.779 | 4.001 |
| `gigue_britannique_coloniale` | 2.291 | 1.958 | 2.403 |
| `hornpipe_britannique_coloniale` | 3.146 | 3.013 | 3.445 |

L'objectif après ≥ avant P3-T1 est atteint dans les sept profils. Un contrat
sur les snapshots compare aussi cette inégalité sur les mêmes 200 seeds.

Fréquences structurelles en %, **avant P3-T3 → après P3-T3** :

| Profil | m.5–6 = m.1–2 | « a b a c » | « a b c d » | A:m.8 repos tonique | B:m.7 = A:m.7 |
|---|---:|---:|---:|---:|---:|
| `reel_quebecois` | 46.25 → 46.25 | 28.00 → 28.00 | 17.75 → 16.25 | 17.00 → 16.50 | 20.50 → 20.50 |
| `gigue_quebecoise` | 36.75 → 36.75 | 23.00 → 23.00 | 24.00 → 23.50 | 12.00 → 12.00 | 33.00 → 33.50 |
| `valse_quebecoise` | 13.00 → 14.00 | 11.00 → 12.00 | 43.50 → 43.50 | 12.50 → 12.50 | 10.50 → 9.50 |
| `reel_acadien` | 35.50 → 36.00 | 21.50 → 21.75 | 18.25 → 18.25 | 9.00 → 9.00 | 33.00 → 33.50 |
| `marche_britannique_coloniale` | 9.25 → 10.00 | 6.00 → 6.75 | 39.50 → 39.25 | 26.50 → 26.00 | 45.50 → 46.00 |
| `gigue_britannique_coloniale` | 26.50 → 26.75 | 20.25 → 20.50 | 28.50 → 27.50 | 9.50 → 10.50 | 37.50 → 36.50 |
| `hornpipe_britannique_coloniale` | 39.50 → 39.75 | 29.25 → 29.25 | 23.00 → 22.50 | 5.00 → 5.00 | 39.00 → 39.50 |

Écart maximal des cinq indicateurs : **1,5 point**, sous la tolérance de dix
points. Les marges CopyOf/CadTonic et les secondes fins restent aussi testées
sur 200 airs contre leurs poids configurés. Les taux de repos tonique DS-T9
(tenue d'au moins un temps métrique) ne sont pas la fréquence de la seule
classe finale tonique ni celle des rôles CadTonic.

### Reproduction

```sh
cargo run -p harmonium_core --example phrase_metrics -- /tmp/phrase-params.json
/Users/william.quintal/Project/bascanada/harmonium_lab/.venv/bin/python \
  specs/011-phase3-phrase/measure_parameters.py \
  --lab-root /Users/william.quintal/Project/bascanada/harmonium_lab \
  --snapshots /tmp/phrase-params.json --output /tmp/phrase-params-metrics.json
```

Le script échoue si l'entropie passe sous le témoin pré-phrase ou si l'un des
cinq indicateurs structurels dévie de plus de dix points. Les exemples du
profil, valeurs mécaniques/libres et trois recettes sont dans
[le guide humain](../../profiles/README.md#règles-de-phrase).

### Régression d'intro révélée et corrigée

Les nouveaux rythmes ont exposé une fin Tail attaquant après le début de sa
levée : la piste ajoutait les notes de levée en fin de vecteur, alors que le
playhead utilise un curseur d'attaques croissantes. `handoff_tests` a échoué
à **bar 2, step 12**, levée MIDI 61 absente au rendu mais présente au snapshot.
Après ajout de la levée, le producteur trie sans allocation par `(start_step,id)`.
Aucun changement au callback, aucune note supprimée pour masquer l'erreur.
Le même scénario de handoff passe après correction ; un contrat Tail + levée
vérifie aussi ordre, positions 12/14 et note d'arrivée sur le début de A.

### Contrat, smoke et références P3-T3

- `TUNING_VERSION = 13`, goldens courants régénérés avec ce changement.
  D2 courant : **23 178 notes / 1 000 mesures** ; D2 historique à rôles fixes,
  sans seconde fin ni préférence rythmique : **22 558 notes**, zéro différence.
  **`profile_d2_fixed.json` n'est pas modifié.**
- Les anciens goldens gigue/valse à rôles fixes conservent eux aussi tous les
  événements (piste, hauteur, attaque, durée, vélocité). Leur comparaison
  canonise seulement l'ordre des événements : une levée Tail historiquement
  non triée ne doit pas imposer à nouveau un ordre illisible par le playhead.
  Le nouveau contrat d'intro vérifie séparément l'ordre chronologique réel.
- Smoke de l'API, seed 7, sans seconde fin : `repeat_variation = 0` donne
  **0/16** mesures de reprise différentes ; `= 1` avec octave donne **14/16**.
  `tour_variation = 0` donne **0/32** différences ; `= 1` donne **28/32**.
  Les quatre cibles et tenues cadentielles comparées sont conservées. Une
  octave impossible dans la tessiture est correctement sautée, pas forcée.
  Le programme de smoke temporaire a été supprimé après exécution.
- CLI offline seed 42, reel, 34 mesures, deux tours : **35,48 s audio en
  0,80 s**, MIDI et vérité JSON exportés dans `/tmp/harmonium-p3t3-smoke-final.*`.
  La banque GM complète et les samples de pieds ne sont pas installés :
  rendu avec `assets/test.sf2` et avertissements explicites, **pas de nouvelle
  validation instrumentale à l'oreille revendiquée**.
- Clippy hors tests : **165 avertissements avant, 165 après, aucun nouveau**,
  comparaison des multisets fichier/code/message ; commande
  `cargo clippy --workspace --lib --bins --message-format=json`, code retour 0.
- Aucun `unwrap()`/`expect()` dans les nouveaux chemins de production, aucun
  changement du callback. Le matériel et les variantes restent construits
  sur le thread principal ; le tri d'intro est sans allocation.
- Guide humain : chaque paramètre, valeurs mécanique/libre et trois recettes.
  L'exemple `phrase_metrics` et `measure_parameters.py` sont les outils durables
  de reproduction, pas une seconde implémentation des métriques du labo.
- Vérification finale : **`cargo test --workspace` : 386 tests passent,
  39 suites, 2 ignorés** ; `cargo fmt --all` puis `cargo fmt --all --check`
  passent. Les dix-sept nouveaux contrats de paramètres et le contrat d'intro
  portent sur les snapshots, pas sur l'état privé du générateur. Ils couvrent
  aussi les longueurs de cellule impaires (demi-mesure tronquée, comme avant)
  et la tenue tonique de CadFull précédant une levée, conservée aux variations.
- Branche issue de `main` (`9323ef1`) ; changement livré dans un commit
  atomique en français, sans fusion ni push.


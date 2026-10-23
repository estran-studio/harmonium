# Phase 3 — Couche phrase probabiliste et fins de reprise (P3-T1 / P3-T2)

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

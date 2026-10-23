# Phase 3 — Couche phrase probabiliste (P3-T1)

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
- `TUNING_VERSION = 11` et goldens régénérés dans le même commit. D2 courant :
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

## Vérification

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

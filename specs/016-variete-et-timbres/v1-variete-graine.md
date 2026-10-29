# V1 — Variété par graine

## Décisions

- **Un seul schéma d'accords** : `kind = "chord_loop"` expose `loops = [{ weight, chords }]`. `weight` vaut 1 par défaut, doit être positif ; listes et accords non vides. L'ancien `chords` directement sur la partie est refusé. Chaque alternative couvre exactement la longueur de la partie. Les anciennes boucles courtes de `nt_desert.B`, Burning/C2 et de la fixture gigue sont développées explicitement, sans changer leur séquence initiale.
- **Identité par air et partie** : une graine d'alternatives est tirée avant la mélodie, seulement si l'harmonie ou la batterie possède plusieurs candidats. Deux tirages d'entiers `u64` bornés à 32 bits la composent ; aucun tirage supplémentaire pour les profils entièrement à candidat unique. Une nouvelle graine est tirée à chaque air, même lorsqu'une identité mélodique revient par la mémoire de set.
- **Flux séparés** : `LoopHarmony` sélectionne par partie avec `ChaCha8Rng`, `ornaments::mix` et le sel `0x4841_524D`. `FeetPatternPlan` sélectionne par partie et densité avec le sel `0x4645_4554`. Cumuls de poids et tirages en `u64`, indépendants de la largeur du pointeur. Chaque sélection à candidat unique retourne avant toute création/tirage de RNG. Les reprises et tours relisent les mêmes choix ; une bascule de mode reconstruit l'harmonie avec la même graine et réinterprète seulement les degrés modaux.
- **Guidage mélodique conservé** : le choix harmonique précède la construction de la mélodie ; sa réalisation existante rapproche les notes des sons de la boucle choisie, avant publication des snapshots.
- **Un seul rendu de batterie** : `feet_notes(table, pattern, steps, humanize)` reçoit un pattern explicite du plan. Les accès implicites au premier candidat et `feet_notes_with_cadence` disparaissent. Les intros percussives utilisent le choix de la première partie réelle de la forme, pas une partie 0 implicite. Les patterns euclidiens participent au même mécanisme.
- **Cadences inchangées** : `cadence_patterns` reste unique par densité et remplace la sélection ordinaire aux fins de phrase ; aucune banque cadentielle pondérée. Revenir à une densité retrouve le choix déjà fixé pour cette densité.
- Aucun nom de style dans l'algorithme ; la fixture `gigue_bourdon` exerce les deux banques hors Isaac. Aucun changement au callback audio, aux timbres Odin/mix, au studio ni à `TUNING_VERSION` (**21**).

## Données musicales

Sources : `harmonium_lab/references/isaac-basement/README.md`, §3 et §6, analyses de Ridiculon (*Diptera Sonata*, *Periculum*, *Fundamentum*, *River of Despair*). Alternatives composées à partir des structures et caractères analysés ; aucun nouvel enregistrement ni transcription sous droits dans le dépôt.

| Profil | Parties avec deux boucles | Parties avec deux motifs de basse | Patterns aux densités 1 et 2 | Cellules de chœur |
|---|---|---|---|---|
| `isaac_basement` | A2, D, Pont | A1, A2, D, Pont | Deux par densité | 8, dont 3 nouvelles |
| `isaac_cellar` | A2, Pont | A, A2, Pont | Deux par densité | 8, dont 3 nouvelles |
| `isaac_burning` | C, C2 | R, C, C2 | Deux par densité | 8, dont 3 nouvelles |
| `isaac_burning_psyche` | C, C2 | R, C, C2 | Deux par densité | 8, dont 3 nouvelles |
| `isaac_downpour` | A, B, C | A, B, C | Deux par densité | 8, dont 3 nouvelles |

Les alternatives d'accords ont des poids 1:1 ; les réponses de basse et batterie des poids 1:2. Pédales sombres, sensible, voisines phrygiennes, descentes/paliers chromatiques, stomp ou demi-temps conservés. Pas de programme 78 ni de solo ajouté. Instruments, registres, tempos et longueurs de partie conservés. Burning et Psyche sont identiques musicalement, fichiers annexes compris ; leurs métadonnées d'identité et sections Odin/mix existantes restent distinctes.

Schémas et règles : `profiles/README.md`. Variantes : en-têtes des cinq profils et `profiles/isaac_basement/README.md`.

## Preuves exécutées

### Snapshots et invariants

Avant modification : `cargo run -q -p harmonium_determinism --bin determinism_dump -- /tmp/harmonium-016/v1-before` ; après intégration : même commande vers `/tmp/harmonium-016/v1-after`. Chaque passage écrit 98 fichiers et exerce `LiveHost`, commandes publiques, composition de 64 mesures, tampon audio et horloge de pulsation.

Comparaison octet pour octet : **66/66 cas identiques** pour les dix profils trad et `nt_desert` (graines 1, 2, 3 × émotions `mid`, `hi` × 64 mesures). Cela inclut six comparaisons `nt_desert`, donc les deux graines demandées. Aucun golden régénéré.

Sur chacun des cinq profils Isaac, les dumps LiveHost `mid` des graines **1 et 2** présentent des suites d'accords et des grilles Heel/Toe distinctes.

Contrats permanents `seed_variety_tests.rs` : variété des deux couches aux densités 1 et 2 ; rejouabilité JSON exacte ; identité des retours de partie et du second tour ; renouvellement aux airs suivants ; aller-retour de densité ; réemploi en gigue ; mélodie réalisée dans la boucle choisie. Pour isoler l'identité du pattern de sa densité effective, le test de reprise neutralise les courbes de densité et impose deux tours (Downpour n'en déclare qu'un). Le test de variété considère les graines 1 à 16 : un tirage pondéré peut légitimement répéter un choix entre deux graines. **5/5 tests verts**.

Autres contrats ciblés exercés : harmonie en boucle, changement de mode avec alternatives, candidat unique, validation, motifs de basse, forme en boucle, patterns euclidiens, profils et notes `nt_desert`. La validation vérifie les rejets, sans figer les libellés d'erreur ; l'ancienne grille exacte de la fixture gigue est remplacée par les invariants de variété et d'alignement mélodie/accords.

### Vérifications workspace

- `cargo fmt -p harmonium_core`, puis `cargo fmt --all -- --check` : réussis. Aucun fichier du studio formaté ou modifié.
- `cargo test --workspace -q --no-fail-fast` : réussi en un passage complet à la fin des modifications ; 670 tests réussis, deux ignorés, aucune relance du test de seek nécessaire.
- Les contrats ciblés du core et les dumps LiveHost ont aussi été exécutés sans activer Odin ; la suite workspace inclut le studio et unifie l'option Odin.
- `cargo clippy --workspace` : réussi, avec avertissements non bloquants dans les modules hors périmètre (`timeline/export`, chargement Odin, hôte, CLI, labo). L'`expect` ajouté initialement au plan de pieds a été remplacé par une précondition explicite ; revalidation ciblée ensuite : 167 tests de bibliothèque et les 5 contrats de variété verts, puis clippy et fmt réussis. La suite complète n'a pas été relancée.

### Rendus d'écoute

Construction : `cargo build --release -p harmonium-cli --features odin` réussie. Banque `assets/GeneralUser-GS.sf2` disponible par lien local vers la banque du checkout principal ; presets `/tmp/harmonium-014/odin-presets`. Aucune modification aux banques.

Commande exécutée pour chaque ligne :

```sh
./target/release/harmonium-cli --export --bars 64 \
  --profile profiles/<id> --seed <g> \
  --soundfont assets/GeneralUser-GS.sf2 \
  --odin-presets /tmp/harmonium-014/odin-presets \
  --record-wav ~/Project/bascanada/harmonium_lab/corpus/isaac/ecoute/v1_<id>_graine<g>.wav
ffmpeg -hide_banner -nostats -i <wav> -af astats -f null -
```

Rendus silencieux, 64 mesures chacun. Dossier commun : `~/Project/bascanada/harmonium_lab/corpus/isaac/ecoute/`.

| WAV | Crête maximale des canaux (dBFS) |
|---|---:|
| `v1_isaac_basement_graine1.wav` | -6,014021 |
| `v1_isaac_basement_graine2.wav` | -4,278163 |
| `v1_isaac_cellar_graine1.wav` | -7,231657 |
| `v1_isaac_cellar_graine2.wav` | -5,885390 |
| `v1_isaac_burning_graine1.wav` | -4,761274 |
| `v1_isaac_burning_graine2.wav` | -3,586175 |
| `v1_isaac_downpour_graine1.wav` | -4,523674 |
| `v1_isaac_downpour_graine2.wav` | -4,824568 |

**8/8 sous 0 dBFS**, sans normalisation ni correction de mix. Journaux d'export et `astats` : `/tmp/harmonium-016/v1_*.log` et `v1_*.astats.log`. Psyche n'est pas rendu, conformément au brief.

## Limites

- Validation hors ligne et silencieuse : ni écoute humaine ni jugement perceptif revendiqués. Pour écouter : `afplay ~/Project/bascanada/harmonium_lab/corpus/isaac/ecoute/v1_isaac_cellar_graine1.wav` ; le studio reste le chemin de validation sonore du propriétaire.
- La variété est pondérée, pas une contrainte de non-répétition : deux graines ou deux airs peuvent choisir la même alternative. Les couches ne sont pas forcées à changer ensemble.
- La mémoire de set peut réutiliser une mélodie et un motif de basse existants avec une nouvelle sélection harmonique ; la réalisation mélodique suit l'accord de l'air courant.
- Comparaisons de snapshots effectuées en natif ; aucune nouvelle exécution de la matrice WASM dans cette tâche.

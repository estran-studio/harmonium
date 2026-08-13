# Plan d'implémentation : position de transport infra-temps

**Branche** : `006-sub-beat-transport-position` | **Date** : 2026-08-13 |
**Spec** : [spec.md](./spec.md)

## Résumé

Publier la position de lecture à la résolution de la grille interne
(double-croche) au lieu d'une fois par temps, par un unique `AtomicU64`
empaquetant mesure et step, lisible sans verrou depuis un callback temps
réel.

Le changement remplace le `playhead_bar: Arc<AtomicUsize>` existant plutôt
que de s'ajouter à côté, et ne touche ni la génération, ni le
`MeasureSnapshot`, ni la cadence des rapports.

## Contexte technique

**Langage** : Rust 2024, `unsafe_code = "deny"`, clippy strict. `gen` est
réservé.

**Crates touchées** : `harmonium_host` uniquement — `playback.rs`,
`composer.rs`, `audio.rs`, `native_handle.rs`, `lib.rs`. `harmonium_core`
n'est pas modifié.

**Concurrence** : un producteur (thread audio), plusieurs lecteurs.
`Ordering::Relaxed`, comme l'atomique existant — on lit une valeur unique,
sans synchroniser d'autres écritures avec elle.

**Cibles** : natif et `wasm32-unknown-unknown`. `AtomicU64` y compile,
vérifié au compilateur.

**Tests** : `cargo test --workspace`. Ce repo a une **vraie CI GitHub** — la
PR doit passer.

**Périmètre serré** : c'est du code sur le chemin audio temps réel. Le
changement doit rester une addition de lecture, et le prouver.

## Vérification constitutionnelle

### Constitution méta

- **I. Déterminisme** ✓ — aucun aléa touché. La position lue est celle du
  playhead, jamais interpolée depuis une horloge murale.
- **II. Pas de hack, pas de legacy** ✓ — `playhead_bar` est **remplacé**, pas
  doublé. Deux atomiques côte à côte auraient été deux sources de vérité pour
  la mesure ; c'est exactement le chemin parallèle que la constitution
  interdit.
- **III. Planification par workstream** ✓ — spec au niveau du comportement,
  chemins de fichiers ici et en pied de page.
- **IV. Tests d'intégration sur les vrais sous-systèmes** ✓ — le
  `PlaybackEngine` réel, pas un simulacre ; la non-régression du snapshot se
  vérifie sur le flux réel.
- **V. Cohésion par repo** ✓ — le changement est entièrement dans
  `harmonium`. Les adaptations côté app sont des tâches de l'autre repo,
  listées dans le contrat mais pas faites ici.

### Constitution locale du repo moteur

- **I. Le contrat audio, c'est `MeasureSnapshot`** ✓ — intact. Ce changement
  ajoute une lecture de la position de lecture ; il ne modifie ni la
  génération ni la timeline. **Aucun bump de version de `TuningParams`** —
  et c'est prouvé par un test de non-régression du flux de snapshots, pas
  affirmé.
- **II. Les tests portent sur le snapshot, pas sur les internes** ✓ — les
  tests de position portent sur la valeur publiée par l'accesseur public,
  pas sur l'état interne du playhead.
- **III. wasm32 de première classe** ✓ — `AtomicU64` compile pour
  `wasm32-unknown-unknown`, vérifié au compilateur avant d'écrire ce plan.
  Aucun `cfg` conditionnel.
- **IV. Frontières du workspace** ✓ — tout est dans `harmonium_host`, qui est
  la bonne place : c'est la couche qui porte le thread audio et le partage
  d'état entre threads.
- **V. Les paramètres de tuning vivent dans la config** — sans objet, aucune
  constante musicale ajoutée.

Revérification après conception : aucune violation. Pas de section
« Complexity Tracking ».

## Structure

```text
harmonium/harmonium_host/src/
├── playback.rs        le champ partagé, 4 points d'écriture (R7)
├── composer.rs        playhead_bar() dérivé du champ empaqueté
├── audio.rs           les 2 chemins de création de l'atomique
├── native_handle.rs   transport_position() + transport_position_handle()
└── lib.rs             mêmes accesseurs sur le Handle wasm
```

**Décision de structure** : le type `TransportPosition` et les fonctions
d'empaquetage vivent dans `harmonium_host`, aux côtés du champ partagé.
Elles ne remontent pas dans `harmonium_core` : le core ne connaît pas le
partage entre threads, c'est précisément la frontière que la constitution
locale IV pose.

## Séquencement

L'ordre est dicté par une seule chose : **prouver tôt qu'on n'a rien cassé**,
parce que le terrain est le chemin audio.

1. **Le filet** — le test de non-régression du flux de snapshots, écrit et
   vert AVANT toute modification. Sans lui, on ne saura pas ce qu'on a cassé.
2. **L'empaquetage** — fonctions pures, testées isolément. Aucun risque, et
   ça fixe la sémantique.
3. **Le remplacement** — `playhead_bar` devient la position empaquetée, les
   quatre points d'écriture suivent, `playhead_bar()` devient un dérivé.
4. **Les accesseurs** — `transport_position()` et le partage brut, sur
   `NativeHandle` et sur le Handle wasm.
5. **La preuve** — les 16 positions par mesure, observées et non déduites.

## Risque principal

Un point d'écriture oublié ne casse rien de visible : la position se fige ou
recule, silencieusement, et le symptôme n'apparaît que des couches plus loin
sous forme de notes mal horodatées. Les quatre sites sont énumérés dans
`research.md` R7 et chacun a son test — le seek et le démarrage autant que
la lecture continue.

## Prérequis

Aucun. Ce travail ne dépend ni de W1 ni de la suite de W2, et rien ne le
bloque.

## Suite, dans l'autre repo

Une fois mergé, `harmonium_training` peut lire la position sans verrou dans
son callback MIDI et supprimer sa dérivation locale du temps fractionnaire.
C'est ce qui débloque réellement les métriques rythmiques de W2 — mais c'est
une tâche de ce repo-là, référencée par le méta-epic.

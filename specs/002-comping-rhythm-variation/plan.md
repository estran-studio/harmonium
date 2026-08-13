# Plan d'implémentation : W1 — la voix d'accords (comping)

**Branche** : `002-comping-rhythm-variation` | **Date** : 2026-08-13 |
**Spec** : [spec.md](./spec.md)

## Résumé

Donner au moteur une cinquième piste dédiée aux accords, jouée avec un
rythme de comping jazz qui lui est propre — frappes irrégulières, mesures
muettes, anticipations Charleston liées par-dessus la barre — et l'amener
jusqu'à l'audio, au snapshot, aux exports, au mixer et au soundfont.

Le workstream ne raffine pas des voicings existants : **il n'en existe
aucun dans le chemin de génération**. `harmonium_audio::voicing` est un
module orphelin sans appelant, et `enable_voicing` est un paramètre câblé
sur six couches qui ne fait rien. Voir `research.md` R0 — la prémisse de la
spec a été corrigée à la lumière du code.

## Contexte technique

**Langage** : Rust 2024, workspace `harmonium`. `unsafe_code = "deny"`,
clippy strict. `gen` est un mot-clé réservé.

**Crates touchées** : `harmonium_core` (génération, timeline, playhead,
voicing porté, params, tuning), `harmonium_audio` (suppression du module
voicing orphelin), `harmonium_host` (routage de programme du canal 4,
réexport supprimé).

**Aléa** : `ChaCha8Rng`, passé en `&mut dyn RngCore`. Le comping tire dans
un flux enfant dérivé de `(session_seed, index de mesure)` — jamais dans le
flux principal.

**Tests** : `cargo test --workspace`. Ce repo a une **vraie CI GitHub** — la
PR doit passer (asymétrie assumée avec `harmonium_training`, dont la porte
est locale).

**Contraintes** : additif à l'oreille — aucune piste existante ne doit
changer de son. C'est vérifié par un test d'invariance, pas par relecture.

**Échelle** : une piste supplémentaire sur un moteur temps réel ; le
placement des frappes est O(steps par mesure).

## Vérification constitutionnelle

*Porte avant Phase 0, revérifiée après la conception.*

- **I. Déterminisme** ✓ — RNG enfant semé explicitement, aucun tirage dans
  le flux principal ; l'invariant de non-perturbation est un test, pas une
  intention (R5). Aucun nouveau type de RNG.
- **II. Pas de hack, pas de legacy** ✓ — le module `harmonium_audio::voicing`
  est porté puis **supprimé dans le même changement**. Aucun chemin
  parallèle, aucun `_legacy`. L'ordonnanceur de NoteOff par durée est la
  bonne architecture, choisie plutôt qu'un contournement du remplacement
  existant (R3).
- **III. Planification par workstream** ✓ — `spec.md` reste au niveau du
  comportement observable ; les chemins de fichiers vivent dans ce plan, dans
  `research.md` et dans le pied de page de la spec.
- **IV. Tests d'intégration sur les vrais sous-systèmes** ✓ — le générateur
  et le playhead réels, pas de moteur simulé ; les exports MIDI et MusicXML
  sont vérifiés sur les fichiers produits.
- **V. Cohésion par repo** ✓ — cette spec est le volet moteur du méta-epic
  `012-w1-chord-voice`, référencé en frontmatter. Le peuplement des profils
  de style est renvoyé à `harmonium_training` (R6).

### Constitution locale du repo moteur

*(Elle existe — `harmonium/.specify/memory/constitution.md` — et la première
version de ce plan l'ignorait. Les deux premiers points sont les corrections
qui en découlent.)*

- **I. Le contrat audio, c'est `MeasureSnapshot`** ✓ *après correction* —
  ajouter une piste et activer le voicing par défaut altèrent le snapshot à
  seed identique. C'est un changement cassant : il exige un bump de version
  sur `TuningParams` (T008b), et une note de migration.
- **II. Les tests portent sur le snapshot, pas sur les internes** ✓ *après
  correction* — les tests de déterminisme affirment sur les notes de la piste
  d'accords dans le snapshot, jamais sur les `CompingTrigger` internes.
- **III. wasm32 est une cible de première classe** ✓ — le module `voicing`
  porté depuis `harmonium_audio` doit laisser `harmonium_core` compilable en
  wasm sans `cfg` conditionnel ; vérifié par T008c.
- **IV. Frontières du workspace** ✓ — `harmonium_core` est « pure types, no
  I/O, fully wasm-safe ». Le voicing est de la théorie musicale pure : sa
  place est dans le core, pas dans un crate de primitives DSP. Le portage
  corrige un mauvais rangement autant qu'il débloque la fonctionnalité.
- **V. Les paramètres de tuning vivent dans la config, pas en constantes** ✓
  — `CompingParams` entre dans `TuningParams`, et le coefficient de réponse à
  la mélodie y entre aussi plutôt que d'être écrit en dur (T024). La chaîne
  de tuning LLM de `harmonium_lab` dépend d'une couverture exhaustive.

Revérification après conception : aucune violation restante. Pas de
section « Complexity Tracking ».

## Structure

```text
harmonium/
├── harmonium_core/src/
│   ├── timeline/
│   │   ├── mod.rs              TrackId::Chord, canal 4, ALL passe à 5
│   │   ├── generator.rs        génération des CompingTrigger + hauteurs
│   │   ├── pointers.rs         track_cursors [5], bras Chord,
│   │   │                       ordonnanceur de NoteOff par durée
│   │   ├── export.rs           portée d'accords en MusicXML
│   │   └── midi_export.rs      cinquième piste MIDI
│   ├── voicing/                NOUVEAU — porté depuis harmonium_audio
│   │   ├── mod.rs
│   │   ├── shell.rs            guide tones — le voicing par défaut
│   │   └── block_chord.rs      locked hands
│   ├── params.rs               enable_voicing par défaut à true
│   ├── tuning.rs               CompingParams, dixième sous-structure
│   └── report.rs               suit ALL tout seul
├── harmonium_audio/src/
│   └── voicing/                SUPPRIMÉ (porté vers core)
└── harmonium_host/src/
    ├── lib.rs                  réexport voicing supprimé
    └── main.rs                 routage programme GM du canal 4
```

**Décision de structure** : le choix des hauteurs remonte dans
`harmonium_core` parce que `harmonium_audio` dépend de `harmonium_core` et
non l'inverse, et surtout parce que les notes d'accords doivent apparaître
dans `MeasureSnapshot` — sinon elles n'atteignent ni l'écran de l'app, ni les
exports, ni le scoring de W2 (R1).

## Séquencement

L'ordre est contraint par une seule chose : **la piste doit s'entendre le
plus tôt possible**, pour que le jugement d'écoute puisse corriger le tir
avant que le raffinement rythmique ne soit construit.

1. **La piste existe et sonne** — `TrackId::Chord`, un accord plaqué au
   temps 1 de chaque mesure, le chemin complet jusqu'à l'audio. Moche mais
   audible. C'est le jalon qui dérisque tout le reste.
2. **L'invariant de non-régression** — le test qui prouve que rien d'existant
   n'a bougé. Avant tout raffinement.
3. **Le rythme propre** — `CompingParams`, mesures muettes, placement
   irrégulier, indépendance du lead.
4. **La liaison par-dessus la barre** — l'ordonnanceur de NoteOff par durée,
   puis les anticipations Charleston.
5. **Réponse à la mélodie et styles** — la couche de finition.

## Prérequis

**Le méta-epic.** Ce travail change le contrat de snapshot **et** son
consommateur dans l'app d'entraînement (routage soundfont, mixer, profils de
style) : la constitution locale, section Spec Scope, en fait explicitement du
travail transverse. La racine du workstream est donc
`harmonium_specs/specs/012-w1-chord-voice/`, et cette spec en est le volet
moteur (lié par `meta_epic:` en frontmatter).

**Le bump de version.** La constitution locale, principe I, fait du snapshot
le seul engagement du moteur envers l'extérieur : « un changement qui altère
le snapshot à seed et entrées identiques est un changement cassant et exige
une migration explicite — le bump de version sur `TuningParams` est le
mécanisme canonique ». W1 altère le snapshot deux fois (piste supplémentaire,
voicing actif par défaut). Le bump est en phase 1, dans le même commit que le
changement qui le provoque (T008b), pas en fin de parcours.

En revanche, W1 ne dépend pas de W2 et ne partage aucun fichier avec lui :
les deux workstreams vivent dans des repos différents et avancent en
parallèle.

## Risque principal

L'étape 4 est celle qui peut déraper : toucher au playhead, c'est toucher au
chemin audio temps réel. La réserve « la piste d'accords uniquement » est ce
qui contient le risque — si l'ordonnanceur de durée devait être généralisé à
toutes les pistes, ce serait une autre spec, avec sa propre recette d'écoute
sur les sons existants.

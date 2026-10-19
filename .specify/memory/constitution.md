# Constitution Harmonium

Harmonium est un moteur Rust de musique procédurale **de style folklorique
québécois / canadien-français**, piloté en temps réel par l'état d'un jeu
(« 1837 », roguelike Rust/Bevy). Le beat est exposé au gameplay ; deux
jauges émotionnelles (sacre, faim) et l'intensité de l'action pilotent la
musique.

Ce document est court à dessein. Il fixe ce qui ne se renégocie pas à chaque
feature. Le reste se décide en écoutant.

## Principes

### I. Le contrat audio, c'est `MeasureSnapshot` + `BeatInfo`

Le moteur s'engage sur deux flux : les `MeasureSnapshot` générés (notes par
piste, accord, tempo, signature ; bars et beats 1-based, tick sur la grille)
et l'horloge `BeatInfo` lue par le gameplay (pulse courante, phase, temps
fort, prochaine pulse en temps mur). Tout l'intérieur — générateur,
harmonisation, banques de cellules, playhead — est détail d'implémentation.

Un changement qui altère le snapshot pour un même seed + mêmes entrées est
une rupture : bump de version de `TuningParams`, goldens régénérés
explicitement dans le même commit.

### II. Déterminisme

Même seed, mêmes paramètres, même musique. Pas d'aléa hors du
`ChaCha8Rng` injecté ; aucune lecture d'horloge murale dans la génération.
La position lue par le jeu vient du playhead, jamais interpolée depuis une
horloge externe côté moteur.

### III. Temps réel sans compromis

Le callback audio n'alloue pas, ne prend pas de verrou bloquant, ne fait
pas d'I/O. `RTCheckAllocator` panique en debug sur une allocation dans le
callback : c'est voulu. Thread principal → thread audio par SPSC `rtrb` ou
atomiques ; jamais par `Mutex` partagé sur le chemin chaud.

### IV. Pas de chemin parallèle, pas de legacy

Une seule façon de faire chaque chose. On remplace, on ne double pas. Code
mort, alias de compatibilité, `#[cfg]` sans consommateur, commentaires
« removed » : supprimés dans le commit qui les rend obsolètes.

### V. Le style est en données, les algorithmes en code

Tout ce qui définit la personnalité musicale (patterns de pieds, cellules
mélodiques, seuils, vélocités, tessitures, tonalités permises, tempos par
type de tune) vit dans `TuningParams` / `StyleProfile` (TOML), jamais en
constante inline. `harmonium_lab` mesure la distance entre le généré et un
corpus de référence ; les décisions de style s'appuient sur cette mesure et
sur l'écoute, dans cet ordre.

### VI. Tests sur le contrat, pas sur les internes

Les tests portent sur le flux de snapshots et sur `BeatInfo` : timing des
notes, cadences, répétition de forme, position publiée. Un test qui lit un
état privé est fragile et refusé.

## Périmètre du workspace

- `harmonium_core` — types purs, génération, aucune I/O.
- `harmonium_audio` — rendu SoundFont (oxisynth), enregistrement WAV/MIDI/MusicXML.
- `harmonium_host` — `MusicComposer` (thread principal), `PlaybackEngine`
  (callback cpal), `NativeHandle`, `create_offline_engine`, binaire
  `harmonium` avec contrôle OSC.
- `harmonium_cli` — REPL d'écoute et de test.
- `harmonium_lab` — ingestion de corpus, métriques, tuning assisté.

Pas de frontend dans ce repo. L'intégration Bevy vit dans le jeu, sur
`create_offline_engine`.

## Contraintes techniques

- Rust 2024, `unsafe_code = "deny"`, clippy strict. `gen` est réservé.
- `ChaCha8Rng` est le seul RNG.
- Synthèse : SoundFont uniquement. Banque de test : `assets/test.sf2`.
- Tempo exprimé à la noire ; la pulse dansée (blanche en reel, noire
  pointée en gigue) est dérivée dans `BeatInfo`, pas dans le tempo.

## Processus

Un `plan.md` par phase dans `specs/NNN-nom/` : problème, décisions,
fichiers touchés, critère d'acceptation observable. Pas de template, pas de
spec avant d'avoir entendu. Les décisions de style se figent en ADR dans le
plan de la phase *après* validation à l'oreille et au lab.

**Version** : 2.0.0 | **Ratifiée** : 2026-10-18

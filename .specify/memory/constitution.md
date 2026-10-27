# Constitution Harmonium

Harmonium est un moteur Rust de musique procédurale **tous styles** : la
boîte à chansons procédurale ultime. Tout style de musique — tradition,
musique de jeu, genre populaire ou savant — doit pouvoir être généré, et les
styles doivent pouvoir se fusionner. Le moteur est piloté en temps réel par
l'état d'un jeu : « 1837 » (roguelike Rust/Bevy ; traditions québécoise,
acadienne, autochtones, britannique coloniale…) est le premier, d'autres
suivent (clones de Nuclear Throne et de The Binding of Isaac). Le beat est
exposé au gameplay ; quatre émotions (arousal, valence, density, tension)
pilotent la musique, et chaque jeu traduit ses propres signaux en émotions.

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

Un style (reel québécois, marche fifre-et-tambour britannique, chant au
tambour, boucle synthé basse-batterie d'un désert post-apocalyptique, thème
d'étage orgue-et-chœur…) est un **profil de style** en données : signature
et grille, forme, banques de cellules mélodiques, modes et tonalités permis,
modèle d'harmonie (fonctionnelle, modale, boucle, pédale ou aucune),
instruments, percussions, arrangement, tempos, vélocités, tessitures. Il vit
dans `TuningParams` / `StyleProfile` (TOML), jamais en constante inline ni en
`match` sur un nom de style. Ajouter un style ne demande que des données ; un
algorithme manquant (ex. appel-réponse, bourdon, riff en ostinato) s'ajoute
une fois, en code générique, réutilisable par tous les styles.
`harmonium_lab` mesure la distance entre le généré et un corpus de référence
par style ; les décisions de style s'appuient sur cette mesure et sur
l'écoute, dans cet ordre.

### VI. Tests sur le contrat, pas sur les internes

Les tests portent sur le flux de snapshots et sur `BeatInfo` : timing des
notes, cadences, répétition de forme, position publiée. Un test qui lit un
état privé est fragile et refusé.

### VII. Respect des sources

Chaque style s'appuie sur un corpus de référence dont la provenance (œuvre,
auteur, source, licence) est notée dans un manifeste. Toute musique peut
servir de référence d'analyse ou d'inspiration, bandes originales
commerciales comprises ; seul ce qui en est dérivé (mesures, cellules,
profils) entre dans les dépôts, jamais les enregistrements ni les
transcriptions sous droits. Pour les musiques autochtones : pas de chants
sacrés ou cérémoniels, pas de corpus sans droits clairs, et une validation
par des musiciens ou conseillers des nations concernées avant qu'un style
entre dans un jeu. Un style se nomme par sa source précise — tradition
(nation, région, époque) ou jeu et zone (`nt_desert`, `isaac_basement`) —,
jamais par une étiquette générique.

### VIII. La fusion est en données

Deux styles se fusionnent en composant leurs profils, pas en écrivant du
code pour la paire : un profil peut reprendre d'un autre n'importe quelle
couche (mètre, forme, cellules, harmonie, percussions, instruments, mixage)
et en remplacer d'autres — par exemple une grammaire de reel sur la basse et
la batterie d'une boucle synthé. Un algorithme qui ne fonctionne que dans
son style d'origine est un défaut.

## Périmètre du workspace

- `harmonium_core` — types purs, génération, aucune I/O.
- `harmonium_audio` — rendu SoundFont (oxisynth), enregistrement WAV/MIDI/MusicXML.
- `harmonium_host` — `MusicComposer` (thread principal), `PlaybackEngine`
  (callback cpal), `NativeHandle`, `create_offline_engine`, binaire
  `harmonium` avec contrôle OSC.
- `harmonium_cli` — REPL d'écoute et de test.
- `harmonium_lab` — ingestion de corpus, métriques, tuning assisté.
- `harmonium_studio` — application egui : vitrine publique sur le web (WASM)
  et atelier de développement en natif.

Un seul frontend, `harmonium_studio`. Il ne lit que le contrat (principe I)
et l'API publique du host, et exécute les commandes par le même chemin que
le REPL, qui garde les mêmes capacités. L'intégration Bevy vit dans chaque
jeu, sur `create_offline_engine`.

## Contraintes techniques

- Rust 2024, `unsafe_code = "deny"`, clippy strict. `gen` est réservé.
- `ChaCha8Rng` est le seul RNG.
- Synthèse : SoundFont uniquement ; chaque jeu fournit sa banque. Banque de
  test : `assets/test.sf2`.
- Tempo exprimé à la noire ; la pulse dansée (blanche en reel, noire
  pointée en gigue) est dérivée dans `BeatInfo`, pas dans le tempo.

## Processus

Un `plan.md` par phase dans `specs/NNN-nom/` : problème, décisions,
fichiers touchés, critère d'acceptation observable. Pas de template, pas de
spec avant d'avoir entendu. Les décisions de style se figent en ADR dans le
plan de la phase *après* validation à l'oreille et au lab.

**Version** : 4.1.0 | **Ratifiée** : 2026-10-18 | **Amendée** : 2026-10-27
(moteur tous styles et multi-jeux, fusion en données — principe VIII ;
sources sous droits analysables, seuls les dérivés entrent au dépôt —
principe VII ; un seul frontend, `harmonium_studio`, vitrine web et atelier
natif)

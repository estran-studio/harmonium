---
description: "Liste de tâches — W1 la voix d'accords (002-comping-rhythm-variation)"
---

# Tâches : W1 — la voix d'accords (comping)

**Entrée** : documents de conception de `specs/002-comping-rhythm-variation/`

**Prérequis** : plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests** : inclus — les critères de succès de la spec les exigent, et la
constitution IV impose les vrais sous-systèmes. Ce repo a une CI GitHub
réelle : la PR doit passer.

**Format** : `- [ ] T### [P?] [US#] Description avec chemin de fichier`
`[P]` = parallélisable (fichiers différents, sans dépendance).

---

## Phase 1 : Fondation — la piste existe et sonne

**But** : entendre un accord, même bête, le plus tôt possible. Tout le reste
se raffine dessus, et le jugement d'écoute peut corriger le tir avant que le
rythme ne soit construit.

- [ ] T001 Ajouter la variante `Chord` à `TrackId` avec le canal MIDI 4 et étendre `ALL` à 5 dans `harmonium_core/src/timeline/mod.rs` — puis laisser le compilateur énumérer les sites cassés (c'est la liste de travail des tâches suivantes)
- [ ] T002 Étendre `track_cursors` de `[usize; 4]` à `[usize; 5]` et ajouter le bras `TrackId::Chord` au `match` de `Playhead::tick` dans `harmonium_core/src/timeline/pointers.rs` — polyphonique, NoteOn de chaque hauteur, coupure par remplacement pour l'instant (la durée arrive en phase 4)
- [ ] T003 [P] Ajouter la piste d'accords à l'export MIDI dans `harmonium_core/src/timeline/midi_export.rs`
- [ ] T004 [P] Ajouter la portée d'accords à l'export MusicXML dans `harmonium_core/src/timeline/export.rs`
- [ ] T005 Porter le choix des hauteurs de `harmonium_audio/src/voicing/` vers `harmonium_core/src/voicing/` (shell + block chord, `get_guide_tones`, `apply_drop_two`) et **supprimer `harmonium_audio/src/voicing/` ainsi que son réexport dans `harmonium_host/src/lib.rs`** dans le même commit (constitution II — pas de chemin parallèle)
- [ ] T006 Générer un accord plaqué au temps 1 de chaque mesure sur `TrackId::Chord` dans `harmonium_core/src/timeline/generator.rs`, en voicing shell depuis `Measure::chord_context`, conditionné par `enable_voicing` — le paramètre devient enfin effectif
- [ ] T007 Router le canal 4 vers un programme GM de piano au chargement d'un soundfont dans `harmonium_host/src/main.rs` et `harmonium_host/src/timeline_engine.rs`
- [ ] T008 Passer `enable_voicing` à `true` par défaut dans `harmonium_core/src/params.rs` (lignes 390 et 712) — SC-003
- [ ] T008b **Bump de version `TuningParams` + note de migration** dans `harmonium_core/src/tuning.rs` — constitution locale I : le snapshot n'est plus le même à seed et entrées identiques (une piste de plus, et le voicing actif par défaut), donc c'est un changement cassant du contrat audio et le bump est le mécanisme canonique. À faire dans le même commit que T006/T008, pas après
- [ ] T008c [P] Vérifier que le module `voicing` porté garde le core compilable en wasm : `cargo check -p harmonium_core --target wasm32-unknown-unknown` — constitution locale III/IV, `harmonium_core` est « fully wasm-safe, no I/O » et le portage vient d'un crate qui n'a pas cette contrainte

**Point de contrôle** : `cargo run -p harmonium_cli` sur un blues en Si♭ →
on entend un piano. Moche, plaqué, mais présent. **Ne pas continuer sans
avoir écouté.**

---

## Phase 2 : L'invariant de non-régression

**But** : prouver que rien d'existant n'a bougé. Avant tout raffinement,
parce qu'une régression introduite ici serait invisible et empoisonnerait
tout le workstream.

- [ ] T009 Dériver un RNG enfant pour le comping depuis `(session_seed, index de mesure)` dans `harmonium_core/src/timeline/generator.rs` — aucun tirage de comping dans le flux principal (research R5)
- [ ] T010 Test `comping_does_not_perturb_existing_tracks` dans `harmonium_core/tests/` : rendre 100 mesures à graine fixe avec comping désactivé puis activé, affirmer que les notes de Bass, Lead, Snare et Hat sont strictement identiques
- [ ] T011 Test de déterminisme du comping lui-même : deux rendus à graine identique produisent des **notes de piste d'accords identiques dans le snapshot** — on affirme sur le flux de `MeasureSnapshot`, jamais sur les `CompingTrigger` internes (constitution locale II : les tests portent sur le snapshot, pas sur les internes) (FR-010)
- [ ] T012 Vérifier que la suite existante passe intégralement avec les `CompingParams` par défaut (FR-011) : `cargo test --workspace`

**Point de contrôle** : l'invariant est vert. À partir d'ici, toute
régression sur les pistes existantes est détectée automatiquement.

---

## Phase 3 : User Story 1 — rythme indépendant, frappes éparses et mesures muettes (P1) 🎯

**But** : le comping a son propre rythme, il ne double plus la mélodie
(FR-001 à FR-004, FR-008, FR-009)

**Test indépendant** : `hits_per_bar: 2.5, layoff_probability: 0.15`, graine
fixe, 100 mesures — les steps de comping ne sont pas un sous-ensemble des
steps de lead, la moyenne est dans ±20 % de 2.5, et ~15 % des mesures sont
muettes.

- [ ] T013 [US1] Définir `CompingParams` (les cinq champs de `data-model.md`) comme dixième sous-structure de `TuningParams` dans `harmonium_core/src/tuning.rs`, avec `serde(default)` et couverture dans `validate()`
- [ ] T014 [US1] Définir `CompingTrigger { step, velocity, duration_steps, tied }` dans `harmonium_core/src/timeline/mod.rs`
- [ ] T015 [P] [US1] Tests du placement des frappes dans `harmonium_core/tests/` : moyenne de frappes par mesure dans ±20 %, mesures muettes au taux configuré, `hits_per_bar = 0.0` sans division par zéro, `hits_per_bar` au-delà de la grille clampé et non rejeté (research R4)
- [ ] T016 [P] [US1] Test d'indépendance : sur 100 mesures, le recouvrement entre steps de comping et steps de lead reste sous 50 % (SC-001)
- [ ] T017 [US1] Implémenter la génération des `CompingTrigger` dans `harmonium_core/src/timeline/generator.rs` : tirage de mesure muette d'abord, puis placement irrégulier des frappes, en remplacement de l'accord plaqué de T006
- [ ] T018 [US1] Résoudre chaque déclencheur en voicing via `harmonium_core/src/voicing/`, piloté par `voicing_density` et `voicing_tension` (FR-009 — cette spec décide du *quand*, le voicing décide du *quoi*)

**Point de contrôle** : US1 livrée. Le comping sonne comme un pianiste
éparse, plus comme un séquenceur.

---

## Phase 4 : User Story 2 — anticipations Charleston et réponse à la mélodie (P2)

**But** : le geste jazz identifiable, et un comping qui écoute (FR-005 à
FR-007)

**Dépend de** : US1

⚠️ **Phase à risque** : elle touche au chemin audio temps réel. La réserve
« piste d'accords uniquement » est ce qui contient le risque.

- [ ] T019 [US2] Ajouter au playhead un ordonnancement des NoteOff par durée **pour la piste d'accords seule**, en échéances de steps absolus qui survivent à `load_measure()`, dans `harmonium_core/src/timeline/pointers.rs` (research R3 — `duration_steps` n'est aujourd'hui jamais lu par le playhead)
- [ ] T020 [US2] Test du playhead : une note d'accords de durée supérieure au reste de la mesure reçoit son NoteOff dans la mesure suivante, à la bonne échéance ; les pistes Bass, Lead, Snare et Hat gardent exactement leur comportement de coupure par remplacement
- [ ] T021 [US2] Placer les anticipations sur le « et du 4 » selon `anticipation_probability` dans `harmonium_core/src/timeline/generator.rs` (FR-005)
- [ ] T022 [US2] Poser la liaison Charleston selon `charleston_probability` : `duration_steps` qui dépasse la mesure, `tied = true` ; raccourcir en dernière mesure générée plutôt que de supprimer ou de laisser pendre (data-model.md)
- [ ] T023 [P] [US2] Tests des anticipations : à `anticipation_probability = 1.0` chaque mesure porte une frappe sur le « et du 4 » ; à `charleston_probability = 1.0` chaque anticipation se prolonge sur le temps 1 suivant ; le cas de la dernière mesure ne laisse aucune note bloquée
- [ ] T024 [US2] Implémenter la réponse à la mélodie dans `harmonium_core/src/timeline/generator.rs` : densité de comping inversement corrélée à la densité de lead de la mesure ; un lead vide revient à la densité de référence et ne tombe pas à zéro (FR-007). **Le coefficient de réponse va dans `CompingParams`, pas en constante inline** — constitution locale V : toute constante qui affecte la sortie musicale vit dans `TuningParams`, la chaîne de tuning LLM de `harmonium_lab` en dépend
- [ ] T025 [P] [US2] Test de la réponse à la mélodie : sur un passage de croches denses au lead, la densité de comping baisse sous la référence ; avec un lead vide, elle y revient
- [ ] T026 [US2] Vérifier que la liaison ressort correctement dans les deux exports : note tenue par-dessus la barre en MusicXML, durée MIDI correcte — pas deux notes réattaquées

**Point de contrôle** : US1 et US2 livrées. À l'écoute, le comping pousse
sur le « et du 4 » et s'efface quand la mélodie s'active.

---

## Phase 5 : User Story 3 — le feeling par style (P3)

**But** : bossa, ballade, funk et bebop compent différemment

**Dépend de** : US1, US2

- [ ] T027 [US3] Poser les valeurs par défaut de `CompingParams` par style dans les profils de tuning du moteur, selon le tableau de `spec.md`, dans `harmonium_core/src/tuning.rs`
- [ ] T028 [P] [US3] Tests par style : rendre 50 mesures par profil et affirmer que la moyenne de frappes par mesure correspond au tableau à ±20 %, et que les styles sont statistiquement distinguables
- [ ] T029 [US3] Vérifier que les `CompingParams` traversent le morphing existant (`morph_factor = 0.03`) sans à-coup lors d'un changement de style en cours de session
- [ ] T030 [US3] Ouvrir la tâche de peuplement des 15 profils de style dans `harmonium_training/static/profiles/` — autre repo, constitution V : à référencer, pas à faire ici

**Point de contrôle** : les trois user stories livrées.

---

## Phase 6 : Finition et recette

- [ ] T031 [P] Vérifier le mixer : `muted_channels[4]` coupe le piano et laisse le reste jouer
- [ ] T032 [P] Passe complète de `quickstart.md`, y compris les exports
- [ ] T033 **La recette d'écoute** — blues en Si♭ à 120, swing medium, 12 mesures **sans regarder l'écran** : peut-on suivre les changements d'accord à l'oreille ? C'est le critère d'arrêt du workstream (research R8). Consigner le jugement dans la PR.
- [ ] T034 `cargo test --workspace` vert, `cargo check -p harmonium_core --target wasm32-unknown-unknown` vert, et CI GitHub verte sur la PR
- [ ] T035 Remonter l'avancement au méta-epic `harmonium_specs/specs/012-w1-chord-voice/tasks.md` et ouvrir le volet `harmonium_training` (routage soundfont, mixer, profils de style)

---

## Dépendances et ordre d'exécution

### Entre phases

- **Phase 1** : T001 en premier (le compilateur produit la liste), puis T002 ; T003/T004 en parallèle ; T005 avant T006 et T008c ; T007/T008 en fin de phase, avec T008b dans le même commit que T006/T008
- **Phase 2** : après la phase 1 — T009 avant T010/T011
- **Phase 3 (US1)** : après la phase 2. T013/T014 d'abord, tests T015/T016 avant T017/T018
- **Phase 4 (US2)** : après US1. T019 et son test T020 avant tout le reste de la phase
- **Phase 5 (US3)** : après US2
- **Phase 6** : à la fin

### Entre user stories

- **US1 (P1)** : ne dépend d'aucune autre story — le MVP du workstream
- **US2 (P2)** : dépend d'US1, et surtout de l'ordonnanceur de NoteOff (T019)
- **US3 (P3)** : dépend d'US1 et US2 ; c'est la couche de finition

### Parallélisme

- Phase 1 : T003 ∥ T004 (deux formats d'export, deux fichiers)
- Phase 3 : T015 ∥ T016
- Phase 4 : T023 ∥ T025
- Phase 6 : T031 ∥ T032

---

## Stratégie d'implémentation

### Le MVP, c'est la phase 1

Sortir du son avant de le rendre beau. Un accord plaqué au temps 1 prouve
que les huit couches — génération, timeline, playhead, canal, mixer,
soundfont, snapshot, exports — sont branchées. C'est là qu'un câblage
manquant se découvre, pas après avoir écrit le générateur rythmique.

### Livraison incrémentale

1. Phase 1 → on entend l'harmonie (le constat de `STRATEGY_2026.md` tombe)
2. Phase 2 → on sait qu'on n'a rien cassé
3. + US1 → ça sonne comme un pianiste, plus comme un séquenceur
4. + US2 → le geste jazz et l'écoute mutuelle
5. + US3 → le feeling par style

---

## Notes

- `gen` est un mot-clé réservé en Rust 2024 — utiliser `tgen` ou un nom métier
- La CI GitHub de ce repo est réelle : la PR doit passer, contrairement à
  `harmonium_training` dont la porte est locale
- Aucun tirage de comping dans le flux d'aléa principal — c'est l'invariant
  qui protège toutes les sessions déjà sauvegardées
- La métrique d'authenticité du comping dans `harmonium_lab` n'existe pas
  (research R7) : ce n'est pas un critère de W1, c'est une spec à écrire dans
  ce repo-là

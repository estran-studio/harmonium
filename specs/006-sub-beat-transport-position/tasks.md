---
description: "Liste de tâches — position de transport infra-temps"
---

# Tâches : position de transport infra-temps

**Entrée** : documents de conception de `specs/006-sub-beat-transport-position/`

**Prérequis** : plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests** : inclus. Le terrain est le chemin audio temps réel et le contrat
public du moteur ; la constitution locale I et IV s'appliquent directement.

**Format** : `- [ ] T### [P?] [US#] Description avec chemin de fichier`

---

## Phase 1 : Le filet

**But** : pouvoir prouver, à tout moment, qu'on n'a pas touché au contrat
audio. Écrit et vert **avant** la moindre modification.

- [ ] T001 Test de non-régression `snapshot_stream_unchanged` dans `harmonium_host/tests/` : rendre N mesures à graine fixe et figer le flux de `MeasureSnapshot` (notes, positions, accords) comme référence. Il doit être vert sur le code actuel, non modifié — sinon il ne prouve rien
- [ ] T002 Vérifier la base de départ : `cargo test --workspace` vert, et `cargo check -p harmonium_host --target wasm32-unknown-unknown` vert, avant tout changement

**Point de contrôle** : le filet est en place et l'état initial est connu.

---

## Phase 2 : L'empaquetage

**But** : la sémantique de la valeur partagée, en fonctions pures sans aucun
risque

- [ ] T003 Définir `TransportPosition { bar, step, beat }` et les fonctions pures `pack(bar, step) -> u64` / `unpack(u64) -> (bar, step)` dans `harmonium_host/src/playback.rs`, selon `data-model.md`
- [ ] T004 [P] Tests de l'empaquetage : aller-retour `unpack(pack(b, s)) == (b, s)` sur des valeurs représentatives et aux bornes, valeur initiale `pack(1, 0)`, et dérivation de `beat` (step 0 → 1.0, step 2 → 1.5, step 6 → 2.5)

---

## Phase 3 : User Story 1 — la position publiée suit la grille (P1) 🎯

**But** : la position exposée change à chaque step, plus une fois par temps
(FR-001, FR-003, FR-005)

**Test indépendant** : faire avancer le transport d'une mesure en 4/4 et
collecter les positions publiées — 16 distinctes, pas 4.

- [ ] T005 [US1] Remplacer le champ `playhead_bar: Arc<AtomicUsize>` par la position empaquetée `Arc<AtomicU64>` dans `harmonium_host/src/playback.rs` — remplacement, pas ajout : deux sources de vérité pour la mesure pourraient diverger (constitution II)
- [ ] T006 [US1] Mettre à jour les **quatre** points d'écriture de `harmonium_host/src/playback.rs` listés dans `research.md` R7 : ligne 242 (démarrage → step 0), 293 (chaque buffer → step courant), 375 et 384 (seek → step 0). Un site oublié fige ou fait reculer la position en silence
- [ ] T007 [US1] Faire de `playhead_bar()` un dérivé du champ empaqueté dans `harmonium_host/src/composer.rs:731` — les appelants (`composer.rs:214`, `:267`, `:289`) ne changent pas
- [ ] T008 [US1] Propager le nouveau type dans les constructeurs : `MusicComposer::new` / `new_with_seed` (`composer.rs`), `PlaybackEngine::new` (`playback.rs`), et les **deux** chemins de création de `harmonium_host/src/audio.rs` (lignes 73 et 174 — natif et wasm)
- [ ] T009 [US1] Mettre à jour `harmonium_host/tests/deterministic_seek_tests.rs:90` qui construit l'atomique
- [ ] T010 [P] [US1] Test : sur une mesure en 4/4, la position publiée prend 16 valeurs distinctes (SC-001)
- [ ] T011 [P] [US1] Test du seek : après un seek vers la mesure N, la position vaut `(N, 0)` — ni figée, ni en recul (couvre les lignes 375 et 384)
- [ ] T012 [P] [US1] Test du démarrage à `start_bar` : la position vaut `(start_bar, 0)` (couvre la ligne 242)
- [ ] T013 [P] [US1] Test en pause : deux lectures successives rendent la même valeur (cas limite de la spec)
- [ ] T014 [US1] Vérifier que `snapshot_stream_unchanged` (T001) est **toujours vert** — c'est la preuve que le contrat audio n'a pas bougé et donc qu'aucun bump de `TuningParams` n'est requis (FR-004, SC-003, constitution locale I)
- [ ] T014b [US1] Vérifier que la condition d'émission des rapports de `harmonium_host/src/playback.rs:296` est **inchangée** (FR-005) : le diff ne doit toucher aucune ligne de `send_report`. Rapporter plus souvent est une non-solution documentée (`research.md` R2), pas une amélioration à glisser en passant

**Point de contrôle** : la résolution est là et rien d'autre n'a bougé.

---

## Phase 4 : Accès sans verrou

**But** : rendre la position lisible depuis un callback temps réel (FR-002)

- [ ] T015 Ajouter `transport_position() -> TransportPosition` et `transport_position_handle() -> Arc<AtomicU64>` sur `harmonium_host/src/native_handle.rs`, selon `contracts/transport-position.md`
- [ ] T016 Ajouter les mêmes accesseurs au Handle wasm dans `harmonium_host/src/lib.rs` — le Handle navigateur partage la même architecture découplée (`lib.rs:153`), donc la parité desktop/web ne demande aucun travail spécifique
- [ ] T017 Exposer publiquement la fonction de dépaquetage avec le partage brut, pour qu'un appelant temps réel n'ait pas à reproduire le décalage de bits
- [ ] T018 [P] Test : le chemin de lecture ne contient aucun verrou (SC-004) — la lecture depuis le partage brut est un simple `load`, vérifiable par revue et par un test qui lit sans jamais toucher au mutex du composer

---

## Phase 5 : Preuve et finition

- [ ] T019 `cargo check -p harmonium_core --target wasm32-unknown-unknown` et `cargo check -p harmonium_host --target wasm32-unknown-unknown` verts — aucun `cfg` conditionnel ajouté (constitution locale III)
- [ ] T020 **La preuve observée** : passe manuelle de `quickstart.md` — échantillonner la position bien plus vite qu'un temps et compter 16 valeurs distinctes par mesure en 4/4. Tout peut compiler et rester quantifié au temps si un point d'écriture manque ; seule l'observation le montre. Consigner le relevé dans la PR
- [ ] T021 `cargo test --workspace` vert et CI GitHub verte sur la PR
- [ ] T022 Passer la main pour SC-002, qui ne se vérifie PAS dans ce repo : le critère « une note sur le « et du 4 » porte un temps ≥ 4.5 » se constate côté `harmonium_training`, sur un vrai clavier. Écrire dans la PR ce que l'autre repo doit faire ensuite — lire la position sans verrou dans le callback MIDI et supprimer la dérivation locale `current_step / 4.0 + 1.0` (`harmonium_practice/src/engine.rs:1339`) — et ouvrir la tâche correspondante. Ne rien implémenter ici

---

## Dépendances et ordre

- **Phase 1 avant tout.** T001 doit être vert sur le code non modifié, sinon
  le filet ne prouve rien.
- **Phase 2** : T003 puis T004.
- **Phase 3** : T005 → T006 → T007 → T008 → T009 (la chaîne de compilation),
  puis les tests T010 à T013 en parallèle, puis T014 et T014b.
- **Phase 4** : après la phase 3. T015 et T016 se suivent, T017 avec, T018 après.
- **Phase 5** : à la fin.

### Parallélisme

- T004 seul dans sa phase
- T010 ∥ T011 ∥ T012 ∥ T013 — quatre tests, quatre cas indépendants
- T018 après T017

---

## Stratégie

### Le MVP, c'est la phase 3

Les phases 1 et 2 ne changent aucun comportement : elles posent le filet et
la sémantique. La phase 3 est le changement réel, et son point de contrôle
répond aux deux seules questions qui comptent — la résolution est-elle là, et
le reste est-il intact.

### Ce qui doit rester vrai à chaque étape

- `MeasureSnapshot` inchangé à graine identique — T001 le surveille en continu
- Aucun `cfg(target_arch)` ajouté
- Aucun verrou sur le chemin de lecture temps réel

---

## Notes

- `gen` est un mot-clé réservé en Rust 2024 — utiliser `tgen` ou un nom métier
- La CI GitHub de ce repo est réelle : la PR doit passer
- `Ordering::Relaxed` suffit et c'est déjà l'ordering de l'atomique existant :
  on lit une valeur unique, sans synchroniser d'autres écritures avec elle
- La cadence des rapports (`playback.rs:296`) **ne change pas** — c'est une
  décision, pas un oubli : `research.md` R2 explique pourquoi rapporter plus
  souvent ne résoudrait ni la fraîcheur ni le verrou

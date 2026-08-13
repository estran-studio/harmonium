# Phase 0 — Recherche : position de transport infra-temps

Relevé le 2026-08-13 sur `harmonium` @ `b01c1ad`. Chaque décision est adossée
à une preuve dans le code, citée par fichier et ligne.

---

## R1 — Où la résolution se perd

```rust
// harmonium_host/src/playback.rs:296-298
let current_step = self.playhead.position.step_in_bar(4);
if current_step.is_multiple_of(4) {
    self.send_report();
}
```

Le rapport n'est envoyé que sur les temps entiers. Tout consommateur en aval
lit donc un `current_step` toujours multiple de 4, et les deux endroits qui
en dérivent un temps fractionnaire —
`harmonium_practice/src/engine.rs:1339` et `harmonium_host/src/lib.rs:713`,
tous deux `current_step / 4.0 + 1.0` — ne produisent jamais que 1.0, 2.0,
3.0, 4.0.

La grille interne est pourtant quatre fois plus fine : `step_in_bar(4)`
compte en double-croches. L'information existe à chaque buffer audio ; elle
n'est publiée qu'une fois par temps.

---

## R2 — Pourquoi pas simplement rapporter plus souvent

C'est l'option évidente, et il faut la traiter honnêtement plutôt que
l'écarter d'un revers de main.

**Le coût n'est pas l'argument.** `EngineReport` (`harmonium_core/src/report.rs:121`)
n'alloue pas : ses champs texte sont des `ArrayString<64>`. Passer de 4 à 16
rapports par mesure quadruple une copie mémoire sur le thread audio, sans
allocation. C'est mesurable mais ce n'est pas rédhibitoire.

**Les deux vrais obstacles :**

1. **La fraîcheur reste bornée par le drainage.** `poll_state()`
   (`native_handle.rs:278`) appelle `poll_reports()` puis rend
   `cached_state` — la valeur du dernier rapport *drainé*, pas la position
   courante. Rapporter plus souvent rapproche la valeur sans jamais la
   rendre exacte : elle reste celle du dernier drainage.
2. **Le drainage exige `&mut` et donc le mutex.** `poll_reports()` mute le
   handle ; côté app, `PracticeEngine` garde `handle: Mutex<Option<NativeHandle>>`.
   Un callback MIDI ne peut pas prendre un mutex partagé avec le thread
   audio — c'est précisément le problème constaté côté app, où l'horodatage
   MIDI verrouille aujourd'hui le moteur.

**Décision** : ne pas toucher à la cadence des rapports. Le canal des
rapports reste ce qu'il est — un flux d'état pour l'UI, sondé à 100 ms, où
la résolution au temps suffit. La position précise passe par un canal
dédié, lock-free.

---

## R3 — Le mécanisme : un seul atomique, empaqueté

Le motif existe déjà. `playhead_bar: Arc<AtomicUsize>` est partagé entre le
`PlaybackEngine` (thread audio, écrit) et le `MusicComposer` (thread
principal, lit), créé dans `audio.rs:73` et `audio.rs:174`, écrit
**à chaque buffer** (`playback.rs:293`) et lu sans verrou
(`composer.rs:731`, `native_handle.rs:217`).

**Décision** : remplacer `playhead_bar` par un unique `AtomicU64`
empaquetant la position complète — mesure dans les 32 bits hauts, step dans
la mesure dans les 32 bits bas.

**Pourquoi un seul atomique et non deux.** Deux atomiques — un pour la
mesure, un pour le step — se lisent en deux chargements. Entre les deux, le
thread audio peut franchir une barre : on lit la mesure 5 puis le step 0 de
la mesure 6, ou la mesure 6 puis le step 15 de la mesure 5. Une position qui
n'a jamais existé, et l'erreur se produit **exactement au franchissement de
mesure** — l'instant que toute cette fonctionnalité existe pour rendre
exact. Un seul chargement rend une paire par construction cohérente.

**Pourquoi remplacer plutôt qu'ajouter.** Garder `playhead_bar` à côté du
nouvel atomique créerait deux sources de vérité pour la mesure, qui
pourraient diverger. `playhead_bar()` devient un dérivé du champ empaqueté
(constitution II : on migre, on ne juxtapose pas).

**Vérifié** : `AtomicU64` compile pour `wasm32-unknown-unknown` (testé au
compilateur, pas supposé). La contrainte de la constitution locale III est
donc tenue sans `cfg` conditionnel.

---

## R4 — Lecture depuis le callback MIDI, sans verrou

Le consommateur critique est l'horodatage MIDI côté app, qui tourne dans le
callback `midir`.

**Décision** : exposer l'`Arc<AtomicU64>` par un accesseur, que l'appelant
clone **une fois** au démarrage de l'entrée MIDI et capture dans sa closure.
Le callback ne fait plus qu'un `load(Relaxed)`.

Le motif est déjà celui de l'app : `start_midi_input`
(`harmonium_training/src-tauri/src/midi_input.rs`) capture déjà le
`TimeAnchor`, les notes attendues et le `live_sender` au démarrage, une
seule fois, sous verrou, pour que le callback n'ait plus à verrouiller.

`Ordering::Relaxed` suffit : on lit une valeur unique, sans synchroniser
d'autres écritures avec elle. C'est l'ordering déjà employé pour
`playhead_bar`.

---

## R5 — Le web est servi par le même changement

`harmonium_host/src/lib.rs:153` : *« Wasm Handle uses the decoupled
MusicComposer + PlaybackEngine architecture »*. Les deux builds partagent
donc le `PlaybackEngine`, et les deux chemins de création de l'atomique
(`audio.rs:73` et `audio.rs:174`) couvrent le natif et le wasm.

**Conséquence** : aucun travail spécifique au navigateur. Le Web MIDI, quand
il arrivera, lira la même position par le même accesseur. La parité
desktop/web exigée par W2 est obtenue sans effort supplémentaire.

`timeline_engine.rs` est l'ancien chemin, non utilisé par le Handle wasm ni
par `NativeHandle` : hors périmètre.

---

## R6 — Le contrat de snapshot n'est pas touché

Constitution locale I : un changement qui altère le `MeasureSnapshot` à
graine et entrées identiques est cassant et exige un bump de version de
`TuningParams`.

**Ce changement n'y touche pas.** Il ajoute une lecture de la position de
lecture ; il ne modifie ni la génération, ni la timeline, ni le snapshot.
Aucun bump de version, aucune migration.

**À vérifier par un test** plutôt qu'à affirmer : rendre N mesures à graine
fixe avant et après, et comparer les snapshots.

---

## R7 — Points d'écriture à couvrir

`playhead_bar` est écrit en quatre endroits ; tous doivent écrire la
position empaquetée, sinon la valeur se désynchronise silencieusement :

| Ligne | Contexte | Step à écrire |
|---|---|---|
| `playback.rs:242` | démarrage à `start_bar` | 0 |
| `playback.rs:293` | chaque buffer | position courante |
| `playback.rs:375` | seek | 0 |
| `playback.rs:384` | seek (seconde voie) | 0 |

Le seek pose la position au début de la mesure visée : c'est déjà la
sémantique de `playhead_bar` aujourd'hui, on la rend simplement explicite
sur le step.

---

## R8 — Ce que la résolution donne réellement

`ticks_per_beat = 4` → la double-croche. En 4/4 : 16 positions par mesure,
temps fractionnaires par pas de 0,25.

C'est suffisant pour les règles du scoring W2, qui distinguent les demies de
temps (`[B, B+0.5)` contre `[B+0.5, B+1)`). Ça ne l'est pas pour du micro-
timing expressif — le swing, le laid-back de quelques millisecondes. Ce
n'est pas l'objet : W2 mesure où tombe une note dans la grille, pas la
finesse de son placement contre le beat. Si un jour on veut mesurer le
swing, ce sera un autre changement, avec sa propre justification.

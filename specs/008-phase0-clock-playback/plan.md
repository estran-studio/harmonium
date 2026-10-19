# Phase 0 — Horloge et playback

**Statut** : en cours | **Date** : 2026-10-18 | **Crates** : `harmonium_host` (principal), `harmonium_core` (types), `harmonium_cli` (vérification)

## Problème

Le jeu a besoin d'une horloge de beat qu'il peut interroger à chaque frame
et **prédire** : l'esquive est un pas de gigue calé sur la pulse, le combo
récompense les actions synchronisées (fenêtre ±40 ms). Aujourd'hui :

- La position `(bar, step)` est publiée après chaque pas (`PlaybackEngine::tick`,
  `playback.rs:336`), donc lue avec 0 à 1 pas de retard (125 ms à 120 BPM),
  plus la latence du périphérique. Aucun lien avec le temps mur : le
  `cpal::OutputCallbackInfo` est ignoré (`audio.rs:139`).
- « Beat » = `step / 4 + 1` : la noire. La pulse dansée est la blanche en
  reel (2/2), la noire pointée en gigue (6/8). Rien n'expose temps fort /
  temps faible.
- Le swing n'existe pas (le hi-hat PerfectBalance décalé par `tension`
  n'est pas un swing). Un reel sans croches inégales n'est pas un reel.
- Le tempo n'est relu qu'au chargement d'une mesure (`playback.rs:299`) :
  « le tempo qui monte avec la colère » arrive avec jusqu'à 2 s de retard.
- Le thread audio prend un `Mutex` sur `SharedPages` et **clone** la
  mesure dans le callback (`playback.rs:291-292`) : verrou + allocation sur
  le chemin chaud.
- `set_bpm` est borné 70–180 (`composer.rs`) : un reel se joue à 210–260 à
  la noire.

Cette phase ne génère pas une note de trad. Elle prouve l'horloge d'abord :
si le beat ne tient pas, le reste ne sert à rien.

## Décisions

### D1 — `TransportClock` prédictif, publié par le callback

Le callback publie, via `triple_buffer` (déjà dépendance workspace), un
`TransportClock` :

```rust
pub struct TransportClock {
    pub position: TransportPosition,   // bar, step (inchangé)
    pub frames_at_last_step: u64,      // compteur global de frames au dernier tick
    pub frames_per_step: u32,          // après swing (voir D3)
    pub buffer_start_frame: u64,       // premier frame du buffer en cours
    pub buffer_start_instant: Instant, // cpal playback timestamp → Instant
    pub sample_rate: u32,
}
```

L'`AtomicU64` packé reste pour les lecteurs qui ne veulent que la position
(spec 006, contrat inchangé). `TransportClock` s'ajoute à côté parce que ce
n'est pas la même donnée : position vs horloge. Un seul écrivain (callback),
lecture sans verrou côté jeu.

### D2 — `BeatInfo`, l'API gameplay

Calculé côté lecteur à partir de `TransportClock` + `PulseSpec` :

```rust
pub struct PulseSpec { pub steps_per_pulse: u8, pub pulses_per_bar: u8 }  // reel 2/2: 8, 2 ; gigue 6/8: 6, 2 ; valse 3/4: 4, 3
pub struct BeatInfo {
    pub bar: usize,
    pub pulse: u8,            // 1-based dans la mesure
    pub phase: f32,           // 0.0..1.0 dans la pulse, interpolé en frames
    pub is_downbeat: bool,    // pulse == 1
    pub is_backbeat: bool,    // pulse paire
    pub next_pulse_at: Instant,
    pub pulse_period: Duration,
}
impl NativeHandle { pub fn beat_info(&self, now: Instant) -> BeatInfo }
```

`PulseSpec` vient de `MusicalParams` (champ `pulse`, défaut dérivé de la
signature : 4/4 → noire, 2/2 → blanche, 6/8 → noire pointée, 3/4 → noire).
Le tempo reste à la noire (constitution).

### D3 — Swing comme table de timing dans le playback

`MusicalParams.swing_ratio: f32` (0.5 = droit, 0.6 ≈ reel, borné 0.5–0.75),
dans `TuningParams` pour le défaut par style. Capturé dans
`Measure.swing_ratio` et `MeasureSnapshot.swing_ratio` à la génération.
`PlaybackEngine` dérive deux longueurs de pas : pas pairs (sur la croche)
allongés, pas impairs raccourcis, somme conservée par paire. La position
publiée reste sur la grille ; seul le timing audio bouge.

Rejeté : passer la grille à 12 ticks/temps. Touche `[usize;16]`,
`rhythm_steps`, l'export, le contrat snapshot, pour un gain que la table de
timing donne sans rien casser.

### D4 — Rampe de tempo par pulse dans le playback

`PlaybackCommand::SetTempo { bpm: f32, ramp_pulses: u8 }` : interpolation
linéaire de `frames_per_step` à chaque pulse, immédiate (ring buffer de
commandes, déjà traité en tête de callback). Le tempo des mesures générées
devient la valeur *nominale* ; `EngineReport.musical_params.bpm` reflète le
tempo audible. Le snapshot garde le tempo nominal : l'horloge
(`BeatInfo`) est la vérité pour le jeu, pas le snapshot.

### D5 — Handoff de mesures sans verrou ni allocation

`SharedPages: Arc<Mutex<Vec<Measure>>>` disparaît. Remplacé par :

- composer → playback : `rtrb::Producer<Box<Measure>>` (capacité 16) ;
- playback → composer : `rtrb::Producer<Box<Measure>>` de retour, pour que
  la désallocation se fasse sur le thread principal (`Measure` contient
  `HashMap`/`Vec`) ;
- `PlaybackCommand::DropFrom { bar }` pour `invalidate_future` et `seek` :
  le playback renvoie les mesures `>= bar` au composer, qui régénère.

Le playback garde une `ArrayVec<Box<Measure>, 16>` ordonnée par index.
Underflow inchangé : `tick()` retourne sans jouer.

### D6 — Hygiène

- `set_bpm` borné 40–300.
- `Playhead::track_cursors` dimensionné par `TrackId::ALL.len()`, pas `4`.
- `Playhead::load_measure` utilise le `sample_rate` réel, pas `44100.0`
  codé en dur (`pointers.rs:120`).

## Fichiers

```text
harmonium_core/src/params.rs        swing_ratio, pulse: PulseSpec
harmonium_core/src/tuning.rs        swing_ratio par défaut (bump version TuningParams)
harmonium_core/src/timeline/mod.rs  Measure.swing_ratio
harmonium_core/src/report.rs        MeasureSnapshot.swing_ratio
harmonium_core/src/timeline/pointers.rs  cursors N pistes, sample_rate réel
harmonium_host/src/clock.rs         TransportClock, PulseSpec → BeatInfo (nouveau)
harmonium_host/src/playback.rs      swing map, rampe tempo, handoff rtrb, publication clock
harmonium_host/src/composer.rs      producteur de mesures, DropFrom, clamp BPM
harmonium_host/src/audio.rs         cpal timestamps → clock, câblage des rings
harmonium_host/src/native_handle.rs beat_info(), set_tempo(), set_swing()
harmonium_cli/src/repl.rs           commandes `beat`, `swing`, `tempo <bpm> [pulses]`
harmonium_host/tests/beat_info_tests.rs  (nouveau)
```

## Acceptation (observable)

1. **Prédiction exacte hors ligne** : avec `create_offline_engine`, pour
   chaque buffer, `next_pulse_at` prédit depuis le `TransportClock` publié
   au buffer *k* tombe sur le frame exact où le playhead franchit la pulse
   au buffer *k+n*. Erreur : 0 frame, sur 64 mesures, en 4/4, 2/2, 6/8,
   3/4, avec et sans swing, pendant une rampe de tempo.
2. **Swing mesurable** : à `swing_ratio = 0.6`, l'écart entre deux NoteOn
   consécutifs sur croches vaut 0.6 / 0.4 de la noire, à ±1 frame.
3. **Rampe** : `SetTempo { 240, ramp_pulses: 8 }` atteint 240 BPM
   exactement 8 pulses après réception ; monotone.
4. **Non-régression** : `snapshot_stream_tests` et goldens inchangés pour
   un même seed (champ `swing_ratio` à sa valeur par défaut).
5. **RT** : `cargo test` en debug ne panique pas via `RTCheckAllocator`
   pendant le handoff, le seek et l'invalidation — preuve que D5 a retiré
   l'allocation du callback.
6. **À l'oreille** : `harmonium-cli`, commande `beat` affichant un
   métronome visuel calé sur `BeatInfo` ; `swing 0.6` audible sur un
   pattern de croches ; `tempo 240 8` sans clic ni saut.

## Risques

- `cpal` ne fournit pas de timestamp fiable sur tous les backends
  (CoreAudio oui ; ALSA parfois `None`). Repli : `Instant::now()` à
  l'entrée du callback, documenté comme dégradé, jamais silencieux.
- Les goldens JSON comparent des snapshots sérialisés : ajouter
  `swing_ratio` avec `#[serde(default)]` et vérifier que la comparaison
  tolère le champ, sinon régénérer dans le même commit (constitution I).
- D5 est le changement le plus invasif de la phase : il touche `seek`,
  `set_loop`, `invalidate_future`, `reset`. Chacun a déjà un test dans
  `transport_position_tests.rs` et `deterministic_seek_tests.rs` ; ils
  doivent rester verts sans modification.

## Suite

Phase 1 — reel minimal identifiable : pistes `Fiddle/Guitar/Bass/FeetHeel/FeetToe`,
`TuneGenerator` AABB, banque de cellules, `TradHarmonizer`, podorythmie,
articulations rendues en temps-échantillon (`GraceCut`, `BowedTriplet`,
`DoubleStop`) dans le `Playhead`. Son plan s'écrit quand la phase 0 est
verte.

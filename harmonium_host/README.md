# Harmonium Host

`harmonium_host` (crate `harmonium`) wires the pure music logic of `harmonium_core` to the oxisynth renderer of `harmonium_audio` and to a `cpal` output stream. It exposes one native Rust API, `LiveHost`, the single live host of the engine: the REPL (`harmonium_cli`) and the studio drive the same object. There is no binary.

## Architecture

Generation and playback are decoupled across two threads, communicating only through lock-free structures:

```mermaid
graph LR
    Composer[MusicComposer (main thread)] -- staged Box<Measure> (rtrb) --> Playback[PlaybackEngine (cpal callback)]
    Playback -- returned Box<Measure> (rtrb) --> Composer
    Composer -- PlaybackCommand (rtrb) --> Playback
    Playback -- EngineReport (rtrb) --> Composer
    Playback -- AtomicU64 (bar, step) --> Composer
```

*   **`MusicComposer`** (`src/composer.rs`): main-thread generation. Owns the `TradGenerator` (piloté par le profil de style actif, `reel_quebecois` par défaut), `Writehead`, `MusicalParams` and the `EmotionMapper` (from `harmonium_core::mapper`). `generate_ahead()` stages `Box<Measure>`s through an SPSC ring (`composer → playback`, capacity 16) ahead of the transport, drains the return ring (deallocating played boxes on the main thread), and emits `MeasureSnapshot`s.
*   **`PlaybackEngine`** (`src/playback.rs`): audio-thread playback. Reads staged measures from the ring into an ordered prefetch window, ticks the `Playhead`, feeds `AudioEvent`s to the `AudioRenderer` (oxisynth), and handles recording (WAV / MIDI / MusicXML). Played boxes are handed back through the return ring (`playback → composer`) so they are only ever dropped on the main thread. No allocation in steady state (enforced in debug builds by `RTCheckAllocator`).
*   **Transport position**: packed `(bar, step)` in a single `Arc<AtomicU64>` (`playback::pack` / `playback::unpack`), written by the audio thread and read lock-free by anyone holding the `Arc`. `TransportPosition::from_packed` derives the beat on a `TICKS_PER_BEAT` grid.
*   **`audio::create_timeline_stream(sf2_bytes, feet_sf2_bytes, odin_presets, buffer_frames)`** (`src/audio.rs`): builds the whole graph plus a `cpal::Stream`. `audio::create_offline_engine(sf2_bytes, feet_sf2_bytes, odin_presets, sample_rate)` builds the same graph without a stream, for tests and offline rendering. `sf2_bytes = None` runs oxisynth without a bank (silence, but the engine and snapshot stream work); `feet_sf2_bytes` charge la banque de pieds dédiée (Heel/Toe → canaux 3/4).
*   **`LiveHost`** (`src/live_host.rs`): owns the stream, the composer, the command producer, the report consumer, the transport position and the clock reader.
    *   `LiveHost::start(sf2, feet_sf2, odin_presets, buffer_frames)` / `start_paused(…)` open the default output device (`buffer_frames`: device buffer in frames, `None` = platform default); `LiveHost::detached(…, sample_rate)` builds a host without device and hands back the `PlaybackEngine` for the caller to run (tests, rendering).
    *   **`apply(EngineCommand)`** exécute les commandes. Une modification de génération invalide les mesures futures, garde celle du playhead et évince les mesures remplacées du playback. Mixeur, boucle, tempo audible et enregistrement passent par la file de commandes audio ; `NoteOn`, `NoteOff`, `AllNotesOff` et `SetLiveProgram` rejoignent la voie jouée décrite ci-dessous.
    *   **`pump()`** is the generation driver: it composes ahead of the playhead, frees the played boxes, drains the audio thread's reports and returns the `MeasureSnapshot`s composed since the last call. The caller decides where it runs (a thread every ~50 ms in the REPL and the native studio, a Worker timer in the web studio). `apply` never composes: the replacement bars come from the next `pump`.
    *   **Published state**: `poll_state()` (last `EngineReport`), `transport_position()` (lock-free), `beat_info(now)` / `take_clock_reader()` / `restore_clock_reader()` (predictive clock), `take_recordings()`, `musical_params()`, `emotions()`, `is_emotion_mode()`, `session_seed()`, `session_config()`, `profile_bar_info(bar)`.
    *   `apply` and `pump` take `&mut self`: a host shared between threads sits behind a `Mutex` (as in the REPL). On wasm the output stream is not `Send`, and neither is the host: it lives on the page's one thread.
*   **`osc`** (`src/osc.rs`): native OSC listener on UDP (`rosc`). `/harmonium/params <arousal> <valence> <density> <tension>` (4 floats) becomes `EngineCommand::SetEmotionParams`, handed to a callback that applies it to the host: `OscListener::spawn(osc::DEFAULT_ADDR, |command| …)`. `harmonium-cli --osc` and the native studio (Live screen, Engine → OSC listener) open it on `127.0.0.1:8080`.

### Notes jouées en direct (MIDI, studio, jeux)

Une seule voie sonore, deux entrées : `LiveHost::apply(EngineCommand::NoteOn/NoteOff)`
et une poignée `LiveNoteSender` obtenue **une fois** par `take_note_sender()`.
La poignée est `Send`, non `Clone`, et ne référence pas l'hôte : le thread MIDI
ne prend pas le verrou éventuel de `LiveHost` pendant que `pump()` compose.

```rust,no_run
use harmonium::LiveHost;
use harmonium_core::EngineCommand;

# fn exemple(mut host: LiveHost) -> Result<(), Box<dyn std::error::Error>> {
host.apply(EngineCommand::SetLiveProgram(0))?; // programme SoundFont, index 0-based
let mut notes = host.take_note_sender().ok_or("poignée déjà prise")?;
let midi = std::thread::spawn(move || -> Result<(), harmonium::LiveNoteError> {
    // Depuis les rappels du périphérique MIDI, sans accès à LiveHost :
    notes.note_on(0, 60, 100)?;
    notes.note_off(0, 60)?;
    notes.all_notes_off()?;
    Ok(())
});
// Entrée équivalente pour les actions de l'interface ou les systèmes de jeu :
host.apply(EngineCommand::NoteOn { channel: 0, note: 64, velocity: 100 })?;
host.apply(EngineCommand::NoteOff { channel: 0, note: 64 })?;
# midi.join().map_err(|_| "thread MIDI interrompu")??;
# Ok(())
# }
```

- **Transport borné** : deux files SPSC `rtrb` fixes de 256 événements, une
  par producteur (`apply`, MIDI), drainées par le même callback vers le même
  synthé. Ni verrou ni allocation à l'envoi ou à la consommation. La poignée
  n'est pas clonable pour conserver un seul producteur par file.
- **Ordre et latence** : FIFO par entrée ; à chaque début de callback, les
  commandes jouées de l'hôte précèdent les événements MIDI. Le nombre à
  dépiler est capturé pour les deux entrées avant leur drain (au plus 512).
  Tout événement déjà publié à ce point est appliqué dans ce callback,
  sans attendre `pump` ni une frontière musicale. Pas d'ordre temporel global
  entre les deux producteurs ; un NoteOn MIDI peut suivre un AllNotesOff de
  l'hôte au même callback. Oxisynth conserve un bloc PCM interne de 64 frames :
  jusqu'à 63 frames de délai de synthèse, distinct du transport par callback.
- **Refus explicite** : `LiveNoteError::Full` ou `InvalidMidiData` ; `apply`
  retourne l'erreur en texte. Aucun blocage, aucune perte silencieuse. Un NoteOff
  refusé laisse la note tenue : garder le relâchement à réessayer, ou réessayer
  `all_notes_off` après le prochain callback. Aucun envoi n'est consommé tant
  que la sortie est suspendue.
- **MIDI** : canaux `0..16`, hauteur/vélocité/programme `0..128`, tous 0-based ;
  vélocité zéro = NoteOff. Les seize canaux, y compris 9, sont mélodiques.
  `set_program(u8)` / `SetLiveProgram(u8)` choisit un programme global de banque 0
  pour les prochaines attaques (défaut 0), sans modifier les notes déjà tenues.
- **Voix indépendantes** : synthé SoundFont dédié, 256 voix propres préallouées
  (plusieurs zones possibles par note), sans vol de voix aux pistes générées.
  Mutes, gains, programmes, routages et seeks des pistes ne le coupent pas.
  NoteOff respecte le relâchement SF2 ; AllNotesOff coupe la seule voie jouée,
  y compris son PCM précalculé. Le volume et le mute de sortie restent globaux.
- **Périmètre** : la voie fonctionne sans mesure composée ; ni snapshot, ni
  timeline, ni compositeur, ni export WAV/MIDI/MusicXML ne reçoit ces notes.
  Le retour direct est ajouté après l'enregistreur et avant le volume de sortie.
  Pas de réverbération ou chorus sur cette voie. Sans banque valide, silence ;
  avec `odin`, les notes jouées restent SoundFont (aucun preset Odin dans cette API).
  La banque est chargée une seconde fois à la construction, hors callback.
- **Clients** : le studio remettra la poignée au rappel du clavier MIDI et
  utilisera les commandes pour son clavier à l'écran ; Bevy pourra la déplacer
  dans sa tâche MIDI et envoyer les mêmes commandes depuis ses systèmes.
  Aucun branchement studio/Bevy n'est inclus dans E4. Sur le web, les rappels
  Web MIDI peuvent posséder la poignée sur le même thread, sans créer de thread.

Les mesures de `pump`, les preuves hors ligne et le patch oxisynth sans
exemption d'allocation figurent dans le
[compte rendu E4](../specs/018-grille-et-moteur-neutre/e4-notes-jouees.md).

### Zones Odin et hôte direct

Le schéma des zones et les règles de sélection sont dans
[`profiles/README.md`](../profiles/README.md#instrument-odin-par-piste-option-odin).
`LiveHost::pump()` entretient aussi les transactions Odin en attente.
Un hôte qui emploie directement `create_offline_engine` appelle périodiquement
`composer.send_odin(&mut commands)` sur son thread principal, après la génération :
cela récupère les instances rendues par SPSC et réessaie une installation
suspendue par un fondu ou la contrepression. À état inchangé, aucun nouveau
moteur n'est construit. Le callback choisit la frontière sur la position de
la mesure effectivement entrée, jamais sur une mesure préchargée.

Avec `odin`, `PlaybackEngine::odin_diagnostics()` fournit un instantané fixe
des attaques/fins par instance et des positions d'installation/retrait.
L'historique des retraits est borné à 128 instances ; `history_overflow`
signale explicitement qu'un audit exhaustif n'est plus possible.

## Features

*   `standalone` (default): enables `cpal`, `rosc`, and the `audio`, `live_host` and `osc` modules. `composer` and `playback` are always available.

## WebAssembly (`wasm32-unknown-unknown`)

The engine compiles for the browser and produces the same snapshots as natively (checked in CI: `make wasm/check`, `make test/determinism`).

*   `cpal` uses its `wasm-bindgen` feature (WebAudio); `getrandom` its `js` feature (`thread_rng` of the composer); the `osc` module and `rosc` do not exist on wasm.
*   `Instant` is `web_time::Instant`: `std::time::Instant` on native (same type, nothing changes), `performance.now()` on wasm, where `std::time::Instant::now()` panics. `BeatInfo::next_pulse_at` and `process_buffer(…, Instant)` use it.
*   No thread: the recorder's encoder is handed to the host (`EngineControls::recorder`) and `LiveHost::pump()` advances it, so recordings finish through `pump` followed by `take_recordings()`. Its overflow policy is always drop-and-count (`LiveHost::detached` does not block on wasm). `create_offline_engine` is native only.
*   No random draw on `usize` anywhere in the engine: rand 0.8 samples a `usize` with the pointer-width integer, which would consume the ChaCha8 stream differently on a 32-bit target. Draws are `u64` (`harmonium_core::algo::RngCore::next_range_u64`).
*   `odin`: per-track Odin synth (GPLv3, see the root README); presets are provided by the caller as `OdinPresetSource { name, bytes }`.

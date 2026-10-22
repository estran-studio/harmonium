# Harmonium Host

`harmonium_host` (crate `harmonium`) wires the pure music logic of `harmonium_core` to the oxisynth renderer of `harmonium_audio` and to a `cpal` output stream. It exposes a native Rust API (`NativeHandle`) and a CLI binary.

## Architecture

Generation and playback are decoupled across two threads, communicating only through lock-free structures:

```mermaid
graph LR
    Composer[MusicComposer (main thread)] -- staged Box<Measure> (rtrb) --> Playback[PlaybackEngine (cpal callback)]
    Playback -- returned Box<Measure> (rtrb) --> Composer
    Composer -- PlaybackCommand (rtrb) --> Playback
    Playback -- EngineReport (rtrb) --> Composer
    Playback -- AtomicU64 (bar, step) --> Composer
    MIDI[LiveMidiSender] -- LiveMidiEvent (rtrb) --> Playback
```

*   **`MusicComposer`** (`src/composer.rs`): main-thread generation. Owns the `TradGenerator` (piloté par le profil de style actif, `reel_quebecois` par défaut), `Writehead`, `MusicalParams` and the `EmotionMapper` (from `harmonium_core::mapper`). `generate_ahead()` stages `Box<Measure>`s through an SPSC ring (`composer → playback`, capacity 16) ahead of the transport, drains the return ring (deallocating played boxes on the main thread), and emits `MeasureSnapshot`s.
*   **`PlaybackEngine`** (`src/playback.rs`): audio-thread playback. Reads staged measures from the ring into an ordered prefetch window, ticks the `Playhead`, feeds `AudioEvent`s to the `AudioRenderer` (oxisynth), and handles recording (WAV / MIDI / MusicXML). Played boxes are handed back through the return ring (`playback → composer`) so they are only ever dropped on the main thread. No allocation in steady state (enforced in debug builds by `RTCheckAllocator`).
*   **Transport position**: packed `(bar, step)` in a single `Arc<AtomicU64>` (`playback::pack` / `playback::unpack`), written by the audio thread and read lock-free by anyone holding the `Arc`. `TransportPosition::from_packed` derives the beat on a `TICKS_PER_BEAT` grid.
*   **`audio::create_timeline_stream(sf2_bytes, feet_sf2_bytes)`** (`src/audio.rs`): builds the whole graph plus a `cpal::Stream`. `audio::create_offline_engine(sf2_bytes, feet_sf2_bytes, sample_rate)` builds the same graph without a stream, for tests and offline rendering. `sf2_bytes = None` runs oxisynth without a bank (silence, but the engine and snapshot stream work); `feet_sf2_bytes` charge la banque de pieds dédiée (Heel/Toe → canaux 3/4).
*   **`NativeHandle`** (`src/native_handle.rs`): owned handle over the stream, composer and command producer. `NativeHandle::start(sf2)` / `start_paused(sf2)`, transport control, emotion/mixer setters, `take_live_midi_sender()` for monitoring the player's own MIDI input on `MONITOR_CHANNEL`.

## Features

*   `standalone` (default): enables `cpal`, `audio`, `native_handle` and the binary. `composer` and `playback` are always available.

## CLI

Binary `harmonium` (`src/main.rs`):

```bash
cargo run --release --bin harmonium -- [OPTIONS] [SOUNDFONT.sf2]
```

Options:

*   `--osc` — listen on UDP `127.0.0.1:8080`; `/harmonium/params <arousal> <valence> <density> <tension>` (4 floats) updates the emotion mapper.
*   `--simulate` — randomly change emotions every 5 s.
*   `--record-wav [PATH]`, `--record-midi [PATH]`, `--record-musicxml [PATH]` — record; finalized on Ctrl+C or when `--duration <SECONDS>` elapses.
*   `--soundfont, -s [PATH]` — chemin vers une banque SoundFont (.sf2).

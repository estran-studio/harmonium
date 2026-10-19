# Harmonium CLI

Interactive command-line interface for the Harmonium generative music engine.

## Overview

The Harmonium CLI provides a REPL (Read-Eval-Print Loop) for testing and controlling the Harmonium engine. It validates the new command/report queue architecture before the frontend rebuild (Phase 3).

## Features

- ✅ **Interactive REPL** with command history and tab completion
- ✅ **Smart autocomplete** - Tab completion for commands, parameters, and values
- ✅ **Real-time state display** showing BPM, current chord, bar, beat
- ✅ **All 40+ engine parameters** controllable via commands
- ✅ **Emotion mode** - Control via arousal, valence, density, tension
- ✅ **Relative adjustments** - Increment/decrement emotion values (e.g., `emotion a+0.1 t-0.2`)
- ✅ **Direct mode** - Technical control of all parameters
- ✅ **Live pattern visualization** - See Euclidean rhythm patterns
- ✅ **Command help system** - Type `help` for available commands
- ✅ **Colored output** - Easy to read terminal interface

## Installation

```bash
cd harmonium/harmonium_cli
cargo build --release
```

The binary will be at `../../target/release/harmonium-cli`

## Usage

### Quick Start

```bash
# Using make (recommended)
make run

# Or directly
harmonium-cli
```

### With SoundFont

```bash
harmonium-cli --soundfont path/to/soundfont.sf2
```

Without `--soundfont`, the CLI loads `assets/test.sf2` from the current directory if it exists; otherwise it runs without a SoundFont (silent) and prints a warning.

### Keyboard Shortcuts

- **Tab** - Autocomplete commands, parameters, and values
- **Ctrl+C** - Quit immediately
- **Ctrl+D** - Quit (EOF)
- **↑/↓ arrows** - Navigate command history

## Commands

### Control Modes

```bash
emotion                          # Switch to emotion mode
emotion 0.9 0.5 0.7 0.6         # Set emotional parameters (arousal valence density tension)
emotion a+0.1 v-0.2 d+0.05      # Relative adjustments (a=arousal, v=valence, d=density, t=tension)
direct                           # Switch to direct/technical mode
```

**Emotion Shortcuts:**
- `a` = arousal (energy/activation)
- `v` = valence (positive/negative)
- `d` = density (rhythmic complexity)
- `t` = tension (harmonic dissonance)

**Examples:**
```bash
emotion a+0.1        # Increase arousal by 0.1
emotion t-0.2        # Decrease tension by 0.2
emotion a+0.1 d-0.05 # Multiple adjustments
```

### Global Parameters

```bash
set bpm 140                      # Set BPM (70-180)
set volume 0.8                   # Set master volume (0-1)
set time 4/4                     # Set time signature
```

### Rhythm Parameters

```bash
set rhythm_mode euclidean        # euclidean | perfect | classic
set steps 16                     # Pattern steps (4-192)
set pulses 4                     # Pattern pulses (1-16)
set rotation 2                   # Pattern rotation (0-15)
set density 0.7                  # Rhythmic density (0-1)
set rhythm_tension 0.5           # Rhythmic tension (0-1)
```

### Harmony Parameters

```bash
set harmony_mode driver          # basic | driver
set valence 0.5                  # Positive/negative emotion (-1 to 1)
set harmony_tension 0.3          # Harmonic dissonance (0-1)
```

### Melody & Voicing

```bash
set smoothness 0.7               # Melodic smoothness (0-1)
set octave 4                     # Melody octave (3-6)
set voicing_density 0.5          # Chord voicing density (0-1)
set voicing_tension 0.3          # Voicing tension (0-1)
```

### Module Toggles

```bash
enable rhythm                    # Enable rhythm module
disable harmony                  # Disable harmony module
enable melody                    # Enable melody module
disable voicing                  # Disable voicing module
```

### Mixer

```bash
set gain 0 0.8                   # Set channel 0 gain to 0.8
set mute 2                       # Mute channel 2 (snare)
set unmute 2                     # Unmute channel 2
```

### Recording

```bash
record wav                       # Start WAV recording
record midi                      # Start MIDI recording
record musicxml                  # Start MusicXML recording
stop                             # Stop all recordings
stop musicxml                    # Stop specific format
```

**Note**: Recordings are saved to the current directory when stopped.

### Utility

```bash
state | show | status            # Show current engine state
reset                            # Reset engine to defaults
help                             # Show help message
help set                         # Show help for specific command
quit | exit                      # Exit the CLI
```

## Examples

### Example Session

```bash
$ harmonium-cli

╔════════════════════════════════════════════════════════╗
║                                                        ║
║     🎵  Harmonium CLI - Interactive Music Engine  🎵   ║
║                                                        ║
╚════════════════════════════════════════════════════════╝

  Type help for available commands
  Type quit or exit to exit

harmonium 120bpm Imaj7 [bar:1] set bpm 140
[OK] BPM set to 140

harmonium 140bpm Imaj7 [bar:3] emotion 0.9 0.5 0.7 0.6
[OK] Emotions set: A=0.90 V=0.50 D=0.70 T=0.60

harmonium 169bpm Imaj7 [bar:5] emotion a+0.05 t-0.1
[OK] Emotions set: A=0.95 V=0.50 D=0.70 T=0.60

harmonium 169bpm Imaj7 [bar:7] set rhythm_mode per[TAB]
harmonium 169bpm Imaj7 [bar:7] set rhythm_mode perfect
[OK] Rhythm mode set to PerfectBalance

harmonium 169bpm Imaj7 [bar:9] enable voicing
[OK] Voicing enabled

harmonium 169bpm Imaj7 [bar:10] show

╔══════════════════════════════════════════════════════╗
║              ENGINE STATE                            ║
╚══════════════════════════════════════════════════════╝

TIMING:
  BPM: 169.0
  Time Signature: 4/4
  Bar: 10
  Beat: 1
  Step: 1

HARMONY:
  Mode: Driver
  Current Chord: Imaj7
  Progression: Happy Energy (4 chords)
  Key: C PentatonicMajor

RHYTHM:
  Mode: PerfectBalance
  Primary: 16 steps, 4 pulses, rotation 0
  Secondary: 12 steps, 6 pulses, rotation 0
  Pattern: ████·███·███·███|················

MODULES:
  Rhythm:  ON
  Harmony: ON
  Melody:  ON
  Voicing: ON

harmonium 169bpm Imaj7 [bar:12] quit
Goodbye!
```

## Notes

- **Emotion Mode**: The CLI includes a built-in EmotionMapper that translates emotional parameters (arousal, valence, density, tension) into musical parameters (BPM, rhythm patterns, harmony, etc.) in real-time.
- **Logging**: Engine logs are automatically suppressed in CLI mode to keep the REPL clean. Only critical errors are shown.
- **Exit**: Press Ctrl+C or Ctrl+D to exit gracefully.
- **History**: Command history is saved to `~/.harmonium_history` and persists between sessions.

## Architecture

The REPL drives the host's decoupled engine directly:

```
┌─────────────┐  setters (Mutex<MusicComposer>)   ┌─────────────────┐
│   REPL      ├──────────────────────────────────→│  MusicComposer  │ main thread: generates measures ahead
│             │                                   └────────┬────────┘
│             │  PlaybackCommand (SPSC 256)                │ Measure
│             ├─────────────────────────────┐              ↓
│             │  EngineReport (SPSC 256)    │     ┌─────────────────┐
│             │←────────────────────────────┴─────┤  PlaybackEngine │ cpal callback: plays steps, publishes (bar, step)
└─────────────┘                                   └─────────────────┘
```

- Structural parameters (tempo, emotions, harmony) take effect on the next generated measure.
- Gains, mute, GM program changes and seek are immediate (ring buffer, processed at the top of the audio callback).
- No allocation in the audio callback: `RTCheckAllocator` panics on violation in debug builds.

## Development

### Running Tests

```bash
cargo test -p harmonium-cli
```

### Building

```bash
cargo build -p harmonium-cli
```

## Status

**Phase 2 Complete!** ✅

The CLI successfully validates:
- ✅ All 40+ parameters controllable
- ✅ Emotion ↔ Direct mode switching
- ✅ Real-time state updates
- ✅ Command/report queue architecture
- ✅ No audio dropouts

Ready for **Phase 3: Frontend Rebuild**

## License

MIT

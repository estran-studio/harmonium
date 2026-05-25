# harmonium (core) Constitution

This document extends the [meta constitution] in `harmonium_specs/.specify/memory/constitution.md`.
On conflict, the meta document wins. The principles below are repo-local additions
specific to the engine workspace.

[meta constitution]: https://github.com/estran-studio/harmonium_specs/blob/main/.specify/memory/constitution.md

## Repo-Local Principles

### I. The Audio Contract is `MeasureSnapshot`

The engine's only commitment to the outside world is the stream of
`MeasureSnapshot` values it produces, with bar/beat/tick positions in the
documented coordinate system (1-based bars and beats; tick is step-grid
multiples). Internal layout (Conductor, Sequencer, HarmonicDriver,
MelodyGenerator, fractal noise, tuning params) is implementation detail
and may change. The contract is the snapshot stream.

When a change to the engine would alter the snapshot for the same seed +
inputs, it is a breaking change and requires explicit migration (a version
bump on `TuningParams` is the canonical mechanism).

### II. Tests Cover the Snapshot, Not the Internals

The 280+ tests in `harmonium_core` are organized to assert properties of the
snapshot stream (note timing, chord progression, density envelopes) rather
than to introspect internal RNG state or sequencer step indices. Tests that
reach into private state are fragile and discouraged.

### III. wasm32 is a First-Class Target

The core compiles to `wasm32-unknown-unknown` without conditional
`cfg(target_arch = "wasm32")` workarounds in shared modules. Any code that
truly cannot run in wasm (audio device access, threading, OS file I/O)
lives in `harmonium_host` and its native-only modules — never in
`harmonium_core`. The browser port's correctness depends on this rule.

### IV. Workspace Boundaries

The workspace members and their roles:

- `harmonium_core` — pure types, no I/O, no audio device, fully wasm-safe
- `harmonium_ai` — ML-adjacent features behind a feature flag
- `harmonium_audio` — DSP primitives (filters, envelopes, mixers)
- `harmonium_host` — OS integration: `cpal::Stream`, `NativeHandle`,
  thread management. Not wasm-compatible.
- `harmonium_bevy` — visualization layer (Bevy renderer)
- `harmonium_cli` — command-line entry point for local runs
- `xtask` — build automation, not shipped

A crate that needs `tokio`, threads, or device handles belongs in `_host`
or higher, never in `_core` or `_audio`.

### V. Tuning Parameters Live in Config, Not Constants

The ~83 style parameters that define engine personality are accessed via
`TuningParams`. New constants that affect musical output (rhythm thresholds,
voice-leading rules, melody weights) MUST be added to `TuningParams`, not
declared inline. The LLM tuning pipeline (`harmonium_lab`) depends on
exhaustive parameter coverage.

## Tech Constraints (Repo-Specific)

- Rust 2024 edition workspace; `unsafe_code = "deny"` and strict clippy.
- `gen` is reserved — use `tgen` or domain-specific names in identifiers.
- `ChaCha8Rng` is the canonical RNG. Adding another RNG type requires a
  written justification in the relevant spec's `plan.md`.
- `opt-level = 2` for dependencies (already configured in `Cargo.toml`);
  audio paths benefit measurably from optimized rustc output even in debug.

## Spec Scope

Per-repo specs in this repo live in `specs/` and cover changes to the
engine workspace. Cross-cutting work (e.g., a feature that changes the
snapshot contract AND the training app's lookahead consumer) lives in
`harmonium_specs/` and references this repo via `participating_repos:`.

**Version**: 1.0.0 | **Ratified**: 2026-05-25 | **Last Amended**: 2026-05-25

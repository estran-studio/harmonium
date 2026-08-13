---
spec_type: per_repo_feature
plane_history:
  - C-28 — Comping rhythm variation — independent jazz voicing rhythm (Groomed)
status: groomed
priority: high
workstream: W1
strategy: STRATEGY_2026.md — Hear the harmony (chordal voice)
---

# Comping rhythm variation — independent jazz voicing rhythm

**Feature Branch**: `002-comping-rhythm-variation`

**Created**: 2026-05-25

**Status**: Draft (migrated from Plane)

**Input**: Migrated from Plane on 2026-05-25 as part of the spec-kit transition.

## Plane Content (verbatim)

## Problem

The voicing system exists (`enable_voicing`, `voicing_density`, `voicing_tension`) but is disabled by default. When enabled, chord voicings follow the same trigger pattern as the lead voice — they don't have their own rhythmic identity. A real jazz pianist comps with rhythmic independence: sparse hits, Charleston rhythm (beat 4 → beat 1 tied), anticipations, and "laying out" (stopping entirely for 1-2 bars to give the soloist space).

## What a Real Jazz Pianist Does

- **Sparse, irregular rhythm** — not every beat, not every bar. Typically 2-4 hits per bar in medium swing.
- **Anticipations** — chord hit on the "and" of beat 4 tied into beat 1 of next bar (the Charleston figure).
- **Responsive density** — comps more during held melody notes, lays out during busy melody passages.
- **Laying out** — entire bars of silence, especially in the first chorus or during drum solos.
- **Accent the form** — heavier comping at the top of a section, lighter on the bridge.
- **Rhythmic variety per style** — bossa: steady quarter-note pattern; funk: 16th-note stabs; ballad: whole-note pads; swing: off-beat accents.

## Proposed Implementation

### 1. CompingRhythmGenerator

New module or method in the timeline generator that produces independent comping triggers, separate from the lead polygon pattern.

```
struct CompingParams {
    /// Average hits per bar (1.0 = sparse ballad, 4.0 = active swing, 8.0 = dense funk)
    hits_per_bar: f32,
    /// Probability of anticipation (hit on "and of 4") per bar
    anticipation_probability: f32,
    /// Probability of laying out (silent bar) per bar
    layoff_probability: f32,
    /// Whether comping density should inversely correlate with melody density
    responsive_to_melody: bool,
    /// Charleston rhythm probability (and-of-4 → beat-1 tied)
    charleston_probability: f32,
}

```

### 2. Integration with TimelineGenerator

- Add a `comping_pattern: Vec<CompingTrigger>` generated per measure alongside the sequencer pattern
- Comping triggers are independent of kick/snare/hat/lead triggers
- Each trigger has: `step`, `velocity`, `duration_steps` (stab vs sustained)
- Responsive mode: if lead has many notes in a region, reduce comping density in that region

### 3. Voicing Output

- On each comping trigger, generate a chord voicing using the current chord context
- Voicing density/tension params (already exist) control number of notes and extensions
- Output as TrackId::Chord (new) or repurpose an existing channel

## Style Profile Integration

`CompingParams` becomes part of `TuningParams` (CORELIB-23). Different styles set very different comping feel:

| Style | hits_per_bar | anticipation | layoff | charleston |
| Medium Swing | 2.5 | 0.4 | 0.15 | 0.3 |
| Bossa Nova | 4.0 | 0.1 | 0.05 | 0.0 |
| Ballad | 1.5 | 0.2 | 0.3 | 0.1 |
| Funk | 6.0 | 0.5 | 0.05 | 0.0 |
| Bebop | 3.0 | 0.5 | 0.2 | 0.4 |

## Testing

- Comping triggers are independent from lead triggers (never 100% overlap)
- Average hits per bar matches configured value ±20%
- Layoff bars have zero comping triggers
- Anticipation triggers land on correct step (and-of-4)
- Existing tests pass (default CompingParams should produce reasonable output)

## Dependencies

- Benefits from CORELIB-23 (TuningParams) but can ship with hardcoded defaults first
- Voicing pitch selection already exists — this task is about *when* voicings play, not *what* notes

## Files

- Modify: `harmonium_core/src/timeline/generator.rs` — comping trigger generation + output
- Modify or new: comping params struct (inline or in `params.rs`)
- Modify: `harmonium_core/src/params.rs` — add CompingParams to MusicalParams

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Independent comping triggers with sparse hits and layoffs (Priority: P1)

A jazz pianist listening to the engine notes that the comping voicings click on every lead trigger — it sounds like a sequencer doubling the melody, not a pianist comping behind a soloist. As a musician hearing the output, I want the comping voice to fire on its own sparse, irregular rhythm (with occasional silent bars) so that the chord pad sits behind the melody the way a real jazz pianist does.

**Why this priority**: This is the core problem the spec exists to solve — the existing voicing system is "disabled by default" because, even when on, it parrots the lead pattern. Without independent timing there is no point shipping further comping refinements; this story unblocks the entire effort.

**Independent Test**: Enable comping with `CompingParams { hits_per_bar: 2.5, layoff_probability: 0.15, .. }` and a fixed `ChaCha8Rng` seed. Generate 100 bars of `MeasureSnapshot`. Assert that (a) comping trigger steps are not a subset of lead trigger steps across the run, (b) the mean hits-per-bar is within ±20% of 2.5, (c) approximately 15% of bars contain zero comping triggers.

**Acceptance Scenarios**:

1. **Given** `CompingParams { hits_per_bar: 2.5, layoff_probability: 0.0 }`, **When** the engine renders 50 bars, **Then** mean comping hits per bar is within ±20% of 2.5 and at least one bar contains a comping trigger that does not coincide with a lead trigger.
2. **Given** `CompingParams { layoff_probability: 0.3 }`, **When** the engine renders 100 bars, **Then** roughly 30% of bars (±5 absolute) contain zero comping triggers and the remaining bars carry the configured `hits_per_bar`.
3. **Given** the same seed and `CompingParams`, **When** the engine renders twice, **Then** the comping trigger streams are byte-identical (deterministic via `ChaCha8Rng`).

### User Story 2 — Charleston anticipations and melody-responsive density (Priority: P2)

A jazz pianist wants the comping to land on the "and of 4" tied into beat 1 (the Charleston figure) and to thin out when the soloist gets busy. As a musician hearing the output, I want anticipation hits and melody-responsive density so that comping interacts with the melody instead of running in parallel.

**Why this priority**: These are the next two behaviors that separate "independent triggers" (P1) from "feels like a real pianist" — Charleston is the most identifiable jazz comping gesture and melody-responsiveness is what makes comping feel listening rather than mechanical. P2 because P1 must land first.

**Independent Test**: With `CompingParams { charleston_probability: 1.0, anticipation_probability: 1.0 }`, every bar must place a comping trigger on the "and of 4" step. Then run with `responsive_to_melody: true` over a passage where the lead has dense 8th-note runs in bars 5–8: assert comping density in those bars drops below the configured baseline.

**Acceptance Scenarios**:

1. **Given** `CompingParams { anticipation_probability: 1.0 }`, **When** the engine renders 20 bars, **Then** every bar contains a comping trigger on the "and of 4" step.
2. **Given** `CompingParams { charleston_probability: 1.0 }`, **When** the engine renders, **Then** each "and of 4" anticipation is followed by a sustained trigger crossing into beat 1 of the next bar [NEEDS CLARIFICATION: precise representation of a "tied" trigger in `MeasureSnapshot` — Plane content describes the gesture but not whether ties span snapshots or are encoded as `duration_steps` on a single trigger].
3. **Given** `responsive_to_melody: true` and a lead track with high note density in a region, **When** the engine generates comping for that region, **Then** comping hits-per-bar is reduced (relative to baseline) for that region.

### User Story 3 — Style-profile-tuned comping feel (Priority: P3)

A composer switching between styles wants the comping feel to switch with them — bossa is steady quarters, ballad is sparse pads, funk is dense 16ths, bebop pushes anticipations. As a dev configuring `TuningParams`, I want `CompingParams` to be a first-class member of `TuningParams` so style profiles can carry the table from the Plane content verbatim.

**Why this priority**: Plane explicitly notes "can ship with hardcoded defaults first" — style integration is the polish layer once P1/P2 are reliable. Also depends on CORELIB-23 (TuningParams).

**Independent Test**: Load the Medium Swing, Bossa Nova, Ballad, Funk, and Bebop style profiles. For each, render 50 bars and assert mean hits-per-bar matches the Plane table values within ±20%, and that `anticipation_probability` / `charleston_probability` / `layoff_probability` produce statistically distinguishable behaviors across styles.

**Acceptance Scenarios**:

1. **Given** the Bossa Nova style profile applied, **When** the engine renders, **Then** comping hits land on steady quarter-note positions with low anticipation rate (matching `hits_per_bar=4.0, anticipation=0.1` from the Plane table).
2. **Given** the Bebop style profile, **When** the engine renders 50 bars, **Then** ~50% of bars contain an anticipation trigger and ~40% contain a Charleston tie (within statistical noise for the configured probabilities).
3. **Given** `TuningParams` is mutated mid-session via the existing morph mechanism, **When** comping params transition between styles, **Then** the comping stream evolves smoothly rather than snapping (consistent with the existing `morph_factor = 0.03` behavior).

### Edge Cases

- `hits_per_bar = 0.0` — comping is effectively always silent; ensure no division-by-zero in distribution logic.
- `hits_per_bar` greater than the grid resolution (e.g., 32 hits on a 16-step grid) — clamp or reject [NEEDS CLARIFICATION: behavior not specified].
- `layoff_probability = 1.0` — every bar is silent. Engine must still emit a valid (empty-comping) `MeasureSnapshot`.
- `responsive_to_melody = true` but the lead track has zero notes — comping should fall back to baseline density, not collapse to zero.
- `charleston_probability > 0` on the final bar of a piece — the tied note has nowhere to resolve to. Define whether the trigger is dropped, shortened, or allowed to dangle.
- Existing default `CompingParams` (when feature is enabled at all) MUST produce reasonable output without per-style configuration (Plane testing note).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST generate comping triggers independently of the lead polygon pattern such that comping trigger steps are not constrained to be a subset of lead trigger steps.
- **FR-002**: `CompingParams` MUST expose `hits_per_bar: f32`, `anticipation_probability: f32`, `layoff_probability: f32`, `responsive_to_melody: bool`, and `charleston_probability: f32` as documented in the Plane content.
- **FR-003**: Across a long run, the mean comping hits per bar MUST be within ±20% of the configured `hits_per_bar` (per Plane testing note).
- **FR-004**: A bar selected for layoff (per `layoff_probability`) MUST emit zero comping triggers.
- **FR-005**: An anticipation trigger MUST land on the "and of 4" step relative to the engine's current step grid.
- **FR-006**: A Charleston-flagged anticipation MUST sustain across the barline into beat 1 of the following bar [NEEDS CLARIFICATION: tie representation in `MeasureSnapshot`].
- **FR-007**: When `responsive_to_melody` is true, the comping density in a region MUST inversely correlate with lead note density in that region.
- **FR-008**: Comping triggers MUST carry `step`, `velocity`, and `duration_steps` fields (stab vs sustained).
- **FR-009**: Each comping trigger MUST resolve to a chord voicing using the existing `voicing_density` / `voicing_tension` pitch-selection pipeline (this spec controls *when*, not *what*).
- **FR-010**: `CompingParams` MUST be deterministic under a fixed `ChaCha8Rng` seed — re-rendering the same session with the same seed produces byte-identical comping streams.
- **FR-011**: Existing tests MUST pass with default `CompingParams` (Plane testing note).

### Key Entities *(include if data is involved)*

- **CompingParams**: New struct in `harmonium_core/src/params.rs` carrying the five style knobs above. Becomes a member of `MusicalParams` / `TuningParams`.
- **CompingTrigger**: Per-step record `(step, velocity, duration_steps)` produced by the comping generator and consumed by the voicing pipeline. Independent of kick/snare/hat/lead triggers.
- **TrackId::Chord**: New (or repurposed) channel on `MeasureSnapshot` carrying the comping voicing output [NEEDS CLARIFICATION: Plane content lists both options — new variant vs. repurpose; pick at planning time].
- **TimelineGenerator**: Existing generator in `harmonium_core/src/timeline/generator.rs` gains a per-measure `comping_pattern: Vec<CompingTrigger>` alongside the sequencer pattern.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Comping trigger steps overlap lead trigger steps less than 50% of the time on average (verified across at least 100 bars, default `CompingParams`).
- **SC-002**: Across 100 bars with `hits_per_bar=2.5`, mean is within ±20% (i.e., 2.0–3.0); across 100 bars with `layoff_probability=0.15`, layoff bar count is within ±5 absolute of the expected 15.
- **SC-003**: Voicing system, currently disabled by default, becomes enabled by default with reasonable output (Plane testing note: "default `CompingParams` should produce reasonable output").
- **SC-004**: New tests in `harmonium_core` cover all FRs (FR-001 through FR-011), at minimum one test per FR.
- **SC-005**: `harmonium_lab` "jazz comping authenticity" composite score (reference: medium swing standards corpus) improves measurably over the current "comping = lead pattern" baseline. [NEEDS CLARIFICATION: target delta — depends on whether harmonium_lab already has a comping-specific metric].

## Assumptions

- Voicing pitch selection (which notes are in the voicing) is already implemented and out of scope. This spec is purely about *when* voicings fire.
- The engine's existing `ChaCha8Rng` seeded determinism extends to the new comping generator without separate seeding plumbing.
- `TuningParams` (CORELIB-23) is the destination home for `CompingParams`. This spec can ship with `CompingParams` constructed inline / hardcoded if TuningParams lands later.
- Comping fires on a single chord-instrument channel (typically jazz piano); multi-voice comping (e.g., piano + guitar with different feels) is out of scope.

---

## Footer — File & Branch Pointers

- `harmonium_core/src/timeline/generator.rs` — comping trigger generation + emission
- `harmonium_core/src/params.rs` — new `CompingParams` struct, integration into `MusicalParams` / `TuningParams`
- `harmonium_core/src/voicing/` (existing voicing pipeline — pitch selection unchanged)
- `harmonium_audio/src/voicing/comping.rs` — existing module that may need updating for new trigger source
- **Plane history**: see frontmatter — CORELIB-28 (Groomed)

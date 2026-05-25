---
spec_type: per_repo_feature
plane_history:
  - C-28 — Comping rhythm variation — independent jazz voicing rhythm (Groomed)
status: groomed
priority: high
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

> ⚠️ This spec was migrated from Plane and needs decomposition into prioritized
> user stories via `/speckit-specify` or manual editing. The Plane content
> above is the source material.

### User Story 1 — [TO BE FILLED] (Priority: P1)

[Decompose from Plane content above]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

### Edge Cases

- [To be enumerated]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: [TO BE FILLED from Plane content]

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: [TO BE FILLED]

## Assumptions

- [TO BE FILLED]

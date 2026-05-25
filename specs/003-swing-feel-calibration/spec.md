---
spec_type: per_repo_feature
plane_history:
  - C-29 — Swing feel calibration — tempo-dependent swing ratio and explicit swing parameter (Groomed)
status: groomed
priority: high
---

# Swing feel calibration — tempo-dependent swing ratio and explicit swing parameter

**Feature Branch**: `003-swing-feel-calibration`

**Created**: 2026-05-25

**Status**: Draft (migrated from Plane)

**Input**: Migrated from Plane on 2026-05-25 as part of the spec-kit transition.

## Plane Content (verbatim)

## Problem

Swing is currently implemented as a hi-hat offset in PerfectBalance mode (`tension * (steps / hat_vertices) * 0.5`). This has several issues:

1. **Swing is tied to tension** — but swing feel and harmonic tension are independent musical dimensions. A relaxed ballad swings hard; a tense bebop tune at 280 BPM barely swings at all.
2. **Only affects hi-hat** — real swing displaces *all* off-beat notes (ride cymbal, melody, bass walk-ups, comping).
3. **No tempo-dependent ratio** — in practice, swing ratio varies by tempo. Slow tempos (ballad, ~60 BPM): near triplet swing (67:33). Medium tempos (~140 BPM): moderate swing (~60:40). Fast tempos (bebop, ~280 BPM): almost straight (52:48).
4. **Not exposed as a user/style parameter** — no `swing_amount` knob for the user or style profiles.

## Musical Background

Swing ratio = the duration ratio between the on-beat and off-beat eighth note within a beat. Expressed as a percentage or ratio:

- **Straight**: 50:50 (no swing) — bossa nova, funk, latin
- **Light swing**: 55:45 — medium-up swing, some fusion
- **Medium swing**: 60:40 — standard jazz swing
- **Heavy swing**: 67:33 (triplet feel) — slow blues, ballads
- **Shuffle**: 67:33 or harder — blues shuffle

Research shows the ratio is **inversely correlated with tempo**. Friberg & Sundström (2002) measured professional jazz drummers and found swing ratio decreases linearly from ~3.5:1 at 100 BPM to ~1.2:1 at 300 BPM.

## Proposed Implementation

### 1. Add swing_amount to MusicalParams

```
/// Swing ratio as a 0.0-1.0 parameter
/// 0.0 = straight (50:50), 0.5 = medium swing (~60:40), 1.0 = heavy shuffle (~67:33)
pub swing_amount: f32,  // default: 0.0 (straight — let style profile set it)

```

### 2. Tempo-Dependent Auto-Swing

When swing_amount is set by a style profile, optionally auto-adjust based on tempo:

```
fn effective_swing_ratio(swing_amount: f32, bpm: f32) -> f32 {
    // Base ratio from swing_amount: 0.5 (straight) to 0.667 (triplet)
    let base_ratio = 0.5 + swing_amount * 0.167;

    // Tempo correction: reduce swing at high tempos
    // Linear interpolation: full swing at 100 BPM, 60% swing at 300 BPM
    let tempo_factor = 1.0 - ((bpm - 100.0) / 400.0).clamp(0.0, 0.4);

    // Interpolate between straight and base_ratio
    0.5 + (base_ratio - 0.5) * tempo_factor
}

```

### 3. Apply Swing Displacement to All Voices

In `TimelineGenerator::generate_measure()`, displace off-beat step positions:

```
fn swing_offset(step: usize, ticks_per_beat: usize, swing_ratio: f32) -> f32 {
    // Off-beat = odd eighth-note positions
    let eighth_pos = step % (ticks_per_beat / 2);  // for 16th grid: 0,1,2,3 per beat
    if eighth_pos == ticks_per_beat / 4 {
        // This is the "and" — displace it
        let max_displacement = (ticks_per_beat as f32) / 4.0;
        (swing_ratio - 0.5) * 2.0 * max_displacement
    } else {
        0.0
    }
}

```

Apply to: lead melody, bass, comping voicings, hi-hat. Do NOT apply to kick/snare backbeat (they stay on the grid).

### 4. Style Profile Integration

`swing_amount` becomes part of `TuningParams` (CORELIB-23):

| Style | swing_amount | Notes |
| Medium Swing | 0.55 | Standard jazz swing |
| Ballad | 0.7 | Heavy swing, near triplet |
| Bebop (fast) | 0.3 | Light swing, tempo will reduce further |
| Blues Shuffle | 0.85 | Hard shuffle |
| Bossa Nova | 0.0 | Straight eighths |
| Funk | 0.0 | Straight sixteenths |
| Latin | 0.0 | Straight |
| Waltz | 0.4 | Light swing |

### 5. EngineCommand

```
EngineCommand::SetSwingAmount(f32)  // 0.0-1.0

```

## Testing

- swing_amount=0.0 produces notes on exact grid positions (no displacement)
- swing_amount=0.5 displaces off-beat notes by expected offset
- Higher BPM reduces effective swing ratio
- Kick and snare are NOT displaced (stay on grid)
- Lead, bass, hat, comping ARE displaced
- Existing golden file tests updated (default swing=0.0, so no regression)

## Dependencies

- Benefits from CORELIB-23 (TuningParams) for per-style defaults
- Benefits from Comping task (comping notes also need swing displacement)
- Can ship independently with a hardcoded default of 0.0 (no behavior change)

## Files

- Modify: `harmonium_core/src/params.rs` — add `swing_amount` field
- Modify: `harmonium_core/src/timeline/generator.rs` — apply swing displacement in generate_measure()
- Modify: `harmonium_core/src/command.rs` — add SetSwingAmount command
- Modify: `harmonium_core/src/controller.rs` — handle SetSwingAmount

## References

- Friberg & Sundström (2002) — "Swing Ratios and Ensemble Timing in Jazz Performance" (empirical tempo-swing relationship)
- Honing & De Haas (2008) — "Swing Once More: Relating Timing and Tempo in Expert Jazz Drumming"

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

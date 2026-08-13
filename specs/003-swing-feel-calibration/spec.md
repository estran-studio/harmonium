---
spec_type: per_repo_feature
plane_history:
  - C-29 — Swing feel calibration — tempo-dependent swing ratio and explicit swing parameter (Groomed)
status: groomed
priority: high
workstream: W1
strategy: STRATEGY_2026.md — Hear the harmony (feel/swing)
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

### User Story 1 — Explicit, independent swing parameter applied to all off-beat voices (Priority: P1)

A jazz musician hearing the engine notes that the hi-hat swings but the melody, bass walk-up, and comping land dead-on the grid — the band sounds like one swinging drummer playing over a straight rhythm section. As a musician, I want a single `swing_amount` parameter (independent of tension) that displaces *all* off-beat notes consistently so that the whole band swings together.

**Why this priority**: This is the foundational fix the Plane content frames first — swing being tied to tension and hat-only are the two issues that block every other improvement. Without an explicit, hat-independent swing knob, style profiles cannot encode genre-correct feel.

**Independent Test**: With `swing_amount = 0.5` and a fixed seed, render a measure. Assert that off-beat (the "and" of each beat) onsets in lead, bass, comping, and hat carry the expected positive time-offset, while kick and snare onsets remain exactly on the grid. Re-render with `swing_amount = 0.0` and assert all onsets are on the grid (no displacement anywhere).

**Acceptance Scenarios**:

1. **Given** `swing_amount = 0.0`, **When** the engine renders, **Then** all onset positions in the `MeasureSnapshot` are exact step-grid multiples (no displacement on any voice).
2. **Given** `swing_amount = 0.5`, **When** the engine renders, **Then** off-beat onsets in lead, bass, comping, and hi-hat are displaced by the expected offset (per the `swing_offset` formula from the Plane content) and kick / snare backbeat onsets remain on the grid.
3. **Given** `swing_amount` and `tension` set to different values, **When** the engine renders, **Then** swing displacement depends only on `swing_amount` (it does not change with `tension`).
4. **Given** an `EngineCommand::SetSwingAmount(f32)` is sent mid-session, **When** the next measure is generated, **Then** the new value is in effect (subject to the standard `morph_factor` smoothing).

### User Story 2 — Tempo-dependent automatic swing-ratio correction (Priority: P2)

A jazz musician knows that the same "medium swing" feel at 80 BPM and at 280 BPM is *not* the same ratio — slow tempos want near-triplet, fast tempos want near-even. As a musician, I want the engine to auto-attenuate the swing ratio at high tempos so that a fixed `swing_amount` produces tempo-appropriate feel without the user re-tuning per song.

**Why this priority**: Lands on top of P1. Without P1's independent swing parameter and global voice application, tempo correction has nothing to act on. Plane content backs this with Friberg & Sundström and Honing & De Haas references — well-grounded musically.

**Independent Test**: With `swing_amount = 0.55` (medium swing), render at 100 BPM, 200 BPM, and 300 BPM. Measure the effective off-beat displacement. Assert it decreases monotonically with tempo and matches the `effective_swing_ratio` formula from the Plane content within float tolerance.

**Acceptance Scenarios**:

1. **Given** `swing_amount = 0.55` and bpm = 100, **When** the engine renders, **Then** effective swing ratio equals the formula's full-swing value (no tempo attenuation).
2. **Given** `swing_amount = 0.55` and bpm = 300, **When** the engine renders, **Then** effective swing ratio is attenuated to approximately 60% of the configured swing (per the linear interpolation in the Plane content).
3. **Given** `swing_amount = 0.0` at any tempo, **When** the engine renders, **Then** effective ratio is exactly 0.5 (straight) — tempo correction does not introduce swing.

### User Story 3 — Per-style swing defaults via TuningParams (Priority: P3)

A composer switching between Bossa Nova, Medium Swing, Bebop, and Blues Shuffle wants each style to carry its native swing feel without manual reconfiguration. As a dev / composer, I want `swing_amount` to be a first-class member of `TuningParams` so style profiles carry the table from the Plane content verbatim.

**Why this priority**: Polish layer over P1/P2. Plane content explicitly notes the spec "can ship independently with a hardcoded default of 0.0 (no behavior change)," so style integration is value-add, not blocking.

**Independent Test**: Load each of the styles in the Plane table (Medium Swing 0.55, Ballad 0.7, Bebop 0.3, Blues Shuffle 0.85, Bossa Nova 0.0, Funk 0.0, Latin 0.0, Waltz 0.4). For each, assert the configured `swing_amount` is reflected in the engine's effective swing ratio at a reference tempo.

**Acceptance Scenarios**:

1. **Given** the Bossa Nova style is selected, **When** the engine renders, **Then** `swing_amount = 0.0` (straight eighths regardless of tempo).
2. **Given** the Blues Shuffle style is selected, **When** the engine renders at 90 BPM, **Then** off-beat displacement matches a hard shuffle (near-triplet) feel.
3. **Given** style is switched mid-session, **When** `swing_amount` morphs to the new value, **Then** the transition respects the existing exponential smoothing (`morph_factor = 0.03`) and does not produce audible discontinuities.

### Edge Cases

- `swing_amount` outside [0.0, 1.0] — clamp at construction or reject [NEEDS CLARIFICATION: clamp vs reject — Plane content doesn't specify].
- Very fast tempo (> 300 BPM) — `tempo_factor` clamps at the lower bound (60%) per the Plane formula; verify no NaN.
- Existing golden file tests use default `swing_amount = 0.0`, so they MUST pass unchanged.
- Grid resolutions other than 16 (e.g., `rhythm_steps = 48` for polyrhythm) — the "and" computation must scale by `ticks_per_beat` correctly, not assume a fixed grid.
- Kick or snare placed on an off-beat by an unusual style profile — per Plane spec, backbeat instruments are NEVER displaced. The displacement rule keys off instrument role, not step position [NEEDS CLARIFICATION: confirm rule is "kick + snare exempt by track id" vs "exempt only when on a backbeat step"].

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `MusicalParams` MUST expose a `swing_amount: f32` field in the range [0.0, 1.0] with default `0.0` (straight, no swing).
- **FR-002**: Swing displacement MUST be independent of the `tension` parameter (decouple from the current `tension * (steps / hat_vertices) * 0.5` hi-hat offset).
- **FR-003**: An `effective_swing_ratio(swing_amount, bpm)` function MUST attenuate swing at high tempos using the Plane content's linear-interpolation formula (full swing ≤ 100 BPM, 60% swing at 300 BPM, clamped).
- **FR-004**: Swing displacement MUST be applied to off-beat onsets in lead, bass, comping voicings, and hi-hat tracks.
- **FR-005**: Swing displacement MUST NOT be applied to kick or snare backbeat onsets (they remain on the grid).
- **FR-006**: A new `EngineCommand::SetSwingAmount(f32)` MUST be accepted by the controller, with the new value taking effect via the existing `morph_factor` smoothing.
- **FR-007**: With `swing_amount = 0.0`, the engine output MUST be byte-identical to the pre-swing baseline (existing golden file tests pass).
- **FR-008**: `swing_amount` MUST be deterministic under a fixed `ChaCha8Rng` seed (swing offset is not random; same input produces same output).
- **FR-009**: `swing_amount` MUST be a member of `TuningParams` (CORELIB-23 integration) so style profiles carry per-style defaults.

### Key Entities *(include if data is involved)*

- **MusicalParams / TuningParams**: Existing params struct in `harmonium_core/src/params.rs` gains a `swing_amount: f32` field.
- **EngineCommand**: Existing command enum in `harmonium_core/src/command.rs` gains a `SetSwingAmount(f32)` variant.
- **TimelineGenerator**: Existing generator in `harmonium_core/src/timeline/generator.rs` gains a `swing_offset()` displacement step applied per voice during `generate_measure()`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A listening A/B comparison on a medium-swing reference (e.g., a standards corpus in `harmonium_lab`) shows the new swing implementation rated more "authentic" than the old hat-only swing in at least 70% of paired comparisons. [NEEDS CLARIFICATION: rater methodology — depends on whether harmonium_lab has a listening-test harness or relies on metric-based scoring].
- **SC-002**: All existing golden file tests pass unchanged (default `swing_amount = 0.0` regression suite).
- **SC-003**: New unit tests cover every FR (FR-001 through FR-009), at minimum one test per FR; plus a parametric test sweeping `bpm` from 60 to 320 with `swing_amount = 0.5` and asserting the tempo curve matches the formula.
- **SC-004**: `harmonium_lab` swing-ratio measurement on synthesized output matches Friberg & Sundström's empirical tempo-swing curve within an acceptable RMS error [NEEDS CLARIFICATION: tolerance — typical published curves have ±10% inter-drummer variance, so a fit within that band is plausible].

## Assumptions

- The existing `tension`-based hat-only swing displacement is replaced, not preserved alongside the new mechanism. Backward compatibility is maintained by `swing_amount = 0.0` default producing no displacement.
- Off-beat detection keys off the 16th-grid `step % (ticks_per_beat / 2) == ticks_per_beat / 4` rule from the Plane content, generalized to whatever `ticks_per_beat` is in effect.
- Kick and snare exemption is by track / role (TrackId-based), not by step position. Other percussion (e.g., toms, splashes) follow the hat / lead displacement rule [NEEDS CLARIFICATION].
- Style-profile-side wiring is part of CORELIB-23; this spec ships the engine mechanism plus the parameter slot.

---

## Footer — File & Branch Pointers

- `harmonium_core/src/params.rs` — `swing_amount` field on `MusicalParams` / `TuningParams`
- `harmonium_core/src/timeline/generator.rs` — `swing_offset()` and per-voice displacement in `generate_measure()`
- `harmonium_core/src/command.rs` — `EngineCommand::SetSwingAmount`
- `harmonium_core/src/controller.rs` — command handler
- References: Friberg & Sundström (2002); Honing & De Haas (2008)
- **Plane history**: see frontmatter — CORELIB-29 (Groomed)

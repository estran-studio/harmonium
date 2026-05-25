---
spec_type: per_repo_feature
plane_history:
  - CORELIB-30 — Polyrhythm Level 1 (Groomed)
  - CORELIB-31 — Polyrhythm Level 2 (Backlog)
  - CORELIB-32 — Polyrhythm Level 3 (Backlog)
  - CORELIB-33 — Polyrhythm Level 4 (Backlog)
status: groomed
priority: medium
---

# Polyrhythm progression (Levels 1–4)

**Feature Branch**: `004-polyrhythm-progression`

**Created**: 2026-05-25

**Status**: Draft (migrated from Plane; combines 4 sequential CORELIB items)

**Input**: A 4-level progression for polyrhythmic capability in the engine,
from explicit polygon vertex ratios (L1) to continuous pattern morphing (L4).
Each level builds on the prior.

## Plane Content (verbatim, all 4 levels)

### Level 1 (CORELIB-30, Groomed) — Polyrhythm Level 1 — expose polygon vertex ratios as explicit params in TuningParams

## Goal

Reframe existing PerfectBalance polygon vertex counts as explicit polyrhythmic ratios so style profiles can define genre-specific rhythmic relationships.

## Context

PerfectBalance mode already layers 5 polygons with different vertex counts — a 4-vertex kick against a 7-vertex hat IS a 4:7 polyrhythmic relationship. But this isn't exposed as "polyrhythm" and vertex counts are density-driven, not ratio-driven.

## What Changes

- Add `PolyrhythmConfig` to TuningParams (part of CORELIB-23):
  - `density: f32` — 0.0=straight time, 1.0=max polyrhythmic layering
  - `max_ratio: u32` — caps complexity (2=only 3:2, 3=allows 4:3, 4=allows 5:4)
  - `phase_spread: f32` — 0.0=aligned, 1.0=max displacement between layers
  - `oddity_preference: f32` — 0.0=allow symmetric, 1.0=enforce rhythmic oddity (Simha Arom)

- Tension-to-polyrhythm thresholds:
  - 0.0-0.2: straight time
  - 0.2-0.4: 3:2 (swing)
  - 0.4-0.6: 3:4 / 4:3 (Elvin Jones)
  - 0.6-0.8: 5:4 (modern jazz)
  - 0.8+: 7:4 (avant-garde)

- Style profile examples:
  - Swing: hat=12, kick=4, snare=2 — natural 3:2
  - Afro-Cuban: hat=12, kick=5, snare=3 — 5:3:4 texture
  - Bossa: hat=8, kick=4, snare=2 — straight duple

## Grid Resolution

Default to `rhythm_steps=48` (12/beat) when polyrhythm active. Supports duple (2,4,8,16), triple (3,6,12), and 3:4/4:3 natively. Already exists as option.

## Effort: Low

Parameter exposure + threshold mapping. Existing PerfectBalance polygon system does the heavy lifting.

## Dependencies

Part of CORELIB-23 (TuningParams). Benefits from CORELIB-29 (swing).

### Level 2 (CORELIB-31, Backlog) — Polyrhythm Level 2 — independent track subdivisions (polymeter textures)

## Goal

Allow individual instrument tracks to use different cycle lengths (n values), creating polymeter textures. E.g., drums on 12-grid while piano comps on 8-grid.

## Approach

- Per-track `EuclideanLayer { k, n, rotation }` where n can differ
- All tracks align at bar boundaries (LCM determines re-sync)
- Independent subdivision flag per track

## Priority: V2.1

Nice textural addition but not critical for launch. Current shared-grid produces good results.

## Dependencies

Level 1 (explicit vertex ratios) should ship first.

## Research Notes

- For 5:4 (LCM=20) and 7:4 (LCM=28), need 420+ ticks/beat
- Standard MIDI 480 PPQ handles everything except 7-tuplets exactly
- Metric modulation (Level 3) and continuous morphing (Level 4) are V3 scope — require further research on implied tempo shifts (Elvin Jones technique) and maintaining Perfect Balance (X[1]=0) during polygon transitions

### Level 3 (CORELIB-32, Backlog) — Polyrhythm Level 3 — implied metric modulation triggered by emotion/tension state

## Goal

Drums temporarily reinterpret the beat grid at a different rate (implied metric modulation) while bass and piano hold the original tempo. Triggered by emotion/tension state changes. Creates the "floating" feel pioneered by Elvin Jones and implemented by Genius JamTracks as their killer feature.

## What Metric Modulation Is

A rhythm that already exists in the music becomes the new beat. Example: at 120 BPM, triplet-eighths run at 360/min. Make the triplet-eighth the new quarter = 180 BPM (3:2 ratio). The listener heard the triplets before the shift, so the new tempo feels organic — like stepping onto a moving walkway.

**Implied** = drums shift, band stays. Creates tension because two temporal layers coexist. When drums snap back, there's release. This is far more common in jazz than "actual" modulation where everyone shifts.

## Trigger Points — Tied to Existing Engine State

The engine already tracks state changes that are natural triggers:

| Trigger | Current Behavior | Metric Modulation Adds |
| **Phrase arc climax** (bar 3 of 4-bar phrase) | Density/tension peak +15% | Drums shift to 3:2 implied tempo at climax, snap back at phrase resolution |
| **Section B (bridge)** | AABA form: arousal drops -0.25 | Drums imply half-time (2:1) on bridge, contrast with A sections |
| **Tension spike** (tension > 0.7) | Harmony shifts to Neo-Riemannian, chords accelerate | Drums start implying 3:2 or 4:3 — rhythm "lifts" alongside harmony |
| **Tension drop** (tension < 0.3) | Harmony returns to Steedman, density decreases | Drums resolve back to straight time — tension release felt rhythmically |
| **Emotion pad movement** | All params morph via morph_factor | Drums cross modulation threshold during morph — natural transition |
| **Long stable section** (VarietyParams jitter) | Phrase/section arc jitter for variety | Random modulation event (like a drummer deciding to "go somewhere") |

## Style Profile Control

Different styles use metric modulation differently — part of TuningParams:

| Style | Behavior |
| Swing | 3:2 at high tension, resolve at phrase end. Classic Elvin feel. |
| Bebop | Rare — tempo already fast, modulation would be chaotic |
| Ballad | 2:1 double-time feel at climax (bar 24 of 32), resolve for last A |
| Bossa | Almost never — straight feel is the identity |
| Modal | Frequent — long static harmony needs rhythmic movement to stay interesting |
| Funk | 4:3 shifts — 16th grid to triplet grid creates "slippery" funk feel |

## Proposed Config (Part of TuningParams)

```
struct MetricModulationConfig {
    /// Tension above this enables modulation
    tension_threshold: f32,        // e.g., 0.6 for swing, 0.9 for bossa (effectively never)
    /// Chance per section boundary when threshold met
    probability: f32,              // e.g., 0.3 for swing, 0.0 for bossa
    /// Style-specific ratio
    preferred_ratio: (u32, u32),   // (3,2) for swing, (2,1) for ballad double-time
    /// How long before forced resolution
    max_duration_bars: u32,        // 4-8 bars typical
    /// Implied (drums only) vs Actual (entire band)
    scope: ModulationScope,        // Almost always Implied for jazz
    /// Bars to phase transition in/out
    transition_bars: u32,          // 1-2 bars
}
```

## Implementation Approach

1. When tension crosses `tension_threshold` at a section/phrase boundary, roll against `probability`
2. If triggered: drums re-quantize their pattern to the implied grid
  - For 3:2: drums play as if tempo is 1.5x faster
  - Accent placement shifts to create the illusion of new meter
  - Bass + piano + lead continue at original tempo

3. After `max_duration_bars`, or when tension drops below threshold: resolve back
4. Resolution must align at a bar boundary — for 3:2, 2 bars original = 3 bars implied

## What the Listener Hears

- A "moment of ambiguity" where you can hear it both ways (1-2 bars)
- Tension while two tempos coexist
- Release when drums snap back
- It does NOT sound like someone changed the tempo — it sounds like the ground shifted

## Famous References

- **Elvin Jones** / Coltrane Quartet — accent displacement over triplet streams creating "illusion that the meter changed"
- **Tony Williams** / "Footprints" (Miles Davis, 1966) — "brilliantly jumps around changing tempos while band holds primary metric"
- **Genius JamTracks** — implements this as adjustable per-instrument polyrhythm levels with 8 metric modulation types

## Technical Challenges

- Drums need to render at a different effective tempo than other tracks — requires tempo-independent pattern generation for the drum track
- Resolution alignment: for 3:2, the 2-bar/3-bar equivalence must land cleanly on a barline
- Grid resolution: 48 ticks/beat (already supported) handles 3:2 and 4:3. 5:4 needs 60+ ticks.
- The transition bars need to gradually phase accents rather than abruptly shifting

## Priority: V3

High effort. Requires tempo-independent drum rendering and careful alignment logic. But this is what separates "good" from "alive" in the rhythm engine — and it's Genius JamTracks' strongest feature.

## Dependencies

- CORELIB-30 (Level 1 — explicit polyrhythm ratios in TuningParams)
- CORELIB-31 (Level 2 — independent track subdivisions)
- CORELIB-29 (swing calibration — swing must be global before modulation makes sense)
- CORELIB-28 (comping variation — comping should NOT modulate, only drums)
- CORELIB-23/24 (TuningParams — MetricModulationConfig lives here)

### Level 4 (CORELIB-33, Backlog) — Polyrhythm Level 4 — continuous rhythm pattern morphing (all modes)

## Goal

Replace abrupt pattern swaps at barlines with smooth rhythmic crossfades when parameters change. When density, tension, or style shift, the groove **evolves** over 2-4 bars instead of clicking into a new pattern. Works for all three rhythm modes (PerfectBalance, Euclidean, ClassicGroove).

## The Problem

Currently when a parameter crosses a threshold (e.g., density 0.4→0.7), the sequencer generates a new pattern and swaps it in at the next barline. This creates audible "clicks" — one groove stops, another starts. Real drummers don't do this. They gradually add ghost notes, shift accents, open the hi-hat, add splashes. The groove transforms continuously.

This matters most in:

- **Emotion Lab:** User drags the pad — harmony morphs smoothly (morph_factor=0.03) but rhythm SNAPS at thresholds. The rhythm is the only system without smooth transitions.
- **Phrase arcs:** Density/tension modulate ±15% over 4-bar phrases. Small threshold crossings cause audible pattern swaps mid-phrase.
- **Style transitions:** Switching from Ballad to Swing mid-session produces an abrupt rhythmic break.

## Architecture — Unified Morph Layer

The morph layer sits **above** the rhythm mode, not inside it. It doesn't need to know which mode generated the pattern — it just sees onset positions and velocities.

```
Parameters change (density, tension, style...)
       │
       ▼
Rhythm mode generates TARGET pattern
(Euclidean, PerfectBalance, or ClassicGroove — unchanged)
       │
       ▼
Morph layer compares CURRENT playing pattern vs TARGET
       │
       ├─ Same? → do nothing
       │
       └─ Different? → crossfade over N bars
              ├─ Hits only in CURRENT: velocity ramps DOWN to 0
              ├─ Hits only in TARGET: velocity ramps UP from 0
              └─ Hits in BOTH: stay at full velocity (anchors)

```

### Per-Mode Details

| Mode | What Morphs | Complexity |
| **PerfectBalance** | Polygon vertex count + phase rotation. Can verify mathematical balance (X[1]=0) during morph. Interpolate onset positions from current polygon config to target. | Medium — geometric interpolation |
| **Euclidean** | Onset count E(k,n). E(5,16)→E(7,16) = fade in 2 new hits at maximally-even positions. Bjorklund gives positions directly. | Low — simpler than PB |
| **ClassicGroove** | Handcrafted patterns cross density thresholds. When density crosses 0.40→0.60, crossfade from "beats 1&3" to "straight kick" over 2-4 bars. | Low — pure velocity interpolation |

## Data Model

```
struct PatternMorphState {
    /// Currently playing pattern (what the audience hears)
    current_pattern: Vec<StepTrigger>,
    /// Target pattern (what the sequencer wants to play next)
    target_pattern: Option<Vec<StepTrigger>>,
    /// 0.0 = fully current, 1.0 = fully target
    progress: f32,
    /// Bars to complete the transition
    transition_bars: u32,    // default: 2-4, configurable via TuningParams
    /// Current bar within the transition
    current_transition_bar: u32,
}

impl PatternMorphState {
    /// Called each barline. Returns the blended pattern to play.
    fn advance(&mut self) -> Vec<StepTrigger> {
        if let Some(target) = &self.target_pattern {
            self.current_transition_bar += 1;
            self.progress = self.current_transition_bar as f32
                          / self.transition_bars as f32;

            if self.progress >= 1.0 {
                // Morph complete — target becomes current
                self.current_pattern = target.clone();
                self.target_pattern = None;
                self.progress = 0.0;
                return self.current_pattern.clone();
            }

            // Blend: interpolate velocities per step
            self.blend_patterns(self.progress)
        } else {
            self.current_pattern.clone()
        }
    }

    fn blend_patterns(&self, t: f32) -> Vec<StepTrigger> {
        // For each step position:
        // - Hit in both: velocity = max of both (anchor)
        // - Hit only in current: velocity *= (1.0 - t) (fading out)
        // - Hit only in target: velocity *= t (fading in)
        // - Hit in neither: silent
    }
}

```

## Integration Points

- `Sequencer.prepare_next_bar()` — instead of directly swapping `next_pattern` into `pattern`, feed the new pattern into `PatternMorphState` as a target
- `TimelineGenerator.generate_measure()` — read from `morph_state.advance()` instead of raw sequencer pattern
- `TuningParams` — add `rhythm_morph_bars: u32` (default: 2). Style profiles control transition speed: ballad=4 (slow evolve), bebop=1 (fast snap), funk=2 (medium).
- Setting `rhythm_morph_bars = 0` disables morphing entirely (current behavior, backward compatible)

## Musical Scenarios

### Scenario A: Ballad builds to climax

```
Bars 1-8:   Sparse brush (kick=2, hat=6, snare=1) — calm, open
Bars 9-12:  Morph begins — density rising
            kick: 2→3 (bass drum activates gradually)
            hat: 6→8 (brushes sweep faster)
            snare: 1→2 (backbeat solidifies)
Bars 13-16: Full swing (kick=4, hat=12, snare=2) — climax arrived

16-bar build where rhythm organically grew. No abrupt changes.

```

### Scenario B: Emotion Lab pad drag

```
User drags from calm+dark (bottom-left) to intense+bright (top-right)

Without morphing: harmony flows, rhythm SNAPS at each threshold
With morphing: rhythm flows WITH the harmony

hat vertices: 6 → 7 → 8 → 10 → 12 (over ~4 bars)
kick phase rotates: 0° → 5° → 12° → 20°
ghost notes: velocity ramps from 0 to full (not binary on/off)

Result: the ENTIRE texture — harmony, melody, dynamics, rhythm —
flows together as one continuous transformation.

```

### Scenario C: Style switch mid-session

```
User switches "Ballad" → "Medium Swing"

Without morphing: abrupt pattern change at barline
With morphing: 4-bar transition where groove evolves from ballad to swing

The listener hears the band "shift gears" — like a real band when
the leader signals a new feel and the drummer transitions naturally.

```

## Velocity Crossfade Detail

The crossfade works on velocity, not on/off triggers:

```
Step 3 example over 4-bar morph:

  Current pattern: kick=true, vel=0.9
  Target pattern:  kick=false

  Bar 1 (progress=0.25): kick plays, vel = 0.9 * 0.75 = 0.675
  Bar 2 (progress=0.50): kick plays, vel = 0.9 * 0.50 = 0.45
  Bar 3 (progress=0.75): kick plays, vel = 0.9 * 0.25 = 0.225 (barely audible)
  Bar 4 (progress=1.00): kick gone

  Listener hears: kick gradually disappears rather than vanishing

```

Hits that exist in both patterns stay at full velocity throughout — these are **anchors** that maintain groove continuity during the transition.

## PerfectBalance-Specific: Balance Verification

For PerfectBalance mode only: during the morph, verify that the interpolated onset positions maintain X[1]≈0 (first Fourier coefficient near zero = balanced). If the blend drifts from balance, adjust phase rotations slightly to compensate. This ensures the rhythm is always a valid, well-formed pattern even mid-transition — never "broken."

For Euclidean and ClassicGroove: no balance constraint needed. The crossfade is pure velocity interpolation.

## What This Doesn't Change

- User doesn't control morphing directly — it happens automatically when parameters change
- Bar structure unchanged (no polymeter effects)
- Other instruments don't need to know about the morph
- Default morph_bars=0 preserves current instant-swap behavior (full backward compatibility)

## Effort: High

The onset-position interpolation and per-step velocity blending are non-trivial. Needs careful testing across all three rhythm modes with various density/tension transition scenarios. But it's the kind of feature that makes the engine feel qualitatively different from pattern-based systems.

## Priority: V3

Engine sounds good enough with barline-swapped patterns for V2. Morphing is polish that makes transitions feel "alive" — the rhythmic equivalent of the harmony morph_factor that already exists.

## Dependencies

- CORELIB-30 (Level 1 — explicit vertex ratios) — morph targets need well-defined polyrhythm configs
- CORELIB-23/24 (TuningParams) — `rhythm_morph_bars` parameter lives here
- Can be implemented independently from Level 2 (subdivisions) and Level 3 (metric modulation)

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Level 1: Vertex ratios as TuningParams (Priority: P1)

A composer building a style profile knows that "4-vertex kick against 7-vertex hat" *is* a 4:7 polyrhythm — but right now those numbers are buried in density-driven vertex selection. As a dev configuring a style profile, I want to declare polyrhythm ratios explicitly in `TuningParams` so that style profiles can encode genre-specific rhythmic relationships (swing 3:2, Afro-Cuban 5:3:4, bossa 4:2) rather than emerging accidentally from density tuning.

**Why this priority**: Plane explicitly tags this as **Effort: Low** ("parameter exposure + threshold mapping; existing PerfectBalance polygon system does the heavy lifting"). It is Groomed (the other three levels are Backlog). Most importantly, L2/L3/L4 all depend on L1 — well-defined polyrhythm configs are the morph targets for L4, the per-track subdivision ratios for L2, and the source of modulation ratios for L3.

**Independent Test**: Add `PolyrhythmConfig` to `TuningParams`. Load a "Swing" style profile with `density=0.5, max_ratio=2`. Render a measure and assert the polygon vertex counts match the documented swing config (hat=12, kick=4, snare=2 — i.e., a 3:2 natural relationship). Then set `tension=0.5` and assert vertex counts crossfade to 3:4 / 4:3 per the threshold table.

**Acceptance Scenarios**:

1. **Given** `PolyrhythmConfig { density: 0.0, max_ratio: 2, .. }`, **When** the engine renders, **Then** all polygon layers use simple duple ratios (straight time).
2. **Given** `tension = 0.5` and `max_ratio = 4`, **When** the engine renders, **Then** the polygon vertex counts implement a 3:4 / 4:3 relationship (per Plane threshold table).
3. **Given** `tension = 0.9` and `max_ratio = 4`, **When** the engine renders, **Then** vertex counts implement a 7:4 relationship (avant-garde band of the threshold table).
4. **Given** `oddity_preference = 1.0`, **When** the engine selects polygon vertex counts, **Then** rhythmic-oddity-violating symmetric configurations are excluded (Simha Arom criterion).
5. **Given** a style profile that loads `PolyrhythmConfig`, **When** rendered at `rhythm_steps = 48`, **Then** all expected duple (2,4,8,16), triple (3,6,12), and 3:4 / 4:3 ratios are representable exactly.

### User Story 2 — Level 2: Independent track subdivisions (Priority: P2)

A composer wants drums to operate on a 12-grid while piano comps on an 8-grid, all aligning at the barline — a polymeter texture in the spirit of Steve Reich or West African layering. As a dev, I want each track to declare its own `EuclideanLayer { k, n, rotation }` with independent `n` (cycle length) so that polymeter textures emerge without coupling every track to a shared grid.

**Why this priority**: Plane calls this "V2.1 — nice textural addition but not critical for launch. Current shared-grid produces good results." Depends on L1 shipping first. Lands after the engine has explicit vertex ratios for tracks to reason about independently.

**Independent Test**: Configure two tracks with `EuclideanLayer { k: 5, n: 12 }` and `EuclideanLayer { k: 3, n: 8 }`. Render until LCM(12, 8) = 24 ticks — assert the per-track trigger streams match each layer independently and that both align cleanly at every barline boundary (LCM-derived).

**Acceptance Scenarios**:

1. **Given** two tracks with different `n` values, **When** the engine renders a bar, **Then** each track's triggers respect its own `EuclideanLayer` independently and both align at the bar boundary.
2. **Given** an `n` that requires more than 480 PPQ to represent exactly (e.g., 7-tuplets), **When** configured, **Then** the engine rejects the configuration or rounds with a documented warning [NEEDS CLARIFICATION: Plane research note flags standard MIDI 480 PPQ handles everything except 7-tuplets exactly, but doesn't specify the chosen mitigation].
3. **Given** track-A on `n=12` and track-B on `n=8`, **When** rendered across an LCM cycle, **Then** track alignment at each barline is exact (no drift accumulates).

### User Story 3 — Level 3: Implied metric modulation (Priority: P3)

A musician hearing the engine at a tension spike wants to feel the ground shift — drums implying 3:2 while the band holds tempo, Elvin Jones style. As a musician, I want the engine to trigger implied metric modulation at tension peaks, phrase climaxes, and section boundaries so that the rhythm participates in the emotional arc instead of merely keeping time.

**Why this priority**: Plane explicitly tags **V3, High effort**. Depends on L1 (explicit ratios for modulation), L2 (independent track subdivision — drums need to render at a different effective tempo than bass/piano/lead), CORELIB-29 (swing must be global first), and CORELIB-28 (comping must NOT modulate, only drums). P3 because the dependency chain mandates this order.

**Independent Test**: Configure `MetricModulationConfig { tension_threshold: 0.6, probability: 1.0, preferred_ratio: (3, 2), max_duration_bars: 4, scope: Implied }`. Drive tension from 0.0 to 0.8 over a phrase. Assert drums re-quantize to the 3:2 implied grid for ≤ 4 bars at the climax and snap back at the next phrase boundary, while bass / piano / lead onsets remain at the original tempo throughout.

**Acceptance Scenarios**:

1. **Given** `tension_threshold = 0.6` and `probability = 1.0` and a tension crossing at a phrase boundary, **When** the modulation is triggered, **Then** the drum track's onsets re-quantize to the implied grid (e.g., 1.5x faster effective tempo for 3:2) while bass / piano / lead onsets remain on the original grid.
2. **Given** an active 3:2 modulation, **When** `max_duration_bars` is reached or tension drops below threshold at a bar boundary, **Then** drums resolve back to the original grid cleanly at the next barline boundary (no mid-bar snap).
3. **Given** the Bossa Nova style profile (`probability = 0.0`), **When** tension spikes, **Then** no modulation is ever triggered (style identity is preserved).
4. **Given** `transition_bars > 0`, **When** modulation begins or resolves, **Then** the drums phase accents gradually across the transition window rather than snapping.
5. **Given** the modulation triggers mid-bar from a sub-section-boundary event, **When** the engine attempts the shift, **Then** the shift is deferred to the next section / phrase boundary [NEEDS CLARIFICATION: Plane spec frames triggers as "at a section/phrase boundary" but the trigger table includes mid-phrase events like phrase-arc climax — confirm whether the climax counts as a boundary or waits].

### User Story 4 — Level 4: Continuous pattern morphing across all rhythm modes (Priority: P4)

A musician dragging the Emotion Lab pad hears harmony morph smoothly but rhythm snaps at every density threshold — the rhythm is the only system without smooth transitions. As a musician, I want rhythm patterns to crossfade over 2–4 bars when parameters change so that ballad-to-swing builds, style switches, and Emotion Lab moves transform the whole texture (harmony + rhythm) together.

**Why this priority**: Plane explicitly tags **V3, High effort**. Depends on L1 (well-defined polyrhythm configs as morph targets). Lands last because it's a polish layer over working pattern generation — Plane notes "engine sounds good enough with barline-swapped patterns for V2."

**Independent Test**: Set `rhythm_morph_bars = 4`. Trigger a density change at bar 4 (e.g., `density = 0.4 → 0.7`). Render bars 4–7 and assert each step's velocity is the linearly interpolated blend of the current and target patterns. Hits present in both patterns must stay at full velocity (anchors). Setting `rhythm_morph_bars = 0` must produce the current instant-swap behavior byte-identically.

**Acceptance Scenarios**:

1. **Given** `rhythm_morph_bars = 0`, **When** parameters change, **Then** patterns swap at the next barline byte-identically to current behavior (backward compatible).
2. **Given** `rhythm_morph_bars = 4` and a parameter change at bar B, **When** the engine renders bars B..B+4, **Then** velocity at each step interpolates linearly between current and target patterns; hits in both stay at full velocity (anchors).
3. **Given** the morph is mid-progress and a *new* parameter change arrives, **When** the engine receives the new target, **Then** the morph re-targets cleanly without producing an audible click [NEEDS CLARIFICATION: target-update semantics during an in-flight morph — Plane spec describes single-target transitions but Emotion Lab can produce continuous changes].
4. **Given** PerfectBalance mode is active during a morph, **When** the engine interpolates onset positions, **Then** the blended pattern's first Fourier coefficient X[1] is verified near zero (mathematically balanced throughout the morph).
5. **Given** Euclidean mode with `E(5,16) → E(7,16)`, **When** the morph runs, **Then** the two new hits fade in at the maximally-even positions Bjorklund places them.
6. **Given** ClassicGroove mode with a density threshold crossing, **When** the morph runs, **Then** velocity interpolation alone (no onset repositioning) implements the crossfade.

### Edge Cases

- L1: `max_ratio = 0` — degenerate; clamp to ≥ 2 or reject.
- L1: `oddity_preference` plus `density = 0` — straight time is symmetric by definition; document precedence rule.
- L2: 7-tuplets at 480 PPQ — Plane research note flags inexact representation [NEEDS CLARIFICATION above].
- L2: a track with `n = 0` or `n` larger than `rhythm_steps` — reject at construction.
- L3: tension stays above threshold longer than `max_duration_bars` — must still resolve at the cap, then optionally re-trigger.
- L3: phrase boundary lands inside a `transition_bars` window — the in-flight transition either completes before the next trigger fires, or is interrupted [NEEDS CLARIFICATION].
- L3: drums implying 3:2 while the bar boundary itself shifts — Plane spec says "Resolution must align at a bar boundary — for 3:2, 2 bars original = 3 bars implied." Engine must enforce this alignment, not let modulation cancel mid-bar.
- L4: morph target changes before the previous morph completes [NEEDS CLARIFICATION above].
- L4: a step exists in both patterns at different velocities (e.g., 0.9 in current vs 0.5 in target) — Plane spec says "stay at full velocity (anchors)" but doesn't specify which velocity wins [NEEDS CLARIFICATION: max vs interpolated for anchor steps].
- L4: morphing in PerfectBalance mode pushes X[1] above the balance tolerance — engine must adjust phase rotation to compensate.
- Cross-level: L3 modulation active during an L4 morph — interaction is unspecified [NEEDS CLARIFICATION].

## Requirements *(mandatory)*

### Functional Requirements

#### Level 1 (P1)

- **FR-L1-001**: `TuningParams` MUST include a `PolyrhythmConfig` sub-struct with fields `density: f32`, `max_ratio: u32`, `phase_spread: f32`, `oddity_preference: f32`.
- **FR-L1-002**: The engine MUST map `tension` to polyrhythm complexity using the Plane threshold table (0.0–0.2 straight, 0.2–0.4 3:2, 0.4–0.6 3:4/4:3, 0.6–0.8 5:4, 0.8+ 7:4).
- **FR-L1-003**: Style profiles MUST be able to set explicit polygon vertex counts per track (hat / kick / snare) that produce the documented genre ratios (swing 3:2, Afro-Cuban 5:3:4, bossa duple).
- **FR-L1-004**: When polyrhythm is active, the engine MUST default to `rhythm_steps = 48` (12/beat) to support duple, triple, and 3:4 / 4:3 natively.
- **FR-L1-005**: `oddity_preference = 1.0` MUST exclude symmetric vertex configurations (Simha Arom rhythmic oddity).

#### Level 2 (P2)

- **FR-L2-001**: Each track MUST be configurable with an independent `EuclideanLayer { k, n, rotation }` where `n` may differ across tracks.
- **FR-L2-002**: All tracks MUST align at bar boundaries — the bar length MUST be the LCM of all active `n` values (re-sync point).
- **FR-L2-003**: Configurations whose LCM exceeds the engine's tick capacity (e.g., 7-tuplets at 480 PPQ) MUST be rejected or handled with documented rounding [NEEDS CLARIFICATION].

#### Level 3 (P3)

- **FR-L3-001**: `TuningParams` MUST include a `MetricModulationConfig` with fields per the Plane spec: `tension_threshold`, `probability`, `preferred_ratio: (u32, u32)`, `max_duration_bars`, `scope: ModulationScope`, `transition_bars`.
- **FR-L3-002**: Modulation MUST be triggered when tension crosses `tension_threshold` at a section / phrase boundary, gated by `probability`.
- **FR-L3-003**: When `scope: Implied` (the jazz default), only drum-track onsets MUST re-quantize to the implied grid; bass / piano / lead MUST remain on the original grid.
- **FR-L3-004**: Modulation MUST resolve at the next bar-aligned boundary that satisfies the original-vs-implied bar ratio (e.g., 3:2 resolves after 2 original = 3 implied bars).
- **FR-L3-005**: Modulation MUST always resolve by `max_duration_bars` even if tension stays above threshold.
- **FR-L3-006**: `transition_bars` MUST phase accent placement gradually across the transition window rather than producing an abrupt shift.

#### Level 4 (P4)

- **FR-L4-001**: `TuningParams` MUST include `rhythm_morph_bars: u32` with default `0` (current instant-swap behavior preserved).
- **FR-L4-002**: When `rhythm_morph_bars > 0` and the sequencer receives a new target pattern, the engine MUST crossfade over the configured number of bars rather than swap at the next barline.
- **FR-L4-003**: Crossfade MUST blend per-step velocities linearly: steps in both patterns stay at full velocity (anchors); steps only in current ramp down; steps only in target ramp up.
- **FR-L4-004**: Crossfade MUST work across all three rhythm modes (PerfectBalance, Euclidean, ClassicGroove).
- **FR-L4-005**: In PerfectBalance mode, the blended pattern MUST maintain X[1] ≈ 0 (balanced) throughout the morph, adjusting phase rotation as needed.
- **FR-L4-006**: `rhythm_morph_bars = 0` MUST produce byte-identical output to the pre-L4 engine (backward compatibility verified by golden file tests).

### Key Entities *(include if data is involved)*

- **PolyrhythmConfig**: New sub-struct of `TuningParams` carrying the four L1 fields.
- **EuclideanLayer**: Per-track config `(k, n, rotation)` enabling independent subdivisions (L2).
- **MetricModulationConfig**: Per-style config gating implied-modulation events (L3).
- **PatternMorphState**: Per-track state machine carrying `current_pattern`, optional `target_pattern`, `progress`, `transition_bars`, `current_transition_bar`; produces a blended `Vec<StepTrigger>` each bar (L4).
- **ModulationScope**: Enum `{ Implied, Actual }` selecting drums-only vs full-band modulation (L3).
- **TuningParams**: Existing central struct gains `PolyrhythmConfig`, `MetricModulationConfig`, and `rhythm_morph_bars`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001 (L1)**: Style profiles can produce the documented genre ratios (swing 3:2, Afro-Cuban 5:3:4, bossa duple) and `harmonium_lab` polyrhythm-classification metric correctly identifies the ratio in synthesized output for each genre.
- **SC-002 (L1)**: All existing PerfectBalance golden file tests pass unchanged when `PolyrhythmConfig` is left at defaults.
- **SC-003 (L2)**: Two tracks configured at different `n` values render alignment-correct output across at least one LCM cycle with zero drift.
- **SC-004 (L3)**: At least one triggered metric modulation per emotional-arc test fixture is detectable in `harmonium_lab` analysis, and the drum track's implied tempo is within 5% of the configured ratio.
- **SC-005 (L4)**: A/B listening evaluation on Emotion Lab pad drags rates the morphing-on output as "smoother / more musical" than morphing-off in at least 75% of paired comparisons. [NEEDS CLARIFICATION: rater methodology — depends on harmonium_lab listening-test harness].
- **SC-006 (L4)**: With `rhythm_morph_bars = 0`, all existing rhythm-mode golden file tests pass byte-identically.
- **SC-007**: Each FR is covered by at least one unit test in `harmonium_core`. Cross-level interactions (L3 modulation during L4 morph) are covered by an integration test or explicitly documented as out-of-scope.

## Assumptions

- The four levels ship sequentially in P1 → P4 order; nothing in the spec assumes simultaneous landing.
- "Drums" in L3 means specifically the kick / snare / hat tracks; comping (CORELIB-28) is explicitly excluded from modulation per the dependency note.
- Plane research notes flag standard MIDI 480 PPQ as sufficient for everything except 7-tuplets — the engine accepts this constraint and rejects (or rounds) unsupported subdivisions at construction.
- The morph layer (L4) sits above the rhythm mode (not inside it) and only needs to see onset positions and velocities — it does not require mode-specific knowledge beyond the PerfectBalance balance check.
- Default `rhythm_morph_bars = 0` preserves current behavior — L4 is opt-in per style profile.
- L3 and L4 may interact (modulation triggers during an in-flight morph). This interaction is out of scope for the first ship of either level [NEEDS CLARIFICATION above].

---

## Footer — File & Branch Pointers

- `harmonium_core/src/params.rs` — `PolyrhythmConfig`, `MetricModulationConfig`, `rhythm_morph_bars` on `TuningParams`
- `harmonium_core/src/timeline/generator.rs` — vertex-ratio selection (L1), per-track subdivision (L2), modulation triggering (L3), morph state read (L4)
- `harmonium_core/src/rhythm/` — PerfectBalance / Euclidean / ClassicGroove mode implementations (L4 morph integrates here)
- New: `harmonium_core/src/rhythm/morph.rs` — `PatternMorphState` (L4)
- New: `harmonium_core/src/rhythm/modulation.rs` — implied metric modulation logic (L3)
- `harmonium_core/src/sequencer.rs` — `prepare_next_bar()` feeds the morph state target (L4)
- **Plane history**: see frontmatter — CORELIB-30 (Groomed, L1), CORELIB-31/32/33 (Backlog, L2–L4)

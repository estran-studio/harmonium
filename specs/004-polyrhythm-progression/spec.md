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

> ⚠️ This spec was migrated from Plane and needs decomposition into prioritized
> user stories via `/speckit-specify` or manual editing. The Plane content
> above is the source material.

### User Story 1 — Level 1: Vertex ratios as TuningParams (Priority: P1)

[Decompose from L1 Plane content above]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

### User Story 2 — Level 2: Independent track subdivisions (Priority: P2)

[Decompose from L2 Plane content above]

### User Story 3 — Level 3: Implied metric modulation (Priority: P3)

[Decompose from L3 Plane content above]

### User Story 4 — Level 4: Continuous pattern morphing (Priority: P4)

[Decompose from L4 Plane content above]

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

---
spec_type: per_repo_feature
plane_history:
  - C-34 — PerfectBalance algorithm improvements — closer to full XronoMorph theory (Groomed)
status: groomed
priority: medium
---

# PerfectBalance algorithm improvements — closer to full XronoMorph theory

**Feature Branch**: `001-perfectbalance-improvements`

**Created**: 2026-05-25

**Status**: Draft (migrated from Plane)

**Input**: Migrated from Plane on 2026-05-25 as part of the spec-kit transition.

## Plane Content (verbatim)

## PerfectBalance Algorithm Improvements

Close the gap between our current PerfectBalance implementation and full XronoMorph theory. The current implementation works well — 5 regular polygon layers with density-driven vertex selection, phase rotation, meter awareness, and collision resolution. But several features from the academic framework would make grooves sound significantly more human and varied.

## Current State

Working: regular polygon superposition, per-polygon velocity, density-driven vertex selection (hat 6→8→12→16), meter-aware snare, hat masking, tension-driven hat swing, notation-safe lead, 14 tunable params in `PerfectBalanceParams`.

## Gap Analysis vs XronoMorph

|

Gap

 |

Musical Impact

 |

Effort

 |

Target

 |
|

Per-vertex velocity envelopes

 |

High — makes grooves feel human

 |

Low-Medium

 |

V2

 |
|

Multiple polygons per instrument

 |

Medium — richer layered patterns

 |

Medium

 |

V2.1

 |
|

Irregular balanced polygons

 |

Medium — less mechanical grooves

 |

Medium

 |

V2.1

 |
|

Negative weights (cancellation)

 |

Medium — intentional silences

 |

Low

 |

V2.1

 |
|

Balance verification (X[1]=0)

 |

Low now, needed for morphing (Level 4)

 |

Low

 |

V3

 |
|

Weighted superposition

 |

Low-Medium

 |

Low

 |

V2.1

 |
|

Well-formedness check

 |

Low for jazz

 |

Low

 |

Optional

 |

See child tasks for implementation details.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Per-vertex velocity envelopes (Priority: P1)

A composer hears the engine's groove and notes that every kick hits at the same volume — the result sounds mechanical even though the polygon math is musically sound. As a composer/musician, I want each polygon vertex to carry its own velocity weight so that the same polygon (e.g., a 4-vertex kick) sounds like a human drummer accenting beat 1, ghosting beat 3, and so on, rather than a metronome.

**Why this priority**: Plane Gap Analysis flags this as **High musical impact, Low-Medium effort** — the cheapest single change that audibly removes the "machine groove" character. All current PerfectBalance children are Done, so this is the first open-vs-shipped item the next phase should consume.

**Independent Test**: Generate a measure from `TimelineGenerator` with a 4-vertex kick polygon plus a per-vertex velocity envelope `[1.0, 0.6, 0.8, 0.4]`. Inspect the emitted `MeasureSnapshot`: the four kick triggers must carry velocities in exactly that ratio (within float tolerance). A/B listen against the same polygon with flat velocities — the new pattern must sound accented at beat 1 and ghosted on the off-beats.

**Acceptance Scenarios**:

1. **Given** a PerfectBalance kick polygon with N vertices and a per-vertex velocity envelope of length N, **When** the engine renders a measure, **Then** each kick onset in the `MeasureSnapshot` carries the velocity from the corresponding envelope slot.
2. **Given** no velocity envelope is configured for a polygon, **When** the engine renders a measure, **Then** behavior is identical to the current (flat per-polygon velocity) output (backward compatible).
3. **Given** a velocity envelope length that does not match vertex count, **When** the engine validates the envelope, **Then** the configuration is rejected with a clear error and the engine falls back to flat velocity for that polygon.

### User Story 2 — Negative weights for intentional silences (Priority: P2)

A composer wants to design a groove where a polygon explicitly *cancels* a vertex from another polygon at the same step (e.g., a hat layer that suppresses every fourth kick to leave breathing room). As a composer/dev, I want to assign negative weights to polygon vertices so that overlapping triggers can subtract rather than always add, enabling intentional silences inside a balanced texture.

**Why this priority**: Plane Gap Analysis flags this as **Medium impact, Low effort** — cheapest path to "intentional silence" expressivity once velocity envelopes (P1) ship.

**Independent Test**: Configure two PerfectBalance polygons where the second polygon's vertex 0 has weight `-1.0` on the same step as the first polygon's vertex 0 (weight `+1.0`). Render a measure: that step must produce **no** kick trigger. A different step where only the positive polygon hits must still trigger.

**Acceptance Scenarios**:

1. **Given** two PerfectBalance polygons whose vertices coincide on step S, one weighted `+1.0` and one weighted `-1.0`, **When** the engine resolves triggers for step S, **Then** no onset is emitted for that step on that instrument.
2. **Given** a polygon with a mix of positive and negative weights, **When** the engine renders a measure, **Then** only steps with a positive sum produce onsets and velocity scales with the absolute residual.
3. **Given** existing configurations without negative weights, **When** the engine renders, **Then** output is byte-identical to the current behavior (golden tests pass).

### User Story 3 — Weighted superposition of multiple polygons per instrument (Priority: P3)

A composer wants richer layered patterns on a single instrument — e.g., two overlapping kick polygons (a 4-vertex backbone plus a 7-vertex syncopation layer) whose contributions blend, rather than a single polygon defining the part. As a dev configuring `PerfectBalanceParams`, I want to attach multiple weighted polygons to the same instrument so that the engine sums their contributions into a single trigger stream.

**Why this priority**: Plane Gap Analysis flags **Multiple polygons per instrument** (Medium impact, Medium effort) and **Weighted superposition** (Low-Medium impact, Low effort) together as the "V2.1" textural enrichment block. Lands after P1/P2 because it depends on per-vertex velocity (P1) and negative-weight resolution (P2) being well-defined.

**Independent Test**: Configure a kick instrument with two polygons (N=4 weight 1.0, N=7 weight 0.5). Render a measure and assert the trigger count and velocities equal the weighted sum of the two polygons evaluated independently. Removing the second polygon must reproduce the single-polygon baseline exactly.

**Acceptance Scenarios**:

1. **Given** an instrument configured with multiple weighted polygons, **When** the engine renders a measure, **Then** the trigger stream equals the weighted superposition of each polygon's contribution at each step.
2. **Given** a single-polygon configuration, **When** the engine renders, **Then** behavior matches the current single-polygon path (backward compatible).
3. **Given** a configuration that pushes a vertex above max velocity (1.0) after summation, **When** the engine resolves the trigger, **Then** velocity is clamped to 1.0 [NEEDS CLARIFICATION: clamp vs renormalize — Plane content doesn't specify the overflow rule].

### Edge Cases

- Per-vertex envelope longer or shorter than vertex count — must reject or fall back, not panic.
- Negative weights that fully cancel every vertex — instrument is silent for the bar; engine must not emit an empty trigger that downstream rendering treats as garbage.
- Multiple polygons whose LCM exceeds the configured grid resolution — current implementation assumes vertices snap to grid; document and reject unsupported combinations.
- Backward compatibility: default `PerfectBalanceParams` (no envelopes, no negative weights, single polygon per instrument) must produce byte-identical `MeasureSnapshot` output to today's engine (golden file tests).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `PerfectBalanceParams` MUST support an optional per-polygon velocity envelope (`Vec<f32>` of length equal to vertex count) that scales each vertex's emitted velocity.
- **FR-002**: `PerfectBalanceParams` MUST support negative vertex weights and the trigger-resolution stage MUST sum coincident vertex weights, emitting an onset only when the residual is positive.
- **FR-003**: `PerfectBalanceParams` MUST allow attaching multiple weighted polygons to a single instrument and the generator MUST emit the weighted superposition of their contributions.
- **FR-004**: All new fields MUST be optional and absent-by-default so existing configurations produce byte-identical `MeasureSnapshot` output (verified by golden file tests).
- **FR-005**: Invalid configurations (envelope length mismatch, negative weight without sibling positive, unsupported polygon LCM) MUST be rejected at construction with a typed error rather than panicking at render time.
- **FR-006**: Velocity overflow after superposition MUST be handled deterministically [NEEDS CLARIFICATION: clamp at 1.0 or renormalize the polygon? Plane content does not specify].

### Key Entities *(include if data is involved)*

- **PerfectBalanceParams**: The existing 14-field params struct in `harmonium_core` that configures the polygon superposition engine. Gains optional fields for velocity envelopes, signed weights, and multi-polygon-per-instrument lists.
- **PolygonLayer**: Conceptual unit of (vertex_count, phase_rotation, weight, optional velocity_envelope) that gets attached to an instrument. Multiple PolygonLayers may target the same instrument under FR-003.
- **MeasureSnapshot**: Unchanged output type — gains no new variant but its existing per-trigger velocity field now carries the envelope-scaled / superposition-summed value.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `harmonium_lab` "human feel" composite score on the default jazz reference corpus improves by at least 10% (relative) over the current PerfectBalance baseline when per-vertex velocity envelopes are enabled with style-profile-tuned values.
- **SC-002**: All existing PerfectBalance golden file tests pass unchanged with the new code (zero regression for absent-by-default configurations).
- **SC-003**: At least one new unit test per FR (FR-001 through FR-005) in `harmonium_core`, plus a property test asserting that random superposition configurations produce a valid (non-NaN, non-negative-velocity, within-bar) `MeasureSnapshot`.
- **SC-004**: Inference cost stays within 5% of current `TimelineGenerator::generate_measure()` wall-clock time on the existing bench (no audible-latency regression in real-time playback).

## Assumptions

- "Next phase" PerfectBalance work scope is bounded to the **V2** and **V2.1** gaps from the Plane analysis (per-vertex velocity, negative weights, multiple polygons, weighted superposition). The V3 items (Balance verification X[1]=0, irregular balanced polygons, well-formedness check) are out of scope for this spec and would re-groom as a follow-up.
- The existing density-driven vertex selection, phase rotation, meter awareness, collision resolution, and notation-safe lead logic remain untouched — this is additive surface area on `PerfectBalanceParams`.
- Style-profile tuning of the new fields (`TuningParams` integration) is part of the broader CORELIB-23 effort, not this spec. This spec ships the mechanism with sensible defaults; style profiles wire it up separately.
- Negative weights are intended for *intentional silence inside a polygon family*, not as a substitute for muting an entire instrument (use existing instrument enable/disable for that).

---

## Footer — File & Branch Pointers

- `harmonium_core/src/rhythm/perfect_balance.rs` (or equivalent — polygon generation)
- `harmonium_core/src/params.rs` — `PerfectBalanceParams` struct extension
- `harmonium_core/src/timeline/generator.rs` — trigger resolution and superposition stage
- Golden file tests under `harmonium_core/tests/` covering current PerfectBalance behavior
- **Plane history**: see frontmatter — CORELIB-34 (Groomed; children all Done, parent kept open for this next-phase work)

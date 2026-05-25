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

---
spec_type: per_repo_feature
plane_history:
  - C-10 — Text-to-params mood mapper via ONNX Runtime (Backlog)
status: backlog
priority: medium
---

# Text-to-params mood mapper via ONNX Runtime

**Feature Branch**: `005-text-to-params-mood-mapper`

**Created**: 2026-05-25

**Status**: Draft (migrated from Plane)

**Input**: Migrated from Plane on 2026-05-25 as part of the spec-kit transition.

## Plane Content (verbatim)

## Problem

Harmonium is intentionally LLM-free, but users should be able to describe a mood in natural language ("calm evening study session", "intense dramatic climax") and have the engine map it to musical parameters. This requires a local, lightweight ML model — not a cloud API.

## Proposal

Implement a local **text-to-parameter** pipeline using ONNX Runtime:

### Option A: Small Classifier (Recommended)

- **Crate:** `ort` (ONNX Runtime for Rust)
- **Input:** Text prompt tokenized via simple vocabulary or sentence-piece
- **Output:** 6 floats — (bpm, density, tension, smoothness, valence, arousal)
- **Model size:** <5MB, inference <10ms on CPU
- **Training:** Fine-tune a small MLP or tiny transformer on (mood_text → parameter_vector) pairs
- **Training data:** Generate by labeling parameter combinations with mood descriptors (100-500 pairs)

### Option B: Embeddings-Based Similarity

- Pre-compute embeddings for ~100 mood presets using `all-MiniLM-L6-v2` (22MB)
- At runtime: embed user text → find nearest preset(s) → interpolate parameters
- No custom training required — just curate mood-to-parameter mappings

### Option C: candle (pure Rust ML)

- Use `candle` crate from Hugging Face for fully Rust-native inference
- Zero Python dependencies
- Can run a tiny BERT variant locally

### API sketch:

```
pub struct MoodMapper {
    session: ort::Session,  // or candle model
}

impl MoodMapper {
    pub fn map_text(&self, text: &str) -> MusicalParams {
        let tokens = self.tokenize(text);
        let output = self.session.run(tokens);
        MusicalParams::from_mood_vector(output)
    }
}
```

## Files to modify

- New: `harmonium_ai/src/mood_mapper.rs` — MoodMapper struct (in harmonium_ai crate)
- `harmonium_ai/Cargo.toml` — add `ort` or `candle-core` dependency
- Training scripts (Python): `scripts/train_mood_mapper.py`

## Impact

Medium — enables natural language control of the engine without cloud dependencies. Key differentiator for harmonium_training UX.

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

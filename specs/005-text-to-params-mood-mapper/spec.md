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

### User Story 1 — Natural-language mood produces reasonable TuningParams locally (Priority: P1)

A musician using harmonium_training types "calm evening study session" into a mood input and expects the engine to respond by lowering tempo, easing tension, and biasing toward a smooth, low-arousal feel — without any cloud round-trip. As an end-musician, I want to describe a mood in natural language and have the engine map it to a 6-tuple of musical parameters (bpm, density, tension, smoothness, valence, arousal) locally so that natural-language control works offline, fast, and privately.

**Why this priority**: This is the entire spec — the Plane content is single-purpose. Without local text → 6-tuple mapping working end-to-end, there is no feature to ship. All other concerns (training corpus curation, multilingual support, larger embedding models) are scope expansions on top.

**Independent Test**: Construct a `MoodMapper` (whichever option ships — Option A small classifier, B embeddings similarity, or C candle). Feed in 3 reference prompts: "calm evening study session", "intense dramatic climax", "playful upbeat morning". Assert each produces a `MusicalParams` (6-tuple) where: (a) the calm prompt yields lower bpm / lower arousal / higher smoothness than the intense prompt; (b) the playful prompt yields higher valence than the intense prompt; (c) inference completes in under 10 ms on CPU; (d) model file is under 25 MB on disk.

**Acceptance Scenarios**:

1. **Given** a `MoodMapper` instance loaded from the bundled ONNX (or candle) model, **When** `map_text("calm evening study session")` is called, **Then** the returned `MusicalParams` has bpm < 100, arousal < 0.4, and smoothness > 0.6.
2. **Given** the same mapper, **When** `map_text("intense dramatic climax")` is called, **Then** the returned `MusicalParams` has tension > 0.6 and arousal > 0.7.
3. **Given** an empty string or a string with only whitespace, **When** `map_text("")` is called, **Then** the mapper returns a documented default `MusicalParams` (or a typed error) — it does not panic.
4. **Given** a string in a language the model was not trained on (e.g., the model is English-only), **When** `map_text(...)` is called, **Then** the mapper either falls back gracefully or returns a documented "unknown" result [NEEDS CLARIFICATION: training data language coverage — Plane content does not specify].
5. **Given** the model is invoked repeatedly, **When** wall-clock inference time is measured on a representative CPU, **Then** mean inference time is < 10 ms (per Plane budget).
6. **Given** the produced `MusicalParams` is fed back into the engine via the existing parameter-update path, **When** the engine renders subsequent measures, **Then** the change applies through the standard `morph_factor = 0.03` smoothing (no abrupt audio discontinuity).

### Edge Cases

- Empty / whitespace-only input — return a default `MusicalParams` or a typed error (above).
- Unsupported language input — graceful fallback (above) [NEEDS CLARIFICATION].
- Very long input (e.g., a 10 KB essay) — truncate or reject at a documented token-count limit.
- Adversarial input (e.g., "BPM=999 tension=2.5") — output is still constrained to the valid `MusicalParams` ranges; the model does not blindly trust attacker-supplied numbers.
- Model file missing or corrupted at runtime — `MoodMapper::new()` returns a typed error; the engine continues to operate without the mood-mapper feature (it is additive).
- Cross-platform consistency — ONNX Runtime / candle inference results must be deterministic and identical across desktop platforms (macOS, Windows, Linux) and within a tolerance on mobile (Android) [NEEDS CLARIFICATION: target platforms — Plane content mentions harmonium_ai crate but not platform scope].
- Implementation choice — Plane lists three options (A: small ONNX classifier, B: embeddings similarity, C: candle). The spec accepts any of the three; the planning step picks one [NEEDS CLARIFICATION].

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A new `MoodMapper` struct in `harmonium_ai` MUST accept a `&str` input and return a `MusicalParams` (the existing 6-tuple: bpm, density, tension, smoothness, valence, arousal).
- **FR-002**: Inference MUST run locally — no network calls and no cloud-API dependencies (Plane: "intentionally LLM-free").
- **FR-003**: The bundled model file MUST be no larger than 25 MB on disk (Plane targets <5 MB for Option A, 22 MB for Option B).
- **FR-004**: Mean inference time MUST be under 10 ms on a representative CPU (Plane budget).
- **FR-005**: The returned `MusicalParams` MUST be within the engine's valid parameter ranges (no out-of-bounds outputs — clamp at the mapper boundary, do not push validation into the engine).
- **FR-006**: `MoodMapper` MUST handle empty input, whitespace-only input, and missing-model-file gracefully — no panic.
- **FR-007**: The `harmonium_ai` crate MUST add the chosen ML dependency (`ort` for Option A/B, `candle-core` for Option C) and document the rationale.
- **FR-008**: Training scripts (Python, under `scripts/`) MUST be provided for reproducing the bundled model from a documented (mood_text → parameter_vector) training corpus.
- **FR-009**: The output of `MoodMapper::map_text` MUST integrate with the engine's existing parameter-update path so that mood changes flow through the standard `morph_factor` smoothing.

### Key Entities *(include if data is involved)*

- **MoodMapper**: New struct in `harmonium_ai/src/mood_mapper.rs` carrying the loaded ML session (ONNX or candle) and exposing `map_text(&self, text: &str) -> MusicalParams` (or `Result<MusicalParams, _>` per FR-006).
- **MusicalParams**: Existing 6-tuple in `harmonium_core` (bpm, density, tension, smoothness, valence, arousal). The mapper produces an instance of this type.
- **Model file**: A bundled `.onnx` (Option A/B) or candle weights (Option C) under 25 MB shipped with the binary or downloaded on first run [NEEDS CLARIFICATION: shipping mechanism — bundle in binary, download from CDN like SoundFont EP12, or both].
- **Training corpus**: 100–500 (mood_text → parameter_vector) pairs maintained in repo, plus training scripts.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A set of at least 20 reference prompts spanning calm / intense / playful / contemplative / energetic moods produces `MusicalParams` outputs that match a human-labeled reference within an acceptable mean-error budget. [NEEDS CLARIFICATION: error metric and tolerance — likely per-parameter RMS error; depends on rubric].
- **SC-002**: Mean inference time on a representative CPU is < 10 ms over at least 1000 invocations (per FR-004 / Plane budget).
- **SC-003**: Bundled model is ≤ 25 MB on disk (per FR-003).
- **SC-004**: Zero network calls during inference, verified by mocking the network layer in a test (per FR-002).
- **SC-005**: At least one unit test per FR (FR-001 through FR-009) in `harmonium_ai`, plus an end-to-end test that feeds a mood prompt through the mapper into the engine and asserts the engine's emitted `MeasureSnapshot` reflects the parameter change.
- **SC-006**: `harmonium_training` UX integration produces an audible engine change within one bar of the user submitting a mood prompt (latency budget: < 100 ms inference + smoothing into the next bar). [NEEDS CLARIFICATION: UX scope — Plane content focuses on the engine-side mapper; harmonium_training wiring may be a follow-up spec].

## Assumptions

- The three implementation options (A: small ONNX classifier, B: embeddings similarity, C: candle) are roughly equivalent for the user-visible behavior; the planning step picks one based on dependency footprint and cross-platform availability. This spec is mechanism-agnostic.
- The 6-tuple `MusicalParams` (bpm, density, tension, smoothness, valence, arousal) remains the engine's external mood surface. If `TuningParams` (CORELIB-23) supersedes `MusicalParams` as the engine's mood surface, the mapper output may need to expand — but the Plane content names the 6-tuple explicitly.
- Training data curation is part of this spec (FR-008) — without 100–500 labeled pairs there is no model. The pairs may be hand-curated by the developer (Plane: "Generate by labeling parameter combinations with mood descriptors").
- English-only training data is the working assumption; multilingual mood input is a follow-up [NEEDS CLARIFICATION above].
- The mapper is additive: if the model file is missing or fails to load, the engine continues to operate without it (existing parameter-control paths are unaffected).
- Mobile (Android) support is desired but not gating — the spec ships when desktop works; mobile inference performance may need a different model size [NEEDS CLARIFICATION above].

---

## Footer — File & Branch Pointers

- New: `harmonium_ai/src/mood_mapper.rs` — `MoodMapper` struct
- `harmonium_ai/Cargo.toml` — add `ort` or `candle-core` dependency
- `harmonium_core/src/params.rs` — `MusicalParams` (consumed by the mapper, unchanged)
- New: `scripts/train_mood_mapper.py` — training pipeline
- New: bundled model file (location TBD per shipping mechanism — bundle vs CDN like SoundFont EP12)
- **Plane history**: see frontmatter — CORELIB-10 (Backlog)

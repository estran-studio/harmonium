# CORELIB-Tuplets: True 3:2 Tuplet Support

> **Status:** Scoping
> **Effort:** ~1.5–2 weeks (engine + MIDI + VexFlow + tests)
> **Depends on:** none — orthogonal to existing rhythmic-cell logic

---

## 1. Problem & Goal

The engine's step grid (`ticks_per_beat = 4`, `NOTATION_SAFE_STEPS = [16, 12, 8, 6, 4, 3, 2, 1]`)
cannot express a true 3:2 tuplet (three notes occupying the time of two). The
current "triplet feel" cell `[3, 3, 2]` is a 3+3+2 syncopation — clave-flavored
but not a real triplet. Consequence: generated music has no swung 8th-note
triplets, no quarter-note triplets, no 16th-note triplets. The user perceives
rhythmic monotony in jazz/swing/Latin scenarios where triplets are core
vocabulary.

**Goal.** Add genuine tuplet support end-to-end: generation → MIDI export →
VexFlow rendering, with test coverage validating that a tuplet emitted by the
generator round-trips to a `Tuplet` glyph on the sheet and to a 3:2 rhythmic
ratio in MIDI.

**Non-goal (V1).** Arbitrary tuplets (5:4, 7:8, nested). Start with 3:2
(eighth-note triplet, quarter-note triplet) and `[5:4]` if it falls out cheaply.

---

## 2. Scope

| In scope | Out of scope |
|---|---|
| 3:2 eighth-note triplets (3 notes per beat) | 5:4, 7:8, mixed tuplets |
| 3:2 quarter-note triplets (3 notes per half-bar) | Nested tuplets |
| Cell-emission API for triplet groups | Per-note micro-timing / swing ratio |
| MIDI export with correct tuplet ticks | Live audio playback timing changes (already works at sample level) |
| VexFlow rendering via `factory.Tuplet(...)` | Tuplets across barlines |
| Tests: round-trip MIDI, count distribution | Performance: re-derive cell variety from RL/profile params |

---

## 3. Schema changes

### 3a. `Articulation` → carries tuplet metadata, OR new field on `TimelineNote`

Two options. Recommend **3a.ii**.

**3a.i. Pack into `Articulation`.** Add `Triplet8th`, `Triplet4`, etc. variants.
- ✅ No schema change to `TimelineNote`/`NoteSnapshot`/MIDI.
- ❌ Conflates rhythm with articulation (Staccato vs Legato). Future-hostile.

**3a.ii. Add `tuplet: Option<TupletGroup>` to `TimelineNote`** *(recommended)*.

```rust
// harmonium_core/src/timeline/mod.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TupletGroup {
    /// Group id (notes sharing same id render under one bracket)
    pub id: u32,
    /// Tuplet ratio numerator (3 in 3:2)
    pub num: u8,
    /// Tuplet ratio denominator (2 in 3:2)
    pub den: u8,
    /// Position within the group (0..num)
    pub index: u8,
}

pub struct TimelineNote {
    // ... existing fields ...
    pub tuplet: Option<TupletGroup>,
}
```

`duration_steps` semantics: stays in the **logical** step grid as before.
For a 3:2 8th-note triplet inside a quarter (gap=4 steps), each member has
`duration_steps = 4/3` rounded — but we never store fractional steps. Instead
**duration_steps for tuplet members carries the displayed-glyph duration**
(i.e. each is `2` for an eighth-triplet, `4` for a quarter-triplet) and the
`tuplet` flag tells consumers to compress the group's *total* time by `den/num`.

This keeps `start_step` integer-valued because group members align to the start
of the parent beat — only the *internal* spacing is sub-step.

### 3b. Mirror on `NoteSnapshot`

```rust
// harmonium_core/src/report.rs
pub struct NoteSnapshot {
    // ... existing ...
    pub tuplet: Option<TupletGroup>,
}
```

Required for VexFlow consumer to render tuplet brackets.

---

## 4. Generator wiring

### 4a. New cell variant in `pick_rhythmic_cell`

`Vec<usize>` is no longer enough — need to carry tuplet metadata per note.
Either:

- Change return type to `Vec<CellNote>` where `CellNote { dur: usize, tuplet: Option<(u8, u8)> }`, or
- Keep `Vec<usize>` and emit a parallel `Vec<Option<(u8, u8)>>`. Uglier, prefer the struct.

```rust
struct CellNote {
    duration_steps: usize,
    tuplet_ratio: Option<(u8, u8)>,  // (num, den), e.g. (3, 2)
}
```

### 4b. New cells (gap = 4, eighth-note triplet)

```rust
// 3:2 eighth-note triplet — three notes in one beat
vec![
    CellNote { duration_steps: 2, tuplet_ratio: Some((3, 2)) },
    CellNote { duration_steps: 2, tuplet_ratio: Some((3, 2)) },
    CellNote { duration_steps: 2, tuplet_ratio: Some((3, 2)) },
]
```

The three `duration_steps=2` glyphs are eighth-notes; `tuplet_ratio=(3,2)`
tells consumers "play three of these in the time of two". MIDI ticks per
member = `(2 * TICKS_PER_STEP) * 2 / 3 = 160` (correct for 480-PPQ triplet 8th).

### 4c. New cells (gap = 8, quarter-note triplet)

```rust
// 3:2 quarter-note triplet — three notes in half a bar
vec![
    CellNote { duration_steps: 4, tuplet_ratio: Some((3, 2)) },
    CellNote { duration_steps: 4, tuplet_ratio: Some((3, 2)) },
    CellNote { duration_steps: 4, tuplet_ratio: Some((3, 2)) },
]
```

### 4d. Group-id assignment

Generator allocates `tuplet.id` from a per-measure monotonic counter (similar to
`next_id()` for note ids). Each member of a triplet shares the same id.

### 4e. Variety knob

Wire `VarietyParams::rhythmic_cell_variety` (already added by recent patch) to
gate triplet probability. Suggest:
- `triplet_chance = (variety - 0.4).max(0.0) * 0.6` — only fires above variety=0.4, scaling up to 36% at variety=1.0.

### 4f. Step-loop emission

`generator.rs:644` cell-emission loop currently emits notes at
`step + cell_offset` with integer `cell_offset += dur`. For tuplets we keep the
same outer loop — `cell_offset` still tracks logical steps, so a triplet group
still spans gap=4 steps in the step grid. The tuplet metadata is the only
difference. **Important:** all three triplet members share `start_step =
parent_beat_start` for VexFlow; or use the integer-rounded sub-positions.
Decision needed (see §9 Open Questions).

---

## 5. MIDI export

`harmonium_core/src/timeline/midi_export.rs:163`:

```rust
// Current
let dur_ticks = if note.duration_steps == 0 {
    TICKS_PER_STEP / 2
} else {
    (note.duration_steps as u32) * TICKS_PER_STEP
};

// Proposed
let dur_ticks = match (&note.tuplet, note.duration_steps) {
    (_, 0) => TICKS_PER_STEP / 2,
    (Some(TupletGroup { num, den, .. }), d) => {
        (d as u32) * TICKS_PER_STEP * (*den as u32) / (*num as u32)
    }
    (None, d) => (d as u32) * TICKS_PER_STEP,
};
```

For NoteOn `delta` (start time): all three members of an 8th-triplet must have
`on_tick`s at `parent_beat_start + 0, +160, +320`, not `+0, +120, +240`. The
generator can't know this at integer-`start_step` resolution — solution: MIDI
export computes start times *within a tuplet group* by re-distributing tuplet
members evenly across the parent span.

```rust
// Pseudo: for each tuplet group, find its members, recompute on_ticks to
// span (group_first.start_step .. group_first.start_step + group_total_steps)
// uniformly with `num` divisions instead of `num * (den/num)`.
```

This avoids leaking sub-step positions into `TimelineNote::start_step`.

### Test
`tests/midi_tuplet_roundtrip.rs`: emit a measure with a known triplet, parse
the MIDI, assert NoteOn deltas at `0, 160, 320` and durations of `160` each.

---

## 6. VexFlow rendering (frontend consumer)

Per `harmonium_training/CLAUDE.md` rules: VexFlow 5 Factory API, `factory.Tuplet`.

```typescript
// In the consumer (e.g. harmonium_practice/web)
import { Factory } from 'vexflow';

// Group notes by tuplet.id; for each group, build a Tuplet:
const tuplet = factory.Tuplet({
    notes: [n1, n2, n3],          // factory.StaveNote instances
    options: { num_notes: 3, notes_occupied: 2 },
});
```

Backend already exposes `tuplet: Option<TupletGroup>` on `NoteSnapshot` after
§3b. Frontend changes:
1. Iterate `notes`, group by `tuplet.id` where present.
2. For each group, after creating the StaveNotes, call `factory.Tuplet({...})`.
3. The notes still go into the voice individually; `Tuplet` is metadata on top.

**Risk:** existing VexFlow consumers don't know about tuplets — backwards
compatible because `tuplet` defaults to `None` and old behavior is preserved.

---

## 7. Test plan

### Unit
- `harmonium_core::timeline::tests::test_tuplet_group_serde` — JSON roundtrip
- `harmonium_core::timeline::generator::tests::test_triplet_cell_emits_group` —
  cell selection produces a 3-note group sharing tuplet.id
- `harmonium_core::timeline::generator::tests::test_triplet_chance_zero_when_variety_low`
- `harmonium_core::timeline::midi_export::tests::test_triplet_8th_midi_ticks` —
  on_ticks at 0/160/320, durations 160

### Integration
- `harmonium/tests/measure_snapshot_golden.rs` — new golden
  `golden_triplet_8bars.json` with a high-variety jazz scenario
- `harmonium_host` snapshot exposes tuplet metadata to consumers

### Frontend (deferred, separate PR)
- `harmonium_training/web/.../score.test.ts` — render snapshot containing a
  tuplet group, assert `<g>` SVG element with class `vf-tuplet` exists
- Manual: jazz scenario at variety=0.8, density=0.6 produces audibly swung 8ths

---

## 8. Phasing

| Phase | Deliverable | Effort | Stops at |
|---|---|---|---|
| 1 | `TupletGroup` schema + `Option<TupletGroup>` on `TimelineNote`/`NoteSnapshot` | 1 day | Compiles, default `None` everywhere, all existing tests pass |
| 2 | `pick_rhythmic_cell` returns `Vec<CellNote>`; cell emission loop handles tuplet metadata | 2 days | Generator emits triplet groups; unit tests pass |
| 3 | MIDI export honors tuplet ratio (durations + on-tick redistribution) | 1 day | Round-trip test: emit → parse → assert |
| 4 | Variety wiring + new golden snapshot for jazz scenario | 0.5 day | All `cargo test --workspace` green |
| 5 | Frontend VexFlow consumer renders `factory.Tuplet`; manual audio QA | 2–3 days | User can hear & see real triplets in practice app |

Phase 1–4 are core work; phase 5 is the practice-app frontend.

---

## 9. Open questions

- [ ] **Sub-step positions on `TimelineNote::start_step`?** Recommend: keep
  `start_step` integer, encode triplet member position via `tuplet.index` and
  let MIDI export redistribute. Avoids `f32` positions.
- [ ] **Tuplet across barlines?** Reject in V1 (return `None` from cell picker
  when gap straddles the bar end). Document as known limitation.
- [ ] **Live playback path** — does the audio side need to know about tuplets,
  or is it already sample-accurate from the timeline tick? Likely
  sample-accurate already since the audio path consumes resolved tick deltas
  from MIDI/timeline, not step indices. Confirm in phase 1.
- [ ] **Should `[3, 3, 2]` clave cells be migrated to a tuplet representation?**
  No — they are genuine 3+3+2 patterns, not triplets. Leave as-is.
- [ ] **Pivot vs. Polygon-vertex approach** — the lead trigger placement still
  uses `notation_safe_vertices`. A 3-vertex polygon (gap pattern 5+5+6 in 16
  steps) is irregular but could be the trigger source for a quarter-triplet.
  Defer: tuplets emit from cell layer only, polygon stays clean.

---

## 10. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Sub-step start times leak into `TimelineNote` | Schema bloat, downstream consumer churn | MIDI export redistributes; keep `start_step: usize` |
| VexFlow consumer breaks on unknown `tuplet` field | Frontend regression | `Option<TupletGroup>`, defaults to `None`; old consumers ignore |
| Tuplet × `notation_safe_vertices` interplay produces orphan rests | Sheet-music ugliness returns | Triplets emit only from gap=4 / gap=8 cells where parent gap is already notation-clean |
| MIDI tick redistribution off-by-one rounding | Notes drift over long measures | Use exact integer math: `total_ticks * den / num` (480 is divisible by 3) |
| Audio path needs tuplet awareness | Live playback wrong | Audit in phase 1; likely already correct via sample-accurate timeline |

---

## Critical files (orientation only)

- `harmonium_core/src/timeline/mod.rs:115-148` — `Articulation`, `TimelineNote`
- `harmonium_core/src/report.rs:18-93` — `NoteSnapshot`, `MeasureSnapshot::from_measure`
- `harmonium_core/src/timeline/generator.rs:640-700` — lead cell emission loop
- `harmonium_core/src/timeline/generator.rs:925-1000` — `pick_rhythmic_cell`
- `harmonium_core/src/timeline/midi_export.rs:160-170` — duration tick math
- `harmonium_core/src/params.rs:24-58` — `VarietyParams`

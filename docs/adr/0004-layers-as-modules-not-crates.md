# ADR 0004 — The layers are modules within `codimate-core`, not separate crates

**Status:** Accepted — 2026-06-01

## Context
`codimate-core` was a single 1,461-line `lib.rs` mixing Layer 1 (Value) and Layer 2 (Scene). An architecture review proposed making the file structure reflect the 3-layer model. Two granularities were on the table:

1. **Module split** — `value` (Layer 1) and `scene/` (Layer 2, one file per Node) within `codimate-core`.
2. **Crate split** — promote each layer to its own crate (`codimate-value`, `codimate-scene`) so the compiler *enforces* the dependency direction (`value` may not depend on `scene`).

We did (1). This ADR records why we deliberately did **not** do (2), because it's a natural thing for a future reviewer to re-suggest.

## Decision
Keep Layer 1 and Layer 2 as **modules** (`value`, `scene`) inside `codimate-core`. Do not split them into separate crates.

## Consequences / rationale
- **The navigability win is already captured** by the module split: the file tree mirrors `CONTEXT.md`'s layer table (`value.rs` = Layer 1, `scene/` = Layer 2), each Node is its own file.
- **The only extra thing a crate split buys is compiler-enforced layer direction** — and that direction is *already* naturally correct: `value` has zero references to `scene`. The accidental-violation risk on a small, single-author codebase doesn't justify the machinery.
- **Cost:** more crates mean more "which crate is this in?" overhead — a real tax for a beginner-facing project that values a small, simple surface. It also contradicts the documented crate map ("`codimate-core` = Layer 1 + 2").
- **Reversible enough:** if layer leakage ever becomes a real problem, the module boundary is the seam along which a crate split can happen later, mechanically. Doing it now would be speculative.

If a future review re-proposes the crate split, the bar to clear is: *has accidental Layer 1 → Layer 2 coupling actually happened, or is enforcement otherwise needed?* Absent that, prefer the modules.

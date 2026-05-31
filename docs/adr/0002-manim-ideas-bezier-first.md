# ADR 0002 — Which Manim ideas to adopt: "dissolve, don't port"; pursue a Bézier-first path primitive

**Status:** Accepted (direction) — 2026-05-31. The path-primitive design itself must still go through `/grill-codimate`.

## Context
Manim has several admired features. We evaluated each against Codimate's One Law (`f(t) → Scene`, pure, stateless). The recurring finding: many of Manim's clever features are **workarounds for Manim being stateful and sequential** (a "stage play"), whereas Codimate is a "formula" that can compute any moment directly. Where that's true, we port the *benefit*, not the *mechanism*.

## Decision

**Adopt as an architectural direction — the one genuinely valuable idea:**
- **Bézier-first / "every shape is a list of points."** Manim represents all shapes (circle, rect, text, arrow) as the same primitive — control points — so morphing one shape into another is just interpolating those points. Codimate is already shaped for this: our existing `Lerp` + `tween` **is** "interpolate control points." If Codimate's geometry primitive becomes a `Path` (list of control points) with `impl Lerp for Path`, then `tween(circle_path, square_path)` yields shape morphing **for free**, using machinery we already built — and more cleanly than Manim, because a morph is the *same* `tween` we use for a `radius`, not a special `Transform` object.
  - **Crux to grill:** morphing two paths requires **matching control-point counts**. Manim solves this with point alignment (`align_points`, inserting degenerate curves). The alignment strategy is the hard design question and the heart of the future grill.

**Do NOT port (Codimate's purity dissolves the problem):**
- **`.animate` builder** — bridges Manim's *mutable* mobjects to declarative animation. Codimate is already declarative (`circle().x(tween(0,100))`). The useful kernel — "continue from where the previous animation ended" — belongs in a future Layer 3 composition operator, not a method-capturing builder.
- **`embed()` live shell** — exists because Manim renders are stateful/expensive. Codimate's equivalent is **time-scrubbing** the preview (jump to any `t` instantly, because `resolve(t)` is pure). Build scrubbing later; skip the REPL.
- **CheckpointManager** — caches accumulated state to skip replay. Codimate's `resolve_at(secs)` computes any moment directly; no replay exists. Keep only the safe shadow: **memoizing expensive pure sub-results** (constant complex paths, text layout), which is trivially correct under purity.

**Keep as principle, defer as feature:**
- **GPU/GLSL rendering** — the real lesson is *separation of scene description from rasterization*, which we already have via the `Renderer` trait. GPU is a future backend, not an architecture change. See [ADR 0001](0001-rendering-backend.md).

## Consequences
- Near-term order: ship the `tiny-skia` `RasterRenderer` first (you can't iterate on a primitive you can't see), **then** grill the `Path` primitive — potentially reshaping Layer 2 (Circle/Rect become path constructors rather than distinct structs).
- Backlog (noted, not built): preview `t`-scrubbing; a Layer 3 "continue from final state" operator; pure-function memoization.

# ADR 0005 — Formula rendering via a cached Typst subprocess

**Status:** Accepted — 2026-06-05

## Context
Codimate examples need real mathematics — not just `F = ma`, but physics-grade
LaTeX such as Maxwell's equations:

```latex
\oint_{S} \mathbf{E} \cdot d\mathbf{A} = \frac{Q_{enc}}{\epsilon_0}
\oint_{C} \mathbf{B} \cdot d\mathbf{l}
    = \mu_0 \left( I_{enc} + \epsilon_0 \frac{d}{dt} \int_{S} \mathbf{E} \cdot d\mathbf{A} \right)
```

That single example exercises contour integrals (`\oint`), stretchy delimiters
(`\left( \right)`), nested fractions, bold vectors (`\mathbf`), and Greek with
subscripts (`\epsilon_0`). This is essentially *all* of TeX's math layout.

We surveyed the field for a way to turn LaTeX math into **animatable glyph
paths** (the representation required for Codimate to `Write`/morph/recolor
individual symbols — see the `Formula` term in CONTEXT.md, and ADR 0002's
Bézier-first direction):

| Candidate | Emits glyph paths? | Dependency health |
|---|---|---|
| LaTeX binary → SVG (Manim's way) | yes | needs full TeX install; non-deterministic |
| ReX (ReTeX) | yes | git-only, niche/maybe unmaintained |
| `iced_math` | no — Iced widget | drags in the Iced GUI stack |
| `katex` / `katex-rs` | no — HTML/CSS/MathML | needs a JS engine; wrong substrate |
| pure-Rust hand-roll on `ttf-parser` + MATH table | yes | slim, but reimplements most of TeX |

An earlier plan was to **hand-roll** a small math-layout engine over
`ttf-parser` + the OpenType MATH table. The Maxwell example overturned it:
hand-building stretchy delimiters, big operators with limits, and nested
fractions *faithfully* is months of fragile layout work for a single-author,
beginner-facing project.

Critically, **ADR 0001 already accepts an external runtime tool** (`ffmpeg`) for
video export, distinguishing it from a compile-time Rust dependency. That
precedent reframes "shell out to a typesetter" as an already-blessed pattern,
not a new kind of sin.

## Decision
A **Formula** is produced by shelling out to **Typst** once, at Scene-build
time, and importing the resulting vector paths. The pipeline:

```
LaTeX string
  → mitex            (Rust crate: LaTeX → Typst markup)
  → typst (binary)   (external tool, like ffmpeg: Typst → SVG with outlined glyphs)
  → usvg             (Rust crate: SVG → path geometry)
  → codimate-core Path nodes  → existing tiny-skia pipeline
```

- **Input stays LaTeX**, bridged by `mitex`, so authored equations are standard
  LaTeX (Maxwell pastes in unchanged).
- **Typst is invoked as an external binary**, not compiled in as the `typst`
  crate. Keeps the Rust dependency surface tiny and mirrors the `ffmpeg`
  decision in ADR 0001. Typst is a single lightweight binary, not a multi-GB
  TeX distribution.
- **Output is glyph paths**, satisfying the `Formula` invariant (animatable
  Béziers, never a flat blob, never a raster image).
- Lives in a new **`codimate-math`** crate (deps: `codimate-core`, `mitex`,
  `usvg`, `std::process`). This is a capability crate like `codimate-layout`
  (taffy) and `codimate-render` (tiny-skia); it is **not** the layer-into-crate
  split that ADR 0004 rejects.

### The One-Law boundary (load-bearing)
The Typst subprocess runs **only at Scene-construction time** — `formula("…")`
shells out once, parses the SVG, and bakes the result into static `Path` nodes.
It is **never** invoked inside `resolve(t)`. So `f(t) → Scene` stays pure (no
side effects, no I/O per frame); the subprocess is a one-time asset bake,
analogous to loading a font or image at build time. This is what makes an
external process legal under Invariant 1.

### Caching & determinism
- Compiled output is **cached by content hash** (LaTeX source + Typst version +
  bundled font set), so each unique equation compiles once — the same caching
  Manim uses to hide LaTeX latency.
- Determinism for golden-image tests is achieved by **pinning the Typst
  version** and **bundling/pinning fonts**. This is more reproducible than a
  LaTeX install, though less hermetic than pure Rust — golden images for
  Formulas are gated on the pinned Typst.

## Consequences
- **Real math on day one.** Maxwell-class LaTeX renders immediately and
  correctly, instead of after months of hand-rolled layout.
- **Tiny Rust footprint.** Only `mitex` + `usvg` are added at compile time;
  Typst itself is an external tool the user installs once (like `ffmpeg`).
- **New external prerequisite.** Rendering Formulas requires `typst` on `PATH`.
  Examples without Formulas are unaffected. Document this alongside the existing
  `ffmpeg` requirement.
- **Scope is bounded by mitex/Typst coverage, not by our architecture.** Math
  complexity is no longer the limiting factor; full-LaTeX edge cases that mitex
  doesn't translate are the boundary, and they degrade to a clear error rather
  than a wrong render.
- **Phase-2 symbol addressing is the known hard part.** Animating an individual
  symbol means mapping Typst's SVG glyph output back to source tokens — the same
  "`tex[0][2]`" fiddliness Manim has. Display works first; per-symbol animation
  is a later, separately-scoped effort.
- **Reversible at the seam.** If a healthy pure-Rust LaTeX-math engine appears,
  or we later want a hermetic build, the `codimate-math` boundary (LaTeX in,
  `Path` nodes out) is exactly where a different backend swaps in without
  touching Scenes or examples.
- Supersedes the earlier "no TeX binary / hand-roll a MATH-table subset"
  direction for Formulas.

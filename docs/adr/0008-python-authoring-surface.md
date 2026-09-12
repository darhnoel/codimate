# ADR 0008 — Python is the Authoring Surface; Rust is the Engine

**Status:** Accepted — 2026-09-10

## Context

Codimate's pitch is "you map algorithm events to visuals; it does the rest."
The Rust authoring API does not deliver that. Authoring an explanation means a
six-module split (`state`, `algorithm`, `view`, `motion`, `timing`, `builder`),
a builder chain, `Animated<T>`, Slots, and the three-layer model — before the
first bar is drawn. The promise and the ergonomics had drifted apart, and the
audience for algorithm explainers (educators, TAs, course authors) writes
Python, not Rust.

A prototype in Python showed the whole authoring model fits in ~400 lines with
no `Animated<T>` in sight, because **motion can be derived rather than
authored**: give every shape a stable identity, project each trace event into a
flat Scene, and diff consecutive Scenes to discover what moved.

## Decision

**Python is the only supported Authoring Surface. The Rust crates are the
Engine.** The Rust authoring API (`explain()...`, Slots, `box_in`, Effects) is
no longer a front door; it becomes internal machinery.

The split is **per-event versus per-frame**, not "slow versus fast":

| Python (per event, ~50×) | Rust (per frame, ~1800×) |
|--------------------------|--------------------------|
| state, algorithm, `emit()` | pair shapes by Item, build tweens |
| the view function | interpolate every frame |
| motion rules, durations | rasterize, encode, parallelize |

The boundary is **a data structure, not an API**: one handover at `render()`
carrying a list of flat Scenes (list of dicts), motion rules, and durations.
Nothing crosses the FFI boundary in a loop. Shape `kind` is matched against a
**closed vocabulary** in Rust (rect, circle, text, line, path); an unknown kind
is a Python `ValueError`, not a silent no-op.

Rust reconstructs real `codimate_core` values from the payload —
`Primitive::new(Geometry::rect(tween(w0, w1), tween(h0, h1)))` — so the diff
*produces* `Animated<T>` rather than replacing it. One `Scene` per segment,
sampled with `scene.resolve(t)`. **Invariant 1 (`f(t) → Scene`) is untouched**,
and `codimate-core` needs no changes: identity (`item=`) lives in the payload,
never in `Scene`.

## Considered Options

- **Two front doors (Rust *and* Python as first-class).** Rejected: every
  feature ships twice or the Python side rots into a neglected subset. Twelve
  crates of surface, one pair of hands.
- **Python as a thin export shim** (`render(explanation)` only, authoring stays
  in Rust). Rejected: nobody wants to render someone else's animation.
- **Python drives Rust objects through `#[pyclass]` handles.** Rejected:
  `Animated<T>` is `Arc<dyn Fn(f32) -> T>` — a Rust closure does not cross into
  Python cleanly, and it would force the whole core surface through bindings.
- **Everything is a path** (Python converts shapes to path data before the
  handover, Rust knows one shape type). Elegant, but text→paths needs font
  shaping, which lives in Rust; it pushes the hard part to the wrong side.
  Revisit if the shape vocabulary outgrows ~10 kinds.

## Consequences

- **Motion is implied by identity.** This is the one idea an Explanation Author
  must hold, and the Engine cannot check it: keying an Item by value makes it
  travel, keying it by slot makes it morph in place. Both are legitimate; a
  wrong choice produces a confusing video, never an error. See **Item** in
  CONTEXT.md — the API says `item=`, not `id=`, so the parameter name teaches.
- **The flat `Shape` payload is a union** — a text shape carries unused `w`/`h`.
  Deliberate: it makes the diff a uniform field-by-field comparison instead of
  per-kind special cases. It stops paying off around 8–10 shape kinds.
- **Python's `y` for text means the vertical centre**, converted to a baseline
  inside the binding. Rust keeps baseline semantics; one field secretly meaning
  "baseline" is exactly what makes a framework feel unfinished.
- **The Rust examples become internal tests**, not the teaching path, and
  `docs/daily-workflow.md` is rewritten around Python.
- **PyO3 and `maturin` enter the build**, and shipping means prebuilt wheels per
  OS — a Python user must never need a Rust toolchain.
- **ffmpeg remains an external runtime tool** (ADR 0001), which is now a
  user-facing install requirement rather than a developer one.

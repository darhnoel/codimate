# Codimate — Domain Context

## The One Law
An animation is a pure function from time to a visual scene.
`f(t: f32) → Scene` — NEVER break this invariant.

## The Authoring Model
Codimate animations are authored from the concept outward. Describe the
concept's state, derive a trace from its logic, project each trace moment into a
Scene, then let Layer 3 timing compose the result into `f(t) → Scene`.

## Ubiquitous Language

**Concept**: The idea being explained: a sort, a matrix multiplication, a
network signal flow, a swap. Avoid treating the video timeline as the concept.

**State**: The concept's data at a meaningful point in the explanation. State is
domain data, not visual data and not renderer state.

**Algorithm**: A pure transformation from State into a Trace. For non-algorithmic
topics, this is still the concept logic that decides what happens next.

**Trace**: The ordered explanation events derived from the concept's logic. A
Trace is the script Codimate can regenerate; avoid "hand-authored timeline".

**Trace Event**: One meaningful event in a Trace, such as compare, swap, choose
pivot, compute output cell, or fire signal group. Avoid "keyframe".

**View**: The projection from State plus Trace Event into a Scene. View code
decides what the concept looks like; it does not decide the concept's logic.

**Motion**: Timeless movement/styling choices used by a View, such as easing,
paths, reveals, pulses, and style transitions. Motion has no duration.

**Timing**: The Layer 3 durations assigned to Trace Events and holds. Timing is
where pacing lives; never hide duration inside Motion or View.

**Explanation**: A composed animation built from State, Algorithm, View, Motion,
and Timing. It is the authoring-level object that eventually renders as a
Playable.

**Animated<T>**: A value that resolves at time `t ∈ [0.0, 1.0]`.
A plain value is trivially `Animated<T>`. Never say "keyframe value".

**tween(a, b)**: A Layer 1 builder for an `Animated<T>` that travels from `a`
(at `t = 0.0`) to `b` (at `t = 1.0`) by interpolation. **Timeless — takes NO
duration argument.** How long the travel takes is decided later in Layer 3.
Endpoints are `impl IntoAnimated<T>` (Invariant 7).

**Easing**: A pure remap of `t` through a curve (`f32 → f32`), applied via
`Animated::ease`. Layer 1, timeless. `f(t)` becomes `f(curve(t))`. Overshoot
curves (e.g. `back`) deliberately produce values outside `[0,1]`, which flow
into `tween`'s intended extrapolation — this is not an Invariant 2 violation,
since the eased value still *receives* `t ∈ [0,1]` from its Animation context.

**Animation**: A named thing with a `duration: f32` and a Scene that resolves
with normalized `t`. Never say "clip", "track", or "timeline object".

**Scene**: A tree of Nodes where every stylistic property is `Animated<T>`.
Never say "stage", "canvas state", or "frame data".

**Node**: Pure data. Has layout properties and style properties.
Never say "object", "element", or "sprite".

**Style**: A timeless, lerpable bundle of visual style leaf values:
`fill: Color`, `stroke_width: f32`, and `stroke_color: Color`. A `Style` holds
plain values, not `Animated` fields; animate a coordinated look by tweening
between two Styles.

**Connection**: A Node (Layer 2) that links two shape Anchors with a line,
optionally ending in an arrowhead. It derives its geometry from its endpoints at
resolve time, so it tracks the shapes as they move. "Connection" is the canonical
term — not "edge", "link", "wire", or "arrow" (an arrowhead is a feature of it).

**Anchor**: A point on a shape's boundary (top, bottom, left, right, center),
resolvable at `t` as an `Animated<Vec2>`. Connections attach to Anchors so they
follow the shape as it animates.

**Port**: An evenly-divided Anchor slot along an edge, addressed as "slot i of n"
(e.g. a bottom edge split into 3 for three incoming Connections) so fan-in/out
does not overlap on a single point. Ports are stated explicitly for now;
automatic allocation is a future layout concern.

**Pulse**: A marker (a dot) that travels along a Connection's path as a progress
value goes 0→1, showing flow / "firing". The Connection (line + arrowhead) stays
fully drawn the whole time; the Pulse is an *overlay* on top — it does not reveal
the line. Positioned by a point a fraction along the path, measured by arc length.

**ConcreteScene**: A Scene resolved at a specific `t` — all values are plain `f32`,
`Color`, `Vec2`, etc. Produced by `scene.resolve(t)`.
Never say "snapshot" or "frame state".

**Sequence**: A named Layer 3 Composition that plays Animations back-to-back.
Child Animation timestamps are local to that child; boundaries are hard cuts.

**Parallel**: A named Layer 3 Composition that plays Animations at the same
time. Duration is the longest child duration; shorter children hold their final
state.

**Stagger**: A named Layer 3 Composition that starts Animations at fixed time
offsets. Not-yet-started children are absent; finished children hold final state.

**Playable**: A Layer 3 value with `name`, `duration`, `resolve(t)`, and
`resolve_at(seconds)`. Preview/export code should accept `impl Playable` when it
can sample any Animation or Composition.

**Composition**: Combining Animations in time via `sequence`, `parallel`,
`stagger`.
Never say "sequencer", "timeline", or "animation graph".

**Duration**: Lives in Layer 3 (Composition) ONLY.
Layer 1 (Value) and Layer 2 (Scene) are timeless.

## The Three Layers

| Layer | Name        | Responsibility                          | Key Type       |
|-------|-------------|-----------------------------------------|----------------|
| 1     | Value       | How a single value changes over t       | `Animated<T>`  |
| 2     | Scene       | What exists at a moment in time         | `Node`, `Scene`|
| 3     | Composition | How animations combine in time          | `Animation`    |

**Rule**: Every PR touches exactly one layer. If a change spans two layers,
it needs two PRs. If you cannot place a feature in one layer, it does not
belong in Codimate yet.

## Crate Structure

codimate/
├── crates/
│   ├── codimate-core/      # Layer 1 + 2 — no I/O, no Skia
│   ├── codimate-animation/ # Layer 3 — Animation duration + composition
│   ├── codimate-layout/    # taffy integration, layout pass
│   ├── codimate-render/    # tiny-skia CPU raster, Renderer trait (see ADR 0001)
│   ├── codimate-previewer/ # interactive preview window, sampled from Playable
│   └── codimate-export/    # raw RGBA -> ffmpeg pipe (PNG optional, see ADR 0001)
└── examples/

**codimate-core has zero non-pure dependencies.** If a PR adds an I/O import
to codimate-core, reject it.

## Invariants (Never Violate)

1. `f(t) → Scene` is always pure — no side effects, no mutation
2. `t` is always normalized to `[0.0, 1.0]` within any Animation context
3. Nodes do not render themselves
4. The render pipeline is strictly one-directional — no feedback to Scene
5. Duration lives in Layer 3 only
6. `codimate-core` has no I/O dependencies
7. Every public API accepts `impl IntoAnimated<T>` not `Animated<T>` directly

## Formatting Conventions

Use standard `cargo fmt` output. The root `rustfmt.toml` only pins stable,
project-wide defaults: Unix newlines, 100-column width, and 2021 style edition.
Do not add nightly-only rustfmt options.

In docs and examples, use named multi-line builder chains when a value is reused
or carries domain meaning:

```rust
let rest = Style::new()
    .fill(Color::WHITE)
    .stroke(1.0, Color::BLACK);
```

Inline builder chains only for tiny one-offs where naming would add noise.

## Out of Scope (Do Not Implement)
- Stateful particles (violates pure f(t))
- Physics simulation (frame-dependent state)  
- Audio synchronization (out of scope v1)
- 3D rendering (Skia is 2D, keep it that way)
- GUI editor (preview window is viewer only)

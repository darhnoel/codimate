# Codimate — Domain Context

## The One Law
An animation is a pure function from time to a visual scene.
`f(t: f32) → Scene` — NEVER break this invariant.

## Ubiquitous Language

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

**ConcreteScene**: A Scene resolved at a specific `t` — all values are plain `f32`,
`Color`, `Vec2`, etc. Produced by `scene.resolve(t)`.
Never say "snapshot" or "frame state".

**Sequence**: A named Layer 3 Composition that plays Animations back-to-back.
Child Animation timestamps are local to that child; boundaries are hard cuts.

**Composition**: Combining Animations in time via `sequence`, `par`, `stagger`.
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
│   ├── codimate-core/      # Layer 1 + 2 — no I/O, no Wayland, no Skia
│   ├── codimate-animation/ # Layer 3 — Animation duration + composition
│   ├── codimate-layout/    # taffy integration, layout pass
│   ├── codimate-render/    # skia-safe, Renderer trait + SkiaRenderer
│   ├── codimate-wayland/   # live preview window, frame callbacks
│   └── codimate-export/    # PNG frames, ffmpeg pipe
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

## Out of Scope (Do Not Implement)
- Stateful particles (violates pure f(t))
- Physics simulation (frame-dependent state)  
- Audio synchronization (out of scope v1)
- 3D rendering (Skia is 2D, keep it that way)
- GUI editor (preview window is viewer only)

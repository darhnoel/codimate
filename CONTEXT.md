# Codimate — Domain Context

## The One Law
An animation is a pure function from time to a visual scene.
`f(t: f32) → Scene` — NEVER break this invariant.

## The Authoring Model
Codimate animations are authored from the concept outward. Describe the
concept's state, derive a trace from its logic, project each trace moment into a
Scene, then let Layer 3 timing compose the result into `f(t) → Scene`.

## Slot Model
A Slot is a View-only layout position: a centre point and a size. It is not
concept state, not a shape, and not a runtime layout pass. Nothing draws a
Slot.

- `cm.row(items, gap=)` divides the canvas into one Slot per item, centred,
  sized from the canvas unless overridden.
- A Slot exposes the edges a View actually asks for: `.x`, `.y`, `.w`, `.h`,
  `.left`, `.right`, `.top`, `.bottom`.
- Shapes are placed by **anchor**, not by centre arithmetic: give one
  horizontal anchor (`x`/`left`/`right`) and one vertical anchor
  (`y`/`top`/`bottom`). Two anchors on one axis is an error, never a silent
  winner.
- **Items are concept identity; Slots are visual positions.** An Item moving
  from one Slot to another is still the same Item — that separation is what
  makes motion derivable.

Use Slots in View code before reaching for raw coordinates. `cm.width()` and
`cm.height()` cover the occasional hand-placed element; repeated alignment
math belongs in a Slot.

## Ubiquitous Language

**Concept**: The idea being explained: a sort, a matrix multiplication, a
network signal flow, a swap. Avoid treating the video timeline as the concept.

**Explanation Author**: The primary human actor Codimate optimizes for. This is
a **Python developer** who can express a concept as an algorithm and wants a
reliable daily workflow from concept to preview/export without learning
renderer internals — or Rust — first. An Explanation Author never writes Rust
and never encounters `Animated<T>`, Slots, Effects, or the three-layer model.

**Authoring Surface**: The Python API an Explanation Author writes: `emit`,
`trace`, `Scene`, `Rule`, `Timing`, `explain`. It is the only supported way to
author an explanation. The Rust crates are the **Engine** — machinery the
Authoring Surface drives, not a second front door.

**Engine**: The Rust crates behind the Authoring Surface. The Engine owns every
per-frame concern (diffing, interpolation, rasterization, encoding); the
Authoring Surface owns every per-event concern (state, trace, view, motion
rules, durations). "Hard things in Rust, easy things in Python" means exactly
this split — per-frame versus per-event, not fast versus slow.

**Primary Job (Authoring)**: Build a new explanation from scratch by writing
one Python file containing the four pieces — algorithm, view, motion, timing —
and repeatedly rendering until the concept reads clearly. There is no module
split to learn; a whole explanation fits in a single file.

**Canonical Onboarding Workflow**: The single recommended path an Explanation
Author follows from first run to first custom explanation. This workflow is
documentation-first, maps to one starter example
(`python/examples/bubble_sort.py`), and must match runnable commands in the
repository.

**Onboarding Success Metric**: An Explanation Author can go from clone to first
custom animation in less than 30 minutes via the Canonical Onboarding Workflow,
without writing or reading Rust.

**Item**: Concept identity — the thing an Explanation Author is talking about,
carried across Scenes by a stable name on every shape and Group. Motion is
**implied by identity**: the Engine pairs shapes by name between consecutive
Scenes, and whatever changed becomes a tween. A name keyed to the *thing*
travels; a name keyed to the *place* never moves and only changes shape. Both
are legitimate; choosing between them is the Explanation Author's single most
consequential decision, and it is never an error the Engine can catch.

`cm.items(values)` is how a bare scalar acquires identity: two 3s in a list are
two different bars, and only a wrapper can say so. An **Item** orders by
`.value` (so the Algorithm stays ordinary Python) and is equal by `.id` (so it
survives the deep copy taken at every event — comparing by `is` would not).
Identity keyed to a place needs no wrapper: `("cell", row, col)` is already
unique and already stable.

Names are given explicitly, never inferred from an object. Reading an `.id`
off a passed object can be added later; a convention, once shipped, can never
be removed.

**Group**: A named place to draw a thing made of several shapes. A Group has
its own origin and its own name; children are named beneath it (`3/bar`,
`3/label`), so **everything on a Group moves as one thing** and a motion rule
cannot pull a thing's parts apart. Groups nest. Inside a Group, `0` is that
Group's anchor point — nothing is inherited from the parent and nothing is
decided implicitly, which is what keeps nesting free of rules to learn. A
Scene is the root Group.

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

**Slot**: A View-only visual position — a centre point and a size, exposing its
edges. Slots are authoring-time helpers in the Authoring Surface; they are not
shapes, not concept State, and not Engine objects, and they carry no identity.
Use Slots to derive rows, labels, and group positions before placing shapes.
Contrast with [Item]: a Slot is *where*, an Item is *what*.

**Anchor (Authoring Surface)**: The edge or centre an Explanation Author uses
to place a shape — one horizontal (`x`, `left`, `right`) and one vertical
(`y`, `top`, `bottom`). Anchors exist so an author never converts an edge into
a centre by hand. Ambiguous or missing anchors are rejected. Distinct from the
Engine's **Anchor** (a point on a shape's boundary that a Connection attaches
to); the two never meet, because Connections are not in the Authoring Surface.

**Box**: A reusable View authoring component for a styled rectangular visual
area, usually positioned by a Slot. A Box may have a corner radius, fill, stroke
width, and stroke color. "Box" is the canonical authoring term; "rounded
rectangle" describes the geometry used to draw it. Box starts as View-layer
authoring sugar that returns a Scene Node; it is not a new core Node variant.
Its styling mirrors `PathNode`: prefer `.style(...)`, allow `.fill(...)` and
`.stroke(...)`, and let builder order decide overrides. Use `box_in(&slot)` for
a Slot-sized Box, and `box_at(center, size)` for a moving center-positioned Box.

**Motion**: Timeless movement/styling choices used by a View, such as easing,
paths, reveals, pulses, and style transitions. Motion has no duration.

**Effect**: A pure, timeless authoring recipe for a visual change, such as fade,
write, transform, indicate, or grow. An Effect resolves with normalized `t` into
a Scene, accepts Scene-shaped input in v1, has no duration by itself, and never
mutates Nodes or renderer state. Effect is a concrete authoring value, not a
trait hierarchy. `Effect::ease` reuses the same Easing curves as
`Animated<T>::ease`. Timing turns an Effect into an Animation at an explicit
`effect.animate(name, duration)` boundary.

**Manim Behavior Parity**: Manim's `manimlib.animation` modules are a behavior
reference, not an API or lifecycle contract. Codimate may provide pure Effect or
Composition equivalents for useful behavior families, but it does not copy
Manim's mutable `begin`/`finish` lifecycle, scene cleanup side effects, mobject
updaters, or method-animation mutation.

**Effect Boundary**: Effects support View and Motion authoring. They do not
replace Concept, State, Algorithm, Trace, Trace Event, or Timing. Educational
examples still derive meaningful Trace Events first; Effects only describe how a
Scene appears or changes within one event.

**Transform Effect**: An Effect that interpolates one Scene into another. In v1,
Transform requires matching Scene structure and matching Node counts; Codimate
does not guess correspondences between different Scene shapes. Mismatched
Transforms are rejected explicitly rather than silently approximated.

**Fade Effect**: An Effect that changes Scene opacity without adding or removing
Nodes. `fade_in` resolves from transparent styling to the original Scene, and
`fade_out` resolves from the original Scene to transparent styling.

**Reveal Effect**: An Effect that makes a Scene visible over local `t` in a
Codimate-native way. Reveal may draw path-like geometry progressively and fade
non-path Nodes, while keeping Scene structure stable. Reveal is the Codimate
name for the useful behavior family behind Manim's creation/write animations.
In v1, PathNode and Connection reveal by path prefix, while Circle, Rect, Text,
and Pulse fade in.

**Scene Opacity**: A pure Scene-level visual operation that multiplies the alpha
of every color-bearing Node without changing Scene structure. Effects may use
Scene Opacity, but it is not Effect-specific.

**Easing (Authoring Surface)**: `cm.ease(t)` returns the Engine's own curve.
It exists so an Explanation Author drawing or reasoning about pacing calls into
the Engine instead of keeping a second copy that can silently drift out of
agreement with what the animation actually does.

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

**Formula**: A typeset mathematical expression (e.g. `F = ma`, `F_{net} = 0`,
`\vec{F}`). Authored as a **LaTeX-subset string** and realized as a
[`GlyphBlock`] — every symbol is a first-class animatable Path — never a flat
text blob and never a rendered image. An "equation" is just a Formula that
contains `=`; prefer "Formula" as the general term. Not "label", "tex",
"MathText", or "math image".

**GlyphBlock**: A [`codimate_core::GlyphBlock`] — a group of animatable
`PathNode`s with an overall width/height. Produced by both
[`codimate_math::formula()`] (math → paths) and
[`codimate_glyph::shape()`] (text → paths). Each glyph is a standalone
`PathNode` that can be tweened, styled, or transformed independently.

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
│   ├── codimate-effects/   # Pure timeless visual change recipes + explicit Timing adapter
│   ├── codimate-layout/    # taffy integration, layout pass
│   ├── codimate-fonts/     # Central font registry with Unicode coverage
│   ├── codimate-glyph/     # Text -> animatable glyph paths (harfbuzz + ttf)
│   ├── codimate-math/      # Formula: LaTeX -> Typst subprocess -> Paths (see ADR 0005)
│   ├── codimate-render/    # tiny-skia CPU raster, Renderer trait (see ADR 0001)
│   ├── codimate-previewer/ # interactive preview window, sampled from Playable
│   ├── codimate-export/    # raw RGBA -> ffmpeg pipe (PNG optional, see ADR 0001)
│   └── codimate-py/        # PyO3 bindings — the Authoring Surface (see ADR 0008)
├── python/                 # the `codimate` Python package
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

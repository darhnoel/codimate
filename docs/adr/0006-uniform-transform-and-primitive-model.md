# ADR 0006 — Uniform Transform + three-slot Primitive (the motionity import)

**Status:** Accepted — 2026-06-09

## Context
Onboarding a newcomer is harder than it should be, and the cause is structural,
not cosmetic. Each Scene Node (`Circle`, `Rect`, `Text`, …) hand-rolls its own
positioning fields (`x`, `y`), there is no rotation or scale anywhere, and
"opacity" is faked by multiplying into `fill.a` via a `with_opacity` hack. Three
node types carry only `fill: Color`; `PathNode` alone carries the full `Style`
(fill + stroke). `Pulse` exists as its own node type only because circles could
not previously scale or fade. This is the "implementations mixed here and there"
the project owner called out.

The reference brief was: take motionity's *fundamental components* and apply them
without breaking the One Law (`f(t) → Scene`) or the authoring model. motionity
is a GUI keyframe editor; its timeline/keyframes do **not** port — that role is
already played, more purely, by `Animated<T>` + `tween` + the Layer 3
composition primitives. What *is* fundamental and genuinely missing is the thing
every fabric.js object shares identically: a **uniform transform**
(`left/top, scaleX/scaleY, angle, opacity, originX/originY`) plus a small closed
set of shape types with a shared appearance model.

The goal is **model-first as the means to ergonomics**: fix the model so the
beginner surface becomes small as a consequence, not paper over the mess with
builders.

## Decision

### 1. Every primitive is `{ transform, style, geometry }` (three slots)
Replace the flat, per-node field soup with a wrapper that separates the two
*universal* concerns from the one *type-specific* concern:

```rust
struct Primitive {
    transform: Transform,        // universal — placement & visibility
    style:     Animated<Style>,  // universal — fill + stroke + stroke_width
    geometry:  Geometry,         // type-specific — PURE shape only
}
enum Geometry { Circle { radius }, Rect { width, height }, Path { path },
                Text { text, font_size, font } /* , Image (ADR 0007) */ }
```

Geometry is authored in **local space** (centered on its own origin); the
transform places it in the world. This is exactly fabric's object model and is
the single change that gives rotation/scale/opacity/stroke to *all* shapes at
once.

### 2. `Transform` — the uniform transform
```rust
struct Transform {
    pos:      Animated<Vec2>,   // translation (px)     default (0,0)
    scale:    Animated<Vec2>,   // non-uniform allowed   default (1,1)
    rotation: Animated<f32>,    // DEGREES               default 0
    opacity:  Animated<f32>,    // 0..1                  default 1
    pivot:    Anchor,           // pivot for scale+rot   default Center
}
```
- `scale` is `Vec2` (matches `scaleX/scaleY`); a `.scale(2.0)` helper sets both
  axes, `.scale_xy(x, y)` for the non-uniform case.
- `rotation` is in **degrees** at the authoring surface — a deliberate
  beginner-first choice (motionity uses degrees; `.rotate(45.0)` reads as
  forty-five degrees). Converted to radians only at the tiny-skia boundary;
  `codimate-math` stays radians internally. Unit choice does not touch purity.
- `opacity` lives on the transform (not on `Style`/geometry) because it must
  travel with the object and multiply down a future group tree. This deletes the
  `with_opacity → fill.a` hack entirely.

### 3. Center-origin local space; `.pivot()` writes, `.anchor()` reads
Local `(0,0)` is the **center** of every geometry (circle center, rect center,
text box center). The word "anchor" split into two roles that previously shared
one name:
- `.pivot(AnchorKind)` — **sets** the transform pivot (default `Center`).
- `.anchor(AnchorKind)` — **queries**, returns `Animated<Vec2>` in **world**
  space, now resolved *through* the transform. `Connection` keeps using this
  unchanged (ADR 0003 is preserved).

One `AnchorKind { Center, Top, Bottom, Left, Right }` vocabulary serves both the
write (pivot) and the read (boundary query). The configurable `pivot` field ships
in v1 (default `Center`) even though most authors never touch it: it must exist
so rotation/grouping compose correctly later, and retrofitting it after grouping
exists is expensive.

### 4. `Style` becomes universal
`Style { fill, stroke_width, stroke_color }` already exists, is `Animated`, and
is `Lerp`-able. Promote it to the appearance leaf of **every** primitive; delete
the bare `fill: Color` on `Circle`/`Rect`/`Text`. `.fill(c)` and
`.stroke(w, c)` are convenience setters over `style`. Morphing fill+stroke now
comes for free.

### 5. The node taxonomy collapses from six peers to two kinds
```rust
enum SceneNode { Primitive(Primitive), Connection(Connection) }
```
- **`Connection` stays relational** — its endpoints *are* other nodes' world
  anchors; it has no transform of its own. ADR 0003 unchanged.
- **`Pulse` dissolves.** A pulse is a circle with animated `scale` (grow) and/or
  `opacity` (fade), or a small circle whose `pos` is driven along a path via
  `point_at(progress)` — all now expressible. We keep a one-line `pulse(...)`
  **constructor helper** that returns a pre-animated `circle()`, but it stops
  being a `SceneNode` variant. The six-arm `match` boilerplate in `scene/mod.rs`
  (`with_opacity`, `ease`, `reveal`, `try_lerp_to`, `kind`) collapses toward the
  transform/trait layer.

### 6. Universal setters come from one prelude trait — no macro, no duplication
`.pos / .x / .y / .scale / .rotate / .opacity / .pivot / .fill / .stroke` are
default methods on a single `Transformable` trait (widened to cover `style`),
implemented by each primitive via `transform_mut()` + `style_mut()`. Geometry
setters (`.radius()`, `.width()`) stay inherent. Everything returns `Self`, so a
beginner chains geometry and universal setters in any order without knowing which
is which. `.x()`/`.y()` are kept (continuity) alongside `.pos((x, y))`; all
accept `impl IntoAnimated<…>`, so `.x(100.0)` and `.x(tween(0.0, 400.0))` are the
same method. `(f32, f32): Into<Vec2>`.

### 7. Concrete output is decomposed; the renderer applies it
`resolve()` stays pure and emits decomposed concrete data — it does **not** bake
a matrix:
```rust
struct ConcretePrimitive { transform: ConcreteTransform, geometry: ConcreteGeometry }
struct ConcreteTransform { pos: Vec2, scale: Vec2, rotation_deg: f32, pivot: Vec2, opacity: f32 }
```
The renderer composes the tiny-skia matrix
(`translate(pos) · rotate(deg, pivot) · scale(pivot)`) — tiny-skia already draws
with a `Transform` (today hardcoded `identity()`); that call site becomes the
home of the matrix. Opacity is applied as **paint alpha** at draw time.

**Known seam (decided, not forgotten):** decomposed `ConcreteTransform` becomes a
**2×3 matrix** the day grouping/parenting lands, because composing a parent's
non-uniform-scale + rotation into a child produces shear, which a decomposed
struct cannot represent. Opacity stays out of the matrix regardless (it is not
affine). This is a pure-data, render-boundary migration — no model change.

**Known limitation:** paint-alpha opacity can double-composite at overlaps within
a multi-part draw (stroke+fill, overlapping glyphs). True group opacity needs
render-to-layer; deferred as a renderer-only upgrade.

### 8. A facade crate `codimate` is the newbie's single entry point
New crate `codimate` re-exports `core` + `animation` + an export prelude and
provides a one-line `render(&movie, "out.mp4")` with sensible defaults. A
beginner gets **one dependency** and **one `use codimate::*;`**, replacing the
3-crate dependency set and the ~50-line `main.rs` with `ExportConfig`/presets.

### 9. Crate structure — consolidate into layers, do not fragment
- `Transform`, `Anchor`, `Geometry`, `Primitive` live in **`core::scene`**, beside
  `Scene`. No new `codimate-transform` crate — fragmenting Layer 2 is the disease,
  not the cure (consistent with [ADR 0004](0004-layers-as-modules-not-crates.md)).
- `codimate-effects` **shrinks hard**: `opacity`/`reveal`/`ease` become universal
  trait/`Transform` methods, so most of `Effect` is now dead weight. Gut it to any
  genuinely compound recipes that remain; stop teaching it to beginners.
- `codimate-arrange` stays as an **advanced** crate (domain diagram math), not in
  the prelude.

## The bar this refactor must hold
```rust
use codimate::*;

fn main() {
    let hello = animation("hello", 2.0,
        scene().add(
            circle()
                .radius(50.0)
                .fill(Color::RED)
                .x(tween(0.0, 400.0))      // slide right
                .rotate(tween(0.0, 360.0)) // spin once
        ),
    );
    render(&hello, "hello.mp4");
}
```
A `Scene` is the instantaneous, **nameless** picture (`scene().add(...)`). Names
and duration live one layer up on `animation(...)`; named *segments* come from
`sequence(...)`, never from naming a scene.

## Consequences
- Rotation, scale, opacity, and stroke arrive for **all** shapes in one change.
- `ellipse()` = `circle().scale_xy(2,1)`, `line()`/`polygon()` = `Path` sugar —
  "more shapes" is mostly the payoff of the wrapper, not new node types.
- `SceneNode` drops from 6 arms to 2; the per-node `match` boilerplate collapses.
- **Breaking:** every node's authoring API changes and all ~30 examples must be
  migrated. Accepted deliberately (model-first). Geometry is now local-space, so
  `circle().x(100).y(200)` means "place by transform", not "center field".
- `with_opacity`/`fill.a` hack and the `Pulse` node type are deleted.

If a future reviewer proposes folding `style` back into geometry or splitting
transform into its own crate, the bar to clear is the same as ADR 0004: has the
three-slot split actually caused harm, or is this aesthetic? Absent harm, keep it.

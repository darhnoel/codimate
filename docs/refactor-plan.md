# Refactor Plan — uniform transform, primitive model, media channel

Companion to [ADR 0006](adr/0006-uniform-transform-and-primitive-model.md)
(model) and [ADR 0007](adr/0007-media-channel-audio-video-subtitle.md) (media).

**North star:** model-first as the means to ergonomics. The success metric is the
hello-world bar below compiling and rendering.

```rust
use codimate::*;

fn main() {
    let hello = animation("hello", 2.0,
        scene().add(
            circle().radius(50.0).fill(Color::RED)
                .x(tween(0.0, 400.0))
                .rotate(tween(0.0, 360.0)),
        ),
    );
    render(&hello, "hello.mp4");
}
```

## Build order (all must-have; they cannot be built at once)

Phase 2 *requires* Phase 1's clean primitive and the facade's `render()` seam to
hang the audio channel on. Doing them together means designing the timeline
against a model that is still moving.

### Phase 1 — pure model refactor (no new media subsystem)
1. **`Transform` in `core::scene`** — fields per ADR 0006 §2; all `Animated`.
   `AnchorKind` reused for `pivot`. Degrees authored, radians at render.
2. **`Style` universalized** — delete `fill: Color` on `Circle`/`Rect`/`Text`;
   every primitive carries `Animated<Style>`.
3. **Three-slot `Primitive { transform, style, geometry }`**; `Geometry` enum =
   pure shapes (`Circle`, `Rect`, `Path`, `Text`). Geometry authored local-space,
   center-origin.
4. **`SceneNode { Primitive, Connection }`** — collapse the six-arm enum;
   `Connection` unchanged; **delete `Pulse`** node type, keep `pulse(...)` helper.
5. **`Transformable` prelude trait** — default `.pos/.x/.y/.scale/.rotate/`
   `.opacity/.pivot/.fill/.stroke` via `transform_mut()`/`style_mut()`. No macro.
6. **`scene().add(...)`** rename from `.node(...)`; Scene stays nameless.
7. **Concrete output** — `ConcretePrimitive { transform, geometry }`, decomposed
   `ConcreteTransform`. Renderer composes the tiny-skia matrix (replace the
   hardcoded `identity()`); opacity = paint alpha.
8. **`.pivot()` write / `.anchor()` read** split; `.anchor()` resolves through the
   transform; verify `Connection` still tracks shapes.
9. **Free shapes as sugar** — `ellipse()` (= circle + non-uniform scale),
   `line()`, `polygon()` over `Path`. No new `Geometry` arms.
10. **`Geometry::Image { src, w, h }`** — pure data in `core`; tiny-skia decodes
    PNG/JPEG at render. (Image is static, no timeline → rides Phase 1.)
11. **Subtitle = timed `Text`** + `.srt → Vec<Text>` loader sugar.
12. **Facade crate `codimate`** — re-export prelude + `render(&playable | &movie)`.
13. **Shrink `codimate-effects`**; keep `codimate-arrange` out of the prelude.
14. **Migrate all ~30 examples** to the new API; the hello-world is the canonical
    smoke test. Update `docs/authoring-model.md`.

### Phase 2 — the media channel (new architecture on the Phase-1 foundation)
1. **`Clip { source, in, out, rate }`** + **`AudioTrack`** (seconds-based) in
   `codimate-animation`.
2. **`Movie { visual, audio }`**; `render(&movie, ...)`. Silent animation = empty
   `AudioTrack`. Keep `render(&playable, ...)` working (wraps into a silent Movie).
3. **`Geometry::Video`** — sample source frame at scene-`t`; `codimate-render`
   gains the ffmpeg-side video decoder dependency.
4. **`video("x.mp4")` fan-out** — one call emits `Geometry::Video` (picture) + an
   audio `Clip` (sound).
5. **`codimate-export`** — audio decode + A/V mux via ffmpeg; reconcile
   seconds-based media against normalized `t`.

## Known seams (decided, recorded so they are not surprises)
- `ConcreteTransform` decomposed → **2×3 matrix** when grouping/parenting lands
  (non-uniform-scale + rotation composition produces shear). Opacity stays out of
  the matrix. Pure-data, render-boundary change.
- Paint-alpha opacity can double-composite on multi-part draws; true group opacity
  needs render-to-layer (renderer-only upgrade, deferred).
- **Grouping/parenting** itself is deferred; the `pivot` field and decomposed
  concrete transform are designed so it slots in without a model change.

## Out of scope (explicitly)
- motionity's GUI: timeline UI, keyframe handles, layers/properties panels.
  Codimate is code-authored; `Animated<T>`/`tween`/Layer 3 replace keyframes.
- Automatic arrangement/routing (still deferred per [ADR 0003](adr/0003-pure-local-connections.md)).

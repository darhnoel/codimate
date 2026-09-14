# ADR 0012 — Curves, fitted through their points rather than authored as control points

**Status:** Accepted — 2026-09-14

## Context

Python cannot draw a curve. Every path an author can make is straight-edged:
`polygon` emits `Segment::Line` and nothing else, `line` is one segment, and
`rect`/`circle` are their own primitives.

The Engine has had curves all along. `Segment::Quad` and `Segment::Cubic` are
defined, interpolated by the tween, handled by `render_path`, and used on every
frame — they are how a rounded rectangle's corners are drawn. So this is the
fifth capability found finished in the Engine with nothing exposing it, after
LaTeX, text measurement, fill-plus-stroke and polygons.

The absence is not theoretical. Every example that wants a curve fakes one:

- `explain_codimate/curve.py` draws an easing curve as 24 straight lines.
- `bernoulli_lift` draws its streamlines as hundreds.
- `fourier_einstein` traces a portrait with 1,600.
- `docs/drawing.md` documents the workaround as a technique — *"A curve — from
  short straight pieces"* — which is the library teaching its own limitation.

It also costs speed. Profiling the exporter put 42% of render time in
`stroke_path`, and that cost tracks path segment count. A portrait traced with
a few dozen Béziers is less work than the same portrait traced with 1,600
lines, on every frame.

## Decision

**Add one kind, `curve`, whose `points` are points the curve passes through.**

    scene.curve("plot", [(x0, y0), (x1, y1), ...])
    scene.curve("loop", corners, closed=True)

The author gives samples. The Engine fits a smooth curve through all of them
and emits `Segment::Cubic`.

### No new payload field

`polygon` already carries a flat `points: Vec<f32>`, and already reads `w` as
its closed flag. `curve` reuses both unchanged. The payload that ADR 0010
argued so hard over does not grow by a single field — this is a new *reading*
of data already there, the way `r` is a circle's radius and a rect's corner
rounding.

That also means curves tween for free. `points` is diffed field by field like
everything else, so two curves with the same number of points interpolate and
the fitting happens after; two with different counts fall back to the later
shape, exactly as `polygon` and `text` already do.

### Through the points, not control points

A cubic Bézier is normally authored as endpoints plus two control points that
the curve does *not* touch. That is right for designing a glyph and wrong for
every case in this repository, all of which are the same shape: *here are my
samples, draw a smooth line through them.* Nobody plotting `ease(t)` wants to
solve for control points.

So `points` are on-curve, and the Engine derives the control points with a
uniform Catmull-Rom construction:

    c1 = p[i]     + (p[i+1] - p[i-1]) / 6
    c2 = p[i+1]   - (p[i+2] - p[i])   / 6

End points are handled by reflecting the neighbouring point, so an open curve
starts and ends where it is told. It is deterministic, local, and passes
through every input point — which is the property that makes it checkable: a
test can assert the curve touches its samples.

Authoring exact control points stays unavailable. If a case turns up that
genuinely needs them, it is a separate argument and a separate kind; making
`points` mean "on-curve, unless a flag says otherwise" would be the sort of
double meaning ADR 0010 rejected.

### A curve is not a smoothed polygon

`polygon` keeps emitting straight lines. Smoothing it with a flag would change
what existing examples draw, and a triangle is not a thing anyone wants
rounded off. They are different shapes with different names.

## Consequences

- **`scene.curve` replaces the loop.** `explain_codimate/curve.py` becomes one
  call rather than 24, and `docs/drawing.md` loses a section that taught people
  to approximate.
- **Fewer segments to stroke.** The same picture with an order of magnitude
  fewer path segments, on every frame. The exporter's hottest cost is
  proportional to that.
- **A curve through two points is a straight line**, which is correct and means
  `curve` is never wrong to reach for, only sometimes unnecessary.
- **The spline is fixed.** No tension or alpha parameter. Centripetal
  Catmull-Rom avoids the cusps uniform Catmull-Rom can produce when samples are
  unevenly spaced, and is the obvious future change if anybody hits one; until
  then it is a knob nobody has asked for.
- **`KINDS` grows to seven.** The architecture review quoted in ADR 0010
  predicted the flat union would strain "around 8–10 kinds". This is the
  seventh, and unlike `polygon` it costs the payload nothing, but the count is
  worth watching.

## Alternatives rejected

**Author control points explicitly.** More expressive, and what SVG does.
Rejected because every case in the repository wants interpolation, and an API
that makes the common case hard to get right is worse than one that cannot
express the rare case at all.

**A `smooth=True` flag on `polygon`.** No new kind, and the payload already has
spare floats. Rejected: it makes one name mean two shapes, and a reader cannot
tell from `scene.polygon(name, pts, smooth=True)` whether the result touches
its points.

**Fit the curve in Python and send the control points.** Keeps the Engine
unchanged and needs no new kind — `polygon` could carry them. Rejected because
the tween would then interpolate control points rather than samples, so two
curves morphing would move through shapes that pass through neither set of
points, and because it puts geometry in the Authoring Surface that ADR 0008
says belongs in the Engine.

**Do nothing.** The workarounds do produce correct pictures, and at screen
resolution nobody counts the segments. Rejected because the cost is paid by
every author forever, on a primitive every drawing tool has, and because the
Engine has been able to do it since the first commit.

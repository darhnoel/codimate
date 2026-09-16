# ADR 0016 — Axes, which hand back pixels rather than owning the plot

**Status:** Accepted — 2026-09-16

## Context

Codimate cannot draw a graph, and every example that wants one builds the
arithmetic again. `bernoulli_lift` maps aerofoil coordinates to the screen,
`galton_board` maps bin counts to bar heights, `pendulum` maps an angle to a
sector. None of them shares a line of it, and none of them draws a labelled
axis, because the labelling is the tedious part and each example only needed
the mapping.

The request that prompted this is a comparison with Manim, which ships `Axes`,
`NumberPlane` and `ThreeDAxes`. An author coming from there reasonably asks
where the equivalent is.

The answer must not be "a chart object", and the reason is the whole design.
Motion here is derived from **names** (ADR 0008): two consecutive Scenes are
diffed by stable identity and the differences become tweens. A Manim-style
`Axes` owns its submobjects and hands back a plot object; the names would be
inside it, and the author could not reach them. A curve the author cannot name
is a curve the author cannot animate, focus, or route a motion rule at — which
is everything the library does.

## Decision

**`cm.axes(...)` returns a coordinate map, not a drawing.** Its job is to turn
data into pixels. What is drawn with those pixels stays the author's, named by
the author, in the ordinary way.

```python
plot = cm.axes(x=(-4, 4), y=(-2, 6), at=cm.at(x=640, y=380), size=(760, 420))

plot.draw(scene)                                   # frame, ticks, labels
scene.curve("f", plot.line(lambda x: x * x), w=4).fill("orange")
scene.circle("dot", r=8, at=plot.at(t, t * t))
```

- `plot.at(x, y)` — one data point, as a pixel pair.
- `plot.line(f, steps=200)` — a list of pixel pairs, for `curve` or `polygon`.
- `plot.draw(scene, name="plot")` — the frame, ticks and labels, as shapes with
  ordinary names the author can restyle or leave out.
- `plot.looks(ink=, label=, layer=)` — a separate call rather than more
  arguments on `draw`, because no call in this library takes more than five
  things and the shapes made the same choice.

### It is a helper in the Authoring Surface, and nothing else

No new `Geometry`, no eleventh `KIND`, no payload field, no Rust. That matters
more than it sounds: ADR 0015 left `KINDS` at ten with **no free field pair**,
so the next kind costs a payload rethink. An axes needs none of it, because an
axes is arithmetic and the shapes it produces are the ones that already exist.

`cm.ngon` and `cm.star` are the precedent and already state the principle:

> Returns points rather than drawing, so it composes: you can shift them, hand
> them to `polygon`, or measure them yourself.

### Handing back pixels is what makes the rest free

Because `plot.at()` is pixels, everything downstream is ordinary Codimate and
none of it has to be re-implemented inside a chart object:

- A curve **tweens**, because it is a `curve` with a fixed point count.
- `scene.focus("f")` **zooms to it**, because focus already frames named shapes
  (ADR 0009).
- A dot **travels along it** if the trace says where it is at each moment, the
  way every other moving thing in the library travels.
- A motion `Rule` can be aimed at it by name.

A chart object that drew its own plot would have to grow an equivalent of each
of those, and would get them subtly wrong.

### Ticks are named by step index, not by value

This is the one decision that is not obvious and is forced by reconciliation.

Tick labels are shapes, so they have names, so they enter and leave. If the
domain changes between two Scenes — which is how a zoom is authored, by
emitting a different range — the tick set changes with it. Name a tick after
its value and a float that drifts by one ulp is a different shape: the old one
fades out, an identical-looking one fades in, and a still picture flickers.

So a tick is named `("tick", "x", n)` where `n` is the integer multiple of the
current step. While the step holds, a tick that survives a pan keeps its
identity and **slides**, which is right. When the step changes — a zoom far
enough to go from twos to tens — every tick is renamed at once and the whole
set cross-fades, which is also right, because that is what happened.

Steps are chosen from the 1–2–5 sequence, so they are stable under small
changes of range rather than jittering with every frame.

### What it is not

Deliberately absent, and the list is the point of the ADR as much as the
feature is:

- no log, date or categorical scales
- no legends, titles, dual axes, error bars, gridline styling beyond on/off
- no `plot_surface`, no 3D

The escape hatch for every one of them is the same: `plot.at()` gives pixels,
and the author draws whatever they like with the primitives. A legend is text
and swatches. A second axis is a second `cm.axes`.

## Consequences

- **Four examples could lose their private mapping code**, and `bernoulli_lift`
  could gain a real labelled axis it does not currently have.
- **The library ships its first chart-shaped thing**, and chart APIs grow. The
  guard is that `axes` returns a map rather than a picture, so every request of
  the form "can it also draw X" has a truthful answer that is not a new
  parameter: yes, with `plot.at()` and the primitive for X.
- **Zooming an axis is authored as a range change**, which is a pleasing
  consequence rather than a designed feature — and it is only pleasing because
  of the tick-naming rule above.
- **3D is not answered here.** A projection helper is possible and both
  `helical_solar_system/space.py` and `rubiks_cube/geometry.py` already
  contain one, but "3D axes" implies depth sorting the Engine does not have —
  painter's order is one integer per shape (`.on(layer=)`), which is only
  correct when the shapes admit a separating plane. The Rubik's cube needed
  four attempts and a cube-specific argument to get that right. Anything that
  interpenetrates needs a real depth buffer in the rasterizer, which is a
  larger decision than this one.

## Alternatives rejected

**A Manim-style `Axes` object that owns its plot**, with `axes.plot(f)`
returning a mobject the author animates. Familiar to anyone arriving from
Manim, and shorter for the first graph. Rejected because it puts the names
inside the object, and a name here is not a convenience — it is the identity
the whole reconciler runs on. The author would have a curve they could not
focus, could not aim a rule at, and could not tween against anything else.

**An `axes` Geometry in the Engine.** Ticks and labels computed in Rust, one
Shape in the payload, the way `formula` carries LaTeX and expands at diff time.
Genuinely tempting, because it would make an axis one payload entry instead of
forty. Rejected on two counts: `KINDS` is at ten with no free field pair, so it
is not free; and unlike a formula, nothing about an axis needs the Engine —
there is no glyph shaping, no file to read, no cache. It would be arithmetic
moved across the FFI for no gain.

**Return a `Group` whose origin is the plot's origin**, so children could be
placed in data coordinates. Reuses machinery that exists. Rejected because a
Group *translates* and does not scale, and adding a scale to it would scale
stroke widths, text and corner radii along with the coordinates — which is
exactly the bug a plotting library must not have.

**Do nothing, and let each example keep its own mapping.** What happens today,
and it has cost nothing so far, because no example wanted a labelled axis.
Rejected because the moment one does, the labelling — not the mapping — is the
work, and it is the same work every time.

# ADR 0015 — Arcs, which tween their angles

**Status:** Accepted — 2026-09-15

## Context

ADR 0013 left a standing demand: `KINDS` had reached nine, past the "8–10
kinds" an architecture review predicted the flat payload union would strain
at, and **the next kind would need a harder argument than that one got, or the
payload would need rethinking first.**

This is that kind. It has to answer both halves.

An arc was on the deferred list since ADR 0010, and the reason it stayed there
was that it has a workaround: a `curve` through points on a circle. `pendulum`
draws its angle sector exactly that way, sixteen straight segments, and the
worst error is 0.03 pixels — invisible. So the case for a kind was never about
the picture.

## Decision

**Add `arc`, whose `w`/`h` are a bounding box, `x2`/`y2` are start and end
angles in degrees, and `r` above zero closes it back to the centre.**

    scene.arc("angle", r=90, sweep=(0, 50)).fill("none", edge="cyan", edge_w=3)
    scene.arc("slice", r=120, sweep=(0, 120)).fill("orange").round(1)

Angles run clockwise from twelve o'clock, the zero `cm.ngon` already uses.
`r=(rx, ry)` gives an ellipse, which costs nothing because the box was always
two numbers.

### It adds no payload field

`x2` and `y2` are used by `line` and nothing else. `w`, `h` and `r` are free
for a kind that does not read them for something else. So the tenth kind needs
none of the six meanings already in flight and does not widen the struct by a
byte. That is the first half of the answer ADR 0013 asked for.

### It earns the kind by tweening, which a curve cannot

The second half. A `curve` through points on a circle draws an arc perfectly
well, and cannot *sweep*. Changing the sweep changes the number of sample
points, and ADR 0010's rule is that two polygons or curves with different
point counts do not interpolate — the later shape stands for the whole beat.
So a dial filling, a pie growing, an angle mark opening as a value changes all
snap from one state to the next.

An arc tweens its angles instead, so every instant in between is a real arc.
That is a capability, not a tidier spelling of one, and it is the only reason
this is a kind rather than a helper that returns points.

### Fixed spans, and angles rather than control points

Two arcs only tween if their paths have the same structure, so an arc is
always eight cubic spans however far it sweeps — not "one per ninety degrees".
At a full turn each span is 45 degrees, where the standard cubic fit is good to
a few thousandths of a radius.

**The tween interpolates the angles and rebuilds the arc, rather than
interpolating the path's control points.** Those are not the same thing, and
the difference is invisible until it is glaring: halfway between a collapsed
sector and a 300-degree one, control-point interpolation gives a self-crossing
shape that fills as a sliver beside a half-disc. It looks plausible on an open
arc and obviously broken on a pie. The first implementation did it that way and
rendering one is what found it.

## Consequences

- **`KINDS` reaches ten**, the top of the predicted range. The payload did not
  grow, but the *next* kind will find `x2`/`y2` taken as well, and there is no
  free pair left. The rethink ADR 0013 deferred is now the price of kind
  eleven, not a choice.
- **`pendulum`'s sixteen-segment sector could become one call**, and is left
  alone: it is correct, its comment explains why the corner count is fixed, and
  changing a working example to use a newer primitive is churn rather than
  improvement.
- **A pie chart is now four lines**, which it was not before.
- **An ellipse comes free** and was never asked for, which is the sort of thing
  worth noticing rather than celebrating.

## Alternatives rejected

**`cm.arc_points()` returning points for a `curve`.** No new kind, no payload
question, and the picture is identical at rest. Rejected because it cannot
sweep, which is the only reason to build this at all.

**One span per ninety degrees.** More accurate for small arcs and the obvious
implementation. Rejected because the span count would then depend on the sweep,
so two arcs of different sweeps would have different path structures and would
refuse to tween — losing the capability the kind exists for.

**Separate `arc` and `sector` kinds.** Clearer names, and no flag hiding in
`r`. Rejected as two kinds where one flag does, with `KINDS` already at the
limit — and they differ by two line segments.

**Keep deferring it.** What ADR 0010 did, twice, correctly: nothing needed it.
Accepted now because something did.

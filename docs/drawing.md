# What You Have to Work With

Sooner or later you'll want to animate something particular. A car, a queue, a
molecule. And the first question is always the same: what can I actually draw?

The answer is shorter than you might expect, and it's better to hear it now
than to discover it halfway through building something. Codimate gives you four
shapes. There is no arbitrary Bezier path and no image.

That sounds limiting, and for about ten minutes it is. Then you notice that a
thick line is a rectangle at any angle, that two circles make a ring, and that
a wing can be filled one column at a time. This chapter is the complete
inventory, what a handful of shapes turn out to be enough for, and a car built out of
them.

## The Four Shapes

Every one of these exists on a `Scene` and on any `Group`.

| | you give it | it draws |
|---|---|---|
| `scene.rect(name, h=, w=)` | a width and height | a rectangle — `.round(r)` for the corners |
| `scene.circle(name, r=)` | a radius | a circle |
| `scene.text(name, content, size=)` | some text | centred text, no baselines |
| `scene.line(name, start=, end=, w=)` | two points and a **thickness** | a stroked line |
| `scene.formula(name, latex, size=)` | LaTeX maths | typeset glyph outlines |
| `scene.polygon(name, points)` | corners | a filled shape with straight edges |
| `scene.curve(name, points, w=)` | samples | a smooth line through all of them |
| `scene.svg(name, file, size=)` | a file | vector art, as geometry you can animate |
| `scene.arrow(name, start=, end=)` | two points | a shaft and a head, as one shape |
| `scene.group(name, slot)` | a place | not a shape — somewhere to put several |

Each one hands back a **handle**, and the rest of what a shape looks like is
said on that: `.fill(color, edge=, edge_w=)`, `.round(r)`, `.turn(deg)`,
`.grow(scale)`, `.on(layer=, opacity=)`, `.write(reveal=, pen=)`.

All of them also take `color`, `layer`, `opacity`, and
[anchors](reference.md#anchors).

**That is the complete list.** There is no arbitrary
path, no image, no gradient, and no rotation.

---

## What Four Shapes Are Enough For

Every technique below comes from a working example in this repository, and none
of them needed a shape that doesn't exist.

**A ring — from one stroked circle.** `.fill("none", edge=…)` outlines a shape
without filling it, so a band is one circle at the mean radius, stroked at the
band's width:

```python
scene.circle("rim", r=188, at=(cx, cy)).fill("none", edge="gold", edge_w=32)
```

`dharma_wheel` predates that and still stacks four discs — a big one with a
smaller background-coloured one on top. It works, and it is four shapes and
three layers where one will do.

**An angled bar — from a thick line.** `line` strokes a path, so a short span
at a large width draws a **rectangle at any angle**. This is how you get
anything that is not axis-aligned:

```python
scene.line("spoke", start=(x1, y1), end=(x2, y2), w=24.0).fill("gold")
```

**A filled outline — one polygon.** `scene.polygon(name, points)` fills any
closed run of corners, and `cm.ngon`/`cm.star` produce the regular ones.
[`bernoulli_lift`](../python/examples/bernoulli_lift/) predates it and still
fills its aerofoil with 140 vertical lines, one per column, the way a
rasteriser would — correct, and no longer necessary.

**A curve — with `scene.curve`.** Hand it samples and it draws a smooth line
through all of them:

```python
scene.curve("plot", [(x, f(x)) for x in xs], w=2.5).fill("cyan")
```

The points are *on* the curve, not control points, so plotting a function is
the obvious thing rather than a fitting exercise. `bernoulli_lift` predates it
and still draws its streamlines as hundreds of straight pieces, which is
correct and no longer necessary.

**Rotation — from positions over time.** There is no `rotate`. Emit the angle
at each moment and let the Engine work out everything between, exactly as it
does for a bar sliding sideways. [`dharma_wheel`](../python/examples/dharma_wheel/)
turns a wheel this way; the car below turns its wheels.

---

## What You Genuinely Cannot Do

| you wanted | do this instead |
|---|---|
| a triangle or polygon | `scene.polygon(name, points)`, or `cm.ngon` for a regular one |
| a smooth curve | a run of short lines |
| an image or sprite | not supported — build it from shapes |
| `rotate=` on a group | emit the rotated positions; the Engine tweens them |
| a gradient | several shapes with stepped colours |

---

## Building a Car

Let's put all of that together. This is a complete program: paste it into
`car.py` at the repository root and run it.

```python
import math
import codimate as cm

ROAD, SPAN = 560.0, (120.0, 1160.0)
WHEEL_R, STEPS = 28.0, 40


@cm.trace()
def drive(car):
    for step in range(STEPS):
        car["x"] = SPAN[0] + (SPAN[1] - SPAN[0]) * step / (STEPS - 1)
        cm.emit("roll")


def view(frame):
    scene = cm.Scene()
    scene.line("road", start=(0, ROAD), end=(1280, ROAD), w=3.0).fill("#3a465e")

    x = frame.state["x"]
    car = scene.group("car", at=cm.at(x=x, bottom=ROAD))

    car.rect("body", w=230, h=62, at=cm.at(bottom=-16)).fill("#e05a4a")
    car.rect("roof", w=126, h=50, at=cm.at(x=-14, bottom=-78)).fill("#c94b3c")
    car.rect("window", w=104, h=34, at=cm.at(x=-14, bottom=-86)).fill("#2b3648")

    turn = x / WHEEL_R                       # rolling without slipping
    for side, wx in (("rear", -70), ("front", 70)):
        wheel = car.group(side, at=cm.at(x=wx, bottom=0))
        wheel.circle("tyre", r=WHEEL_R, at=cm.at(y=-WHEEL_R)) \
             .fill("#222832").on(layer=2)
        wheel.circle("hub", r=9, at=cm.at(y=-WHEEL_R)) \
             .fill("#8b93a3").on(layer=4)
        for spoke in range(3):
            a = turn + spoke * math.pi / 3
            dx, dy = WHEEL_R * 0.82 * math.cos(a), WHEEL_R * 0.82 * math.sin(a)
            wheel.line(("spoke", spoke),
                       start=(-dx, -WHEEL_R - dy), end=(dx, -WHEEL_R + dy),
                       w=4.0).fill("#8b93a3").on(layer=3)

    return scene


cm.explain(trace=drive({"x": SPAN[0]}), view=view,
           motion=[cm.Rule("*", position="linear")],
           timing=cm.Timing(default=0.09, opening=0.5, final_hold=1.2),
           ).render("results/car.mp4", fps=60, scale=1.5)

```

```bash
.venv/bin/python car.py
```

### Walking Through It

**The car is a group, so it moves as one thing.** `scene.group("car",
at=cm.at(x=x, bottom=ROAD))` is placed once; the body, roof, window and both
wheels are drawn on it and can never come apart.

**Inside a group, `0` is the group's own point.** The group sits on the road,
so `cm.at(bottom=0)` puts the wheels on the road and `cm.at(bottom=-16)` lifts
the body clear of it. Negative is upwards.

**Groups nest.** Each wheel is a group inside the car, so a spoke is placed
relative to its own wheel rather than to the road.

**The wheels turn without a rotate.** `turn = x / WHEEL_R` is rolling without
slipping — the angle a wheel of that radius has turned after travelling `x`.
The spokes are drawn at that angle each moment, and the Engine fills in the
rest.

**`linear`, not the default.** The car is mid-journey at every moment, so
easing would make it accelerate and stop once per tick.

### One Thing That Will Catch You

Within a single `layer`, shapes are drawn **in name order**. The first version
of this car had invisible spokes, because `hub` and `spoke` sort alphabetically
before `tyre`, so the tyre was painted over both of them. When one thing has to
cover another, say so:

```python
wheel.circle("tyre", r=WHEEL_R, at=cm.at(y=-WHEEL_R)).on(layer=2)
wheel.line(("spoke", i), ...).on(layer=3)
wheel.circle("hub", r=9, at=cm.at(y=-WHEEL_R)).on(layer=4)
```

---

## Where to Go Next

- [Tutorial](tutorial.md) — the four pieces, from an empty file
- [Reference](reference.md) — every call and parameter
- [The one thing to understand](../README.md#the-one-thing-to-understand) —
  names decide what moves
- [`python/examples/`](../python/examples/) — six worked ones, with notes

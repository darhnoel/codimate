# Reference

Chapter 4 of [the guide](../README.md#more). Every call and parameter, on one
page. For the walkthrough see [Writing Your First Animation](tutorial.md), for
what the shapes are enough for see [What You Have to Work With](drawing.md),
and for the ideas underneath see [How Codimate Thinks](concepts.md).

```python
import codimate as cm
```

---

## Putting it together

```python
cm.explain(*, trace, view, motion=None, timing=None) -> Explanation
```

The four pieces. `motion` is a list of `Rule`; omit it and everything travels
in a straight line.

```python
Explanation.render(output, *, fps=30, scale=1.0) -> str
```

Draws every frame and writes the video. **Resolution is a render argument, not
something the view knows about** — coordinates always mean what `cm.canvas()`
says, and `scale` only changes how many pixels each one becomes. Frames are
rasterized at the larger size, not upscaled.

```python
.render("out.mp4")                    # 720p30 from the default canvas
.render("out.mp4", fps=60, scale=1.5) # 1080p60
```

The folder is created if it does not exist. `ffmpeg` must be on your PATH.

---

## What happened

```python
cm.trace(fn, state, *, snapshot=None) -> Trace
```

Runs `fn(state, emit)` and records what happened. `snapshot(state)` returns the
data worth showing; it defaults to a deep copy of the state.

```python
emit(name, **data) -> None
```

The second argument your function is handed. Records that a moment worth
showing has happened — **call it after changing your data**, and Codimate
snapshots the result. `name` is what you later give a duration to; `**data`
arrives in the view as `frame.event.data`.

`emit` is an argument rather than something ambient, so your algorithm stays
ordinary Python: `bubble_sort(values, print)` runs it and prints the events.

**`sound=` is the one piece of data Codimate reads itself.** A path to an
audio file starts it as that event's beat begins:

```python
emit("said", sound="audio/said.mp3")
```

`render` lays every clip at its start and muxes them under the picture — the
video is not redrawn, and a missing file fails before the render starts rather
than after it. The start is read off the same timeline the picture uses, so the
two cannot drift; there is no clock to keep by hand. Clips may overlap, and each
keeps its recorded level. `exp.sounds()` lists `(start, path)`, and
`exp.mix_sound("x.wav")` writes the mix alone. Sound never enters a Scene
(ADR 0007).

The older form — `@cm.trace()` on a function calling a module-level `cm.emit`
— still works and warns. It is removed in 0.2.

```python
cm.items(values) -> list[Item]
cm.Item(value, id=None)
```

Gives each value an identity of its own, so two equal values are two different
things on screen. An Item **orders by `.value`** (your algorithm stays ordinary
Python) and is **equal by `.id`** (it survives the snapshot taken at every
event). Only needed for things that *move* — a grid keyed by position does not
need it.

---

## One moment

The view is called once per event with a `Frame`, and returns a `Scene`.

```python
frame.state           # your data at that moment
frame.event           # what just happened, or None for the opening moment
frame.is_(name)       # did this moment come from an event called `name`?
frame.items(key="items")   # the list an `items=[...]` event named; [] if none
```

`frame.event.data` holds whatever you passed to `emit()`.

---

## Drawing

`Scene` is the root; every method below exists on `Scene` and on any `Group`.

```python
scene.rect(name, *, h, w=None, at=None)          -> Handle
scene.circle(name, *, r, at=None)                -> Handle
scene.text(name, content, *, size=16.0, at=None) -> Handle
scene.formula(name, latex, *, size=16.0, at=None) -> Handle
scene.polygon(name, points, *, closed=True)      -> Handle
scene.curve(name, points, *, w=2.0, closed=False) -> Handle
scene.arc(name, *, r, sweep, at=None)            -> Handle
scene.svg(name, file, *, size=120.0, at=None)    -> Handle
scene.image(name, file, *, size=None, at=None)   -> Handle
scene.line(name, *, start, end, w=2.0)           -> Handle
scene.arrow(name, *, start, end, w=4.0, head=16.0) -> Handle
scene.group(name, slot=None, *, at=None, anchor=None, w=None) -> Group
scene.focus(*names, pad=40.0, least=240.0)      # what the camera looks at
scene.overlay() -> Group                        # what the camera does not move
```

Each shape takes the few things that decide **what it is** — a radius, a
height, some words. Everything else is said afterwards, on the Handle it hands
back. No call takes more than five arguments; a shape used to take eighteen.

`start` and `end` on a line or an arrow may each be a `Slot` or a plain `(x, y)`.

`image` draws a picture — a photo, a screenshot, a figure. PNG and JPEG,
read by content rather than extension. `size` is a fit box as for `svg`;
leaving it out draws the file at its own pixel size. It moves, scales, turns
and fades like anything else, but it is pixels, so `.fill()` cannot recolour it
and the pen cannot draw it. For a logo or a diagram prefer `svg`.

`arc` is a slice of a circle — an angle mark, a dial, a pie. `sweep` is
`(start, end)` in degrees, clockwise from twelve o'clock; `r` is a radius or
`(rx, ry)` for an ellipse. Open by default, so it draws as a curved line;
`.round(1)` closes it to the centre and makes a fillable slice. Two arcs always
tween, so a dial sweeps and a pie fills — which a `curve` through points on a
circle cannot do, because changing the sweep changes its point count and it
snaps instead.

`svg` imports vector art — a logo, an icon, a diagram exported from
somewhere else — as real geometry, so it tweens, `.turn()` and `.grow()`
transform it, and `.write(pen=2)` draws it on stroke by stroke. `size` is a box
it fits inside, one number or `(w, h)`, aspect always kept. It keeps the file's
own colours; `.fill(colour)` flattens it to a silhouette. Labels come across as real text, shaped by the same code that draws every
`scene.text` — so an imported diagram keeps its Khmer or its CJK. Layout that
cannot be read is refused rather than guessed at: a `tspan`, a `textPath`, or
a turned label, since the renderer cannot turn glyphs.

`curve` draws a smooth line **through** every point you give it — they are
samples, not control points, so you hand it a function you plotted or a path
something travelled and it does the fitting. An open curve is stroked the way
a line is (it encloses nothing); `closed=True` makes it a fillable shape.

`polygon` takes a sequence of `(x, y)` — a triangle, a wedge, a wing. `cm.ngon`
and `cm.star` produce the corners of the regular ones, so you rarely compute
them. An `arrow` is a single polygon rather than a line with a head stuck on,
so it carries one name and travels as one thing.

Two polygons only tween if they have the same number of corners; otherwise the
later shape stands for the whole beat. To morph one, keep the count fixed and
move the corners.

`focus` aims the camera by name, so you never write a camera coordinate: the
Engine knows where everything is, works out the framing, and the usual tween
animates the move. `least` is the smallest thing it will fill the frame with.
Anything drawn on an `overlay` stays where it is put — titles and captions
belong there, since a caption that zooms with the diagram ends up off the edge.

## The Handle

A shape call returns a `Handle`. Saying more about the shape is a chained call,
and each one returns the handle again:

```python
scene.rect("card", h=120, w=200).fill("#243046", edge="grey", edge_w=2).round(8)
scene.circle("bob", r=28, at=(x, y)).fill("orange").on(layer=4)
scene.polygon("tri", cm.ngon(3, r=60)).grow(1.8).turn(12)
```

- `.fill(color, edge=, edge_w=)` — the fill, and an outline. `color="none"`
  leaves it unfilled, so a shape can be an outline alone.
- `.turn(degrees, pivot="center")` — rotate. `pivot` is `center`, `top`,
  `bottom`, `left` or `right`.
- `.grow(scale)` — a number for both axes, or `(sx, sy)` to stretch.
- `.on(layer=, opacity=)` — draw order, and how solid it is.
- `.round(radius)` — a rectangle's corners, clamped to half its short side.
- `.write(reveal=, pen=)` — how much of a formula shows, and whether a pen
  traces it on.

All of it tweens like everything else, so a shape grows or turns between two
moments without you saying how.

**`.turn()` does not turn text.** Its position moves, but the glyphs stay
upright — rotating them is renderer work that has not been done.

`color` fills a shape and `edge`/`edge_w` outline it — both at once, so a
bordered box is one rectangle rather than two stacked ones. `color="none"`
leaves it unfilled, which is how you draw a ring. Lines are drawn rather than
filled, so their `w=` is the stroke and they take no edge.

`radius` rounds a rectangle's corners, clamped to half its short side — so a
big radius gives a pill, not a broken shape. It animates like anything else.

`formula` typesets LaTeX maths into glyph outlines, so it moves, fades and
recolours like any other shape. `reveal` is how much of it shows, left to
right — animate it from `0.0` to `1.0` and the equation writes itself on, with
several glyphs fading at once so it flows rather than ticking glyph by glyph.
Give `pen` a stroke width and it is *drawn* instead of faded: a pen traces each
glyph's outline and the solid letter fills in behind it as the pen moves on.
Both live on the handle: `.write(reveal=1.0, pen=2.2)`. Pass a raw string —
`r"\frac{a}{b}"` — or every backslash needs doubling. `size` means what it means for `text`. It needs the
`typst` binary on PATH, the way rendering needs `ffmpeg`.

## Placement

Where a shape goes is **one argument**, `at`. It takes a plain point, a `Slot`
from `cm.row`/`cm.column`, or `cm.at(...)` when you want an edge:

```python
scene.rect("bar", h=200, w=90, at=(640, 460))            # a point
scene.rect("bar", h=200, w=90, at=cm.at(x=640, bottom=560))  # stands on a line
scene.text("label", 3, at=cm.at(x=640, top=580))         # under something
bar = scene.group(item.id, slot)                         # a Slot
```

`cm.at(x=, y=, top=, bottom=)` — give at most one vertical of the three;
two on one axis raises `ValueError` rather than picking a winner. Anything you
leave out falls back to the group's own centre. Text is placed by its centre —
there are no baselines.

### Names

A name is the identity of the thing you are talking about, and **motion is
implied by it**: the Engine pairs shapes by name between one moment and the
next, and whatever changed becomes movement.

```python
scene.group(item.id, slot)          # named after the THING — it travels
scene.group(("cell", row, col))     # named after the PLACE — it stays put
```

Tuples flatten, so `("edge", src, dst)` becomes `edge/0/1/1/2` — build names
from other names rather than formatting strings.

### Groups

```python
bar = scene.group(item.id, slot)
bar.rect("box", h=value * 70, at=cm.at(bottom=0))
bar.text("label", value, at=cm.at(top=20))
```

**Everything on a group moves as one thing** — the Engine sees `3/box` and
`3/label`, so a motion rule cannot pull them apart. Groups nest.

**Inside a group, `0` is that group's own point.** Nothing is inherited from
the parent, at any depth. `w` defaults to the group's width and `x` to its
centre.

### Colours and layers

Names — `white` `black` `red` `orange` `blue` `green` `grey` `yellow` `cyan` —
or `#rrggbb`. Higher `layer` draws on top.

A colour is checked by the Engine, so a name it does not know is caught **at
render**, not when you write it: `ValueError: unknown color "puce" — use a name
or #rrggbb`.

Shapes that appear or disappear between moments **fade**; shapes that change
size, colour or position **tween**. You ask for none of it.

---

## Where things sit

```python
cm.canvas(w, h) -> None      # default 1280x720; call before building anything
cm.width() -> float
cm.measure(text, size=16.0) -> (w, h)   # how big that string will actually be
cm.height() -> float
```

```python
cm.row(items, gap=40.0, size=None, at=cm.at(y=None, bottom=None), within=None)
cm.column(items, gap=40.0, size=None, at=cm.at(x=None, y=None), within=None)
```

One `Slot` per item, centred. A **row** spreads and its slots anchor at
bottom-centre; a **column** stacks and its slots anchor at their centre. Sizes
come from the canvas unless given. Pass `within=slot` to divide a region
instead of the whole canvas.

Yields `(slot, item)` pairs — or bare slots if you passed a count.

```python
cm.Slot(x, y, w, h, anchor="center")

slot.left, slot.right, slot.top, slot.bottom   # the edges
slot.point(anchor=None)                        # the single point it anchors at
```

A Slot is a *place*, not a shape. Nothing draws it, and it carries no identity:
**Slots are where, names are what.**

---

### Graphs

```python
cm.axes(x=(lo, hi), y=(lo, hi), at=None, size=None, within=None) -> Axes

plot.at(x, y) -> (px, py)              # one data point, as pixels
plot.line(f, steps=200, over=None)     # points along y = f(x), as pixels
plot.ticks(axis="x", about=6)          # (n, value, label) per tick
plot.looks(ink=None, label=None, layer=None) -> Axes
plot.draw(scene, name="plot", about=6, grid=False) -> Axes
```

```python
plot = cm.axes(x=(-4, 4), y=(-2, 6), size=(820, 440)).draw(scene, grid=True)
scene.curve("f", plot.line(lambda t: t * t), w=4).fill("orange")
scene.circle("dot", r=9, at=plot.at(t, t * t)).fill("pink")
```

An Axes is a **coordinate map, not a drawing**. It hands back pixels, so what
you plot is a shape with a name of yours — which is what lets it tween, lets
`focus` frame it, and lets a motion Rule be aimed at it. `draw` adds the frame,
ticks and labels under names beginning with `name`, so two plots on one canvas
do not collide and you can leave any of them out and draw your own.

An open `curve` is *drawn* rather than filled, so its colour is `fill()` and its
thickness is `w=`. `fill("none", edge=...)` draws nothing.

**Zoom by changing the range**, not by moving the plot: emit a narrower `x=` and
the axis, its grid and every curve interpolate to it. Ticks are named after
their step multiple rather than their value, so one that survives a pan keeps
its identity and slides; a zoom big enough to change the step renames them all
and the set cross-fades.

Every sample `line` takes is returned, including any off the box. Dropping them
would be prettier and would stop the curve animating — two curves of different
lengths do not interpolate. Use `over=(lo, hi)` to sample less.

No log or date scales, no legends, no dual axes, no 3D; `plot.at()` and the
ordinary primitives are the answer to each. See
[ADR 0016](./adr/0016-axes-that-hand-back-pixels.md).

---

### Sizing a box around text

You have no canvas to interrogate, so `cm.measure` asks the engine what a
string will actually measure — with the real fonts and font fallback, which is
why a character count is wrong for anything but ASCII:

```python
w, h = cm.measure(label, size=30)
scene.rect("box", w=w + 24, h=h + 12, at=(x, y)).fill("#243046").round(6)
scene.text("label", label, size=30, at=(x, y))
```

The height is the line height, so it is the same for `"cat"` and `"Qgy"` and a
row of boxes lines up instead of jittering with whatever letters it holds.

## The science kit

```python
from codimate import science        # also reachable as cm.science
```

The marks and the space that explaining a physical thing keeps needing. Optional,
and none of it is in the Engine: it is arithmetic that ends in the ordinary shapes
above. Every piece follows `cm.axes` — it works out pixels and hands them back, and
where it draws it draws named shapes, so they tween, `focus` can frame them and a
motion Rule can be aimed at them. See
[ADR 0019](./adr/0019-a-science-kit-that-draws-flat.md).

### Marks

```python
science.tag(content, *, at, color="#e8eef7", size=20.0, formula=True) -> Tag

box.draw(scene, name, layer=3) -> Tag         # shapes (name, "plate"), (name, "text")
box.leader(scene, name, start, side, layer=2) # a line to the middle of one edge
box.clear_of(*others, gap=6.0) -> Tag         # moved the shortest way out of them
box.edge(side) -> (x, y)                      # top | bottom | left | right
box.left, box.right, box.top, box.bottom, box.x, box.y, box.w, box.h
```

A label on its own plate, sized from its real content plus padding. `at` is a
point, a Slot or `cm.at(...)`; with `top=` or `bottom=` the *plate's* edge lands
there, and the plate and its text share one centre. `formula=False` for words —
use it for Khmer. `leader` aims at the plate's actual edge, not a spot near it.

**Labels that follow bodies collide**, and no fixed offset prevents it, because
the bodies move. `clear_of` moves a tag the shortest way out of the others' boxes
— and does nothing to one that is already clear — so a name slides round another
instead of landing on it. Earlier tags win. It hurries only when two pass nearly
dead centre.

```python
box = science.tag(r"\rho = 1000\ \text{kg/m}^3", at=cm.at(x=900, top=80))
box.draw(scene, "density")
box.leader(scene, "density-line", start=(620, 300), side="left")
```

```python
science.Glow(color, steps=8, spread=1.8, strength=1.0, rim="#0b0f16")
glow.dot(scene, name, at, r, layer=0)          # a glowing disc
glow.ring(scene, name, at, r, layer=0)         # a glowing rim, nothing inside
```

A soft halo made of stacked, fainter, wider copies. It works on **one shape at a
time**. Over many thin lines — a grid, a mesh — the copies wash into mud or
bands, and more steps do not fix that.

```python
science.wave(start, end, *, cycles=2.5, amp=8.0, steps=20) -> [(x, y), ...]
```

Points along a wave between two places, for `curve`. Like `cm.ngon` it returns
points rather than drawing, so the count is fixed and one wave tweens into
another. Use a whole or half number of `cycles` and the ends land on the line.

```python
science.Bracket(color, tick=10.0, w=2.0, side=1, size=22.0, formula=False, layer=0)
bracket.draw(scene, name, start, end, label=None)
```

A measured span — a line, a tick at each end, a label beside it. `side` is `+1`
or `-1`, the side of the span the label sits on. `label` is one string or a list,
stacked. Shapes are named `(name, "span")`, `(name, "tick", 0|1)` and
`(name, "label", n)`.

```python
science.ForceScale(px_per_unit, cap=None, color="#58c4dd", w=7.0, head=19.0)
scale.length(value) -> (pixels, capped)
scale.arrow(scene, name, at, vector, label=None) -> tip
```

**One scale for every arrow in a film.** The arrows are usually the argument, and
one that quietly rescaled between scenes would make steel look no heavier than ice.
`vector` is `(fx, fy)` with **y up**, as physics writes it. An arrow longer than
`cap` is cut at the cap and marked with a break — rather than shrunk to fit or run
into the title — and `length` tells you it was capped. `label` is LaTeX, set above
and beside the tip. Returns the tip, or `None` if the arrow is too short to draw.

### Space

```python
science.Camera(azimuth=45.0, elevation=35.264, distance=1800.0, scale=0.7,
               target=(0, 0, 0), centre=None)

cam.project(point) -> ((x, y), depth)
cam.at(point) -> (x, y)
cam.facing(point, normal) -> bool         # does that surface face the camera?
cam.hidden_by(point, centre, r) -> bool   # is the point behind that sphere?
cam.moved(**changes) -> Camera            # same scale; perspective changes
cam.dolly(distance) -> Camera             # same lens; things shrink
```

Where you are looking from. **Z is up and space is right-handed**, as a physics
diagram writes it. The defaults are isometric: all three axes 120 degrees apart.
`scale` is pixels per unit at the target. `centre` is the pixel the target lands
on, the middle of the canvas by default.

**Distance is how much perspective there is.** A sphere far from the middle of the
picture stretches into an egg — measured at ratio 1.36 close in against 1.02 far
back — so if a round thing looks oval, stand further away rather than moving the
thing. A very large `distance` is an orthographic camera.

**To pull back, `dolly`.** It keeps the lens, so things shrink. `moved(distance=)`
keeps `scale`, so nothing shrinks and only the perspective changes.

```python
science.Sphere(centre, r, color="#58c4dd", style="solid", spin=0.0, tilt=0.0,
               detail=None)
science.Light(direction=(0.4, 0.3, 0.85), source=None, ambient=0.12,
              shadow="#000000")
science.Orbit(a, e=0.0, inclination=0.0, around=(0, 0, 0), color="#4a5568",
              hide=(), steps=240)

orbit.point(angle) -> (x, y, z)           # angle is mean anomaly, in degrees
orbit.path(steps=240) -> [(x, y, z), ...]
body.on_equator(angle, reach=1.0) -> (x, y, z)
body.moved(**changes), orbit.moved(**changes)
```

A `Sphere` is a value, not a drawing. `style` is `"solid"` (shaded), `"flat"` (one
colour: for something that is itself the light), `"star"` (lit from within: dimmer
toward the rim, mottled, the mottle turning with `spin`) or `"wire"` (its mesh only,
see-through, faded with depth). `spin` turns it about its own axis; `tilt` leans
that axis and stays put as the body orbits, which is what makes seasons. Colours
for shaded bodies must be hex.

A `Light` is a `direction` — rays all parallel — or a `source` point, so light
arrives *from* it and the lit side follows a body round its orbit. Faces are
shaded by mixing toward `shadow`, so what is drawn is opaque.

An `Orbit` is a real ellipse with the orbited body at a **focus**, not the middle.
`point` takes the *mean anomaly*, an angle that grows evenly with time, and solves
Kepler's equation — so the body goes quicker near the sun without you doing
anything but stepping the angle. `hide` names bodies whose silhouettes cut the
drawn path, so a ring never slices across a planet's face.

```python
science.World(camera, light=None, name="world", layer=0)
world.draw(scene, bodies, paths=None) -> next_free_layer

science.RingArrows(count=6, color="#ffd23f", span=16.0, w=4.0, head=15.0,
                   gap=1.2, layer=science.ABOVE)
arrows.draw(scene, world, name, body, angle) -> number_in_sight
```

```python
view = science.World(science.Camera(), light=science.Light(source=(0, 0, 0)))
top = view.draw(scene, {"sun": sun, "earth": earth}, {"year": orbit})
science.RingArrows().draw(scene, view, "spin", earth, angle)
```

`draw` takes every body **as one set**, because they must be sorted together —
sorted separately, a moon behind its planet would be painted over it. Each face
and each orbit segment gets its own layer, far to near, from `layer` upward, and
is named after the key *you* gave the body — `(name, key, "face", n)`,
`(name, key, "edge", i, j)` or `(name, key, "orbit", n)`. It returns the next free
layer; `ABOVE` is above any world.

**Emit one moment per frame.** Draw order is resolved once per *segment*, so a
layer that changes is a hard cut at the boundary between two Scenes. One Scene
per frame makes that cut one frame long and invisible; one every few frames
makes a turning ball re-sort in jumps and a ring tear across a planet. Use
`Timing(default=1 / fps)` and render at that `fps`.

A World draws a **dense** scene — a few thousand named shapes a frame. The
previewer's index records every shape of every moment — 470 MB beside a 1.5 MB
video for `examples/orbits`, and 1.3 GB for a denser one — so `render` skips it
for a scene like that and says so (`index="auto"`, below).

`RingArrows` ride a body's equator at the angle you give, so a turning solid ball
has something to follow. Those on the far side are skipped by a real ray against
the sphere, so they go *behind* it rather than floating over it.

What it does not do: bodies that interpenetrate, anything not a sphere, shadows
cast on other bodies, or a camera that goes behind what it looks at. Painter's
order is correct when shapes admit a separating plane, which separate spheres
do; the rest needs a depth buffer the Engine does not have. See
[ADR 0016](./adr/0016-axes-that-hand-back-pixels.md) and
[ADR 0019](./adr/0019-a-science-kit-that-draws-flat.md).

## Motion

```python
cm.Rule(pattern, position="straight", **options)
```

`pattern` matches a shape's full name with `*` and `?`; first match wins. A
shape inside a group is named `group/child`, so `"3/*"` targets one group.
A rule cannot make something move that did not move.

| `position` | |
|---|---|
| `straight` | a straight line, easing in and out — **the default** |
| `linear` | constant speed; for a thing mid-journey at every event, like something turning |
| `fall` | a parabola: sideways at a constant rate, downwards accelerating |
| `lift_carry_drop` | arcs up and over, then falls. Takes `clearance` |

```python
motion=[cm.Rule("*", position="lift_carry_drop", clearance=90)]
```

Use `linear` when something turns or orbits — easing would make it accelerate
and stop inside every segment.

```python
cm.ease(t) -> float
```

The Engine's own easing curve, in case you need to draw or reason about pacing.
It calls into the Engine, so it cannot drift from what your animation does.

---

## Looking at what you made

```python
exp = cm.explain(trace=..., view=..., timing=...)

exp.timeline()                       # [(start, length, event), ...] in seconds
exp.frame_at(12.5, "check.png")      # one moment, without rendering the video
exp.sheet([8, 22, 54], "look.png")   # several moments, tiled into one picture
exp.index()                          # every beat as data: state, shapes, boxes
exp.covered()                        # [(seconds, above, below), ...] — labels drawn on
```

`frame_at` uses the same scenes, timing and arithmetic as `render`, resolved at
one instant — checking a frame by rendering the whole video and seeking into it
costs a minute to look at one second. Pass the same `scale` you render with
when you are checking text.

`sheet` answers the question you actually have after a render — does the whole
thing hang together — instead of the one frame at a time `frame_at` answers.
Its `scale` shrinks in ffmpeg rather than in the Engine, which will not
rasterize below 1:1, so a whole film fits on a screen.

`timeline` answers the two questions you have when a video feels wrong: what is
on screen at 0:42, and how long each beat actually lasts.

`index` answers the third: *what put it there*. One entry per beat, in step
with `timeline`, carrying the State behind it.

```python
{"at": 3.5, "secs": 2.0, "event": "swap",
 "state": [{"value": 1, "id": 7}, ...],
 "shapes": {"set":  [{"name": "7", "kind": "rect", "layer": 0,
                      "box": [107.5, 78.5, 40, 20]}],
            "gone": []}}
```

**Shapes are a difference, not a list.** `set` is what arrived or moved, `gone`
the names that left; apply them in order from the first beat to know what is on
screen at any of them. A whole list per beat is the obvious format and it is
94% repetition — on a hundred-second film, seven megabytes of the same
rectangle against one and a bit.

A formula's box is measured by Typst, so it is `null` only on a machine
without `typst`. Each beat also carries `covered`: the `[above, below]` pairs
where something is drawn on a label (see *Covered labels* below).
`render(index="auto")` writes this beside the video as `<name>.index.json`
unless it would be huge — over `INDEX_LIMIT`, 50 MB — and then it skips it and
prints one line with the size it avoided. `index=True` writes it anyway,
`index=False` never does, and `exp.index_size()` estimates the bytes without
building it. Preview the script itself (not the mp4) and the previewer builds
the index in memory, so a skipped file costs nothing. `write_index(output)`
writes it without rendering. The file
wraps the beats as `{"version": 1, "canvas": [w, h], "beats": [...]}` — the
canvas is what turns a click on a scaled video back into these coordinates.
See [ADR 0018](adr/0018-a-previewer-that-reads-an-index.md).

## Covered labels

`render` checks every Trace Event for a label something is drawn on, and says
so when it has finished:

```text
1 label is covered:
  0:02.80  "arrow" is drawn over "label"

rendered anyway.
```

Two rules, and nothing to configure: nothing visible may be drawn **above** a
text or a formula, at any opacity, and no two of them may **overlap**. The plate
a label sits on is drawn below it and passes; an arrow drawn across it does
not. "Above" is the Engine's draw order — `layer`, then name — and text is on
layer 10 unless you move it.

It complains and never fails: a deliberate overlap, such as a mask revealing a
title letter by letter, is reported every time, by design. Each collision is
reported once, where it starts. `exp.covered()` returns the same list without
rendering. See [ADR 0017](adr/0017-a-check-that-says-a-label-is-covered.md).

## The Previewer

```bash
python -m codimate.preview python/examples/archimedes/main.py km
python -m codimate.preview results/archimedes-km.mp4
```

A window on a film, in the browser, laid out as Figma lays out a file:
chapters down the left, the picture in the middle, and what you have selected
on the right.

**Point at things.** Paused, the shape under the cursor is outlined with its
name. Click to select it: the panel shows its kind, layer and box, the event
that drew it, and the State behind it as a tree. Click the same spot again to
reach the shape beneath — a word, then the plate under it.

**Write notes.** Type what is wrong and press Enter. Notes are kept in this
browser, per film, so a refresh does not lose them; **Copy all** gives plain
text a person or an agent can act on:

```text
0:55.00  say/12/ជាង
  event    more.15
  chapter  សង្កត់ឲ្យលិច
  under    say_plate/…
  state    title=សង្កត់​ឲ្យ​លិច  said=16  …
> this word is too close to the next one
```

**Issues** lists every covered label (above); click one to jump to it with the
pair outlined.

**Chapters** are where you say a part of the film begins —
`emit("water", chapter="Water")` — and each gets a picture in the strip down
the left. A film has hundreds of events, one per highlighted word, so nothing
could work these out for you. `exp.chapters()` lists `(start, name)`. A film
without chapters has no strip.

**Given a script**, it runs the script up to its `render` call and keeps the
film in memory — no video is drawn. Arguments after the script go to it, as
they would under `python`. Each moment is drawn when you scrub to it (about
60ms), and saving the script, or any `.py` beside it, runs it again; a broken
edit shows its error and keeps the last good film. Clips given with
`emit(..., sound=)` play in step with it.

**Given an mp4**, it plays the finished film, with sound, against the index
written beside it.

| key | |
| --- | --- |
| space | play, pause |
| ← → | a tenth of a second; with shift, a second |
| `[` `]` | the previous, the next chapter |
| esc | deselect |
| `n` | write a note |

**It never writes.** No save, no edit — changing the film stays an edit to your
script, and notes never leave the browser until you copy them. The lookup it
uses is plain Python too, for asking without a window:

```python
from codimate import preview
index = preview.load("results/archimedes-km.mp4")   # or preview.index_of(explanation)
print(preview.note(preview.at(index, 55.0, (640, 640))))
```

`--no-open` prints the address instead of opening a browser; `--port` picks
one. Mid-transition, what is reported is the nearer of the transition's two
ends.

The page is Preact, with no build step: its libraries are vendored under
`python/codimate/viewer/vendor/` and loaded as plain modules.

## Timing

```python
cm.Timing(*, default=0.6, events=None, opening=0.8, final_hold=1.2)
```

Seconds, keyed by event **name**. `opening` holds the first picture before
anything moves; `final_hold` holds the last.

```python
cm.Timing(default=0.55, events={"swap": 0.9}, final_hold=2.0)
```

Duration lives here and nowhere else. If one appears in your algorithm or your
view, it is in the wrong place.

---

## When something looks wrong

| What you see | Why |
|---|---|
| Nothing moves, shapes just resize | Names follow position, not identity — use `cm.items()` |
| A shape pops in and out | Its name changes between moments; make it stable |
| Part of a thing moves without the rest | Draw it on one `scene.group()` |
| It surges and stalls once per event | Use `position="linear"` |
| Everything jumps at the start of a step | Consecutive moments differ by more than one event; `emit()` more often |
| `ValueError: unknown kind` | Codimate draws `rect`, `circle`, `text`, `line` |
| `ValueError: give only one of x=, left=, right=` | Two anchors on one axis |
| `ValueError: two shapes share the name` | One name used twice in one Scene |
| `ValueError: unknown path` | A `Rule` position that is not in the table above |
| `ValueError: unknown color` | Not a name above or `#rrggbb` — raised at render |
| `RuntimeError: emit() called outside a @trace function` | The old `cm.emit` outside `@cm.trace()` — take `emit` as an argument instead |

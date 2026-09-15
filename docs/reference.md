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
@cm.trace(*, snapshot=None)
```

Turns a state-mutating function into a `Trace`. `snapshot` receives the same
arguments as your function and returns the data worth showing; it defaults to a
deep copy of the first argument.

```python
cm.emit(name, **data) -> None
```

Records that a moment worth showing has happened. **Call it after changing your
data** — Codimate snapshots the result. `name` is what you later give a
duration to; `**data` arrives in the view as `frame.event.data`.

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
```

`frame_at` uses the same scenes, timing and arithmetic as `render`, resolved at
one instant — checking a frame by rendering the whole video and seeking into it
costs a minute to look at one second. Pass the same `scale` you render with
when you are checking text.

`timeline` answers the two questions you have when a video feels wrong: what is
on screen at 0:42, and how long each beat actually lasts.

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
| `RuntimeError: emit() called outside a @trace function` | Missing the `@cm.trace()` decorator |

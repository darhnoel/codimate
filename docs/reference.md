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
scene.rect(name, *, h, w=None, radius=0.0, color="white", layer=0, opacity=1.0, <anchors>)
scene.circle(name, *, r, color="white", layer=0, opacity=1.0, <anchors>)
scene.text(name, content, *, size=16.0, color="white", layer=10, opacity=1.0, <anchors>)
scene.line(name, *, start, end, w=2.0, color="white", layer=0, opacity=1.0)
scene.formula(name, latex, *, size=16.0, reveal=1.0, pen=0.0, color="white", layer=10, opacity=1.0, <anchors>)
scene.group(name, slot=None, *, anchor=None, w=None, <anchors>) -> Group
scene.focus(*names, pad=40.0, least=240.0)      # what the camera looks at
scene.overlay() -> Group                        # what the camera does not move
```

`start` and `end` on a line may each be a `Slot` or a plain `(x, y)`.

`focus` aims the camera by name, so you never write a camera coordinate: the
Engine knows where everything is, works out the framing, and the usual tween
animates the move. `least` is the smallest thing it will fill the frame with.
Anything drawn on an `overlay` stays where it is put — titles and captions
belong there, since a caption that zooms with the diagram ends up off the edge.

`radius` rounds a rectangle's corners, clamped to half its short side — so a
big radius gives a pill, not a broken shape. It animates like anything else.

`formula` typesets LaTeX maths into glyph outlines, so it moves, fades and
recolours like any other shape. `reveal` is how much of it shows, left to
right — animate it from `0.0` to `1.0` and the equation writes itself on, with
several glyphs fading at once so it flows rather than ticking glyph by glyph.
Give `pen` a stroke width and it is *drawn* instead of faded: a pen traces each
glyph's outline and the solid letter fills in behind it as the pen moves on. Pass a raw string — `r"\frac{a}{b}"` — or every
backslash needs doubling. `size` means what it means for `text`. It needs the
`typst` binary on PATH, the way rendering needs `ffmpeg`.

### Anchors

Give **one horizontal** and **one vertical**, and never convert an edge to a
centre yourself:

| horizontal | vertical |
|---|---|
| `x` centre · `left` · `right` | `y` centre · `top` · `bottom` |

```python
scene.rect("bar", x=640, bottom=560, w=90, h=200)   # stands on a line
scene.rect("box", left=100, top=100, w=200, h=80)   # from a corner
scene.text("label", 3, x=640, top=580)              # under something
```

Two anchors on one axis raises `ValueError` rather than picking a winner.
Text is placed by its centre — there are no baselines.

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
bar.rect("box", h=value * 70, bottom=0)
bar.text("label", value, top=20)
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
cm.row(items, *, gap=40.0, w=None, h=None, bottom=None, y=None, within=None)
cm.column(items, *, gap=40.0, w=None, h=None, x=None, y=None, within=None)
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
scene.rect("box", x=x, y=y, w=w + 24, h=h + 12, radius=6, color="#243046")
scene.text("label", label, x=x, y=y, size=30)
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

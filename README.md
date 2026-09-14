# Codimate

[![PyPI](https://img.shields.io/pypi/v/codimate)](https://pypi.org/project/codimate/)
[![Python](https://img.shields.io/pypi/pyversions/codimate)](https://pypi.org/project/codimate/)

Turn a running algorithm into an explainer video.

You write your algorithm as normal Python and say what each step looks like.
Codimate works out the motion, the timing, and every frame.

```python
import codimate as cm

@cm.trace()
def bubble_sort(values):
    for i in range(len(values)):
        for j in range(len(values) - 1 - i):
            cm.emit("compare", items=[values[j], values[j + 1]])
            if values[j] > values[j + 1]:
                values[j], values[j + 1] = values[j + 1], values[j]
                cm.emit("swap", items=[values[j], values[j + 1]])

def bars(frame):
    scene = cm.Scene()
    active = frame.items()

    for slot, item in cm.row(frame.state, gap=40):
        bar = scene.group(item.id, slot)
        bar.rect("bar", h=item.value * 70, at=cm.at(bottom=0)) \
           .fill("orange" if item in active else "blue")
        bar.text("label", item.value, size=32, at=cm.at(top=20))

    return scene

cm.explain(
    trace=bubble_sort(cm.items([3, 1, 4, 2])),
    view=bars,
    motion=[cm.Rule("*", position="lift_carry_drop", clearance=90)],
    timing=cm.Timing(default=0.55, events={"swap": 0.9}),
).render("results/bubble_sort.mp4")
```

That is the whole program. Run it, get `results/bubble_sort.mp4`.

## Install

```bash
pip install codimate
```

Codimate is a Rust engine with a Python front end, but the wheels are
prebuilt, so there is no Rust toolchain to install. One wheel covers every
Python from 3.9 up, on Linux, macOS and Windows.

### Working on the engine

Only if you are changing the Rust, rather than using Codimate:

```bash
git clone https://github.com/darhnoel/codimate && cd codimate
python3 -m venv .venv && .venv/bin/pip install maturin
.venv/bin/maturin develop --release
.venv/bin/python python/examples/bubble_sort/main.py
```

**Use `--release`.** Without it the Rust engine is unoptimized and renders
roughly 17x slower — a scene that draws in 6ms takes 100ms. Only leave it off
if you are debugging the engine itself.

### What else you need

`ffmpeg` does the video encoding. Codimate uses the one on your PATH if you
have it (`brew install ffmpeg`, `apt install ffmpeg`) and otherwise falls back
to the copy that comes with `imageio-ffmpeg`, which pip installs for you. Point
`CODIMATE_FFMPEG` at a binary to override both.

`typst` is needed **only** if you use `scene.formula` to typeset LaTeX maths
(`brew install typst`). Everything else renders without it.

## The four pieces

```text
algorithm   your normal code, with emit() where something happens
view        what one moment looks like
motion      how things travel between moments
timing      how long each moment lasts
```

**algorithm** — write it the way you normally would. Call `emit()` *after* you
change your data; Codimate snapshots the result for you.

**view** — a function from one moment to a picture. It receives `frame.state`
(your data at that moment) and `frame.event` (what just happened). It runs once
per event, not once per frame, so it can be as slow as you like.

**motion** — patterns matched against shape names. First match wins, and a
straight line is the default, so most explanations need no rules at all.

```python
motion=[cm.Rule("*", position="lift_carry_drop", clearance=90)]
```

A shape inside a group is named `group/child`, so `"3/*"` targets one group.
A rule cannot make something move that did not move.

| path | |
|---|---|
| `straight` | a straight line, easing in and out — the default |
| `linear` | a straight line at constant speed, for things mid-journey at every event |
| `lift_carry_drop` | arcs up and over, then falls; takes `clearance` |

Use `linear` when something turns or orbits: easing would make it accelerate
and stop inside every segment.

Every path eases in and out. `cm.ease(t)` calls into the Engine if you need
the curve itself — to draw it, or to pace something by hand — so you never
keep a second copy that can drift.

**timing** — seconds per event, by event name.

```python
cm.Timing(default=0.55, events={"swap": 0.9}, opening=0.8, final_hold=1.2)
```

## The one thing to understand

Codimate has no idea what a "swap" looks like. It only knows *this thing was
here, and now it is there.* **The name you give each shape is how you tell it
what moved.**

```python
scene.group(item.id, slot)    # the name follows the THING
scene.group(position, slot)   # the name follows the PLACE
```

Same algorithm, same data, completely different video:

| | what you see |
|---|---|
| name follows the thing | bar "3" is in slot 0, then slot 1 — it **slides across** |
| name follows the place | slot 0 stays put and changes height — bars **morph in place** |

Both are useful. Sorting wants the first. A grid, a heatmap or a matrix
multiply wants the second — cells don't travel, they light up:

```python
scene.group(("cell", row, col), slot)
```

Codimate cannot tell you which you meant, so this is the one decision worth
thinking about.

**`cm.items()` is how a plain value gets a name of its own.** Two 3s in a list
are two different bars, and only an identity can say so:

```python
values = cm.items([3, 1, 4, 2])   # each gets an .id and a .value
```

Items compare by value, so your algorithm stays ordinary Python
(`values[j] > values[j + 1]`), and they stay themselves across every moment of
the trace. You only need them for things that **move** — a grid keyed by
position doesn't.

## What you can draw

```python
scene.rect(name, h=, w=, at=)
scene.circle(name, r=, at=)
scene.text(name, content, size=, at=)
scene.polygon(name, points)                             # cm.ngon, cm.star
scene.line(name, start=slot_or_point, end=slot_or_point, w=)
scene.formula(name, r"\frac{a}{b}", size=, at=)         # LaTeX, needs `typst`
cm.measure(text, size) -> (w, h)                        # to size a box around text
scene.group(name, slot)      # a place to draw a thing made of several shapes
```

Each takes only what decides *what the shape is*. Everything else — colour,
outline, rotation, layer, opacity — is said afterwards on the handle it hands
back, so no call grows past five arguments:

```python
scene.circle("bob", r=28, at=(x, y)).fill("orange").on(layer=4)
scene.rect("card", h=120, w=200).fill("#243046", edge="grey").round(8)
scene.polygon("tri", cm.ngon(3, r=60)).grow(1.8).turn(12)
```

**Place things by whichever edge you actually mean.** `at=` takes a point, a
`Slot`, or `cm.at(...)` when an edge is what you mean:

```python
scene.rect("bar", h=200, w=90, at=(640, 460))               # a point
scene.rect("bar", h=200, w=90, at=cm.at(x=640, bottom=560)) # sits on a line
scene.text("label", 3, size=32, at=cm.at(x=640, top=580))   # under something
```

No baselines, no `y - h/2`. Give both `y` and `top` and you get an error, not
a silent winner.

Colors are names (`white`, `black`, `red`, `orange`, `blue`, `green`, `grey`,
`yellow`, `cyan`) or `#rrggbb`. Higher `layer` draws on top.

Shapes that appear or disappear between moments fade, and shapes that change
size, colour or position tween. You do not ask for any of that.

## Where things sit

You should not be inventing layout arithmetic. `cm.row()` and `cm.column()`
divide the canvas and hand back one **slot** per item; `scene.group()` puts a
thing in one:

```python
for slot, item in cm.row(frame.state, gap=40):
    bar = scene.group(item.id, slot)
    bar.rect("bar", h=item.value * 70, at=cm.at(bottom=0))
    bar.text("label", item.value, at=cm.at(top=20))
```

**Inside a group, `0` is the group's own point.** `cm.at(bottom=0)` stands the
bar on it; `cm.at(top=20)` puts the label 20 below it. `w` defaults to the
group's width and
`x` to its centre, so the only thing left to say is the one thing a bar and a
label disagree about.

**A name can be built from other names.** Nested keys flatten, so you never
concatenate tuples by hand — and a line takes Slots directly rather than making
you pull `.x` and `.y` out of them:

```python
scene.line(("edge", src, dst), start=at[src], end=at[dst])   # -> "edge/0/1/1/2"
```

**Everything on a group moves as one thing.** The bar and its label cannot come
apart, because the Engine sees them as `3/bar` and `3/label` — one name, two
shapes. Groups nest, so a thing made of a thing made of a thing still travels
as a unit.

A slot is a place, not a shape — nothing draws it. Hand one to `group()` and
you never take it apart; read `slot.x`, `slot.bottom`, `slot.left`, `slot.top`,
`slot.right`, `slot.w`, `slot.h` for the odd case that needs it.

`cm.column()` stacks instead of spreading — layers of a network, levels of a
tree — and its slots anchor at their centre rather than a baseline:

```python
for slot in cm.column(4, gap=44, at=cm.at(x=640)):
    scene.group(("neuron", 1, i), slot).circle("body", r=30)
```

Widths, spacing and the baseline come from the canvas unless you override them
(`size=`, `at=`, `within=`). The canvas is 1280x720 by default:

```python
cm.canvas(1920, 1080)     # everything below follows
cm.width(), cm.height()   # for the odd thing you place by hand
```

**Output resolution is separate from your coordinates.** `scale` only changes
how many pixels each coordinate becomes, so nothing in your view has to move:

```python
.render("out.mp4", fps=60, scale=1.5)   # 1080p60 from the default canvas
```

Frames are rasterized at the larger size rather than upscaled afterwards, so
1080p is genuinely drawn at 1080p.

## How it works

```text
Python (once per event)          Rust (once per frame)
──────────────────────           ─────────────────────
algorithm + emit()
view → a Scene per event
motion rules, durations
        │
        └─ hands over ONCE ───▶  pair shapes by item
                                 build tweens
                                 rasterize
                                 pipe to ffmpeg
                                        │
                                        ▼
                                    out.mp4
```

One handover, not one per frame. A frame is never played forward — it is
computed from scratch at time `t`, so any moment can be rendered in any order,
and twice the same way.

## Under the hood

The Rust crates are the Engine. You do not need to read them to use Codimate,
and the Rust API is not a second way to author explanations — see
[ADR 0008](docs/adr/0008-python-authoring-surface.md).

```text
crates/codimate-core/      pure animation model — f(t) → Scene
crates/codimate-animation/ duration and composition
crates/codimate-render/    tiny-skia CPU raster
crates/codimate-export/    raw RGBA → ffmpeg
crates/codimate-glyph/     text → glyph outlines
crates/codimate-py/        the bindings — the diff lives here
python/codimate/           the Python package
```

## More

All of this is also a site — **<https://darhnoel.github.io/codimate/>** — with
the API reference generated from the docstrings alongside it. Build it locally
with `python docs/build_site.py`.

**The guide, in order.** Four chapters; read them front to back the first time.

1. [Writing Your First Animation](docs/tutorial.md) — build one from an empty
   file, meeting all four pieces on the way.
2. [What You Have to Work With](docs/drawing.md) — the shapes, what they
   are enough for, and a car built out of them.
3. [How Codimate Thinks](docs/concepts.md) — why motion is derived rather than
   authored, and the one decision you have to make.
4. [Reference](docs/reference.md) — every call and parameter, on one page.

Then [`python/examples/`](python/examples/), eight worked examples with notes, and
[the decisions](docs/adr/) behind the design.

```bash
cargo test                                  # the Engine
.venv/bin/python python/tests/run.py             # the Authoring Surface
```

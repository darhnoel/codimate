# Codimate

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
        bar.rect("bar", h=item.value * 70, bottom=0,
                 color="orange" if item in active else "blue")
        bar.text("label", item.value, top=20, size=32)

    return scene

cm.explain(
    trace=bubble_sort(cm.items([3, 1, 4, 2])),
    view=bars,
    motion=[cm.Rule("*", position="lift_carry_drop", clearance=90)],
    timing=cm.Timing(default=0.55, events={"swap": 0.9}),
).render("sort.mp4")
```

That is the whole program. Run it, get `sort.mp4`.

## Install

Codimate is a Rust engine with a Python front end. Wheels are not published
yet, so for now build from source — you need a Rust toolchain and `ffmpeg`:

```bash
git clone https://github.com/darhnoel/codimate && cd codimate
python3 -m venv .venv && .venv/bin/pip install maturin
.venv/bin/maturin develop --release
.venv/bin/python python/examples/bubble_sort.py
```

**Use `--release`.** Without it the Rust engine is unoptimized and renders
roughly 17x slower — a scene that draws in 6ms takes 100ms. Only leave it off
if you are debugging the engine itself.

`ffmpeg` must be on your PATH (`brew install ffmpeg`, `apt install ffmpeg`).

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

**motion** — patterns matched against item names. First match wins.

```python
motion=[
    cm.Rule("bar_*", "lift_carry_drop", clearance=90),   # arcs over
    cm.Rule("label_*", "straight"),                      # slides
]
```

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
scene.rect(name,   h=, w=, color=, layer=, opacity=)
scene.circle(name, r=, color=, layer=, opacity=)
scene.text(name, content, size=, color=, layer=, opacity=)
scene.group(name, slot)      # a place to draw a thing made of several shapes
```

**Place things by whichever edge you actually mean.** Give one horizontal
anchor (`x`, `left`, `right`) and one vertical anchor (`y`, `top`, `bottom`):

```python
scene.rect("bar", x=640, bottom=560, w=90, h=200)   # sits on a line
scene.rect("box", left=100, top=100, w=200, h=80)   # from a corner
scene.text("label", 3, x=640, top=580, size=32)     # just under something
```

No baselines, no `y - h/2`. Give two anchors on the same axis and you get an
error, not a silent winner.

Colors are names (`white`, `black`, `red`, `orange`, `blue`, `green`, `grey`,
`yellow`, `cyan`) or `#rrggbb`. Higher `layer` draws on top.

Shapes that appear or disappear between moments fade, and shapes that change
size, colour or position tween. You do not ask for any of that.

## Where things sit

You should not be inventing layout arithmetic. `cm.row()` divides the canvas
and hands back one **slot** per item; `scene.group()` puts a thing in one:

```python
for slot, item in cm.row(frame.state, gap=40):
    bar = scene.group(item.id, slot)
    bar.rect("bar", h=item.value * 70, bottom=0)
    bar.text("label", item.value, top=20)
```

**Inside a group, `0` is the group's own point.** `bottom=0` stands the bar on
it; `top=20` puts the label 20 below it. `w` defaults to the group's width and
`x` to its centre, so the only thing left to say is the one thing a bar and a
label disagree about.

**Everything on a group moves as one thing.** The bar and its label cannot come
apart, because the Engine sees them as `3/bar` and `3/label` — one name, two
shapes. Groups nest, so a thing made of a thing made of a thing still travels
as a unit.

A slot is a place, not a shape — nothing draws it. Hand one to `group()` and
you never take it apart; read `slot.x`, `slot.bottom`, `slot.left`, `slot.top`,
`slot.right`, `slot.w`, `slot.h` for the odd case that needs it.

Widths, spacing and the baseline come from the canvas unless you override them
(`w=`, `h=`, `bottom=`). The canvas is 1280x720 by default:

```python
cm.canvas(1920, 1080)     # everything below follows
cm.width(), cm.height()   # for the odd thing you place by hand
```

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

- [Daily Workflow](docs/daily-workflow.md) — clone to first custom video
- [Authoring Model](docs/authoring-model.md) — why it is shaped this way
- [Domain Context](CONTEXT.md) — the vocabulary
- [Decisions](docs/adr/) — architecture decision records

```bash
cargo test                                  # the Engine
.venv/bin/python python/tests/test_codimate.py   # the Authoring Surface
```

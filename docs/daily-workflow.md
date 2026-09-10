# Codimate Daily Workflow

The canonical onboarding path for an Explanation Author.

Goal: clone to your own custom video in **less than 30 minutes**.

## Stage 1 — First win (5 minutes)

Build once and run the example. Nothing to edit yet.

```bash
python3 -m venv .venv && .venv/bin/pip install maturin
.venv/bin/maturin develop --release
.venv/bin/python python/examples/bubble_sort.py
```

The `--release` matters: without it the engine runs about 17x slower. You only
drop it when debugging the engine itself.

Open `sort.mp4`. Bars slide when they swap, and the pair being compared turns
orange.

This proves your toolchain, the Rust build, `ffmpeg`, and export all work. If
it fails here, fix it here — everything later assumes this worked.

**`ffmpeg` not found?** `brew install ffmpeg` or `apt install ffmpeg`.
**Rust missing?** <https://rustup.rs>.

You only run `maturin develop` again if you change Rust. Editing Python needs
no rebuild.

## Stage 2 — Change things (10 minutes)

Open [`python/examples/bubble_sort.py`](../examples/bubble_sort.py). Make one
change at a time and re-run. Each of these teaches one of the four pieces.

**Change the data** — the video regenerates from the algorithm:

```python
trace=bubble_sort(cm.items([5, 2, 8, 1, 9]))
```

**Change the timing** — make swaps linger:

```python
timing=cm.Timing(default=0.4, events={"swap": 1.5})
```

**Change the motion** — make bars slide through each other instead of arcing:

```python
motion=[cm.Rule("*", position="straight")]
```

**Change the view** — colour by size instead of by activity:

```python
color = "red" if item.value > 2 else "blue"
```

**Then break it on purpose.** In `bars()`, name the bar after its position
instead of its value:

```python
for position, (slot, item) in enumerate(cm.row(frame.state, gap=40)):
    bar = scene.group(position, slot)      # was scene.group(item.id, slot)
```

Re-run. Nothing slides any more — the bars change height in place. No error,
just a different video. That is the single most important thing to understand
about Codimate, and it is worth seeing once deliberately. See
[the README](../README.md#the-one-thing-to-understand).

Change it back.

## Stage 3 — Your own explanation (15 minutes)

Copy `bubble_sort.py` and replace the four pieces. Do them in this order.

### 1. The algorithm

Write it normally. Add `emit()` where something worth showing happens, **after**
you change your data:

```python
@cm.trace()
def insertion_sort(values):
    for i in range(1, len(values)):
        while i > 0 and values[i - 1] > values[i]:
            values[i - 1], values[i] = values[i], values[i - 1]
            cm.emit("shift", items=[values[i - 1]])
            i -= 1
```

Call it with `cm.items([...])` so each value is a thing of its own that can
move. Comparisons still read normally — `values[i - 1] > values[i]`.

Anything you pass to `emit()` reaches your view as `frame.event.data`.
`frame.items()` gives you back what an `items=[...]` event named, and
`frame.is_("shift")` tells you which event this moment came from.

Name events for what they *mean* — `shift`, `pivot`, `visit` — not `step_3`.
Those names are what you give durations to later.

### 2. The view

One moment, one picture. Two rules:

- **Let `cm.row()` and `scene.group()` do the layout.** A row divides the
  canvas into slots; a group puts one thing in one slot. You never take a slot
  apart, and everything on a group travels together.
- **The group's name carries identity.** Name things after what they *are*, not
  where they sit, unless you want them to stay put.

```python
def view(frame):
    scene = cm.Scene()
    active = frame.items()
    for slot, item in cm.row(frame.state, gap=40):
        bar = scene.group(item.id, slot)
        bar.rect("bar", h=item.value * 70, bottom=0,
                 color="orange" if item in active else "blue")
        bar.text("label", item.value, top=20)
    return scene
```

Inside a group, `0` is the group's own point — `bottom=0` stands the bar on it,
`top=20` puts the label below it. Everything on the group travels together.

Sizes and spacing come from the canvas, so this looks right at any resolution.
Override them when you care: `cm.row(items, gap=10, w=120, bottom=600)`.

`frame.event` is `None` for the opening moment, before anything has happened.
`frame.items()` and `frame.is_()` handle that for you.

### 3. Motion

Only if the default straight line is not enough:

```python
motion=[cm.Rule("*", position="lift_carry_drop", clearance=90)]
```

A pattern matches shape names with `*` and `?`; the first matching rule wins.
A shape inside a group is named `group/child`, so `"3/*"` targets one group.
Shapes that do not move are unaffected by any rule.

### 4. Timing

Give the events that matter more room:

```python
timing=cm.Timing(default=0.5, events={"shift": 0.9}, final_hold=1.5)
```

## The loop

```bash
.venv/bin/python my_explanation.py     # edit, run, watch, repeat
```

There is no build step and no preview window yet — you render and watch. Keep
your data small (4-6 items) while iterating, then grow it once it reads well.

## When something looks wrong

| What you see | Why |
|---|---|
| Nothing moves, shapes just resize | Names follow position, not identity — use `cm.items()` |
| A shape pops in and out | Its name changes between moments — make it stable |
| Everything jumps at the start of a step | Two consecutive moments differ more than one event's worth; emit more often |
| Part of a thing moves without the rest | Draw it on one `scene.group()` so it travels as a unit |
| `ValueError: unknown kind` | Codimate draws `rect`, `circle`, `text` |
| `ValueError: two shapes share the name` | One name used twice in one Scene |
| `RuntimeError: emit() called outside a @trace function` | The function needs the `@cm.trace()` decorator |

## Next

- [Authoring Model](./authoring-model.md) — why it is shaped this way
- [Domain Context](../CONTEXT.md) — the vocabulary this project uses

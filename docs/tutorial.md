# Your first animation, from an empty file

Nothing is copied here and no example is opened. Five steps, each one a
complete file you can paste and run.

We will animate a coin being flipped twenty times, tallying heads against
tails. Have [the install](../README.md#install) done first.

---

## 1. The shape of every explanation

Every Codimate program is four things. Start with the smallest one that runs:

```python
import codimate as cm


@cm.trace()
def flip(tally):
    cm.emit("start")


def view(frame):
    scene = cm.Scene()
    scene.text("hello", "nothing yet", x=cm.width() / 2, y=360, size=40)
    return scene


cm.explain(trace=flip({"heads": 0, "tails": 0}), view=view).render("results/coins.mp4")
```

```bash
.venv/bin/python coins.py
```

Two seconds of "nothing yet". Not much, but every piece is there:

- **the algorithm** — `flip`, ordinary Python, with `@cm.trace()` on it
- **the view** — one moment in, one picture out
- **`cm.explain(...)`** — puts them together and renders

`emit()` is how you tell Codimate that a moment worth showing has happened.
`view` is called once for each of them.

---

## 2. Make the algorithm do something

Write the logic as you normally would. Call `emit()` **after** you change your
data — Codimate takes a snapshot of the result for you.

```python
import codimate as cm
import random


@cm.trace()
def flip(tally):
    coin = random.Random(4)
    for toss in range(20):
        side = "heads" if coin.random() < 0.5 else "tails"
        tally[side] += 1
        cm.emit("flip", side=side)


def view(frame):
    scene = cm.Scene()
    scene.text("count", frame.state, x=cm.width() / 2, y=360, size=40)
    return scene


cm.explain(trace=flip({"heads": 0, "tails": 0}), view=view).render("results/coins.mp4")
```

Twenty moments now, and `frame.state` is the tally at each one. The numbers
change on screen because the view is asked again for every moment.

Notice you never said "animate". You said what happened.

---

## 3. Draw something real

Replace the text with two bars. `cm.row()` divides the canvas and hands you a
**slot** per item, so you never invent coordinates:

```python
import codimate as cm
import random

SIDES = ("heads", "tails")


@cm.trace()
def flip(tally):
    coin = random.Random(4)
    for toss in range(20):
        side = "heads" if coin.random() < 0.5 else "tails"
        tally[side] += 1
        cm.emit("flip", side=side)


def view(frame):
    scene = cm.Scene()
    for slot, side in cm.row(SIDES, gap=120, w=190):
        count = frame.state[side]
        bar = scene.group(side, slot)
        bar.rect("box", h=max(count * 22, 1), bottom=0, color="blue")
        bar.text("label", side, top=16, size=24)
    return scene


cm.explain(trace=flip({"heads": 0, "tails": 0}), view=view).render("results/coins.mp4")
```

Two bars that grow. Three things just happened worth knowing:

**A group is a thing made of several shapes.** The bar and its label are drawn
on one `scene.group(side, slot)`, so they can never come apart.

**Inside a group, `0` is the group's own point.** `bottom=0` stands the bar on
it; `top=16` puts the label just below it.

Sizes come from the canvas unless you say otherwise — `w=190` here, because
two bars filling most of the frame look like slabs.

**The name carries identity.** `scene.group(side, slot)` names each bar after
the side it counts. Codimate pairs shapes by name between one moment and the
next, and whatever changed becomes movement — here, a height.

---

## 4. Show which one just happened

The view gets `frame.event` as well as `frame.state` — what just happened, not
only the data afterwards. Anything you pass to `emit()` arrives in
`frame.event.data`:

```python
import codimate as cm
import random

SIDES = ("heads", "tails")


@cm.trace()
def flip(tally):
    coin = random.Random(4)
    for toss in range(20):
        side = "heads" if coin.random() < 0.5 else "tails"
        tally[side] += 1
        cm.emit("flip", side=side)
    cm.emit("done")


def view(frame):
    scene = cm.Scene()
    landed = frame.event.data.get("side") if frame.event else None

    scene.text("title", "20 coin flips", x=cm.width() / 2, y=90, size=38, color="grey")

    for slot, side in cm.row(SIDES, gap=120, w=190):
        count = frame.state[side]
        bar = scene.group(side, slot)
        bar.rect("box", h=max(count * 22, 1), bottom=0,
                 color="orange" if side == landed else "blue")
        bar.text("label", f"{side}  {count}", top=16, size=24)
    return scene


cm.explain(trace=flip({"heads": 0, "tails": 0}), view=view).render("results/coins.mp4")
```

The bar that just grew turns orange, and fades back as the next flip lands.
Codimate does the fading — you only said what colour it is at each moment.

`frame.event` is `None` for the opening moment, before anything has happened,
which is what the `if frame.event` guards.

---

## 5. Pace it

`Timing` is where duration lives, and nowhere else. Give the events that matter
more room, and hold the ending so the result can be read:

```python
import codimate as cm
import random

SIDES = ("heads", "tails")


@cm.trace()
def flip(tally):
    coin = random.Random(4)
    for toss in range(20):
        side = "heads" if coin.random() < 0.5 else "tails"
        tally[side] += 1
        cm.emit("flip", side=side)
    cm.emit("done")


def view(frame):
    scene = cm.Scene()
    landed = frame.event.data.get("side") if frame.event else None
    finished = frame.is_("done")

    scene.text("title", "20 coin flips", x=cm.width() / 2, y=90, size=38, color="grey")

    for slot, side in cm.row(SIDES, gap=120, w=190):
        count = frame.state[side]
        bar = scene.group(side, slot)
        bar.rect("box", h=max(count * 22, 1), bottom=0,
                 color="green" if finished else "orange" if side == landed else "blue")
        bar.text("label", f"{side}  {count}", top=16, size=24)
    return scene


cm.explain(
    trace=flip({"heads": 0, "tails": 0}),
    view=view,
    timing=cm.Timing(default=0.35, events={"done": 1.0}, opening=0.8, final_hold=2.0),
).render("results/coins.mp4", fps=60, scale=1.5)
```

`fps=60, scale=1.5` renders at 1080p60. **Resolution is a render argument, not
something the view knows about** — your coordinates still mean what
`cm.canvas()` says.

---

## What you did not have to do

You never wrote a keyframe, a duration on a shape, a tween, or a frame number.
You described **what happened** and **what a moment looks like**; everything
between the moments was worked out.

That is the whole idea, and it has one cost — the next page.

## Next

- [The one thing to understand](../README.md#the-one-thing-to-understand) —
  names decide what moves. Read this before your second animation.
- [Reference](reference.md) — every call, every parameter, one page.
- [Authoring Model](authoring-model.md) — why it is shaped this way.
- [`python/examples/`](../python/examples/) — six worked ones, with notes.

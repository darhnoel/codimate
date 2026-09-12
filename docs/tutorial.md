# Writing Your First Animation

Let's build something together. Over the next few pages you'll write a complete
Codimate program from an empty file: one that flips a coin twenty times and
shows the running tally rise as each flip lands.

It's a small thing to animate, and that's deliberate. What matters is that by
the end you'll have met all four pieces every Codimate program is made from,
and you'll have seen the idea the whole library rests on. You never describe
movement. You describe moments, and Codimate works out the movement between
them.

## Setting Up

If you haven't installed Codimate yet, follow [the install
steps](../README.md#install) first, then come back.

Create a file called `coins.py` in the repository root, alongside `README.md`.
Run every command in this chapter from that folder, so your video lands in
`results/` with the others.

## The Shape of a Codimate Program

Before we animate anything, let's get the smallest possible program running.
This gives us something to build on, and it proves your installation works
before we add anything that could obscure a problem.

Type this into `coins.py`:

```python
import codimate as cm


@cm.trace()
def flip(tally):
    cm.emit("start")


def view(frame):
    scene = cm.Scene()
    scene.text("hello", "nothing yet", x=cm.width() / 2, y=cm.height() / 2, size=40)
    return scene


cm.explain(trace=flip({"heads": 0, "tails": 0}), view=view).render("results/coins.mp4")
```

Then run it:

```bash
.venv/bin/python coins.py
```

The program prints nothing and writes `results/coins.mp4`. Open it and you'll
see two seconds of the words "nothing yet" in the middle of a black frame. If
that plays, everything is working.

There isn't much on screen, but every part of a Codimate program is already
here. `flip` is the **algorithm**: ordinary Python, marked with `@cm.trace()`
so Codimate can watch it run. `view` is the **view**: it receives one moment
and returns one picture. And `cm.explain` gathers them together and renders.

The call to `cm.emit` is how you tell Codimate that something worth showing has
happened. Your view is asked for a picture once for each one. Right now there
is a single moment, so you get a single unchanging picture.

## Giving the Algorithm Something to Do

Our program has the right shape but nothing happens in it. Let's write the
actual logic.

The important thing here is that you write it the way you always would. There
is no Codimate-shaped way to flip a coin. You add one line, calling `cm.emit`
**after** you change your data, because Codimate takes a snapshot of the
result:

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
    scene.text("count", frame.state, x=cm.width() / 2, y=cm.height() / 2, size=40)
    return scene


cm.explain(trace=flip({"heads": 0, "tails": 0}), view=view).render("results/coins.mp4")
```

Run it again, and this time the numbers count upward on screen.

Two things are worth noticing. The first is that `frame.state` holds your data
as it was at that moment, not as it ended up. Codimate kept a copy each time
you called `emit`, so the view can ask for any of them.

The second is `random.Random(4)`. That's a fixed seed, so you get the same
twenty flips every time you render. Without it, every run would produce a
different video, and you'd have no way to tell whether a change you made was
responsible for a difference you noticed.

## Drawing Something Real

Numbers on a screen aren't much of an animation. Let's replace them with two
bars, one for heads and one for tails, that grow as the flips land.

This is where you'd expect to start working out coordinates. You don't have to.
`cm.row` divides the canvas for you and hands back a **slot** for each item:

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

Run it and you'll see two bars rising, each labelled underneath.

Let's walk through what changed. `cm.row(SIDES, gap=120, w=190)` gives us one
slot per side, spaced evenly and centred on the canvas. Sizes come from the
canvas unless you say otherwise, and here we do say otherwise, because two bars
filling most of the frame look like slabs rather than bars.

`scene.group(side, slot)` puts a **group** in that slot. A group is somewhere
to draw a thing made of several shapes, and everything drawn on it moves
together. Our bar is a rectangle and a label, and because they share a group
they can never drift apart.

Inside a group, `0` means the group's own point, which is why the rectangle
says `bottom=0` to stand on it and the label says `top=16` to sit just below.
You never convert an edge into a centre yourself.

The call to `max(count * 22, 1)` keeps each bar at least one pixel tall. A
rectangle with no height has nothing to draw, so without it the first flip
would make a bar flicker into existence rather than grow.

Finally, and most importantly: `scene.group(side, slot)` names each bar after
the side it counts. Codimate pairs shapes by name between one moment and the
next, and whatever changed becomes movement. Here the height changed, so the
bars grow. We'll come back to this idea, because it's the one that decides
whether anything moves at all.

## Reacting to What Just Happened

Our bars grow, but they don't tell you which flip caused which growth. The view
can know that, because it receives more than your data. It also receives the
event that produced the moment.

Anything you pass to `cm.emit` arrives as `frame.event.data`:

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

Run it. Now the bar that just grew turns orange, then fades back as the next
flip lands.

Notice that you didn't ask for a fade. You said the bar is orange in one moment
and blue in the next, and Codimate worked out the transition. This is the same
mechanism that grew the bars, applied to colour instead of height.

One detail worth remembering: `frame.event` is `None` for the opening moment,
before anything has happened. That's what the `if frame.event` is guarding
against.

## Controlling the Pace

Our animation is complete, but it reads too quickly, and it ends the instant
the last flip lands. Let's fix the timing.

In Codimate, duration lives in exactly one place. Your algorithm doesn't know
how long anything takes, and neither does your view. `Timing` holds all of it,
keyed by the event names you chose earlier:

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

Run it once more. The flips are slower now, both bars turn green when the
tally is complete, and the final picture is held long enough to read.

The two arguments to `render` are new. `fps=60, scale=1.5` produces a 1080p60
video. Resolution is a render-time decision rather than something your view
knows about, so your coordinates still mean what `cm.canvas()` says they mean.

## Experiments Worth Running

Before moving on, change one thing at a time in `coins.py` and re-render. Each
of these exercises a different one of the four pieces, and each takes a few
seconds to see.

**Change the data.** Make it a hundred flips instead of twenty. The video
regenerates from the algorithm; you don't touch the view.

```python
for toss in range(100):
```

**Change the timing.** Give the final moment room to breathe.

```python
timing=cm.Timing(default=0.15, events={"done": 2.5}, final_hold=3.0)
```

**Change the view.** Colour by which side is winning rather than by which just
landed.

```python
color="green" if count == max(frame.state.values()) else "blue"
```

**Then break it deliberately.** In the view, name each bar after its position
instead of after the side it counts:

```python
for i, (slot, side) in enumerate(cm.row(SIDES, gap=120, w=190)):
    bar = scene.group(i, slot)              # was scene.group(side, slot)
```

Re-render. Nothing looks different, because with two fixed bars a position and
a side identify the same thing. Now swap the order of `SIDES` as well. The bars
jump rather than sliding, because you renamed them: Codimate believes the
heads bar left and a different bar arrived in its place.

No error, just a different video. That is the most important thing to
understand about Codimate, and it's worth provoking once on purpose.

## What You Didn't Have to Write

Look back over what you wrote. There's no keyframe anywhere in it. No duration
attached to a shape. No tween, no interpolation, no frame number. You wrote
down what happened and what a single moment looks like, and everything between
the moments was worked out for you.

That's the whole idea, and it asks one thing of you in return. Because Codimate
matches shapes between moments by **name**, the names you choose are what
decide whether something moves or merely changes shape. Our bars were named
after the side they count, so they grew in place. Had they been named after
their position, something quite different would have happened.

That idea is worth understanding before you write a second animation, and it's
the first link below.

## Where to Go Next

- [The one thing to understand](../README.md#the-one-thing-to-understand):
  how names decide what moves.
- [What you have to work with](drawing.md): the shapes Codimate gives you,
  what they're enough for, and a car built out of them.
- [Reference](reference.md): every call and parameter, on one page.
- [How Codimate Thinks](concepts.md): why the library is shaped this way.
- [`python/examples/`](../python/examples/): six worked examples, each with notes.

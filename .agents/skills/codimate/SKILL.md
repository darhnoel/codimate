---
name: codimate
description: Create runnable Codimate examples in Python from user prompts, with minimal token usage. Use when the user asks to visualize a concept or algorithm, create an example under python/examples/<name>, or learn the Codimate authoring API quickly.
---

# Codimate Skill

Turn a prompt into a runnable Codimate explainer.

**Python is the authoring surface. Rust is the engine** (ADR 0008). Never write
a Rust example, never touch `crates/` for authoring work, and never wire
anything into `codimate-previewer` — it is internal machinery and does not
build.

## The model, in one paragraph

You write ordinary Python and mark it with `@cm.trace()`. Call `cm.emit()`
where something worth showing happens. A `view` function receives one moment
and returns one `Scene`. Codimate pairs shapes between consecutive Scenes **by
name** and turns whatever differs into movement. You never write a keyframe, a
duration on a shape, or an interpolation.

The names you choose decide what moves. A name that follows a *value* travels
with it; a name that follows a *position* stays put and changes contents. Both
are useful; choosing wrong gives a confusing video, not an error.

## Smallest complete example

```python
import codimate as cm

SIDES = ("heads", "tails")


@cm.trace()
def flip(tally):
    for _ in range(20):
        tally["heads" if coin() else "tails"] += 1
        cm.emit("flip")          # after changing the data — it snapshots the result


def view(frame):
    scene = cm.Scene()
    for slot, side in cm.row(SIDES, gap=120, w=190):
        bar = scene.group(side, slot)
        bar.rect("box", h=max(frame.state[side] * 22, 1), bottom=0, color="blue")
        bar.text("label", side, top=16, size=24)
    return scene


cm.explain(
    trace=flip({"heads": 0, "tails": 0}),
    view=view,
    timing=cm.Timing(default=0.35, opening=0.8, final_hold=2.0),
).render("results/coins.mp4", fps=60, scale=1.5)

print("wrote results/coins.mp4")
```

`print("wrote <path>")` on its own line is a contract: `test_examples` reads it
to find what was produced. Anything else goes on a separate line.

## Token budget: signature-first

Do not read whole files. In order:

1. `docs/reference.md` — every call and parameter on one page. Usually enough.
2. `python/codimate/scene.py` — read 20–40 lines around a method signature.
3. One neighbouring example, for a pattern you are copying.

Do not read `crates/` unless fixing the engine.

## The API you actually need

```python
# shapes — all take color=, layer=, opacity=, and anchors
scene.rect(name, w=, h=, radius=)        # radius rounds the corners
scene.circle(name, r=)
scene.text(name, content, size=)
scene.line(name, start=, end=, w=)       # start/end take a Slot or (x, y)
scene.formula(name, latex, size=, reveal=, pen=)   # real LaTeX; needs `typst`
scene.group(name, slot)                  # several shapes that move together

# placement — pick ONE per axis
at=(x, y)          # a point, or a Slot
x=, y=             # one axis by its centre
left= right= top= bottom=                # one axis by an edge

# where things sit
cm.row(items, gap=, w=), cm.column(...)  # slots, so you write no coordinates
cm.canvas(w, h), cm.width(), cm.height()
cm.measure(text, size)                   # how big text will really be
cm.measure_math(latex, size)             # same, for a formula

# camera
scene.focus(*names, pad=, least=)        # frame these, by name
scene.overlay()                          # a group the camera does not move

# motion and time
cm.Rule("*", position="linear")          # straight | linear | fall | lift_carry_drop
cm.Timing(default=, events={}, opening=, final_hold=)

# looking at what you made
exp = cm.explain(trace=..., view=..., timing=...)
exp.timeline()                           # [(start, length, event), ...]
exp.frame_at(12.5, "check.png", scale=1.5)   # one moment, no full render
```

Use `frame_at` to check a frame. Rendering the whole video and seeking into it
wastes a minute per look.

## Choosing motion

`straight` eases in and out — right when each event is a distinct step.

`linear` is right whenever the thing is mid-journey at every event: a turning
wheel, an orbit, a simulation sampled at a fixed step. Easing each segment
there makes it surge and stall once per event. Dharmachakra measured a 6.6x
speed swing with easing against 1.2x with `linear`.

`fall` is a parabola, for anything dropped. `lift_carry_drop` arcs up and over.

## Output shape

```text
python/examples/<snake_case_name>/
    main.py       the algorithm, the view, and the render call
    README.md     what this example teaches, and what to try changing
```

Split only when one file stops fitting in your head — and split by
*responsibility*, not by size:

```text
main.py       the algorithm and the render call
physics.py    the simulation; knows nothing about being drawn
view.py       the drawing; reads the simulation, never steers it
board.py      the measurements — where things sit
```

`galton_board` is the reference for a split example; `bubble_sort` for a small
one. Examples are discovered, not listed, so a new folder is covered by the
tests the day it is added.

## Rules

- Seed every random source. An unseeded example renders differently each run
  and you cannot tell a change from noise.
- Render at `fps=60, scale=1.5` for 1080p60.
- Give every shape a name that says what it *is*, not where it sits.
- Write the README as what the example teaches, not what the code does.
- Do not commit. Do not run `git add` or `git commit`.

## Validation

```bash
.venv/bin/python python/examples/<name>/main.py
cd python && ../.venv/bin/python tests/run.py --fast
```

The full suite also renders every example; `--fast` skips that while iterating.
Then report the runnable command.

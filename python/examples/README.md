# Examples

One folder per example, one `main.py` inside it, and a README saying what that
example teaches.

Small examples are a single `main.py`. Bigger ones split — see
[When to split](#when-to-split) below for the rule, which is not the one you
might expect.

Run any of them from the repository root:

```bash
.venv/bin/python python/examples/bubble_sort/main.py
```

Each writes 1080p60 into [`results/`](../../results/), which is not committed.
Resolution is a render argument (`scale=1.5`), not something the views know about.

## Start here

**[`bubble_sort/`](bubble_sort/)** — things that **move**.

Bars slide when they swap. Names follow the *thing*, so `cm.items()` gives each
value an identity of its own and the Engine can tell that bar 3 travelled
rather than that two bars changed height. Read this one first; everything else
assumes it.

## The other half

**[`neural_net/`](neural_net/)** — things that **stay put**.

Neurons never move, so their names follow the *place*:
`("neuron", layer, index)`. No `cm.items()` anywhere. The only travellers are
the pulses, and they travel because the trace says where a signal *is* at each
moment — the view never mentions movement.

Between them, these two cover the single decision Codimate cannot make for you.

## Motion you might think needs a new feature

**[`dharma_wheel/`](dharma_wheel/)** — a turning wheel, with no rotation in the
API.

The trace says where the spokes are every 15 degrees and the Engine fills in
the rest, exactly as it does for a sliding bar. Rotation is just position over
time. The README works through why 15 degrees and not 45.

## Both at once

**[`galton_board/`](galton_board/)** — where the bell curve comes from.

Pegs and bins named after their place, balls named after themselves, a dozen in
flight at any moment each on its own path. The falling is a real ballistic
simulation sampled at a fixed time step, which is what lets balls at different
depths move at different speeds.

## Under the hood

**[`explain_codimate/`](explain_codimate/)** — Codimate explaining its own
maths.

Three stored pictures and the formula that computes every frame between them.
The playhead sweeping the timeline is moved by exactly the formula it draws,
and the curve is `cm.ease` — the Engine's own easing, called into rather than
copied.

## When to split

**Split by what is on screen, not by the four pieces.**

The temptation is `state.py` / `algorithm.py` / `view.py` / `motion.py` /
`timing.py`, mirroring the Rust examples. Don't. Those four pieces are short —
in `bubble_sort` they are 8, 20, 1 and 1 lines — and they are already named
where they are used:

```python
cm.explain(trace=..., view=..., motion=..., timing=...)
```

That call shows how the pieces *connect*, which a directory listing cannot.
Splitting there gives you five files of a dozen lines each and five import
blocks, which is the ceremony the Python surface exists to remove
([ADR 0008](../../docs/adr/0008-python-authoring-surface.md)).

**The length is in the drawing.** So that is where the seams are. When a panel
of the picture stops fitting on a screen, give it a file:

```text
explain_codimate/
    main.py        the four pieces, together, plus the view that composes panels
    story.py       what is being explained: the data and its state
    theme.py       where the panels sit, and what colour things are
    timeline.py    one panel, exporting draw(scene, ...)
    curve.py       one panel
    bars.py        one panel
```

Each panel exports a `draw(scene, ...)` and imports only `theme` and `story` —
never another panel — so there is no import order to remember. Python puts the
script's own directory on `sys.path`, so plain `import timeline` works when you
run `main.py` directly.

**Rough threshold:** one `main.py` until it passes ~150 lines or grows a second
distinct panel. `bubble_sort` (61 lines) and `neural_net` (125) are single
files and should stay that way; `explain_codimate` draws three panels and is
split.

## Adding one

```text
python/examples/your_example/
    main.py       the whole explanation
    README.md     what it teaches, and what to try changing
```

Start with one file. Split a panel out only when you find yourself scrolling
to reach it.

See [Daily Workflow](../../docs/daily-workflow.md) for the walkthrough and
[Authoring Model](../../docs/authoring-model.md) for why it is shaped this way.

# Examples

One folder per example, one `main.py` inside it, and a README saying what that
example teaches.

**The folder never splits the example up.** A whole explanation is an
algorithm, a view, some motion rules and some durations, and it stays on one
screen — that is the point of the Python surface. The folder is there to hold
the example's notes and anything it needs, not to reintroduce a module split.

Run any of them from the repository root:

```bash
.venv/bin/python python/examples/bubble_sort/main.py
```

Each writes into [`results/`](../../results/), which is not committed.

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

## Under the hood

**[`explain_codimate/`](explain_codimate/)** — Codimate explaining its own
maths.

Three stored pictures and the formula that computes every frame between them.
The playhead sweeping the timeline is moved by exactly the formula it draws,
and the curve is `cm.ease` — the Engine's own easing, called into rather than
copied.

## Adding one

```text
python/examples/your_example/
    main.py       the whole explanation
    README.md     what it teaches, and what to try changing
```

See [Daily Workflow](../../docs/daily-workflow.md) for the walkthrough and
[Authoring Model](../../docs/authoring-model.md) for why it is shaped this way.

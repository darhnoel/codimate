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

## The one decision Codimate cannot make for you

**[`galton_board/`](galton_board/)** — both kinds of name, side by side.

Pegs and bins never move, so their names follow the *place*:
`("peg", row, i)`, no `cm.items()` anywhere. Balls do move, so each is named
after itself and travels because the trace says where it *is* at each moment —
the view never mentions movement. A dozen are in flight at once, each on its
own path, falling under a real ballistic simulation sampled at a fixed time
step, which is what lets balls at different depths move at different speeds.

Everything else is a consequence of choosing between those two.

## Motion you might think needs a new feature

**[`dharma_wheel/`](dharma_wheel/)** — a turning wheel, with no rotation in the
API.

The trace says where the spokes are every 15 degrees and the Engine fills in
the rest, exactly as it does for a sliding bar. Rotation is just position over
time. The README works through why 15 degrees and not 45.

## Continuous motion from sampled physics

**[`pendulum/`](pendulum/)** — gravity supplies the acceleration; Codimate
connects the moments.

The simulation advances angle and angular velocity at a fixed time step. The
view turns each sampled angle into a string and bob, and `linear` interpolation
keeps that already-continuous motion from stopping at every event.

## Getting it right when the famous version does not

**[`helical_solar_system/`](helical_solar_system/)** — the Sun moves, so every
orbit is a helix.

The popular "solar system is a vortex" video has the orbital plane square to
the direction of travel and the planets trailing behind like a comet's tail.
Neither is true. This one inclines the plane 60° and lets half of each orbit
run ahead of the Sun, which is what actually happens.

**[`bernoulli_lift/`](bernoulli_lift/)** — and the story that goes with it.

The air over a wing really is faster and really is at lower pressure. The
"equal transit time" reason for it is not true, and this measures the two
parcels to show it: the upper one arrives 1.35x sooner. The flow is the exact
Joukowski solution, so the speeds and the times are consequences rather than
choices.

## When the drawing already knows something

**[`rubiks_cube/`](rubiks_cube/)** — a cube solving itself, beside a drawing
that turned out to be the same cube.

Nine overlapping circles, twelve places on each, every place on exactly two.
That is not decoration, it is the cube's nine layers, and the example searches
for the pairing that proves it rather than assuming one. Turn a layer and its
twelve dots slide three places along their circle.

It is also where the drawing order had to be got right four separate times, and
each of those is written down where it happened.

## If you are coming from Manim

**[`manim/`](manim/)** — five of Manim's tutorial scenes, translated.

Not a fight Codimate wins: for two shapes and three verbs, a library built
around shapes-and-verbs is shorter, and the README says so. It is here because
two of them say the quiet part out loud. `DifferentRotations` — where
`.animate` and `Rotate` do different things from calls that look alike — is
the clearest statement of the rule this library runs on: what you put in the
payload decides what the motion is. And `TwoTransforms` asks for a distinction
that does not exist here, because the name *is* the identity.

## When to split

**Split where the knowledge splits, not by the four pieces.**

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

**A seam is worth having when the file on one side of it does not need
Codimate.** That is the test, and it is easy to apply:

```text
rubiks_cube/
    main.py        the four pieces, together, plus the view
    cube.py        the cube as a permutation of 54 stickers — no drawing at all
    geometry.py    where a sticker is in space, and what a turn does to it
    graph.py       the traced drawing: places, lines, circles, colours
    places.py      which place holds which sticker — searched for, then checked
```

Only `main.py` imports `codimate`. The other four are plain Python that can be
run, checked and argued with on their own — and they are: `cube.py` verifies
that `(R U R' U')` has order 6, `places.py` verifies that a quarter turn slides
a circle's twelve places exactly three along it. Neither needs a video to say
whether it is right.

`galton_board` splits on the same test for one file: `physics.py` is the
ballistic simulation and imports nothing of ours.

**Rough threshold:** one `main.py` until it passes ~150 lines or grows a part
that could be checked without rendering anything. `bubble_sort` (57 lines) and
`galton_board`'s view (72) are single files and should stay that way.

## Adding one

```text
python/examples/your_example/
    main.py       the whole explanation
    README.md     what it teaches, and what to try changing
```

Start with one file. Split a part out only when you could test it on its own.

See [Writing Your First Animation](../../docs/tutorial.md) for the walkthrough
and [How Codimate Thinks](../../docs/concepts.md) for why it is shaped this way.

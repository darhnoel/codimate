# cavendish — how do you weigh the Earth?

```bash
.venv/bin/python python/examples/cavendish/main.py    # results/cavendish.mp4
```

Two minutes, in Khmer captions, on one question: we can weigh a rock and a person,
so how do we weigh the whole Earth? Five parts:

1. **The impossible question** — an Earth dropped into one pan of a balance, which tips.
2. **Newton's clue** — the Earth pulls an apple down, and so do two balls on a table.
3. **The catch** — the pull between two lab balls is absurdly small.
4. **Turn the invisible into motion** — Michell's torsion balance, seen from above.
5. **From a tiny twist to the planet** — twist angle, then force, then G, then the mass
   of the Earth, and a zoom out to say how small the twist was.

## What it teaches

**Zoom levels nested round one point.** The apparatus, a lab, a city and the planet
are all drawn at their own scale about `ZC`, and `lvl_op` fades each in and out as the
zoom passes it, so zooming out is one number going down rather than five scenes. The
twist is taught large (`PHI_EXAG`) so it can be seen, and drawn at its real size
(`PHI_REAL`) only at the end, where seeing nothing is the point.

**A story held by its captions.** Each line is read word by word on a plate, as in
[`archimedes`](../archimedes/), and the scene lasts as long as the line takes to read.
`say()` starts a line, the scene does whatever it does while it is read, and `finish()`
waits out the rest. No duration is tuned by hand.

**One moment per tick.** Motion is sampled at `TICK` (30 a second) and the film renders
at 30 fps to match, so nothing is interpolated between two draw orders.

**Arguments that travel together are one value.** `Look` (colour, opacity, layer, size),
`Arc` and `Pose` exist because no function here takes more than five arguments.

## The captions

They are Khmer, and Khmer has no spaces, so the mark cannot step word by word without
knowing where the words are. `segment_lines.py` inserts U+200B at each word boundary and
writes `lines.py`; the film imports that and needs no segmenter. It borrows the segmenter
that `archimedes/segment.py` finds, so it only runs on a machine that has it.

## Not here

It is drawn by hand, from the same spheres and projections the science kit later
extracted. [`orbits/`](../orbits/) and [`year/`](../year/) are what the kit looks like
in use; this is what it was cut out of, and it has not been moved onto the kit.

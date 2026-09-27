# ADR 0017 — A check that says a label is covered

**Status:** Accepted — 2026-09-25

## Context

Building one explainer end to end surfaced the same bug six times, wearing six
different faces:

| what happened | what it was |
| --- | --- |
| the material's name sat under the force arrow | text with a shape over it |
| the weight arrow's label ran through the caption | text over text |
| the `91.7% SUBMERGED` readout landed on the caption | text over text |
| the same readout later landed on the floating box | text with a shape over it |
| the submerged bracket's labels sat on the glass | text with a shape over it |
| the chart's axis label collided with its own unit | text over a formula |

Every one is a label something was drawn on top of. None was caught by a test,
because none is wrong in any way the code can be wrong — the film renders, the
assertions in `world.py` pass, the numbers are right. They were caught by a
human looking at a frame and saying so, which cost a round trip each.

The obvious counter is "just check for overlapping shapes", and it does not
survive contact with a real film. Shapes overlap constantly and on purpose:
water is drawn over the body at 0.46 opacity so submerged things look
submerged, the cavity is drawn over the water so the inside of a box reads as
air, and every label sits on a plate. A blanket overlap warning would be almost
entirely false positives, and a warning that is usually wrong is worse than no
warning.

What the six have in common is narrower and checkable: **a text had something
drawn on top of it**, or **two texts were in the same place**. The plate under
a label is drawn *below* it and passes; the arrow over the label is drawn
*above* it and does not.

One more thing decided where this could live. The Authoring Surface can measure
text — `cm.measure` asks the Engine, with the real fonts — but it cannot
measure a **Formula**, because a Formula is typeset by Typst when the video is
built ([ADR 0005](0005-formula-via-typst-subprocess.md)). The chart's axis
label is exactly that case, and it is why the fix for it was guesswork: put the
word on one side of the water line and the quantity on the other, because there
was nothing to measure.

## Decision

**The Engine complains, once per Trace Event, when a label is covered.**

Two rules, and no configuration:

1. Nothing may be drawn *above* a text. Any fill counts, at any opacity.
2. No two texts may overlap.

The report goes where `wrote results/x.mp4` already goes:

```
wrote results/archimedes-km.mp4

2 labels are covered:
  0:31  "wt" is drawn over "body_word"
  0:54  "big" overlaps "say"

rendered anyway.
```

### In the Engine, because that is where the boxes are

The Engine already computes every shape's box in order to draw it — shaped
Khmer text, Typst-typeset formulas and all. A check in Python would be blind to
a whole shape kind and would have missed one of the six. This is a per-frame
concern in the glossary's sense — it is about what the rasteriser sees — even
though it is sampled per event.

### Once per Trace Event, not per frame

An event is where an author decided something, and all six bugs were steady
states somebody authored rather than collisions that flashed past mid-tween.
The cost decides the rest: `spacetime` draws 1,226 shapes across some 9,700
frames, so a per-frame check is millions of box tests and would have to be
opt-in — which means off in exactly the case it was built for.

### Any fill, at any opacity

There is no threshold. Translucency does not rescue legibility: 46% water over
a caption is as unreadable as a solid arrow over a label and far harder to
notice. A number here would be arbitrary, would be wrong for some palette, and
would leave a reader asking where it came from.

### It complains and never fails

The rule is a guess about intent, and a tool that can be wrong must not stop
work. It is always on, so it protects films nobody thought to annotate, and
`tests/run.py` can assert the report is empty for the examples that should be
clean.

## Consequences

- **Existing examples may light up.** That is the point, and each one is either
  a bug worth fixing or a line the author learns to expect.
- **A deliberate overlap has no escape hatch**, by design. Adding one would
  make the check advisory, and an advisory check is a comment.
- **The Engine now holds an opinion about authoring quality**, which is new. It
  is confined to reporting: nothing about the render changes, and the One Law
  is untouched because the check reads a resolved Scene and writes no part of
  it.

## Alternatives rejected

**The author declares what must stay apart** — `cm.apart("title", "say")`.
Exact, and able to express things the automatic rule cannot. Rejected because
it only protects what somebody remembered to list, and all six bugs were in
places nobody was thinking about.

**A helper in Python, in the spirit of [ADR 0016](0016-axes-that-hand-back-pixels.md).**
Ships without touching Rust and catches five of the six. Rejected for the sixth:
a Formula has no box until Typst has run.

**Every frame.** Nothing escapes, including a label that only crosses an arrow
halfway through a tween. Rejected on cost, which would force it to be opt-in.

**Failing the render.** Impossible to ignore, and impossible to ship: every
example would need auditing first, and the first deliberate overlap anywhere
reintroduces the escape hatch.

**An opacity threshold.** Kinder to faint washes, and an unjustifiable number.

## Amended: built in Python, where the names are

The decision put the check in the Engine for one reason: a Formula had no box
until Typst ran. That stopped being true — `cm.measure_math` asks the Engine,
which asks Typst once and caches the glyphs it will draw — so the one case a
Python check would miss is no longer missed.

And the Engine turned out to be the wrong side for the other half of the job.
By the time a Scene is resolved it carries geometry and no names; reporting
`"wt" is drawn over "body_word"` from there would mean threading every name
through reconcile, layout and render only to print it. The Authoring Surface
holds the names, the draw order `(layer, name)`, every box, and the index the
Previewer reads.

So `covered(scene)` lives beside `box()`, and everything else stands as
decided: the two rules, no configuration, any opacity, once per Trace Event,
reported where a collision starts, complain and never fail. `render` prints the
report; the index carries it per beat; the Previewer lists it. A whole
162-second film checks in 0.3s.

Measured against the examples on the day it landed: one real bug (otsu's axis
labels centred at a fixed x, so "1000" ran into its tick — fixed in the axes
helper), one deliberate overlap (archimedes' typewriter mask over its title),
and no false positives once a label's edge was allowed 2px: a measured box
includes the blank side bearing of its outer glyphs, so words set edge to
edge are not covered.

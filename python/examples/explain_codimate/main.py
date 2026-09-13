"""Codimate explaining the maths behind Codimate.

    python python/examples/explain_codimate/main.py

The bars on the right are three moments of a bubble sort: compare 3 and 1,
swap them, compare 3 and 4. Codimate stores only those three pictures. Every
frame in between is computed:

        local = (t - start) / duration
        value = before + (after - before) * ease(local)

The playhead sweeping the timeline is itself moved by that formula. This trace
records 28 instants; everything you see between them is worked out.

This example is split up because it draws three separate panels, not because
an explanation needs splitting — the four pieces still live here, together:

    story.py     what is being explained
    theme.py     where the panels sit, and what colour things are
    timeline.py  the timeline panel
    curve.py     the easing panel
    bars.py      the bars panel
"""

import codimate as cm

import bars
import curve
import timeline
from story import Clock, TICKS, TOTAL
from theme import INK


# --- the algorithm ----------------------------------------------------------


@cm.trace()
def sweep(clock):
    for tick in range(TICKS):
        clock.t = TOTAL * tick / (TICKS - 1)
        cm.emit("tick")


# --- the view ---------------------------------------------------------------


def explain_frame(frame):
    scene = cm.Scene()
    clock = frame.state
    active, start, duration, local = clock.locate()

    scene.text("title", "f(t) → Scene", size=44, at=(cm.width() / 2, 72)).fill(INK)
    scene.text("sub",
               "three pictures are stored — every frame between them is computed",
               size=21, at=(cm.width() / 2, 112)).fill("grey")
    scene.text("formula", f"local = (t − {start:.1f}) / {duration:.1f}  =  {local:.2f}",
               size=26, at=(cm.width() / 2, 310)).fill(INK)

    timeline.draw(scene, clock, active)
    curve.draw(scene, local)
    bars.draw(scene, active, local)
    return scene


# --- motion and timing ------------------------------------------------------

cm.explain(
    trace=sweep(Clock()),
    view=explain_frame,
    timing=cm.Timing(default=0.13, opening=1.4, final_hold=2.0),
).render("results/explain_codimate.mp4", fps=60, scale=1.5)

print("wrote results/explain_codimate.mp4")

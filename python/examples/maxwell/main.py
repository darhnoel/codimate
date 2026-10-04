"""How did Maxwell find his equations? The discovery as one story.

1. The question — four equations, and one term added later
2. Electricity moves a compass — Ørsted and Ampère, 1820
3. Magnetism makes electricity — Faraday, 1831
4. Faraday's picture — fields are real, and fill space
5. The hole — Ampère's law gives two answers in a capacitor
6. Maxwell's patch — a changing E acts like a current
7. The wave — E and B, rising and falling together
8. What ε₀ and μ₀ are — charges pushing, wires pulling
9. The speed — the number that matched light

    .venv/bin/python python/examples/maxwell/main.py
    .venv/bin/python python/examples/maxwell/main.py --silent

Where things are:

    scenes/       one module per scene, each drawing a single moment
    pen.py        how a scene draws: a Look, and a Stage that fades shapes in and out
    palette.py    the canvas and the colours
    equations.py  the equations, as LaTeX
    geometry.py   curves, arrowheads and laying formulas side by side
    narration.py  the voice, the word timings, and the caption
    lines.py      the Khmer captions (made by segment_lines.py)
    speak.py      records the captions and hears when each word is said

This file is what connects them: it decides how long each scene lasts, emits one
moment per frame, and hands the result to `cm.explain`. Every equation is LaTeX,
written on the moment its discovery happens. Each scene lasts as long as its line
takes to say."""

import codimate as cm
import narration
from palette import H
from pen import Stage, clamp
from scenes import SCENES

FPS = 30  # one moment per frame, so nothing is interpolated between two
FADE = 0.5  # how long a scene takes to arrive, and to leave
OUT = "results/maxwell-silent.mp4" if narration.SILENT else "results/maxwell.mp4"


def length(k):
    """A scene is as long as its line takes to say, and never shorter than the
    picture needs."""
    key, _, least = SCENES[k]
    return max(least, narration.LEAD + narration.seconds(key) + narration.TAIL)


def story(state, emit):
    for k, (key, _, _) in enumerate(SCENES):
        total = round(length(k) * FPS)
        heard, cued = narration.clip(key), False
        for i in range(total + 1):
            t = length(k) * i / total
            # the caption comes in just after the scene and leaves just before it
            shown = 0.2 <= t <= length(k) - 0.4
            state.update(
                k=k,
                t=t,
                line=key if shown else "",
                said=narration.said(key, t),
            )
            cue = {}
            if heard and not cued and t >= narration.LEAD:
                # the recording rides on the event the line starts at, so the
                # voice and the picture are timed by the same clock
                cue, cued = {"sound": narration.AUDIO / heard["file"]}, True
            emit("tick", **cue)


def view(frame):
    state = frame.state
    k, t = state["k"], state["t"]
    D = length(k)
    scene = cm.Scene()
    SCENES[k][1](Stage(scene, k, clamp(min(t, D - t) / FADE)), t / D, t, D)
    narration.draw(scene, state["line"], state["said"], y=H - 60)
    return scene


cm.explain(
    trace=cm.trace(story, {"k": 0, "t": 0.0, "line": "", "said": 0}),
    view=view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=1 / FPS, opening=0.4, final_hold=0.6),
).render(OUT, fps=FPS, scale=1.5)

print(f"wrote {OUT}")

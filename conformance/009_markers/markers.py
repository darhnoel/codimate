"""Markers and highlights: a stroke drawn on and turned, a pink one, and a
highlighted word whose band rubs out while its words lighten in step."""

import sys

import codimate as cm


def walk(state, emit):
    for reveal in (0.5, 0.0):
        state["reveal"] = reveal
        emit("rub")


def view(frame):
    scene = cm.Scene()
    reveal = frame.state["reveal"]
    scene.marker("note", w=260, h=70, at=(320, 200)).turn(-8).write(reveal)
    scene.marker("aside", w=180, h=50, at=(900, 200)).fill("pink")
    band, words = scene.highlight("pivot", "Pivot", size=40, at=(640, 450))
    words.fill("#ffd60a")
    band.write(reveal)
    return scene


cm.explain(trace=cm.trace(walk, {"reveal": 1.0}), view=view).export_web(sys.argv[1])

"""Axes: a plot with a grid and its own ink, a curve from the author's own
function, and a dot that moves along it, mapped by `at`."""

import sys

import codimate as cm

plot = cm.axes(x=(-4, 4), y=(-2, 6), size=(760, 420)).looks(ink="#5b6478", label=16)


def walk(state, emit):
    for t in (-1.0, 2.0):
        state["t"] = t
        emit("move")


def view(frame):
    t = frame.state["t"]
    scene = cm.Scene()
    plot.draw(scene, grid=True)
    scene.curve("f", plot.line(lambda x: x * x / 2, steps=80), w=4).fill("#ff9f0a")
    scene.circle("dot", r=9, at=plot.at(t, t * t / 2)).fill("white")
    return scene


cm.explain(trace=cm.trace(walk, {"t": -3.0}), view=view).export_web(sys.argv[1])

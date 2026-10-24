"""A line, an arrow, a polygon, a curve and an arc, each changing between two
moments, so their points and angles are carried and tweened."""

import sys

import codimate as cm


def walk(state, emit):
    state["t"] = 1
    emit("grow")


def view(frame):
    t = frame.state["t"]
    scene = cm.Scene()
    scene.line("axis", start=(100, 600), end=(1180, 600), w=2).fill("#48484a")
    scene.arrow("force", start=(200, 500), end=(200 + 300 * (1 + t), 500 - 120 * t),
                w=6, head=24).fill("#ff9f0a")
    scene.polygon("roof", [(500, 300), (640, 180 - 40 * t), (780, 300)]).fill("#30d158")
    scene.polygon("open", [(900, 100), (1000, 160), (1100, 100)], closed=False).fill(
        "none", edge="#bf5af2", edge_w=3)
    wave = [(100, 400), (300, 350 - 50 * t), (500, 400), (700, 450)]
    scene.curve("wave", wave, w=3).fill("#64d2ff")
    scene.arc("angle", r=(90, 60), sweep=(0, 40 + 80 * t), at=(1000, 450)).fill(
        "none", edge="#ffd60a", edge_w=4)
    return scene


cm.explain(trace=cm.trace(walk, {"t": 0}), view=view).export_web(sys.argv[1])

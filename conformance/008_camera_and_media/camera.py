"""A formula drawn on, a picture, vector art, an overlay title the camera
leaves where it is, and a camera that moves to frame two things."""

import sys

import codimate as cm


def walk(state, emit):
    state["look"] = True
    emit("zoom")


def view(frame):
    scene = cm.Scene()
    scene.formula("eq", r"\frac{QK^{T}}{\sqrt{d_k}}", size=40, at=(400, 300)).write(
        reveal=1.0 if frame.state["look"] else 0.3, pen=2)
    scene.image("photo", "figures/attention.png", size=(320, 180), at=(900, 300))
    scene.svg("logo", "brand.svg", size=90, at=cm.at(x=1180, top=24))
    hud = scene.overlay()
    hud.text("title", "Scaled dot-product attention", size=32, at=(640, 52))
    if frame.state["look"]:
        scene.focus("eq", "photo", pad=60)
    return scene


cm.explain(trace=cm.trace(walk, {"look": False}), view=view).export_web(sys.argv[1])

"""A ball moves left to right twice, with its label following it."""

import sys
from dataclasses import dataclass

import codimate as cm


@dataclass
class Ball:
    x: float = 200


def walk(ball, emit):
    for x in (640, 1080):
        ball.x = x
        emit("move")


def view(frame):
    scene = cm.Scene()
    scene.rect("floor", h=4, w=1100, at=(640, 404)).fill("#48484a")
    scene.circle("ball", r=40, at=(frame.state.x, 360)).fill("#2997ff", edge="#ffffff")
    scene.text("label", "ball", size=28, at=cm.at(x=frame.state.x, top=420))
    return scene


cm.canvas(1280, 720)
cm.explain(trace=cm.trace(walk, Ball()), view=view).export_web(sys.argv[1])

from dataclasses import dataclass

import codimate as cm

cm.canvas(640, 360)


@dataclass
class Ball:
    x: float = 100


def move(ball, emit):
    ball.x = 540
    emit("moved")


def view(frame):
    scene = cm.Scene()
    scene.circle("ball", r=30, at=(frame.state.x, 180)).fill("blue")
    return scene


explanation = cm.explain(
    trace=cm.trace(move, Ball()),
    view=view,
    timing=cm.Timing(default=2.0, opening=0.5, final_hold=0.5),
)
explanation.render("ball.mp4")

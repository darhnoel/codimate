"""Manim's `AnimateExample`, written in Codimate.

    python python/examples/manim/animate_example.py

    square = Square().set_fill(RED, opacity=1.0)
    self.add(square)

    self.play(square.animate.set_fill(WHITE))
    self.wait(1)
    self.play(square.animate.shift(UP).rotate(PI / 3))
    self.wait(1)

Two things translate almost for free, and the second is the interesting one.

`self.add(square)` is not an animation, so there is nothing to translate: the
square is simply in the first Scene. And `.animate.set_fill(WHITE)` is a colour
in the payload, which the Engine tweens like any other number.

The chain — `.shift(UP).rotate(PI / 3)` — is Manim composing two animations so
they play together. There is nothing to compose here. Two Scenes differ by a
position *and* an angle, so both change over the same beat, because **everything
that differs between two Scenes happens at once** is not a feature, it is the
only thing a difference can mean.
"""

import codimate as cm

import shapes

cm.canvas(1280, 720)

RED, WHITE, PAPER = "#fc6255", "#ffffff", "#8b96a8"
LIFT = 130.0


def animate_example(state, emit):
    state["colour"] = WHITE                  # .animate.set_fill(WHITE)
    emit("recolour")
    emit("wait")                          # .wait(1) — nothing differs
    state["lifted"], state["spin"] = True, 60.0   # .shift(UP).rotate(PI / 3)
    emit("move")
    emit("wait")


def view(frame):
    scene = cm.Scene()
    state = frame.state

    # Shifting moves the points; turning is a field of its own. Both differ
    # between the last two Scenes, so both run over the same beat.
    middle = (shapes.MIDDLE[0],
              shapes.MIDDLE[1] - (LIFT if state["lifted"] else 0.0))
    # `closed=False` because `outline` already ends on its own first point;
    # closing it again adds a zero-length segment that notches the stroke.
    scene.polygon("square", shapes.outline(shapes.square, centre=middle),
                  closed=False) \
         .turn(state["spin"]) \
         .fill(state["colour"], edge=state["colour"], edge_w=4)

    # `frame.event` is None for the opening moment, before anything happened.
    said = ("set_fill(WHITE)" if frame.is_("recolour")
            else "shift(UP).rotate(PI / 3)" if frame.is_("move")
            else "add(square)" if frame.event is None else "wait(1)")
    scene.text("said", said, size=30, at=cm.at(x=640, bottom=650)).fill(PAPER)
    return scene


cm.explain(
    trace=cm.trace(animate_example, {"colour": RED, "lifted": False, "spin": 0.0}),
    view=view,
    timing=cm.Timing(default=1.0, events={"wait": 1.0},
                     opening=0.6, final_hold=0.4),
).render("results/manim_animate_example.mp4", fps=60, scale=1.5)

print("wrote results/manim_animate_example.mp4")

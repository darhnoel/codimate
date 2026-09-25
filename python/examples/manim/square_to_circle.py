"""Manim's `SquareToCircle`, written in Codimate.

    python python/examples/manim/square_to_circle.py

Manim names three animations and plays them:

    self.play(Create(square))
    self.play(Transform(square, circle))
    self.play(FadeOut(square))

There is no equivalent call here, and that is the point of the example. The
square being drawn on is *sampled*, the way every continuous motion in this
library is sampled. The morph is what two consecutive Scenes sharing a name
means. Fading out is what leaving means. See the README for what that buys and
what it costs.
"""

import codimate as cm

import shapes

cm.canvas(1280, 720)

PINK, CHALK = "#d147a3", "#ffffff"
STEPS = 48          # samples of the pen going round
TILT = 45.0         # Manim rotates the square before it draws it


def square_to_circle(state, emit):
    for step in range(1, STEPS + 1):        # Create(square)
        state["drawn"] = step / STEPS
        emit("draw")
    state["shape"] = "circle"               # Transform(square, circle)
    emit("morph")
    state["shape"] = None                   # FadeOut(square)
    emit("gone")


def view(frame):
    scene = cm.Scene()
    state = frame.state
    if state["shape"] == "circle":
        scene.polygon("shape", shapes.outline(shapes.circle, tilt=TILT),
                      closed=False) \
             .fill(PINK, edge=PINK, edge_w=4).on(opacity=0.5)
    elif state["shape"] == "square":
        scene.polygon("shape",
                      shapes.outline(shapes.square, state["drawn"], TILT),
                      closed=False) \
             .fill("none", edge=CHALK, edge_w=4)
    return scene


cm.explain(
    trace=cm.trace(square_to_circle, {"shape": "square", "drawn": 0.0}),
    view=view,
    # The pen is already continuous by the time it reaches the Engine, so it
    # must not be eased again at every sample — `dharma_wheel` and `pendulum`
    # need the same rule for the same reason.
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=0.02, events={"morph": 1.2, "gone": 0.8},
                     opening=0.5, final_hold=0.8),
).render("results/manim_square_to_circle.mp4", fps=60, scale=1.5)

print("wrote results/manim_square_to_circle.mp4")

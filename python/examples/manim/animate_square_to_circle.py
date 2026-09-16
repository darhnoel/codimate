"""Manim's `AnimatedSquareToCircle`, written in Codimate.

    python python/examples/manim/animate_square_to_circle.py

Manim's version leans on `.animate`, which turns any method call into an
animation:

    self.play(Create(square))
    self.play(square.animate.rotate(PI / 4))
    self.play(Transform(square, circle))
    self.play(square.animate.set_fill(PINK, opacity=0.5))

Four beats, four states. The rotation is the one that maps most directly:
`.turn()` is a field on the shape and the Engine tweens it, so a quarter turn
is one number changing between two Scenes rather than an animation object.
"""

import codimate as cm

import shapes

cm.canvas(1280, 720)

PINK, CHALK = "#d147a3", "#ffffff"
STEPS = 48


@cm.trace()
def animated_square_to_circle(state):
    for step in range(1, STEPS + 1):        # Create(square)
        state["drawn"] = step / STEPS
        cm.emit("draw")
    state["tilt"] = 45.0                    # square.animate.rotate(PI / 4)
    cm.emit("turn")
    state["shape"] = "circle"               # Transform(square, circle)
    cm.emit("morph")
    state["filled"] = True                  # square.animate.set_fill(PINK, 0.5)
    cm.emit("fill")


def view(frame):
    scene = cm.Scene()
    state = frame.state

    # The square is drawn square and turned afterwards, so the outline itself
    # carries no tilt — `.turn()` does, and the Engine tweens it. During the
    # drawing the pivot would be the middle of a part-drawn arc rather than of
    # the square, which is why the turn is a beat of its own and not a rule.
    if state["shape"] == "circle":
        points = shapes.outline(shapes.circle)
    else:
        points = shapes.outline(shapes.square, state["drawn"])

    edge = scene.polygon("shape", points, closed=False).turn(state["tilt"])
    if state["filled"]:
        edge.fill(PINK, edge=PINK, edge_w=4).on(opacity=0.5)
    else:
        edge.fill("none", edge=CHALK, edge_w=4)
    return scene


cm.explain(
    trace=animated_square_to_circle({"shape": "square", "drawn": 0.0,
                                     "tilt": 0.0, "filled": False}),
    view=view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=0.02,
                     events={"turn": 1.0, "morph": 1.2, "fill": 1.0},
                     opening=0.5, final_hold=1.0),
).render("results/manim_animated_square_to_circle.mp4", fps=60, scale=1.5)

print("wrote results/manim_animated_square_to_circle.mp4")

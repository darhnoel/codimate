"""Manim's `DifferentRotations`, written in Codimate.

    python python/examples/manim/different_rotations.py

    self.play(
        left_square.animate.rotate(PI), Rotate(right_square, angle=PI), run_time=2
    )

This is the most useful of the three to translate, because the thing it exists
to show is the thing Codimate is built around.

Manim's `.animate` interpolates a shape's *points* from where they start to
where they end. Half way through a half turn every corner is half way to its
opposite one, so the square flattens to a line and opens out again. `Rotate`
turns the shape instead, so it stays square the whole way.

Both are one call in Manim and it is not obvious which you have written. Here
they are visibly different things:

    left.  polygon("left", corners turned by state["spin"])   the points move
    right. polygon("right", corners).turn(state["spin"])      the shape turns

The left square's corners are *in the payload*, so the reconciler interpolates
them one by one and gives Manim's flattening exactly. The right square's angle
is a field the Engine spins, so it turns. Same half turn, same two seconds.
"""

import math

import codimate as cm

import shapes

cm.canvas(1280, 720)

BLUE, GREEN, PAPER = "#58c4dd", "#83c167", "#e8eef7"
LEFT, RIGHT = (400.0, 380.0), (880.0, 380.0)


def different_rotations(state, emit):
    state["spin"] = 180.0
    emit("turn")


def _turned(centre, degrees):
    """The square's corners, actually rotated — what `.animate` interpolates
    between rather than passes through."""
    a = math.radians(degrees)
    return [(centre[0] + (x - centre[0]) * math.cos(a)
             - (y - centre[1]) * math.sin(a),
             centre[1] + (x - centre[0]) * math.sin(a)
             + (y - centre[1]) * math.cos(a))
            for x, y in shapes.corners(centre=centre)[:4]]


def view(frame):
    scene = cm.Scene()
    spin = frame.state["spin"]

    # The corners are the thing that changes, so the corners are what gets
    # interpolated — straight lines to the far side, through the middle.
    scene.polygon("left", _turned(LEFT, spin)) \
         .fill(BLUE, edge=BLUE, edge_w=4).on(opacity=0.7)

    # The angle is the thing that changes, so the angle is what gets
    # interpolated, and the square stays a square.
    scene.polygon("right", shapes.corners(centre=RIGHT)[:4]).turn(spin) \
         .fill(GREEN, edge=GREEN, edge_w=4).on(opacity=0.7)

    scene.text("left_label", "corners in the payload", size=26,
               at=cm.at(x=LEFT[0], top=560)).fill(PAPER).on(opacity=0.75)
    scene.text("right_label", "angle in the payload", size=26,
               at=cm.at(x=RIGHT[0], top=560)).fill(PAPER).on(opacity=0.75)
    return scene


cm.explain(
    trace=cm.trace(different_rotations, {"spin": 0.0}),
    view=view,
    timing=cm.Timing(default=2.0, opening=0.6, final_hold=1.2),   # run_time=2
).render("results/manim_different_rotations.mp4", fps=60, scale=1.5)

print("wrote results/manim_different_rotations.mp4")

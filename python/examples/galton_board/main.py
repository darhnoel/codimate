"""A Galton board — where the bell curve comes from.

    python python/examples/galton_board/main.py

Balls are dropped through a triangle of pegs. At each peg a ball goes left or
right, each with even chance, and lands in the bin counting how many times it
went right. Nothing aims for a bell curve; it is what evenly-weighted coin
flips add up to.

The falling is simulated, not faked: gravity, energy lost to every peg,
parabolas between one peg and the next, balls with bodies that cannot pass
through each other, and dividers that catch a ball still carrying sideways
momentum off the last peg.

    board.py     how big the board is and where everything sits
    physics.py   the simulation — knows nothing about being drawn
    view.py      the drawing — reads the simulation, never steers it

Two identity modes at once, which is what makes this one worth reading:

    pegs and bins  named after their PLACE  — they never move
    balls          named after the BALL     — they fall
"""

import codimate as cm

import physics
import view
from physics import DT


# --- the algorithm ----------------------------------------------------------


@cm.trace()
def drop(board):
    """Run the simulation, sampling it at a fixed step of real time.

    One event per row instead would give every ball the same duration per row,
    which is a constant descent rate however the motion is drawn. Sampling
    time means a ball near the bottom, moving faster, covers more ground per
    tick — which is the whole point of simulating it.
    """
    while not board.done():
        board.step()
        cm.emit("tick")
    cm.emit("settled")


# --- the view ---------------------------------------------------------------


def board_view(frame):
    scene = cm.Scene()
    view.draw(scene, frame.state)
    return scene


# --- motion and timing ------------------------------------------------------

# `linear`: the simulation is sampled densely enough that the parabolas are
# already in the samples. Easing between them would add a wobble physics never
# asked for.
cm.explain(
    trace=drop(physics.Board()),
    view=board_view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=DT, events={"settled": 0.8},
                     opening=1.0, final_hold=2.4),
).render("results/galton_board.mp4", fps=60, scale=1.5)

print("wrote results/galton_board.mp4")

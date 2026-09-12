"""Bernoulli's principle, and the story that usually goes with it.

    python python/examples/bernoulli_lift/main.py

The air over the top of a wing really is faster, and faster really does mean
lower pressure. That much of the familiar diagram is right.

The reason given for it is not. "The air over the top has further to go, so it
must speed up to meet its partner at the back" requires the two halves to
arrive together. They do not. Two parcels are released side by side here,
either side of the streamline that actually splits at the nose, and the clock
in the corner reports when each reaches the trailing edge.

The flow is not drawn. It is the exact potential-flow solution for a Joukowski
airfoil, with the circulation fixed by the Kutta condition — which is where
lift comes from, and which says nothing about path length.

    airfoil.py       the wing and the exact velocity field — complex arithmetic
    flow.py          parcels carried along it
    scene_setup.py   where it sits on screen, and the speed colours
    view.py          the drawing
"""

import codimate as cm

import flow
import view
from flow import DT


# --- the algorithm ----------------------------------------------------------


@cm.trace()
def blow(air):
    while not air.done():
        air.step()
        cm.emit("tick")
    cm.emit("settled")


# --- the view ---------------------------------------------------------------


def flow_view(frame):
    scene = cm.Scene()
    view.draw(scene, frame.state)
    return scene


# --- motion and timing ------------------------------------------------------

# `linear`: every parcel is mid-flight at every tick.
cm.explain(
    trace=blow(flow.Flow()),
    view=flow_view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=DT * 0.8, events={"settled": 1.0},
                     opening=1.0, final_hold=3.0),
).render("results/bernoulli_lift.mp4", fps=60, scale=1.5)

print("wrote results/bernoulli_lift.mp4")

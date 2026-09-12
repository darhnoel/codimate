"""The helical model — our solar system as it moves through the galaxy.

    python python/examples/helical_solar_system/main.py

The Sun is not still. It carries the whole system through the galaxy at about
230 km/s, so relative to anything outside, a planet does not trace an ellipse —
it traces a helix around the Sun's path.

That much of the popular "vortex" video is right. Two things in it are not, and
this does them properly:

  * The orbital plane is inclined about 60 degrees to the galactic plane. It is
    not square to the direction of travel.
  * The planets do not trail behind the Sun like a comet's tail. Half of every
    orbit is ahead of it. Watch Venus fall behind while Earth runs ahead.

    space.py    the measurements, and how 3D reaches a flat screen
    orbits.py   where every body is, tick by tick — no Codimate in it
    view.py     the drawing
"""

import codimate as cm

import orbits
import view
from space import YEARS


# --- the algorithm ----------------------------------------------------------


@cm.trace()
def travel(system):
    while system.years < YEARS:
        system.step()
        cm.emit("tick")


# --- the view ---------------------------------------------------------------


def helical_view(frame):
    scene = cm.Scene()
    view.draw(scene, frame.state)
    return scene


# --- motion and timing ------------------------------------------------------

# `linear`: every body is mid-orbit at every tick, so easing would make the
# whole system surge and stall once per sample.
cm.explain(
    trace=travel(orbits.System()),
    view=helical_view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=0.055, opening=0.8, final_hold=2.0),
).render("results/helical_solar_system.mp4", fps=60, scale=1.5)

print("wrote results/helical_solar_system.mp4")

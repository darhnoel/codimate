"""A swinging pendulum — continuous motion from sampled physics.

    .venv/bin/python python/examples/pendulum/main.py

The simulation advances at a fixed time step and records one moment per tick.
Codimate connects those closely spaced moments with linear motion; it does not
need a pendulum-specific animation primitive.
"""

import math

import codimate as cm


# --- the simulation ---------------------------------------------------------

DT = 1 / 30
DURATION = 10.0

GRAVITY = 9.81
LENGTH_METRES = 1.0
DAMPING = 0.08

PIVOT = (640.0, 145.0)
ROD_LENGTH = 240.0


class Pendulum:
    def __init__(self):
        self.angle = math.radians(50.0)
        self.angular_velocity = 0.0
        self.time = 0.0

    def step(self):
        acceleration = (
            -(GRAVITY / LENGTH_METRES) * math.sin(self.angle)
            - DAMPING * self.angular_velocity
        )

        # Semi-implicit Euler: update velocity before position. It is nearly
        # as small as Euler's method, but behaves much better for oscillation.
        self.angular_velocity += acceleration * DT
        self.angle += self.angular_velocity * DT
        self.time += DT

    def bob_position(self):
        return (
            PIVOT[0] + ROD_LENGTH * math.sin(self.angle),
            PIVOT[1] + ROD_LENGTH * math.cos(self.angle),
        )


# --- the algorithm ----------------------------------------------------------


@cm.trace()
def swing(pendulum):
    for _ in range(round(DURATION / DT)):
        pendulum.step()
        cm.emit("tick")
    cm.emit("done")


# --- the view ---------------------------------------------------------------


def pendulum_view(frame):
    scene = cm.Scene()
    pendulum = frame.state
    bob = pendulum.bob_position()

    scene.text("title", "A Simple Pendulum", x=640, y=62, size=38, color="#e8eef7")
    scene.text(
        "equation",
        "theta'' = -(g / L) sin(theta) - damping",
        x=640,
        y=104,
        size=21,
        color="#8f9bad",
    )

    # A quiet equilibrium marker makes the shrinking amplitude visible.
    scene.line(
        "equilibrium",
        start=PIVOT,
        end=(PIVOT[0], PIVOT[1] + ROD_LENGTH + 48),
        w=1,
        color="#303947",
    )
    scene.line("string", start=PIVOT, end=bob, w=5, color="#aab4c3", layer=1)
    scene.circle("pivot", at=PIVOT, r=10, color="#e8eef7", layer=2)
    scene.circle("bob", at=bob, r=30, color="#f5a623", layer=3)

    scene.text(
        "reading",
        f"t = {pendulum.time:4.1f} s     angle = {math.degrees(pendulum.angle):+5.1f} deg",
        x=640,
        y=570,
        size=24,
        color="#aab4c3",
    )

    if frame.is_("done"):
        scene.text(
            "done",
            "10 seconds of simulated motion",
            x=640,
            y=625,
            size=22,
            color="#6fce88",
        )

    return scene


# --- motion and timing ------------------------------------------------------

# The simulation already contains the acceleration and curved trajectory.
# Linear motion joins its dense samples without adding stop-start easing.
cm.explain(
    trace=swing(Pendulum()),
    view=pendulum_view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(
        default=DT,
        events={"done": 0.8},
        opening=1.0,
        final_hold=2.0,
    ),
).render("results/pendulum.mp4", fps=60, scale=1.5)

print("wrote results/pendulum.mp4")

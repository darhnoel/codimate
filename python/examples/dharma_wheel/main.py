"""The Dharmachakra turning — the wheel of the Noble Eightfold Path.

    python python/examples/dharma_wheel/main.py

Eight spokes, one for each factor of the path. The wheel turns, and the spoke
reaching the top is named below it.

There is no rotation in Codimate. There does not need to be: the trace says
where the spokes are every 15 degrees, and the Engine works out everything in
between — the same way it works out a bar sliding between two slots. Rotation
is just position over time.
"""

import math

import codimate as cm

CENTRE = (640.0, 372.0)
RIM_OUTER, RIM_INNER = 206.0, 182.0
HUB_OUTER, HUB_INNER = 46.0, 30.0
SPOKE_W = 11.0

SPOKES = 8
STEP = 15                      # degrees per event: 1.5px of chord error, invisible
PER_SPOKE = 360 // SPOKES // STEP

GOLD = "#d9a441"
BRIGHT = "#fff6e0"
GROUND = "black"
INK = "#e8eef7"

PATH = (
    "Right View",
    "Right Intention",
    "Right Speech",
    "Right Action",
    "Right Livelihood",
    "Right Effort",
    "Right Mindfulness",
    "Right Concentration",
)


# --- the state --------------------------------------------------------------


class Wheel:
    def __init__(self):
        self.angle = 0.0

    def spoke_angle(self, index):
        return self.angle + index * (360 / SPOKES)

    def at_top(self):
        """Which spoke is closest to pointing straight up."""
        def from_top(index):
            offset = (self.spoke_angle(index) + 90.0) % 360.0
            return min(offset, 360.0 - offset)

        return min(range(SPOKES), key=from_top)


def on_circle(degrees, radius):
    radians = math.radians(degrees)
    return (CENTRE[0] + radius * math.cos(radians), CENTRE[1] + radius * math.sin(radians))


# --- the algorithm ----------------------------------------------------------


@cm.trace()
def turn(wheel):
    for step in range(SPOKES * PER_SPOKE):
        wheel.angle = (step + 1) * STEP
        cm.emit("turn")


# --- the view ---------------------------------------------------------------


def wheel_view(frame):
    scene = cm.Scene()
    wheel = frame.state
    leading = wheel.at_top()

    # The rim is a ring: a gold disc with the ground punched out of it.
    scene.circle("rim", x=CENTRE[0], y=CENTRE[1], r=RIM_OUTER, color=GOLD, layer=1)
    scene.circle("rim_hollow", x=CENTRE[0], y=CENTRE[1], r=RIM_INNER, color=GROUND, layer=2)

    for index in range(SPOKES):
        degrees = wheel.spoke_angle(index)
        here = index == leading
        scene.line(
            ("spoke", index),
            start=on_circle(degrees, HUB_OUTER - 4),
            end=on_circle(degrees, RIM_INNER + 4),
            w=SPOKE_W + (6.0 if here else 0.0),
            color=BRIGHT if here else GOLD,
            layer=3,
        )

    # A fixed marker on the rim at the top, so which spoke is being read is
    # unmistakable. It never moves — the wheel turns underneath it.
    scene.circle("marker", x=CENTRE[0], y=CENTRE[1] - (RIM_OUTER + RIM_INNER) / 2, r=10,
                 color=GROUND, layer=6)

    scene.circle("hub", x=CENTRE[0], y=CENTRE[1], r=HUB_OUTER, color=GOLD, layer=4)
    scene.circle("hub_hollow", x=CENTRE[0], y=CENTRE[1], r=HUB_INNER, color=GROUND, layer=5)

    scene.text("title", "Dharmachakra", x=CENTRE[0], y=92, size=38, color=INK)
    scene.text("subtitle", "the wheel of the Noble Eightfold Path",
               x=CENTRE[0], y=134, size=20, color="grey")
    scene.text("factor", PATH[leading], x=CENTRE[0], y=648, size=30, color=BRIGHT)

    return scene


cm.explain(
    trace=turn(Wheel()),
    view=wheel_view,
    timing=cm.Timing(default=0.22, opening=1.0, final_hold=1.6),
).render("results/dharma_wheel.mp4")

print("wrote results/dharma_wheel.mp4")

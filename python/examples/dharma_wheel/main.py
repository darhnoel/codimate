"""The Dharmachakra turning — the wheel of the Noble Eightfold Path.

    python python/examples/dharma_wheel/main.py

Eight spokes, one for each factor of the path. The wheel turns; the spoke
reaching the top brightens and its factor is named below, in Khmer.

There is no rotation in Codimate. There does not need to be: the trace says
where the wheel is every 15 degrees, and the Engine works out everything in
between — the same way it works out a bar sliding between two slots. Rotation
is just position over time.

Two things make it turn smoothly rather than judder: 15 degrees per event, so
a spoke tip barely leaves its arc, and the `linear` motion path, so it does
not ease to a stop inside every step.

    wheel.py    the drawing, which is most of the length
"""

import codimate as cm

import wheel

CENTRE = (640.0, 392.0)
SPOKES = 8
STEP = 15                      # degrees per event: 1.5px of chord error, invisible
PER_SPOKE = 360 // SPOKES // STEP

# Each factor is named while its spoke crosses the top, which takes PER_SPOKE
# events — so the per-event duration follows from how long you want to read it.
FACTOR_SECONDS = 1.5
STEP_SECONDS = FACTOR_SECONDS / PER_SPOKE

# The eight factors, in the Pali terms as they are written in Khmer.
PATH = (
    "សម្មាទិដ្ឋិ",      # right view
    "សម្មាសង្កប្បៈ",     # right intention
    "សម្មាវាចា",        # right speech
    "សម្មាកម្មន្តៈ",     # right action
    "សម្មាអាជីវៈ",      # right livelihood
    "សម្មាវាយាមៈ",      # right effort
    "សម្មាសតិ",         # right mindfulness
    "សម្មាសមាធិ",       # right concentration
)


# --- the state --------------------------------------------------------------


class Wheel:
    def __init__(self):
        self.angle = 0.0

    def at_top(self):
        """Which spoke is closest to pointing straight up."""
        def from_top(index):
            offset = (self.angle + index * (360 / SPOKES) + 90.0) % 360.0
            return min(offset, 360.0 - offset)

        return min(range(SPOKES), key=from_top)


# --- the algorithm ----------------------------------------------------------


@cm.trace()
def turn(wheel_state):
    for step in range(SPOKES * PER_SPOKE):
        wheel_state.angle = (step + 1) * STEP
        cm.emit("turn")


# --- the view ---------------------------------------------------------------


def wheel_view(frame):
    scene = cm.Scene()
    state = frame.state
    leading = state.at_top()

    wheel.draw(scene, CENTRE, state.angle, leading, SPOKES)

    scene.text("title", "ធម្មចក្រ", x=CENTRE[0], y=72, size=44, color="#e8eef7")
    scene.text("subtitle", "កង់នៃអដ្ឋង្គិកមគ្គ",
               x=CENTRE[0], y=122, size=24, color="grey")
    scene.text("factor", PATH[leading], x=CENTRE[0], y=674, size=38, color=wheel.GOLD_LIT)

    return scene


# `linear`, not the default `straight`: the wheel is mid-turn at every event,
# so easing would make it accelerate and stop 24 times a revolution.
cm.explain(
    trace=turn(Wheel()),
    view=wheel_view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=STEP_SECONDS, opening=1.2, final_hold=2.0),
).render("results/dharma_wheel.mp4")

print("wrote results/dharma_wheel.mp4")

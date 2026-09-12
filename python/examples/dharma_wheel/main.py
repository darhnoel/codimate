"""The Dharmachakra turning — the wheel of the Noble Eightfold Path.

    python python/examples/dharma_wheel/main.py

Eight spokes, one for each factor of the path. The wheel turns evenly, and the
factor belonging to the spoke at the top is named below, in Khmer. The spokes
themselves are all alike — the wheel is a symbol, not a chart.

There is no rotation in Codimate. There does not need to be: the trace says
where the wheel is every 15 degrees, and the Engine works out everything in
between — the same way it works out a bar sliding between two slots. Rotation
is just position over time.

Two things make it turn smoothly rather than judder: 15 degrees per event, so
a spoke tip barely leaves its arc, and the `linear` motion path, so it does
not ease to a stop inside every step.

    wheel.py    the drawing, which is most of the length

Rendered at 1080p60: `scale=1.5` draws every frame at 1920x1080 rather than
upscaling a 720p one, and coordinates still mean what `cm.canvas()` says.
"""

import codimate as cm

import wheel

CENTRE = (640.0, 372.0)
SPOKES = 8
STEP = 15                      # degrees per event: 1.5px of chord error, invisible
PER_SPOKE = 360 // SPOKES // STEP

# Each factor is named while its spoke crosses the top, which takes PER_SPOKE
# events — so the per-event duration follows from how long you want to read it.
FACTOR_SECONDS = 1.5
STEP_SECONDS = FACTOR_SECONDS / PER_SPOKE

# The eight factors: the Pali term as written in Khmer, its plain-Khmer
# meaning, and which of the three trainings (ត្រៃសិក្ខា) it belongs to.
WISDOM, MORALITY, CONCENTRATION = "ក្រុមបញ្ញា", "ក្រុមសីល", "ក្រុមសមាធិ"

PATH = (
    ("សម្មាទិដ្ឋិ", "ការយល់ឃើញត្រូវ", WISDOM),            # right view
    ("សម្មាសង្កប្បៈ", "ការត្រិះរិះត្រូវ", WISDOM),          # right intention
    ("សម្មាវាចា", "ការពោលស្តីត្រូវ", MORALITY),           # right speech
    ("សម្មាកម្មន្តៈ", "ការងារត្រូវ", MORALITY),            # right action
    ("សម្មាអាជីវៈ", "ការចិញ្ចឹមជីវិតត្រូវ", MORALITY),      # right livelihood
    ("សម្មាវាយាមៈ", "ការព្យាយាមត្រូវ", CONCENTRATION),     # right effort
    ("សម្មាសតិ", "ការរលឹកត្រូវ", CONCENTRATION),          # right mindfulness
    ("សម្មាសមាធិ", "ការតម្កល់ចិត្តត្រូវ", CONCENTRATION),   # right concentration
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

    wheel.draw(scene, CENTRE, state.angle, SPOKES)

    scene.text("title", "ធម្មចក្រ", x=CENTRE[0], y=62, size=44, color="#e8eef7")
    scene.text("subtitle", "អរិយអដ្ឋង្គិកមគ្គ",
               x=CENTRE[0], y=110, size=24, color="grey")

    factor, meaning, _training = PATH[leading]
    scene.text("factor", factor, x=CENTRE[0], y=628, size=36, color=wheel.GOLD_LIT)
    scene.text("meaning", meaning, x=CENTRE[0], y=672, size=24, color="grey")

    return scene


# `linear`, not the default `straight`: the wheel is mid-turn at every event,
# so easing would make it accelerate and stop 24 times a revolution.
cm.explain(
    trace=turn(Wheel()),
    view=wheel_view,
    motion=[cm.Rule("*", position="linear")],
    timing=cm.Timing(default=STEP_SECONDS, opening=1.2, final_hold=2.0),
).render("results/dharma_wheel.mp4", fps=60, scale=1.5)

print("wrote results/dharma_wheel.mp4")

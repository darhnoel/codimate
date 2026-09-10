"""Codimate explaining the maths behind Codimate.

    python python/examples/how_codimate_works.py

The bars on the right are three moments of a bubble sort: compare 3 and 1,
swap them, compare 3 and 4. Codimate stores only those three pictures. Every
frame in between is computed:

        local = (t - start) / duration
        value = before + (after - before) * ease(local)

The playhead sweeping the timeline is itself moved by that formula. This trace
records 28 instants; everything you see between them is worked out.
"""

import codimate as cm

# The three moments being explained, each a snapshot of a bubble sort:
# the values in order, and the pair the event is about.
MOMENTS = (
    ((3, 1, 4, 2), (), "start"),
    ((3, 1, 4, 2), (3, 1), "compare"),
    ((1, 3, 4, 2), (1, 3), "swap"),
    ((1, 3, 4, 2), (3, 4), "compare"),
)
SEGMENTS = tuple((MOMENTS[i + 1][2], d) for i, d in enumerate((0.6, 0.9, 0.6)))
TOTAL = sum(d for _, d in SEGMENTS)
TICKS = 28

TRACK = cm.Slot(x=640.0, y=205.0, w=940.0, h=54.0)
CURVE_L, CURVE_R, CURVE_TOP, CURVE_BOT = 130.0, 400.0, 400.0, 620.0
CHART = cm.Slot(x=850.0, y=490.0, w=560.0, h=340.0)

INK = "#e8eef7"
DIM = "#2b3648"
GHOST = "#38455c"
LIVE = "orange"
ACCENT = "#4a9eff"


def ease_in_out(u):
    """The same curve the Engine uses (crates/codimate-py/src/lib.rs)."""
    return 2 * u * u if u < 0.5 else 1 - (-2 * u + 2) ** 2 / 2


def mix(a, b, u):
    return a + (b - a) * u


# --- the state --------------------------------------------------------------


class Clock:
    """A playhead somewhere on a timeline of segments."""

    def __init__(self):
        self.t = 0.0

    def locate(self):
        """Which segment holds t, and how far through it?

        This is what `Explanation::resolve` does in the Engine, written out.
        """
        start = 0.0
        for index, (_, duration) in enumerate(SEGMENTS):
            if self.t < start + duration or index == len(SEGMENTS) - 1:
                return index, start, duration, (self.t - start) / duration
            start += duration
        raise AssertionError("unreachable")


@cm.trace()
def sweep(clock):
    for tick in range(TICKS):
        clock.t = TOTAL * tick / (TICKS - 1)
        cm.emit("tick")


# --- the view ---------------------------------------------------------------


def track_x(t):
    return TRACK.left + TRACK.w * (t / TOTAL)


def timeline(scene, clock, active):
    start = 0.0
    for index, (name, duration) in enumerate(SEGMENTS):
        left, right = track_x(start), track_x(start + duration)
        here = index == active
        band = scene.group(("segment", index), x=(left + right) / 2, y=TRACK.y)
        band.rect("box", w=right - left - 4, h=TRACK.h, color=ACCENT if here else DIM)
        band.text("name", name, size=22, color=INK if here else "grey")
        band.text("span", f"{start:.1f}s", top=TRACK.h / 2 + 12, size=17, color="grey")
        start += duration

    scene.text("end", f"{TOTAL:.1f}s", x=track_x(TOTAL), top=TRACK.bottom + 12,
               size=17, color="grey")

    # One group, so the stem and its reading can never drift apart.
    head = scene.group("playhead", x=track_x(clock.t), y=TRACK.y)
    head.line("stem", start=(0, -TRACK.h / 2 - 12), end=(0, TRACK.h / 2 + 4), w=3.0, color=LIVE)
    head.text("t", f"t = {clock.t:.2f}s", bottom=-TRACK.h / 2 - 18, size=24, color=LIVE)


def easing_curve(scene, local):
    def at(u):
        return (mix(CURVE_L, CURVE_R, u), mix(CURVE_BOT, CURVE_TOP, ease_in_out(u)))

    scene.line("axis_x", start=(CURVE_L, CURVE_BOT), end=(CURVE_R, CURVE_BOT), w=1.0, color=DIM)
    scene.line("axis_y", start=(CURVE_L, CURVE_BOT), end=(CURVE_L, CURVE_TOP), w=1.0, color=DIM)
    for i in range(24):
        scene.line(("curve", i), start=at(i / 24), end=at((i + 1) / 24), w=2.5, color=GHOST)

    x, y = at(local)
    scene.circle("rider", x=x, y=y, r=8, color=LIVE, layer=9)
    scene.text("curve_label", "ease(local)", x=(CURVE_L + CURVE_R) / 2,
               top=CURVE_BOT + 16, size=20, color="grey")


def bars(scene, active, local):
    """The two stored pictures, and the one computed between them.

    Doing by hand, for four bars, exactly what the Engine does for every shape.
    """
    before_order, _, _ = MOMENTS[active]
    after_order, highlight, _ = MOMENTS[active + 1]

    places = {}
    for index, slot in enumerate(cm.row(4, gap=26, within=CHART)):
        places[index] = slot

    eased = ease_in_out(local)

    scene.text("chart_label", "computed, not stored", x=CHART.x, y=CHART.bottom - 6,
               size=20, color="grey")

    for value in after_order:
        here = places[before_order.index(value)]
        there = places[after_order.index(value)]

        bar = scene.group(("bar", value), x=mix(here.x, there.x, eased), y=here.bottom)
        bar.rect("box", w=here.w, h=value * 46,
                 bottom=0, color=LIVE if value in highlight else ACCENT)
        bar.text("v", value, top=16, size=26, color=INK)


def explain_frame(frame):
    scene = cm.Scene()
    clock = frame.state
    active, start, duration, local = clock.locate()

    scene.text("title", "f(t) → Scene", x=cm.width() / 2, y=72, size=44, color=INK)
    scene.text("sub", "three pictures are stored — every frame between them is computed",
               x=cm.width() / 2, y=112, size=21, color="grey")

    timeline(scene, clock, active)

    scene.text("formula",
               f"local = (t − {start:.1f}) / {duration:.1f}  =  {local:.2f}",
               x=cm.width() / 2, y=310, size=26, color=INK)

    easing_curve(scene, local)
    bars(scene, active, local)
    return scene


cm.explain(
    trace=sweep(Clock()),
    view=explain_frame,
    timing=cm.Timing(default=0.13, opening=1.4, final_hold=2.0),
).render("results/how_codimate_works.mp4")

print("wrote results/how_codimate_works.mp4")

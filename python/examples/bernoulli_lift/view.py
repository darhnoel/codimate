"""Drawing the wing, the air, and the pair that settles the argument."""

from airfoil import fill_columns, turned_by
from scene_setup import (DIM, INK, OVER, SCALE, UNDER, WING, WING_EDGE,
                          heat, place)

COLUMNS = fill_columns(140)
COLUMN_W = (COLUMNS[-1][0] - COLUMNS[0][0]) / len(COLUMNS) * SCALE + 1.6


def _wing(scene):
    """Filled a column at a time, because there is no polygon to fill."""
    for i, (x, top, bottom) in enumerate(COLUMNS):
        scene.line(("wing", i), start=place(x, top), end=place(x, bottom),
                   w=COLUMN_W, color=WING, layer=4)
    for i, (x, top, bottom) in enumerate(COLUMNS[:-1]):
        nx, ntop, nbottom = COLUMNS[i + 1]
        scene.line(("edge_top", i), start=place(x, top), end=place(nx, ntop),
                   w=2.0, color=WING_EDGE, layer=5)
        scene.line(("edge_bot", i), start=place(x, bottom), end=place(nx, nbottom),
                   w=2.0, color=WING_EDGE, layer=5)


def _labels(scene, flow):
    scene.text("title", "Bernoulli's principle — and the story that goes with it",
               x=640, y=48, size=30, color=INK)
    # Laid out piece by piece. As one centred string the swatches, which are
    # placed at fixed x, landed on top of the word they were labelling.
    # The font runs about 12.1px per character at this size: "colour is speed"
    # is 182px wide, not the 135 first guessed, which is how it came to sit on
    # top of "slow".
    scene.text("legend_what", "colour is speed", x=100, y=676, size=17, color=DIM)
    scene.text("legend_slow", "slow", x=225, y=676, size=17, color=DIM)
    for i in range(16):
        scene.line(("scale", i), start=(262 + i * 9, 676), end=(270 + i * 9, 676),
                   w=9.0, color=heat(0.55 + i * 0.06))
    scene.text("legend_fast", "fast", x=440, y=676, size=17, color=DIM)

    over, under = flow.arrived.get("over"), flow.arrived.get("under")
    if flow.pair_released:
        scene.text("claim",
                   "these two split at the nose — the old story says they meet again",
                   x=640, y=92, size=19, color=DIM)
    if over:
        scene.text("over_time", f"over:  {over:.2f}", x=1040, y=150,
                   size=22, color=OVER)
    if under:
        scene.text("under_time", f"under: {under:.2f}", x=1040, y=182,
                   size=22, color=UNDER)
    if over and under:
        scene.text("verdict", f"the upper one arrived {under / over:.2f}x sooner",
                   x=1040, y=222, size=20, color=INK)
        scene.text("verdict2", "so they never met — the story is wrong",
                   x=1040, y=250, size=18, color=DIM)

        # The half the familiar diagram leaves out.
        scene.text("newton",
                   f"and the wing turns the air {abs(turned_by()):.0f}\u00b0 downwards",
                   x=640, y=606, size=20, color=INK)
        scene.text("newton2",
                   "it throws air down, the air throws it up — no path lengths needed",
                   x=640, y=634, size=17, color=DIM)


def draw(scene, flow):
    _wing(scene)

    for parcel in flow.parcels:
        if parcel.marked:
            continue
        scene.circle(("air", parcel.id), x=place(parcel.x, parcel.y)[0],
                     y=place(parcel.x, parcel.y)[1], r=3.1,
                     color=heat(parcel.speed), layer=8)

    # The marked pair, with the paths they took.
    for parcel in flow.parcels:
        if not parcel.marked:
            continue
        color = OVER if parcel.side == "over" else UNDER
        for i, ((ax, ay), (bx, by)) in enumerate(zip(parcel.trail, parcel.trail[1:])):
            scene.line(("path", parcel.side, i), start=place(ax, ay), end=place(bx, by),
                       w=2.2, color=color, opacity=0.55, layer=9)
        px, py = place(parcel.x, parcel.y)
        scene.circle(("mark", parcel.side), x=px, y=py, r=7.5, color=color, layer=10)

    _labels(scene, flow)

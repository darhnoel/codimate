"""Drawing the wing, the air, and the pair that settles the argument."""

from airfoil import fill_columns, turned_by
from scene_setup import (DIM, INK, OVER, SCALE, UNDER, WING, WING_EDGE,
                          heat, place)

COLUMNS = fill_columns(140)
COLUMN_W = (COLUMNS[-1][0] - COLUMNS[0][0]) / len(COLUMNS) * SCALE + 1.6


def _wing(scene):
    """One closed curve: over the top, back along the bottom.

    This used to be 140 vertical lines, one per column, because nothing could
    fill a closed outline — and then 278 more to draw the edges. An aerofoil
    is a smooth shape, which is exactly what `curve` is for.
    """
    outline = ([place(x, top) for x, top, _ in COLUMNS]
               + [place(x, bottom) for x, _, bottom in reversed(COLUMNS)])
    scene.curve("wing", outline, w=2.0, closed=True) \
         .fill(WING, edge=WING_EDGE, edge_w=2.0).on(layer=4)


def _labels(scene, flow):
    scene.text("title", "Bernoulli's principle", size=32, at=(640, 48)).fill(INK)
    # Laid out piece by piece. As one centred string the swatches, which are
    # placed at fixed x, landed on top of the word they were labelling.
    # The font runs about 12.1px per character at this size: "colour is speed"
    # is 182px wide, not the 135 first guessed, which is how it came to sit on
    # top of "slow".
    scene.text("legend_what", "colour is speed", size=17, at=(100, 676)).fill(DIM)
    scene.text("legend_slow", "slow", size=17, at=(225, 676)).fill(DIM)
    for i in range(16):
        scene.line(("scale", i), start=(262 + i * 9, 676), end=(270 + i * 9, 676),
                   w=9.0).fill(heat(0.55 + i * 0.06))
    scene.text("legend_fast", "fast", size=17, at=(440, 676)).fill(DIM)

    over, under = flow.arrived.get("over"), flow.arrived.get("under")
    if flow.pair_released:
        scene.text("claim", "two parcels released together, one over and one under",
                   size=19, at=(640, 92)).fill(DIM)
    if over:
        scene.text("over_time", f"over:  {over:.2f}", size=22,
                   at=(1040, 150)).fill(OVER)
    if under:
        scene.text("under_time", f"under: {under:.2f}", size=22,
                   at=(1040, 182)).fill(UNDER)
    if over and under:
        scene.text("verdict", f"the upper one arrived {under / over:.2f}x sooner",
                   size=20, at=(1040, 222)).fill(INK)
        scene.text("newton",
                   f"the wing turns the air {abs(turned_by()):.0f}\u00b0 downwards",
                   size=20, at=(640, 606)).fill(INK)
        scene.text("newton2", "it throws air down, the air throws it up", size=17,
                   at=(640, 634)).fill(DIM)


def draw(scene, flow):
    _wing(scene)

    for parcel in flow.parcels:
        if parcel.marked:
            continue
        scene.circle(("air", parcel.id), r=3.1,
                     at=place(parcel.x, parcel.y)).fill(heat(parcel.speed)).on(layer=8)

    # The marked pair, with the paths they took.
    for parcel in flow.parcels:
        if not parcel.marked:
            continue
        color = OVER if parcel.side == "over" else UNDER
        if len(parcel.trail) >= 2:
            scene.curve(("path", parcel.side),
                        [place(px, py) for px, py in parcel.trail],
                        w=2.2).fill(color).on(opacity=0.55, layer=9)
        px, py = place(parcel.x, parcel.y)
        scene.circle(("mark", parcel.side), r=7.5, at=(px, py)).fill(color).on(layer=10)

    _labels(scene, flow)

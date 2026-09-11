"""Drawing the wheel: rim, finials, spokes, hub.

Everything here is a filled circle or a thick line. Two tricks do the work:

* **Rings come from discs.** A circle can only be filled, so a ring is a large
  disc with a smaller one of the background colour on top. Stack four and you
  get a gold band with a dark edge on both sides.
* **A thick line is a rotated rectangle.** `line(start, end, w=24)` strokes a
  path, so a short span at a large width draws a block at any angle — which is
  how the spokes taper and how the ornaments sit square to the spoke.
"""

import math

EDGE = "#8a5a12"
GOLD = "#f2c230"
GOLD_DEEP = "#d9a020"
GOLD_LIT = "#fff1c4"
GROUND = "black"

RIM_OUT, RIM_IN = 167.0, 140.0
FINIAL_R = 185.0
DOT_R = 5.0
HUB_OUT, HUB_IN = 49.0, 33.0
SPOKE_W, SPOKE_EDGE = 12.0, 18.5
ORNAMENT_IN, ORNAMENT_OUT = 86.0, 113.0

_E = 4.0     # how far the dark edge shows beyond a gold shape


def polar(centre, degrees, radius):
    radians = math.radians(degrees)
    return (centre[0] + radius * math.cos(radians), centre[1] + radius * math.sin(radians))


# One tall centre lobe with two smaller ones tucked beside it, so a finial
# reads as a bud rather than as three beads in a row.
_LOBES = ((0.0, 13.0, 16.0), (-10.5, -4.5, 10.0), (10.5, -4.5, 10.0))


def _finials(scene, centre, angle, spokes):
    """A three-lobed bud outside the rim at every spoke."""
    for index in range(spokes):
        base = angle + index * (360 / spokes)
        for lobe, (offset, reach, size) in enumerate(_LOBES):
            x, y = polar(centre, base + offset, FINIAL_R + reach)
            scene.circle(("finial", index, lobe, "edge"), x=x, y=y,
                         r=size + _E, color=EDGE, layer=1)
            scene.circle(("finial", index, lobe), x=x, y=y,
                         r=size, color=GOLD, layer=2)


def _rim(scene, centre):
    """Four discs: dark, gold, dark, ground — a banded ring."""
    for name, radius, color, layer in (
        ("edge_out", RIM_OUT + _E, EDGE, 3),
        ("band", RIM_OUT, GOLD, 4),
        ("edge_in", RIM_IN + _E, EDGE, 5),
        ("hollow", RIM_IN, GROUND, 6),
    ):
        scene.circle(("rim", name), x=centre[0], y=centre[1], r=radius, color=color, layer=layer)


def _spokes(scene, centre, angle, spokes):
    for index in range(spokes):
        base = angle + index * (360 / spokes)
        # Tapered: a broad shaft from the hub, a narrower one out to the rim.
        hub_end = polar(centre, base, HUB_OUT - 6)
        waist = polar(centre, base, ORNAMENT_OUT - 6)
        rim_end = polar(centre, base, RIM_IN + 6)

        for part, (a, b, wide) in enumerate(((hub_end, waist, SPOKE_W + 6.0),
                                             (waist, rim_end, SPOKE_W - 3.0))):
            scene.line(("spoke", index, part, "edge"), start=a, end=b,
                       w=wide + (SPOKE_EDGE - SPOKE_W), color=EDGE, layer=7)
            scene.line(("spoke", index, part), start=a, end=b,
                       w=wide, color=GOLD, layer=8)

        # The diamond block partway along — a short, very thick line.
        a, b = polar(centre, base, ORNAMENT_IN), polar(centre, base, ORNAMENT_OUT)
        scene.line(("ornament", index, "edge"), start=a, end=b, w=24.0, color=EDGE, layer=9)
        scene.line(("ornament", index), start=a, end=b, w=17.5,
                   color=GOLD_DEEP, layer=10)


def _studs(scene, centre, angle, spokes):
    """The ring of dots on the rim — one per spoke, turning with the wheel."""
    band = (RIM_OUT + RIM_IN) / 2
    for index in range(spokes):
        x, y = polar(centre, angle + index * (360 / spokes), band)
        scene.circle(("stud", index, "edge"), x=x, y=y, r=DOT_R + 2.5, color=EDGE, layer=11)
        scene.circle(("stud", index), x=x, y=y, r=DOT_R, color=GOLD_DEEP, layer=12)


def _hub(scene, centre, angle):
    for name, radius, color, layer in (
        ("edge", HUB_OUT + _E, EDGE, 13),
        ("face", HUB_OUT, GOLD, 14),
        ("inner_edge", HUB_IN + _E, EDGE, 15),
        ("inner", HUB_IN, GOLD_DEEP, 16),
    ):
        scene.circle(("hub", name), x=centre[0], y=centre[1], r=radius, color=color, layer=layer)

    # Three lobes turning together — a triskelion, as far as circles allow.
    for lobe in range(3):
        x, y = polar(centre, angle * 1.5 + lobe * 120, 13.0)
        scene.circle(("swirl", lobe, "edge"), x=x, y=y, r=12.5, color=EDGE, layer=17)
        scene.circle(("swirl", lobe), x=x, y=y, r=9.5, color=GOLD, layer=18)


def draw(scene, centre, angle, spokes):
    _finials(scene, centre, angle, spokes)
    _rim(scene, centre)
    _spokes(scene, centre, angle, spokes)
    _studs(scene, centre, angle, spokes)
    _hub(scene, centre, angle)

"""Scene 3. Faraday, 1831: a magnet pushed into a coil moves a meter needle."""

import math

from palette import INK, DIM, TAG, B_COL, COPPER, STEEL, PANEL, RED, NORTH, SOUTH, W
from pen import Look, clamp, ramp
from geometry import bezier, head_at
from equations import FARADAY


def magnet_x(u):
    """Where the magnet's centre is: in, held still, then out again."""
    moved = ramp(u, 0.12, 0.4) - ramp(u, 0.6, 0.88)
    return 190 + 420 * moved


COIL_X = 635  # the middle of the coil


def flux(mid):
    """How much of the magnet's field threads the coil: next to nothing while it
    is far off, most of it once it is inside."""
    return math.exp(-(((mid - COIL_X) / 120) ** 2))


def flux_rate(u):
    """How fast that is changing. A current is made by the change, not by the
    magnet moving: far from the coil the magnet can move all it likes."""
    h = 0.002
    return (flux(magnet_x(u + h)) - flux(magnet_x(u - h))) / (2 * h)


PEAK = max(abs(flux_rate(i / 400)) for i in range(401))


def draw(s, u, t, D):
    mid = magnet_x(u)
    for i in range(6):
        s.ring(
            ("loop", i), (580 + i * 22, 400), (16, 62), Look(color=COPPER, w=5, layer=3)
        )
    for name, a, b in (
        ("top-1", (580, 338), (580, 280)),
        ("top-2", (580, 280), (1010, 280)),
        ("top-3", (1010, 280), (1010, 345)),
        ("bot-1", (690, 462), (690, 540)),
        ("bot-2", (690, 540), (1010, 540)),
        ("bot-3", (1010, 540), (1010, 495)),
    ):
        s.line(name, a, b, Look(color=COPPER, w=5, layer=-1))
    # the magnet's own field, carried along with it: out of N, round, into S
    north, south = (mid + 100, 400), (mid - 100, 400)
    for k in (1, 2):
        for side in (-1, 1):
            rise = 50 * k * side
            pts = bezier(
                north,
                (north[0] + 60 * k, 400 + rise),
                (south[0] - 60 * k, 400 + rise),
                south,
            )
            look = Look(color=B_COL, w=2.5, op=0.9 - 0.25 * k, layer=1)
            s.curve(("field", k, side), pts, look)
            here, ahead = pts[20], pts[21]
            dx, dy = ahead[0] - here[0], ahead[1] - here[1]
            span = math.hypot(dx, dy)
            s.poly(
                ("field-head", k, side),
                head_at(here, (dx / span, dy / span)),
                Look(color=B_COL, op=look.op, layer=1),
            )
    s.rect("south", (mid - 50, 400), (100, 58), Look(color=SOUTH, layer=1))
    s.rect("north", (mid + 50, 400), (100, 58), Look(color=NORTH, layer=1))
    s.rect("magnet", (mid, 400), (200, 58), Look(edge=INK, w=3, layer=2)).round(4)
    s.text("S", "S", (mid - 50, 400), Look(size=34, layer=5))
    s.text("N", "N", (mid + 50, 400), Look(size=34, layer=5))
    # a galvanometer: a dial with a scale and a pivoting needle. The needle
    # follows how fast the flux through the coil is changing.
    tilt = math.radians(clamp(flux_rate(u) / PEAK, -1, 1) * 45)
    face, pivot = (1010, 420), (1010, 470)
    s.circle("meter", face, 84, Look(color=PANEL))
    s.circle("meter-rim", face, 84, Look(edge=STEEL, w=4, layer=1))
    for i, deg in enumerate(range(-45, 46, 15)):
        a = math.radians(deg)
        centre = deg == 0
        s.line(
            ("tick", i),
            (pivot[0] + 64 * math.sin(a), pivot[1] - 64 * math.cos(a)),
            (pivot[0] + 78 * math.sin(a), pivot[1] - 78 * math.cos(a)),
            Look(color=INK if centre else DIM, w=3 if centre else 2, layer=1),
        )
    s.text("meter-name", "G", (1010, 368), Look(color=DIM, size=28, layer=1))
    s.line(
        "needle",
        pivot,
        (pivot[0] + 72 * math.sin(tilt), pivot[1] - 72 * math.cos(tilt)),
        Look(color=RED, w=4, layer=2),
    )
    s.circle("pivot", pivot, 8, Look(color=STEEL, layer=3))
    s.formula("eq", FARADAY, (W / 2, 150), Look(size=76, reveal=ramp(u, 0.3, 0.52)))
    s.text("tag", "ផារ៉ាដេ", (W / 2, 44), Look(color=TAG, size=46))

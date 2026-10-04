"""Scene 4. Faraday's picture: field lines fill space and never end."""

import math

from palette import INK, DIM, TAG, B_COL, NORTH, SOUTH, W
from pen import Look, ramp
from geometry import bezier, head_at
from equations import NO_MONOPOLE


def draw(s, u, t, D):
    north, south = (490, 360), (790, 360)
    s.rect("south", (715, 360), (150, 64), Look(color=SOUTH, layer=2))
    s.rect("north", (565, 360), (150, 64), Look(color=NORTH, layer=2))
    s.rect("magnet", (640, 360), (300, 64), Look(edge=INK, w=3, layer=4)).round(5)
    s.text("S", "S", (715, 360), Look(size=38, layer=5))
    s.text("N", "N", (565, 360), Look(size=38, layer=5))
    for k in range(1, 5):
        on = ramp(u, 0.05 + 0.08 * k, 0.15 + 0.08 * k)
        for side in (-1, 1):
            rise = 55 * k * side
            pts = bezier(
                north,
                (north[0] - 60 * k, north[1] + rise),
                (south[0] + 60 * k, south[1] + rise),
                south,
            )
            s.curve(
                ("line", k, side),
                pts,
                Look(color=B_COL, w=2.5, op=on * (1.1 - 0.15 * k), layer=1),
            )
            mid, ahead = pts[20], pts[21]
            dx, dy = ahead[0] - mid[0], ahead[1] - mid[1]
            span = math.hypot(dx, dy)
            s.poly(
                ("head", k, side),
                head_at(mid, (dx / span, dy / span)),
                Look(color=B_COL, op=on, layer=1),
            )
    s.formula("eq", NO_MONOPOLE, (W / 2, 150), Look(size=84, reveal=ramp(u, 0.5, 0.7)))
    s.text("tag", "ផារ៉ាដេ", (W / 2, 56), Look(color=TAG, size=48))
    s.text(
        "note",
        "ខ្សែដែនមិនមានទីបញ្ចប់",
        (W / 2, 590),
        Look(color=DIM, size=32, op=ramp(u, 0.72, 0.85)),
    )

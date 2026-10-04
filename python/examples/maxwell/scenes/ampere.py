"""Scene 2. Ørsted and Ampère, 1820: a current moves a compass needle."""

import math

from palette import INK, TAG, B_COL, COPPER, STEEL, PANEL, RED
from pen import Look, ramp
from geometry import head_at
from equations import AMPERE


def draw(s, u, t, D):
    wire, compass = (380, 390), (380, 190)
    on = ramp(u, 0.22, 0.34)
    for i, radius in enumerate((70, 130, 200)):
        s.ring(
            ("field", i), wire, (radius, radius), Look(color=B_COL, w=2.5, op=on * 0.7)
        )
        for j, angle in enumerate((45, 135, 225, 315)):
            a = math.radians(angle)
            p = (wire[0] + radius * math.cos(a), wire[1] - radius * math.sin(a))
            # counter-clockwise on screen: current is coming out of the page
            s.poly(
                ("arrow", i, j),
                head_at(p, (-math.sin(a), -math.cos(a))),
                Look(color=B_COL, op=on),
            )
    s.circle("wire", wire, 30, Look(color=COPPER))
    s.circle("current", wire, 8, Look(color=INK, op=on, layer=1))
    s.text("I", "I", (wire[0] + 22, wire[1] + 62), Look(color=INK, size=34, op=on))
    # the needle starts pointing up ("north"), then swings to follow the field
    since = max(0.0, u - 0.22) * D
    swing = math.radians(90) * (1 - math.exp(-5 * since) * math.cos(9 * since))
    d = (-math.sin(swing), -math.cos(swing))
    n = (-d[1], d[0])
    s.circle("compass", compass, 44, Look(color=PANEL, layer=1))
    s.circle("rim", compass, 44, Look(edge=STEEL, w=3, layer=2))
    for name, sign, color in (("tip", 1, RED), ("tail", -1, INK)):
        end = (compass[0] + sign * d[0] * 38, compass[1] + sign * d[1] * 38)
        s.poly(
            name,
            [
                end,
                (compass[0] + n[0] * 7, compass[1] + n[1] * 7),
                (compass[0] - n[0] * 7, compass[1] - n[1] * 7),
            ],
            Look(color=color, layer=3),
        )
    s.text("tag", "អឺស្ទែត · អំពែរ", (930, 240), Look(color=TAG, size=48))
    s.formula("eq", AMPERE, (930, 345), Look(size=76, reveal=ramp(u, 0.5, 0.7)))

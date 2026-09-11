"""Where the three panels sit, and what colour everything is.

Shared by every panel, so no panel has to import another.
"""

import codimate as cm

TRACK = cm.Slot(x=640.0, y=205.0, w=940.0, h=54.0)
CURVE_L, CURVE_R, CURVE_TOP, CURVE_BOT = 130.0, 400.0, 400.0, 620.0
CHART = cm.Slot(x=850.0, y=490.0, w=560.0, h=340.0)

INK = "#e8eef7"
DIM = "#2b3648"
GHOST = "#38455c"
LIVE = "orange"
ACCENT = "#4a9eff"


def mix(a, b, u):
    return a + (b - a) * u

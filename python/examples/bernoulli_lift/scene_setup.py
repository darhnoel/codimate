"""Where the flow sits on the screen, and what everything looks like."""

import math

from airfoil import ALPHA

# The maths puts the angle of attack into the free stream and leaves the wing
# level, which draws air arriving uphill. Rotating the whole picture back by
# the same angle is the view everyone actually means: the air comes in
# horizontally and the wing sits nose-up in it. It changes nothing physical —
# only which of the two you are standing still relative to.
_C, _S = math.cos(-ALPHA), math.sin(-ALPHA)

SCALE = 148.0
ORIGIN = (616.0, 402.0)          # where (0, 0) of the airfoil plane lands

WING = "#525c6e"
WING_EDGE = "#7b869b"
INK = "#e8eef7"
DIM = "#7c8799"

FAST = (0xff, 0xb0, 0x3a)        # what a quick parcel is coloured
SLOW = (0x3f, 0x74, 0xd0)        # and a slow one
SPEED_LO, SPEED_HI = 0.55, 1.45

OVER, UNDER = "#4ade80", "#f472b6"   # the marked pair


def place(x, y):
    """Airfoil-plane coordinates onto the screen, rotated so the air comes in
    level. Screen y grows downwards."""
    rx = x * _C - y * _S
    ry = x * _S + y * _C
    return (ORIGIN[0] + rx * SCALE, ORIGIN[1] - ry * SCALE)


def heat(speed):
    """Colour by speed, which is the quantity Bernoulli is actually about."""
    t = (speed - SPEED_LO) / (SPEED_HI - SPEED_LO)
    t = 0.0 if t < 0.0 else 1.0 if t > 1.0 else t
    return "#%02x%02x%02x" % tuple(
        round(SLOW[i] + (FAST[i] - SLOW[i]) * t) for i in range(3))

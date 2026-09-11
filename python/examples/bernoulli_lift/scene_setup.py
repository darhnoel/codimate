"""Where the flow sits on the screen, and what everything looks like."""

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
    """Airfoil-plane coordinates onto the screen. Screen y grows downwards."""
    return (ORIGIN[0] + x * SCALE, ORIGIN[1] - y * SCALE)


def heat(speed):
    """Colour by speed, which is the quantity Bernoulli is actually about."""
    t = (speed - SPEED_LO) / (SPEED_HI - SPEED_LO)
    t = 0.0 if t < 0.0 else 1.0 if t > 1.0 else t
    return "#%02x%02x%02x" % tuple(
        round(SLOW[i] + (FAST[i] - SLOW[i]) * t) for i in range(3))

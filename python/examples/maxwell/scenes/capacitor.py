"""A charging capacitor, and the ring and bar that probe it. Scenes 5 and 6 share
it: the same picture, with Maxwell's term off and then on."""

from palette import DIM, E_COL, B_COL, COPPER, STEEL, YELLOW
from pen import Look, clamp, ramp


PLATES = (600, 680)
WIRE_Y = 400


def capacitor(s, t):
    """A charging capacitor: wire in, two plates, wire out, and charge flowing."""
    s.line("wire-in", (150, WIRE_Y), (PLATES[0] - 7, WIRE_Y), Look(color=COPPER, w=5))
    s.line("wire-out", (PLATES[1] + 7, WIRE_Y), (1130, WIRE_Y), Look(color=COPPER, w=5))
    for i, x in enumerate(PLATES):
        s.rect(("plate", i), (x, WIRE_Y), (14, 200), Look(color=STEEL, layer=2))
    for i in range(8):
        for name, a, b in (("in", 150, PLATES[0] - 14), ("out", PLATES[1] + 14, 1130)):
            x = a + ((i / 8 + t * 0.1) % 1) * (b - a)
            s.circle(("charge", name, i), (x, WIRE_Y), 6, Look(color=YELLOW, layer=3))


BAR_X, BAR_H = 1190, 200
RING_RADII = (22, 150)  # wide enough to slide over the plates, which are 200 tall


def slide(u, a, b):
    """Where the probe ring is, sliding along the wire from `a` to `b`."""
    return 200 + 880 * clamp((u - a) / (b - a))


def real_current(x):
    """1 where a wire runs through a ring at `x`, 0 in the gap between the plates."""
    return 1 - ramp(x, 596, 604) * (1 - ramp(x, 676, 684))


def probe(s, x, patch, show=1.0):
    """A ring sliding along the wire, and a bar for the B that Ampère's law gives
    at the ring: the real current fills it blue; `patch` fills what is missing
    in the gap with Maxwell's displacement current, in orange."""
    real = real_current(x)
    s.ring("probe", (x, WIRE_Y), RING_RADII, Look(w=5, layer=5, op=show))
    s.rect("bar-frame", (BAR_X, WIRE_Y), (44, BAR_H + 6), Look(edge=DIM, w=2, op=show))
    low = WIRE_Y + BAR_H / 2
    height = BAR_H * real
    s.rect(
        "bar-real",
        (BAR_X, low - height / 2),
        (36, max(height, 0.5)),
        Look(color=B_COL, layer=1, op=show if height > 1 else 0),
    )
    extra = BAR_H * (1 - real) * patch
    s.rect(
        "bar-patch",
        (BAR_X, low - height - extra / 2),
        (36, max(extra, 0.5)),
        Look(color=E_COL, layer=1, op=show if extra > 1 else 0),
    )
    s.text(
        "bar-name",
        "B",
        (BAR_X, WIRE_Y - BAR_H / 2 - 32),
        Look(color=B_COL, size=40, op=show),
    )
    s.text(
        "bar-why",
        "តាមអំពែរ",
        (BAR_X, WIRE_Y + BAR_H / 2 + 28),
        Look(color=DIM, size=24, op=show),
    )
    return real

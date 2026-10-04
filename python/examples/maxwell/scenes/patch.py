"""Scene 6. Maxwell's patch: a changing E in the gap acts like a current."""

from palette import E_COL, W
from pen import Look, ramp
from geometry import side_by_side
from equations import AMPERE, MAXWELL_TERM
from .capacitor import PLATES, WIRE_Y, capacitor, slide, probe


def draw(s, u, t, D):
    capacitor(s, t)
    grow = ramp(u, 0.05, 0.35)
    for i, y in enumerate((WIRE_Y - 55, WIRE_Y, WIRE_Y + 55)):
        s.arrow(
            ("e", i),
            (PLATES[0] + 14, y),
            (PLATES[1] - 14, y),
            Look(color=E_COL, w=2 + 5 * grow, head=18, op=0.25 + 0.75 * grow, layer=4),
        )
    s.text("plus", "+", (PLATES[0] - 38, WIRE_Y - 125), Look(size=24 + 22 * grow))
    s.text("minus", "−", (PLATES[1] + 38, WIRE_Y - 125), Look(size=24 + 22 * grow))
    size = 64
    first, second = side_by_side([AMPERE, MAXWELL_TERM], size, W / 2)
    s.formula("eq", AMPERE, (first, 150), Look(size=size))
    s.formula(
        "term",
        MAXWELL_TERM,
        (second, 150),
        Look(color=E_COL, size=size, reveal=ramp(u, 0.2, 0.45)),
    )
    s.text(
        "e-tag",
        "E ប្តូរ",
        (W / 2, 214),
        Look(color=E_COL, size=32, op=ramp(u, 0.15, 0.3)),
    )
    probe(s, slide(u, 0.45, 0.92), 1.0, show=ramp(u, 0.38, 0.45))

"""Scene 1. The question: four equations, and the one term added later."""

from palette import DIM, E_COL, W
from pen import Look, ramp
from geometry import side_by_side
from equations import GAUSS, NO_MONOPOLE, FARADAY, AMPERE, MAXWELL_TERM


def draw(s, u, t, D):
    s.text("who", "ម៉ាក់ស្វិល", (W / 2, 78), Look(size=54))
    s.text(
        "when",
        "James Clerk Maxwell · 1861 – 1865",
        (W / 2, 130),
        Look(color=DIM, size=26),
    )
    rows = [(GAUSS, 215), (NO_MONOPOLE, 300), (FARADAY, 385)]
    for i, (latex, y) in enumerate(rows):
        s.formula(
            ("eq", i),
            latex,
            (W / 2, y),
            Look(color=DIM, size=50, reveal=ramp(u, 0.06 + 0.1 * i, 0.16 + 0.1 * i)),
        )
    size = 50
    first, second = side_by_side([AMPERE, MAXWELL_TERM], size, W / 2)
    s.formula(
        "ampere",
        AMPERE,
        (first, 505),
        Look(color=DIM, size=size, reveal=ramp(u, 0.36, 0.48)),
    )
    s.formula(
        "maxwell",
        MAXWELL_TERM,
        (second, 505),
        Look(color=E_COL, size=size, reveal=ramp(u, 0.62, 0.78)),
    )
    s.text(
        "added",
        "ធាតុដែលម៉ាក់ស្វិលបន្ថែម",
        (second, 600),
        Look(color=E_COL, size=30, op=ramp(u, 0.8, 0.9)),
    )

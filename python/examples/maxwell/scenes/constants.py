"""Scene 8. What ε₀ and μ₀ are: charges pushing, wires pulling."""

import math

from palette import INK, E_COL, B_COL, COPPER
from pen import Look, ramp


def draw(s, u, t, D):
    """What ε₀ and μ₀ are, with no numbers: each is one fixed number of nature.
    ε₀ says how hard electricity pushes, μ₀ how hard currents pull."""
    pulse = math.sin(2 * math.pi * t / 1.6)
    # ε₀ on the left
    x, show = 320, ramp(u, 0.04, 0.16)
    s.formula("sym-e", r"\varepsilon_0", (x, 110), Look(size=90, reveal=show))
    s.text(
        "name-e", "ថេរអគ្គិសនី (ពែរមីទីវីតេ)", (x, 195), Look(color=E_COL, size=32, op=show)
    )
    for side in (-1, 1):
        on = ramp(u, 0.1, 0.22)
        s.circle(("charge", side), (x + side * 100, 310), 34, Look(color=E_COL, op=on))
        s.text(
            ("plus", side),
            "+",
            (x + side * 100, 310),
            Look(color="#1b1000", size=48, op=on, layer=3),
        )
        s.arrow(
            ("push", side),
            (x + side * 146, 310),
            (x + side * (146 + 64 + 10 * pulse), 310),
            Look(color=E_COL, w=6, head=24, op=ramp(u, 0.14, 0.26)),
        )
    s.text(
        "says-e",
        "កំណត់កម្លាំងរុញរវាងបន្ទុក",
        (x, 410),
        Look(size=32, op=ramp(u, 0.22, 0.34)),
    )
    # μ₀ on the right
    x, show = 960, ramp(u, 0.42, 0.54)
    s.formula("sym-m", r"\mu_0", (x, 110), Look(size=90, reveal=show))
    s.text(
        "name-m",
        "ថេរម៉ាញេទិច (ជម្រាបសុញ្ញកាស)",
        (x, 195),
        Look(color=B_COL, size=32, op=show),
    )
    for name, y, push in (("upper", 270, 1), ("lower", 360, -1)):
        on = ramp(u, 0.48, 0.6)
        s.line(
            ("wire", name), (x - 190, y), (x + 190, y), Look(color=COPPER, w=9, op=on)
        )
        s.arrow(
            ("current", name),
            (x - 175, y),
            (x - 105, y),
            Look(color=INK, w=4, head=18, layer=3, op=on),
        )
        for side in (-1, 1):
            start = y + push * 12
            s.arrow(
                ("pull", name, side),
                (x + side * 70, start),
                (x + side * 70, start + push * (30 + 4 * pulse)),
                Look(color=B_COL, w=5, head=14, op=ramp(u, 0.52, 0.64)),
            )
    s.text(
        "says-m",
        "កំណត់កម្លាំងទាញរវាងខ្សែចរន្ត",
        (x, 410),
        Look(size=32, op=ramp(u, 0.6, 0.72)),
    )

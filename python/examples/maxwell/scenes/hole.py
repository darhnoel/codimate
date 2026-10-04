"""Scene 5. The hole: Ampère's law, asked at each place along a capacitor."""

from palette import TAG, RED, W
from pen import Look, ramp
from geometry import side_by_side
from equations import AMPERE
from .capacitor import capacitor, slide, probe


def draw(s, u, t, D):
    """Slide a ring along the wire and ask Ampère's law for B at each place:
    steady where a wire runs through it, zero in the gap, steady again after."""
    capacitor(s, t)
    x = slide(u, 0.12, 0.82)
    real = probe(s, x, 0.0, show=ramp(u, 0.04, 0.12))
    s.text(
        "gap-tag",
        "ក្នុងចន្លោះ៖ គ្មានចរន្ត",
        (W / 2, 232),
        Look(color=RED, size=32, op=(1 - real) * 0.9),
    )
    size = 64
    first_x, second_x = side_by_side([AMPERE, "??"], size, W / 2)
    s.formula("eq", AMPERE, (first_x, 150), Look(size=size))
    s.formula(
        "query",
        "??",
        (second_x + 20, 150),
        Look(color=RED, size=size, reveal=ramp(x, 600, 650)),
    )
    s.text("tag", "កុងដង់សាទ័រ", (W / 2, 52), Look(color=TAG, size=48))

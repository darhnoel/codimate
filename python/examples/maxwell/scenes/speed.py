"""Scene 9. The speed: the values put into the formula, step by step."""

import codimate as cm

from palette import INK, E_COL, B_COL, W
from pen import Look, ramp


def draw(s, u, t, D):
    """One line that changes in place: the formula, then the values put in, then
    the product, the root, and the number. Each stage replaces the last where it
    stands, so the eye follows the one equation instead of reading a list."""
    values = (
        ("e", 330, r"\varepsilon_0=8.8541878\times10^{-12}\,\mathrm{F/m}", E_COL),
        ("m", 950, r"\mu_0=4\pi\times10^{-7}\,\mathrm{H/m}", B_COL),
    )
    for name, x, latex, color in values:
        s.formula(
            ("value", name),
            latex,
            (x, 55),
            Look(
                color=color,
                size=34,
                op=1 - ramp(u, 0.46, 0.54),
                reveal=ramp(u, 0.04, 0.14),
            ),
        )
    stages = (
        (0.16, r"c=\frac{1}{\sqrt{\mu_0\varepsilon_0}}"),
        (0.34, r"c=\frac{1}{\sqrt{(4\pi\times10^{-7})(8.8541878\times10^{-12})}}"),
        (0.50, r"c=\frac{1}{\sqrt{1.11265006\times10^{-17}}}"),
        (0.64, r"c=\frac{1}{3.33564095\times10^{-9}}"),
        (0.78, r"c=299\,792\,458\ \mathrm{m/s}"),
    )
    y = 370
    for i, (start, latex) in enumerate(stages):
        gone = (
            0.0
            if i == len(stages) - 1
            else ramp(u, stages[i + 1][0], stages[i + 1][0] + 0.06)
        )
        s.formula(
            ("stage", i),
            latex,
            (W / 2, y),
            Look(
                color=E_COL if i == len(stages) - 1 else INK,
                size=56,
                op=1 - gone,
                reveal=ramp(u, start, start + 0.08),
            ),
        )
    # the values travel into the formula as it fills in
    arrive = ramp(u, 0.3, 0.38) * (1 - ramp(u, 0.46, 0.52))
    for name, x, _, color in values:
        end = 520 if x < W / 2 else 760
        s.arrow(("into", name), (x, 90), (end, 250), Look(color=color, w=3, op=arrive))
    # a box drawn round the answer once it is written, settling in as it appears
    wide, high = cm.measure_math(stages[-1][1], 56)
    framed = ramp(u, 0.88, 0.95)
    s.rect(
        "frame",
        (W / 2, y),
        (wide + 80, high + 50),
        Look(edge=E_COL, w=4, op=framed),
    ).round(10).grow(1 + 0.1 * (1 - framed))

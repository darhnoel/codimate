"""Scene 7. The wave: E and B rising and falling together, travelling at c."""

import math

from codimate import science

from palette import INK, DIM, TAG, E_COL, B_COL, W
from pen import Look, clamp, ramp
from equations import FARADAY, AMPERE_ONLY_E


WAVE_LENGTH, WAVE_PERIOD, WAVE_AMP, WAVE_SPAN = 420.0, 3.0, 130.0, 560.0


def draw(s, u, t, D):
    """E and B as one wave: two sine curves in planes at right angles, rising
    and falling together, moving along x. Drawn flat by `science.Camera`. The
    real speed is 3 × 10⁸ m/s, so the wave here crawls, and the film says so."""
    cam = science.Camera(
        azimuth=-55, elevation=25, distance=4000.0, scale=0.8, centre=(W / 2, 330)
    )
    grow = ramp(u, 0.04, 0.22)
    amp = WAVE_AMP * grow + 1.0

    def wave(x):
        return math.sin(2 * math.pi * (x / WAVE_LENGTH - t / WAVE_PERIOD))

    def e_at(x, k=1.0):
        return cam.at((x, 0.0, amp * k * wave(x) + 0.6))

    def b_at(x, k=1.0):
        return cam.at((x, amp * k * wave(x) + 0.6, 0.0))

    lo, hi = -WAVE_SPAN, WAVE_SPAN
    A = WAVE_AMP
    s.poly(
        "plane-e",
        [cam.at(p) for p in ((lo, 0, -A), (hi, 0, -A), (hi, 0, A), (lo, 0, A))],
        Look(color=E_COL, op=0.13 * grow, layer=-3),
    )
    s.poly(
        "plane-b",
        [cam.at(p) for p in ((lo, -A, 0), (hi, -A, 0), (hi, A, 0), (lo, A, 0))],
        Look(color=B_COL, op=0.13 * grow, layer=-3),
    )
    s.line("axis", cam.at((lo, 0, 0)), cam.at((hi, 0, 0)), Look(color=DIM, w=2))
    s.arrow(
        "direction",
        cam.at((hi, 0, 0)),
        cam.at((hi + 110, 0, 0)),
        Look(color=INK, w=4, head=22, op=grow),
    )
    s.formula("c", "c", cam.at((hi + 150, 0, 0)), Look(size=44, op=grow))
    for i in range(-13, 14):
        x = i * 40.0
        strength = clamp(abs(wave(x)) * 2.2) * 0.85 * grow
        s.arrow(
            ("e-stem", i),
            cam.at((x, 0, 0)),
            e_at(x),
            Look(color=E_COL, w=3, head=15, op=strength),
        )
        s.arrow(
            ("b-stem", i),
            cam.at((x, 0, 0)),
            b_at(x),
            Look(color=B_COL, w=3, head=15, op=strength),
        )
    xs = [lo + (hi - lo) * i / 90 for i in range(91)]
    s.curve("b-wave", [b_at(x) for x in xs], Look(color=B_COL, w=6, op=grow, layer=1))
    s.curve("e-wave", [e_at(x) for x in xs], Look(color=E_COL, w=6, op=grow, layer=2))
    s.formula(
        "e-name",
        r"\mathbf{E}",
        cam.at((lo - 10, 0, A + 30)),
        Look(color=E_COL, size=58, op=ramp(u, 0.2, 0.32)),
    )
    s.text(
        "tag-f", "ផារ៉ាដេ", (330, 468), Look(color=TAG, size=38, op=ramp(u, 0.3, 0.4))
    )
    s.formula("eq-f", FARADAY, (330, 540), Look(size=44, reveal=ramp(u, 0.32, 0.5)))
    s.text(
        "tag-a",
        "អំពែរ–ម៉ាក់ស្វិល",
        (950, 66),
        Look(color=TAG, size=38, op=ramp(u, 0.5, 0.6)),
    )
    s.formula(
        "eq-a", AMPERE_ONLY_E, (950, 150), Look(size=44, reveal=ramp(u, 0.52, 0.7))
    )
    s.formula(
        "b-name",
        r"\mathbf{B}",
        cam.at((hi + 30, A + 40, 0)),
        Look(color=B_COL, size=58, op=ramp(u, 0.2, 0.32)),
    )

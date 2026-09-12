"""The easing panel: ease(local), with a dot riding it.

The curve is `cm.ease` — the Engine's own easing, called into rather than
copied, so this diagram cannot drift from what the animation actually does.
"""

import codimate as cm

from theme import CURVE_BOT, CURVE_L, CURVE_R, CURVE_TOP, DIM, GHOST, LIVE, mix


def _at(u):
    return (mix(CURVE_L, CURVE_R, u), mix(CURVE_BOT, CURVE_TOP, cm.ease(u)))


def draw(scene, local):
    scene.line("axis_x", start=(CURVE_L, CURVE_BOT), end=(CURVE_R, CURVE_BOT), w=1.0, color=DIM)
    scene.line("axis_y", start=(CURVE_L, CURVE_BOT), end=(CURVE_L, CURVE_TOP), w=1.0, color=DIM)
    for i in range(24):
        scene.line(("curve", i), start=_at(i / 24), end=_at((i + 1) / 24), w=2.5, color=GHOST)

    x, y = _at(local)
    scene.circle("rider", x=x, y=y, r=8, color=LIVE, layer=9)
    scene.text("curve_label", "ease(local)", x=(CURVE_L + CURVE_R) / 2,
               top=CURVE_BOT + 16, size=20, color="grey")

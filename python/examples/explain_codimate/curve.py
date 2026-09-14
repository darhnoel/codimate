"""The easing panel: ease(local), with a dot riding it.

The curve is `cm.ease` — the Engine's own easing, called into rather than
copied, so this diagram cannot drift from what the animation actually does.
"""

import codimate as cm

from theme import CURVE_BOT, CURVE_L, CURVE_R, CURVE_TOP, DIM, GHOST, LIVE, mix


def _at(u):
    return (mix(CURVE_L, CURVE_R, u), mix(CURVE_BOT, CURVE_TOP, cm.ease(u)))


def draw(scene, local):
    scene.line("axis_x", start=(CURVE_L, CURVE_BOT), end=(CURVE_R, CURVE_BOT),
               w=1.0).fill(DIM)
    scene.line("axis_y", start=(CURVE_L, CURVE_BOT), end=(CURVE_L, CURVE_TOP),
               w=1.0).fill(DIM)
    # Nine samples and one curve, rather than 24 straight pieces: `curve`
    # passes through every point it is given, so the shape of `ease` survives.
    scene.curve("curve", [_at(i / 8) for i in range(9)], w=2.5).fill(GHOST)

    x, y = _at(local)
    scene.circle("rider", r=8, at=(x, y)).fill(LIVE).on(layer=9)
    scene.text("curve_label", "ease(local)", size=20,
               at=cm.at(x=(CURVE_L + CURVE_R) / 2, top=CURVE_BOT + 16)).fill("grey")

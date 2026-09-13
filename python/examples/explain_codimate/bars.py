"""The bars panel: two stored pictures, and the one computed between them.

Does by hand, for four bars, exactly what the Engine does for every shape.
"""

import codimate as cm

from story import MOMENTS
from theme import ACCENT, CHART, INK, LIVE, mix


def draw(scene, active, local):
    before_order, _, _ = MOMENTS[active]
    after_order, highlight, _ = MOMENTS[active + 1]

    places = {index: slot for index, slot in enumerate(cm.row(4, gap=26, within=CHART))}
    eased = cm.ease(local)

    scene.text("chart_label", "computed, not stored", size=20,
               at=(CHART.x, CHART.bottom - 6)).fill("grey")

    for value in after_order:
        here = places[before_order.index(value)]
        there = places[after_order.index(value)]

        bar = scene.group(("bar", value), at=(mix(here.x, there.x, eased), here.bottom))
        bar.rect("box", w=here.w, h=value * 46,
                 at=cm.at(bottom=0)).fill(LIVE if value in highlight else ACCENT)
        bar.text("v", value, size=26, at=cm.at(top=16)).fill(INK)

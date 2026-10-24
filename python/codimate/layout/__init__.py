"""Where things sit: the canvas, Slots, and the helpers that divide it up.

The parts:

- `canvas` — The canvas's size, and how big text and formulas measure on it.
- `points` — Corner points of regular shapes: polygons and stars.
- `place` — Placing things: points, edges, Slots, and rows and columns of them.
- `axes` — Axes: a graph's frame, its ticks, and the map from data to pixels.
"""

from .canvas import (
    _CANVAS, _FONT, font, measure, measure_math, canvas, width, height,
)
from .points import (
    ngon, star,
)
from .place import (
    Place, at, Slot, row, column, _UNSET, _resolve,
)
from .axes import (
    Axes, axes,
)

__all__ = [
    'Axes', 'Place', 'Slot', '_CANVAS', '_FONT', '_UNSET',
    '_resolve', 'at', 'axes', 'canvas', 'column', 'font',
    'height', 'measure', 'measure_math', 'ngon', 'row', 'star', 'width',
]

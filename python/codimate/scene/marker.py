"""The marker: a highlighter stroke over any box (ADR 0027).

Pure geometry and colour. `Handle` and `Group` call into this; nothing here
knows about shapes, names or the payload.
"""

from __future__ import annotations

import math
import random
from dataclasses import dataclass

# Its colours are the highlighter names; on a marker these override the
# engine's named colours.
HIGHLIGHTS = {
    "yellow": "#ffec4f", "orange": "#ffa33a", "pink": "#ff8fc7",
    "green": "#8ee88a", "blue": "#7fd1ff", "purple": "#c9a3ff",
}
OPACITY = 0.85      # bright on black, with a hint of what is under it
LAYER = 9           # under the words, which are layer 10
SLANT = 0.2         # the chisel cut of each end, as a fraction of the height
RAGGED = 0.05       # how far an end's bumps bite in, as a fraction of height
BUMPS = 3           # bumps along each end; the point counts never change
INK, PAPER = "#171717", "#ffffff"   # marked words, and words unmarked by default
_NAMED = {"white": "#ffffff", "black": "#000000", "grey": "#8c8c8c",
          "gray": "#8c8c8c"}


@dataclass(frozen=True)
class Stroke:
    """What a marker needs to redraw itself at another reveal."""

    w: float
    h: float
    seed: str
    reveal: float = 1.0
    angle: float = 0.0     # degrees, from .turn(): laid out here, about the centre
    words: str = None      # the words a highlight's marker darkens as it draws on
    paper: str = PAPER     # those words' colour while unmarked


def parts(w: float, h: float, reveal: float, seed: str, angle: float = 0.0):
    """A highlighter stroke, centred on the origin and drawn from its left end
    to `reveal` (0 to 1) of the way along: its corners, one even colour.

    Straight along the top and bottom, tilted a little as a hand never draws
    level, and cut at each end at a slant like a chisel-tip pen, with a few
    small bumps biting in. The tilt and bumps are seeded from the marker's
    name, so they stay put from moment to moment. The same number of points
    at every reveal, so the Engine tweens one stroke into the next. `angle`
    (degrees) turns it about the centre, on top of the hand's tilt.
    """
    rng = random.Random(seed)
    tilt = math.radians(rng.uniform(0.5, 1.5) * rng.choice((-1, 1)))
    bites = [rng.uniform(0.3, 1.0) for _ in range(2 * BUMPS)]
    drawn = min(max(reveal, 0.0), 1.0) * w
    slant = min(SLANT * h, drawn)    # a stub too short to slant is cut flat
    bite = RAGGED * h * (slant / (SLANT * h) if h else 0.0)
    left, right = -w / 2, -w / 2 + drawn
    top, bottom = -h / 2, h / 2

    def end(x_top, x_bottom, inward, bumps):
        # From the top corner down to the bottom one, bitten in a little.
        out = []
        for i, b in enumerate(bumps, start=1):
            f = i / (BUMPS + 1)
            out.append((x_top + (x_bottom - x_top) * f + inward * bite * b,
                        top + (bottom - top) * f))
        return out

    body = ([(left + slant, top), (right, top)]
            + end(right, right - slant, -1.0, bites[:BUMPS])
            + [(right - slant, bottom), (left, bottom)]
            + end(left + slant, left, 1.0, bites[BUMPS:])[::-1])

    turn = tilt + math.radians(angle)
    c, s = math.cos(turn), math.sin(turn)
    return [(x * c - y * s, x * s + y * c) for x, y in body]


def blend(a: str, b: str, t: float) -> str:
    """The colour `t` of the way from `a` to `#rrggbb` `b`. `a` may be a name;
    one this does not know is not blended, only swapped half way."""
    t = min(max(t, 0.0), 1.0)
    a = _NAMED.get(a, HIGHLIGHTS.get(a, a))
    if not (a.startswith("#") and len(a) == 7):
        return a if t < 0.5 else b
    pa = [int(a[i:i + 2], 16) for i in (1, 3, 5)]
    pb = [int(b[i:i + 2], 16) for i in (1, 3, 5)]
    return "#" + "".join(f"{round(x + (y - x) * t):02x}" for x, y in zip(pa, pb))

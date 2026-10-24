"""The arithmetic of space: vectors and 3x3 matrices, as plain tuples.

Private to the kit. Space is **z up, right-handed**, the way a physics diagram
writes it.
"""

from __future__ import annotations

import math


def _dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def _sub(a, b):
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def _cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0])


def _unit(v):
    m = math.sqrt(_dot(v, v))
    if m == 0:
        raise ValueError("a direction needs a length")
    return (v[0] / m, v[1] / m, v[2] / m)


def _apply(rows, v):
    """A 3x3 matrix, given as rows, times a vector."""
    return tuple(_dot(row, v) for row in rows)


def _columns(rows):
    return tuple(zip(*rows))


def _turned(turn):
    """Rows of the rotation that turns by ``turn`` degrees about x, then y,
    then z."""
    ax, ay, az = (math.radians(a) for a in turn)
    cx, sx, cy, sy = math.cos(ax), math.sin(ax), math.cos(ay), math.sin(ay)
    cz, sz = math.cos(az), math.sin(az)
    return ((cz * cy, cz * sy * sx - sz * cx, cz * sy * cx + sz * sx),
            (sz * cy, sz * sy * sx + cz * cx, sz * sy * cx - cz * sx),
            (-sy, cy * sx, cy * cx))


def _tilt(p, degrees):
    """``p`` turned about the x axis — how an axis leans from the vertical."""
    c, s = math.cos(math.radians(degrees)), math.sin(math.radians(degrees))
    return (p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c)

"""Where light comes from, and the colours a lit surface is mixed from."""

from __future__ import annotations

from dataclasses import dataclass

from ._vectors import _dot, _sub, _unit


def _rgb(color: str):
    h = color.lstrip("#")
    if len(h) == 3:
        h = "".join(c * 2 for c in h)
    if len(h) != 6 or not color.startswith("#"):
        raise ValueError(
            f"a shaded body needs a hex colour like '#58c4dd', got {color!r}")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


@dataclass(frozen=True)
class Light:
    """Where light comes from.

    Give ``direction`` — the way to the light, in space — for a light so far
    away that its rays run parallel. Give ``source`` — a point — for a Sun:
    light then arrives *from that point*, so as a body moves around it the lit
    side moves too. A fixed headlamp that never moved would make an orbiting
    planet's day side stay put.

    Faces are shaded by mixing their colour toward ``shadow``, so what is
    drawn is opaque. Shading by opacity instead looks the same on black and
    lets everything behind show through on anything else.
    """

    direction: tuple = (0.4, 0.3, 0.85)
    source: "tuple | None" = None
    ambient: float = 0.12
    shadow: str = "#000000"

    def toward(self, point) -> tuple:
        """The unit vector from ``point`` to the light."""
        if self.source is not None:
            v = _sub(self.source, point)
            if _dot(v, v) > 0:
                return _unit(v)
        return _unit(self.direction)

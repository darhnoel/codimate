"""Paths round a body, and arrows that ride a body's equator."""

from __future__ import annotations

import math
from dataclasses import dataclass, replace

from ._vectors import _tilt
from .marks import _name
from .world import ABOVE


@dataclass(frozen=True)
class Orbit:
    """The path a body follows around another — a real ellipse, with the body
    it circles at a **focus**, not the middle.

        path = science.Orbit(a=1500, e=0.15, around=(0, 0, 0))
        earth_at = path.point(angle)

    ``a`` is the semi-major axis and ``e`` the eccentricity (0 is a circle).
    ``inclination`` leans the orbital plane about the x axis, in degrees.
    ``around`` is what is being circled; pass a body's centre each event to
    follow it, as a moon does an earth.

    ``point(angle)`` takes the **mean anomaly** — an angle that grows evenly
    with time — and solves Kepler's equation, so the body really does go
    quicker near the sun and slower far from it. Step ``angle`` by the same
    amount each tick and the speed changes itself. The drawn ``path`` is the
    same ellipse either way.

    ``hide`` names bodies (keys of a `World`'s ``bodies``) whose silhouettes
    cut the drawn path, so a ring never slices across a planet's face. That is
    a decision about how it looks, not about what is in front: the ring really
    does pass in front of the body at some angles, and reads as broken there
    whichever side is "correct".

    ``steps`` is how many segments the drawn path has. Each is a named shape,
    so a bigger ring that needs to look smooth costs more of them.

    ``periapsis`` turns the ellipse in its own plane, so the closest approach
    is that many degrees round from the line of nodes (0 puts it on +x), and
    ``node`` turns the whole tilted plane about the vertical — the longitude of
    the line where the orbit crosses the ground plane. Both default to 0, so an
    orbit that is not told is the one it always was.
    """

    a: float
    e: float = 0.0
    inclination: float = 0.0
    around: tuple = (0.0, 0.0, 0.0)
    color: str = "#4a5568"
    hide: tuple = ()
    steps: int = 240
    periapsis: float = 0.0
    node: float = 0.0

    def __post_init__(self):
        if self.a <= 0:
            raise ValueError(f"an orbit needs a size, got a={self.a}")
        if not 0.0 <= self.e < 1.0:
            raise ValueError(f"eccentricity is 0 up to (not including) 1, got {self.e}")

    def moved(self, **changes) -> "Orbit":
        """A copy with some of ``a``, ``e``, ``inclination``, ``around``,
        ``color``, ``hide``, ``steps``, ``periapsis`` or ``node`` changed."""
        return replace(self, **changes)

    def _place(self, eccentric: float) -> tuple:
        b = self.a * math.sqrt(1 - self.e ** 2)
        x = self.a * math.cos(eccentric) - self.a * self.e
        y = b * math.sin(eccentric)
        c, s = (math.cos(math.radians(self.periapsis)),
                math.sin(math.radians(self.periapsis)))
        local = _tilt((x * c - y * s, x * s + y * c, 0.0), self.inclination)
        c, s = math.cos(math.radians(self.node)), math.sin(math.radians(self.node))
        local = (local[0] * c - local[1] * s, local[0] * s + local[1] * c, local[2])
        return tuple(c + v for c, v in zip(self.around, local))

    def point(self, angle: float) -> tuple:
        """Where the body is at ``angle`` degrees of mean anomaly. ``0`` is
        the closest approach."""
        m = math.radians(angle)
        eccentric = m
        for _ in range(12):
            eccentric -= ((eccentric - self.e * math.sin(eccentric) - m)
                          / (1 - self.e * math.cos(eccentric)))
        return self._place(eccentric)

    def path(self, steps: int = 240) -> list:
        """``steps + 1`` points round the whole ellipse, closing on itself."""
        steps = max(int(steps), 3)
        return [self._place(2 * math.pi * i / steps) for i in range(steps + 1)]


@dataclass(frozen=True)
class RingArrows:
    """Short arrows riding round a body's equator: spin, made visible.

    On a solid ball a turn is hard to see, because the surface is one colour.
    Arrows riding the equator at the spin angle travel while the body turns, so
    the eye has something to follow. Draw them with the angle that turns the
    body and they keep pace with it.

    ``count`` arrows are spaced evenly, each covering ``span`` degrees. Those
    on the far side of the body are skipped — a real ray against the sphere,
    so they disappear behind it rather than floating on top like a diagram
    overlay. Give the arrows a ``layer`` above the world's (default `ABOVE`).
    """

    count: int = 6
    color: str = "#ffd23f"
    span: float = 16.0
    w: float = 4.0
    head: float = 15.0
    gap: float = 1.2
    layer: int = ABOVE

    def draw(self, scene, world, name, body, angle) -> int:
        """Draw them round ``body`` at ``angle`` degrees. Returns how many were
        in sight. Shapes are named ``(name, n)``."""
        cam, shown = world.camera, 0
        for n in range(self.count):
            start = angle + n * 360.0 / self.count
            tail = body.on_equator(start, self.gap)
            tip = body.on_equator(start + self.span, self.gap)
            middle = tuple((p + q) / 2 for p, q in zip(tail, tip))
            if cam.hidden_by(middle, body.centre, body.reach):
                continue
            scene.arrow(_name(name, n), start=cam.at(tail), end=cam.at(tip),
                        w=self.w, head=self.head) \
                .fill(self.color).on(layer=self.layer, opacity=0.95)
            shown += 1
        return shown

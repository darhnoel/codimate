"""The things a `World` draws: balls, and squished balls.

A body is a value, not a drawing. Every body gives the `World` the same few
things, so the `World` never asks which kind it has:

- ``centre``, ``color``, ``style``, and ``reach``: how far its farthest point
  is from its centre, which stands in wherever one radius is needed;
- ``_mesh()``: ``(points, faces, edges)`` in space, every face wound outward
  (counter-clockwise seen from outside), for the ``solid``, ``flat``, ``star``
  and ``wire`` styles;
- a round body also gives ``_stretch()``, the matrix that turns the unit ball
  into it, and ``levels``, for the ``smooth`` style.

A new kind of body is a class here that gives those; the `World` is unchanged.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, replace

from ._meshes import icosphere
from ._vectors import _apply, _tilt, _turned

STYLES = ("solid", "smooth", "flat", "star", "wire")

# What an Ellipsoid can be drawn as: no "star", whose mottle needs a spin.
OVAL_STYLES = ("solid", "smooth", "flat", "wire")


@dataclass(frozen=True)
class Sphere:
    """A ball in space. A value, not a drawing: a `World` draws it.

    ``style`` is how it is drawn: ``"solid"`` shaded by a `Light`;
    ``"smooth"`` shaded by a `Light` too, but drawn as stacked discs instead of
    a mesh — perfectly smooth and about a fifth of the shapes, ordered as one
    whole ball, with no surface for ``spin`` to turn; ``"flat"``
    one colour, for something that is itself the light; ``"star"`` a lit-from-within
    ball, dimmer toward its rim and mottled, the mottling turning with ``spin``;
    ``"wire"``
    only its mesh, see-through, faded with depth.

    ``spin`` turns it about its own axis (degrees), which is what the mesh
    shows moving. ``tilt`` leans that axis from the vertical, and stays put
    while the planet orbits — which is what makes seasons.

    ``detail`` is mesh fineness. It defaults to 2 for ``wire`` (480 edges) and
    3 for the others (1280 faces) — enough that flat shading does not show
    facets at a few hundred pixels across. For ``smooth`` it is the number of
    discs, 32 by default: 8 shows bands like contour lines, 32 reads as smooth.
    Every face is a named shape, and a dense scene is a lot of them: see
    ADR 0019 on ``render(index=...)``.
    """

    centre: tuple
    r: float
    color: str = "#58c4dd"
    style: str = "solid"
    spin: float = 0.0
    tilt: float = 0.0
    detail: "int | None" = None

    def __post_init__(self):
        if self.style not in STYLES:
            raise ValueError(
                f"unknown style {self.style!r} — use {', '.join(STYLES)}")
        if self.r <= 0:
            raise ValueError(f"a sphere needs a radius, got {self.r}")

    @property
    def levels(self) -> int:
        if self.detail is not None:
            return int(self.detail)
        if self.style == "smooth":
            return 32
        return 2 if self.style == "wire" else 3

    @property
    def reach(self) -> float:
        return self.r

    @property
    def axis(self) -> tuple:
        """The unit vector it spins about, after ``tilt``."""
        return _tilt((0.0, 0.0, 1.0), self.tilt)

    def moved(self, **changes) -> "Sphere":
        """A copy with some of ``centre``, ``r``, ``color``, ``style``,
        ``spin``, ``tilt`` or ``detail`` changed."""
        return replace(self, **changes)

    def on_equator(self, angle: float, reach: float = 1.0) -> tuple:
        """The point at ``angle`` degrees round the equator, ``reach`` times
        the radius out. ``angle`` is measured the way ``spin`` is, so a mark at
        ``spin`` turns with the surface."""
        a = math.radians(angle)
        local = _tilt((math.cos(a), math.sin(a), 0.0), self.tilt)
        return tuple(c + reach * self.r * v for c, v in zip(self.centre, local))

    def _mesh(self):
        _, faces, edges = icosphere(self.levels)
        points = [tuple(c + self.r * v for c, v in zip(self.centre, unit))
                  for unit in self._posed()]
        return points, faces, edges

    def _stretch(self):
        """Rows of the matrix that turns the unit ball into this one: for a
        sphere, just its radius. A smooth ball's discs are worked out through
        it, so a sphere and an Ellipsoid share one drawing."""
        r = self.r
        return ((r, 0.0, 0.0), (0.0, r, 0.0), (0.0, 0.0, r))

    def _posed(self):
        """The unit mesh after spin and tilt."""
        verts = icosphere(self.levels)[0]
        cs, sn = math.cos(math.radians(self.spin)), math.sin(math.radians(self.spin))
        ct, st = math.cos(math.radians(self.tilt)), math.sin(math.radians(self.tilt))
        out = []
        for x, y, z in verts:
            x, y = x * cs - y * sn, x * sn + y * cs
            out.append((x, y * ct - z * st, y * st + z * ct))
        return out


@dataclass(frozen=True)
class Ellipsoid:
    """A squished ball: a sphere stretched to three ``radii`` along x, y and z,
    then turned by ``turn`` degrees about x, then y, then z. A value, not a
    drawing: a `World` draws it, sorted together with spheres.

        egg = science.Ellipsoid((0, 0, 0), radii=(40, 30, 55),
                                color="#ece6da", style="smooth", turn=(0, 20, 35))

    ``style`` is ``"solid"``, ``"smooth"``, ``"flat"`` or ``"wire"``, as for a
    `Sphere`; ``detail`` likewise. Seen in perspective its outline is only
    nearly an ellipse; with an orthographic `Camera` it is exactly one.

    Where something needs a single radius — an orbit hiding behind it, a wire
    mesh fading with depth — the largest of the three stands in, which is a
    little generous for a flat one.
    """

    centre: tuple
    radii: tuple
    color: str = "#58c4dd"
    style: str = "solid"
    turn: tuple = (0.0, 0.0, 0.0)
    detail: "int | None" = None

    def __post_init__(self):
        if self.style not in OVAL_STYLES:
            raise ValueError(
                f"unknown style {self.style!r} — use {', '.join(OVAL_STYLES)}")
        if len(self.radii) != 3 or min(self.radii) <= 0:
            raise ValueError(f"radii are three positive lengths, got {self.radii}")

    @property
    def r(self) -> float:
        """The largest radius, for whatever needs just one."""
        return max(self.radii)

    @property
    def reach(self) -> float:
        return self.r

    @property
    def levels(self) -> int:
        if self.detail is not None:
            return int(self.detail)
        if self.style == "smooth":
            return 32
        return 2 if self.style == "wire" else 3

    def moved(self, **changes) -> "Ellipsoid":
        """A copy with some of ``centre``, ``radii``, ``color``, ``style``,
        ``turn`` or ``detail`` changed."""
        return replace(self, **changes)

    def _stretch(self):
        """Rows of the matrix that turns the unit ball into this one: stretch
        by the radii, then turn."""
        turned = _turned(self.turn)
        return tuple(tuple(turned[i][j] * self.radii[j] for j in range(3))
                     for i in range(3))

    def _mesh(self):
        verts, faces, edges = icosphere(self.levels)
        stretch = self._stretch()
        points = [tuple(c + v for c, v in zip(self.centre, _apply(stretch, unit)))
                  for unit in verts]
        return points, faces, edges

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

from ._meshes import edges_of, icosphere, lathe, ring, square_to, stack
from ._vectors import _apply, _cross, _dot, _sub, _tilt, _turned, _unit

STYLES = ("solid", "smooth", "painted", "flat", "star", "wire")

# What an Ellipsoid can be drawn as: no "star", whose mottle needs a spin.
OVAL_STYLES = ("solid", "smooth", "painted", "flat", "wire")


@dataclass(frozen=True)
class Sphere:
    """A ball in space. A value, not a drawing: a `World` draws it.

    ``style`` is how it is drawn: ``"solid"`` shaded by a `Light`;
    ``"smooth"`` shaded by a `Light` too, but drawn as stacked discs instead of
    a mesh — perfectly smooth and about a fifth of the shapes, ordered as one
    whole ball, with no surface for ``spin`` to turn; ``"painted"`` one
    picture with a truly smooth gradient, painted once per look and reused
    (needs ``pip install "codimate[painted]"``); ``"flat"``
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


# --------------------------------------------------------------------------
# Shapes made of a mesh: rods, arrows, tubes, boxes
# --------------------------------------------------------------------------

# What a mesh shape can be drawn as. "smooth" is for round bodies only.
MESH_STYLES = ("solid", "flat", "wire")


def _check(shape, sides=None):
    if shape.style not in MESH_STYLES:
        raise ValueError(
            f"unknown style {shape.style!r} — use {', '.join(MESH_STYLES)}")
    if sides is not None and sides < 3:
        raise ValueError(f"a round cross-section needs at least 3 sides, got {sides}")


def _with_edges(mesh):
    points, faces = mesh
    return points, faces, edges_of(faces)


@dataclass(frozen=True)
class Cylinder:
    """A rod from ``start`` to ``end``, ``r`` thick. ``caps="flat"`` closes its
    ends with discs; ``caps="round"`` with half-balls, which makes a capsule:
    a bond between two atoms, a rod-shaped bacterium.

        bond = science.Cylinder((0, 0, 0), (120, 0, 40), 8, "#c9d3e0")

    ``style`` is ``"solid"`` (shaded by a `Light`), ``"flat"`` or ``"wire"``.
    ``sides`` is how many faces go round it."""

    start: tuple
    end: tuple
    r: float
    color: str = "#58c4dd"
    style: str = "solid"
    caps: str = "flat"
    sides: int = 24

    def __post_init__(self):
        _check(self, self.sides)
        if self.caps not in ("flat", "round"):
            raise ValueError(f"caps are 'flat' or 'round', got {self.caps!r}")
        if self.r <= 0 or self.start == self.end:
            raise ValueError("a cylinder needs a length and a positive radius")

    @property
    def centre(self) -> tuple:
        return tuple((a + b) / 2 for a, b in zip(self.start, self.end))

    @property
    def reach(self) -> float:
        """How far its farthest point is from its centre."""
        return math.dist(self.start, self.end) / 2 + self.r

    def moved(self, **changes) -> "Cylinder":
        return replace(self, **changes)

    def _mesh(self):
        if self.caps == "flat":
            straight = [(0.0, self.r), (1.0, self.r)]
            return _with_edges(lathe(self.start, self.end, straight, self.sides,
                                     caps=True))
        # Round ends: quarter circles of rings, the radius going to zero at the
        # tips, which sit r beyond start and end along the axis.
        length = math.dist(self.start, self.end)
        d = _unit(_sub(self.end, self.start))
        tip0 = tuple(s - self.r * x for s, x in zip(self.start, d))
        tip1 = tuple(e + self.r * x for e, x in zip(self.end, d))
        total = length + 2 * self.r
        steps = max(4, self.sides // 4)
        profile = []
        for k in range(steps + 1):  # the start's half-ball, tip to rim
            a = math.pi / 2 * k / steps
            along = (self.r - self.r * math.cos(a)) / total
            profile.append((along, self.r * math.sin(a)))
        for k in range(steps + 1):  # the end's, rim to tip
            a = math.pi / 2 * k / steps
            profile.append(((self.r + length + self.r * math.sin(a)) / total,
                            self.r * math.cos(a)))
        return _with_edges(lathe(tip0, tip1, profile, self.sides, caps=False))


@dataclass(frozen=True)
class Arrow:
    """A shaded arrow in space, from ``start`` to its tip at ``end``: a shaft
    ``r`` thick and a cone ``head`` long and ``spread`` times as wide as the
    shaft. A force, a velocity, a field, seen from any side.

        push = science.Arrow((0, 0, 0), (0, 0, 150), 6, "#ff6b6b")

    The head is a third of the length unless ``head`` says, and never more
    than the whole arrow."""

    start: tuple
    end: tuple
    r: float
    color: str = "#58c4dd"
    style: str = "solid"
    head: "float | None" = None
    spread: float = 2.6
    sides: int = 20

    def __post_init__(self):
        _check(self, self.sides)
        if self.r <= 0 or self.start == self.end:
            raise ValueError("an arrow needs a length and a positive radius")

    @property
    def centre(self) -> tuple:
        return tuple((a + b) / 2 for a, b in zip(self.start, self.end))

    @property
    def reach(self) -> float:
        return math.dist(self.start, self.end) / 2 + self.r * self.spread

    def moved(self, **changes) -> "Arrow":
        return replace(self, **changes)

    def _mesh(self):
        length = math.dist(self.start, self.end)
        head = min(self.head if self.head is not None else length / 3, length)
        neck = 1 - head / length
        wide = self.r * self.spread
        profile = [(0.0, self.r), (neck, self.r), (neck, wide), (1.0, 0.0)]
        return _with_edges(lathe(self.start, self.end, profile, self.sides, caps=True))


@dataclass(frozen=True)
class Tube:
    """A thick line through space along ``points``, ``r`` thick: a strand of
    DNA, a knot, a field line. ``closed`` joins the end back to the start.

        loop = science.Tube(path, 6, "#7ad3ff", closed=True)

    The ring of the tube is carried along the path without twisting, so it
    does not corkscrew where the path bends; on a closed path the small twist
    left over at the join is spread evenly round it."""

    points: tuple
    r: float
    color: str = "#58c4dd"
    style: str = "solid"
    closed: bool = False
    sides: int = 12

    def __post_init__(self):
        _check(self, self.sides)
        if len(self.points) < 2 or self.r <= 0:
            raise ValueError("a tube needs at least two points and a positive radius")

    @property
    def centre(self) -> tuple:
        n = len(self.points)
        return tuple(sum(p[i] for p in self.points) / n for i in range(3))

    @property
    def reach(self) -> float:
        c = self.centre
        return max(math.dist(c, p) for p in self.points) + self.r

    def moved(self, **changes) -> "Tube":
        return replace(self, **changes)

    def _mesh(self):
        pts = [tuple(p) for p in self.points]
        n = len(pts)
        if self.closed and pts[0] == pts[-1]:
            pts, n = pts[:-1], n - 1
        ahead = []
        for i in range(n):  # the way the path goes at each point
            a = pts[i - 1] if (i > 0 or self.closed) else pts[i]
            b = pts[(i + 1) % n] if (i < n - 1 or self.closed) else pts[i]
            ahead.append(_unit(_sub(b, a)))
        u, _ = square_to(ahead[0])
        frames = []
        for i, d in enumerate(ahead):
            if i:  # carry u along: keep it square to the new direction
                u = _unit(_sub(u, tuple(_dot(u, d) * x for x in d)))
            frames.append((u, _cross(d, u)))
        if self.closed:  # spread the twist left at the join round the loop
            u0, v0 = frames[0]
            last_u = _unit(_sub(frames[-1][0], tuple(_dot(frames[-1][0], ahead[0]) * x
                                                     for x in ahead[0])))
            twist = math.atan2(_dot(last_u, v0), _dot(last_u, u0))
            fixed = []
            for i, ((u, v), d) in enumerate(zip(frames, ahead)):
                a = -twist * i / n
                c, s_ = math.cos(a), math.sin(a)
                nu = tuple(c * x + s_ * y for x, y in zip(u, v))
                fixed.append((nu, _cross(d, nu)))
            frames = fixed
        rings = [ring(p, u, v, self.r, self.sides) for p, (u, v) in zip(pts, frames)]
        points, faces = stack(rings, self.sides, close=self.closed)
        if not self.closed:  # flat ends
            for ring_index, outward in ((0, -1), (n - 1, 1)):
                first = ring_index * self.sides
                points.append(pts[ring_index])
                hub = len(points) - 1
                for k in range(self.sides):
                    k1 = (k + 1) % self.sides
                    tri = (hub, first + k, first + k1)
                    faces.append(tri if outward > 0 else (hub, first + k1, first + k))
        return _with_edges((points, faces))


@dataclass(frozen=True)
class Box:
    """A block: ``size`` is its width, depth and height along x, y and z,
    centred on ``centre``, then turned by ``turn`` degrees about x, then y,
    then z. A crystal, a container, a building block.

        crate = science.Box((0, 0, 50), (100, 100, 100), "#d9a066", turn=(0, 0, 30))
    """

    centre: tuple
    size: tuple
    color: str = "#58c4dd"
    style: str = "solid"
    turn: tuple = (0.0, 0.0, 0.0)

    def __post_init__(self):
        _check(self)
        if len(self.size) != 3 or min(self.size) <= 0:
            raise ValueError(f"size is three positive lengths, got {self.size}")

    @property
    def reach(self) -> float:
        return math.hypot(*self.size) / 2

    def moved(self, **changes) -> "Box":
        return replace(self, **changes)

    def _mesh(self):
        turned = _turned(self.turn)
        half = [s / 2 for s in self.size]
        points = []
        for x in (-1, 1):
            for y in (-1, 1):
                for z in (-1, 1):
                    local = (x * half[0], y * half[1], z * half[2])
                    points.append(tuple(c + v for c, v in
                                        zip(self.centre, _apply(turned, local))))
        # Corner index = 4*(x>0) + 2*(y>0) + (z>0); each face wound outward.
        faces = [(0, 1, 3, 2), (4, 6, 7, 5),   # -x, +x
                 (0, 4, 5, 1), (2, 3, 7, 6),   # -y, +y
                 (0, 2, 6, 4), (1, 5, 7, 3)]   # -z, +z
        return _with_edges((points, faces))

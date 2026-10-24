"""Where you are looking from: turns a point in space into a pixel."""

from __future__ import annotations

import math
from dataclasses import dataclass, replace
from functools import cached_property

from .. import layout
from ._vectors import _cross, _dot, _sub, _unit


@dataclass(frozen=True)
class Camera:
    """Where you are looking from. Turns a point in space into a pixel.

        cam = science.Camera(azimuth=45, elevation=35.264)   # true isometric
        pixel, depth = cam.project((100.0, 0.0, 50.0))

    ``azimuth`` spins around the vertical, ``elevation`` lifts above the
    ground plane, and ``distance`` is how far back the camera stands, looking
    at ``target``. The defaults are isometric: all three axes 120 degrees
    apart on screen.

    ``scale`` is pixels per unit **at the target**, so a unit of space is a
    pixel count you chose rather than something that depends on the lens.

    **Distance is how much perspective there is.** Far is flat; near is
    dramatic, and a sphere far from the middle of the picture stretches into an
    egg — measured on a real orbit at ratio 1.36 close in against 1.02 far
    back. If a round thing looks oval, stand further back, not move the
    thing.

    ``orthographic=True`` takes the perspective out exactly: every unit is
    ``scale`` pixels at any depth, so near and far are the same size, and
    ``distance`` only says where the camera stands for depth order. It is the
    flat view an engineering drawing, a map or a depth illusion needs, without
    standing a camera a million units away and trusting the rounding.

    It is a map, in the sense of `cm.axes` (ADR 0016): it hands back pixels
    and draws nothing. ``moved`` returns a changed copy, which is how a camera
    move is authored — emit a different camera at each event.
    """

    azimuth: float = 45.0
    elevation: float = 35.264
    distance: float = 1800.0
    scale: float = 0.7
    target: tuple = (0.0, 0.0, 0.0)
    centre: "tuple | None" = None
    orthographic: bool = False

    def __post_init__(self):
        if not -90.0 <= self.elevation <= 90.0:
            raise ValueError(f"elevation is -90 to 90 degrees, got {self.elevation}")
        if self.distance <= 0 or self.scale <= 0:
            raise ValueError("distance and scale must be positive")

    @property
    def origin(self) -> "tuple[float, float]":
        """The pixel the target lands on: the middle of the canvas by default."""
        if self.centre is not None:
            return (float(self.centre[0]), float(self.centre[1]))
        return (layout.width() / 2, layout.height() / 2)

    @property
    def focal(self) -> float:
        return self.scale * self.distance

    @cached_property
    def _frame(self):
        az, el = math.radians(self.azimuth), math.radians(self.elevation)
        view = (math.cos(el) * math.cos(az), math.cos(el) * math.sin(az),
                math.sin(el))
        position = tuple(t + self.distance * v for t, v in zip(self.target, view))
        forward = (-view[0], -view[1], -view[2])
        right = (-math.sin(az), math.cos(az), 0.0)
        return position, right, _cross(right, forward), forward

    @property
    def position(self) -> tuple:
        """Where the camera is, in space."""
        return self._frame[0]

    def _per_unit(self, depth: float) -> float:
        """Pixels per unit of space at ``depth``: the same everywhere when
        orthographic, fewer further off otherwise."""
        return self.scale if self.orthographic else self.focal / depth

    def _toward_eye(self, point) -> tuple:
        """The unit direction from ``point`` to the viewer. Orthographic, the
        viewer is at infinity, so it is the same for every point."""
        if self.orthographic:
            forward = self._frame[3]
            return (-forward[0], -forward[1], -forward[2])
        return _unit(_sub(self.position, point))

    def moved(self, **changes) -> "Camera":
        """A copy with some of ``azimuth``, ``elevation``, ``distance``,
        ``scale``, ``target``, ``centre`` or ``orthographic`` changed."""
        return replace(self, **changes)

    def dolly(self, distance: float) -> "Camera":
        """Stand at ``distance`` with the **same lens**: things shrink as you
        back away, which is what a pull-back is.

            wide = cam.dolly(4800)      # the same shot, from further off

        ``moved(distance=...)`` is not that. It keeps ``scale``, so a body stays
        the size it was and only the perspective changes — the vertigo shot.
        Reaching for ``moved`` to pull back and finding nothing shrinks is the
        mistake this exists to save.
        """
        return replace(self, distance=distance,
                       scale=self.scale * self.distance / distance)

    def project(self, point) -> "tuple[tuple[float, float], float]":
        """``((x, y), depth)``: the pixel, and how far along the line of sight.

        Depth is handed back rather than hidden, so one number can drive draw
        order and shading both. A point behind the camera is clamped to a
        depth of 1 rather than dividing by zero; keep bodies in front of it.
        """
        position, right, up, forward = self._frame
        rel = _sub(point, position)
        depth = max(_dot(rel, forward), 1.0)
        s = self._per_unit(depth)
        cx, cy = self.origin
        return (cx + _dot(rel, right) * s, cy - _dot(rel, up) * s), depth

    def at(self, point) -> "tuple[float, float]":
        """Just the pixel, for any shape's ``at=``."""
        return self.project(point)[0]

    def facing(self, point, normal) -> bool:
        """Does a surface at ``point`` with outward ``normal`` face the camera?

        Exact for a perspective camera. The cruder "normal points toward the
        viewing axis" is only right for a camera at infinity, and drops visible
        faces near the edge of a close body.
        """
        return _dot(normal, self._toward_eye(point)) > 0

    def hidden_by(self, point, centre, r: float) -> bool:
        """Is ``point`` behind the sphere at ``centre`` with radius ``r``, seen
        from here?

        A real ray against the sphere. Testing "which half of its own orbit is
        this on" is only valid for a point on the sphere's own surface; for a
        ring much larger than the body it answers a different question and
        wrongly hides about half of what the sphere never covered.
        """
        if self.orthographic:
            # The line of sight is parallel to the view: from the point, back
            # toward the viewer, does it meet the sphere?
            back = self._toward_eye(point)
            w = _sub(point, centre)
            b = _dot(w, back)
            disc = b * b - (_dot(w, w) - r * r)
            return disc >= 0 and -b + math.sqrt(disc) > 1e-6
        pos = self.position
        d = _sub(point, pos)
        dist = math.sqrt(_dot(d, d))
        if dist == 0:
            return False
        d = (d[0] / dist, d[1] / dist, d[2] / dist)
        to_centre = _sub(centre, pos)
        along = _dot(to_centre, d)
        off = _dot(to_centre, to_centre) - along * along
        if off > r * r:
            return False                 # the line of sight misses it entirely
        near = along - math.sqrt(r * r - off)
        return 0 < near < dist - 1e-6    # its near side comes before the point

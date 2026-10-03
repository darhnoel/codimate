"""Bodies in space — the 3D half of the kit: a camera, spheres, orbits.

The Engine is 2D and stays 2D (ADR 0019). Everything here is arithmetic that
ends in ordinary shapes: a `Camera` turns a point in space into a pixel, and a
`World` turns spheres into polygons or lines, sorted far to near, under names
you chose. Nothing is rendered in three dimensions; it is drawn flat in the
right order, which is a different and much smaller promise.

That promise has a limit, stated in ADR 0016 and kept: painter's order is
correct when shapes admit a separating plane. A convex body drawn at one place
and another one elsewhere do. Bodies that *interpenetrate* do not, and need a
depth buffer this does not have.

Space is **z up, right-handed**, the way a physics diagram writes it.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, replace
from functools import cached_property, lru_cache

from .. import layout
from .marks import _name

# Above any body a World will draw, for marks that must sit on top of one.
ABOVE = 100_000

STYLES = ("solid", "flat", "star", "wire")


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


def _tilt(p, degrees):
    """``p`` turned about the x axis — how an axis leans from the vertical."""
    c, s = math.cos(math.radians(degrees)), math.sin(math.radians(degrees))
    return (p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c)


# --------------------------------------------------------------------------
# Camera
# --------------------------------------------------------------------------


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
    thing. A very large ``distance`` is an orthographic camera.

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

    def moved(self, **changes) -> "Camera":
        """A copy with some of ``azimuth``, ``elevation``, ``distance``,
        ``scale``, ``target`` or ``centre`` changed."""
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
        s = self.focal / depth
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
        return _dot(normal, _sub(self.position, point)) > 0

    def hidden_by(self, point, centre, r: float) -> bool:
        """Is ``point`` behind the sphere at ``centre`` with radius ``r``, seen
        from here?

        A real ray against the sphere. Testing "which half of its own orbit is
        this on" is only valid for a point on the sphere's own surface; for a
        ring much larger than the body it answers a different question and
        wrongly hides about half of what the sphere never covered.
        """
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


# --------------------------------------------------------------------------
# The sphere mesh
# --------------------------------------------------------------------------


@lru_cache(maxsize=None)
def icosphere(detail: int):
    """``(vertices, faces, edges)`` of a unit geodesic sphere.

    Made by cutting an icosahedron's triangles into four, ``detail`` times:
    ``10 * 4**detail + 2`` vertices, ``20 * 4**detail`` faces. Computed once
    per detail and shared.

    Not a latitude/longitude grid, on purpose. That has two poles where every
    meridian meets, and seen near edge-on a wireframe of it shows two dense
    tufts that read as a peanut — finer lines do not help, because the poles
    are the problem. A geodesic mesh has no special points to look at.
    """
    if not 0 <= detail <= 6:
        raise ValueError(f"detail is 0 to 6, got {detail}")
    phi = (1 + 5 ** 0.5) / 2
    verts = [_unit(v) for v in (
        (-1, phi, 0), (1, phi, 0), (-1, -phi, 0), (1, -phi, 0),
        (0, -1, phi), (0, 1, phi), (0, -1, -phi), (0, 1, -phi),
        (phi, 0, -1), (phi, 0, 1), (-phi, 0, -1), (-phi, 0, 1))]
    faces = [(0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11),
             (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
             (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9),
             (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1)]
    cache = {}

    def middle(i, j):
        key = (min(i, j), max(i, j))
        if key not in cache:
            a, b = verts[i], verts[j]
            verts.append(_unit(((a[0] + b[0]) / 2, (a[1] + b[1]) / 2,
                                (a[2] + b[2]) / 2)))
            cache[key] = len(verts) - 1
        return cache[key]

    for _ in range(detail):
        cut = []
        for a, b, c in faces:
            ab, bc, ca = middle(a, b), middle(b, c), middle(c, a)
            cut += [(a, ab, ca), (b, bc, ab), (c, ca, bc), (ab, bc, ca)]
        faces = cut
    edges = sorted({(min(i, j), max(i, j)) for a, b, c in faces
                    for i, j in ((a, b), (b, c), (c, a))})
    return tuple(verts), tuple(faces), tuple(edges)


# --------------------------------------------------------------------------
# Light, Sphere
# --------------------------------------------------------------------------


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


@dataclass(frozen=True)
class Sphere:
    """A ball in space. A value, not a drawing: a `World` draws it.

    ``style`` is how it is drawn: ``"solid"`` shaded by a `Light`; ``"flat"``
    one colour, for something that is itself the light; ``"star"`` a lit-from-within
    ball, dimmer toward its rim and mottled, the mottling turning with ``spin``;
    ``"wire"``
    only its mesh, see-through, faded with depth.

    ``spin`` turns it about its own axis (degrees), which is what the mesh
    shows moving. ``tilt`` leans that axis from the vertical, and stays put
    while the planet orbits — which is what makes seasons.

    ``detail`` is mesh fineness. It defaults to 2 for ``wire`` (480 edges) and
    3 for the others (1280 faces) — enough that flat shading does not show
    facets at a few hundred pixels across. Every face is a named shape, and a
    dense scene is a lot of them: see ADR 0019 on ``render(index=...)``.
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
        return 2 if self.style == "wire" else 3

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


# --------------------------------------------------------------------------
# Orbit
# --------------------------------------------------------------------------


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


# --------------------------------------------------------------------------
# World
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class World:
    """A camera, a light and a name: what draws bodies and orbits together.

        view = science.World(science.Camera(), light=science.Light(source=(0, 0, 0)))
        top = view.draw(scene, {"sun": sun, "earth": earth}, {"year": orbit})

    ``draw`` takes the bodies **as one set**, because they have to be sorted
    together: sorted separately, a moon behind its planet would be painted
    over it. Every face and every orbit segment of every body is ordered far to
    near and given its own layer, from ``layer`` upward.

    Each shape is named after the key *you* gave the body: ``(name, key, "face",
    n)`` for a face of the mesh, ``(name, key, "edge", i, j)`` for a wire edge
    between two vertices, ``(name, key, "orbit", n)`` for a segment of a path.
    The number belongs to a patch of the surface, which spins with the body — so
    a face that turns out of sight and back keeps its identity, and the Engine
    tweens it rather than fading a stranger in.

    **Emit one event per frame.** Draw order is resolved once per *segment*, not
    per frame, so a layer that changes is a hard cut at the boundary between
    two Scenes. With one Scene per frame that cut is one frame long and
    cannot be seen. With one every few frames, a turning ball's faces
    re-sort in jumps, and anything that depends on them — a ring passing a
    planet, a moon going behind it — tears. Set ``Timing(default=1 / fps)`` and
    render at that ``fps``. (`examples/spacetime` found this the hard way.)

    Returns the next free layer, so a mark that must sit above the world — or
    ``ABOVE``, which is above any world — knows where to start.
    """

    camera: Camera
    light: "Light | None" = None
    name: str = "world"
    layer: int = 0

    def draw(self, scene, bodies, paths=None) -> int:
        """Draw ``bodies`` (a dict of key to `Sphere`) and ``paths`` (a dict of
        key to `Orbit`). Returns the next free layer."""
        light = self.light or Light()
        items = []
        for key, body in bodies.items():
            items += self._body(light, key, body)
        for key, orbit in (paths or {}).items():
            items += self._orbit(key, orbit, bodies)
        items.sort(key=lambda item: -item[0])

        for rank, (_, kind, name, geom, color, opacity, width) in enumerate(items):
            layer = self.layer + 1 + rank
            if kind == "face":
                scene.polygon(name, geom).fill(color, edge=color, edge_w=0.4) \
                    .on(layer=layer)
            else:
                scene.line(name, start=geom[0], end=geom[1], w=width) \
                    .fill(color).on(layer=layer, opacity=opacity)
        return self.layer + len(items) + 1

    def _silhouette(self, body):
        centre, depth = self.camera.project(body.centre)
        return centre, body.r * self.camera.focal / depth

    def _body(self, light, key, body):
        cam = self.camera
        _, faces, edges = icosphere(body.levels)
        posed = body._posed()
        world = [tuple(c + body.r * v for c, v in zip(body.centre, p)) for p in posed]
        screen = [cam.project(p) for p in world]
        items = []

        if body.style == "wire":
            centre_depth = cam.project(body.centre)[1]
            lw = min(max(self._silhouette(body)[1] / 70, 1.0), 1.8)
            for i, j in edges:
                (a, da), (b, db) = screen[i], screen[j]
                depth = (da + db) / 2
                t = min(max((depth - (centre_depth - body.r)) / (2 * body.r), 0.0), 1.0)
                items.append((depth, "edge", _name(self.name, key, "edge", i, j),
                              (a, b), body.color, 0.95 - 0.6 * t, lw))
            return items

        base, shade = _rgb(body.color), _rgb(light.shadow)
        lit = body.style == "solid"
        to_light = light.toward(body.centre)
        for n, (a, b, c) in enumerate(faces):
            normal = _unit(_cross(_sub(world[b], world[a]), _sub(world[c], world[a])))
            if _dot(normal, posed[a]) < 0:
                normal = (-normal[0], -normal[1], -normal[2])
            middle = tuple((world[a][k] + world[b][k] + world[c][k]) / 3
                           for k in range(3))
            if not cam.facing(middle, normal):
                continue
            if body.style == "star":
                # Limb darkening (rim dimmer) plus a fixed per-face mottle,
                # which belongs to the surface and so turns with `spin`.
                eye = _unit(_sub(cam.position, middle))
                limb = max(0.0, _dot(normal, eye)) ** 0.5
                mottle = ((n * 2654435761) % 1000) / 1000 - 0.5
                bright = min(1.0, 0.55 + 0.45 * limb + 0.16 * mottle)
                rgb = tuple(v * bright for v in base)
            elif lit:
                bright = light.ambient + (1 - light.ambient) * max(
                    0.0, _dot(normal, to_light))
                rgb = tuple(s + (v - s) * bright for s, v in zip(shade, base))
            else:
                rgb = base
            color = "#%02x%02x%02x" % tuple(int(round(v)) for v in rgb)
            depth = (screen[a][1] + screen[b][1] + screen[c][1]) / 3
            items.append((depth, "face", _name(self.name, key, "face", n),
                          [screen[a][0], screen[b][0], screen[c][0]], color, 1.0, 0))
        return items

    def _orbit(self, key, orbit, bodies):
        cam = self.camera
        cuts = [self._silhouette(bodies[k]) for k in orbit.hide if k in bodies]
        pts = [cam.project(p) for p in orbit.path(orbit.steps)]
        items = []
        for i in range(len(pts) - 1):
            (a, da), (b, db) = pts[i], pts[i + 1]
            mx, my = (a[0] + b[0]) / 2, (a[1] + b[1]) / 2
            if any(math.hypot(mx - c[0], my - c[1]) < rad * 1.03 for c, rad in cuts):
                continue
            items.append(((da + db) / 2, "edge", _name(self.name, key, "orbit", i),
                          (a, b), orbit.color, 0.5, 1.5))
        return items


# --------------------------------------------------------------------------
# RingArrows
# --------------------------------------------------------------------------


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
            if cam.hidden_by(middle, body.centre, body.r):
                continue
            scene.arrow(_name(name, n), start=cam.at(tail), end=cam.at(tip),
                        w=self.w, head=self.head) \
                .fill(self.color).on(layer=self.layer, opacity=0.95)
            shown += 1
        return shown

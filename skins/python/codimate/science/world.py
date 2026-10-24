"""A camera, a light and a name: what draws bodies and orbits together.

The Engine is 2D and stays 2D (ADR 0019). Everything here is arithmetic that
ends in ordinary shapes: a `Camera` turns a point in space into a pixel, and a
`World` turns bodies into polygons, discs or lines, sorted far to near, under
names you chose. Nothing is rendered in three dimensions; it is drawn flat in
the right order, which is a different and much smaller promise.

That promise has a limit, stated in ADR 0016 and kept: painter's order is
correct when shapes admit a separating plane. A convex body drawn at one place
and another one elsewhere do. Bodies that *interpenetrate* do not, and need a
depth buffer this does not have.
"""

from __future__ import annotations

import math
from dataclasses import dataclass

from ._vectors import _apply, _columns, _cross, _dot, _sub, _unit
from .camera import Camera
from .light import Light, _rgb
from .marks import _name

# Above any body a World will draw, for marks that must sit on top of one.
ABOVE = 100_000


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
        """Draw ``bodies`` (a dict of key to a body: a `Sphere`, an `Ellipsoid`,
        …) and ``paths`` (a dict of key to `Orbit`). Returns the next free
        layer."""
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
            elif kind == "image":
                at, size, squash, angle, path = geom
                picture = scene.image(name, path, size=size, at=at).on(layer=layer)
                if abs(1.0 - squash) > 1e-9:  # an egg: stretch the ball to it
                    picture.grow((1.0, squash)).turn(angle)
            elif kind == "disc":
                at, major, minor, angle = geom
                disc = scene.circle(name, r=major, at=at).fill(color).on(layer=layer)
                if major - minor > 1e-9 * major:  # an ellipse, not a circle
                    disc.grow((1.0, minor / major)).turn(angle)
            else:
                scene.line(name, start=geom[0], end=geom[1], w=width) \
                    .fill(color).on(layer=layer, opacity=opacity)
        return self.layer + len(items) + 1

    def _silhouette(self, body):
        centre, depth = self.camera.project(body.centre)
        return centre, body.reach * self.camera._per_unit(depth)

    def _unit_view(self, body):
        """How a round body looks from the camera, worked out on the unit ball
        it is stretched from (``_stretch``): its outline on screen, and the
        frame the round styles work in.

        Returns ``(cx, cy, depth, major, minor, angle)`` (the outline is an
        ellipse with those semi-axes, turned by ``angle`` degrees) and
        ``(stretch, cols, unstretch, w, p, q, screen)``: on the unit ball the
        outline is the circle square to ``w``, spanned by ``p`` and ``q``, and
        ``screen`` takes an offset there to pixels across and down."""
        cam = self.camera
        (cx, cy), depth = cam.project(body.centre)
        k = cam._per_unit(depth)
        _, right, up, _ = cam._frame
        stretch = body._stretch()
        cols = _columns(stretch)
        # The stretch's inverse, for a turn times a scale: unscale, unturn.
        unstretch = tuple(tuple(cols[i][j] / _dot(cols[i], cols[i]) for j in range(3))
                          for i in range(3))
        w = _unit(_apply(unstretch, cam._toward_eye(body.centre)))
        p_ = _unit(_cross(w, (0.0, 0.0, 1.0) if abs(w[2]) < 0.9 else (1.0, 0.0, 0.0)))
        q_ = _cross(w, p_)

        def screen(offset):
            """Pixels across and down for an offset on the unit ball."""
            v = _apply(stretch, offset)
            return _dot(v, right) * k, -_dot(v, up) * k

        # A circle through the stretch: an ellipse, by its two axes and angle.
        (m00, m10), (m01, m11) = screen(p_), screen(q_)
        a, b, c = m00 * m00 + m01 * m01, m00 * m10 + m01 * m11, m10 * m10 + m11 * m11
        spread = math.hypot((a - c) / 2, b)
        major = math.sqrt((a + c) / 2 + spread)
        minor = math.sqrt(max((a + c) / 2 - spread, 0.0))
        angle = math.degrees(0.5 * math.atan2(2 * b, a - c))
        outline = (cx, cy, depth, major, minor, angle)
        return outline, (stretch, cols, unstretch, w, p_, q_, screen)

    def _painted(self, light, key, body):
        """A round body as one picture: a ball painted with true shading, once
        per look and cached (`_paint`), placed at the body, sized to it, and
        stretched and turned to its outline. The light is given to the painting
        in the picture's own terms (across, up, toward the eye), so a light
        that is still relative to the camera reuses one painting all film.

        An egg is the painted ball stretched to its outline, which is close to
        its true shading but not exact: the stretch leans the lit side a
        little."""
        from . import _paint

        cx, cy, depth, major, minor, angle = self._unit_view(body)[0]
        cam = self.camera
        _, right, up, _ = cam._frame
        to_light = light.toward(body.centre)
        across, rise = _dot(to_light, right), _dot(to_light, up)
        toward = _dot(to_light, cam._toward_eye(body.centre))
        # The picture is drawn turned by `angle` (clockwise on screen), so the
        # light is turned back the other way into its terms.
        t = math.radians(angle)
        light_in_picture = (math.cos(t) * across - math.sin(t) * rise,
                            math.sin(t) * across + math.cos(t) * rise, toward)
        path = _paint.ball(body.color, light.shadow, light_in_picture, light.ambient)
        geom = ((cx, cy), 2 * major, minor / major, angle, path)
        return [(depth, "image", _name(self.name, key, "ball"), geom,
                 body.color, 1.0, 0)]

    def _smooth(self, light, key, body):
        """A ball, or a squished one, as stacked discs: each smaller, nearer the
        light, and brighter. Each disc shows as a crescent on the side away from
        the light, and takes the shade the `Light` gives the surface in the
        middle of that crescent, so this is the solid style's shading without
        the mesh. All the discs share the body's depth, stacked in order, so it
        sorts among the others as one thing.

        The work is done on the unit ball the body is stretched from
        (``_stretch``). There, the outline seen from the camera is a circle
        square to ``w``, the discs are circles in that plane, and the stretch
        carries each to the same ellipse on screen. For a sphere the stretch is
        its radius, and the ellipses are circles.
        """
        outline, frame = self._unit_view(body)
        cx, cy, depth, major, minor, angle = outline
        stretch, cols, unstretch, w, p_, q_, screen = frame

        # The brightest spot is where the surface faces the light. On the body
        # a normal is stretch⁻ᵀ·u, so it points at the light where u is along
        # stretchᵀ·light: that is the highlight's direction on the unit ball.
        to_light = light.toward(body.centre)
        lit = _unit(tuple(_dot(col, to_light) for col in cols))
        lx, ly = _dot(lit, p_), _dot(lit, q_)
        reach = math.hypot(lx, ly)
        # The way to the light across the disc; straight-on light has none, and
        # any way will do, the discs then sitting one inside the other.
        ax, ay = (lx / reach, ly / reach) if reach > 1e-9 else (1.0, 0.0)
        base, shade = _rgb(body.color), _rgb(light.shadow)
        count = max(1, body.levels)

        # The highlight is where the surface faces the light: `reach` out from
        # the middle, toward the light. The outer discs lean all the way that
        # way, touching the rim on the lit side, so each shows only as a
        # crescent on the dark side; the inner ones close in on the highlight.
        # Lit straight from the front there is no lit side, and they centre.
        highlight = min(reach / 0.87, 1.0)
        lean = min(reach / 0.3, 1.0)

        def disc(n):
            """(size, offset along p, offset along q) of disc n, on the unit ball."""
            t = n / (count - 1) if count > 1 else 0.0
            share = (1 - t * t) * lean + t * t * highlight
            out = 0.87 * t * share
            return 1.0 - 0.87 * t, out * ax, out * ay

        items = []
        for n in range(count):
            size, ox, oy = disc(n)
            inner, ix, iy = disc(n + 1) if n + 1 < count else (0.0, ox, oy)
            # What shows of this disc is a band between its edge and the next
            # disc's: a wide crescent away from the light, a thin sliver toward
            # it. Its shade is the light at the middle of each, weighted by how
            # wide each is, so the lit rim is not painted the dark side's shade.
            step = (ix - ox) * ax + (iy - oy) * ay
            far = (size - inner) + step
            near = max((size - inner) - step, 0.0)
            bright = 0.0
            # The smallest disc is the highlight: shade it by its own middle.
            sides = ((-1.0, far), (1.0, near)) if n + 1 < count else ((0.0, 1.0),)
            for side, weight in sides:
                px = (ox + side * ax * size + ix + side * ax * inner) / 2
                py = (oy + side * ay * size + iy + side * ay * inner) / 2
                rim = math.hypot(px, py)
                if rim > 0.97:  # just inside the outline, where the normal is sure
                    px, py = px * 0.97 / rim, py * 0.97 / rim
                pz = math.sqrt(max(0.0, 1.0 - px * px - py * py))
                spot = tuple(px * a_ + py * b_ + pz * c_
                             for a_, b_, c_ in zip(p_, q_, w))
                # The surface faces along the stretch's inverse-transpose.
                normal = _unit(tuple(_dot(col, spot) for col in zip(*unstretch)))
                facing = max(0.0, _dot(normal, to_light))
                bright += weight * (light.ambient + (1 - light.ambient) * facing)
            bright /= sum(weight for _, weight in sides) or 1.0
            rgb = tuple(s + (v - s) * bright for s, v in zip(shade, base))
            color = "#%02x%02x%02x" % tuple(int(round(v)) for v in rgb)
            dx, dy = screen(tuple(ox * a_ + oy * b_ for a_, b_ in zip(p_, q_)))
            items.append((depth - n * 1e-6, "disc", _name(self.name, key, "disc", n),
                          ((cx + dx, cy + dy), size * major, size * minor, angle),
                          color, 1.0, 0))
        return items

    def _body(self, light, key, body):
        if body.style == "smooth":
            return self._smooth(light, key, body)
        if body.style == "painted":
            return self._painted(light, key, body)
        cam = self.camera
        world, faces, edges = body._mesh()
        screen = [cam.project(p) for p in world]
        reach = body.reach
        items = []

        if body.style == "wire":
            centre_depth = cam.project(body.centre)[1]
            lw = min(max(self._silhouette(body)[1] / 70, 1.0), 1.8)
            for i, j in edges:
                (a, da), (b, db) = screen[i], screen[j]
                depth = (da + db) / 2
                t = min(max((depth - (centre_depth - reach)) / (2 * reach), 0.0), 1.0)
                items.append((depth, "edge", _name(self.name, key, "edge", i, j),
                              (a, b), body.color, 0.95 - 0.6 * t, lw))
            return items

        base, shade = _rgb(body.color), _rgb(light.shadow)
        lit = body.style == "solid"
        to_light = light.toward(body.centre)
        for n, face in enumerate(faces):
            a, b, c = face[0], face[1], face[2]
            # Faces are wound outward, so this points out of the body.
            try:
                normal = _unit(_cross(_sub(world[b], world[a]),
                                      _sub(world[c], world[a])))
            except ValueError:
                continue  # a face with no area
            middle = tuple(sum(world[i][k] for i in face) / len(face) for k in range(3))
            if not cam.facing(middle, normal):
                continue
            if body.style == "star":
                # Limb darkening (rim dimmer) plus a fixed per-face mottle,
                # which belongs to the surface and so turns with `spin`.
                eye = cam._toward_eye(middle)
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
            depth = sum(screen[i][1] for i in face) / len(face)
            items.append((depth, "face", _name(self.name, key, "face", n),
                          [screen[i][0] for i in face], color, 1.0, 0))
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

"""Group: where shapes are drawn, each placed relative to the group's point."""

from __future__ import annotations

from dataclasses import replace
from typing import TYPE_CHECKING, Any, Hashable

from ..layout import Place, Slot, _UNSET, _resolve
from . import marker as mk
from .handle import Handle
from .shape import _Shape, _key, _point

if TYPE_CHECKING:
    from .root import Scene


class Group:
    """Somewhere to draw, with its own origin and its own name.

    Everything drawn on a Group moves as one thing, because the Engine sees
    each child's name as part of the group's name.

    **Inside a Group, ``0`` is the Group's anchor point.** ``bottom=0`` stands
    a shape on it; ``top=20`` puts one 20 below it.
    """

    def __init__(self, scene: "Scene", path: tuple, ox: float, oy: float, w: float):
        self._scene = scene
        self._path = path
        self._ox = ox
        self._oy = oy
        self._w = w

    # -- the origin a child measures from ------------------------------------

    @property
    def _child_default(self):
        """Inside a Group, a shape that says nothing sits at the Group's point.

        Not inherited from the parent's *anchor* — 0 always means this Group's
        own point, at any depth. That is what keeps nesting rule-free.
        """
        return 0.0 if self._path else _UNSET


    def _place(self, key, kind, **fields) -> "Handle":
        path = self._path + (key,)
        name = _key(path)
        if name in self._scene._shapes:
            raise ValueError(
                f"two shapes share the name {name!r} — a name means exactly one thing"
            )
        # Positions arrive relative to this Group and leave absolute.
        for axis, origin in (("x", self._ox), ("y", self._oy),
                             ("x2", self._ox), ("y2", self._oy)):
            if axis in fields:
                fields[axis] = origin + fields[axis]
        shape = _Shape(item=name, kind=kind, **fields)
        self._scene._shapes[name] = shape
        return Handle(self._scene._shapes, name, self._scene._markers)

    def _where(self, at, half: float):
        """One placement argument in, a centre out.

        `at` may be a point, a Slot, a `cm.at(...)`, or nothing at all — inside
        a Group, nothing means the Group's own point.
        """
        if at is None:
            place = Place()
        elif isinstance(at, Place):
            place = at
        else:
            px, py = _point(at)
            place = Place(x=px, y=py)
        return (
            _resolve((place.x, None, None), 0.0, ("x",), self._child_default),
            _resolve((place.y, place.top, place.bottom), half,
                     ("y", "top", "bottom"), self._child_default),
        )

    # -- shapes ---------------------------------------------------------------
    #
    # Each one takes only what makes it that shape, plus where it goes.
    # Everything optional — colour, outline, scale, rotation, layer — is said
    # afterwards on the handle it returns:
    #
    #     scene.circle("bob", r=28, at=(x, y)).fill("orange").on(layer=4)
    #
    # That keeps every function small enough to hold in your head, and small
    # enough for a linter: `ruff --select PLR0913` allows five arguments and
    # these signatures used to carry eighteen.

    def rect(self, key: Hashable, *, h: float, w: float = None, at=None) -> "Handle":
        """A rectangle. Say where with a point, a Slot, or `cm.at(...)`.

            scene.rect("box", h=40, at=cm.at(bottom=0)).fill("blue")

        Round the corners with `.round(12)`.
        """
        if w is None:
            fallback = self._w if self._path else _UNSET
            w = _resolve((None, None, None), 0, ("w",), fallback)
        x, y = self._where(at, h / 2)
        return self._place(key, "rect", x=x, y=y, w=w, h=h)

    def circle(self, key: Hashable, *, r: float, at=None) -> "Handle":
        """A filled circle. `color="none"` with `.fill(edge=...)` draws a ring."""
        x, y = self._where(at, r)
        return self._place(key, "circle", x=x, y=y, r=r)

    def text(self, key: Hashable, content: Any, *,
             size: float = 16.0, at=None) -> "Handle":
        """Text, centred horizontally. You never deal with baselines."""
        x, y = self._where(at, size / 2)
        return self._place(key, "text", x=x, y=y, text=str(content), size=size,
                           layer=10)

    def marker(self, key: Hashable, *, w: float, h: float, at=None) -> "Handle":
        """A highlighter stroke over any box on screen.

            scene.marker("note", w=260, h=70, at=(640, 300)).turn(-8)
            scene.marker("note", w=260, h=70, at=(640, 300)).write(reveal=0.5)

        It is not tied to text. `.write(reveal=0..1)` draws it on from its left
        end, and `.turn(degrees)` sets its angle and so the way it draws. Colour
        with `.fill(name)`: yellow, orange, pink, green, blue or purple, or any
        `#rrggbb`. It is slightly translucent and sits under layer 10, where
        the words go. Drawn by hand: straight edges, a slight tilt and
        chisel-cut ends, the same every time for the same name.
        """
        x, y = self._where(at, h / 2)
        name = _key(self._path + (key,))
        handle = self._place(key, "polygon", x=x, y=y, points=(), w=1.0)
        self._scene._markers[name] = mk.Stroke(
            w=float(w), h=float(h), seed=f"marker {name}")
        handle._redraw(1.0)
        return handle.on(layer=mk.LAYER, opacity=mk.OPACITY).fill("yellow")

    def highlight(self, key: Hashable, content: Any, *, size: float = 16.0,
                  at=None) -> "tuple[Handle, Handle]":
        """Words under a highlighter stroke, the stroke sized to them.

            band, words = scene.highlight("pivot", "Pivot", size=30, at=(640, 360))
            band.write(reveal=0.0)       # rubbed out; the words turn white
            words.fill("#171717")        # on a white page, keep them dark

        A marker over the words' measured box, padded by a quarter of `size`
        on every side, with the words drawn as their own text shape on top.
        Marked, the words are near-black; `band.write(reveal=...)` blends them
        back to their own colour as the stroke rubs out, in step with it. Their
        own colour is white unless `words.fill(...)` says otherwise. Give each
        word its own name and keep its words fixed under that name: the stroke
        can then move and draw on or off between words without the letters
        overflowing it.
        """
        from ..layout import measure

        w, h = measure(str(content), size)
        pad = size / 4
        x, y = self._where(at, h / 2)
        band = self.marker(f"{key} marker", w=w + 2 * pad, h=h + 2 * pad, at=(x, y))
        words = self.text(key, content, size=size, at=(x, y))
        marks = self._scene._markers
        marks[band._name] = replace(marks[band._name], words=words._name)
        marks[words._name] = band._name
        band._redraw(marks[band._name].reveal)
        return band, words

    def formula(self, key: Hashable, latex: str, *,
                size: float = 16.0, at=None) -> "Handle":
        r"""Real mathematics, written as LaTeX.

            scene.formula("eq", r"\frac{QK^{T}}{\sqrt{d_k}}", size=34)

        Use a raw string, or every backslash needs doubling. `size` means what
        it means for :meth:`text`. Draw it on with `.write(reveal=, pen=)`.

        The result is glyph outlines, not a font — so it moves, fades and
        recolours like any other shape, and is typeset once when the video is
        built. Needs the `typst` binary on PATH. Rendering uses the bundled
        `ffmpeg`.
        """
        x, y = self._where(at, size / 2)
        return self._place(key, "formula", x=x, y=y, text=latex, size=size,
                           layer=10, r=1.0)

    def arc(self, key: Hashable, *, r, sweep, at=None) -> "Handle":
        """A slice of a circle — an angle mark, a pie, a dial, an orbit.

            scene.arc("angle", r=90, sweep=(0, 50)).fill("none", edge="cyan",
                                                          edge_w=3)
            scene.arc("slice", r=120, sweep=(0, 120)).fill("orange").round(1)

        `sweep` is `(start, end)` in degrees, clockwise from twelve o'clock —
        the same zero `cm.ngon` uses. `r` is the radius, or `(rx, ry)` for an
        ellipse.

        Open by default, so it draws as a curved line. `.round(1)` closes it
        back to the centre and makes a pie slice you can fill.

        Two arcs always tween, however far apart their angles are, so an angle
        mark grows and a pie fills smoothly. That is what this cannot be done
        with `curve`: a curve through points on a circle changes its point
        count when the sweep changes, and then it snaps instead of sweeping.
        """
        rx, ry = (r, r) if isinstance(r, (int, float)) else r
        start, end = sweep
        x, y = self._where(at, 0.0)
        return self._place(
            key, "arc", x=x, y=y,
            # The bounding box, so an ellipse costs nothing extra.
            w=float(rx) * 2, h=float(ry) * 2,
            # `x2`/`y2` are the line's endpoints elsewhere and free here.
            x2=float(start), y2=float(end),
        )

    def image(self, key: Hashable, file, *, size=None, at=None) -> "Handle":
        """A picture — a photo, a screenshot, a figure — drawn into the frame.

            scene.image("paper", "figures/attention.png", size=(520, 300))
            scene.image("shot", "screen.png", size=400).on(opacity=0.4)

        PNG and JPEG, read by content rather than by extension. `size` is a box
        it fits inside, one number or `(w, h)`, aspect always kept; leave it out
        and the picture is drawn at its own pixel size.

        It moves, scales, turns and fades like anything else, and `focus()`
        frames it. For a logo or a diagram prefer `svg`, which arrives as
        geometry you can recolour and draw on — an image is pixels, so it can
        only be placed.
        """
        box = (size, size) if isinstance(size, (int, float)) else size
        w, h = (0.0, 0.0) if box is None else (float(box[0]), float(box[1]))
        x, y = self._where(at, 0.0)
        return self._place(key, "image", x=x, y=y, text=str(file), w=w, h=h)

    def svg(self, key: Hashable, file, *, size=120.0, at=None) -> "Handle":
        """Vector art from a file, drawn as real geometry.

            scene.svg("logo", "brand.svg", size=90, at=cm.at(x=1180, top=24))
            scene.svg("chart", "flow.svg", size=(900, 420))

        Because it arrives as geometry rather than pixels it behaves like
        anything else you draw: it tweens, `.turn()` and `.grow()` transform
        it, `focus()` frames it, and `.write(pen=2)` draws it on stroke by
        stroke.

        `size` is a box it fits inside — one number for a square, or `(w, h)`.
        The aspect ratio is always kept, so a wide diagram and a tall one both
        land inside the box you named.

        The artwork keeps its own colours. `.fill(colour)` overrides all of
        them at once, which flattens it to a silhouette on purpose.

        Labels come across as real text, shaped by the same code that draws
        every `scene.text`. Layout that cannot be read is refused rather than
        guessed at — a `tspan`, a `textPath`, or a turned label, since the
        renderer cannot turn glyphs.
        """
        # A scalar is a square box, not a width — `_size` reads a bare number
        # as "width, height follows", which is right for a Slot and wrong for
        # something being fitted inside a box.
        box = (size, size) if isinstance(size, (int, float)) else size
        w, h = float(box[0]), float(box[1])
        x, y = self._where(at, 0.0)
        return self._place(
            key, "svg", x=x, y=y, text=str(file), w=w, h=h,
            # Empty means "as authored" — the Engine reads it as leave the
            # file's own colours alone, which `"white"` could not say.
            color="",
            # Fully revealed unless `.write(reveal=...)` says otherwise, the
            # same default a formula takes. `size` is the pen width here and
            # starts at zero — no pen — because `w` is the fit box.
            r=1.0, size=0.0,
        )

    def polygon(self, key: Hashable, points, *, closed: bool = True) -> "Handle":
        """A shape with corners: a triangle, a wedge, an arrow head, a wing.

            scene.polygon("tri", cm.ngon(3, r=50, at=(640, 360))).fill("green")

        `points` is a sequence of `(x, y)`. Closed and filled by default;
        `closed=False` leaves an open outline, which shows only with an edge.

        Two polygons tween only if they have the same number of corners —
        interpolating a triangle into a pentagon has no answer worth inventing,
        so the later shape stands for the whole beat instead.
        """
        flat, xs, ys = [], [], []
        for px, py in points:
            flat += [float(px), float(py)]
            xs.append(float(px))
            ys.append(float(py))
        if not flat:
            raise ValueError(f"polygon {key!r} has no points")
        return self._place(
            key, "polygon",
            # Anchored at the middle of its corners, so it travels as one thing.
            x=(min(xs) + max(xs)) / 2, y=(min(ys) + max(ys)) / 2,
            points=tuple(flat), w=1.0 if closed else 0.0,
        )

    def curve(self, key: Hashable, points, *, w: float = 2.0,
              closed: bool = False) -> "Handle":
        """A smooth line through every one of `points`.

            scene.curve("plot", samples, w=3).fill("cyan")

        The points are on the curve, not control points: you hand it samples —
        a function you plotted, a path something travelled — and it draws a
        smooth line through them. `w` is the stroke width.

        Open by default, and an open curve is *drawn* rather than filled, the
        way a line is. `closed=True` joins the ends and makes it a fillable
        shape, like a polygon with rounded-off corners.

        Two curves tween only if they have the same number of points, the same
        rule polygons follow; otherwise the later one stands for the beat.
        """
        flat, xs, ys = [], [], []
        for px, py in points:
            flat += [float(px), float(py)]
            xs.append(float(px))
            ys.append(float(py))
        if len(flat) < 4:
            raise ValueError(f"curve {key!r} needs at least 2 points")
        return self._place(
            key, "curve",
            # Anchored at the middle of its samples, so it travels as one thing.
            x=(min(xs) + max(xs)) / 2, y=(min(ys) + max(ys)) / 2,
            points=tuple(flat),
            # `w` is the closed flag in the payload, the way it is for a
            # polygon, so the stroke width travels as `edge_w`.
            w=1.0 if closed else 0.0, edge_w=float(w),
        )

    def arrow(self, key: Hashable, *, start, end, w: float = 4.0,
              head: float = 16.0) -> "Handle":
        """An arrow, as a single filled shape.

        One shape rather than a line plus a separate head, so it carries one
        name and travels as one thing. `w` is the shaft, `head` the point.
        """
        import math

        (x0, y0), (x1, y1) = _point(start), _point(end)
        dx, dy = x1 - x0, y1 - y0
        span = math.hypot(dx, dy) or 1.0
        ux, uy = dx / span, dy / span
        px, py = -uy, ux
        head = min(head, span)
        bx, by = x1 - ux * head, y1 - uy * head
        half, wing = w / 2, max(head * 0.55, w)
        return self.polygon(key, [
            (x0 + px * half, y0 + py * half), (bx + px * half, by + py * half),
            (bx + px * wing, by + py * wing), (x1, y1),
            (bx - px * wing, by - py * wing), (bx - px * half, by - py * half),
            (x0 - px * half, y0 - py * half),
        ])

    def line(self, key: Hashable, *, start, end, w: float = 2.0) -> "Handle":
        """A line between two points. `w` is its thickness.

        A line has no centre to anchor, so it takes its two ends directly. Each
        may be a Slot — a line joins the middles of two places — or an `(x, y)`.
        """
        (x, y), (x2, y2) = _point(start), _point(end)
        return self._place(key, "line", x=x, y=y, x2=x2, y2=y2, w=w)

    # -- nesting --------------------------------------------------------------

    def group(self, key: Hashable, slot: "Slot | None" = None, *,
              at=None, anchor: str = None, w: float = None) -> "Group":
        """A place to draw a thing made of several shapes.

            bar = scene.group(item.id, slot)
            bar.rect("bar", h=item.value * 70, at=cm.at(bottom=0))
            bar.text("label", item.value, at=cm.at(top=20))

        Give it a Slot and it sits where the Slot says, or place it with ``at``
        like any shape. Inside it, ``0`` is that point, and everything drawn on
        it moves as one.
        """
        if slot is not None:
            gx, gy = slot.point(anchor)
            gw = slot.w if w is None else w
        else:
            # A group with no Slot and no anchors sits exactly where its
            # parent does — a group that exists only to name things.
            gw = self._w if w is None else w
            gx, gy = self._where(at, 0.0)

        return Group(self._scene, self._path + (key,), self._ox + gx, self._oy + gy, gw)

"""What one moment looks like: Groups, Scenes, and the flat shape payload."""

from __future__ import annotations

from dataclasses import asdict, dataclass, replace
from typing import Any, Hashable

from .layout import Place, Slot, _UNSET, _resolve, width


@dataclass(frozen=True)
class _Shape:
    """One shape in the payload. Flat by design (ADR 0008) — the Engine diffs
    it field by field, so every kind carries every field."""

    item: str
    kind: str
    x: float
    y: float
    x2: float = 0.0
    y2: float = 0.0
    w: float = 0.0
    h: float = 0.0
    r: float = 0.0
    color: str = "white"
    points: tuple = ()
    edge: str = "white"
    edge_w: float = 0.0
    text: str = ""
    size: float = 16.0
    layer: int = 0
    opacity: float = 1.0
    scale_x: float = 1.0
    scale_y: float = 1.0
    rotate: float = 0.0
    pivot: str = "center"


PIVOTS = ("center", "top", "bottom", "left", "right")


def _key(path) -> str:
    """Join a path of keys into the string the Engine pairs shapes by.

    Nested keys flatten, so a name built from other names stays readable:
    ``("edge", (0, 1), (1, 2))`` becomes ``"edge/0/1/1/2"``. That is what lets
    you write ``("edge", src, dst)`` instead of concatenating tuples by hand.
    """
    parts: "list[str]" = []

    def walk(key):
        if isinstance(key, (tuple, list)):
            for part in key:
                walk(part)
        else:
            parts.append(str(key))

    walk(path)
    return "/".join(parts)


def _point(place) -> "tuple[float, float]":
    """A place: a Slot (its centre) or a plain (x, y)."""
    if isinstance(place, Slot):
        return (place.x, place.y)
    return (float(place[0]), float(place[1]))


def _at(at, x, y, edges):
    """Resolve ``at=`` against the per-axis arguments.

    A position is one thing, so it should arrive as one value. Splitting it
    into ``x=`` and ``y=`` forces a caller holding a point to take it apart —
    and in practice that meant calling the function twice and indexing ``[0]``
    and ``[1]``.

    The per-axis arguments stay, because they are not the same question: they
    are for when only one axis is known, or when an edge is what you mean
    (``top=``, ``bottom=``). Mixing the two is always a mistake, so it is an
    error rather than a precedence rule.
    """
    if at is None:
        return x, y
    if any(value is not None for value in (x, y, *edges)):
        raise ValueError("give at=, or x=/y= and edges — not both")
    return _point(at)


class Handle:
    """What a shape call gives back: a way to say more about that shape.

    Chained rather than passed, so no function carries eighteen arguments:

        scene.rect("bar", h=40, at=cm.at(bottom=0)).fill("blue").turn(30)

    Each call returns the handle again, and each one is small enough to read.
    """

    def __init__(self, shapes: dict, name: str):
        self._shapes = shapes
        self._name = name

    def _set(self, **changes) -> "Handle":
        # `_Shape` is frozen, so saying more about a shape replaces it rather
        # than mutating it. A handful of copies per shape, and the payload
        # stays a value.
        self._shapes[self._name] = replace(self._shapes[self._name], **changes)
        return self

    def fill(self, color: str = "white", edge: str = None,
             edge_w: float = None) -> "Handle":
        """Colour it. `edge` outlines it; `color="none"` leaves it unfilled.

        Saying nothing about the outline leaves the outline alone, so
        recolouring a shape does not silently erase the edge it was given —
        by an earlier `fill`, or by `curve`, which carries its stroke width
        there.
        """
        changes = {"color": color}
        if edge is not None:
            changes["edge"] = edge
        if edge_w is not None:
            changes["edge_w"] = float(edge_w)
        elif edge is not None:
            # An edge with no width was asked to be visible, so give it one.
            changes["edge_w"] = max(self._shapes[self._name].edge_w, 1.0)
        return self._set(**changes)

    def turn(self, degrees: float, pivot: str = "center") -> "Handle":
        """Rotate it. `pivot` is center, top, bottom, left or right.

        Text moves but does not turn — rotating glyphs is renderer work that
        has not been done.
        """
        if pivot not in PIVOTS:
            raise ValueError(
                f"unknown pivot {pivot!r} — use one of {', '.join(PIVOTS)}")
        return self._set(rotate=float(degrees), pivot=pivot)

    def grow(self, scale) -> "Handle":
        """Scale it: a number for both axes, or `(sx, sy)` to stretch."""
        sx, sy = (scale, scale) if isinstance(scale, (int, float)) else scale
        return self._set(scale_x=float(sx), scale_y=float(sy))

    def on(self, layer: int = None, opacity: float = None) -> "Handle":
        """Which layer it draws on, and how solid it is."""
        changes = {}
        if layer is not None:
            changes["layer"] = int(layer)
        if opacity is not None:
            changes["opacity"] = float(opacity)
        return self._set(**changes)

    def round(self, radius: float) -> "Handle":
        """Round a rectangle's corners, clamped to half its short side."""
        return self._set(r=float(radius))

    def write(self, reveal: float = None, pen: float = 0.0) -> "Handle":
        """How much of a formula or an imported SVG shows, and whether a pen
        draws it on.

        The pen rides in a different field for the two kinds — `w` for a
        formula, `size` for an SVG, whose `w` is already the box it fits
        inside. The payload reuses fields per kind by design (ADR 0008); this
        is the one place that reuse is visible from Python.
        """
        svg = self._shapes[self._name].kind == "svg"
        changes = {"size" if svg else "w": float(pen)}
        if reveal is not None:
            changes["r"] = float(reveal)
        return self._set(**changes)


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
        return Handle(self._scene._shapes, name)

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

    def formula(self, key: Hashable, latex: str, *,
                size: float = 16.0, at=None) -> "Handle":
        r"""Real mathematics, written as LaTeX.

            scene.formula("eq", r"\frac{QK^{T}}{\sqrt{d_k}}", size=34)

        Use a raw string, or every backslash needs doubling. `size` means what
        it means for :meth:`text`. Draw it on with `.write(reveal=, pen=)`.

        The result is glyph outlines, not a font — so it moves, fades and
        recolours like any other shape, and is typeset once when the video is
        built. Needs the `typst` binary on PATH, the way rendering needs
        `ffmpeg`.
        """
        x, y = self._where(at, size / 2)
        return self._place(key, "formula", x=x, y=y, text=latex, size=size,
                           layer=10, r=1.0)

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
            bar.rect("bar", h=item.value * 70, bottom=0)
            bar.text("label", item.value, top=20)

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


class Scene(Group):
    """The picture at one moment. No animation, no timing, no memory of the
    frame before.

    Draw on it with the shapes it inherits from `Group` — `rect`, `circle`,
    `text`, `line`, `formula` — and `group` to put several of them together.

    Every shape carries a name — the identity of the thing you are talking
    about. **Motion is implied by identity:** the Engine pairs shapes by name
    between one moment and the next, and whatever changed becomes movement. A
    name that follows a *value* travels; a name that follows a *position*
    stays put and changes shape. Both are useful, and choosing wrong produces a
    confusing video, not an error.
    """

    def __init__(self) -> None:
        self._shapes: dict[str, _Shape] = {}
        self._overlays: set = set()
        self._focus: "dict[str, Any] | None" = None
        super().__init__(self, (), 0.0, 0.0, width())

    def focus(self, *names: Hashable, pad: float = 40.0, least: float = 240.0) -> None:
        """Look at these things. The camera works out where to stand.

            scene.focus("cat")                    # frame this shape
            scene.focus(("cell", 2, 1), pad=60)   # any name, with room around it
            scene.focus("The", "cat", "sat")      # several — all of them in shot
            scene.focus()                         # the whole canvas again

        You never write a camera coordinate. The Engine knows where everything
        is and how big it is, so it derives the framing — and because each
        moment is framed on its own, a camera that moves between two moments is
        just two sets of coordinates that differ, which the usual tween
        animates. Camera movement needs nothing new.

        Naming rather than positioning also means the shot survives a layout
        change: move the shape and the camera follows, because the name is the
        part that is stable.

        ``least`` is the smallest thing the camera will fill the frame with, so
        focusing on a full stop does not magnify it into abstraction.

        Anything on an :meth:`overlay` stays where it is put.
        """
        self._focus = {
            "names": [_key(n) for n in names],
            "pad": float(pad),
            "min_size": float(least),
            "fixed": sorted(self._overlays),
        }

    def overlay(self) -> "Group":
        """A place for things the camera does not move.

            hud = scene.overlay()
            hud.text("title", "Scaled dot-product attention", x=640, y=52)

        The diagram pans and zooms underneath; whatever is drawn here stays
        put. Titles and narration belong here — the moment the picture zooms, a
        caption that zooms with it is wrong, and usually off the edge of the
        frame entirely.

        Content is in the world unless it says otherwise, because in an
        explanation most of the frame is diagram and only a little is
        narration.
        """
        # At the canvas origin, so its children are placed in plain screen
        # coordinates rather than relative to somewhere.
        group = self.group("_overlay", at=(0.0, 0.0))
        self._overlays.add("_overlay")
        return group

    def _payload(self) -> "list[dict[str, Any]]":
        return [asdict(s) for s in self._shapes.values()]

    def _camera(self) -> "dict[str, Any] | None":
        # Recomputed here rather than in `focus`, because an overlay may be
        # created after the camera is aimed.
        if self._focus is not None:
            self._focus["fixed"] = sorted(self._overlays)
        return self._focus

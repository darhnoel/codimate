"""What a shape call gives back: the chained way to say more about a shape."""

from __future__ import annotations

from dataclasses import replace

from . import marker as mk
from .._surface import _HandleSurface
from .._waist import call
from .shape import PIVOTS, _shape_of


class Handle(_HandleSurface):
    """What a shape call gives back: a way to say more about that shape.

    Chained rather than passed, so no function carries eighteen arguments:

        scene.rect("bar", h=40, at=cm.at(bottom=0)).fill("blue").turn(30)

    Each call returns the handle again, and each one is small enough to read.
    """

    def __init__(self, shapes: dict, name: str, marks: dict = None):
        self._shapes = shapes
        self._name = name
        self._marks = {} if marks is None else marks

    def _stroke(self) -> "mk.Stroke | None":
        mark = self._marks.get(self._name)
        return mark if isinstance(mark, mk.Stroke) else None

    def _redraw(self, reveal: float) -> "Handle":
        """Lay a marker and a highlight's words out at `reveal`."""
        stroke = replace(self._stroke(), reveal=float(reveal))
        self._marks[self._name] = stroke
        shape = self._shapes[self._name]

        def placed(part):
            return tuple(v for dx, dy in part for v in (shape.x + dx, shape.y + dy))

        body = mk.parts(stroke.w, stroke.h, stroke.reveal, stroke.seed, stroke.angle)
        if stroke.words in self._shapes:   # a highlight's words darken with it
            self._shapes[stroke.words] = replace(
                self._shapes[stroke.words],
                color=mk.blend(stroke.paper, mk.INK, stroke.reveal))
        return self._set(points=placed(body))

    def _set(self, **changes) -> "Handle":
        # `_Shape` is frozen, so saying more about a shape replaces it rather
        # than mutating it. A handful of copies per shape, and the payload
        # stays a value.
        self._shapes[self._name] = replace(self._shapes[self._name], **changes)
        return self

    def _shape(self) -> dict:
        # A shallow copy of its fields: everything in it is already plain data,
        # and `asdict`'s recursive copy cost more than the crossing itself.
        return dict(vars(self._shapes[self._name]))

    def _change(self, op: str, data: dict) -> None:
        """Change this shape with the Authoring Core and keep the result."""
        self._shapes[self._name] = _shape_of(call(op, data))

    def fill(self, color: str = "white", *, edge: str = None,
             edge_w: float = None) -> "Handle":
        """Colour it. `edge` outlines it; `color="none"` leaves it unfilled.

        Saying nothing about the outline leaves the outline alone, so
        recolouring a shape does not silently erase the edge it was given —
        by an earlier `fill`, or by `curve`, which carries its stroke width
        there.
        """
        # ponytail: markers still live here until their slice moves to the Core.
        if self._stroke() is not None:
            color = mk.HIGHLIGHTS.get(color, color)
        super().fill(color, edge=edge, edge_w=edge_w)
        marker = self._marks.get(self._name)
        if isinstance(marker, str):   # a highlight's words: this is them unmarked
            self._marks[marker] = replace(self._marks[marker], paper=color)
            Handle(self._shapes, marker, self._marks)._redraw(
                self._marks[marker].reveal)
        return self

    def turn(self, degrees: float, pivot: str = "center") -> "Handle":
        """Rotate it. `pivot` is center, top, bottom, left or right.

        Text moves but does not turn — rotating glyphs is renderer work that
        has not been done.
        """
        if pivot not in PIVOTS:
            raise ValueError(
                f"unknown pivot {pivot!r} — use one of {', '.join(PIVOTS)}")
        if self._stroke() is not None:   # turned here, with the hand's tilt
            self._marks[self._name] = replace(self._stroke(), angle=float(degrees))
            return self._redraw(self._stroke().reveal)
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
        draws it on. On a marker, how far its stroke has been drawn on.

        The pen rides in a different field for the two kinds — `w` for a
        formula, `size` for an SVG, whose `w` is already the box it fits
        inside. The payload reuses fields per kind by design (ADR 0008); this
        is the one place that reuse is visible from Python.
        """
        if self._stroke() is not None:
            return self if reveal is None else self._redraw(float(reveal))
        svg = self._shapes[self._name].kind == "svg"
        changes = {"size" if svg else "w": float(pen)}
        if reveal is not None:
            changes["r"] = float(reveal)
        return self._set(**changes)

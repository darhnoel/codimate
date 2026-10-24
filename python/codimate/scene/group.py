"""Group: where shapes are drawn, each placed relative to the group's point."""

from __future__ import annotations

from dataclasses import replace
from typing import TYPE_CHECKING, Any, Hashable

from .._surface import _GroupSurface
from .._waist import call
from ..layout import Place, _UNSET, _resolve
from . import marker as mk
from .handle import Handle
from .shape import _Shape, _key, _point, _shape_of

if TYPE_CHECKING:
    from .root import Scene


class Group(_GroupSurface):
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

    def _frame(self) -> dict:
        """This Group as the Core needs it: its name path, point and width.
        Built once: a Group never moves after it is made."""
        frame = self.__dict__.get("_frame_data")
        if frame is None:
            frame = self._frame_data = {"path": [_key(p) for p in self._path],
                                        "ox": self._ox, "oy": self._oy, "w": self._w}
        return frame

    def _nest(self, frame: dict) -> "Group":
        """A Group at the frame the Authoring Core worked out."""
        group = Group(self._scene, tuple(frame["path"]), frame["ox"], frame["oy"],
                      frame["w"])
        group._frame_data = frame
        return group

    def _adopt(self, shapes: list) -> "tuple[Handle, ...]":
        """Keep several shapes the Authoring Core drew at once, in order."""
        return tuple(self._keep(_shape_of(shape)) for shape in shapes)

    def _keep(self, shape) -> "Handle":
        if shape.item in self._scene._shapes:
            raise ValueError(
                f"two shapes share the name {shape.item!r} — "
                "a name means exactly one thing"
            )
        self._scene._shapes[shape.item] = shape
        return Handle(self._scene._shapes, shape.item, self._scene._markers)

    def _build(self, op: str, data: dict) -> "Handle":
        """Draw with the Authoring Core: it places the shape, this keeps it."""
        return self._keep(_shape_of(call(op, data)))

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
                  font: str = None, at=None) -> "tuple[Handle, Handle]":
        """Words under a highlighter stroke, the stroke sized to them.

            band, words = scene.highlight("pivot", "Pivot", size=30, at=(640, 360))
            band.write(reveal=0.0)       # rubbed out; the words turn white
            words.fill("#171717")        # on a white page, keep them dark

        A marker over the words' measured box, padded by a quarter of `size`
        on every side, with the words drawn as their own text shape on top.
        Marked, the words are near-black; `band.write(reveal=...)` blends them
        back to their own colour as the stroke rubs out, in step with it. Their
        own colour is white unless `words.fill(...)` says otherwise. `font` is
        as for `text`, and the stroke is sized to the words in it. Give each
        word its own name and keep its words fixed under that name: the stroke
        can then move and draw on or off between words without the letters
        overflowing it.
        """
        from ..layout import measure

        w, h = measure(str(content), size, font)
        pad = size / 4
        x, y = self._where(at, h / 2)
        band = self.marker(f"{key} marker", w=w + 2 * pad, h=h + 2 * pad, at=(x, y))
        words = self.text(key, content, size=size, font=font, at=(x, y))
        marks = self._scene._markers
        marks[band._name] = replace(marks[band._name], words=words._name)
        marks[words._name] = band._name
        band._redraw(marks[band._name].reveal)
        return band, words

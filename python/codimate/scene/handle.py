"""What a shape call gives back: the chained way to say more about a shape."""

from __future__ import annotations

from .._surface import _HandleSurface
from .._waist import call
from .shape import _shape_of


class Handle(_HandleSurface):
    """What a shape call gives back: a way to say more about that shape.

    Chained rather than passed, so no function carries eighteen arguments:

        scene.rect("bar", h=40, at=cm.at(bottom=0)).fill("blue").turn(30)

    Each call returns the handle again, and each one is small enough to read.
    Every method is worked out by the Authoring Core (the generated methods of
    `_HandleSurface`); this keeps the answers.
    """

    def __init__(self, shapes: dict, name: str, marks: dict = None):
        self._shapes = shapes
        self._name = name
        self._marks = {} if marks is None else marks

    def _shape(self) -> dict:
        # A shallow copy of its fields: everything in it is already plain data,
        # and `asdict`'s recursive copy cost more than the crossing itself.
        return dict(vars(self._shapes[self._name]))

    def _change(self, op: str, data: dict) -> None:
        """Change this shape with the Authoring Core and keep the result.

        A marked shape (a marker, or a highlight's words) goes with its mark
        and its partner, and every shape and mark the Core changes is kept.
        """
        mark = self._marks.get(self._name)
        if mark is None:
            self._shapes[self._name] = _shape_of(call(op, data))
            return
        partner = mark.get("band") or mark.get("words")
        names = [self._name] + ([partner] if partner in self._shapes else [])
        answer = call("marked", {
            "op": op, "name": self._name, "data": data,
            "shapes": {n: dict(vars(self._shapes[n])) for n in names},
            "marks": {n: self._marks[n] for n in names if n in self._marks},
        })
        for name, shape in answer["shapes"].items():
            self._shapes[name] = _shape_of(shape)
        self._marks.update(answer["marks"])

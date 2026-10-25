"""Group: where shapes are drawn, each placed relative to the group's point."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .._surface import _GroupSurface
from .._waist import _own, call
from .character import Character
from .handle import Handle
from .shape import _key, _shape_of

if TYPE_CHECKING:
    from .root import Scene


@_own(_GroupSurface)
class Group(_GroupSurface):
    """Somewhere to draw, with its own origin and its own name.

    A Group supplies a shared origin and puts its name before each child's
    name. It does not add an inherited transform; use a Character for
    connected, poseable parts.

    **Inside a Group, ``0`` is the Group's anchor point.** ``bottom=0`` stands
    a shape on it; ``top=20`` puts one 20 below it.
    """

    def __init__(self, scene: "Scene", path: tuple, ox: float, oy: float, w: float):
        self._scene = scene
        self._path = path
        self._ox = ox
        self._oy = oy
        self._w = w

    def _frame(self) -> dict:
        """This Group as the Core needs it: its name path, point and width.
        Built once: a Group never moves after it is made."""
        frame = self.__dict__.get("_frame_data")
        if frame is None:
            frame = self._frame_data = {
                "path": [_key(p) for p in self._path],
                "ox": self._ox,
                "oy": self._oy,
                "w": self._w,
            }
        return frame

    def _nest(self, frame: dict) -> "Group":
        """A Group at the frame the Authoring Core worked out."""
        group = Group(
            self._scene, tuple(frame["path"]), frame["ox"], frame["oy"], frame["w"]
        )
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

    def _keep_marked(self, answer: dict):
        """Keep a marker (one shape and its stroke) or a highlight (band and
        words, and their marks) the Core drew."""
        self._scene._markers.update(answer.get("marks", {}))
        if "shape" in answer:
            handle = self._keep(_shape_of(answer["shape"]))
            self._scene._markers[handle._name] = answer["mark"]
            return handle
        return self._adopt(answer["shapes"])

    def _build(self, op: str, data: dict) -> "Handle":
        """Draw with the Authoring Core: it places the shape, this keeps it."""
        return self._keep(_shape_of(call(op, data)))

    def _build_character(self, op: str, data: dict) -> Character:
        """Draw one character Item and return its pose handle."""
        handle = self._keep(_shape_of(call(op, data)))
        return Character(self._scene._shapes, handle._name, self._scene._markers)

    # -- shapes ---------------------------------------------------------------
    #
    # rect, circle, text, line, arrow, polygon, curve, arc, formula, image, svg,
    # marker, highlight and group are generated from api/surface.toml into
    # `_GroupSurface`, and drawn by the Authoring Core. Each takes only what
    # makes it that shape, plus where it goes; everything optional is said
    # afterwards on the Handle it returns:
    #
    #     scene.circle("bob", r=28, at=(x, y)).fill("orange").on(layer=4)

"""The flat shape payload, and how a shape's name and place are read."""

from __future__ import annotations

from dataclasses import dataclass

from ..layout import Slot

PIVOTS = ("center", "top", "bottom", "left", "right")


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
    font: str = ""
    pose: str = ""
    layer: int = 0
    opacity: float = 1.0
    scale_x: float = 1.0
    scale_y: float = 1.0
    rotate: float = 0.0
    pivot: str = "center"


def _shape_of(fields: dict) -> _Shape:
    """A shape from the Authoring Core's answer. Its points arrive as a list
    and are kept as a tuple, as everywhere else, so two shapes compare alike."""
    fields.setdefault("pose", "")
    if fields["points"]:
        fields["points"] = tuple(fields["points"])
    else:
        fields["points"] = ()
    return _Shape(**fields)


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

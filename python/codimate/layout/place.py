"""Placing things: points, edges, Slots, and rows and columns of them."""

from __future__ import annotations

from dataclasses import dataclass
from .canvas import height, width


@dataclass(frozen=True)
class Place:
    """Where a shape goes, as one value.

    Built by :func:`at`. Exists so a shape takes one placement argument
    instead of six — the six were one idea wearing a disguise.
    """

    x: "float | None" = None
    y: "float | None" = None
    top: "float | None" = None
    bottom: "float | None" = None


def at(x=None, y=None, top=None, bottom=None) -> Place:
    """A place, for a shape's ``at=``.

        scene.rect("bar", h=40, at=cm.at(bottom=0))
        scene.text("l", "hi", at=cm.at(x=col, top=y + 22))

    Give one value per axis: ``x`` or nothing for horizontal, and ``y``,
    ``top`` or ``bottom`` for vertical. A plain ``(x, y)`` or a `Slot` works
    wherever a Place does, so you only need this for edges.
    """
    if y is not None and (top is not None or bottom is not None):
        raise ValueError("give one of y=, top= or bottom=")
    return Place(x=x, y=y, top=top, bottom=bottom)


@dataclass(frozen=True)
class Slot:
    """A place to put something. Not a shape — nothing draws a Slot.

    `row` and `column` hand you these; you rarely build one.

    - `x`, `y` — its centre
    - `w`, `h` — its size
    - `left`, `right`, `top`, `bottom` — its edges
    - `anchor` — which edge things placed here line up on
    """

    x: float
    y: float
    w: float
    h: float
    anchor: str = "center"

    @property
    def left(self) -> float:
        return self.x - self.w / 2

    @property
    def right(self) -> float:
        return self.x + self.w / 2

    @property
    def top(self) -> float:
        return self.y - self.h / 2

    @property
    def bottom(self) -> float:
        return self.y + self.h / 2

    def point(self, anchor: "str | None" = None) -> "tuple[float, float]":
        """The single point this Slot anchors things at."""
        anchor = anchor or self.anchor
        if anchor == "center":
            return (self.x, self.y)
        if anchor == "bottom":
            return (self.x, self.bottom)
        if anchor == "top":
            return (self.x, self.top)
        raise ValueError(f"unknown anchor {anchor!r} — use center, top or bottom")


# ponytail: `axes` still lays itself out here; these two move with it.
def _within(box) -> "Slot":
    """The region a run of Slots divides up — the whole canvas by default."""
    if box is not None:
        return box
    return Slot(x=width() / 2, y=height() / 2, w=width(), h=height())


def _size(size):
    """A Slot size: one number for both sides, or `(w, h)`."""
    if size is None:
        return None, None
    if isinstance(size, (int, float)):
        return float(size), None
    return size


from .._surface import column, row  # noqa: E402, F401  (worked out by the Authoring Core)

_UNSET = object()


def _resolve(given, half, names, default=_UNSET):
    """Turn whichever anchor was given into a centre coordinate."""
    centre, low, high = given
    given = [v for v in given if v is not None]
    if len(given) > 1:
        raise ValueError(f"give only one of {', '.join(f'{n}=' for n in names)}")
    if not given:
        if default is _UNSET:
            raise ValueError(f"give one of {', '.join(f'{n}=' for n in names)}")
        return float(default)
    if centre is not None:
        return float(centre)
    if low is not None:
        return float(low) + half
    return float(high) - half
